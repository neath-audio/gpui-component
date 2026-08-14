//! Reusable text, surface, and menu recipes for styled `gpui-neath` consumers.
//!
//! Each recipe carries the visual canon for one recurring role. Consumers pick
//! a role rather than composing its size, weight, color, surface, or density
//! properties independently.

use super::tokens;
use crate::{ActiveTheme as _, ElevatedSurfaceExt as _, Sizable, h_flex, menu::PopupMenu, v_flex};
use gpui::{App, Div, FontWeight, IntoElement, ParentElement as _, SharedString, Styled, div, px};

fn label_fragment(text: SharedString, color: gpui::Hsla) -> Div {
    div()
        .text_size(tokens::TEXT_13)
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(color)
        .child(text)
}

fn required_label_fragments(text: &str, cx: &App) -> [Div; 2] {
    [
        label_fragment(
            SharedString::from(text.to_string()),
            cx.theme().muted_foreground,
        ),
        label_fragment("*".into(), cx.theme().danger),
    ]
}

/// Section label: TEXT_13 semibold muted (dialog-form tier, user-ruled
/// 2026-07-11), Capitalized as written — never transformed to uppercase.
/// The weight carries the hierarchy.
pub fn section_label(text: &str, cx: &App) -> Div {
    label_fragment(
        SharedString::from(text.to_string()),
        cx.theme().muted_foreground,
    )
}

/// Section label with a red `*` suffix marking the field as required.
pub fn required_label(text: &str, cx: &App) -> Div {
    let [label, marker] = required_label_fragments(text, cx);
    h_flex().gap(px(4.)).child(label).child(marker)
}

/// Body prose/content: TEXT_12 regular foreground.
// Canonical body recipe; its former checkbox/radio-row label consumers all
// moved to `control_label` (they read as a VALUE, not body prose) — zero
// callers currently, kept for the next in-form content site.
#[allow(dead_code)]
pub fn body(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .text_size(tokens::TEXT_12)
        .text_color(cx.theme().foreground)
        .child(text.into())
}

/// Secondary prose: in-form guidance paragraphs, status lines, empty-state
/// bodies. TEXT_12 regular muted. Emphasis (warning/danger) swaps the COLOR
/// at this same size — never a tier jump.
// Canonical body_muted recipe; its former spot-dialog guidance consumers all
// moved to `dialog_prose` (the DialogDescription tier reads at 14, not 12) —
// zero callers currently, kept for the next in-form secondary-line site.
#[allow(dead_code)]
pub fn body_muted(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .text_size(tokens::TEXT_12)
        .text_color(cx.theme().muted_foreground)
        .child(text.into())
}

/// Dialog guidance prose: TEXT_14 regular muted — the DialogDescription
/// tier. Multi-sentence instructions users actually read; confirm-dialog
/// descriptions sit at the same tier.
pub fn dialog_prose(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .text_size(tokens::TEXT_14)
        .text_color(cx.theme().muted_foreground)
        .child(text.into())
}

/// Value/readout (click-to-edit numerics, paths, stats): TEXT_12 regular
/// foreground — the transport treatment.
// Canonical value recipe; current sites with builder chains (click-to-edit
// numerics, path/stat readouts) pin the same TEXT_12/foreground properties
// inline instead of routing through this helper.
#[allow(dead_code)]
pub fn value(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .text_size(tokens::TEXT_12)
        .text_color(cx.theme().foreground)
        .child(text.into())
}

/// Density value for EQ/FX card surfaces ONLY: TEXT_10 semibold foreground.
// Canonical dense-value recipe; the EQ/FX card surfaces that want this
// treatment currently pin TEXT_10/semibold inline at their call sites.
#[allow(dead_code)]
pub fn value_dense(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .text_size(tokens::TEXT_10)
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(cx.theme().foreground)
        .child(text.into())
}

/// Trailing meta (file counts, timestamps, badges): TEXT_10 regular muted.
/// NEVER multi-sentence prose — that is `body_muted`.
pub fn caption(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .text_size(tokens::TEXT_10)
        .text_color(cx.theme().muted_foreground)
        .child(text.into())
}

/// Neutral fill for anything painted ON a surface (badges, chips, soft
/// cards) — a low-opacity `muted` neutral (`muted.opacity(0.5)`) so it
/// reads on any parent without per-context verification.
pub fn on_surface_fill(cx: &App) -> gpui::Hsla {
    cx.theme().muted.opacity(0.5)
}

/// Neutral edge for anything OUTLINED on a surface (drop zones, wells,
/// soft-card borders) — the plain `hairline` theme token, the edge sibling
/// of [`on_surface_fill`]. Structural seams (shell panel dividers, table
/// row lines, title-bar borders) keep the opaque `border`/area tokens —
/// they sit between FIXED surfaces and are tuned per theme.
pub fn on_surface_border(cx: &App) -> gpui::Hsla {
    cx.theme().hairline
}

pub fn control_label(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .text_size(tokens::TEXT_14)
        // Tight line box: the default ~1.5 leading made checkbox/radio rows
        // taller than their 14px indicator, so the row's gap to the control
        // above read ~4px larger than the dialog's nominal rhythm (user
        // smoke 2026-07-21, PT spot dialog).
        .line_height(gpui::relative(1.))
        .text_color(cx.theme().foreground)
        .child(text.into())
}

/// A gpui-component control (Input/Select/…) at `.small()` geometry with
/// content-tier TEXT_12 text. The fork has no 12px text below Medium
/// height except XSmall (which also shrinks geometry); this named idiom
/// replaces the hand-rolled `.small().text_size(..)` override. Relies on
/// the later child `text_size` refinement winning — pinned by the
/// `control_body_overrides_small_text` test in this module's docs sense
/// (visual verification; the ordering contract is documented at the fork's
/// refine_style).
pub fn control_body<T: Sizable + Styled>(control: T) -> T {
    control.small().text_size(tokens::TEXT_12)
}

/// Compact menu tier: 12px rows for menus opened FROM dense tool surfaces
/// (waveform strip, EQ graph, FX/inspector cards). Every other menu stays
/// at the framework default, which the fork renders at 13px. This is the
/// ONLY sanctioned second menu tier.
pub fn compact_menu(menu: PopupMenu) -> PopupMenu {
    menu.xsmall()
}

/// Alpha for menus/popovers opened over dense tool surfaces — the single
/// tweak point for how much of the surface ghosts through.
pub const MENU_BG_ALPHA: f32 = 0.92;

/// The full "menu over a dense tool surface" recipe: [`compact_menu`]
/// density plus the shared translucent popover tint ([`MENU_BG_ALPHA`]).
/// Used by the waveform strip, EQ graph, corpus dots, and the transport
/// bar's menus.
pub fn overlay_menu(menu: PopupMenu, cx: &App) -> PopupMenu {
    compact_menu(menu).menu_bg(cx.theme().popover.opacity(MENU_BG_ALPHA))
}

/// Chrome for an interactive popover opened from a tool surface — the
/// container twin of [`overlay_menu`]. OPAQUE by default (styled
/// elevated-surface card); the transport process panel is the sole tint
/// opt-in (re-applies [`MENU_BG_ALPHA`] locally — shipped canon).
///
/// The table grid (spec 2026-07-13-tool-popover-table-redesign): `SPACE_3`
/// (12px) inset, `SPACE_1` (4px) between rows, `TEXT_12` tier. Rows are
/// fixed-height [`popover_row`]s; groups are fenced by [`popover_rule`]s
/// that carry their own vertical padding, so group separation reads at a
/// visibly different beat from row separation.
///
/// The panel paints this ITSELF, so its `Popover` MUST be mounted
/// `.appearance(false)` — otherwise the component draws a second card
/// behind it.
///
/// NOT for the inspector's fake-menu column (`file_mode.rs`), which
/// imitates `PopupMenu` geometry rather than a param panel — leave it alone.
pub fn tool_popover(cx: &App) -> Div {
    v_flex()
        .elevated_surface(cx)
        .p(tokens::SPACE_3)
        .gap(tokens::SPACE_1)
        .text_size(tokens::TEXT_12)
}

/// The faint full-width hairline that fences groups — and the footer —
/// inside a [`tool_popover`]. ONE element carrying its own `SPACE_1P5`
/// vertical MARGINS — never a padded wrapper: an earlier revision nested
/// the 1px line inside a plain `div()`, and the block-layout wrapper
/// swallowed it (the hairline painted on no panel; user-flagged at smoke
/// round 6). With the card's 4px row gap on both sides, a group boundary
/// lands at ~21px against the 4px row beat, which is what makes grouping
/// legible. Compact-variant callers may chain `.my(...)` to tighten the
/// beat (later margin wins).
pub fn popover_rule(cx: &App) -> Div {
    div()
        .w_full()
        .flex_none()
        .h(px(1.))
        .my(tokens::SPACE_1P5)
        .bg(cx.theme().border)
}

/// The one-line row atom of a [`tool_popover`]: fixed 24px height (the
/// framework Slider's hit container is exactly 24px, so an inline track
/// fills the row with zero phantom air), muted label hard left, control
/// cluster hard right.
pub fn popover_row(label: impl Into<SharedString>, value: impl IntoElement, cx: &App) -> Div {
    h_flex()
        .w_full()
        .h(px(24.))
        .flex_none()
        .items_center()
        .justify_between()
        .gap(tokens::SPACE_2)
        .child(
            div()
                .text_color(cx.theme().muted_foreground)
                .child(label.into()),
        )
        .child(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Theme;
    use gpui::{TestAppContext, hsla};

    #[gpui::test]
    fn required_label_fragments_preserve_muted_and_danger_typography(cx: &mut TestAppContext) {
        cx.update(crate::init);
        cx.update(|cx| {
            let muted = hsla(190. / 360., 0.2, 0.45, 1.);
            let danger = hsla(350. / 360., 0.7, 0.5, 1.);
            {
                let theme = Theme::global_mut(cx);
                theme.muted_foreground = muted;
                theme.danger = danger;
            }

            let [mut label, mut marker] = required_label_fragments("Required", cx);
            let mut expected_label = div()
                .text_size(tokens::TEXT_13)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(muted);
            let mut expected_marker = div()
                .text_size(tokens::TEXT_13)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(danger);
            assert_eq!(label.style().clone(), expected_label.style().clone());
            assert_eq!(marker.style().clone(), expected_marker.style().clone());
        });
    }
}
