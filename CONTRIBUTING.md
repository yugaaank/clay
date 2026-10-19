# Contributing

Clay is an early Linux-focused Rust/GPUI prototype. Contributions are
welcome, but keep changes small, evidence-based, and tied to a working
application, a reproducible bug, or a clearly documented API problem.

## Before you start

1. Find an issue labelled `good first issue` or `help wanted`.
2. Comment that you would like to work on it and wait for confirmation before
   starting substantial work. This prevents duplicated effort and conflicting
   designs.
3. For a new feature, explain the user problem and proposed behavior before
   writing code. Architectural proposals are not `good first issue` tasks until
   their scope and acceptance criteria are clear.

## Fork and branch

Fork the repository on GitHub, clone your fork, and work on a focused branch:

```sh
git clone https://github.com/<your-user>/clay.git
cd clay
git switch -c fix/clipboard-error
```

Use a descriptive branch such as `fix/clipboard-error`, `docs/getting-started`,
or `feat/list-selection`. Keep unrelated formatting or cleanup out of the
branch.

## Development setup

The repository pins Rust `1.99.0` in `rust-toolchain.toml`. Linux is the verified
development platform. Install the Rust toolchain, a C/C++ compiler, `pkg-config`,
and the Linux GPUI dependencies described in
[`docs/getting-started.md`](docs/getting-started.md).

The GPUI dependency is pinned to a Git revision. The first build therefore needs
network access and can take a while. Use the checked-in lockfile; do not update
dependencies unless the issue specifically concerns dependency maintenance.

## Required checks

Run these commands from the repository root before opening a pull request:

```sh
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo build --workspace --release --locked
```

For UI changes, run the relevant example in a graphical Linux session. For
example:

```sh
cargo run --locked -p settings-app
cargo run --locked -p system-monitor
```

Exercise the behavior affected by the change, including pointer, keyboard,
focus, clipboard, theme, dialog, error, and close behavior when relevant.

Windows and macOS need native validation. Do not require contributors to pass a
platform test that the repository CI cannot run, and do not claim cross-platform
support from a Linux cross-compilation result alone.

## Implementation rules

- Inspect the relevant source and tests before editing.
- Verify GPUI calls against the pinned GPUI source or a compiled example.
- Prefer the existing public framework API before adding a raw GPUI dependency.
- Keep platform-specific code in service or platform modules.
- Keep the core dependency list small and justify new dependencies.
- Add a behavior test for public API and CLI changes.
- Add or update an example when the public workflow changes.
- Use semantic theme roles and spacing tokens; do not hard-code component colors.
- Preserve the `clay::raw` escape hatch.
- Avoid unrelated features, broad rewrites, and speculative abstractions.
- Do not add a JavaScript runtime, visual editor, plugin marketplace, or updater
  to the current MVP without an accepted design issue.

## Pull requests

Push your branch to your fork and open a pull request against `main`. Include:

- A short summary of the user-visible change.
- `Closes #123` or `Related to #123` when applicable.
- The exact checks you ran and any checks you could not run.
- Screenshots or a short recording for visual changes when useful.
- Public API, compatibility, dependency, or platform impact.
- Remaining limitations and follow-up work.

Respond to review feedback by updating the same pull request. The maintainer
will review and merge the change when it meets the acceptance criteria; a pull
request using `Closes #123` will let GitHub close the linked issue after merge.

## Commits

Use focused conventional commit messages where practical:
`feat:`, `fix:`, `docs:`, `test:`, `refactor:`, or `chore:`. Do not commit,
push, tag, publish, or change repository settings on behalf of the maintainer
unless the maintainer explicitly authorizes that action.

## Proposing work

Use a bug report for reproducible broken behavior and a feature request for a
user problem that needs a new capability. Include scope, non-goals, acceptance
criteria, required tests, and whether a public API change is acceptable. Usage
questions belong in the support path described in [`SUPPORT.md`](SUPPORT.md).
