use dlms_learning_lab::{
    acl_editor, acl_workspace, network_transfer as core, network_transfer_workspace as work,
};
#[path = "support/network_transfer_fixture.rs"]
mod fixture;
use fixture::{definition, task};
use std::collections::BTreeMap;
fn complete(t: &work::Task) -> work::Attempt {
    let mut a = work::Attempt::new(t).unwrap();
    assert!(a.transfer(t).unwrap().success);
    a.check(t).unwrap();
    assert!(a.checks(t).iter().all(|r| r.0));
    a
}
#[test]
fn copies_are_replayed_against_changed_inputs_and_reports_cannot_grant_credit() {
    let t = task();
    let mut a = work::Attempt::new(&t).unwrap();
    assert!(!a.checks(&t).iter().all(|r| r.0));
    a.check(&t).unwrap();
    assert!(!a.checks(&t).iter().all(|r| r.0));
    let r = a.transfer(&t).unwrap();
    assert!(r.success);
    assert!(!a.checked, "Transfer output alone is not completion");
    a.check(&t).unwrap();
    assert!(a.checks(&t).iter().all(|r| r.0));
    for i in 0..2 {
        let w = t.replay(i, &a.history).unwrap();
        assert_eq!(
            w.files["Client"]["staged.cfg"],
            t.cases[i].initial.files["Server"]["public/recovery.cfg"]
        );
    }
    assert_ne!(
        t.replay(0, &a.history).unwrap().files["Client"]["staged.cfg"],
        t.replay(1, &a.history).unwrap().files["Client"]["staged.cfg"]
    );
}
#[test]
fn wrong_source_partial_work_retry_and_actual_overwrite_invalidate_success() {
    let t = task();
    let mut a = work::Attempt::new(&t).unwrap();
    let mut d = a.draft.clone();
    d.request.remote_path = "archive/keep.cfg".into();
    a.edit(&t, d).unwrap();
    assert!(a.transfer(&t).unwrap().success);
    a.check(&t).unwrap();
    assert!(!a.checks(&t).iter().all(|r| r.0));
    assert!(
        t.case_checks(0, &a)
            .iter()
            .any(|r| !r.0 && r.1.contains("different file"))
    );
    a.edit(&t, t.new_draft().unwrap()).unwrap();
    a.transfer(&t).unwrap();
    a.check(&t).unwrap();
    assert!(a.checks(&t).iter().all(|r| r.0));
    let mut d = a.draft.clone();
    d.request.local_path = "startup.cfg".into();
    a.edit(&t, d).unwrap();
    assert!(!a.checked);
    assert!(a.last.is_none());
    assert!(a.transfer(&t).unwrap().success);
    a.check(&t).unwrap();
    assert!(!a.checks(&t).iter().all(|r| r.0));
    assert!(
        t.case_checks(0, &a)
            .iter()
            .any(|r| !r.0 && r.1.contains("overwrote"))
    );
}
#[test]
fn failed_and_malformed_requests_keep_files_and_round_trip_as_separate_drafts() {
    let t = task();
    let mut a = complete(&t);
    let history = a.history.clone();
    let mut d = a.draft.clone();
    d.request.local_path = "../secret".into();
    a.edit(&t, d.clone()).unwrap();
    assert!(a.transfer(&t).is_err());
    assert_eq!(a.history, history);
    assert!(!a.checked);
    assert!(
        a.last_report(&t)
            .unwrap()
            .unwrap_err()
            .contains("traversal")
    );
    let raw = serde_json::to_vec(&a).unwrap();
    let restored: work::Attempt = serde_json::from_slice(&raw).unwrap();
    restored.bound(&t).unwrap();
    assert_eq!(restored, a);
    assert_eq!(restored.draft, d);
    let mut a = restored;
    let mut d = t.new_draft().unwrap();
    d.request.remote_path = "missing.cfg".into();
    a.edit(&t, d).unwrap();
    let r = a.transfer(&t).unwrap();
    assert!(!r.success);
    assert_eq!(a.history, history);
    assert!(!a.last_report(&t).unwrap().unwrap().success);
}
#[test]
fn a_changed_passive_port_case_needs_an_actual_repair_not_only_a_new_draft() {
    let mut t = task();
    for c in &mut t.cases {
        c.requirements = vec![
            work::Requirement::Copied {
                source: work::FileRef {
                    host: "Client".into(),
                    path: "startup.cfg".into(),
                },
                destination: work::FileRef {
                    host: "Server".into(),
                    path: "archive/site.cfg".into(),
                },
            },
            work::Requirement::Ready {
                request: fixture::request(core::Protocol::Ftp, core::Direction::Upload),
            },
        ];
    }
    t.starter = fixture::request(core::Protocol::Ftp, core::Direction::Upload);
    t.cases[1]
        .context
        .servers
        .get_mut("Server")
        .unwrap()
        .passive_port = 50002;
    let doc = |end: &str| {
        format!(
            "ip access-list extended OUTWARD\n 10 permit tcp host 192.0.2.10 host 198.51.100.20 eq 21\n 20 permit tcp host 192.0.2.10 host 198.51.100.20 range 50001 {end}\n 100 deny ip any any\nexit\nip access-list extended INWARD\n 10 permit tcp host 198.51.100.20 eq 21 host 192.0.2.10 eq 49152\n 20 permit tcp host 198.51.100.20 range 50001 {end} host 192.0.2.10 eq 49152\n 100 deny ip any any\nexit\ninterface office\n ip access-group OUTWARD in\n ip access-group INWARD out\nexit\n"
        )
    };
    let mut a = work::Attempt::new(&t).unwrap();
    let mut d = a.draft.clone();
    d.documents.routers.insert("R1".into(), doc("50001"));
    a.edit(&t, d).unwrap();
    assert!(a.transfer(&t).unwrap().success);
    a.check(&t).unwrap();
    assert!(t.case_checks(0, &a).iter().all(|r| r.0));
    assert!(!t.case_checks(1, &a).iter().all(|r| r.0));
    let mut d = a.draft.clone();
    d.documents.routers.insert("R1".into(), doc("50002"));
    a.edit(&t, d).unwrap();
    a.check(&t).unwrap();
    assert!(
        !a.checks(&t).iter().all(|r| r.0),
        "A successful readiness probe on a clone must never create assignment files"
    );
    assert!(a.transfer(&t).unwrap().success);
    assert_eq!(
        a.history.len(),
        2,
        "Changed policy must be retained even if the first sample already had identical destination bytes"
    );
    a.check(&t).unwrap();
    assert!(a.checks(&t).iter().all(|r| r.0));
    for _ in 0..25 {
        assert!(a.transfer(&t).unwrap().success);
    }
    assert_eq!(
        a.history.len(),
        2,
        "Consecutive identical retries do not consume the finite plan budget"
    );
}
#[test]
fn current_acl_errors_clear_success_and_protected_documents_cannot_be_edited() {
    let t = task();
    let mut a = complete(&t);
    let mut d = a.draft.clone();
    d.documents
        .routers
        .insert("R1".into(), "unsupported firewall setting".into());
    a.edit(&t, d).unwrap();
    a.check(&t).unwrap();
    assert!(!a.checks(&t).iter().all(|r| r.0));
    let mut d = t.new_draft().unwrap();
    d.documents
        .routers
        .insert("R2".into(), "! changed protected document\n".into());
    assert!(a.edit(&t, d).is_err());
    let mut d = t.new_draft().unwrap();
    d.documents = acl_editor::Drafts {
        schema_version: 1,
        routers: BTreeMap::new(),
    };
    assert!(a.edit(&t, d).is_err());
}
#[test]
fn malformed_requirements_and_forged_or_oversized_history_are_rejected() {
    let t = task();
    let a = complete(&t);
    let mut forged = a.clone();
    forged.history[0].request.remote_path = "public/missing.cfg".into();
    assert!(forged.bound(&t).is_err());
    let mut forged = a.clone();
    forged.last.as_mut().unwrap().before = 1000;
    assert!(forged.bound(&t).is_err());
    let mut forged = a.clone();
    forged.history = vec![a.history[0].clone(); 17];
    assert!(forged.bound(&t).is_err());
    assert!(
        serde_json::from_slice::<work::Attempt>(&serde_json::to_vec(&forged).unwrap()).is_err()
    );
    let mut invalid = t.clone();
    invalid.cases[0]
        .requirements
        .push(work::Requirement::Unchanged {
            file: work::FileRef {
                host: "UNKNOWN".into(),
                path: "x".into(),
            },
        });
    assert!(invalid.validate().is_err());
    assert!(invalid.case_checks(0, &a).iter().all(|r| !r.0));
    let mut invalid = t;
    invalid.cases[0].initial.files.remove("Client");
    assert!(invalid.replay(0, &[]).is_err());
}
#[test]
fn blocked_protection_is_fresh_and_deny_all_cannot_pass_file_and_readiness_goals() {
    let mut t = task();
    let flow = dlms_learning_lab::acl_routing::Flow {
        source: dlms_learning_lab::acl_routing::Endpoint {
            device: "Client".into(),
            interface: "lan".into(),
        },
        destination: dlms_learning_lab::acl_routing::Endpoint {
            device: "Server".into(),
            interface: "lan".into(),
        },
        protocol: dlms_learning_lab::acl::PacketProtocol::Tcp {
            source_port: 49152,
            destination_port: 23,
        },
    };
    for c in &mut t.cases {
        c.requirements.push(work::Requirement::Blocked {
            label: "Keep Telnet blocked".into(),
            flow: flow.clone(),
        });
    }
    let mut a = complete_without_all(&t);
    a.check(&t).unwrap();
    assert!(!a.checks(&t).iter().all(|r| r.0));
    let mut d = a.draft.clone();
    d.documents.routers.insert("R1".into(),"ip access-list extended DENY\n 10 deny ip any any\nexit\ninterface office\n ip access-group DENY in\nexit\n".into());
    a.edit(&t, d).unwrap();
    a.check(&t).unwrap();
    assert!(!a.checks(&t).iter().all(|r| r.0));
    assert!(
        t.case_checks(0, &a)
            .iter()
            .any(|r| !r.0 && r.1.contains("transfer path"))
    );
}
fn complete_without_all(t: &work::Task) -> work::Attempt {
    let mut a = work::Attempt::new(t).unwrap();
    assert!(a.transfer(t).unwrap().success);
    a
}
#[test]
fn protected_store_copy_has_a_legitimate_alternate_source_when_bytes_really_match() {
    let mut t = task();
    for c in &mut t.cases {
        let text = c.initial.files["Server"]["public/recovery.cfg"].clone();
        c.initial
            .files
            .get_mut("Server")
            .unwrap()
            .insert("public/equivalent.cfg".into(), text);
    }
    let mut a = work::Attempt::new(&t).unwrap();
    let mut d = a.draft.clone();
    d.request.remote_path = "public/equivalent.cfg".into();
    a.edit(&t, d).unwrap();
    assert!(a.transfer(&t).unwrap().success);
    a.check(&t).unwrap();
    assert!(a.checks(&t).iter().all(|r| r.0));
    let _ = acl_workspace::Draft::new(&t.cases[0].context.network).unwrap();
}
#[test]
fn worked_practice_is_isolated_reset_is_selected_and_saved_state_replays_after_reload() {
    let d = definition();
    let mut s = d.new_session().unwrap();
    let original = s.assignment.clone();
    for recipe in &d.worked_example {
        s.practice.edit(&d.example, recipe.clone()).unwrap();
        assert!(s.practice.transfer(&d.example).unwrap().success);
    }
    s.practice.check(&d.example).unwrap();
    assert!(s.checks(&d, true).iter().all(|r| r.0));
    assert!(!s.checks(&d, false).iter().all(|r| r.0));
    assert_eq!(s.assignment, original);
    s.assignment.transfer(&d.assignment).unwrap();
    s.assignment.check(&d.assignment).unwrap();
    assert!(s.checks(&d, false).iter().all(|r| r.0));
    let practice = s.practice.clone();
    let serialized = serde_json::to_vec(&s).unwrap();
    let mut loaded: work::Session = serde_json::from_slice(&serialized).unwrap();
    d.context(&loaded).unwrap();
    assert_eq!(loaded, s);
    assert!(loaded.checks(&d, false).iter().all(|r| r.0));
    loaded.reset(&d, false).unwrap();
    assert_eq!(loaded.assignment, original);
    assert_eq!(loaded.practice, practice);
    assert!(!loaded.checks(&d, false).iter().all(|r| r.0));
    let assignment = loaded.assignment.clone();
    loaded.reset(&d, true).unwrap();
    assert_eq!(loaded.assignment, assignment);
    assert!(loaded.practice.history.is_empty());
}
#[test]
fn definition_rejects_unreachable_worked_values_source_mismatch_and_missing_case_progression() {
    let d = definition();
    let mut broken = d.clone();
    broken.worked_example[0].request.remote_path = "public/nonexistent".into();
    assert!(broken.validate().is_err());
    let mut broken = d.clone();
    broken.assignment.cases.truncate(1);
    assert!(broken.validate().is_err());
    let mut s = d.new_session().unwrap();
    s.id = "other-source".into();
    assert!(d.context(&s).is_err());
    let mut s = d.new_session().unwrap();
    s.version = 2;
    assert!(d.context(&s).is_err());
    let mut s = d.new_session().unwrap();
    s.assignment.checked = true;
    assert!(
        !s.checks(&d, false).iter().all(|r| r.0),
        "A persisted check marker does not create required files"
    );
}
#[test]
fn finite_history_limit_keeps_existing_files_and_draft_without_inventing_a_copy() {
    let t = task();
    let mut a = work::Attempt::new(&t).unwrap();
    for i in 0..work::MAX_OPERATIONS {
        let mut d = a.draft.clone();
        d.request.local_path = format!("copy-{i}.cfg");
        a.edit(&t, d).unwrap();
        assert!(a.transfer(&t).unwrap().success);
    }
    let before = t.replay(0, &a.history).unwrap();
    let mut d = a.draft.clone();
    d.request.local_path = "not-created.cfg".into();
    a.edit(&t, d.clone()).unwrap();
    assert!(a.transfer(&t).unwrap_err().contains("at most 16"));
    assert_eq!(a.history.len(), work::MAX_OPERATIONS);
    assert_eq!(a.draft, d);
    assert_eq!(t.replay(0, &a.history).unwrap(), before);
    a.bound(&t).unwrap();
    assert!(
        a.last_report(&t)
            .unwrap()
            .unwrap_err()
            .contains("retention limit")
    );
    let loaded: work::Attempt = serde_json::from_slice(&serde_json::to_vec(&a).unwrap()).unwrap();
    loaded.bound(&t).unwrap();
    assert_eq!(loaded, a);
}
#[test]
fn saved_byte_budget_refuses_a_new_plan_without_losing_existing_history_or_draft() {
    let t = task();
    let mut a = work::Attempt::new(&t).unwrap();
    let mut draft = a.draft.clone();
    draft
        .documents
        .routers
        .insert("R1".into(), format!("! {}\n", "x".repeat(500)).repeat(120));
    let mut refused = false;
    for i in 0..work::MAX_OPERATIONS {
        draft.request.local_path = format!("budget-{i}.cfg");
        a.edit(&t, draft.clone()).unwrap();
        let before = a.history.clone();
        match a.transfer(&t) {
            Ok(r) => assert!(r.success),
            Err(e) => {
                assert!(e.contains("saved-byte budget"), "{e}");
                assert_eq!(a.history, before);
                assert_eq!(a.draft, draft);
                assert!(!a.checked);
                a.bound(&t).unwrap();
                let w = t.replay(0, &a.history).unwrap();
                assert!(!w.files["Client"].contains_key(&draft.request.local_path));
                refused = true;
                break;
            }
        }
        a.bound(&t).unwrap();
    }
    assert!(
        refused,
        "Finite ordinary fixture should reach the explicit saved-byte bound before the16-operation bound"
    );
}
#[test]
fn a_known_protected_router_document_stays_immutable_even_when_its_syntax_is_valid() {
    let mut t = task();
    t.editable_routers.clear();
    let mut a = work::Attempt::new(&t).unwrap();
    let before = a.clone();
    let mut draft = a.draft.clone();
    draft.documents.routers.insert(
        "R1".into(),
        "! valid comment in a protected document\n".into(),
    );
    assert!(acl_workspace::apply_documents(&t.cases[0].context.network, &draft.documents).is_ok());
    assert!(a.edit(&t, draft).is_err());
    assert_eq!(a, before);
}
