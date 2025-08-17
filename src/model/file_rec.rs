use crate::exif::exif;

#[derive(Debug, Clone)]
pub(crate) struct FileRec {
    exif_info: exif::ExifInfo,
    selected: bool,
}

impl FileRec {
    pub fn new(exif_info: exif::ExifInfo) -> Self {
        Self {
            exif_info,
            selected: false,
        }
    }

    pub fn has_lens_data(&self) -> bool {
        self.exif_info.lens_info.is_some()
            || (self.exif_info.lens_model.is_some() && self.exif_info.lens_make.is_some())
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
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

    pub fn exif_info(&self) -> &crate::exif::exif::ExifInfo {
        &self.exif_info
    }
}
