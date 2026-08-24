use std::{path::Path, rc::Rc};

use anyhow::{Context as _, Result, bail};
use gpui_neath::{Colorize as _, Theme, ThemeConfig, ThemeSet};
use serde_json::{Value, json};

fn resolved_colors(config: &ThemeConfig) -> Value {
    let mut theme = if config.mode.is_dark() {
        Theme::fallback_dark()
    } else {
        Theme::fallback_light()
    };
    theme.apply_config(&Rc::new(config.clone()));

    json!({
        "bg": theme.bg.to_hex(),
        "surface": theme.surface.to_hex(),
        "surface_raised": theme.surface_raised.to_hex(),
        "surface_raised_hover": theme.surface_raised_hover.to_hex(),
        "surface_card": theme.surface_card.to_hex(),
        "surface_dialog": theme.surface_dialog.to_hex(),
        "surface_overlay": theme.surface_overlay.to_hex(),
        "band": theme.band.to_hex(),
        "input_bg": theme.input_bg.to_hex(),
        "text": theme.text.to_hex(),
        "text_muted": theme.text_muted.to_hex(),
        "text_faint": theme.text_faint.to_hex(),
        "text_dim": theme.text_dim.to_hex(),
        "border": theme.border.to_hex(),
        "border_strong": theme.border_strong.to_hex(),
        "accent": theme.accent.to_hex(),
        "accent_strong": theme.accent_strong.to_hex(),
        "on_accent": theme.on_accent.to_hex(),
        "solid": theme.solid.to_hex(),
        "on_solid": theme.on_solid.to_hex(),
        "danger": theme.danger.to_hex(),
        "danger_muted": theme.danger_muted.to_hex(),
        "danger_strong": theme.danger_strong.to_hex(),
        "warning": theme.warning.to_hex(),
        "warning_muted": theme.warning_muted.to_hex(),
        "success": theme.success.to_hex(),
        "success_muted": theme.success_muted.to_hex(),
        "busy": theme.busy.to_hex(),
        "focus": theme.focus.to_hex(),
        "row_hover": theme.row_hover.to_hex(),
        "row_selected": theme.row_selected.to_hex(),
        "table_header_bg": theme.table_header_bg.to_hex(),
        "table_row_hover": theme.table_row_hover.to_hex(),
        "table_row_selected": theme.table_row_selected.to_hex(),
        "table_row_selected_border": theme.table_row_selected_border.to_hex(),
        "selection": theme.selection.to_hex(),
        "caret": theme.caret.to_hex(),
        "cursor": theme.cursor.to_hex(),
    })
}

fn write_schema(path: &Path) -> Result<()> {
    let schema = schemars::schema_for!(ThemeSet);
    let json = serde_json::to_string_pretty(&schema).context("serialize theme schema")?;
    std::fs::write(path, format!("{json}\n"))
        .with_context(|| format!("write theme schema {}", path.display()))
}

fn canonicalize_default(source: &Path, destination: &Path) -> Result<()> {
    let source_json = std::fs::read_to_string(source)
        .with_context(|| format!("read default theme {}", source.display()))?;
    let typed: ThemeSet = serde_json::from_str(&source_json).context("parse default theme")?;
    let mut raw: Value = serde_json::from_str(&source_json).context("parse raw default theme")?;
    let raw_themes = raw
        .get_mut("themes")
        .and_then(Value::as_array_mut)
        .context("default theme is missing themes array")?;
    if raw_themes.len() != typed.themes.len() {
        bail!("raw and typed default theme counts differ");
    }

    for (raw_theme, config) in raw_themes.iter_mut().zip(&typed.themes) {
        raw_theme
            .as_object_mut()
            .context("default theme entry is not an object")?
            .insert("colors".to_string(), resolved_colors(config));
    }

    let json = serde_json::to_string_pretty(&raw).context("serialize canonical default theme")?;
    std::fs::write(destination, format!("{json}\n"))
        .with_context(|| format!("write canonical default theme {}", destination.display()))
}

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [command, destination] if command == "schema" => write_schema(Path::new(destination)),
        [command, source, destination] if command == "canonicalize-default" => {
            canonicalize_default(Path::new(source), Path::new(destination))
        }
        _ => bail!(
            "usage: theme_assets schema <destination> | canonicalize-default <source> <destination>"
        ),
    }
}
