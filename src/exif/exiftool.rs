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

    // ---- entry parsing ----

    use super::{ExifInfo, apply_profile, read_files};
    use crate::exif::info::ExifChanges;
    use crate::model::lens_profile::LensProfile;
    use crate::test_util::{TEST_JPG, copy_test_jpg, temp_dir};
    use chrono::{DateTime, Utc};
    use serde_json::json;

    fn parse(entry: serde_json::Value) -> Option<ExifInfo> {
        parse_exif_entry(&entry)
    }

    #[test]
    fn entry_without_source_file_is_ignored() {
        assert!(parse(json!({"Make": "SONY"})).is_none());
    }

    #[test]
    fn modification_date_is_converted_to_utc() {
        let info = parse(json!({
            "SourceFile": "a.jpg",
            "FileModifyDate": "2026:10:29 21:33:18+02:00"
        }))
        .unwrap();
        assert_eq!(info.mod_time.to_rfc3339(), "2026-10-29T19:33:18+00:00");
    }

    #[test]
    fn malformed_modification_date_falls_back_to_the_file_system() {
        let info = parse(json!({"SourceFile": TEST_JPG, "FileModifyDate": "yesterday"})).unwrap();
        let modified = std::fs::metadata(TEST_JPG).unwrap().modified().unwrap();
        assert_eq!(info.mod_time, DateTime::<Utc>::from(modified));
    }

    #[test]
    fn entry_without_date_for_a_missing_file_is_ignored() {
        assert!(parse(json!({"SourceFile": "test-data/img/no-such-file.jpg"})).is_none());
    }

    #[test]
    fn missing_tags_are_none() {
        let info = parse(json!({"SourceFile": TEST_JPG})).unwrap();
        assert!(info.make.is_none() && info.lens_info.is_none() && info.preview_image.is_none());
    }

    #[test]
    fn preview_is_decoded_from_base64_with_or_without_prefix() {
        let with_prefix = parse(json!({"SourceFile": TEST_JPG, "PreviewImage": "base64:YWJj"}));
        assert_eq!(with_prefix.unwrap().preview_image.unwrap(), b"abc");
        let without_prefix = parse(json!({"SourceFile": TEST_JPG, "PreviewImage": "YWJj"}));
        assert_eq!(without_prefix.unwrap().preview_image.unwrap(), b"abc");
    }

    #[test]
    fn preview_tags_are_used_in_order_of_priority() {
        let all = parse(json!({
            "SourceFile": TEST_JPG,
            "ThumbnailImage": "base64:dGh1bWI=",
            "JpgFromRaw": "base64:cmF3",
            "PreviewImage": "base64:cHJldmlldw=="
        }));
        assert_eq!(all.unwrap().preview_image.unwrap(), b"preview");

        let no_preview = parse(json!({
            "SourceFile": TEST_JPG,
            "ThumbnailImage": "base64:dGh1bWI=",
            "JpgFromRaw": "base64:cmF3"
        }));
        assert_eq!(no_preview.unwrap().preview_image.unwrap(), b"raw");
    }

    #[test]
    fn invalid_base64_preview_is_ignored() {
        let info = parse(json!({"SourceFile": TEST_JPG, "PreviewImage": "base64:!!!"})).unwrap();
        assert!(info.preview_image.is_none());
    }

    // ---- preview decoding ----

    #[test]
    fn preview_without_embedded_image_is_decoded_from_the_jpeg_and_downscaled() {
        let info = parse(json!({"SourceFile": TEST_JPG})).unwrap();
        let image = decode_preview(&info).expect("a plain JPEG can be used as a preview");
        assert!(image.width() <= 1600 && image.height() <= 1600);
        assert!(image.width() == 1600 || image.height() == 1600);
    }

    #[test]
    fn undecodable_embedded_preview_falls_back_to_the_jpeg() {
        let info = parse(json!({"SourceFile": TEST_JPG, "PreviewImage": "base64:YWJj"})).unwrap();
        assert!(decode_preview(&info).is_some());
    }

    #[test]
    fn small_embedded_preview_is_not_resized() {
        // Valid 2x1 PNG (red and blue pixels)
        let png = "iVBORw0KGgoAAAANSUhEUgAAAAIAAAABCAIAAAB7QOjdAAAADUlEQVR4nGP4zwAE/wEHAAH/4iOeWQAAAABJRU5ErkJggg==";
        let info = parse(json!({
            "SourceFile": "a.arw",
            "FileModifyDate": "2026:10:29 21:33:18+00:00",
            "PreviewImage": format!("base64:{png}")
        }));
        let image = decode_preview(&info.unwrap()).expect("embedded PNG must be decoded");
        assert_eq!((image.width(), image.height()), (2, 1));
    }

    #[test]
    fn raw_file_without_a_preview_has_none() {
        let info = parse(json!({"SourceFile": "test-data/img/_DSC3006.ARW"})).unwrap();
        assert!(decode_preview(&info).is_none());
    }

    // ---- writing ----

    fn profile() -> LensProfile {
        LensProfile {
            name: "Helios".to_string(),
            lens_make: "Helios".to_string(),
            lens_model: "44-2 58mm f/2".to_string(),
            min_focal_length: 58.0,
            max_focal_length: 58.0,
            min_aperture: "2".to_string(),
            max_aperture: "16".to_string(),
            crop_factor: 1.5,
        }
    }

    #[test]
    fn applied_profile_can_be_read_back() {
        let dir = temp_dir("apply_profile");
        let file = copy_test_jpg(&dir, "photo.jpg");
        let dir_name = dir.to_str().unwrap();

        let changes = ExifChanges::new(&profile(), None, Some("2.8".to_string()));
        apply_profile(
            None,
            dir_name,
            vec![file.to_str().unwrap().to_string()],
            changes,
        )
        .expect("exiftool must write the tags");

        let infos = read_files(dir_name, None).unwrap();
        // exiftool keeps `photo.jpg_original` next to the file, it must not be listed
        assert_eq!(infos.len(), 1);
        let info = &infos[0];
        assert_eq!(info.lens_make.as_deref(), Some("Helios"));
        assert_eq!(info.lens_model.as_deref(), Some("44-2 58mm f/2"));
        assert_eq!(info.aperture.as_deref(), Some("2.8"));
        assert!(info.focal_length.as_deref().unwrap().starts_with("58"));
        assert!(info.focal_length_35mm.as_deref().unwrap().starts_with("87"));
        assert!(info.lens_info.is_some());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn applying_a_profile_twice_overwrites_the_values() {
        let dir = temp_dir("apply_profile_twice");
        let file = copy_test_jpg(&dir, "photo.jpg");
        let dir_name = dir.to_str().unwrap();
        let files = vec![file.to_str().unwrap().to_string()];

        let first = ExifChanges::new(&profile(), None, None);
        apply_profile(None, dir_name, files.clone(), first).unwrap();
        let mut other = profile();
        other.lens_model = "Jupiter-9".to_string();
        apply_profile(None, dir_name, files, ExifChanges::new(&other, None, None)).unwrap();

        let infos = read_files(dir_name, None).unwrap();
        assert_eq!(infos[0].lens_model.as_deref(), Some("Jupiter-9"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn applying_a_profile_to_a_broken_file_reports_the_error() {
        let dir = temp_dir("apply_profile_broken");
        std::fs::write(dir.join("bad.jpg"), "not a jpeg").unwrap();

        let changes = ExifChanges::new(&profile(), None, None);
        let error = apply_profile(
            None,
            dir.to_str().unwrap(),
            vec![dir.join("bad.jpg").to_str().unwrap().to_string()],
            changes,
        )
        .unwrap_err();

        assert!(error.contains("bad.jpg"), "{error}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn applying_a_profile_reports_missing_exiftool() {
        let dir = temp_dir("apply_profile_no_exiftool");
        let changes = ExifChanges::new(&profile(), None, None);
        let error = apply_profile(
            Some("/nonexistent/exiftool".to_string()),
            dir.to_str().unwrap(),
            vec!["a.jpg".to_string()],
            changes,
        )
        .unwrap_err();
        assert!(error.contains("Failed to run exiftool"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn files_are_listed_in_path_order_and_other_extensions_are_skipped() {
        let dir = temp_dir("read_files_order");
        copy_test_jpg(&dir, "b.jpg");
        copy_test_jpg(&dir, "a.JPG");
        std::fs::write(dir.join("notes.txt"), "hello").unwrap();

        let infos = read_files(dir.to_str().unwrap(), None).unwrap();

        let names: Vec<&str> = infos
            .iter()
            .map(|info| info.file_path.rsplit('/').next().unwrap())
            .collect();
        assert_eq!(names, ["a.JPG", "b.jpg"]);
        let _ = std::fs::remove_dir_all(dir);
    }
}
