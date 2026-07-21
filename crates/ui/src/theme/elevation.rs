//! Elevation: surface / hairline / shadow triples per semantic level.
//! Initial constants per the depth-color spec; tuned via theme presets.
//!
//! Dark-mode hairlines are luminous alpha-white rather than opaque lifted
//! grays: `hairline` fallback = white @ 10% (`hsla(0., 0., 1., 0.10)`),
//! `hairline_strong` fallback = white @ 15% (`hsla(0., 0., 1., 0.15)`) — a
//! translucent edge reads correctly over ANY dark surface it happens to sit
//! on, whereas an opaque gray only matched the one surface it was tuned
//! against. Light mode is unchanged (opaque steps off `border`). Ruling
//! 2026-07-20, shadcn v4 dark tokens
//! (docs/superpowers/specs/2026-07-19-depth-color-language-design.md, neath
//! repo).

use gpui::{BoxShadow, Hsla, hsla, point, px};
use serde::{Deserialize, Serialize};
use smallvec::{SmallVec, smallvec};

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElevationLevel {
    Sunken,
    Base,
    Raised,
    Overlay,
}

#[derive(Debug, Clone)]
pub struct Elevation {
    pub surface: Hsla,
    pub hairline: Hsla,
    pub shadow: SmallVec<[BoxShadow; 2]>,
}

fn shift_l(c: Hsla, dl: f32) -> Hsla {
    Hsla {
        l: (c.l + dl).clamp(0., 1.),
        ..c
    }
}

fn layer(y: f32, blur: f32, spread: f32, alpha: f32) -> BoxShadow {
    BoxShadow {
        color: hsla(0., 0., 0., alpha),
        offset: point(px(0.), px(y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        inset: false,
    }
}

// Shadow tiers = Tailwind's sm/md/lg/xl recipes (which Nova uses), SAME
// values in both modes — browser-measured from the live Nova preview
// (user-ruled 2026-07-20): dark mode does NOT boost shadows; dark depth
// comes from lighter surfaces + luminous alpha-white hairlines. The
// earlier mode-gated alphas (dark .30-.50) read far heavier than the
// reference.
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

    /// Resolve the elevation triple for a level: explicit theme override
    /// first, otherwise derived from `background` (mode-aware).
    pub fn elevation(&self, level: ElevationLevel) -> Elevation {
        let bg = self.colors.background;
        let dark = self.is_dark();
        let surface = match level {
            ElevationLevel::Sunken => self
                .colors
                .elevation_sunken
                .unwrap_or_else(|| shift_l(bg, if dark { -0.02 } else { -0.035 })),
            ElevationLevel::Base => bg,
            ElevationLevel::Raised => self
                .colors
                .elevation_raised
                .unwrap_or_else(|| shift_l(bg, if dark { 0.02 } else { 0.01 })),
            ElevationLevel::Overlay => self
                .colors
                .elevation_overlay
                // Subtle: the earlier shadcn-derived 0.06 lift made large
                // overlays (dialogs) read as foreign to tinted themes —
                // user-ruled 2026-07-21 that hairline + shadow carry the
                // depth and the surface stays in the theme's family.
                .unwrap_or_else(|| shift_l(bg, if dark { 0.03 } else { 0. })),
        };
        let hairline = match level {
            ElevationLevel::Sunken | ElevationLevel::Base => {
                self.colors.hairline.unwrap_or_else(|| {
                    if dark {
                        hsla(0., 0., 1., 0.10)
                    } else {
                        let b = self.colors.border;
                        Hsla { a: b.a * 0.6, ..b }
                    }
                })
            }
            _ => self.colors.hairline_strong.unwrap_or_else(|| {
                if dark {
                    hsla(0., 0., 1., 0.15)
                } else {
                    self.colors.border
                }
            }),
        };
        let shadow = match level {
            ElevationLevel::Sunken | ElevationLevel::Base => smallvec![],
            ElevationLevel::Raised => self.shadow_1(),
            // Tier 2 (shadow-md), not 3 — matches the measured Nova
            // menu/popover shadow; dialogs/sheets override to shadow_4.
            ElevationLevel::Overlay => self.shadow_2(),
        };
        Elevation {
            surface,
            hairline,
            shadow,
        }
    }

    /// Fill color for input/checkbox/radio control interiors.
    ///
    /// `elevation(Sunken)` is an ABSOLUTE well anchored to `background` —
    /// right for table viewports, tree panes, scroll areas, the surfaces
    /// that always sit directly on the window background. A control's
    /// interior doesn't have that guarantee: it can sit on any parent
    /// surface (a raised toolbar, an overlay dialog, ...), so painting it
    /// with the absolute Sunken surface reads as a flat mismatched patch
    /// once the parent isn't `Base`. Dark mode instead uses a RELATIVE
    /// translucent lighten — shadcn Nova's `dark:bg-input/30` — here white
    /// @ 5% (`hsla(0., 0., 1., 0.05)`), which composites correctly over
    /// whatever surface it lands on. Light mode is unchanged: it still
    /// reads the Sunken well directly (light hairlines/surfaces don't have
    /// the same cross-surface mismatch light leaks dark suffers).
    pub fn input_fill(&self) -> Hsla {
        if self.is_dark() {
            hsla(0., 0., 1., 0.05)
        } else {
            self.elevation(ElevationLevel::Sunken).surface
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Theme, ThemeColor};

    #[test]
    fn old_theme_json_still_loads() {
        // A ThemeColor JSON with none of the new fields must deserialize.
        let json = serde_json::to_string(&ThemeColor::default()).unwrap();
        let stripped: serde_json::Value = serde_json::from_str(&json).unwrap();
        let mut obj = stripped.as_object().unwrap().clone();
        for k in [
            "elevation_sunken",
            "elevation_raised",
            "elevation_overlay",
            "hairline",
            "hairline_strong",
        ] {
            obj.remove(k);
        }
        let restored: ThemeColor = serde_json::from_value(serde_json::Value::Object(obj)).unwrap();
        assert!(restored.elevation_sunken.is_none());
    }

    #[test]
    fn derived_ladder_is_ordered_dark() {
        let mut theme = Theme::default();
        theme.mode = crate::ThemeMode::Dark;
        theme.colors.background = gpui::hsla(0., 0., 0.12, 1.);
        let sunken = theme.elevation(ElevationLevel::Sunken).surface;
        let base = theme.elevation(ElevationLevel::Base).surface;
        let raised = theme.elevation(ElevationLevel::Raised).surface;
        let overlay = theme.elevation(ElevationLevel::Overlay).surface;
        assert!(sunken.l < base.l && base.l < raised.l && raised.l < overlay.l);
    }

    #[test]
    fn explicit_override_wins() {
        let mut theme = Theme::default();
        let custom = gpui::hsla(0.6, 0.2, 0.3, 1.);
        theme.colors.elevation_raised = Some(custom);
        assert_eq!(theme.elevation(ElevationLevel::Raised).surface, custom);
    }

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
