use gpui::{
    App, AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Styled,
    Window, px,
};
use gpui_component::{ActiveTheme, ElevationLevel, StyledExt, h_flex, label::Label, v_flex};

use crate::section;

const DESCRIPTION: &str = "Surface / hairline / shadow triples for each elevation level, applied via `StyledExt::elevation()`. Switch the app's theme mode (View > Theme) to compare light and dark.";

pub struct ElevationStory {
    focus_handle: gpui::FocusHandle,
}

impl super::Story for ElevationStory {
    fn title() -> &'static str {
        "Elevation"
    }

    fn description() -> &'static str {
        "Surface, hairline, and shadow tokens for each elevation level."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render> {
        Self::view(window, cx)
    }
}

impl ElevationStory {
    pub fn view(_window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self {
            focus_handle: cx.focus_handle(),
        })
    }

    fn swatch(label: &'static str, level: ElevationLevel, cx: &App) -> impl IntoElement {
        v_flex()
            .items_center()
            .justify_center()
            .gap_2()
            .w(px(140.))
            .h(px(100.))
            .rounded(cx.theme().radius)
            .elevation(level, cx)
            .child(Label::new(label))
    }
}

impl Focusable for ElevationStory {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ElevationStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_6()
            .child(
                v_flex().gap_y_2().child("Elevation").child(
                    Label::new(DESCRIPTION)
                        .text_color(cx.theme().muted_foreground)
                        .text_sm(),
                ),
            )
            .child(
                section("Ladder").child(
                    h_flex()
                        .gap_4()
                        .p_6()
                        .rounded(cx.theme().radius)
                        .elevation(ElevationLevel::Base, cx)
                        .child(Self::swatch("Sunken", ElevationLevel::Sunken, cx))
                        .child(Self::swatch("Base", ElevationLevel::Base, cx))
                        .child(Self::swatch("Raised", ElevationLevel::Raised, cx))
                        .child(Self::swatch("Overlay", ElevationLevel::Overlay, cx)),
                ),
            )
    }
}
