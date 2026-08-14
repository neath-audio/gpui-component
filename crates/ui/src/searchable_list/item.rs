use gpui::{
    AnyElement, App, Background, ElementId, Hsla, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, StyleRefinement, Styled, Window, prelude::FluentBuilder,
};

use crate::{
    ActiveTheme, Disableable, Icon, IconName, Selectable, Sizable, Size, StyleSized, StyledExt,
    h_flex,
};

/// A single row element used inside searchable-list dropdowns (Select, ComboBox, MultiComboBox).
///
/// - `selected` — controls the cursor-highlight background (the `List` overwrites this field via
///   `Selectable::selected` to match the keyboard cursor position).
/// - `checked` — controls the visibility of the trailing check icon; set by the adapter based on
///   the current selection state and NOT overwritten by the `List`.
#[derive(IntoElement)]
pub struct SearchableListItemElement {
    id: ElementId,
    size: Size,
    style: StyleRefinement,
    /// Cursor/highlight background (overridden by `List` to the keyboard cursor row).
    selected: bool,
    /// Whether the trailing check icon is shown.
    checked: bool,
    disabled: bool,
    children: Vec<AnyElement>,
    /// The icon drawn at the trailing edge when `checked` is `true`.
    check_icon: Option<Icon>,
    /// Optional override for the selected/hover highlight background. `None`
    /// keeps the theme defaults — the gradient-capable `accent` token for the
    /// selected row and a dimmed flat `accent` for hover. `Some` tints both
    /// with the given solid color (selected at full strength, hover dimmed) so
    /// a caller can color the menu cursor to match.
    highlight: Option<Hsla>,
}

impl SearchableListItemElement {
    pub fn new(ix: usize) -> Self {
        Self {
            id: ("searchable-list-item", ix).into(),
            size: Size::default(),
            style: StyleRefinement::default(),
            selected: false,
            checked: false,
            disabled: false,
            children: Vec::new(),
            check_icon: Some(Icon::new(IconName::Check)),
            highlight: None,
        }
    }

    /// Set whether the trailing check icon is visible.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Override the selected/hover highlight background. `None` (default)
    /// keeps the gradient-capable theme `accent` token; `Some` tints the cursor
    /// highlight with the given solid color.
    pub fn highlight(mut self, highlight: impl Into<Option<Hsla>>) -> Self {
        self.highlight = highlight.into();
        self
    }

    /// Override the default check icon, or pass `None` to remove the
    /// trailing check column entirely (no reserved space).
    pub fn check_icon(mut self, icon: impl Into<Option<Icon>>) -> Self {
        self.check_icon = icon.into();
        self
    }
}

impl ParentElement for SearchableListItemElement {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Disableable for SearchableListItemElement {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Selectable for SearchableListItemElement {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl Sizable for SearchableListItemElement {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl Styled for SearchableListItemElement {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for SearchableListItemElement {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        // Selected and hover are styled independently so each keeps its full
        // theme capacity. Selected is a `Background` — the default uses the
        // gradient-capable `accent` token (matching upstream); hover is an
        // `Hsla` because dimming via `.opacity()` only applies to a flat color
        // (also matching upstream). `highlight`, when set, tints both with the
        // caller's solid color.
        let selected_bg: Background = match self.highlight {
            Some(c) => c.into(),
            None => cx.theme().tokens.accent.into(),
        };
        let hover_bg: Hsla = match self.highlight {
            Some(c) => c.opacity(0.6),
            None => cx.theme().accent.opacity(0.7),
        };
        h_flex()
            .id(self.id)
            .relative()
            .gap_x_1()
            .py_1()
            .px_2()
            .rounded(cx.theme().radius)
            .text_base()
            .text_color(cx.theme().foreground)
            .items_center()
            .justify_between()
            .input_text_size(self.size)
            .list_size(self.size)
            .refine_style(&self.style)
            .when(!self.disabled, |this| {
                this.when(!self.selected, |this| this.hover(|this| this.bg(hover_bg)))
            })
            .when(self.selected, |this| this.bg(selected_bg))
            .when(self.disabled, |this| {
                this.cursor_not_allowed()
                    .text_color(cx.theme().muted_foreground)
            })
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .gap_x_1()
                    .child(h_flex().w_full().items_center().children(self.children))
                    .when_some(self.check_icon, |this, icon| {
                        this.child(
                            icon.xsmall()
                                .text_color(cx.theme().foreground)
                                .when(!self.checked, |this| this.invisible()),
                        )
                    }),
            )
    }
}
