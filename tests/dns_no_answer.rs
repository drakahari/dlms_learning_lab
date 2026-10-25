//! Current, selected-copy negative queries; never contact a host resolver.
use dlms_learning_lab::{
    dns::{self, Code, Data, QueryType, Record, Server, Transfer, Zone},
    dns_workspace::{self as editor, Requirement, Snapshot},
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};

const ORIGIN: &str = "2.0.192.in-addr.arpa.";
const OLD: &str = "36.2.0.192.in-addr.arpa.";
const LIVE: &str = "83.2.0.192.in-addr.arpa.";
fn record(owner: &str, data: Data) -> Record {
    Record {
        owner: owner.into(),
        ttl: 300,
        data,
    }
}
fn ptr(owner: &str) -> Record {
    record(
        owner,
        Data::Ptr {
            target: "service.example.test.".into(),
        },
    )
}
fn alias(owner: &str, target: &str) -> Record {
    record(
        owner,
        Data::Cname {
            target: target.into(),
        },
    )
}
fn server() -> Server {
    Server::new(Zone {
        origin: ORIGIN.into(),
        serial: 20,
        records: vec![
            record(
                ORIGIN,
                Data::Ns {
                    target: "ns.example.test.".into(),
                },
            ),
            ptr(LIVE),
        ],
    })
    .unwrap()
}
fn absent(copy: Snapshot, name: &str, kind: QueryType) -> Requirement {
    Requirement::NoAnswer {
        copy,
        name: name.into(),
        record_type: kind,
    }
}
fn goals() -> Vec<Requirement> {
    vec![
        absent(Snapshot::Primary, OLD, QueryType::Ptr),
        absent(Snapshot::Secondary, OLD, QueryType::Ptr),
        Requirement::Resolves {
            copy: Snapshot::Primary,
            name: LIVE.into(),
            record_type: QueryType::Ptr,
            data: vec![Data::Ptr {
                target: "service.example.test.".into(),
            }],
        },
        Requirement::Published { secondary: true },
    ]
}

#[test]
fn authoritative_missing_name_and_existing_name_without_type_both_pass() {
    let s = server();
    for copy in [Snapshot::Primary, Snapshot::Secondary] {
        assert!(absent(copy, OLD, QueryType::Ptr).check(&s).0);
        assert!(absent(copy, LIVE, QueryType::A).check(&s).0);
        assert!(!absent(copy, LIVE, QueryType::Ptr).check(&s).0);
        // An external PTR target is returned data, not an unresolved walk.
        assert!(absent(copy, LIVE, QueryType::Cname).check(&s).0);
    }
    assert_eq!(
        s.primary.query(OLD, QueryType::Ptr).unwrap().code,
        Code::Nxdomain
    );
    assert_eq!(
        s.primary.query(LIVE, QueryType::A).unwrap().code,
        Code::Noerror
    );
}

#[test]
fn internal_aliases_may_end_at_missing_name_or_missing_type_but_not_ptr() {
    for target in ["missing.2.0.192.in-addr.arpa.", LIVE] {
        let mut s = server();
        s.configured.records.push(alias(OLD, target));
        s.reload_primary().unwrap();
        assert!(absent(Snapshot::Primary, OLD, QueryType::A).check(&s).0);
        assert_eq!(
            absent(Snapshot::Primary, OLD, QueryType::Ptr).check(&s).0,
            target != LIVE
        );
        assert!(!absent(Snapshot::Primary, OLD, QueryType::Cname).check(&s).0);
    }
}

#[test]
fn any_walked_external_alias_target_is_incomplete_not_negative_evidence() {
    for external in ["elsewhere.example.test.", "2.0.192.in-addr.arpa.evil.test."] {
        let mut s = server();
        let middle = "37.2.0.192.in-addr.arpa.";
        s.configured
            .records
            .extend([alias(OLD, middle), alias(middle, external)]);
        s.reload_primary().unwrap();
        let answer = s.primary.query(OLD, QueryType::Ptr).unwrap();
        assert!(answer.authoritative && answer.code == Code::Noerror);
        assert_eq!(answer.records.len(), 2);
        let (pass, feedback) = absent(Snapshot::Primary, OLD, QueryType::Ptr).check(&s);
        assert!(!pass);
        assert!(feedback.contains("outside this zone"));
        assert!(!absent(Snapshot::Primary, OLD, QueryType::A).check(&s).0);
    }
}

#[test]
fn loops_and_excessive_alias_walks_never_count_as_absence() {
    let mut looped = server();
    looped.configured.records.extend([
        alias(OLD, "37.2.0.192.in-addr.arpa."),
        alias("37.2.0.192.in-addr.arpa.", OLD),
    ]);
    looped.reload_primary().unwrap();
    assert_eq!(
        looped.primary.query(OLD, QueryType::Ptr).unwrap().code,
        Code::Servfail
    );
    assert!(
        !absent(Snapshot::Primary, OLD, QueryType::Ptr)
            .check(&looped)
            .0
    );
    let mut long = server();
    let mut owner = OLD.to_owned();
    for i in 0..=dns::MAX_ALIASES {
        let target = format!("hop{i}.2.0.192.in-addr.arpa.");
        long.configured.records.push(alias(&owner, &target));
        owner = target;
    }
    long.reload_primary().unwrap();
    assert_eq!(
        long.primary.query(OLD, QueryType::Ptr).unwrap().code,
        Code::Servfail
    );
    assert!(
        !absent(Snapshot::Primary, OLD, QueryType::Ptr)
            .check(&long)
            .0
    );
}

#[test]
fn validation_rejects_configured_copy_outside_authority_and_malformed_context() {
    let s = server();
    for req in [
        absent(Snapshot::Configured, OLD, QueryType::Ptr),
        absent(Snapshot::Primary, "x.example.test.", QueryType::Ptr),
        absent(Snapshot::Primary, "bad/name.", QueryType::Ptr),
        absent(Snapshot::Secondary, &"x".repeat(300), QueryType::Ptr),
    ] {
        assert!(!req.valid_for(&s));
        assert!(!req.check(&s).0);
    }
    let req = absent(Snapshot::Primary, OLD, QueryType::Ptr);
    let mut wrong_origin = s.clone();
    wrong_origin.secondary.origin = "other.test.".into();
    assert!(!req.check(&wrong_origin).0);
    let mut broken = s.clone();
    broken.primary.records.push(record(
        LIVE,
        Data::Cname {
            target: "elsewhere.test.".into(),
        },
    ));
    assert!(!req.check(&broken).0);
    assert!(
        !s.primary
            .query("x.example.test.", QueryType::Ptr)
            .unwrap()
            .authoritative
    );
}

#[test]
fn absence_is_current_per_copy_and_is_not_a_publication_credential() {
    let mut s = server();
    assert!(goals().iter().all(|r| r.check(&s).0));
    s.configured.records.push(ptr(OLD));
    // A saved edit does not change the selected serving copies.
    assert!(goals()[0].check(&s).0 && goals()[1].check(&s).0);
    assert!(!goals()[3].check(&s).0);
    s.reload_primary().unwrap();
    assert!(!goals()[0].check(&s).0 && goals()[1].check(&s).0);
    s.configured.records.retain(|r| r.owner != OLD);
    s.configured.serial += 1;
    s.reload_primary().unwrap();
    s.update_secondary().unwrap();
    assert!(goals().iter().all(|r| r.check(&s).0));
    // Retiring an old answer on only the primary leaves a stale secondary.
    s.secondary.records.push(ptr(OLD));
    assert!(goals()[0].check(&s).0 && !goals()[1].check(&s).0);
    assert!(!goals()[3].check(&s).0);
    assert_eq!(s.update_secondary().unwrap(), Transfer::UnchangedEqual);
    assert!(!goals()[1].check(&s).0);
    s.configured.serial += 1;
    s.reload_primary().unwrap();
    assert_eq!(s.update_secondary().unwrap(), Transfer::Updated);
    assert!(goals().iter().all(|r| r.check(&s).0));
}

#[test]
fn negative_rows_do_not_replace_positive_resolution_and_publication_guards() {
    let s = server();
    let rows = goals();
    assert!(editor::valid_requirements(&rows, &s));
    assert!(!editor::valid_requirements(&rows[..2], &s));
    assert!(!editor::valid_requirements(&rows[..3], &s));
    assert!(!editor::valid_requirements(
        &[rows[0].clone(), rows[3].clone()],
        &s
    ));
    let mut no_service = s.clone();
    no_service.configured.records.retain(|r| r.owner != LIVE);
    no_service.reload_primary().unwrap();
    assert!(rows[0].check(&no_service).0);
    assert!(!rows[2].check(&no_service).0);
}

#[test]
fn pack_reachability_accepts_retirement_and_rejects_invalid_negative_outcomes() {
    let mut lab = pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| matches!(s.initial, State::AuthoritativeDns { .. }))
        .unwrap();
    let mut initial = server();
    initial.configured.records.push(ptr(OLD));
    initial.reload_primary().unwrap();
    initial.secondary = initial.primary.clone();
    let mut final_zone = server().configured;
    final_zone.serial += 1;
    lab.id = "test-dns-negative-pack".into();
    lab.prerequisites.clear();
    lab.dns_teaching = None;
    lab.initial = State::AuthoritativeDns { server: initial };
    lab.goal = Goal::AuthoritativeDns {
        requirements: goals(),
    };
    lab.solution = vec![
        format!("zone {}", serde_json::to_string(&final_zone).unwrap()),
        "reload-primary".into(),
        "update-secondary".into(),
    ];
    let text =
        serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":[lab]})).unwrap();
    assert_eq!(pack::load(&text).unwrap().len(), 1);
    for bad in [
        absent(Snapshot::Configured, OLD, QueryType::Ptr),
        absent(Snapshot::Secondary, "outside.test.", QueryType::Ptr),
    ] {
        let mut value: serde_json::Value = serde_json::from_str(&text).unwrap();
        value["scenarios"][0]["goal"]["requirements"][0] = serde_json::to_value(bad).unwrap();
        assert!(pack::load(&serde_json::to_string(&value).unwrap()).is_err());
    }
}

#[test]
fn tagged_requirement_roundtrip_and_invalid_fields_keep_the_old_wire_contract() {
    let req = absent(Snapshot::Primary, OLD, QueryType::Ptr);
    let value = serde_json::to_value(&req).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"kind":"NoAnswer","copy":"Primary","name":OLD,"record_type":"PTR"})
    );
    let decoded: Requirement = serde_json::from_value(value.clone()).unwrap();
    assert!(decoded.check(&server()).0);
    for text in [
        r#"{"kind":"NoAnswer","copy":"Primary","name":"x.test."}"#,
        r#"{"kind":"NoAnswer","copy":"Primary","name":"x.test.","record_type":"ANY"}"#,
        r#"{"kind":"NoAnswer","copy":"Primary","name":"x.test.","record_type":"PTR","data":[]}"#,
    ] {
        assert!(serde_json::from_str::<Requirement>(text).is_err());
    }
    let old = Requirement::Published { secondary: true };
    assert_eq!(
        serde_json::to_value(old).unwrap(),
        serde_json::json!({"kind":"Published","secondary":true})
    );
    let before = serde_json::to_vec(&server()).unwrap();
    let stable = server();
    for _ in 0..3 {
        assert!(decoded.check(&stable).0);
    }
    assert_eq!(serde_json::to_vec(&stable).unwrap(), before);
}

#[test]
fn negative_goals_and_unfinished_query_drafts_resume_with_pinned_origin() {
    let mut labs = pack::bundled().unwrap();
    let mut lab = labs
        .iter()
        .find(|s| matches!(s.initial, State::AuthoritativeDns { .. }))
        .unwrap()
        .clone();
    lab.id = "test-dns-negative-capability".into();
    lab.initial = State::AuthoritativeDns { server: server() };
    lab.goal = Goal::AuthoritativeDns {
        requirements: goals(),
    };
    lab.dns_teaching = None;
    let mut a = Attempt::new(&lab, 1);
    workspace::prepare(&mut a);
    assert!(progress::current_solved(&a, &lab));
    let d = a.workspace.as_mut().unwrap().dns_draft.as_mut().unwrap();
    d.query_name = "unfinished query /".into();
    d.query_type = QueryType::Ptr;
    d.query_copy = Snapshot::Secondary;
    d.text.push_str("\nunfinished record");
    assert!(!progress::current_solved(&a, &lab));
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), a);
    labs.push(lab.clone());
    let bytes = serde_json::to_vec(&p).unwrap();
    let restored = progress::decode(&bytes, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&restored.attempts[&lab.id]).unwrap(),
        serde_json::to_value(&p.attempts[&lab.id]).unwrap()
    );
    assert!(!progress::current_solved(&restored.attempts[&lab.id], &lab));
    let mut wrong_context = serde_json::to_value(&p).unwrap();
    wrong_context["attempts"][&lab.id]["state"]["server"]["configured"]["origin"] =
        "wrong.test.".into();
    assert!(progress::decode(&serde_json::to_vec(&wrong_context).unwrap(), &labs).is_err());
}

#[test]
fn canonical_in_zone_names_have_bounded_feedback_without_dumping_records() {
    let mut s = server();
    let name = format!(
        "{}.{}.{}.{}.",
        "a".repeat(63),
        "b".repeat(63),
        "c".repeat(63),
        "d".repeat(61)
    );
    // Use the maximum supported name as the origin; it is not an IP address.
    s.configured.origin = name.clone();
    s.configured.records = vec![record(
        &name,
        Data::Ns {
            target: "ns.example.test.".into(),
        },
    )];
    s.primary = s.configured.clone();
    s.secondary = s.configured.clone();
    let req = absent(Snapshot::Primary, &name.to_uppercase(), QueryType::Ptr);
    assert!(req.valid_for(&s));
    let (pass, feedback) = req.check(&s);
    assert!(pass);
    assert!(feedback.len() <= 512);
    assert!(!feedback.contains("ns.example.test"));
}
