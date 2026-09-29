mod common;

use std::{cell::Cell, rc::Rc, time::Duration};

use gpui_kit::component::{
    ActiveTheme,
    breadcrumb::{Breadcrumb, BreadcrumbItem},
    button::{Button, ButtonVariants},
    h_flex,
    truncate::TruncateMiddleExt,
};
use gpui_kit::{
    AppContext, Background, Bounds, Context, Entity, Modifiers, Pixels, Point, TestAppContext,
    VisualTestContext, Window, div, point, prelude::*, px, rgb, size,
};

const ITEM_FILL: u32 = 0x13579b;
const CONTROL_FILL: u32 = 0x2468ac;
const LONG_LABEL: &str = "Very long folder containing the complete original recording name.wav";

struct BreadcrumbFixture {
    width: Pixels,
    label: &'static str,
    middle: bool,
    disabled: bool,
    clicks: Rc<Cell<usize>>,
}

impl Render for BreadcrumbFixture {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let clicks = self.clicks.clone();
        let item = BreadcrumbItem::new(self.label)
            .tooltip(self.label)
            .flex_1()
            .min_w_0()
            .bg(rgb(ITEM_FILL))
            .when(self.middle, |item| item.truncate_middle())
            .when(!self.middle, |item| item.truncate())
            .disabled(self.disabled)
            .on_click(move |_, _, _| clicks.set(clicks.get() + 1));

        div()
            .size_full()
            .child(
                h_flex()
                    .absolute()
                    .left(px(400.))
                    .top(px(240.))
                    .w(self.width)
                    .child(
                        Breadcrumb::new()
                            .flex_1()
                            .min_w_0()
                            .text_xs()
                            .child(BreadcrumbItem::new("New Database").flex_shrink_0())
                            .child(item),
                    ),
            )
            .child(
                div().absolute().left(px(1100.)).top(px(240.)).child(
                    Button::new("reference-control")
                        .ghost()
                        .label("Reference")
                        .tooltip("Reference tooltip")
                        .bg(rgb(CONTROL_FILL)),
                ),
            )
    }
}

fn fixture(cx: &mut TestAppContext) -> (Entity<BreadcrumbFixture>, VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_reduce_motion(true);
    });
    let (handle, content) = common::open_window(cx, Some(size(px(1600.), px(600.))), |_, cx| {
        cx.new(|_| BreadcrumbFixture {
            width: px(480.),
            label: "CTE01",
            middle: true,
            disabled: false,
            clicks: Rc::new(Cell::new(0)),
        })
    });
    let mut visual = VisualTestContext::from_window(handle.into(), cx);
    visual.run_until_parked();
    visual.update(|window, cx| window.draw(cx).clear(cx));
    (content, visual)
}

fn filled_bounds(cx: &mut VisualTestContext, color: u32) -> Bounds<Pixels> {
    cx.update(|window, _| {
        let fill: Background = rgb(color).into();
        window
            .painted_quads()
            .iter()
            .find(|quad| quad.background == fill)
            .expect("fixture target is painted")
            .bounds
            .map(|value| px(value.0 / window.scale_factor()))
    })
}

fn hover(cx: &mut VisualTestContext, position: Point<Pixels>) {
    cx.simulate_mouse_move(position, None, Modifiers::default());
    cx.executor().advance_clock(Duration::from_millis(550));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

fn tooltip(cx: &mut VisualTestContext) -> Option<Bounds<Pixels>> {
    cx.update(|window, cx| {
        let fill: Background = cx.theme().tokens.popover.into();
        let quads: Vec<_> = window
            .painted_quads()
            .into_iter()
            .filter(|quad| quad.background == fill && quad.bounds.size.height.0 < 200.)
            .collect();
        assert!(quads.len() <= 1, "at most one managed tooltip is visible");
        quads
            .first()
            .map(|quad| quad.bounds.map(|value| px(value.0 / window.scale_factor())))
    })
}

// Hover underline measures the rendered label independently of the tooltip
// target's internal structure, including the text after truncation.
fn visible_label(cx: &mut VisualTestContext) -> Bounds<Pixels> {
    cx.update(|window, _| {
        window
            .painted_underlines()
            .iter()
            .map(|line| line.bounds.map(|value| px(value.0 / window.scale_factor())))
            .reduce(|left, right| left.union(&right))
            .expect("hovered breadcrumb label is underlined")
    })
}

#[gpui_kit::test]
fn breadcrumb_tooltip_centers_on_label_as_lane_width_changes(cx: &mut TestAppContext) {
    let (view, mut cx) = fixture(cx);
    for width in [280., 480., 180.] {
        hover(&mut cx, point(px(20.), px(20.)));
        view.update(&mut cx, |view, cx| {
            view.width = px(width);
            cx.notify();
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let item = filled_bounds(&mut cx, ITEM_FILL);
        hover(&mut cx, item.origin + point(px(8.), px(8.)));
        let label = visible_label(&mut cx);
        let popup = tooltip(&mut cx).expect("hover shows breadcrumb tooltip");
        assert!(
            (popup.center().x - label.center().x).abs() <= px(0.5),
            "tooltip must center on visible text: width={width}, label={label:?}, popup={popup:?}",
        );
    }
}

#[gpui_kit::test]
fn breadcrumb_tooltip_stays_adjacent_to_its_label_and_reference_control(cx: &mut TestAppContext) {
    let (_, mut cx) = fixture(cx);
    let item = filled_bounds(&mut cx, ITEM_FILL);
    let control = filled_bounds(&mut cx, CONTROL_FILL);
    hover(&mut cx, control.center());
    let control_gap = control.top() - tooltip(&mut cx).unwrap().bottom();
    hover(&mut cx, point(px(20.), px(20.)));
    hover(&mut cx, item.origin + point(px(8.), px(8.)));
    let item_gap = item.top() - tooltip(&mut cx).unwrap().bottom();
    assert!(
        item_gap.abs() <= px(0.5),
        "tooltip must stay adjacent to the label box, allowing pixel rounding: gap={item_gap:?}",
    );
    assert!(
        (item_gap - control_gap).abs() <= px(0.5),
        "tooltip gap must match the reference control: item={item:?}, control={control:?}, item_gap={item_gap:?}, control_gap={control_gap:?}",
    );
}

#[gpui_kit::test]
fn breadcrumb_tooltip_dismisses_over_unused_lane_space(cx: &mut TestAppContext) {
    let (_, mut cx) = fixture(cx);
    let item = filled_bounds(&mut cx, ITEM_FILL);
    hover(&mut cx, item.origin + point(px(8.), px(8.)));
    assert!(tooltip(&mut cx).is_some());
    hover(&mut cx, point(item.right() - px(2.), item.center().y));
    assert!(
        tooltip(&mut cx).is_none(),
        "empty lane space is not a tooltip target"
    );
}

#[gpui_kit::test]
fn breadcrumb_tooltip_keeps_end_and_middle_truncated_labels_inside_the_lane(
    cx: &mut TestAppContext,
) {
    let (view, mut cx) = fixture(cx);
    for middle in [false, true] {
        hover(&mut cx, point(px(20.), px(20.)));
        view.update(&mut cx, |view, cx| {
            view.width = px(260.);
            view.label = LONG_LABEL;
            view.middle = middle;
            cx.notify();
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let item = filled_bounds(&mut cx, ITEM_FILL);
        hover(&mut cx, item.origin + point(px(8.), px(8.)));
        let label = visible_label(&mut cx);
        let popup = tooltip(&mut cx).expect("truncated label retains its full tooltip");
        assert!(label.size.width > px(30.), "label must remain visible");
        assert!(
            label.left() >= item.left() && label.right() <= item.right() + px(0.5),
            "text must truncate within allocation: {label:?}, {item:?}"
        );
        assert!(
            (popup.center().x - item.center().x).abs() <= px(0.5),
            "tooltip must center on the constrained label target: {item:?}, {popup:?}"
        );
    }
}

#[gpui_kit::test]
fn breadcrumb_tooltip_target_preserves_click_and_disabled_behavior(cx: &mut TestAppContext) {
    let (view, mut cx) = fixture(cx);
    let item = filled_bounds(&mut cx, ITEM_FILL);
    let position = item.origin + point(px(8.), px(8.));
    cx.simulate_click(position, Modifiers::default());
    view.read_with(&cx, |view, _| assert_eq!(view.clicks.get(), 1));
    view.update(&mut cx, |view, cx| {
        view.disabled = true;
        cx.notify();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.simulate_click(position, Modifiers::default());
    view.read_with(&cx, |view, _| assert_eq!(view.clicks.get(), 1));
}
