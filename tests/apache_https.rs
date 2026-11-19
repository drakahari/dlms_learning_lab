use dlms_learning_lab::apache_https::{
    Name, Trust, config,
    session::Definition,
    workspace::{Action, Task, Work},
};
use serde_json::Value;

fn definition(id: &str) -> (Definition, Value) {
    let all: Value =
        serde_json::from_str(include_str!("fixtures/apache_https_tasks_v1.json")).unwrap();
    let row = &all[id];
    (
        serde_json::from_value(row["definition"].clone()).unwrap(),
        row["answers"].clone(),
    )
}

fn apply(task: &Task, work: &mut Work, configuration: &str) {
    work.apply(
        task,
        Action::Draft {
            contents: configuration.into(),
        },
    )
    .unwrap();
    work.apply(task, Action::Save).unwrap();
    work.apply(task, Action::Apply).unwrap();
}

fn passed(work: &Work, task: &Task) -> usize {
    work.checks(task)
        .iter()
        .filter(|check| check.passed)
        .count()
}

#[test]
fn applied_chain_and_host_results_are_independent_of_save_and_key_binding() {
    let (d, answers) = definition("lpic2-apache-https-chain");
    let t = &d.assignment;
    d.validate().unwrap();
    let mut w = Work::new(t);
    let initial = w.inspect(t, &t.checks[0]).unwrap().verification;
    assert!(initial.hostname_matches);
    assert!(matches!(initial.trust, Trust::MissingIssuer { .. }));
    assert_eq!(passed(&w, t), 2);
    let good = answers["assignment"].as_str().unwrap();
    w.apply(
        t,
        Action::Draft {
            contents: good.into(),
        },
    )
    .unwrap();
    w.apply(t, Action::Save).unwrap();
    assert_eq!(w.inspect(t, &t.checks[0]).unwrap().verification, initial);
    assert_eq!(passed(&w, t), 1);
    w.apply(t, Action::Apply).unwrap();
    assert_eq!(passed(&w, t), 3);
    w.apply(t, Action::Check).unwrap();
    let mismatch = good.replace("/reports.key", "/archive.key");
    w.apply(t, Action::Draft { contents: mismatch }).unwrap();
    assert!(!w.checked);
    w.apply(t, Action::Save).unwrap();
    assert!(
        w.apply(t, Action::Apply)
            .unwrap_err()
            .contains("does not match")
    );
    assert_eq!(w.applied, good);
    assert_eq!(passed(&w, t), 2);
    assert!(matches!(
        w.inspect(t, &t.checks[0]).unwrap().verification.trust,
        Trust::Verified(_)
    ));
}

#[test]
fn alternate_bundle_passes_expired_leaf_and_authority_confusion_fail() {
    let (d, answers) = definition("lpic2-apache-https-chain");
    let t = &d.assignment;
    let mut w = Work::new(t);
    let alternate = answers["alternate"].as_str().unwrap();
    apply(t, &mut w, alternate);
    assert_eq!(passed(&w, t), 3);
    let mut untrusted = t.checks[0].clone();
    untrusted.client.trusted.clear();
    let result = w.inspect(t, &untrusted).unwrap().verification;
    assert!(result.hostname_matches);
    assert_eq!(
        result.trust,
        Trust::UnknownSelfSigned(Name("campus-root".into()))
    );
    apply(t, &mut w, answers["expired"].as_str().unwrap());
    assert_eq!(passed(&w, t), 2);
    assert!(matches!(
        w.inspect(t, &t.checks[0]).unwrap().verification.trust,
        Trust::OutsideValidity(_)
    ));
}

#[test]
fn named_clients_and_two_default_cases_require_different_outcomes() {
    let (d, answers) = definition("lpic2-apache-https-sni");
    let t = &d.assignment;
    let mut w = Work::new(t);
    assert_eq!(passed(&w, t), 1);
    let mismatched_name = w.inspect(t, &t.checks[0]).unwrap();
    assert!(!mismatched_name.used_default);
    assert!(!mismatched_name.verification.hostname_matches);
    assert!(matches!(
        mismatched_name.verification.trust,
        Trust::Verified(_)
    ));
    let good = answers["assignment"].as_str().unwrap();
    apply(t, &mut w, good);
    assert_eq!(passed(&w, t), 5);
    let blocks = good.split("\n\n").collect::<Vec<_>>();
    assert_eq!(blocks.len(), 2);
    apply(t, &mut w, &format!("{}\n\n{}\n", blocks[1], blocks[0]));
    assert_eq!(passed(&w, t), 3);
    for case in &t.checks[2..] {
        let outcome = w.inspect(t, case).unwrap();
        assert!(outcome.used_default);
        assert!(!outcome.verification.hostname_matches);
    }
}

#[test]
fn rejects_unmodeled_directives_and_does_not_apply_invalid_paths() {
    let (d, _) = definition("lpic2-apache-https-chain");
    let initial = &d.assignment.starter;
    for (literal, contains) in [
        ("Include /etc/passwd", "supports only"),
        ("SSLProtocol -all", "supports only"),
        (
            "SSLCertificateChainFile /etc/ssl/campus/other.pem",
            "intermediates",
        ),
        (
            "SSLCertificateKeyFile /etc/ssl/campus/../key",
            "literal filename",
        ),
        ("ServerName `escape.example`", "literal DNS name"),
        ("SSLEngine on; echo", "one literal value"),
    ] {
        let changed = initial.replacen("SSLEngine on", literal, 1);
        assert!(
            config::parse(&changed).unwrap_err().contains(contains),
            "{literal}"
        );
    }
    assert!(config::parse(&"\n".repeat(49)).is_err());
    assert!(config::parse(&"#".repeat(4097)).is_err());
}

#[test]
fn client_path_reports_an_issuer_role_error_or_a_real_cycle() {
    let (mut d, answers) = definition("lpic2-apache-https-chain");
    let t = &mut d.assignment;
    let alternate = answers["alternate"].as_str().unwrap();
    let mut w = Work::new(t);
    apply(t, &mut w, alternate);
    let name = Name("campus-intermediate".into());
    t.inventory.certificates[1].is_ca = false;
    t.inventory.certificates[1]
        .dns_names
        .push(Name("issuer.example".into()));
    t.validate().unwrap();
    assert_eq!(
        w.inspect(t, &t.checks[0]).unwrap().verification.trust,
        Trust::IssuerNotCa(name.clone())
    );
    t.inventory.certificates[1].is_ca = true;
    t.inventory.certificates[1].dns_names.clear();
    t.inventory.certificates[0].issuer = name;
    t.checks[0].client.trusted.clear();
    assert!(matches!(
        w.inspect(t, &t.checks[0]).unwrap().verification.trust,
        Trust::Cycle(_)
    ));
}
