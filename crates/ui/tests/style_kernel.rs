use gpui::{Hsla, Styled as _, TestAppContext, div, hsla, px, rems};
use gpui_neath::{
    Theme,
    style::{
        tokens,
        typography::{
            TruncateMiddleExt as _, middle_truncating_cell_sized, truncating_cell,
            truncating_cell_sized,
        },
    },
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
