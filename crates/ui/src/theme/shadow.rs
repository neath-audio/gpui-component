//! Box-shadow tiers = Tailwind's sm/md/lg/xl recipes (which Nova uses),
//! SAME values in both modes — browser-measured from the live Nova preview
//! (user-ruled 2026-07-20): dark mode does NOT boost shadows; dark depth
//! comes from lighter surfaces + luminous hairlines. Plain constants — no
//! theme state involved.

use gpui::{BoxShadow, hsla, point, px};
use smallvec::{SmallVec, smallvec};

use crate::theme::Theme;

fn layer(y: f32, blur: f32, spread: f32, alpha: f32) -> BoxShadow {
    BoxShadow {
        color: hsla(0., 0., 0., alpha),
        offset: point(px(0.), px(y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        inset: false,
    }
}

impl Theme {
    /// Tailwind `shadow-sm`.
    pub fn shadow_1(&self) -> SmallVec<[BoxShadow; 2]> {
        smallvec![layer(1., 3., 0., 0.10), layer(1., 2., -1., 0.10)]
    }
    /// Tailwind `shadow-md` — the measured Nova menu/popover shadow.
    pub fn shadow_2(&self) -> SmallVec<[BoxShadow; 2]> {
        smallvec![layer(4., 6., -1., 0.10), layer(2., 4., -2., 0.10)]
    }
    /// Tailwind `shadow-lg`.
    pub fn shadow_3(&self) -> SmallVec<[BoxShadow; 2]> {
        smallvec![layer(10., 15., -3., 0.10), layer(4., 6., -4., 0.10)]
    }
    /// Tailwind `shadow-xl` — dialogs/sheets.
    pub fn shadow_4(&self) -> SmallVec<[BoxShadow; 2]> {
        smallvec![layer(20., 25., -5., 0.10), layer(8., 10., -6., 0.10)]
    }
}

#[cfg(test)]
mod tests {
    use crate::theme::Theme;

    #[test]
    fn shadow_ladder_grows() {
        let theme = Theme::default();
        assert!(theme.shadow_1().len() >= 1);
        assert!(
            theme.shadow_4().last().unwrap().blur_radius
                > theme.shadow_1().last().unwrap().blur_radius
        );
    }
}
