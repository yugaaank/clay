# Clay application setup

This guide targets Clay `0.3.0` and Linux. Windows and macOS are not
runtime-verified release targets.

## Prerequisites

Install Rust `1.99.0` or newer, a C/C++ compiler, `pkg-config`, and the Linux
GPUI libraries. Debian/Ubuntu users can start with:

```sh
sudo apt-get update
sudo apt-get install -y \
  build-essential pkg-config \
  libfontconfig1-dev libxkbcommon-dev libxkbcommon-x11-dev \
  libwayland-dev libx11-xcb-dev libvulkan-dev \
  xdg-desktop-portal
```

Use a desktop portal backend such as `xdg-desktop-portal-gtk` when using file
dialogs. The first build needs network access for pinned GPUI Git dependencies.

## Install the public CLI

After the `v0.3.0` tag is published:

```sh
cargo install --locked \
  --git https://github.com/yugaaank/clay.git \
  --tag v0.3.0 \
  clay-cli \
  --bin clay

clay --version
```

The explicit `clay-cli` package and `--bin clay` are
required because the repository contains multiple packages and binaries.

## Generate and run an application

```sh
clay new ./my-app
cd my-app
clay dev
```

The destination must not already exist. Generation writes `Cargo.toml`,
`Cargo.lock`, `README.md`, `.gitignore`, `src/main.rs`, and `src/counter.rs`.
The generated application is its own Cargo workspace.

Build a release binary:

```sh
clay build
```

This runs `cargo build --release --locked`; the binary is normally in
`target/release/`.

## Local framework development

When the release tag is unavailable or you are changing the framework locally:

```sh
cargo build --locked -p clay-cli
./target/debug/clay new ./my-app \
  --framework-path "$PWD/crates/clay"
```

The generated manifest records a local path. This is not a distributable setup;
use the public Git mode for applications shared with other developers.

## Application checks

Run these inside a generated application after changes:

```sh
cargo fmt --all -- --check
cargo check --locked
cargo test --locked
cargo build --release --locked
```

For application arguments, place them after `--`:

```sh
clay dev -- --example-argument
```

For a manifest outside the current directory, quote the complete path:

```sh
clay build --manifest-path "/path with spaces/my-app/Cargo.toml"
```
