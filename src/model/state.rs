use iced::{
    Element, Task,
    widget::{button, row, text},
};

use crate::model::file_rec::FileRec;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    PathChange(String),
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

    fn title(&self) -> String {
        String::from("Vintakon")
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PathChange(new_path) => self.path = Some(new_path),
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        row![
            text(format!(
                "Current Path: {}",
                self.path.as_deref().unwrap_or("No Path Selected")
            ))
            .size(30),
            button("Select Path").on_press(Message::PathChange("New/Path/To/Image.jpg".into())),
        ]
        .padding(20)
        .spacing(10)
        .into()
    }
}
