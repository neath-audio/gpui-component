use std::{rc::Rc, sync::Arc};

use gpui::{Background, BoxShadow, FontWeight, Hsla, SharedString, px};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::highlighter::{HighlightTheme, HighlightThemeStyle};

use super::color::{
    try_parse_background, try_parse_background_clamped, try_parse_color, try_parse_theme_color,
};
use super::{
    Colorize, SemanticThemeTokens, Theme, ThemeColor, ThemeMode, ThemeToken, ThemeTokens,
    ThemeTranslucency,
};

fn try_parse_theme_token(value: &str) -> anyhow::Result<ThemeToken> {
    Ok(ThemeToken::new(
        try_parse_theme_color(value)?,
        try_parse_background(value)?,
    ))
}

/// Represents a theme configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ThemeSet {
    /// The name of the theme set.
    pub name: SharedString,
    /// The author of the theme.
    pub author: Option<SharedString>,
    /// The URL of the theme.
    pub url: Option<SharedString>,
    /// The theme list of the theme set.
    #[serde(rename = "themes")]
    pub themes: Vec<ThemeConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ThemeConfig {
    /// Whether this theme is the default theme.
    pub is_default: bool,
    /// The name of the theme.
    pub name: SharedString,
    /// The mode of the theme, default is light.
    pub mode: ThemeMode,

    /// The base font size, default is 16.
    #[serde(rename = "font.size")]
    pub font_size: Option<f32>,
    /// The base font family, default is system font: `.SystemUIFont`.
    #[serde(rename = "font.family")]
    pub font_family: Option<SharedString>,
    /// The monospace font family, default is platform specific:
    /// - macOS: `Menlo`
    /// - Windows: `Consolas`
    /// - Linux: `DejaVu Sans Mono`
    #[serde(rename = "mono_font.family")]
    pub mono_font_family: Option<SharedString>,
    /// The monospace font size, default is 13.
    #[serde(rename = "mono_font.size")]
    pub mono_font_size: Option<f32>,

    /// The border radius for general elements, default is 6.
    #[serde(rename = "radius")]
    pub radius: Option<usize>,
    /// The border radius for large elements like Dialogs and Notifications, default is 8.
    #[serde(rename = "radius.lg")]
    pub radius_lg: Option<usize>,
    /// Set shadows in the theme, for example the Input and Button, default is true.
    #[serde(rename = "shadow")]
    pub shadow: Option<bool>,

    /// Opt-in controls for whole-window and local translucent materials.
    pub translucency: ThemeTranslucencyConfig,

    /// The colors of the theme.
    pub colors: ThemeConfigColors,
    /// The highlight theme, this part is combilbility with `style` section in Zed theme.
    ///
    /// https://github.com/zed-industries/zed/blob/f50041779dcfd7a76c8aec293361c60c53f02d51/assets/themes/ayu/ayu.json#L9
    pub highlight: Option<HighlightThemeStyle>,
}

/// Theme-authored translucency settings.
///
/// Transparency in a color never enables glass. Themes must explicitly set
/// [`Self::window`] before the platform window or local materials become translucent.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ThemeTranslucencyConfig {
    pub window: bool,
    #[schemars(range(min = 0.0, max = 64.0))]
    pub overlay_blur: f32,
    #[schemars(range(min = 0.0, max = 64.0))]
    pub panel_blur: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SemanticThemeConfig {
    pub colors: SemanticColorConfig,
    pub radius: SemanticRadiusConfig,
    pub spacing: SemanticSpacingConfig,
    pub typography: SemanticTypographyConfig,
    pub shadow: SemanticShadowConfig,
}

/// Standalone semantic theme configuration file.
///
/// This wrapper is intentionally separate from [`ThemeConfig`] so adding
/// semantic tokens does not change the legacy public struct shape.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SemanticThemeConfigFile {
    pub tokens: SemanticThemeConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SemanticColorConfig {
    pub background: Option<SharedString>,
    pub foreground: Option<SharedString>,
    pub surface: Option<SharedString>,
    pub surface_foreground: Option<SharedString>,
    pub primary: Option<SharedString>,
    pub primary_foreground: Option<SharedString>,
    pub secondary: Option<SharedString>,
    pub secondary_foreground: Option<SharedString>,
    pub muted: Option<SharedString>,
    pub muted_foreground: Option<SharedString>,
    pub accent: Option<SharedString>,
    pub accent_foreground: Option<SharedString>,
    pub destructive: Option<SharedString>,
    pub destructive_foreground: Option<SharedString>,
    pub border: Option<SharedString>,
    pub input: Option<SharedString>,
    pub ring: Option<SharedString>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SemanticRadiusConfig {
    pub none: Option<f32>,
    pub sm: Option<f32>,
    pub md: Option<f32>,
    pub lg: Option<f32>,
    pub xl: Option<f32>,
    pub full: Option<f32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SemanticSpacingConfig {
    pub xxs: Option<f32>,
    pub xs: Option<f32>,
    pub sm: Option<f32>,
    pub md: Option<f32>,
    pub lg: Option<f32>,
    pub xl: Option<f32>,
    pub xxl: Option<f32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SemanticTextStyleConfig {
    pub size: Option<f32>,
    pub line_height: Option<f32>,
    pub weight: Option<FontWeight>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SemanticTypographyConfig {
    pub sans: Option<SharedString>,
    pub mono: Option<SharedString>,
    pub xs: SemanticTextStyleConfig,
    pub sm: SemanticTextStyleConfig,
    pub md: SemanticTextStyleConfig,
    pub lg: SemanticTextStyleConfig,
    pub xl: SemanticTextStyleConfig,
    pub mono_md: SemanticTextStyleConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct SemanticShadowConfig {
    pub sm: Option<Vec<BoxShadow>>,
    pub md: Option<Vec<BoxShadow>>,
    pub lg: Option<Vec<BoxShadow>>,
}

impl SemanticThemeConfig {
    pub(crate) fn apply_to(&self, tokens: &mut SemanticThemeTokens) {
        macro_rules! apply_color {
            ($field:ident) => {
                if let Some(value) = &self.colors.$field
                    && let Ok(value) = try_parse_color(value)
                {
                    tokens.colors.$field = value;
                }
            };
        }
        apply_color!(background);
        apply_color!(foreground);
        apply_color!(surface);
        apply_color!(surface_foreground);
        apply_color!(primary);
        apply_color!(primary_foreground);
        apply_color!(secondary);
        apply_color!(secondary_foreground);
        apply_color!(muted);
        apply_color!(muted_foreground);
        apply_color!(accent);
        apply_color!(accent_foreground);
        apply_color!(destructive);
        apply_color!(destructive_foreground);
        apply_color!(border);
        apply_color!(input);
        apply_color!(ring);

        macro_rules! apply_pixels {
            ($config:expr, $tokens:expr, $($field:ident),+ $(,)?) => {
                $(if let Some(value) = $config.$field { $tokens.$field = px(value); })+
            };
        }
        apply_pixels!(self.radius, tokens.radius, none, sm, md, lg, xl, full);
        apply_pixels!(self.spacing, tokens.spacing, xxs, xs, sm, md, lg, xl, xxl);

        if let Some(value) = &self.typography.sans {
            tokens.typography.sans = value.clone();
        }
        if let Some(value) = &self.typography.mono {
            tokens.typography.mono = value.clone();
        }
        apply_text_style(&self.typography.xs, &mut tokens.typography.xs);
        apply_text_style(&self.typography.sm, &mut tokens.typography.sm);
        apply_text_style(&self.typography.md, &mut tokens.typography.md);
        apply_text_style(&self.typography.lg, &mut tokens.typography.lg);
        apply_text_style(&self.typography.xl, &mut tokens.typography.xl);
        apply_text_style(&self.typography.mono_md, &mut tokens.typography.mono_md);

        if let Some(value) = &self.shadow.sm {
            tokens.shadow.sm = value.clone();
        }
        if let Some(value) = &self.shadow.md {
            tokens.shadow.md = value.clone();
        }
        if let Some(value) = &self.shadow.lg {
            tokens.shadow.lg = value.clone();
        }
    }
}

fn apply_text_style(config: &SemanticTextStyleConfig, token: &mut gpui_base::TextStyleToken) {
    if let Some(value) = config.size {
        token.size = px(value);
    }
    if let Some(value) = config.line_height {
        token.line_height = px(value);
    }
    if let Some(value) = config.weight {
        token.weight = value;
    }
}

#[derive(Debug, Default, Clone, JsonSchema, Serialize, Deserialize)]
pub struct ThemeConfigColors {
    /// Used for accents such as hover background on MenuItem, ListItem, etc.
    #[serde(rename = "accent.background")]
    pub accent: Option<SharedString>,
    /// Used for accent text color.
    #[serde(rename = "accent.foreground")]
    pub accent_foreground: Option<SharedString>,
    /// Accordion background color.
    #[serde(rename = "accordion.background")]
    pub accordion: Option<SharedString>,
    /// Default background color.
    #[serde(rename = "background")]
    pub background: Option<SharedString>,
    /// Window background color.
    #[serde(rename = "window.background")]
    pub window_background: Option<SharedString>,
    /// Default border color
    #[serde(rename = "border")]
    pub border: Option<SharedString>,
    /// Default Button background color.
    #[serde(rename = "button.background")]
    pub button: Option<SharedString>,
    /// Default Button active background color.
    #[serde(rename = "button.active.background")]
    pub button_active: Option<SharedString>,
    /// Default Button text color.
    #[serde(rename = "button.foreground")]
    pub button_foreground: Option<SharedString>,
    /// Default Button hover background color.
    #[serde(rename = "button.hover.background")]
    pub button_hover: Option<SharedString>,
    /// Button danger background color, fallback to `danger`.
    #[serde(rename = "button.danger.background")]
    pub button_danger: Option<SharedString>,
    /// Button danger active background color, fallback to `danger_active`.
    #[serde(rename = "button.danger.active.background")]
    pub button_danger_active: Option<SharedString>,
    /// Button danger text color, fallback to `danger_foreground`.
    #[serde(rename = "button.danger.foreground")]
    pub button_danger_foreground: Option<SharedString>,
    /// Button danger hover background color, fallback to `danger_hover`.
    #[serde(rename = "button.danger.hover.background")]
    pub button_danger_hover: Option<SharedString>,
    /// Button info background color, fallback to `info`.
    #[serde(rename = "button.info.background")]
    pub button_info: Option<SharedString>,
    /// Button info active background color, fallback to `info_active`.
    #[serde(rename = "button.info.active.background")]
    pub button_info_active: Option<SharedString>,
    /// Button info text color, fallback to `info_foreground`.
    #[serde(rename = "button.info.foreground")]
    pub button_info_foreground: Option<SharedString>,
    /// Button info hover background color, fallback to `info_hover`.
    #[serde(rename = "button.info.hover.background")]
    pub button_info_hover: Option<SharedString>,
    /// Button primary background color, fallback to `primary`.
    #[serde(rename = "button.primary.background")]
    pub button_primary: Option<SharedString>,
    /// Button primary active background color, fallback to `primary_active`.
    #[serde(rename = "button.primary.active.background")]
    pub button_primary_active: Option<SharedString>,
    /// Button primary text color, fallback to `primary_foreground`.
    #[serde(rename = "button.primary.foreground")]
    pub button_primary_foreground: Option<SharedString>,
    /// Button primary hover background color, fallback to `primary_hover`.
    #[serde(rename = "button.primary.hover.background")]
    pub button_primary_hover: Option<SharedString>,
    /// Button secondary background color, fallback to `secondary`.
    #[serde(rename = "button.secondary.background")]
    pub button_secondary: Option<SharedString>,
    /// Button secondary active background color, fallback to `secondary_active`.
    #[serde(rename = "button.secondary.active.background")]
    pub button_secondary_active: Option<SharedString>,
    /// Button secondary text color, fallback to `secondary_foreground`.
    #[serde(rename = "button.secondary.foreground")]
    pub button_secondary_foreground: Option<SharedString>,
    /// Button secondary hover background color, fallback to `secondary_hover`.
    #[serde(rename = "button.secondary.hover.background")]
    pub button_secondary_hover: Option<SharedString>,
    /// Button success background color, fallback to `success`.
    #[serde(rename = "button.success.background")]
    pub button_success: Option<SharedString>,
    /// Button success active background color, fallback to `success_active`.
    #[serde(rename = "button.success.active.background")]
    pub button_success_active: Option<SharedString>,
    /// Button success text color, fallback to `success_foreground`.
    #[serde(rename = "button.success.foreground")]
    pub button_success_foreground: Option<SharedString>,
    /// Button success hover background color, fallback to `success_hover`.
    #[serde(rename = "button.success.hover.background")]
    pub button_success_hover: Option<SharedString>,
    /// Button warning background color, fallback to `warning`.
    #[serde(rename = "button.warning.background")]
    pub button_warning: Option<SharedString>,
    /// Button warning active background color, fallback to `warning_active`.
    #[serde(rename = "button.warning.active.background")]
    pub button_warning_active: Option<SharedString>,
    /// Button warning text color, fallback to `warning_foreground`.
    #[serde(rename = "button.warning.foreground")]
    pub button_warning_foreground: Option<SharedString>,
    /// Button warning hover background color, fallback to `warning_hover`.
    #[serde(rename = "button.warning.hover.background")]
    pub button_warning_hover: Option<SharedString>,
    /// Background color for GroupBox.
    #[serde(rename = "group_box.background")]
    pub group_box: Option<SharedString>,
    /// Text color for GroupBox.
    #[serde(rename = "group_box.foreground")]
    pub group_box_foreground: Option<SharedString>,
    /// Title text color for GroupBox.
    #[serde(rename = "group_box.title.foreground")]
    pub group_box_title_foreground: Option<SharedString>,
    /// Input caret color (Blinking cursor).
    #[serde(rename = "caret")]
    pub caret: Option<SharedString>,
    /// Chart 1 color.
    #[serde(rename = "chart.1")]
    pub chart_1: Option<SharedString>,
    /// Chart 2 color.
    #[serde(rename = "chart.2")]
    pub chart_2: Option<SharedString>,
    /// Chart 3 color.
    #[serde(rename = "chart.3")]
    pub chart_3: Option<SharedString>,
    /// Chart 4 color.
    #[serde(rename = "chart.4")]
    pub chart_4: Option<SharedString>,
    /// Chart 5 color.
    #[serde(rename = "chart.5")]
    pub chart_5: Option<SharedString>,
    /// Bullish color for candlestick charts (upward price movement).
    #[serde(rename = "chart_bullish")]
    pub chart_bullish: Option<SharedString>,
    /// Bearish color for candlestick charts (downward price movement).
    #[serde(rename = "chart_bearish")]
    pub chart_bearish: Option<SharedString>,
    /// Danger background color.
    #[serde(rename = "danger.background")]
    pub danger: Option<SharedString>,
    /// Danger active background color.
    #[serde(rename = "danger.active.background")]
    pub danger_active: Option<SharedString>,
    /// Danger text color.
    #[serde(rename = "danger.foreground")]
    pub danger_foreground: Option<SharedString>,
    /// Danger hover background color.
    #[serde(rename = "danger.hover.background")]
    pub danger_hover: Option<SharedString>,
    /// Description List label background color.
    #[serde(rename = "description_list.label.background")]
    pub description_list_label: Option<SharedString>,
    /// Description List label foreground color.
    #[serde(rename = "description_list.label.foreground")]
    pub description_list_label_foreground: Option<SharedString>,
    /// Drag border color.
    #[serde(rename = "drag.border")]
    pub drag_border: Option<SharedString>,
    /// Drop target background color.
    #[serde(rename = "drop_target.background")]
    pub drop_target: Option<SharedString>,
    /// Default text color.
    #[serde(rename = "foreground")]
    pub foreground: Option<SharedString>,
    /// Info background color.
    #[serde(rename = "info.background")]
    pub info: Option<SharedString>,
    /// Info active background color.
    #[serde(rename = "info.active.background")]
    pub info_active: Option<SharedString>,
    /// Info text color.
    #[serde(rename = "info.foreground")]
    pub info_foreground: Option<SharedString>,
    /// Info hover background color.
    #[serde(rename = "info.hover.background")]
    pub info_hover: Option<SharedString>,
    /// Border color for inputs such as Input, Select, etc.
    #[serde(rename = "input.border")]
    pub input: Option<SharedString>,
    /// Link text color.
    #[serde(rename = "link")]
    pub link: Option<SharedString>,
    /// Active link text color.
    #[serde(rename = "link.active")]
    pub link_active: Option<SharedString>,
    /// Hover link text color.
    #[serde(rename = "link.hover")]
    pub link_hover: Option<SharedString>,
    /// Background color for List and ListItem.
    #[serde(rename = "list.background")]
    pub list: Option<SharedString>,
    /// Background color for active ListItem.
    #[serde(rename = "list.active.background")]
    pub list_active: Option<SharedString>,
    /// Border color for active ListItem.
    #[serde(rename = "list.active.border")]
    pub list_active_border: Option<SharedString>,
    /// Stripe background color for even ListItem.
    #[serde(rename = "list.even.background")]
    pub list_even: Option<SharedString>,
    /// Background color for List header.
    #[serde(rename = "list.head.background")]
    pub list_head: Option<SharedString>,
    /// Hover background color for ListItem.
    #[serde(rename = "list.hover.background")]
    pub list_hover: Option<SharedString>,
    /// Muted backgrounds such as Skeleton and Switch.
    #[serde(rename = "muted.background")]
    pub muted: Option<SharedString>,
    /// Muted text color, as used in disabled text.
    #[serde(rename = "muted.foreground")]
    pub muted_foreground: Option<SharedString>,
    /// Background color for Popover.
    #[serde(rename = "popover.background")]
    pub popover: Option<SharedString>,
    /// Text color for Popover.
    #[serde(rename = "popover.foreground")]
    pub popover_foreground: Option<SharedString>,
    /// Primary background color.
    #[serde(rename = "primary.background")]
    pub primary: Option<SharedString>,
    /// Active primary background color.
    #[serde(rename = "primary.active.background")]
    pub primary_active: Option<SharedString>,
    /// Primary text color.
    #[serde(rename = "primary.foreground")]
    pub primary_foreground: Option<SharedString>,
    /// Hover primary background color.
    #[serde(rename = "primary.hover.background")]
    pub primary_hover: Option<SharedString>,
    /// Progress bar background color.
    #[serde(rename = "progress.bar.background")]
    pub progress_bar: Option<SharedString>,
    /// Used for focus ring.
    #[serde(rename = "ring")]
    pub ring: Option<SharedString>,
    /// Scrollbar background color.
    #[serde(rename = "scrollbar.background")]
    pub scrollbar: Option<SharedString>,
    /// Scrollbar thumb background color.
    #[serde(rename = "scrollbar.thumb.background")]
    pub scrollbar_thumb: Option<SharedString>,
    /// Scrollbar thumb hover background color.
    #[serde(rename = "scrollbar.thumb.hover.background")]
    pub scrollbar_thumb_hover: Option<SharedString>,
    /// Secondary background color.
    #[serde(rename = "secondary.background")]
    pub secondary: Option<SharedString>,
    /// Active secondary background color.
    #[serde(rename = "secondary.active.background")]
    pub secondary_active: Option<SharedString>,
    /// Secondary text color, used for secondary Button text color or secondary text.
    #[serde(rename = "secondary.foreground")]
    pub secondary_foreground: Option<SharedString>,
    /// Hover secondary background color.
    #[serde(rename = "secondary.hover.background")]
    pub secondary_hover: Option<SharedString>,
    /// Input selection background color.
    #[serde(rename = "selection.background")]
    pub selection: Option<SharedString>,
    /// Sidebar background color.
    #[serde(rename = "sidebar.background")]
    pub sidebar: Option<SharedString>,
    /// Sidebar accent background color.
    #[serde(rename = "sidebar.accent.background")]
    pub sidebar_accent: Option<SharedString>,
    /// Sidebar accent text color.
    #[serde(rename = "sidebar.accent.foreground")]
    pub sidebar_accent_foreground: Option<SharedString>,
    /// Sidebar border color.
    #[serde(rename = "sidebar.border")]
    pub sidebar_border: Option<SharedString>,
    /// Sidebar text color.
    #[serde(rename = "sidebar.foreground")]
    pub sidebar_foreground: Option<SharedString>,
    /// Sidebar primary background color.
    #[serde(rename = "sidebar.primary.background")]
    pub sidebar_primary: Option<SharedString>,
    /// Sidebar primary text color.
    #[serde(rename = "sidebar.primary.foreground")]
    pub sidebar_primary_foreground: Option<SharedString>,
    /// Skeleton background color.
    #[serde(rename = "skeleton.background")]
    pub skeleton: Option<SharedString>,
    /// Slider bar background color.
    #[serde(rename = "slider.background")]
    pub slider_bar: Option<SharedString>,
    /// Slider thumb background color.
    #[serde(rename = "slider.thumb.background")]
    pub slider_thumb: Option<SharedString>,
    /// Success background color.
    #[serde(rename = "success.background")]
    pub success: Option<SharedString>,
    /// Success text color.
    #[serde(rename = "success.foreground")]
    pub success_foreground: Option<SharedString>,
    /// Success hover background color.
    #[serde(rename = "success.hover.background")]
    pub success_hover: Option<SharedString>,
    /// Success active background color.
    #[serde(rename = "success.active.background")]
    pub success_active: Option<SharedString>,
    /// Switch background color.
    #[serde(rename = "switch.background")]
    pub switch: Option<SharedString>,
    /// Switch thumb background color.
    #[serde(rename = "switch.thumb.background")]
    pub switch_thumb: Option<SharedString>,
    /// Tab background color.
    #[serde(rename = "tab.background")]
    pub tab: Option<SharedString>,
    /// Tab active background color.
    #[serde(rename = "tab.active.background")]
    pub tab_active: Option<SharedString>,
    /// Tab active text color.
    #[serde(rename = "tab.active.foreground")]
    pub tab_active_foreground: Option<SharedString>,
    /// TabBar background color.
    #[serde(rename = "tab_bar.background")]
    pub tab_bar: Option<SharedString>,
    /// TabBar segmented background color.
    #[serde(rename = "tab_bar.segmented.background")]
    pub tab_bar_segmented: Option<SharedString>,
    /// Tab text color.
    #[serde(rename = "tab.foreground")]
    pub tab_foreground: Option<SharedString>,
    /// Table background color.
    #[serde(rename = "table.background")]
    pub table: Option<SharedString>,
    /// Table active item background color.
    #[serde(rename = "table.active.background")]
    pub table_active: Option<SharedString>,
    /// Table active item border color.
    #[serde(rename = "table.active.border")]
    pub table_active_border: Option<SharedString>,
    /// Stripe background color for even TableRow.
    #[serde(rename = "table.even.background")]
    pub table_even: Option<SharedString>,
    /// Table header background color.
    #[serde(rename = "table.head.background")]
    pub table_head: Option<SharedString>,
    /// Table header text color.
    #[serde(rename = "table.head.foreground")]
    pub table_head_foreground: Option<SharedString>,
    /// Table footer background color.
    #[serde(rename = "table.foot.background")]
    pub table_foot: Option<SharedString>,
    /// Table footer text color.
    #[serde(rename = "table.foot.foreground")]
    pub table_foot_foreground: Option<SharedString>,
    /// Table item hover background color.
    #[serde(rename = "table.hover.background")]
    pub table_hover: Option<SharedString>,
    /// Table row border color.
    #[serde(rename = "table.row.border")]
    pub table_row_border: Option<SharedString>,
    /// TitleBar background color, use for Window title bar.
    #[serde(rename = "title_bar.background")]
    pub title_bar: Option<SharedString>,
    /// TitleBar border color.
    #[serde(rename = "title_bar.border")]
    pub title_bar_border: Option<SharedString>,
    /// StatusBar background color, use for the bottom status bar.
    #[serde(rename = "status_bar.background")]
    pub status_bar: Option<SharedString>,
    /// StatusBar border color.
    #[serde(rename = "status_bar.border")]
    pub status_bar_border: Option<SharedString>,
    /// Background color for Tiles.
    #[serde(rename = "tiles.background")]
    pub tiles: Option<SharedString>,
    /// Warning background color.
    #[serde(rename = "warning.background")]
    pub warning: Option<SharedString>,
    /// Warning active background color.
    #[serde(rename = "warning.active.background")]
    pub warning_active: Option<SharedString>,
    /// Warning hover background color.
    #[serde(rename = "warning.hover.background")]
    pub warning_hover: Option<SharedString>,
    /// Warning foreground color.
    #[serde(rename = "warning.foreground")]
    pub warning_foreground: Option<SharedString>,
    /// Overlay background color.
    #[serde(rename = "overlay")]
    pub overlay: Option<SharedString>,
    /// Window border color.
    ///
    /// # Platform specific:
    ///
    /// This is only works on Linux, other platforms we can't change the window border color.
    #[serde(rename = "window.border")]
    pub window_border: Option<SharedString>,

    /// Base blue color.
    #[serde(rename = "base.blue")]
    blue: Option<String>,
    /// Base light blue color.
    #[serde(rename = "base.blue.light")]
    blue_light: Option<String>,
    /// Base cyan color.
    #[serde(rename = "base.cyan")]
    cyan: Option<String>,
    /// Base light cyan color.
    #[serde(rename = "base.cyan.light")]
    cyan_light: Option<String>,
    /// Base green color.
    #[serde(rename = "base.green")]
    green: Option<String>,
    /// Base light green color.
    #[serde(rename = "base.green.light")]
    green_light: Option<String>,
    /// Base magenta color.
    #[serde(rename = "base.magenta")]
    magenta: Option<String>,
    /// Base light magenta color.
    #[serde(rename = "base.magenta.light")]
    magenta_light: Option<String>,
    /// Base red color.
    #[serde(rename = "base.red")]
    red: Option<String>,
    /// Base light red color.
    #[serde(rename = "base.red.light")]
    red_light: Option<String>,
    /// Base yellow color.
    #[serde(rename = "base.yellow")]
    yellow: Option<String>,
    /// Base light yellow color.
    #[serde(rename = "base.yellow.light")]
    yellow_light: Option<String>,

    /// Strong border color for emphasized outlines and drop zones.
    #[serde(rename = "border.strong")]
    pub border_strong: Option<SharedString>,
    /// Card background color.
    #[serde(rename = "card.background")]
    pub card: Option<SharedString>,
    /// Card foreground color.
    #[serde(rename = "card.foreground")]
    pub card_foreground: Option<SharedString>,
    /// Card border color.
    #[serde(rename = "card.border")]
    pub card_border: Option<SharedString>,
    /// Card hover background color.
    #[serde(rename = "card.hover.background")]
    pub card_hover: Option<SharedString>,
    /// Card active background color.
    #[serde(rename = "card.active.background")]
    pub card_active: Option<SharedString>,
    /// Selected card background color.
    #[serde(rename = "card.selected.background")]
    pub card_selected: Option<SharedString>,
    /// Selected card border color.
    #[serde(rename = "card.selected.border")]
    pub card_selected_border: Option<SharedString>,
    /// Checked switch track background color.
    #[serde(rename = "switch.checked.background")]
    pub switch_checked: Option<SharedString>,
    /// Checked switch thumb background color.
    #[serde(rename = "switch.thumb.checked.background")]
    pub switch_thumb_checked: Option<SharedString>,
    /// Transport strip background color.
    #[serde(rename = "transport.background")]
    pub transport: Option<SharedString>,
    /// Transport strip border color.
    #[serde(rename = "transport.border")]
    pub transport_border: Option<SharedString>,
    /// Knob track background color.
    #[serde(rename = "knob.background")]
    pub knob: Option<SharedString>,
    /// Knob value background color.
    #[serde(rename = "knob.value.background")]
    pub knob_value: Option<SharedString>,
    /// Knob pointer foreground color.
    #[serde(rename = "knob.foreground")]
    pub knob_foreground: Option<SharedString>,
    /// Low similarity score color.
    #[serde(rename = "similarity.low")]
    pub similarity_low: Option<SharedString>,
    /// Medium similarity score color.
    #[serde(rename = "similarity.medium")]
    pub similarity_medium: Option<SharedString>,
    /// High similarity score color.
    #[serde(rename = "similarity.high")]
    pub similarity_high: Option<SharedString>,
    /// Meter fill color.
    #[serde(rename = "meter.fill")]
    pub meter_fill: Option<SharedString>,
    /// Meter held-peak color.
    #[serde(rename = "meter.peak")]
    pub meter_peak: Option<SharedString>,
    /// Meter track color.
    #[serde(rename = "meter.track")]
    pub meter_track: Option<SharedString>,
    /// Meter clipping indicator color.
    #[serde(rename = "meter.clip")]
    pub meter_clip: Option<SharedString>,
    /// Waveform canvas background color.
    #[serde(rename = "waveform.background")]
    pub waveform: Option<SharedString>,
    /// Compact waveform thumbnail fill color.
    #[serde(rename = "waveform.thumbnail.fill")]
    pub waveform_thumbnail_fill: Option<SharedString>,
    /// Main waveform fill color.
    #[serde(rename = "waveform.fill")]
    pub waveform_fill: Option<SharedString>,
    /// Waveform fill inside a time selection.
    #[serde(rename = "waveform.time_selection.foreground")]
    pub waveform_time_selection_foreground: Option<SharedString>,
    /// Time-selection band background color.
    #[serde(rename = "waveform.time_selection.background")]
    pub waveform_time_selection: Option<SharedString>,
    /// Time-selection border color.
    #[serde(rename = "waveform.time_selection.border")]
    pub waveform_time_selection_border: Option<SharedString>,
    /// Active time-selection border color.
    #[serde(rename = "waveform.time_selection.active.border")]
    pub waveform_time_selection_active_border: Option<SharedString>,
    /// Waveform fill over a visual overlay.
    #[serde(rename = "waveform.overlay.fill")]
    pub waveform_overlay_fill: Option<SharedString>,
    /// Waveform overlay zero-line color.
    #[serde(rename = "waveform.overlay.zero_line")]
    pub waveform_overlay_zero_line: Option<SharedString>,
    /// Selected waveform fill over a visual overlay.
    #[serde(rename = "waveform.overlay.time_selection.foreground")]
    pub waveform_overlay_time_selection_foreground: Option<SharedString>,
    /// Waveform zero-line color.
    #[serde(rename = "waveform.zero_line")]
    pub waveform_zero_line: Option<SharedString>,
    /// Waveform fade-control color.
    #[serde(rename = "waveform.fade_control")]
    pub waveform_fade_control: Option<SharedString>,
    /// Waveform playhead color.
    #[serde(rename = "waveform.playhead")]
    pub waveform_playhead: Option<SharedString>,
    /// Waveform ruler background color.
    #[serde(rename = "waveform.ruler.background")]
    pub waveform_ruler: Option<SharedString>,
    /// Waveform ruler foreground color.
    #[serde(rename = "waveform.ruler.foreground")]
    pub waveform_ruler_foreground: Option<SharedString>,
    /// Waveform channel-label background color.
    #[serde(rename = "waveform.channel_label.background")]
    pub waveform_channel_label: Option<SharedString>,
    /// Waveform channel-label foreground color.
    #[serde(rename = "waveform.channel_label.foreground")]
    pub waveform_channel_label_foreground: Option<SharedString>,
    /// Waveform marker background color.
    #[serde(rename = "waveform.marker.background")]
    pub waveform_marker: Option<SharedString>,
    /// Waveform marker foreground color.
    #[serde(rename = "waveform.marker.foreground")]
    pub waveform_marker_foreground: Option<SharedString>,
    /// Active waveform marker color.
    #[serde(rename = "waveform.marker.active")]
    pub waveform_marker_active: Option<SharedString>,
    /// Waveform segment background color.
    #[serde(rename = "waveform.segment.background")]
    pub waveform_segment: Option<SharedString>,
    /// Active waveform segment color.
    #[serde(rename = "waveform.segment.active")]
    pub waveform_segment_active: Option<SharedString>,
}

/// Canonical schema metadata paired with one resolved representative color.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedThemeColorProperty {
    /// Persisted JSON key from [`ThemeConfigColors`]' generated schema.
    pub key: SharedString,
    /// Human-readable description from the generated schema.
    pub description: SharedString,
    /// Solid representative color for swatches and color editors.
    pub color: Hsla,
}

fn waveform_tint(
    background: Hsla,
    foreground: Hsla,
    selection: Hsla,
    saturation: f32,
    contrast: f32,
) -> Hsla {
    Hsla {
        h: selection.h,
        s: saturation,
        l: background.l + (foreground.l - background.l) * contrast,
        a: 1.,
    }
}

fn mix_hsla(left: Hsla, right: Hsla, amount: f32) -> Hsla {
    let mut hue_delta = right.h - left.h;
    if hue_delta > 0.5 {
        hue_delta -= 1.;
    } else if hue_delta < -0.5 {
        hue_delta += 1.;
    }

    Hsla {
        h: (left.h + hue_delta * amount).rem_euclid(1.),
        s: left.s + (right.s - left.s) * amount,
        l: left.l + (right.l - left.l) * amount,
        a: left.a + (right.a - left.a) * amount,
    }
}

fn selected_overlay_waveform(mut color: Hsla) -> Hsla {
    color.l += 0.18 * (1. - color.l);
    color.s = (color.s * 1.08).min(1.);
    color
}

impl ThemeColor {
    /// Create a new `ThemeColor` from a `ThemeConfig`.
    pub(crate) fn apply_config(
        &mut self,
        config: &ThemeConfig,
        default_theme: &ThemeColor,
    ) -> ThemeTokens {
        let colors = config.colors.clone();
        let default_tokens = ThemeTokens::from(default_theme);
        let mut tokens = default_tokens;

        macro_rules! apply_color {
            ($config_field:ident) => {
                if let Some(value) = &colors.$config_field {
                    self.$config_field =
                        try_parse_color(value).unwrap_or(default_theme.$config_field);
                } else {
                    self.$config_field = default_theme.$config_field;
                }
                tokens.$config_field = self.$config_field.into();
            };
            // With fallback
            ($config_field:ident, fallback = $fallback:expr) => {
                let fallback: gpui::Hsla = ($fallback).into();
                if let Some(value) = &colors.$config_field {
                    self.$config_field = try_parse_color(value).unwrap_or(fallback);
                } else {
                    self.$config_field = fallback;
                }
                tokens.$config_field = self.$config_field.into();
            };
        }

        macro_rules! apply_background_color {
            ($config_field:ident) => {
                let token = if let Some(value) = &colors.$config_field {
                    if let Ok(token) = try_parse_theme_token(&value) {
                        token
                    } else {
                        default_tokens.$config_field
                    }
                } else {
                    default_tokens.$config_field
                };
                self.$config_field = token.color;
                tokens.$config_field = token;
            };
            ($config_field:ident, fallback = $fallback:expr) => {
                let fallback: ThemeToken = ($fallback).into();
                let token = if let Some(value) = &colors.$config_field {
                    if let Ok(token) = try_parse_theme_token(&value) {
                        token
                    } else {
                        fallback
                    }
                } else {
                    fallback
                };
                self.$config_field = token.color;
                tokens.$config_field = token;
            };
        }

        apply_background_color!(background);
        apply_background_color!(window_background, fallback = tokens.background);

        // Base colors for fallback
        apply_color!(red);
        apply_color!(
            red_light,
            fallback = self.background.blend(self.red.opacity(0.8))
        );
        apply_color!(green);
        apply_color!(
            green_light,
            fallback = self.background.blend(self.green.opacity(0.8))
        );
        apply_color!(blue);
        apply_color!(
            blue_light,
            fallback = self.background.blend(self.blue.opacity(0.8))
        );
        apply_color!(magenta);
        apply_color!(
            magenta_light,
            fallback = self.background.blend(self.magenta.opacity(0.8))
        );
        apply_color!(yellow);
        apply_color!(
            yellow_light,
            fallback = self.background.blend(self.yellow.opacity(0.8))
        );
        apply_color!(cyan);
        apply_color!(
            cyan_light,
            fallback = self.background.blend(self.cyan.opacity(0.8))
        );

        apply_color!(border);
        apply_color!(foreground);
        apply_color!(input, fallback = self.border);
        apply_background_color!(muted);
        apply_color!(
            muted_foreground,
            fallback = self.muted.blend(self.foreground.opacity(0.7))
        );

        // Button colors
        let active_darken = if config.mode.is_dark() { 0.2 } else { 0.1 };
        let hover_opacity = 0.9;
        let transparent = gpui::transparent_black();
        let button_background = if config.mode.is_dark() {
            self.input.mix_oklab(transparent, 0.3)
        } else {
            self.background
        };
        apply_background_color!(button, fallback = button_background);
        apply_color!(button_foreground, fallback = self.foreground);
        apply_background_color!(
            button_hover,
            fallback = self.input.mix_oklab(transparent, 0.5)
        );
        apply_background_color!(
            button_active,
            fallback = self.input.mix_oklab(transparent, 0.7)
        );
        apply_background_color!(primary);
        apply_color!(primary_foreground, fallback = self.foreground);
        apply_background_color!(
            primary_hover,
            fallback = self.background.blend(self.primary.opacity(hover_opacity))
        );
        apply_background_color!(
            primary_active,
            fallback = self.primary.darken(active_darken)
        );
        apply_background_color!(button_primary, fallback = tokens.primary);
        apply_color!(
            button_primary_foreground,
            fallback = self.primary_foreground
        );
        apply_background_color!(button_primary_hover, fallback = tokens.primary_hover);
        apply_background_color!(button_primary_active, fallback = tokens.primary_active);
        apply_background_color!(secondary);
        apply_color!(secondary_foreground, fallback = self.foreground);
        apply_background_color!(
            secondary_hover,
            fallback = self.background.blend(self.secondary.opacity(hover_opacity))
        );
        apply_background_color!(
            secondary_active,
            fallback = self.secondary.darken(active_darken)
        );
        apply_background_color!(button_secondary, fallback = tokens.secondary);
        apply_color!(
            button_secondary_foreground,
            fallback = self.secondary_foreground
        );
        apply_background_color!(button_secondary_hover, fallback = tokens.secondary_hover);
        apply_background_color!(button_secondary_active, fallback = tokens.secondary_active);
        apply_background_color!(success, fallback = self.green);
        apply_color!(success_foreground, fallback = self.primary_foreground);
        apply_background_color!(
            success_hover,
            fallback = self.background.blend(self.success.opacity(hover_opacity))
        );
        apply_background_color!(
            success_active,
            fallback = self.success.darken(active_darken)
        );
        apply_background_color!(
            button_success,
            fallback = self.success.mix_oklab(transparent, 0.2)
        );
        apply_color!(button_success_foreground, fallback = self.success);
        apply_background_color!(
            button_success_hover,
            fallback = self.success.mix_oklab(transparent, 0.3)
        );
        apply_background_color!(
            button_success_active,
            fallback = self.success.mix_oklab(transparent, 0.4)
        );
        apply_background_color!(info, fallback = self.cyan);
        apply_color!(info_foreground, fallback = self.primary_foreground);
        apply_background_color!(
            info_hover,
            fallback = self.background.blend(self.info.opacity(hover_opacity))
        );
        apply_background_color!(info_active, fallback = self.info.darken(active_darken));
        apply_background_color!(
            button_info,
            fallback = self.info.mix_oklab(transparent, 0.2)
        );
        apply_color!(button_info_foreground, fallback = self.info);
        apply_background_color!(
            button_info_hover,
            fallback = self.info.mix_oklab(transparent, 0.3)
        );
        apply_background_color!(
            button_info_active,
            fallback = self.info.mix_oklab(transparent, 0.4)
        );
        apply_background_color!(warning, fallback = self.yellow);
        apply_color!(warning_foreground, fallback = self.primary_foreground);
        apply_background_color!(
            warning_hover,
            fallback = self.background.blend(self.warning.opacity(0.9))
        );
        apply_background_color!(
            warning_active,
            fallback = self.background.blend(self.warning.darken(active_darken))
        );
        apply_background_color!(
            button_warning,
            fallback = self.warning.mix_oklab(transparent, 0.2)
        );
        apply_color!(button_warning_foreground, fallback = self.warning);
        apply_background_color!(
            button_warning_hover,
            fallback = self.warning.mix_oklab(transparent, 0.3)
        );
        apply_background_color!(
            button_warning_active,
            fallback = self.warning.mix_oklab(transparent, 0.4)
        );

        // Other colors
        apply_background_color!(accent, fallback = tokens.secondary);
        apply_color!(accent_foreground, fallback = self.foreground);
        apply_background_color!(accordion, fallback = tokens.background);
        apply_background_color!(
            group_box,
            fallback = self
                .background
                .blend(
                    self.secondary
                        .opacity(if config.mode.is_dark() { 0.3 } else { 0.4 })
                )
        );
        apply_color!(group_box_foreground, fallback = self.foreground);
        apply_color!(group_box_title_foreground, fallback = self.muted_foreground);
        apply_color!(caret, fallback = self.primary);
        apply_color!(chart_1, fallback = self.blue.lighten(0.4));
        apply_color!(chart_2, fallback = self.blue.lighten(0.2));
        apply_color!(chart_3, fallback = self.blue);
        apply_color!(chart_4, fallback = self.blue.darken(0.2));
        apply_color!(chart_5, fallback = self.blue.darken(0.4));
        apply_color!(chart_bullish, fallback = self.green);
        apply_color!(chart_bearish, fallback = self.red);
        apply_background_color!(danger, fallback = self.red);
        apply_background_color!(danger_active, fallback = self.danger.darken(active_darken));
        apply_color!(danger_foreground, fallback = self.primary_foreground);
        apply_background_color!(
            danger_hover,
            fallback = self.background.blend(self.danger.opacity(0.9))
        );
        apply_background_color!(
            button_danger,
            fallback = self.danger.mix_oklab(transparent, 0.2)
        );
        apply_color!(button_danger_foreground, fallback = self.danger);
        apply_background_color!(
            button_danger_hover,
            fallback = self.danger.mix_oklab(transparent, 0.3)
        );
        apply_background_color!(
            button_danger_active,
            fallback = self.danger.mix_oklab(transparent, 0.4)
        );
        apply_background_color!(
            description_list_label,
            fallback = self.background.blend(self.border.opacity(0.2))
        );
        apply_color!(
            description_list_label_foreground,
            fallback = self.muted_foreground
        );
        apply_color!(drag_border, fallback = self.primary.opacity(0.65));
        apply_background_color!(drop_target, fallback = self.primary.opacity(0.2));
        apply_color!(link, fallback = self.primary);
        apply_color!(link_active, fallback = self.link);
        apply_color!(link_hover, fallback = self.link);
        apply_background_color!(list, fallback = tokens.background);
        apply_background_color!(
            list_active,
            fallback = self.background.blend(self.primary.opacity(0.1))
        );
        apply_color!(
            list_active_border,
            fallback = self.background.blend(self.primary.opacity(0.6))
        );
        apply_background_color!(list_even, fallback = tokens.list);
        apply_background_color!(list_head, fallback = tokens.list);
        apply_background_color!(list_hover, fallback = self.accent.opacity(0.6));
        apply_background_color!(popover, fallback = tokens.background);
        apply_color!(popover_foreground, fallback = self.foreground);
        apply_background_color!(progress_bar, fallback = tokens.primary);
        apply_color!(ring, fallback = self.blue);
        apply_background_color!(scrollbar, fallback = tokens.background);
        apply_background_color!(scrollbar_thumb, fallback = tokens.accent);
        apply_background_color!(scrollbar_thumb_hover, fallback = tokens.scrollbar_thumb);
        apply_background_color!(selection, fallback = tokens.primary);
        apply_background_color!(
            sidebar,
            fallback = self.background.blend(self.border.opacity(0.15))
        );
        apply_background_color!(sidebar_accent, fallback = tokens.accent);
        apply_color!(sidebar_accent_foreground, fallback = self.accent_foreground);
        apply_color!(sidebar_border, fallback = self.border);
        apply_color!(sidebar_foreground, fallback = self.foreground);
        apply_background_color!(sidebar_primary, fallback = tokens.primary);
        apply_color!(
            sidebar_primary_foreground,
            fallback = self.primary_foreground
        );
        apply_background_color!(skeleton, fallback = tokens.secondary);
        apply_background_color!(slider_bar, fallback = tokens.primary);
        apply_background_color!(slider_thumb, fallback = self.primary_foreground);
        apply_background_color!(switch, fallback = tokens.secondary_active);
        apply_background_color!(switch_thumb, fallback = tokens.background);
        apply_background_color!(tab, fallback = tokens.background);
        apply_background_color!(tab_active, fallback = tokens.background);
        apply_color!(tab_active_foreground, fallback = self.foreground);
        apply_background_color!(tab_bar, fallback = tokens.background);
        apply_background_color!(tab_bar_segmented, fallback = tokens.secondary);
        apply_color!(tab_foreground, fallback = self.foreground);
        apply_background_color!(table, fallback = tokens.list);
        apply_background_color!(table_active, fallback = tokens.list_active);
        apply_color!(table_active_border, fallback = self.list_active_border);
        apply_background_color!(table_even, fallback = tokens.list_even);
        apply_background_color!(table_head, fallback = tokens.list_head);
        apply_color!(table_head_foreground, fallback = self.muted_foreground);
        apply_background_color!(table_foot, fallback = tokens.list_head);
        apply_color!(table_foot_foreground, fallback = self.muted_foreground);
        apply_background_color!(table_hover, fallback = tokens.list_hover);
        apply_color!(table_row_border, fallback = self.border);
        apply_background_color!(title_bar, fallback = tokens.background);
        apply_color!(title_bar_border, fallback = self.border);
        apply_background_color!(status_bar, fallback = tokens.title_bar);
        apply_color!(status_bar_border, fallback = self.title_bar_border);
        apply_background_color!(tiles, fallback = tokens.background);
        apply_background_color!(overlay);
        apply_color!(window_border, fallback = self.border);

        // TODO: Apply default fallback colors to highlight.

        // Ensure opacity for list_active, table_active, selection.
        let clamp_alpha = |raw: Option<&str>, color: Hsla, background: Background, max: f32| {
            let base = color.a;
            let target = base.min(max);
            let color = color.alpha(target);
            let background = raw
                .and_then(|value| try_parse_background_clamped(value, max).ok())
                .unwrap_or_else(|| {
                    let factor = if base > 0. { target / base } else { 1. };
                    background.opacity(factor)
                });
            (color, ThemeToken::new(color, background))
        };

        (self.list_active, tokens.list_active) = clamp_alpha(
            colors.list_active.as_deref(),
            self.list_active,
            tokens.list_active.background,
            0.2,
        );
        (self.table_active, tokens.table_active) = clamp_alpha(
            colors.table_active.as_deref(),
            self.table_active,
            tokens.table_active.background,
            0.2,
        );
        (self.selection, tokens.selection) = clamp_alpha(
            colors.selection.as_deref(),
            self.selection,
            tokens.selection.background,
            0.3,
        );

        // Direct component and product roles resolve only after their upstream
        // parents are final, including the highlight alpha clamps above.
        apply_color!(border_strong, fallback = self.input);

        apply_background_color!(card, fallback = tokens.group_box);
        apply_color!(card_foreground, fallback = self.group_box_foreground);
        apply_color!(card_border, fallback = self.border);
        apply_background_color!(card_hover, fallback = tokens.list_hover);
        apply_background_color!(card_active, fallback = tokens.button_secondary_active);
        apply_background_color!(card_selected, fallback = tokens.list_active);
        apply_color!(card_selected_border, fallback = self.list_active_border);

        apply_background_color!(switch_checked, fallback = tokens.primary);
        apply_background_color!(switch_thumb_checked, fallback = tokens.switch_thumb);

        apply_background_color!(transport, fallback = tokens.background);
        apply_color!(transport_border, fallback = self.border);

        apply_background_color!(knob, fallback = self.muted_foreground.opacity(0.2));
        apply_background_color!(knob_value, fallback = tokens.slider_bar);
        apply_color!(knob_foreground, fallback = self.foreground);

        apply_color!(similarity_low, fallback = self.danger);
        apply_color!(similarity_medium, fallback = self.warning);
        apply_color!(similarity_high, fallback = self.success);

        apply_background_color!(meter_fill, fallback = tokens.primary);
        apply_background_color!(meter_peak, fallback = tokens.primary);
        apply_background_color!(
            meter_track,
            fallback = self.border.opacity(0.45).opacity(0.6)
        );
        apply_background_color!(meter_clip, fallback = tokens.danger);

        let background = self.background;
        let foreground = self.foreground;
        let muted_foreground = self.muted_foreground;
        let selection = self.selection;
        let is_dark = background.l < 0.5;

        apply_background_color!(waveform, fallback = tokens.background);

        let thumbnail_fill = Hsla {
            a: 1.,
            ..muted_foreground
        }
        .mix_oklab(
            Hsla {
                a: 1.,
                ..self.accent
            },
            0.92,
        );
        apply_background_color!(waveform_thumbnail_fill, fallback = thumbnail_fill);

        let waveform_fill = waveform_tint(
            background,
            foreground,
            selection,
            0.30,
            if is_dark { 0.78 } else { 0.76 },
        );
        apply_background_color!(waveform_fill, fallback = waveform_fill);

        let selected_waveform_fill = waveform_tint(
            background,
            foreground,
            selection,
            0.55,
            if is_dark { 0.86 } else { 0.84 },
        );
        apply_color!(
            waveform_time_selection_foreground,
            fallback = selected_waveform_fill
        );

        let selection_alpha_factor = if selection.a > 0. {
            0.18 / selection.a
        } else {
            1.
        };
        let time_selection = ThemeToken::new(
            selection.alpha(0.18),
            tokens.selection.background.opacity(selection_alpha_factor),
        );
        apply_background_color!(waveform_time_selection, fallback = time_selection);
        apply_color!(
            waveform_time_selection_border,
            fallback = selection.alpha(if is_dark { 0.72 } else { 0.66 })
        );
        apply_color!(
            waveform_time_selection_active_border,
            fallback = selection.alpha(0.78)
        );

        let overlay_fill = Hsla {
            h: selection.h,
            s: 0.22,
            l: background.l + (foreground.l - background.l) * if is_dark { 0.92 } else { 0.88 },
            a: 1.,
        };
        apply_background_color!(waveform_overlay_fill, fallback = overlay_fill);
        apply_color!(
            waveform_overlay_zero_line,
            fallback = Hsla {
                h: 0.,
                s: 0.,
                l: if overlay_fill.l > 0.5 { 0. } else { 1. },
                a: 0.80,
            }
        );
        apply_color!(
            waveform_overlay_time_selection_foreground,
            fallback = selected_overlay_waveform(overlay_fill)
        );
        apply_color!(waveform_zero_line, fallback = foreground.alpha(0.05));
        apply_color!(
            waveform_fade_control,
            fallback = waveform_tint(background, foreground, selection, 0.72, 0.55).alpha(0.85)
        );
        apply_color!(
            waveform_playhead,
            fallback = Hsla {
                h: 0.02,
                s: 0.56,
                l: 0.46,
                a: 1.,
            }
        );
        apply_background_color!(waveform_ruler, fallback = tokens.table_head);
        apply_color!(
            waveform_ruler_foreground,
            fallback = muted_foreground.alpha(0.65)
        );

        let channel_label = Hsla {
            a: 0.92,
            ..mix_hsla(background, foreground, 0.10)
        };
        apply_background_color!(waveform_channel_label, fallback = channel_label);
        apply_color!(
            waveform_channel_label_foreground,
            fallback = foreground.alpha(0.60)
        );

        apply_background_color!(waveform_marker, fallback = tokens.danger);
        apply_color!(
            waveform_marker_foreground,
            fallback = self.danger_foreground
        );
        apply_color!(waveform_marker_active, fallback = self.danger_active);

        let segment_alpha = (self.primary.a * 0.55).clamp(0.35, 1.);
        let segment_alpha_factor = if self.primary.a > 0. {
            segment_alpha / self.primary.a
        } else {
            1.
        };
        let segment = ThemeToken::new(
            self.primary.alpha(segment_alpha),
            tokens.primary.background.opacity(segment_alpha_factor),
        );
        apply_background_color!(waveform_segment, fallback = segment);
        apply_background_color!(waveform_segment_active, fallback = tokens.primary);

        tokens
    }
}

impl Theme {
    fn resolved_config_colors(&self) -> ThemeConfigColors {
        let color = |value: Hsla| Some(SharedString::from(value.to_hex()));
        let base_color = |value: Hsla| Some(value.to_hex());

        // Keep this construction exhaustive: adding a schema field must fail
        // compilation until its resolved representative is wired here.
        ThemeConfigColors {
            accent: color(self.accent),
            accent_foreground: color(self.accent_foreground),
            accordion: color(self.accordion),
            background: color(self.background),
            window_background: color(self.window_background),
            border: color(self.border),
            button: color(self.button),
            button_active: color(self.button_active),
            button_foreground: color(self.button_foreground),
            button_hover: color(self.button_hover),
            button_danger: color(self.button_danger),
            button_danger_active: color(self.button_danger_active),
            button_danger_foreground: color(self.button_danger_foreground),
            button_danger_hover: color(self.button_danger_hover),
            button_info: color(self.button_info),
            button_info_active: color(self.button_info_active),
            button_info_foreground: color(self.button_info_foreground),
            button_info_hover: color(self.button_info_hover),
            button_primary: color(self.button_primary),
            button_primary_active: color(self.button_primary_active),
            button_primary_foreground: color(self.button_primary_foreground),
            button_primary_hover: color(self.button_primary_hover),
            button_secondary: color(self.button_secondary),
            button_secondary_active: color(self.button_secondary_active),
            button_secondary_foreground: color(self.button_secondary_foreground),
            button_secondary_hover: color(self.button_secondary_hover),
            button_success: color(self.button_success),
            button_success_active: color(self.button_success_active),
            button_success_foreground: color(self.button_success_foreground),
            button_success_hover: color(self.button_success_hover),
            button_warning: color(self.button_warning),
            button_warning_active: color(self.button_warning_active),
            button_warning_foreground: color(self.button_warning_foreground),
            button_warning_hover: color(self.button_warning_hover),
            group_box: color(self.group_box),
            group_box_foreground: color(self.group_box_foreground),
            group_box_title_foreground: color(self.group_box_title_foreground),
            caret: color(self.caret),
            chart_1: color(self.chart_1),
            chart_2: color(self.chart_2),
            chart_3: color(self.chart_3),
            chart_4: color(self.chart_4),
            chart_5: color(self.chart_5),
            chart_bullish: color(self.chart_bullish),
            chart_bearish: color(self.chart_bearish),
            danger: color(self.danger),
            danger_active: color(self.danger_active),
            danger_foreground: color(self.danger_foreground),
            danger_hover: color(self.danger_hover),
            description_list_label: color(self.description_list_label),
            description_list_label_foreground: color(self.description_list_label_foreground),
            drag_border: color(self.drag_border),
            drop_target: color(self.drop_target),
            foreground: color(self.foreground),
            info: color(self.info),
            info_active: color(self.info_active),
            info_foreground: color(self.info_foreground),
            info_hover: color(self.info_hover),
            input: color(self.input),
            link: color(self.link),
            link_active: color(self.link_active),
            link_hover: color(self.link_hover),
            list: color(self.colors.list),
            list_active: color(self.list_active),
            list_active_border: color(self.list_active_border),
            list_even: color(self.list_even),
            list_head: color(self.list_head),
            list_hover: color(self.list_hover),
            muted: color(self.muted),
            muted_foreground: color(self.muted_foreground),
            popover: color(self.popover),
            popover_foreground: color(self.popover_foreground),
            primary: color(self.primary),
            primary_active: color(self.primary_active),
            primary_foreground: color(self.primary_foreground),
            primary_hover: color(self.primary_hover),
            progress_bar: color(self.progress_bar),
            ring: color(self.ring),
            scrollbar: color(self.scrollbar),
            scrollbar_thumb: color(self.scrollbar_thumb),
            scrollbar_thumb_hover: color(self.scrollbar_thumb_hover),
            secondary: color(self.secondary),
            secondary_active: color(self.secondary_active),
            secondary_foreground: color(self.secondary_foreground),
            secondary_hover: color(self.secondary_hover),
            selection: color(self.selection),
            sidebar: color(self.sidebar),
            sidebar_accent: color(self.sidebar_accent),
            sidebar_accent_foreground: color(self.sidebar_accent_foreground),
            sidebar_border: color(self.sidebar_border),
            sidebar_foreground: color(self.sidebar_foreground),
            sidebar_primary: color(self.sidebar_primary),
            sidebar_primary_foreground: color(self.sidebar_primary_foreground),
            skeleton: color(self.skeleton),
            slider_bar: color(self.slider_bar),
            slider_thumb: color(self.slider_thumb),
            success: color(self.success),
            success_foreground: color(self.success_foreground),
            success_hover: color(self.success_hover),
            success_active: color(self.success_active),
            switch: color(self.switch),
            switch_thumb: color(self.switch_thumb),
            tab: color(self.tab),
            tab_active: color(self.tab_active),
            tab_active_foreground: color(self.tab_active_foreground),
            tab_bar: color(self.tab_bar),
            tab_bar_segmented: color(self.tab_bar_segmented),
            tab_foreground: color(self.tab_foreground),
            table: color(self.table),
            table_active: color(self.table_active),
            table_active_border: color(self.table_active_border),
            table_even: color(self.table_even),
            table_head: color(self.table_head),
            table_head_foreground: color(self.table_head_foreground),
            table_foot: color(self.table_foot),
            table_foot_foreground: color(self.table_foot_foreground),
            table_hover: color(self.table_hover),
            table_row_border: color(self.table_row_border),
            title_bar: color(self.title_bar),
            title_bar_border: color(self.title_bar_border),
            status_bar: color(self.status_bar),
            status_bar_border: color(self.status_bar_border),
            tiles: color(self.tiles),
            warning: color(self.warning),
            warning_active: color(self.warning_active),
            warning_hover: color(self.warning_hover),
            warning_foreground: color(self.warning_foreground),
            overlay: color(self.overlay),
            window_border: color(self.window_border),
            blue: base_color(self.blue),
            blue_light: base_color(self.blue_light),
            cyan: base_color(self.cyan),
            cyan_light: base_color(self.cyan_light),
            green: base_color(self.green),
            green_light: base_color(self.green_light),
            magenta: base_color(self.magenta),
            magenta_light: base_color(self.magenta_light),
            red: base_color(self.red),
            red_light: base_color(self.red_light),
            yellow: base_color(self.yellow),
            yellow_light: base_color(self.yellow_light),
            border_strong: color(self.border_strong),
            card: color(self.card),
            card_foreground: color(self.card_foreground),
            card_border: color(self.card_border),
            card_hover: color(self.card_hover),
            card_active: color(self.card_active),
            card_selected: color(self.card_selected),
            card_selected_border: color(self.card_selected_border),
            switch_checked: color(self.switch_checked),
            switch_thumb_checked: color(self.switch_thumb_checked),
            transport: color(self.transport),
            transport_border: color(self.transport_border),
            knob: color(self.knob),
            knob_value: color(self.knob_value),
            knob_foreground: color(self.knob_foreground),
            similarity_low: color(self.similarity_low),
            similarity_medium: color(self.similarity_medium),
            similarity_high: color(self.similarity_high),
            meter_fill: color(self.meter_fill),
            meter_peak: color(self.meter_peak),
            meter_track: color(self.meter_track),
            meter_clip: color(self.meter_clip),
            waveform: color(self.waveform),
            waveform_thumbnail_fill: color(self.waveform_thumbnail_fill),
            waveform_fill: color(self.waveform_fill),
            waveform_time_selection_foreground: color(self.waveform_time_selection_foreground),
            waveform_time_selection: color(self.waveform_time_selection),
            waveform_time_selection_border: color(self.waveform_time_selection_border),
            waveform_time_selection_active_border: color(
                self.waveform_time_selection_active_border,
            ),
            waveform_overlay_fill: color(self.waveform_overlay_fill),
            waveform_overlay_zero_line: color(self.waveform_overlay_zero_line),
            waveform_overlay_time_selection_foreground: color(
                self.waveform_overlay_time_selection_foreground,
            ),
            waveform_zero_line: color(self.waveform_zero_line),
            waveform_fade_control: color(self.waveform_fade_control),
            waveform_playhead: color(self.waveform_playhead),
            waveform_ruler: color(self.waveform_ruler),
            waveform_ruler_foreground: color(self.waveform_ruler_foreground),
            waveform_channel_label: color(self.waveform_channel_label),
            waveform_channel_label_foreground: color(self.waveform_channel_label_foreground),
            waveform_marker: color(self.waveform_marker),
            waveform_marker_foreground: color(self.waveform_marker_foreground),
            waveform_marker_active: color(self.waveform_marker_active),
            waveform_segment: color(self.waveform_segment),
            waveform_segment_active: color(self.waveform_segment_active),
        }
    }

    /// Return every schema color property with its canonical key, generated
    /// description, and resolved representative color.
    pub fn resolved_color_properties(&self) -> anyhow::Result<Vec<ResolvedThemeColorProperty>> {
        let schema = serde_json::to_value(schemars::schema_for!(ThemeConfigColors))?;
        let properties = schema
            .get("properties")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| anyhow::anyhow!("ThemeConfigColors schema has no properties object"))?;
        let resolved = serde_json::to_value(self.resolved_config_colors())?;
        let resolved = resolved
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("resolved theme colors are not an object"))?;

        properties
            .iter()
            .map(|(key, property_schema)| {
                let value = resolved
                    .get(key)
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("missing resolved theme color for {key}"))?;
                let description = property_schema
                    .get("description")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                Ok(ResolvedThemeColorProperty {
                    key: key.clone().into(),
                    description: description.to_string().into(),
                    color: try_parse_color(value)?,
                })
            })
            .collect()
    }

    /// Apply the given theme configuration to the current theme.
    pub fn apply_config(&mut self, config: &Rc<ThemeConfig>) {
        if config.mode.is_dark() {
            self.dark_theme = config.clone();
        } else {
            self.light_theme = config.clone();
        }
        self.highlight_theme = if let Some(style) = &config.highlight {
            Arc::new(HighlightTheme {
                name: config.name.to_string(),
                appearance: config.mode,
                style: style.clone(),
            })
        } else if config.mode.is_dark() {
            HighlightTheme::default_dark()
        } else {
            HighlightTheme::default_light()
        };

        let default_colors = if config.mode.is_dark() {
            ThemeColor::dark()
        } else {
            ThemeColor::light()
        };
        let defaults = Theme::from(default_colors.as_ref());

        // Theme configs are sparse. Every application starts from canonical
        // defaults so omitted non-color properties cannot leak from the theme
        // that happened to be active immediately before this one.
        self.font_size = config.font_size.map(px).unwrap_or(defaults.font_size);
        self.font_family = config.font_family.clone().unwrap_or(defaults.font_family);
        self.mono_font_family = config
            .mono_font_family
            .clone()
            .unwrap_or(defaults.mono_font_family);
        self.mono_font_size = config
            .mono_font_size
            .map(px)
            .unwrap_or(defaults.mono_font_size);
        self.radius = config
            .radius
            .map(|radius| px(radius as f32))
            .unwrap_or(defaults.radius);
        self.radius_lg = config
            .radius_lg
            .map(|radius| px(radius as f32))
            .unwrap_or(defaults.radius_lg);
        self.shadow = config.shadow.unwrap_or(defaults.shadow);
        self.translucency = ThemeTranslucency::resolve(&config.translucency);

        self.tokens = self.colors.apply_config(&config, &default_colors);
        self.mode = config.mode;
    }
}

#[cfg(test)]
mod tests {
    use gpui::{WindowBackgroundAppearance, linear_color_stop, linear_gradient, px};

    use crate::{
        Theme, ThemeConfig, ThemeMode, ThemeSet, ThemeTranslucencyConfig, try_parse_color,
    };

    #[test]
    fn test_semantic_theme_config_parses_and_roundtrips() {
        let value = serde_json::json!({
            "name": "Semantic",
            "mode": "dark",
            "tokens": {
                "colors": {
                    "surface": "#111827",
                    "surface_foreground": "#f9fafb",
                    "primary": "#2563eb",
                    "destructive": "#dc2626"
                },
                "radius": { "sm": 2.0, "md": 6.0, "lg": 10.0 },
                "spacing": { "xs": 4.0, "md": 12.0, "xl": 24.0 },
                "typography": {
                    "sans": "Inter",
                    "md": { "size": 15.0, "line_height": 22.0 }
                }
            }
        });
        let config: super::SemanticThemeConfigFile = serde_json::from_value(value).unwrap();
        let serialized = serde_json::to_string(&config).unwrap();
        let reparsed: super::SemanticThemeConfigFile = serde_json::from_str(&serialized).unwrap();
        let semantic = reparsed.tokens;

        assert_eq!(semantic.colors.surface.as_deref(), Some("#111827"));
        assert_eq!(semantic.colors.destructive.as_deref(), Some("#dc2626"));
        assert_eq!(semantic.radius.lg, Some(10.0));
        assert_eq!(semantic.spacing.xl, Some(24.0));
        assert_eq!(semantic.typography.sans.as_deref(), Some("Inter"));
        assert_eq!(semantic.typography.md.line_height, Some(22.0));

        let mut theme = Theme::default();
        let resolved = theme.apply_semantic_config_str(&serialized).unwrap();
        assert_eq!(theme.primary, try_parse_color("#2563eb").unwrap());
        assert_eq!(resolved.spacing.xl, px(24.));
        assert_eq!(resolved.typography.md.line_height, px(22.));
    }

    #[test]
    fn test_semantic_tokens_override_legacy_generic_fields_only() {
        let config = serde_json::from_value::<super::SemanticThemeConfigFile>(serde_json::json!({
            "tokens": {
                "colors": { "primary": "#2563eb", "destructive": "#b91c1c" },
                "spacing": { "md": 14.0 },
                "typography": { "md": { "size": 15.0 } }
            }
        }))
        .unwrap();
        let mut theme = Theme::default();
        let component_color = theme.button_primary;
        let resolved = theme.apply_semantic_config(&config.tokens);

        assert_eq!(theme.primary, try_parse_color("#2563eb").unwrap());
        assert_eq!(theme.danger, try_parse_color("#b91c1c").unwrap());
        assert_eq!(theme.button_primary, component_color);
        assert_eq!(resolved.spacing.md, px(14.));
        assert_eq!(resolved.typography.md.size, px(15.));
    }

    #[test]
    fn test_legacy_config_without_semantic_tokens_is_unchanged() {
        let config = serde_json::from_value::<ThemeConfig>(serde_json::json!({
            "name": "Legacy",
            "mode": "light",
            "radius": 7,
            "colors": { "primary.background": "#7c3aed" }
        }))
        .unwrap();
        let mut theme = Theme::default();
        theme.apply_config(&std::rc::Rc::new(config));
        assert_eq!(theme.primary, try_parse_color("#7c3aed").unwrap());
        assert_eq!(theme.radius, px(7.));
        assert_eq!(theme.semantic_tokens().spacing, Default::default());
    }

    #[test]
    fn translucency_is_disabled_by_default_even_with_a_transparent_background() {
        let config = serde_json::from_value::<ThemeConfig>(serde_json::json!({
            "name": "Opaque by default",
            "mode": "light",
            "colors": { "background": "#ffffff80" }
        }))
        .unwrap();

        let mut theme = Theme::default();
        theme.apply_config(&std::rc::Rc::new(config));

        assert!(!theme.glass_active());
        assert_eq!(
            theme.window_background_appearance(),
            WindowBackgroundAppearance::Opaque
        );
        assert_eq!(theme.overlay_blur(), px(0.));
        assert_eq!(theme.panel_blur(), px(0.));
    }

    #[test]
    fn translucency_roundtrips_and_resolves_authored_blur_radii() {
        let config = serde_json::from_value::<ThemeConfig>(serde_json::json!({
            "name": "Glass",
            "mode": "dark",
            "translucency": {
                "window": true,
                "overlay_blur": 44,
                "panel_blur": 12
            }
        }))
        .unwrap();

        assert_eq!(
            serde_json::to_value(&config).unwrap()["translucency"],
            serde_json::json!({ "window": true, "overlay_blur": 44.0, "panel_blur": 12.0 })
        );

        let mut theme = Theme::default();
        theme.apply_config(&std::rc::Rc::new(config));

        assert!(theme.glass_active());
        assert_eq!(
            theme.window_background_appearance(),
            WindowBackgroundAppearance::Blurred
        );
        assert_eq!(theme.overlay_blur(), px(44.));
        assert_eq!(theme.panel_blur(), px(12.));
    }

    #[test]
    fn translucency_clamps_blur_and_disables_local_material_when_window_is_opaque() {
        let config = serde_json::from_value::<ThemeConfig>(serde_json::json!({
            "name": "Clamped",
            "mode": "dark",
            "translucency": {
                "window": true,
                "overlay_blur": -4,
                "panel_blur": 128
            }
        }))
        .unwrap();
        let mut theme = Theme::default();
        theme.apply_config(&std::rc::Rc::new(config));

        assert_eq!(theme.overlay_blur(), px(0.));
        assert_eq!(theme.panel_blur(), px(64.));

        let opaque_config = serde_json::from_value::<ThemeConfig>(serde_json::json!({
            "name": "Opaque",
            "mode": "light",
            "translucency": {
                "window": false,
                "overlay_blur": 44,
                "panel_blur": 12
            }
        }))
        .unwrap();
        theme.apply_config(&std::rc::Rc::new(opaque_config));

        assert!(!theme.glass_active());
        assert_eq!(theme.overlay_blur(), px(0.));
        assert_eq!(theme.panel_blur(), px(0.));
    }

    #[test]
    fn translucency_normalizes_non_finite_blur_radii_to_zero() {
        let mut theme = Theme::default();

        for (overlay_blur, panel_blur) in [
            (f32::NAN, f32::NAN),
            (f32::INFINITY, f32::NEG_INFINITY),
            (f32::NEG_INFINITY, f32::INFINITY),
        ] {
            let config = ThemeConfig {
                translucency: ThemeTranslucencyConfig {
                    window: true,
                    overlay_blur,
                    panel_blur,
                },
                ..ThemeConfig::default()
            };
            theme.apply_config(&std::rc::Rc::new(config));

            assert!(theme.glass_active());
            assert_eq!(theme.overlay_blur(), px(0.));
            assert_eq!(theme.panel_blur(), px(0.));
        }
    }

    #[test]
    fn window_background_token_falls_back_without_discarding_authored_alpha() {
        let fallback_config = serde_json::from_value::<ThemeConfig>(serde_json::json!({
            "name": "Window fallback",
            "mode": "light",
            "colors": { "background": "#11223380" }
        }))
        .unwrap();
        let mut theme = Theme::default();
        theme.apply_config(&std::rc::Rc::new(fallback_config));
        assert_eq!(theme.tokens.window_background, theme.tokens.background);

        let config = serde_json::from_value::<ThemeConfig>(serde_json::json!({
            "name": "Window background",
            "mode": "light",
            "colors": {
                "background": "#112233",
                "window.background": "#44556680"
            }
        }))
        .unwrap();
        theme.apply_config(&std::rc::Rc::new(config));

        let window_background = try_parse_color("#44556680").unwrap();
        assert_eq!(theme.window_background, window_background);
        assert_eq!(theme.tokens.window_background.color, window_background);
        assert_eq!(
            theme.tokens.window_background.background,
            window_background.into()
        );
    }

    #[test]
    fn translucency_and_window_background_are_in_the_generated_schema() {
        let schema = serde_json::to_value(schemars::schema_for!(ThemeConfig)).unwrap();
        let properties = schema["properties"].as_object().unwrap();
        assert_eq!(
            properties["translucency"]["$ref"],
            "#/$defs/ThemeTranslucencyConfig"
        );
        let translucency_properties = schema["$defs"]["ThemeTranslucencyConfig"]["properties"]
            .as_object()
            .unwrap();
        assert!(translucency_properties.contains_key("window"));
        assert!(translucency_properties.contains_key("overlay_blur"));
        assert!(translucency_properties.contains_key("panel_blur"));
        assert_eq!(translucency_properties["overlay_blur"]["minimum"], 0.0);
        assert_eq!(translucency_properties["overlay_blur"]["maximum"], 64.0);

        assert_eq!(properties["colors"]["$ref"], "#/$defs/ThemeConfigColors");
        assert!(
            schema["$defs"]["ThemeConfigColors"]["properties"]
                .get("window.background")
                .is_some()
        );
    }

    #[test]
    fn test_apply_config_preserves_gradient_background_and_solid_color_fallback() {
        let config = serde_json::from_value::<ThemeConfig>(serde_json::json!({
            "name": "Gradient",
            "mode": "light",
            "colors": {
                "primary.background": "linear-gradient(135deg, #4F46E5, #06B6D4)",
                "button.primary.hover.background": "linear-gradient(to right, red-500 25%, blue-600 75%)"
            }
        }))
        .unwrap();

        let mut theme = Theme::default();
        theme.apply_config(&std::rc::Rc::new(config));

        let primary_from = try_parse_color("#4F46E5").unwrap();
        let primary_to = try_parse_color("#06B6D4").unwrap();
        assert_eq!(theme.primary, primary_from);
        assert_eq!(theme.tokens.primary.color, primary_from);
        assert_eq!(
            theme.tokens.primary.background,
            linear_gradient(
                135.,
                linear_color_stop(primary_from, 0.),
                linear_color_stop(primary_to, 1.)
            )
        );
        assert_eq!(
            theme.tokens.button_primary.background,
            theme.tokens.primary.background
        );
        assert_eq!(
            theme.tokens.button_primary_hover.background,
            linear_gradient(
                90.,
                linear_color_stop(crate::red_500(), 0.25),
                linear_color_stop(crate::blue_600(), 0.75)
            )
        );
        assert_eq!(theme.mode, ThemeMode::Light);
    }

    #[test]
    fn test_waveform_segment_fallbacks_follow_primary_background() {
        let config = serde_json::from_value::<ThemeConfig>(serde_json::json!({
            "name": "Waveform Segment",
            "mode": "dark",
            "colors": {
                "accent.background": "#202020",
                "primary.background": "linear-gradient(135deg, #4F46E5, #06B6D4)"
            }
        }))
        .unwrap();

        let mut theme = Theme::default();
        theme.apply_config(&std::rc::Rc::new(config));

        let primary_from = try_parse_color("#4F46E5").unwrap();
        let primary_to = try_parse_color("#06B6D4").unwrap();
        assert_eq!(theme.waveform_segment, primary_from.alpha(0.55));
        assert_eq!(
            theme.tokens.waveform_segment.background,
            linear_gradient(
                135.,
                linear_color_stop(primary_from.alpha(0.55), 0.),
                linear_color_stop(primary_to.alpha(0.55), 1.),
            )
        );
        assert_eq!(theme.waveform_segment_active, primary_from);
        assert_eq!(
            theme.tokens.waveform_segment_active.background,
            theme.tokens.primary.background
        );
    }

    #[test]
    fn test_aurora_theme_parses_gradient_backgrounds() {
        let theme_set =
            serde_json::from_str::<ThemeSet>(include_str!("../../../../themes/aurora.json"))
                .unwrap();
        assert_eq!(theme_set.themes.len(), 1);
        assert!(theme_set.themes.iter().all(|theme| !theme.mode.is_dark()));

        let light = theme_set
            .themes
            .iter()
            .find(|theme| theme.name.as_ref() == "Aurora Light")
            .unwrap();
        let mut theme = Theme::default();
        theme.apply_config(&std::rc::Rc::new(light.clone()));

        assert_ne!(
            theme.tokens.button_primary.background,
            theme.button_primary.into()
        );
        assert_eq!(theme.tokens.background.background, theme.background.into());
        assert_eq!(theme.button_primary, try_parse_color("#1E293B").unwrap());
        assert_eq!(theme.background, try_parse_color("#FFFFFF").unwrap());
        assert_ne!(
            theme.tokens.progress_bar.background,
            theme.progress_bar.into()
        );
        assert_ne!(
            theme.tokens.scrollbar_thumb.background,
            theme.scrollbar_thumb.into()
        );
        assert_ne!(theme.tokens.switch.background, theme.switch.into());
        assert_ne!(
            theme.tokens.switch_thumb.background,
            theme.switch_thumb.into()
        );
        assert_ne!(theme.tokens.title_bar.background, theme.title_bar.into());
        assert_ne!(theme.tokens.status_bar.background, theme.status_bar.into());
    }

    #[test]
    fn test_apply_config_clamps_highlight_alpha_per_gradient_stop() {
        let config = serde_json::from_value::<ThemeConfig>(serde_json::json!({
            "name": "Highlight",
            "mode": "light",
            "colors": {
                // Solid above the cap: must be capped to 0.2, not attenuated twice.
                "list.active.background": "#3b82f6",
                // Gradient with a faint `from` stop and an opaque `to` stop: the
                // `to` stop must be clamped independently, not left at full alpha.
                "table.active.background": "linear-gradient(#bfdbfe33, #3b82f6)",
                // Gradient with a transparent `from` stop: the opaque `to` stop
                // must still be clamped (the `base == 0` factor fallback used to
                // leave it untouched).
                "selection.background": "linear-gradient(#3b82f600, #3b82f6)",
            }
        }))
        .unwrap();

        let mut theme = Theme::default();
        theme.apply_config(&std::rc::Rc::new(config));

        // Solid: representative color and rendered background both capped at 0.2.
        let blue = try_parse_color("#3b82f6").unwrap();
        assert_eq!(theme.list_active, blue.alpha(0.2));
        assert_eq!(theme.tokens.list_active.background, blue.alpha(0.2).into());

        // Gradient: the opaque `to` stop is clamped to 0.2, not left fully opaque.
        let faint = try_parse_color("#bfdbfe33").unwrap();
        assert_eq!(
            theme.tokens.table_active.background,
            linear_gradient(
                180.,
                linear_color_stop(faint.alpha(faint.a.min(0.2)), 0.),
                linear_color_stop(blue.alpha(0.2), 1.),
            )
        );

        // Gradient: a transparent `from` stop stays transparent while the opaque
        // `to` stop is still clamped to 0.3 (selection cap).
        let clear = try_parse_color("#3b82f600").unwrap();
        assert_eq!(
            theme.tokens.selection.background,
            linear_gradient(
                180.,
                linear_color_stop(clear.alpha(clear.a.min(0.3)), 0.),
                linear_color_stop(blue.alpha(0.3), 1.),
            )
        );
    }
}
