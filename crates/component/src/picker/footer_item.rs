use std::rc::Rc;

use gpui::{
    App, ClickEvent, ElementId, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled as _, Window,
    prelude::FluentBuilder as _,
};

use crate::{ActiveTheme as _, Icon, Sizable as _, Size, StyleSized as _, h_flex};

type Callback = Rc<dyn Fn(&mut Window, &mut App)>;

/// A command row below the picker's separator, with the same metrics as a
/// [`PickerItem`](super::PickerItem) plus a leading icon. The menu text tier
/// matches the surface: Medium for a project switcher, Small for recents. The
/// icon is 12px at every tier.
#[derive(IntoElement)]
pub struct PickerFooterItem {
    id: ElementId,
    label: SharedString,
    icon: Option<Icon>,
    menu_size: Size,
    before_click: Option<Callback>,
    on_click: Option<Callback>,
}

impl PickerFooterItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            menu_size: Size::Medium,
            before_click: None,
            on_click: None,
        }
    }

    /// Leading icon, drawn 12px in the foreground color.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Menu text tier, resolved when rendered. Default `Medium`.
    pub fn menu_text_size(mut self, size: Size) -> Self {
        self.menu_size = size;
        self
    }

    pub fn on_click(mut self, on_click: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(on_click));
        self
    }

    /// Runs before [`on_click`](Self::on_click); the hosting picker uses it to
    /// close itself first.
    pub(crate) fn before_click(mut self, before: Rc<dyn Fn(&mut Window, &mut App)>) -> Self {
        self.before_click = Some(before);
        self
    }
}

impl RenderOnce for PickerFooterItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let before = self.before_click;
        let action = self.on_click;
        let foreground = cx.theme().foreground;
        let id = self.id.clone();
        h_flex()
            .id(self.id)
            .debug_selector(move || format!("picker-footer-{id}"))
            // The footer sits OUTSIDE the List, so it carries the 4px side inset
            // the List's `px_1` gives the rows; paddings are the rows' own
            // (`StyleSized` at Small), so the icon column aligns with the rows.
            .mx_1()
            .list_px(Size::Small)
            .list_py(Size::Small)
            .gap_1()
            .items_center()
            .rounded(cx.theme().radius)
            .text_size(self.menu_size.menu_text_size(cx))
            .text_color(foreground)
            .hover(|this| this.bg(cx.theme().tokens.list_hover))
            .on_click(move |_: &ClickEvent, window, cx| {
                if let Some(before) = &before {
                    before(window, cx);
                }
                if let Some(action) = &action {
                    action(window, cx);
                }
            })
            .when_some(self.icon, |this, icon| {
                this.child(
                    gpui::div()
                        .when(cfg!(test), |this| {
                            this.debug_selector(|| "picker-footer-icon".into())
                        })
                        .child(icon.xsmall().text_color(foreground)),
                )
            })
            .child(self.label)
    }
}
