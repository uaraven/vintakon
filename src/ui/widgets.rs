//! Small reusable pieces shared by the screens.

use freya::prelude::*;

use crate::resources::Icon;

/// Color of the "primary" accent, used for the selected list items
pub const PRIMARY: (u8, u8, u8) = (88, 101, 242);

/// Colors of the active (light or dark) theme
pub fn colors() -> ColorsSheet {
    get_theme_or_default().read().colors.clone()
}

fn is_dark() -> bool {
    get_theme_or_default().read().name == "dark"
}

/// Asks the desktop whether it uses a dark theme. winit does not report it on
/// many Linux setups, so the freedesktop portal and GNOME settings are queried.
#[cfg(target_os = "linux")]
fn linux_prefers_dark() -> Option<bool> {
    use std::process::Command;
    let run = |cmd: &str, args: &[&str]| {
        Command::new(cmd)
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
    };
    // color-scheme: 1 = dark, 2 = light, 0 = no preference
    let portal = run(
        "gdbus",
        &[
            "call",
            "--session",
            "--dest",
            "org.freedesktop.portal.Desktop",
            "--object-path",
            "/org/freedesktop/portal/desktop",
            "--method",
            "org.freedesktop.portal.Settings.Read",
            "org.freedesktop.appearance",
            "color-scheme",
        ],
    );
    if let Some(out) = portal {
        if out.contains("uint32 1") {
            return Some(true);
        }
        if out.contains("uint32 2") {
            return Some(false);
        }
    }
    let gnome = run(
        "gsettings",
        &["get", "org.gnome.desktop.interface", "color-scheme"],
    )?;
    Some(gnome.contains("dark"))
}

/// Theme the application starts with
pub fn initial_theme(reported: PreferredTheme) -> PreferredTheme {
    #[cfg(target_os = "linux")]
    if reported == PreferredTheme::Light && linux_prefers_dark() == Some(true) {
        return PreferredTheme::Dark;
    }
    reported
}

/// Theme for the preferred (OS) setting
pub fn theme_for(preferred: PreferredTheme) -> Theme {
    match preferred {
        PreferredTheme::Dark => dark_theme(),
        PreferredTheme::Light => light_theme(),
    }
}

/// 24x24 icon from the embedded PNG resources, matching the theme
pub fn icon(icon: Icon) -> impl IntoElement {
    let source = if is_dark() {
        (icon.dark_name, icon.on_dark)
    } else {
        (icon.light_name, icon.on_light)
    };
    ImageViewer::new(source)
        .width(Size::px(24.))
        .height(Size::px(24.))
}

/// Horizontal rule
pub fn divider(thickness: f32) -> impl IntoElement {
    rect()
        .width(Size::fill())
        .height(Size::px(thickness))
        .background(colors().border)
}

/// Text with an explicit size
pub fn text(content: impl Into<std::borrow::Cow<'static, str>>, size: f32) -> Label {
    label().font_size(size).text(content)
}

/// Button that is clearly grayed out when disabled (the default theme only dims it by 10%)
pub fn button(enabled: bool) -> Button {
    let button = Button::new().enabled(enabled);
    if enabled {
        return button;
    }
    let (background, text) = if is_dark() {
        (Color::from_rgb(58, 58, 58), Color::from_rgb(120, 120, 120))
    } else {
        (
            Color::from_rgb(220, 220, 220),
            Color::from_rgb(150, 150, 150),
        )
    };
    button.theme_colors(
        ButtonColorsThemePartial::new()
            .background(background)
            .hover_background(background)
            .border_fill(background)
            .color(text),
    )
}

/// A rounded panel for the content. To make the content scroll or stretch,
/// give it a flexible height (`Size::flex`) and the panel a fixed one
pub fn panel() -> Rect {
    rect()
        .width(Size::fill())
        .vertical()
        .content(Content::flex())
        .padding(15.)
        .spacing(12.)
        .background(colors().surface_secondary)
        .corner_radius(8.)
        .border(Border::new().fill(colors().border).width(1.))
}

/// A panel with a title and a content area
pub fn card(title: String, content: impl IntoElement) -> Rect {
    panel()
        .child(text(title, 16.))
        .child(divider(1.))
        .child(content)
}

/// Icon button with a tooltip
pub fn icon_button(
    tooltip: &'static str,
    position: AttachedPosition,
    enabled: bool,
    icon: impl IntoElement,
    on_press: impl FnMut(Event<PressEventData>) + 'static,
) -> impl IntoElement {
    TooltipContainer::new(Tooltip::new(tooltip))
        .position(position)
        .child(button(enabled).on_press(on_press).child(icon))
}

/// A box that centers its content in all the available space
pub fn centered(child: impl IntoElement) -> impl IntoElement {
    rect().expanded().center().child(child)
}

/// Width of the menu item content, wide enough for the longest caption with its shortcut
const MENU_ITEM_WIDTH: f32 = 340.;

/// Content of a menu item: the caption on the left and the keyboard shortcut (Cmd on macOS,
/// Ctrl elsewhere) on the right. It keeps the default font size and color of the menu, so all
/// the items look the same; disabled items are dimmed (`MenuButton` has no disabled state)
pub fn menu_label(
    caption: &'static str,
    shortcut_key: Option<&'static str>,
    enabled: bool,
) -> Rect {
    let modifier = if cfg!(target_os = "macos") {
        "Cmd"
    } else {
        "Ctrl"
    };
    let row = rect()
        .width(Size::px(MENU_ITEM_WIDTH))
        .horizontal()
        .content(Content::flex())
        .spacing(20.)
        .child(rect().width(Size::flex(1.)).child(label().text(caption)))
        .maybe_child(shortcut_key.map(|key| label().text(format!("{modifier}+{key}"))));
    if enabled {
        row
    } else {
        row.color(colors().text_secondary)
    }
}
