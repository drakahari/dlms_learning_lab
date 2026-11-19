use dlms_learning_lab::iscsi::{
    self, Portal, Startup,
    command::{self, Command},
    session::{Definition, Session},
    workspace::{Attempt, Task},
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    definition: Definition,
    assignment_commands: Vec<String>,
    example_commands: Vec<String>,
}

fn fixtures() -> Vec<Fixture> {
    serde_json::from_str(include_str!("fixtures/iscsi_tasks_v1.json")).unwrap()
}

fn update(task: &Task, policy: &str) -> String {
    let key = &task.goal.target;
    format!(
        "iscsiadm -m node -T {} -p {} -o update -n node.startup -v {policy}",
        key.target, key.portal
    )
}

fn run(task: &Task, work: &mut Attempt, command: &str) -> bool {
    work.input = command.into();
    work.run(task);
    work.history.last().unwrap().ok
}

#[test]
fn parses_bounded_supported_forms_and_rejects_shell_syntax() {
    let simple =
        command::parse("iscsiadm -m discoverydb -t sendtargets -p 192.0.2.1 --discover").unwrap();
    assert_eq!(simple, Command::Discover("192.0.2.1:3260".parse().unwrap()));
    assert_eq!(
        command::parse("iscsiadm --portal=192.0.2.1 --mode=discoverydb --type=sendtargets -D")
            .unwrap(),
        simple
    );
    assert_eq!(
        command::parse("iscsiadm --mode=session --print=3").unwrap(),
        Command::Sessions { devices: true }
    );
    assert_eq!(
        command::parse("lsblk --output=NAME,WWN").unwrap(),
        Command::DeviceIds
    );
    assert_eq!(
        command::parse("iscsiadm -L automatic -m node").unwrap(),
        Command::LoginAutomatic
    );
    for invalid in [
        "iscsiadm -m node --login",
        "iscsiadm -m node -T iqn.local -p 192.0.2.1",
        "iscsiadm -m node --loginall=all",
        "iscsiadm -m node -m session",
        "iscsiadm -m node --login=1",
        "iscsiadm -m discoverydb -t sendtargets -p 192.0.2.1 --discover --login",
        "iscsiadm -m discoverydb -t sendtargets -p 192.0.2.1:3260,4 --discover",
        "iscsiadm -m session -P 5",
        "iscsiadm -m session -P 3 -p 192.0.2.1",
        "iscsiadm -m node -T iqn.local -p 192.0.2.1 -o update -n node.conn[0].startup -v automatic",
        "iscsiadm -m node -T iqn.local -p 192.0.2.1 -o update -n node.startup -v onboot",
        "iscsiadm -m session; cat /etc/shadow",
        "iscsiadm -m session\nprintf ok",
        "iscsiadm -m session$(id)",
        "lsblk --sysroot /",
        "cat /sys/class/scsi_device",
    ] {
        assert!(command::parse(invalid).is_err(), "{invalid}");
    }
    for bad in [
        "127.0.0.1:",
        "127.0.0.1:0",
        "127.0.0.1:65536",
        "127.0.0.1:+2",
        "::1",
        "localhost",
    ] {
        assert!(bad.parse::<Portal>().is_err(), "{bad}");
    }
}

#[test]
fn discovery_records_can_repeat_without_creating_or_restarting_sessions() {
    let f = &fixtures()[0];
    let t = &f.definition.assignment;
    let mut w = Attempt::new(t);
    let key = &t.goal.target;
    assert!(!run(t, &mut w, &f.assignment_commands[2]));
    assert!(run(t, &mut w, &f.assignment_commands[0]));
    assert_eq!(w.initiator.nodes.len(), 2);
    assert!(w.initiator.active.is_empty());
    assert!(t.checks(&w.initiator)[0].passed);
    assert!(!t.checks(&w.initiator)[1].passed);
    assert!(!run(t, &mut w, "iscsiadm -m session"));
    assert!(run(t, &mut w, &f.assignment_commands[2]));
    let initial_session = w.initiator.session(key).unwrap().clone();
    assert!(run(t, &mut w, &update(t, "automatic")));
    assert!(run(t, &mut w, &f.assignment_commands[0]));
    assert_eq!(w.initiator.nodes.len(), 2);
    assert_eq!(w.initiator.session(key), Some(&initial_session));
    assert_eq!(w.initiator.node(key).unwrap().startup, Startup::Automatic);
    assert!(!w.history[0].ok);
    assert!(t.passed(&w.initiator));
}

#[test]
fn different_target_on_same_portal_and_lun_fails_until_corrected() {
    let f = &fixtures()[0];
    let t = &f.definition.assignment;
    let wrong = &t.inventory.exports[0].key;
    let mut w = Attempt::new(t);
    assert!(run(t, &mut w, &f.assignment_commands[0]));
    let target = &t.goal.target;
    assert_eq!(wrong.portal, target.portal);
    let discovered_at = t.inventory.exports[0].discovery;
    assert!(!run(
        t,
        &mut w,
        &format!(
            "iscsiadm -m node -T {} -p {discovered_at} -l",
            target.target
        )
    ));
    let wrong_command = format!(
        "iscsiadm -m node -T {} -p {} -l",
        wrong.target, wrong.portal
    );
    assert!(run(t, &mut w, &wrong_command));
    assert!(run(t, &mut w, "iscsiadm -m session -P 3"));
    assert!(w.history.last().unwrap().output.contains("Lun: 0"));
    assert!(run(t, &mut w, "lsblk -o NAME,WWN"));
    assert!(!w.history.last().unwrap().output.contains(&t.goal.wwn));
    assert!(!t.passed(&w.initiator));
    assert!(run(t, &mut w, &f.assignment_commands[2]));
    assert!(t.checks(&w.initiator)[1].passed);
    assert!(!t.passed(&w.initiator));
    assert!(run(t, &mut w, &wrong_command.replace(" -l", " -u")));
    assert!(t.passed(&w.initiator));
    assert!(run(t, &mut w, "lsblk --output=NAME,WWN"));
    assert!(w.history.last().unwrap().output.contains(&t.goal.wwn));
}

#[test]
fn automatic_policy_and_live_connection_are_distinct_checks() {
    let f = &fixtures()[1];
    let t = &f.definition.assignment;
    let mut w = Attempt::new(t);
    let initial = w.initiator.active.clone();
    assert!(run(t, &mut w, &update(t, "automatic")));
    assert_eq!(w.initiator.active, initial);
    assert_eq!(
        t.startup_plan(&w.initiator).collect::<Vec<_>>(),
        [(&t.goal.target, true)]
    );
    assert!(t.checks(&w.initiator)[3].passed);
    assert!(!t.passed(&w.initiator));
    let target = &t.goal.target;
    assert!(run(
        t,
        &mut w,
        &format!(
            "iscsiadm --mode=node --targetname={} --portal={} -l",
            target.target, target.portal.address
        )
    ));
    assert!(t.passed(&w.initiator));
    w.checked = true;
    assert!(run(t, &mut w, "iscsiadm -m session -P 3"));
    assert!(w.checked);
    assert!(run(t, &mut w, &update(t, "manual")));
    assert!(!w.checked);
    assert!(t.checks(&w.initiator)[1].passed);
    assert!(!t.checks(&w.initiator)[3].passed);
}

#[test]
fn preserve_original_auxiliary_connection_and_limit_automatic_startup() {
    let f = &fixtures()[1];
    let t = &f.definition.assignment;
    let original = &t.initial.active[0];
    let k = &original.key;
    let mut w = Attempt::new(t);
    for command in &f.assignment_commands {
        assert!(run(t, &mut w, command));
    }
    assert_eq!(w.initiator.session(k), Some(original));
    assert!(t.passed(&w.initiator));
    let extra = format!(
        "iscsiadm -m node -T {} -p {} -o update -n node.startup -v automatic",
        k.target, k.portal
    );
    assert!(run(t, &mut w, &extra));
    assert!(!t.checks(&w.initiator)[3].passed);
    assert!(run(t, &mut w, &extra.replace("automatic", "manual")));
    assert!(t.passed(&w.initiator));
    let selector = format!("iscsiadm -m node -T {} -p {}", k.target, k.portal);
    assert!(run(t, &mut w, &format!("{selector} -u")));
    assert!(run(t, &mut w, &format!("{selector} -l")));
    assert_ne!(w.initiator.session(k), Some(original));
    assert!(!t.checks(&w.initiator)[2].passed);
}

#[test]
fn actual_login_failure_does_not_change_saved_policy_or_attach_devices() {
    let f = &fixtures()[0];
    let mut t = f.definition.assignment.clone();
    t.inventory.exports[0].login_available = false;
    let key = t.inventory.exports[0].key.clone();
    let mut w = Attempt::new(&t);
    assert!(run(&t, &mut w, &f.assignment_commands[0]));
    let startup = format!(
        "iscsiadm -m node -T {} -p {} -o update -n node.startup -v automatic",
        key.target, key.portal
    );
    assert!(run(&t, &mut w, &startup));
    let before = w.initiator.clone();
    assert!(!run(&t, &mut w, "iscsiadm -m node --loginall=automatic"));
    assert_eq!(w.initiator, before);
    assert_eq!(
        t.startup_plan(&w.initiator).collect::<Vec<_>>(),
        [(&key, false)]
    );
    assert!(run(&t, &mut w, "lsblk -o NAME,WWN"));
    assert_eq!(w.history.last().unwrap().output, "NAME WWN");
}

#[test]
fn history_and_input_are_bounded_and_errors_do_not_open_a_session() {
    let t = &fixtures()[0].definition.assignment;
    let mut w = Attempt::new(t);
    for _ in 0..12 {
        assert!(run(t, &mut w, "iscsiadm --help"));
    }
    let long = "ä".repeat(iscsi::command::INPUT_LIMIT);
    assert!(!run(t, &mut w, &long));
    assert_eq!(w.history.len(), 8);
    assert!(w.history.last().unwrap().input.len() <= command::INPUT_LIMIT);
    assert!(w.initiator.active.is_empty());
    w.validate(t).unwrap();
}

#[test]
fn validated_pack_models_produce_success_in_both_scopes_without_credit_leakage() {
    let mut models = Vec::new();
    for f in fixtures() {
        let d = &f.definition;
        let initial = Session::new(d).unwrap();
        initial.validate_shape().unwrap();
        let mut session = initial.clone();
        for command in &f.example_commands {
            assert!(
                run(&d.example, &mut session.example.work, command),
                "{command}"
            );
        }
        session.example.work.checked = true;
        assert!(d.example.passed(&session.example.work.initiator));
        assert_eq!(session.assignment, initial.assignment);
        assert!(iscsi::source::checks(&session, d).iter().all(|c| !c.0));
        for command in &f.assignment_commands {
            assert!(
                run(&d.assignment, &mut session.assignment.work, command),
                "{command}"
            );
        }
        session.assignment.work.checked = true;
        assert!(iscsi::source::checks(&session, d).iter().all(|c| c.0));
        session.assignment.work.input = "iscsiadm -m session".into();
        let json = serde_json::to_string(&session).unwrap();
        let mut restored: Session = serde_json::from_str(&json).unwrap();
        restored.bind(d).unwrap();
        assert_eq!(restored, session);
        restored.reset_example(d).unwrap();
        assert_eq!(restored.example, initial.example);
        assert_eq!(restored.assignment, session.assignment);
        let mut tampered = d.clone();
        tampered.assignment.inventory.exports[0].group += 1;
        assert!(restored.bind(&tampered).is_err());
        models.push(serde_json::json!({
            "definition": d, "simulation": initial,
            "assignment_commands": f.assignment_commands, "example_commands": f.example_commands,
        }));
    }
    println!(
        "ISCSI_AUTHORED_MODELS={}",
        serde_json::to_string(&models).unwrap()
    );
}

#[test]
fn rejects_invalid_export_capacity_and_state() {
    let mut d = fixtures().remove(0).definition;
    let duplicate = d.assignment.inventory.exports[0].clone();
    d.assignment.inventory.exports.push(duplicate);
    assert!(d.validate().is_err());
    d.assignment.inventory.exports.pop();
    d.assignment.goal.wwn = "0x5000000000000000".into();
    assert!(d.validate().is_err());
    let f = &fixtures()[1];
    let mut w = Attempt::new(&f.definition.assignment);
    w.initiator.next_id = 1;
    assert!(w.validate(&f.definition.assignment).is_err());
}
