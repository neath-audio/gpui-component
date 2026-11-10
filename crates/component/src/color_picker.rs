//! Color selection composed from the retained [`ColorPickerState`] model.
//!
//! The popup holds three views of one color: a spectrum plane with hue and alpha
//! rails, a palette of swatches, and HSLA rails with numeric fields. Only the
//! visible view is built, and only while the popup is open. Hex edits preview
//! while typing and commit on Enter or Apply, direct adjustments commit live, and
//! a palette selection commits and closes the popup.

use std::sync::Arc;

use gpui::{
    Anchor, AnyElement, App, BorderStyle, Corners, ElementId, Entity, FocusHandle, Focusable, Hsla,
    InteractiveElement, IntoElement, KeyBinding, ParentElement, Pixels, RenderOnce, Rgba, Role,
    SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled, Subscription, Window,
    canvas, checkerboard, div, fill, outline, prelude::FluentBuilder as _, rems,
};
use rust_i18n::t;

use gpui_base::{ColorPicker as BaseColorPicker, actions::Confirm};
pub use gpui_base::{ColorPickerEvent, ColorPickerState};

use crate::{
    ActiveTheme as _, Colorize as _, Disableable, Icon, Selectable, Sizable, Size, StyleSized,
    StyledExt as _, ThemeStyled as _,
    actions::Cancel,
    button::Button,
    h_flex,
    input::{Input, InputEvent, input_style},
    popover::Popover,
    select::Caret,
    tab::{Tab, TabBar},
    tooltip::{ManagedTooltipExt as _, Tooltip},
    v_flex,
};

mod channels;
mod palette;
mod spectrum;
#[cfg(test)]
mod tests;

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([KeyBinding::new(
        "escape",
        Cancel,
        Some(spectrum::KEY_CONTEXT),
    )]);
}

/// A named preset color. Its color is user data, independent of theme chrome.
#[derive(Clone)]
pub struct ColorSwatch {
    label: SharedString,
    color: Hsla,
}

impl ColorSwatch {
    /// Create a swatch. The label names it in a tooltip and to assistive technology.
    pub fn new(label: impl Into<SharedString>, color: impl Into<Hsla>) -> Self {
        Self {
            label: label.into(),
            color: color.into(),
        }
    }

    /// The name of this swatch.
    pub fn label(&self) -> &SharedString {
        &self.label
    }

    /// The color of this swatch.
    pub fn color(&self) -> Hsla {
        self.color
    }
}

/// The views of one color in a [`ColorPicker`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorPickerMode {
    /// A saturation and lightness plane with hue and alpha rails.
    #[default]
    Spectrum,
    /// A grid of preset swatches.
    Palette,
    /// A rail and a numeric field for each of hue, saturation, lightness and alpha.
    Hsla,
}

impl ColorPickerMode {
    /// [`ColorPickerState::active_tab`] holds this position in the mode bar.
    fn from_tab(tab: usize) -> Self {
        match tab {
            1 => Self::Palette,
            2 => Self::Hsla,
            _ => Self::Spectrum,
        }
    }
}

/// How far apart two colors may be and still be one color once stored as hex.
///
/// Hex keeps eight bits per channel, and a truncating writer can move a channel a
/// whole step, so the bound is one step and half of another for float noise.
const HEX_PRECISION: f32 = 1.5 / 255.;

fn same_at_hex_precision(a: Hsla, b: Hsla) -> bool {
    let (a, b) = (Rgba::from(a), Rgba::from(b));
    [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)]
        .into_iter()
        .all(|(a, b)| (a - b).abs() <= HEX_PRECISION)
}

/// Formats a color as the hex field shows it: `#RRGGBB`, or `#RRGGBBAA` when
/// translucent, each channel rounded to the nearest 8-bit step.
///
/// The trigger must read the same text as the field, and `Colorize::to_hex`
/// truncates instead.
fn hex_label(color: Hsla) -> String {
    let rgba = Rgba::from(color);
    let channel = |value: f32| (value.clamp(0., 1.) * 255.).round() as u8;
    let (r, g, b) = (channel(rgba.r), channel(rgba.g), channel(rgba.b));
    if rgba.a < 1. {
        format!("#{r:02X}{g:02X}{b:02X}{:02X}", channel(rgba.a))
    } else {
        format!("#{r:02X}{g:02X}{b:02X}")
    }
}

/// What the views remember of their own edits between frames.
struct ColorMemory {
    /// The full-precision color the views last wrote to the model.
    exact: Option<Hsla>,
    _hex_edits: Subscription,
}

/// The model of one mounted picker, as its views read and write it.
///
/// An owner that stores colors as hex hands each change straight back through
/// [`ColorPickerState::set_value`], reduced to eight bits per channel. Hue has no
/// value at zero saturation or at black and white, so adopting that echo would
/// snap the plane to red at its edges, and it would rewrite a numeric field while
/// it is being typed in. While the model matches the last edit at hex precision,
/// the views go on editing the exact color.
#[derive(Clone)]
pub(super) struct ColorEditor {
    state: Entity<ColorPickerState>,
    memory: Entity<ColorMemory>,
}

impl ColorEditor {
    fn new(
        id: &ElementId,
        state: &Entity<ColorPickerState>,
        default_mode: Option<ColorPickerMode>,
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let key = (id.clone(), format!("color-{}", state.entity_id().as_u64()));
        let model = state.clone();
        let memory = window.use_keyed_state(key, cx, move |window, cx| {
            if let Some(mode) = default_mode {
                model.update(cx, |model, cx| model.set_active_tab(mode as usize, cx));
            }
            // The first edit after the model writes the field does not reach the
            // model's own subscriber, so a single-edit paste would not preview.
            let input = model.read(cx).hex_input().clone();
            ColorMemory {
                exact: None,
                _hex_edits: cx.subscribe_in(
                    &input,
                    window,
                    move |_, input, event: &InputEvent, window, cx| {
                        if matches!(event, InputEvent::Change) {
                            let value = input.read(cx).value();
                            model.update(cx, |model, cx| {
                                model.preview_hex(value.as_str(), window, cx);
                            });
                        }
                    },
                ),
            }
        });
        Self {
            state: state.clone(),
            memory,
        }
    }

    fn resolved(&self, model: Option<Hsla>, cx: &App) -> Option<Hsla> {
        let exact = self.memory.read(cx).exact;
        match (model, exact) {
            (Some(model), Some(exact)) if same_at_hex_precision(model, exact) => Some(exact),
            _ => model,
        }
    }

    /// The color the views show and edit.
    pub(super) fn color(&self, cx: &App) -> Hsla {
        self.resolved(self.state.read(cx).displayed_color(), cx)
            .unwrap_or(cx.theme().primary)
    }

    /// Writes a directly adjusted color, keeping the popup open. Repeating the
    /// current color emits nothing.
    pub(super) fn commit(&self, color: Hsla, window: &mut Window, cx: &mut App) {
        let model = self.state.read(cx);
        let unchanged = self.resolved(model.value(), cx) == Some(color)
            && self.resolved(model.displayed_color(), cx) == Some(color);
        self.memory
            .update(cx, |memory, _| memory.exact = Some(color));
        if !unchanged {
            self.state
                .update(cx, |state, cx| state.update_color(color, window, cx));
        }
    }

    pub(super) fn state(&self) -> &Entity<ColorPickerState> {
        &self.state
    }
}

/// Applies a requested popup state. Closing drops an uncommitted preview, so an
/// abandoned hex edit or hovered swatch does not outlive the popup.
fn request_open(state: &Entity<ColorPickerState>, open: bool, window: &mut Window, cx: &mut App) {
    state.update(cx, |state, cx| {
        state.set_open(open, cx);
        if !open {
            match state.value() {
                Some(value) => state.set_value(value, window, cx),
                None => state.clear_value(window, cx),
            }
        }
    });
}

/// Keeps a confirm in the text field it was pressed in.
///
/// Enter and Space are also bound to [`Confirm`] by every popover and dialog
/// around the picker. A field's own Enter lets the key travel on, and its
/// subscriber commits only once the key has been dispatched, so without this the
/// popover would close first and restore the old value over the typed one.
fn keep_confirm<E: InteractiveElement>(element: E) -> E {
    element.on_action(|_: &Confirm, _, _| {})
}

fn valid_hex(value: &str) -> bool {
    let value = value.strip_prefix('#').unwrap_or(value);
    matches!(value.len(), 3 | 4 | 6 | 8) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// A color picker element.
///
/// The open popup edits one [`ColorPickerState`], which owns the committed color
/// and its transient preview. Subscribe to [`ColorPickerEvent::Change`] for
/// committed colors, and call [`ColorPickerState::set_value`] when the owner's
/// value changes.
#[derive(IntoElement)]
pub struct ColorPicker {
    id: ElementId,
    style: StyleRefinement,
    state: Entity<ColorPickerState>,
    featured_colors: Option<Vec<Hsla>>,
    swatches: Option<Arc<[ColorSwatch]>>,
    default_mode: Option<ColorPickerMode>,
    label: Option<SharedString>,
    /// The announced name, when the visible label is not it.
    accessibility_label: Option<SharedString>,
    icon: Option<Icon>,
    size: Size,
    anchor: Anchor,
    disabled: bool,
    inline: bool,
    /// Draws the trigger as a framed field, see [`ColorSelect`].
    field: bool,
    placeholder: Option<SharedString>,
}

impl ColorPicker {
    /// Create a new color picker element with the given [`ColorPickerState`].
    pub fn new(state: &Entity<ColorPickerState>) -> Self {
        Self {
            id: ("color-picker", state.entity_id()).into(),
            style: StyleRefinement::default(),
            state: state.clone(),
            featured_colors: None,
            swatches: None,
            default_mode: None,
            size: Size::Medium,
            label: None,
            accessibility_label: None,
            icon: None,
            anchor: Anchor::TopLeft,
            disabled: false,
            inline: false,
            field: false,
            placeholder: None,
        }
    }

    /// Set the featured colors to be displayed in the color picker.
    ///
    /// This is used to display a set of colors that the user can quickly select from,
    /// for example provided user's last used colors. They head the palette.
    pub fn featured_colors(mut self, colors: Vec<Hsla>) -> Self {
        self.featured_colors = Some(colors);
        self
    }

    /// Replace the default palette with named swatches.
    pub fn swatches(mut self, swatches: impl IntoIterator<Item = ColorSwatch>) -> Self {
        self.swatches = Some(swatches.into_iter().collect());
        self
    }

    /// Choose the view shown first. This initializes the view once for a mounted
    /// picker; the user's later choice is retained.
    ///
    /// Default is [`ColorPickerMode::Spectrum`].
    pub fn default_mode(mut self, mode: ColorPickerMode) -> Self {
        self.default_mode = Some(mode);
        self
    }

    /// Embed the same controls in a panel without an opening trigger.
    pub fn inline(mut self, inline: bool) -> Self {
        self.inline = inline;
        self
    }

    /// Set the icon to the color picker button.
    ///
    /// If this is set the color picker button will display the icon.
    /// Else it will display the square color of the current value.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the label to be displayed above the color picker.
    ///
    /// Default is `None`.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the name a screen reader announces, when the visible label is not
    /// it.
    ///
    /// A color picker's name comes from its [`label`](Self::label) by default.
    /// Setting this replaces the announced name without changing the visible
    /// label.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Set the anchor corner of the color picker.
    ///
    /// Default is `Anchor::TopLeft`.
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// The tier of the popup's controls. A pixel size describes only the trigger.
    fn panel_size(&self) -> Size {
        match self.size {
            Size::Size(_) => Size::Medium,
            size => size,
        }
    }

    fn accessible_name(&self) -> Option<SharedString> {
        self.accessibility_label
            .clone()
            .or_else(|| self.label.clone())
    }

    fn render_panel(&self, editor: &ColorEditor, window: &mut Window, cx: &mut App) -> AnyElement {
        let mode = ColorPickerMode::from_tab(self.state.read(cx).active_tab());
        let color = editor.color(cx);
        let size = self.panel_size();
        let disabled = self.disabled;
        let input = self.state.read(cx).hex_input().clone();
        let input_value = input.read(cx).value();
        let valid = valid_hex(&input_value);
        let tab_state = self.state.clone();
        let apply_state = self.state.clone();
        // Deliberately build just the visible view. The numeric editors exist only
        // while the HSLA view is mounted.
        let content = match mode {
            ColorPickerMode::Spectrum => {
                spectrum::panel(self.id.clone(), editor.clone(), disabled, window, cx)
            }
            ColorPickerMode::Palette => palette::panel(
                self.id.clone(),
                editor,
                self.featured_colors.as_deref(),
                self.swatches.as_deref(),
                disabled,
                cx,
            ),
            ColorPickerMode::Hsla => {
                channels::panel(self.id.clone(), editor.clone(), size, disabled, window, cx)
            }
        };

        v_flex()
            .id((self.id.clone(), "panel"))
            .debug_selector(|| "color-picker-panel".into())
            .when(self.inline, |this| {
                this.role(Role::Group)
                    .when_some(self.accessible_name(), |this, name| this.aria_label(name))
            })
            .w_full()
            .min_w_0()
            .gap_3()
            .child(
                TabBar::new("mode")
                    .segmented()
                    .with_size(size)
                    .selected_index(mode as usize)
                    .on_click(move |ix: &usize, _, cx| {
                        tab_state.update(cx, |state, cx| state.set_active_tab(*ix, cx));
                    })
                    .child(
                        Tab::new()
                            .flex_1()
                            .disabled(disabled)
                            .label(t!("ColorPicker.Spectrum")),
                    )
                    .child(
                        Tab::new()
                            .flex_1()
                            .disabled(disabled)
                            .label(t!("ColorPicker.Palette")),
                    )
                    .child(
                        Tab::new()
                            .flex_1()
                            .disabled(disabled)
                            .label(t!("ColorPicker.HSLA")),
                    ),
            )
            .child(content)
            .child(
                h_flex()
                    .w_full()
                    .min_w_0()
                    .gap_2()
                    .child(
                        div()
                            .map(|this| match size {
                                Size::XSmall => this.size_4(),
                                Size::Small => this.size_5(),
                                Size::Large => this.size_8(),
                                _ => this.size_6(),
                            })
                            .flex_none()
                            .child(color_preview(color, window, cx)),
                    )
                    .child(
                        keep_confirm(div().flex_1().min_w_0()).child(
                            Input::new(&input)
                                .with_size(size)
                                .aria_label(t!("ColorPicker.Hex"))
                                .disabled(disabled)
                                .when(!input_value.is_empty() && !valid, |this| {
                                    this.border_color(cx.theme().danger)
                                }),
                        ),
                    )
                    .child(
                        Button::new((self.id.clone(), "apply"))
                            .with_size(size)
                            .label(t!("ColorPicker.Apply"))
                            .disabled(disabled || !valid)
                            .on_click(move |_, window, cx| {
                                apply_state.update(cx, |state, cx| {
                                    let value = state.hex_input().read(cx).value();
                                    state.commit_hex(value.as_str(), window, cx);
                                });
                            }),
                    ),
            )
            .into_any_element()
    }
}

/// The current color over a checkerboard, so its alpha is visible.
fn color_preview(color: Hsla, window: &Window, cx: &App) -> impl IntoElement {
    let surface = cx.theme().muted;
    let checker = checkerboard(cx.theme().border, f32::from(window.rem_size()) / 4.);
    let border = cx.theme().border;
    let radius = cx.theme().radius_tokens().sm;

    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            window.paint_quad(fill(bounds, surface).corner_radii(radius));
            window.paint_quad(fill(bounds, checker).corner_radii(radius));
            window.paint_quad(fill(bounds, color).corner_radii(radius));
            window.paint_quad(outline(bounds, border, BorderStyle::Solid).corner_radii(radius));
        },
    )
    .size_full()
}

/// A keyboard focus ring drawn around a control without taking layout space.
pub(super) fn focus_outline(
    focus: FocusHandle,
    color: Hsla,
    radius: Pixels,
    gap: Pixels,
) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            if focus.is_focused(window) && window.last_input_was_keyboard() {
                let ring = bounds.dilate(gap);
                window.paint_quad(
                    outline(ring, color, BorderStyle::Solid)
                        .corner_radii(Corners::all(radius).clamp_radii_for_quad_size(ring.size)),
                );
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

impl Sizable for ColorPicker {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl Disableable for ColorPicker {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Focusable for ColorPicker {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.state.focus_handle(cx)
    }
}

impl Styled for ColorPicker {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for ColorPicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.state
            .update(cx, |state, cx| state.sync_pending_value(window, cx));
        let editor = ColorEditor::new(&self.id, &self.state, self.default_mode, window, cx);
        if self.disabled && self.state.read(cx).is_open() {
            request_open(&self.state, false, window, cx);
        }

        if self.inline {
            return self.render_panel(&editor, window, cx);
        }

        let state = self.state.read(cx);
        let display_title: SharedString = if let Some(value) = state.value() {
            hex_label(value)
        } else {
            "".to_string()
        }
        .into();

        let open = state.is_open();
        let value = state.value();
        let focus_handle = self.state.focus_handle(cx);
        let focused = focus_handle.is_focused(window);
        let open_state = self.state.clone();
        let popover_state = self.state.clone();
        let disabled = self.disabled;

        BaseColorPicker::new(self.id.clone())
            .when(self.field, |this| this.w_full())
            .refine_style(&self.style)
            .open(open)
            .disabled(disabled)
            .track_focus(&focus_handle)
            .when_some(self.accessible_name(), |this, label| {
                this.accessibility_label(label)
            })
            .on_open_change(move |open, window, cx| {
                request_open(&open_state, open, window, cx);
            })
            .child(
                Popover::new("popover")
                    .open(open)
                    .anchor(self.anchor)
                    .map(|this| match self.panel_size() {
                        Size::Large => this.w_80(),
                        _ => this.w(rems(18.5)),
                    })
                    .on_open_change(move |open: &bool, window, cx| {
                        if !(disabled && *open) {
                            request_open(&popover_state, *open, window, cx);
                        }
                    })
                    .when(self.field, |this| {
                        this.trigger_style(StyleRefinement::default().w_full())
                    })
                    .trigger(ColorPickerButton {
                        id: "trigger".into(),
                        size: self.size,
                        label: self.label.clone(),
                        value,
                        tooltip: if display_title.is_empty() || self.field {
                            None
                        } else {
                            Some(display_title.clone())
                        },
                        icon: self.icon.clone(),
                        selected: false,
                        field: self.field,
                        focused,
                        disabled,
                        placeholder: self.placeholder.clone(),
                    })
                    // The popover drops its content while closed, so building the
                    // views then is wasted work.
                    .when(open, |this| {
                        this.child(self.render_panel(&editor, window, cx))
                    }),
            )
            .into_any_element()
    }
}

/// A color picker drawn as a framed field, like a [`Select`](crate::select::Select).
///
/// The field shows a swatch of the current color and its hex value; clicking
/// anywhere on it opens the same popover as [`ColorPicker`]. Use it in forms,
/// where a control is expected to share the height and frame of the inputs
/// around it; use [`ColorPicker`] for a compact swatch in a toolbar.
#[derive(IntoElement)]
pub struct ColorSelect {
    picker: ColorPicker,
}

impl ColorSelect {
    /// Create a new color select with the given [`ColorPickerState`].
    pub fn new(state: &Entity<ColorPickerState>) -> Self {
        let mut picker = ColorPicker::new(state);
        picker.id = ("color-select", state.entity_id()).into();
        picker.field = true;
        Self { picker }
    }

    /// Set the featured colors shown at the top of the palette.
    pub fn featured_colors(mut self, colors: Vec<Hsla>) -> Self {
        self.picker = self.picker.featured_colors(colors);
        self
    }

    /// Replace the default palette with named swatches.
    pub fn swatches(mut self, swatches: impl IntoIterator<Item = ColorSwatch>) -> Self {
        self.picker = self.picker.swatches(swatches);
        self
    }

    /// Choose the view shown first, see [`ColorPicker::default_mode`].
    pub fn default_mode(mut self, mode: ColorPickerMode) -> Self {
        self.picker = self.picker.default_mode(mode);
        self
    }

    /// Set the text shown while no color is selected.
    ///
    /// Default is the same placeholder as [`Select`](crate::select::Select).
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.picker.placeholder = Some(placeholder.into());
        self
    }

    /// Set the name a screen reader announces.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.picker = self.picker.accessibility_label(label);
        self
    }
}

impl Sizable for ColorSelect {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.picker = self.picker.with_size(size);
        self
    }
}

impl Disableable for ColorSelect {
    fn disabled(mut self, disabled: bool) -> Self {
        self.picker = self.picker.disabled(disabled);
        self
    }
}

impl Focusable for ColorSelect {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.picker.focus_handle(cx)
    }
}

impl Styled for ColorSelect {
    fn style(&mut self) -> &mut StyleRefinement {
        self.picker.style()
    }
}

impl RenderOnce for ColorSelect {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.picker.render(window, cx)
    }
}

#[derive(IntoElement)]
struct ColorPickerButton {
    id: ElementId,
    selected: bool,
    icon: Option<Icon>,
    value: Option<Hsla>,
    size: Size,
    label: Option<SharedString>,
    tooltip: Option<SharedString>,
    field: bool,
    focused: bool,
    disabled: bool,
    placeholder: Option<SharedString>,
}

impl Selectable for ColorPickerButton {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl Sizable for ColorPickerButton {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl ColorPickerButton {
    /// The framed trigger of a [`ColorSelect`]: the swatch, the hex value and
    /// a caret, laid out like a [`Select`](crate::select::Select) trigger.
    fn render_field(self, window: &mut Window, cx: &mut App) -> AnyElement {
        let (bg, fg) = input_style(self.disabled, cx);
        let outline_visible = self.selected || self.focused;

        h_flex()
            .id(self.id)
            .w_full()
            .gap_2()
            .bg(bg)
            .text_color(fg)
            .border_1()
            .border_color(cx.theme().input)
            .rounded(cx.theme().radius)
            .input_size(self.size)
            .input_text_size(self.size, cx)
            .when(self.disabled, |this| this.opacity(0.5))
            .when(outline_visible, |this| {
                this.border_color(cx.theme().ring)
                    .focus_ring_style(window, cx)
            })
            .child(
                div()
                    .id("swatch")
                    .flex_shrink_0()
                    .border_1()
                    .border_color(cx.theme().input)
                    .rounded(cx.theme().radius / 2.)
                    .map(|this| match self.size {
                        Size::XSmall => this.size_3(),
                        Size::Small => this.size_3p5(),
                        Size::Large => this.size_5(),
                        _ => this.size_4(),
                    })
                    .when_some(self.value, |this, value| {
                        this.bg(value).border_color(value.darken(0.3))
                    }),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .map(|this| match self.value {
                        Some(value) => this.child(hex_label(value)),
                        None => this.text_color(cx.theme().muted_foreground).child(
                            self.placeholder
                                .unwrap_or_else(|| t!("Select.placeholder").into()),
                        ),
                    }),
            )
            .child(Caret::new(self.size).text_color(cx.theme().muted_foreground))
            .into_any_element()
    }
}

impl RenderOnce for ColorPickerButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if self.field {
            return self.render_field(window, cx);
        }

        let has_icon = self.icon.is_some();
        h_flex()
            .id(self.id)
            .debug_selector(|| "color-picker-trigger".into())
            .gap_2()
            .when(self.disabled, |this| this.opacity(0.5))
            .children(self.icon)
            .when(!has_icon, |this| {
                this.child(
                    div()
                        .id("square")
                        .bg(cx.theme().tokens.background)
                        .border_1()
                        .border_color(cx.theme().input)
                        .rounded(cx.theme().radius)
                        .overflow_hidden()
                        .size_with(self.size)
                        .when_some(self.value, |this, value| {
                            this.bg(value)
                                .border_color(value.darken(0.3))
                                .when(self.selected, |this| this.border_2())
                        })
                        .when_some(self.tooltip, |this, tooltip| {
                            this.managed_tooltip(move |window, cx| {
                                Tooltip::new(tooltip.clone())
                                    .overlay_anchored()
                                    .build(window, cx)
                            })
                        }),
                )
            })
            .when_some(self.label, |this, label| this.child(label))
            .into_any_element()
    }
}
