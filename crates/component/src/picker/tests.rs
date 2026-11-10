use super::{Picker, PickerEvent, PickerFooterItem, PickerItem, PickerState};
use crate::{
    IndexPath, Selectable,
    button::Button,
    list::{ListDelegate, ListState},
};
use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable as _, InteractiveElement as _,
    IntoElement, Modifiers, ParentElement as _, Render, RenderOnce,
    StatefulInteractiveElement as _, Styled as _, Subscription, Task, TestAppContext,
    VisualTestContext, Window, div, prelude::FluentBuilder as _, px, size,
};
use std::{cell::RefCell, rc::Rc, time::Duration};

const ITEMS: [&str; 3] = ["Alpha", "Beta", "Gamma"];

/// The dropdown motion is 150ms; four times that settles it on the test clock.
const SETTLE: Duration = Duration::from_millis(150 * 4);

/// A `PickerItem` row with a debug selector, so tests can find it.
#[derive(IntoElement)]
struct Row {
    ix: usize,
    item: PickerItem,
}

impl Selectable for Row {
    fn selected(mut self, selected: bool) -> Self {
        self.item = self.item.selected(selected);
        self
    }

    fn is_selected(&self) -> bool {
        self.item.is_selected()
    }
}

impl RenderOnce for Row {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let ix = self.ix;
        div()
            .debug_selector(move || format!("row-{ix}"))
            .child(self.item)
    }
}

struct Rows {
    items: Vec<&'static str>,
    selected: Option<IndexPath>,
    log: Rc<RefCell<Vec<String>>>,
}

impl ListDelegate for Rows {
    type Item = Row;

    fn perform_search(
        &mut self,
        query: &str,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> Task<()> {
        self.log.borrow_mut().push(format!("search {query:?}"));
        Task::ready(())
    }

    fn items_count(&self, _: usize, _: &App) -> usize {
        self.items.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> Option<Row> {
        let row = ix.row;
        let label = *self.items.get(row)?;
        let log = self.log.clone();
        Some(Row {
            ix: row,
            item: PickerItem::new(("row", row), label).hover_action(
                div()
                    .id(("remove", row))
                    .debug_selector(move || format!("remove-{row}"))
                    .size(px(16.))
                    .on_click(move |_, _, _| log.borrow_mut().push(format!("remove {label}"))),
            ),
        })
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) {
        self.selected = ix;
    }

    fn confirm(&mut self, _: bool, _: &mut Window, _: &mut Context<ListState<Self>>) {
        if let Some(label) = self.selected.and_then(|ix| self.items.get(ix.row)) {
            self.log.borrow_mut().push(format!("confirm {label}"));
        }
    }
}

struct Host {
    state: Entity<PickerState<Rows>>,
    log: Rc<RefCell<Vec<String>>>,
    opens: usize,
    header: bool,
    field: FocusHandle,
    _events: Subscription,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let log = self.log.clone();
        let state = self.state.clone();
        div()
            .size_full()
            .child(
                div().absolute().left(px(40.)).top(px(40.)).child(
                    Picker::new("p", &self.state)
                        .trigger(
                            Button::new("t")
                                .label("Open")
                                .debug_selector(|| "trigger".into()),
                        )
                        .max_w(px(200.))
                        .when(self.header, |this| {
                            this.header(
                                div()
                                    .debug_selector(|| "header".into())
                                    .h(px(24.))
                                    .child("Header"),
                            )
                        })
                        .footer([PickerFooterItem::new("clear", "Clear").on_click(
                            move |_, cx| {
                                // Records whether the picker had already closed.
                                let open = state.read(cx).is_open();
                                log.borrow_mut().push(format!("clear (open: {open})"));
                            },
                        )]),
                ),
            )
            .child(
                div()
                    .id("field")
                    .track_focus(&self.field)
                    .absolute()
                    .left(px(400.))
                    .top(px(400.))
                    .size(px(20.)),
            )
    }
}

fn setup<'a>(
    cx: &'a mut TestAppContext,
    items: Vec<&'static str>,
    searchable: bool,
) -> (Entity<Host>, &'a mut VisualTestContext) {
    cx.update(crate::init);
    let (view, visual) = cx.add_window_view(move |window, cx| {
        window.activate_window();
        let log = Rc::new(RefCell::new(Vec::new()));
        let delegate = Rows {
            items,
            selected: None,
            log: log.clone(),
        };
        let state = cx.new(|cx| PickerState::new(delegate, window, cx).searchable(searchable, cx));
        let events = cx.subscribe(&state, |this: &mut Host, _, event: &PickerEvent, cx| {
            match event {
                PickerEvent::Open => this.opens += 1,
            }
            cx.notify();
        });
        Host {
            state,
            log,
            opens: 0,
            header: false,
            field: cx.focus_handle(),
            _events: events,
        }
    });
    visual.simulate_resize(size(px(600.), px(500.)));
    draw(visual);
    (view, visual)
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| crate::test_redraw(window, cx));
}

fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let bounds = cx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("{selector} is drawn"));
    cx.simulate_click(bounds.center(), Modifiers::default());
    draw(cx);
}

fn state(view: &Entity<Host>, cx: &mut VisualTestContext) -> Entity<PickerState<Rows>> {
    cx.update(|_, cx| view.read(cx).state.clone())
}

fn is_open(view: &Entity<Host>, cx: &mut VisualTestContext) -> bool {
    cx.update(|_, cx| view.read(cx).state.read(cx).is_open())
}

fn log(view: &Entity<Host>, cx: &mut VisualTestContext) -> Vec<String> {
    cx.update(|_, cx| view.read(cx).log.borrow().clone())
}

fn opens(view: &Entity<Host>, cx: &mut VisualTestContext) -> usize {
    cx.update(|_, cx| view.read(cx).opens)
}

fn cursor(view: &Entity<Host>, cx: &mut VisualTestContext) -> Option<IndexPath> {
    cx.update(|_, cx| {
        view.read(cx)
            .state
            .read(cx)
            .list()
            .read(cx)
            .selected_index()
    })
}

#[gpui::test]
fn clicking_the_trigger_opens_and_draws_the_picker(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, ITEMS.into(), false);
    assert!(cx.debug_bounds("row-0").is_none());

    click(cx, "trigger");

    assert!(cx.debug_bounds("row-0").is_some());
    assert!(is_open(&view, cx));
    let state = state(&view, cx);
    cx.update(|window, cx| assert!(state.read(cx).focus_handle(cx).is_focused(window)));
}

#[gpui::test]
fn arrow_keys_and_enter_confirm_the_cursor_row(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, ITEMS.into(), false);
    click(cx, "trigger");

    cx.simulate_keystrokes("down down enter");

    assert_eq!(log(&view, cx), ["confirm Beta"]);
}

#[gpui::test]
fn enter_with_no_cursor_row_confirms_nothing_and_stays_open(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, ITEMS.into(), false);
    click(cx, "trigger");

    cx.simulate_keystrokes("enter");
    draw(cx);

    assert!(log(&view, cx).is_empty());
    assert!(is_open(&view, cx));
}

#[gpui::test]
fn escape_closes_the_picker(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, ITEMS.into(), false);
    click(cx, "trigger");

    cx.simulate_keystrokes("escape");
    draw(cx);

    assert!(!is_open(&view, cx));
    assert!(cx.debug_bounds("row-0").is_none());
}

#[gpui::test]
fn clicking_the_trigger_again_closes_and_the_next_open_resets(cx: &mut TestAppContext) {
    let executor = cx.executor();
    let (view, cx) = setup(cx, ITEMS.into(), false);
    click(cx, "trigger");
    executor.advance_clock(SETTLE);
    draw(cx);
    let settled = cx.debug_bounds("row-0").expect("row-0 is drawn").top();
    cx.simulate_keystrokes("down down");
    draw(cx);
    assert_eq!(cursor(&view, cx), Some(IndexPath::new(1)));

    click(cx, "trigger");
    assert!(!is_open(&view, cx));
    click(cx, "trigger");

    assert!(is_open(&view, cx));
    assert_eq!(cursor(&view, cx), None);
    assert_eq!(opens(&view, cx), 2);
    // The reopened surface slides down into place again.
    let reopened = cx.debug_bounds("row-0").expect("row-0 is drawn").top();
    assert!(reopened < settled, "{reopened:?} vs settled {settled:?}");
}

#[gpui::test]
fn every_open_clears_the_query_and_cursor_and_emits_open(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, ITEMS.into(), true);
    let state = state(&view, cx);
    click(cx, "trigger");
    cx.simulate_input("be");
    draw(cx);
    cx.update(|_, cx| {
        let list = state.read(cx).list().read(cx);
        assert_eq!(list.query_input.read(cx).value(), "be");
    });

    cx.update(|_, cx| state.update(cx, |state, cx| state.close(cx)));
    draw(cx);
    cx.update(|window, cx| state.update(cx, |state, cx| state.open(window, cx)));
    draw(cx);

    cx.update(|_, cx| {
        let list = state.read(cx).list().read(cx);
        assert_eq!(list.query_input.read(cx).value(), "");
        assert_eq!(list.selected_index(), None);
    });
    assert_eq!(
        log(&view, cx).last().map(String::as_str),
        Some(r#"search """#)
    );
    assert_eq!(opens(&view, cx), 2);
}

#[gpui::test]
fn opening_from_code_focuses_the_list(cx: &mut TestAppContext) {
    let executor = cx.executor();
    let (view, cx) = setup(cx, ITEMS.into(), false);
    let state = state(&view, cx);

    cx.update(|window, cx| state.update(cx, |state, cx| state.open(window, cx)));
    draw(cx);
    executor.advance_clock(SETTLE);
    draw(cx);

    assert!(is_open(&view, cx));
    cx.update(|window, cx| {
        assert!(state.read(cx).focus_handle(cx).contains_focused(window, cx));
    });
}

#[gpui::test]
fn escape_after_opening_from_code_restores_the_earlier_focus(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, ITEMS.into(), false);
    let state = state(&view, cx);
    let field = cx.update(|_, cx| view.read(cx).field.clone());
    cx.update(|window, cx| field.focus(window, cx));
    draw(cx);

    cx.update(|window, cx| state.update(cx, |state, cx| state.open(window, cx)));
    draw(cx);
    cx.simulate_keystrokes("escape");
    draw(cx);

    assert!(!is_open(&view, cx));
    cx.update(|window, _| assert!(field.is_focused(window)));
}

#[gpui::test]
fn a_hover_action_click_keeps_the_picker_open(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, ITEMS.into(), false);
    click(cx, "trigger");
    let row = cx.debug_bounds("row-1").expect("row-1 is drawn");
    // Hover first so the revealed action is hit-testable.
    cx.simulate_mouse_move(row.center(), None, Modifiers::default());
    draw(cx);

    click(cx, "remove-1");

    assert_eq!(log(&view, cx), ["remove Beta"]);
    assert!(is_open(&view, cx));
}

#[gpui::test]
fn a_footer_item_closes_the_picker_then_runs(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, ITEMS.into(), false);
    click(cx, "trigger");

    click(cx, "picker-footer-clear");

    // The action ran once, after the picker had closed.
    assert_eq!(log(&view, cx), ["clear (open: false)"]);
    assert!(!is_open(&view, cx));
}

#[gpui::test]
fn close_emptied_focuses_the_named_target(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, ITEMS.into(), false);
    let state = state(&view, cx);
    let field = cx.update(|_, cx| view.read(cx).field.clone());
    click(cx, "trigger");

    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.close_emptied(Some(&field), window, cx)
        })
    });
    draw(cx);

    assert!(!is_open(&view, cx));
    cx.update(|window, _| assert!(field.is_focused(window)));
}

#[gpui::test]
fn rows_fill_a_picker_that_hugs_its_footer(cx: &mut TestAppContext) {
    let (_, cx) = setup(cx, ITEMS.into(), false);
    click(cx, "trigger");

    let row = cx.debug_bounds("row-0").expect("row-0 is drawn");
    let footer = cx
        .debug_bounds("picker-footer-clear")
        .expect("the footer item is drawn");
    assert_eq!(row.size.width, footer.size.width);
}

#[gpui::test]
fn a_long_label_truncates_instead_of_widening_past_max_w(cx: &mut TestAppContext) {
    let long: &'static str = "A".repeat(400).leak();
    let (_, cx) = setup(cx, vec![long, "Beta", "Gamma"], false);
    click(cx, "trigger");

    let row = cx.debug_bounds("row-0").expect("row-0 is drawn");
    assert!(row.size.width <= px(200.), "{:?}", row.size.width);
    // The label gives way: the row's trailing action stays inside the row
    // instead of being pushed past its edge.
    cx.simulate_mouse_move(row.center(), None, Modifiers::default());
    draw(cx);
    let action = cx.debug_bounds("remove-0").expect("remove-0 is drawn");
    assert!(action.right() <= row.right(), "{action:?} in {row:?}");
}

#[gpui::test]
fn the_frame_is_flush_on_top_only_with_a_header_or_query(cx: &mut TestAppContext) {
    let executor = cx.executor();
    let (view, cx) = setup(cx, ITEMS.into(), false);
    click(cx, "trigger");
    executor.advance_clock(SETTLE);
    draw(cx);
    let trigger = cx.debug_bounds("trigger").expect("trigger is drawn");
    let row = cx.debug_bounds("row-0").expect("row-0 is drawn");
    let row_gap = row.top() - trigger.bottom();

    show_header(&view, cx);
    executor.advance_clock(SETTLE);
    draw(cx);
    let header = cx.debug_bounds("header").expect("header is drawn");
    let header_gap = header.top() - trigger.bottom();

    assert_eq!(row_gap - header_gap, px(4.));
}

/// The query band owns a searchable picker's top edge, so the frame has no top
/// padding with or without a header: a header pushes the rows down by exactly
/// its own height.
#[gpui::test]
fn a_searchable_frame_is_flush_on_top_under_its_query(cx: &mut TestAppContext) {
    let executor = cx.executor();
    let (view, cx) = setup(cx, ITEMS.into(), true);
    click(cx, "trigger");
    executor.advance_clock(SETTLE);
    draw(cx);
    let without_header = cx.debug_bounds("row-0").expect("row-0 is drawn").top();

    show_header(&view, cx);
    executor.advance_clock(SETTLE);
    draw(cx);
    let with_header = cx.debug_bounds("row-0").expect("row-0 is drawn").top();
    let header = cx.debug_bounds("header").expect("header is drawn");

    assert_eq!(with_header - without_header, header.size.height);
}

fn show_header(view: &Entity<Host>, cx: &mut VisualTestContext) {
    cx.update(|_, cx| {
        view.update(cx, |host, cx| {
            host.header = true;
            cx.notify();
        })
    });
    draw(cx);
}
