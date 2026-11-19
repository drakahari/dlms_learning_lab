//! Current review-catalog integration; no historical learner answers are invented.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    hsrp_workspace as work,
    pack::{self, Pack},
    progress::{self, Attempt, Progress},
};
use serde_json::{Value, json};
fn value<T: serde::Serialize + ?Sized>(x: &T) -> Value {
    serde_json::to_value(x).unwrap()
}
const IDS: [&str; 4] = [
    "ccna-hsrp-virtual-gateway",
    "ccna-hsrp-incumbent-return",
    "ccna-hsrp-wan-tracking",
    "ccna-hsrp-resilient-handover",
];
fn accepted() -> Vec<pack::Scenario> {
    let mut labs = pack::bundled().unwrap();
    labs.extend(
        serde_json::from_str::<Pack>(include_str!("../scenarios/ccna-magic-number-v1.json"))
            .unwrap()
            .scenarios,
    );
    assert_eq!(labs.len(), 205);
    labs
}
#[test]
fn gateway_unit_appends_four_complete_sources_preserving_every_accepted_lab_and_unit() {
    let old = accepted();
    let all = pack::accepted209().unwrap();
    assert_eq!(all.len(), 209);
    assert_eq!(value(&all[..205]), value(&old));
    let proposed =
        serde_json::from_str::<Pack>(include_str!("../scenarios/hsrp-resilient-gateway-v1.json"))
            .unwrap();
    assert_eq!(value(&all[205..]), value(&proposed.scenarios));
    assert_eq!(
        all[205..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    let mut old_units: Value =
        serde_json::from_str(include_str!("../scenarios/ccna-magic-units-v1.json")).unwrap();
    let baseline: Value = serde_json::from_str(include_str!("../scenarios/units-v1.json")).unwrap();
    old_units["units"]
        .as_array_mut()
        .unwrap()
        .extend(baseline["units"].as_array().unwrap().clone());
    let old_units = course::load(&old_units.to_string(), &old).unwrap();
    let units = course::accepted209(&all).unwrap();
    assert_eq!(units.len(), 56);
    assert_eq!(value(&units[..55]), value(&old_units));
    let recipe: Value =
        serde_json::from_str(include_str!("fixtures/hsrp-resilient-gateway-review.json")).unwrap();
    assert_eq!(value(&units[55]), recipe["unit_proposal"]);
    assert_eq!(units[55].prerequisites, ["ipv4-foundations"]);
    for s in &all[205..] {
        assert_eq!(s.prerequisites[..2], ["ccna-ipv4", "ccna-gateway"]);
        assert!(!engine::solved(&s.initial, &s.goal));
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [65, 44, 36, 64]
    );
}
#[test]
fn gateway_unit_current_catalog_save_reopen_reset_keeps_prior_work_and_new_unfinished_drafts() {
    let all = pack::accepted209().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for (n, s) in all[..205].iter().enumerate() {
        let mut a = Attempt::new(s, n as u64 + 1);
        dlms_learning_lab::workspace::prepare(&mut a);
        p.attempts.insert(s.id.clone(), a);
    }
    p.clock = 205;
    p.favorites.extend([all[0].id.clone(), IDS[0].into()]);
    let old = value(&p.attempts);
    let s = &all[205];
    let mut a = Attempt::new(s, 206);
    work::binding::prepare(&mut a, s).unwrap();
    let State::Hsrp { simulation } = &a.state else {
        panic!()
    };
    let mut draft = simulation.draft.clone();
    draft.clients[0].gateway = "192.0.".into();
    work::binding::change(
        &mut a,
        s,
        work::binding::Change::Draft {
            draft,
            practice: false,
        },
    )
    .unwrap();
    p.attempts.insert(s.id.clone(), a);
    p.clock = 206;
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("progress-v1.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let reopened = progress::load(&path, &all).unwrap();
    assert_eq!(value(&reopened), value(&p));
    assert!(!progress::current_solved(&reopened.attempts[&s.id], s));
    let mut reset = reopened;
    reset.attempts.insert(s.id.clone(), Attempt::new(s, 207));
    reset.attempts.remove(&s.id);
    assert_eq!(value(&reset.attempts), old);
    assert_eq!(reset.favorites, p.favorites);
}
#[test]
fn gateway_unit_registered_solutions_grade_outcomes_and_preserve_actual_incumbency_on_reopen() {
    let all = pack::accepted209().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for (n, s) in all[205..].iter().enumerate() {
        let mut a = Attempt::new(s, n as u64 + 1);
        work::binding::prepare(&mut a, s).unwrap();
        assert!(!progress::current_solved(&a, s));
        for command in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
        }
        work::binding::change(&mut a, s, work::binding::Change::Check { practice: false }).unwrap();
        assert!(progress::current_solved(&a, s));
        p.attempts.insert(s.id.clone(), a);
    }
    p.clock = 4;
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("progress-v1.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let mut reread = progress::load(&path, &all).unwrap();
    assert_eq!(value(&reread), value(&p));
    for s in &all[205..] {
        let a = reread.attempts.get_mut(&s.id).unwrap();
        assert!(progress::current_solved(a, s));
        let (Goal::Hsrp { definition }, State::Hsrp { simulation }) = (&s.goal, &a.state) else {
            panic!()
        };
        let mut draft = simulation.draft.clone();
        let editable = definition.policy.editable_clients.first().copied();
        if let Some(id) = editable {
            let c = draft.clients.iter_mut().find(|c| c.id == id).unwrap();
            c.gateway = definition
                .policy
                .initial
                .router(dlms_learning_lab::hsrp::RouterId::A)
                .address
                .to_string();
        } else {
            draft.routers[0].preempt = false;
        }
        work::binding::change(
            a,
            s,
            work::binding::Change::Draft {
                draft,
                practice: false,
            },
        )
        .unwrap();
        assert!(!progress::current_solved(a, s));
        work::binding::change(a, s, work::binding::Change::Apply { practice: false }).unwrap();
        assert!(!progress::current_solved(a, s));
        work::binding::change(a, s, work::binding::Change::Check { practice: false }).unwrap();
        assert!(!progress::current_solved(a, s));
    }
    assert_ne!(json!(reread.attempts), json!(p.attempts));
}
