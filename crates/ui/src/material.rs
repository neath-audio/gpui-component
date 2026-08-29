use gpui::{
    AnyElement, App, Bounds, Corners, Element, ElementId, GlobalElementId, InspectorElementId,
    IntoElement, LayoutId, Pixels, Window,
};

use crate::Theme;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaterialDepth {
    Overlay,
    Panel,
}

pub struct Material {
    id: ElementId,
    depth: MaterialDepth,
    corner_radii: Corners<Pixels>,
    child: AnyElement,
}

impl Material {
    pub fn new(id: impl Into<ElementId>, depth: MaterialDepth, child: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            depth,
            corner_radii: Corners::default(),
            child: child.into_any_element(),
        }
    }

    pub fn corner_radii(mut self, radii: Corners<Pixels>) -> Self {
        self.corner_radii = radii;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ResolvedMaterial {
    blur_radius: Pixels,
}

fn resolve_material(theme: &Theme, depth: MaterialDepth) -> Option<ResolvedMaterial> {
    theme.glass_active().then(|| ResolvedMaterial {
        blur_radius: match depth {
            MaterialDepth::Overlay => theme.overlay_blur(),
            MaterialDepth::Panel => theme.panel_blur(),
        },
    })
}

impl Element for Material {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        (self.child.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.child.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let material = resolve_material(Theme::global(cx), self.depth);
        window.paint_layer(bounds, |window| {
            if let Some(material) = material
                && material.blur_radius.as_f32().is_finite()
                && material.blur_radius != Pixels::ZERO
            {
                window.paint_backdrop_blur(bounds, self.corner_radii, material.blur_radius);
            }
            self.child.paint(window, cx);
        });
    }
}

impl IntoElement for Material {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{
        Bounds, ParentElement as _, Styled as _, TestAppContext, canvas, div, point, px, size,
    };
    use gpui_base::ElementExt as _;

    use crate::{Theme, ThemeConfig, ThemeTranslucencyConfig};

    use super::{Material, MaterialDepth, resolve_material};

    fn theme_with_translucency(window: bool, overlay_blur: f32, panel_blur: f32) -> Theme {
        let config = ThemeConfig {
            translucency: ThemeTranslucencyConfig {
                window,
                overlay_blur,
                panel_blur,
            },
            ..ThemeConfig::default()
        };
        let mut theme = Theme::default();
        theme.apply_config(&Rc::new(config));
        theme
    }

    #[gpui::test]
    fn opaque_theme_resolves_no_material_backdrop(_: &mut TestAppContext) {
        let theme = theme_with_translucency(false, 44., 12.);

        assert!(resolve_material(&theme, MaterialDepth::Overlay).is_none());
        assert!(resolve_material(&theme, MaterialDepth::Panel).is_none());
    }

    #[gpui::test]
    fn material_depth_resolves_the_corresponding_theme_radius(_: &mut TestAppContext) {
        let theme = theme_with_translucency(true, 44., 12.);

        assert_eq!(
            resolve_material(&theme, MaterialDepth::Overlay).map(|material| material.blur_radius),
            Some(px(44.))
        );
        assert_eq!(
            resolve_material(&theme, MaterialDepth::Panel).map(|material| material.blur_radius),
            Some(px(12.))
        );
    }

    #[gpui::test]
    fn active_zero_radius_remains_an_unblurred_material(_: &mut TestAppContext) {
        let theme = theme_with_translucency(true, 0., 12.);

        let material = resolve_material(&theme, MaterialDepth::Overlay)
            .expect("active glass must remain distinct from an opaque theme");
        assert_eq!(material.blur_radius, px(0.));
    }

    #[gpui::test]
    fn material_preserves_child_bounds_and_paints_it_once(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let window = cx.add_empty_window();
        let child_bounds = Rc::new(Cell::new(Bounds::default()));
        let paint_count = Rc::new(Cell::new(0));
        let expected_bounds = Bounds::new(point(px(13.), px(17.)), size(px(123.), px(47.)));

        window.draw(expected_bounds.origin, size(px(300.), px(200.)), {
            let child_bounds = child_bounds.clone();
            let paint_count = paint_count.clone();
            move |_, _| {
                Material::new(
                    "material-test",
                    MaterialDepth::Panel,
                    div()
                        .w(expected_bounds.size.width)
                        .h(expected_bounds.size.height)
                        .on_prepaint(move |bounds, _, _| child_bounds.set(bounds))
                        .child(canvas(
                            |_, _, _| {},
                            move |_, _, _, _| paint_count.set(paint_count.get() + 1),
                        )),
                )
            }
        });

        assert_eq!(child_bounds.get(), expected_bounds);
        assert_eq!(paint_count.get(), 1);
    }
}
