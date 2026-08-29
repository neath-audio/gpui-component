use gpui_neath::theme::{ThemeConfig, ThemeConfigColors, ThemeMode, ThemeTranslucencyConfig};

#[test]
fn theme_config_struct_literals_include_the_translucency_contract() {
    let _ = ThemeConfig {
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
        translucency: ThemeTranslucencyConfig::default(),
        colors: ThemeConfigColors::default(),
        highlight: None,
    };
}
