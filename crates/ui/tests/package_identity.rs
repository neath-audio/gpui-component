#[test]
fn styled_package_identity_is_gpui_neath() {
    assert_eq!(env!("CARGO_PKG_NAME"), "gpui-neath");
}
