use clay::prelude::*;
use clay::raw::{KeyBinding, SharedString};
use serde::{Deserialize, Serialize};

actions!(settings_app, [SaveSettings, OpenFolderFile]);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
struct Preferences {
    display_name: String,
    dark: bool,
    restore_workspace: bool,
    startup_file: Option<String>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            display_name: "Developer".into(),
            dark: true,
            restore_workspace: true,
            startup_file: None,
        }
    }
}

struct SettingsApp {
    name: Entity<TextInput>,
    preferences: Preferences,
    store: Option<SettingsStore>,
    busy: bool,
    dirty: bool,
    status: String,
    error: bool,
    _subscriptions: Vec<Subscription>,
}

impl SettingsApp {
    fn new(window: &mut UiWindow, cx: &mut Context<Self>) -> Self {
        let store = SettingsStore::new("clay-example");
        Self::with_store(store, window, cx)
    }

    fn with_store(
        store: Result<SettingsStore, SystemError>,
        window: &mut UiWindow,
        cx: &mut Context<Self>,
    ) -> Self {
        for primary in ["ctrl", "cmd"] {
            cx.bind_keys([
                KeyBinding::new(&format!("{primary}-s"), SaveSettings, Some("SettingsApp")),
                KeyBinding::new(&format!("{primary}-o"), OpenFolderFile, Some("SettingsApp")),
            ]);
        }
        let name = cx.new(|cx| {
            TextInput::new("", window, cx)
                .label("Display name")
                .placeholder("Your name")
        });
        let subscription = cx.subscribe(&name, |this, input, event, cx| {
            if matches!(event, InputEvent::Changed) && !this.busy {
                let text = input.read(cx).text().to_string();
                if this.preferences.display_name != text {
                    this.preferences.display_name = text;
                    this.dirty = true;
                    cx.notify();
                }
            }
        });
        let mut this = Self {
            name,
            preferences: Preferences::default(),
            store: None,
            busy: false,
            dirty: false,
            status: "Loading settings…".into(),
            error: false,
            _subscriptions: vec![subscription],
        };
        match store {
            Ok(store) => {
                this.store = Some(store);
                this.load(window, cx);
            }
            Err(error) => this.fail(error),
        }
        this
    }

    fn fail(&mut self, error: SystemError) {
        self.status = error.to_string();
        self.error = true;
    }

    fn set_busy(&mut self, busy: bool, cx: &mut Context<Self>) {
        self.busy = busy;
        self.name
            .update(cx, |input, cx| input.set_disabled(busy, cx));
        cx.notify();
    }

    fn apply(&mut self, preferences: Preferences, window: &mut UiWindow, cx: &mut Context<Self>) {
        self.preferences = preferences;
        self.name.update(cx, |input, cx| {
            input.set_text(
                SharedString::from(self.preferences.display_name.clone()),
                window,
                cx,
            )
        });
        Theme::set_mode(
            if self.preferences.dark {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            },
            cx,
        );
    }

    fn load(&mut self, window: &mut UiWindow, cx: &mut Context<Self>) {
        let Some(store) = self.store.clone() else {
            return;
        };
        self.set_busy(true, cx);
        let task = cx.background_spawn(async move { store.load::<Preferences>() });
        cx.spawn_in(window, async move |this, cx| {
            let result = task.await;
            if let Err(error) = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(preferences) => {
                        this.apply(preferences, window, cx);
                        this.status = "Settings loaded".into();
                        this.error = false;
                        this.dirty = false;
                    }
                    Err(error) => this.fail(error),
                }
                this.set_busy(false, cx);
            }) {
                eprintln!("settings load result unavailable: {error}");
            }
        })
        .detach();
    }

    fn save(&mut self, window: &mut UiWindow, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let Some(store) = self.store.clone() else {
            return;
        };
        let preferences = self.preferences.clone();
        self.status = "Saving…".into();
        self.set_busy(true, cx);
        let task = cx.background_spawn(async move { store.save(&preferences) });
        cx.spawn_in(window, async move |this, cx| {
            let result = task.await;
            if let Err(error) = this.update_in(cx, |this, _, cx| {
                match result {
                    Ok(()) => {
                        this.status = "Settings saved".into();
                        this.error = false;
                        this.dirty = false;
                    }
                    Err(error) => this.fail(error),
                }
                this.set_busy(false, cx);
            }) {
                eprintln!("settings save result unavailable: {error}");
            }
        })
        .detach();
    }

    fn open(&mut self, window: &mut UiWindow, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.set_busy(true, cx);
        Dialog::open_file(window, cx, |this, result, _, cx| {
            match result {
                Ok(Some(path)) => {
                    this.preferences.startup_file = Some(path.to_string_lossy().into_owned());
                    this.dirty = true;
                    this.status = "Selected file; save to remember it".into();
                    this.error = false;
                }
                Ok(None) => this.status = "File selection canceled".into(),
                Err(error) => this.fail(error),
            }
            this.set_busy(false, cx);
        });
    }

    fn reset(&mut self, window: &mut UiWindow, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.set_busy(true, cx);
        Dialog::confirm(
            "Reset the form to defaults? Save to keep the changes.",
            window,
            cx,
            |this, result, window, cx| {
                match result {
                    Ok(true) => {
                        this.apply(Preferences::default(), window, cx);
                        this.dirty = true;
                        this.status = "Defaults restored; not saved yet".into();
                        this.error = false;
                    }
                    Ok(false) => this.status = "Reset canceled".into(),
                    Err(error) => this.fail(error),
                }
                this.set_busy(false, cx);
            },
        );
    }
}

impl Render for SettingsApp {
    fn render(&mut self, _: &mut UiWindow, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let busy = self.busy;
        Row::new()
            .size_full()
            .gap(Spacing::None)
            .child(
                Container::new()
                    .w(px(210.))
                    .h_full()
                    .padding(Spacing::Lg)
                    .bg(theme.colors.surface)
                    .border_r_1()
                    .border_color(theme.colors.border)
                    .child(Text::new("Clay").heading())
                    .child(Text::new("Settings example").muted())
                    .child(Text::new("General"))
                    .child(
                        Text::new("Ctrl+S to save\nCtrl+O to choose a file\nTab to move focus")
                            .muted(),
                    ),
            )
            .child(
                clay::raw::div()
                    .id("settings-content")
                    .key_context("SettingsApp")
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .overflow_y_scroll()
                    .on_action(
                        cx.listener(|this, _: &SaveSettings, window, cx| this.save(window, cx)),
                    )
                    .on_action(
                        cx.listener(|this, _: &OpenFolderFile, window, cx| this.open(window, cx)),
                    )
                    .child(
                        Column::new()
                            .padding(Spacing::Lg)
                            .gap(Spacing::Lg)
                            .child(Text::new("General settings").heading())
                            .child(Text::new("Changes stay in this form until you save.").muted())
                            .child(self.name.clone())
                            .child(
                                Checkbox::new("dark-theme", "Dark theme", self.preferences.dark)
                                    .disabled(busy)
                                    .on_change(cx.listener(|this, dark: &bool, _, cx| {
                                        this.preferences.dark = *dark;
                                        this.dirty = true;
                                        Theme::set_mode(
                                            if *dark {
                                                ThemeMode::Dark
                                            } else {
                                                ThemeMode::Light
                                            },
                                            cx,
                                        );
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Checkbox::new(
                                    "restore-workspace",
                                    "Remember workspace on startup",
                                    self.preferences.restore_workspace,
                                )
                                .disabled(busy)
                                .on_change(cx.listener(
                                    |this, enabled: &bool, _, cx| {
                                        this.preferences.restore_workspace = *enabled;
                                        this.dirty = true;
                                        cx.notify();
                                    },
                                )),
                            )
                            .child(Text::new("Startup file").heading())
                            .child(
                                Text::new(
                                    self.preferences
                                        .startup_file
                                        .clone()
                                        .unwrap_or_else(|| "No file selected".into()),
                                )
                                .muted(),
                            )
                            .child(
                                Row::new()
                                    .gap(Spacing::Sm)
                                    .child(
                                        Button::new("open-file", "Choose file…")
                                            .disabled(busy)
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.open(window, cx)
                                            })),
                                    )
                                    .child(
                                        Button::new("copy-name", "Copy name")
                                            .disabled(busy)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                Clipboard::write(
                                                    this.preferences.display_name.clone(),
                                                    cx,
                                                );
                                                this.status = "Name copied".into();
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("paste-name", "Paste name")
                                            .disabled(busy)
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                if let Some(text) = Clipboard::read(cx) {
                                                    this.name.update(cx, |input, cx| {
                                                        input.set_text(text, window, cx)
                                                    });
                                                }
                                            })),
                                    ),
                            )
                            .child(
                                Row::new()
                                    .gap(Spacing::Sm)
                                    .child(
                                        Button::new("save-settings", "Save settings")
                                            .primary()
                                            .disabled(busy || self.store.is_none())
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.save(window, cx)
                                            })),
                                    )
                                    .child(
                                        Button::new("reset-settings", "Reset…")
                                            .disabled(busy)
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.reset(window, cx)
                                            })),
                                    ),
                            )
                            .child(
                                Text::new(if self.dirty {
                                    "Unsaved changes"
                                } else {
                                    "No unsaved changes"
                                })
                                .muted(),
                            )
                            .child(
                                Container::new()
                                    .text_color(if self.error {
                                        theme.colors.danger
                                    } else {
                                        theme.colors.text
                                    })
                                    .child(self.status.clone()),
                            ),
                    ),
            )
    }
}

fn main() -> Result<(), SystemError> {
    App::new()
        .theme(ThemeMode::Dark)
        .window(
            Window::new("Framework Settings").size(960., 680.),
            SettingsApp::new,
        )
        .run()
}

#[cfg(all(test, feature = "test-support"))]
#[path = "tests.rs"]
mod tests;
