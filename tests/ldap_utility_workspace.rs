use dlms_learning_lab::{ldap_change_workspace as w, pack};
use std::sync::OnceLock;
use w::utility::{Credential, Draft, Utility};
fn legacy() -> &'static w::Definition {
    static D: OnceLock<w::Definition> = OnceLock::new();
    D.get_or_init(|| {
        let scenarios = pack::ccna_pilot().unwrap();
        w::source(
            scenarios
                .iter()
                .find(|s| s.id == "lpic2-ldap-person-import")
                .unwrap(),
        )
        .unwrap()
        .clone()
    })
}
fn adapt(task: &mut w::Task) {
    let primary = task.grant.principal.clone();
    let observer = primary.replace("uid=operator,", "uid=sam,");
    let utility = Utility {
        input_kind: Default::default(),
        uri: "ldap://directory.example.test".into(),
        input_file: "/work/people.ldif".into(),
        credentials: vec![
            Credential {
                id: "operator".into(),
                label: "Supplied import operator".into(),
                principal: primary.clone(),
                sample_password: "Fictional-operator-only".into(),
            },
            Credential {
                id: "observer".into(),
                label: "Supplied read-only observer".into(),
                principal: observer.clone(),
                sample_password: "Fictional-observer-only".into(),
            },
        ],
    };
    let command = |who: &str, verb: &str| {
        format!(
            "{verb} -x -W -H {} -D '{who}' -f {}",
            utility.uri, utility.input_file
        )
    };
    task.starter = Draft {
        credential: "observer".into(),
        command: command(&observer, "ldapadd"),
        ldif: task.reference.clone(),
    }
    .encode()
    .unwrap();
    task.reference = Draft {
        credential: "operator".into(),
        command: command(&primary, "ldapadd"),
        ldif: task.reference.clone(),
    }
    .encode()
    .unwrap();
    task.utility = Some(utility);
}
fn definition() -> w::Definition {
    let mut d = legacy().clone();
    adapt(&mut d.initial);
    adapt(&mut d.example);
    d
}
fn all(s: &w::Session, d: &w::Definition) -> bool {
    s.checks(d).iter().all(|r| r.0)
}
#[test]
fn legacy_definitions_serialize_identically_and_do_not_gain_utility_requirements() {
    let d = legacy();
    d.validate().unwrap();
    assert!(d.initial.utility.is_none());
    assert!(
        !serde_json::to_value(d).unwrap()["initial"]
            .as_object()
            .unwrap()
            .contains_key("utility")
    );
    assert!(
        d.initial
            .reports(&d.initial.reference)
            .unwrap()
            .iter()
            .all(|r| r.passed)
    );
}
#[test]
fn the_utility_adapter_grades_outcomes_in_every_case_and_rejects_valid_but_unauthorized_work() {
    let d = definition();
    d.validate().unwrap();
    let passing = d.initial.reports(&d.initial.reference).unwrap();
    assert_eq!(passing.len(), 3);
    assert!(passing.iter().all(|r| r.passed));
    let wrong = d.initial.reports(&d.initial.starter).unwrap();
    assert!(wrong.iter().all(|r| !r.passed && r.feedback.contains("50")));
    let mut mixed = Draft::parse(&d.initial.reference).unwrap();
    mixed.credential = "observer".into();
    assert!(
        d.initial
            .reports(&mixed.encode().unwrap())
            .unwrap()
            .iter()
            .all(|r| !r.passed && r.feedback.contains("49"))
    );
}
#[test]
fn equivalent_supported_commands_and_input_methods_pass_without_exact_string_matching() {
    let d = definition();
    for remove_file in [false, true] {
        let mut draft = Draft::parse(&d.initial.reference).unwrap();
        draft.command = draft.command.replacen("ldapadd", "ldapmodify -a", 1);
        if remove_file {
            draft.command = draft.command.replace(" -f /work/people.ldif", "");
        }
        assert!(
            d.initial
                .reports(&draft.encode().unwrap())
                .unwrap()
                .iter()
                .all(|r| r.passed)
        );
    }
    let mut draft = Draft::parse(&d.initial.reference).unwrap();
    draft.command = draft.command.replacen("ldapadd", "ldapmodify", 1);
    draft.ldif = draft
        .ldif
        .replace("\nobjectClass: top", "\nchangetype: add\nobjectClass: top");
    assert!(
        d.initial
            .reports(&draft.encode().unwrap())
            .unwrap()
            .iter()
            .all(|r| r.passed)
    );
}
#[test]
fn practice_drafts_and_results_do_not_change_or_complete_assignment_work() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let initial = s.assignment.clone();
    s.fill_example(&d).unwrap();
    s.practice.run(&d.example).unwrap();
    assert!(!s.practice.checked);
    assert_eq!(s.assignment, initial);
    assert!(!all(&s, &d));
    s.practice.check(&d.example).unwrap();
    assert_eq!(s.assignment, initial);
    assert!(!all(&s, &d));
    s.assignment.edit(d.initial.reference.clone()).unwrap();
    s.assignment.check(&d.initial).unwrap();
    assert!(all(&s, &d));
    let mut changed = Draft::parse(&s.assignment.draft).unwrap();
    changed.command = "ldapdelete -x".into();
    s.assignment.edit(changed.encode().unwrap()).unwrap();
    assert!(!s.assignment.checked);
    assert!(!all(&s, &d));
    assert!(s.practice.checked);
    s.reset_assignment(&d).unwrap();
    assert_eq!(s.assignment.draft, d.initial.starter);
    assert!(s.practice.checked);
}
#[test]
fn save_resume_preserves_command_file_and_profile_but_not_passwords_or_transient_output() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.edit(d.initial.reference.clone()).unwrap();
    s.assignment.check(&d.initial).unwrap();
    s.assignment.run(&d.initial).unwrap();
    s.fill_example(&d).unwrap();
    let raw = serde_json::to_string(&s).unwrap();
    assert!(!raw.contains("Fictional-operator-only"));
    assert!(!raw.contains("Fictional-observer-only"));
    assert!(!raw.contains("Exit status:"));
    let restored = w::Session::from_json(&d, &raw).unwrap();
    assert_eq!(restored.assignment.draft, s.assignment.draft);
    assert_eq!(restored.practice.draft, s.practice.draft);
    assert_eq!(
        restored.assignment.checked_draft,
        s.assignment.checked_draft
    );
    assert!(restored.assignment.checked);
    assert!(restored.assignment.output.is_none());
    assert!(all(&restored, &d));
}
#[test]
fn forged_flags_unknown_profiles_foreign_context_and_invalid_drafts_do_not_create_credit() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.checked = true;
    s.assignment.checked_draft = s.assignment.draft.clone();
    assert!(!all(&s, &d));
    let mut value = serde_json::to_value(&s).unwrap();
    value["assignment"]["draft"] = value["assignment"]["draft"]
        .as_str()
        .unwrap()
        .replacen(
            "sample-credential: observer",
            "sample-credential: unknown",
            1,
        )
        .into();
    assert!(w::Session::from_json(&d, &value.to_string()).is_err());
    let mut value = serde_json::to_value(&s).unwrap();
    value["origin"] = "another-lab".into();
    assert!(w::Session::from_json(&d, &value.to_string()).is_err());
    for raw in [
        "",
        "operator\nldapadd\n\nversion: 1",
        "sample-credential: operator\nldapadd\nnot-separated",
    ] {
        assert!(Draft::parse(raw).is_err());
    }
}
#[test]
fn source_profiles_and_virtual_paths_are_validated_and_unreachable_worked_methods_fail() {
    let d = definition();
    let mut bad = d.clone();
    bad.initial.utility.as_mut().unwrap().input_file = "/etc/passwd".into();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.utility.as_mut().unwrap().credentials[1].id = "operator".into();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.utility.as_mut().unwrap().credentials[0]
        .sample_password
        .clear();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.utility.as_mut().unwrap().credentials.remove(0);
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    let mut draft = Draft::parse(&bad.initial.reference).unwrap();
    draft.command = "ldapadd -x".into();
    bad.initial.reference = draft.encode().unwrap();
    assert!(bad.validate().is_err());
    let draft = Draft {
        credential: "operator".into(),
        command: "x".repeat(4097),
        ldif: "".into(),
    };
    assert!(draft.encode().is_err());
    let draft = Draft {
        credential: "operator".into(),
        command: "ldapadd\necho no".into(),
        ldif: "".into(),
    };
    assert!(draft.encode().is_err());
    let draft = Draft {
        credential: "operator".into(),
        command: "ldapadd".into(),
        ldif: "x".repeat(w::MAX_DRAFT),
    };
    assert!(draft.encode().is_err());
}
#[test]
fn partial_state_is_visible_but_a_later_syntax_error_or_wrong_data_never_passes() {
    let d = definition();
    let mut draft = Draft::parse(&d.initial.reference).unwrap();
    draft.ldif.push_str("\nnot a dn\n");
    let run = d.initial.run(0, &draft.encode().unwrap()).unwrap();
    assert!(!run.applied);
    assert!(run.feedback.contains("Record 3"));
    assert!(run.actual.contains("uid=nova"));
    assert!(run.actual.contains("uid=theo"));
    assert!(
        d.initial
            .reports(&draft.encode().unwrap())
            .unwrap()
            .iter()
            .all(|r| !r.passed)
    );
    let mut wrong = Draft::parse(&d.initial.reference).unwrap();
    wrong.ldif = wrong.ldif.replace("@cedar.test", "@incorrect.test");
    assert!(
        d.initial
            .reports(&wrong.encode().unwrap())
            .unwrap()
            .iter()
            .all(|r| !r.passed)
    );
}
