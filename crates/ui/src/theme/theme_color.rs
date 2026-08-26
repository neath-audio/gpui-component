use std::{ops::Deref, sync::Arc};

use crate::{ThemeMode, theme::DEFAULT_THEME_COLORS};

use gpui::{Background, Fill, Hsla};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A theme token that keeps a solid representative color and its renderable background.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ThemeToken {
    pub color: Hsla,
    pub background: Background,
}

impl ThemeToken {
    pub fn new(color: Hsla, background: Background) -> Self {
        Self { color, background }
    }
}

impl Deref for ThemeToken {
    type Target = Hsla;

    fn deref(&self) -> &Self::Target {
        &self.color
    }
}

impl From<Hsla> for ThemeToken {
    fn from(color: Hsla) -> Self {
        Self {
            color,
            background: color.into(),
        }
    }
}

impl From<ThemeToken> for Hsla {
    fn from(token: ThemeToken) -> Self {
        token.color
    }
}

impl From<ThemeToken> for Background {
    fn from(token: ThemeToken) -> Self {
        token.background
    }
}

impl From<ThemeToken> for Fill {
    fn from(token: ThemeToken) -> Self {
        Fill::Color(token.background)
    }
}

/// Theme colors used throughout the UI components.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct ThemeColor {
    /// Used for accents such as hover background on MenuItem, ListItem, etc.
    pub accent: Hsla,
    /// Used for accent text color.
    pub accent_foreground: Hsla,
    /// Accordion background color.
    pub accordion: Hsla,
    /// Default background color.
    pub background: Hsla,
    /// Default border color
    pub border: Hsla,
    /// Default Button background color.
    pub button: Hsla,
    /// Default Button active background color.
    pub button_active: Hsla,
    /// Default Button text color.
    pub button_foreground: Hsla,
    /// Default Button hover background color.
    pub button_hover: Hsla,
    /// Button danger background color, fallback to `danger`.
    pub button_danger: Hsla,
    /// Button danger active background color, fallback to `danger_active`.
    pub button_danger_active: Hsla,
    /// Button danger text color, fallback to `danger_foreground`.
    pub button_danger_foreground: Hsla,
    /// Button danger hover background color, fallback to `danger_hover`.
    pub button_danger_hover: Hsla,
    /// Button info background color, fallback to `info`.
    pub button_info: Hsla,
    /// Button info active background color, fallback to `info_active`.
    pub button_info_active: Hsla,
    /// Button info text color, fallback to `info_foreground`.
    pub button_info_foreground: Hsla,
    /// Button info hover background color, fallback to `info_hover`.
    pub button_info_hover: Hsla,
    /// Button primary background color, fallback to `primary`.
    pub button_primary: Hsla,
    /// Button primary active background color, fallback to `primary_active`.
    pub button_primary_active: Hsla,
    /// Button primary text color, fallback to `primary_foreground`.
    pub button_primary_foreground: Hsla,
    /// Button primary hover background color, fallback to `primary_hover`.
    pub button_primary_hover: Hsla,
    /// Button secondary background color, fallback to `secondary`.
    pub button_secondary: Hsla,
    /// Button secondary active background color, fallback to `secondary_active`.
    pub button_secondary_active: Hsla,
    /// Button secondary text color, fallback to `secondary_foreground`.
    pub button_secondary_foreground: Hsla,
    /// Button secondary hover background color, fallback to `secondary_hover`.
    pub button_secondary_hover: Hsla,
    /// Button success background color, fallback to `success`.
    pub button_success: Hsla,
    /// Button success active background color, fallback to `success_active`.
    pub button_success_active: Hsla,
    /// Button success text color, fallback to `success_foreground`.
    pub button_success_foreground: Hsla,
    /// Button success hover background color, fallback to `success_hover`.
    pub button_success_hover: Hsla,
    /// Button warning background color, fallback to `warning`.
    pub button_warning: Hsla,
    /// Button warning active background color, fallback to `warning_active`.
    pub button_warning_active: Hsla,
    /// Button warning text color, fallback to `warning_foreground`.
    pub button_warning_foreground: Hsla,
    /// Button warning hover background color, fallback to `warning_hover`.
    pub button_warning_hover: Hsla,
    /// Background color for GroupBox.
    pub group_box: Hsla,
    /// Text color for GroupBox.
    pub group_box_foreground: Hsla,
    /// Title text color for GroupBox.
    pub group_box_title_foreground: Hsla,
    /// Input caret color (Blinking cursor).
    pub caret: Hsla,
    /// Chart 1 color.
    pub chart_1: Hsla,
    /// Chart 2 color.
    pub chart_2: Hsla,
    /// Chart 3 color.
    pub chart_3: Hsla,
    /// Chart 4 color.
    pub chart_4: Hsla,
    /// Chart 5 color.
    pub chart_5: Hsla,
    /// Bullish color for candlestick charts (upward price movement).
    pub chart_bullish: Hsla,
    /// Bearish color for candlestick charts (downward price movement).
    pub chart_bearish: Hsla,
    /// Danger background color.
    pub danger: Hsla,
    /// Danger active background color.
    pub danger_active: Hsla,
    /// Danger text color.
    pub danger_foreground: Hsla,
    /// Danger hover background color.
    pub danger_hover: Hsla,
    /// Description List label background color.
    pub description_list_label: Hsla,
    /// Description List label foreground color.
    pub description_list_label_foreground: Hsla,
    /// Drag border color.
    pub drag_border: Hsla,
    /// Drop target background color.
    pub drop_target: Hsla,
    /// Default text color.
    pub foreground: Hsla,
    /// Info background color.
    pub info: Hsla,
    /// Info active background color.
    pub info_active: Hsla,
    /// Info text color.
    pub info_foreground: Hsla,
    /// Info hover background color.
    pub info_hover: Hsla,
    /// Border color for inputs such as Input, Select, etc.
    pub input: Hsla,
    /// Link text color.
    pub link: Hsla,
    /// Active link text color.
    pub link_active: Hsla,
    /// Hover link text color.
    pub link_hover: Hsla,
    /// Background color for List and ListItem.
    pub list: Hsla,
    /// Background color for active ListItem.
    pub list_active: Hsla,
    /// Border color for active ListItem.
    pub list_active_border: Hsla,
    /// Stripe background color for even ListItem.
    pub list_even: Hsla,
    /// Background color for List header.
    pub list_head: Hsla,
    /// Hover background color for ListItem.
    pub list_hover: Hsla,
    /// Muted backgrounds such as Skeleton and Switch.
    pub muted: Hsla,
    /// Muted text color, as used in disabled text.
    pub muted_foreground: Hsla,
    /// Background color for Popover.
    pub popover: Hsla,
    /// Text color for Popover.
    pub popover_foreground: Hsla,
    /// Primary background color.
    pub primary: Hsla,
    /// Active primary background color.
    pub primary_active: Hsla,
    /// Primary text color.
    pub primary_foreground: Hsla,
    /// Hover primary background color.
    pub primary_hover: Hsla,
    /// Progress bar background color.
    pub progress_bar: Hsla,
    /// Used for focus ring.
    pub ring: Hsla,
    /// Scrollbar background color.
    pub scrollbar: Hsla,
    /// Scrollbar thumb background color.
    pub scrollbar_thumb: Hsla,
    /// Scrollbar thumb hover background color.
    pub scrollbar_thumb_hover: Hsla,
    /// Secondary background color.
    pub secondary: Hsla,
    /// Active secondary background color.
    pub secondary_active: Hsla,
    /// Secondary text color, used for secondary Button text color or secondary text.
    pub secondary_foreground: Hsla,
    /// Hover secondary background color.
    pub secondary_hover: Hsla,
    /// Input selection background color.
    pub selection: Hsla,
    /// Sidebar background color.
    pub sidebar: Hsla,
    /// Sidebar accent background color.
    pub sidebar_accent: Hsla,
    /// Sidebar accent text color.
    pub sidebar_accent_foreground: Hsla,
    /// Sidebar border color.
    pub sidebar_border: Hsla,
    /// Sidebar text color.
    pub sidebar_foreground: Hsla,
    /// Sidebar primary background color.
    pub sidebar_primary: Hsla,
    /// Sidebar primary text color.
    pub sidebar_primary_foreground: Hsla,
    /// Skeleton background color.
    pub skeleton: Hsla,
    /// Slider bar background color.
    pub slider_bar: Hsla,
    /// Slider thumb background color.
    pub slider_thumb: Hsla,
    /// Success background color.
    pub success: Hsla,
    /// Success text color.
    pub success_foreground: Hsla,
    /// Success hover background color.
    pub success_hover: Hsla,
    /// Success active background color.
    pub success_active: Hsla,
    /// Switch background color.
    pub switch: Hsla,
    /// Switch thumb background color.
    pub switch_thumb: Hsla,
    /// Tab background color.
    pub tab: Hsla,
    /// Tab active background color.
    pub tab_active: Hsla,
    /// Tab active text color.
    pub tab_active_foreground: Hsla,
    /// TabBar background color.
    pub tab_bar: Hsla,
    /// TabBar segmented background color.
    pub tab_bar_segmented: Hsla,
    /// Tab text color.
    pub tab_foreground: Hsla,
    /// Table background color.
    pub table: Hsla,
    /// Table active item background color.
    pub table_active: Hsla,
    /// Table active item border color.
    pub table_active_border: Hsla,
    /// Stripe background color for even TableRow.
    pub table_even: Hsla,
    /// Table head background color.
    pub table_head: Hsla,
    /// Table head text color.
    pub table_head_foreground: Hsla,
    /// Table footer background color.
    pub table_foot: Hsla,
    /// Table footer text color.
    pub table_foot_foreground: Hsla,
    /// Table item hover background color.
    pub table_hover: Hsla,
    /// Table row border color.
    pub table_row_border: Hsla,
    /// TitleBar background color, use for Window title bar.
    pub title_bar: Hsla,
    /// TitleBar border color.
    pub title_bar_border: Hsla,
    /// StatusBar background color, use for the bottom status bar.
    pub status_bar: Hsla,
    /// StatusBar border color.
    pub status_bar_border: Hsla,
    /// Background color for Tiles.
    pub tiles: Hsla,
    /// Warning background color.
    pub warning: Hsla,
    /// Warning active background color.
    pub warning_active: Hsla,
    /// Warning hover background color.
    pub warning_hover: Hsla,
    /// Warning foreground color.
    pub warning_foreground: Hsla,
    /// Overlay background color.
    pub overlay: Hsla,
    /// Window border color.
    ///
    /// # Platform specific:
    ///
    /// This is only works on Linux, other platforms we can't change the window border color.
    pub window_border: Hsla,

    /// The base red color.
    pub red: Hsla,
    /// The base red light color.
    pub red_light: Hsla,
    /// The base green color.
    pub green: Hsla,
    /// The base green light color.
    pub green_light: Hsla,
    /// The base blue color.
    pub blue: Hsla,
    /// The base blue light color.
    pub blue_light: Hsla,
    /// The base yellow color.
    pub yellow: Hsla,
    /// The base yellow light color.
    pub yellow_light: Hsla,
    /// The base magenta color.
    pub magenta: Hsla,
    /// The base magenta light color.
    pub magenta_light: Hsla,
    /// The base cyan color.
    pub cyan: Hsla,
    /// The base cyan light color.
    pub cyan_light: Hsla,

    /// Strong border color for emphasized outlines and drop zones.
    pub border_strong: Hsla,
    /// Card background color.
    pub card: Hsla,
    /// Card foreground color.
    pub card_foreground: Hsla,
    /// Card border color.
    pub card_border: Hsla,
    /// Card hover background color.
    pub card_hover: Hsla,
    /// Card active background color.
    pub card_active: Hsla,
    /// Selected card background color.
    pub card_selected: Hsla,
    /// Selected card border color.
    pub card_selected_border: Hsla,
    /// Checked switch track background color.
    pub switch_checked: Hsla,
    /// Checked switch thumb background color.
    pub switch_thumb_checked: Hsla,
    /// Transport strip background color.
    pub transport: Hsla,
    /// Transport strip border color.
    pub transport_border: Hsla,
    /// Knob track background color.
    pub knob: Hsla,
    /// Knob value background color.
    pub knob_value: Hsla,
    /// Knob pointer foreground color.
    pub knob_foreground: Hsla,
    /// Low similarity score color.
    pub similarity_low: Hsla,
    /// Medium similarity score color.
    pub similarity_medium: Hsla,
    /// High similarity score color.
    pub similarity_high: Hsla,
    /// Meter fill color.
    pub meter_fill: Hsla,
    /// Meter held-peak color.
    pub meter_peak: Hsla,
    /// Meter track color.
    pub meter_track: Hsla,
    /// Meter clipping indicator color.
    pub meter_clip: Hsla,
    /// Waveform canvas background color.
    pub waveform: Hsla,
    /// Compact waveform thumbnail fill color.
    pub waveform_thumbnail_fill: Hsla,
    /// Main waveform fill color.
    pub waveform_fill: Hsla,
    /// Waveform fill inside a time selection.
    pub waveform_time_selection_foreground: Hsla,
    /// Time-selection band background color.
    pub waveform_time_selection: Hsla,
    /// Time-selection border color.
    pub waveform_time_selection_border: Hsla,
    /// Active time-selection border color.
    pub waveform_time_selection_active_border: Hsla,
    /// Waveform fill over a visual overlay.
    pub waveform_overlay_fill: Hsla,
    /// Waveform overlay zero-line color.
    pub waveform_overlay_zero_line: Hsla,
    /// Selected waveform fill over a visual overlay.
    pub waveform_overlay_time_selection_foreground: Hsla,
    /// Waveform zero-line color.
    pub waveform_zero_line: Hsla,
    /// Waveform fade-control color.
    pub waveform_fade_control: Hsla,
    /// Waveform playhead color.
    pub waveform_playhead: Hsla,
    /// Waveform ruler background color.
    pub waveform_ruler: Hsla,
    /// Waveform ruler foreground color.
    pub waveform_ruler_foreground: Hsla,
    /// Waveform channel-label background color.
    pub waveform_channel_label: Hsla,
    /// Waveform channel-label foreground color.
    pub waveform_channel_label_foreground: Hsla,
    /// Waveform marker background color.
    pub waveform_marker: Hsla,
    /// Waveform marker foreground color.
    pub waveform_marker_foreground: Hsla,
    /// Active waveform marker color.
    pub waveform_marker_active: Hsla,
    /// Waveform segment background color.
    pub waveform_segment: Hsla,
    /// Active waveform segment color.
    pub waveform_segment_active: Hsla,
}

macro_rules! define_theme_tokens {
    ($($field:ident),+ $(,)?) => {
        /// Schema-backed resolved tokens for `gpui-component` and product roles.
        ///
        /// This is the complete public projection of [`ThemeColor`], including
        /// component-specific and application-owned fields such as `button_primary`,
        /// `tab_active`, and `waveform`. [`gpui_base::SemanticThemeTokens`] is the
        /// smaller generic projection for code that depends only on `gpui-base`.
        #[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
        pub struct ThemeTokens {
            $(pub $field: ThemeToken,)+
        }

        impl From<ThemeColor> for ThemeTokens {
            fn from(colors: ThemeColor) -> Self {
                Self {
                    $($field: colors.$field.into(),)+
                }
            }
        }

        impl From<&ThemeColor> for ThemeTokens {
            fn from(colors: &ThemeColor) -> Self {
                Self::from(*colors)
            }
        }
    };
}

define_theme_tokens! {
    accent,
    accent_foreground,
    accordion,
    background,
    border,
    button,
    button_active,
    button_foreground,
    button_hover,
    button_danger,
    button_danger_active,
    button_danger_foreground,
    button_danger_hover,
    button_info,
    button_info_active,
    button_info_foreground,
    button_info_hover,
    button_primary,
    button_primary_active,
    button_primary_foreground,
    button_primary_hover,
    button_secondary,
    button_secondary_active,
    button_secondary_foreground,
    button_secondary_hover,
    button_success,
    button_success_active,
    button_success_foreground,
    button_success_hover,
    button_warning,
    button_warning_active,
    button_warning_foreground,
    button_warning_hover,
    group_box,
    group_box_foreground,
    group_box_title_foreground,
    caret,
    chart_1,
    chart_2,
    chart_3,
    chart_4,
    chart_5,
    chart_bullish,
    chart_bearish,
    danger,
    danger_active,
    danger_foreground,
    danger_hover,
    description_list_label,
    description_list_label_foreground,
    drag_border,
    drop_target,
    foreground,
    info,
    info_active,
    info_foreground,
    info_hover,
    input,
    link,
    link_active,
    link_hover,
    list,
    list_active,
    list_active_border,
    list_even,
    list_head,
    list_hover,
    muted,
    muted_foreground,
    popover,
    popover_foreground,
    primary,
    primary_active,
    primary_foreground,
    primary_hover,
    progress_bar,
    ring,
    scrollbar,
    scrollbar_thumb,
    scrollbar_thumb_hover,
    secondary,
    secondary_active,
    secondary_foreground,
    secondary_hover,
    selection,
    sidebar,
    sidebar_accent,
    sidebar_accent_foreground,
    sidebar_border,
    sidebar_foreground,
    sidebar_primary,
    sidebar_primary_foreground,
    skeleton,
    slider_bar,
    slider_thumb,
    success,
    success_foreground,
    success_hover,
    success_active,
    switch,
    switch_thumb,
    tab,
    tab_active,
    tab_active_foreground,
    tab_bar,
    tab_bar_segmented,
    tab_foreground,
    table,
    table_active,
    table_active_border,
    table_even,
    table_head,
    table_head_foreground,
    table_foot,
    table_foot_foreground,
    table_hover,
    table_row_border,
    title_bar,
    title_bar_border,
    status_bar,
    status_bar_border,
    tiles,
    warning,
    warning_active,
    warning_hover,
    warning_foreground,
    overlay,
    window_border,
    red,
    red_light,
    green,
    green_light,
    blue,
    blue_light,
    yellow,
    yellow_light,
    magenta,
    magenta_light,
    cyan,
    cyan_light,
    border_strong,
    card,
    card_foreground,
    card_border,
    card_hover,
    card_active,
    card_selected,
    card_selected_border,
    switch_checked,
    switch_thumb_checked,
    transport,
    transport_border,
    knob,
    knob_value,
    knob_foreground,
    similarity_low,
    similarity_medium,
    similarity_high,
    meter_fill,
    meter_peak,
    meter_track,
    meter_clip,
    waveform,
    waveform_thumbnail_fill,
    waveform_fill,
    waveform_time_selection_foreground,
    waveform_time_selection,
    waveform_time_selection_border,
    waveform_time_selection_active_border,
    waveform_overlay_fill,
    waveform_overlay_zero_line,
    waveform_overlay_time_selection_foreground,
    waveform_zero_line,
    waveform_fade_control,
    waveform_playhead,
    waveform_ruler,
    waveform_ruler_foreground,
    waveform_channel_label,
    waveform_channel_label_foreground,
    waveform_marker,
    waveform_marker_foreground,
    waveform_marker_active,
    waveform_segment,
    waveform_segment_active,
}

impl ThemeColor {
    /// Get the default light theme colors.
    pub fn light() -> Arc<Self> {
        DEFAULT_THEME_COLORS[&ThemeMode::Light].0.clone()
    }

    /// Get the default dark theme colors.
    pub fn dark() -> Arc<Self> {
        DEFAULT_THEME_COLORS[&ThemeMode::Dark].0.clone()
    }
}
