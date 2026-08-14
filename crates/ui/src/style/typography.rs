use gpui::{Div, ParentElement as _, SharedString, Styled, div};

fn end_truncating_text(text: SharedString) -> Div {
    div().flex_1().min_w_0().truncate().child(text)
}

fn middle_truncating_text(text: SharedString) -> Div {
    div().flex_1().min_w_0().truncate_middle().child(text)
}

pub fn truncating_cell(text: impl Into<SharedString>) -> Div {
    div()
        .size_full()
        .flex()
        .items_center()
        .overflow_hidden()
        .text_xs()
        .child(end_truncating_text(text.into()))
}

pub fn truncating_cell_sized(text: impl Into<SharedString>, size: gpui::Pixels) -> Div {
    div()
        .size_full()
        .flex()
        .items_center()
        .overflow_hidden()
        .text_size(size)
        .child(end_truncating_text(text.into()))
}

pub fn middle_truncating_cell_sized(text: impl Into<SharedString>, size: gpui::Pixels) -> Div {
    div()
        .size_full()
        .flex()
        .items_center()
        .overflow_hidden()
        .text_size(size)
        .child(middle_truncating_text(text.into()))
}

pub trait TruncateMiddleExt: Styled {
    fn truncate_middle(self) -> Self {
        self.overflow_hidden()
            .whitespace_nowrap()
            .text_ellipsis_middle()
    }
}

impl<E: Styled> TruncateMiddleExt for E {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inner_text_wrappers_preserve_end_and_middle_truncation() {
        let mut end = end_truncating_text("end".into());
        let mut expected_end = div().flex_1().min_w_0().truncate();
        assert_eq!(end.style().clone(), expected_end.style().clone());

        let mut middle = middle_truncating_text("middle".into());
        let mut expected_middle = div()
            .flex_1()
            .min_w_0()
            .overflow_hidden()
            .whitespace_nowrap()
            .text_ellipsis_middle();
        assert_eq!(middle.style().clone(), expected_middle.style().clone());
    }
}
