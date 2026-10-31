//! Synthetic capability fixture only, never a bundled/countable assignment.
#![allow(dead_code)] // Integration and native capability groups import different subsets.
use dlms_learning_lab::{
    engine::{Goal, State},
    hsrp as core, hsrp_workspace as work,
    pack::Scenario,
};
use std::net::Ipv4Addr;
pub fn config(priority: u8, preempt: bool) -> core::RouterConfig {
    core::RouterConfig {
        participating: true,
        priority,
        preempt,
        track_wan_decrement: None,
    }
}
fn world(practice: bool) -> core::State {
    let ip = |n| {
        if practice {
            Ipv4Addr::new(198, 51, 100, n)
        } else {
            Ipv4Addr::new(192, 0, 2, n)
        }
    };
    let name = if practice { "willow" } else { "alder" };
    let router = |n, priority, participating| core::Router {
        label: format!("{name}-{n}"),
        address: ip(n),
        config: core::RouterConfig {
            participating,
            ..config(priority, false)
        },
        facts: core::RouterFacts {
            power_on: true,
            lan_link_up: true,
            wan_link_up: true,
            route_available: true,
        },
    };
    core::State::new(
        7,
        24,
        ip(254),
        [router(1, 110, true), router(2, 100, false)],
        vec![
            core::Client {
                id: 1,
                label: format!("{name}-desk"),
                address: ip(20),
                gateway: ip(1),
                link_up: true,
            },
            core::Client {
                id: 2,
                label: format!("{name}-counter"),
                address: ip(21),
                gateway: ip(1),
                link_up: true,
            },
        ],
    )
    .unwrap()
}
pub fn actions(d: &work::Definition, practice: bool) -> Vec<work::Action> {
    let s = if practice {
        &d.lesson.policy.initial
    } else {
        &d.policy.initial
    };
    vec![
        work::Action::Router {
            router: core::RouterId::B,
            config: config(100, false),
        },
        work::Action::Gateway {
            client: 1,
            gateway: s.virtual_ip(),
        },
        work::Action::Gateway {
            client: 2,
            gateway: s.virtual_ip(),
        },
    ]
}
fn policy(practice: bool) -> work::Policy {
    work::Policy {
        initial: world(practice),
        editable_routers: core::RouterId::ALL.to_vec(),
        editable_clients: vec![1, 2],
        normal_active: None,
        probes: core::RouterId::ALL
            .map(|router| work::Probe {
                title: format!("Router{router:?} power failure and return"),
                events: vec![
                    work::Event::Power { router, up: false },
                    work::Event::Power { router, up: true },
                ],
                expected_active: vec![None, None],
            })
            .to_vec(),
    }
}
pub fn definition() -> work::Definition {
    let p = policy(true);
    let mut d=work::Definition{policy:policy(false),lesson:work::Lesson{prerequisite_note:"Know IPv4 addresses and a host's off-LAN default gateway. Both supplied routers already have usable upstream routes.".into(),concepts:vec![work::Concept{title:"Virtual gateway".into(),text:"Clients use one virtual address; either eligible router can serve it. A real router address stays tied to that router.".into()},work::Concept{title:"Settled roles and history".into(),text:"An eligible incumbent stays Active unless a strictly higher-priority peer is configured to preempt. An eligible standby takes over on power or LAN loss without preemption. Real HSRP also depends on timers and packets, which are omitted here.".into()},work::Concept{title:"Tracking and independent routing".into(),text:"A WAN failure alone does not remove the Active. Optional WAN-link tracking lowers its effective priority; the higher-priority peer still needs preemption. A lost route is a different supplied fact and is not route tracking.".into()}],description:"Separate synthetic Willow world for capability tests; no curriculum registration.".into(),policy:p,worked:vec![],try_change:"Point one example client at the real router address; observe loss of the virtual-gateway requirement.".into(),experiment:work::Action::Gateway{client:1,gateway:Ipv4Addr::new(198,51,100,1)},repair:work::Action::Gateway{client:1,gateway:Ipv4Addr::new(198,51,100,254)}}};
    d.lesson.worked = actions(&d, true)
        .into_iter()
        .enumerate()
        .map(|(i, action)| work::Worked {
            title: format!("Example setting{}", i + 1),
            why: "Apply this native configuration setting, then check both failure paths.".into(),
            action,
        })
        .collect();
    d
}
pub fn scenario() -> Scenario {
    let d = definition();
    let solution = actions(&d, false)
        .iter()
        .map(|a| format!("{}{}", work::PREFIX, serde_json::to_string(a).unwrap()))
        .collect::<Vec<_>>();
    let initial = State::Hsrp {
        simulation: Box::new(work::Session::new(&d).unwrap()),
    };
    serde_json::from_value(serde_json::json!({"id":"private-hsrp-source-boundary","version":1,"track":"CCNA","topic":"HSRP capability fixture","unit":"private-hsrp-foundation","title":"Private HSRP fixture, not authored curriculum","summary":"Exercise native state and source authority only.","objective":"Keep both supplied clients on the virtual gateway and preserve forwarding across either router's power failure and return.","retry_guidance":"Check both candidates, every client gateway and every temporary failure result.","difficulty":"Intermediate","minutes":10,"steps":["Inspect fixed facts","Configure both candidates and gateways","Check both failure paths"],"guide":[],"explanation":"SIM HSRP is a typed test adapter, not an IOS command.","hints":["Use the virtual gateway on both clients."],"sources":["https://www.cisco.com/c/en/us/support/docs/ip/hot-standby-router-protocol-hsrp/9281-3.html"],"simplifications":"Connected LAN; already-settled events; no timers, ARP, packet exchange or host networking.","initial":initial,"goal":Goal::Hsrp{definition:Box::new(d)},"solution":solution,"prerequisites":["ccna-ipv4"]})).unwrap()
}
