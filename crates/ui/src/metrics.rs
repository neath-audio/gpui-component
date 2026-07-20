//! One canonical control-size ladder. Components consume `size.metrics()`
//! and never match on `Size` for geometry. Spec:
//! neath docs/superpowers/specs/2026-07-19-depth-color-language-design.md
//!
//! Ladder = a uniform 4px staircase: xxs/xs/sm/default/lg heights =
//! 20/24/28/32/36px. XXSmall is neath's dense-chrome tier (user ruling
//! 2026-07-20); XSmall-Large stay shadcn Nova parity, extracted from the
//! shadcn registry's radix-nova components (treated as authoritative),
//! user-ruled 2026-07-20
//! (docs/superpowers/specs/2026-07-19-depth-color-language-design.md, neath
//! repo).

use gpui::{Pixels, Rems, px, rems};

use crate::{Size, theme::Theme};

/// Rem base the ladder is authored against (gpui window default).
const REM: f32 = 16.;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlMetrics {
    pub height: Rems,
    pub pad_x: Rems,
    pub pad_y: Rems,
    pub text: Rems,
    pub icon: Rems,
    pub gap: Rems,
    pub radius: Rems,
}

/// (height_px, pad_x, pad_y, text, icon, gap, radius) — px at 16 rem base.
const LADDER: [(f32, f32, f32, f32, f32, f32, f32); 5] = [
    (20., 4., 2., 12., 12., 4., 4.),  // XXSmall (neath dense chrome)
    (24., 8., 2., 12., 12., 4., 4.),  // XSmall
    (28., 10., 3., 13., 14., 4., 6.), // Small
    (32., 10., 4., 14., 16., 6., 6.), // Medium
    (36., 10., 4., 14., 16., 6., 8.), // Large
];

fn row(i: usize) -> ControlMetrics {
    let (h, px_, py, t, ic, g, r) = LADDER[i];
    ControlMetrics {
        height: rems(h / REM),
        pad_x: rems(px_ / REM),
        pad_y: rems(py / REM),
        text: rems(t / REM),
        icon: rems(ic / REM),
        gap: rems(g / REM),
        radius: rems(r / REM),
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn interpolated(height_px: f32) -> ControlMetrics {
    if height_px <= LADDER[0].0 {
        return ControlMetrics {
            height: rems(height_px / REM),
            ..row(0)
        };
    }
    if height_px >= LADDER[4].0 {
        return ControlMetrics {
            height: rems(height_px / REM),
            ..row(4)
        };
    }
    let i = (0..4)
        .find(|&i| height_px < LADDER[i + 1].0)
        .expect("height inside ladder bounds");
    let (lo, hi) = (LADDER[i], LADDER[i + 1]);
    let t = (height_px - lo.0) / (hi.0 - lo.0);
    ControlMetrics {
        height: rems(height_px / REM),
        pad_x: rems(lerp(lo.1, hi.1, t) / REM),
        pad_y: rems(lerp(lo.2, hi.2, t) / REM),
        text: rems(lerp(lo.3, hi.3, t) / REM),
        icon: rems(lerp(lo.4, hi.4, t) / REM),
        gap: rems(lerp(lo.5, hi.5, t) / REM),
        radius: rems(lerp(lo.6, hi.6, t) / REM),
    }
}

impl Size {
    /// The canonical geometry for this size. The ONLY sanctioned way for a
    /// component to turn a `Size` into dimensions.
    pub fn metrics(&self) -> ControlMetrics {
        match self {
            Size::XXSmall => row(0),
            Size::XSmall => row(1),
            Size::Small => row(2),
            Size::Medium => row(3),
            Size::Large => row(4),
            Size::Size(v) => interpolated(Pixels::from(*v).as_f32()),
        }
    }

    /// Theme-relative corner radius for control chrome (buttons, toggles) —
    /// shadcn Nova's curve, user-ruled 2026-07-20: XSmall/Small take
    /// `theme.radius − 2px` clamped at Nova's 10/12px caps
    /// (`rounded-[min(var(--radius-md),10px|12px)]`); Medium/Large take the
    /// full `theme.radius` (`rounded-lg`). At the default 10px radius this
    /// yields 8/8/10/10. A flat `theme.radius` at every size read as a pill
    /// on the short variants. Custom sizes split at the Medium height.
    /// XXSmall (the neath dense tier below the Nova curve) mirrors XSmall.
    pub fn control_radius(&self, theme: &Theme) -> Pixels {
        let radius_md = (theme.radius - px(2.)).max(px(0.));
        match self {
            Size::XXSmall | Size::XSmall => radius_md.min(px(10.)),
            Size::Small => radius_md.min(px(12.)),
            Size::Medium | Size::Large => theme.radius,
            Size::Size(v) if *v < px(32.) => radius_md.min(px(12.)),
            Size::Size(_) => theme.radius,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Size;
    use gpui::{px, rems};

    #[test]
    fn ladder_matches_canonical_table() {
        let m = Size::Medium.metrics();
        assert_eq!(m.height, rems(2.0));
        assert_eq!(m.pad_x, rems(0.625));
        assert_eq!(m.text, rems(0.875));
        assert_eq!(m.icon, rems(1.0));

        // XXSmall = neath dense-chrome tier (user ruling 2026-07-20):
        // 20px height, 4/2 pads, 12px text+icon, 4px gap+radius.
        let xxs = Size::XXSmall.metrics();
        assert_eq!(xxs.height, rems(1.25));
        assert_eq!(xxs.pad_x, rems(0.25));
        assert_eq!(xxs.pad_y, rems(0.125));
        assert_eq!(xxs.text, rems(0.75));
        assert_eq!(xxs.icon, rems(0.75));
        assert_eq!(xxs.gap, rems(0.25));
        assert_eq!(xxs.radius, rems(0.25));

        // XSmall stays byte-identical Nova parity below the new tier.
        let xs = Size::XSmall.metrics();
        assert_eq!(xs.height, rems(1.5));
        assert_eq!(xs.pad_x, rems(0.5));
    }

    #[test]
    fn control_radius_follows_nova_curve() {
        // Default theme radius is 10 → 8/8/8/10/10 (XXSmall mirrors XSmall).
        let mut theme = crate::theme::Theme::default();
        theme.radius = px(10.);
        assert_eq!(Size::XXSmall.control_radius(&theme), px(8.));
        assert_eq!(Size::XSmall.control_radius(&theme), px(8.));
        assert_eq!(Size::Small.control_radius(&theme), px(8.));
        assert_eq!(Size::Medium.control_radius(&theme), px(10.));
        assert_eq!(Size::Large.control_radius(&theme), px(10.));
        // Nova's caps bind for exotic large radii: min(radius−2, 10|12).
        theme.radius = px(20.);
        assert_eq!(Size::XXSmall.control_radius(&theme), px(10.));
        assert_eq!(Size::XSmall.control_radius(&theme), px(10.));
        assert_eq!(Size::Small.control_radius(&theme), px(12.));
        assert_eq!(Size::Large.control_radius(&theme), px(20.));
        // Small radii degrade gracefully (radius−2, floored at 0).
        theme.radius = px(1.);
        assert_eq!(Size::XXSmall.control_radius(&theme), px(0.));
        assert_eq!(Size::XSmall.control_radius(&theme), px(0.));
        // Custom sizes split at the Medium height.
        theme.radius = px(10.);
        assert_eq!(Size::Size(px(26.)).control_radius(&theme), px(8.));
        assert_eq!(Size::Size(px(40.)).control_radius(&theme), px(10.));
    }

    #[test]
    fn ladder_is_strictly_monotonic() {
        // Nova's box-model fields (pad_x/pad_y/text/icon/gap/radius) plateau
        // between adjacent tiers (e.g. icon holds at 16px across Medium/
        // Large), so only `height` stays strictly increasing; the rest relax
        // to non-decreasing.
        let steps = [
            Size::XXSmall,
            Size::XSmall,
            Size::Small,
            Size::Medium,
            Size::Large,
        ];
        for pair in steps.windows(2) {
            let (a, b) = (pair[0].metrics(), pair[1].metrics());
            assert!(a.height.0 < b.height.0);
            assert!(a.pad_x.0 <= b.pad_x.0);
            assert!(a.pad_y.0 <= b.pad_y.0);
            assert!(a.text.0 <= b.text.0);
            assert!(a.icon.0 <= b.icon.0);
            assert!(a.gap.0 <= b.gap.0);
            assert!(a.radius.0 <= b.radius.0);
        }
    }

    #[test]
    fn custom_size_interpolates_between_neighbors() {
        // 26px sits halfway between XSmall(24) and Small(28)
        let m = Size::Size(px(26.)).metrics();
        assert_eq!(m.height, rems(26. / 16.));
        assert_eq!(m.pad_x, rems((0.5 + 0.625) / 2.));
        // 22px sits halfway between XXSmall(20) and XSmall(24)
        let m = Size::Size(px(22.)).metrics();
        assert_eq!(m.height, rems(22. / 16.));
        assert_eq!(m.pad_x, rems((0.25 + 0.5) / 2.));
        // clamped below the ladder (10px < XXSmall's 20px floor)
        let lo = Size::Size(px(10.)).metrics();
        assert_eq!(lo.pad_x, Size::XXSmall.metrics().pad_x);
        // clamped above (64px > Large's 36px ceiling)
        let hi = Size::Size(px(64.)).metrics();
        assert_eq!(hi.pad_x, Size::Large.metrics().pad_x);
    }

    #[test]
    fn custom_smaller_larger_step_by_4px() {
        assert_eq!(Size::Size(px(26.)).smaller(), Size::Size(px(22.)));
        assert_eq!(Size::Size(px(26.)).larger(), Size::Size(px(30.)));
    }
}
