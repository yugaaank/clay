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
- A complete settings example and a generated counter template.

## Quick start

Requirements: Rust 1.99.0, a C/C++ toolchain, Linux GPUI graphics/windowing
libraries, and a working graphical session. The first build downloads pinned
GPUI and platform dependencies.

```sh
cargo run --locked -p settings-app
```

Build the CLI and create an app:

```sh
cargo build --locked -p gpui-framework-cli
./target/debug/gpui-framework new ./my-app
cd my-app
../target/debug/gpui-framework dev
../target/debug/gpui-framework build
```

The generated app uses the framework checkout as a path dependency and contains
its own Cargo workspace and lockfile. It refuses to overwrite an existing path.

Full setup, API examples, platform notes, and limitations are in
[`docs/getting-started.md`](docs/getting-started.md).

## Workspace

```text
crates/framework/       public facade and components
crates/framework-cli/   project generation and Cargo commands
examples/settings-app/  persisted settings example
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

The verified Linux run includes 18 framework tests, 21 CLI/example tests, 2
doctests, a release build, a native settings window, and a generated app build
and launch. The workspace was tested against GPUI revision
`56934ec0560bce8929025497411de7f5322fb6ca`.

## License

Apache-2.0. See [`LICENSE`](LICENSE). GPUI and the platform crates are also
Apache-2.0 at the pinned revision. Dependency licenses remain the responsibility
of applications when they redistribute a built product.
