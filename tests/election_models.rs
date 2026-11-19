use dlms_learning_lab::automation_workshop::ospf_broadcast::election::{
    Participant, Relationship, Role, Roles,
};

fn member(label: &str, id: &str, priority: u8, present: bool) -> Participant {
    Participant {
        label: label.into(),
        router_id: id.parse().unwrap(),
        priority,
        present,
    }
}

#[test]
fn fresh_election_compares_priority_then_numeric_router_id() {
    let mut members = vec![
        member("District", "192.0.2.10", 10, true),
        member("Station", "192.0.2.9", 10, true),
        member("Archive", "192.0.2.50", 0, true),
        member("Monitor", "192.0.2.110", 0, true),
    ];
    let roles = Roles::fresh(&members).unwrap();
    assert_eq!((roles.designated, roles.backup), (Some(0), Some(1)));
    assert_eq!(roles.member_role(&members, 2), Role::Other);
    assert_eq!(
        roles.relationship(&members, 2, 3),
        Relationship::RemainTwoWay
    );
    assert_eq!(
        roles.relationship(&members, 2, 0),
        Relationship::WouldAttemptAdjacency
    );

    members[1].priority = 22;
    let roles = Roles::fresh(&members).unwrap();
    assert_eq!((roles.designated, roles.backup), (Some(1), Some(0)));
    members[0].priority = 0;
    assert_eq!(Roles::fresh(&members).unwrap().backup, None);
    members[1].priority = 0;
    let roles = Roles::fresh(&members).unwrap();
    assert_eq!((roles.designated, roles.backup), (None, None));
    assert_eq!(
        roles.relationship(&members, 1, 2),
        Relationship::RemainTwoWay
    );
}

#[test]
fn join_does_not_preempt_and_the_existing_backup_takes_over_on_loss() {
    let mut members = vec![
        member("Old", "192.0.2.11", 100, true),
        member("Spare", "192.0.2.12", 90, true),
        member("New", "192.0.2.13", 230, false),
        member("Edge", "192.0.2.40", 0, true),
    ];
    let initial = Roles::fresh(&members).unwrap();
    assert_eq!((initial.designated, initial.backup), (Some(0), Some(1)));
    members[2].present = true;
    assert_eq!(Roles::settle(&members, initial).unwrap(), initial);
    assert_eq!(Roles::fresh(&members).unwrap().designated, Some(2));

    members[0].present = false;
    let failure = Roles::settle(&members, initial).unwrap();
    assert_eq!((failure.designated, failure.backup), (Some(1), Some(2)));
    assert_eq!(
        failure.relationship(&members, 0, 3),
        Relationship::NoCurrentNeighbor
    );
    assert_eq!(
        failure.relationship(&members, 1, 3),
        Relationship::WouldAttemptAdjacency
    );
    members[0].present = true;
    assert_eq!(Roles::settle(&members, failure).unwrap(), failure);

    members[1].priority = 0;
    let ineligible = Roles::settle(&members, failure).unwrap();
    assert_eq!(
        (ineligible.designated, ineligible.backup),
        (Some(2), Some(0))
    );
    assert_eq!(
        ineligible.relationship(&members, 1, 3),
        Relationship::RemainTwoWay
    );
}
