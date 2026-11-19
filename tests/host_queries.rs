//! Fixed pure in-process host-query checks; not a BIND execution/reference fixture.
use dlms_learning_lab::{engine::Goal, host_direct_dns as dns, host_workspace as work, pack};
fn fixture() -> work::Definition {
    let labs = pack::load(include_str!("../scenarios/linux-host-lessons-v2.json")).unwrap();
    let s = labs
        .into_iter()
        .find(|s| s.id == "lpic1-host-resolver")
        .unwrap();
    let Goal::HostNetwork { definition } = s.goal else {
        panic!()
    };
    *definition
}
fn event(s: &str) -> work::Event {
    work::Event::Command { input: s.into() }
}
#[test]
fn typed_host_forms_observe_each_actual_server_without_repairing_the_client() {
    let d = fixture();
    let mut s = work::Session::new(&d);
    let before = s.clone();
    for (command, server, name, ip) in [
        (
            "host -t A service.lab.test. 203.0.113.53",
            "203.0.113.53",
            "service.lab.test",
            "203.0.113.80",
        ),
        (
            "host -4 -t a SERVICE.LAB.TEST. 203.0.113.54",
            "203.0.113.54",
            "service.lab.test",
            "203.0.113.90",
        ),
        (
            "host -t A inventory.lab.test. 203.0.113.53",
            "203.0.113.53",
            "inventory.lab.test",
            "203.0.113.90",
        ),
    ] {
        let q = dns::Query::parse(command).unwrap();
        assert_eq!(q.tool, dns::Tool::Host);
        let out = s.run(&d, &event(command), false).unwrap();
        assert!(
            out.starts_with(&format!(
                "Using domain server:\nName: {server}\nAddress: {server}#53\nAliases: \n\n"
            )),
            "{out}"
        );
        assert!(out.contains(&format!("{name} has address {ip}\n")));
        assert!(out.contains("command exit status 0"));
        assert_eq!(s, before);
        assert!(!d.checks(&s, false).iter().all(|r| r.0));
    }
    let out = s
        .run(&d, &event("host -t A service.lab.test. 10.73.0.53"), true)
        .unwrap();
    assert!(out.contains("service.lab.test has address 10.73.0.80\n"));
    assert_eq!(s, before);
    let mut changed = d.initial.clone();
    changed.profile.resolvers[0].records[0].address = "203.0.113.90".parse().unwrap();
    let observed = d
        .direct_dns
        .as_ref()
        .unwrap()
        .initial
        .observe(&changed, "host -t A service.lab.test. 203.0.113.53")
        .unwrap();
    assert!(
        observed
            .output()
            .contains("service.lab.test has address 203.0.113.90\n"),
        "The response must follow changed record state, never a canned command result"
    );
}
#[test]
fn default_reverse_unknown_and_shell_forms_are_honestly_unsupported() {
    let d = fixture();
    let mut s = work::Session::new(&d);
    let before = s.clone();
    for command in [
        "host service.lab.test. 203.0.113.53",
        "host -t A service.lab.test.",
        "host -t AAAA service.lab.test. 203.0.113.53",
        "host -t A missing.lab.test. 203.0.113.53",
        "host -t A service.lab.test 203.0.113.53",
        "host -t A service.lab.test. @203.0.113.53",
        "host -t A service.lab.test. 1.1.1.1",
        "host -t A service.lab.test. resolver.lab.test",
        "host -t A 203.0.113.80 203.0.113.53",
        "host -t A service.lab.test. 203.0.113.53;id",
        "host -t A $(id).lab.test. 203.0.113.53",
        "host -t A service.lab.test. 203.0.113.53 > /tmp/x",
        "host -t A service.lab.test. 203.0.113.53\nhost",
    ] {
        assert!(s.run(&d, &event(command), false).is_err(), "{command}");
        assert_eq!(s, before, "{command}");
    }
}
#[test]
fn host_queries_keep_current_repair_and_path_failures_do_not_invent_dns_status() {
    let d = fixture();
    let mut s = work::Session::new(&d);
    s.run(
        &d,
        &work::Event::Resolver {
            text: "nameserver 203.0.113.53\n".into(),
        },
        false,
    )
    .unwrap();
    s.assignment.checked = true;
    let before = s.clone();
    s.run(
        &d,
        &event("host -t A service.lab.test. 203.0.113.54"),
        false,
    )
    .unwrap();
    assert_eq!(s, before);
    assert!(d.checks(&s, false).iter().all(|r| r.0));
    s.run(&d, &event("ip link set dev ens3 down"), false)
        .unwrap();
    assert!(!s.assignment.checked);
    let out = s
        .run(
            &d,
            &event("host -t A service.lab.test. 203.0.113.53"),
            false,
        )
        .unwrap();
    assert!(out.contains("no reply evaluated"));
    assert!(!out.contains("has address"));
    assert!(!out.contains("command exit status 0"));
    assert!(out.contains("no utility exit status is inferred"));
    assert!(!d.checks(&s, false).iter().all(|r| r.0));
    let mut old = d.clone();
    old.direct_dns = None;
    let mut legacy = work::Session::new(&old);
    assert!(
        legacy
            .run(
                &old,
                &event("host -t A service.lab.test. 203.0.113.53"),
                false
            )
            .is_err()
    );
}
