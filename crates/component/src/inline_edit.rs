//! Retained inline editing over the shared native input engine.

use std::rc::Rc;

use gpui::{
    App, AppContext as _, ClickEvent, Context, ElementId, Entity, EventEmitter, FocusHandle,
    Focusable, InteractiveElement as _, IntoElement, KeyBinding, KeyDownEvent, MouseDownEvent,
    ParentElement as _, Render, RenderOnce, Role, SharedString, StatefulInteractiveElement as _,
    StyleRefinement, Styled, Subscription, TextAlign, Window, div, prelude::FluentBuilder as _,
    transparent_black,
};

use crate::{
    ActiveTheme as _, Disableable, Sizable, Size, StyleSized as _, StyledExt as _,
    StyledTypography as _, TextSize, ThemeStyled as _,
    actions::{Cancel, Confirm},
    input::{Input, InputEvent, InputState},
    label::Label,
};

/// The key context of an inline edit, where [`init`] binds Escape to [`Cancel`].
pub(crate) const KEY_CONTEXT: &str = "InlineEdit";

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("escape", Cancel, Some(KEY_CONTEXT))]);
}

/// What losing focus does to an edit in progress.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InlineEditBlurBehavior {
    /// Keep the draft, if it is valid.
    #[default]
    Commit,
    /// Drop the draft.
    Cancel,
}

/// Events emitted by [`InlineEditState`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InlineEditEvent {
    /// An edit started.
    Begin,
    /// Only emitted when a successfully committed value changes.
    Commit(SharedString),
    /// An edit was dropped.
    Cancel,
    /// An edit ended, however it ended: after [`Commit`](Self::Commit) or
    /// [`Cancel`](Self::Cancel), and also when the committed text did not change. A host that
    /// only mounts the field while it is edited unmounts it on this.
    End,
}

type Validator = Rc<dyn Fn(&str) -> Result<(), SharedString>>;
type Normalizer = Rc<dyn Fn(&str) -> Option<SharedString>>;
type CommitHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
type CancelHandler = Rc<dyn Fn(&mut Window, &mut App)>;

struct Presentation {
    id: ElementId,
    label: SharedString,
    placeholder: SharedString,
    size: Size,
    text_size: Option<TextSize>,
    display: Option<SharedString>,
    borderless: bool,
    disabled: bool,
    readonly: bool,
    blur: InlineEditBlurBehavior,
    return_focus: Option<FocusHandle>,
    commit: Option<CommitHandler>,
    cancel: Option<CancelHandler>,
    style: StyleRefinement,
}

impl Default for Presentation {
    fn default() -> Self {
        Self {
            id: "inline-edit".into(),
            label: "".into(),
            placeholder: "Click to edit…".into(),
            size: Size::Medium,
            text_size: None,
            display: None,
            borderless: false,
            disabled: false,
            readonly: false,
            blur: InlineEditBlurBehavior::Commit,
            return_focus: None,
            commit: None,
            cancel: None,
            style: StyleRefinement::default(),
        }
    }
}

/// Owns committed text independently from the draft. The native editor and its subscriptions are
/// allocated on first edit, not for every displayed label.
pub struct InlineEditState {
    value: SharedString,
    input: Option<Entity<InputState>>,
    editing: bool,
    error: Option<SharedString>,
    focus: FocusHandle,
    validator: Option<Validator>,
    normalizer: Option<Normalizer>,
    presentation: Presentation,
    _events: Option<Subscription>,
}

impl InlineEditState {
    /// Create the state with its committed value.
    pub fn new(value: impl Into<SharedString>, _: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            value: value.into(),
            input: None,
            editing: false,
            error: None,
            focus: cx.focus_handle(),
            validator: None,
            normalizer: None,
            presentation: Presentation::default(),
            _events: None,
        }
    }

    /// Validate on commit rather than blocking native editing, paste, or IME.
    pub fn validate(
        mut self,
        validator: impl Fn(&str) -> Result<(), SharedString> + 'static,
    ) -> Self {
        self.validator = Some(Rc::new(validator));
        self
    }

    /// Turn the draft into the text to keep, before validation: trim it, or parse and
    /// reformat a number. `None` means the draft is not a value at all, and the edit then
    /// reverts quietly, as Escape does, with no error shown.
    pub fn normalize(
        mut self,
        normalizer: impl Fn(&str) -> Option<SharedString> + 'static,
    ) -> Self {
        self.normalizer = Some(Rc::new(normalizer));
        self
    }

    /// The committed value.
    pub fn value(&self) -> &SharedString {
        &self.value
    }

    /// Whether an edit is in progress.
    pub fn is_editing(&self) -> bool {
        self.editing
    }

    /// The message from the last failed commit, while the draft stays invalid.
    pub fn error(&self) -> Option<&SharedString> {
        self.error.as_ref()
    }

    /// The native editor, once the first edit has allocated it.
    pub fn input(&self) -> Option<&Entity<InputState>> {
        self.input.as_ref()
    }

    /// Replace the committed value. A new value drops a draft without a commit or cancel
    /// event. Setting the value it already has does nothing, so a host can mirror its model
    /// into the field on every render without disturbing an edit in progress.
    pub fn set_value(
        &mut self,
        value: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = value.into();
        if value == self.value {
            return;
        }
        self.value = value;
        if !self.discard(window, cx) {
            cx.notify();
        }
    }

    /// Drop a draft without a commit or cancel event, as when the host closes the surface
    /// around the field. Returns whether an edit was in progress.
    pub fn discard(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if !self.editing {
            return false;
        }
        let restore = self
            .input
            .as_ref()
            .is_some_and(|input| input.read(cx).focus_handle(cx).is_focused(window));
        self.editing = false;
        self.error = None;
        if let Some(input) = &self.input {
            input.update(cx, |input, cx| {
                input.set_value(self.value.clone(), window, cx)
            });
        }
        if restore {
            self.restore_focus(window, cx);
        }
        cx.emit(InlineEditEvent::End);
        cx.notify();
        true
    }

    /// Start editing, focusing the editor with the whole value selected. Re-entering an edit in
    /// progress keeps its draft and selection.
    pub fn begin(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(input) = self.open(window, cx) {
            input.update(cx, |input, cx| input.focus(window, cx));
        }
    }

    /// Start editing now and focus the editor on the next frame. For an edit started from a
    /// menu item or a command palette, which hands focus back to where it was as it closes
    /// and would otherwise end the edit at once.
    pub fn begin_deferred(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.open(window, cx).is_none() {
            return;
        }
        let state = cx.weak_entity();
        window.on_next_frame(move |window, cx| {
            let _ = state.update(cx, |state, cx| {
                if let Some(input) = state.input.clone().filter(|_| state.editing) {
                    input.update(cx, |input, cx| input.focus(window, cx));
                }
            });
        });
    }

    /// Put the field into editing with the whole value selected, and return the editor to
    /// focus. `None` when the field cannot be edited.
    fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<Entity<InputState>> {
        if self.presentation.disabled || self.presentation.readonly {
            return None;
        }
        if self.editing {
            return self.input.clone();
        }
        let input = match &self.input {
            Some(input) => input.clone(),
            None => {
                let input = cx.new(|cx| InputState::new(window, cx));
                self._events = Some(cx.subscribe_in(&input, window, Self::on_input_event));
                self.input = Some(input.clone());
                input
            }
        };
        self.editing = true;
        self.error = None;
        let placeholder = self.presentation.placeholder.clone();
        input.update(cx, |input, cx| {
            input.set_placeholder(placeholder, window, cx);
            input.set_value(self.value.clone(), window, cx);
            input.select_all(window, cx);
        });
        cx.emit(InlineEditEvent::Begin);
        cx.notify();
        Some(input)
    }

    fn on_input_event(
        &mut self,
        input: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.editing {
            return;
        }
        match event {
            InputEvent::PressEnter { .. } => {
                self.finish(true, window, cx);
            }
            InputEvent::Blur => self.end_by_blur(window, cx),
            InputEvent::Change if self.error.is_some() => {
                // Once a commit exposes an error, keep feedback accurate while the draft is
                // being corrected.
                let error = self
                    .validator
                    .as_ref()
                    .and_then(|validate| validate(&input.read(cx).value()).err());
                if self.error != error {
                    self.error = error;
                    cx.notify();
                }
            }
            _ => {}
        }
    }

    fn end_by_blur(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.presentation.blur {
            InlineEditBlurBehavior::Commit => {
                self.finish(false, window, cx);
            }
            InlineEditBlurBehavior::Cancel => self.cancel_inner(false, window, cx),
        }
    }

    /// Where focus goes when the keyboard ends an edit: the host's handle if it named one,
    /// otherwise the displayed value.
    fn restore_focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.presentation
            .return_focus
            .as_ref()
            .unwrap_or(&self.focus)
            .focus(window, cx);
    }

    /// Validate the draft and keep it. Returns whether the edit ended.
    pub fn commit(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.finish(true, window, cx)
    }

    fn finish(&mut self, restore_focus: bool, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if !self.editing {
            return false;
        }
        let Some(input) = &self.input else {
            return false;
        };
        let mut value = input.read(cx).value();
        if let Some(normalize) = &self.normalizer {
            match normalize(&value) {
                Some(normalized) => value = normalized,
                None => {
                    self.cancel_inner(restore_focus, window, cx);
                    return true;
                }
            }
        }
        if let Some(validator) = &self.validator
            && let Err(error) = validator(&value)
        {
            self.error = Some(error);
            cx.notify();
            return false;
        }
        self.editing = false;
        self.error = None;
        let changed = self.value != value;
        self.value = value.clone();
        if restore_focus {
            self.restore_focus(window, cx);
        }
        cx.notify();
        if changed {
            cx.emit(InlineEditEvent::Commit(value.clone()));
            if let Some(callback) = self.presentation.commit.clone() {
                // Application callbacks may read this field or start another edit.
                window.defer(cx, move |window, cx| callback(value, window, cx));
            }
        }
        cx.emit(InlineEditEvent::End);
        true
    }

    /// Drop the draft and return focus to the displayed value.
    pub fn cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.cancel_inner(true, window, cx);
    }

    fn cancel_inner(&mut self, restore_focus: bool, window: &mut Window, cx: &mut Context<Self>) {
        if !self.editing {
            return;
        }
        self.editing = false;
        self.error = None;
        if let Some(input) = &self.input {
            input.update(cx, |input, cx| {
                input.set_value(self.value.clone(), window, cx)
            });
        }
        if restore_focus {
            self.restore_focus(window, cx);
        }
        cx.emit(InlineEditEvent::Cancel);
        cx.emit(InlineEditEvent::End);
        cx.notify();
        if let Some(callback) = self.presentation.cancel.clone() {
            window.defer(cx, move |window, cx| callback(window, cx));
        }
    }

    fn render_editor(
        &self,
        align: Option<TextAlign>,
        cx: &App,
    ) -> Option<impl IntoElement + use<>> {
        let p = &self.presentation;
        let input = self.input.as_ref()?;
        Some(
            Input::new(input)
                .aria_label(p.label.clone())
                .with_size(p.size)
                .when_some(p.text_size, |input, size| input.text_ui_size(size, cx))
                // As in a file tree, the containing row owns its surface and focus border when
                // the edit is borderless; the editor contributes text and the caret.
                .when(p.borderless, |input| input.appearance(false).px_0())
                .when(self.error.is_some(), |input| {
                    input.border_color(cx.theme().danger)
                })
                .w_full()
                // The text stays where the display had it when editing starts.
                .map(|input| match align {
                    Some(TextAlign::Center) => input.text_center(),
                    Some(TextAlign::Right) => input.text_right(),
                    _ => input,
                }),
        )
    }

    fn render_display(
        &self,
        align: Option<TextAlign>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let p = &self.presentation;
        let is_empty = self.value.trim().is_empty();
        let value = if is_empty {
            p.placeholder.clone()
        } else {
            p.display.clone().unwrap_or_else(|| self.value.clone())
        };
        let shows_nothing = value.trim().is_empty();
        let interactive = !p.disabled && !p.readonly;
        let theme = cx.theme();
        let color = if p.disabled || is_empty {
            theme.muted_foreground
        } else {
            theme.foreground
        };
        let hover = theme.foreground.opacity(0.12);
        let radius = theme.radius;
        div()
            .id((p.id.clone(), "display"))
            .role(if p.readonly {
                Role::Label
            } else {
                Role::Button
            })
            // A blank value with a blank placeholder draws nothing, so the name names it.
            .aria_label(if shows_nothing || p.readonly {
                p.label.clone()
            } else {
                value.clone()
            })
            .when(interactive, |this| {
                this.aria_description(format!("{}. Click or press Enter or F2 to edit.", p.label))
                    .track_focus(&self.focus.clone().tab_stop(true))
                    .cursor_text()
                    .hover(move |style| style.bg(hover))
                    .on_click(
                        cx.listener(|this, _: &ClickEvent, window, cx| this.begin(window, cx)),
                    )
                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                        if !event.keystroke.modifiers.modified()
                            && matches!(event.keystroke.key.as_str(), "enter" | "space")
                        {
                            this.begin(window, cx);
                            cx.stop_propagation();
                        }
                    }))
            })
            .relative()
            .w_full()
            .min_w_0()
            .flex()
            .items_center()
            // The display and the editor share one frame, so starting an edit moves nothing.
            .input_h(p.size)
            .input_text_size(p.size, cx)
            .when_some(p.text_size, |this, size| this.text_ui_size(size, cx))
            .map(|this| {
                if p.borderless {
                    this.px_0()
                } else {
                    this.input_px(p.size).border_1()
                }
            })
            .border_color(transparent_black())
            .rounded(radius)
            .text_color(color)
            .map(|this| match align {
                Some(TextAlign::Center) => this.justify_center(),
                Some(TextAlign::Right) => this.justify_end(),
                _ => this,
            })
            .when(
                interactive && self.focus.is_focused(window) && window.last_input_was_keyboard(),
                |this| this.focus_ring_style(window, cx),
            )
            .child(div().min_w_0().truncate().child(value))
    }
}

impl EventEmitter<InlineEditEvent> for InlineEditState {}

impl Focusable for InlineEditState {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        match self.input.as_ref().filter(|_| self.editing) {
            Some(input) => input.read(cx).focus_handle(cx),
            None => self.focus.clone(),
        }
    }
}

impl Render for InlineEditState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The alignment the app set through `Styled`. The outer column hands it down to the
        // display, and the editor takes it from here.
        let align = self.presentation.style.text.text_align;
        let editor = self
            .editing
            .then(|| self.render_editor(align, cx))
            .flatten();
        let display = (!self.editing).then(|| self.render_display(align, window, cx));
        let id = self.presentation.id.clone();
        let danger = cx.theme().danger;
        let style = self.presentation.style.clone();
        div()
            .id(id)
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_1()
            .key_context(KEY_CONTEXT)
            // Escape drops the draft while editing, and otherwise passes on.
            .on_action(cx.listener(|this, _: &Cancel, window, cx| {
                if this.editing {
                    this.cancel(window, cx);
                } else {
                    cx.propagate();
                }
            }))
            // A dialog or popover around the edit binds Enter to confirm, and would act
            // on it before the edit commits.
            .when(self.editing, |this| this.on_action(|_: &Confirm, _, _| {}))
            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                match event.keystroke.key.as_str() {
                    "enter" if this.editing => {
                        // Own confirmation before the single-line editor's propagating Enter
                        // action reaches an enclosing form.
                        this.finish(true, window, cx);
                        cx.stop_propagation();
                    }
                    "f2" if !this.editing => {
                        this.begin(window, cx);
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }))
            // Any press outside ends the edit, committing or cancelling by the blur behavior.
            // GPUI leaves focus where it was when a press lands on nothing focusable.
            .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, window, cx| {
                if !this.editing {
                    return;
                }
                let focused = this
                    .input
                    .as_ref()
                    .is_some_and(|input| input.read(cx).focus_handle(cx).is_focused(window));
                this.end_by_blur(window, cx);
                // The editor is about to leave the tree. Left focused, it would strand the
                // keyboard on nothing until the next press on something focusable.
                if focused && !this.editing {
                    this.restore_focus(window, cx);
                }
            }))
            .children(editor)
            .children(display)
            .when_some(self.error.clone(), |column, error| {
                column.child(
                    div()
                        .id("inline-edit-error")
                        .role(Role::Alert)
                        .child(Label::new(error).text_ui_sm(cx).text_color(danger)),
                )
            })
            .refine_style(&style)
    }
}

/// A value shown as a click target that turns into a text field while it is edited.
///
/// The text alignment must be set on the InlineEdit itself through [`Styled`], such as
/// `text_center()` or `text_right()`. It then holds in the shown value, in the editor and in a
/// read-only value alike, so the text stays where it is when editing starts.
///
/// Escape while editing drops the draft. It comes as [`Cancel`], bound in the edit's own key
/// context, so it comes before a binding of Escape that the host makes around the edit, or a
/// popover's around it. With no edit under way, the action passes on.
#[derive(IntoElement)]
pub struct InlineEdit {
    state: Entity<InlineEditState>,
    presentation: Presentation,
}

impl InlineEdit {
    /// Create an inline edit bound to its retained state.
    pub fn new(id: impl Into<ElementId>, state: &Entity<InlineEditState>) -> Self {
        Self {
            state: state.clone(),
            presentation: Presentation {
                id: id.into(),
                ..Default::default()
            },
        }
    }

    /// Name the field for assistive technology, such as "Project name". Required. It names the
    /// editor and describes the display, and is never drawn, not even as the placeholder.
    pub fn accessibility_label(mut self, value: impl Into<SharedString>) -> Self {
        self.presentation.label = value.into();
        self
    }

    /// Text shown while the value is blank.
    pub fn placeholder(mut self, value: impl Into<SharedString>) -> Self {
        self.presentation.placeholder = value.into();
        self
    }

    /// Show this text in place of the value while it is not being edited, such as "+3.0 dB"
    /// for a value edited as "3.0". A blank value still shows the placeholder.
    pub fn display(mut self, value: impl Into<SharedString>) -> Self {
        self.presentation.display = Some(value.into());
        self
    }

    /// Set the text role apart from the control size, for the shown value and the editor
    /// alike, so a field in a row can match the row's own text.
    pub fn text_ui_size(mut self, size: TextSize) -> Self {
        self.presentation.text_size = Some(size);
        self
    }

    /// Embed an editor directly in a row: transparent surface, no field border or horizontal
    /// inset, with the same native input height and typography. The container can supply
    /// selection and focus styling. Validation still exposes its message below the value.
    pub fn borderless(mut self, value: bool) -> Self {
        self.presentation.borderless = value;
        self
    }

    /// Show the value without letting it be edited, keeping it readable.
    pub fn readonly(mut self, value: bool) -> Self {
        self.presentation.readonly = value;
        self
    }

    /// Send focus here when Enter or Escape ends an edit, in place of the displayed value: to
    /// the surface around the field, so its own keys work again at once.
    pub fn return_focus(mut self, handle: &FocusHandle) -> Self {
        self.presentation.return_focus = Some(handle.clone());
        self
    }

    /// Set what losing focus does to a draft, default: commit.
    pub fn blur_behavior(mut self, value: InlineEditBlurBehavior) -> Self {
        self.presentation.blur = value;
        self
    }

    /// Run this after a commit changes the value.
    pub fn on_commit(
        mut self,
        callback: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.presentation.commit = Some(Rc::new(callback));
        self
    }

    /// Run this after an edit is dropped.
    pub fn on_cancel(mut self, callback: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.presentation.cancel = Some(Rc::new(callback));
        self
    }
}

impl Disableable for InlineEdit {
    fn disabled(mut self, disabled: bool) -> Self {
        self.presentation.disabled = disabled;
        self
    }
}

impl Sizable for InlineEdit {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.presentation.size = size.into();
        self
    }
}

impl Styled for InlineEdit {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.presentation.style
    }
}

impl RenderOnce for InlineEdit {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        debug_assert!(
            !self.presentation.label.trim().is_empty(),
            "InlineEdit needs an accessibility label"
        );
        self.state.update(cx, |state, cx| {
            let old = &state.presentation;
            let new = &self.presentation;
            let changed = old.id != new.id
                || old.label != new.label
                || old.placeholder != new.placeholder
                || old.size != new.size
                || old.text_size != new.text_size
                || old.display != new.display
                || old.borderless != new.borderless
                || old.disabled != new.disabled
                || old.readonly != new.readonly
                || old.style != new.style;
            if state.editing && (self.presentation.disabled || self.presentation.readonly) {
                let restore = state
                    .input
                    .as_ref()
                    .is_some_and(|input| input.read(cx).focus_handle(cx).is_focused(window));
                // A host property change is not a user cancellation. In particular, do not
                // invoke a host callback while it renders.
                state.discard(window, cx);
                if restore {
                    window.blur(cx);
                }
            }
            state.presentation = self.presentation;
            if changed {
                cx.notify();
            }
        });
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::{InlineEdit, InlineEditBlurBehavior, InlineEditEvent, InlineEditState};
    use crate::actions::Confirm;
    use crate::{Disableable as _, Sizable as _, Size};
    use gpui::{
        Context, Entity, FocusHandle, Focusable, IntoElement, Modifiers, Render, Subscription,
        TestAppContext, VisualTestContext, Window, div, prelude::*, px,
    };
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    struct Probe {
        state: Entity<InlineEditState>,
        blur: InlineEditBlurBehavior,
        size: Size,
        borderless: bool,
        disabled: bool,
        readonly: bool,
        outside: FocusHandle,
        _events: Subscription,
    }

    impl Render for Probe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .w(px(300.))
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(
                    div().debug_selector(|| "edit".into()).child(
                        InlineEdit::new("name", &self.state)
                            .accessibility_label("Name")
                            .with_size(self.size)
                            .borderless(self.borderless)
                            .disabled(self.disabled)
                            .readonly(self.readonly)
                            .blur_behavior(self.blur),
                    ),
                )
                .child(
                    div()
                        .id("outside")
                        .h(px(20.))
                        .debug_selector(|| "outside".into())
                        .track_focus(&self.outside),
                )
        }
    }

    type Events = Rc<RefCell<Vec<InlineEditEvent>>>;

    fn mount(
        cx: &mut TestAppContext,
        validate: bool,
    ) -> (Entity<Probe>, &mut VisualTestContext, Events) {
        cx.update(crate::init);
        let events: Events = Rc::new(RefCell::new(Vec::new()));
        let collected = events.clone();
        let (probe, visual) = cx.add_window_view(move |window, cx| {
            let state = cx.new(|cx| {
                let state = InlineEditState::new("Original", window, cx);
                if validate {
                    state.validate(|value| {
                        if value.trim().is_empty() {
                            Err("Required".into())
                        } else {
                            Ok(())
                        }
                    })
                } else {
                    state
                }
            });
            let subscription = cx.subscribe(&state, move |_, _, event: &InlineEditEvent, _| {
                collected.borrow_mut().push(event.clone())
            });
            Probe {
                state,
                blur: InlineEditBlurBehavior::Commit,
                size: Size::Medium,
                borderless: false,
                disabled: false,
                readonly: false,
                outside: cx.focus_handle(),
                _events: subscription,
            }
        });
        // GPUI dispatches native focus/blur subscriptions only in an active window.
        visual.update(|window, _| window.activate_window());
        visual.simulate_resize(gpui::size(px(400.), px(300.)));
        visual.run_until_parked();
        draw(visual);
        (probe, visual, events)
    }

    /// The last event other than the `End` that closes every edit.
    fn last_outcome(events: &Events) -> Option<InlineEditEvent> {
        events
            .borrow()
            .iter()
            .rev()
            .find(|event| **event != InlineEditEvent::End)
            .cloned()
    }

    /// A numeric field: edited as a plain number, shown with its unit.
    struct Gain {
        state: Entity<InlineEditState>,
        events: Events,
        _events: Subscription,
    }

    impl Render for Gain {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let shown = format!("{} dB", self.state.read(cx).value());
            div().w(px(120.)).child(
                div().debug_selector(|| "gain".into()).child(
                    InlineEdit::new("gain", &self.state)
                        .accessibility_label("Gain")
                        .display(shown)
                        .xsmall()
                        .borderless(true),
                ),
            )
        }
    }

    fn mount_gain(cx: &mut TestAppContext) -> (Entity<Gain>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (gain, visual) = cx.add_window_view(|window, cx| {
            let state = cx.new(|cx| {
                InlineEditState::new("3.0", window, cx).normalize(|draft| {
                    let value: f32 = draft.trim().parse().ok()?;
                    Some(format!("{:.1}", value.clamp(-12., 12.)).into())
                })
            });
            let events: Events = Rc::new(RefCell::new(Vec::new()));
            let collected = events.clone();
            let subscription = cx.subscribe(&state, move |_, _, event: &InlineEditEvent, _| {
                collected.borrow_mut().push(event.clone())
            });
            Gain {
                state,
                events,
                _events: subscription,
            }
        });
        visual.update(|window, _| window.activate_window());
        visual.simulate_resize(gpui::size(px(400.), px(300.)));
        visual.run_until_parked();
        draw(visual);
        (gain, visual)
    }

    #[gpui::test]
    fn a_normalizer_keeps_its_own_text_and_quietly_reverts_what_is_not_a_value(
        cx: &mut TestAppContext,
    ) {
        let (gain, visual) = mount_gain(cx);
        let (state, events) =
            visual.update(|_, cx| (gain.read(cx).state.clone(), gain.read(cx).events.clone()));

        begin(&state, visual);
        edit(&state, " 40 ", visual);
        visual.simulate_keystrokes("enter");
        draw(visual);
        visual.update(|_, cx| assert_eq!(state.read(cx).value(), "12.0"));
        assert_eq!(
            events.borrow()[..],
            [
                InlineEditEvent::Begin,
                InlineEditEvent::Commit("12.0".into()),
                InlineEditEvent::End
            ]
        );

        events.borrow_mut().clear();
        begin(&state, visual);
        edit(&state, "loud", visual);
        visual.simulate_keystrokes("enter");
        draw(visual);
        visual.update(|_, cx| {
            let state = state.read(cx);
            assert_eq!(state.value(), "12.0");
            assert!(!state.is_editing());
            assert!(state.error().is_none());
        });
        assert_eq!(
            events.borrow()[..],
            [
                InlineEditEvent::Begin,
                InlineEditEvent::Cancel,
                InlineEditEvent::End
            ]
        );

        // A draft that normalizes to the committed text ends the edit and changes nothing.
        events.borrow_mut().clear();
        begin(&state, visual);
        edit(&state, "12", visual);
        visual.simulate_keystrokes("enter");
        draw(visual);
        assert_eq!(
            events.borrow()[..],
            [InlineEditEvent::Begin, InlineEditEvent::End]
        );
    }

    #[gpui::test]
    fn mirroring_the_same_value_keeps_a_draft_and_a_new_value_drops_it(cx: &mut TestAppContext) {
        let (gain, visual) = mount_gain(cx);
        let state = visual.update(|_, cx| gain.read(cx).state.clone());
        begin(&state, visual);
        edit(&state, "6", visual);
        visual.update(|window, cx| {
            state.update(cx, |state, cx| state.set_value("3.0", window, cx));
        });
        draw(visual);
        visual.update(|_, cx| {
            let state = state.read(cx);
            assert!(state.is_editing());
            assert_eq!(state.input().unwrap().read(cx).value(), "6");
        });

        visual.update(|window, cx| {
            state.update(cx, |state, cx| state.set_value("-2.0", window, cx));
        });
        draw(visual);
        visual.update(|_, cx| {
            let state = state.read(cx);
            assert!(!state.is_editing());
            assert_eq!(state.value(), "-2.0");
        });
    }

    #[gpui::test]
    fn a_deferred_begin_edits_at_once_and_takes_focus_on_the_next_frame(cx: &mut TestAppContext) {
        let (gain, visual) = mount_gain(cx);
        let state = visual.update(|_, cx| gain.read(cx).state.clone());
        visual.update(|window, cx| {
            state.update(cx, |state, cx| state.begin_deferred(window, cx));
            assert!(state.read(cx).is_editing());
            let input = state.read(cx).input().unwrap().clone();
            assert!(!input.read(cx).focus_handle(cx).is_focused(window));
        });
        draw(visual);
        visual.update(|window, cx| {
            window.simulate_next_frame(cx);
        });
        draw(visual);
        visual.update(|window, cx| {
            let input = state.read(cx).input().unwrap().clone();
            assert!(input.read(cx).focus_handle(cx).is_focused(window));
        });
    }

    /// A name in a row: compact field geometry with the row's own text role.
    struct RowName(Entity<InlineEditState>);

    impl Render for RowName {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(300.)).child(
                InlineEdit::new("name", &self.0)
                    .accessibility_label("Name")
                    .xsmall()
                    .borderless(true)
                    .text_ui_size(crate::TextSize::Default),
            )
        }
    }

    /// The editor draws its text at the role the host named, not at the control size's own,
    /// and follows that role when the application changes its sizes.
    #[gpui::test]
    fn the_editor_takes_the_named_text_role_at_every_text_size(cx: &mut TestAppContext) {
        use crate::{TextSize, Theme, UiTypography, input::Position};

        cx.update(crate::init);
        let text = "Rename 世界";
        let (row, visual) = cx.add_window_view(|window, cx| {
            RowName(cx.new(|cx| InlineEditState::new(text, window, cx)))
        });
        visual.update(|window, _| window.activate_window());
        let state = visual.update(|_, cx| row.read(cx).0.clone());
        begin(&state, visual);
        let input = visual.update(|_, cx| state.read(cx).input().unwrap().clone());
        for default_px in [13., 15., 12.] {
            visual.update(|window, cx| {
                Theme::update(cx, |theme| {
                    theme.set_ui_typography(
                        UiTypography::default()
                            .with_size(TextSize::Default, gpui::rems(default_px / 16.)),
                    )
                });
                input.update(cx, |input, cx| {
                    input.set_cursor_position(Position::new(0, 0), window, cx)
                });
                window.draw(cx).clear(cx);
                let start = input.read(cx).cursor_layout().unwrap().0.left();
                input.update(cx, |input, cx| input.select_all(window, cx));
                window.draw(cx).clear(cx);
                let end = input.read(cx).cursor_layout().unwrap().0.left();
                let mut style = window.text_style();
                style.font_family = Theme::global(cx).font_family.clone();
                let label = window
                    .text_system()
                    .shape_line(
                        text.into(),
                        TextSize::Default.to_pixels(window, cx),
                        &[style.to_run(text.len())],
                        None,
                    )
                    .width();
                assert!(
                    ((end - start) - label).abs() <= px(1.),
                    "caret advance {:?} must match the label {label:?} at {default_px}px",
                    end - start,
                );
            });
        }
    }

    fn draw(visual: &mut VisualTestContext) {
        visual.update(|window, cx| window.draw(cx).clear(cx));
    }

    fn begin(state: &Entity<InlineEditState>, visual: &mut VisualTestContext) {
        visual.update(|window, cx| state.update(cx, |state, cx| state.begin(window, cx)));
        draw(visual);
    }

    fn edit(state: &Entity<InlineEditState>, value: &str, visual: &mut VisualTestContext) {
        visual.update(|window, cx| {
            let input = state.read(cx).input().unwrap().clone();
            input.update(cx, |input, cx| {
                input.set_value(value.to_string(), window, cx)
            });
        });
        draw(visual);
    }

    fn commits(events: &Events) -> usize {
        events
            .borrow()
            .iter()
            .filter(|event| matches!(event, InlineEditEvent::Commit(_)))
            .count()
    }

    #[gpui::test]
    fn the_editor_is_lazy_commits_once_cancels_and_restores_focus(cx: &mut TestAppContext) {
        let (probe, visual, events) = mount(cx, false);
        let state = visual.update(|_, cx| probe.read(cx).state.clone());
        visual.update(|_, cx| assert!(state.read(cx).input().is_none()));
        let bounds = visual.debug_bounds("edit").unwrap();
        visual.simulate_click(bounds.center(), Modifiers::default());
        draw(visual);
        visual.update(|window, cx| {
            assert!(state.read(cx).is_editing());
            assert!(state.focus_handle(cx).is_focused(window));
        });
        edit(&state, "Saved", visual);
        visual.simulate_keystrokes("enter");
        draw(visual);
        visual.update(|window, cx| {
            assert_eq!(state.read(cx).value(), "Saved");
            assert!(!state.read(cx).is_editing());
            assert!(state.focus_handle(cx).is_focused(window));
        });
        assert_eq!(commits(&events), 1);

        visual.simulate_keystrokes("f2");
        draw(visual);
        visual.update(|_, cx| assert!(state.read(cx).is_editing()));
        edit(&state, "Discarded", visual);
        visual.simulate_keystrokes("escape");
        draw(visual);
        visual.update(|window, cx| {
            assert_eq!(state.read(cx).value(), "Saved");
            assert!(!state.read(cx).is_editing());
            assert!(state.focus_handle(cx).is_focused(window));
        });
        assert_eq!(last_outcome(&events), Some(InlineEditEvent::Cancel));

        // Enter on the display opens the edit; a second Enter closes it unchanged.
        visual.simulate_keystrokes("enter");
        draw(visual);
        visual.update(|_, cx| assert!(state.read(cx).is_editing()));
        visual.simulate_keystrokes("enter");
        draw(visual);
        visual.update(|_, cx| assert!(!state.read(cx).is_editing()));
        assert_eq!(commits(&events), 1, "an unchanged commit emits nothing");
    }

    #[gpui::test]
    fn commit_and_cancel_callbacks_can_touch_the_retained_state(cx: &mut TestAppContext) {
        struct CallbackProbe {
            state: Entity<InlineEditState>,
            commits: usize,
            cancellations: usize,
        }
        impl Render for CallbackProbe {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let commit_owner = cx.weak_entity();
                let cancel_owner = cx.weak_entity();
                InlineEdit::new("callback-name", &self.state)
                    .accessibility_label("Name")
                    .on_commit(move |value, _, cx| {
                        commit_owner
                            .update(cx, |owner, cx| {
                                // Saving a rename commonly reads the owner's retained field.
                                assert_eq!(*owner.state.read(cx).value(), value);
                                owner.commits += 1;
                            })
                            .unwrap();
                    })
                    .on_cancel(move |window, cx| {
                        cancel_owner
                            .update(cx, |owner, cx| {
                                owner.cancellations += 1;
                                assert_eq!(owner.state.read(cx).value(), "Renamed");
                                owner.state.update(cx, |state, cx| state.begin(window, cx));
                            })
                            .unwrap();
                    })
            }
        }
        cx.update(crate::init);
        let (probe, visual) = cx.add_window_view(|window, cx| {
            window.activate_window();
            CallbackProbe {
                state: cx.new(|cx| InlineEditState::new("Original", window, cx)),
                commits: 0,
                cancellations: 0,
            }
        });
        draw(visual);
        let state = visual.update(|_, cx| probe.read(cx).state.clone());
        begin(&state, visual);
        edit(&state, "Renamed", visual);
        visual.simulate_keystrokes("enter");
        visual.run_until_parked();
        visual.update(|_, cx| {
            assert_eq!(state.read(cx).value(), "Renamed");
            assert_eq!(probe.read(cx).commits, 1);
        });

        begin(&state, visual);
        edit(&state, "Discarded", visual);
        visual.simulate_keystrokes("escape");
        visual.run_until_parked();
        draw(visual);
        visual.update(|window, cx| {
            assert_eq!(probe.read(cx).cancellations, 1);
            let input = state.read(cx).input().unwrap();
            assert_eq!(input.read(cx).value(), "Renamed");
            assert!(input.read(cx).focus_handle(cx).is_focused(window));
        });
    }

    #[gpui::test]
    fn blur_follows_its_policy_and_validation_keeps_the_draft(cx: &mut TestAppContext) {
        let (probe, visual, events) = mount(cx, true);
        let state = visual.update(|_, cx| probe.read(cx).state.clone());
        let outside = visual.update(|_, cx| probe.read(cx).outside.clone());

        begin(&state, visual);
        edit(&state, "Blurred", visual);
        visual.update(|window, cx| outside.focus(window, cx));
        draw(visual);
        visual.update(|_, cx| {
            assert_eq!(state.read(cx).value(), "Blurred");
            assert!(!state.read(cx).is_editing());
        });

        visual.update(|_, cx| {
            probe.update(cx, |probe, cx| {
                probe.blur = InlineEditBlurBehavior::Cancel;
                cx.notify();
            })
        });
        draw(visual);
        begin(&state, visual);
        edit(&state, "Dropped", visual);
        visual.update(|window, cx| outside.focus(window, cx));
        draw(visual);
        visual.update(|_, cx| {
            assert_eq!(state.read(cx).value(), "Blurred");
            assert!(!state.read(cx).is_editing());
        });
        assert_eq!(last_outcome(&events), Some(InlineEditEvent::Cancel));

        // An invalid draft stays open with its error, which tracks corrections.
        begin(&state, visual);
        edit(&state, "   ", visual);
        visual.simulate_keystrokes("enter");
        draw(visual);
        visual.update(|_, cx| {
            assert!(state.read(cx).is_editing());
            assert_eq!(state.read(cx).error().map(|e| e.as_ref()), Some("Required"));
            assert_eq!(state.read(cx).value(), "Blurred");
        });
        // Typing, unlike a programmatic set, is what reports a change.
        visual.simulate_input("Fixed");
        draw(visual);
        visual.update(|_, cx| assert!(state.read(cx).error().is_none()));
        visual.simulate_keystrokes("enter");
        draw(visual);
        visual.update(|_, cx| assert_eq!(state.read(cx).value(), "   Fixed"));
    }

    #[gpui::test]
    fn unavailable_states_do_not_edit_and_end_an_edit_in_progress(cx: &mut TestAppContext) {
        let (probe, visual, events) = mount(cx, false);
        let state = visual.update(|_, cx| probe.read(cx).state.clone());
        for (disabled, readonly) in [(true, false), (false, true)] {
            visual.update(|_, cx| {
                probe.update(cx, |probe, cx| {
                    probe.disabled = disabled;
                    probe.readonly = readonly;
                    cx.notify();
                })
            });
            draw(visual);
            let bounds = visual.debug_bounds("edit").unwrap();
            visual.simulate_click(bounds.center(), Modifiers::default());
            begin(&state, visual);
            visual.update(|_, cx| assert!(!state.read(cx).is_editing()));

            visual.update(|_, cx| {
                probe.update(cx, |probe, cx| {
                    probe.disabled = false;
                    probe.readonly = false;
                    cx.notify();
                })
            });
            draw(visual);
            begin(&state, visual);
            edit(&state, "Draft", visual);
            let count = events.borrow().len();
            visual.update(|_, cx| {
                probe.update(cx, |probe, cx| {
                    probe.disabled = disabled;
                    probe.readonly = readonly;
                    cx.notify();
                })
            });
            draw(visual);
            visual.update(|_, cx| {
                assert!(!state.read(cx).is_editing());
                assert_eq!(state.read(cx).value(), "Original");
            });
            assert_eq!(
                events.borrow()[count..],
                [InlineEditEvent::End],
                "not a user cancellation"
            );
            visual.update(|_, cx| {
                probe.update(cx, |probe, cx| {
                    probe.disabled = false;
                    probe.readonly = false;
                    cx.notify();
                })
            });
            draw(visual);
        }
    }

    #[gpui::test]
    fn the_display_and_the_editor_keep_one_height(cx: &mut TestAppContext) {
        let (probe, visual, _) = mount(cx, false);
        let state = visual.update(|_, cx| probe.read(cx).state.clone());
        for size in [Size::XSmall, Size::Small, Size::Medium, Size::Large] {
            for borderless in [false, true] {
                visual.update(|_, cx| {
                    probe.update(cx, |probe, cx| {
                        probe.size = size;
                        probe.borderless = borderless;
                        cx.notify();
                    })
                });
                draw(visual);
                let display = visual.debug_bounds("edit").unwrap();
                begin(&state, visual);
                let editor = visual.debug_bounds("edit").unwrap();
                assert_eq!(
                    display.size.height, editor.size.height,
                    "{size:?} borderless={borderless}"
                );
                visual.simulate_keystrokes("escape");
                draw(visual);
            }
        }
    }

    #[gpui::test]
    fn a_press_on_empty_space_ends_the_edit_by_its_blur_behavior(cx: &mut TestAppContext) {
        let (probe, visual, _) = mount(cx, false);
        let state = visual.update(|_, cx| probe.read(cx).state.clone());
        let empty = gpui::point(px(380.), px(280.));
        for (blur, typed, expected) in [
            (InlineEditBlurBehavior::Commit, "Accepted", "Accepted"),
            (InlineEditBlurBehavior::Cancel, "Discarded", "Accepted"),
        ] {
            visual.update(|_, cx| {
                probe.update(cx, |probe, cx| {
                    probe.blur = blur;
                    cx.notify();
                })
            });
            draw(visual);
            begin(&state, visual);
            edit(&state, typed, visual);
            visual.simulate_click(empty, Modifiers::default());
            draw(visual);
            visual.update(|window, cx| {
                assert!(!state.read(cx).is_editing(), "{blur:?}");
                assert_eq!(state.read(cx).value(), expected, "{blur:?}");
                // The hidden editor does not keep the keyboard.
                assert!(state.focus_handle(cx).is_focused(window), "{blur:?}");
            });
        }
    }

    gpui::actions!(inline_edit_tests, [HostEscape]);

    struct EscapeHost {
        state: Entity<InlineEditState>,
        runs: Rc<Cell<usize>>,
    }

    impl Render for EscapeHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let runs = self.runs.clone();
            div()
                .key_context("Host")
                .on_action(move |_: &HostEscape, _, _| runs.set(runs.get() + 1))
                .child(InlineEdit::new("name", &self.state).accessibility_label("Name"))
        }
    }

    struct ConfirmHost {
        state: Entity<InlineEditState>,
        runs: Rc<Cell<usize>>,
    }

    impl Render for ConfirmHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let runs = self.runs.clone();
            div()
                .key_context("Host")
                .on_action(move |_: &Confirm, _, _| runs.set(runs.get() + 1))
                .child(InlineEdit::new("name", &self.state).accessibility_label("Name"))
        }
    }

    #[gpui::test]
    fn enter_commits_the_edit_without_confirming_the_host(cx: &mut TestAppContext) {
        cx.update(|cx| {
            crate::init(cx);
            cx.bind_keys([gpui::KeyBinding::new(
                "enter",
                Confirm { secondary: false },
                Some("Host"),
            )]);
        });
        let runs = Rc::new(Cell::new(0));
        let host_runs = runs.clone();
        let (view, visual) = cx.add_window_view(move |window, cx| {
            window.activate_window();
            ConfirmHost {
                state: cx.new(|cx| InlineEditState::new("Original", window, cx)),
                runs: host_runs,
            }
        });
        visual.simulate_resize(gpui::size(px(400.), px(300.)));
        draw(visual);
        let state = visual.update(|_, cx| view.read(cx).state.clone());
        begin(&state, visual);
        edit(&state, "Renamed", visual);
        draw(visual);

        visual.simulate_keystrokes("enter");
        draw(visual);
        visual.update(|_, cx| {
            assert!(!state.read(cx).is_editing());
            assert_eq!(state.read(cx).value().as_ref(), "Renamed");
        });
        assert_eq!(runs.get(), 0, "Enter belongs to the edit while it is open");
    }

    #[gpui::test]
    fn escape_drops_the_draft_before_a_binding_around_the_edit(cx: &mut TestAppContext) {
        cx.update(|cx| {
            crate::init(cx);
            cx.bind_keys([gpui::KeyBinding::new("escape", HostEscape, Some("Host"))]);
        });
        let runs = Rc::new(Cell::new(0));
        let host_runs = runs.clone();
        let (view, visual) = cx.add_window_view(move |window, cx| {
            window.activate_window();
            EscapeHost {
                state: cx.new(|cx| InlineEditState::new("Original", window, cx)),
                runs: host_runs,
            }
        });
        visual.simulate_resize(gpui::size(px(400.), px(300.)));
        draw(visual);
        let state = visual.update(|_, cx| view.read(cx).state.clone());
        begin(&state, visual);
        edit(&state, "Discarded", visual);

        visual.simulate_keystrokes("escape");
        draw(visual);
        visual.update(|window, cx| {
            assert!(!state.read(cx).is_editing(), "Escape drops the draft");
            assert_eq!(state.read(cx).value(), "Original");
            assert!(state.focus_handle(cx).is_focused(window));
        });
        assert_eq!(runs.get(), 0, "the host's binding does not run");

        visual.simulate_keystrokes("escape");
        draw(visual);
        assert_eq!(
            runs.get(),
            1,
            "with no edit under way, Escape is the host's"
        );
    }

    #[gpui::test]
    fn a_read_only_value_is_not_a_tab_stop(cx: &mut TestAppContext) {
        let (probe, visual, _) = mount(cx, false);
        let state = visual.update(|_, cx| probe.read(cx).state.clone());
        visual.update(|_, cx| {
            probe.update(cx, |probe, cx| {
                probe.readonly = true;
                cx.notify();
            })
        });
        draw(visual);
        visual.update(|window, cx| {
            let handle = state.read(cx).focus_handle(cx);
            window.focus_next(cx);
            assert!(!handle.is_focused(window));
        });
    }
}
