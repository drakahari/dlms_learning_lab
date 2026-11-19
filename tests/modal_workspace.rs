use dlms_learning_lab::{
    modal_editing::{Key, Mode},
    modal_workspace::{Attempt, Definition, Event, File, Fixture, Requirement},
};
use std::collections::BTreeMap;
fn keys(a: &mut Attempt, f: &Fixture, keys: &str) -> Result<(), String> {
    for c in keys.chars() {
        a.apply(
            f,
            &Event::Key {
                key: match c {
                    '\x1b' => Key::Escape,
                    '\r' => Key::Enter,
                    '\x08' => Key::Backspace,
                    _ => Key::Character(c),
                },
            },
        )?;
    }
    Ok(())
}
fn fixture() -> Fixture {
    Fixture {
        files: BTreeMap::from([
            (
                "/lab/service.conf".into(),
                File {
                    text: "name=edge-a\nport=80\nworkers=2\n".into(),
                    writable: true,
                    draft: None,
                },
            ),
            (
                "/lab/notes.txt".into(),
                File {
                    text: "Keep this original note.\n".into(),
                    writable: false,
                    draft: None,
                },
            ),
        ]),
        start: "/lab/service.conf".into(),
        requirements: vec![
            Requirement::Settings {
                path: "/lab/service.conf".into(),
                values: BTreeMap::from([
                    ("port".into(), "8080".into()),
                    ("workers".into(), "4".into()),
                ]),
            },
            Requirement::Unchanged {
                path: "/lab/notes.txt".into(),
            },
        ],
    }
}
fn replace(a: &mut Attempt, f: &Fixture, text: &str) {
    let lines = a.editor().contents().lines().count();
    keys(a, f, "gg").unwrap();
    for _ in 0..lines {
        keys(a, f, "dd").unwrap();
    }
    keys(a, f, "i").unwrap();
    a.apply(
        f,
        &Event::Insert {
            text: text.trim_end_matches('\n').into(),
        },
    )
    .unwrap();
    keys(a, f, "\x1b").unwrap();
}
fn definition() -> Definition {
    let mut example = fixture();
    example.files.get_mut("/lab/service.conf").unwrap().text =
        "name=demo-b\nport=3000\nworkers=1\n".into();
    if let Requirement::Settings { values, .. } = &mut example.requirements[0] {
        *values = BTreeMap::from([
            ("port".into(), "9000".into()),
            ("workers".into(), "2".into()),
        ]);
    }
    let mut example_solution = vec![];
    for c in "ggddddddi".chars() {
        example_solution.push(Event::Key {
            key: Key::Character(c),
        });
    }
    example_solution.push(Event::Insert {
        text: "name=demo-b\nport=9000\nworkers=2".into(),
    });
    example_solution.extend([
        Event::Key { key: Key::Escape },
        Event::Key {
            key: Key::Character(':'),
        },
        Event::Key {
            key: Key::Character('w'),
        },
        Event::Key { key: Key::Enter },
    ]);
    Definition {
        version: 1,
        id: "modal-file-repair".into(),
        initial: fixture(),
        example,
        example_solution,
    }
}
#[test]
fn saved_outcomes_accept_alternatives_reject_partial_wrong_and_duplicate_settings() {
    let f = fixture();
    let mut a = Attempt::new(&f).unwrap();
    a.apply(&f, &Event::Check {}).unwrap();
    assert!(!a.solved(&f));
    replace(&mut a, &f, "name=edge-a\nport=8080\nworkers=2");
    keys(&mut a, &f, ":w\r").unwrap();
    a.apply(&f, &Event::Check {}).unwrap();
    assert!(!a.solved(&f));
    for wrong in [
        "name=other\nport=8080\nworkers=4",
        "name=edge-a\nport=8080\nworkers=4\nworkers=2",
        "name=edge-a\nport=8080\nworkers=4\nextra=1",
        "name=edge-a\nport=wrong\nworkers=4",
    ] {
        replace(&mut a, &f, wrong);
        keys(&mut a, &f, ":w\r").unwrap();
        a.apply(&f, &Event::Check {}).unwrap();
        assert!(!a.solved(&f), "{wrong}");
    }
    // Different order, comments and whitespace: outcomes, not literal text/keys.
    replace(
        &mut a,
        &f,
        "# repaired file\nworkers = 4\nport = 8080\nname = edge-a",
    );
    a.apply(&f, &Event::Check {}).unwrap();
    assert!(!a.solved(&f)); // unsaved buffer
    keys(&mut a, &f, ":w\r").unwrap();
    a.apply(&f, &Event::Check {}).unwrap();
    assert!(a.solved(&f));
    keys(&mut a, &f, "ggG/port\r").unwrap();
    assert!(a.solved(&f));
    keys(&mut a, &f, "ix\x1b").unwrap();
    assert!(!a.checked);
    assert!(!a.solved(&f));
    a.apply(&f, &Event::Check {}).unwrap();
    assert!(!a.solved(&f));
}
#[test]
fn actual_line_copy_and_delete_meet_records_but_wrong_copies_and_extra_lines_fail() {
    let f = Fixture {
        files: BTreeMap::from([(
            "/lab/jobs.txt".into(),
            File {
                text: "KEEP archive\nRUN inspect\nRETIRE legacy\n".into(),
                writable: true,
                draft: None,
            },
        )]),
        start: "/lab/jobs.txt".into(),
        requirements: vec![Requirement::Records {
            path: "/lab/jobs.txt".into(),
            counts: BTreeMap::from([("RUN inspect".into(), 2), ("RETIRE legacy".into(), 0)]),
        }],
    };
    let mut a = Attempt::new(&f).unwrap();
    keys(&mut a, &f, "jyypGdd:w\r").unwrap();
    a.apply(&f, &Event::Check {}).unwrap();
    assert!(a.solved(&f));
    for wrong in [
        "KEEP archive\nRUN inspect\nRETIRE legacy",
        "KEEP archive\nRUN inspect",
        "KEEP archive\nRUN inspect\nRUN inspect\nEXTRA",
        "KEEP archive\nKEEP archive\nRUN inspect",
    ] {
        replace(&mut a, &f, wrong);
        keys(&mut a, &f, ":w\r").unwrap();
        a.apply(&f, &Event::Check {}).unwrap();
        assert!(!a.solved(&f));
    }
    replace(&mut a, &f, "RUN inspect\nKEEP archive\nRUN inspect");
    keys(&mut a, &f, ":w\r").unwrap();
    a.apply(&f, &Event::Check {}).unwrap();
    assert!(a.solved(&f));
}
#[test]
fn switching_files_preserves_drafts_and_read_only_writes_and_path_escape_refuse() {
    let f = fixture();
    let mut a = Attempt::new(&f).unwrap();
    keys(&mut a, &f, "iDRAFT\x1b").unwrap();
    let before = a.clone();
    assert!(
        a.apply(
            &f,
            &Event::Open {
                path: "/lab/notes.txt".into()
            }
        )
        .unwrap_err()
        .contains("Save")
    );
    assert_eq!(a, before);
    for path in [
        "/etc/passwd",
        "/lab/../notes.txt",
        "/lab/service.conf/extra",
    ] {
        assert!(a.apply(&f, &Event::Open { path: path.into() }).is_err());
        assert_eq!(a, before);
    }
    keys(&mut a, &f, ":q!\r").unwrap();
    a.apply(
        &f,
        &Event::Open {
            path: "/lab/notes.txt".into(),
        },
    )
    .unwrap();
    keys(&mut a, &f, "iNO\x1b:w").unwrap();
    let before = a.clone();
    assert!(
        a.apply(&f, &Event::Key { key: Key::Enter })
            .unwrap_err()
            .contains("read-only")
    );
    assert_eq!(a, before);
    keys(&mut a, &f, "\x1b:q!\r").unwrap();
    a.apply(
        &f,
        &Event::Open {
            path: "/lab/service.conf".into(),
        },
    )
    .unwrap();
    assert_eq!(a.editor().contents(), f.files["/lab/service.conf"].text);
}
#[test]
fn worked_example_drafts_and_assignment_reset_are_separate_and_never_award_credit() {
    let d = definition();
    d.validate().unwrap();
    let mut s = d.new_session().unwrap();
    s.apply(
        &d,
        false,
        &Event::Key {
            key: Key::Character('i'),
        },
    )
    .unwrap();
    s.apply(
        &d,
        false,
        &Event::Insert {
            text: "DRAFT".into(),
        },
    )
    .unwrap();
    let draft = s.assignment.clone();
    s.show_worked(&d).unwrap();
    assert_eq!(s.assignment, draft);
    assert!(!s.practice.checked);
    assert!(!s.solved(&d));
    s.apply(&d, true, &Event::Check {}).unwrap();
    assert!(s.practice.solved(&d.example));
    assert!(!s.solved(&d));
    s.reset_practice(&d).unwrap();
    assert_eq!(s.assignment, draft);
    assert!(!s.practice.checked);
    s.show_worked(&d).unwrap();
    let worked = s.practice.clone();
    s.reset_assignment(&d).unwrap();
    assert_eq!(s.practice, worked);
    assert_eq!(s.assignment, Attempt::new(&d.initial).unwrap());
}
#[test]
fn partial_insert_search_undo_and_written_data_survive_serialized_resume() {
    let d = definition();
    let mut s = d.new_session().unwrap();
    s.apply(
        &d,
        false,
        &Event::Key {
            key: Key::Character('i'),
        },
    )
    .unwrap();
    s.apply(
        &d,
        false,
        &Event::Insert {
            text: "DRAFT".into(),
        },
    )
    .unwrap();
    let raw = serde_json::to_string(&s).unwrap();
    let mut restored = serde_json::from_str(&raw).unwrap();
    d.context(&restored).unwrap();
    assert_eq!(s, restored);
    s = restored;
    s.apply(&d, false, &Event::Key { key: Key::Escape })
        .unwrap();
    s.apply(
        &d,
        false,
        &Event::Key {
            key: Key::Character('u'),
        },
    )
    .unwrap();
    assert_eq!(
        s.assignment.editor().contents(),
        d.initial.files["/lab/service.conf"].text
    );
    keys(&mut s.assignment, &d.initial, "/port").unwrap();
    let raw = serde_json::to_string(&s).unwrap();
    restored = serde_json::from_str(&raw).unwrap();
    d.context(&restored).unwrap();
    assert_eq!(restored.assignment.editor().mode(), Mode::SearchForward);
    restored
        .apply(&d, false, &Event::Key { key: Key::Enter })
        .unwrap();
    assert_eq!(restored.assignment.editor().cursor().unwrap(), (1, 0));
}
#[test]
fn invalid_source_forged_progress_and_bounded_imports_are_rejected() {
    let d = definition();
    let s = d.new_session().unwrap();
    let mut bad = s.clone();
    bad.id = "other".into();
    assert!(d.context(&bad).is_err());
    assert!(!bad.solved(&d));
    let mut bad = s.clone();
    bad.assignment.selected = "/personal/file".into();
    assert!(d.context(&bad).is_err());
    let mut bad = s.clone();
    bad.assignment.editors.remove("/lab/notes.txt");
    assert!(d.context(&bad).is_err());
    let mut f = fixture();
    f.files.insert(
        "/lab/../escape".into(),
        File {
            text: "x".into(),
            writable: true,
            draft: None,
        },
    );
    assert!(f.validate().is_err());
    let mut d = definition();
    d.example_solution.clear();
    assert!(d.validate().is_err());
    let duplicate =
        r#"{"editors":{"/lab/x":null,"/lab/x":null},"selected":"/lab/x","checked":true}"#;
    assert!(serde_json::from_str::<Attempt>(duplicate).is_err());
    let oversized = serde_json::json!({"files":{("x".repeat(121)):{"text":"","writable":true}},"start":"x","requirements":[]});
    assert!(serde_json::from_value::<Fixture>(oversized).is_err());
}

#[test]
fn supplied_recovery_buffer_can_be_discarded_or_undone_without_inventing_saved_edits() {
    for action in [":q!\r", "u"] {
        let mut f = fixture();
        f.start = "/lab/notes.txt".into();
        f.files.get_mut(&f.start).unwrap().draft = Some("Accidental unsaved note.\n".into());
        let mut a = Attempt::new(&f).unwrap();
        assert!(a.editor().modified());
        assert_eq!(a.editor().file_bytes(), "Keep this original note.\n");
        assert!(
            a.apply(
                &f,
                &Event::Open {
                    path: "/lab/service.conf".into()
                }
            )
            .is_err()
        );
        let raw = serde_json::to_string(&a).unwrap();
        let mut restored: Attempt = serde_json::from_str(&raw).unwrap();
        restored.valid_for(&f).unwrap();
        assert_eq!(restored, a);
        keys(&mut restored, &f, action).unwrap();
        assert!(!restored.editor().modified());
        restored
            .apply(
                &f,
                &Event::Open {
                    path: "/lab/service.conf".into(),
                },
            )
            .unwrap();
        replace(&mut restored, &f, "name=edge-a\nport=8080\nworkers=4");
        keys(&mut restored, &f, ":w\r").unwrap();
        restored.apply(&f, &Event::Check {}).unwrap();
        assert!(restored.solved(&f));
        a.apply(&f, &Event::Check {}).unwrap();
        assert!(!a.solved(&f));
    }
}
#[test]
fn feedback_identifies_the_actual_wrong_setting_count_and_unsaved_file() {
    let f = fixture();
    let mut a = Attempt::new(&f).unwrap();
    let rows = a.checks(&f);
    let feedback = rows
        .iter()
        .map(|r| r.1.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(feedback.contains("port: expected \"8080\", saved \"80\""));
    assert!(feedback.contains("workers: expected \"4\", saved \"2\""));
    keys(&mut a, &f, "iX").unwrap();
    assert!(a.checks(&f)[0].1.contains("/lab/service.conf"));
    let mut f = fixture();
    f.files.get_mut("/lab/service.conf").unwrap().text = "KEEP\nRUN inspect\n".into();
    f.requirements[0] = Requirement::Records {
        path: "/lab/service.conf".into(),
        counts: BTreeMap::from([("RUN inspect".into(), 2)]),
    };
    let a = Attempt::new(&f).unwrap();
    assert!(
        a.checks(&f)[1]
            .1
            .contains("\"RUN inspect\": expected 2, saved 1")
    );
}
#[test]
fn no_op_assignments_and_canonically_equal_recovery_drafts_are_invalid_packs() {
    let mut f = fixture();
    f.files.get_mut("/lab/service.conf").unwrap().text = "same".into();
    f.files.get_mut("/lab/service.conf").unwrap().draft = Some("same\n".into());
    assert!(f.validate().unwrap_err().contains("actually differ"));
    let mut d = definition();
    d.initial.files.get_mut("/lab/service.conf").unwrap().text =
        "name=edge-a\nport=8080\nworkers=4\n".into();
    assert!(d.validate().unwrap_err().contains("concrete incomplete"));
}
#[test]
fn worked_answer_remains_unchecked_even_if_an_authored_example_contains_check() {
    let mut d = definition();
    d.example_solution.push(Event::Check {});
    d.validate().unwrap();
    let mut s = d.new_session().unwrap();
    let assignment = s.assignment.clone();
    s.show_worked(&d).unwrap();
    assert!(!s.practice.checked);
    assert_eq!(s.assignment, assignment);
    assert!(!s.solved(&d));
}
