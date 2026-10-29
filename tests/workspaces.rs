use dlms_learning_lab::{
    engine::{self, State},
    ipv4, pack,
    progress::{self, Attempt, Progress},
    workspace::{self, WorkspaceData},
};
#[test]
fn legacy_completed_subnet_is_upgraded_without_losing_credit_and_backup_is_exact() {
    let labs = pack::bundled().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = Attempt::new(&labs[0], 7);
    if let State::Subnet {
        answers, second, ..
    } = &mut a.state
    {
        *answers = [
            "192.168.10.64",
            "192.168.10.95",
            "192.168.10.65",
            "192.168.10.94",
        ]
        .map(String::from);
        *second = None;
    }
    a.step = 2;
    a.challenge = true;
    a.hints = 2;
    a.transcript = vec!["synthetic legacy transcript".into()];
    p.favorites.insert(labs[0].id.clone());
    p.attempts.insert(labs[0].id.clone(), a);
    progress::save(&path, &p).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mut q = progress::load(&path, &labs).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        original,
        "load must not rewrite legacy data"
    );
    let a = q.attempts.get_mut(&labs[0].id).unwrap();
    let before = ipv4::Exercise::from_state(&a.state).unwrap();
    workspace::prepare(a);
    let after = ipv4::Exercise::from_state(&a.state).unwrap();
    assert_eq!(before.answers, after.answers);
    assert!(!after.solved());
    assert_eq!(after.second.as_deref(), Some(""));
    assert!(progress::legacy_credit(a));
    assert!(!progress::current_solved(a, &labs[0]));
    assert_eq!(a.step, 2);
    assert_eq!(a.hints, 2);
    assert!(a.challenge);
    assert_eq!(a.transcript, ["synthetic legacy transcript"]);
    assert!(a.workspace.as_ref().unwrap().migrated_host);
    progress::save(&path, &q).unwrap();
    let backup = path.with_extension("before-workspace.json");
    assert_eq!(std::fs::read(&backup).unwrap(), original);
    let q = progress::load(&path, &labs).unwrap();
    assert!(progress::legacy_credit(&q.attempts[&labs[0].id]));
    assert!(!progress::current_solved(
        &q.attempts[&labs[0].id],
        &labs[0]
    ));
    assert!(q.favorites.contains(&labs[0].id));
    progress::save(&path, &q).unwrap();
    assert_eq!(
        std::fs::read(backup).unwrap(),
        original,
        "later writes cannot replace the compatibility backup"
    );
}
#[test]
fn partial_legacy_answers_and_practice_are_retained_without_fabricating_success() {
    let labs = pack::bundled().unwrap();
    let mut a = Attempt::new(&labs[0], 1);
    if let State::Subnet {
        second, answers, ..
    } = &mut a.state
    {
        *second = None;
        answers[0] = "192.168.10.".into();
    }
    let mut e = ipv4::Exercise::from_state(&a.state).unwrap();
    let mut l = ipv4::Lesson::from_legacy(e.ip, e.prefix, 1);
    e.ip = "10.8.7.77".parse().unwrap();
    e.prefix = 23;
    e.answers = e.expected().unwrap().map(|a| a.to_string());
    l.variant = Some(e);
    l.use_variant = true;
    l.sandbox_host = 1;
    a.ipv4 = Some(l);
    workspace::prepare(&mut a);
    assert!(
        matches!(&a.state,State::Subnet{answers,second:Some(second),..}if answers[0]=="192.168.10."&&second.is_empty())
    );
    assert!(!engine::solved(&a.state, &labs[0].goal));
    let v = a.ipv4.as_ref().unwrap().variant.as_ref().unwrap();
    assert!(!v.solved());
    assert_eq!(v.second.as_deref(), Some(""));
    assert!(a.workspace.as_ref().unwrap().migrated_practice_host);
}
#[test]
fn every_free_host_is_accepted_and_reserved_duplicate_outside_or_malformed_hosts_fail() {
    let labs = pack::bundled().unwrap();
    let mut state = labs[0].initial.clone();
    for c in &labs[0].solution {
        engine::execute(&mut state, c).unwrap();
    }
    for last in 0..=255 {
        if let State::Subnet { second, .. } = &mut state {
            *second = Some(format!("192.168.10.{last}"));
        }
        assert_eq!(
            engine::solved(&state, &labs[0].goal),
            (65..=94).contains(&last) && last != 77,
            "last octet {last}"
        );
    }
    for second in ["", "bad", "192.168.10.999", "192.168.11.80"] {
        if let State::Subnet { second: value, .. } = &mut state {
            *value = Some(second.into());
        }
        assert!(!engine::solved(&state, &labs[0].goal));
    }
    assert_eq!(
        ipv4::second_result("192.0.2.1".parse().unwrap(), 30, "192.0.2.2"),
        ipv4::SecondResult::Valid
    );
    assert_eq!(
        ipv4::second_result("10.5.7.77".parse().unwrap(), 23, "10.5.6.1"),
        ipv4::SecondResult::Valid
    );
    for prefix in [31, 32] {
        assert_ne!(
            ipv4::second_result("192.0.2.1".parse().unwrap(), prefix, "192.0.2.2"),
            ipv4::SecondResult::Valid
        );
    }
}
#[test]
fn versioned_workspace_drafts_roundtrip_and_invalid_metadata_preserves_original() {
    let labs = pack::bundled().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut original_drafts = 0;
    let mut backup_drafts = 0;
    let mut physical_drafts = 0;
    let mut private_drafts = 0;
    let mut one_off_drafts = 0;
    let mut lvm_drafts = 0;
    for (n, s) in labs.iter().enumerate() {
        let mut a = Attempt::new(s, n as u64);
        workspace::prepare(&mut a);
        if matches!(a.state, State::Cron { .. }) {
            dlms_learning_lab::cron_workspace::budget::prepare(&mut a, s).unwrap();
        }
        if let Some(lesson) = &s.file_backup_teaching {
            // Preserve this real new-family positive in the cross-family matrix.
            dlms_learning_lab::file_backup_teaching::prepare(&mut a, lesson, true).unwrap();
        }
        if matches!(a.state, State::OneOff { .. }) {
            use dlms_learning_lab::one_off_workspace::{self as one, Action};
            one::budget::prepare(&mut a, s).unwrap();
            let w = a.workspace.as_ref().unwrap();
            let assignment_path = w.one_off_session.as_ref().unwrap().selected.clone();
            let practice_path = w
                .one_off_learning
                .as_ref()
                .unwrap()
                .session
                .selected
                .clone();
            one::budget::apply_attempt(
                &mut a,
                s,
                &Action::Edit {
                    path: assignment_path,
                    text: "unfinished assignment script |".into(),
                },
                false,
            )
            .unwrap();
            one::budget::apply_attempt(
                &mut a,
                s,
                &Action::Edit {
                    path: practice_path,
                    text: "separate unfinished example |".into(),
                },
                true,
            )
            .unwrap();
            one_off_drafts += 1;
            p.attempts.insert(s.id.clone(), a);
            continue;
        }
        if let (State::Lvm { simulation }, engine::Goal::Lvm { definition }) =
            (&mut a.state, &s.goal)
        {
            simulation
                .set_command(definition, "unfinished LVM assignment |".into(), false)
                .unwrap();
            simulation
                .set_command(definition, "separate unfinished LVM example |".into(), true)
                .unwrap();
            dlms_learning_lab::lvm_workspace::context(&a, s).unwrap();
            lvm_drafts += 1;
            p.attempts.insert(s.id.clone(), a);
            continue;
        }
        if matches!(a.state, State::PhysicalLink { .. }) {
            use dlms_learning_lab::physical_link_workspace as physical;
            physical::prepare_attempt(&mut a, s, true).unwrap();
            let mut draft = a
                .workspace
                .as_ref()
                .unwrap()
                .physical_link_draft
                .as_ref()
                .unwrap()
                .clone();
            let port = draft.configuration.ports.values_mut().next().unwrap();
            port.admin_up = !port.admin_up;
            physical::set_draft(&mut a, s, physical::Scope::Assignment, &draft).unwrap();
            physical_drafts += 1;
            p.attempts.insert(s.id.clone(), a);
            continue;
        }
        if s.id == "ccna-private-office-plan" {
            let draft = a
                .workspace
                .as_mut()
                .unwrap()
                .routing_v4_draft
                .as_mut()
                .unwrap();
            draft
                .devices
                .get_mut("Alder")
                .unwrap()
                .interfaces
                .get_mut("lan")
                .unwrap()
                .address = "unfinished office draft".into();
            private_drafts += 1;
            p.attempts.insert(s.id.clone(), a);
            continue;
        }
        let w = a.workspace.as_mut().unwrap();
        w.command = "unfinished |".into();
        if s.file_backup_teaching.is_some() {
            // New backup work has its own isolated draft, not another activity's
            // service/DNS/mount fields. Keep every original164 case unchanged.
            w.file_backup_learning.as_mut().unwrap().command = "tar --unfinished".into();
            backup_drafts += 1;
        } else {
            w.service_port = Some("unfinished".into());
            w.dns_address = Some("192.0.".into());
            w.dns_serial = Some("draft".into());
            w.mount_device = Some("/dev/vd".into());
            original_drafts += 1;
        }
        p.attempts.insert(s.id.clone(), a);
    }
    assert_eq!((original_drafts, backup_drafts), (169, 4));
    assert_eq!(
        (original_drafts - 2, backup_drafts),
        (167, 4),
        "All original167generic and4backup drafts retained"
    );
    assert_eq!(physical_drafts, 4);
    assert_eq!(private_drafts, 1);
    assert_eq!(one_off_drafts, 4);
    assert_eq!(lvm_drafts, 4);
    assert_eq!(p.attempts.len(), 186);
    assert_eq!(
        labs[176..180]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "lpic1-at-report",
            "lpic1-at-capture-repair",
            "lpic1-at-owner-queue",
            "lpic1-at-handover"
        ]
    );
    assert_eq!(
        labs[168..171]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "shell-numeric-retry",
            "shell-numeric-intake",
            "shell-numeric-publication"
        ],
        "Only the three new numeric drafts extend the prior164 non-backup drafts"
    );
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    let q = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&q).unwrap()
    );
    let valid_raw = std::fs::read(&path).unwrap();
    for source in labs
        .iter()
        .filter(|s| matches!(s.initial, State::Lvm { .. }))
    {
        let a = &q.attempts[&source.id];
        let State::Lvm { simulation } = &a.state else {
            panic!()
        };
        assert_eq!(simulation.command, "unfinished LVM assignment |");
        assert_eq!(
            simulation.example_command,
            "separate unfinished LVM example |"
        );
        assert_eq!(simulation.current, simulation.initial);
        assert_eq!(simulation.example, simulation.example_initial);
        assert!(!progress::current_solved(a, source));
        for field in 0..4 {
            let mut foreign = progress::decode(&valid_raw, &labs).unwrap();
            let w = foreign
                .attempts
                .get_mut(&source.id)
                .unwrap()
                .workspace
                .as_mut()
                .unwrap();
            match field {
                0 => w.service_port = Some("unfinished".into()),
                1 => w.dns_address = Some("192.0.".into()),
                2 => w.dns_serial = Some("draft".into()),
                _ => w.mount_device = Some("/dev/vd".into()),
            }
            let before = serde_json::to_value(&foreign).unwrap();
            assert!(progress::save_with_catalog(&path, &foreign, &labs).is_err());
            assert_eq!(std::fs::read(&path).unwrap(), valid_raw);
            assert_eq!(serde_json::to_value(&foreign).unwrap(), before);
        }
    }

    for s in &labs[176..180] {
        let a = &q.attempts[&s.id];
        let w = a.workspace.as_ref().unwrap();
        let own = w.one_off_session.as_ref().unwrap();
        assert_eq!(own.drafts[&own.selected], "unfinished assignment script |");
        let example = &w.one_off_learning.as_ref().unwrap().session;
        assert_eq!(
            example.drafts[&example.selected],
            "separate unfinished example |"
        );
        assert!(!progress::current_solved(a, s));
        assert!(
            w.command.is_empty()
                && w.service_port.is_none()
                && w.dns_address.is_none()
                && w.dns_serial.is_none()
                && w.mount_device.is_none()
        );
        for field in 0..4 {
            let mut foreign = progress::decode(&valid_raw, &labs).unwrap();
            let w = foreign
                .attempts
                .get_mut(&s.id)
                .unwrap()
                .workspace
                .as_mut()
                .unwrap();
            match field {
                0 => w.service_port = Some("unfinished".into()),
                1 => w.dns_address = Some("192.0.".into()),
                2 => w.dns_serial = Some("draft".into()),
                3 => w.mount_device = Some("/dev/vd".into()),
                _ => unreachable!(),
            }
            assert!(
                progress::save_with_catalog(&path, &foreign, &labs).is_err(),
                "Foreign OneOff field {field}"
            );
            assert_eq!(std::fs::read(&path).unwrap(), valid_raw);
        }
    }

    for s in labs.iter().filter(|s| s.file_backup_teaching.is_some()) {
        let a = &q.attempts[&s.id];
        assert_eq!(a.workspace.as_ref().unwrap().command, "unfinished |");
        assert_eq!(
            a.workspace
                .as_ref()
                .unwrap()
                .file_backup_learning
                .as_ref()
                .unwrap()
                .command,
            "tar --unfinished"
        );
        assert!(!progress::current_solved(a, s));
        for field in 0..4 {
            let mut foreign = progress::decode(&valid_raw, &labs).unwrap();
            let w = foreign
                .attempts
                .get_mut(&s.id)
                .unwrap()
                .workspace
                .as_mut()
                .unwrap();
            match field {
                0 => w.service_port = Some("unfinished".into()),
                1 => w.dns_address = Some("192.0.".into()),
                2 => w.dns_serial = Some("draft".into()),
                3 => w.mount_device = Some("/dev/vd".into()),
                _ => unreachable!(),
            }
            assert!(
                progress::save_with_catalog(&path, &foreign, &labs).is_err(),
                "Foreign backup field {field}"
            );
            assert_eq!(
                std::fs::read(&path).unwrap(),
                valid_raw,
                "Refusal preserves the complete valid file"
            );
        }
    }
    p.attempts
        .get_mut(&labs[0].id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .version = 99;
    let retained = std::fs::read(&path).unwrap();
    assert!(progress::save_with_catalog(&path, &p, &labs).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), retained);
    std::fs::write(&path, serde_json::to_vec(&p).unwrap()).unwrap();
    let original = std::fs::read(&path).unwrap();
    assert!(progress::load(&path, &labs).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert!(
        !WorkspaceData {
            version: 1,
            command: "x".repeat(4097),
            ..Default::default()
        }
        .valid()
    );
}
#[test]
fn dns_edits_are_atomic_and_mount_editor_changes_only_persistent_configuration() {
    let labs = pack::bundled().unwrap();
    let mut dns = labs[7].initial.clone();
    let before = dns.clone();
    assert!(workspace::edit_dns(&mut dns, "192.0.2.80", "word").is_err());
    assert_eq!(dns, before);
    assert!(workspace::edit_dns(&mut dns, "bad", "2026102102").is_err());
    assert_eq!(dns, before);
    workspace::edit_dns(&mut dns, "192.0.2.80", "2026102102").unwrap();
    assert!(!engine::solved(&dns, &labs[7].goal));
    assert!(
        workspace::issues(&dns, &labs[7].goal)
            .iter()
            .any(|s| s.contains("not the loaded"))
    );
    engine::execute(&mut dns, "systemctl reload named").unwrap();
    assert!(engine::solved(&dns, &labs[7].goal));
    let mut mount = labs[8].initial.clone();
    let original = mount.clone();
    assert!(workspace::edit_mount(&mut mount, "/tmp/host").is_err());
    assert_eq!(mount, original);
    workspace::edit_mount(&mut mount, "/dev/vdb1").unwrap();
    assert!(matches!(
        mount,
        State::Mount {
            used: 950,
            writable: false,
            ..
        }
    ));
    assert!(!engine::solved(&mount, &labs[8].goal));
}
#[test]
fn vlsm_feedback_distinguishes_blank_host_boundary_capacity_overlap_and_accepts_alternates() {
    let labs = pack::bundled().unwrap();
    let mut s = labs[1].initial.clone();
    assert!(workspace::vlsm_issue(&s, 0).unwrap().contains("Enter"));
    let set = |s: &mut State, index: usize, value: &str| {
        if let State::Vlsm { allocations, .. } = s {
            allocations[index].cidr = value.into();
        }
    };
    for (value, word) in [
        ("bad", "IPv4 network"),
        ("10.20.0.1/26", "host address"),
        ("10.20.0.0/28", "Too small"),
        ("10.20.1.0/26", "outside"),
    ] {
        set(&mut s, 0, value);
        assert!(workspace::vlsm_issue(&s, 0).unwrap().contains(word));
    }
    set(&mut s, 0, "10.20.0.128/26");
    set(&mut s, 1, "10.20.0.128/27");
    assert!(workspace::vlsm_issue(&s, 0).unwrap().contains("overlaps"));
    set(&mut s, 1, "10.20.0.0/27");
    set(&mut s, 2, "10.20.0.32/28");
    assert!(engine::solved(&s, &labs[1].goal));
    for i in 0..3 {
        assert!(workspace::vlsm_issue(&s, i).is_none());
    }
}

#[test]
fn switching_from_wider_practice_subnets_keeps_the_original_and_save_valid() {
    let labs = pack::bundled().unwrap();
    let mut a = Attempt::new(&labs[0], 1);
    for c in &labs[0].solution {
        engine::execute(&mut a.state, c).unwrap();
    }
    workspace::prepare(&mut a);
    let original = a.state.clone();
    for _ in 0..12 {
        workspace::start_practice(&mut a);
        let l = a.ipv4.as_mut().unwrap();
        l.sandbox_host = !ipv4::mask(l.variant.as_ref().unwrap().prefix).unwrap();
        workspace::select_original(&mut a);
        assert_eq!(a.state, original);
        assert!(ipv4::validate_lesson(
            a.ipv4.as_ref().unwrap(),
            &ipv4::Exercise::from_state(&a.state).unwrap()
        ));
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(labs[0].id.clone(), a);
    progress::save(&path, &p).unwrap();
    assert!(progress::load(&path, &labs).is_ok());
}
