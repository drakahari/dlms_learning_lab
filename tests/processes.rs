use dlms_learning_lab::processes::{
    Activity, ExpectedPhase, Identity, Phase, Process, Requirement, Table,
};
use std::collections::BTreeMap;
fn identity(name: &str, uid: u32) -> Identity {
    Identity {
        name: name.into(),
        uid,
    }
}
fn active(activity: Activity) -> Phase {
    Phase::Active { activity }
}
fn fixture() -> Table {
    let root = identity("root", 0);
    let alex = identity("alex", 1000);
    let blair = identity("blair", 1001);
    let process = |parent, owner: Identity, session, comm: &str, nice, phase| Process {
        parent,
        owner,
        session,
        comm: comm.into(),
        nice,
        phase,
    };
    Table {
        user: alex.clone(),
        session: 200,
        processes: BTreeMap::from([
            (
                1,
                process(0, root.clone(), 1, "init", 0, active(Activity::Sleeping)),
            ),
            (
                200,
                process(1, alex.clone(), 200, "bash", 0, active(Activity::Sleeping)),
            ),
            (
                201,
                process(
                    200,
                    alex.clone(),
                    200,
                    "worker",
                    0,
                    active(Activity::Running),
                ),
            ),
            (
                202,
                process(201, alex, 200, "worker", 5, active(Activity::Sleeping)),
            ),
            (
                300,
                process(1, root, 300, "daemon", 0, active(Activity::Sleeping)),
            ),
            (
                400,
                process(
                    1,
                    blair.clone(),
                    400,
                    "worker",
                    0,
                    active(Activity::Running),
                ),
            ),
            (
                401,
                process(1, blair, 200, "helper", 0, active(Activity::Sleeping)),
            ),
        ]),
    }
}
#[test]
fn formatted_selection_reports_current_virtual_values_in_requested_order() {
    let mut t = fixture();
    assert!(t.valid());
    let r = t
        .execute("ps -p 201,202 -o pid,ppid,user,stat,ni,comm")
        .unwrap();
    assert_eq!(r.status, 0);
    assert_eq!(r.stderr, "");
    let rows: Vec<Vec<_>> = r
        .stdout
        .lines()
        .map(|l| l.split_whitespace().collect())
        .collect();
    assert_eq!(
        rows,
        vec![
            vec!["PID", "PPID", "USER", "STAT", "NI", "COMMAND"],
            vec!["201", "200", "alex", "R", "0", "worker"],
            vec!["202", "201", "alex", "SN", "5", "worker"]
        ]
    );
    assert_eq!(
        t.execute("ps -eo pid,comm").unwrap(),
        t.execute("ps -e -o pid,comm").unwrap()
    );
    assert_eq!(
        t.execute("ps -p 300 -o comm,user,pid").unwrap().stdout,
        "COMMAND USER PID\ndaemon  root 300\n"
    );
    assert_eq!(t.execute("ps -p 999 -o pid,comm").unwrap().status, 1);
    assert_eq!(
        t.execute("show").unwrap(),
        t.execute("ps -eo pid,ppid,user,stat,ni,comm").unwrap()
    );
}
#[test]
fn pgrep_exact_names_owner_filters_and_no_matches_are_state_driven() {
    let mut t = fixture();
    assert_eq!(
        t.execute("pgrep -x worker").unwrap().stdout,
        "201\n202\n400\n"
    );
    assert_eq!(
        t.execute("pgrep -x worker -u alex").unwrap().stdout,
        "201\n202\n"
    );
    assert_eq!(
        t.execute("pgrep -u alex -x worker").unwrap().stdout,
        "201\n202\n"
    );
    assert_eq!(t.execute("pgrep -x missing").unwrap().status, 1);
    t.execute("kill 201").unwrap();
    assert_eq!(
        t.execute("pgrep -u alex -x worker").unwrap().stdout,
        "202\n"
    );
    for command in [
        "pgrep worker",
        "pgrep -x worker.v1",
        "pgrep -x work.*",
        "pgrep -f -x worker",
        "pgrep -u unknown -x worker",
    ] {
        let before = t.clone();
        assert!(t.execute(command).is_err(), "{command}");
        assert_eq!(t, before);
    }
}
#[test]
fn stop_continue_and_pending_termination_retain_typed_activity() {
    for (id, resume) in [(201, Activity::Running), (202, Activity::Sleeping)] {
        let mut t = fixture();
        assert_eq!(t.execute(&format!("kill -STOP {id}")).unwrap().status, 0);
        assert_eq!(
            t.processes[&id].phase,
            Phase::Stopped {
                resume,
                pending_termination: false
            }
        );
        t.execute(&format!("kill -STOP {id}")).unwrap();
        t.execute(&format!("kill -CONT {id}")).unwrap();
        assert_eq!(t.processes[&id].phase, active(resume));
        t.execute(&format!("kill -STOP {id}")).unwrap();
        t.execute(&format!("kill -TERM {id}")).unwrap();
        assert_eq!(
            t.processes[&id].phase,
            Phase::Stopped {
                resume,
                pending_termination: true
            }
        );
        assert!(
            t.execute(&format!("ps -p {id} -o stat"))
                .unwrap()
                .stdout
                .contains('T')
        );
        t.execute(&format!("kill -INT {id}")).unwrap();
        t.execute(&format!("kill -STOP {id}")).unwrap();
        assert!(matches!(
            t.processes[&id].phase,
            Phase::Stopped {
                pending_termination: true,
                ..
            }
        ));
        t.execute(&format!("kill -CONT {id}")).unwrap();
        assert_eq!(t.processes[&id].phase, Phase::Exited);
        assert_eq!(t.execute(&format!("ps -p {id} -o pid")).unwrap().status, 1);
    }
}
#[test]
fn default_named_and_numeric_termination_are_equivalent_and_kill_is_immediate() {
    let mut expected = fixture();
    expected.execute("kill 201").unwrap();
    for command in [
        "kill -TERM 201",
        "kill -15 201",
        "kill -s SIGTERM 201",
        "kill -s term -- 201",
    ] {
        let mut t = fixture();
        let r = t.execute(command).unwrap();
        assert_eq!(r.status, 0);
        assert_eq!(t, expected, "{command}");
    }
    for signal in ["-INT", "-2", "-KILL", "-9"] {
        let mut t = fixture();
        t.execute(&format!("kill {signal} 201")).unwrap();
        assert_eq!(t, expected);
    }
    let mut t = fixture();
    t.execute("kill -STOP 201").unwrap();
    t.execute("kill -KILL 201").unwrap();
    assert_eq!(t, expected);
}
#[test]
fn permissions_probe_and_same_session_continue_match_the_disclosed_identity_model() {
    let mut t = fixture();
    let initial = t.clone();
    assert_eq!(t.execute("kill -0 201").unwrap().status, 0);
    assert_eq!(t, initial);
    assert_eq!(t.execute("kill -0 300").unwrap().status, 1);
    assert_eq!(t, initial);
    assert_eq!(t.execute("kill -STOP 401").unwrap().status, 1);
    assert_eq!(t, initial);
    t.processes.get_mut(&401).unwrap().phase = Phase::Stopped {
        resume: Activity::Sleeping,
        pending_termination: false,
    };
    assert_eq!(t.execute("kill -CONT 401").unwrap().status, 0);
    assert_eq!(t.processes[&401].phase, active(Activity::Sleeping));
    assert_eq!(t.execute("kill -CONT 400").unwrap().status, 1);
    t.user = identity("root", 0);
    t.session = 1;
    assert_eq!(t.execute("kill -0 1").unwrap().status, 0);
    let before = t.clone();
    assert_eq!(t.execute("kill -9 1").unwrap().status, 1);
    assert_eq!(t, before);
    assert_eq!(t.execute("kill -TERM 400").unwrap().status, 0);
    assert_eq!(t.processes[&400].phase, Phase::Exited);
}
#[test]
fn malformed_commands_are_atomic_while_runtime_target_failures_allow_other_changes() {
    let mut t = fixture();
    for command in [
        "kill 201 nope",
        "kill -s",
        "kill -s TERM",
        "kill -s SIG0 201",
        "kill -s SIG15 201",
        "kill -SIG9 201 202",
        "kill -s -- 201",
        "kill -- -1",
        "kill 0",
        "kill -USR1 201",
        "kill -015 201",
        "kill -l",
        "kill -- 201; touch secret",
        "kill 201\nprintf leaked",
        "kill $(id)",
        "kill `id`",
        "kill /proc/201",
        "kill '201'",
        "kill 201 | cat",
        "renice -n 10 -p 201",
        "renice --relative 10 -p 201",
        "renice 99 -p 201",
        "renice 10 -p 201 invalid",
        "ps",
        "ps -ef",
        "ps -p 201 -o pid,args",
        "ps -eo pid=ID",
        "ps -eo pid,,comm",
    ] {
        let before = t.clone();
        assert!(t.execute(command).is_err(), "{command}");
        assert_eq!(t, before, "{command}");
    }
    let r = t.execute("kill 201 999 400 202").unwrap();
    assert_eq!(r.status, 1);
    assert_eq!(r.stdout, "");
    assert!(r.stderr.contains("(999): No such process"));
    assert!(r.stderr.contains("(400): Operation not permitted"));
    assert_eq!(t.processes[&201].phase, Phase::Exited);
    assert_eq!(t.processes[&202].phase, Phase::Exited);
    assert_eq!(t.processes[&400].phase, active(Activity::Running));
    assert!(t.valid());
}
#[test]
fn exited_parents_are_projected_as_init_without_mutating_original_process_metadata() {
    let mut t = fixture();
    let initial = t.clone();
    t.execute("kill 201").unwrap();
    assert_eq!(
        t.execute("ps -p 202 -o pid,ppid").unwrap().stdout,
        "PID PPID\n202    1\n"
    );
    assert_eq!(t.processes[&202].parent, 201);
    assert!(t.same_problem(&initial));
    assert_eq!(t.execute("kill 201").unwrap().status, 1);
    assert_eq!(t.processes.len(), initial.processes.len());
}
#[test]
fn niceness_direction_permissions_and_partial_errors_are_honest() {
    let mut t = fixture();
    let r = t.execute("renice --priority 10 -p 201 999 400").unwrap();
    assert_eq!(r.status, 1);
    assert!(
        r.stdout
            .contains("201 (process ID) old priority 0, new priority 10")
    );
    assert!(r.stderr.contains("No such process"));
    assert!(r.stderr.contains("Permission denied"));
    assert_eq!(t.processes[&201].nice, 10);
    assert_eq!(t.processes[&400].nice, 0);
    let before = t.clone();
    assert_eq!(t.execute("renice 0 201").unwrap().status, 1);
    assert_eq!(t, before);
    assert_eq!(t.execute("renice 10 201").unwrap().status, 0);
    assert!(
        t.execute("ps -p 201 -o stat,ni")
            .unwrap()
            .stdout
            .contains("RN")
    );
    t.user = identity("root", 0);
    t.session = 1;
    assert_eq!(t.execute("renice --priority -20 -p 201").unwrap().status, 0);
    assert_eq!(t.processes[&201].nice, -20);
    assert!(
        t.execute("ps -p 201 -o stat")
            .unwrap()
            .stdout
            .contains("R<")
    );
    let before = t.clone();
    assert_eq!(t.execute("renice 1 -p 1").unwrap().status, 1);
    assert_eq!(t, before);
    t.execute("kill 201").unwrap();
    assert_eq!(t.execute("renice 19 -p 201").unwrap().status, 1);
}
#[test]
fn bounds_and_serde_reject_cycles_corrupted_identities_foreign_fields_and_large_commands() {
    let mut t = fixture();
    let base = t.processes[&201].clone();
    for id in 1000..1057 {
        t.processes.insert(
            id,
            Process {
                parent: 1,
                ..base.clone()
            },
        );
    }
    assert_eq!(t.processes.len(), 64);
    assert!(t.valid());
    assert!(
        t.execute(&format!("kill {}", vec!["999999"; 127].join(" ")))
            .unwrap()
            .stderr
            .len()
            < 16_384
    );
    let before = t.clone();
    assert!(t.execute(&"x".repeat(4097)).is_err());
    assert_eq!(t, before);
    assert!(
        t.execute(&format!("kill {}", vec!["201"; 128].join(" ")))
            .is_err()
    );
    assert_eq!(t, before);
    t.processes.insert(1057, base.clone());
    assert!(!t.valid());
    for mutate in [0, 1, 2, 3, 4, 5] {
        let mut t = fixture();
        match mutate {
            0 => t.processes.get_mut(&201).unwrap().parent = 202,
            1 => t.processes.get_mut(&201).unwrap().owner.uid = 0,
            2 => t.processes.get_mut(&201).unwrap().nice = 20,
            3 => t.processes.get_mut(&201).unwrap().comm = "x".repeat(16),
            4 => t.processes.get_mut(&1).unwrap().phase = Phase::Exited,
            _ => t.processes.get_mut(&201).unwrap().session = 0,
        }
        assert!(!t.valid(), "{mutate}");
        assert!(t.execute("show").is_err());
    }
    let t = fixture();
    let bytes = serde_json::to_vec(&t).unwrap();
    assert_eq!(serde_json::from_slice::<Table>(&bytes).unwrap(), t);
    let mut json = serde_json::to_value(&t).unwrap();
    json["host_command"] = serde_json::json!("ignored");
    assert!(serde_json::from_value::<Table>(json).is_err());
}
#[test]
fn imported_shape_is_pinned_and_current_requirements_cannot_reuse_old_success() {
    let initial = fixture();
    let mut t = initial.clone();
    let stopped = Requirement::Phase {
        pid: 201,
        expected: ExpectedPhase::Stopped,
    };
    let priority = Requirement::Niceness {
        pid: 202,
        value: 10,
    };
    let protected = Requirement::Unchanged {
        pid: 300,
        expected: t.processes[&300].clone(),
    };
    for r in [&stopped, &priority, &protected] {
        assert!(r.valid(&initial));
    }
    assert!(!stopped.check(&t).0);
    assert!(!priority.check(&t).0);
    assert!(protected.check(&t).0);
    t.execute("kill -STOP 201").unwrap();
    t.execute("renice 10 -p 202").unwrap();
    assert!(stopped.check(&t).0 && priority.check(&t).0 && protected.check(&t).0);
    assert!(t.same_problem(&initial));
    t.execute("kill -CONT 201").unwrap();
    assert!(!stopped.check(&t).0);
    t.execute("kill 202").unwrap();
    assert!(!priority.check(&t).0);
    for change in [0, 1, 2, 3, 4] {
        let mut foreign = initial.clone();
        match change {
            0 => foreign.processes.get_mut(&201).unwrap().comm = "renamed".into(),
            1 => foreign.processes.get_mut(&201).unwrap().parent = 1,
            2 => foreign.processes.remove(&201).map(|_| ()).unwrap(),
            3 => foreign.session = 400,
            _ => foreign.user = identity("root", 0),
        }
        assert!(!foreign.same_problem(&initial), "{change}");
    }
    assert!(!Requirement::Niceness { pid: 999, value: 0 }.valid(&initial));
    assert!(
        !Requirement::Niceness {
            pid: 201,
            value: 30
        }
        .valid(&initial)
    );
}

#[test]
fn duplicate_and_oversized_serialized_process_tables_are_rejected_before_overwriting_entries() {
    let t = fixture();
    let text = serde_json::to_string(&t).unwrap();
    let duplicate = text.replacen(
        "\"processes\":{",
        &format!(
            "\"processes\":{{\"201\":{},",
            serde_json::to_string(&t.processes[&201]).unwrap()
        ),
        1,
    );
    assert!(serde_json::from_str::<Table>(&duplicate).is_err());
    let mut many = t.clone();
    for id in 1000..1058 {
        many.processes.insert(
            id,
            Process {
                parent: 1,
                ..t.processes[&201].clone()
            },
        );
    }
    assert_eq!(many.processes.len(), 65);
    assert!(serde_json::from_str::<Table>(&serde_json::to_string(&many).unwrap()).is_err());
    let bad = text.replacen("\"1\":", "\"0\":", 1);
    assert!(serde_json::from_str::<Table>(&bad).is_err());
}
