//! Three unregistered outcomes, not host NAT/IOS or action-history grading.
use dlms_learning_lab::{
    backup,
    engine::{Goal, State},
    nat::{self, Action, Configuration},
    nat_teaching::{self as teaching, Learning},
    nat_workspace::{self as work, Draft},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};
use serde::Deserialize;
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicUsize},
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pending {
    statics: Option<Vec<work::StaticRow>>,
    network: Option<String>,
    prefix: Option<String>,
    first: Option<String>,
    last: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Candidate {
    name: String,
    actions: Vec<Action>,
    expected: Vec<bool>,
    pending: Option<Pending>,
    error_contains: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Review {
    id: String,
    reference: Vec<Action>,
    alternatives: Vec<Candidate>,
    wrong: Vec<Candidate>,
    partial: Candidate,
    pending: Candidate,
    broken: Candidate,
    misconception: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Outcomes {
    id: String,
    initial: Vec<bool>,
    reference: Vec<bool>,
    experiment: Vec<bool>,
    row_order: Vec<String>,
}
fn labs() -> Vec<Scenario> {
    static ALL: OnceLock<Vec<Scenario>> = OnceLock::new();
    ALL.get_or_init(|| {
        let text = include_str!("../scenarios/nat-basic-v1.json");
        let proposed: pack::Pack = serde_json::from_str(text).unwrap();
        let ids: std::collections::BTreeSet<_> =
            proposed.scenarios.iter().map(|s| s.id.clone()).collect();
        let prior: Vec<_> = pack::bundled()
            .unwrap()
            .into_iter()
            .filter(|s| !ids.contains(&s.id))
            .collect();
        let mut chunks: Vec<String> = prior
            .chunks(8)
            .map(|group| {
                serde_json::to_string(&pack::Pack {
                    schema_version: 1,
                    scenarios: group.to_vec(),
                })
                .unwrap()
            })
            .collect();
        chunks.push(text.into());
        pack::load_catalog(&chunks.iter().map(String::as_str).collect::<Vec<_>>())
            .unwrap()
            .into_iter()
            .filter(|s| ids.contains(&s.id))
            .collect()
    })
    .clone()
}
fn reviews() -> Vec<Review> {
    serde_json::from_str(include_str!("fixtures/nat-basic-review.json")).unwrap()
}
fn outcomes() -> Vec<Outcomes> {
    serde_json::from_str(include_str!("fixtures/nat-basic-outcomes.json")).unwrap()
}
fn simulation(s: &State) -> &nat::State {
    let State::BasicNat { simulation } = s else {
        panic!()
    };
    simulation
}
fn initial(s: &Scenario) -> &nat::State {
    simulation(&s.initial)
}
fn def(s: &Scenario) -> &work::Definition {
    let Goal::BasicNat { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn attempt(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 9);
    workspace::prepare(&mut a);
    teaching::prepare(&mut a, s.nat_teaching.as_ref().unwrap(), true).unwrap();
    a
}
#[track_caller]
fn exact(a: &impl serde::Serialize, b: &impl serde::Serialize) {
    assert!(
        serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap(),
        "Complete bounded value differs (no field projection)"
    );
}
fn fill(d: &mut Draft, c: &Configuration) {
    d.statics = c
        .statics
        .iter()
        .map(|(i, g)| work::StaticRow {
            inside: i.clone(),
            global: g.to_string(),
        })
        .collect();
    d.dynamic_enabled = c.dynamic.is_some();
    if let Some(r) = c.dynamic {
        d.network = r.source.network.to_string();
        d.prefix = r.source.length.to_string();
        d.first = r.pool.first.to_string();
        d.last = r.pool.last.to_string();
    }
}
fn act(a: &mut Attempt, s: &Scenario, action: &Action) -> Result<(), String> {
    let State::BasicNat { simulation } = &mut a.state else {
        panic!()
    };
    let d = a.workspace.as_mut().unwrap().nat_draft.as_mut().unwrap();
    if let Action::Configure { configuration } = action {
        fill(d, configuration);
        d.apply(initial(s), simulation)?;
    } else {
        work::perform(initial(s), simulation, action)?;
    }
    Ok(())
}
fn actions(a: &mut Attempt, s: &Scenario, items: &[Action]) -> Result<(), String> {
    for item in items {
        act(a, s, item)?;
    }
    Ok(())
}
fn pending(a: &mut Attempt, p: &Pending) {
    let d = a.workspace.as_mut().unwrap().nat_draft.as_mut().unwrap();
    if let Some(v) = &p.statics {
        d.statics = v.clone();
    }
    for (dst, value) in [
        (&mut d.network, &p.network),
        (&mut d.prefix, &p.prefix),
        (&mut d.first, &p.first),
        (&mut d.last, &p.last),
    ] {
        if let Some(v) = value {
            *dst = v.clone();
        }
    }
}
fn rows(a: &Attempt, s: &Scenario) -> Vec<bool> {
    progress::current_nat_checks(a, s)
        .iter()
        .map(|r| r.0)
        .collect()
}
fn candidate(s: &Scenario, c: &Candidate, a: &mut Attempt) {
    let before = a.state.clone();
    let result = actions(a, s, &c.actions);
    if let Some(expected) = &c.error_contains {
        let e = result.expect_err("Declared atomic rejection");
        assert!(e.contains(expected), "{}: {e}", c.name);
        assert_eq!(a.state, before, "{} failed mutation was not atomic", c.name);
    } else {
        result.unwrap_or_else(|e| panic!("{}: {e}", c.name));
    }
    if let Some(p) = &c.pending {
        pending(a, p);
    }
    assert_eq!(rows(a, s), c.expected, "{}: {}", s.id, c.name);
}
fn practice_action(l: &mut Learning, action: &Action) -> Result<(), String> {
    if let Action::Configure { configuration } = action {
        fill(&mut l.draft, configuration);
        l.draft.apply(&l.fixture, &mut l.simulation)?;
    } else {
        work::perform(&l.fixture, &mut l.simulation, action)?;
    }
    Ok(())
}
#[test]
fn normal_catalog_context_prerequisite_chain_and_source_shape() {
    let all = labs();
    assert_eq!(all.len(), 3);
    let normal = pack::bundled().unwrap();
    assert_eq!(normal.len(), 171);
    assert!(
        all.iter()
            .all(|s| normal.iter().any(|current| current.id == s.id))
    );
    assert_eq!(
        all[0].prerequisites,
        ["ccna-ipv4", "ccna-vlsm", "ccna-gateway"]
    );
    assert_eq!(all[1].prerequisites, [all[0].id.clone()]);
    assert_eq!(all[2].prerequisites, [all[1].id.clone()]);
    for (s, r, t) in all
        .iter()
        .zip(reviews())
        .zip(outcomes())
        .map(|((s, r), t)| (s, r, t))
    {
        assert_eq!(s.id, r.id);
        assert_eq!(s.id, t.id);
        assert_eq!(
            t.row_order.len(),
            1 + def(s).requirements.len() + def(s).trials.len()
        );
        assert_eq!(r.alternatives.len(), 2);
        assert_eq!(r.wrong.len(), 4);
        assert!(!r.misconception.trim().is_empty());
        assert!(s.guide.iter().all(|g| g.commands.is_empty()));
        assert!(s.nat_teaching.as_ref().unwrap().separate_from(initial(s)));
        assert_eq!(rows(&attempt(s), s), t.initial);
        assert!(t.initial.iter().any(|v| !*v));
        assert_eq!(
            s.solution
                .iter()
                .map(
                    |x| Action::from_json(x.strip_prefix(work::PREFIX).unwrap().as_bytes())
                        .unwrap()
                )
                .collect::<Vec<_>>(),
            r.reference
        );
    }
}
#[test]
fn real_reference_and_two_independent_alternatives_per_task_pass_current_and_new_requests() {
    for (s, r, t) in labs()
        .iter()
        .zip(reviews())
        .zip(outcomes())
        .map(|((s, r), t)| (s, r, t))
    {
        let mut a = attempt(s);
        actions(&mut a, s, &r.reference).unwrap();
        assert_eq!(rows(&a, s), t.reference);
        assert!(progress::current_solved(&a, s));
        let mut bridge = s.initial.clone();
        for command in &s.solution {
            dlms_learning_lab::engine::execute_for_goal(&mut bridge, &s.goal, command).unwrap();
        }
        assert_eq!(bridge, a.state);
        for c in &r.alternatives {
            assert_ne!(c.actions, r.reference);
            let mut alt = attempt(s);
            candidate(s, c, &mut alt);
            assert!(progress::current_solved(&alt, s));
        }
    }
}
#[test]
fn misconceptions_partial_pending_and_noop_assert_their_exact_failure_rows() {
    for (s, r, t) in labs()
        .iter()
        .zip(reviews())
        .zip(outcomes())
        .map(|((s, r), t)| (s, r, t))
    {
        assert_eq!(rows(&attempt(s), s), t.initial);
        for c in r.wrong.iter().chain([&r.partial, &r.pending]) {
            let mut a = attempt(s);
            candidate(s, c, &mut a);
            assert!(!progress::current_solved(&a, s));
            assert!(a.workspace.as_ref().unwrap().valid());
        }
        let mut malformed = attempt(s);
        let state = malformed.state.clone();
        let d = malformed
            .workspace
            .as_mut()
            .unwrap()
            .nat_draft
            .as_mut()
            .unwrap();
        if d.dynamic_enabled {
            d.prefix = "unfinished".into();
        } else {
            d.statics[0].global = "not-an-ip".into();
        }
        assert!(malformed.workspace.as_ref().unwrap().valid());
        assert_eq!(rows(&malformed, s), vec![false]);
        let State::BasicNat { simulation } = &mut malformed.state else {
            panic!()
        };
        assert!(
            malformed
                .workspace
                .as_ref()
                .unwrap()
                .nat_draft
                .as_ref()
                .unwrap()
                .apply(initial(s), simulation)
                .is_err()
        );
        assert_eq!(
            malformed.state, state,
            "Malformed Apply must preserve current State"
        );
    }
}
#[test]
fn static_only_reversal_policy_only_capacity_and_current_missing_have_distinct_truth() {
    let all = labs();
    let review = reviews();
    let mut static_a = attempt(&all[0]);
    act(&mut static_a, &all[0], &review[0].reference[0]).unwrap();
    assert!(progress::current_solved(&static_a, &all[0]));
    assert!(simulation(&static_a.state).bindings().is_empty());
    let before = static_a.state.clone();
    let mut observed = simulation(&static_a.state).clone();
    let effect = work::perform(
        initial(&all[0]),
        &mut observed,
        &Action::Inbound {
            peer: "remote".into(),
            global: "198.51.100.40".parse().unwrap(),
        },
    )
    .unwrap()
    .1;
    assert!(matches!(
        effect,
        nat::Effect::Probe {
            result: nat::ProbeResult::Translated {
                kind: nat::MappingKind::Static,
                ..
            }
        }
    ));
    assert_eq!(static_a.state, before);
    let mut policy = attempt(&all[1]);
    actions(&mut policy, &all[1], &review[1].reference).unwrap();
    assert!(progress::current_solved(&policy, &all[1]));
    assert!(simulation(&policy.state).bindings().is_empty());
    let static_shortcut = &review[1].wrong[3];
    let mut wrong = attempt(&all[1]);
    candidate(&all[1], static_shortcut, &mut wrong);
    let n = 1 + def(&all[1]).requirements.len();
    let q = rows(&wrong, &all[1]);
    assert!(q[..n].iter().all(|v| *v));
    assert!(q[n..].iter().all(|v| !*v));
    let mut handover = attempt(&all[2]);
    candidate(&all[2], &review[2].partial, &mut handover);
    let n = 1 + def(&all[2]).requirements.len();
    let q = rows(&handover, &all[2]);
    assert!(q[..n].iter().any(|v| !*v));
    assert!(q[n..].iter().all(|v| *v));
    assert!(simulation(&handover.state).bindings().is_empty());
    actions(&mut handover, &all[2], &review[2].reference[2..]).unwrap();
    assert!(progress::current_solved(&handover, &all[2]));
    assert_eq!(
        simulation(&handover.state).bindings()["alpha"],
        "198.51.100.70".parse::<std::net::Ipv4Addr>().unwrap(),
        "Former retired global may be reused"
    );
}
#[test]
fn every_annotated_example_insert_explicit_action_experiment_and_repair_works_separately() {
    for (s, t) in labs().iter().zip(outcomes()) {
        let lesson = s.nat_teaching.as_ref().unwrap();
        let mut a = attempt(s);
        let assignment = serde_json::to_value(&a).unwrap();
        let mut l = Learning::new(lesson).unwrap();
        for (n, step) in lesson.example_actions.iter().enumerate() {
            let before = l.simulation.clone();
            l.selected_action = n;
            l.fill_selected(lesson).unwrap();
            assert_eq!(l.simulation, before, "Insert is not execution");
            let action = match &step.action {
                Action::Configure { .. } => Action::Configure {
                    configuration: l.draft.configuration().unwrap(),
                },
                Action::Outbound { .. } => Action::Outbound {
                    inside: l.draft.inside.clone().unwrap(),
                    peer: l.draft.peer.clone().unwrap(),
                },
                Action::Inbound { .. } => Action::Inbound {
                    peer: l.draft.peer.clone().unwrap(),
                    global: l.draft.inbound_global.parse().unwrap(),
                },
                Action::ClearDynamic {} => Action::ClearDynamic {},
            };
            assert_eq!(action, step.action);
            practice_action(&mut l, &action).unwrap();
        }
        assert!(l.checks(lesson).iter().all(|r| r.0));
        for action in &lesson.try_change.actions {
            practice_action(&mut l, action).unwrap();
        }
        assert_eq!(
            l.checks(lesson).iter().map(|r| r.0).collect::<Vec<_>>(),
            t.experiment
        );
        for action in &lesson.repair {
            practice_action(&mut l, action).unwrap();
        }
        assert!(l.checks(lesson).iter().all(|r| r.0));
        exact(&a, &assignment);
        let mut expected = assignment;
        expected["workspace"]["nat_learning"] = serde_json::to_value(&l).unwrap();
        a.workspace.as_mut().unwrap().nat_learning = Some(l);
        exact(&a, &expected);
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn first_task_and_example_hardcodes_reject_other_contexts_without_changing_state_or_draft() {
    let all = labs();
    let ref_action = reviews()[0].reference[0].clone();
    let example_action = all[0].nat_teaching.as_ref().unwrap().example_actions[0]
        .action
        .clone();
    for s in &all[1..] {
        let mut a = attempt(s);
        for action in [&ref_action, &example_action] {
            let before = serde_json::to_value(&a).unwrap();
            let State::BasicNat { simulation } = &mut a.state else {
                panic!()
            };
            assert!(work::perform(initial(s), simulation, action).is_err());
            exact(&a, &before);
        }
        let l = a.workspace.as_mut().unwrap().nat_learning.as_mut().unwrap();
        let before = serde_json::to_value(&l).unwrap();
        assert!(work::perform(&l.fixture, &mut l.simulation, &example_action).is_err());
        exact(&l, &before);
    }
}
#[test]
fn breaking_current_state_or_policy_invalidates_then_reference_retry_restores_truth() {
    for (s, r) in labs().iter().zip(reviews()) {
        let mut a = attempt(s);
        actions(&mut a, s, &r.reference).unwrap();
        assert!(progress::current_solved(&a, s));
        candidate(s, &r.broken, &mut a);
        assert!(!progress::current_solved(&a, s));
        actions(&mut a, s, &r.reference).unwrap();
        assert!(progress::current_solved(&a, s));
        let l = a.workspace.as_mut().unwrap().nat_learning.as_mut().unwrap();
        l.draft.statics[0].global = "malformed practice".into();
        assert!(l.valid());
        assert!(
            progress::current_solved(&a, s),
            "Valid unsolved practice cannot revoke assignment completion"
        );
    }
}
#[test]
fn selected_practice_pending_drafts_backup_resume_reset_and_other_work_remain_exact() {
    let all = labs();
    let normal = pack::bundled().unwrap();
    let old = normal
        .iter()
        .find(|s| matches!(s.initial, State::Service { .. }))
        .unwrap_or(&normal[0])
        .clone();
    let other = Attempt::new(&old, 3);
    let mut p = Progress {
        schema_version: 1,
        clock: 77,
        ..Default::default()
    };
    p.attempts.insert(old.id.clone(), other.clone());
    for (s, r) in all.iter().zip(reviews()) {
        let mut a = attempt(s);
        actions(&mut a, s, &r.reference).unwrap();
        let lesson = s.nat_teaching.as_ref().unwrap();
        let l = a.workspace.as_mut().unwrap().nat_learning.as_mut().unwrap();
        for step in &lesson.example_actions {
            practice_action(l, &step.action).unwrap();
        }
        l.view = View::Example;
        l.checked = true;
        l.draft.statics[0].global = "unfinished practice".into();
        l.draft.inbound_global = "raw observation".into();
        a.workspace
            .as_mut()
            .unwrap()
            .nat_draft
            .as_mut()
            .unwrap()
            .inbound_global = "different assignment query".into();
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let mut catalog = normal;
    catalog.extend(all.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let loaded = progress::load(&path, &catalog).unwrap();
    exact(&loaded.attempts, &p.attempts);
    assert_eq!(loaded.clock, 77);
    assert_eq!(loaded.favorites, p.favorites);
    let backup_path = dir.path().join("backup.json");
    backup::export(&backup_path, &loaded, &catalog).unwrap();
    assert!(
        backup::inspect(
            &backup_path,
            &catalog,
            &AtomicBool::new(true),
            &AtomicUsize::new(0)
        )
        .is_err()
    );
    let candidate = backup::inspect(
        &backup_path,
        &catalog,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    exact(&candidate.progress.attempts, &p.attempts);
    let out = dir.path().join("restored.json");
    backup::restore(&out, &candidate, &catalog).unwrap();
    let mut restored = progress::load(&out, &catalog).unwrap();
    exact(&restored.attempts, &p.attempts);
    let expected = serde_json::to_value(&restored.attempts).unwrap();
    let selected = &all[0];
    restored
        .attempts
        .insert(selected.id.clone(), attempt(selected));
    for (id, a) in expected.as_object().unwrap() {
        if id != &selected.id {
            exact(&restored.attempts[id], a);
        }
    }
    assert_eq!(restored.favorites, p.favorites);
    assert!(!progress::current_solved(
        &restored.attempts[&selected.id],
        selected
    ));
    exact(&restored.attempts[&old.id], &other);
}
