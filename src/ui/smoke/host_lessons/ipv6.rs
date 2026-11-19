//! Fixed three-lab review plan, not a learner command source or host-network bridge.
use super::*;
pub(super) fn route(labs: &[Scenario]) -> Vec<Op> {
    let ids = [
        "lpic2-host-ipv6-addition",
        "lpic2-host-ipv6-gateway-scope",
        "lpic2-host-dual-stack-handover",
    ];
    let data: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/host-ipv6-experiments-v1.json"
    ))
    .unwrap();
    let mut ops = vec![];
    for (id, recipe) in ids.into_iter().zip(data["experiments"].as_array().unwrap()) {
        assert_eq!(recipe["id"].as_str(), Some(id));
        let s = labs.iter().find(|s| s.id == id).unwrap();
        let engine::Goal::HostNetwork { definition } = &s.goal else {
            panic!()
        };
        assert!(definition.initial.profile.ipv6.is_some());
        ops.extend([
            Op::Open(id.into()),
            Op::Click("example".into()),
            Op::Click("fill-example".into()),
            Op::Unrun,
            Op::Example(false),
        ]);
        for e in &definition.example_actions {
            action(&mut ops, e);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Example(true),
            Op::Shot(format!("host-ipv6-{id}-practice-pass")),
        ]);
        for c in recipe["commands"].as_array().unwrap() {
            action(
                &mut ops,
                &work::Event::Command {
                    input: c.as_str().unwrap().into(),
                },
            );
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Rows(
                serde_json::from_value(recipe["expected"].clone()).unwrap(),
                true,
            ),
            Op::Shot(format!("host-ipv6-{id}-practice-adverse")),
        ]);
        for c in recipe["repair"].as_array().unwrap() {
            action(
                &mut ops,
                &work::Event::Command {
                    input: c.as_str().unwrap().into(),
                },
            );
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Example(true),
            Op::Click("assignment".into()),
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(format!("host-ipv6-{id}-initial-feedback")),
        ]);
        for (n, c) in s.solution.iter().enumerate() {
            let e = serde_json::from_str(c.strip_prefix("host-action ").unwrap()).unwrap();
            action(&mut ops, &e);
            if n == 0 && s.solution.len() > 1 {
                ops.extend([
                    Op::Click("check".into()),
                    Op::Solved(false),
                    Op::Shot(format!("host-ipv6-{id}-partial-feedback")),
                ]);
            }
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Save,
            Op::Shot(format!("host-ipv6-{id}-assignment-pass")),
        ]);
        action(
            &mut ops,
            &work::Event::Command {
                input: "ip -6 route del 2001:db8:140::/64 via fe80::1 dev ens4 metric 50".into(),
            },
        );
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(format!("host-ipv6-{id}-success-invalidated")),
        ]);
        action(
            &mut ops,
            &work::Event::Command {
                input: "ip -6 route add 2001:db8:140::/64 via fe80::1 dev ens4 metric 50".into(),
            },
        );
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Save,
            Op::Theme(1),
            Op::Field("probe".into(), "2001:db8:110::20".into()),
            Op::Click("probe-run".into()),
            Op::ProbeContains("IPv6 path available"),
            Op::ControlShot(format!("host-ipv6-{id}-light150-probe"), "probe".into()),
            Op::Theme(2),
            Op::Field("probe".into(), "2001:db8:110:0:ffff:ffff:ffff:ffff".into()),
            Op::Click("probe-run".into()),
            Op::ProbeContains("No available supplied IPv6 destination"),
            Op::ControlShot(format!("host-ipv6-{id}-dark200-long-input"), "probe".into()),
            Op::Solved(true),
            Op::Save,
            Op::Theme(0),
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Fresh,
            Op::Click("challenge-header".into()),
            Op::Challenge(true),
            Op::ControlShot(
                format!("host-ipv6-{id}-challenge-default"),
                "reset-header".into(),
            ),
            Op::Click("learn-header".into()),
            Op::Challenge(false),
        ]);
    }
    ops.extend([
        Op::Open("lpic1-host-address".into()),
        Op::Click("assignment".into()),
        Op::ControlShot(
            "host-ipv6-existing-ipv4-unchanged".into(),
            "reset-header".into(),
        ),
        Op::Solved(false),
    ]);
    ops
}
