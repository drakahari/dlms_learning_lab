use dlms_learning_lab::modal_text::{Buffer, MAX_TEXT, MAX_UNDO};
#[test]
fn editing_is_separate_from_written_virtual_bytes_and_undo_never_rewrites_them() {
    let mut b = Buffer::new("mode=old\nkeep=yes\n").unwrap();
    b.seek(5).unwrap();
    b.replace(5..8, "new").unwrap();
    assert_eq!(b.contents(), "mode=new\nkeep=yes\n");
    assert_eq!(b.written(), "mode=old\nkeep=yes\n");
    assert!(b.differs_from_written());
    b.mark_written().unwrap();
    b.undo().unwrap();
    assert_eq!(b.contents(), "mode=old\nkeep=yes\n");
    assert_eq!(b.written(), "mode=new\nkeep=yes\n");
    assert!(b.differs_from_written());
}
#[test]
fn a_complete_insert_group_undoes_once_despite_separate_events_and_cursor_moves() {
    let mut b = Buffer::new("alpha\nbeta\n").unwrap();
    b.seek(5).unwrap();
    b.begin_group().unwrap();
    b.insert(" first").unwrap();
    b.insert(" second").unwrap();
    b.seek(0).unwrap();
    b.finish_group().unwrap();
    b.undo().unwrap();
    assert_eq!(b.contents(), "alpha\nbeta\n");
    assert_eq!(b.cursor(), 5);
    let before = b.clone();
    assert!(b.undo().is_err());
    assert_eq!(before, b);
}
#[test]
fn empty_and_final_newline_are_data_not_silently_normalized_by_the_primitive() {
    for text in ["", "\n", "a", "a\n", "a\n\n"] {
        let mut b = Buffer::new(text).unwrap();
        b.seek(text.len()).unwrap();
        let before = b.clone();
        b.mark_written().unwrap();
        assert_eq!(b, before);
        let json = serde_json::to_string(&b).unwrap();
        assert_eq!(Buffer::from_json(&json).unwrap(), b);
    }
    let mut b = Buffer::new("a\n").unwrap();
    b.replace(0..2, "").unwrap();
    assert_eq!(b.contents(), "");
    b.undo().unwrap();
    assert_eq!(b.contents(), "a\n");
}
#[test]
fn unsupported_text_invalid_ranges_and_size_limit_reject_atomically() {
    let base = Buffer::new("report\n").unwrap();
    for text in ["\t", "é", "\r", "\0", "\u{1b}"] {
        let mut b = base.clone();
        assert!(b.insert(text).is_err());
        assert_eq!(b, base);
    }
    for range in [8..8, 0..8] {
        let mut b = base.clone();
        assert!(b.replace(range, "x").is_err());
        assert_eq!(b, base);
    }
    let mut b = Buffer::new(&"x".repeat(MAX_TEXT)).unwrap();
    let before = b.clone();
    assert!(b.insert("x").is_err());
    assert_eq!(b, before);
    assert!(Buffer::new(&"x".repeat(MAX_TEXT + 1)).is_err());
}
#[test]
fn partial_edit_groups_resume_and_cannot_write_or_undo_before_finishing() {
    let mut b = Buffer::new("hello\n").unwrap();
    b.begin_group().unwrap();
    b.insert("new ").unwrap();
    let json = serde_json::to_string(&b).unwrap();
    let mut restored = Buffer::from_json(&json).unwrap();
    assert_eq!(restored, b);
    let before = restored.clone();
    assert!(restored.mark_written().is_err());
    assert!(restored.undo().is_err());
    assert!(restored.begin_group().is_err());
    assert_eq!(restored, before);
    restored.finish_group().unwrap();
    restored.undo().unwrap();
    assert_eq!(restored.contents(), "hello\n");
}
#[test]
fn history_is_bounded_and_saved_imports_validate_cursor_and_shape() {
    let mut b = Buffer::new("").unwrap();
    for _ in 0..MAX_UNDO + 3 {
        b.insert("x").unwrap();
    }
    for _ in 0..MAX_UNDO {
        b.undo().unwrap();
    }
    assert_eq!(b.contents(), "xxx");
    assert!(b.undo().is_err());
    let mut value = serde_json::to_value(&b).unwrap();
    value["current"]["cursor"] = serde_json::json!(usize::MAX);
    assert!(Buffer::from_json(&value.to_string()).is_err());
    let mut value = serde_json::to_value(&b).unwrap();
    value["written"] = serde_json::json!("x".repeat(MAX_TEXT + 1));
    assert!(Buffer::from_json(&value.to_string()).is_err());
    let mut value = serde_json::to_value(&b).unwrap();
    value["unknown"] = serde_json::json!(true);
    assert!(Buffer::from_json(&value.to_string()).is_err());
}

#[test]
fn cursor_readout_handles_line_boundaries_and_refuses_forged_deserialized_state() {
    let mut b = Buffer::new("abc\nxy\n").unwrap();
    for (offset, expected) in [
        (0, (0, 0)),
        (3, (0, 3)),
        (4, (1, 0)),
        (6, (1, 2)),
        (7, (2, 0)),
    ] {
        b.seek(offset).unwrap();
        assert_eq!(b.position().unwrap(), expected);
    }
    let mut value = serde_json::to_value(b).unwrap();
    value["current"]["cursor"] = serde_json::json!(usize::MAX);
    // Even a caller that bypasses from_json must receive a diagnostic, not a slice panic.
    let forged: Buffer = serde_json::from_value(value).unwrap();
    assert!(forged.position().is_err());
}

#[test]
fn modification_marker_is_not_a_comparison_of_bytes_and_undo_restores_saved_revision() {
    let mut b = Buffer::new("abc\n").unwrap();
    b.begin_group().unwrap();
    b.insert("X").unwrap();
    b.replace(0..1, "").unwrap();
    assert!(!b.differs_from_written());
    assert!(b.modified_since_written());
    b.finish_group().unwrap();
    assert!(b.modified_since_written());
    b.undo().unwrap();
    assert!(!b.modified_since_written());
    b.replace(0..1, "A").unwrap();
    b.mark_written().unwrap();
    assert!(!b.modified_since_written());
    b.undo().unwrap();
    assert!(b.modified_since_written());
    assert_eq!(b.written(), "Abc\n");
    assert_eq!(b.contents(), "abc\n");
}
#[test]
fn restored_modification_state_and_forged_revision_bounds_are_validated() {
    let mut b = Buffer::new("abc\n").unwrap();
    b.insert("X").unwrap();
    let restored = Buffer::from_json(&serde_json::to_string(&b).unwrap()).unwrap();
    assert_eq!(restored, b);
    assert!(restored.modified_since_written());
    for key in ["clock", "written_revision"] {
        let mut value = serde_json::to_value(&b).unwrap();
        value[key] = serde_json::json!(u64::MAX);
        assert!(Buffer::from_json(&value.to_string()).is_err());
    }
    let mut value = serde_json::to_value(&b).unwrap();
    value["group_changed"] = serde_json::json!(true);
    assert!(Buffer::from_json(&value.to_string()).is_err());
}
