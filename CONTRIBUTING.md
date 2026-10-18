# Contributing

This is an early GPUI framework prototype. Keep contributions small and tied to
a working application or a documented API problem.

## Setup

Use the sibling Zed checkout only for source inspection; the workspace pins GPUI
to a Git revision. Run the checks from `gpui-framework/`:

```sh
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

For UI changes, run `cargo run --locked -p settings-app` in a graphical Linux
session and test pointer, keyboard, focus, clipboard, theme, dialog, and close
behavior when relevant.

## Change rules

- Verify every GPUI call against the pinned source or a compiled example.
- Keep platform-specific code in service or platform modules.
- Keep the core dependency list small.
- Add a behavior test for public API changes.
- Add or update an example when the public workflow changes.
- Use semantic theme roles and spacing tokens; do not hard-code component colors.
- Do not claim Windows or macOS support without native checks on those systems.
- Do not add a JavaScript runtime, visual editor, plugin marketplace, or updater
  to the MVP.

Use conventional commits for local history: `feat:`, `fix:`, `docs:`, `test:`,
`refactor:`, or `chore:`. Keep commits focused.
