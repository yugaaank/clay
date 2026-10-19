# Application recipes

Each recipe below is grounded in a current example or public API. Copy the
smallest pattern that matches the application; do not claim unsupported widgets
exist.

## Complete minimal application

This is a complete `src/main.rs` for a generated application. It uses only the
public facade and is the smallest known-good stateful UI:

```rust
use clay::prelude::*;

struct Counter {
    count: i32,
}

impl Render for Counter {
    fn render(&mut self, _: &mut UiWindow, cx: &mut Context<Self>) -> impl IntoElement {
        Column::new()
            .padding(Spacing::Lg)
            .gap(Spacing::Md)
            .child(Text::new(format!("Count: {}", self.count)))
            .child(Button::new("increment", "Increment").on_click(
                cx.listener(|this, _, _, cx| {
                    this.count += 1;
                    cx.notify();
                }),
            ))
    }
}

fn main() -> Result<(), SystemError> {
    App::new()
        .theme(ThemeMode::Dark)
        .window(Window::new("Counter").size(900., 600.), |_, _| Counter { count: 0 })
        .run()
}
```

Run it with:

```sh
cargo check --locked
cargo build --release --locked
```

## Counter or interactive dashboard

Use an entity field, a `Button::on_click` listener, and `cx.notify()`:

```rust
struct Dashboard { count: i32 }

impl Render for Dashboard {
    fn render(&mut self, _: &mut UiWindow, cx: &mut Context<Self>) -> impl IntoElement {
        Column::new()
            .padding(Spacing::Lg)
            .child(Text::new(format!("Count: {}", self.count)))
            .child(Button::new("increment", "Increment").on_click(
                cx.listener(|this, _, _, cx| {
                    this.count += 1;
                    cx.notify();
                }),
            ))
    }
}
```

Build with `cargo check --locked`. This covers local state updates only; it does
not provide a data-fetching or dashboard widget library.

## Header, sidebar, and content

Compose a full-size `Row`, a fixed-width `Container`, and a flexible `Column` as
shown in [`component-guide.md`](component-guide.md). Use the settings app's
`render` method as the complete reference. Build with:

```sh
cargo run --locked -p settings-app
```

The layout is verified on Linux. Responsive breakpoints and navigation are
application responsibilities.

## Settings screen

Use `examples/settings-app` as the complete, compilable recipe. It demonstrates
`TextInput`, controlled `Checkbox` values, theme switching, dirty state,
background `SettingsStore` load/save, clipboard, dialogs, and status/error text.

```sh
cargo test --locked -p settings-app --all-features
cargo run --locked -p settings-app
```

Do not move settings I/O into `render`; use its `load` and `save` methods as the
lifecycle reference.

## Form with text input and checkbox

Create the input as an entity, retain a `Subscription`, and render a controlled
checkbox. Validation is manual application state:

```rust
struct Form {
    name: Entity<TextInput>,
    enabled: bool,
    valid: bool,
    _subscriptions: Vec<Subscription>,
}
```

Subscribe to `InputEvent::Changed`, calculate `valid`, and disable the submit
button until the value is valid. There is no framework validation schema.

## Loading, error, and empty states

Use an explicit enum such as `Loading`, `Ready(data)`, and `Error(message)` and
render an empty message when `Ready` contains no records. The system monitor is a
complete reference:

```sh
cargo test --locked -p system-monitor
cargo run --locked -p system-monitor
```

Its collector is bounded to 60 history samples and 50 process rows, so it is a
useful model for avoiding unbounded UI state.

## File picker and confirmation

Combine `Dialog::open_file` and `Dialog::confirm` with explicit cancellation and
error branches. The settings example's `open` and `reset` methods are complete
reference implementations. These APIs require a graphical desktop portal on
Linux.

## Custom chart or visualization

No chart component exists. For a small visualization, use bounded application
data and raw GPUI `div()` styling or a custom GPUI element. Keep the collector
and transformation logic separately testable, as in `system-monitor`.

```rust
let chart = clay::raw::div()
    .id("chart")
    .flex()
    .items_end()
    .children(points.iter().map(|point| {
        clay::raw::div().w(px(4.)).h(px(point.height)).bg(theme.colors.accent)
    }));
```

The exact painting and scaling are raw GPUI work; validate the pinned GPUI API
before expanding this pattern.

## Multi-window applications

`App::window` can be called more than once while constructing the application,
but the framework does not provide coordinated multi-window state or routing.
Use shared GPUI entities only after understanding their ownership and lifecycle.
Treat multi-window behavior as unverified unless tested natively.
