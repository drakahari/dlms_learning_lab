//! Synthetic composition fixtures; these add no authored labs or learner credit.
use dlms_learning_lab::{
    engine::{self, State as EngineState},
    linux_host::ipv6_host::Node,
    linux_v6_filter::Verdict,
    linux_v6_saved_filter as file,
    linux_v6_startup::{State, Status, UNIT},
    linux_v6_startup_task::Task,
    linux_v6_state_filter::Table,
    linux_v6_workspace::{Expected, PacketInput},
    pack,
};
use std::collections::BTreeSet;
fn fixture() -> (Task, Table) {
    let source = serde_json::from_str::<pack::Pack>(include_str!(
        "../scenarios/linux-v6-admin-lessons-v1.json"
    ))
    .unwrap()
    .scenarios
    .remove(0);
    let mut engine_state = source.initial.clone();
    for command in &source.solution {
        engine::execute_for_goal(&mut engine_state, &source.goal, command).unwrap();
    }
    let EngineState::LinuxV6Administration { simulation } = engine_state else {
        panic!()
    };
    let mut router = simulation.assignment.world;
    router
        .command("sysctl -w net.ipv6.conf.all.forwarding=1")
        .unwrap();
    let raw = include_str!("../scenarios/linux-v6-lessons-v1.json")
        .replace("fd20:30:40", "fd62:70:80")
        .replace("2001:db8:50", "2001:db8:7a")
        .replace("2001:db8:60", "2001:db8:99");
    let data: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let source = data["scenarios"].as_array().unwrap().last().unwrap();
    let mut traffic: dlms_learning_lab::linux_v6_workspace::Task =
        serde_json::from_value(source["goal"]["definition"]["assignment"].clone()).unwrap();
    traffic.facts = router.facts().unwrap();
    let nodes: BTreeSet<_> = traffic
        .cases
        .iter()
        .flat_map(|c| &c.events)
        .filter(|e| e.expected == Expected::Allowed)
        .filter_map(|e| match &e.packet {
            PacketInput::Literal { header } => Some([header.source, header.destination]),
            _ => None,
        })
        .flatten()
        .collect();
    router.host.profile.ipv6.as_mut().unwrap().nodes = nodes
        .into_iter()
        .map(|address| Node {
            address,
            available: true,
        })
        .collect();
    let initial = State {
        version: 1,
        router,
        filter: traffic.initial.filter.clone(),
        saved_filter: file::save(&Table::new(Verdict::Accept)).unwrap(),
        enabled: false,
        status: Status::Inactive,
    };
    let task = Task {
        id: "synthetic-startup-policy".into(),
        initial,
        supplied_settings: "net.ipv6.conf.all.forwarding=1\n".into(),
        traffic,
    };
    task.validate().unwrap();
    let mut correct = task.initial.filter.clone();
    for record in source["solution"].as_array().unwrap() {
        let action: serde_json::Value = serde_json::from_str(
            record
                .as_str()
                .unwrap()
                .strip_prefix("v6-workspace ")
                .unwrap(),
        )
        .unwrap();
        if action["kind"] == "command" {
            correct.command(action["line"].as_str().unwrap()).unwrap();
        }
    }
    (task, correct)
}
fn passes(task: &Task, s: &State) -> bool {
    task.checks(s).iter().all(|c| c.passed)
}
fn solved(task: &Task, correct: &Table) -> State {
    let mut s = task.initial.clone();
    s.filter = correct.clone();
    s.save_draft(&file::save(correct).unwrap()).unwrap();
    s.command(&format!("systemctl enable {UNIT}")).unwrap();
    s
}
#[test]
fn grade_requires_live_saved_and_enabled_policy_and_reports_specific_failures() {
    let (t, correct) = fixture();
    assert!(!passes(&t, &t.initial));
    let mut live = t.initial.clone();
    live.filter = correct.clone();
    assert!(!passes(&t, &live));
    let mut saved = t.initial.clone();
    saved.save_draft(&file::save(&correct).unwrap()).unwrap();
    saved.command(&format!("systemctl enable {UNIT}")).unwrap();
    assert!(!passes(&t, &saved));
    let mut s = solved(&t, &correct);
    let before = s.clone();
    assert!(passes(&t, &s), "{:?}", t.checks(&s));
    assert_eq!(s, before, "checks are isolated");
    s.command("ip6tables -P FORWARD ACCEPT").unwrap();
    assert!(!passes(&t, &s));
    assert!(t.checks(&s).iter().any(|c| !c.passed
        && c.label.starts_with("Current · FORWARD")
        && c.detail.contains("expected")));
    s.filter = correct.clone();
    s.enabled = false;
    assert!(!passes(&t, &s));
    s.enabled = true;
    s.save_draft("*filter\n:INPUT DROP [0:0]\n").unwrap();
    assert!(!passes(&t, &s));
    assert!(t.checks(&s).iter().any(|c| !c.passed
        && c.label == "Startup · provided loader"
        && c.detail.contains("could not")));
}
#[test]
fn loaded_bad_policy_is_not_success_and_file_only_does_not_change_live_table() {
    let (t, correct) = fixture();
    let mut s = solved(&t, &correct);
    s.save_draft(&file::save(&Table::new(Verdict::Accept)).unwrap())
        .unwrap();
    assert_eq!(s.filter, correct);
    s.command(&format!("systemctl restart {UNIT}")).unwrap();
    assert_eq!(s.status, Status::Loaded);
    assert!(!passes(&t, &s));
    assert!(
        t.checks(&s)
            .iter()
            .any(|c| !c.passed && c.label.starts_with("Startup · FORWARD"))
    );
}
#[test]
fn equivalent_split_prefixes_and_file_comments_pass_whole_policy_comparison() {
    let (t, correct) = fixture();
    let mut s = solved(&t, &correct);
    let command = s.filter.command("ip6tables -S FORWARD").unwrap().text;
    s.command("ip6tables -F FORWARD").unwrap();
    for line in command.lines().filter(|l| l.starts_with("-A")) {
        if line.contains("-s fd62:70:80::/64") {
            for prefix in ["fd62:70:80::/65", "fd62:70:80:0:8000::/65"] {
                s.command(&format!(
                    "ip6tables {}",
                    line.replace("fd62:70:80::/64", prefix)
                ))
                .unwrap();
            }
        } else {
            s.command(&format!("ip6tables {line}")).unwrap();
        }
    }
    s.save_draft(&format!(
        "# Equivalent split rules\n{}",
        file::save(&s.filter).unwrap()
    ))
    .unwrap();
    assert!(passes(&t, &s), "{:?}", t.checks(&s));
    assert_ne!(s.filter, correct);
}
#[test]
fn hardcoded_sample_hosts_and_protected_chain_changes_fail() {
    let (t, correct) = fixture();
    let mut s = solved(&t, &correct);
    let limited = file::save(&correct)
        .unwrap()
        .replace("fd62:70:80::/64", "fd62:70:80::55/128");
    s.filter = file::restore(&limited).unwrap();
    s.save_draft(&limited).unwrap();
    assert!(!passes(&t, &s));
    assert!(
        t.checks(&s)
            .iter()
            .any(|c| !c.passed && c.label.contains("FORWARD"))
    );
    let mut s = solved(&t, &correct);
    s.command("ip6tables -F INPUT").unwrap();
    assert!(!passes(&t, &s));
    assert!(
        t.checks(&s)
            .iter()
            .any(|c| !c.passed && c.label == "Current · Preserve INPUT")
    );
}
#[test]
fn invalid_source_paths_network_mutation_and_truncated_documents_are_rejected() {
    let (t, correct) = fixture();
    let mut bad = t.clone();
    bad.initial
        .router
        .host
        .profile
        .ipv6
        .as_mut()
        .unwrap()
        .nodes
        .clear();
    assert!(bad.validate().unwrap_err().contains("physical path"));
    let mut bad = t.clone();
    bad.supplied_settings = "net.ipv6.conf.all.forwarding=0\n".into();
    assert!(bad.validate().is_err());
    let mut s = solved(&t, &correct);
    s.router.settings.global_forwarding = false;
    assert!(t.bound(&s).is_err());
    assert!(!passes(&t, &s));
    for text in ["{}", "{\"id\":\"truncated\"}"] {
        assert!(serde_json::from_str::<Task>(text).is_err());
    }
    let mut v = serde_json::to_value(&t).unwrap();
    v["unexpected"] = serde_json::Value::Bool(true);
    assert!(serde_json::from_value::<Task>(v).is_err());
}

use dlms_learning_lab::linux_v6_startup_workspace::{Action, Definition, Scope, Session};
fn definition() -> (Definition, Table) {
    let (assignment, correct) = fixture();
    let raw = serde_json::to_string(&assignment)
        .unwrap()
        .replace("fd62:70:80", "fd63:71:81")
        .replace("2001:db8:7a", "2001:db8:7b")
        .replace("2001:db8:99", "2001:db8:9a")
        .replace("synthetic-startup-policy", "synthetic-startup-example");
    let example = serde_json::from_str(&raw).unwrap();
    let example_file = file::save(&correct)
        .unwrap()
        .replace("fd62:70:80", "fd63:71:81")
        .replace("2001:db8:7a", "2001:db8:7b")
        .replace("2001:db8:99", "2001:db8:9a");
    let d = Definition {
        source_id: "synthetic-firewall-startup".into(),
        assignment,
        example,
        worked: recipe(&example_file),
    };
    d.validate().unwrap();
    (d, correct)
}
fn recipe(text: &str) -> Vec<Action> {
    vec![
        Action::EditFile { text: text.into() },
        Action::SaveFile,
        Action::Command {
            input: format!(
                "ip6tables-restore {}",
                dlms_learning_lab::linux_v6_startup::FILE
            ),
        },
        Action::Command {
            input: format!("systemctl enable {UNIT}"),
        },
        Action::Check,
    ]
}
#[test]
fn practice_is_separate_unsaved_changes_invalidate_and_reset_preserves_other_draft() {
    let (d, correct) = definition();
    let mut s = Session::new(&d).unwrap();
    let assignment = s.assignment.clone();
    for a in &d.worked {
        s.action(&d, a, true).unwrap();
    }
    assert!(s.practice.checked && s.practice.checks(&d.example).iter().all(|c| c.passed));
    assert_eq!(s.assignment, assignment);
    assert!(!d.solved(&s));
    for a in recipe(&file::save(&correct).unwrap()) {
        s.action(&d, &a, false).unwrap();
    }
    assert!(
        s.assignment.last_startup.is_none(),
        "Check computes actual startup outcomes without requiring a separate report button click"
    );
    assert!(d.solved(&s));
    let practice = s.practice.clone();
    s.action(
        &d,
        &Action::EditFile {
            text: "# Unfinished assignment draft\n*filter\n".into(),
        },
        false,
    )
    .unwrap();
    assert!(!d.solved(&s));
    assert!(!s.assignment.checked);
    assert!(
        s.assignment
            .checks(&d.assignment)
            .iter()
            .any(|c| !c.passed && c.label == "Save the current file draft")
    );
    let restored: Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(s, restored);
    s.reset(&d, false).unwrap();
    assert_eq!(s.practice, practice);
    assert_eq!(s.assignment, assignment);
}
#[test]
fn failed_loader_replays_without_forging_success_and_bad_commands_are_atomic() {
    let (d, _) = definition();
    let mut s = Session::new(&d).unwrap();
    for a in [
        Action::EditFile {
            text: "*filter\n".into(),
        },
        Action::SaveFile,
        Action::Command {
            input: format!("systemctl restart {UNIT}"),
        },
    ] {
        s.action(&d, &a, false).unwrap();
    }
    assert!(matches!(s.assignment.world.status, Status::Failed { .. }));
    assert!(s.assignment.output.contains("Exit status 1"));
    let loaded: Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    d.bound(&loaded).unwrap();
    assert_eq!(loaded, s);
    let before = s.assignment.world.clone();
    let history = s.assignment.history.clone();
    s.action(
        &d,
        &Action::Command {
            input: "bash -c exit".into(),
        },
        false,
    )
    .unwrap();
    assert_eq!(s.assignment.world, before);
    assert_eq!(s.assignment.history, history);
    assert!(s.assignment.output.contains("Not applied"));
    let mut reference = Scope::new(&d.assignment);
    let before = reference.clone();
    assert!(
        reference
            .reference(
                &d.assignment,
                &Action::Command {
                    input: "bash -c exit".into()
                }
            )
            .is_err()
    );
    assert_eq!(reference, before);
}
#[test]
fn cli_save_does_not_overwrite_unsaved_editor_and_cached_feedback_is_source_bound() {
    let (d, correct) = definition();
    let mut s = Session::new(&d).unwrap();
    for a in recipe(&file::save(&correct).unwrap()) {
        s.action(&d, &a, false).unwrap();
    }
    s.action(&d, &Action::RunStartup, false).unwrap();
    s.action(&d, &Action::RunCase, false).unwrap();
    let old = s.clone();
    let mut forged = s.clone();
    forged.assignment.last_startup.as_mut().unwrap().checks[0].passed = false;
    assert!(d.bound(&forged).is_err());
    let mut forged = s.clone();
    forged.assignment.world.filter = Table::new(Verdict::Accept);
    assert!(d.bound(&forged).is_err());
    s.action(
        &d,
        &Action::EditFile {
            text: "# Keep my unsaved draft\n".into(),
        },
        false,
    )
    .unwrap();
    assert!(s.assignment.last_startup.is_none());
    assert!(!d.solved(&s));
    s.action(
        &d,
        &Action::Command {
            input: format!(
                "ip6tables-save -f {}",
                dlms_learning_lab::linux_v6_startup::FILE
            ),
        },
        false,
    )
    .unwrap();
    assert_eq!(s.assignment.editor, "# Keep my unsaved draft\n");
    s.action(&d, &Action::ReloadEditor, false).unwrap();
    assert_eq!(s.assignment.editor, s.assignment.world.saved_filter);
    assert!(s.assignment.last_trial.is_none());
    assert_eq!(old.practice, s.practice);
}
#[test]
fn bounded_edits_commands_and_source_identity_fail_without_data_loss() {
    let (d, _) = definition();
    let mut s = Session::new(&d).unwrap();
    let original = s.clone();
    for a in [
        Action::EditFile {
            text: "x".repeat(32769),
        },
        Action::Input {
            text: "x".repeat(513),
        },
        Action::SelectCase { case: usize::MAX },
    ] {
        assert!(s.action(&d, &a, false).is_err());
        assert_eq!(s, original);
    }
    let mut bad = s.clone();
    bad.source_id = "different-source".into();
    assert!(d.bound(&bad).is_err());
    let mut bad = d.clone();
    bad.example = bad.assignment.clone();
    assert!(bad.validate().is_err());
    let mut scope = Scope::new(&d.assignment);
    for n in 0..96 {
        scope
            .apply(
                &d.assignment,
                &Action::Command {
                    input: format!(
                        "ip6tables -P FORWARD {}",
                        if n % 2 == 0 { "ACCEPT" } else { "DROP" }
                    ),
                },
            )
            .unwrap();
    }
    assert_eq!(scope.history.len(), 96);
    let before = scope.clone();
    assert!(
        scope
            .apply(
                &d.assignment,
                &Action::Command {
                    input: "ip6tables -P FORWARD ACCEPT".into()
                }
            )
            .is_err()
    );
    assert_eq!(scope, before);
}
#[test]
fn policy_feedback_names_headers_ports_and_defaults_without_raw_option_values() {
    let (t, _) = fixture();
    let mut s = t.initial.clone();
    s.filter = Table::new(Verdict::Accept);
    let rows = t.checks(&s);
    let failures = rows
        .iter()
        .filter(|c| !c.passed && c.label.contains("FORWARD"))
        .collect::<Vec<_>>();
    assert!(!failures.is_empty());
    for row in failures {
        assert!(row.detail.contains("expected") && row.detail.contains("actual"));
        assert!(!row.detail.contains("Some(") && !row.detail.contains("None"));
        assert!(row.detail.contains("chain default"));
    }
}
