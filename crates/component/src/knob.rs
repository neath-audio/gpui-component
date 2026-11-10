//! A controlled rotary value control with relative vertical dragging.

use std::{ops::RangeInclusive, rc::Rc};

use gpui::{
    AbsoluteLength, AccessibleAction, App, Bounds, DispatchPhase, Element, ElementId, FontWeight,
    GlobalElementId, InspectorElementId, InteractiveElement as _, IntoElement, LayoutId,
    MouseButton, MouseMoveEvent, MouseUpEvent, ParentElement as _, PathBuilder, Pixels,
    Refineable as _, RenderOnce, Role, SharedString, StatefulInteractiveElement as _,
    StyleRefinement, Styled, TextStyle, Window, accesskit, canvas, div, fill, point,
    prelude::FluentBuilder as _, px, rems,
};

use crate::{
    ActiveTheme as _, Disableable, Sizable, Size, StyledExt as _, StyledTypography as _, TextSize,
    ThemeStyled as _, label::Label,
};

type Change = Rc<dyn Fn(f32, &mut Window, &mut App)>;

#[derive(Clone, Copy)]
struct Range {
    min: f32,
    max: f32,
    step: f32,
    /// Whether position runs geometrically from the minimum to the maximum.
    log: bool,
}

impl Range {
    fn new(min: f32, max: f32, step: f32) -> Self {
        let min = if min.is_finite() { min } else { 0. };
        let max = if max.is_finite() {
            max.max(min)
        } else {
            min + 100.
        };
        Self {
            min,
            max,
            step: if step.is_finite() { step.max(0.) } else { 1. },
            log: false,
        }
    }

    /// Place values geometrically along the range. Taken only when the range starts above zero
    /// and has some width; otherwise the range stays linear, as a logarithm of zero or less has
    /// no meaning.
    fn logarithmic(mut self, on: bool) -> Self {
        self.log = on && self.min > 0. && self.max > self.min;
        self
    }

    fn clamp(self, value: f32) -> f32 {
        if value.is_finite() {
            value.clamp(self.min, self.max)
        } else {
            self.min
        }
    }

    fn snap(self, value: f32) -> f32 {
        let value = self.clamp(value);
        // A logarithmic step is a distance in position, not a grid of values to land on.
        if self.log || self.step == 0. || value == self.min || value == self.max {
            return value;
        }
        let min = self.min as f64;
        let step = self.step as f64;
        let lower = min + ((value as f64 - min) / step).floor() * step;
        let upper = (lower + step).min(self.max as f64);
        if value as f64 - lower < upper - value as f64 {
            lower as f32
        } else {
            upper as f32
        }
    }

    /// The value one step from `value` in `direction`. On a logarithmic range the step is in
    /// position: the value is multiplied by e to the step, and a zero step is 1%.
    fn next(self, value: f32, direction: f32) -> f32 {
        if self.log {
            let step = if self.step == 0. { 0.01 } else { self.step };
            return self.shifted(value, direction, step);
        }
        if self.step == 0. {
            return self.clamp(
                (value as f64 + (self.max as f64 - self.min as f64) * direction as f64 / 100.)
                    .clamp(self.min as f64, self.max as f64) as f32,
            );
        }
        let position = (value as f64 - self.min as f64) / self.step as f64;
        let index = if direction > 0. {
            (position + 0.00001).floor() + 1.
        } else {
            (position - 0.00001).ceil() - 1.
        };
        self.clamp(
            (self.min as f64 + index * self.step as f64).clamp(self.min as f64, self.max as f64)
                as f32,
        )
    }

    /// `value` moved by `amount` in `direction`, added on a linear range and, on a logarithmic
    /// one, as a distance in position (the value is multiplied by e to the amount).
    fn shifted(self, value: f32, direction: f32, amount: f32) -> f32 {
        if self.log {
            self.clamp(self.clamp(value) * (direction * amount).exp())
        } else {
            self.clamp(self.clamp(value) + direction * amount)
        }
    }

    /// The value `fraction` of the way along the range, the inverse of [`Self::fraction`].
    fn at_fraction(self, fraction: f32) -> f32 {
        if fraction <= 0. {
            self.min
        } else if fraction >= 1. {
            self.max
        } else if self.log {
            self.clamp(
                (self.min as f64 * (self.max as f64 / self.min as f64).powf(fraction as f64))
                    as f32,
            )
        } else {
            self.clamp(
                (self.min as f64 + fraction as f64 * (self.max as f64 - self.min as f64)) as f32,
            )
        }
    }

    /// How far along the range `value` sits, from 0 at the minimum to 1 at the maximum.
    fn fraction(self, value: f32) -> f32 {
        if self.log {
            let (min, max) = (self.min as f64, self.max as f64);
            return ((self.clamp(value) as f64 / min).ln() / (max / min).ln()).clamp(0., 1.) as f32;
        }
        if self.max > self.min {
            ((self.clamp(value) as f64 - self.min as f64) / (self.max as f64 - self.min as f64))
                as f32
        } else {
            0.
        }
    }

    /// The value a bipolar arc grows from: zero when the range holds it, else the midpoint.
    fn centre(self) -> f32 {
        if self.min <= 0. && 0. <= self.max {
            0.
        } else {
            ((self.min as f64 + self.max as f64) / 2.) as f32
        }
    }
}

/// Where the dial's sweep starts, in degrees clockwise from the right: the lower left.
const SWEEP_START: f32 = 135.;
/// How far the sweep turns, over the top to the lower right.
const SWEEP: f32 = 270.;
/// How much a drag moves while Shift is held, for fine adjustment.
const FINE_DRAG: f32 = 0.1;
/// How far a press may wander and still be a click, as GPUI's own drags measure it.
const DRAG_THRESHOLD: Pixels = px(2.);
/// The drag that sweeps the whole dial, as a multiple of the rem size (160 px at the default).
const DRAG_DISTANCE_REMS: f32 = 10.;

/// The angle of `fraction` of the way along the sweep, in degrees.
fn sweep_angle(fraction: f32) -> f32 {
    SWEEP_START + SWEEP * fraction
}

/// The value arc as its start and end angles in degrees, the start never after the end. It runs
/// from the minimum to the value, or from `origin` to the value on whichever side the value
/// lies; at the origin it is empty.
fn value_arc(range: Range, value: f32, origin: Option<f32>) -> (f32, f32) {
    let at = range.fraction(value);
    let from = origin.map_or(0., |origin| range.fraction(origin));
    (sweep_angle(from.min(at)), sweep_angle(from.max(at)))
}

struct CaptionMetrics {
    min: f32,
    max: f32,
    style: TextStyle,
    rem_size: Pixels,
    width: Pixels,
}

struct KnobMemory {
    drag: Option<(Pixels, f32)>,
    /// Whether the drag in progress is in fine mode, so a change of Shift re-anchors it.
    fine: bool,
    /// Whether the press has moved past [`DRAG_THRESHOLD`] and become a drag.
    moved: bool,
    reset_handler: Option<Change>,
    range: Range,
    value: f32,
    mounted_default: f32,
    reset_value: f32,
    enabled: bool,
    distance: Pixels,
    change: Option<Change>,
    caption_metrics: Option<CaptionMetrics>,
}

impl KnobMemory {
    fn reset(&mut self) -> Option<(Change, f32)> {
        // A second press ends capture before notifying the owner. Its later
        // move/release events must never replace the reset with a dragged value.
        self.drag = None;
        if !self.enabled || self.value == self.reset_value {
            return None;
        }
        self.value = self.reset_value;
        self.reset_handler
            .clone()
            .or_else(|| self.change.clone())
            .map(|change| (change, self.reset_value))
    }

    /// Follow the pointer to `y`. Shift moves at a tenth of the speed; pressing or releasing it
    /// mid-drag re-anchors at the pointer so the value never jumps.
    fn update_pointer(&mut self, y: Pixels, shift: bool) -> Option<(Change, f32)> {
        let (start_y, start_value) = self.drag?;
        if !self.enabled {
            return None;
        }
        if !self.moved {
            if (y - start_y).abs() <= DRAG_THRESHOLD {
                return None;
            }
            self.moved = true;
        }
        if shift != self.fine {
            self.fine = shift;
            self.drag = Some((y, self.value));
            return None;
        }
        let delta = (start_y - y) / self.distance * if shift { FINE_DRAG } else { 1. };
        let value = if y == start_y {
            // A click is not an adjustment. In particular, the first click of
            // a double-click must not snap an exact off-step reset target.
            self.range.clamp(start_value)
        } else {
            self.range.snap(
                self.range
                    .at_fraction(self.range.fraction(start_value) + delta),
            )
        };
        if value == self.value {
            return None;
        }
        self.value = value;
        self.change.clone().map(|change| (change, value))
    }
}

/// A numeric dial. Drag up to increase, down to decrease; Shift drags at a tenth of the speed.
/// Arrow keys step, Home and End select the limits, and a double-click resets. The owner
/// supplies `value` and accepts changes through `on_change`.
///
/// The arc takes the `text_color` and the track the `bg` set through [`Styled`]; both default to
/// the theme's knob colors.
#[derive(IntoElement)]
pub struct Knob {
    id: ElementId,
    accessibility_label: Option<SharedString>,
    value: f32,
    default_value: Option<f32>,
    min: f32,
    max: f32,
    step: f32,
    size: Size,
    diameter: Option<AbsoluteLength>,
    stroke_width: AbsoluteLength,
    bipolar: bool,
    bipolar_origin: Option<f32>,
    disabled: bool,
    readonly: bool,
    focusable: bool,
    show_value: bool,
    value_text: Option<SharedString>,
    value_width: Option<AbsoluteLength>,
    logarithmic: bool,
    wheel_step: Option<f32>,
    drag_distance: Option<AbsoluteLength>,
    change: Option<Change>,
    reset: Option<Change>,
    style: StyleRefinement,
}

impl Knob {
    /// Create a new knob with a range of 0 to 100 in steps of 1.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            accessibility_label: None,
            value: 0.,
            default_value: None,
            min: 0.,
            max: 100.,
            step: 1.,
            size: Size::Medium,
            diameter: None,
            stroke_width: rems(0.125).into(),
            bipolar: false,
            bipolar_origin: None,
            disabled: false,
            readonly: false,
            focusable: true,
            show_value: true,
            value_text: None,
            value_width: None,
            logarithmic: false,
            wheel_step: None,
            drag_distance: None,
            change: None,
            reset: None,
            style: StyleRefinement::default(),
        }
    }

    /// Name the control for assistive technology. Required: nearby text, such as a form field's
    /// label, is not read as its name.
    pub fn accessibility_label(mut self, value: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(value.into());
        self
    }

    /// Set the current value.
    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    /// Reset to this value on double-click. The target follows builder updates and is clamped
    /// to the current range without snapping to its step grid. If omitted, the first mounted,
    /// normalized controlled value is retained. Nonfinite defaults reset to the current range
    /// minimum.
    pub fn default_value(mut self, value: f32) -> Self {
        self.default_value = Some(value);
        self
    }

    /// Set the range of values.
    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.min = *range.start();
        self.max = *range.end();
        self
    }

    /// Set the step. Zero gives continuous dragging and 1% keyboard steps.
    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// Override the size presets. Values below 12 px are ignored.
    pub fn diameter(mut self, diameter: impl Into<AbsoluteLength>) -> Self {
        self.diameter = Some(diameter.into());
        self
    }

    /// Set the arc and pointer thickness without changing the dial diameter.
    pub fn stroke_width(mut self, width: impl Into<AbsoluteLength>) -> Self {
        self.stroke_width = width.into();
        self
    }

    /// Grow the value arc from the centre toward the value, on whichever side it lies, for a
    /// setting that leans either way, such as a pan or a pitch offset. The centre is zero when
    /// the range holds it, else the range's midpoint. Off by default, when the arc fills from
    /// the minimum. The pointer, the readout and the accessible value are the same either way.
    pub fn bipolar(mut self, bipolar: bool) -> Self {
        self.bipolar = bipolar;
        self
    }

    /// Grow the value arc from `origin` toward the value, for a setting whose rest position is
    /// not the range's centre. Turns [`Self::bipolar`] on.
    pub fn bipolar_origin(mut self, origin: f32) -> Self {
        self.bipolar = true;
        self.bipolar_origin = Some(origin);
        self
    }

    /// Read-only dials stay focusable so their value can be inspected.
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// Whether the dial takes keyboard focus, from a press or from Tab. On unless turned off.
    ///
    /// Turn it off for a dial where the keyboard belongs to something else, such as a transport
    /// bar beside a list: the arrow keys keep moving the list, and the dial still follows the
    /// pointer and the wheel.
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    /// Draw the value under the dial, as a small muted label. On unless turned off.
    pub fn show_value(mut self, show_value: bool) -> Self {
        self.show_value = show_value;
        self
    }

    /// What the value reads, such as "40%", instead of the number.
    pub fn value_text(mut self, text: impl Into<SharedString>) -> Self {
        self.value_text = Some(text.into());
        self
    }

    /// Reserve a stable caption column. The default is at least 4 rems and enough space for the
    /// complete numeric range. Use the widest expected custom format; longer text wraps without
    /// clipping. A caller's Styled width takes precedence and owns caption wrapping.
    pub fn value_width(mut self, width: impl Into<AbsoluteLength>) -> Self {
        self.value_width = Some(width.into());
        self
    }

    /// Place values geometrically along the dial, so equal turns multiply the value by equal
    /// amounts, as pitch and frequency are heard. The range must start above zero; otherwise the
    /// dial stays linear. Off by default.
    ///
    /// Positions are then distances in that geometric space: the arrow keys and the wheel
    /// multiply the value by e to the step, and a zero step is 1%. A step does not snap a drag
    /// to a grid of values.
    pub fn logarithmic(mut self, logarithmic: bool) -> Self {
        self.logarithmic = logarithmic;
        self
    }

    /// Let the mouse wheel move the value by this step per tick, up for more. Off by default, so
    /// a wheel scrolling a panel past a dial never changes it unasked. A zero or negative step
    /// turns it off. On a logarithmic dial the step is a distance in position, as for the arrow
    /// keys.
    pub fn wheel_step(mut self, step: f32) -> Self {
        self.wheel_step = (step.is_finite() && step > 0.).then_some(step);
        self
    }

    /// The vertical drag that sweeps the whole dial. 10 rems unless set.
    pub fn drag_distance(mut self, distance: impl Into<AbsoluteLength>) -> Self {
        self.drag_distance = Some(distance.into());
        self
    }

    /// Run this with each new value. Without it the dial is read-only.
    pub fn on_change(mut self, change: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.change = Some(Rc::new(change));
        self
    }

    /// Run this with the default value on a double-click, instead of `on_change`, for an owner
    /// that treats a reset differently from a drag, such as when recording undo steps. Without
    /// it a reset reports through `on_change`. The dial still needs an `on_change` to be
    /// adjustable.
    pub fn on_reset(mut self, reset: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.reset = Some(Rc::new(reset));
        self
    }
}

impl Disableable for Knob {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Sizable for Knob {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl Styled for Knob {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Knob {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let name = self.accessibility_label.clone().unwrap_or_default();
        debug_assert!(!name.trim().is_empty(), "Knob needs an accessibility label");
        let rem_size = window.rem_size();
        let range = Range::new(self.min, self.max, self.step).logarithmic(self.logarithmic);
        let value = range.clamp(self.value);
        let disabled = self.disabled || range.min == range.max;
        let enabled = !disabled && !self.readonly && self.change.is_some();
        let readonly = self.readonly || self.change.is_none();
        let preset = match self.size {
            Size::Size(size) => size,
            Size::XSmall => rems(1.25).to_pixels(rem_size),
            Size::Small => rems(1.5).to_pixels(rem_size),
            Size::Medium => rems(2.).to_pixels(rem_size),
            Size::Large => rems(2.5).to_pixels(rem_size),
        };
        let diameter = self
            .diameter
            .map(|diameter| diameter.to_pixels(rem_size))
            .filter(|diameter| *diameter >= px(12.))
            .unwrap_or(preset);
        let distance = self
            .drag_distance
            .map(|distance| distance.to_pixels(rem_size))
            .filter(|distance| *distance > px(0.))
            .unwrap_or(rem_size * DRAG_DISTANCE_REMS);
        let memory =
            window.use_keyed_state((self.id.clone(), "knob-memory"), cx, |_, _| KnobMemory {
                drag: None,
                fine: false,
                moved: false,
                reset_handler: None,
                range,
                value,
                mounted_default: value,
                reset_value: value,
                enabled,
                distance,
                change: None,
                caption_metrics: None,
            });
        memory.update(cx, |memory, _| {
            memory.range = range;
            memory.value = value;
            memory.reset_value = range.clamp(self.default_value.unwrap_or(memory.mounted_default));
            memory.enabled = enabled;
            memory.distance = distance;
            memory.change = self.change.clone();
            memory.reset_handler = self.reset.clone();
            if !enabled {
                memory.drag = None;
            }
        });
        let focus = window
            .use_keyed_state((self.id.clone(), "knob-focus"), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone();
        let press_memory = memory.clone();
        let wheel_memory = memory.clone();
        let press_focus = focus.clone();
        let increment = self.change.clone();
        let decrement = self.change.clone();
        let keyboard = self.change.clone();
        let wheel = self.change.clone();
        let wheel_step = self.wheel_step;
        let focusable = self.focusable;
        let value_text = self.value_text.unwrap_or_else(|| {
            format!("{value:.2}")
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_owned()
                .into()
        });
        let pointer_angle = sweep_angle(range.fraction(value));
        let origin = self
            .bipolar
            .then(|| self.bipolar_origin.unwrap_or_else(|| range.centre()));
        let (arc_start, arc_end) = value_arc(range, value, origin);
        let stroke = self.stroke_width.to_pixels(rem_size).min(diameter / 6.);

        // The caption is a label, which sets its own size and weight, so the column is measured
        // at those whatever text style the caller gives the knob. The rest, such as the font
        // family, still reaches the label.
        let mut caption_style = window.text_style();
        caption_style.refine(&self.style.text);
        caption_style.font_size = TextSize::Small.rems(cx).into();
        caption_style.font_weight = FontWeight::NORMAL;
        let width = if self.show_value {
            // Reserve from the range, never the live value. A current-caption
            // measurement changes both the dial center and adjacent controls.
            let cached_width = memory
                .read(cx)
                .caption_metrics
                .as_ref()
                .filter(|metrics| {
                    metrics.min == range.min
                        && metrics.max == range.max
                        && metrics.style == caption_style
                        && metrics.rem_size == rem_size
                })
                .map(|metrics| metrics.width);
            let numeric_width = cached_width.unwrap_or_else(|| {
                let width = {
                    let measure = |text: SharedString| {
                        window
                            .text_system()
                            .shape_line(
                                text.clone(),
                                caption_style.font_size.to_pixels(rem_size),
                                &[caption_style.to_run(text.len())],
                                None,
                            )
                            .width()
                    };
                    let digit_width = ('0'..='9')
                        .map(|digit| measure(digit.to_string().into()))
                        .fold(px(0.), Pixels::max);
                    // Default formatting has at most two fraction digits. Always
                    // reserve them, even when a range endpoint ends in zeroes.
                    let digits = [range.min, range.max]
                        .into_iter()
                        .map(|value| {
                            format!("{value:.2}")
                                .bytes()
                                .filter(u8::is_ascii_digit)
                                .count()
                        })
                        .max()
                        .unwrap_or(0);
                    let sign = if range.min < 0. {
                        measure("-".into())
                    } else {
                        px(0.)
                    };
                    (digit_width * digits as f32 + measure(".".into()) + sign).ceil()
                };
                memory.update(cx, |memory, _| {
                    memory.caption_metrics = Some(CaptionMetrics {
                        min: range.min,
                        max: range.max,
                        style: caption_style.clone(),
                        rem_size,
                        width,
                    })
                });
                width
            });
            let column = self
                .value_width
                .map(|width| width.to_pixels(rem_size))
                .filter(|width| *width > px(0.))
                .unwrap_or_else(|| rems(4.).to_pixels(rem_size));
            diameter.max(column).max(numeric_width)
        } else {
            diameter
        };

        let theme = cx.theme();
        let dimmed = theme.muted_foreground.opacity(0.5);
        let track = if disabled {
            theme.muted
        } else {
            self.style
                .background
                .clone()
                .and_then(|background| background.color())
                .and_then(|background| background.as_solid())
                .unwrap_or(theme.knob)
        };
        let arc = if disabled {
            dimmed
        } else {
            self.style.text.color.unwrap_or(theme.knob_value)
        };
        let pointer = if disabled {
            dimmed
        } else {
            theme.knob_foreground
        };
        let surface = theme.secondary;
        let caption_color = if disabled {
            dimmed
        } else {
            theme.muted_foreground
        };
        // Those two tint the artwork. Everything else in the style belongs to the root.
        self.style.background = None;
        self.style.text.color = None;

        let dial = canvas(
            |_, _, _| (),
            move |bounds: Bounds<Pixels>, _, window: &mut Window, _| {
                let center = bounds.center();
                let radius = bounds.size.width / 2. - stroke;
                let mut paint_arc = |start: f32, end: f32, color| {
                    if end <= start {
                        return;
                    }
                    let mut path = PathBuilder::stroke(stroke);
                    for index in 0..=48 {
                        let angle = (start + (end - start) * index as f32 / 48.).to_radians();
                        let point = center + point(radius * angle.cos(), radius * angle.sin());
                        if index == 0 {
                            path.move_to(point);
                        } else {
                            path.line_to(point);
                        }
                    }
                    if let Ok(path) = path.build() {
                        window.paint_path(path, color);
                    }
                };
                paint_arc(sweep_angle(0.), sweep_angle(1.), track);
                paint_arc(arc_start, arc_end, arc);
                let inner = bounds.inset(stroke * 3.);
                window.paint_quad(fill(inner, surface).corner_radii(inner.size.width / 2.));
                let angle = pointer_angle.to_radians();
                let mut indicator = PathBuilder::stroke(stroke);
                indicator.move_to(
                    center + point(radius * 0.35 * angle.cos(), radius * 0.35 * angle.sin()),
                );
                indicator.line_to(
                    center + point(radius * 0.68 * angle.cos(), radius * 0.68 * angle.sin()),
                );
                if let Ok(path) = indicator.build() {
                    window.paint_path(path, pointer);
                }

                // Listen at the window during a captured drag, including outside the dial.
                let move_memory = memory.clone();
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                    if phase != DispatchPhase::Capture {
                        return;
                    }
                    if event.pressed_button != Some(MouseButton::Left) {
                        move_memory.update(cx, |memory, _| memory.drag = None);
                        return;
                    }
                    let next = move_memory.update(cx, |memory, _| {
                        memory.update_pointer(event.position.y, event.modifiers.shift)
                    });
                    if let Some((change, value)) = next {
                        change(value, window, cx);
                        cx.stop_propagation();
                    }
                });
                let release_memory = memory.clone();
                window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                    if phase != DispatchPhase::Capture || event.button != MouseButton::Left {
                        return;
                    }
                    let next = release_memory.update(cx, |memory, _| {
                        let next = memory.update_pointer(event.position.y, event.modifiers.shift);
                        memory.drag = None;
                        next
                    });
                    if let Some((change, value)) = next {
                        change(value, window, cx);
                    }
                });
            },
        )
        .size(diameter);

        let show_focus = !disabled && focus.is_focused(window) && window.last_input_was_keyboard();
        let root = div()
            .id(self.id)
            .when(cfg!(test), |root| {
                root.debug_selector(|| "knob-control".into())
            })
            .role(Role::Slider)
            .aria_label(name)
            .aria_value(value_text.clone())
            .aria_numeric_value(value as f64)
            .aria_min_numeric_value(range.min as f64)
            .aria_max_numeric_value(range.max as f64)
            .aria_numeric_value_step(if range.step == 0. {
                (range.max as f64 - range.min as f64) / 100.
            } else {
                range.step as f64
            })
            .relative()
            .w(width)
            .flex()
            .flex_col()
            .items_center()
            .gap_1()
            .flex_shrink_0()
            .when(!disabled && focusable, |root| {
                root.track_focus(&focus.clone().tab_stop(true))
            })
            .when(enabled, |root| {
                root.cursor_pointer()
                    .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                        if focusable {
                            press_focus.focus(window, cx);
                        }
                        if event.click_count == 2 {
                            let reset = press_memory.update(cx, |memory, _| memory.reset());
                            if let Some((change, value)) = reset {
                                change(value, window, cx);
                            }
                        } else {
                            press_memory.update(cx, |memory, _| {
                                memory.drag = Some((event.position.y, memory.value));
                                memory.fine = event.modifiers.shift;
                                memory.moved = false;
                            });
                        }
                        // An unfocusable dial is transparent to the press, so an editor
                        // open nearby still loses focus and ends as it would for any child.
                        if focusable {
                            cx.stop_propagation();
                        }
                    })
                    .when_some(wheel_step, |root, step| {
                        root.on_scroll(move |event, window, cx| {
                            let dy = f32::from(event.delta.pixel_delta(px(1.)).y);
                            if dy.abs() < 0.01 {
                                return;
                            }
                            // Several ticks can land between frames, so each one steps
                            // from the latest value rather than the one last drawn.
                            let (current, next) = wheel_memory.update(cx, |memory, _| {
                                let current = memory.value;
                                memory.value = range.shifted(current, dy.signum(), step);
                                (current, memory.value)
                            });
                            if next != current
                                && let Some(change) = &wheel
                            {
                                change(next, window, cx);
                            }
                            cx.stop_propagation();
                        })
                    })
                    .on_a11y_action(AccessibleAction::Increment, move |_, window, cx| {
                        let next = range.next(value, 1.);
                        if next != value
                            && let Some(change) = &increment
                        {
                            change(next, window, cx);
                        }
                    })
                    .on_a11y_action(AccessibleAction::Decrement, move |_, window, cx| {
                        let next = range.next(value, -1.);
                        if next != value
                            && let Some(change) = &decrement
                        {
                            change(next, window, cx);
                        }
                    })
                    .on_key_down(move |event, window, cx| {
                        if event.keystroke.modifiers.modified() {
                            return;
                        }
                        let next = match event.keystroke.key.as_str() {
                            "up" | "right" => range.next(value, 1.),
                            "down" | "left" => range.next(value, -1.),
                            "home" => range.min,
                            "end" => range.max,
                            _ => return,
                        };
                        if next != value
                            && let Some(change) = &keyboard
                        {
                            change(next, window, cx);
                        }
                        cx.stop_propagation();
                    })
            })
            .child(
                div()
                    .when(cfg!(test), |dial| {
                        dial.debug_selector(|| "knob-dial".into())
                    })
                    .relative()
                    .size(diameter)
                    .flex_none()
                    .rounded_full()
                    .child(dial)
                    .when(show_focus, |dial| dial.focus_ring_style(window, cx)),
            )
            .when(self.show_value, |root| {
                root.child(
                    div()
                        .when(cfg!(test), |caption| {
                            caption.debug_selector(|| "knob-value".into())
                        })
                        .w_full()
                        .min_w_0()
                        .whitespace_normal()
                        .text_center()
                        .flex_none()
                        // A readout is supporting text, so it stays small and muted whatever
                        // the dial's size.
                        .child(
                            Label::new(value_text)
                                .text_ui_sm(cx)
                                .text_color(caption_color),
                        ),
                )
            })
            .refine_style(&self.style);
        Semantics {
            inner: root,
            disabled,
            readonly,
        }
    }
}

/// Reports the states GPUI's `aria_` builders have no setter for.
struct Semantics<E: Element> {
    inner: E,
    disabled: bool,
    readonly: bool,
}

impl<E: Element> IntoElement for Semantics<E> {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl<E: Element> Element for Semantics<E> {
    type RequestLayoutState = E::RequestLayoutState;
    type PrepaintState = E::PrepaintState;

    fn id(&self) -> Option<ElementId> {
        self.inner.id()
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        self.inner.source_location()
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.inner.request_layout(id, inspector, window, cx)
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.inner
            .prepaint(id, inspector, bounds, layout, window, cx)
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.inner
            .paint(id, inspector, bounds, layout, prepaint, window, cx)
    }

    fn a11y_role(&self) -> Option<accesskit::Role> {
        self.inner.a11y_role()
    }

    fn write_a11y_info(&self, node: &mut accesskit::Node) {
        self.inner.write_a11y_info(node);
        if self.disabled {
            node.set_disabled();
        }
        if self.readonly {
            node.set_read_only();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Knob, Range, value_arc};
    use crate::{Disableable as _, Sizable as _, Size};
    use gpui::{
        Context, Element as _, IntoElement, Modifiers, MouseButton, Pixels, Point, Render,
        RenderOnce as _, Role, ScrollDelta, ScrollEvent, TestAppContext, TouchPhase,
        VisualTestContext, Window, accesskit, div, point, prelude::*, px, size,
    };

    struct Probe {
        value: f32,
        disabled: bool,
        readonly: bool,
        changes: usize,
    }

    impl Render for Probe {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let view = cx.entity().downgrade();
            let knob = Knob::new("dial")
                .accessibility_label("Offset")
                .range(-20. ..=80.)
                .step(5.)
                .value(self.value)
                .disabled(self.disabled)
                .readonly(self.readonly)
                .on_change(move |value, _, cx| {
                    view.update(cx, |this, cx| {
                        this.value = value;
                        this.changes += 1;
                        cx.notify();
                    })
                    .unwrap()
                });
            let rendered = knob.render(window, cx).into_element();
            let mut node = accesskit::Node::new(Role::Slider);
            rendered.write_a11y_info(&mut node);
            assert_eq!(node.numeric_value(), Some(self.value as f64));
            assert_eq!(node.is_disabled(), self.disabled);
            assert_eq!(node.is_read_only(), self.readonly);
            assert_eq!(
                node.supports_action(accesskit::Action::Increment),
                !self.disabled && !self.readonly
            );
            div().p(px(220.)).child(rendered)
        }
    }

    fn probe(cx: &mut TestAppContext) -> (gpui::Entity<Probe>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, visual) = cx.add_window_view(|window, _| {
            window.activate_window();
            Probe {
                value: 30.,
                disabled: false,
                readonly: false,
                changes: 0,
            }
        });
        visual.simulate_resize(size(px(1400.), px(1400.)));
        draw(visual);
        (view, visual)
    }

    fn draw(visual: &mut VisualTestContext) {
        visual.update(|window, cx| window.draw(cx).clear(cx));
    }

    struct FocusProbe {
        focusable: bool,
        elsewhere: gpui::FocusHandle,
        value: f32,
        presses_around: usize,
    }

    impl Render for FocusProbe {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let view = cx.entity().downgrade();
            let around = cx.entity().downgrade();
            div()
                .p(px(220.))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    around
                        .update(cx, |this, _| this.presses_around += 1)
                        .unwrap();
                })
                .child(div().id("elsewhere").track_focus(&self.elsewhere))
                .child(
                    Knob::new("dial")
                        .accessibility_label("Offset")
                        .focusable(self.focusable)
                        .value(self.value)
                        .on_change(move |value, _, cx| {
                            view.update(cx, |this, cx| {
                                this.value = value;
                                cx.notify();
                            })
                            .unwrap()
                        }),
                )
        }
    }

    #[gpui::test]
    fn an_unfocusable_knob_still_drags_and_leaves_focus_where_it_was(cx: &mut TestAppContext) {
        cx.update(crate::init);
        for focusable in [true, false] {
            let (view, visual) = cx.add_window_view(|window, cx| {
                window.activate_window();
                FocusProbe {
                    focusable,
                    elsewhere: cx.focus_handle(),
                    value: 30.,
                    presses_around: 0,
                }
            });
            visual.simulate_resize(size(px(800.), px(800.)));
            visual.update(|window, cx| {
                let elsewhere = view.read(cx).elsewhere.clone();
                window.focus(&elsewhere, cx);
            });
            draw(visual);

            let start = visual.debug_bounds("knob-control").unwrap().center();
            visual.simulate_mouse_down(start, MouseButton::Left, Default::default());
            let dragged = start - point(px(0.), px(40.));
            visual.simulate_mouse_move(dragged, MouseButton::Left, Default::default());
            visual.simulate_mouse_up(dragged, MouseButton::Left, Default::default());
            draw(visual);

            visual.update(|window, cx| {
                let view = view.read(cx);
                assert!(
                    view.value != 30.,
                    "focusable={focusable}: the drag must apply"
                );
                assert_eq!(
                    view.elsewhere.is_focused(window),
                    !focusable,
                    "focusable={focusable}"
                );
                assert_eq!(
                    view.presses_around,
                    usize::from(!focusable),
                    "focusable={focusable}: only an unfocusable dial lets the press carry on"
                );
            });
        }
    }

    #[gpui::test]
    fn drag_continues_outside_the_dial_and_release_stops_changes(cx: &mut TestAppContext) {
        let (view, visual) = probe(cx);
        let start = visual.debug_bounds("knob-control").unwrap().center();
        visual.simulate_mouse_down(start, MouseButton::Left, Default::default());
        visual.simulate_mouse_move(
            start - point(px(0.), px(40.)),
            MouseButton::Left,
            Default::default(),
        );
        visual.update(|window, cx| {
            assert_eq!(view.read(cx).value, 55.);
            window.draw(cx).clear(cx);
        });
        let high = start - point(px(0.), px(200.));
        visual.simulate_mouse_up(high, MouseButton::Left, Default::default());
        visual.update(|window, cx| {
            assert_eq!(view.read(cx).value, 80.);
            window.draw(cx).clear(cx);
        });
        visual.simulate_mouse_move(start, None, Default::default());
        visual.update(|_, cx| assert_eq!(view.read(cx).value, 80.));
    }

    #[gpui::test]
    fn keys_step_and_select_the_limits(cx: &mut TestAppContext) {
        let (view, visual) = probe(cx);
        let start = visual.debug_bounds("knob-control").unwrap().center();
        visual.simulate_click(start, Default::default());
        draw(visual);
        for (key, expected) in [("right", 35.), ("left", 30.), ("home", -20.), ("end", 80.)] {
            visual.simulate_keystrokes(key);
            visual.update(|window, cx| {
                assert_eq!(view.read(cx).value, expected, "{key}");
                window.draw(cx).clear(cx);
            });
        }
    }

    #[gpui::test]
    fn readonly_and_disabled_drop_a_drag_in_progress(cx: &mut TestAppContext) {
        let (view, visual) = probe(cx);
        for disabled in [false, true] {
            let start = visual.debug_bounds("knob-control").unwrap().center();
            visual.simulate_mouse_down(start, MouseButton::Left, Default::default());
            view.update(visual, |view, cx| {
                view.disabled = disabled;
                view.readonly = !disabled;
                cx.notify();
            });
            draw(visual);
            let before = visual.update(|_, cx| view.read(cx).changes);
            visual.simulate_mouse_move(
                start - point(px(0.), px(40.)),
                MouseButton::Left,
                Default::default(),
            );
            visual.simulate_mouse_up(start, MouseButton::Left, Default::default());
            visual.update(|_, cx| assert_eq!(view.read(cx).changes, before));
            view.update(visual, |view, cx| {
                view.disabled = false;
                view.readonly = false;
                cx.notify();
            });
            draw(visual);
        }
    }

    #[test]
    fn ranges_reach_unaligned_endpoints_and_handle_invalid_input() {
        let range = Range::new(-3., 8., 4.);
        assert_eq!(range.snap(7.5), 8.);
        assert_eq!(range.next(8., -1.), 5.);
        assert_eq!(range.next(5., 1.), 8.);
        assert_eq!(range.clamp(f32::NAN), -3.);
        assert_eq!(Range::new(5., -3., f32::NAN).clamp(0.), 5.);
        let wide = Range::new(-f32::MAX, f32::MAX, 0.);
        assert_eq!(wide.next(f32::MAX, 1.), f32::MAX);
        assert_eq!(wide.next(-f32::MAX, -1.), -f32::MAX);
        let continuous = Range::new(-1., 1., 0.);
        assert!((continuous.next(0., 1.) - 0.02).abs() < 0.00001);
    }

    fn same_arc(drawn: (f32, f32), expected: (f32, f32)) -> bool {
        (drawn.0 - expected.0).abs() < 0.001 && (drawn.1 - expected.1).abs() < 0.001
    }

    #[test]
    fn a_bipolar_arc_grows_from_its_origin_and_the_default_fills_from_the_minimum() {
        let centred = |range: Range, value: f32| value_arc(range, value, Some(range.centre()));
        let pitch = Range::new(-12., 12., 1.);
        for (value, expected) in [
            (6., (270., 337.5)),
            (-6., (202.5, 270.)),
            (12., (270., 405.)),
            (-12., (135., 270.)),
            (0., (270., 270.)),
        ] {
            let drawn = centred(pitch, value);
            assert!(same_arc(drawn, expected), "{value}: {drawn:?}");
        }
        for (value, expected) in [
            (6., (135., 337.5)),
            (-6., (135., 202.5)),
            (-12., (135., 135.)),
        ] {
            let drawn = value_arc(pitch, value, None);
            assert!(same_arc(drawn, expected), "{value}: {drawn:?}");
        }
        // Zero off the middle of the range is still the centre: a fifth of the way along.
        let offset = Range::new(-20., 80., 1.);
        assert!(same_arc(centred(offset, 30.), (189., 270.)));
        // A range without zero grows from its midpoint, here 50.
        let level = Range::new(20., 80., 1.);
        assert!(same_arc(centred(level, 65.), (270., 337.5)));
        // A range that starts at zero grows from it, as the default does.
        let plain = Range::new(0., 100., 1.);
        assert!(same_arc(centred(plain, 50.), value_arc(plain, 50., None)));
        // An explicit origin overrides the centre.
        assert!(same_arc(
            value_arc(pitch, 6., Some(-12.)),
            value_arc(pitch, 6., None)
        ));
        // A value outside the range is drawn at its end.
        assert!(same_arc(centred(pitch, 40.), (270., 405.)));
    }

    #[test]
    fn logarithmic_position_is_geometric() {
        let range = Range::new(1., 100., 0.).logarithmic(true);
        assert!((range.fraction(10.) - 0.5).abs() < 1e-6);
        assert!((range.at_fraction(0.5) - 10.).abs() < 1e-4);
        assert_eq!(range.at_fraction(0.), 1.);
        assert_eq!(range.at_fraction(1.), 100.);
        assert!((Range::new(1., 100., 0.).fraction(50.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn logarithmic_keys_step_in_position() {
        let range = Range::new(1., 100., 0.1).logarithmic(true);
        assert!((range.next(10., 1.) - 10. * 0.1f32.exp()).abs() < 1e-3);
        assert!((range.next(10., -1.) - 10. * (-0.1f32).exp()).abs() < 1e-3);
        assert_eq!(range.next(100., 1.), 100.);
        assert_eq!(range.next(1., -1.), 1.);
        let continuous = Range::new(1., 100., 0.).logarithmic(true);
        assert!((continuous.next(10., 1.) - 10. * 0.01f32.exp()).abs() < 1e-3);
    }

    #[test]
    fn logarithmic_ignores_a_range_at_or_below_zero() {
        for (min, max) in [(0., 100.), (-5., 100.), (-10., -1.)] {
            let range = Range::new(min, max, 0.).logarithmic(true);
            let linear = Range::new(min, max, 0.);
            assert_eq!(range.fraction(max / 2.), linear.fraction(max / 2.));
            assert_eq!(range.at_fraction(0.5), linear.at_fraction(0.5));
        }
    }

    struct OptionProbe {
        value: f32,
        default: Option<f32>,
        min: f32,
        wheel_step: Option<f32>,
        log: bool,
        on_reset: bool,
        changes: Vec<f32>,
        resets: Vec<f32>,
    }

    impl OptionProbe {
        fn new() -> Self {
            Self {
                value: 30.,
                default: Some(0.),
                min: -20.,
                wheel_step: None,
                log: false,
                on_reset: false,
                changes: Vec::new(),
                resets: Vec::new(),
            }
        }
    }

    impl Render for OptionProbe {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let owner = cx.entity().downgrade();
            let reset_owner = owner.clone();
            let (min, max) = if self.log {
                (1., 1000.)
            } else {
                (self.min, 80.)
            };
            div().p(px(32.)).child(
                Knob::new("option-knob")
                    .accessibility_label("Offset")
                    .value(self.value)
                    .when_some(self.default, |knob, default| knob.default_value(default))
                    .range(min..=max)
                    .step(0.)
                    .logarithmic(self.log)
                    .with_size(Size::Medium)
                    .when_some(self.wheel_step, |knob, step| knob.wheel_step(step))
                    .on_change(move |value, _, cx| {
                        owner
                            .update(cx, |this, cx| {
                                this.value = value;
                                this.changes.push(value);
                                cx.notify();
                            })
                            .unwrap();
                    })
                    .when(self.on_reset, |knob| {
                        knob.on_reset(move |value, _, cx| {
                            reset_owner
                                .update(cx, |this, cx| {
                                    this.value = value;
                                    this.resets.push(value);
                                    cx.notify();
                                })
                                .unwrap();
                        })
                    }),
            )
        }
    }

    fn option_probe(
        cx: &mut TestAppContext,
        probe: OptionProbe,
    ) -> (gpui::Entity<OptionProbe>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, visual) = cx.add_window_view(|_, _| probe);
        visual.simulate_resize(size(px(700.), px(600.)));
        draw(visual);
        (view, visual)
    }

    fn wheel(visual: &mut VisualTestContext, at: Point<Pixels>, dy: f32) {
        visual.simulate_event(ScrollEvent {
            position: at,
            delta: ScrollDelta::Pixels(point(px(0.), px(dy))),
            touch_phase: TouchPhase::Moved,
            ..Default::default()
        });
    }

    fn double_press(visual: &mut VisualTestContext, at: Point<Pixels>) {
        visual.simulate_event(gpui::MouseDownEvent {
            position: at,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 2,
            first_mouse: false,
        });
    }

    #[gpui::test]
    fn wheel_ticks_between_frames_all_count(cx: &mut TestAppContext) {
        let (view, visual) = option_probe(cx, OptionProbe::new());
        let at = visual.debug_bounds("knob-dial").unwrap().center();
        view.update(visual, |view, cx| {
            view.wheel_step = Some(2.);
            cx.notify();
        });
        draw(visual);
        // Dispatched in one update, so no frame runs between the ticks.
        visual.update(|window, cx| {
            for _ in 0..3 {
                window.dispatch_event(
                    gpui::PlatformInput::Scroll(ScrollEvent {
                        position: at,
                        delta: ScrollDelta::Pixels(point(px(0.), px(10.))),
                        touch_phase: TouchPhase::Moved,
                        ..Default::default()
                    }),
                    cx,
                );
            }
        });
        visual.update(|_, cx| assert_eq!(view.read(cx).value, 36.));
    }

    #[gpui::test]
    fn a_press_that_barely_moves_is_a_click(cx: &mut TestAppContext) {
        let (view, visual) = option_probe(cx, OptionProbe::new());
        let start = visual.debug_bounds("knob-dial").unwrap().center();
        visual.simulate_mouse_down(start, MouseButton::Left, Default::default());
        let jitter = start - point(px(0.), px(2.));
        visual.simulate_mouse_move(jitter, MouseButton::Left, Default::default());
        visual.update(|_, cx| assert!(view.read(cx).changes.is_empty(), "within the threshold"));

        let drag = start - point(px(0.), px(8.));
        visual.simulate_mouse_move(drag, MouseButton::Left, Default::default());
        visual.simulate_mouse_up(drag, MouseButton::Left, Default::default());
        visual.update(|_, cx| assert!(!view.read(cx).changes.is_empty(), "past the threshold"));
    }

    #[gpui::test]
    fn the_wheel_changes_the_value_only_with_a_wheel_step(cx: &mut TestAppContext) {
        let (view, visual) = option_probe(cx, OptionProbe::new());
        let at = visual.debug_bounds("knob-dial").unwrap().center();
        wheel(visual, at, 10.);
        visual.update(|window, cx| {
            assert_eq!(view.read(cx).value, 30.);
            window.draw(cx).clear(cx);
        });
        view.update(visual, |view, cx| {
            view.wheel_step = Some(2.);
            cx.notify();
        });
        draw(visual);
        wheel(visual, at, 10.);
        visual.update(|window, cx| {
            assert_eq!(view.read(cx).value, 32.);
            window.draw(cx).clear(cx);
        });
        wheel(visual, at, -10.);
        visual.update(|_, cx| assert_eq!(view.read(cx).value, 30.));
    }

    #[gpui::test]
    fn the_wheel_steps_in_position_when_logarithmic(cx: &mut TestAppContext) {
        let (view, visual) = option_probe(
            cx,
            OptionProbe {
                log: true,
                value: 10.,
                default: Some(10.),
                wheel_step: Some(0.1),
                ..OptionProbe::new()
            },
        );
        let at = visual.debug_bounds("knob-dial").unwrap().center();
        wheel(visual, at, 10.);
        visual.update(|_, cx| {
            assert!((view.read(cx).value - 10. * 0.1f32.exp()).abs() < 1e-3);
        });
    }

    #[gpui::test]
    fn a_reset_runs_on_reset_and_not_on_change(cx: &mut TestAppContext) {
        let (view, visual) = option_probe(
            cx,
            OptionProbe {
                on_reset: true,
                ..OptionProbe::new()
            },
        );
        let at = visual.debug_bounds("knob-dial").unwrap().center();
        double_press(visual, at);
        visual.update(|_, cx| {
            assert_eq!(view.read(cx).resets, vec![0.]);
            assert!(view.read(cx).changes.is_empty());
            assert_eq!(view.read(cx).value, 0.);
        });
    }

    #[gpui::test]
    fn a_reset_falls_back_to_on_change(cx: &mut TestAppContext) {
        let (view, visual) = option_probe(cx, OptionProbe::new());
        let at = visual.debug_bounds("knob-dial").unwrap().center();
        double_press(visual, at);
        visual.update(|_, cx| {
            assert_eq!(view.read(cx).changes, vec![0.]);
            assert!(view.read(cx).resets.is_empty());
        });
    }

    #[gpui::test]
    fn a_double_click_restores_the_mounted_value_and_cancels_a_late_drag(cx: &mut TestAppContext) {
        let (view, visual) = option_probe(
            cx,
            OptionProbe {
                default: None,
                ..OptionProbe::new()
            },
        );
        view.update(visual, |view, cx| {
            view.value = 80.;
            cx.notify();
        });
        draw(visual);
        let start = visual.debug_bounds("knob-dial").unwrap().center();
        visual.simulate_mouse_down(start, MouseButton::Left, Default::default());
        visual.simulate_mouse_move(
            start + point(px(0.), px(20.)),
            MouseButton::Left,
            Default::default(),
        );
        draw(visual);
        let before = visual.update(|_, cx| view.read(cx).changes.len());
        double_press(visual, start);
        visual.update(|window, cx| {
            assert_eq!(view.read(cx).value, 30.);
            assert_eq!(view.read(cx).changes.len(), before + 1);
            window.draw(cx).clear(cx);
        });
        let late = start - point(px(0.), px(200.));
        visual.simulate_mouse_move(late, MouseButton::Left, Default::default());
        visual.simulate_mouse_up(late, MouseButton::Left, Default::default());
        visual.update(|_, cx| {
            assert_eq!(view.read(cx).value, 30., "the drag ended with the reset");
            assert_eq!(view.read(cx).changes.len(), before + 1);
        });
    }

    #[gpui::test]
    fn an_explicit_default_clamps_without_snapping(cx: &mut TestAppContext) {
        let (view, visual) = option_probe(cx, OptionProbe::new());
        for (default, min, expected) in [
            (Some(5.25), -20., 5.25),
            (Some(-50.), -20., -20.),
            (Some(-50.), -60., -50.),
            (Some(f32::NAN), -60., -60.),
        ] {
            view.update(visual, |view, cx| {
                view.value = 70.;
                view.default = default;
                view.min = min;
                cx.notify();
            });
            draw(visual);
            let at = visual.debug_bounds("knob-dial").unwrap().center();
            double_press(visual, at);
            visual.simulate_mouse_up(at, MouseButton::Left, Default::default());
            visual.update(|window, cx| {
                assert_eq!(view.read(cx).value, expected);
                window.draw(cx).clear(cx);
            });
        }
    }

    #[gpui::test]
    fn shift_drags_at_a_tenth_and_releasing_it_does_not_jump(cx: &mut TestAppContext) {
        let shift = Modifiers::shift();
        let (view, visual) = option_probe(cx, OptionProbe::new());
        let start = visual.debug_bounds("knob-dial").unwrap().center();
        // 40px of a 160px sweep over a span of 100 is 25 units; with Shift it is 2.5.
        visual.simulate_mouse_down(start, MouseButton::Left, shift);
        visual.simulate_mouse_move(start - point(px(0.), px(40.)), MouseButton::Left, shift);
        visual.update(|window, cx| {
            assert!((view.read(cx).value - 32.5).abs() < 1e-3);
            window.draw(cx).clear(cx);
        });
        visual.simulate_mouse_move(
            start - point(px(0.), px(40.)),
            MouseButton::Left,
            Default::default(),
        );
        visual.update(|_, cx| assert!((view.read(cx).value - 32.5).abs() < 1e-3));
        visual.simulate_mouse_move(
            start - point(px(0.), px(80.)),
            MouseButton::Left,
            Default::default(),
        );
        visual.update(|_, cx| assert!((view.read(cx).value - 57.5).abs() < 1e-3));
        visual.simulate_mouse_up(
            start - point(px(0.), px(80.)),
            MouseButton::Left,
            Default::default(),
        );
    }

    #[gpui::test]
    fn a_caption_reserves_its_column_from_the_range_not_the_value(cx: &mut TestAppContext) {
        let (view, visual) = option_probe(cx, OptionProbe::new());
        let control = visual.debug_bounds("knob-control").unwrap();
        let dial = visual.debug_bounds("knob-dial").unwrap();
        for value in [-20., 7.25, 80.] {
            view.update(visual, |view, cx| {
                view.value = value;
                cx.notify();
            });
            draw(visual);
            assert_eq!(visual.debug_bounds("knob-control").unwrap(), control);
            assert_eq!(visual.debug_bounds("knob-dial").unwrap(), dial);
        }
    }
}
