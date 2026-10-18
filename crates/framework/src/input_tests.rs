use super::*;

#[test]
fn sanitization_preserves_unicode_and_collapses_crlf_once() {
    assert_eq!(single_line("é🙂漢").as_deref(), Some("é🙂漢"));
    assert_eq!(
        single_line("a\r\nb\nc\rd\u{2028}e\u{2029}f").as_deref(),
        Some("a b c d e f")
    );
    assert_eq!(single_line("a\r\n\r\nb").as_deref(), Some("a  b"));
    assert!(single_line("valid\0invalid\n").is_none());
}

#[test]
fn utf16_ranges_clamp_without_splitting_scalars_or_deleting_at_an_insertion() {
    let text = "A🙂e\u{301}漢";
    assert_eq!(range_from_utf16(text, 1..3), 1..5);
    assert_eq!(range_from_utf16(text, 2..3), 1..5);
    assert_eq!(range_from_utf16(text, 2..2), 1..1);
    assert_eq!(
        range_from_utf16(text, 99..usize::MAX),
        text.len()..text.len()
    );
    assert_eq!(
        range_from_utf16(text, std::ops::Range { start: 5, end: 1 }),
        1..8
    );
    assert_eq!(range_to_utf16(text, &(5..8)), 3..5);
    for (byte, _) in text
        .char_indices()
        .chain(std::iter::once((text.len(), '\0')))
    {
        assert_eq!(utf16_to_byte(text, byte_to_utf16(text, byte), false), byte);
    }
}

#[test]
fn grapheme_navigation_keeps_combining_marks_flags_and_zwj_sequences_together() {
    let clusters = ["A", "e\u{301}", "🇮🇳", "👩‍👩‍👧‍👦", "漢"];
    let text = clusters.concat();
    let mut offset = 0;
    for cluster in clusters {
        let end = offset + cluster.len();
        assert_eq!(next_grapheme(&text, offset), end);
        assert_eq!(previous_grapheme(&text, end), offset);
        assert_eq!(grapheme_floor(&text, offset), offset);
        assert_eq!(grapheme_ceil(&text, offset), offset);
        for interior in offset + 1..end {
            assert_eq!(grapheme_floor(&text, interior), offset);
            assert_eq!(grapheme_ceil(&text, interior), end);
        }
        offset = end;
    }
    assert_eq!(previous_grapheme(&text, 0), 0);
    assert_eq!(next_grapheme(&text, text.len()), text.len());
}

#[test]
fn caret_scrolling_follows_both_ends_and_recovers_after_resize_or_shortening() {
    assert_eq!(caret_scroll(px(0.), px(200.), px(200.), px(50.)), px(151.));
    assert_eq!(caret_scroll(px(151.), px(0.), px(200.), px(50.)), px(0.));
    assert_eq!(caret_scroll(px(151.), px(200.), px(200.), px(500.)), px(0.));
    assert_eq!(caret_scroll(px(151.), px(10.), px(10.), px(50.)), px(0.));
    assert_eq!(caret_scroll(px(0.), px(200.), px(200.), px(0.)), px(200.));
}

#[cfg(all(test, feature = "test-support"))]
mod native {
    use super::*;
    use gpui::{Empty, InputHandler, TestAppContext, WindowHandle};
    use std::{cell::RefCell, rc::Rc};

    fn setup(cx: &mut TestAppContext, value: &str) -> (WindowHandle<Empty>, Entity<TextInput>) {
        // Keeping the test window empty exercises editing without depending on application theme setup.
        let window = cx.add_window(|_, _| Empty);
        let input = window
            .update(cx, |_, window, cx| {
                cx.new(|cx| TextInput::new(value.to_owned(), window, cx))
            })
            .expect("test window exists");
        (window, input)
    }

    fn update<R>(
        cx: &mut TestAppContext,
        window: WindowHandle<Empty>,
        input: &Entity<TextInput>,
        callback: impl FnOnce(&mut TextInput, &mut Window, &mut Context<TextInput>) -> R,
    ) -> R {
        window
            .update(cx, |_, window, cx| {
                input.update(cx, |input, cx| callback(input, window, cx))
            })
            .expect("test window exists")
    }

    #[gpui::test]
    fn construction_and_programmatic_updates_do_not_steal_focus(cx: &mut TestAppContext) {
        let (window, input) = setup(cx, "a\r\nb");
        update(cx, window, &input, |input, window, cx| {
            assert!(!input.focus_handle.is_focused(window));
            assert_eq!(input.text(), "a b");
            input.set_selected_text_range(0..1, window, cx);
            input.set_text("a b", window, cx);
            assert_eq!(input.selected_range, 0..1);
            input.set_text("bad\0value", window, cx);
            assert_eq!(input.text(), "a b");
            assert_eq!(input.selected_range, 0..1);
            input.set_text("é\n🙂", window, cx);
            assert_eq!(input.text(), "é 🙂");
            assert_eq!(input.selected_range, 7..7);
            assert!(!input.focus_handle.is_focused(window));
        });
    }

    #[gpui::test]
    fn native_handler_uses_typed_utf16_selection_and_replacement(cx: &mut TestAppContext) {
        let (window, input) = setup(cx, "A🙂漢");
        let mut handler = ElementInputHandler::new(
            Bounds::new(point(px(10.), px(10.)), size(px(100.), px(24.))),
            input.clone(),
        );
        window
            .update(cx, |_, window, cx| {
                assert_eq!(handler.text_length_utf16(window, cx), Some(4));
                handler.set_selected_text_range(1..3, window, cx);
                let selection: UTF16Selection = handler
                    .selected_text_range(false, window, cx)
                    .expect("enabled input has a selection");
                assert_eq!(selection.range, 1..3);
                assert!(!selection.reversed);
                let mut actual = None;
                assert_eq!(
                    handler.text_for_range(2..3, &mut actual, window, cx),
                    Some("🙂".into())
                );
                assert_eq!(actual, Some(1..3));
                handler.replace_text_in_range(None, "é", window, cx);
            })
            .expect("test window exists");
        update(cx, window, &input, |input, window, cx| {
            assert_eq!(input.text(), "Aé漢");
            assert_eq!(input.selected_range, 3..3);
            input.set_selected_text_range(99..usize::MAX, window, cx);
            assert_eq!(input.selected_range, 6..6);
            input.replace_text_in_range(Some(usize::MAX..usize::MAX), "!", window, cx);
            assert_eq!(input.text(), "Aé漢!");
        });
    }

    #[gpui::test]
    fn keyboard_deletion_removes_whole_graphemes_in_both_directions(cx: &mut TestAppContext) {
        let clusters = ["A", "e\u{301}", "🇮🇳", "👩‍👩‍👧‍👦", "漢"];
        let (window, input) = setup(cx, &clusters.concat());
        update(cx, window, &input, |input, window, cx| {
            for remaining in (0..clusters.len()).rev() {
                input.delete(true, window, cx);
                assert_eq!(input.text().as_ref(), clusters[..remaining].concat());
            }
            input.delete(true, window, cx);
            assert_eq!(input.text(), "");
            input.set_text(clusters.concat(), window, cx);
            input.navigate(Navigation::Home, false, window, cx);
            for removed in 1..=clusters.len() {
                input.delete(false, window, cx);
                assert_eq!(input.text().as_ref(), clusters[removed..].concat());
            }
            input.set_text("e\u{301}", window, cx);
            input.set_selected_text_range(1..1, window, cx);
            input.delete(true, window, cx);
            assert_eq!(input.text(), "");
            input.set_text("e\u{301}", window, cx);
            input.set_selected_text_range(1..2, window, cx);
            input.delete(false, window, cx);
            assert_eq!(input.text(), "e");
        });
    }

    #[gpui::test]
    fn shift_selection_keeps_its_anchor_when_crossing_it(cx: &mut TestAppContext) {
        let (window, input) = setup(cx, "A🙂Z");
        update(cx, window, &input, |input, window, cx| {
            input.navigate(Navigation::Left, false, window, cx);
            input.navigate(Navigation::Left, true, window, cx);
            assert_eq!(input.selected_range, 1..5);
            assert!(
                input
                    .selected_text_range(false, window, cx)
                    .expect("selection")
                    .reversed
            );
            input.navigate(Navigation::Right, true, window, cx);
            assert_eq!(input.selected_range, 5..5);
            input.navigate(Navigation::Right, true, window, cx);
            assert_eq!(input.selected_range, 5..6);
            assert!(!input.selection_reversed);
            input.navigate(Navigation::Left, false, window, cx);
            assert_eq!(input.selected_range, 5..5);
            input.navigate(Navigation::Home, true, window, cx);
            assert_eq!(input.selected_range, 0..5);
            input.navigate(Navigation::End, true, window, cx);
            assert_eq!(input.selected_range, 5..6);
            input.select_all(window, cx);
            assert_eq!(input.selected_range, 0..6);
        });
    }

    #[gpui::test]
    fn ime_selection_is_relative_and_repeated_marking_replaces_the_composition(
        cx: &mut TestAppContext,
    ) {
        let (window, input) = setup(cx, "A🙂Z");
        update(cx, window, &input, |input, window, cx| {
            input.replace_and_mark_text_in_range(Some(1..3), "é🙂", Some(1..3), window, cx);
            assert_eq!(input.text(), "Aé🙂Z");
            assert_eq!(input.marked_text_range(window, cx), Some(1..4));
            assert_eq!(input.selected_range, 3..7);
            assert_eq!(
                input
                    .selected_text_range(false, window, cx)
                    .expect("selection")
                    .range,
                2..4
            );
            input.replace_and_mark_text_in_range(None, "bad\0value", Some(0..0), window, cx);
            assert_eq!(input.text(), "Aé🙂Z");
            assert_eq!(input.selected_range, 3..7);
            assert_eq!(input.marked_range, Some(1..7));
            input.replace_and_mark_text_in_range(None, "漢", Some(1..1), window, cx);
            assert_eq!(input.text(), "A漢Z");
            assert_eq!(input.selected_range, 4..4);
            input.replace_text_in_range(None, "字", window, cx);
            assert_eq!(input.text(), "A字Z");
            assert_eq!(input.marked_range, None);
            input.replace_and_mark_text_in_range(Some(1..2), "a\r\n🙂", Some(3..5), window, cx);
            assert_eq!(input.text(), "Aa 🙂Z");
            assert_eq!(input.selected_range, 3..7);
            input.replace_and_mark_text_in_range(None, "", Some(0..0), window, cx);
            assert_eq!(input.text(), "AZ");
            assert_eq!(input.marked_range, None);
            assert_eq!(input.selected_range, 1..1);
        });
    }

    #[gpui::test]
    fn native_geometry_and_hit_testing_include_horizontal_scroll(cx: &mut TestAppContext) {
        let (window, input) = setup(cx, "A🙂Z");
        update(cx, window, &input, |input, window, cx| {
            let style = window.text_style();
            let line = window.text_system().shape_line(
                input.text(),
                style.font_size.to_pixels(window.rem_size()),
                &[TextRun {
                    len: input.content.len(),
                    font: style.font(),
                    color: style.color,
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                }],
                None,
            );
            let bounds = Bounds::new(
                point(px(20.), px(30.)),
                size(line.width() + px(10.), px(24.)),
            );
            let scroll = line.x_for_index(1);
            let expected_end = bounds.left() + line.x_for_index(5) - scroll;
            input.scroll_offset = scroll;
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
            let rectangles = input.selection_bounds_for_range(1..3, bounds, window, cx);
            assert_eq!(rectangles.len(), 1);
            assert_eq!(rectangles[0].left(), bounds.left());
            assert_eq!(rectangles[0].right(), expected_end);
            let caret = input
                .bounds_for_range(3..3, bounds, window, cx)
                .expect("shaped caret");
            assert_eq!(caret.left(), expected_end);
            assert_eq!(caret.size.width, px(1.));
            assert_eq!(
                input.character_index_for_point(
                    point(bounds.left(), bounds.top() + px(1.)),
                    window,
                    cx
                ),
                Some(1)
            );
            assert_eq!(
                input.index_for_mouse_position(point(bounds.left() - px(1_000.), bounds.top())),
                0
            );
            assert_eq!(
                input.index_for_mouse_position(point(bounds.right() + px(1_000.), bounds.top())),
                6
            );
            input.on_mouse_down(
                &MouseDownEvent {
                    button: MouseButton::Left,
                    position: point(bounds.left(), bounds.top() + px(1.)),
                    click_count: 1,
                    ..Default::default()
                },
                window,
                cx,
            );
            assert!(input.focus_handle.is_focused(window));
            assert_eq!(input.selected_range, 1..1);
            input.on_mouse_move(
                &MouseMoveEvent {
                    position: point(bounds.left() - px(1_000.), bounds.top()),
                    pressed_button: Some(MouseButton::Left),
                    ..Default::default()
                },
                window,
                cx,
            );
            assert_eq!(input.selected_range, 0..1);
            assert!(input.selection_reversed);
            input.on_mouse_move(
                &MouseMoveEvent {
                    position: point(bounds.right() + px(1_000.), bounds.top()),
                    pressed_button: Some(MouseButton::Left),
                    ..Default::default()
                },
                window,
                cx,
            );
            assert_eq!(input.selected_range, 1..6);
            assert!(!input.selection_reversed);
            input.on_mouse_move(&MouseMoveEvent::default(), window, cx);
            assert!(!input.is_selecting);
            input.set_text("changed", window, cx);
            assert!(input.bounds_for_range(0..1, bounds, window, cx).is_none());
        });
    }

    #[gpui::test]
    fn clipboard_and_native_paste_share_atomic_sanitization(cx: &mut TestAppContext) {
        let (window, input) = setup(cx, "A🙂Z");
        update(cx, window, &input, |input, window, cx| {
            input.set_selected_text_range(1..3, window, cx);
            input.copy(cx);
            assert_eq!(
                cx.read_from_clipboard()
                    .and_then(|item| item.text())
                    .as_deref(),
                Some("🙂")
            );
            input.cut(window, cx);
            assert_eq!(input.text(), "AZ");
            input.paste_clipboard(window, cx);
            assert_eq!(input.text(), "A🙂Z");
            input.select_all(window, cx);
            cx.write_to_clipboard(ClipboardItem::new_string("invalid\0paste".into()));
            input.paste_clipboard(window, cx);
            assert_eq!(input.text(), "A🙂Z");
            assert_eq!(input.selected_range, 0..6);
            EntityInputHandler::paste(
                input,
                ClipboardItem::new_string("x\r\ny\nz".into()),
                window,
                cx,
            );
            assert_eq!(input.text(), "x y z");
            input.select_all(window, cx);
            EntityInputHandler::paste(input, ClipboardItem::new_string("\r\n".into()), window, cx);
            assert_eq!(input.text(), " ");
        });
    }

    #[gpui::test]
    fn disabled_input_rejects_user_edits_and_selection_but_allows_set_text(
        cx: &mut TestAppContext,
    ) {
        let (window, input) = setup(cx, "A🙂Z");
        update(cx, window, &input, |input, window, cx| {
            input.set_selected_text_range(1..3, window, cx);
            input.set_disabled(true, cx);
            input.replace_text_in_range(None, "bad", window, cx);
            input.replace_and_mark_text_in_range(None, "bad", None, window, cx);
            input.set_selected_text_range(0..0, window, cx);
            input.navigate(Navigation::End, false, window, cx);
            input.select_all(window, cx);
            input.delete(true, window, cx);
            input.cut(window, cx);
            input.paste_clipboard(window, cx);
            input.submit(window, cx);
            assert_eq!(input.text(), "A🙂Z");
            assert_eq!(input.selected_range, 1..5);
            assert!(!input.accepts_text_input(window, cx));
            assert!(input.selected_text_range(false, window, cx).is_none());
            assert_eq!(
                input
                    .selected_text_range(true, window, cx)
                    .expect("inspect disabled input")
                    .range,
                1..3
            );
            input.set_text("updated", window, cx);
            assert_eq!(input.text(), "updated");
            input.set_disabled(false, cx);
            input.replace_text_in_range(None, "!", window, cx);
            assert_eq!(input.text(), "updated!");
        });
    }

    #[gpui::test]
    fn changed_and_enter_events_are_observable_and_composition_enter_does_not_submit(
        cx: &mut TestAppContext,
    ) {
        let (window, input) = setup(cx, "initial");
        let events = Rc::new(RefCell::new(Vec::new()));
        let _subscription = cx.update(|cx| {
            let events = events.clone();
            cx.subscribe(&input, move |input, event: &InputEvent, cx| {
                events
                    .borrow_mut()
                    .push((event.clone(), input.read(cx).text()));
            })
        });
        update(cx, window, &input, |input, window, cx| {
            input.set_text("initial", window, cx);
            input.set_text("invalid\0value", window, cx);
            input.set_text("updated", window, cx);
        });
        assert_eq!(events.borrow().len(), 1);
        assert!(matches!(events.borrow()[0].0, InputEvent::Changed));
        assert_eq!(events.borrow()[0].1, "updated");
        update(cx, window, &input, |input, window, cx| {
            input.replace_text_in_range(None, "\n", window, cx)
        });
        assert!(matches!(events.borrow()[1].0, InputEvent::Submitted));
        assert_eq!(events.borrow()[1].1, "updated");
        update(cx, window, &input, |input, window, cx| {
            input.replace_and_mark_text_in_range(None, "漢", None, window, cx);
        });
        assert_eq!(events.borrow().len(), 3);
        update(cx, window, &input, |input, window, cx| {
            input.replace_text_in_range(None, "\r", window, cx)
        });
        assert_eq!(events.borrow().len(), 3);
        update(cx, window, &input, |input, window, cx| {
            input.submit(window, cx)
        });
        assert!(matches!(events.borrow()[3].0, InputEvent::Submitted));
        update(cx, window, &input, |input, window, cx| {
            input.set_disabled(true, cx);
            input.submit(window, cx);
            input.replace_text_in_range(None, "no", window, cx);
        });
        assert_eq!(events.borrow().len(), 4);
    }
}
