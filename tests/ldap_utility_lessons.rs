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
use w::utility::Draft;
const IDS: [&str; 2] = ["lpic2-ldap-import-command", "lpic2-ldap-stopped-update"];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::ccna_pilot().unwrap())
}
fn lab(id: &str) -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == id).unwrap()
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    w::context(&a, s).unwrap();
    a
}
#[test]
fn all364_previous_labs_and130_units_remain_exact_with_two_integrated_operational_assignments() {
    let c = catalog();
    assert_eq!(c.len(), 376);
    let previous = pack::accepted364().unwrap();
    assert_eq!(previous.len(), 364);
    assert_eq!(
        serde_json::to_value(&c[..364]).unwrap(),
        serde_json::to_value(&previous).unwrap()
    );
    let units = course::ccna_pilot(c).unwrap();
    assert_eq!(units.len(), 139);
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "ldap-utility-work" && u.id != "ldap-account-retirement" && u.id != "ldap-protected-passwords" && u.id != "ldap-access-policy" && u.id != "ldap-config-deployment" && u.id != "ldap-schema-extension" && u.id != "ldap-schema-consumers" && u.id != "ipv6-interface-identifiers" && u.id != "linux-boot-entry-recovery")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(course::accepted364(&previous).unwrap()).unwrap()
    );
    let unit = units.iter().find(|u| u.id == "ldap-utility-work").unwrap();
    assert_eq!(unit.labs, IDS);
    assert_eq!(unit.prerequisites, ["ldap-directory-changes"]);
    for id in IDS {
        let s = lab(id);
        let d = w::source(s).unwrap();
        assert!(d.initial.utility.is_some() && d.example.utility.is_some());
        assert_ne!(d.initial.reference, d.example.reference);
        assert_ne!(d.initial.cases, d.example.cases);
        assert!(!engine::solved(&s.initial, &s.goal));
        assert_eq!(d.initial.cases.len(), 3);
    }
}
#[test]
fn taught_methods_and_alternate_supported_commands_and_ldif_pass_all_behavioral_cases() {
    for id in IDS {
        let s = lab(id);
        let d = w::source(s).unwrap();
        let mut a = fresh(s);
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        assert!(progress::current_solved(&a, s));
        let mut alternate = Draft::parse(&d.initial.reference).unwrap();
        alternate.command = if id == IDS[0] {
            alternate.command.replacen("ldapadd", "ldapmodify -a", 1)
        } else {
            alternate.command.replacen("ldapmodify -a", "ldapadd", 1)
        };
        if id == IDS[1] {
            alternate.ldif = alternate
                .ldif
                .replace("replace: mail\nmail:", "replace: mail\n-\nadd: mail\nmail:");
        }
        let State::LdapChanges { simulation } = &mut a.state else {
            panic!()
        };
        simulation
            .assignment
            .edit(alternate.encode().unwrap())
            .unwrap();
        let rows = simulation.assignment.check(&d.initial).unwrap();
        assert!(rows.iter().all(|r| r.passed), "{id}: {rows:?}");
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn wrong_profile_input_default_mode_partial_data_and_later_errors_do_not_pass() {
    for id in IDS {
        let s = lab(id);
        let d = w::source(s).unwrap();
        assert!(
            d.initial
                .reports(&d.initial.starter)
                .unwrap()
                .iter()
                .all(|r| !r.passed)
        );
        let correct = Draft::parse(&d.initial.reference).unwrap();
        let mut wrong = correct.clone();
        wrong.credential = "observer".into();
        assert!(
            d.initial
                .reports(&wrong.encode().unwrap())
                .unwrap()
                .iter()
                .all(|r| !r.passed && r.feedback.contains("49"))
        );
        let mut wrong = correct.clone();
        wrong.command = wrong
            .command
            .replace("/work/changes.ldif", "/work/absent.ldif");
        assert!(
            d.initial
                .reports(&wrong.encode().unwrap())
                .unwrap()
                .iter()
                .all(|r| !r.passed)
        );
        let mut wrong = correct.clone();
        wrong.ldif = wrong.ldif.replace("@orchard.test", "@incorrect.test");
        assert!(
            d.initial
                .reports(&wrong.encode().unwrap())
                .unwrap()
                .iter()
                .all(|r| !r.passed)
        );
        let mut partial = correct.clone();
        let blocks: Vec<_> = partial.ldif.split("\n\n").collect();
        partial.ldif = format!("{}\n\n{}", blocks[0], blocks[1]);
        assert!(
            d.initial
                .reports(&partial.encode().unwrap())
                .unwrap()
                .iter()
                .all(|r| !r.passed)
        );
        let mut stopped = correct;
        stopped.ldif.push_str("\nnot a dn\n");
        let r = d.initial.run(0, &stopped.encode().unwrap()).unwrap();
        assert!(!r.applied);
        assert!(r.feedback.contains("Record 3"));
        assert!(
            d.initial
                .reports(&stopped.encode().unwrap())
                .unwrap()
                .iter()
                .all(|r| !r.passed)
        );
    }
    let d = w::source(lab(IDS[1])).unwrap();
    let mut wrong = Draft::parse(&d.initial.reference).unwrap();
    wrong.ldif = wrong
        .ldif
        .replace("replace: mail", "add: mail")
        .replace("replace: description", "add: description");
    let rows = d.initial.reports(&wrong.encode().unwrap()).unwrap();
    assert_eq!(rows.iter().filter(|r| r.passed).count(), 1);
    assert!(rows[2].passed);
}
#[test]
fn examples_do_not_grade_assignments_edits_clear_success_and_reset_only_selected_work() {
    for id in IDS {
        let s = lab(id);
        let d = w::source(s).unwrap();
        let mut a = fresh(s);
        let State::LdapChanges { simulation } = &mut a.state else {
            panic!()
        };
        let initial = simulation.assignment.clone();
        simulation.fill_example(d).unwrap();
        simulation.practice.run(&d.example).unwrap();
        simulation.practice.check(&d.example).unwrap();
        assert_eq!(simulation.assignment, initial);
        assert!(!progress::current_solved(&a, s));
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        assert!(progress::current_solved(&a, s));
        let State::LdapChanges { simulation } = &mut a.state else {
            panic!()
        };
        let practice = simulation.practice.clone();
        let mut broken = Draft::parse(&simulation.assignment.draft).unwrap();
        broken.command = "ldapadd -x".into();
        simulation
            .assignment
            .edit(broken.encode().unwrap())
            .unwrap();
        assert!(!simulation.assignment.checked);
        assert!(!progress::current_solved(&a, s));
        let State::LdapChanges { simulation } = &mut a.state else {
            panic!()
        };
        simulation.reset_assignment(d).unwrap();
        assert_eq!(simulation.assignment.draft, d.initial.starter);
        assert_eq!(simulation.practice, practice);
    }
}
#[test]
fn actual_private_save_backup_restore_keeps_two_utility_drafts_favorites_and_old_work() {
    let c = catalog();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for id in IDS {
        let s = lab(id);
        let d = w::source(s).unwrap();
        let mut a = fresh(s);
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        let State::LdapChanges { simulation } = &mut a.state else {
            panic!()
        };
        simulation.fill_example(d).unwrap();
        simulation.assignment.run(&d.initial).unwrap();
        p.favorites.insert(id.into());
        p.attempts.insert(id.into(), a);
    }
    let old = lab("lpic2-ldap-contact-maintenance");
    let old_a = fresh(old);
    p.attempts.insert(old.id.clone(), old_a.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
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
    let bytes = serde_json::to_string(&loaded).unwrap();
    assert!(!bytes.contains("Fictional-operator-only-61"));
    assert!(!bytes.contains("Fictional-observer-only-62"));
    for id in IDS {
        assert!(progress::current_solved(&loaded.attempts[id], lab(id)));
    }
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, c).unwrap();
    let inspected =
        backup::inspect(&export, c, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert!(
        backup::restore(&path, &inspected, c)
            .unwrap()
            .safety_copy
            .is_some()
    );
    assert_eq!(
        serde_json::to_value(progress::load(&path, c).unwrap()).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
}
