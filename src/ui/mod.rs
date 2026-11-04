mod main_view;
mod settings;
mod widgets;

use freya::prelude::*;

use crate::state::AppState;

/// Root component. Shows either the main screen or the settings dialog
pub fn app() -> impl IntoElement {
    let state = AppState::new();
    use_provide_context(|| state);

    // Follow the light/dark setting of the OS
    let preferred = Platform::get().preferred_theme;
    let mut theme =
        use_init_theme(move || widgets::theme_for(widgets::initial_theme(*preferred.peek())));
    // Only react to changes reported by the window system after the start
    let mut last = use_state(move || *preferred.peek());
    use_side_effect(move || {
        let now = *preferred.read();
        if now != *last.peek() {
            last.set(now);
            theme.set(widgets::theme_for(now));
        }
    });
    let colors = widgets::colors();

    let screen: Element = if *state.show_settings.read() {
        settings::SettingsDialog.into()
    } else {
        main_view::MainView.into()
    };

    rect()
        .expanded()
        .background(colors.background)
        .color(colors.text_primary)
        .child(screen)
}
