//! Elevation: surface / hairline / shadow triples per semantic level.
//! DEPRECATED by the flat-token teardown (neath spec
//! 2026-07-21-flat-token-system-design.md) — the resolver now only relays
//! plain theme tokens; the module is deleted once the last consumer is gone.

use gpui::{BoxShadow, Hsla};
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

impl Theme {
    /// Resolve the elevation triple for a level: explicit theme override
    /// first, otherwise derived from `background` (mode-aware).
    pub fn elevation(&self, level: ElevationLevel) -> Elevation {
        let bg = self.colors.background;
        let dark = self.is_dark();
        let surface = match level {
            ElevationLevel::Sunken => self.colors.well,
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
            ElevationLevel::Sunken | ElevationLevel::Base => self.colors.hairline,
            _ => self.colors.hairline_strong,
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
        self.colors.input_fill
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Theme, ThemeColor};

    #[test]
    fn old_theme_json_still_loads() {
        // A ThemeColor JSON without the new plain fields (or the surviving
        // Option overrides) must still deserialize — `#[serde(default)]`.
        let json = serde_json::to_string(&ThemeColor::default()).unwrap();
        let stripped: serde_json::Value = serde_json::from_str(&json).unwrap();
        let mut obj = stripped.as_object().unwrap().clone();
        for k in [
            "elevation_raised",
            "elevation_overlay",
            "hairline",
            "hairline_strong",
            "well",
            "input_fill",
            "surface_fill",
        ] {
            obj.remove(k);
        }
        let restored: ThemeColor = serde_json::from_value(serde_json::Value::Object(obj)).unwrap();
        assert!(restored.elevation_raised.is_none());
    }

    #[test]
    fn derived_ladder_is_ordered_dark() {
        let mut theme = Theme::default();
        theme.mode = crate::ThemeMode::Dark;
        theme.colors.background = gpui::hsla(0., 0., 0.12, 1.);
        theme.colors.well = gpui::hsla(0., 0., 0.10, 1.);
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

}
