//! Prospective typed/native-workspace backend boundaries. No registration or host disk access.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    partition_fs as core, partition_fs_workspace as work, progress,
    script_teaching::View,
};
#[path = "support/partition_fs_fixture.rs"]
mod fixture;
use fixture::*;
fn value<T: serde::Serialize + ?Sized>(x: &T) -> serde_json::Value {
    serde_json::to_value(x).unwrap()
}
fn solved(a: &progress::Attempt, s: &dlms_learning_lab::pack::Scenario) -> bool {
    progress::current_solved(a, s)
}
#[test]
fn alternate_media_extents_order_and_generated_or_explicit_uuid_are_state_graded() {
    for (disk, size, reverse, explicit) in [
        ("/dev/vda", 8, false, false),
        ("/dev/vdc", 12, false, true),
        ("/dev/vda", 16, true, false),
    ] {
        let (s, mut a) = fresh();
        assert!(!solved(&a, &s));
        let mut ops = operations(disk, "reports", "paging", size, 2, "reports");
        if explicit {
            for (n, o) in ops.iter_mut().enumerate() {
                if let work::Operation::Format { uuid, .. } = o {
                    *uuid = Some(format!("00000000-0000-4000-8000-{:012x}", 100 + n));
                }
            }
        }
        if reverse {
            // Same final relationships, no required transcript order.
            let last = ops.pop().unwrap();
            let add_swap = ops.pop().unwrap();
            let format_data = ops.pop().unwrap();
            ops.push(add_swap);
            ops.push(last);
            ops.push(format_data);
        }
        for o in &ops {
            work::binding::apply(&mut a, &s, o, false).unwrap();
        }
        assert!(solved(&a, &s));
        assert!(engine::solved(&a.state, &s.goal));
        assert_eq!(
            session(&a).current.disks["/dev/vdb"],
            definition().initial.disks["/dev/vdb"]
        );
        assert!(session(&a).example == definition().lesson.fixture);
    }
}
#[test]
fn partition_hint_is_not_formatting_and_extra_allocation_invalidates_current_success() {
    let (s, mut a) = fresh();
    let ops = operations("/dev/vda", "reports", "paging", 8, 2, "reports");
    for n in [0, 1, 3] {
        work::binding::apply(&mut a, &s, &ops[n], false).unwrap();
    }
    assert!(!solved(&a, &s));
    let Goal::PartitionFs { definition: d } = &s.goal else {
        panic!()
    };
    assert!(d.checks(session(&a))[0].1.contains("unformatted"));
    for n in [2, 4] {
        work::binding::apply(&mut a, &s, &ops[n], false).unwrap();
    }
    assert!(solved(&a, &s));
    work::binding::check(&mut a, &s, false).unwrap();
    assert!(session(&a).checked);
    work::binding::apply(
        &mut a,
        &s,
        &work::Operation::AddPartition {
            disk: "/dev/vda".into(),
            name: "unrequested".into(),
            start: 40960,
            end: 43007,
            type_hint: core::TypeHint::LinuxData,
        },
        false,
    )
    .unwrap();
    assert!(!solved(&a, &s));
    assert!(!session(&a).checked);
    assert!(!d.checks(session(&a)).last().unwrap().0);
}
#[test]
fn unfinished_inactive_drafts_do_not_apply_or_replace_correct_current_configuration() {
    let (s, mut a) = fresh();
    for o in operations("/dev/vda", "reports", "paging", 8, 2, "reports") {
        work::binding::apply(&mut a, &s, &o, false).unwrap();
    }
    let before = session(&a).current.clone();
    let mut draft = session(&a).assignment_editor.drafts["/dev/vdc"].clone();
    draft.start = "unfinished".into();
    draft.end = "18446744073709551616".into();
    draft.label = "unapplied".into();
    work::binding::set_draft(&mut a, &s, "/dev/vdc", draft.clone(), false).unwrap();
    assert!(solved(&a, &s));
    assert_eq!(session(&a).current, before);
    assert_eq!(session(&a).assignment_editor.drafts["/dev/vdc"], draft);
    assert!(draft.add_operation("/dev/vdc").is_err());
    assert_eq!(session(&a).current, before);
    work::binding::select_disk(&mut a, &s, "/dev/vda", false).unwrap();
    assert_eq!(session(&a).assignment_editor.drafts["/dev/vdc"], draft);
}
#[test]
fn separate_practice_insert_is_not_execution_and_reset_then_manual_replay_is_honest_repair() {
    let (s, mut a) = fresh();
    let Goal::PartitionFs { definition: d } = &s.goal else {
        panic!()
    };
    d.validate().unwrap();
    let initial = session(&a).current.clone();
    let editor = session(&a).assignment_editor.clone();
    work::binding::set_view(&mut a, &s, View::Example).unwrap();
    work::binding::insert(&mut a, &s, 1).unwrap();
    assert_eq!(session(&a).example, d.lesson.fixture);
    assert!(session(&a).example_transcript.is_empty());
    assert!(!session(&a).example_checked);
    assert!(!solved(&a, &s));
    for step in &d.lesson.worked {
        work::binding::apply(&mut a, &s, &step.operation, true).unwrap();
    }
    assert!(d.practice_checks(session(&a)).iter().all(|r| r.0));
    work::binding::apply(&mut a, &s, &d.lesson.experiment, true).unwrap();
    assert!(!d.practice_checks(session(&a)).iter().all(|r| r.0));
    work::binding::reset_example(&mut a, &s).unwrap();
    assert_eq!(session(&a).example, d.lesson.fixture);
    assert!(session(&a).example_transcript.is_empty());
    for step in &d.lesson.worked {
        work::binding::apply(&mut a, &s, &step.operation, true).unwrap();
    }
    assert!(d.practice_checks(session(&a)).iter().all(|r| r.0));
    assert_eq!(session(&a).current, initial);
    assert_eq!(session(&a).assignment_editor, editor);
    assert!(!solved(&a, &s));
}
#[test]
fn wrong_format_on_blank_media_and_changed_spare_are_retained_wrong_results_not_normalized() {
    let (s, mut a) = fresh();
    let ops = operations("/dev/vda", "reports", "paging", 8, 2, "reports");
    for n in [0, 1, 3] {
        work::binding::apply(&mut a, &s, &ops[n], false).unwrap();
    }
    work::binding::apply(
        &mut a,
        &s,
        &work::Operation::Format {
            partition: "/dev/vda1".into(),
            format: work::FormatChoice::Swap,
            label: "reports".into(),
            uuid: None,
        },
        false,
    )
    .unwrap();
    work::binding::context(&a, &s).unwrap();
    assert!(!solved(&a, &s));
    assert!(matches!(
        session(&a).current.disks["/dev/vda"]
            .table
            .as_ref()
            .unwrap()
            .partitions[0]
            .content,
        core::Content::Swap { .. }
    ));
    work::binding::apply(
        &mut a,
        &s,
        &work::Operation::CreateGpt {
            disk: "/dev/vdd".into(),
        },
        false,
    )
    .unwrap();
    work::binding::context(&a, &s).unwrap();
    assert!(!solved(&a, &s));
    assert!(session(&a).current.disks["/dev/vdd"].table.is_some());
}
#[test]
fn refusal_of_overwrite_overlap_unknown_or_invalid_operation_is_whole_attempt_atomic() {
    let (s, mut a) = fresh();
    for o in operations("/dev/vda", "reports", "paging", 8, 2, "reports") {
        work::binding::apply(&mut a, &s, &o, false).unwrap();
    }
    for o in [
        work::Operation::CreateGpt {
            disk: "/dev/vda".into(),
        },
        work::Operation::CreateGpt {
            disk: "/dev/vdz".into(),
        },
        work::Operation::Format {
            partition: "/dev/vdb1".into(),
            format: work::FormatChoice::Ext4,
            label: "bad".into(),
            uuid: None,
        },
        work::Operation::AddPartition {
            disk: "/dev/vda".into(),
            name: "overlap".into(),
            start: 2048,
            end: 4095,
            type_hint: core::TypeHint::LinuxData,
        },
    ] {
        let before = value(&a);
        assert!(work::binding::apply(&mut a, &s, &o, false).is_err());
        assert_eq!(value(&a), before);
    }
}
#[test]
fn independent_source_authority_rejects_joint_saved_witness_and_goal_substitution() {
    let (s, mut a) = fresh();
    for operation in operations("/dev/vda", "reports", "paging", 8, 2, "reports") {
        work::binding::apply(&mut a, &s, &operation, false).unwrap();
    }
    assert!(solved(&a, &s));
    let original_source = value(&s);
    let State::PartitionFs { simulation } = &mut a.state else {
        panic!()
    };
    simulation.origin.plan.targets[0].minimum_bytes = 4 * MIB;
    simulation.validate().unwrap(); // The forged context is internally valid.
    let mut forged = s.clone();
    let Goal::PartitionFs { definition: d } = &mut forged.goal else {
        panic!()
    };
    d.plan.targets[0].minimum_bytes = 4 * MIB;
    let State::PartitionFs {
        simulation: supplied,
    } = &mut forged.initial
    else {
        panic!()
    };
    supplied.origin.plan.targets[0].minimum_bytes = 4 * MIB;
    work::binding::context(&a, &forged).unwrap();
    assert!(engine::solved(&a.state, &forged.goal));
    // Internally consistent carried witnesses plus Goal cannot replace the
    // real source that the app retains for Save/restore/open/current credit.
    assert!(work::binding::context(&a, &s).is_err());
    assert!(!solved(&a, &s));
    assert_eq!(value(&s), original_source);
}
#[test]
fn retained_data_geometry_and_version_forgeries_fail_before_any_mutation() {
    let (s, a) = fresh();
    for n in 0..4 {
        let mut bad: progress::Attempt = serde_json::from_value(value(&a)).unwrap();
        let State::PartitionFs { simulation } = &mut bad.state else {
            panic!()
        };
        match n {
            0 => {
                simulation
                    .current
                    .disks
                    .get_mut("/dev/vda")
                    .unwrap()
                    .sectors += 1
            }
            1 => {
                if let core::Content::Ext4 { files, .. } = &mut simulation
                    .current
                    .disks
                    .get_mut("/dev/vdb")
                    .unwrap()
                    .table
                    .as_mut()
                    .unwrap()
                    .partitions[0]
                    .content
                {
                    files.get_mut("/keep.txt").unwrap().text = "lost".into();
                } else {
                    panic!()
                }
            }
            2 => simulation.version += 1,
            _ => {
                simulation
                    .current
                    .apply(core::Action::CreateGpt {
                        disk: "/dev/vda".into(),
                    })
                    .unwrap();
                simulation
                    .current
                    .apply(core::Action::AddPartition {
                        disk: "/dev/vda".into(),
                        name: "reports".into(),
                        start: 2048,
                        end: 18431,
                        type_hint: core::TypeHint::LinuxData,
                    })
                    .unwrap();
                simulation
                    .current
                    .apply(core::Action::FormatExt4 {
                        partition: "/dev/vda1".into(),
                        label: "reports".into(),
                        uuid: None,
                    })
                    .unwrap();
                let core::Content::Ext4 { files, .. } = &mut simulation
                    .current
                    .disks
                    .get_mut("/dev/vda")
                    .unwrap()
                    .table
                    .as_mut()
                    .unwrap()
                    .partitions[0]
                    .content
                else {
                    panic!()
                };
                files.insert(
                    "/unreachable-write".into(),
                    core::File {
                        text: "No write action exists in this preparation-only core".into(),
                    },
                );
                simulation.current.validate().unwrap();
            }
        }
        let before = value(&bad);
        assert!(work::binding::context(&bad, &s).is_err());
        assert!(work::binding::check(&mut bad, &s, false).is_err());
        assert_eq!(value(&bad), before);
        assert!(!solved(&bad, &s));
    }
}
#[test]
fn native_workspace_has_no_terminal_alias_and_readonly_help_cannot_apply_a_disk_change() {
    let (s, mut a) = fresh();
    let before = value(&a);
    assert!(
        engine::execute_for_goal(&mut a.state, &s.goal, "parted -s /dev/vda mklabel gpt").is_err()
    );
    assert_eq!(value(&a), before);
    assert!(
        engine::execute_for_goal(&mut a.state, &s.goal, "help")
            .unwrap()
            .contains("No delete/resize/overwrite")
    );
    assert_eq!(value(&a), before);
    assert!(
        engine::execute_for_goal(&mut a.state, &s.goal, "show")
            .unwrap()
            .contains("no real devices")
    );
    assert_eq!(value(&a), before);
}

#[path = "support/partition_fs_capacity.rs"]
mod capacity;
#[test]
fn content_feedback_distinguishes_blank_wrong_label_and_wrong_format_without_changing_grading() {
    for (format, label, expected) in [
        (
            None,
            "",
            "actual content is unformatted; need ext4 labeled \"reports\"",
        ),
        (
            Some(work::FormatChoice::Ext4),
            "wrong",
            "actual content is ext4 labeled \"wrong\"; need ext4 labeled \"reports\"",
        ),
        (
            Some(work::FormatChoice::Swap),
            "reports",
            "actual content is swap labeled \"reports\"; need ext4 labeled \"reports\"",
        ),
        (
            Some(work::FormatChoice::Ext4),
            "",
            "actual content is ext4 with no label; need ext4 labeled \"reports\"",
        ),
    ] {
        let (s, mut a) = fresh();
        for mut op in operations("/dev/vda", "reports", "paging", 8, 2, "reports") {
            if let work::Operation::Format {
                partition,
                format: actual_format,
                label: actual_label,
                ..
            } = &mut op
                && partition == "/dev/vda1"
            {
                let Some(format) = format else { continue };
                *actual_format = format;
                *actual_label = label.into();
            }
            work::binding::apply(&mut a, &s, &op, false).unwrap();
        }
        let before = value(&session(&a).current);
        work::binding::check(&mut a, &s, false).unwrap();
        let rows = engine::checks(&a.state, &s.goal);
        assert_eq!(
            rows.iter().map(|r| r.0).collect::<Vec<_>>(),
            vec![false, true, true]
        );
        assert!(rows[0].1.contains(expected), "{}", rows[0].1);
        assert!(!solved(&a, &s));
        assert_eq!(value(&session(&a).current), before);
    }
}
#[test]
fn every_saved_mutation_admits_complete_authored_scenario_and_attempt_before_copy() {
    let (s, mut a) = fresh();
    let s = capacity::padded_source(s);
    capacity::fill_to(&s, &mut a, work::MAX_CONTEXT);
    // Individually valid transcript, session and source; only their complete pair is too large.
    let row = a.transcript.iter_mut().find(|r| r.len() < 8192).unwrap();
    row.push('x');
    assert_eq!(capacity::size(&s, &a), work::MAX_CONTEXT + 1);
    let State::PartitionFs { simulation } = &a.state else {
        unreachable!()
    };
    if let Goal::PartitionFs { definition } = &s.goal {
        definition.bound(simulation).unwrap();
    }
    for action in 0..12 {
        let mut candidate = a.clone();
        let before = value(&candidate);
        let draft = fixture::session(&candidate).assignment_editor.drafts["/dev/vda"].clone();
        let result = match action {
            0 => work::binding::prepare(&mut candidate, &s),
            1 => work::binding::set_view(&mut candidate, &s, View::Assignment),
            2 => work::binding::select_disk(&mut candidate, &s, "/dev/vda", false),
            3 => work::binding::set_draft(&mut candidate, &s, "/dev/vda", draft, false),
            4 => work::binding::apply(
                &mut candidate,
                &s,
                &work::Operation::CreateGpt {
                    disk: "/dev/vda".into(),
                },
                false,
            ),
            5 => work::binding::check(&mut candidate, &s, false),
            6 => work::binding::select_example(&mut candidate, &s, 0),
            7 => work::binding::insert(&mut candidate, &s, 0),
            8 => work::binding::reset_example(&mut candidate, &s),
            9 => work::binding::request_hint(&mut candidate, &s),
            10 => work::binding::admit_launch(Some(&candidate), &s, 2),
            _ => work::binding::set_challenge(&mut candidate, &s, true),
        };
        assert!(
            result.unwrap_err().contains("Complete disk source/Attempt"),
            "action{action}"
        );
        assert_eq!(value(&candidate), before, "action{action}");
    }
}
#[test]
fn projected_growing_mutations_refuse_at_pair_limit_and_retry_with_room_without_cap_raise() {
    for action in 0..5 {
        let (s, mut a) = fresh();
        let s = capacity::padded_source(s);
        work::binding::set_view(&mut a, &s, View::Example).unwrap();
        work::binding::set_challenge(&mut a, &s, true).unwrap();
        a.hints = 9;
        capacity::fill_to(&s, &mut a, work::MAX_CONTEXT);
        let before = value(&a);
        let update = |a: &mut progress::Attempt| match action {
            0 => work::binding::set_view(a, &s, View::Introduction),
            1 => work::binding::set_challenge(a, &s, false),
            2 => work::binding::request_hint(a, &s),
            3 => {
                let mut draft = fixture::session(a).assignment_editor.drafts["/dev/vda"].clone();
                draft.name = "a".into();
                work::binding::set_draft(a, &s, "/dev/vda", draft, false)
            }
            _ => work::binding::apply(
                a,
                &s,
                &work::Operation::CreateGpt {
                    disk: "/dev/vda".into(),
                },
                false,
            ),
        };
        assert!(update(&mut a).is_err(), "growing action{action}");
        assert_eq!(value(&a), before);
        assert_eq!(capacity::size(&s, &a), work::MAX_CONTEXT);
        // Same action succeeds with legitimate saved room; no source/model limits change.
        a.transcript.remove(0);
        update(&mut a).unwrap();
        work::binding::context(&a, &s).unwrap();
    }
}
