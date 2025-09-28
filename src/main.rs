use iced::{
    Settings, Theme,
    window::{self, icon},
};
use image::ImageFormat;

use crate::{resources::resources::APP_ICON, state::VintakonApp};

mod exif;
mod model;
mod resources;
mod state;

fn main() -> iced::Result {
    let app_icon =
        icon::from_file_data(APP_ICON, Some(ImageFormat::Png)).expect("Failed to load app icon");

    let settings = window::Settings {
        resizable: true,
        decorations: true,
        min_size: Some(iced::Size::new(1200.0, 800.0)),
        icon: Some(app_icon),
        ..window::Settings::default()
    };

    iced::application("Vintakon", VintakonApp::update, VintakonApp::view)
        .subscription(VintakonApp::subscription)
        .theme(|_| Theme::Dark)
        .window(settings)
        .settings(Settings::default())
        .antialiasing(true)
        .run_with(VintakonApp::new)
}
