use crate::{
    highlighter::HighlightTheme, list::ListSettings, notification::NotificationSettings,
    scroll::ScrollbarMode, sheet::SheetSettings,
};
use gpui::{
    App, Global, Hsla, IsZero as _, Pixels, SharedString, Window, WindowAppearance, hsla, px,
};
pub use gpui_base::{RadiusTokens, ShadowTokens, SpacingTokens, TextStyleToken, TypographyTokens};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{rc::Rc, sync::Arc, time::Duration};

mod color;
mod paint;
mod registry;
mod schema;
mod shadow;

pub use color::*;
pub use paint::{
    INK_ZEBRA, PRESS_ACTIVE, SCRIM_ALPHA_DARK, WASH_HOVER, WASH_SELECTED, contrast_ratio, flatten,
};
pub use registry::*;
pub use schema::*;

pub fn init(cx: &mut App) {
    registry::init(cx);

    // Ensure theme is loaded directly on startup for WASM compatibility
    Theme::change(ThemeMode::Light, None, cx);
    Theme::sync_scrollbar_appearance(cx);
}

pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for App {
    #[inline(always)]
    fn theme(&self) -> &Theme {
        Theme::global(self)
    }
}

fn default_true() -> bool {
    true
}

/// The radius that rounds a shape as far as its own size allows, giving a
/// circle or a pill. Any value past half the shorter side is clamped by the
/// renderer, so this is simply "as round as it goes".
const RADIUS_FULL: Pixels = px(9999.);

/// How long the scrollbar stays visible after the last scroll, drag, or hover.
const SCROLLBAR_IDLE: Duration = Duration::from_secs(2);
/// How long the scrollbar takes to appear.
const SCROLLBAR_ENTER: Duration = Duration::from_millis(300);
/// How long the scrollbar takes to fade away once the idle hold expires.
const SCROLLBAR_EXIT: Duration = Duration::from_millis(500);
/// How long the thumb takes to reach its hovered or resting width.
const SCROLLBAR_EXPAND: Duration = Duration::from_millis(300);

/// The scrollbar motion this design system projects onto Base.
///
/// Scrolling and track hover reveal a scrollbar by fading it in place. In hover
/// mode, pointing at the thumb slides it in from the nearest edge as it fades.
fn scrollbar_motion(mode: ScrollbarMode) -> gpui_base::ScrollbarMotion {
    gpui_base::ScrollbarMotion::default()
        .with_idle(SCROLLBAR_IDLE)
        .with_enter(SCROLLBAR_ENTER)
        .with_exit(SCROLLBAR_EXIT)
        .with_expand(SCROLLBAR_EXPAND)
        .with_entrance(gpui_base::ScrollbarEntrance::Fade)
        .with_thumb_hover_entrance(match mode {
            ScrollbarMode::Scrolling | ScrollbarMode::Always => gpui_base::ScrollbarEntrance::Fade,
            ScrollbarMode::Hover => gpui_base::ScrollbarEntrance::SlideAndFade,
        })
}

/// The global theme configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Theme {
    /// Bezel role colors.
    #[serde(default)]
    pub bg: Hsla,
    #[serde(default)]
    pub surface: Hsla,
    #[serde(default)]
    pub surface_raised: Hsla,
    #[serde(default)]
    pub surface_raised_hover: Hsla,
    #[serde(default)]
    pub surface_card: Hsla,
    #[serde(default)]
    pub surface_dialog: Hsla,
    #[serde(default)]
    pub surface_overlay: Hsla,
    #[serde(default)]
    pub band: Hsla,
    #[serde(default)]
    pub input_bg: Hsla,
    #[serde(default)]
    pub text: Hsla,
    #[serde(default)]
    pub text_muted: Hsla,
    #[serde(default)]
    pub text_faint: Hsla,
    #[serde(default)]
    pub text_dim: Hsla,
    #[serde(default)]
    pub border: Hsla,
    #[serde(default)]
    pub border_strong: Hsla,
    #[serde(default)]
    pub accent: Hsla,
    #[serde(default)]
    pub accent_strong: Hsla,
    #[serde(default)]
    pub on_accent: Hsla,
    #[serde(default)]
    pub solid: Hsla,
    #[serde(default)]
    pub on_solid: Hsla,
    #[serde(default)]
    pub danger: Hsla,
    #[serde(default)]
    pub danger_muted: Hsla,
    #[serde(default)]
    pub danger_strong: Hsla,
    #[serde(default)]
    pub warning: Hsla,
    #[serde(default)]
    pub warning_muted: Hsla,
    #[serde(default)]
    pub success: Hsla,
    #[serde(default)]
    pub success_muted: Hsla,
    #[serde(default)]
    pub busy: Hsla,
    #[serde(default)]
    pub focus: Hsla,
    #[serde(default)]
    pub row_hover: Hsla,
    #[serde(default)]
    pub row_selected: Hsla,
    #[serde(default)]
    pub table_header_bg: Hsla,
    #[serde(default)]
    pub table_row_hover: Hsla,
    #[serde(default)]
    pub table_row_selected: Hsla,
    #[serde(default)]
    pub table_row_selected_border: Hsla,
    #[serde(default)]
    pub selection: Hsla,
    #[serde(default)]
    pub caret: Hsla,
    #[serde(default)]
    pub cursor: Hsla,
    pub highlight_theme: Arc<HighlightTheme>,
    pub light_theme: Rc<ThemeConfig>,
    pub dark_theme: Rc<ThemeConfig>,

    pub mode: ThemeMode,
    /// The font family for the application, default is `.SystemUIFont`.
    pub font_family: SharedString,
    /// The base font size for the application, default is 16px.
    pub font_size: Pixels,
    /// The monospace font family for the application.
    ///
    /// Defaults to:
    ///
    /// - macOS: `Menlo`
    /// - Windows: `Consolas`
    /// - Linux: `DejaVu Sans Mono`
    pub mono_font_family: SharedString,
    /// The monospace font size for the application, default is 13px.
    pub mono_font_size: Pixels,
    /// Radius for the general elements.
    pub radius: Pixels,
    /// Radius for the large elements, e.g.: Dialog, Notification border radius.
    pub radius_lg: Pixels,
    pub shadow: bool,
    /// Whether focused controls draw a ring outside their border, default true.
    ///
    /// The ring is painted outside the element, so any ancestor that clips its
    /// content will cut it off. An application whose layout clips heavily can
    /// turn it off here: focused controls then show only their tinted border,
    /// which costs no space and cannot be clipped.
    #[serde(default = "default_true")]
    pub focus_ring: bool,
    pub transparent: Hsla,
    /// Show the scrollbar mode, default: Scrolling
    #[serde(alias = "scrollbar_show")]
    pub scrollbar_mode: ScrollbarMode,
    /// The notification setting.
    #[serde(skip)]
    pub notification: NotificationSettings,
    /// Tile grid size, default is 4px.
    pub tile_grid_size: Pixels,
    /// The shadow of the tile panel.
    pub tile_shadow: bool,
    /// The border radius of the tile panel, default is 0px.
    pub tile_radius: Pixels,
    /// The list settings.
    pub list: ListSettings,
    /// The sheet settings.
    pub sheet: SheetSettings,
}

impl Default for Theme {
    fn default() -> Self {
        Self::fallback_dark()
    }
}

impl Global for Theme {}

impl Theme {
    /// Returns the global theme reference
    #[inline(always)]
    pub fn global(cx: &App) -> &Theme {
        cx.global::<Theme>()
    }

    /// Returns the global theme mutable reference
    ///
    /// Changes to fields the Base layer mirrors — the radius, the colors, the
    /// fonts — reach the scrollbar and resize handles only once
    /// [`Theme::sync_base`] runs.
    #[inline(always)]
    pub fn global_mut(cx: &mut App) -> &mut Theme {
        cx.global_mut::<Theme>()
    }

    /// Returns true if the theme is dark.
    #[inline(always)]
    pub fn is_dark(&self) -> bool {
        self.mode.is_dark()
    }

    pub fn ink(&self, alpha: f32) -> Hsla {
        paint::ink_for(self.is_dark(), alpha)
    }

    pub fn wash(&self, alpha: f32) -> Hsla {
        paint::wash_for(self.is_dark(), alpha)
    }

    pub fn press(&self, alpha: f32) -> Hsla {
        paint::press_for(alpha)
    }

    pub fn hairline(&self, alpha: f32) -> Hsla {
        paint::hairline_for(self.is_dark(), alpha)
    }

    pub fn scrim(&self, alpha_dark: f32) -> Hsla {
        paint::scrim_for(self.is_dark(), alpha_dark)
    }

    pub fn fallback_dark() -> Self {
        let mut t = Self::blank(ThemeMode::Dark);
        t.apply_fallback_roles(true);
        t
    }

    pub fn fallback_light() -> Self {
        let mut t = Self::blank(ThemeMode::Light);
        t.apply_fallback_roles(false);
        t
    }

    fn apply_fallback_roles(&mut self, dark: bool) {
        use paint::{grey, neutral, oklch};
        if dark {
            self.bg = grey(6);
            self.surface = grey(13);
            self.surface_raised = neutral(0.235);
            self.surface_raised_hover = neutral(0.29);
            self.surface_card = grey(0x0e);
            self.surface_dialog = grey(0x10);
            self.surface_overlay = grey(0x16);
            self.band = hsla(0.0, 0.0, 0.0, 0.16);
            self.input_bg = hsla(0.0, 0.0, 1.0, 0.03);
            self.text = neutral(0.922);
            self.text_muted = neutral(0.708);
            self.text_faint = neutral(0.556);
            self.text_dim = grey(0x98);
            self.border = hsla(0.0, 0.0, 1.0, 0.08);
            self.border_strong = hsla(0.0, 0.0, 1.0, 0.14);
            self.accent = neutral(0.673);
            self.accent_strong = neutral(0.922);
            self.on_accent = grey(0x0e);
            self.solid = neutral(0.922);
            self.on_solid = grey(0x0e);
            self.danger = oklch(0.704, 0.191, 22.216);
            self.danger_muted = oklch(0.808, 0.114, 19.571);
            self.danger_strong = oklch(0.58, 0.16, 25.0);
            self.warning = oklch(0.828, 0.189, 84.429);
            self.warning_muted = oklch(0.924, 0.12, 95.746);
            self.success = oklch(0.765, 0.177, 163.223);
            self.success_muted = oklch(0.845, 0.143, 164.978);
            self.busy = oklch(0.718, 0.202, 349.761);
            self.focus = self.accent;
            self.row_hover = self.wash(WASH_HOVER);
            self.row_selected = self.wash(WASH_SELECTED);
            self.table_header_bg = self.bg;
            self.table_row_hover = paint::flatten(self.wash(WASH_HOVER), self.bg);
            self.table_row_selected = self.accent.opacity(0.18);
            self.table_row_selected_border = self.accent;
            self.selection = hsla(0.66, 0.6, 0.55, 0.35);
            self.caret = self.text;
            self.cursor = hsla(0.0, 0.0, 1.0, 0.35);
        } else {
            self.bg = grey(0xff);
            self.surface = neutral(0.968);
            self.surface_raised = neutral(0.940);
            self.surface_raised_hover = neutral(0.900);
            self.surface_card = grey(0xff);
            self.surface_dialog = grey(0xff);
            self.surface_overlay = grey(0xff);
            self.band = hsla(0.0, 0.0, 0.0, 0.045);
            self.input_bg = grey(0xff);
            self.text = neutral(0.25);
            self.text_muted = neutral(0.439);
            self.text_faint = neutral(0.535);
            self.text_dim = neutral(0.50);
            self.border = hsla(0.0, 0.0, 0.0, 0.10);
            self.border_strong = hsla(0.0, 0.0, 0.0, 0.17);
            self.accent = neutral(0.511);
            self.accent_strong = neutral(0.205);
            self.on_accent = neutral(0.985);
            self.solid = neutral(0.205);
            self.on_solid = neutral(0.985);
            self.danger = oklch(0.577, 0.245, 27.325);
            self.danger_muted = oklch(0.505, 0.213, 27.518);
            self.danger_strong = oklch(0.51, 0.20, 25.0);
            self.warning = oklch(0.555, 0.163, 48.998);
            self.warning_muted = oklch(0.473, 0.137, 46.201);
            self.success = oklch(0.596, 0.145, 163.225);
            self.success_muted = oklch(0.508, 0.118, 165.612);
            self.busy = oklch(0.592, 0.249, 0.584);
            self.focus = self.accent;
            self.row_hover = self.wash(WASH_HOVER);
            self.row_selected = self.wash(WASH_SELECTED);
            self.table_header_bg = self.bg;
            self.table_row_hover = paint::flatten(self.wash(WASH_HOVER), self.bg);
            self.table_row_selected = self.accent.opacity(0.18);
            self.table_row_selected_border = self.accent;
            self.selection = hsla(0.66, 0.75, 0.62, 0.28);
            self.caret = self.text;
            self.cursor = hsla(0.0, 0.0, 0.0, 0.55);
        }
    }

    /// Returns the current theme name.
    pub fn theme_name(&self) -> &SharedString {
        if self.is_dark() {
            &self.dark_theme.name
        } else {
            &self.light_theme.name
        }
    }

    /// Sync the theme with the system appearance
    pub fn sync_system_appearance(window: Option<&mut Window>, cx: &mut App) {
        // Better use window.appearance() for avoid error on Linux.
        // https://github.com/longbridge/gpui-component/issues/104
        let appearance = window
            .as_ref()
            .map(|window| window.appearance())
            .unwrap_or_else(|| cx.window_appearance());

        Self::change(appearance, window, cx);
    }

    /// Sync the Scrollbar showing behavior with the system
    pub fn sync_scrollbar_appearance(cx: &mut App) {
        let mode = if cx.should_auto_hide_scrollbars() {
            ScrollbarMode::Scrolling
        } else {
            ScrollbarMode::Hover
        };
        Self::set_scrollbar_mode(mode, cx);
    }

    /// Changes the scrollbar display mode and synchronizes the Base projection.
    pub fn set_scrollbar_mode(mode: ScrollbarMode, cx: &mut App) {
        Theme::global_mut(cx).scrollbar_mode = mode;
        let base_theme = gpui_base::Theme::global_mut(cx);
        base_theme.scrollbar = base_theme
            .scrollbar
            .clone()
            .with_mode(mode)
            .with_motion(scrollbar_motion(mode));
    }

    /// Change the theme mode.
    pub fn change(mode: impl Into<ThemeMode>, window: Option<&mut Window>, cx: &mut App) {
        let mode = mode.into();
        if !cx.has_global::<Theme>() {
            let mut theme = Theme::default();
            theme.light_theme = ThemeRegistry::global(cx).default_light_theme().clone();
            theme.dark_theme = ThemeRegistry::global(cx).default_dark_theme().clone();
            cx.set_global(theme);
        }

        let theme = cx.global_mut::<Theme>();
        theme.mode = mode;
        if mode.is_dark() {
            theme.apply_config(&theme.dark_theme.clone());
        } else {
            theme.apply_config(&theme.light_theme.clone());
        }

        let base_theme = theme.base_theme();
        cx.set_global(base_theme);

        if let Some(window) = window {
            window.refresh();
        }
    }

    /// Recessed chrome surface, 4% darker than [`Self::bg`].
    #[inline]
    pub fn bg_sunken(&self) -> Hsla {
        mix_toward_black(self.bg, 0.04)
    }

    /// Selected-row surface, mixed 10% toward [`Self::accent`].
    #[inline]
    pub fn bg_active(&self) -> Hsla {
        mix(self.bg, self.accent, 0.10)
    }

    /// Soft accent-tinted surface, mixed 8% from background toward accent.
    #[inline]
    pub fn accent_soft(&self) -> Hsla {
        mix(self.bg, self.accent, 0.08)
    }

    /// This theme projected onto the Base layer, which owns the scrollbar and
    /// resize handles.
    fn base_theme(&self) -> gpui_base::Theme {
        gpui_base::Theme {
            tokens: gpui_base::SemanticThemeTokens::default(),
            scrollbar: gpui_base::ScrollbarTheme::new()
                .with_mode(self.scrollbar_mode)
                .with_motion(scrollbar_motion(self.scrollbar_mode))
                .with_styles(
                    gpui_base::ScrollbarStyles::default()
                        .track(|s| s.bg(self.transparent))
                        .track_hover(|s| s.bg(self.transparent))
                        .track_active(|s| s.bg(self.transparent).border_color(self.border))
                        .thumb(|s| s.bg(self.ink(0.35)).radius(self.radius))
                        .thumb_hover(|s| s.bg(self.ink(0.50)).radius(self.radius))
                        .thumb_active(|s| s.bg(self.ink(0.50)).radius(self.radius)),
                ),
            resizable: gpui_base::ResizableTheme {
                handle: self.border,
                active_handle: self.accent,
            },
        }
    }

    /// Push the current theme down to the Base layer.
    ///
    /// The Base layer holds its own copy of the theme — default semantic tokens
    /// plus the scrollbar and resize-handle styles — because it paints those
    /// without going through `gpui-component`. [`Theme::change`] refreshes that
    /// copy, but writing to the theme's public fields directly does not, so a
    /// scrollbar keeps painting with the radius and colors it was last given.
    ///
    /// Call this after mutating the theme through [`Theme::global_mut`]:
    ///
    /// ```ignore
    /// Theme::global_mut(cx).radius = px(0.);
    /// Theme::sync_base(cx);
    /// ```
    ///
    /// It rebuilds the Base theme from scratch, so any style written straight
    /// onto the Base global is replaced — the same thing [`Theme::change`]
    /// does.
    pub fn sync_base(cx: &mut App) {
        let base_theme = Theme::global(cx).base_theme();
        cx.set_global(base_theme);
    }

    /// Get the input background color.
    ///
    /// Returns the authored [`Self::input_bg`] role.
    #[inline]
    pub fn input_background(&self) -> Hsla {
        self.input_bg
    }

    /// Get the editor background color, if not set, use the input background color.
    #[inline]
    pub(crate) fn editor_background(&self) -> Hsla {
        self.highlight_theme
            .style
            .editor_background
            .unwrap_or_else(|| self.input_bg)
    }

    /// The radius of a semantic circle or pill. Square themes square these
    /// shapes too, while every non-zero preference retains a full curve.
    pub fn radius_full(&self) -> Pixels {
        if self.radius.is_zero() {
            px(0.)
        } else {
            RADIUS_FULL
        }
    }

    pub fn radius_tokens(&self) -> RadiusTokens {
        RadiusTokens {
            none: px(0.),
            sm: self.radius / 2.,
            md: self.radius,
            lg: self.radius_lg,
            xl: self.radius * 2.,
            full: self.radius_full(),
        }
    }

    pub fn spacing_tokens(&self) -> SpacingTokens {
        SpacingTokens::default()
    }

    pub fn typography_tokens(&self) -> TypographyTokens {
        let mut tokens = TypographyTokens::default();
        tokens.sans = self.font_family.clone();
        tokens.mono = self.mono_font_family.clone();
        tokens.md.size = self.font_size;
        tokens.mono_md.size = self.mono_font_size;
        tokens
    }

    pub fn shadow_tokens(&self) -> ShadowTokens {
        if self.shadow {
            ShadowTokens::elevations(self.transparent.alpha(0.18))
        } else {
            ShadowTokens::default()
        }
    }
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

#[cfg(test)]
mod theme_scale_tests {
    use gpui::{Hsla, px};

    use super::{RADIUS_FULL, Theme, mix, mix_toward_black};

    #[test]
    fn surface_helpers_use_pin_mix_amounts() {
        let mut theme = Theme::default();
        theme.bg = Hsla {
            h: 0.1,
            s: 0.2,
            l: 0.8,
            a: 1.0,
        };
        theme.accent = Hsla {
            h: 0.5,
            s: 0.6,
            l: 0.4,
            a: 1.0,
        };

        assert_eq!(theme.bg_sunken(), mix_toward_black(theme.bg, 0.04));
        assert_eq!(theme.bg_active(), mix(theme.bg, theme.accent, 0.10));
        assert_eq!(theme.accent_soft(), mix(theme.bg, theme.accent, 0.08));
        assert!((theme.bg_sunken().l - 0.76).abs() < f32::EPSILON);
    }

    #[test]
    fn square_themes_square_semantic_pills_and_circles() {
        let mut theme = Theme::default();
        assert_eq!(theme.radius_full(), RADIUS_FULL);
        assert_eq!(theme.radius_tokens().full, RADIUS_FULL);

        theme.radius = px(0.);
        assert_eq!(theme.radius_full(), px(0.));
        assert_eq!(theme.radius_tokens().full, px(0.));
    }

    #[test]
    fn base_projection_zeros_base_tokens() {
        let mut theme = Theme::default();
        assert_eq!(
            theme.base_theme().tokens,
            gpui_base::SemanticThemeTokens::default()
        );

        // Radius and colors are instance scrollbar / resize styles, not a
        // Base token snapshot.
        theme.radius = px(0.);
        assert_eq!(
            theme.base_theme().tokens,
            gpui_base::SemanticThemeTokens::default()
        );
    }

    #[test]
    fn disabled_legacy_shadows_project_to_empty_elevations() {
        let mut theme = Theme::default();
        theme.shadow = false;

        let shadows = theme.shadow_tokens();
        assert!(shadows.sm.is_empty());
        assert!(shadows.md.is_empty());
        assert!(shadows.lg.is_empty());
    }
}

impl Theme {
    /// Zeroed shell used by [`Self::fallback_dark`] / [`Self::fallback_light`]
    /// before they fill the 38 role fields.
    fn blank(mode: ThemeMode) -> Self {
        Theme {
            mode,
            transparent: Hsla::transparent_black(),
            font_family: ".SystemUIFont".into(),
            font_size: px(16.),
            mono_font_family: if cfg!(target_os = "macos") {
                // https://en.wikipedia.org/wiki/Menlo_(typeface)
                "Menlo".into()
            } else if cfg!(target_os = "windows") {
                "Consolas".into()
            } else {
                "DejaVu Sans Mono".into()
            },
            mono_font_size: px(13.),
            radius: px(6.),
            radius_lg: px(8.),
            shadow: true,
            focus_ring: true,
            scrollbar_mode: ScrollbarMode::default(),
            notification: NotificationSettings::default(),
            tile_grid_size: px(8.),
            tile_shadow: true,
            tile_radius: px(0.),
            list: ListSettings::default(),
            bg: Hsla::default(),
            surface: Hsla::default(),
            surface_raised: Hsla::default(),
            surface_raised_hover: Hsla::default(),
            surface_card: Hsla::default(),
            surface_dialog: Hsla::default(),
            surface_overlay: Hsla::default(),
            band: Hsla::default(),
            input_bg: Hsla::default(),
            text: Hsla::default(),
            text_muted: Hsla::default(),
            text_faint: Hsla::default(),
            text_dim: Hsla::default(),
            border: Hsla::default(),
            border_strong: Hsla::default(),
            accent: Hsla::default(),
            accent_strong: Hsla::default(),
            on_accent: Hsla::default(),
            solid: Hsla::default(),
            on_solid: Hsla::default(),
            danger: Hsla::default(),
            danger_muted: Hsla::default(),
            danger_strong: Hsla::default(),
            warning: Hsla::default(),
            warning_muted: Hsla::default(),
            success: Hsla::default(),
            success_muted: Hsla::default(),
            busy: Hsla::default(),
            focus: Hsla::default(),
            row_hover: Hsla::default(),
            row_selected: Hsla::default(),
            table_header_bg: Hsla::default(),
            table_row_hover: Hsla::default(),
            table_row_selected: Hsla::default(),
            table_row_selected_border: Hsla::default(),
            selection: Hsla::default(),
            caret: Hsla::default(),
            cursor: Hsla::default(),
            light_theme: Rc::new(ThemeConfig::default()),
            dark_theme: Rc::new(ThemeConfig::default()),
            highlight_theme: HighlightTheme::default_light(),
            sheet: SheetSettings::default(),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

impl ThemeMode {
    #[inline(always)]
    pub fn is_dark(&self) -> bool {
        matches!(self, Self::Dark)
    }

    /// Return lower_case theme name: `light`, `dark`.
    pub fn name(&self) -> &'static str {
        match self {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        }
    }
}

impl From<WindowAppearance> for ThemeMode {
    fn from(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::Dark,
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::Light,
        }
    }
}

#[cfg(test)]
mod base_theme_projection_tests {
    use super::*;
    use gpui::TestAppContext;

    #[gpui::test]
    fn base_theme_tracks_initialization_and_mode_changes(cx: &mut TestAppContext) {
        cx.update(|cx| {
            init(cx);
            assert_styled_projection(cx);

            Theme::change(ThemeMode::Dark, None, cx);
            assert_styled_projection(cx);

            Theme::set_scrollbar_mode(ScrollbarMode::Always, cx);
            assert_eq!(Theme::global(cx).scrollbar_mode, ScrollbarMode::Always);
            assert_eq!(
                gpui_base::Theme::global(cx).scrollbar.mode(),
                gpui_base::ScrollbarMode::Always
            );
            assert_styled_projection(cx);
        });
    }

    #[gpui::test]
    fn scrollbar_motion_is_owned_here_and_projected_onto_base(cx: &mut TestAppContext) {
        cx.update(|cx| {
            init(cx);

            // Base itself ships none of this timing.
            let bare = gpui_base::ScrollbarMotion::default();
            assert_eq!(bare.enter(), Duration::ZERO);
            assert_eq!(bare.exit(), Duration::ZERO);
            assert_eq!(bare.expand(), Duration::ZERO);

            Theme::set_scrollbar_mode(ScrollbarMode::Scrolling, cx);
            let motion = gpui_base::Theme::global(cx).scrollbar.motion();
            assert_eq!(motion.idle(), SCROLLBAR_IDLE);
            assert_eq!(motion.enter(), SCROLLBAR_ENTER);
            assert_eq!(motion.exit(), SCROLLBAR_EXIT);
            assert_eq!(motion.expand(), SCROLLBAR_EXPAND);
            assert_eq!(
                motion.entrance(),
                gpui_base::ScrollbarEntrance::Fade,
                "scroll-revealed scrollbars fade in without sliding"
            );

            Theme::set_scrollbar_mode(ScrollbarMode::Hover, cx);
            let motion = gpui_base::Theme::global(cx).scrollbar.motion();
            assert_eq!(motion.entrance(), gpui_base::ScrollbarEntrance::Fade);
            assert_eq!(
                motion.thumb_hover_entrance(),
                gpui_base::ScrollbarEntrance::SlideAndFade
            );
        });
    }

    fn assert_styled_projection(cx: &App) {
        let theme = Theme::global(cx);
        let base = gpui_base::Theme::global(cx);

        assert_eq!(base.tokens, gpui_base::SemanticThemeTokens::default());
        assert_eq!(base.scrollbar.mode(), theme.scrollbar_mode);
        assert_eq!(
            base.scrollbar.motion(),
            scrollbar_motion(theme.scrollbar_mode)
        );
        assert_eq!(base.resizable.handle, theme.border);
        assert_eq!(base.resizable.active_handle, theme.accent);
    }
}

#[cfg(test)]
mod role_tests {
    use super::*;
    use crate::theme::paint::contrast_ratio;

    #[test]
    fn fallback_text_clears_aa_on_bg_and_surface() {
        for t in [Theme::fallback_dark(), Theme::fallback_light()] {
            for (name, fg, floor) in [
                ("text", t.text, 4.5),
                ("text_muted", t.text_muted, 4.5),
                ("text_dim", t.text_dim, 4.5),
                ("text_faint", t.text_faint, 4.1),
            ] {
                let on_bg = contrast_ratio(fg, t.bg);
                let on_surface = contrast_ratio(fg, t.surface);
                assert!(on_bg >= floor, "{:?} {name} on bg {on_bg:.2}", t.mode);
                assert!(
                    on_surface >= floor,
                    "{:?} {name} on surface {on_surface:.2}",
                    t.mode
                );
            }
        }
    }

    #[test]
    fn fallback_solid_and_accent_plates_carry_labels() {
        for t in [Theme::fallback_dark(), Theme::fallback_light()] {
            assert!(contrast_ratio(t.on_solid, t.solid) >= 7.0);
            assert!(contrast_ratio(t.on_accent, t.accent_strong) >= 4.0);
        }
    }

    #[test]
    fn fallback_surface_order() {
        let d = Theme::fallback_dark();
        assert!(d.bg.l < d.surface.l);
        let l = Theme::fallback_light();
        assert!(l.surface.l < l.bg.l);
        assert!((l.bg.l - l.surface.l) > 0.015);
    }

    #[test]
    fn focus_and_table_roles_are_independent_interactions() {
        for t in [Theme::fallback_dark(), Theme::fallback_light()] {
            assert_ne!(t.focus, t.caret);
            assert_ne!(t.table_row_hover, t.table_row_selected);
            assert!(t.table_row_hover.a > 0.0);
            assert!(t.table_row_selected.a > 0.0);
        }
    }

    #[test]
    fn light_does_not_reuse_dark_400_warning() {
        let d = Theme::fallback_dark();
        let l = Theme::fallback_light();
        assert!(contrast_ratio(d.warning, l.bg) < 3.0);
        assert!(contrast_ratio(l.warning, l.bg) >= 3.0);
    }
}
