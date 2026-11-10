use gpui::{
    AppContext as _, Context, MouseButton, Render, TestAppContext, VisualTestContext, div, hsla,
    point, px, size,
};

use super::*;

struct Probe {
    state: Entity<ColorPickerState>,
    inline: bool,
    disabled: bool,
    mode: ColorPickerMode,
    swatches: Option<Vec<ColorSwatch>>,
    changes: Vec<Option<Hsla>>,
    /// Stores each committed color as hex and hands it back, as a theme editor does.
    echo: bool,
    _events: Subscription,
}

impl Render for Probe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(320.)).child(
            ColorPicker::new(&self.state)
                .inline(self.inline)
                .disabled(self.disabled)
                .default_mode(self.mode)
                .when_some(self.swatches.clone(), |this, swatches| {
                    this.swatches(swatches)
                }),
        )
    }
}

fn setup(
    cx: &mut TestAppContext,
    initial: impl Into<Hsla>,
    configure: impl FnOnce(&mut Probe),
) -> (Entity<Probe>, &mut VisualTestContext) {
    cx.update(crate::init);
    let initial = initial.into();
    let (view, visual) = cx.add_window_view(move |window, cx| {
        window.activate_window();
        let state = cx.new(|cx| ColorPickerState::new(window, cx).default_value(initial));
        let events = cx.subscribe_in(
            &state,
            window,
            |this: &mut Probe, state, event: &ColorPickerEvent, window, cx| {
                let ColorPickerEvent::Change(color) = event;
                this.changes.push(*color);
                if let (true, Some(color)) = (this.echo, color) {
                    let stored = Hsla::parse_hex(&color.to_hex()).unwrap();
                    state.update(cx, |state, cx| state.set_value(stored, window, cx));
                }
                cx.notify();
            },
        );
        let mut probe = Probe {
            state,
            inline: true,
            disabled: false,
            mode: ColorPickerMode::Spectrum,
            swatches: None,
            changes: Vec::new(),
            echo: false,
            _events: events,
        };
        configure(&mut probe);
        probe
    });
    visual.simulate_resize(size(px(700.), px(600.)));
    draw(visual);
    (view, visual)
}

fn draw(visual: &mut VisualTestContext) {
    visual.update(|window, cx| window.draw(cx).clear(cx));
}

fn model(view: &Entity<Probe>, visual: &mut VisualTestContext) -> (Option<Hsla>, Option<Hsla>) {
    visual.update(|_, cx| {
        let state = view.read(cx).state.read(cx);
        (state.value(), state.displayed_color())
    })
}

fn focus_hex(view: &Entity<Probe>, visual: &mut VisualTestContext) {
    visual.update(|window, cx| {
        let input = view.read(cx).state.read(cx).hex_input().clone();
        input.update(cx, |input, cx| {
            input.focus(window, cx);
            input.select_all(window, cx);
        });
    });
}

fn open_popup(visual: &mut VisualTestContext) {
    let trigger = visual.debug_bounds("color-picker-trigger").unwrap();
    visual.simulate_click(trigger.center(), Default::default());
    draw(visual);
}

fn swatch_selector(color: Hsla) -> &'static str {
    Box::leak(format!("color-picker-swatch-{}", color.to_hex()).into_boxed_str())
}

#[gpui::test]
fn an_explicit_accessibility_label_replaces_the_visible_one(cx: &mut TestAppContext) {
    cx.update(crate::init);
    let cx = cx.add_empty_window();
    cx.update(|window, cx| {
        let state = cx.new(|cx| ColorPickerState::new(window, cx));

        let plain = ColorPicker::new(&state).label("Color");
        assert_eq!(plain.accessibility_label, None);
        assert_eq!(plain.label.as_deref(), Some("Color"));

        let named = ColorPicker::new(&state)
            .label("Color")
            .accessibility_label("Text color");
        assert_eq!(
            named.accessibility_label.as_deref(),
            Some("Text color"),
            "an explicit name must win over the visible label"
        );
        assert_eq!(
            named.label.as_deref(),
            Some("Color"),
            "and must not change what is drawn"
        );
    });
}

#[gpui::test]
fn test_color_select_builder(cx: &mut TestAppContext) {
    cx.update(crate::init);
    let cx = cx.add_empty_window();
    cx.update(|window, cx| {
        let state = cx.new(|cx| ColorPickerState::new(window, cx));
        let select = ColorSelect::new(&state)
            .large()
            .placeholder("Pick a color")
            .accessibility_label("Theme color")
            .featured_colors(vec![hsla(0., 1., 0.5, 1.)])
            .swatches([ColorSwatch::new("Red", hsla(0., 1., 0.5, 1.))])
            .default_mode(ColorPickerMode::Palette)
            .disabled(true);

        assert!(select.picker.field);
        assert_eq!(select.picker.size, Size::Large);
        assert_eq!(select.picker.placeholder.as_deref(), Some("Pick a color"));
        assert_eq!(
            select.picker.accessibility_label.as_deref(),
            Some("Theme color")
        );
        assert_eq!(
            select.picker.featured_colors.as_ref().map(Vec::len),
            Some(1)
        );
        assert_eq!(select.picker.swatches.as_ref().map(|s| s.len()), Some(1));
        assert_eq!(select.picker.default_mode, Some(ColorPickerMode::Palette));
        assert!(select.picker.disabled);
    });
}

struct SelectHarness {
    state: Entity<ColorPickerState>,
}

impl Render for SelectHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(400.)).child(ColorSelect::new(&self.state))
    }
}

#[gpui::test]
fn a_click_anywhere_on_the_color_select_opens_the_picker(cx: &mut TestAppContext) {
    cx.update(crate::init);
    let (view, cx) = cx.add_window_view(|window, cx| SelectHarness {
        state: cx.new(|cx| ColorPickerState::new(window, cx)),
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let open = cx.update(|_, cx| view.read(cx).state.read(cx).is_open());
    assert!(!open, "the picker starts closed");

    // Far from the swatch, where only the field frame can take the click.
    cx.simulate_click(point(px(300.), px(16.)), Default::default());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let open = cx.update(|_, cx| view.read(cx).state.read(cx).is_open());
    assert!(open, "the whole field is the trigger, like a Select");
}

struct PixelSizedHarness {
    state: Entity<ColorPickerState>,
}

impl Render for PixelSizedHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(ColorPicker::new(&self.state).with_size(px(36.)))
    }
}

/// An application lays the trigger out with a pixel size, as a theme editor's
/// color cell does. That sizes the swatch only; the popup keeps its own controls.
#[gpui::test]
fn a_pixel_sized_trigger_opens_the_full_panel(cx: &mut TestAppContext) {
    cx.update(crate::init);
    let (view, visual) = cx.add_window_view(|window, cx| {
        window.activate_window();
        PixelSizedHarness {
            state: cx
                .new(|cx| ColorPickerState::new(window, cx).default_value(hsla(0.2, 0.5, 0.5, 1.))),
        }
    });
    visual.simulate_resize(size(px(700.), px(600.)));
    draw(visual);
    assert!(visual.debug_bounds("color-picker-panel").is_none());
    let trigger = visual.debug_bounds("color-picker-trigger").unwrap();
    assert_eq!(trigger.size.height, px(36.));
    open_popup(visual);
    assert!(visual.update(|_, cx| view.read(cx).state.read(cx).is_open()));
    assert!(visual.debug_bounds("color-picker-spectrum").is_some());
}

#[gpui::test]
fn hex_preview_does_not_commit_until_enter_and_bad_input_cannot_commit(cx: &mut TestAppContext) {
    let initial: Hsla = gpui::rgb(0x336699).into();
    let (view, visual) = setup(cx, initial, |_| {});
    focus_hex(&view, visual);
    visual.simulate_input("#FF000080");
    draw(visual);
    let (value, displayed) = model(&view, visual);
    assert_eq!(value, Some(initial));
    visual.update(|_, cx| assert!(view.read(cx).changes.is_empty()));
    let preview = Rgba::from(displayed.unwrap());
    assert_eq!(preview.r, 1.);
    assert!((preview.a - 128. / 255.).abs() < 0.001);

    visual.simulate_keystrokes("enter");
    draw(visual);
    let color = Rgba::from(model(&view, visual).0.unwrap());
    assert_eq!(color.r, 1.);
    assert!((color.a - 128. / 255.).abs() < 0.001);
    visual.update(|_, cx| assert_eq!(view.read(cx).changes.len(), 1));

    focus_hex(&view, visual);
    visual.simulate_input("#12");
    visual.simulate_keystrokes("enter");
    draw(visual);
    visual.update(|_, cx| assert_eq!(view.read(cx).changes.len(), 1));
}

/// The first edit after the model writes the field does not reach the model's own
/// subscriber, so a single-edit paste of a complete value must preview from here.
#[gpui::test]
fn the_first_hex_edit_after_programmatic_sync_previews_without_a_commit(cx: &mut TestAppContext) {
    let initial: Hsla = gpui::rgb(0x336699).into();
    let (view, visual) = setup(cx, initial, |_| {});
    for next in [gpui::rgb(0x336699), gpui::rgb(0x995533)] {
        visual.update(|window, cx| {
            let state = view.read(cx).state.clone();
            state.update(cx, |state, cx| state.set_value(next, window, cx));
            let input = state.read(cx).hex_input().clone();
            input.update(cx, |input, cx| {
                input.replace_all("#00FF0080", window, cx);
            });
        });
        draw(visual);
        let (value, displayed) = model(&view, visual);
        assert_eq!(value, Some(next.into()));
        let preview = Rgba::from(displayed.unwrap());
        assert_eq!((preview.g, preview.r), (1., 0.));
        assert!((preview.a - 128. / 255.).abs() < 0.001);
        visual.update(|_, cx| assert!(view.read(cx).changes.is_empty()));
    }
}

#[gpui::test]
fn slider_keyboard_and_disabled_state_are_effective(cx: &mut TestAppContext) {
    let (view, visual) = setup(cx, gpui::rgb(0xff0000), |probe| {
        probe.mode = ColorPickerMode::Hsla;
    });
    let rail = visual.debug_bounds("color-picker-hue").unwrap();
    visual.simulate_click(rail.center(), Default::default());
    visual.simulate_keystrokes("right");
    draw(visual);
    visual.update(|_, cx| assert!(view.read(cx).changes.len() >= 2));
    view.update(visual, |view, cx| {
        view.disabled = true;
        view.changes.clear();
        cx.notify();
    });
    draw(visual);
    visual.simulate_click(rail.center(), Default::default());
    visual.simulate_keystrokes("right end");
    draw(visual);
    visual.update(|_, cx| assert!(view.read(cx).changes.is_empty()));
}

#[gpui::test]
fn a_disabled_trigger_does_not_open(cx: &mut TestAppContext) {
    let (view, visual) = setup(cx, gpui::rgb(0xff0000), |probe| {
        probe.inline = false;
        probe.disabled = true;
    });
    open_popup(visual);
    assert!(!visual.update(|_, cx| view.read(cx).state.read(cx).is_open()));
    assert!(visual.debug_bounds("color-picker-panel").is_none());
}

#[gpui::test]
fn cancelling_or_committing_the_popup_restores_focus_and_drops_the_preview(
    cx: &mut TestAppContext,
) {
    let initial: Hsla = gpui::rgb(0x336699).into();
    let (view, visual) = setup(cx, initial, |probe| probe.inline = false);
    let root_focus = visual.update(|window, cx| {
        let focus = view.read(cx).state.focus_handle(cx);
        focus.focus(window, cx);
        focus
    });
    visual.simulate_keystrokes("enter");
    draw(visual);
    assert!(visual.update(|_, cx| view.read(cx).state.read(cx).is_open()));
    focus_hex(&view, visual);
    visual.simulate_input("#00FF00");
    draw(visual);
    assert_ne!(model(&view, visual).1, Some(initial));

    visual.simulate_keystrokes("escape");
    draw(visual);
    assert_eq!(model(&view, visual), (Some(initial), Some(initial)));
    visual.update(|window, cx| {
        assert!(!view.read(cx).state.read(cx).is_open());
        assert!(view.read(cx).changes.is_empty());
        assert!(root_focus.is_focused(window), "focus returns to the root");
    });

    visual.simulate_keystrokes("enter");
    draw(visual);
    focus_hex(&view, visual);
    visual.simulate_input("#00FF00");
    visual.simulate_keystrokes("enter");
    draw(visual);
    visual.update(|window, cx| {
        let state = view.read(cx).state.read(cx);
        assert!(!state.is_open(), "committing a hex value closes the popup");
        assert_eq!(view.read(cx).changes.len(), 1);
        assert_eq!(Rgba::from(state.value().unwrap()).g, 1.);
        assert!(root_focus.is_focused(window));
    });
}

#[gpui::test]
fn closed_picker_omits_panels_and_only_mounts_the_selected_mode(cx: &mut TestAppContext) {
    let (view, visual) = setup(cx, gpui::rgb(0x336699), |probe| probe.inline = false);
    for selector in [
        "color-picker-panel",
        "color-picker-spectrum",
        "color-picker-palette",
        "color-picker-hsla",
    ] {
        assert!(
            visual.debug_bounds(selector).is_none(),
            "closed picker mounted {selector}"
        );
    }
    open_popup(visual);
    let bar = visual.debug_bounds("color-picker-panel").unwrap();
    for (position, expected) in [
        (0.9, "color-picker-hsla"),
        (0.1, "color-picker-spectrum"),
        (0.5, "color-picker-palette"),
    ] {
        visual.simulate_click(
            point(bar.left() + bar.size.width * position, bar.top() + px(8.)),
            Default::default(),
        );
        draw(visual);
        for selector in [
            "color-picker-spectrum",
            "color-picker-palette",
            "color-picker-hsla",
        ] {
            assert_eq!(
                visual.debug_bounds(selector).is_some(),
                selector == expected,
                "clicking its tab must mount exactly {expected}"
            );
        }
    }
    visual.update(|_, cx| assert!(view.read(cx).changes.is_empty()));
    visual.update(|window, cx| {
        let focus = view.read(cx).state.read(cx).hex_input().focus_handle(cx);
        focus.focus(window, cx);
    });
    visual.simulate_keystrokes("escape");
    draw(visual);
    assert!(visual.debug_bounds("color-picker-panel").is_none());
    open_popup(visual);
    assert!(
        visual.debug_bounds("color-picker-palette").is_some(),
        "reopening preserves the chosen view"
    );
}

#[gpui::test]
fn spectrum_changes_show_the_same_hex_in_the_field_and_trigger(cx: &mut TestAppContext) {
    let (view, visual) = setup(cx, gpui::rgb(0xff0000), |probe| probe.inline = false);
    open_popup(visual);
    for selector in ["color-picker-plane", "color-picker-alpha"] {
        let bounds = visual.debug_bounds(selector).unwrap();
        visual.simulate_click(bounds.center(), Default::default());
        draw(visual);
        visual.update(|_, cx| {
            let state = view.read(cx).state.read(cx);
            assert_eq!(
                state.hex_input().read(cx).value().as_str(),
                super::hex_label(state.value().unwrap())
            );
        });
    }
    visual.update(|_, cx| {
        let committed = view.read(cx).state.read(cx).value().unwrap();
        assert!((committed.s - 0.5).abs() < 0.001);
        assert!((committed.l - 0.5).abs() < 0.001);
        assert!((committed.a - 0.5).abs() < 0.001);
        assert_eq!(view.read(cx).changes.len(), 2);
    });
}

/// Hovering a swatch shows its color without touching the committed one, and a
/// repaint by an owner whose value did not change must leave that alone.
#[gpui::test]
fn hovering_a_swatch_previews_survives_repaints_and_ends_with_the_popup(cx: &mut TestAppContext) {
    let initial: Hsla = gpui::rgb(0x336699).into();
    let swatch = hsla(0.1, 0.8, 0.5, 1.);
    let (view, visual) = setup(cx, initial, |probe| {
        probe.inline = false;
        probe.mode = ColorPickerMode::Palette;
        probe.swatches = Some(vec![ColorSwatch::new("One", swatch)]);
    });
    open_popup(visual);
    let bounds = visual
        .debug_bounds(swatch_selector(swatch))
        .expect("the palette is open");
    visual.simulate_mouse_move(bounds.center(), None, Default::default());
    draw(visual);
    draw(visual);
    assert_eq!(model(&view, visual), (Some(initial), Some(swatch)));
    visual.update(|_, cx| assert!(view.read(cx).changes.is_empty()));

    visual.simulate_keystrokes("escape");
    draw(visual);
    assert_eq!(model(&view, visual), (Some(initial), Some(initial)));
}

/// A popover binds Enter to its own confirm. On a focused swatch that must select
/// the color rather than close the popup with nothing chosen.
#[gpui::test]
fn enter_on_a_focused_swatch_commits_in_a_popup(cx: &mut TestAppContext) {
    let initial: Hsla = gpui::rgb(0x336699).into();
    let swatch = hsla(0.1, 0.8, 0.5, 1.);
    let (view, visual) = setup(cx, initial, |probe| {
        probe.inline = false;
        probe.mode = ColorPickerMode::Palette;
        probe.swatches = Some(vec![ColorSwatch::new("One", swatch)]);
    });
    open_popup(visual);
    let bounds = visual.debug_bounds(swatch_selector(swatch)).unwrap();
    visual.simulate_mouse_down(bounds.center(), MouseButton::Left, Default::default());
    visual.simulate_mouse_up(
        point(px(690.), px(590.)),
        MouseButton::Left,
        Default::default(),
    );
    // A real key press: the popover leaves Enter in its content to the focused
    // control's own activation, which needs the key down and up pair.
    let keystroke = gpui::Keystroke::parse("enter").unwrap();
    visual.simulate_event(gpui::KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    visual.simulate_event(gpui::KeyUpEvent { keystroke });
    draw(visual);
    let (value, _) = model(&view, visual);
    assert_eq!(value, Some(swatch));
    visual.update(|_, cx| {
        assert!(!view.read(cx).state.read(cx).is_open());
        assert_eq!(view.read(cx).changes, vec![Some(swatch)]);
    });
}

/// A hex round trip loses the hue at the plane's left edge, where saturation is
/// zero. The drag must carry the hue through, not snap to red on the way back.
#[gpui::test]
fn a_hex_owner_echo_keeps_the_hue_through_the_plane_edge(cx: &mut TestAppContext) {
    let (view, visual) = setup(cx, hsla(0.6, 0.7, 0.4, 1.), |probe| probe.echo = true);
    let plane = visual.debug_bounds("color-picker-plane").unwrap();
    let edge = point(plane.left() - px(20.), plane.center().y);
    visual.simulate_mouse_down(plane.center(), MouseButton::Left, Default::default());
    visual.simulate_mouse_move(edge, MouseButton::Left, Default::default());
    draw(visual);
    let at_edge = model(&view, visual).0.unwrap();
    assert_eq!(at_edge.s, 0., "the echo stored a gray");

    let inside = point(plane.left() + plane.size.width * 0.7, plane.center().y);
    visual.simulate_mouse_move(inside, MouseButton::Left, Default::default());
    draw(visual);
    let value = model(&view, visual).0.unwrap();
    assert!(
        same_at_hex_precision(value, hsla(0.6, 0.7, 0.5, 1.)),
        "the hue survives the gray at the edge: {value:?}"
    );
    visual.simulate_mouse_up(inside, MouseButton::Left, Default::default());
}

#[test]
fn hex_field_validity_matches_the_parser_lengths() {
    for value in ["#fff", "#ffff", "#123456", "#12345678", "abc"] {
        assert!(valid_hex(value), "{value}");
    }
    for value in ["", "#12", "#12345", "#1234567", "#gggggg"] {
        assert!(!valid_hex(value), "{value}");
    }
}
