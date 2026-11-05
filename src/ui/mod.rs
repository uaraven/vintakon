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
    let error_message = state.error_message.read().clone();

    rect()
        .expanded()
        .background(colors.background)
        .color(colors.text_primary)
        .child(main_view::MainView)
        .maybe_child(show_settings.then(|| settings::SettingsDialog.into_element()))
        .maybe_child(error_message.map(|message| ErrorDialog(message).into_element()))
}

/// Modal dialog that reports a failed operation, shown on top of everything else
#[derive(PartialEq)]
struct ErrorDialog(String);

impl Component for ErrorDialog {
    fn render(&self) -> impl IntoElement {
        let state = use_consume::<AppState>();
        let close = move || {
            let mut message = state.error_message;
            message.set(None);
        };

        Popup::new()
            .width(Size::window_percent(60.))
            .on_close_request(move |_| close())
            .child(PopupTitle::new("Error".to_string()))
            .child(rect().padding(8.).child(widgets::text(self.0.clone(), 14.)))
            .child(PopupButtons::new().child(Button::new().on_press(move |_| close()).child("Ok")))
    }
}
