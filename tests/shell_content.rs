//! Authored content review: solutions are compared by behavior, never source text.
use dlms_learning_lab::{engine::Goal, pack, shell};
use serde::Deserialize;
#[derive(Deserialize)]
struct Review {
    id: String,
    alternate: String,
    incorrect: String,
    misconception: String,
}
fn reviews() -> Vec<Review> {
    let mut rows: Vec<Review> =
        serde_json::from_str(include_str!("fixtures/shell-content-review.json")).unwrap();
    let options: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/shell-getopts-content-review.json")).unwrap();
    rows.extend(options.into_iter().map(|r| Review {
        id: r["id"].as_str().unwrap().into(),
        alternate: r["alternates"][0].as_str().unwrap().into(),
        incorrect: r["wrong"][0]["source"].as_str().unwrap().into(),
        misconception: r["wrong"][0]["misconception"].as_str().unwrap().into(),
    }));
    let safe: Vec<Review> = serde_json::from_str(include_str!(
        "fixtures/shell-safe-automation-native-review.json"
    ))
    .unwrap();
    rows.extend(safe);
    rows
}
fn quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
#[test]
fn every_script_has_a_working_alternate_and_rejects_a_plausible_mistake_and_hardcoding() {
    let labs = pack::bundled().unwrap();
    let rows = reviews();
    assert_eq!(
        rows.len(),
        labs.iter()
            .filter(|s| matches!(s.goal, Goal::Script { .. }))
            .count()
    );
    assert_eq!(
        rows.iter()
            .map(|r| r.id.as_str())
            .collect::<std::collections::BTreeSet<_>>(),
        labs.iter()
            .filter(|s| matches!(s.goal, Goal::Script { .. }))
            .map(|s| s.id.as_str())
            .collect()
    );
    for row in rows {
        let lab = labs.iter().find(|s| s.id == row.id).unwrap();
        let Goal::Script { cases } = &lab.goal else {
            panic!()
        };
        assert!(cases.len() >= 3);
        assert!(row.misconception.len() > 15);
        for case in cases {
            let actual = shell::run(&row.alternate, case);
            assert!(
                case.accepts(&actual),
                "{} alternate/{}: {:?}",
                row.id,
                case.name,
                actual
            );
        }
        assert!(
            cases
                .iter()
                .any(|c| !c.accepts(&shell::run(&row.incorrect, c))),
            "{} does not diagnose {}",
            row.id,
            row.misconception
        );
        // Construct an overfit program that exactly reproduces case 1, including files.
        // At least one different input must reject it. This checks the case design.
        let first = &cases[0];
        let mut overfit = format!(
            "printf '%s' {}\nprintf '%s' {} >&2\n",
            quoted(&first.stdout),
            quoted(&first.stderr)
        );
        for (path, value) in &first.after {
            if first.files.get(path) != Some(value) {
                overfit.push_str(&format!(
                    "printf '%s' {} > {}\n",
                    quoted(value),
                    quoted(path)
                ));
            }
        }
        overfit.push_str(&format!("exit {}\n", first.status));
        assert!(
            first.accepts(&shell::run(&overfit, first)),
            "{} invalid test fixture",
            row.id
        );
        assert!(
            cases
                .iter()
                .skip(1)
                .any(|c| !c.accepts(&shell::run(&overfit, c))),
            "{} accepts hardcoded first case",
            row.id
        );
    }
}
#[test]
fn authored_mistakes_can_be_partial_but_never_earn_full_credit() {
    let labs = pack::bundled().unwrap();
    let rows = reviews();
    for id in [
        "shell-eof",
        "shell-batch-audit",
        "shell-functions",
        "shell-return",
        "shell-shift",
        "shell-bundle",
    ] {
        let row = rows.iter().find(|r| r.id == id).unwrap();
        let lab = labs.iter().find(|s| s.id == id).unwrap();
        let Goal::Script { cases } = &lab.goal else {
            panic!()
        };
        let results: Vec<_> = cases
            .iter()
            .map(|c| c.accepts(&shell::run(&row.incorrect, c)))
            .collect();
        assert!(
            results.contains(&true) && results.contains(&false),
            "{id}: {results:?}"
        );
    }
}
#[test]
fn batch_no_matches_is_not_a_missing_file_and_does_not_change_final_status() {
    let labs = pack::bundled().unwrap();
    let lab = labs.iter().find(|s| s.id == "shell-batch-audit").unwrap();
    let Goal::Script { cases } = &lab.goal else {
        panic!()
    };
    let wrong = lab.solution[0].replace(
        "grep -F ERROR \"$path\"",
        "grep -F ERROR \"$path\" || failed=yes",
    );
    let quiet = cases
        .iter()
        .find(|c| c.name == "Readable quiet log after an incident")
        .unwrap();
    let r = shell::run(&wrong, quiet);
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, quiet.stdout);
    assert_eq!(r.status, 2);
    assert!(!quiet.accepts(&r));
}
#[test]
fn literal_filter_must_not_assume_a_space_after_the_pattern() {
    let labs = pack::bundled().unwrap();
    let lab = labs.iter().find(|s| s.id == "shell-pipelines").unwrap();
    let Goal::Script { cases } = &lab.goal else {
        panic!()
    };
    let wrong = "grep -F 'ERROR ' \"$1\" | grep -F \"$2\"";
    let boundary = cases.last().unwrap();
    assert!(!boundary.accepts(&shell::run(wrong, boundary)));
}
#[test]
fn preflight_failure_must_not_even_create_an_empty_new_destination() {
    let labs = pack::bundled().unwrap();
    let lab = labs.iter().find(|s| s.id == "shell-bundle").unwrap();
    let Goal::Script { cases } = &lab.goal else {
        panic!()
    };
    let dlms_learning_lab::engine::State::Script { source, .. } = &lab.initial else {
        panic!()
    };
    let wrong = source.replace(
        "printf '' > \"$destination\"",
        "printf '' >> \"$destination\"",
    );
    let c = cases.last().unwrap();
    let r = shell::run(&wrong, c);
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, c.stdout);
    assert_eq!(r.stderr, c.stderr);
    assert_eq!(r.status, c.status);
    assert!(r.files.contains_key("/work/new-bundle"));
    assert!(!c.accepts(&r));
}
