//! Prospective closed-command tests; expected data is authored, not host observations.
use dlms_learning_lab::{
    debian_packages::{self as core, Action, Architecture, Status, Version},
    debian_packages_terminal::{self as terminal, DiagnosticKind},
    debian_packages_workspace::{self as work, Definition, Requirement},
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt},
};
use std::collections::{BTreeMap, BTreeSet};
fn scenario() -> Scenario {
    static FIXTURE: std::sync::OnceLock<Scenario> = std::sync::OnceLock::new();
    FIXTURE
        .get_or_init(|| {
            pack::load(include_str!(
                "fixtures/debian-packages-workspace-preview.json"
            ))
            .unwrap()
            .remove(0)
        })
        .clone()
}
fn definition(s: &Scenario) -> &Definition {
    let Goal::DebianPackages { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn simulation(s: &Scenario) -> core::State {
    let State::DebianPackages { simulation } = &s.initial else {
        panic!()
    };
    simulation.clone()
}
fn changed(s: &core::State, f: impl FnOnce(&mut core::Catalog, &mut core::Runtime)) -> core::State {
    let (mut c, mut r) = (s.catalog().clone(), s.runtime().clone());
    f(&mut c, &mut r);
    core::State::new(c, r).unwrap()
}
fn execute(s: &Scenario, state: &mut core::State, input: &str) -> terminal::Output {
    terminal::execute(definition(s), state, input).unwrap()
}
fn reject(s: &Scenario, state: &mut core::State, input: &str, kind: DiagnosticKind, status: u8) {
    let old = serde_json::to_vec(state).unwrap();
    let e = terminal::execute(definition(s), state, input).unwrap_err();
    assert_eq!(e.kind, kind, "{input}: {e:?}");
    assert_eq!(e.status, status, "{input}");
    assert!(e.message.len() <= 512 && e.message.capacity() <= 512);
    assert_eq!(serde_json::to_vec(state).unwrap(), old, "{input}");
}
#[test]
fn real_formatted_queries_distinguish_installed_and_configuration_only_records() {
    let s = scenario();
    let mut state = simulation(&s);
    let out = execute(&s, &mut state, "dpkg-query -W museum-report");
    assert_eq!(out.stdout, "museum-report\t1.0-1\n");
    assert_eq!(out.status, 0);
    assert!(!out.changed);
    execute(&s, &mut state, "dpkg --remove museum-retired");
    let exact = serde_json::to_vec(&state).unwrap();
    for command in [
        "dpkg-query -W -f='${Package}\\t${Version}\\t${Architecture}\\t${db:Status-Status}\\n' museum-retired",
        "dpkg-query --showformat='${Package}\\t${Version}\\t${Architecture}\\t${db:Status-Status}\\n' --show 'museum-retired'",
    ] {
        let out = execute(&s, &mut state, command);
        assert_eq!(out.stdout, "museum-retired\t1.0-1\tamd64\tconfig-files\n");
        assert_eq!(out.status, 0);
        assert!(!out.changed);
    }
    let out = execute(&s, &mut state, "dpkg-query -W");
    assert_eq!(out.stdout.lines().count(), 4);
    assert_eq!(serde_json::to_vec(&state).unwrap(), exact);
    let out = execute(&s, &mut state, "dpkg-query -W museum-missing");
    assert_eq!(out.status, 1);
    assert!(out.stdout.is_empty());
    assert_eq!(serde_json::to_vec(&state).unwrap(), exact);
}
#[test]
fn exact_owner_query_does_not_claim_file_presence_or_model_a_full_search_language() {
    let s = scenario();
    let original = simulation(&s);
    let mut state = changed(&original, |_, r| {
        r.files.remove("/usr/bin/museum-report");
    });
    let exact = serde_json::to_vec(&state).unwrap();
    for command in [
        "dpkg-query -S /usr/bin/museum-report",
        "dpkg --search '/usr/bin/museum-report'",
    ] {
        let out = execute(&s, &mut state, command);
        assert_eq!(out.stdout, "museum-report: /usr/bin/museum-report\n");
        assert_eq!(out.status, 0);
        assert!(!out.changed);
    }
    assert!(!state.runtime().files.contains_key("/usr/bin/museum-report"));
    assert_eq!(serde_json::to_vec(&state).unwrap(), exact);
    assert_eq!(execute(&s, &mut state, "dpkg -S /work/note.txt").status, 1);
    for command in [
        "dpkg -S museum-report",
        "dpkg -S '/usr/*/museum-report'",
        "dpkg -S /usr/../bin/museum-report",
    ] {
        reject(&s, &mut state, command, DiagnosticKind::Unsupported, 2);
    }
    execute(&s, &mut state, "dpkg -r museum-retired");
    reject(
        &s,
        &mut state,
        "dpkg -S /etc/museum/retired.conf",
        DiagnosticKind::Unsupported,
        2,
    );
}
#[test]
fn apt_cache_query_is_immediate_cached_metadata_and_remains_read_only_offline() {
    let s = scenario();
    let mut state = simulation(&s);
    let exact = serde_json::to_vec(&state).unwrap();
    let out = execute(
        &s,
        &mut state,
        "apt-cache --important depends museum-report",
    );
    assert_eq!(out.stdout, "museum-report\n  Depends: museum-lib\n");
    assert_eq!(out.status, 0);
    assert_eq!(serde_json::to_vec(&state).unwrap(), exact);
    let offline = changed(&state, |c, _| c.repository_reachable = false);
    let mut def = definition(&s).clone();
    def.initial = offline.clone();
    let mut current = offline;
    let exact = serde_json::to_vec(&current).unwrap();
    assert_eq!(
        terminal::execute(
            &def,
            &mut current,
            "apt-cache depends --important museum-report"
        )
        .unwrap()
        .stdout,
        out.stdout
    );
    assert_eq!(serde_json::to_vec(&current).unwrap(), exact);
    let mut current = changed(&state, |_, r| {
        r.records.get_mut("museum-lib").unwrap().archive = "lib-2".into();
    });
    reject(
        &s,
        &mut current,
        "apt-cache --important depends museum-lib",
        DiagnosticKind::Unsupported,
        100,
    );
    reject(
        &s,
        &mut state,
        "apt-cache policy museum-report",
        DiagnosticKind::Unsupported,
        100,
    );
}
#[test]
fn strict_update_changes_only_cached_metadata_and_failure_retains_every_field() {
    let s = scenario();
    let mut state = simulation(&s);
    let records = state.runtime().records.clone();
    let files = state.runtime().files.clone();
    let local = state.runtime().local_archives.clone();
    let out = execute(&s, &mut state, "apt-get --error-on=any update");
    assert_eq!(out.status, 0);
    assert!(out.changed && out.stdout.starts_with("DLMS simulation:"));
    assert_eq!(state.runtime().cached.as_deref(), Some("current"));
    assert_eq!(state.runtime().records, records);
    assert_eq!(state.runtime().files, files);
    assert_eq!(state.runtime().local_archives, local);
    reject(
        &s,
        &mut state,
        "apt-get update",
        DiagnosticKind::Unsupported,
        100,
    );
    let offline = changed(&state, |c, _| c.repository_reachable = false);
    let mut def = definition(&s).clone();
    def.initial = offline.clone();
    let mut current = offline;
    let exact = serde_json::to_vec(&current).unwrap();
    let e = terminal::execute(&def, &mut current, "apt-get --error-on=any update").unwrap_err();
    assert_eq!(e.status, 100);
    assert_eq!(e.kind, DiagnosticKind::Model);
    assert_eq!(serde_json::to_vec(&current).unwrap(), exact);
}
#[test]
fn explicit_install_closure_and_safe_order_alternatives_have_real_current_results() {
    let s = scenario();
    let mut state = changed(&simulation(&s), |_, r| {
        r.files
            .insert("/etc/museum/report.conf".into(), "style=plain\n".into());
    });
    execute(&s, &mut state, "apt-get --error-on=any update");
    let start = state.clone();
    execute(&s, &mut state, "apt-get --yes install museum-report=2.0-1");
    assert_eq!(state.runtime().records["museum-report"].archive, "report-2");
    assert_eq!(state.runtime().records["museum-lib"].archive, "lib-2");
    assert!(state.dependency_issues().unwrap().is_empty());
    let mut alternate = start;
    execute(
        &s,
        &mut alternate,
        "apt-get install --assume-yes museum-lib=2.0-1",
    );
    execute(&s, &mut alternate, "apt-get -y install museum-report=2.0-1");
    assert_eq!(state, alternate);
    reject(
        &s,
        &mut state,
        "apt-get -y install museum-report=1.0-1",
        DiagnosticKind::Unsupported,
        100,
    );
    reject(
        &s,
        &mut state,
        "apt-get -y install museum-foreign",
        DiagnosticKind::Unsupported,
        100,
    );
}
#[test]
fn same_install_cannot_repair_payload_but_exact_current_reinstall_can_without_conffile_loss() {
    let s = scenario();
    let mut state = simulation(&s);
    let conf = state.runtime().files["/etc/museum/report.conf"].clone();
    let old = serde_json::to_vec(&state).unwrap();
    let out = execute(&s, &mut state, "apt-get -y install museum-report=1.0-1");
    assert!(!out.changed);
    assert_eq!(serde_json::to_vec(&state).unwrap(), old);
    assert!(!definition(&s).checks(&state).iter().all(|r| r.0));
    let out = execute(
        &s,
        &mut state,
        "apt-get --reinstall install -y museum-report=1.0-1",
    );
    assert!(out.changed);
    assert!(definition(&s).checks(&state).iter().all(|r| r.0));
    assert_eq!(state.runtime().files["/etc/museum/report.conf"], conf);
    let mut absent = changed(&state, |_, r| {
        r.files.remove("/etc/museum/report.conf");
    });
    execute(
        &s,
        &mut absent,
        "apt-get -y --reinstall install museum-report=1.0-1",
    );
    assert!(
        !absent
            .runtime()
            .files
            .contains_key("/etc/museum/report.conf")
    );
}
#[test]
fn reinstall_never_substitutes_an_old_artifact_for_a_new_candidate_or_skips_unrelated_broken_depends()
 {
    let s = scenario();
    let mut state = simulation(&s);
    execute(&s, &mut state, "apt-get --error-on=any update");
    reject(
        &s,
        &mut state,
        "apt-get -y --reinstall install museum-report=1.0-1",
        DiagnosticKind::Unsupported,
        100,
    );
    let mut state = changed(&simulation(&s), |_, r| {
        r.records.remove("museum-lib");
    });
    assert!(state.catalog().archives["retired-1"].depends.is_empty());
    assert!(
        state
            .plan(&Action::Reinstall {
                name: "museum-retired".into()
            })
            .is_ok(),
        "the pure exact reinstall alone would allow this; adapter must check the entire installed DB"
    );
    let exact = serde_json::to_vec(&state).unwrap();
    let e = terminal::execute(
        definition(&s),
        &mut state,
        "apt-get -y --reinstall install museum-retired=1.0-1",
    )
    .unwrap_err();
    assert_eq!(e.kind, DiagnosticKind::Unsupported);
    assert_eq!(e.status, 100);
    assert!(e.message.contains("ALL currently installed"));
    assert_eq!(serde_json::to_vec(&state).unwrap(), exact);
}
#[test]
fn apt_trailing_selectors_are_rejected_after_quote_removal_and_before_equal_version_selection() {
    let s = scenario();
    let mut state = simulation(&s);
    for arg in [
        "museum-retired+",
        "museum-retired-",
        "'museum-retired+'",
        "\"museum-retired-\"",
        "museum-retired+=1.0-1",
        "'museum-retired-=1.0-1'",
    ] {
        reject(
            &s,
            &mut state,
            &format!("apt-get -y install {arg}"),
            DiagnosticKind::Unsupported,
            100,
        );
    }
    for arg in ["museum-retired+", "'museum-retired-'"] {
        reject(
            &s,
            &mut state,
            &format!("apt-cache --important depends {arg}"),
            DiagnosticKind::Unsupported,
            100,
        );
    }
}
#[test]
fn exact_remove_purge_and_absent_noop_preserve_unrelated_records_and_recreated_files() {
    let s = scenario();
    let mut state = simulation(&s);
    reject(
        &s,
        &mut state,
        "dpkg --remove museum-lib",
        DiagnosticKind::Model,
        2,
    );
    let other = state.runtime().records["museum-viewer"].clone();
    let custom = state.runtime().files["/etc/museum/retired.conf"].clone();
    execute(&s, &mut state, "dpkg -r museum-retired");
    assert_eq!(
        state.runtime().records["museum-retired"].status,
        Status::ConfigFiles
    );
    assert!(
        !state
            .runtime()
            .files
            .contains_key("/usr/bin/museum-retired")
    );
    assert_eq!(state.runtime().files["/etc/museum/retired.conf"], custom);
    state = changed(&state, |_, r| {
        r.files.insert(
            "/usr/bin/museum-retired".into(),
            "recreated outside package ownership".into(),
        );
    });
    execute(&s, &mut state, "dpkg --purge museum-retired");
    assert!(!state.runtime().records.contains_key("museum-retired"));
    assert!(
        !state
            .runtime()
            .files
            .contains_key("/etc/museum/retired.conf")
    );
    assert_eq!(
        state.runtime().files["/usr/bin/museum-retired"],
        "recreated outside package ownership"
    );
    assert_eq!(state.runtime().records["museum-viewer"], other);
    let exact = serde_json::to_vec(&state).unwrap();
    let o = execute(&s, &mut state, "dpkg -P museum-retired");
    assert_eq!(o.status, 0);
    assert!(!o.changed);
    assert_eq!(serde_json::to_vec(&state).unwrap(), exact);
}
#[test]
fn version_comparison_has_boolean_exit_status_without_claiming_installation_or_output() {
    let s = scenario();
    let mut state = simulation(&s);
    let exact = serde_json::to_vec(&state).unwrap();
    for (command, status) in [
        ("dpkg --compare-versions '1.0~rc1' lt 1.0", 0),
        ("dpkg --compare-versions 1.0 eq 1.0-0", 0),
        ("dpkg --compare-versions 1.0 gt 2.0", 1),
        ("dpkg --compare-versions 1:1.0 ne 1.0", 0),
    ] {
        let o = execute(&s, &mut state, command);
        assert_eq!(o.status, status);
        assert!(o.stdout.is_empty() && o.stderr.is_empty());
        assert!(!o.changed);
    }
    assert_eq!(serde_json::to_vec(&state).unwrap(), exact);
    reject(
        &s,
        &mut state,
        "dpkg --compare-versions '' lt 1",
        DiagnosticKind::Unsupported,
        2,
    );
}
#[test]
fn unknown_options_shell_forms_and_unmodeled_utilities_are_atomic_not_canned_success() {
    let s = scenario();
    let mut state = simulation(&s);
    for command in [
        "sudo dpkg -r museum-retired",
        "/usr/bin/dpkg -r museum-retired",
        "dpkg -i report.deb",
        "dpkg --verify museum-report",
        "dpkg --audit",
        "dpkg --configure -a",
        "dpkg-query -W -f='${Status}' museum-report",
        "dpkg-query -W -W museum-report",
        "dpkg-query -W -f='${Package}' --showformat='${Version}' museum-report",
        "dpkg-query -W \"${Package}\"",
        "dpkg-query -W $(printf museum-report)",
        "dpkg-query -W museum-*",
        "dpkg -r museum-retired; dpkg -P museum-retired",
        "X=1 dpkg -r museum-retired",
        "dpkg -r 'museum-retired;anything'",
        "dpkg-query -W -f='${Package}\\q' museum-report",
    ] {
        let exact = serde_json::to_vec(&state).unwrap();
        let e = terminal::execute(definition(&s), &mut state, command).unwrap_err();
        assert_eq!(e.status, 2, "{command}");
        assert_eq!(serde_json::to_vec(&state).unwrap(), exact, "{command}");
    }
    for command in [
        "apt-get -y -y install museum-report",
        "apt-get -y --force-yes install museum-report",
        "apt-get -y remove museum-retired",
        "apt-get --reinstall install museum-report=1.0-1",
    ] {
        let exact = serde_json::to_vec(&state).unwrap();
        assert_eq!(
            terminal::execute(definition(&s), &mut state, command)
                .unwrap_err()
                .status,
            100
        );
        assert_eq!(serde_json::to_vec(&state).unwrap(), exact);
    }
}
#[test]
fn input_argument_and_format_limits_are_checked_before_mutation_with_bounded_diagnostics() {
    let s = scenario();
    let mut state = simulation(&s);
    reject(
        &s,
        &mut state,
        &" ".repeat(terminal::MAX_COMMAND_BYTES + 1),
        DiagnosticKind::Limit,
        2,
    );
    for command in [
        format!("apt-get -y install {}", ["museum-report"; 9].join(" ")),
        format!("dpkg-query -W -f='{}' museum-report", "a".repeat(257)),
        vec!["x"; 65].join(" "),
    ] {
        let exact = serde_json::to_vec(&state).unwrap();
        assert!(terminal::execute(definition(&s), &mut state, &command).is_err());
        assert_eq!(serde_json::to_vec(&state).unwrap(), exact);
    }
    for command in ["dpkg-query\0 -W", "dpkg-query -W\n", "dpkg-query\r-W"] {
        reject(&s, &mut state, command, DiagnosticKind::Unsupported, 2);
    }
}
#[test]
fn large_but_admitted_formatted_query_rejects_output_without_truncating_or_mutating() {
    let mut archives = BTreeMap::new();
    let mut candidates = BTreeMap::new();
    let mut records = BTreeMap::new();
    for n in 0..24 {
        let name = format!("pkg-{n:02}-{}", "a".repeat(57));
        assert_eq!(name.len(), 64);
        let id = format!("archive-{n}");
        archives.insert(
            id.clone(),
            core::Archive {
                name: name.clone(),
                version: Version::parse("1").unwrap(),
                architecture: Architecture::All,
                depends: vec![],
                files: BTreeMap::new(),
            },
        );
        candidates.insert(name.clone(), id.clone());
        records.insert(
            name,
            core::PackageRecord {
                archive: id,
                status: Status::Installed,
            },
        );
    }
    let mut state = core::State::new(
        core::Catalog {
            architecture: Architecture::Amd64,
            repository_reachable: true,
            published: "index".into(),
            archives,
            publications: BTreeMap::from([("index".into(), core::Index { candidates })]),
            extra_paths: BTreeSet::new(),
        },
        core::Runtime {
            cached: Some("index".into()),
            local_archives: BTreeSet::new(),
            records,
            files: BTreeMap::new(),
        },
    )
    .unwrap();
    assert!(work::admitted(&state));
    let name = state.runtime().records.keys().next().unwrap().clone();
    let d = Definition {
        initial: state.clone(),
        requirements: vec![Requirement::Installed {
            package: name,
            constraint: None,
        }],
    };
    assert!(d.validate().is_ok());
    let exact = serde_json::to_vec(&state).unwrap();
    let command = format!("dpkg-query -W -f='{}'", "${Package}".repeat(25));
    let e = terminal::execute(&d, &mut state, &command).unwrap_err();
    assert_eq!(e.kind, DiagnosticKind::Limit);
    assert_eq!(serde_json::to_vec(&state).unwrap(), exact);
}
#[test]
fn goal_aware_real_commands_fresh_grade_and_isolated_practice_keep_existing_context_boundaries() {
    let s = scenario();
    let mut a = Attempt::new(&s, 1);
    engine::execute_for_goal(
        &mut a.state,
        &s.goal,
        "apt-get -y --reinstall install museum-report=1.0-1",
    )
    .unwrap();
    assert!(progress::current_solved(&a, &s));
    let exact = serde_json::to_vec(&a).unwrap();
    engine::execute_for_goal(&mut a.state, &s.goal, "dpkg-query -W museum-report").unwrap();
    assert_eq!(serde_json::to_vec(&a).unwrap(), exact);
    assert!(progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, "dpkg -r museum-report").unwrap();
    assert!(!progress::current_solved(&a, &s));
    engine::execute_for_goal(
        &mut a.state,
        &s.goal,
        "apt-get -y install museum-report=1.0-1",
    )
    .unwrap();
    assert!(progress::current_solved(&a, &s));
    let l = s.debian_packages_teaching.as_ref().unwrap();
    let mut practice = l.definition.initial.clone();
    let before = serde_json::to_vec(&practice).unwrap();
    let e = terminal::execute(definition(&s), &mut practice, "dpkg-query -W").unwrap_err();
    assert_eq!(e.kind, DiagnosticKind::Context);
    assert_eq!(serde_json::to_vec(&practice).unwrap(), before);
    let mut assignment = simulation(&s);
    let old = serde_json::to_vec(&assignment).unwrap();
    assert!(terminal::execute(&l.definition, &mut assignment, "dpkg -P museum-retired").is_err());
    assert_eq!(serde_json::to_vec(&assignment).unwrap(), old);
    assert!(engine::execute(&mut a.state, "dpkg-query -W museum-report").is_err());
}

#[test]
fn exact_cached_version_ending_plus_installs_and_reinstalls_without_becoming_a_name_selector() {
    let s = scenario();
    let initial = changed(&simulation(&s), |c, _| {
        c.archives.get_mut("report-1").unwrap().version = Version::parse("1.0+").unwrap();
    });
    let mut d = definition(&s).clone();
    d.initial = initial.clone();
    for requirement in &mut d.requirements {
        if let Requirement::Installed {
            package,
            constraint,
        } = requirement
            && package.as_str() == "museum-report"
        {
            *constraint = Some(core::Constraint {
                relation: core::Relation::Eq,
                version: Version::parse("1.0+").unwrap(),
            });
        }
    }
    assert!(d.validate().is_ok());
    for operand in ["museum-report=1.0+", "'museum-report=1.0+'"] {
        let mut state = initial.clone();
        terminal::execute(&d, &mut state, "dpkg -r museum-report").unwrap();
        assert_eq!(
            state.runtime().records["museum-report"].status,
            Status::ConfigFiles
        );
        let installed =
            terminal::execute(&d, &mut state, &format!("apt-get -y install {operand}")).unwrap();
        assert_eq!(installed.status, 0);
        assert!(installed.changed);
        assert_eq!(
            state.catalog().archives[&state.runtime().records["museum-report"].archive]
                .version
                .as_str(),
            "1.0+"
        );
        assert!(d.checks(&state).iter().all(|row| row.0));
        state = changed(&state, |_, r| {
            r.files
                .insert("/usr/bin/museum-report".into(), "damaged again".into());
        });
        assert!(!d.checks(&state).iter().all(|row| row.0));
        let repaired = terminal::execute(
            &d,
            &mut state,
            &format!("apt-get -y --reinstall install {operand}"),
        )
        .unwrap();
        assert_eq!(repaired.status, 0);
        assert!(repaired.changed);
        assert!(d.checks(&state).iter().all(|row| row.0));
        assert_eq!(
            state.runtime().files["/etc/museum/report.conf"],
            "style=local\n"
        );
    }
}
