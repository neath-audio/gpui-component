use gpui::{
    AppContext as _, Bounds, Context, DevicePixels, Font, FontId, FontMetrics, FontRun, GlyphId,
    HeadlessAppContext, NoopTextSystem, ParentElement as _, Pixels, PlatformTextSystem, Render,
    RenderGlyphParams, Result, Size, Styled as _, TestAppContext, TextRenderingMode, Window, div,
    px,
};
use gpui_neath::{
    TruncateMiddleExt as _, middle_truncating_cell_sized, truncating_cell, truncating_cell_sized,
};
use std::{
    borrow::Cow,
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

const CELL_TEXT: &str = "Wind through canyon / archive 2026";
const SIZED_CELL_TEXT: &str = "Field recording take 17";
const MIDDLE_CELL_TEXT: &str = "/sessions/2026/canyon/ambience.wav";

#[derive(Clone, Copy)]
enum TruncationProbeKind {
    TruncatingCell,
    TruncatingCellSized,
    MiddleTruncatingCell,
}

fn public_cell(kind: TruncationProbeKind) -> gpui::Div {
    match kind {
        TruncationProbeKind::TruncatingCell => truncating_cell(CELL_TEXT),
        TruncationProbeKind::TruncatingCellSized => truncating_cell_sized(SIZED_CELL_TEXT, px(11.)),
        TruncationProbeKind::MiddleTruncatingCell => {
            middle_truncating_cell_sized(MIDDLE_CELL_TEXT, px(11.))
        }
    }
}

fn expected_cell(kind: TruncationProbeKind) -> gpui::Div {
    match kind {
        TruncationProbeKind::TruncatingCell => div()
            .size_full()
            .flex()
            .items_center()
            .overflow_hidden()
            .text_size(gpui_neath::Size::Small.text_size())
            .child(div().flex_1().min_w_0().truncate().child(CELL_TEXT)),
        TruncationProbeKind::TruncatingCellSized => div()
            .size_full()
            .flex()
            .items_center()
            .overflow_hidden()
            .text_size(px(11.))
            .child(div().flex_1().min_w_0().truncate().child(SIZED_CELL_TEXT)),
        TruncationProbeKind::MiddleTruncatingCell => div()
            .size_full()
            .flex()
            .items_center()
            .overflow_hidden()
            .text_size(px(11.))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis_middle()
                    .child(MIDDLE_CELL_TEXT),
            ),
    }
}

struct CompositionProbe {
    kind: TruncationProbeKind,
    actual: Rc<RefCell<Vec<Bounds<Pixels>>>>,
    expected: Rc<RefCell<Vec<Bounds<Pixels>>>>,
}

impl CompositionProbe {
    fn capture(element: gpui::Div, bounds: Rc<RefCell<Vec<Bounds<Pixels>>>>) -> gpui::Div {
        element.on_children_prepainted(move |children, _, _| {
            *bounds.borrow_mut() = children;
        })
    }
}

impl Render for CompositionProbe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        let actual = public_cell(self.kind);
        let expected = expected_cell(self.kind);

        div()
            .size_full()
            .child(Self::capture(actual, self.actual.clone()))
            .child(Self::capture(expected, self.expected.clone()))
    }
}

#[derive(Clone, Debug, PartialEq)]
struct LayoutCall {
    text: String,
    font_size: Pixels,
}

struct RecordingTextSystem {
    inner: NoopTextSystem,
    calls: Arc<Mutex<Vec<LayoutCall>>>,
}

impl RecordingTextSystem {
    fn new(calls: Arc<Mutex<Vec<LayoutCall>>>) -> Self {
        Self {
            inner: NoopTextSystem::new(),
            calls,
        }
    }
}

impl PlatformTextSystem for RecordingTextSystem {
    fn add_fonts(&self, fonts: Vec<Cow<'static, [u8]>>) -> Result<()> {
        self.inner.add_fonts(fonts)
    }

    fn all_font_names(&self) -> Vec<String> {
        self.inner.all_font_names()
    }

    fn font_id(&self, descriptor: &Font) -> Result<FontId> {
        self.inner.font_id(descriptor)
    }

    fn font_metrics(&self, font_id: FontId) -> FontMetrics {
        self.inner.font_metrics(font_id)
    }

    fn typographic_bounds(&self, font_id: FontId, glyph_id: GlyphId) -> Result<Bounds<f32>> {
        self.inner.typographic_bounds(font_id, glyph_id)
    }

    fn advance(&self, font_id: FontId, glyph_id: GlyphId) -> Result<Size<f32>> {
        self.inner.advance(font_id, glyph_id)
    }

    fn glyph_for_char(&self, font_id: FontId, ch: char) -> Option<GlyphId> {
        self.inner.glyph_for_char(font_id, ch)
    }

    fn glyph_raster_bounds(&self, params: &RenderGlyphParams) -> Result<Bounds<DevicePixels>> {
        self.inner.glyph_raster_bounds(params)
    }

    fn rasterize_glyph(
        &self,
        params: &RenderGlyphParams,
        raster_bounds: Bounds<DevicePixels>,
    ) -> Result<(Size<DevicePixels>, Vec<u8>)> {
        self.inner.rasterize_glyph(params, raster_bounds)
    }

    fn layout_line(&self, text: &str, font_size: Pixels, runs: &[FontRun]) -> gpui::LineLayout {
        self.calls
            .lock()
            .expect("layout recorder lock")
            .push(LayoutCall {
                text: text.to_owned(),
                font_size,
            });
        self.inner.layout_line(text, font_size, runs)
    }

    fn recommended_rendering_mode(&self, font_id: FontId, font_size: Pixels) -> TextRenderingMode {
        self.inner.recommended_rendering_mode(font_id, font_size)
    }
}

struct NarrowTruncationProbe {
    kind: TruncationProbeKind,
}

impl Render for NarrowTruncationProbe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        div().w(px(64.)).h(px(20.)).child(public_cell(self.kind))
    }
}

fn final_ellipsis_layout(calls: &[LayoutCall]) -> &str {
    &calls
        .iter()
        .rev()
        .find(|call| call.text.contains('…'))
        .unwrap_or_else(|| panic!("narrow cell did not request an ellipsized layout: {calls:?}"))
        .text
}

fn draw_narrow_truncation(kind: TruncationProbeKind) -> String {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let text_system: Arc<dyn PlatformTextSystem> =
        Arc::new(RecordingTextSystem::new(calls.clone()));
    let mut cx = HeadlessAppContext::new(text_system);
    cx.update(gpui_neath::init);
    let window = cx
        .open_window(gpui::size(px(64.), px(20.)), move |_, cx| {
            cx.new(|_| NarrowTruncationProbe { kind })
        })
        .expect("narrow headless window opens");
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .expect("narrow headless window draws");
    final_ellipsis_layout(&calls.lock().expect("layout recorder lock")).to_owned()
}

fn assert_public_cell_children(cx: &mut TestAppContext, kind: TruncationProbeKind) {
    let actual = Rc::new(RefCell::new(Vec::new()));
    let expected = Rc::new(RefCell::new(Vec::new()));
    let window = cx.open_window(gpui::size(px(320.), px(160.)), {
        let actual = actual.clone();
        let expected = expected.clone();
        move |_, _| CompositionProbe {
            kind,
            actual,
            expected,
        }
    });
    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .expect("truncation probe window draws");

    let actual = actual.borrow();
    let expected = expected.borrow();
    assert_eq!(actual.len(), 1, "actual child count");
    assert_eq!(actual.len(), expected.len(), "reference child count");
    for (actual, expected) in actual.iter().zip(expected.iter()) {
        assert_eq!(actual.size, expected.size, "ordered child bounds");
    }
}

#[test]
fn truncate_middle_composes_the_existing_gpui_contract() {
    let mut actual = div().truncate_middle();
    let mut expected = div()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis_middle();
    assert_eq!(actual.style().clone(), expected.style().clone());
}

#[test]
fn truncating_cells_preserve_outer_size_and_overflow() {
    let mut fixed = truncating_cell("name");
    let mut expected_fixed = div()
        .size_full()
        .flex()
        .items_center()
        .overflow_hidden()
        .text_size(gpui_neath::Size::Small.text_size());
    assert_eq!(fixed.style().clone(), expected_fixed.style().clone());

    let mut sized = truncating_cell_sized("name", px(11.));
    let mut expected_sized = div()
        .size_full()
        .flex()
        .items_center()
        .overflow_hidden()
        .text_size(px(11.));
    assert_eq!(sized.style().clone(), expected_sized.style().clone());

    let mut middle = middle_truncating_cell_sized("/long/path/name", px(11.));
    assert_eq!(middle.style().clone(), expected_sized.style().clone());
}

#[gpui::test]
fn public_truncating_cells_render_their_text_children(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    for kind in [
        TruncationProbeKind::TruncatingCell,
        TruncationProbeKind::TruncatingCellSized,
        TruncationProbeKind::MiddleTruncatingCell,
    ] {
        assert_public_cell_children(cx, kind);
    }
}

#[test]
fn public_truncating_cells_rewrite_visible_text_in_their_declared_direction() {
    let end = draw_narrow_truncation(TruncationProbeKind::TruncatingCell);
    assert!(
        end.starts_with("Wind"),
        "end layout keeps the prefix: {end:?}"
    );
    assert!(
        end.ends_with('…'),
        "end layout ends at the ellipsis: {end:?}"
    );
    assert!(
        !end.ends_with("2026"),
        "end layout must not preserve the trailing segment: {end:?}",
    );

    let sized_end = draw_narrow_truncation(TruncationProbeKind::TruncatingCellSized);
    assert!(
        sized_end.starts_with("Field"),
        "sized end layout keeps the prefix: {sized_end:?}",
    );
    assert!(
        sized_end.ends_with('…'),
        "sized end layout ends at the ellipsis: {sized_end:?}",
    );
    assert!(
        !sized_end.ends_with("17"),
        "sized end layout must not preserve the trailing segment: {sized_end:?}",
    );

    let middle = draw_narrow_truncation(TruncationProbeKind::MiddleTruncatingCell);
    assert!(
        middle.starts_with('/'),
        "middle layout keeps the leading segment: {middle:?}",
    );
    assert!(
        middle.contains('…'),
        "middle layout includes its ellipsis: {middle:?}",
    );
    assert!(
        middle.ends_with("av"),
        "middle layout keeps the trailing segment: {middle:?}",
    );
}
