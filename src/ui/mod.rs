mod main_view;
mod settings;
mod widgets;

use freya::prelude::*;

use crate::state::AppState;

/// Root component. Shows the main screen, with the settings dialog on top of it when open
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

    let show_settings = *state.show_settings.read();

    rect()
        .expanded()
        .background(colors.background)
        .color(colors.text_primary)
        .child(main_view::MainView)
        .maybe_child(show_settings.then(|| settings::SettingsDialog.into_element()))
}
