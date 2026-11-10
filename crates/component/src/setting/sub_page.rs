//! Detail pages stacked over a settings page.
//!
//! Only the last page on the stack is drawn. Each one keeps its own scroll position,
//! and going back returns focus to what was focused when the page opened.
use std::{collections::HashMap, rc::Rc};

use gpui::{
    AnyElement, App, Context, Entity, FocusHandle, InteractiveElement as _, IntoElement,
    ParentElement as _, ScrollHandle, SharedString, StatefulInteractiveElement as _,
    StyleRefinement, Styled, WeakEntity, Window, div, prelude::FluentBuilder as _,
};
use rust_i18n::t;

use crate::{
    ActiveTheme, IconName, Sizable as _, StyledExt as _, StyledTypography as _,
    breadcrumb::{Breadcrumb, BreadcrumbItem},
    button::{Button, ButtonVariants as _},
    h_flex,
    input::InputState,
    label::Label,
    scroll::ScrollableElement as _,
    setting::{RenderOptions, SettingGroup, SettingItem},
    v_flex,
};

type ContentRenderer = Rc<dyn Fn(&RenderOptions, &mut Window, &mut App) -> AnyElement>;
type ActionsRenderer = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// A detail page opened over a [`SettingPage`](crate::setting::SettingPage).
///
/// Open one from a [`SettingItem::sub_page`](crate::setting::SettingItem::sub_page) row, or
/// from code through [`SettingsNavigation::push`].
#[derive(Clone)]
pub struct SettingSubPage {
    pub(super) title: SharedString,
    description: Option<SharedString>,
    groups: Vec<SettingGroup>,
    content: Option<ContentRenderer>,
    actions: Option<ActionsRenderer>,
}

impl SettingSubPage {
    /// Create a sub-page. The title is its last breadcrumb.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            groups: Vec::new(),
            content: None,
            actions: None,
        }
    }

    /// Set the description shown under the breadcrumbs.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a setting group to the page.
    pub fn group(mut self, group: SettingGroup) -> Self {
        self.groups.push(group);
        self
    }

    /// Add setting groups to the page.
    pub fn groups(mut self, groups: impl IntoIterator<Item = SettingGroup>) -> Self {
        self.groups.extend(groups);
        self
    }

    /// Draw custom content instead of the groups.
    ///
    /// The application owns the sizing and scrolling of this content.
    pub fn content<F, E>(mut self, render: F) -> Self
    where
        E: IntoElement,
        F: Fn(&RenderOptions, &mut Window, &mut App) -> E + 'static,
    {
        self.content = Some(Rc::new(move |options, window, cx| {
            render(options, window, cx).into_any_element()
        }));
        self
    }

    /// Draw actions at the right edge of the breadcrumb row.
    pub fn actions<F, E>(mut self, render: F) -> Self
    where
        E: IntoElement,
        F: Fn(&mut Window, &mut App) -> E + 'static,
    {
        self.actions = Some(Rc::new(move |window, cx| {
            render(window, cx).into_any_element()
        }));
        self
    }

    /// Whether search should find this page's link through its contents.
    pub(super) fn is_match(&self, query: &str, cx: &App) -> bool {
        self.groups.iter().any(|group| group.is_match(query, cx))
    }
}

struct SubPageFrame {
    page: SettingSubPage,
    /// The title of the group holding the link that opened this page.
    section: Option<SharedString>,
    /// The (group, item) of that link, to pick up the page as it is rebuilt.
    link: Option<(usize, usize)>,
    scroll: ScrollHandle,
    focus: FocusHandle,
    return_focus: Option<FocusHandle>,
}

/// The stack of sub-pages open in a [`Settings`](crate::setting::Settings).
///
/// `Settings` creates one by default. Create your own and pass it to
/// [`Settings::navigation`](crate::setting::Settings::navigation) to open or close pages from
/// code, for example to show a provider that was just added.
#[derive(Default)]
pub struct SettingsNavigation {
    stack: Vec<SubPageFrame>,
    /// Link focus outlives the link, which is not drawn while its page is covered.
    link_focus: HashMap<(usize, usize, usize, usize), FocusHandle>,
    search: Option<WeakEntity<InputState>>,
}

impl SettingsNavigation {
    /// Create an empty stack.
    pub fn new() -> Self {
        Self::default()
    }

    /// How many sub-pages are open.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Open a page over the current one and move focus to it, ending any search.
    ///
    /// Unlike a page opened from a link, this one is not rebuilt with the settings
    /// page. Its closures should read live state.
    pub fn push(&mut self, page: SettingSubPage, window: &mut Window, cx: &mut Context<Self>) {
        self.push_from(page, None, None, window, cx);
    }

    pub(super) fn set_search(&mut self, search: &Entity<InputState>) {
        self.search = Some(search.downgrade());
    }

    pub(super) fn push_from(
        &mut self,
        page: SettingSubPage,
        section: Option<SharedString>,
        link: Option<(usize, usize)>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let focus = cx.focus_handle();
        let return_focus = window.focused(cx);
        // Search results are drawn in place of an open page, so opening one ends the search.
        if let Some(search) = self.search.as_ref().and_then(WeakEntity::upgrade) {
            search.update(cx, |input, cx| input.set_value("", window, cx));
        }
        window.focus(&focus, cx);
        self.stack.push(SubPageFrame {
            page,
            section,
            link,
            scroll: ScrollHandle::new(),
            focus,
            return_focus,
        });
        cx.notify();
    }

    /// Close the top page, returning to the one below it.
    pub fn pop(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(depth) = self.depth().checked_sub(1) {
            self.pop_to(depth, window, cx);
        }
    }

    /// Keep the first `depth` pages. Zero returns to the settings page itself.
    pub fn pop_to(&mut self, depth: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(frame) = self.stack.get_mut(depth) else {
            return;
        };
        match frame.return_focus.take() {
            Some(focus) => window.focus(&focus, cx),
            None => window.blur(cx),
        }
        self.stack.truncate(depth);
        cx.notify();
    }

    /// Take the open pages from their links in `groups`, which are rebuilt every render.
    /// A page whose link has gone stays as it was, and so do the pages above it.
    pub(super) fn refresh(&mut self, groups: &[SettingGroup]) {
        if self.stack.is_empty() {
            return;
        }
        let mut groups = groups.to_vec();
        for frame in &mut self.stack {
            let Some((group_ix, item_ix)) = frame.link else {
                return;
            };
            let Some(SettingItem::SubPage { page, .. }) = groups
                .get(group_ix)
                .and_then(|group| group.items.get(item_ix))
            else {
                return;
            };
            frame.page = page.clone();
            groups = frame.page.groups.clone();
        }
    }

    /// Close every page, leaving focus alone, for navigation that goes elsewhere.
    pub(super) fn clear(&mut self, cx: &mut Context<Self>) {
        if !self.stack.is_empty() {
            self.stack.clear();
            cx.notify();
        }
    }

    pub(super) fn link_focus(
        &mut self,
        options: &RenderOptions,
        cx: &mut Context<Self>,
    ) -> FocusHandle {
        let key = (
            options.page_ix(),
            options.group_ix(),
            options.item_ix(),
            self.depth(),
        );
        self.link_focus
            .entry(key)
            .or_insert_with(|| cx.focus_handle())
            .clone()
    }
}

/// What a link row needs from the page drawing it.
pub(super) struct SubPageLink {
    pub(super) navigation: Entity<SettingsNavigation>,
    /// The title of the group holding the row, for the breadcrumbs.
    pub(super) section: Option<SharedString>,
}

/// The top sub-page, or `None` when the settings page itself is showing.
pub(super) fn render_top(
    navigation: &Entity<SettingsNavigation>,
    page_title: SharedString,
    header_style: &StyleRefinement,
    options: &RenderOptions,
    window: &mut Window,
    cx: &mut App,
) -> Option<AnyElement> {
    let stack = &navigation.read(cx).stack;
    let frame = stack.last()?;
    let depth = stack.len();
    let page = frame.page.clone();
    let scroll = frame.scroll.clone();
    let focus = frame.focus.clone();

    // (label, depth it goes back to), then every crumb but the current page links.
    let mut crumbs = vec![(page_title, 0)];
    crumbs.extend(stack[0].section.clone().map(|section| (section, 0)));
    crumbs.extend(
        stack
            .iter()
            .enumerate()
            .map(|(ix, frame)| (frame.page.title.clone(), ix + 1)),
    );
    let current = crumbs.len() - 1;
    let breadcrumb =
        Breadcrumb::new().children(crumbs.into_iter().enumerate().map(|(ix, (label, target))| {
            let navigation = navigation.clone();
            BreadcrumbItem::new(label).when(ix != current, |item| {
                item.on_click(move |_, window, cx| {
                    navigation.update(cx, |navigation, cx| navigation.pop_to(target, window, cx));
                })
            })
        }));

    let actions = page.actions.as_ref().map(|render| render(window, cx));
    let header = v_flex()
        .p_4()
        .gap_3()
        .border_b_1()
        .border_color(cx.theme().border)
        .refine_style(header_style)
        .child(
            h_flex()
                .gap_2()
                .child(
                    Button::new("settings-sub-page-back")
                        .icon(IconName::ArrowLeft)
                        .ghost()
                        .small()
                        .tooltip(t!("Settings.Back").to_string())
                        .accessibility_label(t!("Settings.Back").to_string())
                        .when(cfg!(test), |this| {
                            this.debug_selector(|| "setting-sub-page-back".into())
                        })
                        .on_click({
                            let navigation = navigation.clone();
                            move |_, window, cx| {
                                navigation.update(cx, |navigation, cx| navigation.pop(window, cx));
                            }
                        }),
                )
                .child(div().flex_1().min_w_0().child(breadcrumb))
                .children(actions),
        )
        .when_some(page.description.clone(), |this, description| {
            this.child(
                Label::new(description)
                    .text_ui_sm(cx)
                    .text_color(cx.theme().muted_foreground),
            )
        });

    let body = match &page.content {
        Some(render) => render(options, window, cx),
        None => {
            let groups = page
                .groups
                .into_iter()
                .enumerate()
                .map(|(group_ix, group)| {
                    group
                        .py_4()
                        .render("", &options.with_group_ix(group_ix), navigation, window, cx)
                        .into_any_element()
                })
                .collect::<Vec<_>>();
            div()
                .relative()
                .size_full()
                .child(
                    v_flex()
                        .id("settings-sub-page-scroll")
                        .size_full()
                        .px_4()
                        .overflow_y_scroll()
                        .track_scroll(&scroll)
                        .children(groups),
                )
                .vertical_scrollbar(&scroll)
                .into_any_element()
        }
    };

    Some(
        v_flex()
            // Keeps a sub-page's element state apart from the page below it.
            .id(("settings-sub-page", depth))
            .track_focus(&focus)
            .size_full()
            .child(header)
            .child(div().flex_1().min_h_0().w_full().child(body))
            .into_any_element(),
    )
}
