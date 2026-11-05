use crate::exif::info::{ExifChanges, ExifInfo, LensInfo};
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use image::DynamicImage;
use serde_json::Value;
use std::path::Path;
use std::process::Command;

const SUPPORTED_EXTENSIONS: [&str; 22] = [
    "jpg", "jpeg", "raf", "crw", "cr2", "cr3", "nef", "nrw", "arw", "srf", "sr2", "orf", "tiff",
    "dng", "3fr", "gpr", "mef", "raw", "rw2", "rwl", "x3f", "srw",
];

/// Previews are downscaled to fit into a square of this size: it is enough for the preview pane
/// and keeps the bitmap (which is kept in memory and uploaded to the GPU) small
const MAX_PREVIEW_SIZE: u32 = 1600;

/// Tags that can contain an embedded preview, the first existing one is used
const PREVIEW_TAGS: [&str; 3] = ["PreviewImage", "JpgFromRaw", "ThumbnailImage"];

/// `exiftool` from the PATH is used unless the path to the program is configured
fn exiftool_command(program: Option<String>) -> Command {
    Command::new(program.unwrap_or_else(|| "exiftool".to_string()))
}

/// Reads exif info of all supported files in the folder. Blocks while exiftool is running.
/// Returns the error message if exiftool could not be run or its output is not understood
pub(crate) fn read_files(
    path: &str,
    exiftool_program: Option<String>,
) -> Result<Vec<ExifInfo>, String> {
    // Run exiftool command to extract EXIF data as JSON
    let mut cmd = exiftool_command(exiftool_program);
    cmd.args(["-j", "-b", "-q", "-api", "structformat=jsonq"])
        .args([
            "-FileModifyDate",
            "-Make",
            "-Model",
            "-LensMake",
            "-LensModel",
            "-LensInfo#",
            "-ExposureCompensation",
            "-ISO",
            "-Aperture",
            "-ShutterSpeed",
            "-FocalLength",
            "-FocalLengthIn35mmFormat",
            "-PreviewImage",
            "-JpgFromRaw",
            "-ThumbnailImage",
        ]);
    for ext in SUPPORTED_EXTENSIONS {
        cmd.arg("-ext").arg(ext);
    }
    cmd.arg(path);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run exiftool: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "exiftool failed with status {}:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    // exiftool prints nothing at all when the folder has no supported files
    if output.stdout.trim_ascii().is_empty() {
        return Ok(Vec::new());
    }

    let entries: Vec<Value> = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse the output of exiftool: {e}"))?;

    let mut exif_infos: Vec<ExifInfo> = entries.iter().filter_map(parse_exif_entry).collect();
    exif_infos.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    Ok(exif_infos)
}

fn parse_exif_entry(entry: &Value) -> Option<ExifInfo> {
    let str_field = |name: &str| entry.get(name).and_then(Value::as_str).map(String::from);

    let file_path = str_field("SourceFile")?;

    // Modification date from the tag, or from the file system if the tag is missing or malformed
    let mod_time = str_field("FileModifyDate")
        .and_then(|date| DateTime::parse_from_str(&date, "%Y:%m:%d %H:%M:%S%z").ok())
        .map(|date| date.with_timezone(&Utc))
        .or_else(|| {
            let modified = std::fs::metadata(&file_path).ok()?.modified().ok()?;
            Some(DateTime::<Utc>::from(modified))
        })?;

    let preview_image = PREVIEW_TAGS
        .iter()
        .find_map(|tag| entry.get(tag))
        .and_then(Value::as_str)
        .map(|data| data.strip_prefix("base64:").unwrap_or(data))
        .and_then(|data| {
            general_purpose::STANDARD
                .decode(data)
                .inspect_err(|e| eprintln!("Failed to decode base64 preview image: {e}"))
                .ok()
        });

    Some(ExifInfo {
        mod_time,
        make: str_field("Make"),
        model: str_field("Model"),
        lens_make: str_field("LensMake"),
        lens_model: str_field("LensModel"),
        lens_info: entry
            .get("LensInfo")
            .and_then(Value::as_str)
            .and_then(LensInfo::from_string),
        iso: str_field("ISO"),
        aperture: str_field("Aperture"),
        shutter_speed: str_field("ShutterSpeed"),
        focal_length: str_field("FocalLength"),
        focal_length_35mm: str_field("FocalLengthIn35mmFormat"),
        preview_image,
        file_path,
    })
}

/// Decodes the preview of the file: the embedded one or, for the files without it
/// (e.g. edited JPEGs), the image itself. The result is downscaled
pub(crate) fn decode_preview(exif_info: &ExifInfo) -> Option<DynamicImage> {
    let image = exif_info
        .preview_image
        .as_deref()
        .and_then(|data| image::load_from_memory(data).ok())
        .or_else(|| load_plain_image(&exif_info.file_path))?;
    if image.width() > MAX_PREVIEW_SIZE || image.height() > MAX_PREVIEW_SIZE {
        Some(image.thumbnail(MAX_PREVIEW_SIZE, MAX_PREVIEW_SIZE))
    } else {
        Some(image)
    }
}

/// Decodes a plain JPEG file
fn load_plain_image(file_path: &str) -> Option<DynamicImage> {
    let ext = Path::new(file_path).extension()?.to_str()?.to_lowercase();
    if ext != "jpg" && ext != "jpeg" {
        return None;
    }
    image::open(file_path).ok()
}

/// Writes the changes to the files. Blocks while exiftool is running.
/// Returns the error message if exiftool could not be started or reported errors
pub fn apply_profile(
    exiftool_program: Option<String>,
    dir: &str,
    file_names: Vec<String>,
    changes: ExifChanges,
) -> Result<(), String> {
    let mut cmd = exiftool_command(exiftool_program);
    cmd.current_dir(dir);
    cmd.arg("-q");
    cmd.args(changes.to_exif_tool());
    for file_name in file_names {
        let rel_name = Path::new(file_name.as_str());
        let rel_name = rel_name.strip_prefix(dir).unwrap_or(rel_name);
        cmd.arg(rel_name);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run exiftool: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("{stdout}");
    println!("{stderr}");

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "exiftool failed with status {}:\n{}",
            output.status,
            stderr.trim()
        ))
    }
}

#[cfg(test)]
mod exiftool_test {
    use crate::exif::{
        exiftool::{decode_preview, parse_exif_entry},
        info::LensInfo,
    };
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
        let preview = decode_preview(&exif);
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

        // Verify that the base64 image was extracted and can be decoded
        assert!(
            exif.preview_image.is_some(),
            "Preview image should be decoded from base64"
        );
        let image = preview.expect("Preview should be decodable");
        assert!(image.width() > 0, "Image should have positive width");
        assert!(image.height() > 0, "Image should have positive height");
    }

    #[test]
    fn test_parsing_zoom() {
        let entry: serde_json::Value = load_test_data("moon");
        let exif = parse_exif_entry(&entry);

        assert!(exif.is_some());
        let exif = exif.unwrap();
        let preview = decode_preview(&exif);
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

        // Verify that the base64 image was extracted and can be decoded
        assert!(
            exif.preview_image.is_some(),
            "Preview image should be decoded from base64"
        );
        let image = preview.expect("Preview should be decodable");
        assert!(image.width() > 0, "Image should have positive width");
        assert!(image.height() > 0, "Image should have positive height");
    }

    #[test]
    fn test_read_files() {
        use super::read_files;

        let exif_infos = read_files("test-data/img", None).unwrap();

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

    #[test]
    fn test_read_files_reports_missing_exiftool() {
        use super::read_files;

        let result = read_files("test-data/img", Some("/nonexistent/exiftool".to_string()));
        assert!(result.unwrap_err().contains("Failed to run exiftool"));
    }

    #[test]
    fn test_read_files_empty_folder() {
        use super::read_files;

        let dir = std::env::temp_dir().join("vintakon_empty_folder_test");
        std::fs::create_dir_all(&dir).unwrap();
        let result = read_files(dir.to_str().unwrap(), None);
        std::fs::remove_dir(&dir).unwrap();
        assert!(result.unwrap().is_empty());
    }
}
