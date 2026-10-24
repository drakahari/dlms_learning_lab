//! Closed in-process terminal and retained-event tests. No host commands/services.
use dlms_learning_lab::{
    unit_manager::{self as core, Action, Runtime, State, journal::EventKind},
    unit_manager_terminal::{self as terminal, DiagnosticKind},
    unit_manager_workspace::{self as workspace, Definition, Draft, EditAction},
};
use serde::Deserialize;
use serde_json::{Value, json};
#[derive(Deserialize)]
struct Fixture {
    definition: Definition,
    solution: Vec<EditAction>,
}
fn fixture() -> Fixture {
    serde_json::from_str(include_str!("fixtures/unit-manager-workspace-preview.json")).unwrap()
}
const APP: &str = "work-application.service";
const OBS: &str = "work-observer.service";
fn action(state: &mut State, action: Action) {
    state.apply(&action).unwrap();
}
fn solved(f: &Fixture) -> State {
    let mut state = f.definition.initial.clone();
    for edit in &f.solution {
        workspace::apply(&mut state, &f.definition.initial, edit).unwrap();
    }
    state
}
fn command(f: &Fixture, state: &mut State, text: &str) -> terminal::Outcome {
    terminal::execute(&f.definition, state, text).unwrap()
}
fn value_with_journal(state: &State, last: u64) -> Value {
    let mut value = serde_json::to_value(state).unwrap();
    value["journal"] = json!({"version":1,"last":last,"dropped":last-1,"events":[
        {"sequence":last,"unit":OBS,"event":{"kind":"Stopped"}}
    ]});
    value
}
fn from_value(value: &Value) -> State {
    State::from_json(&serde_json::to_string(value).unwrap()).unwrap()
}

#[test]
fn literal_parser_accepts_closed_options_and_rejects_shell_unknown_and_incomplete_atomically() {
    let f = fixture();
    let mut state = f.definition.initial.clone();
    for text in [
        "systemctl --no-pager status 'work-observer.service'",
        "systemctl is-active -q work-observer.service",
        "systemctl --quiet is-enabled work-observer.service",
        "systemctl show --property=Description,ActiveState work-observer.service",
        "journalctl --output=cat --unit=work-observer.service --lines=2 --no-pager",
        "systemctl get-default",
        "help",
    ] {
        terminal::parse(text, state.inventory()).unwrap();
    }
    for text in [
        "systemctl restart work-observer.service",
        "systemctl start work-observer",
        "systemctl start work-observer.service --now",
        "systemctl --user status work-observer.service",
        "systemctl start work-observer.service; echo done",
        "systemctl cat /etc/passwd",
        "systemctl start $UNIT",
        "systemctl status work-*.service",
        "systemctl status work-observer.service | cat",
        "systemctl show work-observer.service",
        "systemctl show work-observer.service -p MainPID",
        "systemctl status work-observer.service -q",
        "journalctl -u work-observer.service",
        "journalctl -u work-observer.service -o short",
        "journalctl -u work-observer.service -o cat -n",
        "journalctl -u work-observer.service -o cat -n 0",
        "journalctl -u work-observer.service -o cat -n 257",
        "journalctl -u work-observer.service -o cat -f",
        "systemctl -p Description cat work-observer.service",
        "systemctl start",
        "systemctl get-default work.target",
        "systemctl set-default work-observer.service",
        "systemctl --no-pager --no-pager status work-observer.service",
    ] {
        let before = state.clone();
        assert!(
            terminal::execute(&f.definition, &mut state, text).is_err(),
            "{text}"
        );
        assert_eq!(state, before);
    }
    assert_eq!(
        terminal::parse(
            &"x".repeat(terminal::MAX_COMMAND_BYTES + 1),
            state.inventory()
        )
        .unwrap_err()
        .kind,
        DiagnosticKind::Limit
    );
}
#[test]
fn query_layers_keep_saved_text_loaded_definition_runtime_install_and_default_independent() {
    let f = fixture();
    let mut state = f.definition.initial.clone();
    let original = state.clone();
    assert_eq!(
        command(&f, &mut state, "systemctl get-default").stdout,
        "work-spare.target\n"
    );
    assert_eq!(
        command(&f, &mut state, "systemctl is-enabled work.target").stdout,
        "static\n"
    );
    assert_eq!(
        command(&f, &mut state, "systemctl is-enabled work.target").status,
        0
    );
    assert_eq!(
        command(&f, &mut state, "systemctl is-active work.target").status,
        3
    );
    assert_eq!(state, original); // static/zero is not future execution or current success
    let edited = state.documents()[OBS].replace("Unaffected observer", "Changed saved description");
    action(
        &mut state,
        Action::SaveUnit {
            unit: OBS.into(),
            text: edited,
        },
    );
    let before = state.clone();
    assert!(
        command(&f, &mut state, "systemctl cat work-observer.service")
            .stdout
            .contains("Changed saved description")
    );
    assert_eq!(
        command(
            &f,
            &mut state,
            "systemctl show work-observer.service -p Description"
        )
        .stdout,
        "Description=Unaffected observer\n"
    );
    assert!(
        command(&f, &mut state, "systemctl status work-observer.service")
            .stdout
            .contains("saved and loaded unit text differ")
    );
    assert_eq!(state, before);
    command(&f, &mut state, "systemctl daemon-reload");
    assert!(
        command(
            &f,
            &mut state,
            "systemctl show work-observer.service -p Description"
        )
        .stdout
        .contains("Changed saved description")
    );
    command(&f, &mut state, "systemctl enable work-observer.service");
    assert_eq!(
        command(&f, &mut state, "systemctl is-enabled work-observer.service").stdout,
        "enabled\n"
    );
    assert_eq!(
        command(&f, &mut state, "systemctl is-active work-observer.service").stdout,
        "inactive\n"
    );
    assert_eq!(
        command(&f, &mut state, "systemctl get-default").stdout,
        "work-spare.target\n"
    );
}
#[test]
fn simple_start_status_comes_from_job_then_initialization_failure_is_truthful() {
    let f = fixture();
    let mut state = f.definition.initial.clone();
    let result = command(&f, &mut state, "systemctl start work-application.service");
    assert_eq!(result.status, 0);
    assert!(result.changed); // simple dispatch succeeded
    assert!(matches!(
        state.runtime().get(APP),
        Some(Runtime::Failed { .. })
    ));
    let status = command(&f, &mut state, "systemctl status work-application.service");
    assert_eq!(status.status, 3);
    assert!(status.stdout.contains("Active: failed"));
    let journal = command(
        &f,
        &mut state,
        "journalctl -u work-application.service -o cat",
    )
    .stdout;
    assert!(journal.contains("initialization pending"));
    assert!(journal.contains("could not initialize"));
    assert!(!workspace::solved(
        &f.definition,
        &state,
        &Draft::new(&state).unwrap()
    ));
    assert!(
        state
            .journal()
            .unwrap()
            .events()
            .iter()
            .any(|e| matches!(e.kind(), EventKind::OneshotCompleted { remaining: true }))
    );
}
#[test]
fn failed_oneshot_is_a_failed_job_and_queries_never_advance_pending_programs() {
    let f = fixture();
    let mut state = f.definition.initial.clone();
    action(
        &mut state,
        Action::SaveConfig {
            path: "/etc/work/prepare.conf".into(),
            text: "INVALID\n".into(),
        },
    );
    assert_eq!(
        command(&f, &mut state, "systemctl start work-prepare.service").status,
        1
    );
    assert!(
        command(&f, &mut state, "journalctl -u work-prepare.service -o cat")
            .stdout
            .contains("start job")
    );
    action(&mut state, Action::Start { unit: OBS.into() });
    assert!(matches!(
        state.runtime().get(OBS),
        Some(Runtime::Dispatched { .. })
    ));
    let before = state.clone();
    for text in [
        "systemctl status work-observer.service",
        "systemctl is-active work-observer.service",
        "systemctl show work-observer.service -p ActiveState",
        "journalctl -u work-observer.service -o cat",
    ] {
        terminal::read_only(&state, text).unwrap();
    }
    assert_eq!(state, before);
    assert_eq!(
        terminal::read_only(&state, "systemctl start work-observer.service")
            .unwrap_err()
            .kind,
        DiagnosticKind::Context
    );
}
#[test]
fn journals_capture_committed_results_without_duplicate_start_or_inactive_stop_events() {
    let f = fixture();
    let mut state = f.definition.initial.clone();
    assert!(state.journal().is_none());
    command(&f, &mut state, "systemctl stop work-observer.service");
    assert!(state.journal().is_none());
    command(&f, &mut state, "systemctl start work-observer.service");
    assert_eq!(state.journal().unwrap().events().len(), 2);
    command(&f, &mut state, "systemctl start work-observer.service");
    assert_eq!(state.journal().unwrap().events().len(), 2);
    command(&f, &mut state, "systemctl stop work-observer.service");
    assert_eq!(state.journal().unwrap().events().len(), 3);
    let all = command(&f, &mut state, "journalctl -u work-observer.service -o cat");
    assert_eq!(all.stdout.lines().count(), 3);
    let tail = command(
        &f,
        &mut state,
        "journalctl -o cat -n 2 -u work-observer.service",
    );
    assert_eq!(
        tail.stdout,
        all.stdout
            .lines()
            .skip(1)
            .map(|line| format!("{line}\n"))
            .collect::<String>()
    );
    assert_eq!(
        command(&f, &mut state, "journalctl -u work-spare.target -o cat").stdout,
        ""
    );
    assert_eq!(
        state.journal().unwrap().events().last().unwrap().sequence(),
        3
    );
}
#[test]
fn retention_is_bounded_and_fresh_session_has_its_own_non_authoritative_window() {
    let f = fixture();
    let mut state = f.definition.initial.clone();
    for _ in 0..90 {
        command(&f, &mut state, "systemctl start work-observer.service");
        command(&f, &mut state, "systemctl stop work-observer.service");
    }
    let journal = state.journal().unwrap();
    assert_eq!(journal.events().len(), core::journal::MAX_EVENTS);
    assert_eq!(journal.dropped(), 14);
    assert_eq!(journal.events().first().unwrap().sequence(), 15);
    assert_eq!(
        State::from_json(&serde_json::to_string_pretty(&state).unwrap()).unwrap(),
        state
    );
    let before = state.clone();
    let (fresh, _) = state.fresh_session().unwrap();
    assert_eq!(state, before);
    assert!(
        fresh
            .journal()
            .unwrap()
            .events()
            .first()
            .unwrap()
            .sequence()
            == 1
    );
    assert_ne!(fresh.journal(), state.journal());
}
#[test]
fn malformed_or_foreign_journals_reject_at_decode_and_old_absent_field_roundtrips() {
    let f = fixture();
    let base = serde_json::to_value(&f.definition.initial).unwrap();
    assert!(base.get("journal").is_none());
    let good = value_with_journal(&f.definition.initial, 1);
    from_value(&good);
    for changed in [
        {
            let mut v = good.clone();
            v["journal"]["version"] = json!(2);
            v
        },
        {
            let mut v = good.clone();
            v["journal"]["last"] = json!(2);
            v
        },
        {
            let mut v = good.clone();
            v["journal"]["events"][0]["unit"] = json!("foreign.service");
            v
        },
        {
            let mut v = good.clone();
            v["journal"]["events"][0]["event"] = json!({"kind":"TargetActivated"});
            v
        },
        {
            let mut v = good.clone();
            v["journal"]["events"][0]["event"] =
                json!({"kind":"StartFailed","fault":"Program","detail":"é".repeat(257)});
            v
        },
        {
            let mut v = good.clone();
            v["journal"]["events"][0]["event"] =
                json!({"kind":"StartFailed","fault":"Program","detail":"bad\nline"});
            v
        },
        {
            let mut v = good.clone();
            v["journal"]["events"][0]["event"] =
                json!({"kind":"DependencyFailed","dependency":OBS});
            v
        },
        {
            let mut v = good.clone();
            v["journal"]["events"][0]["event"] =
                json!({"kind":"InitializationFailed","fault":"Unknown","detail":"failure"});
            v
        },
        {
            let mut v = good.clone();
            v["journal"]["events"][0]["event"] =
                json!({"kind":"DependencyFailed","dependency":"foreign.service"});
            v
        },
        {
            let mut v = good.clone();
            v["journal"]["extra"] = json!(true);
            v
        },
        {
            let mut v = good.clone();
            v["journal"]["events"] = json!(vec![good["journal"]["events"][0].clone(); 257]);
            v
        },
    ] {
        assert!(State::from_json(&serde_json::to_string(&changed).unwrap()).is_err());
    }
    let duplicate = serde_json::to_string(&good)
        .unwrap()
        .replace("\"version\":1", "\"version\":1,\"version\":1");
    assert!(State::from_json(&duplicate).is_err());
    assert_eq!(from_value(&base), f.definition.initial);
}
#[test]
fn overflow_at_start_or_later_program_event_keeps_exact_original_state() {
    let f = fixture();
    for last in [u64::MAX, u64::MAX - 1] {
        let mut state = from_value(&value_with_journal(&f.definition.initial, last));
        let before = state.clone();
        assert!(
            terminal::execute(
                &f.definition,
                &mut state,
                "systemctl start work-observer.service"
            )
            .is_err()
        );
        assert_eq!(state, before); // includes start event that was otherwise valid before later overflow
    }
    let mut state = from_value(&value_with_journal(&f.definition.initial, u64::MAX));
    let before = state.clone();
    assert!(state.apply(&Action::Start { unit: OBS.into() }).is_err());
    assert_eq!(state, before);
}
#[test]
fn journal_history_cannot_supply_credit_or_hide_broken_current_inputs() {
    let f = fixture();
    let mut state = solved(&f);
    assert!(workspace::solved(
        &f.definition,
        &state,
        &Draft::new(&state).unwrap()
    ));
    let history = state.journal().unwrap().clone();
    action(
        &mut state,
        Action::SaveConfig {
            path: "/etc/work/application.conf".into(),
            text: "PORT=wrong\n".into(),
        },
    );
    assert_eq!(state.journal(), Some(&history));
    assert!(!workspace::solved(
        &f.definition,
        &state,
        &Draft::new(&state).unwrap()
    ));
    let before = state.clone();
    let _ = workspace::checks(&f.definition, &state, &Draft::new(&state).unwrap());
    assert_eq!(state, before);
    assert!(
        command(
            &f,
            &mut state,
            "journalctl -u work-application.service -o cat"
        )
        .stdout
        .contains("initialized successfully")
    );
    assert!(!workspace::solved(
        &f.definition,
        &state,
        &Draft::new(&state).unwrap()
    ));
}
#[test]
fn bounded_output_is_not_successfully_truncated_and_raw_controls_are_display_only_escaped() {
    let f = fixture();
    let mut state = f.definition.initial.clone();
    action(
        &mut state,
        Action::SaveUnit {
            unit: OBS.into(),
            text: "\u{1b}[31m\n".into(),
        },
    );
    let shown = command(&f, &mut state, "systemctl cat work-observer.service");
    assert!(shown.display_escaped);
    assert!(shown.stdout.contains("\\u{1b}"));
    assert_eq!(state.documents()[OBS], "\u{1b}[31m\n");
    assert!(
        terminal::execute(
            &f.definition,
            &mut state,
            "systemctl is-enabled work-observer.service"
        )
        .is_err()
    );
    let mut value = value_with_journal(&f.definition.initial, 256);
    value["journal"]["dropped"] = json!(0);
    value["journal"]["events"]=json!((1..=256).map(|n|json!({"sequence":n,"unit":OBS,"event":{"kind":"StartFailed","fault":"Program","detail":"é".repeat(256)}})).collect::<Vec<_>>());
    let state = from_value(&value);
    let before = state.clone();
    assert_eq!(
        terminal::read_only(&state, "journalctl -u work-observer.service -o cat")
            .unwrap_err()
            .kind,
        DiagnosticKind::Limit
    );
    let tail =
        terminal::read_only(&state, "journalctl -u work-observer.service -o cat -n 2").unwrap();
    assert_eq!(tail.stdout.lines().count(), 2);
    assert!(tail.stdout.len() <= terminal::MAX_OUTPUT_BYTES);
    assert_eq!(state, before);
    let encoded = serde_json::to_string_pretty(&state).unwrap();
    assert!(encoded.len() < core::MAX_JSON_BYTES);
    assert_eq!(State::from_json(&encoded).unwrap(), state);
}
#[test]
fn goal_context_and_oversized_direct_actions_reject_before_any_candidate_is_published() {
    let f = fixture();
    let mut value = serde_json::to_value(&f.definition.initial).unwrap();
    value["inventory"]["fixture"] = json!("another-valid-fixture");
    let mut state = from_value(&value);
    let before = state.clone();
    assert_eq!(
        terminal::execute(
            &f.definition,
            &mut state,
            "systemctl start work-observer.service"
        )
        .unwrap_err()
        .kind,
        DiagnosticKind::Context
    );
    assert_eq!(state, before);
    let mut state = f.definition.initial.clone();
    let before = state.clone();
    assert!(
        terminal::perform(
            &f.definition,
            &mut state,
            &Action::Start {
                unit: "x".repeat(100_000)
            }
        )
        .is_err()
    );
    assert_eq!(state, before);
    assert!(
        terminal::perform(
            &f.definition,
            &mut state,
            &Action::SaveConfig {
                path: "/etc/work/application.conf".into(),
                text: "x".repeat(100_000)
            }
        )
        .is_err()
    );
    assert_eq!(state, before);
}

#[test]
fn ordered_dependency_failure_records_its_real_blocking_cause_and_no_success_for_consumer() {
    let f = fixture();
    let mut state = f.definition.initial.clone();
    let mut source = state.documents()[APP].clone();
    source = source.replace(
        "Requires=work-prepare.service",
        "Requires=work-prepare.service\nAfter=work-prepare.service",
    );
    action(
        &mut state,
        Action::SaveUnit {
            unit: APP.into(),
            text: source,
        },
    );
    action(
        &mut state,
        Action::SaveConfig {
            path: "/etc/work/prepare.conf".into(),
            text: "WRONG=value\n".into(),
        },
    );
    command(&f, &mut state, "systemctl daemon-reload");
    let result = command(&f, &mut state, "systemctl start work-application.service");
    assert_eq!(result.status, 1);
    assert!(state.journal().unwrap().events().iter().any(|e| e.unit() == APP && matches!(e.kind(), EventKind::DependencyFailed { dependency } if dependency == "work-prepare.service")));
    assert!(
        !state
            .journal()
            .unwrap()
            .events()
            .iter()
            .any(|e| e.unit() == APP
                && matches!(
                    e.kind(),
                    EventKind::SimpleDispatched | EventKind::ProgramInitialized
                ))
    );
    assert!(!workspace::solved(
        &f.definition,
        &state,
        &Draft::new(&state).unwrap()
    ));
}

#[test]
fn malformed_quoting_and_unsupported_operations_are_distinct_without_mutation() {
    let f = fixture();
    let mut state = f.definition.initial.clone();
    let before = state.clone();
    for text in [
        "",
        "systemctl status 'work-observer.service",
        "systemctl status work-observer.service\\",
    ] {
        assert_eq!(
            terminal::execute(&f.definition, &mut state, text)
                .unwrap_err()
                .kind,
            DiagnosticKind::Syntax
        );
        assert_eq!(state, before);
    }
    assert_eq!(
        terminal::execute(
            &f.definition,
            &mut state,
            "systemctl restart work-observer.service"
        )
        .unwrap_err()
        .kind,
        DiagnosticKind::Unsupported
    );
    assert_eq!(state, before);
}

#[test]
fn requested_target_succeeds_despite_failed_wants_but_ordered_requires_blocks_requested_job() {
    let f = fixture();
    for requires in [false, true] {
        let mut state = f.definition.initial.clone();
        action(
            &mut state,
            Action::SaveUnit {
                unit: "work.target".into(),
                text: format!(
                    "[Unit]\nDescription=Requested target\n{}=work-prepare.service\n",
                    if requires { "Requires" } else { "Wants" }
                ),
            },
        );
        action(
            &mut state,
            Action::SaveConfig {
                path: "/etc/work/prepare.conf".into(),
                text: "WRONG=value\n".into(),
            },
        );
        command(&f, &mut state, "systemctl daemon-reload");
        let result = command(&f, &mut state, "systemctl start work.target");
        assert_eq!(result.status, u8::from(requires));
        assert_eq!(
            matches!(
                state.runtime().get("work.target"),
                Some(Runtime::TargetActive {})
            ),
            !requires
        );
        assert!(
            state
                .journal()
                .unwrap()
                .events()
                .iter()
                .any(|event| event.unit() == "work-prepare.service"
                    && matches!(event.kind(), EventKind::StartFailed { .. }))
        );
        if requires {
            assert!(result.stderr.contains("requested job blocked"));
            assert!(
                state
                    .journal()
                    .unwrap()
                    .events()
                    .iter()
                    .any(|event| event.unit() == "work.target"
                        && matches!(event.kind(), EventKind::DependencyFailed { .. }))
            );
        } else {
            assert!(result.stderr.is_empty());
            assert!(
                state
                    .journal()
                    .unwrap()
                    .events()
                    .iter()
                    .any(|event| event.unit() == "work.target"
                        && matches!(event.kind(), EventKind::TargetActivated))
            );
        }
    }
}
fn relation_names(values: &[&str]) -> std::collections::BTreeSet<String> {
    values.iter().map(|s| (*s).into()).collect()
}
#[test]
fn show_effective_direct_relations_include_inverse_target_and_install_edges_without_closure() {
    let f = fixture();
    let mut state = f.definition.initial.clone();
    for (unit, key, relation) in [
        ("work-prepare.service", "Before", OBS),
        (APP, "After", OBS),
        ("work.target", "Wants", APP),
    ] {
        let source =
            state.documents()[unit].replace("[Unit]\n", &format!("[Unit]\n{key}={relation}\n"));
        action(
            &mut state,
            Action::SaveUnit {
                unit: unit.into(),
                text: source,
            },
        );
    }
    command(&f, &mut state, "systemctl daemon-reload");
    let prep = state.effective_properties("work-prepare.service").unwrap();
    let observer = state.effective_properties(OBS).unwrap();
    let app = state.effective_properties(APP).unwrap();
    let target = state.effective_properties("work.target").unwrap();
    assert_eq!(prep.before, relation_names(&[OBS])); // not transitive APP or target
    assert_eq!(observer.after, relation_names(&["work-prepare.service"])); // inverse Before
    assert_eq!(observer.before, relation_names(&[APP, "work.target"])); // inverse After + target link
    assert_eq!(app.after, relation_names(&[OBS]));
    assert_eq!(app.before, relation_names(&["work.target"])); // target default complement
    assert_eq!(app.requires, relation_names(&["work-prepare.service"])); // no implicit order
    assert_eq!(target.wants, relation_names(&[APP, OBS])); // parsed Wants plus installed link
    assert_eq!(target.after, relation_names(&[APP, OBS]));
    let before = state.clone();
    assert_eq!(
        command(
            &f,
            &mut state,
            "systemctl show work-observer.service -p After,Before"
        )
        .stdout,
        "After=work-prepare.service\nBefore=work-application.service work.target\n"
    );
    assert_eq!(
        command(&f, &mut state, "systemctl show work.target -p Wants,After").stdout,
        "Wants=work-application.service work-observer.service\nAfter=work-application.service work-observer.service\n"
    );
    assert_eq!(state, before);
    command(&f, &mut state, "systemctl enable work-prepare.service");
    assert_eq!(
        state.effective_properties("work.target").unwrap().wants,
        relation_names(&[APP, OBS, "work-prepare.service"])
    );
    command(&f, &mut state, "systemctl disable work-observer.service");
    assert_eq!(
        state.effective_properties("work.target").unwrap().wants,
        relation_names(&[APP, "work-prepare.service"])
    );
    // Saved WantedBy alone is install metadata, not an existing Wants relation.
    assert!(
        !state
            .effective_properties("work.target")
            .unwrap()
            .wants
            .contains(OBS)
    );
}
#[test]
fn unsupported_default_dependency_override_is_not_silently_applied_or_hidden_in_show() {
    let f = fixture();
    let mut state = f.definition.initial.clone();
    let original = state.effective_properties("work.target").unwrap();
    let source =
        state.documents()["work.target"].replace("[Unit]\n", "[Unit]\nDefaultDependencies=no\n");
    assert_eq!(
        core::parse_unit(state.inventory(), "work.target", &source)
            .unwrap_err()
            .kind,
        core::ErrorKind::Unsupported
    );
    action(
        &mut state,
        Action::SaveUnit {
            unit: "work.target".into(),
            text: source,
        },
    );
    let before = state.clone();
    assert_eq!(state.effective_properties("work.target").unwrap(), original);
    assert!(terminal::execute(&f.definition, &mut state, "systemctl daemon-reload").is_err());
    assert_eq!(state, before);
    assert_eq!(
        command(&f, &mut state, "systemctl show work.target -p After,Wants").stdout,
        "After=work-observer.service\nWants=work-observer.service\n"
    );
}
#[test]
fn direct_properties_keep_known_missing_dependencies_visible_and_do_not_invent_saved_only_edges() {
    let f = fixture();
    let mut raw = serde_json::to_value(&f.definition.initial).unwrap();
    raw["loaded"]
        .as_object_mut()
        .unwrap()
        .remove("work-prepare.service");
    let mut state = from_value(&raw);
    assert!(
        state
            .effective_properties(APP)
            .unwrap()
            .requires
            .contains("work-prepare.service")
    );
    assert_eq!(
        state
            .effective_properties("work-prepare.service")
            .unwrap_err()
            .kind,
        core::ErrorKind::Missing
    );
    let source =
        state.documents()[APP].replace("[Unit]\n", "[Unit]\nAfter=work-observer.service\n");
    action(
        &mut state,
        Action::SaveUnit {
            unit: APP.into(),
            text: source,
        },
    );
    assert!(!state.effective_properties(APP).unwrap().after.contains(OBS));
    assert_eq!(
        command(
            &f,
            &mut state,
            "systemctl show work-application.service -p Requires,After"
        )
        .stdout,
        "Requires=work-prepare.service\nAfter=\n"
    );
}
