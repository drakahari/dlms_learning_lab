//! Bounded authored-workspace tests, with exclusively synthetic virtual media.
use dlms_learning_lab::{
    raid1 as core, raid1_workspace as work,
    script_teaching::{Concept, View},
};
#[path = "support/raid1.rs"]
mod support;
fn definition() -> work::Definition {
    let initial = support::world();
    let raw = initial
        .to_json()
        .unwrap()
        .replace("/dev/vd", "/dev/vx")
        .replace("/dev/md0", "/dev/md2")
        .replace("/dev/md1", "/dev/md3")
        .replace("records", "sample");
    let example = core::State::from_json(&raw).unwrap();
    let command = |title: &str, c: &str| work::Step {
        title: title.into(),
        explanation: "Inspect the named member and perform the supported simulated operation."
            .into(),
        command: Some(c.into()),
        action: None,
    };
    work::Definition{initial,plan:work::Plan{array:"/dev/md0".into()},lesson:work::Lesson{prerequisite_note:"Know the difference between a device and its filesystem; every supplied path is virtual.".into(),concepts:vec![Concept{title:"A second complete copy".into(),text:"Mirroring retains a synchronized replica; a newly added blank replacement still needs data copied.".into()}],description:"A different sample mirror has one failed member; retire it, add blank media and copy the supplied data.".into(),initial:example,plan:work::Plan{array:"/dev/md2".into()},worked:vec![command("Inspect first","mdadm --detail /dev/md2"),command("Retire failed member","mdadm /dev/md2 --remove /dev/vxb1"),command("Add replacement","mdadm /dev/md2 --add /dev/vxc1"),work::Step{title:"Finish copying".into(),explanation:"Advance only the simulator's bounded sample replication, then inspect the actual copies.".into(),command:None,action:Some(core::Action::Advance{array:"/dev/md2".into(),units:128})}]}}
}
fn complete(d: &work::Definition, s: &mut work::Session, device: &str, example: bool) {
    let array = if example { "/dev/md2" } else { "/dev/md0" };
    let failed = if example { "/dev/vxb1" } else { "/dev/vdb1" };
    for c in [
        format!("mdadm {array} --remove {failed}"),
        format!("mdadm {array} --add {device}"),
    ] {
        assert_eq!(s.execute(d, &c, example).unwrap().status, 0);
    }
    s.action(
        d,
        &core::Action::Advance {
            array: array.into(),
            units: 128,
        },
        example,
    )
    .unwrap();
}
#[test]
fn taught_example_and_alternative_independent_media_work_without_matching_strings() {
    let d = definition();
    d.validate().unwrap();
    for device in ["/dev/vdc1", "/dev/vdd1"] {
        let mut s = work::Session::new(&d).unwrap();
        assert!(!d.checks(&s).iter().all(|r| r.0));
        complete(&d, &mut s, device, false);
        assert!(d.checks(&s).iter().all(|r| r.0));
        assert!(
            !s.checked,
            "state, not a checked boolean, is the outcome authority"
        );
    }
}
#[test]
fn a_wrong_candidate_and_partial_copy_have_specific_feedback_and_remain_retryable() {
    let d = definition();
    let mut s = work::Session::new(&d).unwrap();
    let old = s.current.clone();
    let out = s
        .execute(&d, "mdadm /dev/md0 --add /dev/vde1", false)
        .unwrap();
    assert_eq!(out.status, 1);
    assert!(out.output.contains("usable MD capacity"));
    assert_eq!(s.current, old);
    s.execute(&d, "mdadm /dev/md0 --remove /dev/vdb1", false)
        .unwrap();
    s.execute(&d, "mdadm /dev/md0 --add /dev/vdc1", false)
        .unwrap();
    s.action(
        &d,
        &core::Action::Advance {
            array: "/dev/md0".into(),
            units: 1,
        },
        false,
    )
    .unwrap();
    let rows = d.checks(&s);
    assert_eq!(rows.iter().filter(|r| r.0).count(), 3);
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("Rebuilding")));
    s.action(
        &d,
        &core::Action::Advance {
            array: "/dev/md0".into(),
            units: 128,
        },
        false,
    )
    .unwrap();
    assert!(d.checks(&s).iter().all(|r| r.0));
}
#[test]
fn a_finished_copy_does_not_hide_an_original_failed_member_still_attached() {
    let d = definition();
    let mut s = work::Session::new(&d).unwrap();
    s.execute(&d, "mdadm /dev/md0 --add /dev/vdc1", false)
        .unwrap();
    s.action(
        &d,
        &core::Action::Advance {
            array: "/dev/md0".into(),
            units: 128,
        },
        false,
    )
    .unwrap();
    assert_eq!(s.current.health("/dev/md0").unwrap(), core::Health::Healthy);
    assert!(!d.checks(&s)[0].0);
    s.execute(&d, "mdadm /dev/md0 --remove /dev/vdb1", false)
        .unwrap();
    assert!(d.checks(&s).iter().all(|r| r.0));
}
#[test]
fn guided_runs_reset_and_navigation_preserve_the_assignment_and_its_draft() {
    let d = definition();
    let mut s = work::Session::new(&d).unwrap();
    s.set_command(&d, "mdadm --detail /dev/md0".into(), false)
        .unwrap();
    let assignment = s.current.clone();
    complete(&d, &mut s, "/dev/vxc1", true);
    assert!(s.example_checks(&d).iter().all(|r| r.0));
    assert!(!d.checks(&s).iter().all(|r| r.0));
    s.set_view(&d, View::Assignment).unwrap();
    assert_eq!(s.current, assignment);
    assert_eq!(s.command, "mdadm --detail /dev/md0");
    s.reset_example(&d).unwrap();
    assert_eq!(s.current, assignment);
    assert_eq!(s.example, s.example_initial);
}
#[test]
fn recovery_cursor_drafts_and_realized_outcomes_survive_serialization_without_auto_execution() {
    let d = definition();
    let mut s = work::Session::new(&d).unwrap();
    s.execute(&d, "mdadm /dev/md0 --remove /dev/vdb1", false)
        .unwrap();
    s.execute(&d, "mdadm /dev/md0 --add /dev/vdd1", false)
        .unwrap();
    s.action(
        &d,
        &core::Action::Advance {
            array: "/dev/md0".into(),
            units: 2,
        },
        false,
    )
    .unwrap();
    s.set_command(&d, "mdadm --detail /dev/md0".into(), false)
        .unwrap();
    let raw = serde_json::to_string(&s).unwrap();
    let mut loaded: work::Session = serde_json::from_str(&raw).unwrap();
    d.bound(&loaded).unwrap();
    assert_eq!(loaded, s);
    assert_eq!(
        loaded.current.health("/dev/md0").unwrap(),
        core::Health::Rebuilding {
            copied: 2,
            total: 8
        }
    );
    assert!(!d.checks(&loaded).iter().all(|r| r.0));
    loaded
        .action(
            &d,
            &core::Action::Advance {
                array: "/dev/md0".into(),
                units: 6,
            },
            false,
        )
        .unwrap();
    assert!(d.checks(&loaded).iter().all(|r| r.0));
}
#[test]
fn later_failure_invalidates_actual_success_and_edits_clear_checked_feedback() {
    let d = definition();
    let mut s = work::Session::new(&d).unwrap();
    complete(&d, &mut s, "/dev/vdc1", false);
    s.checked = true;
    assert!(d.checks(&s).iter().all(|r| r.0));
    s.set_command(&d, "mdadm --detail /dev/md0".into(), false)
        .unwrap();
    assert!(!s.checked);
    s.execute(&d, "mdadm /dev/md0 --fail /dev/vdc1", false)
        .unwrap();
    assert!(!d.checks(&s).iter().all(|r| r.0));
}
#[test]
fn forged_initial_markers_foreign_capacity_and_fake_cached_credit_do_not_authorize_success() {
    let d = definition();
    let mut s = work::Session::new(&d).unwrap();
    s.checked = true;
    s.example_checked = true;
    assert!(!d.checks(&s).iter().all(|r| r.0));
    s.initial.devices.get_mut("/dev/vdc1").unwrap().raw_sectors += 1;
    assert!(d.bound(&s).is_err());
    let mut s = work::Session::new(&d).unwrap();
    s.current
        .devices
        .get_mut("/dev/vdc1")
        .unwrap()
        .usable_sectors += 1;
    assert!(d.bound(&s).is_err());
}
#[test]
fn impossible_assignment_is_refused_even_when_its_different_worked_example_succeeds() {
    let mut d = definition();
    for device in d.initial.devices.values_mut() {
        if device.media == core::Media::Blank {
            device.protected = true;
        }
    }
    d.shape().unwrap();
    assert!(
        d.validate()
            .unwrap_err()
            .contains("No eligible replacement")
    );
}
#[test]
fn view_mutation_revalidates_the_exact_serialized_session_capacity_before_publication() {
    let d = definition();
    let mut s = work::Session::new(&d).unwrap();
    s.view = View::Example;
    let row = work::Row {
        input: "mdadm -D /dev/md0".into(),
        output: String::new(),
        status: 0,
    };
    s.transcript = vec![row.clone(); 32];
    s.example_transcript = vec![row; 32];
    for index in 0..64 {
        let used = dlms_learning_lab::cron_workspace::size(&s, usize::MAX).unwrap();
        let n = (work::MAX_SESSION - used).min(8192);
        if index < 32 {
            s.transcript[index].output = "x".repeat(n)
        } else {
            s.example_transcript[index - 32].output = "x".repeat(n)
        }
    }
    assert_eq!(
        dlms_learning_lab::cron_workspace::size(&s, usize::MAX),
        Some(work::MAX_SESSION)
    );
    d.bound(&s).unwrap();
    let old = s.clone();
    assert!(s.set_view(&d, View::Introduction).is_err());
    assert_eq!(s, old);
}
#[test]
fn native_action_codec_is_advance_only_and_bad_drafts_preserve_work() {
    let d = definition();
    let mut s = work::Session::new(&d).unwrap();
    let old = s.clone();
    assert!(
        s.action(
            &d,
            &core::Action::Fail {
                array: "/dev/md0".into(),
                device: "/dev/vda1".into()
            },
            false
        )
        .unwrap_err()
        .contains("terminal")
    );
    assert_eq!(s, old);
    assert!(s.set_command(&d, "a\nb".into(), false).is_err());
    assert_eq!(s, old);
    assert!(s.execute(&d, "sh -c id", false).is_err());
    assert_eq!(s, old);
}

#[test]
fn clearing_a_detached_failed_devices_bytes_cannot_pass_a_preservation_requirement() {
    let d = definition();
    let mut s = work::Session::new(&d).unwrap();
    complete(&d, &mut s, "/dev/vdc1", false);
    s.current
        .devices
        .get_mut("/dev/vdb1")
        .unwrap()
        .content
        .clear();
    s.current.validate().unwrap();
    d.bound(&s).unwrap();
    assert!(
        !d.checks(&s)[0].0,
        "detachment alone must not claim retained recovery bytes"
    );
}
#[test]
fn agreeing_replicas_with_the_wrong_contents_do_not_pass_the_retained_data_objective() {
    let d = definition();
    let mut s = work::Session::new(&d).unwrap();
    complete(&d, &mut s, "/dev/vdc1", false);
    for path in ["/dev/vda1", "/dev/vdc1"] {
        s.current.devices.get_mut(path).unwrap().content = support::data("wrong recovered content");
    }
    s.current.validate().unwrap();
    assert_eq!(s.current.health("/dev/md0").unwrap(), core::Health::Healthy);
    assert_eq!(
        d.checks(&s).iter().map(|r| r.0).collect::<Vec<_>>(),
        [true, true, false, true],
        "healthy membership must not stand in for preserved data"
    );
}
