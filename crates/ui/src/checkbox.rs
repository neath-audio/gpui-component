use std::{rc::Rc, time::Duration};

use crate::{
    ActiveTheme, Disableable, FocusableExt, IconName, Selectable, Sizable, Size, StyledExt as _,
    icon::IconNamed, text::Text, tooltip::ComponentTooltip, v_flex,
};
use gpui::{
    Animation, AnimationExt, AnyElement, App, Div, ElementId, InteractiveElement, IntoElement,
    ParentElement, Rems, RenderOnce, Role, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, Toggled, Window, div, prelude::FluentBuilder as _, px, relative, rems,
    svg,
};

/// NAMED EXCEPTION (docs/superpowers/specs/2026-07-19-depth-color-language-design.md,
/// neath repo): compressed indicator ladder, user-ruled 2026-07-20 (round 2).
/// Neither the control-tier icon column (12/12/14/16/16 — Medium/Large read
/// too large beside their labels) nor Nova's single fixed 16px (sizes became
/// indistinguishable): the box steps 12/14/15/16/17px — distinct at every
/// `Size`, narrow range, roughly label-text + 2px (text tier is
/// 12/12/13/14/14; XXSmall floors at 12 = its text size, per the dense-tier
/// ruling 2026-07-20). Custom sizes interpolate between adjacent tiers keyed
/// off the size's `metrics().height`, clamped at the ends (mirrors
/// metrics.rs `interpolated()`). Shared by `Radio` (see radio.rs).
pub(crate) fn indicator_size(size: Size) -> Rems {
    /// (control_height_px, indicator_px) anchors on the canonical
    /// 20/24/28/32/36 control-height ladder.
    const ANCHORS: [(f32, f32); 5] = [
        (20., 12.), // XXSmall
        (24., 14.), // XSmall
        (28., 15.), // Small
        (32., 16.), // Medium
        (36., 17.), // Large
    ];
    const REM: f32 = 16.;
    match size {
        Size::XXSmall => rems(0.75), // 12px
        Size::XSmall => rems(0.875), // 14px
        Size::Small => rems(0.9375), // 15px
        Size::Medium => rems(1.0),   // 16px
        Size::Large => rems(1.0625), // 17px
        Size::Size(_) => {
            // metrics().height passes the raw custom height through, so key
            // the interpolation off it directly (px at the 16px rem base).
            let h = size.metrics().height.0 * REM;
            if h <= ANCHORS[0].0 {
                return rems(ANCHORS[0].1 / REM);
            }
            if h >= ANCHORS[4].0 {
                return rems(ANCHORS[4].1 / REM);
            }
            let i = (0..4)
                .find(|&i| h < ANCHORS[i + 1].0)
                .expect("height inside anchor bounds");
            let (lo, hi) = (ANCHORS[i], ANCHORS[i + 1]);
            let t = (h - lo.0) / (hi.0 - lo.0);
            rems((lo.1 + (hi.1 - lo.1) * t) / REM)
        }
    }
}

/// Check glyph = box − 4px: 1px border + 1px breathing room per side.
fn glyph_size(size: Size) -> Rems {
    rems(indicator_size(size).0 - 0.25)
}

/// A Checkbox element.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    base: Div,
    style: StyleRefinement,
    label: Option<Text>,
    children: Vec<AnyElement>,
    checked: bool,
    disabled: bool,
    size: Size,
    tab_stop: bool,
    tab_index: isize,
    on_click: Option<Rc<dyn Fn(&bool, &mut Window, &mut App) + 'static>>,
    tooltip: ComponentTooltip,
}

impl Checkbox {
    /// Create a new Checkbox with the given id.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            base: div(),
            style: StyleRefinement::default(),
            label: None,
            children: Vec::new(),
            checked: false,
            disabled: false,
            size: Size::default(),
            on_click: None,
            tab_stop: true,
            tab_index: 0,
            tooltip: ComponentTooltip::default(),
        }
    }

    /// Set tooltip text for the checkbox.
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip.text = Some((tooltip.into(), None));
        self
    }

    /// Set the label for the checkbox.
    pub fn label(mut self, label: impl Into<Text>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the checked state for the checkbox.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set the click handler for the checkbox.
    ///
    /// The `&bool` parameter indicates the new checked state after the click.
    pub fn on_click(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Set the tab stop for the checkbox, default is true.
    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.tab_stop = tab_stop;
        self
    }

    /// Set the tab index for the checkbox, default is 0.
    pub fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = tab_index;
        self
    }

    fn handle_click(
        on_click: &Option<Rc<dyn Fn(&bool, &mut Window, &mut App) + 'static>>,
        checked: bool,
        window: &mut Window,
        cx: &mut App,
    ) {
        let new_checked = !checked;
        if let Some(f) = on_click {
            (f)(&new_checked, window, cx);
        }
    }
}

impl InteractiveElement for Checkbox {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}
impl StatefulInteractiveElement for Checkbox {}

impl Styled for Checkbox {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl Disableable for Checkbox {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Selectable for Checkbox {
    fn selected(self, selected: bool) -> Self {
        self.checked(selected)
    }

    fn is_selected(&self) -> bool {
        self.checked
    }
}

impl ParentElement for Checkbox {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Sizable for Checkbox {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

pub(crate) fn checkbox_check_icon(
    id: ElementId,
    size: Size,
    checked: bool,
    disabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let toggle_state = window.use_keyed_state(id, cx, |_, _| checked);
    let color = if disabled {
        cx.theme().primary_foreground.opacity(0.5)
    } else {
        cx.theme().primary_foreground
    };

    // Sized per the compressed indicator ladder (module-level named
    // exception above). Positioned by the parent box's flex centering —
    // never by absolute offsets, which silently break when the box/glyph
    // arithmetic changes.
    svg()
        .size(glyph_size(size))
        .text_color(color)
        .map(|this| match checked {
            true => this.path(IconName::Check.path()),
            _ => this,
        })
        .map(|this| {
            if !disabled && checked != *toggle_state.read(cx) {
                let duration = Duration::from_secs_f64(0.25);
                cx.spawn({
                    let toggle_state = toggle_state.clone();
                    async move |cx| {
                        cx.background_executor().timer(duration).await;
                        _ = toggle_state.update(cx, |this, _| *this = checked);
                    }
                })
                .detach();

                this.with_animation(
                    ElementId::NamedInteger("toggle".into(), checked as u64),
                    Animation::new(Duration::from_secs_f64(0.25)),
                    move |this, delta| {
                        this.opacity(if checked { 1.0 * delta } else { 1.0 - delta })
                    },
                )
                .into_any_element()
            } else {
                this.into_any_element()
            }
        })
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let checked = self.checked;

        let focus_handle = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let is_focused = focus_handle.is_focused(window);

        let border_color = if checked {
            cx.theme().primary
        } else {
            cx.theme().input
        };
        let color = if self.disabled {
            border_color.opacity(0.5)
        } else {
            border_color
        };
        let radius = cx.theme().radius.min(px(4.));

        self.base
            .id(self.id.clone())
            .role(Role::CheckBox)
            .aria_toggled(if checked {
                Toggled::True
            } else {
                Toggled::False
            })
            .when_some(
                self.label.as_ref().map(|l| l.get_text(cx)),
                |this, label| this.aria_label(label),
            )
            .when(!self.disabled, |this| {
                this.track_focus(
                    &focus_handle
                        .tab_stop(self.tab_stop)
                        .tab_index(self.tab_index),
                )
            })
            .h_flex()
            .gap_2()
            .items_start()
            .line_height(relative(1.))
            .text_color(cx.theme().foreground)
            .text_size(self.size.metrics().text)
            .when(self.disabled, |this| {
                this.text_color(cx.theme().muted_foreground)
            })
            .rounded(cx.theme().radius * 0.5)
            .focus_ring(is_focused, px(2.), window, cx)
            .refine_style(&self.style)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    // Compressed indicator ladder, not control-height- or
                    // icon-tier-sized (see the named exception above).
                    .size(indicator_size(self.size))
                    .flex_shrink_0()
                    .border_1()
                    .border_color(color)
                    .rounded(radius)
                    .when(cx.theme().shadow && !self.disabled, |this| this.shadow_xs())
                    .map(|this| match checked {
                        false => this.bg(cx.theme().input_fill()),
                        true if self.disabled => this.bg(color),
                        true => this.bg(cx.theme().tokens.primary),
                    })
                    .child(checkbox_check_icon(
                        self.id,
                        self.size,
                        checked,
                        self.disabled,
                        window,
                        cx,
                    )),
            )
            .when(self.label.is_some() || !self.children.is_empty(), |this| {
                this.child(
                    v_flex()
                        .flex_1()
                        .overflow_hidden()
                        .line_height(relative(1.2))
                        .gap_1()
                        .map(|this| {
                            if let Some(label) = self.label {
                                this.child(
                                    div()
                                        .size_full()
                                        .text_color(cx.theme().foreground)
                                        .when(self.disabled, |this| {
                                            this.text_color(cx.theme().muted_foreground)
                                        })
                                        .line_height(relative(1.))
                                        .child(label),
                                )
                            } else {
                                this
                            }
                        })
                        .children(self.children),
                )
            })
            .on_mouse_down(gpui::MouseButton::Left, |_, window, _| {
                // Avoid focus on mouse down.
                window.prevent_default();
            })
            .when(!self.disabled, |this| {
                this.on_click({
                    let on_click = self.on_click.clone();
                    move |_, window, cx| {
                        window.prevent_default();
                        Self::handle_click(&on_click, checked, window, cx);
                    }
                })
            })
            .map(|this| self.tooltip.apply(this))
    }
}

#[cfg(test)]
mod tests {
    use super::{glyph_size, indicator_size};
    use crate::Size;
    use gpui::{px, rems};

    #[test]
    fn indicator_ladder_is_pinned() {
        // Compressed indicator ladder (named exception above):
        // 12/14/15/16/17px across XXSmall-Large.
        assert_eq!(indicator_size(Size::XXSmall), rems(0.75));
        assert_eq!(indicator_size(Size::XSmall), rems(0.875));
        assert_eq!(indicator_size(Size::Small), rems(0.9375));
        assert_eq!(indicator_size(Size::Medium), rems(1.0));
        assert_eq!(indicator_size(Size::Large), rems(1.0625));
    }

    #[test]
    fn custom_indicator_interpolates_between_tiers() {
        // 22px control height sits halfway between XXSmall(20) and
        // XSmall(24) -> indicator halfway between 12 and 14 = 13px.
        assert_eq!(indicator_size(Size::Size(px(22.))), rems(13. / 16.));
        // 30px sits halfway between Small(28) and Medium(32) -> 15.5px.
        assert_eq!(indicator_size(Size::Size(px(30.))), rems(15.5 / 16.));
        // Exact tier heights land on the tier's own value.
        assert_eq!(indicator_size(Size::Size(px(32.))), rems(1.0));
        // Clamped at both ends.
        assert_eq!(indicator_size(Size::Size(px(10.))), rems(0.75));
        assert_eq!(indicator_size(Size::Size(px(64.))), rems(1.0625));
    }

    #[test]
    fn glyph_is_box_minus_4px() {
        for size in [
            Size::XXSmall,
            Size::XSmall,
            Size::Small,
            Size::Medium,
            Size::Large,
        ] {
            assert_eq!(glyph_size(size), rems(indicator_size(size).0 - 0.25));
        }
        // XXSmall floor: 12px box -> 8px glyph.
        assert_eq!(glyph_size(Size::XXSmall), rems(0.5));
    }
}
