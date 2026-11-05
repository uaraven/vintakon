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
}
