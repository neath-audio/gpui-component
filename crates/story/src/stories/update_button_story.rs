use gpui_kit::component::{
    Disableable as _, Sizable as _, Size, h_flex,
    update_button::{UpdateButton, UpdateState},
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
};

use crate::section;

pub struct UpdateButtonStory {
    presses: usize,
    dismissed: bool,
}

impl super::Story for UpdateButtonStory {
    fn title() -> &'static str {
        "Update button"
    }

    fn description() -> &'static str {
        "Reports an update's progress. It only shows the status the application gives it; Ready and Error take a press."
    }

    fn new_view(_: &mut Window, cx: &mut App) -> Entity<impl Render> {
        cx.new(|_| Self {
            presses: 0,
            dismissed: false,
        })
    }
}

impl Render for UpdateButtonStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let phases = [
            ("update-checking", UpdateState::Checking),
            (
                "update-downloading",
                UpdateState::Downloading { progress: None },
            ),
            (
                "update-progress",
                UpdateState::Downloading {
                    progress: Some(0.4),
                },
            ),
            ("update-installing", UpdateState::Installing),
        ];
        v_flex()
            .w_full()
            .gap_3()
            .child(section("Reporting").child(
                h_flex().gap_2().flex_wrap().children(
                    phases.map(|(id, state)| UpdateButton::new(id, state).version("0.9.0")),
                ),
            ))
            .child(
                section("Waiting for a press")
                    .description(format!("Pressed {} times.", self.presses))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                UpdateButton::new("update-ready", UpdateState::Ready)
                                    .version("0.9.0")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.presses += 1;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                UpdateButton::new("update-error", UpdateState::Error)
                                    .tooltip("The update server could not be reached.")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.presses += 1;
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .child(
                section("Dismissable").child(h_flex().gap_2().when(!self.dismissed, |row| {
                    row.child(
                        UpdateButton::new("update-dismissable", UpdateState::Ready)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.presses += 1;
                                cx.notify();
                            }))
                            .on_dismiss(cx.listener(|this, _, _, cx| {
                                this.dismissed = true;
                                cx.notify();
                            })),
                    )
                })),
            )
            .child(
                section("Sizes and states").child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            UpdateButton::new("update-medium", UpdateState::Ready)
                                .with_size(Size::Medium),
                        )
                        .child(
                            UpdateButton::new("update-compact", UpdateState::Ready).compact(true),
                        )
                        .child(
                            UpdateButton::new("update-disabled", UpdateState::Ready).disabled(true),
                        ),
                ),
            )
    }
}
