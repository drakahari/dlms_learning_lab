use dlms_learning_lab::{
    automation_workshop::route_reading::{
        table::Origin,
        workspace::{Draft, Task},
    },
    ipv4_routing::Prefix,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Tasks {
    example: Task,
    assignment: Task,
}

fn cases() -> Tasks {
    serde_json::from_str(include_str!("fixtures/route-reading.json")).unwrap()
}

#[test]
fn distance_and_cost_make_distinct_decisions_and_restore_fallback() {
    for (task, accepted_cost) in [(cases().example, "3"), (cases().assignment, "5")] {
        task.validate().unwrap();
        let prefix = task.static_backup.destination;
        let start = task.start();
        let initial = task.observe(&start).unwrap();
        assert_eq!(initial.active.exact(prefix).unwrap().origin, Origin::Static);
        assert_ne!(
            initial
                .ospf_candidate
                .unwrap()
                .next_hops
                .first()
                .unwrap()
                .interface,
            task.primary_interface
        );

        let branch_totals = task
            .branch_costs(&start)
            .unwrap()
            .into_iter()
            .map(|branch| branch.total.unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            branch_totals,
            if accepted_cost == "3" {
                [19, 32]
            } else {
                [35, 22]
            }
        );
        for distance in ["111", "240"] {
            let draft = Draft {
                cost: accepted_cost.into(),
                distance: distance.into(),
                mask_query: "255.255.255.192".into(),
            };
            let obs = task.observe(&draft).unwrap();
            let winner = obs.active.exact(prefix).unwrap();
            assert_eq!(winner.origin, Origin::Ospf);
            assert_eq!(winner.distance, 110);
            assert_eq!(winner.dotted_mask().to_string(), "255.255.255.192");
            assert_eq!(winner.origin.code(prefix), "O");
            let expected_cost = if accepted_cost == "3" { 15 } else { 18 };
            assert_eq!(winner.metric, expected_cost);
            assert_eq!(obs.active.uninstalled(prefix).len(), 1);
            assert!(task.outcomes(&draft).unwrap().iter().all(|row| row.passed));
            assert_eq!(obs.withdrawal.exact(prefix).unwrap().origin, Origin::Static);
            assert_eq!(
                obs.active
                    .lookup(task.probes[2])
                    .unwrap()
                    .destination
                    .length,
                24
            );
        }
        let alternate = Draft {
            cost: if accepted_cost == "3" { "4" } else { "6" }.into(),
            distance: "162".into(),
            mask_query: "255.255.255.192".into(),
        };
        assert!(
            task.outcomes(&alternate)
                .unwrap()
                .iter()
                .all(|row| row.passed)
        );
        let tied = Draft {
            cost: if accepted_cost == "3" { "7" } else { "9" }.into(),
            ..alternate.clone()
        };
        let output = task.observe(&tied).unwrap();
        assert_eq!(output.active.exact(prefix).unwrap().exits.len(), 2);
        assert!(!task.outcomes(&tied).unwrap()[1].passed);
        let wrong_cost = Draft {
            cost: start.cost,
            distance: "145".into(),
            mask_query: "255.255.255.192".into(),
        };
        let outcomes = task.outcomes(&wrong_cost).unwrap();
        assert!(outcomes[0].passed);
        assert!(!outcomes[1].passed);
        let no_backup = Draft {
            cost: accepted_cost.into(),
            distance: "255".into(),
            mask_query: "255.255.255.192".into(),
        };
        assert!(!task.outcomes(&no_backup).unwrap()[3].passed);
        let incomparable = Draft {
            cost: accepted_cost.into(),
            distance: "110".into(),
            mask_query: "255.255.255.192".into(),
        };
        assert!(
            task.observe(&incomparable)
                .err()
                .unwrap()
                .contains("unlike sources")
        );
    }
}

#[test]
fn local_network_and_default_codes_keep_their_own_scope() {
    let task = cases().assignment;
    let observed = task.observe(&task.start()).unwrap();
    for (index, code, mask) in [
        (3, "L", "255.255.255.255"),
        (4, "C", "255.255.255.252"),
        (5, "S*", "0.0.0.0"),
    ] {
        let entry = observed.active.lookup(task.probes[index]).unwrap();
        assert_eq!(entry.origin.code(entry.destination), code);
        assert_eq!(entry.dotted_mask().to_string(), mask);
    }
    let right = task.inspect(&observed.active, "255.255.255.192").unwrap();
    assert_eq!(right.destination, task.static_backup.destination);
    for invalid_mask in ["", "255.255.0.255", "255.255.255.0", "255.255.255.224"] {
        let invalid = Draft {
            mask_query: invalid_mask.into(),
            ..task.start()
        };
        assert!(
            !task.outcomes(&invalid).unwrap().last().unwrap().passed,
            "{invalid_mask}"
        );
    }
    let outside = Prefix::parse("172.16.0.0/24").unwrap();
    assert!(observed.active.exact(outside).is_none());
    assert_eq!(
        observed
            .active
            .lookup(task.probes[5])
            .unwrap()
            .destination
            .length,
        0
    );
    for cost in ["", "0", "65536", "10; ls"] {
        let invalid = Draft {
            cost: cost.into(),
            distance: "123".into(),
            mask_query: "255.255.255.192".into(),
        };
        assert!(task.observe(&invalid).err().unwrap().contains("OSPF cost"));
    }
}
