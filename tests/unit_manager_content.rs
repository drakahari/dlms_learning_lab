//! STAGED ONLY. Copy to tests/ after content/core/native review; never host tools.
use dlms_learning_lab::{
    backup,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    shell,
    unit_manager::{Action, Program, UnitType},
    unit_manager_teaching::Learning,
    unit_manager_terminal,
    unit_manager_workspace::{self as ws, Definition, Draft, EditAction, Requirement},
    workspace,
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicBool, AtomicUsize},
};

const IDS: [&str; 4] = [
    "lpic2-unit-loaded-recovery",
    "lpic2-unit-preparation-order",
    "lpic2-unit-optional-wants",
    "lpic2-unit-target-handover",
];
#[derive(Deserialize)]
struct Review {
    original_catalog_ids: Vec<String>,
    plans: Vec<Plan>,
}
#[derive(Deserialize)]
struct Plan {
    id: String,
    reference: Vec<EditAction>,
    alternates: Vec<Vec<EditAction>>,
    negative_sequences: Vec<Negative>,
}
#[derive(Deserialize)]
struct Negative {
    name: String,
    actions: Vec<EditAction>,
}
fn plans() -> Vec<Plan> {
    serde_json::from_str::<Review>(include_str!("fixtures/unit-manager-content-review.json"))
        .unwrap()
        .plans
}
fn catalog() -> Vec<pack::Scenario> {
    // All real prerequisites resolve; do not remove the original lpic2-service gate.
    pack::load_catalog(&[
        include_str!("../scenarios/starter-v1.json"),
        include_str!("../scenarios/unit-manager-v1.json"),
    ])
    .unwrap()
}
fn definition(s: &pack::Scenario) -> &Definition {
    let Goal::UnitManager { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn run(def: &Definition, actions: &[EditAction]) -> dlms_learning_lab::unit_manager::State {
    let mut state = def.initial.clone();
    for action in actions {
        ws::apply(&mut state, &def.initial, action).unwrap();
    }
    state
}
fn passed(def: &Definition, state: &dlms_learning_lab::unit_manager::State) -> bool {
    ws::solved(def, state, &Draft::new(state).unwrap())
}
fn manager(kind: Action) -> EditAction {
    EditAction::Manager { action: kind }
}
fn rejected_with_feedback(
    def: &Definition,
    state: &dlms_learning_lab::unit_manager::State,
    label: &str,
) {
    let checks = ws::checks(def, state, &Draft::new(state).unwrap());
    assert!(
        checks
            .iter()
            .any(|(ok, text)| !ok && !text.trim().is_empty()),
        "{label}"
    );
    assert!(!passed(def, state), "{label}");
}
fn change_input(state: &mut dlms_learning_lab::unit_manager::State, unit: &str) {
    let defs = state.definitions().unwrap();
    let UnitType::Service { exec, .. } = &defs[unit].kind else {
        panic!("service input required")
    };
    let (path, text) = match &state.inventory().programs[exec] {
        Program::PortDaemon { config, key, .. } => (config.clone(), format!("{key}=1\n")),
        Program::ValueTask { config, key, .. } => {
            (config.clone(), format!("{key}=changed-after-pass\n"))
        }
    };
    state.apply(&Action::SaveConfig { path, text }).unwrap();
}
#[test]
fn every_required_current_predicate_can_be_broken_after_passing_without_history_credit() {
    for plan in plans() {
        let labs = catalog();
        let def = definition(labs.iter().find(|s| s.id == plan.id).unwrap());
        let solved = run(def, &plan.reference);
        for (index, requirement) in def.requirements.iter().enumerate() {
            let mut wrong = solved.clone();
            match requirement {
                Requirement::SavedConfigurationValid {} => {
                    let unit = wrong.documents().keys().next().unwrap().clone();
                    wrong
                        .apply(&Action::SaveUnit {
                            unit,
                            text: "[broken\n".into(),
                        })
                        .unwrap();
                }
                Requirement::LoadedCurrent { unit } => {
                    let text = format!("{}# pending reload after pass\n", wrong.documents()[unit]);
                    wrong
                        .apply(&Action::SaveUnit {
                            unit: unit.clone(),
                            text,
                        })
                        .unwrap();
                }
                Requirement::Runtime { unit, phase, .. } => {
                    use ws::Phase;
                    let action = if matches!(phase, Phase::Inactive) {
                        Action::Start { unit: unit.clone() }
                    } else {
                        Action::Stop { unit: unit.clone() }
                    };
                    wrong.apply(&action).unwrap();
                    wrong.apply(&Action::AdvancePrograms {}).unwrap();
                }
                Requirement::ExecutionCurrent { unit } => change_input(&mut wrong, unit),
                Requirement::Artifact { producer, .. } => change_input(&mut wrong, producer),
                Requirement::InstallLink { unit, present, .. } => {
                    wrong
                        .apply(&if *present {
                            Action::Disable { unit: unit.clone() }
                        } else {
                            Action::Enable { unit: unit.clone() }
                        })
                        .unwrap();
                }
                Requirement::DefaultTarget { target } => {
                    let other = wrong
                        .inventory()
                        .units
                        .iter()
                        .find(|u| u.ends_with(".target") && *u != target)
                        .unwrap()
                        .clone();
                    wrong.apply(&Action::SetDefault { target: other }).unwrap();
                }
                Requirement::Dependency {
                    unit,
                    target,
                    dependency,
                    present,
                } => {
                    let line = format!("{dependency:?}={target}\n");
                    let text = if *present {
                        wrong.documents()[unit].replace(&line, "")
                    } else {
                        wrong.documents()[unit].replace("[Unit]\n", &format!("[Unit]\n{line}"))
                    };
                    wrong
                        .apply(&Action::SaveUnit {
                            unit: unit.clone(),
                            text,
                        })
                        .unwrap();
                    wrong.apply(&Action::DaemonReload {}).unwrap();
                }
            }
            rejected_with_feedback(def, &wrong, &format!("{} {requirement:?}", plan.id));
            // Draft row first, then current requirements in declared order.
            // Another failed preservation/fresh row cannot stand in for this.
            let checks = ws::checks(def, &wrong, &Draft::new(&wrong).unwrap());
            assert!(checks.len() > index + 1);
            assert!(
                !checks[index + 1].0,
                "{} target predicate stayed true: {requirement:?}",
                plan.id
            );
        }
    }
}
#[test]
fn each_preserved_document_configuration_link_and_default_has_real_negative_evidence() {
    let labs = catalog();
    for plan in plans() {
        let def = definition(labs.iter().find(|s| s.id == plan.id).unwrap());
        let solved = run(def, &plan.reference);
        for unit in &def.preservation.unit_documents {
            let mut wrong = solved.clone();
            let text = format!("{}# unrelated change\n", wrong.documents()[unit]);
            wrong
                .apply(&Action::SaveUnit {
                    unit: unit.clone(),
                    text,
                })
                .unwrap();
            wrong.apply(&Action::DaemonReload {}).unwrap();
            rejected_with_feedback(def, &wrong, unit);
            assert!(
                ws::checks(def, &wrong, &Draft::new(&wrong).unwrap())
                    .iter()
                    .any(|(ok, m)| !ok && m.contains(unit) && m.contains("document unchanged"))
            );
        }
        for path in &def.preservation.configuration {
            let mut wrong = solved.clone();
            let key = wrong.configuration()[path].split_once('=').unwrap().0;
            let text = format!("{key}=changed-after-pass\n");
            wrong
                .apply(&Action::SaveConfig {
                    path: path.clone(),
                    text,
                })
                .unwrap();
            rejected_with_feedback(def, &wrong, path);
            assert!(
                ws::checks(def, &wrong, &Draft::new(&wrong).unwrap())
                    .iter()
                    .any(|(ok, m)| !ok
                        && m.contains(path)
                        && m.contains("configuration unchanged"))
            );
        }
        for (target, units) in def.initial.boot_links() {
            for unit in units {
                if def
                    .preservation
                    .editable_boot_links
                    .iter()
                    .any(|p| p.target == *target && p.unit == *unit)
                {
                    continue;
                }
                let mut wrong = solved.clone();
                wrong
                    .apply(&Action::Disable { unit: unit.clone() })
                    .unwrap();
                rejected_with_feedback(def, &wrong, &format!("{target}/{unit}"));
                assert!(
                    ws::checks(def, &wrong, &Draft::new(&wrong).unwrap())
                        .iter()
                        .any(|(ok, m)| !ok && m.contains("enablement") && m.contains("unchanged"))
                );
            }
        }
        if def.preservation.default_target {
            let mut wrong = solved.clone();
            let other = wrong
                .inventory()
                .units
                .iter()
                .find(|u| u.ends_with(".target") && u.as_str() != wrong.default_target())
                .unwrap()
                .clone();
            wrong.apply(&Action::SetDefault { target: other }).unwrap();
            rejected_with_feedback(def, &wrong, "preserved default");
            assert!(
                ws::checks(def, &wrong, &Draft::new(&wrong).unwrap())
                    .iter()
                    .any(|(ok, m)| !ok && m == "Keep the unaffected default target unchanged")
            );
        }
    }
}
#[test]
fn valid_wrong_port_and_inactive_oneshot_are_not_current_program_or_preparation_success() {
    let labs = catalog();
    for plan in plans() {
        let def = definition(labs.iter().find(|s| s.id == plan.id).unwrap());
        let mut wrong = run(def, &plan.reference);
        let app = wrong
            .inventory()
            .units
            .iter()
            .find(|u| u.contains("-app."))
            .unwrap()
            .clone();
        change_input(&mut wrong, &app);
        wrong.apply(&Action::Stop { unit: app.clone() }).unwrap();
        wrong.apply(&Action::Start { unit: app.clone() }).unwrap();
        wrong.apply(&Action::AdvancePrograms {}).unwrap();
        assert_eq!(wrong.result(&app).unwrap().unwrap().port, Some(1));
        rejected_with_feedback(def, &wrong, "valid wrong application port");
        if let Some(prep) = def
            .initial
            .inventory()
            .units
            .iter()
            .find(|u| u.contains("-prepare."))
        {
            let mut actions = plan.reference.clone();
            let EditAction::Save { draft } = &mut actions[0] else {
                panic!()
            };
            let text = draft.documents[prep].replace("RemainAfterExit=yes", "RemainAfterExit=no");
            draft.documents.insert(prep.clone(), text);
            let wrong = run(def, &actions);
            assert!(wrong.result(&app).unwrap().is_some());
            rejected_with_feedback(def, &wrong, "successful oneshot no longer remains active");
        }
    }
}
#[test]
fn current_only_default_only_enable_only_and_repaired_optional_helper_are_plausible_partial_work() {
    let labs = catalog();
    let review = plans();
    let cap = &review[3];
    let def = definition(labs.iter().find(|s| s.id == cap.id).unwrap());
    for action in [
        Action::SetDefault {
            target: "publish.target".into(),
        },
        Action::Enable {
            unit: "publish-app.service".into(),
        },
    ] {
        rejected_with_feedback(def, &run(def, &[manager(action)]), "policy only");
    }
    let mut current_only = cap.reference.clone();
    current_only.retain(|a| {
        !matches!(
            a,
            EditAction::Manager {
                action: Action::Enable { .. } | Action::SetDefault { .. }
            }
        )
    });
    let start = current_only
        .iter_mut()
        .find(|a| {
            matches!(
                a,
                EditAction::Manager {
                    action: Action::Start { .. }
                }
            )
        })
        .unwrap();
    *start = manager(Action::Start {
        unit: "publish-app.service".into(),
    });
    let state = run(def, &current_only);
    assert_eq!(
        state.result("publish-app.service").unwrap().unwrap().port,
        Some(9443)
    );
    rejected_with_feedback(def, &state, "working now but no new default/link");
    let opt = &review[2];
    let def = definition(labs.iter().find(|s| s.id == opt.id).unwrap());
    let mut wrong = run(def, &opt.reference);
    wrong
        .apply(&Action::SaveConfig {
            path: "/etc/portal/optional.conf".into(),
            text: "VALUE=now-successful\n".into(),
        })
        .unwrap();
    wrong
        .apply(&Action::Stop {
            unit: "portal-optional.service".into(),
        })
        .unwrap();
    wrong
        .apply(&Action::Start {
            unit: "portal-optional.service".into(),
        })
        .unwrap();
    rejected_with_feedback(
        def,
        &wrong,
        "protected optional helper was silently repaired",
    );
}
#[test]
fn both_preparation_tasks_reject_missing_saved_order_despite_working_manual_current_start() {
    let labs = catalog();
    let review = plans();
    for index in [1, 3] {
        let p = &review[index];
        let def = definition(labs.iter().find(|s| s.id == p.id).unwrap());
        let prep = def
            .initial
            .inventory()
            .units
            .iter()
            .find(|u| u.contains("-prepare."))
            .unwrap()
            .clone();
        let app = def
            .initial
            .inventory()
            .units
            .iter()
            .find(|u| u.contains("-app."))
            .unwrap()
            .clone();
        let mut actions = p.reference.clone();
        let EditAction::Save { draft } = &mut actions[0] else {
            panic!()
        };
        let text = draft.documents[&app].replace(&format!("After={prep}\n"), "");
        draft.documents.insert(app.clone(), text);
        actions.insert(2, manager(Action::Start { unit: prep }));
        let state = run(def, &actions);
        assert!(state.result(&app).unwrap().is_some());
        rejected_with_feedback(def, &state, "manual order cannot replace saved ordering");
        let checks = ws::checks(def, &state, &Draft::new(&state).unwrap());
        assert!(
            checks
                .iter()
                .filter(|(_, m)| !m.starts_with("Fresh session:"))
                .all(|r| r.0),
            "Current/preservation failure masked the next-session-only guarantee"
        );
        assert!(checks.iter().any(|(ok, m)| !ok
            && m.starts_with("Fresh session:")
            && m.contains("actual transaction ordering path")));
    }
}
#[test]
fn adding_supported_but_preserved_absent_preparation_link_fails_without_damaging_current_work() {
    let labs = catalog();
    let review = plans();
    for index in [1, 3] {
        let p = &review[index];
        let def = definition(labs.iter().find(|s| s.id == p.id).unwrap());
        let mut state = run(def, &p.reference);
        let prep = state
            .inventory()
            .units
            .iter()
            .find(|u| u.contains("-prepare."))
            .unwrap()
            .clone();
        assert!(
            def.initial
                .boot_links()
                .values()
                .all(|units| !units.contains(&prep))
        );
        assert!(
            def.preservation
                .editable_boot_links
                .iter()
                .all(|link| link.unit != prep)
        );
        state.apply(&Action::Enable { unit: prep }).unwrap();
        let checks = ws::checks(def, &state, &Draft::new(&state).unwrap());
        assert!(
            checks[1..=def.requirements.len()].iter().all(|r| r.0),
            "Current work was damaged rather than isolated preservation failure"
        );
        assert!(
            checks
                .iter()
                .any(|(ok, m)| !ok && m.contains("enablement") && m.contains("unchanged"))
        );
        rejected_with_feedback(def, &state, "added previously absent protected boot link");
    }
}
fn terminal_sequence(
    def: &Definition,
    actions: &[EditAction],
) -> dlms_learning_lab::unit_manager::State {
    let mut state = def.initial.clone();
    terminal_actions(def, &mut state, actions);
    state
}
fn terminal_actions(
    def: &Definition,
    state: &mut dlms_learning_lab::unit_manager::State,
    actions: &[EditAction],
) {
    for action in actions {
        let command = match action {
            EditAction::Save { draft } => {
                draft.apply(state).unwrap();
                continue;
            }
            EditAction::Manager {
                action: Action::AdvancePrograms {},
            } => continue,
            EditAction::Manager { action } => match action {
                Action::DaemonReload {} => "systemctl daemon-reload".to_owned(),
                Action::Start { unit } => format!("systemctl start {unit}"),
                Action::Stop { unit } => format!("systemctl stop {unit}"),
                Action::Enable { unit } => format!("systemctl enable {unit}"),
                Action::Disable { unit } => format!("systemctl disable {unit}"),
                Action::SetDefault { target } => format!("systemctl set-default {target}"),
                _ => panic!("Unsupported native reference mapping"),
            },
        };
        let result = unit_manager_terminal::execute(def, state, &command).unwrap();
        assert_eq!(result.status, 0, "{command}");
    }
}
#[test]
fn retry_after_an_early_start_reruns_active_preparation_with_corrected_input() {
    let labs = catalog();
    for plan in plans().into_iter().filter(|p| {
        matches!(
            p.id.as_str(),
            "lpic2-unit-preparation-order" | "lpic2-unit-target-handover"
        )
    }) {
        let def = definition(labs.iter().find(|s| s.id == plan.id).unwrap());
        let mut state = def.initial.clone();
        let app = state
            .documents()
            .keys()
            .find(|u| u.ends_with("-app.service"))
            .unwrap()
            .clone();
        let prepare = state
            .documents()
            .keys()
            .find(|u| u.ends_with("-prepare.service"))
            .unwrap()
            .clone();
        unit_manager_terminal::execute(def, &mut state, &format!("systemctl start {app}")).unwrap();
        let previous = state.runtime()[&prepare].clone();
        assert!(matches!(
            previous,
            dlms_learning_lab::unit_manager::Runtime::Exited { .. }
        ));
        terminal_actions(def, &mut state, &plan.reference);
        assert_eq!(
            state.runtime()[&prepare],
            previous,
            "start must not silently rerun an active oneshot"
        );
        rejected_with_feedback(
            def,
            &state,
            "saved repair cannot replace earlier preparation output",
        );
        for unit in [&app, &prepare] {
            unit_manager_terminal::execute(def, &mut state, &format!("systemctl stop {unit}"))
                .unwrap();
        }
        unit_manager_terminal::execute(def, &mut state, &format!("systemctl start {app}")).unwrap();
        assert!(
            passed(def, &state),
            "explicit recovery must work for {}",
            plan.id
        );
        for alternative in &plan.alternates {
            for unit in [&app, &prepare] {
                unit_manager_terminal::execute(def, &mut state, &format!("systemctl stop {unit}"))
                    .unwrap();
            }
            terminal_actions(def, &mut state, alternative);
            assert!(
                passed(def, &state),
                "alternate must work after prior attempts: {}",
                plan.id
            );
        }
    }
}
#[test]
fn native_terminal_controller_matches_typed_reference_and_alternates_with_automatic_initialization()
{
    let labs = catalog();
    for p in plans() {
        let def = definition(labs.iter().find(|s| s.id == p.id).unwrap());
        for actions in std::iter::once(&p.reference).chain(&p.alternates) {
            let typed = run(def, actions);
            let terminal = terminal_sequence(def, actions);
            assert_eq!(terminal, typed);
            assert!(passed(def, &terminal));
        }
    }
    let p = plans().remove(2);
    let def = definition(labs.iter().find(|s| s.id == p.id).unwrap());
    let state = terminal_sequence(def, &p.reference);
    assert_eq!(
        state.result("portal-app.service").unwrap().unwrap().port,
        Some(8282)
    );
    assert!(matches!(
        state.runtime().get("portal-optional.service"),
        Some(dlms_learning_lab::unit_manager::Runtime::Failed { .. })
    ));
    // The weak helper actually failed while the requested application start returned0.
    let mut state = state;
    let result =
        unit_manager_terminal::execute(def, &mut state, "systemctl start portal-optional.service")
            .unwrap();
    assert_eq!(result.status, 1);
    assert!(passed(def, &state));
    for input in [
        "systemctl status portal-app.service",
        "systemctl is-active portal-optional.service",
        "journalctl -u portal-optional.service -o cat -n 4",
    ] {
        let before = state.clone();
        unit_manager_terminal::execute(def, &mut state, input).unwrap();
        assert_eq!(state, before);
    }
}
#[test]
fn references_alternates_and_plain_failure_feedback_use_actual_current_and_fresh_state() {
    let labs = catalog();
    for plan in plans() {
        let lab = labs.iter().find(|s| s.id == plan.id).unwrap();
        let def = definition(lab);
        assert!(!passed(def, &def.initial));
        for sequence in std::iter::once(&plan.reference).chain(&plan.alternates) {
            let state = run(def, sequence);
            assert!(passed(def, &state), "{}", lab.id);
        }
        for negative in plan.negative_sequences {
            let state = run(def, &negative.actions);
            let checks = ws::checks(def, &state, &Draft::new(&state).unwrap());
            assert!(
                checks
                    .iter()
                    .any(|(ok, text)| !ok && !text.trim().is_empty()),
                "{}: {}",
                lab.id,
                negative.name
            );
            assert!(!passed(def, &state));
        }
    }
}
#[test]
fn preparation_started_by_hand_cannot_replace_saved_ordering_and_weak_helper_must_remain_requested()
{
    let labs = catalog();
    let review = plans();
    let p = &review[1];
    let def = definition(labs.iter().find(|s| s.id == p.id).unwrap());
    let mut actions = p.reference.clone();
    let EditAction::Save { draft } = &mut actions[0] else {
        panic!()
    };
    let source =
        draft.documents["archive-app.service"].replace("After=archive-prepare.service\n", "");
    draft.documents.insert("archive-app.service".into(), source);
    actions.insert(
        2,
        manager(Action::Start {
            unit: "archive-prepare.service".into(),
        }),
    );
    let state = run(def, &actions);
    assert_eq!(
        state.result("archive-app.service").unwrap().unwrap().port,
        Some(8181)
    );
    assert!(!passed(def, &state));
    let p = &review[2];
    let def = definition(labs.iter().find(|s| s.id == p.id).unwrap());
    for replace in ["Requires=portal-optional.service\n", ""] {
        let mut actions = p.reference.clone();
        let EditAction::Save { draft } = &mut actions[0] else {
            panic!()
        };
        let source = draft.documents["portal-app.service"]
            .replace("Wants=portal-optional.service\n", replace);
        draft.documents.insert("portal-app.service".into(), source);
        assert!(!passed(def, &run(def, &actions)));
    }
}
#[test]
fn handover_cannot_leave_old_target_active_in_next_session_or_old_service_enabled() {
    let labs = catalog();
    let p = plans().pop().unwrap();
    let def = definition(labs.iter().find(|s| s.id == p.id).unwrap());
    let mut wrong = p.reference.clone();
    let EditAction::Save { draft } = &mut wrong[0] else {
        panic!()
    };
    let source = draft.documents["publish-app.service"]
        .replace("[Unit]\n", "[Unit]\nWants=publish-spare.target\n");
    draft.documents.insert("publish-app.service".into(), source);
    assert!(!passed(def, &run(def, &wrong)));
    let mut wrong = run(def, &p.reference);
    wrong
        .apply(&Action::Enable {
            unit: "publish-old.service".into(),
        })
        .unwrap();
    assert!(!passed(def, &wrong));
}
#[test]
fn separate_examples_fill_only_then_work_experiment_and_repair_without_assignment_access() {
    let labs = catalog();
    for id in IDS {
        let lab = labs.iter().find(|s| s.id == id).unwrap();
        let lesson = lab.unit_manager_teaching.as_ref().unwrap();
        assert!(lesson.distinct_from(&definition(lab).initial));
        let assignment = definition(lab).initial.clone();
        let mut learning = Learning::new(lesson).unwrap();
        let before = learning.state.clone();
        learning.fill_worked(lesson).unwrap();
        assert_eq!(learning.state, before);
        assert_eq!(assignment, definition(lab).initial);
        assert!(!learning.draft.matches(&learning.state));
        for annotated in &lesson.example_actions {
            learning.execute(lesson, &annotated.action).unwrap();
        }
        assert!(passed(&lesson.definition, &learning.state));
        for action in &lesson.try_actions {
            learning.execute(lesson, action).unwrap();
        }
        assert!(!passed(&lesson.definition, &learning.state));
        for action in &lesson.repair {
            learning.execute(lesson, action).unwrap();
        }
        assert!(passed(&lesson.definition, &learning.state));
        assert_eq!(assignment, definition(lab).initial);
    }
}
#[test]
fn edited_pending_text_and_old_checks_cannot_hold_success_and_real_repair_restores_it() {
    let labs = catalog();
    for p in plans() {
        let lab = labs.iter().find(|s| s.id == p.id).unwrap();
        let def = definition(lab);
        let mut state = run(def, &p.reference);
        let mut draft = Draft::new(&state).unwrap();
        draft
            .documents
            .values_mut()
            .next()
            .unwrap()
            .push_str("\n[unfinished\n");
        let before = state.clone();
        assert!(!ws::solved(def, &state, &draft));
        let mut invalid_saved = state.clone();
        draft.apply(&mut invalid_saved).unwrap();
        assert!(invalid_saved.validate_documents().is_err());
        assert!(!passed(def, &invalid_saved));
        assert_eq!(state, before);
        let observer = state
            .inventory()
            .units
            .iter()
            .find(|u| u.contains("-observer."))
            .unwrap()
            .clone();
        state
            .apply(&Action::Stop {
                unit: observer.clone(),
            })
            .unwrap();
        assert!(!passed(def, &state));
        state.apply(&Action::Start { unit: observer }).unwrap();
        state.apply(&Action::AdvancePrograms {}).unwrap();
        assert!(passed(def, &state));
    }
}
#[derive(Deserialize)]
struct Historical {
    id: String,
    record: shell::Record,
}
#[test]
fn real_old_catalog_runs_history_drafts_favorites_and_selected_reset_survive_roundtrip_and_restore()
{
    assert_eq!(
        shell::EVALUATOR_REVISION,
        7,
        "Never execute a later VM and label its observations7"
    );
    let mut all: Vec<_> = pack::bundled()
        .unwrap()
        .into_iter()
        .filter(|s| !IDS.contains(&s.id.as_str()))
        .collect();
    let original_ids: Vec<_> = all.iter().map(|s| s.id.clone()).collect();
    let review: Review =
        serde_json::from_str(include_str!("fixtures/unit-manager-content-review.json")).unwrap();
    assert_eq!(
        original_ids.iter().collect::<BTreeSet<_>>(),
        review.original_catalog_ids.iter().collect::<BTreeSet<_>>()
    );
    let mut p = Progress {
        schema_version: 1,
        clock: 4000,
        ..Default::default()
    };
    let frozen: Vec<Historical> =
        serde_json::from_str(include_str!("fixtures/shell-v6-authored-records.json")).unwrap();
    for (i, lab) in all.iter().enumerate() {
        let mut a = Attempt::new(lab, i as u64 + 1);
        if let Goal::Script { cases } = &lab.goal {
            assert_eq!(lab.solution.len(), 1);
            let source = lab.solution[0].clone();
            let results = cases.iter().map(|case| shell::run(&source, case)).collect();
            a.state = State::Script {
                source: source.clone(),
                run: Some(shell::Record {
                    engine_revision: shell::EVALUATOR_REVISION,
                    source,
                    results,
                }),
            };
        } else {
            for command in &lab.solution {
                engine::execute_for_goal(&mut a.state, &lab.goal, command).unwrap();
            }
        }
        assert!(
            engine::solved(&a.state, &lab.goal),
            "Original reference {}",
            lab.id
        );
        workspace::prepare(&mut a);
        if let Some(lesson) = &lab.teaching {
            dlms_learning_lab::script_teaching::prepare(&mut a, lesson, true);
            let l = a
                .workspace
                .as_mut()
                .unwrap()
                .script_learning
                .as_mut()
                .unwrap();
            l.example_result = Some(shell::run(&lesson.example, &lesson.fixture));
            l.example_revision = 7;
        }
        if let Some(lesson) = &lab.acl_teaching {
            dlms_learning_lab::acl_teaching::prepare(&mut a, lesson, true).unwrap();
            let w = a.workspace.as_mut().unwrap();
            let l = w.acl_learning.as_mut().unwrap();
            l.fill_example(lesson).unwrap();
            assert!(l.apply_check(lesson).unwrap().iter().all(|r| r.0));
            l.draft
                .documents
                .routers
                .values_mut()
                .next()
                .unwrap()
                .push_str("\nunfinished separate example\n");
            w.acl_draft
                .as_mut()
                .unwrap()
                .documents
                .routers
                .values_mut()
                .next()
                .unwrap()
                .push_str("\nunfinished assignment\n");
        }
        if let Some(lesson) = &lab.accounts_teaching {
            dlms_learning_lab::accounts_teaching::prepare(&mut a, lesson, true);
            let w = a.workspace.as_mut().unwrap();
            w.command = "usermod -G".into();
            let l = w.accounts_learning.as_mut().unwrap();
            for command in &lesson.example {
                l.command = command.clone();
                l.execute(lesson).unwrap();
            }
            l.command = "usermod -g".into();
        }
        if let Some(lesson) = &lab.storage_teaching {
            dlms_learning_lab::storage_teaching::prepare(&mut a, lesson, true);
            let w = a.workspace.as_mut().unwrap();
            w.storage_draft.as_mut().unwrap().text = "unfinished assignment\n".into();
            let l = w.storage_learning.as_mut().unwrap();
            l.fill_example(lesson).unwrap();
            l.draft.text.push_str("unfinished separate example\n");
        }
        if let Some(lesson) = &lab.ospf_teaching {
            dlms_learning_lab::ospf_teaching::prepare(&mut a, lesson, true).unwrap();
            let w = a.workspace.as_mut().unwrap();
            w.ospf_draft
                .as_mut()
                .unwrap()
                .routers
                .values_mut()
                .next()
                .unwrap()
                .process_id
                .clear();
            let l = w.ospf_learning.as_mut().unwrap();
            l.fill_example(lesson).unwrap();
            assert!(l.apply_check(lesson).unwrap().iter().all(|r| r.0));
            l.draft.routers.values_mut().next().unwrap().area =
                "unfinished separate example".into();
        }
        if let Some(old) = frozen.iter().find(|h| h.id == lab.id) {
            a.workspace.as_mut().unwrap().script_history = Some(old.record.clone());
        }
        p.favorites.insert(lab.id.clone());
        p.attempts.insert(lab.id.clone(), a);
    }
    let original: BTreeMap<_, _> = p
        .attempts
        .iter()
        .map(|(id, a)| (id.clone(), serde_json::to_value(a).unwrap()))
        .collect();
    let staged = catalog();
    let lab = staged.iter().find(|s| s.id == IDS[0]).unwrap().clone();
    let mut a = Attempt::new(&lab, 3000);
    workspace::prepare(&mut a);
    let State::UnitManager { simulation } = &mut a.state else {
        panic!()
    };
    *simulation = run(definition(&lab), &plans()[0].reference);
    let w = a.workspace.as_mut().unwrap();
    w.unit_manager_draft = Some(Draft::new(simulation).unwrap());
    w.unit_manager_draft
        .as_mut()
        .unwrap()
        .documents
        .values_mut()
        .next()
        .unwrap()
        .push_str("\nunfinished assignment\n");
    let lesson = lab.unit_manager_teaching.as_ref().unwrap();
    let mut learning = Learning::new(lesson).unwrap();
    learning.fill_worked(lesson).unwrap();
    learning
        .draft
        .documents
        .values_mut()
        .next()
        .unwrap()
        .push_str("\nunfinished separate example\n");
    w.unit_manager_learning = Some(learning);
    let expected = serde_json::to_value(&a).unwrap();
    p.attempts.insert(lab.id.clone(), a);
    all.push(lab.clone());
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("progress.json");
    progress::save(&target, &p).unwrap();
    let loaded = progress::load(&target, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&lab.id]).unwrap(),
        expected
    );
    let export = dir.path().join("export.json");
    backup::export(&export, &loaded, &all).unwrap();
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    let restored = dir.path().join("restored.json");
    backup::restore(&restored, &candidate, &all).unwrap();
    let mut loaded = progress::load(&restored, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&lab.id]).unwrap(),
        expected
    );
    loaded
        .attempts
        .insert(lab.id.clone(), Attempt::new(&lab, 3999));
    for id in original_ids {
        assert_eq!(
            serde_json::to_value(&loaded.attempts[&id]).unwrap(),
            original[&id],
            "{id}"
        );
        assert!(loaded.favorites.contains(&id));
    }
    assert_eq!(loaded.attempts[&lab.id].state, lab.initial);
    assert!(!progress::current_solved(&loaded.attempts[&lab.id], &lab));
    assert_eq!(shell::EVALUATOR_REVISION, 7);
}
