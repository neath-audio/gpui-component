use crate::Size;
use std::rc::Rc;

use gpui::{
    App, ClickEvent, ElementId, InteractiveElement as _, IntoElement, ParentElement, RenderOnce,
    Role, SharedString, StatefulInteractiveElement, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _,
};

use crate::{
    ActiveTheme, Icon, IconName, StyledExt, h_flex,
    tooltip::{ManagedTooltipExt as _, Tooltip},
};

/// A breadcrumb navigation element.
#[derive(IntoElement)]
pub struct Breadcrumb {
    style: StyleRefinement,
    items: Vec<BreadcrumbItem>,
}

/// Item for the [`Breadcrumb`].
#[derive(IntoElement)]
pub struct BreadcrumbItem {
    id: ElementId,
    style: StyleRefinement,
    label: SharedString,
    on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
    disabled: bool,
    is_last: bool,
    tooltip: Option<SharedString>,
}

impl BreadcrumbItem {
    /// Create a new BreadcrumbItem with the given id and label.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            id: ElementId::Integer(0),
            style: StyleRefinement::default(),
            label: label.into(),
            on_click: None,
            disabled: false,
            is_last: false,
            tooltip: None,
        }
    }

    /// Show a tooltip on hover — for a crumb whose label is truncated and
    /// whose full text the user still needs, matching
    /// [`crate::button::Button::tooltip`].
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(on_click));
        self
    }

    fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// For internal use only.
    fn is_last(mut self, is_last: bool) -> Self {
        self.is_last = is_last;
        self
    }
}

impl Styled for BreadcrumbItem {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl From<&'static str> for BreadcrumbItem {
    fn from(value: &'static str) -> Self {
        Self::new(value)
    }
}

impl From<String> for BreadcrumbItem {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<SharedString> for BreadcrumbItem {
    fn from(value: SharedString) -> Self {
        Self::new(value)
    }
}

impl RenderOnce for BreadcrumbItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .id(self.id)
            .role(if self.on_click.is_some() && !self.disabled {
                Role::Link
            } else {
                Role::ListItem
            })
            .child(self.label)
            .text_color(cx.theme().muted_foreground)
            .when(self.is_last, |this| this.text_color(cx.theme().foreground))
            .when(self.disabled, |this| {
                this.text_color(cx.theme().muted_foreground)
            })
            .refine_style(&self.style)
            .when(!self.disabled, |this| {
                this.when_some(self.on_click, |this, on_click| {
                    // Hover feedback lives here rather than at the call site:
                    // `BreadcrumbItem` implements `Styled` but not
                    // `InteractiveElement`, so a caller cannot supply one. Runs
                    // after `refine_style`, so a call site can still override
                    // the REST color without losing the hover.
                    this.cursor_pointer()
                        .hover(|s| s.text_color(cx.theme().foreground).underline())
                        .on_click(move |event, window, cx| {
                            on_click(event, window, cx);
                        })
                })
            })
            .when_some(self.tooltip, |this, tooltip| {
                this.managed_tooltip(move |window, cx| {
                    Tooltip::new(tooltip.clone()).build(window, cx)
                })
            })
    }
}

impl Breadcrumb {
    /// Create a new breadcrumb.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// Add an [`BreadcrumbItem`] to the breadcrumb.
    pub fn child(mut self, item: impl Into<BreadcrumbItem>) -> Self {
        self.items.push(item.into());
        self
    }

    /// Add multiple [`BreadcrumbItem`] items to the breadcrumb.
    pub fn children(mut self, items: impl IntoIterator<Item = impl Into<BreadcrumbItem>>) -> Self {
        self.items.extend(items.into_iter().map(Into::into));
        self
    }
}

#[derive(IntoElement)]
struct BreadcrumbSeparator;
impl RenderOnce for BreadcrumbSeparator {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        Icon::new(IconName::ChevronRight)
            .text_color(cx.theme().muted_foreground)
            .size_3p5()
            .into_any_element()
    }
}

impl Styled for Breadcrumb {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Breadcrumb {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let items_count = self.items.len();

        let mut children = vec![];
        for (ix, item) in self.items.into_iter().enumerate() {
            let is_last = ix == items_count - 1;

            let item = item.id(ix);
            children.push(item.is_last(is_last).into_any_element());
            if !is_last {
                children.push(BreadcrumbSeparator.into_any_element());
            }
        }

        h_flex()
            .gap_1p5()
            .text_size(Size::Medium.text_size())
            .text_color(cx.theme().muted_foreground)
            .refine_style(&self.style)
            .children(children)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use gpui::{AppContext as _, Bounds, Context, Pixels, Render, TestAppContext, point, px};
    use gpui_base::{Root, TooltipOverlay, TooltipRequest, TooltipTransition};

    use super::*;
    use crate::root::WindowState;

    struct BreadcrumbTooltipHarness;

    impl Render for BreadcrumbTooltipHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(
                div()
                    .absolute()
                    .left(px(80.))
                    .top(px(100.))
                    .w(px(160.))
                    .h(px(24.))
                    .debug_selector(|| "breadcrumb-lane".into())
                    .child(
                        Breadcrumb::new()
                            .size_full()
                            .child(BreadcrumbItem::new("Folder").size_full().tooltip("Folder")),
                    ),
            )
        }
    }

    #[gpui::test]
    fn tooltip_anchors_to_the_breadcrumb_bounds(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|window, cx| {
            let view = cx.new(|_| BreadcrumbTooltipHarness);
            Root::new(view, window, cx)
        });
        let anchor = Rc::new(Cell::new(None::<Bounds<Pixels>>));
        cx.update(|window, cx| {
            let captured = anchor.clone();
            WindowState::tooltip_overlay(window, cx)
                .unwrap()
                .update(cx, |overlay, cx| {
                    *overlay = TooltipOverlay::new().render_with(move |view, transition, _, _| {
                        if let TooltipTransition::Switch { current, .. } = transition {
                            captured.set(Some(current));
                        }
                        view.into_any_element()
                    });
                    overlay.request_show(
                        TooltipRequest::new(Bounds::default(), |window, cx| {
                            Tooltip::new("Previous").build(window, cx)
                        }),
                        window,
                        cx,
                    );
                });
        });
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(501));
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let expected = cx.debug_bounds("breadcrumb-lane").unwrap();
        cx.simulate_mouse_move(
            point(expected.left() + px(8.), expected.top() + px(8.)),
            None,
            gpui::Modifiers::default(),
        );
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(anchor.get(), Some(expected));
    }
}
