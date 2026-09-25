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

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PaintedMaterial {
    pub(crate) id: ElementId,
    pub(crate) depth: MaterialDepth,
    pub(crate) bounds: Bounds<Pixels>,
    pub(crate) corner_radii: Corners<Pixels>,
}

#[cfg(test)]
std::thread_local! {
    static PAINTED_MATERIALS: std::cell::RefCell<Vec<PaintedMaterial>> = const {
        std::cell::RefCell::new(Vec::new())
    };
}

#[cfg(test)]
pub(crate) fn clear_painted_materials() {
    PAINTED_MATERIALS.with(|materials| materials.borrow_mut().clear());
}

#[cfg(test)]
pub(crate) fn take_painted_materials() -> Vec<PaintedMaterial> {
    PAINTED_MATERIALS.with(|materials| std::mem::take(&mut *materials.borrow_mut()))
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

/// Private one-to-one adapter for the three paint operations Material owns.
///
/// GPUI's test-support API exposes painted quads but not backdrop-blur scene
/// operations. Keeping the seam here lets tests record the exact orchestration
/// that [`Element::paint`] delegates to, while the production implementation
/// forwards each operation directly to the same `Window` method or child.
trait MaterialPaintTarget {
    fn paint_layer<R>(&mut self, bounds: Bounds<Pixels>, paint: impl FnOnce(&mut Self) -> R) -> R;

    fn paint_backdrop_blur(
        &mut self,
        bounds: Bounds<Pixels>,
        corner_radii: Corners<Pixels>,
        blur_radius: Pixels,
    );

    fn paint_child(&mut self, child: &mut AnyElement, cx: &mut App);
}

impl MaterialPaintTarget for Window {
    fn paint_layer<R>(&mut self, bounds: Bounds<Pixels>, paint: impl FnOnce(&mut Self) -> R) -> R {
        Window::paint_layer(self, bounds, paint)
    }

    fn paint_backdrop_blur(
        &mut self,
        bounds: Bounds<Pixels>,
        corner_radii: Corners<Pixels>,
        blur_radius: Pixels,
    ) {
        Window::paint_backdrop_blur(self, bounds, corner_radii, blur_radius);
    }

    fn paint_child(&mut self, child: &mut AnyElement, cx: &mut App) {
        child.paint(self, cx);
    }
}

impl Material {
    fn paint_resolved<T: MaterialPaintTarget>(
        &mut self,
        bounds: Bounds<Pixels>,
        material: Option<ResolvedMaterial>,
        target: &mut T,
        cx: &mut App,
    ) {
        let Some(material) = material else {
            target.paint_child(&mut self.child, cx);
            return;
        };

        target.paint_layer(bounds, |target| {
            if material.blur_radius.as_f32().is_finite() && material.blur_radius != Pixels::ZERO {
                target.paint_backdrop_blur(bounds, self.corner_radii, material.blur_radius);
            }
            target.paint_child(&mut self.child, cx);
        });
    }
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
        #[cfg(test)]
        PAINTED_MATERIALS.with(|materials| {
            materials.borrow_mut().push(PaintedMaterial {
                id: self.id.clone(),
                depth: self.depth,
                bounds,
                corner_radii: self.corner_radii,
            });
        });

        let material = resolve_material(Theme::global(cx), self.depth);
        self.paint_resolved(bounds, material, window, cx);
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
        AnyElement, App, Bounds, Corners, ParentElement as _, Pixels, Styled as _, TestAppContext,
        canvas, div, point, px, size,
    };
    use gpui_base::ElementExt as _;

    use crate::{Theme, ThemeConfig, ThemeTranslucencyConfig};

    use super::{Material, MaterialDepth, MaterialPaintTarget, ResolvedMaterial, resolve_material};

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum PaintEvent {
        LayerStarted(Bounds<Pixels>),
        BackdropBlur {
            bounds: Bounds<Pixels>,
            corner_radii: Corners<Pixels>,
            blur_radius: Pixels,
        },
        ChildPainted,
        LayerFinished,
    }

    #[derive(Default)]
    struct RecordingPaintTarget {
        events: Vec<PaintEvent>,
    }

    impl MaterialPaintTarget for RecordingPaintTarget {
        fn paint_layer<R>(
            &mut self,
            bounds: Bounds<Pixels>,
            paint: impl FnOnce(&mut Self) -> R,
        ) -> R {
            self.events.push(PaintEvent::LayerStarted(bounds));
            let result = paint(self);
            self.events.push(PaintEvent::LayerFinished);
            result
        }

        fn paint_backdrop_blur(
            &mut self,
            bounds: Bounds<Pixels>,
            corner_radii: Corners<Pixels>,
            blur_radius: Pixels,
        ) {
            self.events.push(PaintEvent::BackdropBlur {
                bounds,
                corner_radii,
                blur_radius,
            });
        }

        fn paint_child(&mut self, _: &mut AnyElement, _: &mut App) {
            self.events.push(PaintEvent::ChildPainted);
        }
    }

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
    fn nonzero_material_paints_expected_blur_before_child(cx: &mut TestAppContext) {
        let bounds = Bounds::new(point(px(13.), px(17.)), size(px(123.), px(47.)));
        let corner_radii = Corners {
            top_left: px(3.),
            top_right: px(5.),
            bottom_right: px(7.),
            bottom_left: px(11.),
        };
        let mut material = Material::new("recorded", MaterialDepth::Overlay, gpui::Empty)
            .corner_radii(corner_radii);
        let mut target = RecordingPaintTarget::default();

        cx.update(|cx| {
            material.paint_resolved(
                bounds,
                Some(ResolvedMaterial {
                    blur_radius: px(44.),
                }),
                &mut target,
                cx,
            );
        });

        assert_eq!(
            target.events,
            vec![
                PaintEvent::LayerStarted(bounds),
                PaintEvent::BackdropBlur {
                    bounds,
                    corner_radii,
                    blur_radius: px(44.),
                },
                PaintEvent::ChildPainted,
                PaintEvent::LayerFinished,
            ]
        );
    }

    #[gpui::test]
    fn opaque_material_bypasses_the_layer(cx: &mut TestAppContext) {
        let mut material = Material::new("recorded", MaterialDepth::Panel, gpui::Empty);
        let mut target = RecordingPaintTarget::default();

        cx.update(|cx| material.paint_resolved(bounds(), None, &mut target, cx));

        assert_eq!(target.events, vec![PaintEvent::ChildPainted]);
    }

    #[gpui::test]
    fn zero_and_nonfinite_materials_keep_the_layer_without_blur(cx: &mut TestAppContext) {
        for resolved in [
            Some(ResolvedMaterial {
                blur_radius: Pixels::ZERO,
            }),
            Some(ResolvedMaterial {
                blur_radius: px(f32::NAN),
            }),
        ] {
            let mut material = Material::new("recorded", MaterialDepth::Panel, gpui::Empty);
            let mut target = RecordingPaintTarget::default();

            cx.update(|cx| material.paint_resolved(bounds(), resolved, &mut target, cx));

            assert_eq!(
                target.events,
                vec![
                    PaintEvent::LayerStarted(bounds()),
                    PaintEvent::ChildPainted,
                    PaintEvent::LayerFinished,
                ]
            );
        }
    }

    fn bounds() -> Bounds<Pixels> {
        Bounds::new(point(px(2.), px(3.)), size(px(80.), px(50.)))
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
