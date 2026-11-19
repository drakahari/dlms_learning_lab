//! Typed name goals; no learner host command execution or authored-content credit.
use dlms_learning_lab::{lvm, lvm_workspace as work};
use serde_json::{Value, json};
fn original() -> work::Definition {
    let p: Value =
        serde_json::from_str(include_str!("../scenarios/lvm-activation-lessons-v1.json")).unwrap();
    serde_json::from_value(p["scenarios"][1]["goal"]["definition"].clone()).unwrap()
}
fn identity(s: &mut lvm::State) {
    let mut next = 1;
    let groups = s
        .groups
        .keys()
        .map(|g| {
            let id = next;
            next += 1;
            (g.clone(), id)
        })
        .collect();
    let volumes = s
        .groups
        .iter()
        .flat_map(|(g, d)| d.volumes.keys().map(move |v| format!("{g}/{v}")))
        .map(|v| {
            let id = next;
            next += 1;
            (v, id)
        })
        .collect();
    s.identity = Some(lvm::rename::Identity {
        groups,
        volumes,
        next,
    });
}
fn fixture() -> work::Definition {
    let mut d = original();
    identity(&mut d.initial);
    identity(&mut d.lesson.fixture);
    d.availability = None;
    d.lesson.availability = None;
    d.renaming=Some(serde_json::from_value(json!({"groups":[{"source":"cold","name":"warehouse"}],"volumes":[{"source":"cold/archive","name":"records"}]})).unwrap());
    d.lesson.renaming=Some(serde_json::from_value(json!({"groups":[{"source":"workshop","name":"studio"}],"volumes":[{"source":"workshop/notes","name":"journal"}]})).unwrap());
    d.lesson.worked = vec![
        work::Command {
            text: "vgrename workshop studio".into(),
            explanation: "Move the group's name without copying storage.".into(),
        },
        work::Command {
            text: "lvrename studio notes journal".into(),
            explanation: "Rename the original LV at its current group name.".into(),
        },
    ];
    d.lesson.experiment = "lvrename studio journal notes".into();
    d.lesson.repair = "lvrename studio notes journal".into();
    d
}
fn all(rows: &[(bool, String)]) -> bool {
    !rows.is_empty() && rows.iter().all(|r| r.0)
}
fn solve(s: &mut work::Session, d: &work::Definition) {
    for c in ["lvrename cold archive records", "vgrename cold warehouse"] {
        s.execute(d, c, false).unwrap();
    }
}
#[test]
fn partial_names_diagnose_the_original_objects_and_either_order_passes() {
    let d = fixture();
    d.validate().unwrap();
    let mut s = work::Session::new(&d).unwrap();
    assert!(!all(&d.checks(&s)));
    s.checked = true;
    s.transcript.push(work::Row {
        input: "lvs".into(),
        output: "all passed".into(),
    });
    assert!(!all(&d.checks(&s)));
    s.execute(&d, "vgrename cold warehouse", false).unwrap();
    let rows = d.checks(&s);
    assert!(!all(&rows));
    assert!(
        rows.iter()
            .any(|r| r.0 && r.1.contains("required name warehouse"))
    );
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.contains("warehouse/archive"))
    );
    s.execute(&d, "lvrename warehouse/archive records", false)
        .unwrap();
    assert!(all(&d.checks(&s)));
    let mut alternate = work::Session::new(&d).unwrap();
    solve(&mut alternate, &d);
    assert_eq!(s.current, alternate.current);
    assert!(all(&d.checks(&alternate)));
}
#[test]
fn wrong_name_and_changed_availability_clear_success_and_can_be_repaired() {
    let d = fixture();
    let mut s = work::Session::new(&d).unwrap();
    solve(&mut s, &d);
    s.checked = true;
    s.execute(&d, "lvrename warehouse records temporary", false)
        .unwrap();
    assert!(!s.checked);
    assert!(!all(&d.checks(&s)));
    assert!(
        d.checks(&s)
            .iter()
            .any(|r| !r.0 && r.1.contains("warehouse/temporary"))
    );
    s.execute(&d, "lvrename warehouse temporary records", false)
        .unwrap();
    assert!(all(&d.checks(&s)));
    s.execute(&d, "vgchange -ay warehouse", false).unwrap();
    assert!(!all(&d.checks(&s)));
    assert!(
        d.checks(&s)
            .iter()
            .any(|r| !r.0 && r.1.contains("active/inactive"))
    );
    s.execute(&d, "vgchange -an warehouse", false).unwrap();
    assert!(all(&d.checks(&s)));
}
#[test]
fn extra_storage_growth_and_changed_contents_do_not_hide_behind_correct_names() {
    let d = fixture();
    for c in [
        "lvcreate -l 1 -n extra warehouse",
        "lvextend -l +1 warehouse/records",
    ] {
        let mut s = work::Session::new(&d).unwrap();
        solve(&mut s, &d);
        s.execute(&d, c, false).unwrap();
        assert!(!all(&d.checks(&s)), "{c}");
        assert!(
            d.checks(&s)
                .iter()
                .any(|r| !r.0 && r.1.contains("allocations"))
        );
    }
    let mut s = work::Session::new(&d).unwrap();
    solve(&mut s, &d);
    s.current
        .groups
        .get_mut("live")
        .unwrap()
        .volumes
        .get_mut("ledger")
        .unwrap()
        .filesystem
        .as_mut()
        .unwrap()
        .files
        .get_mut("/keep.txt")
        .unwrap()
        .contents = "changed".into();
    d.bound(&s).unwrap();
    assert!(!all(&d.checks(&s)));
    assert!(
        d.checks(&s)
            .iter()
            .any(|r| !r.0 && r.1.contains("contents changed"))
    );
}
#[test]
fn raw_lookalike_with_required_name_cannot_replace_the_original_object() {
    let mut d = fixture();
    d.renaming = Some(
        serde_json::from_value(
            json!({"groups":[],"volumes":[{"source":"staging/buffer","name":"batch"}]}),
        )
        .unwrap(),
    );
    let mut s = work::Session::new(&d).unwrap();
    s.execute(&d, "lvremove -y staging/buffer", false).unwrap();
    s.execute(&d, "lvcreate -l 1 -n batch staging", false)
        .unwrap();
    assert!(s.current.volume("staging/batch").is_ok());
    assert!(!all(&d.checks(&s)));
    assert!(
        d.checks(&s)
            .iter()
            .any(|r| !r.0 && r.1.contains("original LV missing"))
    );
    let mut s = work::Session::new(&d).unwrap();
    s.execute(&d, "lvrename staging buffer batch", false)
        .unwrap();
    assert!(all(&d.checks(&s)));
}
#[test]
fn drafts_practice_reset_and_safe_roundtrip_remain_separate() {
    let d = fixture();
    let mut s = work::Session::new(&d).unwrap();
    s.set_command(&d, "my unrun assignment draft".into(), false)
        .unwrap();
    s.insert(&d, 0).unwrap();
    assert_eq!(s.current, d.initial);
    assert_eq!(s.example, d.lesson.fixture);
    for c in &d.lesson.worked {
        s.execute(&d, &c.text, true).unwrap();
    }
    assert!(all(&d.practice_checks(&s)));
    assert!(!all(&d.checks(&s)));
    s.execute(&d, &d.lesson.experiment, true).unwrap();
    assert!(!all(&d.practice_checks(&s)));
    s.execute(&d, &d.lesson.repair, true).unwrap();
    assert!(all(&d.practice_checks(&s)));
    solve(&mut s, &d);
    let copy: work::Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    d.bound(&copy).unwrap();
    assert_eq!(copy, s);
    assert!(all(&d.checks(&copy)));
    s.reset_example(&d).unwrap();
    assert_eq!(s.example, d.lesson.fixture);
    assert!(all(&d.checks(&s)));
    assert_eq!(s.command, "my unrun assignment draft");
    let reset = work::Session::new(&d).unwrap();
    assert_eq!(reset.current, d.initial);
    assert!(!all(&d.checks(&reset)));
}
#[test]
fn invalid_plans_and_old_goal_identity_injection_fail_without_weakening_old_binding() {
    let d = fixture();
    for mode in 0..9 {
        let mut raw = serde_json::to_value(&d).unwrap();
        match mode {
            0 => raw["renaming"]["groups"][0]["name"] = json!("live"),
            1 => raw["renaming"]["groups"][0]["source"] = json!("missing"),
            2 => raw["renaming"]["volumes"][0]["name"] = json!("archive"),
            3 => raw["renaming"]["volumes"][0]["source"] = json!("/dev/cold/archive"),
            4 => raw["renaming"]["volumes"][0]["name"] = json!("../escape"),
            5 => raw["renaming"]["groups"][0]["name"] = json!("cold"),
            6 => {
                raw["renaming"]["volumes"] = json!([{"source":"cold/archive","name":"one"},{"source":"cold/archive","name":"two"}])
            }
            7 => raw["initial"]["identity"] = Value::Null,
            _ => raw["availability"] = json!({"groups":[{"name":"cold","active":true}]}),
        }
        assert!(
            serde_json::from_value::<work::Definition>(raw)
                .unwrap()
                .shape()
                .is_err(),
            "mode{mode}"
        );
    }
    let original = original();
    let mut old = work::Session::new(&original).unwrap();
    old.current = d.initial.clone();
    assert!(original.bound(&old).is_err());
    let mut bad = original.clone();
    bad.initial = d.initial.clone();
    assert!(bad.shape().is_err());
}
