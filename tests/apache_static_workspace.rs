//! Prospective app-only model checks. No host Apache/HTTP/command execution.
#[path = "../examples/support/automation_digest.rs"]
mod amendment_sha;
#[path = "support/teaching_amendment.rs"]
mod teaching_amendment;
use dlms_learning_lab::{
    apache_static as web,
    apache_static_teaching::{self as teaching, Learning},
    apache_static_workspace::{self as work, Draft, Requirement},
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    workspace,
};
use dlms_learning_lab::{backup, script_teaching::View, shell};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        OnceLock,
        atomic::{AtomicBool, AtomicUsize},
    },
};
fn scenario() -> Scenario {
    pack::load(include_str!(
        "fixtures/apache-static-workspace-preview.json"
    ))
    .unwrap()
    .remove(0)
}
fn def(s: &Scenario) -> &work::Definition {
    let Goal::ApacheStatic { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn sim(s: &State) -> &web::State {
    let State::ApacheStatic { simulation } = s else {
        panic!()
    };
    simulation
}
fn sim_mut(s: &mut State) -> &mut web::State {
    let State::ApacheStatic { simulation } = s else {
        panic!()
    };
    simulation
}
fn run(s: &Scenario, state: &mut State, action: &web::Action) {
    engine::execute_for_goal(
        state,
        &s.goal,
        &format!("{}{}", work::PREFIX, serde_json::to_string(action).unwrap()),
    )
    .unwrap();
}
fn good(s: &Scenario) -> State {
    let mut state = s.initial.clone();
    for command in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, command).unwrap();
    }
    assert!(
        engine::solved(&state, &s.goal),
        "{:?}",
        def(s).checks(sim(&state))
    );
    state
}
fn correct(s: &Scenario) -> String {
    let a: web::Action =
        serde_json::from_str(s.solution[0].strip_prefix(work::PREFIX).unwrap()).unwrap();
    let web::Action::SaveConfig { text } = a else {
        panic!()
    };
    text
}
fn prepared(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    teaching::prepare(&mut a, s.apache_static_teaching.as_ref().unwrap()).unwrap();
    a
}
fn catalog() -> Vec<Scenario> {
    static ALL: OnceLock<Vec<Scenario>> = OnceLock::new();
    ALL.get_or_init(|| pack::bundled().unwrap()).clone()
}
#[test]
fn current_and_next_start_positive_serving_negative_private_and_preservation_reach_reference() {
    let s = scenario();
    let before = s.initial.clone();
    let rows = def(&s).checks(sim(&before));
    assert!(rows.iter().any(|r| r.0) && rows.iter().any(|r| !r.0));
    let pass = good(&s);
    assert!(def(&s).checks(sim(&pass)).iter().all(|r| r.0));
    assert_eq!(before, s.initial);
}
#[test]
fn legal_whitespace_directive_and_hostname_case_alternatives_are_not_one_hidden_config() {
    let s = scenario();
    let text = correct(&s)
        .replace(
            "ServerName report.example.test",
            "servername REPORT.EXAMPLE.TEST",
        )
        .replace("Listen 8080", "listen 8080");
    let text = format!("# alternate comments and case\n\n{text}\n");
    let mut state = s.initial.clone();
    run(&s, &mut state, &web::Action::SaveConfig { text });
    run(&s, &mut state, &web::Action::Reload {});
    assert!(engine::solved(&state, &s.goal));
}
#[test]
fn saved_only_and_loaded_only_shortcuts_fail_their_exact_rows_without_touching_current_state() {
    let s = scenario();
    let mut state = s.initial.clone();
    run(
        &s,
        &mut state,
        &web::Action::SaveConfig { text: correct(&s) },
    );
    let rows = def(&s).checks(sim(&state));
    assert!(
        !rows[0].0 && rows[1].0,
        "correct next start is not current serving"
    );
    run(&s, &mut state, &web::Action::Reload {});
    let before = sim(&state).active.clone();
    run(
        &s,
        &mut state,
        &web::Action::SaveConfig {
            text: sim(&s.initial).saved.clone(),
        },
    );
    let snapshot = state.clone();
    let rows = def(&s).checks(sim(&state));
    assert!(rows[0].0 && !rows[1].0 && rows[2].0 && rows[3].0 && rows[4].0 && !rows[5].0);
    assert!(!rows[8].0);
    assert_eq!(sim(&state).active, before);
    assert_eq!(state, snapshot, "next-start test is read-only");
}
#[test]
fn stopped_or_bad_saved_source_never_substitutes_for_http_denial() {
    let s = scenario();
    let mut state = good(&s);
    run(&s, &mut state, &web::Action::Stop {});
    let rows = def(&s).checks(sim(&state));
    assert!(
        !rows[0].0 && !rows[4].0 && rows[1].0,
        "no listener is not HTTP403/positive200"
    );
    state = good(&s);
    run(
        &s,
        &mut state,
        &web::Action::SaveConfig {
            text: "Listen invalid\n".into(),
        },
    );
    let rows = def(&s).checks(sim(&state));
    assert!(
        rows[0].0 && rows[4].0 && !rows[1].0 && !rows[5].0,
        "fresh start failure is not a negative response"
    );
    let mut only = def(&s).clone();
    only.requirements
        .retain(|r| !matches!(r,Requirement::CurrentHttp{expected,..}if expected.status==200));
    assert!(only.validate().is_err());
}
#[test]
fn static_file_save_is_live_without_reload_and_named_preserves_fail_exact_rows() {
    let s = scenario();
    let mut state = good(&s);
    let active = sim(&state).active.clone();
    run(
        &s,
        &mut state,
        &web::Action::WriteFile {
            path: "/srv/report/index.html".into(),
            text: "wrong but valid UTF8\n".into(),
        },
    );
    let rows = def(&s).checks(sim(&state));
    assert!(!rows[0].0 && !rows[1].0 && rows[2].0 && rows[3].0);
    assert_eq!(sim(&state).active, active);
    state = good(&s);
    run(
        &s,
        &mut state,
        &web::Action::WriteFile {
            path: "/srv/observer/index.html".into(),
            text: "changed observer\n".into(),
        },
    );
    let rows = def(&s).checks(sim(&state));
    assert!(!rows[2].0 && !rows[3].0 && !rows[6].0);
}
#[test]
fn bad_reload_unknown_file_and_missing_goal_context_are_atomic() {
    let s = scenario();
    let mut state = good(&s);
    run(
        &s,
        &mut state,
        &web::Action::SaveConfig {
            text: "Unsupported on\n".into(),
        },
    );
    let before = state.clone();
    for a in [
        web::Action::Reload {},
        web::Action::WriteFile {
            path: "/etc/passwd".into(),
            text: "no host path\n".into(),
        },
    ] {
        let input = format!("{}{}", work::PREFIX, serde_json::to_string(&a).unwrap());
        assert!(engine::execute_for_goal(&mut state, &s.goal, &input).is_err());
        assert_eq!(state, before);
    }
    assert!(engine::execute(&mut state, "apache-static {\"kind\":\"Stop\"}").is_err());
    assert_eq!(state, before);
    assert!(
        engine::execute_for_goal(
            &mut state,
            &s.goal,
            &"x".repeat(work::PREFIX.len() + web::MAX_ACTION_JSON + 1)
        )
        .is_err()
    );
    assert_eq!(state, before);
}
#[test]
fn query_fields_and_clean_file_selection_are_readonly_not_pending_credit() {
    let s = scenario();
    let mut a = prepared(&s);
    a.state = good(&s);
    let state = sim(&a.state).clone();
    let w = a.workspace.as_mut().unwrap();
    w.apache_static_draft = Some(Draft::new(&state).unwrap());
    let d = w.apache_static_draft.as_mut().unwrap();
    d.request_port = "bad port".into();
    d.request_hosts = vec!["excluded_host".into()];
    d.request_path = "/%2e".into();
    d.select_file(&state, "/srv/observer/index.html").unwrap();
    assert!(d.query(&state).is_err());
    assert!(d.matches(&state));
    assert!(progress::current_solved(&a, &s));
    a.workspace
        .as_mut()
        .unwrap()
        .apache_static_draft
        .as_mut()
        .unwrap()
        .file_text = "unsaved text\n".into();
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn malformed_config_and_file_drafts_are_retained_until_atomic_explicit_save() {
    let s = scenario();
    let mut a = prepared(&s);
    a.state = good(&s);
    let initial = sim(&s.initial);
    let mut d = Draft::new(sim(&a.state)).unwrap();
    d.config_text = "x".repeat(513);
    let before = a.state.clone();
    assert!(d.save_config(initial, sim_mut(&mut a.state)).is_err());
    assert_eq!(a.state, before);
    assert_eq!(d.config_text.len(), 513);
    d.config_text = sim(&a.state).saved.clone();
    d.select_file(sim(&a.state), "/srv/report/index.html")
        .unwrap();
    d.file_text = "pending page\n".into();
    d.select_file(sim(&a.state), "/srv/report/index.html")
        .unwrap();
    assert_eq!(
        d.file_text, "pending page\n",
        "reselecting must not silently discard"
    );
    let snapshot = d.clone();
    assert!(
        d.select_file(sim(&a.state), "/srv/observer/index.html")
            .is_err()
    );
    assert_eq!(d, snapshot);
    d.save_file(initial, sim_mut(&mut a.state)).unwrap();
    assert_eq!(
        sim(&a.state).files["/srv/report/index.html"],
        "pending page\n"
    );
}
#[test]
fn direct_goal_bounds_duplicate_and_typed_reader_limits_reject_without_publication() {
    let s = scenario();
    let mut d = def(&s).clone();
    d.requirements.push(d.requirements[0].clone());
    assert!(d.validate().is_err());
    let mut d = def(&s).clone();
    let Requirement::CurrentHttp { expected, .. } = &mut d.requirements[0] else {
        panic!()
    };
    expected.body = "x".repeat(100_000);
    assert!(d.validate().is_err());
    let mut raw = serde_json::to_value(def(&s)).unwrap();
    raw["requirements"] = serde_json::Value::Array(
        (0..32)
            .map(|_| raw["requirements"][0].clone())
            .chain([serde_json::Value::Null])
            .collect(),
    );
    let e = serde_json::from_value::<work::Definition>(raw).unwrap_err();
    assert!(e.to_string().len() <= 512);
    let mut draft = serde_json::to_value(Draft::new(sim(&s.initial)).unwrap()).unwrap();
    draft["request_hosts"] = serde_json::json!(["a", "b", {}]);
    assert!(serde_json::from_value::<Draft>(draft).is_err());
    let mut unknown = serde_json::to_value(def(&s)).unwrap();
    unknown["x".repeat(10_000)] = true.into();
    let e = serde_json::from_value::<work::Definition>(unknown).unwrap_err();
    assert!(e.to_string().len() <= 512);
}
#[test]
fn separate_example_insert_only_exact_stale_saved_experiment_and_repair_keep_assignment() {
    let s = scenario();
    let l = s.apache_static_teaching.as_ref().unwrap();
    let mut p = Learning::new(l).unwrap();
    let before = p.simulation.clone();
    p.insert_selected(l).unwrap();
    assert_eq!(p.simulation, before);
    assert!(!p.draft.matches(&p.simulation));
    for a in &l.example_actions {
        p.perform(l, &a.action).unwrap();
    }
    assert!(p.checks(l).iter().all(|r| r.0));
    for a in &l.try_change.actions {
        p.perform(l, a).unwrap();
    }
    let rows = p.checks(l);
    assert!(rows[0].0);
    assert_eq!(
        rows[1..].iter().map(|r| r.0).collect::<Vec<_>>(),
        l.try_change.expected
    );
    for a in &l.repair {
        p.perform(l, a).unwrap();
    }
    assert!(p.checks(l).iter().all(|r| r.0));
    assert_eq!(s.initial, scenario().initial);
}
#[test]
fn foreign_fixture_wrong_kind_and_context_changes_reject_exact_prior_attempt() {
    let s = scenario();
    let l = s.apache_static_teaching.as_ref().unwrap();
    let mut other = Attempt::new(&catalog()[0], 1);
    let before = serde_json::to_value(&other).unwrap();
    assert!(teaching::prepare(&mut other, l).is_err());
    assert_eq!(serde_json::to_value(other).unwrap(), before);
    let mut a = prepared(&s);
    let p = a
        .workspace
        .as_mut()
        .unwrap()
        .apache_static_learning
        .as_mut()
        .unwrap();
    p.fixture.saved.push('\n');
    assert!(p.valid());
    assert!(!p.valid_for(l));
    let before = serde_json::to_value(&a).unwrap();
    assert!(teaching::prepare(&mut a, l).is_err());
    assert_eq!(serde_json::to_value(&a).unwrap(), before);
}
#[test]
fn pure_legal_escaped_world_over_app_admission_is_rejected_without_widening_core() {
    let s = scenario();
    let mut inventory = sim(&s.initial).inventory.clone();
    inventory
        .entries
        .retain(|_, k| *k == web::EntryKind::Directory);
    let mut files = BTreeMap::new();
    for n in 0..90 {
        let path = format!("/srv/report/page{n}.txt");
        inventory.entries.insert(path.clone(), web::EntryKind::File);
        files.insert(path, "\0".repeat(1000));
    }
    let world = web::State::new(inventory, sim(&s.initial).saved.clone(), files).unwrap();
    assert!(world.validate().is_ok());
    assert!(!work::admitted(&world));
    let mut d = def(&s).clone();
    d.initial = world;
    assert!(d.validate().is_err());
}

#[test]
fn head_redirect_and_verified_bad_host_probes_keep_real_typed_outcomes() {
    let s = scenario();
    let state = good(&s);
    let mut d = def(&s).clone();
    let Requirement::CurrentHttp { request, expected } = &mut d.requirements[0] else {
        panic!()
    };
    request.method = web::Method::Head;
    expected.body.clear();
    // The corresponding GET remains13 UTF8 bytes in the independent next-start check.
    assert_eq!(expected.representation_bytes, Some("Report ready\n".len()));
    assert!(d.validate().is_ok());
    assert!(d.checks(sim(&state)).iter().all(|r| r.0));
    let probe = web::Request {
        server: "report-server".into(),
        port: 8080,
        method: web::Method::Get,
        hosts: vec![],
        path: "/".into(),
    };
    let expected = work::HttpExpectation {
        status: 400,
        vhost: None,
        resource: None,
        body: String::new(),
        representation_bytes: None,
        redirect_path: None,
        cause: Some(work::ResponseCause::InvalidHost),
    };
    d.requirements.push(Requirement::CurrentHttp {
        request: probe,
        expected,
    });
    assert!(d.validate().is_ok());
    assert!(d.checks(sim(&state)).iter().all(|r| r.0));
    let Requirement::CurrentHttp { request, .. } = d.requirements.last_mut().unwrap() else {
        panic!()
    };
    request.hosts = vec!["excluded_name".into()];
    assert!(
        !d.checks(sim(&state)).last().unwrap().0,
        "excludedHost never becomes canned400"
    );
}
#[test]
fn maximum_admitted_escaped_world_probes_and_learning_roundtrip_are_bounded() {
    let mut s = scenario();
    let lesson = s.apache_static_teaching.as_mut().unwrap();
    for (state, prefix) in [
        (&mut lesson.definition.initial, "/practice/gallery"),
        (sim_mut(&mut s.initial), ""),
    ] {
        for n in 0..80 {
            let path = format!("{prefix}/srv/report/page{n}.txt");
            state
                .inventory
                .entries
                .insert(path.clone(), web::EntryKind::File);
            state.files.insert(path, "\0".repeat(1000));
        }
        assert!(state.validate().is_ok());
        assert!(work::admitted(state));
    }
    let Goal::ApacheStatic { definition } = &mut s.goal else {
        panic!()
    };
    definition.initial = sim(&s.initial).clone();
    for n in 0..23 {
        let request = web::Request {
            server: "report-server".into(),
            port: 8080,
            method: web::Method::Get,
            hosts: vec!["report.example.test".into()],
            path: format!("/page{n}.txt"),
        };
        let expected = work::HttpExpectation {
            status: 200,
            vhost: Some("report.example.test".into()),
            resource: Some(format!("/srv/report/page{n}.txt")),
            body: "\0".repeat(1000),
            representation_bytes: Some(1000),
            redirect_path: None,
            cause: Some(work::ResponseCause::File),
        };
        definition
            .requirements
            .push(Requirement::CurrentHttp { request, expected });
    }
    assert_eq!(definition.requirements.len(), 32);
    assert!(definition.validate().is_ok());
    // Sample goal/actions remain the disclosed example; larger fixed files are immutable context only.
    lesson.validate().unwrap();
    let mut p = Learning::new(lesson).unwrap();
    p.draft.config_text = "\\".repeat(web::MAX_CONFIG);
    p.draft.request_hosts = vec!["\\".repeat(260); 2];
    p.draft.request_path = "\\".repeat(256);
    p.draft.request_port = "invalid!".into();
    assert!(p.valid_for(lesson));
    for bytes in [
        serde_json::to_vec(&p).unwrap(),
        serde_json::to_vec_pretty(&p).unwrap(),
    ] {
        assert!(bytes.len() < teaching::MAX_LEARNING);
        let restored: Learning = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            serde_json::to_value(restored).unwrap(),
            serde_json::to_value(&p).unwrap()
        );
    }
    let good = good(&s);
    let start = std::time::Instant::now();
    let rows = def(&s).checks(sim(&good));
    assert!(rows.iter().all(|r| r.0));
    eprintln!(
        "Apache max admitted32-probe check measured {:?}; source admission remains bounded",
        start.elapsed()
    );
}
#[test]
fn original125_typed_specs_relative_order_units_and_optional_nil_wire_are_exact() {
    let baseline: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/spanning-tree-original125.json")).unwrap();
    let expected: Vec<Scenario> = serde_json::from_value(baseline["scenarios"].clone()).unwrap();
    assert_eq!(expected.len(), 125);
    let ids: BTreeSet<_> = expected.iter().map(|s| s.id.as_str()).collect();
    let all = catalog();
    let selected: Vec<_> = all.iter().filter(|s| ids.contains(s.id.as_str())).collect();
    assert_eq!(selected.len(), 125);
    for (a, b) in selected.into_iter().zip(&expected) {
        assert_eq!(a.id, b.id);
        let value = teaching_amendment::original_scenario(a).unwrap();
        assert_eq!(value, serde_json::to_value(b).unwrap(), "{}", a.id);
        assert!(value.get("apache_static_teaching").is_none());
        let mut a = Attempt::new(a, 1);
        workspace::prepare(&mut a);
        let w = serde_json::to_value(a.workspace).unwrap();
        assert!(
            w.get("apache_static_draft").is_none() && w.get("apache_static_learning").is_none()
        );
    }
    let current_units: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/units-v1.json")).unwrap();
    assert_eq!(current_units["version"], baseline["units"]["version"]);
    assert_eq!(current_units["version"], serde_json::json!(1));
    let old_units = baseline["units"]["units"].as_array().unwrap();
    let all_units = current_units["units"].as_array().unwrap();
    assert_eq!(old_units.len(), 31);
    assert_eq!(all_units.len(), 39);
    assert_eq!(&all_units[..old_units.len()], old_units.as_slice());
}

fn retained_profile(all: &[Scenario]) -> Progress {
    let mut p = progress::decode(
        include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json"),
        all,
    )
    .unwrap();
    for s in all {
        let a = p
            .attempts
            .entry(s.id.clone())
            .or_insert_with(|| Attempt::new(s, 1));
        workspace::prepare(a);
        if let (State::Script { source, run }, Goal::Script { cases }) = (&mut a.state, &s.goal) {
            let authored = if s.id == "shell-case-publication" {
                "# retained unfinished assignment\nprintf '".to_owned()
            } else {
                s.solution.last().unwrap().clone()
            };
            *source = authored.clone();
            if s.id == "shell-case-publication" {
                *run = None;
            } else {
                *run = Some(shell::Record {
                    engine_revision: shell::EVALUATOR_REVISION,
                    source: authored.clone(),
                    results: cases.iter().map(|c| shell::run(&authored, c)).collect(),
                });
            }
            if let Some(l) = a
                .workspace
                .as_mut()
                .and_then(|w| w.script_learning.as_mut())
            {
                if s.id == "shell-case-publication" {
                    l.example_source = "# separate retained unfinished example\nprintf '".into();
                }
                if let Some(teaching) = &s.teaching {
                    l.example_result = Some(shell::run(&l.example_source, &teaching.fixture));
                    l.example_revision = shell::EVALUATOR_REVISION;
                }
            }
        }
    }
    p.favorites.insert("shell-case-publication".into());
    p.favorites.insert("ccna-dhcp-held-options".into());
    p
}
fn prior(p: &Progress, all: &[Scenario]) -> BTreeMap<String, serde_json::Value> {
    all.iter()
        .map(|s| {
            (
                s.id.clone(),
                serde_json::to_value(&p.attempts[&s.id]).unwrap(),
            )
        })
        .collect()
}
#[test]
fn current_real_script_records_genuine678_histories_both_editors_backup_restore_and_reset_survive()
{
    let mut all = catalog();
    let originals = all.clone();
    let mut p = retained_profile(&all);
    let histories = p
        .attempts
        .values()
        .filter_map(|a| a.workspace.as_ref())
        .flat_map(|w| {
            w.script_history
                .as_ref()
                .into_iter()
                .chain(&w.script_history_more)
        })
        .map(|r| r.engine_revision)
        .collect::<BTreeSet<_>>();
    assert!(histories.is_superset(&BTreeSet::from([6, 7, 8])));
    let script_count = all
        .iter()
        .filter(|s| matches!(s.initial, State::Script { .. }))
        .count();
    let current=p.attempts.values().filter(|a|matches!(&a.state,State::Script{run:Some(r),..}if r.engine_revision==shell::EVALUATOR_REVISION)).count();
    assert_eq!(current, script_count - 1);
    let before = prior(&p, &all);
    let s = scenario();
    let mut a = prepared(&s);
    a.state = good(&s);
    let lesson = s.apache_static_teaching.as_ref().unwrap();
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .apache_static_learning
        .as_mut()
        .unwrap();
    for step in &lesson.example_actions {
        l.perform(lesson, &step.action).unwrap();
    }
    l.view = View::Example;
    l.draft.config_text = "different unfinished example document".into();
    let w = a.workspace.as_mut().unwrap();
    w.apache_static_draft = Some(Draft::new(sim(&a.state)).unwrap());
    let d = w.apache_static_draft.as_mut().unwrap();
    d.config_text = "unfinished assignment document".into();
    d.request_port = "bad!".into();
    assert!(
        d.valid_for(sim(&a.state)),
        "retain bounded malformed query text"
    );
    assert!(
        d.query(sim(&a.state)).is_err(),
        "malformed port cannot run a request"
    );
    let selected = serde_json::to_value(&a).unwrap();
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    all.push(s.clone());
    let favorites = p.favorites.clone();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let old8 = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let old7 = include_bytes!("fixtures/shell-v7-synthetic-writer-progress.json");
    std::fs::write(&path, old8).unwrap();
    std::fs::write(path.with_extension("before-shell-v8.json"), old7).unwrap();
    progress::save(&path, &p).unwrap();
    assert_eq!(
        std::fs::read(
            path.with_extension(format!("before-shell-v{}.json", shell::EVALUATOR_REVISION))
        )
        .unwrap(),
        old8
    );
    assert_eq!(
        std::fs::read(path.with_extension("before-shell-v8.json")).unwrap(),
        old7
    );
    let loaded = progress::load(&path, &all).unwrap();
    for (id, value) in &before {
        assert_eq!(
            serde_json::to_value(&loaded.attempts[id]).unwrap(),
            *value,
            "load{id}"
        )
    }
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&s.id]).unwrap(),
        selected
    );
    let export = dir.path().join("explicit-backup.json");
    backup::export(&export, &loaded, &all).unwrap();
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert_eq!(
        serde_json::to_value(&candidate.progress.attempts[&s.id]).unwrap(),
        selected
    );
    for (id, value) in &before {
        assert_eq!(
            serde_json::to_value(&candidate.progress.attempts[id]).unwrap(),
            *value,
            "inspect{id}"
        )
    }
    backup::restore(&path, &candidate, &all).unwrap();
    let mut restored = progress::load(&path, &all).unwrap();
    assert_eq!(restored.favorites, favorites);
    assert_eq!(
        serde_json::to_value(&restored.attempts[&s.id]).unwrap(),
        selected
    );
    restored.attempts.insert(s.id.clone(), prepared(&s));
    for (id, value) in &before {
        assert_eq!(
            serde_json::to_value(&restored.attempts[id]).unwrap(),
            *value,
            "reset{id}"
        )
    }
    assert_eq!(restored.attempts[&s.id].state, s.initial);
    assert_eq!(originals.len(), 154);
}
#[test]
fn context_corruption_and_wrong_kind_save_never_overwrite_a_valid_profile() {
    let s = scenario();
    let mut all = catalog();
    all.push(s.clone());
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), prepared(&s));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    progress::save(&path, &p).unwrap();
    let prior = std::fs::read(&path).unwrap();
    let mut oversized = progress::decode(&prior, &all).unwrap();
    oversized
        .attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .apache_static_draft
        .as_mut()
        .unwrap()
        .request_port = "bad query port".into();
    assert!(progress::save(&path, &oversized).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), prior);
    assert!(progress::decode(&serde_json::to_vec(&oversized).unwrap(), &all).is_err());
    let mut raw = serde_json::to_value(&p).unwrap();
    raw["attempts"][&s.id]["state"]["simulation"]["inventory"]["fixture"] =
        "other-inventory".into();
    assert!(progress::decode(&serde_json::to_vec(&raw).unwrap(), &all).is_err());
    let old = all.iter().find(|s| s.id == "lpic1-permissions").unwrap();
    let mut wrong = Attempt::new(old, 1);
    wrong.workspace = p.attempts[&s.id].workspace.clone();
    p.attempts.insert(old.id.clone(), wrong);
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), prior);
}

#[test]
fn authorized_directory_redirect_uses_exact_current_and_next_start_target_path() {
    let s = scenario();
    let mut state = good(&s);
    let text = correct(&s).replace(
        "<Directory \"/srv/report/private\">\nRequire all denied",
        "<Directory \"/srv/report/private\">\nRequire all granted",
    );
    run(&s, &mut state, &web::Action::SaveConfig { text });
    run(&s, &mut state, &web::Action::Reload {});
    // Independent redirect task, not a weakening of the original private-file task.
    let mut d = def(&s).clone();
    let request = web::Request {
        server: "report-server".into(),
        port: 8080,
        method: web::Method::Get,
        hosts: vec!["report.example.test".into()],
        path: "/private".into(),
    };
    let expected = work::HttpExpectation {
        status: 301,
        vhost: Some("report.example.test".into()),
        resource: Some("/srv/report/private".into()),
        body: String::new(),
        representation_bytes: None,
        redirect_path: Some("/private/".into()),
        cause: Some(work::ResponseCause::DirectorySlash),
    };
    d.requirements[4] = Requirement::CurrentHttp {
        request: request.clone(),
        expected: expected.clone(),
    };
    d.requirements[5] = Requirement::NextStartHttp { request, expected };
    assert!(d.validate().is_ok());
    assert!(d.checks(sim(&state)).iter().all(|r| r.0));
    let Requirement::NextStartHttp { expected, .. } = &mut d.requirements[5] else {
        panic!()
    };
    expected.redirect_path = Some("/unrelated/".into());
    let rows = d.checks(sim(&state));
    assert!(rows[4].0 && !rows[5].0);
}

#[test]
fn empty_struct_requirements_preserve_wire_but_reject_unknown_fields_and_duplicate_kind() {
    for kind in ["ActiveMatchesSaved", "SavedTextPreserved"] {
        let raw = format!("{{\"kind\":\"{kind}\"}}");
        let value: Requirement = serde_json::from_str(&raw).unwrap();
        assert_eq!(
            serde_json::to_value(value).unwrap(),
            serde_json::from_str::<serde_json::Value>(&raw).unwrap()
        );
        for raw in [
            format!("{{\"kind\":\"{kind}\",\"extra\":true}}"),
            format!("{{\"kind\":\"{kind}\",\"kind\":\"{kind}\"}}"),
            format!(
                "{{\"kind\":\"{kind}\",\"kind\":\"FilePreserved\",\"path\":\"/srv/report/index.html\"}}"
            ),
        ] {
            assert!(serde_json::from_str::<Requirement>(&raw).is_err(), "{raw}");
        }
    }
    let s = scenario();
    let mut raw = serde_json::to_value(def(&s)).unwrap();
    raw["requirements"][8]["extra"] = true.into();
    assert!(serde_json::from_value::<work::Definition>(raw).is_err());
}
#[test]
fn direct_oversized_whitespace_teaching_rejects_before_trim_without_publishing() {
    let s = scenario();
    let original = serde_json::to_value(&s).unwrap();
    let mut l = s.apache_static_teaching.clone().unwrap();
    l.prerequisite_note = " ".repeat(100_000);
    assert!(l.validate().is_err());
    let mut l = s.apache_static_teaching.clone().unwrap();
    l.concepts[0].text = "\t".repeat(100_000);
    assert!(l.validate().is_err());
    for invalid in [" ", "ok\0bad"] {
        let mut l = s.apache_static_teaching.clone().unwrap();
        l.prerequisite_note = invalid.into();
        assert!(l.validate().is_err());
    }
    assert_eq!(serde_json::to_value(&s).unwrap(), original);
}

#[test]
fn both_family_current_helpers_reject_foreign_valid_metadata_without_mutating_or_saving() {
    let apache_scenario = scenario();
    let mut all = catalog();
    assert_eq!(all.len(), 154);
    let package_scenario = all
        .iter()
        .find(|s| s.id == "lpic1-package-install")
        .unwrap()
        .clone();
    assert!(all.iter().all(|s| s.id != apache_scenario.id));
    all.push(apache_scenario.clone());

    // Both valid controls reach their real goals via approved goal-aware references.
    let mut apache = prepared(&apache_scenario);
    apache.state = good(&apache_scenario);
    apache.workspace.as_mut().unwrap().apache_static_draft =
        Some(Draft::new(sim(&apache.state)).unwrap());
    let mut package = Attempt::new(&package_scenario, 2);
    workspace::prepare(&mut package);
    dlms_learning_lab::debian_packages_teaching::prepare(
        &mut package,
        package_scenario.debian_packages_teaching.as_ref().unwrap(),
        true,
    )
    .unwrap();
    for command in &package_scenario.solution {
        engine::execute_for_goal(&mut package.state, &package_scenario.goal, command).unwrap();
    }
    let State::DebianPackages {
        simulation: package_state,
    } = &package.state
    else {
        panic!()
    };
    let session = dlms_learning_lab::debian_packages_terminal::Session {
        command: "dpkg --status unfinished-report".into(),
        ..Default::default()
    };
    assert!(session.valid_for(package_state));
    package.workspace.as_mut().unwrap().debian_packages_terminal = Some(session.clone());
    assert!(
        progress::current_apache_static_checks(&apache, &apache_scenario)
            .iter()
            .all(|r| r.0)
    );
    assert!(
        progress::current_debian_packages_checks(&package, &package_scenario)
            .iter()
            .all(|r| r.0)
    );
    assert!(progress::current_solved(&apache, &apache_scenario));
    assert!(progress::current_solved(&package, &package_scenario));

    let mut profile = Progress {
        schema_version: 1,
        ..Default::default()
    };
    profile
        .attempts
        .insert(apache_scenario.id.clone(), apache.clone());
    profile
        .attempts
        .insert(package_scenario.id.clone(), package.clone());
    let valid_portable = serde_json::to_vec(&profile).unwrap();
    let decoded = progress::decode(&valid_portable, &all).unwrap();
    assert!(
        serde_json::to_vec(&decoded).unwrap() == valid_portable,
        "valid control portable bytes differ"
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    progress::save(&path, &profile).unwrap();
    let saved = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert!(
        serde_json::to_vec(&loaded).unwrap() == valid_portable,
        "valid saved control differs"
    );

    let mut foreign_session = apache.clone();
    foreign_session
        .workspace
        .as_mut()
        .unwrap()
        .debian_packages_terminal = Some(session);
    let mut variants = vec![(true, foreign_session)];
    for (with_draft, with_learning) in [(true, false), (false, true), (true, true)] {
        let mut foreign = package.clone();
        let w = foreign.workspace.as_mut().unwrap();
        let source = apache.workspace.as_ref().unwrap();
        if with_draft {
            w.apache_static_draft = source.apache_static_draft.clone();
        }
        if with_learning {
            w.apache_static_learning = source.apache_static_learning.clone();
        }
        assert!(
            w.valid(),
            "foreign metadata is individually valid, not malformed"
        );
        variants.push((false, foreign));
    }
    for (is_apache, bad) in variants {
        let (scenario, valid) = if is_apache {
            (&apache_scenario, &apache)
        } else {
            (&package_scenario, &package)
        };
        assert!(
            bad.state == valid.state,
            "setup changed the actual solved state"
        );
        assert!(
            bad.workspace.as_ref().unwrap().valid(),
            "foreign metadata is well formed; this test targets family ownership"
        );
        let before = serde_json::to_vec(&bad).unwrap();
        let rows = if is_apache {
            progress::current_apache_static_checks(&bad, scenario)
        } else {
            progress::current_debian_packages_checks(&bad, scenario)
        };
        assert!(
            rows.len() == 1 && !rows[0].0,
            "foreign family must reject before grading"
        );
        assert!(
            rows[0].1.contains("metadata on another activity") && rows[0].1.len() <= 512,
            "foreign-family rejection needs bounded useful feedback"
        );
        assert!(!progress::current_solved(&bad, scenario));
        assert!(
            serde_json::to_vec(&bad).unwrap() == before,
            "current checks mutated full Attempt"
        );
        profile.attempts.insert(scenario.id.clone(), bad.clone());
        let invalid = serde_json::to_vec(&profile).unwrap();
        assert!(progress::decode(&invalid, &all).is_err());
        assert!(progress::save(&path, &profile).is_err());
        assert!(
            std::fs::read(&path).unwrap() == saved,
            "rejected save changed target bytes"
        );
        assert!(
            serde_json::to_vec(&profile).unwrap() == invalid,
            "rejected admission mutated caller"
        );
        assert!(
            serde_json::to_vec(&profile.attempts[&scenario.id]).unwrap() == before,
            "foreign metadata/raw drafts were silently rewritten"
        );
        profile.attempts.insert(scenario.id.clone(), valid.clone());
        assert!(
            serde_json::to_vec(&profile).unwrap() == valid_portable,
            "other state/metadata changed"
        );
    }
}

#[test]
fn optional_native_editor_kind_omits_legacy_wire_and_preserves_raw_drafts() {
    use work::EditorKind;
    let s = scenario();
    let draft = Draft::new(sim(&s.initial)).unwrap();
    let original = serde_json::to_value(&draft).unwrap();
    assert!(original.get("editor_kind").is_none());
    assert_eq!(
        serde_json::from_value::<Draft>(original.clone()).unwrap(),
        draft
    );
    for kind in [EditorKind::Config, EditorKind::File] {
        let mut d = draft.clone();
        d.editor_kind = Some(kind);
        d.config_text = "unfinished raw config".into();
        d.request_port = "bad!".into();
        assert!(d.valid_for(sim(&s.initial)));
        for raw in [
            serde_json::to_string(&d).unwrap(),
            serde_json::to_string_pretty(&d).unwrap(),
        ] {
            assert_eq!(serde_json::from_str::<Draft>(&raw).unwrap(), d);
        }
    }
    for bad in [
        serde_json::json!("future"),
        serde_json::json!("x".repeat(100_000)),
        serde_json::json!({"File":true}),
    ] {
        let mut value = original.clone();
        value["editor_kind"] = bad;
        let error = serde_json::from_value::<Draft>(value)
            .unwrap_err()
            .to_string();
        assert!(error.len() < 256, "bounded editor diagnostic");
    }
    let mut null = original;
    null["editor_kind"] = serde_json::Value::Null;
    assert_eq!(serde_json::from_value::<Draft>(null).unwrap(), draft);
}

#[test]
fn probe_feedback_names_first_status_site_resource_page_text_and_length_difference() {
    let s = scenario();
    let state = good(&s);
    let before = serde_json::to_value(&state).unwrap();
    for (kind, words) in [
        ("status", vec!["HTTP 200 returned", "expected HTTP 403"]),
        (
            "site",
            vec![
                "Selected site report.example.test",
                "expected other.example.test",
            ],
        ),
        (
            "resource",
            vec![
                "Mapped resource /srv/report/index.html",
                "expected /srv/observer/index.html",
            ],
        ),
        ("body", vec!["Page text differs", "UTF-8 bytes"]),
        (
            "length",
            vec!["Page length is 13 UTF-8 bytes", "expected 14"],
        ),
    ] {
        let mut d = def(&s).clone();
        let Requirement::CurrentHttp { request, expected } = &mut d.requirements[0] else {
            panic!("positive current fixture row")
        };
        match kind {
            "status" => {
                expected.status = 403;
                expected.body.clear();
                expected.representation_bytes = None;
                expected.cause = Some(work::ResponseCause::AccessDenied);
            }
            "site" => expected.vhost = Some("other.example.test".into()),
            "resource" => {
                expected.vhost = Some("REPORT.EXAMPLE.TEST".into());
                expected.resource = Some("/srv/observer/index.html".into());
            }
            "body" => expected.body = "Wrong report\n".into(),
            "length" => {
                request.method = web::Method::Head;
                expected.body.clear();
                expected.representation_bytes = Some(14);
            }
            _ => unreachable!(),
        }
        d.validate().unwrap();
        let rows = d.checks(sim(&state));
        assert!(!rows[0].0);
        for word in words {
            assert!(
                rows[0].1.contains(word),
                "first differing response field is named"
            );
        }
        assert!(
            !rows[0].1.contains("declared")
                && !rows[0].1.contains("Report ready")
                && !rows[0].1.contains("Wrong report"),
            "no developer jargon or page-content dump"
        );
        assert!(rows.iter().all(|r| r.1.len() <= 512));
        assert!(
            serde_json::to_value(&state).unwrap() == before,
            "diagnosis is read-only"
        );
    }
    assert!(
        def(&s).checks(sim(&state)).iter().all(|r| r.0),
        "independent original expected rows were not replaced"
    );
}
#[test]
fn probe_feedback_distinguishes_redirect_location_and_same403_wrong_reason() {
    let s = scenario();
    let mut state = good(&s);
    let mut d = def(&s).clone();
    for inventory in [&mut d.initial.inventory, &mut sim_mut(&mut state).inventory] {
        inventory
            .entries
            .insert("/srv/report/docs".into(), web::EntryKind::Directory);
    }
    let Requirement::CurrentHttp { request, expected } = &mut d.requirements[0] else {
        panic!("current row")
    };
    request.path = "/docs".into();
    expected.status = 301;
    expected.resource = Some("/srv/report/docs".into());
    expected.body.clear();
    expected.representation_bytes = None;
    expected.redirect_path = Some("/elsewhere/".into());
    expected.cause = Some(work::ResponseCause::DirectorySlash);
    d.validate().unwrap();
    let before = serde_json::to_value(&state).unwrap();
    let row = &d.checks(sim(&state))[0];
    assert!(
        !row.0
            && row.1.contains("Redirect location /docs/")
            && row.1.contains("expected /elsewhere/")
    );
    assert!(serde_json::to_value(&state).unwrap() == before);
    // The authored access task has a genuine same403 false positive that only
    // the response reason distinguishes: no index is not an access-rule denial.
    let s = pack::load(include_str!("../scenarios/apache-static-v1.json"))
        .unwrap()
        .remove(2);
    let (index, request) = def(&s)
        .requirements
        .iter()
        .enumerate()
        .find_map(|(i, r)| match r {
            Requirement::CurrentHttp { request, expected }
                if request.path == "/private/" && expected.status == 403 =>
            {
                Some((i, request))
            }
            _ => None,
        })
        .unwrap();
    let web::Outcome::Http(actual) = web::request(sim(&s.initial), request).unwrap() else {
        panic!("active public server")
    };
    assert_eq!(actual.status.code(), 403);
    assert_eq!(actual.cause, web::Cause::NoIndex);
    let row = &def(&s).checks(sim(&s.initial))[index];
    assert!(
        !row.0
            && row.1.contains("no directory page was available")
            && row.1.contains("denial by the directory access rule")
    );
}
#[test]
fn probe_feedback_excerpts_are_bounded_and_no_listener_stays_distinct() {
    let s = scenario();
    let state = good(&s);
    let mut d = def(&s).clone();
    let long = [
        "a".repeat(63),
        "b".repeat(63),
        "c".repeat(63),
        "d".repeat(61),
    ]
    .join(".");
    assert_eq!(long.len(), 253);
    let Requirement::CurrentHttp { expected, .. } = &mut d.requirements[0] else {
        panic!()
    };
    expected.vhost = Some(long.clone());
    d.validate().unwrap();
    let row = &d.checks(sim(&state))[0];
    assert!(!row.0 && row.1.len() <= 512 && row.1.contains("...") && !row.1.contains(&long));
    let mut stopped = state;
    run(&s, &mut stopped, &web::Action::Stop {});
    let rows = def(&s).checks(sim(&stopped));
    assert!(
        !rows[0].0
            && rows[0].1.contains("no active virtual listener")
            && rows[0].1.contains("expected HTTP 200")
    );
    assert!(rows.iter().all(|r| r.1.len() <= 512));
}
