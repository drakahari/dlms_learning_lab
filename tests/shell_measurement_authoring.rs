//! Authoring capability checks only; these fixtures are not registered lessons.
//! All programs here run in the existing bounded in-process evaluator19.
use dlms_learning_lab::{
    resource_measurements as model,
    shell::{self, Case},
};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema_version: u32,
    source: String,
    alternate: String,
    cases: Vec<Case>,
    example_source: String,
    example_case: Case,
}
fn fixture(kind: &str) -> Fixture {
    let text = match kind {
        "rates" => include_str!("fixtures/shell-measurement-rates-reference.json"),
        "cpu" => include_str!("fixtures/shell-measurement-cpu-reference.json"),
        "recovery" => include_str!("fixtures/shell-measurement-recovery-reference.json"),
        _ => panic!(),
    };
    let f: Fixture = serde_json::from_str(text).unwrap();
    assert_eq!(f.schema_version, 1);
    assert_eq!(f.cases.len(), 12);
    assert!(f.cases.iter().all(Case::valid));
    assert!(f.example_case.valid());
    f
}
#[test]
fn two_supported_implementations_match_all_thirty_six_behavioral_cases() {
    assert_eq!(shell::EVALUATOR_REVISION, 19);
    for name in ["rates", "cpu", "recovery"] {
        let f = fixture(name);
        for source in [&f.source, &f.alternate] {
            for c in &f.cases {
                let actual = shell::run(source, c);
                assert!(c.accepts(&actual), "{name} / {}: {actual:?}", c.name);
            }
        }
    }
}
#[test]
fn worked_programs_use_separate_files_and_actually_produce_their_expected_results() {
    for name in ["rates", "cpu", "recovery"] {
        let f = fixture(name);
        if name == "recovery" {
            assert!(f.example_case.files.contains_key("/work/practice.csv"));
            assert!(!f.example_case.files.contains_key("/work/observations.csv"));
        } else {
            assert!(f.example_case.files.contains_key("/work/sample-before.csv"));
            assert!(!f.example_case.files.contains_key("/work/before.csv"));
        }
        let actual = shell::run(&f.example_source, &f.example_case);
        assert!(f.example_case.accepts(&actual), "{name}: {actual:?}");
    }
}
fn numbers(text: &str) -> Vec<u64> {
    text.trim_end_matches('\n')
        .split(',')
        .map(|v| v.parse().unwrap())
        .collect()
}
fn snapshot() -> model::Snapshot {
    model::Snapshot {
        boot: "fixed".into(),
        at_ms: 1000,
        cores: 2,
        cpu: model::Cpu { ticks: [0; 8] },
        memory: model::Memory {
            total_kib: 1024,
            free_kib: 100,
            available_kib: 500,
            swap_total_kib: 0,
            swap_free_kib: 0,
        },
        disk: model::Disk {
            name: "vda".into(),
            read_ops: 0,
            write_ops: 0,
            read_sectors: 0,
            write_sectors: 0,
            read_ms: 0,
            write_ms: 0,
        },
        network: model::Network {
            name: "eth0".into(),
            rx_bytes: 0,
            tx_bytes: 0,
        },
    }
}
#[test]
fn expected_outputs_are_independently_checked_against_typed_counter_math() {
    for c in fixture("rates").cases {
        let a: Vec<_> = c.files["/work/before.csv"]
            .trim_end_matches('\n')
            .split(',')
            .collect();
        let b: Vec<_> = c.files["/work/after.csv"]
            .trim_end_matches('\n')
            .split(',')
            .collect();
        let mut before = snapshot();
        let mut after = snapshot();
        for (row, s) in [(&a, &mut before), (&b, &mut after)] {
            s.boot = row[0].into();
            s.network.name = row[1].into();
            s.at_ms = row[2].parse().unwrap();
            s.network.rx_bytes = row[3].parse().unwrap();
            s.network.tx_bytes = row[4].parse().unwrap();
        }
        after.cpu.ticks[3] = 200;
        match model::measure(&before, &after) {
            Ok(w) => {
                let text = format!(
                    "{} rx={} B/s tx={} B/s\n",
                    after.network.name,
                    w.rx_bytes_per_second.scaled_floor(1).unwrap(),
                    w.tx_bytes_per_second.scaled_floor(1).unwrap()
                );
                assert_eq!(c.stdout, text);
                assert_eq!(c.status, 0);
            }
            Err(e) => {
                assert_eq!(c.status, 2);
                let expected = match e {
                    model::Error::Identity => "Different source\n",
                    model::Error::Time => "Invalid interval\n",
                    model::Error::Reset => "Counter reset\n",
                    _ => panic!("{e:?}"),
                };
                assert_eq!(c.stderr, expected);
                assert!(c.stdout.is_empty());
            }
        }
    }
    for c in fixture("cpu").cases {
        let mut a = snapshot();
        let mut b = snapshot();
        b.at_ms = 2000;
        a.cpu.ticks = numbers(&c.files["/work/before.csv"]).try_into().unwrap();
        b.cpu.ticks = numbers(&c.files["/work/after.csv"]).try_into().unwrap();
        match model::measure(&a, &b) {
            Ok(w) => {
                assert_eq!(c.status, 0);
                assert_eq!(
                    c.stdout,
                    format!(
                        "work={}% idle={}% wait={}% steal={}%\n",
                        w.cpu_work_share.scaled_floor(100).unwrap(),
                        w.cpu_shares[3].scaled_floor(100).unwrap(),
                        w.cpu_shares[4].scaled_floor(100).unwrap(),
                        w.cpu_shares[7].scaled_floor(100).unwrap()
                    )
                );
            }
            Err(e) => {
                assert_eq!(c.status, 2);
                assert_eq!(
                    c.stderr,
                    match e {
                        model::Error::Reset => "Counter decreased\n",
                        model::Error::NoCpuInterval => "CPU counters did not advance\n",
                        _ => panic!("{e:?}"),
                    }
                );
                assert!(c.stdout.is_empty());
            }
        }
    }
}
fn flags(f: &Fixture, source: &str) -> Vec<bool> {
    let outcomes: Vec<_> = f.cases.iter().map(|c| shell::run(source, c)).collect();
    assert!(
        outcomes.iter().all(|r| r.error.is_none()),
        "Plausible wrong approaches must execute: {outcomes:?}"
    );
    f.cases
        .iter()
        .zip(&outcomes)
        .map(|(c, r)| c.accepts(r))
        .collect()
}
#[test]
fn unit_errors_boot_averages_wrong_cpu_categories_and_changed_inputs_fail_partially() {
    for kind in ["rates", "cpu"] {
        let f = fixture(kind);
        let wrong = if kind == "rates" {
            vec![
                f.source.replace(" * 1000 / elapsed", " / elapsed"),
                f.source
                    .replace("elapsed=$((last_time - first_time))", "elapsed=1000"),
                f.source
                    .replace("(last_rx - first_rx)", "last_rx")
                    .replace("(last_tx - first_tx)", "last_tx"),
            ]
        } else {
            vec![
                f.source.replace("change=$((last-first))", "change=$last"),
                f.source.replace("1|2|3|6|7)", "1|2|3)"),
                f.source
                    .replace("5) wait=$change", "5) work=$((work+change)); wait=$change"),
            ]
        };
        for source in wrong {
            let passed = flags(&f, &source);
            assert!(
                passed.iter().any(|p| *p) && passed.iter().any(|p| !*p),
                "{kind}: {passed:?}"
            );
        }
        let first = &f.cases[0];
        assert!(!first.stdout.contains('\''));
        let hardcoded = format!("printf '%s' '{}'\n", first.stdout);
        let passed = flags(&f, &hardcoded);
        assert!(passed[0]);
        assert!(passed.iter().any(|p| !*p));
        let changed = format!("{}\nprintf 'changed\\n' > keep.txt\n", f.source);
        let passed = flags(&f, &changed);
        assert!(passed.iter().any(|p| *p) && passed.iter().any(|p| !*p));
    }
}

#[test]
fn recovery_stream_cannot_hide_eof_reset_baselines_status_or_final_file_failures() {
    let f = fixture("recovery");
    let wrong = [
        f.source.replace(" || test -n \"$line\"", ""),
        f.source.replace(
            "    previous=$line\ndone < observations.csv",
            "    previous=$previous\ndone < observations.csv",
        ),
        f.source.replace(
            "if test \"$warnings\" -gt 0; then exit 1; fi",
            "if test \"$warnings\" -gt 0; then :; fi",
        ),
        format!("{}\nprintf 'changed\\n' > keep.txt\n", f.source),
    ];
    for source in wrong {
        assert_ne!(source, f.source);
        let passed = flags(&f, &source);
        assert!(
            passed.iter().any(|p| *p) && passed.iter().any(|p| !*p),
            "Wrong recovery must be partly useful and fail specific cases: {passed:?}"
        );
    }
    let first = &f.cases[0];
    let report = &first.after["/work/report.txt"];
    assert!(!report.contains('\''));
    assert!(!first.stdout.contains('\''));
    let hardcoded = format!(
        "printf '%s' '{report}' > report.txt\nprintf '%s' '{}'\n",
        first.stdout
    );
    let passed = flags(&f, &hardcoded);
    assert!(passed[0]);
    assert!(passed.iter().any(|p| !*p));
}

#[test]
fn every_recovery_report_matches_the_independent_typed_interval_model() {
    fn parse(line: &str) -> Option<model::Snapshot> {
        let parts: Vec<_> = line.split(',').collect();
        if parts.len() != 12 {
            return None;
        }
        let boot = parts[0];
        if boot.is_empty()
            || boot.len() > 16
            || !boot.as_bytes()[0].is_ascii_lowercase()
            || !boot
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        {
            return None;
        }
        let mut n = Vec::new();
        for raw in &parts[1..] {
            if raw.is_empty()
                || raw.len() > 6
                || !raw.bytes().all(|b| b.is_ascii_digit())
                || (raw.len() > 1 && raw.starts_with('0'))
            {
                return None;
            }
            n.push(raw.parse::<u64>().ok()?);
        }
        let mut s = snapshot();
        s.boot = boot.into();
        s.at_ms = n[0];
        s.cpu.ticks = n[1..9].try_into().unwrap();
        s.network.rx_bytes = n[9];
        s.network.tx_bytes = n[10];
        Some(s)
    }
    for c in fixture("recovery").cases {
        let mut previous: Option<model::Snapshot> = None;
        let mut lines = vec!["Measurement report".to_string()];
        let mut errors = String::new();
        let (mut count, mut warnings) = (0, 0);
        for line in c.files["/work/observations.csv"].split_terminator('\n') {
            let Some(now) = parse(line) else {
                errors.push_str("Skipped malformed record\n");
                warnings += 1;
                previous = None;
                continue;
            };
            if let Some(before) = previous {
                match model::measure(&before, &now) {
                    Ok(w) => {
                        lines.push(format!(
                            "{}@{} cpu={}% rx={} B/s tx={} B/s",
                            now.boot,
                            now.at_ms,
                            w.cpu_work_share.scaled_floor(100).unwrap(),
                            w.rx_bytes_per_second.scaled_floor(1).unwrap(),
                            w.tx_bytes_per_second.scaled_floor(1).unwrap()
                        ));
                        count += 1;
                    }
                    Err(e) => {
                        let reason = match e {
                            model::Error::Identity => "Different boot",
                            model::Error::Time => "Time did not advance",
                            model::Error::Reset => "Counter decreased",
                            model::Error::NoCpuInterval => "CPU counters did not advance",
                            _ => panic!("{e:?}"),
                        };
                        errors.push_str(&format!("{reason} at {}; new baseline\n", now.at_ms));
                        warnings += 1;
                    }
                }
            }
            previous = Some(now);
        }
        let status = if count == 0 {
            errors.push_str("Not enough compatible observations\n");
            2
        } else if warnings > 0 {
            1
        } else {
            0
        };
        assert_eq!(c.status, status, "{}", c.name);
        assert_eq!(c.stderr, errors, "{}", c.name);
        assert_eq!(
            c.stdout,
            format!("Report: {count} valid windows, {warnings} rejected records or intervals\n")
        );
        lines.extend([
            format!("Valid windows: {count}"),
            format!("Warnings: {warnings}"),
        ]);
        assert_eq!(
            c.after["/work/report.txt"],
            lines.join("\n") + "\n",
            "{}",
            c.name
        );
        for (key, value) in &c.files {
            if key != "/work/report.txt" {
                assert_eq!(c.after.get(key), Some(value));
            }
        }
    }
}
