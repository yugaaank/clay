# Public API reference

The public facade is exported from `clay`. Most applications should
start with:

```rust
use clay::prelude::*;
```

The prelude re-exports framework types plus selected GPUI traits and types:
`AppContext`, `Context`, `Entity`, `EventEmitter`, `Focusable`, `IntoElement`,
`Render`, `Subscription`, `actions`, `px`, `UiWindow`, and `State<T>`.

## Application

```rust
App::new()
    .theme(ThemeMode::Dark)
    .window(Window::new("Notes").size(900., 600.), |window, cx| {
        NotesView::new(window, cx)
    })
    .run()?;
```

Verified methods:

- `App::new() -> App`
- `App::theme(ThemeMode) -> App`
- `App::window(config, build) -> App`
- `App::run() -> Result<(), SystemError>`
- `Window::new(title) -> Window`
- `Window::size(width, height) -> Window`

`App::run` requires at least one window. Window dimensions must be finite and at
least `240 × 160`; the framework's minimum window size is `480 × 320`.

## Layout and text

`Column`, `Row`, and `Container` support `new`, `child`, `children`, `gap`, and
`padding`. `Text` supports `new`, `muted`, and `heading`:

```rust
Column::new()
    .padding(Spacing::Lg)
    .gap(Spacing::Md)
    .child(Text::new("Title").heading())
    .child(Text::new("Supporting text").muted());
```

They retain GPUI styling through the standard `Styled` methods, but use the
framework theme for their defaults.

## Controls

```rust
Button::new("save", "Save")
    .primary()
    .disabled(is_busy)
    .on_click(cx.listener(|this, _, _, cx| {
        this.save(cx);
    }));

Checkbox::new("dark", "Dark theme", dark)
    .disabled(is_busy)
    .on_change(cx.listener(|this, value: &bool, _, cx| {
        this.dark = *value;
        cx.notify();
    }));
```

`Button::new` and `Checkbox::new` require stable element IDs. `Button` supports
`disabled`, `primary`, and `on_click`. `Checkbox` supports `disabled` and
`on_change`; it is controlled, so store the checked value in the parent view.

## Text input

```rust
let input = cx.new(|cx| {
    TextInput::new("initial value", window, cx)
        .label("Name")
        .placeholder("Your name")
});
```

Verified methods include `new`, `label`, `placeholder`, `text`, `set_text`, and
`set_disabled`. `TextInput` emits `InputEvent::Changed` and
`InputEvent::Submitted`. Subscribe to the entity to mirror changes into parent
state. It is single-line and handles Unicode, IME, selection, and clipboard
keyboard actions.

## Themes

`ThemeMode` is `Light`, `Dark`, or `System`. `Theme::dark()`, `Theme::light()`,
`Theme::get(cx)`, `Theme::apply(cx)`, and `Theme::set_mode(mode, cx)` are the
verified theme entry points. Use `Theme::get(cx).colors` and
`Theme::get(cx).spacing` for semantic styling.

`Spacing` provides `None`, `Xs`, `Sm`, `Md`, and `Lg`. Do not copy palette values
from the built-in themes into application components.

## Raw GPUI

```rust
let custom = clay::raw::div()
    .id("custom-area")
    .flex_1()
    .child(Text::new("Custom GPUI element"));
```

`clay::raw` is the pinned GPUI crate. It is appropriate for custom
elements, painting, advanced actions, charts, and collection UIs not yet covered
by the facade. Check the installed GPUI revision before using unfamiliar raw
methods.
