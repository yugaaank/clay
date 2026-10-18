use super::*;
use gpui_framework::raw::{TestAppContext, VisualTestContext};

fn setup(
    cx: &mut TestAppContext,
    store: SettingsStore,
) -> (Entity<SettingsApp>, &mut VisualTestContext) {
    cx.update(|cx| Theme::dark().apply(cx));
    let (view, cx) =
        cx.add_window_view(|window, cx| SettingsApp::with_store(Ok(store), window, cx));
    cx.run_until_parked();
    (view, cx)
}

fn click(cx: &mut VisualTestContext, label: &'static str) {
    let bounds = cx.debug_bounds(label).expect("control rendered");
    cx.simulate_click(bounds.center(), Default::default());
}

#[gpui_framework::raw::test]
fn form_saves_reloads_and_changes_theme(cx: &mut TestAppContext) {
    let directory = tempfile::tempdir().expect("test directory");
    let store = SettingsStore::from_path(directory.path().join("preferences.json"));
    let (view, cx) = setup(cx, store.clone());
    let input = view.read_with(cx, |view, _| view.name.clone());
    cx.update(|window, cx| window.focus(&input.focus_handle(cx), cx));
    cx.simulate_keystrokes("ctrl-a");
    cx.simulate_input("Alice🙂");
    click(cx, "Dark theme");
    view.read_with(cx, |view, cx| {
        assert_eq!(view.preferences.display_name, "Alice🙂");
        assert!(!view.preferences.dark);
        assert_eq!(Theme::get(cx).mode, ThemeMode::Light);
    });
    click(cx, "Save settings");
    cx.run_until_parked();
    let persisted = store.load::<Preferences>().expect("saved settings");
    assert_eq!(persisted.display_name, "Alice🙂");
    assert!(!persisted.dark);
    view.update_in(cx, |view, window, cx| {
        view.name
            .update(cx, |input, cx| input.set_text("Temporary", window, cx));
    });
    view.update_in(cx, |view, window, cx| view.load(window, cx));
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(view.preferences.display_name, "Alice🙂");
        assert!(!view.dirty);
        assert!(!view.busy);
    });
}

#[gpui_framework::raw::test]
fn clipboard_dialogs_and_confirmation(cx: &mut TestAppContext) {
    let directory = tempfile::tempdir().expect("test directory");
    let (view, cx) = setup(
        cx,
        SettingsStore::from_path(directory.path().join("settings.json")),
    );
    click(cx, "Copy name");
    assert_eq!(
        cx.read_from_clipboard()
            .and_then(|item| item.text())
            .as_deref(),
        Some("Developer")
    );
    cx.write_to_clipboard(gpui_framework::raw::ClipboardItem::new_string(
        "Pasted name".into(),
    ));
    click(cx, "Paste name");
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(view.preferences.display_name, "Pasted name")
    });
    click(cx, "Choose file…");
    cx.simulate_path_prompt_response(|_| None);
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(view.preferences.startup_file, None)
    });
    click(cx, "Choose file…");
    let path = directory.path().join("example.txt");
    cx.simulate_path_prompt_response(|_| Some(vec![path.clone()]));
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(view.preferences.startup_file.as_deref(), path.to_str())
    });
    click(cx, "Reset…");
    cx.simulate_prompt_answer("Cancel");
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(view.preferences.display_name, "Pasted name")
    });
    click(cx, "Reset…");
    cx.simulate_prompt_answer("Continue");
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(view.preferences, Preferences::default())
    });
}
