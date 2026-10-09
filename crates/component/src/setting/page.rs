use std::rc::Rc;

use gpui::{
    AnyElement, App, Entity, FontWeight, InteractiveElement as _, IntoElement, ListAlignment,
    ListOffset, ListState, ParentElement as _, SharedString, StyleRefinement, Styled, Window, div,
    list, prelude::FluentBuilder as _, px,
};
use rust_i18n::t;

use crate::{
    ActiveTheme, Icon, IconName, Sizable, StyledExt, StyledTypography as _,
    button::{Button, ButtonVariants},
    h_flex,
    label::Label,
    scroll::ScrollableElement,
    setting::{RenderOptions, SettingGroup, settings::SettingsState},
    v_flex,
};

type ContentRenderer = Rc<dyn Fn(&RenderOptions, &mut Window, &mut App) -> Option<AnyElement>>;

/// A setting page that can contain multiple setting groups.
#[derive(Clone)]
pub struct SettingPage {
    pub(super) icon: Option<Icon>,
    resettable: bool,
    pub(super) default_open: bool,
    pub(super) title: SharedString,
    pub(super) title_suffix: Option<Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>>,
    pub(super) description: Option<SharedString>,
    pub(super) groups: Vec<SettingGroup>,
    pub(super) header_style: StyleRefinement,
    content: Option<ContentRenderer>,
    pub(super) sidebar_groups_visible: Option<Rc<dyn Fn(&App) -> bool>>,
}

impl SettingPage {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            icon: None,
            resettable: true,
            default_open: false,
            title: title.into(),
            title_suffix: None,
            description: None,
            groups: Vec::new(),
            header_style: StyleRefinement::default(),
            content: None,
            sidebar_groups_visible: None,
        }
    }

    /// Set the title of the setting page.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    /// Set a custom element to render after the title in the page header.
    ///
    /// For example, an info icon button that opens the help documentation.
    pub fn title_suffix<F, E>(mut self, suffix: F) -> Self
    where
        E: IntoElement,
        F: Fn(&mut Window, &mut App) -> E + 'static,
    {
        self.title_suffix = Some(Rc::new(move |window, cx| {
            suffix(window, cx).into_any_element()
        }));
        self
    }

    /// Set the icon of the setting page.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the description of the setting page, default is None.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the default open state of the setting page, default is false.
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    /// Set whether the setting page is resettable, default is true.
    ///
    /// If true and the items in this page has changed, the reset button will appear.
    pub fn resettable(mut self, resettable: bool) -> Self {
        self.resettable = resettable;
        self
    }

    /// Add a setting group to the page.
    pub fn group(mut self, group: SettingGroup) -> Self {
        self.groups.push(group);
        self
    }

    /// Add multiple setting groups to the page.
    pub fn groups(mut self, groups: impl IntoIterator<Item = SettingGroup>) -> Self {
        self.groups.extend(groups);
        self
    }

    /// Set the style refinement for the header of the setting page.
    pub fn header_style(mut self, style: &StyleRefinement) -> Self {
        self.header_style = style.clone();
        self
    }

    /// Optionally replace the page header and groups with application content.
    ///
    /// Return `None` for the normal settings page. Its group list and scroll
    /// position are retained across content changes and sidebar navigation.
    /// A nonempty settings search always shows the matching normal groups. The
    /// application owns custom content sizing, scrolling, focus, and navigation
    /// state, and must notify its owning view when that state changes.
    pub fn content<F, E>(mut self, render: F) -> Self
    where
        E: IntoElement,
        F: Fn(&RenderOptions, &mut Window, &mut App) -> Option<E> + 'static,
    {
        self.content = Some(Rc::new(move |options, window, cx| {
            render(options, window, cx).map(IntoElement::into_any_element)
        }));
        self
    }

    /// Set whether the page's groups appear in the sidebar (default: visible).
    ///
    /// This only affects navigation links, not page content or search matching.
    /// Search results always expose their matching group links.
    pub fn sidebar_groups_visible<F>(mut self, visible: F) -> Self
    where
        F: Fn(&App) -> bool + 'static,
    {
        self.sidebar_groups_visible = Some(Rc::new(visible));
        self
    }

    fn is_resettable(&self, query: &str, cx: &App) -> bool {
        self.resettable
            && self
                .groups
                .iter()
                .any(|group| group.is_resettable(query, cx))
    }

    fn reset_all(&self, query: &str, window: &mut Window, cx: &mut App) {
        for group in &self.groups {
            group.reset(query, window, cx);
        }
    }

    pub(super) fn render(
        &self,
        ix: usize,
        group_indices: &[usize],
        state: &Entity<SettingsState>,
        options: &RenderOptions,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        let search_input = state.read(cx).search_input.clone();
        let query = search_input.read(cx).value();
        let groups = group_indices
            .iter()
            .map(|&ix| (ix, self.groups[ix].clone()))
            .collect::<Vec<_>>();
        let groups_count = groups.len();

        let (list_state, changed) = state.update(cx, |state, _| {
            let page = state.pages.entry(ix).or_insert_with(|| PageState {
                list: ListState::new(groups_count, ListAlignment::Top, px(100.)),
                query: query.clone(),
                groups: group_indices.to_vec(),
            });
            let changed = page.query != query || page.groups != group_indices;
            if changed {
                page.list.reset(groups_count);
                page.query = query.clone();
                page.groups = group_indices.to_vec();
            }
            (page.list.clone(), changed)
        });

        let deferred_scroll_group_ix = state.read(cx).deferred_scroll_group_ix;
        let scroll_group_ix = deferred_scroll_group_ix.or_else(|| {
            changed
                .then_some(state.read(cx).selected_index.group_ix)
                .flatten()
        });
        if deferred_scroll_group_ix.is_some() {
            state.update(cx, |state, _| state.deferred_scroll_group_ix = None);
        }
        if let Some(group_ix) = scroll_group_ix
            && let Some(visible_ix) = group_indices.iter().position(|&ix| ix == group_ix)
        {
            // Scroll by index so jumps also work before the target groups have
            // been measured; `scroll_to_reveal_item` would resolve to the top.
            list_state.scroll_to(ListOffset {
                item_ix: visible_ix,
                offset_in_item: px(0.),
            });
        }

        // Search must reveal its matched controls, including while a nested
        // application page is open. The application keeps its own draft state.
        if query.is_empty()
            && let Some(content) = self
                .content
                .as_ref()
                .and_then(|render| render(&options.with_page_ix(ix), window, cx))
        {
            return div().id(ix).size_full().child(content).into_any_element();
        }

        v_flex()
            .id(ix)
            .size_full()
            .child(
                v_flex()
                    .p_4()
                    .gap_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .refine_style(&self.header_style)
                    // The page title is the page's one heading — semibold so it
                    // outranks the group titles below, which stay regular. Size
                    // is inherited, so `header_style` can re-tier it without
                    // touching the description (whose Settings size still wins).
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                // Keep the title semibold (fork) while carrying
                                // upstream's optional suffix slot (e.g. a help
                                // icon) alongside it in the same header row.
                                h_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(self.title.clone()),
                                    )
                                    .when_some(self.title_suffix.clone(), |this, suffix| {
                                        this.child(suffix(window, cx))
                                    }),
                            )
                            .when(self.is_resettable(&query, cx), |this| {
                                this.child(
                                    Button::new("reset")
                                        .icon(IconName::Undo2)
                                        .ghost()
                                        .small()
                                        .tooltip(t!("Settings.Reset All"))
                                        .on_click({
                                            let page = self.clone();
                                            let query = query.clone();
                                            move |_, window, cx| {
                                                page.reset_all(&query, window, cx);
                                            }
                                        }),
                                )
                            }),
                    )
                    .when_some(self.description.clone(), |this, description| {
                        this.child(
                            Label::new(description)
                                .text_ui_sm(cx)
                                .text_color(cx.theme().muted_foreground),
                        )
                    }),
            )
            .child(
                div()
                    .px_4()
                    .relative()
                    .flex_1()
                    .w_full()
                    .child(
                        list(list_state.clone(), {
                            let query = query.clone();
                            let options = *options;
                            move |visible_ix, window, cx| {
                                let (group_ix, group) = groups[visible_ix].clone();
                                group
                                    .py_4()
                                    .render(
                                        &query,
                                        &options.with_page_ix(ix).with_group_ix(group_ix),
                                        window,
                                        cx,
                                    )
                                    .into_any_element()
                            }
                        })
                        .size_full(),
                    )
                    .vertical_scrollbar(&list_state),
            )
            .into_any_element()
    }
}

pub(super) struct PageState {
    list: ListState,
    query: SharedString,
    groups: Vec<usize>,
}
