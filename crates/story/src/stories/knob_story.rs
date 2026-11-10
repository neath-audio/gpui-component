use gpui_kit::component::Sizable as _;
use gpui_kit::component::{ActiveTheme as _, Disableable as _, Size, h_flex, knob::Knob, v_flex};
use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
};

use crate::section;

pub struct KnobStory {
    level: f32,
    frequency: f32,
    pan: f32,
    trim: f32,
}

impl super::Story for KnobStory {
    fn title() -> &'static str {
        "Knob"
    }

    fn description() -> &'static str {
        "A rotary dial for a numeric value. Drag vertically, hold Shift for fine control, and double-click to reset."
    }

    fn new_view(_: &mut Window, cx: &mut App) -> Entity<impl Render> {
        cx.new(|_| Self {
            level: 40.,
            frequency: 1000.,
            pan: 0.,
            trim: 3.,
        })
    }
}

impl Render for KnobStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let update = move |apply: fn(&mut Self, f32)| {
            let view = view.clone();
            move |value: f32, _: &mut Window, cx: &mut App| {
                view.update(cx, |this, cx| {
                    apply(this, value);
                    cx.notify();
                })
                .ok();
            }
        };

        v_flex()
            .w_full()
            .gap_3()
            .child(
                section("Sizes").child(
                    h_flex().gap_4().items_start().children(
                        [
                            (Size::XSmall, "knob-xs"),
                            (Size::Small, "knob-sm"),
                            (Size::Medium, "knob-md"),
                            (Size::Large, "knob-lg"),
                        ]
                        .map(|(size, id)| {
                            Knob::new(id)
                                .accessibility_label("Level")
                                .with_size(size)
                                .value(self.level)
                                .on_change(update(|this, value| this.level = value))
                        }),
                    ),
                ),
            )
            .child(
                section("Logarithmic").child(
                    Knob::new("knob-frequency")
                        .accessibility_label("Frequency")
                        .range(20. ..=20000.)
                        .step(0.)
                        .logarithmic(true)
                        .wheel_step(0.05)
                        .default_value(1000.)
                        .value(self.frequency)
                        .value_text(format!("{:.0} Hz", self.frequency))
                        .on_change(update(|this, value| this.frequency = value)),
                ),
            )
            .child(
                section("Bipolar").child(
                    Knob::new("knob-pan")
                        .accessibility_label("Pan")
                        .range(-50. ..=50.)
                        .bipolar(true)
                        .default_value(0.)
                        .value(self.pan)
                        .on_change(update(|this, value| this.pan = value)),
                ),
            )
            .child(
                section("Colors").child(
                    Knob::new("knob-color")
                        .accessibility_label("Trim")
                        .range(-12. ..=12.)
                        .bipolar_origin(0.)
                        .value(self.trim)
                        .text_color(cx.theme().green)
                        .on_change(update(|this, value| this.trim = value)),
                ),
            )
            .child(
                section("Disabled and read-only").child(
                    h_flex()
                        .gap_4()
                        .items_start()
                        .child(
                            Knob::new("knob-disabled")
                                .accessibility_label("Disabled")
                                .value(25.)
                                .disabled(true),
                        )
                        .child(
                            Knob::new("knob-readonly")
                                .accessibility_label("Read-only")
                                .value(75.)
                                .readonly(true),
                        ),
                ),
            )
    }
}
