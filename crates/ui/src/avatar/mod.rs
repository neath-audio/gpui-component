mod avatar;
mod avatar_group;

pub use avatar::*;
pub use avatar_group::*;

use crate::{Icon, Size, StyledExt as _};
use gpui::{Div, Img, IntoElement, Pixels, Styled, px, rems};

/// Returns the size of the avatar based on the given [`Size`].
///
/// NAMED EXCEPTION (docs/superpowers/specs/2026-07-19-depth-color-language-design.md,
/// neath repo): an avatar's diameter (16-80px) is unrelated to the
/// control-height axis (16-32px) — it's a photo/initials portrait, not a form
/// control, and needs to range far larger than any control ever does. Kept
/// as its own component-local ladder rather than a proportional
/// `metrics()` derivation.
pub(super) fn avatar_size(size: Size) -> Pixels {
    match size {
        Size::Large => px(80.),
        Size::Medium => px(48.),
        Size::Small => px(24.),
        Size::XSmall => px(16.),
        // Dense tier continues the diameter trend one step below XSmall
        // (matches the 12px XXSmall icon column).
        Size::XXSmall => px(12.),
        Size::Size(size) => size,
    }
}

/// Extension for add `avatar_size` method to `IntoElement` to apply avatar size to element.
pub(super) trait AvatarSized: IntoElement + Styled {
    fn avatar_size(self, size: Size) -> Self {
        self.size(avatar_size(size))
    }

    /// NAMED EXCEPTION (docs/superpowers/specs/2026-07-19-depth-color-language-design.md,
    /// neath repo): avatar glyph text tracks the avatar's own diameter
    /// ladder above, not `metrics().text`.
    fn avatar_text_size(self, size: Size) -> Self {
        match size {
            Size::Large => self.text_3xl().font_semibold(),
            Size::Medium => self.text_sm(),
            Size::Small => self.text_xs(),
            Size::XSmall => self.text_size(rems(0.65)),
            // Same ~0.65x-of-diameter ratio as XSmall (8/12 vs 10.4/16).
            Size::XXSmall => self.text_size(rems(0.5)),
            Size::Size(size) => self.size(size * 0.5),
        }
    }
}
impl AvatarSized for Div {}
impl AvatarSized for Icon {}
impl AvatarSized for Img {}
