use std::collections::HashMap;

use gpui::{App, Bounds, EntityId, Global, Pixels, Point, Window};

pub use gpui_base::GlobalState;

/// Signature of an app-registered TextView link-click interceptor.
pub(crate) type TextLinkHandler = std::rc::Rc<dyn Fn(&str, &mut Window, &mut App) -> bool>;

pub(crate) fn init(cx: &mut App) {
    // Preserve the legacy initialization point while `gpui_base::init` remains
    // after Root initialization for focus-trap ordering compatibility.
    GlobalState::init(cx);
    if !cx.has_global::<UiGlobalState>() {
        cx.set_global(UiGlobalState::new());
    }
}

/// UI-only global state whose types cannot cross into `gpui-base`.
pub(crate) struct UiGlobalState {
    open_menu_bounds: HashMap<EntityId, Bounds<Pixels>>,
    pub(crate) text_link_handler: Option<TextLinkHandler>,
}

impl Global for UiGlobalState {}

impl UiGlobalState {
    fn new() -> Self {
        Self {
            open_menu_bounds: HashMap::new(),
            text_link_handler: None,
        }
    }

    pub(crate) fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub(crate) fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
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
}
