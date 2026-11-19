use dlms_learning_lab::dns_access::acl::{Acl, Network};
use std::net::Ipv4Addr;
fn list(texts: &[&str]) -> Acl {
    Acl::Networks {
        networks: texts.iter().map(|s| Network::parse(s).unwrap()).collect(),
    }
}
#[test]
fn prefix_zero_host_thirty_two_and_octet_boundaries_are_exact() {
    for prefix in 0..=32 {
        let mask = u32::MAX.checked_shl(32 - prefix).unwrap_or(0);
        let base = u32::from(Ipv4Addr::new(203, 0, 113, 77)) & mask;
        let n = Network {
            address: Ipv4Addr::from(base),
            prefix: prefix as u8,
        };
        let end = base | !mask;
        assert_eq!(n.interval().unwrap(), (base, end));
        assert!(n.contains(Ipv4Addr::from(base)));
        assert!(n.contains(Ipv4Addr::from(end)));
        if base > 0 {
            assert!(!n.contains(Ipv4Addr::from(base - 1)));
        }
        if end < u32::MAX {
            assert!(!n.contains(Ipv4Addr::from(end + 1)));
        }
    }
    assert_eq!(Network::parse("192.0.2.9").unwrap().prefix, 32);
    assert!(Network::parse("192.0.2.9/24").is_err());
    for bad in [
        "10/8",
        "192.0.2.0/33",
        "192.0.2.0/-1",
        "192.0.2.0/001",
        "trusted",
        "::1",
        "192.0.2.0/24/2",
    ] {
        assert!(Network::parse(bad).is_err(), "{bad}");
    }
}
#[test]
fn equivalent_split_ranges_merge_without_overflow_or_sampled_holes() {
    assert!(
        list(&["192.0.2.0/25", "192.0.2.128/25"])
            .equivalent(&list(&["192.0.2.0/24"]))
            .unwrap()
    );
    assert!(
        list(&["0.0.0.0/1", "128.0.0.0/1"])
            .equivalent(&Acl::Any)
            .unwrap()
    );
    assert!(list(&[]).equivalent(&Acl::None).unwrap());
    assert!(
        list(&["255.255.255.255/32", "0.0.0.0/0"])
            .equivalent(&Acl::Any)
            .unwrap()
    );
    assert!(
        !list(&["192.0.2.0/25"])
            .equivalent(&list(&["192.0.2.0/24"]))
            .unwrap()
    );
    assert!(
        !list(&["192.0.2.1", "192.0.2.254"])
            .equivalent(&list(&["192.0.2.0/24"]))
            .unwrap()
    );
    assert!(
        !list(&["192.0.2.0/24"])
            .permits("192.0.3.0".parse().unwrap())
            .unwrap()
    );
}
#[test]
fn malformed_typed_values_and_oversized_lists_refuse_instead_of_broadening_access() {
    let bad = Network {
        address: Ipv4Addr::new(1, 2, 3, 4),
        prefix: 255,
    };
    assert!(bad.validate().is_err());
    assert!(!bad.contains(Ipv4Addr::new(1, 2, 3, 4)));
    let acl = Acl::Networks {
        networks: vec![Network::parse("192.0.2.1").unwrap(); 33],
    };
    assert!(acl.intervals().is_err());
    assert!(serde_json::from_str::<Acl>(r#"{"kind":"any","networks":[]}"#).is_err());
    for json in [
        r#"{"kind":"none","unknown":1}"#,
        r#"{"kind":"any","networks":null}"#,
        r#"{"kind":"any","kind":"none"}"#,
        r#"{"kind":"networks","networks":[{"address":"192.0.2.1","prefix":33}]}"#,
    ] {
        assert!(serde_json::from_str::<Acl>(json).is_err(), "{json}");
    }
    for acl in [Acl::Any, Acl::None, list(&["192.0.2.0/24"])] {
        assert_eq!(
            serde_json::from_str::<Acl>(&serde_json::to_string(&acl).unwrap()).unwrap(),
            acl
        );
    }
}

#[test]
fn intersections_cover_all_values_and_accept_equivalent_ways_to_limit_service() {
    let staff = list(&["192.0.2.0/24"]);
    let split = list(&["192.0.2.128/25", "192.0.2.0/25"]);
    assert_eq!(
        Acl::intersection(&[&Acl::Any, &staff, &split]).unwrap(),
        staff.intervals().unwrap()
    );
    assert_eq!(Acl::intersection(&[&staff, &Acl::None]).unwrap(), vec![]);
    assert_eq!(Acl::intersection(&[]).unwrap(), vec![(0, u32::MAX)]);
    let edge = list(&["255.255.255.255"]);
    assert_eq!(
        Acl::intersection(&[&Acl::Any, &edge]).unwrap(),
        vec![(u32::MAX, u32::MAX)]
    );
    assert!(Acl::intersection(&[&Acl::Any; 9]).is_err());
    let disjoint = list(&["198.51.100.0/24"]);
    assert!(Acl::intersection(&[&staff, &disjoint]).unwrap().is_empty());
}
