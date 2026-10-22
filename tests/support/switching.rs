use dlms_learning_lab::switching::{Cable, Fabric, Host, Jack, Mode, Port, Switch};
use std::collections::{BTreeMap, BTreeSet};
pub fn access(vlan: u16) -> Port {
    Port {
        up: true,
        mode: Mode::Access { vlan },
    }
}
pub fn trunk(native: u16, allowed: &[u16]) -> Port {
    Port {
        up: true,
        mode: Mode::Trunk {
            native,
            allowed: allowed.iter().copied().collect(),
        },
    }
}
pub fn jack(s: &str, p: &str) -> Jack {
    Jack::Switch {
        switch: s.into(),
        port: p.into(),
    }
}
pub fn host(h: &str) -> Jack {
    Jack::Host { host: h.into() }
}
pub fn cable(id: &str, a: Jack, b: Jack) -> Cable {
    Cable {
        id: id.into(),
        a,
        b,
        up: true,
    }
}
pub fn fabric() -> Fabric {
    let switch = || Switch {
        label: "Office switch".into(),
        vlans: BTreeSet::from([10, 20, 99]),
        ports: BTreeMap::from([
            ("staff".into(), access(10)),
            ("guest".into(), access(20)),
            ("uplink".into(), trunk(99, &[10, 20, 99])),
        ]),
    };
    Fabric {
        switches: BTreeMap::from([("left".into(), switch()), ("right".into(), switch())]),
        hosts: ["staff-left", "staff-right", "guest-left", "guest-right"]
            .into_iter()
            .map(|id| {
                (
                    id.into(),
                    Host {
                        label: id.into(),
                        up: true,
                    },
                )
            })
            .collect(),
        cables: vec![
            cable("trunk", jack("left", "uplink"), jack("right", "uplink")),
            cable("sl", host("staff-left"), jack("left", "staff")),
            cable("sr", host("staff-right"), jack("right", "staff")),
            cable("gl", host("guest-left"), jack("left", "guest")),
            cable("gr", host("guest-right"), jack("right", "guest")),
        ],
    }
}
