# gpui-pre-apple Neath patch

Published package: `gpui-pre-apple` 0.3.12 from crates.io, GPUI Kit's republish of
Zed's `gpui_apple` crate.

- Upstream repository: https://github.com/zed-industries/zed
- Upstream commit: `d2a4fb6ded` (the snapshot named in the package description)
- Upstream directory: `crates/gpui_apple`
- Published crate SHA-256: `a460605ec4e8cbcc9b213462a84b1d6cc3efe4ed8f0e18669659f07b2d5c6aec`
- Changed files: `Cargo.toml`, `build.rs`, `src/metal_renderer.rs`, `src/shaders.metal`, `vendor/gpui/src/scene.rs`
- License: Apache-2.0 (upstream's); the published license file is kept.

Metal implementation of the backdrop blur declared in `../gpui-pre/src/scene.rs`,
ported from `neath-audio/zed` `fdf4160c` (metal-rs) onto this snapshot's
`objc2-metal` renderer:

- For each `BackdropBlur`, in draw order: end the render encoder, blit the
  region under the (outward-rounded, viewport-clamped) bounds into a scratch
  texture, run `MPSImageGaussianBlur` (sigma in device pixels, two-entry LRU of
  kernels), then draw the blurred result back inside the rounded bounds with the
  no-blend `backdrop_blur` pipeline (`shaders.metal`).
- Sampling the drawable needs `framebufferOnly = false`, enabled lazily on the
  first blurred frame. A drawable vended before that stays framebuffer-only and
  is skipped for that frame (tint fallback) rather than violating the Metal API.
- Scratch textures follow the viewport and are dropped on a blur-free frame.
- `ensure_command_buffer_completed` surfaces command-buffer errors in the
  offscreen (test) path.
- `Cargo.toml` adds `objc2-metal-performance-shaders` 0.3.2 with the MPS image
  kernel features. `build.rs` generates `scene.h` from the package's own
  `vendor/gpui/src/scene.rs`, so that snapshot carries the same scene change.

Unit tests cover region rounding, clamping, clipping, the kernel cache and two
pixel-readback renders on a transparent target. The published package cannot
build its own tests as is; run them with
`cargo test -p gpui-pre-apple --lib --features "test-support,objc2-core-video/CVBase"`.
