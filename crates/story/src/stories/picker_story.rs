use gpui_kit::component::{
    Disableable as _, Icon, IconName, IndexPath, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    list::{ListDelegate, ListState},
    picker::{Picker, PickerFooterItem, PickerItem, PickerSectionHeader, PickerState},
    separator::Separator,
    v_flex,
};
use gpui_kit::{
    Anchor, App, AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString,
    Styled, Subscription, WeakEntity, Window, div, px,
};

use crate::section;

const PICKER_WIDTH: f32 = 240.;

/// One delegate drives every example; the flags decide what a row shows.
struct Rows {
    seed: Vec<(&'static str, Vec<&'static str>)>,
    sections: Vec<(&'static str, Vec<SharedString>)>,
    query: String,
    current: Option<SharedString>,
    cursor: Option<IndexPath>,
    in_list_headers: bool,
    removable: bool,
    picker: Option<WeakEntity<PickerState<Rows>>>,
}

impl Rows {
    fn new(seed: Vec<(&'static str, Vec<&'static str>)>) -> Self {
        let mut this = Self {
            seed,
            sections: Vec::new(),
            query: String::new(),
            current: None,
            cursor: None,
            in_list_headers: false,
            removable: false,
            picker: None,
        };
        this.refill();
        this
    }

    fn refill(&mut self) {
        self.sections = self
            .seed
            .iter()
            .map(|(title, rows)| (*title, rows.iter().map(|row| (*row).into()).collect()))
            .collect();
    }

    fn visible(&self, section: usize) -> Vec<&SharedString> {
        let query = self.query.to_lowercase();
        self.sections
            .get(section)
            .map_or_else(Vec::new, |(_, rows)| {
                rows.iter()
                    .filter(|row| row.to_lowercase().contains(&query))
                    .collect()
            })
    }

    fn is_empty(&self) -> bool {
        self.sections.iter().all(|(_, rows)| rows.is_empty())
    }

    fn remove(&mut self, label: &SharedString) {
        // Clear the cursor: the rows below slide into its index, and a later
        // Enter must not confirm whichever one landed there.
        self.cursor = None;
        for (_, rows) in &mut self.sections {
            rows.retain(|row| row != label);
        }
    }

    fn close_picker(&self, cx: &mut App) {
        if let Some(picker) = &self.picker {
            _ = picker.update(cx, |picker, cx| picker.close(cx));
        }
    }
}

impl ListDelegate for Rows {
    type Item = PickerItem;

    fn perform_search(
        &mut self,
        query: &str,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> gpui_kit::Task<()> {
        self.query = query.to_string();
        cx.notify();
        gpui_kit::Task::ready(())
    }

    fn sections_count(&self, _: &App) -> usize {
        self.sections.len().max(1)
    }

    fn items_count(&self, section: usize, _: &App) -> usize {
        self.visible(section).len()
    }

    fn render_section_header(
        &mut self,
        section: usize,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> Option<impl IntoElement> {
        let (title, _) = self.sections.get(section)?;
        self.in_list_headers
            .then(|| PickerSectionHeader::new(*title).in_list(true))
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<PickerItem> {
        let label = (*self.visible(ix.section).get(ix.row)?).clone();
        let mut item = PickerItem::new(("row", ix.section * 100 + ix.row), label.clone());
        if self.current.as_ref() == Some(&label) {
            item = item
                .active(true)
                .accessory(Icon::new(IconName::Check).small());
        }
        if self.removable {
            let list = cx.entity().downgrade();
            let picker = self.picker.clone();
            item = item.hover_action(
                Button::new(("remove", ix.section * 100 + ix.row))
                    .ghost()
                    .xsmall()
                    .icon(IconName::Close)
                    .on_click(move |_, window, cx| {
                        let emptied = list
                            .update(cx, |list, cx| {
                                list.delegate_mut().remove(&label);
                                cx.notify();
                                list.delegate().is_empty()
                            })
                            .unwrap_or(false);
                        if emptied && let Some(picker) = &picker {
                            _ = picker
                                .update(cx, |picker, cx| picker.close_emptied(None, window, cx));
                        }
                    }),
            );
        }
        Some(item)
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) {
        self.cursor = ix;
    }

    fn confirm(&mut self, _: bool, _: &mut Window, cx: &mut Context<ListState<Self>>) {
        let Some(ix) = self.cursor else { return };
        let Some(label) = self.visible(ix.section).get(ix.row).map(|l| (*l).clone()) else {
            return;
        };
        if !self.removable {
            self.current = Some(label);
        }
        cx.notify();
        self.close_picker(cx);
    }
}

fn picker_state(
    rows: Rows,
    searchable: bool,
    window: &mut Window,
    cx: &mut App,
) -> Entity<PickerState<Rows>> {
    let state = cx.new(|cx| PickerState::new(rows, window, cx).searchable(searchable, cx));
    let weak = state.downgrade();
    state.read(cx).list().clone().update(cx, |list, _| {
        list.delegate_mut().picker = Some(weak);
    });
    state
}

pub struct PickerStory {
    searchable: Entity<PickerState<Rows>>,
    removable: Entity<PickerState<Rows>>,
    header: Entity<PickerState<Rows>>,
    above: Entity<PickerState<Rows>>,
    _subscriptions: Vec<Subscription>,
}

impl PickerStory {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let searchable = {
            let mut rows = Rows::new(vec![
                (
                    "Recent",
                    vec![
                        "Ambience Library",
                        "Foley Session",
                        "Trailer Mix",
                        "Dialogue Pass",
                    ],
                ),
                (
                    "All projects",
                    vec![
                        "Archive 2023",
                        "Creature Vocals",
                        "Impacts and Hits",
                        "Weather",
                    ],
                ),
            ]);
            rows.in_list_headers = true;
            rows.current = Some("Foley Session".into());
            picker_state(rows, true, window, cx)
        };
        let removable = {
            let mut rows = Rows::new(Self::recents());
            rows.removable = true;
            picker_state(rows, false, window, cx)
        };
        let header = picker_state(
            Rows::new(vec![(
                "",
                vec!["Ambience Library", "Foley Session", "Trailer Mix"],
            )]),
            false,
            window,
            cx,
        );
        let above = picker_state(
            Rows::new(vec![("", vec!["Draft", "Review", "Approved", "Published"])]),
            false,
            window,
            cx,
        );

        let mut subscriptions = Vec::new();
        for state in [&searchable, &removable, &header, &above] {
            subscriptions.push(cx.observe(state, |_, _, cx| cx.notify()));
            let list = state.read(cx).list().clone();
            subscriptions.push(cx.observe(&list, |_, _, cx| cx.notify()));
        }
        Self {
            searchable,
            removable,
            header,
            above,
            _subscriptions: subscriptions,
        }
    }

    fn recents() -> Vec<(&'static str, Vec<&'static str>)> {
        vec![(
            "",
            vec![
                "Ambience Library",
                "Foley Session",
                "Trailer Mix",
                "Dialogue Pass",
            ],
        )]
    }

    fn current(state: &Entity<PickerState<Rows>>, cx: &App) -> Option<SharedString> {
        state.read(cx).list().read(cx).delegate().current.clone()
    }

    fn empty(state: &Entity<PickerState<Rows>>, cx: &App) -> bool {
        state.read(cx).list().read(cx).delegate().is_empty()
    }
}

impl super::Story for PickerStory {
    fn title() -> &'static str {
        "Picker"
    }

    fn description() -> &'static str {
        "Find an item in a list the app owns and act on it, in a popover that opens like Select."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Render for PickerStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current =
            Self::current(&self.searchable, cx).unwrap_or_else(|| "Choose a project".into());
        let emptied = Self::empty(&self.removable, cx);
        let removable_list = self.removable.read(cx).list().clone();
        let clear_list = removable_list.clone();
        let header_check = Icon::new(IconName::Check).small();

        v_flex()
            .w_full()
            .gap_3()
            .child(
                section("Searchable")
                    .description(
                        "Type to filter. Confirming a row makes it current and closes the picker.",
                    )
                    .child(
                        Picker::new("picker-searchable", &self.searchable)
                            .trigger(Button::new("picker-searchable-trigger").outline().label(current))
                            .search_placeholder("Search projects")
                            .w(px(PICKER_WIDTH))
                            .footer([PickerFooterItem::new("picker-searchable-new", "New project")
                                .icon(IconName::Plus)]),
                    ),
            )
            .child(
                section("No query")
                    .description(
                        "Hover a row to remove it; the picker stays open. Removing the last row closes it.",
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                Picker::new("picker-removable", &self.removable)
                                    .trigger(
                                        Button::new("picker-removable-trigger")
                                            .outline()
                                            .label("Recent projects")
                                            .disabled(emptied),
                                    )
                                    .w(px(PICKER_WIDTH))
                                    .footer([PickerFooterItem::new("picker-removable-clear", "Clear")
                                        .icon(IconName::Close)
                                        .on_click(move |_, cx| {
                                            clear_list.update(cx, |list, cx| {
                                                list.delegate_mut().sections.iter_mut().for_each(
                                                    |(_, rows)| rows.clear(),
                                                );
                                                cx.notify();
                                            });
                                        })]),
                            )
                            .child(Button::new("picker-removable-refill").ghost().label("Refill").on_click(
                                move |_, _, cx| {
                                    removable_list.update(cx, |list, cx| {
                                        list.delegate_mut().refill();
                                        cx.notify();
                                    });
                                },
                            )),
                    ),
            )
            .child(
                section("Header")
                    .description("A header above the list: an outside section header and a pinned row.")
                    .child(
                        Picker::new("picker-header", &self.header)
                            .trigger(Button::new("picker-header-trigger").outline().label("Switch project"))
                            .w(px(PICKER_WIDTH))
                            .header(
                                v_flex()
                                    .pt_1()
                                    .child(PickerSectionHeader::new("Pinned"))
                                    .child(
                                        div().px_1().pb_1().child(
                                            PickerItem::new("picker-header-pinned", "Untitled Session")
                                                .active(true)
                                                .accessory(header_check),
                                        ),
                                    )
                                    .child(Separator::horizontal().mb_1())
                                    .child(PickerSectionHeader::new("Recent")),
                            ),
                    ),
            )
            .child(
                section("Opens above")
                    .description("Anchored at the bottom left, so the surface opens above its trigger.")
                    .child(
                        Picker::new("picker-above", &self.above)
                            .trigger(Button::new("picker-above-trigger").outline().label("Status"))
                            .anchor(Anchor::BottomLeft)
                            .w(px(PICKER_WIDTH)),
                    ),
            )
    }
}
