pub use crate::component_traits::{Collapsible, Disableable, Selectable};
pub use crate::sizing::{Sizable, Size, StyleSized};
use gpui::{
    App, BoxShadow, Corners, Edges, ParentElement, Pixels, StyleRefinement, Styled, Window, div,
    hsla, px,
};
pub use gpui_base::{FocusableExt, RoleOverride, StyledExt, box_shadow, h_flex, v_flex};

use crate::ActiveTheme as _;

const FOCUS_RING_WIDTH: Pixels = px(3.);
const FOCUS_RING_OPACITY: f32 = 0.5;

/// Ink every layer of a surface's shadow carries — the `rgb(0 0 0 / 0.1)`
/// shadcn/ui spends at each elevation.
const SURFACE_SHADOW_INK: f32 = 0.1;

/// GPUI's native `shadow-md` at `strength` of its full ink.
///
/// Callers animating a surface in pass a rising `strength`; a resting surface
/// passes `1.0`.
///
/// This duplicates the native method's two values only because an animation
/// needs to vary their alpha after the styling context is gone.
pub(crate) fn popover_shadow(strength: f32) -> Vec<BoxShadow> {
    let strength = strength.clamp(0., 1.);
    let ink = hsla(0., 0., 0., SURFACE_SHADOW_INK * strength);
    vec![
        BoxShadow::new(px(0.), px(4.), ink)
            .blur_radius(px(6.))
            .spread_radius(px(-1.)),
        BoxShadow::new(px(0.), px(2.), ink)
            .blur_radius(px(4.))
            .spread_radius(px(-2.)),
    ]
}

/// shadcn/ui's `shadow-lg`, the elevation it lifts a toast to, at `strength` of
/// its full ink.
///
/// A toast sits higher than a popover and is built differently: shadcn gives it
/// a real 1px border rather than the translucent ring it puts on a popup, so
/// there is no ring layer here. Its corner radius is left to the caller.
///
/// The radii retain the existing toast tuning independently of popovers.
pub(crate) fn toast_shadow(strength: f32) -> Vec<BoxShadow> {
    let ink = hsla(0., 0., 0., SURFACE_SHADOW_INK * strength.clamp(0., 1.));
    vec![
        BoxShadow::new(px(0.), px(10.), ink)
            .blur_radius(px(7.5))
            .spread_radius(px(-3.)),
        BoxShadow::new(px(0.), px(4.), ink)
            .blur_radius(px(3.))
            .spread_radius(px(-4.)),
    ]
}

/// Finished styles that read the theme.
///
/// Separate from [`StyledExt`], which holds neutral helpers that make no
/// visual decisions. Everything here does: it reaches into the theme and
/// produces a specific look, which is why it belongs above the base layer.
pub trait ThemeStyled: Styled + Sized {
    /// Give this element the focus appearance the framework's own controls
    /// use: its border tinted with the focus colour, and the ring outside it.
    ///
    /// The ring is dropped when [`crate::Theme::focus_ring`] is off, leaving
    /// the tinted border — an application whose layout clips its containers can
    /// turn it off rather than finding room for the ring in each of them.
    ///
    /// Calling this turns the ring on; gate it with `when` for the conditions
    /// that decide whether the control shows one at all — its focus state,
    /// [`FocusableExt::focus_ring`], appearance, and so on.
    ///
    /// The ring sits outside the element's border, so an ancestor that clips
    /// its content will cut it off — leave it a few pixels of room, or don't
    /// clip.
    fn focus_ring_style(self, window: &Window, cx: &App) -> Self
    where
        Self: ParentElement;

    /// Give this element the surface, edge, shadow and radius of a popover.
    ///
    /// This is the one surface every popup shares — Popover, PopupMenu, Select,
    /// Combobox, DatePicker and the editor's hover popovers — so they cannot
    /// drift apart. See [`popover_shadow`] for what the shadow is modelled on.
    fn popover_style(self, cx: &App) -> Self;

    /// Round this element as far as its size allows — a circle if it is square,
    /// a pill if it is not — unless the theme squares its corners.
    ///
    /// Use this instead of [`gpui::Styled::rounded_full`] on anything the theme
    /// owns. A hardcoded `rounded_full` survives [`crate::Theme::radius`] being
    /// set to zero, which leaves avatars, badge dots and slider thumbs round in
    /// a UI that is square everywhere else. See [`crate::Theme::radius_full`].
    fn rounded_full_style(self, cx: &App) -> Self {
        self.rounded(cx.theme().radius_full())
    }
}

impl<T: Styled + Sized> ThemeStyled for T {
    /// Draw the focus ring the framework's own controls use.
    ///
    /// Calling this turns the ring on; gate it with `when` for the conditions
    /// that decide whether the control shows one at all — its focus state,
    /// [`crate::FocusableExt::focus_ring`], appearance, and so on.
    ///
    /// The ring sits outside the element's border, so an ancestor that clips
    /// its content will cut it off — leave it a few pixels of room, or don't
    /// clip.
    fn focus_ring_style(mut self, window: &Window, cx: &App) -> Self
    where
        Self: ParentElement,
    {
        // The ring is painted outside the border, so a clipping ancestor cuts
        // it off. An application whose layout clips heavily turns it off in the
        // theme and keeps the tinted border, which takes no space.
        if !cx.theme().focus_ring {
            return self.border_color(cx.theme().ring);
        }

        let rem_size = window.rem_size();
        let style = self.style();
        let border_widths = Edges::<Pixels> {
            top: style
                .border_widths
                .top
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or_default(),
            bottom: style
                .border_widths
                .bottom
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or_default(),
            left: style
                .border_widths
                .left
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or_default(),
            right: style
                .border_widths
                .right
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or_default(),
        };
        let radius = Corners::<Pixels> {
            top_left: style
                .corner_radii
                .top_left
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or_default(),
            top_right: style
                .corner_radii
                .top_right
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or_default(),
            bottom_left: style
                .corner_radii
                .bottom_left
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or_default(),
            bottom_right: style
                .corner_radii
                .bottom_right
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or_default(),
        }
        .map(|value| *value + FOCUS_RING_WIDTH);
        let mut ring_style = StyleRefinement::default();
        ring_style.corner_radii.top_left = Some(radius.top_left.into());
        ring_style.corner_radii.top_right = Some(radius.top_right.into());
        ring_style.corner_radii.bottom_left = Some(radius.bottom_left.into());
        ring_style.corner_radii.bottom_right = Some(radius.bottom_right.into());
        let inset = FOCUS_RING_WIDTH;

        self.border_color(cx.theme().ring).child(
            div()
                .flex_none()
                .absolute()
                .top(-(inset + border_widths.top))
                .left(-(inset + border_widths.left))
                .right(-(inset + border_widths.right))
                .bottom(-(inset + border_widths.bottom))
                .border(FOCUS_RING_WIDTH)
                .border_color(cx.theme().ring.alpha(FOCUS_RING_OPACITY))
                .refine_style(&ring_style),
        )
    }

    fn popover_style(self, cx: &App) -> Self {
        let theme = cx.theme();
        // Opaque strong edge plus GPUI's native medium elevation.
        self.bg(theme.tokens.popover)
            .text_color(theme.popover_foreground)
            .border_1()
            .border_color(theme.border_strong)
            .shadow_md()
            .rounded(theme.radius)
    }
}
