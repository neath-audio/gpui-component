use super::{Channels, ColorEditor, ColorPickerMode, panel};
use crate::color_picker::{ColorPickerEvent, ColorPickerState};
use crate::{Colorize as _, Size, input::InputState};
use gpui::{
    AppContext as _, Context, ElementId, Entity, Hsla, InteractiveElement as _, IntoElement,
    KeyBinding, ParentElement as _, Render, Styled as _, Subscription, TestAppContext,
    VisualTestContext, Window, div, px,
};
use gpui_base::actions::Confirm;

const CONTEXT: &str = "ChannelsHost";

struct Probe {
    model: Entity<ColorPickerState>,
    disabled: bool,
    changes: usize,
    /// How many confirms reached the panel's parent, which a field's Enter must not.
    enters: usize,
    /// Stores each committed color as hex and hands it back, as a theme editor does.
    echo: bool,
    inputs: Option<[Entity<InputState>; 4]>,
    _subscription: Subscription,
}

impl Render for Probe {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.model
            .update(cx, |model, cx| model.sync_pending_value(window, cx));
        let id: ElementId = "picker".into();
        let editor = ColorEditor::new(&id, &self.model, None, window, cx);
        let content = panel(id.clone(), editor, Size::Medium, self.disabled, window, cx);
        let model_id = self.model.entity_id().as_u64();
        self.inputs = Some(
            window
                .use_keyed_state(
                    (id, format!("hsla-fields-{model_id}")),
                    cx,
                    |_, _: &mut Context<Channels>| panic!("HSLA panel was not mounted"),
                )
                .read(cx)
                .inputs
                .clone(),
        );
        div()
            .w(px(272.))
            .key_context(CONTEXT)
            .on_action(cx.listener(|this, _: &Confirm, _, _| this.enters += 1))
            .child(content)
    }
}

fn initial() -> Hsla {
    gpui::hsla(0.27182818, 0.618034, 0.4123456, 0.7312345)
}

fn setup(cx: &mut TestAppContext) -> (Entity<Probe>, &mut VisualTestContext) {
    cx.update(|cx| {
        crate::init(cx);
        cx.bind_keys([KeyBinding::new(
            "enter",
            Confirm { secondary: false },
            Some(CONTEXT),
        )]);
    });
    let (view, visual) = cx.add_window_view(|window, cx| {
        window.activate_window();
        let model = cx.new(|cx| {
            let mut model = ColorPickerState::new(window, cx).default_value(initial());
            model.set_active_tab(ColorPickerMode::Hsla as usize, cx);
            model
        });
        let subscription = cx.subscribe_in(
            &model,
            window,
            |this: &mut Probe, model, event: &ColorPickerEvent, window, cx| {
                this.changes += 1;
                let ColorPickerEvent::Change(Some(color)) = event else {
                    return;
                };
                if this.echo {
                    let stored = Hsla::parse_hex(&color.to_hex()).unwrap();
                    model.update(cx, |model, cx| model.set_value(stored, window, cx));
                }
                cx.notify();
            },
        );
        Probe {
            model,
            disabled: false,
            changes: 0,
            enters: 0,
            echo: false,
            inputs: None,
            _subscription: subscription,
        }
    });
    draw(visual);
    (view, visual)
}

fn draw(visual: &mut VisualTestContext) {
    visual.update(|window, cx| window.draw(cx).clear(cx));
}

fn input(view: &Entity<Probe>, visual: &mut VisualTestContext, index: usize) -> Entity<InputState> {
    visual.update(|_, cx| view.read(cx).inputs.as_ref().unwrap()[index].clone())
}

fn type_channel(view: &Entity<Probe>, visual: &mut VisualTestContext, index: usize, text: &str) {
    let input = input(view, visual, index);
    visual.update(|window, cx| {
        input.update(cx, |input, cx| {
            input.focus(window, cx);
            input.select_all(window, cx);
        });
    });
    visual.simulate_input(text);
    draw(visual);
}

/// A channel's field commits its value on Enter and keeps the confirm, so a dialog
/// around the panel does not also take it for its own.
#[gpui::test]
fn enter_in_a_channel_field_stays_in_the_panel(cx: &mut TestAppContext) {
    let (view, visual) = setup(cx);
    type_channel(&view, visual, 1, "50");
    visual.simulate_keystrokes("enter");
    draw(visual);
    visual.update(|_, cx| {
        assert_eq!(view.read(cx).model.read(cx).value().unwrap().s, 0.5);
        assert_eq!(
            view.read(cx).enters,
            0,
            "the panel keeps the Enter it acts on"
        );
    });
}

#[gpui::test]
fn hsla_keyboard_steps_preserve_undo_and_typed_precision(cx: &mut TestAppContext) {
    let (view, visual) = setup(cx);
    type_channel(&view, visual, 0, "123.456789");
    visual.simulate_keystrokes("up");
    draw(visual);
    let hue = input(&view, visual, 0);
    visual.update(|_, cx| assert_eq!(hue.read(cx).value(), "124.456789"));
    visual.simulate_keystrokes(if cfg!(target_os = "macos") {
        "cmd-z"
    } else {
        "ctrl-z"
    });
    draw(visual);
    visual.update(|_, cx| {
        assert_eq!(hue.read(cx).value(), "123.456789");
        assert_eq!(
            view.read(cx).model.read(cx).value().unwrap().h,
            "123.456789".parse::<f32>().unwrap() / 360.
        );
    });
}

#[gpui::test]
fn hsla_numeric_edits_and_arrow_keys_commit_only_the_edited_channel(cx: &mut TestAppContext) {
    let (view, visual) = setup(cx);
    // The keyboard helper inserts characters separately: each of the nine digits
    // changes the live value, while the decimal separator does not.
    const TYPED_CHANGES: usize = 9;
    type_channel(&view, visual, 0, "123.456789");
    visual.update(|_, cx| {
        let color = view.read(cx).model.read(cx).value().unwrap();
        assert_eq!(color.h, "123.456789".parse::<f32>().unwrap() / 360.);
        assert_eq!(
            (color.s, color.l, color.a),
            (initial().s, initial().l, initial().a)
        );
        assert_eq!(view.read(cx).changes, TYPED_CHANGES);
    });
    let hue = input(&view, visual, 0);
    let saturation = input(&view, visual, 1);
    visual.update(|window, cx| saturation.update(cx, |input, cx| input.focus(window, cx)));
    draw(visual);
    visual.update(|_, cx| {
        assert_eq!(hue.read(cx).value().as_str(), "123.456789");
        assert_eq!(view.read(cx).changes, TYPED_CHANGES);
    });
    visual.update(|window, cx| hue.update(cx, |input, cx| input.focus(window, cx)));
    visual.simulate_keystrokes("up");
    draw(visual);
    visual.update(|_, cx| {
        let color = view.read(cx).model.read(cx).value().unwrap();
        assert_eq!(color.h, "124.456789".parse::<f32>().unwrap() / 360.);
        assert_eq!(
            (color.s, color.l, color.a),
            (initial().s, initial().l, initial().a)
        );
        assert_eq!(view.read(cx).changes, TYPED_CHANGES + 1);
    });
    visual.simulate_keystrokes("down");
    draw(visual);
    visual.update(|_, cx| {
        assert_eq!(
            view.read(cx).model.read(cx).value().unwrap().h,
            "123.456789".parse::<f32>().unwrap() / 360.
        );
        assert_eq!(view.read(cx).changes, TYPED_CHANGES + 2);
    });
    // Enter and repeated rendering must not feed field and model sync back into
    // another commit, even though the input has more than display precision.
    visual.simulate_keystrokes("enter");
    draw(visual);
    draw(visual);
    visual.update(|_, cx| assert_eq!(view.read(cx).changes, TYPED_CHANGES + 2));
}

#[gpui::test]
fn hsla_programmatic_sync_preserves_precision_and_incomplete_edits_do_not_commit(
    cx: &mut TestAppContext,
) {
    let (view, visual) = setup(cx);
    let replacement = gpui::hsla(0.8765432, 0.2345678, 0.3456789, 0.456789);
    visual.update(|window, cx| {
        view.read(cx)
            .model
            .clone()
            .update(cx, |model, cx| model.set_value(replacement, window, cx));
    });
    draw(visual);
    let saturation = input(&view, visual, 1);
    visual.update(|_, cx| {
        assert_eq!(saturation.read(cx).value().as_str(), "23.46");
        assert_eq!(view.read(cx).model.read(cx).value(), Some(replacement));
        assert_eq!(view.read(cx).changes, 0);
    });
    visual.update(|window, cx| {
        saturation.update(cx, |input, cx| {
            input.focus(window, cx);
            input.select_all(window, cx);
        });
    });
    visual.simulate_keystrokes("backspace");
    draw(visual);
    visual.update(|_, cx| {
        assert!(saturation.read(cx).value().is_empty());
        assert_eq!(view.read(cx).model.read(cx).value(), Some(replacement));
        assert_eq!(view.read(cx).changes, 0);
    });
    visual.simulate_input("52.25");
    draw(visual);
    visual.update(|_, cx| {
        let color = view.read(cx).model.read(cx).value().unwrap();
        assert_eq!(color.s, 0.5225);
        assert_eq!(
            (color.h, color.l, color.a),
            (replacement.h, replacement.l, replacement.a)
        );
        // Four digits change the value; appending the decimal point does not.
        assert_eq!(view.read(cx).changes, 4);
    });
    // Replacing a value with itself is one edit that must not emit at all.
    visual.update(|window, cx| {
        saturation.update(cx, |input, cx| input.replace_all("52.25", window, cx));
    });
    draw(visual);
    visual.update(|_, cx| assert_eq!(view.read(cx).changes, 4));
}

#[gpui::test]
fn hsla_disabled_fields_and_sliders_block_all_mutation(cx: &mut TestAppContext) {
    let (view, visual) = setup(cx);
    type_channel(&view, visual, 3, "60");
    let committed = visual.update(|_, cx| view.read(cx).model.read(cx).value());
    view.update(visual, |view, cx| {
        view.disabled = true;
        view.changes = 0;
        cx.notify();
    });
    draw(visual);
    visual.simulate_keystrokes("cmd-a backspace up");
    visual.simulate_input("20");
    for selector in [
        "color-picker-hue",
        "color-picker-saturation",
        "color-picker-lightness",
        "color-picker-alpha",
    ] {
        let rail = visual.debug_bounds(selector).unwrap();
        visual.simulate_click(rail.center(), Default::default());
    }
    draw(visual);
    visual.update(|_, cx| {
        assert_eq!(view.read(cx).model.read(cx).value(), committed);
        assert_eq!(view.read(cx).changes, 0);
    });
}

#[gpui::test]
fn hsla_stale_field_notifications_cannot_commit_after_switching_views_or_closing(
    cx: &mut TestAppContext,
) {
    let (view, visual) = setup(cx);
    let hue = input(&view, visual, 0);
    visual.update(|window, cx| {
        let model = view.read(cx).model.clone();
        model.update(cx, |model, cx| {
            model.set_active_tab(ColorPickerMode::Palette as usize, cx)
        });
        // A delayed field notification must check the live mode, even before the
        // next paint drops the previous panel's retained subscriptions.
        hue.update(cx, |input, cx| input.replace_all("30", window, cx));
    });
    visual.update(|_, cx| {
        assert_eq!(view.read(cx).model.read(cx).value(), Some(initial()));
        assert_eq!(view.read(cx).changes, 0);
    });
    // Mount a fresh channel model as a popup, then close it while a retained editor
    // still exists. The old editor must not revive a closed popup.
    visual.update(|window, cx| {
        let model = cx.new(|cx| {
            let mut model = ColorPickerState::new(window, cx).default_value(initial());
            model.set_active_tab(ColorPickerMode::Hsla as usize, cx);
            model.set_open(true, cx);
            model
        });
        view.update(cx, |view, cx| {
            view.model = model;
            cx.notify();
        });
    });
    draw(visual);
    let hue = input(&view, visual, 0);
    visual.update(|window, cx| {
        let model = view.read(cx).model.clone();
        model.update(cx, |model, cx| model.set_open(false, cx));
        hue.update(cx, |input, cx| input.replace_all("70", window, cx));
    });
    visual.update(|_, cx| {
        assert_eq!(view.read(cx).model.read(cx).value(), Some(initial()));
        assert!(!view.read(cx).model.read(cx).is_open());
    });
}

/// A hex round trip moves each channel a little. That must not rewrite the digits
/// under the caret, or the next keystroke lands in a value the user never typed.
#[gpui::test]
fn a_hex_owner_echo_does_not_rewrite_the_field_being_typed(cx: &mut TestAppContext) {
    let (view, visual) = setup(cx);
    view.update(visual, |view, _| view.echo = true);
    type_channel(&view, visual, 0, "123");
    let hue = input(&view, visual, 0);
    visual.update(|_, cx| assert_eq!(hue.read(cx).value().as_str(), "123"));
}
