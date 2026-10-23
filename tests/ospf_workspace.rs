//! Private native-adapter fixtures; no bundled OSPF curriculum or host protocol use.
use dlms_learning_lab::{
    backup,
    engine::{self, Goal, State},
    ospf::{self, Action, Endpoint, NextHop, State as Simulation},
    ospf_teaching::{self, Learning},
    ospf_workspace::{self as model, Draft, Requirement},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::{self, View},
    shell, workspace,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicBool, AtomicUsize},
};
fn scenario() -> Scenario {
    pack::load(include_str!("fixtures/ospf-workspace-preview.json"))
        .unwrap()
        .remove(0)
}
fn sim(state: &State) -> &Simulation {
    let State::Ospf { simulation } = state else {
        panic!()
    };
    simulation
}
fn sim_mut(state: &mut State) -> &mut Simulation {
    let State::Ospf { simulation } = state else {
        panic!()
    };
    simulation
}
fn goal(s: &Scenario) -> (&Simulation, &[Requirement]) {
    let Goal::Ospf {
        initial,
        requirements,
    } = &s.goal
    else {
        panic!()
    };
    (initial, requirements)
}
fn endpoint(r: &str, p: &str) -> Endpoint {
    Endpoint::new(r, p)
}
fn command(a: &Action) -> String {
    format!("ospf {}", serde_json::to_string(a).unwrap())
}
fn action(s: &Simulation, r: &str, p: &str, f: impl FnOnce(&mut ospf::PortConfig)) -> Action {
    let mut configuration = s.configuration().routers[r].interfaces[p].clone();
    f(&mut configuration);
    Action::SetPort {
        endpoint: endpoint(r, p),
        configuration,
    }
}
fn prepared(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    ospf_teaching::prepare(&mut a, s.ospf_teaching.as_ref().unwrap(), true).unwrap();
    a
}
fn learning(a: &mut Attempt) -> &mut Learning {
    a.workspace
        .as_mut()
        .unwrap()
        .ospf_learning
        .as_mut()
        .unwrap()
}
fn solved(s: &Scenario) -> Attempt {
    let mut a = prepared(s);
    let w = a.workspace.as_mut().unwrap();
    let draft = w.ospf_draft.as_mut().unwrap();
    draft
        .routers
        .get_mut("Core")
        .unwrap()
        .interfaces
        .get_mut("branch-core")
        .unwrap()
        .hello_seconds = "10".into();
    draft.apply(goal(s).0, sim_mut(&mut a.state)).unwrap();
    w.checked = true;
    assert!(progress::current_solved(&a, s));
    a
}
fn catalog() -> Vec<Scenario> {
    let mut all = pack::bundled().unwrap();
    all.push(scenario());
    all
}
fn profile(s: &Scenario, a: Attempt) -> Progress {
    Progress {
        schema_version: 1,
        attempts: BTreeMap::from([(s.id.clone(), a)]),
        ..Default::default()
    }
}
// Fixed pre-adapter catalog identities, including the four ACL and four account labs.
// Future catalog growth must not make this preservation regression silently omit them.
const ORIGINAL_98_IDS: [&str; 98] = [
    "ccna-ipv4",
    "ccna-vlsm",
    "ccna-gateway",
    "lpic1-permissions",
    "lpic1-traversal",
    "lpic1-logs",
    "lpic2-service",
    "lpic2-dns",
    "lpic2-storage",
    "shell-handover",
    "shell-arguments",
    "shell-input",
    "shell-validation",
    "shell-report",
    "shell-loop",
    "shell-capstone",
    "shell-streams",
    "shell-pipelines",
    "shell-eof",
    "shell-batch-audit",
    "shell-functions",
    "shell-return",
    "shell-shift",
    "shell-bundle",
    "lpic1-file-navigation",
    "lpic1-file-names",
    "lpic1-copy-move",
    "lpic1-file-cleanup",
    "lpic1-creation-mask",
    "lpic1-file-handover",
    "ccna-ipv6-neighbor",
    "ccna-ipv6-workstation",
    "ccna-ipv6-scope",
    "ccna-ipv6-specific-route",
    "ccna-ipv6-backup",
    "ccna-ipv6-handover",
    "shell-local-heading",
    "shell-scope-review",
    "shell-scope-capstone",
    "shell-cut-inventory",
    "shell-sort-manifest",
    "shell-uniq-counts",
    "shell-text-capstone",
    "shell-number-records",
    "shell-order-total",
    "shell-pack-items",
    "shell-count-capstone",
    "lpic1-team-brief",
    "lpic1-drop-directory",
    "lpic1-update-existing",
    "lpic1-retire-handover",
    "ccna-vlan-access",
    "ccna-vlan-trunk",
    "ccna-vlan-native",
    "ccna-vlan-handover",
    "shell-file-manifest",
    "shell-collect-errors",
    "shell-file-batch-capstone",
    "ccna-ipv4-network-route",
    "ccna-ipv4-return-route",
    "ccna-ipv4-specific-route",
    "ccna-ipv4-host-exception",
    "ccna-ipv4-floating-route",
    "ccna-ipv4-branch-handover",
    "lpic2-dns-dualstack",
    "lpic2-dns-aliases",
    "lpic2-dns-mail",
    "lpic2-dns-rrsets",
    "lpic2-dns-secondary",
    "lpic2-dns-handover",
    "shell-array-intake",
    "shell-array-priority",
    "shell-array-package",
    "shell-array-handover",
    "lpic1-process-pause",
    "lpic1-process-pending",
    "lpic1-process-priority",
    "lpic1-process-handover",
    "ccna-lldp-receive",
    "ccna-lldp-inventory",
    "ccna-lldp-transmit-policy",
    "ccna-lldp-handover",
    "lpic2-mount-identity",
    "lpic2-mount-policy",
    "lpic2-mount-manual",
    "lpic2-mount-handover",
    "shell-capture-preview",
    "shell-capture-evidence",
    "shell-capture-decision",
    "shell-capture-handover",
    "ccna-acl-source-policy",
    "ccna-acl-service-order",
    "ccna-acl-direction-repair",
    "ccna-acl-policy-handover",
    "lpic1-account-onboarding",
    "lpic1-account-membership",
    "lpic1-account-retirement",
    "lpic1-account-team-handover",
];
#[derive(serde::Deserialize)]
struct OldAuthored {
    id: String,
    scenario_version: u32,
    cases: Vec<shell::Case>,
    record: shell::Record,
}
fn prior_profile(all: &[Scenario]) -> Progress {
    // Actual evaluator-6 evidence is read unchanged, never generated or relabeled here.
    let frozen: Vec<OldAuthored> =
        serde_json::from_str(include_str!("fixtures/shell-v6-authored-records.json")).unwrap();
    assert_eq!(frozen.len(), 33);
    assert_eq!(
        frozen.iter().map(|o| o.record.results.len()).sum::<usize>(),
        182
    );
    assert_eq!(ORIGINAL_98_IDS.iter().collect::<BTreeSet<_>>().len(), 98);
    let mut p = Progress {
        schema_version: 1,
        clock: 1001,
        ..Default::default()
    };
    let mut scripts = 0;
    let mut histories = 0;
    for (i, id) in ORIGINAL_98_IDS.iter().enumerate() {
        let old = all
            .iter()
            .find(|s| s.id == *id)
            .expect("original lab retained");
        let mut a = Attempt::new(old, i as u64 + 1);
        if let Goal::Script { cases } = &old.goal {
            scripts += 1;
            assert_eq!(old.solution.len(), 1);
            let source = old.solution[0].clone();
            let record = shell::Record {
                engine_revision: shell::EVALUATOR_REVISION,
                results: cases.iter().map(|c| shell::run(&source, c)).collect(),
                source: source.clone(),
            };
            assert_eq!(record.engine_revision, 7);
            assert!(record.valid());
            assert!(
                cases.iter().zip(&record.results).all(|(c, r)| c.accepts(r)),
                "{id}"
            );
            a.state = State::Script {
                source,
                run: Some(record),
            };
        } else {
            for input in &old.solution {
                // Account mutations need their immutable goal context, just as in the UI.
                engine::execute_for_goal(&mut a.state, &old.goal, input).unwrap();
            }
        }
        assert!(
            engine::solved(&a.state, &old.goal),
            "real reference for {id}"
        );
        workspace::prepare(&mut a);
        if matches!(&old.goal, Goal::Script { .. }) {
            let lesson = old.teaching.as_ref().expect("script teaching retained");
            script_teaching::prepare(&mut a, lesson, true);
            if scripts == 1 {
                // A real failed practice run plus its unfinished editor source is resumable.
                let practice = a
                    .workspace
                    .as_mut()
                    .unwrap()
                    .script_learning
                    .as_mut()
                    .unwrap();
                practice.view = View::Example;
                practice.example_source = "printf 'unfinished".into();
                let result = shell::run(&practice.example_source, &lesson.fixture);
                assert!(result.error.is_some());
                practice.example_result = Some(result);
                practice.example_revision = shell::EVALUATOR_REVISION;
            }
        }
        if let Some(history) = frozen.iter().find(|o| o.id == *id) {
            histories += 1;
            assert_eq!(history.scenario_version, old.version);
            let Goal::Script { cases } = &old.goal else {
                panic!()
            };
            assert_eq!(
                serde_json::to_value(cases).unwrap(),
                serde_json::to_value(&history.cases).unwrap()
            );
            assert_eq!(history.record.engine_revision, 6);
            assert!(history.record.valid());
            a.workspace.as_mut().unwrap().script_history = Some(history.record.clone());
            assert!(progress::legacy_script_credit(&a, old));
        }
        a.challenge = i % 2 == 0;
        a.transcript
            .push(format!("Retained virtual reference work for {id}"));
        p.attempts.insert((*id).into(), a);
    }
    assert_eq!((scripts, histories), (37, 33));
    for id in [
        "ccna-ipv4",
        "shell-capture-handover",
        "lpic1-account-onboarding",
    ] {
        p.favorites.insert(id.into());
    }
    p
}
fn prior_status(p: &Progress, all: &[Scenario]) -> BTreeMap<String, bool> {
    ORIGINAL_98_IDS
        .iter()
        .map(|id| {
            let s = all.iter().find(|s| s.id == *id).unwrap();
            ((*id).into(), progress::current_solved(&p.attempts[*id], s))
        })
        .collect()
}

fn sync_draft(a: &mut Attempt) {
    a.workspace.as_mut().unwrap().ospf_draft = Some(Draft::new(sim(&a.state)).unwrap());
}
#[test]
fn real_pack_reference_and_alternate_costs_areas_or_hello_repairs_are_reachable() {
    let s = scenario();
    assert!(!pack::bundled().unwrap().iter().any(|l| l.id == s.id));
    assert!(!engine::solved(&s.initial, &s.goal));
    let mut reference = s.initial.clone();
    for c in &s.solution {
        engine::execute_for_goal(&mut reference, &s.goal, c).unwrap();
    }
    assert!(engine::solved(&reference, &s.goal));
    assert_eq!(
        sim(&reference).snapshot().unwrap().routers["Branch"]
            .route(dlms_learning_lab::ipv4_routing::Prefix::parse("198.51.100.0/24").unwrap())
            .unwrap()
            .cost,
        18
    );
    for variant in ["other-hello", "other-cost", "other-area"] {
        let mut state = s.initial.clone();
        let repair = action(
            sim(&state),
            if variant == "other-hello" {
                "Branch"
            } else {
                "Core"
            },
            "branch-core",
            |p| p.hello_seconds = if variant == "other-hello" { 20 } else { 10 },
        );
        engine::execute_for_goal(&mut state, &s.goal, &command(&repair)).unwrap();
        if variant == "other-cost" {
            let cost = action(sim(&state), "Branch", "branch-core", |p| p.cost = 101);
            engine::execute_for_goal(&mut state, &s.goal, &command(&cost)).unwrap();
        }
        if variant == "other-area" {
            for r in ["Branch", "Core", "Edge"] {
                let mut process = sim(&state).configuration().routers[r].process.clone();
                process.area = 42;
                engine::execute_for_goal(
                    &mut state,
                    &s.goal,
                    &command(&Action::SetProcess {
                        router: r.into(),
                        process,
                    }),
                )
                .unwrap();
            }
        }
        assert!(engine::solved(&state, &s.goal), "{variant}");
    }
}
#[test]
fn raw_engine_mutations_require_goal_and_wrong_context_cannot_publish() {
    let s = scenario();
    let mut state = s.initial.clone();
    let original = state.clone();
    assert!(engine::execute(&mut state, &s.solution[0]).is_err());
    assert_eq!(state, original);
    assert!(
        engine::execute(&mut state, "help")
            .unwrap()
            .contains("not observed Full")
    );
    assert!(
        engine::execute(&mut state, "show")
            .unwrap()
            .contains("no observed Full")
    );
    let wrong = pack::bundled().unwrap().remove(0).goal;
    assert!(engine::execute_for_goal(&mut state, &wrong, &s.solution[0]).is_err());
    assert_eq!(state, original);
    let mut encoded = serde_json::to_value(&state).unwrap();
    encoded["simulation"]["topology"]["devices"]["Branch"]["interfaces"]["branch-core"]["link"] =
        "other-link".into();
    let mut foreign: State = serde_json::from_value(encoded).unwrap();
    let before = foreign.clone();
    assert!(engine::execute_for_goal(&mut foreign, &s.goal, &s.solution[0]).is_err());
    assert_eq!(foreign, before);
    assert!(engine::execute_for_goal(&mut state, &s.goal, "router ospf 1").is_err());
    assert_eq!(state, original);
}
#[test]
fn explicit_goal_predicates_pin_fixed_peer_identities_and_complete_hop_sets() {
    let s = scenario();
    let (initial, rs) = goal(&s);
    let a = solved(&s);
    let current = sim(&a.state);
    let destination = dlms_learning_lab::ipv4_routing::Prefix::parse("198.51.100.0/24").unwrap();
    let route = current.snapshot().unwrap().routers["Branch"]
        .route(destination)
        .unwrap()
        .clone();
    let explicit = Requirement::RemoteRoute {
        router: "Branch".into(),
        destination,
        cost: Some(18),
        next_hops: Some(route.next_hops.clone()),
    };
    assert!(model::valid_requirements(
        initial,
        std::slice::from_ref(&explicit)
    ));
    assert!(model::checks(initial, current, std::slice::from_ref(&explicit))[0].0);
    let mut wrong = explicit.clone();
    let Requirement::RemoteRoute { next_hops, .. } = &mut wrong else {
        panic!()
    };
    *next_hops = Some(BTreeSet::from([NextHop {
        interface: "branch-core".into(),
        via: "10.60.0.2".parse().unwrap(),
        peer: endpoint("Edge", "core-edge"),
    }]));
    assert!(!model::valid_requirements(initial, &[wrong]));
    let wrong_peer = Requirement::Compatibility {
        local: endpoint("Branch", "branch-core"),
        peer: endpoint("Edge", "core-edge"),
    };
    assert!(!model::valid_requirements(initial, &[wrong_peer]));
    let mut wrong_cost = explicit;
    let Requirement::RemoteRoute { cost, .. } = &mut wrong_cost else {
        panic!()
    };
    *cost = Some(19);
    let c = model::checks(initial, current, &[wrong_cost]);
    assert!(!c[0].0 && c[0].1.contains("18") && c[0].1.contains("19"));
    assert!(!model::valid_requirements(initial, &[]));
    assert!(!model::valid_requirements(
        initial,
        &vec![rs[0].clone(); 33]
    ));
}
#[test]
fn negative_and_partial_configs_produce_current_local_feedback_without_history_credit() {
    let s = scenario();
    let (initial, rs) = goal(&s);
    let checks = model::checks(initial, initial, rs);
    assert!(checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0));
    assert!(checks.iter().any(|c| !c.0 && c.1.contains("Hello")));
    let mut a = solved(&s);
    let mutate = action(sim(&a.state), "Edge", "lan", |p| p.enabled = false);
    sim_mut(&mut a.state)
        .apply_against(initial, &mutate)
        .unwrap();
    sync_draft(&mut a);
    a.workspace.as_mut().unwrap().checked = true;
    assert!(!progress::current_solved(&a, &s));
    let c = model::checks(initial, sim(&a.state), rs);
    assert!(
        c.iter()
            .any(|c| !c.0 && c.1.contains("no current remote SPF"))
    );
    assert!(c.iter().any(|c| c.0));
    let repair = action(sim(&a.state), "Edge", "lan", |p| p.enabled = true);
    sim_mut(&mut a.state)
        .apply_against(initial, &repair)
        .unwrap();
    sync_draft(&mut a);
    assert!(progress::current_solved(&a, &s));
    for (pass, message) in c {
        if !pass {
            assert!(message.len() > 20);
            assert!(
                !message.contains("Condition::") && !message.contains("Full adjacency established")
            );
        }
    }
}
#[test]
fn absence_requires_positive_behavior_and_explicit_physical_guard_blocks_down_shortcut() {
    let s = scenario();
    let good = solved(&s);
    let initial = sim(&good.state);
    let absent = Requirement::AbsentCompatibility {
        local: endpoint("Branch", "branch-core"),
    };
    assert!(!model::valid_requirements(
        initial,
        std::slice::from_ref(&absent)
    ));
    let positive = Requirement::Compatibility {
        local: endpoint("Core", "core-edge"),
        peer: endpoint("Edge", "core-edge"),
    };
    assert!(!model::valid_requirements(
        initial,
        &[absent.clone(), positive.clone()]
    ));
    let rs = vec![
        absent,
        positive,
        Requirement::Physical {
            expected: initial.topology().clone(),
        },
    ];
    assert!(model::valid_requirements(initial, &rs));
    let mut wrong = initial.clone();
    wrong
        .apply_against(
            initial,
            &Action::SetUp {
                endpoint: endpoint("Branch", "branch-core"),
                up: false,
            },
        )
        .unwrap();
    let checks = model::checks(initial, &wrong, &rs);
    assert_eq!(
        checks.iter().map(|c| c.0).collect::<Vec<_>>(),
        [true, true, false]
    );
    let mut correct = initial.clone();
    let passive = action(&correct, "Branch", "branch-core", |p| p.passive = true);
    correct.apply_against(initial, &passive).unwrap();
    assert!(model::checks(initial, &correct, &rs).iter().all(|c| c.0));
}
#[test]
fn invalid_and_valid_unapplied_drafts_are_saved_editing_state_but_not_current_success() {
    let s = scenario();
    let initial = goal(&s).0;
    for text in ["", "bad", "42949672960", "0"] {
        let mut a = solved(&s);
        let before = a.state.clone();
        let d = a.workspace.as_mut().unwrap().ospf_draft.as_mut().unwrap();
        d.routers
            .get_mut("Branch")
            .unwrap()
            .interfaces
            .get_mut("branch-core")
            .unwrap()
            .cost = text.into();
        assert!(d.valid());
        assert!(d.apply(initial, sim_mut(&mut a.state)).is_err());
        assert_eq!(a.state, before);
        assert!(!progress::current_solved(&a, &s));
        let encoded =
            serde_json::to_string(a.workspace.as_ref().unwrap().ospf_draft.as_ref().unwrap())
                .unwrap();
        let decoded = Draft::from_json(&encoded, sim(&a.state)).unwrap();
        assert_eq!(
            decoded.routers["Branch"].interfaces["branch-core"].cost,
            text
        );
    }
    let mut a = solved(&s);
    a.workspace
        .as_mut()
        .unwrap()
        .ospf_draft
        .as_mut()
        .unwrap()
        .routers
        .get_mut("Branch")
        .unwrap()
        .interfaces
        .get_mut("branch-core")
        .unwrap()
        .cost = "51".into();
    assert!(!progress::current_solved(&a, &s));
    a.workspace
        .as_mut()
        .unwrap()
        .ospf_draft
        .as_mut()
        .unwrap()
        .apply(initial, sim_mut(&mut a.state))
        .unwrap();
    assert!(progress::current_solved(&a, &s));
}
#[test]
fn complete_draft_parses_before_publication_and_invalid_addresses_do_not_partly_apply() {
    let s = scenario();
    let initial = goal(&s).0;
    let mut a = prepared(&s);
    let before = a.state.clone();
    let d = a.workspace.as_mut().unwrap().ospf_draft.as_mut().unwrap();
    d.routers
        .get_mut("Core")
        .unwrap()
        .interfaces
        .get_mut("branch-core")
        .unwrap()
        .hello_seconds = "10".into();
    d.routers
        .get_mut("Edge")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = "not-an-address".into();
    assert!(d.apply(initial, sim_mut(&mut a.state)).is_err());
    assert_eq!(a.state, before);
    let d = a.workspace.as_mut().unwrap().ospf_draft.as_mut().unwrap();
    d.routers
        .get_mut("Edge")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = "10.60.0.1".into();
    assert!(d.apply(initial, sim_mut(&mut a.state)).is_err());
    assert_eq!(a.state, before);
    let d = a.workspace.as_mut().unwrap().ospf_draft.as_mut().unwrap();
    d.routers
        .get_mut("Edge")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = "198.51.100.1".into();
    d.routers
        .get_mut("Edge")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .prefix = "31".into();
    assert!(d.apply(initial, sim_mut(&mut a.state)).is_err());
    assert_eq!(a.state, before);
}
#[test]
fn separate_example_fill_only_changes_draft_then_try_and_repair_remain_ungraded() {
    let s = scenario();
    let lesson = s.ospf_teaching.as_ref().unwrap();
    assert!(lesson.separate_from(sim(&s.initial)));
    let mut a = prepared(&s);
    let assignment = a.state.clone();
    let l = learning(&mut a);
    let before = l.simulation.clone();
    l.fill_example(lesson).unwrap();
    assert_eq!(l.simulation, before);
    assert!(!l.checked);
    assert!(!l.draft.matches(&l.simulation));
    assert!(l.apply_check(lesson).unwrap().iter().all(|c| c.0));
    let mut desired = l.simulation.clone();
    for action in &lesson.try_change.actions {
        desired.apply_against(&lesson.fixture, action).unwrap();
    }
    l.draft = Draft::new(&desired).unwrap();
    let checks = l.apply_check(lesson).unwrap();
    assert_eq!(
        checks.iter().map(|c| c.0).collect::<Vec<_>>(),
        lesson.try_change.expected
    );
    let changed = l.simulation.clone();
    l.fill_example(lesson).unwrap();
    assert_eq!(l.simulation, changed);
    assert!(l.apply_check(lesson).unwrap().iter().all(|c| c.0));
    assert_eq!(a.state, assignment);
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn actual_98_attempts_current_37_scripts_and_frozen_6_history_survive_all_profile_paths() {
    let all = catalog();
    let s = all.last().unwrap();
    let mut p = prior_profile(&all);
    let prior = serde_json::to_value(&p.attempts).unwrap();
    let prior_status = prior_status(&p, &all);
    // Observe existing review semantics; do not manufacture completion from checked flags.
    for id in ORIGINAL_98_IDS {
        let lab = all.iter().find(|s| s.id == id).unwrap();
        if matches!(lab.goal, Goal::Script { .. }) {
            assert!(prior_status[id], "genuine current script record for {id}");
        }
    }
    let mut a = solved(s);
    let lesson = s.ospf_teaching.as_ref().unwrap();
    learning(&mut a).fill_example(lesson).unwrap();
    learning(&mut a).apply_check(lesson).unwrap();
    learning(&mut a).view = View::Example;
    a.workspace
        .as_mut()
        .unwrap()
        .ospf_draft
        .as_mut()
        .unwrap()
        .routers
        .get_mut("Branch")
        .unwrap()
        .area = "unfinished".into();
    learning(&mut a)
        .draft
        .routers
        .get_mut("PracticeLeft")
        .unwrap()
        .process_id = "".into();
    assert!(!progress::current_solved(&a, s));
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let expected = serde_json::to_value(&p.attempts).unwrap();
    let assert_preserved = |loaded: &Progress| {
        assert_eq!(serde_json::to_value(&loaded.attempts).unwrap(), expected);
        assert_eq!(loaded.clock, p.clock);
        assert_eq!(loaded.favorites, p.favorites);
        assert_eq!(self::prior_status(loaded, &all), prior_status);
        assert!(!progress::current_solved(&loaded.attempts[&s.id], s));
    };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic-ospf-progress.json");
    progress::save(&path, &p).unwrap();
    let original = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), original, "load is read-only");
    assert_preserved(&loaded);

    let input = dir.path().join("synthetic-export.json");
    backup::export(&input, &loaded, &all).unwrap();
    let exported = std::fs::read(&input).unwrap();
    let checked = AtomicUsize::new(0);
    let candidate = backup::inspect(&input, &all, &AtomicBool::new(false), &checked).unwrap();
    assert_eq!(
        candidate.rechecked, 38,
        "37 real current records and one real failed practice run"
    );
    assert_eq!(checked.load(std::sync::atomic::Ordering::Relaxed), 38);
    assert_preserved(&candidate.progress);
    assert_eq!(std::fs::read(&input).unwrap(), exported);
    assert_eq!(
        std::fs::read(&path).unwrap(),
        original,
        "inspection cannot replace the current profile"
    );
    let target = dir.path().join("synthetic-restore-target.json");
    let _lock = progress::lock_profile(&target).unwrap();
    progress::save(&target, &profile(s, prepared(s))).unwrap();
    let target_before = std::fs::read(&target).unwrap();
    let restored = backup::restore(&target, &candidate, &all).unwrap();
    assert_eq!(
        std::fs::read(restored.safety_copy.unwrap()).unwrap(),
        target_before
    );
    let mut restored = progress::load(&target, &all).unwrap();
    assert_preserved(&restored);
    assert_eq!(std::fs::read(&input).unwrap(), exported);

    restored.attempts.insert(s.id.clone(), prepared(s));
    progress::save(&target, &restored).unwrap();
    let reset = progress::load(&target, &all).unwrap();
    assert_eq!(reset.favorites, p.favorites);
    assert_eq!(reset.clock, p.clock);
    assert_eq!(reset.attempts[&s.id].state, s.initial);
    assert_eq!(self::prior_status(&reset, &all), prior_status);
    for id in ORIGINAL_98_IDS {
        assert_eq!(
            serde_json::to_value(&reset.attempts[id]).unwrap(),
            prior[id],
            "only selected OSPF attempt resets: {id}"
        );
    }
}
#[test]
fn independent_namespaces_can_use_coincident_evolving_router_ids_without_save_rejection() {
    let all = catalog();
    let s = all.last().unwrap();
    let mut a = prepared(s);
    let lesson = s.ospf_teaching.as_ref().unwrap();
    let mut process = learning(&mut a).simulation.configuration().routers["PracticeLeft"]
        .process
        .clone();
    process.router_id = sim(&s.initial).configuration().routers["Branch"]
        .process
        .router_id;
    learning(&mut a)
        .simulation
        .apply_against(
            &lesson.fixture,
            &Action::SetProcess {
                router: "PracticeLeft".into(),
                process,
            },
        )
        .unwrap();
    let fresh = Draft::new(&learning(&mut a).simulation).unwrap();
    learning(&mut a).draft = fresh;
    let p = profile(s, a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic-overlap.json");
    progress::save(&path, &p).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
}
#[test]
fn imported_graph_draft_cross_context_and_saved_snapshot_forgeries_are_rejected() {
    let all = catalog();
    let s = all.last().unwrap();
    let p = profile(s, prepared(s));
    let value = serde_json::to_value(&p).unwrap();
    for attack in ["graph", "draft", "practice", "cached"] {
        let mut bad = value.clone();
        let a = &mut bad["attempts"][&s.id];
        match attack {
            "graph" => {
                a["state"]["simulation"]["topology"]["devices"]["Branch"]["interfaces"]["branch-core"]
                    ["link"] = "foreign".into()
            }
            "draft" => a["workspace"]["ospf_draft"]["selected"]["router"] = "PracticeLeft".into(),
            "practice" => {
                a["workspace"]["ospf_learning"]["simulation"] =
                    serde_json::to_value(sim(&s.initial)).unwrap();
                a["workspace"]["ospf_learning"]["draft"] =
                    serde_json::to_value(Draft::new(sim(&s.initial)).unwrap()).unwrap();
            }
            _ => a["state"]["simulation"]["cached_complete"] = true.into(),
        }
        assert!(
            progress::decode(&serde_json::to_vec(&bad).unwrap(), &all).is_err(),
            "{attack}"
        );
    }
    let mut checked = p;
    checked
        .attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .checked = true;
    assert!(!progress::current_solved(&checked.attempts[&s.id], s));
}
#[test]
fn save_side_wrong_kind_metadata_rejects_before_overwriting_existing_bytes() {
    let all = catalog();
    let s = all.last().unwrap();
    let mut p = profile(s, prepared(s));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic-prior.json");
    progress::save(&path, &p).unwrap();
    let before = std::fs::read(&path).unwrap();
    let other = all.iter().find(|old| old.id != s.id).unwrap();
    let draft = p.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .ospf_draft
        .clone();
    let mut a = Attempt::new(other, 2);
    workspace::prepare(&mut a);
    a.workspace.as_mut().unwrap().ospf_draft = draft;
    p.attempts.insert(other.id.clone(), a);
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
}
#[test]
fn pack_rejects_wrong_identity_unreachable_reference_and_nonseparate_examples() {
    let root: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/ospf-workspace-preview.json")).unwrap();
    for attack in [
        "wrong-peer",
        "no-positive",
        "no-reference",
        "example-overlap",
    ] {
        let mut bad = root.clone();
        let s = &mut bad["scenarios"][0];
        match attack {
            "wrong-peer" => {
                s["goal"]["requirements"][0]["peer"] =
                    serde_json::json!({"router":"Edge","interface":"core-edge"})
            }
            "no-positive" => {
                s["goal"]["requirements"] = serde_json::json!([{ "kind":"Physical","expected":s["initial"]["simulation"]["topology"].clone() }])
            }
            "no-reference" => {
                s["solution"] = serde_json::json!([command(&Action::SetUp {
                    endpoint: endpoint("Branch", "branch-core"),
                    up: true
                })])
            }
            _ => {
                s["ospf_teaching"]["fixture"] = s["initial"]["simulation"].clone();
            }
        }
        assert!(pack::load(&bad.to_string()).is_err(), "{attack}");
    }
}
#[test]
fn pack_rejects_identical_requirements_but_allows_distinct_overlapping_guards() {
    let mut s = scenario();
    let initial = goal(&s).0.clone();
    let Goal::Ospf { requirements, .. } = &mut s.goal else {
        panic!()
    };
    requirements.push(requirements[0].clone());
    assert!(!model::valid_requirements(&initial, requirements));
    assert!(
        pack::load(&serde_json::json!({"schema_version":1,"scenarios":[s]}).to_string()).is_err()
    );
    let mut s = scenario();
    let Goal::Ospf { requirements, .. } = &mut s.goal else {
        panic!()
    };
    requirements.push(Requirement::Process {
        router: "Core".into(),
        expected: initial.configuration().routers["Core"].process.clone(),
    });
    assert!(model::valid_requirements(&initial, requirements));
    let s = pack::load(&serde_json::json!({"schema_version":1,"scenarios":[s]}).to_string())
        .unwrap()
        .remove(0);
    let mut state = s.initial.clone();
    let alternative = action(sim(&state), "Branch", "branch-core", |p| {
        p.hello_seconds = 20
    });
    engine::execute_for_goal(&mut state, &s.goal, &command(&alternative)).unwrap();
    assert!(engine::solved(&state, &s.goal));
}
#[test]
fn bounded_generic_state_and_draft_decoders_reject_oversized_names_values_and_duplicate_fields() {
    let s = scenario();
    let initial = sim(&s.initial);
    let json = serde_json::to_string(initial).unwrap();
    let decoded: Simulation = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, *initial);
    let draft = Draft::new(initial).unwrap();
    let value = serde_json::to_value(&draft).unwrap();
    let mut bad = value.clone();
    bad["routers"]["Branch"]["area"] = "x".repeat(65).into();
    assert!(serde_json::from_value::<Draft>(bad).is_err());
    assert!(Draft::from_json(&" ".repeat(model::MAX_DRAFT_BYTES + 1), initial).is_err());
    let mut invalid = value;
    invalid["routers"]["Branch"]["area"] = "not-a-number".into();
    let parsed: Draft = serde_json::from_value(invalid).unwrap();
    assert!(parsed.valid() && !parsed.matches(initial));
    let mut unknown = serde_json::to_value(initial).unwrap();
    unknown["x".repeat(4096)] = 0.into();
    let error = serde_json::from_value::<Simulation>(unknown)
        .unwrap_err()
        .to_string();
    assert!(error.len() < 512);
    let mut too_many = serde_json::to_value(initial).unwrap();
    let node = too_many["topology"]["devices"]["Branch"].clone();
    for i in 0..13 {
        too_many["topology"]["devices"][format!("extra{i}")] = node.clone();
    }
    assert!(serde_json::from_value::<Simulation>(too_many).is_err());
    let bad = r#"{"schema_version":1,"schema_version":1,"topology":{},"configuration":{}}"#;
    assert!(serde_json::from_str::<Simulation>(bad).is_err());
}
#[test]
fn export_inspect_and_confirmed_restore_preserve_current_states_and_unfinished_native_drafts() {
    let all = catalog();
    let s = all.last().unwrap();
    let mut a = solved(s);
    let lesson = s.ospf_teaching.as_ref().unwrap();
    learning(&mut a).fill_example(lesson).unwrap();
    learning(&mut a).apply_check(lesson).unwrap();
    a.workspace
        .as_mut()
        .unwrap()
        .ospf_draft
        .as_mut()
        .unwrap()
        .routers
        .get_mut("Branch")
        .unwrap()
        .area = "unfinished assignment".into();
    learning(&mut a)
        .draft
        .routers
        .get_mut("PracticeRight")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .cost = "".into();
    let mut p = profile(s, a);
    p.clock = 21;
    p.favorites.insert(s.id.clone());
    let expected = serde_json::to_value(&p.attempts).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("synthetic-ospf-backup.json");
    let target = dir.path().join("synthetic-current.json");
    let _lock = progress::lock_profile(&target).unwrap();
    progress::save(&target, &profile(s, prepared(s))).unwrap();
    let prior = std::fs::read(&target).unwrap();
    backup::export(&input, &p, &all).unwrap();
    let source = std::fs::read(&input).unwrap();
    let candidate =
        backup::inspect(&input, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert_eq!(candidate.rechecked, 0);
    assert_eq!(
        serde_json::to_value(&candidate.progress.attempts).unwrap(),
        expected
    );
    assert_eq!(std::fs::read(&target).unwrap(), prior);
    let restored = backup::restore(&target, &candidate, &all).unwrap();
    assert_eq!(std::fs::read(restored.safety_copy.unwrap()).unwrap(), prior);
    let loaded = progress::load(&target, &all).unwrap();
    assert_eq!(serde_json::to_value(&loaded.attempts).unwrap(), expected);
    assert_eq!(loaded.clock, 21);
    assert_eq!(loaded.favorites, p.favorites);
    assert_eq!(std::fs::read(&input).unwrap(), source);
    assert!(!progress::current_solved(&loaded.attempts[&s.id], s));
}
#[test]
fn immutable_practice_origin_distinguishes_same_graph_with_different_authored_settings() {
    let s = scenario();
    let lesson = s.ospf_teaching.as_ref().unwrap();
    let l = Learning::new(lesson).unwrap();
    let mut other = lesson.clone();
    let r = "PracticeLeft";
    let p = "uplink";
    let change = action(&other.fixture, r, p, |c| c.hello_seconds = 6);
    let initial = other.fixture.clone();
    other.fixture.apply_against(&initial, &change).unwrap();
    assert!(model::same_context(&other.fixture, &l.simulation));
    assert!(!l.valid_for(&other));
    let encoded = serde_json::to_string(&l).unwrap();
    assert!(Learning::from_json(&encoded, &other).is_err());
    let mut copy = l.clone();
    let mut process = copy.simulation.configuration().routers[r].process.clone();
    process.router_id = Some("1.1.1.1".parse().unwrap());
    copy.simulation
        .apply_against(
            &lesson.fixture,
            &Action::SetProcess {
                router: r.into(),
                process,
            },
        )
        .unwrap();
    copy.draft = Draft::new(&copy.simulation).unwrap();
    assert!(copy.valid_for(lesson));
    assert_eq!(copy.fixture, lesson.fixture);
    assert_eq!(l.simulation, lesson.fixture);
    assert_eq!(l.fixture, lesson.fixture);
}
fn maximum_state(
    tag: char,
    ring_octet: u8,
    stub_host: u8,
    rid_octet: u8,
) -> (Simulation, Endpoint, Endpoint) {
    use dlms_learning_lab::ipv4_routing::{Assignment, Device, Interface, Topology};
    use std::net::Ipv4Addr;
    let names: Vec<_> = (0..12)
        .map(|r| format!("{tag}R{r:02}{}", "r".repeat(28)))
        .collect();
    let out = format!("P0{}", "o".repeat(30));
    let input = format!("P1{}", "i".repeat(30));
    let mut topology = Topology {
        devices: BTreeMap::new(),
    };
    let mut configuration = ospf::Configuration {
        routers: BTreeMap::new(),
    };
    for (r, name) in names.iter().enumerate() {
        let mut interfaces = BTreeMap::new();
        let mut ports = BTreeMap::new();
        for p in 0..8 {
            let port = match p {
                0 => out.clone(),
                1 => input.clone(),
                _ => format!("P{p}{}", "p".repeat(30)),
            };
            let prev = (r + 11) % 12;
            let link_index = if p == 1 { prev } else { r };
            let (link, address, prefix) = if p < 2 {
                (
                    format!("{tag}L{link_index:02}{}", "l".repeat(28)),
                    Ipv4Addr::new(
                        192,
                        168,
                        ring_octet,
                        128 + link_index as u8 * 4 + if p == 0 { 1 } else { 2 },
                    ),
                    30,
                )
            } else {
                (
                    format!("{tag}S{r:02}{p}{}", "s".repeat(27)),
                    Ipv4Addr::new(192, 168, 100 + r as u8 * 8 + p, stub_host),
                    24,
                )
            };
            interfaces.insert(
                port.clone(),
                Interface {
                    link,
                    up: true,
                    address: Some(Assignment { address, prefix }),
                },
            );
            ports.insert(
                port,
                ospf::PortConfig {
                    enabled: true,
                    passive: p >= 2,
                    network_type: if p >= 2 {
                        ospf::NetworkType::Broadcast
                    } else {
                        ospf::NetworkType::PointToPoint
                    },
                    cost: u16::MAX,
                    hello_seconds: if r == 0 && p == 0 {
                        u16::MAX - 1
                    } else {
                        u16::MAX
                    },
                    dead_seconds: u32::MAX,
                },
            );
        }
        topology.devices.insert(
            name.clone(),
            Device {
                forwarding: true,
                interfaces,
                routes: vec![],
            },
        );
        configuration.routers.insert(
            name.clone(),
            ospf::RouterConfig {
                process: ospf::ProcessConfig {
                    enabled: true,
                    process_id: u16::MAX,
                    router_id: Some(Ipv4Addr::new(255, rid_octet, 255, 200 + r as u8)),
                    area: u32::MAX,
                },
                interfaces: ports,
            },
        );
    }
    let local = endpoint(&names[0], &out);
    let peer = endpoint(&names[1], &input);
    (
        Simulation::new(topology, configuration).unwrap(),
        local,
        peer,
    )
}
fn maximum_pending(d: &mut Draft) {
    let text = "\\\"".repeat(32);
    assert_eq!(text.len(), 64);
    for r in d.routers.values_mut() {
        r.process_id = text.clone();
        r.router_id = text.clone();
        r.area = text.clone();
        for p in r.interfaces.values_mut() {
            p.address = text.clone();
            p.prefix = text.clone();
            p.cost = text.clone();
            p.hello_seconds = text.clone();
            p.dead_seconds = text.clone();
        }
    }
    d.notice = Some("\\\"".repeat(256));
    assert!(d.valid());
}
#[test]
fn maximum_shape_names_and_escaped_pending_text_fit_learning_and_profile_roundtrip_bounds() {
    let (assignment, at, peer) = maximum_state('A', 200, 129, 254);
    let (example, ex_at, ex_peer) = maximum_state('P', 201, 130, 255);
    let fix = action(&example, &ex_at.router, &ex_at.interface, |p| {
        p.hello_seconds = u16::MAX
    });
    let break_it = action(&example, &ex_at.router, &ex_at.interface, |p| {
        p.hello_seconds = u16::MAX - 1
    });
    let lesson=ospf_teaching::Lesson{prerequisite_note:"Synthetic maximum-shape resource fixture; no learner curriculum.".into(),concepts:vec![dlms_learning_lab::script_teaching::Concept{title:"Finite settings model".into(),text:"Positive costs and bounded graph identities define a finite current compatibility calculation.".into()}],example_description:"Different maximum-size graph identities and independent draft buffers.".into(),fixture:example.clone(),requirements:vec![Requirement::Compatibility{local:ex_at,peer:ex_peer}],example_actions:vec![ospf_teaching::ExampleAction{action:fix.clone(),explanation:"Match the selected P2P Hello parameter without changing other fields.".into()}],try_change:ospf_teaching::TryChange{description:"Restore the mismatch, inspect the failed current check, then repair it.".into(),actions:vec![break_it],expected:vec![false]},repair:vec![fix]};
    lesson.validate().unwrap();
    assert!(lesson.separate_from(&assignment));
    let mut l = Learning::new(&lesson).unwrap();
    maximum_pending(&mut l.draft);
    assert!(l.valid_for(&lesson));
    for encoded in [
        serde_json::to_string(&l).unwrap(),
        serde_json::to_string_pretty(&l).unwrap(),
    ] {
        assert!(encoded.len() < ospf_teaching::MAX_LEARNING_BYTES);
        let decoded = Learning::from_json(&encoded, &lesson).unwrap();
        assert_eq!(
            serde_json::to_value(&decoded).unwrap(),
            serde_json::to_value(&l).unwrap()
        );
    }
    let mut s = scenario();
    s.initial = State::Ospf {
        simulation: assignment.clone(),
    };
    s.goal = Goal::Ospf {
        initial: assignment.clone(),
        requirements: vec![Requirement::Compatibility {
            local: at.clone(),
            peer,
        }],
    };
    s.solution = vec![command(&action(
        &assignment,
        &at.router,
        &at.interface,
        |p| p.hello_seconds = u16::MAX,
    ))];
    s.ospf_teaching = Some(lesson);
    let s = pack::load(&serde_json::json!({"schema_version":1,"scenarios":[s]}).to_string())
        .unwrap()
        .remove(0);
    let mut a = prepared(&s);
    maximum_pending(a.workspace.as_mut().unwrap().ospf_draft.as_mut().unwrap());
    a.workspace.as_mut().unwrap().ospf_learning = Some(l);
    let p = profile(&s, a);
    let pretty = serde_json::to_vec_pretty(&p).unwrap();
    assert!(pretty.len() < 16_000_000);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic-maximum-ospf.json");
    progress::save(&path, &p).unwrap();
    let loaded = progress::load(&path, std::slice::from_ref(&s)).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
    assert!(!progress::current_solved(&loaded.attempts[&s.id], &s));
}
