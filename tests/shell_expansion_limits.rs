use dlms_learning_lab::{
    backup,
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    shell::{self, Case, Outcome, Record, Vm},
    workspace,
};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, AtomicUsize},
};
fn case() -> Case {
    Case {
        name: "Controlled expansion fixture".into(),
        args: vec![],
        stdin: String::new(),
        env: BTreeMap::new(),
        files: BTreeMap::new(),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
fn limited(source: &str, c: &Case, kind: &str) -> Outcome {
    let r = shell::run(source, c);
    assert_eq!(r.status, 2);
    // Reviewed v7 groundwork: aggregate live memory may reject a pending
    // expansion before its older per-expansion ceiling. Both remain failures.
    let expected = if kind == "Expansion memory limit" {
        matches!(
            r.error.as_deref(),
            Some(
                "Expansion memory limit reached (256 KiB of expanded text)"
                    | "Virtual memory/file limit exceeded"
            )
        )
    } else {
        r.error.as_ref().is_some_and(|e| e.contains(kind))
    };
    assert!(expected, "{source:.120}: {:?}", r.error);
    r
}
#[test]
fn multipart_fields_and_all_direct_consumers_fail_before_retaining_oversized_text() {
    let mut c = case();
    c.env.insert("blob".into(), "x".repeat(8192));
    c.files.insert("/work/output.txt".into(), "retained".into());
    for part in ["\"$blob\"''", "$blob''", "$blob\\x", "\"$blob\"'é'"] {
        let word = part.repeat(1000);
        assert!(word.len() < 16_000);
        for source in [
            format!("test -n {word}"),
            format!("value={word}"),
            format!("f() {{ local value={word}; }}; f"),
            format!("cat < {word}"),
            format!("while false; do :; done < {word}"),
            format!("printf '%s' {word} > output.txt"),
        ] {
            let r = limited(&source, &c, "Expansion memory limit");
            assert_eq!(r.files, c.files);
            assert!(r.stdout.is_empty());
        }
    }
    c.env.insert("blob".into(), "é".repeat(4096));
    let r = limited(
        &format!("test -n {}", "\"$blob\"''".repeat(40)),
        &c,
        "Expansion memory limit",
    );
    assert_eq!(r.files, c.files);
    // Actual v6 accepted29/30/31; their frozen inputs/results and original
    // test hash are in shell-v6-resource-before-substitution.json. With all
    // pending values charged,30 crosses the copy peak while29 still fits.
    let small = shell::run(&format!("test -n {}", "\"$blob\"''".repeat(29)), &c);
    assert_eq!(small.error, None);
    assert_eq!(small.status, 0);
    let boundary = limited(
        &format!("test -n {}", "\"$blob\"''".repeat(30)),
        &c,
        "Expansion memory limit",
    );
    assert_eq!(boundary.files, c.files);
}
#[test]
fn expanded_field_count_and_aggregate_command_bytes_are_bounded_before_execution() {
    let mut c = case();
    c.env.insert("list".into(), "x ".repeat(200));
    c.files
        .insert("/work/output.txt".into(), "retain before redirect".into());
    for source in [
        "printf '%s' $list > output.txt",
        "cat < $list",
        "while false; do :; done < $list",
    ] {
        let r = limited(source, &c, "Argument limit");
        assert_eq!(r.files, c.files);
    }
    c.env.clear();
    c.env.insert("blob".into(), "x".repeat(65536));
    let r = limited(
        "test -n \"$blob\" \"$blob\" \"$blob\" \"$blob\"",
        &c,
        "Expansion memory limit",
    );
    assert_eq!(r.files, c.files);
    let source = format!("printf '%s' {}", "x ".repeat(127));
    limited(&source, &case(), "argument/redirection limit");
}
#[test]
fn quoted_empty_fields_utf8_and_sole_at_keep_their_meaning_within_limits() {
    for (source, expected) in [
        ("printf '<%s>\\n' \"\" x\\ y 'é'", "<>\n<x y>\n<é>\n"),
        (
            "text=' a  b '; printf '<%s>\\n' $text\"\"",
            "<a>\n<b>\n<>\n",
        ),
        ("printf '<%s>\\n' \"$@\"", "<one two>\n<>\n<☃>\n"),
    ] {
        let mut c = case();
        c.args = vec!["one two".into(), String::new(), "☃".into()];
        let r = shell::run(source, &c);
        assert_eq!(r.error, None);
        assert_eq!(r.stdout, expected);
    }
    let r = shell::run("printf '<%s>' \"$@\"", &case());
    assert_eq!(r.stdout, "<>");
    assert_eq!(r.error, None);
    for (count, okay) in [(126, true), (127, false)] {
        let source = format!("f() {{ printf '%s' \"$@\"; }}; f {}", "x ".repeat(count));
        let r = shell::run(&source, &case());
        assert_eq!(r.error.is_none(), okay, "{count}: {:?}", r.error);
        if okay {
            assert_eq!(r.stdout, "x".repeat(count));
        }
    }
}
#[test]
fn work_is_cumulative_across_functions_pipelines_loops_and_ticks_but_new_vms_are_fresh() {
    let mut c = case();
    c.env.insert("blob".into(), "x".repeat(8192));
    let sources = [
        format!("f() {{ test -n \"$blob\"; }}; {}", "f;".repeat(80)),
        "test -n \"$blob\" | test -n \"$blob\";".repeat(40),
        format!(
            "for item in {}; do test -n \"$blob\"; done",
            "x ".repeat(80)
        ),
    ];
    for source in sources {
        let expected = limited(&source, &c, "Expansion work limit");
        for slice in [1, 16, 64] {
            let mut vm = Vm::new(&source, &c).unwrap();
            for _ in 0..2000 {
                if vm.done() {
                    break;
                }
                vm.tick(slice);
            }
            assert!(vm.done());
            assert_eq!(vm.outcome, expected);
        }
        let r = shell::run("test -n \"$blob\"", &c);
        assert_eq!(r.error, None);
        assert_eq!(r.status, 0);
    }
    let mut arithmetic = case();
    arithmetic.env.insert("blob".into(), "0".repeat(4096));
    limited(
        &"printf '%s' \"$((blob))\";".repeat(350),
        &arithmetic,
        "Expansion work limit",
    );
    let mut args = case();
    args.args = vec!["x".repeat(2000); 64];
    let good = shell::run("f() { :; }; f \"$@\"", &args);
    assert_eq!(good.error, None);
    limited(
        &format!("f() {{ :; }}; {}", "f \"$@\";".repeat(10)),
        &args,
        "Expansion work limit",
    );
    let mut vm = Vm::new("while true; do :; done", &case()).unwrap();
    vm.tick(16);
    assert!(!vm.done());
    vm.stop();
    assert!(vm.done());
    assert!(vm.outcome.error.unwrap().contains("Stopped"));
}
fn earlier_cases() -> Vec<Case> {
    [32768, 65536, 98304]
        .into_iter()
        .map(|size| {
            let mut c = case();
            c.name = format!("Actual prior expansion {size}");
            c.env.insert("spaces".into(), " ".repeat(size));
            c.stdout = "\n".repeat(32);
            c
        })
        .collect()
}
fn prior_source() -> String {
    format!("printf '%s\\n' {}\n", "$spaces\"\"".repeat(32))
}
#[test]
fn actual_revision3_resource_credit_is_historical_and_exact_bytes_survive_revision4_upgrade() {
    let mut lab = pack::bundled().unwrap()[9].clone();
    lab.id = "test-expansion-upgrade".into();
    lab.teaching = None;
    lab.prerequisites.clear();
    lab.unit.clear();
    let cases = earlier_cases();
    lab.goal = Goal::Script {
        cases: cases.clone(),
    };
    let old = Record {
        engine_revision: 3,
        source: prior_source(),
        results: cases
            .iter()
            .map(|c| Outcome {
                stdout: c.stdout.clone(),
                stderr: String::new(),
                status: 0,
                files: BTreeMap::new(),
                error: None,
            })
            .collect(),
    };
    assert!(old.valid());
    let mut a = Attempt::new(&lab, 1);
    a.state = State::Script {
        source: old.source.clone(),
        run: Some(old.clone()),
    };
    assert!(progress::legacy_script_credit(&a, &lab));
    assert!(!progress::current_solved(&a, &lab));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), a);
    let bytes = serde_json::to_vec_pretty(&p).unwrap();
    std::fs::write(&path, &bytes).unwrap();
    let mut p = progress::load(&path, std::slice::from_ref(&lab)).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let a = p.attempts.get_mut(&lab.id).unwrap();
    workspace::prepare(a);
    assert_eq!(
        a.workspace.as_ref().unwrap().script_history.as_ref(),
        Some(&old)
    );
    let State::Script { source, run } = &mut a.state else {
        panic!()
    };
    *run = Some(Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.clone(),
        results: cases.iter().map(|c| shell::run(source, c)).collect(),
    });
    assert!(
        run.as_ref().unwrap().results.iter().all(|r| r
            .error
            .as_ref()
            .unwrap()
            .contains("Expansion work limit"))
    );
    assert!(!progress::current_solved(a, &lab));
    assert!(progress::legacy_script_credit(a, &lab));
    progress::save(&path, &p).unwrap();
    let before = path.with_extension(format!("before-shell-v{}.json", shell::EVALUATOR_REVISION));
    assert_eq!(std::fs::read(&before).unwrap(), bytes);
    progress::save(&path, &p).unwrap();
    assert_eq!(std::fs::read(&before).unwrap(), bytes);
    let export = dir.path().join("export.json");
    backup::export(&export, &p, std::slice::from_ref(&lab)).unwrap();
    let reviewed = backup::inspect(
        &export,
        std::slice::from_ref(&lab),
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    assert!(progress::legacy_script_credit(
        &reviewed.progress.attempts[&lab.id],
        &lab
    ));
    let repair = format!("printf '%s\\n' {}", "\"\" ".repeat(32));
    let a = p.attempts.get_mut(&lab.id).unwrap();
    a.state = State::Script {
        source: repair.clone(),
        run: Some(Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source: repair.clone(),
            results: cases.iter().map(|c| shell::run(&repair, c)).collect(),
        }),
    };
    assert!(progress::current_solved(a, &lab));
    assert!(progress::legacy_script_credit(a, &lab));
    p.attempts.insert(lab.id.clone(), Attempt::new(&lab, 2));
    progress::save(&path, &p).unwrap();
    assert_eq!(std::fs::read(&before).unwrap(), bytes);
}
