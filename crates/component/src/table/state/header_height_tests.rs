use super::*;
use crate::Sizable as _;
use gpui::{App, Entity, TestAppContext};

struct HeaderDelegate {
    grouped: bool,
}

impl TableDelegate for HeaderDelegate {
    fn columns_count(&self, _: &App) -> usize {
        2
    }

    fn rows_count(&self, _: &App) -> usize {
        100
    }

    fn column(&self, col_ix: usize, _: &App) -> Column {
        let column = Column::new(format!("col-{col_ix}"), "Column").width(px(150.));
        if col_ix == 0 {
            column.fixed_left()
        } else {
            column
        }
    }

    fn group_headers(&self, _: &App) -> Option<Vec<Vec<ColumnGroup>>> {
        self.grouped.then(|| {
            vec![vec![
                ColumnGroup::new("Fixed", 1),
                ColumnGroup::new("Scrolling", 1),
            ]]
        })
    }

    fn render_header(
        &mut self,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> Stateful<Div> {
        div().id("header").debug_selector(|| "height-header".into())
    }

    fn render_tr(
        &mut self,
        row_ix: usize,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> Stateful<Div> {
        div()
            .id(("row", row_ix))
            .debug_selector(move || format!("height-row-{row_ix}"))
    }

    fn render_th(
        &mut self,
        col_ix: usize,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        div()
            .size_full()
            .debug_selector(move || format!("height-column-{col_ix}"))
    }

    fn render_td(
        &mut self,
        _: usize,
        _: usize,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        div()
    }
}

struct HeaderRoot {
    table: Entity<TableState<HeaderDelegate>>,
    header_height: Option<Pixels>,
}

impl Render for HeaderRoot {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let table = DataTable::new(&self.table)
            .with_size(px(48.))
            .bordered(false)
            .scrollbar_visible(true, false);
        div()
            .w(px(300.))
            .h(px(420.))
            .child(match self.header_height {
                Some(height) => table.header_height(height),
                None => table,
            })
    }
}

fn check_independent_header_height(cx: &mut TestAppContext, grouped: bool) {
    cx.update(crate::init);
    let track: gpui::Hsla = gpui::rgba(0x123456ff).into();
    cx.update(|cx| {
        let theme = gpui_base::Theme::global_mut(cx);
        theme.scrollbar = theme
            .scrollbar
            .clone()
            .with_mode(gpui_base::ScrollbarMode::Always)
            .with_styles(gpui_base::ScrollbarStyles::default().track(|style| style.bg(track)));
    });
    let (root, cx) = cx.add_window_view(|window, cx| HeaderRoot {
        table: cx.new(|cx| TableState::new(HeaderDelegate { grouped }, window, cx)),
        header_height: None,
    });
    let table = root.read_with(cx, |root, _| root.table.clone());

    // Re-render the same retained table: overrides must neither resize body
    // rows nor leak into the next frame when the caller removes the override.
    for (height, expected_total) in [
        (None, if grouped { 96. } else { 48. }),
        (Some(px(30.)), if grouped { 60. } else { 30. }),
        (Some(px(28.)), if grouped { 56. } else { 28. }),
        (None, if grouped { 96. } else { 48. }),
    ] {
        root.update(cx, |root, cx| {
            root.header_height = height;
            cx.notify();
        });
        cx.run_until_parked();
        let (quads, scale_factor) = cx.update(|window, cx| {
            let _ = window.draw(cx);
            (window.painted_quads(), window.scale_factor())
        });

        let header = cx.debug_bounds("height-header").expect("header rendered");
        let first = cx.debug_bounds("height-row-0").expect("first row rendered");
        let second = cx
            .debug_bounds("height-row-1")
            .expect("second row rendered");
        let fixed = cx
            .debug_bounds("height-column-0")
            .expect("fixed header rendered");
        let scrolling = cx
            .debug_bounds("height-column-1")
            .expect("scrolling header rendered");
        assert_eq!(header.size.height, px(expected_total));
        let scrollbar = quads
            .iter()
            .find(|quad| quad.background == track.into())
            .expect("scrollbar track painted");
        assert_eq!(scrollbar.bounds.top(), header.bottom().scale(scale_factor));
        assert_eq!(first.top(), header.bottom());
        assert_eq!(first.size.height, px(48.));
        assert_eq!(second.top() - first.top(), px(48.));
        assert_eq!(fixed.top(), scrolling.top());
        assert_eq!(fixed.size.height, scrolling.size.height);
        table.read_with(cx, |table, _| {
            assert_eq!(table.fixed_head_cols_bounds.size.height, header.size.height);
            assert_eq!(
                table
                    .vertical_scroll_handle
                    .0
                    .borrow()
                    .base_handle
                    .bounds()
                    .top(),
                header.bottom()
            );
        });
    }
}

#[gpui::test]
fn header_height_is_independent_and_defaults_to_body_rows(cx: &mut TestAppContext) {
    check_independent_header_height(cx, false);
}

#[gpui::test]
fn header_height_applies_to_fixed_and_scrolling_group_headers(cx: &mut TestAppContext) {
    check_independent_header_height(cx, true);
}

#[gpui::test]
fn page_navigation_uses_the_body_viewport_with_independent_headers(cx: &mut TestAppContext) {
    cx.update(crate::init);
    for (grouped, header_height, expected_row) in [
        (false, None, 7),
        (false, Some(px(30.)), 8),
        (true, None, 6),
        (true, Some(px(30.)), 7),
    ] {
        let (root, cx) = cx.add_window_view(|window, cx| HeaderRoot {
            table: cx.new(|cx| TableState::new(HeaderDelegate { grouped }, window, cx)),
            header_height,
        });
        let table = root.read_with(cx, |root, _| root.table.clone());
        cx.run_until_parked();
        cx.update(|window, cx| {
            table.update(cx, |table, cx| {
                table.set_selected_row(0, cx);
                table.focus_handle.focus(window, cx);
            });
        });
        cx.simulate_keystrokes("pagedown");
        assert_eq!(
            table.read_with(cx, |table, _| table.selected_row()),
            Some(expected_row)
        );
        cx.simulate_keystrokes("pageup");
        assert_eq!(
            table.read_with(cx, |table, _| table.selected_row()),
            Some(0)
        );
    }
}
