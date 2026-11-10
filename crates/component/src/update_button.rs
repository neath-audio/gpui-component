//! A compact outlined status and action control for software updates.

use std::rc::Rc;

use gpui::{
    App, ClickEvent, Corners, ElementId, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, Role, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled,
    Window, div, prelude::FluentBuilder as _, px,
};

use crate::{
    ActiveTheme as _, Disableable, Icon, IconName, Sizable, Size, StyledExt as _,
    button::{Button, ButtonCustomVariant, ButtonVariants as _},
    progress::ProgressCircle,
    spinner::Spinner,
};

type Click = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// The phase of an update, as the application knows it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UpdateState {
    /// Looking for an update.
    Checking,
    /// Fetching an update. `progress` runs from 0 to 1 when the amount is known.
    Downloading {
        /// How much has arrived, from 0 to 1.
        progress: Option<f32>,
    },
    /// Applying an update.
    Installing,
    /// An update is waiting for a restart.
    Ready,
    /// The update failed.
    Error,
}

impl UpdateState {
    /// Whether this phase only reports, taking no press.
    fn passive(self) -> bool {
        matches!(
            self,
            Self::Checking | Self::Downloading { .. } | Self::Installing
        )
    }

    fn label(self) -> &'static str {
        match self {
            Self::Checking => "Checking for updates…",
            Self::Downloading { .. } => "Downloading update…",
            Self::Installing => "Installing update…",
            Self::Ready => "Restart to update",
            Self::Error => "Failed to update",
        }
    }
}

/// Caller-owned update status. This component never downloads, installs or restarts anything.
/// Checking, downloading and installing only report; Ready and Error take a press.
#[derive(IntoElement)]
pub struct UpdateButton {
    id: ElementId,
    state: UpdateState,
    label: Option<SharedString>,
    tooltip: Option<SharedString>,
    version: Option<SharedString>,
    size: Size,
    compact: bool,
    disabled: bool,
    icon_size: Option<Size>,
    dismiss_icon_size: Size,
    action: Option<Click>,
    dismiss: Option<Click>,
    style: StyleRefinement,
}

impl UpdateButton {
    /// Create a button showing `state`.
    pub fn new(id: impl Into<ElementId>, state: UpdateState) -> Self {
        Self {
            id: id.into(),
            state,
            label: None,
            tooltip: None,
            version: None,
            size: Size::Small,
            compact: false,
            disabled: false,
            icon_size: None,
            dismiss_icon_size: Size::Size(px(10.)),
            action: None,
            dismiss: None,
            style: StyleRefinement::default(),
        }
    }

    /// Replace the phase's own words.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Help shown while the pointer rests on it, such as why the update failed. Without it the
    /// tooltip names the [`version`](Self::version).
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// The version on offer.
    pub fn version(mut self, version: impl Into<SharedString>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Tighten the padding.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Set the status icon's size, one step below the button's unless set.
    pub fn icon_size(mut self, size: impl Into<Size>) -> Self {
        self.icon_size = Some(size.into());
        self
    }

    /// Set the dismiss icon's size.
    pub fn dismiss_icon_size(mut self, size: impl Into<Size>) -> Self {
        self.dismiss_icon_size = size.into();
        self
    }

    /// Run this when Ready or Error is pressed.
    pub fn on_click(
        mut self,
        callback: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some(Rc::new(callback));
        self
    }

    /// Show a dismiss button in Ready and Error, and run this when it is pressed.
    pub fn on_dismiss(
        mut self,
        callback: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.dismiss = Some(Rc::new(callback));
        self
    }

    fn default_icon_size(&self) -> Size {
        match self.size {
            Size::Size(size) => Size::Size(size * 0.75),
            Size::XSmall | Size::Small => Size::XSmall,
            Size::Medium => Size::Small,
            Size::Large => Size::Medium,
        }
    }
}

impl Disableable for UpdateButton {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Sizable for UpdateButton {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl Styled for UpdateButton {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for UpdateButton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let passive = self.state.passive();
        let border = if passive || self.disabled {
            theme.border
        } else {
            theme.foreground.opacity(0.15)
        };
        let icon_color = if self.disabled {
            theme.muted_foreground
        } else {
            theme.foreground
        };
        let warning = theme.warning;
        let radius = theme.radius;
        let show_dismiss = !passive && self.dismiss.is_some();
        let label: SharedString = self
            .label
            .clone()
            .unwrap_or_else(|| self.state.label().into());
        let progress = match self.state {
            UpdateState::Downloading { progress } => progress
                .filter(|value| value.is_finite())
                .map(|value| value.clamp(0., 1.)),
            _ => None,
        };
        let tooltip = self.tooltip.clone().or_else(|| {
            self.version.as_ref().map(|version| {
                let words = match progress {
                    Some(value) => {
                        let percent = (value * 100.).round();
                        format!("Update to version {version} ({percent}% downloaded)")
                    }
                    None => format!("Update to version {version}"),
                };
                words.into()
            })
        });
        let icon_size = self.icon_size.unwrap_or_else(|| self.default_icon_size());
        let action = if self.disabled || passive {
            None
        } else {
            self.action.clone()
        };
        // A phase that only reports stays in full colour: passive is not disabled. It takes no
        // press, no hover and no tab stop, and its tooltip still answers the pointer.
        let inert = action.is_none() && !self.disabled;
        let inert_variant = ButtonCustomVariant::new(cx).foreground(icon_color);
        let button = Button::new((self.id.clone(), "action"))
            .label(label.clone())
            .when(cfg!(test), |button| {
                button.debug_selector(|| "update-action".into())
            })
            .when(inert, |button| button.custom(inert_variant).tab_stop(false))
            .when(!inert, |button| button.ghost())
            .with_size(self.size)
            .when(self.compact, |button| button.compact())
            .disabled(self.disabled)
            .content_style(StyleRefinement::default(), icon_size)
            .map(|button| match self.state {
                UpdateState::Checking | UpdateState::Installing => button.icon(
                    Spinner::new()
                        .icon(Icon::new(IconName::LoaderCircle))
                        .color(icon_color),
                ),
                UpdateState::Downloading { .. } if progress.is_some() => button.icon(
                    ProgressCircle::new((self.id.clone(), "progress"))
                        .value(progress.unwrap_or_default() * 100.)
                        .color(icon_color)
                        .accessibility_label("Update download"),
                ),
                UpdateState::Error => button.icon(
                    Icon::new(IconName::TriangleAlert).text_color(if self.disabled {
                        icon_color
                    } else {
                        warning
                    }),
                ),
                _ => button
                    .icon(Icon::new(gpui_kit_assets::IconName::Download).text_color(icon_color)),
            })
            .when(show_dismiss, |button| {
                button.border_corners(Corners {
                    top_left: true,
                    top_right: false,
                    bottom_left: true,
                    bottom_right: false,
                })
            })
            .when_some(tooltip, |button, tooltip| button.tooltip(tooltip))
            .when_some(action, |button, callback| {
                button.on_click(move |event, window, cx| callback(event, window, cx))
            });
        let dismiss_size = self.dismiss_icon_size;
        let dismiss_id = (self.id.clone(), "dismiss");
        let dismiss = self.dismiss.clone();
        let dismiss_button = show_dismiss.then(|| {
            Button::new(dismiss_id)
                .when(cfg!(test), |button| {
                    button.debug_selector(|| "update-dismiss".into())
                })
                .icon(Icon::new(IconName::Close))
                .accessibility_label("Dismiss update notification")
                .ghost()
                .with_size(self.size)
                .when(self.compact, |button| button.compact())
                .disabled(self.disabled)
                .content_style(StyleRefinement::default(), dismiss_size)
                .border_corners(Corners {
                    top_left: false,
                    top_right: true,
                    bottom_left: false,
                    bottom_right: true,
                })
                .on_click(move |event, window, cx| {
                    if let Some(dismiss) = &dismiss {
                        dismiss(event, window, cx);
                    }
                })
        });
        div()
            .id(self.id)
            .when(cfg!(test), |this| {
                this.debug_selector(|| "update-button".into())
            })
            .role(Role::Status)
            .aria_label(label)
            .min_w_0()
            .max_w_full()
            .flex_none()
            .self_center()
            .flex()
            .items_center()
            .rounded(radius)
            .border_1()
            .border_color(border)
            .child(button)
            .when_some(dismiss_button, |row, dismiss| {
                row.child(
                    div()
                        .flex_none()
                        .border_l_1()
                        .border_color(border)
                        .child(dismiss),
                )
            })
            .refine_style(&self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::{UpdateButton, UpdateState};
    use crate::{Disableable as _, Sizable as _, Size};
    use gpui::{
        Context, Element as _, IntoElement, Modifiers, Render, RenderOnce as _, Role,
        TestAppContext, VisualTestContext, Window, accesskit, div, prelude::*, px,
    };

    struct Probe {
        state: UpdateState,
        disabled: bool,
        clicks: u32,
        dismissals: u32,
        compact: bool,
        size: Size,
    }

    impl Render for Probe {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().w(px(360.)).p(px(20.)).flex().flex_col().child(
                UpdateButton::new("update", self.state)
                    .version("1.24.0")
                    .disabled(self.disabled)
                    .with_size(self.size)
                    .compact(self.compact)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.clicks += 1;
                        cx.notify();
                    }))
                    .on_dismiss(cx.listener(|this, _, _, cx| {
                        this.dismissals += 1;
                        cx.notify();
                    })),
            )
        }
    }

    fn mount(
        cx: &mut TestAppContext,
        state: UpdateState,
    ) -> (gpui::Entity<Probe>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, visual) = cx.add_window_view(|window, _| {
            window.activate_window();
            Probe {
                state,
                disabled: false,
                clicks: 0,
                dismissals: 0,
                compact: false,
                size: Size::Small,
            }
        });
        visual.simulate_resize(gpui::size(px(500.), px(200.)));
        draw(visual);
        (view, visual)
    }

    fn draw(visual: &mut VisualTestContext) {
        visual.update(|window, cx| window.draw(cx).clear(cx));
    }

    fn press(visual: &mut VisualTestContext, selector: &'static str) {
        let target = visual
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is drawn"));
        visual.simulate_click(target.center(), Modifiers::default());
        draw(visual);
    }

    fn has(visual: &mut VisualTestContext, selector: &'static str) -> bool {
        visual.debug_bounds(selector).is_some()
    }

    fn set_state(view: &gpui::Entity<Probe>, visual: &mut VisualTestContext, state: UpdateState) {
        visual.update(|_, cx| {
            view.update(cx, |view, cx| {
                view.state = state;
                cx.notify();
            })
        });
        draw(visual);
    }

    #[gpui::test]
    fn phases_that_report_take_no_press_and_ready_and_error_do(cx: &mut TestAppContext) {
        let (view, visual) = mount(cx, UpdateState::Checking);
        for state in [
            UpdateState::Checking,
            UpdateState::Downloading { progress: None },
            UpdateState::Downloading {
                progress: Some(0.4),
            },
            UpdateState::Installing,
        ] {
            set_state(&view, visual, state);
            assert!(!has(visual, "update-dismiss"), "{state:?} has no dismiss");
            press(visual, "update-action");
            visual.update(|_, cx| assert_eq!(view.read(cx).clicks, 0, "{state:?}"));
        }
        for (index, state) in [UpdateState::Ready, UpdateState::Error]
            .into_iter()
            .enumerate()
        {
            set_state(&view, visual, state);
            press(visual, "update-action");
            press(visual, "update-dismiss");
            visual.update(|_, cx| {
                assert_eq!(view.read(cx).clicks, index as u32 + 1, "{state:?}");
                assert_eq!(view.read(cx).dismissals, index as u32 + 1, "{state:?}");
            });
        }
    }

    #[gpui::test]
    fn a_disabled_button_takes_no_press_at_all(cx: &mut TestAppContext) {
        let (view, visual) = mount(cx, UpdateState::Ready);
        visual.update(|_, cx| {
            view.update(cx, |view, cx| {
                view.disabled = true;
                cx.notify();
            })
        });
        draw(visual);
        press(visual, "update-action");
        press(visual, "update-dismiss");
        visual.update(|_, cx| {
            assert_eq!(view.read(cx).clicks, 0);
            assert_eq!(view.read(cx).dismissals, 0);
        });
    }

    #[gpui::test]
    fn the_item_announces_its_words_as_a_status(cx: &mut TestAppContext) {
        struct StatusProbe;

        impl Render for StatusProbe {
            fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                for (state, words) in [
                    (UpdateState::Checking, "Checking for updates…"),
                    (
                        UpdateState::Downloading { progress: None },
                        "Downloading update…",
                    ),
                    (UpdateState::Installing, "Installing update…"),
                    (UpdateState::Ready, "Restart to update"),
                    (UpdateState::Error, "Failed to update"),
                ] {
                    let item = UpdateButton::new("update", state)
                        .render(window, cx)
                        .into_element();
                    assert_eq!(item.a11y_role(), Some(Role::Status), "{state:?}");
                    let mut node = accesskit::Node::new(Role::Status);
                    item.write_a11y_info(&mut node);
                    assert_eq!(node.label(), Some(words), "{state:?}");
                }
                div()
            }
        }

        cx.update(crate::init);
        let (_, visual) = cx.add_window_view(|_, _| StatusProbe);
        draw(visual);
    }

    #[gpui::test]
    fn compact_keeps_the_button_from_growing(cx: &mut TestAppContext) {
        let (view, visual) = mount(cx, UpdateState::Ready);
        let regular = visual.debug_bounds("update-button").unwrap();
        visual.update(|_, cx| {
            view.update(cx, |view, cx| {
                view.compact = true;
                cx.notify();
            })
        });
        draw(visual);
        let compact = visual.debug_bounds("update-button").unwrap();
        assert!(compact.size.width <= regular.size.width);
        assert!(compact.size.height <= regular.size.height);
    }
}
