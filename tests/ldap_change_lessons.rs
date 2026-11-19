use dlms_learning_lab::{
    backup, course,
    engine::{self, State},
    ldap_change_workspace as w, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicUsize},
};
const IDS: [&str; 2] = ["lpic2-ldap-person-import", "lpic2-ldap-contact-maintenance"];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::ccna_pilot().unwrap())
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    w::context(&a, s).unwrap();
    a
}
#[test]
fn real_catalog_retains_all362_scenarios_and129_units_with_two_substantive_tasks() {
    let c = catalog();
    assert_eq!(c.len(), 366);
    let old = pack::accepted362().unwrap();
    assert_eq!(
        serde_json::to_value(&c[..362]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let units = course::ccna_pilot(c).unwrap();
    assert_eq!(units.len(), 131);
    let retained: Vec<_> = units
        .iter()
        .filter(|u| u.id != "ldap-directory-changes" && u.id != "ldap-utility-work" && u.id != "ipv6-interface-identifiers" && u.id != "linux-boot-entry-recovery")
        .collect();
    assert_eq!(
        serde_json::to_value(retained).unwrap(),
        serde_json::to_value(course::accepted362(&old).unwrap()).unwrap()
    );
    for id in IDS {
        let s = c.iter().find(|s| s.id == id).unwrap();
        let d = w::source(s).unwrap();
        assert_ne!(d.initial.reference, d.example.reference);
        assert_ne!(d.initial.cases, d.example.cases);
        assert!(!engine::solved(&s.initial, &s.goal));
        assert_eq!(d.initial.cases.len(), 3);
    }
    assert!(
        units
            .iter()
            .position(|u| u.id == "ldap-client-queries")
            .unwrap()
            < units
                .iter()
                .position(|u| u.id == "ldap-directory-changes")
                .unwrap()
    );
}
#[test]
fn taught_methods_and_equivalent_supported_solutions_pass_behavioral_cases() {
    for (i, id) in IDS.into_iter().enumerate() {
        let s = catalog().iter().find(|s| s.id == id).unwrap();
        let d = w::source(s).unwrap();
        let mut a = fresh(s);
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        assert!(progress::current_solved(&a, s));
        let alternate = if i == 0 {
            let mut chunks = d.initial.reference.split("\n\n");
            let version = chunks.next().unwrap();
            let mut entries: Vec<_> = chunks
                .map(|s| s.replacen("\nobjectClass:", "\nchangetype: add\nobjectClass:", 1))
                .collect();
            entries.reverse();
            format!("{version}\n\n{}", entries.join("\n\n"))
        } else {
            d.initial
                .reference
                .replace("replace: mail\nmail:", "replace: mail\n-\nadd: mail\nmail:")
                .replace(
                    "replace: description\ndescription:",
                    "replace: description\n-\nadd: description\ndescription:",
                )
        };
        assert_ne!(alternate, d.initial.reference);
        let State::LdapChanges { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.edit(alternate).unwrap();
        let reports = simulation.assignment.check(&d.initial).unwrap();
        assert!(progress::current_solved(&a, s), "{id}: {reports:?}");
    }
}
#[test]
fn malformed_unsupported_partial_destructive_and_hardcoded_wrong_work_fails() {
    for (i, id) in IDS.into_iter().enumerate() {
        let s = catalog().iter().find(|s| s.id == id).unwrap();
        let d = w::source(s).unwrap();
        for wrong in [
            "",
            "version: 1\n\ndn: ../not-a-path\nchangetype: delete\n",
            "version: 1\n\ndn: uid=operator,ou=People,dc=cedar,dc=test\nchangetype: modify\nreplace: userPassword\nuserPassword: no\n-\n",
            &d.initial.starter,
        ] {
            assert!(
                d.initial.reports(wrong).unwrap().iter().any(|r| !r.passed),
                "{id}"
            );
        }
        let partial = if i == 0 {
            let blocks: Vec<_> = d.initial.reference.split("\n\n").collect();
            format!("{}\n\n{}", blocks[0], blocks[1])
        } else {
            d.initial
                .reference
                .split("\n\ndn: uid=retired")
                .next()
                .unwrap()
                .into()
        };
        let reports = d.initial.reports(&partial).unwrap();
        assert!(reports.iter().any(|r| !r.passed));
        assert!(reports.iter().any(
            |r| r.feedback.contains("Missing entry") || r.feedback.contains("Unexpected entry")
        ));
        let destructive = format!(
            "{}\n\ndn: uid=operator,ou=People,dc=cedar,dc=test\nchangetype: delete\n",
            d.initial.reference.trim_end()
        );
        assert!(
            d.initial
                .reports(&destructive)
                .unwrap()
                .iter()
                .all(|r| !r.passed)
        );
        let wrong = d
            .initial
            .reference
            .replace("@cedar.test", "@incorrect.test");
        assert!(d.initial.reports(&wrong).unwrap().iter().all(|r| !r.passed));
    }
    let d = w::source(catalog().iter().find(|s| s.id == IDS[1]).unwrap()).unwrap();
    let wrong = d
        .initial
        .reference
        .replace("replace: mail", "delete: mail\n-\nadd: mail");
    let r = d.initial.reports(&wrong).unwrap();
    assert_eq!(r.iter().filter(|r| r.passed).count(), 2);
    assert!(r[2].feedback.contains("NoSuchAttribute"));
    let wrong = d.initial.reference.replace("replace: mail", "add: mail");
    let reports = d.initial.reports(&wrong).unwrap();
    assert_eq!(reports.iter().filter(|r| r.passed).count(), 1);
    assert!(!reports[0].passed && !reports[1].passed && reports[2].passed);
}
#[test]
fn separate_practice_run_is_ungraded_edits_invalidate_and_reset_is_scoped() {
    for id in IDS {
        let s = catalog().iter().find(|s| s.id == id).unwrap();
        let d = w::source(s).unwrap();
        let mut a = fresh(s);
        let State::LdapChanges { simulation } = &mut a.state else {
            panic!()
        };
        let original = simulation.assignment.clone();
        simulation.fill_example(d).unwrap();
        simulation.practice.run(&d.example).unwrap();
        assert!(!simulation.practice.checked);
        simulation.practice.check(&d.example).unwrap();
        assert_eq!(simulation.assignment, original);
        assert!(!progress::current_solved(&a, s));
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        assert!(progress::current_solved(&a, s));
        let State::LdapChanges { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.run(&d.initial).unwrap();
        simulation
            .assignment
            .edit("version: 1\n\nunfinished".into())
            .unwrap();
        assert!(!simulation.assignment.checked);
        assert!(simulation.assignment.output.is_none());
        assert!(!progress::current_solved(&a, s));
        let State::LdapChanges { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.draft = d.initial.reference.clone();
        simulation.assignment.checked = true;
        simulation.assignment.checked_draft = "stale".into();
        assert!(!progress::current_solved(&a, s));
        let State::LdapChanges { simulation } = &mut a.state else {
            panic!()
        };
        let practice = simulation.practice.clone();
        simulation.reset_assignment(d).unwrap();
        assert_eq!(simulation.assignment, w::Scope::new(&d.initial).unwrap());
        assert_eq!(simulation.practice, practice);
    }
}
#[test]
fn source_validation_rejects_unreachable_outcomes_solved_starters_and_bad_grants() {
    let s = catalog().iter().find(|s| s.id == IDS[0]).unwrap();
    let d = w::source(s).unwrap();
    let mut bad = d.clone();
    bad.initial.starter = bad.initial.reference.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.grant.add = false;
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.reference = bad
        .initial
        .reference
        .replace("sn: Chen", "description: Chen");
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.cases[1].id = bad.initial.cases[0].id.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.example = bad.initial.clone();
    assert!(bad.validate().is_err());
    let mut a = fresh(s);
    a.scenario_version += 1;
    assert!(w::context(&a, s).is_err());
}
#[test]
fn versioned_save_backup_restore_preserve_old_work_and_two_new_drafts() {
    let c = catalog();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for id in IDS {
        let s = c.iter().find(|s| s.id == id).unwrap();
        let mut a = fresh(s);
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        let State::LdapChanges { simulation } = &mut a.state else {
            panic!()
        };
        simulation
            .practice
            .edit("version: 1\n\n# unfinished practice\n".into())
            .unwrap();
        p.favorites.insert(id.into());
        p.attempts.insert(id.into(), a);
    }
    let old = c
        .iter()
        .find(|s| s.id == "lpic2-ldap-contact-query")
        .unwrap();
    let mut old_a = Attempt::new(old, 2);
    workspace::prepare(&mut old_a);
    p.attempts.insert(old.id.clone(), old_a.clone());
    let data = serde_json::to_string(&p).unwrap();
    let q: Progress = serde_json::from_str(&data).unwrap();
    assert_eq!(
        serde_json::to_value(&q).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress.json");
    progress::save_with_catalog(&path, &p, c).unwrap();
    let loaded = progress::load(&path, c).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&old.id]).unwrap(),
        serde_json::to_value(old_a).unwrap()
    );
    let export = directory.path().join("backup.json");
    backup::export(&export, &p, c).unwrap();
    let cancel = AtomicBool::new(false);
    let count = AtomicUsize::new(0);
    let inspected = backup::inspect(&export, c, &cancel, &count).unwrap();
    let restored = backup::restore(&path, &inspected, c).unwrap();
    assert!(restored.safety_copy.is_some());
    let loaded = progress::load(&path, c).unwrap();
    assert_eq!(
        serde_json::to_value(loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
}

#[test]
fn bounded_resume_rejects_unknown_foreign_and_oversized_data_without_trusting_checked_flags() {
    let s = catalog().iter().find(|s| s.id == IDS[0]).unwrap();
    let d = w::source(s).unwrap();
    let mut session = w::Session::new(d).unwrap();
    for scope in [&mut session.assignment, &mut session.practice] {
        scope.draft = "\"".repeat(w::MAX_DRAFT);
        scope.checked_draft = scope.draft.clone();
        scope.checked = true;
    }
    let raw = serde_json::to_string(&session).unwrap();
    assert!(raw.len() < w::MAX_SESSION);
    let restored = w::Session::from_json(d, &raw).unwrap();
    assert_eq!(restored, session);
    assert!(restored.checks(d).iter().any(|r| !r.0));
    let mut foreign = session.clone();
    foreign.origin = "another-task".into();
    assert!(w::Session::from_json(d, &serde_json::to_string(&foreign).unwrap()).is_err());
    let mut unknown: serde_json::Value = serde_json::from_str(&raw).unwrap();
    unknown["command"] = "not supported".into();
    assert!(w::Session::from_json(d, &unknown.to_string()).is_err());
    assert!(w::Session::from_json(d, &"a".repeat(w::MAX_SESSION + 1)).is_err());
    let before = session.clone();
    assert!(
        session
            .execute_reference(d, &format!("{}{}", w::PREFIX, "a".repeat(w::MAX_DRAFT + 1)))
            .is_err()
    );
    assert_eq!(session, before);
    assert!(
        session
            .execute_reference(d, "ldapmodify -f /not-a-host-path")
            .is_err()
    );
    assert_eq!(session, before);
    let mut invalid = d.initial.cases[0].initial.clone();
    invalid.entries[0].dn = "a".repeat(1025);
    assert!(w::output(&invalid).is_err());
}
