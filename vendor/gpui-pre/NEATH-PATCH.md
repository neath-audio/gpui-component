# gpui-pre Neath patch

Published package: `gpui-pre` 0.3.12 from crates.io, GPUI Kit's republish of
Zed's `gpui` crate.

- Upstream repository: https://github.com/zed-industries/zed
- Upstream commit: `d2a4fb6ded` (the snapshot named in the package description)
- Upstream directory: `crates/gpui`
- Published crate SHA-256: `29db89613527079737f53752eab27fb65c9a7dd94979256c65a0e590c9fd34d8`
- Changed files: `src/elements/div.rs`, `src/platform.rs`, `src/scene.rs`, `src/window.rs`
- License: Apache-2.0 (upstream's); the published license file is kept.

Neath's GPUI changes, carried from `neath-audio/zed` (`7fdb97cad..fdf4160c`)
and the earlier port to gpui-pre 0.3.10:

- **Native file drag-out** (`platform.rs`, `window.rs`): `FileDragStarter`,
  `PlatformWindow::file_drag_starter`, `Window::file_drag_starter`, and the
  process-wide `file_drag_session_active` flag that distinguishes a
  self-originated OS drag re-entering the window from a foreign one.
- **Group drag-over hitboxes** (`elements/div.rs`): a group drag style applies
  to children without their own drag listener; regression test included.
- **`FocusHandle::is_rendered`** (`window.rs`): whether an element tracked the
  handle in the last rendered frame, for focus restoration.
- **Backdrop blur** (`scene.rs`, `window.rs`): the `BackdropBlur` primitive,
  `Scene::backdrop_blurs` interleaved by draw order, and
  `Window::paint_backdrop_blur`. Renderers without support ignore it; see
  `../gpui-pre-apple/NEATH-PATCH.md` for the Metal implementation.
  `crates/component/src/material.rs` depends on this.

The patched crates in `vendor/` are selected by the root `[patch.crates-io]`
and move together with the `gpui-pre` pin. Remove a piece when upstream GPUI
offers an equivalent and the consumers have moved to it.
