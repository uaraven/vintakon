use iced::{Element, Settings, Task};

use crate::model::state::VintakonApp;

mod exif;
mod model;

fn main() -> iced::Result {
    iced::application("Vintakon", VintakonApp::update, VintakonApp::view)
        .settings(Settings::default())
        .run_with(VintakonApp::new)
}
