use crate::{Spacing, Theme};
use gpui::{
    App, ClickEvent, ElementId, IntoElement, ParentElement, Refineable, RenderOnce, SharedString,
    Window, div, prelude::*,
};
use std::rc::Rc;

/// Layout wrappers. GPUI styling remains available through [`Styled`].
macro_rules! layout {
    ($name:ident, $direction:ident) => {
        #[doc = concat!("A reusable ", stringify!($name), " layout. Use `.child(...)`, `.gap(...)`, and `.padding(...)`.")]
        #[derive(IntoElement)]
        pub struct $name { children: Vec<gpui::AnyElement>, padding: Spacing, gap: Spacing, style: gpui::StyleRefinement }
        impl Default for $name { fn default() -> Self { Self::new() } }
        impl $name {
            pub fn new() -> Self { Self { children: Vec::new(), padding: Spacing::None, gap: Spacing::Md, style: Default::default() } }
            pub fn gap(mut self, gap: Spacing) -> Self { self.gap = gap; self }
            pub fn padding(mut self, padding: Spacing) -> Self { self.padding = padding; self }
        }
        impl ParentElement for $name {
            fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) { self.children.extend(elements); }
        }
        impl Styled for $name { fn style(&mut self) -> &mut gpui::StyleRefinement { &mut self.style } }
        impl RenderOnce for $name {
            fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
                let theme = Theme::get(cx);
                let mut element = div().flex().$direction().gap(self.gap.pixels(theme)).p(self.padding.pixels(theme));
                element.style().refine(&self.style);
                element.children(self.children)
            }
        }
    };
}

layout!(Column, flex_col);
layout!(Row, flex_row);
layout!(Container, flex_col);

/// A text label using inherited typography.
#[derive(IntoElement)]
pub struct Text {
    value: SharedString,
    muted: bool,
    heading: bool,
}

impl Text {
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            muted: false,
            heading: false,
        }
    }
    pub fn muted(mut self) -> Self {
        self.muted = true;
        self
    }
    pub fn heading(mut self) -> Self {
        self.heading = true;
        self
    }
}

impl RenderOnce for Text {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        div()
            .text_color(if self.muted {
                theme.colors.muted
            } else {
                theme.colors.text
            })
            .when(self.heading, |element| {
                element
                    .text_size(theme.font_size * 1.5)
                    .font_weight(gpui::FontWeight::SEMIBOLD)
            })
            .child(self.value)
    }
}

type Click = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
type Change = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// A focusable, keyboard-activatable button with a stable ID.
///
/// ```no_run
/// use gpui_framework::Button;
/// let button = Button::new("save", "Save").on_click(|_, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    disabled: bool,
    primary: bool,
    callback: Option<Click>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            primary: false,
            callback: None,
        }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn primary(mut self) -> Self {
        self.primary = true;
        self
    }
    pub fn on_click(
        mut self,
        callback: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callback = Some(Rc::new(callback));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let focus = theme.colors.focus;
        let border = theme.colors.border;
        let callback = self.callback;
        div()
            .id(self.id)
            .debug_selector(|| self.label.to_string())
            .role(gpui::Role::Button)
            .aria_label(self.label.clone())
            .focusable()
            .tab_index(0)
            .tab_stop(!self.disabled)
            .border_1()
            .border_color(border)
            .rounded(theme.radius)
            .px(theme.spacing.md)
            .py(theme.spacing.sm)
            .bg(if self.primary {
                theme.colors.accent
            } else {
                theme.colors.surface
            })
            .text_color(if self.primary {
                theme.colors.accent_text
            } else {
                theme.colors.text
            })
            .when(self.disabled, |element| element.opacity(0.5))
            .when(!self.disabled, |element| {
                element
                    .cursor_pointer()
                    .hover(move |style| style.border_color(focus))
                    .focus(move |style| style.border_color(focus))
                    .on_click(move |event, window, cx| {
                        if let Some(callback) = &callback {
                            callback(event, window, cx);
                        }
                    })
            })
            .child(self.label)
    }
}

/// A controlled checkbox: store its value in the parent view.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    label: SharedString,
    checked: bool,
    disabled: bool,
    callback: Option<Change>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, checked: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            checked,
            disabled: false,
            callback: None,
        }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn on_change(mut self, callback: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.callback = Some(Rc::new(callback));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let focus = theme.colors.focus;
        let checked = self.checked;
        let callback = self.callback;
        div()
            .id(self.id)
            .debug_selector(|| self.label.to_string())
            .role(gpui::Role::CheckBox)
            .aria_label(self.label.clone())
            .aria_toggled(if checked {
                gpui::Toggled::True
            } else {
                gpui::Toggled::False
            })
            .focusable()
            .tab_index(0)
            .tab_stop(!self.disabled)
            .flex()
            .items_center()
            .gap(theme.spacing.sm)
            .p(theme.spacing.sm)
            .border_1()
            .border_color(theme.colors.border)
            .rounded(theme.radius)
            .when(self.disabled, |element| element.opacity(0.5))
            .when(!self.disabled, |element| {
                element
                    .cursor_pointer()
                    .focus(move |style| style.border_color(focus))
                    .on_click(move |_, window, cx| {
                        if let Some(callback) = &callback {
                            callback(&!checked, window, cx);
                        }
                    })
            })
            .child(
                div()
                    .w(theme.spacing.lg)
                    .text_color(theme.colors.focus)
                    .child(if checked { "[x]" } else { "[ ]" }),
            )
            .child(self.label)
    }
}
