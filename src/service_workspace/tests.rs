use super::*;
const DB: &str = "ssh 22/tcp secure-shell\nhttp 80/tcp www\nreports-demo 8080/tcp reports\ndomain 53/tcp\ndomain 53/udp dns\n";
fn task(mode: Mode) -> Task {
    let profile = client::Profile {
        version: 1,
        client: "192.0.2.10".parse().unwrap(),
        server: "192.0.2.20".parse().unwrap(),
        listeners: vec![
            client::Listener {
                role: "remote".into(),
                protocol: net::Protocol::Tcp,
                port: 22,
            },
            client::Listener {
                role: "resolver".into(),
                protocol: net::Protocol::Udp,
                port: 53,
            },
            client::Listener {
                role: "reports".into(),
                protocol: net::Protocol::Tcp,
                port: 8080,
            },
        ],
    };
    let required_clients = vec![
        Setting {
            id: "admin".into(),
            protocol: net::Protocol::Tcp,
            service: "ssh".into(),
        },
        Setting {
            id: "lookup".into(),
            protocol: net::Protocol::Udp,
            service: "dns".into(),
        },
    ];
    let mut initial_clients = required_clients.clone();
    let mut initial_database = DB.to_string();
    if mode == Mode::Clients {
        initial_clients[0].service = "reports".into();
        initial_clients[1].protocol = net::Protocol::Tcp;
        initial_clients[1].service = "domain".into();
    } else {
        initial_database = DB
            .replace("ssh 22/tcp secure-shell", "ssh 8080/tcp secure-shell")
            .replace(
                "reports-demo 8080/tcp reports",
                "reports-demo 22/tcp reports",
            );
    }
    let requests = [("admin", "remote"), ("lookup", "resolver")]
        .into_iter()
        .flat_map(|(client, role)| {
            [false, true].into_iter().map(move |outage| Request {
                id: format!("{client}-{}", if outage { "outage" } else { "online" }),
                label: format!(
                    "{client} {}",
                    if outage {
                        "unavailable listener"
                    } else {
                        "available listener"
                    }
                ),
                client: client.into(),
                required_role: role.into(),
                payload: format!("{client}-{}", if outage { 2 } else { 1 }),
                disabled_roles: if outage { vec![role.into()] } else { vec![] },
            })
        })
        .collect();
    Task {
        version: 1,
        id: "services-assignment".into(),
        mode,
        profile,
        initial_database,
        required_database: DB.into(),
        initial_clients,
        required_clients,
        requests,
    }
}
fn definition(mode: Mode) -> Definition {
    let initial = task(mode);
    let mut example = initial.clone();
    example.id = "services-example".into();
    example.profile.client = "198.51.100.10".parse().unwrap();
    example.profile.server = "198.51.100.20".parse().unwrap();
    for r in &mut example.requests {
        r.payload = format!("example-{}", r.payload);
    }
    Definition { initial, example }
}
fn solve(t: &Task, d: &mut Draft) {
    match t.mode {
        Mode::Clients => {
            for c in &t.required_clients {
                d.configure(t, &c.id, c.protocol, c.service.clone())
                    .unwrap();
            }
        }
        Mode::Database => {
            d.edit(t, t.required_database.clone()).unwrap();
            d.save(t).unwrap();
        }
    }
}
#[test]
fn clients_require_destination_and_behavior_not_just_lookup_or_delivery() {
    let t = task(Mode::Clients);
    let mut d = Draft::new(&t).unwrap();
    let bad = d.check(&t).unwrap();
    assert!(!bad.passed());
    let wrong = &bad.cases[0];
    assert_eq!(
        wrong.actual.as_ref().unwrap().transport.outcome,
        net::Outcome::Delivered
    );
    assert!(!wrong.passed);
    assert!(wrong.explanation.contains("different open service"));
    assert!(bad.cases.iter().any(|r| r.explanation.contains("53/tcp")));
    solve(&t, &mut d);
    let good = d.check(&t).unwrap();
    assert!(good.passed());
    assert_eq!(good.cases.len(), 4);
    for c in &good.cases {
        assert!(c.passed);
        assert!(
            c.actual
                .as_ref()
                .unwrap()
                .transport
                .received
                .as_deref()
                .is_none_or(|s| s.starts_with("admin-") || s.starts_with("lookup-"))
        );
    }
    d.configure(&t, "admin", net::Protocol::Udp, "ssh".into())
        .unwrap();
    assert!(!d.checked());
    assert!(!t.report(&d).unwrap().passed());
}
#[test]
fn aliases_numbers_whitespace_and_equivalent_database_formatting_pass() {
    let t = task(Mode::Clients);
    for setting in ["secure-shell", "22", " 00022 "] {
        let mut d = Draft::new(&t).unwrap();
        solve(&t, &mut d);
        d.configure(&t, "admin", net::Protocol::Tcp, setting.into())
            .unwrap();
        assert!(d.check(&t).unwrap().passed(), "{setting}");
    }
    let t = task(Mode::Database);
    let mut d = Draft::new(&t).unwrap();
    let equivalent = format!(
        "# same meaning\n{}",
        DB.lines()
            .rev()
            .map(|l| format!("{} # preserved\n", l.replace(' ', "\t")))
            .collect::<String>()
    );
    d.edit(&t, equivalent).unwrap();
    assert!(!d.check(&t).unwrap().passed());
    d.save(&t).unwrap();
    assert!(!d.checked());
    assert!(d.check(&t).unwrap().passed());
}
#[test]
fn repairing_only_queried_names_cannot_delete_unrelated_reference_meanings() {
    let t = task(Mode::Database);
    let mut d = Draft::new(&t).unwrap();
    d.edit(&t, "ssh 22/tcp secure-shell\ndomain 53/udp dns\n".into())
        .unwrap();
    d.save(&t).unwrap();
    let r = d.check(&t).unwrap();
    assert!(r.cases.iter().all(|c| c.passed));
    assert!(!r.passed());
    assert!(r.database_errors.iter().any(|e| e.contains("reports")));
    assert!(r.database_errors.iter().any(|e| e.contains("www")));
}
#[test]
fn partial_repair_syntax_unsupported_and_unsaved_results_are_specific() {
    let t = task(Mode::Clients);
    let mut d = Draft::new(&t).unwrap();
    d.configure(&t, "admin", net::Protocol::Tcp, "ssh".into())
        .unwrap();
    let r = d.check(&t).unwrap();
    assert_eq!(r.cases.iter().filter(|c| c.passed).count(), 2);
    assert!(!r.passed());
    d.configure(&t, "lookup", net::Protocol::Udp, "".into())
        .unwrap();
    assert!(d.check(&t).unwrap().cases[2].explanation.contains("Syntax"));
    let t = task(Mode::Database);
    let mut d = Draft::new(&t).unwrap();
    d.edit(&t, "ssh".into()).unwrap();
    let r = d.check(&t).unwrap();
    assert!(r.unsaved);
    d.save(&t).unwrap();
    let r = d.check(&t).unwrap();
    assert!(r.database_errors[0].contains("Syntax"));
    assert!(!r.passed());
    d.edit(&t, "ssh 22/sctp\n".into()).unwrap();
    d.save(&t).unwrap();
    assert!(d.check(&t).unwrap().database_errors[0].contains("Unsupported"));
    solve(&t, &mut d);
    assert!(d.check(&t).unwrap().passed());
}
#[test]
fn practice_assignment_reset_and_roundtrip_are_independent_and_source_bound() {
    let def = definition(Mode::Clients);
    let mut s = Session::new(&def).unwrap();
    let original = s.assignment.clone();
    solve(&def.example, &mut s.practice);
    assert!(s.practice.check(&def.example).unwrap().passed());
    assert_eq!(s.assignment, original);
    assert!(def.checks(&s, false).iter().all(|x| !x.0));
    s.assignment
        .configure(&def.initial, "admin", net::Protocol::Tcp, "22".into())
        .unwrap();
    s.assignment.terminal_input = "getent services ssh/tcp".into();
    s.view = View::Assignment;
    let saved = serde_json::to_string(&s).unwrap();
    let mut loaded: Session = serde_json::from_str(&saved).unwrap();
    assert_eq!(s, loaded);
    loaded.validate(&def).unwrap();
    loaded.practice = Draft::new(&def.example).unwrap();
    assert_eq!(s.assignment, loaded.assignment);
    let mut changed = def.clone();
    changed.initial.requests[0].payload = "changed".into();
    assert!(loaded.validate(&changed).is_err());
    solve(&def.initial, &mut loaded.assignment);
    loaded.assignment.check(&def.initial).unwrap();
    assert!(def.checks(&loaded, false).iter().all(|x| x.0));
    loaded.assignment = Draft::new(&def.initial).unwrap();
    assert!(def.checks(&loaded, false).iter().any(|x| !x.0));
}
#[test]
fn forged_current_stamp_cannot_authorize_incorrect_state() {
    let def = definition(Mode::Clients);
    let mut s = Session::new(&def).unwrap();
    solve(&def.initial, &mut s.assignment);
    s.assignment.check(&def.initial).unwrap();
    s.assignment.clients[0].service = "reports".into();
    s.assignment.checked_stamp = Some(s.assignment.stamp());
    assert!(s.assignment.checked());
    assert!(def.checks(&s, false).iter().any(|x| !x.0));
    s.assignment.clients[0].id = "invented".into();
    assert!(s.validate(&def).is_err());
}
#[test]
fn virtual_terminal_is_read_only_bounded_and_uses_saved_text() {
    let t = task(Mode::Database);
    let mut d = Draft::new(&t).unwrap();
    let initial = d.saved_database.clone();
    d.command(&t, "getent services ssh/tcp").unwrap();
    assert!(d.transcript.contains("8080/tcp"));
    d.edit(&t, DB.into()).unwrap();
    d.command(&t, "getent services ssh/tcp").unwrap();
    assert!(d.transcript.ends_with("8080/tcp secure-shell\n[exit 0]\n"));
    d.save(&t).unwrap();
    d.command(&t, "getent services ssh/tcp").unwrap();
    assert!(d.transcript.ends_with("22/tcp secure-shell\n[exit 0]\n"));
    for command in [
        "cat /private",
        "getent passwd",
        "rm -rf /",
        "getent services ssh | sh",
    ] {
        d.command(&t, command).unwrap();
        assert!(d.transcript.contains("Unsupported"));
    }
    assert_ne!(d.saved_database, initial);
    assert_eq!(d.saved_database, DB);
    for _ in 0..200 {
        d.command(&t, "cat /etc/services").unwrap();
    }
    assert!(d.transcript.len() <= 20000);
    d.shape().unwrap();
}
#[test]
fn invalid_source_wrong_modes_and_import_limits_fail_without_mutation() {
    let t = task(Mode::Clients);
    let mut d = Draft::new(&t).unwrap();
    let before = d.clone();
    assert!(d.edit(&t, DB.into()).is_err());
    assert!(d.save(&t).is_err());
    assert!(
        d.configure(&t, "invented", net::Protocol::Tcp, "22".into())
            .is_err()
    );
    assert_eq!(d, before);
    let mut bad = t.clone();
    bad.required_clients[0].service = "reports".into();
    assert!(bad.validate().is_err());
    bad = t.clone();
    bad.requests[0].disabled_roles = vec!["invented".into()];
    assert!(bad.validate().is_err());
    bad = t.clone();
    bad.requests.remove(1);
    assert!(bad.validate().is_err());
    let t = task(Mode::Database);
    let mut d = Draft::new(&t).unwrap();
    assert!(
        d.configure(&t, "admin", net::Protocol::Tcp, "22".into())
            .is_err()
    );
    assert!(d.edit(&t, "x".repeat(32769)).is_err());
    let mut v = serde_json::to_value(&d).unwrap();
    v["editor"] = serde_json::Value::String("x".repeat(32769));
    assert!(serde_json::from_value::<Draft>(v).is_err());
}

#[test]
fn imported_outage_roles_are_bounded_before_constructing_source_state() {
    let source = task(Mode::Clients);
    let mut value = serde_json::to_value(source).unwrap();
    value["requests"][0]["disabled_roles"] = serde_json::json!(["x".repeat(65)]);
    assert!(serde_json::from_value::<Task>(value.clone()).is_err());
    value["requests"][0]["disabled_roles"] = serde_json::json!(vec!["remote"; 9]);
    assert!(serde_json::from_value::<Task>(value).is_err());
}
