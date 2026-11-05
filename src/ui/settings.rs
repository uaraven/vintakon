//! Settings dialog: exiftool location and lens profile editor.

use freya::prelude::*;

use super::widgets::{PRIMARY, card, colors, icon, icon_button, text};
use crate::{
    logic::{is_focal_length_valid, is_valid_floating_point},
    model::lens_profile::LensProfile,
    resources,
    settings_state::SettingsState,
    state::AppState,
};

#[derive(PartialEq)]
pub struct SettingsDialog;

impl Component for SettingsDialog {
    fn render(&self) -> impl IntoElement {
        let state = SettingsState::new(use_consume::<AppState>());
        use_provide_context(|| state);

        let exiftool_path = rect().width(Size::fill()).padding(10.).child(card(
            "Exiftool Path".to_string(),
            rect()
                .width(Size::fill())
                .horizontal()
                .content(Content::flex())
                .cross_align(Alignment::Center)
                .spacing(10.)
                .child(
                    rect().width(Size::flex(1.)).child(
                        Input::new(state.exiftool_text)
                            .placeholder("Path to exiftool executable")
                            .width(Size::fill())
                            .on_validate(move |validator: InputValidator| {
                                state.edit_exiftool_path(validator.text().clone());
                            }),
                    ),
                )
                .child(icon_button(
                    "Select location of 'exiftool' utility",
                    AttachedPosition::Left,
                    true,
                    icon(resources::OPEN),
                    move |_| state.select_exiftool_path(),
                )),
        ));

        let profiles = rect()
            .width(Size::fill())
            .height(Size::flex(1.))
            .horizontal()
            .content(Content::flex())
            .child(
                rect()
                    .width(Size::flex(1.))
                    .height(Size::fill())
                    .padding(10.)
                    .child(ProfileList),
            )
            .child(
                rect()
                    .width(Size::flex(2.))
                    .height(Size::fill())
                    .padding(10.)
                    .child(ProfileEditor),
            );

        let body = rect()
            .width(Size::fill())
            .height(Size::flex(1.))
            .padding(8.)
            .vertical()
            .content(Content::flex())
            .child(exiftool_path)
            .child(profiles);

        // The popup takes 90% of the window, its content is a flex column so that
        // the body gets all the space left between the title and the buttons
        Popup::new()
            .width(Size::window_percent(90.))
            .height(Size::window_percent(90.))
            .on_close_request(move |_| state.close())
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::fill())
                    .vertical()
                    .content(Content::flex())
                    .child(PopupTitle::new("Settings".to_string()))
                    .child(body)
                    .child(
                        PopupButtons::new()
                            .child(
                                Button::new()
                                    .on_press(move |_| state.close())
                                    .child("Cancel"),
                            )
                            .child(Button::new().on_press(move |_| state.save()).child("Ok")),
                    ),
            )
    }
}

#[derive(PartialEq)]
struct ProfileList;

impl Component for ProfileList {
    fn render(&self) -> impl IntoElement {
        let state = use_consume::<SettingsState>();
        let selected_index = *state.selected_profile_index.read();

        let names: Vec<String> = state
            .temp_config
            .read()
            .lens_profiles
            .iter()
            .map(|p| p.name.clone())
            .collect();

        let items = names.into_iter().enumerate().map(|(index, name)| {
            let is_selected = selected_index == Some(index);
            CursorArea::new()
                .key(index)
                .child(
                    rect()
                        .width(Size::fill())
                        .padding(12.)
                        .background(if is_selected {
                            PRIMARY.into()
                        } else {
                            Color::TRANSPARENT
                        })
                        .color(if is_selected {
                            Color::WHITE
                        } else {
                            colors().text_primary
                        })
                        .on_press(move |_| {
                            let mut selected = state.selected_profile_index;
                            selected.set(Some(index));
                        })
                        .child(text(name, 14.)),
                )
                .into()
        });

        let buttons = rect()
            .horizontal()
            .spacing(10.)
            .child(icon_button(
                "Add new profile",
                AttachedPosition::Top,
                true,
                icon(resources::ADD),
                move |_| state.add_profile(),
            ))
            .child(icon_button(
                "Delete selected profile",
                AttachedPosition::Top,
                selected_index.is_some(),
                icon(resources::DELETE),
                move |_| {
                    if let Some(index) = selected_index {
                        state.delete_profile(index);
                    }
                },
            ));

        card(
            "Lens Profiles".to_string(),
            rect()
                .width(Size::fill())
                .height(Size::flex(1.))
                .vertical()
                .content(Content::flex())
                .spacing(10.)
                .child(
                    ScrollView::new()
                        .width(Size::fill())
                        .height(Size::flex(1.))
                        .child(rect().width(Size::fill()).spacing(10.).children(items)),
                )
                .child(buttons),
        )
        .height(Size::fill())
    }
}

#[derive(PartialEq)]
struct ProfileEditor;

impl Component for ProfileEditor {
    fn render(&self) -> impl IntoElement {
        let state = use_consume::<SettingsState>();

        let Some(index) = *state.selected_profile_index.read() else {
            return placeholder("Select a profile to edit");
        };
        let Some(profile) = state.temp_config.read().lens_profiles.get(index).cloned() else {
            return placeholder("Invalid profile selection");
        };

        // Fields keep the text being typed, so they are created anew for every profile
        let generation = *state.profile_generation.read();
        let field = |kind: Field, placeholder: &'static str| -> Element {
            ProfileField {
                index,
                kind,
                placeholder,
                initial: kind.initial_value(&profile),
                key: DiffKey::None,
            }
            .key((generation, index, kind))
            .into()
        };
        let labeled = |caption: &'static str, input: Element| {
            rect()
                .width(Size::fill())
                .spacing(5.)
                .child(text(caption, 12.))
                .child(input)
        };
        let range = |caption: &'static str, min: Element, max: Element| {
            rect()
                .width(Size::fill())
                .spacing(10.)
                .child(text(caption, 12.))
                .child(
                    rect()
                        .width(Size::fill())
                        .horizontal()
                        .content(Content::flex())
                        .cross_align(Alignment::Center)
                        .spacing(10.)
                        .child(rect().width(Size::flex(1.)).child(min))
                        .child(text("-", 14.))
                        .child(rect().width(Size::flex(1.)).child(max)),
                )
        };

        let form = rect()
            .width(Size::fill())
            .spacing(15.)
            .child(labeled("Name:", field(Field::Name, "Profile name")))
            .child(
                rect()
                    .width(Size::fill())
                    .horizontal()
                    .content(Content::flex())
                    .spacing(10.)
                    .child(
                        rect()
                            .width(Size::flex(1.))
                            .child(labeled("Make:", field(Field::Make, "Lens make"))),
                    )
                    .child(
                        rect()
                            .width(Size::flex(1.))
                            .child(labeled("Model:", field(Field::Model, "Lens model"))),
                    ),
            )
            .child(range(
                "Focal Length Range",
                field(Field::MinFocal, "Min"),
                field(Field::MaxFocal, "Max"),
            ))
            .child(range(
                "Max aperture range",
                field(Field::MinAperture, "Min aperture"),
                field(Field::MaxAperture, "Max aperture"),
            ))
            .child(labeled(
                "Crop Factor:",
                field(Field::CropFactor, "Crop factor"),
            ));

        card(
            "Profile Editor".to_string(),
            ScrollView::new()
                .width(Size::fill())
                .height(Size::flex(1.))
                .child(form),
        )
        .height(Size::fill())
        .into_element()
    }
}

/// Editor card that tells why there is nothing to edit
fn placeholder(message: &'static str) -> Element {
    card(
        "Profile Editor".to_string(),
        rect()
            .width(Size::fill())
            .height(Size::flex(1.))
            .center()
            .child(text(message, 16.)),
    )
    .height(Size::fill())
    .into_element()
}

#[derive(Clone, Copy, PartialEq, Hash)]
enum Field {
    Name,
    Make,
    Model,
    MinFocal,
    MaxFocal,
    MinAperture,
    MaxAperture,
    CropFactor,
}

impl Field {
    fn initial_value(self, profile: &LensProfile) -> String {
        match self {
            Field::Name => profile.name.clone(),
            Field::Make => profile.lens_make.clone(),
            Field::Model => profile.lens_model.clone(),
            Field::MinFocal => profile.min_focal_length.to_string(),
            Field::MaxFocal => profile.max_focal_length.to_string(),
            Field::MinAperture => profile.min_aperture.clone(),
            Field::MaxAperture => profile.max_aperture.clone(),
            Field::CropFactor => profile.crop_factor.to_string(),
        }
    }

    /// Checks whether the text is allowed in the field
    fn accepts(self, text: &str) -> bool {
        match self {
            Field::Name | Field::Make | Field::Model => true,
            Field::MinFocal | Field::MaxFocal => is_focal_length_valid(text),
            Field::MinAperture | Field::MaxAperture | Field::CropFactor => {
                is_valid_floating_point(text)
            }
        }
    }

    /// Stores the (already validated) text in the profile
    fn store(self, profile: &mut LensProfile, text: String) {
        match self {
            Field::Name => profile.name = text,
            Field::Make => profile.lens_make = text,
            Field::Model => profile.lens_model = text,
            Field::MinFocal => profile.min_focal_length = text.parse().unwrap_or_default(),
            Field::MaxFocal => profile.max_focal_length = text.parse().unwrap_or_default(),
            Field::MinAperture => profile.min_aperture = text,
            Field::MaxAperture => profile.max_aperture = text,
            Field::CropFactor => profile.crop_factor = text.parse().unwrap_or(1.0),
        }
    }
}

/// Text input bound to one field of the profile in the temporary config
#[derive(PartialEq)]
struct ProfileField {
    index: usize,
    kind: Field,
    placeholder: &'static str,
    initial: String,
    key: DiffKey,
}

impl KeyExt for ProfileField {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for ProfileField {
    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }

    fn render(&self) -> impl IntoElement {
        let state = use_consume::<SettingsState>();
        let text = use_state(|| self.initial.clone());
        let (index, kind) = (self.index, self.kind);

        Input::new(text)
            .placeholder(self.placeholder)
            .width(Size::fill())
            .on_validate(move |validator: InputValidator| {
                let new_text = validator.text().clone();
                if kind.accepts(&new_text) {
                    state.edit_profile(index, |profile| kind.store(profile, new_text));
                } else {
                    validator.set_valid(false);
                }
            })
    }
}
