use gpui::{App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable, Window};

use crate::list::{ListDelegate, ListState};

/// Events a [`PickerState`] emits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerEvent {
    /// The picker opened, however that happened: a trigger click, a key on
    /// the trigger, or [`PickerState::open`]. The query and the keyboard
    /// cursor are already cleared; reload or refilter rows here.
    Open,
}

/// The state behind a [`Picker`](super::Picker): its [`ListState`] and its
/// open flag.
///
/// Every open clears the List's query (when searchable) and its keyboard
/// cursor, then emits [`PickerEvent::Open`].
///
/// A host rendered inside a cached view must observe the `PickerState` (or
/// forward its notifications), because a close made from code changes only the
/// state entity.
pub struct PickerState<D: ListDelegate> {
    list: Entity<ListState<D>>,
    searchable: bool,
    open: bool,
}

impl<D: ListDelegate> PickerState<D> {
    pub fn new(delegate: D, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let list = cx.new(|cx| ListState::new(delegate, window, cx));
        Self {
            list,
            searchable: false,
            open: false,
        }
    }

    /// Turns on the List's own query field, drawn at the top of the picker.
    /// Default `false`.
    pub fn searchable(mut self, searchable: bool, cx: &mut App) -> Self {
        self.searchable = searchable;
        self.list
            .update(cx, |list, cx| list.set_searchable(searchable, cx));
        self
    }

    /// The List state, for the owner's delegate work.
    pub fn list(&self) -> &Entity<ListState<D>> {
        &self.list
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub(super) fn is_searchable(&self) -> bool {
        self.searchable
    }

    /// Opens from code, with the same reset and [`PickerEvent::Open`] as a
    /// trigger click.
    ///
    /// The List (or its query field) takes focus on the next frame through
    /// the popover's tracked handle, which also records where focus was, so
    /// a close returns focus to where it was before the open.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_open(true, window, cx);
    }

    /// Closes from code. The popover's closing frame restores the focus it
    /// had before opening.
    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.write_open(false, cx);
    }

    /// Closes because the list just emptied and its trigger is about to
    /// disappear without a closing frame, so the popover cannot restore focus
    /// itself. Focuses `focus` when given; otherwise focus is left where it is.
    pub fn close_emptied(
        &mut self,
        focus: Option<&FocusHandle>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.close(cx);
        if let Some(focus) = focus {
            focus.focus(window, cx);
        }
    }

    /// The popover's own open changes (both ways) and [`open`](Self::open)
    /// land here. The flag goes through [`write_open`](Self::write_open), the
    /// one writer of `open`, so a redundant write is ignored and one open
    /// emits one [`PickerEvent::Open`].
    pub(super) fn set_open(&mut self, open: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.write_open(open, cx) && open {
            let searchable = self.searchable;
            self.list.update(cx, |list, cx| {
                // `set_query` starts a search, so the delegate refilters to
                // every row, and the search moves the cursor to the first row;
                // the cursor is cleared after it.
                if searchable {
                    list.set_query("", window, cx);
                }
                list.set_selected_index(None, window, cx);
            });
            cx.emit(PickerEvent::Open);
        }
    }

    /// The one writer of the open flag. Skips a redundant write, notifies,
    /// and returns whether the value changed.
    fn write_open(&mut self, open: bool, cx: &mut Context<Self>) -> bool {
        if self.open == open {
            return false;
        }
        self.open = open;
        cx.notify();
        true
    }
}

impl<D: ListDelegate> EventEmitter<PickerEvent> for PickerState<D> {}

/// The List's focus handle (its query field's when searchable). The
/// [`Picker`](super::Picker) gives it to the popover, so ↑/↓/Enter/Escape
/// reach the List.
impl<D: ListDelegate> Focusable for PickerState<D> {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.list.read(cx).focus_handle(cx)
    }
}
