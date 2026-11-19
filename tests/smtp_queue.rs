use dlms_learning_lab::{
    smtp_config::ErrorKind,
    smtp_queue::{Facts, Runtime, Sample, Status},
    smtp_routing::Outcome,
};
use std::collections::BTreeMap;
#[path = "support/smtp_fixture.rs"]
mod setup;
fn fixture() -> Facts {
    let (world, active) = setup::fixture();
    Facts {
        world,
        initial_files: active.clone(),
        initial_active: active,
        samples: BTreeMap::from([
            (
                "staff-notice".into(),
                Sample {
                    entry: "gate".into(),
                    sender: Some("service@sender.test".into()),
                    recipient: "nora@staff.lab.test".into(),
                    subject: "Shift notice".into(),
                    body: "A fixed harmless sample body. $(not-a-command)\n".into(),
                },
            ),
            (
                "external-receipt".into(),
                Sample {
                    entry: "gate".into(),
                    sender: None,
                    recipient: "visitor@outside.test".into(),
                    subject: "Receipt sample".into(),
                    body: "Only represented mailboxes receive this sample.".into(),
                },
            ),
        ]),
    }
}
#[test]
fn queue_retry_changes_typed_delivery_state_without_modifying_source_payloads_or_configs() {
    let f = fixture();
    let before = f.clone();
    let mut r = Runtime::new(&f).unwrap();
    assert_eq!(r.pending("gate", &f).unwrap().len(), 2);
    assert!(
        r.flush(&f, "relay").unwrap().is_empty(),
        "A relay does not own another server's queue"
    );
    let seen = r.flush(&f, "gate").unwrap();
    assert_eq!(seen.len(), 2);
    assert!(
        r.messages
            .values()
            .all(|m| matches!(m.status, Status::Delivered { .. }))
    );
    assert!(r.pending("gate", &f).unwrap().is_empty());
    assert_eq!(r.files, f.initial_files);
    assert_eq!(r.active, f.initial_active);
    assert_eq!(f, before);
    assert!(
        r.flush(&f, "gate").unwrap().is_empty(),
        "Already delivered work is not sent twice by an ordinary retry"
    );
    r.validate(&f).unwrap();
}
#[test]
fn saving_and_reloading_are_separate_and_a_malformed_file_cannot_replace_active_configuration() {
    let f = fixture();
    let mut r = Runtime::new(&f).unwrap();
    let active = r.active.clone();
    let mail = r.messages.clone();
    r.save(&f, "gate", "not a setting").unwrap();
    assert_eq!(r.active, active);
    assert!(r.reload(&f, "gate").is_err());
    assert_eq!(r.active, active);
    assert_eq!(r.messages, mail);
    r.validate(&f).unwrap();
    let good = f.initial_files["gate"].clone();
    r.save(&f, "gate", &good).unwrap();
    r.reload(&f, "gate").unwrap();
    assert_eq!(r.active, active);
}
#[test]
fn a_deferred_loop_remains_pending_then_a_real_reload_and_retry_repair_it() {
    let f = fixture();
    let mut r = Runtime::new(&f).unwrap();
    let looped = r.files["relay"].replace("[delivery.lab.test]", "[gate.lab.test]");
    r.save(&f, "relay", &looped).unwrap();
    assert_eq!(r.active, f.initial_active);
    r.reload(&f, "relay").unwrap();
    let seen = r.flush(&f, "gate").unwrap();
    assert_eq!(seen.len(), 2);
    assert!(
        seen.iter()
            .all(|(_, t)| matches!(t.outcome, Outcome::Loop { .. }))
    );
    assert!(
        r.messages
            .values()
            .all(|m| matches!(m.status, Status::Deferred { .. }))
    );
    assert_eq!(r.pending("gate", &f).unwrap().len(), 2);
    r.save(&f, "relay", &f.initial_files["relay"]).unwrap();
    assert!(
        r.flush(&f, "gate")
            .unwrap()
            .iter()
            .all(|(_, t)| matches!(t.outcome, Outcome::Loop { .. })),
        "Saving alone cannot repair the running relay"
    );
    r.reload(&f, "relay").unwrap();
    r.flush(&f, "gate").unwrap();
    assert!(
        r.messages
            .values()
            .all(|m| matches!(m.status, Status::Delivered { .. }))
    );
    r.validate(&f).unwrap();
}
#[test]
fn an_empty_queue_does_not_prove_delivery_and_sample_reset_preserves_configuration_work() {
    let f = fixture();
    let mut r = Runtime::new(&f).unwrap();
    let wrong = r.files["gate"].replace(
        "gate.lab.test, localhost",
        "gate.lab.test, staff.lab.test, localhost",
    );
    r.save(&f, "gate", &wrong).unwrap();
    r.reload(&f, "gate").unwrap();
    r.flush(&f, "gate").unwrap();
    assert!(matches!(
        r.messages["staff-notice"].status,
        Status::Failed { .. }
    ));
    assert!(r.pending("gate", &f).unwrap().is_empty());
    assert!(
        !r.messages
            .values()
            .all(|m| matches!(m.status, Status::Delivered { .. }))
    );
    r.save(&f, "gate", &f.initial_files["gate"]).unwrap();
    r.reload(&f, "gate").unwrap();
    let (saved, active) = (r.files.clone(), r.active.clone());
    let ids = r
        .messages
        .iter()
        .map(|(id, m)| (id.clone(), m.queue_id.clone()))
        .collect::<BTreeMap<_, _>>();
    r.reset_samples(&f).unwrap();
    assert_eq!(r.files, saved);
    assert_eq!(r.active, active);
    assert!(
        r.messages
            .values()
            .all(|m| matches!(m.status, Status::Queued))
    );
    assert_eq!(
        r.messages
            .iter()
            .map(|(id, m)| (id.clone(), m.queue_id.clone()))
            .collect::<BTreeMap<_, _>>(),
        ids
    );
    r.flush(&f, "gate").unwrap();
    assert!(
        r.messages
            .values()
            .all(|m| matches!(m.status, Status::Delivered { .. }))
    );
}
#[test]
fn already_admitted_mail_bypasses_its_owners_new_relay_policy_but_not_the_next_servers_policy() {
    let f = fixture();
    let mut r = Runtime::new(&f).unwrap();
    let gate = r.files["gate"].replace("192.0.2.0/24", "192.0.2.99/32");
    r.save(&f, "gate", &gate).unwrap();
    r.reload(&f, "gate").unwrap();
    assert!(matches!(
        f.world
            .trace(
                &r.active,
                "gate",
                setup::ip("192.0.2.50"),
                "visitor@outside.test"
            )
            .unwrap()
            .outcome,
        Outcome::Rejected { .. }
    ));
    r.flush(&f, "gate").unwrap();
    assert!(matches!(
        r.messages["external-receipt"].status,
        Status::Delivered { .. }
    ));
    r.reset_samples(&f).unwrap();
    let relay = r.files["relay"].replace("192.0.2.10/32", "192.0.2.99/32");
    r.save(&f, "relay", &relay).unwrap();
    r.reload(&f, "relay").unwrap();
    r.flush(&f, "gate").unwrap();
    assert!(matches!(
        r.messages["external-receipt"].status,
        Status::Failed { .. }
    ));
}
#[test]
fn temporary_failures_stay_queued_and_partial_delivery_is_never_all_success() {
    let mut f = fixture();
    f.world.nodes.get_mut("relay").unwrap().available = false;
    let mut r = Runtime::new(&f).unwrap();
    r.flush(&f, "gate").unwrap();
    assert_eq!(r.pending("gate", &f).unwrap().len(), 2);
    assert!(
        r.messages
            .values()
            .all(|m| matches!(m.status, Status::Deferred { .. }))
    );
    let mut f = fixture();
    f.samples.get_mut("staff-notice").unwrap().recipient = "team@staff.lab.test".into();
    f.world.nodes.get_mut("delivery").unwrap().aliases = "team: nora, absent\n".into();
    let mut r = Runtime::new(&f).unwrap();
    r.flush(&f, "gate").unwrap();
    assert!(
        matches!(&r.messages["staff-notice"].status,Status::Partial{mailboxes,failures,..}if mailboxes.contains("nora")&&!failures.is_empty())
    );
    r.validate(&f).unwrap();
}
#[test]
fn snapshots_resume_deferred_and_distinct_file_sets_while_another_runtime_is_untouched() {
    let f = fixture();
    let mut assignment = Runtime::new(&f).unwrap();
    let practice = Runtime::new(&f).unwrap();
    let looped = assignment.files["relay"].replace("[delivery.lab.test]", "[gate.lab.test]");
    assignment.save(&f, "relay", &looped).unwrap();
    assignment.reload(&f, "relay").unwrap();
    assignment.flush(&f, "gate").unwrap();
    assignment.save(&f, "gate", "unfinished text").unwrap();
    let raw = serde_json::to_vec(&assignment).unwrap();
    let mut restored: Runtime = serde_json::from_slice(&raw).unwrap();
    assert_eq!(restored, assignment);
    restored.validate(&f).unwrap();
    restored.reset_samples(&f).unwrap();
    assert_eq!(restored.files["gate"], "unfinished text");
    assert_eq!(practice, Runtime::new(&f).unwrap());
    assert_eq!(assignment, serde_json::from_slice::<Runtime>(&raw).unwrap());
}
#[test]
fn foreign_deleted_forged_and_oversized_records_cannot_be_valid_progress() {
    let f = fixture();
    let original = Runtime::new(&f).unwrap();
    let mut r = original.clone();
    r.messages.remove("staff-notice");
    assert!(r.validate(&f).is_err());
    let mut r = original.clone();
    r.messages.get_mut("staff-notice").unwrap().queue_id = "FOREIGN".into();
    assert!(r.validate(&f).is_err());
    let mut r = original.clone();
    r.messages.get_mut("staff-notice").unwrap().status = Status::Delivered {
        node: "delivery".into(),
        mailboxes: Default::default(),
    };
    assert!(r.validate(&f).is_err());
    let mut r = original.clone();
    r.version = 2;
    assert!(r.validate(&f).is_err());
    let mut r = original.clone();
    r.active
        .insert("foreign".into(), f.initial_active["gate"].clone());
    assert!(r.validate(&f).is_err());
    let mut r = original.clone();
    assert_eq!(
        r.save(&f, "gate", &"x".repeat(8193)).unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(r, original);
    assert!(r.flush(&f, "../../host").is_err());
    assert_eq!(r, original);
    let mut raw = serde_json::to_value(&original).unwrap();
    raw["execute_on_host"] = true.into();
    assert!(serde_json::from_value::<Runtime>(raw).is_err());
}
#[test]
fn malformed_sources_and_excess_messages_fail_before_a_runtime_can_be_created() {
    let mut f = fixture();
    let sample = f.samples["staff-notice"].clone();
    for n in 0..17 {
        f.samples.insert(format!("extra-{n}"), sample.clone());
    }
    assert_eq!(Runtime::new(&f).unwrap_err().kind, ErrorKind::Limit);
    let mut f = fixture();
    f.initial_active
        .insert("gate".into(), "not a setting".into());
    assert!(Runtime::new(&f).is_err());
    let mut f = fixture();
    f.initial_files
        .insert("gate".into(), "not a setting".into());
    Runtime::new(&f).unwrap().validate(&f).unwrap();
    let mut f = fixture();
    f.samples.get_mut("staff-notice").unwrap().body = "x".repeat(2049);
    assert!(Runtime::new(&f).is_err());
}
