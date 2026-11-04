use freya::prelude::*;

use crate::{
    logic,
    model::{
        file_rec::FileRec,
        lens_profile::{Config, LensProfile},
    },
};

/// Application state. All fields are Freya `State` handles which are `Copy`,
/// so the struct can be shared with every component through the context.
#[derive(Clone, Copy)]
pub(crate) struct AppState {
    pub path: State<Option<String>>,
    pub images: State<Vec<FileRec>>,
    pub is_applying: State<bool>,
    pub is_loading: State<bool>,
    pub selected_file_index: State<Option<usize>>,
    pub config: State<Config>,
    pub selected_profile: State<Option<String>>,
    pub apply_aperture: State<String>,
    pub apply_focal_length: State<String>,
    pub show_settings: State<bool>,
    /// Whether the "Apply profile" dialog is open
    pub show_apply: State<bool>,
    /// Copy of the config that is edited in the settings dialog
    pub temp_config: State<Config>,
    pub selected_profile_index: State<Option<usize>>,
    /// Text of the exiftool path input in the settings dialog
    pub exiftool_text: State<String>,
}

impl AppState {
    /// Creates all the state. Must be called from a component body (it uses hooks).
    pub fn new() -> Self {
        Self {
            path: use_state(|| None),
            images: use_state(Vec::new),
            is_applying: use_state(|| false),
            is_loading: use_state(|| false),
            selected_file_index: use_state(|| None),
            config: use_state(Config::load_with_fallback),
            selected_profile: use_state(|| None),
            apply_aperture: use_state(String::new),
            apply_focal_length: use_state(String::new),
            show_settings: use_state(|| false),
            show_apply: use_state(|| false),
            temp_config: use_state(|| Config {
                exiftool_path: None,
                lens_profiles: Vec::new(),
            }),
            selected_profile_index: use_state(|| None),
            exiftool_text: use_state(String::new),
        }
    }

    // ---- queries ----

    /// Names of the configured lens profiles, sorted alphabetically
    pub fn profile_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .config
            .read()
            .lens_profiles
            .iter()
            .map(|p| p.name.clone())
            .collect();
        names.sort();
        names
    }

    pub fn is_any_file_checked(&self) -> bool {
        self.images.read().iter().any(|img| img.is_selected())
    }

    fn find_profile_by_name(&self, name: &str) -> Option<LensProfile> {
        self.config
            .read()
            .lens_profiles
            .iter()
            .find(|p| p.name == name)
            .cloned()
    }

    fn exiftool_path(&self) -> Option<String> {
        self.config.read().exiftool_path.clone()
    }

    // ---- main screen actions ----

    /// Asks user for a folder and loads the images from it
    pub fn select_folder(self) {
        // Not tied to the caller's scope: the File menu item that triggers this is unmounted on press
        spawn_forever(async move {
            if let Some(folder) = logic::select_folder().await {
                let mut this = self;
                this.is_loading.set(true);
                this.reload(folder).await;
            }
        });
    }

    /// Reads folder content and replaces the image list
    async fn reload(mut self, folder: String) {
        let images = logic::load_images(folder.clone(), self.exiftool_path()).await;
        self.images.set(images);
        self.path.set(Some(folder));
        self.is_loading.set(false);
        self.selected_file_index.set(None);
    }

    pub fn set_file_checked(mut self, index: usize, checked: bool) {
        if let Some(file_rec) = self.images.write().get_mut(index) {
            file_rec.set_selected(checked);
        }
    }

    /// Writes selected lens profile to the checked images and reloads the folder
    pub fn apply_profile(mut self) {
        let (Some(working_dir), Some(profile_name)) = (
            self.path.read().clone(),
            self.selected_profile.read().clone(),
        ) else {
            return;
        };
        let Some(profile) = self.find_profile_by_name(&profile_name) else {
            return;
        };
        let selected: Vec<FileRec> = self
            .images
            .read()
            .iter()
            .filter(|img| img.is_selected())
            .cloned()
            .collect();
        let aperture = self.apply_aperture.read().clone();
        let focal_length = self.apply_focal_length.read().clone();
        let exiftool = self.exiftool_path();

        self.is_applying.set(true);
        spawn(async move {
            let mut this = self;
            logic::apply_profile_to_images(
                working_dir.clone(),
                profile,
                aperture,
                focal_length,
                selected,
                exiftool,
            )
            .await;
            this.is_applying.set(false);
            this.reload(working_dir).await;
        });
    }

    // ---- settings dialog actions ----

    pub fn open_settings(mut self) {
        let config = self.config.read().clone();
        self.selected_profile_index
            .set(if config.lens_profiles.is_empty() {
                None
            } else {
                Some(0)
            });
        self.exiftool_text
            .set(config.exiftool_path.clone().unwrap_or_default());
        self.temp_config.set(config);
        self.show_settings.set(true);
    }

    pub fn close_settings(mut self) {
        self.show_settings.set(false);
        self.selected_profile_index.set(None);
    }

    pub fn save_settings(mut self) {
        let new_config = self.temp_config.read().clone();
        let _ = new_config.save();
        self.config.set(new_config);
        // Drop the selection if the selected profile does not exist any more
        let selected = self.selected_profile.read().clone();
        if let Some(name) = selected
            && self.find_profile_by_name(&name).is_none()
        {
            self.selected_profile.set(None);
        }
        self.close_settings();
    }

    pub fn add_profile(mut self) {
        let mut config = self.temp_config.write();
        let name = format!("New Lens {}", config.lens_profiles.len() + 1);
        config.lens_profiles.push(LensProfile {
            name,
            lens_make: String::new(),
            lens_model: String::new(),
            min_focal_length: 50.0,
            max_focal_length: 50.0,
            min_aperture: "1.8".to_string(),
            max_aperture: "22".to_string(),
            crop_factor: 1.0,
        });
        let last = config.lens_profiles.len() - 1;
        drop(config);
        self.selected_profile_index.set(Some(last));
    }

    pub fn delete_profile(mut self, index: usize) {
        let remaining = {
            let mut config = self.temp_config.write();
            if index >= config.lens_profiles.len() {
                return;
            }
            config.lens_profiles.remove(index);
            config.lens_profiles.len()
        };
        let selected = *self.selected_profile_index.read();
        if let Some(selected) = selected {
            if selected == index {
                self.selected_profile_index.set(if remaining == 0 {
                    None
                } else {
                    Some(index.saturating_sub(1))
                });
            } else if selected > index {
                self.selected_profile_index.set(Some(selected - 1));
            }
        }
    }

    /// Applies a change to one of the profiles that are being edited
    pub fn edit_profile(mut self, index: usize, change: impl FnOnce(&mut LensProfile)) {
        if let Some(profile) = self.temp_config.write().lens_profiles.get_mut(index) {
            change(profile);
        }
    }

    /// Stores the exiftool path being typed in the settings dialog
    pub fn edit_exiftool_path(mut self, path: String) {
        self.temp_config.write().exiftool_path = if path.is_empty() { None } else { Some(path) };
    }

    /// Asks user for the location of the exiftool executable
    pub fn select_exiftool_path(self) {
        spawn(async move {
            if let Some(path) = logic::select_exiftool_file().await {
                let mut this = self;
                this.exiftool_text.set(path.clone());
                this.edit_exiftool_path(path);
            }
        });
    }
}
