use dlms_learning_lab::{
    sieve_envelope::bare,
    sieve_lex::{Kind, MAX_STRING},
    sieve_vacation_history::{
        CAPACITY, Decision, History, ResponseKey, SECONDS_PER_DAY, effective_days,
    },
};
fn response(body: &str) -> ResponseKey {
    ResponseKey::Plain {
        subject: None,
        from: None,
        reason: body.into(),
    }
}
#[test]
fn days_use_default_and_site_clamps_instead_of_rejecting_out_of_range_values() {
    assert_eq!(effective_days(None), 7);
    assert_eq!(effective_days(Some(0)), 1);
    assert_eq!(effective_days(Some(1)), 1);
    assert_eq!(effective_days(Some(31)), 31);
    assert_eq!(effective_days(Some(u64::MAX)), 365);
    let mut h = History::default();
    assert_eq!(
        h.consider(
            &bare("one@outside.test").unwrap(),
            &response("Away"),
            0,
            Some(0)
        )
        .unwrap(),
        Decision::Send { effective_days: 1 }
    );
}
#[test]
fn exact_second_expiry_and_suppressed_arrivals_do_not_extend_the_interval() {
    let mut h = History::default();
    let sender = bare("one@outside.test").unwrap();
    let key = response("Away");
    assert_eq!(
        h.consider(&sender, &key, 10, Some(1)).unwrap(),
        Decision::Send { effective_days: 1 }
    );
    assert_eq!(
        h.consider(&sender, &key, 10 + SECONDS_PER_DAY - 1, Some(1))
            .unwrap(),
        Decision::Suppressed {
            last_sent: 10,
            remaining_seconds: 1
        }
    );
    assert_eq!(
        h.consider(&sender, &key, 10 + SECONDS_PER_DAY, Some(1))
            .unwrap(),
        Decision::Send { effective_days: 1 }
    );
    assert_eq!(
        h.consider(&sender, &key, 10 + SECONDS_PER_DAY, Some(1))
            .unwrap(),
        Decision::Suppressed {
            last_sent: 10 + SECONDS_PER_DAY,
            remaining_seconds: SECONDS_PER_DAY
        }
    );
}
#[test]
fn sender_and_raw_response_parameters_define_distinct_history_entries() {
    let mut h = History::default();
    let a = bare("one@outside.test").unwrap();
    let b = bare("two@outside.test").unwrap();
    assert!(matches!(
        h.consider(&a, &response("Away"), 0, None).unwrap(),
        Decision::Send { .. }
    ));
    assert!(matches!(
        h.consider(&b, &response("Away"), 0, None).unwrap(),
        Decision::Send { .. }
    ));
    assert!(matches!(
        h.consider(&a, &response("Different message"), 0, None)
            .unwrap(),
        Decision::Send { .. }
    ));
    let one = ResponseKey::Plain {
        subject: Some("x".into()),
        from: None,
        reason: "y".into(),
    };
    let two = ResponseKey::Plain {
        subject: None,
        from: Some("x".into()),
        reason: "y".into(),
    };
    assert!(matches!(
        h.consider(&a, &one, 0, None).unwrap(),
        Decision::Send { .. }
    ));
    assert!(matches!(
        h.consider(&a, &two, 0, None).unwrap(),
        Decision::Send { .. }
    ));
    assert_eq!(h.len(), 5);
}
#[test]
fn a_handle_reuses_identity_and_days_are_not_part_of_the_response_key() {
    let mut h = History::default();
    let sender = bare("one@outside.test").unwrap();
    let key = ResponseKey::Handle("absence".into());
    assert!(matches!(
        h.consider(&sender, &key, 0, Some(7)).unwrap(),
        Decision::Send { .. }
    ));
    assert_eq!(
        h.consider(&sender, &key, 1, Some(1)).unwrap(),
        Decision::Suppressed {
            last_sent: 0,
            remaining_seconds: SECONDS_PER_DAY - 1
        }
    );
    assert!(matches!(
        h.consider(&sender, &key, SECONDS_PER_DAY, Some(1)).unwrap(),
        Decision::Send { .. }
    ));
    assert_eq!(h.len(), 1);
    assert!(matches!(
        h.consider(
            &sender,
            &ResponseKey::Handle("different".into()),
            SECONDS_PER_DAY,
            None
        )
        .unwrap(),
        Decision::Send { .. }
    ));
}
#[test]
fn sender_domain_case_is_ignored_but_local_case_follows_the_disclosed_site_policy() {
    let mut h = History::default();
    let key = response("Away");
    h.consider(&bare("Name@Outside.Test").unwrap(), &key, 0, None)
        .unwrap();
    assert!(matches!(
        h.consider(&bare("Name@outside.test").unwrap(), &key, 1, None)
            .unwrap(),
        Decision::Suppressed { .. }
    ));
    assert!(matches!(
        h.consider(&bare("name@outside.test").unwrap(), &key, 1, None)
            .unwrap(),
        Decision::Send { .. }
    ));
}
#[test]
fn capacity_retains1000_entries_and_evicts_the_oldest_deterministically() {
    assert_eq!(CAPACITY, 1000);
    let mut h = History::default();
    let key = response("Away");
    for n in 0..CAPACITY {
        h.consider(
            &bare(&format!("sender{n}@outside.test")).unwrap(),
            &key,
            n as u64,
            None,
        )
        .unwrap();
    }
    assert_eq!(h.len(), CAPACITY);
    h.consider(
        &bare("new@outside.test").unwrap(),
        &key,
        CAPACITY as u64,
        None,
    )
    .unwrap();
    assert_eq!(h.len(), CAPACITY);
    assert!(matches!(
        h.consider(
            &bare("sender1@outside.test").unwrap(),
            &key,
            CAPACITY as u64,
            None
        )
        .unwrap(),
        Decision::Suppressed { .. }
    ));
    assert!(matches!(
        h.consider(
            &bare("sender0@outside.test").unwrap(),
            &key,
            CAPACITY as u64,
            None
        )
        .unwrap(),
        Decision::Send { .. }
    ));
    assert_eq!(h.len(), CAPACITY);
}
#[test]
fn malformed_api_values_and_backward_time_leave_history_intact() {
    let mut h = History::default();
    let key = response("Away");
    let sender = bare("one@outside.test").unwrap();
    h.consider(&sender, &key, 100, None).unwrap();
    let before = h.clone();
    assert_eq!(
        h.consider(&sender, &key, 99, None).unwrap_err().kind,
        Kind::Runtime
    );
    assert_eq!(h, before);
    assert_eq!(
        h.consider(
            &sender,
            &ResponseKey::Handle("x".repeat(MAX_STRING + 1)),
            101,
            None
        )
        .unwrap_err()
        .kind,
        Kind::Limit
    );
    assert_eq!(h, before);
    let invalid = dlms_learning_lab::sieve_address::Mailbox {
        local: "../bad@extra".into(),
        domain: "outside.test".into(),
    };
    assert!(h.consider(&invalid, &key, 101, None).is_err());
    assert_eq!(h, before);
}
#[test]
fn interval_comparison_near_u64_max_does_not_overflow() {
    let mut h = History::default();
    let key = response("Away");
    let sender = bare("one@outside.test").unwrap();
    h.consider(&sender, &key, u64::MAX - 1, None).unwrap();
    assert_eq!(
        h.consider(&sender, &key, u64::MAX, None).unwrap(),
        Decision::Suppressed {
            last_sent: u64::MAX - 1,
            remaining_seconds: 7 * SECONDS_PER_DAY - 1
        }
    );
}
