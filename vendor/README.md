# Vendored GPUI patches

GPUI comes from crates.io `gpui-pre` (GPUI Kit's snapshots of Zed's crates).
The four packages here are those snapshots plus Neath's changes: backdrop blur,
native file drag-out, `FocusHandle::is_rendered`. The root `[patch.crates-io]`
selects them for this workspace; applications select the same four from this
repository's git rev, so the UI library and its GPUI always move together.

Each package's `NEATH-PATCH.md` records the published source and checksum, the
changed files, what the change does and when it can go. When the `gpui-pre` pin
moves, re-vendor the published package, re-apply the changed files, update the
notes, and run the application's acceptance checks (blur on macOS, drag-out on
macOS and Windows).
