use gpui::{
    AnyElement, App, BorderStyle, Bounds, Corners, Edges, Hsla, IntoElement, PaintQuad,
    ParentElement, Pixels, RenderOnce, Styled as _, Window, canvas, div, hsla, point, px, size,
    transparent_black,
};

use crate::ActiveTheme;

/// A transparency-checkerboard backdrop for revealing true hue and alpha.
#[derive(IntoElement)]
pub struct Checkerboard {
    children: Vec<AnyElement>,
    is_dark: bool,
}

impl Checkerboard {
    pub fn new(is_dark: bool) -> Self {
        Self {
            children: Vec::new(),
            is_dark,
        }
    }
}

impl ParentElement for Checkerboard {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

const CELL: Pixels = px(12.);

fn palette(is_dark: bool) -> (Hsla, Hsla) {
    if is_dark {
        (hsla(0., 0., 0.10, 1.), hsla(0., 0., 0.13, 1.))
    } else {
        (hsla(0., 0., 1.00, 1.), hsla(0., 0., 0.95, 1.))
    }
}

fn grid_len(span: Pixels) -> i32 {
    (span / CELL).ceil() as i32
}

impl RenderOnce for Checkerboard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (c1, c2) = palette(self.is_dark);

        div()
            .bg(c1)
            .rounded(cx.theme().radius_lg)
            .overflow_hidden()
            .size_full()
            .child(
                canvas(
                    move |_, _, _| (),
                    move |bounds, _, window, _| {
                        let rows = grid_len(bounds.size.height);
                        let cols = grid_len(bounds.size.width);
                        for row in 0..rows {
                            for col in 0..cols {
                                if (row + col) % 2 == 0 {
                                    let origin =
                                        bounds.origin + point(CELL * col as f32, CELL * row as f32);
                                    window.paint_quad(PaintQuad {
                                        bounds: Bounds {
                                            origin,
                                            size: size(CELL, CELL),
                                        },
                                        corner_radii: Corners::default(),
                                        background: c2.into(),
                                        border_widths: Edges::default(),
                                        border_color: transparent_black(),
                                        border_style: BorderStyle::default(),
                                    });
                                }
                            }
                        }
                    },
                )
                .absolute()
                .size_full(),
            )
            .children(self.children)
    }
}

#[test]
fn palette_matches_the_accepted_neutral_pairs() {
    assert_eq!(
        palette(true),
        (hsla(0., 0., 0.10, 1.), hsla(0., 0., 0.13, 1.))
    );
    assert_eq!(
        palette(false),
        (hsla(0., 0., 1.00, 1.), hsla(0., 0., 0.95, 1.))
    );
}

#[test]
fn grid_covers_partial_edge_cells() {
    assert_eq!(grid_len(px(0.)), 0);
    assert_eq!(grid_len(px(12.)), 1);
    assert_eq!(grid_len(px(12.5)), 2);
    assert_eq!(grid_len(px(25.)), 3);
}
