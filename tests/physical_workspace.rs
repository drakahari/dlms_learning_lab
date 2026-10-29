//! Prospective complete-attempt integration and isolation gates. No curriculum credit.
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use dlms_learning_lab::{
    backup,
    engine::{self, State},
    pack, physical_link as core, physical_link_workspace as work,
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};
#[path = "support/physical_scenario.rs"]
mod fixture;
fn sim(a: &Attempt) -> &core::State {
    let State::PhysicalLink { simulation } = &a.state else {
        panic!("physical fixture")
    };
    simulation
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 9);
    workspace::prepare(&mut a);
    work::prepare_attempt(&mut a, s, true).unwrap();
    a
}
fn wire(a: &Attempt) -> Vec<u8> {
    serde_json::to_vec(a).unwrap()
}
fn reference(s: &pack::Scenario) -> core::Configuration {
    let raw = &s.solution[0];
    let core::Action::Configure { configuration } =
        core::Action::from_json(raw.strip_prefix(work::PREFIX).unwrap().as_bytes()).unwrap()
    else {
        panic!("real configure reference")
    };
    configuration
}
fn solve(a: &mut Attempt, s: &pack::Scenario) {
    let mut draft = a
        .workspace
        .as_ref()
        .unwrap()
        .physical_link_draft
        .clone()
        .unwrap();
    draft.configuration = reference(s);
    work::set_draft(a, s, work::Scope::Assignment, &draft).unwrap();
    work::apply(a, s, work::Scope::Assignment).unwrap();
}
fn frontier(a: &mut Attempt) {
    a.transcript = vec![String::new(); 120];
    let remaining = work::MAX_ATTEMPT - work::size(a, work::MAX_ATTEMPT).unwrap();
    for (i, line) in a.transcript.iter_mut().enumerate() {
        *line = "x".repeat(remaining / 120 + usize::from(i < remaining % 120));
    }
    assert_eq!(wire(a).len(), work::MAX_ATTEMPT);
}
#[test]
fn current_outcomes_pending_edits_partial_break_and_goal_less_refusal_are_real() {
    let s = fixture::scenario();
    let mut a = fresh(&s);
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .physical_link_draft
            .as_ref()
            .unwrap()
            .selected_link,
        "assignment-uplink"
    );
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .physical_link_learning
            .as_ref()
            .unwrap()
            .draft
            .selected_link,
        "example-uplink"
    );
    assert!(!progress::current_solved(&a, &s));
    let initial = a.state.clone();
    assert!(engine::execute(&mut a.state, &s.solution[0]).is_err());
    assert_eq!(a.state, initial);
    solve(&mut a, &s);
    assert!(progress::current_solved(&a, &s));
    assert!(
        progress::current_physical_link_checks(&a, &s)
            .iter()
            .all(|r| r.0)
    );
    let mut draft = a
        .workspace
        .as_ref()
        .unwrap()
        .physical_link_draft
        .clone()
        .unwrap();
    draft
        .configuration
        .ports
        .get_mut("assignment-left")
        .unwrap()
        .admin_up = false;
    let applied = a.state.clone();
    work::set_draft(&mut a, &s, work::Scope::Assignment, &draft).unwrap();
    assert_eq!(a.state, applied);
    assert!(!progress::current_solved(&a, &s));
    work::apply(&mut a, &s, work::Scope::Assignment).unwrap();
    assert!(!progress::current_solved(&a, &s));
    solve(&mut a, &s);
    assert!(progress::current_solved(&a, &s));
    // A protected neighbor is not a grading shortcut even with target healthy.
    draft.configuration = reference(&s);
    draft
        .configuration
        .ports
        .get_mut("assignment-ops-left")
        .unwrap()
        .admin_up = false;
    work::set_draft(&mut a, &s, work::Scope::Assignment, &draft).unwrap();
    work::apply(&mut a, &s, work::Scope::Assignment).unwrap();
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn examples_fill_apply_probe_reset_and_scope_resume_never_replace_assignment_or_credit() {
    let s = fixture::scenario();
    let mut a = fresh(&s);
    let before = a.state.clone();
    let draft = a.workspace.as_ref().unwrap().physical_link_draft.clone();
    work::set_view(&mut a, &s, View::Example).unwrap();
    work::fill_example(&mut a, &s).unwrap();
    assert_eq!(a.state, before);
    assert_eq!(a.workspace.as_ref().unwrap().physical_link_draft, draft);
    assert!(!progress::current_solved(&a, &s));
    work::apply(&mut a, &s, work::Scope::Practice).unwrap();
    work::perform(
        &mut a,
        &s,
        work::Scope::Practice,
        &core::Action::Probe {
            link: "example-uplink".into(),
            a_frames: 4,
            b_frames: 8,
        },
    )
    .unwrap();
    let practice = a.workspace.as_ref().unwrap().physical_link_learning.clone();
    work::reset(&mut a, &s, work::Scope::Assignment).unwrap();
    assert_eq!(
        a.workspace.as_ref().unwrap().physical_link_learning,
        practice
    );
    let mut p = Progress {
        schema_version: 1,
        clock: 9,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a.clone());
    let mut all = pack::bundled().unwrap();
    assert_eq!(one_off_boundary::before_len(&all), 176);
    all.push(s.clone());
    let decoded = progress::decode(&serde_json::to_vec(&p).unwrap(), &all).unwrap();
    assert_eq!(
        serde_json::to_value(&decoded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("physical-progress-backup.json");
    backup::export(&path, &p, &all).unwrap();
    let inspected = backup::inspect(
        &path,
        &all,
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    )
    .unwrap();
    let restored_path = dir.path().join("restored-profile.json");
    backup::restore(&restored_path, &inspected, &all).unwrap();
    let restored = progress::load(&restored_path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    work::reset(&mut a, &s, work::Scope::Practice).unwrap();
    assert_eq!(a.state, before);
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn absent_workspace_mode_visit_digit_boundary_and_failed_prepare_keep_complete_attempt() {
    let s = fixture::scenario();
    let mut a = Attempt::new(&s, 9);
    frontier(&mut a);
    work::context(&a, &s).unwrap();
    let old = wire(&a);
    assert!(work::prepare_attempt(&mut a, &s, true).is_err());
    assert_eq!(wire(&a), old);
    assert!(a.workspace.is_none());
    work::record_mode(&mut a, &s, true).unwrap();
    assert_eq!(wire(&a).len(), work::MAX_ATTEMPT - 1);
    assert!(a.workspace.is_none());
    work::record_visit(&mut a, &s, 10).unwrap();
    assert_eq!(wire(&a).len(), work::MAX_ATTEMPT);
    let full = wire(&a);
    assert!(work::record_visit(&mut a, &s, 100).is_err());
    assert_eq!(wire(&a), full);
    assert!(work::record_mode(&mut a, &s, false).is_err());
    assert_eq!(wire(&a), full);
}
#[test]
fn full_workspace_refuses_probe_growth_but_allows_exact_shrinking_draft_and_view() {
    let s = fixture::scenario();
    let mut a = fresh(&s);
    frontier(&mut a);
    let old = wire(&a);
    assert!(
        work::perform(
            &mut a,
            &s,
            work::Scope::Assignment,
            &core::Action::Probe {
                link: "assignment-uplink".into(),
                a_frames: 12,
                b_frames: 18
            }
        )
        .is_err()
    );
    assert_eq!(wire(&a), old);
    let mut draft = a
        .workspace
        .as_ref()
        .unwrap()
        .physical_link_draft
        .clone()
        .unwrap();
    draft.selected_port = "assignment-ops-right".into();
    assert!(work::set_draft(&mut a, &s, work::Scope::Assignment, &draft).is_err());
    assert_eq!(wire(&a), old);
    draft.selected_port.clear();
    work::set_draft(&mut a, &s, work::Scope::Assignment, &draft).unwrap();
    assert!(wire(&a).len() < work::MAX_ATTEMPT);
    work::set_view(&mut a, &s, View::Example).unwrap();
    assert!(wire(&a).len() < work::MAX_ATTEMPT);
    work::context(&a, &s).unwrap();
}
#[test]
fn valid_foreign_metadata_is_rejected_in_all_old_kinds_without_preparation_mutation() {
    let s = fixture::scenario();
    let a = fresh(&s);
    let w = a.workspace.as_ref().unwrap();
    let draft = w.physical_link_draft.clone().unwrap();
    let learning = w.physical_link_learning.clone().unwrap();
    assert!(learning.valid());
    let all = pack::bundled().unwrap();
    assert_eq!(one_off_boundary::before_len(&all), 176);
    let old = all
        .iter()
        .filter(|s| !matches!(s.initial, State::PhysicalLink { .. }))
        .collect::<Vec<_>>();
    assert_eq!(
        old[..172].len(),
        172,
        "Every original171 and private-office1 remains in the foreign-kind matrix"
    );
    let prior180 = one_off_boundary::loop_registration::before_scenarios(&all);
    let previous = prior180
        .iter()
        .filter(|s| !matches!(s.initial, State::PhysicalLink { .. }))
        .collect::<Vec<_>>();
    assert_eq!(previous.len(), 176);
    assert_eq!(old.len(), 178);
    assert_eq!(
        old[..previous.len()]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        previous.iter().map(|s| s.id.as_str()).collect::<Vec<_>>()
    );
    assert_eq!(
        old[previous.len()..]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        ["shell-loop-intake", "shell-loop-fallback"]
    );
    assert!(
        old[previous.len()..]
            .iter()
            .all(|s| matches!(s.initial, State::Script { .. })
                && matches!(s.goal, engine::Goal::Script { .. }))
    );
    assert_eq!(
        previous[172..]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "lpic1-at-report",
            "lpic1-at-capture-repair",
            "lpic1-at-owner-queue",
            "lpic1-at-handover"
        ]
    );
    assert!(
        previous[172..]
            .iter()
            .all(|s| matches!(&s.initial, State::OneOff { .. }))
    );

    assert_eq!(
        old.iter()
            .filter(|s| s.id == "ccna-private-office-plan")
            .count(),
        1
    );
    let current = all
        .iter()
        .filter(|s| matches!(s.initial, State::PhysicalLink { .. }))
        .collect::<Vec<_>>();
    assert_eq!(current.len(), 4);
    for source in current {
        let mut own = Attempt::new(source, 1);
        work::prepare_attempt(&mut own, source, true).unwrap();
        assert!(
            work::metadata(&own).is_ok(),
            "Own-family metadata must be admitted"
        );
        assert!(work::context(&own, source).is_ok());
        assert!(!progress::current_solved(&own, source));
    }
    for old in old {
        for mask in 1..4 {
            let mut own = Attempt::new(old, 1);
            workspace::prepare(&mut own);
            let w = own
                .workspace
                .get_or_insert_with(workspace::WorkspaceData::new);
            if mask & 1 != 0 {
                w.physical_link_draft = Some(draft.clone());
            }
            if mask & 2 != 0 {
                w.physical_link_learning = Some(learning.clone());
            }
            let before = wire(&own);
            assert!(work::metadata(&own).is_err());
            assert!(!progress::current_solved(&own, old));
            workspace::prepare(&mut own);
            assert_eq!(wire(&own), before, "{}/mask{mask}", old.id);
        }
    }
}
#[test]
fn source_pin_malformed_pack_changed_topology_and_partial_drafts_are_honest_failures() {
    let s = fixture::scenario();
    let mut a = fresh(&s);
    let before = wire(&a);
    let mut wrong = s.clone();
    wrong.title.push_str(" source mutation");
    // Title-only source is not a topology mutation; source-aware future memo must
    // key it fully. Changed teaching/source equipment, however, cannot resume.
    wrong
        .physical_link_teaching
        .as_mut()
        .unwrap()
        .introduction
        .push_str(" changed");
    assert!(work::context(&a, &wrong).is_err());
    assert_eq!(wire(&a), before);
    let mut bad = serde_json::to_value(&s).unwrap();
    bad["initial"]["simulation"]["topology"]["links"]["assignment-uplink"]["required_span_m"] =
        999.into();
    let raw = serde_json::json!({"schema_version":1,"scenarios":[bad]});
    assert!(pack::load(&serde_json::to_string(&raw).unwrap()).is_err());
    let mut draft = a
        .workspace
        .as_ref()
        .unwrap()
        .physical_link_draft
        .clone()
        .unwrap();
    draft
        .configuration
        .ports
        .get_mut("assignment-left")
        .unwrap()
        .profile = core::Profile::LxLhOs2;
    work::set_draft(&mut a, &s, work::Scope::Assignment, &draft).unwrap();
    let pending = wire(&a);
    assert!(work::apply(&mut a, &s, work::Scope::Assignment).is_err());
    assert_eq!(wire(&a), pending);
    assert!(!progress::current_solved(&a, &s));
    let mut other = pack::bundled().unwrap()[0].clone();
    other.physical_link_teaching = s.physical_link_teaching.clone();
    let raw = serde_json::json!({"schema_version":1,"scenarios":[other]});
    assert!(pack::load(&serde_json::to_string(&raw).unwrap()).is_err());
}
#[test]
fn action_projection_matches_every_real_action_and_rejects_without_mutating_current_state() {
    let s = fixture::scenario();
    let a = fresh(&s);
    let state = sim(&a);
    let original = serde_json::to_vec(state).unwrap();
    let config = reference(&s);
    let port = config.ports["assignment-left"].clone();
    let actions = [
        core::Action::Configure {
            configuration: config,
        },
        core::Action::ConfigurePort {
            port: "assignment-left".into(),
            configuration: port,
        },
        core::Action::SelectCable {
            link: "assignment-uplink".into(),
            cable: "assignment-clean".into(),
        },
        core::Action::ClearCounters {
            port: "assignment-left".into(),
        },
        core::Action::Probe {
            link: "assignment-uplink".into(),
            a_frames: 4,
            b_frames: 8,
        },
    ];
    for action in actions {
        let projected = state.project(&action).unwrap();
        let expected = serde_json::to_value(&projected).unwrap();
        let mut actual = state.clone();
        let report = actual.perform(&action).unwrap();
        assert_eq!(serde_json::to_value(actual).unwrap(), expected);
        assert_eq!(
            serde_json::to_value(projected.report()).unwrap(),
            serde_json::to_value(report).unwrap()
        );
        assert_eq!(serde_json::to_vec(state).unwrap(), original);
    }
    let invalid = core::Action::SelectCable {
        link: "assignment-uplink".into(),
        cable: "assignment-ops-wire".into(),
    };
    assert!(state.project(&invalid).is_err());
    assert_eq!(serde_json::to_vec(state).unwrap(), original);
}
#[test]
fn full_practice_draft_apply_fill_and_flags_are_atomic_and_do_not_edit_assignment() {
    let s = fixture::scenario();
    let mut a = fresh(&s);
    let assignment = a.state.clone();
    frontier(&mut a);
    let full = wire(&a);
    assert!(work::fill_example(&mut a, &s).is_err());
    assert_eq!(wire(&a), full);
    let mut draft = a
        .workspace
        .as_ref()
        .unwrap()
        .physical_link_learning
        .as_ref()
        .unwrap()
        .draft
        .clone();
    draft.selected_port = "example-ops-right".into();
    assert!(work::set_draft(&mut a, &s, work::Scope::Practice, &draft).is_err());
    assert_eq!(wire(&a), full);
    assert!(
        work::perform(
            &mut a,
            &s,
            work::Scope::Practice,
            &core::Action::Probe {
                link: "example-uplink".into(),
                a_frames: 12,
                b_frames: 18
            }
        )
        .is_err()
    );
    assert_eq!(wire(&a), full);
    work::set_checked(&mut a, &s, work::Scope::Practice, true).unwrap();
    assert_eq!(wire(&a).len(), work::MAX_ATTEMPT - 1);
    assert_eq!(a.state, assignment);
    assert!(!progress::current_solved(&a, &s));
    work::set_checked(&mut a, &s, work::Scope::Practice, false).unwrap();
    assert_eq!(wire(&a), full);
    // Apply can also grow CURRENT example state even when the pending draft was
    // already admitted; populate it before bringing only metadata to the frontier.
    a.transcript.clear();
    work::fill_example(&mut a, &s).unwrap();
    frontier(&mut a);
    let pending = wire(&a);
    assert!(work::apply(&mut a, &s, work::Scope::Practice).is_err());
    assert_eq!(wire(&a), pending);
    assert_eq!(a.state, assignment);
    work::reset(&mut a, &s, work::Scope::Practice).unwrap();
    assert_eq!(a.state, assignment);
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn full_section_hint_digits_and_scope_reset_use_exact_whole_attempt_sizes() {
    let mut s = fixture::scenario();
    s.steps = (0..11)
        .map(|n| format!("Technical supporting section {n}"))
        .collect();
    s.hints = (0..11).map(|n| format!("Technical hint {n}")).collect();
    let mut a = fresh(&s);
    work::record_step(&mut a, &s, 9).unwrap();
    work::record_hints(&mut a, &s, 9).unwrap();
    frontier(&mut a);
    let before = wire(&a);
    assert!(work::record_step(&mut a, &s, 10).is_err());
    assert_eq!(wire(&a), before);
    assert!(work::record_hints(&mut a, &s, 10).is_err());
    assert_eq!(wire(&a), before);
    work::record_mode(&mut a, &s, true).unwrap();
    work::record_step(&mut a, &s, 10).unwrap();
    assert_eq!(wire(&a).len(), work::MAX_ATTEMPT);
    let before = wire(&a);
    assert!(work::record_hints(&mut a, &s, 10).is_err());
    assert_eq!(wire(&a), before);
    work::record_step(&mut a, &s, 0).unwrap();
    work::record_hints(&mut a, &s, 10).unwrap();
    assert_eq!(wire(&a).len(), work::MAX_ATTEMPT);
    a.transcript.clear();
    solve(&mut a, &s);
    let practice = a.workspace.as_ref().unwrap().physical_link_learning.clone();
    frontier(&mut a);
    work::reset(&mut a, &s, work::Scope::Assignment).unwrap();
    assert_eq!(
        a.workspace.as_ref().unwrap().physical_link_learning,
        practice
    );
    assert_eq!(a.state, s.initial);
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn current_registered_catalog_and_nil_optional_fields_remain_exactly_old_scope() {
    let all = pack::bundled().unwrap();
    assert_eq!(one_off_boundary::before_len(&all), 176);
    assert_eq!(
        one_off_boundary::before_len(&dlms_learning_lab::course::bundled(&all).unwrap()),
        46
    );
    assert_eq!(
        all[171..175]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "ccna-physical-media-span",
            "ccna-physical-duplex-diagnosis",
            "ccna-physical-counter-baseline",
            "ccna-physical-uplink-handover"
        ]
    );
    for s in &all[171..175] {
        assert!(s.physical_link_teaching.is_some());
        let mut a = Attempt::new(s, 1);
        assert!(!work::has_metadata(&a));
        work::prepare_attempt(&mut a, s, true).unwrap();
        assert!(work::has_metadata(&a));
        assert!(!progress::current_solved(&a, s));
    }
    for s in &all[..171] {
        assert!(s.physical_link_teaching.is_none());
        assert!(
            serde_json::to_value(s)
                .unwrap()
                .get("physical_link_teaching")
                .is_none()
        );
        let a = Attempt::new(s, 1);
        assert!(!work::has_metadata(&a));
    }
    let p: Progress = serde_json::from_slice(include_bytes!(
        "fixtures/shell-v8-synthetic-writer-progress.json"
    ))
    .unwrap();
    assert_eq!(p.attempts.len(), 116);
    for a in p.attempts.values() {
        if let Some(w) = &a.workspace {
            let value = serde_json::to_value(w).unwrap();
            assert!(value.get("physical_link_draft").is_none());
            assert!(value.get("physical_link_learning").is_none());
        }
    }
    let decoded = progress::decode(
        include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json"),
        &all,
    )
    .unwrap();
    assert_eq!(decoded.attempts.len(), 116);
    assert!(decoded.attempts.values().all(|a| !work::has_metadata(a)));
}
