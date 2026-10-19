# GPUI Framework

A small Rust application framework built on GPUI. It removes routine application
bootstrap and styling code while keeping an explicit escape hatch to the pinned
GPUI APIs.

This is an early Linux prototype. It is not a promise of Windows or macOS
support; those targets need native builds and interaction checks before they can
be advertised.

## What works

- `App`, `Window`, `Column`, `Row`, `Container`, `Text`, `Button`, `Checkbox`,
  and native single-line `TextInput`.
- GPUI entity state, listeners, focus, keyboard traversal, clipboard, IME input,
  accessible names/roles, and semantic light/dark theme tokens.
- JSON settings persistence on the GPUI background executor.
- Native file selection and confirmation dialogs where the host platform provides
  them.
- A CLI with `new`, `dev`, and `build`.
- A complete settings example, a live system-monitor example, and a generated
  counter template.

## Quick start

Requirements: Rust 1.99.0, a C/C++ toolchain, Linux GPUI graphics/windowing
libraries, and a working graphical session. The first build downloads pinned
GPUI and platform dependencies.

```sh
cargo run --locked -p settings-app
```

Run the system-monitor example:

```sh
cargo run --locked -p system-monitor
```

It collects CPU, memory, uptime, and process data off the render path, keeps a
bounded 60-sample history and 50-process view, and displays explicit loading,
error, and empty states.

Build the CLI and create an app:

```sh
cargo build --locked -p gpui-framework-cli
./target/debug/gpui-framework new ./my-app --framework-path "$PWD/crates/framework"
cd my-app
../target/debug/gpui-framework dev
../target/debug/gpui-framework build
```

The `--framework-path` form is for unreleased development against this checkout.
The generated app contains its own Cargo workspace and lockfile, and the CLI
refuses to overwrite an existing path.

For independent generation from the public release, install the CLI from the
published tag and omit `--framework-path`:

```sh
cargo install --locked \
  --git https://github.com/yugaaank/gpui-framework.git \
  --tag v0.2.1 \
  gpui-framework-cli \
  --bin gpui-framework
gpui-framework new ./my-app
```

The default generated dependency is the public Git source at tag `v0.2.1`.
That maintenance-release tag must be published before independent generation
can resolve it; local paths are only for unreleased development.

Full setup, API examples, platform notes, and limitations are in
[`docs/getting-started.md`](docs/getting-started.md).

## Workspace

```text
crates/framework/       public facade and components
crates/framework-cli/   project generation and Cargo commands
examples/settings-app/  persisted settings example
examples/system-monitor/ bounded live metrics and process example
templates/minimal/      generated counter project
docs/getting-started.md setup and API guide
```

`gpui_framework::raw` exposes GPUI for custom views, elements, painting, tasks,
and platform-specific work. The facade does not link Zed's GPL `ui` crate; it
uses the Apache-2.0 GPUI crates directly and defines its own semantic tokens.

## Verification

```sh
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo build --workspace --release --locked
```

The verified Linux run includes 18 framework tests, 28 CLI/example tests, 2
doctests, a release build, a native settings window, and a system-monitor build.
The workspace was tested against GPUI revision
`56934ec0560bce8929025497411de7f5322fb6ca`.

## License

Apache-2.0. See [`LICENSE`](LICENSE). GPUI and the platform crates are also
Apache-2.0 at the pinned revision. Dependency licenses remain the responsibility
of applications when they redistribute a built product.
