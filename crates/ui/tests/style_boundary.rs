use std::path::{Path, PathBuf};

use syn::{
    ext::IdentExt,
    visit::{self, Visit},
};

const BASE_LAYER_RUST_CRATES: &[&str] = &["gpui_neath"];

const BASE_CONSTANTS: &[&str] = &[
    "TEXT_10",
    "TEXT_12",
    "TEXT_13",
    "TEXT_14",
    "TEXT_15",
    "TEXT_16",
    "SPACE_HALF",
    "SPACE_1",
    "SPACE_1P5",
    "SPACE_2",
    "SPACE_3",
    "SPACE_4",
    "SPACE_6",
    "ACCENT_RIM_PX",
    "RADIUS_SM",
    "RADIUS_MD",
    "RADIUS_LG",
    "RADIUS_PILL",
    "ICON_INLINE",
    "ICON_CHROME",
    "ICON_PRIMARY",
    "MENU_BG_ALPHA",
];

const BASE_FUNCTIONS: &[&str] = &[
    "bg_sunken",
    "bg_active",
    "accent_soft",
    "truncating_cell",
    "truncating_cell_sized",
    "middle_truncating_cell_sized",
    "section_label",
    "required_label",
    "body",
    "body_muted",
    "dialog_prose",
    "value",
    "value_dense",
    "caption",
    "control_label",
    "control_body",
    "on_surface_fill",
    "on_surface_border",
    "compact_menu",
    "overlay_menu",
    "tool_popover",
    "popover_rule",
    "popover_row",
];

const BASE_TRAITS: &[&str] = &["TruncateMiddleExt"];

const NEATH_RUST_CRATES: &[&str] = &[
    "neath",
    "gpui_canvas_controls",
    "gpui_waveform",
    "waapi",
    "ptsl",
    "neath_core",
    "neath_dsp",
    "neath_fx",
    "neath_embed",
    "neath_gui",
    "neath_cli",
    "neath_agent",
    "neath_agent_eval",
    "neath_gen",
    "signalsmith_stretch_rs",
    "ucs",
    "neath_update_helper",
];

const DEPENDENCY_TABLES: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

fn is_forbidden_package(name: &str, forbidden_rust_crates: &[&str]) -> bool {
    let normalized = name.replace('-', "_");
    forbidden_rust_crates
        .iter()
        .any(|forbidden| normalized == *forbidden)
}

fn identifier_name(ident: &syn::Ident) -> String {
    ident.unraw().to_string()
}

fn record_ident(
    violations: &mut Vec<String>,
    path: &Path,
    kind: &str,
    ident: &syn::Ident,
    forbidden: &[&str],
) {
    let name = identifier_name(ident);
    if forbidden.iter().any(|candidate| name == *candidate) {
        violations.push(format!("{}: forbidden {kind} `{name}`", path.display()));
    }
}

fn record_use_branch_roots(
    violations: &mut Vec<String>,
    path: &Path,
    kind: &str,
    tree: &syn::UseTree,
    forbidden: &[&str],
) {
    match tree {
        syn::UseTree::Group(group) => {
            for tree in &group.items {
                record_use_branch_roots(violations, path, kind, tree, forbidden);
            }
        }
        syn::UseTree::Path(tree) => record_ident(violations, path, kind, &tree.ident, forbidden),
        syn::UseTree::Name(tree) => record_ident(violations, path, kind, &tree.ident, forbidden),
        syn::UseTree::Rename(tree) => record_ident(violations, path, kind, &tree.ident, forbidden),
        syn::UseTree::Glob(_) => {}
    }
}

struct ImportVisitor<'a> {
    path: &'a Path,
    forbidden: &'a [&'a str],
    violations: Vec<String>,
}

impl<'ast> Visit<'ast> for ImportVisitor<'_> {
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        record_use_branch_roots(
            &mut self.violations,
            self.path,
            "crate import",
            &item.tree,
            self.forbidden,
        );
        visit::visit_item_use(self, item);
    }

    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        record_ident(
            &mut self.violations,
            self.path,
            "extern crate",
            &item.ident,
            self.forbidden,
        );
        visit::visit_item_extern_crate(self, item);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if let Some(segment) = path.segments.first() {
            record_ident(
                &mut self.violations,
                self.path,
                "crate path",
                &segment.ident,
                self.forbidden,
            );
        }
        visit::visit_path(self, path);
    }
}

fn import_violations(path: &Path, source: &str, forbidden: &[&str]) -> Vec<String> {
    let file = match syn::parse_file(source) {
        Ok(file) => file,
        Err(error) => return vec![format!("{}: invalid Rust: {error}", path.display())],
    };
    let mut visitor = ImportVisitor {
        path,
        forbidden,
        violations: Vec::new(),
    };
    visitor.visit_file(&file);
    visitor.violations
}

struct BaseVisitor<'a> {
    path: &'a Path,
    violations: Vec<String>,
}

impl BaseVisitor<'_> {
    fn record_declaration(&mut self, kind: &str, ident: &syn::Ident) {
        let name = identifier_name(ident);
        self.violations.push(format!(
            "{}: forbidden base {kind} `{name}`",
            self.path.display()
        ));
    }

    fn check_constant(&mut self, ident: &syn::Ident) {
        if BASE_CONSTANTS.contains(&identifier_name(ident).as_str()) {
            self.record_declaration("constant", ident);
        }
    }

    fn check_function(&mut self, ident: &syn::Ident, associated: bool) {
        let name = identifier_name(ident);
        if BASE_FUNCTIONS.contains(&name.as_str()) && !(associated && name == "value") {
            self.record_declaration("function", ident);
        }
    }

    fn check_trait(&mut self, ident: &syn::Ident) {
        if BASE_TRAITS.contains(&identifier_name(ident).as_str()) {
            self.record_declaration("trait", ident);
        }
    }
}

impl<'ast> Visit<'ast> for BaseVisitor<'_> {
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        record_use_branch_roots(
            &mut self.violations,
            self.path,
            "base crate import",
            &item.tree,
            BASE_LAYER_RUST_CRATES,
        );
        visit::visit_item_use(self, item);
    }

    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        record_ident(
            &mut self.violations,
            self.path,
            "base extern crate",
            &item.ident,
            BASE_LAYER_RUST_CRATES,
        );
        visit::visit_item_extern_crate(self, item);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if let Some(segment) = path.segments.first() {
            record_ident(
                &mut self.violations,
                self.path,
                "base crate path",
                &segment.ident,
                BASE_LAYER_RUST_CRATES,
            );
        }
        visit::visit_path(self, path);
    }

    fn visit_item_const(&mut self, item: &'ast syn::ItemConst) {
        self.check_constant(&item.ident);
        visit::visit_item_const(self, item);
    }

    fn visit_impl_item_const(&mut self, item: &'ast syn::ImplItemConst) {
        self.check_constant(&item.ident);
        visit::visit_impl_item_const(self, item);
    }

    fn visit_trait_item_const(&mut self, item: &'ast syn::TraitItemConst) {
        self.check_constant(&item.ident);
        visit::visit_trait_item_const(self, item);
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.check_function(&item.sig.ident, false);
        visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.check_function(&item.sig.ident, true);
        visit::visit_impl_item_fn(self, item);
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        self.check_function(&item.sig.ident, true);
        visit::visit_trait_item_fn(self, item);
    }

    fn visit_foreign_item_fn(&mut self, item: &'ast syn::ForeignItemFn) {
        self.check_function(&item.sig.ident, false);
        visit::visit_foreign_item_fn(self, item);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        self.check_trait(&item.ident);
        visit::visit_item_trait(self, item);
    }
}

fn base_violations(path: &Path, source: &str) -> Vec<String> {
    let file = match syn::parse_file(source) {
        Ok(file) => file,
        Err(error) => return vec![format!("{}: invalid Rust: {error}", path.display())],
    };
    let mut visitor = BaseVisitor {
        path,
        violations: Vec::new(),
    };
    visitor.visit_file(&file);
    visitor.violations
}

fn dependency_violations(
    path: &Path,
    table: &toml::Table,
    workspace_dependencies: Option<&toml::Table>,
    forbidden: &[&str],
    violations: &mut Vec<String>,
) {
    for (name, specification) in table {
        let forbidden_name = is_forbidden_package(name, forbidden);
        if forbidden_name {
            violations.push(format!("{}: forbidden dependency `{name}`", path.display()));
        }
        let package = specification
            .as_table()
            .and_then(|table| table.get("package"))
            .and_then(toml::Value::as_str);
        let forbidden_package =
            package.is_some_and(|package| is_forbidden_package(package, forbidden));
        if forbidden_package {
            violations.push(format!(
                "{}: forbidden dependency package alias `{name}`",
                path.display()
            ));
        }
        let inherits_workspace = specification
            .as_table()
            .and_then(|table| table.get("workspace"))
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);
        let workspace_package = workspace_dependencies
            .and_then(|dependencies| dependencies.get(name))
            .and_then(toml::Value::as_table)
            .and_then(|table| table.get("package"))
            .and_then(toml::Value::as_str);
        if inherits_workspace
            && !forbidden_name
            && !forbidden_package
            && workspace_package.is_some_and(|package| is_forbidden_package(package, forbidden))
        {
            violations.push(format!(
                "{}: forbidden workspace dependency alias `{name}`",
                path.display()
            ));
        }
    }
}

fn walk_manifest(
    path: &Path,
    table: &toml::Table,
    workspace_dependencies: Option<&toml::Table>,
    forbidden: &[&str],
    violations: &mut Vec<String>,
) {
    for (name, value) in table {
        let Some(child) = value.as_table() else {
            continue;
        };
        if DEPENDENCY_TABLES.contains(&name.as_str()) {
            dependency_violations(path, child, workspace_dependencies, forbidden, violations);
        }
        walk_manifest(path, child, workspace_dependencies, forbidden, violations);
    }
}

fn parse_manifest(path: &Path, source: &str) -> Result<toml::Value, String> {
    source
        .parse::<toml::Value>()
        .map_err(|error| format!("{}: invalid TOML: {error}", path.display()))
}

fn workspace_dependencies(manifest: &toml::Value) -> Option<&toml::Table> {
    manifest
        .as_table()
        .and_then(|table| table.get("workspace"))
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("dependencies"))
        .and_then(toml::Value::as_table)
}

fn manifest_violations_with_workspace_dependencies(
    path: &Path,
    source: &str,
    workspace_dependencies: Option<&toml::Table>,
    forbidden: &[&str],
) -> Vec<String> {
    let manifest = match parse_manifest(path, source) {
        Ok(manifest) => manifest,
        Err(error) => return vec![error],
    };
    let Some(table) = manifest.as_table() else {
        return vec![format!("{}: TOML manifest is not a table", path.display())];
    };
    let mut violations = Vec::new();
    walk_manifest(
        path,
        table,
        workspace_dependencies,
        forbidden,
        &mut violations,
    );
    violations
}

fn manifest_violations_with_workspace(
    path: &Path,
    source: &str,
    workspace_manifest: Option<(&Path, &str)>,
    forbidden: &[&str],
) -> Vec<String> {
    let workspace_manifest = match workspace_manifest {
        Some((workspace_path, source)) => match parse_manifest(workspace_path, source) {
            Ok(manifest) => Some(manifest),
            Err(error) => return vec![format!("invalid workspace manifest: {error}")],
        },
        None => None,
    };
    manifest_violations_with_workspace_dependencies(
        path,
        source,
        workspace_manifest.as_ref().and_then(workspace_dependencies),
        forbidden,
    )
}

fn manifest_violations(path: &Path, source: &str, forbidden: &[&str]) -> Vec<String> {
    manifest_violations_with_workspace_dependencies(path, source, None, forbidden)
}

fn app_source_violations(path: &Path, source: &str) -> Vec<String> {
    import_violations(path, source, NEATH_RUST_CRATES)
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
fn checker_accepts_clean_base_and_styled_fixtures() {
    assert!(
        base_violations(
            Path::new("crates/base/src/fake.rs"),
            "pub fn neutral_layout() {}"
        )
        .is_empty()
    );
    assert!(
        app_source_violations(Path::new("crates/ui/src/fake.rs"), "use gpui::{App, div};")
            .is_empty()
    );
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
    let found = app_source_violations(
        Path::new("crates/ui/src/fake.rs"),
        "use neath_core::SourceId;",
    );
    assert_eq!(found.len(), 1);
}

#[test]
fn checker_rejects_a_direct_neath_workspace_import_in_styled_fixture() {
    let found = app_source_violations(Path::new("crates/ui/src/fake.rs"), "use neath::SourceId;");
    assert_eq!(found.len(), 1);
}

#[test]
fn checker_rejects_a_neath_package_alias_in_styled_manifest_fixture() {
    let found = manifest_violations(
        Path::new("crates/ui/Cargo.toml"),
        "[dependencies]\napp_library = { package = \"neath\", workspace = true }",
        NEATH_RUST_CRATES,
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
            "[dependencies]\napp={package=\"neath\",path=\"../../../neath\"}",
        ),
        (
            Path::new("crates/ui/Cargo.toml"),
            "[dependencies]\napp = { package = \"neath\", path = \"../../../neath\" }",
        ),
    ] {
        let found = if path
            .extension()
            .is_some_and(|extension| extension == "toml")
        {
            manifest_violations(path, source, NEATH_RUST_CRATES)
        } else {
            app_source_violations(path, source)
        };
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
            "impl Slider { pub fn value(&self) {} }"
        )
        .is_empty()
    );
    assert!(
        base_violations(
            Path::new("crates/base/src/fake.rs"),
            "/// See [the design](https://example.test/gpui_neath).\npub fn neutral_layout() {}"
        )
        .is_empty()
    );
    assert!(
        app_source_violations(
            Path::new("crates/ui/src/fake.rs"),
            "// use { neath::SourceId };\npub fn neutral_layout() {}"
        )
        .is_empty()
    );
    assert!(
        app_source_violations(
            Path::new("crates/ui/src/fake.rs"),
            "fn neutral() { let root = \"gpui_neath::Root\"; }"
        )
        .is_empty()
    );
    assert!(
        manifest_violations(
            Path::new("crates/ui/Cargo.toml"),
            "# [dependencies.neath]",
            NEATH_RUST_CRATES
        )
        .is_empty()
    );
}

#[test]
fn checker_rejects_syntax_aware_neath_import_and_manifest_escapes() {
    for source in [
        "use neath as app;",
        "use neath :: SourceId;",
        "extern crate neath as app;",
        "type Leak = neath :: SourceId;",
        "fn leak() { let _ = neath :: open(); }",
    ] {
        let found = app_source_violations(Path::new("crates/ui/src/fake.rs"), source);
        assert_eq!(found.len(), 1, "fixture should be rejected: {source}");
    }
    for source in [
        "[target.'cfg(target_os = \"windows\")'.dependencies]\napp = { package = 'neath-core' }",
        "[target.'cfg(unix)'.build-dependencies.neath-update-helper]\nversion = \"1\"",
        "[dependencies]\napp = { package = 'neath' }",
    ] {
        let found =
            manifest_violations(Path::new("crates/ui/Cargo.toml"), source, NEATH_RUST_CRATES);
        assert_eq!(found.len(), 1, "fixture should be rejected: {source}");
    }
}

#[test]
fn checker_normalizes_raw_identifiers_and_checks_only_use_branch_roots() {
    for source in ["use r#neath as app;", "type Leak = r#neath :: SourceId;"] {
        let found = app_source_violations(Path::new("crates/ui/src/fake.rs"), source);
        assert_eq!(found.len(), 1, "fixture should be rejected: {source}");
    }
    for source in [
        "pub fn r#tool_popover() {}",
        "pub const r#RADIUS_SM: f32 = 1.0;",
    ] {
        let found = base_violations(Path::new("crates/base/src/fake.rs"), source);
        assert_eq!(found.len(), 1, "fixture should be rejected: {source}");
    }
    for source in ["use self::neath::Thing;", "use crate::neath;"] {
        assert!(
            app_source_violations(Path::new("crates/ui/src/fake.rs"), source).is_empty(),
            "fixture should be accepted: {source}",
        );
    }
    let found = app_source_violations(
        Path::new("crates/ui/src/fake.rs"),
        "use { neath::X, self::neath::Y };",
    );
    assert_eq!(found.len(), 1);
}

#[test]
fn checker_rejects_a_workspace_inherited_neath_alias() {
    let ui_manifest = "[dependencies]\napp_library = { workspace = true }";
    let workspace_manifest = "[workspace.dependencies]\napp_library = { package = \"neath-core\", path = \"../../../neath\" }";
    let found = manifest_violations_with_workspace(
        Path::new("crates/ui/Cargo.toml"),
        ui_manifest,
        Some((Path::new("Cargo.toml"), workspace_manifest)),
        NEATH_RUST_CRATES,
    );
    assert_eq!(
        found.len(),
        1,
        "workspace alias should be rejected: {workspace_manifest}",
    );
}

#[test]
fn checker_rejects_a_base_workspace_inherited_styled_alias() {
    let base_manifest = "[dependencies]\ngpui-component = { workspace = true }";
    let workspace_manifest = "[workspace.dependencies]\ngpui-component = { package = \"gpui-neath\", path = \"crates/ui\" }";
    let found = manifest_violations_with_workspace(
        Path::new("crates/base/Cargo.toml"),
        base_manifest,
        Some((Path::new("Cargo.toml"), workspace_manifest)),
        BASE_LAYER_RUST_CRATES,
    );
    assert_eq!(
        found.len(),
        1,
        "base workspace alias should be rejected: {workspace_manifest}",
    );
}

#[test]
fn checker_allows_an_unused_forbidden_workspace_dependency() {
    let ui_manifest = "[dependencies]\napp_library = { workspace = true }";
    let workspace_manifest = concat!(
        "[workspace.dependencies]\n",
        "app_library = { package = \"safe-package\", version = \"1\" }\n",
        "unused_neath = { package = \"neath-core\", path = \"../../../neath\" }"
    );
    assert!(
        manifest_violations_with_workspace(
            Path::new("crates/ui/Cargo.toml"),
            ui_manifest,
            Some((Path::new("Cargo.toml"), workspace_manifest)),
            NEATH_RUST_CRATES,
        )
        .is_empty()
    );
}

#[test]
fn checker_rejects_style_declarations_with_modifiers_or_restricted_visibility() {
    for source in [
        "pub(crate) async fn tool_popover() {}",
        "pub(super) unsafe fn tool_popover() {}",
        "pub(in crate) const RADIUS_SM: f32 = 1.0;",
        "pub unsafe extern \"C\" fn tool_popover() {}",
        "impl Surface { pub const RADIUS_SM: f32 = 1.0; }",
        "impl Surface { pub fn tool_popover() {} }",
        "trait Styled { fn tool_popover(); }",
    ] {
        let found = base_violations(Path::new("crates/base/src/fake.rs"), source);
        assert_eq!(found.len(), 1, "fixture should be rejected: {source}");
    }
}

#[test]
fn checker_uses_syntax_not_text_for_base_source() {
    for source in [
        "const GUIDE: &str = \"gpui_neath::Root; pub fn tool_popover() {}\";",
        "/* gpui_neath and pub fn tool_popover() {} are only documentation. */",
        "pub fn tool_popover_metrics() {}",
        "impl Slider { pub fn value(&self) {} }",
    ] {
        assert!(
            base_violations(Path::new("crates/base/src/fake.rs"), source).is_empty(),
            "fixture should be accepted: {source}"
        );
    }
}

#[test]
fn checker_fails_closed_on_unparseable_boundary_input() {
    assert_eq!(
        app_source_violations(Path::new("crates/ui/src/fake.rs"), "fn {").len(),
        1
    );
    assert_eq!(
        base_violations(Path::new("crates/base/src/fake.rs"), "fn {").len(),
        1
    );
    assert_eq!(
        manifest_violations(
            Path::new("crates/ui/Cargo.toml"),
            "[dependencies",
            NEATH_RUST_CRATES
        )
        .len(),
        1
    );
}

#[test]
fn style_kernel_stays_above_base_and_below_the_application() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut failures = Vec::new();
    let workspace_manifest = manifest_dir.join("../../Cargo.toml");
    let workspace_source =
        std::fs::read_to_string(&workspace_manifest).expect("readable workspace manifest");
    let workspace_manifest = match parse_manifest(&workspace_manifest, &workspace_source) {
        Ok(manifest) => Some(manifest),
        Err(error) => {
            failures.push(format!("invalid workspace manifest: {error}"));
            None
        }
    };
    let root_workspace_dependencies = workspace_manifest.as_ref().and_then(workspace_dependencies);
    let mut base_files = Vec::new();
    rust_files(&manifest_dir.join("../base/src"), &mut base_files);
    for file in base_files {
        let source = std::fs::read_to_string(&file).expect("readable base source");
        failures.extend(base_violations(&file, &source));
    }
    let base_manifest = manifest_dir.join("../base/Cargo.toml");
    let base_manifest_source =
        std::fs::read_to_string(&base_manifest).expect("readable base manifest");
    failures.extend(manifest_violations_with_workspace_dependencies(
        &base_manifest,
        &base_manifest_source,
        root_workspace_dependencies,
        BASE_LAYER_RUST_CRATES,
    ));
    let mut styled_files = Vec::new();
    rust_files(&manifest_dir.join("src"), &mut styled_files);
    for file in styled_files {
        let source = std::fs::read_to_string(&file).expect("readable styled source");
        failures.extend(app_source_violations(&file, &source));
    }
    let styled_manifest = manifest_dir.join("Cargo.toml");
    let manifest_source =
        std::fs::read_to_string(&styled_manifest).expect("readable styled manifest");
    failures.extend(manifest_violations_with_workspace_dependencies(
        &styled_manifest,
        &manifest_source,
        root_workspace_dependencies,
        NEATH_RUST_CRATES,
    ));
    let build_script = manifest_dir.join("build.rs");
    let build_source =
        std::fs::read_to_string(&build_script).expect("readable styled build script");
    failures.extend(app_source_violations(&build_script, &build_source));
    assert!(
        failures.is_empty(),
        "style ownership boundary violations:\n{}",
        failures.join("\n")
    );
}
