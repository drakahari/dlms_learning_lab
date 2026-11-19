use dlms_learning_lab::network_architecture::*;
#[path = "support/network_architecture_fixture.rs"]
mod fixture;
use fixture::*;
#[test]
fn all_sixteen_uplink_combinations_match_an_independent_two_arm_oracle() {
    let (f, full) = two();
    f.validate().unwrap();
    for mask in 0..16 {
        let mut c = full.clone();
        for (i, link) in c.connections.iter_mut().enumerate() {
            if mask & (1 << i) == 0 {
                link.a = None;
                link.b = None;
            }
        }
        // Independently hand-derived: north-west AND south-west, OR the east pair.
        let west = mask & 1 != 0 && mask & 4 != 0;
        let east = mask & 2 != 0 && mask & 8 != 0;
        for (e, expected) in [
            (Experiment::default(), west || east),
            (failure("west"), east),
            (failure("east"), west),
        ] {
            let actual = f.inspect(&c, &e, "north", "south").unwrap();
            assert_eq!(actual.physical.is_some(), expected, "mask{mask}");
            assert_eq!(actual.hierarchy.is_some(), expected, "mask{mask}");
            if let Some(path) = actual.hierarchy {
                assert_eq!(path.nodes.len(), 3);
                assert_eq!(path.wires.len(), 2);
            }
        }
    }
}
#[test]
fn three_tier_paths_cross_distribution_and_core_and_survive_each_stated_device_loss() {
    let (f, c) = three();
    for lost in [
        None,
        Some("east"),
        Some("west"),
        Some("dn1"),
        Some("dn2"),
        Some("ds1"),
        Some("ds2"),
    ] {
        let e = lost.map(failure).unwrap_or_default();
        let o = f.inspect(&c, &e, "north", "south").unwrap();
        assert!(o.layout_issues.is_empty());
        let p = o.hierarchy.unwrap();
        assert_eq!(p.nodes.len(), 5);
        assert_eq!(p.wires.len(), 4);
        assert_eq!(f.node(&p.nodes[1]).unwrap().role, Role::Distribution);
        assert_eq!(f.node(&p.nodes[2]).unwrap().role, Role::Core);
        assert_eq!(f.node(&p.nodes[3]).unwrap().role, Role::Distribution);
        if let Some(lost) = lost {
            assert!(!p.nodes.iter().any(|n| n == lost));
        }
    }
    let both = Experiment {
        unavailable: vec![
            Failure::Node { id: "east".into() },
            Failure::Node { id: "west".into() },
        ],
    };
    assert!(
        f.inspect(&c, &both, "north", "south")
            .unwrap()
            .hierarchy
            .is_none()
    );
}
#[test]
fn physical_connectivity_does_not_hide_wrong_tiers_or_an_access_transit_shortcut() {
    let (f, mut c) = two();
    c.connections[0] = link("nw", "north", 1, "south", 3);
    c.connections[1].a = None;
    c.connections[1].b = None;
    let o = f
        .inspect(&c, &Experiment::default(), "north", "south")
        .unwrap();
    assert_eq!(o.physical.unwrap().nodes, vec!["north", "south"]);
    assert!(o.hierarchy.is_none());
    assert!(
        o.layout_issues
            .iter()
            .any(|s| s.contains("outside the supplied tier"))
    );
    let (mut f, _) = two();
    f.nodes
        .push(node("middle", Role::Access, Some("middle"), 2));
    let c = Configuration {
        connections: vec![
            link("nw", "north", 1, "west", 1),
            link("ne", "west", 2, "middle", 1),
            link("sw", "middle", 2, "east", 1),
            link("se", "east", 2, "south", 1),
        ],
    };
    let o = f
        .inspect(&c, &Experiment::default(), "north", "south")
        .unwrap();
    assert!(o.layout_issues.is_empty());
    assert_eq!(o.physical.unwrap().wires.len(), 4);
    assert!(o.hierarchy.is_none());
}
#[test]
fn a_wrong_building_can_form_a_physical_path_but_cannot_become_a_valid_hierarchy() {
    let (f, mut c) = three();
    c.connections[0] = link("a-dn1", "north", 1, "ds1", 2);
    // Free the destination jack while retaining a complete, distinct cable inventory.
    let other = c
        .connections
        .iter_mut()
        .find(|c| c.wire == "w-ds1")
        .unwrap();
    other.a = None;
    other.b = None;
    let o = f
        .inspect(&c, &Experiment::default(), "north", "south")
        .unwrap();
    assert!(o.physical.is_some());
    assert!(o.layout_issues.iter().any(|s| s.contains("building roles")));
    assert!(!o.hierarchy.unwrap().wires.iter().any(|w| w == "a-dn1"));
}
#[test]
fn equivalent_free_ports_and_wire_order_preserve_derived_paths() {
    let (f, c) = two();
    let baseline = f
        .inspect(&c, &Experiment::default(), "north", "south")
        .unwrap();
    let mut alternate = c.clone();
    alternate.connections[0].b.as_mut().unwrap().port = "p3".into();
    alternate.connections.reverse();
    assert_eq!(
        f.inspect(&alternate, &Experiment::default(), "north", "south")
            .unwrap(),
        baseline
    );
}
#[test]
fn explicit_wire_and_endpoint_outages_break_and_restore_actual_paths() {
    let (f, c) = two();
    let e = Experiment {
        unavailable: vec![
            Failure::Wire { id: "nw".into() },
            Failure::Wire { id: "ne".into() },
        ],
    };
    assert!(
        f.inspect(&c, &e, "north", "south")
            .unwrap()
            .physical
            .is_none()
    );
    assert!(
        f.inspect(&c, &failure("north"), "north", "north")
            .unwrap()
            .physical
            .is_none()
    );
    assert!(
        f.inspect(&c, &Experiment::default(), "north", "south")
            .unwrap()
            .hierarchy
            .is_some()
    );
    assert!(
        f.inspect(&c, &Experiment::default(), "unknown", "south")
            .is_err()
    );
}
#[test]
fn conflicting_and_unplugged_drafts_resume_without_inventing_a_physical_connection() {
    let (f, mut c) = two();
    c.connections[1].a = c.connections[0].a.clone();
    f.validate_draft(&c).unwrap();
    let loaded: Configuration = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    assert_eq!(loaded, c);
    assert!(
        f.inspect(&loaded, &Experiment::default(), "north", "south")
            .unwrap_err()
            .contains("more than one cable")
    );
    c.connections[1].a = None;
    c.connections[1].b = None;
    let o = f
        .inspect(&c, &Experiment::default(), "north", "south")
        .unwrap();
    assert!(o.hierarchy.is_some());
    assert!(o.layout_issues.iter().any(|s| s.contains("unplugged end")));
    c.connections[1].a = Some(Jack {
        node: "north".into(),
        port: "p2".into(),
    });
    c.connections[1].b = Some(Jack {
        node: "north".into(),
        port: "p3".into(),
    });
    assert!(
        f.inspect(&c, &Experiment::default(), "north", "south")
            .unwrap_err()
            .contains("itself")
    );
}
#[test]
fn source_ids_case_references_roles_and_fixed_cable_inventory_are_validated() {
    let (f, c) = two();
    let mut bad = f.clone();
    bad.schema_version = 2;
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    let duplicate = bad.nodes[0].ports[0].clone();
    bad.nodes[0].ports.push(duplicate);
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.nodes[0].block = None;
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.nodes[2].role = Role::Core;
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.nodes[0].id = "../../host".into();
    assert!(bad.validate().is_err());
    let mut bad = c.clone();
    bad.connections[0].wire = "foreign".into();
    assert!(f.validate_draft(&bad).is_err());
    let mut bad = c.clone();
    bad.connections.pop();
    assert!(f.validate_draft(&bad).is_err());
    let mut bad = c.clone();
    bad.connections[0].a.as_mut().unwrap().port = "missing".into();
    assert!(f.validate_draft(&bad).is_err());
    assert!(
        f.validate_experiment(&Experiment {
            unavailable: vec![Failure::Node {
                id: "foreign".into()
            }]
        })
        .is_err()
    );
    assert!(
        f.validate_experiment(&Experiment {
            unavailable: vec![Failure::Node { id: "east".into() }; 2]
        })
        .is_err()
    );
}
#[test]
fn resource_bounds_and_unknown_json_are_rejected_before_unbounded_model_allocation() {
    let (f, c) = two();
    let mut value = serde_json::to_value(&f).unwrap();
    value["nodes"][0]["label"] = serde_json::json!("x".repeat(121));
    assert!(serde_json::from_value::<Facts>(value).is_err());
    let mut value = serde_json::to_value(&f).unwrap();
    value["nodes"][0]["block"] = serde_json::json!("x".repeat(49));
    assert!(serde_json::from_value::<Facts>(value).is_err());
    let mut value = serde_json::to_value(&f).unwrap();
    value["nodes"] = serde_json::json!(vec![&f.nodes[0]; 17]);
    assert!(serde_json::from_value::<Facts>(value).is_err());
    let mut value = serde_json::to_value(&c).unwrap();
    value["connections"] = serde_json::json!(vec![&c.connections[0]; 33]);
    assert!(serde_json::from_value::<Configuration>(value).is_err());
    let mut value = serde_json::to_value(&f).unwrap();
    value["execute"] = serde_json::json!("host command");
    assert!(serde_json::from_value::<Facts>(value).is_err());
    let mut value = serde_json::to_value(&f).unwrap();
    value["nodes"][0]["role"] = serde_json::json!("UnimplementedRouter");
    assert!(serde_json::from_value::<Facts>(value).is_err());
    let value = serde_json::json!({"unavailable":vec![Failure::Node{id:"east".into()};5]});
    assert!(serde_json::from_value::<Experiment>(value).is_err());
}

#[test]
fn upstream_counts_distinct_devices_not_parallel_cables_and_rejects_conflicts() {
    let (f, mut c) = two();
    assert_eq!(
        f.upstream(&c, "north").unwrap(),
        ["east".into(), "west".into()].into()
    );
    c.connections[1] = link("ne", "north", 2, "west", 3);
    assert_eq!(f.upstream(&c, "north").unwrap(), ["west".into()].into());
    c.connections[1].b.as_mut().unwrap().port = "p1".into();
    assert!(
        f.upstream(&c, "north")
            .unwrap_err()
            .contains("more than one cable")
    );
    assert!(f.upstream(&c, "missing").is_err());
    let (f, c) = three();
    assert_eq!(
        f.upstream(&c, "north").unwrap(),
        ["dn1".into(), "dn2".into()].into()
    );
    assert_eq!(
        f.upstream(&c, "dn1").unwrap(),
        ["east".into(), "west".into()].into()
    );
    assert!(f.upstream(&c, "east").is_err());
}

#[test]
fn a_distribution_block_is_not_a_transit_backbone_between_disconnected_cores() {
    let (f, c) = facts(
        Design::ThreeTier,
        vec![
            node("north", Role::Access, Some("north"), 1),
            node("south", Role::Access, Some("south"), 1),
            node("dn", Role::Distribution, Some("north"), 2),
            node("ds", Role::Distribution, Some("south"), 2),
            node("middle", Role::Distribution, Some("middle"), 2),
            node("west", Role::Core, None, 2),
            node("east", Role::Core, None, 2),
        ],
        vec![
            link("n", "north", 1, "dn", 1),
            link("nw", "dn", 2, "west", 1),
            link("mw", "middle", 1, "west", 2),
            link("me", "middle", 2, "east", 2),
            link("se", "ds", 2, "east", 1),
            link("s", "south", 1, "ds", 1),
        ],
    );
    let o = f
        .inspect(&c, &Experiment::default(), "north", "south")
        .unwrap();
    assert!(o.layout_issues.is_empty());
    assert!(o.physical.is_some());
    assert!(o.hierarchy.is_none());
}

#[test]
fn leaf_spine_alternatives_match_an_independent_six_link_boolean_oracle() {
    let (f, reference) = leaf_spine();
    for mask in 0..64 {
        let mut c = reference.clone();
        for (bit, index) in [0, 1, 2, 9, 10, 11].into_iter().enumerate() {
            if mask & (1 << bit) == 0 {
                c.connections[index].a = None;
                c.connections[index].b = None;
            }
        }
        for lost in [None, Some(0), Some(1), Some(2)] {
            let e = lost
                .map(|n| failure(&format!("spine-{n}")))
                .unwrap_or_default();
            let expected = (0..3)
                .filter(|n| Some(*n) != lost && mask & (1 << n) != 0 && mask & (1 << (n + 3)) != 0)
                .count();
            let o = f.inspect(&c, &e, "leaf-0", "leaf-3").unwrap();
            let paths = o.spine_alternatives.unwrap();
            assert_eq!(paths.len(), expected, "mask{mask}/lost{lost:?}");
            assert_eq!(o.hierarchy.is_some(), expected > 0);
            for p in paths {
                assert_eq!(p.nodes.len(), 3);
                assert_eq!(p.wires.len(), 2);
                assert_eq!(f.node(&p.nodes[1]).unwrap().role, Role::Spine);
            }
        }
    }
}
#[test]
fn every_leaf_pair_uses_two_links_and_each_single_spine_loss_removes_one_alternative() {
    let (f, c) = leaf_spine();
    for a in 0..4 {
        for b in 0..4 {
            if a == b {
                continue;
            }
            for lost in [None, Some(0), Some(1), Some(2)] {
                let e = lost
                    .map(|n| failure(&format!("spine-{n}")))
                    .unwrap_or_default();
                let o = f
                    .inspect(&c, &e, &format!("leaf-{a}"), &format!("leaf-{b}"))
                    .unwrap();
                assert_eq!(o.hierarchy.unwrap().wires.len(), 2);
                assert_eq!(
                    o.spine_alternatives.unwrap().len(),
                    if lost.is_some() { 2 } else { 3 }
                );
            }
        }
    }
    let o = f
        .inspect(&c, &failure("leaf-0"), "leaf-0", "leaf-1")
        .unwrap();
    assert!(o.hierarchy.is_none());
    assert!(o.spine_alternatives.unwrap().is_empty());
    let mut reordered = c.clone();
    reordered.connections.reverse();
    for w in &mut reordered.connections {
        std::mem::swap(&mut w.a, &mut w.b);
    }
    assert_eq!(
        f.inspect(&c, &Experiment::default(), "leaf-0", "leaf-3")
            .unwrap(),
        f.inspect(&reordered, &Experiment::default(), "leaf-0", "leaf-3")
            .unwrap()
    );
}
#[test]
fn parallel_cables_do_not_multiply_independent_spines_and_peer_links_do_not_qualify() {
    let (mut f, mut c) = leaf_spine();
    f.nodes
        .iter_mut()
        .find(|n| n.id == "leaf-0")
        .unwrap()
        .ports
        .push(Port { id: "p4".into() });
    f.nodes
        .iter_mut()
        .find(|n| n.id == "spine-0")
        .unwrap()
        .ports
        .push(Port { id: "p5".into() });
    f.wires.push(Wire {
        id: "parallel".into(),
        label: "Parallel cable".into(),
    });
    c.connections
        .push(link("parallel", "leaf-0", 4, "spine-0", 5));
    assert_eq!(
        f.inspect(&c, &Experiment::default(), "leaf-0", "leaf-3")
            .unwrap()
            .spine_alternatives
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        f.inspect(&c, &failure("spine-0"), "leaf-0", "leaf-3")
            .unwrap()
            .spine_alternatives
            .unwrap()
            .len(),
        2
    );
    let (mut f, mut c) = leaf_spine();
    f.nodes
        .iter_mut()
        .find(|n| n.id == "leaf-3")
        .unwrap()
        .ports
        .push(Port { id: "p4".into() });
    c.connections[0] = link("l0-s0", "leaf-0", 1, "leaf-3", 4);
    let o = f
        .inspect(&c, &Experiment::default(), "leaf-0", "leaf-3")
        .unwrap();
    assert_eq!(o.physical.unwrap().wires.len(), 1);
    assert!(!o.layout_issues.is_empty());
    assert_eq!(o.spine_alternatives.unwrap().len(), 2);
}
#[test]
fn topology_roles_cannot_be_mixed_or_relabel_old_campuses_as_a_clos() {
    let (f, _) = leaf_spine();
    let mut wrong = f.clone();
    wrong.design = Design::TwoTier;
    assert!(wrong.validate().is_err());
    let mut wrong = f.clone();
    wrong.nodes[0].role = Role::Access;
    assert!(wrong.validate().is_err());
    let mut wrong = f.clone();
    wrong.nodes[0].block = None;
    assert!(wrong.validate().is_err());
    let (mut old, c) = two();
    old.design = Design::SpineLeaf;
    assert!(old.validate_draft(&c).is_err());
    let (f, c) = two();
    let o = f
        .inspect(&c, &Experiment::default(), "north", "south")
        .unwrap();
    assert!(o.spine_alternatives.is_none());
    assert!(
        serde_json::to_value(o)
            .unwrap()
            .get("spine_alternatives")
            .is_none()
    );
}
