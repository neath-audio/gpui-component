use crate::StyledTypography as _;
use gpui::{
    AnyElement, App, Axis, Div, InteractiveElement as _, IntoElement, ParentElement, Rems,
    SharedString, Stateful, Styled, Window, div, prelude::FluentBuilder as _,
};
use rust_i18n::t;
use std::{any::TypeId, ops::Deref, rc::Rc};

use crate::{
    ActiveTheme as _, AxisExt, Disableable as _, Icon, IconName, Sizable as _, StyledExt as _,
    button::Button,
    h_flex,
    label::Label,
    setting::{
        AnySettingField, ElementField, RenderOptions, SettingSubPage,
        fields::{
            BoolField, DropdownField, NumberField, ResetHandler, SettingFieldRender, StringField,
        },
        sub_page::SubPageLink,
    },
    text::Text,
    v_flex,
};

/// Setting item.
#[derive(Clone)]
pub enum SettingItem {
    /// A normal setting item with a title, description, and field.
    Item {
        title: SharedString,
        description: Option<Text>,
        keywords: Vec<SharedString>,
        layout: Axis,
        disabled: bool,
        field: Rc<dyn AnySettingField>,
    },
    /// A full custom element to render.
    Element {
        disabled: bool,
        keywords: Vec<SharedString>,
        /// Optional custom reset behavior. The first closure reports whether
        /// the item is "dirty" (controls reset button visibility), the second
        /// performs the reset.
        reset_handler: Option<ResetHandler>,
        render: Rc<dyn Fn(&RenderOptions, &mut Window, &mut App) -> AnyElement + 'static>,
    },
    /// A row that opens a [`SettingSubPage`] over the current page.
    SubPage {
        title: SharedString,
        description: Option<Text>,
        keywords: Vec<SharedString>,
        disabled: bool,
        page: SettingSubPage,
    },
}

fn matches_text(
    query: &str,
    title: &SharedString,
    description: &Option<Text>,
    keywords: &[SharedString],
    cx: &App,
) -> bool {
    let q = &query.to_lowercase();
    title.to_lowercase().contains(q)
        || description
            .as_ref()
            .map_or(false, |d| d.get_text(cx).to_lowercase().contains(q))
        || keywords.iter().any(|s| s.to_lowercase().contains(q))
}

impl SettingItem {
    /// Create a new setting item.
    pub fn new<F>(title: impl Into<SharedString>, field: F) -> Self
    where
        F: AnySettingField + 'static,
    {
        SettingItem::Item {
            title: title.into(),
            description: None,
            layout: Axis::Horizontal,
            disabled: false,
            keywords: Vec::new(),
            field: Rc::new(field),
        }
    }

    /// Create a row with a Configure button that opens `page` over the current one.
    ///
    /// Search finds this row by its own text or by anything in the page's groups.
    pub fn sub_page(title: impl Into<SharedString>, page: SettingSubPage) -> Self {
        SettingItem::SubPage {
            title: title.into(),
            description: None,
            keywords: Vec::new(),
            disabled: false,
            page,
        }
    }

    /// Create a new custom element setting item with a render closure.
    pub fn render<R, E>(render: R) -> Self
    where
        E: IntoElement,
        R: Fn(&RenderOptions, &mut Window, &mut App) -> E + 'static,
    {
        SettingItem::Element {
            disabled: false,
            keywords: Vec::new(),
            reset_handler: None,
            render: Rc::new(move |options, window, cx| {
                render(options, window, cx).into_any_element()
            }),
        }
    }

    /// Provide custom reset behavior for a custom element item.
    ///
    /// Only applies to [`SettingItem::Element`] (created via
    /// [`SettingItem::render`]). When set, the page-level reset button will
    /// appear while `is_dirty` returns true, and clicking it invokes `reset`.
    ///
    /// - `is_dirty` reports whether the item differs from its default state.
    /// - `reset` performs the reset.
    pub fn on_reset<D, R>(mut self, is_dirty: D, reset: R) -> Self
    where
        D: Fn(&App) -> bool + 'static,
        R: Fn(&mut Window, &mut App) + 'static,
    {
        match &mut self {
            SettingItem::Element { reset_handler, .. } => {
                *reset_handler = Some((Rc::new(is_dirty), Rc::new(reset)));
            }
            // `on_reset` is meaningless for a value-bearing item: use the
            // field's own `default_value` / `SettingField::on_reset` instead.
            SettingItem::Item { .. } | SettingItem::SubPage { .. } => {
                debug_assert!(
                    false,
                    "SettingItem::on_reset only applies to SettingItem::Element; \
                     use SettingField::default_value or SettingField::on_reset for a normal item"
                );
            }
        }
        self
    }

    /// Set additional keywords used only for search matching (not rendered).
    ///
    /// For example, an item titled "Enable Two-factor auth" can be made
    /// searchable via "MFA". This is also useful for custom elements that
    /// have no title/description but should still show up in search results.
    pub fn keywords<I, S>(mut self, keywords: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        let keywords: Vec<SharedString> = keywords.into_iter().map(Into::into).collect();
        match &mut self {
            SettingItem::Item { keywords: k, .. } => *k = keywords,
            SettingItem::Element { keywords: k, .. } => *k = keywords,
            SettingItem::SubPage { keywords: k, .. } => *k = keywords,
        }
        self
    }

    /// Set whether the setting item is disabled, default is false.
    ///
    /// A disabled item is rendered with reduced opacity. For
    /// [`SettingItem::Item`] the underlying field is also rendered in a
    /// non-interactive state. For [`SettingItem::Element`] the `disabled` flag
    /// is forwarded via [`RenderOptions::disabled`] so the custom renderer can
    /// disable its interactive controls.
    pub fn disabled(mut self, disabled: bool) -> Self {
        match &mut self {
            SettingItem::Item { disabled: d, .. } => *d = disabled,
            SettingItem::Element { disabled: d, .. } => *d = disabled,
            SettingItem::SubPage { disabled: d, .. } => *d = disabled,
        }
        self
    }

    /// Set the description of the setting item.
    ///
    /// Only applies to [`SettingItem::Item`].
    pub fn description(mut self, description: impl Into<Text>) -> Self {
        match &mut self {
            SettingItem::Item { description: d, .. }
            | SettingItem::SubPage { description: d, .. } => {
                *d = Some(description.into());
            }
            SettingItem::Element { .. } => {}
        }
        self
    }

    /// Set the layout of the setting item.
    ///
    /// Only applies to [`SettingItem::Item`].
    pub fn layout(mut self, layout: Axis) -> Self {
        match &mut self {
            SettingItem::Item { layout: l, .. } => {
                *l = layout;
            }
            SettingItem::Element { .. } | SettingItem::SubPage { .. } => {}
        }
        self
    }

    pub(crate) fn is_match(&self, query: &str, cx: &App) -> bool {
        match self {
            SettingItem::Item {
                title,
                description,
                keywords,
                ..
            } => matches_text(query, title, description, keywords, cx),
            SettingItem::SubPage {
                title,
                description,
                keywords,
                page,
                ..
            } => matches_text(query, title, description, keywords, cx) || page.is_match(query, cx),
            // We need to show all custom elements when not searching.
            SettingItem::Element { keywords, .. } => {
                let q = &query.to_lowercase();
                query.is_empty() || keywords.iter().any(|s| s.to_lowercase().contains(q))
            }
        }
    }

    pub(crate) fn is_resettable(&self, cx: &App) -> bool {
        match self {
            SettingItem::Item { field, .. } => field.is_resettable(cx),
            SettingItem::Element { reset_handler, .. } => reset_handler
                .as_ref()
                .is_some_and(|(is_dirty, _)| is_dirty(cx)),
            SettingItem::SubPage { .. } => false,
        }
    }

    pub(crate) fn reset(&self, window: &mut Window, cx: &mut App) {
        match self {
            SettingItem::Item { field, .. } => field.reset(window, cx),
            SettingItem::Element { reset_handler, .. } => {
                if let Some((_, reset)) = reset_handler.as_ref() {
                    reset(window, cx);
                }
            }
            SettingItem::SubPage { .. } => {}
        }
    }

    fn render_field(
        title: SharedString,
        field: Rc<dyn AnySettingField>,
        options: RenderOptions,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        // A field-level size (set via `Sizable`) overrides the page-wide size.
        let options = options.with_size(field.size().unwrap_or(options.size()));
        let field_type = field.field_type();
        let style = field.style().clone();
        let type_id = field.deref().type_id();
        let renderer: Box<dyn SettingFieldRender> = match type_id {
            t if t == std::any::TypeId::of::<bool>() => {
                Box::new(BoolField::new(field_type.is_switch(), title))
            }
            t if t == TypeId::of::<f64>() && field_type.is_number_input() => {
                Box::new(NumberField::new(field_type.number_input_options(), title))
            }
            t if t == TypeId::of::<SharedString>() && field_type.is_input() => {
                Box::new(StringField::<SharedString>::new(title))
            }
            t if t == TypeId::of::<String>() && field_type.is_input() => {
                Box::new(StringField::<String>::new(title))
            }
            t if t == TypeId::of::<SharedString>() && field_type.is_dropdown() => {
                Box::new(DropdownField::<SharedString>::new(
                    field_type.dropdown_options(),
                    field_type.dropdown_scrollable(),
                    title,
                ))
            }
            t if t == TypeId::of::<String>() && field_type.is_dropdown() => {
                Box::new(DropdownField::<String>::new(
                    field_type.dropdown_options(),
                    field_type.dropdown_scrollable(),
                    title,
                ))
            }
            _ if field_type.is_element() => Box::new(ElementField::new(field_type.element())),
            _ => unimplemented!("Unsupported setting type: {}", field.deref().type_name()),
        };

        renderer.render(field, &options, &style, window, cx)
    }

    /// The title and description on one side, and the control on the other.
    fn labeled_row(
        title: SharedString,
        description: Option<Text>,
        layout: Axis,
        disabled: bool,
        text_size: Rems,
        control: impl IntoElement,
        cx: &App,
    ) -> AnyElement {
        div()
            .w_full()
            .when(disabled, |this| this.opacity(0.5))
            .map(|this| {
                if layout.is_horizontal() {
                    this.h_flex().justify_between().items_center()
                } else {
                    this.v_flex()
                }
            })
            .gap_3()
            .child(
                v_flex()
                    .map(|this| {
                        if layout.is_horizontal() {
                            this.flex_1().max_w_3_5()
                        } else {
                            this.w_full()
                        }
                    })
                    .gap_1()
                    .child(Label::new(title).text_size(text_size))
                    .when_some(description, |this, description| {
                        this.child(
                            div()
                                .size_full()
                                .text_ui_sm(cx)
                                .text_color(cx.theme().muted_foreground)
                                .child(description),
                        )
                    }),
            )
            .child(control)
            .into_any_element()
    }

    pub(super) fn render_item(
        self,
        options: &RenderOptions,
        link: &SubPageLink,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let text_size = options.size().control_text_size(cx);
        div()
            .id(SharedString::from(format!("item-{}", options.item_ix())))
            .w_full()
            .child(match self {
                SettingItem::Item {
                    title,
                    description,
                    layout,
                    disabled,
                    field,
                    ..
                } => {
                    let layout = if options.layout().is_vertical() {
                        Axis::Vertical
                    } else {
                        layout
                    };
                    let field = div().id("field").child(Self::render_field(
                        title.clone(),
                        field,
                        options.with_layout(layout).with_disabled(disabled),
                        window,
                        cx,
                    ));
                    Self::labeled_row(title, description, layout, disabled, text_size, field, cx)
                }
                SettingItem::SubPage {
                    title,
                    description,
                    disabled,
                    page,
                    ..
                } => {
                    let layout = if options.layout().is_vertical() {
                        Axis::Vertical
                    } else {
                        Axis::Horizontal
                    };
                    let focus = link
                        .navigation
                        .update(cx, |navigation, cx| navigation.link_focus(options, cx));
                    let configure = t!("Settings.Configure").to_string();
                    let button = Button::new("configure")
                        .with_size(options.size())
                        .disabled(disabled)
                        .track_focus(&focus)
                        .accessibility_label(format!("{configure} {title}"))
                        .when(cfg!(test), |this| {
                            this.debug_selector(|| "setting-sub-page-link".into())
                        })
                        .child(
                            h_flex()
                                .gap_1()
                                .items_center()
                                .child(configure)
                                .child(Icon::new(IconName::ChevronRight).xsmall()),
                        )
                        .on_click({
                            let navigation = link.navigation.clone();
                            let section = link.section.clone();
                            let origin = Some((options.group_ix(), options.item_ix()));
                            move |_, window, cx| {
                                navigation.update(cx, |navigation, cx| {
                                    navigation.push_from(
                                        page.clone(),
                                        section.clone(),
                                        origin,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        });
                    Self::labeled_row(title, description, layout, disabled, text_size, button, cx)
                }
                SettingItem::Element {
                    disabled, render, ..
                } => div()
                    .w_full()
                    .when(disabled, |this| this.opacity(0.5))
                    .child((render)(&options.with_disabled(disabled), window, cx))
                    .into_any_element(),
            })
    }
}
