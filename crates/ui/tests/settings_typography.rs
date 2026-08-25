use gpui::{
    AppContext as _, Bounds, Context, DevicePixels, Font, FontId, FontMetrics, FontRun, GlyphId,
    HeadlessAppContext, NoopTextSystem, Pixels, PlatformTextSystem, Render, RenderGlyphParams,
    Result, Size as GeometricSize, TextRenderingMode, Window, px,
};
use gpui_neath::{
    Sizable as _, Size,
    setting::{SettingField, SettingGroup, SettingItem, SettingPage, Settings},
};
use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
};

const PAGE_DESCRIPTION: &str = "Typography page description";
const GROUP_TITLE: &str = "Typography group title";
const GROUP_DESCRIPTION: &str = "Typography group description";
const ITEM_TITLE: &str = "Typography item title";
const ITEM_DESCRIPTION: &str = "Typography item description";
const SIDEBAR_PAGE_TITLE: &str = "Sidebar-only page title";

#[derive(Clone, Debug)]
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

    fn advance(&self, font_id: FontId, glyph_id: GlyphId) -> Result<GeometricSize<f32>> {
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
    ) -> Result<(GeometricSize<DevicePixels>, Vec<u8>)> {
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

struct SettingsTypographyHarness {
    size: Size,
}

impl Render for SettingsTypographyHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        Settings::new("settings-typography")
            .with_size(self.size)
            .page(
                SettingPage::new("Typography")
                    .description(PAGE_DESCRIPTION)
                    .resettable(false)
                    .group(
                        SettingGroup::new()
                            .title(GROUP_TITLE)
                            .description(GROUP_DESCRIPTION)
                            .item(
                                SettingItem::new(
                                    ITEM_TITLE,
                                    SettingField::switch(|_| false, |_, _| {}),
                                )
                                .description(ITEM_DESCRIPTION),
                            ),
                    )
                    .group(
                        SettingGroup::new()
                            .title("Second group title")
                            .item(SettingItem::new(
                                "Second item title",
                                SettingField::switch(|_| false, |_, _| {}),
                            )),
                    ),
            )
            .page(
                SettingPage::new(SIDEBAR_PAGE_TITLE)
                    .resettable(false)
                    .group(SettingGroup::new().item(SettingItem::new(
                        "Inactive page item",
                        SettingField::switch(|_| false, |_, _| {}),
                    ))),
            )
    }
}

fn draw_settings(size: Size) -> Vec<LayoutCall> {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let text_system: Arc<dyn PlatformTextSystem> =
        Arc::new(RecordingTextSystem::new(calls.clone()));
    let mut cx = HeadlessAppContext::new(text_system);
    cx.update(gpui_neath::init);
    let window = cx
        .open_window(gpui::size(px(800.), px(600.)), move |_, cx| {
            cx.new(|_| SettingsTypographyHarness { size })
        })
        .expect("settings typography window opens");
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .expect("settings typography window draws");
    calls.lock().expect("layout recorder lock").clone()
}

fn assert_role_sizes(calls: &[LayoutCall], expected: Pixels) {
    for role in [
        PAGE_DESCRIPTION,
        GROUP_TITLE,
        GROUP_DESCRIPTION,
        ITEM_TITLE,
        ITEM_DESCRIPTION,
        SIDEBAR_PAGE_TITLE,
    ] {
        let actual = calls
            .iter()
            .filter(|call| call.text == role)
            .map(|call| call.font_size)
            .collect::<Vec<_>>();
        assert!(!actual.is_empty(), "{role:?} was not laid out: {calls:?}");
        assert!(
            actual.iter().all(|size| *size == expected),
            "{role:?} must follow Settings::size; got {actual:?}, expected {expected:?}"
        );
    }
}

#[test]
fn page_group_and_item_text_follow_the_settings_size() {
    assert_role_sizes(&draw_settings(Size::Small), px(12.));
    assert_role_sizes(&draw_settings(Size::Medium), px(13.));
    assert_role_sizes(&draw_settings(Size::Large), px(15.));
}
