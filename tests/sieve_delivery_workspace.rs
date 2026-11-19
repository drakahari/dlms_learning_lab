use dlms_learning_lab::{
    pack, sieve_source,
    sieve_workspace::{Action, Definition, Session, Task},
};
use serde_json::json;
fn task(practice: bool) -> Task {
    let site = if practice {
        "practice.test"
    } else {
        "delivery.test"
    };
    let sample = |id: &str,
                  from: &str,
                  to: &str,
                  visible: &str,
                  folder: bool,
                  forward: bool,
                  refused: bool| {
        json!({
            "id":id,"label":format!("{id} sample"),"headers":[["From","someone@visible.test"],["To",visible]],"body":"A simulated message.",
            "expected":if folder{vec!["INBOX"]}else{vec![]},
            "envelope":{"from":from,"to":to},
            "expected_redirects":if forward{vec![format!("archive@{site}")]}else{vec![]},
            "expected_rejection":refused
        })
    };
    serde_json::from_value(json!({
        "address_tests":true,"delivery_actions":true,"redirect_targets":[format!("archive@{site}")],
        "starter":"# Repair the delivery rule.\nkeep;\n","mailboxes":["INBOX"],
        "reference":format!("require [\"envelope\",\"reject\"]; if envelope :is \"to\" \"closed@{site}\" {{reject \"This mailbox is retired.\";}} elsif envelope :is \"to\" \"alias@{site}\" {{redirect \"archive@{site}\";}}"),
        "cases":[
            sample("hidden-recipient","sender@outside.test",&format!("alias@{site}"),"other@visible.test",false,true,false),
            sample("visible-decoy","sender@outside.test",&format!("other@{site}"),&format!("alias@{site}"),true,false,false),
            sample("retired","sender@outside.test",&format!("closed@{site}"),"other@visible.test",false,false,true),
            sample("null-sender","",&format!("alias@{site}"),"other@visible.test",false,true,false),
            sample("longer-domain","sender@outside.test",&format!("alias@{site}.outside.test"),"other@visible.test",true,false,false),
            sample("case-domain","sender@outside.test",&format!("alias@{}",site.to_uppercase()),"other@visible.test",false,true,false)
        ]
    })).unwrap()
}
fn definition() -> Definition {
    Definition {
        schema: 1,
        source_id: "delivery-capability-fixture".into(),
        assignment: task(false),
        example: task(true),
    }
}
fn run(s: &mut Session, d: &Definition, code: &str, practice: bool) {
    s.action(
        d,
        &Action::Edit {
            script: code.into(),
        },
        practice,
    )
    .unwrap();
    s.action(d, &Action::Run, practice).unwrap();
    for _ in 0..6 {
        s.action(d, &Action::Tick, practice).unwrap();
    }
}
#[test]
fn outcome_grading_accepts_equivalent_rules_and_refusal_wording_not_exact_source() {
    let d = definition();
    d.validate().unwrap();
    let code = r#"require ["envelope","reject"]; if envelope :matches "to" "alias@delivery.test" {redirect "archive@DELIVERY.TEST"; stop;} if envelope :all :is "to" "closed@delivery.test" {reject "Please contact support about the retired mailbox.";} else {keep;}"#;
    let mut s = Session::new(&d).unwrap();
    run(&mut s, &d, code, false);
    assert!(d.solved(&s), "{:?}", d.checks(&s));
    assert_eq!(s.assignment.results[0].redirects.len(), 1);
    assert!(!s.assignment.results[0].discarded);
    assert!(s.assignment.results[2].rejection.is_some());
    assert!(!s.assignment.results[2].discarded);
    assert!(d.checks(&s).iter().all(|c| c.passed));
}
#[test]
fn plausible_wrong_effects_fail_with_specific_diagnostics_and_retry_succeeds() {
    let d = definition();
    for (code, needle) in [
        (
            r#"require ["envelope","reject"];if envelope "to" "closed@delivery.test" {discard;} elsif envelope "to" "alias@delivery.test" {redirect "archive@delivery.test";}"#,
            "Discard silently",
        ),
        (
            r#"require ["envelope","reject"];if envelope "to" "closed@delivery.test" {reject " ";} elsif envelope "to" "alias@delivery.test" {redirect "archive@delivery.test";}"#,
            "nonblank",
        ),
        (
            r#"require ["envelope","reject"];if envelope "to" "closed@delivery.test" {reject "No";} elsif header "To" "alias@delivery.test" {redirect "archive@delivery.test";}"#,
            "forwarding targets differ",
        ),
        (
            r#"require ["envelope","reject"];if envelope "to" "closed@delivery.test" {reject "No";} elsif envelope "to" "alias@delivery.test" {redirect "archive@delivery.test";keep;}"#,
            "explicit keep adds",
        ),
    ] {
        let mut s = Session::new(&d).unwrap();
        run(&mut s, &d, code, false);
        assert!(!d.solved(&s));
        assert!(s.assignment.results.iter().any(|r| r.passed));
        assert!(
            d.checks(&s)
                .iter()
                .any(|c| !c.passed && c.detail.contains(needle)),
            "{:?}",
            d.checks(&s)
        );
        run(&mut s, &d, &d.assignment.reference, false);
        assert!(d.solved(&s));
        s.action(
            &d,
            &Action::Edit {
                script: "keep;".into(),
            },
            false,
        )
        .unwrap();
        assert!(s.assignment.results.is_empty());
        assert!(!d.solved(&s));
    }
}
#[test]
fn practice_partial_stop_restore_and_reset_preserve_independent_work() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    let untouched = s.assignment.clone();
    run(&mut s, &d, &d.example.reference, true);
    assert!(s.practice.solved(&d.example));
    assert_eq!(s.assignment, untouched);
    assert!(!d.solved(&s));
    s.action(
        &d,
        &Action::Edit {
            script: d.assignment.reference.clone(),
        },
        false,
    )
    .unwrap();
    s.action(&d, &Action::Run, false).unwrap();
    s.action(&d, &Action::Tick, false).unwrap();
    s.action(&d, &Action::Stop, false).unwrap();
    assert!(s.assignment.stopped);
    assert_eq!(s.assignment.results.len(), 1);
    assert!(!d.solved(&s));
    let bytes = serde_json::to_vec(&s).unwrap();
    let restored: Session = serde_json::from_slice(&bytes).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, s);
    let practice = s.practice.clone();
    s.action(&d, &Action::Reset, false).unwrap();
    assert_eq!(s.practice, practice);
    assert_eq!(s.assignment, untouched);
}
#[test]
fn forged_delivery_effects_and_invalid_profiles_are_rejected() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    run(&mut s, &d, &d.assignment.reference, false);
    s.assignment.results[0].redirects.clear();
    assert!(d.bound(&s).is_err());
    assert!(!d.solved(&s));
    for mutation in 0..6 {
        let mut bad = d.clone();
        match mutation {
            0 => bad.assignment.address_tests = false,
            1 => bad.assignment.cases[0].envelope = None,
            2 => {
                bad.assignment.cases[2].expected.insert("INBOX".into());
            }
            3 => {
                bad.assignment.cases[0].expected_redirects =
                    ["unknown@elsewhere.test".into()].into()
            }
            4 => bad.example.delivery_actions = false,
            _ => {
                bad.assignment
                    .redirect_targets
                    .insert("archive@DELIVERY.TEST".into());
            }
        }
        assert!(bad.validate().is_err(), "{mutation}");
    }
    let mut old: Task = serde_json::from_value(serde_json::to_value(task(false)).unwrap()).unwrap();
    old.delivery_actions = false;
    old.address_tests = true;
    old.redirect_targets.clear();
    assert!(old.validate().is_err());
}
#[test]
fn all_five_prior_lesson_tasks_and_saved_results_keep_exact_serialized_fields() {
    for raw in [
        include_str!("../scenarios/sieve-filter-lessons-v1.json"),
        include_str!("../scenarios/sieve-address-lessons-v1.json"),
    ] {
        let value: serde_json::Value = serde_json::from_str(raw).unwrap();
        for source in value["scenarios"].as_array().unwrap() {
            for scope in ["assignment", "example"] {
                let v = source["goal"]["definition"][scope].clone();
                let t: Task = serde_json::from_value(v.clone()).unwrap();
                assert!(!t.delivery_actions);
                assert_eq!(serde_json::to_value(&t).unwrap(), v);
                let mut s = dlms_learning_lab::sieve_workspace::Scope::new(&t);
                s.edit(t.reference.clone()).unwrap();
                s.start_for(&t);
                for _ in 0..t.cases.len() {
                    s.tick(&t);
                }
                assert!(s.solved(&t));
                let result = serde_json::to_value(&s.results[0]).unwrap();
                assert!(result.get("redirects").is_none() && result.get("rejection").is_none());
            }
        }
    }
    // The registered catalog remains 472: these are capability fixtures, not new labs.
    let labs = pack::accepted472().unwrap();
    assert_eq!(labs.len(), 472);
    for source in labs.iter().filter(|s| s.id.starts_with("lpic2-sieve-")) {
        sieve_source::source(source).unwrap();
    }
}
