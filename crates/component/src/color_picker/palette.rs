//! Preset colors on a fixed column grid.

use std::sync::LazyLock;

use gpui::{
    AnyElement, App, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, div, hsla,
    prelude::FluentBuilder as _,
};
use gpui_base::{ColorSwatch as BaseColorSwatch, actions::Confirm};

use super::{ColorEditor, ColorSwatch, same_at_hex_precision};
use crate::{
    ActiveTheme as _, Colorize as _, Icon, IconName, Sizable as _,
    separator::Separator,
    tooltip::{ManagedTooltipExt as _, Tooltip},
    v_flex,
};

/// More columns give small, useful color targets without leaving empty space.
/// Custom palettes keep the same column rhythm.
const COLUMNS: u16 = 12;

/// The ink that reads against a swatch: black or white, whichever contrasts more
/// with the color as it appears over the popup's surface.
fn contrast_marker(color: Hsla, surface: Hsla) -> Hsla {
    let rgb = surface.blend(color).to_rgb();
    let linear = |value: f32| {
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    let luminance = 0.2126 * linear(rgb.r) + 0.7152 * linear(rgb.g) + 0.0722 * linear(rgb.b);
    // This crossover chooses the greater WCAG contrast of black and white.
    hsla(0., 0., if luminance > 0.179 { 0. } else { 1. }, 1.)
}

struct Swatches<'a> {
    id: &'a ElementId,
    editor: &'a ColorEditor,
    value: Option<Hsla>,
    disabled: bool,
}

impl Swatches<'_> {
    fn item(&self, key: String, label: SharedString, color: Hsla, cx: &App) -> AnyElement {
        let selected = self
            .value
            .is_some_and(|value| same_at_hex_precision(value, color));
        let marker = contrast_marker(color, cx.theme().popover);
        let border = cx.theme().border;
        let radius = cx.theme().radius_tokens().sm;
        let disabled = self.disabled;
        let hover_state = self.editor.state().clone();
        let click_state = self.editor.state().clone();
        let confirm_state = self.editor.state().clone();
        let tooltip_label = label.clone();
        let cell_id: ElementId = (self.id.clone(), format!("{key}-cell")).into();
        let swatch_id: ElementId = (self.id.clone(), key).into();

        div()
            .id(cell_id)
            .w_full()
            .min_w_0()
            .aspect_square()
            .managed_tooltip(move |window, cx| {
                Tooltip::new(tooltip_label.clone()).build(window, cx)
            })
            .child(
                BaseColorSwatch::new(swatch_id, color)
                    .selected(selected)
                    .disabled(disabled)
                    .accessibility_label(label)
                    .debug_selector(move || format!("color-picker-swatch-{}", color.to_hex()))
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(radius)
                    // The color fills the whole target. Hover and press change the
                    // outline, never the color being sampled.
                    .bg(color)
                    .border_1()
                    .border_color(if selected { marker } else { border })
                    .when(!disabled, |this| {
                        this.hover(move |this| this.border_color(marker))
                            .active(move |this| this.border_color(marker))
                            .focus_visible(move |this| this.border_2().border_color(marker))
                    })
                    .when(selected, |this| {
                        this.child(Icon::new(IconName::Check).xsmall().text_color(marker))
                    })
                    .on_hover(move |color, entered, window, cx| {
                        if entered {
                            hover_state
                                .update(cx, |state, cx| state.preview_color(color, window, cx));
                        }
                    })
                    .on_click(move |color, _, window, cx| {
                        click_state.update(cx, |state, cx| state.select_color(color, window, cx));
                    })
                    // An enclosing dialog or color picker binds Enter to confirm,
                    // which would reach it instead of the focused swatch's click.
                    .when(!disabled, |this| {
                        this.on_action(move |_: &Confirm, window, cx| {
                            confirm_state
                                .update(cx, |state, cx| state.select_color(color, window, cx));
                        })
                    }),
            )
            .into_any_element()
    }

    fn grid(&self, items: Vec<AnyElement>) -> impl IntoElement {
        div()
            .w_full()
            .min_w_0()
            .grid()
            .grid_cols(COLUMNS)
            .gap_1()
            .children(items)
    }
}

pub(super) fn panel(
    id: ElementId,
    editor: &ColorEditor,
    featured: Option<&[Hsla]>,
    swatches: Option<&[ColorSwatch]>,
    disabled: bool,
    cx: &App,
) -> AnyElement {
    let swatches = swatches.unwrap_or_else(|| default_palette());
    let grid = Swatches {
        id: &id,
        editor,
        value: editor.state().read(cx).value(),
        disabled,
    };
    let featured = featured.filter(|colors| !colors.is_empty()).map(|colors| {
        colors
            .iter()
            // Featured slots may contain the same color more than once.
            .enumerate()
            .map(|(ix, color)| {
                grid.item(format!("featured-{ix}"), color.to_hex().into(), *color, cx)
            })
            .collect::<Vec<_>>()
    });
    let items = swatches
        .iter()
        .enumerate()
        .map(|(ix, swatch)| {
            grid.item(
                format!("swatch-{ix}"),
                swatch.label().clone(),
                swatch.color(),
                cx,
            )
        })
        .collect::<Vec<_>>();

    let leave_state = editor.state().clone();
    v_flex()
        .id((id.clone(), "palette-surface"))
        .debug_selector(|| "color-picker-palette".into())
        .w_full()
        .min_w_0()
        .gap_3()
        // Moving between swatches keeps previewing. Leaving them drops the preview,
        // since the other views edit the color they display.
        .on_hover(move |hovered, window, cx| {
            if !hovered {
                leave_state.update(cx, |state, cx| state.clear_preview(window, cx));
            }
        })
        .when_some(featured, |this, featured| {
            this.child(grid.grid(featured))
                .child(Separator::horizontal())
        })
        .child(grid.grid(items))
        .into_any_element()
}

/// Built-in swatches are color content, not chrome. Their labels and colors are
/// allocated once, only if a picker actually opens the default palette.
fn default_palette() -> &'static [ColorSwatch] {
    static PALETTE: LazyLock<Vec<ColorSwatch>> = LazyLock::new(|| {
        let mut swatches = Vec::with_capacity(72);
        for (label, lightness) in [
            ("White", 1.),
            ("Pearl", 0.95),
            ("Mist", 0.9),
            ("Silver", 0.8),
            ("Ash", 0.7),
            ("Gray", 0.6),
            ("Stone", 0.5),
            ("Slate", 0.4),
            ("Charcoal", 0.3),
            ("Graphite", 0.2),
            ("Ink", 0.1),
            ("Black", 0.),
        ] {
            swatches.push(ColorSwatch::new(label, hsla(0., 0., lightness, 1.)));
        }
        for (shade, lightness) in [
            ("Deep", 0.22),
            ("Dark", 0.35),
            ("", 0.5),
            ("Light", 0.68),
            ("Pale", 0.84),
        ] {
            for (label, hue) in [
                ("Red", 0.),
                ("Orange", 30.),
                ("Yellow", 50.),
                ("Lime", 85.),
                ("Green", 140.),
                ("Teal", 165.),
                ("Cyan", 185.),
                ("Blue", 215.),
                ("Indigo", 240.),
                ("Violet", 270.),
                ("Magenta", 300.),
                ("Pink", 330.),
            ] {
                swatches.push(ColorSwatch::new(
                    format!("{shade} {label}").trim().to_owned(),
                    hsla(hue / 360., 0.75, lightness, 1.),
                ));
            }
        }
        swatches
    });
    &PALETTE
}

#[cfg(test)]
mod tests;
