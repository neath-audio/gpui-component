use crate::ActiveTheme as _;
use gpui::{App, Hsla, Pixels, Rems, px, rems};

pub const TEXT_10: Rems = rems(0.625);
pub const TEXT_12: Rems = rems(0.75);
pub const TEXT_13: Rems = rems(0.8125);
pub const TEXT_14: Rems = rems(0.875);
pub const TEXT_15: Rems = rems(0.9375);
pub const TEXT_16: Rems = rems(1.0);

pub const SPACE_HALF: Pixels = px(2.);
pub const SPACE_1: Pixels = px(4.);
pub const SPACE_1P5: Pixels = px(6.);
pub const SPACE_2: Pixels = px(8.);
pub const SPACE_3: Pixels = px(12.);
pub const SPACE_4: Pixels = px(16.);
pub const SPACE_6: Pixels = px(24.);

pub const ACCENT_RIM_PX: Pixels = px(2.);

pub const RADIUS_SM: Pixels = px(4.);
pub const RADIUS_MD: Pixels = px(6.);
pub const RADIUS_LG: Pixels = px(8.);
pub const RADIUS_PILL: Pixels = px(100.);

pub const ICON_INLINE: Pixels = px(12.);
pub const ICON_CHROME: Pixels = px(14.);
pub const ICON_PRIMARY: Pixels = px(16.);

pub fn bg_sunken(cx: &App) -> Hsla {
    mix_toward_black(cx.theme().background, 0.04)
}

pub fn bg_active(cx: &App) -> Hsla {
    mix(cx.theme().background, cx.theme().accent, 0.10)
}

pub fn accent_soft(cx: &App) -> Hsla {
    mix(cx.theme().background, cx.theme().accent, 0.08)
}

fn mix(base: Hsla, target: Hsla, amount: f32) -> Hsla {
    Hsla {
        h: base.h * (1.0 - amount) + target.h * amount,
        s: base.s * (1.0 - amount) + target.s * amount,
        l: base.l * (1.0 - amount) + target.l * amount,
        a: base.a * (1.0 - amount) + target.a * amount,
    }
}

fn mix_toward_black(base: Hsla, amount: f32) -> Hsla {
    Hsla {
        l: (base.l - amount).max(0.0),
        ..base
    }
}
