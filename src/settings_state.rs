use freya::prelude::*;

use crate::{
    logic,
    model::lens_profile::{Config, LensProfile},
    state::AppState,
};

/// State of the settings dialog. It lives as long as the dialog is open, so every time
/// the dialog is opened it starts with a fresh copy of the config.
#[derive(Clone, Copy)]
pub(crate) struct SettingsState {
    app: AppState,
    /// Copy of the config that is edited in the dialog
    pub temp_config: State<Config>,
    pub selected_profile_index: State<Option<usize>>,
    /// Text of the exiftool path input
    pub exiftool_text: State<String>,
    /// Bumped when a profile is deleted, so the editor fields do not keep the text of the removed profile
    pub profile_generation: State<u32>,
}

impl SettingsState {
    /// Must be called from a component body (it uses hooks).
    pub fn new(app: AppState) -> Self {
        Self {
            app,
            temp_config: use_state(|| app.config.peek().clone()),
            selected_profile_index: use_state(|| {
                (!app.config.peek().lens_profiles.is_empty()).then_some(0)
            }),
            exiftool_text: use_state(|| {
                app.config.peek().exiftool_path.clone().unwrap_or_default()
            }),
            profile_generation: use_state(|| 0),
        }
    }

    pub fn close(self) {
        self.app.close_settings();
    }

    /// Saves the edited settings and closes the dialog. If they are not valid or cannot be
    /// saved the dialog stays open, so the edited profiles are not lost.
    pub fn save(self) {
        let new_config = self.temp_config.read().clone();
        if let Err(message) = new_config.validate() {
            self.app.report_error(message);
            return;
        }
        if let Err(e) = new_config.save() {
            self.app
                .report_error(format!("Failed to save the settings: {e}"));
            return;
        }
        self.app.set_config(new_config);
        self.close();
    }

    pub fn add_profile(mut self) {
        let mut config = self.temp_config.write();
        let name = config.new_profile_name();
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
        *self.profile_generation.write() += 1;
        let selected = *self.selected_profile_index.read();
        self.selected_profile_index
            .set(selection_after_delete(selected, index, remaining));
    }

    /// Applies a change to one of the profiles that are being edited
    pub fn edit_profile(mut self, index: usize, change: impl FnOnce(&mut LensProfile)) {
        if let Some(profile) = self.temp_config.write().lens_profiles.get_mut(index) {
            change(profile);
        }
    }

    /// Stores the exiftool path being typed
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

/// Which profile is selected after the profile at `deleted` was removed, leaving `remaining`
/// profiles: the selection follows its profile, and if the selected profile itself was removed
/// the previous one (or the first one) becomes selected
fn selection_after_delete(
    selected: Option<usize>,
    deleted: usize,
    remaining: usize,
) -> Option<usize> {
    let selected = selected?;
    if remaining == 0 {
        None
    } else if selected == deleted {
        Some(deleted.saturating_sub(1))
    } else if selected > deleted {
        Some(selected - 1)
    } else {
        Some(selected)
    }
}

#[cfg(test)]
mod tests {
    use super::selection_after_delete;

    #[test]
    fn deleting_the_selected_profile_selects_the_previous_one() {
        assert_eq!(selection_after_delete(Some(2), 2, 3), Some(1));
    }

    #[test]
    fn deleting_the_first_selected_profile_selects_the_new_first_one() {
        assert_eq!(selection_after_delete(Some(0), 0, 2), Some(0));
    }

    #[test]
    fn deleting_the_only_profile_clears_the_selection() {
        assert_eq!(selection_after_delete(Some(0), 0, 0), None);
    }

    #[test]
    fn deleting_a_profile_before_the_selected_one_shifts_the_selection() {
        assert_eq!(selection_after_delete(Some(3), 1, 4), Some(2));
    }

    #[test]
    fn deleting_a_profile_after_the_selected_one_keeps_the_selection() {
        assert_eq!(selection_after_delete(Some(1), 3, 4), Some(1));
    }

    #[test]
    fn nothing_selected_stays_nothing_selected() {
        assert_eq!(selection_after_delete(None, 0, 2), None);
    }
}
