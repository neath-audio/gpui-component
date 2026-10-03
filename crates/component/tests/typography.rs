use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    App, AppContext, Context, IntoElement, ParentElement, Pixels, Render, RenderOnce, Styled,
    TestAppContext, Window, div, px, rems,
};
use gpui_neath::{
    Root, Sizable, StyledTypography as _, TextSize, Theme, ThemeMode, UiTypography, button::Button,
    input::InputGroupButton,
};

#[derive(IntoElement)]
struct FontProbe(Rc<Cell<Pixels>>);

impl RenderOnce for FontProbe {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        self.0
            .set(window.text_style().font_size.to_pixels(window.rem_size()));
        div().child("Long label 世界")
    }
}

struct ButtonHost(Rc<Cell<Pixels>>);

impl Render for ButtonHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Button::new("typed-label")
            .small()
            .text_size(rems(1.125))
            .child(FontProbe(self.0.clone()))
    }
}

#[gpui::test]
fn explicit_button_text_size_reaches_content_at_each_rem_base(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    let painted_size = Rc::new(Cell::new(px(0.)));
    let (_, cx) = cx.add_window_view(|_, _| ButtonHost(painted_size.clone()));
    for base in [16., 20., 18., 16.] {
        cx.update(|window, cx| {
            window.set_rem_size(px(base));
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(painted_size.get(), px(base * 1.125));
    }
}

struct AddonHost(Rc<Cell<Pixels>>);

impl Render for AddonHost {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        InputGroupButton::new("addon-label")
            .text_ui_sm(cx)
            .child(FontProbe(self.0.clone()))
    }
}

#[gpui::test]
fn explicit_role_overrides_input_group_button_defaults(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    let painted_size = Rc::new(Cell::new(px(0.)));
    let (_, cx) = cx.add_window_view(|_, _| AddonHost(painted_size.clone()));
    cx.update(|window, cx| {
        window.set_rem_size(px(20.));
        window.draw(cx).clear(cx);
        assert_eq!(painted_size.get(), TextSize::Small.to_pixels(window, cx));
    });
}

#[derive(IntoElement)]
struct RoleProbe {
    role: TextSize,
    seen: Rc<RefCell<Vec<(Pixels, Pixels, Pixels)>>>,
}

impl RenderOnce for RoleProbe {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.seen.borrow_mut().push((
            window.text_style().font_size.to_pixels(window.rem_size()),
            self.role.to_pixels(window, cx),
            window.line_height(),
        ));
        div().child("Text 文本")
    }
}

struct RolesHost(Rc<RefCell<Vec<(Pixels, Pixels, Pixels)>>>);

impl Render for RolesHost {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().line_height(rems(1.5)).children(
            [
                TextSize::XSmall,
                TextSize::Small,
                TextSize::Default,
                TextSize::Large,
            ]
            .into_iter()
            .map(|role| {
                div().text_ui_size(role, cx).child(RoleProbe {
                    role,
                    seen: self.0.clone(),
                })
            }),
        )
    }
}

#[gpui::test]
fn root_theme_round_trips_and_live_measurement_share_the_rem_base(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    let seen = Rc::new(RefCell::new(Vec::new()));
    let (_, cx) = cx.add_window_view(|window, cx| {
        let host = cx.new(|_| RolesHost(seen.clone()));
        Root::new(host, window, cx)
    });
    for (mode, base) in [
        (ThemeMode::Dark, 16.),
        (ThemeMode::Light, 20.),
        (ThemeMode::Dark, 18.),
        (ThemeMode::Light, 16.),
    ] {
        let sizes = cx.update(|window, cx| {
            Theme::change(mode, None, cx);
            Theme::update(cx, |theme| {
                theme.font_size = px(base);
                theme.set_ui_typography(
                    UiTypography::default().with_size(TextSize::Default, rems(14. / 16.)),
                );
            });
            let tokens = Theme::global(cx).semantic_tokens();
            // Repeated projection and inversion must not apply the scale twice.
            for _ in 0..3 {
                Theme::update(cx, |theme| theme.apply_semantic_tokens(&tokens));
                assert_eq!(Theme::global(cx).font_size, px(base));
            }
            seen.borrow_mut().clear();
            window.draw(cx).clear(cx);
            assert_eq!(window.rem_size(), px(base));
            [
                tokens.typography.xs.size,
                tokens.typography.sm.size,
                tokens.typography.md.size,
                tokens.typography.lg.size,
            ]
        });
        let observed = seen.borrow();
        // GPUI may lay out the root more than once in a draw.
        assert!(!observed.is_empty());
        assert_eq!(observed.len() % sizes.len(), 0);
        for pass in observed.chunks_exact(sizes.len()) {
            for ((painted, measured, line_height), token_size) in pass.iter().zip(sizes) {
                assert_eq!(*painted, token_size);
                assert_eq!(*measured, token_size);
                assert_eq!(
                    *line_height,
                    px(base * 1.5),
                    "roles preserve the inherited line box"
                );
            }
        }
    }
}

#[gpui::test]
fn live_ui_typography_changes_paint_and_measurement_without_changing_rem(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    let seen = Rc::new(RefCell::new(Vec::new()));
    let (_, cx) = cx.add_window_view(|window, cx| {
        let host = cx.new(|_| RolesHost(seen.clone()));
        Root::new(host, window, cx)
    });
    for (mode, values) in [
        (ThemeMode::Dark, [11., 12., 13., 15.]),
        (ThemeMode::Light, [10., 12., 12., 15.]),
        (ThemeMode::Dark, [11., 13., 14., 16.]),
        (ThemeMode::Light, [11., 12., 13., 15.]),
    ] {
        let profile = [
            TextSize::XSmall,
            TextSize::Small,
            TextSize::Default,
            TextSize::Large,
        ]
        .into_iter()
        .zip(values)
        .fold(UiTypography::default(), |profile, (role, size)| {
            profile.with_size(role, rems(size / 16.))
        });
        cx.update(|window, cx| {
            Theme::update(cx, |theme| theme.set_ui_typography(profile));
            Theme::change(mode, None, cx);
            let tokens = Theme::global(cx).semantic_tokens();
            for _ in 0..3 {
                Theme::update(cx, |theme| theme.apply_semantic_tokens(&tokens));
            }
            assert_eq!(Theme::global(cx).ui_typography(), profile);
            seen.borrow_mut().clear();
            window.draw(cx).clear(cx);
            assert_eq!(window.rem_size(), px(16.));
        });
        let observed = seen.borrow();
        assert!(!observed.is_empty());
        for pass in observed.chunks_exact(4) {
            for ((painted, measured, line_height), expected) in pass.iter().zip(values) {
                assert_eq!(*painted, px(expected));
                assert_eq!(*measured, px(expected));
                assert_eq!(*line_height, px(24.));
            }
        }
    }
}

struct MenuListHost {
    menus: Vec<gpui::Entity<gpui_neath::menu::PopupMenu>>,
    list_font: Rc<Cell<Pixels>>,
}

impl Render for MenuListHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().children(self.menus.iter().cloned()).child(
            gpui_neath::list::ListItem::new("primary-list")
                .child(FontProbe(self.list_font.clone())),
        )
    }
}

#[gpui::test]
fn primary_menus_and_lists_resolve_live_control_tier(cx: &mut TestAppContext) {
    use gpui_neath::menu::{PopupMenu, PopupMenuItem};

    cx.update(gpui_neath::init);
    let seen: Vec<_> = (0..4).map(|_| Rc::new(Cell::new(px(0.)))).collect();
    let (_, cx) = cx.add_window_view(|window, cx| {
        let menus = seen[..3]
            .iter()
            .enumerate()
            .map(|(ix, seen)| {
                let seen = seen.clone();
                PopupMenu::build(window, cx, |menu, _, _| {
                    let menu = match ix {
                        1 => menu.small(),
                        2 => menu.xsmall(),
                        _ => menu,
                    };
                    menu.item(PopupMenuItem::element(move |_, _| FontProbe(seen.clone())))
                })
            })
            .collect();
        let host = cx.new(|_| MenuListHost {
            menus,
            list_font: seen[3].clone(),
        });
        Root::new(host, window, cx)
    });

    // Retain the same menu entities across profile changes. Compact lowers
    // ordinary text, while primary menu/list text keeps the Medium control tier.
    for base in [13_f32, 12., 14., 12., 13.] {
        cx.update(|window, cx| {
            Theme::update(cx, |theme| {
                theme.set_ui_typography(
                    UiTypography::default().with_size(TextSize::Default, rems(base / 16.)),
                );
            });
            window.draw(cx).clear(cx);
            assert_eq!(window.rem_size(), px(16.));
        });
        let primary = base.max(13.);
        assert_eq!(
            seen.iter().map(|font| font.get()).collect::<Vec<_>>(),
            [primary, base, 12., primary].map(px),
        );
    }
}
