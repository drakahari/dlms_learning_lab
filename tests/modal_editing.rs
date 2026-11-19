use dlms_learning_lab::modal_editing::{Editor, Key, Mode};
#[test]
fn first_ten_file_and_cursor_boundaries_match_the_reviewed_vim92_tiny_observations() {
    // Actual fixed private-PTY observation R10,13Nov2026; no host editor runs here.
    // Cursor was measured by separate identical-key marker replay, not expressions.
    let cases = [
        ("", "", "wq", "", (0, 0)),
        ("\n", "", "wq", "\n", (0, 0)),
        ("abc\n", "dd", "wq", "", (0, 0)),
        ("x\n", "x", "wq", "\n", (0, 0)),
        ("abc\n", "iX\x1b", "wq", "Xabc\n", (0, 0)),
        ("abc\n", "li\x1b", "wq", "abc\n", (0, 0)),
        ("abc\n", "la\x1b", "wq", "abc\n", (0, 1)),
        ("abc\n", "iHello\x1bu", "wq", "abc\n", (0, 0)),
        ("", "o\x1b", "wq", "\n\n", (1, 0)),
        ("abc\n", "iX\x1b", "q!", "abc\n", (0, 0)),
    ];
    for (before, keys, finish, written, cursor) in cases {
        let mut editor = Editor::new(before).unwrap();
        editor.replay(keys).unwrap();
        assert_eq!(editor.cursor().unwrap(), cursor, "{before:?} {keys:?}");
        editor.replay(&format!(":{finish}\r")).unwrap();
        assert_eq!(editor.file_bytes(), written, "{before:?} {keys:?}");
        assert!(editor.closed());
    }
}
#[test]
fn insert_draft_is_separate_and_writing_then_undoing_does_not_rewrite_virtual_file() {
    let mut e = Editor::new("abc\n").unwrap();
    e.replay("iX\x1b").unwrap();
    assert_eq!(e.mode(), Mode::Normal);
    assert_eq!(e.file_bytes(), "abc\n");
    assert!(e.modified());
    e.replay(":w\r").unwrap();
    assert_eq!(e.file_bytes(), "Xabc\n");
    assert!(!e.modified());
    e.replay("u").unwrap();
    assert_eq!(e.contents(), "abc\n");
    assert_eq!(e.file_bytes(), "Xabc\n");
    assert!(e.modified());
    e.replay(":q").unwrap();
    let before = e.clone();
    assert!(e.key(Key::Enter).is_err());
    assert_eq!(e, before);
    e.key(Key::Escape).unwrap();
    e.replay(":q!\r").unwrap();
    assert_eq!(e.file_bytes(), "Xabc\n");
}
#[test]
fn malformed_or_unsupported_syntax_cannot_execute_files_or_hide_prior_accepted_edits() {
    let mut e = Editor::new("abc\n").unwrap();
    e.replay("iX\x1b:!shell").unwrap();
    let before = e.clone();
    assert!(e.key(Key::Enter).unwrap_err().contains("Unsupported"));
    assert_eq!(e, before);
    assert_eq!(e.file_bytes(), "abc\n");
    assert_eq!(e.contents(), "Xabc\n");
    e.key(Key::Escape).unwrap();
    e.replay("d").unwrap();
    let before = e.clone();
    assert!(e.replay("w").is_err());
    assert_eq!(e, before);
    e.key(Key::Escape).unwrap();
    assert!(e.replay("é").is_err());
}
#[test]
fn empty_buffer_insert_and_blank_line_deletion_keep_distinct_file_bytes() {
    let mut e = Editor::new("").unwrap();
    e.replay("i\x1b:wq\r").unwrap();
    assert_eq!(e.file_bytes(), "");
    let mut e = Editor::new("").unwrap();
    e.replay("iX\x1bx:wq\r").unwrap();
    assert_eq!(e.file_bytes(), "\n");
    let mut e = Editor::new("\n").unwrap();
    e.replay("dd:wq\r").unwrap();
    assert_eq!(e.file_bytes(), "");
}
#[test]
fn partial_modes_resume_and_invalid_mode_group_cursor_or_source_shape_are_rejected() {
    let mut e = Editor::new("abc\n").unwrap();
    e.replay("aXY").unwrap();
    let raw = serde_json::to_string(&e).unwrap();
    let mut restored: Editor = serde_json::from_str(&raw).unwrap();
    restored.validate().unwrap();
    assert_eq!(restored, e);
    restored.replay("\x1bu").unwrap();
    assert_eq!(restored.contents(), "abc\n");
    let mut value = serde_json::to_value(&e).unwrap();
    value["mode"] = serde_json::json!("normal");
    let invalid: Editor = serde_json::from_value(value).unwrap();
    assert!(invalid.validate().is_err());
    e.key(Key::Escape).unwrap();
    let mut value = serde_json::to_value(e).unwrap();
    value["file_bytes"] = serde_json::json!("forged");
    let invalid: Editor = serde_json::from_value(value).unwrap();
    assert!(invalid.validate().is_err());
}

#[test]
fn navigation_register_and_search_match_the_separate_actual_vim_profile() {
    // R20 actual21 fixed recipes; original foundation10 evidence remains distinct.
    let cases = [
        (
            "vertical-column",
            "abcdef\nx\nuvwxyz\n",
            "lllljj",
            "abcdef\nx\nuvwxyz\n",
            (2, 4),
        ),
        (
            "vertical-end",
            "abc\nx\nuvwxyz\n",
            "$jj",
            "abc\nx\nuvwxyz\n",
            (2, 5),
        ),
        (
            "vertical-reset",
            "abcdef\nx\nuvwxyz\n",
            "llllj0j",
            "abcdef\nx\nuvwxyz\n",
            (2, 0),
        ),
        (
            "vertical-left-reset",
            "abcdef\nx\nuvwxyz\n",
            "lllljhj",
            "abcdef\nx\nuvwxyz\n",
            (2, 4),
        ),
        (
            "jump-first",
            "  abc\nx\n  last\n",
            "Ggg",
            "  abc\nx\n  last\n",
            (0, 2),
        ),
        (
            "jump-last",
            "abc\nx\n  last\n",
            "G",
            "abc\nx\n  last\n",
            (2, 2),
        ),
        ("copy-after", "abc\ndef\n", "yyp", "abc\nabc\ndef\n", (1, 0)),
        (
            "copy-before",
            "abc\ndef\n",
            "jyyP",
            "abc\ndef\ndef\n",
            (1, 0),
        ),
        ("move-line", "abc\ndef\n", "ddp", "def\nabc\n", (1, 0)),
        ("restore-only-line", "abc\n", "ddp", "\nabc\n", (1, 0)),
        ("copy-empty-line", "\nabc\n", "yyp", "\n\nabc\n", (1, 0)),
        ("put-character-after", "abc\n", "xp", "bac\n", (0, 1)),
        ("put-character-before", "abc\n", "xP", "abc\n", (0, 0)),
        ("undo-put", "abc\ndef\n", "yypu", "abc\ndef\n", (0, 0)),
        (
            "put-survives-undo",
            "abc\ndef\n",
            "ddup",
            "abc\nabc\ndef\n",
            (1, 0),
        ),
        (
            "search-forward",
            "red blue\nred green\n",
            "/red\r",
            "red blue\nred green\n",
            (1, 0),
        ),
        (
            "search-repeat",
            "red blue\nred green\n",
            "/red\rn",
            "red blue\nred green\n",
            (0, 0),
        ),
        (
            "search-backward",
            "red blue\nred green\n",
            "G?red\r",
            "red blue\nred green\n",
            (0, 0),
        ),
        (
            "search-wrap",
            "red blue\nred green\n",
            "?red\r",
            "red blue\nred green\n",
            (1, 0),
        ),
        ("search-overlap", "aaa\n", "/aa\rn", "aaa\n", (0, 0)),
        (
            "search-repeat-empty",
            "red blue\nred green\n",
            "/red\r/\r",
            "red blue\nred green\n",
            (0, 0),
        ),
    ];
    for (id, before, keys, written, cursor) in cases {
        let mut e = Editor::new(before).unwrap();
        e.replay(keys).unwrap();
        assert_eq!(e.cursor().unwrap(), cursor, "{id}");
        e.replay(":wq\r").unwrap();
        assert_eq!(e.file_bytes(), written, "{id}");
    }
}
#[test]
fn search_errors_and_pending_operations_are_honest_and_cannot_mutate_written_data() {
    let mut e = Editor::new("red blue\nred green\n").unwrap();
    e.replay("/missing\r").unwrap();
    assert!(e.notice().contains("not found"));
    assert_eq!(e.mode(), Mode::Normal);
    assert_eq!(e.cursor().unwrap(), (0, 0));
    e.replay("/r.*").unwrap();
    let before = e.clone();
    assert!(
        e.key(Key::Enter)
            .unwrap_err()
            .contains("Regular expressions")
    );
    assert_eq!(before, e);
    e.key(Key::Escape).unwrap();
    e.replay("y").unwrap();
    let before = e.clone();
    assert!(e.replay("w").is_err());
    assert_eq!(before, e);
    e.key(Key::Escape).unwrap();
    assert!(!e.modified());
    assert_eq!(e.file_bytes(), "red blue\nred green\n");
    let mut e = Editor::new("  a\nb\n  c\n").unwrap();
    e.replay("y").unwrap();
    let raw = serde_json::to_string(&e).unwrap();
    let mut restored: Editor = serde_json::from_str(&raw).unwrap();
    restored.validate().unwrap();
    restored.replay("ypG?b\r").unwrap();
    assert_eq!(restored.cursor().unwrap(), (2, 0));
}
#[test]
fn forged_search_register_and_motion_state_refuse_without_panics() {
    let e = Editor::new("abc\n").unwrap();
    for (field, value) in [
        (
            "goal",
            serde_json::json!({"kind":"Column","value":usize::MAX}),
        ),
        (
            "register",
            serde_json::json!({"text":"abc","linewise":true}),
        ),
        ("search", serde_json::json!({"text":".*","forward":true})),
        ("pending", serde_json::json!("delete")),
    ] {
        let mut value_e = serde_json::to_value(&e).unwrap();
        value_e[field] = value;
        if field == "pending" {
            value_e["mode"] = serde_json::json!("command");
        }
        let invalid: Editor = serde_json::from_value(value_e).unwrap();
        assert!(invalid.validate().is_err(), "{field}");
    }
}

#[test]
fn insertion_correction_matches_eight_actual_vim_recipes() {
    let cases = [
        ("typed-backspace", "abc\n", "iXY\x08\x1b", "Xabc\n", (0, 0)),
        (
            "backspace-insert-start",
            "abc\n",
            "li\x08\x1b",
            "bc\n",
            (0, 0),
        ),
        (
            "backspace-newline",
            "abc\n",
            "aX\rY\x08\x08\x1b",
            "aXbc\n",
            (0, 1),
        ),
        ("command-backspace", "abc\n", ":wx\x08\r", "abc\n", (0, 0)),
        ("put-into-blank", "x\n", "xpp", "xx\n", (0, 1)),
        ("line-before-empty", "abc\n", "ddP", "abc\n\n", (0, 0)),
        ("open-backspace", "abc\n", "oX\x08\x1b", "abc\n\n", (1, 0)),
        (
            "undo-erased-insert",
            "abc\n",
            "iX\x08\x1bu",
            "abc\n",
            (0, 0),
        ),
    ];
    for (id, before, keys, written, cursor) in cases {
        let mut e = Editor::new(before).unwrap();
        e.replay(keys).unwrap();
        assert_eq!(e.cursor().unwrap(), cursor, "{id}");
        e.replay(":wq\r").unwrap();
        assert_eq!(e.file_bytes(), written, "{id}");
    }
}
#[test]
fn paste_is_bounded_insert_data_and_never_interprets_keys_or_escapes() {
    let mut e = Editor::new("abc\n").unwrap();
    let before = e.clone();
    assert!(e.insert_text("xx").is_err());
    assert_eq!(e, before);
    e.replay("i").unwrap();
    e.insert_text(":!shell\ndd").unwrap();
    assert_eq!(e.contents(), ":!shell\nddabc\n");
    assert_eq!(e.file_bytes(), "abc\n");
    let before = e.clone();
    for bad in ["\x1b", "\t", "é"] {
        assert!(e.insert_text(bad).is_err());
        assert_eq!(e, before);
    }
    assert!(e.insert_text(&"x".repeat(16384)).is_err());
    assert_eq!(e, before);
    e.replay("\x1bu").unwrap();
    assert_eq!(e.contents(), "abc\n");
}
