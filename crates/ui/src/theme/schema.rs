use std::{rc::Rc, sync::Arc};

use gpui::{Hsla, SharedString, px};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeMap};
use serde_json::{Map, Value};

use crate::highlighter::{HighlightTheme, HighlightThemeStyle};

use super::color::try_parse_color;
use super::{Colorize as _, Theme, ThemeMode};

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

    /// The colors of the theme.
    pub colors: ThemeConfigColors,
    /// The highlight theme, this part is combilbility with `style` section in Zed theme.
    ///
    /// https://github.com/zed-industries/zed/blob/f50041779dcfd7a76c8aec293361c60c53f02d51/assets/themes/ayu/ayu.json#L9
    pub highlight: Option<HighlightThemeStyle>,
}

/// Color overrides for the 38 theme roles.
///
/// Deserialize accepts the new field names and the spec load-map's old
/// dotted keys (first present wins). Serialize emits only set field names.
#[derive(Debug, Clone, Default, PartialEq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ThemeConfigColors {
    pub bg: Option<SharedString>,
    pub surface: Option<SharedString>,
    pub surface_raised: Option<SharedString>,
    pub surface_raised_hover: Option<SharedString>,
    pub surface_card: Option<SharedString>,
    pub surface_dialog: Option<SharedString>,
    pub surface_overlay: Option<SharedString>,
    pub band: Option<SharedString>,
    pub input_bg: Option<SharedString>,
    pub text: Option<SharedString>,
    pub text_muted: Option<SharedString>,
    pub text_faint: Option<SharedString>,
    pub text_dim: Option<SharedString>,
    pub border: Option<SharedString>,
    pub border_strong: Option<SharedString>,
    pub accent: Option<SharedString>,
    pub accent_strong: Option<SharedString>,
    pub on_accent: Option<SharedString>,
    pub solid: Option<SharedString>,
    pub on_solid: Option<SharedString>,
    pub danger: Option<SharedString>,
    pub danger_muted: Option<SharedString>,
    pub danger_strong: Option<SharedString>,
    pub warning: Option<SharedString>,
    pub warning_muted: Option<SharedString>,
    pub success: Option<SharedString>,
    pub success_muted: Option<SharedString>,
    pub busy: Option<SharedString>,
    pub focus: Option<SharedString>,
    pub row_hover: Option<SharedString>,
    pub row_selected: Option<SharedString>,
    pub table_header_bg: Option<SharedString>,
    pub table_row_hover: Option<SharedString>,
    pub table_row_selected: Option<SharedString>,
    pub table_row_selected_border: Option<SharedString>,
    pub selection: Option<SharedString>,
    pub caret: Option<SharedString>,
    pub cursor: Option<SharedString>,
}

fn first(map: &Map<String, Value>, keys: &[&str]) -> Option<SharedString> {
    keys.iter()
        .find_map(|k| map.get(*k).and_then(|v| v.as_str()).map(SharedString::from))
}

fn surface_from_legacy_colors(map: &Map<String, Value>) -> Option<SharedString> {
    if let Some(surface) = first(map, &["surface", "sidebar.background"]) {
        return Some(surface);
    }

    // The legacy ThemeColor loader did not use tab/title-bar colors when a
    // theme omitted sidebar.background. It derived the shell plane after
    // applying that theme's background and border values.
    let background = map.get("background")?.as_str()?;
    let border = map.get("border")?.as_str()?;
    let background = try_parse_color(background).ok()?;
    let border = try_parse_color(border).ok()?;
    Some(background.blend(border.opacity(0.15)).to_hex().into())
}

fn table_header_from_legacy_colors(map: &Map<String, Value>) -> Option<SharedString> {
    first(
        map,
        &[
            "table_header_bg",
            "table.head.background",
            "list.head.background",
            "list.background",
            "background",
        ],
    )
}

fn row_selected_from_legacy_colors(map: &Map<String, Value>) -> Option<SharedString> {
    first(
        map,
        &[
            "row_selected",
            "sidebar.accent.background",
            "accent.background",
        ],
    )
}

fn row_hover_from_legacy_colors(map: &Map<String, Value>) -> Option<SharedString> {
    if let Some(hover) = first(map, &["row_hover"]) {
        return Some(hover);
    }

    let selected = row_selected_from_legacy_colors(map)?;
    let selected = try_parse_color(selected.as_ref()).ok()?;
    Some(selected.opacity(0.8).to_hex().into())
}

fn table_selected_border_from_legacy_colors(map: &Map<String, Value>) -> Option<SharedString> {
    if let Some(border) = first(
        map,
        &[
            "table_row_selected_border",
            "table.active.border",
            "list.active.border",
        ],
    ) {
        return Some(border);
    }

    // Preserve the legacy list/table selection-outline fallback for old theme
    // files. Canonical files use `bg`/`accent`, so this branch cannot invent a
    // serialized role for a new file that intentionally omitted it.
    let background = map.get("background")?.as_str()?;
    let primary = map.get("primary.background")?.as_str()?;
    let background = try_parse_color(background).ok()?;
    let primary = try_parse_color(primary).ok()?;
    Some(background.blend(primary.opacity(0.6)).to_hex().into())
}

impl<'de> Deserialize<'de> for ThemeConfigColors {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = Map::<String, Value>::deserialize(deserializer)?;
        Ok(Self {
            bg: first(&map, &["bg", "background"]),
            surface: surface_from_legacy_colors(&map),
            surface_raised: first(
                &map,
                &["surface_raised", "secondary.background", "muted.background"],
            ),
            surface_raised_hover: first(&map, &["surface_raised_hover"]),
            surface_card: first(&map, &["surface_card", "background"]),
            surface_dialog: first(
                &map,
                &["surface_dialog", "popover.background", "background"],
            ),
            surface_overlay: first(
                &map,
                &["surface_overlay", "popover.background", "background"],
            ),
            band: first(&map, &["band"]),
            input_bg: first(&map, &["input_bg"]),
            text: first(&map, &["text", "foreground"]),
            text_muted: first(&map, &["text_muted", "muted.foreground"]),
            text_faint: first(&map, &["text_faint", "tab.foreground", "muted.foreground"]),
            text_dim: first(&map, &["text_dim"]),
            border: first(&map, &["border"]),
            border_strong: first(
                &map,
                &["border_strong", "hairline.strong", "hairline", "border"],
            ),
            accent: first(&map, &["accent", "primary.background", "primary"]),
            accent_strong: first(&map, &["accent_strong", "primary.background", "primary"]),
            on_accent: first(&map, &["on_accent", "primary.foreground"]),
            solid: first(&map, &["solid"]),
            on_solid: first(&map, &["on_solid"]),
            danger: first(&map, &["danger", "danger.background"]),
            danger_muted: first(&map, &["danger_muted"]),
            danger_strong: first(&map, &["danger_strong", "danger.background"]),
            warning: first(&map, &["warning", "warning.background"]),
            warning_muted: first(&map, &["warning_muted", "warning.background"]),
            success: first(&map, &["success", "success.background"]),
            success_muted: first(&map, &["success_muted", "success.background"]),
            busy: first(&map, &["busy", "info.background", "info"]),
            focus: first(&map, &["focus", "ring"]),
            row_hover: row_hover_from_legacy_colors(&map),
            row_selected: row_selected_from_legacy_colors(&map),
            table_header_bg: table_header_from_legacy_colors(&map),
            table_row_hover: first(
                &map,
                &[
                    "table_row_hover",
                    "table.hover.background",
                    "list.hover.background",
                ],
            ),
            table_row_selected: first(
                &map,
                &[
                    "table_row_selected",
                    "table.active.background",
                    "list.active.background",
                ],
            ),
            table_row_selected_border: table_selected_border_from_legacy_colors(&map),
            selection: first(&map, &["selection", "selection.background"]),
            caret: first(&map, &["caret"]),
            cursor: first(&map, &["cursor"]),
        })
    }
}

impl Serialize for ThemeConfigColors {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        for (key, value) in [
            ("bg", &self.bg),
            ("surface", &self.surface),
            ("surface_raised", &self.surface_raised),
            ("surface_raised_hover", &self.surface_raised_hover),
            ("surface_card", &self.surface_card),
            ("surface_dialog", &self.surface_dialog),
            ("surface_overlay", &self.surface_overlay),
            ("band", &self.band),
            ("input_bg", &self.input_bg),
            ("text", &self.text),
            ("text_muted", &self.text_muted),
            ("text_faint", &self.text_faint),
            ("text_dim", &self.text_dim),
            ("border", &self.border),
            ("border_strong", &self.border_strong),
            ("accent", &self.accent),
            ("accent_strong", &self.accent_strong),
            ("on_accent", &self.on_accent),
            ("solid", &self.solid),
            ("on_solid", &self.on_solid),
            ("danger", &self.danger),
            ("danger_muted", &self.danger_muted),
            ("danger_strong", &self.danger_strong),
            ("warning", &self.warning),
            ("warning_muted", &self.warning_muted),
            ("success", &self.success),
            ("success_muted", &self.success_muted),
            ("busy", &self.busy),
            ("focus", &self.focus),
            ("row_hover", &self.row_hover),
            ("row_selected", &self.row_selected),
            ("table_header_bg", &self.table_header_bg),
            ("table_row_hover", &self.table_row_hover),
            ("table_row_selected", &self.table_row_selected),
            ("table_row_selected_border", &self.table_row_selected_border),
            ("selection", &self.selection),
            ("caret", &self.caret),
            ("cursor", &self.cursor),
        ] {
            if let Some(v) = value {
                map.serialize_entry(key, v)?;
            }
        }
        map.end()
    }
}

fn parse_role(value: &Option<SharedString>) -> Option<Hsla> {
    value
        .as_ref()
        .and_then(|v| try_parse_color(v.as_ref()).ok())
}

impl ThemeConfigColors {
    /// Apply present, parseable role overrides. Missing or invalid values keep
    /// the mode fallback already on `theme`.
    fn apply_roles_to(&self, theme: &mut Theme) {
        let assign = |slot: &mut Hsla, value: &Option<SharedString>| {
            if let Some(v) = parse_role(value) {
                *slot = v;
            }
        };
        assign(&mut theme.bg, &self.bg);
        assign(&mut theme.surface, &self.surface);
        assign(&mut theme.surface_raised, &self.surface_raised);
        assign(&mut theme.surface_raised_hover, &self.surface_raised_hover);
        assign(&mut theme.surface_card, &self.surface_card);
        assign(&mut theme.surface_dialog, &self.surface_dialog);
        assign(&mut theme.surface_overlay, &self.surface_overlay);
        assign(&mut theme.band, &self.band);
        assign(&mut theme.input_bg, &self.input_bg);
        assign(&mut theme.text, &self.text);
        assign(&mut theme.text_muted, &self.text_muted);
        assign(&mut theme.text_faint, &self.text_faint);
        assign(&mut theme.text_dim, &self.text_dim);
        assign(&mut theme.border, &self.border);
        assign(&mut theme.border_strong, &self.border_strong);
        assign(&mut theme.accent, &self.accent);
        assign(&mut theme.accent_strong, &self.accent_strong);
        assign(&mut theme.on_accent, &self.on_accent);
        assign(&mut theme.solid, &self.solid);
        assign(&mut theme.on_solid, &self.on_solid);
        assign(&mut theme.danger, &self.danger);
        assign(&mut theme.danger_muted, &self.danger_muted);
        assign(&mut theme.danger_strong, &self.danger_strong);
        assign(&mut theme.warning, &self.warning);
        assign(&mut theme.warning_muted, &self.warning_muted);
        assign(&mut theme.success, &self.success);
        assign(&mut theme.success_muted, &self.success_muted);
        assign(&mut theme.busy, &self.busy);
        if let Some(v) = parse_role(&self.focus) {
            theme.focus = v;
        } else {
            theme.focus = theme.accent;
        }
        assign(&mut theme.row_hover, &self.row_hover);
        assign(&mut theme.row_selected, &self.row_selected);
        if let Some(v) = parse_role(&self.table_header_bg) {
            theme.table_header_bg = v;
        } else {
            theme.table_header_bg = theme.bg;
        }
        assign(&mut theme.table_row_hover, &self.table_row_hover);
        if let Some(v) = parse_role(&self.table_row_selected) {
            theme.table_row_selected = v;
        } else {
            theme.table_row_selected = theme.accent.opacity(0.18);
        }
        if let Some(v) = parse_role(&self.table_row_selected_border) {
            theme.table_row_selected_border = v;
        } else {
            theme.table_row_selected_border = theme.accent;
        }
        assign(&mut theme.selection, &self.selection);
        if let Some(v) = parse_role(&self.caret) {
            theme.caret = v;
        } else {
            theme.caret = theme.text;
        }
        assign(&mut theme.cursor, &self.cursor);
    }
}

impl Theme {
    /// Apply the given theme configuration to the current theme.
    pub fn apply_config(&mut self, config: &Rc<ThemeConfig>) {
        let light_theme = if config.mode.is_dark() {
            self.light_theme.clone()
        } else {
            config.clone()
        };
        let dark_theme = if config.mode.is_dark() {
            config.clone()
        } else {
            self.dark_theme.clone()
        };
        let highlight_theme = if let Some(style) = &config.highlight {
            Arc::new(HighlightTheme {
                name: config.name.to_string(),
                appearance: config.mode,
                style: style.clone(),
            })
        } else {
            self.highlight_theme.clone()
        };
        let notification = self.notification.clone();
        let list = self.list.clone();
        let sheet = self.sheet.clone();
        let scrollbar_mode = self.scrollbar_mode;
        let tile_grid_size = self.tile_grid_size;
        let tile_shadow = self.tile_shadow;
        let tile_radius = self.tile_radius;
        let focus_ring = self.focus_ring;
        let transparent = self.transparent;

        *self = if config.mode.is_dark() {
            Theme::fallback_dark()
        } else {
            Theme::fallback_light()
        };

        self.light_theme = light_theme;
        self.dark_theme = dark_theme;
        self.highlight_theme = highlight_theme;
        self.notification = notification;
        self.list = list;
        self.sheet = sheet;
        self.scrollbar_mode = scrollbar_mode;
        self.tile_grid_size = tile_grid_size;
        self.tile_shadow = tile_shadow;
        self.tile_radius = tile_radius;
        self.focus_ring = focus_ring;
        self.transparent = transparent;
        self.mode = config.mode;

        if let Some(font_size) = config.font_size {
            self.font_size = px(font_size);
        }
        if let Some(font_family) = &config.font_family {
            self.font_family = font_family.clone();
        }
        if let Some(mono_font_family) = &config.mono_font_family {
            self.mono_font_family = mono_font_family.clone();
        }
        if let Some(mono_font_size) = config.mono_font_size {
            self.mono_font_size = px(mono_font_size);
        }
        if let Some(radius) = config.radius {
            self.radius = px(radius as f32);
            if radius == 0 && config.radius_lg.is_none() {
                self.radius_lg = px(0.);
            }
        }
        if let Some(radius_lg) = config.radius_lg {
            self.radius_lg = px(radius_lg as f32);
        }
        if let Some(shadow) = config.shadow {
            self.shadow = shadow;
        }

        config.colors.apply_roles_to(self);
    }
}

#[cfg(test)]
mod tests {
    use gpui::px;

    use crate::{Theme, ThemeConfig, ThemeMode, ThemeSet, try_parse_color};

    fn canonical_color_role_names() -> Vec<&'static str> {
        vec![
            "accent",
            "accent_strong",
            "band",
            "bg",
            "border",
            "border_strong",
            "busy",
            "caret",
            "cursor",
            "danger",
            "danger_muted",
            "danger_strong",
            "focus",
            "input_bg",
            "on_accent",
            "on_solid",
            "row_hover",
            "row_selected",
            "selection",
            "solid",
            "success",
            "success_muted",
            "surface",
            "surface_card",
            "surface_dialog",
            "surface_overlay",
            "surface_raised",
            "surface_raised_hover",
            "table_header_bg",
            "table_row_hover",
            "table_row_selected",
            "table_row_selected_border",
            "text",
            "text_dim",
            "text_faint",
            "text_muted",
            "warning",
            "warning_muted",
        ]
    }

    #[test]
    fn shipped_schema_and_default_themes_expose_only_canonical_color_roles() {
        let schema: serde_json::Value =
            serde_json::from_str(include_str!("../../../../.theme-schema.json")).unwrap();
        let mut schema_keys = schema["$defs"]["ThemeConfigColors"]["properties"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>();
        schema_keys.sort_unstable();
        assert_eq!(schema_keys, canonical_color_role_names());
        assert_eq!(
            schema["$defs"]["ThemeConfigColors"]["additionalProperties"], false,
            "the canonical schema must reject retired or misspelled color keys"
        );

        let default: serde_json::Value =
            serde_json::from_str(include_str!("default-theme.json")).unwrap();
        assert_eq!(default["$schema"], "../../../../.theme-schema.json");
        for theme in default["themes"].as_array().unwrap() {
            let mut keys = theme["colors"]
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>();
            keys.sort_unstable();
            assert_eq!(keys, canonical_color_role_names());
        }
    }

    #[test]
    fn old_json_loads_and_new_json_drops_list_hover() {
        let raw = r##"{
        "name": "Fixture",
        "themes": [{
            "name": "Fixture Dark",
            "mode": "dark",
            "colors": {
                "background": "#191919",
                "foreground": "#E6E6E3",
                "sidebar.background": "#202020",
                "primary.background": "#89A6AD",
                "primary.foreground": "#191919",
                "muted.foreground": "#A1A09C",
                "popover.background": "#202020",
                "border": "#303030",
                "list.hover.background": "#262626"
            }
        }]
    }"##;
        let set: ThemeSet = serde_json::from_str(raw).unwrap();
        let c = &set.themes[0].colors;
        assert_eq!(c.bg.as_deref(), Some("#191919"));
        assert_eq!(c.surface.as_deref(), Some("#202020"));
        assert_eq!(c.accent_strong.as_deref(), Some("#89A6AD"));
        let out = serde_json::to_value(&set).unwrap();
        let colors = &out["themes"][0]["colors"];
        assert!(colors.get("bg").is_some());
        assert!(colors.get("list.hover.background").is_none());
        assert!(colors.get("background").is_none());
    }

    #[test]
    fn legacy_theme_without_sidebar_preserves_derived_shell_surface() {
        let config = serde_json::from_value::<ThemeConfig>(serde_json::json!({
            "name": "Catppuccin Frappe Legacy Shape",
            "mode": "dark",
            "colors": {
                "background": "#232634",
                "border": "#3e4255",
                "tab_bar.background": "#1E202B"
            }
        }))
        .unwrap();

        assert_eq!(config.colors.surface.as_deref(), Some("#272A38"));

        let mut theme = Theme::default();
        theme.apply_config(&std::rc::Rc::new(config));
        assert_eq!(theme.surface, try_parse_color("#272A38").unwrap());
    }

    #[test]
    fn legacy_table_header_uses_the_resolved_list_header_plane() {
        let colors = serde_json::from_value::<super::ThemeConfigColors>(serde_json::json!({
            "background": "#151515",
            "tab_bar.background": "#101010"
        }))
        .unwrap();

        let canonical = serde_json::to_value(colors).unwrap();
        assert_eq!(canonical["table_header_bg"], "#151515");
    }

    #[test]
    fn legacy_sidebar_accent_becomes_independent_row_states() {
        let colors = serde_json::from_value::<super::ThemeConfigColors>(serde_json::json!({
            "accent.background": "#3b3f4f",
            "sidebar.accent.background": "#424656"
        }))
        .unwrap();

        let canonical = serde_json::to_value(colors).unwrap();
        assert_eq!(canonical["row_hover"], "#424656CC");
        assert_eq!(canonical["row_selected"], "#424656");
    }

    #[test]
    fn legacy_accent_seeds_row_states_when_sidebar_accent_is_absent() {
        let colors = serde_json::from_value::<super::ThemeConfigColors>(serde_json::json!({
            "accent.background": "#3b3f4f"
        }))
        .unwrap();

        let canonical = serde_json::to_value(colors).unwrap();
        assert_eq!(canonical["row_hover"], "#3B3F4FCC");
        assert_eq!(canonical["row_selected"], "#3b3f4f");
    }

    #[test]
    fn legacy_table_selected_border_stays_independent_from_primary() {
        let colors = serde_json::from_value::<super::ThemeConfigColors>(serde_json::json!({
            "background": "#1f1d45",
            "primary.background": "#5f72c6",
            "table.active.border": "#549235"
        }))
        .unwrap();

        let canonical = serde_json::to_value(colors).unwrap();
        assert_eq!(canonical["table_row_selected_border"], "#549235");
    }

    #[test]
    fn legacy_table_interactions_fall_back_to_list_roles() {
        let colors = serde_json::from_value::<super::ThemeConfigColors>(serde_json::json!({
            "list.hover.background": "#262626",
            "list.active.background": "#384152"
        }))
        .unwrap();

        let canonical = serde_json::to_value(colors).unwrap();
        assert_eq!(canonical["table_row_hover"], "#262626");
        assert_eq!(canonical["table_row_selected"], "#384152");
    }

    #[test]
    fn zero_general_radius_also_squares_large_surfaces_when_unspecified() {
        let config = serde_json::from_value::<ThemeConfig>(serde_json::json!({
            "name": "Square",
            "mode": "dark",
            "radius": 0
        }))
        .unwrap();
        let mut theme = Theme::default();
        theme.apply_config(&std::rc::Rc::new(config));

        assert_eq!(theme.radius, px(0.));
        assert_eq!(theme.radius_lg, px(0.));
    }

    #[test]
    fn apply_config_maps_old_keys_to_roles() {
        let config = serde_json::from_value::<ThemeConfig>(serde_json::json!({
            "name": "Legacy",
            "mode": "dark",
            "radius": 7,
            "colors": {
                "background": "#191919",
                "foreground": "#E6E6E3",
                "sidebar.background": "#202020",
                "primary.background": "#89A6AD",
                "primary.foreground": "#191919",
                "muted.foreground": "#A1A09C",
                "popover.background": "#303030",
                "border": "#404040",
                "ring": "#2383E2",
                "table.hover.background": "#2F2F2F",
                "table.active.background": "#2383E21F",
                "caret": "#FFFFFF"
            }
        }))
        .unwrap();
        let mut theme = Theme::default();
        theme.apply_config(&std::rc::Rc::new(config));

        assert_eq!(theme.bg, try_parse_color("#191919").unwrap());
        assert_eq!(theme.surface, try_parse_color("#202020").unwrap());
        assert_eq!(theme.surface_card, try_parse_color("#191919").unwrap());
        assert_eq!(theme.surface_overlay, try_parse_color("#303030").unwrap());
        assert_eq!(theme.surface_dialog, try_parse_color("#303030").unwrap());
        assert_eq!(theme.text, try_parse_color("#E6E6E3").unwrap());
        assert_eq!(theme.text_muted, try_parse_color("#A1A09C").unwrap());
        assert_eq!(theme.accent, try_parse_color("#89A6AD").unwrap());
        assert_eq!(theme.accent_strong, try_parse_color("#89A6AD").unwrap());
        assert_eq!(theme.on_accent, try_parse_color("#191919").unwrap());
        assert_eq!(theme.border, try_parse_color("#404040").unwrap());
        assert_eq!(theme.focus, try_parse_color("#2383E2").unwrap());
        assert_eq!(theme.table_row_hover, try_parse_color("#2F2F2F").unwrap());
        assert_eq!(
            theme.table_row_selected,
            try_parse_color("#2383E21F").unwrap()
        );
        assert_eq!(theme.caret, try_parse_color("#FFFFFF").unwrap());
        assert_eq!(theme.radius, px(7.));
        assert_eq!(theme.mode, ThemeMode::Dark);
    }

    #[test]
    fn apply_config_skips_invalid_color_and_keeps_fallback() {
        let fallback = Theme::fallback_light();
        let config = serde_json::from_value::<ThemeConfig>(serde_json::json!({
            "name": "Broken",
            "mode": "light",
            "colors": {
                "background": "not-a-color",
                "foreground": "#111111"
            }
        }))
        .unwrap();
        let mut theme = Theme::default();
        theme.apply_config(&std::rc::Rc::new(config));
        assert_eq!(theme.bg, fallback.bg);
        assert_eq!(theme.text, try_parse_color("#111111").unwrap());
        assert_eq!(theme.caret, theme.text);
        assert_eq!(theme.focus, fallback.focus);
    }

    #[test]
    fn canonical_interaction_roles_round_trip() {
        let raw = serde_json::json!({
            "focus": "#2383E2",
            "table_row_hover": "#2F2F2F",
            "table_row_selected": "#2383E21F"
        });
        let colors: super::ThemeConfigColors = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(serde_json::to_value(colors).unwrap(), raw);
    }

    #[test]
    fn test_aurora_theme_parses_and_maps_roles() {
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

        assert_eq!(theme.bg, try_parse_color("#FFFFFF").unwrap());
        assert_eq!(theme.border, try_parse_color("#E2E8F0").unwrap());
        assert_eq!(theme.mode, ThemeMode::Light);
    }
}
