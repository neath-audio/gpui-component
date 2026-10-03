use gpui::{App, Pixels, Rems, Styled, Window, rems};

use crate::ActiveTheme as _;

/// Application-owned UI text sizes, independent of the layout's rem base.
/// Theme files do not replace this runtime policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiTypography {
    xsmall: Rems,
    small: Rems,
    default: Rems,
    large: Rems,
}

impl Default for UiTypography {
    fn default() -> Self {
        Self {
            xsmall: rems(11. / 16.),
            small: rems(12. / 16.),
            default: rems(13. / 16.),
            large: rems(15. / 16.),
        }
    }
}

impl UiTypography {
    /// Set a role's relative size.
    ///
    /// Panics if the size is not finite and positive.
    pub fn with_size(mut self, role: TextSize, size: Rems) -> Self {
        assert!(
            size.0.is_finite() && size.0 > 0.,
            "UI text size must be finite and positive"
        );
        match role {
            TextSize::XSmall => self.xsmall = size,
            TextSize::Small => self.small = size,
            TextSize::Default => self.default = size,
            TextSize::Large => self.large = size,
        }
        self
    }

    /// The relative size selected for this semantic text role.
    pub fn size(&self, role: TextSize) -> Rems {
        match role {
            TextSize::XSmall => self.xsmall,
            TextSize::Small => self.small,
            TextSize::Default => self.default,
            TextSize::Large => self.large,
        }
    }
}

/// UI text roles, independent of a control's height or padding.
///
/// The active Theme resolves the role independently of control geometry.
/// Root supplies the layout rem base; measurements use that same base.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextSize {
    /// Captions and compact annotations.
    XSmall,
    /// Supporting text and metadata.
    Small,
    /// Ordinary labels, actions, navigation and table content.
    #[default]
    Default,
    /// Prominent UI titles.
    Large,
}

impl TextSize {
    /// The role's relative size. Keep it relative when styling an element.
    pub fn rems(self, cx: &App) -> Rems {
        cx.theme().ui_typography().size(self)
    }

    /// Resolve against the current window for text shaping or measurement.
    pub fn to_pixels(self, window: &Window, cx: &App) -> Pixels {
        self.rems(cx).to_pixels(window.rem_size())
    }
}

/// Apply UI text roles without changing line height, weight or control geometry.
pub trait StyledTypography: Styled {
    fn text_ui_size(self, size: TextSize, cx: &App) -> Self {
        self.text_size(size.rems(cx))
    }

    fn text_ui_xs(self, cx: &App) -> Self {
        self.text_ui_size(TextSize::XSmall, cx)
    }

    fn text_ui_sm(self, cx: &App) -> Self {
        self.text_ui_size(TextSize::Small, cx)
    }

    fn text_ui(self, cx: &App) -> Self {
        self.text_ui_size(TextSize::Default, cx)
    }

    fn text_ui_lg(self, cx: &App) -> Self {
        self.text_ui_size(TextSize::Large, cx)
    }
}

impl<T: Styled> StyledTypography for T {}
