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
}
