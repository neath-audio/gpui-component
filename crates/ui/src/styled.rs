use crate::theme::ActiveTheme as _;
use gpui::{App, Styled};

pub use crate::component_traits::{Collapsible, Disableable, Selectable};
pub use crate::sizing::{Sizable, Size, StyleSized};
pub use gpui_base::{FocusableExt, RoleOverride, StyledExt, box_shadow, h_flex, v_flex};

pub trait ElevatedSurfaceExt: Styled + Sized {
    fn elevated_surface(self, cx: &App) -> Self {
        self.bg(cx.theme().tokens.popover)
            .text_color(cx.theme().popover_foreground)
            .border_1()
            .border_color(cx.theme().hairline_strong)
            .shadow(cx.theme().shadow_2().into_vec())
            .rounded(cx.theme().radius)
    }
}

impl<T: Styled> ElevatedSurfaceExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Theme, ThemeToken};
    use gpui::{TestAppContext, div, hsla, linear_color_stop, linear_gradient, px};

    #[gpui::test]
    fn elevated_surface_uses_component_theme_roles(cx: &mut TestAppContext) {
        cx.update(crate::init);
        cx.update(|cx| {
            let representative = hsla(210., 0.25, 0.31, 1.);
            let foreground = hsla(0., 0., 0.96, 1.);
            let generic_border = hsla(20., 0.20, 0.20, 1.);
            let strong_hairline = hsla(190., 0.50, 0.70, 0.65);
            let background = linear_gradient(
                37.,
                linear_color_stop(hsla(220., 0.30, 0.22, 1.), 0.),
                linear_color_stop(hsla(185., 0.25, 0.38, 1.), 1.),
            );

            {
                let theme = Theme::global_mut(cx);
                theme.popover = representative;
                theme.tokens.popover = ThemeToken::new(representative, background);
                theme.popover_foreground = foreground;
                theme.border = generic_border;
                theme.hairline_strong = strong_hairline;
                theme.radius = px(7.);
            }

            let expected_shadow = Theme::global(cx).shadow_2().into_vec();
            let mut actual = div().elevated_surface(cx);
            let mut expected = div()
                .bg(background)
                .text_color(foreground)
                .border_1()
                .border_color(strong_hairline)
                .shadow(expected_shadow)
                .rounded(px(7.));

            assert_eq!(actual.style().clone(), expected.style().clone());
            assert_ne!(actual.style().border_color, Some(generic_border));
        });
    }
}
