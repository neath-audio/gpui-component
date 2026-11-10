use gpui::{
    App, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _, Window, div,
};

use crate::{ActiveTheme as _, Size, StyleSized as _, StyledTypography as _};

/// Muted group header for a picker body.
///
/// Outside a list it carries primary menu text and `px_3`, 12px from the popover
/// edge, lining up with row text (list `px_1` plus `list_px` Small 8). Inside a
/// list that already carries `px_1` ([`in_list`](Self::in_list)) it uses the
/// rows' `list_px` Small and the supporting 12px role, so the label lines up
/// with [`PickerItem`](super::PickerItem) text.
#[derive(IntoElement)]
pub struct PickerSectionHeader {
    text: SharedString,
    in_list: bool,
}

impl PickerSectionHeader {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            in_list: false,
        }
    }

    /// Whether the header sits inside a `List` that already insets its rows.
    pub fn in_list(mut self, in_list: bool) -> Self {
        self.in_list = in_list;
        self
    }
}

impl RenderOnce for PickerSectionHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let header = div().py_0p5().text_color(cx.theme().muted_foreground);
        let header = if self.in_list {
            header.list_px(Size::Small).text_ui_sm(cx)
        } else {
            header.px_3().text_size(Size::Medium.menu_text_size(cx))
        };
        header.child(self.text)
    }
}
