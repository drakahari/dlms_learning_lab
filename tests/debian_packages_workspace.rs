//! Prospective stage1 package app checks. No host package commands/real archives.
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use dlms_learning_lab::{
    backup,
    debian_packages::{self as core, Action, Status},
    debian_packages_teaching::{self as teaching, Learning},
    debian_packages_workspace::{self as ws, Draft, QueryKind, Requirement},
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicBool, AtomicUsize},
};
const RAW: &str = include_str!("fixtures/debian-packages-workspace-preview.json");
fn scenario() -> Scenario {
    static FIXTURE: std::sync::OnceLock<Scenario> = std::sync::OnceLock::new();
    FIXTURE
        .get_or_init(|| pack::load(RAW).unwrap().remove(0))
        .clone()
}
fn bundled() -> Vec<Scenario> {
    static CATALOG: std::sync::OnceLock<Vec<Scenario>> = std::sync::OnceLock::new();
    CATALOG.get_or_init(|| pack::bundled().unwrap()).clone()
}
fn definition(s: &Scenario) -> &ws::Definition {
    let Goal::DebianPackages { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn state(s: &State) -> &core::State {
    let State::DebianPackages { simulation } = s else {
        panic!()
    };
    simulation
}
fn replace(st: &mut State, f: impl FnOnce(&mut core::Catalog, &mut core::Runtime)) {
    let s = state(st);
    let (mut c, mut r) = (s.catalog().clone(), s.runtime().clone());
    f(&mut c, &mut r);
    *st = State::DebianPackages {
        simulation: core::State::new(c, r).unwrap(),
    };
}
fn apply(s: &Scenario, st: &mut State, a: &Action) {
    engine::execute_for_goal(
        st,
        &s.goal,
        &format!("packages {}", serde_json::to_string(a).unwrap()),
    )
    .unwrap();
}
fn repair() -> Action {
    Action::Reinstall {
        name: "museum-report".into(),
    }
}
fn install(n: &str) -> Action {
    Action::Install {
        packages: vec![core::Request {
            name: n.into(),
            version: None,
        }],
    }
}
fn good(s: &Scenario) -> State {
    let mut st = s.initial.clone();
    apply(s, &mut st, &repair());
    assert!(engine::solved(&st, &s.goal));
    st
}
fn all(s: &Scenario) -> Vec<Scenario> {
    let mut c = bundled();
    c.push(s.clone());
    c
}
fn profile(s: &Scenario) -> Progress {
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    teaching::prepare(&mut a, s.debian_packages_teaching.as_ref().unwrap(), true).unwrap();
    p.attempts.insert(s.id.clone(), a);
    p
}
#[test]
fn technical_fixture_is_unbundled_and_reference_and_lifecycle_equivalent_repair_pass() {
    let s = scenario();
    assert!(!pack::bundled().unwrap().iter().any(|x| x.id == s.id));
    assert!(!engine::solved(&s.initial, &s.goal));
    let g = good(&s);
    let mut partial = s.initial.clone();
    apply(&s, &mut partial, &install("museum-report"));
    let partial_rows = definition(&s).checks(state(&partial));
    assert!(partial_rows.iter().any(|r| r.0) && partial_rows.iter().any(|r| !r.0));
    assert!(
        !partial_rows[1].0,
        "an installed record/no-op Install is not ordinary payload repair"
    );
    assert_eq!(definition(&s).checks(state(&g)).len(), 14);
    for aa in [
        vec![repair(), repair()],
        vec![
            Action::Remove {
                name: "museum-report".into(),
            },
            install("museum-report"),
        ],
    ] {
        let mut st = s.initial.clone();
        for a in aa {
            apply(&s, &mut st, &a);
        }
        assert!(engine::solved(&st, &s.goal));
        assert_eq!(state(&st).runtime().files, state(&g).runtime().files);
    }
}
#[test]
fn read_only_queries_separate_record_owner_payload_cached_metadata_and_local_artifact() {
    let s = scenario();
    let mut world = state(&s.initial).clone();
    assert_eq!(
        world.runtime().records["museum-retired"].status,
        Status::Installed
    );
    // Arrange an actual retained-configuration record before testing read-only
    // inventory. The authored initial/Goal intentionally preserve it installed.
    world
        .apply(&Action::Remove {
            name: "museum-retired".into(),
        })
        .unwrap();
    let st = &world;
    let b = serde_json::to_vec(st).unwrap();
    for (k, arg) in [
        (QueryKind::Inventory, ""),
        (QueryKind::RecordedOwner, "/usr/bin/museum-report"),
        (QueryKind::Payload, "museum-report"),
        (QueryKind::CachedCandidate, "museum-report"),
        (QueryKind::LocalArchive, "report-1"),
        (QueryKind::Dependencies, ""),
    ] {
        let out = ws::query(st, k, arg).unwrap();
        assert!(serde_json::to_vec(&out).unwrap().len() <= ws::MAX_QUERY_BYTES);
    }
    let ws::QueryResult::Inventory { records, .. } =
        ws::query(st, QueryKind::Inventory, "").unwrap()
    else {
        panic!()
    };
    assert!(
        records
            .iter()
            .any(|r| r.package == "museum-retired" && r.status == Status::ConfigFiles)
    );
    let ws::QueryResult::RecordedOwner { owner: Some(o) } =
        ws::query(st, QueryKind::RecordedOwner, "/usr/bin/museum-report").unwrap()
    else {
        panic!()
    };
    assert_eq!(o.package, "museum-report");
    let ws::QueryResult::Payload { findings } =
        ws::query(st, QueryKind::Payload, "museum-report").unwrap()
    else {
        panic!()
    };
    assert_eq!(findings.len(), 1);
    assert!(ws::query(st, QueryKind::LocalArchive, "report-2").is_err());
    assert_eq!(serde_json::to_vec(st).unwrap(), b);
}
#[test]
fn unsupported_real_command_syntax_missing_context_and_failed_actions_are_atomic() {
    let s = scenario();
    let mut st = s.initial.clone();
    let old = serde_json::to_vec(&st).unwrap();
    for command in [
        "dpkg -i report.deb",
        "apt-get install museum-report",
        "rpm -q museum-report",
        "packages {\"kind\":\"RefreshIndex\",\"host\":true}",
    ] {
        assert!(engine::execute_for_goal(&mut st, &s.goal, command).is_err());
        assert_eq!(serde_json::to_vec(&st).unwrap(), old);
    }
    assert!(engine::execute(&mut st, &s.solution[0]).is_err());
    assert!(
        engine::execute(&mut st, "help")
            .unwrap()
            .contains("not dpkg")
    );
    assert_eq!(serde_json::to_vec(&st).unwrap(), old);
    let foreign = &s.debian_packages_teaching.as_ref().unwrap().definition;
    let Goal::DebianPackages { definition: d } = &s.goal else {
        panic!()
    };
    let mut sim = d.initial.clone();
    assert!(ws::apply(foreign, &mut sim, &repair()).is_err());
    assert_eq!(sim, d.initial);
}
#[test]
fn each_explicit_integrity_record_file_cache_and_availability_predicate_has_its_own_negative() {
    let s = scenario();
    let d = definition(&s);
    let pass = good(&s);
    for (i, r) in d.requirements.iter().enumerate() {
        let mut st = pass.clone();
        match r {
            Requirement::Installed { package, .. } | Requirement::PayloadIntact { package } => {
                replace(&mut st, |_, rt| {
                    rt.records.remove(package);
                })
            }
            Requirement::DependenciesSatisfied {} => replace(&mut st, |_, rt| {
                rt.records.remove("museum-lib");
            }),
            Requirement::FilePreserved { path } | Requirement::FileMatches { path, .. } => {
                replace(&mut st, |_, rt| {
                    rt.files.insert(path.clone(), "different\n".into());
                })
            }
            Requirement::PackagePreserved { package } => replace(&mut st, |_, rt| {
                rt.records.remove(package);
            }),
            Requirement::Cached { .. } => {
                replace(&mut st, |_, rt| rt.cached = Some("current".into()))
            }
            Requirement::ArchiveAvailable { archive, .. } => replace(&mut st, |_, rt| {
                rt.local_archives.remove(archive);
            }),
            _ => panic!("fixture covers its declared predicates only"),
        }
        assert!(!d.checks(state(&st))[i].0, "row{i}");
        assert!(!engine::solved(&st, &s.goal));
    }
}
#[test]
fn retained_config_no_record_and_absent_file_rows_mean_their_explicit_scopes() {
    let s = scenario();
    let mut d = definition(&s).clone();
    d.requirements.extend([
        Requirement::RetainedConfig {
            package: "museum-retired".into(),
            version: Some(core::Version::parse("1.0-01").unwrap()),
        },
        Requirement::FileAbsent {
            path: "/usr/bin/museum-retired".into(),
        },
    ]);
    d.requirements.retain(|r| {
        !matches!(r,Requirement::PackagePreserved{package}if package=="museum-retired")
            && !matches!(r,Requirement::FilePreserved{path}if path=="/usr/bin/museum-retired")
    });
    d.validate().unwrap();
    let mut st = state(&good(&s)).clone();
    ws::apply(
        &d,
        &mut st,
        &Action::Remove {
            name: "museum-retired".into(),
        },
    )
    .unwrap();
    assert!(d.checks(&st).iter().all(|r| r.0));
    assert_eq!(
        st.runtime().records["museum-retired"].status,
        Status::ConfigFiles
    );
    let mut no = d.clone();
    no.requirements.retain(|r| {
        !matches!(r, Requirement::RetainedConfig { .. })
            && !matches!(r,Requirement::FilePreserved{path}if path=="/etc/museum/retired.conf")
    });
    no.requirements.push(Requirement::NoRecord {
        package: "museum-retired".into(),
    });
    ws::apply(
        &no,
        &mut st,
        &Action::Purge {
            name: "museum-retired".into(),
        },
    )
    .unwrap();
    assert!(no.checks(&st).iter().all(|r| r.0));
}
#[test]
fn current_success_breaks_and_repairs_without_query_or_action_history_credit() {
    let s = scenario();
    let mut p = profile(&s);
    let a = p.attempts.get_mut(&s.id).unwrap();
    a.state = good(&s);
    assert!(progress::current_solved(a, &s));
    replace(&mut a.state, |_, rt| {
        rt.files.remove("/usr/bin/museum-report");
    });
    assert!(!progress::current_solved(a, &s));
    ws::query(state(&a.state), QueryKind::Payload, "museum-report").unwrap();
    assert!(!progress::current_solved(a, &s));
    apply(&s, &mut a.state, &repair());
    assert!(progress::current_solved(a, &s));
    assert!(!a.workspace.as_ref().unwrap().checked);
}
#[test]
fn proposed_malformed_operation_and_query_drafts_are_not_unapplied_configuration() {
    let s = scenario();
    let mut p = profile(&s);
    let a = p.attempts.get_mut(&s.id).unwrap();
    a.state = good(&s);
    let d = a
        .workspace
        .as_mut()
        .unwrap()
        .debian_packages_draft
        .as_mut()
        .unwrap();
    d.operation = ws::Operation::Install;
    d.requests[0].package = "unfinished package".into();
    d.requests[0].version = "not a version".into();
    d.query = QueryKind::RecordedOwner;
    d.query_argument = "not a supplied absolute path".into();
    assert!(d.valid());
    assert!(d.action().is_err());
    assert!(d.query(state(&a.state)).is_err());
    assert!(progress::current_solved(a, &s));
    let before = serde_json::to_vec(a).unwrap();
    let c = all(&s);
    let b = serde_json::to_vec(&p).unwrap();
    let q = progress::decode(&b, &c).unwrap();
    assert_eq!(serde_json::to_vec(&q.attempts[&s.id]).unwrap(), before);
    assert!(progress::current_solved(&q.attempts[&s.id], &s));
}
#[test]
fn independent_practice_fill_only_experiment_repair_and_selected_drafts_stay_separate() {
    let s = scenario();
    let l = s.debian_packages_teaching.as_ref().unwrap();
    assert!(l.separate_from(state(&s.initial)));
    let mut practice = Learning::new(l).unwrap();
    let initial = practice.simulation.clone();
    let assignment = s.initial.clone();
    practice.fill(l).unwrap();
    assert_eq!(practice.simulation, initial);
    assert_eq!(
        practice.draft.action().unwrap(),
        l.example_actions[0].action
    );
    for a in &l.example_actions {
        practice.execute(l, &a.action).unwrap();
    }
    assert!(practice.checks(l).iter().all(|r| r.0));
    for a in &l.try_change.actions {
        practice.execute(l, a).unwrap();
    }
    assert_eq!(
        practice.checks(l).iter().map(|r| r.0).collect::<Vec<_>>(),
        l.try_change.expected
    );
    for a in &l.repair {
        practice.execute(l, a).unwrap();
    }
    assert!(practice.checks(l).iter().all(|r| r.0));
    assert_eq!(s.initial, assignment);
    let mut draft = practice.draft.clone();
    draft.package = "unfinished practice".into();
    practice.save_draft(l, &draft).unwrap();
    assert_eq!(practice.draft.package, "unfinished practice");
}
#[test]
fn same_catalog_different_authored_initial_is_foreign_but_coincident_current_values_are_valid() {
    let s = scenario();
    let l = s.debian_packages_teaching.as_ref().unwrap();
    let mut a = Learning::new(l).unwrap();
    let mut runtime = a.simulation.runtime().clone();
    let path = "/practice/gallery/work/note.txt";
    runtime
        .files
        .insert(path.into(), "different authored fixture".into());
    a.context = core::State::new(a.context.catalog().clone(), runtime).unwrap();
    assert!(a.valid());
    assert!(!a.valid_for(l));
    let mut b = Learning::new(l).unwrap();
    let mut coincident = s.initial.clone();
    replace(&mut coincident, |_, r| {
        r.files
            .insert("/work/note.txt".into(), "coincident evolving text".into());
    });
    let mut rt = b.simulation.runtime().clone();
    rt.files.insert(
        path.into(),
        state(&coincident).runtime().files["/work/note.txt"].clone(),
    );
    b.simulation = core::State::new(b.simulation.catalog().clone(), rt).unwrap();
    assert_ne!(b.simulation, l.definition.initial);
    assert!(b.valid_for(l));
}
#[test]
fn immutable_catalog_foreign_practice_and_wrong_kind_metadata_reject_before_saved_replacement() {
    let s = scenario();
    let c = all(&s);
    let mut p = profile(&s);
    let before = serde_json::to_vec(&p).unwrap();
    let mut foreign_s = s.clone();
    let Goal::DebianPackages {
        definition: foreign_definition,
    } = &mut foreign_s.goal
    else {
        panic!()
    };
    let mut runtime = foreign_definition.initial.runtime().clone();
    runtime
        .files
        .insert("/work/note.txt".into(), "different authored initial".into());
    foreign_definition.initial =
        core::State::new(foreign_definition.initial.catalog().clone(), runtime).unwrap();
    assert!(foreign_definition.validate().is_ok());
    assert!(
        pack::load(
            &serde_json::to_string(&json!({"schema_version":1,"scenarios":[foreign_s.clone()]}))
                .unwrap()
        )
        .is_err()
    );
    assert!(!progress::current_solved(&p.attempts[&s.id], &foreign_s));
    let mut foreign_catalog = bundled();
    foreign_catalog.push(foreign_s);
    assert!(progress::decode(&before, &foreign_catalog).is_err());
    let mut v: Value = serde_json::from_slice(&before).unwrap();
    v["attempts"][&s.id]["state"]["simulation"]["catalog"]["repository_reachable"] = false.into();
    assert!(progress::decode(&serde_json::to_vec(&v).unwrap(), &c).is_err());
    let mut v: Value = serde_json::from_slice(&before).unwrap();
    v["attempts"][&s.id]["workspace"]["debian_packages_learning"]["context"]["runtime"]["files"]
        ["/practice/gallery/work/note.txt"] = "foreign fixture".into();
    assert!(progress::decode(&serde_json::to_vec(&v).unwrap(), &c).is_err());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let old = progress::read_bounded(&path, 16_000_000).unwrap();
    p.attempts.get_mut(&s.id).unwrap().state = c[0].initial.clone();
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(progress::read_bounded(&path, 16_000_000).unwrap(), old);
}
#[test]
fn absent_optional_metadata_is_omitted_and_truthful_on_legacy_attempts() {
    let s = scenario();
    let mut a = Attempt::new(&s, 1);
    a.state = good(&s);
    assert!(a.workspace.is_none());
    assert!(progress::current_solved(&a, &s));
    workspace::prepare(&mut a);
    assert!(
        a.workspace
            .as_ref()
            .unwrap()
            .debian_packages_draft
            .is_some()
    );
    let d = serde_json::to_value(workspace::WorkspaceData::new()).unwrap();
    assert!(d.get("debian_packages_draft").is_none());
    assert!(d.get("debian_packages_learning").is_none());
    let prior = serde_json::to_vec(&a).unwrap();
    teaching::prepare(&mut a, s.debian_packages_teaching.as_ref().unwrap(), false).unwrap();
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .debian_packages_learning
            .as_ref()
            .unwrap()
            .view,
        View::Assignment
    );
    let b = serde_json::to_vec(&a).unwrap();
    assert_ne!(b, prior);
    assert!(progress::current_solved(&a, &s));
}
#[test]
fn every_original125_definition_unit_and_actual_history_survives_optional_metadata() {
    let s = scenario();
    let catalog = all(&s);
    let baseline: Value =
        serde_json::from_str(include_str!("fixtures/debian-packages-original125.json")).unwrap();
    assert_eq!(baseline["scenarios"].as_array().unwrap().len(), 125);
    let current = bundled();
    assert_eq!(one_off_boundary::before_len(&current), 176);
    for raw in baseline["scenarios"].as_array().unwrap() {
        let x = current
            .iter()
            .find(|x| x.id == raw["id"].as_str().unwrap())
            .unwrap();
        assert_eq!(x.version, raw["version"].as_u64().unwrap() as u32);
        assert_eq!(
            x.initial,
            serde_json::from_value::<State>(raw["initial"].clone()).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&x.goal).unwrap(),
            serde_json::to_value(serde_json::from_value::<Goal>(raw["goal"].clone()).unwrap())
                .unwrap()
        );
        assert_eq!(
            x.solution,
            serde_json::from_value::<Vec<String>>(raw["solution"].clone()).unwrap()
        );
    }
    let units: Value = serde_json::from_str(include_str!("../scenarios/units-v1.json")).unwrap();
    assert_eq!(units["version"], baseline["units"]["version"]);
    assert_eq!(
        &units["units"].as_array().unwrap()[..31],
        baseline["units"]["units"].as_array().unwrap().as_slice()
    );
    let original = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let mut p = progress::decode(original, &catalog).unwrap();
    let old = serde_json::to_value(&p.attempts).unwrap();
    p.attempts
        .insert(s.id.clone(), profile(&s).attempts.remove(&s.id).unwrap());
    let b = serde_json::to_vec(&p).unwrap();
    let q = progress::decode(&b, &catalog).unwrap();
    let mut actual = serde_json::to_value(&q.attempts).unwrap();
    actual.as_object_mut().unwrap().remove(&s.id);
    assert_eq!(actual, old);
    let raw: Value = serde_json::from_slice(original).unwrap();
    assert_eq!(raw["schema_version"], 1);
}
#[test]
fn actual_save_export_inspect_restore_selected_reset_and_other_attempt_preservation() {
    let s = scenario();
    let catalog = all(&s);
    let mut p = profile(&s);
    let other = &catalog[0];
    p.attempts.insert(other.id.clone(), Attempt::new(other, 7));
    p.favorites.insert(s.id.clone());
    let a = p.attempts.get_mut(&s.id).unwrap();
    a.state = good(&s);
    a.workspace
        .as_mut()
        .unwrap()
        .debian_packages_draft
        .as_mut()
        .unwrap()
        .query_argument = "unfinished query".into();
    let lesson = s.debian_packages_teaching.as_ref().unwrap();
    let practice = a
        .workspace
        .as_mut()
        .unwrap()
        .debian_packages_learning
        .as_mut()
        .unwrap();
    for action in &lesson.example_actions {
        practice.execute(lesson, &action.action).unwrap();
    }
    assert_ne!(practice.simulation, lesson.definition.initial);
    assert!(practice.checks(lesson).iter().all(|row| row.0));
    practice.view = View::Example;
    let mut draft = practice.draft.clone();
    draft.operation = ws::Operation::Install;
    draft.requests[0].package = "unfinished practice package".into();
    draft.requests[0].version = "1:".into();
    draft.query_argument = "separate unfinished practice query".into();
    practice.save_draft(lesson, &draft).unwrap();
    assert!(practice.draft.action().is_err());
    assert!(practice.valid_for(lesson));
    let other_before = serde_json::to_value(&p.attempts[&other.id]).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let q = progress::load(&path, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&q.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
    let selected_before = serde_json::to_value(&q.attempts[&s.id]).unwrap();
    let export = dir.path().join("backup.json");
    backup::export(&export, &q, &catalog).unwrap();
    let candidate = backup::inspect(
        &export,
        &catalog,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(&candidate.progress.attempts[&s.id]).unwrap(),
        selected_before
    );
    assert_eq!(
        serde_json::to_value(&candidate.progress.attempts[&other.id]).unwrap(),
        other_before
    );
    let old = progress::read_bounded(&path, 16_000_000).unwrap();
    let result = backup::restore(&path, &candidate, &catalog).unwrap();
    assert_eq!(
        progress::read_bounded(&result.safety_copy.unwrap(), 16_000_000).unwrap(),
        old
    );
    let mut re = progress::load(&path, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&re.attempts[&s.id]).unwrap(),
        selected_before
    );
    assert_eq!(
        serde_json::to_value(&re.attempts[&other.id]).unwrap(),
        other_before
    );
    let reset = Attempt::new(&s, 9);
    let reset_exact = serde_json::to_value(&reset).unwrap();
    re.attempts.insert(s.id.clone(), reset);
    assert_eq!(
        serde_json::to_value(&re.attempts[&s.id]).unwrap(),
        reset_exact
    );
    assert_eq!(re.attempts[&s.id].state, s.initial);
    assert_eq!(
        serde_json::to_value(&re.attempts[&other.id]).unwrap(),
        other_before
    );
    assert!(re.favorites.contains(&s.id));
}
#[test]
fn direct_oversized_drafts_actions_and_lesson_replay_reject_before_clone_and_keep_practice() {
    let s = scenario();
    let l = s.debian_packages_teaching.as_ref().unwrap();
    let mut p = Learning::new(l).unwrap();
    let old = serde_json::to_vec(&p).unwrap();
    let mut d = p.draft.clone();
    d.requests = vec![
        ws::RequestDraft {
            package: "x".repeat(64),
            version: String::new()
        };
        9
    ];
    assert!(p.save_draft(l, &d).is_err());
    assert_eq!(serde_json::to_vec(&p).unwrap(), old);
    let action = Action::Install {
        packages: vec![
            core::Request {
                name: "museum-report".into(),
                version: None
            };
            10_000
        ],
    };
    assert!(p.execute(l, &action).is_err());
    let mut valid_draft = p.draft.clone();
    let draft_before = valid_draft.clone();
    assert!(valid_draft.fill(&p.simulation, &action).is_err());
    assert_eq!(valid_draft, draft_before);
    assert_eq!(serde_json::to_vec(&p).unwrap(), old);
    let mut lesson = l.clone();
    lesson.example_actions = vec![l.example_actions[0].clone(); 10_000];
    assert!(Learning::new(&lesson).is_err());
    let mut absent = Attempt::new(&s, 1);
    absent.workspace = None;
    let before = serde_json::to_vec(&absent).unwrap();
    assert!(teaching::prepare(&mut absent, &lesson, true).is_err());
    assert_eq!(serde_json::to_vec(&absent).unwrap(), before);
    assert!(p.fill(&lesson).is_err());
    assert_eq!(serde_json::to_vec(&p).unwrap(), old);
}
#[test]
fn strict_json_unknowns_count_bounds_and_app_capacity_do_not_expand_global_limits() {
    let s = scenario();
    let d = definition(&s);
    let mut raw: Value = serde_json::to_value(d).unwrap();
    raw["requirements"] = Value::Array(vec![serde_json::to_value(&d.requirements[0]).unwrap(); 33]);
    assert!(serde_json::from_value::<ws::Definition>(raw).is_err());
    let mut raw: Value = serde_json::from_str(RAW).unwrap();
    raw["scenarios"][0]["initial"]["simulation"]["catalog"]["architecture"] = "arm64".into();
    assert!(pack::load(&raw.to_string()).is_err());
    let text = serde_json::to_string(d).unwrap().replacen(
        "\"initial\":",
        "\"extra\":true,\"initial\":",
        1,
    );
    assert!(serde_json::from_str::<ws::Definition>(&text).is_err());
    assert!(pack::load(&" ".repeat(2_000_001)).is_err());
    let mut d = d.clone();
    d.requirements.clear();
    assert!(d.validate().is_err());
    let mut d = definition(&s).clone();
    d.requirements
        .retain(|r| !matches!(r, Requirement::Installed { .. }));
    assert!(d.validate().is_err());
}
#[test]
fn escaped_admitted_state_practice_and_real_profile_roundtrip_exactly() {
    let mut s = scenario();
    fn escaped(state: &core::State) -> core::State {
        let mut c = state.catalog().clone();
        let mut r = state.runtime().clone();
        for a in c.archives.values_mut() {
            for f in a.files.values_mut() {
                f.contents = "\u{1}".repeat(1024);
            }
        }
        for text in r.files.values_mut() {
            *text = "\u{2}".repeat(1024);
        }
        core::State::new(c, r).unwrap()
    }
    let initial = escaped(state(&s.initial));
    assert!(ws::admitted(&initial));
    s.initial = State::DebianPackages {
        simulation: initial.clone(),
    };
    let Goal::DebianPackages { definition } = &mut s.goal else {
        panic!()
    };
    definition.initial = initial;
    let l = s.debian_packages_teaching.as_mut().unwrap();
    l.definition.initial = escaped(&l.definition.initial);
    let mut p = profile(&s);
    let b = serde_json::to_vec(&p).unwrap();
    assert!(b.len() < 16_000_000);
    let catalog = all(&s);
    for bytes in [b, serde_json::to_vec_pretty(&p).unwrap()] {
        let q = progress::decode(&bytes, &catalog).unwrap();
        assert_eq!(
            serde_json::to_value(&q.attempts).unwrap(),
            serde_json::to_value(&p.attempts).unwrap()
        );
    }
    let a = p.attempts.get_mut(&s.id).unwrap();
    a.state = good(&s);
    a.workspace
        .as_mut()
        .unwrap()
        .debian_packages_learning
        .as_mut()
        .unwrap()
        .draft
        .query_argument = "é".repeat(80);
    assert!(progress::current_solved(a, &s) == engine::solved(&a.state, &s.goal));
}

#[test]
fn legal_pure_model_maxima_can_exceed_app_admission_without_raising_pack_or_profile_caps() {
    let files: BTreeMap<_, _> = (0..32)
        .map(|n| {
            (
                format!("/large/file-{n}"),
                core::Payload {
                    contents: "\u{1}".repeat(2048),
                    conffile: false,
                },
            )
        })
        .collect();
    let catalog = core::Catalog {
        architecture: core::Architecture::Amd64,
        repository_reachable: true,
        published: "large-index".into(),
        archives: BTreeMap::from([(
            "large-archive".into(),
            core::Archive {
                name: "large-package".into(),
                version: core::Version::parse("1").unwrap(),
                architecture: core::Architecture::All,
                depends: vec![],
                files: files.clone(),
            },
        )]),
        publications: BTreeMap::from([(
            "large-index".into(),
            core::Index {
                candidates: BTreeMap::from([("large-package".into(), "large-archive".into())]),
            },
        )]),
        extra_paths: BTreeSet::new(),
    };
    let runtime = core::Runtime {
        cached: Some("large-index".into()),
        local_archives: BTreeSet::from(["large-archive".into()]),
        records: BTreeMap::from([(
            "large-package".into(),
            core::PackageRecord {
                archive: "large-archive".into(),
                status: Status::Installed,
            },
        )]),
        files: files
            .keys()
            .map(|p| (p.clone(), "\u{2}".repeat(2048)))
            .collect(),
    };
    let large = core::State::new(catalog, runtime).unwrap();
    assert!(large.validate().is_ok());
    assert!(serde_json::to_vec(&large).unwrap().len() > ws::MAX_APP_STATE_BYTES);
    assert!(!ws::admitted(&large));
    assert!(Draft::new(&large).is_err());
    let mut ascii_catalog = large.catalog().clone();
    let mut ascii_runtime = large.runtime().clone();
    for artifact in ascii_catalog.archives.values_mut() {
        for f in artifact.files.values_mut() {
            f.contents = "a".repeat(2048);
        }
    }
    for contents in ascii_runtime.files.values_mut() {
        *contents = "b".repeat(2048);
    }
    let ascii = core::State::new(ascii_catalog, ascii_runtime).unwrap();
    assert!(ws::admitted(&ascii));
    let exact = serde_json::to_vec(&ascii).unwrap();
    assert!(
        ws::query(&ascii, QueryKind::LocalArchive, "large-archive")
            .unwrap_err()
            .contains("32KiB")
    );
    assert_eq!(serde_json::to_vec(&ascii).unwrap(), exact);
    let d = ws::Definition {
        initial: large.clone(),
        requirements: vec![
            Requirement::Installed {
                package: "large-package".into(),
                constraint: None,
            },
            Requirement::PayloadIntact {
                package: "large-package".into(),
            },
        ],
    };
    assert!(d.validate().is_err());
    assert!(
        pack::validate_state(&State::DebianPackages {
            simulation: large.clone()
        })
        .is_err()
    );
    let mut s = scenario();
    s.initial = State::DebianPackages { simulation: large };
    s.goal = Goal::DebianPackages {
        definition: Box::new(d),
    };
    s.debian_packages_teaching = None;
    let raw = serde_json::to_string(&json!({"schema_version":1,"scenarios":[s]})).unwrap();
    assert!(raw.len() < 2_000_000);
    assert!(pack::load(&raw).is_err());
}

#[test]
fn package_dispatch_rejects_excess_input_before_goal_or_trim_work() {
    let s = scenario();
    let mut current = s.initial.clone();
    let exact = serde_json::to_vec(&current).unwrap();
    let other = bundled().remove(0);
    let too_long = format!("{}help", " ".repeat(ws::MAX_ACTION_BYTES));
    for goal in [&s.goal, &other.goal] {
        let error = engine::execute_for_goal(&mut current, goal, &too_long).unwrap_err();
        assert!(
            error.contains("16KiB"),
            "length guard precedes context selection: {error}"
        );
        assert_eq!(serde_json::to_vec(&current).unwrap(), exact);
    }
}
#[test]
fn oversized_direct_requirement_field_rejects_before_aggregate_json_count() {
    let s = scenario();
    let mut d = definition(&s).clone();
    d.requirements.push(Requirement::FileMatches {
        path: "/work/note.txt".into(),
        contents: "\u{1}".repeat(ws::MAX_DEFINITION_BYTES),
    });
    assert!(d.validate().unwrap_err().contains("requirement field"));
    assert!(!d.checks(state(&s.initial)).iter().any(|row| row.0));
}
#[test]
fn practice_prepare_rejects_other_kind_shared_catalog_and_foreign_existing_origin_atomically() {
    let s = scenario();
    let lesson = s.debian_packages_teaching.as_ref().unwrap();
    let old_unix = bundled()
        .into_iter()
        .find(|v| matches!(v.initial, State::Unix { .. }))
        .unwrap();
    let mut other = Attempt::new(&old_unix, 1);
    let exact = serde_json::to_vec(&other).unwrap();
    assert!(teaching::prepare(&mut other, lesson, true).is_err());
    assert_eq!(serde_json::to_vec(&other).unwrap(), exact);

    let mut shared = s.clone();
    shared.initial = State::DebianPackages {
        simulation: lesson.definition.initial.clone(),
    };
    let mut other = Attempt::new(&shared, 1);
    let exact = serde_json::to_vec(&other).unwrap();
    assert!(teaching::prepare(&mut other, lesson, true).is_err());
    assert_eq!(serde_json::to_vec(&other).unwrap(), exact);

    let mut foreign = lesson.clone();
    let mut runtime = foreign.definition.initial.runtime().clone();
    runtime.files.insert(
        "/practice/gallery/work/note.txt".into(),
        "other supplied practice initial".into(),
    );
    foreign.definition.initial =
        core::State::new(foreign.definition.initial.catalog().clone(), runtime).unwrap();
    assert!(foreign.bounded());
    let foreign_learning = Learning::new(&foreign).unwrap();
    assert!(foreign_learning.valid_for(&foreign));
    let mut other = Attempt::new(&s, 1);
    workspace::prepare(&mut other);
    other.workspace.as_mut().unwrap().debian_packages_learning = Some(foreign_learning);
    let exact = serde_json::to_vec(&other).unwrap();
    assert!(teaching::prepare(&mut other, lesson, false).is_err());
    assert_eq!(serde_json::to_vec(&other).unwrap(), exact);

    let mut compatible = profile(&s).attempts.remove(&s.id).unwrap();
    let exact = serde_json::to_vec(&compatible).unwrap();
    teaching::prepare(&mut compatible, lesson, false).unwrap();
    assert_eq!(serde_json::to_vec(&compatible).unwrap(), exact);
}
