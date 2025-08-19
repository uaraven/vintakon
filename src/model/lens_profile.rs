use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

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

impl Config {
    /// Creates a new empty Config
    pub fn new() -> Self {
        Self {
            exiftool_path: None,
            lens_profiles: Vec::new(),
        }
    }

    /// Loads configuration with fallback logic:
    /// 1. Try vintakon.conf in current directory
    /// 2. Try config.json in $XDG_CONFIG_HOME/vintakon
    /// 3. Return empty config if neither exists
    pub fn load_with_fallback() -> Self {
        // Try vintakon.conf in current directory first
        if let Ok(config) = Self::load_from_file("vintakon.conf") {
            return config;
        }

        // Try XDG_CONFIG_HOME/vintakon/config.json
        if let Some(config_dir) = std::env::var_os("XDG_CONFIG_HOME") {
            let config_path = Path::new(&config_dir).join("vintakon").join("config.json");
            if let Ok(config) = Self::load_from_file(&config_path) {
                return config;
            }
        }

        // If XDG_CONFIG_HOME is not set, try ~/.config/vintakon/config.json
        if let Some(home_dir) = std::env::var_os("HOME") {
            let config_path = Path::new(&home_dir).join(".config").join("vintakon").join("config.json");
            if let Ok(config) = Self::load_from_file(&config_path) {
                return config;
            }
        }

        // Return empty config if no files found
        Self::new()
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
        let config: Config = serde_json::from_str(&json_string)?;
        Ok(config)
    }

    /// Loads a Config from a JSON file, or creates a new one if the file doesn't exist
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
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

    /// Saves configuration to $XDG_CONFIG_HOME/vintakon/config.json or $HOME/.config/vintakon/config.json
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
            Path::new(&config_home).join("vintakon").join("config.json")
        } else if let Some(home_dir) = std::env::var_os("HOME") {
            Path::new(&home_dir).join(".config").join("vintakon").join("config.json")
        } else {
            return Err("Unable to determine config directory: neither XDG_CONFIG_HOME nor HOME environment variables are set".into());
        };

        // Create the parent directory if it doesn't exist
        if let Some(parent_dir) = config_path.parent() {
            fs::create_dir_all(parent_dir)?;
        }

        self.save_to_file(config_path)
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

    #[test]
    fn test_config_load_with_fallback() {
        // Test with no config files - should return empty config
        let config = Config::load_with_fallback();
        assert_eq!(config.lens_profiles.len(), 0);
        assert!(config.exiftool_path.is_none());

        // Create vintakon.conf in current directory
        let test_config = Config {
            exiftool_path: Some("/usr/local/bin/exiftool".to_string()),
            lens_profiles: vec![LensProfile {
                name: "Test Local Lens".to_string(),
                lens_make: "Local".to_string(),
                lens_model: "Test 50mm".to_string(),
                min_focal_length: 50.0,
                max_focal_length: 50.0,
                min_aperture: "1.8".to_string(),
                max_aperture: "22".to_string(),
                crop_factor: 1.0,
            }],
        };

        test_config
            .save_to_file("vintakon.conf")
            .expect("Failed to save vintakon.conf");

        // Test loading from vintakon.conf (should prefer this over XDG config)
        let loaded_config = Config::load_with_fallback();
        assert_eq!(loaded_config.lens_profiles.len(), 1);
        assert_eq!(loaded_config.lens_profiles[0].name, "Test Local Lens");
        assert_eq!(loaded_config.exiftool_path.as_deref(), Some("/usr/local/bin/exiftool"));

        // Clean up
        let _ = fs::remove_file("vintakon.conf");
    }
}
