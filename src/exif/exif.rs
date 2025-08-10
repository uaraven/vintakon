use std::fmt::Display;

use chrono::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct ExifInfoRaw {
    pub file_path: String,
    pub mod_time: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
    pub lens_info: Option<String>,
    pub exposure_compensation: Option<String>,
    pub iso: Option<String>,
    pub aperture: Option<String>,
    pub shutter_speed: Option<String>,
    pub focal_length: Option<String>,
    pub focal_length_35mm: Option<String>,
    pub preview_base64: Option<String>,
}

impl ExifInfoRaw {
    pub fn new_from_json(json: String) -> Self {
        serde_json::from_str(&json).unwrap()
    }
}

pub(crate) enum LensInfo {
    Zoom {
        min_f: f32,
        max_f: f32,
        min_focal_length: f32,
        max_focal_legth: f32,
    },
    Prime {
        min_f: f32,
        focal_length: f32,
    },
}

impl LensInfo {
    pub fn from_string(lens_info: &str) -> Option<Self> {
        let parts: Vec<&str> = lens_info.split(' ').collect();
        if parts.len() == 4 {
            Some(Self::Zoom {
                min_f: parts[0].parse().ok()?,
                max_f: parts[1].parse().ok()?,
                min_focal_length: parts[2].parse().ok()?,
                max_focal_legth: parts[3].parse().ok()?,
            })
        } else if parts.len() == 2 {
            Some(Self::Prime {
                min_f: parts[0].parse().ok()?,
                focal_length: parts[1].parse().ok()?,
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
                max_focal_legth,
            } => {
                write!(
                    f,
                    "f/{}-{} ({}-{}mm)",
                    min_f, max_f, min_focal_length, max_focal_legth
                )
            }
            Self::Prime {
                min_f,
                focal_length,
            } => {
                write!(f, "f/{} ({}mm)", min_f, focal_length)
            }
        }
    }
}

pub(crate) struct ExifTag {
    pub tag: String,
    pub value: String,
}

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
    pub preview_base64: Option<String>,
}
