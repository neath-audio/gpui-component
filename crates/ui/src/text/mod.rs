mod document;
mod format;
mod inline;
mod inline_flow;
mod markdown_ext;
mod node;
pub(crate) mod selection;
mod selection_adapter;
mod state;
mod style;
mod text_view;
mod utils;
#[cfg(test)]
mod window_selection;

use gpui::{App, ElementId, IntoElement, RenderOnce, SharedString, Window};
pub use markdown_ext::*;
pub use node::TableData;
pub use state::*;
pub use style::*;
pub use text_view::*;

use crate::global_state::UiGlobalState;

pub(crate) fn init(cx: &mut App) {
    state::init(cx);
}

/// Register an app-global interceptor for TextView link clicks. The handler
/// returns `true` when it consumed the link; any other link falls through to
/// `cx.open_url`. Registering again replaces the previous handler.
pub fn set_link_handler(
    cx: &mut App,
    handler: impl Fn(&str, &mut Window, &mut App) -> bool + 'static,
) {
    // Use the crate's existing mutable-global accessor (same one other
    // GlobalState writers in this crate use).
    UiGlobalState::global_mut(cx).text_link_handler = Some(std::rc::Rc::new(handler));
}

/// True when the registered handler consumed the link. Cloning the Rc first
/// releases the global borrow before the handler runs (the handler will
/// re-enter `cx`).
pub(crate) fn link_handled(url: &str, window: &mut Window, cx: &mut App) -> bool {
    let handler = UiGlobalState::global(cx).text_link_handler.clone();
    handler.is_some_and(|h| h(url, window, cx))
}

/// The one link-opening path every TextView click site routes through.
pub(crate) fn open_text_link(url: &str, window: &mut Window, cx: &mut App) {
    if link_handled(url, window, cx) {
        return;
    }
    cx.open_url(url);
}

/// Create a new markdown text view with code location as id.
#[track_caller]
pub fn markdown(source: impl Into<SharedString>) -> TextView {
    let id: ElementId = ElementId::CodeLocation(*std::panic::Location::caller());
    TextView::markdown(id, source)
}

/// Create a new html text view with code location as id.
#[track_caller]
pub fn html(source: impl Into<SharedString>) -> TextView {
    let id: ElementId = ElementId::CodeLocation(*std::panic::Location::caller());
    TextView::html(id, source)
}

#[derive(IntoElement, Clone)]
pub enum Text {
    String(SharedString),
    TextView(Box<TextView>),
}

impl From<SharedString> for Text {
    fn from(s: SharedString) -> Self {
        Self::String(s)
    }
}

impl From<&str> for Text {
    fn from(s: &str) -> Self {
        Self::String(SharedString::from(s.to_string()))
    }
}

impl From<String> for Text {
    fn from(s: String) -> Self {
        Self::String(s.into())
    }
}

impl From<TextView> for Text {
    fn from(e: TextView) -> Self {
        Self::TextView(Box::new(e))
    }
}

impl Text {
    /// Set the style for [`TextView`].
    ///
    /// Do nothing if this is `String`.
    pub fn style(self, style: TextViewStyle) -> Self {
        match self {
            Self::String(s) => Self::String(s),
            Self::TextView(e) => Self::TextView(Box::new(e.style(style))),
        }
    }

    /// Get the text content.
    pub(crate) fn get_text(&self, cx: &App) -> SharedString {
        match self {
            Self::String(s) => s.clone(),
            Self::TextView(view) => {
                if let Some(state) = &view.state {
                    state.read(cx).source()
                } else {
                    SharedString::default()
                }
            }
        }
    }
}

impl RenderOnce for Text {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        match self {
            Self::String(s) => s.into_any_element(),
            Self::TextView(e) => e.into_any_element(),
        }
    }
}

#[cfg(test)]
mod link_handler_tests {
    use super::*;
    use gpui::TestAppContext;

    #[gpui::test]
    async fn handler_consumes_matching_links_and_passes_others(cx: &mut TestAppContext) {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::<String>::new()));
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|_, _| gpui::Empty);
        cx.update(|window, cx| {
            let seen2 = seen.clone();
            set_link_handler(cx, move |url, _window, _cx| {
                seen2.borrow_mut().push(url.to_string());
                url.starts_with("neath:")
            });
            assert!(link_handled("neath:search?q=x", window, cx));
            assert!(!link_handled("https://example.com", window, cx));
        });
        assert_eq!(
            *seen.borrow(),
            vec!["neath:search?q=x", "https://example.com"]
        );
    }

    #[gpui::test]
    async fn no_handler_means_nothing_is_handled(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|_, _| gpui::Empty);
        cx.update(|window, cx| {
            assert!(!link_handled("neath:search?q=x", window, cx));
        });
    }
}
