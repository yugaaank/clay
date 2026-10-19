# Claude Code instructions

Use [`AGENTS.md`](AGENTS.md) as the canonical project guidance before making
changes. Follow [`CONTRIBUTING.md`](CONTRIBUTING.md) for the human contribution
workflow.

In particular:

- Inspect the relevant code, tests, docs, and Git status before editing.
- Keep changes focused on the requested task and preserve the public GPUI escape
  hatch.
- Use the pinned Rust/GPUI toolchain and run the applicable locked checks.
- Linux is verified; do not claim Windows or macOS support without native tests.
- Report changed files, exact checks, failures, and remaining limitations.
- Do not commit, push, tag, publish, or change GitHub settings unless the user
  explicitly asks for that action.
