//! Helpers shared by the tests.

use std::path::{Path, PathBuf};

/// JPEG with the exif data, it is used as the source of the test files
pub(crate) const TEST_JPG: &str = "test-data/img/_DSC2713.jpg";

/// Creates a new empty directory for a test
pub(crate) fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vintakon_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("Failed to create the test directory");
    dir
}

/// Puts a copy of the test JPEG into the directory and returns its path
pub(crate) fn copy_test_jpg(dir: &Path, name: &str) -> PathBuf {
    let target = dir.join(name);
    std::fs::copy(TEST_JPG, &target).expect("Failed to copy the test image");
    target
}
