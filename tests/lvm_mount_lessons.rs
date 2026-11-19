//! Actual authored lessons, normal catalog integration and preserved old values.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    lvm_workspace::{self as w, mount_flow as f},
    pack,
    progress::{self, Attempt},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};
const IDS: [&str; 2] = ["lpic2-lvm-mount-repair", "lpic2-lvm-mounted-handover"];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted335().unwrap())
}
fn lab(id: &str) -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == id).unwrap()
}
fn definition(s: &pack::Scenario) -> &w::Definition {
    let Goal::Lvm { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn sim(a: &mut Attempt) -> &mut w::Session {
    let State::Lvm { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn catalog_preserves_all_previous_values_order_and_units_and_adds_only_two_distinct_tasks() {
    let before = pack::accepted333().unwrap();
    assert_eq!(before.len(), 333);
    assert_eq!(catalog().len(), 335);
    let ids = before
        .iter()
        .map(|s| s.id.as_str())
        .collect::<BTreeSet<_>>();
    let prior = catalog()
        .iter()
        .filter(|s| ids.contains(s.id.as_str()))
        .collect::<Vec<_>>();
    for (a, b) in before.iter().zip(prior) {
        assert_eq!(
            serde_json::to_value(a).unwrap(),
            serde_json::to_value(b).unwrap(),
            "{}",
            a.id
        );
    }
    let units = course::accepted335(catalog()).unwrap();
    assert_eq!(units.len(), 115);
    let old = course::accepted333(&before).unwrap();
    assert_eq!(old.len(), 114);
    let unit_ids = old.iter().map(|u| u.id.as_str()).collect::<BTreeSet<_>>();
    let prior = units
        .iter()
        .filter(|u| unit_ids.contains(u.id.as_str()))
        .collect::<Vec<_>>();
    for (a, b) in old.iter().zip(prior) {
        assert_eq!(
            serde_json::to_value(a).unwrap(),
            serde_json::to_value(b).unwrap()
        );
    }
    let mut tracks = BTreeMap::new();
    for s in catalog() {
        *tracks.entry(s.track.as_str()).or_insert(0) += 1;
    }
    assert_eq!(
        tracks,
        BTreeMap::from([
            ("CCNA", 74),
            ("LPIC-1", 114),
            ("LPIC-2", 70),
            ("Bash/Shell", 77)
        ])
    );
    let unit = units
        .iter()
        .find(|u| u.id == "lvm-filesystem-mounting")
        .unwrap();
    assert_eq!(unit.labs, IDS);
    assert_eq!(unit.prerequisites, vec!["lvm-device-selection"]);
    let pos = units
        .iter()
        .position(|u| u.id == "lvm-filesystem-mounting")
        .unwrap();
    assert_eq!(units[pos - 1].id, "lvm-device-selection");
    assert_eq!(lab(IDS[0]).prerequisites, vec!["lpic2-lvm-devices-file"]);
    assert_eq!(lab(IDS[1]).prerequisites, vec![IDS[0]]);
}
#[test]
fn taught_methods_reach_both_real_assignments_and_separate_practice_repairs() {
    for id in IDS {
        let s = lab(id);
        let d = definition(s);
        d.validate().unwrap();
        w::source(s).unwrap();
        let mut a = Attempt::new(s, 1);
        assert!(!progress::current_solved(&a, s));
        let assignment = sim(&mut a)
            .mount_runtime
            .as_ref()
            .unwrap()
            .assignment
            .clone();
        let p = d.lesson.mount_practice.as_ref().unwrap();
        for step in &p.worked {
            sim(&mut a).mount_action(d, &step.operation, true).unwrap();
        }
        assert!(d.practice_checks(sim(&mut a)).iter().all(|r| r.0));
        assert_eq!(
            sim(&mut a).mount_runtime.as_ref().unwrap().assignment,
            assignment
        );
        assert!(!progress::current_solved(&a, s));
        sim(&mut a).mount_action(d, &p.experiment, true).unwrap();
        assert!(!d.practice_checks(sim(&mut a)).iter().all(|r| r.0));
        for op in &p.repair {
            sim(&mut a).mount_action(d, op, true).unwrap();
        }
        assert!(d.practice_checks(sim(&mut a)).iter().all(|r| r.0));
        for op in &d.mount_task.as_ref().unwrap().reference {
            sim(&mut a).mount_action(d, op, false).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        w::context(&a, s).unwrap();
    }
}
#[test]
fn alternate_uuid_name_option_order_and_mount_order_pass_actual_outcomes() {
    let s = lab(IDS[0]);
    let d = definition(s);
    let c = &d.mount_task.as_ref().unwrap().context;
    let mut a = Attempt::new(s, 1);
    let source = |id| {
        format!(
            "UUID={}",
            c.inventory
                .filesystems
                .iter()
                .find(|f| f.original_volume == id)
                .unwrap()
                .uuid
        )
    };
    let text = format!(
        "# audit before reports\n{} /srv/audit ext4 auto,ro 0 2\n{} /srv/reports ext4 rw,auto 0 2\n",
        source(5),
        source(4)
    );
    sim(&mut a)
        .mount_action(d, &f::Operation::SavePolicy { text }, false)
        .unwrap();
    for text in [
        "umount /srv/audit",
        "umount /srv/reports",
        "mount /srv/audit",
        "mount /srv/reports",
    ] {
        sim(&mut a).execute(d, text, false).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    let s = lab(IDS[1]);
    let d = definition(s);
    let mut a = Attempt::new(s, 1);
    for text in [
        "vgrename intake production",
        "lvrename production draft reports",
    ] {
        sim(&mut a).execute(d, text, false).unwrap();
    }
    sim(&mut a)
        .mount_action(
            d,
            &f::Operation::SavePolicy {
                text: "/dev/production/reports /srv/reports ext4 rw,auto 0 2\n".into(),
            },
            false,
        )
        .unwrap();
    assert!(progress::current_solved(&a, s));
    sim(&mut a)
        .execute(d, "umount /srv/reports", false)
        .unwrap();
    assert!(!progress::current_solved(&a, s));
    sim(&mut a).execute(d, "mount /srv/reports", false).unwrap();
    assert!(progress::current_solved(&a, s));
}
#[test]
fn plausible_wrong_and_partial_repairs_do_not_pass_or_hide_current_wrong_data() {
    let s = lab(IDS[0]);
    let d = definition(s);
    let mut a = Attempt::new(s, 1);
    let reference = &d.mount_task.as_ref().unwrap().reference;
    sim(&mut a).mount_action(d, &reference[0], false).unwrap();
    assert!(!progress::current_solved(&a, s));
    let rows = d.checks(sim(&mut a));
    assert!(rows.iter().any(|r| r.0));
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.contains("another filesystem"))
    );
    for op in &reference[1..] {
        sim(&mut a).mount_action(d, op, false).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    for text in [
        "/dev/ledger/draft /srv/reports ext4 defaults 0 2\n/dev/ledger/reference /srv/audit ext4 defaults 0 2\n",
        "/dev/ledger/reference /srv/reports ext4 defaults 0 2\n/dev/ledger/draft /srv/audit ext4 ro 0 2\n",
        "/dev/ledger/draft /srv/reports xfs defaults 0 2\n/dev/ledger/reference /srv/audit ext4 ro 0 2\n",
    ] {
        sim(&mut a)
            .mount_action(d, &f::Operation::SavePolicy { text: text.into() }, false)
            .unwrap();
        assert!(!progress::current_solved(&a, s));
    }
    let s = lab(IDS[1]);
    let d = definition(s);
    let mut a = Attempt::new(s, 1);
    for text in [
        "lvrename intake draft reports",
        "vgrename intake production",
    ] {
        sim(&mut a).execute(d, text, false).unwrap();
    }
    let output = sim(&mut a)
        .execute(d, "cat /srv/reports/keep.txt", false)
        .unwrap();
    assert!(output.starts_with("intake/draft:"));
    assert!(!progress::current_solved(&a, s));
    assert!(
        d.checks(sim(&mut a))
            .iter()
            .any(|r| !r.0 && r.1.contains("Saved mount configuration"))
    );
}
#[test]
fn invalidating_edits_and_current_state_changes_clear_success_and_both_drafts_resume() {
    for id in IDS {
        let s = lab(id);
        let d = definition(s);
        let mut a = Attempt::new(s, 1);
        for op in &d.mount_task.as_ref().unwrap().reference {
            sim(&mut a).mount_action(d, op, false).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        sim(&mut a).checked = true;
        sim(&mut a)
            .edit_mount(d, "# unfinished assignment\nUUID=".into(), false)
            .unwrap();
        sim(&mut a)
            .edit_mount(d, "# unfinished practice\nUUID=".into(), true)
            .unwrap();
        assert!(!progress::current_solved(&a, s));
        assert!(!sim(&mut a).checked);
        let before = sim(&mut a).clone();
        let mut a: Attempt = serde_json::from_slice(&serde_json::to_vec(&a).unwrap()).unwrap();
        w::context(&a, s).unwrap();
        assert_eq!(sim(&mut a), &before);
        sim(&mut a).reset_example(d).unwrap();
        assert_eq!(
            sim(&mut a).mount_runtime.as_ref().unwrap().assignment,
            before.mount_runtime.as_ref().unwrap().assignment
        );
        assert!(!progress::current_solved(&Attempt::new(s, 2), s));
    }
}
#[test]
fn authoring_rejects_unreachable_and_malformed_mount_pack_without_crediting_old_sources() {
    let s = lab(IDS[0]);
    for change in 0..4 {
        let mut bad = s.clone();
        let Goal::Lvm { definition } = &mut bad.goal else {
            panic!()
        };
        match change {
            0 => definition.mount_task.as_mut().unwrap().context.targets[0].original_volume = 99,
            1 => definition.lesson.mount_practice = None,
            2 => {
                definition.mount_task.as_mut().unwrap().reference = vec![f::Operation::Command {
                    text: "findmnt".into(),
                }]
            }
            _ => {
                definition
                    .lesson
                    .mount_practice
                    .as_mut()
                    .unwrap()
                    .experiment = f::Operation::Command {
                    text: "findmnt".into(),
                }
            }
        }
        assert!(
            pack::load(
                &serde_json::to_string(&pack::Pack {
                    schema_version: 1,
                    scenarios: vec![bad]
                })
                .unwrap()
            )
            .is_err()
        );
    }
    for old in pack::accepted333().unwrap() {
        if let State::Lvm { simulation } = &old.initial {
            assert!(simulation.mount_runtime.is_none());
        }
    }
}
#[test]
fn all_three_views_teach_prerequisites_and_keep_expected_actual_and_worked_material_distinct() {
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../scenarios/presentation/lpic2.json")).unwrap();
    for id in IDS {
        let s = lab(id);
        let d = definition(s);
        let concepts = d
            .lesson
            .concepts
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        for term in [
            "filesystem",
            "mountpoint",
            "six fields",
            "Save configuration",
            "UUID=",
            "findmnt",
            "cat",
        ] {
            assert!(concepts.contains(term), "{id} {term}");
        }
        let row = rows.iter().find(|r| r["id"] == id).unwrap();
        assert_eq!(
            row["requirements"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect::<Vec<_>>()
                .join(" "),
            s.objective
        );
        assert_eq!(row["practice"].as_array().unwrap().len(), 3);
        assert!(
            d.lesson
                .mount_practice
                .as_ref()
                .unwrap()
                .worked
                .iter()
                .any(|s| matches!(s.operation, f::Operation::SavePolicy { .. }))
        );
        assert!(s.guide.is_empty());
        assert!(!s.objective.contains("UUID=3bde"));
    }
    assert!(!engine::solved(&lab(IDS[0]).initial, &lab(IDS[0]).goal));
}
