use crate::{
    ActiveTheme, Disableable, Side, Sizable, Size, StyledExt, h_flex, text::Text,
    tooltip::ComponentTooltip,
};
use gpui::{
    Animation, AnimationExt as _, App, Background, ElementId, Hsla, InteractiveElement,
    IntoElement, ParentElement as _, RenderOnce, SharedString, StyleRefinement, Styled, Window,
    div, prelude::FluentBuilder as _, px, rems,
};
use std::{rc::Rc, time::Duration};

/// A Switch element that can be toggled on or off.
#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    style: StyleRefinement,
    checked: bool,
    disabled: bool,
    label: Option<Text>,
    label_side: Side,
    on_click: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
    size: Size,
    color: Option<Hsla>,
    tooltip: ComponentTooltip,
}

impl Switch {
    /// Create a new Switch element.
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id: ElementId = id.into();
        Self {
            id: id.clone(),
            style: StyleRefinement::default(),
            checked: false,
            disabled: false,
            label: None,
            on_click: None,
            label_side: Side::Right,
            size: Size::Medium,
            color: None,
            tooltip: ComponentTooltip::default(),
        }
    }

    /// Set the checked state of the switch.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set the label of the switch.
    pub fn label(mut self, label: impl Into<Text>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Add a click handler for the switch.
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(&bool, &mut Window, &mut App) + 'static,
    {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Set the background color of the switch when checked.
    /// Defaults to `cx.theme().primary`.
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set tooltip text for the switch.
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip.text = Some((tooltip.into(), None));
        self
    }
}

impl Styled for Switch {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl Sizable for Switch {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl Disableable for Switch {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let checked = self.checked;
        let on_click = self.on_click.clone();
        let toggle_state = window.use_keyed_state(self.id.clone(), cx, |_, _| checked);

        let checked_bg = self
            .color
            .map(Background::from)
            .unwrap_or(cx.theme().tokens.primary.into());
        let (bg, toggle_bg): (Background, Background) = match checked {
            true => (checked_bg, cx.theme().tokens.switch_thumb.into()),
            false => (
                cx.theme().tokens.switch.into(),
                cx.theme().tokens.switch_thumb.into(),
            ),
        };

        let (bg, toggle_bg) = if self.disabled {
            (
                if checked { bg.opacity(0.5) } else { bg },
                toggle_bg.opacity(0.35),
            )
        } else {
            (bg, toggle_bg)
        };

        // NAMED EXCEPTION (docs/superpowers/specs/2026-07-19-depth-color-
        // language-design.md, neath repo), re-ruled 2026-07-21: the first
        // ladder pass stretched tracks to 2:1 with a height-minus-inset
        // thumb, which read as "width and thumb not matching" in smoke.
        // Nova's real relationship (default 32x18.4/16, sm 24x14/12) is
        // thumb = HALF the track width, track height = thumb + 1px inset
        // per side. Width still keys off the control-height axis
        // (20/24/28/32/36) so every `Size` step stays visually distinct
        // (user ruling 2026-07-20): tracks 20x12 / 24x14 / 28x16 / 32x18 /
        // 36x20 with 10/12/14/16/18 thumbs — XSmall lands Nova sm exactly,
        // Medium lands Nova default (rounded to integer height). Thumb
        // travel (`max_x` below) re-derives from these at every use.
        let m = self.size.metrics();
        let bg_width = m.height.to_pixels(window.rem_size());
        let bar_width = rems(m.height.0 * 0.5).to_pixels(window.rem_size());
        let inset = px(1.);
        let bg_height = bar_width + inset * 2.;
        let radius = if cx.theme().radius >= px(4.) {
            bg_height
        } else {
            cx.theme().radius
        };

        div().refine_style(&self.style).child(
            h_flex()
                .id(self.id.clone())
                .gap_2()
                .items_start()
                .when(self.label_side.is_left(), |this| this.flex_row_reverse())
                .child(
                    // Switch Bar
                    div()
                        .id(self.id.clone())
                        .w(bg_width)
                        .h(bg_height)
                        .rounded(radius)
                        .flex()
                        .items_center()
                        .border(inset)
                        .border_color(cx.theme().transparent)
                        .bg(bg)
                        .map(|this| self.tooltip.apply(this))
                        .child(
                            // Switch Toggle
                            div()
                                .rounded(radius)
                                .bg(toggle_bg)
                                .shadow_md()
                                .size(bar_width)
                                .map(|this| {
                                    let prev_checked = toggle_state.read(cx);
                                    if !self.disabled && *prev_checked != checked {
                                        let duration = Duration::from_secs_f64(0.15);
                                        cx.spawn({
                                            let toggle_state = toggle_state.clone();
                                            async move |cx| {
                                                cx.background_executor().timer(duration).await;
                                                _ = toggle_state
                                                    .update(cx, |this, _| *this = checked);
                                            }
                                        })
                                        .detach();

                                        this.with_animation(
                                            ElementId::NamedInteger("move".into(), checked as u64),
                                            Animation::new(duration),
                                            move |this, delta| {
                                                let max_x = bg_width - bar_width - inset * 2;
                                                let x = if checked {
                                                    max_x * delta
                                                } else {
                                                    max_x - max_x * delta
                                                };
                                                this.left(x)
                                            },
                                        )
                                        .into_any_element()
                                    } else {
                                        let max_x = bg_width - bar_width - inset * 2;
                                        let x = if checked { max_x } else { px(0.) };
                                        this.left(x).into_any_element()
                                    }
                                }),
                        ),
                )
                .when_some(self.label, |this, label| {
                    this.child(div().line_height(bg_height).text_size(m.text).child(label))
                })
                .when_some(
                    on_click
                        .as_ref()
                        .map(|c| c.clone())
                        .filter(|_| !self.disabled),
                    |this, on_click| {
                        let toggle_state = toggle_state.clone();
                        this.on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                            cx.stop_propagation();
                            _ = toggle_state.update(cx, |this, _| *this = checked);
                            on_click(&!checked, window, cx);
                        })
                    },
                ),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::Size;

    #[test]
    fn track_derives_nova_proportions_from_control_height() {
        // Track = metrics().height wide; thumb = half that; track height =
        // thumb + 2px of inset (see the named exception in `render`):
        // 20x12/10 / 24x14/12 / 28x16/14 / 32x18/16 / 36x20/18. XSmall is
        // Nova's sm switch exactly (24x14, 12 thumb); Medium is Nova's
        // default (32x18.4, 16 thumb) at integer height. Pinned at the
        // 16px rem base the ladder is authored against.
        for (size, w, h, thumb) in [
            (Size::XXSmall, 20., 12., 10.),
            (Size::XSmall, 24., 14., 12.),
            (Size::Small, 28., 16., 14.),
            (Size::Medium, 32., 18., 16.),
            (Size::Large, 36., 20., 18.),
        ] {
            let width = size.metrics().height.0 * 16.;
            assert_eq!(width, w);
            assert_eq!(width * 0.5, thumb);
            assert_eq!(width * 0.5 + 2., h);
        }
    }
}
