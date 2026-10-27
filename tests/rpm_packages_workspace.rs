//! Prospective app/progress tests. No host package utilities or fabricated historical runs.
use dlms_learning_lab::{
    backup, engine, pack,
    progress::{self, Attempt, Progress},
    rpm_packages as core, rpm_packages_teaching as teaching, rpm_packages_terminal as terminal,
    rpm_packages_workspace as work, workspace,
};
use std::sync::atomic::{AtomicBool, AtomicUsize};
fn scenario() -> pack::Scenario {
    pack::load(include_str!("fixtures/rpm-workspace-preview.json"))
        .unwrap()
        .remove(0)
}
fn d(s: &pack::Scenario) -> &work::Definition {
    let engine::Goal::RpmPackages { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn live(a: &Attempt) -> &core::State {
    let engine::State::RpmPackages { simulation } = &a.state else {
        panic!()
    };
    simulation
}
fn solve(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    for c in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    a
}
fn exact<T: serde::Serialize>(a: &T, b: &serde_json::Value) {
    assert!(
        serde_json::to_value(a).unwrap() == *b,
        "Complete bounded serialized value differs"
    );
}
#[test]
fn all_current_positive_rows_and_preservation_require_real_outcomes() {
    let s = scenario();
    let mut a = Attempt::new(&s, 1);
    assert_eq!(
        progress::current_rpm_checks(&a, &s)
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>(),
        vec![true, false, false, false, true, true]
    );
    engine::execute_for_goal(&mut a.state, &s.goal, "rpm -i /pkgs/provider-a.rpm").unwrap();
    assert_eq!(
        progress::current_rpm_checks(&a, &s)
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>(),
        vec![true, true, false, true, true, true]
    );
    engine::execute_for_goal(&mut a.state, &s.goal, "rpm --reinstall /pkgs/client-a.rpm").unwrap();
    assert!(progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, "rpm -e desk-client").unwrap();
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn missing_optional_session_never_infers_pending_and_old_outputs_never_grade() {
    let s = scenario();
    let mut a = solve(&s);
    a.workspace = None;
    assert!(progress::current_solved(&a, &s));
    workspace::prepare(&mut a);
    let t = a.workspace.as_mut().unwrap().rpm_session.as_mut().unwrap();
    t.command = "unrun ; $bad".into();
    t.entries.push(terminal::Entry {
        command: "rpm -q anything".into(),
        output: terminal::Outcome::Utility {
            stdout: "made-up completion".into(),
            stderr: String::new(),
            status: 0,
            summary: None,
        },
    });
    assert!(progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, "rpm -e desk-client").unwrap();
    a.workspace.as_mut().unwrap().checked = true;
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn full_immutable_catalog_pin_and_negative_only_goals_fail() {
    let s = scenario();
    let mut a = solve(&s);
    let mut c = live(&a).catalog().clone();
    c.fixture = "foreign-valid".into();
    a.state = engine::State::RpmPackages {
        simulation: core::State::new(c, live(&a).runtime().clone()).unwrap(),
    };
    assert!(!progress::current_solved(&a, &s));
    let mut def = d(&s).clone();
    def.requirements = vec![work::Requirement::Absent {
        package: core::Identity {
            name: "ink-bridge".into(),
            arch: core::Architecture::Noarch,
        },
    }];
    assert!(def.validate().is_err());
    let mut def = d(&s).clone();
    def.archive_paths
        .insert("/pkgs/another.rpm".into(), "client-a".into());
    assert!(def.validate().is_err());
}
#[test]
fn actual_epoch_update_preserves_local_config_and_current_provider_health() {
    let s = scenario();
    let mut def = d(&s).clone();
    let mut c = def.initial.catalog().clone();
    let mut next = c.archives["client-a"].clone();
    next.evr = core::Evr::parse("0002:0.1-1").unwrap();
    next.files
        .get_mut("/usr/share/desk/report")
        .unwrap()
        .file
        .contents = "updated\n".into();
    c.archives.insert("client-new".into(), next);
    let mut r = def.initial.runtime().clone();
    r.available.push("client-new".into());
    def.initial = core::State::new(c, r).unwrap();
    def.archive_paths
        .insert("/pkgs/client-new.rpm".into(), "client-new".into());
    if let work::Requirement::Installed { version, .. } = &mut def.requirements[0] {
        *version = Some(core::Constraint {
            relation: core::Relation::Ge,
            evr: core::Evr::parse("2:0.1-1").unwrap(),
        });
    }
    def.validate().unwrap();
    let mut x = def.initial.clone();
    for c in ["rpm -i /pkgs/provider-b.rpm", "rpm -U /pkgs/client-new.rpm"] {
        assert!(matches!(
            terminal::execute(&def, &mut x, c),
            terminal::Outcome::Utility { status: 0, .. }
        ));
    }
    assert!(def.checks(&x).iter().all(|r| r.0));
    let query = format!("rpm -q --queryformat '{}' desk-client", terminal::FORMAT);
    assert!(
        matches!(terminal::execute(&def,&mut x,&query),terminal::Outcome::Utility{stdout,..} if stdout=="desk-client\t2:0.1-1.noarch\n")
    );
}
#[test]
fn foreign_typed_metadata_is_rejected_both_directions_without_touching_target() {
    let s = scenario();
    let mut a = solve(&s);
    let all = pack::bundled().unwrap();
    let script = all.iter().find(|s| s.teaching.is_some()).unwrap();
    let mut old = Attempt::new(script, 1);
    workspace::prepare(&mut old);
    dlms_learning_lab::script_teaching::prepare(&mut old, script.teaching.as_ref().unwrap(), true);
    let l = old.workspace.as_ref().unwrap().script_learning.clone();
    assert!(l.as_ref().unwrap().valid());
    a.workspace.as_mut().unwrap().script_learning = l;
    assert!(!progress::current_solved(&a, &s));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("keep.json");
    std::fs::write(&path, b"old-target").unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"old-target");
    let gateway = all.iter().find(|s| s.id == "ccna-gateway").unwrap();
    let mut old = Attempt::new(gateway, 1);
    for c in &gateway.solution {
        engine::execute_for_goal(&mut old.state, &gateway.goal, c).unwrap();
    }
    assert!(progress::current_solved(&old, gateway));
    workspace::prepare(&mut old);
    old.workspace.as_mut().unwrap().rpm_session = Some(terminal::Session::default());
    assert!(!progress::current_solved(&old, gateway));
}
#[test]
fn no_context_free_mutation_and_pretrim_length_cannot_be_bypassed() {
    let s = scenario();
    let mut a = Attempt::new(&s, 1);
    let before = serde_json::to_value(&a).unwrap();
    assert!(engine::execute(&mut a.state, "rpm -i /pkgs/provider-a.rpm").is_err());
    assert!(
        engine::execute_for_goal(
            &mut a.state,
            &s.goal,
            &format!("{}rpm -q desk-client", " ".repeat(2049))
        )
        .is_err()
    );
    exact(&a, &before);
}
#[test]
fn bound_guards_precede_clone_and_max_escaped_state_has_a_closed_frontier() {
    let s = scenario();
    let original = d(&s).initial.clone();
    let construct = |n: usize| {
        let mut c = original.catalog().clone();
        c.archives
            .get_mut("client-a")
            .unwrap()
            .files
            .get_mut("/usr/share/desk/report")
            .unwrap()
            .file
            .contents = "\u{1}".repeat(n);
        c.archives
            .get_mut("provider-a")
            .unwrap()
            .files
            .get_mut("/usr/share/ink/label")
            .unwrap()
            .file
            .contents = "\u{1}".repeat(n);
        core::State::new(c, original.runtime().clone()).unwrap()
    };
    let mut lo = 0usize;
    let mut hi = 4096;
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        if work::admitted(&construct(mid)) {
            lo = mid
        } else {
            hi = mid - 1
        }
    }
    let accepted = construct(lo);
    assert!(work::admitted(&accepted));
    assert!(serde_json::to_vec(&accepted).unwrap().len() <= work::MAX_STATE);
    if lo < 4096 {
        assert!(!work::admitted(&construct(lo + 1)));
    }
    let huge = construct(4096);
    assert!(!work::admitted(&huge));
    let mut def = d(&s).clone();
    def.initial = huge;
    assert!(def.validate().is_err());
    let mut def = d(&s).clone();
    if let work::Requirement::OwnedPayloadMatches { path, .. } = &mut def.requirements[2] {
        *path = "x".repeat(100_000);
    }
    assert!(def.validate().is_err());
}
#[test]
fn original143_and_nil_old_wire_remain_exact_under_registered_rpm_and_raw_limits() {
    let all = pack::bundled().unwrap();
    assert_eq!(all.len(), 147);
    assert!(all[..143].iter().all(|s| s.rpm_packages_teaching.is_none()
        && !matches!(s.initial, engine::State::RpmPackages { .. })));
    assert_eq!(
        serde_json::to_value(workspace::WorkspaceData::new())
            .unwrap()
            .get("rpm_session"),
        None
    );
    let f = scenario();
    assert!(serde_json::to_vec(&f).unwrap().len() <= work::MAX_SCENARIO);
    let raw = serde_json::from_str::<serde_json::Value>(include_str!(
        "fixtures/rpm-workspace-preview.json"
    ))
    .unwrap();
    let mut unknown = raw.clone();
    unknown["scenarios"][0]["goal"]["definition"]["requirements"][3]["unknown"] =
        serde_json::json!(1);
    assert!(pack::load(&serde_json::to_string(&unknown).unwrap()).is_err());
}
#[test]
fn real6_7_8_histories_two_sessions_full_restore_and_selected_reset_are_preserved() {
    let raw = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let mut all = pack::bundled().unwrap();
    let mut old = progress::decode(raw, &all).unwrap();
    for a in old.attempts.values_mut() {
        workspace::prepare(a);
    }
    let records = old
        .attempts
        .values()
        .filter_map(|a| a.workspace.as_ref())
        .flat_map(|w| w.script_history.iter().chain(&w.script_history_more))
        .collect::<Vec<_>>();
    for (revision, count) in [(6, 33), (7, 37), (8, 40)] {
        assert_eq!(
            records
                .iter()
                .filter(|r| r.engine_revision == revision)
                .count(),
            count,
            "Exact genuine historical counts"
        );
    }
    let before = serde_json::to_value(&old.attempts).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let control = dir.path().join("old-only.json");
    backup::export(&control, &old, &all).unwrap();
    let old_candidate = backup::inspect(
        &control,
        &all,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    let s = scenario();
    let mut selected = solve(&s);
    teaching::prepare(
        &mut selected,
        d(&s),
        s.rpm_packages_teaching.as_ref().unwrap(),
        true,
    )
    .unwrap();
    let w = selected.workspace.as_mut().unwrap();
    w.rpm_session
        .as_mut()
        .unwrap()
        .insert("rpm -q --path /usr/share/desk/report")
        .unwrap();
    let l = w.rpm_learning.as_mut().unwrap();
    let source = s.rpm_packages_teaching.as_ref().unwrap();
    for c in &source.example_commands {
        l.insert(source, &c.command).unwrap();
        l.run(source).unwrap();
    }
    l.insert(source, "rpm -q museum-panel").unwrap();
    let expected = serde_json::to_value(&selected).unwrap();
    old.attempts.insert(s.id.clone(), selected);
    old.favorites.insert(s.id.clone());
    old.clock += 1;
    all.push(s.clone());
    let path = dir.path().join("progress.json");
    std::fs::write(&path, raw).unwrap();
    let slot = include_bytes!("fixtures/shell-v7-synthetic-writer-progress.json");
    std::fs::write(path.with_extension("before-shell-v8.json"), slot).unwrap();
    progress::save(&path, &old).unwrap();
    assert_eq!(
        std::fs::read(path.with_extension("before-shell-v10.json")).unwrap(),
        raw
    );
    assert_eq!(
        std::fs::read(path.with_extension("before-shell-v8.json")).unwrap(),
        slot
    );
    let loaded = progress::load(&path, &all).unwrap();
    for (id, v) in before.as_object().unwrap() {
        exact(&loaded.attempts[id], v);
    }
    exact(&loaded.attempts[&s.id], &expected);
    let exported = dir.path().join("backup.json");
    backup::export(&exported, &loaded, &all).unwrap();
    let candidate = backup::inspect(
        &exported,
        &all,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    exact(&candidate.progress.attempts[&s.id], &expected);
    for (id, a) in &old_candidate.progress.attempts {
        exact(
            &candidate.progress.attempts[id],
            &serde_json::to_value(a).unwrap(),
        );
    }
    let prior = std::fs::read(&path).unwrap();
    let restored = backup::restore(&path, &candidate, &all).unwrap();
    assert_eq!(std::fs::read(restored.safety_copy.unwrap()).unwrap(), prior);
    let mut restored = progress::load(&path, &all).unwrap();
    exact(&restored.attempts[&s.id], &expected);
    for (id, a) in &old_candidate.progress.attempts {
        exact(&restored.attempts[id], &serde_json::to_value(a).unwrap());
    }
    restored
        .attempts
        .insert(s.id.clone(), Attempt::new(&s, restored.clock + 1));
    assert!(!progress::current_solved(&restored.attempts[&s.id], &s));
    for (id, a) in &old_candidate.progress.attempts {
        exact(&restored.attempts[id], &serde_json::to_value(a).unwrap());
    }
    assert!(restored.favorites.contains(&s.id));
    assert_eq!(
        include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json"),
        raw
    );
}
#[path = "support/actual9_evidence.rs"]
mod evidence;
#[test]
#[ignore = "Requires retained exact real public9 COMPLETE; root-only explicit guarded replay, no history generation"]
fn actual9_full_selected_sessions_and_old_records_and_backup_bytes_remain_exact() {
    let raw = evidence::original9();
    let mut all = pack::bundled().unwrap();
    let mut p = progress::decode(&raw, &all).unwrap();
    let before = serde_json::to_value(&p.attempts).unwrap();
    assert_eq!(p.attempts.len(), 133);
    let s = scenario();
    let mut a = solve(&s);
    teaching::prepare(
        &mut a,
        d(&s),
        s.rpm_packages_teaching.as_ref().unwrap(),
        true,
    )
    .unwrap();
    a.workspace
        .as_mut()
        .unwrap()
        .rpm_session
        .as_mut()
        .unwrap()
        .insert("rpm -q desk-client")
        .unwrap();
    {
        let l = a.workspace.as_mut().unwrap().rpm_learning.as_mut().unwrap();
        let source = s.rpm_packages_teaching.as_ref().unwrap();
        for c in &source.example_commands {
            l.insert(source, &c.command).unwrap();
            l.run(source).unwrap();
        }
        l.insert(source, "rpm -qp --requires /museum/rpms/panel-a.rpm")
            .unwrap();
    }
    let selected = serde_json::to_value(&a).unwrap();
    p.attempts.insert(s.id.clone(), a);
    all.push(s.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    std::fs::write(&path, &raw).unwrap();
    progress::save(&path, &p).unwrap();
    assert_eq!(
        std::fs::read(path.with_extension("before-shell-v10.json")).unwrap(),
        raw
    );
    let loaded = progress::load(&path, &all).unwrap();
    for (id, v) in before.as_object().unwrap() {
        evidence::exact(&loaded.attempts[id], v);
    }
    evidence::exact(&loaded.attempts[&s.id], &selected);
    assert_eq!(loaded.attempts.len(), 134);
    assert_eq!(evidence::original9(), raw);
}

#[test]
fn direct_nul_session_fields_match_decoder_and_generic_save_is_atomic() {
    let s = scenario();
    let mut baseline = solve(&s);
    {
        let t = baseline
            .workspace
            .as_mut()
            .unwrap()
            .rpm_session
            .as_mut()
            .unwrap();
        let engine::State::RpmPackages { simulation } = &mut baseline.state else {
            panic!()
        };
        for command in ["rpm --reinstall /pkgs/client-a.rpm", "rpm -q desk-client"] {
            t.insert(command).unwrap();
            t.run(d(&s), simulation).unwrap();
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("profile.json");
    let mut good = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    good.attempts.insert(s.id.clone(), baseline.clone());
    progress::save(&target, &good).unwrap();
    let prior = std::fs::read(&target).unwrap();
    let loaded = progress::load(&target, std::slice::from_ref(&s)).unwrap();
    exact(
        &loaded.attempts[&s.id],
        &serde_json::to_value(&baseline).unwrap(),
    );
    for field in ["entry-command", "stdout", "summary", "reason"] {
        let mut t = terminal::Session::default();
        let command = if field == "entry-command" {
            "rpm -q desk-client\0"
        } else {
            "rpm -q desk-client"
        };
        let output = match field {
            "stdout" => terminal::Outcome::Utility {
                stdout: "bad\0data".into(),
                stderr: String::new(),
                status: 0,
                summary: None,
            },
            "summary" => terminal::Outcome::Utility {
                stdout: String::new(),
                stderr: String::new(),
                status: 0,
                summary: Some("bad\0summary".into()),
            },
            "reason" => terminal::Outcome::SimulatorUnsupported {
                reason: "bad\0reason".into(),
            },
            _ => terminal::Outcome::Utility {
                stdout: "desk-client-1.0-1.noarch\n".into(),
                stderr: String::new(),
                status: 0,
                summary: None,
            },
        };
        if field != "entry-command" {
            assert!(!output.valid(), "{field}");
            assert!(
                serde_json::from_value::<terminal::Outcome>(serde_json::to_value(&output).unwrap())
                    .is_err()
            );
        }
        t.entries.push(terminal::Entry {
            command: command.into(),
            output,
        });
        assert!(!t.valid(), "direct {field} should reject NUL");
        assert!(
            serde_json::from_value::<terminal::Session>(serde_json::to_value(&t).unwrap()).is_err()
        );
        let mut a = baseline.clone();
        a.workspace.as_mut().unwrap().rpm_session = Some(t);
        let mut p = Progress {
            schema_version: 1,
            clock: 1,
            ..Default::default()
        };
        p.attempts.insert(s.id.clone(), a);
        let caller = serde_json::to_value(&p).unwrap();
        assert!(progress::save(&target, &p).is_err(), "{field}");
        assert_eq!(std::fs::read(&target).unwrap(), prior);
        assert!(serde_json::to_value(&p).unwrap() == caller);
        assert!(
            progress::decode(&serde_json::to_vec(&p).unwrap(), std::slice::from_ref(&s)).is_err()
        );
    }
    let valid = terminal::Outcome::Utility {
        stdout: "desk-client-1.0-1.noarch\n".into(),
        stderr: String::new(),
        status: 0,
        summary: Some("Simulation applied".into()),
    };
    assert!(valid.valid());
    assert_eq!(
        serde_json::from_value::<terminal::Outcome>(serde_json::to_value(&valid).unwrap()).unwrap(),
        valid
    );
}
#[test]
fn preservation_uses_initial_runtime_including_modified_config_and_missing_ordinary_file() {
    let s = scenario();
    let mut def = d(&s).clone();
    let mut catalog = def.initial.catalog().clone();
    let mut config = catalog.archives["neighbor-a"].files["/usr/share/neighbor/note"].clone();
    config.kind = core::FileKind::ConfigNoreplace;
    config.file.contents = "packaged default\n".into();
    catalog
        .archives
        .get_mut("neighbor-a")
        .unwrap()
        .files
        .insert("/etc/neighbor.conf".into(), config.clone());
    catalog.allowed_paths.push("/etc/neighbor.conf".into());
    let mut runtime = def.initial.runtime().clone();
    let mut local = config.file.clone();
    local.contents = "locally tuned\n".into();
    runtime
        .files
        .insert("/etc/neighbor.conf".into(), local.clone());
    runtime.files.remove("/usr/share/neighbor/note");
    def.initial = core::State::new(catalog, runtime).unwrap();
    def.validate().unwrap();
    assert!(
        def.checks(&def.initial)[4].0,
        "Keep actual initial configuration and missing file"
    );
    let mut current = def.initial.clone();
    for command in [&s.solution[0], &s.solution[1]] {
        assert!(matches!(
            terminal::execute(&def, &mut current, command),
            terminal::Outcome::Utility { status: 0, .. }
        ));
    }
    assert!(def.checks(&current).iter().all(|r| r.0));
    assert_eq!(current.runtime().files["/etc/neighbor.conf"], local);
    assert!(
        !current
            .runtime()
            .files
            .contains_key("/usr/share/neighbor/note")
    );
    for mutation in ["edit-config", "restore-missing", "remove-config"] {
        let mut runtime = current.runtime().clone();
        match mutation {
            "edit-config" => {
                runtime
                    .files
                    .get_mut("/etc/neighbor.conf")
                    .unwrap()
                    .contents = "changed again\n".into()
            }
            "restore-missing" => {
                runtime.files.insert(
                    "/usr/share/neighbor/note".into(),
                    current.catalog().archives["neighbor-a"].files["/usr/share/neighbor/note"]
                        .file
                        .clone(),
                );
            }
            _ => {
                runtime.files.remove("/etc/neighbor.conf");
            }
        }
        let changed = core::State::new(current.catalog().clone(), runtime).unwrap();
        assert!(
            !def.checks(&changed)[4].0,
            "{mutation} changes preserved runtime"
        );
    }
    let mut runtime = def.initial.runtime().clone();
    runtime
        .installed
        .retain(|x| x.identity.name != "quiet-neighbor");
    let mut absent = def.clone();
    absent.initial = core::State::new(def.initial.catalog().clone(), runtime).unwrap();
    assert!(
        absent.validate().is_err(),
        "Initial installed identity is still required"
    );
}
