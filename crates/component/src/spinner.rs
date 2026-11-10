use crate::{Icon, IconName, Sizable, Size};
use gpui::{
    Animation, AnimationExt as _, AnyElement, App, Hsla, IntoElement, ParentElement, Path,
    PathBuilder, Pixels, Point, RenderOnce, Styled as _, Transformation, Window, canvas, div,
    ease_in_out, percentage, point, prelude::FluentBuilder as _, px, rems,
};
use instant::Duration;

/// How a [`Spinner`] draws its activity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpinnerVariant {
    /// A rotating icon, [`IconName::Loader`] unless [`Spinner::icon`] picks another.
    #[default]
    Icon,
    /// Ten frames of Braille dots, stepping in place.
    Braille,
    /// A Braille cell that fills and drains, for longer waits.
    Sand,
    /// A three-quarter arc that turns.
    Ring,
}

impl SpinnerVariant {
    fn frames(self) -> &'static [&'static str] {
        match self {
            Self::Braille => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            Self::Sand => &[
                "⠁", "⠂", "⠄", "⡀", "⡈", "⡐", "⡠", "⣀", "⣁", "⣂", "⣄", "⣌", "⣔", "⣤", "⣥", "⣦",
                "⣮", "⣶", "⣷", "⣿", "⡿", "⠿", "⢟", "⠟", "⡛", "⠛", "⠫", "⢋", "⠋", "⠍", "⡉", "⠉",
                "⠑", "⠡", "⢁",
            ],
            Self::Ring | Self::Icon => &[],
        }
    }

    fn period(self) -> Duration {
        Duration::from_millis(match self {
            Self::Sand => 2000,
            Self::Ring => 900,
            Self::Braille | Self::Icon => 1000,
        })
    }

    /// The mark for one point in the cycle, `phase` running from 0 to 1.
    fn mark(self, diameter: Pixels, color: Hsla, phase: f32) -> AnyElement {
        let frames = self.frames();
        if frames.is_empty() {
            return ring(diameter, color, phase);
        }
        div()
            .size(diameter)
            .flex()
            .items_center()
            .justify_center()
            .text_size(diameter)
            .line_height(diameter)
            .text_color(color)
            .child(frames[(phase * frames.len() as f32) as usize % frames.len()])
            .into_any_element()
    }
}

fn ring(diameter: Pixels, color: Hsla, phase: f32) -> AnyElement {
    let stroke = (diameter * 0.08).max(px(1.5));
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let radius = (diameter / 2. - stroke).max(px(0.));
            if let Some(path) = arc_path(bounds.center(), radius, stroke, 0.75, phase) {
                window.paint_path(path, color);
            }
        },
    )
    .size(diameter)
    .flex_none()
    .into_any_element()
}

fn arc_path(
    center: Point<Pixels>,
    radius: Pixels,
    stroke: Pixels,
    fraction: f32,
    phase: f32,
) -> Option<Path<Pixels>> {
    if radius <= px(0.) {
        return None;
    }
    let start = (phase - 0.25) * std::f32::consts::TAU;
    let end = start + fraction * std::f32::consts::TAU;
    let mut path = PathBuilder::stroke(stroke);
    path.move_to(center + point(radius * start.cos(), radius * start.sin()));
    path.arc_to(
        point(radius, radius),
        px(0.),
        fraction > 0.5,
        true,
        center + point(radius * end.cos(), radius * end.sin()),
    );
    path.build().ok()
}

/// A cycling loading spinner.
#[derive(IntoElement)]
pub struct Spinner {
    size: Size,
    variant: SpinnerVariant,
    icon: Icon,
    speed: Duration,
    easing: Box<dyn Fn(f32) -> f32>,
    color: Option<Hsla>,
}

impl Spinner {
    /// Create a new loading spinner.
    pub fn new() -> Self {
        Self {
            size: Size::Medium,
            variant: SpinnerVariant::default(),
            speed: Duration::from_secs_f64(0.8),
            easing: Box::new(ease_in_out),
            icon: Icon::new(IconName::Loader),
            color: None,
        }
    }

    /// Set how the spinner draws, [`SpinnerVariant::Icon`] by default.
    pub fn variant(mut self, variant: SpinnerVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Rotate this icon, [`IconName::Loader`] by default. This selects
    /// [`SpinnerVariant::Icon`].
    ///
    /// Please ensure the icon used is suitable for a loading spinner.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = icon.into();
        self.variant = SpinnerVariant::Icon;
        self
    }

    /// Set the color. Without one, the spinner takes the current text color.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the easing function of a rotating [`SpinnerVariant::Icon`].
    pub fn ease(mut self, easing: impl Fn(f32) -> f32 + 'static) -> Self {
        self.easing = Box::new(easing);
        self
    }

    /// The same diameters as an [`Icon`] of this size.
    fn diameter(&self, window: &Window) -> Pixels {
        let rem = match self.size {
            Size::Size(size) => return size,
            Size::XSmall => 0.75,
            Size::Small => 0.875,
            Size::Medium => 1.,
            Size::Large => 1.5,
        };
        rems(rem).to_pixels(window.rem_size())
    }
}

impl Sizable for Spinner {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl RenderOnce for Spinner {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let variant = self.variant;
        if variant == SpinnerVariant::Icon {
            return div()
                .child(
                    self.icon
                        .with_size(self.size)
                        .when_some(self.color, |this, color| this.text_color(color))
                        .with_animation(
                            "circle",
                            Animation::new(self.speed).repeat().with_easing(self.easing),
                            |this, delta| this.transform(Transformation::rotate(percentage(delta))),
                        ),
                )
                .into_any_element();
        }

        let diameter = self.diameter(window);
        let color = self.color.unwrap_or_else(|| window.text_style().color);
        div()
            .size(diameter)
            .flex_none()
            .with_animation(
                "spinner",
                Animation::new(variant.period()).repeat(),
                move |this, phase| this.child(variant.mark(diameter, color, phase)),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Render, TestAppContext, size};

    const VARIANTS: [SpinnerVariant; 4] = [
        SpinnerVariant::Braille,
        SpinnerVariant::Sand,
        SpinnerVariant::Ring,
        SpinnerVariant::Icon,
    ];

    struct SpinnerHost(SpinnerVariant);

    impl Render for SpinnerHost {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            Spinner::new().variant(self.0)
        }
    }

    fn frames_requested(variant: SpinnerVariant, cx: &mut TestAppContext) -> usize {
        let window = cx.open_window(size(px(100.), px(100.)), |_, _| SpinnerHost(variant));
        cx.run_until_parked();
        window
            .update(cx, |_, window, cx| window.simulate_next_frame(cx))
            .unwrap()
    }

    #[gpui::test]
    fn reduced_motion_spinner_is_static_and_requests_no_frame(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        for variant in VARIANTS {
            assert_eq!(frames_requested(variant, cx), 0, "{variant:?}");
        }
    }

    #[gpui::test]
    fn every_variant_keeps_animating_while_motion_is_allowed(cx: &mut TestAppContext) {
        for variant in VARIANTS {
            assert!(frames_requested(variant, cx) > 0, "{variant:?}");
        }
    }

    #[test]
    fn a_custom_icon_selects_the_rotating_variant() {
        let spinner = Spinner::new()
            .variant(SpinnerVariant::Braille)
            .icon(IconName::Loader);
        assert_eq!(spinner.variant, SpinnerVariant::Icon);
    }

    #[test]
    fn the_rotating_icon_is_the_default() {
        assert_eq!(Spinner::new().variant, SpinnerVariant::Icon);
    }
}
