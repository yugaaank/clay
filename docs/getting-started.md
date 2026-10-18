# Getting started

This prototype currently targets Linux. It uses GPUI revision
`56934ec0560bce8929025497411de7f5322fb6ca` from Zed and enables Wayland and X11.
Windows and macOS remain isolated future targets; they are not verified by this
release.

## Prerequisites

- Rust 1.99.0 or newer
- A C/C++ toolchain
- Linux Vulkan, Wayland/X11, xkbcommon, font, and file-chooser portal packages
- Network access for the first dependency resolution

The framework uses Git dependencies pinned to the GPUI source revision, so builds
are reproducible with the checked-in `Cargo.lock` files after dependencies have
been downloaded.

## Run the example

```sh
cargo run --locked -p settings-app
```

The example demonstrates:

- `App::new().theme(...).window(...).run()`
- `Column`, `Row`, `Container`, `Text`, `Button`, and `Checkbox`
- `TextInput` with native clipboard and IME handling
- light/dark theme tokens
- JSON settings loaded and saved on a background executor
- native file selection and confirmation dialogs

The settings file is written to `XDG_CONFIG_HOME/gpui-framework-example` on
Linux, or the platform's application configuration directory.

## Generate an app

Build the CLI once:

```sh
cargo build --locked -p gpui-framework-cli
```

Generate a project. The generated project is standalone and refuses to reuse an
existing path:

```sh
./target/debug/gpui-framework new ./my-app
cd my-app
../target/debug/gpui-framework dev
../target/debug/gpui-framework build
```

The generated `Cargo.toml` points at this checkout's framework crate. For a
different checkout:

```sh
gpui-framework new ./my-app --framework-path /path/to/gpui-framework/crates/framework
```

`dev` runs `cargo run --locked`; `build` runs `cargo build --release --locked`.
Arguments after `--` are passed to the generated application:

```sh
gpui-framework dev -- --some-app-argument
```

## Public API shape

```rust
use gpui_framework::prelude::*;

struct Counter { value: i32 }

impl Render for Counter {
    fn render(&mut self, _: &mut UiWindow, cx: &mut Context<Self>) -> impl IntoElement {
        Column::new()
            .padding(Spacing::Lg)
            .child(Text::new(format!("Count: {}", self.value)))
            .child(Button::new("increment", "Increment").on_click(
                cx.listener(|this, _, _, cx| {
                    this.value += 1;
                    cx.notify();
                }),
            ))
    }
}

fn main() -> Result<(), SystemError> {
    App::new()
        .theme(ThemeMode::Dark)
        .window(Window::new("Counter").size(900., 600.), |window, cx| {
            let _ = window;
            Counter { value: 0 }
        })
        .run()
}
```

Advanced users can use `gpui_framework::raw` for the pinned GPUI APIs. That
module is intentionally exposed so the facade does not block custom elements,
custom painting, lower-level event handling, or GPUI task execution.

## Checks

```sh
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo build --workspace --release --locked
```

## Known limits

- Linux is the only platform tested in this prototype.
- File dialogs depend on the host desktop portal.
- `SettingsStore` is a small JSON file store, not a migration framework.
- Settings I/O is moved to the background executor by the example; callers must
  not perform synchronous I/O in render.
- No hot reload, packaging, updater, plugin system, TypeScript runtime, or visual
  editor is included.
- The GPUI dependency is pre-1.0 and must be upgraded deliberately.
