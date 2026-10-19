# State, events, and background tasks

## View state

Application state lives in a type that implements `Render`. GPUI owns it as an
entity created by `App::window` or `cx.new`:

```rust
struct Counter {
    value: i32,
}

impl Render for Counter {
    fn render(&mut self, _: &mut UiWindow, cx: &mut Context<Self>) -> impl IntoElement {
        Button::new("increment", format!("{}", self.value)).on_click(
            cx.listener(|this, _, _, cx| {
                this.value += 1;
                cx.notify();
            }),
        )
    }
}
```

The event handler mutates the entity and calls `cx.notify()`. Without the
notification, the state changes but the rendered tree is not invalidated.

## Subscribing to input

`TextInput` implements `EventEmitter<InputEvent>`. The settings example mirrors
the input into its preferences:

```rust
let subscription = cx.subscribe(&name, |this, input, event, cx| {
    if matches!(event, InputEvent::Changed) && !this.busy {
        this.preferences.display_name = input.read(cx).text().to_string();
        this.dirty = true;
        cx.notify();
    }
});
```

Store subscriptions in the parent so they live as long as the relationship:

```rust
struct Form {
    name: Entity<TextInput>,
    _subscriptions: Vec<Subscription>,
}
```

## Actions and keyboard shortcuts

Define actions with GPUI's `actions!` macro, bind them in the entity context, and
handle them on a stable raw element. The settings example binds Ctrl/Cmd-S and
Ctrl/Cmd-O. Use the raw escape hatch for action routing because the facade does
not wrap arbitrary application actions.

## Background work

Never perform synchronous file, process, or network work in `render`. Use this
shape:

```rust
fn load(&mut self, window: &mut UiWindow, cx: &mut Context<Self>) {
    self.busy = true;
    cx.notify();

    let task = cx.background_spawn(async move {
        // Perform blocking or CPU-heavy work here.
        load_value()
    });

    cx.spawn_in(window, async move |this, cx| {
        let result = task.await;
        let _ = this.update_in(cx, |this, _, cx| {
            this.busy = false;
            this.value = result;
            cx.notify();
        });
    })
    .detach();
}
```

The exact result type and error handling are application-specific. See
`examples/settings-app/src/main.rs` for a complete load/save implementation and
`examples/system-monitor/src/main.rs` for a repeating collector.

## Lifecycle and cancellation

Detached tasks stop updating an entity after it is gone because `update` or
`update_in` returns an error. The framework does not yet provide a high-level
cancellation token or task manager. For long-running work, add an application
cancel flag or generation counter and discard stale results explicitly.

## Loading, error, and empty states

Represent the state explicitly rather than using a hidden sentinel:

```rust
enum ScreenState<T> {
    Loading,
    Ready(T),
    Error(String),
}
```

Render a separate empty state when `Ready` contains no records. The system
monitor is the reference for `Loading`, `Error`, and empty process states.
