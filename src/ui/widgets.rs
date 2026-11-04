//! Small reusable pieces shared by the screens.

use freya::prelude::*;

use crate::resources::resources::Icon;

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
        .child(
            Button::new()
                .enabled(enabled)
                .on_press(on_press)
                .child(icon),
        )
}

/// A box that centers its content in all the available space
pub fn centered(child: impl IntoElement) -> impl IntoElement {
    rect().expanded().center().child(child)
}
