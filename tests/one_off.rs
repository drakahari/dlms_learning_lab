//! Prospective pure model tests. No real at, atd, spool, filesystem or shell.
use dlms_learning_lab::{
    cron::Moment,
    one_off::{self as at, ErrorKind, Run, State, Status},
};
use std::collections::{BTreeMap, BTreeSet};
fn moment(s: &str) -> Moment {
    Moment::parse(s).unwrap()
}
fn initial() -> State {
    State {
        version: 1,
        clock: moment("2028-02-28 23:58"),
        next_id: 1,
        users: BTreeSet::from(["root".into(), "alex".into(), "blair".into()]),
        files: BTreeMap::from([
            (
                "/work/job.sh".into(),
                "printf '%s\\n' \"$LABEL\"\ncat /work/input.txt\n".into(),
            ),
            ("/work/input.txt".into(), "Old sample input\n".into()),
        ]),
        jobs: BTreeMap::new(),
        records: vec![],
    }
}
fn env() -> BTreeMap<String, String> {
    BTreeMap::from([("LABEL".into(), "Captured label".into())])
}
fn finish(s: &State, end: Moment) -> State {
    let mut run = Run::begin(s, end).unwrap();
    for _ in 0..4096 {
        if run.done() {
            break;
        }
        run.tick(8);
    }
    assert!(run.done(), "bounded one-off run must finish");
    run.take_result().unwrap().unwrap()
}
#[test]
fn captures_script_environment_owner_but_reads_live_virtual_files_at_dispatch() {
    let mut s = initial();
    let mut supplied = env();
    let id = s
        .submit(
            "alex",
            "/work/job.sh",
            moment("2028-02-29 00:01"),
            &supplied,
        )
        .unwrap();
    supplied.insert("LABEL".into(), "Later environment".into());
    s.edit_file("/work/job.sh", "printf 'Changed script must not run\\n'\n")
        .unwrap();
    s.edit_file("/work/input.txt", "Current input at dispatch\n")
        .unwrap();
    let saved = s.clone();
    let result = finish(&s, moment("2028-02-29 00:02"));
    assert_eq!(s, saved);
    assert_eq!(result.jobs.len(), 0);
    assert_eq!(result.records[0].id, id);
    assert_eq!(result.records[0].owner, "alex");
    assert_eq!(
        result.records[0].stdout,
        "Captured label\nCurrent input at dispatch\n"
    );
    assert_eq!(result.records[0].status, Status::Completed);
    assert_eq!(result.records[0].exit_status, Some(0));
    assert_eq!(result.clock, moment("2028-02-29 00:02"));
}
#[test]
fn calendar_absolute_and_relative_forms_are_equivalent_and_validate_boundaries() {
    let s = initial();
    assert_eq!(
        at::parse_time(&["now", "+", "3", "minutes"], s.clock).unwrap(),
        moment("2028-02-29 00:01")
    );
    assert_eq!(
        at::parse_time(&["-t", "202802290001"], s.clock).unwrap(),
        moment("2028-02-29 00:01")
    );
    assert_eq!(
        at::parse_time(&["now", "+", "1", "day"], s.clock).unwrap(),
        moment("2028-02-29 23:58")
    );
    assert_eq!(
        at::parse_time(&["now", "+", "1", "week"], s.clock).unwrap(),
        moment("2028-03-06 23:58")
    );
    assert!(at::parse_time(&["-t", "202902290001"], s.clock).is_err());
    for bad in [
        vec!["now", "+", "8", "days"],
        vec!["now", "+", "99999", "weeks"],
        vec!["now", "+", "-1", "minute"],
        vec!["noon"],
        vec!["-t", "202802290001.30"],
        vec!["12:00", "UTC"],
    ] {
        assert!(at::parse_time(&bad, s.clock).is_err(), "{bad:?}");
    }
    assert!(at::parse_time(&["now", "+", "1", "minute"], moment("2099-12-31 23:59")).is_err());
    assert!(Run::begin(&s, s.clock).is_err());
    assert!(Run::begin(&s, moment("2028-03-07 00:00")).is_err());
}
#[test]
fn absolute_past_is_eligible_now_due_order_then_id_is_deterministic_and_effects_are_live() {
    let mut s = initial();
    for (file, source) in [
        ("a.sh", "printf 'A' >> /work/order.txt"),
        ("b.sh", "printf 'B' >> /work/order.txt"),
        ("c.sh", "printf 'C' >> /work/order.txt"),
    ] {
        s.edit_file(&format!("/work/{file}"), source).unwrap();
    }
    let b = s
        .submit(
            "alex",
            "/work/b.sh",
            moment("2028-02-29 00:01"),
            &BTreeMap::new(),
        )
        .unwrap();
    let c = s
        .submit(
            "blair",
            "/work/c.sh",
            moment("2028-02-29 00:01"),
            &BTreeMap::new(),
        )
        .unwrap();
    let a = s
        .submit(
            "alex",
            "/work/a.sh",
            moment("2028-01-01 00:00"),
            &BTreeMap::new(),
        )
        .unwrap();
    let result = finish(&s, moment("2028-02-29 00:02"));
    assert_eq!(
        result.records.iter().map(|r| r.id).collect::<Vec<_>>(),
        vec![a, b, c]
    );
    assert_eq!(result.files["/work/order.txt"], "ABC");
    assert_eq!(result.records[0].when, s.clock);
    assert_eq!(result.records[1].when, moment("2028-02-29 00:01"));
}
#[test]
fn queue_aliases_submission_flag_order_and_owner_visibility_preserve_real_jobs() {
    let mut a = initial();
    let mut b = initial();
    a.command("alex", "at -f /work/job.sh -t 202802290001", &env())
        .unwrap();
    b.command("alex", "at -t 202802290001 -f /work/job.sh", &env())
        .unwrap();
    assert_eq!(a, b);
    let mut c = initial();
    c.command("alex", "at -f /work/job.sh now + 3 minutes", &env())
        .unwrap();
    assert_eq!(a, c);
    a.command("blair", "at -f /work/job.sh now + 4 minutes", &env())
        .unwrap();
    let own = a.command("alex", "atq", &env()).unwrap();
    assert!(own.contains("alex"));
    assert!(!own.contains("blair"));
    assert_eq!(own, a.command("alex", "at -l", &env()).unwrap());
    let all = a.command("root", "atq", &env()).unwrap();
    assert!(all.contains("alex") && all.contains("blair"));
    let shown = a.command("alex", "at -c 1", &env()).unwrap();
    assert!(shown.contains("Simulator metadata"));
    assert!(shown.ends_with(&initial().files["/work/job.sh"]));
    assert_eq!(
        a.command("alex", "at -c 2", &env()).unwrap_err().kind,
        ErrorKind::Permission
    );
}
#[test]
fn cancellation_is_owner_checked_atomic_and_removed_jobs_never_execute() {
    let mut s = initial();
    let when = moment("2028-02-29 00:01");
    let a = s.submit("alex", "/work/job.sh", when, &env()).unwrap();
    let b = s.submit("blair", "/work/job.sh", when, &env()).unwrap();
    let before = s.clone();
    assert_eq!(
        s.remove("alex", &[a, b]).unwrap_err().kind,
        ErrorKind::Permission
    );
    assert_eq!(s, before);
    for ids in [vec![], vec![a, a], vec![a, 999]] {
        assert!(s.remove("root", &ids).is_err());
        assert_eq!(s, before);
    }
    s.command("alex", "at -r 1", &env()).unwrap();
    assert!(!s.jobs.contains_key(&a));
    assert!(s.jobs.contains_key(&b));
    assert_eq!(s.records[0].status, Status::Removed);
    assert_eq!(s.records[0].exit_status, None);
    let result = finish(&s, moment("2028-02-29 00:02"));
    assert_eq!(result.records.len(), 2);
    assert_eq!(result.records[1].id, b);
    assert_eq!(result.records[1].status, Status::Completed);
    assert_eq!(result.queued("root").unwrap().len(), 0);
}
#[test]
fn unsupported_or_malformed_script_is_queued_then_fails_before_any_script_effect() {
    for bad in [
        "printf 'not applied' > /work/wrong.txt\nwhile true; do echo x; done",
        "printf '%s\\n' \"$(cat /work/input.txt)\"",
        "cat /work/input.txt | cat",
        "echo 'unclosed",
        "A=hello\nprintf '%s' \"$A\"",
        "printf '%s' *",
    ] {
        let mut s = initial();
        s.edit_file("/work/bad.sh", bad).unwrap();
        s.submit("alex", "/work/bad.sh", s.clock, &env()).unwrap();
        let result = finish(&s, s.clock.next().unwrap());
        assert_eq!(result.records[0].status, Status::Failed, "{bad}");
        assert_eq!(result.records[0].exit_status, Some(2));
        assert!(result.records[0].error.is_some());
        assert_eq!(
            result.files, s.files,
            "unsupported syntax must fail before accepted-prefix effects"
        );
    }
}
#[test]
fn failed_utility_status_does_not_hide_later_jobs_or_invent_success() {
    let mut s = initial();
    s.edit_file("/work/missing.sh", "cat /work/absent.txt")
        .unwrap();
    s.submit("alex", "/work/missing.sh", s.clock, &env())
        .unwrap();
    s.submit("alex", "/work/job.sh", s.clock, &env()).unwrap();
    let result = finish(&s, s.clock.next().unwrap());
    assert_eq!(result.records.len(), 2);
    assert_eq!(result.records[0].status, Status::Failed);
    assert_eq!(result.records[0].exit_status, Some(1));
    assert!(result.records[0].stderr.contains("absent"));
    assert_eq!(result.records[0].error, None);
    assert_eq!(result.records[1].status, Status::Completed);
}
#[test]
fn later_success_in_same_script_sets_last_status_and_future_jobs_remain_pending() {
    let mut s = initial();
    s.edit_file(
        "/work/job.sh",
        "cat /work/absent.txt\nprintf 'continued\\n'",
    )
    .unwrap();
    let due = s.clock;
    s.submit("alex", "/work/job.sh", due, &env()).unwrap();
    let later = s
        .submit("blair", "/work/job.sh", due.after(1440).unwrap(), &env())
        .unwrap();
    let result = finish(&s, due.next().unwrap());
    assert_eq!(result.records.len(), 1);
    assert_eq!(result.records[0].stdout, "continued\n");
    assert!(!result.records[0].stderr.is_empty());
    assert_eq!(result.records[0].exit_status, Some(0));
    assert_eq!(result.records[0].status, Status::Completed);
    assert!(result.jobs.contains_key(&later));
    assert_eq!(result.queued("blair").unwrap().len(), 1);
}
#[test]
fn learner_paths_stay_virtual_and_thirty_three_commands_are_rejected_before_prefix_effects() {
    for path in ["/etc/passwd", "/work/../escape", "../../home"] {
        let mut s = initial();
        s.edit_file("/work/job.sh", &format!("cat {path}")).unwrap();
        s.submit("alex", "/work/job.sh", s.clock, &env()).unwrap();
        let result = finish(&s, s.clock.next().unwrap());
        assert_eq!(result.records[0].status, Status::Failed);
        assert_eq!(result.files, s.files);
        assert!(result.records[0].error.is_some() || !result.records[0].stderr.is_empty());
    }
    let mut s = initial();
    let source = "printf 'not committed' > /work/wrong.txt\n".repeat(33);
    s.edit_file("/work/job.sh", &source).unwrap();
    s.submit("alex", "/work/job.sh", s.clock, &env()).unwrap();
    let result = finish(&s, s.clock.next().unwrap());
    assert_eq!(result.files, s.files);
    assert!(
        result.records[0]
            .error
            .as_ref()
            .unwrap()
            .contains("at most 32")
    );
}
#[test]
fn printf_percent_and_quoted_expansion_use_the_same_vm_without_cron_preprocessing() {
    let mut s = initial();
    s.edit_file(
        "/work/format.sh",
        "# A portable example\nprintf '%s\\n' \"$LABEL\" > /work/report.txt\ncat /work/report.txt",
    )
    .unwrap();
    s.submit("alex", "/work/format.sh", s.clock, &env())
        .unwrap();
    let result = finish(&s, s.clock.next().unwrap());
    assert_eq!(result.files["/work/report.txt"], "Captured label\n");
    assert_eq!(result.records[0].stdout, "Captured label\n");
    assert_eq!(result.records[0].status, Status::Completed);
}
#[test]
fn stop_discards_only_this_simulated_transaction_and_never_publishes_partial_results() {
    let mut s = initial();
    s.edit_file(
        "/work/output.sh",
        "printf 'changed' > /work/output.txt\ncat /work/input.txt",
    )
    .unwrap();
    s.submit("alex", "/work/output.sh", s.clock, &env())
        .unwrap();
    let before = s.clone();
    let mut run = Run::begin(&s, s.clock.next().unwrap()).unwrap();
    run.tick(1);
    assert!(run.active_job().is_some());
    run.cancel();
    assert!(run.done());
    assert_eq!(
        run.take_result().unwrap().unwrap_err().kind,
        ErrorKind::Cancelled
    );
    for _ in 0..24 {
        run.tick(usize::MAX);
    }
    assert_eq!(s, before);
    assert!(run.take_result().is_none());
    let result = finish(&s, s.clock.next().unwrap());
    assert_eq!(result.files["/work/output.txt"], "changed");
}
#[test]
fn execution_and_file_limits_are_bounded_without_mutating_the_supplied_state() {
    let mut s = initial();
    s.edit_file("/work/large.sh", "cat /work/input.txt")
        .unwrap();
    s.edit_file("/work/input.txt", &"x".repeat(65536)).unwrap();
    s.submit("alex", "/work/large.sh", s.clock, &env()).unwrap();
    let result = finish(&s, s.clock.next().unwrap());
    assert_eq!(result.records[0].status, Status::Failed);
    assert!(result.records[0].error.is_some());
    let mut s = initial();
    s.edit_file("/work/job.sh", "printf 'new' > /work/new.txt")
        .unwrap();
    for i in 0..30 {
        s.edit_file(&format!("/work/f{i}"), "x").unwrap();
    }
    assert_eq!(s.files.len(), at::MAX_FILES);
    s.submit("alex", "/work/job.sh", s.clock, &env()).unwrap();
    let before = s.clone();
    let mut run = Run::begin(&s, s.clock.next().unwrap()).unwrap();
    for _ in 0..4096 {
        if run.done() {
            break;
        }
        let before = run.work_units;
        run.tick(usize::MAX);
        assert!(run.work_units - before <= at::MAX_TICK);
    }
    assert!(run.done());
    assert_eq!(
        run.take_result().unwrap().unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(s, before);
}
#[test]
fn invalid_paths_options_environments_time_and_queue_capacity_preserve_every_field() {
    let mut s = initial();
    let before = s.clone();
    for p in [
        "/tmp/job.sh",
        "/work/../escape",
        "/work/sub/job.sh",
        "/work/",
        "/work/a\n",
    ] {
        assert!(s.edit_file(p, "echo x").is_err());
        assert_eq!(s, before);
    }
    for command in [
        "at -f /work/job.sh noon",
        "at -f /work/job.sh -q b now",
        "at -f /work/job.sh now; cat /etc/passwd",
        "at -f /work/job.sh -t 202902290001",
        "at -f /work/job.sh -f /work/job.sh now",
        "batch",
        "atrm 1 1",
    ] {
        assert!(s.command("alex", command, &env()).is_err(), "{command}");
        assert_eq!(s, before);
    }
    for key in ["PATH", "UID", "OPTIND", "SHELL"] {
        let bad = BTreeMap::from([(key.into(), "x".into())]);
        assert!(s.submit("alex", "/work/job.sh", s.clock, &bad).is_err());
        assert_eq!(s, before);
    }
    for _ in 0..at::MAX_JOBS {
        s.submit("alex", "/work/job.sh", s.clock, &env()).unwrap();
    }
    let full = s.clone();
    assert_eq!(
        s.submit("alex", "/work/job.sh", s.clock, &env())
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(s, full);
}
#[test]
fn serialization_roundtrip_preserves_pending_capture_and_completed_outcomes_without_running() {
    let mut s = initial();
    s.submit("alex", "/work/job.sh", s.clock, &env()).unwrap();
    let bytes = serde_json::to_vec(&s).unwrap();
    assert_eq!(State::from_json(&bytes).unwrap(), s);
    assert!(State::from_json(&vec![b' '; at::MAX_WIRE + 1]).is_err());
    let out = finish(&s, s.clock.next().unwrap());
    let raw = serde_json::to_vec(&out).unwrap();
    assert_eq!(State::from_json(&raw).unwrap(), out);
    assert_eq!(serde_json::to_vec(&s).unwrap(), bytes);
    let mut invalid = serde_json::to_value(&s).unwrap();
    invalid["jobs"]["1"]["owner"] = "unknown".into();
    assert!(State::from_json(&serde_json::to_vec(&invalid).unwrap()).is_err());
}
#[test]
fn embedded_collections_reject_excess_or_duplicate_bodies_before_parsing_their_values() {
    let s = initial();
    let pairs = (0..32)
        .map(|i| format!("\"/work/f{i}\":\"x\""))
        .collect::<Vec<_>>()
        .join(",");
    let poisoned = format!("{{{pairs},\"/work/excess\":THIS_BODY_MUST_NOT_BE_READ}}");
    let mut value = serde_json::to_value(&s).unwrap();
    value["files"] = serde_json::json!({});
    let raw = serde_json::to_string(&value)
        .unwrap()
        .replace("\"files\":{}", &format!("\"files\":{poisoned}"));
    let e = serde_json::from_str::<State>(&raw).unwrap_err().to_string();
    assert!(e.contains("collection bound before excess body"), "{e}");
    let duplicate = "{\"/work/job.sh\":\"x\",\"/work/job.sh\":POISON}";
    let raw = serde_json::to_string(&value)
        .unwrap()
        .replace("\"files\":{}", &format!("\"files\":{duplicate}"));
    let e = serde_json::from_str::<State>(&raw).unwrap_err().to_string();
    assert!(e.contains("Duplicate one-off field before value"), "{e}");
    let raw = serde_json::to_string(&value)
        .unwrap()
        .replace("\"files\":{}", "\"files\":{\"/work/../escape\":POISON}");
    let e = serde_json::from_str::<State>(&raw).unwrap_err().to_string();
    assert!(e.contains("Invalid one-off field before value"), "{e}");
}

#[test]
fn ordinary_environment_identifiers_reject_early_and_keep_every_caller_field() {
    let mut s = initial();
    let before = s.clone();
    for key in ["", "1BAD", "A-B", "A B", "é", "A\nB"] {
        let invalid = BTreeMap::from([(key.into(), "value".into())]);
        assert!(
            s.submit("alex", "/work/job.sh", s.clock, &invalid).is_err(),
            "{key:?}"
        );
        assert_eq!(s, before);
    }
    s.submit("alex", "/work/job.sh", s.clock, &env()).unwrap();
    for key in ["", "1BAD", "A-B", "A B", "é", "A\nB"] {
        let mut invalid = s.clone();
        invalid.jobs.get_mut(&1).unwrap().environment =
            BTreeMap::from([(key.into(), "value".into())]);
        let original = invalid.clone();
        assert!(invalid.validate().is_err());
        assert_eq!(invalid, original);
        let mut value = serde_json::to_value(&s).unwrap();
        value["jobs"]["1"]["environment"] = serde_json::json!({});
        let key = serde_json::to_string(key).unwrap();
        let raw = serde_json::to_string(&value).unwrap().replace(
            "\"environment\":{}",
            &format!("\"environment\":{{{key}:POISON_MUST_NOT_BE_READ}}"),
        );
        let e = serde_json::from_str::<State>(&raw).unwrap_err().to_string();
        assert!(e.contains("Invalid one-off field before value"), "{e}");
    }
}

#[test]
fn ordinary_underscore_digit_rest_and_quoted_special_names_are_data_not_implicit_reads() {
    let mut s = initial();
    let source =
        "printf '%s\\n' \"${_NAME1}\" \"$A2\" '$OPTIND' '${OPTARG}' 'prefix-$OPTERR-suffix'";
    s.edit_file("/work/job.sh", source).unwrap();
    let ordinary = BTreeMap::from([
        ("_NAME1".into(), "Under score".into()),
        ("A2".into(), "$OPTIND".into()),
    ]);
    s.submit("alex", "/work/job.sh", s.clock, &ordinary)
        .unwrap();
    let raw = serde_json::to_vec(&s).unwrap();
    assert_eq!(State::from_json(&raw).unwrap(), s);
    let result = finish(&s, s.clock.next().unwrap());
    assert_eq!(result.records[0].status, Status::Completed);
    assert_eq!(
        result.records[0].stdout,
        "Under score\n$OPTIND\n$OPTIND\n${OPTARG}\nprefix-$OPTERR-suffix\n"
    );
}

#[test]
fn implicit_getopts_reads_are_rejected_in_all_forms_before_prefix_effects() {
    for key in ["OPTIND", "OPTARG", "OPTERR"] {
        for read in [
            format!("${key}"),
            format!("${{{key}}}"),
            format!("prefix-${{{key}}}-suffix"),
        ] {
            let mut s = initial();
            let source =
                format!("printf 'must not apply' > /work/prefix.txt\nprintf '%s' \"{read}\"");
            assert!(dlms_learning_lab::shell::one_off::validate(&source).is_err());
            s.edit_file("/work/job.sh", &source).unwrap();
            s.submit("alex", "/work/job.sh", s.clock, &env()).unwrap();
            let result = finish(&s, s.clock.next().unwrap());
            assert_eq!(result.records[0].status, Status::Failed);
            assert_eq!(result.records[0].exit_status, Some(2));
            assert!(result.records[0].error.is_some());
            assert_eq!(result.files, s.files);
        }
    }
    for body in [
        "printf '%s' \"$(printf '%s' \"$OPTIND\")\"",
        "printf '%s' \"${ITEM:-$OPTERR}\"",
        "printf '%s' \"${ITEM[0]}\"",
    ] {
        let mut s = initial();
        let source = format!("printf 'must not apply' > /work/prefix.txt\n{body}");
        assert!(dlms_learning_lab::shell::one_off::validate(&source).is_err());
        s.edit_file("/work/job.sh", &source).unwrap();
        s.submit("alex", "/work/job.sh", s.clock, &env()).unwrap();
        let result = finish(&s, s.clock.next().unwrap());
        assert_eq!(result.records[0].status, Status::Failed);
        assert_eq!(result.files, s.files);
    }
}

#[test]
fn unsupported_output_names_are_reported_separately_from_file_memory_limits() {
    for name in ["space name.txt", "é.txt"] {
        let mut s = initial();
        let source = format!("printf 'new' > '/work/{name}'");
        s.edit_file("/work/job.sh", &source).unwrap();
        s.submit("alex", "/work/job.sh", s.clock, &env()).unwrap();
        let before = s.clone();
        let mut run = Run::begin(&s, s.clock.next().unwrap()).unwrap();
        for _ in 0..4096 {
            if run.done() {
                break;
            }
            run.tick(8);
        }
        assert!(run.done());
        let error = run.take_result().unwrap().unwrap_err();
        assert_eq!(error.kind, ErrorKind::Unsupported);
        assert!(error.message.contains("unsupported name"));
        assert_eq!(s, before);
    }
}

#[test]
fn canonical_job_keys_survive_buffered_values_and_reject_bad_duplicate_and_excess_bodies() {
    let mut s = initial();
    s.submit("alex", "/work/job.sh", s.clock, &env()).unwrap();
    let raw = serde_json::to_vec(&s).unwrap();
    let value = serde_json::to_value(&s).unwrap();
    assert_eq!(State::from_json(&raw).unwrap(), s);
    assert_eq!(serde_json::from_value::<State>(value).unwrap(), s);
    let mut boundary = s.clone();
    let mut job = boundary.jobs.remove(&1).unwrap();
    job.id = at::MAX_ID;
    boundary.jobs.insert(at::MAX_ID, job);
    boundary.next_id = at::MAX_ID + 1;
    boundary.validate().unwrap();
    let wire = serde_json::to_vec(&boundary).unwrap();
    assert_eq!(State::from_json(&wire).unwrap(), boundary);
    assert_eq!(
        serde_json::from_value::<State>(serde_json::to_value(&boundary).unwrap()).unwrap(),
        boundary
    );
    let initial_wire = serde_json::to_string(&initial()).unwrap();
    for key in ["", "0", "01", "-1", "1x", "1000001", "4294967296", "Ω"] {
        let raw = initial_wire.replace(
            "\"jobs\":{}",
            &format!(
                "\"jobs\":{{{}:POISON_MUST_NOT_BE_READ",
                serde_json::to_string(key).unwrap()
            ),
        );
        let error = serde_json::from_str::<State>(&raw).unwrap_err().to_string();
        assert!(
            error.contains("Invalid/duplicate one-off job ID before value"),
            "{key}: {error}"
        );
    }
    let job = serde_json::to_string(&s.jobs[&1]).unwrap();
    let raw = initial_wire.replace(
        "\"jobs\":{}",
        &format!("\"jobs\":{{\"1\":{job},\"1\":POISON_MUST_NOT_BE_READ"),
    );
    assert!(
        serde_json::from_str::<State>(&raw)
            .unwrap_err()
            .to_string()
            .contains("Invalid/duplicate one-off job ID before value")
    );
    let raw = initial_wire.replace(
        "\"jobs\":{}",
        "\"jobs\":{\"11111111111\":POISON_MUST_NOT_BE_READ",
    );
    assert!(
        serde_json::from_str::<State>(&raw)
            .unwrap_err()
            .to_string()
            .contains("Scheduling UTF-8 text byte limit")
    );
    for _ in 1..at::MAX_JOBS {
        s.submit("alex", "/work/job.sh", s.clock, &env()).unwrap();
    }
    let members = s
        .jobs
        .iter()
        .map(|(id, job)| format!("\"{id}\":{}", serde_json::to_string(job).unwrap()))
        .collect::<Vec<_>>()
        .join(",");
    let raw = initial_wire.replace(
        "\"jobs\":{}",
        &format!("\"jobs\":{{{members},\"17\":POISON_MUST_NOT_BE_READ"),
    );
    assert!(
        serde_json::from_str::<State>(&raw)
            .unwrap_err()
            .to_string()
            .contains("One-off collection bound before excess body")
    );
}

#[test]
fn inspection_distinguishes_captured_values_from_current_environment_without_effects() {
    let mut state = initial();
    let captured = BTreeMap::from([
        ("EMPTY".into(), String::new()),
        (
            "LABEL".into(),
            "Queued label\n# Captured script follows unchanged\nnot a command \" \\".into(),
        ),
    ]);
    state
        .submit(
            "alex",
            "/work/job.sh",
            moment("2028-02-29 00:01"),
            &captured,
        )
        .unwrap();
    let before = state.clone();
    let current = BTreeMap::from([
        ("LABEL".into(), "Later value".into()),
        ("NEW".into(), "Current only".into()),
    ]);
    let shown = state.command("alex", "at -c 1", &current).unwrap();
    assert_eq!(
        state, before,
        "Inspecting never changes queued work or files"
    );
    assert!(shown.ends_with(&initial().files["/work/job.sh"]));
    let metadata = shown
        .lines()
        .find_map(|line| {
            line.strip_prefix("# Captured variables (JSON values, not shell commands): ")
        })
        .unwrap();
    let decoded: BTreeMap<String, String> = serde_json::from_str(metadata).unwrap();
    assert_eq!(decoded, captured);
    assert_eq!(decoded.get("EMPTY").map(String::as_str), Some(""));
    assert!(
        !decoded.contains_key("NEW"),
        "Absent and empty variables differ"
    );
    assert!(!shown.contains("Later value") && !shown.contains("Current only"));
    assert_eq!(
        shown
            .matches("\n# Captured script follows unchanged\n")
            .count(),
        1,
        "A newline in a variable cannot masquerade as a source boundary"
    );
    assert_eq!(
        state
            .command("blair", "at -c 1", &current)
            .unwrap_err()
            .kind,
        ErrorKind::Permission
    );
    assert_eq!(state, before);
    assert_eq!(state.command("root", "at -c 1", &current).unwrap(), shown);
    let mut no_variables = initial();
    no_variables
        .submit(
            "alex",
            "/work/job.sh",
            moment("2028-02-29 00:01"),
            &BTreeMap::new(),
        )
        .unwrap();
    assert!(
        no_variables
            .command("alex", "at -c 1", &current)
            .unwrap()
            .contains("# Captured variables (JSON values, not shell commands): {}\n")
    );
}
