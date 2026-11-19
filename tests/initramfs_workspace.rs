#[path = "support/initramfs_fixture.rs"]
mod fixture;
use dlms_learning_lab::{
    initramfs_workspace::{Action, Definition, Kind, Session, Work},
    script_teaching::View,
};
fn definition(kind: Kind) -> Definition {
    let assignment = fixture::task(kind, false);
    Definition {
        version: 1,
        source_id: assignment.id.clone(),
        assignment,
        example: fixture::task(kind, true),
    }
}
#[test]
fn source_tasks_and_separate_examples_reach_actual_outcomes() {
    for kind in [Kind::Drivers, Kind::Handover] {
        let d = definition(kind);
        let mut s = Session::new(&d).unwrap();
        let original = s.assignment.clone();
        for a in &d.example.reference {
            s.apply(&d, true, a).unwrap();
        }
        assert!(d.example.solved(&s.example.work));
        assert_eq!(s.assignment, original);
        assert!(!d.assignment.solved(&s.assignment.work));
        for a in &d.assignment.reference {
            s.apply(&d, false, a).unwrap();
        }
        assert!(
            d.assignment.solved(&s.assignment.work),
            "{:?}",
            d.assignment.checks(&s.assignment.work)
        );
        s.bind(&d).unwrap();
        s.validate_shape().unwrap();
    }
}
#[test]
fn driver_draft_save_generation_and_boot_are_distinct_states() {
    let t = fixture::task(Kind::Drivers, false);
    let mut w = Work::new(&t);
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(!t.solved(&w));
    assert!(
        !t.checks(&w)
            .iter()
            .find(|c| c.label == "Candidate root handoff")
            .unwrap()
            .passed
    );
    t.apply(
        &mut w,
        &Action::ConfigDraft {
            text: "omit_drivers=''\n".into(),
        },
    )
    .unwrap();
    assert!(!w.checked);
    t.apply(
        &mut w,
        &Action::Run {
            input: "dracut -f --kver 6.6.61 /boot/initramfs-6.6.61.img".into(),
        },
    )
    .unwrap();
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(!t.solved(&w), "Unsaved choices must not affect generation");
    t.apply(&mut w, &Action::SaveConfig).unwrap();
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(!t.solved(&w), "Saving alone does not regenerate an image");
    assert!(
        !t.checks(&w)
            .iter()
            .find(|c| c.label == "Current target early userspace")
            .unwrap()
            .passed
    );
    t.apply(
        &mut w,
        &Action::Run {
            input: "dracut /boot/initramfs-6.6.61.img 6.6.61 --force".into(),
        },
    )
    .unwrap();
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(t.solved(&w));
}
#[test]
fn alternate_literal_repair_in_place_and_new_file_handover_are_supported() {
    for text in [
        "# Remove the omission rather than append more text.\n",
        "omit_drivers=''\n",
        "omit_drivers=nvme\nomit_drivers=\"\"\n",
    ] {
        let t = fixture::task(Kind::Drivers, false);
        let mut w = Work::new(&t);
        for a in [
            Action::ConfigDraft { text: text.into() },
            Action::SaveConfig,
            Action::Run {
                input: "dracut -f /boot/initramfs-6.6.61.img 6.6.61".into(),
            },
            Action::Check,
        ] {
            t.apply(&mut w, &a).unwrap();
        }
        assert!(t.solved(&w));
    }
    let t = fixture::task(Kind::Handover, false);
    let mut w = Work::new(&t);
    for a in [
        Action::Run {
            input: "dracut -f --kver 6.6.61 /boot/initramfs-6.6.61.img".into(),
        },
        Action::Check,
    ] {
        t.apply(&mut w, &a).unwrap();
    }
    assert!(
        t.solved(&w),
        "Repairing the current image is a legitimate alternative to a new filename"
    );
    let mut w = Work::new(&t);
    for a in [
        Action::Run {
            input: "dracut --kver 6.6.61 /boot/another.img".into(),
        },
        Action::EntryDraft {
            filename: "another.conf".into(),
            text: fixture::entry("6.6.61", "/another.img", &t.root.uuid),
        },
        Action::SaveEntry,
        Action::Check,
    ] {
        t.apply(&mut w, &a).unwrap();
    }
    assert!(t.solved(&w));
}
#[test]
fn generation_for_wrong_kernel_and_unchanged_wrong_file_do_not_pass() {
    let t = fixture::task(Kind::Handover, false);
    let mut w = Work::new(&t);
    t.apply(&mut w, &Action::Check).unwrap();
    let failed = t
        .checks(&w)
        .into_iter()
        .filter(|c| !c.passed)
        .collect::<Vec<_>>();
    assert!(failed.iter().any(|c| c.detail.contains("Filenames")));
    assert!(!t.solved(&w));
    t.apply(
        &mut w,
        &Action::Run {
            input: "dracut --kver 6.6.61 /boot/unselected.img".into(),
        },
    )
    .unwrap();
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(
        !t.solved(&w),
        "A valid unused output cannot hide the selected wrong image"
    );
    t.apply(
        &mut w,
        &Action::EntryDraft {
            filename: "candidate.conf".into(),
            text: fixture::entry("6.6.60", "/initramfs-6.6.60.img", &t.root.uuid),
        },
    )
    .unwrap();
    t.apply(&mut w, &Action::SaveEntry).unwrap();
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(
        t.checks(&w)
            .iter()
            .any(|c| !c.passed && c.label == "Target kernel selection")
    );
}
#[test]
fn changing_saved_choices_or_selected_output_invalidates_current_success() {
    let t = fixture::task(Kind::Handover, false);
    let mut w = Work::new(&t);
    for a in &t.reference {
        t.apply(&mut w, a).unwrap();
    }
    assert!(t.solved(&w));
    t.apply(
        &mut w,
        &Action::CommandDraft {
            text: "dracut -f --kver 6.6.60 /boot/repaired.img".into(),
        },
    )
    .unwrap();
    assert!(t.solved(&w));
    let input = w.command_draft.clone();
    t.apply(&mut w, &Action::Run { input }).unwrap();
    assert!(!w.checked);
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(!t.solved(&w));
    t.apply(
        &mut w,
        &Action::Run {
            input: "dracut --force --kver 6.6.61 /boot/repaired.img".into(),
        },
    )
    .unwrap();
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(t.solved(&w));
    t.apply(
        &mut w,
        &Action::ConfigDraft {
            text: "omit_drivers=nvme\n".into(),
        },
    )
    .unwrap();
    assert!(!w.checked);
    t.apply(&mut w, &Action::SaveConfig).unwrap();
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(!t.solved(&w));
}
#[test]
fn protected_fallback_damage_is_graded_and_can_be_restored_without_reset() {
    let t = fixture::task(Kind::Handover, false);
    let mut w = Work::new(&t);
    for a in &t.reference {
        t.apply(&mut w, a).unwrap();
    }
    t.apply(
        &mut w,
        &Action::Run {
            input: "dracut -f --kver 6.6.61 /boot/initramfs-6.6.60.img".into(),
        },
    )
    .unwrap();
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(!t.solved(&w));
    assert!(
        t.checks(&w)
            .iter()
            .any(|c| !c.passed && c.detail.contains("Protected files changed"))
    );
    t.apply(
        &mut w,
        &Action::Run {
            input: "dracut -f --kver 6.6.60 /boot/initramfs-6.6.60.img".into(),
        },
    )
    .unwrap();
    t.apply(&mut w, &Action::Check).unwrap();
    assert!(t.solved(&w));
}
#[test]
fn partial_resume_and_selected_scope_reset_preserve_other_work() {
    let d = definition(Kind::Handover);
    let mut s = Session::new(&d).unwrap();
    for practice in [true, false] {
        let t = if practice { &d.example } else { &d.assignment };
        s.apply(&d, practice, &t.reference[0]).unwrap();
        s.apply(
            &d,
            practice,
            &Action::CommandDraft {
                text: if practice {
                    "dracut --kver 6.1.121 /boot/exam"
                } else {
                    "dracut --kver 6.6.61 /boot/assig"
                }
                .into(),
            },
        )
        .unwrap();
        s.apply(&d, practice, &Action::Check).unwrap();
    }
    s.view = View::Assignment;
    let copy: Session = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(s, copy);
    copy.bind(&d).unwrap();
    copy.validate_shape().unwrap();
    let example = s.example.clone();
    s.apply(&d, false, &Action::Reset).unwrap();
    assert_eq!(s.example, example);
    assert_eq!(s.assignment.work, Work::new(&d.assignment));
}
#[test]
fn invalid_sources_forged_facts_and_unsafe_entry_names_are_refused() {
    let d = definition(Kind::Handover);
    let mut s = Session::new(&d).unwrap();
    let mut changed = d.clone();
    changed.assignment.wanted_release = "6.6.60".into();
    assert!(s.bind(&changed).is_err());
    s.assignment.work.store.images[0].build.source_id = "forged".into();
    assert!(s.bind(&d).is_err());
    assert!(!d.assignment.solved(&s.assignment.work));
    let mut t = d.assignment.clone();
    t.reference = vec![Action::Check];
    assert!(t.validate().is_err());
    let mut t = d.assignment.clone();
    t.preserve_paths = vec!["/boot/nonexistent.img".into()];
    assert!(t.validate().is_err());
    let t = d.assignment;
    let mut w = Work::new(&t);
    let before = w.store.clone();
    t.apply(
        &mut w,
        &Action::EntryDraft {
            filename: "../fallback.conf".into(),
            text: "title bad".into(),
        },
    )
    .unwrap();
    assert!(t.apply(&mut w, &Action::SaveEntry).is_err());
    assert_eq!(w.store, before);
    let mut w = Work::new(&t);
    t.apply(
        &mut w,
        &Action::EntryDraft {
            filename: "fallback.conf".into(),
            text: "title overwrite".into(),
        },
    )
    .unwrap();
    assert!(t.apply(&mut w, &Action::SaveEntry).is_err());
    assert_eq!(w.store, before);
}
