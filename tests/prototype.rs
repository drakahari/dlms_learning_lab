use dlms_learning_lab::{
    catalog::{self, Filter},
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
};
fn scenario(id: &str) -> pack::Scenario {
    pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.id == id)
        .unwrap()
}
fn solution(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for cmd in &s.solution {
        engine::execute(&mut state, cmd).unwrap();
    }
    state
}
#[test]
fn all_nine_start_unsolved_and_reach_success() {
    let labs = pack::bundled().unwrap();
    assert_eq!(labs.len(), 9);
    for s in labs {
        assert!(!engine::solved(&s.initial, &s.goal), "{}", s.id);
        assert!(engine::solved(&solution(&s), &s.goal), "{}", s.id);
    }
}
#[test]
fn success_is_recomputed_after_every_repair_is_broken() {
    let breaks = [
        ("ccna-ipv4", "answer first 192.168.10.64"),
        ("ccna-vlsm", "allocate lab 10.20.0.0/28"),
        ("ccna-gateway", "ip route replace default via 192.168.40.1"),
        ("lpic1-permissions", "chmod 600 /srv/report.txt"),
        ("lpic1-traversal", "chmod 740 /srv/project"),
        ("lpic1-logs", "cat /var/log/app.log"),
        ("lpic2-service", "systemctl stop atlas"),
        ("lpic2-dns", "dns set portal.lab.test 192.0.2.1"),
        ("lpic2-storage", "umount /data"),
    ];
    for (id, cmd) in breaks {
        let s = scenario(id);
        let mut state = solution(&s);
        engine::execute(&mut state, cmd).unwrap();
        assert!(!engine::solved(&state, &s.goal), "{id}");
    }
}
#[test]
fn subnet_math_handles_every_prefix_without_overflow() {
    for p in 0..=32 {
        let (n, b) = engine::bounds("255.255.255.255".parse().unwrap(), p).unwrap();
        assert!(n <= b);
        assert_eq!((b as u64) - (n as u64) + 1, 1u64 << (32 - p));
    }
    assert!(engine::bounds("0.0.0.0".parse().unwrap(), 33).is_none());
    assert!(engine::cidr("10.0.0.7/24").is_none());
    assert_eq!(
        engine::bounds("192.168.10.77".parse().unwrap(), 27),
        Some((0xc0a80a40, 0xc0a80a5f))
    );
}
#[test]
fn alternate_vlsm_layout_and_invalid_capacity_overlap() {
    let s = scenario("ccna-vlsm");
    let mut state = s.initial.clone();
    for c in [
        "allocate design 10.20.0.128/26",
        "allocate support 10.20.0.0/27",
        "allocate lab 10.20.0.32/28",
    ] {
        engine::execute(&mut state, c).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
    engine::execute(&mut state, "allocate support 10.20.0.0/28").unwrap();
    assert!(!engine::solved(&state, &s.goal));
    engine::execute(&mut state, "allocate support 10.20.1.0/27").unwrap();
    assert!(!engine::solved(&state, &s.goal));
    assert!(engine::execute(&mut state, "allocate support 10.20.0.7/27").is_err());
}
#[test]
fn symbolic_permissions_and_ownership_are_valid_alternates() {
    let s = scenario("lpic1-permissions");
    let mut st = s.initial.clone();
    engine::execute(&mut st, "chmod u=rw,g=rw,o= /srv/report.txt").unwrap();
    assert!(engine::solved(&st, &s.goal));
    let mut st = s.initial.clone();
    engine::execute(&mut st, "chown alex:editors /srv/report.txt").unwrap();
    assert!(engine::solved(&st, &s.goal));
    engine::execute(&mut st, "chmod 666 /srv/report.txt").unwrap();
    assert!(!engine::solved(&st, &s.goal));
}
#[test]
fn unix_owner_precedence_and_all_parent_traversal_are_modeled() {
    let s = scenario("lpic1-traversal");
    let mut st = solution(&s);
    engine::execute(&mut st, "chmod 640 /srv/project").unwrap();
    assert!(!engine::solved(&st, &s.goal));
    engine::execute(&mut st, "chmod 750 /srv/project").unwrap();
    engine::execute(&mut st, "chown blair:editors /srv/project/notes.txt").unwrap();
    engine::execute(&mut st, "chmod 040 /srv/project/notes.txt").unwrap();
    assert!(
        !engine::solved(&st, &s.goal),
        "owner cannot fall back to group permission"
    );
}
#[test]
fn service_requires_restart_and_enable_independently() {
    let s = scenario("lpic2-service");
    let mut st = s.initial.clone();
    assert!(engine::execute(&mut st, "systemctl start atlas").is_err());
    engine::execute(&mut st, "systemctl enable atlas").unwrap();
    assert!(!engine::solved(&st, &s.goal));
    engine::execute(&mut st, "config set port 80").unwrap();
    engine::execute(&mut st, "systemctl start atlas").unwrap();
    engine::execute(&mut st, "config set port 8080").unwrap();
    engine::execute(&mut st, "systemctl start atlas").unwrap();
    assert!(!engine::solved(&st, &s.goal));
    engine::execute(&mut st, "systemctl restart atlas").unwrap();
    assert!(engine::solved(&st, &s.goal));
    engine::execute(&mut st, "config set port invalid").unwrap();
    assert!(!engine::solved(&st, &s.goal));
    assert!(engine::execute(&mut st, "systemctl restart atlas").is_err());
}
#[test]
fn dns_edit_serial_reload_and_resolution_are_distinct() {
    let s = scenario("lpic2-dns");
    let mut st = s.initial.clone();
    engine::execute(&mut st, "dns set portal.lab.test 192.0.2.80").unwrap();
    assert!(!engine::solved(&st, &s.goal));
    assert!(engine::execute(&mut st, "systemctl reload named").is_err());
    assert!(
        engine::execute(&mut st, "dig portal.lab.test")
            .unwrap()
            .contains("192.0.2.8")
    );
    engine::execute(&mut st, "dns serial 2026102102").unwrap();
    engine::execute(&mut st, "systemctl reload named").unwrap();
    assert!(engine::solved(&st, &s.goal));
    assert!(engine::execute(&mut st, "dns set portal.lab.test 999.0.0.1").is_err());
}
#[test]
fn storage_requires_writable_mount_and_persistent_device() {
    let s = scenario("lpic2-storage");
    let mut st = s.initial.clone();
    assert!(engine::execute(&mut st, "rm -r /data/cache").is_err());
    assert!(engine::execute(&mut st, "mount /data").is_err());
    engine::execute(&mut st, "mount -o remount,rw /data").unwrap();
    engine::execute(&mut st, "rm -r /data/cache").unwrap();
    assert!(!engine::solved(&st, &s.goal));
    engine::execute(&mut st, "config set device /dev/vdb1").unwrap();
    assert!(engine::solved(&st, &s.goal));
    engine::execute(&mut st, "rm -r /data/cache").unwrap();
    assert!(engine::solved(&st, &s.goal));
    assert!(engine::execute(&mut st, "rm /etc/fstab").is_err());
}
#[test]
fn log_alternates_pipeline_and_limits() {
    let s = scenario("lpic1-logs");
    let mut st = s.initial.clone();
    engine::execute(&mut st, "grep ERROR /var/log/app.log").unwrap();
    assert!(engine::solved(&st, &s.goal));
    engine::execute(&mut st, "cat /var/log/app.log | grep ERROR | tail -n 1").unwrap();
    assert!(!engine::solved(&st, &s.goal));
    assert!(engine::execute(&mut st, "cat /etc/passwd").is_err());
    assert!(engine::execute(&mut st, "cat /var/log/app.log | sh").is_err());
}
#[test]
fn no_shell_escape_or_unsupported_command_succeeds() {
    for s in pack::bundled().unwrap() {
        for c in [
            "sudo whoami",
            "sh -c id",
            "$(touch /tmp/escaped)",
            "help ; id",
            "cat /etc/passwd > /tmp/foo",
            "curl https://example.com",
        ] {
            let mut st = s.initial.clone();
            assert!(engine::execute(&mut st, c).is_err(), "{} {c}", s.id);
            assert_eq!(st, s.initial);
        }
    }
}
#[test]
fn resume_reset_only_selected_and_round_trip_all_states() {
    let labs = pack::bundled().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &labs {
        let mut a = Attempt::new(s, 3);
        a.state = solution(s);
        a.step = 1;
        a.hints = 1;
        a.challenge = true;
        p.attempts.insert(s.id.clone(), a);
    }
    p.favorites.insert(labs[0].id.clone());
    progress::save(&path, &p).unwrap();
    let mut q = progress::load(&path, &labs).unwrap();
    assert_eq!(q.attempts.len(), 9);
    for s in &labs {
        let a = &q.attempts[&s.id];
        assert!(engine::solved(&a.state, &s.goal));
        assert_eq!(a.step, 1);
        assert!(a.challenge);
    }
    q.attempts
        .insert(labs[0].id.clone(), Attempt::new(&labs[0], 4));
    progress::save(&path, &q).unwrap();
    let q = progress::load(&path, &labs).unwrap();
    assert!(!engine::solved(
        &q.attempts[&labs[0].id].state,
        &labs[0].goal
    ));
    assert!(engine::solved(
        &q.attempts[&labs[1].id].state,
        &labs[1].goal
    ));
    assert!(q.favorites.contains(&labs[0].id));
}
#[test]
fn corrupt_or_future_progress_is_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("p.json");
    for data in [
        "not json",
        r#"{"schema_version":999,"clock":0,"favorites":[],"attempts":{}}"#,
    ] {
        std::fs::write(&path, data).unwrap();
        assert!(progress::load(&path, &pack::bundled().unwrap()).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), data);
    }
}
#[test]
fn packs_reject_bad_schema_ids_metadata_state_and_unreachable_solutions() {
    let source = include_str!("../scenarios/starter-v1.json");
    let base: serde_json::Value = serde_json::from_str(source).unwrap();
    for (pointer, value) in [
        ("/schema_version", serde_json::json!(99)),
        ("/scenarios/0/id", serde_json::json!("../escape")),
        ("/scenarios/1/id", serde_json::json!("ccna-ipv4")),
        ("/scenarios/0/hints", serde_json::json!([])),
        ("/scenarios/0/initial/prefix", serde_json::json!(33)),
        ("/scenarios/0/solution", serde_json::json!(["show"])),
        ("/scenarios/0/goal/kind", serde_json::json!("Mount")),
        (
            "/scenarios/0/sources",
            serde_json::json!(["http://untrusted"]),
        ),
        ("/scenarios/0/initial/extra", serde_json::json!(true)),
    ] {
        let mut v = base.clone();
        if pointer.ends_with("extra") {
            v["scenarios"][0]["initial"]["extra"] = value;
        } else {
            *v.pointer_mut(pointer).unwrap() = value;
        }
        assert!(pack::load(&v.to_string()).is_err(), "{pointer}");
    }
}
#[test]
fn catalog_900_filters_and_synthetic_boundary() {
    let labs = pack::bundled().unwrap();
    let large = catalog::synthetic_catalog(&labs, 900);
    assert_eq!(large.len(), 900);
    assert!(
        large
            .iter()
            .all(|s| s.id.starts_with("synthetic-") && s.title.starts_with("SYNTHETIC"))
    );
    let mut f = Filter {
        track: "CCNA".into(),
        ..Default::default()
    };
    assert_eq!(
        large
            .iter()
            .filter(|s| catalog::matches(s, &f, false, false, false))
            .count(),
        300
    );
    f.search = "boundaries".into();
    f.max_minutes = 6;
    assert_eq!(
        large
            .iter()
            .filter(|s| catalog::matches(s, &f, false, false, false))
            .count(),
        100
    );
    f.favorites = true;
    assert_eq!(
        large
            .iter()
            .filter(|s| catalog::matches(s, &f, false, false, false))
            .count(),
        0
    );
    f.favorites = false;
    f.status = "Complete".into();
    assert_eq!(
        large
            .iter()
            .filter(|s| catalog::matches(s, &f, true, true, false))
            .count(),
        100
    );
}

#[test]
fn broken_attempts_are_resumable_instead_of_becoming_invalid_saves() {
    let labs = pack::bundled().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("p.json");
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &labs {
        let mut a = Attempt::new(s, 0);
        a.state = solution(s);
        let cmd = match s.id.as_str() {
            "ccna-ipv4" => "answer last 0.0.0.0",
            "ccna-vlsm" => "allocate design 10.21.0.0/26",
            "ccna-gateway" => "ip addr set 192.168.40.65/24",
            "lpic1-permissions" => "chmod 000 /srv/report.txt",
            "lpic1-traversal" => "chmod 000 /srv/project",
            "lpic1-logs" => "grep INFO /var/log/app.log",
            "lpic2-service" => "config set port broken",
            "lpic2-dns" => "dns serial 1",
            _ => "config set device invalid-device",
        };
        engine::execute(&mut a.state, cmd).unwrap();
        assert!(!engine::solved(&a.state, &s.goal));
        p.attempts.insert(s.id.clone(), a);
    }
    progress::save(&path, &p).unwrap();
    let q = progress::load(&path, &labs).unwrap();
    for s in &labs {
        assert!(!engine::solved(&q.attempts[&s.id].state, &s.goal));
    }
}
