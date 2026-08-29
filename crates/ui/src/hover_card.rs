use std::rc::Rc;

use gpui::{
    Anchor, AnyElement, App, Context, Corners, ElementId, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
};
use gpui_base::HoverCard as BaseHoverCard;
pub use gpui_base::HoverCardState;
use instant::Duration;

use crate::{
    ActiveTheme as _, Material, MaterialDepth, StyledExt as _, popover::Popover,
    styled::resolved_corner_radii,
};

/// A hover card element that displays content when hovering over a trigger element.
///
/// Similar to Popover but triggered by mouse hover instead of click, with configurable delays
/// for showing and hiding the content.
#[derive(IntoElement)]
pub struct HoverCard {
    id: ElementId,
    style: StyleRefinement,
    anchor: Anchor,
    trigger: Option<Box<dyn FnOnce(&mut Window, &App) -> AnyElement + 'static>>,
    content: Option<
        Rc<
            dyn Fn(&mut HoverCardState, &mut Window, &mut Context<HoverCardState>) -> AnyElement
                + 'static,
        >,
    >,
    children: Vec<AnyElement>,
    open_delay: Duration,
    close_delay: Duration,
    appearance: bool,
    on_open_change: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
}

impl HoverCard {
    /// Create a new HoverCard.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: StyleRefinement::default(),
            anchor: Anchor::TopCenter,
            trigger: None,
            content: None,
            children: vec![],
            open_delay: Duration::from_secs_f64(0.6),
            close_delay: Duration::from_secs_f64(0.3),
            appearance: true,
            on_open_change: None,
        }
    }

    /// Set the anchor corner of the hover card, default is [`Anchor::TopCenter`].
    pub fn anchor(mut self, anchor: impl Into<Anchor>) -> Self {
        self.anchor = anchor.into();
        self
    }

    /// Set the trigger element of the hover card.
    pub fn trigger<T>(mut self, trigger: T) -> Self
    where
        T: IntoElement + 'static,
    {
        self.trigger = Some(Box::new(|_, _| trigger.into_any_element()));
        self
    }

    /// Set the content builder of the hover card.
    pub fn content<F, E>(mut self, content: F) -> Self
    where
        F: Fn(&mut HoverCardState, &mut Window, &mut Context<HoverCardState>) -> E + 'static,
        E: IntoElement + 'static,
    {
        self.content = Some(Rc::new(move |state, window, cx| {
            content(state, window, cx).into_any_element()
        }));
        self
    }

    /// Set the delay before showing the hover card, default is 600ms.
    pub fn open_delay(mut self, duration: Duration) -> Self {
        self.open_delay = duration;
        self
    }

    /// Set the delay before hiding the hover card, default is 300ms.
    pub fn close_delay(mut self, duration: Duration) -> Self {
        self.close_delay = duration;
        self
    }

    /// Set whether to apply default appearance styles, default is `true`.
    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = appearance;
        self
    }

    /// Set a callback to be called when the open state changes.
    pub fn on_open_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&bool, &mut Window, &mut App) + 'static,
    {
        self.on_open_change = Some(Rc::new(callback));
        self
    }
}

impl Styled for HoverCard {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for HoverCard {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for HoverCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Some(trigger) = self.trigger else {
            return div().id("empty").into_any_element();
        };

        let anchor = self.anchor;
        let appearance = self.appearance;
        let content = self.content;
        let children = self.children;
        let style = self.style;
        let material_id = (self.id.clone(), "material");
        let material_host_id = (self.id.clone(), "material-host");

        BaseHoverCard::new(self.id)
            .anchor(anchor)
            .open_delay(self.open_delay)
            .close_delay(self.close_delay)
            .trigger((trigger)(window, cx))
            .content(move |state, window, cx| {
                let corner_radii = resolved_corner_radii(
                    if appearance {
                        Corners::all(cx.theme().radius)
                    } else {
                        Corners::default()
                    },
                    &style,
                    window.rem_size(),
                );
                let surface = Popover::render_popover_surface(appearance, cx)
                    .overflow_hidden()
                    .when_some(content, |this, content| {
                        this.child((content)(state, window, cx))
                    })
                    .children(children)
                    .refine_style(&style);
                let material_host = div().id(material_host_id).occlude();
                #[cfg(test)]
                let material_host =
                    material_host.debug_selector(|| "hover-card-material-host".into());

                Popover::offset_popover_surface(
                    material_host.child(
                        Material::new(material_id, MaterialDepth::Overlay, surface)
                            .corner_radii(corner_radii),
                    ),
                    anchor,
                )
            })
            .when_some(self.on_open_change, |this, callback| {
                this.on_open_change(move |open, window, cx| callback(open, window, cx))
            })
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Context, Render, TestAppContext, point, px};

    use super::*;
    use crate::material::{MaterialDepth, clear_painted_materials, take_painted_materials};

    struct HoverCardHarness;

    impl Render for HoverCardHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let delay = Duration::from_millis(1);
            HoverCard::new("hover-card")
                .appearance(false)
                .bg(cx.theme().popover)
                .rounded(px(19.))
                .open_delay(delay)
                .close_delay(delay)
                .trigger(
                    div()
                        .debug_selector(|| "hover-card-trigger".into())
                        .size(px(20.)),
                )
                .content(|_, _, _| {
                    div()
                        .debug_selector(|| "hover-card-surface-content".into())
                        .size(px(10.))
                })
        }
    }

    #[gpui::test]
    fn overlay_material_wraps_only_the_open_hover_card_surface(cx: &mut TestAppContext) {
        let delay = Duration::from_millis(1);
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|_, _| HoverCardHarness);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert!(cx.debug_bounds("hover-card-trigger").is_some());

        cx.simulate_mouse_move(point(px(10.), px(10.)), None, gpui::Modifiers::default());
        cx.executor().advance_clock(delay);
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        clear_painted_materials();
        cx.update(|window, cx| window.draw(cx).clear(cx));

        assert!(cx.debug_bounds("hover-card-surface-content").is_some());
        assert!(cx.debug_bounds("hover-card-trigger").is_some());
        let material = take_painted_materials()
            .into_iter()
            .filter(|material| material.id.to_string() == "hover-card-material")
            .collect::<Vec<_>>();
        assert_eq!(material.len(), 1, "HoverCard mounts one surface Material");
        assert_eq!(material[0].depth, MaterialDepth::Overlay);
        assert_eq!(material[0].corner_radii, Corners::all(px(19.)));
        assert_eq!(
            cx.debug_bounds("hover-card-material-host"),
            Some(material[0].bounds),
            "the BaseHoverCard hover host must share the visible Material bounds",
        );

        let visible_edge = point(
            material[0].bounds.center().x,
            material[0].bounds.bottom() - px(1.),
        );
        cx.simulate_mouse_move(visible_edge, None, gpui::Modifiers::default());
        cx.executor().advance_clock(delay);
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert!(
            cx.debug_bounds("hover-card-surface-content").is_some(),
            "hovering the visible offset edge must keep the card open",
        );

        cx.simulate_mouse_move(point(px(100.), px(100.)), None, gpui::Modifiers::default());
        cx.executor().advance_clock(delay);
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert!(cx.debug_bounds("hover-card-surface-content").is_none());
        assert!(cx.debug_bounds("hover-card-trigger").is_some());
    }
}
