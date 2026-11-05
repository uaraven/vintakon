use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensProfile {
    pub name: String,
    pub lens_make: String,
    pub lens_model: String,
    pub min_focal_length: f32,
    pub max_focal_length: f32,
    pub min_aperture: String,
    pub max_aperture: String,
    pub crop_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub exiftool_path: Option<String>,
    pub lens_profiles: Vec<LensProfile>,
}

/// Config file in the current directory, it has the highest priority
const LOCAL_CONFIG_FILE: &str = "vintakon.conf";

/// Places where the config can be stored: the local file, then the config directory of the user
/// (`$XDG_CONFIG_HOME/vintakon/config.json`, `$HOME/.config/vintakon/config.json`)
fn config_candidates(xdg_config_home: Option<OsString>, home: Option<OsString>) -> Vec<PathBuf> {
    let mut candidates = vec![PathBuf::from(LOCAL_CONFIG_FILE)];
    if let Some(dir) = xdg_config_home {
        candidates.push(Path::new(&dir).join("vintakon").join("config.json"));
    }
    if let Some(dir) = home {
        candidates.push(
            Path::new(&dir)
                .join(".config")
                .join("vintakon")
                .join("config.json"),
        );
    }
    candidates
}

/// The config that exists, or the place for a new one (the user's config directory, never the
/// local file). Loading and saving use the same file, so saved settings are always loaded back
fn pick_config_path(candidates: &[PathBuf], exists: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    candidates
        .iter()
        .find(|path| exists(path))
        .or_else(|| candidates.get(1))
        .cloned()
}

fn config_path() -> Option<PathBuf> {
    let candidates = config_candidates(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    );
    pick_config_path(&candidates, Path::exists)
}

impl Config {
    /// Creates a new empty Config
    pub fn new() -> Self {
        Self {
            exiftool_path: None,
            lens_profiles: Vec::new(),
        }
    }

    /// Loads the configuration from the first existing config file, empty config is returned
    /// if there is none. If the file cannot be read it is renamed to `<name>.broken` (so that
    /// saving the settings does not overwrite it) and the second value describes the problem.
    pub fn load_with_fallback() -> (Self, Option<String>) {
        let Some(path) = config_path().filter(|path| path.exists()) else {
            return (Self::new(), None);
        };
        match Self::load_from_file(&path) {
            Ok(config) => (config, None),
            Err(e) => {
                let mut backup = path.clone().into_os_string();
                backup.push(".broken");
                let backup = PathBuf::from(backup);
                let message = match fs::rename(&path, &backup) {
                    Ok(()) => format!(
                        "Failed to read the settings from {}: {e}\nThe file was renamed to {}",
                        path.display(),
                        backup.display()
                    ),
                    Err(rename_error) => format!(
                        "Failed to read the settings from {}: {e}\nIt could not be renamed: {rename_error}",
                        path.display()
                    ),
                };
                (Self::new(), Some(message))
            }
        }
    }

    /// Saves the Config to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let json_string = serde_json::to_string_pretty(self)?;
        fs::write(path, json_string)?;
        Ok(())
    }

    /// Loads a Config from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let json_string = fs::read_to_string(path)?;
        let mut config: Config = serde_json::from_str(&json_string)?;

        // Sort lens profiles alphabetically by name
        config.lens_profiles.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(config)
    }

    #[cfg(test)]
    /// Loads a Config from a JSON file, or creates a new one if the file doesn't exist
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        use std::io;
        match Self::load_from_file(&path) {
            Ok(config) => Ok(config),
            Err(e) => {
                if let Some(io_error) = e.downcast_ref::<io::Error>() {
                    if io_error.kind() == io::ErrorKind::NotFound {
                        // File doesn't exist, create a new config
                        return Ok(Self::new());
                    }
                }
                // Re-throw other errors
                Err(e)
            }
        }
    }

    /// Saves the configuration to the file it was loaded from, or to
    /// $XDG_CONFIG_HOME/vintakon/config.json or $HOME/.config/vintakon/config.json
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = config_path().ok_or(
            "Unable to determine config directory: neither XDG_CONFIG_HOME nor HOME environment variables are set",
        )?;

        // Create the parent directory if it doesn't exist
        if let Some(parent_dir) = config_path.parent() {
            fs::create_dir_all(parent_dir)?;
        }

        self.save_to_file(config_path)
    }

    /// Profiles are identified by their names, so the names must not be empty or repeated
    pub fn validate(&self) -> Result<(), String> {
        let mut names = HashSet::new();
        for profile in &self.lens_profiles {
            if profile.name.trim().is_empty() {
                return Err("Lens profile name must not be empty".to_string());
            }
            if !names.insert(profile.name.as_str()) {
                return Err(format!(
                    "Lens profile name '{}' is used more than once",
                    profile.name
                ));
            }
        }
        Ok(())
    }

    /// Name for a new profile that is not used by any of the existing ones
    pub fn new_profile_name(&self) -> String {
        (self.lens_profiles.len() + 1..)
            .map(|n| format!("New Lens {n}"))
            .find(|name| self.lens_profiles.iter().all(|p| &p.name != name))
            .expect("there are infinitely many candidate names")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_config_save_and_load() {
        let config = Config {
            exiftool_path: None,
            lens_profiles: vec![LensProfile {
                name: "Test Lens".to_string(),
                lens_make: "Canon".to_string(),
                lens_model: "EF 50mm f/1.8".to_string(),
                min_focal_length: 50.0,
                max_focal_length: 50.0,
                min_aperture: "1.8".to_string(),
                max_aperture: "22".to_string(),
                crop_factor: 1.0,
            }],
        };

        let temp_file = "/tmp/test_config.json";

        // Test save
        config
            .save_to_file(temp_file)
            .expect("Failed to save config");

        // Test load
        let loaded_config = Config::load_from_file(temp_file).expect("Failed to load config");

        assert_eq!(
            config.lens_profiles.len(),
            loaded_config.lens_profiles.len()
        );
        assert_eq!(
            config.lens_profiles[0].name,
            loaded_config.lens_profiles[0].name
        );
        assert_eq!(
            config.lens_profiles[0].lens_make,
            loaded_config.lens_profiles[0].lens_make
        );

        // Clean up
        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_config_load_or_create() {
        let non_existent_file = "/tmp/non_existent_config.json";

        // Ensure file doesn't exist
        let _ = fs::remove_file(non_existent_file);

        // Should create new config if file doesn't exist
        let config =
            Config::load_or_create(non_existent_file).expect("Failed to load or create config");
        assert_eq!(config.lens_profiles.len(), 0);

        // Create a test config file
        let test_config = Config {
            exiftool_path: None,
            lens_profiles: vec![LensProfile {
                name: "Test Lens".to_string(),
                lens_make: "Sony".to_string(),
                lens_model: "FE 85mm f/1.8".to_string(),
                min_focal_length: 85.0,
                max_focal_length: 85.0,
                min_aperture: "1.8".to_string(),
                max_aperture: "22".to_string(),
                crop_factor: 1.0,
            }],
        };
        test_config
            .save_to_file(non_existent_file)
            .expect("Failed to save test config");

        // Should load existing config if file exists
        let loaded_config =
            Config::load_or_create(non_existent_file).expect("Failed to load existing config");
        assert_eq!(loaded_config.lens_profiles.len(), 1);
        assert_eq!(loaded_config.lens_profiles[0].name, "Test Lens");

        // Clean up
        let _ = fs::remove_file(non_existent_file);
    }

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert_eq!(config.lens_profiles.len(), 0);
        assert!(config.lens_profiles.is_empty());
    }

    #[test]
    fn test_config_empty_save_and_load() {
        let config = Config::new();
        let temp_file = "/tmp/test_empty_config.json";

        // Test save empty config
        config
            .save_to_file(temp_file)
            .expect("Failed to save empty config");

        // Test load empty config
        let loaded_config = Config::load_from_file(temp_file).expect("Failed to load empty config");
        assert_eq!(loaded_config.lens_profiles.len(), 0);

        // Clean up
        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_config_multiple_lens_profiles() {
        let config = Config {
            exiftool_path: None,
            lens_profiles: vec![
                LensProfile {
                    name: "Canon Prime".to_string(),
                    lens_make: "Canon".to_string(),
                    lens_model: "EF 50mm f/1.8".to_string(),
                    min_focal_length: 50.0,
                    max_focal_length: 50.0,
                    min_aperture: "1.8".to_string(),
                    max_aperture: "22".to_string(),
                    crop_factor: 1.0,
                },
                LensProfile {
                    name: "Sony Zoom".to_string(),
                    lens_make: "Sony".to_string(),
                    lens_model: "FE 24-70mm f/2.8".to_string(),
                    min_focal_length: 24.0,
                    max_focal_length: 70.0,
                    min_aperture: "2.8".to_string(),
                    max_aperture: "22".to_string(),
                    crop_factor: 1.0,
                },
                LensProfile {
                    name: "Vintage Manual".to_string(),
                    lens_make: "Pentax".to_string(),
                    lens_model: "SMC Takumar 55mm f/1.8".to_string(),
                    min_focal_length: 55.0,
                    max_focal_length: 55.0,
                    min_aperture: "1.8".to_string(),
                    max_aperture: "16".to_string(),
                    crop_factor: 1.5,
                },
            ],
        };

        let temp_file = "/tmp/test_multiple_config.json";

        // Test save
        config
            .save_to_file(temp_file)
            .expect("Failed to save multiple lens config");

        // Test load
        let loaded_config =
            Config::load_from_file(temp_file).expect("Failed to load multiple lens config");

        assert_eq!(
            config.lens_profiles.len(),
            loaded_config.lens_profiles.len()
        );
        assert_eq!(loaded_config.lens_profiles.len(), 3);

        // Verify all profiles are loaded correctly
        for (original, loaded) in config
            .lens_profiles
            .iter()
            .zip(loaded_config.lens_profiles.iter())
        {
            assert_eq!(original.name, loaded.name);
            assert_eq!(original.lens_make, loaded.lens_make);
            assert_eq!(original.lens_model, loaded.lens_model);
            assert_eq!(original.min_focal_length, loaded.min_focal_length);
            assert_eq!(original.max_focal_length, loaded.max_focal_length);
            assert_eq!(original.min_aperture, loaded.min_aperture);
            assert_eq!(original.max_aperture, loaded.max_aperture);
            assert_eq!(original.crop_factor, loaded.crop_factor);
        }

        // Clean up
        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_config_load_invalid_json() {
        let temp_file = "/tmp/test_invalid_config.json";

        // Write invalid JSON to file
        fs::write(temp_file, "{ invalid json content }").expect("Failed to write invalid JSON");

        // Should fail to load invalid JSON
        let result = Config::load_from_file(temp_file);
        assert!(result.is_err());

        // Clean up
        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_config_load_nonexistent_file() {
        let nonexistent_file = "/tmp/definitely_does_not_exist_config.json";

        // Ensure file doesn't exist
        let _ = fs::remove_file(nonexistent_file);

        // Should fail to load non-existent file
        let result = Config::load_from_file(nonexistent_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_save_to_invalid_path() {
        let config = Config::new();
        let invalid_path = "/root/definitely_cannot_write_here/config.json";

        // Should fail to save to invalid path
        let result = config.save_to_file(invalid_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_json_format() {
        let config = Config {
            exiftool_path: None,
            lens_profiles: vec![LensProfile {
                name: "Test Lens".to_string(),
                lens_make: "TestMake".to_string(),
                lens_model: "TestModel".to_string(),
                min_focal_length: 35.0,
                max_focal_length: 85.0,
                min_aperture: "2.8".to_string(),
                max_aperture: "16".to_string(),
                crop_factor: 1.6,
            }],
        };

        let temp_file = "/tmp/test_format_config.json";

        // Save config
        config
            .save_to_file(temp_file)
            .expect("Failed to save config");

        // Read the raw JSON and verify it's properly formatted
        let json_content = fs::read_to_string(temp_file).expect("Failed to read JSON file");

        // Should be valid JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&json_content).expect("Invalid JSON format");

        // Verify structure
        assert!(parsed.get("lens_profiles").is_some());
        let profiles = parsed["lens_profiles"]
            .as_array()
            .expect("lens_profiles should be an array");
        assert_eq!(profiles.len(), 1);

        let profile = &profiles[0];
        assert_eq!(profile["name"].as_str().unwrap(), "Test Lens");
        assert_eq!(profile["lens_make"].as_str().unwrap(), "TestMake");
        assert_eq!(profile["min_focal_length"].as_f64().unwrap(), 35.0);

        // Clean up
        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_config_load_or_create_with_corrupted_file() {
        let temp_file = "/tmp/test_corrupted_config.json";

        // Create a corrupted JSON file
        fs::write(temp_file, "{ corrupted json ").expect("Failed to write corrupted JSON");

        // load_or_create should fail with corrupted file (not return empty config)
        let result = Config::load_or_create(temp_file);
        assert!(result.is_err());

        // Clean up
        let _ = fs::remove_file(temp_file);
    }

    // #[test]

    fn named(name: &str) -> LensProfile {
        LensProfile {
            name: name.to_string(),
            lens_make: String::new(),
            lens_model: String::new(),
            min_focal_length: 50.0,
            max_focal_length: 50.0,
            min_aperture: "1.8".to_string(),
            max_aperture: "22".to_string(),
            crop_factor: 1.0,
        }
    }

    #[test]
    fn test_validate_rejects_empty_and_duplicate_names() {
        let mut config = Config::new();
        assert!(config.validate().is_ok());

        config.lens_profiles = vec![named("A"), named("B")];
        assert!(config.validate().is_ok());

        config.lens_profiles = vec![named("A"), named("A")];
        assert!(config.validate().unwrap_err().contains("'A'"));

        config.lens_profiles = vec![named("A"), named("  ")];
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_new_profile_name_is_unique() {
        let mut config = Config::new();
        assert_eq!(config.new_profile_name(), "New Lens 1");

        // "New Lens 2" is taken although there is only one profile
        config.lens_profiles = vec![named("New Lens 2")];
        assert_eq!(config.new_profile_name(), "New Lens 3");
    }

    #[test]
    fn test_config_path_is_used_for_both_loading_and_saving() {
        let candidates = config_candidates(Some("/xdg".into()), Some("/home/user".into()));
        assert_eq!(candidates[0], PathBuf::from("vintakon.conf"));
        let xdg = PathBuf::from("/xdg/vintakon/config.json");
        let home = PathBuf::from("/home/user/.config/vintakon/config.json");

        // Nothing exists: a new config goes to the user's config directory
        assert_eq!(pick_config_path(&candidates, |_| false), Some(xdg.clone()));
        // The local file wins when it exists
        assert_eq!(
            pick_config_path(&candidates, |p| p == Path::new("vintakon.conf")),
            Some(PathBuf::from("vintakon.conf"))
        );
        // An existing config in HOME is kept even if XDG_CONFIG_HOME is set
        assert_eq!(pick_config_path(&candidates, |p| p == home), Some(home));
        // Without both variables there is nowhere to save
        let none = config_candidates(None, None);
        assert_eq!(pick_config_path(&none, |_| false), None);
    }
}
