use gpui::{Context, PathPromptOptions, PromptButton, PromptLevel, Window};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fmt, fs,
    io::Write,
    path::{Path, PathBuf},
};

/// Errors returned by setup, persistence, and native dialogs.
#[derive(Debug)]
pub enum SystemError {
    Unsupported(&'static str),
    Unavailable(String),
    InvalidInput(String),
    Backend(String),
    Io(std::io::Error),
    Data(serde_json::Error),
}

impl fmt::Display for SystemError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(feature) => write!(formatter, "unsupported feature: {feature}"),
            Self::Unavailable(message) | Self::InvalidInput(message) | Self::Backend(message) => {
                formatter.write_str(message)
            }
            Self::Io(error) => error.fmt(formatter),
            Self::Data(error) => error.fmt(formatter),
        }
    }
}
impl std::error::Error for SystemError {}
impl From<std::io::Error> for SystemError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<serde_json::Error> for SystemError {
    fn from(error: serde_json::Error) -> Self {
        Self::Data(error)
    }
}

/// Native text clipboard. GPUI cannot report clipboard write failures.
pub struct Clipboard;

impl Clipboard {
    pub fn write(text: impl Into<String>, cx: &mut gpui::App) {
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(text.into()));
    }
    pub fn read(cx: &gpui::App) -> Option<String> {
        cx.read_from_clipboard().and_then(|item| item.text())
    }
}

/// Native asynchronous dialogs. Cancellation is `Ok(None)` or `Ok(false)`.
pub struct Dialog;

impl Dialog {
    /// Pick one file. Linux requires a file-chooser desktop portal.
    pub fn open_file<V: 'static>(
        window: &mut Window,
        cx: &mut Context<V>,
        callback: impl FnOnce(
            &mut V,
            Result<Option<PathBuf>, SystemError>,
            &mut Window,
            &mut Context<V>,
        ) + 'static,
    ) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open file".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = match receiver.await {
                Ok(Ok(paths)) => Ok(paths.and_then(|paths| paths.into_iter().next())),
                Ok(Err(error)) => Err(SystemError::Backend(error.to_string())),
                Err(error) => Err(SystemError::Unavailable(error.to_string())),
            };
            if let Err(error) =
                this.update_in(cx, |view, window, cx| callback(view, result, window, cx))
            {
                eprintln!("dialog result unavailable: {error}");
            }
        })
        .detach();
    }

    /// Ask for confirmation without blocking the UI thread.
    pub fn confirm<V: 'static>(
        message: &str,
        window: &mut Window,
        cx: &mut Context<V>,
        callback: impl FnOnce(&mut V, Result<bool, SystemError>, &mut Window, &mut Context<V>) + 'static,
    ) {
        let receiver = window.prompt(
            PromptLevel::Info,
            message,
            None,
            &[PromptButton::ok("Continue"), PromptButton::cancel("Cancel")],
            cx,
        );
        cx.spawn_in(window, async move |this, cx| {
            let result = receiver
                .await
                .map(|answer| answer == 0)
                .map_err(|error| SystemError::Unavailable(error.to_string()));
            if let Err(error) =
                this.update_in(cx, |view, window, cx| callback(view, result, window, cx))
            {
                eprintln!("confirmation result unavailable: {error}");
            }
        })
        .detach();
    }
}

/// JSON settings at an explicit or platform-specific location.
///
/// I/O is synchronous: call `load`/`save` on `cx.background_spawn`, not in render.
/// Missing files use `Default`; malformed files return an error and are not reset.
#[derive(Clone)]
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(application: &str) -> Result<Self, SystemError> {
        if application.is_empty()
            || !application
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
        {
            return Err(SystemError::InvalidInput(
                "application identifier must contain only letters, numbers, '-' or '_'".into(),
            ));
        }
        Ok(Self::from_path(
            config_directory()?.join(application).join("settings.json"),
        ))
    }
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn load<T: DeserializeOwned + Default>(&self) -> Result<T, SystemError> {
        match fs::read(&self.path) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
            Err(error) => Err(error.into()),
        }
    }
    pub fn save<T: Serialize>(&self, value: &T) -> Result<(), SystemError> {
        let parent = self
            .path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let metadata = match fs::symlink_metadata(&self.path) {
            Ok(metadata) if metadata.is_file() => Some(metadata),
            Ok(_) => {
                return Err(SystemError::InvalidInput(
                    "settings destination must be a regular file, not a symlink".into(),
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        temporary.write_all(&serde_json::to_vec_pretty(value)?)?;
        temporary.write_all(b"\n")?;
        if let Some(metadata) = metadata {
            temporary
                .as_file()
                .set_permissions(metadata.permissions())?;
        }
        temporary.as_file().sync_all()?;
        temporary.persist(&self.path).map_err(|error| error.error)?;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn config_directory() -> Result<PathBuf, SystemError> {
    if let Some(directory) = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
    {
        return Ok(directory);
    }
    std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".config"))
        .ok_or_else(|| SystemError::Unavailable("HOME and XDG_CONFIG_HOME are unavailable".into()))
}
#[cfg(target_os = "windows")]
fn config_directory() -> Result<PathBuf, SystemError> {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| SystemError::Unavailable("APPDATA is unavailable".into()))
}
#[cfg(target_os = "macos")]
fn config_directory() -> Result<PathBuf, SystemError> {
    std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join("Library/Application Support"))
        .ok_or_else(|| SystemError::Unavailable("HOME is unavailable".into()))
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn config_directory() -> Result<PathBuf, SystemError> {
    Err(SystemError::Unsupported("settings directory"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug, Default, PartialEq, Serialize, serde::Deserialize)]
    struct Settings {
        name: String,
        enabled: bool,
    }
    #[test]
    fn persists_round_trip_and_reports_invalid_data() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let store = SettingsStore::from_path(directory.path().join("sub/settings.json"));
        assert_eq!(store.load::<Settings>()?, Settings::default());
        let settings = Settings {
            name: "Zoë 漢".into(),
            enabled: true,
        };
        store.save(&settings)?;
        assert_eq!(store.load::<Settings>()?, settings);
        fs::write(store.path(), b"not json")?;
        assert!(matches!(
            store.load::<Settings>(),
            Err(SystemError::Data(_))
        ));
        assert_eq!(fs::read(store.path())?, b"not json");
        Ok(())
    }
    #[test]
    fn rejects_path_traversal_in_application_identifier() {
        assert!(SettingsStore::new("../private").is_err());
    }
}
