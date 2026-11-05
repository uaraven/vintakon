use std::{collections::HashSet, rc::Rc};

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
    /// Indexes of the checked images. It is separate from `images`, so toggling a checkbox
    /// does not invalidate everything that shows the images
    pub checked: State<HashSet<usize>>,
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
    /// Message of the error dialog, if it is shown
    pub error_message: State<Option<String>>,
}

impl AppState {
    /// Creates all the state. Must be called from a component body (it uses hooks).
    pub fn new() -> Self {
        // The config is read once; a problem with it is shown as the first error
        let startup = use_hook(|| Rc::new(Config::load_with_fallback()));
        Self {
            path: use_state(|| None),
            images: use_state(Vec::new),
            checked: use_state(HashSet::new),
            is_applying: use_state(|| false),
            is_loading: use_state(|| false),
            selected_file_index: use_state(|| None),
            config: use_state(|| startup.0.clone()),
            selected_profile: use_state(|| None),
            apply_aperture: use_state(String::new),
            apply_focal_length: use_state(String::new),
            show_settings: use_state(|| false),
            show_apply: use_state(|| false),
            error_message: use_state(|| startup.1.clone()),
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
        !self.checked.read().is_empty()
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
                self.reload(folder).await;
            }
        });
    }

    pub fn is_folder_selected(&self) -> bool {
        self.path.read().is_some()
    }

    /// Re-reads the currently selected folder. Does nothing if there is no folder
    pub fn reload_folder(self) {
        let Some(folder) = self.path.read().clone() else {
            return;
        };
        // Not tied to the caller's scope: the File menu item that triggers this is unmounted on press
        spawn_forever(async move {
            self.reload(folder).await;
        });
    }

    /// Reads folder content and replaces the image list. It starts a new batch of work, so
    /// the checked files and the actual aperture / focal length of the previous one are dropped.
    /// If the folder cannot be read, the current state is kept and the error is shown.
    async fn reload(mut self, folder: String) {
        self.is_loading.set(true);
        match logic::load_images(folder.clone(), self.exiftool_path()).await {
            Ok(images) => {
                self.images.set(images);
                self.checked.set(HashSet::new());
                self.apply_aperture.set(String::new());
                self.apply_focal_length.set(String::new());
                self.path.set(Some(folder));
                self.selected_file_index.set(None);
            }
            Err(message) => self.report_error(message),
        }
        self.is_loading.set(false);
    }

    /// Shows the error dialog. An error that is already shown is not replaced:
    /// it is usually the first one that explains the others
    pub fn report_error(mut self, message: String) {
        if self.error_message.read().is_none() {
            self.error_message.set(Some(message));
        }
    }

    pub fn set_file_checked(mut self, index: usize, checked: bool) {
        let mut set = self.checked.write();
        if checked {
            set.insert(index);
        } else {
            set.remove(&index);
        }
    }

    pub fn select_file(mut self, index: usize) {
        self.selected_file_index.set(Some(index));
    }

    /// Opens the "Apply profile" dialog if there is something to apply it to
    pub fn open_apply(mut self) {
        if self.is_any_file_checked() {
            self.show_apply.set(true);
        }
    }

    pub fn close_apply(mut self) {
        self.show_apply.set(false);
    }

    /// Closes the "Apply profile" dialog and applies the selected profile
    pub fn confirm_apply(self) {
        if self.selected_profile.read().is_some() {
            self.close_apply();
            self.apply_profile();
        }
    }

    /// Writes selected lens profile to the checked images and reloads the folder
    fn apply_profile(mut self) {
        let (Some(working_dir), Some(profile_name)) = (
            self.path.read().clone(),
            self.selected_profile.read().clone(),
        ) else {
            return;
        };
        let Some(profile) = self.find_profile_by_name(&profile_name) else {
            return;
        };
        let selected: Vec<String> = {
            let images = self.images.read();
            let checked = self.checked.read();
            images
                .iter()
                .enumerate()
                .filter(|(index, _)| checked.contains(index))
                .map(|(_, img)| img.exif_info().file_path.clone())
                .collect()
        };
        if selected.is_empty() {
            return;
        }
        let aperture = self.apply_aperture.read().clone();
        let focal_length = self.apply_focal_length.read().clone();
        let exiftool = self.exiftool_path();

        self.is_applying.set(true);
        // Not tied to the caller's scope: the Apply dialog that triggers this is closed (unmounted)
        // right before, which would cancel the task and leave `is_applying` set forever
        spawn_forever(async move {
            let mut this = self;
            let result = logic::apply_profile_to_images(
                working_dir.clone(),
                profile,
                aperture,
                focal_length,
                selected,
                exiftool,
            )
            .await;
            // `reload` shows its own busy dialog right away, so the UI is never left idle in between
            this.is_applying.set(false);
            if let Err(message) = result {
                this.report_error(message);
            }
            this.reload(working_dir).await;
        });
    }

    // ---- settings ----

    pub fn open_settings(mut self) {
        self.show_settings.set(true);
    }

    pub fn close_settings(mut self) {
        self.show_settings.set(false);
    }

    /// Takes the edited settings into use. Drops the profile selection
    /// if the selected profile does not exist any more
    pub fn set_config(mut self, config: Config) {
        self.config.set(config);
        let selected = self.selected_profile.read().clone();
        if let Some(name) = selected
            && self.find_profile_by_name(&name).is_none()
        {
            self.selected_profile.set(None);
        }
    }
}
