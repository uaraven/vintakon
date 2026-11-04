//! Settings dialog: exiftool location and lens profile editor.

use freya::prelude::*;

use super::widgets::{PRIMARY, centered, colors, divider, icon, icon_button, text};
use crate::{
    logic::{is_focal_length_valid, is_valid_floating_point},
    model::lens_profile::LensProfile,
    resources::resources,
    state::AppState,
};

#[derive(PartialEq)]
pub struct SettingsDialog;

impl Component for SettingsDialog {
    fn render(&self) -> impl IntoElement {
        let state = use_consume::<AppState>();

        let exiftool_path = rect()
            .width(Size::fill())
            .padding(10.)
            .spacing(10.)
            .vertical()
            .child(text("Exiftool Path", 16.))
            .child(
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
            );

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
            .width(Size::px(828.))
            .height(Size::px(520.))
            .vertical()
            .content(Content::flex())
            .child(exiftool_path)
            .child(divider(1.))
            .child(profiles);

        Popup::new()
            // dialog width = content width + popup padding (2x8) + content padding (2x8)
            .width(Size::px(860.))
            .on_close_request(move |_| state.close_settings())
            .child(PopupTitle::new("Settings".to_string()))
            .child(PopupContent::new().child(body))
            .child(
                PopupButtons::new()
                    .child(
                        Button::new()
                            .on_press(move |_| state.close_settings())
                            .child("Cancel"),
                    )
                    .child(
                        Button::new()
                            .on_press(move |_| state.save_settings())
                            .child("Ok"),
                    ),
            )
    }
}

#[derive(PartialEq)]
struct ProfileList;

impl Component for ProfileList {
    fn render(&self) -> impl IntoElement {
        let state = use_consume::<AppState>();
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
                        .padding(2.)
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

        rect()
            .expanded()
            .vertical()
            .content(Content::flex())
            .spacing(10.)
            .child(
                ScrollView::new()
                    .width(Size::fill())
                    .height(Size::flex(1.))
                    .child(
                        rect()
                            .width(Size::fill())
                            .spacing(10.)
                            .child(text("Lens Profiles", 18.))
                            .child(divider(2.))
                            .children(items),
                    ),
            )
            .child(buttons)
    }
}

#[derive(PartialEq)]
struct ProfileEditor;

impl Component for ProfileEditor {
    fn render(&self) -> impl IntoElement {
        let state = use_consume::<AppState>();

        let Some(index) = *state.selected_profile_index.read() else {
            return centered(text("Select a profile to edit", 16.)).into_element();
        };
        let Some(profile) = state.temp_config.read().lens_profiles.get(index).cloned() else {
            return centered(text("Invalid profile selection", 16.)).into_element();
        };

        // Fields keep the text being typed, so they are created anew for every profile
        let field = |kind: Field, placeholder: &'static str| -> Element {
            ProfileField {
                index,
                kind,
                placeholder,
                initial: kind.initial_value(&profile),
                key: DiffKey::None,
            }
            .key((index, kind))
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

        ScrollView::new()
            .width(Size::fill())
            .height(Size::fill())
            .child(
                rect()
                    .width(Size::fill())
                    .padding(10.)
                    .spacing(15.)
                    .child(text("Profile Editor", 18.))
                    .child(divider(2.))
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
                    )),
            )
            .into_element()
    }
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
        let state = use_consume::<AppState>();
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
