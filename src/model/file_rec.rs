use crate::exif::info;

#[derive(Debug, Clone)]
pub(crate) struct FileRec {
    exif_info: info::ExifInfo,
}

impl FileRec {
    pub fn new(exif_info: info::ExifInfo) -> Self {
        Self { exif_info }
    }

    pub fn has_lens_data(&self) -> bool {
        self.exif_info.lens_info.is_some()
            || (self.exif_info.lens_model.is_some() && self.exif_info.lens_make.is_some())
    }

    pub fn file_name(&self) -> String {
        std::path::Path::new(&self.exif_info.file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    pub fn mod_time(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.exif_info.mod_time
    }

    pub fn exif_info(&self) -> &crate::exif::info::ExifInfo {
        &self.exif_info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exif::info::LensInfo;

    fn rec(
        path: &str,
        lens_make: Option<&str>,
        lens_model: Option<&str>,
        lens_info: bool,
    ) -> FileRec {
        FileRec::new(info::ExifInfo {
            file_path: path.to_string(),
            mod_time: chrono::DateTime::UNIX_EPOCH,
            make: None,
            model: None,
            lens_make: lens_make.map(String::from),
            lens_model: lens_model.map(String::from),
            lens_info: lens_info.then(|| LensInfo::from_string("50 1.8").unwrap()),
            iso: None,
            aperture: None,
            shutter_speed: None,
            focal_length: None,
            focal_length_35mm: None,
            preview_image: None,
        })
    }

    #[test]
    fn lens_info_is_lens_data() {
        assert!(rec("a.jpg", None, None, true).has_lens_data());
    }

    #[test]
    fn make_and_model_together_are_lens_data() {
        assert!(rec("a.jpg", Some("Helios"), Some("44-2"), false).has_lens_data());
    }

    #[test]
    fn make_or_model_alone_is_not_lens_data() {
        assert!(!rec("a.jpg", Some("Helios"), None, false).has_lens_data());
        assert!(!rec("a.jpg", None, Some("44-2"), false).has_lens_data());
        assert!(!rec("a.jpg", None, None, false).has_lens_data());
    }

    #[test]
    fn file_name_is_the_last_path_component() {
        assert_eq!(
            rec("/photos/2026/IMG_0001.jpg", None, None, false).file_name(),
            "IMG_0001.jpg"
        );
    }

    #[test]
    fn file_name_of_an_empty_path_is_unknown() {
        assert_eq!(rec("", None, None, false).file_name(), "Unknown");
    }
}
