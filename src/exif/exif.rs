use std::fmt::Display;

use chrono::DateTime;

use crate::{exif::tags, model::lens_profile::LensProfile};

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

    fn from_profile(profile: &LensProfile) -> Self {
        if profile.min_focal_length == profile.max_focal_length
            && profile.min_aperture == profile.max_aperture
        {
            Self::Prime {
                focal_length: format!("{:.1}", profile.min_focal_length),
                min_f: format!("{}", profile.min_aperture.clone()),
            }
        } else {
            Self::Zoom {
                min_focal_length: format!("{:.1}", profile.min_focal_length),
                max_focal_length: format!("{:.1}", profile.max_focal_length),
                min_f: format!("{}", profile.min_aperture.clone()),
                max_f: format!("{}", profile.max_aperture.clone()),
            }
        }
    }

    pub fn to_exif_string(&self) -> String {
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

#[derive(Debug, Clone)]
pub(crate) struct ExifInfo {
    pub file_path: String,
    pub mod_time: DateTime<chrono::Utc>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
    pub lens_info: Option<LensInfo>,
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
    pub fn new(
        lens_profile: &LensProfile,
        actual_focal_length: Option<f32>,
        actual_aperture: Option<String>,
    ) -> Self {
        let make = Some(lens_profile.lens_make.clone());
        let model = Some(lens_profile.lens_model.clone());
        let aperture = actual_aperture.clone();
        // fl will contain the focal length and its equivalent in 35mm format
        let fl = if let Some(actual_fl) = actual_focal_length {
            Some((actual_fl, actual_fl * lens_profile.crop_factor))
        } else if lens_profile.min_focal_length == lens_profile.max_focal_length {
            Some((
                lens_profile.min_focal_length,
                lens_profile.min_focal_length * lens_profile.crop_factor,
            ))
        } else {
            None
        };

        let focal_length = fl.map(|f| format!("{:.1}", f.0));
        let focal_length_35mm = fl.map(|f| format!("{:.1}", f.1));
        let lens_info = LensInfo::from_profile(&lens_profile);

        Self {
            make,
            model,
            aperture,
            focal_length,
            focal_length_35mm,
            lens_info: Some(lens_info),
        }
    }

    pub fn to_exif_tool(&self) -> Vec<String> {
        let mut args = vec![];
        let mut lens = String::new();
        if let Some(make) = &self.make {
            args.push(format!("-{}={}", tags::LENS_MAKE, make));
            lens.push_str(make.as_str());
            lens.push_str(" ");
        }
        if let Some(model) = &self.model {
            args.push(format!("-{}={}", tags::LENS_MODEL, model));
            lens.push_str(model.as_str());
        }
        if !lens.is_empty() {
            args.push(format!("-{}={}", tags::LENS, lens));
        }
        if let Some(aperture) = &self.aperture {
            args.push(format!("-{}={}", tags::APERTURE, aperture));
            args.push(format!("-{}={}", tags::F_NUMBER, aperture));
        }
        if let Some(focal_length) = &self.focal_length {
            args.push(format!("-{}={}", tags::FOCAL_LENGTH, focal_length));
        }
        if let Some(focal_length_35mm) = &self.focal_length_35mm {
            args.push(format!(
                "-{}={}",
                tags::FOCAL_LENGTH_35MM,
                focal_length_35mm
            ));
        }
        if let Some(lens_info) = &self.lens_info {
            args.push(format!(
                "-{}={}",
                tags::LENS_INFO,
                lens_info.to_exif_string()
            ));
        }
        args
    }
}
