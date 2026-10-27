//! Manually derived eligibility vectors and bounded pure-state invariants. No external LACP.
use dlms_learning_lab::etherchannel::*;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
fn profile() -> Profile {
    Profile::Trunk {
        native: 1,
        allowed: BTreeSet::from([1, 10, 20]),
    }
}
fn gr(d: &str) -> GroupRef {
    GroupRef {
        device: d.into(),
        group: format!("g{d}"),
    }
}
fn endpoint(d: &str, p: usize) -> Endpoint {
    Endpoint {
        device: d.into(),
        port: format!("p{p}"),
    }
}
fn base(members: usize) -> State {
    let mut t = Topology {
        fixture: "bounded-lacp".into(),
        devices: BTreeMap::new(),
        cables: BTreeMap::new(),
    };
    let mut c = Configuration {
        devices: BTreeMap::new(),
    };
    let mut env = Environment {
        cables: BTreeMap::new(),
    };
    for (n, d) in ["a", "b", "c", "d"].iter().enumerate() {
        let mut ports = BTreeMap::new();
        let mut cp = BTreeMap::new();
        let group = format!("g{d}");
        for p in 0..8 {
            let name = format!("p{p}");
            ports.insert(
                name.clone(),
                PortSpec {
                    rates: BTreeSet::from([Rate::R100, Rate::R1000, Rate::R10000]),
                },
            );
            cp.insert(
                name,
                PortConfig {
                    up: true,
                    group: Some(group.clone()),
                    mode: if *d == "a" || *d == "c" {
                        LacpMode::Active
                    } else {
                        LacpMode::Passive
                    },
                    rate: Rate::R1000,
                    duplex: Duplex::Full,
                    profile: profile(),
                },
            );
        }
        t.devices.insert(
            (*d).into(),
            Device {
                mac: Mac([2, 0, 0, 0, 0, n as u8 + 1]),
                ports,
                groups: BTreeMap::from([(group.clone(), GroupSlot {})]),
            },
        );
        c.devices.insert(
            (*d).into(),
            DeviceConfig {
                vlans: BTreeSet::from([1, 10, 20]),
                groups: BTreeMap::from([(
                    group,
                    GroupConfig {
                        up: true,
                        template: Template {
                            rate: Rate::R1000,
                            profile: profile(),
                        },
                    },
                )]),
                ports: cp,
            },
        );
    }
    for i in 0..members {
        let n = format!("l{i}");
        t.cables.insert(
            n.clone(),
            Cable {
                a: endpoint("a", i),
                b: endpoint("b", i),
            },
        );
        env.cables.insert(n, true);
    }
    t.cables.insert(
        "preserved".into(),
        Cable {
            a: endpoint("c", 0),
            b: endpoint("d", 0),
        },
    );
    env.cables.insert("preserved".into(), true);
    State::new(t, c, env).unwrap()
}
fn change(s: &mut State, f: impl FnOnce(&mut Configuration)) {
    let mut c = s.configuration().clone();
    f(&mut c);
    s.reconfigure(c).unwrap();
}
fn query(vlan: u16, minimum_members: u8) -> ServiceQuery {
    ServiceQuery {
        a: gr("a"),
        b: gr("b"),
        vlan,
        minimum_members,
    }
}
fn plan(q: Vec<ServiceQuery>) -> Evaluation {
    Evaluation {
        queries: q,
        faults: vec![],
    }
}
fn service(s: &State, v: u16, n: u8) -> Admission {
    s.evaluate(&plan(vec![query(v, n)])).unwrap().services[0][0]
}
fn members(s: &State) -> usize {
    let q = s.snapshot().unwrap();
    let g = q.group(&gr("a")).unwrap();
    if matches!(g.status, GroupStatus::Ready { .. }) {
        g.eligible_members.len()
    } else {
        0
    }
}
fn unchanged(s: &State, before: &Value) {
    assert!(
        serde_json::to_value(s).unwrap() == *before,
        "Complete immutable caller changed"
    );
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vectors {
    schema_version: u32,
    origin: String,
    cases: Vec<Vector>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    name: String,
    members: usize,
    mutation: String,
    ready_members: usize,
    vlan10: AdmissionExpected,
    vlan20: AdmissionExpected,
}
#[derive(Clone, Copy, Deserialize)]
enum AdmissionExpected {
    Ready,
    NoReadyPair,
    TooFewMembers,
    VlanNotAllowed,
    InactiveVlan,
    NativeMismatch,
    AccessTrunkMismatch,
}
impl AdmissionExpected {
    fn check(self, a: Admission, n: usize) {
        match self {
            Self::Ready => assert_eq!(a, Admission::Ready { members: n }),
            Self::NoReadyPair => assert_eq!(
                a,
                Admission::Unavailable {
                    reason: ServiceFailure::NoReadyPair
                }
            ),
            Self::TooFewMembers => assert_eq!(
                a,
                Admission::Unavailable {
                    reason: ServiceFailure::TooFewMembers
                }
            ),
            Self::VlanNotAllowed => assert_eq!(
                a,
                Admission::Unavailable {
                    reason: ServiceFailure::VlanNotAllowed
                }
            ),
            Self::InactiveVlan => assert_eq!(
                a,
                Admission::Unavailable {
                    reason: ServiceFailure::InactiveVlan
                }
            ),
            Self::NativeMismatch => assert_eq!(
                a,
                Admission::Unsupported {
                    reason: UnsupportedService::NativeMismatch
                }
            ),
            Self::AccessTrunkMismatch => assert_eq!(
                a,
                Admission::Unsupported {
                    reason: UnsupportedService::AccessTrunkMismatch
                }
            ),
        }
    }
}
fn mutation(s: &mut State, n: &str) {
    if n == "cable-down" {
        let mut e = s.environment().clone();
        e.cables.insert("l1".into(), false);
        *s = State::new(s.topology().clone(), s.configuration().clone(), e).unwrap();
        return;
    }
    change(s, |c| match n {
        "none" => {}
        "active-active" => {
            for p in c.devices.get_mut("b").unwrap().ports.values_mut() {
                p.mode = LacpMode::Active;
            }
        }
        "reverse-initiator" => {
            for p in c.devices.get_mut("a").unwrap().ports.values_mut() {
                p.mode = LacpMode::Passive;
            }
            for p in c.devices.get_mut("b").unwrap().ports.values_mut() {
                p.mode = LacpMode::Active;
            }
        }
        "passive-passive" => {
            for p in c.devices.get_mut("a").unwrap().ports.values_mut() {
                p.mode = LacpMode::Passive;
            }
        }
        "port-down" => {
            c.devices
                .get_mut("a")
                .unwrap()
                .ports
                .get_mut("p1")
                .unwrap()
                .up = false
        }
        "group-down" => {
            c.devices
                .get_mut("b")
                .unwrap()
                .groups
                .get_mut("gb")
                .unwrap()
                .up = false
        }
        "missing-member" => {
            c.devices
                .get_mut("b")
                .unwrap()
                .ports
                .get_mut("p1")
                .unwrap()
                .group = None
        }
        "half-duplex" => {
            c.devices
                .get_mut("a")
                .unwrap()
                .ports
                .get_mut("p1")
                .unwrap()
                .duplex = Duplex::Half
        }
        "rate-mismatch" => {
            c.devices
                .get_mut("a")
                .unwrap()
                .ports
                .get_mut("p1")
                .unwrap()
                .rate = Rate::R100
        }
        "member-profile" => {
            c.devices
                .get_mut("a")
                .unwrap()
                .ports
                .get_mut("p1")
                .unwrap()
                .profile = Profile::Trunk {
                native: 1,
                allowed: BTreeSet::from([1, 10]),
            }
        }
        "remote-allowed" => {
            let d = c.devices.get_mut("b").unwrap();
            let v = Profile::Trunk {
                native: 1,
                allowed: BTreeSet::from([1, 10]),
            };
            d.groups.get_mut("gb").unwrap().template.profile = v.clone();
            for p in d.ports.values_mut() {
                p.profile = v.clone();
            }
        }
        "remote-native" => {
            let d = c.devices.get_mut("b").unwrap();
            let v = Profile::Trunk {
                native: 20,
                allowed: BTreeSet::from([1, 10, 20]),
            };
            d.groups.get_mut("gb").unwrap().template.profile = v.clone();
            for p in d.ports.values_mut() {
                p.profile = v.clone();
            }
        }
        "inactive-vlan" => {
            c.devices.get_mut("b").unwrap().vlans.remove(&20);
        }
        "access10" => {
            for d in c.devices.values_mut() {
                d.groups
                    .values_mut()
                    .for_each(|g| g.template.profile = Profile::Access { vlan: 10 });
                d.ports
                    .values_mut()
                    .for_each(|p| p.profile = Profile::Access { vlan: 10 });
            }
        }
        "access-trunk" => {
            let d = c.devices.get_mut("b").unwrap();
            d.groups.get_mut("gb").unwrap().template.profile = Profile::Access { vlan: 10 };
            d.ports
                .values_mut()
                .for_each(|p| p.profile = Profile::Access { vlan: 10 });
        }
        "all100" | "all10000" => {
            let rate = if n == "all100" {
                Rate::R100
            } else {
                Rate::R10000
            };
            for d in c.devices.values_mut() {
                d.groups.values_mut().for_each(|g| g.template.rate = rate);
                d.ports.values_mut().for_each(|p| p.rate = rate);
            }
        }
        _ => panic!("Unrecognized manually authored mutation {n}"),
    });
}
#[test]
fn independently_authored_literal_mode_rate_duplex_membership_and_service_vectors() {
    let v: Vectors =
        serde_json::from_str(include_str!("fixtures/etherchannel-vectors-v1.json")).unwrap();
    assert_eq!(v.schema_version, 1);
    assert!(v.origin.contains("manually"));
    assert_eq!(v.cases.len(), 20);
    let mut names = BTreeSet::new();
    for q in v.cases {
        assert!(names.insert(q.name.clone()));
        let mut s = base(q.members);
        mutation(&mut s, &q.mutation);
        let before = serde_json::to_value(&s).unwrap();
        assert_eq!(members(&s), q.ready_members, "{}", q.name);
        q.vlan10.check(service(&s, 10, 2), q.ready_members);
        q.vlan20.check(service(&s, 20, 2), q.ready_members);
        unchanged(&s, &before);
    }
}
#[test]
fn one_member_is_operationally_ready_while_two_is_an_explicit_query_requirement() {
    let s = base(1);
    assert_eq!(members(&s), 1);
    assert_eq!(service(&s, 10, 1), Admission::Ready { members: 1 });
    assert_eq!(
        service(&s, 10, 2),
        Admission::Unavailable {
            reason: ServiceFailure::TooFewMembers
        }
    );
}
#[test]
fn independent_local_numbers_and_active_end_are_not_exact_answer_credentials() {
    let mut s = base(2);
    let mut t = s.topology().clone();
    t.devices.get_mut("b").unwrap().groups = BTreeMap::from([("remote19".into(), GroupSlot {})]);
    let mut c = s.configuration().clone();
    let b = c.devices.get_mut("b").unwrap();
    let old = b.groups.remove("gb").unwrap();
    b.groups.insert("remote19".into(), old);
    for p in b.ports.values_mut() {
        p.group = Some("remote19".into());
    }
    s = State::new(t, c, s.environment().clone()).unwrap();
    let mut q = query(10, 2);
    q.b.group = "remote19".into();
    assert_eq!(
        s.evaluate(&plan(vec![q])).unwrap().services[0][0],
        Admission::Ready { members: 2 }
    );
    mutation(&mut s, "reverse-initiator");
    assert_eq!(members(&s), 2);
}
fn mixed() -> State {
    let s = base(2);
    let mut t = s.topology().clone();
    t.cables.insert(
        "mixed".into(),
        Cable {
            a: endpoint("a", 2),
            b: endpoint("c", 1),
        },
    );
    let mut e = s.environment().clone();
    e.cables.insert("mixed".into(), true);
    State::new(t, s.configuration().clone(), e).unwrap()
}
#[test]
fn mixed_live_partners_and_their_reciprocal_peers_never_receive_positive_admission() {
    let s = mixed();
    let snap = s.snapshot().unwrap();
    for d in ["a", "b", "c", "d"] {
        assert_eq!(
            snap.group(&gr(d)).unwrap().status,
            GroupStatus::UnsupportedPartnerSelection {}
        );
    }
    assert!(snap.links().iter().all(|l| l.eligible
        && !l.participating
        && l.reasons.contains(&Exclusion::UnsupportedPartnerSelection)));
    assert_eq!(
        service(&s, 10, 1),
        Admission::Unsupported {
            reason: UnsupportedService::PartnerSelection
        }
    );
}
#[test]
fn disabled_down_unassigned_mismatched_or_noninitiating_noncandidates_do_not_poison_a_healthy_pair()
{
    for mutation in 0..6 {
        let mut s = mixed();
        let mut e = s.environment().clone();
        let mut c = s.configuration().clone();
        match mutation {
            0 => {
                e.cables.insert("mixed".into(), false);
            }
            1 => {
                c.devices
                    .get_mut("a")
                    .unwrap()
                    .ports
                    .get_mut("p2")
                    .unwrap()
                    .up = false
            }
            2 => {
                c.devices
                    .get_mut("a")
                    .unwrap()
                    .ports
                    .get_mut("p2")
                    .unwrap()
                    .group = None
            }
            3 => {
                c.devices
                    .get_mut("a")
                    .unwrap()
                    .ports
                    .get_mut("p2")
                    .unwrap()
                    .profile = Profile::Access { vlan: 10 }
            }
            4 => {
                c.devices
                    .get_mut("a")
                    .unwrap()
                    .ports
                    .get_mut("p2")
                    .unwrap()
                    .mode = LacpMode::Passive;
                c.devices
                    .get_mut("c")
                    .unwrap()
                    .ports
                    .get_mut("p1")
                    .unwrap()
                    .mode = LacpMode::Passive;
            }
            5 => {
                c.devices
                    .get_mut("a")
                    .unwrap()
                    .ports
                    .get_mut("p2")
                    .unwrap()
                    .duplex = Duplex::Half
            }
            _ => unreachable!(),
        };
        s = State::new(s.topology().clone(), c, e).unwrap();
        assert_eq!(
            service(&s, 10, 2),
            Admission::Ready { members: 2 },
            "filter{mutation}"
        );
        assert_eq!(
            s.snapshot().unwrap().group(&gr("c")).unwrap().status,
            GroupStatus::Ready {
                partner: GroupIdentity {
                    device: "d",
                    group: "gd"
                }
            }
        );
    }
}
#[test]
fn an_entirely_disabled_partner_group_is_ignored_before_ambiguity() {
    let mut s = mixed();
    change(&mut s, |c| {
        c.devices
            .get_mut("c")
            .unwrap()
            .groups
            .get_mut("gc")
            .unwrap()
            .up = false
    });
    assert_eq!(service(&s, 10, 2), Admission::Ready { members: 2 });
    assert_eq!(
        s.snapshot().unwrap().group(&gr("c")).unwrap().status,
        GroupStatus::Unavailable {}
    );
}
#[test]
fn split_remote_groups_are_unsupported_only_when_the_extra_link_is_actually_eligible() {
    let mut s = base(2);
    let mut t = s.topology().clone();
    t.devices
        .get_mut("b")
        .unwrap()
        .groups
        .insert("other".into(), GroupSlot {});
    let mut c = s.configuration().clone();
    let d = c.devices.get_mut("b").unwrap();
    d.groups.insert("other".into(), d.groups["gb"].clone());
    d.ports.get_mut("p1").unwrap().group = Some("other".into());
    s = State::new(t, c, s.environment().clone()).unwrap();
    assert_eq!(
        service(&s, 10, 1),
        Admission::Unsupported {
            reason: UnsupportedService::PartnerSelection
        }
    );
    change(&mut s, |c| {
        c.devices
            .get_mut("b")
            .unwrap()
            .ports
            .get_mut("p1")
            .unwrap()
            .up = false
    });
    assert_eq!(service(&s, 10, 1), Admission::Ready { members: 1 });
}
#[test]
fn current_plus_four_fault_snapshots_are_computed_once_and_do_not_install_repairs() {
    let s = base(3);
    let b = serde_json::to_value(&s).unwrap();
    let p = Evaluation {
        queries: vec![query(10, 2), query(20, 2)],
        faults: vec![
            FaultCondition {
                down: BTreeSet::from(["l0".into()]),
            },
            FaultCondition {
                down: BTreeSet::from(["l1".into()]),
            },
            FaultCondition {
                down: BTreeSet::from(["l2".into()]),
            },
            FaultCondition {
                down: BTreeSet::from(["l0".into(), "l1".into()]),
            },
        ],
    };
    let r = s.evaluate(&p).unwrap();
    assert_eq!(r.snapshots.len(), 5);
    assert_eq!(r.cable_visits, 20);
    assert_eq!(r.work, 30);
    assert_eq!(r.services[0], [Admission::Ready { members: 3 }; 2]);
    for a in &r.services[1..4] {
        assert_eq!(*a, [Admission::Ready { members: 2 }; 2]);
    }
    assert!(r.services[4].iter().all(|a| *a
        == Admission::Unavailable {
            reason: ServiceFailure::TooFewMembers
        }));
    unchanged(&s, &b);
}
#[test]
fn current_mutation_breaks_again_and_unrelated_configuration_is_preserved() {
    let mut s = base(2);
    let before_c = s.configuration().devices["c"].clone();
    let before_d = s.configuration().devices["d"].clone();
    let before = s.snapshot().unwrap().group(&gr("c")).unwrap().clone();
    let before = serde_json::to_value(before).unwrap();
    assert_eq!(service(&s, 10, 2), Admission::Ready { members: 2 });
    mutation(&mut s, "port-down");
    assert_eq!(
        service(&s, 10, 2),
        Admission::Unavailable {
            reason: ServiceFailure::TooFewMembers
        }
    );
    assert_eq!(&s.configuration().devices["c"], &before_c);
    assert_eq!(&s.configuration().devices["d"], &before_d);
    assert_eq!(
        serde_json::to_value(s.snapshot().unwrap().group(&gr("c")).unwrap()).unwrap(),
        before
    );
    change(&mut s, |c| {
        c.devices
            .get_mut("a")
            .unwrap()
            .ports
            .get_mut("p1")
            .unwrap()
            .up = true
    });
    assert_eq!(service(&s, 10, 2), Admission::Ready { members: 2 });
}
#[test]
fn topology_and_environment_context_never_change_during_configuration_apply() {
    let mut s = base(2);
    let before = serde_json::to_value(&s).unwrap();
    let mut c = s.configuration().clone();
    c.devices
        .get_mut("a")
        .unwrap()
        .ports
        .get_mut("p0")
        .unwrap()
        .group = Some("foreign".into());
    assert_eq!(
        s.apply(&Action::Configure { configuration: c })
            .unwrap_err()
            .kind,
        FaultKind::Context
    );
    unchanged(&s, &before);
    let mut t = s.topology().clone();
    t.devices.get_mut("a").unwrap().mac = Mac([2, 0, 0, 0, 5, 9]);
    assert_eq!(s.validate_context(&t).unwrap_err().kind, FaultKind::Context);
    unchanged(&s, &before);
}
#[test]
fn malformed_attachments_duplicate_mac_and_missing_exact_keysets_are_rejected() {
    let s = base(2);
    for n in 0..6 {
        let mut t = s.topology().clone();
        match n {
            0 => t.cables.get_mut("l1").unwrap().a = t.cables["l0"].a.clone(),
            1 => t.cables.get_mut("l0").unwrap().b.device = "a".into(),
            2 => t.cables.get_mut("l0").unwrap().b.port = "unknown".into(),
            3 => t.devices.get_mut("b").unwrap().mac = t.devices["a"].mac,
            4 => t.devices.get_mut("a").unwrap().mac = Mac([1, 0, 0, 0, 0, 1]),
            5 => {
                t.devices.get_mut("a").unwrap().ports.clear();
            }
            _ => unreachable!(),
        };
        assert!(State::new(t, s.configuration().clone(), s.environment().clone()).is_err());
    }
    let mut c = s.configuration().clone();
    c.devices.get_mut("a").unwrap().ports.remove("p0");
    assert!(State::new(s.topology().clone(), c, s.environment().clone()).is_err());
    let mut e = s.environment().clone();
    e.cables.remove("l0");
    assert!(State::new(s.topology().clone(), s.configuration().clone(), e).is_err());
}
#[test]
fn disconnected_and_empty_groups_are_unavailable_not_unsupported_or_ready() {
    let s = base(0);
    let q = s.snapshot().unwrap();
    assert_eq!(
        q.group(&gr("a")).unwrap().status,
        GroupStatus::Unavailable {}
    );
    assert_eq!(
        service(&s, 10, 1),
        Admission::Unavailable {
            reason: ServiceFailure::NoReadyPair
        }
    );
    assert_eq!(
        q.group(&gr("c")).unwrap().status,
        GroupStatus::Ready {
            partner: GroupIdentity {
                device: "d",
                group: "gd"
            }
        }
    );
}
#[test]
fn unknown_query_and_override_cannot_be_mistaken_for_a_known_negative_result() {
    let s = base(2);
    let b = serde_json::to_value(&s).unwrap();
    let mut q = query(10, 2);
    q.a.group = "absent".into();
    assert_eq!(
        s.evaluate(&plan(vec![q])).unwrap_err().kind,
        FaultKind::Context
    );
    let mut p = plan(vec![query(10, 2)]);
    p.faults.push(FaultCondition {
        down: BTreeSet::from(["absent".into()]),
    });
    assert_eq!(s.evaluate(&p).unwrap_err().kind, FaultKind::Context);
    unchanged(&s, &b);
}
#[test]
fn vlan_domain_boundaries_and_large_direct_fields_fail_before_cloning() {
    let mut s = base(2);
    let before = serde_json::to_value(&s).unwrap();
    for bad in [0, 4095] {
        let mut c = s.configuration().clone();
        c.devices
            .get_mut("a")
            .unwrap()
            .groups
            .get_mut("ga")
            .unwrap()
            .template
            .profile = Profile::Access { vlan: bad };
        assert_eq!(
            s.apply(&Action::Configure { configuration: c })
                .unwrap_err()
                .kind,
            FaultKind::Invalid
        );
        unchanged(&s, &before);
    }
    let mut c = s.configuration().clone();
    c.devices
        .get_mut("a")
        .unwrap()
        .ports
        .get_mut("p0")
        .unwrap()
        .group = Some("a".repeat(100000));
    assert_eq!(
        s.apply(&Action::Configure { configuration: c })
            .unwrap_err()
            .kind,
        FaultKind::Invalid
    );
    unchanged(&s, &before);
    let mut c = s.configuration().clone();
    for i in 0..9 {
        c.devices.get_mut("a").unwrap().ports.insert(
            format!("extra{i}"),
            s.configuration().devices["a"].ports["p0"].clone(),
        );
    }
    assert_eq!(
        s.apply(&Action::Configure { configuration: c })
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
    unchanged(&s, &before);
}
#[test]
fn strict_wire_roundtrip_unknown_duplicate_tags_and_future_version() {
    let s = base(2);
    let bytes = serde_json::to_vec(&s).unwrap();
    assert_eq!(State::from_json(&bytes).unwrap(), s);
    let action = Action::Configure {
        configuration: s.configuration().clone(),
    };
    let bytes = serde_json::to_vec(&action).unwrap();
    assert_eq!(Action::from_json(&bytes).unwrap(), action);
    let duplicate_kind = String::from_utf8(bytes.clone()).unwrap().replacen(
        "\"kind\":\"Configure\"",
        "\"kind\":\"Configure\",\"kind\":\"Configure\"",
        1,
    );
    assert!(
        serde_json::from_str::<Action>(&duplicate_kind)
            .unwrap_err()
            .to_string()
            .contains("duplicate field")
    );
    for raw in [
        "{\"kind\":\"Configure\",\"kind\":\"Configure\",\"configuration\":{}}",
        "{\"kind\":\"Unmodelled\"}",
    ] {
        assert!(Action::from_json(raw.as_bytes()).is_err());
    }
    let mut v = serde_json::to_value(&action).unwrap();
    v["unknown"] = json!(1);
    assert!(Action::from_json(&serde_json::to_vec(&v).unwrap()).is_err());
    let mut v = serde_json::to_value(&s).unwrap();
    v["schema_version"] = json!(2);
    assert!(State::from_json(&serde_json::to_vec(&v).unwrap()).is_err());
    v = serde_json::to_value(&s).unwrap();
    v["topology"]["devices"]["a"]["groups"]["ga"]["unknown"] = json!(1);
    assert!(State::from_json(&serde_json::to_vec(&v).unwrap()).is_err());
}
#[test]
fn duplicate_maps_sets_and_valid_length_escaped_identifier_limits_are_strict() {
    let s = base(2);
    let text = serde_json::to_string(&s).unwrap();
    let duplicate = text.replacen(
        "\"fixture\":\"bounded-lacp\"",
        "\"fixture\":\"bounded-lacp\",\"fixture\":\"bounded-lacp\"",
        1,
    );
    assert!(State::from_json(duplicate.as_bytes()).is_err());
    let mut v = serde_json::to_value(&s).unwrap();
    v["configuration"]["devices"]["a"]["vlans"] = json!([1, 10, 10]);
    assert!(State::from_json(&serde_json::to_vec(&v).unwrap()).is_err());
    let escaped = text.replacen(
        "\"bounded-lacp\"",
        &format!("\"{}\"", "\\u0061".repeat(32)),
        1,
    );
    let good = State::from_json(escaped.as_bytes()).unwrap();
    assert_eq!(good.topology().fixture.len(), 32);
    let over = text.replacen(
        "\"bounded-lacp\"",
        &format!("\"{}\"", "\\u0061".repeat(33)),
        1,
    );
    assert!(State::from_json(over.as_bytes()).is_err());
}
#[test]
fn raw_and_pretyped_n_plus_one_frontiers_fail_closed() {
    assert_eq!(
        State::from_json(&vec![b' '; MAX_STATE_BYTES + 1])
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
    assert_eq!(
        Action::from_json(&vec![b' '; MAX_ACTION_BYTES + 1])
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
    assert_eq!(
        Evaluation::from_json(&vec![b' '; MAX_ACTION_BYTES + 1])
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
    let raw = format!(
        "{{\"queries\":[{},{{\"impossible\":true}}],\"faults\":[]}}",
        std::iter::repeat_n(serde_json::to_string(&query(10, 2)).unwrap(), 8)
            .collect::<Vec<_>>()
            .join(",")
    );
    let e = serde_json::from_str::<Evaluation>(&raw)
        .unwrap_err()
        .to_string();
    assert!(
        e.contains("sequence limit"),
        "N+1 should refuse before typed object fields: {e}"
    );
    let s = base(2);
    let mut p = plan(vec![query(10, 2); 9]);
    assert_eq!(s.evaluate(&p).unwrap_err().kind, FaultKind::Limit);
    p = plan(vec![]);
    p.faults = vec![
        FaultCondition {
            down: BTreeSet::new()
        };
        5
    ];
    assert_eq!(s.evaluate(&p).unwrap_err().kind, FaultKind::Limit);
}
#[test]
fn maximal_topology_batch_and_repeated_queries_remain_bounded() {
    let mut t = Topology {
        fixture: "maximum".into(),
        devices: BTreeMap::new(),
        cables: BTreeMap::new(),
    };
    let mut c = Configuration {
        devices: BTreeMap::new(),
    };
    let mut e = Environment {
        cables: BTreeMap::new(),
    };
    for i in 0..8 {
        let id = format!("device{i}xxxxxxxxxxxxxxxxxxxxxxxx");
        assert_eq!(id.len(), 31);
        let mut ports = BTreeMap::new();
        let mut cp = BTreeMap::new();
        let mut groups = BTreeMap::new();
        let mut cg = BTreeMap::new();
        for j in 0..2 {
            let g = format!("group{j}xxxxxxxxxxxxxxxxxxxxxxxxx");
            groups.insert(g.clone(), GroupSlot {});
            cg.insert(
                g,
                GroupConfig {
                    up: true,
                    template: Template {
                        rate: Rate::R10000,
                        profile: Profile::Trunk {
                            native: 1,
                            allowed: (1..=64).collect(),
                        },
                    },
                },
            );
        }
        for j in 0..8 {
            let p = format!("port{j}xxxxxxxxxxxxxxxxxxxxxxxxxx");
            ports.insert(
                p.clone(),
                PortSpec {
                    rates: BTreeSet::from([Rate::R10000]),
                },
            );
            cp.insert(
                p,
                PortConfig {
                    up: true,
                    group: Some(format!("group{}xxxxxxxxxxxxxxxxxxxxxxxxx", j / 4)),
                    mode: LacpMode::Active,
                    rate: Rate::R10000,
                    duplex: Duplex::Full,
                    profile: Profile::Trunk {
                        native: 1,
                        allowed: (1..=64).collect(),
                    },
                },
            );
        }
        t.devices.insert(
            id.clone(),
            Device {
                mac: Mac([2, 0, 0, 0, 0, i + 1]),
                ports,
                groups,
            },
        );
        c.devices.insert(
            id,
            DeviceConfig {
                vlans: (1..=64).collect(),
                groups: cg,
                ports: cp,
            },
        );
    }
    let devices = t.devices.keys().cloned().collect::<Vec<_>>();
    for i in (0..8).step_by(2) {
        for p in 0..8 {
            let n = format!("cable{i}_{p}xxxxxxxxxxxxxxxxxxxxxxx");
            let port = format!("port{p}xxxxxxxxxxxxxxxxxxxxxxxxxx");
            t.cables.insert(
                n.clone(),
                Cable {
                    a: Endpoint {
                        device: devices[i].clone(),
                        port: port.clone(),
                    },
                    b: Endpoint {
                        device: devices[i + 1].clone(),
                        port,
                    },
                },
            );
            e.cables.insert(n, true);
        }
    }
    let s = State::new(t, c, e).unwrap();
    let mut queries = Vec::new();
    for i in (0..8).step_by(2) {
        for g in 0..2 {
            queries.push(ServiceQuery {
                a: GroupRef {
                    device: devices[i].clone(),
                    group: format!("group{g}xxxxxxxxxxxxxxxxxxxxxxxxx"),
                },
                b: GroupRef {
                    device: devices[i + 1].clone(),
                    group: format!("group{g}xxxxxxxxxxxxxxxxxxxxxxxxx"),
                },
                vlan: 20,
                minimum_members: 1,
            });
        }
    }
    let p = Evaluation {
        queries,
        faults: s
            .topology()
            .cables
            .keys()
            .take(4)
            .map(|n| FaultCondition {
                down: BTreeSet::from([n.clone()]),
            })
            .collect(),
    };
    let before = serde_json::to_value(&s).unwrap();
    let a = s.evaluate(&p).unwrap();
    let b = s.evaluate(&p).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.cable_visits, 160);
    assert_eq!(a.work, 200);
    assert!(
        a.services
            .iter()
            .flatten()
            .all(|a| matches!(a, Admission::Ready { .. }))
    );
    assert!(serde_json::to_vec(&a).unwrap().len() <= MAX_REPORT_BYTES);
    unchanged(&s, &before);
}
#[test]
fn port_and_cable_order_and_reversed_attachments_do_not_choose_a_lucky_member() {
    let s = base(2);
    let mut t = s.topology().clone();
    for cable in t.cables.values_mut() {
        std::mem::swap(&mut cable.a, &mut cable.b);
    }
    let other = State::new(t, s.configuration().clone(), s.environment().clone()).unwrap();
    assert_eq!(members(&s), members(&other));
    for v in [10, 20] {
        assert_eq!(service(&s, v, 2), service(&other, v, 2));
    }
    assert_eq!(
        s.snapshot()
            .unwrap()
            .group(&gr("a"))
            .unwrap()
            .eligible_members,
        other
            .snapshot()
            .unwrap()
            .group(&gr("a"))
            .unwrap()
            .eligible_members
    );
}
#[test]
fn exact_raw_caps_and_n_plus_one_map_value_preflight() {
    let s = base(2);
    let mut raw = serde_json::to_vec(&s).unwrap();
    raw.resize(MAX_STATE_BYTES, b' ');
    assert_eq!(State::from_json(&raw).unwrap(), s);
    raw.push(b' ');
    assert_eq!(State::from_json(&raw).unwrap_err().kind, FaultKind::Limit);
    let a = Action::Configure {
        configuration: s.configuration().clone(),
    };
    let mut raw = serde_json::to_vec(&a).unwrap();
    raw.resize(MAX_ACTION_BYTES, b' ');
    assert_eq!(Action::from_json(&raw).unwrap(), a);
    let mut v = serde_json::to_value(&s).unwrap();
    v["topology"]["devices"]["a"]["ports"]["z"] = json!({"must_not_parse":true});
    let e = serde_json::from_value::<State>(v).unwrap_err().to_string();
    assert!(
        e.contains("map limit"),
        "Map N+1 must precede PortSpec value parsing: {e}"
    );
    let raw = format!(
        "{{\"kind\":\"Trunk\",\"native\":1,\"allowed\":[{},{{\"not_a_vlan\":true}}]}}",
        (1..=64)
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    assert!(
        serde_json::from_str::<Profile>(&raw)
            .unwrap_err()
            .to_string()
            .contains("sequence limit")
    );
}
#[test]
fn coherent_template_changes_are_valid_but_unsupported_supplied_rate_is_an_atomic_context_error() {
    let mut s = base(2);
    let original = serde_json::to_value(&s).unwrap();
    mutation(&mut s, "all100");
    assert_eq!(service(&s, 10, 2), Admission::Ready { members: 2 });
    assert!(serde_json::to_value(&s).unwrap() != original);
    let mut t = s.topology().clone();
    t.devices
        .get_mut("a")
        .unwrap()
        .ports
        .get_mut("p0")
        .unwrap()
        .rates = BTreeSet::from([Rate::R100]);
    let mut limited = State::new(t, s.configuration().clone(), s.environment().clone()).unwrap();
    let b = serde_json::to_value(&limited).unwrap();
    let mut c = limited.configuration().clone();
    c.devices
        .get_mut("a")
        .unwrap()
        .ports
        .get_mut("p0")
        .unwrap()
        .rate = Rate::R1000;
    assert_eq!(
        limited
            .apply(&Action::Configure { configuration: c })
            .unwrap_err()
            .kind,
        FaultKind::Context
    );
    unchanged(&limited, &b);
}
#[test]
fn equal_local_templates_do_not_hide_cross_cable_rate_mismatch() {
    let mut s = base(2);
    change(&mut s, |c| {
        let a = c.devices.get_mut("a").unwrap();
        a.groups.get_mut("ga").unwrap().template.rate = Rate::R100;
        for p in a.ports.values_mut() {
            p.rate = Rate::R100;
        }
    });
    let q = s.snapshot().unwrap();
    let first = q.links().iter().find(|l| l.cable == "l0").unwrap();
    assert!(first.reasons.contains(&Exclusion::LineRateMismatch));
    assert!(
        !first.reasons.contains(&Exclusion::TemplateMismatchA)
            && !first.reasons.contains(&Exclusion::TemplateMismatchB)
    );
    assert_eq!(
        service(&s, 10, 1),
        Admission::Unavailable {
            reason: ServiceFailure::NoReadyPair
        }
    );
}
