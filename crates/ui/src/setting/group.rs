use gpui::{
    App, IntoElement, ParentElement as _, SharedString, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _,
};

use crate::{
    ActiveTheme, StyledExt,
    group_box::{GroupBox, GroupBoxVariants},
    label::Label,
    setting::{RenderOptions, SettingItem},
    v_flex,
};

/// A setting group that can contain multiple setting items.
#[derive(Clone)]
pub struct SettingGroup {
    style: StyleRefinement,

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

    pub(super) fn is_resettable(&self, cx: &App) -> bool {
        self.items.iter().any(|item| item.is_resettable(cx))
    }

    pub(crate) fn render(
        self,
        query: &str,
        options: &RenderOptions,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        GroupBox::new()
            .id(SharedString::from(format!("group-{}", options.group_ix())))
            .with_variant(options.group_variant())
            .when_some(self.title.clone(), |this, title| {
                // Promote the group title to a real section heading: the
                // `GroupBox` wraps its title in `text_muted`, so override
                // it back to `text` here. `text_sm`, regular
                // weight — one tier below the (semibold, size-inherited) page
                // title above; against the same-size item labels the heading
                // role is carried by position and the muted description,
                // not by typography.
                this.title(
                    v_flex()
                        // `gap_2` (not `gap_1`): the `GroupBox` title wrapper
                        // forces `line_height(1.)`, so a 4px gap crowds the
                        // promoted heading against its description. 8px gives
                        // it room without matching the page header's 12px.
                        .gap_2()
                        .child(div().text_sm().text_color(cx.theme().text).child(title))
                        .when_some(self.description.clone(), |this, description| {
                            this.child(
                                Label::new(description)
                                    .text_sm()
                                    .text_color(cx.theme().text_muted),
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
            .refine_style(&self.style)
    }

    pub(crate) fn reset(&self, window: &mut Window, cx: &mut App) {
        for item in &self.items {
            item.reset(window, cx);
        }
    }
}
