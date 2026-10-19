# Current limitations

## Platform

- Linux with Wayland/X11 is the only runtime-verified platform.
- Windows cross-target checking reaches GPUI, but release shader compilation
  needs the Windows SDK and `fxc.exe`; native interaction is unverified.
- macOS cross-target checking needs Xcode's `xcrun metal` and `metallib`; native
  interaction is unverified.
- File dialogs require a host desktop portal and backend.

## UI surface

The public facade currently includes `Column`, `Row`, `Container`, `Text`,
`Button`, `Checkbox`, and single-line `TextInput`. It does not include:

- tables or virtualized lists
- selection models, sorting, or filtering helpers
- menus, tabs, navigation, trees, or modal components
- form schemas or validation
- built-in charting or a declarative markup language

Use raw GPUI for custom work, and treat that as application code unless the
pattern is reusable across multiple applications.

## Runtime and release

- GPUI is pre-1.0 and pinned to a Zed Git revision.
- The CLI generates locked Cargo projects but does not package installers,
  AppImages, `.deb` files, Windows installers, or macOS bundles.
- There is no hot reload, updater, plugin system, TypeScript runtime, or visual
  editor.
- Background tasks have no framework-level cancellation handle; applications
  must manage stale results and cancellation policy.
- `SettingsStore` is a small JSON store, not a migration framework or secure
  secret store.

## Agent guidance

Agents must report these limits instead of silently adding raw workarounds or
claiming unsupported behavior. A recurring workaround may become a framework
issue only after a concrete application demonstrates that it is reusable.
