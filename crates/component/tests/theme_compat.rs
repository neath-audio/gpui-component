use gpui_neath::theme::{ThemeConfig, ThemeConfigColors, ThemeMode};

#[test]
fn legacy_chart_keys_load_and_serialize_with_current_names() {
    let colors: ThemeConfigColors = serde_json::from_value(serde_json::json!({
        "chart_bullish": "#12ab34",
        "chart_bearish": "#cd3456"
    }))
    .unwrap();
    assert_eq!(colors.chart_bullish.as_deref(), Some("#12ab34"));
    assert_eq!(colors.chart_bearish.as_deref(), Some("#cd3456"));
    let encoded = serde_json::to_value(colors).unwrap();
    assert_eq!(encoded["chart.bullish"], "#12ab34");
    assert_eq!(encoded["chart.bearish"], "#cd3456");
    assert!(encoded.get("chart_bullish").is_none());
}

#[test]
fn legacy_text_view_style_struct_literal_shape_is_unchanged() {
    use gpui_neath::text::TextViewStyle;

    let defaults = TextViewStyle::default();
    let _ = TextViewStyle {
        paragraph_gap: defaults.paragraph_gap,
        heading_base_font_size: defaults.heading_base_font_size,
        heading_font_size: defaults.heading_font_size,
        highlight_theme: defaults.highlight_theme,
        code_block: defaults.code_block,
        table: defaults.table,
        table_head: defaults.table_head,
        table_cell: defaults.table_cell,
        inline_code: defaults.inline_code,
        is_dark: defaults.is_dark,
    };
}

#[test]
fn legacy_theme_config_struct_literal_shape_is_unchanged() {
    let _ = ThemeConfig {
        translucency: Default::default(),
        is_default: false,
        name: "Compatibility".into(),
        mode: ThemeMode::Light,
        font_size: None,
        font_family: None,
        mono_font_family: None,
        mono_font_size: None,
        radius: None,
        radius_lg: None,
        shadow: None,
        colors: ThemeConfigColors::default(),
        highlight: None,
    };
}
