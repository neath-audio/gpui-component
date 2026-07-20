use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, Corners, Edges, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, Role, SharedString, StatefulInteractiveElement, StyleRefinement, Styled, Toggled,
    Window, div, prelude::FluentBuilder as _,
};
use smallvec::{SmallVec, smallvec};

use crate::{
    ActiveTheme, Disableable, Icon, Sizable, Size, StyledExt, h_flex, tooltip::ComponentTooltip,
};

#[derive(Default, Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ToggleVariant {
    #[default]
    Ghost,
    Outline,
}

pub trait ToggleVariants: Sized {
    /// Set the variant of the toggle.
    fn with_variant(self, variant: ToggleVariant) -> Self;
    /// Set the variant to ghost.
    fn ghost(self) -> Self {
        self.with_variant(ToggleVariant::Ghost)
    }
    /// Set the variant to outline.
    fn outline(self) -> Self {
        self.with_variant(ToggleVariant::Outline)
    }
}

/// A `Toggle` child, in call order — either an opaque rendered element (from
/// `.label()`/`.child()`/`.children()`) or an `Icon` kept unconverted so
/// render time can still resize it to `metrics().icon` and, when it's the
/// *only* kind of child present, detect the icon-only case (see
/// `Toggle::is_icon_only` and `RenderOnce for Toggle`).
enum ToggleChild {
    Element(AnyElement),
    // Boxed: `Icon` is much larger than `AnyElement`, and clippy's
    // large-enum-variant lint flags the resulting size gap otherwise.
    Icon(Box<Icon>),
}

#[derive(IntoElement)]
pub struct Toggle {
    id: ElementId,
    style: StyleRefinement,
    checked: bool,
    size: Size,
    variant: ToggleVariant,
    disabled: bool,
    border_corners: Corners<bool>,
    border_edges: Edges<bool>,
    children: SmallVec<[ToggleChild; 1]>,
    on_click: Option<Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>>,
    tooltip: ComponentTooltip,
}

impl Toggle {
    /// Create a new Toggle element.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: StyleRefinement::default(),
            checked: false,
            size: Size::default(),
            variant: ToggleVariant::default(),
            disabled: false,
            border_corners: Corners {
                top_left: true,
                top_right: true,
                bottom_left: true,
                bottom_right: true,
            },
            border_edges: Edges::all(true),
            children: smallvec![],
            on_click: None,
            tooltip: ComponentTooltip::default(),
        }
    }

    /// Set tooltip text for the toggle.
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip.text = Some((tooltip.into(), None));
        self
    }

    /// Add a label to the toggle.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        let label: SharedString = label.into();
        self.children
            .push(ToggleChild::Element(label.into_any_element()));
        self
    }

    /// Add icon to the toggle.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        let icon: Icon = icon.into();
        self.children.push(ToggleChild::Icon(Box::new(icon)));
        self
    }

    /// Whether every child added so far is an `Icon` (i.e. no
    /// `.label()`/`.child()`/`.children()` element was ever added). Icon-only
    /// toggles render as a square button — see `RenderOnce for Toggle`.
    ///
    /// NAMED EXCEPTION (docs/superpowers/specs/2026-07-19-depth-color-language-design.md,
    /// neath repo): the stock toggle was square only by coincidence before
    /// the Nova ladder retune (Medium: `min_w` 32 == `icon` 16 + `pad_x` 2x8);
    /// the retuned `pad_x` (8/10/10/10) breaks that arithmetic, so icon-only
    /// toggles now size explicitly off `metrics().height` instead of relying
    /// on `min_w` + content width to coincidentally match it.
    fn is_icon_only(&self) -> bool {
        !self.children.is_empty()
            && self
                .children
                .iter()
                .all(|child| matches!(child, ToggleChild::Icon(_)))
    }

    /// Set the checked state of the toggle, default: false
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set the callback to be called when the toggle is clicked.
    ///
    /// The `&bool` parameter represents the new checked state of the toggle.
    pub fn on_click(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub(crate) fn border_corners(mut self, corners: impl Into<Corners<bool>>) -> Self {
        self.border_corners = corners.into();
        self
    }

    pub(crate) fn border_edges(mut self, edges: impl Into<Edges<bool>>) -> Self {
        self.border_edges = edges.into();
        self
    }
}

impl ToggleVariants for Toggle {
    fn with_variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl ParentElement for Toggle {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(ToggleChild::Element));
    }
}

impl Disableable for Toggle {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Sizable for Toggle {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl Styled for Toggle {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Toggle {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let checked = self.checked;
        let disabled = self.disabled;
        let hoverable = !disabled && !checked;
        // Per-size Nova radius curve — matches Button (see
        // `Size::control_radius`); flat `theme.radius` pilled the short sizes.
        let rounding = self.size.control_radius(cx.theme());
        let m = self.size.metrics();
        let icon_only = self.is_icon_only();
        // Icons are rendered at metrics size like Button does (see
        // button.rs's `icon_size`), converting each `ToggleChild` into its
        // final `AnyElement` in call order — this preserves the exact
        // interleaving of labels and icons a caller built up via
        // `.label()`/`.icon()`/`.child()`.
        let icon_px = m.icon.to_pixels(window.rem_size());
        let children = self
            .children
            .into_iter()
            .map(|child| match child {
                ToggleChild::Element(el) => el,
                ToggleChild::Icon(icon) => (*icon).with_size(Size::Size(icon_px)).into(),
            })
            .collect::<Vec<AnyElement>>();

        div()
            .id(self.id)
            .role(Role::Button)
            .aria_toggled(if checked {
                Toggled::True
            } else {
                Toggled::False
            })
            .when_some(
                self.tooltip.text.as_ref().map(|(text, _)| text.clone()),
                |this, label| this.aria_label(label),
            )
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            // NAMED EXCEPTION (docs/superpowers/specs/2026-07-19-depth-color-language-design.md,
            // neath repo): icon-only toggles are sized explicitly off
            // `metrics().height` (both axes, no padding) to stay square;
            // labeled toggles keep the `min_w` + `pad_x`/`pad_y` box model.
            .map(|this| {
                if icon_only {
                    this.w(m.height).h(m.height).px_0()
                } else {
                    this.min_w(m.height).h(m.height).px(m.pad_x).py(m.pad_y)
                }
            })
            .text_size(m.text)
            .when(self.border_corners.top_left, |this| {
                this.rounded_tl(rounding)
            })
            .when(self.border_corners.top_right, |this| {
                this.rounded_tr(rounding)
            })
            .when(self.border_corners.bottom_left, |this| {
                this.rounded_bl(rounding)
            })
            .when(self.border_corners.bottom_right, |this| {
                this.rounded_br(rounding)
            })
            .when(self.variant == ToggleVariant::Outline, |this| {
                this.when(self.border_edges.left, |this| this.border_l_1())
                    .when(self.border_edges.right, |this| this.border_r_1())
                    .when(self.border_edges.top, |this| this.border_t_1())
                    .when(self.border_edges.bottom, |this| this.border_b_1())
                    .border_color(cx.theme().border)
                    .bg(cx.theme().tokens.background)
                    .when(cx.theme().shadow, |this| this.shadow_xs())
            })
            .when(hoverable, |this| {
                this.hover(|this| {
                    this.bg(cx.theme().tokens.accent)
                        .text_color(cx.theme().accent_foreground)
                })
            })
            .when(checked, |this| {
                this.bg(cx.theme().tokens.accent)
                    .text_color(cx.theme().accent_foreground)
            })
            .refine_style(&self.style)
            .children(children)
            .when(!disabled, |this| {
                this.when_some(self.on_click, |this, on_click| {
                    this.on_click(move |_, window, cx| on_click(&!checked, window, cx))
                })
            })
            .map(|this| self.tooltip.apply(this))
    }
}

/// A group of toggles.
#[derive(IntoElement)]
pub struct ToggleGroup {
    id: ElementId,
    style: StyleRefinement,
    size: Size,
    variant: ToggleVariant,
    disabled: bool,
    segmented: bool,
    items: Vec<Toggle>,
    on_click: Option<Rc<dyn Fn(&Vec<bool>, &mut Window, &mut App) + 'static>>,
}

impl ToggleGroup {
    /// Create a new ToggleGroup element.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: StyleRefinement::default(),
            size: Size::default(),
            variant: ToggleVariant::default(),
            disabled: false,
            segmented: false,
            items: Vec::new(),
            on_click: None,
        }
    }

    /// Add a child [`Toggle`] to the group.
    pub fn child(mut self, toggle: impl Into<Toggle>) -> Self {
        self.items.push(toggle.into());
        self
    }

    /// Add multiple [`Toggle`]s to the group.
    pub fn children(mut self, children: impl IntoIterator<Item = impl Into<Toggle>>) -> Self {
        self.items.extend(children.into_iter().map(Into::into));
        self
    }

    /// Set the callback to be called when the toggle group changes.
    ///
    /// The `&Vec<bool>` parameter represents the new check state of each [`Toggle`] in the group.
    pub fn on_click(
        mut self,
        on_click: impl Fn(&Vec<bool>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(on_click));
        self
    }

    /// Render the group as a connected segmented control.
    ///
    /// This keeps the existing multi-toggle behavior, but removes the default
    /// gap and joins adjacent item borders into a single segmented outline.
    pub fn segmented(mut self) -> Self {
        self.segmented = true;
        self
    }
}

impl Sizable for ToggleGroup {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl ToggleVariants for ToggleGroup {
    fn with_variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl Disableable for ToggleGroup {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Styled for ToggleGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for ToggleGroup {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let disabled = self.disabled;
        let items_len = self.items.len();
        let checks = self
            .items
            .iter()
            .map(|item| item.checked)
            .collect::<Vec<bool>>();
        let state = Rc::new(Cell::new(None));

        h_flex()
            .id(self.id)
            .role(Role::Toolbar)
            .when(!self.segmented, |this| this.gap_2())
            .refine_style(&self.style)
            .children(self.items.into_iter().enumerate().map({
                |(ix, item)| {
                    let state = state.clone();
                    let item = if !self.segmented || items_len == 1 {
                        item
                    } else if ix == 0 {
                        item.border_corners(Corners {
                            top_left: true,
                            top_right: false,
                            bottom_left: true,
                            bottom_right: false,
                        })
                        .border_edges(Edges {
                            left: true,
                            top: true,
                            right: true,
                            bottom: true,
                        })
                    } else if ix == items_len - 1 {
                        item.border_corners(Corners {
                            top_left: false,
                            top_right: true,
                            bottom_left: false,
                            bottom_right: true,
                        })
                        .border_edges(Edges {
                            left: false,
                            top: true,
                            right: true,
                            bottom: true,
                        })
                    } else {
                        item.border_corners(Corners {
                            top_left: false,
                            top_right: false,
                            bottom_left: false,
                            bottom_right: false,
                        })
                        .border_edges(Edges {
                            left: false,
                            top: true,
                            right: true,
                            bottom: true,
                        })
                    };

                    item.disabled(disabled)
                        .with_size(self.size)
                        .with_variant(self.variant)
                        .on_click(move |_, _, _| {
                            state.set(Some(ix));
                        })
                }
            }))
            .when(!disabled, |this| {
                this.when_some(self.on_click, |this, on_click| {
                    this.on_click(move |_, window, cx| {
                        if let Some(ix) = state.get() {
                            let mut checks = checks.clone();
                            checks[ix] = !checks[ix];
                            on_click(&checks, window, cx);
                        }
                    })
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IconName;

    #[gpui::test]
    fn test_toggle_builder(_cx: &mut gpui::TestAppContext) {
        let toggle = Toggle::new("complex-toggle")
            .label("Enable Feature")
            .icon(IconName::Check)
            .checked(true)
            .outline()
            .large()
            .disabled(false)
            .on_click(|_, _, _| {});

        assert_eq!(toggle.children.len(), 2); // label + icon
        assert!(toggle.checked);
        assert_eq!(toggle.variant, ToggleVariant::Outline);
        assert_eq!(toggle.size, Size::Large);
        assert!(!toggle.disabled);
        assert!(toggle.on_click.is_some());
        // A label was added alongside the icon, so this is not icon-only.
        assert!(!toggle.is_icon_only());
    }

    #[gpui::test]
    fn test_toggle_icon_only_detection(_cx: &mut gpui::TestAppContext) {
        // Only `.icon()` was ever called: icon-only, must stay square.
        let icon_only = Toggle::new("icon-only").icon(IconName::Check);
        assert!(icon_only.is_icon_only());

        // Icon followed by a label: no longer icon-only.
        let icon_then_label = Toggle::new("icon-then-label")
            .icon(IconName::Check)
            .label("Enabled");
        assert!(!icon_then_label.is_icon_only());

        // Label only, no icon at all: not icon-only (and not empty either).
        let label_only = Toggle::new("label-only").label("Enabled");
        assert!(!label_only.is_icon_only());

        // No children at all: not icon-only (nothing to be square about).
        let empty = Toggle::new("empty");
        assert!(!empty.is_icon_only());
    }

    #[gpui::test]
    fn test_toggle_group_builder(_cx: &mut gpui::TestAppContext) {
        let group = ToggleGroup::new("complex-group")
            .child(Toggle::new("toggle1").label("Option 1"))
            .child(Toggle::new("toggle2").label("Option 2").checked(true))
            .child(Toggle::new("toggle3").label("Option 3"))
            .outline()
            .large()
            .segmented()
            .disabled(false)
            .on_click(|_, _, _| {});

        assert_eq!(group.items.len(), 3);
        assert_eq!(group.variant, ToggleVariant::Outline);
        assert_eq!(group.size, Size::Large);
        assert!(group.segmented);
        assert!(!group.disabled);
        assert!(group.on_click.is_some());
    }
}
