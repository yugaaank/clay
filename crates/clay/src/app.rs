use crate::{SystemError, Theme, ThemeMode};
use gpui::{
    AppContext, Bounds, Context, FocusHandle, KeyBinding, Render, WindowBounds, WindowOptions,
    actions, div, prelude::*, px, size,
};
use std::sync::{Arc, Mutex};

actions!(clay, [FocusNext, FocusPrevious, CloseWindow]);

fn init_keys(cx: &mut gpui::App) {
    for key in ["ctrl-w", "cmd-w"] {
        cx.bind_keys([KeyBinding::new(key, CloseWindow, Some("ClayRoot"))]);
    }
    cx.bind_keys([
        KeyBinding::new("tab", FocusNext, Some("ClayRoot")),
        KeyBinding::new("shift-tab", FocusPrevious, Some("ClayRoot")),
    ]);
}

/// Window configuration in logical pixels.
///
/// ```
/// use clay::Window;
/// let window = Window::new("Notes").size(900., 600.);
/// ```
pub struct Window {
    title: String,
    width: f32,
    height: f32,
}

impl Window {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: 900.,
            height: 600.,
        }
    }
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

type Launch = Box<dyn FnOnce(&mut gpui::App) -> Result<(), SystemError>>;

/// Bootstrap platform, theme, window, and keyboard focus without GPUI setup code.
/// See the settings example and generated template for complete runnable apps.
pub struct App {
    theme: ThemeMode,
    launches: Vec<Launch>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            theme: ThemeMode::Dark,
            launches: Vec::new(),
        }
    }
    pub fn theme(mut self, mode: ThemeMode) -> Self {
        self.theme = mode;
        self
    }

    /// Add a view factory. State belongs to this view's GPUI entity.
    pub fn window<V: Render + 'static>(
        mut self,
        config: Window,
        build: impl FnOnce(&mut gpui::Window, &mut Context<V>) -> V + 'static,
    ) -> Self {
        self.launches.push(Box::new(move |cx| {
            if !config.width.is_finite()
                || !config.height.is_finite()
                || config.width < 240.
                || config.height < 160.
            {
                return Err(SystemError::InvalidInput(
                    "window dimensions must be finite and at least 240 × 160".into(),
                ));
            }
            let bounds = Bounds::centered(None, size(px(config.width), px(config.height)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_min_size: Some(size(px(480.), px(320.))),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some(config.title.clone().into()),
                        ..Default::default()
                    }),
                    app_id: Some("clay".into()),
                    ..Default::default()
                },
                |window, cx| {
                    window.set_window_title(&config.title);
                    let child = cx.new(|cx| build(window, cx));
                    cx.new(|cx| {
                        let focus = cx.focus_handle();
                        if window.focused(cx).is_none() {
                            window.focus(&focus, cx);
                        }
                        Root { child, focus }
                    })
                },
            )
            .map_err(|error| SystemError::Backend(error.to_string()))?;
            Ok(())
        }));
        self
    }

    /// Run the desktop event loop. Calling without windows returns an error.
    pub fn run(self) -> Result<(), SystemError> {
        if self.launches.is_empty() {
            return Err(SystemError::InvalidInput(
                "at least one window is required".into(),
            ));
        }
        let failure = Arc::new(Mutex::new(None));
        let result = failure.clone();
        gpui_platform::application()
            .with_quit_mode(gpui::QuitMode::LastWindowClosed)
            .run(move |cx| {
                Theme::set_mode(self.theme, cx);
                init_keys(cx);
                for launch in self.launches {
                    if let Err(error) = launch(cx) {
                        if let Ok(mut failure) = failure.lock() {
                            *failure = Some(error);
                        }
                        cx.quit();
                        return;
                    }
                }
                cx.activate(true);
            });
        let error = result
            .lock()
            .map_err(|_| SystemError::Backend("application error state unavailable".into()))?
            .take();
        error.map_or(Ok(()), Err)
    }
}

struct Root<V: Render> {
    child: gpui::Entity<V>,
    focus: FocusHandle,
}

impl<V: Render + 'static> Render for Root<V> {
    fn render(&mut self, _: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get(cx);
        div()
            .size_full()
            .key_context("ClayRoot")
            .track_focus(&self.focus)
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .text_size(theme.font_size)
            .font_family(theme.font_family.clone())
            .on_action(|_: &FocusNext, window, cx| window.focus_next(cx))
            .on_action(|_: &FocusPrevious, window, cx| window.focus_prev(cx))
            .on_action(|_: &CloseWindow, window, _| window.remove_window())
            .child(self.child.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_app_fails_before_platform_initialization() {
        assert!(matches!(
            App::new().run(),
            Err(SystemError::InvalidInput(_))
        ));
    }

    #[cfg(feature = "test-support")]
    #[gpui::test]
    fn controls_support_pointer_keyboard_focus_and_disabled_state(cx: &mut gpui::TestAppContext) {
        use crate::{Button, Checkbox, Column, TextInput};
        struct Demo {
            input: gpui::Entity<TextInput>,
            clicks: usize,
            checked: bool,
        }
        impl Render for Demo {
            fn render(&mut self, _: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
                Column::new()
                    .child(Button::new("increment", "Increment").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.clicks += 1;
                            cx.notify();
                        },
                    )))
                    .child(
                        Checkbox::new("checkbox", "Check me", self.checked).on_change(cx.listener(
                            |this, checked: &bool, _, cx| {
                                this.checked = *checked;
                                cx.notify();
                            },
                        )),
                    )
                    .child(
                        Button::new("disabled", "Disabled").disabled(true).on_click(
                            cx.listener(|_, _, _, _| panic!("disabled button activated")),
                        ),
                    )
                    .child(self.input.clone())
            }
        }
        cx.update(|cx| {
            Theme::dark().apply(cx);
            init_keys(cx);
        });
        let (root, cx) = cx.add_window_view(|window, cx| {
            let child = cx.new(|cx| {
                let input = cx.new(|cx| TextInput::new("", window, cx).label("Name"));
                Demo {
                    input,
                    clicks: 0,
                    checked: false,
                }
            });
            let focus = cx.focus_handle();
            window.focus(&focus, cx);
            Root { child, focus }
        });
        let child = root.read_with(cx, |root, _| root.child.clone());
        let bounds = cx.debug_bounds("Increment").expect("button bounds");
        cx.simulate_click(bounds.center(), Default::default());
        child.read_with(cx, |child, _| assert_eq!(child.clicks, 1));
        let bounds = cx.debug_bounds("Check me").expect("checkbox bounds");
        cx.simulate_click(bounds.center(), Default::default());
        let input = child.read_with(cx, |child, _| {
            assert!(child.checked);
            child.input.clone()
        });
        cx.update(|window, cx| {
            input.update(cx, |input, cx| input.set_text("Typed name", window, cx));
        });
        input.read_with(cx, |input, _| assert_eq!(input.text(), "Typed name"));
        let bounds = cx.debug_bounds("Increment").expect("button bounds");
        cx.simulate_click(bounds.center(), Default::default());
        child.read_with(cx, |child, _| assert_eq!(child.clicks, 2));
        let bounds = cx.debug_bounds("Disabled").expect("disabled bounds");
        cx.simulate_click(bounds.center(), Default::default());
    }
}
