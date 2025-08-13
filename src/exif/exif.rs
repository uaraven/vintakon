use std::fmt::Display;

use chrono::DateTime;

#[derive(Debug, Clone)]
pub(crate) enum LensInfo {
    Zoom {
        min_f: String,
        max_f: String,
        min_focal_length: String,
        max_focal_length: String,
    },
    Prime {
        min_f: String,
        focal_length: String,
    },
}

impl LensInfo {
    pub fn from_string(lens_info: &str) -> Option<Self> {
        let parts: Vec<&str> = lens_info.split(' ').collect();
        if parts.len() == 4 {
            if parts[0] == parts[1] && parts[2] == parts[3] {
                Some(Self::Prime {
                    focal_length: parts[0].to_string(),
                    min_f: parts[2].to_string(),
                })
            } else {
                Some(Self::Zoom {
                    min_focal_length: parts[0].to_string(),
                    max_focal_length: parts[1].to_string(),
                    min_f: parts[2].to_string(),
                    max_f: parts[3].to_string(),
                })
            }
        } else if parts.len() == 2 {
            Some(Self::Prime {
                focal_length: parts[0].to_string(),
                min_f: parts[1].to_string(),
            })
        } else {
            None
        }
    }
}

impl Display for LensInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zoom {
                min_f,
                max_f,
                min_focal_length,
                max_focal_length,
            } => {
                write!(
                    f,
                    "{}-{}mm f/{}-{}",
                    min_focal_length, max_focal_length, min_f, max_f
                )
            }
            Self::Prime {
                min_f,
                focal_length,
            } => {
                write!(f, "{}mm f/{}", focal_length, min_f)
            }
        }
    }
}

pub(crate) struct ExifTag {
    pub tag: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ExifInfo {
    pub file_path: String,
    pub mod_time: DateTime<chrono::Utc>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
    pub lens_info: Option<LensInfo>,
    pub exposure_compensation: Option<String>,
    pub iso: Option<String>,
    pub aperture: Option<String>,
    pub shutter_speed: Option<String>,
    pub focal_length: Option<String>,
    pub focal_length_35mm: Option<String>,
    pub preview_image: Option<image::DynamicImage>,
}
