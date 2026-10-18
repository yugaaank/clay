# {{package_name}}

A small GPUI Framework counter application. Click **Increment** to update the
count; use Tab to move keyboard focus. The framework supplies the theme, fonts,
and application root. UI-independent state and its tests live in `src/counter.rs`.

## Develop and build

Use Rust 1.99 or newer and install the native platform dependencies required by
GPUI. Keep the local framework source checkout available at the absolute path in
`Cargo.toml` (update that dependency path if you move the checkout).

Run these commands from this application's directory:

```sh
gpui-framework dev
gpui-framework build
cargo test --locked
```

`dev` invokes `cargo run --locked`; `build` invokes
`cargo build --release --locked`. The release executable is placed in
`target/release/` unless a Cargo target directory is configured.

From another directory, pass the manifest as a single argument (quote paths
containing spaces):

```sh
gpui-framework dev --manifest-path "/path to app/Cargo.toml"
gpui-framework build --manifest-path "/path to app/Cargo.toml"
```

Application arguments go after `--`, for example `gpui-framework dev -- --help`.
The minimal counter does not interpret arguments yet.

## Lockfile and dependencies

`gpui-framework new` automatically runs `cargo generate-lockfile`. Commit the
resulting `Cargo.lock` with your application; development and release builds use
it without updating dependencies. First-time resolution needs access to the
configured registries and upstream Git repositories.

If resolution failed during creation, all generated files are retained. Fix the
reported Cargo error, then retry from this directory:

```sh
cargo generate-lockfile
gpui-framework dev
```

The root `[workspace]` keeps this application independent of any enclosing
workspace. Keep the pinned `async-task` and `calloop` patches in this root
manifest: Cargo does not inherit patches from a path dependency's workspace.
