//! Source-enabled VLAN tasks through the existing bounded native-editor session.
//! These are capability fixtures, not registered curriculum or learner records.
#[path = "support/vlan_host.rs"]
mod f;
use dlms_learning_lab::{nm_workspace as w, script_teaching::View};
fn task(id: &str, base: u8) -> w::Task {
    let change = |s: &str| {
        s.replace("10.20.", &format!("10.{base}."))
            .replace("10.21.", &format!("10.{}.", base + 1))
    };
    let env: dlms_learning_lab::nm_policy::Environment =
        serde_json::from_str(&change(&serde_json::to_string(&f::environment()).unwrap())).unwrap();
    let good: w::Files = f::files()
        .into_iter()
        .map(|(name, text)| {
            (
                name,
                change(&text)
                    .replace("000000000001", &format!("{base:012}"))
                    .replace("000000000002", &format!("{:012}", base + 1)),
            )
        })
        .collect();
    let mut starter = good.clone();
    let s = starter.get_mut("reports.nmconnection").unwrap();
    *s = s.replace("parent=enp1s0", "parent=enp2s0");
    let req = vec![
        w::Requirement::CurrentUnchanged {
            interface: "audit0".into(),
        },
        w::Requirement::FutureAddress {
            interface: "reports0".into(),
            address: dlms_learning_lab::ipv4_routing::Assignment {
                address: f::ip(&format!("10.{base}.0.2")),
                prefix: 24,
            },
        },
        w::Requirement::FuturePath {
            target: f::ip(&format!("10.{base}.0.50")),
            interface: "reports0".into(),
        },
        w::Requirement::FuturePath {
            target: f::ip(&format!("10.{}.0.50", base + 1)),
            interface: "audit0".into(),
        },
        w::Requirement::FutureNoDefault {
            interface: "audit0".into(),
        },
    ];
    let mut second: dlms_learning_lab::nm_policy::Environment = env.clone();
    second
        .vlan
        .as_mut()
        .unwrap()
        .fabric
        .hosts
        .get_mut("native")
        .unwrap()
        .up = false;
    // Audit moves to the primary NIC in both starter/current fixture bundles.
    // A correct handover must work when the alternate NIC is unavailable too.
    let s = starter.get_mut("audit.nmconnection").unwrap();
    *s = s.replace("parent=enp2s0", "parent=enp1s0");
    let mut current = good;
    let s = current.get_mut("audit.nmconnection").unwrap();
    *s = s.replace("parent=enp2s0", "parent=enp1s0");
    w::Task { id:id.into(), instructions:"Prepare the reporting VLAN for next startup and keep the current audit selection unchanged. Check both directions of the supplied paths, including an unavailable alternate parent.".into(), starter, protected_profiles:vec!["audit.nmconnection".into()], cases:vec![
        w::Case {current_bridge:None,current_controls:None,id:"native-return".into(),label:"Carrier up; alternate parent strips the return tag".into(),environment:env,current_files:current.clone(),initially_selected:vec![format!("00000000-0000-4000-8000-{:012}",base+1)],requirements:req.clone()},
        w::Case {current_bridge:None,current_controls:None,id:"alternate-unavailable".into(),label:"Alternate physical parent unavailable".into(),environment:second,current_files:current,initially_selected:vec![format!("00000000-0000-4000-8000-{:012}",base+1)],requirements:req}
    ] }
}
fn solved(t: &w::Task) -> w::Files {
    t.cases[0].current_files.clone()
}
fn definition() -> w::Definition {
    let example = task("vlan-capability-practice", 30);
    w::Definition {
        initial: task("vlan-capability-assignment", 20),
        example_worked: solved(&example),
        example,
    }
}
#[test]
fn taught_configuration_and_equivalent_explicit_settings_pass_all_contexts() {
    let d = definition();
    d.validate().unwrap();
    for t in [&d.initial, &d.example] {
        let mut files = solved(t);
        assert!(t.run(&files).unwrap().iter().all(w::CaseResult::passed));
        let s = files.get_mut("reports.nmconnection").unwrap();
        *s = s.replace("protocol=802.1Q\n", "").replace(
            "autoconnect=true",
            "autoconnect=true\nautoconnect-priority=0",
        );
        s.push_str("\n# A supported equivalent configuration, not exact text\n");
        assert!(t.run(&files).unwrap().iter().all(w::CaseResult::passed));
    }
}
#[test]
fn correct_ip_wrong_parent_produces_partial_directional_feedback_and_retry() {
    let t = definition().initial;
    let rows = t.run(&t.starter).unwrap();
    assert!(rows.iter().all(|r| !r.passed()));
    assert!(rows[0].checks.iter().any(|c| c.0));
    assert!(
        rows[0]
            .checks
            .iter()
            .any(|c| !c.0 && c.1.contains("outward") && c.1.contains("return"))
    );
    assert!(
        t.run(&solved(&t))
            .unwrap()
            .iter()
            .all(w::CaseResult::passed)
    );
}
#[test]
fn separate_practice_resume_reset_and_rechecked_credit_remain_content_bound() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let untouched = s.assignment.clone();
    s.practice.replace(&d.example_worked).unwrap();
    s.practice.run(&d.example).unwrap();
    assert!(s.practice.passed(&d.example));
    assert!(!d.solved(&s));
    assert_eq!(s.assignment, untouched);
    s.view = View::Assignment;
    s.assignment.replace(&solved(&d.initial)).unwrap();
    s.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&s));
    let saved = serde_json::to_string(&s).unwrap();
    let restored = w::Session::from_json(&saved, &d).unwrap();
    assert_eq!(s, restored);
    assert!(d.solved(&restored));
    s.assignment
        .edit(
            "reports.nmconnection",
            &d.initial.starter["reports.nmconnection"],
        )
        .unwrap();
    assert!(!d.solved(&s));
    assert!(s.assignment.tested.is_none());
    s.assignment.replace(&solved(&d.initial)).unwrap();
    assert!(!d.solved(&s));
    s.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&s));
    let reset = w::Session::new(&d).unwrap();
    assert!(!d.solved(&reset));
    assert_eq!(reset.assignment.draft, d.initial.starter);
}
#[test]
fn stop_and_mid_run_edit_cannot_publish_a_vlan_result() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.replace(&solved(&d.initial)).unwrap();
    let mut run = s.assignment.begin(&d.initial).unwrap();
    assert!(!run.tick());
    run.cancel();
    for _ in 0..w::MAX_TICKS {
        assert!(!run.tick());
    }
    assert!(run.results().is_none());
    assert!(s.assignment.finish(&d.initial, &run).is_err());
    let mut run = s.assignment.begin(&d.initial).unwrap();
    for _ in 0..w::MAX_TICKS {
        if run.tick() {
            break;
        }
    }
    assert!(run.done());
    s.assignment
        .edit("reports.nmconnection", "# unfinished edit")
        .unwrap();
    assert!(s.assignment.finish(&d.initial, &run).is_err());
    assert!(!d.solved(&s));
}
#[test]
fn incorrect_tag_unsupported_settings_and_forged_proof_never_earn_credit() {
    let d = definition();
    for (from, to) in [
        ("id=30\n", "id=99\n"),
        ("flags=1", "flags=0"),
        ("protocol=802.1Q", "protocol=802.1ad"),
    ] {
        let mut fs = solved(&d.initial);
        let s = fs.get_mut("reports.nmconnection").unwrap();
        *s = s.replace(from, to);
        assert!(d.initial.run(&fs).unwrap().iter().all(|r| !r.passed()));
    }
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.tested = Some(s.assignment.draft.clone());
    let restored = w::Session::from_json(&serde_json::to_string(&s).unwrap(), &d).unwrap();
    assert!(!d.solved(&restored));
    let mut d2 = d.clone();
    d2.initial.cases[1].environment.vlan = None;
    assert!(d2.validate().is_err());
}
#[test]
fn foreign_files_and_oversized_drafts_cannot_replace_saved_work() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let before = s.clone();
    assert!(
        s.assignment
            .edit("../host.nmconnection", "not read")
            .is_err()
    );
    assert!(
        s.assignment
            .edit("reports.nmconnection", &"x".repeat(16385))
            .is_err()
    );
    assert_eq!(s, before);
}
