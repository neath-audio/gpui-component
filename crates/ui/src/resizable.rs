use std::rc::Rc;

use gpui::{IntoElement as _, Styled as _, div, px, transparent_black};

use crate::{ActiveTheme as _, AxisExt as _};

pub use gpui_base::{
    ResizablePanel, ResizablePanelEvent, ResizablePanelGroup, ResizableState, h_resizable,
    resizable_panel, resize_handle, v_resizable,
};

/// Paint a resize divider only while it is active.
///
/// The panel boundary supplies the resting seam. GPUI Base continues to own
/// the hit area, cursor, and drag interaction; this recipe owns only Neath's
/// transparent-idle and accent-active presentation.
pub fn seamless_handle_appearance() -> gpui_base::ResizeHandleRenderer {
    Rc::new(|handle, _, cx| {
        let color = if handle.is_active() {
            cx.theme().drag_border
        } else {
            transparent_black()
        };
        let line = div().bg(color);
        let line = if handle.axis().is_horizontal() {
            line.h_full().w(px(1.))
        } else {
            line.w_full().h(px(1.))
        };
        Some(line.into_any_element())
    })
}
