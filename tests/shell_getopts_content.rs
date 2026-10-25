//! Proposed short-option projects. All outcomes are authored data, not captured results.
use dlms_learning_lab::{
    backup, course,
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::{self, View},
    shell::{self, Case, Outcome, Record, Vm},
    workspace,
};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    sync::atomic::{AtomicBool, AtomicUsize},
};

#[derive(Deserialize)]
struct Pack {
    scenarios: Vec<Scenario>,
}
#[derive(Deserialize)]
struct Review {
    id: String,
    alternates: Vec<String>,
    wrong: Vec<Wrong>,
    partial: String,
    try_change: Change,
    first_case_hardcode: String,
    file_damage: String,
    argument_file_format: Option<Format>,
}
#[derive(Deserialize)]
struct Wrong {
    name: String,
    source: String,
    misconception: String,
}
#[derive(Deserialize)]
struct Change {
    source: String,
    expected: Case,
    repair: String,
}
#[derive(Deserialize)]
struct Format {
    canonical_line_edges: bool,
    one_line_per_argument: bool,
    no_eval: bool,
}
#[derive(Deserialize)]
struct Original {
    id: String,
    version: u32,
    unit: String,
}
fn authored() -> Vec<Scenario> {
    serde_json::from_str::<Pack>(include_str!("../scenarios/shell-getopts-v1.json"))
        .unwrap()
        .scenarios
}
fn reviews() -> Vec<Review> {
    serde_json::from_str(include_str!("fixtures/shell-getopts-content-review.json")).unwrap()
}
fn catalog() -> Vec<Scenario> {
    let all = pack::bundled().unwrap();
    for proposed in authored() {
        let normal = all.iter().find(|s| s.id == proposed.id).unwrap();
        assert_eq!(
            serde_json::to_value(normal).unwrap(),
            serde_json::to_value(proposed).unwrap()
        );
    }
    all
}
fn cases(s: &Scenario) -> &[Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!("script goal")
    };
    cases
}
fn outcome(source: &str, case: &Case, slice: usize) -> Outcome {
    let mut vm = Vm::new(source, case).unwrap_or_else(|e| panic!("{}: {e}", case.name));
    for _ in 0..100_000 {
        vm.tick(slice);
        if vm.done() {
            return vm.outcome;
        }
    }
    panic!("bounded script did not finish");
}
fn run(source: &str, s: &Scenario) -> State {
    State::Script {
        source: source.into(),
        run: Some(Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source: source.into(),
            results: cases(s).iter().map(|c| shell::run(source, c)).collect(),
        }),
    }
}

#[test]
fn three_projects_are_normally_bundled_with_real_prerequisites_and_one_unit() {
    const {
        assert!(shell::EVALUATOR_REVISION >= 9);
    }
    let all = catalog();
    assert_eq!(all.len(), 122);
    let units = course::bundled(&all).unwrap();
    assert_eq!(units.len(), 30);
    let proposed: course::Unit =
        serde_json::from_str(include_str!("fixtures/shell-getopts-unit-proposal.json")).unwrap();
    assert!(
        proposed
            .prerequisites
            .iter()
            .all(|p| units.iter().any(|u| &u.id == p))
    );
    assert_eq!(
        proposed.labs,
        authored().iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    assert_eq!(
        serde_json::to_value(units.last().unwrap()).unwrap(),
        serde_json::to_value(proposed).unwrap()
    );
    for s in authored() {
        assert_eq!(all.iter().filter(|o| o.id == s.id).count(), 1);
        assert_eq!(s.version, 1);
        assert_eq!(s.unit, "shell-option-interfaces");
        assert!(
            s.prerequisites
                .iter()
                .all(|p| all.iter().any(|o| &o.id == p))
        );
        assert!(!engine::solved(&s.initial, &s.goal));
        assert!(s.objective.ends_with("Use the supplied filenames exactly."));
        assert_eq!(cases(&s).len(), 12);
        assert!(cases(&s).iter().all(Case::valid));
        assert!(cases(&s).iter().filter(|c| c.status == 0).count() >= 2);
        assert!(cases(&s).iter().filter(|c| c.status != 0).count() >= 2);
        s.teaching.as_ref().unwrap().validate().unwrap();
    }
}

#[test]
fn references_and_both_supported_alternates_match_full_independent_tables() {
    for s in authored() {
        let r = reviews().into_iter().find(|r| r.id == s.id).unwrap();
        assert_eq!(r.alternates.len(), 2);
        assert_ne!(r.alternates[0], s.solution[0]);
        assert_ne!(r.alternates[1], s.solution[0]);
        for source in std::iter::once(&s.solution[0]).chain(&r.alternates) {
            for c in cases(&s) {
                for slice in [1, 16, 64] {
                    let got = outcome(source, c, slice);
                    assert!(c.accepts(&got), "{} / {} / {slice}: {got:?}", s.id, c.name);
                    assert!(
                        got.error.is_none(),
                        "successful source must remain in the declared subset"
                    );
                }
            }
            assert!(engine::solved(&run(source, &s), &s.goal));
        }
        if s.id != "shell-options-publication" {
            assert!(
                !r.alternates[0].contains("getopts "),
                "a genuine manual-parser equivalent"
            );
        }
    }
}

#[test]
fn common_wrong_partial_hardcodes_and_early_file_damage_are_rejected() {
    for s in authored() {
        let r = reviews().into_iter().find(|r| r.id == s.id).unwrap();
        for wrong in &r.wrong {
            assert_ne!(wrong.source, s.solution[0]);
            assert!(wrong.misconception.len() > 30);
            let results: Vec<_> = cases(&s)
                .iter()
                .map(|c| shell::run(&wrong.source, c))
                .collect();
            let pass: Vec<_> = cases(&s)
                .iter()
                .zip(&results)
                .map(|(c, o)| c.accepts(o))
                .collect();
            assert!(
                pass.contains(&false),
                "{} / {} unexpectedly passes",
                s.id,
                wrong.name
            );
            assert!(
                results.iter().any(|o| o.error.is_none()),
                "not solely an unsupported-syntax negative"
            );
            if wrong.name == r.partial {
                assert!(pass.contains(&true), "partial needs real passed cases");
            }
            let record = Some(Record {
                engine_revision: shell::EVALUATOR_REVISION,
                source: wrong.source.clone(),
                results,
            });
            assert!(
                shell::checks(&wrong.source, &record, cases(&s))
                    .iter()
                    .any(|(ok, text)| !ok && !text.is_empty())
            );
        }
        let first = &cases(&s)[0];
        assert!(first.accepts(&shell::run(&r.first_case_hardcode, first)));
        assert!(!engine::solved(&run(&r.first_case_hardcode, &s), &s.goal));
        // Early damage is observable even when later validation returns an error.
        for c in cases(&s) {
            assert!(
                !c.accepts(&shell::run(&r.file_damage, c)),
                "{} / {}",
                s.id,
                c.name
            );
        }
    }
}

#[test]
fn every_declared_row_assesses_exact_output_status_and_preserved_files() {
    for s in authored() {
        for c in cases(&s) {
            let good = shell::run(&s.solution[0], c);
            assert!(c.accepts(&good));
            let mut bad = good.clone();
            bad.status = (bad.status + 1) % 256;
            assert!(!c.accepts(&bad));
            let mut bad = good.clone();
            bad.stdout.push('!');
            assert!(!c.accepts(&bad));
            let mut bad = good.clone();
            bad.stderr.push('!');
            assert!(!c.accepts(&bad));
            let mut bad = good;
            let key = c.files.keys().next().unwrap();
            bad.files.insert(key.clone(), "changed".into());
            assert!(!c.accepts(&bad));
        }
    }
}

#[test]
fn practice_examples_changes_and_repairs_are_separate_runnable_experiments() {
    for s in authored() {
        let lesson = s.teaching.as_ref().unwrap();
        let r = reviews().into_iter().find(|r| r.id == s.id).unwrap();
        assert_ne!(lesson.example, s.solution[0]);
        assert_eq!(r.try_change.repair, lesson.example);
        assert_ne!(lesson.fixture.files, cases(&s)[0].files);
        assert_ne!(r.try_change.source, lesson.example);
        assert_eq!(r.try_change.expected.files, lesson.fixture.files);
        for slice in [1, 16, 64] {
            assert!(
                lesson
                    .fixture
                    .accepts(&outcome(&lesson.example, &lesson.fixture, slice))
            );
            let changed = outcome(&r.try_change.source, &r.try_change.expected, slice);
            assert!(
                r.try_change.expected.accepts(&changed),
                "{} changed: {changed:?}",
                s.id
            );
            assert!(!lesson.fixture.accepts(&changed));
            assert!(
                lesson
                    .fixture
                    .accepts(&outcome(&r.try_change.repair, &lesson.fixture, slice))
            );
        }
    }
}

fn canonical_records(text: &str) -> bool {
    text.split_terminator('\n')
        .all(|line| line == line.trim_matches([' ', '\t']))
}
#[test]
fn argument_files_keep_a_finite_declared_format_without_shell_evaluation() {
    assert!(canonical_records("-p\n\nNorth  room\n*.txt\\label"));
    for bad in [" leading\n", "trailing ", "\tleading", "trailing\t\n"] {
        assert!(!canonical_records(bad));
    }
    for s in authored() {
        let r = reviews().into_iter().find(|r| r.id == s.id).unwrap();
        let Some(format) = r.argument_file_format else {
            assert_eq!(s.id, "shell-options-report");
            continue;
        };
        assert!(format.canonical_line_edges && format.one_line_per_argument && format.no_eval);
        assert!(s.objective.contains("one line is exactly one argument"));
        assert!(
            s.objective
                .contains("Quotes and backslashes are literal data")
        );
        for c in cases(&s).iter().chain([
            &s.teaching.as_ref().unwrap().fixture,
            &r.try_change.expected,
        ]) {
            for (name, text) in &c.files {
                if name.ends_with(".args") {
                    assert!(canonical_records(text), "{} / {name}", c.name);
                }
            }
        }
        for source in std::iter::once(&s.solution[0]).chain(&r.alternates) {
            assert!(
                !source.contains("eval ") && !source.contains("set --") && !source.contains("IFS=")
            );
        }
    }
}

#[test]
fn raw_source_changes_invalidate_current_completion_until_a_real_rerun() {
    for s in authored() {
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        a.state = run(&s.solution[0], &s);
        assert!(progress::current_solved(&a, &s));
        let State::Script { source, .. } = &mut a.state else {
            panic!()
        };
        source.push_str("\n# pending source edit\n");
        assert!(!progress::current_solved(&a, &s));
        let State::Script { source, .. } = &a.state else {
            panic!()
        };
        a.state = run(source, &s);
        assert!(progress::current_solved(&a, &s));
        a.state = run("printf wrong", &s);
        assert!(!progress::current_solved(&a, &s));
        a.state = run(&s.solution[0], &s);
        assert!(progress::current_solved(&a, &s));
    }
}

#[test]
fn fill_is_only_a_draft_and_separate_unfinished_sources_resume_and_reset_locally() {
    let all = catalog();
    let projects = authored();
    let mut p = Progress {
        schema_version: 1,
        clock: 70,
        ..Default::default()
    };
    let old = &pack::bundled().unwrap()[0];
    let old_attempt = Attempt::new(old, 1);
    p.attempts.insert(old.id.clone(), old_attempt.clone());
    for (i, s) in projects.iter().enumerate() {
        let mut a = Attempt::new(s, i as u64 + 2);
        workspace::prepare(&mut a);
        let original = a.state.clone();
        script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        assert_eq!(a.state, original);
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        assert!(l.example_result.is_none());
        assert_eq!(l.example_source, s.teaching.as_ref().unwrap().example);
        // Fill replaces only the example source. It cannot earn assignment credit.
        l.example_source = s.teaching.as_ref().unwrap().example.clone();
        assert!(!progress::current_solved(&a, s));
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        l.example_source.push_str("\nprintf 'unfinished example");
        l.view = View::Example;
        let State::Script { source, .. } = &mut a.state else {
            panic!()
        };
        source.push_str("\nprintf 'unfinished assignment");
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let bytes = serde_json::to_vec_pretty(&p).unwrap();
    let mut loaded = progress::decode(&bytes, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
    assert_eq!(loaded.favorites, p.favorites);
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&old.id]).unwrap(),
        serde_json::to_value(&old_attempt).unwrap()
    );
    let other = loaded.attempts[&projects[1].id].clone();
    loaded
        .attempts
        .insert(projects[0].id.clone(), Attempt::new(&projects[0], 100));
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&projects[1].id]).unwrap(),
        serde_json::to_value(&other).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&old.id]).unwrap(),
        serde_json::to_value(&old_attempt).unwrap()
    );
    assert_eq!(loaded.favorites, p.favorites);
}

#[test]
fn all_original_119_identities_and_actual_frozen_history_survive_read_only_loading() {
    let baseline: Vec<Original> =
        serde_json::from_str(include_str!("fixtures/shell-getopts-original-catalog.json")).unwrap();
    let old = pack::bundled().unwrap();
    let all = catalog();
    assert_eq!(baseline.len(), 119);
    assert_eq!(
        old.iter().map(|s| &s.id).collect::<BTreeSet<_>>().len(),
        122
    );
    // The119 metadata fixture is sorted by source filename, not app order.
    // Preserve actual order using the immutable original116 snapshot plus PTR3.
    let order: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/dns-reverse-original116.json")).unwrap();
    let expected_order: Vec<_> = order["original_scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["id"].as_str().unwrap())
        .chain([
            "lpic2-dns-reverse-investigation",
            "lpic2-dns-reverse-ipv6",
            "lpic2-dns-reverse-handover",
        ])
        .collect();
    assert_eq!(
        old.iter()
            .take(119)
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        expected_order
    );
    assert_eq!(
        old.iter()
            .skip(119)
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "shell-options-report",
            "shell-options-jobs",
            "shell-options-publication"
        ]
    );
    for item in baseline {
        assert!(
            old.iter()
                .any(|s| s.id == item.id && s.version == item.version && s.unit == item.unit)
        );
    }
    let bytes = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let frozen: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let p = progress::decode(bytes, &all).unwrap();
    assert_eq!(p.attempts.len(), 116);
    assert!(authored().iter().all(|s| !p.attempts.contains_key(&s.id)));
    let loaded = serde_json::to_value(&p).unwrap();
    assert_eq!(loaded["attempts"], frozen["attempts"]);
    let mut main = 0;
    let mut six = 0;
    let mut seven = 0;
    for a in p.attempts.values() {
        if let State::Script {
            run: Some(record), ..
        } = &a.state
        {
            assert_eq!(record.engine_revision, 8);
            main += 1;
        }
        if let Some(w) = &a.workspace {
            for record in w.script_history.iter().chain(&w.script_history_more) {
                match record.engine_revision {
                    6 => six += 1,
                    7 => seven += 1,
                    r => panic!("unexpected frozen history {r}"),
                }
            }
        }
    }
    assert_eq!((main, six, seven), (40, 33, 37));
    // No run, inspect, save or revision relabeling was needed to load these exact originals.
    assert_eq!(
        bytes.as_slice(),
        include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json").as_slice()
    );
}

#[test]
fn current_projects_export_inspect_and_confirmed_restore_preserve_separate_sources() {
    let all = catalog();
    let projects = authored();
    let mut p = Progress {
        schema_version: 1,
        clock: 90,
        ..Default::default()
    };
    for (i, s) in projects.iter().enumerate() {
        let mut a = Attempt::new(s, i as u64 + 1);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        a.state = run(&s.solution[0], s);
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        l.example_result = Some(shell::run(
            &l.example_source,
            &s.teaching.as_ref().unwrap().fixture,
        ));
        l.example_revision = shell::EVALUATOR_REVISION;
        l.view = View::Assignment;
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let dir = tempfile::tempdir().unwrap();
    let export = dir.path().join("export.json");
    backup::export(&export, &p, &all).unwrap();
    let original_export = std::fs::read(&export).unwrap();
    let cancel = AtomicBool::new(false);
    let checked = AtomicUsize::new(0);
    let candidate = backup::inspect(&export, &all, &cancel, &checked).unwrap();
    assert_eq!(candidate.rechecked, 6);
    for s in &projects {
        assert!(progress::current_solved(
            &candidate.progress.attempts[&s.id],
            s
        ));
    }
    let profile = dir.path().join("progress.json");
    progress::save(&profile, &p).unwrap();
    let prior = std::fs::read(&profile).unwrap();
    let restored = backup::restore(&profile, &candidate, &all).unwrap();
    assert_eq!(std::fs::read(restored.safety_copy.unwrap()).unwrap(), prior);
    let loaded = progress::decode(&std::fs::read(&profile).unwrap(), &all).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&candidate.progress.attempts).unwrap()
    );
    assert_eq!(loaded.favorites, p.favorites);
    assert_eq!(std::fs::read(&export).unwrap(), original_export);
}
