//! SplitButton — a hairline-bordered horizontal group of independent segments
//! divided by 1 px rules. IconDropdown is a thin adapter over
//! [`DropdownMenuPopover`].

use std::rc::Rc;

use gpui::{
    Anchor, AnyElement, App, Context, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StyleRefinement, Styled, Window, div, px,
};

use crate::{
    ActiveTheme, StyledExt as _, h_flex,
    menu::{DropdownMenuPopover, PopupMenu},
};

use super::IconButton;

/// A hairline-bordered horizontal group of independent segments.
///
/// Chrome only: interactivity lives in the child IconButtons/IconToggles.
#[derive(IntoElement)]
pub struct SplitButton {
    id: ElementId,
    segments: Vec<AnyElement>,
    style: StyleRefinement,
}

impl SplitButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            segments: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// Append one segment (any element; usually IconButton/IconToggle).
    pub fn segment(mut self, segment: impl IntoElement) -> Self {
        self.segments.push(segment.into_any_element());
        self
    }

    #[cfg(test)]
    fn divider_count(&self) -> usize {
        self.segments.len().saturating_sub(1)
    }
}

impl Styled for SplitButton {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for SplitButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let divider_color = cx.theme().border;
        let n = self.segments.len();
        h_flex()
            .id(self.id)
            .items_center()
            .rounded(px(5.))
            .border_1()
            .border_color(cx.theme().border)
            .refine_style(&self.style)
            .children(
                self.segments
                    .into_iter()
                    .enumerate()
                    .flat_map(move |(i, seg)| {
                        let mut parts: Vec<AnyElement> = Vec::new();
                        parts.push(div().child(seg).into_any_element());
                        if i + 1 < n {
                            parts.push(
                                div()
                                    .w(px(1.))
                                    .self_stretch()
                                    .bg(divider_color)
                                    .into_any_element(),
                            );
                        }
                        parts
                    }),
            )
    }
}

type MenuBuilder = Rc<dyn Fn(PopupMenu, &mut Window, &mut Context<PopupMenu>) -> PopupMenu>;

/// An IconButton that opens a PopupMenu on left click, anchored under the
/// trigger. A thin wrapper over [`DropdownMenuPopover`].
#[derive(IntoElement)]
pub struct IconDropdown {
    id: ElementId,
    trigger: IconButton,
    builder: MenuBuilder,
    anchor: Anchor,
}

impl IconDropdown {
    pub fn new(
        id: impl Into<ElementId>,
        trigger: IconButton,
        builder: impl Fn(PopupMenu, &mut Window, &mut Context<PopupMenu>) -> PopupMenu + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            trigger,
            builder: Rc::new(builder),
            anchor: Anchor::TopLeft,
        }
    }
}

impl RenderOnce for IconDropdown {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let builder = self.builder;
        DropdownMenuPopover::new(
            self.id,
            self.anchor,
            self.trigger.hoverable(true),
            move |menu, window, cx| builder(menu, window, cx),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n_segments_produce_n_minus_one_dividers() {
        assert_eq!(SplitButton::new("empty").divider_count(), 0);
        assert_eq!(SplitButton::new("one").segment(div()).divider_count(), 0);
        assert_eq!(
            SplitButton::new("two")
                .segment(div())
                .segment(div())
                .divider_count(),
            1
        );
        assert_eq!(
            SplitButton::new("three")
                .segment(div())
                .segment(div())
                .segment(div())
                .divider_count(),
            2
        );
    }

    #[test]
    fn icon_dropdown_retains_top_left_and_builder() {
        let dropdown = IconDropdown::new("menu", IconButton::new("trigger"), |menu, _, _| menu);
        assert_eq!(dropdown.anchor, Anchor::TopLeft);
        let _ = dropdown.builder;
    }
}
