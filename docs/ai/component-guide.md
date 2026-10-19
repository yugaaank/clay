# Components, layout, themes, and input

## Layout convention

Use `Column` for vertical content, `Row` for horizontal regions, and
`Container` for a styled or fixed-size region. A header/sidebar/content shell is
usually a full-size `Row`:

```rust
Row::new()
    .size_full()
    .gap(Spacing::None)
    .child(
        Container::new()
            .w(px(220.))
            .h_full()
            .padding(Spacing::Lg)
            .bg(theme.colors.surface)
            .child(Text::new("Sidebar").heading()),
    )
    .child(
        Column::new()
            .flex_1()
            .min_w_0()
            .padding(Spacing::Lg)
            .child(Text::new("Main content").heading()),
    )
```

The `size_full`, `flex_1`, `min_w_0`, `h_full`, `w`, `bg`, and border methods in
this example are GPUI `Styled` methods re-exported through the prelude. The
framework layout wrappers do not hide GPUI styling.

## Text and controls

Use `Text::new(value)` for labels, `.heading()` for a heading, and `.muted()`
for secondary information. Give each button and checkbox a stable ID:

```rust
Row::new()
    .gap(Spacing::Sm)
    .child(Button::new("cancel", "Cancel"))
    .child(Button::new("submit", "Submit").primary())
```

Buttons and checkboxes are keyboard-focusable. The application root binds Tab and
Shift-Tab to focus traversal and Ctrl-W/Cmd-W to close the window.

## Controlled checkboxes

The checkbox does not own application state. Render the current value and update
the parent when it changes:

```rust
Checkbox::new("notifications", "Notifications", self.notifications)
    .on_change(cx.listener(|this, value: &bool, _, cx| {
        this.notifications = *value;
        cx.notify();
    }))
```

## Text input and focus

Create a `TextInput` as an entity because it owns focus and native input state:

```rust
let input = cx.new(|cx| {
    TextInput::new("", window, cx)
        .label("Search")
        .placeholder("Type to search")
});
```

`TextInput::new` does not steal focus. Keep the returned `Entity<TextInput>` in
the parent view and render it as a child. Use `set_disabled` while saving or
loading. Use `set_text` for programmatic updates; it places the caret at the end.

The input is single-line. Newlines and paragraph separators are normalized to
spaces, NUL-containing edits are rejected, and Unicode graphemes are handled by
the native editor bridge.

## Themes and styling

Read semantic roles inside `render`:

```rust
let theme = Theme::get(cx).clone();

Container::new()
    .bg(theme.colors.surface)
    .border_1()
    .border_color(theme.colors.border)
    .padding(Spacing::Md)
    .child(Text::new("Panel"));
```

Start the application with `ThemeMode::Dark`, `ThemeMode::Light`, or
`ThemeMode::System`. To switch later, call `Theme::set_mode` in an event handler
and notify the parent when other state also changed.

## What is not a component

There is no framework table, list, virtualized collection, select, menu, tabs,
navigation system, modal component, or form-validation component yet. Use raw
GPUI only when the application genuinely needs one, and document the workaround
in the application.
