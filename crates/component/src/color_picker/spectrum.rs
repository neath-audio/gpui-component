//! Native, exact HSL gradients with retained pointer capture.

use gpui::{
    AccessibleAction, AnyElement, App, Background, BorderStyle, Bounds, ColorSpace, CursorStyle,
    DispatchPhase, ElementId, Entity, Hsla, InteractiveElement as _, IntoElement, MouseButton,
    MouseMoveEvent, MouseUpEvent, ParentElement as _, Pixels, Point, Role,
    StatefulInteractiveElement as _, Styled as _, Subscription, Window, canvas, checkerboard, div,
    fill, hsla, linear_color_stop, linear_gradient, outline, point, prelude::FluentBuilder as _,
    px, rems, size,
};

use super::{ColorEditor, ColorPickerState, focus_outline};
use crate::{ActiveTheme as _, actions::Cancel, v_flex};

/// The key context of each rail and of the plane, where Escape is bound to
/// [`Cancel`] so it ends a drag before any binding a host makes around the picker.
pub(super) const KEY_CONTEXT: &str = "ColorPickerSpectrum";

fn gradient(angle: f32, from: Hsla, to: Hsla) -> Background {
    linear_gradient(
        angle,
        linear_color_stop(from, 0.),
        linear_color_stop(to, 1.),
    )
    .color_space(ColorSpace::Srgb)
}

/// Saturation at L=.5, followed by white and black overlays over each half.
/// Interpolating in sRGB exactly follows HSL, including its neutral axis.
fn plane_stops(hue: f32) -> [[Hsla; 2]; 3] {
    [
        [hsla(hue, 0., 0.5, 1.), hsla(hue, 1., 0.5, 1.)],
        [hsla(0., 0., 1., 1.), hsla(0., 0., 1., 0.)],
        [hsla(0., 0., 0., 0.), hsla(0., 0., 0., 1.)],
    ]
}

fn plane_gradients(hue: f32) -> [Background; 3] {
    let [base, top, bottom] = plane_stops(hue);
    [
        gradient(90., base[0], base[1]),
        gradient(180., top[0], top[1]),
        gradient(180., bottom[0], bottom[1]),
    ]
}

fn hue_gradients() -> [Background; 6] {
    std::array::from_fn(|index| {
        gradient(
            90.,
            hsla(index as f32 / 6., 1., 0.5, 1.),
            hsla((index + 1) as f32 / 6., 1., 0.5, 1.),
        )
    })
}

fn alpha_gradient(color: Hsla) -> Background {
    gradient(90., Hsla { a: 0., ..color }, Hsla { a: 1., ..color })
}

fn saturation_gradient(color: Hsla) -> Background {
    gradient(
        90.,
        Hsla {
            s: 0.,
            a: 1.,
            ..color
        },
        Hsla {
            s: 1.,
            a: 1.,
            ..color
        },
    )
}

fn lightness_gradients(color: Hsla) -> [Background; 2] {
    let middle = Hsla {
        l: 0.5,
        a: 1.,
        ..color
    };
    [
        gradient(90., Hsla { l: 0., ..middle }, middle),
        gradient(90., middle, Hsla { l: 1., ..middle }),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Channel {
    Plane,
    Hue,
    Alpha,
    Saturation,
    Lightness,
}

impl Channel {
    fn part(self) -> &'static str {
        match self {
            Self::Plane => "plane",
            Self::Hue => "hue",
            Self::Alpha => "alpha",
            Self::Saturation => "saturation",
            Self::Lightness => "lightness",
        }
    }

    fn label(self) -> SharedLabel {
        match self {
            Self::Plane => rust_i18n::t!("ColorPicker.Plane"),
            Self::Hue => rust_i18n::t!("ColorPicker.Hue"),
            Self::Alpha => rust_i18n::t!("ColorPicker.Alpha"),
            Self::Saturation => rust_i18n::t!("ColorPicker.Saturation"),
            Self::Lightness => rust_i18n::t!("ColorPicker.Lightness"),
        }
    }

    fn selector(self) -> &'static str {
        match self {
            Self::Plane => "color-picker-plane",
            Self::Hue => "color-picker-hue",
            Self::Alpha => "color-picker-alpha",
            Self::Saturation => "color-picker-saturation",
            Self::Lightness => "color-picker-lightness",
        }
    }

    fn index(self) -> usize {
        self as usize
    }

    fn value(self, color: Hsla) -> f32 {
        match self {
            Self::Plane | Self::Saturation => color.s,
            Self::Hue => color.h,
            Self::Alpha => color.a,
            Self::Lightness => color.l,
        }
    }

    fn description(self, color: Hsla) -> String {
        match self {
            Self::Plane => format!(
                "{:.0}% saturation, {:.0}% lightness",
                color.s * 100.,
                color.l * 100.
            ),
            Self::Hue => format!("{:.0}°", color.h * 360.),
            Self::Alpha => format!("{:.0}%", color.a * 100.),
            Self::Saturation => format!("{:.0}%", color.s * 100.),
            Self::Lightness => format!("{:.0}%", color.l * 100.),
        }
    }

    fn pointer(self, color: Hsla, bounds: Bounds<Pixels>, position: Point<Pixels>) -> Hsla {
        let x = ((position.x - bounds.left()) / bounds.size.width.max(px(1.))).clamp(0., 1.);
        match self {
            Self::Plane => Hsla {
                s: x,
                l: (1. - (position.y - bounds.top()) / bounds.size.height.max(px(1.)))
                    .clamp(0., 1.),
                ..color
            },
            Self::Hue => Hsla { h: x, ..color },
            Self::Alpha => Hsla { a: x, ..color },
            Self::Saturation => Hsla { s: x, ..color },
            Self::Lightness => Hsla { l: x, ..color },
        }
    }

    fn key(self, color: Hsla, key: &str, large: bool) -> Option<Hsla> {
        let step = if large {
            0.1
        } else if self == Self::Hue {
            1. / 360.
        } else {
            0.01
        };
        let mut next = color;
        let value = match self {
            Self::Plane | Self::Saturation => &mut next.s,
            Self::Hue => &mut next.h,
            Self::Alpha => &mut next.a,
            Self::Lightness => &mut next.l,
        };
        match key {
            "home" => *value = 0.,
            "end" => *value = 1.,
            "left" => *value = (*value - step).max(0.),
            "right" => *value = (*value + step).min(1.),
            "up" if self == Self::Plane => next.l = (next.l + step).min(1.),
            "down" if self == Self::Plane => next.l = (next.l - step).max(0.),
            "up" => *value = (*value + step).min(1.),
            "down" => *value = (*value - step).max(0.),
            _ => return None,
        }
        Some(next)
    }
}

type SharedLabel = std::borrow::Cow<'static, str>;

#[derive(Clone, Copy)]
struct Drag {
    channel: Channel,
    bounds: Bounds<Pixels>,
}

struct Memory {
    enabled: bool,
    bounds: [Bounds<Pixels>; 5],
    drag: Option<Drag>,
    lifecycle: (usize, bool),
    _lifecycle: Subscription,
    _activation: Subscription,
}

fn lifecycle(state: &ColorPickerState) -> (usize, bool) {
    (state.active_tab(), state.is_open())
}

fn pointer(
    memory: &Entity<Memory>,
    editor: &ColorEditor,
    position: Point<Pixels>,
    window: &mut Window,
    cx: &mut App,
) -> bool {
    let retained = memory.read(cx);
    let Some(drag) = retained.drag.filter(|_| retained.enabled) else {
        return false;
    };
    let next = drag
        .channel
        .pointer(editor.color(cx), drag.bounds, position);
    editor.commit(next, window, cx);
    true
}

struct Controls {
    id: ElementId,
    editor: ColorEditor,
    memory: Entity<Memory>,
    color: Hsla,
    disabled: bool,
    capture_channel: Channel,
}

fn controls(
    id: ElementId,
    editor: ColorEditor,
    disabled: bool,
    capture_channel: Channel,
    window: &mut Window,
    cx: &mut App,
) -> Controls {
    let state = editor.state().clone();
    let memory = window.use_keyed_state(
        (
            id.clone(),
            format!("spectrum-memory-{}", state.entity_id().as_u64()),
        ),
        cx,
        |window, cx| Memory {
            enabled: !disabled,
            bounds: [Bounds::default(); 5],
            drag: None,
            lifecycle: lifecycle(state.read(cx)),
            _lifecycle: cx.observe(&state, |memory: &mut Memory, state, cx| {
                let lifecycle = lifecycle(state.read(cx));
                if lifecycle != memory.lifecycle {
                    memory.drag = None;
                    memory.lifecycle = lifecycle;
                }
            }),
            _activation: cx.observe_window_activation(window, |memory, window, _| {
                if !window.is_window_active() {
                    memory.drag = None;
                }
            }),
        },
    );
    memory.update(cx, |memory, _| {
        memory.enabled = !disabled;
        if disabled {
            memory.drag = None;
        }
    });
    Controls {
        id,
        color: editor.color(cx),
        editor,
        memory,
        disabled,
        capture_channel,
    }
}

pub(super) fn panel(
    id: ElementId,
    editor: ColorEditor,
    disabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let controls = controls(id.clone(), editor, disabled, Channel::Plane, window, cx);
    v_flex()
        .id((id, "spectrum"))
        .debug_selector(|| "color-picker-spectrum".into())
        .w_full()
        .min_w_0()
        .gap_2()
        .children(
            [Channel::Plane, Channel::Hue, Channel::Alpha]
                .map(|channel| control(channel, &controls, window, cx)),
        )
        .into_any_element()
}

/// Builds only the four rails used by the mounted HSLA view. The gradient painter
/// and captured interaction are shared with the spectrum.
pub(super) fn hsla_tracks(
    id: ElementId,
    editor: ColorEditor,
    disabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> [AnyElement; 4] {
    let controls = controls(id, editor, disabled, Channel::Hue, window, cx);
    [
        Channel::Hue,
        Channel::Saturation,
        Channel::Lightness,
        Channel::Alpha,
    ]
    .map(|channel| control(channel, &controls, window, cx))
}

fn control(channel: Channel, controls: &Controls, window: &mut Window, cx: &mut App) -> AnyElement {
    let id: ElementId = (controls.id.clone(), channel.part()).into();
    let focus = window
        .use_keyed_state((id.clone(), "focus"), cx, |_, cx| {
            cx.focus_handle().tab_stop(true)
        })
        .read(cx)
        .clone();
    let editor = controls.editor.clone();
    let memory = controls.memory.clone();
    let press_memory = memory.clone();
    let press_editor = editor.clone();
    let press_focus = focus.clone();
    let cancel_memory = memory.clone();
    let keyboard_editor = editor.clone();
    let increment = editor.clone();
    let decrement = editor.clone();
    let color = controls.color;
    let disabled = controls.disabled;
    let capture_channel = controls.capture_channel;
    let round_marker = capture_channel == Channel::Hue;
    let theme = cx.theme();
    let (checker_base, checker_color, border, focus_ring) =
        (theme.muted, theme.border, theme.border, theme.ring);
    // Two contrasting strokes stay visible over every color. They are not theme
    // chrome: they sit on the color data the rail paints.
    let (marker_dark, marker_light) = if disabled {
        (theme.border, theme.muted_foreground)
    } else {
        (hsla(0., 0., 0., 0.8), hsla(0., 0., 1., 1.))
    };
    let rem = window.rem_size();

    let graphic = canvas(
        |_, _, _| (),
        move |bounds, _, window, cx| {
            memory.update(cx, |memory, _| memory.bounds[channel.index()] = bounds);
            match channel {
                Channel::Plane => {
                    let [base, top, bottom] = plane_gradients(color.h);
                    let half = size(bounds.size.width, bounds.size.height / 2.);
                    window.paint_quad(fill(bounds, base));
                    window.paint_quad(fill(Bounds::new(bounds.origin, half), top));
                    window.paint_quad(fill(
                        Bounds::new(point(bounds.left(), bounds.top() + half.height), half),
                        bottom,
                    ));
                }
                Channel::Hue => {
                    for (index, gradient) in hue_gradients().into_iter().enumerate() {
                        let left = bounds.left() + bounds.size.width * (index as f32 / 6.);
                        let right = bounds.left() + bounds.size.width * ((index + 1) as f32 / 6.);
                        window.paint_quad(fill(
                            Bounds::new(
                                point(left, bounds.top()),
                                size(right - left, bounds.size.height),
                            ),
                            gradient,
                        ));
                    }
                }
                Channel::Alpha => {
                    window.paint_quad(fill(bounds, checker_base));
                    window.paint_quad(fill(
                        bounds,
                        checkerboard(checker_color, f32::from(rem) / 4.),
                    ));
                    window.paint_quad(fill(bounds, alpha_gradient(color)));
                }
                Channel::Saturation => {
                    window.paint_quad(fill(bounds, saturation_gradient(color)));
                }
                Channel::Lightness => {
                    for (index, gradient) in lightness_gradients(color).into_iter().enumerate() {
                        window.paint_quad(fill(
                            Bounds::new(
                                point(
                                    bounds.left() + bounds.size.width * (index as f32 / 2.),
                                    bounds.top(),
                                ),
                                size(bounds.size.width / 2., bounds.size.height),
                            ),
                            gradient,
                        ));
                    }
                }
            }
            let radius = (rem * if round_marker { 0.4375 } else { 0.3125 })
                .min(bounds.size.width / 2.)
                .min(bounds.size.height / 2.);
            let mut center = point(
                bounds.left() + bounds.size.width * channel.value(color),
                if channel == Channel::Plane {
                    bounds.top() + bounds.size.height * (1. - color.l)
                } else {
                    bounds.center().y
                },
            );
            center.x = center
                .x
                .clamp(bounds.left() + radius, bounds.right() - radius);
            center.y = center
                .y
                .clamp(bounds.top() + radius, bounds.bottom() - radius);
            let marker = Bounds::new(
                center - point(radius, radius),
                size(radius * 2., radius * 2.),
            );
            if round_marker {
                window.paint_quad(fill(marker, marker_light).corner_radii(radius));
            }
            window
                .paint_quad(outline(marker, marker_dark, BorderStyle::Solid).corner_radii(radius));
            window.paint_quad(
                outline(marker.inset(px(1.)), marker_light, BorderStyle::Solid)
                    .corner_radii(radius),
            );
            window.paint_quad(outline(bounds, border, BorderStyle::Solid));
            // One captured listener pair serves the whole spectrum. It keeps a drag
            // going outside the active rail and cancels it when the button is lost.
            if channel == capture_channel {
                let moves = memory.clone();
                let move_editor = editor.clone();
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                    if phase != DispatchPhase::Capture {
                        return;
                    }
                    if event.pressed_button != Some(MouseButton::Left) {
                        moves.update(cx, |memory, _| memory.drag = None);
                        return;
                    }
                    if pointer(&moves, &move_editor, event.position, window, cx) {
                        cx.stop_propagation();
                    }
                });
                let release = memory.clone();
                let release_editor = editor.clone();
                window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                    if phase != DispatchPhase::Capture || event.button != MouseButton::Left {
                        return;
                    }
                    pointer(&release, &release_editor, event.position, window, cx);
                    release.update(cx, |memory, _| memory.drag = None);
                });
            }
        },
    )
    .size_full();

    let maximum = if channel == Channel::Hue { 360. } else { 100. };
    div()
        .id(id)
        .debug_selector(move || channel.selector().into())
        .role(Role::Slider)
        .aria_label(channel.label())
        .aria_value(channel.description(color))
        .aria_numeric_value((channel.value(color) * maximum) as f64)
        .aria_min_numeric_value(0.)
        .aria_max_numeric_value(maximum as f64)
        .aria_numeric_value_step(1.)
        .w_full()
        .min_w_0()
        .relative()
        .map(|this| {
            if channel == Channel::Plane {
                this.h(rems(9.))
            } else {
                this.h_4()
            }
        })
        .when(!disabled, |this| {
            this.key_context(KEY_CONTEXT)
                .track_focus(&focus)
                .cursor(CursorStyle::Crosshair)
                .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                    press_focus.focus(window, cx);
                    press_memory.update(cx, |memory, _| {
                        memory.drag = Some(Drag {
                            channel,
                            bounds: memory.bounds[channel.index()],
                        })
                    });
                    pointer(&press_memory, &press_editor, event.position, window, cx);
                    cx.stop_propagation();
                })
                .on_key_down(move |event, window, cx| {
                    let modifiers = event.keystroke.modifiers;
                    if modifiers.control
                        || modifiers.platform
                        || modifiers.alt
                        || modifiers.function
                    {
                        return;
                    }
                    if let Some(next) = channel.key(
                        keyboard_editor.color(cx),
                        &event.keystroke.key,
                        modifiers.shift,
                    ) {
                        keyboard_editor.commit(next, window, cx);
                        cx.stop_propagation();
                    }
                })
                // Escape ends a drag and leaves the color where the drag took it.
                // With no drag it passes on, to the popover or to the host.
                .on_action(move |_: &Cancel, _, cx| {
                    if !cancel_memory.update(cx, |memory, _| memory.drag.take().is_some()) {
                        cx.propagate();
                    }
                })
                .on_a11y_action(AccessibleAction::Increment, move |_, window, cx| {
                    if let Some(next) = channel.key(increment.color(cx), "right", false) {
                        increment.commit(next, window, cx);
                    }
                })
                .on_a11y_action(AccessibleAction::Decrement, move |_, window, cx| {
                    if let Some(next) = channel.key(decrement.color(cx), "left", false) {
                        decrement.commit(next, window, cx);
                    }
                })
        })
        .child(graphic)
        .child(focus_outline(focus, focus_ring, px(0.), px(2.)))
        .into_any_element()
}

#[cfg(test)]
mod tests;
