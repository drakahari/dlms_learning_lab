//! Bounded supplied listener metadata: real model checks, no host networking.
use dlms_learning_lab::{
    engine,
    processes::{Activity, Identity, Listener, Phase, Protocol, Requirement, Table},
};
use std::net::Ipv4Addr;
const OLD_JSON: &str = r###"{"user":{"name":"alex","uid":1000},"session":100,"processes":{"1":{"parent":0,"owner":{"name":"root","uid":0},"session":1,"comm":"init","nice":0,"phase":{"kind":"Active","activity":"Sleeping"}},"100":{"parent":1,"owner":{"name":"alex","uid":1000},"session":100,"comm":"controller","nice":0,"phase":{"kind":"Active","activity":"Sleeping"}},"101":{"parent":100,"owner":{"name":"alex","uid":1000},"session":100,"comm":"worker","nice":0,"phase":{"kind":"Active","activity":"Running"}},"201":{"parent":1,"owner":{"name":"blair","uid":1001},"session":200,"comm":"worker","nice":0,"phase":{"kind":"Active","activity":"Running"}},"300":{"parent":1,"owner":{"name":"root","uid":0},"session":300,"comm":"daemon","nice":0,"phase":{"kind":"Active","activity":"Sleeping"}}}}"###;
fn old() -> Table {
    serde_json::from_str(OLD_JSON).unwrap()
}
fn fixture() -> Table {
    let mut table = old();
    table.processes.get_mut(&101).unwrap().comm = "previewd".into();
    let mut local = table.processes[&101].clone();
    local.comm = "previewd".into();
    table.processes.insert(102, local);
    table.listeners = vec![
        Listener {
            protocol: Protocol::Tcp,
            address: Ipv4Addr::UNSPECIFIED,
            port: 8081,
            pid: 101,
            fd: 3,
            backlog: 128,
        },
        Listener {
            protocol: Protocol::Tcp,
            address: Ipv4Addr::LOCALHOST,
            port: 8082,
            pid: 102,
            fd: 4,
            backlog: 64,
        },
        Listener {
            protocol: Protocol::Tcp,
            address: "192.0.2.2".parse().unwrap(),
            port: 443,
            pid: 201,
            fd: 3,
            backlog: 128,
        },
        Listener {
            protocol: Protocol::Udp,
            address: Ipv4Addr::LOCALHOST,
            port: 5353,
            pid: 300,
            fd: 5,
            backlog: 0,
        },
    ];
    assert!(table.valid());
    table
}
fn closed(t: &Table) -> Requirement {
    Requirement::Listener {
        endpoint: t.listeners[0].clone(),
        present: false,
    }
}

#[test]
fn old_table_wire_bytes_do_not_gain_a_listener_field() {
    let t = old();
    assert!(t.valid());
    assert!(t.listeners.is_empty());
    assert_eq!(serde_json::to_string(&t).unwrap(), OLD_JSON);
    let current: Table = serde_json::from_str(&serde_json::to_string(&t).unwrap()).unwrap();
    assert_eq!(current, t);
    assert!(current.same_problem(&t));
    assert!(
        !serde_json::to_value(t)
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("listeners")
    );
}
#[test]
fn ss_tcp_udp_numeric_projection_is_read_only_and_equivalent_flags_agree() {
    let mut t = fixture();
    let before = t.clone();
    let all = t.execute("ss -lntup").unwrap();
    assert_eq!(all.status, 0);
    assert!(all.stderr.is_empty());
    assert_eq!(all, t.execute("ss -p -u -t -n -l").unwrap());
    assert_eq!(all, t.execute("ss -utpnl").unwrap());
    assert_eq!(all.stdout.lines().count(), 5);
    let first = all
        .stdout
        .lines()
        .find(|s| s.contains("0.0.0.0:8081"))
        .unwrap();
    assert_eq!(
        first.split_whitespace().collect::<Vec<_>>(),
        vec![
            "tcp",
            "LISTEN",
            "0",
            "128",
            "0.0.0.0:8081",
            "0.0.0.0:*",
            "users:((\"previewd\",pid=101,fd=3))"
        ]
    );
    let udp_line = all
        .stdout
        .lines()
        .find(|s| s.contains("127.0.0.1:5353"))
        .unwrap();
    assert_eq!(
        udp_line.split_whitespace().collect::<Vec<_>>(),
        vec!["udp", "UNCONN", "0", "0", "127.0.0.1:5353", "0.0.0.0:*"]
    );
    let tcp = t.execute("ss -lntp").unwrap().stdout;
    assert_eq!(tcp.lines().count(), 4);
    assert!(!tcp.contains("UNCONN"));
    let udp = t.execute("ss -lnup").unwrap().stdout;
    assert_eq!(udp.lines().count(), 2);
    assert!(!udp.contains("LISTEN"));
    let no_p = t.execute("ss -lntu").unwrap().stdout;
    assert!(!no_p.contains("Process"));
    assert!(!no_p.contains("users:"));
    assert_eq!(t, before);
}
#[test]
fn ordinary_user_sees_all_endpoints_but_only_own_process_details() {
    let mut t = fixture();
    let owner = t.execute("ss -lntup").unwrap().stdout;
    assert!(owner.contains("192.0.2.2:443"));
    assert!(owner.contains("127.0.0.1:5353"));
    assert!(owner.contains("pid=101"));
    assert!(owner.contains("pid=102"));
    assert!(!owner.contains("pid=201"));
    assert!(!owner.contains("pid=300"));
    t.user = Identity {
        name: "root".into(),
        uid: 0,
    };
    assert!(t.valid());
    let root = t.execute("ss -lntup").unwrap().stdout;
    assert!(root.contains("pid=201"));
    assert!(root.contains("pid=300"));
}
#[test]
fn stopped_and_pending_termination_owners_still_hold_the_socket_until_exit() {
    let mut t = fixture();
    let goal = closed(&t);
    let source = t.clone();
    assert!(!goal.check(&t).0);
    assert_eq!(t.execute("kill -STOP 101").unwrap().status, 0);
    assert!(!goal.check(&t).0);
    assert!(goal.check(&t).1.contains("paused"));
    assert!(
        t.execute("ss -lntp")
            .unwrap()
            .stdout
            .contains("0.0.0.0:8081")
    );
    assert_eq!(t.execute("kill -TERM 101").unwrap().status, 0);
    assert!(!goal.check(&t).0);
    assert!(goal.check(&t).1.contains("termination request"));
    assert!(
        t.execute("ss -lntp")
            .unwrap()
            .stdout
            .contains("0.0.0.0:8081")
    );
    assert_eq!(t.execute("kill -CONT 101").unwrap().status, 0);
    assert!(goal.check(&t).0);
    assert!(
        !t.execute("ss -lntp")
            .unwrap()
            .stdout
            .contains("0.0.0.0:8081")
    );
    assert!(t.same_problem(&source));
    assert_eq!(t.listeners, source.listeners);
    assert_eq!(t.processes[&102], source.processes[&102]);
}
#[test]
fn legitimate_end_signals_close_all_owned_rows_without_touching_other_processes() {
    for signal in ["TERM", "INT", "KILL"] {
        let mut t = fixture();
        let mut other = t.listeners[0].clone();
        other.fd = 9;
        other.port = 9091;
        t.listeners.push(other);
        let source = t.clone();
        let req = closed(&t);
        assert_eq!(t.execute(&format!("kill -{signal} 101")).unwrap().status, 0);
        assert!(req.check(&t).0);
        let text = t.execute("ss -lntup").unwrap().stdout;
        assert!(!text.contains("0.0.0.0:8081"));
        assert!(!text.contains("0.0.0.0:9091"));
        for id in [1, 100, 102, 201, 300] {
            assert_eq!(t.observed_process(id), source.observed_process(id));
        }
        assert_eq!(t.listeners, source.listeners);
    }
}
#[test]
fn ownership_failure_and_protected_init_never_remove_other_listeners() {
    let mut t = fixture();
    let source = t.clone();
    for cmd in ["kill -TERM 201", "kill -KILL 300", "kill -KILL 1"] {
        assert_eq!(t.execute(cmd).unwrap().status, 1);
        assert_eq!(t, source);
    }
    assert_eq!(t.execute("kill -0 101").unwrap().status, 0);
    assert_eq!(t, source);
}
#[test]
fn immutable_endpoint_roster_is_pinned_when_other_mutable_process_facts_change() {
    let source = fixture();
    for field in [
        "port", "fd", "address", "backlog", "pid", "protocol", "remove", "order",
    ] {
        let mut t = source.clone();
        match field {
            "port" => t.listeners[0].port = 9091,
            "fd" => t.listeners[0].fd = 19,
            "address" => t.listeners[0].address = Ipv4Addr::LOCALHOST,
            "backlog" => t.listeners[0].backlog = 32,
            "pid" => t.listeners[0].pid = 100,
            "protocol" => {
                t.listeners[0].protocol = Protocol::Udp;
                t.listeners[0].backlog = 0;
            }
            "remove" => {
                t.listeners.remove(0);
            }
            "order" => t.listeners.swap(0, 1),
            _ => unreachable!(),
        }
        assert!(t.valid(), "{field}");
        assert!(!t.same_problem(&source), "{field}");
    }
    let mut ended = source.clone();
    ended.execute("kill -TERM 101").unwrap();
    assert!(ended.same_problem(&source));
}
#[test]
fn bounded_socket_rows_reject_bad_fields_unknown_pids_and_alias_conflicts() {
    let t = fixture();
    let base = serde_json::to_value(&t).unwrap();
    for (field, value) in [
        ("port", serde_json::json!(0)),
        ("pid", serde_json::json!(0)),
        ("fd", serde_json::json!(1048576)),
        ("address", serde_json::json!("::1")),
        ("address", serde_json::json!("x".repeat(200))),
        ("unexpected", serde_json::json!(true)),
    ] {
        let mut raw = base.clone();
        raw["listeners"][0][field] = value;
        let parsed = serde_json::from_value::<Table>(raw);
        assert!(parsed.is_err(), "{field}");
    }
    let mut missing = t.clone();
    missing.listeners[0].pid = 999;
    assert!(!missing.valid());
    let mut udp = t.clone();
    udp.listeners[3].backlog = 1;
    assert!(!udp.valid());
    let mut alias = t.clone();
    let mut row = alias.listeners[0].clone();
    row.port = 9090;
    alias.listeners.push(row);
    assert!(!alias.valid());
    let mut bind = t.clone();
    let mut row = bind.listeners[0].clone();
    row.pid = 102;
    row.fd = 9;
    row.address = Ipv4Addr::LOCALHOST;
    bind.listeners.push(row);
    assert!(!bind.valid());
    let mut direct = t.clone();
    let mut row = direct.listeners[2].clone();
    row.pid = 102;
    row.fd = 9;
    direct.listeners.push(row);
    assert!(!direct.valid());
}
#[test]
fn sixty_four_rows_are_bounded_and_transport_or_specific_address_separation_is_honest() {
    let mut t = fixture();
    t.listeners = (0..64)
        .map(|n| Listener {
            protocol: Protocol::Tcp,
            address: Ipv4Addr::LOCALHOST,
            port: n + 1,
            pid: 101,
            fd: u32::from(n),
            backlog: 128,
        })
        .collect();
    assert!(t.valid());
    let raw = serde_json::to_string(&t).unwrap();
    let read: Table = serde_json::from_str(&raw).unwrap();
    assert_eq!(read, t);
    let mut extra = t.listeners[0].clone();
    extra.port = 65000;
    extra.fd = 100;
    t.listeners.push(extra);
    assert!(!t.valid());
    assert!(serde_json::from_str::<Table>(&serde_json::to_string(&t).unwrap()).is_err());
    let mut t = fixture();
    let mut udp = t.listeners[0].clone();
    udp.protocol = Protocol::Udp;
    udp.backlog = 0;
    udp.fd = 9;
    t.listeners.push(udp);
    assert!(t.valid());
    let mut t = fixture();
    let mut specific = t.listeners[2].clone();
    specific.address = "192.0.2.3".parse().unwrap();
    specific.fd = 9;
    t.listeners.push(specific);
    assert!(t.valid());
}
#[test]
fn malformed_ss_and_host_composition_are_honestly_refused_without_state_change() {
    for input in [
        "ss",
        "ss -a",
        "ss -ltp",
        "ss -lnp",
        "ss --listening",
        "ss -lntp sport = 8081",
        "ss -lntp /etc/passwd",
        "ss -lntp; whoami",
        "ss -lntp | sh",
        "ss -lntp > output",
        "ss -lntp $(id)",
        "ss -lntx",
    ] {
        let mut t = fixture();
        let source = t.clone();
        assert!(t.execute(input).is_err(), "{input}");
        assert_eq!(t, source);
    }
}
#[test]
fn unknown_or_forged_goal_metadata_is_never_a_successful_absence() {
    let t = fixture();
    let mut endpoint = t.listeners[0].clone();
    endpoint.fd = 19;
    let unknown = Requirement::Listener {
        endpoint,
        present: false,
    };
    assert!(!unknown.valid(&t));
    assert!(!unknown.check(&t).0);
    let goal = closed(&t);
    assert!(goal.valid(&t));
    let mut forged = t.clone();
    forged.listeners.remove(0);
    assert!(forged.valid());
    assert!(!forged.same_problem(&t));
    assert!(!goal.check(&forged).0);
    assert!(goal.check(&forged).1.contains("missing or changed"));
}
#[test]
fn current_success_clears_when_a_required_other_process_is_later_ended() {
    let source = fixture();
    let protected = source.processes[&102].clone();
    let goal = engine::Goal::Processes {
        requirements: vec![
            closed(&source),
            Requirement::Unchanged {
                pid: 102,
                expected: protected,
            },
        ],
    };
    let mut state = engine::State::Processes { table: source };
    assert!(!engine::solved(&state, &goal));
    engine::execute(&mut state, "kill -TERM 101").unwrap();
    assert!(engine::solved(&state, &goal));
    engine::execute(&mut state, "kill -TERM 102").unwrap();
    assert!(!engine::solved(&state, &goal));
}
#[test]
fn nonempty_endpoint_goal_and_state_roundtrip_retains_source_shape() {
    let t = fixture();
    let req = closed(&t);
    assert!(req.valid(&t));
    let state = engine::State::Processes { table: t.clone() };
    let raw = serde_json::to_string(&state).unwrap();
    let read: engine::State = serde_json::from_str(&raw).unwrap();
    assert_eq!(read, state);
    let decoded: Requirement = serde_json::from_str(&serde_json::to_string(&req).unwrap()).unwrap();
    assert!(decoded.valid(&t));
    assert_eq!(decoded.check(&t), req.check(&t));
    // Stopped protected endpoints still exist; active/Unchanged is a separate requirement.
    let mut paused = t.clone();
    paused.processes.get_mut(&102).unwrap().phase = Phase::Stopped {
        resume: Activity::Running,
        pending_termination: false,
    };
    assert!(
        Requirement::Listener {
            endpoint: t.listeners[1].clone(),
            present: true
        }
        .check(&paused)
        .0
    );
    assert!(
        !Requirement::Unchanged {
            pid: 102,
            expected: t.processes[&102].clone()
        }
        .check(&paused)
        .0
    );
}

#[test]
fn actual_ss_headers_and_rows_share_calculated_monospace_column_starts() {
    let mut t = fixture();
    for command in ["ss -lntup", "ss -lntp", "ss -lnup"] {
        let output = t.execute(command).unwrap().stdout;
        let mut lines = output.lines();
        let header = lines.next().unwrap();
        let state = header.find("State").unwrap();
        let recv = header.find("Recv-Q").unwrap();
        let send = header.find("Send-Q").unwrap();
        let local = header.find("Local Address:Port").unwrap();
        let peer = header.find("Peer Address:Port").unwrap();
        let process = header.find("Process").unwrap();
        for line in lines {
            if command == "ss -lntup" {
                assert!(line.starts_with("tcp ") || line.starts_with("udp "));
            }
            assert!(line[state..].starts_with("LISTEN") || line[state..].starts_with("UNCONN"));
            assert_eq!(line[recv..send].trim(), "0");
            assert_eq!(line.as_bytes()[recv], b'0');
            assert!(line.as_bytes()[send].is_ascii_digit());
            assert!(!line[send..local].trim().is_empty());
            assert!(line.as_bytes()[local].is_ascii_digit());
            assert!(line[local..peer].trim().contains(':'));
            assert!(line[peer..].starts_with("0.0.0.0:*"));
            if line.contains("users:") {
                assert!(line[process..].starts_with("users:"));
            }
        }
    }
}

#[test]
fn persisted_endpoint_forgery_is_refused_without_overwriting_good_progress() {
    use dlms_learning_lab::{
        pack,
        progress::{self, Attempt, Progress},
        workspace,
    };
    // This private test task is not a registered lesson and grants no catalog credit.
    let mut s = pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| matches!(s.initial, engine::State::Processes { .. }))
        .unwrap();
    s.id = "test-listener-progress".into();
    let table = fixture();
    s.initial = engine::State::Processes {
        table: table.clone(),
    };
    s.goal = engine::Goal::Processes {
        requirements: vec![
            closed(&table),
            Requirement::Phase {
                pid: 101,
                expected: dlms_learning_lab::processes::ExpectedPhase::Exited,
            },
            Requirement::Unchanged {
                pid: 102,
                expected: table.processes[&102].clone(),
            },
        ],
    };
    let lesson = s.process_teaching.as_mut().unwrap();
    let pid = *lesson
        .fixture
        .processes
        .iter()
        .find(|(pid, p)| **pid != 1 && p.owner.uid == 1000)
        .unwrap()
        .0;
    lesson.fixture.listeners.push(Listener {
        protocol: Protocol::Tcp,
        address: Ipv4Addr::LOCALHOST,
        port: 9090,
        pid,
        fd: 3,
        backlog: 64,
    });
    lesson.validate().unwrap();
    let mut a = Attempt::new(&s, 3);
    workspace::prepare(&mut a);
    dlms_learning_lab::process_teaching::prepare(
        &mut a,
        s.process_teaching.as_ref().unwrap(),
        true,
    );
    engine::execute(&mut a.state, "kill -TERM 101").unwrap();
    assert!(progress::current_solved(&a, &s));
    let mut p = Progress {
        schema_version: 1,
        clock: 3,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("progress.json");
    let catalog = vec![s.clone()];
    assert!(
        progress::save(&path, &p).is_err(),
        "Endpoint-bearing work needs its authored source"
    );
    assert!(!path.exists());
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let good = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, &catalog).unwrap();
    assert!(progress::current_solved(&loaded.attempts[&s.id], &s));
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    for change in ["erase", "retarget", "add"] {
        let mut forged: Progress = serde_json::from_slice(&good).unwrap();
        let a = forged.attempts.get_mut(&s.id).unwrap();
        let engine::State::Processes { table } = &mut a.state else {
            panic!()
        };
        match change {
            "erase" => {
                table.listeners.remove(0);
            }
            "retarget" => table.listeners[0].port = 9091,
            "add" => {
                let mut row = table.listeners[0].clone();
                row.port = 9091;
                row.fd = 99;
                table.listeners.push(row)
            }
            _ => unreachable!(),
        }
        assert!(table.valid());
        assert!(!progress::current_solved(a, &s));
        assert!(progress::save_with_catalog(&path, &forged, &catalog).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), good);
    }
    assert!(progress::save_with_catalog(&path, &p, &[]).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), good);
    for change in ["erase example", "retarget example", "version", "family"] {
        let mut forged: Progress = serde_json::from_slice(&good).unwrap();
        let a = forged.attempts.get_mut(&s.id).unwrap();
        match change {
            "erase example" => a
                .workspace
                .as_mut()
                .unwrap()
                .process_learning
                .as_mut()
                .unwrap()
                .table
                .listeners
                .clear(),
            "retarget example" => {
                a.workspace
                    .as_mut()
                    .unwrap()
                    .process_learning
                    .as_mut()
                    .unwrap()
                    .table
                    .listeners[0]
                    .port = 9091
            }
            "version" => a.scenario_version += 1,
            "family" => {
                a.state = engine::State::Script {
                    source: "printf example".into(),
                    run: None,
                }
            }
            _ => unreachable!(),
        }
        assert!(
            progress::save_with_catalog(&path, &forged, &catalog).is_err(),
            "{change}"
        );
        assert_eq!(std::fs::read(&path).unwrap(), good);
    }
}
