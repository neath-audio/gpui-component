use super::*;
use crate::{
    Root,
    group_box::GroupBoxVariant,
    setting::{SettingGroup, SettingItem, SettingSubPage},
};
use gpui::{
    Context, Focusable as _, KeyBinding, KeyDownEvent, KeyUpEvent, Keystroke, Render, ScrollDelta,
    ScrollEvent, TestAppContext, VisualTestContext, actions, point, size,
};

actions!(settings_test, [Dismiss]);

struct SettingsHost {
    pages: Vec<SettingPage>,
    state: Option<Entity<SettingsState>>,
    /// Escape presses that reached the host, like an enclosing dialog's.
    dismissed: usize,
}

impl Render for SettingsHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = Settings::new("search-test")
            .pages(self.pages.clone())
            .default_selected_index(SelectIndex {
                page_ix: 1,
                group_ix: None,
            })
            .render(window, cx)
            .into_any_element();
        self.state = Some(window.use_keyed_state("search-test", cx, |_, _| unreachable!()));
        div()
            .size_full()
            .key_context("SettingsHost")
            .on_action(cx.listener(|this, _: &Dismiss, _, _| this.dismissed += 1))
            .child(settings)
    }
}

fn item(keyword: &'static str) -> SettingItem {
    item_with_height(keyword, 80.)
}

fn item_with_height(keyword: &'static str, height: f32) -> SettingItem {
    SettingItem::render(move |options, _, _| {
        let selector = format!(
            "setting-{}-{}-{}",
            options.page_ix(),
            options.group_ix(),
            options.item_ix()
        );
        div()
            .h(px(height))
            .child("Setting")
            .debug_selector(move || selector.clone())
    })
    .keywords([keyword])
}

fn setup(cx: &mut TestAppContext) -> (Entity<SettingsHost>, &mut VisualTestContext) {
    cx.update(|cx| {
        crate::init(cx);
        crate::Theme::global_mut(cx).font_size = px(16.);
        cx.bind_keys([KeyBinding::new("escape", Dismiss, Some("SettingsHost"))]);
    });
    let pages = vec![
        SettingPage::new("General").group(SettingGroup::new().item(item("language"))),
        SettingPage::new("Appearance").default_open(true).groups([
            SettingGroup::new().item(item("unrelated")),
            SettingGroup::new()
                .title("Colors")
                .item(item("theme colors")),
            SettingGroup::new()
                .title("Fonts")
                .items([item("unrelated"), item("theme font")]),
        ]),
        SettingPage::new("Editor").group(SettingGroup::new().item(item("theme editor"))),
    ];
    let mut host = None;
    let (_, cx) = cx.add_window_view(|window, cx| {
        let view = cx.new(|_| SettingsHost {
            pages,
            state: None,
            dismissed: 0,
        });
        host = Some(view.clone());
        Root::new(view, window, cx)
    });
    cx.simulate_resize(size(px(1000.), px(700.)));
    draw(cx);
    (host.unwrap(), cx)
}

fn draw(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

fn search(host: &Entity<SettingsHost>, query: &str, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        let input = host
            .read(cx)
            .state
            .as_ref()
            .unwrap()
            .read(cx)
            .search_input
            .clone();
        input.update(cx, |input, cx| input.set_value(query, window, cx));
    });
    draw(cx);
}

fn selection(host: &Entity<SettingsHost>, cx: &mut VisualTestContext) -> (usize, Option<usize>) {
    cx.update(|_, cx| {
        let selected = host
            .read(cx)
            .state
            .as_ref()
            .unwrap()
            .read(cx)
            .selected_index;
        (selected.page_ix, selected.group_ix)
    })
}

/// A point inside the second row of the fixture's sidebar.
const SECOND_SIDEBAR_ROW: f32 = 100.;

fn click_nav(y: f32, cx: &mut VisualTestContext) {
    // Fixed-size fixture with a 16px rem; click the label area, away from carets.
    cx.simulate_click(point(px(50.), px(y)), Default::default());
    draw(cx);
}

#[gpui::test]
fn search_preserves_the_page_and_clicks_use_original_indices(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    // The old numeric index is still in range, but would point to Editor.
    search(&host, "theme", cx);
    assert_eq!(selection(&host, cx), (1, None));
    assert!(cx.debug_bounds("setting-1-1-0").is_some());
    assert!(cx.debug_bounds("setting-2-0-0").is_none());

    // Only Appearance matches; the old implementation renders a blank page.
    search(&host, "font", cx);
    assert_eq!(selection(&host, cx), (1, None));
    assert!(cx.debug_bounds("setting-1-2-1").is_some());

    // Select Editor from the compressed result list, then clear the query.
    search(&host, "theme", cx);
    click_nav(156., cx);
    assert_eq!(selection(&host, cx), (2, None));
    search(&host, "", cx);
    assert_eq!(selection(&host, cx), (2, None));
    assert!(cx.debug_bounds("setting-2-0-0").is_some());

    // The current page disappears: choose the first matching page.
    search(&host, "font", cx);
    assert_eq!(selection(&host, cx), (1, None));
    assert!(cx.debug_bounds("setting-1-2-1").is_some());
    search(&host, "no matching setting", cx);
    assert_eq!(selection(&host, cx), (1, None));
    assert!(cx.debug_bounds("setting-1-2-1").is_none());
    assert!(cx.debug_bounds("settings-no-results").is_some());
    search(&host, "", cx);
    assert_eq!(selection(&host, cx), (1, None));
    assert!(cx.debug_bounds("settings-no-results").is_none());
}

#[gpui::test]
fn escape_clears_the_search_and_passes_through_when_empty(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let input = cx.update(|_, cx| {
        host.read(cx)
            .state
            .as_ref()
            .unwrap()
            .read(cx)
            .search_input
            .clone()
    });
    cx.update(|window, cx| input.update(cx, |input, cx| input.focus(window, cx)));
    draw(cx);
    cx.simulate_input("font");
    draw(cx);
    assert!(cx.debug_bounds("setting-1-1-0").is_none());

    // A query is cleared, and the host does not see the Escape.
    cx.simulate_keystrokes("escape");
    draw(cx);
    assert_eq!(cx.update(|_, cx| input.read(cx).value()), "");
    assert!(cx.debug_bounds("setting-1-1-0").is_some());
    assert_eq!(cx.update(|_, cx| host.read(cx).dismissed), 0);

    // An empty query leaves Escape to the host.
    cx.simulate_keystrokes("escape");
    assert_eq!(cx.update(|_, cx| host.read(cx).dismissed), 1);
}

#[gpui::test]
fn scrolling_marks_the_group_at_the_top(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    cx.update(|_, cx| {
        host.update(cx, |host, cx| {
            host.pages[1].groups[1].items = vec![item_with_height("theme colors", 900.)];
            cx.notify();
        });
    });
    draw(cx);
    assert_eq!(selection(&host, cx), (1, None));

    let scroll = |delta: f32, cx: &mut VisualTestContext| {
        cx.simulate_event(ScrollEvent {
            position: point(px(600.), px(400.)),
            delta: ScrollDelta::Pixels(point(px(0.), px(delta))),
            ..Default::default()
        });
        draw(cx);
    };
    // Past the untitled first group, Colors reaches the top.
    scroll(-200., cx);
    assert_eq!(selection(&host, cx), (1, Some(1)));
    // At the end of the list, the last group is marked.
    scroll(-2000., cx);
    assert_eq!(selection(&host, cx), (1, Some(2)));
    // Back at the top, the untitled group marks the page itself.
    scroll(4000., cx);
    assert_eq!(selection(&host, cx), (1, None));
}

#[gpui::test]
fn empty_search_keeps_input_editable_and_restores_selected_page(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let input = cx.update(|window, cx| {
        let input = host
            .read(cx)
            .state
            .as_ref()
            .unwrap()
            .read(cx)
            .search_input
            .clone();
        input.update(cx, |input, cx| input.focus(window, cx));
        input
    });
    draw(cx);
    assert!(cx.debug_bounds("settings-no-results").is_none());

    cx.simulate_input("no matching setting");
    draw(cx);
    assert!(cx.debug_bounds("settings-no-results").is_some());
    assert!(cx.debug_bounds("setting-1-2-1").is_none());
    assert_eq!(selection(&host, cx), (1, None));
    cx.update(|window, cx| {
        assert!(input.read(cx).focus_handle(cx).is_focused(window));
    });

    // Continue typing without refocusing while the empty state is visible.
    cx.simulate_input("x");
    draw(cx);
    cx.update(|window, cx| {
        assert_eq!(input.read(cx).value(), "no matching settingx");
        assert!(input.read(cx).focus_handle(cx).is_focused(window));
    });
    assert!(cx.debug_bounds("settings-no-results").is_some());

    #[cfg(target_os = "macos")]
    cx.simulate_keystrokes("cmd-a backspace");
    #[cfg(not(target_os = "macos"))]
    cx.simulate_keystrokes("ctrl-a backspace");
    draw(cx);
    cx.update(|window, cx| {
        assert!(input.read(cx).value().is_empty());
        assert!(input.read(cx).focus_handle(cx).is_focused(window));
    });
    assert!(cx.debug_bounds("settings-no-results").is_none());
    assert_eq!(selection(&host, cx), (1, None));
    assert!(cx.debug_bounds("setting-1-2-1").is_some());
}

#[gpui::test]
fn search_preserves_group_and_item_identity(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    // Make the target group require scrolling, with an unnamed group before it.
    cx.update(|_, cx| {
        host.update(cx, |host, cx| {
            host.pages[1].groups[1].items = vec![item_with_height("theme colors", 450.)];
            cx.notify();
        });
    });
    draw(cx);
    click_nav(156., cx);
    assert_eq!(selection(&host, cx), (1, Some(2)));
    search(&host, "theme", cx);
    assert_eq!(selection(&host, cx), (1, Some(2)));
    assert!(cx.debug_bounds("setting-1-2-1").is_some());
    assert!(cx.debug_bounds("setting-1-2-0").is_none());
    // Click Fonts after the leading page/group/item have been filtered out.
    click_nav(120., cx);
    assert_eq!(selection(&host, cx), (1, Some(2)));
    let target = cx.debug_bounds("setting-1-2-1").unwrap();
    assert!(target.top() >= px(0.) && target.bottom() <= px(700.));
    search(&host, "font", cx);
    assert_eq!(selection(&host, cx), (1, Some(2)));
    search(&host, "", cx);
    assert_eq!(selection(&host, cx), (1, Some(2)));
    search(&host, "colors", cx);
    assert_eq!(selection(&host, cx), (1, None));
    assert!(cx.debug_bounds("setting-1-1-0").is_some());
}

#[gpui::test]
fn search_placeholder_follows_the_locale_and_keeps_the_page(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let (state, input) = cx.update(|_, cx| {
        let state = host.read(cx).state.clone().unwrap();
        let input = state.read(cx).search_input.clone();
        (state, input)
    });

    // The locale is process-wide and tests run in parallel, so stand in for a
    // switch with a state that was created under another locale.
    cx.update(|window, cx| {
        state.update(cx, |state, _| {
            state.selected_index = SelectIndex {
                page_ix: 2,
                group_ix: None,
            };
        });
        input.update(cx, |input, cx| {
            let placeholder = t!("Settings.search_placeholder", locale = "zh-CN");
            input.set_placeholder(placeholder, window, cx);
        });
        // As an application does after `set_locale`.
        window.refresh();
    });
    draw(cx);

    cx.update(|_, cx| {
        let placeholder = input.read(cx).presentation().placeholder().clone();
        assert_eq!(placeholder, &*t!("Settings.search_placeholder"));
    });
    assert_eq!(selection(&host, cx), (2, None));
}

#[gpui::test]
fn footer_follows_group_search_visibility(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    cx.update(|_, cx| {
        host.update(cx, |host, cx| {
            host.pages[1].groups[2] = host.pages[1].groups[2].clone().footer(|_, _| {
                div()
                    .child("Changes apply to this device only.")
                    .debug_selector(|| "font-footer".into())
            });
            cx.notify();
        });
    });
    search(&host, "font", cx);
    assert!(cx.debug_bounds("setting-1-2-1").is_some());
    assert!(cx.debug_bounds("font-footer").is_some());

    // Footer copy does not independently make a group match the query.
    search(&host, "colors", cx);
    assert!(cx.debug_bounds("setting-1-1-0").is_some());
    assert!(cx.debug_bounds("font-footer").is_none());

    search(&host, "font", cx);
    assert!(cx.debug_bounds("font-footer").is_some());
    assert_eq!(selection(&host, cx), (1, None));
}

#[gpui::test]
fn resetting_search_results_leaves_hidden_settings_unchanged(cx: &mut TestAppContext) {
    use std::{cell::Cell, rc::Rc};

    let (_, cx) = setup(cx);
    let visible = Rc::new(Cell::new(false));
    let hidden = Rc::new(Cell::new(true));
    let resettable_item = |keyword, dirty: &Rc<Cell<bool>>| {
        let read = dirty.clone();
        let reset = dirty.clone();
        item(keyword).on_reset(move |_| read.get(), move |_, _| reset.set(false))
    };
    let group = SettingGroup::new().items([
        resettable_item("theme", &visible),
        resettable_item("hidden", &hidden),
    ]);
    cx.update(|window, cx| {
        assert!(!group.is_resettable("theme", cx));
        visible.set(true);
        assert!(group.is_resettable("theme", cx));
        group.reset("theme", window, cx);
    });
    assert!(!visible.get());
    assert!(hidden.get());
}

#[gpui::test]
fn custom_page_content_replaces_the_page_and_restores_overview_scroll(cx: &mut TestAppContext) {
    use std::{cell::Cell, rc::Rc};

    let (host, cx) = setup(cx);
    let detail = Rc::new(Cell::new(false));
    cx.update(|_, cx| {
        host.update(cx, |host, cx| {
            let detail = detail.clone();
            host.pages[1] = SettingPage::new("Appearance")
                .sidebar_groups_visible({
                    let detail = detail.clone();
                    move |_| !detail.get()
                })
                .title_suffix(|_, _| {
                    div()
                        .child("Overview")
                        .debug_selector(|| "overview-title".into())
                })
                .groups([
                    SettingGroup::new().item(item_with_height("theme", 900.)),
                    SettingGroup::new()
                        .title("Colors")
                        .item(item_with_height("theme", 900.)),
                    SettingGroup::new()
                        .title("Fonts")
                        .item(item_with_height("theme", 900.)),
                ])
                .content(move |options, _, _| {
                    assert_eq!(options.page_ix(), 1);
                    detail.get().then(|| {
                        let detail = detail.clone();
                        div()
                            .size_full()
                            .debug_selector(|| "detail-page".into())
                            .child(
                                crate::button::Button::new("back")
                                    .label("Back")
                                    .on_click(move |_, window, _| {
                                        detail.set(false);
                                        window.refresh();
                                    })
                                    .debug_selector(|| "back-overview".into()),
                            )
                    })
                });
            cx.notify();
        });
    });
    draw(cx);
    click_nav(156., cx);
    cx.simulate_event(gpui::ScrollEvent {
        position: point(px(500.), px(300.)),
        delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-70.))),
        ..Default::default()
    });
    draw(cx);
    let before = cx.debug_bounds("setting-1-2-0").unwrap();
    assert!(
        before.top() < px(70.),
        "fixture is scrolled inside the final group"
    );

    detail.set(true);
    cx.update(|window, _| window.refresh());
    draw(cx);
    let page = cx
        .debug_bounds("detail-page")
        .expect("custom content replaces the page");
    assert_eq!(page.top(), px(0.));
    assert!(page.left() > px(0.), "sidebar keeps its own region");
    assert!(cx.debug_bounds("overview-title").is_none());
    assert!(cx.debug_bounds("setting-1-2-0").is_none());
    // Keep the original list state alive across multiple detail frames.
    cx.update(|window, _| window.refresh());
    draw(cx);
    let back = cx.debug_bounds("back-overview").unwrap();
    cx.simulate_click(back.center(), Default::default());
    draw(cx);
    assert!(cx.debug_bounds("detail-page").is_none());
    assert_eq!(cx.debug_bounds("setting-1-2-0").unwrap(), before);
    click_nav(156., cx);
    assert_eq!(
        selection(&host, cx),
        (1, Some(2)),
        "selected group links reopen on Back"
    );
    cx.simulate_event(gpui::ScrollEvent {
        position: point(px(500.), px(300.)),
        delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-70.))),
        ..Default::default()
    });
    draw(cx);

    detail.set(true);
    cx.update(|window, _| window.refresh());
    draw(cx);
    click_nav(60., cx);
    assert_eq!(selection(&host, cx), (0, None));
    assert!(cx.debug_bounds("setting-0-0-0").is_some());
    assert!(cx.debug_bounds("detail-page").is_none());
    click_nav(92., cx);
    let back = cx.debug_bounds("back-overview").unwrap();
    cx.simulate_click(back.center(), Default::default());
    draw(cx);
    assert_eq!(
        cx.debug_bounds("setting-1-2-0").unwrap(),
        before,
        "sidebar excursions retain the overview position too"
    );
}

#[gpui::test]
fn custom_content_hides_irrelevant_group_links_and_search_reveals_matching_settings(
    cx: &mut TestAppContext,
) {
    let (host, cx) = setup(cx);
    cx.update(|_, cx| {
        host.update(cx, |host, cx| {
            host.pages[1] = host.pages[1]
                .clone()
                .sidebar_groups_visible(|_| false)
                .content(|_, _, _| Some(div().size_full().debug_selector(|| "detail-page".into())));
            cx.notify();
        });
    });
    draw(cx);
    assert!(cx.debug_bounds("detail-page").is_some());
    // Editor follows Appearance directly; its hidden overview groups cannot
    // change the selected group or its retained scroll behind the detail page.
    click_nav(124., cx);
    assert_eq!(selection(&host, cx), (2, None));
    click_nav(92., cx);
    assert!(cx.debug_bounds("detail-page").is_some());
    search(&host, "font", cx);
    assert!(cx.debug_bounds("detail-page").is_none());
    assert!(cx.debug_bounds("setting-1-2-1").is_some());
    search(&host, "", cx);
    assert!(cx.debug_bounds("detail-page").is_some());
}

struct VariantHost {
    pages: Vec<SettingPage>,
    outline: bool,
}

impl Render for VariantHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut settings = Settings::new("variant-test").pages(self.pages.clone());
        if self.outline {
            settings = settings.with_group_variant(GroupBoxVariant::Outline);
        }
        div()
            .size_full()
            .child(settings.render(window, cx).into_any_element())
    }
}

fn variant_pages(override_first_group: bool) -> Vec<SettingPage> {
    let first = if override_first_group {
        SettingGroup::new().variant(GroupBoxVariant::Normal)
    } else {
        SettingGroup::new()
    };
    vec![SettingPage::new("General").groups([
        first.item(item("plain")),
        SettingGroup::new().item(item("outlined")),
    ])]
}

#[gpui::test]
fn group_variant_overrides_the_settings_default(cx: &mut TestAppContext) {
    cx.update(|cx| {
        crate::init(cx);
        crate::Theme::global_mut(cx).font_size = px(16.);
    });

    let open_window = |pages, outline, cx: &mut TestAppContext| {
        let (_, cx) = cx.add_window_view(|window, cx| {
            let view = cx.new(|_| VariantHost { pages, outline });
            Root::new(view, window, cx)
        });
        cx.simulate_resize(size(px(1000.), px(700.)));
        draw(cx);
        (
            cx.debug_bounds("setting-0-0-0").unwrap(),
            cx.debug_bounds("setting-0-1-0").unwrap(),
        )
    };

    // Baseline: no global variant, so both groups present their items directly.
    let (baseline_first, baseline_second) = open_window(variant_pages(false), false, cx);
    // A global Outline default with the first group overriding back to Normal.
    let (overridden_first, inherited_second) = open_window(variant_pages(true), true, cx);

    // The override presents the group exactly like the settings-level default.
    assert_eq!(
        f64::from(overridden_first.top() - baseline_first.top()),
        0.,
        "the overridden group must lose the global Outline chrome"
    );
    // A group without the override keeps the global Outline chrome:
    // a 1px border and 16px padding above its items.
    assert_eq!(
        f64::from(inherited_second.top() - baseline_second.top()),
        17.,
        "groups without an override must keep the settings-level variant"
    );
}

#[gpui::test]
fn selecting_a_group_from_another_page_scrolls_to_it(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    // Push Fonts below the viewport.
    cx.update(|_, cx| {
        host.update(cx, |host, cx| {
            host.pages[1].groups[1].items = vec![item_with_height("theme colors", 900.)];
            cx.notify();
        });
    });
    draw(cx);
    let select = |page_ix, group_ix: Option<usize>, cx: &mut VisualTestContext| {
        cx.update(|_, cx| {
            let state = host.read(cx).state.clone().unwrap();
            state.update(cx, |state, cx| {
                state.selected_index = SelectIndex { page_ix, group_ix };
                state.deferred_scroll_group_ix = group_ix;
                cx.notify();
            });
        });
        draw(cx);
    };

    // Returning to a specific group must honor the jump, not the retained top.
    select(0, None, cx);
    select(1, Some(2), cx);
    let target = cx.debug_bounds("setting-1-2-1").unwrap();
    assert!(target.top() >= px(0.) && target.bottom() <= px(700.));
}

#[gpui::test]
fn retained_settings_groups_remeasure_after_typography_changes(cx: &mut TestAppContext) {
    use crate::{StyledTypography as _, TextSize, Theme, UiTypography};
    use gpui::rems;

    let (host, cx) = setup(cx);
    cx.update(|_, cx| {
        host.update(cx, |host, cx| {
            host.pages[1].groups =
                vec![SettingGroup::new().item(SettingItem::render(|_, _, cx| {
                    crate::v_flex().w(px(160.)).children([
                        div()
                            .text_ui(cx)
                            .child("Retained setting text with several words 世界")
                            .debug_selector(|| "live-setting-text".into()),
                        div()
                            .text_size(px(13.))
                            .child("Explicit text with several words 世界")
                            .debug_selector(|| "explicit-setting-text".into()),
                    ])
                }))];
            cx.notify();
        });
    });
    let mut heights = Vec::new();
    for font_size in [13., 24., 13.] {
        cx.update(|window, cx| {
            Theme::update(cx, |theme| {
                theme.set_ui_typography(
                    UiTypography::default().with_size(TextSize::Default, rems(font_size / 16.)),
                )
            });
            window.refresh();
        });
        draw(cx);
        let live = cx.debug_bounds("live-setting-text").unwrap();
        let explicit = cx.debug_bounds("explicit-setting-text").unwrap();
        heights.push((live.size.height, explicit.size.height, explicit.top()));
        cx.update(|window, _| assert_eq!(window.rem_size(), px(16.)));
    }
    assert!(
        heights[1].0 > heights[0].0,
        "retained wrapped text must grow"
    );
    assert!(
        heights[1].2 > heights[0].2,
        "the following content must move with the new height"
    );
    assert_eq!(
        heights[0].1, heights[1].1,
        "explicit text size remains authoritative"
    );
    assert_eq!(heights[0], heights[2]);
}

struct SubPageHost {
    navigation: Entity<SettingsNavigation>,
    search: Entity<InputState>,
    pages: Vec<SettingPage>,
}

impl Render for SubPageHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(
            Settings::new("sub-page-test")
                .navigation(&self.navigation)
                .search_input(&self.search)
                .pages(self.pages.clone()),
        )
    }
}

fn marked(selector: &'static str, keyword: &'static str) -> SettingItem {
    SettingItem::render(move |_, _, _| {
        div()
            .h(px(40.))
            .child("Setting")
            .debug_selector(move || selector.to_string())
    })
    .keywords([keyword])
}

fn provider_page() -> SettingSubPage {
    SettingSubPage::new("Provider")
        .group(SettingGroup::new().item(marked("inside-sub-page", "api key")))
}

fn sub_page_setup(
    cx: &mut TestAppContext,
) -> (
    Entity<SettingsNavigation>,
    Entity<InputState>,
    &mut VisualTestContext,
) {
    let (_, navigation, search, cx) = sub_page_host(cx);
    (navigation, search, cx)
}

fn sub_page_host(
    cx: &mut TestAppContext,
) -> (
    Entity<SubPageHost>,
    Entity<SettingsNavigation>,
    Entity<InputState>,
    &mut VisualTestContext,
) {
    cx.update(|cx| {
        crate::init(cx);
        crate::Theme::global_mut(cx).font_size = px(16.);
    });
    let pages = vec![
        SettingPage::new("General").group(SettingGroup::new().title("Providers").items([
            marked("on-parent-page", "language"),
            SettingItem::sub_page("Provider", provider_page()),
        ])),
        SettingPage::new("Editor").group(SettingGroup::new().item(marked("other-page", "theme"))),
    ];
    let mut handles = None;
    let (_, cx) = cx.add_window_view(|window, cx| {
        let navigation = cx.new(|_| SettingsNavigation::new());
        let search = cx.new(|cx| InputState::new(window, cx));
        let view = cx.new(|_| SubPageHost {
            navigation: navigation.clone(),
            search: search.clone(),
            pages,
        });
        handles = Some((view.clone(), navigation, search));
        Root::new(view, window, cx)
    });
    cx.simulate_resize(size(px(1000.), px(700.)));
    draw(cx);
    let (host, navigation, search) = handles.unwrap();
    (host, navigation, search, cx)
}

fn depth(navigation: &Entity<SettingsNavigation>, cx: &mut VisualTestContext) -> usize {
    cx.update(|_, cx| navigation.read(cx).depth())
}

fn click(selector: &'static str, cx: &mut VisualTestContext) {
    let target = cx.debug_bounds(selector).unwrap();
    cx.simulate_click(target.center(), Default::default());
    draw(cx);
}

fn press_enter(cx: &mut VisualTestContext) {
    let keystroke = Keystroke::parse("enter").unwrap();
    cx.simulate_event(KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(KeyUpEvent { keystroke });
    draw(cx);
}

fn type_search(input: &Entity<InputState>, query: &str, cx: &mut VisualTestContext) {
    cx.update(|window, cx| input.update(cx, |input, cx| input.set_value(query, window, cx)));
    draw(cx);
}

#[gpui::test]
fn a_link_opens_its_sub_page_and_back_returns_to_the_page(cx: &mut TestAppContext) {
    let (navigation, _, cx) = sub_page_setup(cx);
    assert!(cx.debug_bounds("on-parent-page").is_some());
    assert!(cx.debug_bounds("inside-sub-page").is_none());

    click("setting-sub-page-link-Provider", cx);
    assert_eq!(depth(&navigation, cx), 1);
    assert!(cx.debug_bounds("inside-sub-page").is_some());
    assert!(cx.debug_bounds("on-parent-page").is_none());

    click("setting-sub-page-back", cx);
    assert_eq!(depth(&navigation, cx), 0);
    assert!(cx.debug_bounds("on-parent-page").is_some());
    assert!(cx.debug_bounds("inside-sub-page").is_none());
}

#[gpui::test]
fn going_back_returns_keyboard_focus_to_the_link_that_opened_the_page(cx: &mut TestAppContext) {
    let (navigation, _, cx) = sub_page_setup(cx);
    let options = RenderOptions::new()
        .with_page_ix(0)
        .with_group_ix(0)
        .with_item_ix(1);
    let link = cx.update(|_, cx| navigation.update(cx, |nav, cx| nav.link_focus(&options, cx)));
    cx.update(|window, cx| window.focus(&link, cx));
    draw(cx);
    press_enter(cx);
    assert_eq!(
        depth(&navigation, cx),
        1,
        "Enter on the link opens the page"
    );
    assert!(!cx.update(|window, _| link.is_focused(window)));

    click("setting-sub-page-back", cx);
    assert!(cx.update(|window, _| link.is_focused(window)));

    // The link was not drawn while covered; it must really hold focus again.
    press_enter(cx);
    assert_eq!(depth(&navigation, cx), 1, "Enter still activates the link");
}

#[gpui::test]
fn search_finds_a_link_through_what_its_sub_page_contains(cx: &mut TestAppContext) {
    let (_, search, cx) = sub_page_setup(cx);
    type_search(&search, "api key", cx);
    assert!(cx.debug_bounds("setting-sub-page-link-Provider").is_some());
    assert!(cx.debug_bounds("on-parent-page").is_none());
    type_search(&search, "no such setting", cx);
    assert!(cx.debug_bounds("setting-sub-page-link-Provider").is_none());
}

#[gpui::test]
fn search_shows_results_over_an_open_sub_page_and_clearing_restores_it(cx: &mut TestAppContext) {
    let (navigation, search, cx) = sub_page_setup(cx);
    click("setting-sub-page-link-Provider", cx);

    type_search(&search, "language", cx);
    assert!(cx.debug_bounds("on-parent-page").is_some());
    assert!(cx.debug_bounds("inside-sub-page").is_none());
    assert_eq!(depth(&navigation, cx), 1);

    type_search(&search, "", cx);
    assert!(cx.debug_bounds("inside-sub-page").is_some());
}

#[gpui::test]
fn leaving_the_page_closes_its_sub_pages(cx: &mut TestAppContext) {
    let (navigation, search, cx) = sub_page_setup(cx);
    click("setting-sub-page-link-Provider", cx);
    assert_eq!(depth(&navigation, cx), 1);

    // Only Editor matches, so the selection moves there.
    type_search(&search, "theme", cx);
    assert_eq!(depth(&navigation, cx), 0);
    type_search(&search, "", cx);
    assert!(cx.debug_bounds("other-page").is_some());
    assert!(cx.debug_bounds("inside-sub-page").is_none());
}

#[gpui::test]
fn clicking_the_sidebar_closes_sub_pages(cx: &mut TestAppContext) {
    let (navigation, _, cx) = sub_page_setup(cx);
    click("setting-sub-page-link-Provider", cx);
    assert_eq!(depth(&navigation, cx), 1);

    click_nav(SECOND_SIDEBAR_ROW, cx);
    assert_eq!(depth(&navigation, cx), 0);
    assert!(cx.debug_bounds("other-page").is_some());
}

#[gpui::test]
fn popping_to_a_depth_keeps_the_pages_below_it(cx: &mut TestAppContext) {
    let (navigation, _, cx) = sub_page_setup(cx);
    let nested = SettingSubPage::new("Model")
        .group(SettingGroup::new().item(marked("inside-nested-page", "model")));
    cx.update(|window, cx| {
        navigation.update(cx, |nav, cx| {
            nav.push(provider_page(), window, cx);
            nav.push(nested, window, cx);
        })
    });
    draw(cx);
    assert_eq!(depth(&navigation, cx), 2);
    assert!(cx.debug_bounds("inside-nested-page").is_some());

    cx.update(|window, cx| navigation.update(cx, |nav, cx| nav.pop_to(1, window, cx)));
    draw(cx);
    assert_eq!(depth(&navigation, cx), 1);
    assert!(cx.debug_bounds("inside-sub-page").is_some());
    assert!(cx.debug_bounds("inside-nested-page").is_none());
}

#[gpui::test]
fn opening_a_link_from_search_results_ends_the_search_and_shows_the_page(cx: &mut TestAppContext) {
    let (navigation, search, cx) = sub_page_setup(cx);
    type_search(&search, "api key", cx);
    click("setting-sub-page-link-Provider", cx);

    assert_eq!(depth(&navigation, cx), 1);
    assert!(cx.update(|_, cx| search.read(cx).value().is_empty()));
    assert!(cx.debug_bounds("inside-sub-page").is_some());
}

#[gpui::test]
fn an_open_sub_page_follows_the_page_as_it_is_rebuilt(cx: &mut TestAppContext) {
    let (host, navigation, _, cx) = sub_page_host(cx);
    click("setting-sub-page-link-Provider", cx);
    assert!(cx.debug_bounds("inside-sub-page").is_some());

    cx.update(|_, cx| {
        host.update(cx, |host, cx| {
            let rebuilt = SettingSubPage::new("Provider")
                .group(SettingGroup::new().item(marked("rebuilt-sub-page", "api key")));
            host.pages[0] =
                SettingPage::new("General").group(SettingGroup::new().title("Providers").items([
                    marked("on-parent-page", "language"),
                    SettingItem::sub_page("Provider", rebuilt),
                ]));
            cx.notify();
        });
    });
    draw(cx);

    assert_eq!(depth(&navigation, cx), 1);
    assert!(cx.debug_bounds("rebuilt-sub-page").is_some());
    assert!(cx.debug_bounds("inside-sub-page").is_none());
}

fn block_leaves(
    navigation: &Entity<SettingsNavigation>,
    seen: std::rc::Rc<std::cell::RefCell<Vec<usize>>>,
    cx: &mut VisualTestContext,
) {
    cx.update(|_, cx| {
        navigation.update(cx, |nav, _| {
            nav.set_leave_guard(move |target, _, _| {
                seen.borrow_mut().push(target);
                false
            })
        })
    });
}

fn has_pending_leave(navigation: &Entity<SettingsNavigation>, cx: &mut VisualTestContext) -> bool {
    cx.update(|_, cx| navigation.read(cx).has_pending_leave())
}

#[gpui::test]
fn a_named_link_row_can_be_targeted(cx: &mut TestAppContext) {
    let (navigation, _, cx) = sub_page_setup(cx);
    assert!(cx.debug_bounds("setting-sub-page-link-Provider").is_some());
    click("setting-sub-page-link-Provider", cx);
    assert_eq!(depth(&navigation, cx), 1);
    assert!(cx.debug_bounds("inside-sub-page").is_some());
}

#[gpui::test]
fn a_blocking_guard_holds_back_until_resumed(cx: &mut TestAppContext) {
    let (navigation, _, cx) = sub_page_setup(cx);
    click("setting-sub-page-link-Provider", cx);
    let seen = Default::default();
    block_leaves(&navigation, std::rc::Rc::clone(&seen), cx);

    click("setting-sub-page-back", cx);
    assert_eq!(depth(&navigation, cx), 1);
    assert!(cx.debug_bounds("inside-sub-page").is_some());
    assert_eq!(*seen.borrow(), vec![0]);
    assert!(has_pending_leave(&navigation, cx));

    cx.update(|window, cx| navigation.update(cx, |nav, cx| nav.resume_leave(window, cx)));
    draw(cx);
    assert_eq!(depth(&navigation, cx), 0);
    assert!(cx.debug_bounds("on-parent-page").is_some());
    assert!(cx.debug_bounds("inside-sub-page").is_none());
    assert!(!has_pending_leave(&navigation, cx));
}

#[gpui::test]
fn cancelling_a_held_leave_keeps_the_page(cx: &mut TestAppContext) {
    let (navigation, _, cx) = sub_page_setup(cx);
    click("setting-sub-page-link-Provider", cx);
    block_leaves(&navigation, Default::default(), cx);

    click("setting-sub-page-back", cx);
    assert!(has_pending_leave(&navigation, cx));
    cx.update(|window, cx| navigation.update(cx, |nav, cx| nav.cancel_leave(window, cx)));
    draw(cx);
    assert_eq!(depth(&navigation, cx), 1);
    assert!(cx.debug_bounds("inside-sub-page").is_some());
    assert!(!has_pending_leave(&navigation, cx));
}

#[gpui::test]
fn a_breadcrumb_asks_with_its_target_depth(cx: &mut TestAppContext) {
    let (navigation, _, cx) = sub_page_setup(cx);
    let nested = SettingSubPage::new("Model")
        .group(SettingGroup::new().item(marked("inside-nested-page", "model")));
    cx.update(|window, cx| {
        navigation.update(cx, |nav, cx| {
            nav.push(provider_page(), window, cx);
            nav.push(nested, window, cx);
        })
    });
    draw(cx);
    let seen = Default::default();
    block_leaves(&navigation, std::rc::Rc::clone(&seen), cx);

    click("setting-sub-page-crumb-0", cx);
    assert_eq!(*seen.borrow(), vec![0]);
    assert_eq!(depth(&navigation, cx), 2);
    assert!(cx.debug_bounds("inside-nested-page").is_some());

    cx.update(|window, cx| navigation.update(cx, |nav, cx| nav.resume_leave(window, cx)));
    draw(cx);
    assert_eq!(depth(&navigation, cx), 0);
    assert!(cx.debug_bounds("on-parent-page").is_some());
}

#[gpui::test]
fn an_allowing_guard_and_no_guard_leave_at_once(cx: &mut TestAppContext) {
    let (navigation, _, cx) = sub_page_setup(cx);
    click("setting-sub-page-link-Provider", cx);
    let seen: std::rc::Rc<std::cell::RefCell<Vec<usize>>> = Default::default();
    cx.update(|_, cx| {
        navigation.update(cx, |nav, _| {
            let seen = std::rc::Rc::clone(&seen);
            nav.set_leave_guard(move |target, _, _| {
                seen.borrow_mut().push(target);
                true
            })
        })
    });
    click("setting-sub-page-back", cx);
    assert_eq!(depth(&navigation, cx), 0);
    assert!(cx.debug_bounds("on-parent-page").is_some());
    assert_eq!(*seen.borrow(), vec![0]);
    assert!(!has_pending_leave(&navigation, cx));

    cx.update(|_, cx| navigation.update(cx, |nav, _| nav.clear_leave_guard()));
    click("setting-sub-page-link-Provider", cx);
    click("setting-sub-page-back", cx);
    assert_eq!(depth(&navigation, cx), 0);
    assert_eq!(seen.borrow().len(), 1);
}

#[gpui::test]
fn app_pops_are_not_guarded(cx: &mut TestAppContext) {
    let (navigation, _, cx) = sub_page_setup(cx);
    click("setting-sub-page-link-Provider", cx);
    let seen = Default::default();
    block_leaves(&navigation, std::rc::Rc::clone(&seen), cx);

    cx.update(|window, cx| navigation.update(cx, |nav, cx| nav.pop_to(0, window, cx)));
    draw(cx);
    assert_eq!(depth(&navigation, cx), 0);
    assert!(cx.debug_bounds("on-parent-page").is_some());
    assert!(seen.borrow().is_empty());
}

#[gpui::test]
fn a_page_pushed_in_a_section_shows_that_crumb(cx: &mut TestAppContext) {
    let (navigation, _, cx) = sub_page_setup(cx);
    cx.update(|window, cx| {
        navigation.update(cx, |nav, cx| {
            nav.push_in_section(provider_page(), "Providers", window, cx)
        })
    });
    draw(cx);
    // General, Providers, Provider.
    assert!(cx.debug_bounds("setting-sub-page-crumb-0").is_some());
    assert!(cx.debug_bounds("setting-sub-page-crumb-1").is_some());
    assert!(cx.debug_bounds("setting-sub-page-crumb-2").is_some());
    assert!(cx.debug_bounds("setting-sub-page-crumb-3").is_none());

    click("setting-sub-page-crumb-1", cx);
    assert_eq!(depth(&navigation, cx), 0);
    assert!(cx.debug_bounds("on-parent-page").is_some());
}

fn search_value(input: &Entity<InputState>, cx: &mut VisualTestContext) -> String {
    cx.update(|_, cx| input.read(cx).value().to_string())
}

#[gpui::test]
fn a_blocked_sidebar_click_keeps_the_sub_page_until_resumed(cx: &mut TestAppContext) {
    let (navigation, _, cx) = sub_page_setup(cx);
    click("setting-sub-page-link-Provider", cx);
    let seen = Default::default();
    block_leaves(&navigation, std::rc::Rc::clone(&seen), cx);

    click_nav(SECOND_SIDEBAR_ROW, cx);
    assert_eq!(*seen.borrow(), vec![0]);
    assert_eq!(depth(&navigation, cx), 1);
    assert!(cx.debug_bounds("inside-sub-page").is_some());
    assert!(cx.debug_bounds("other-page").is_none());

    cx.update(|window, cx| navigation.update(cx, |nav, cx| nav.resume_leave(window, cx)));
    draw(cx);
    assert_eq!(depth(&navigation, cx), 0);
    assert!(cx.debug_bounds("other-page").is_some());
    assert!(cx.debug_bounds("inside-sub-page").is_none());
}

#[gpui::test]
fn a_blocked_search_that_changes_page_reverts_the_query(cx: &mut TestAppContext) {
    let (navigation, search, cx) = sub_page_setup(cx);
    click("setting-sub-page-link-Provider", cx);
    let seen = Default::default();
    block_leaves(&navigation, std::rc::Rc::clone(&seen), cx);

    type_search(&search, "theme", cx);
    draw(cx);
    assert_eq!(depth(&navigation, cx), 1);
    assert_eq!(search_value(&search, cx), "");
    assert!(cx.debug_bounds("inside-sub-page").is_some());
    assert!(cx.debug_bounds("other-page").is_none());
    assert_eq!(*seen.borrow(), vec![0]);

    cx.update(|window, cx| navigation.update(cx, |nav, cx| nav.resume_leave(window, cx)));
    draw(cx);
    draw(cx);
    assert_eq!(search_value(&search, cx), "theme");
    assert_eq!(depth(&navigation, cx), 0);
    assert!(cx.debug_bounds("other-page").is_some());
    // Resuming does not ask again.
    assert_eq!(*seen.borrow(), vec![0]);
}

#[gpui::test]
fn search_results_over_an_open_page_do_not_ask(cx: &mut TestAppContext) {
    let (navigation, search, cx) = sub_page_setup(cx);
    click("setting-sub-page-link-Provider", cx);
    let seen = Default::default();
    block_leaves(&navigation, std::rc::Rc::clone(&seen), cx);

    type_search(&search, "language", cx);
    draw(cx);
    assert!(seen.borrow().is_empty());
    assert_eq!(depth(&navigation, cx), 1);
    assert!(cx.debug_bounds("on-parent-page").is_some());
    assert!(cx.debug_bounds("inside-sub-page").is_none());

    type_search(&search, "", cx);
    assert!(cx.debug_bounds("inside-sub-page").is_some());
}

/// Installs a guard that records the depth each leave would drop to and allows it.
fn allow_leaves(
    navigation: &Entity<SettingsNavigation>,
    seen: std::rc::Rc<std::cell::RefCell<Vec<usize>>>,
    cx: &mut VisualTestContext,
) {
    cx.update(|_, cx| {
        navigation.update(cx, |nav, _| {
            nav.set_leave_guard(move |target, _, _| {
                seen.borrow_mut().push(target);
                true
            })
        })
    });
}

/// A point inside the first row of the sub-page fixture's sidebar.
const FIRST_SIDEBAR_ROW: f32 = 60.;

/// Selects General again and opens its sub-page, from wherever the fixture is.
fn reopen_sub_page(navigation: &Entity<SettingsNavigation>, cx: &mut VisualTestContext) {
    click_nav(FIRST_SIDEBAR_ROW, cx);
    click("setting-sub-page-link-Provider", cx);
    assert_eq!(depth(navigation, cx), 1);
    assert!(cx.debug_bounds("inside-sub-page").is_some());
}

#[gpui::test]
fn an_allowed_search_keeps_the_caret_and_the_undo_history(cx: &mut TestAppContext) {
    let (navigation, search, cx) = sub_page_setup(cx);
    click("setting-sub-page-link-Provider", cx);
    let seen = Default::default();
    allow_leaves(&navigation, std::rc::Rc::clone(&seen), cx);

    // Nothing matches, so the page stays open and nothing is asked.
    type_search(&search, "thexme", cx);
    assert_eq!(depth(&navigation, cx), 1);
    cx.update(|window, cx| search.update(cx, |input, cx| input.focus(window, cx)));
    draw(cx);

    // Deleting the stray letter makes the query match only Editor.
    cx.simulate_keystrokes("left left backspace");
    draw(cx);
    draw(cx);
    assert_eq!(*seen.borrow(), vec![0]);
    assert_eq!(depth(&navigation, cx), 0);
    assert!(cx.debug_bounds("other-page").is_some());
    assert_eq!(search_value(&search, cx), "theme");
    assert_eq!(
        cx.update(|_, cx| search.read(cx).selected_range()),
        3..3,
        "the caret stays where the deletion left it"
    );

    cx.simulate_keystrokes(if cfg!(target_os = "macos") {
        "cmd-z"
    } else {
        "ctrl-z"
    });
    draw(cx);
    assert_eq!(
        search_value(&search, cx),
        "thexme",
        "undo restores the text from before the allowed search"
    );
}

#[gpui::test]
fn an_allowing_guard_and_no_guard_follow_a_breadcrumb_at_once(cx: &mut TestAppContext) {
    let (navigation, _, cx) = sub_page_setup(cx);
    let push_nested = |cx: &mut VisualTestContext| {
        let nested = SettingSubPage::new("Model")
            .group(SettingGroup::new().item(marked("inside-nested-page", "model")));
        cx.update(|window, cx| {
            navigation.update(cx, |nav, cx| {
                nav.pop_to(0, window, cx);
                nav.push(provider_page(), window, cx);
                nav.push(nested, window, cx);
            })
        });
        draw(cx);
        assert_eq!(depth(&navigation, cx), 2);
        assert!(cx.debug_bounds("inside-nested-page").is_some());
    };
    let seen: std::rc::Rc<std::cell::RefCell<Vec<usize>>> = Default::default();
    allow_leaves(&navigation, std::rc::Rc::clone(&seen), cx);

    for guarded in [true, false] {
        push_nested(cx);
        // General, Provider, Model: the Provider crumb drops to depth 1.
        click("setting-sub-page-crumb-1", cx);
        assert_eq!(depth(&navigation, cx), 1);
        assert!(cx.debug_bounds("inside-sub-page").is_some());
        assert!(cx.debug_bounds("inside-nested-page").is_none());
        assert!(!has_pending_leave(&navigation, cx));
        assert_eq!(*seen.borrow(), vec![1], "guarded: {guarded}");

        cx.update(|_, cx| navigation.update(cx, |nav, _| nav.clear_leave_guard()));
    }
}

#[gpui::test]
fn an_allowing_guard_and_no_guard_follow_a_sidebar_click_at_once(cx: &mut TestAppContext) {
    let (navigation, _, cx) = sub_page_setup(cx);
    click("setting-sub-page-link-Provider", cx);
    let seen: std::rc::Rc<std::cell::RefCell<Vec<usize>>> = Default::default();
    allow_leaves(&navigation, std::rc::Rc::clone(&seen), cx);

    for guarded in [true, false] {
        click_nav(SECOND_SIDEBAR_ROW, cx);
        assert_eq!(depth(&navigation, cx), 0);
        assert!(cx.debug_bounds("other-page").is_some());
        assert!(cx.debug_bounds("inside-sub-page").is_none());
        assert!(!has_pending_leave(&navigation, cx));
        assert_eq!(*seen.borrow(), vec![0], "guarded: {guarded}");

        cx.update(|_, cx| navigation.update(cx, |nav, _| nav.clear_leave_guard()));
        reopen_sub_page(&navigation, cx);
    }
}

#[gpui::test]
fn an_allowing_guard_and_no_guard_follow_a_page_changing_search_at_once(cx: &mut TestAppContext) {
    let (navigation, search, cx) = sub_page_setup(cx);
    click("setting-sub-page-link-Provider", cx);
    let seen: std::rc::Rc<std::cell::RefCell<Vec<usize>>> = Default::default();
    allow_leaves(&navigation, std::rc::Rc::clone(&seen), cx);

    for guarded in [true, false] {
        // Only Editor matches, so the selection moves there.
        type_search(&search, "theme", cx);
        draw(cx);
        assert_eq!(search_value(&search, cx), "theme");
        assert_eq!(depth(&navigation, cx), 0);
        assert!(cx.debug_bounds("other-page").is_some());
        assert!(cx.debug_bounds("inside-sub-page").is_none());
        assert!(!has_pending_leave(&navigation, cx));
        assert_eq!(*seen.borrow(), vec![0], "guarded: {guarded}");

        cx.update(|_, cx| navigation.update(cx, |nav, _| nav.clear_leave_guard()));
        type_search(&search, "", cx);
        reopen_sub_page(&navigation, cx);
    }
}

#[gpui::test]
fn a_second_held_leave_replaces_the_first(cx: &mut TestAppContext) {
    let (navigation, _, cx) = sub_page_setup(cx);
    click("setting-sub-page-link-Provider", cx);
    let seen = Default::default();
    block_leaves(&navigation, std::rc::Rc::clone(&seen), cx);

    click("setting-sub-page-back", cx);
    assert!(has_pending_leave(&navigation, cx));
    click_nav(SECOND_SIDEBAR_ROW, cx);
    assert_eq!(*seen.borrow(), vec![0, 0]);
    assert_eq!(depth(&navigation, cx), 1);
    assert!(cx.debug_bounds("inside-sub-page").is_some());

    // Back alone would show General; the held sidebar click shows Editor.
    cx.update(|window, cx| navigation.update(cx, |nav, cx| nav.resume_leave(window, cx)));
    draw(cx);
    assert_eq!(depth(&navigation, cx), 0);
    assert!(cx.debug_bounds("other-page").is_some());
    assert!(cx.debug_bounds("on-parent-page").is_none());
    assert!(!has_pending_leave(&navigation, cx));

    // Only one leave was held: resuming again does nothing.
    cx.update(|window, cx| navigation.update(cx, |nav, cx| nav.resume_leave(window, cx)));
    draw(cx);
    assert!(cx.debug_bounds("other-page").is_some());
}
