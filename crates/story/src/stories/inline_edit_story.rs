use gpui_kit::component::{
    Disableable as _, Sizable as _,
    inline_edit::{InlineEdit, InlineEditBlurBehavior, InlineEditState},
    v_flex,
};
use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px,
};

use crate::section;

pub struct InlineEditStory {
    name: Entity<InlineEditState>,
    small: Entity<InlineEditState>,
    borderless: Entity<InlineEditState>,
    centered: Entity<InlineEditState>,
    validated: Entity<InlineEditState>,
    gain: Entity<InlineEditState>,
    cancelling: Entity<InlineEditState>,
    empty: Entity<InlineEditState>,
    disabled: Entity<InlineEditState>,
    readonly: Entity<InlineEditState>,
}

impl super::Story for InlineEditStory {
    fn title() -> &'static str {
        "Inline edit"
    }

    fn description() -> &'static str {
        "A value that turns into a text field when clicked. Enter or leaving the field commits, and Escape drops the draft."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render> {
        cx.new(|cx| {
            let mut state =
                |value: &'static str| cx.new(|cx| InlineEditState::new(value, window, cx));
            let name = state("Forest ambience");
            let small = state("Small value");
            let borderless = state("Embedded in a row");
            let centered = state("Centered");
            let cancelling = state("Leaving the field drops the draft");
            let empty = state("");
            let disabled = state("Disabled");
            let readonly = state("Read-only");
            let validated = cx.new(|cx| {
                InlineEditState::new("Required value", window, cx).validate(|value| {
                    if value.trim().is_empty() {
                        Err("Enter a name.".into())
                    } else {
                        Ok(())
                    }
                })
            });
            let gain = cx.new(|cx| {
                InlineEditState::new("3.0", window, cx).normalize(|draft| {
                    let value: f32 = draft.trim().parse().ok()?;
                    Some(format!("{:.1}", value.clamp(-24., 24.)).into())
                })
            });
            Self {
                gain,
                name,
                small,
                borderless,
                centered,
                validated,
                cancelling,
                empty,
                disabled,
                readonly,
            }
        })
    }
}

impl Render for InlineEditStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let gain = format!("{} dB", self.gain.read(cx).value());
        let frame = || div().w(px(280.));
        v_flex()
            .w_full()
            .gap_3()
            .child(section("Default").child(
                frame().child(InlineEdit::new("edit-name", &self.name).accessibility_label("Name")),
            ))
            .child(
                section("Small").child(
                    frame().child(
                        InlineEdit::new("edit-small", &self.small)
                            .accessibility_label("Small")
                            .small(),
                    ),
                ),
            )
            .child(
                section("Borderless").child(
                    frame().child(
                        InlineEdit::new("edit-borderless", &self.borderless)
                            .accessibility_label("Row name")
                            .borderless(true),
                    ),
                ),
            )
            .child(
                section("Centered").child(
                    frame().child(
                        InlineEdit::new("edit-centered", &self.centered)
                            .accessibility_label("Centered value")
                            .text_center(),
                    ),
                ),
            )
            .child(
                section("Validation").child(
                    frame().child(
                        InlineEdit::new("edit-validated", &self.validated)
                            .accessibility_label("Required name"),
                    ),
                ),
            )
            .child(
                section("Cancel on blur").child(
                    frame().child(
                        InlineEdit::new("edit-cancelling", &self.cancelling)
                            .accessibility_label("Draft")
                            .blur_behavior(InlineEditBlurBehavior::Cancel),
                    ),
                ),
            )
            .child(
                section("Number with a unit").child(
                    // Edited as a plain number, shown with its unit. Text that is not a
                    // number reverts quietly; a number is clamped and reformatted.
                    div().w(px(72.)).child(
                        InlineEdit::new("edit-gain", &self.gain)
                            .accessibility_label("Gain")
                            .display(gain)
                            .xsmall()
                            .borderless(true)
                            .text_right(),
                    ),
                ),
            )
            .child(
                section("Placeholder").child(
                    frame().child(
                        InlineEdit::new("edit-empty", &self.empty)
                            .accessibility_label("Description")
                            .placeholder("Add a description"),
                    ),
                ),
            )
            .child(
                section("Disabled").child(
                    frame().child(
                        InlineEdit::new("edit-disabled", &self.disabled)
                            .accessibility_label("Disabled value")
                            .disabled(true),
                    ),
                ),
            )
            .child(
                section("Read-only").child(
                    frame().child(
                        InlineEdit::new("edit-readonly", &self.readonly)
                            .accessibility_label("Read-only value")
                            .readonly(true),
                    ),
                ),
            )
    }
}
