use dlms_learning_lab::{ldap_password_workspace as w, script_teaching::View};
#[path = "support/ldap_directory.rs"]
mod directory;
#[path = "support/ldap_password_task.rs"]
mod fixture;
#[test]
fn defined_worked_methods_reach_all_three_behavioral_states_and_starters_do_not() {
    let d = fixture::definition();
    d.validate().unwrap();
    for t in [&d.initial, &d.example] {
        assert!(t.reports(&t.reference).unwrap().iter().all(|r| r.passed));
        assert!(t.reports(&t.starter).unwrap().iter().all(|r| !r.passed));
        let run = t.run(0, &t.reference).unwrap();
        assert!(run.applied && run.directory_preserved);
        assert_eq!(run.probes.len(), 3);
        assert!(run.probes.iter().all(|p| p.accepted == p.expected));
    }
}
#[test]
fn alternate_option_order_explicit_self_and_equivalent_client_settings_pass() {
    let t = fixture::definition().initial;
    for config in [
        "tls_cacert /work/training-ca.pem\n",
        "TLS_REQCERT hard\nTLS_CACERT /work/training-ca.pem\n",
    ] {
        let mut d = t.reference.clone();
        d.config = config.into();
        d.command = format!(
            "ldappasswd -D '{}' -H {} -A -S -W -x -ZZ -- '{}'",
            t.actor, t.uri, t.actor
        );
        assert!(t.reports(&d).unwrap().iter().all(|r| r.passed));
    }
}
#[test]
fn per_case_partial_results_remain_correct_for_optional_old_and_shared_values() {
    let t = fixture::definition().initial;
    let mut d = t.reference.clone();
    d.command = d.command.replace(" -A", "");
    let rows = t.reports(&d).unwrap();
    assert_eq!(rows.iter().filter(|r| r.passed).count(), 2);
    assert!(!rows[0].passed && rows[1].passed && rows[2].passed);
    assert!(rows[0].feedback.contains("requires"));
    let mut d = t.reference.clone();
    d.old_profile = Some(w::Key("other".into()));
    let rows = t.reports(&d).unwrap();
    assert_eq!(rows.iter().filter(|r| r.passed).count(), 1);
    assert!(rows[2].passed);
    let mut d = t.reference.clone();
    d.bind_profile = "other".into();
    let rows = t.reports(&d).unwrap();
    assert_eq!(rows.iter().filter(|r| r.passed).count(), 1);
    assert!(rows[2].passed);
}
#[test]
fn successful_request_without_rotation_or_for_the_wrong_identity_never_earns_credit() {
    let t = fixture::definition().initial;
    let mut d = t.reference.clone();
    d.replacement_profile = Some(w::Key("current".into()));
    assert!(t.run(0, &d).unwrap().applied);
    let rows = t.reports(&d).unwrap();
    assert!(
        rows.iter()
            .all(|r| !r.passed && r.feedback.contains("Requested replacement"))
    );
    let mut d = t.reference.clone();
    d.command = d.command.replace(&t.actor, &t.cases[0].people[1].dn);
    d.bind_profile = "other".into();
    d.old_profile = Some(w::Key("other".into()));
    assert!(t.run(0, &d).unwrap().applied);
    assert!(t.reports(&d).unwrap().iter().all(|r| !r.passed));
}
#[test]
fn missing_bad_and_unsupported_inputs_produce_useful_feedback_without_false_success() {
    let t = fixture::definition().initial;
    for config in [
        "",
        "TLS_CACERT /etc/passwd",
        "TLS_CACERT /work/missing.pem",
        "TLS_CACERT /work/other-ca.pem",
        "TLS_CACERT /work/training-ca.pem\nTLS_REQCERT never",
    ] {
        let mut d = t.reference.clone();
        d.config = config.into();
        let rows = t.reports(&d).unwrap();
        assert!(rows.iter().all(|r| !r.passed && !r.feedback.is_empty()));
    }
    for command in [
        "ldappasswd -x".into(),
        t.reference.command.replace(" -ZZ", ""),
        t.reference
            .command
            .replace(&t.uri, "ldap://other.example.test"),
        format!("{} -s private-test-value", t.reference.command),
        format!("{} ; echo wrong", t.reference.command),
        t.reference.command.replace("'", ""),
    ] {
        let mut d = t.reference.clone();
        d.command = command;
        if d.command == t.reference.command.replace("'", "") {
            assert!(t.reports(&d).unwrap().iter().all(|r| r.passed));
        } else {
            assert!(t.reports(&d).unwrap().iter().all(|r| !r.passed));
        }
    }
    let mut d = t.reference.clone();
    d.replacement_profile = None;
    assert!(
        t.reports(&d)
            .unwrap()
            .iter()
            .all(|r| !r.passed && r.feedback.contains("-S"))
    );
}
#[test]
fn practice_isolated_and_edits_breaking_success_clear_current_results_and_reset_only_assignment() {
    let d = fixture::definition();
    let mut s = w::Session::new(&d).unwrap();
    let before = s.assignment.clone();
    s.fill_example(&d).unwrap();
    s.practice.run(&d.example).unwrap();
    assert!(
        s.practice
            .check(&d.example)
            .unwrap()
            .iter()
            .all(|r| r.passed)
    );
    assert_eq!(s.assignment, before);
    assert!(s.checks(&d).iter().any(|r| !r.0));
    s.assignment.edit(d.initial.reference.clone()).unwrap();
    s.assignment.run(&d.initial).unwrap();
    assert!(s.assignment.output.is_some());
    assert!(
        s.assignment
            .check(&d.initial)
            .unwrap()
            .iter()
            .all(|r| r.passed)
    );
    assert!(s.checks(&d).iter().all(|r| r.0));
    let practice = s.practice.clone();
    s.view = View::Assignment;
    let mut wrong = s.assignment.draft.clone();
    wrong.bind_profile = "replacement".into();
    s.assignment.edit(wrong).unwrap();
    assert!(!s.assignment.checked && s.assignment.output.is_none());
    assert!(s.checks(&d).iter().any(|r| !r.0));
    s.reset_assignment(&d).unwrap();
    assert_eq!(s.assignment.draft, d.initial.starter);
    assert_eq!(s.practice, practice);
}
#[test]
fn roundtrip_keeps_two_drafts_and_regrades_flags_without_persisting_source_passwords_or_proofs() {
    let d = fixture::definition();
    let mut s = w::Session::new(&d).unwrap();
    s.fill_example(&d).unwrap();
    s.practice.check(&d.example).unwrap();
    s.assignment.edit(d.initial.reference.clone()).unwrap();
    s.assignment.check(&d.initial).unwrap();
    s.assignment.run(&d.initial).unwrap();
    let text = serde_json::to_string(&s).unwrap();
    assert!(
        !text.contains("Fictional")
            && !text.contains("authority")
            && !text.contains("sample_password")
    );
    let restored: w::Session = serde_json::from_str(&text).unwrap();
    restored.bound(&d).unwrap();
    assert_eq!(restored.assignment.draft, s.assignment.draft);
    assert_eq!(restored.practice.draft, s.practice.draft);
    assert!(restored.assignment.output.is_none());
    assert!(restored.checks(&d).iter().all(|r| r.0));
    let mut forged = w::Session::new(&d).unwrap();
    forged.assignment.checked = true;
    forged.assignment.checked_draft = Some(forged.assignment.draft.clone());
    assert!(forged.checks(&d).iter().all(|r| !r.0));
    let mut foreign = restored;
    foreign.origin = "another-task".into();
    assert!(foreign.bound(&d).is_err());
}
#[test]
fn malformed_sources_and_unreachable_or_already_solved_content_are_rejected() {
    let d = fixture::definition();
    for mode in 0..7 {
        let mut bad = d.clone();
        match mode {
            0 => bad.initial.profiles.push(bad.initial.profiles[0].clone()),
            1 => bad.initial.cases[0].values[0].sample_password.clear(),
            2 => bad.initial.ca_inputs[0].path = "/etc/ssl/ca.pem".into(),
            3 => bad.initial.reference.config = "TLS_CACERT /work/missing.pem".into(),
            4 => bad.initial.starter = bad.initial.reference.clone(),
            5 => bad.example = bad.initial.clone(),
            _ => bad.initial.cases[0].people[0].profile = "not-supplied".into(),
        };
        assert!(bad.validate().is_err(), "mode {mode}");
    }
    let mut bad = d.initial.clone();
    bad.cases = vec![bad.cases[0].clone(), bad.cases[0].clone()];
    bad.cases[1].id = "second".into();
    bad.cases[1].label = "Different label only".into();
    assert!(bad.validate().unwrap_err().contains("meaningfully"));
    let mut json = serde_json::to_value(d).unwrap();
    json["initial"]["host_command"] = serde_json::json!("true");
    assert!(serde_json::from_value::<w::Definition>(json).is_err());
}
#[test]
fn oversized_or_foreign_saved_work_is_rejected_and_invalid_edit_preserves_the_previous_draft() {
    let d = fixture::definition();
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.edit(d.initial.reference.clone()).unwrap();
    s.assignment.check(&d.initial).unwrap();
    let before = s.assignment.draft.clone();
    let mut huge = before.clone();
    huge.config = "x".repeat(8193);
    assert!(s.assignment.edit(huge).is_err());
    assert_eq!(s.assignment.draft, before);
    assert!(!s.assignment.checked);
    assert!(s.checks(&d).iter().all(|r| !r.0));
    let mut foreign = s.clone();
    foreign.assignment.draft.bind_profile = "unprovided".into();
    assert!(foreign.bound(&d).is_err());
    let mut foreign = s;
    foreign.assignment.selected_case = 7;
    assert!(foreign.bound(&d).is_err());
}

#[test]
fn unchanged_modeled_directory_is_not_rejected_for_source_attribute_order_or_case() {
    let mut t = fixture::definition().initial;
    for c in &mut t.cases {
        for entry in &mut c.initial.entries {
            entry.attributes.reverse();
            for attribute in &mut entry.attributes {
                attribute.name = attribute.name.to_ascii_uppercase();
            }
        }
    }
    let run = t.run(0, &t.reference).unwrap();
    assert!(run.applied, "{run:?}");
    assert!(
        run.probes.iter().all(|p| p.accepted == p.expected),
        "{run:?}"
    );
    assert!(
        run.directory_preserved,
        "The password-only operation must preserve its canonical starting directory: {run:?}"
    );
    let rows = t.reports(&t.reference).unwrap();
    assert!(rows.iter().all(|r| r.passed), "{rows:?}");
}

fn integrated_catalog() -> &'static Vec<dlms_learning_lab::pack::Scenario> {
    use dlms_learning_lab::{engine, pack};
    static C: std::sync::OnceLock<Vec<pack::Scenario>> = std::sync::OnceLock::new();
    C.get_or_init(|| {
        let mut catalog = pack::ccna_pilot().unwrap();
        let mut source = catalog
            .iter()
            .find(|s| s.id == "lpic2-ldap-account-retirement")
            .unwrap()
            .clone();
        let d = fixture::definition();
        source.id = d.initial.id.clone();
        source.title = "Synthetic protected password workspace".into();
        source.summary =
            "Development fixture for typed credential grading and progress isolation.".into();
        source.objective = d.initial.instructions.clone();
        source.prerequisites.clear();
        source.hints = vec!["Check protection and the separate fictional prompt roles.".into()];
        source.explanation =
            "Verify fresh replacement and previous credentials after the simulated change.".into();
        source.solution = vec![format!(
            "{}{}",
            w::PREFIX,
            serde_json::to_string(&d.initial.reference).unwrap()
        )];
        source.initial = engine::State::LdapPassword {
            simulation: Box::new(w::Session::new(&d).unwrap()),
        };
        source.goal = engine::Goal::LdapPassword {
            definition: Box::new(d),
        };
        catalog.push(source);
        catalog
    })
}
fn integrated_source() -> &'static dlms_learning_lab::pack::Scenario {
    integrated_catalog()
        .iter()
        .find(|s| s.id == "password-assignment")
        .unwrap()
}
#[test]
fn engine_context_preparation_and_result_grading_use_the_source_bound_password_model() {
    use dlms_learning_lab::{
        engine,
        progress::{self, Attempt},
        workspace,
    };
    let s = integrated_source();
    w::source(s).unwrap();
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    w::context(&a, s).unwrap();
    assert!(!progress::current_solved(&a, s));
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    assert!(progress::current_solved(&a, s));
    let engine::State::LdapPassword { simulation } = &mut a.state else {
        panic!()
    };
    let mut broken = simulation.assignment.draft.clone();
    broken.bind_profile = "replacement".into();
    simulation.assignment.edit(broken).unwrap();
    assert!(!progress::current_solved(&a, s));
    let d = fixture::definition();
    let engine::State::LdapPassword { simulation } = &mut a.state else {
        panic!()
    };
    simulation.reset_assignment(&d).unwrap();
    assert_eq!(simulation.assignment.draft, d.initial.starter);
}
#[test]
fn real_private_progress_save_and_backup_restore_keep_drafts_and_unrelated_attempt_bytes() {
    use dlms_learning_lab::{
        backup, engine,
        progress::{self, Attempt, Progress},
        workspace,
    };
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    let catalog = integrated_catalog();
    let s = integrated_source();
    let d = w::source(s).unwrap();
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    let engine::State::LdapPassword { simulation } = &mut a.state else {
        panic!()
    };
    simulation.fill_example(d).unwrap();
    simulation.practice.check(&d.example).unwrap();
    let old = catalog
        .iter()
        .find(|s| s.id == "lpic2-ldap-account-retirement")
        .unwrap();
    let mut old_a = Attempt::new(old, 1);
    workspace::prepare(&mut old_a);
    let old_bytes = serde_json::to_value(&old_a).unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(old.id.clone(), old_a);
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    assert!(progress::save(&path, &p).is_err());
    progress::save_with_catalog(&path, &p, catalog).unwrap();
    let loaded = progress::load(&path, catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&old.id]).unwrap(),
        old_bytes
    );
    assert!(progress::current_solved(&loaded.attempts[&s.id], s));
    assert!(
        !serde_json::to_string(&loaded)
            .unwrap()
            .contains("Fictional-")
    );
    let exported = dir.path().join("backup.json");
    backup::export(&exported, &p, catalog).unwrap();
    let inspected = backup::inspect(
        &exported,
        catalog,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    assert!(
        backup::restore(&path, &inspected, catalog)
            .unwrap()
            .safety_copy
            .is_some()
    );
    assert_eq!(
        serde_json::to_value(progress::load(&path, catalog).unwrap()).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
}
#[test]
fn reference_execution_rejects_malformed_or_oversized_input_and_foreign_context_without_credit() {
    use dlms_learning_lab::{
        engine,
        progress::{self, Attempt},
        workspace,
    };
    let s = integrated_source();
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    for raw in [
        "not a shell command".into(),
        format!("{}{{bad json", w::PREFIX),
        format!("{}{}", w::PREFIX, "x".repeat(16385)),
    ] {
        assert!(engine::execute_for_goal(&mut a.state, &s.goal, &raw).is_err());
        assert!(!progress::current_solved(&a, s));
    }
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    let engine::State::LdapPassword { simulation } = &mut a.state else {
        panic!()
    };
    simulation.origin = "foreign".into();
    assert!(w::context(&a, s).is_err());
    assert!(!progress::current_solved(&a, s));
}
