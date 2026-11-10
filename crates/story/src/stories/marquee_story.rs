use gpui_kit::component::{ActiveTheme as _, marquee::Marquee, v_flex};
use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px,
};

use crate::section;

const LONG: &str = "Field recording of rain on a tin roof, Hokkaido, take twelve, final.wav";

pub struct MarqueeStory;

impl super::Story for MarqueeStory {
    fn title() -> &'static str {
        "Marquee"
    }

    fn description() -> &'static str {
        "One line of text that scrolls back and forth only when it is too long for its box."
    }

    fn new_view(_: &mut Window, cx: &mut App) -> Entity<impl Render> {
        cx.new(|_| Self)
    }
}

impl Render for MarqueeStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let frame = |cx: &App| {
            div()
                .w(px(240.))
                .p_2()
                .border_1()
                .border_color(cx.theme().border)
                .rounded(cx.theme().radius)
        };
        v_flex()
            .w_full()
            .gap_3()
            .child(section("Too long").child(frame(cx).child(Marquee::new("marquee-long", LONG))))
            .child(section("Fits").child(frame(cx).child(Marquee::new("marquee-fits", "Rain.wav"))))
            .child(
                section("Not animated")
                    .child(frame(cx).child(Marquee::new("marquee-still", LONG).animated(false))),
            )
            .child(
                section("Inherited text style").child(
                    frame(cx)
                        .text_color(cx.theme().muted_foreground)
                        .text_sm()
                        .child(Marquee::new("marquee-styled", LONG)),
                ),
            )
    }
}
