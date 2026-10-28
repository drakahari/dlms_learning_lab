//! Four distinct authored tasks, not registered until native/progress acceptance.
#[path = "support/file_backup_authored.rs"]
mod fixture;
use dlms_learning_lab::{
    engine::{self, Goal, State},
    file_backup_teaching::Learning,
    pack::{self, Scenario},
};
use fixture::*;
use serde_json::Value;
#[path = "../examples/support/automation_digest.rs"]
mod digest;
fn data(s: &Scenario) -> &dlms_learning_lab::file_backup_terminal::Store {
    let State::FileBackup { simulation } = &s.initial else {
        panic!("Typed virtual backup task")
    };
    simulation
}
fn execute(s: &Scenario, commands: &[String]) -> State {
    let mut state = s.initial.clone();
    for c in commands {
        engine::execute_for_goal(&mut state, &s.goal, c)
            .unwrap_or_else(|e| panic!("{}: {c}: {e}", s.id));
    }
    state
}
fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().into())
        .collect()
}

fn hash(bytes: &[u8]) -> String {
    digest::hash(bytes)
}

#[test]
fn authored_pack_references_and_valid_alternatives_reach_actual_outcomes() {
    for row in review()["labs"].as_array().unwrap() {
        let s = lab(row["id"].as_str().unwrap());
        assert!(!engine::solved(&s.initial, &s.goal));
        for commands in [&s.solution, &strings(&row["alternate"])] {
            let result = execute(s, commands);
            assert!(
                engine::solved(&result, &s.goal),
                "{}: {:?}",
                s.id,
                engine::checks(&result, &s.goal)
            );
        }
    }
}
#[test]
fn wrong_and_partial_work_fail_without_grading_commands_or_earlier_success() {
    for row in review()["labs"].as_array().unwrap() {
        let s = lab(row["id"].as_str().unwrap());
        for name in ["wrong", "partial"] {
            let state = execute(s, &strings(&row[name]));
            assert!(!engine::solved(&state, &s.goal), "{} {name}", s.id);
            assert!(engine::checks(&state, &s.goal).iter().any(|r| !r.0));
        }
        let mut solved = execute(s, &s.solution);
        assert!(engine::solved(&solved, &s.goal));
        for c in strings(&row["break"]) {
            engine::execute_for_goal(&mut solved, &s.goal, &c).unwrap();
        }
        assert!(!engine::solved(&solved, &s.goal), "{} stale result", s.id);
    }
}
#[test]
fn every_introduction_example_experiment_and_repair_are_real_and_separate() {
    for row in review()["labs"].as_array().unwrap() {
        let s = lab(row["id"].as_str().unwrap());
        let lesson = s.file_backup_teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        assert!(lesson.separate_from(data(s)));
        let original = serde_json::to_value(&s.initial).unwrap();
        let mut learning = Learning::new(lesson).unwrap();
        for c in &lesson.example_commands {
            learning.command = c.command.clone();
            learning.execute(lesson).unwrap();
        }
        assert!(
            lesson
                .definition
                .checks(&learning.state)
                .iter()
                .all(|r| r.0)
        );
        for c in &lesson.try_change.commands {
            learning.command = c.command.clone();
            learning.execute(lesson).unwrap();
        }
        assert_eq!(
            lesson
                .definition
                .checks(&learning.state)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            lesson.try_change.expected,
            "{} experiment",
            s.id
        );
        for c in &lesson.repair {
            learning.command = c.command.clone();
            learning.execute(lesson).unwrap();
        }
        assert!(
            lesson
                .definition
                .checks(&learning.state)
                .iter()
                .all(|r| r.0)
        );
        assert_eq!(serde_json::to_value(&s.initial).unwrap(), original);
        assert!(!engine::solved(&s.initial, &s.goal));
    }
}
#[test]
fn old37_packs_154_definitions_and_raw39_units_are_not_rewritten() {
    let r = review();
    let texts = r["original_source_pins"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            let name = p["path"].as_str().unwrap();
            assert!(
                name.starts_with("scenarios/")
                    && std::path::Path::new(name).components().count() == 2
            );
            let bytes =
                std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name)).unwrap();
            assert_eq!(hash(&bytes), p["sha256"].as_str().unwrap());
            String::from_utf8(bytes).unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(texts.len(), 37);
    let old = pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(old.len(), 154);
    let current = catalog();
    assert_eq!(
        serde_json::to_value(&current[..154]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(
        current
            .iter()
            .filter(|s| s.file_backup_teaching.is_some())
            .count(),
        4
    );
    let units = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("scenarios/units-v1.json"),
    )
    .unwrap();
    let raw = String::from_utf8(units).unwrap();
    let start = raw.find("\"units\": [").unwrap() + "\"units\": [".len();
    let (mut quoted, mut escaped, mut depth, mut count) = (false, false, 0usize, 0usize);
    let mut end = None;
    for (offset, c) in raw[start..].char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                quoted = false;
            }
            continue;
        }
        match c {
            '"' => quoted = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    count += 1;
                    if count == 39 {
                        end = Some(start + offset + 1);
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    let reconstructed = format!(
        "{}{}",
        &raw[..end.unwrap()],
        r["original_units_tail"].as_str().unwrap()
    );
    assert_eq!(
        hash(reconstructed.as_bytes()),
        r["original_units_sha256"].as_str().unwrap()
    );
    let parsed: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(parsed["version"], 1);
    assert!(parsed["units"].as_array().unwrap().len() >= 39);
}
#[test]
fn malformed_context_and_unsupported_commands_cannot_mutate_the_assignment() {
    let s = lab("lpic2-backup-selective-recovery");
    let mut state = s.initial.clone();
    for command in [
        "tar -xf /backups/project-full.tar -C /personal",
        "tar -xf /backups/project-full.tar -C /restore project/../../personal",
        "tar -czf /backups/out.tar -C /srv project",
        "tar -cf /backups/a.tar -C /srv project; echo escaped",
        "bash /etc/profile",
    ] {
        let before = state.clone();
        assert!(
            engine::execute_for_goal(&mut state, &s.goal, command).is_err(),
            "{command}"
        );
        assert_eq!(state, before);
    }
    let wrong = Goal::Subnet;
    let before = state.clone();
    assert!(
        engine::execute_for_goal(&mut state, &wrong, "rm /restore/project/handover.txt").is_err()
    );
    assert_eq!(state, before);
    assert!(engine::execute(&mut state, "rm /restore/project/handover.txt").is_err());
}
