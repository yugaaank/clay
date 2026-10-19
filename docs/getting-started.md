# Getting started with Clay

Clay is a Rust application layer over GPUI. It currently targets Linux
and uses GPUI revision `56934ec0560bce8929025497411de7f5322fb6ca` from Zed.
Windows and macOS are not native-tested release targets yet.

This guide assumes basic Rust and Cargo knowledge. You will learn how to install
the native dependencies, run the examples, generate an application, build a
screen, persist settings, run background work, and create a release binary.

## 1. Install prerequisites

You need:

- Rust `1.99.0` or newer. The repository pins `1.99.0` in
  `rust-toolchain.toml`.
- `rustfmt` and `clippy` for the repository checks.
- A C/C++ compiler and `pkg-config`.
- A Linux graphical session using Wayland or X11.
- Network access for the first download of GPUI's pinned Git dependencies.

On Debian or Ubuntu, install the native libraries used by the current Linux
build:

```sh
sudo apt-get update
sudo apt-get install -y \
  build-essential pkg-config \
  libfontconfig1-dev \
  libxkbcommon-dev libxkbcommon-x11-dev \
  libwayland-dev libx11-xcb-dev \
  libvulkan-dev \
  xdg-desktop-portal
```

Your desktop may also need a portal backend such as
`xdg-desktop-portal-gtk` or `xdg-desktop-portal-kde` for file dialogs.

Install Rust with rustup if it is not already available:

```sh
rustup toolchain install 1.99.0 --profile minimal
rustup component add rustfmt clippy --toolchain 1.99.0
```

The first Cargo command downloads and compiles a large native dependency graph.
That initial build is slow; later builds use Cargo's cache.

## 2. Run the examples

From a checkout of this repository:

```sh
cargo run --locked -p settings-app
```

The settings example demonstrates:

- `App::new().theme(...).window(...).run()`
- `Column`, `Row`, `Container`, `Text`, `Button`, and `Checkbox`
- native single-line `TextInput` with clipboard and IME handling
- light/dark semantic theme tokens
- JSON settings loaded and saved on a background executor
- native file selection and confirmation dialogs

The settings file is stored below `XDG_CONFIG_HOME` when that variable is an
absolute path, otherwise below `~/.config/clay-example` on Linux.

The system-monitor example exercises background work and repeated UI updates:

```sh
cargo run --locked -p system-monitor
```

It refreshes CPU, memory, uptime, and process metrics once per second without
blocking render. Its collector keeps 60 history samples and 50 process rows and
has tests for bounds, sorting, and deterministic ordering.

## 3. Install the public CLI

When the `v0.3.0` tag is available on GitHub, install the CLI with the explicit
crate and binary names:

```sh
cargo install --locked \
  --git https://github.com/yugaaank/clay.git \
  --tag v0.3.0 \
  clay-cli \
  --bin clay

clay --version
```

The expected output is similar to:

```text
clay 0.3.0
```

If Cargo says that `v0.3.0` cannot be found, the release tag has not reached
GitHub yet. For local framework development, use the checkout flow below.

## 4. Generate your first application

Create a new destination that does not already exist:

```sh
clay new ./my-app
cd my-app
clay dev
```

The CLI writes:

```text
my-app/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── .gitignore
└── src/
    ├── main.rs
    └── counter.rs
```

`src/main.rs` creates the window and renders the counter. `src/counter.rs`
contains UI-independent state and tests. The generated project is its own Cargo
workspace, so it can live inside another repository without inheriting that
repository's members or patches.

The `new` command also runs `cargo generate-lockfile`. Commit the generated
`Cargo.lock` with your application. If dependency resolution fails, the CLI
keeps the generated files; fix the reported problem and run:

```sh
cargo generate-lockfile
```

For unreleased development against a local framework checkout:

```sh
cargo build --locked -p clay-cli
./target/debug/clay new ./my-app \
  --framework-path "$PWD/crates/clay"
```

That option records a canonical local path in the generated manifest. It is for
development only and is not suitable for sharing the generated project.

## 5. Run and build the application

From the generated application's directory:

```sh
clay dev
clay build
```

These commands are thin wrappers around:

```sh
cargo run --locked
cargo build --release --locked
```

The release binary is normally at `target/release/<package-name>`. You can work
from another directory by passing a manifest path as one argument:

```sh
clay dev --manifest-path "/path with spaces/my-app/Cargo.toml"
clay build --manifest-path "/path with spaces/my-app/Cargo.toml"
```

Arguments after `--` go to the application rather than the CLI:

```sh
clay dev -- --example-argument
```

## 6. Build a screen

Import the framework prelude and implement `Render` for an application state
type. Components are builder-style values, so compose the UI by adding children:

```rust
use clay::prelude::*;

struct Counter {
    value: i32,
}

impl Render for Counter {
    fn render(&mut self, _: &mut UiWindow, cx: &mut Context<Self>) -> impl IntoElement {
        Column::new()
            .padding(Spacing::Lg)
            .gap(Spacing::Md)
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
        .window(Window::new("Counter").size(900., 600.), |_, _| Counter { value: 0 })
        .run()
}
```

The important concepts are:

- `Render::render` returns a new element tree for the current state.
- `Context<Self>` provides listeners and entity notifications.
- Mutate state in an event handler, then call `cx.notify()` to render again.
- Use stable IDs for interactive elements such as `Button::new("save", ...)`.
- Use `Theme::get(cx)` and semantic colors instead of hard-coded palette values.

For a header/sidebar/content layout, use a full-size `Row` with a fixed-width
`Container` for the sidebar and a flexible child for the main content. The
settings example is the canonical layout reference.

## 7. Add controls and forms

The current controls are intentionally small. Typical usage looks like this:

```rust
let name = cx.new(|cx| {
    TextInput::new("", window, cx)
        .label("Display name")
        .placeholder("Your name")
});

let form = Column::new()
    .gap(Spacing::Md)
    .child(name)
    .child(Checkbox::new("enabled", "Enabled", enabled))
    .child(Button::new("submit", "Save").primary());
```

Subscribe to `TextInput` changes when the application state needs to mirror the
input. Validation and submit-button enablement are application code today; there
is no framework form model or validation schema yet. The settings example shows
how to disable controls while an operation is running.

`TextInput` handles Unicode scalar boundaries, grapheme-aware keyboard movement,
UTF-16 selection for the native GPUI input bridge, clipboard actions, and IME
composition. It is single-line only.

## 8. Persist settings safely

`SettingsStore` stores JSON at a platform configuration directory. It accepts a
serializable value and uses a temporary file plus rename when saving:

```rust
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Preferences {
    dark: bool,
}

let store = SettingsStore::new("my-app")?;
let task = cx.background_spawn(async move { store.load::<Preferences>() });
```

Call `load` and `save` from `cx.background_spawn`; they are synchronous file
operations and must not run inside `render`. Missing files produce `Default`.
Malformed JSON returns an error and is not silently replaced.

The settings example uses `cx.spawn_in` to apply the result back to the UI,
updates a busy state, and reports errors in the view. Copy that lifecycle pattern
for application I/O rather than blocking the render path.

## 9. Run background work

GPUI provides the task primitives through the prelude. The normal pattern is:

1. Capture owned input for the task.
2. Start work with `cx.background_spawn(...)`.
3. Await it from `cx.spawn_in(window, ...)`.
4. Use `this.update_in(...)` or `this.update(...)` to apply the result.
5. Call `cx.notify()` after changing visible state.

The system monitor uses this pattern and keeps its collector off the UI thread.
Detached tasks stop applying results when the entity no longer exists, but the
framework does not yet provide a higher-level cancellation handle. Long-running
application tasks should define their own cancellation and stale-result policy.

## 10. Use native services

Clipboard operations are synchronous GPUI application calls:

```rust
Clipboard::write("copied text", cx);
let text = Clipboard::read(cx);
```

Dialogs are asynchronous and report cancellation as a normal result:

```rust
Dialog::confirm("Delete this item?", window, cx, |this, result, window, cx| {
    match result {
        Ok(true) => {
            // Mutate application state and call cx.notify().
        }
        Ok(false) => { /* The user canceled. */ }
        Err(error) => {
            // Store or display error, then call cx.notify().
            let _ = (this, window, cx, error);
        }
    }
});
```

Use `Dialog::open_file` for one file. On Linux, file dialogs depend on a working
desktop portal and backend.

## 11. Use raw GPUI when needed

Clay deliberately exposes `clay::raw`. Use it for:

- custom `div()` styling not covered by the facade
- custom elements and painting
- lower-level pointer, keyboard, and action handling
- GPUI task APIs not wrapped by the framework
- tables, virtualized lists, charts, and other components not yet provided

This means advanced applications still need GPUI knowledge. The facade reduces
bootstrap and common-control boilerplate; it does not hide GPUI's architecture.

## 12. Test and release-build

For an application, the minimum useful checks are:

```sh
cargo fmt --all -- --check
cargo check --locked
cargo test --locked
cargo build --release --locked
```

For the framework repository itself, run the full workspace checks:

```sh
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo build --workspace --release --locked
```

Clay has headless tests for stateful behavior and CLI tests for
argument boundaries, no-clobber generation, lockfile invocation, and public
dependency substitution. Native window interaction is still a platform-level
smoke test rather than a complete automated UI test suite.

## Troubleshooting

### `cargo` or the pinned toolchain is unavailable

Install Rust with rustup and make sure the command is on `PATH`:

```sh
rustup toolchain install 1.99.0 --profile minimal
cargo +1.99.0 --version
```

### Fontconfig or XKB linker errors

Install the development packages listed in the prerequisites, including
`libfontconfig1-dev`, `libxkbcommon-dev`, and `libxkbcommon-x11-dev`. The
`xkbcommon-x11` package is needed even when the application normally runs under
Wayland because the workspace currently enables both X11 and Wayland support.

### The application cannot open a window

Check that you are inside a graphical session and that `DISPLAY` or
`WAYLAND_DISPLAY` is set. A headless SSH session cannot run the native examples
without a suitable display server or virtual display.

### File selection fails or does nothing

Install and start a desktop portal service. The framework delegates file
selection to the host portal; it does not provide its own file chooser.

### The generated project says a tag cannot be found

The public CLI flow requires the Git tag referenced by the generated manifest.
Check the tag directly:

```sh
git ls-remote --tags https://github.com/yugaaank/clay.git refs/tags/v0.3.0
```

For unreleased local development, regenerate with `--framework-path` instead.

### Cargo reports an error inside `templates/minimal`

Use the `v0.3.0` CLI or newer. The raw template is stored as
`Cargo.toml.template` so Cargo does not parse its substitution placeholders as a
normal package manifest.

## Known limits

- Linux is the only platform tested in this prototype.
- Windows cross-target checking reaches GPUI, but release shader compilation
  needs the Windows SDK's `fxc.exe`; native Windows builds and interaction are
  not verified.
- macOS cross-target checking needs Xcode's `xcrun metal` and `metallib` tools;
  native macOS builds and interaction are not verified.
- File dialogs depend on the host desktop portal.
- `SettingsStore` is a small JSON file store, not a migration framework.
- Settings I/O is moved to the background executor by the example; callers must
  not perform synchronous I/O in render.
- There is no hot reload, application packaging, updater, plugin system,
  TypeScript runtime, or visual editor.
- There are no built-in tables, virtualized lists, menus, navigation, or form
  validation helpers yet.
- The system-monitor process view is intentionally capped at 50 rows and its
  CPU reading settles after the first refresh because it is delta-based.
- The GPUI dependency is pre-1.0 and must be upgraded deliberately.
