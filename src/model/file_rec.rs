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
