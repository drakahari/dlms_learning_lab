//! Authored L2 LACP assignments; expected outcome vectors are independently reasoned.
use dlms_learning_lab::{
    engine::{Goal, State},
    etherchannel as core, etherchannel_teaching as learn, etherchannel_workspace as work, pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
};
use serde::Deserialize;
const IDS: [&str; 3] = [
    "ccna-lacp-initiation",
    "ccna-lacp-member-service",
    "ccna-lacp-resilient-handover",
];
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Review {
    id: String,
    reason: String,
    reference: Vec<core::Action>,
    alternatives: Vec<Vec<core::Action>>,
    wrong: Vec<core::Action>,
    wrong_expected: Vec<bool>,
    partial: Vec<core::Action>,
    partial_expected: Vec<bool>,
    initial_expected: Vec<bool>,
    adverse: Vec<core::Action>,
    adverse_expected: Vec<bool>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reviews {
    schema_version: u32,
    origin: String,
    lessons: Vec<Review>,
}
fn sources() -> Vec<pack::Scenario> {
    pack::load_catalog(&[
        include_str!("../scenarios/starter-v1.json"),
        include_str!("../scenarios/vlan-data-v1.json"),
        include_str!("../scenarios/etherchannel-v1.json"),
    ])
    .unwrap()
    .into_iter()
    .filter(|s| IDS.contains(&s.id.as_str()))
    .collect()
}
fn reviews() -> Vec<Review> {
    let r: Reviews =
        serde_json::from_str(include_str!("fixtures/etherchannel-content-review.json")).unwrap();
    assert_eq!(r.schema_version, 1);
    assert!(r.origin.contains("not recorded"));
    assert_eq!(r.lessons.len(), 3);
    r.lessons
}
fn simulation(a: &Attempt) -> &core::State {
    let State::EtherChannel { simulation } = &a.state else {
        panic!("Actual typed LACP state")
    };
    simulation
}
fn prepared(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    learn::prepare(&mut a, s, true).unwrap();
    a
}
fn apply(a: &mut Attempt, s: &pack::Scenario, actions: &[core::Action]) {
    for core::Action::Configure { configuration } in actions {
        let draft =
            work::Draft::from_configuration(configuration, simulation(a).topology()).unwrap();
        work::set_assignment_draft(a, s, &draft).unwrap();
        work::apply_assignment(a, s).unwrap();
    }
}
fn vector(a: &Attempt, s: &pack::Scenario) -> Vec<bool> {
    progress::current_etherchannel_checks(a, s)
        .iter()
        .map(|r| r.0)
        .collect()
}
fn exact<T: serde::Serialize + ?Sized>(a: &T, b: &T) {
    assert_eq!(
        serde_json::to_value(a).unwrap(),
        serde_json::to_value(b).unwrap()
    );
}
#[test]
fn three_distinct_assignments_have_honest_prerequisites_teaching_and_positive_anchors() {
    let labs = sources();
    assert_eq!(labs.len(), 3);
    for (n, (s, r)) in labs.iter().zip(reviews()).enumerate() {
        assert_eq!(s.id, IDS[n]);
        assert_eq!(r.id, s.id);
        assert!(!r.reason.is_empty());
        assert_eq!(s.unit, "ccna-lacp");
        assert_eq!(
            s.prerequisites,
            if n == 0 {
                vec!["ccna-vlan-trunk".to_owned()]
            } else {
                vec![IDS[n - 1].to_owned()]
            }
        );
        let Goal::EtherChannel { definition } = &s.goal else {
            panic!()
        };
        definition.validate().unwrap();
        let l = s.etherchannel_teaching.as_ref().unwrap();
        l.validate().unwrap();
        assert!(l.separate_from(&definition.initial));
        assert!(
            l.concepts
                .iter()
                .any(|c| c.text.contains("Active") && c.text.contains("Passive"))
        );
        assert!(
            l.concepts
                .iter()
                .any(|c| c.text.contains("Apply configuration")
                    && c.text.contains("Check configuration"))
        );
        assert!(s.simplifications.contains("routed EtherChannel"));
        assert!(
            s.sources
                .iter()
                .any(|s| s.contains("200-301-CCNA-v1.1.pdf"))
        );
        assert_eq!(
            definition
                .initial
                .topology()
                .devices
                .values()
                .map(|d| d.ports.len())
                .max(),
            Some(if n < 2 { 4 } else { 6 })
        );
        let pairs = definition
            .requirements
            .iter()
            .filter(|r| matches!(r, work::Requirement::BundlePair { .. }))
            .count();
        assert_eq!(pairs, if n < 2 { 1 } else { 5 });
        assert_eq!(r.alternatives.len(), 2);
        assert_eq!(r.initial_expected.len(), definition.row_count());
    }
}
#[test]
fn every_reference_alternate_wrong_partial_and_after_pass_break_matches_literal_outcomes() {
    for (s, r) in sources().iter().zip(reviews()) {
        let mut a = prepared(s);
        assert_eq!(vector(&a, s), r.initial_expected);
        assert!(!progress::current_solved(&a, s));
        for (actions, expected) in [
            (&r.wrong, &r.wrong_expected),
            (&r.partial, &r.partial_expected),
        ] {
            let mut a = prepared(s);
            apply(&mut a, s, actions);
            assert_eq!(&vector(&a, s), expected);
            assert!(expected.iter().any(|v| !*v));
            assert!(expected.iter().any(|v| *v));
            assert!(!progress::current_solved(&a, s));
        }
        for actions in std::iter::once(&r.reference).chain(&r.alternatives) {
            let mut solved = prepared(s);
            apply(&mut solved, s, actions);
            assert!(progress::current_solved(&solved, s));
            let before = serde_json::to_value(&solved).unwrap();
            assert!(
                progress::current_etherchannel_checks(&solved, s)
                    .iter()
                    .all(|r| r.0 && r.1.len() <= 512)
            );
            assert_eq!(serde_json::to_value(&solved).unwrap(), before);
        }
        apply(&mut a, s, &r.reference);
        work::set_saved_check(&mut a, s, true).unwrap();
        assert!(progress::current_solved(&a, s));
        apply(&mut a, s, &r.adverse);
        assert_eq!(vector(&a, s), r.adverse_expected);
        assert!(!progress::current_solved(&a, s));
        apply(&mut a, s, &r.reference);
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn drafts_practice_applied_changes_and_validated_resume_remain_distinct() {
    let all = sources();
    for (s, r) in all.iter().zip(reviews()) {
        let mut a = prepared(s);
        let initial = a.clone();
        work::set_practice_presentation(&mut a, s, Some(View::Example), Some(0), None).unwrap();
        work::fill_practice(&mut a, s).unwrap();
        exact(&a.state, &initial.state);
        assert!(!progress::current_solved(&a, s));
        work::apply_practice(&mut a, s).unwrap();
        work::set_practice_presentation(&mut a, s, None, None, Some(true)).unwrap();
        let l = a
            .workspace
            .as_ref()
            .unwrap()
            .etherchannel_learning
            .as_ref()
            .unwrap();
        assert!(
            l.checks(s.etherchannel_teaching.as_ref().unwrap())
                .iter()
                .all(|r| r.0)
        );
        exact(&a.state, &initial.state);
        assert!(!progress::current_solved(&a, s));
        let practice = l.clone();
        apply(&mut a, s, &r.reference);
        assert!(progress::current_solved(&a, s));
        exact(
            a.workspace
                .as_ref()
                .unwrap()
                .etherchannel_learning
                .as_ref()
                .unwrap(),
            &practice,
        );
        let mut pending = a
            .workspace
            .as_ref()
            .unwrap()
            .etherchannel_draft
            .as_ref()
            .unwrap()
            .clone();
        let first = pending
            .devices
            .values_mut()
            .next()
            .unwrap()
            .ports
            .values_mut()
            .next()
            .unwrap();
        first.up = false;
        let applied = a.state.clone();
        work::set_assignment_draft(&mut a, s, &pending).unwrap();
        exact(&a.state, &applied);
        assert!(!progress::current_solved(&a, s));
        let mut p = Progress {
            schema_version: 1,
            clock: 1,
            ..Default::default()
        };
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a.clone());
        let raw = serde_json::to_vec(&p).unwrap();
        let resumed = progress::decode(&raw, &all).unwrap();
        exact(&resumed.attempts[&s.id], &a);
        assert!(resumed.favorites.contains(&s.id));
        work::reset_practice(&mut a, s).unwrap();
        exact(&a.state, &applied);
        assert_eq!(
            a.workspace
                .as_ref()
                .unwrap()
                .etherchannel_draft
                .as_ref()
                .unwrap(),
            &pending
        );
        let reset = prepared(s);
        exact(&reset.state, &initial.state);
        assert!(!progress::current_solved(&reset, s));
    }
}
#[test]
fn copied_answers_changed_topology_and_missing_vlans_cannot_earn_credit() {
    let all = sources();
    for (s, r) in all.iter().zip(reviews()) {
        let mut a = prepared(s);
        apply(&mut a, s, &r.reference);
        assert!(progress::current_solved(&a, s));
        let mut changed = s.clone();
        if let Goal::EtherChannel { definition } = &mut changed.goal {
            let mut raw = serde_json::to_value(&definition.initial).unwrap();
            raw["topology"]["fixture"] = "different-supplied-site".into();
            definition.initial = serde_json::from_value(raw).unwrap();
        }
        assert!(!progress::current_solved(&a, &changed));
        // A caller's configured state is never authority for a different assignment.
        assert!(!progress::current_solved(
            &a,
            &all[(IDS.iter().position(|id| *id == s.id).unwrap() + 1) % 3]
        ));
        let before = serde_json::to_value(&a).unwrap();
        let mut missing = a.clone();
        let mut config = simulation(&missing).configuration().clone();
        config
            .devices
            .values_mut()
            .next()
            .unwrap()
            .vlans
            .remove(&20);
        apply(
            &mut missing,
            s,
            &[core::Action::Configure {
                configuration: config,
            }],
        );
        assert!(!progress::current_solved(&missing, s));
        assert!(
            progress::current_etherchannel_checks(&missing, s)
                .iter()
                .any(|r| !r.0 && r.1.contains("VLAN 20"))
        );
        let mut foreign = a.clone();
        foreign.workspace.as_mut().unwrap().nfs_session = Some(Default::default());
        assert!(!progress::current_solved(&foreign, s));
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
    }
}
