use iced::{
    Element, Length, Task,
    widget::{button, row, text},
};
use rfd::AsyncFileDialog;

use crate::{exif::exiftool::read_files, model::file_rec::FileRec};

#[derive(Debug, Clone)]
pub(crate) enum Message {
    PathChange(String),
    LoadImages((String, Vec<FileRec>)),
    SelectFolder,
}

#[derive(Default)]
pub(crate) struct VintakonApp {
    path: Option<String>,
    images: Vec<FileRec>,
}

impl VintakonApp {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PathChange(new_path) => {
                return Task::perform(load_images(new_path), Message::LoadImages);
            }
            Message::LoadImages(folder_images) => {
                self.images = folder_images.1;
                self.path = Some(folder_images.0);
            }
            Message::SelectFolder => {
                return Task::perform(select_folder(), Message::PathChange);
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        row![
            text(format!(
                "Current Path: {}",
                self.path.as_deref().unwrap_or("No Path Selected")
            ))
            .size(24)
            .width(Length::Fill),
            button("Select Folder").on_press(Message::SelectFolder),
        ]
        .padding(20)
        .spacing(10)
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
