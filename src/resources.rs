#[allow(clippy::module_inception)]
pub(crate) mod resources {
    /// Toolbar icon with a variant for each theme
    #[derive(Clone, Copy)]
    pub struct Icon {
        /// Image cache keys
        pub dark_name: &'static str,
        pub light_name: &'static str,
        /// Icon to draw on a dark background
        pub on_dark: &'static [u8],
        /// Icon to draw on a light background
        pub on_light: &'static [u8],
    }

    macro_rules! icon {
        ($name:literal, $file:literal) => {
            Icon {
                dark_name: $name,
                light_name: concat!($name, "_light"),
                on_dark: include_bytes!(concat!("../assets/", $file, ".png")),
                on_light: include_bytes!(concat!("../assets/", $file, "_dark.png")),
            }
        };
    }

    /// Icon that is readable on any background
    macro_rules! colored_icon {
        ($name:literal, $file:literal) => {
            Icon {
                dark_name: $name,
                light_name: $name,
                on_dark: include_bytes!(concat!("../assets/", $file, ".png")),
                on_light: include_bytes!(concat!("../assets/", $file, ".png")),
            }
        };
    }

    pub const APP_ICON: &[u8] = include_bytes!("../assets/app_icon.png");
    pub const OPEN_FOLDER: Icon = icon!("open_folder", "folder_open");
    pub const APPLY: Icon = icon!("apply", "check");
    pub const SETTINGS: Icon = icon!("settings", "settings");
    pub const BAD_LENS: Icon = colored_icon!("bad_lens", "bad_lens");
    pub const ADD: Icon = icon!("add", "add");
    pub const DELETE: Icon = colored_icon!("delete", "delete");
    pub const OPEN: Icon = icon!("open_file", "open_file");
}
