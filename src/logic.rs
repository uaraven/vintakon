//! UI-framework independent helpers: input validation, file dialogs and
//! the long-running exiftool operations.

use rfd::AsyncFileDialog;

use crate::{
    exif::{
        exiftool::{apply_profile, read_files},
        info::ExifChanges,
    },
    model::{file_rec::FileRec, lens_profile::LensProfile},
};

/// Checks if aperture/crop factor text is a number with an optional fractional part
/// (e.g. `1.8`, `22`, `5.`), or is empty
pub(crate) fn is_valid_floating_point(text: &str) -> bool {
    let (integer, fraction) = match text.split_once('.') {
        Some((integer, fraction)) => (integer, Some(fraction)),
        None => (text, None),
    };
    let is_digits = |part: &str| part.bytes().all(|b| b.is_ascii_digit());
    text.is_empty() || (!integer.is_empty() && is_digits(integer) && fraction.is_none_or(is_digits))
}

/// Checks if focal length text contains digits only
pub(crate) fn is_focal_length_valid(text: &str) -> bool {
    text.bytes().all(|b| b.is_ascii_digit())
}

/// Reads exif info for all supported files in the folder.
///
/// `read_files` blocks while exiftool is running, so it is moved to a blocking thread.
pub(crate) async fn load_images(
    new_path: String,
    exiftool: Option<String>,
) -> Result<Vec<FileRec>, String> {
    tokio::task::spawn_blocking(move || {
        read_files(&new_path, exiftool).map(|infos| infos.into_iter().map(FileRec::new).collect())
    })
    .await
    .map_err(|e| format!("Scanning the folder was interrupted: {e}"))?
}

/// Shows a folder selection dialog. Returns `None` if the dialog was cancelled
pub(crate) async fn select_folder() -> Option<String> {
    AsyncFileDialog::new()
        .pick_folder()
        .await
        .map(|folder| folder.path().to_string_lossy().to_string())
}

/// Shows a file selection dialog for the exiftool executable.
/// Returns `None` if the dialog was cancelled
pub(crate) async fn select_exiftool_file() -> Option<String> {
    AsyncFileDialog::new()
        .set_title("Select exiftool executable")
        .add_filter("Executable", &["exe", ""])
        .pick_file()
        .await
        .map(|file| file.path().to_string_lossy().to_string())
}

/// Writes lens profile to the given files. Returns the error message if exiftool failed
pub(crate) async fn apply_profile_to_images(
    root_path: String,
    profile: LensProfile,
    aperture: String,
    focal_length: String,
    selected_files: Vec<String>,
    exiftool: Option<String>,
) -> Result<(), String> {
    let actual_aperture = (!aperture.is_empty()).then_some(aperture);

    // The input is validated to contain digits only, so parsing can only fail on overflow
    let actual_focal_length: Option<f32> = focal_length.parse().ok();

    let changes = ExifChanges::new(&profile, actual_focal_length, actual_aperture);

    // exiftool blocks until it is done, so keep it off the UI thread
    tokio::task::spawn_blocking(move || {
        apply_profile(exiftool, root_path.as_str(), selected_files, changes)
    })
    .await
    .map_err(|e| format!("Writing the lens profile was interrupted: {e}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aperture_validator() {
        assert!(is_valid_floating_point(""));
        assert!(is_valid_floating_point("1.8"));
        assert!(is_valid_floating_point("22"));
        assert!(is_valid_floating_point("5."));
        assert!(!is_valid_floating_point("f1.8"));
        assert!(!is_valid_floating_point("1.8.2"));
        assert!(!is_valid_floating_point("-1"));
    }

    #[test]
    fn focal_length_validator() {
        assert!(is_focal_length_valid(""));
        assert!(is_focal_length_valid("50"));
        assert!(!is_focal_length_valid("50.5"));
        assert!(!is_focal_length_valid("5a"));
    }

    // ---- exiftool operations ----

    use crate::test_util::{copy_test_jpg, temp_dir};

    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(future)
    }

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
    fn load_images_wraps_the_files_of_the_folder() {
        let dir = temp_dir("logic_load");
        copy_test_jpg(&dir, "photo.jpg");

        let images = block_on(load_images(dir.to_str().unwrap().to_string(), None)).unwrap();

        assert_eq!(images.len(), 1);
        assert_eq!(images[0].file_name(), "photo.jpg");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn load_images_reports_missing_exiftool() {
        let result = block_on(load_images(
            "test-data/img".to_string(),
            Some("/nonexistent/exiftool".to_string()),
        ));
        assert!(result.unwrap_err().contains("Failed to run exiftool"));
    }

    #[test]
    fn apply_uses_actual_aperture_and_focal_length_when_given() {
        let dir = temp_dir("logic_apply_actual");
        let file = copy_test_jpg(&dir, "photo.jpg");
        let dir_name = dir.to_str().unwrap().to_string();

        block_on(apply_profile_to_images(
            dir_name.clone(),
            profile(),
            "4".to_string(),
            "85".to_string(),
            vec![file.to_str().unwrap().to_string()],
            None,
        ))
        .unwrap();

        let images = block_on(load_images(dir_name, None)).unwrap();
        let info = images[0].exif_info();
        assert_eq!(info.lens_model.as_deref(), Some("44-2 58mm f/2"));
        assert_eq!(info.aperture.as_deref(), Some("4.0"));
        assert!(info.focal_length.as_deref().unwrap().starts_with("85"));
        // 85 * 1.5 = 127.5, the tag is an integer and exiftool rounds it
        assert!(
            info.focal_length_35mm
                .as_deref()
                .unwrap()
                .starts_with("128")
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn apply_uses_the_profile_focal_length_when_actual_values_are_empty() {
        let dir = temp_dir("logic_apply_profile");
        let file = copy_test_jpg(&dir, "photo.jpg");
        let dir_name = dir.to_str().unwrap().to_string();

        block_on(apply_profile_to_images(
            dir_name.clone(),
            profile(),
            String::new(),
            String::new(),
            vec![file.to_str().unwrap().to_string()],
            None,
        ))
        .unwrap();

        let images = block_on(load_images(dir_name, None)).unwrap();
        let info = images[0].exif_info();
        assert!(info.focal_length.as_deref().unwrap().starts_with("58"));
        assert!(info.focal_length_35mm.as_deref().unwrap().starts_with("87"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn apply_reports_exiftool_errors() {
        let dir = temp_dir("logic_apply_error");
        std::fs::write(dir.join("bad.jpg"), "not a jpeg").unwrap();

        let result = block_on(apply_profile_to_images(
            dir.to_str().unwrap().to_string(),
            profile(),
            String::new(),
            String::new(),
            vec![dir.join("bad.jpg").to_str().unwrap().to_string()],
            None,
        ));

        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(dir);
    }
}
