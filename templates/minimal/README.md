# {{package_name}}

A small Clay counter application. Click **Increment** to update the
count; use Tab to move keyboard focus. The framework supplies the theme, fonts,
and application root. UI-independent state and its tests live in `src/counter.rs`.

## Develop and build

Use Rust 1.99 or newer and install the native platform dependencies required by
GPUI. For independent project generation, install the CLI from the published
framework release:

```sh
cargo install --locked \
  --git https://github.com/yugaaank/clay.git \
  --tag v0.3.0 \
  clay-cli \
  --bin clay
clay new ./my-app
```

By default, the generated `Cargo.toml` uses the public Git source at tag
`v0.3.0`, so the tag must be published before this flow can resolve
dependencies. This is the first Clay release after the project rename.

For unreleased local framework development, generate with an explicit local
override instead:

```sh
clay new ./my-app --framework-path /path/to/clay/crates/clay
```

That mode records the canonical local path in `Cargo.toml` and is only intended
for development against an unpublished checkout.

Run these commands from this application's directory:

```sh
clay dev
clay build
cargo test --locked
```

`dev` invokes `cargo run --locked`; `build` invokes
`cargo build --release --locked`. The release executable is placed in
`target/release/` unless a Cargo target directory is configured.

From another directory, pass the manifest as a single argument (quote paths
containing spaces):

```sh
clay dev --manifest-path "/path to app/Cargo.toml"
clay build --manifest-path "/path to app/Cargo.toml"
```

Application arguments go after `--`, for example `clay dev -- --help`.
The minimal counter does not interpret arguments yet.

## Lockfile and dependencies

`clay new` automatically runs `cargo generate-lockfile`. Commit the
resulting `Cargo.lock` with your application; development and release builds use
it without updating dependencies. First-time resolution needs access to the
configured registries and the public framework Git tag (or the local framework
checkout when `--framework-path` was used).

If resolution failed during creation, all generated files are retained. Fix the
reported Cargo error, then retry from this directory:

```sh
cargo generate-lockfile
clay dev
```

The root `[workspace]` keeps this application independent of any enclosing
workspace. Keep the pinned `async-task` and `calloop` patches in this root
manifest: Cargo does not inherit patches from a path dependency's workspace.
