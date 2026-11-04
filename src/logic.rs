//! UI-framework independent helpers: input validation, file dialogs and
//! the long-running exiftool operations.

use std::sync::LazyLock;

use regex::Regex;
use rfd::AsyncFileDialog;

use crate::{
    exif::{
        exif::ExifChanges,
        exiftool::{apply_profile, read_files},
    },
    model::{file_rec::FileRec, lens_profile::LensProfile},
};

static APERTURE_VALIDATOR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+(\.\d*)?)?$").unwrap());
static FL_VALIDATOR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d*$").unwrap());

/// Checks if aperture/crop factor text is matching `^(\d+(\.\d*)?)?$`
pub(crate) fn is_valid_floating_point(text: &str) -> bool {
    APERTURE_VALIDATOR.is_match(text)
}

/// Checks if focal length text contains digits only
pub(crate) fn is_focal_length_valid(text: &str) -> bool {
    FL_VALIDATOR.is_match(text)
}

/// Reads exif info for all supported files in the folder.
///
/// `read_files` blocks while exiftool is running, so it is moved to a blocking thread.
pub(crate) async fn load_images(new_path: String, exiftool: Option<String>) -> Vec<FileRec> {
    let path = new_path.clone();
    tokio::task::spawn_blocking(move || {
        read_files(&path, exiftool)
            .into_iter()
            .map(FileRec::new)
            .collect()
    })
    .await
    .unwrap_or_default()
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

/// Writes lens profile to all checked images
pub(crate) async fn apply_profile_to_images(
    root_path: String,
    profile: LensProfile,
    aperture: String,
    focal_length: String,
    images: Vec<FileRec>,
    exiftool: Option<String>,
) {
    let actual_aperture = if !aperture.is_empty() {
        Some(aperture)
    } else {
        None
    };

    // The input is validated to contain digits only, so parsing can only fail on overflow
    let actual_focal_length: Option<f32> = focal_length.parse().ok();

    let selected_files: Vec<String> = images
        .iter()
        .filter(|file_rec| file_rec.is_selected())
        .map(|file_rec| file_rec.exif_info().file_path.clone())
        .collect();

    let changes = ExifChanges::new(&profile, actual_focal_length, actual_aperture);

    apply_profile(exiftool, root_path.as_str(), selected_files, changes).await;
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
}
