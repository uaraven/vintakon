use iced::{
    Settings,
    window::{self, icon},
};
use image::ImageFormat;

use crate::{model::state::VintakonApp, resources::resources::APP_ICON};

mod exif;
mod model;
mod resources;

fn main() -> iced::Result {
    let app_icon =
        icon::from_file_data(APP_ICON, Some(ImageFormat::Png)).expect("Failed to load app icon");

    let settings = window::Settings {
        
        resizable: true,
        decorations: true,
        icon: Some(app_icon),
        ..window::Settings::default()
    };

    iced::application("Vintakon", VintakonApp::update, VintakonApp::view)
        .window(settings)
        .settings(Settings::default())
        .run_with(VintakonApp::new)
}
