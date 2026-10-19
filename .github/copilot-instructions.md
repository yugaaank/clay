# Copilot instructions

Read the repository-root [`AGENTS.md`](../AGENTS.md) before editing. It is the
canonical source for architecture, commands, platform status, API compatibility,
template behavior, and safe Git workflow.

Keep changes focused, inspect existing tests first, prefer the framework API over
new dependencies or raw GPUI, and add behavior tests for public changes. Run the
locked workspace checks that apply. Treat Linux as the only verified platform;
do not infer native Windows/macOS support from cross-compilation. Do not commit,
push, tag, publish, or change repository settings unless the user explicitly
authorizes it.
