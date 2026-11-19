use dlms_learning_lab::{
    dkms as d,
    dkms_workspace::{Action, Definition, Required, Requirement, Session, Task, Work},
};
fn task(id: &str, release: &str) -> Task {
    let facts = d::Facts {
        kernels: ["6.6.49", release]
            .map(|r| d::Kernel { release: r.into() })
            .to_vec(),
        running: 0,
        architecture: "x86_64".into(),
        sources: vec![d::Source {
            name: "survey".into(),
            version: "1.0".into(),
            object: "survey".into(),
        }],
    };
    let mut initial = d::State::default();
    d::execute(&facts, &mut initial, "dkms install survey/1.0").unwrap();
    Task {
        id: id.into(),
        instructions:
            "Install the supplied module for the target kernel while preserving the running kernel."
                .into(),
        facts,
        initial,
        requirements: vec![Requirement {
            source: 0,
            kernel: 1,
            state: Required::Installed,
        }],
        preserve_kernels: vec![0],
        reference: vec![
            Action::Run {
                input: format!("dkms install survey/1.0 -k {release}"),
            },
            Action::Check,
        ],
    }
}
fn definition() -> Definition {
    Definition {
        version: 1,
        source_id: "dkms-unit-task".into(),
        assignment: task("dkms-unit-task", "6.6.50"),
        example: task("dkms-unit-example", "6.6.51"),
    }
}

#[test]
fn worked_practice_is_separate_and_never_awards_assignment_success() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    let original = s.assignment.clone();
    for a in &d.example.reference {
        s.apply(&d, true, a).unwrap();
    }
    assert!(d.example.solved(&s.example.work));
    assert_eq!(s.assignment, original);
    assert!(!d.assignment.solved(&s.assignment.work));
}
#[test]
fn partial_wrong_default_and_correct_target_have_specific_feedback() {
    let t = definition().assignment;
    let mut w = Work::new(&t);
    w.log.push(dlms_learning_lab::dkms_workspace::Line {
        input: "dkms install survey/1.0 -k 6.6.50".into(),
        output: "survey/1.0, 6.6.50, x86_64: installed".into(),
        ok: true,
    });
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(!t.solved(&w));
    t.apply(
        &mut w,
        &Action::Run {
            input: "dkms build survey/1.0 -k 6.6.50".into(),
        },
    )
    .unwrap();
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(!t.solved(&w));
    assert!(
        t.checks(&w)[0]
            .detail
            .contains("successful build has not placed")
    );
    t.apply(
        &mut w,
        &Action::Run {
            input: "dkms install survey/1.0".into(),
        },
    )
    .unwrap();
    assert!(!t.solved(&w));
    t.apply(
        &mut w,
        &Action::Run {
            input: "dkms install survey/1.0 -k 6.6.50".into(),
        },
    )
    .unwrap();
    assert!(!t.solved(&w));
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(t.solved(&w));
}
#[test]
fn state_changes_clear_success_but_read_only_status_and_unexecuted_draft_do_not() {
    let t = definition().assignment;
    let mut w = Work::new(&t);
    for a in &t.reference {
        t.apply(&mut w, a).unwrap();
    }
    assert!(t.solved(&w));
    t.apply(
        &mut w,
        &Action::CommandDraft {
            text: "dkms uninstall survey/1.0 -k 6.6.50".into(),
        },
    )
    .unwrap();
    assert!(t.solved(&w));
    t.apply(
        &mut w,
        &Action::Run {
            input: "dkms status".into(),
        },
    )
    .unwrap();
    assert!(t.solved(&w));
    let input = w.command_draft.clone();
    t.apply(&mut w, &Action::Run { input }).unwrap();
    assert!(!w.checked);
    assert!(!t.solved(&w));
    w.checked = true;
    assert!(
        !t.solved(&w),
        "A forged checked flag must not replace state grading"
    );
}
#[test]
fn protected_kernel_is_independently_graded_and_repair_is_possible() {
    let t = definition().assignment;
    let mut w = Work::new(&t);
    for a in &t.reference {
        t.apply(&mut w, a).unwrap();
    }
    t.apply(
        &mut w,
        &Action::Run {
            input: "dkms uninstall survey/1.0".into(),
        },
    )
    .unwrap();
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(!t.solved(&w));
    assert!(t.checks(&w)[0].passed);
    assert!(!t.checks(&w)[1].passed);
    assert!(t.checks(&w)[1].detail.contains("protected kernel"));
    t.apply(
        &mut w,
        &Action::Run {
            input: "dkms install survey/1.0".into(),
        },
    )
    .unwrap();
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(t.solved(&w));
}
#[test]
fn reset_only_selected_scope_and_roundtrip_preserves_partial_work_and_drafts() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    s.apply(
        &d,
        false,
        &Action::CommandDraft {
            text: "dkms install survey/1.0 -k 6.6.50".into(),
        },
    )
    .unwrap();
    s.apply(
        &d,
        false,
        &Action::Run {
            input: "dkms build survey/1.0 -k 6.6.50".into(),
        },
    )
    .unwrap();
    s.apply(&d, true, &d.example.reference[0]).unwrap();
    let example = s.example.clone();
    let restored: Session = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(s, restored);
    restored.bind(&d).unwrap();
    restored.validate_shape().unwrap();
    s.apply(&d, false, &Action::Reset).unwrap();
    assert_eq!(s.assignment.work, Work::new(&d.assignment));
    assert_eq!(s.example, example);
}
#[test]
fn source_bound_progress_refuses_changed_requirements_or_crossed_scopes() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    let mut changed = d.clone();
    changed.assignment.instructions.push_str(" Changed task.");
    assert!(s.bind(&changed).is_err());
    std::mem::swap(&mut s.assignment, &mut s.example);
    assert!(s.bind(&d).is_err());
    assert!(s.validate_shape().is_err());
}
#[test]
fn invalid_and_unreachable_tasks_and_oversize_imports_fail_closed() {
    let t = definition().assignment;
    let mut bad = t.clone();
    bad.requirements[0].source = 99;
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.requirements.push(bad.requirements[0].clone());
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.reference = vec![Action::Check];
    assert!(bad.validate().is_err());
    let mut bad = t;
    bad.preserve_kernels = vec![1];
    assert!(bad.validate().is_err());
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    s.assignment.work.command_draft = "x".repeat(2049);
    assert!(s.bind(&d).is_err());
    let mut s = Session::new(&d).unwrap();
    s.assignment.source_task = "x".repeat(32769);
    assert!(s.validate_shape().is_err());
}
#[test]
fn console_history_is_bounded_and_failed_commands_are_actual_diagnostics() {
    let t = definition().assignment;
    let mut w = Work::new(&t);
    for _ in 0..110 {
        t.apply(
            &mut w,
            &Action::Run {
                input: "dkms status".into(),
            },
        )
        .unwrap();
    }
    assert_eq!(w.log.len(), 96);
    let before = w.state.clone();
    assert!(
        t.apply(
            &mut w,
            &Action::Run {
                input: "dkms install $(id)".into()
            }
        )
        .is_err()
    );
    assert_eq!(w.state, before);
    assert_eq!(w.log.len(), 96);
    assert!(!w.log.last().unwrap().ok);
    assert!(w.log.last().unwrap().output.contains("outside"));
}

#[test]
fn largest_supported_status_fits_the_console_log_without_silent_truncation() {
    let mut t = definition().assignment;
    t.facts.kernels = (0..4)
        .map(|i| d::Kernel {
            release: format!("{}{}", "k".repeat(63), i),
        })
        .collect();
    t.facts.sources = (0..8)
        .map(|i| d::Source {
            name: format!("{}{}", "m".repeat(31), i),
            version: "v".repeat(24),
            object: format!("{}{}", "o".repeat(31), i),
        })
        .collect();
    t.initial = d::State {
        registered: (0..8).collect(),
        records: (0..8)
            .flat_map(|source| {
                (0..4).map(move |kernel| d::Record {
                    source,
                    kernel,
                    installed: true,
                })
            })
            .collect(),
    };
    let mut w = Work::new(&t);
    let output = t
        .apply(
            &mut w,
            &Action::Run {
                input: "dkms status".into(),
            },
        )
        .unwrap();
    assert_eq!(output.lines().count(), 32);
    assert!(output.len() > 4096 && output.len() < 8192);
    assert_eq!(w.log[0].output, output);
    t.validate_work(&w).unwrap();
    let restored: Work = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    assert_eq!(restored, w);
}
