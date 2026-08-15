use std::path::{Path, PathBuf};

const BASE_LAYER_FORBIDDEN: &[&str] = &["gpui_neath", "gpui-neath"];

// `value` is a common base method name, so it remains a top-level-only match.
// The other names are style-kernel-only and must not escape through indentation
// or crate-private visibility.
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
    "package = \"neath\"",
    "[dependencies.neath]",
    "neath::",
    "use neath::",
    "pub use neath::",
    "extern crate neath",
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

fn uncommented<'a>(path: &Path, line: &'a str) -> &'a str {
    if path
        .extension()
        .is_some_and(|extension| extension == "toml")
    {
        line.split_once('#').map_or(line, |(code, _)| code)
    } else {
        // `#` introduces attributes in Rust; it is not a Rust comment marker.
        line.split_once("//").map_or(line, |(code, _)| code)
    }
}

fn without_whitespace(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn contains_crate_path(line: &str, crate_name: &str) -> bool {
    let path = format!("{crate_name}::");
    line.match_indices(&path).any(|(index, _)| {
        line[..index]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
    })
}

fn violations(path: &Path, source: &str, forbidden: &[&str]) -> Vec<String> {
    let is_manifest = path
        .extension()
        .is_some_and(|extension| extension == "toml");

    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line = uncommented(path, line);
            let compact;
            let line = if is_manifest {
                compact = without_whitespace(line);
                &compact
            } else {
                line
            };
            forbidden
                .iter()
                .find(|needle| {
                    if is_manifest {
                        line.contains(&without_whitespace(needle))
                    } else if **needle == "neath::" {
                        contains_crate_path(line, "neath")
                    } else {
                        line.contains(**needle)
                    }
                })
                .map(|needle| format!("{}:{} contains {needle}", path.display(), index + 1))
        })
        .collect()
}

fn is_base_declaration(line: &str, needle: &str) -> bool {
    if needle == "pub fn value(" {
        return line.starts_with(needle);
    }

    let line = line.trim_start();
    if line.starts_with(needle) {
        return true;
    }

    line.strip_prefix("pub(")
        .and_then(|visibility| {
            visibility
                .split_once(')')
                .map(|(_, rest)| rest.trim_start())
        })
        .zip(needle.strip_prefix("pub "))
        .is_some_and(|(declaration, needle)| declaration.starts_with(needle))
}

fn base_violations(path: &Path, source: &str) -> Vec<String> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line = uncommented(path, line);
            let found = BASE_LAYER_FORBIDDEN
                .iter()
                .find(|needle| line.contains(**needle))
                .or_else(|| {
                    BASE_DECLARATION_FORBIDDEN
                        .iter()
                        .find(|needle| is_base_declaration(line, needle))
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
fn checker_rejects_a_direct_neath_workspace_import_in_styled_fixture() {
    let found = violations(
        Path::new("crates/ui/src/fake.rs"),
        "use neath::SourceId;",
        APP_IMPORT_FORBIDDEN,
    );
    assert_eq!(found.len(), 1);
}

#[test]
fn checker_rejects_a_neath_package_alias_in_styled_manifest_fixture() {
    let found = violations(
        Path::new("crates/ui/Cargo.toml"),
        "app_library = { package = \"neath\", workspace = true }",
        APP_IMPORT_FORBIDDEN,
    );
    assert_eq!(found.len(), 1);
}

#[test]
fn checker_rejects_compact_neath_dependency_syntax_in_styled_fixtures() {
    for (path, source) in [
        (
            Path::new("crates/ui/src/fake.rs"),
            "type Leak = neath::SourceId;",
        ),
        (
            Path::new("crates/ui/src/fake.rs"),
            "use { neath::SourceId };",
        ),
        (Path::new("crates/ui/Cargo.toml"), "[dependencies.neath]"),
        (
            Path::new("crates/ui/Cargo.toml"),
            "app={package=\"neath\",path=\"../../../neath\"}",
        ),
        (
            Path::new("crates/ui/Cargo.toml"),
            "app = { package = \"neath\", path = \"../../../neath\" }",
        ),
    ] {
        let found = violations(path, source, APP_IMPORT_FORBIDDEN);
        assert_eq!(found.len(), 1, "fixture should be rejected: {source}");
    }
}

#[test]
fn checker_rejects_indented_and_restricted_styled_recipes_in_base_fixtures() {
    for source in [
        "    pub fn tool_popover() {}",
        "pub(crate) fn tool_popover() {}",
        "pub(super) fn tool_popover() {}",
        "pub(in crate) fn tool_popover() {}",
    ] {
        let found = base_violations(Path::new("crates/base/src/fake.rs"), source);
        assert_eq!(found.len(), 1, "fixture should be rejected: {source}");
    }
}

#[test]
fn checker_keeps_comments_urls_and_unrelated_base_methods_out_of_scope() {
    assert!(
        base_violations(
            Path::new("crates/base/src/fake.rs"),
            "    pub fn value(&self) {}",
        )
        .is_empty()
    );
    assert!(
        base_violations(
            Path::new("crates/base/src/fake.rs"),
            "/// See [the design](https://example.test/gpui_neath).",
        )
        .is_empty()
    );
    assert!(
        violations(
            Path::new("crates/ui/src/fake.rs"),
            "// use { neath::SourceId };",
            APP_IMPORT_FORBIDDEN,
        )
        .is_empty()
    );
    assert!(
        violations(
            Path::new("crates/ui/src/fake.rs"),
            "let root = \"gpui_neath::Root\";",
            APP_IMPORT_FORBIDDEN,
        )
        .is_empty()
    );
    assert!(
        violations(
            Path::new("crates/ui/Cargo.toml"),
            "# [dependencies.neath]",
            APP_IMPORT_FORBIDDEN,
        )
        .is_empty()
    );
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
