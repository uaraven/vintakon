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
    let exposure_compensation = entry
        .get("ExposureCompensation")
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

#[cfg(test)]
mod exiftool_test {
    use crate::exif::{exif::LensInfo, exiftool::parse_exif_entry};

    const TEST_EXIFTOOL_JSON: &str = r#"{
    "SourceFile": "DSCF3505.jpg",
    "FileModifyDate": "2025:08:10 14:01:05-04:00",
    "Make": "FUJIFILM",
    "Model": "X-T5",
    "LensMake": "SIGMA",
    "LensModel": "SIGMA 30mm F1.4 DC DN | Contemporary 016",
    "LensInfo": "30 30 1.4 1.4",
    "ExposureCompensation": "0",
    "ISO": "125",
    "Aperture": "7.1",
    "ShutterSpeed": "1/160",
    "FocalLength": "30.0 mm",
    "FocalLengthIn35mmFormat": "45 mm",
    "ThumbnailImage": "base64:/9j/2wCEAAYEBAQFBAYFBQYJBgUGCQsIBgYICwwKCgsKCgwQDAwMDAwMEAwMDAwM DAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwBBwcHDQwNGBAQGBQODg4UFA4ODg4UEQwMDAwMEREMDAwMDAwRDAwMDAwMDAwM DAwMDAwMDAwMDAwMDAwMDAwMDP/dAAQAIP/uAA5BZG9iZQBkwAAAAAH/wAARCACdAQADABEAAREBAhEB/8QBogAAAAcB AQEBAQAAAAAAAAAABAUDAgYBAAcICQoLAQACAgMBAQEBAQAAAAAAAAABAAIDBAUGBwgJCgsQAAIBAwMCBAIGBwMEAgYC cwECAxEEAAUhEjFBUQYTYSJxgRQykaEHFbFCI8FS0eEzFmLwJHKC8SVDNFOSorJjc8I1RCeTo7M2F1RkdMPS4ggmgwkK GBmElEVGpLRW01UoGvLj88TU5PRldYWVpbXF1eX1ZnaGlqa2xtbm9jdHV2d3h5ent8fX5/c4SFhoeIiYqLjI2Oj4KTlJ WWl5iZmpucnZ6fkqOkpaanqKmqq6ytrq+hEAAgIBAgMFBQQFBgQIAwNtAQACEQMEIRIxQQVRE2EiBnGBkTKhsfAUwdHh I0IVUmJy8TMkNEOCFpJTJaJjssIHc9I14kSDF1STCAkKGBkmNkUaJ2R0VTfyo7PDKCnT4/OElKS0xNTk9GV1hZWltcXV 5fVGVmZ2hpamtsbW5vZHV2d3h5ent8fX5/c4SFhoeIiYqLjI2Oj4OUlZaXmJmam5ydnp+So6SlpqeoqaqrrK2ur6/9oA DAMAAAERAhEAPwDjPDFXcMVb4Yq7hiruGKt8MVdwxVvhirfDFXcMVdwxV3DFXcMVdwxV3DFW+GKu4Yq7hiruGKtcMVdw xVwTFW+GKt8MVdxxV3HFXccVdxxVsDFW+OKrSMVa44q//9Dj/DFW+GKt8MVdwxV3DFW+GKt+nirfp4q708Vb9PFXenir fp4q708Vd6eKu9LFWvTxV3p4q708VdwxV3DFXcMVdwxV3DFXccVdwxV3DFXcMVdxxV3HFWqHFWuOKu4Yq//R5P6eKt+n irvTxVvhirYjxVv08VbEWKrhFirfpYq36WKu9HFXelirfpYq16WKu9LFWvSxV3pYq16eKtenirvTxV3DFWuGKu4Yq7hi rXDFXcMVdwxV3H2xV3EeGKu9MYq708Vd6RxV/9LmPpYq36WKtiLFWxFirfo4quEOKrhD7Yq2IfbFV3o4q36OKt+jirvQ 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"
}"#;

    #[test]
    fn test_parsing() {
        let entry: serde_json::Value = serde_json::from_str(TEST_EXIFTOOL_JSON).unwrap();
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
    }
}
