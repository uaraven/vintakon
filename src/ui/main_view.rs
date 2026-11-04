//! Main screen: header, file list with preview and the "apply profile" pane.

use freya::{elements::image::ImageHandle, engine::prelude::AlphaType, prelude::*};

use super::widgets::{PRIMARY, centered, colors, divider, icon, icon_button, text};
use crate::{exif::exif::ExifInfo, resources::resources, state::AppState};

const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(PartialEq)]
pub struct MainView;

impl Component for MainView {
    fn render(&self) -> impl IntoElement {
        let state = use_consume::<AppState>();
        let show_apply = *state.show_apply.read();

        rect()
            .expanded()
            .vertical()
            .content(Content::flex())
            .child(MenuBar)
            .child(Header)
            .child(ContentPane)
            .maybe_child(show_apply.then(|| ApplyDialog.into_element()))
    }
}

/// Application menu bar with the "File" menu
#[derive(PartialEq)]
struct MenuBar;

impl Component for MenuBar {
    fn render(&self) -> impl IntoElement {
        let state = use_consume::<AppState>();
        let mut show_menu = use_state(|| false);

        let menu = show_menu().then(|| {
            rect()
                .position(Position::new_absolute().top(32.).left(0.))
                .child(
                    Menu::new()
                        .on_close(move |_| show_menu.set(false))
                        .child(
                            MenuButton::new()
                                .child("Open Folder...")
                                .on_press(move |_| {
                                    show_menu.set(false);
                                    state.select_folder();
                                }),
                        )
                        .child(MenuButton::new().child("Settings...").on_press(move |_| {
                            show_menu.set(false);
                            state.open_settings();
                        }))
                        .child(
                            MenuButton::new()
                                .child("Exit")
                                .on_press(move |_| std::process::exit(0)),
                        ),
                )
        });

        rect()
            .width(Size::fill())
            .horizontal()
            .padding(2.)
            .child(
                Button::new()
                    .flat()
                    .on_press(move |_| show_menu.toggle())
                    .child("File"),
            )
            .maybe_child(menu)
    }
}

/// Thin vertical rule with some room on both sides
fn vertical_separator() -> impl IntoElement {
    rect()
        .width(Size::px(1.))
        .height(Size::px(24.))
        .margin(Gaps::new(0., 8., 0., 8.))
        .background(colors().border)
}

#[derive(PartialEq)]
struct Header;

impl Component for Header {
    fn render(&self) -> impl IntoElement {
        let state = use_consume::<AppState>();
        let path = state.path.read().clone();

        let path_text = match &path {
            Some(path) => text(path.clone(), 16.),
            None => text("<Select a folder with photographs>", 16.).color(colors().text_secondary),
        };

        rect()
            .width(Size::fill())
            .vertical()
            .child(
                rect()
                    .width(Size::fill())
                    .horizontal()
                    .content(Content::flex())
                    .cross_align(Alignment::Center)
                    .padding(5.)
                    .spacing(5.)
                    .child(icon_button(
                        "Apply lens profile to selected images",
                        AttachedPosition::Bottom,
                        state.is_any_file_checked(),
                        icon(resources::APPLY),
                        move |_| {
                            let mut show = state.show_apply;
                            show.set(true);
                        },
                    ))
                    .child(vertical_separator())
                    .child(text("Location", 14.).color(colors().text_secondary))
                    .child(rect().width(Size::flex(1.)).child(path_text))
                    .child(icon_button(
                        "Select a folder with photographs",
                        AttachedPosition::Bottom,
                        true,
                        icon(resources::OPEN_FOLDER),
                        move |_| state.select_folder(),
                    ))
                    .child(vertical_separator())
                    .child(icon_button(
                        "Settings",
                        AttachedPosition::Bottom,
                        true,
                        icon(resources::SETTINGS),
                        move |_| state.open_settings(),
                    )),
            )
            .child(divider(5.))
    }
}

/// Left side contains a list of images, the right side contains their EXIF information and a preview.
/// When loading or applying a lens profile, the whole content is replaced with an indicator text.
#[derive(PartialEq)]
struct ContentPane;

impl Component for ContentPane {
    fn render(&self) -> impl IntoElement {
        let state = use_consume::<AppState>();

        let pane = rect()
            .width(Size::fill())
            .height(Size::flex(1.))
            .horizontal();

        if *state.is_applying.read() {
            pane.child(centered(text("Applying lens profile...", 16.)))
        } else if *state.is_loading.read() {
            pane.child(centered(text("Scanning folder...", 16.)))
        } else if state.images.read().is_empty() {
            pane.child(centered(text(
                "No images loaded. Select a folder to get started.",
                14.,
            )))
        } else {
            pane.content(Content::flex())
                .child(
                    rect()
                        .width(Size::flex(1.))
                        .height(Size::fill())
                        .child(FileList),
                )
                .child(
                    rect()
                        .width(Size::flex(2.))
                        .height(Size::fill())
                        .padding(10.)
                        .child(Preview),
                )
        }
    }
}

#[derive(PartialEq)]
struct FileList;

impl Component for FileList {
    fn render(&self) -> impl IntoElement {
        let state = use_consume::<AppState>();
        let count = state.images.read().len();

        ScrollView::new()
            .width(Size::fill())
            .height(Size::fill())
            .child(
                rect()
                    .width(Size::fill())
                    .padding(10.)
                    .spacing(5.)
                    .children((0..count).map(|index| FileRow::new(index).into())),
            )
    }
}

#[derive(PartialEq)]
struct FileRow {
    index: usize,
    key: DiffKey,
}

impl FileRow {
    fn new(index: usize) -> Self {
        Self {
            index,
            key: DiffKey::None,
        }
        .key(index)
    }
}

impl KeyExt for FileRow {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for FileRow {
    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }

    fn render(&self) -> impl IntoElement {
        let state = use_consume::<AppState>();
        let index = self.index;

        let images = state.images.read();
        let Some(file_rec) = images.get(index) else {
            return rect().into_element();
        };
        let is_checked = file_rec.is_selected();
        let has_lens_data = file_rec.has_lens_data();
        let file_name = file_rec.file_name();
        let mod_time = file_rec.mod_time().format(TIME_FORMAT).to_string();
        drop(images);

        let is_current = *state.selected_file_index.read() == Some(index);

        let checkbox = rect()
            .on_press(move |e: Event<PressEventData>| {
                // do not select the row when only the checkbox is toggled
                e.stop_propagation();
                state.set_file_checked(index, !is_checked);
            })
            .child(Checkbox::new().selected(is_checked));

        CursorArea::new()
            .child(
                rect()
                    .width(Size::fill())
                    .padding(5.)
                    .horizontal()
                    .content(Content::flex())
                    .cross_align(Alignment::Center)
                    .spacing(10.)
                    .background(if is_current {
                        PRIMARY.into()
                    } else {
                        Color::TRANSPARENT
                    })
                    .color(if is_current {
                        Color::WHITE
                    } else {
                        colors().text_primary
                    })
                    .on_press(move |_| {
                        let mut selected = state.selected_file_index;
                        selected.set(Some(index));
                    })
                    .child(checkbox)
                    .child(
                        rect()
                            .width(Size::flex(1.))
                            .spacing(2.)
                            .child(text(file_name, 16.))
                            .child(text(mod_time, 12.)),
                    )
                    .maybe_child((!has_lens_data).then(|| icon(resources::BAD_LENS))),
            )
            .into_element()
    }
}

#[derive(PartialEq)]
struct Preview;

impl Component for Preview {
    fn render(&self) -> impl IntoElement {
        let state = use_consume::<AppState>();

        // Convert the preview only when the selection changes
        let preview = use_memo(move || {
            let index = (*state.selected_file_index.read())?;
            let images = state.images.read();
            let preview_img = images.get(index)?.exif_info().preview_image.as_ref()?;
            ImageHandle::from_rgba(
                preview_img.width(),
                preview_img.height(),
                preview_img.to_rgba8().into_raw().into(),
                AlphaType::Unpremul,
            )
        });

        let Some(index) = *state.selected_file_index.read() else {
            return centered(text("Select a file to view preview and metadata", 14.))
                .into_element();
        };

        let images = state.images.read();
        let Some(selected_file) = images.get(index) else {
            return text("Selected file not found", 14.).into_element();
        };
        let exif_info = selected_file.exif_info();

        ScrollView::new()
            .width(Size::fill())
            .height(Size::fill())
            .child(
                rect()
                    .width(Size::fill())
                    .padding(10.)
                    .spacing(10.)
                    .child(text(selected_file.file_name(), 18.))
                    .child(text(
                        format!("Modified: {}", selected_file.mod_time().format(TIME_FORMAT)),
                        12.,
                    ))
                    .maybe_child(preview.read().clone().map(|handle| {
                        image(handle)
                            .width(Size::fill())
                            .aspect_ratio(AspectRatio::Min)
                    }))
                    .child(metadata(exif_info)),
            )
            .into_element()
    }
}

fn metadata(exif_info: &ExifInfo) -> impl IntoElement {
    let fields: [(&str, Option<String>); 9] = [
        ("Make", exif_info.make.clone()),
        ("Model", exif_info.model.clone()),
        ("Lens Make", exif_info.lens_make.clone()),
        ("Lens Model", exif_info.lens_model.clone()),
        ("ISO", exif_info.iso.as_ref().map(|v| v.to_string())),
        (
            "Aperture",
            exif_info.aperture.as_ref().map(|v| v.to_string()),
        ),
        (
            "Shutter Speed",
            exif_info.shutter_speed.as_ref().map(|v| v.to_string()),
        ),
        (
            "Focal Length",
            exif_info.focal_length.as_ref().map(|v| v.to_string()),
        ),
        (
            "Focal Length (35mm)",
            exif_info.focal_length_35mm.as_ref().map(|v| v.to_string()),
        ),
    ];

    rect().spacing(5.).children(
        fields
            .into_iter()
            .filter_map(|(name, value)| value.map(|v| text(format!("{name}: {v}"), 12.).into())),
    )
}

/// Modal dialog with the lens profile selection and the actual aperture / focal length
#[derive(PartialEq)]
struct ApplyDialog;

impl Component for ApplyDialog {
    fn render(&self) -> impl IntoElement {
        let state = use_consume::<AppState>();

        let selected_profile = state.selected_profile.read().clone();
        let profile_select = Select::new()
            .selected_item(selected_profile.clone().unwrap_or_default())
            .children(state.profile_names().into_iter().map(|name| {
                let is_selected = selected_profile.as_deref() == Some(name.as_str());
                let mut selected = state.selected_profile;
                let value = name.clone();
                MenuItem::new()
                    .selected(is_selected)
                    .on_press(move |_| selected.set(Some(value.clone())))
                    .child(name)
                    .into()
            }));

        let aperture = Input::new(state.apply_aperture)
            .placeholder("Aperture")
            .width(Size::fill())
            .on_validate(|validator: InputValidator| {
                if !crate::logic::is_valid_floating_point(&validator.text()) {
                    validator.set_valid(false);
                }
            });
        let focal_length = Input::new(state.apply_focal_length)
            .placeholder("Focal Length")
            .width(Size::fill())
            .on_validate(|validator: InputValidator| {
                if !crate::logic::is_focal_length_valid(&validator.text()) {
                    validator.set_valid(false);
                }
            });

        let labeled = |caption: &'static str, control: Element| {
            rect()
                .width(Size::fill())
                .horizontal()
                .content(Content::flex())
                .cross_align(Alignment::Center)
                .spacing(10.)
                .child(rect().width(Size::px(150.)).child(text(caption, 14.)))
                .child(rect().width(Size::flex(1.)).child(control))
        };

        let close = move || {
            let mut show = state.show_apply;
            show.set(false);
        };

        Popup::new()
            .on_close_request(move |_| close())
            .child(PopupTitle::new("Apply profile".to_string()))
            .child(
                PopupContent::new().child(
                    rect()
                        .width(Size::px(480.))
                        .spacing(12.)
                        .child(labeled("Profile:", profile_select.into()))
                        .child(labeled("Actual Aperture:", aperture.into()))
                        .child(labeled("Actual Focal Length:", focal_length.into())),
                ),
            )
            .child(
                PopupButtons::new()
                    .child(Button::new().on_press(move |_| close()).child("Cancel"))
                    .child(
                        Button::new()
                            .enabled(state.selected_profile.read().is_some())
                            .on_press(move |_| {
                                close();
                                state.apply_profile();
                            })
                            .child("Apply"),
                    ),
            )
    }
}
