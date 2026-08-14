use std::path::{Path, PathBuf};

const BASE_LAYER_FORBIDDEN: &[&str] = &["gpui_neath", "gpui-neath"];

// These are top-level declarations. Requiring column zero avoids rejecting
// unrelated base methods such as `Slider::value`.
const BASE_DECLARATION_FORBIDDEN: &[&str] = &[
    "pub const TEXT_10",
    "pub const TEXT_12",
    "pub const TEXT_13",
    "pub const TEXT_14",
    "pub const TEXT_15",
    "pub const TEXT_16",
    "pub const SPACE_HALF",
    "pub const SPACE_1",
    "pub const SPACE_1P5",
    "pub const SPACE_2",
    "pub const SPACE_3",
    "pub const SPACE_4",
    "pub const SPACE_6",
    "pub const ACCENT_RIM_PX",
    "pub const RADIUS_SM",
    "pub const RADIUS_MD",
    "pub const RADIUS_LG",
    "pub const RADIUS_PILL",
    "pub const ICON_INLINE",
    "pub const ICON_CHROME",
    "pub const ICON_PRIMARY",
    "pub const MENU_BG_ALPHA",
    "pub fn bg_sunken",
    "pub fn bg_active",
    "pub fn accent_soft",
    "pub fn truncating_cell",
    "pub fn truncating_cell_sized",
    "pub fn middle_truncating_cell_sized",
    "pub fn section_label",
    "pub fn required_label",
    "pub fn body(",
    "pub fn body_muted",
    "pub fn dialog_prose",
    "pub fn value(",
    "pub fn value_dense",
    "pub fn caption",
    "pub fn control_label",
    "pub fn control_body",
    "pub fn on_surface_fill",
    "pub fn on_surface_border",
    "pub fn compact_menu",
    "pub fn overlay_menu",
    "pub fn tool_popover",
    "pub fn popover_rule",
    "pub fn popover_row",
    "pub trait TruncateMiddleExt",
];

const APP_IMPORT_FORBIDDEN: &[&str] = &[
    "neath = {",
    "neath.workspace",
    "gpui_canvas_controls",
    "gpui-canvas-controls",
    "gpui_waveform",
    "gpui-waveform",
    "waapi",
    "ptsl",
    "neath_core",
    "neath-core",
    "neath_dsp",
    "neath-dsp",
    "neath_fx",
    "neath-fx",
    "neath_embed",
    "neath-embed",
    "neath_gui",
    "neath-gui",
    "neath_cli",
    "neath-cli",
    "neath_agent",
    "neath-agent",
    "neath_agent_eval",
    "neath-agent-eval",
    "neath_gen",
    "neath-gen",
    "signalsmith_stretch_rs",
    "signalsmith-stretch-rs",
    "ucs",
    "neath_update_helper",
    "neath-update-helper",
];

fn uncommented(line: &str) -> &str {
    let line = line.split_once("//").map_or(line, |(code, _)| code);
    line.split_once('#').map_or(line, |(code, _)| code)
}

fn violations(path: &Path, source: &str, forbidden: &[&str]) -> Vec<String> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line = uncommented(line);
            forbidden
                .iter()
                .find(|needle| line.contains(**needle))
                .map(|needle| format!("{}:{} contains {needle}", path.display(), index + 1))
        })
        .collect()
}

fn base_violations(path: &Path, source: &str) -> Vec<String> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line = uncommented(line);
            let found = BASE_LAYER_FORBIDDEN
                .iter()
                .find(|needle| line.contains(**needle))
                .or_else(|| {
                    BASE_DECLARATION_FORBIDDEN
                        .iter()
                        .find(|needle| line.starts_with(**needle))
                });
            found.map(|needle| format!("{}:{} contains {needle}", path.display(), index + 1))
        })
        .collect()
}

#[test]
fn checker_accepts_clean_base_and_styled_fixtures() {
    assert!(
        base_violations(
            Path::new("crates/base/src/fake.rs"),
            "pub fn neutral_layout() {}",
        )
        .is_empty(),
    );
    assert!(
        violations(
            Path::new("crates/ui/src/fake.rs"),
            "use gpui::{App, div};",
            APP_IMPORT_FORBIDDEN,
        )
        .is_empty(),
    );
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("readable source directory") {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn checker_rejects_a_styled_recipe_in_base_fixture() {
    let found = base_violations(
        Path::new("crates/base/src/fake.rs"),
        "pub fn tool_popover() {}",
    );
    assert_eq!(found.len(), 1);
}

#[test]
fn checker_rejects_a_neath_workspace_import_in_styled_fixture() {
    let found = violations(
        Path::new("crates/ui/src/fake.rs"),
        "use neath_core::SourceId;",
        APP_IMPORT_FORBIDDEN,
    );
    assert_eq!(found.len(), 1);
}

#[test]
fn style_kernel_stays_above_base_and_below_the_application() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut failures = Vec::new();

    let mut base_files = Vec::new();
    rust_files(&manifest_dir.join("../base/src"), &mut base_files);
    for file in base_files {
        let source = std::fs::read_to_string(&file).expect("readable base source");
        failures.extend(base_violations(&file, &source));
    }

    let base_manifest = manifest_dir.join("../base/Cargo.toml");
    let base_manifest_source =
        std::fs::read_to_string(&base_manifest).expect("readable base manifest");
    failures.extend(violations(
        &base_manifest,
        &base_manifest_source,
        BASE_LAYER_FORBIDDEN,
    ));

    let mut styled_files = Vec::new();
    rust_files(&manifest_dir.join("src"), &mut styled_files);
    for file in styled_files {
        let source = std::fs::read_to_string(&file).expect("readable styled source");
        failures.extend(violations(&file, &source, APP_IMPORT_FORBIDDEN));
    }

    let styled_manifest = manifest_dir.join("Cargo.toml");
    let manifest_source =
        std::fs::read_to_string(&styled_manifest).expect("readable styled manifest");
    failures.extend(violations(
        &styled_manifest,
        &manifest_source,
        APP_IMPORT_FORBIDDEN,
    ));

    let build_script = manifest_dir.join("build.rs");
    let build_source =
        std::fs::read_to_string(&build_script).expect("readable styled build script");
    failures.extend(violations(
        &build_script,
        &build_source,
        APP_IMPORT_FORBIDDEN,
    ));

    assert!(
        failures.is_empty(),
        "style ownership boundary violations:\n{}",
        failures.join("\n"),
    );
}
