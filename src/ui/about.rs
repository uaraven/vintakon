//! Help -> About dialog: application icon, name, version and copyright.

use freya::prelude::*;

use super::widgets::{colors, text};
use crate::{resources::APP_ICON, state::AppState};

const ICON_SIZE: f32 = 72.;

#[derive(PartialEq)]
pub struct AboutDialog;

impl Component for AboutDialog {
    fn render(&self) -> impl IntoElement {
        let state = use_consume::<AppState>();
        let close = move || state.close_about();

        Popup::new()
            .width(Size::px(380.))
            .height(Size::px(260.))
            .on_close_request(move |_| close())
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::fill())
                    .center()
                    .spacing(10.)
                    .child(
                        ImageViewer::new(("app_icon", APP_ICON))
                            .width(Size::px(ICON_SIZE))
                            .height(Size::px(ICON_SIZE)),
                    )
                    .child(text("Vintakon", 24.).font_weight(FontWeight::BOLD))
                    .child(
                        text(format!("Version {}", env!("CARGO_PKG_VERSION")), 14.)
                            .color(colors().text_secondary),
                    )
                    .child(text("©2025-2026 Oles Voronin", 14.))
                    .child(
                        rect()
                            .margin(Gaps::new(6., 0., 0., 0.))
                            .child(Button::new().on_press(move |_| close()).child("Close")),
                    ),
            )
    }
}
