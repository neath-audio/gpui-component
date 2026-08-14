use gpui::{
    BoxShadow, FontWeight, Hsla, Styled as _, TestAppContext, div, hsla, linear_color_stop,
    linear_gradient, point, px, relative, rems,
};
use gpui_neath::{
    Sizable as _, Theme, ThemeToken,
    button::Button,
    h_flex,
    style::{
        recipes, tokens,
        typography::{
            TruncateMiddleExt as _, middle_truncating_cell_sized, truncating_cell,
            truncating_cell_sized,
        },
    },
    v_flex,
};

#[test]
fn neutral_token_scale_matches_the_accepted_contract() {
    assert_eq!(tokens::TEXT_10, rems(0.625));
    assert_eq!(tokens::TEXT_12, rems(0.75));
    assert_eq!(tokens::TEXT_13, rems(0.8125));
    assert_eq!(tokens::TEXT_14, rems(0.875));
    assert_eq!(tokens::TEXT_15, rems(0.9375));
    assert_eq!(tokens::TEXT_16, rems(1.0));
    assert_eq!(tokens::SPACE_HALF, px(2.));
    assert_eq!(tokens::SPACE_1, px(4.));
    assert_eq!(tokens::SPACE_1P5, px(6.));
    assert_eq!(tokens::SPACE_2, px(8.));
    assert_eq!(tokens::SPACE_3, px(12.));
    assert_eq!(tokens::SPACE_4, px(16.));
    assert_eq!(tokens::SPACE_6, px(24.));
    assert_eq!(tokens::ACCENT_RIM_PX, px(2.));
    assert_eq!(tokens::RADIUS_SM, px(4.));
    assert_eq!(tokens::RADIUS_MD, px(6.));
    assert_eq!(tokens::RADIUS_LG, px(8.));
    assert_eq!(tokens::RADIUS_PILL, px(100.));
    assert_eq!(tokens::ICON_INLINE, px(12.));
    assert_eq!(tokens::ICON_CHROME, px(14.));
    assert_eq!(tokens::ICON_PRIMARY, px(16.));
}

#[gpui::test]
fn theme_relative_surface_tones_preserve_the_accepted_math(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    cx.update(|cx| {
        let background = hsla(72. / 360., 0.4, 0.6, 0.8);
        let accent = hsla(288. / 360., 0.6, 0.2, 1.0);
        {
            let theme = Theme::global_mut(cx);
            theme.background = background;
            theme.accent = accent;
        }

        let expected_mix = |amount: f32| Hsla {
            h: background.h * (1.0 - amount) + accent.h * amount,
            s: background.s * (1.0 - amount) + accent.s * amount,
            l: background.l * (1.0 - amount) + accent.l * amount,
            a: background.a * (1.0 - amount) + accent.a * amount,
        };
        assert_eq!(
            tokens::bg_sunken(cx),
            Hsla {
                l: (background.l - 0.04).max(0.0),
                ..background
            },
        );
        assert_eq!(tokens::bg_active(cx), expected_mix(0.10));
        assert_eq!(tokens::accent_soft(cx), expected_mix(0.08));
    });
}

#[test]
fn truncate_middle_composes_the_existing_gpui_contract() {
    let mut actual = div().truncate_middle();
    let mut expected = div()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis_middle();
    assert_eq!(actual.style().clone(), expected.style().clone());
}

#[test]
fn truncating_cells_preserve_outer_size_and_overflow() {
    let mut fixed = truncating_cell("name");
    let mut expected_fixed = div()
        .size_full()
        .flex()
        .items_center()
        .overflow_hidden()
        .text_xs();
    assert_eq!(fixed.style().clone(), expected_fixed.style().clone());

    let mut sized = truncating_cell_sized("name", px(11.));
    let mut expected_sized = div()
        .size_full()
        .flex()
        .items_center()
        .overflow_hidden()
        .text_size(px(11.));
    assert_eq!(sized.style().clone(), expected_sized.style().clone());

    let mut middle = middle_truncating_cell_sized("/long/path/name", px(11.));
    assert_eq!(middle.style().clone(), expected_sized.style().clone());
}

#[gpui::test]
fn tool_popover_preserves_styled_elevation_and_dense_geometry(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    cx.update(|cx| {
        let representative = hsla(210. / 360., 0.25, 0.31, 1.);
        let foreground = hsla(0., 0., 0.96, 1.);
        let structural_border = hsla(20. / 360., 0.20, 0.20, 1.);
        let soft_hairline = hsla(80. / 360., 0.30, 0.45, 0.4);
        let strong_hairline = hsla(190. / 360., 0.50, 0.70, 0.65);
        let background = linear_gradient(
            37.,
            linear_color_stop(hsla(220. / 360., 0.30, 0.22, 1.), 0.),
            linear_color_stop(hsla(185. / 360., 0.25, 0.38, 1.), 1.),
        );
        {
            let theme = Theme::global_mut(cx);
            theme.popover = representative;
            theme.tokens.popover = ThemeToken::new(representative, background);
            theme.popover_foreground = foreground;
            theme.border = structural_border;
            theme.hairline = soft_hairline;
            theme.hairline_strong = strong_hairline;
            theme.radius = px(7.);
        }

        let shadow = vec![
            BoxShadow {
                color: hsla(0., 0., 0., 0.10),
                offset: point(px(0.), px(4.)),
                blur_radius: px(6.),
                spread_radius: px(-1.),
                inset: false,
            },
            BoxShadow {
                color: hsla(0., 0., 0., 0.10),
                offset: point(px(0.), px(2.)),
                blur_radius: px(4.),
                spread_radius: px(-2.),
                inset: false,
            },
        ];
        let mut actual = recipes::tool_popover(cx);
        let mut expected = v_flex()
            .bg(background)
            .text_color(foreground)
            .border_1()
            .border_color(strong_hairline)
            .shadow(shadow)
            .rounded(px(7.))
            .p(tokens::SPACE_3)
            .gap(tokens::SPACE_1)
            .text_size(tokens::TEXT_12);
        assert_eq!(actual.style().clone(), expected.style().clone());
        assert_ne!(actual.style().border_color, Some(structural_border));
        assert_ne!(actual.style().border_color, Some(soft_hairline));
    });
}

#[gpui::test]
fn surface_and_separator_roles_remain_distinct(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    cx.update(|cx| {
        let muted = hsla(10. / 360., 0.2, 0.4, 0.8);
        let hairline = hsla(20. / 360., 0.3, 0.5, 0.6);
        let border = hsla(30. / 360., 0.4, 0.6, 1.0);
        {
            let theme = Theme::global_mut(cx);
            theme.muted = muted;
            theme.hairline = hairline;
            theme.border = border;
        }
        assert_eq!(recipes::on_surface_fill(cx), muted.opacity(0.5));
        assert_eq!(recipes::on_surface_border(cx), hairline);

        let mut actual = recipes::popover_rule(cx);
        let mut expected = div()
            .w_full()
            .flex_none()
            .h(px(1.))
            .my(tokens::SPACE_1P5)
            .bg(border);
        assert_eq!(actual.style().clone(), expected.style().clone());
    });
}

#[gpui::test]
fn typography_and_control_roles_preserve_the_accepted_styles(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    cx.update(|cx| {
        let foreground = hsla(12. / 360., 0.4, 0.7, 1.);
        let muted = hsla(190. / 360., 0.2, 0.45, 1.);
        let danger = hsla(350. / 360., 0.7, 0.5, 1.);
        {
            let theme = Theme::global_mut(cx);
            theme.foreground = foreground;
            theme.muted_foreground = muted;
            theme.danger = danger;
        }

        let mut body = recipes::body("body", cx);
        let mut expected_body = div().text_size(tokens::TEXT_12).text_color(foreground);
        assert_eq!(body.style().clone(), expected_body.style().clone());

        let mut body_muted = recipes::body_muted("body-muted", cx);
        let mut expected_body_muted = div().text_size(tokens::TEXT_12).text_color(muted);
        assert_eq!(
            body_muted.style().clone(),
            expected_body_muted.style().clone(),
        );

        let mut dialog_prose = recipes::dialog_prose("dialog", cx);
        let mut expected_dialog = div().text_size(tokens::TEXT_14).text_color(muted);
        assert_eq!(
            dialog_prose.style().clone(),
            expected_dialog.style().clone()
        );

        let mut value = recipes::value("value", cx);
        assert_eq!(value.style().clone(), expected_body.style().clone());

        let mut value_dense = recipes::value_dense("dense", cx);
        let mut expected_dense = div()
            .text_size(tokens::TEXT_10)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(foreground);
        assert_eq!(value_dense.style().clone(), expected_dense.style().clone());

        let mut caption = recipes::caption("caption", cx);
        let mut expected_caption = div().text_size(tokens::TEXT_10).text_color(muted);
        assert_eq!(caption.style().clone(), expected_caption.style().clone());

        let mut control_label = recipes::control_label("control", cx);
        let mut expected_control_label = div()
            .text_size(tokens::TEXT_14)
            .line_height(relative(1.))
            .text_color(foreground);
        assert_eq!(
            control_label.style().clone(),
            expected_control_label.style().clone(),
        );

        let mut control = recipes::control_body(Button::new("actual-control"));
        let mut expected_control = Button::new("expected-control")
            .small()
            .text_size(tokens::TEXT_12);
        assert_eq!(control.style().clone(), expected_control.style().clone());

        let mut section = recipes::section_label("Section", cx);
        let mut expected_section = div()
            .text_size(tokens::TEXT_13)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(muted);
        assert_eq!(section.style().clone(), expected_section.style().clone());

        let mut required = recipes::required_label("Required", cx);
        let mut expected_required = h_flex().gap(px(4.));
        assert_eq!(required.style().clone(), expected_required.style().clone());
    });
}

#[gpui::test]
fn popover_row_preserves_outer_geometry(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    cx.update(|cx| {
        let mut actual = recipes::popover_row("Label", div(), cx);
        let mut expected = h_flex()
            .w_full()
            .h(px(24.))
            .flex_none()
            .items_center()
            .justify_between()
            .gap(tokens::SPACE_2);
        assert_eq!(actual.style().clone(), expected.style().clone());
    });
}
