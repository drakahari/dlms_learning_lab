use dlms_learning_lab::automation_workshop::{access_defaults, svi_draft, svi_projection};

fn access() -> access_defaults::Task {
    let source: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/default-access-fixtures.json")).unwrap();
    serde_json::from_value(source["assignment"].clone()).unwrap()
}
fn routed() -> svi_projection::Task {
    let source: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/routed-vlan-fixtures.json")).unwrap();
    serde_json::from_value(source["assignment"].clone()).unwrap()
}

#[test]
fn default_and_explicit_one_are_equivalent_with_a_different_native_vlan() {
    let task = access();
    task.validate().unwrap();
    let by_default = task.configured(task.initial).unwrap();
    assert_eq!(
        by_default,
        task.configured(access_defaults::Choice { access: Some(1) })
            .unwrap()
    );
    let initial = task.outcomes(task.initial).unwrap();
    assert_eq!(
        initial.iter().map(|r| r.passed).collect::<Vec<_>>(),
        [false, true, false, false]
    );
    let wrong_native = task
        .outcomes(access_defaults::Choice { access: Some(88) })
        .unwrap();
    assert_eq!(
        wrong_native.iter().map(|r| r.passed).collect::<Vec<_>>(),
        [false, true, true, true]
    );
    let repair = task
        .outcomes(access_defaults::Choice { access: Some(40) })
        .unwrap();
    assert!(repair.iter().all(|r| r.passed));
    assert!(
        task.configured(access_defaults::Choice { access: Some(4095) })
            .is_err()
    );
}

fn completed(last: u8) -> svi_draft::Draft {
    use svi_draft::AddressText;
    let task = routed();
    let mut draft = svi_draft::Draft::from(&task.initial);
    draft.routing = true;
    draft.enable = [true; 2];
    draft.allow_remote = [true; 2];
    draft.interfaces = [
        AddressText(format!("10.45.1.{last}/24")),
        AddressText(format!("10.70.1.{last}/24")),
    ];
    draft.gateways = [
        AddressText(format!("10.45.1.{last}")),
        AddressText(format!("10.45.1.{last}")),
        AddressText(format!("10.70.1.{last}")),
    ];
    draft
}

#[test]
fn independent_gateway_choices_can_route_without_merging_broadcasts() {
    let task = routed();
    task.validate().unwrap();
    for last in [1, 254] {
        let edit = completed(last).compile();
        let result = task.outcomes(&edit).unwrap();
        assert_eq!(result.len(), 6);
        assert!(result.iter().all(|r| r.passed));
        assert!(task.ethernet(&edit, 0, 1).unwrap().mutual());
        assert!(task.ethernet(&edit, 0, 2).unwrap().isolated());
        for i in [0, 1] {
            let probe = task.probe(&edit, i, 2).unwrap();
            assert_eq!(probe.forward.hops[0].peer, "gateway");
            assert_eq!(probe.reply.hops[0].peer, "gateway");
            assert_eq!(probe.forward.hops.len(), 2);
            assert_eq!(probe.reply.hops.len(), 2);
        }
    }
}

#[test]
fn enabled_svi_is_not_proof_of_remote_admission_or_ip_forwarding() {
    use svi_projection::Availability::*;
    let task = routed();
    let mut draft = completed(1);
    draft.allow_remote[1] = false;
    assert_eq!(task.availability(&draft.compile()).unwrap(), [Up, Up]);
    assert!(
        task.probe(&draft.compile(), 0, 2)
            .unwrap_err()
            .contains("neighbor")
    );
    let mut draft = completed(1);
    draft.routing = false;
    assert!(
        task.probe(&draft.compile(), 0, 2)
            .unwrap_err()
            .contains("forwarding is disabled")
    );
    assert!(task.ethernet(&draft.compile(), 0, 1).unwrap().mutual());
    let mut task = task;
    task.fabric
        .cables
        .iter_mut()
        .find(|c| c.id == "floor-link")
        .unwrap()
        .up = false;
    assert_eq!(
        task.availability(&completed(1).compile()).unwrap(),
        [Up, NoForwardingMembers]
    );
}

#[test]
fn wrong_return_gateway_and_bad_prefix_fail_while_drafts_roundtrip() {
    let task = routed();
    let mut draft = completed(254);
    draft.gateways[2].0 = "10.70.1.1".into();
    assert!(
        task.probe(&draft.compile(), 0, 2)
            .unwrap_err()
            .contains("reply fails")
    );
    draft.interfaces[1].0 = "10.70.1.254/".into();
    assert!(draft.interfaces[1].cidr().is_err());
    let restored: svi_draft::Draft =
        serde_json::from_str(&serde_json::to_string(&draft).unwrap()).unwrap();
    assert_eq!(draft, restored);
    assert!(restored.valid());
    assert!(
        !task
            .outcomes(&draft.compile())
            .unwrap()
            .iter()
            .all(|r| r.passed)
    );
    let mut draft = completed(1);
    draft.interfaces[0].0 = "10.45.1.22/24".into();
    assert!(
        task.outcomes(&draft.compile()).unwrap()[0]
            .detail
            .contains("already assigned")
    );
}
