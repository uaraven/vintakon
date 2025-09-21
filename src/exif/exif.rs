use std::fmt::Display;

use chrono::DateTime;

use crate::exif::tags::{self, LENS_MAKE};

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

    fn to_exif_string(&self) -> String {
        match self {
            Self::Zoom {
                min_focal_length,
                max_focal_length,
                min_f,
                max_f,
            } => format!(
                "{} {} {} {}",
                min_focal_length, max_focal_length, min_f, max_f
            ),
            Self::Prime {
                focal_length,
                min_f,
            } => format!("{} {} {} {}", focal_length, focal_length, min_f, min_f),
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

#[derive(Debug, Clone)]
pub(crate) struct ExifChanges {
    pub make: Option<String>,
    pub model: Option<String>,
    pub aperture: Option<String>,
    pub focal_length: Option<String>,
    pub focal_length_35mm: Option<String>,
    pub lens_info: Option<LensInfo>,
}

impl ExifChanges {
    pub fn to_exif_tool(&self) -> Vec<String> {
        let mut args = vec![];
        if let Some(make) = &self.make {
            args.push(format!("--{}={}", tags::LENS_MAKE, make));
        }
        if let Some(model) = &self.model {
            args.push(format!("--{}={}", tags::LENS_MODEL, model));
        }
        if let Some(aperture) = &self.aperture {
            args.push(format!("--{}={}", tags::APERTURE, aperture));
            args.push(format!("--{}={}", tags::F_NUMBER, aperture));
        }
        if let Some(focal_length) = &self.focal_length {
            args.push(format!("--{}={}", tags::FOCAL_LENGTH, focal_length));
        }
        if let Some(focal_length_35mm) = &self.focal_length_35mm {
            args.push(format!(
                "--{}={}",
                tags::FOCAL_LENGTH_35MM,
                focal_length_35mm
            ));
        }
        if let Some(lens_info) = &self.lens_info {
            args.push(format!(
                "--{}={}",
                tags::LENS_INFO,
                lens_info.to_exif_string()
            ));
        }
        args
    }
}
