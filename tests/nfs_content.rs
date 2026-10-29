//! Registered authored NFS pack. Independent source loading retains all reviewed outcomes.
//! Expected vectors are authored independently, never generated from the evaluator.
#[path = "support/combined_catalog.rs"]
mod combined_catalog;
#[path = "../examples/support/automation_digest.rs"]
mod digest;
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
#[path = "support/one_off_registration.rs"]
mod one_off_registration;
use dlms_learning_lab::{
    backup, course,
    engine::{self, State},
    nfs as core, nfs_teaching as teach, nfs_workspace as work, pack,
    progress::{self, Attempt},
};
use serde::Deserialize;
use std::sync::atomic::{AtomicBool, AtomicUsize};
const IDS: [&str; 4] = [
    "lpic2-nfs-reference",
    "lpic2-nfs-publication",
    "lpic2-nfs-handover",
    "lpic2-nfs-capstone",
];
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    name: String,
    actions: Vec<core::Action>,
    rows: Vec<bool>,
    outcomes: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Review {
    id: String,
    reason: String,
    initial: Vec<bool>,
    reference: Plan,
    alternatives: Vec<Plan>,
    wrong: Plan,
    partial: Plan,
    no_op: Plan,
    read_only: Plan,
    after_pass_break: Plan,
    repair: Plan,
    practice_initial: Vec<bool>,
    practice_worked: Vec<bool>,
    practice_adverse: Vec<bool>,
    preservation: Vec<work::Requirement>,
    fault_probe: core::Action,
    current_vs_fresh_break: Vec<bool>,
    extra_known_reader: Option<Plan>,
    server_rw: Option<Plan>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reviews {
    schema_version: u32,
    expected_origin: String,
    lessons: Vec<Review>,
}
fn labs() -> Vec<pack::Scenario> {
    static ALL: std::sync::OnceLock<Vec<pack::Scenario>> = std::sync::OnceLock::new();
    ALL.get_or_init(|| {
        pack::load_catalog(&[
            include_str!("../scenarios/starter-v1.json"),
            include_str!("../scenarios/linux-files-v1.json"),
            include_str!("../scenarios/storage-v1.json"),
            include_str!("../scenarios/nfs-v1.json"),
        ])
        .unwrap()
        .into_iter()
        .filter(|s| IDS.contains(&s.id.as_str()))
        .collect()
    })
    .clone()
}
fn reviews() -> Vec<Review> {
    let r: Reviews = serde_json::from_str(include_str!("fixtures/nfs-review.json")).unwrap();
    assert_eq!(r.schema_version, 1);
    assert!(r.expected_origin.contains("not recorded"));
    assert_eq!(r.lessons.len(), 4);
    for (r, id) in r.lessons.iter().zip(IDS) {
        assert_eq!(r.id, id);
        assert!(!r.reason.is_empty());
        assert_eq!(r.alternatives.len(), 2);
        for p in std::iter::once(&r.reference).chain(&r.alternatives).chain([
            &r.wrong,
            &r.partial,
            &r.no_op,
            &r.read_only,
            &r.after_pass_break,
            &r.repair,
        ]) {
            assert!(!p.name.is_empty());
            assert_eq!(p.actions.len(), p.outcomes.len());
            assert!(p.actions.len() <= 16);
            assert_eq!(p.rows.len(), r.initial.len());
        }
    }
    r.lessons
}
fn definition(s: &pack::Scenario) -> &work::Definition {
    let engine::Goal::Nfs { definition } = &s.goal else {
        panic!("Actual typed NFS source")
    };
    definition
}
fn simulation(a: &Attempt) -> &core::State {
    let State::Nfs { simulation } = &a.state else {
        panic!("Actual NFS State")
    };
    simulation
}
fn exact<A: serde::Serialize + ?Sized, B: serde::Serialize + ?Sized>(a: &A, b: &B) {
    assert!(
        serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap(),
        "Complete value differs"
    );
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 7);
    teach::prepare(&mut a, s, true).unwrap();
    a
}
fn rows(a: &Attempt, s: &pack::Scenario) -> Vec<bool> {
    progress::current_nfs_checks(a, s)
        .iter()
        .map(|r| r.0)
        .collect()
}
fn kind(d: &core::Decision) -> String {
    match d {
        core::Decision::Changed { .. } => "changed".into(),
        core::Decision::Mounted {} => "mounted".into(),
        core::Decision::Unmounted {} => "unmounted".into(),
        core::Decision::AllowedRead { .. } => "read".into(),
        core::Decision::AllowedWrite { .. } => "write".into(),
        core::Decision::Denied { stage, .. } => format!(
            "denied:{}",
            serde_json::to_value(stage).unwrap().as_str().unwrap()
        ),
    }
}
fn perform(a: &mut Attempt, s: &pack::Scenario, p: &Plan) {
    for (action, expected) in p.actions.iter().zip(&p.outcomes) {
        let result = work::perform_attempt(a, s, action).unwrap();
        assert_eq!(&kind(&result), expected, "{} / {action:?}", p.name);
    }
    assert_eq!(rows(a, s), p.rows, "{}", p.name);
    assert_eq!(progress::current_solved(a, s), p.rows.iter().all(|v| *v));
}
fn preserve(a: &Attempt, s: &pack::Scenario, requirements: &[work::Requirement]) {
    let current = simulation(a);
    let initial = &definition(s).initial;
    exact(current.inventory(), initial.inventory());
    for r in requirements {
        match r {
            work::Requirement::PreserveInitialMount { client, mountpoint } => {
                let get = |state: &core::State| {
                    state
                        .runtime()
                        .mounts
                        .iter()
                        .find(|m| &m.client == client && &m.specification.mountpoint == mountpoint)
                        .cloned()
                };
                exact(&get(current), &get(initial));
                assert!(get(initial).is_some());
            }
            work::Requirement::PreserveInitialFile { server, path } => {
                let get = |state: &core::State| {
                    state
                        .runtime()
                        .contents
                        .iter()
                        .find(|f| &f.server == server && &f.path == path)
                        .cloned()
                };
                exact(&get(current), &get(initial));
                assert!(get(initial).is_some());
            }
            work::Requirement::PreserveInitialClientLayers { client } => {
                let get = |state: &core::State| {
                    state
                        .runtime()
                        .clients
                        .iter()
                        .find(|c| &c.client == client)
                        .cloned()
                };
                exact(&get(current), &get(initial));
            }
            work::Requirement::PreserveInitialServerLayers { server } => {
                let get = |state: &core::State| {
                    state
                        .runtime()
                        .servers
                        .iter()
                        .find(|s| &s.server == server)
                        .cloned()
                };
                exact(&get(current), &get(initial));
            }
            _ => panic!("Review preservation must be an independent unchanged-value assertion"),
        }
    }
}
#[test]
fn prerequisites_guides_and_six_or_fewer_concepts_precede_real_native_actions() {
    let normal = pack::bundled().unwrap();
    assert_eq!(one_off_registration::original_catalog(&normal).len(), 176);
    let raw46 =
        one_off_registration::before_units(include_bytes!("../scenarios/units-v1.json")).unwrap();
    let units = course::load(std::str::from_utf8(&raw46).unwrap(), &normal[..176]).unwrap();
    assert_eq!(units.len(), 46);
    let raw44 = original44_unit_bytes();
    let original44 =
        dlms_learning_lab::course::load(std::str::from_utf8(&raw44).unwrap(), &normal[..171])
            .unwrap();
    assert_eq!(original44.len(), 44);
    assert_eq!(
        serde_json::to_value(&units[..44]).unwrap(),
        serde_json::to_value(&original44).unwrap()
    );
    assert_eq!(original44.last().unwrap().id, "shell-numeric-validation");
    assert!(normal[..147].iter().all(|s| !IDS.contains(&s.id.as_str())));
    let all = labs();
    assert_eq!(all.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), IDS);
    exact(&normal[150..154], &all);
    let known = normal
        .iter()
        .chain(&all)
        .map(|s| s.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        all[0].prerequisites,
        ["lpic2-mount-policy", "lpic1-traversal"]
    );
    for (n, s) in all.iter().enumerate() {
        assert_eq!(s.version, 1);
        assert_eq!(s.track, "LPIC-2");
        assert_eq!(s.unit, "nfs-network-file-sharing");
        assert!(s.prerequisites.iter().all(|p| known.contains(&p.as_str())));
        if n > 0 {
            assert_eq!(s.prerequisites, [IDS[n - 1]]);
        }
        assert_eq!(s.steps.len(), 3);
        assert_eq!(s.guide.len(), 3);
        assert!(s.guide.iter().all(|g| g.commands.is_empty()));
        let l = s.nfs_teaching.as_ref().unwrap();
        assert!((3..=6).contains(&l.concepts.len()));
        assert!(l.separate_from(&definition(s).initial));
        assert!(
            l.concepts
                .iter()
                .all(|c| !c.title.is_empty() && c.text.len() <= 1200)
        );
        assert!(s.hints.len() >= 3);
        assert!(s.solution.iter().all(|c| c.starts_with(work::PREFIX)));
        assert!(s.guide.iter().all(|g| !g.why.contains("nfs {")));
        let r = &reviews()[n];
        assert_eq!(r.initial.len(), definition(s).row_count());
        assert_eq!(
            r.preservation.len(),
            definition(s)
                .requirements
                .iter()
                .filter(|r| matches!(
                    r,
                    work::Requirement::PreserveInitialMount { .. }
                        | work::Requirement::PreserveInitialFile { .. }
                        | work::Requirement::PreserveInitialClientLayers { .. }
                        | work::Requirement::PreserveInitialServerLayers { .. }
                ))
                .count()
        );
    }
    let inventories = all
        .iter()
        .map(|s| definition(s).initial.inventory())
        .collect::<Vec<_>>();
    assert_ne!(inventories[0].profiles.len(), inventories[1].profiles.len());
    assert_ne!(inventories[1].clients.len(), inventories[2].clients.len());
    assert_eq!(inventories[3].servers.len(), 2);
    assert!(inventories[..3].iter().all(|i| i.servers.len() == 1));
}
#[test]
fn every_other_supplied_client_is_denied_and_server_ro_cannot_hide_behind_client_ro_or_dac() {
    for (n, (s, r)) in labs().iter().zip(reviews()).enumerate() {
        if n != 0 && n != 2 {
            assert!(r.extra_known_reader.is_none() && r.server_rw.is_none());
            continue;
        }
        let d = definition(s);
        assert_eq!(d.fresh.len(), 4);
        let authorized = d
            .requirements
            .iter()
            .find_map(|r| {
                if let work::Requirement::CurrentMount { client, .. } = r {
                    Some(client.as_str())
                } else {
                    None
                }
            })
            .unwrap();
        let expected_clients = d
            .initial
            .inventory()
            .clients
            .iter()
            .filter(|c| c.id != authorized)
            .map(|c| c.id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        let denied_clients = d.fresh[1]
            .steps
            .iter()
            .map(|step| {
                if let core::Action::MountExplicit { client, .. } = &step.action {
                    client.as_str()
                } else {
                    panic!("Every supplied-client admission probe is a real mount")
                }
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(denied_clients, expected_clients);
        assert_eq!(d.fresh[1].steps.len(), expected_clients.len());
        // Declared client/target routes must produce actual denials rather than unsupported faults.
        let mut positive = fresh(s);
        perform(&mut positive, s, &r.reference);
        let correct = simulation(&positive)
            .evaluate_batch(&core::Evaluation {
                current: vec![],
                fresh: d.fresh.clone(),
            })
            .unwrap();
        assert!(correct.fresh[1].steps.iter().all(|step| step.passed
            && matches!(
                &step.observation,
                core::Observation::Decision {
                    decision: core::Decision::Denied {
                        stage: core::DenialStage::ExportAdmission,
                        ..
                    }
                }
            )));
        for bad in [
            r.extra_known_reader.as_ref().unwrap(),
            r.server_rw.as_ref().unwrap(),
        ] {
            let mut a = fresh(s);
            perform(&mut a, s, bad);
            assert!(!progress::current_solved(&a, s));
            preserve(&a, s, &r.preservation);
            let before = serde_json::to_value(&a).unwrap();
            let report = simulation(&a)
                .evaluate_batch(&core::Evaluation {
                    current: vec![],
                    fresh: d.fresh.clone(),
                })
                .unwrap();
            if bad.name.contains("extra supplied") {
                assert!(!report.fresh[1].passed);
                assert!(report.fresh[1].steps.iter().any(|step| matches!(
                    &step.observation,
                    core::Observation::Decision {
                        decision: core::Decision::Mounted {}
                    }
                )));
            } else {
                let probe = report.fresh.last().unwrap();
                assert!(!probe.passed);
                assert!(probe.steps[0].passed);
                assert!(!probe.steps[1].passed);
                if n == 0 {
                    assert!(
                        matches!(
                            &probe.steps[1].observation,
                            core::Observation::Decision {
                                decision: core::Decision::Denied {
                                    stage: core::DenialStage::FilePermission,
                                    ..
                                }
                            }
                        ),
                        "A DAC denial must not substitute for required export_read_only"
                    );
                } else {
                    assert!(matches!(
                        &probe.steps[1].observation,
                        core::Observation::Decision {
                            decision: core::Decision::AllowedWrite { .. }
                        }
                    ));
                }
            }
            exact(&a, &before);
        }
    }
}
#[test]
fn initial_noop_readonly_wrong_and_partial_have_independent_full_vectors() {
    for (s, r) in labs().iter().zip(reviews()) {
        let a = fresh(s);
        assert_eq!(rows(&a, s), r.initial);
        assert!(!progress::current_solved(&a, s));
        for p in [&r.no_op, &r.read_only, &r.wrong, &r.partial] {
            let mut a = fresh(s);
            perform(&mut a, s, p);
            preserve(&a, s, &r.preservation);
            assert!(!progress::current_solved(&a, s));
        }
    }
}
#[test]
fn taught_references_and_two_distinct_supported_alternatives_satisfy_actual_goal_semantics() {
    for (s, r) in labs().iter().zip(reviews()) {
        let encoded = s
            .solution
            .iter()
            .map(|c| core::Action::from_json(c.strip_prefix(work::PREFIX).unwrap()).unwrap())
            .collect::<Vec<_>>();
        exact(&encoded, &r.reference.actions);
        for p in std::iter::once(&r.reference).chain(&r.alternatives) {
            let mut a = fresh(s);
            perform(&mut a, s, p);
            preserve(&a, s, &r.preservation);
            assert!(progress::current_solved(&a, s));
            let mut raw = s.initial.clone();
            for c in &s.solution {
                engine::execute_for_goal(&mut raw, &s.goal, c).unwrap();
            }
            assert!(engine::solved(&raw, &s.goal));
        }
        assert!(
            r.alternatives[0].actions != r.reference.actions
                && r.alternatives[1].actions != r.reference.actions
                && r.alternatives[0].actions != r.alternatives[1].actions
        );
    }
}
#[test]
fn actual_success_is_revoked_by_current_or_future_changes_and_can_be_repaired() {
    for (s, r) in labs().iter().zip(reviews()) {
        let mut a = fresh(s);
        perform(&mut a, s, &r.reference);
        perform(&mut a, s, &r.after_pass_break);
        assert_eq!(rows(&a, s), r.current_vs_fresh_break);
        preserve(&a, s, &r.preservation);
        perform(&mut a, s, &r.repair);
        assert!(progress::current_solved(&a, s));
        preserve(&a, s, &r.preservation);
    }
}
#[test]
fn worked_practice_adverse_experiment_and_repair_never_change_assignment_or_other_attempt_fields() {
    for (s, r) in labs().iter().zip(reviews()) {
        let mut a = fresh(s);
        perform(&mut a, s, &r.reference);
        let before = serde_json::to_value(&a).unwrap();
        let source = s.nfs_teaching.as_ref().unwrap();
        assert_eq!(
            a.workspace
                .as_ref()
                .unwrap()
                .nfs_learning
                .as_ref()
                .unwrap()
                .checks(source)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            r.practice_initial
        );
        for action in &source.example_actions {
            work::perform_practice(&mut a, s, &action.action).unwrap();
        }
        assert_eq!(
            a.workspace
                .as_ref()
                .unwrap()
                .nfs_learning
                .as_ref()
                .unwrap()
                .checks(source)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            r.practice_worked
        );
        for action in &source.try_change.actions {
            work::perform_practice(&mut a, s, action).unwrap();
        }
        assert_eq!(
            a.workspace
                .as_ref()
                .unwrap()
                .nfs_learning
                .as_ref()
                .unwrap()
                .checks(source)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            r.practice_adverse
        );
        assert!(
            progress::current_solved(&a, s),
            "Separate adverse practice cannot revoke applied assignment credit"
        );
        for action in &source.repair {
            work::perform_practice(&mut a, s, action).unwrap();
        }
        assert!(
            a.workspace
                .as_ref()
                .unwrap()
                .nfs_learning
                .as_ref()
                .unwrap()
                .checks(source)
                .iter()
                .all(|r| r.0)
        );
        let mut expected = before;
        expected["workspace"]["nfs_learning"] =
            serde_json::to_value(&a.workspace.as_ref().unwrap().nfs_learning).unwrap();
        exact(&a, &expected);
    }
}
#[test]
fn fresh_trials_report_outcomes_without_installing_their_mounts_or_probe_file_writes() {
    for (s, r) in labs().iter().zip(reviews()) {
        let mut a = fresh(s);
        perform(&mut a, s, &r.reference);
        let before = serde_json::to_value(&a).unwrap();
        let report = simulation(&a)
            .evaluate_batch(&core::Evaluation {
                current: vec![],
                fresh: definition(s).fresh.clone(),
            })
            .unwrap();
        assert!(report.fresh.iter().all(|t| t.passed));
        for _ in 0..2 {
            assert!(progress::current_solved(&a, s));
        }
        exact(&a, &before);
        // No text-matching history credential: an unsolved State remains unsolved with a copied checked flag.
        let mut unsolved = fresh(s);
        unsolved.workspace.as_mut().unwrap().checked = true;
        unsolved
            .workspace
            .as_mut()
            .unwrap()
            .nfs_session
            .as_mut()
            .unwrap()
            .entries = a
            .workspace
            .as_ref()
            .unwrap()
            .nfs_session
            .as_ref()
            .unwrap()
            .entries
            .clone();
        assert!(!progress::current_solved(&unsolved, s));
    }
}
#[test]
fn malformed_or_foreign_requests_are_faults_not_expected_denial_and_are_atomic() {
    for (s, r) in labs().iter().zip(reviews()) {
        let mut a = fresh(s);
        perform(&mut a, s, &r.reference);
        let before = serde_json::to_value(&a).unwrap();
        assert!(work::perform_attempt(&mut a, s, &r.fault_probe).is_err());
        exact(&a, &before);
        let mut probe = r.fault_probe.clone();
        if let core::Action::ReadExisting {
            client, relative, ..
        } = &mut probe
        {
            *client = "absent-client".into();
            *relative = "notice.txt".into();
        }
        let plan = core::Evaluation {
            current: vec![],
            fresh: vec![core::Trial {
                base: core::TrialBase::Applied {},
                steps: vec![core::TrialStep {
                    action: probe,
                    expected: core::Expected::Denied {
                        stage: core::DenialStage::ExportAdmission,
                    },
                }],
            }],
        };
        let report = simulation(&a).evaluate_batch(&plan).unwrap();
        assert!(!report.fresh[0].passed);
        assert!(!report.fresh[0].steps[0].passed);
        assert!(matches!(
            report.fresh[0].steps[0].observation,
            core::Observation::Fault { .. }
        ));
        exact(&a, &before);
    }
}
#[test]
fn practice_and_other_task_hardcodes_cannot_pass_or_mutate_the_authored_assignment() {
    let all = labs();
    for (n, s) in all.iter().enumerate() {
        let mut a = fresh(s);
        let source = s.nfs_teaching.as_ref().unwrap();
        for action in &source.example_actions {
            work::perform_practice(&mut a, s, &action.action).unwrap();
        }
        let mut foreign = a.clone();
        foreign.state = State::Nfs {
            simulation: a
                .workspace
                .as_ref()
                .unwrap()
                .nfs_learning
                .as_ref()
                .unwrap()
                .simulation
                .clone(),
        };
        assert!(!progress::current_solved(&foreign, s));
        let before = serde_json::to_value(&foreign).unwrap();
        assert!(teach::prepare(&mut foreign, s, false).is_err());
        exact(&foreign, &before);
        let other = &all[(n + 1) % 4];
        let before = serde_json::to_value(&a).unwrap();
        assert!(
            work::perform_attempt(&mut a, s, &reviews()[(n + 1) % 4].reference.actions[0]).is_err()
        );
        exact(&a, &before);
        assert!(teach::prepare(&mut a, other, false).is_err());
        exact(&a, &before);
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn unfinished_saved_policy_has_specific_truth_and_failed_apply_preserves_all_work() {
    for (s, r) in labs().iter().zip(reviews()) {
        let mut a = fresh(s);
        perform(&mut a, s, &r.reference);
        let server = simulation(&a).inventory().servers[0].id.clone();
        work::perform_attempt(
            &mut a,
            s,
            &core::Action::SetExportsDraft {
                server: server.clone(),
                text: "unfinished unsupported export text".into(),
            },
        )
        .unwrap();
        work::perform_attempt(
            &mut a,
            s,
            &core::Action::SaveExportsDraft {
                server: server.clone(),
            },
        )
        .unwrap();
        let actual = progress::current_nfs_checks(&a, s);
        assert_eq!(actual.len(), definition(s).row_count());
        for (n, requirement) in definition(s).requirements.iter().enumerate() {
            if matches!(
                requirement,
                work::Requirement::CurrentMount { .. }
                    | work::Requirement::CurrentAccess { .. }
                    | work::Requirement::CurrentFileText { .. }
            ) {
                assert!(actual[n + 1].0);
            }
        }
        assert!(!progress::current_solved(&a, s));
        let before = serde_json::to_value(&a).unwrap();
        assert!(
            work::perform_attempt(&mut a, s, &core::Action::ApplySavedExports { server }).is_err()
        );
        exact(&a, &before);
    }
}
#[test]
fn two_full_raw_drafts_histories_genuine_old_records_backup_resume_and_selected_reset_are_preserved()
 {
    let all = labs();
    let catalog = pack::bundled().unwrap();
    assert_eq!(one_off_registration::original_catalog(&catalog).len(), 176);
    exact(&catalog[150..154], &all);
    let original = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let mut p = progress::decode(original, &catalog).unwrap();
    let old = serde_json::to_value(&p.attempts).unwrap();
    for (s, r) in all.iter().zip(reviews()) {
        let mut a = fresh(s);
        perform(&mut a, s, &r.reference);
        for step in &s.nfs_teaching.as_ref().unwrap().example_actions {
            work::perform_practice(&mut a, s, &step.action).unwrap();
        }
        let w = a.workspace.as_mut().unwrap();
        w.nfs_session.as_mut().unwrap().relative = "../unfinished assignment".into();
        w.nfs_session.as_mut().unwrap().write_text = "unsent assignment replacement\n".into();
        let l = w.nfs_learning.as_mut().unwrap();
        l.session.relative = "/different unfinished example".into();
        l.session.write_text = "unsent example replacement\n".into();
        l.selected_example = s.nfs_teaching.as_ref().unwrap().example_actions.len() - 1;
        l.view = dlms_learning_lab::script_teaching::View::Example;
        assert!(progress::current_solved(&a, s));
        let before = serde_json::to_value(&a).unwrap();
        teach::prepare(&mut a, s, false).unwrap();
        exact(&a, &before);
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    p.clock = 91;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.json");
    std::fs::write(&path, original).unwrap();
    progress::save(&path, &p).unwrap();
    assert_eq!(
        std::fs::read(path.with_extension(format!(
            "before-shell-v{}.json",
            dlms_learning_lab::shell::EVALUATOR_REVISION
        )))
        .unwrap(),
        original
    );
    let raw = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, &catalog).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), raw);
    exact(&loaded.attempts, &p.attempts);
    exact(&loaded.favorites, &p.favorites);
    assert_eq!(loaded.clock, 91);
    for (id, a) in old.as_object().unwrap() {
        exact(&loaded.attempts[id], a);
    }
    let backup_path = dir.path().join("portable.json");
    backup::export(&backup_path, &loaded, &catalog).unwrap();
    let candidate = backup::inspect(
        &backup_path,
        &catalog,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    for s in &all {
        exact(&candidate.progress.attempts[&s.id], &p.attempts[&s.id]);
    }
    let mut old_only = progress::decode(&raw, &catalog).unwrap();
    for s in &all {
        old_only.attempts.remove(&s.id);
        old_only.favorites.remove(&s.id);
    }
    let control_path = dir.path().join("old-only.json");
    backup::export(&control_path, &old_only, &catalog).unwrap();
    let control = backup::inspect(
        &control_path,
        &catalog,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    for (id, a) in &control.progress.attempts {
        exact(&candidate.progress.attempts[id], a);
    }
    backup::restore(&path, &candidate, &catalog).unwrap();
    let mut restored = progress::load(&path, &catalog).unwrap();
    exact(&restored.attempts, &candidate.progress.attempts);
    exact(&restored.favorites, &p.favorites);
    assert_eq!(restored.clock, 91);
    let selected = &all[0];
    let before = serde_json::to_value(&restored.attempts).unwrap();
    restored
        .attempts
        .insert(selected.id.clone(), Attempt::new(selected, 7));
    teach::prepare(
        restored.attempts.get_mut(&selected.id).unwrap(),
        selected,
        true,
    )
    .unwrap();
    for (id, a) in before.as_object().unwrap() {
        if id != &selected.id {
            exact(&restored.attempts[id], a);
        }
    }
    assert!(!progress::current_solved(
        &restored.attempts[&selected.id],
        selected
    ));
    exact(&restored.favorites, &p.favorites);
    assert_eq!(restored.clock, 91);
}

// Recover the complete reviewed44 predecessor through both closed raw inverses.
// Canonical physical/private append recipes and all historical37 bytes stay fixed.
fn original44_unit_bytes() -> Vec<u8> {
    let raw47 = include_bytes!("../scenarios/units-v1.json");
    let original46 = one_off_registration::before_units(raw47).unwrap();
    let raw = original46.as_slice();
    let before = combined_catalog::before_physical_units(raw).unwrap();
    let current: serde_json::Value = serde_json::from_slice(raw).unwrap();
    let original: serde_json::Value = serde_json::from_slice(&before).unwrap();
    assert_eq!(current["units"].as_array().unwrap().len(), 46);
    assert_eq!(original["units"].as_array().unwrap().len(), 44);
    assert_eq!(
        &current["units"].as_array().unwrap()[..44],
        original["units"].as_array().unwrap().as_slice()
    );
    let physical: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/catalog175-registration.json")).unwrap();
    let private: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/catalog176-registration.json")).unwrap();
    assert_eq!(current["units"][44], physical["physical_unit"]);
    assert_eq!(current["units"][45], private["private_unit"]);
    // Existing pinned helper validates original raw37 plus every prior recipe.
    assert_eq!(
        combined_catalog::old_unit_bytes(raw).unwrap(),
        combined_catalog::old_unit_bytes(&before).unwrap()
    );
    before
}
