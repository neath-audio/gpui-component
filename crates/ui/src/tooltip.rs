use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::{
    Action, AnyElement, AnyView, App, AppContext, Bounds, Context, Display, Element, ElementId,
    GlobalElementId, Half, InspectorElementId, IntoElement, LayoutId, MouseDownEvent,
    ParentElement, Pixels, Point, Position, Render, ScrollWheelEvent, SharedString, Size,
    StatefulInteractiveElement, Style, StyleRefinement, Styled, Task, Window, canvas, deferred,
    div, point, prelude::FluentBuilder, px,
};

use crate::{
    ActiveTheme, StyledExt,
    animation::{Transition, ease_in_out_cubic, ease_out_cubic},
    geometry::Placement,
    global_state::GlobalState,
    h_flex,
    kbd::Kbd,
    root::Root,
    text::Text,
};

pub(crate) fn init(_cx: &mut App) {
    // No app-level init needed — TooltipOverlay is per-window via Root.
}

// ── Tooltip view (unchanged API) ────────────────────────────────────────────

enum TooltipContext {
    Text(Text),
    Element(Box<dyn Fn(&mut Window, &mut App) -> AnyElement>),
}

/// A Tooltip element that can display text or custom content,
/// with optional key binding information.
pub struct Tooltip {
    style: StyleRefinement,
    content: TooltipContext,
    key_binding: Option<Kbd>,
    action: Option<(Box<dyn Action>, Option<SharedString>)>,
    overlay_anchored: bool,
}

impl Tooltip {
    /// Create a Tooltip with a text content.
    pub fn new(text: impl Into<Text>) -> Self {
        Self {
            style: StyleRefinement::default(),
            content: TooltipContext::Text(text.into()),
            key_binding: None,
            action: None,
            overlay_anchored: false,
        }
    }

    /// Mark this tooltip as rendered by the managed [`TooltipOverlay`], which
    /// anchors and gaps the bubble itself (`ANCHOR_GAP`). The bubble then
    /// carries NO outer margin: the overlay positions against the measured
    /// element box, and a margin makes the painted bubble disagree with that
    /// box (the tooltip-covers-trigger bug, 2026-07-21 — margin semantics in
    /// the layout engine shifted under the gpui pin bump). Builders passed to
    /// [`ManagedTooltipExt::managed_tooltip`] MUST set this; the native
    /// mouse-anchored `.tooltip()` path keeps the default margin for cursor
    /// clearance.
    pub fn overlay_anchored(mut self) -> Self {
        self.overlay_anchored = true;
        self
    }

    /// Create a Tooltip with a custom element.
    pub fn element<E, F>(builder: F) -> Self
    where
        E: IntoElement,
        F: Fn(&mut Window, &mut App) -> E + 'static,
    {
        Self {
            style: StyleRefinement::default(),
            key_binding: None,
            action: None,
            content: TooltipContext::Element(Box::new(move |window, cx| {
                builder(window, cx).into_any_element()
            })),
            overlay_anchored: false,
        }
    }

    /// Set Action to display key binding information for the tooltip if it exists.
    pub fn action(mut self, action: &dyn Action, context: Option<&str>) -> Self {
        self.action = Some((action.boxed_clone(), context.map(SharedString::new)));
        self
    }

    /// Set KeyBinding information for the tooltip.
    pub fn key_binding(mut self, key_binding: Option<Kbd>) -> Self {
        self.key_binding = key_binding;
        self
    }

    /// Build the tooltip and return it as an `AnyView`.
    pub fn build(self, _: &mut Window, cx: &mut App) -> AnyView {
        cx.new(|_| self).into()
    }
}

impl FluentBuilder for Tooltip {}
impl Styled for Tooltip {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl Render for Tooltip {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let key_binding = if let Some(key_binding) = &self.key_binding {
            Some(key_binding.clone())
        } else {
            if let Some((action, context)) = &self.action {
                Kbd::binding_for_action(
                    action.as_ref(),
                    context.as_ref().map(|s| s.as_ref()),
                    window,
                )
            } else {
                None
            }
        };

        div().child(
            // Wrap in a child, to ensure the left margin is applied to the tooltip
            h_flex()
                .font_family(cx.theme().font_family.clone())
                // Overlay-anchored bubbles are margin-free: the overlay
                // positioner owns the trigger gap (`ANCHOR_GAP`) and measures
                // the element box — an outer margin here would make the
                // painted bubble disagree with the measured box. The native
                // mouse-anchored path keeps the margin as cursor clearance.
                .when(!self.overlay_anchored, |this| this.m_3())
                .bg(cx.theme().popover)
                .border_1()
                .border_color(cx.theme().hairline_strong)
                .shadow(cx.theme().shadow_2().into_vec())
                .text_color(cx.theme().popover_foreground)
                .rounded(px(6.))
                .justify_between()
                .py_0p5()
                .px_2()
                .text_xs()
                .gap_3()
                .refine_style(&self.style)
                .map(|this| {
                    this.child(div().map(|this| match self.content {
                        TooltipContext::Text(ref text) => this.child(text.clone()),
                        TooltipContext::Element(ref builder) => this.child(builder(window, cx)),
                    }))
                })
                .when_some(key_binding, |this, kbd| {
                    this.child(
                        div()
                            .text_xs()
                            .flex_shrink_0()
                            .text_color(cx.theme().muted_foreground)
                            .child(kbd.appearance(false)),
                    )
                }),
        )
    }
}

// ── Managed tooltip system ──────────────────────────────────────────────────

/// Grace period: if a tooltip was hidden within this time, skip delay for next show.
const GRACE_PERIOD: Duration = Duration::from_millis(300);
/// Delay before showing a tooltip when no tooltip is currently active.
const SHOW_DELAY: Duration = Duration::from_millis(500);
/// Duration of the slide-down enter animation.
const ENTER_DURATION: Duration = Duration::from_millis(150);
/// Duration of the position-slide animation when switching tooltips.
const SLIDE_DURATION: Duration = Duration::from_millis(200);
const TOOLTIP_WINDOW_MARGIN: Pixels = px(4.);
/// Gap between the trigger edge and the bubble. Owned by the POSITIONER —
/// never by a margin on the bubble view, which would desynchronize the
/// painted bubble from the measured box (see `Tooltip::overlay_anchored`).
const ANCHOR_GAP: Pixels = px(12.);

#[derive(Clone, Copy, Debug, PartialEq)]
struct TooltipOverlayPosition {
    bounds: Bounds<Pixels>,
    placement: Placement,
}

fn tooltip_overlay_position(
    trigger_bounds: Bounds<Pixels>,
    tooltip_size: Size<Pixels>,
    viewport_size: Size<Pixels>,
    margin: Pixels,
    preferred_placement: Option<Placement>,
) -> TooltipOverlayPosition {
    let right_limit = (viewport_size.width - margin).max(margin);
    let bottom_limit = (viewport_size.height - margin).max(margin);
    let available_left = (trigger_bounds.left() - margin).max(px(0.));
    let available_right = (right_limit - trigger_bounds.right()).max(px(0.));
    let available_above = (trigger_bounds.top() - margin).max(px(0.));
    let available_below = (bottom_limit - trigger_bounds.bottom()).max(px(0.));

    // The bubble is margin-free (`Tooltip::overlay_anchored`); the positioner
    // owns the trigger gap, so the gap counts toward the space a side needs.
    let needed_width = tooltip_size.width + ANCHOR_GAP;
    let needed_height = tooltip_size.height + ANCHOR_GAP;

    let placement = match preferred_placement {
        Some(Placement::Right) if needed_width <= available_right => Placement::Right,
        Some(Placement::Right) if needed_width <= available_left => Placement::Left,
        Some(Placement::Right) if available_right >= available_left => Placement::Right,
        Some(Placement::Right) => Placement::Left,
        Some(Placement::Left) if needed_width <= available_left => Placement::Left,
        Some(Placement::Left) if needed_width <= available_right => Placement::Right,
        Some(Placement::Left) if available_left >= available_right => Placement::Left,
        Some(Placement::Left) => Placement::Right,
        Some(Placement::Bottom) if needed_height <= available_below => Placement::Bottom,
        Some(Placement::Bottom) if needed_height <= available_above => Placement::Top,
        Some(Placement::Bottom) if available_below >= available_above => Placement::Bottom,
        Some(Placement::Bottom) => Placement::Top,
        Some(Placement::Top) | None if needed_height <= available_above => Placement::Top,
        Some(Placement::Top) | None if needed_height <= available_below => Placement::Bottom,
        Some(Placement::Top) | None if available_below >= available_above => Placement::Bottom,
        Some(Placement::Top) | None => Placement::Top,
    };

    let centered_x = trigger_bounds.center().x - tooltip_size.width.half();
    let centered_y = trigger_bounds.center().y - tooltip_size.height.half();
    let origin = match placement {
        Placement::Top => point(
            centered_x,
            trigger_bounds.top() - tooltip_size.height - ANCHOR_GAP,
        ),
        Placement::Bottom => point(centered_x, trigger_bounds.bottom() + ANCHOR_GAP),
        Placement::Left => point(
            trigger_bounds.left() - tooltip_size.width - ANCHOR_GAP,
            centered_y,
        ),
        Placement::Right => point(trigger_bounds.right() + ANCHOR_GAP, centered_y),
    };
    let bounds = Bounds::new(origin, tooltip_size);

    TooltipOverlayPosition {
        bounds: clamp_tooltip_bounds(bounds, viewport_size, margin),
        placement,
    }
}

fn clamp_tooltip_bounds(
    mut bounds: Bounds<Pixels>,
    viewport_size: Size<Pixels>,
    margin: Pixels,
) -> Bounds<Pixels> {
    let right_limit = (viewport_size.width - margin).max(margin);
    let bottom_limit = (viewport_size.height - margin).max(margin);

    if bounds.right() > right_limit {
        bounds.origin.x -= bounds.right() - right_limit;
    }
    if bounds.left() < margin {
        bounds.origin.x = margin;
    }

    if bounds.bottom() > bottom_limit {
        bounds.origin.y -= bounds.bottom() - bottom_limit;
    }
    if bounds.top() < margin {
        bounds.origin.y = margin;
    }

    bounds
}

struct TooltipOverlayPositioner {
    /// Live cell refreshed by the trigger's prepaint — read fresh in THIS
    /// element's prepaint each frame, so the bubble tracks a trigger that
    /// reflows while the tooltip is visible (or between hover and the
    /// delayed show).
    trigger_bounds: Rc<Cell<Bounds<Pixels>>>,
    preferred_placement: Option<Placement>,
    children: Vec<AnyElement>,
}

struct TooltipOverlayPositionerState {
    child_layout_ids: Vec<LayoutId>,
}

fn tooltip_overlay_positioner(
    trigger_bounds: Rc<Cell<Bounds<Pixels>>>,
    preferred_placement: Option<Placement>,
) -> TooltipOverlayPositioner {
    TooltipOverlayPositioner {
        trigger_bounds,
        preferred_placement,
        children: Vec::new(),
    }
}

impl ParentElement for TooltipOverlayPositioner {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Element for TooltipOverlayPositioner {
    type RequestLayoutState = TooltipOverlayPositionerState;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let child_layout_ids = self
            .children
            .iter_mut()
            .map(|child| child.request_layout(window, cx))
            .collect::<Vec<_>>();

        let layout_id = window.request_layout(
            Style {
                position: Position::Absolute,
                display: Display::Flex,
                ..Style::default()
            },
            child_layout_ids.iter().copied(),
            cx,
        );

        (
            layout_id,
            TooltipOverlayPositionerState { child_layout_ids },
        )
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if request_layout.child_layout_ids.is_empty() {
            return;
        }

        let mut child_min: Point<Pixels> = point(Pixels::MAX, Pixels::MAX);
        let mut child_max = Point::default();
        for child_layout_id in &request_layout.child_layout_ids {
            let child_bounds = window.layout_bounds(*child_layout_id);
            child_min = child_min.min(&child_bounds.origin);
            child_max = child_max.max(&child_bounds.bottom_right());
        }

        let tooltip_size: Size<Pixels> = (child_max - child_min).into();
        let client_inset = window.client_inset().unwrap_or(px(0.));
        let tooltip_position = tooltip_overlay_position(
            self.trigger_bounds.get(),
            tooltip_size,
            window.viewport_size(),
            TOOLTIP_WINDOW_MARGIN + client_inset,
            self.preferred_placement,
        );

        let offset = tooltip_position.bounds.origin - bounds.origin;
        let offset = point(offset.x.round(), offset.y.round());

        window.with_element_offset(offset, |window| {
            for child in &mut self.children {
                child.prepaint(window, cx);
            }
        });
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        for child in &mut self.children {
            child.paint(window, cx);
        }
    }
}

impl IntoElement for TooltipOverlayPositioner {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// Content for a managed tooltip.
///
/// `trigger_bounds` is the LIVE cell the trigger refreshes on every prepaint
/// (see [`ManagedTooltipExt::managed_tooltip`]) — never a copied snapshot.
/// The show path may fire up to `SHOW_DELAY` after hover-enter, and browse
/// headers reflow asynchronously in that window; positioning against a
/// frozen copy painted the bubble where the trigger USED to be (covering
/// the moved control — user report 2026-07-21).
#[derive(Clone)]
pub(crate) struct TooltipContent {
    pub build: Rc<dyn Fn(&mut Window, &mut App) -> AnyView>,
    pub trigger_bounds: Rc<Cell<Bounds<Pixels>>>,
    pub preferred_placement: Option<Placement>,
}

/// Manages tooltip lifecycle: delay, grace period, animations, and rendering.
///
/// A single instance lives in [`Root`] per window. Components register hover
/// via [`ManagedTooltipExt::managed_tooltip`] which calls into this overlay.
pub struct TooltipOverlay {
    content: Option<TooltipContent>,
    prev_trigger_bounds: Option<Bounds<Pixels>>,
    epoch: usize,
    had_recent_tooltip: bool,
    animation_epoch: usize,
    is_switching: bool,

    _show_task: Option<Task<()>>,
    _hide_task: Option<Task<()>>,
}

impl TooltipOverlay {
    pub fn new() -> Self {
        Self {
            content: None,
            prev_trigger_bounds: None,
            epoch: 0,
            had_recent_tooltip: false,
            animation_epoch: 0,
            is_switching: false,
            _show_task: None,
            _hide_task: None,
        }
    }

    fn next_epoch(&mut self) -> usize {
        self.epoch += 1;
        self.epoch
    }

    /// Request showing a tooltip. If another tooltip is active or was recently
    /// hidden, shows immediately with a slide animation. Otherwise starts a delay.
    pub(crate) fn request_show(
        &mut self,
        content: TooltipContent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // No tooltip may appear while a menu is open (a menu owns focus
        // exactly while open). Without this, a trigger that stays hovered
        // after opening a menu — or any trigger hovered while one is up —
        // would draw its tooltip on top of the menu.
        if GlobalState::global(cx).is_menu_focused(window, cx) {
            return;
        }

        // Cancel any pending hide
        self._hide_task = None;

        let was_visible = self.content.is_some();
        let in_grace = self.had_recent_tooltip;

        if was_visible || in_grace {
            // Switch: show immediately with slide animation. The previous
            // bounds are snapshotted (`.get()`) — only the ACTIVE tooltip
            // tracks its trigger live.
            self.prev_trigger_bounds = self.content.as_ref().map(|c| c.trigger_bounds.get());
            self.content = Some(content);
            self._show_task = None;
            self.is_switching = was_visible;
            self.animation_epoch += 1;
            cx.notify();
        } else {
            // New: delay then show with slideDown
            let epoch = self.next_epoch();
            let content = content.clone();
            self._show_task = Some(cx.spawn_in(window, async move |this, cx| {
                cx.background_executor().timer(SHOW_DELAY).await;
                let _ = this.update_in(cx, |this, _, cx| {
                    if this.epoch != epoch {
                        return;
                    }

                    this.content = Some(content);
                    this.prev_trigger_bounds = None;
                    this.is_switching = false;
                    this.animation_epoch += 1;
                    cx.notify();
                });
            }));
        }
    }

    /// Request hiding the current tooltip. Starts a brief grace period so that
    /// moving to another tooltip-bearing element feels instant.
    pub(crate) fn request_hide(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Cancel any pending show
        self._show_task = None;

        if self.content.is_none() {
            return;
        }

        let epoch = self.next_epoch();
        self.had_recent_tooltip = true;

        self._hide_task = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(GRACE_PERIOD).await;
            let _ = this.update_in(cx, |this, _, cx| {
                if this.epoch != epoch {
                    return;
                }
                this.content = None;
                this.prev_trigger_bounds = None;
                this.had_recent_tooltip = false;
                cx.notify();
            });
        }));
    }

    pub(crate) fn hide(&mut self, cx: &mut Context<Self>) {
        if self.clear_state() {
            cx.notify();
        }
    }

    fn clear_state(&mut self) -> bool {
        let changed = self.content.is_some()
            || self.prev_trigger_bounds.is_some()
            || self.had_recent_tooltip
            || self.is_switching
            || self._show_task.is_some()
            || self._hide_task.is_some();

        self.content = None;
        self.prev_trigger_bounds = None;
        self.had_recent_tooltip = false;
        self.is_switching = false;
        self._show_task = None;
        self._hide_task = None;

        changed
    }
}

impl Render for TooltipOverlay {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Native parity: gpui's built-in tooltips clear on any mouse press or
        // scroll anywhere in the window. A zero-size canvas re-registers the
        // window-level listeners every frame — unconditionally, so a pending
        // (delayed) show is cancelled by a press as well.
        let overlay = cx.entity();
        let dismisser = canvas(
            |_, _, _| (),
            move |_, _, window, _| {
                window.on_mouse_event({
                    let overlay = overlay.clone();
                    move |_: &MouseDownEvent, _, _, cx| {
                        overlay.update(cx, |overlay: &mut TooltipOverlay, cx| overlay.hide(cx));
                    }
                });
                window.on_mouse_event({
                    let overlay = overlay.clone();
                    move |_: &ScrollWheelEvent, _, _, cx| {
                        overlay.update(cx, |overlay: &mut TooltipOverlay, cx| overlay.hide(cx));
                    }
                });
            },
        )
        .absolute()
        .size_0();
        let root = div().child(dismisser);

        let Some(content) = self.content.as_ref() else {
            return root.into_any_element();
        };

        // Level guard for menus opened without a press (e.g. keyboard): an
        // already-visible tooltip must not stay painted over an open menu.
        // The content is kept — it resumes (or is hidden by the normal
        // hover-leave path) once the menu closes.
        if GlobalState::global(cx).is_menu_focused(window, cx) {
            return root.into_any_element();
        }

        let content_view = (content.build)(window, cx);
        // Live cell for the positioner (re-read every prepaint, so the
        // bubble tracks a trigger that reflows while visible); a `.get()`
        // snapshot only for the slide-animation math below.
        let trigger_bounds_cell = content.trigger_bounds.clone();
        let trigger_bounds = content.trigger_bounds.get();
        let preferred_placement = content.preferred_placement;
        let animation_epoch = self.animation_epoch;
        let is_switching = self.is_switching;
        let prev_trigger_bounds = self.prev_trigger_bounds;

        root.child(
            deferred(
                tooltip_overlay_positioner(trigger_bounds_cell, preferred_placement).child(
                    div().child(content_view).map(|el| {
                        if is_switching {
                            let Some(prev_bounds) = prev_trigger_bounds else {
                                return el.into_any_element();
                            };

                            let is_same_y =
                                (trigger_bounds.origin.y - prev_bounds.origin.y).abs() < px(10.);
                            if !is_same_y {
                                // If the new trigger is at a different Y level, don't slide horizontally
                                // to avoid weird diagonal movement. (We could consider sliding vertically
                                // in this case, but it might be less visually clear.)
                                return el.into_any_element();
                            }

                            let dx = trigger_bounds.center().x - prev_bounds.center().x;

                            Transition::new(SLIDE_DURATION)
                                .ease(ease_in_out_cubic)
                                .slide_x(-dx, px(0.))
                                .apply(
                                    el,
                                    ElementId::NamedInteger(
                                        "tooltip-slide".into(),
                                        animation_epoch as u64,
                                    ),
                                )
                                .into_any_element()
                        } else {
                            // New tooltip: slideDown + fadeIn
                            Transition::new(ENTER_DURATION)
                                .ease(ease_out_cubic)
                                .slide_y(px(4.), px(0.))
                                .fade(0.0, 1.0)
                                .apply(
                                    el,
                                    ElementId::NamedInteger(
                                        "tooltip-enter".into(),
                                        animation_epoch as u64,
                                    ),
                                )
                                .into_any_element()
                        }
                    }),
                ),
            )
            .with_priority(2),
        )
        .into_any_element()
    }
}

// ── Extension trait for managed tooltips ─────────────────────────────────────

// ── Shared tooltip state for components ─────────────────────────────────────

/// Shared tooltip state that components (Button, Switch, Checkbox, Radio, etc.)
/// can embed to get `.tooltip()` support with minimal boilerplate.
#[derive(Default)]
pub(crate) struct ComponentTooltip {
    pub text: Option<(
        SharedString,
        Option<(Rc<Box<dyn Action>>, Option<SharedString>)>,
    )>,
    pub builder: Option<Rc<dyn Fn(&mut Window, &mut App) -> AnyView>>,
}

impl ComponentTooltip {
    /// Apply this tooltip to a `Stateful<Div>` (or any `ManagedTooltipExt` element).
    pub fn apply<E: ManagedTooltipExt>(self, el: E) -> E {
        if let Some(builder) = self.builder {
            el.managed_tooltip(move |window, cx| builder(window, cx))
        } else if let Some((text, action)) = self.text {
            el.managed_tooltip(move |window, cx| {
                Tooltip::new(text.clone())
                    .overlay_anchored()
                    .when_some(action.clone(), |this, (action, context)| {
                        this.action(
                            action.boxed_clone().as_ref(),
                            context.as_ref().map(|c| c.as_ref()),
                        )
                    })
                    .build(window, cx)
            })
        } else {
            el
        }
    }
}

// ── Managed tooltip extension trait ──────────────────────────────────────────

/// Extension trait to attach a managed tooltip to any stateful element.
///
/// Managed tooltips are rendered by the per-window [`TooltipOverlay`] owned by
/// [`Root`]: they are anchored to the trigger element (above, flipping below
/// near the window edge), show after a short delay with an enter animation,
/// and switch instantly between adjacent triggers within a grace period.
///
/// This is the same mechanism used by built-in components such as
/// [`Button`](crate::button::Button); use it to give custom widgets
/// consistent tooltip behavior.
pub trait ManagedTooltipExt: StatefulInteractiveElement + crate::ElementExt + Sized {
    /// Show a managed tooltip built by `build_tooltip` while this element is hovered.
    fn managed_tooltip(
        self,
        build_tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static,
    ) -> Self {
        self.managed_tooltip_with_placement(None, build_tooltip)
    }

    fn managed_tooltip_at(
        self,
        placement: Placement,
        build_tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static,
    ) -> Self {
        self.managed_tooltip_with_placement(Some(placement), build_tooltip)
    }

    fn managed_tooltip_with_placement(
        self,
        preferred_placement: Option<Placement>,
        build_tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static,
    ) -> Self {
        let build_tooltip = Rc::new(build_tooltip);
        let trigger_bounds_cell: Rc<Cell<Bounds<Pixels>>> = Rc::new(Cell::new(Bounds::default()));
        let bounds_writer = trigger_bounds_cell.clone();

        self.on_prepaint(move |bounds, _, _| {
            bounds_writer.set(bounds);
        })
        .on_hover({
            let trigger_bounds_cell = trigger_bounds_cell.clone();
            let build_tooltip = build_tooltip.clone();
            move |hovered, window, cx| {
                if let Some(overlay) = Root::tooltip_overlay(window, cx) {
                    if *hovered {
                        overlay.update(cx, |o: &mut TooltipOverlay, cx| {
                            o.request_show(
                                TooltipContent {
                                    build: build_tooltip.clone(),
                                    // The live cell itself — the overlay and
                                    // positioner re-read it each frame, so
                                    // the bubble tracks reflows instead of a
                                    // hover-time snapshot.
                                    trigger_bounds: trigger_bounds_cell.clone(),
                                    preferred_placement,
                                },
                                window,
                                cx,
                            );
                        });
                    } else {
                        overlay.update(cx, |o: &mut TooltipOverlay, cx| {
                            o.request_hide(window, cx);
                        });
                    }
                }
            }
        })
        // No press handler here: the overlay itself hides on any mouse press
        // or scroll anywhere in the window (see `TooltipOverlay::render`),
        // which subsumes the old left-down-on-trigger dismissal.
    }
}

impl<E: StatefulInteractiveElement + crate::ElementExt> ManagedTooltipExt for E {}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{
        Entity, Focusable as _, InteractiveElement as _, MouseButton, MouseMoveEvent, Render,
        ScrollDelta, TestAppContext, VisualTestContext, size,
    };

    fn test_content(bounds: Bounds<Pixels>) -> TooltipContent {
        TooltipContent {
            build: Rc::new(|window, cx| Tooltip::new("Test tooltip").build(window, cx)),
            trigger_bounds: Rc::new(Cell::new(bounds)),
            preferred_placement: None,
        }
    }

    fn test_bounds(x: f32, y: f32, width: f32, height: f32) -> Bounds<Pixels> {
        Bounds::new(point(px(x), px(y)), size(px(width), px(height)))
    }

    fn test_size(width: f32, height: f32) -> Size<Pixels> {
        size(px(width), px(height))
    }

    #[test]
    fn tooltip_overlay_clear_state_resets_active_tooltip() {
        let mut overlay = TooltipOverlay::new();

        overlay.content = Some(test_content(test_bounds(10., 10., 40., 20.)));
        overlay.prev_trigger_bounds = Some(test_bounds(0., 0., 40., 20.));
        overlay.had_recent_tooltip = true;
        overlay.is_switching = true;
        overlay._show_task = Some(Task::ready(()));

        assert!(overlay.clear_state());
        assert!(overlay.content.is_none());
        assert!(overlay.prev_trigger_bounds.is_none());
        assert!(!overlay.had_recent_tooltip);
        assert!(!overlay.is_switching);
        assert!(overlay._show_task.is_none());
        assert!(overlay._hide_task.is_none());
    }

    #[test]
    fn tooltip_overlay_position_prefers_above_when_space_allows() {
        let trigger_bounds = test_bounds(100., 80., 80., 24.);
        let position = tooltip_overlay_position(
            trigger_bounds,
            test_size(120., 30.),
            test_size(300., 200.),
            TOOLTIP_WINDOW_MARGIN,
            None,
        );

        assert_eq!(position.placement, Placement::Top);
        assert_eq!(position.bounds.origin.x, px(80.));
        assert_eq!(position.bounds.origin.y, px(50.) - ANCHOR_GAP);
        assert_eq!(position.bounds.bottom(), trigger_bounds.top() - ANCHOR_GAP);
    }

    #[test]
    fn tooltip_overlay_position_flips_below_near_top_edge() {
        let trigger_bounds = test_bounds(24., 4., 120., 32.);
        let position = tooltip_overlay_position(
            trigger_bounds,
            test_size(240., 32.),
            test_size(520., 260.),
            TOOLTIP_WINDOW_MARGIN,
            None,
        );

        assert_eq!(position.placement, Placement::Bottom);
        assert_eq!(position.bounds.top(), trigger_bounds.bottom() + ANCHOR_GAP);
        assert!(position.bounds.top() >= trigger_bounds.bottom());
    }

    #[test]
    fn tooltip_overlay_position_clamps_horizontal_edges() {
        let trigger_bounds = test_bounds(4., 80., 24., 24.);
        let position = tooltip_overlay_position(
            trigger_bounds,
            test_size(120., 30.),
            test_size(300., 200.),
            TOOLTIP_WINDOW_MARGIN,
            None,
        );

        assert_eq!(position.placement, Placement::Top);
        assert_eq!(position.bounds.left(), TOOLTIP_WINDOW_MARGIN);
    }

    #[test]
    fn tooltip_overlay_position_uses_larger_side_when_neither_side_fits() {
        let trigger_bounds = test_bounds(120., 20., 40., 20.);
        let position = tooltip_overlay_position(
            trigger_bounds,
            test_size(160., 120.),
            test_size(300., 100.),
            TOOLTIP_WINDOW_MARGIN,
            None,
        );

        assert_eq!(position.placement, Placement::Bottom);
        assert_eq!(position.bounds.top(), TOOLTIP_WINDOW_MARGIN);
        assert_eq!(position.bounds.left(), px(60.));
    }

    // ── Dismissal parity with gpui's native tooltips ────────────────────────

    struct TriggerView;

    impl Render for TriggerView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().child(
                div()
                    .id("trigger")
                    .w(px(60.))
                    .h(px(60.))
                    .managed_tooltip(|window, cx| {
                        Tooltip::new("tip").overlay_anchored().build(window, cx)
                    }),
            )
        }
    }

    fn setup(cx: &mut TestAppContext) -> (Entity<TooltipOverlay>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (root, cx) = cx.add_window_view(|window, cx| {
            let view = cx.new(|_| TriggerView);
            Root::new(view, window, cx).bordered(false)
        });
        let overlay = root.read_with(cx, |root, _| root.tooltip_overlay.clone());
        (overlay, cx)
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| {
            _ = window.draw(cx);
        });
    }

    fn hover_trigger(cx: &mut VisualTestContext) {
        cx.simulate_event(MouseMoveEvent {
            position: point(px(30.), px(30.)),
            pressed_button: None,
            modifiers: Default::default(),
        });
    }

    fn right_mouse_down(cx: &mut VisualTestContext, x: f32, y: f32) {
        cx.simulate_event(MouseDownEvent {
            button: MouseButton::Right,
            position: point(px(x), px(y)),
            modifiers: Default::default(),
            click_count: 1,
            first_mouse: false,
        });
    }

    fn show_tooltip(overlay: &Entity<TooltipOverlay>, cx: &mut VisualTestContext) {
        draw(cx);
        hover_trigger(cx);
        overlay.read_with(cx, |o, _| assert!(o._show_task.is_some()));
        cx.executor()
            .advance_clock(SHOW_DELAY + Duration::from_millis(50));
        draw(cx);
        overlay.read_with(cx, |o, _| assert!(o.content.is_some()));
    }

    #[gpui::test]
    fn managed_tooltip_hides_on_any_mouse_down(cx: &mut TestAppContext) {
        let (overlay, cx) = setup(cx);
        show_tooltip(&overlay, cx);

        // A right press (e.g. opening a context menu) hides the tooltip
        // immediately — the trigger-local left-only handler this replaces
        // never saw it.
        right_mouse_down(cx, 30., 30.);
        overlay.read_with(cx, |o, _| {
            assert!(o.content.is_none());
            assert!(o._show_task.is_none());
        });
    }

    #[gpui::test]
    fn managed_tooltip_hides_on_scroll(cx: &mut TestAppContext) {
        let (overlay, cx) = setup(cx);
        show_tooltip(&overlay, cx);

        cx.simulate_event(ScrollWheelEvent {
            position: point(px(30.), px(30.)),
            delta: ScrollDelta::Pixels(point(px(0.), px(-10.))),
            ..Default::default()
        });
        overlay.read_with(cx, |o, _| assert!(o.content.is_none()));
    }

    #[gpui::test]
    fn managed_tooltip_press_cancels_pending_show(cx: &mut TestAppContext) {
        let (overlay, cx) = setup(cx);
        draw(cx);

        hover_trigger(cx);
        overlay.read_with(cx, |o, _| assert!(o._show_task.is_some()));

        // Press during the show delay: the delayed show must never fire
        // (otherwise it would pop over whatever the press opened).
        right_mouse_down(cx, 30., 30.);
        overlay.read_with(cx, |o, _| assert!(o._show_task.is_none()));

        cx.executor()
            .advance_clock(SHOW_DELAY + Duration::from_millis(50));
        draw(cx);
        overlay.read_with(cx, |o, _| assert!(o.content.is_none()));
    }

    #[gpui::test]
    fn request_show_is_suppressed_while_menu_focused(cx: &mut TestAppContext) {
        let (overlay, cx) = setup(cx);
        draw(cx);

        // An open menu owns focus; while it does, show requests are ignored.
        let menu = cx.update(|window, cx| {
            let menu = crate::menu::PopupMenu::build(window, cx, |menu, _, _| menu);
            menu.focus_handle(cx).focus(window, cx);
            menu
        });
        draw(cx);

        cx.update(|window, cx| {
            overlay.update(cx, |o, cx| {
                o.request_show(test_content(test_bounds(10., 10., 40., 20.)), window, cx);
            });
        });
        overlay.read_with(cx, |o, _| {
            assert!(o._show_task.is_none());
            assert!(o.content.is_none());
        });

        // Focus released (menu closed): shows work again.
        cx.update(|window, cx| {
            window.blur();
            overlay.update(cx, |o, cx| {
                o.request_show(test_content(test_bounds(10., 10., 40., 20.)), window, cx);
            });
        });
        overlay.read_with(cx, |o, _| assert!(o._show_task.is_some()));
        drop(menu);
    }

    #[test]
    fn tooltip_overlay_position_places_tooltip_to_the_right() {
        let trigger_bounds = test_bounds(20., 60., 32., 32.);
        let position = tooltip_overlay_position(
            trigger_bounds,
            test_size(120., 30.),
            test_size(300., 200.),
            TOOLTIP_WINDOW_MARGIN,
            Some(Placement::Right),
        );

        assert_eq!(position.placement, Placement::Right);
        assert_eq!(position.bounds.left(), trigger_bounds.right() + ANCHOR_GAP);
        assert_eq!(position.bounds.center().y, trigger_bounds.center().y);
    }

    #[test]
    fn tooltip_overlay_position_flips_left_near_right_edge() {
        let trigger_bounds = test_bounds(260., 60., 32., 32.);
        let position = tooltip_overlay_position(
            trigger_bounds,
            test_size(120., 30.),
            test_size(300., 200.),
            TOOLTIP_WINDOW_MARGIN,
            Some(Placement::Right),
        );

        assert_eq!(position.placement, Placement::Left);
        assert_eq!(position.bounds.right(), trigger_bounds.left() - ANCHOR_GAP);
    }

    #[test]
    fn tooltip_overlay_position_clamps_vertical_edges_for_right_placement() {
        let trigger_bounds = test_bounds(20., 2., 32., 20.);
        let position = tooltip_overlay_position(
            trigger_bounds,
            test_size(120., 40.),
            test_size(300., 200.),
            TOOLTIP_WINDOW_MARGIN,
            Some(Placement::Right),
        );

        assert_eq!(position.placement, Placement::Right);
        assert_eq!(position.bounds.top(), TOOLTIP_WINDOW_MARGIN);
        assert_eq!(position.bounds.left(), trigger_bounds.right() + ANCHOR_GAP);
    }
}
