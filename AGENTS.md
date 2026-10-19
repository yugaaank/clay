# Clay agent guide

This file is the canonical engineering guidance for coding agents working in
this repository. Read it before inspecting or changing code. Human contribution
workflow is in [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Project purpose and maturity

Clay is a small Rust application layer over GPUI. It provides
application/window bootstrap, common layout and controls, semantic themes,
native services, and a project-generation CLI while exposing
`clay::raw` as an escape hatch to pinned GPUI APIs.

The product direction may explore React-like component composition and state
ergonomics in Rust. The current API is builder-style Rust, not a React runtime or
declarative TypeScript layer.

It is an early-stage, Linux-focused prototype. Linux is the verified platform.
Windows and macOS are not native build, runtime, or interaction-tested release
targets. Never turn a cross-target check into a claim of platform support.

The current intended audience is Rust developers building small native desktop
tools. Do not silently expand the project into a TypeScript runtime, visual
editor, plugin marketplace, updater, or broad web-style component platform.

## Workspace map

```text
crates/clay/             public facade and framework implementation
crates/clay-cli/         clay new/dev/build commands
examples/settings-app/  persisted settings and native-service reference app
examples/system-monitor/ background metrics and bounded-list reference app
templates/minimal/      embedded generated counter project files
docs/                   user and setup documentation
.github/workflows/      Linux CI checks
```

Important source locations:

- `crates/clay/src/lib.rs`: public exports, `prelude`, and `raw` escape hatch.
- `crates/clay/src/app.rs`: `App`, `Window`, lifecycle, focus traversal,
  and application tests.
- `crates/clay/src/components.rs`: `Column`, `Row`, `Container`, `Text`,
  `Button`, and `Checkbox`.
- `crates/clay/src/input.rs`: native single-line `TextInput`, Unicode,
  IME, selection, clipboard, and input events.
- `crates/clay/src/services.rs`: `Clipboard`, `Dialog`, `SettingsStore`,
  and `SystemError`.
- `crates/clay/src/theme.rs`: semantic colors, spacing, light/dark modes.
- `crates/clay-cli/src/args.rs`: CLI argument parsing and help text.
- `crates/clay-cli/src/project.rs`: safe generation, source selection,
  template substitution, and public Git dependency constants.
- `crates/clay-cli/src/cargo.rs`: direct Cargo child-process commands.
- `templates/minimal/Cargo.toml.template`: raw manifest template. Do not rename
  it back to `Cargo.toml`; Cargo must not parse unexpanded placeholders.
- `examples/settings-app/src/main.rs`: reference for forms, persistence,
  dialogs, actions, and background UI updates.
- `examples/system-monitor/src/collector.rs`: platform-neutral collector tests
  and bounded history/process ordering.

## Toolchain and dependencies

- Rust is pinned to `1.99.0` in `rust-toolchain.toml`.
- The workspace uses edition 2024 and a checked-in `Cargo.lock`.
- GPUI and `gpui_platform` are pinned to Zed revision
  `56934ec0560bce8929025497411de7f5322fb6ca`.
- The workspace directly uses Apache-2.0 GPUI crates and deliberately does not
  link Zed's GPL `ui`, settings, or theme crates.
- Linux builds need the native packages listed in
  `docs/getting-started.md`; the repository CI currently installs the core
  graphics packages. File dialogs also depend on a desktop portal.
- Keep dependencies minimal. Do not update the GPUI revision or lockfile
  opportunistically.

## Required checks

Run from the repository root. Use the pinned toolchain selected by
`rust-toolchain.toml`:

```sh
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo build --workspace --release --locked
```

For a framework or public-API change, add or update behavior tests. For a UI
change, run the affected example in a graphical Linux session. If a check cannot
run because the environment lacks a display, native SDK, or platform, report the
exact command and blocker instead of calling it passed.

## Architecture and API rules

- Prefer the framework prelude and existing components before using raw GPUI.
- Preserve `clay::raw`; advanced users must retain a GPUI escape hatch.
- Use GPUI entity state, listeners, subscriptions, and `cx.notify()` consistently.
- Keep synchronous file I/O out of `render`; use `cx.background_spawn` and apply
  results through a window/entity task.
- Use semantic theme roles and `Spacing`; do not introduce hard-coded component
  palette values.
- Use stable element IDs for interactive controls.
- Verify GPUI APIs against the pinned source or a compiled example before coding.
- Keep platform-specific behavior in service/platform boundaries.
- Avoid breaking public API changes. If one is necessary, document the migration
  and compatibility impact in the change description.

## CLI and template rules

`clay new` refuses existing destinations, validates package names,
embeds template files, and resolves a lockfile with a direct Cargo child process.
The default dependency is the public Git framework at the release tag encoded in
`crates/clay-cli/src/project.rs`. `--framework-path` is an explicit local
development override.

Never place shell syntax, unescaped paths, or an unexpanded template manifest in
generated output. Preserve argument boundaries and no-clobber behavior. Test
invalid paths, existing destinations, public dependency substitution, lockfile
failure, and forwarded application arguments when changing the CLI.

## Examples and known limits

Use `settings-app` for common forms, themes, persistence, clipboard, dialogs,
focus, and background work. Use `system-monitor` for repeated updates, bounded
history, process rows, and loading/error/empty states.

The framework does not yet provide tables, virtualized lists, form validation,
packaging, hot reload, plugins, updater support, or a TypeScript layer. Do not
add these because a competitor has them; require a focused product/design issue.

## Safe agent workflow

1. Inspect current Git status, relevant source, tests, docs, and CI.
2. State the user-visible result and acceptance criteria.
3. Make the smallest complete change in the intended files.
4. Add behavior tests or documentation evidence.
5. Run the smallest meaningful checks, then the full applicable checks.
6. Inspect the complete diff and report changed files, commands, failures, and
   remaining risks.

Do not modify unrelated files or add dependencies without a clear need. Do not
claim a capability, platform, benchmark, or release flow that was not verified.
Do not commit, push, tag, publish, open pull requests, create issues, change
labels, enable Discussions, or change GitHub settings unless the user explicitly
authorizes that action.
