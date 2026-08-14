use std::collections::HashMap;

use gpui::{
    App, Bounds, Entity, EntityId, FocusHandle, Global, Pixels, Point, WeakFocusHandle, Window,
};

use crate::text::{SelectionScope, TextViewState};

pub use gpui_base::GlobalState;

/// Signature of an app-registered TextView link-click interceptor.
pub(crate) type TextLinkHandler = std::rc::Rc<dyn Fn(&str, &mut Window, &mut App) -> bool>;

pub(crate) fn init(cx: &mut App) {
    // Preserve the legacy initialization point while `gpui_base::init` remains
    // after Root initialization for focus-trap ordering compatibility.
    GlobalState::init(cx);
    cx.set_global(UiGlobalState::new());
}

/// UI-only global state whose types cannot cross into `gpui-base`.
#[derive(Default)]
pub(crate) struct UiGlobalState {
    pub(crate) text_view_state_stack: Vec<Entity<TextViewState>>,
    selection_scope_stack: Vec<SelectionScope>,
    menu_focus_handles: Vec<WeakFocusHandle>,
    open_menu_bounds: HashMap<EntityId, Bounds<Pixels>>,
    pub(crate) text_link_handler: Option<TextLinkHandler>,
}

impl Global for UiGlobalState {}

impl UiGlobalState {
    fn new() -> Self {
        Self::default()
    }

    pub(crate) fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub(crate) fn global_mut(cx: &mut App) -> &mut Self {
        cx.default_global::<Self>()
    }

    pub(crate) fn text_view_state(&self) -> Option<&Entity<TextViewState>> {
        self.text_view_state_stack.last()
    }

    pub(crate) fn push_selection_scope(&mut self, scope: SelectionScope) {
        self.selection_scope_stack.push(scope);
    }

    pub(crate) fn pop_selection_scope(&mut self) {
        self.selection_scope_stack.pop();
    }

    pub(crate) fn current_selection_scope(&self) -> SelectionScope {
        self.selection_scope_stack
            .last()
            .copied()
            .unwrap_or(SelectionScope::Base)
    }

    pub(crate) fn update_menu_bounds(&mut self, id: EntityId, bounds: Bounds<Pixels>) {
        self.open_menu_bounds.insert(id, bounds);
    }

    pub(crate) fn remove_menu_bounds(&mut self, id: EntityId) {
        self.open_menu_bounds.remove(&id);
    }

    pub(crate) fn position_in_open_menu(&self, position: &Point<Pixels>) -> bool {
        self.open_menu_bounds
            .values()
            .any(|bounds| bounds.contains(position))
    }

    pub(crate) fn register_menu_focus_handle(&mut self, focus_handle: &FocusHandle) {
        self.menu_focus_handles
            .retain(|handle| handle.upgrade().is_some());
        self.menu_focus_handles.push(focus_handle.downgrade());
    }

    pub(crate) fn is_menu_focused(&self, window: &Window, cx: &App) -> bool {
        self.menu_focus_handles
            .iter()
            .filter_map(|handle| handle.upgrade())
            .any(|handle| handle.contains_focused(window, cx))
    }
}
