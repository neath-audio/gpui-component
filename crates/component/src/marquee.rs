//! One line of text that scrolls back and forth when it is too long for its box.

use std::time::Duration;

use gpui::{
    App, Bounds, ContentMask, Element, ElementId, Global, GlobalElementId, InspectorElementId,
    InteractiveElement as _, IntoElement, LayoutId, ParentElement as _, Pixels, Point, RenderOnce,
    Role, ShapedLine, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled,
    TextAlign, TruncateFrom, Window, div, point, prelude::FluentBuilder as _, px, relative,
};

use crate::{Sizable, Size, StyledExt as _};

/// How fast text that doesn't fit moves, in logical pixels a second.
const SPEED: f32 = 30.;
/// How long text that doesn't fit rests at each end, in seconds.
const PAUSE: f32 = 1.5;
/// How far, in logical pixels, a line may run past its box and still count as fitting. A
/// shaped width can land a fraction past the width it was measured for.
const SLACK: f32 = 0.5;
/// What ends a line cut short, as GPUI's own truncation ends it.
const ELLIPSIS: &str = "…";

/// How far along text that runs `overflow` logical pixels past its box has moved after
/// `elapsed_secs` seconds, in logical pixels from its start. Text that fits, to within half a
/// pixel, stays at 0. Text that doesn't rests 1.5 s at its start, moves at 30 px a second until
/// its end meets the end of the box, rests 1.5 s there, and moves back, over and over.
pub fn marquee_offset(overflow: f32, elapsed_secs: f32) -> f32 {
    if overflow <= SLACK || !overflow.is_finite() || !elapsed_secs.is_finite() {
        return 0.;
    }
    let travel = overflow / SPEED;
    let t = elapsed_secs.rem_euclid(cycle_secs(overflow));
    if t < PAUSE {
        0.
    } else if t < PAUSE + travel {
        (t - PAUSE) * SPEED
    } else if t < PAUSE * 2. + travel {
        overflow
    } else {
        (overflow - (t - PAUSE * 2. - travel) * SPEED).max(0.)
    }
}

/// How long one trip out and back takes for text that runs `overflow` logical pixels past its
/// box, in seconds, with both rests.
fn cycle_secs(overflow: f32) -> f32 {
    (PAUSE + overflow / SPEED) * 2.
}

/// The one clock every marquee keeps time by, started by the first one to paint. A marquee that
/// is drawn again keeps its place, and marquees that run equally far past their boxes move
/// together.
struct MarqueeClock(Box<dyn Fn(&App) -> Duration>);

impl Global for MarqueeClock {}

fn since_clock_start(cx: &mut App) -> Duration {
    if !cx.has_global::<MarqueeClock>() {
        let start = cx.background_executor().now();
        cx.set_global(MarqueeClock(Box::new(move |cx| {
            cx.background_executor()
                .now()
                .saturating_duration_since(start)
        })));
    }
    (cx.global::<MarqueeClock>().0)(cx)
}

/// One line of text that scrolls back and forth inside its box when it is too long for it, such
/// as a file's name in a header that can't grow to fit the name. Text that fits holds still
/// where a label would put it.
///
/// Text that doesn't fit rests 1.5 s at its start, moves at 30 px a second until its end shows,
/// rests 1.5 s, and moves back, inside the box and clipped to it. A marquee asks for frames only
/// while it is moving and on screen. Under reduced motion, or when [`animated`](Self::animated)
/// is off, the text holds still at its start and ends with an ellipsis where the box cuts it.
///
/// Assistive technology reads the whole text, however much of it is in view.
///
/// A marquee fills the width its parent gives it and is one line tall. Line breaks in the text
/// read as spaces. It takes its font, color, weight, size and alignment from the surrounding
/// text style unless [`Styled`] or [`Sizable`] set them.
#[derive(IntoElement)]
pub struct Marquee {
    id: ElementId,
    text: SharedString,
    size: Option<Size>,
    animated: bool,
    style: StyleRefinement,
}

impl Marquee {
    /// Create a marquee that takes its text style from its parent.
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            size: None,
            animated: true,
            style: StyleRefinement::default(),
        }
    }

    /// Whether text that doesn't fit moves. Off, it holds still at its start and ends with an
    /// ellipsis where the box cuts it, as it does under reduced motion. On unless set.
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }
}

impl Sizable for Marquee {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = Some(size.into());
        self
    }
}

impl Styled for Marquee {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Marquee {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let line = MarqueeLine {
            text: one_line(&self.text),
            moving: self.animated && !cx.reduce_motion(),
        };
        div()
            .id(self.id)
            .role(Role::Label)
            .aria_label(self.text)
            .w_full()
            .min_w_0()
            .when_some(self.size, |this, size| {
                this.text_size(size.control_text_size(cx))
            })
            .refine_style(&self.style)
            .child(line)
    }
}

/// `text` with its line breaks read as spaces, since a marquee is one line.
fn one_line(text: &SharedString) -> SharedString {
    if text.contains(['\n', '\r']) {
        text.replace(['\n', '\r'], " ").into()
    } else {
        text.clone()
    }
}

/// The line a marquee draws, in the text style its parent sets.
struct MarqueeLine {
    text: SharedString,
    /// Whether text that doesn't fit moves, or holds still cut short.
    moving: bool,
}

/// A line shaped for the width it was laid out at.
struct Shaped {
    line: ShapedLine,
    line_height: Pixels,
    /// How far the whole text runs past the box, in logical pixels, when it moves.
    overflow: Option<f32>,
    align: TextAlign,
}

impl IntoElement for MarqueeLine {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl Element for MarqueeLine {
    type RequestLayoutState = ();
    type PrepaintState = Shaped;

    fn id(&self) -> Option<ElementId> {
        None
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
    ) -> (LayoutId, ()) {
        let mut style = gpui::Style::default();
        style.size.width = relative(1.).into();
        style.size.height = line_height(window).into();
        style.min_size.width = px(0.).into();
        (window.request_layout(style, None, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Shaped {
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = line_height(window);
        let runs = [style.to_run(self.text.len())];
        let line = window
            .text_system()
            .shape_line(self.text.clone(), font_size, &runs, None);
        let overflow = f32::from(line.width() - bounds.size.width);
        if overflow <= SLACK {
            return Shaped {
                line,
                line_height,
                overflow: None,
                align: style.text_align,
            };
        }
        if self.moving {
            return Shaped {
                line,
                line_height,
                overflow: Some(overflow),
                align: TextAlign::Left,
            };
        }
        let (text, runs) = cx
            .text_system()
            .line_wrapper(style.font(), font_size)
            .truncate_line(
                self.text.clone(),
                bounds.size.width,
                ELLIPSIS,
                &runs,
                TruncateFrom::End,
            );
        Shaped {
            line: window
                .text_system()
                .shape_line(text, font_size, &runs, None),
            line_height,
            overflow: None,
            align: style.text_align,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        shaped: &mut Shaped,
        window: &mut Window,
        cx: &mut App,
    ) {
        let x = match shaped.overflow {
            Some(overflow) => {
                let elapsed = since_clock_start(cx);
                // Take the cycle out in f64 first: an f32 count of seconds since the clock
                // started loses the precision smooth motion needs within days.
                let t = elapsed.as_secs_f64() % f64::from(cycle_secs(overflow));
                bounds.left() - px(marquee_offset(overflow, t as f32))
            }
            None => {
                let room = bounds.size.width - shaped.line.width();
                bounds.left()
                    + match shaped.align {
                        TextAlign::Left => px(0.),
                        TextAlign::Center => room / 2.,
                        TextAlign::Right => room,
                    }
            }
        };
        let origin: Point<Pixels> = point(x, bounds.top());
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            #[cfg(test)]
            painted::note(&self.text, &shaped.line, origin, window);
            if let Err(error) = shaped.line.paint(
                origin,
                shaped.line_height,
                TextAlign::Left,
                None,
                window,
                cx,
            ) {
                eprintln!("Unable to paint a marquee: {error}");
            }
        });
        if shaped.overflow.is_some() && is_visible(bounds, window) {
            window.request_animation_frame();
        }
    }
}

/// Whether any of `bounds` shows, so that a marquee scrolled out of view asks for no frames.
fn is_visible(bounds: Bounds<Pixels>, window: &Window) -> bool {
    let visible = bounds
        .intersect(&window.content_mask().bounds)
        .intersect(&Bounds::new(Point::default(), window.viewport_size()));
    visible.size.width > px(0.) && visible.size.height > px(0.)
}

/// The line the window's text style sets, rounded to the device pixel as GPUI rounds a label's.
/// `Window::line_height` rounds it to a whole logical pixel instead, which a label's text doesn't.
fn line_height(window: &Window) -> Pixels {
    let style = window.text_style();
    window.pixel_snap(
        style
            .line_height
            .to_pixels(style.font_size, window.rem_size()),
    )
}

/// Where each marquee painted its line, for tests. The test text system draws no glyphs, so a
/// test window can't report them; this notes what the line asked of them instead.
#[cfg(test)]
mod painted {
    use gpui::{Bounds, FontWeight, Hsla, Pixels, Point, ShapedLine, SharedString, Window};
    use std::cell::RefCell;

    /// One line a marquee painted.
    #[derive(Clone, Debug)]
    pub(super) struct Line {
        /// The marquee's whole text.
        source: SharedString,
        /// The text the line painted: the whole text, or its start and an ellipsis.
        pub(super) text: SharedString,
        /// Where the line's first glyph starts, and the top of the line.
        pub(super) origin: Point<Pixels>,
        pub(super) width: Pixels,
        /// What the glyphs are clipped to.
        pub(super) mask: Bounds<Pixels>,
        pub(super) font_size: Pixels,
        pub(super) line_height: Pixels,
        pub(super) color: Hsla,
        pub(super) weight: FontWeight,
    }

    thread_local! {
        static LINES: RefCell<Vec<Line>> = const { RefCell::new(Vec::new()) };
    }

    /// Notes the line a marquee of `source` paints at `origin`, in the window's text style
    /// and inside its content mask.
    pub(super) fn note(
        source: &SharedString,
        line: &ShapedLine,
        origin: Point<Pixels>,
        window: &Window,
    ) {
        let style = window.text_style();
        LINES.with_borrow_mut(|lines| {
            lines.push(Line {
                source: source.clone(),
                text: line.text.clone(),
                origin,
                width: line.width(),
                mask: window.content_mask().bounds,
                font_size: style.font_size.to_pixels(window.rem_size()),
                line_height: super::line_height(window),
                color: style.color,
                weight: style.font_weight,
            })
        });
    }

    /// The last line a marquee of `source` painted.
    #[track_caller]
    pub(super) fn last(source: &str) -> Line {
        LINES.with_borrow(|lines| {
            lines
                .iter()
                .rev()
                .find(|line| line.source.as_ref() == source)
                .cloned()
                .unwrap_or_else(|| panic!("no marquee painted {source:?}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Marquee, marquee_offset, painted};
    use crate::{Sizable as _, Size};
    use gpui::{
        Bounds, Context, Div, FontWeight, IntoElement, Pixels, Render, TestAppContext,
        VisualTestContext, Window, div, prelude::*, px, red,
    };
    use std::time::Duration;

    /// A name that fits the box.
    const SHORT: &str = "Rain";
    /// A name far too long for the box.
    const LONG: &str = "Project brief and design principles to review.md";
    /// The box's width, in logical pixels.
    const BOX: f32 = 160.;

    /// A marquee in a box `BOX` px wide, which `style` styles.
    struct Probe {
        marquee: Box<dyn Fn() -> Marquee>,
        style: fn(Div) -> Div,
    }

    impl Probe {
        fn new(marquee: impl Fn() -> Marquee + 'static) -> Self {
            Self {
                marquee: Box::new(marquee),
                style: |frame| frame,
            }
        }

        fn styled(mut self, style: fn(Div) -> Div) -> Self {
            self.style = style;
            self
        }
    }

    impl Render for Probe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().child((self.style)(
                div()
                    .debug_selector(|| "box".into())
                    .w(px(BOX))
                    .child((self.marquee)()),
            ))
        }
    }

    fn open(cx: &mut TestAppContext, probe: Probe) -> VisualTestContext {
        cx.update(crate::init);
        let (_, visual) = cx.add_window_view(|_, _| probe);
        let mut visual = visual.clone();
        visual.simulate_resize(gpui::size(px(800.), px(600.)));
        draw(&mut visual);
        visual
    }

    fn draw(visual: &mut VisualTestContext) {
        visual.update(|window, cx| window.draw(cx).clear(cx));
    }

    fn advance(visual: &mut VisualTestContext, seconds: f32) {
        visual
            .executor()
            .advance_clock(Duration::from_secs_f32(seconds));
        draw(visual);
    }

    fn frames_asked(visual: &mut VisualTestContext) -> usize {
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            window.simulate_next_frame(cx);
            window.draw(cx).clear(cx);
            window.simulate_next_frame(cx)
        })
    }

    fn frame(visual: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
        visual
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is laid out"))
    }

    fn close(actual: Pixels, expected: f32) -> bool {
        (f32::from(actual) - expected).abs() <= 0.05
    }

    #[gpui::test]
    fn text_that_fits_stays_still_and_asks_for_no_frames(cx: &mut TestAppContext) {
        let mut visual = open(cx, Probe::new(|| Marquee::new("name", SHORT)));
        let frame = frame(&mut visual, "box");
        let first = painted::last(SHORT);
        assert_eq!(first.origin.x, frame.left(), "it starts at the box's start");

        advance(&mut visual, 1.);
        let later = painted::last(SHORT);
        assert_eq!(
            later.origin.x, first.origin.x,
            "a second later it hasn't moved"
        );
        assert_eq!(later.text.as_ref(), SHORT, "it paints its whole text");
        assert_eq!(frames_asked(&mut visual), 0);
    }

    #[test]
    fn overflow_ping_pongs_with_pauses() {
        // 60 px at 30 px a second is 2 s each way, so a cycle is 1.5 s still, 2 s out, 1.5 s
        // still at the end and 2 s back: 7 s.
        for (elapsed, expected) in [
            (0., 0.),
            (1.4, 0.),
            (2.5, 30.),
            (3.5, 60.),
            (4.9, 60.),
            (6.0, 30.),
            (7.0, 0.),
            (8.4, 0.),
        ] {
            let offset = marquee_offset(60., elapsed);
            assert!(
                (offset - expected).abs() <= 1e-3,
                "at {elapsed} s the text is {offset} px along, not {expected}"
            );
        }
        for elapsed in [0., 1., 2.5, 3.5, 6., 7., 100.] {
            assert_eq!(
                marquee_offset(0.4, elapsed),
                0.,
                "{elapsed} s: half a pixel over counts as fitting"
            );
        }
    }

    #[gpui::test]
    fn long_text_moves_inside_its_box_and_keeps_asking_for_frames(cx: &mut TestAppContext) {
        let mut visual = open(cx, Probe::new(|| Marquee::new("name", LONG)));
        let frame = frame(&mut visual, "box");
        let overflow = f32::from(painted::last(LONG).width) - BOX;
        assert!(overflow > 0., "the line is longer than the box");
        let mut elapsed = 0.;
        // Still at the start, part of the way out, and part of the way back.
        for step in [0., 2.5, 7.6] {
            if step > 0. {
                advance(&mut visual, step);
                elapsed += step;
            }
            let line = painted::last(LONG);
            let along = marquee_offset(overflow, elapsed);
            assert!(
                close(frame.left() - line.origin.x, along),
                "at {elapsed} s the line starts {:?} before the box, not {along}",
                frame.left() - line.origin.x
            );
            assert!(
                line.mask.left() >= frame.left() - px(0.5)
                    && line.mask.right() <= frame.right() + px(0.5),
                "at {elapsed} s the line is clipped to {:?}, outside {frame:?}",
                line.mask
            );
        }
        assert!(
            frames_asked(&mut visual) > 0,
            "a moving line asks for frames"
        );
    }

    #[gpui::test]
    fn reduced_motion_holds_still_with_an_ellipsis(cx: &mut TestAppContext) {
        // The platform's reduced motion, then a marquee told not to move.
        for (reduced, animated) in [(true, true), (false, false)] {
            cx.update(|cx| cx.set_reduce_motion(reduced));
            let mut visual = open(
                cx,
                Probe::new(move || Marquee::new("name", LONG).animated(animated)),
            );
            let frame = frame(&mut visual, "box");
            for step in [0., 2.5, 1.5, 4.] {
                advance(&mut visual, step);
                let line = painted::last(LONG);
                assert_eq!(
                    line.origin.x,
                    frame.left(),
                    "reduced {reduced}: it holds still"
                );
                assert!(line.text.ends_with('…'), "{:?} ends cut short", line.text);
                assert!(
                    LONG.starts_with(line.text.trim_end_matches('…')),
                    "{:?} is the start of the name",
                    line.text
                );
                assert!(line.width <= frame.size.width);
            }
            assert_eq!(frames_asked(&mut visual), 0);
        }
    }

    #[gpui::test]
    fn it_takes_its_text_style_from_its_parent_until_set(cx: &mut TestAppContext) {
        let mut visual = open(
            cx,
            Probe::new(|| Marquee::new("name", SHORT)).styled(|frame| {
                frame
                    .text_size(px(11.))
                    .line_height(px(17.))
                    .text_color(red())
                    .font_weight(FontWeight::MEDIUM)
            }),
        );
        let line = painted::last(SHORT);
        assert!(close(line.font_size, 11.));
        assert!(close(line.line_height, 17.));
        assert_eq!(line.color, red());
        assert_eq!(line.weight, FontWeight::MEDIUM);
        assert!(close(frame(&mut visual, "box").size.height, 17.));

        open(
            cx,
            Probe::new(|| {
                Marquee::new("sized", SHORT)
                    .with_size(Size::Size(px(20.)))
                    .text_color(red())
            }),
        );
        let line = painted::last(SHORT);
        assert!(close(line.font_size, 20.), "{:?}", line.font_size);
        assert_eq!(line.color, red());
    }

    #[gpui::test]
    fn a_line_that_fits_takes_its_parents_alignment(cx: &mut TestAppContext) {
        for (style, place) in [
            ((|frame| frame.text_center()) as fn(Div) -> Div, 0.5),
            (|frame| frame.text_right(), 1.),
        ] {
            let mut visual = open(cx, Probe::new(|| Marquee::new("name", SHORT)).styled(style));
            let frame = frame(&mut visual, "box");
            let line = painted::last(SHORT);
            let expected = f32::from(frame.left() + (frame.size.width - line.width) * place);
            assert!(
                close(line.origin.x, expected),
                "the line starts at {:?}, not {expected}",
                line.origin.x
            );
        }
    }
}
