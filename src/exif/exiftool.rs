use crate::exif::exif::{ExifChanges, ExifInfo, LensInfo};
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use image::io::Reader as ImageReader;
use serde_json::Value;
use std::io::Cursor;
use std::path::Path;
use std::process::Command;

fn supported_extensions() -> Vec<&'static str> {
    vec![
        "jpg", "jpeg", "raf", "crw", "cr2", "cr3", "nef", "nrw", "arw", "srf", "sr2", "orf",
        "tiff", "dng", "3fr", "gpr", "mef", "raw", "rw2", "rwl", "x3f", "srw",
    ]
}

pub(crate) fn read_files(path: &str, exiftool_program: Option<String>) -> Vec<ExifInfo> {
    let mut exif_infos = Vec::new();

    let exif_tool_program = match exiftool_program {
        Some(program) => program,
        None => "exiftool".to_string(),
    };

    // Run exiftool command to extract EXIF data as JSON
    let mut cmd = Command::new(exif_tool_program);
    cmd.arg("-j")
        .arg("-b")
        .arg("-q")
        .arg("-api")
        .arg("structformat=jsonq")
        .arg("-FileModifyDate")
        .arg("-Make")
        .arg("-Model")
        .arg("-LensMake")
        .arg("-LensModel")
        .arg("-LensInfo#")
        .arg("-ExposureCompensation")
        .arg("-ISO")
        .arg("-Aperture")
        .arg("-ShutterSpeed")
        .arg("-FocalLength")
        .arg("-FocalLengthIn35mmFormat")
        .arg("-PreviewImage")
        .arg("-JpgFromRaw")
        .arg("-ThumbnailImage");

    for ext in supported_extensions() {
        cmd.arg("-ext").arg(ext);
    }
    cmd.arg(path);

    let output = match cmd.output() {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Failed to run exiftool: {}", e);
            return exif_infos;
        }
    };

    if !output.status.success() {
        eprintln!("exiftool failed with status: {}", output.status);
        return exif_infos;
    }

    let json_str = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to parse exiftool output as UTF-8: {}", e);
            return exif_infos;
        }
    };

    // Parse JSON response
    let json_data: Value = match serde_json::from_str(&json_str) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to parse JSON from exiftool: {}", e);
            return exif_infos;
        }
    };

    // Process each file entry
    if let Some(array) = json_data.as_array() {
        for entry in array {
            if let Some(exif_info) = parse_exif_entry(entry) {
                exif_infos.push(exif_info);
            }
        }
    }

    exif_infos.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    exif_infos
}

fn parse_exif_entry(entry: &Value) -> Option<ExifInfo> {
    let file_path = entry.get("SourceFile")?.as_str()?.to_string();

    // Parse modification date
    let mod_time = if let Some(date_str) = entry.get("FileModifyDate").and_then(|v| v.as_str()) {
        DateTime::parse_from_str(date_str, "%Y:%m:%d %H:%M:%S%z")
            .ok()?
            .with_timezone(&Utc)
    } else {
        // Fallback to file system metadata
        let metadata = std::fs::metadata(&file_path).ok()?;
        let system_time = metadata.modified().ok()?;
        DateTime::from(system_time)
    };

    // Extract optional string fields
    let make = entry.get("Make").and_then(|v| v.as_str()).map(String::from);
    let model = entry
        .get("Model")
        .and_then(|v| v.as_str())
        .map(String::from);
    let lens_make = entry
        .get("LensMake")
        .and_then(|v| v.as_str())
        .map(String::from);
    let lens_model = entry
        .get("LensModel")
        .and_then(|v| v.as_str())
        .map(String::from);
    let iso = entry.get("ISO").and_then(|v| v.as_str()).map(String::from);
    let aperture = entry
        .get("Aperture")
        .and_then(|v| v.as_str())
        .map(String::from);
    let shutter_speed = entry
        .get("ShutterSpeed")
        .and_then(|v| v.as_str())
        .map(String::from);
    let focal_length = entry
        .get("FocalLength")
        .and_then(|v| v.as_str())
        .map(String::from);
    let focal_length_35mm = entry
        .get("FocalLengthIn35mmFormat")
        .and_then(|v| v.as_str())
        .map(String::from);
    let preview_base64 = entry
        .get("PreviewImage")
        .or_else(|| entry.get("JpgFromRaw"))
        .or_else(|| entry.get("ThumbnailImage"))
        .and_then(|v| v.as_str())
        .map(|s| s.strip_prefix("base64:").unwrap_or(s))
        .map(String::from);

    // Parse lens info
    let lens_info = entry
        .get("LensInfo")
        .and_then(|v| v.as_str())
        .and_then(LensInfo::from_string);

    // Decode base64 image data if present
    let preview_image = preview_base64.as_ref().and_then(|base64_str| {
        match general_purpose::STANDARD.decode(base64_str) {
            Ok(image_data) => {
                let cursor = Cursor::new(image_data);
                match ImageReader::new(cursor).with_guessed_format() {
                    Ok(reader) => reader.decode().ok(),
                    Err(_) => None,
                }
            }
            Err(e) => {
                eprintln!("Failed to decode base64 preview image: {}", e);
                None
            }
        }
    });

    // Files without an embedded preview (e.g. edited JPEGs): decode the image itself
    let preview_image = preview_image.or_else(|| load_plain_image(&file_path));

    Some(ExifInfo {
        file_path,
        mod_time,
        make,
        model,
        lens_make,
        lens_model,
        lens_info,
        iso,
        aperture,
        shutter_speed,
        focal_length,
        focal_length_35mm,
        preview_image,
    })
}

/// Decodes a plain JPEG file (downscaled) to use as a preview
fn load_plain_image(file_path: &str) -> Option<image::DynamicImage> {
    let ext = Path::new(file_path).extension()?.to_str()?.to_lowercase();
    if ext != "jpg" && ext != "jpeg" {
        return None;
    }
    let img = ImageReader::open(file_path).ok()?.decode().ok()?;
    Some(img.thumbnail(1600, 1600))
}

pub async fn apply_profile(
    exiftool_program: Option<String>,
    dir: &str,
    file_names: Vec<String>,
    changes: ExifChanges,
) {
    let exif_tool_program = match exiftool_program {
        Some(program) => program,
        None => "exiftool".to_string(),
    };

    let mut cmd = Command::new(exif_tool_program);
    cmd.current_dir(dir);
    cmd.arg("-q");
    cmd.args(changes.to_exif_tool());
    for file_name in file_names {
        let rel_name = Path::new(file_name.as_str());
        let rel_name = rel_name.strip_prefix(dir).unwrap_or(rel_name);
        cmd.arg(rel_name);
    }
    // cmd.get_args().for_each(|f| println!("{} ", f.display()));

    match cmd.output() {
        Ok(output) => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => {
            eprintln!("Failed to run exiftool: {}", e);
        }
    };
}

#[cfg(test)]
mod exiftool_test {
    use crate::exif::{exif::LensInfo, exiftool::parse_exif_entry};
    fn load_test_data(name: &str) -> serde_json::Value {
        let path = format!("test-data/{}.json", name);
        let data = std::fs::read_to_string(path).expect("Failed to read test data");
        serde_json::from_str(&data).expect("Failed to parse test data")
    }

    #[test]
    fn test_parsing_prime() {
        let entry: serde_json::Value = load_test_data("fuji_30mm");
        let exif = parse_exif_entry(&entry);

        assert!(exif.is_some());
        let exif = exif.unwrap();
        assert_eq!(exif.make.unwrap(), "FUJIFILM");
        assert_eq!(exif.model.unwrap(), "X-T5");
        assert_eq!(exif.lens_make.unwrap(), "SIGMA");
        assert_eq!(
            exif.lens_model.unwrap(),
            "SIGMA 30mm F1.4 DC DN | Contemporary 016"
        );
        assert!(exif.lens_info.is_some());
        let lens_info = exif.lens_info.unwrap();
        match lens_info {
            LensInfo::Prime {
                focal_length,
                min_f,
            } => {
                assert_eq!(focal_length, "30");
                assert_eq!(min_f, "1.4");
            }
            _ => panic!("Expected Prime variant"),
        }

        // Verify that the base64 image was decoded successfully
        assert!(
            exif.preview_image.is_some(),
            "Preview image should be decoded from base64"
        );
        let image = exif.preview_image.unwrap();
        assert!(image.width() > 0, "Image should have positive width");
        assert!(image.height() > 0, "Image should have positive height");
    }

    #[test]
    fn test_parsing_zoom() {
        let entry: serde_json::Value = load_test_data("moon");
        let exif = parse_exif_entry(&entry);

        assert!(exif.is_some());
        let exif = exif.unwrap();
        assert_eq!(exif.make.unwrap(), "SONY");
        assert_eq!(exif.model.unwrap(), "ILCE-7M4");
        assert!(exif.lens_make.is_none());
        assert_eq!(exif.lens_model.unwrap(), "FE 200-600mm F5.6-6.3 G OSS");
        assert!(exif.lens_info.is_some());
        let lens_info = exif.lens_info.unwrap();
        match lens_info {
            LensInfo::Zoom {
                min_focal_length,
                max_focal_length,
                min_f,
                max_f,
            } => {
                assert_eq!(min_focal_length, "200");
                assert_eq!(max_focal_length, "600");
                assert_eq!(min_f, "5.6");
                assert_eq!(max_f, "6.3");
            }
            _ => panic!("Expected Prime variant"),
        }

        // Verify that the base64 image was decoded successfully
        assert!(
            exif.preview_image.is_some(),
            "Preview image should be decoded from base64"
        );
        let image = exif.preview_image.unwrap();
        assert!(image.width() > 0, "Image should have positive width");
        assert!(image.height() > 0, "Image should have positive height");
    }

    #[test]
    fn test_read_files() {
        use super::read_files;

        let exif_infos = read_files("test-data/img", None);

        assert!(!exif_infos.is_empty(), "Should read at least one file");
        assert!(
            exif_infos.len() <= 2,
            "Should not read more than 2 test files"
        );

        for exif_info in &exif_infos {
            assert!(exif_info.file_path.contains("test-data/img"));
            assert!(
                exif_info.file_path.ends_with(".jpg")
                    || exif_info.file_path.ends_with(".ARW")
                    || exif_info.file_path.ends_with(".arw")
            );
        }
    }
}
