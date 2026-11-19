use dlms_learning_lab::{bios_loader as core, bios_loader_workspace as w, partition_fs};
#[path = "support/bios_fixture.rs"]
mod fixture;
fn definition() -> w::Definition {
    let facts = fixture::facts();
    let initial = core::World {
        version: 1,
        records: vec![
            core::Record {
                disk: "/dev/vda".into(),
                code: core::Code::Other {
                    label: "stale loader".into(),
                },
            },
            core::Record {
                disk: "/dev/vdb".into(),
                code: core::Code::Other {
                    label: "archive loader".into(),
                },
            },
        ],
        checked: None,
    };
    let assignment = w::Task {
        id: "lpic1-bios-workspace-test".into(),
        instructions: "Recover the supplied BIOS boot disk without changing the archive loader."
            .into(),
        facts: facts.clone(),
        initial: initial.clone(),
        reference: vec![
            "grub-install --boot-directory=/target/boot /dev/vda".into(),
            w::CHECK_ACTION.into(),
        ],
        required_root: fixture::ROOT.into(),
        required_release: "7.0".into(),
        read_only: true,
        target: "multi-user.target".into(),
        preserve: vec!["/dev/vdb".into()],
    };
    let mut example = assignment.clone();
    example.id = "workshop-bios-workspace-test".into();
    example.facts.boot_disk = "/dev/vdb".into();
    example.preserve = vec!["/dev/vda".into()];
    example.reference[0] = "grub-install --boot-directory=/workshop/boot /dev/vdb".into();
    let uuid = "00112233-4455-6677-8899-aabbccddeeff";
    example.required_root = uuid.into();
    example.facts.directories[0].path = "/workshop/boot".into();
    for disk in example.facts.geometry.disks.values_mut() {
        if let Some(table) = &mut disk.mbr {
            for entry in &mut table.entries {
                if let partition_fs::Content::Ext4 { uuid: value, .. } =
                    &mut entry.partition.content
                    && value == fixture::ROOT
                {
                    *value = uuid.into();
                }
            }
        }
    }
    let c = example.facts.directories[0].configuration.as_mut().unwrap();
    c.menu = c.menu.replace(fixture::ROOT, uuid);
    c.inventory.roots[0].uuid = uuid.into();
    w::Definition {
        version: 1,
        initial: assignment,
        example,
    }
}
fn run(d: &w::Task, s: &mut w::Draft, command: &str) {
    s.command = command.into();
    s.run(d).unwrap();
}
#[test]
fn complete_console_task_has_separate_practice_real_failures_alternatives_and_current_test_credit()
{
    let d = definition();
    d.validate().unwrap();
    let mut s = w::Session::new(&d).unwrap();
    run(&d.example, &mut s.practice, &d.example.reference[0]);
    s.practice.test(&d.example).unwrap();
    assert!(d.example.checks(&s.practice.current).iter().all(|c| c.0));
    assert!(d.checks(&s).iter().any(|c| !c.0));
    assert_eq!(s.assignment.current, d.initial.initial);
    for command in [
        "grub-install --boot-directory=/target/boot /dev/vda",
        "grub-install /dev/vda --boot-directory '/target/boot' --target i386-pc",
    ] {
        run(&d.initial, &mut s.assignment, command);
        assert!(d.checks(&s).iter().any(|c| !c.0));
        s.assignment.test(&d.initial).unwrap();
        assert!(d.checks(&s).iter().all(|c| c.0));
    }
    run(&d.initial, &mut s.assignment, "grub-install /dev/vda");
    assert!(d.checks(&s).iter().any(|c| !c.0));
    s.assignment.test(&d.initial).unwrap();
    assert!(
        d.checks(&s)
            .iter()
            .any(|(ok, msg)| !ok && msg.contains("does not generate"))
    );
    run(&d.initial, &mut s.assignment, &d.initial.reference[0]);
    s.assignment.test(&d.initial).unwrap();
    assert!(d.checks(&s).iter().all(|c| c.0));
    run(
        &d.initial,
        &mut s.assignment,
        "grub-install --boot-directory=/target/boot /dev/vdb",
    );
    s.assignment.test(&d.initial).unwrap();
    assert!(
        s.assignment
            .current
            .trace(&d.initial.facts)
            .unwrap()
            .passed()
    );
    assert!(
        d.checks(&s)
            .iter()
            .any(|(ok, msg)| !ok && msg.contains("/dev/vdb changed")),
        "a good boot path must not conceal an overwritten protected loader"
    );
}
#[test]
fn references_require_repair_and_test_and_cannot_pass_by_hardcoded_output() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    for command in &d.initial.reference {
        s.execute(&d, command).unwrap();
    }
    assert!(d.checks(&s).iter().all(|c| c.0));
    for reference in [
        vec![w::CHECK_ACTION.into()],
        vec!["grub-install /dev/vda".into(), w::CHECK_ACTION.into()],
        vec![d.initial.reference[0].clone()],
        vec!["echo success".into(), w::CHECK_ACTION.into()],
    ] {
        let mut bad = d.clone();
        bad.initial.reference = reference;
        assert!(bad.validate().is_err());
    }
    let mut only_test = d.clone();
    only_test.initial.initial = s.assignment.current.clone();
    only_test.initial.initial.checked = None;
    assert!(
        only_test.validate().is_err(),
        "do not register a trivial press-Test-only task"
    );
    let mut changed = d.initial.clone();
    changed.facts.core_sectors = 2048;
    assert!(
        changed.validate().is_err(),
        "actual geometry must control solution reachability"
    );
}
#[test]
fn unfinished_input_transcript_and_independent_observations_resume_and_reset_without_invented_answers()
 {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    run(&d.initial, &mut s.assignment, &d.initial.reference[0]);
    s.assignment.test(&d.initial).unwrap();
    s.assignment.command = "grub-install --boot-directory='unfinished".into();
    run(
        &d.example,
        &mut s.practice,
        "lsblk -o NAME,TYPE,FSTYPE,MOUNTPOINTS",
    );
    let example = s.practice.clone();
    let json = serde_json::to_string(&s).unwrap();
    let loaded: w::Session = serde_json::from_str(&json).unwrap();
    loaded.bound(&d).unwrap();
    assert_eq!(loaded, s);
    assert!(d.checks(&loaded).iter().all(|c| c.0));
    let mut forged = loaded.clone();
    forged.context = "another-task".into();
    assert!(forged.bound(&d).is_err());
    let mut wrong = loaded.clone();
    let core::Code::GrubBios { boot_directory, .. } = &mut wrong.assignment.current.records[0].code
    else {
        panic!()
    };
    *boot_directory = "/boot".into();
    assert!(wrong.bound(&d).is_err());
    s.assignment = w::Draft::new(&d.initial);
    assert!(d.checks(&s).iter().any(|c| !c.0));
    assert_eq!(s.practice, example);
}
#[test]
fn unsupported_input_is_educational_and_console_history_stays_bounded() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    run(
        &d.initial,
        &mut s.assignment,
        "grub-install --force /dev/vda",
    );
    assert!(s.assignment.transcript.contains("Unsupported"));
    assert_eq!(s.assignment.command, "grub-install --force /dev/vda");
    assert_eq!(s.assignment.current, d.initial.initial);
    for _ in 0..100 {
        run(
            &d.initial,
            &mut s.assignment,
            "cat /target/boot/grub/grub.cfg",
        );
    }
    assert!(s.assignment.transcript.len() <= 8192);
    assert!(
        s.assignment
            .transcript
            .starts_with("Earlier console output omitted.")
    );
    assert!(d.checks(&s).iter().any(|c| !c.0));
}

#[test]
fn early_image_failure_does_not_call_correct_root_and_target_settings_wrong() {
    let mut d = definition();
    let mut world = d.initial.initial.clone();
    world
        .execute(&d.initial.facts, &d.initial.reference[0])
        .unwrap();
    d.initial.facts.directories[0]
        .configuration
        .as_mut()
        .unwrap()
        .inventory
        .initramfs[0]
        .drivers
        .clear();
    world.test(&d.initial.facts).unwrap();
    let checks = d.initial.checks(&world);
    assert!(
        checks
            .iter()
            .any(|(ok, msg)| !*ok && msg.contains("Root filesystem"))
    );
    assert!(
        checks
            .iter()
            .any(|(ok, msg)| *ok && msg.contains("Configured root=UUID="))
    );
    assert!(
        checks
            .iter()
            .any(|(ok, msg)| *ok && msg.contains("configured startup request"))
    );
    assert!(
        checks
            .iter()
            .any(|(ok, msg)| !*ok && msg.contains("complete modeled init handoff"))
    );
}
