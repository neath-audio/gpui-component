use super::{Channel, ColorEditor, hsla_tracks, lightness_gradients, panel, plane_stops};
use crate::color_picker::{ColorPickerEvent, ColorPickerState};
use gpui::{
    AppContext as _, Bounds, Context, ElementId, Entity, Hsla, InteractiveElement as _,
    IntoElement, KeyBinding, MouseButton, ParentElement as _, Render, Rgba, Styled as _,
    Subscription, TestAppContext, VisualTestContext, Window, actions, div, hsla, point, px, size,
};
use std::{cell::Cell, rc::Rc};

fn editor(
    state: &Entity<ColorPickerState>,
    window: &mut Window,
    cx: &mut gpui::App,
) -> (ElementId, ColorEditor) {
    let id: ElementId = "spectrum-test".into();
    let editor = ColorEditor::new(&id, state, None, window, cx);
    (id, editor)
}

struct Probe {
    state: Entity<ColorPickerState>,
    disabled: bool,
    hsla: bool,
    changes: usize,
    _events: Subscription,
}

impl Render for Probe {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (id, editor) = editor(&self.state, window, cx);
        let root = div().p(px(24.)).w(px(340.));
        if self.hsla {
            return root.flex().flex_col().gap(px(8.)).children(hsla_tracks(
                id,
                editor,
                self.disabled,
                window,
                cx,
            ));
        }
        root.child(panel(id, editor, self.disabled, window, cx))
    }
}

fn setup(cx: &mut TestAppContext) -> (Entity<Probe>, &mut VisualTestContext) {
    cx.update(crate::init);
    let (view, visual) = cx.add_window_view(|window, cx| {
        window.activate_window();
        let state = cx.new(|cx| ColorPickerState::new(window, cx));
        state.update(cx, |state, cx| {
            state.set_value(hsla(0.6, 0.7, 0.4, 0.8), window, cx)
        });
        let events = cx.subscribe(&state, |this: &mut Probe, _, _: &ColorPickerEvent, cx| {
            this.changes += 1;
            cx.notify();
        });
        Probe {
            state,
            disabled: false,
            hsla: false,
            changes: 0,
            _events: events,
        }
    });
    visual.simulate_resize(size(px(700.), px(500.)));
    draw(visual);
    (view, visual)
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

fn value(view: &Entity<Probe>, cx: &mut VisualTestContext) -> Hsla {
    cx.update(|_, cx| view.read(cx).state.read(cx).value().unwrap())
}

fn near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.00001,
        "{actual} != {expected}"
    );
}

fn sample_gradient(stops: [Hsla; 2], position: f32) -> Rgba {
    let from = Rgba::from(stops[0]);
    let to = Rgba::from(stops[1]);
    let mix = |a: f32, b: f32| a + (b - a) * position;
    Rgba {
        r: mix(from.r, to.r),
        g: mix(from.g, to.g),
        b: mix(from.b, to.b),
        a: mix(from.a, to.a),
    }
}

#[test]
fn plane_layers_composite_to_exact_hsl() {
    for hue in (0..=12).map(|v| v as f32 / 12.) {
        let [base, upper, lower] = plane_stops(hue);
        for saturation in (0..=5).map(|v| v as f32 / 5.) {
            for lightness in (0..=10).map(|v| v as f32 / 10.) {
                let base = sample_gradient(base, saturation);
                let overlay = if lightness >= 0.5 {
                    sample_gradient(upper, 2. * (1. - lightness))
                } else {
                    sample_gradient(lower, 1. - 2. * lightness)
                };
                let expected = Rgba::from(hsla(hue, saturation, lightness, 1.));
                near(
                    base.r * (1. - overlay.a) + overlay.r * overlay.a,
                    expected.r,
                );
                near(
                    base.g * (1. - overlay.a) + overlay.g * overlay.a,
                    expected.g,
                );
                near(
                    base.b * (1. - overlay.a) + overlay.b * overlay.a,
                    expected.b,
                );
            }
        }
    }
}

#[test]
fn lightness_rail_meets_full_chroma_at_its_midpoint() {
    let color = hsla(0.3, 0.8, 0.2, 0.4);
    let midpoint = Hsla {
        l: 0.5,
        a: 1.,
        ..color
    };
    let [dark, light] = lightness_gradients(color);
    assert_eq!(
        dark,
        super::gradient(90., Hsla { l: 0., ..midpoint }, midpoint)
    );
    assert_eq!(
        light,
        super::gradient(90., midpoint, Hsla { l: 1., ..midpoint })
    );
}

#[gpui::test]
fn plane_drag_captures_outside_releases_and_deduplicates_while_preserving_hue(
    cx: &mut TestAppContext,
) {
    let (view, visual) = setup(cx);
    let plane = visual.debug_bounds("color-picker-plane").unwrap();
    let start = point(
        plane.left() + plane.size.width * 0.7,
        plane.top() + plane.size.height * 0.2,
    );
    visual.simulate_mouse_down(start, MouseButton::Left, Default::default());
    near(value(&view, visual).s, 0.7);
    near(value(&view, visual).l, 0.8);
    let outside = point(
        plane.left() + plane.size.width * 0.7,
        plane.bottom() + px(40.),
    );
    visual.simulate_mouse_move(outside, MouseButton::Left, Default::default());
    near(value(&view, visual).h, 0.6);
    near(value(&view, visual).s, 0.7);
    near(value(&view, visual).l, 0.);
    let changes = visual.update(|_, cx| view.read(cx).changes);
    visual.simulate_mouse_move(outside, MouseButton::Left, Default::default());
    visual.simulate_mouse_up(outside, MouseButton::Left, Default::default());
    visual.simulate_mouse_move(start, MouseButton::Left, Default::default());
    visual.update(|_, cx| assert_eq!(view.read(cx).changes, changes));
    draw(visual);
    visual.simulate_keystrokes("up shift-up left");
    near(value(&view, visual).l, 0.11);
    near(value(&view, visual).s, 0.69);
    near(value(&view, visual).h, 0.6);
    let white = point(start.x, plane.top() - px(30.));
    visual.simulate_mouse_down(start, MouseButton::Left, Default::default());
    visual.simulate_mouse_up(white, MouseButton::Left, Default::default());
    near(value(&view, visual).l, 1.);
    near(value(&view, visual).s, 0.7);
    near(value(&view, visual).h, 0.6);
}

#[gpui::test]
fn scalar_rails_keyboard_escape_and_disabled_cancel_capture(cx: &mut TestAppContext) {
    let (view, visual) = setup(cx);
    for (selector, channel) in [
        ("color-picker-hue", Channel::Hue),
        ("color-picker-alpha", Channel::Alpha),
    ] {
        let rail = visual.debug_bounds(selector).unwrap();
        visual.simulate_click(rail.center(), Default::default());
        near(channel.value(value(&view, visual)), 0.5);
        visual.simulate_keystrokes("home");
        near(channel.value(value(&view, visual)), 0.);
        visual.simulate_keystrokes("end");
        near(channel.value(value(&view, visual)), 1.);
        visual.simulate_mouse_down(rail.center(), MouseButton::Left, Default::default());
        visual.simulate_keystrokes("escape");
        let changes = visual.update(|_, cx| view.read(cx).changes);
        let away = point(rail.right() + px(50.), rail.center().y);
        visual.simulate_mouse_move(away, MouseButton::Left, Default::default());
        visual.simulate_mouse_up(away, MouseButton::Left, Default::default());
        visual.update(|_, cx| assert_eq!(view.read(cx).changes, changes));
        visual.simulate_mouse_down(rail.center(), MouseButton::Left, Default::default());
        visual.update(|_, cx| {
            view.update(cx, |view, cx| {
                view.disabled = true;
                cx.notify();
            })
        });
        draw(visual);
        let color = value(&view, visual);
        visual.simulate_mouse_move(away, MouseButton::Left, Default::default());
        visual.simulate_mouse_up(away, MouseButton::Left, Default::default());
        visual.simulate_keystrokes("home right");
        assert_eq!(value(&view, visual), color);
        visual.update(|_, cx| {
            view.update(cx, |view, cx| {
                view.disabled = false;
                cx.notify();
            })
        });
        draw(visual);
    }
}

actions!(color_picker_test, [HostEscape]);

/// A host that binds Escape around the spectrum, as an app binds it at its root to
/// clear its selection, and counts each time that binding runs.
struct AncestorSpectrum {
    state: Entity<ColorPickerState>,
    runs: Rc<Cell<usize>>,
}

impl Render for AncestorSpectrum {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (id, editor) = editor(&self.state, window, cx);
        let runs = self.runs.clone();
        div()
            .key_context("SpectrumHost")
            .on_action(move |_: &HostEscape, _, _| runs.set(runs.get() + 1))
            .p(px(24.))
            .w(px(340.))
            .child(panel(id, editor, false, window, cx))
    }
}

#[gpui::test]
fn escape_cancels_a_drag_under_an_ancestor_binding(cx: &mut TestAppContext) {
    cx.update(|cx| {
        crate::init(cx);
        cx.bind_keys([KeyBinding::new("escape", HostEscape, Some("SpectrumHost"))]);
    });
    let runs = Rc::new(Cell::new(0));
    let host_runs = runs.clone();
    let (view, visual) = cx.add_window_view(move |window, cx| {
        window.activate_window();
        let state = cx.new(|cx| ColorPickerState::new(window, cx));
        state.update(cx, |state, cx| {
            state.set_value(hsla(0.6, 0.7, 0.4, 0.8), window, cx)
        });
        AncestorSpectrum {
            state,
            runs: host_runs,
        }
    });
    visual.simulate_resize(size(px(700.), px(500.)));
    draw(visual);
    let value = |visual: &mut VisualTestContext| {
        visual.update(|_, cx| view.read(cx).state.read(cx).value().unwrap())
    };

    // The press focuses the hue rail and starts a drag there. Escape ends it, so
    // moving on and releasing changes nothing.
    let rail = visual.debug_bounds("color-picker-hue").unwrap();
    visual.simulate_mouse_down(rail.center(), MouseButton::Left, Default::default());
    let pressed = value(visual);
    visual.simulate_keystrokes("escape");
    let away = point(rail.right() + px(50.), rail.center().y);
    visual.simulate_mouse_move(away, MouseButton::Left, Default::default());
    visual.simulate_mouse_up(away, MouseButton::Left, Default::default());
    assert_eq!(value(visual), pressed, "Escape ends the drag");
    assert_eq!(runs.get(), 0, "the host's binding does not run");

    visual.simulate_keystrokes("escape");
    assert_eq!(runs.get(), 1, "with no drag, Escape is the host's");
}

#[test]
fn coordinate_mapping_clamps_all_channels_without_rgb_round_trips() {
    let bounds = Bounds::new(point(px(10.), px(20.)), size(px(200.), px(100.)));
    let color = hsla(0.43, 0.72, 0.5, 0.6);
    let black = Channel::Plane.pointer(color, bounds, point(px(154.), px(300.)));
    near(black.s, 0.72);
    near(black.h, 0.43);
    near(black.a, 0.6);
    near(black.l, 0.);
    assert_eq!(
        Channel::Hue.pointer(color, bounds, point(px(-20.), px(0.))),
        Hsla { h: 0., ..color }
    );
    assert_eq!(
        Channel::Alpha.pointer(color, bounds, point(px(999.), px(0.))),
        Hsla { a: 1., ..color }
    );
}

#[gpui::test]
fn changing_mode_or_closing_and_reopening_cannot_resume_an_old_drag(cx: &mut TestAppContext) {
    let (view, visual) = setup(cx);
    visual.update(|_, cx| {
        let state = view.read(cx).state.clone();
        state.update(cx, |state, cx| state.set_open(true, cx));
    });
    for (tab, open) in [(1, true), (0, true), (0, false), (0, true)] {
        let plane = visual.debug_bounds("color-picker-plane").unwrap();
        visual.simulate_mouse_down(plane.center(), MouseButton::Left, Default::default());
        visual.update(|_, cx| {
            let state = view.read(cx).state.clone();
            state.update(cx, |state, cx| {
                state.set_active_tab(tab, cx);
                state.set_open(open, cx);
            });
        });
        draw(visual);
        let color = value(&view, visual);
        let end = point(plane.right() + px(40.), plane.bottom() + px(40.));
        visual.simulate_mouse_move(end, MouseButton::Left, Default::default());
        visual.simulate_mouse_up(end, MouseButton::Left, Default::default());
        assert_eq!(value(&view, visual), color);
    }
}

#[gpui::test]
fn hsla_rails_adjust_each_channel_independently(cx: &mut TestAppContext) {
    let (view, visual) = setup(cx);
    visual.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.hsla = true;
            view.state
                .update(cx, |state, cx| state.set_active_tab(2, cx));
            cx.notify();
        });
    });
    draw(visual);
    for channel in [
        Channel::Hue,
        Channel::Saturation,
        Channel::Lightness,
        Channel::Alpha,
    ] {
        let before = value(&view, visual);
        let rail = visual.debug_bounds(channel.selector()).unwrap();
        let target = point(rail.left() + rail.size.width * 0.35, rail.center().y);
        visual.simulate_click(target, Default::default());
        let expected = channel.pointer(before, rail, target);
        assert_eq!(value(&view, visual), expected);
        visual.simulate_keystrokes("right");
        assert_eq!(
            value(&view, visual),
            channel.key(expected, "right", false).unwrap()
        );
        visual.simulate_keystrokes("home end");
        near(channel.value(value(&view, visual)), 1.);
        draw(visual);
    }
}
