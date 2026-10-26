//! Authored static-site assignments. All requests and repairs are in-process.
use dlms_learning_lab::{
    apache_static::{self as web, Action, Method, Outcome, Request},
    apache_static_teaching as teaching,
    apache_static_workspace::{self as work, Requirement},
    backup,
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};
use std::sync::atomic::{AtomicBool, AtomicUsize};
fn labs() -> Vec<Scenario> {
    pack::load(include_str!("../scenarios/apache-static-v1.json")).unwrap()
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
fn definition(s: &Scenario) -> &work::Definition {
    let Goal::ApacheStatic { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn apply(s: &Scenario, state: &mut State, a: Action) {
    engine::execute_for_goal(
        state,
        &s.goal,
        &format!("{}{}", work::PREFIX, serde_json::to_string(&a).unwrap()),
    )
    .unwrap();
}
fn solve(s: &Scenario) -> State {
    let mut state = s.initial.clone();
    for line in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, line).unwrap();
    }
    assert!(
        engine::solved(&state, &s.goal),
        "{}: {:?}",
        s.id,
        engine::checks(&state, &s.goal)
    );
    state
}
fn query(state: &State, host: &str, path: &str, method: Method) -> web::Response {
    let r = Request {
        server: sim(state).inventory.server.clone(),
        port: 8080,
        method,
        hosts: vec![host.into()],
        path: path.into(),
    };
    let Outcome::Http(response) = web::request(sim(state), &r).unwrap() else {
        panic!("Need actual virtual serving")
    };
    response
}
#[test]
fn four_distinct_assignments_have_coherent_prerequisites_teaching_and_independent_expected_outcomes()
 {
    let all = labs();
    assert_eq!(all.len(), 4);
    for (i, s) in all.iter().enumerate() {
        assert_eq!(s.track, "LPIC-2");
        assert_eq!(s.unit, "static-web-services");
        assert_eq!(
            s.prerequisites,
            if i == 0 {
                vec![]
            } else {
                vec![all[i - 1].id.clone()]
            }
        );
        let lesson = s.apache_static_teaching.as_ref().unwrap();
        assert_eq!(lesson.prerequisites, s.prerequisites);
        assert!(lesson.separate_from(sim(&s.initial)));
        assert_eq!(lesson.concepts.len(), 3);
        assert!(!engine::solved(&s.initial, &s.goal));
        assert!(
            s.sources
                .iter()
                .any(|u| u.contains("httpd.apache.org/docs/2.4"))
        );
        let result = solve(s);
        match i {
            0 => {
                for (host, body, site) in [
                    (
                        "welcome.example.test",
                        "Welcome desk\n",
                        "welcome.example.test",
                    ),
                    (
                        "records.example.test",
                        "Records catalog\n",
                        "records.example.test",
                    ),
                    (
                        "catalog.example.test",
                        "Records catalog\n",
                        "records.example.test",
                    ),
                    (
                        "unknown.example.test",
                        "Welcome desk\n",
                        "welcome.example.test",
                    ),
                ] {
                    let r = query(&result, host, "/", Method::Get);
                    assert_eq!(r.status.code(), 200);
                    assert_eq!(r.body, body);
                    assert_eq!(r.vhost.as_deref(), Some(site));
                }
            }
            1 => {
                let get = query(&result, "docs.example.test", "/", Method::Get);
                assert_eq!(get.body, "Welcome to the repair guide.\n");
                assert_eq!(get.resource.as_deref(), Some("/srv/docs/intro.html"));
                let head = query(&result, "docs.example.test", "/", Method::Head);
                assert!(head.body.is_empty());
                assert_eq!(head.representation_bytes, Some(29));
                let redirect = query(&result, "docs.example.test", "/manual", Method::Get);
                assert_eq!(redirect.status.code(), 301);
                assert_eq!(redirect.redirect_path.as_deref(), Some("/manual/"));
                assert_eq!(
                    query(&result, "docs.example.test", "/manual/", Method::Get).body,
                    "Reference pages\n"
                );
            }
            2 => {
                assert_eq!(
                    query(&result, "hub.example.test", "/", Method::Get).body,
                    "Visitor guide\n"
                );
                assert_eq!(
                    query(&result, "hub.example.test", "/private-notes/", Method::Get).body,
                    "Public safety notes\n"
                );
                for p in ["/private/secret.txt", "/private/"] {
                    assert_eq!(
                        query(&result, "hub.example.test", p, Method::Get)
                            .status
                            .code(),
                        403
                    );
                }
                assert_eq!(
                    sim(&result).files["/srv/hub/private/secret.txt"],
                    "Staff only\n"
                );
            }
            3 => {
                for name in ["release.example.test", "download.example.test"] {
                    assert_eq!(
                        query(&result, name, "/", Method::Get).body,
                        "Release 2 ready\n"
                    );
                }
                assert_eq!(
                    query(&result, "status.example.test", "/", Method::Get).body,
                    "Status healthy\n"
                );
                assert_eq!(
                    query(
                        &result,
                        "release.example.test",
                        "/private/notes.txt",
                        Method::Get
                    )
                    .status
                    .code(),
                    403
                );
                assert_eq!(
                    sim(&result).files["/srv/archive/index.html"],
                    "Retired release\n"
                );
                assert_eq!(
                    query(
                        &result,
                        "download.example.test",
                        "/private/notes.txt",
                        Method::Get
                    )
                    .status
                    .code(),
                    403
                );
            }
            _ => unreachable!(),
        }
    }
}
#[test]
fn semantic_alternatives_reordered_live_file_writes_and_stop_start_are_valid() {
    for s in labs() {
        let solved = solve(&s);
        let mut current = s.initial.clone();
        // Publish requested live files first; activate equivalent supported text with Stop/Start.
        for (path, text) in &sim(&solved).files {
            if sim(&current).files.get(path) != Some(text) {
                apply(
                    &s,
                    &mut current,
                    Action::WriteFile {
                        path: path.clone(),
                        text: text.clone(),
                    },
                );
            }
        }
        let alternate = format!(
            "# independent case/layout alternative\n\n{}\n",
            sim(&solved)
                .saved
                .replace("ServerName ", "servername ")
                .replace("Require all ", "require all ")
                .replace("Listen 8080", "listen 8080")
        );
        apply(&s, &mut current, Action::SaveConfig { text: alternate });
        apply(&s, &mut current, Action::Stop {});
        apply(&s, &mut current, Action::Start {});
        assert!(engine::solved(&current, &s.goal), "{}", s.id);
        assert_ne!(
            sim(&current).saved,
            sim(&solved).saved,
            "No exact hidden document requirement"
        );
    }
}
#[test]
fn common_shortcuts_partial_repairs_and_broken_success_fail_current_outcomes() {
    for s in labs() {
        let pass = solve(&s);
        let mut stopped = pass.clone();
        apply(&s, &mut stopped, Action::Stop {});
        assert!(
            !engine::solved(&stopped, &s.goal),
            "stopped is not denied {}",
            s.id
        );
        let rows = engine::checks(&stopped, &s.goal);
        assert!(rows.iter().any(|r| r.0) && rows.iter().any(|r| !r.0));
        let mut saved_only = s.initial.clone();
        apply(
            &s,
            &mut saved_only,
            Action::SaveConfig {
                text: sim(&pass).saved.clone(),
            },
        );
        assert!(
            !engine::solved(&saved_only, &s.goal),
            "Save alone does not repair all layers {}",
            s.id
        );
        let mut corrupted = pass.clone();
        apply(
            &s,
            &mut corrupted,
            Action::SaveConfig {
                text: "Listen 8080\n<VirtualHost *:8080>\n".into(),
            },
        );
        let old_active = sim(&corrupted).active.clone();
        let before = corrupted.clone();
        assert!(sim_mut(&mut corrupted).apply(&Action::Reload {}).is_err());
        assert_eq!(corrupted, before);
        assert_eq!(sim(&corrupted).active, old_active);
        let rows = engine::checks(&corrupted, &s.goal);
        assert!(rows.iter().any(|r| r.0) && rows.iter().any(|r| !r.0));
        assert!(
            !engine::solved(&corrupted, &s.goal),
            "current page does not prove saved next start {}",
            s.id
        );
        let mut broken = pass.clone();
        let public = definition(&s)
            .requirements
            .iter()
            .find_map(|r| match r {
                Requirement::CurrentHttp { expected, .. } if expected.status == 200 => {
                    expected.resource.clone()
                }
                _ => None,
            })
            .unwrap();
        apply(
            &s,
            &mut broken,
            Action::WriteFile {
                path: public,
                text: "Wrong but superficially successful page\n".into(),
            },
        );
        assert!(
            !engine::solved(&broken, &s.goal),
            "HTTP200 is not the intended content {}",
            s.id
        );
    }
}
#[test]
fn every_outcome_family_has_a_real_state_negative_and_no_history_credit() {
    for s in labs() {
        let solved = solve(&s);
        for (index, rule) in definition(&s).requirements.iter().enumerate() {
            let mut broken = solved.clone();
            match rule {
                Requirement::CurrentHttp { .. } => apply(&s, &mut broken, Action::Stop {}),
                Requirement::NextStartHttp { .. } => apply(
                    &s,
                    &mut broken,
                    Action::SaveConfig {
                        text: "Listen 8080\n<VirtualHost *:8080>\n".into(),
                    },
                ),
                Requirement::FilePreserved { path } | Requirement::FileMatches { path, .. } => {
                    apply(
                        &s,
                        &mut broken,
                        Action::WriteFile {
                            path: path.clone(),
                            text: "Changed after passing\n".into(),
                        },
                    )
                }
                Requirement::ActiveMatchesSaved {} => {
                    let text = format!("{}\n", sim(&solved).saved);
                    apply(&s, &mut broken, Action::SaveConfig { text });
                }
                Requirement::SavedTextPreserved {} => panic!("No hidden exact-text objective"),
            }
            let rows = engine::checks(&broken, &s.goal);
            assert!(!rows[index].0, "{} row {index}: {}", s.id, rows[index].1);
            assert!(!engine::solved(&broken, &s.goal));
            assert!(
                engine::solved(&solved, &s.goal),
                "probe never mutates original"
            );
        }
    }
}

#[test]
fn duplicate_site_names_cannot_hide_an_alias_private_file_leak() {
    let all = labs();
    let s = &all[3];
    let mut state = solve(s);
    let text = sim(&state)
        .saved
        .replace("    ServerAlias download.example.test\n", "");
    let bypass = format!(
        "{text}<VirtualHost *:8080>\nServerName release.example.test\nServerAlias download.example.test\nDocumentRoot \"/srv/release\"\nDirectoryIndex index.html\n<Directory \"/srv/release\">\nRequire all granted\n</Directory>\n</VirtualHost>\n"
    );
    apply(s, &mut state, Action::SaveConfig { text: bypass });
    apply(s, &mut state, Action::Reload {});
    assert_eq!(
        query(
            &state,
            "release.example.test",
            "/private/notes.txt",
            Method::Get
        )
        .status
        .code(),
        403
    );
    assert_eq!(
        query(
            &state,
            "download.example.test",
            "/private/notes.txt",
            Method::Get
        )
        .status
        .code(),
        200
    );
    let rows = engine::checks(&state, &s.goal);
    let failed: Vec<_> = rows
        .iter()
        .enumerate()
        .filter_map(|(n, r)| (!r.0).then_some(n))
        .collect();
    assert_eq!(
        failed,
        vec![6, 7],
        "both alias-private promises fail; other rows still pass"
    );
    assert!(!engine::solved(&state, &s.goal));
}

#[test]
fn each_example_insert_save_experiment_repair_is_ungraded_and_assignment_draft_stays_exact() {
    for s in labs() {
        let lesson = s.apache_static_teaching.as_ref().unwrap();
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, lesson).unwrap();
        let w = a.workspace.as_mut().unwrap();
        w.apache_static_draft.as_mut().unwrap().config_text =
            "# unfinished independent assignment\n".into();
        let before_state = a.state.clone();
        let before_draft = serde_json::to_value(w.apache_static_draft.as_ref().unwrap()).unwrap();
        let l = w.apache_static_learning.as_mut().unwrap();
        l.view = View::Example;
        for (idx, step) in lesson.example_actions.iter().enumerate() {
            l.selected_action = idx;
            if matches!(
                step.action,
                Action::SaveConfig { .. } | Action::WriteFile { .. }
            ) {
                let previous = l.simulation.clone();
                l.insert_selected(lesson).unwrap();
                assert_eq!(l.simulation, previous, "insert never executes");
            }
            l.perform(lesson, &step.action).unwrap();
        }
        assert!(l.checks(lesson).iter().all(|r| r.0));
        for action in &lesson.try_change.actions {
            l.perform(lesson, action).unwrap();
        }
        assert_eq!(
            lesson
                .definition
                .checks(&l.simulation)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            lesson.try_change.expected,
            "{}",
            s.id
        );
        assert!(l.checks(lesson).iter().any(|r| !r.0));
        for action in &lesson.repair {
            l.perform(lesson, action).unwrap();
        }
        assert!(l.checks(lesson).iter().all(|r| r.0));
        assert_eq!(a.state, before_state);
        assert_eq!(
            serde_json::to_value(
                a.workspace
                    .as_ref()
                    .unwrap()
                    .apache_static_draft
                    .as_ref()
                    .unwrap()
            )
            .unwrap(),
            before_draft
        );
        assert!(!progress::current_solved(&a, &s));
    }
}
#[test]
fn pending_drafts_query_edits_reset_and_actual_backup_restore_preserve_all_four_attempts() {
    let all = labs();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &all {
        let mut a = Attempt::new(s, 1);
        a.state = solve(s);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, s.apache_static_teaching.as_ref().unwrap()).unwrap();
        assert!(progress::current_solved(&a, s));
        let d = a
            .workspace
            .as_mut()
            .unwrap()
            .apache_static_draft
            .as_mut()
            .unwrap();
        d.request_port = "bad!".into();
        d.request_hosts = vec!["unsupported_host".into()];
        assert!(
            progress::current_solved(&a, s),
            "read-only malformed query is not pending configuration"
        );
        a.workspace
            .as_mut()
            .unwrap()
            .apache_static_draft
            .as_mut()
            .unwrap()
            .config_text
            .push_str("# unfinished edit\n");
        assert!(!progress::current_solved(&a, s));
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .apache_static_learning
            .as_mut()
            .unwrap();
        l.view = View::Example;
        l.draft.config_text = "# separate unfinished practice\n".into();
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    progress::save(&path, &p).unwrap();
    let target = std::fs::read(&path).unwrap();
    let expected = serde_json::to_value(&p).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(serde_json::to_value(&loaded).unwrap(), expected);
    let export = dir.path().join("backup.json");
    backup::export(&export, &loaded, &all).unwrap();
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    backup::restore(&path, &candidate, &all).unwrap();
    let mut restored = progress::load(&path, &all).unwrap();
    assert_eq!(serde_json::to_value(&restored).unwrap(), expected);
    let other = serde_json::to_value(&restored.attempts[&all[1].id]).unwrap();
    restored
        .attempts
        .insert(all[0].id.clone(), Attempt::new(&all[0], 2));
    assert_eq!(
        serde_json::to_value(&restored.attempts[&all[1].id]).unwrap(),
        other
    );
    assert_eq!(
        std::fs::read(&path).unwrap(),
        target,
        "reset stays in-memory until explicit save"
    );
}
#[test]
fn unsound_pack_references_contexts_and_missing_positive_guards_are_rejected() {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/apache-static-v1.json")).unwrap();
    for mutate in 0..4 {
        let mut bad = raw.clone();
        match mutate {
            0 => bad["scenarios"][0]["goal"]["definition"]["requirements"]
                .as_array_mut()
                .unwrap()
                .retain(|r| r["kind"] != "NextStartHttp"),
            1 => bad["scenarios"][0]["solution"] = serde_json::json!([]),
            2 => {
                bad["scenarios"][0]["apache_static_teaching"]["definition"]["initial"]["inventory"]
                    ["server"] = "routing-task-server".into()
            }
            3 => {
                bad["scenarios"][0]["goal"]["definition"]["initial"]["inventory"]["fixture"] =
                    "foreign-fixture".into()
            }
            _ => unreachable!(),
        }
        assert!(
            pack::load(&serde_json::to_string(&bad).unwrap()).is_err(),
            "bad case {mutate}"
        );
    }
}
