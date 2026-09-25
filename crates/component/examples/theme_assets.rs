use std::{path::Path, rc::Rc};

use anyhow::{Context as _, Result, bail};
use gpui_neath::{Colorize as _, Theme, ThemeConfig, ThemeSet};
use serde_json::{Map, Value};

fn resolved_colors(config: &ThemeConfig) -> Result<Value> {
    let mut theme = Theme::default();
    theme.apply_config(&Rc::new(config.clone()));

    let mut colors = Map::new();
    for property in theme.resolved_color_properties()? {
        colors.insert(property.key.to_string(), property.color.to_hex().into());
    }
    Ok(colors.into())
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
            .insert("colors".to_string(), resolved_colors(config)?);
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
