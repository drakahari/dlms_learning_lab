//! Original access tasks receive separate typed practice; no host identity API.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    legacy_access_teaching::{self as access, Learning, Snapshot},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use serde_json::Value;
fn lab(id: &str) -> pack::Scenario {
    pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.id == id)
        .unwrap()
}
fn practice(id: &str) -> (pack::Scenario, Learning) {
    let s = lab(id);
    let p = Learning::new(s.legacy_access_teaching.as_ref().unwrap()).unwrap();
    (s, p)
}
fn execute(p: &mut Learning, l: &access::Lesson, commands: &[String]) {
    for c in commands {
        p.command = c.clone();
        p.submit(l).unwrap();
    }
}
#[test]
fn both_separate_examples_reference_experiment_repair_and_helpers_match_current_goals() {
    for id in ["lpic1-permissions", "lpic1-traversal"] {
        let (s, mut p) = practice(id);
        let l = s.legacy_access_teaching.as_ref().unwrap();
        assert!(l.valid_for(&s.initial, &s.goal));
        assert!(!p.checks(l).iter().all(|r| r.0));
        execute(&mut p, l, &l.reference);
        assert!(p.checks(l).iter().all(|r| r.0));
        execute(&mut p, l, &l.try_commands);
        assert!(!p.checks(l)[0].0);
        assert!(p.checks(l)[0].1.contains("is blocked"));
        assert!(p.checks(l).iter().all(|r| r.1.len() <= 512));
        execute(&mut p, l, &l.repair_commands);
        assert!(p.checks(l).iter().all(|r| r.0));
        assert!(
            l.commands
                .iter()
                .all(|c| c.simulator_only == (c.command.starts_with("test-access ")))
        );
        let assignment = serde_json::to_vec(&s.initial).unwrap();
        p.selected_action = 0;
        let before = p.current.clone();
        let transcript = p.transcript.clone();
        p.insert(l).unwrap();
        assert_eq!(p.current, before);
        assert_eq!(p.transcript, transcript);
        assert_eq!(p.command, l.commands[0].command);
        assert_eq!(serde_json::to_vec(&s.initial).unwrap(), assignment);
        let mut mislabeled = l.clone();
        mislabeled
            .commands
            .iter_mut()
            .find(|c| c.command.starts_with("test-access "))
            .unwrap()
            .simulator_only = false;
        assert!(mislabeled.validate().is_err());
    }
    let mut wrong_kind: Value =
        serde_json::from_str(include_str!("../scenarios/starter-v1.json")).unwrap();
    wrong_kind["scenarios"][0]["legacy_access_teaching"] =
        serde_json::to_value(lab("lpic1-permissions").legacy_access_teaching.unwrap()).unwrap();
    assert!(pack::load(&serde_json::to_string(&wrong_kind).unwrap()).is_err());
}
#[test]
fn ownership_alternative_and_owner_precedence_are_real_access_not_exact_modes() {
    let (s, mut p) = practice("lpic1-permissions");
    let l = s.legacy_access_teaching.as_ref().unwrap();
    execute(
        &mut p,
        l,
        &[
            "chown dana:dana /srv/brief.txt".into(),
            "chmod 600 /srv/brief.txt".into(),
        ],
    );
    assert!(p.checks(l).iter().all(|r| r.0));
    execute(
        &mut p,
        l,
        &[
            "chown dana:reviewers /srv/brief.txt".into(),
            "chmod 060 /srv/brief.txt".into(),
        ],
    );
    // Owner bits0 take precedence over a matching reviewers group with6.
    assert!(!p.checks(l)[0].0);
    assert_eq!(
        p.current
            .execute("test-access dana /srv/brief.txt rw")
            .unwrap(),
        "Permission denied"
    );
    execute(&mut p, l, &["chmod 600 /srv/brief.txt".into()]);
    assert!(p.checks(l).iter().all(|r| r.0));
    execute(&mut p, l, &["chmod 666 /srv/brief.txt".into()]);
    assert!(p.checks(l)[0].0 && !p.checks(l)[1].0);
    assert!(p.checks(l)[1].1.contains("Another modeled user"));
}
#[test]
fn traversal_needs_membership_and_search_but_no_unasked_listing_write_or_exclusivity() {
    let (s, mut p) = practice("lpic1-traversal");
    let l = s.legacy_access_teaching.as_ref().unwrap();
    p.current.execute("usermod -aG inspectors evan").unwrap();
    assert!(!p.checks(l)[0].0);
    p.current.execute("chmod 710 /srv/review").unwrap();
    assert!(p.checks(l).iter().all(|r| r.0));
    assert_eq!(
        p.current
            .execute("test-access colleague /srv/review/plan.txt r")
            .unwrap(),
        "Access allowed"
    );
    let mut other = Learning::new(l).unwrap();
    other.current.execute("chmod 750 /srv/review").unwrap();
    assert!(!other.checks(l)[0].0);
    other
        .current
        .execute("usermod -aG inspectors evan")
        .unwrap();
    assert!(other.checks(l).iter().all(|r| r.0));
    let mut owner = Learning::new(l).unwrap();
    for c in [
        "chown evan:evan /srv/review/plan.txt",
        "chmod 600 /srv/review/plan.txt",
        "chmod 755 /srv/review",
    ] {
        owner.current.execute(c).unwrap();
    }
    assert!(owner.checks(l).iter().all(|r| r.0));
    let Goal::Access {
        exclusive, private, ..
    } = s.goal
    else {
        panic!()
    };
    assert!(!exclusive && private);
}
#[test]
fn failed_commands_foreign_context_and_direct_oversize_inputs_are_atomic() {
    let (s, mut p) = practice("lpic1-permissions");
    let l = s.legacy_access_teaching.as_ref().unwrap();
    let original = p.current.clone();
    for c in [
        "chmod g+=r /srv/brief.txt",
        "chmod 888 /srv/brief.txt",
        "chown missing:reviewers /srv/brief.txt",
        "usermod -aG unknown dana",
        "sh -c chmod",
    ] {
        assert!(p.current.execute(c).is_err());
        assert_eq!(p.current, original);
    }
    assert!(p.current.execute(&"x".repeat(4097)).is_err());
    assert_eq!(p.current, original);
    assert!(
        !p.valid_for(
            lab("lpic1-traversal")
                .legacy_access_teaching
                .as_ref()
                .unwrap()
        )
    );
    let mut primary = p.clone();
    primary.current.users.get_mut("dana").unwrap().swap(0, 1);
    assert!(primary.valid());
    assert!(!primary.valid_for(l));
    let mut large = original.clone();
    for i in 0..65 {
        large.nodes.insert(
            format!("/extra{i}"),
            original.nodes["/srv/brief.txt"].clone(),
        );
    }
    assert!(!large.valid());
    let raw = serde_json::to_value(&large).unwrap();
    assert!(large.execute("show").is_err());
    assert_eq!(serde_json::to_value(large).unwrap(), raw);
    assert!(
        Snapshot::from_state(&State::Logs {
            files: Default::default(),
            result: String::new()
        })
        .is_none()
    );
}
#[test]
fn typed_snapshot_readers_bound_counts_keys_values_groups_and_duplicates() {
    let (_, p) = practice("lpic1-permissions");
    let raw = serde_json::to_value(&p.current).unwrap();
    let mut huge = raw.clone();
    huge["nodes"] = serde_json::json!({});
    huge["nodes"]["/"] = raw["nodes"]["/"].clone();
    for i in 0..63 {
        huge["nodes"][format!("/extra{i}")] = raw["nodes"]["/srv/brief.txt"].clone();
    }
    huge["nodes"]["/zz-last-invalid-body"] =
        serde_json::json!({"not_an_entry":"unparsed typed body"});
    assert!(
        serde_json::from_value::<Snapshot>(huge)
            .unwrap_err()
            .to_string()
            .contains("Practice map entry limit")
    );
    let duplicate = r#"{"nodes":{"/":ENTRY,"/":{"not_an_entry":true}},"users":USERS}"#
        .replace("ENTRY", &serde_json::to_string(&raw["nodes"]["/"]).unwrap())
        .replace("USERS", &serde_json::to_string(&raw["users"]).unwrap());
    assert!(
        serde_json::from_str::<Snapshot>(&duplicate)
            .unwrap_err()
            .to_string()
            .contains("Duplicate practice map key")
    );
    let mut groups = raw.clone();
    groups["users"]["dana"] = serde_json::to_value(vec!["x"; 17]).unwrap();
    assert!(serde_json::from_value::<Snapshot>(groups).is_err());
    let mut big_key = raw.clone();
    big_key["nodes"][format!("/{}", "x".repeat(256))] = raw["nodes"]["/srv/brief.txt"].clone();
    assert!(serde_json::from_value::<Snapshot>(big_key).is_err());
    let mut owner = raw;
    owner["nodes"]["/srv/brief.txt"]["owner"] = "x".repeat(33).into();
    assert!(serde_json::from_value::<Snapshot>(owner).is_err());
}
#[test]
fn max_shape_roundtrip_output_and_transcript_eviction_remain_bounded() {
    let s = lab("lpic1-permissions");
    let mut l = s.legacy_access_teaching.unwrap();
    while l.fixture.nodes.len() < 64 {
        let name = format!("/{}{:02}", "x".repeat(250), l.fixture.nodes.len());
        l.fixture
            .nodes
            .insert(name, l.fixture.nodes["/srv/brief.txt"].clone());
    }
    assert!(l.fixture.valid());
    let bytes = serde_json::to_vec_pretty(&l.fixture).unwrap();
    assert_eq!(
        serde_json::from_slice::<Snapshot>(&bytes).unwrap(),
        l.fixture
    );
    assert_eq!(Snapshot::decode(&bytes).unwrap(), l.fixture);
    assert!(Snapshot::decode(&vec![b' '; 1024 * 1024 + 1]).is_err());
    let mut p = Learning::new(&l).unwrap();
    for _ in 0..20 {
        p.command = "show".into();
        p.submit(&l).unwrap();
    }
    assert!(p.valid_for(&l));
    assert!(p.transcript.iter().map(String::len).sum::<usize>() <= 131072);
    assert!(
        p.transcript
            .iter()
            .any(|s| s.starts_with("[Earlier practice output omitted]"))
    );
    assert!(p.current.execute("show").unwrap().len() <= 32768);
    let mut bad = serde_json::to_value(&p).unwrap();
    bad["command"] = "é".repeat(2049).into();
    assert!(serde_json::from_value::<Learning>(bad).is_err());
}
#[test]
fn old_assignment_imports_keep_legacy_bounds_and_existing_first_open_view() {
    let s = lab("lpic1-permissions");
    let l = s.legacy_access_teaching.as_ref().unwrap();
    let mut a = Attempt::new(&s, 1);
    workspace::prepare(&mut a);
    // Reachable legacy work and raw command survive adding the independent example.
    a.workspace.as_mut().unwrap().command = "chmod 'unfinished".into();
    engine::execute(&mut a.state, "chmod 660 /srv/report.txt").unwrap();
    let before = a.state.clone();
    access::prepare(&mut a, l, false).unwrap();
    assert_eq!(a.state, before);
    assert_eq!(a.workspace.as_ref().unwrap().command, "chmod 'unfinished");
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .legacy_access_learning
            .as_ref()
            .unwrap()
            .view,
        dlms_learning_lab::script_teaching::View::Assignment
    );
    let State::Unix { nodes, users } = &mut a.state else {
        panic!()
    };
    let sample = nodes["/srv/report.txt"].clone();
    for i in 0..80 {
        nodes.insert(format!("/legacy{i}"), sample.clone());
    }
    users.insert(
        "long-legacy-account-name-not-in-new-practice".into(),
        vec!["root".into()],
    );
    users.insert("dana".into(), vec!["root".into()]);
    assert!(pack::validate_state(&a.state).is_ok());
    assert!(Snapshot::from_state(&a.state).is_none());
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    let bytes = serde_json::to_vec(&p).unwrap();
    assert!(progress::decode(&bytes, &pack::bundled().unwrap()).is_ok());
}
#[test]
fn all_original116_specifications_goals_solutions_units_and_versions_are_preserved() {
    let old: Value =
        serde_json::from_str(include_str!("fixtures/legacy-access-logs-original116.json")).unwrap();
    let all = pack::bundled().unwrap();
    assert_eq!(old["scenarios"].as_array().unwrap().len(), 116);
    for row in old["scenarios"].as_array().unwrap() {
        let s = all
            .iter()
            .find(|s| Some(s.id.as_str()) == row["id"].as_str())
            .unwrap();
        assert_eq!(s.version as u64, row["version"].as_u64().unwrap());
        let initial: State = serde_json::from_value(row["initial"].clone()).unwrap();
        assert_eq!(s.initial, initial);
        let goal: Goal = serde_json::from_value(row["goal"].clone()).unwrap();
        assert_eq!(
            serde_json::to_value(&s.goal).unwrap(),
            serde_json::to_value(goal).unwrap()
        );
        assert_eq!(serde_json::to_value(&s.solution).unwrap(), row["solution"]);
        assert_eq!(s.unit, row["unit"].as_str().unwrap());
        if s.id != "lpic1-traversal" {
            assert_eq!(s.objective, row["objective"].as_str().unwrap());
        } else {
            assert_eq!(
                s.objective,
                "Give blair read access to /srv/project/notes.txt without public file permissions. Repair group membership and traversal as needed."
            );
        }
    }
    let mut current: Value =
        serde_json::from_str(include_str!("../scenarios/units-v1.json")).unwrap();
    let units = current["units"].as_array_mut().unwrap();
    assert_eq!(units.len(), 30);
    assert_eq!(units.iter().filter(|u| u["id"] == "dns-reverse").count(), 1);
    assert_eq!(
        units
            .iter()
            .filter(|u| u["id"] == "shell-option-interfaces")
            .count(),
        1
    );
    units.retain(|u| u["id"] != "dns-reverse" && u["id"] != "shell-option-interfaces");
    assert_eq!(current, old["units"]);
}
