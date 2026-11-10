use super::{ColorEditor, ColorSwatch, default_palette, panel};
use crate::Colorize as _;
use crate::color_picker::{ColorPickerEvent, ColorPickerState};
use gpui::{
    AppContext as _, Context, ElementId, Entity, Hsla, IntoElement, MouseButton,
    ParentElement as _, Render, Styled as _, TestAppContext, VisualTestContext, Window, div, hsla,
    point, px,
};
use std::{cell::RefCell, rc::Rc};

struct Probe {
    state: Entity<ColorPickerState>,
    featured: Option<Vec<Hsla>>,
    swatches: Option<Vec<ColorSwatch>>,
}

impl Render for Probe {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let id: ElementId = "palette".into();
        let editor = ColorEditor::new(&id, &self.state, None, window, cx);
        div().w(px(320.)).child(panel(
            id,
            &editor,
            self.featured.as_deref(),
            self.swatches.as_deref(),
            false,
            cx,
        ))
    }
}

fn setup(
    cx: &mut TestAppContext,
    featured: Option<Vec<Hsla>>,
    swatches: Option<Vec<ColorSwatch>>,
) -> (Entity<Probe>, &mut VisualTestContext) {
    cx.update(crate::init);
    let (view, visual) = cx.add_window_view(|window, cx| {
        window.activate_window();
        Probe {
            state: cx.new(|cx| ColorPickerState::new(window, cx)),
            featured,
            swatches,
        }
    });
    visual.update(|window, cx| window.draw(cx).clear(cx));
    (view, visual)
}

fn selector(color: Hsla) -> &'static str {
    Box::leak(format!("color-picker-swatch-{}", color.to_hex()).into_boxed_str())
}

#[gpui::test]
fn repeated_palette_colors_have_independent_focus_stops(cx: &mut TestAppContext) {
    let color = hsla(0.1, 0.5, 0.5, 1.);
    let (_, visual) = setup(
        cx,
        Some(vec![color, color]),
        Some(vec![ColorSwatch::new("One", color)]),
    );
    visual.update(|window, cx| {
        window.draw(cx).clear(cx);
        let mut focused = Vec::new();
        for _ in 0..3 {
            window.focus_next(cx);
            let handle = window.focused(cx).expect("each swatch is focusable");
            assert!(
                !focused.contains(&handle),
                "equal colors must not share element identity or a focus stop"
            );
            focused.push(handle);
        }
        window.focus_next(cx);
        assert_eq!(window.focused(cx), focused.first().cloned());
    });
}

/// Each row holds twelve colors, whatever the palette, so a thirteenth starts a
/// new row under the first.
#[gpui::test]
fn swatches_wrap_on_a_fixed_column_grid(cx: &mut TestAppContext) {
    let colors: Vec<Hsla> = (0..13)
        .map(|index| hsla(index as f32 / 13., 0.7, 0.5, 1.))
        .collect();
    let swatches = colors
        .iter()
        .enumerate()
        .map(|(index, color)| ColorSwatch::new(format!("Color {index}"), *color))
        .collect();
    let (_, visual) = setup(cx, None, Some(swatches));
    let first = visual.debug_bounds(selector(colors[0])).unwrap();
    let twelfth = visual.debug_bounds(selector(colors[11])).unwrap();
    let thirteenth = visual.debug_bounds(selector(colors[12])).unwrap();
    assert_eq!(first.top(), twelfth.top());
    assert!(twelfth.left() > first.left());
    assert!(thirteenth.top() > first.top());
    assert!((thirteenth.left() - first.left()).abs() <= px(1.));
}

/// A swatch focused by keyboard commits with Space, as a click would.
#[gpui::test]
fn a_focused_swatch_commits_from_the_keyboard(cx: &mut TestAppContext) {
    let color = hsla(0.1, 0.5, 0.5, 1.);
    let (view, visual) = setup(cx, None, Some(vec![ColorSwatch::new("One", color)]));
    let changes = Rc::new(RefCell::new(Vec::new()));
    let _subscription = visual.update(|_, cx| {
        let changes = changes.clone();
        let state = view.read(cx).state.clone();
        cx.subscribe(&state, move |_, event: &ColorPickerEvent, _| {
            let ColorPickerEvent::Change(color) = event;
            changes.borrow_mut().push(*color);
        })
    });
    let swatch = visual.debug_bounds(selector(color)).unwrap();
    visual.simulate_mouse_down(swatch.center(), MouseButton::Left, Default::default());
    visual.simulate_mouse_up(
        point(px(600.), px(600.)),
        MouseButton::Left,
        Default::default(),
    );
    // GPUI activates a focused element on key up, so the press needs both halves.
    let keystroke = gpui::Keystroke::parse("space").unwrap();
    visual.simulate_event(gpui::KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    visual.simulate_event(gpui::KeyUpEvent { keystroke });
    assert_eq!(&*changes.borrow(), &[Some(color)]);
}

#[test]
fn default_palette_has_distinct_neutrals_and_five_ordered_tones_for_twelve_hues() {
    let colors = default_palette();
    assert_eq!(colors.len(), 72);
    for (index, swatch) in colors.iter().enumerate() {
        assert!(
            colors[..index]
                .iter()
                .all(|other| other.label() != swatch.label())
        );
        assert!(
            colors[..index]
                .iter()
                .all(|other| other.color() != swatch.color())
        );
    }
    assert!(colors[..12].iter().all(|swatch| swatch.color().s == 0.));
    for column in 0..12 {
        let tones: Vec<_> = (1..6)
            .map(|row| colors[row * 12 + column].color())
            .collect();
        assert!(
            tones
                .windows(2)
                .all(|pair| pair[0].h == pair[1].h && pair[0].l < pair[1].l)
        );
    }
}

#[gpui::test]
fn leaving_the_swatches_drops_the_hover_preview(cx: &mut TestAppContext) {
    let (view, visual) = setup(cx, None, None);
    let state = visual.update(|_, cx| view.read(cx).state.clone());
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let panel = visual.debug_bounds("color-picker-palette").unwrap();

    let swatch = panel.origin + point(px(8.), px(8.));
    visual.simulate_mouse_move(swatch, None, Default::default());
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let hovered = visual.update(|_, cx| state.read(cx).preview());
    assert!(hovered.is_some(), "hovering a swatch previews its color");
    assert_ne!(hovered, visual.update(|_, cx| state.read(cx).value()));

    visual.simulate_mouse_move(
        panel.bottom_right() + point(px(40.), px(40.)),
        None,
        Default::default(),
    );
    visual.update(|window, cx| window.draw(cx).clear(cx));
    visual.update(|_, cx| assert_eq!(state.read(cx).preview(), state.read(cx).value()));
}
