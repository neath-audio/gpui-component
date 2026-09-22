use gpui::{
    AppContext as _, Bounds, Context, DevicePixels, Font, FontId, FontMetrics, FontRun, GlyphId,
    HeadlessAppContext, NoopTextSystem, Pixels, PlatformTextSystem, Render, RenderGlyphParams,
    Result, Size as GeometricSize, TextRenderingMode, Window, px,
};
use gpui_neath::Size;
use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
};

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

use gpui::{ParentElement as _, Rems, Styled as _, prelude::FluentBuilder as _};
use gpui_neath::{
    Root, Sizable as _, Theme,
    accordion::AccordionItem,
    button::Toggle,
    checkbox::Checkbox,
    input::{OtpInput, OtpState},
    label::Label,
    radio::Radio,
    switch::Switch,
    tab::Tab,
    v_flex,
};

const CONTROL_LABELS: [&str; 7] = [
    "Checkbox",
    "Radio",
    "Switch",
    "Toggle",
    "Tab",
    "Label",
    "Accordion",
];

struct ControlTypographyHarness {
    otp: gpui::Entity<OtpState>,
    size: Option<Size>,
    rem_size: Pixels,
    override_size: Option<Rems>,
}

impl ControlTypographyHarness {
    fn control(
        &self,
        element: impl gpui_neath::Sizable + gpui::Styled + gpui::IntoElement,
    ) -> gpui::AnyElement {
        element
            .when_some(self.size, |this, size| this.with_size(size))
            .when_some(self.override_size, |this, size| this.text_size(size))
            .into_any_element()
    }
}

impl Render for ControlTypographyHarness {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        window.set_rem_size(self.rem_size);
        v_flex()
            .gap_2()
            .child(self.control(Checkbox::new("checkbox").label(CONTROL_LABELS[0])))
            .child(self.control(Radio::new("radio").label(CONTROL_LABELS[1])))
            .child(self.control(Switch::new("switch").label(CONTROL_LABELS[2])))
            .child(self.control(Toggle::new("toggle").child(CONTROL_LABELS[3])))
            .child(self.control(Tab::new().label(CONTROL_LABELS[4])))
            .child(self.control(Label::new(CONTROL_LABELS[5])))
            .child(self.control(AccordionItem::new().title(CONTROL_LABELS[6])))
            .child(OtpInput::new(&self.otp).when_some(self.size, |this, size| this.with_size(size)))
            .child(
                gpui_neath::description_list::DescriptionList::new()
                    .when_some(self.size, |this, size| {
                        gpui_neath::Sizable::with_size(this, size)
                    })
                    .item("Description label", "Description value", 1),
            )
    }
}

fn draw_controls(
    size: Option<Size>,
    rem_size: Pixels,
    override_size: Option<Rems>,
) -> Vec<LayoutCall> {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let text_system: Arc<dyn PlatformTextSystem> =
        Arc::new(RecordingTextSystem::new(calls.clone()));
    let mut cx = HeadlessAppContext::new(text_system);
    cx.update(gpui_neath::init);
    let window = cx
        .open_window(gpui::size(px(800.), px(600.)), move |window, cx| {
            let otp = cx.new(|cx| OtpState::new(1, window, cx).default_value("7"));
            cx.new(|_| ControlTypographyHarness {
                otp,
                size,
                rem_size,
                override_size,
            })
        })
        .expect("control typography window opens");
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .expect("control typography window draws");
    calls.lock().expect("layout recorder lock").clone()
}

fn assert_control_sizes(calls: &[LayoutCall], expected: Pixels) {
    for label in CONTROL_LABELS {
        let sizes = calls
            .iter()
            .filter(|call| call.text == label)
            .map(|call| call.font_size)
            .collect::<Vec<_>>();
        assert!(!sizes.is_empty(), "{label:?} was not rendered: {calls:?}");
        assert!(
            sizes.iter().all(|size| *size == expected),
            "{label}: rendered {sizes:?}, expected {expected:?}"
        );
    }
}

#[test]
fn built_in_control_labels_share_the_size_contract() {
    for rem_size in [px(16.), px(20.)] {
        for size in [
            Size::XSmall,
            Size::Small,
            Size::Medium,
            Size::Large,
            Size::Size(px(17.)),
        ] {
            let calls = draw_controls(Some(size), rem_size, None);
            let expected = size.control_text_size().to_pixels(rem_size);
            assert_control_sizes(&calls, expected);
        }
    }
}

#[test]
fn description_and_otp_text_use_the_shared_size_contract() {
    for rem_size in [px(16.), px(20.)] {
        for size in [
            Size::XSmall,
            Size::Small,
            Size::Medium,
            Size::Large,
            Size::Size(px(17.)),
        ] {
            let calls = draw_controls(Some(size), rem_size, None);
            let expected = size.control_text_size().to_pixels(rem_size);
            for label in ["Description label", "Description value"] {
                let call = calls
                    .iter()
                    .find(|call| call.text == label)
                    .expect("sized text was rendered");
                assert_eq!(call.font_size, expected, "{label}");
            }
            if !matches!(size, Size::Size(_)) {
                let otp = calls
                    .iter()
                    .find(|call| call.text == "7")
                    .expect("OTP text was rendered");
                assert_eq!(otp.font_size, expected, "OTP named size");
            }
        }
    }
}

#[test]
fn default_control_labels_use_medium_type() {
    assert_control_sizes(
        &draw_controls(None, px(16.), None),
        Size::Medium.control_text_size().to_pixels(px(16.)),
    );
}

#[test]
fn explicit_control_text_overrides_win() {
    let override_size = gpui::rems(1.0625);
    assert_control_sizes(
        &draw_controls(Some(Size::Small), px(16.), Some(override_size)),
        px(17.),
    );
}

#[test]
fn semantic_typography_projects_the_same_scale_without_changing_zoom() {
    let mut theme = Theme::default();
    for rem_size in [px(16.), px(20.)] {
        theme.font_size = rem_size;
        let tokens = theme.semantic_tokens();
        for (token, size) in [
            (tokens.typography.xs, Size::XSmall),
            (tokens.typography.sm, Size::Small),
            (tokens.typography.md, Size::Medium),
            (tokens.typography.lg, Size::Large),
        ] {
            assert_eq!(token.size, size.text_size().to_pixels(rem_size));
        }
        assert_eq!(tokens.typography.mono_md.size, theme.mono_font_size);
        theme.apply_semantic_tokens(&tokens);
        assert_eq!(
            theme.font_size, rem_size,
            "round-tripping tokens must not resize the UI"
        );
    }
}

#[test]
fn semantic_medium_size_remains_a_body_text_size() {
    let mut theme = Theme::default();
    let mut tokens = theme.semantic_tokens();
    tokens.typography.md.size = px(15.);
    theme.apply_semantic_tokens(&tokens);
    assert_eq!(theme.typography_tokens().md.size, px(15.));
    assert_eq!(Size::Medium.text_size().to_pixels(theme.font_size), px(15.));
}

struct CodeTypographyHarness;

impl Render for CodeTypographyHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        gpui_neath::text::TextView::markdown("code-typography", "```\nsample_code\n```")
    }
}

#[test]
fn rich_text_code_blocks_use_the_projected_monospace_size() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let text_system: Arc<dyn PlatformTextSystem> =
        Arc::new(RecordingTextSystem::new(calls.clone()));
    let mut cx = HeadlessAppContext::new(text_system);
    cx.update(|cx| {
        gpui_neath::init(cx);
        Theme::global_mut(cx).mono_font_size = px(11.);
        Theme::sync_base(cx);
    });
    let window = cx
        .open_window(gpui::size(px(800.), px(600.)), |_, cx| {
            cx.new(|_| CodeTypographyHarness)
        })
        .expect("code typography window opens");
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .expect("code typography window draws");
    let calls = calls.lock().expect("layout recorder lock");
    let code = calls
        .iter()
        .filter(|call| call.text.trim() == "sample_code")
        .collect::<Vec<_>>();
    assert!(!code.is_empty(), "code block was not rendered: {calls:?}");
    assert!(
        code.iter().all(|call| call.font_size == px(11.)),
        "{code:?}"
    );
}

struct RootTypographyHarness;

impl Render for RootTypographyHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        gpui::div().child("Root body text")
    }
}

#[test]
fn root_body_text_uses_theme_medium_without_resizing_rem() {
    for rem_size in [px(16.), px(20.)] {
        for override_size in [None, Some(px(17.))] {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let text_system: Arc<dyn PlatformTextSystem> =
                Arc::new(RecordingTextSystem::new(calls.clone()));
            let mut cx = HeadlessAppContext::new(text_system);
            cx.update(|cx| {
                gpui_neath::init(cx);
                Theme::global_mut(cx).font_size = rem_size;
                Theme::sync_base(cx);
            });
            let window = cx
                .open_window(gpui::size(px(800.), px(600.)), |window, cx| {
                    let content = cx.new(|_| RootTypographyHarness);
                    cx.new(|cx| {
                        let root = Root::new(content, window, cx).bordered(false);
                        match override_size {
                            Some(size) => root.text_size(size),
                            None => root,
                        }
                    })
                })
                .expect("root typography window opens");
            cx.update_window(window.into(), |_, window, cx| {
                window.draw(cx).clear(cx);
                assert_eq!(window.rem_size(), rem_size);
            })
            .expect("root typography window draws");
            let calls = calls.lock().expect("layout recorder lock");
            let body = calls
                .iter()
                .filter(|call| call.text == "Root body text")
                .collect::<Vec<_>>();
            assert!(!body.is_empty(), "body text was not rendered: {calls:?}");
            let expected =
                override_size.unwrap_or_else(|| Size::Medium.text_size().to_pixels(rem_size));
            assert!(
                body.iter().all(|call| call.font_size == expected),
                "{body:?}, expected {expected:?}"
            );
        }
    }
}

#[test]
fn custom_otp_cell_size_keeps_proportional_text() {
    for rem_size in [px(16.), px(20.)] {
        let calls = draw_controls(Some(Size::Size(px(55.))), rem_size, None);
        let otp = calls
            .iter()
            .find(|call| call.text == "7")
            .expect("OTP text was rendered");
        assert_eq!(
            otp.font_size,
            px(27.5),
            "55px OTP cells retain half-size text independently of rem"
        );
    }
}
