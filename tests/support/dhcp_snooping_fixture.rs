// Same-project synthetic access-switch fixture; no host state or interfaces.
use dlms_learning_lab::{
    mac_learning::{Mac, State},
    switching::{Cable, Fabric, Host, Jack, Mode, Port, Switch},
};
use std::collections::{BTreeMap, BTreeSet};
pub fn mac(n: u8) -> Mac {
    Mac([2, 0, 0, 0, 0, n])
}
pub fn names(s: &[&str]) -> BTreeSet<String> {
    s.iter().map(|v| (*v).into()).collect()
}
pub fn fixture() -> State {
    let mut fabric = Fabric {
        switches: BTreeMap::from([(
            "edge".into(),
            Switch {
                label: "Supplied access switch".into(),
                vlans: BTreeSet::from([10, 20]),
                ports: BTreeMap::new(),
            },
        )]),
        hosts: BTreeMap::new(),
        cables: vec![],
    };
    let mut addresses = BTreeMap::new();
    for (i, name) in ["client", "server", "peer", "other"]
        .into_iter()
        .enumerate()
    {
        fabric.switches.get_mut("edge").unwrap().ports.insert(
            name.into(),
            Port {
                up: true,
                mode: Mode::Access {
                    vlan: if name == "other" { 20 } else { 10 },
                },
            },
        );
        fabric.hosts.insert(
            name.into(),
            Host {
                label: name.into(),
                up: true,
            },
        );
        addresses.insert(name.into(), mac(i as u8 + 1));
        fabric.cables.push(Cable {
            id: format!("wire-{name}"),
            a: Jack::Host { host: name.into() },
            b: Jack::Switch {
                switch: "edge".into(),
                port: name.into(),
            },
            up: true,
        });
    }
    State::new(fabric, addresses, 300).unwrap()
}
