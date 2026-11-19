use dlms_learning_lab::{
    engine::{self, State},
    magic_subnet::{Problem, Session, Stage, checks},
    pack, progress,
};
use std::net::Ipv4Addr;
fn p(ip: &str, prefix: u8) -> Problem {
    Problem {
        ip: ip.parse().unwrap(),
        prefix,
    }
}
#[test]
fn worked_example_and_cross_octet_and_aligned_boundaries_are_exact() {
    for (ip, prefix, net, bc, next, octet, block) in [
        (
            "192.168.10.77",
            26,
            "192.168.10.64",
            "192.168.10.127",
            "192.168.10.128",
            3,
            64,
        ),
        (
            "172.19.70.9",
            20,
            "172.19.64.0",
            "172.19.79.255",
            "172.19.80.0",
            2,
            16,
        ),
        (
            "10.24.9.254",
            24,
            "10.24.9.0",
            "10.24.9.255",
            "10.24.10.0",
            2,
            1,
        ),
        (
            "10.35.255.255",
            16,
            "10.35.0.0",
            "10.35.255.255",
            "10.36.0.0",
            1,
            1,
        ),
        (
            "192.168.255.255",
            24,
            "192.168.255.0",
            "192.168.255.255",
            "192.169.0.0",
            2,
            1,
        ),
    ] {
        let c = p(ip, prefix).calculate().unwrap();
        assert_eq!(c.network.to_string(), net);
        assert_eq!(c.broadcast.to_string(), bc);
        assert_eq!(c.next.unwrap().to_string(), next);
        assert_eq!((c.octet, c.block), (octet, block));
    }
    assert!(p("255.255.255.255", 30).calculate().unwrap().next.is_none());
    for prefix in [0, 31, 32, 33, 255] {
        assert!(p("192.168.1.1", prefix).calculate().is_err());
    }
}
#[test]
fn every_supported_prefix_contains_boundary_hosts_and_matches_independent_integer_blocks() {
    for prefix in 1..=30 {
        let span = 1u64 << (32 - prefix);
        for address in [
            0u32,
            1,
            63,
            64,
            77,
            127,
            128,
            255,
            256,
            0xac134609,
            u32::MAX,
        ] {
            let c = Problem {
                ip: Ipv4Addr::from(address),
                prefix,
            }
            .calculate()
            .unwrap();
            let n = u64::from(address) / span * span;
            let b = n + span - 1;
            assert_eq!(u64::from(u32::from(c.network)), n);
            assert_eq!(u64::from(u32::from(c.broadcast)), b);
            assert_eq!(u32::from(c.first) as u64, n + 1);
            assert_eq!(u32::from(c.last) as u64, b - 1);
            assert_eq!(
                c.next.map(|v| u64::from(u32::from(v))),
                (b < u32::MAX as u64).then_some(b + 1)
            );
            assert!(
                c.start
                    <= Problem {
                        ip: address.into(),
                        prefix
                    }
                    .ip
                    .octets()[c.octet]
            );
        }
    }
}
#[test]
fn specific_feedback_partial_answers_alternatives_and_success_invalidation() {
    let problem = p("192.168.10.77", 26);
    let mut a: [String; 6] = Default::default();
    assert!(
        checks(Stage::Boundaries, problem, &a)
            .iter()
            .all(|v| !v.0 && v.1.contains("blank"))
    );
    a[0] = "not-an-address".into();
    assert!(
        checks(Stage::Boundaries, problem, &a)[0]
            .1
            .contains("four decimal")
    );
    a[0] = "192.168.10.77".into();
    assert!(
        checks(Stage::Boundaries, problem, &a)[0]
            .1
            .contains("host inside")
    );
    a[0] = "192.168.10.65".into();
    assert!(
        checks(Stage::Boundaries, problem, &a)[0]
            .1
            .contains("block boundary")
    );
    for (i, f) in Stage::Boundaries
        .fields(problem)
        .unwrap()
        .iter()
        .enumerate()
    {
        a[i] = format!(" {} ", f.expected);
    }
    assert!(checks(Stage::Boundaries, problem, &a).iter().all(|v| v.0));
    a[2] = "192.168.10.64".into();
    let c = checks(Stage::Boundaries, problem, &a);
    assert_eq!(c.iter().filter(|v| v.0).count(), 4);
    assert!(c[2].1.contains("Reserve the network"));
    let mut b: [String; 6] = Default::default();
    b[2] = "192".into();
    assert!(checks(Stage::Blocks, problem, &b)[2].1.contains("Subtract"));
}
#[test]
fn all_old_ids_survive_new_tasks_grade_real_state_and_example_never_earns_credit() {
    let all = pack::accepted209().unwrap();
    assert_eq!(all.len(), 209);
    assert!(all[..201].iter().all(|s| !s.id.starts_with("ccna-magic-")));
    for s in &all[201..205] {
        let mut a = progress::Attempt::new(s, 1);
        assert!(!engine::solved(&a.state, &s.goal));
        if let State::MagicSubnet { session } = &mut a.state {
            let ex = session.example_problem().unwrap();
            for (i, f) in session.stage.fields(ex).unwrap().iter().enumerate() {
                session.example.answers[i] = f.expected.clone();
            }
        }
        assert!(!engine::solved(&a.state, &s.goal));
        for cmd in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, cmd).unwrap();
        }
        assert!(engine::solved(&a.state, &s.goal));
        let raw = serde_json::to_string(&a).unwrap();
        let mut restored: progress::Attempt = serde_json::from_str(&raw).unwrap();
        assert_eq!(serde_json::to_string(&restored).unwrap(), raw);
        if let State::MagicSubnet { session } = &mut restored.state {
            session.answers[0] = "wrong".into();
        }
        assert!(!engine::solved(&restored.state, &s.goal));
        if let State::MagicSubnet { session } = &mut a.state {
            session.problem.prefix = 30;
        }
        assert!(!engine::solved(&a.state, &s.goal));
        let reset = progress::Attempt::new(s, 2);
        assert!(!engine::solved(&reset.state, &s.goal));
    }
}
#[test]
fn pilot_progress_copy_is_private_once_only_and_preserves_the_original_bytes() {
    let d = tempfile::tempdir().unwrap();
    let original = d.path().join("progress-v1.json");
    std::fs::write(&original, b"original-progress-bytes").unwrap();
    std::fs::write(d.path().join("preferences-v1.json"), b"original-prefs").unwrap();
    let path = progress::pilot_profile(&original).unwrap();
    assert_ne!(path, original);
    assert_eq!(std::fs::read(&path).unwrap(), b"original-progress-bytes");
    std::fs::write(&path, b"separate-pilot-work").unwrap();
    progress::pilot_profile(&original).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"separate-pilot-work");
    assert_eq!(
        std::fs::read(&original).unwrap(),
        b"original-progress-bytes"
    );
}
#[test]
fn example_drafts_and_checks_survive_without_requiring_valid_partial_typing() {
    let mut s = Session::new(p("10.0.0.77", 26), Stage::Blocks);
    s.example.address = "192.168.".into();
    s.example.prefix = "/".into();
    s.example_checked = Some(s.example.clone());
    assert!(s.validate().is_ok());
    assert!(s.example_problem().is_err());
    s.example.address="é".repeat(64);assert!(s.validate().is_ok());assert!(s.example_problem().is_err());
    let raw = serde_json::to_string(&s).unwrap();
    assert_eq!(serde_json::from_str::<Session>(&raw).unwrap(), s);
}
#[test]
fn saved_pilot_attempts_resume_through_normal_loader_and_reject_changed_problem() {
    let all = pack::accepted209().unwrap();
    let s = &all[201];
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = progress::Attempt::new(s, 1);
    if let State::MagicSubnet { session } = &mut a.state {
        session.answers[2] = "32".into();
        session.example.address = "198.51.".into();
    }
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    let bytes = serde_json::to_vec(&p).unwrap();
    let resumed = progress::decode(&bytes, &all).unwrap();
    assert_eq!(serde_json::to_vec(&resumed).unwrap(), bytes);
    let a = p.attempts.get_mut(&s.id).unwrap();
    for cmd in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, cmd).unwrap();
    }
    assert!(
        !progress::current_solved(a, s),
        "unchecked values do not claim checked completion"
    );
    if let State::MagicSubnet { session } = &mut a.state {
        session.checked = Some(session.answers.clone());
    }
    assert!(progress::current_solved(a, s));
    if let State::MagicSubnet { session } = &mut a.state {
        session.problem.prefix = 26;
    }
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &all).is_err());
}
#[test]
fn pilot_extends_but_does_not_rewrite_normal_catalog_or_unit_objects() {
    let normal = pack::bundled().unwrap();
    let pilot = pack::accepted209().unwrap();
    assert_eq!(normal.len(), 201);
    assert_eq!(pilot.len(), 209);
    assert_eq!(
        serde_json::to_value(&pilot[..201]).unwrap(),
        serde_json::to_value(&normal).unwrap()
    );
    let original = dlms_learning_lab::course::bundled(&normal).unwrap();
    let units = dlms_learning_lab::course::accepted209(&pilot).unwrap();
    assert_eq!(units[0].id, "ccna-magic-number");
    assert_eq!(units[0].labs.len(), 4);
    assert_eq!(
        serde_json::to_value(&units[1..55]).unwrap(),
        serde_json::to_value(&original).unwrap()
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| pilot.iter().filter(|s| s.track == t).count()),
        [65, 44, 36, 64]
    );
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/ccna-magic-number-v1.json")).unwrap();
    v["scenarios"][0]["initial"]["session"]["problem"]["prefix"] = 31.into();
    assert!(pack::load(&v.to_string()).is_err());
}
