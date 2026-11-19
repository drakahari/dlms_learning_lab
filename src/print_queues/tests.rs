//! State, parser and isolation regressions; authored curriculum/native acceptance
//! are separate gates.
use super::*;
fn fixture() -> (Facts, World) {
    let f = Facts {
        printers: vec![QueueName("Room".into()), QueueName("Labels".into())],
        documents: vec![
            Document {
                path: "/work/brief.txt".into(),
                title: "Room briefing".into(),
                text: "Room briefing\nSafety notice\n".into(),
                pages: 2,
            },
            Document {
                path: "/work/labels.txt".into(),
                title: "Stock labels".into(),
                text: "A-01\nB-02\n".into(),
                pages: 1,
            },
        ],
        operator: "learner".into(),
    };
    let w = World {
        printers: vec![
            Printer {
                name: f.printers[0].clone(),
                accepting: true,
                enabled: false,
                rejection_reason: String::new(),
                stopped_reason: "Paper replaced; awaiting operator".into(),
            },
            Printer {
                name: f.printers[1].clone(),
                accepting: true,
                enabled: true,
                rejection_reason: String::new(),
                stopped_reason: String::new(),
            },
        ],
        jobs: vec![],
        local_default: Some(f.printers[0].clone()),
        next_id: 81,
    };
    w.validate(&f).unwrap();
    (f, w)
}
#[test]
fn queue_acceptance_and_processing_are_independent() {
    let (f, mut w) = fixture();
    let id = w.submit(&f, None, "/work/brief.txt", 1, false).unwrap();
    assert_eq!(w.request(&id.to_string()).unwrap().state, JobState::Pending);
    w.set_accepting("Room", true, "").unwrap();
    assert_eq!(w.request(&id.to_string()).unwrap().state, JobState::Pending);
    w.set_enabled("rOoM", true, "").unwrap();
    assert_eq!(w.request("Room-81").unwrap().state, JobState::Completed);
    assert_eq!(w.completed_copies("/work/brief.txt", "ROOM"), 1);
    w.validate(&f).unwrap();
}
#[test]
fn rejection_does_not_spool_or_consume_a_number() {
    let (f, mut w) = fixture();
    w.set_accepting("Room", false, "Waiting for maintenance")
        .unwrap();
    let before = w.clone();
    let e = w.submit(&f, None, "/work/brief.txt", 1, false).unwrap_err();
    assert!(e.message.contains("rejecting"));
    assert_eq!(w, before);
    w.set_enabled("Room", true, "").unwrap();
    assert!(w.submit(&f, None, "/work/brief.txt", 1, false).is_err());
}
#[test]
fn pending_jobs_can_finish_while_new_requests_are_rejected() {
    let (f, mut w) = fixture();
    let id = w.submit(&f, None, "/work/brief.txt", 1, false).unwrap();
    w.set_accepting("Room", false, "Closed to new requests")
        .unwrap();
    w.set_enabled("Room", true, "").unwrap();
    assert_eq!(
        w.request(&id.to_string()).unwrap().state,
        JobState::Completed
    );
    assert!(!w.printer("Room").unwrap().accepting);
    assert!(w.submit(&f, None, "/work/brief.txt", 1, false).is_err());
}
#[test]
fn starting_a_printer_does_not_release_an_individually_held_request() {
    let (f, mut w) = fixture();
    let id = w.submit(&f, None, "/work/brief.txt", 2, true).unwrap();
    w.set_enabled("Room", true, "").unwrap();
    assert_eq!(w.request(&id.to_string()).unwrap().state, JobState::Held);
    assert_eq!(w.completed_copies("/work/brief.txt", "Room"), 0);
    w.hold(&id.to_string(), false).unwrap();
    assert_eq!(
        w.request(&id.to_string()).unwrap().state,
        JobState::Completed
    );
    assert_eq!(w.completed_copies("/work/brief.txt", "Room"), 2);
}
#[test]
fn releasing_a_job_is_not_enough_when_the_printer_is_stopped() {
    let (f, mut w) = fixture();
    let id = w.submit(&f, None, "/work/brief.txt", 1, true).unwrap();
    w.hold(&id.to_string(), false).unwrap();
    assert_eq!(w.request(&id.to_string()).unwrap().state, JobState::Pending);
    w.set_enabled("Room", true, "").unwrap();
    assert_eq!(
        w.request(&id.to_string()).unwrap().state,
        JobState::Completed
    );
}
#[test]
fn selecting_a_default_does_not_move_existing_jobs() {
    let (f, mut w) = fixture();
    let old = w.submit(&f, None, "/work/brief.txt", 1, false).unwrap();
    w.set_default("labels").unwrap();
    let new = w.submit(&f, None, "/work/labels.txt", 3, false).unwrap();
    assert_eq!(w.request(&old.to_string()).unwrap().printer.0, "Room");
    assert_eq!(w.request(&new.to_string()).unwrap().printer.0, "Labels");
    assert_eq!(w.completed_copies("/work/labels.txt", "Labels"), 3);
    assert_eq!(
        w.request(&old.to_string()).unwrap().state,
        JobState::Pending
    );
}
#[test]
fn canceling_one_request_preserves_others_and_does_not_print_it() {
    let (f, mut w) = fixture();
    let one = w.submit(&f, None, "/work/brief.txt", 1, true).unwrap();
    let two = w
        .submit(&f, Some("Labels"), "/work/labels.txt", 1, true)
        .unwrap();
    let preserved = w.request(&two.to_string()).unwrap().clone();
    w.cancel(&one.to_string()).unwrap();
    assert_eq!(
        w.request(&one.to_string()).unwrap().state,
        JobState::Canceled
    );
    assert_eq!(w.request(&two.to_string()).unwrap(), &preserved);
    w.set_enabled("Room", true, "").unwrap();
    assert_eq!(w.completed_copies("/work/brief.txt", "Room"), 0);
}
#[test]
fn wrong_names_paths_counts_or_finished_request_leave_state_unchanged() {
    let (f, mut w) = fixture();
    for (printer, path, copies) in [
        (Some("missing"), "/work/brief.txt", 1),
        (None, "/etc/passwd", 1),
        (None, "/work/../brief.txt", 1),
        (None, "/work/brief.txt", 0),
        (None, "/work/brief.txt", 21),
    ] {
        let before = w.clone();
        assert!(w.submit(&f, printer, path, copies, false).is_err());
        assert_eq!(w, before);
    }
    w.set_enabled("Room", true, "").unwrap();
    w.submit(&f, None, "/work/brief.txt", 1, false).unwrap();
    let before = w.clone();
    assert!(w.cancel("Room-81").is_err());
    assert!(w.hold("Room-81", true).is_err());
    assert_eq!(w, before);
}
#[test]
fn queue_names_are_case_insensitive_but_sources_cannot_collide() {
    let (mut f, w) = fixture();
    assert_eq!(w.printer("rOoM").unwrap().name.0, "Room");
    f.printers[1].0 = "room".into();
    assert!(f.validate().is_err());
}
#[test]
fn bounded_history_and_request_ids_refuse_rather_than_wrap() {
    let (f, mut w) = fixture();
    for _ in 0..64 {
        w.submit(&f, None, "/work/brief.txt", 1, true).unwrap();
    }
    w.validate(&f).unwrap();
    let before = w.clone();
    assert!(w.submit(&f, None, "/work/brief.txt", 1, true).is_err());
    assert_eq!(w, before);
    w.jobs.pop();
    w.next_id = 1_000_000;
    let before = w.clone();
    assert!(w.submit(&f, None, "/work/brief.txt", 1, true).is_err());
    assert_eq!(w, before);
}
#[test]
fn malformed_source_and_foreign_runtime_data_refuse() {
    let (f, w) = fixture();
    let mut bad = f.clone();
    bad.documents[0].pages = 0;
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.documents[0].path = "/work/../../outside".into();
    assert!(bad.validate().is_err());
    let mut bad = w.clone();
    bad.printers[0].name.0 = "foreign".into();
    assert!(bad.validate(&f).is_err());
    let mut bad = w.clone();
    bad.jobs.push(Job {
        id: bad.next_id,
        printer: QueueName("Room".into()),
        document: "/work/brief.txt".into(),
        copies: 1,
        state: JobState::Completed,
    });
    assert!(bad.validate(&f).is_err());
}

fn run(w: &mut World, f: &Facts, raw: &str) -> String {
    let out = command::run(w, f, raw).unwrap();
    assert_eq!(out.status, 0);
    out.stdout
}
#[test]
fn commands_derive_output_from_the_actual_queue_and_request_state() {
    let (f, mut w) = fixture();
    assert!(run(&mut w, &f, "lpstat -p Room").contains("disabled"));
    assert!(run(&mut w, &f, "lpstat -a Room").contains("accepting requests"));
    assert!(run(&mut w, &f, "lp brief.txt").contains("Room-81"));
    assert!(run(&mut w, &f, "lpstat -l -o Room").contains("Status: pending"));
    assert!(run(&mut w, &f, "cupsaccept Room").is_empty());
    assert!(run(&mut w, &f, "lpstat -l -o Room").contains("Status: pending"));
    run(&mut w, &f, "cupsenable Room");
    assert!(run(&mut w, &f, "lpstat -o Room").is_empty());
    assert!(run(&mut w, &f, "lpstat -l -W completed -o Room").contains("Status: completed"));
    assert_eq!(w.completed_copies("/work/brief.txt", "Room"), 1);
}
#[test]
fn literal_quotes_reordered_options_and_separate_requests_are_equivalent_outcomes() {
    let (f, mut one) = fixture();
    let mut two = one.clone();
    run(&mut one, &f, "lp -n 2 -d 'Room' -- /work/brief.txt");
    run(&mut two, &f, "lp -d Room -n 1 brief.txt");
    run(&mut two, &f, "lp brief.txt");
    for w in [&mut one, &mut two] {
        run(w, &f, "cupsenable room");
        assert_eq!(w.completed_copies("/work/brief.txt", "ROOM"), 2);
        assert_eq!(w.completed_copies("/work/brief.txt", "Labels"), 0);
        assert_eq!(w.printers[1], fixture().1.printers[1]);
    }
    assert_ne!(one.jobs.len(), two.jobs.len());
}
#[test]
fn command_hold_release_default_and_cancel_preserve_existing_identity_and_neighbors() {
    let (f, mut w) = fixture();
    run(&mut w, &f, "lp -H hold -d Room brief.txt");
    run(&mut w, &f, "lp -H hold -d Labels labels.txt");
    let neighbor = w.request("Labels-82").unwrap().clone();
    run(&mut w, &f, "lpoptions -d Labels");
    assert!(run(&mut w, &f, "lpstat -d").contains("Labels"));
    run(&mut w, &f, "cupsenable Room");
    assert_eq!(w.request("81").unwrap().state, JobState::Held);
    run(&mut w, &f, "lp -H resume -i Room-81");
    assert_eq!(w.request("81").unwrap().state, JobState::Completed);
    assert_eq!(w.request("82").unwrap(), &neighbor);
    run(&mut w, &f, "cancel Labels-82");
    assert_eq!(w.request("82").unwrap().state, JobState::Canceled);
    assert_eq!(w.request("81").unwrap().state, JobState::Completed);
}
#[test]
fn quoted_reasons_and_case_insensitive_names_describe_independent_controls() {
    let (f, mut w) = fixture();
    run(&mut w, &f, "cupsreject -r 'Changing paper stock' rOoM");
    let status = run(&mut w, &f, "lpstat -a room");
    assert!(status.contains("not accepting requests") && status.contains("Changing paper stock"));
    assert!(run(&mut w, &f, "lpstat -p ROOM").contains("disabled"));
    let before = w.clone();
    assert!(command::run(&mut w, &f, "lp brief.txt").is_err());
    assert_eq!(w, before);
    run(&mut w, &f, "cupsaccept Room");
    run(&mut w, &f, "cupsdisable -r \"Operator check\" Room");
    let status = run(&mut w, &f, "lpstat -t");
    assert!(status.contains("Operator check") && status.contains("accepting requests"));
}
#[test]
fn unsupported_shell_and_options_are_honest_and_transactional() {
    let (f, mut w) = fixture();
    for raw in [
        "lp brief.txt; cupsenable Room",
        "lp $(cat /etc/passwd)",
        "lp brief.txt | cat",
        "lp brief.txt > /work/result",
        "lp -h localhost brief.txt",
        "lp -o media=letter brief.txt",
        "lp -d Room brief.txt labels.txt",
        "lp -i 81 -d Labels -H resume",
        "lpstat -o -W completed",
        "lpadmin -p New -v socket://127.0.0.1",
        "sh -c 'lp brief.txt'",
    ] {
        let before = w.clone();
        let error = command::run(&mut w, &f, raw).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Unsupported, "{raw}: {error}");
        assert_eq!(w, before, "{raw}");
    }
}
#[test]
fn syntax_limit_and_state_errors_are_distinct_and_never_spool_partial_jobs() {
    let (f, mut w) = fixture();
    for (raw, kind) in [
        ("", ErrorKind::Syntax),
        ("lp 'brief.txt", ErrorKind::Syntax),
        ("lp -d", ErrorKind::Syntax),
        ("lp -n two brief.txt", ErrorKind::Syntax),
        ("lp -n 2 -n 3 brief.txt", ErrorKind::Syntax),
        ("lp -n 0 brief.txt", ErrorKind::Invalid),
        ("lp -d Missing brief.txt", ErrorKind::Invalid),
        ("lp /etc/passwd", ErrorKind::Invalid),
        ("lp ../outside.txt", ErrorKind::Syntax),
        ("cancel 999", ErrorKind::Invalid),
    ] {
        let before = w.clone();
        assert_eq!(
            command::run(&mut w, &f, raw).unwrap_err().kind,
            kind,
            "{raw}"
        );
        assert_eq!(w, before);
    }
    let before = w.clone();
    assert_eq!(
        command::run(&mut w, &f, &"x".repeat(1025))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(w, before);
}
#[test]
fn virtual_file_queries_use_supplied_bytes_and_refuse_host_paths() {
    let (f, mut w) = fixture();
    assert_eq!(run(&mut w, &f, "cat brief.txt"), f.documents[0].text);
    let list = run(&mut w, &f, "ls /work");
    assert!(list.contains("brief.txt") && list.contains("labels.txt"));
    for raw in [
        "cat /proc/self/environ",
        "cat /home/learner/private.txt",
        "cat /work/../brief.txt",
    ] {
        let before = w.clone();
        assert!(command::run(&mut w, &f, raw).is_err());
        assert_eq!(w, before);
    }
}
#[test]
fn strict_sources_and_runtime_bounds_apply_before_even_a_readonly_command() {
    let (f, w) = fixture();
    let mut bad = w.clone();
    bad.printers[0].name.0 = "Foreign".into();
    let before = bad.clone();
    assert!(command::run(&mut bad, &f, "lpstat -t").is_err());
    assert_eq!(bad, before);
    let mut wire = serde_json::to_value(&f).unwrap();
    wire["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Facts>(wire).is_err());
    let mut wire = serde_json::to_value(&w).unwrap();
    wire["printers"][0]["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<World>(wire).is_err());
    let mut wire = serde_json::to_value(&f).unwrap();
    wire["documents"][0]["text"] = serde_json::json!("x".repeat(16385));
    assert!(serde_json::from_value::<Facts>(wire).is_err());
}

#[test]
fn normalized_job_listing_uses_kib_spool_metadata_not_exact_raw_text_length() {
    let (f, mut w) = fixture();
    run(&mut w, &f, "lp brief.txt");
    let listing = run(&mut w, &f, "lpstat -o");
    assert!(listing.split_whitespace().any(|word| word == "1024"));
    assert_ne!(f.documents[0].text.len(), 1024);
    assert!(command::HELP.contains("unfiltered text"));
}

#[test]
fn bsd_submission_and_system_v_submission_reach_identical_typed_requests() {
    let (f, mut lp_world) = fixture();
    let mut bsd_world = lp_world.clone();
    run(&mut lp_world, &f, "lp -d Room -n 2 brief.txt");
    assert!(run(&mut bsd_world, &f, "lpr -PRoom -#2 brief.txt").is_empty());
    assert_eq!(lp_world, bsd_world);
    run(&mut lp_world, &f, "cupsenable Room");
    run(&mut bsd_world, &f, "cupsenable Room");
    assert_eq!(lp_world, bsd_world);
    assert_eq!(bsd_world.completed_copies("/work/brief.txt", "Room"), 2);
}
#[test]
fn lpr_separate_quoted_hash_option_and_job_hold_are_real_transitions() {
    let (f, mut w) = fixture();
    run(&mut w, &f, "cupsenable Room");
    run(&mut w, &f, "lpr -P 'Room' -# 2 -q /work/brief.txt");
    assert_eq!(w.request("81").unwrap().state, JobState::Held);
    assert_eq!(w.request("81").unwrap().copies, 2);
    run(&mut w, &f, "lpr '-#' 1 -q brief.txt");
    assert_eq!(w.jobs.len(), 2);
    run(&mut w, &f, "lp -i 81 -H resume");
    assert_eq!(w.completed_copies("/work/brief.txt", "Room"), 2);
    assert_eq!(w.request("82").unwrap().state, JobState::Held);
}
#[test]
fn lpq_lists_actual_active_jobs_and_readiness_without_claiming_a_hold_was_printed() {
    let (f, mut w) = fixture();
    run(&mut w, &f, "lpr -q -#2 brief.txt");
    let before = w.clone();
    let ordinary = run(&mut w, &f, "lpq -PRoom");
    assert!(ordinary.contains("Room is not ready"));
    assert!(ordinary.contains("Rank") && ordinary.contains("81") && ordinary.contains("brief.txt"));
    assert_eq!(w, before);
    let long = run(&mut w, &f, "lpq -l -P Room");
    assert!(long.contains("2 copies of brief.txt") && long.contains("1024 bytes"));
    run(&mut w, &f, "cupsenable Room");
    assert!(run(&mut w, &f, "lpq").contains("Room is ready"));
    assert!(run(&mut w, &f, "lpq").contains("81"));
    assert_eq!(w.request("81").unwrap().state, JobState::Held);
    run(&mut w, &f, "lp -i 81 -H resume");
    assert!(run(&mut w, &f, "lpq").contains("no entries"));
}
#[test]
fn explicit_lprm_cancels_only_the_identified_active_request() {
    let (f, mut w) = fixture();
    run(&mut w, &f, "lpr -q brief.txt");
    run(&mut w, &f, "lpr -P Labels -q labels.txt");
    let protected = w.request("82").unwrap().clone();
    assert!(run(&mut w, &f, "lprm 81").is_empty());
    assert_eq!(w.request("81").unwrap().state, JobState::Canceled);
    assert_eq!(w.request("82").unwrap(), &protected);
    assert!(run(&mut w, &f, "lpq -P Room").contains("no entries"));
}
#[test]
fn hash_option_is_opt_in_and_never_becomes_shell_comment_or_execution() {
    assert_eq!(
        literal::words("lpr -#2 brief.txt").unwrap(),
        ["lpr", "-#2", "brief.txt"]
    );
    assert_eq!(
        literal::words("lpr '-#' 2 brief.txt").unwrap(),
        ["lpr", "-#", "2", "brief.txt"]
    );
    assert_eq!(
        literal::words("lpr file#part").unwrap(),
        ["lpr", "file#part"]
    );
    assert!(literal::words("lpr # comment").is_err());
    assert!(
        crate::bios_loader::words("lpr -#2 brief.txt").is_err(),
        "Other literal adapters retain their original scope."
    );
    let (f, mut w) = fixture();
    for raw in [
        "lpr -#2 brief.txt; lp labels.txt",
        "lpr -#2 $(cat /etc/passwd)",
        "lpr -#2 brief.txt | cat",
        "lpr -#2 brief.txt\nsh script",
    ] {
        let before = w.clone();
        assert!(command::run(&mut w, &f, raw).is_err());
        assert_eq!(w, before);
    }
}
#[test]
fn bsd_unknown_options_nonfinite_forms_bad_inputs_and_paths_refuse_atomically() {
    let (f, mut w) = fixture();
    for raw in [
        "lpr -r brief.txt",
        "lpr -H server brief.txt",
        "lpr -o media=letter brief.txt",
        "lpr -# 0 brief.txt",
        "lpr -# 21 brief.txt",
        "lpr -# two brief.txt",
        "lpr -#2 -#3 brief.txt",
        "lpr -P Room -P Labels brief.txt",
        "lpr /etc/private",
        "lpr",
        "lpq +1",
        "lpq -h server",
        "lpq -P Missing",
        "lprm",
        "lprm -",
        "lprm 81 82",
        "lprm Room-81",
    ] {
        let before = w.clone();
        assert!(command::run(&mut w, &f, raw).is_err(), "{raw}");
        assert_eq!(w, before, "{raw}");
    }
}

#[test]
fn long_systemv_status_and_bsd_document_listing_have_distinct_supported_fields() {
    let (f, mut w) = fixture();
    command::run(&mut w, &f, "lp -n 2 brief.txt").unwrap();
    let before = w.clone();
    let status = command::run(&mut w, &f, "lpstat -l -o Room")
        .unwrap()
        .stdout;
    assert!(status.contains("Room-81") && status.contains("1024"));
    assert!(status.contains("Status: pending") && status.contains("queued for Room"));
    assert!(!status.contains("Document:") && !status.contains("Copies:"));
    let documents = command::run(&mut w, &f, "lpq -l -P Room").unwrap().stdout;
    assert!(documents.contains("2 copies of brief.txt") && documents.contains("job 81"));
    assert_eq!(w, before);
}
