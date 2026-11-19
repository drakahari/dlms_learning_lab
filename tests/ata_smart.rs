use dlms_learning_lab::ata_smart::{
    Layout, State,
    command::{self, Operation},
    monitor,
    session::{Definition, Session},
    source::{self, Action},
    workspace::{Assignment, Disposition, Task, Ticket, Work},
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    definition: Definition,
    assignment_actions: Vec<Action>,
    example_actions: Vec<Action>,
}

fn fixtures() -> Vec<Fixture> {
    serde_json::from_str(include_str!("fixtures/ata_smart_tasks_v1.json")).unwrap()
}

fn start(task: &Task, work: &mut Work, command: &str) {
    source::apply(
        task,
        work,
        Action::Run {
            input: command.into(),
        },
    )
    .unwrap();
}

fn passed(work: &Work, task: &Task) -> usize {
    work.checks(task)
        .iter()
        .filter(|check| check.passed)
        .count()
}

#[test]
fn literal_disk_commands_accept_documented_forms_and_reject_expansion() {
    let value = command::parse("smartctl --health /dev/sdb").unwrap();
    assert!(matches!(
        value,
        command::Command::Device {
            operation: Operation::Health,
            ..
        }
    ));
    assert_eq!(value, command::parse("smartctl -H /dev/sdb").unwrap());
    assert!(command::parse("smartctl --log=selftest /dev/disk/by-id/ata-Workshop_FAKE").is_ok());
    for invalid in [
        "smartctl -H /dev/nvme0",
        "smartctl -H /dev/sda ; whoami",
        "smartctl -l selftest ../../dev/sda",
        "smartctl -l error /dev/sda",
        "smartctl -i /dev/disk/by-id/ata-../sdb",
        "smartctl -i /etc/passwd",
        "smartctl --scan",
        "smartctl -t long /dev/sda",
        "smartctl -t short /dev/sda\ntouch /tmp/unexpected",
        "smartctl -i /dev/sdb | cat",
        "smartctl -i /dev/sd*",
    ] {
        assert!(command::parse(invalid).is_err(), "{invalid}");
    }
    assert!(command::parse(&"x".repeat(command::INPUT_LIMIT + 1)).is_err());
}

#[test]
fn headline_self_test_history_and_current_result_are_separate() {
    let f = fixtures().remove(0);
    let t = &f.definition.assignment;
    let mut w = Work::new(t);
    start(t, &mut w, "smartctl -H /dev/sdb");
    let health = w.transcript.last().unwrap();
    assert_eq!(health.status, Some(0));
    assert!(health.output.contains("PASSED"));
    start(t, &mut w, "smartctl -l selftest /dev/sdb");
    let old = w.transcript.last().unwrap();
    assert_eq!(old.status, Some(128));
    assert!(old.output.contains("491520"));
    assert!(!old.output.contains("497616"));
    assert_eq!(passed(&w, t), 1);

    start(t, &mut w, "smartctl --test=short /dev/sdb");
    assert_eq!(w.state.disks[1].remaining_minutes, Some(3));
    start(t, &mut w, "smartctl -t short /dev/sdb");
    assert!(w.transcript.last().unwrap().status.is_none());
    source::apply(t, &mut w, Action::Check).unwrap();
    assert_eq!(passed(&w, t), 1);
    source::apply(t, &mut w, Action::Advance).unwrap();
    assert!(!w.checked);
    assert_eq!(w.state.disks[1].remaining_minutes, Some(2));
    for _ in 0..2 {
        source::apply(t, &mut w, Action::Advance).unwrap();
    }
    assert_eq!(passed(&w, t), 2);
    start(t, &mut w, "smartctl --log=selftest /dev/sdb");
    let result = w.transcript.last().unwrap();
    assert!(result.output.contains("497616"));
    assert_eq!(result.status, Some(128));
    assert!(result.output.find("497616").unwrap() < result.output.find("491520").unwrap());

    let ticket = |lba: &str| Action::Record {
        ticket: Ticket {
            disposition: Disposition::ReadFailure,
            first_lba: lba.into(),
        },
    };
    source::apply(t, &mut w, ticket("491520")).unwrap();
    assert_eq!(passed(&w, t), 2);
    source::apply(t, &mut w, ticket("497616")).unwrap();
    source::apply(t, &mut w, Action::Check).unwrap();
    assert_eq!(passed(&w, t), 3);
    start(t, &mut w, "smartctl -t short /dev/sda");
    assert!(!w.checked);
    assert_eq!(passed(&w, t), 2);
}

#[test]
fn monitor_parser_keeps_literal_supported_options_and_rejects_duplicates_and_changes() {
    let rules = monitor::parse("/dev/sda -l selftest -H # data\r\n/dev/sdb -H\r\n").unwrap();
    assert!(rules[0].health && rules[0].selftest);
    assert!(rules[1].health && !rules[1].selftest);
    for bad in [
        "DEVICESCAN -H",
        "/dev/sda",
        "/dev/sda -H -H",
        "/dev/sda -l error",
        "/dev/sdb -H -M exec /bin/sh",
        "/dev/sda -H\n/dev/sda -l selftest",
        "/dev/sda -H\r/dev/sdb -H",
        "/dev/../../etc/sda -H",
        "/dev/sda -H \\",
    ] {
        assert!(monitor::parse(bad).is_err(), "{bad}");
    }
}

#[test]
fn monitor_uses_disk_identity_in_both_layouts_and_accepts_equivalent_configuration() {
    let f = fixtures().remove(1);
    let t = &f.definition.assignment;
    let Assignment::Monitoring(goal) = &t.goal else {
        panic!()
    };
    let current = monitor::parse(&goal.initial_config).unwrap();
    let now = monitor::preview(&current, &t.inventory, Layout::Current);
    let future = monitor::preview(&current, &t.inventory, Layout::Maintenance);
    assert_eq!(now[0].disk.unwrap().serial, goal.primary_serial);
    assert_eq!(future[0].disk.unwrap().serial, goal.auxiliary_serial);
    let mut w = Work::new(t);
    assert_eq!(passed(&w, t), 2);
    let first = &t.inventory.disks[0].stable;
    let other = &t.inventory.disks[1].stable;
    let correct = format!("# Keep only these checks\n{other} -H\n{first} -l selftest   -H\n");
    source::apply(t, &mut w, Action::Draft { contents: correct }).unwrap();
    assert_eq!(passed(&w, t), 2);
    assert!(!w.checks(t)[0].passed);
    source::apply(t, &mut w, Action::Save).unwrap();
    source::apply(t, &mut w, Action::Check).unwrap();
    assert_eq!(passed(&w, t), 4);

    let duplicate = format!("{first} -H -l selftest\n/dev/sdb -H\n");
    source::apply(
        t,
        &mut w,
        Action::Draft {
            contents: duplicate,
        },
    )
    .unwrap();
    assert!(!w.checked);
    source::apply(t, &mut w, Action::Save).unwrap();
    assert!(!w.checks(t)[2].passed);
    let saved = w.monitor_saved.clone();
    source::apply(
        t,
        &mut w,
        Action::Draft {
            contents: "DEVICESCAN -H".into(),
        },
    )
    .unwrap();
    assert!(source::apply(t, &mut w, Action::Save).is_err());
    assert_eq!(w.monitor_saved, saved);
}

#[test]
fn every_author_reference_executes_and_scopes_and_source_shapes_round_trip() {
    let mut models = Vec::new();
    for fixture in fixtures() {
        let d = &fixture.definition;
        let initial = Session::new(d).unwrap();
        initial.validate_shape().unwrap();
        let mut work = initial.clone();
        for action in &fixture.example_actions {
            source::apply(&d.example, &mut work.example.work, action.clone()).unwrap();
        }
        assert!(
            work.example.work.checked
                && work
                    .example
                    .work
                    .checks(&d.example)
                    .iter()
                    .all(|c| c.passed)
        );
        assert_eq!(work.assignment, initial.assignment);
        assert!(source::checks(&work, d).iter().all(|c| !c.0));
        for action in &fixture.assignment_actions {
            source::apply(&d.assignment, &mut work.assignment.work, action.clone()).unwrap();
        }
        assert!(source::checks(&work, d).iter().all(|c| c.0));
        let bytes = serde_json::to_vec(&work).unwrap();
        let mut loaded: Session = serde_json::from_slice(&bytes).unwrap();
        loaded.bind(d).unwrap();
        assert_eq!(loaded, work);
        loaded.reset_example(d).unwrap();
        assert_eq!(loaded.example, initial.example);
        assert_eq!(loaded.assignment, work.assignment);
        models.push(serde_json::json!({
            "definition":d,"simulation":initial,
            "assignment_actions":fixture.assignment_actions,"example_actions":fixture.example_actions,
        }));
    }
    println!(
        "SMART_AUTHORED_MODELS={}",
        serde_json::to_string(&models).unwrap()
    );
}

#[test]
fn saved_state_cannot_replace_source_log_entries_or_create_impossible_records() {
    let t = &fixtures()[0].definition.assignment;
    let mut state = State::new(&t.inventory);
    state.disks[1].log.clear();
    assert!(state.validate(&t.inventory).is_err());
    let mut state = State::new(&t.inventory);
    state.disks[0].remaining_minutes = Some(3);
    state.disks[0].started = 1;
    assert!(state.validate(&t.inventory).is_err());
    let mut d = t.inventory.clone();
    d.disks[0].maintenance_name = d.disks[1].maintenance_name.clone();
    assert!(d.validate().is_err());
}
