use std::rc::Rc;

use crate::ThemeStyled as _;
use crate::{
    ActiveTheme, Colorize as _, Disableable, Icon, RoleOverride, Selectable, Sizable, Size,
    StyleSized, StyledExt, WASH_SELECTED,
    button::ButtonIcon,
    h_flex,
    select::Caret,
    tooltip::{ManagedTooltipExt as _, Tooltip},
};
use gpui::{
    AnyElement, App, Background, ClickEvent, Corners, Edges, ElementId, Hsla, InteractiveElement,
    Interactivity, IntoElement, MouseButton, ParentElement, Pixels, RenderOnce, Role, SharedString,
    StatefulInteractiveElement as _, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _, relative, transparent_white,
};

#[derive(Default, Clone, Copy)]
pub enum ButtonRounded {
    None,
    Small,
    #[default]
    Medium,
    Large,
    Size(Pixels),
}

impl From<Pixels> for ButtonRounded {
    fn from(px: Pixels) -> Self {
        ButtonRounded::Size(px)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ButtonCustomVariant {
    color: Hsla,
    foreground: Hsla,
    shadow: bool,
    hover: Hsla,
    active: Hsla,
}

pub trait ButtonVariants: Sized {
    fn with_variant(self, variant: ButtonVariant) -> Self;

    /// With the primary style for the Button.
    fn primary(self) -> Self {
        self.with_variant(ButtonVariant::Primary)
    }

    /// With the secondary style for the Button.
    fn secondary(self) -> Self {
        self.with_variant(ButtonVariant::Secondary)
    }

    /// With the danger style for the Button.
    fn danger(self) -> Self {
        self.with_variant(ButtonVariant::Danger)
    }

    /// With the warning style for the Button.
    fn warning(self) -> Self {
        self.with_variant(ButtonVariant::Warning)
    }

    /// With the success style for the Button.
    fn success(self) -> Self {
        self.with_variant(ButtonVariant::Success)
    }

    /// With the info style for the Button.
    fn info(self) -> Self {
        self.with_variant(ButtonVariant::Info)
    }

    /// With the ghost style for the Button.
    fn ghost(self) -> Self {
        self.with_variant(ButtonVariant::Ghost)
    }

    /// With the link style for the Button.
    fn link(self) -> Self {
        self.with_variant(ButtonVariant::Link)
    }

    /// With the text style for the Button, it will no padding look like a normal text.
    fn text(self) -> Self {
        self.with_variant(ButtonVariant::Text)
    }

    /// With the custom style for the Button.
    fn custom(self, style: ButtonCustomVariant) -> Self {
        self.with_variant(ButtonVariant::Custom(style))
    }
}

impl ButtonCustomVariant {
    pub fn new(cx: &App) -> Self {
        Self {
            color: cx.theme().transparent,
            foreground: cx.theme().text,
            hover: cx.theme().transparent,
            active: cx.theme().transparent,
            shadow: false,
        }
    }

    /// Set background color, default is transparent.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = color;
        self
    }

    /// Set foreground color, default is theme foreground.
    pub fn foreground(mut self, color: Hsla) -> Self {
        self.foreground = color;
        self
    }

    /// Set hover background color, default is transparent.
    pub fn hover(mut self, color: Hsla) -> Self {
        self.hover = color;
        self
    }

    /// Set active background color, default is transparent.
    pub fn active(mut self, color: Hsla) -> Self {
        self.active = color;
        self
    }

    /// Set shadow, default is false.
    pub fn shadow(mut self, shadow: bool) -> Self {
        self.shadow = shadow;
        self
    }
}

/// The variant of the Button.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ButtonVariant {
    #[default]
    Default,
    Primary,
    Secondary,
    Danger,
    Info,
    Success,
    Warning,
    Ghost,
    Link,
    Text,
    Custom(ButtonCustomVariant),
}

impl ButtonVariant {
    #[inline]
    pub fn is_link(&self) -> bool {
        matches!(self, Self::Link)
    }

    #[inline]
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text)
    }

    #[inline]
    pub fn is_ghost(&self) -> bool {
        matches!(self, Self::Ghost)
    }

    #[inline]
    fn no_padding(&self) -> bool {
        self.is_link() || self.is_text()
    }

    #[inline]
    fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }
}

/// A Button element.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    base: gpui_base::Button,
    icon: Option<ButtonIcon>,
    label: Option<SharedString>,
    children: Vec<AnyElement>,
    disabled: bool,
    pub(crate) selected: bool,
    toggled: Option<bool>,
    role: RoleOverride,
    variant: ButtonVariant,
    rounded: ButtonRounded,
    outline: bool,
    border_corners: Corners<bool>,
    border_edges: Edges<bool>,
    dropdown_caret: bool,
    size: Size,
    compact: bool,
    tooltip: Option<(
        SharedString,
        Option<(Rc<Box<dyn gpui::Action>>, Option<SharedString>)>,
    )>,
    tooltip_builder: Option<Rc<dyn Fn(&mut Window, &mut App) -> gpui::AnyView>>,
    on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
    on_hover: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
    loading: bool,
    loading_icon: Option<Icon>,
    focus_ring_enabled: bool,

    tab_index: isize,
    tab_stop: bool,
}

impl From<Button> for AnyElement {
    fn from(button: Button) -> Self {
        button.into_any_element()
    }
}

impl Button {
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();

        Self {
            id: id.clone(),
            base: gpui_base::Button::new(id),
            icon: None,
            label: None,
            children: Vec::new(),
            disabled: false,
            selected: false,
            toggled: None,
            role: RoleOverride::default(),
            variant: ButtonVariant::default(),
            rounded: ButtonRounded::Medium,
            border_corners: Corners {
                top_left: true,
                top_right: true,
                bottom_right: true,
                bottom_left: true,
            },
            border_edges: Edges::all(true),
            size: Size::Medium,
            tooltip: None,
            tooltip_builder: None,
            on_click: None,
            focus_ring_enabled: true,
            on_hover: None,
            loading: false,
            compact: false,
            outline: false,
            loading_icon: None,
            dropdown_caret: false,
            tab_index: 0,
            tab_stop: true,
        }
    }

    pub(super) fn variant(&self) -> ButtonVariant {
        self.variant
    }

    pub(super) fn button_size(&self) -> Size {
        self.size
    }

    pub(super) fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn role(mut self, role: impl Into<RoleOverride>) -> Self {
        self.role = role.into();
        self
    }

    /// Set the outline style of the Button.
    pub fn outline(mut self) -> Self {
        self.outline = true;
        self
    }

    /// Set the border radius of the Button.
    pub fn rounded(mut self, rounded: impl Into<ButtonRounded>) -> Self {
        self.rounded = rounded.into();
        self
    }

    /// Set the border corners side of the Button.
    pub(crate) fn border_corners(mut self, corners: impl Into<Corners<bool>>) -> Self {
        self.border_corners = corners.into();
        self
    }

    /// Set the border edges of the Button.
    pub(crate) fn border_edges(mut self, edges: impl Into<Edges<bool>>) -> Self {
        self.border_edges = edges.into();
        self
    }

    /// Set label to the Button, if no label is set, the button will be in Icon Button mode.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the developer-assigned identifier exposed to accessibility clients.
    pub fn accessibility_id(mut self, id: impl Into<SharedString>) -> Self {
        self.base = self.base.accessibility_id(id);
        self
    }

    /// Set the icon of the button, if the Button have no label, the button well in Icon Button mode.
    pub fn icon(mut self, icon: impl Into<ButtonIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the tooltip of the button.
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some((tooltip.into(), None));
        self
    }

    /// Set the tooltip of the button with action to show keybinding.
    pub fn tooltip_with_action(
        mut self,
        tooltip: impl Into<SharedString>,
        action: &dyn gpui::Action,
        context: Option<&str>,
    ) -> Self {
        self.tooltip = Some((
            tooltip.into(),
            Some((
                Rc::new(action.boxed_clone()),
                context.map(|c| c.to_string().into()),
            )),
        ));
        self
    }

    /// Set true to show the loading indicator.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Set the button to compact mode, then padding will be reduced.
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    /// Add click handler.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Add hover handler, the bool parameter indicates whether the mouse is hovering.
    pub fn on_hover(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_hover = Some(Rc::new(handler));
        self
    }

    /// Set the loading icon of the button, it will be used when loading is true.
    ///
    /// Default is a spinner icon.
    pub fn loading_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.loading_icon = Some(icon.into());
        self
    }

    /// Set the tab index of the button, it will be used to focus the button by tab key.
    ///
    /// Default is 0.
    pub fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = tab_index;
        self
    }

    /// Set the tab stop of the button, if true, the button will be focusable by tab key.
    ///
    /// Default is true.
    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.tab_stop = tab_stop;
        self
    }

    /// Set to show a dropdown caret icon at the end of the button.
    pub fn dropdown_caret(mut self, dropdown_caret: bool) -> Self {
        self.dropdown_caret = dropdown_caret;
        self
    }

    /// Expose this button as a toggle button to assistive technology, with
    /// `toggled` as its pressed state.
    ///
    /// Only affects accessibility metadata. Use [`Selectable::selected`] for
    /// the selected styling, and call this in addition when the button really
    /// is a toggle, otherwise the button stays an ordinary push button.
    pub fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = Some(toggled);
        self
    }

    /// Whether the button responds to the pointer at all.
    ///
    /// A loading button is as inert as a disabled one, it just keeps looking
    /// like itself instead of taking the disabled styling.
    #[inline]
    fn interactive(&self) -> bool {
        !(self.disabled || self.loading)
    }

    #[inline]
    fn hoverable(&self) -> bool {
        self.interactive() && self.on_hover.is_some()
    }
}

impl Disableable for Button {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl crate::FocusableExt for Button {
    fn focus_ring(mut self, enabled: bool) -> Self {
        self.focus_ring_enabled = enabled;
        self
    }

    fn is_focus_ring_enabled(&self) -> bool {
        self.focus_ring_enabled
    }
}

impl Selectable for Button {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl Sizable for Button {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl ButtonVariants for Button {
    fn with_variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl Styled for Button {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for Button {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

impl InteractiveElement for Button {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style: ButtonVariant = self.variant;
        let interactive = self.interactive();
        let hoverable = self.hoverable();
        let disabled = self.disabled;
        let loading = self.loading;
        let mut base = self.base;
        let children = self.children;
        let instance_style = base.style().clone();
        let normal_style = style.normal(self.outline, cx);
        let selected_style = style.selected(self.outline, cx);
        let disabled_style = style.disabled(self.outline, cx);
        let icon_size = match self.size {
            Size::Size(v) => Size::Size(v * 0.75),
            _ => self.size,
        };
        let has_content = self.icon.is_some() || self.label.is_some() || !children.is_empty();

        let focus_handle = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let is_focused = focus_handle.is_focused(window);

        let rounding = match self.rounded {
            ButtonRounded::Small => cx.theme().radius * 0.5,
            ButtonRounded::Medium => cx.theme().radius,
            ButtonRounded::Large => cx.theme().radius * 2.0,
            ButtonRounded::Size(px) => px,
            ButtonRounded::None => Pixels::ZERO,
        };

        let root = base
            .cursor_default()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .cursor_default()
            .when(
                interactive && (self.variant.is_link() || self.variant.is_text()),
                |this| this.cursor_pointer(),
            )
            .when(
                !disabled && cx.theme().shadow && normal_style.shadow,
                |this| this.shadow_xs(),
            )
            .when(!style.no_padding(), |this| {
                if self.label.is_none() && children.is_empty() {
                    // Icon Button
                    match self.size {
                        Size::Size(px) => this.size(px),
                        Size::XSmall => this.size_5(),
                        Size::Small => this.size_6(),
                        Size::Large | Size::Medium => this.size_8(),
                    }
                } else {
                    // Normal Button
                    match self.size {
                        Size::Size(size) => this.px(size * 0.2),
                        Size::XSmall => this.h_5().px_1().when(self.compact, |this| this.min_w_5()),
                        Size::Small => this
                            .h_6()
                            .px_2()
                            .when(self.compact, |this| this.min_w_6().px_1p5()),
                        Size::Medium => this
                            .h_8()
                            .px_2p5()
                            .when(self.compact, |this| this.min_w_8().px_2()),
                        Size::Large => this
                            .h_8()
                            .px_3()
                            .when(self.compact, |this| this.min_w_8().px_2()),
                    }
                }
            })
            .when(self.border_corners.top_left, |this| {
                this.rounded_tl(rounding)
            })
            .when(self.border_corners.top_right, |this| {
                this.rounded_tr(rounding)
            })
            .when(self.border_corners.bottom_left, |this| {
                this.rounded_bl(rounding)
            })
            .when(self.border_corners.bottom_right, |this| {
                this.rounded_br(rounding)
            })
            .when(self.variant.is_default() || self.outline, |this| {
                this.when(self.border_edges.left, |this| this.border_l_1())
                    .when(self.border_edges.right, |this| this.border_r_1())
                    .when(self.border_edges.top, |this| this.border_t_1())
                    .when(self.border_edges.bottom, |this| this.border_b_1())
            })
            .when(!self.disabled && !self.selected, |this| {
                this.border_color(normal_style.border)
                    .bg(normal_style.bg)
                    .text_color(normal_style.fg)
                    .when(normal_style.underline, |this| this.text_decoration_1())
                    // A loading button keeps its normal colors, but must not react
                    // to the pointer, it is not waiting for another click.
                    .when(interactive, |this| {
                        this.hover(|this| {
                            let hover_style = style.hovered(self.outline, cx);
                            this.bg(hover_style.bg)
                                .border_color(hover_style.border)
                                .text_color(hover_style.fg)
                        })
                        .active(|this| {
                            let active_style = style.active(self.outline, cx);
                            this.bg(active_style.bg)
                                .border_color(active_style.border)
                                .text_color(active_style.fg)
                        })
                    })
            })
            .refine_style(&instance_style);

        let accessibility_label = self.label.clone();
        let content = h_flex()
            .id("label")
            .size_full()
            .min_w_0()
            .overflow_hidden()
            .whitespace_nowrap()
            .items_center()
            .justify_center()
            .button_text_size(self.size)
            .map(|this| match self.size {
                Size::XSmall => this.gap_1(),
                Size::Small => this.gap_1(),
                _ => this.gap_2(),
            })
            .when_some(self.icon, |this, icon| {
                this.child(
                    icon.loading_icon(self.loading_icon)
                        .loading(self.loading)
                        .with_size(icon_size),
                )
            })
            .when_some(self.label, |this, label| {
                this.child(
                    div()
                        .min_w_0()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .truncate()
                        .line_height(relative(1.))
                        .child(label),
                )
            })
            .children(children)
            .when(self.dropdown_caret, |this| {
                this.when(has_content, |this| this.justify_between())
                    .child(Caret::new(self.size).text_color(normal_style.fg.opacity(0.75)))
            });
        root.role(self.role.resolve(|| {
            if self.variant.is_link() {
                Role::Link
            } else {
                Role::Button
            }
        }))
        .selected(self.selected)
        .disabled(disabled)
        // Base layers semantic states over the builder chain, so the caller's
        // own style is replayed inside each state to keep it the closest layer.
        .styles(|styles| {
            styles
                .selected(|style| {
                    style
                        .bg(selected_style.bg)
                        .border_color(selected_style.border)
                        .text_color(selected_style.fg)
                        .refine_style(&instance_style)
                })
                .disabled(|style| {
                    style
                        .bg(disabled_style.bg)
                        .text_color(disabled_style.fg)
                        .border_color(disabled_style.border)
                        .shadow_none()
                        .refine_style(&instance_style)
                })
        })
        .when_some(accessibility_label, |this, label| {
            this.accessibility_label(label)
        })
        .when_some(self.toggled, |this, toggled| {
            this.aria_toggled(if toggled {
                gpui::accesskit::Toggled::True
            } else {
                gpui::accesskit::Toggled::False
            })
        })
        .track_focus(&focus_handle)
        .tab_index(self.tab_index)
        .tab_stop(self.tab_stop)
        .child(content)
        // Fade the whole button while loading, so every variant is dimmed by
        // the same amount. Fading `bg`, `border` and `fg` one by one instead
        // only shows up on variants that have a background to begin with:
        // `Ghost`, `Link` and `Text` are transparent, so an alpha on their
        // background changes nothing.
        .when(loading && !disabled, |this| this.opacity(0.8))
        .when(!disabled, |this| {
            this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                if loading {
                    cx.stop_propagation();
                    return;
                }

                // Avoid focus on mouse down.
                window.prevent_default();

                // Pressing a button must not start the window-level text selection.
                crate::global_state::GlobalState::suppress_text_selection(cx);
            })
        })
        .when_some(self.on_click, |this, on_click| {
            this.on_click(move |event, window, cx| {
                if loading {
                    cx.stop_propagation();
                    return;
                }

                on_click(event, window, cx);
            })
        })
        .when_some(self.on_hover.filter(|_| hoverable), |this, on_hover| {
            this.on_hover(move |hovered, window, cx| {
                on_hover(hovered, window, cx);
            })
        })
        .map(|this| {
            if let Some(builder) = self.tooltip_builder {
                this.managed_tooltip(move |window, cx| builder(window, cx))
            } else if let Some((tooltip, action)) = self.tooltip {
                this.managed_tooltip(move |window, cx| {
                    Tooltip::new(tooltip.clone())
                        .overlay_anchored()
                        .when_some(action.clone(), |this, (action, context)| {
                            this.action(
                                action.boxed_clone().as_ref(),
                                context.as_ref().map(|c| c.as_ref()),
                            )
                        })
                        .build(window, cx)
                })
            } else {
                this
            }
        })
        .when(is_focused && self.focus_ring_enabled, |this| {
            this.focus_ring_style(window, cx)
        })
    }
}

struct ButtonVariantStyle {
    bg: Background,
    border: Hsla,
    fg: Hsla,
    underline: bool,
    shadow: bool,
}

#[derive(Clone, Copy)]
enum ButtonStyleState {
    Normal,
    Hovered,
    Active,
    Selected,
}

// Ghost and neutral-outline interaction layers follow the original
// gpui-component foreground-scrim recipe. They are intentionally stronger
// than the generic row wash: a compact control needs an unmistakable pointer
// response, while its resting plate remains quiet.
const BUTTON_HOVER_INK: f32 = 0.12;
const BUTTON_ACTIVE_INK: f32 = 0.20;

impl ButtonVariant {
    fn outline_background(&self, state: ButtonStyleState, cx: &mut App) -> Background {
        let t = cx.theme();
        let semantic_plate = |color: Hsla| match state {
            ButtonStyleState::Normal => color.opacity(0.10).into(),
            ButtonStyleState::Hovered => color.opacity(0.20).into(),
            ButtonStyleState::Active => color.opacity(0.14).into(),
            ButtonStyleState::Selected => color.opacity(0.30).into(),
        };
        match self {
            Self::Default | Self::Secondary => match state {
                ButtonStyleState::Normal => t.input_bg.into(),
                ButtonStyleState::Hovered => t.ink(BUTTON_HOVER_INK).into(),
                ButtonStyleState::Active => t.ink(BUTTON_ACTIVE_INK).into(),
                ButtonStyleState::Selected => t.wash(WASH_SELECTED).into(),
            },
            Self::Primary => semantic_plate(t.accent),
            Self::Danger => semantic_plate(t.danger),
            Self::Warning => semantic_plate(t.warning),
            Self::Success => semantic_plate(t.success),
            Self::Info => semantic_plate(t.busy),
            Self::Ghost | Self::Link | Self::Text => t.transparent.into(),
            Self::Custom(colors) => colors.color.mix_oklab(t.transparent, 0.2).into(),
        }
    }

    fn bg_color(&self, outline: bool, cx: &mut App) -> Background {
        if outline {
            return self.outline_background(ButtonStyleState::Normal, cx);
        }
        let t = cx.theme();
        match self {
            Self::Default => t.surface_raised.into(),
            Self::Primary => t.accent_strong.into(),
            Self::Secondary => t.surface_raised.into(),
            Self::Danger => t.danger_strong.into(),
            Self::Warning => t.warning.into(),
            Self::Success => t.success.into(),
            Self::Info => t.busy.into(),
            Self::Ghost | Self::Link | Self::Text => t.transparent.into(),
            Self::Custom(colors) => colors.color.mix_oklab(t.transparent, 0.2).into(),
        }
    }

    fn text_color(&self, outline: bool, cx: &mut App) -> Hsla {
        let t = cx.theme();
        match self {
            Self::Default | Self::Secondary | Self::Ghost => t.text,
            Self::Primary => {
                if outline {
                    t.accent
                } else {
                    t.on_accent
                }
            }
            Self::Danger => {
                if outline {
                    t.danger
                } else {
                    t.on_accent
                }
            }
            Self::Warning => {
                if outline {
                    t.warning
                } else {
                    t.on_accent
                }
            }
            Self::Success => {
                if outline {
                    t.success
                } else {
                    t.on_accent
                }
            }
            Self::Info => {
                if outline {
                    t.busy
                } else {
                    t.on_accent
                }
            }
            Self::Link => t.accent,
            Self::Text => t.text.opacity(0.9),
            Self::Custom(colors) => colors.foreground,
        }
    }

    fn border_color(&self, outline: bool, cx: &mut App) -> Hsla {
        let t = cx.theme();
        match self {
            Self::Default | Self::Secondary => t.border_strong,
            Self::Primary => {
                if outline {
                    t.accent
                } else {
                    t.accent_strong
                }
            }
            Self::Danger => {
                if outline {
                    t.danger.mix_oklab(transparent_white(), 0.4)
                } else {
                    t.danger_strong
                }
            }
            Self::Info => {
                if outline {
                    t.busy.mix_oklab(transparent_white(), 0.4)
                } else {
                    t.busy
                }
            }
            Self::Warning => {
                if outline {
                    t.warning.mix_oklab(transparent_white(), 0.4)
                } else {
                    t.warning
                }
            }
            Self::Success => {
                if outline {
                    t.success.mix_oklab(transparent_white(), 0.4)
                } else {
                    t.success
                }
            }
            Self::Ghost | Self::Link | Self::Text => t.transparent,
            Self::Custom(colors) => {
                if outline {
                    colors.color.mix_oklab(transparent_white(), 0.4)
                } else {
                    colors.color
                }
            }
        }
    }

    fn underline(&self, _: &App) -> bool {
        match self {
            Self::Link => true,
            _ => false,
        }
    }

    fn shadow(&self, _outline: bool, _: &App) -> bool {
        match self {
            Self::Custom(c) => c.shadow,
            _ => false,
        }
    }

    fn normal(&self, outline: bool, cx: &mut App) -> ButtonVariantStyle {
        let bg = self.bg_color(outline, cx);
        let border = self.border_color(outline, cx);
        let fg = self.text_color(outline, cx);
        let underline = self.underline(cx);
        let shadow = self.shadow(outline, cx);

        ButtonVariantStyle {
            bg,
            border,
            fg,
            underline,
            shadow,
        }
    }

    fn hovered(&self, outline: bool, cx: &mut App) -> ButtonVariantStyle {
        let dark = cx.theme().is_dark();
        let hover_tone = |color: Hsla| {
            if dark {
                color.lighten(0.06)
            } else {
                color.darken(0.06)
            }
        };
        let bg: Background = match self {
            Self::Default => {
                if outline {
                    self.outline_background(ButtonStyleState::Hovered, cx)
                } else {
                    cx.theme().surface_raised_hover.into()
                }
            }
            Self::Primary => {
                if outline {
                    self.outline_background(ButtonStyleState::Hovered, cx)
                } else {
                    hover_tone(cx.theme().accent_strong).into()
                }
            }
            Self::Secondary => {
                if outline {
                    self.outline_background(ButtonStyleState::Hovered, cx)
                } else {
                    cx.theme().surface_raised_hover.into()
                }
            }
            Self::Danger => {
                if outline {
                    self.outline_background(ButtonStyleState::Hovered, cx)
                } else {
                    hover_tone(cx.theme().danger_strong).into()
                }
            }
            Self::Warning => {
                if outline {
                    self.outline_background(ButtonStyleState::Hovered, cx)
                } else {
                    hover_tone(cx.theme().warning).into()
                }
            }
            Self::Success => {
                if outline {
                    self.outline_background(ButtonStyleState::Hovered, cx)
                } else {
                    hover_tone(cx.theme().success).into()
                }
            }
            Self::Info => {
                if outline {
                    self.outline_background(ButtonStyleState::Hovered, cx)
                } else {
                    hover_tone(cx.theme().busy).into()
                }
            }
            Self::Custom(colors) => colors.hover.into(),
            Self::Ghost => cx.theme().ink(BUTTON_HOVER_INK).into(),
            Self::Link | Self::Text => cx.theme().transparent.into(),
        };

        let border = self.border_color(outline, cx);
        let fg = match self {
            Self::Link => cx.theme().accent,
            Self::Text => cx.theme().text,
            _ => self.text_color(outline, cx),
        };

        let underline = self.underline(cx);
        let shadow = self.shadow(outline, cx);

        ButtonVariantStyle {
            bg,
            border,
            fg,
            underline,
            shadow,
        }
    }

    fn active(&self, outline: bool, cx: &mut App) -> ButtonVariantStyle {
        let active_tone = |color: Hsla| color.darken(0.10);
        let neutral_active = cx.theme().surface_raised_hover.darken(0.08);
        let bg = match self {
            Self::Default => {
                if outline {
                    self.outline_background(ButtonStyleState::Active, cx)
                } else {
                    neutral_active.into()
                }
            }
            Self::Primary => {
                if outline {
                    self.outline_background(ButtonStyleState::Active, cx)
                } else {
                    active_tone(cx.theme().accent_strong).into()
                }
            }
            Self::Secondary => {
                if outline {
                    self.outline_background(ButtonStyleState::Active, cx)
                } else {
                    neutral_active.into()
                }
            }
            Self::Ghost => cx.theme().ink(BUTTON_ACTIVE_INK).into(),
            Self::Danger => {
                if outline {
                    self.outline_background(ButtonStyleState::Active, cx)
                } else {
                    active_tone(cx.theme().danger_strong).into()
                }
            }
            Self::Warning => {
                if outline {
                    self.outline_background(ButtonStyleState::Active, cx)
                } else {
                    active_tone(cx.theme().warning).into()
                }
            }
            Self::Success => {
                if outline {
                    self.outline_background(ButtonStyleState::Active, cx)
                } else {
                    active_tone(cx.theme().success).into()
                }
            }
            Self::Info => {
                if outline {
                    self.outline_background(ButtonStyleState::Active, cx)
                } else {
                    active_tone(cx.theme().busy).into()
                }
            }
            Self::Custom(colors) => colors.active.into(),
            Self::Link | Self::Text => cx.theme().transparent.into(),
        };
        let border = self.border_color(outline, cx);
        let fg = match self {
            Self::Link => cx.theme().accent.darken(0.10),
            Self::Text => cx.theme().text.opacity(0.7),
            _ => self.text_color(outline, cx),
        };
        let underline = self.underline(cx);
        let shadow = self.shadow(outline, cx);

        ButtonVariantStyle {
            bg,
            border,
            fg,
            underline,
            shadow,
        }
    }

    fn selected(&self, outline: bool, cx: &mut App) -> ButtonVariantStyle {
        if outline {
            return ButtonVariantStyle {
                bg: self.outline_background(ButtonStyleState::Selected, cx),
                border: self.border_color(true, cx),
                fg: self.text_color(outline, cx),
                underline: self.underline(cx),
                shadow: self.shadow(outline, cx),
            };
        }

        let bg = match self {
            Self::Default | Self::Secondary => cx.theme().surface_raised_hover.into(),
            Self::Primary => cx.theme().accent_strong.darken(0.06).into(),
            Self::Ghost => cx.theme().wash(WASH_SELECTED).into(),
            Self::Danger => cx.theme().danger_strong.darken(0.06).into(),
            Self::Warning => cx.theme().warning.darken(0.06).into(),
            Self::Success => cx.theme().success.darken(0.06).into(),
            Self::Info => cx.theme().busy.darken(0.06).into(),
            Self::Link | Self::Text => cx.theme().transparent.into(),
            Self::Custom(colors) => colors.active.into(),
        };

        let border = self.border_color(outline, cx);
        let fg = match self {
            Self::Link => cx.theme().accent,
            Self::Text => cx.theme().text.opacity(0.7),
            _ => self.text_color(false, cx),
        };
        let underline = self.underline(cx);
        let shadow = self.shadow(outline, cx);

        ButtonVariantStyle {
            bg,
            border,
            fg,
            underline,
            shadow,
        }
    }

    fn disabled(&self, outline: bool, cx: &mut App) -> ButtonVariantStyle {
        let bg = match self {
            Self::Default | Self::Link | Self::Ghost | Self::Text => cx.theme().transparent.into(),
            Self::Primary => cx.theme().accent_strong.opacity(0.15).into(),
            Self::Danger => cx.theme().danger_strong.opacity(0.15).into(),
            Self::Warning => cx.theme().warning.opacity(0.15).into(),
            Self::Success => cx.theme().success.opacity(0.15).into(),
            Self::Info => cx.theme().busy.opacity(0.15).into(),
            Self::Secondary => cx.theme().surface_raised.opacity(0.5).into(),
            Self::Custom(style) => style.color.opacity(0.15).into(),
        };
        let fg = cx.theme().text_muted.opacity(0.5);
        let (bg, border) = if outline {
            (
                self.outline_background(ButtonStyleState::Normal, cx)
                    .opacity(0.5),
                self.border_color(true, cx).opacity(0.5),
            )
        } else if let Self::Default = self {
            (
                cx.theme().surface_raised.opacity(0.5).into(),
                cx.theme().border.opacity(0.5),
            )
        } else {
            let border = match self {
                Self::Primary => cx.theme().accent_strong.opacity(0.15),
                Self::Secondary => cx.theme().border_strong.opacity(0.5),
                Self::Danger => cx.theme().danger_strong.opacity(0.15),
                Self::Warning => cx.theme().warning.opacity(0.15),
                Self::Success => cx.theme().success.opacity(0.15),
                Self::Info => cx.theme().busy.opacity(0.15),
                Self::Custom(style) => style.color.opacity(0.15),
                Self::Default | Self::Link | Self::Ghost | Self::Text => cx.theme().transparent,
            };
            (bg, border)
        };

        let underline = self.underline(cx);
        let shadow = false;

        ButtonVariantStyle {
            bg,
            border,
            fg,
            underline,
            shadow,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IconName;
    use gpui::px;

    #[gpui::test]
    fn disabled_legacy_button_keeps_existing_pointer_blocking(cx: &mut gpui::TestAppContext) {
        use std::{cell::Cell, rc::Rc};

        use gpui::{Context, Modifiers, Render, point};

        struct Harness(Rc<Cell<usize>>, Rc<Cell<usize>>);

        impl Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let button_clicks = self.0.clone();
                let parent_clicks = self.1.clone();
                div()
                    .id("disabled-button-parent")
                    .tab_group()
                    .size(px(100.))
                    .on_click(move |_, _, _| parent_clicks.set(parent_clicks.get() + 1))
                    .child(
                        Button::new("disabled-legacy")
                            .disabled(true)
                            .size_full()
                            .on_click(move |_, _, _| button_clicks.set(button_clicks.get() + 1)),
                    )
            }
        }

        cx.update(crate::init);
        let button_clicks = Rc::new(Cell::new(0));
        let parent_clicks = Rc::new(Cell::new(0));
        let (_, cx) = cx.add_window_view({
            let button_clicks = button_clicks.clone();
            let parent_clicks = parent_clicks.clone();
            move |_, _| Harness(button_clicks, parent_clicks)
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
        cx.update(|window, cx| window.focus_next(cx));
        cx.simulate_keystrokes("enter space");

        assert_eq!(button_clicks.get(), 0);
        assert_eq!(parent_clicks.get(), 0);
        cx.update(|window, cx| assert!(window.focused(cx).is_none()));
    }

    #[gpui::test]
    fn enabled_button_delegates_pointer_enter_and_space_once(cx: &mut gpui::TestAppContext) {
        use std::{cell::Cell, rc::Rc};

        use gpui::{
            ClickEvent, Context, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, Render, point,
        };

        struct Harness(Rc<Cell<usize>>, Rc<Cell<usize>>);

        impl Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let clicks = self.0.clone();
                let keyboard_clicks = self.1.clone();
                div().tab_group().size(px(100.)).child(
                    Button::new("enabled-legacy")
                        .size_full()
                        .on_click(move |event, _, _| {
                            clicks.set(clicks.get() + 1);
                            if matches!(event, ClickEvent::Keyboard(_)) {
                                keyboard_clicks.set(keyboard_clicks.get() + 1);
                            }
                        }),
                )
            }
        }

        cx.update(crate::init);
        let clicks = Rc::new(Cell::new(0));
        let keyboard_clicks = Rc::new(Cell::new(0));
        let (_, cx) = cx.add_window_view({
            let clicks = clicks.clone();
            let keyboard_clicks = keyboard_clicks.clone();
            move |_, _| Harness(clicks, keyboard_clicks)
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
        cx.update(|window, cx| window.focus_next(cx));
        cx.update(|window, cx| {
            assert!(window.focused(cx).is_some());
            window.draw(cx).clear(cx);
        });
        for key in ["enter", "space"] {
            let keystroke = Keystroke::parse(key).unwrap();
            cx.simulate_event(KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            });
            cx.simulate_event(KeyUpEvent { keystroke });
        }

        assert_eq!(clicks.get(), 3);
        assert_eq!(keyboard_clicks.get(), 2);
    }

    #[gpui::test]
    fn loading_button_keeps_existing_pointer_blocking(cx: &mut gpui::TestAppContext) {
        use std::{cell::Cell, rc::Rc};

        use gpui::{Context, Modifiers, Render, point};

        struct Harness(Rc<Cell<usize>>, Rc<Cell<usize>>);

        impl Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let button_clicks = self.0.clone();
                let parent_clicks = self.1.clone();
                div()
                    .id("loading-button-parent")
                    .tab_group()
                    .size(px(100.))
                    .on_click(move |_, _, _| parent_clicks.set(parent_clicks.get() + 1))
                    .child(
                        Button::new("loading-legacy")
                            .loading(true)
                            .size_full()
                            .on_click(move |_, _, _| button_clicks.set(button_clicks.get() + 1)),
                    )
            }
        }

        cx.update(crate::init);
        let button_clicks = Rc::new(Cell::new(0));
        let parent_clicks = Rc::new(Cell::new(0));
        let (_, cx) = cx.add_window_view({
            let button_clicks = button_clicks.clone();
            let parent_clicks = parent_clicks.clone();
            move |_, _| Harness(button_clicks, parent_clicks)
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
        cx.update(|window, cx| window.focus_next(cx));
        cx.simulate_keystrokes("enter space");

        assert_eq!(button_clicks.get(), 0);
        assert_eq!(parent_clicks.get(), 0);
        cx.update(|window, cx| assert!(window.focused(cx).is_some()));
    }

    #[gpui::test]
    fn loading_button_without_callback_still_blocks_parent_activation(
        cx: &mut gpui::TestAppContext,
    ) {
        use std::{cell::Cell, rc::Rc};

        use gpui::{Context, Modifiers, Render, point};

        struct Harness(Rc<Cell<usize>>);

        impl Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let parent_clicks = self.0.clone();
                div()
                    .id("loading-without-callback-parent")
                    .tab_group()
                    .size(px(100.))
                    .on_click(move |_, _, _| parent_clicks.set(parent_clicks.get() + 1))
                    .child(
                        Button::new("loading-without-callback")
                            .loading(true)
                            .size_full(),
                    )
            }
        }

        cx.update(crate::init);
        let parent_clicks = Rc::new(Cell::new(0));
        let (_, cx) = cx.add_window_view({
            let parent_clicks = parent_clicks.clone();
            move |_, _| Harness(parent_clicks)
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
        cx.update(|window, cx| window.focus_next(cx));
        cx.simulate_keystrokes("enter space");

        assert_eq!(parent_clicks.get(), 0);
        cx.update(|window, cx| assert!(window.focused(cx).is_some()));
    }

    #[gpui::test]
    fn test_button_builder(_cx: &mut gpui::TestAppContext) {
        let button = Button::new("complex-button")
            .label("Save Changes")
            .primary()
            .outline()
            .large()
            .tooltip("Click to save")
            .compact()
            .loading(false)
            .disabled(false)
            .selected(false)
            .tab_index(1)
            .tab_stop(true)
            .dropdown_caret(false)
            .rounded(ButtonRounded::Medium)
            .on_click(|_, _, _| {});

        assert_eq!(button.label, Some("Save Changes".into()));
        assert_eq!(button.variant, ButtonVariant::Primary);
        assert!(button.outline);
        assert_eq!(button.size, Size::Large);
        assert!(button.tooltip.is_some());
        assert!(button.compact);
        assert!(!button.loading);
        assert!(!button.disabled);
        assert!(!button.selected);
        assert_eq!(button.toggled, None);
        assert_eq!(button.tab_index, 1);
        assert!(button.tab_stop);
        assert!(!button.dropdown_caret);
        assert!(matches!(button.rounded, ButtonRounded::Medium));
    }

    #[test]
    fn selected_trigger_presentation_does_not_imply_toggled_accessibility() {
        let selected = Button::new("menu-trigger").selected(true);
        assert!(selected.selected);
        assert_eq!(selected.toggled, None);

        let selected_toggle = Button::new("explicit-toggle").selected(true).toggled(true);
        assert!(selected_toggle.selected);
        assert_eq!(selected_toggle.toggled, Some(true));
    }

    /// A loading button must be as inert as a disabled one. `interactive` is what
    /// gates the hover and active styling, the `cursor_pointer` of link buttons and
    /// the `mouse_down` handler, none of which depend on a listener being set.
    #[gpui::test]
    fn test_button_loading_is_not_interactive(_cx: &mut gpui::TestAppContext) {
        assert!(Button::new("test").interactive());
        assert!(!Button::new("test").loading(true).interactive());
        assert!(!Button::new("test").disabled(true).interactive());
        assert!(
            !Button::new("test")
                .loading(true)
                .disabled(true)
                .interactive()
        );

        // Loading gates hovering even when an `on_hover` listener is set.
        let loading = Button::new("test").loading(true).on_hover(|_, _, _| {});
        assert!(!loading.hoverable());
    }

    /// `selected` is styling only; the toggle state must be opted into, so that
    /// ordinary buttons are not announced as toggle buttons.
    #[gpui::test]
    fn test_button_variant_methods(_cx: &mut gpui::TestAppContext) {
        // Test variant check methods
        assert!(ButtonVariant::Link.is_link());
        assert!(ButtonVariant::Text.is_text());
        assert!(ButtonVariant::Ghost.is_ghost());

        // Test no_padding logic
        assert!(ButtonVariant::Link.no_padding());
        assert!(ButtonVariant::Text.no_padding());
        assert!(!ButtonVariant::Ghost.no_padding());
    }

    #[gpui::test]
    fn link_button_prepaints_its_child(cx: &mut gpui::TestAppContext) {
        use gpui::{Context, Render};

        struct LinkButtonHarness;

        impl Render for LinkButtonHarness {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                Button::new("link-button").link().child(
                    div()
                        .debug_selector(|| "link-button-child".into())
                        .child("Visible link button"),
                )
            }
        }

        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|_, _| LinkButtonHarness);
        cx.update(|window, cx| window.draw(cx).clear(cx));

        let bounds = cx
            .debug_bounds("link-button-child")
            .expect("link button child must participate in layout and prepaint");
        assert!(bounds.size.width > px(0.));
        assert!(bounds.size.height > px(0.));
    }

    #[gpui::test]
    fn base_slot_prepaints_complete_button_content(cx: &mut gpui::TestAppContext) {
        use gpui::{Context, Render};

        struct CompleteButtonHarness;

        impl Render for CompleteButtonHarness {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div()
                    .child(
                        Button::new("complete-button")
                            .icon(IconName::Plus)
                            .label("Create")
                            .child(
                                div()
                                    .debug_selector(|| "button-custom-content".into())
                                    .child("Details"),
                            )
                            .dropdown_caret(true),
                    )
                    .child(
                        Button::new("loading-button")
                            .icon(IconName::Plus)
                            .loading_icon(IconName::Loader)
                            .loading(true)
                            .label("Creating")
                            .dropdown_caret(true),
                    )
            }
        }

        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|_, _| CompleteButtonHarness);
        cx.update(|window, cx| window.draw(cx).clear(cx));

        let bounds = cx
            .debug_bounds("button-custom-content")
            .expect("application content must prepaint through the Base child seam");
        assert!(bounds.size.width > px(0.));
        assert!(bounds.size.height > px(0.));
    }

    #[gpui::test]
    fn test_outline_selected_is_distinct_from_transient_press(cx: &mut gpui::TestAppContext) {
        cx.update(crate::init);
        let window = cx.add_empty_window();
        window.update(|_, cx| {
            let variant = ButtonVariant::Danger;
            let active_style = variant.active(true, cx);
            let selected_style = variant.selected(true, cx);

            assert_ne!(selected_style.bg, active_style.bg);
            assert_eq!(selected_style.bg, cx.theme().danger.opacity(0.30).into());
            assert_eq!(selected_style.border, active_style.border);
            assert_eq!(selected_style.fg, cx.theme().danger);
            assert_ne!(selected_style.bg, cx.theme().danger_strong.into());
        });
    }

    #[gpui::test]
    fn test_primary_button_paints_from_roles(cx: &mut gpui::TestAppContext) {
        cx.update(crate::init);
        let window = cx.add_empty_window();
        window.update(|_, cx| {
            assert_eq!(
                ButtonVariant::Primary.normal(false, cx).bg,
                cx.theme().accent_strong.into()
            );
            assert_eq!(
                ButtonVariant::Primary.hovered(false, cx).bg,
                cx.theme().accent_strong.darken(0.06).into()
            );
            assert_eq!(
                ButtonVariant::Primary.active(false, cx).bg,
                cx.theme().accent_strong.darken(0.10).into()
            );
            assert_ne!(
                ButtonVariant::Primary.hovered(false, cx).bg,
                ButtonVariant::Primary.active(false, cx).bg
            );
            assert_eq!(
                ButtonVariant::Primary.normal(false, cx).fg,
                cx.theme().on_accent
            );
            assert_eq!(
                ButtonVariant::Default.normal(false, cx).bg,
                cx.theme().surface_raised.into()
            );
            assert_eq!(
                ButtonVariant::Default.hovered(false, cx).bg,
                cx.theme().surface_raised_hover.into()
            );
            assert_eq!(
                ButtonVariant::Secondary.normal(false, cx).bg,
                cx.theme().surface_raised.into()
            );
            assert_eq!(
                ButtonVariant::Secondary.normal(false, cx).border,
                cx.theme().border_strong
            );
            assert_eq!(
                ButtonVariant::Default.normal(true, cx).bg,
                cx.theme().input_bg.into()
            );
            assert_eq!(
                ButtonVariant::Default.hovered(true, cx).bg,
                cx.theme().ink(BUTTON_HOVER_INK).into()
            );
            assert_eq!(
                ButtonVariant::Ghost.hovered(false, cx).bg,
                cx.theme().ink(BUTTON_HOVER_INK).into()
            );
            assert_eq!(
                ButtonVariant::Ghost.active(false, cx).bg,
                cx.theme().ink(BUTTON_ACTIVE_INK).into()
            );

            crate::Theme::change(crate::ThemeMode::Dark, None, cx);
            assert_eq!(
                ButtonVariant::Primary.hovered(false, cx).bg,
                cx.theme().accent_strong.lighten(0.06).into()
            );
            assert_eq!(
                ButtonVariant::Primary.active(false, cx).bg,
                cx.theme().accent_strong.darken(0.10).into()
            );
            assert_ne!(
                ButtonVariant::Primary.hovered(false, cx).bg,
                ButtonVariant::Primary.active(false, cx).bg
            );
            assert_eq!(
                ButtonVariant::Default.hovered(false, cx).bg,
                cx.theme().surface_raised_hover.into()
            );
            assert_eq!(
                ButtonVariant::Default.normal(true, cx).bg,
                cx.theme().input_bg.into()
            );
            assert_eq!(
                ButtonVariant::Default.hovered(true, cx).bg,
                cx.theme().ink(BUTTON_HOVER_INK).into()
            );
            assert_eq!(
                ButtonVariant::Default.active(true, cx).bg,
                cx.theme().ink(BUTTON_ACTIVE_INK).into()
            );
        });
    }

    #[gpui::test]
    fn test_outline_primary_uses_accent_emphasis(cx: &mut gpui::TestAppContext) {
        cx.update(crate::init);
        let window = cx.add_empty_window();
        window.update(|_, cx| {
            assert_eq!(
                ButtonVariant::Primary.normal(true, cx).bg,
                cx.theme().accent.opacity(0.1).into()
            );
            assert_eq!(
                ButtonVariant::Primary.hovered(true, cx).bg,
                cx.theme().accent.opacity(0.2).into()
            );
            assert_eq!(
                ButtonVariant::Primary.active(true, cx).bg,
                cx.theme().accent.opacity(0.14).into()
            );
            assert_eq!(
                ButtonVariant::Primary.normal(true, cx).fg,
                cx.theme().accent
            );
            assert_eq!(
                ButtonVariant::Primary.normal(true, cx).border,
                cx.theme().accent
            );
        });
    }

    #[gpui::test]
    fn test_buttons_paint_from_theme_roles(cx: &mut gpui::TestAppContext) {
        cx.update(crate::init);
        let window = cx.add_empty_window();
        window.update(|_, cx| {
            assert_eq!(
                ButtonVariant::Primary.normal(false, cx).bg,
                cx.theme().accent_strong.into()
            );
            assert_eq!(
                ButtonVariant::Danger.normal(false, cx).bg,
                cx.theme().danger_strong.into()
            );
            assert_eq!(
                ButtonVariant::Danger.normal(false, cx).fg,
                cx.theme().on_accent
            );
            assert_eq!(
                ButtonVariant::Warning.normal(false, cx).bg,
                cx.theme().warning.into()
            );
            assert_eq!(
                ButtonVariant::Success.normal(false, cx).bg,
                cx.theme().success.into()
            );
            assert_eq!(
                ButtonVariant::Info.normal(false, cx).bg,
                cx.theme().busy.into()
            );
            assert_eq!(
                ButtonVariant::Secondary.normal(false, cx).bg,
                cx.theme().surface_raised.into()
            );
            assert_eq!(ButtonVariant::Ghost.normal(false, cx).fg, cx.theme().text);
            assert_eq!(ButtonVariant::Link.normal(false, cx).fg, cx.theme().accent);
            assert_eq!(
                ButtonVariant::Primary.normal(true, cx).bg,
                cx.theme().accent.opacity(0.1).into()
            );
            assert_eq!(
                ButtonVariant::Danger.normal(true, cx).bg,
                cx.theme().danger.opacity(0.1).into()
            );
            assert_eq!(
                ButtonVariant::Danger.hovered(true, cx).bg,
                cx.theme().danger.opacity(0.2).into()
            );
            assert_eq!(
                ButtonVariant::Danger.active(true, cx).bg,
                cx.theme().danger.opacity(0.14).into()
            );
        });
    }

    #[gpui::test]
    fn every_builtin_variant_separates_hover_press_and_selection(cx: &mut gpui::TestAppContext) {
        cx.update(crate::init);
        let window = cx.add_empty_window();
        window.update(|_, cx| {
            let filled_variants = [
                ButtonVariant::Default,
                ButtonVariant::Primary,
                ButtonVariant::Secondary,
                ButtonVariant::Danger,
                ButtonVariant::Warning,
                ButtonVariant::Success,
                ButtonVariant::Info,
                ButtonVariant::Ghost,
            ];

            for variant in filled_variants {
                assert_ne!(
                    variant.hovered(false, cx).bg,
                    variant.active(false, cx).bg,
                    "{variant:?} filled hover and press"
                );
            }

            let outline_variants = [
                ButtonVariant::Default,
                ButtonVariant::Primary,
                ButtonVariant::Secondary,
                ButtonVariant::Danger,
                ButtonVariant::Warning,
                ButtonVariant::Success,
                ButtonVariant::Info,
            ];
            for variant in outline_variants {
                assert_ne!(
                    variant.active(true, cx).bg,
                    variant.selected(true, cx).bg,
                    "{variant:?} outline press and selection"
                );
            }

            assert_eq!(
                ButtonVariant::Default.normal(true, cx).bg,
                cx.theme().input_bg.into()
            );
            assert_eq!(
                ButtonVariant::Default.hovered(true, cx).bg,
                cx.theme().ink(BUTTON_HOVER_INK).into()
            );
            assert_eq!(
                ButtonVariant::Default.active(true, cx).bg,
                cx.theme().ink(BUTTON_ACTIVE_INK).into()
            );
            assert_eq!(
                ButtonVariant::Default.selected(true, cx).bg,
                cx.theme().wash(WASH_SELECTED).into()
            );
        });
    }

    #[gpui::test]
    fn test_disabled_outline_buttons_keep_role_washes(cx: &mut gpui::TestAppContext) {
        cx.update(crate::init);
        let window = cx.add_empty_window();
        window.update(|_, cx| {
            assert_eq!(
                ButtonVariant::Primary.disabled(true, cx).bg,
                gpui::Background::from(cx.theme().accent.opacity(0.1)).opacity(0.5)
            );
            assert_eq!(
                ButtonVariant::Danger.disabled(true, cx).bg,
                gpui::Background::from(cx.theme().danger.opacity(0.1)).opacity(0.5)
            );
            assert_ne!(
                ButtonVariant::Danger.disabled(true, cx).bg,
                cx.theme().input_bg.opacity(0.5).into()
            );
            assert_ne!(
                ButtonVariant::Danger.disabled(true, cx).bg,
                cx.theme().danger_strong.opacity(0.15).into()
            );
        });
    }
}
