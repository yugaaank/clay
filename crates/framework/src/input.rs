use std::{borrow::Cow, ops::Range};

use gpui::{
    App, Bounds, ClipboardItem, ContentMask, Context, CursorStyle, ElementId, ElementInputHandler,
    Entity, EntityInputHandler, EventEmitter, FocusHandle, Focusable, Global, GlobalElementId,
    InspectorElementId, KeyBinding, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels,
    Point, Role, ShapedLine, SharedString, Style, Subscription, TextAlign, TextInputAction,
    TextInputConfiguration, TextRun, UTF16Selection, UnderlineStyle, Window, actions, div, fill,
    point, prelude::*, px, relative, size,
};
use unicode_segmentation::UnicodeSegmentation;

/// Notifications from a [`TextInput`]. Read the entity's [`TextInput::text`] for its current value.
#[derive(Clone, Debug)]
pub enum InputEvent {
    Changed,
    Submitted,
}

actions!(
    framework_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        Home,
        End,
        SelectHome,
        SelectEnd,
        SelectAll,
        Copy,
        Cut,
        Paste,
        Submit,
    ]
);

struct InputBindings;
impl Global for InputBindings {}

fn init_bindings(cx: &mut App) {
    if cx.has_global::<InputBindings>() {
        return;
    }
    cx.set_global(InputBindings);
    let context = Some("FrameworkTextInput");
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, context),
        KeyBinding::new("delete", Delete, context),
        KeyBinding::new("left", Left, context),
        KeyBinding::new("right", Right, context),
        KeyBinding::new("shift-left", SelectLeft, context),
        KeyBinding::new("shift-right", SelectRight, context),
        KeyBinding::new("home", Home, context),
        KeyBinding::new("end", End, context),
        KeyBinding::new("shift-home", SelectHome, context),
        KeyBinding::new("shift-end", SelectEnd, context),
        KeyBinding::new("enter", Submit, context),
    ]);
    #[cfg(target_os = "macos")]
    cx.bind_keys([
        KeyBinding::new("cmd-a", SelectAll, context),
        KeyBinding::new("cmd-c", Copy, context),
        KeyBinding::new("cmd-x", Cut, context),
        KeyBinding::new("cmd-v", Paste, context),
        KeyBinding::new("cmd-left", Home, context),
        KeyBinding::new("cmd-right", End, context),
        KeyBinding::new("cmd-shift-left", SelectHome, context),
        KeyBinding::new("cmd-shift-right", SelectEnd, context),
    ]);
    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([
        KeyBinding::new("ctrl-a", SelectAll, context),
        KeyBinding::new("ctrl-c", Copy, context),
        KeyBinding::new("ctrl-x", Cut, context),
        KeyBinding::new("ctrl-v", Paste, context),
    ]);
}

/// A single-line native text editor. Construction never takes keyboard focus.
///
/// CRLF, CR, LF, and Unicode line/paragraph separators become spaces. An edit containing
/// NUL is rejected atomically; an invalid initial value becomes empty. Programmatic changes
/// emit `Changed` only when the value changes, including changes made while disabled.
pub struct TextInput {
    focus_handle: FocusHandle,
    content: SharedString,
    placeholder: SharedString,
    label: SharedString,
    disabled: bool,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    is_selecting: bool,
    scroll_offset: Pixels,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    _focus_subscriptions: [Subscription; 2],
}

impl EventEmitter<InputEvent> for TextInput {}

impl TextInput {
    pub fn new(
        value: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        init_bindings(cx);
        let value = value.into();
        let content: SharedString = single_line(&value).unwrap_or_default().into_owned().into();
        let end = content.len();
        let focus_handle = cx.focus_handle().tab_index(0).tab_stop(true);
        let focus = cx.on_focus(&focus_handle, window, |input, window, cx| {
            if input.disabled {
                window.blur(cx);
            }
            cx.notify();
        });
        let blur = cx.on_blur(&focus_handle, window, |input, window, cx| {
            input.is_selecting = false;
            input.marked_range = None;
            window.notify_text_input_changed(&input.focus_handle);
            cx.notify();
        });
        Self {
            focus_handle,
            content,
            placeholder: SharedString::default(),
            label: SharedString::default(),
            disabled: false,
            selected_range: end..end,
            selection_reversed: false,
            marked_range: None,
            is_selecting: false,
            scroll_offset: px(0.),
            last_layout: None,
            last_bounds: None,
            _focus_subscriptions: [focus, blur],
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        let placeholder = placeholder.into();
        self.placeholder = single_line(&placeholder)
            .unwrap_or_default()
            .into_owned()
            .into();
        self
    }

    /// Sets both the visible label and the accessible name.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.label = single_line(&label).unwrap_or_default().into_owned().into();
        self
    }

    pub fn text(&self) -> SharedString {
        self.content.clone()
    }

    /// Replaces the value and places the caret at its end. Equal or invalid values are no-ops.
    pub fn set_text(
        &mut self,
        value: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = value.into();
        let Some(value) = single_line(&value) else {
            return;
        };
        if value.as_ref() == self.content.as_ref() {
            return;
        }
        self.content = value.into_owned().into();
        let end = self.content.len();
        self.selected_range = end..end;
        self.selection_reversed = false;
        self.marked_range = None;
        self.is_selecting = false;
        self.text_changed(true, window, cx);
    }

    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        if self.disabled == disabled {
            return;
        }
        self.disabled = disabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(!disabled);
        self.is_selecting = false;
        self.marked_range = None;
        cx.notify();
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn move_to(&mut self, offset: usize, extend: bool, window: &Window, cx: &mut Context<Self>) {
        let anchor = if extend {
            if self.selection_reversed {
                self.selected_range.end
            } else {
                self.selected_range.start
            }
        } else {
            offset
        };
        self.selected_range = anchor.min(offset)..anchor.max(offset);
        self.selection_reversed = offset < anchor;
        if self.marked_range.take().is_some() {
            window.notify_text_input_changed(&self.focus_handle);
        } else {
            window.notify_text_selection_changed(&self.focus_handle);
        }
        window.invalidate_character_coordinates();
        cx.notify();
    }

    fn text_changed(&mut self, changed: bool, window: &Window, cx: &mut Context<Self>) {
        self.last_layout = None;
        window.notify_text_input_changed(&self.focus_handle);
        window.invalidate_character_coordinates();
        cx.notify();
        if changed {
            cx.emit(InputEvent::Changed);
        }
    }

    fn replace_native(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        selected_utf16: Option<Range<usize>>,
        marking: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        // Validate before touching the selection or composition, even for explicit native ranges.
        let Some(replacement) = single_line(new_text) else {
            return;
        };
        let range = range_utf16
            .map(|range| range_from_utf16(&self.content, range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range.clone());
        let mut content = String::with_capacity(
            self.content.len() - (range.end - range.start) + replacement.len(),
        );
        content.push_str(&self.content[..range.start]);
        content.push_str(&replacement);
        content.push_str(&self.content[range.end..]);
        let changed = content != self.content.as_ref();
        let end = range.start + replacement.len();
        self.selected_range = if marking {
            selected_utf16
                .as_ref()
                .map(|selected| {
                    let raw = range_from_utf16(new_text, selected.clone());
                    // Composition selections refer to the unsanitized insertion, not the document.
                    let start = single_line(&new_text[..raw.start])
                        .map(|text| text.len())
                        .unwrap_or(0);
                    let end = single_line(&new_text[..raw.end])
                        .map(|text| text.len())
                        .unwrap_or(0);
                    range.start + start..range.start + end
                })
                .unwrap_or(end..end)
        } else {
            end..end
        };
        self.selection_reversed =
            marking && selected_utf16.is_some_and(|selection| selection.start > selection.end);
        self.marked_range = (marking && !replacement.is_empty()).then_some(range.start..end);
        self.content = content.into();
        self.text_changed(changed, window, cx);
    }

    fn navigate(
        &mut self,
        direction: Navigation,
        extend: bool,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let cursor = self.cursor_offset();
        let offset = match direction {
            Navigation::Home => 0,
            Navigation::End => self.content.len(),
            Navigation::Left if !extend && !self.selected_range.is_empty() => {
                grapheme_floor(&self.content, self.selected_range.start)
            }
            Navigation::Right if !extend && !self.selected_range.is_empty() => {
                grapheme_ceil(&self.content, self.selected_range.end)
            }
            Navigation::Left => previous_grapheme(&self.content, cursor),
            Navigation::Right => next_grapheme(&self.content, cursor),
        };
        self.move_to(offset, extend, window, cx);
    }

    fn delete(&mut self, backwards: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let range = if !self.selected_range.is_empty() {
            self.selected_range.clone()
        } else if backwards {
            previous_grapheme(&self.content, self.cursor_offset())
                ..grapheme_ceil(&self.content, self.cursor_offset())
        } else {
            grapheme_floor(&self.content, self.cursor_offset())
                ..next_grapheme(&self.content, self.cursor_offset())
        };
        if !range.is_empty() {
            let range = range_to_utf16(&self.content, &range);
            self.replace_native(Some(range), "", None, false, window, cx);
        }
    }

    fn select_all(&mut self, window: &Window, cx: &mut Context<Self>) {
        if !self.disabled {
            self.move_to(0, false, window, cx);
            self.move_to(self.content.len(), true, window, cx);
        }
    }

    fn copy(&self, cx: &mut Context<Self>) {
        if !self.disabled && !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_owned(),
            ));
        }
    }

    fn cut(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.disabled && !self.selected_range.is_empty() {
            self.copy(cx);
            // Cut exactly what was copied, including native selections within a grapheme.
            let range = range_to_utf16(&self.content, &self.selected_range);
            self.replace_native(Some(range), "", None, false, window, cx);
        }
    }

    fn paste_clipboard(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.disabled
            && let Some(text) = cx.read_from_clipboard().and_then(|item| item.text())
        {
            self.replace_native(None, &text, None, false, window, cx);
        }
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        // The Enter that commits an in-progress composition must not submit the form too.
        if self.marked_range.take().is_some() {
            window.notify_text_input_changed(&self.focus_handle);
            window.invalidate_character_coordinates();
            cx.notify();
        } else {
            cx.emit(InputEvent::Submitted);
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        let (Some(bounds), Some(line)) = (self.last_bounds, self.last_layout.as_ref()) else {
            return self.cursor_offset();
        };
        let x = position.x - bounds.left() + self.scroll_offset;
        let index = line.closest_index_for_x(x).min(self.content.len());
        let before = grapheme_floor(&self.content, index);
        let after = grapheme_ceil(&self.content, index);
        if (line.x_for_index(before) - x).abs() <= (line.x_for_index(after) - x).abs() {
            before
        } else {
            after
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        window.focus(&self.focus_handle, cx);
        window.request_virtual_keyboard();
        self.is_selecting = true;
        let offset = self.index_for_mouse_position(event.position);
        if event.click_count >= 3 {
            self.select_all(window, cx);
        } else {
            self.move_to(offset, event.modifiers.shift, window, cx);
        }
        cx.stop_propagation();
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !event.dragging() || self.disabled {
            self.is_selecting = false;
            return;
        }
        if self.is_selecting {
            self.move_to(
                self.index_for_mouse_position(event.position),
                true,
                window,
                cx,
            );
            cx.stop_propagation();
        }
    }
}

#[derive(Clone, Copy)]
enum Navigation {
    Left,
    Right,
    Home,
    End,
}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = range_from_utf16(&self.content, range);
        *actual_range = Some(range_to_utf16(&self.content, &range));
        Some(self.content[range].to_owned())
    }

    fn selected_text_range(
        &mut self,
        ignore_disabled_input: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        if self.disabled && !ignore_disabled_input {
            return None;
        }
        Some(UTF16Selection {
            range: range_to_utf16(&self.content, &self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn set_selected_text_range(
        &mut self,
        range: Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        self.selection_reversed = range.start > range.end;
        self.selected_range = range_from_utf16(&self.content, range);
        window.notify_text_selection_changed(&self.focus_handle);
        window.invalidate_character_coordinates();
        cx.notify();
    }

    fn text_length_utf16(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        Some(self.content.encode_utf16().count())
    }

    fn accepts_text_input(&self, _: &mut Window, _: &mut Context<Self>) -> bool {
        !self.disabled
    }

    fn text_input_configuration(
        &mut self,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> TextInputConfiguration {
        TextInputConfiguration {
            input_action: TextInputAction::Done,
            ..Default::default()
        }
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| range_to_utf16(&self.content, range))
    }

    fn unmark_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.marked_range.take().is_some() {
            window.notify_text_input_changed(&self.focus_handle);
            window.invalidate_character_coordinates();
            cx.notify();
        }
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Software keyboards can deliver their confirm key as text rather than a keybinding.
        if range.is_none() && matches!(text, "\n" | "\r" | "\r\n") {
            self.submit(window, cx);
        } else {
            self.replace_native(range, text, None, false, window, cx);
        }
    }

    fn paste(&mut self, item: ClipboardItem, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = item.text() {
            // A clipboard containing just a newline is still a paste, not a submission.
            self.replace_native(None, &text, None, false, window, cx);
        }
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected_range: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace_native(range, text, selected_range, true, window, cx);
    }

    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let line = self.last_layout.as_ref()?;
        if !self.content.is_empty() && line.text != self.content {
            return None;
        }
        let range = range_from_utf16(&self.content, range);
        let start = bounds.left() + line.x_for_index(range.start) - self.scroll_offset;
        let end = bounds.left() + line.x_for_index(range.end) - self.scroll_offset;
        let left = start.min(end).max(bounds.left()).min(bounds.right());
        let right = start.max(end).max(left).min(bounds.right());
        Some(Bounds::new(
            point(left, bounds.top()),
            size((right - left).max(px(1.)), bounds.size.height),
        ))
    }

    fn selection_bounds_for_range(
        &mut self,
        range: Range<usize>,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<Bounds<Pixels>> {
        self.bounds_for_range(range, bounds, window, cx)
            .into_iter()
            .collect()
    }

    fn character_index_for_point(
        &mut self,
        position: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        let line = self.last_layout.as_ref()?;
        if !bounds.contains(&position) || (!self.content.is_empty() && line.text != self.content) {
            return None;
        }
        Some(byte_to_utf16(
            &self.content,
            self.index_for_mouse_position(position),
        ))
    }
}

struct InputLine {
    input: Entity<TextInput>,
}

struct LinePrepaint {
    line: ShapedLine,
    origin: Point<Pixels>,
    selection: Option<Bounds<Pixels>>,
    caret: Option<Bounds<Pixels>>,
    invalidate_coordinates: bool,
}

impl IntoElement for InputLine {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl Element for InputLine {
    type RequestLayoutState = ();
    type PrepaintState = LinePrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> LinePrepaint {
        let input = self.input.read(cx);
        let theme = crate::Theme::get(cx);
        let focused = input.focus_handle.is_focused(window) && !input.disabled;
        let style = window.text_style();
        let display_text = if input.content.is_empty() {
            input.placeholder.clone()
        } else {
            input.content.clone()
        };
        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: if input.content.is_empty() || input.disabled {
                theme.colors.muted
            } else {
                theme.colors.text
            },
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(marked) = input.marked_range.as_ref() {
            vec![
                TextRun {
                    len: marked.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked.end - marked.start,
                    underline: Some(UnderlineStyle {
                        color: Some(theme.colors.accent),
                        thickness: px(1.),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect::<Vec<_>>()
        } else {
            vec![run]
        };
        let line = window.text_system().shape_line(
            display_text,
            style.font_size.to_pixels(window.rem_size()),
            &runs,
            None,
        );
        let caret_x = line.x_for_index(input.cursor_offset());
        let scroll = caret_scroll(
            input.scroll_offset,
            caret_x,
            if input.content.is_empty() {
                px(0.)
            } else {
                line.width()
            },
            bounds.size.width,
        );
        let origin = point(bounds.left() - scroll, bounds.top());
        let selection = (!input.selected_range.is_empty() && !input.disabled).then(|| {
            let start = line.x_for_index(input.selected_range.start);
            let end = line.x_for_index(input.selected_range.end);
            Bounds::new(
                point(origin.x + start.min(end), bounds.top()),
                size((end - start).abs(), bounds.size.height),
            )
        });
        let caret = (focused && input.selected_range.is_empty()).then(|| {
            Bounds::new(
                point(origin.x + caret_x, bounds.top()),
                size(px(1.), bounds.size.height),
            )
        });
        let invalidate_coordinates = input.last_bounds != Some(bounds)
            || input.scroll_offset != scroll
            || input.last_layout.as_ref().is_none_or(|previous| {
                previous.text != line.text
                    || previous.width() != line.width()
                    || previous.font_size != line.font_size
            });
        self.input
            .update(cx, |input, _| input.scroll_offset = scroll);
        LinePrepaint {
            line,
            origin,
            selection,
            caret,
            invalidate_coordinates,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        prepaint: &mut LinePrepaint,
        window: &mut Window,
        cx: &mut App,
    ) {
        let input = self.input.read(cx);
        if !input.disabled {
            window.handle_input(
                &input.focus_handle,
                ElementInputHandler::new(bounds, self.input.clone()),
                cx,
            );
        }
        // Div mouse-move callbacks only run inside their hitbox. Capture keeps a selection drag
        // working beyond the field, including when horizontal scrolling moves the text underneath it.
        let input = self.input.clone();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase == gpui::DispatchPhase::Capture && input.read(cx).is_selecting {
                input.update(cx, |input, cx| input.on_mouse_move(event, window, cx));
            }
        });
        let input = self.input.clone();
        window.on_mouse_event(move |event: &gpui::MouseUpEvent, phase, _, cx| {
            if phase == gpui::DispatchPhase::Capture
                && event.button == MouseButton::Left
                && input.read(cx).is_selecting
            {
                input.update(cx, |input, _| input.is_selecting = false);
            }
        });

        let theme = crate::Theme::get(cx);
        let selection_color = theme.colors.selection;
        let caret_color = theme.colors.text;
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            if let Some(selection) = prepaint.selection {
                window.paint_quad(fill(selection, selection_color));
            }
            if let Err(error) = prepaint.line.paint(
                prepaint.origin,
                bounds.size.height,
                TextAlign::Left,
                None,
                window,
                cx,
            ) {
                eprintln!("TextInput could not paint text: {error}");
            }
            if let Some(caret) = prepaint.caret {
                window.paint_quad(fill(caret, caret_color));
            }
        });
        self.input.update(cx, |input, _| {
            input.last_layout = Some(prepaint.line.clone());
            input.last_bounds = Some(bounds);
        });
        if prepaint.invalidate_coordinates {
            window.invalidate_character_coordinates();
        }
    }
}

fn caret_scroll(current: Pixels, caret: Pixels, text_width: Pixels, viewport: Pixels) -> Pixels {
    let available = (viewport - px(1.)).max(px(0.));
    let maximum = (text_width - available).max(px(0.));
    let mut scroll = current.max(px(0.)).min(maximum);
    if caret < scroll {
        scroll = caret;
    } else if caret > scroll + available {
        scroll = caret - available;
    }
    scroll.max(px(0.)).min(maximum)
}

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.disabled && self.focus_handle.is_focused(window) {
            window.blur(cx);
        }
        let theme = crate::Theme::get(cx);
        let disabled = self.disabled;
        let accessible_name = if !self.label.is_empty() {
            self.label.clone()
        } else if !self.placeholder.is_empty() {
            self.placeholder.clone()
        } else {
            "Text input".into()
        };
        let field =
            div()
                .id(("framework-text-input", cx.entity_id()))
                .accessibility_id(format!("framework-text-input-{}", cx.entity_id().as_u64()))
                .role(Role::TextInput)
                .aria_label(accessible_name)
                .aria_value(self.content.clone())
                .aria_placeholder(self.placeholder.clone())
                .a11y_synthetic_children(move |builder| {
                    if disabled {
                        builder.parent_node().set_disabled();
                    }
                })
                .track_focus(&self.focus_handle)
                .tab_index(0)
                .tab_stop(!disabled)
                .min_w_0()
                .w_full()
                .px(theme.spacing.sm)
                .py(theme.spacing.xs)
                .rounded(theme.spacing.xs)
                .border_1()
                .border_color(if !disabled && self.focus_handle.is_focused(window) {
                    theme.colors.focus
                } else {
                    theme.colors.border
                })
                .bg(if disabled {
                    theme.colors.background
                } else {
                    theme.colors.surface
                })
                .cursor(if disabled {
                    CursorStyle::Arrow
                } else {
                    CursorStyle::IBeam
                })
                .overflow_hidden()
                .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
                .when(!disabled, |field| {
                    field
                        .key_context("FrameworkTextInput")
                        .on_action(cx.listener(|input, _: &Backspace, window, cx| {
                            input.delete(true, window, cx)
                        }))
                        .on_action(cx.listener(|input, _: &Delete, window, cx| {
                            input.delete(false, window, cx)
                        }))
                        .on_action(cx.listener(|input, _: &Left, window, cx| {
                            input.navigate(Navigation::Left, false, window, cx)
                        }))
                        .on_action(cx.listener(|input, _: &Right, window, cx| {
                            input.navigate(Navigation::Right, false, window, cx)
                        }))
                        .on_action(cx.listener(|input, _: &SelectLeft, window, cx| {
                            input.navigate(Navigation::Left, true, window, cx)
                        }))
                        .on_action(cx.listener(|input, _: &SelectRight, window, cx| {
                            input.navigate(Navigation::Right, true, window, cx)
                        }))
                        .on_action(cx.listener(|input, _: &Home, window, cx| {
                            input.navigate(Navigation::Home, false, window, cx)
                        }))
                        .on_action(cx.listener(|input, _: &End, window, cx| {
                            input.navigate(Navigation::End, false, window, cx)
                        }))
                        .on_action(cx.listener(|input, _: &SelectHome, window, cx| {
                            input.navigate(Navigation::Home, true, window, cx)
                        }))
                        .on_action(cx.listener(|input, _: &SelectEnd, window, cx| {
                            input.navigate(Navigation::End, true, window, cx)
                        }))
                        .on_action(cx.listener(|input, _: &SelectAll, window, cx| {
                            input.select_all(window, cx)
                        }))
                        .on_action(cx.listener(|input, _: &Copy, _, cx| input.copy(cx)))
                        .on_action(cx.listener(|input, _: &Cut, window, cx| input.cut(window, cx)))
                        .on_action(cx.listener(|input, _: &Paste, window, cx| {
                            input.paste_clipboard(window, cx)
                        }))
                        .on_action(
                            cx.listener(|input, _: &Submit, window, cx| input.submit(window, cx)),
                        )
                })
                .child(InputLine { input: cx.entity() });

        div()
            .flex()
            .flex_col()
            .min_w_0()
            .w_full()
            .gap(theme.spacing.xs)
            .font_family(theme.font_family.clone())
            .text_size(theme.font_size)
            .line_height(theme.font_size * 1.5)
            .text_color(theme.colors.text)
            .when(!self.label.is_empty(), |column| {
                column.child(
                    div()
                        .text_color(theme.colors.muted)
                        .child(self.label.clone()),
                )
            })
            .child(field)
    }
}

fn single_line(text: &str) -> Option<Cow<'_, str>> {
    if text.contains('\0') {
        return None;
    }
    if !text.contains(['\r', '\n', '\u{2028}', '\u{2029}']) {
        return Some(Cow::Borrowed(text));
    }
    let mut result = String::with_capacity(text.len());
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '\r' => {
                if characters.peek() == Some(&'\n') {
                    characters.next();
                }
                result.push(' ');
            }
            '\n' | '\u{2028}' | '\u{2029}' => result.push(' '),
            _ => result.push(character),
        }
    }
    Some(Cow::Owned(result))
}

fn utf16_to_byte(text: &str, offset: usize, round_up: bool) -> usize {
    let mut utf16 = 0;
    for (byte, character) in text.char_indices() {
        if utf16 >= offset {
            return byte;
        }
        utf16 += character.len_utf16();
        if utf16 > offset {
            return byte + if round_up { character.len_utf8() } else { 0 };
        }
    }
    text.len()
}

fn byte_to_utf16(text: &str, offset: usize) -> usize {
    text.char_indices()
        .take_while(|(byte, _)| *byte < offset)
        .map(|(_, character)| character.len_utf16())
        .sum()
}

fn range_from_utf16(text: &str, range: Range<usize>) -> Range<usize> {
    let start = range.start.min(range.end);
    let end = range.start.max(range.end);
    let start_byte = utf16_to_byte(text, start, false);
    // An insertion inside a surrogate pair is clamped, never expanded into a deletion.
    let end_byte = if start == end {
        start_byte
    } else {
        utf16_to_byte(text, end, true)
    };
    start_byte..end_byte
}

fn range_to_utf16(text: &str, range: &Range<usize>) -> Range<usize> {
    byte_to_utf16(text, range.start)..byte_to_utf16(text, range.end)
}

fn previous_grapheme(text: &str, offset: usize) -> usize {
    text.grapheme_indices(true)
        .rev()
        .find_map(|(byte, _)| (byte < offset).then_some(byte))
        .unwrap_or(0)
}

fn next_grapheme(text: &str, offset: usize) -> usize {
    text.grapheme_indices(true)
        .find_map(|(byte, _)| (byte > offset).then_some(byte))
        .unwrap_or(text.len())
}

fn grapheme_floor(text: &str, offset: usize) -> usize {
    if offset >= text.len() {
        return text.len();
    }
    text.grapheme_indices(true)
        .rev()
        .find_map(|(byte, _)| (byte <= offset).then_some(byte))
        .unwrap_or(0)
}

fn grapheme_ceil(text: &str, offset: usize) -> usize {
    text.grapheme_indices(true)
        .find_map(|(byte, _)| (byte >= offset).then_some(byte))
        .unwrap_or(text.len())
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod tests;
