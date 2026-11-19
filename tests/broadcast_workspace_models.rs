use dlms_learning_lab::automation_workshop::ospf_broadcast::{
    election::{Relationship, Role},
    workspace::{Draft, Task},
};
use serde::Deserialize;

#[derive(Deserialize)]
struct WorkOrders {
    example: Task,
    assignment: Task,
}

fn fixture() -> WorkOrders {
    serde_json::from_str(include_str!("fixtures/ospf-broadcast.json")).unwrap()
}

fn draft(priorities: [&str; 3]) -> Draft {
    Draft {
        priorities: priorities.map(str::to_owned),
    }
}

#[test]
fn independent_priority_repairs_enable_both_handover_clauses() {
    let tasks = fixture();
    for task in [tasks.example, tasks.assignment] {
        task.validate().unwrap();
        let initial = task.outcomes(&task.start()).unwrap();
        assert!(!initial[0].passed);
        assert!(!initial[4].passed, "Holding DR throughout is not promotion");

        for good in [draft(["120", "65", "0"]), draft(["100", "100", "0"])] {
            let states = task.observe(&good).unwrap();
            assert_eq!(
                states.fresh.roles.member_role(&states.fresh.members, 0),
                Role::Designated
            );
            assert_eq!(states.arrival.roles.backup, Some(1));
            assert_eq!(
                states
                    .arrival
                    .roles
                    .relationship(&states.arrival.members, 2, 3),
                Relationship::RemainTwoWay
            );
            assert_eq!(states.outage.roles.designated, Some(1));
            assert_eq!(states.outage.roles.backup, Some(3));
            assert!(!states.outage.members[0].present);
            assert!(task.outcomes(&good).unwrap().iter().all(|item| item.passed));
        }

        let quiet_neighbor_is_still_eligible = task.outcomes(&draft(["100", "90", "1"])).unwrap();
        assert!(!quiet_neighbor_is_still_eligible[3].passed);
        let visitor_lower_priority = task.outcomes(&draft(["255", "90", "0"])).unwrap();
        assert!(!visitor_lower_priority[2].passed);
        assert!(
            visitor_lower_priority[4].passed,
            "Higher-priority demonstration is independent of promotion"
        );
        assert!(
            task.observe(&draft(["xyz", "90", "0"]))
                .err()
                .unwrap()
                .contains(&task.members[0].label)
        );
    }
}
