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
    /// Parses the `LensInfo` tag value: `min_fl max_fl min_f max_f`, or `fl min_f` for a prime
    pub fn from_string(lens_info: &str) -> Option<Self> {
        let parts: Vec<&str> = lens_info.split_whitespace().collect();
        match parts[..] {
            [min_fl, max_fl, min_f, max_f] if min_fl == max_fl && min_f == max_f => {
                Some(Self::Prime {
                    focal_length: min_fl.to_string(),
                    min_f: min_f.to_string(),
                })
            }
            [min_fl, max_fl, min_f, max_f] => Some(Self::Zoom {
                min_focal_length: min_fl.to_string(),
                max_focal_length: max_fl.to_string(),
                min_f: min_f.to_string(),
                max_f: max_f.to_string(),
            }),
            [focal_length, min_f] => Some(Self::Prime {
                focal_length: focal_length.to_string(),
                min_f: min_f.to_string(),
            }),
            _ => None,
        }
    }

    fn from_profile(profile: &LensProfile) -> Self {
        if profile.min_focal_length == profile.max_focal_length
            && profile.min_aperture == profile.max_aperture
        {
            Self::Prime {
                focal_length: format!("{:.1}", profile.min_focal_length),
                min_f: profile.min_aperture.clone(),
            }
        } else {
            Self::Zoom {
                min_focal_length: format!("{:.1}", profile.min_focal_length),
                max_focal_length: format!("{:.1}", profile.max_focal_length),
                min_f: profile.min_aperture.clone(),
                max_f: profile.max_aperture.clone(),
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
    /// Encoded (JPEG) preview as embedded in the file. It is decoded only when the file is
    /// selected: decoding the previews of the whole folder is slow and takes gigabytes of memory
    pub preview_image: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub(crate) struct ExifChanges {
    pub make: String,
    pub model: String,
    pub aperture: Option<String>,
    pub focal_length: Option<String>,
    pub focal_length_35mm: Option<String>,
    pub lens_info: LensInfo,
}

impl ExifChanges {
    pub fn new(
        lens_profile: &LensProfile,
        actual_focal_length: Option<f32>,
        actual_aperture: Option<String>,
    ) -> Self {
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

        Self {
            make: lens_profile.lens_make.clone(),
            model: lens_profile.lens_model.clone(),
            aperture: actual_aperture,
            focal_length: fl.map(|f| format!("{:.1}", f.0)),
            focal_length_35mm: fl.map(|f| format!("{:.1}", f.1)),
            lens_info: LensInfo::from_profile(lens_profile),
        }
    }

    pub fn to_exif_tool(&self) -> Vec<String> {
        let mut args = vec![
            format!("-{}={}", tags::LENS_MAKE, self.make),
            format!("-{}={}", tags::LENS_MODEL, self.model),
        ];
        // Lens description is "make model" without stray spaces when one of them is empty
        let lens = [self.make.as_str(), self.model.as_str()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        args.push(format!("-{}={}", tags::LENS, lens));
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
        args.push(format!(
            "-{}={}",
            tags::LENS_INFO,
            self.lens_info.to_exif_string()
        ));
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(make: &str, model: &str, min_fl: f32, max_fl: f32) -> LensProfile {
        LensProfile {
            name: "test".to_string(),
            lens_make: make.to_string(),
            lens_model: model.to_string(),
            min_focal_length: min_fl,
            max_focal_length: max_fl,
            min_aperture: "1.8".to_string(),
            max_aperture: "1.8".to_string(),
            crop_factor: 1.5,
        }
    }

    #[test]
    fn parses_prime_lens_info() {
        let expected = "30mm f/1.4";
        assert_eq!(
            LensInfo::from_string("30 30 1.4 1.4").unwrap().to_string(),
            expected
        );
        assert_eq!(
            LensInfo::from_string("30 1.4").unwrap().to_string(),
            expected
        );
    }

    #[test]
    fn parses_zoom_lens_info() {
        let info = LensInfo::from_string("200 600 5.6 6.3").unwrap();
        assert_eq!(info.to_string(), "200-600mm f/5.6-6.3");
        assert_eq!(info.to_exif_string(), "200 600 5.6 6.3");
    }

    #[test]
    fn rejects_malformed_lens_info() {
        assert!(LensInfo::from_string("").is_none());
        assert!(LensInfo::from_string("30 30 1.4").is_none());
    }

    #[test]
    fn lens_description_has_no_stray_spaces() {
        let lens_arg = |make: &str, model: &str| {
            ExifChanges::new(&profile(make, model, 50., 50.), None, None)
                .to_exif_tool()
                .into_iter()
                .find(|arg| arg.starts_with("-Lens="))
                .unwrap()
        };
        assert_eq!(lens_arg("Helios", "44-2"), "-Lens=Helios 44-2");
        assert_eq!(lens_arg("", "44-2"), "-Lens=44-2");
        assert_eq!(lens_arg("Helios", ""), "-Lens=Helios");
        assert_eq!(lens_arg("", ""), "-Lens=");
    }

    #[test]
    fn prime_profile_writes_focal_length_with_crop_factor() {
        let args = ExifChanges::new(&profile("A", "B", 50., 50.), None, Some("2.8".to_string()))
            .to_exif_tool();
        assert!(args.contains(&"-FocalLength=50.0".to_string()));
        assert!(args.contains(&"-FocalLengthIn35mmFormat=75.0".to_string()));
        assert!(args.contains(&"-Aperture=2.8".to_string()));
        assert!(args.contains(&"-FNumber=2.8".to_string()));
        assert!(args.contains(&"-LensInfo=50.0 50.0 1.8 1.8".to_string()));
    }

    #[test]
    fn zoom_profile_needs_actual_focal_length() {
        let zoom = profile("A", "B", 28., 70.);
        let without = ExifChanges::new(&zoom, None, None).to_exif_tool();
        assert!(!without.iter().any(|arg| arg.starts_with("-FocalLength=")));

        let with = ExifChanges::new(&zoom, Some(35.), None).to_exif_tool();
        assert!(with.contains(&"-FocalLength=35.0".to_string()));
        assert!(with.contains(&"-FocalLengthIn35mmFormat=52.5".to_string()));
    }
}
