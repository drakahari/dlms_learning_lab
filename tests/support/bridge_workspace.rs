#![allow(dead_code)]
#[path = "bridge_policy.rs"]
mod foundation;
use dlms_learning_lab::{
    bridge_control, bridge_fabric::Interface, ipv4_routing::Prefix, linux_host, nm_policy as model,
    nm_workspace as w,
};
pub fn task(id: &str, base: u8) -> w::Task {
    let encoded = serde_json::to_string(&(foundation::environment(), foundation::files()))
        .unwrap()
        .replace("10.220.", &format!("10.{base}."))
        .replace("10.230.", &format!("10.{}.", base + 10))
        .replace("30000000", &format!("{base:08x}"));
    let (mut env, mut good): (model::Environment, w::Files) =
        serde_json::from_str(&encoded).unwrap();
    let audit_uuid = format!("{base:08x}-0000-4000-8000-000000000004");
    env.physical.links.push(linux_host::Link {
        name: "mgmt0".into(),
        network: Prefix::parse(&format!("10.{}.0.0/24", base + 1)).unwrap(),
        carrier: true,
    });
    env.default_metrics.insert("mgmt0".into(), 200);
    env.physical.nodes.push(linux_host::Node {
        address: format!("10.{}.0.50", base + 1).parse().unwrap(),
        available: true,
    });
    good.insert("audit.nmconnection".into(),format!("[connection]\nid=Audit\nuuid={audit_uuid}\ntype=ethernet\ninterface-name=mgmt0\nautoconnect=true\n[ipv4]\nmethod=manual\naddress1=10.{}.0.20/24\nnever-default=true\n[ipv6]\nmethod=disabled\n",base+1));
    let source = env.bridge.as_ref().unwrap().profiles.clone();
    let mut starter = good.clone();
    *starter.get_mut("bridge.nmconnection").unwrap() = starter["bridge.nmconnection"]
        .replace("autoconnect-ports=1", "autoconnect-ports=0")
        .replace(
            &format!("10.{base}.0.20/24"),
            &format!("10.{}.0.20/24", base + 2),
        );
    *starter.get_mut("right.nmconnection").unwrap() = starter["right.nmconnection"]
        .replace("controller=br0\nport-type=bridge\n", "")
        .replace(
            "[bridge-port]",
            "[ipv4]\nmethod=disabled\n[ipv6]\nmethod=disabled",
        );
    let cases = (0..3)
        .map(|which| {
            let mut environment = env.clone();
            if which == 1 {
                environment
                    .bridge
                    .as_mut()
                    .unwrap()
                    .lan
                    .fabric
                    .hosts
                    .get_mut("nic-a")
                    .unwrap()
                    .up = false;
            }
            if which == 2 {
                environment
                    .bridge
                    .as_mut()
                    .unwrap()
                    .lan
                    .fabric
                    .hosts
                    .get_mut("nic-b")
                    .unwrap()
                    .up = false;
            }
            let mut requirements = vec![
                w::Requirement::CurrentUnchanged {
                    interface: "br0".into(),
                },
                w::Requirement::CurrentUnchanged {
                    interface: "mgmt0".into(),
                },
                w::Requirement::FutureProfile {
                    interface: "br0".into(),
                    uuid: source.controller.uuid.clone(),
                },
                w::Requirement::FutureNetwork {
                    interface: "br0".into(),
                    network: env.physical.links[0].network,
                },
                w::Requirement::FutureNoDefault {
                    interface: "mgmt0".into(),
                },
                w::Requirement::FuturePath {
                    target: format!("10.{}.0.50", base + 1).parse().unwrap(),
                    interface: "mgmt0".into(),
                },
            ];
            if which != 1 {
                requirements.push(w::Requirement::FuturePath {
                    target: format!("10.{base}.0.50").parse().unwrap(),
                    interface: "br0".into(),
                });
            }
            if which != 2 {
                requirements.push(w::Requirement::FuturePath {
                    target: format!("10.{base}.0.60").parse().unwrap(),
                    interface: "br0".into(),
                });
            }
            requirements.push(w::Requirement::FutureBridgeDelivery {
                interface: "br0".into(),
                from: format!("10.{base}.0.50").parse().unwrap(),
                to: format!("10.{base}.0.60").parse().unwrap(),
                delivered: which == 0,
            });
            w::Case {
                current_bridge: Some(bridge_control::Current {
                    controller: true,
                    members: vec![Interface("enp1s0".into()), Interface("enp2s0".into())],
                }),
                current_controls: None,
                id: ["both-up", "left-unavailable", "right-unavailable"][which].into(),
                label: [
                    "Both bridge ports have carrier",
                    "Left segment has no physical attachment",
                    "Right segment has no physical attachment",
                ][which]
                    .into(),
                environment,
                current_files: good.clone(),
                initially_selected: vec![source.controller.uuid.clone(), audit_uuid.clone()],
                requirements,
            }
        })
        .collect();
    w::Task{id:id.into(),instructions:"Prepare the saved bridge for both untagged LAN segments. Use the supplied LAN prefix on the controller, attach both intended physical ports and inspect each physical case. Preserve current bridge/membership and Audit settings; a down port must isolate its segment, not invent a backup path.".into(),starter,protected_profiles:vec!["audit.nmconnection".into()],cases}
}
pub fn solved(t: &w::Task) -> w::Files {
    t.cases[0].current_files.clone()
}
pub fn partial(t: &w::Task) -> w::Files {
    let mut f = t.starter.clone();
    let good = solved(t);
    f.insert(
        "bridge.nmconnection".into(),
        good["bridge.nmconnection"].clone(),
    );
    f
}
pub fn definition() -> w::Definition {
    let initial = task("bridge-commission-test", 180);
    let example = task("bridge-guided-test", 200);
    let example_worked = solved(&example);
    w::Definition {
        initial,
        example,
        example_worked,
    }
}
