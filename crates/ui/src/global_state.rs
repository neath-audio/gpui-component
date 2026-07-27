use gpui::{
    App, Bounds, ElementId, Entity, EntityId, FocusHandle, Global, OwnedMenu, Pixels, Point,
    WeakFocusHandle, Window,
};
use std::collections::{HashMap, HashSet};

use crate::text::{SelectionScope, TextViewState};

pub(crate) fn init(cx: &mut App) {
    cx.set_global(GlobalState::new());
}

impl Global for GlobalState {}

/// Signature of an app-registered TextView link-click interceptor. `Rc`, not
/// `Arc` — gpui globals are main-thread and the handler will typically
/// capture a gpui `WeakEntity`, which is not `Send`.
pub(crate) type TextLinkHandler = std::rc::Rc<dyn Fn(&str, &mut Window, &mut App) -> bool>;

pub struct GlobalState {
    pub(crate) text_view_state_stack: Vec<Entity<TextViewState>>,
    /// Set of open popover IDs that use deferred rendering.
    /// When this set is not empty, we are inside at least one deferred context.
    /// This is used to prevent double-deferred elements which would cause GPUI to panic.
    open_deferred_popovers: HashSet<ElementId>,
    /// Application menus storage
    app_menus: Vec<OwnedMenu>,
    /// When true, the window-level text selection must not start on the
    /// current mouse down. Set by components that own their own mouse-down
    /// interaction (e.g. `Input`, `Button`); reset by the selection
    /// controller in the capture phase of every left mouse down.
    pub(crate) suppress_text_selection: bool,
    /// Stack of active text-selection scopes, pushed/popped by the
    /// `SelectionScopeMarker` element that wraps each Dialog/Sheet content
    /// subtree during paint. Empty means the base window layer. A selectable
    /// `TextView` reads the top of this stack when it registers, so window
    /// selection can be confined to the active modal.
    selection_scope_stack: Vec<SelectionScope>,
    /// Weak focus handles of every live `PopupMenu`. A menu holds focus
    /// exactly while it is open, so "any live handle contains focus" is a
    /// level signal for "a menu is open in this window". The managed tooltip
    /// overlay consults it to keep tooltips suppressed while menus are up.
    menu_focus_handles: Vec<WeakFocusHandle>,
    /// Live bounds of every open `PopupMenu`, keyed by menu entity. Upserted
    /// on the menu's prepaint; removed on dismiss and on entity release (for
    /// menus torn down without a dismiss). A `Popover` consults this before
    /// dismissing on mouse-down-out: a press inside an open menu belongs to
    /// the menu — which may be the popover's own deferred descendant painted
    /// OUTSIDE the popover's bounds (a row's dropdown) — never a dismissal.
    open_menu_bounds: HashMap<EntityId, Bounds<Pixels>>,
    /// Optional app-registered interceptor for TextView link clicks — consulted
    /// by `text::open_text_link` before falling back to `cx.open_url`. Lets an
    /// app claim custom schemes (e.g. an in-app action scheme) while real URLs
    /// keep opening the browser.
    pub(crate) text_link_handler: Option<TextLinkHandler>,
}

impl GlobalState {
    pub(crate) fn new() -> Self {
        Self {
            text_view_state_stack: Vec::new(),
            open_deferred_popovers: HashSet::new(),
            app_menus: Vec::new(),
            suppress_text_selection: false,
            selection_scope_stack: Vec::new(),
            menu_focus_handles: Vec::new(),
            open_menu_bounds: HashMap::new(),
            text_link_handler: None,
        }
    }

    /// Suppress the window-level text selection for the current mouse down.
    ///
    /// Call this from a mouse-down handler (bubble phase) of a component that
    /// owns its own press/drag interaction, so that pressing it does not start
    /// a window text selection. The flag is reset on the next mouse down.
    pub fn suppress_text_selection(cx: &mut App) {
        Self::global_mut(cx).suppress_text_selection = true;
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }

    pub(crate) fn text_view_state(&self) -> Option<&Entity<TextViewState>> {
        self.text_view_state_stack.last()
    }

    /// Push a selection scope while painting a Dialog/Sheet content subtree.
    pub(crate) fn push_selection_scope(&mut self, scope: SelectionScope) {
        self.selection_scope_stack.push(scope);
    }

    /// Pop the selection scope after painting a Dialog/Sheet content subtree.
    pub(crate) fn pop_selection_scope(&mut self) {
        self.selection_scope_stack.pop();
    }

    /// The selection scope currently being painted. `Base` when not inside any
    /// Dialog/Sheet content subtree.
    pub(crate) fn current_selection_scope(&self) -> SelectionScope {
        self.selection_scope_stack
            .last()
            .copied()
            .unwrap_or(SelectionScope::Base)
    }

    /// Check if we are currently inside a deferred context (e.g., inside an open Popover).
    pub(crate) fn is_in_deferred_context(&self) -> bool {
        !self.open_deferred_popovers.is_empty()
    }

    /// Register a popover that uses deferred rendering as open.
    pub(crate) fn register_deferred_popover(&mut self, focus_handle: &FocusHandle) {
        self.open_deferred_popovers
            .insert(format!("{focus_handle:?}").into());
    }

    /// Unregister a popover when it closes.
    pub(crate) fn unregister_deferred_popover(&mut self, focus_handle: &FocusHandle) {
        let element_id: ElementId = format!("{focus_handle:?}").into();
        self.open_deferred_popovers.remove(&element_id);
    }

    /// Upsert an open menu's bounds (called from the menu's prepaint, which
    /// only runs while the menu is showing).
    pub(crate) fn update_menu_bounds(&mut self, id: EntityId, bounds: Bounds<Pixels>) {
        self.open_menu_bounds.insert(id, bounds);
    }

    /// Forget a menu's bounds — on dismiss, and on entity release for menus
    /// torn down without one.
    pub(crate) fn remove_menu_bounds(&mut self, id: EntityId) {
        self.open_menu_bounds.remove(&id);
    }

    /// Whether `position` (window coordinates) falls inside any open
    /// `PopupMenu`'s bounds.
    pub(crate) fn position_in_open_menu(&self, position: &Point<Pixels>) -> bool {
        self.open_menu_bounds.values().any(|b| b.contains(position))
    }

    /// Track a menu's focus handle for [`GlobalState::is_menu_focused`].
    /// Weakly held: a dropped menu unregisters itself, so there is no
    /// unregister call to forget.
    pub(crate) fn register_menu_focus_handle(&mut self, focus_handle: &FocusHandle) {
        self.menu_focus_handles.retain(|h| h.upgrade().is_some());
        self.menu_focus_handles.push(focus_handle.downgrade());
    }

    /// True while an open menu owns focus in `window`. Menus hold focus
    /// exactly while open (and closed menu entities may stay cached, e.g. by
    /// `ContextMenu`), so focus — not liveness — is the open test.
    pub(crate) fn is_menu_focused(&self, window: &Window, cx: &App) -> bool {
        self.menu_focus_handles
            .iter()
            .filter_map(|h| h.upgrade())
            .any(|h| h.contains_focused(window, cx))
    }

    /// Get the application menus
    pub fn app_menus(&self) -> &[OwnedMenu] {
        &self.app_menus
    }

    /// Set the application menus
    pub fn set_app_menus(&mut self, menus: Vec<OwnedMenu>) {
        self.app_menus = menus;
    }
}
