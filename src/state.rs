use iced::{
    Element, Length, Task, Theme,
    widget::{
        Container, Rule, button, checkbox, column, image, pick_list, row, scrollable, text,
        text_input,
    },
};
use rfd::AsyncFileDialog;

use crate::{
    exif::exiftool::read_files,
    model::{file_rec::FileRec, lens_profile::Config},
    resources::resources,
};

#[derive(Debug, Clone)]
pub(crate) enum Message {
    PathChange(String),
    LoadImages((String, Vec<FileRec>)),
    SelectFolder,
    FileSelectionToggled(usize, bool),
    FileClicked(usize),
    ProfileChange(String),
    ApertureChange(String),
}

#[derive(Default)]
pub(crate) struct VintakonApp {
    path: Option<String>,
    images: Vec<FileRec>,
    is_loading: bool,
    selected_file_index: Option<usize>,
    config: Option<Config>,
    profile_names: Vec<String>,
    selected_profile: Option<String>,
    apply_aperture: String,
}

impl VintakonApp {
    pub fn new() -> (Self, Task<Message>) {
        let config = Config::load_with_fallback();
        (
            Self {
                path: None,
                images: Vec::new(),
                is_loading: false,
                selected_file_index: None,
                config: Some(config.clone()),
                profile_names: config
                    .lens_profiles
                    .iter()
                    .map(|p| p.name.clone())
                    .collect(),
                selected_profile: None,
                apply_aperture: String::new(),
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PathChange(new_path) => {
                self.is_loading = true;
                return Task::perform(load_images(new_path), Message::LoadImages);
            }
            Message::LoadImages(folder_images) => {
                self.images = folder_images.1;
                self.path = Some(folder_images.0);
                self.is_loading = false;
                self.selected_file_index = None;
            }
            Message::SelectFolder => {
                return Task::perform(select_folder(), Message::PathChange);
            }
            Message::FileSelectionToggled(index, selected) => {
                if let Some(file_rec) = self.images.get_mut(index) {
                    file_rec.set_selected(selected);
                }
            }
            Message::FileClicked(index) => {
                self.selected_file_index = Some(index);
            }
            Message::ProfileChange(profile) => {
                self.selected_profile = Some(profile);
            }
            Message::ApertureChange(aperture) => {
                self.apply_aperture = aperture;
            }
        }
        Task::none()
    }

    /// Renders the file list
    fn view_file_list(&self) -> Element<Message, Theme> {
        let file_list = column(
            self.images
                .iter()
                .enumerate()
                .map(|(original_index, file_rec)| self.view_file_row(original_index, file_rec))
                .collect::<Vec<Element<Message, Theme>>>(),
        )
        .spacing(5)
        .padding(10);

        scrollable(file_list)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Renders the single row of file list
    fn view_file_row(&self, original_index: usize, file_rec: &FileRec) -> Element<Message, Theme> {
        let mod_time_str = file_rec.mod_time().format("%Y-%m-%d %H:%M:%S").to_string();

        let is_selected_file = self.selected_file_index == Some(original_index);

        let file_row = row![
            checkbox("", file_rec.is_selected()).on_toggle(move |selected| {
                Message::FileSelectionToggled(original_index, selected)
            }),
            column![
                text(file_rec.file_name()).size(16),
                text(mod_time_str).size(12)
            ]
            .spacing(2)
            .width(Length::Fill)
        ]
        .spacing(10);

        if is_selected_file {
            Container::new(file_row)
                .padding(5)
                .style(|theme: &Theme| {
                    use iced::Background;
                    iced::widget::container::Style {
                        background: Some(Background::Color(theme.palette().primary)),
                        ..Default::default()
                    }
                })
                .into()
        } else {
            button(file_row)
                .padding(5)
                .style(|theme: &Theme, _status| iced::widget::button::Style {
                    background: None,
                    text_color: theme.palette().text,
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                })
                .on_press(Message::FileClicked(original_index))
                .into()
        }
    }

    fn view_image_preview(&self) -> Element<Message, Theme> {
        if let Some(selected_index) = self.selected_file_index {
            if let Some(selected_file) = self.images.get(selected_index) {
                let exif_info = selected_file.exif_info();

                let mut preview_content = column![
                    text(selected_file.file_name()).size(18),
                    text(format!(
                        "Modified: {}",
                        selected_file.mod_time().format("%Y-%m-%d %H:%M:%S")
                    ))
                    .size(12)
                ]
                .spacing(10)
                .padding(10);

                if let Some(preview_img) = &exif_info.preview_image {
                    let img_handle = image::Handle::from_rgba(
                        preview_img.width(),
                        preview_img.height(),
                        preview_img.to_rgba8().into_raw(),
                    );

                    preview_content = preview_content.push(
                        image(img_handle)
                            .width(Length::Fill)
                            .content_fit(iced::ContentFit::Contain),
                    );
                }

                let metadata = view_metadata(exif_info);

                preview_content = preview_content.push(metadata);

                let scrollable_content: Element<Message, Theme> = scrollable(preview_content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into();
                scrollable_content
            } else {
                text("Selected file not found").into()
            }
        } else {
            Container::<Message, Theme>::new(
                text("Select a file to view preview and metadata")
                    .align_x(iced::Alignment::Center)
                    .align_y(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        }
    }

    fn view_content_area(&self) -> Element<Message, Theme> {
        if self.is_loading {
            Container::<Message, Theme>::new(
                column![text("Loading images...").size(16)]
                    .spacing(10)
                    .align_x(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else if self.images.is_empty() {
            Container::<Message, Theme>::new(
                text("No images loaded. Select a folder to get started.")
                    .align_x(iced::Alignment::Center)
                    .align_y(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else {
            self.view_file_list()
        }
    }

    fn view_header(&self) -> Element<Message, Theme> {
        let mut text = text(format!(
            "{}",
            self.path
                .as_deref()
                .unwrap_or("<Select a folder with images>")
        ))
        .size(16)
        .width(Length::Fill);
        if self.path.is_none() {
            text = text.color(iced::Color::from_rgb(0.7, 0.7, 0.7));
        }
        column![
            row![
                text,
                button(
                    image(image::Handle::from_bytes(resources::OPEN_FOLDER))
                        .width(24)
                        .height(24)
                )
                .on_press(Message::SelectFolder),
            ]
            .align_y(iced::Alignment::Center)
            .padding(5)
            .spacing(5),
            Rule::horizontal(5),
        ]
        .into()
    }

    pub fn view(&self) -> Element<Message, Theme> {
        let header = self.view_header();
        let content = self.view_content_area();
        let preview = self.view_image_preview();
        let apply_pane = self.view_apply_profile();

        column![
            header,
            row![
                Container::<Message, Theme>::new(content)
                    .width(Length::FillPortion(1))
                    .height(Length::Fill),
                Container::<Message, Theme>::new(preview)
                    .width(Length::FillPortion(1))
                    .height(Length::Fill)
                    .padding(10)
            ]
            .height(Length::Fill),
            apply_pane,
        ]
        .height(Length::Fill)
        .into()
    }

    fn view_apply_profile(&self) -> iced::widget::Column<'_, Message> {
        column![
            text("Apply Lens Profile").size(16),
            Rule::horizontal(5),
            row![
                column![
                    row![
                        text("Profile: ").size(14),
                        pick_list(
                            self.profile_names.clone(),
                            self.selected_profile.clone(),
                            Message::ProfileChange,
                        )
                        .width(Length::Fill),
                    ]
                    .spacing(10)
                    .padding(5)
                    .align_y(iced::Alignment::Center),
                    row![
                        text("Aperture: ").size(14),
                        text_input("Aperture", &self.apply_aperture)
                            .on_input(Message::ApertureChange)
                            .width(Length::Fill),
                    ]
                    .spacing(10)
                    .padding(5)
                    .align_y(iced::Alignment::Center),
                ],
                button(
                    image(image::Handle::from_bytes(resources::APPLY))
                        .width(24)
                        .height(24),
                )
                .padding(10),
                button(
                    image(image::Handle::from_bytes(resources::SETTINGS))
                        .width(24)
                        .height(24),
                )
                .padding(10),
            ]
            .spacing(10),
        ]
        .spacing(10)
        .padding(10)
        .align_x(iced::Alignment::Center)
    }
}

fn view_metadata(exif_info: &crate::exif::exif::ExifInfo) -> iced::widget::Column<'_, Message> {
    // Add metadata section
    let mut metadata = column![].spacing(5);

    if let Some(make) = &exif_info.make {
        metadata = metadata.push(text(format!("Make: {}", make)).size(12));
    }
    if let Some(model) = &exif_info.model {
        metadata = metadata.push(text(format!("Model: {}", model)).size(12));
    }
    if let Some(lens_make) = &exif_info.lens_make {
        metadata = metadata.push(text(format!("Lens Make: {}", lens_make)).size(12));
    }
    if let Some(lens_model) = &exif_info.lens_model {
        metadata = metadata.push(text(format!("Lens Model: {}", lens_model)).size(12));
    }
    if let Some(iso) = &exif_info.iso {
        metadata = metadata.push(text(format!("ISO: {}", iso)).size(12));
    }
    if let Some(aperture) = &exif_info.aperture {
        metadata = metadata.push(text(format!("Aperture: {}", aperture)).size(12));
    }
    if let Some(shutter_speed) = &exif_info.shutter_speed {
        metadata = metadata.push(text(format!("Shutter Speed: {}", shutter_speed)).size(12));
    }
    if let Some(focal_length) = &exif_info.focal_length {
        metadata = metadata.push(text(format!("Focal Length: {}", focal_length)).size(12));
    }
    metadata
}

async fn load_images(new_path: String) -> (String, Vec<FileRec>) {
    let exif_data_list = read_files(&new_path);
    let file_recs = exif_data_list
        .iter()
        .map(|exif| FileRec::new(exif.clone()))
        .collect();
    (new_path, file_recs)
}

async fn select_folder() -> String {
    if let Some(folder) = AsyncFileDialog::new().pick_folder().await {
        folder.path().to_string_lossy().to_string()
    } else {
        String::new()
    }
}
