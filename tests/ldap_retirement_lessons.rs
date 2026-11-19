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
const ID: &str = "lpic2-ldap-account-retirement";
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::ccna_pilot().unwrap())
}
fn lab() -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == ID).unwrap()
}
fn fresh() -> Attempt {
    let s = lab();
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    w::context(&a, s).unwrap();
    a
}
#[test]
fn all_previous366_scenarios_and131_units_remain_exact_and_retirement_follows_utility_work() {
    let old = pack::accepted366().unwrap();
    let c = catalog();
    assert_eq!(old.len(), 366);
    assert_eq!(c.len(), 376);
    assert_eq!(
        serde_json::to_value(&c[..366]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let units = course::ccna_pilot(c).unwrap();
    assert_eq!(units.len(), 139);
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "ldap-account-retirement" && u.id != "ldap-protected-passwords" && u.id != "ldap-access-policy" && u.id != "ldap-config-deployment" && u.id != "ldap-schema-extension" && u.id != "ldap-schema-consumers" && u.id != "ipv6-interface-identifiers" && u.id != "linux-boot-entry-recovery")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(course::accepted366(&old).unwrap()).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "ldap-account-retirement")
        .unwrap();
    assert_eq!(units[at - 1].id, "ldap-utility-work");
    assert_eq!(units[at].prerequisites, ["ldap-utility-work"]);
    assert_eq!(units[at].labs, [ID]);
    assert_eq!(lab().prerequisites, ["lpic2-ldap-stopped-update"]);
    let d = w::source(lab()).unwrap();
    assert!(d.initial.utility.as_ref().unwrap().is_dn_list());
    assert_ne!(d.initial.cases, d.example.cases);
    assert_ne!(d.initial.reference, d.example.reference);
}
#[test]
fn taught_list_reordered_names_quoted_arguments_and_editor_input_pass_all_three_samples() {
    let d = w::source(lab()).unwrap();
    let correct = Draft::parse(&d.initial.reference).unwrap();
    let mut reverse = correct.clone();
    reverse.ldif = correct.ldif.lines().rev().collect::<Vec<_>>().join("\n") + "\n";
    let mut positional = correct.clone();
    positional.command = correct.command.replace(
        "-f /work/retire.dn",
        &correct
            .ldif
            .lines()
            .map(|n| format!("'{n}'"))
            .collect::<Vec<_>>()
            .join(" "),
    );
    positional.ldif = "This unused buffer is not interpreted as input.".into();
    let mut editor = correct.clone();
    editor.command = correct.command.replace(" -f /work/retire.dn", "");
    for draft in [correct, reverse, positional, editor] {
        let rows = d.initial.reports(&draft.encode().unwrap()).unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|r| r.passed), "{rows:?}");
    }
    let mut a = fresh();
    engine::execute_for_goal(&mut a.state, &lab().goal, &lab().solution[0]).unwrap();
    assert!(progress::current_solved(&a, lab()));
}
#[test]
fn wrong_branch_extra_deletions_incomplete_lists_and_later_request_errors_cannot_pass() {
    let d = w::source(lab()).unwrap();
    let right = Draft::parse(&d.initial.reference).unwrap();
    let starter = d.initial.reports(&d.initial.starter).unwrap();
    assert!(
        starter
            .iter()
            .all(|r| !r.passed && r.feedback.contains("50"))
    );
    let mut extra = right.clone();
    extra
        .ldif
        .push_str("uid=avery,ou=Archive,dc=retirement,dc=test\n");
    let actual = d.initial.run(0, &extra.encode().unwrap()).unwrap();
    assert!(actual.applied, "the requests themselves should succeed");
    let rows = d.initial.reports(&extra.encode().unwrap()).unwrap();
    assert!(rows.iter().all(|r| !r.passed
        && r.feedback.contains("must remain")
        && r.feedback.contains("uid=avery")));
    let mut partial = right.clone();
    partial.ldif = right.ldif.lines().next().unwrap().to_string() + "\n";
    let mut duplicate = right.clone();
    duplicate.ldif.push_str(right.ldif.lines().next().unwrap());
    let mut missing = right.clone();
    missing
        .ldif
        .push_str("uid=absent,ou=Archive,dc=retirement,dc=test\n");
    let mut empty = right.clone();
    empty.ldif.clear();
    let mut wrong_profile = right.clone();
    wrong_profile.credential = "observer".into();
    for wrong in [partial, duplicate, missing, empty, wrong_profile] {
        assert!(
            d.initial
                .reports(&wrong.encode().unwrap())
                .unwrap()
                .iter()
                .all(|r| !r.passed)
        );
    }
    let mut malformed = right;
    malformed.ldif.push_str("dn: not a DN\n");
    let run = d.initial.run(0, &malformed.encode().unwrap()).unwrap();
    assert!(!run.applied);
    assert!(run.feedback.contains("Record 3"));
    assert!(!run.actual.contains("dn: uid=casey,ou=Archive"));
}
#[test]
fn unsupported_formats_do_not_pretend_to_be_valid_or_touch_any_external_resource() {
    let d = w::source(lab()).unwrap();
    let right = Draft::parse(&d.initial.reference).unwrap();
    for raw in [
        format!("version: 1\n{}", right.ldif),
        format!("\n{}", right.ldif),
        right.ldif.replace('\n', "\r\n"),
        format!("# names\n{}", right.ldif),
    ] {
        let mut wrong = right.clone();
        wrong.ldif = raw;
        let rows = d.initial.reports(&wrong.encode().unwrap()).unwrap();
        assert!(rows.iter().all(|r| !r.passed));
    }
    for argument in [
        "-r",
        "-c",
        "-f /etc/passwd",
        "-f /work/../retire.dn",
        "; echo wrong",
        "$(echo wrong)",
    ] {
        let mut wrong = right.clone();
        wrong.command = format!("{} {argument}", right.command);
        assert!(
            d.initial
                .reports(&wrong.encode().unwrap())
                .unwrap()
                .iter()
                .all(|r| !r.passed)
        );
    }
}
#[test]
fn practice_is_separate_edits_invalidate_success_and_reset_only_assignment() {
    let s = lab();
    let d = w::source(s).unwrap();
    let mut a = fresh();
    let State::LdapChanges { simulation } = &mut a.state else {
        panic!()
    };
    let before = simulation.assignment.clone();
    simulation.fill_example(d).unwrap();
    simulation.practice.run(&d.example).unwrap();
    assert!(
        simulation
            .practice
            .check(&d.example)
            .unwrap()
            .iter()
            .all(|r| r.passed)
    );
    assert_eq!(simulation.assignment, before);
    assert!(!progress::current_solved(&a, s));
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    assert!(progress::current_solved(&a, s));
    let State::LdapChanges { simulation } = &mut a.state else {
        panic!()
    };
    let practice = simulation.practice.clone();
    let mut draft = Draft::parse(&simulation.assignment.draft).unwrap();
    draft
        .ldif
        .push_str("uid=avery,ou=Archive,dc=retirement,dc=test\n");
    simulation.assignment.edit(draft.encode().unwrap()).unwrap();
    assert!(!simulation.assignment.checked);
    assert!(!progress::current_solved(&a, s));
    let State::LdapChanges { simulation } = &mut a.state else {
        panic!()
    };
    simulation.reset_assignment(d).unwrap();
    assert_eq!(simulation.assignment.draft, d.initial.starter);
    assert_eq!(simulation.practice, practice);
}
#[test]
fn real_private_save_and_validated_backup_restore_preserve_both_drafts_and_legacy_work() {
    let s = lab();
    let d = w::source(s).unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = fresh();
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    let State::LdapChanges { simulation } = &mut a.state else {
        panic!()
    };
    simulation.fill_example(d).unwrap();
    simulation.practice.check(&d.example).unwrap();
    p.attempts.insert(ID.into(), a);
    p.favorites.insert(ID.into());
    let old = catalog()
        .iter()
        .find(|s| s.id == "lpic2-ldap-import-command")
        .unwrap();
    let mut old_a = Attempt::new(old, 1);
    workspace::prepare(&mut old_a);
    w::context(&old_a, old).unwrap();
    p.attempts.insert(old.id.clone(), old_a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let loaded = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(progress::current_solved(&loaded.attempts[ID], s));
    assert!(
        !serde_json::to_string(&loaded)
            .unwrap()
            .contains("Fictional-retirement-only-71")
    );
    let export = dir.path().join("export.json");
    backup::export(&export, &p, catalog()).unwrap();
    let inspected = backup::inspect(
        &export,
        catalog(),
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    assert!(
        backup::restore(&path, &inspected, catalog())
            .unwrap()
            .safety_copy
            .is_some()
    );
    assert_eq!(
        serde_json::to_value(progress::load(&path, catalog()).unwrap()).unwrap(),
        serde_json::to_value(p).unwrap()
    );
}
