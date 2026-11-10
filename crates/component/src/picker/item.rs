use gpui::{
    AnyElement, App, ElementId, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, RenderOnce, SharedString, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};

use crate::{ActiveTheme as _, Selectable, Size, StyleSized as _, TruncateMiddleExt as _, h_flex};

/// How an item's label is clipped when it outgrows the row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PickerItemTruncate {
    /// End-trim, for text whose head carries the meaning (a search query).
    #[default]
    End,
    /// Middle-ellipsize, keeping the distinctive head and tail (a filesystem path).
    Middle,
}

/// One row of a picker body: a truncating label, an optional trailing action
/// revealed on hover (or on the keyboard cursor), and an optional fixed
/// trailing accessory.
///
/// The row carries no click handler of its own. A `PickerItem` is a `List`
/// item, and `List` already wraps every delegate item in a stateful div whose
/// `on_click` sets the selected index and confirms, without stopping
/// propagation; a row-level `on_click` would fire alongside it and activate
/// twice. Activation belongs in the delegate's `confirm`.
///
/// The hover action swallows the mouse-down, so a click on it never activates
/// the row.
#[derive(IntoElement)]
pub struct PickerItem {
    id: ElementId,
    label: SharedString,
    truncate: PickerItemTruncate,
    menu_size: Size,
    active: bool,
    selected: bool,
    hover_action: Option<AnyElement>,
    accessory: Option<AnyElement>,
    leading: Option<AnyElement>,
}

impl PickerItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            truncate: PickerItemTruncate::End,
            menu_size: Size::Medium,
            active: false,
            selected: false,
            hover_action: None,
            accessory: None,
            leading: None,
        }
    }

    /// Optional provider or identity glyph before the label.
    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.leading = Some(leading.into_any_element());
        self
    }

    /// Label clipping mode. Default [`PickerItemTruncate::End`].
    pub fn truncate(mut self, truncate: PickerItemTruncate) -> Self {
        self.truncate = truncate;
        self
    }

    /// Menu text tier, resolved when rendered. Medium keeps primary rows on
    /// the 13px control tier in Compact; Small follows the text-size preference.
    /// Row padding remains independent of this text choice.
    pub fn menu_text_size(mut self, size: Size) -> Self {
        self.menu_size = size;
        self
    }

    /// The current-value row uses the standard menu highlight. Distinct from
    /// `selected`, which is the transient keyboard cursor.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Trailing button revealed on row hover (and kept visible on the
    /// keyboard-selected row, so an arrow-key user can see the target).
    pub fn hover_action(mut self, action: impl IntoElement) -> Self {
        self.hover_action = Some(action.into_any_element());
        self
    }

    /// Trailing glyph shown at rest (for example the active row's check).
    pub fn accessory(mut self, accessory: impl IntoElement) -> Self {
        self.accessory = Some(accessory.into_any_element());
        self
    }
}

impl Selectable for PickerItem {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl RenderOnce for PickerItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let hover_bg = cx.theme().tokens.list_hover;
        // Match PopupMenu and SearchableListItemElement. list_active is an
        // alpha-capped list selection tint, too faint on a popover surface.
        let active_bg = cx.theme().tokens.accent;
        let selected = self.selected;
        let truncate = self.truncate;

        h_flex()
            .id(self.id)
            // Fill the popover: the row (and its hover fill) spans the full
            // width, not just its text.
            .w_full()
            // Unnamed group so the trailing action reveals on ROW hover.
            .group("")
            // Left inset stays the list gutter. A trailing control occupies the
            // right inset the way a production row action does, so pr drops to
            // 4px instead of stacking row padding and button padding.
            .list_px(Size::Small)
            .when(
                self.hover_action.is_some() || self.accessory.is_some(),
                |this| this.pr(px(4.)),
            )
            .list_py(Size::Small)
            .gap_x_1()
            .rounded(cx.theme().radius)
            .text_size(self.menu_size.menu_text_size(cx))
            .text_color(cx.theme().foreground)
            .items_center()
            .justify_between()
            .when(self.active || selected, |this| this.bg(active_bg))
            .when(!self.active && !selected, |this| {
                this.hover(|this| this.bg(hover_bg))
            })
            .when_some(self.leading, |this, leading| {
                this.child(trailing_slot().mr_1().child(leading))
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .map(|this| match truncate {
                        PickerItemTruncate::End => this.truncate(),
                        PickerItemTruncate::Middle => this.truncate_middle(),
                    })
                    .child(self.label),
            )
            .child(
                h_flex()
                    .flex_shrink_0()
                    .items_center()
                    .gap_1()
                    .when_some(self.hover_action, |this, action| {
                        this.child(
                            trailing_slot()
                                // Hidden at rest; shown on row hover, and always
                                // shown on the keyboard-cursor row.
                                .when(!selected, |this| this.invisible())
                                .group_hover("", |this| this.visible())
                                // Clicking the action must not also fire the row.
                                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                                .child(action),
                        )
                    })
                    .when_some(self.accessory, |this, accessory| {
                        this.child(trailing_slot().child(accessory))
                    }),
            )
    }
}

/// Trailing column hugs the control; a padded slot around a small dismiss
/// stacked on the row's own `pr` would push it away from the edge.
fn trailing_slot() -> gpui::Div {
    div().flex().items_center().justify_center().flex_shrink_0()
}
