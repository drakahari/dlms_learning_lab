use dlms_learning_lab::{
    dns_query_tools as q, dns_teaching,
    dns_workspace::{Draft, UtilityDraft},
    engine::{self, State},
    pack,
};
fn scenario() -> pack::Scenario {
    pack::load(include_str!("../scenarios/dns-forward-v1.json"))
        .unwrap()
        .into_iter()
        .find(|s| s.id == "lpic2-dns-dualstack")
        .unwrap()
}
fn plan() -> q::Plan {
    q::Plan {
        endpoints: vec![q::Endpoint {
            address: "192.0.2.53".parse().unwrap(),
            served: q::Served::Primary,
        }],
    }
}
#[test]
fn legacy_source_and_draft_bytes_stay_equivalent_when_the_new_capability_is_absent() {
    let complete = pack::load_catalog(&[
        include_str!("../scenarios/dns-forward-v1.json"),
        include_str!("../scenarios/dns-reverse-v1.json"),
    ])
    .unwrap();
    for text in [
        include_str!("../scenarios/dns-forward-v1.json"),
        include_str!("../scenarios/dns-reverse-v1.json"),
    ] {
        let raw: serde_json::Value = serde_json::from_str(text).unwrap();
        let labs = complete.iter().filter(|s| {
            raw["scenarios"]
                .as_array()
                .unwrap()
                .iter()
                .any(|x| x["id"] == s.id)
        });
        for s in labs {
            let original = raw["scenarios"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["id"] == s.id)
                .unwrap();
            assert_eq!(
                serde_json::to_value(&s.dns_teaching).unwrap(),
                original["dns_teaching"]
            );
            let State::AuthoritativeDns { server } = &s.initial else {
                panic!()
            };
            let draft = Draft::new(server);
            assert!(draft.utility.is_none());
            let bytes = serde_json::to_vec(&draft).unwrap();
            assert!(!String::from_utf8_lossy(&bytes).contains("utility"));
            let restored: Draft = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
        }
    }
}
#[test]
fn observed_commands_do_not_repair_or_grade_and_failures_clear_prior_output() {
    let s = scenario();
    let State::AuthoritativeDns { server } = &s.initial else {
        panic!()
    };
    let original = server.clone();
    let mut d = Draft::new(server);
    let p = plan();
    d.utility = Some(UtilityDraft {
        enabled: true,
        input: "dig @192.0.2.53 files.workshop.test. AAAA +norecurse +short".into(),
    });
    d.lookup_utility(server, &p).unwrap();
    assert!(d.output.contains("NXDOMAIN"));
    assert!(d.output.contains("command exit status 0"));
    assert_eq!(*server, original);
    assert!(!engine::solved(&s.initial, &s.goal));
    d.utility.as_mut().unwrap().input = "host -r -t AAAA web.workshop.test. 192.0.2.53".into();
    d.lookup_utility(server, &p).unwrap();
    assert!(d.output.contains("has no AAAA record"));
    assert!(d.output.contains("NOERROR"));
    d.utility.as_mut().unwrap().input =
        "dig @1.1.1.1 web.workshop.test. A +norecurse +short".into();
    assert!(d.lookup_utility(server, &p).is_err());
    assert!(d.output.is_empty());
    assert_eq!(*server, original);
    d.utility.as_mut().unwrap().enabled = false;
    assert!(d.lookup_utility(server, &p).is_err());
    assert!(d.output.is_empty());
}
#[test]
fn separate_example_fill_and_draft_resume_preserve_inert_commands_without_credit() {
    let s = scenario();
    let lesson = s.dns_teaching.as_ref().unwrap();
    let mut learning = dns_teaching::Learning::new(lesson);
    let input = "host -r -t A web.dual-practice.test. 192.0.2.53";
    learning.draft.utility = Some(UtilityDraft {
        enabled: true,
        input: input.into(),
    });
    learning
        .draft
        .lookup_utility(&learning.server, &plan())
        .unwrap();
    learning.fill_example(lesson).unwrap();
    assert_eq!(learning.draft.utility.as_ref().unwrap().input, input);
    assert!(learning.draft.utility.as_ref().unwrap().enabled);
    assert!(!learning.checked);
    assert!(learning.checks(lesson).is_empty());
    let bytes = serde_json::to_vec(&learning).unwrap();
    let restored: dns_teaching::Learning = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    assert!(restored.valid());
    assert!(!engine::solved(&s.initial, &s.goal));
    let mut invalid = restored.draft;
    invalid.utility.as_mut().unwrap().input = "x".repeat(513);
    assert!(!invalid.valid());
    assert!(serde_json::from_value::<Draft>(serde_json::to_value(invalid).unwrap()).is_err());
}
#[test]
fn optin_endpoint_metadata_is_validated_and_never_changes_goal_or_initial_state() {
    let original = scenario();
    let mut s = original.clone();
    let lesson = s.dns_teaching.as_mut().unwrap();
    lesson.lookup = Some(q::Lesson {
        assignment: plan(),
        example: plan(),
    });
    lesson.validate().unwrap();
    let raw = serde_json::json!({"schema_version":1,"scenarios":[s]});
    let loaded = pack::load(&raw.to_string()).unwrap().remove(0);
    assert_eq!(
        serde_json::to_value(loaded.initial).unwrap(),
        serde_json::to_value(&original.initial).unwrap()
    );
    assert_eq!(
        serde_json::to_value(loaded.goal).unwrap(),
        serde_json::to_value(&original.goal).unwrap()
    );
    let mut bad = raw;
    bad["scenarios"][0]["dns_teaching"]["lookup"]["assignment"]["endpoints"][0]["served"] =
        serde_json::json!("Configured");
    assert!(pack::load(&bad.to_string()).is_err());
    bad["scenarios"][0]["dns_teaching"]["lookup"]["assignment"]["endpoints"][0]["served"] =
        serde_json::json!("Primary");
    bad["scenarios"][0]["dns_teaching"]["lookup"]["assignment"]["endpoints"][0]["address"] =
        serde_json::json!("1.1.1.1");
    assert!(pack::load(&bad.to_string()).is_err());
}
