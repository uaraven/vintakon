use freya::prelude::*;

use crate::resources::APP_ICON;

mod exif;
mod logic;
mod model;
mod resources;
mod settings_state;
mod state;
mod ui;

fn main() {
    // Freya has its own executor, tokio is used for the background work (exiftool, dialogs)
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to start tokio runtime");
    let _guard = runtime.enter();

    launch(
        LaunchConfig::new().with_window(
            WindowConfig::new(ui::app)
                .with_title("Vintakon")
                .with_size(1200., 800.)
                .with_min_size(1200., 800.)
                .with_icon(LaunchConfig::window_icon(APP_ICON))
                .with_app_id("vintakon"),
        ),
    )
}
