# Clay application-agent guide

This directory is for agents **using** Clay to build applications.
[`AGENTS.md`](../../AGENTS.md) is for agents changing the framework itself.

These documents describe the public API at Clay package version `0.3.0` as
implemented in `crates/clay`. The repository's runnable reference
applications are:

- [`examples/settings-app`](../../examples/settings-app): settings form,
  themes, text input, clipboard, dialogs, persistence, and background I/O.
- [`examples/system-monitor`](../../examples/system-monitor): background
  collection, repeated updates, bounded history, process rows, and loading/error
  states.
- [`templates/minimal`](../../templates/minimal): generated counter project.

## Recommended reading order

1. [`getting-started.md`](getting-started.md) — install, generate, run, and build.
2. [`api-reference.md`](api-reference.md) — verified exports and signatures.
3. [`component-guide.md`](component-guide.md) — layout, controls, themes, and input.
4. [`state-and-events.md`](state-and-events.md) — entities, listeners, and tasks.
5. [`native-services.md`](native-services.md) — clipboard, dialogs, and settings.
6. [`recipes.md`](recipes.md) — complete patterns and reference applications.
7. [`troubleshooting.md`](troubleshooting.md) — compiler and native build failures.
8. [`limitations.md`](limitations.md) — what is intentionally not supported yet.

## Application-agent workflow

1. Inspect the installed framework version and this documentation.
2. Break the request into screens, state, events, persistence, and native services.
3. Generate a standalone project with `clay new`.
4. Implement with verified public exports first.
5. Inspect the installed source or reference examples before using an unfamiliar
   method. Never invent a framework API.
6. Use `clay::raw` only when the public facade is insufficient.
7. Run `cargo check --locked` and `cargo build --release --locked` in the app.
8. Report implemented behavior, exact checks, platform assumptions, and gaps.

Application code belongs in the generated application. Do not modify the
framework checkout to work around an application-specific problem unless the
problem is a reusable framework gap with an agreed design.

## Source-of-truth rule

If this guide and the installed crate disagree, trust the installed crate's
public exports, compiler output, and examples. Record the version and report a
documentation issue rather than guessing.
