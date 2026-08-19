//! Neutral typography, spacing, border, radius, icon, and surface tokens for
//! styled `gpui-neath` consumers.
//!
//! Typography values are rem-based and therefore follow the window rem size.
//! Surface functions take `&App` so they always resolve colors from the live
//! theme.

use crate::ActiveTheme as _;
use gpui::{App, Hsla, Pixels, Rems, px, rems};

/// 0.625rem (10px) for captions, pills, counts, and trailing row metadata.
pub const TEXT_10: Rems = rems(0.625);

/// 0.75rem (12px) for content rows, table cells, and compact menus.
/// Equivalent to gpui's `.text_xs()`.
pub const TEXT_12: Rems = rems(0.75);

/// 0.8125rem (13px) for field and section labels and menu rows.
/// Sits between the 12px content tier and the 14px form tier.
pub const TEXT_13: Rems = rems(0.8125);

/// 0.875rem (14px) for standard menus, tooltips, controls, and dialog prose.
/// Equivalent to gpui's `.text_sm()`.
pub const TEXT_14: Rems = rems(0.875);

/// 0.9375rem (15px) for dialog and sheet titles.
pub const TEXT_15: Rems = rems(0.9375);

/// 1.0rem (16px) for page heroes and empty-state titles.
/// Equivalent to gpui's `.text_base()`.
pub const TEXT_16: Rems = rems(1.0);

/// 2px chip inner gap.
pub const SPACE_HALF: Pixels = px(2.);

/// 4px icon-text or otherwise tight gap.
pub const SPACE_1: Pixels = px(4.);

/// 6px chrome-row gap.
pub const SPACE_1P5: Pixels = px(6.);

/// 8px default comfortable gap.
pub const SPACE_2: Pixels = px(8.);

/// 12px default panel padding.
pub const SPACE_3: Pixels = px(12.);

/// 16px section-break padding.
pub const SPACE_4: Pixels = px(16.);

/// 24px major break.
pub const SPACE_6: Pixels = px(24.);

/// 2px accent rim for selected rows and active tabs.
pub const ACCENT_RIM_PX: Pixels = px(2.);

/// 4px radius for small buttons and panel-header icon boxes.
pub const RADIUS_SM: Pixels = px(4.);

/// 6px radius for tabs, inputs, and cards.
pub const RADIUS_MD: Pixels = px(6.);

/// 8px radius for popovers and dialogs.
pub const RADIUS_LG: Pixels = px(8.);

/// 100px radius for chips and pills.
pub const RADIUS_PILL: Pixels = px(100.);

/// 12px icons placed inline with text or section labels.
pub const ICON_INLINE: Pixels = px(12.);

/// 14px default chrome icons, such as panel toggles and button icons.
pub const ICON_CHROME: Pixels = px(14.);

/// 16px primary-action icons.
pub const ICON_PRIMARY: Pixels = px(16.);

/// Recessed chrome surface, 4% darker than `theme.background`.
pub fn bg_sunken(cx: &App) -> Hsla {
    cx.theme().bg_sunken()
}

/// Selected-row surface, mixed 10% toward `theme.accent`.
pub fn bg_active(cx: &App) -> Hsla {
    cx.theme().bg_active()
}

/// Soft accent-tinted surface, mixed 8% from `theme.background` toward
/// `theme.accent`.
pub fn accent_soft(cx: &App) -> Hsla {
    cx.theme().accent_soft()
}
