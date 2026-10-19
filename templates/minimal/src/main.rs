mod counter;

use counter::Counter;
use clay::prelude::*;

fn main() -> Result<(), SystemError> {
    App::new()
        .theme(ThemeMode::Dark)
        .window(Window::new("My app").size(900., 600.), Demo::new)
        .run()
}

struct Demo {
    count: Counter,
}

impl Demo {
    fn new(_window: &mut UiWindow, _cx: &mut Context<Self>) -> Self {
        Self {
            count: Counter::default(),
        }
    }
}

impl Render for Demo {
    fn render(&mut self, _window: &mut UiWindow, cx: &mut Context<Self>) -> impl IntoElement {
        Column::new()
            .padding(Spacing::Lg)
            .gap(Spacing::Md)
            .child(Text::new(format!("Count: {}", self.count.value())))
            .child(
                Button::new("increment", "Increment").on_click(cx.listener(|this, _, _, cx| {
                    this.count.increment();
                    cx.notify();
                })),
            )
    }
}
