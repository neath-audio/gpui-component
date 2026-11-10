//! Lazily mounted HSLA controls. The model remains the full-precision source.

use gpui::{
    AnyElement, App, AppContext as _, Context, ElementId, Entity, Hsla, InteractiveElement as _,
    IntoElement, ParentElement as _, SharedString, Styled as _, Subscription, TextAlign, Window,
    div, rems,
};
use gpui_base::{Decrement, Increment, StepAction, step_value};
use rust_i18n::t;

use super::{ColorEditor, ColorPickerMode, keep_confirm};
use crate::{
    ActiveTheme as _, Sizable as _, Size, StyledTypography as _, h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};

const MAXIMA: [f32; 4] = [360., 100., 100., 100.];

fn components(color: Hsla) -> [f32; 4] {
    [color.h, color.s, color.l, color.a]
}

fn labels() -> [SharedString; 4] {
    [
        t!("ColorPicker.Hue").into(),
        t!("ColorPicker.Saturation").into(),
        t!("ColorPicker.Lightness").into(),
        t!("ColorPicker.Alpha").into(),
    ]
}

fn display_value(value: f32, index: usize) -> SharedString {
    // Display precision never becomes model precision unless this channel is edited.
    format!("{:.2}", value * MAXIMA[index])
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
        .into()
}

fn parse_channel(text: &str, index: usize) -> Option<f32> {
    text.trim()
        .parse::<f32>()
        .ok()
        .filter(|value| value.is_finite())
        .map(|value| value.clamp(0., MAXIMA[index]) / MAXIMA[index])
}

fn commit_channel(
    editor: &ColorEditor,
    requires_open: bool,
    index: usize,
    value: f32,
    window: &mut Window,
    cx: &mut App,
) {
    let model = editor.state().read(cx);
    if model.active_tab() != ColorPickerMode::Hsla as usize || (requires_open && !model.is_open()) {
        return;
    }
    let mut color = editor.color(cx);
    match index {
        0 => color.h = value,
        1 => color.s = value,
        2 => color.l = value,
        _ => color.a = value,
    }
    editor.commit(color, window, cx);
}

struct Channels {
    editor: ColorEditor,
    inputs: [Entity<InputState>; 4],
    values: [f32; 4],
    text: [SharedString; 4],
    disabled: bool,
    requires_open: bool,
    _subscriptions: Vec<Subscription>,
}

impl Channels {
    fn new(editor: ColorEditor, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let requires_open = editor.state().read(cx).is_open();
        let values = components(editor.color(cx));
        let text = std::array::from_fn(|index| display_value(values[index], index));
        let inputs: [Entity<InputState>; 4] = std::array::from_fn(|index| {
            cx.new(|cx| {
                let mut input = InputState::new(window, cx)
                    .default_value(text[index].clone())
                    .min(0.)
                    .max(MAXIMA[index] as f64)
                    .step(1.);
                input.ensure_number_mask();
                input
            })
        });
        let mut subscriptions = vec![cx.observe(editor.state(), |_, _, cx| cx.notify())];
        for (index, input) in inputs.iter().enumerate() {
            subscriptions.push(cx.subscribe_in(
                input,
                window,
                move |this, _, event: &InputEvent, window, cx| {
                    if matches!(event, InputEvent::Change) {
                        this.changed(index, window, cx);
                    }
                    if matches!(event, InputEvent::Blur | InputEvent::PressEnter { .. }) {
                        this.changed(index, window, cx);
                        // Valid typed precision stays for the next keyboard step. Only
                        // unfinished or out-of-range edits return to the model's value.
                        let value = this.inputs[index].read(cx).value();
                        if !value.trim().parse::<f32>().is_ok_and(|value| {
                            value.is_finite() && (0. ..=MAXIMA[index]).contains(&value)
                        }) {
                            this.restore(index, window, cx);
                        }
                    }
                },
            ));
        }
        Self {
            editor,
            inputs,
            values,
            text,
            disabled: false,
            requires_open,
            _subscriptions: subscriptions,
        }
    }

    fn changed(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.inputs[index].read(cx).value();
        if text == self.text[index] {
            return;
        }
        self.text[index] = text.clone();
        if self.disabled {
            self.restore(index, window, cx);
            return;
        }
        if let Some(value) = parse_channel(&text, index) {
            self.values[index] = value;
            commit_channel(&self.editor, self.requires_open, index, value, window, cx);
        }
        cx.notify();
    }

    fn restore(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let value = components(self.editor.color(cx))[index];
        self.write(index, value, window, cx);
    }

    fn write(&mut self, index: usize, value: f32, window: &mut Window, cx: &mut Context<Self>) {
        self.values[index] = value;
        let text = display_value(value, index);
        self.text[index] = text.clone();
        if self.inputs[index].read(cx).value() != text {
            self.inputs[index].update(cx, |input, cx| input.set_value(text, window, cx));
        }
    }

    fn sync(&mut self, disabled: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.disabled = disabled;
        for (index, value) in components(self.editor.color(cx)).into_iter().enumerate() {
            if self.values[index] != value {
                self.write(index, value, window, cx);
            }
        }
    }

    fn step(
        &mut self,
        index: usize,
        action: StepAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let model = self.editor.state().read(cx);
        if self.disabled
            || model.active_tab() != ColorPickerMode::Hsla as usize
            || (self.requires_open && !model.is_open())
        {
            return;
        }
        let text = self.inputs[index].read(cx).value();
        if let Some(value) = step_value(&text, action, 1., Some(0.), Some(MAXIMA[index] as f64)) {
            self.inputs[index].update(cx, |input, cx| input.replace_all(value, window, cx));
        }
    }
}

pub(super) fn panel(
    id: ElementId,
    editor: ColorEditor,
    size: Size,
    disabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let memory = window.use_keyed_state(
        (
            id.clone(),
            format!("hsla-fields-{}", editor.state().entity_id().as_u64()),
        ),
        cx,
        {
            let editor = editor.clone();
            move |window, cx| Channels::new(editor, window, cx)
        },
    );
    memory.update(cx, |memory, cx| memory.sync(disabled, window, cx));
    let inputs = memory.read(cx).inputs.clone();
    let tracks = super::spectrum::hsla_tracks(id.clone(), editor, disabled, window, cx);
    let labels = labels();

    v_flex()
        .id((id.clone(), "hsla-panel"))
        .debug_selector(|| "color-picker-hsla".into())
        .w_full()
        .min_w_0()
        .gap_2()
        .children(tracks.into_iter().enumerate().map(|(index, track)| {
            let label = labels[index].clone();
            let increment = memory.clone();
            let decrement = memory.clone();
            let number = keep_confirm(
                div()
                    .w_20()
                    .flex_none()
                    .key_context("NumberInput")
                    .on_action(move |_: &Increment, window, cx| {
                        increment.update(cx, |channels, cx| {
                            channels.step(index, StepAction::Increment, window, cx)
                        });
                    })
                    .on_action(move |_: &Decrement, window, cx| {
                        decrement.update(cx, |channels, cx| {
                            channels.step(index, StepAction::Decrement, window, cx)
                        });
                    }),
            )
            .child(
                Input::new(&inputs[index])
                    .id((id.clone(), format!("field-{index}")))
                    .with_size(size)
                    .aria_label(label.clone())
                    .disabled(disabled)
                    .w_full()
                    .text_align(TextAlign::Right),
            );
            h_flex()
                .w_full()
                .min_w_0()
                .gap_2()
                .child(
                    div()
                        .debug_selector(move || format!("color-picker-channel-label-{index}"))
                        .w(rems(4.5))
                        .flex_none()
                        .whitespace_nowrap()
                        .text_ui_sm(cx)
                        .text_color(cx.theme().muted_foreground)
                        .child(label),
                )
                .child(div().flex_1().min_w_0().child(track))
                .child(number)
        }))
        .into_any_element()
}

#[cfg(test)]
mod tests;
