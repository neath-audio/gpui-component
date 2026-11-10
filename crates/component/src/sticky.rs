//! Headers that move with scroll content, then pin to the viewport's top edge.

use std::{cell::Cell, rc::Rc};

use gpui::{
    AbsoluteLength, AnyElement, App, AvailableSpace, Bounds, Element, ElementId, Empty,
    GlobalElementId, InspectorElementId, InteractiveElement as _, IntoElement, LayoutId,
    ParentElement, Pixels, Position, RenderOnce, ScrollHandle, StatefulInteractiveElement as _,
    Style, StyleRefinement, Styled, Window, div, point, prelude::FluentBuilder as _, rems, size,
};

use crate::{
    ActiveTheme as _, ElementExt as _, InteractiveElementExt as _, StyledExt as _,
    scroll::{ScrollableElement as _, ScrollbarAxis},
};

/// A header inserted into a [`StickyArea`] with [`StickyArea::sticky`].
///
/// The header occupies its normal place in the content. Once scrolled past, it pins to the top
/// inset; the next sticky header pushes it away. Its children are mounted only once. Styles
/// apply to the header, not the scroll viewport. Use [`Self::height`] to reserve its full height,
/// including padding and borders.
pub struct Sticky {
    id: ElementId,
    height: AbsoluteLength,
    offset: AbsoluteLength,
    enabled: bool,
    style: StyleRefinement,
    children: Vec<AnyElement>,
}

impl Sticky {
    /// Create a header 2.5 rems tall that pins to the top of the viewport.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            height: rems(2.5).into(),
            offset: rems(0.).into(),
            enabled: true,
            style: StyleRefinement::default(),
            children: Vec::new(),
        }
    }

    /// Reserved height, including padding and borders.
    pub fn height(mut self, height: impl Into<AbsoluteLength>) -> Self {
        self.height = height.into();
        self
    }

    /// Distance from the top of the viewport at which the header pins.
    pub fn offset(mut self, offset: impl Into<AbsoluteLength>) -> Self {
        self.offset = offset.into();
        self
    }

    /// Disable pinning while preserving the header's position in the content.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    fn header(
        self,
        cx: &App,
    ) -> (
        gpui::Stateful<gpui::Div>,
        AbsoluteLength,
        AbsoluteLength,
        bool,
    ) {
        let header = div()
            .id(self.id)
            .w_full()
            .flex()
            .items_center()
            .flex_shrink_0()
            .bg(cx.theme().background)
            .refine_style(&self.style)
            .h(self.height)
            .children(self.children);
        (header, self.height, self.offset, self.enabled)
    }

    fn into_flow(self, cx: &App) -> AnyElement {
        self.header(cx).0.into_any_element()
    }

    fn into_parts(
        self,
        handle: &ScrollHandle,
        window: &Window,
        cx: &App,
    ) -> (AnyElement, StickyLayer) {
        let slot_id = (self.id.clone(), "slot");
        let (header, height, offset, enabled) = self.header(cx);
        let rem_size = window.rem_size();
        let height = height.to_pixels(rem_size);
        let slot = Rc::new(Cell::new(Bounds::default()));
        let measured = slot.clone();
        let placeholder = div()
            .id(slot_id)
            .w_full()
            .h(height)
            .flex_shrink_0()
            .on_prepaint(move |bounds, _, _| measured.set(bounds))
            .into_any_element();
        (
            placeholder,
            StickyLayer {
                child: header.block_mouse_except_scroll().into_any_element(),
                handle: handle.clone(),
                slot,
                next: None,
                height,
                offset: offset.to_pixels(rem_size),
                enabled,
            },
        )
    }
}

impl Styled for Sticky {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Sticky {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// A scroll area whose content can hold [`Sticky`] headers.
///
/// ```ignore
/// StickyArea::new("documents", &self.scroll)
///     .h_72()
///     .child(introduction)
///     .sticky(Sticky::new("collection-header").child(header))
///     .children(documents)
/// ```
///
/// Keep the [`ScrollHandle`] in the view. The area scrolls vertically; use [`Self::axis`] for
/// two-axis content, which keeps its horizontal scrolling while headers pin vertically.
/// Horizontal-only areas render headers in flow.
///
/// Headers are positioned during prepaint from the same frame's clamped scroll offset, so there
/// is no render-time snapshot, polling or lag. They paint under the scrollbar, and the wheel
/// reaches the viewport through them. Headers paint after the content, so Tab reaches their
/// controls after the content's. This targets ordinary vertical sections; the children are not
/// virtualized.
#[derive(IntoElement)]
pub struct StickyArea {
    id: ElementId,
    handle: ScrollHandle,
    axis: ScrollbarAxis,
    style: StyleRefinement,
    children: Vec<AnyElement>,
    headers: Vec<(usize, Sticky)>,
}

impl StickyArea {
    /// Create a vertically scrolling area tracked by `handle`.
    pub fn new(id: impl Into<ElementId>, handle: &ScrollHandle) -> Self {
        Self {
            id: id.into(),
            handle: handle.clone(),
            axis: ScrollbarAxis::Vertical,
            style: StyleRefinement::default(),
            children: Vec::new(),
            headers: Vec::new(),
        }
    }

    /// Set the scroll axis, vertical by default.
    pub fn axis(mut self, axis: impl Into<ScrollbarAxis>) -> Self {
        self.axis = axis.into();
        self
    }

    /// Insert a sticky header at this point in the content flow. Following children and
    /// headers come after it.
    pub fn sticky(mut self, header: Sticky) -> Self {
        // Reserve the header's place in the flow; render swaps the slot in.
        self.headers.push((self.children.len(), header));
        self.children.push(Empty.into_any_element());
        self
    }
}

impl Styled for StickyArea {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for StickyArea {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for StickyArea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let axis = self.axis;
        let mut children = self.children;
        let mut layers: Vec<StickyLayer> = Vec::new();
        for (index, header) in self.headers {
            if !axis.has_vertical() {
                children[index] = header.into_flow(cx);
                continue;
            }
            let (slot, layer) = header.into_parts(&self.handle, window, cx);
            if let Some(previous) = layers.last_mut() {
                previous.next = Some(layer.slot.clone());
            }
            children[index] = slot;
            layers.push(layer);
        }
        let content = div()
            .id((self.id.clone(), "content"))
            .flex()
            .flex_col()
            .flex_none()
            .map(|this| match axis {
                ScrollbarAxis::Vertical => this.w_full().h_auto().min_h_full(),
                ScrollbarAxis::Horizontal => this.h_full().w_auto().min_w_full(),
                ScrollbarAxis::Both => this.size_auto().min_w_full().min_h_full(),
            })
            .children(children);
        let viewport = div()
            .id((self.id.clone(), "viewport"))
            .size_full()
            .flex()
            .track_scroll(&self.handle)
            .map(|this| match axis {
                ScrollbarAxis::Vertical => this.flex_col().overflow_y_scroll(),
                ScrollbarAxis::Horizontal => this.flex_row().overflow_x_scroll(),
                // Keep the content from stretching to the viewport's height, which would
                // leave nothing to scroll vertically.
                ScrollbarAxis::Both => this.items_start().overflow_scroll(),
            })
            .lock_scroll_axis()
            .child(content);
        div()
            .id(self.id)
            .relative()
            .size_full()
            .overflow_hidden()
            .refine_style(&self.style)
            .child(viewport)
            .children(layers)
            .scrollbar(&self.handle, axis)
    }
}

struct StickyLayer {
    child: AnyElement,
    handle: ScrollHandle,
    slot: Rc<Cell<Bounds<Pixels>>>,
    next: Option<Rc<Cell<Bounds<Pixels>>>>,
    height: Pixels,
    offset: Pixels,
    enabled: bool,
}

fn top(
    natural: Pixels,
    pin: Pixels,
    height: Pixels,
    next: Option<Pixels>,
    enabled: bool,
) -> Pixels {
    if !enabled {
        return natural;
    }
    let pinned = natural.max(pin);
    next.map_or(pinned, |next| pinned.min(next - height))
}

impl IntoElement for StickyLayer {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl Element for StickyLayer {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        // This layer is positioned outside flow; the real child is laid out
        // below using the slot's measured width, including horizontal overflow.
        (
            window.request_layout(
                Style {
                    position: Position::Absolute,
                    ..Default::default()
                },
                [],
                cx,
            ),
            (),
        )
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        // The viewport and all slots have prepainted first. Reading here sees
        // the scroll handle's clamped offset in this very frame.
        let slot = self.slot.get();
        let viewport = self.handle.bounds();
        let y = top(
            slot.top(),
            viewport.top() + self.offset,
            self.height,
            self.next.as_ref().map(|next| next.get().top()),
            self.enabled,
        );
        // A separate root preserves the measured content width and horizontal
        // scroll position in two-axis viewports. Absolute placement updates
        // paint and hitboxes together without duplicating the header subtree.
        self.child.prepaint_as_root(
            point(slot.left(), y),
            size(
                AvailableSpace::Definite(slot.size.width),
                AvailableSpace::Definite(self.height),
            ),
            window,
            cx,
        );
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}

#[cfg(test)]
mod tests {
    use super::{Sticky, StickyArea};
    use crate::{button::Button, scroll::ScrollbarAxis};
    use gpui::{
        Bounds, Context, FocusHandle, IntoElement, Pixels, Render, ScrollDelta, ScrollEvent,
        ScrollHandle, TestAppContext, VisualTestContext, Window, div, point, prelude::*, px,
    };

    struct Harness {
        scroll: ScrollHandle,
        enabled: bool,
        offset: f32,
        clicks: usize,
        wide: bool,
    }

    impl Harness {
        fn new() -> Self {
            Self {
                scroll: ScrollHandle::new(),
                enabled: true,
                offset: 0.,
                clicks: 0,
                wide: false,
            }
        }
    }

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().pl(px(57.)).pt(px(43.)).child(
                StickyArea::new("sticky-test", &self.scroll)
                    .w(px(320.))
                    .h(px(160.))
                    .when(self.wide, |area| area.axis(ScrollbarAxis::Both))
                    .child(
                        div()
                            .w_full()
                            .when(self.wide, |d| d.w(px(600.)))
                            .h(px(80.))
                            .flex_shrink_0(),
                    )
                    .sticky(
                        Sticky::new("first")
                            .height(px(40.))
                            .offset(px(self.offset))
                            .enabled(self.enabled)
                            .child(div().size_full().debug_selector(|| "first".into()).child(
                                Button::new("header-action").label("Pin action").on_click(
                                    cx.listener(|this, _, _, cx| {
                                        this.clicks += 1;
                                        cx.notify();
                                    }),
                                ),
                            )),
                    )
                    .child(div().w_full().h(px(120.)).flex_shrink_0())
                    .sticky(
                        Sticky::new("second")
                            .height(px(40.))
                            .enabled(self.enabled)
                            .child("Second"),
                    )
                    .child(div().w_full().h(px(400.)).flex_shrink_0()),
            )
        }
    }

    fn mount(
        cx: &mut TestAppContext,
        harness: Harness,
    ) -> (gpui::Entity<Harness>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, visual) = cx.add_window_view(|window, _| {
            window.activate_window();
            harness
        });
        visual.simulate_resize(gpui::size(px(800.), px(600.)));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        (view, visual)
    }

    fn header(visual: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
        visual
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("missing {selector}"))
    }

    fn scroll_to(view: &gpui::Entity<Harness>, visual: &mut VisualTestContext, x: f32, y: f32) {
        visual.update(|window, cx| {
            view.update(cx, |this, cx| {
                this.scroll.set_offset(point(px(-x), px(-y)));
                cx.notify();
            });
            window.draw(cx).clear(cx);
        });
    }

    #[gpui::test]
    fn sticky_tracks_this_frames_scroll_and_is_pushed_off_by_the_next(cx: &mut TestAppContext) {
        let (view, visual) = mount(cx, Harness::new());
        for (offset, expected) in [(0., 80.), (100., 0.), (220., -20.), (280., -80.)] {
            scroll_to(&view, visual, 0., offset);
            let first = header(visual, "first");
            assert!((first.left() - px(57.)).abs() < px(0.1), "{first:?}");
            assert!(
                (first.top() - px(43. + expected)).abs() < px(0.1),
                "{first:?}, offset={offset}"
            );
        }
    }

    #[gpui::test]
    fn a_pinned_header_takes_clicks_and_lets_the_wheel_through(cx: &mut TestAppContext) {
        let (view, visual) = mount(cx, Harness::new());
        scroll_to(&view, visual, 0., 100.);
        let action = header(visual, "first");
        let center = action.origin + point(px(24.), px(20.));
        visual.simulate_click(center, Default::default());
        visual.update(|window, cx| {
            assert_eq!(view.read(cx).clicks, 1);
            window.draw(cx).clear(cx);
        });
        visual.simulate_event(ScrollEvent {
            position: center,
            delta: ScrollDelta::Pixels(point(px(0.), px(-24.))),
            ..Default::default()
        });
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert_eq!(view.read(cx).scroll.offset().y, px(-124.));
        });
        assert_eq!(header(visual, "first").top(), px(43.));
    }

    #[gpui::test]
    fn the_inset_is_honored_and_disabled_headers_stay_in_flow(cx: &mut TestAppContext) {
        let (view, visual) = mount(
            cx,
            Harness {
                offset: 12.,
                ..Harness::new()
            },
        );
        scroll_to(&view, visual, 0., 100.);
        assert_eq!(header(visual, "first").top(), px(55.));
        visual.update(|window, cx| {
            view.update(cx, |this, cx| {
                this.enabled = false;
                cx.notify();
            });
            window.draw(cx).clear(cx);
        });
        assert_eq!(header(visual, "first").top(), px(23.));
    }

    #[gpui::test]
    fn two_axis_content_keeps_its_horizontal_position_and_width(cx: &mut TestAppContext) {
        let (view, visual) = mount(
            cx,
            Harness {
                wide: true,
                ..Harness::new()
            },
        );
        scroll_to(&view, visual, 80., 100.);
        let first = header(visual, "first");
        assert_eq!(first.left(), px(-23.));
        assert_eq!(first.top(), px(43.));
        assert_eq!(first.size.width, px(600.));
    }

    struct TabOrderHarness {
        scroll: ScrollHandle,
        outside_before: FocusHandle,
        inside: FocusHandle,
        header: FocusHandle,
        outside_after: FocusHandle,
    }

    impl Render for TabOrderHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let focusable = |id: &'static str, handle: &FocusHandle| {
                div()
                    .id(id)
                    .h(px(20.))
                    .track_focus(&handle.clone().tab_stop(true))
            };
            div()
                .size_full()
                .p(px(20.))
                .child(focusable("outside-before", &self.outside_before))
                .child(
                    StickyArea::new("sticky-tab-order", &self.scroll)
                        .w(px(320.))
                        .h(px(200.))
                        .sticky(
                            Sticky::new("focus-header")
                                .height(px(40.))
                                .child(focusable("header", &self.header)),
                        )
                        .child(focusable("inside", &self.inside)),
                )
                .child(focusable("outside-after", &self.outside_after))
        }
    }

    #[gpui::test]
    fn the_area_keeps_its_place_in_the_window_tab_order(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (view, visual) = cx.add_window_view(|window, cx| {
            window.activate_window();
            TabOrderHarness {
                scroll: ScrollHandle::new(),
                outside_before: cx.focus_handle(),
                inside: cx.focus_handle(),
                header: cx.focus_handle(),
                outside_after: cx.focus_handle(),
            }
        });
        visual.simulate_resize(gpui::size(px(800.), px(600.)));
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            view.read(cx).outside_before.clone().focus(window, cx);
            window.focus_next(cx);
            assert!(view.read(cx).inside.is_focused(window));
            window.focus_next(cx);
            assert!(view.read(cx).header.is_focused(window));
            window.focus_next(cx);
            assert!(view.read(cx).outside_after.is_focused(window));
        });
    }
}
