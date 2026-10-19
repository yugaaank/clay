# Native services

## Clipboard

Clipboard access uses the GPUI application context:

```rust
Clipboard::write("hello", cx);

if let Some(text) = Clipboard::read(cx) {
    // Apply text to application state or an input entity.
}
```

Clipboard writes do not return a failure from GPUI. Treat reads as optional and
avoid representing clipboard content as guaranteed application data.

## File picker

`Dialog::open_file` is asynchronous and returns `Ok(None)` when the user
cancels:

```rust
Dialog::open_file(window, cx, |this, result, _, cx| {
    match result {
        Ok(Some(path)) => {
            this.selected_file = Some(path);
            cx.notify();
        }
        Ok(None) => {}
        Err(error) => {
            this.error = Some(error.to_string());
            cx.notify();
        }
    }
});
```

The current API selects one file, not directories or multiple files. On Linux it
requires a functioning desktop file-chooser portal.

## Confirmation dialog

`Dialog::confirm` is also asynchronous. Treat `Ok(false)` as a normal cancel:

```rust
Dialog::confirm("Delete this item?", window, cx, |this, result, _, cx| {
    if matches!(result, Ok(true)) {
        this.delete_requested = true;
        cx.notify();
    }
});
```

The callback receives the application entity, result, window, and context. Do not
block waiting for a dialog result.

## JSON settings

Define a `Serialize + DeserializeOwned + Default` value and use a named store:

```rust
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Preferences {
    dark: bool,
}

let store = SettingsStore::new("my-app")?;
let loaded = cx.background_spawn(async move { store.load::<Preferences>() });
```

`SettingsStore::new` accepts only ASCII letters, numbers, `-`, and `_`. Missing
files return `Default`; malformed JSON returns `SystemError::Data`. `save` uses a
temporary file and preserves existing file permissions. Both methods are
synchronous internally, so call them on the background executor.

Use `SettingsStore::from_path` for tests or an explicit application-specific
location. Do not put secrets in this JSON store without adding an appropriate
secure-storage design; it is plain user-readable data.

## System paths and platform status

The implementation uses `XDG_CONFIG_HOME` or `~/.config` on Linux, `APPDATA` on
Windows, and `~/Library/Application Support` on macOS. These platform branches
exist in source, but only Linux has been runtime-verified in this repository.
