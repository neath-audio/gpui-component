use gpui_kit::component::{
    ActiveTheme as _, h_flex,
    sticky::{Sticky, StickyArea},
    v_flex,
};
use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, ScrollHandle, Styled,
    Window, div, px,
};

use crate::section;

pub struct StickyStory {
    scroll: ScrollHandle,
}

impl super::Story for StickyStory {
    fn title() -> &'static str {
        "Sticky"
    }

    fn description() -> &'static str {
        "Headers that move with their content, pin to the top of the scroll area, and are pushed away by the next one."
    }

    fn new_view(_: &mut Window, cx: &mut App) -> Entity<impl Render> {
        cx.new(|_| Self {
            scroll: ScrollHandle::new(),
        })
    }
}

impl Render for StickyStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let border = cx.theme().border;
        let header = |id: &'static str, title: &'static str| {
            Sticky::new(id)
                .px_3()
                .border_b_1()
                .border_color(border)
                .child(title)
        };
        let rows = |prefix: &'static str| {
            (0..12).map(move |index| {
                h_flex()
                    .h_8()
                    .px_3()
                    .flex_shrink_0()
                    .child(format!("{prefix} {}", index + 1))
            })
        };
        v_flex().w_full().gap_3().child(
            section("Sections").child(
                div()
                    .w(px(360.))
                    .border_1()
                    .border_color(border)
                    .rounded(cx.theme().radius)
                    .child(
                        StickyArea::new("sticky-story", &self.scroll)
                            .h(px(280.))
                            .sticky(header("sticky-ambience", "Ambience"))
                            .children(rows("Ambience"))
                            .sticky(header("sticky-foley", "Foley"))
                            .children(rows("Foley"))
                            .sticky(header("sticky-music", "Music"))
                            .children(rows("Music")),
                    ),
            ),
        )
    }
}
