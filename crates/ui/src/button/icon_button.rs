//! Background-free icon buttons and icon toggles.
//!
//! State lives in the icon color by default — no resting fill and no hover
//! pill. Opt-in [`.filled()`](IconButton::filled) paints a lit pill while
//! active. Cursor stays the default arrow.
//!
//! State priority: disabled > active or selected > dimmed > prominent > ordinary.

use std::rc::Rc;

use gpui::{
    App, ClickEvent, Corners, ElementId, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, StyleRefinement, Styled,
    Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Button as BaseButton, Toggle as BaseToggle};

use crate::ThemeStyled as _;
use crate::{
    ActiveTheme, Colorize as _, Disableable, Icon, Selectable, Sizable as _,
    Size, StyledExt,
    spinner::Spinner,
    tooltip::{ManagedTooltipExt, Tooltip},
};

/// Resolved (base, hover) icon colors for one state combination.
pub(crate) struct Tint {
    pub base: Hsla,
    pub hover: Hsla,
}

fn brighten(c: Hsla, amount: f32) -> Hsla {
    gpui::hsla(c.h, c.s, (c.l + amount).min(1.0), c.a)
}

pub(crate) fn tint(
    active: bool,
    prominent: bool,
    dimmed: bool,
    disabled: bool,
    active_color: Option<Hsla>,
    cx: &App,
) -> Tint {
    let t = cx.theme();
    if disabled {
        let c = t.muted_foreground.opacity(0.5);
        return Tint { base: c, hover: c };
    }
    if active {
        let base = active_color.unwrap_or(t.primary);
        return Tint {
            base,
            hover: brighten(base, 0.12),
        };
    }
    if dimmed {
        return Tint {
            base: t.muted_foreground.opacity(0.5),
            hover: t.muted_foreground,
        };
    }
    if prominent {
        return Tint {
            base: t.foreground.opacity(0.85),
            hover: t.foreground,
        };
    }
    Tint {
        base: t.muted_foreground,
        hover: t.foreground,
    }
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

struct IconChrome {
    tight: bool,
    filled: bool,
    fill_corners: Corners<bool>,
    tint: Tint,
    hover_feedback: bool,
    icon_size: Size,
    icon: Option<Icon>,
    loading: bool,
    label: Option<SharedString>,
    tooltip: Option<SharedString>,
    instance_style: StyleRefinement,
}

impl IconChrome {
    fn apply<E>(self, this: E, cx: &mut App) -> E
    where
        E: Styled
            + ParentElement
            + InteractiveElement
            + StatefulInteractiveElement
            + StyledExt
            + ManagedTooltipExt
            + FluentBuilder,
    {
        let filled = self.filled;
        let base = self.tint.base;
        let hover = self.tint.hover;
        let on_fill = cx.theme().primary_foreground;
        let pressed_fill = base.darken(0.08);
        let pressed_flash = cx.theme().foreground.opacity(0.2);
        let icon_size = self.icon_size;
        let fill_corners = self.fill_corners;
        let instance_style = self.instance_style.clone();

        this.cursor_default()
            .flex()
            .items_center()
            .justify_center()
            .gap_1()
            .map(|this| {
                if self.tight {
                    this.h(px(20.)).px(px(4.))
                } else {
                    this.h(px(28.)).min_w(px(28.)).px(px(2.))
                }
            })
            .text_xs()
            .map(|this| {
                if filled {
                    let r = px(4.);
                    this.bg(base)
                        .text_color(on_fill)
                        .when(fill_corners.top_left, |d| d.rounded_tl(r))
                        .when(fill_corners.top_right, |d| d.rounded_tr(r))
                        .when(fill_corners.bottom_right, |d| d.rounded_br(r))
                        .when(fill_corners.bottom_left, |d| d.rounded_bl(r))
                } else {
                    this.text_color(base)
                }
            })
            .refine_style(&instance_style)
            .when(self.hover_feedback, |this| {
                if filled {
                    this.hover(move |s| s.bg(hover))
                        .active(move |s| s.bg(pressed_fill))
                } else {
                    this.hover(move |s| s.text_color(hover))
                        .active(move |s| s.text_color(hover).bg(pressed_flash).rounded(px(4.)))
                }
            })
            .when_some(self.tooltip, |this, tooltip| {
                this.managed_tooltip(move |window, cx| {
                    Tooltip::new(tooltip.clone())
                        .overlay_anchored()
                        .build(window, cx)
                })
            })
            .when_some(self.icon.filter(|_| !self.loading), |this, icon| {
                this.child(icon.with_size(icon_size))
            })
            .when(self.loading, |this| this.child(Spinner::new()))
            .when_some(self.label, |this, label| {
                this.child(div().flex_none().font_semibold().child(label))
            })
    }
}

fn all_corners() -> Corners<bool> {
    Corners {
        top_left: true,
        top_right: true,
        bottom_right: true,
        bottom_left: true,
    }
}

fn accessibility_label(
    tooltip: Option<&SharedString>,
    label: Option<&SharedString>,
) -> Option<SharedString> {
    tooltip.cloned().or_else(|| label.cloned())
}

fn keyed_focus_handle(id: &ElementId, window: &mut Window, cx: &mut App) -> gpui::FocusHandle {
    window
        .use_keyed_state(id.clone(), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone()
}

/// A momentary, background-free icon (or icon+label) button.
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: Option<Icon>,
    label: Option<SharedString>,
    on_click: Option<ClickHandler>,
    active_color: Option<Hsla>,
    dimmed: bool,
    disabled: bool,
    loading: bool,
    selected: bool,
    hoverable: bool,
    tight: bool,
    icon_size: Option<Size>,
    filled: bool,
    fill_corners: Corners<bool>,
    tooltip: Option<SharedString>,
    prominent: bool,
    style: StyleRefinement,
}

impl IconButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            label: None,
            on_click: None,
            active_color: None,
            dimmed: false,
            disabled: false,
            loading: false,
            selected: false,
            hoverable: false,
            tight: false,
            icon_size: None,
            filled: false,
            fill_corners: all_corners(),
            tooltip: None,
            prominent: false,
            style: StyleRefinement::default(),
        }
    }

    /// The button's icon (tinted by state via `currentColor`).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// A text label, rendered after the icon (or alone).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Present but not ready — kept clickable so the click can explain why.
    pub fn dimmed(mut self, dimmed: bool) -> Self {
        self.dimmed = dimmed;
        self
    }

    /// Bright inactive tier: near-full foreground, full on hover.
    pub fn prominent(mut self) -> Self {
        self.prominent = true;
        self
    }

    /// Swap the icon for the shared spinner while a slow action runs.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Show hover feedback even without an `on_click` — for triggers whose
    /// click is owned by a wrapper (e.g. a dropdown popover).
    pub fn hoverable(mut self, hoverable: bool) -> Self {
        self.hoverable = hoverable;
        self
    }

    /// Hug the content instead of reserving the standalone 28px square.
    pub fn tight(mut self) -> Self {
        self.tight = true;
        self
    }

    /// Override the forced 16px (`Size::Medium`) icon size.
    pub fn icon_size(mut self, size: Size) -> Self {
        self.icon_size = Some(size);
        self
    }

    /// In the active state, paint a filled lit pill instead of only tinting
    /// the icon.
    pub fn filled(mut self) -> Self {
        self.filled = true;
        self
    }

    /// Round only the right corners of the filled background.
    pub fn fill_round_right(mut self) -> Self {
        self.fill_corners = Corners {
            top_left: false,
            top_right: true,
            bottom_right: true,
            bottom_left: false,
        };
        self
    }

    /// Draw the filled background square (no rounded corners).
    pub fn fill_square(mut self) -> Self {
        self.fill_corners = Corners {
            top_left: false,
            top_right: false,
            bottom_right: false,
            bottom_left: false,
        };
        self
    }

    /// Set the hover tooltip. Icon-only controls also use this as the
    /// accessibility label.
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Override the active-state icon color (default `theme.primary`).
    pub fn active_color(mut self, color: Hsla) -> Self {
        self.active_color = Some(color);
        self
    }

    fn chrome(&self, cx: &App) -> IconChrome {
        let active = self.selected;
        let tint = tint(
            active,
            self.prominent,
            self.dimmed,
            self.disabled,
            self.active_color,
            cx,
        );
        let clickable = !self.disabled && !self.loading && self.on_click.is_some();
        let hover_feedback = clickable || (self.hoverable && !self.disabled && !self.loading);
        IconChrome {
            tight: self.tight,
            filled: self.filled && active,
            fill_corners: self.fill_corners,
            tint,
            hover_feedback,
            icon_size: self.icon_size.unwrap_or(Size::Medium),
            icon: self.icon.clone(),
            loading: self.loading,
            label: self.label.clone(),
            tooltip: self.tooltip.clone(),
            instance_style: self.style.clone(),
        }
    }
}

impl Disableable for IconButton {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// `Selectable` lights the trigger while a popover is open; maps onto the
/// active tint and does not set `aria-toggled`.
impl Selectable for IconButton {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl Styled for IconButton {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for IconButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.disabled;
        let loading = self.loading;
        let instance_style = self.style.clone();
        let focus_handle = keyed_focus_handle(&self.id, window, cx);
        let is_focused = focus_handle.is_focused(window);
        let accessibility_label = accessibility_label(self.tooltip.as_ref(), self.label.as_ref());
        let chrome = self.chrome(cx);

        chrome
            .apply(
                BaseButton::new(self.id)
                    .selected(self.selected)
                    .disabled(disabled || loading)
                    .track_focus(&focus_handle)
                    .when_some(accessibility_label, |this, label| {
                        this.accessibility_label(label)
                    })
                    .when_some(
                        self.on_click.filter(|_| !disabled && !loading),
                        |this, on_click| {
                            this.on_click(move |event, window, cx| on_click(event, window, cx))
                        },
                    )
                    .when(!disabled, |this| {
                        this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            if loading {
                                cx.stop_propagation();
                                return;
                            }
                            window.prevent_default();
                            crate::global_state::GlobalState::suppress_text_selection(cx);
                        })
                    })
                    .styles(|styles| {
                        styles
                            .selected(|style| style.refine_style(&instance_style))
                            .disabled(|style| style.refine_style(&instance_style))
                    }),
                cx,
            )
            .when(is_focused, |this| this.focus_ring_style(window, cx))
            .render(window, cx)
    }
}

/// A stateful, background-free icon toggle: active = primary-tinted icon.
#[derive(IntoElement)]
pub struct IconToggle {
    id: ElementId,
    icon: Option<Icon>,
    label: Option<SharedString>,
    on_click: Option<ClickHandler>,
    active: bool,
    active_color: Option<Hsla>,
    disabled: bool,
    selected: bool,
    tight: bool,
    icon_size: Option<Size>,
    filled: bool,
    fill_corners: Corners<bool>,
    tooltip: Option<SharedString>,
    prominent: bool,
    style: StyleRefinement,
}

impl IconToggle {
    pub fn new(id: impl Into<ElementId>, active: bool) -> Self {
        Self {
            id: id.into(),
            icon: None,
            label: None,
            on_click: None,
            active,
            active_color: None,
            disabled: false,
            selected: false,
            tight: false,
            icon_size: None,
            filled: false,
            fill_corners: all_corners(),
            tooltip: None,
            prominent: false,
            style: StyleRefinement::default(),
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// See [`IconButton::tight`].
    pub fn tight(mut self) -> Self {
        self.tight = true;
        self
    }

    /// See [`IconButton::prominent`].
    pub fn prominent(mut self) -> Self {
        self.prominent = true;
        self
    }

    /// See [`IconButton::icon_size`].
    pub fn icon_size(mut self, size: Size) -> Self {
        self.icon_size = Some(size);
        self
    }

    /// See [`IconButton::filled`].
    pub fn filled(mut self) -> Self {
        self.filled = true;
        self
    }

    /// See [`IconButton::fill_round_right`].
    pub fn fill_round_right(mut self) -> Self {
        self.fill_corners = Corners {
            top_left: false,
            top_right: true,
            bottom_right: true,
            bottom_left: false,
        };
        self
    }

    /// See [`IconButton::fill_square`].
    pub fn fill_square(mut self) -> Self {
        self.fill_corners = Corners {
            top_left: false,
            top_right: false,
            bottom_right: false,
            bottom_left: false,
        };
        self
    }

    /// See [`IconButton::active_color`].
    pub fn active_color(mut self, color: Hsla) -> Self {
        self.active_color = Some(color);
        self
    }

    /// Set the hover tooltip. See [`IconButton::tooltip`].
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    fn visually_active(&self) -> bool {
        self.active || self.selected
    }

    fn chrome(&self, cx: &App) -> IconChrome {
        let active = self.visually_active();
        let tint = tint(
            active,
            self.prominent,
            false,
            self.disabled,
            self.active_color,
            cx,
        );
        IconChrome {
            tight: self.tight,
            filled: self.filled && active,
            fill_corners: self.fill_corners,
            tint,
            hover_feedback: !self.disabled && self.on_click.is_some(),
            icon_size: self.icon_size.unwrap_or(Size::Medium),
            icon: self.icon.clone(),
            loading: false,
            label: self.label.clone(),
            tooltip: self.tooltip.clone(),
            instance_style: self.style.clone(),
        }
    }
}

impl Disableable for IconToggle {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Selectable for IconToggle {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl Styled for IconToggle {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for IconToggle {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.disabled;
        let instance_style = self.style.clone();
        let focus_handle = keyed_focus_handle(&self.id, window, cx);
        let is_focused = focus_handle.is_focused(window);
        let accessibility_label = accessibility_label(self.tooltip.as_ref(), self.label.as_ref());
        let chrome = self.chrome(cx);

        chrome
            .apply(
                BaseToggle::new(self.id)
                    .pressed(self.active)
                    .disabled(disabled)
                    .track_focus(&focus_handle)
                    .when_some(accessibility_label, |this, label| {
                        this.accessibility_label(label)
                    })
                    .when_some(self.on_click.filter(|_| !disabled), |this, on_click| {
                        this.on_change(move |_next, event, window, cx| on_click(event, window, cx))
                    })
                    .when(!disabled, |this| {
                        this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            window.prevent_default();
                            crate::global_state::GlobalState::suppress_text_selection(cx);
                        })
                    })
                    .styles(|styles| {
                        styles
                            .pressed(|style| style.refine_style(&instance_style))
                            .disabled(|style| style.refine_style(&instance_style))
                    }),
                cx,
            )
            .when(is_focused, |this| this.focus_ring_style(window, cx))
            .render(window, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::h_flex;
    use gpui::{
        Context, Element as _, IntoElement, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, Render,
        Role, TestAppContext, VisualTestContext, accesskit, canvas, point, px,
    };
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
        sync::{Arc, Mutex},
    };

    #[gpui::test]
    fn tint_tiers_follow_spec(cx: &mut TestAppContext) {
        cx.update(|cx| {
            crate::init(cx);
            let (primary, muted, fg) = {
                let t = cx.theme();
                (t.primary, t.muted_foreground, t.foreground)
            };

            let n = tint(false, false, false, false, None, cx);
            assert_eq!(n.base, muted);
            assert_eq!(n.hover, fg);

            let a = tint(true, false, false, false, None, cx);
            assert_eq!(a.base, primary);
            assert!(a.hover.l > primary.l, "hover brightens, same hue");
            assert_eq!(a.hover.h, primary.h);

            let danger = cx.theme().danger;
            let ac = tint(true, false, false, false, Some(danger), cx);
            assert_eq!(ac.base, danger);
            assert!(ac.hover.l >= danger.l, "hover brightens override, same hue");
            assert_eq!(ac.hover.h, danger.h);

            let d = tint(false, false, true, false, None, cx);
            assert_eq!(d.base, muted.opacity(0.5));
            assert_eq!(d.hover, muted);

            let x = tint(false, false, false, true, None, cx);
            assert_eq!(x.base, muted.opacity(0.5));
            assert_eq!(x.hover, x.base);

            let p = tint(false, true, false, false, None, cx);
            assert_eq!(p.base, fg.opacity(0.85));
            assert_eq!(p.hover, fg);

            let pa = tint(true, true, false, false, None, cx);
            assert_eq!(pa.base, primary);

            let pd = tint(false, true, true, false, None, cx);
            assert_eq!(pd.base, muted.opacity(0.5));

            let px_ = tint(false, true, false, true, None, cx);
            assert_eq!(px_.base, muted.opacity(0.5));
            assert_eq!(px_.hover, px_.base);
        });
    }

    #[test]
    fn tooltip_builder_records_text_for_icon_button_and_toggle() {
        let button = IconButton::new("copy").tooltip("Copy");
        assert_eq!(button.tooltip.as_ref().map(|s| s.as_ref()), Some("Copy"));

        let toggle = IconToggle::new("loop", true).tooltip("Loop");
        assert_eq!(toggle.tooltip.as_ref().map(|s| s.as_ref()), Some("Loop"));
    }

    #[test]
    fn instance_style_priority_in_semantic_states() {
        let selected = IconButton::new("selected").selected(true).opacity(0.37);
        assert_eq!(selected.style.opacity, Some(0.37));

        let disabled = IconButton::new("disabled").disabled(true).opacity(0.37);
        assert_eq!(disabled.style.opacity, Some(0.37));

        let pressed = IconToggle::new("pressed", true).opacity(0.37);
        assert_eq!(pressed.style.opacity, Some(0.37));
    }

    struct ButtonHarness {
        disabled: bool,
        loading: bool,
        clicks: Rc<Cell<usize>>,
        parent_clicks: Rc<Cell<usize>>,
        keyboard_clicks: Rc<Cell<usize>>,
    }

    impl Render for ButtonHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.clicks.clone();
            let parent_clicks = self.parent_clicks.clone();
            let keyboard_clicks = self.keyboard_clicks.clone();
            h_flex()
                .id("icon-button-parent")
                .tab_group()
                .size(px(100.))
                .on_click(move |_, _, _| parent_clicks.set(parent_clicks.get() + 1))
                .child(
                    IconButton::new("icon-button")
                        .disabled(self.disabled)
                        .loading(self.loading)
                        .size_full()
                        .on_click(move |event, _, _| {
                            clicks.set(clicks.get() + 1);
                            if matches!(event, ClickEvent::Keyboard(_)) {
                                keyboard_clicks.set(keyboard_clicks.get() + 1);
                            }
                        }),
                )
        }
    }

    fn button_harness(
        cx: &mut TestAppContext,
        disabled: bool,
        loading: bool,
    ) -> (
        &mut VisualTestContext,
        Rc<Cell<usize>>,
        Rc<Cell<usize>>,
        Rc<Cell<usize>>,
    ) {
        cx.update(crate::init);
        let clicks = Rc::new(Cell::new(0));
        let parent_clicks = Rc::new(Cell::new(0));
        let keyboard_clicks = Rc::new(Cell::new(0));
        let (_, cx) = cx.add_window_view({
            let clicks = clicks.clone();
            let parent_clicks = parent_clicks.clone();
            let keyboard_clicks = keyboard_clicks.clone();
            move |_, _| ButtonHarness {
                disabled,
                loading,
                clicks,
                parent_clicks,
                keyboard_clicks,
            }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        (cx, clicks, parent_clicks, keyboard_clicks)
    }

    fn activate_key(cx: &mut VisualTestContext, key: &str) {
        let keystroke = Keystroke::parse(key).unwrap();
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
    }

    #[gpui::test]
    fn button_pointer_activation_fires_once(cx: &mut TestAppContext) {
        let (cx, clicks, _, keyboard_clicks) = button_harness(cx, false, false);
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
        assert_eq!(clicks.get(), 1);
        assert_eq!(keyboard_clicks.get(), 0);
    }

    #[gpui::test]
    fn button_enter_and_space_fire_once(cx: &mut TestAppContext) {
        let (cx, clicks, _, keyboard_clicks) = button_harness(cx, false, false);
        cx.update(|window, cx| window.focus_next(cx));
        cx.update(|window, cx| {
            assert!(window.focused(cx).is_some());
            window.draw(cx).clear(cx);
        });
        activate_key(cx, "enter");
        activate_key(cx, "space");
        assert_eq!(clicks.get(), 2);
        assert_eq!(keyboard_clicks.get(), 2);
    }

    #[gpui::test]
    fn disabled_button_is_inert_and_blocks_parent(cx: &mut TestAppContext) {
        let (cx, clicks, parent_clicks, _) = button_harness(cx, true, false);
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
        cx.update(|window, cx| window.focus_next(cx));
        cx.simulate_keystrokes("enter space");
        assert_eq!(clicks.get(), 0);
        assert_eq!(parent_clicks.get(), 0);
    }

    #[gpui::test]
    fn loading_button_is_inert_without_disabled_tint(cx: &mut TestAppContext) {
        cx.update(|cx| {
            crate::init(cx);
            let ordinary = tint(false, false, false, false, None, cx);
            let disabled = tint(false, false, false, true, None, cx);
            assert_ne!(ordinary.base, disabled.base);
            let loading = IconButton::new("loading").loading(true);
            assert!(loading.loading);
            assert!(!loading.disabled);
            assert_eq!(
                tint(
                    loading.selected,
                    loading.prominent,
                    loading.dimmed,
                    loading.disabled,
                    loading.active_color,
                    cx
                )
                .base,
                ordinary.base
            );
        });

        let (cx, clicks, parent_clicks, _) = button_harness(cx, false, true);
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
        cx.update(|window, cx| window.focus_next(cx));
        cx.simulate_keystrokes("enter space");
        assert_eq!(clicks.get(), 0);
        assert_eq!(parent_clicks.get(), 0);
    }

    struct ToggleHarness {
        pressed: bool,
        disabled: bool,
        clicks: Rc<RefCell<Vec<bool>>>,
    }

    impl Render for ToggleHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.clicks.clone();
            let pressed = self.pressed;
            h_flex()
                .id("icon-toggle-parent")
                .tab_group()
                .size(px(100.))
                .child(
                    IconToggle::new("icon-toggle", pressed)
                        .disabled(self.disabled)
                        .size_full()
                        .on_click(move |_, _, _| clicks.borrow_mut().push(!pressed)),
                )
        }
    }

    fn toggle_harness(
        cx: &mut TestAppContext,
        pressed: bool,
        disabled: bool,
    ) -> (&mut VisualTestContext, Rc<RefCell<Vec<bool>>>) {
        cx.update(crate::init);
        let clicks = Rc::new(RefCell::new(Vec::new()));
        let (_, cx) = cx.add_window_view({
            let clicks = clicks.clone();
            move |_, _| ToggleHarness {
                pressed,
                disabled,
                clicks,
            }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        (cx, clicks)
    }

    #[gpui::test]
    fn toggle_requests_inverse_controlled_state_once(cx: &mut TestAppContext) {
        for (pressed, expected) in [(false, true), (true, false)] {
            let (cx, clicks) = toggle_harness(cx, pressed, false);
            cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
            assert_eq!(clicks.borrow().as_slice(), &[expected]);
        }
    }

    #[gpui::test]
    fn disabled_toggle_is_inert(cx: &mut TestAppContext) {
        let (cx, clicks) = toggle_harness(cx, false, true);
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
        cx.update(|window, cx| window.focus_next(cx));
        cx.simulate_keystrokes("enter space");
        assert!(clicks.borrow().is_empty());
    }

    type Captured = Arc<Mutex<Option<Vec<accesskit::Node>>>>;

    struct A11yProbe {
        captured: Captured,
    }

    impl Render for A11yProbe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let captured = self.captured.clone();
            canvas(
                move |_, window, cx| {
                    fn info(el: impl IntoElement) -> accesskit::Node {
                        let mut node = accesskit::Node::new(Role::Button);
                        el.into_element().write_a11y_info(&mut node);
                        node
                    }
                    let nodes = vec![
                        info(
                            IconButton::new("selected")
                                .selected(true)
                                .tooltip("Open menu")
                                .render(window, cx),
                        ),
                        info(
                            IconToggle::new("pressed", true)
                                .tooltip("Loop")
                                .render(window, cx),
                        ),
                        info(
                            IconToggle::new("released", false)
                                .tooltip("Loop")
                                .render(window, cx),
                        ),
                        info(
                            IconButton::new("tooltip-only")
                                .tooltip("Copy")
                                .render(window, cx),
                        ),
                        info(IconButton::new("label-only").label("EQ").render(window, cx)),
                        info(
                            IconButton::new("tooltip-and-label")
                                .label("EQ")
                                .tooltip("Equalizer")
                                .render(window, cx),
                        ),
                        info(
                            IconToggle::new("selected-open", false)
                                .selected(true)
                                .tooltip("Spot")
                                .render(window, cx),
                        ),
                    ];
                    *captured.lock().unwrap() = Some(nodes);
                },
                |_, _, _, _| {},
            )
        }
    }

    fn a11y_nodes(cx: &mut TestAppContext) -> Vec<accesskit::Node> {
        cx.update(crate::init);
        let captured: Captured = Arc::new(Mutex::new(None));
        let result = captured.clone();
        let (_, cx) = cx.add_window_view(move |_, _| A11yProbe { captured });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        result.lock().unwrap().take().unwrap()
    }

    #[gpui::test]
    fn selected_presentation_does_not_set_aria_toggled(cx: &mut TestAppContext) {
        let nodes = a11y_nodes(cx);
        assert_eq!(nodes[0].role(), Role::Button);
        assert_eq!(nodes[0].toggled(), None);
    }

    #[gpui::test]
    fn toggle_pressed_accessibility(cx: &mut TestAppContext) {
        let nodes = a11y_nodes(cx);
        assert_eq!(nodes[1].role(), Role::Button);
        assert_eq!(nodes[1].toggled(), Some(accesskit::Toggled::True));
        assert_eq!(nodes[2].toggled(), Some(accesskit::Toggled::False));
    }

    #[gpui::test]
    fn selected_toggle_does_not_set_aria_toggled(cx: &mut TestAppContext) {
        let nodes = a11y_nodes(cx);
        assert_eq!(nodes[6].role(), Role::Button);
        assert_eq!(nodes[6].toggled(), Some(accesskit::Toggled::False));
    }

    #[gpui::test]
    fn selected_toggle_keeps_visual_active_tint(cx: &mut TestAppContext) {
        cx.update(|cx| {
            crate::init(cx);
            let primary = cx.theme().primary;
            let muted = cx.theme().muted_foreground;
            let selected = IconToggle::new("open", false).selected(true).filled();
            let chrome = selected.chrome(cx);
            assert_eq!(chrome.tint.base, primary);
            assert!(chrome.filled, "selected presentation stays visually active");

            let idle = IconToggle::new("idle", false).chrome(cx);
            assert_eq!(idle.tint.base, muted);
            assert!(!idle.filled);
        });
    }

    #[gpui::test]
    fn tooltip_or_visible_label_accessibility(cx: &mut TestAppContext) {
        let nodes = a11y_nodes(cx);
        assert_eq!(nodes[3].label(), Some("Copy"));
        assert_eq!(nodes[4].label(), Some("EQ"));
        assert_eq!(nodes[5].label(), Some("Equalizer"));
    }
}
