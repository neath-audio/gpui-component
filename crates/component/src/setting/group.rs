use crate::StyledTypography as _;
use std::rc::Rc;

use gpui::{
    AnyElement, App, IntoElement, ParentElement as _, SharedString, StyleRefinement, Styled,
    Window, div, prelude::FluentBuilder as _,
};

use crate::{
    ActiveTheme, StyledExt,
    group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants},
    label::Label,
    setting::{RenderOptions, SettingItem},
    v_flex,
};

/// A setting group that can contain multiple setting items.
#[derive(Clone)]
pub struct SettingGroup {
    style: StyleRefinement,
    footer: Option<Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>>,
    variant: Option<GroupBoxVariant>,

    pub(super) title: Option<SharedString>,
    pub(super) description: Option<SharedString>,
    pub(super) items: Vec<SettingItem>,
}

impl Styled for SettingGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl SettingGroup {
    /// Create a new setting group.
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
            footer: None,
            variant: None,
            title: None,
            description: None,
            items: Vec::new(),
        }
    }

    /// Set the label of the setting group, default is None.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the description of the setting group, default is None.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the variant of the group surface, overriding the variant set via
    /// `Settings::with_group_variant` for this group, default is None (use the
    /// settings-level variant).
    ///
    /// For example, `GroupBoxVariant::Normal` presents the items directly,
    /// without the card surface a global `Outline` or `Fill` default draws
    /// around the group.
    pub fn variant(mut self, variant: GroupBoxVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    /// Render supporting content below, and outside, the group's surface.
    ///
    /// The footer aligns with the group title and renders as small muted text,
    /// like a description. It scrolls with the group and follows its search
    /// visibility; it does not add an independently searchable item or a
    /// sidebar entry, and a group needs at least one item to be shown.
    pub fn footer<F, E>(mut self, footer: F) -> Self
    where
        E: IntoElement,
        F: Fn(&mut Window, &mut App) -> E + 'static,
    {
        self.footer = Some(Rc::new(move |window, cx| {
            footer(window, cx).into_any_element()
        }));
        self
    }

    /// Add a setting item to the group.
    pub fn item(mut self, item: SettingItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add multiple setting items to the group.
    pub fn items<I>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = SettingItem>,
    {
        self.items.extend(items);
        self
    }

    /// Return true if any of the setting items in the group match the given query.
    pub(super) fn is_match(&self, query: &str, cx: &App) -> bool {
        self.items.iter().any(|item| item.is_match(query, cx))
    }

    pub(super) fn is_resettable(&self, query: &str, cx: &App) -> bool {
        self.items
            .iter()
            .any(|item| item.is_match(query, cx) && item.is_resettable(cx))
    }

    pub(crate) fn render(
        self,
        query: &str,
        options: &RenderOptions,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        let text_size = options.size().control_text_size(cx);
        GroupBox::new()
            .id(SharedString::from(format!("group-{}", options.group_ix())))
            .with_variant(self.variant.unwrap_or(options.group_variant()))
            .when_some(self.title.clone(), |this, title| {
                // Promote the group title to a real section heading: the
                // `GroupBox` wraps its title in `muted_foreground`, so override
                // it back to the default foreground here. Settings-size text, regular
                // weight — one tier below the (semibold, size-inherited) page
                // title above; against the same-size item labels the heading
                // role is carried by position and the foreground-over-muted
                // description, not by typography.
                this.title(
                    v_flex()
                        // `gap_2` (not `gap_1`): the `GroupBox` title wrapper
                        // forces `line_height(1.)`, so a 4px gap crowds the
                        // promoted heading against its description. 8px gives
                        // it room without matching the page header's 12px.
                        .gap_2()
                        .child(
                            div()
                                .text_size(text_size)
                                .text_color(cx.theme().foreground)
                                .child(title),
                        )
                        .when_some(self.description.clone(), |this, description| {
                            this.child(
                                Label::new(description)
                                    .text_ui_sm(cx)
                                    .text_color(cx.theme().muted_foreground),
                            )
                        }),
                )
            })
            .gap_4()
            .children(self.items.iter().enumerate().filter_map(|(item_ix, item)| {
                if item.is_match(&query, cx) {
                    Some(
                        item.clone()
                            .render_item(&options.with_item_ix(item_ix), window, cx),
                    )
                } else {
                    None
                }
            }))
            .when_some(self.footer, |this, footer| this.footer(footer(window, cx)))
            .refine_style(&self.style)
    }

    pub(crate) fn reset(&self, query: &str, window: &mut Window, cx: &mut App) {
        for item in &self.items {
            if item.is_match(query, cx) {
                item.reset(window, cx);
            }
        }
    }
}
