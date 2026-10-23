//! Pure documented virtual-manager checks; never calls systemctl or host programs.
use dlms_learning_lab::unit_manager::{
    self as manager, Action, ErrorKind, Inventory, JobResult, Need, Program, Runtime, State,
};
use std::collections::{BTreeMap, BTreeSet};

fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|s| (*s).into()).collect()
}
fn task(description: &str, body: &str, remain: bool) -> String {
    format!(
        "[Unit]\nDescription={description}\n{body}[Service]\nType=oneshot\nExecStart=/sim/bin/prepare\nRemainAfterExit={}\n[Install]\nWantedBy=practice.target\n",
        if remain { "yes" } else { "no" }
    )
}
fn daemon(body: &str) -> String {
    format!(
        "[Unit]\nDescription=Virtual application\n{body}[Service]\nType=simple\nExecStart=/sim/bin/application\n[Install]\nWantedBy=practice.target\n"
    )
}
fn target(body: &str) -> String {
    format!("[Unit]\nDescription=Practice synchronization target\n{body}")
}
fn fixture() -> State {
    let inventory = Inventory {
        schema_version: 1,
        fixture: "unit-manager-tests".into(),
        units: names(&[
            "prepare.service",
            "application.service",
            "check.service",
            "practice.target",
            "maintenance.target",
        ]),
        config_paths: names(&[
            "/etc/lab/prepare.conf",
            "/etc/lab/application.conf",
            "/etc/lab/check.conf",
        ]),
        artifact_ids: names(&["prepared", "checked"]),
        programs: BTreeMap::from([
            (
                "/sim/bin/prepare".into(),
                Program::ValueTask {
                    config: "/etc/lab/prepare.conf".into(),
                    key: "VALUE".into(),
                    artifact: "prepared".into(),
                    prerequisite: None,
                },
            ),
            (
                "/sim/bin/check".into(),
                Program::ValueTask {
                    config: "/etc/lab/check.conf".into(),
                    key: "VALUE".into(),
                    artifact: "checked".into(),
                    prerequisite: Some(Need {
                        artifact: "prepared".into(),
                        value: "ready".into(),
                    }),
                },
            ),
            (
                "/sim/bin/application".into(),
                Program::PortDaemon {
                    config: "/etc/lab/application.conf".into(),
                    key: "PORT".into(),
                    prerequisite: Some(Need {
                        artifact: "prepared".into(),
                        value: "ready".into(),
                    }),
                },
            ),
        ]),
    };
    let documents = BTreeMap::from([
        (
            "prepare.service".into(),
            task("Prepare a supplied value", "", true),
        ),
        (
            "application.service".into(),
            daemon("Requires=prepare.service\nAfter=prepare.service\n"),
        ),
        (
            "check.service".into(),
            task(
                "Check prerequisite",
                "Requires=prepare.service\nAfter=prepare.service\n",
                true,
            )
            .replace("ExecStart=/sim/bin/prepare", "ExecStart=/sim/bin/check"),
        ),
        (
            "practice.target".into(),
            target("Wants=application.service\n"),
        ),
        ("maintenance.target".into(), target("")),
    ]);
    let configuration = BTreeMap::from([
        ("/etc/lab/prepare.conf".into(), "VALUE=ready\n".into()),
        ("/etc/lab/application.conf".into(), "PORT=8080\n".into()),
        ("/etc/lab/check.conf".into(), "VALUE=checked\n".into()),
    ]);
    State::new(
        inventory,
        documents.clone(),
        configuration,
        documents,
        "practice.target".into(),
    )
    .unwrap()
}
fn apply(s: &mut State, a: Action) -> manager::Report {
    s.apply(&a).unwrap()
}
fn start(s: &mut State, id: &str) -> manager::Report {
    apply(s, Action::Start { unit: id.into() })
}
fn unit_text(s: &mut State, id: &str, text: String) {
    apply(
        s,
        Action::SaveUnit {
            unit: id.into(),
            text,
        },
    );
}
fn reload(s: &mut State) {
    apply(s, Action::DaemonReload {});
}
fn failed(report: &manager::Report, id: &str) -> bool {
    !report
        .jobs
        .iter()
        .find(|j| j.unit == id)
        .unwrap()
        .result
        .succeeded()
}

#[test]
fn saved_loaded_dispatched_and_application_inputs_are_separate_without_old_engine_changes() {
    let mut s = fixture();
    let started = start(&mut s, "application.service");
    assert_eq!(started.requested_succeeded(), Some(true));
    assert!(matches!(
        s.runtime()["application.service"],
        Runtime::Dispatched { .. }
    ));
    assert!(s.result("application.service").unwrap().is_none());
    apply(&mut s, Action::AdvancePrograms {});
    assert_eq!(
        s.result("application.service").unwrap().unwrap().port,
        Some(8080)
    );
    apply(
        &mut s,
        Action::SaveConfig {
            path: "/etc/lab/application.conf".into(),
            text: "PORT=9090\n".into(),
        },
    );
    assert!(s.execution_differs("application.service").unwrap());
    reload(&mut s);
    assert_eq!(
        s.result("application.service").unwrap().unwrap().port,
        Some(8080)
    );
    let again = start(&mut s, "application.service");
    assert!(matches!(
        again
            .jobs
            .iter()
            .find(|j| j.unit == "application.service")
            .unwrap()
            .result,
        JobResult::AlreadyActive
    ));
    apply(
        &mut s,
        Action::Stop {
            unit: "application.service".into(),
        },
    );
    start(&mut s, "application.service");
    apply(&mut s, Action::AdvancePrograms {});
    assert_eq!(
        s.result("application.service").unwrap().unwrap().port,
        Some(9090)
    );
    assert!(!s.execution_differs("application.service").unwrap());
}

#[test]
fn simple_start_done_does_not_claim_readiness_or_successful_program_initialization() {
    let mut s = fixture();
    apply(
        &mut s,
        Action::SaveConfig {
            path: "/etc/lab/application.conf".into(),
            text: "PORT=not-a-port\n".into(),
        },
    );
    let r = start(&mut s, "application.service");
    assert_eq!(r.requested_succeeded(), Some(true));
    assert!(matches!(
        s.runtime()["application.service"],
        Runtime::Dispatched { .. }
    ));
    let r = apply(&mut s, Action::AdvancePrograms {});
    assert!(failed(&r, "application.service"));
    assert!(matches!(
        s.runtime()["application.service"],
        Runtime::Failed { .. }
    ));
    assert!(s.result("application.service").unwrap().is_none());
    apply(
        &mut s,
        Action::SaveConfig {
            path: "/etc/lab/application.conf".into(),
            text: "PORT=8081\n".into(),
        },
    );
    start(&mut s, "application.service");
    apply(&mut s, Action::AdvancePrograms {});
    assert_eq!(
        s.result("application.service").unwrap().unwrap().port,
        Some(8081)
    );
}

#[test]
fn dependency_pullin_is_not_order_and_unrelated_lexical_names_do_not_supply_a_guarantee() {
    let mut s = fixture();
    unit_text(
        &mut s,
        "check.service",
        task("Consumer", "Requires=prepare.service\n", true)
            .replace("/sim/bin/prepare", "/sim/bin/check"),
    );
    reload(&mut s);
    assert!(
        !s.ordered_before("prepare.service", "check.service")
            .unwrap()
    );
    let unordered_initial = s.clone();
    let r = start(&mut s, "check.service");
    assert_eq!(
        r.layers,
        vec![vec![
            "check.service".to_owned(),
            "prepare.service".to_owned()
        ]]
    );
    assert!(failed(&r, "check.service"));
    assert!(!failed(&r, "prepare.service"));
    assert!(s.artifacts().contains_key("prepared") && !s.artifacts().contains_key("checked"));
    let renamed = serde_json::to_string(&unordered_initial)
        .unwrap()
        .replace("prepare.service", "a-producer.service")
        .replace("check.service", "z-consumer.service");
    let mut renamed = State::from_json(&renamed).unwrap();
    let r = start(&mut renamed, "z-consumer.service");
    assert!(failed(&r, "z-consumer.service"));
    assert!(!failed(&r, "a-producer.service"));
    assert_eq!(
        r.layers.len(),
        1,
        "Earlier lexical execution is not an ordering guarantee"
    );
    let mut inverse = fixture();
    let replacement = task("Producer ordered by Before", "Before=check.service\n", true);
    unit_text(&mut inverse, "prepare.service", replacement);
    unit_text(
        &mut inverse,
        "check.service",
        task("Consumer", "Requires=prepare.service\n", true)
            .replace("/sim/bin/prepare", "/sim/bin/check"),
    );
    reload(&mut inverse);
    assert!(
        inverse
            .ordered_before("prepare.service", "check.service")
            .unwrap()
    );
    let r = start(&mut inverse, "check.service");
    assert_eq!(r.requested_succeeded(), Some(true));
    assert!(inverse.artifacts().contains_key("checked"));
}

#[test]
fn unloaded_known_required_and_wanted_units_have_distinct_job_effects() {
    for (dependency, blocks) in [("Requires", true), ("Wants", false)] {
        let s = fixture();
        let mut documents = s.documents().clone();
        documents.insert(
            "application.service".into(),
            daemon(&format!(
                "{dependency}=prepare.service\nAfter=prepare.service\n"
            )),
        );
        let mut loaded = documents.clone();
        loaded.remove("prepare.service");
        let mut s = State::new(
            s.inventory().clone(),
            documents,
            s.configuration().clone(),
            loaded,
            "practice.target".into(),
        )
        .unwrap();
        let r = start(&mut s, "application.service");
        assert!(failed(&r, "prepare.service"));
        assert_eq!(r.requested_succeeded(), Some(!blocks));
    }
}

fn padded(mut text: String, maximum: usize) -> String {
    while text.len() < maximum {
        let remaining = maximum - text.len();
        if remaining == 1 {
            text.push('\n');
            break;
        }
        text.push('#');
        let count = (maximum - text.len() - 1).min(500);
        text.extend("\"\\".chars().cycle().take(count));
        text.push('\n');
    }
    text
}

#[test]
fn maximum_shape_and_worst_saved_escaping_fit_compact_and_pretty_state_limits() {
    let tasks = (0..8)
        .map(|i| format!("{}{}.service", "s".repeat(55), i))
        .collect::<Vec<_>>();
    let targets = (0..8)
        .map(|i| format!("{}{}.target", "t".repeat(56), i))
        .collect::<Vec<_>>();
    let paths = (0..16)
        .map(|i| format!("/etc/lab/config{i}.conf"))
        .collect::<Vec<_>>();
    let mut inventory = Inventory {
        schema_version: 1,
        fixture: "max-shape".into(),
        units: tasks.iter().chain(&targets).cloned().collect(),
        config_paths: paths.iter().cloned().collect(),
        programs: BTreeMap::new(),
        artifact_ids: (0..8).map(|i| format!("artifact{i}")).collect(),
    };
    let mut documents = BTreeMap::new();
    let mut configuration = BTreeMap::new();
    for path in &paths {
        configuration.insert(
            path.clone(),
            padded("VALUE=ready\n".into(), manager::MAX_CONFIG_BYTES),
        );
    }
    for (i, name) in tasks.iter().enumerate() {
        let exec = format!("/sim/bin/task{i}");
        inventory.programs.insert(
            exec.clone(),
            Program::ValueTask {
                config: paths[i].clone(),
                key: "VALUE".into(),
                artifact: format!("artifact{i}"),
                prerequisite: None,
            },
        );
        documents.insert(name.clone(),padded(format!("[Unit]\nDescription={}\n[Service]\nType=oneshot\nRemainAfterExit=yes\nExecStart={exec}\n", "d".repeat(160)),manager::MAX_UNIT_BYTES));
    }
    for name in &targets {
        documents.insert(name.clone(), padded(target(""), manager::MAX_UNIT_BYTES));
    }
    let mut state = State::new(
        inventory,
        documents.clone(),
        configuration,
        documents,
        targets[0].clone(),
    )
    .unwrap();
    for id in tasks.iter().chain(&targets) {
        assert_eq!(start(&mut state, id).requested_succeeded(), Some(true));
    }
    assert_eq!(state.runtime().len(), 16);
    assert_eq!(state.artifacts().len(), 8);
    let mut raw = serde_json::to_value(&state).unwrap();
    for value in raw["documents"].as_object_mut().unwrap().values_mut() {
        *value = serde_json::Value::String("\0".repeat(manager::MAX_UNIT_BYTES));
    }
    for value in raw["configuration"].as_object_mut().unwrap().values_mut() {
        *value = serde_json::Value::String("\0".repeat(manager::MAX_CONFIG_BYTES));
    }
    let state: State = serde_json::from_value(raw).unwrap();
    for text in [
        serde_json::to_string(&state).unwrap(),
        serde_json::to_string_pretty(&state).unwrap(),
    ] {
        assert!(
            text.len() <= manager::MAX_JSON_BYTES,
            "{} bytes",
            text.len()
        );
        assert_eq!(State::from_json(&text).unwrap(), state);
    }
}

#[test]
fn ordered_required_failure_blocks_but_wants_or_unordered_requires_does_not() {
    for (dependency, order, blocked) in [
        ("Requires", "After=prepare.service\n", true),
        ("Wants", "After=prepare.service\n", false),
        ("Requires", "", false),
    ] {
        let mut s = fixture();
        apply(
            &mut s,
            Action::SaveConfig {
                path: "/etc/lab/prepare.conf".into(),
                text: "WRONG=ready\n".into(),
            },
        );
        unit_text(
            &mut s,
            "application.service",
            daemon(&format!("{dependency}=prepare.service\n{order}")),
        );
        reload(&mut s);
        let r = start(&mut s, "application.service");
        assert!(failed(&r, "prepare.service"));
        assert_eq!(r.requested_succeeded(), Some(!blocked));
        if blocked {
            assert!(matches!(
                r.jobs
                    .iter()
                    .find(|j| j.unit == "application.service")
                    .unwrap()
                    .result,
                JobResult::DependencyFailed { .. }
            ));
        } else {
            assert!(matches!(
                s.runtime()["application.service"],
                Runtime::Dispatched { .. }
            ));
        }
    }
}

#[test]
fn inactive_intermediary_cannot_order_or_propagate_failed_requires_in_a_start_transaction() {
    for include_intermediary in [false, true] {
        let inventory = Inventory {
            schema_version: 1,
            fixture: "transaction-ordering".into(),
            units: names(&[
                "a.service",
                "b.service",
                "c.service",
                "d.service",
                "e.service",
                "x.service",
                "practice.target",
            ]),
            config_paths: names(&["/etc/lab/prepare.conf", "/etc/lab/application.conf"]),
            artifact_ids: names(&["prepared"]),
            programs: BTreeMap::from([
                (
                    "/sim/bin/prepare".into(),
                    Program::ValueTask {
                        config: "/etc/lab/prepare.conf".into(),
                        key: "VALUE".into(),
                        artifact: "prepared".into(),
                        prerequisite: None,
                    },
                ),
                (
                    "/sim/bin/application".into(),
                    Program::PortDaemon {
                        config: "/etc/lab/application.conf".into(),
                        key: "PORT".into(),
                        prerequisite: None,
                    },
                ),
            ]),
        };
        let documents = BTreeMap::from([
            (
                "a.service".into(),
                daemon(&format!(
                    "Requires=b.service\nWants=d.service{}\nAfter=d.service\n",
                    if include_intermediary {
                        " x.service"
                    } else {
                        ""
                    },
                )),
            ),
            (
                "b.service".into(),
                task(
                    "Failing prerequisite",
                    "Wants=c.service\nAfter=c.service\n",
                    true,
                ),
            ),
            ("c.service".into(), daemon("")),
            (
                "d.service".into(),
                daemon("Wants=e.service\nAfter=e.service\n"),
            ),
            ("e.service".into(), daemon("")),
            (
                "x.service".into(),
                daemon("After=b.service\nBefore=a.service\n"),
            ),
            ("practice.target".into(), target("")),
        ]);
        let mut s = State::new(
            inventory,
            documents.clone(),
            BTreeMap::from([
                ("/etc/lab/prepare.conf".into(), "WRONG=ready\n".into()),
                ("/etc/lab/application.conf".into(), "PORT=8080\n".into()),
            ]),
            documents,
            "practice.target".into(),
        )
        .unwrap();
        // The full declaration has B -> X -> A, regardless of whether X starts.
        assert!(s.ordered_before("b.service", "a.service").unwrap());
        let r = start(&mut s, "a.service");
        assert!(failed(&r, "b.service"));
        assert_eq!(r.requested_succeeded(), Some(!include_intermediary));
        assert_eq!(
            r.jobs.iter().any(|j| j.unit == "x.service"),
            include_intermediary
        );
        if include_intermediary {
            assert!(matches!(
                r.jobs
                    .iter()
                    .find(|j| j.unit == "a.service")
                    .unwrap()
                    .result,
                JobResult::DependencyFailed { .. }
            ));
        } else {
            assert_eq!(
                r.layers,
                vec![
                    vec!["c.service".to_owned(), "e.service".to_owned()],
                    vec!["b.service".to_owned(), "d.service".to_owned()],
                    vec!["a.service".to_owned()],
                ]
            );
            assert!(matches!(
                s.runtime()["a.service"],
                Runtime::Dispatched { .. }
            ));
            assert!(!s.runtime().contains_key("x.service"));
        }
    }
}

#[test]
fn after_alone_does_not_pull_in_and_transitive_after_is_derived_from_edges() {
    let mut s = fixture();
    unit_text(
        &mut s,
        "application.service",
        daemon("After=prepare.service\n"),
    );
    reload(&mut s);
    let r = start(&mut s, "application.service");
    assert_eq!(r.jobs.len(), 1);
    assert!(!s.runtime().contains_key("prepare.service"));
    apply(&mut s, Action::AdvancePrograms {});
    assert!(matches!(
        s.runtime()["application.service"],
        Runtime::Failed { .. }
    ));
    let mut s = fixture();
    unit_text(
        &mut s,
        "application.service",
        daemon("Requires=prepare.service check.service\nAfter=check.service\n"),
    );
    reload(&mut s);
    assert!(
        s.ordered_before("prepare.service", "application.service")
            .unwrap()
    );
    assert!(
        start(&mut s, "application.service")
            .requested_succeeded()
            .unwrap()
    );
}

#[test]
fn successful_oneshot_without_remain_is_inactive_but_its_required_job_succeeded() {
    let mut s = fixture();
    unit_text(
        &mut s,
        "prepare.service",
        task("One-time preparation", "", false),
    );
    reload(&mut s);
    let r = start(&mut s, "application.service");
    assert_eq!(r.requested_succeeded(), Some(true));
    assert!(matches!(
        s.runtime()["prepare.service"],
        Runtime::Inactive { .. }
    ));
    assert!(s.artifacts().contains_key("prepared"));
    apply(&mut s, Action::AdvancePrograms {});
    assert!(matches!(
        s.runtime()["application.service"],
        Runtime::Running { .. }
    ));
}

#[test]
fn repeated_list_append_empty_reset_and_strict_exec_syntax_are_real_parser_behavior() {
    let s = fixture();
    let text = daemon(
        "Requires=prepare.service\nRequires=check.service prepare.service\nAfter=prepare.service\nAfter=\nBefore=check.service\n",
    );
    let d = manager::parse_unit(s.inventory(), "application.service", &text).unwrap();
    assert_eq!(d.requires, names(&["prepare.service", "check.service"]));
    assert!(d.after.is_empty());
    for bad in [
        daemon("").replace("/sim/bin/application", "/bin/sh -c anything"),
        daemon("").replace("/sim/bin/application", "/usr/sbin/httpd"),
        daemon("").replace("Type=simple", "Type=notify"),
        daemon("").replace("[Service]", "[Service]\nEnvironment=HOME=/tmp"),
        daemon("").replace("Type=simple", "Type=simple\nType=simple"),
        daemon("").replace("[Install]", "[Unit]\n[Install]"),
    ] {
        assert_eq!(
            manager::parse_unit(s.inventory(), "application.service", &bad)
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported
        );
    }
    assert!(
        manager::parse_unit(
            s.inventory(),
            "maintenance.target",
            "[Target]\nDescription=Unsupported\n"
        )
        .is_err()
    );
    assert!(
        manager::parse_unit(
            s.inventory(),
            "application.service",
            &daemon("").replace('\n', "\r\n")
        )
        .is_ok()
    );
}

#[test]
fn ordering_cycles_fail_atomically_but_plain_pullin_cycles_terminate() {
    let mut s = fixture();
    unit_text(
        &mut s,
        "prepare.service",
        task("Producer", "After=application.service\n", true),
    );
    let before = s.clone();
    let e = s.apply(&Action::DaemonReload {}).unwrap_err();
    assert_eq!(e.kind, ErrorKind::Ordering);
    assert!(e.message.contains("application.service") && e.message.contains("prepare.service"));
    assert_eq!(s, before);
    let mut s = fixture();
    unit_text(
        &mut s,
        "prepare.service",
        task("Producer", "Wants=application.service\n", true),
    );
    unit_text(
        &mut s,
        "application.service",
        daemon("Requires=prepare.service\n"),
    );
    reload(&mut s);
    assert!(start(&mut s, "application.service").jobs.len() <= manager::MAX_UNITS);
}

#[test]
fn bad_saved_unit_is_preserved_and_reload_failure_cannot_replace_loaded_or_running_state() {
    let mut s = fixture();
    start(&mut s, "application.service");
    apply(&mut s, Action::AdvancePrograms {});
    let before = s.loaded_sources().clone();
    unit_text(
        &mut s,
        "application.service",
        "[Service]\nExecStart=/bin/sh malicious\n".into(),
    );
    assert!(s.needs_daemon_reload("application.service").unwrap());
    let exact = s.clone();
    assert!(s.apply(&Action::DaemonReload {}).is_err());
    assert_eq!(s, exact);
    assert_eq!(s.loaded_sources(), &before);
    let restored = State::from_json(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(s, restored);
    assert_eq!(
        s.result("application.service").unwrap().unwrap().port,
        Some(8080)
    );
}

#[test]
fn enable_disable_default_and_fresh_session_do_not_invent_start_stop_or_isolate() {
    let mut s = fixture();
    let old = s.documents()["application.service"].clone();
    unit_text(
        &mut s,
        "application.service",
        old.replace("WantedBy=practice.target", "WantedBy=maintenance.target"),
    );
    apply(
        &mut s,
        Action::Enable {
            unit: "application.service".into(),
        },
    );
    assert!(!s.needs_daemon_reload("application.service").unwrap());
    assert!(s.runtime().is_empty());
    assert!(s.boot_links()["maintenance.target"].contains("application.service"));
    apply(
        &mut s,
        Action::SetDefault {
            target: "maintenance.target".into(),
        },
    );
    assert!(s.runtime().is_empty());
    let (mut fresh, r) = s.fresh_session().unwrap();
    assert_eq!(r.requested_succeeded(), Some(true));
    assert!(s.runtime().is_empty());
    assert!(fresh.runtime().contains_key("application.service"));
    apply(&mut fresh, Action::AdvancePrograms {});
    apply(
        &mut fresh,
        Action::Disable {
            unit: "application.service".into(),
        },
    );
    assert!(fresh.boot_links().is_empty());
    assert!(matches!(
        fresh.runtime()["application.service"],
        Runtime::Running { .. }
    ));
    let old = fixture().documents()["application.service"].clone();
    unit_text(&mut s, "application.service", old);
    apply(
        &mut s,
        Action::Enable {
            unit: "application.service".into(),
        },
    );
    assert!(
        s.boot_links()["practice.target"].contains("application.service")
            && s.boot_links()["maintenance.target"].contains("application.service")
    );
    apply(
        &mut s,
        Action::Disable {
            unit: "application.service".into(),
        },
    );
    apply(
        &mut s,
        Action::Enable {
            unit: "application.service".into(),
        },
    );
    assert_eq!(
        s.boot_links().keys().cloned().collect::<Vec<_>>(),
        vec!["practice.target".to_owned()]
    );
}

#[test]
fn stopping_a_required_unit_cascades_in_reverse_order_but_wants_is_independent() {
    let mut s = fixture();
    start(&mut s, "practice.target");
    apply(&mut s, Action::AdvancePrograms {});
    let r = apply(
        &mut s,
        Action::Stop {
            unit: "prepare.service".into(),
        },
    );
    assert!(r.jobs.iter().any(|j| j.unit == "application.service"));
    assert_eq!(r.layers[0], vec!["application.service".to_owned()]);
    assert!(matches!(
        s.runtime()["practice.target"],
        Runtime::TargetActive { .. }
    ));
    assert!(matches!(
        s.runtime()["application.service"],
        Runtime::Inactive { .. }
    ));
    assert!(s.artifacts().contains_key("prepared"));
    let mut s = fixture();
    unit_text(
        &mut s,
        "application.service",
        daemon("Wants=prepare.service\nAfter=prepare.service\n"),
    );
    reload(&mut s);
    start(&mut s, "application.service");
    apply(&mut s, Action::AdvancePrograms {});
    apply(
        &mut s,
        Action::Stop {
            unit: "prepare.service".into(),
        },
    );
    assert!(matches!(
        s.runtime()["application.service"],
        Runtime::Running { .. }
    ));
}

#[test]
fn strict_json_context_and_runtime_snapshots_reject_foreign_or_forged_state() {
    let mut s = fixture();
    start(&mut s, "application.service");
    apply(&mut s, Action::AdvancePrograms {});
    let raw = serde_json::to_value(&s).unwrap();
    for attack in [
        "schema",
        "source",
        "snapshot",
        "artifact",
        "impossible-failed-input",
        "foreign-key",
    ] {
        let mut v = raw.clone();
        match attack {
            "schema" => v["schema_version"] = 2.into(),
            "source" => {
                v["loaded"]["application.service"] = "[Service]\nExecStart=/bin/sh\n".into()
            }
            "snapshot" => {
                v["runtime"]["application.service"]["snapshot"]["configuration"] =
                    "PORT=wrong\n".into()
            }
            "artifact" => v["artifacts"]["prepared"]["value"] = "forged".into(),
            "impossible-failed-input" => {
                v["runtime"]["prepare.service"]["kind"] = "Failed".into();
                v["runtime"]["prepare.service"]["snapshot"]["artifact_input"] = "invented".into();
            }
            _ => v["documents"]["foreign.service"] = "[Unit]\nDescription=Foreign\n".into(),
        }
        assert!(
            State::from_json(&serde_json::to_string(&v).unwrap()).is_err(),
            "{attack}"
        );
    }
    let mut v = raw.clone();
    v["inventory"]["fixture"] = "other-fixture".into();
    let foreign = State::from_json(&serde_json::to_string(&v).unwrap()).unwrap();
    assert!(!foreign.same_context(&s));
    let compact = serde_json::to_string(&s).unwrap();
    let duplicate = compact.replacen(
        "\"documents\":{",
        "\"documents\":{\"application.service\":\"duplicate\",",
        1,
    );
    assert!(State::from_json(&duplicate).is_err());
    assert!(State::from_json(&" ".repeat(manager::MAX_JSON_BYTES + 1)).is_err());
}

#[test]
fn direct_oversize_mutations_and_unknown_paths_are_rejected_before_any_publication() {
    let mut s = fixture();
    for action in [
        Action::SaveUnit {
            unit: "application.service".into(),
            text: "x".repeat(manager::MAX_UNIT_BYTES + 1),
        },
        Action::SaveConfig {
            path: "/etc/lab/application.conf".into(),
            text: "x".repeat(manager::MAX_CONFIG_BYTES + 1),
        },
        Action::SaveConfig {
            path: "/etc/../personal".into(),
            text: "anything".into(),
        },
        Action::Start {
            unit: "unknown.service".into(),
        },
    ] {
        let before = s.clone();
        assert!(s.apply(&action).is_err());
        assert_eq!(s, before);
    }
}
