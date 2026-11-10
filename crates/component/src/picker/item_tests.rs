use super::{PickerFooterItem, PickerItem};
use crate::{IconName, Root, Size, TextSize, Theme, UiTypography};
use gpui::{
    AppContext as _, Context, InteractiveElement as _, IntoElement, Modifiers, MouseButton,
    ParentElement as _, Render, RenderOnce, Styled as _, TestAppContext, Window, div, px,
};
use std::{cell::Cell, cell::RefCell, rc::Rc};

#[derive(IntoElement)]
struct Probe(Rc<Cell<gpui::Pixels>>);
impl RenderOnce for Probe {
    fn render(self, window: &mut Window, _: &mut gpui::App) -> impl IntoElement {
        self.0
            .set(window.text_style().font_size.to_pixels(window.rem_size()));
        div().child("Text")
    }
}

struct Host(Option<PickerItem>);
impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().children(self.0.take())
    }
}

#[gpui::test]
fn retained_picker_items_resolve_current_text_roles(cx: &mut TestAppContext) {
    cx.update(crate::init);
    let seen = Rc::new(Cell::new(px(0.)));
    let host = cx.new(|_| Host(None));
    let (_, cx) = cx.add_window_view(|window, cx| Root::new(host.clone(), window, cx));
    for (base, expected) in [(12., 13.), (14., 14.), (13., 13.)] {
        // Construct before switching profiles; the row must keep its menu
        // tier, not capture the old profile's numeric font size.
        let row = PickerItem::new("retained", "Long file name 世界").accessory(Probe(seen.clone()));
        cx.update(|window, cx| {
            Theme::update(cx, |theme| {
                theme.set_ui_typography(
                    UiTypography::default().with_size(TextSize::Default, gpui::rems(base / 16.)),
                );
            });
            host.update(cx, |this, cx| {
                this.0 = Some(row);
                cx.notify();
            });
            window.draw(cx).clear(cx);
            assert_eq!(window.rem_size(), px(16.));
        });
        assert_eq!(seen.get(), px(expected));
    }
}

struct Wrapped {
    log: Rc<RefCell<Vec<&'static str>>>,
}
impl Render for Wrapped {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let log = self.log.clone();
        let action_log = self.log.clone();
        div()
            .w(px(300.))
            .on_mouse_down(MouseButton::Left, move |_, _, _| {
                log.borrow_mut().push("row")
            })
            .child(
                PickerItem::new("item", "Label").hover_action(
                    div()
                        .id("act")
                        .debug_selector(|| "action".into())
                        .size(px(16.))
                        .on_mouse_down(MouseButton::Left, move |_, _, _| {
                            action_log.borrow_mut().push("action")
                        }),
                ),
            )
    }
}

#[gpui::test]
fn a_hover_action_click_does_not_reach_the_row(cx: &mut TestAppContext) {
    cx.update(crate::init);
    let log = Rc::new(RefCell::new(Vec::new()));
    let (_, visual) = cx.add_window_view(|_, _| Wrapped { log: log.clone() });
    visual.simulate_resize(gpui::size(px(400.), px(200.)));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let target = visual.debug_bounds("action").expect("action is drawn");
    // Hover first so the revealed action is hit-testable.
    visual.simulate_mouse_move(target.center(), None, Modifiers::default());
    visual.simulate_click(target.center(), Modifiers::default());
    assert_eq!(*log.borrow(), ["action"]);
}

struct Footer {
    log: Rc<RefCell<Vec<&'static str>>>,
    size: Size,
}
impl Render for Footer {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let log = self.log.clone();
        let before = self.log.clone();
        div().w(px(300.)).child(
            PickerFooterItem::new("footer", "Open")
                .icon(IconName::Plus)
                .menu_text_size(self.size)
                .before_click(Rc::new(move |_, _| before.borrow_mut().push("before")))
                .on_click(move |_, _| log.borrow_mut().push("action")),
        )
    }
}

#[gpui::test]
fn a_footer_item_runs_before_click_then_its_action(cx: &mut TestAppContext) {
    cx.update(crate::init);
    let log = Rc::new(RefCell::new(Vec::new()));
    let (_, visual) = cx.add_window_view(|_, _| Footer {
        log: log.clone(),
        size: Size::Medium,
    });
    visual.simulate_resize(gpui::size(px(400.), px(200.)));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let target = visual
        .debug_bounds("picker-footer-icon")
        .expect("icon is drawn");
    visual.simulate_click(target.center(), Modifiers::default());
    assert_eq!(*log.borrow(), ["before", "action"]);
}

#[gpui::test]
fn footer_icons_stay_12px_at_every_menu_tier(cx: &mut TestAppContext) {
    cx.update(crate::init);
    let log = Rc::new(RefCell::new(Vec::new()));
    let (view, visual) = cx.add_window_view(|_, _| Footer {
        log,
        size: Size::Small,
    });
    visual.simulate_resize(gpui::size(px(400.), px(200.)));
    for size in [Size::Small, Size::Medium] {
        visual.update(|window, cx| {
            view.update(cx, |this, cx| {
                this.size = size;
                cx.notify();
            });
            window.draw(cx).clear(cx);
        });
        let bounds = visual
            .debug_bounds("picker-footer-icon")
            .expect("icon is drawn");
        assert_eq!(bounds.size, gpui::size(px(12.), px(12.)), "{size:?}");
    }
}
