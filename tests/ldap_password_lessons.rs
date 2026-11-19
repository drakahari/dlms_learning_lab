use dlms_learning_lab::{
    course,
    engine::{self, State},
    ldap_password_workspace as w, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::OnceLock;
const ID: &str = "lpic2-ldap-protected-password";
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::ccna_pilot().unwrap())
}
fn lab() -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == ID).unwrap()
}
fn fresh() -> Attempt {
    let mut a = Attempt::new(lab(), 1);
    workspace::prepare(&mut a);
    w::context(&a, lab()).unwrap();
    a
}
#[test]
fn all367_prior_scenarios_and132_units_are_exact_and_new_unit_follows_retirement() {
    let c = catalog();
    assert_eq!(c.len(), 376);
    let old = pack::accepted367().unwrap();
    assert_eq!(old.len(), 367);
    assert_eq!(
        serde_json::to_value(&c[..367]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let units = course::ccna_pilot(c).unwrap();
    assert_eq!(units.len(), 139);
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "ldap-protected-passwords" && u.id != "ldap-access-policy" && u.id != "ldap-config-deployment" && u.id != "ldap-schema-extension" && u.id != "ldap-schema-consumers" && u.id != "ipv6-interface-identifiers" && u.id != "linux-boot-entry-recovery")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(course::accepted367(&old).unwrap()).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "ldap-protected-passwords")
        .unwrap();
    assert_eq!(units[at - 1].id, "ldap-account-retirement");
    assert_eq!(units[at].labs, [ID]);
    assert_eq!(units[at].prerequisites, ["ldap-account-retirement"]);
    assert_eq!(lab().prerequisites, ["lpic2-ldap-account-retirement"]);
}
#[test]
fn taught_and_supported_alternate_methods_grade_actual_fresh_logins_across_all_cases() {
    let s = lab();
    let d = w::source(s).unwrap();
    assert_eq!(d.initial.cases.len(), 3);
    assert_ne!(d.initial.reference, d.example.reference);
    for task in [&d.initial, &d.example] {
        assert!(
            task.reports(&task.reference)
                .unwrap()
                .iter()
                .all(|r| r.passed)
        );
        assert!(
            task.reports(&task.starter)
                .unwrap()
                .iter()
                .all(|r| !r.passed)
        );
    }
    let mut a = fresh();
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    assert!(progress::current_solved(&a, s));
    for mode in 0..3 {
        let mut alternate = d.initial.reference.clone();
        match mode {
            0 => alternate.command = alternate.command.replace("-ZZ", "-Z"),
            1 => {
                alternate.command = format!(
                    "ldappasswd -D '{}' -H {} -A -S -W -x -ZZ -- '{}'",
                    d.initial.actor, d.initial.uri, d.initial.actor
                );
                alternate.config = alternate.config.replace("TLS_REQCERT demand\n", "");
            }
            _ => {
                alternate.config = alternate
                    .config
                    .replace("TLS_REQCERT demand", "tls_reqcert hard")
            }
        };
        assert!(
            d.initial
                .reports(&alternate)
                .unwrap()
                .iter()
                .all(|r| r.passed)
        );
    }
}
#[test]
fn partial_wrong_prompt_and_successful_nonrotation_are_never_global_success() {
    let t = &w::source(lab()).unwrap().initial;
    let mut partial = t.reference.clone();
    partial.command = partial.command.replace(" -A", "");
    let rows = t.reports(&partial).unwrap();
    assert_eq!(rows.iter().filter(|r| r.passed).count(), 2);
    assert!(!rows[0].passed && rows[1].passed && rows[2].passed);
    let mut partial = t.reference.clone();
    partial.bind_profile = "other".into();
    let rows = t.reports(&partial).unwrap();
    assert_eq!(rows.iter().filter(|r| r.passed).count(), 1);
    assert!(rows[2].passed);
    let mut nonrotation = t.reference.clone();
    nonrotation.replacement_profile = Some(w::Key("current".into()));
    assert!(t.run(0, &nonrotation).unwrap().applied);
    assert!(t.reports(&nonrotation).unwrap().iter().all(|r| !r.passed));
    for config in [
        "TLS_CACERT /etc/passwd",
        "TLS_CACERT /work/missing.pem",
        "TLS_CACERT /work/other-ca.pem",
        "TLS_CACERT /work/accounts-ca.pem\nTLS_REQCERT never",
    ] {
        let mut wrong = t.reference.clone();
        wrong.config = config.into();
        assert!(t.reports(&wrong).unwrap().iter().all(|r| !r.passed));
    }
}
#[test]
fn practice_credit_isolated_edits_clear_success_and_selected_reset_keeps_practice() {
    let s = lab();
    let d = w::source(s).unwrap();
    let mut a = fresh();
    let State::LdapPassword { simulation } = &mut a.state else {
        panic!()
    };
    let before = simulation.assignment.clone();
    simulation.fill_example(d).unwrap();
    simulation.practice.check(&d.example).unwrap();
    assert_eq!(simulation.assignment, before);
    assert!(!progress::current_solved(&a, s));
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    assert!(progress::current_solved(&a, s));
    let State::LdapPassword { simulation } = &mut a.state else {
        panic!()
    };
    let practice = simulation.practice.clone();
    let mut broken = simulation.assignment.draft.clone();
    broken.command = "ldappasswd -x".into();
    simulation.assignment.edit(broken).unwrap();
    assert!(!simulation.assignment.checked && simulation.assignment.output.is_none());
    assert!(!progress::current_solved(&a, s));
    let State::LdapPassword { simulation } = &mut a.state else {
        panic!()
    };
    simulation.reset_assignment(d).unwrap();
    assert_eq!(simulation.assignment.draft, d.initial.starter);
    assert_eq!(simulation.practice, practice);
}
#[test]
fn real_private_save_resume_and_backup_restore_preserve_work_without_source_passwords() {
    use dlms_learning_lab::backup;
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    let s = lab();
    let mut a = fresh();
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    let State::LdapPassword { simulation } = &mut a.state else {
        panic!()
    };
    simulation.fill_example(w::source(s).unwrap()).unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.favorites.insert(ID.into());
    p.attempts.insert(ID.into(), a);
    let old = catalog()
        .iter()
        .find(|s| s.id == "lpic2-ldap-account-retirement")
        .unwrap();
    let mut old_a = Attempt::new(old, 1);
    workspace::prepare(&mut old_a);
    let old_bytes = serde_json::to_value(&old_a).unwrap();
    p.attempts.insert(old.id.clone(), old_a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let loaded = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&old.id]).unwrap(),
        old_bytes
    );
    assert!(progress::current_solved(&loaded.attempts[ID], s));
    let raw = serde_json::to_string(&loaded).unwrap();
    assert!(!raw.contains("Fictional-") && !raw.contains("sample_password"));
    let export = dir.path().join("backup.json");
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
        serde_json::to_value(&p).unwrap()
    );
}
