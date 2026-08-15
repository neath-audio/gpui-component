use gpui::{
    App, AppContext as _, Bounds, BoxShadow, Context, DevicePixels, Font, FontId, FontMetrics,
    FontRun, FontWeight, GlyphId, HeadlessAppContext, Hsla, LineLayout, NoopTextSystem,
    ParentElement as _, Pixels, PlatformTextSystem, Render, RenderGlyphParams, Result, Size,
    Styled as _, TestAppContext, TextRenderingMode, Window, div, hsla, linear_color_stop,
    linear_gradient, point, px, relative, rems,
};
use gpui_neath::{
    ActiveTheme as _, ElementExt as _, Sizable as _, Theme, ThemeToken,
    button::Button,
    h_flex,
    style::{
        recipes, tokens,
        typography::{
            TruncateMiddleExt as _, middle_truncating_cell_sized, truncating_cell,
            truncating_cell_sized,
        },
    },
    v_flex,
};
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{Arc, Mutex},
};

const CELL_TEXT: &str = "Wind through canyon / archive 2026";
const SIZED_CELL_TEXT: &str = "Field recording take 17";
const MIDDLE_CELL_TEXT: &str = "/sessions/2026/canyon/ambience.wav";
const BODY_TEXT: &str = "Body prose has nine tokens";
const BODY_MUTED_TEXT: &str = "Muted prose has nine tokens";
const DIALOG_PROSE_TEXT: &str = "Dialog prose asks one clear question";
const VALUE_TEXT: &str = "-14.2 dB";
const DENSE_VALUE_TEXT: &str = "127.4 ms";
const CAPTION_TEXT: &str = "127 files · 42.0 MiB";
const CONTROL_LABEL_TEXT: &str = "Enable loudness normalization";
const SECTION_TEXT: &str = "Advanced metadata";
const REQUIRED_TEXT: &str = "Destination folder";
const POPOVER_LABEL: &str = "Wet / Dry Mix";

#[derive(Clone, Copy)]
enum CompositionProbeKind {
    TruncatingCell,
    TruncatingCellSized,
    MiddleTruncatingCell,
    Body,
    BodyMuted,
    DialogProse,
    Value,
    ValueDense,
    Caption,
    ControlLabel,
    SectionLabel,
    RequiredLabel,
    PopoverRow,
}

const COMPOSITION_PROBES: &[CompositionProbeKind] = &[
    CompositionProbeKind::TruncatingCell,
    CompositionProbeKind::TruncatingCellSized,
    CompositionProbeKind::MiddleTruncatingCell,
    CompositionProbeKind::Body,
    CompositionProbeKind::BodyMuted,
    CompositionProbeKind::DialogProse,
    CompositionProbeKind::Value,
    CompositionProbeKind::ValueDense,
    CompositionProbeKind::Caption,
    CompositionProbeKind::ControlLabel,
    CompositionProbeKind::SectionLabel,
    CompositionProbeKind::RequiredLabel,
    CompositionProbeKind::PopoverRow,
];

fn public_recipe(kind: CompositionProbeKind, cx: &App) -> gpui::Div {
    match kind {
        CompositionProbeKind::TruncatingCell => truncating_cell(CELL_TEXT),
        CompositionProbeKind::TruncatingCellSized => {
            truncating_cell_sized(SIZED_CELL_TEXT, px(11.))
        }
        CompositionProbeKind::MiddleTruncatingCell => {
            middle_truncating_cell_sized(MIDDLE_CELL_TEXT, px(11.))
        }
        CompositionProbeKind::Body => recipes::body(BODY_TEXT, cx),
        CompositionProbeKind::BodyMuted => recipes::body_muted(BODY_MUTED_TEXT, cx),
        CompositionProbeKind::DialogProse => recipes::dialog_prose(DIALOG_PROSE_TEXT, cx),
        CompositionProbeKind::Value => recipes::value(VALUE_TEXT, cx),
        CompositionProbeKind::ValueDense => recipes::value_dense(DENSE_VALUE_TEXT, cx),
        CompositionProbeKind::Caption => recipes::caption(CAPTION_TEXT, cx),
        CompositionProbeKind::ControlLabel => recipes::control_label(CONTROL_LABEL_TEXT, cx),
        CompositionProbeKind::SectionLabel => recipes::section_label(SECTION_TEXT, cx),
        CompositionProbeKind::RequiredLabel => recipes::required_label(REQUIRED_TEXT, cx),
        CompositionProbeKind::PopoverRow => {
            recipes::popover_row(POPOVER_LABEL, div().w(px(17.)), cx)
        }
    }
}

fn expected_public_recipe(kind: CompositionProbeKind, cx: &App) -> gpui::Div {
    match kind {
        CompositionProbeKind::TruncatingCell => div()
            .size_full()
            .flex()
            .items_center()
            .overflow_hidden()
            .text_xs()
            .child(div().flex_1().min_w_0().truncate().child(CELL_TEXT)),
        CompositionProbeKind::TruncatingCellSized => div()
            .size_full()
            .flex()
            .items_center()
            .overflow_hidden()
            .text_size(px(11.))
            .child(div().flex_1().min_w_0().truncate().child(SIZED_CELL_TEXT)),
        CompositionProbeKind::MiddleTruncatingCell => div()
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
        CompositionProbeKind::Body => div()
            .text_size(tokens::TEXT_12)
            .text_color(cx.theme().foreground)
            .child(BODY_TEXT),
        CompositionProbeKind::BodyMuted => div()
            .text_size(tokens::TEXT_12)
            .text_color(cx.theme().muted_foreground)
            .child(BODY_MUTED_TEXT),
        CompositionProbeKind::DialogProse => div()
            .text_size(tokens::TEXT_14)
            .text_color(cx.theme().muted_foreground)
            .child(DIALOG_PROSE_TEXT),
        CompositionProbeKind::Value => div()
            .text_size(tokens::TEXT_12)
            .text_color(cx.theme().foreground)
            .child(VALUE_TEXT),
        CompositionProbeKind::ValueDense => div()
            .text_size(tokens::TEXT_10)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(cx.theme().foreground)
            .child(DENSE_VALUE_TEXT),
        CompositionProbeKind::Caption => div()
            .text_size(tokens::TEXT_10)
            .text_color(cx.theme().muted_foreground)
            .child(CAPTION_TEXT),
        CompositionProbeKind::ControlLabel => div()
            .text_size(tokens::TEXT_14)
            .line_height(relative(1.))
            .text_color(cx.theme().foreground)
            .child(CONTROL_LABEL_TEXT),
        CompositionProbeKind::SectionLabel => div()
            .text_size(tokens::TEXT_13)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(cx.theme().muted_foreground)
            .child(SECTION_TEXT),
        CompositionProbeKind::RequiredLabel => h_flex()
            .gap(px(4.))
            .child(
                div()
                    .text_size(tokens::TEXT_13)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(cx.theme().muted_foreground)
                    .child(REQUIRED_TEXT),
            )
            .child(
                div()
                    .text_size(tokens::TEXT_13)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(cx.theme().danger)
                    .child("*"),
            ),
        CompositionProbeKind::PopoverRow => h_flex()
            .w_full()
            .h(px(24.))
            .flex_none()
            .items_center()
            .justify_between()
            .gap(tokens::SPACE_2)
            .child(
                div()
                    .text_color(cx.theme().muted_foreground)
                    .child(POPOVER_LABEL),
            )
            .child(div().w(px(17.))),
    }
}

struct CompositionProbe {
    kind: CompositionProbeKind,
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
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let actual = public_recipe(self.kind, cx);
        let expected = expected_public_recipe(self.kind, cx);

        v_flex()
            .size_full()
            .gap(px(8.))
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

    fn layout_line(&self, text: &str, font_size: Pixels, runs: &[FontRun]) -> LineLayout {
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

struct TextProbe {
    kind: CompositionProbeKind,
}

impl Render for TextProbe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        public_recipe(self.kind, cx)
    }
}

struct NarrowTruncationProbe {
    kind: CompositionProbeKind,
}

impl Render for NarrowTruncationProbe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        div().w(px(64.)).h(px(20.)).child(match self.kind {
            CompositionProbeKind::TruncatingCell => truncating_cell(CELL_TEXT),
            CompositionProbeKind::TruncatingCellSized => {
                truncating_cell_sized(SIZED_CELL_TEXT, px(11.))
            }
            CompositionProbeKind::MiddleTruncatingCell => {
                middle_truncating_cell_sized(MIDDLE_CELL_TEXT, px(11.))
            }
            _ => unreachable!("only public truncation cells are probed narrowly"),
        })
    }
}

fn expected_layouts() -> Vec<LayoutCall> {
    [
        (CELL_TEXT, px(12.)),
        (SIZED_CELL_TEXT, px(11.)),
        (MIDDLE_CELL_TEXT, px(11.)),
        (BODY_TEXT, px(12.)),
        (BODY_MUTED_TEXT, px(12.)),
        (DIALOG_PROSE_TEXT, px(14.)),
        (VALUE_TEXT, px(12.)),
        (DENSE_VALUE_TEXT, px(10.)),
        (CAPTION_TEXT, px(10.)),
        (CONTROL_LABEL_TEXT, px(14.)),
        (SECTION_TEXT, px(13.)),
        (REQUIRED_TEXT, px(13.)),
        ("*", px(13.)),
        (POPOVER_LABEL, px(16.)),
    ]
    .into_iter()
    .map(|(text, font_size)| LayoutCall {
        text: text.to_owned(),
        font_size,
    })
    .collect()
}

fn final_ellipsis_layout(calls: &[LayoutCall]) -> &str {
    &calls
        .iter()
        .rev()
        .find(|call| call.text.contains('…'))
        .unwrap_or_else(|| panic!("narrow cell did not request an ellipsized layout: {calls:?}"))
        .text
}

fn draw_narrow_truncation(kind: CompositionProbeKind) -> String {
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

struct PrivateCompositionAttachment {
    directly_attached: bool,
    bound_outputs: Vec<String>,
    attached_bindings: Vec<String>,
}

fn unwrapped_expression(mut expression: &syn::Expr) -> &syn::Expr {
    loop {
        expression = match expression {
            syn::Expr::Group(group) => &group.expr,
            syn::Expr::Paren(paren) => &paren.expr,
            _ => return expression,
        };
    }
}

fn direct_helper_call(expression: &syn::Expr, helper_name: &str) -> bool {
    let syn::Expr::Call(call) = unwrapped_expression(expression) else {
        return false;
    };
    let syn::Expr::Path(path) = unwrapped_expression(&call.func) else {
        return false;
    };
    path.qself.is_none()
        && path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == helper_name)
}

fn direct_binding(expression: &syn::Expr) -> Option<String> {
    let syn::Expr::Path(path) = unwrapped_expression(expression) else {
        return None;
    };
    (path.qself.is_none() && path.path.leading_colon.is_none() && path.path.segments.len() == 1)
        .then(|| path.path.segments[0].ident.to_string())
}

fn pattern_bindings(pattern: &syn::Pat, bindings: &mut Vec<String>) {
    struct BindingCollector<'a> {
        bindings: &'a mut Vec<String>,
    }

    impl<'ast> syn::visit::Visit<'ast> for BindingCollector<'_> {
        fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
            self.bindings.push(pattern.ident.to_string());
            syn::visit::visit_pat_ident(self, pattern);
        }
    }

    syn::visit::Visit::visit_pat(&mut BindingCollector { bindings }, pattern);
}

fn private_composition_attachment(
    source: &str,
    function_name: &str,
    helper_name: &str,
) -> PrivateCompositionAttachment {
    let file = syn::parse_file(source).expect("typography and recipe source parses");
    let function = file
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Fn(function) if function.sig.ident == function_name => Some(function),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing public function {function_name}"));

    let mut bound_outputs = Vec::new();
    for statement in &function.block.stmts {
        let syn::Stmt::Local(local) = statement else {
            continue;
        };
        let Some(initializer) = &local.init else {
            continue;
        };
        if direct_helper_call(&initializer.expr, helper_name) {
            pattern_bindings(&local.pat, &mut bound_outputs);
        }
    }

    let returned = match function.block.stmts.last() {
        Some(syn::Stmt::Expr(expression, None)) => expression,
        _ => panic!("{function_name} must return its composed element as a tail expression"),
    };

    struct ReturnedChildCollector<'a> {
        helper_name: &'a str,
        directly_attached: bool,
        attached_bindings: Vec<String>,
    }

    impl<'ast> syn::visit::Visit<'ast> for ReturnedChildCollector<'_> {
        fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
            if expression.method == "child" {
                for argument in &expression.args {
                    if direct_helper_call(argument, self.helper_name) {
                        self.directly_attached = true;
                    }
                    if let Some(binding) = direct_binding(argument) {
                        self.attached_bindings.push(binding);
                    }
                }
            }
            syn::visit::visit_expr_method_call(self, expression);
        }
    }

    let mut collector = ReturnedChildCollector {
        helper_name,
        directly_attached: false,
        attached_bindings: Vec::new(),
    };
    syn::visit::Visit::visit_expr(&mut collector, returned);
    PrivateCompositionAttachment {
        directly_attached: collector.directly_attached,
        bound_outputs,
        attached_bindings: collector.attached_bindings,
    }
}

#[test]
fn public_labels_attach_the_color_checked_private_fragments() {
    let recipes = include_str!("../src/style/recipes.rs");
    for (public, private, output_count) in [
        ("required_label", "required_label_fragments", 2),
        ("popover_row", "popover_row_label", 1),
    ] {
        let attachment = private_composition_attachment(recipes, public, private);
        let all_bound_outputs_attached = attachment.bound_outputs.len() == output_count
            && attachment.bound_outputs.iter().all(|binding| {
                attachment
                    .attached_bindings
                    .iter()
                    .any(|attached| attached == binding)
            });
        assert!(
            attachment.directly_attached || all_bound_outputs_attached,
            "{public} must attach every output of color-checked helper {private}; bound={:?}, attached={:?}",
            attachment.bound_outputs,
            attachment.attached_bindings,
        );
    }
}

struct PopoverValueAttachmentProbe {
    value_prepainted: Rc<Cell<bool>>,
}

impl Render for PopoverValueAttachmentProbe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let value_prepainted = self.value_prepainted.clone();
        let value = div().w(px(17.)).on_prepaint(move |_, _, _| {
            value_prepainted.set(true);
        });
        recipes::popover_row(POPOVER_LABEL, value, cx)
    }
}

fn assert_public_recipe_children(
    cx: &mut TestAppContext,
    kind: CompositionProbeKind,
    expected_count: usize,
) {
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
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();

    let actual = actual.borrow();
    let expected = expected.borrow();
    // This public GPUI hook proves immediate child presence, order, and
    // geometry without exposing production-private child storage. Exact text
    // and font sizes are asserted separately through `RecordingTextSystem`.
    assert_eq!(actual.len(), expected_count, "actual child count");
    assert_eq!(actual.len(), expected.len(), "reference child count");
    for (actual, expected) in actual.iter().zip(expected.iter()) {
        assert_eq!(actual.size, expected.size, "ordered child bounds");
    }
}

#[test]
fn neutral_token_scale_matches_the_accepted_contract() {
    assert_eq!(tokens::TEXT_10, rems(0.625));
    assert_eq!(tokens::TEXT_12, rems(0.75));
    assert_eq!(tokens::TEXT_13, rems(0.8125));
    assert_eq!(tokens::TEXT_14, rems(0.875));
    assert_eq!(tokens::TEXT_15, rems(0.9375));
    assert_eq!(tokens::TEXT_16, rems(1.0));
    assert_eq!(tokens::SPACE_HALF, px(2.));
    assert_eq!(tokens::SPACE_1, px(4.));
    assert_eq!(tokens::SPACE_1P5, px(6.));
    assert_eq!(tokens::SPACE_2, px(8.));
    assert_eq!(tokens::SPACE_3, px(12.));
    assert_eq!(tokens::SPACE_4, px(16.));
    assert_eq!(tokens::SPACE_6, px(24.));
    assert_eq!(tokens::ACCENT_RIM_PX, px(2.));
    assert_eq!(tokens::RADIUS_SM, px(4.));
    assert_eq!(tokens::RADIUS_MD, px(6.));
    assert_eq!(tokens::RADIUS_LG, px(8.));
    assert_eq!(tokens::RADIUS_PILL, px(100.));
    assert_eq!(tokens::ICON_INLINE, px(12.));
    assert_eq!(tokens::ICON_CHROME, px(14.));
    assert_eq!(tokens::ICON_PRIMARY, px(16.));
}

#[gpui::test]
fn theme_relative_surface_tones_preserve_the_accepted_math(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    cx.update(|cx| {
        let background = hsla(72. / 360., 0.4, 0.6, 0.8);
        let accent = hsla(288. / 360., 0.6, 0.2, 1.0);
        {
            let theme = Theme::global_mut(cx);
            theme.background = background;
            theme.accent = accent;
        }

        let expected_mix = |amount: f32| Hsla {
            h: background.h * (1.0 - amount) + accent.h * amount,
            s: background.s * (1.0 - amount) + accent.s * amount,
            l: background.l * (1.0 - amount) + accent.l * amount,
            a: background.a * (1.0 - amount) + accent.a * amount,
        };
        assert_eq!(
            tokens::bg_sunken(cx),
            Hsla {
                l: (background.l - 0.04).max(0.0),
                ..background
            },
        );
        assert_eq!(tokens::bg_active(cx), expected_mix(0.10));
        assert_eq!(tokens::accent_soft(cx), expected_mix(0.08));
    });
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
        .text_xs();
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
fn public_recipes_render_their_text_and_truncation_children(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    for &kind in COMPOSITION_PROBES {
        let expected_count = match kind {
            CompositionProbeKind::RequiredLabel | CompositionProbeKind::PopoverRow => 2,
            _ => 1,
        };
        assert_public_recipe_children(cx, kind, expected_count);
    }
}

#[gpui::test]
fn popover_row_prepaints_the_caller_supplied_value(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    let value_prepainted = Rc::new(Cell::new(false));
    let window = cx.open_window(gpui::size(px(320.), px(160.)), {
        let value_prepainted = value_prepainted.clone();
        move |_, _| PopoverValueAttachmentProbe { value_prepainted }
    });
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .expect("popover value window draws");

    assert!(
        value_prepainted.get(),
        "the exact caller-owned value must participate in popover-row prepaint",
    );
}

#[test]
fn public_recipes_layout_exact_text_at_their_role_font_sizes() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let text_system: Arc<dyn PlatformTextSystem> =
        Arc::new(RecordingTextSystem::new(calls.clone()));
    let mut cx = HeadlessAppContext::new(text_system);
    cx.update(gpui_neath::init);

    for &kind in COMPOSITION_PROBES {
        let window = cx
            .open_window(gpui::size(px(320.), px(160.)), move |_, cx| {
                cx.new(|_| TextProbe { kind })
            })
            .expect("headless text window opens");
        cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
            .expect("headless text window draws");
    }

    let calls = calls.lock().expect("layout recorder lock").clone();
    let expected = expected_layouts();
    assert!(
        !calls.is_empty(),
        "each public helper must lay out its caller-supplied text"
    );
    for layout in &expected {
        assert!(
            calls.contains(layout),
            "missing expected text layout: {layout:?}; observed: {calls:?}",
        );
    }
    for layout in &calls {
        assert!(
            expected.contains(layout),
            "unexpected text layout: {layout:?}; expected only: {expected:?}",
        );
    }
}

#[test]
fn public_truncating_cells_rewrite_visible_text_in_their_declared_direction() {
    let end = draw_narrow_truncation(CompositionProbeKind::TruncatingCell);
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

    let sized_end = draw_narrow_truncation(CompositionProbeKind::TruncatingCellSized);
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

    let middle = draw_narrow_truncation(CompositionProbeKind::MiddleTruncatingCell);
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

#[gpui::test]
fn tool_popover_preserves_styled_elevation_and_dense_geometry(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    cx.update(|cx| {
        let representative = hsla(210. / 360., 0.25, 0.31, 1.);
        let foreground = hsla(0., 0., 0.96, 1.);
        let structural_border = hsla(20. / 360., 0.20, 0.20, 1.);
        let soft_hairline = hsla(80. / 360., 0.30, 0.45, 0.4);
        let strong_hairline = hsla(190. / 360., 0.50, 0.70, 0.65);
        let background = linear_gradient(
            37.,
            linear_color_stop(hsla(220. / 360., 0.30, 0.22, 1.), 0.),
            linear_color_stop(hsla(185. / 360., 0.25, 0.38, 1.), 1.),
        );
        {
            let theme = Theme::global_mut(cx);
            theme.popover = representative;
            theme.tokens.popover = ThemeToken::new(representative, background);
            theme.popover_foreground = foreground;
            theme.border = structural_border;
            theme.hairline = soft_hairline;
            theme.hairline_strong = strong_hairline;
            theme.radius = px(7.);
        }

        let shadow = vec![
            BoxShadow {
                color: hsla(0., 0., 0., 0.10),
                offset: point(px(0.), px(4.)),
                blur_radius: px(6.),
                spread_radius: px(-1.),
                inset: false,
            },
            BoxShadow {
                color: hsla(0., 0., 0., 0.10),
                offset: point(px(0.), px(2.)),
                blur_radius: px(4.),
                spread_radius: px(-2.),
                inset: false,
            },
        ];
        let mut actual = recipes::tool_popover(cx);
        let mut expected = v_flex()
            .bg(background)
            .text_color(foreground)
            .border_1()
            .border_color(strong_hairline)
            .shadow(shadow)
            .rounded(px(7.))
            .p(tokens::SPACE_3)
            .gap(tokens::SPACE_1)
            .text_size(tokens::TEXT_12);
        assert_eq!(actual.style().clone(), expected.style().clone());
        assert_ne!(actual.style().border_color, Some(structural_border));
        assert_ne!(actual.style().border_color, Some(soft_hairline));
    });
}

#[gpui::test]
fn surface_and_separator_roles_remain_distinct(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    cx.update(|cx| {
        let muted = hsla(10. / 360., 0.2, 0.4, 0.8);
        let hairline = hsla(20. / 360., 0.3, 0.5, 0.6);
        let border = hsla(30. / 360., 0.4, 0.6, 1.0);
        {
            let theme = Theme::global_mut(cx);
            theme.muted = muted;
            theme.hairline = hairline;
            theme.border = border;
        }
        assert_eq!(recipes::on_surface_fill(cx), muted.opacity(0.5));
        assert_eq!(recipes::on_surface_border(cx), hairline);

        let mut actual = recipes::popover_rule(cx);
        let mut expected = div()
            .w_full()
            .flex_none()
            .h(px(1.))
            .my(tokens::SPACE_1P5)
            .bg(border);
        assert_eq!(actual.style().clone(), expected.style().clone());
    });
}

#[gpui::test]
fn typography_and_control_roles_preserve_the_accepted_styles(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    cx.update(|cx| {
        let foreground = hsla(12. / 360., 0.4, 0.7, 1.);
        let muted = hsla(190. / 360., 0.2, 0.45, 1.);
        let danger = hsla(350. / 360., 0.7, 0.5, 1.);
        {
            let theme = Theme::global_mut(cx);
            theme.foreground = foreground;
            theme.muted_foreground = muted;
            theme.danger = danger;
        }

        let mut body = recipes::body("body", cx);
        let mut expected_body = div().text_size(tokens::TEXT_12).text_color(foreground);
        assert_eq!(body.style().clone(), expected_body.style().clone());

        let mut body_muted = recipes::body_muted("body-muted", cx);
        let mut expected_body_muted = div().text_size(tokens::TEXT_12).text_color(muted);
        assert_eq!(
            body_muted.style().clone(),
            expected_body_muted.style().clone(),
        );

        let mut dialog_prose = recipes::dialog_prose("dialog", cx);
        let mut expected_dialog = div().text_size(tokens::TEXT_14).text_color(muted);
        assert_eq!(
            dialog_prose.style().clone(),
            expected_dialog.style().clone()
        );

        let mut value = recipes::value("value", cx);
        assert_eq!(value.style().clone(), expected_body.style().clone());

        let mut value_dense = recipes::value_dense("dense", cx);
        let mut expected_dense = div()
            .text_size(tokens::TEXT_10)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(foreground);
        assert_eq!(value_dense.style().clone(), expected_dense.style().clone());

        let mut caption = recipes::caption("caption", cx);
        let mut expected_caption = div().text_size(tokens::TEXT_10).text_color(muted);
        assert_eq!(caption.style().clone(), expected_caption.style().clone());

        let mut control_label = recipes::control_label("control", cx);
        let mut expected_control_label = div()
            .text_size(tokens::TEXT_14)
            .line_height(relative(1.))
            .text_color(foreground);
        assert_eq!(
            control_label.style().clone(),
            expected_control_label.style().clone(),
        );

        let mut control = recipes::control_body(Button::new("actual-control"));
        let mut expected_control = Button::new("expected-control")
            .small()
            .text_size(tokens::TEXT_12);
        assert_eq!(control.style().clone(), expected_control.style().clone());

        let mut section = recipes::section_label("Section", cx);
        let mut expected_section = div()
            .text_size(tokens::TEXT_13)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(muted);
        assert_eq!(section.style().clone(), expected_section.style().clone());

        let mut required = recipes::required_label("Required", cx);
        let mut expected_required = h_flex().gap(px(4.));
        assert_eq!(required.style().clone(), expected_required.style().clone());
    });
}

#[gpui::test]
fn popover_row_preserves_outer_geometry(cx: &mut TestAppContext) {
    cx.update(gpui_neath::init);
    cx.update(|cx| {
        let mut actual = recipes::popover_row("Label", div(), cx);
        let mut expected = h_flex()
            .w_full()
            .h(px(24.))
            .flex_none()
            .items_center()
            .justify_between()
            .gap(tokens::SPACE_2);
        assert_eq!(actual.style().clone(), expected.style().clone());
    });
}
