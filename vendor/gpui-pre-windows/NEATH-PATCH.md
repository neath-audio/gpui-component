# gpui-pre-windows Neath patch

Published package: `gpui-pre-windows` 0.3.12 from crates.io, GPUI Kit's republish of
Zed's `gpui_windows` crate.

- Upstream repository: https://github.com/zed-industries/zed
- Upstream commit: `d2a4fb6ded` (the snapshot named in the package description)
- Upstream directory: `crates/gpui_windows`
- Published crate SHA-256: `7201762420222d5b5ce5bdf0056996199212a3349bfb2457a3a0238c688681f3`
- Changed files: `src/window.rs`
- License: Apache-2.0 (upstream's); the published license file is kept.

The Windows half of the native file drag-out in `../gpui-pre/NEATH-PATCH.md`,
applied unchanged from `neath-audio/zed` `677203124d`: `file_drag_starter`
returns a starter that runs an outgoing OLE drag (a `CF_HDROP` data object from
`SHCreateDataObject` and a `FileDragSource` passed to `DoDragDrop`). The call
blocks in the modal loop, so it must start outside any `App` borrow; the
session flag brackets the loop. The drop-target helper logs at debug level.
Not compiled on this host; verified on Windows by the application's CI.
