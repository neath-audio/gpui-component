# gpui-pre-macos Neath patch

Published package: `gpui-pre-macos` 0.3.12 from crates.io, GPUI Kit's republish of
Zed's `gpui_macos` crate.

- Upstream repository: https://github.com/zed-industries/zed
- Upstream commit: `d2a4fb6ded` (the snapshot named in the package description)
- Upstream directory: `crates/gpui_macos`
- Published crate SHA-256: `2de7d3f4ad51ac387e822495db363fcc1188d349793365731d054444c4cb06ee`
- Changed files: `src/window.rs`
- License: Apache-2.0 (upstream's); the published license file is kept.

The AppKit half of the native file drag-out in `../gpui-pre/NEATH-PATCH.md`:
`start_external_drag` and `file_drag_starter` share `mac_start_external_drag`,
which builds the dragging items the way upstream does (lazy icons by content
type) and sets the file-drag session flag when AppKit accepts the session.
