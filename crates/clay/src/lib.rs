//! A small GPUI application layer. See `examples/settings-app` for a complete app.
//!
//! The `raw` module is the explicit escape hatch to the pinned GPUI version.
pub use gpui as raw;

mod app;
mod components;
mod input;
mod services;
mod theme;

pub use app::{App, Window};
pub use components::{Button, Checkbox, Column, Container, Row, Text};
pub use input::{InputEvent, TextInput};
pub use services::{Clipboard, Dialog, SettingsStore, SystemError};
pub use theme::{Colors, Spacing, Theme, ThemeMode};

/// Common framework types and the traits used by view builders.
pub mod prelude {
    pub use crate::{
        App, Button, Checkbox, Clipboard, Column, Container, Dialog, InputEvent, Row,
        SettingsStore, Spacing, SystemError, Text, TextInput, Theme, ThemeMode, Window,
    };
    pub use gpui::{
        self, AppContext, Context, Entity, EventEmitter, Focusable, IntoElement, Render,
        Subscription, actions, prelude::*, px,
    };
    /// The native window context passed to a view or callback.
    pub type UiWindow = gpui::Window;
    /// Shared application state uses GPUI's single-threaded entity ownership.
    pub type State<T> = gpui::Entity<T>;
}
