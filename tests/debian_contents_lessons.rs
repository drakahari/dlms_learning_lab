//! Authored Contents investigation; no host package command executes.
use dlms_learning_lab::{
    course, debian_packages as core, debian_packages_terminal as term,
    debian_packages_workspace as work,
    engine::{self, Goal, State},
    pack,
};
const ID: &str = "lpic1-package-file-provider";
fn lesson() -> pack::Scenario {
    pack::load_catalog(&[
        include_str!("../scenarios/debian-packages-v1.json"),
        include_str!("../scenarios/debian-contents-lessons-v1.json"),
    ])
    .unwrap()
    .into_iter()
    .find(|s| s.id == ID)
    .unwrap()
}
fn definition(s: &pack::Scenario) -> &work::Definition {
    let Goal::DebianPackages { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn simulation(s: &State) -> &core::State {
    let State::DebianPackages { simulation } = s else {
        panic!()
    };
    simulation
}
fn run(s: &pack::Scenario, state: &mut State, c: &str) {
    engine::execute_for_goal(state, &s.goal, c).unwrap_or_else(|e| panic!("{c}: {e}"));
}
#[test]
fn preserve252_labs71_units_exactly_and_integrate_one_distinct_provider_task() {
    let prior = pack::accepted252().unwrap();
    let now = pack::accepted253().unwrap();
    assert_eq!((prior.len(), now.len()), (252, 253));
    assert_eq!(
        serde_json::to_value(&prior).unwrap(),
        serde_json::to_value(&now[..252]).unwrap()
    );
    let units = course::accepted253(&now).unwrap();
    let before = course::accepted252(&prior).unwrap();
    assert_eq!((before.len(), units.len()), (71, 72));
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(&units[..71]).unwrap()
    );
    assert_eq!(units.last().unwrap().labs, [ID]);
    assert_eq!(
        units.last().unwrap().prerequisites,
        ["debian-package-administration"]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| now.iter().filter(|s| s.track == t).count()),
        [72, 72, 43, 66]
    );
    assert_eq!(
        now.last().unwrap().prerequisites,
        ["lpic1-package-handover"]
    );
}
#[test]
fn installed_ownership_repository_contents_and_installation_are_distinct() {
    let s = lesson();
    let mut state = s.initial.clone();
    assert!(!engine::solved(&state, &s.goal));
    let before = state.clone();
    let missing =
        engine::execute_for_goal(&mut state, &s.goal, "dpkg-query -S /usr/bin/ledger-check")
            .unwrap_err();
    assert!(missing.contains("no recorded owner") && missing.contains("exit status 1"));
    assert_eq!(state, before);
    run(&s, &mut state, "apt-file search ledger-check");
    assert_eq!(state, before);
    let mut model = simulation(&state).clone();
    let out = term::execute(definition(&s), &mut model, "apt-file search ledger-check").unwrap();
    assert_eq!(out.stdout.lines().count(), 2);
    assert!(!out.stdout.contains("records-tools"));
    run(&s, &mut state, "apt-get --error-on=any update");
    let metadata_only = state.clone();
    assert!(!engine::solved(&state, &s.goal));
    let mut model = simulation(&state).clone();
    let out = term::execute(
        definition(&s),
        &mut model,
        "apt-file -F search usr/bin/ledger-check",
    )
    .unwrap();
    assert_eq!(out.stdout, "records-tools: usr/bin/ledger-check\n");
    assert!(!out.changed);
    assert_eq!(out.status, 0);
    for c in [
        "apt-file list records-tools",
        "apt-file search ledger-check",
        "dpkg-query -W",
    ] {
        run(&s, &mut state, c);
    }
    assert_eq!(state, metadata_only);
    assert!(!engine::solved(&state, &s.goal));
    run(&s, &mut state, "apt-get -y install records-tools");
    assert!(engine::solved(&state, &s.goal));
}
#[test]
fn alternatives_wrong_providers_partial_work_and_invalidation_use_actual_files() {
    let s = lesson();
    for commands in [
        vec![
            "apt-get --error-on=any update",
            "apt-get -y install records-tools",
        ],
        vec![
            "apt-get --error-on=any update",
            "apt-get -y install records-lib=1.0-1 records-tools=1.0-1",
        ],
        vec![
            "apt-get --error-on=any update",
            "apt-get -y install records-private",
            "dpkg -r records-private",
            "apt-get -y install records-tools=1.0-1",
        ],
    ] {
        let mut state = s.initial.clone();
        for c in commands {
            run(&s, &mut state, c);
        }
        assert!(engine::solved(&state, &s.goal));
        run(&s, &mut state, "dpkg -r records-tools");
        assert!(!engine::solved(&state, &s.goal));
        run(&s, &mut state, "apt-get -y install records-tools");
        assert!(engine::solved(&state, &s.goal));
    }
    for wrong in ["records-docs", "records-private"] {
        let mut state = s.initial.clone();
        run(&s, &mut state, "apt-get --error-on=any update");
        run(&s, &mut state, &format!("apt-get -y install {wrong}"));
        assert!(!engine::solved(&state, &s.goal));
        let checks = definition(&s).checks(simulation(&state));
        assert!(checks.iter().any(|r| r.0) && checks.iter().any(|r| !r.0));
        run(&s, &mut state, "apt-get -y install records-tools");
        assert!(!engine::solved(&state, &s.goal));
        run(&s, &mut state, &format!("dpkg -r {wrong}"));
        assert!(engine::solved(&state, &s.goal));
    }
}
#[test]
fn separate_worked_commands_experiment_and_repair_are_reachable() {
    let s = lesson();
    let l = s.debian_packages_teaching.as_ref().unwrap();
    assert!(l.separate_from(simulation(&s.initial)));
    l.validate().unwrap();
    let mut m = l.definition.initial.clone();
    for action in &l.example_actions {
        term::execute(&l.definition, &mut m, action.command.as_ref().unwrap()).unwrap();
    }
    assert!(l.definition.checks(&m).iter().all(|r| r.0));
    for c in [
        "apt-file search exhibit-audit",
        "apt-file -F search /usr/bin/exhibit-audit",
        "apt-file list gallery-tools",
    ] {
        assert_eq!(term::execute(&l.definition, &mut m, c).unwrap().status, 0);
    }
    for a in &l.try_change.actions {
        work::apply(&l.definition, &mut m, a).unwrap();
    }
    assert_eq!(
        l.definition
            .checks(&m)
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>(),
        l.try_change.expected
    );
    for a in &l.repair {
        work::apply(&l.definition, &mut m, a).unwrap();
    }
    assert!(l.definition.checks(&m).iter().all(|r| r.0));
}
#[test]
fn unsupported_contents_input_is_honest_atomic_and_cannot_earn_completion() {
    let s = lesson();
    let mut m = simulation(&s.initial).clone();
    let before = m.clone();
    for c in [
        "apt-file -x search '.*'",
        "apt-file update",
        "apt-file -a arm64 search ledger-check",
        "apt-file search '../secret'",
        "apt-file search ledger-check | cat",
        "apt-file search $(touch /tmp/escape)",
    ] {
        let e = term::execute(definition(&s), &mut m, c).unwrap_err();
        assert_eq!(e.kind, term::DiagnosticKind::Unsupported);
        assert_eq!(m, before);
    }
}

#[test]
fn empty_contents_cache_status_and_retry_roundtrip_without_forged_dpkg_statuses() {
    let s = lesson();
    let mut runtime = simulation(&s.initial).runtime().clone();
    runtime.cached = None;
    let mut m = core::State::new(simulation(&s.initial).catalog().clone(), runtime).unwrap();
    let before = m.clone();
    let mut t = term::Session {
        command: "apt-file search ledger-check".into(),
        ..Default::default()
    };
    assert!(!t.submit(definition(&s), &mut m).unwrap());
    assert_eq!(t.entries.last().unwrap().status, 3);
    assert_eq!(m, before);
    assert_eq!(t.command, "apt-file search ledger-check");
    let raw = serde_json::to_vec(&t).unwrap();
    assert_eq!(term::Session::decode(&raw, &m).unwrap(), t);
    let mut forged = t.clone();
    forged.entries[0].command = "dpkg-query -W".into();
    assert!(!forged.valid());
    t.command = "apt-get --error-on=any update".into();
    assert!(t.submit(definition(&s), &mut m).unwrap());
    t.command = "apt-file -F search usr/bin/ledger-check".into();
    assert!(!t.submit(definition(&s), &mut m).unwrap());
    assert_eq!(t.entries.last().unwrap().status, 0);
    assert!(!definition(&s).checks(&m).iter().all(|r| r.0));
}
#[test]
fn separate_raw_drafts_credit_export_restore_and_selected_reset_survive_safe_writes() {
    use dlms_learning_lab::{
        backup, debian_packages_teaching as teaching,
        progress::{self, Attempt, Progress},
        workspace,
    };
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    let s = lesson();
    let mut all = pack::load(include_str!("../scenarios/debian-packages-v1.json")).unwrap();
    all.push(s.clone());
    let mut a = Attempt::new(&s, 7);
    workspace::prepare(&mut a);
    teaching::prepare(&mut a, s.debian_packages_teaching.as_ref().unwrap(), true).unwrap();
    for c in &s.solution {
        run(&s, &mut a.state, c);
    }
    let w = a.workspace.as_mut().unwrap();
    w.debian_packages_terminal = Some(term::Session {
        command: "apt-file search 'unfinished assignment".into(),
        ..Default::default()
    });
    let l = w.debian_packages_learning.as_mut().unwrap();
    l.initialize_terminal(s.debian_packages_teaching.as_ref().unwrap())
        .unwrap();
    l.terminal.as_mut().unwrap().command = "apt-file list 'unfinished example".into();
    l.view = dlms_learning_lab::script_teaching::View::Example;
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.favorites.insert(ID.into());
    p.attempts.insert(ID.into(), a);
    p.attempts
        .insert(all[0].id.clone(), Attempt::new(&all[0], 2));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let original = std::fs::read(&path).unwrap();
    let got = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&got).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(progress::current_solved(&got.attempts[ID], &s));
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, &all).unwrap();
    let inspected =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    let restored = backup::restore(&path, &inspected, &all).unwrap();
    assert_eq!(
        std::fs::read(restored.safety_copy.unwrap()).unwrap(),
        original
    );
    let other = serde_json::to_value(&p.attempts[&all[0].id]).unwrap();
    let mut reset = Attempt::new(&s, 8);
    workspace::prepare(&mut reset);
    p.attempts.insert(ID.into(), reset);
    assert!(!progress::current_solved(&p.attempts[ID], &s));
    assert!(p.favorites.contains(ID));
    assert_eq!(
        serde_json::to_value(&p.attempts[&all[0].id]).unwrap(),
        other
    );
    progress::save_with_catalog(&path, &p, &all).unwrap();
    assert_eq!(
        serde_json::to_value(progress::load(&path, &all).unwrap()).unwrap(),
        serde_json::to_value(p).unwrap()
    );
}

#[test]
fn positive_file_provider_grades_behavior_without_revealing_the_package_and_refuses_unowned_text() {
    let s = lesson();
    let d = definition(&s);
    let before = d.checks(simulation(&s.initial));
    assert!(
        before
            .iter()
            .all(|(_, text)| !text.contains("records-tools"))
    );
    assert!(before[0].1.contains("Search the complete path"));
    let m = simulation(&s.initial);
    let mut runtime = m.runtime().clone();
    runtime
        .files
        .insert("/usr/bin/ledger-check".into(), "tool\n".into());
    let unowned = core::State::new(m.catalog().clone(), runtime).unwrap();
    assert!(
        !d.checks(&unowned)[0].0,
        "unowned copied bytes are not an installed provider"
    );
    let mut solved = s.initial.clone();
    for c in &s.solution {
        run(&s, &mut solved, c);
    }
    let m = simulation(&solved);
    assert!(d.checks(m)[0].0);
    let mut runtime = m.runtime().clone();
    runtime
        .files
        .insert("/usr/bin/ledger-check".into(), "damaged\n".into());
    let damaged = core::State::new(m.catalog().clone(), runtime).unwrap();
    assert!(!d.checks(&damaged)[0].0);
    let mut invalid = d.clone();
    invalid.requirements = vec![work::Requirement::FilePreserved {
        path: "/work/records-notes.txt".into(),
    }];
    assert!(
        invalid.validate().is_err(),
        "negative/preservation-only goals still cannot pass"
    );
    invalid.requirements = vec![work::Requirement::InstalledFileProvider {
        path: "/unknown".into(),
    }];
    assert!(invalid.validate().is_err());
}

#[test]
fn file_provider_accepts_equivalent_supplied_packages_and_is_limited_to_ordinary_files() {
    let s = lesson();
    let base = simulation(&s.initial);
    let mut c = base.catalog().clone();
    let mut alternate = c.archives["records-tool"].clone();
    alternate.name = "records-alternate-tools".into();
    alternate
        .files
        .get_mut("/usr/bin/ledger-check")
        .unwrap()
        .contents = "equivalent alternate payload\n".into();
    c.archives.insert("records-alternate".into(), alternate);
    c.publications
        .get_mut("records-current")
        .unwrap()
        .candidates
        .insert("records-alternate-tools".into(), "records-alternate".into());
    let mut m = core::State::new(c, base.runtime().clone()).unwrap();
    let mut d = definition(&s).clone();
    d.initial = m.clone();
    d.validate().unwrap();
    term::execute(&d, &mut m, "apt-get --error-on=any update").unwrap();
    term::execute(&d, &mut m, "apt-get -y install records-alternate-tools").unwrap();
    assert!(
        d.checks(&m).iter().all(|r| r.0),
        "Grade installed file behavior, not one package identity"
    );
    assert!(!m.runtime().records.contains_key("records-tools"));
    // A conffile-only alternate cannot satisfy an ordinary-file provider goal,
    // even while another uninstalled ordinary provider makes the goal valid.
    let mut c = base.catalog().clone();
    let mut alternate = c.archives["records-tool"].clone();
    alternate.name = "records-config-only".into();
    alternate
        .files
        .get_mut("/usr/bin/ledger-check")
        .unwrap()
        .conffile = true;
    c.archives.insert("config-only".into(), alternate);
    c.publications
        .get_mut("records-current")
        .unwrap()
        .candidates
        .insert("records-config-only".into(), "config-only".into());
    let mut m = core::State::new(c, base.runtime().clone()).unwrap();
    let mut d = definition(&s).clone();
    d.initial = m.clone();
    d.validate().unwrap();
    term::execute(&d, &mut m, "apt-get --error-on=any update").unwrap();
    term::execute(&d, &mut m, "apt-get -y install records-config-only").unwrap();
    assert!(!d.checks(&m)[0].0);
    let mut c = base.catalog().clone();
    c.archives
        .get_mut("records-tool")
        .unwrap()
        .files
        .get_mut("/usr/bin/ledger-check")
        .unwrap()
        .conffile = true;
    let mut bad = definition(&s).clone();
    bad.initial = core::State::new(c, base.runtime().clone()).unwrap();
    assert!(
        bad.validate().is_err(),
        "Conffile choice is outside this positive ordinary-file predicate"
    );
}
