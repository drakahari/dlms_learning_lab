//! Authored delegation definition only; not registered catalog coverage yet.
use dlms_learning_lab::ldap_access_workspace as w;
use std::sync::OnceLock;
fn definition() -> &'static w::Definition {
    static D: OnceLock<w::Definition> = OnceLock::new();
    D.get_or_init(|| {
        let d: w::Definition =
            serde_json::from_str(include_str!("fixtures/ldap_access_authored.json")).unwrap();
        d.validate().unwrap();
        d
    })
}
#[test]
fn worked_static_rules_meet_all43_independent_change_and_preservation_outcomes_in_each_scope() {
    let d = definition();
    for t in [&d.initial, &d.example] {
        let rows = t.reports(&t.reference).unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|r| r.passed), "{rows:?}");
        assert_eq!(rows.iter().map(|r| r.outcomes.len()).sum::<usize>(), 43);
        assert!(
            rows.iter()
                .flat_map(|r| &r.outcomes)
                .filter(|r| r.allowed)
                .all(|r| r.before != r.after)
        );
        assert!(
            rows.iter()
                .flat_map(|r| &r.outcomes)
                .filter(|r| !r.allowed)
                .all(|r| r.before == r.after)
        );
        assert!(t.reports(&t.starter).unwrap().iter().all(|r| !r.passed));
    }
}
#[test]
fn equivalent_caller_order_stops_scope_alias_and_read_fallback_are_accepted_by_write_behavior() {
    let t = &definition().initial;
    for variant in 0..4 {
        let mut p = t.reference.clone();
        match variant {
            0 => {
                p=p.replace("    by dn.exact=\"uid=desk,ou=Service,dc=delegation,dc=test\" write\n    by self write","    by self write\n    by dn.exact=\"uid=desk,ou=Service,dc=delegation,dc=test\" write");
            }
            1 => p = p.replace("write\n", "write stop\n"),
            2 => p = p.replace("dn.exact=", "dn.base="),
            _ => p = p.replace("by * none", "by * read"),
        }
        assert_ne!(
            p, t.reference,
            "Alternative fixture must actually change the policy"
        );
        assert!(
            t.reports(&p).unwrap().iter().all(|r| r.passed),
            "variant{variant}"
        );
    }
}
#[test]
fn direct_children_only_policy_retains_two_true_passes_but_fails_the_nested_person() {
    let t = &definition().initial;
    let rows = t
        .reports(&t.reference.replace("dn.subtree=", "dn.one="))
        .unwrap();
    assert!(rows[0].passed && rows[1].passed);
    assert!(!rows[2].passed);
    assert!(rows[2].feedback.contains("Nested caller"));
}
#[test]
fn helper_mail_peer_writes_archive_self_and_combined_requests_expose_overgrants() {
    let t = &definition().initial;
    let over = t
        .reference
        .replacen("attrs=description", "attrs=description,mail", 1);
    let rows = t.reports(&over).unwrap();
    assert!(rows.iter().all(|r| !r.passed));
    for row in rows {
        let mail = row
            .outcomes
            .iter()
            .find(|r| r.label == "Desk cannot change mail")
            .unwrap();
        assert!(mail.allowed && !mail.passed);
        assert_ne!(mail.before, mail.after);
    }
    let rows = t.reports("access to * by self write by * none").unwrap();
    assert!(rows.iter().all(|r| !r.passed));
    assert!(
        rows.iter()
            .flat_map(|r| &r.outcomes)
            .any(|r| r.label == "Archived self remains protected" && r.allowed && !r.passed)
    );
    let good = t.run(&t.reference, 1, 8).unwrap();
    assert!(!good.allowed && good.passed);
    assert_eq!(good.before, good.after);
    assert!(good.feedback.contains("mail"));
}
#[test]
fn deny_everything_syntax_errors_and_pasting_the_separate_workshop_do_not_pass() {
    let d = definition();
    for p in [
        "access to * by * none",
        "access to * by users read",
        "access to * by",
        "include /etc/ldap/slapd.conf",
        &d.example.reference,
    ] {
        let rows = d.initial.reports(p).unwrap();
        assert!(rows.iter().all(|r| !r.passed));
    }
}
#[test]
fn progress_drafts_selections_resets_and_stale_success_follow_the_real_authored_definition() {
    let d = definition();
    let mut s = w::Session::new(d).unwrap();
    let initial = s.assignment.clone();
    s.practice.edit(d.example.reference.clone()).unwrap();
    s.practice.check(&d.example).unwrap();
    s.practice.run(&d.example).unwrap();
    assert_eq!(s.assignment, initial);
    assert!(s.assignment.checks(&d.initial).iter().all(|r| !r.0));
    s.assignment.edit(d.initial.reference.clone()).unwrap();
    s.assignment.select(&d.initial, 2, 14).unwrap();
    s.assignment.check(&d.initial).unwrap();
    assert!(s.assignment.checks(&d.initial).iter().all(|r| r.0));
    let raw = serde_json::to_string(&s).unwrap();
    let mut loaded: w::Session = serde_json::from_str(&raw).unwrap();
    loaded.bound(d).unwrap();
    assert_eq!(
        (
            loaded.assignment.selected_case,
            loaded.assignment.selected_probe
        ),
        (2, 14)
    );
    assert!(loaded.assignment.checks(&d.initial).iter().all(|r| r.0));
    assert!(loaded.practice.output.is_none());
    loaded
        .assignment
        .edit("access to * by * write".into())
        .unwrap();
    assert!(loaded.assignment.checks(&d.initial).iter().all(|r| !r.0));
    let practice = loaded.practice.clone();
    loaded.reset_assignment(d);
    assert_eq!(loaded.practice, practice);
    assert_eq!(loaded.assignment, initial);
}
