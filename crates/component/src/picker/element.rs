use std::rc::Rc;

use gpui::{
    Anchor, AnyElement, App, ElementId, Entity, Focusable as _, IntoElement, ParentElement as _,
    Pixels, Rems, RenderOnce, SharedString, StyleRefinement, Styled, Window,
    prelude::FluentBuilder as _, px,
};

use super::{PickerFooterItem, PickerState};
use crate::{
    Selectable, StyledExt as _,
    list::{List, ListDelegate},
    popover::Popover,
    separator::Separator,
};

type ApplyTrigger = Box<dyn FnOnce(Popover) -> Popover>;

/// A menu-like dropdown built on [`List`]: a trigger, an optional header, the
/// list and an optional footer of commands, in a popover that opens with the
/// Select/Combobox motion.
///
/// The surface has no side padding (the List carries `px_1`, footer items
/// `mx_1`), 4px at the bottom, and 4px at the top unless the picker is
/// searchable or has a header, whose own band owns the top edge. Style
/// methods (`w`, `max_w`) refine that surface.
#[derive(IntoElement)]
pub struct Picker<D: ListDelegate> {
    id: ElementId,
    state: Entity<PickerState<D>>,
    style: StyleRefinement,
    trigger: Option<ApplyTrigger>,
    anchor: Anchor,
    header: Option<AnyElement>,
    list_max_h: Pixels,
    list_visible: bool,
    search_placeholder: Option<SharedString>,
    search_text_size: Option<Rems>,
    footer: Vec<PickerFooterItem>,
}

impl<D: ListDelegate> Picker<D> {
    pub fn new(id: impl Into<ElementId>, state: &Entity<PickerState<D>>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            style: StyleRefinement::default(),
            trigger: None,
            anchor: Anchor::TopLeft,
            header: None,
            list_max_h: px(280.),
            list_visible: true,
            search_placeholder: None,
            search_text_size: None,
            footer: Vec::new(),
        }
    }

    /// The element that opens the picker, as [`Popover::trigger`].
    pub fn trigger<T: Selectable + IntoElement + 'static>(mut self, trigger: T) -> Self {
        self.trigger = Some(Box::new(move |popover: Popover| popover.trigger(trigger)));
        self
    }

    /// The popover's anchor, which also picks the slide direction. Default
    /// [`Anchor::TopLeft`].
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Drawn above the list, flush with the surface's top edge.
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    /// The list's maximum height. Default 280px.
    pub fn list_max_h(mut self, max_h: Pixels) -> Self {
        self.list_max_h = max_h;
        self
    }

    /// Whether the List is drawn. Default `true`. A visible List with no rows
    /// shows the delegate's `render_empty`; the owner decides between that and
    /// leaving the List out.
    pub fn list_visible(mut self, visible: bool) -> Self {
        self.list_visible = visible;
        self
    }

    /// The query field's placeholder, when the state is searchable.
    pub fn search_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.search_placeholder = Some(placeholder.into());
        self
    }

    /// The query field's text size, when the state is searchable, as
    /// [`List::search_text_size`].
    pub fn search_text_size(mut self, size: Rems) -> Self {
        self.search_text_size = Some(size);
        self
    }

    /// Commands below the list, after one separator. A footer item's click
    /// closes the picker before running its action.
    pub fn footer(mut self, items: impl IntoIterator<Item = PickerFooterItem>) -> Self {
        self.footer.extend(items);
        self
    }
}

impl<D: ListDelegate> Styled for Picker<D> {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl<D: ListDelegate> RenderOnce for Picker<D> {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.state.read(cx);
        let open = state.is_open();
        let flush_top = state.is_searchable() || self.header.is_some();
        let focus_handle = state.focus_handle(cx);
        let list = state.list().clone();

        let on_open_change = self.state.downgrade();
        let on_footer_click = self.state.downgrade();

        let popover = Popover::new(self.id)
            .anchor(self.anchor)
            .track_focus(&focus_handle)
            .open(open)
            .on_open_change(move |open, window, cx| {
                _ = on_open_change.update(cx, |state, cx| state.set_open(*open, window, cx));
            })
            .dropdown_motion()
            // The frame overrides the surface's default `p_3`; the caller's
            // style (`w`, `max_w`) refines after it.
            .px_0()
            .pb_1()
            .map(|this| if flush_top { this.pt_0() } else { this.pt_1() })
            .refine_style(&self.style);

        let popover = match self.trigger {
            Some(apply) => apply(popover),
            None => popover,
        };

        popover
            .children(self.header)
            .when(self.list_visible, |this| {
                this.child(
                    List::new(&list)
                        .max_h(self.list_max_h)
                        .px_1()
                        .when_some(self.search_placeholder, |list, placeholder| {
                            list.search_placeholder(placeholder)
                        })
                        .when_some(self.search_text_size, |list, size| {
                            list.search_text_size(size)
                        }),
                )
            })
            .when(!self.footer.is_empty(), |this| {
                // A footer command closes the picker before it runs.
                let close: Rc<dyn Fn(&mut Window, &mut App)> = Rc::new(move |_, cx| {
                    _ = on_footer_click.update(cx, |state, cx| state.close(cx));
                });
                this.child(Separator::horizontal().my_1()).children(
                    self.footer
                        .into_iter()
                        .map(|item| item.before_click(close.clone())),
                )
            })
    }
}
