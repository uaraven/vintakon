use iced::{
    Element, Length, Task, Theme,
    widget::{Container, button, checkbox, column, progress_bar, row, scrollable, text},
};
use rfd::AsyncFileDialog;

use crate::{exif::exiftool::read_files, model::file_rec::FileRec};

#[derive(Debug, Clone)]
pub(crate) enum Message {
    PathChange(String),
    LoadImages((String, Vec<FileRec>)),
    SelectFolder,
    FileSelectionToggled(usize, bool),
    LoadingStarted,
}

#[derive(Default)]
pub(crate) struct VintakonApp {
    path: Option<String>,
    images: Vec<FileRec>,
    is_loading: bool,
}

impl VintakonApp {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
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
            }
            Message::SelectFolder => {
                return Task::perform(select_folder(), Message::PathChange);
            }
            Message::FileSelectionToggled(index, selected) => {
                if let Some(file_rec) = self.images.get_mut(index) {
                    file_rec.set_selected(selected);
                }
            }
            Message::LoadingStarted => {
                self.is_loading = true;
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message, Theme> {
        let header = row![
            text(format!(
                "{}",
                self.path
                    .as_deref()
                    .unwrap_or("<Select a folder with images>")
            ))
            .size(20)
            .width(Length::Fill),
            button("Select Folder").on_press(Message::SelectFolder),
        ]
        .padding(10)
        .spacing(10);

        let content: Element<Message, Theme> = if self.is_loading {
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
            Container::<Message, Theme>::new(text(
                "No images loaded. Select a folder to get started.",
            ))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else {
            let mut sorted_images: Vec<(usize, &FileRec)> =
                self.images.iter().enumerate().collect();
            sorted_images.sort_by(|a, b| a.1.file_name().cmp(&b.1.file_name()));

            let file_list = column(
                sorted_images
                    .into_iter()
                    .map(|(original_index, file_rec)| {
                        let mod_time_str =
                            file_rec.mod_time().format("%Y-%m-%d %H:%M:%S").to_string();

                        row![
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
                        .spacing(10)
                        .padding(5)
                        .into()
                    })
                    .collect::<Vec<Element<Message, Theme>>>(),
            )
            .spacing(5)
            .padding(10);

            scrollable(file_list)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        column![
            header,
            row![
                Container::<Message, Theme>::new(content)
                    .width(Length::FillPortion(1))
                    .height(Length::Fill),
                Container::<Message, Theme>::new(text("Right panel placeholder"))
                    .width(Length::FillPortion(1))
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
            ]
            .height(Length::Fill)
        ]
        .height(Length::Fill)
        .into()
    }
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
