# Application troubleshooting

## First diagnostic steps

Record the framework version, Rust version, OS, command, and complete compiler
error. From the application directory run:

```sh
rustc --version --verbose
cargo tree --locked -p clay
cargo check --locked
```

Inspect the installed crate source or the repository's `crates/clay` and
examples before guessing a method name. Compiler errors are usually more useful
than changing random GPUI types.

## `cargo: command not found` or wrong Rust version

Install Rust with rustup, then verify the pinned toolchain:

```sh
rustup toolchain install 1.99.0 --profile minimal
cargo +1.99.0 --version
```

Run commands from the application directory. A generated application has its
own workspace and lockfile; do not accidentally run commands from a parent
workspace.

## Public tag or Git dependency cannot be found

The generated public manifest points at the release tag encoded by the CLI. Check
it directly:

```sh
git ls-remote --tags https://github.com/yugaaank/clay.git refs/tags/v0.3.0
```

If the tag is unavailable, install/use a published CLI version or generate from
the framework checkout with `--framework-path`. Do not replace the dependency
with an arbitrary branch for a reproducible application.

## Fontconfig, XKB, Wayland, or Vulkan linker errors

Install the Linux development packages from `getting-started.md`, especially:
`libfontconfig1-dev`, `libxkbcommon-dev`, `libxkbcommon-x11-dev`,
`libwayland-dev`, `libx11-xcb-dev`, and `libvulkan-dev`. The workspace enables
both X11 and Wayland support.

## Window cannot open

The examples need a graphical session. Check `DISPLAY` or `WAYLAND_DISPLAY` and
run them on a desktop session rather than a plain headless SSH shell. A virtual
display may be needed for automation, but that does not replace native
interaction testing.

## File dialog fails

Install and start a desktop portal and an appropriate backend such as
`xdg-desktop-portal-gtk` or `xdg-desktop-portal-kde`. The framework delegates
file selection to the host portal.

## Method or trait not found

Check whether the name belongs to the framework facade or raw GPUI. Import the
framework prelude first:

```rust
use clay::prelude::*;
```

Then inspect `crates/clay/src/lib.rs` and the module defining the type. For
advanced methods, inspect the pinned GPUI source referenced by `Cargo.lock`. Do
not invent a method based on APIs from another UI framework.

## UI does not refresh

After mutating entity state in a listener or task, call `cx.notify()`. When a
child entity changes, subscribe to its events or update it through its `Entity`.
Do not perform state mutation during `render`.

## Background work blocks the UI

Move synchronous I/O, process collection, and CPU-heavy work into
`cx.background_spawn`. Await the task from `cx.spawn_in` and apply the result with
`update`/`update_in`. See `state-and-events.md`.

## Generated template manifest errors

Use the `0.3.0` Clay CLI or newer. The raw file is named
`templates/minimal/Cargo.toml.template`; the CLI substitutes placeholders before
writing the application's real `Cargo.toml`.
