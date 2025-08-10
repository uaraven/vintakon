use crate::exif::exif::{ExifInfo, LensInfo};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::process::Command;

fn supported_extensions() -> Vec<&'static str> {
    vec![
        "jpg", "jpeg", "raf", "crw", "cr2", "cr3", "nef", "nrw", "arw", "srf", "sr2", "orf",
        "tiff", "dng", "3fr", "gpr", "mef", "raw", "rw2", "rwl", "x3f", "srw",
    ]
}

pub(crate) fn read_files(path: &str) -> Vec<ExifInfo> {
    let mut exif_infos = Vec::new();

    // Run exiftool command to extract EXIF data as JSON
    let mut cmd = Command::new("exiftool");
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

    exif_infos
}

fn parse_exif_entry(entry: &Value) -> Option<ExifInfo> {
    let file_path = entry.get("SourceFile")?.as_str()?.to_string();

    // Parse modification date
    let mod_time = if let Some(date_str) = entry.get("ModifyDate").and_then(|v| v.as_str()) {
        DateTime::parse_from_str(date_str, "%Y:%m:%d %H:%M:%S")
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
    let exposure_compensation = entry
        .get("ExposureCompensation")
        .and_then(|v| v.as_str())
        .map(String::from);
    let iso = entry.get("ISO").and_then(|v| v.as_str()).map(String::from);
    let aperture = entry
        .get("ApertureValue")
        .and_then(|v| v.as_str())
        .map(String::from);
    let shutter_speed = entry
        .get("ShutterSpeedValue")
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
        .and_then(|s| Some(if s.starts_with("base64:") { &s[7..] } else { s }))
        .map(String::from);

    // Parse lens info
    let lens_info = entry
        .get("LensInfo")
        .and_then(|v| v.as_str())
        .and_then(LensInfo::from_string);

    Some(ExifInfo {
        file_path,
        mod_time,
        make,
        model,
        lens_make,
        lens_model,
        lens_info,
        exposure_compensation,
        iso,
        aperture,
        shutter_speed,
        focal_length,
        focal_length_35mm,
        preview_base64,
    })
}
