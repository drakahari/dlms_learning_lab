use dlms_learning_lab::{
    engine::{self, State},
    ipv4::{self, Diagnosis, Exercise, Lesson, Section},
    pack,
    progress::{self, Attempt, Progress},
};
fn exercise(ip: &str, prefix: u8) -> Exercise {
    Exercise {
        ip: ip.parse().unwrap(),
        prefix,
        answers: std::array::from_fn(|_| String::new()),
        second: None,
    }
}
#[test]
fn conventional_results_cross_octets_and_handle_prefix_extremes() {
    for (ip, prefix, want) in [
        (
            "192.168.10.77",
            27,
            [
                "192.168.10.64",
                "192.168.10.95",
                "192.168.10.65",
                "192.168.10.94",
            ],
        ),
        (
            "10.12.255.254",
            23,
            [
                "10.12.254.0",
                "10.12.255.255",
                "10.12.254.1",
                "10.12.255.254",
            ],
        ),
        (
            "172.20.31.254",
            20,
            [
                "172.20.16.0",
                "172.20.31.255",
                "172.20.16.1",
                "172.20.31.254",
            ],
        ),
        (
            "192.0.2.129",
            25,
            ["192.0.2.128", "192.0.2.255", "192.0.2.129", "192.0.2.254"],
        ),
        (
            "10.255.0.1",
            8,
            ["10.0.0.0", "10.255.255.255", "10.0.0.1", "10.255.255.254"],
        ),
        (
            "255.255.255.254",
            30,
            [
                "255.255.255.252",
                "255.255.255.255",
                "255.255.255.253",
                "255.255.255.254",
            ],
        ),
        (
            "1.2.3.4",
            0,
            ["0.0.0.0", "255.255.255.255", "0.0.0.1", "255.255.255.254"],
        ),
    ] {
        let e = exercise(ip, prefix);
        assert_eq!(
            e.expected().unwrap().map(|a| a.to_string()),
            want.map(String::from)
        );
    }
    for p in [31, 32, 33, 255] {
        assert!(exercise("1.2.3.4", p).expected().is_none());
        assert!(ipv4::mask(p).is_none());
    }
}
#[test]
fn every_supported_prefix_preserves_network_bits_in_sandbox() {
    for p in 0..=30 {
        let e = exercise("172.19.255.254", p);
        let m = ipv4::mask(p).unwrap();
        let original = u32::from(e.ip);
        let host = original & !m;
        assert_eq!(
            u32::from(ipv4::sandbox_address(&e, 0).unwrap()),
            original & m
        );
        assert_eq!(
            u32::from(ipv4::sandbox_address(&e, !m).unwrap()),
            original | !m
        );
        for bit in 0..32 {
            let changed = ipv4::toggle_host(&e, host, bit);
            if bit < p {
                assert!(changed.is_none());
            } else {
                let value = u32::from(ipv4::sandbox_address(&e, changed.unwrap()).unwrap());
                assert_eq!(value & m, original & m);
                assert_eq!(value ^ original, 1_u32 << (31 - bit));
            }
        }
    }
}
#[test]
fn diagnoses_distinguish_missing_syntax_host_and_boundary_errors() {
    let mut e = exercise("192.168.10.77", 27);
    assert_eq!(ipv4::diagnose(&e, 0), Diagnosis::Blank);
    for s in ["hello", "256.1.2.3", "192.168.10.64/27"] {
        e.answers[0] = s.into();
        assert_eq!(ipv4::diagnose(&e, 0), Diagnosis::Malformed);
    }
    for s in ["192.168.10.77", "192.168.10.65"] {
        e.answers[0] = s.into();
        assert_eq!(ipv4::diagnose(&e, 0), Diagnosis::HostInsteadOfNetwork);
    }
    e.answers[0] = "192.168.10.32".into();
    assert_eq!(ipv4::diagnose(&e, 0), Diagnosis::WrongBoundary);
    assert!(
        !ipv4::feedback(&e, 0).1.contains("192.168.10.64"),
        "feedback must not dump expected address"
    );
    e.answers[2] = "192.168.10.64".into();
    assert_eq!(ipv4::diagnose(&e, 2), Diagnosis::ReservedHost);
    e.answers[2] = "192.168.11.1".into();
    assert_eq!(ipv4::diagnose(&e, 2), Diagnosis::OutsideSubnet);
    e.answers[2] = "192.168.10.66".into();
    assert_eq!(ipv4::diagnose(&e, 2), Diagnosis::WrongHostEnd);
}
#[test]
fn prediction_gates_are_per_result_and_explicit() {
    let mut e = exercise("192.168.10.77", 27);
    let mut l = Lesson::from_legacy(e.ip, e.prefix, 0);
    assert!(!l.reveal(Section::Network, &e));
    assert!(!l.fresh());
    e.answers[0] = "wrong but predicted".into();
    assert!(!l.can_reveal(Section::Network, &e));
    l.submit(&e);
    assert!(l.can_reveal(Section::Network, &e));
    assert!(!l.gates().1[0], "submission alone reveals nothing");
    assert!(l.reveal(Section::Network, &e));
    assert!(!l.can_reveal(Section::Broadcast, &e));
    e.answers[1] = "192.168.10.90".into();
    e.answers[2] = "192.168.10.66".into();
    l.submit(&e);
    assert!(!l.can_reveal(Section::Usable, &e));
    e.answers[3] = "192.168.10.93".into();
    l.submit(&e);
    assert!(l.reveal(Section::Broadcast, &e));
    assert!(l.reveal(Section::Usable, &e));
    assert!(l.fresh());
    assert!(!l.variant.as_ref().unwrap().solved());
    assert_eq!(l.variant_revealed, [false; 3]);
}
#[test]
fn generated_variants_are_mathematical_hosts_and_not_catalog_entries() {
    let catalog = pack::bundled().unwrap();
    assert_eq!(catalog.len(), 9);
    let mut prefixes = std::collections::BTreeSet::new();
    let mut seen = std::collections::BTreeSet::new();
    for n in 1..=500 {
        let mut e = ipv4::generated(n);
        assert!(e.prefix <= 30);
        assert_eq!(e.ip.octets()[0], 10);
        let want = e.expected().unwrap();
        assert!(e.ip > want[0] && e.ip < want[1]);
        assert!(seen.insert((e.ip, e.prefix)));
        prefixes.insert(e.prefix);
        e.answers = want.map(|a| a.to_string());
        if e.second.is_some() {
            let mut free = u32::from(want[2]);
            if std::net::Ipv4Addr::from(free) == e.ip {
                free += 1;
            }
            e.second = Some(std::net::Ipv4Addr::from(free).to_string());
        }
        assert!(e.solved());
        let st = State::Subnet {
            ip: e.ip,
            prefix: e.prefix,
            answers: e.answers.clone(),
            second: e.second.clone(),
        };
        assert!(engine::solved(&st, &engine::Goal::Subnet));
        e.answers[1] = want[0].to_string();
        assert!(!e.solved());
    }
    assert!(prefixes.contains(&23) && prefixes.contains(&20) && prefixes.contains(&30));
}
#[test]
fn optional_metadata_and_variant_resume_preserve_the_original_and_other_labs() {
    let labs = pack::bundled().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = Attempt::new(&labs[0], 1);
    let mut e = Exercise::from_state(&a.state).unwrap();
    e.answers = e.expected().unwrap().map(|a| a.to_string());
    e.second = Some("192.168.10.80".into());
    if let State::Subnet { answers, .. } = &mut a.state {
        *answers = e.answers.clone();
    }
    ipv4::write_exercise(&mut a.state, &e);
    let mut l = Lesson::from_legacy(e.ip, e.prefix, 2);
    for section in [Section::Network, Section::Broadcast, Section::Usable] {
        assert!(l.reveal(section, &e));
    }
    assert!(l.fresh());
    l.variant.as_mut().unwrap().answers[0] = "10.12.".into();
    let v = l.variant.as_ref().unwrap();
    l.sandbox_host = u32::from(v.ip) & !ipv4::mask(v.prefix).unwrap();
    a.ipv4 = Some(l);
    p.attempts.insert(labs[0].id.clone(), a);
    p.attempts
        .insert(labs[3].id.clone(), Attempt::new(&labs[3], 2));
    progress::save(&path, &p).unwrap();
    let q = progress::load(&path, &labs).unwrap();
    assert!(engine::solved(
        &q.attempts[&labs[0].id].state,
        &labs[0].goal
    ));
    assert!(q.attempts[&labs[0].id].ipv4.as_ref().unwrap().use_variant);
    assert_eq!(
        q.attempts[&labs[0].id]
            .ipv4
            .as_ref()
            .unwrap()
            .variant
            .as_ref()
            .unwrap()
            .answers[0],
        "10.12."
    );
    assert_eq!(q.attempts[&labs[3].id].state, labs[3].initial);
}

#[test]
fn revealed_result_history_survives_cleared_answers_without_becoming_an_invalid_save() {
    let mut e = exercise("192.168.10.77", 27);
    e.answers[0] = "192.168.10.77".into();
    let mut l = Lesson::from_legacy(e.ip, e.prefix, 0);
    l.submit(&e);
    assert!(l.reveal(Section::Network, &e));
    e.answers[0].clear();
    l.submit(&e);
    assert!(ipv4::validate_lesson(&l, &e));
    let mut corrupt = l.clone();
    corrupt.submitted[0] = false;
    assert!(!ipv4::validate_lesson(&corrupt, &e));
}
#[test]
fn main_and_practice_walkthrough_positions_are_independent() {
    let mut e = exercise("192.168.10.77", 27);
    e.answers = e.expected().unwrap().map(|a| a.to_string());
    let mut l = Lesson::from_legacy(e.ip, e.prefix, 2);
    for section in [Section::Network, Section::Broadcast, Section::Usable] {
        l.reveal(section, &e);
    }
    assert!(l.fresh());
    assert_eq!(l.original_section, Section::Usable);
    assert_eq!(l.section, Section::Original);
    assert_eq!(l.variant_section, Section::Original);
    assert!(ipv4::validate_lesson(&l, &e));
}
