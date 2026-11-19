// Synthetic source-bound security fixtures; no network or host operations.
use dlms_learning_lab::{
    mac_learning::{Mac, State},
    port_security::{ViolationMode as V, World},
    port_security_workspace::*,
    switching::*,
};
use std::collections::{BTreeMap, BTreeSet};
pub fn mac(n: u8) -> Mac {
    Mac([2, 0, 0, 0, 0, n])
}
fn name(s: &str) -> Name {
    Name(s.into())
}
fn allow(host: &str, source: u8, dest: u8, target: &str) -> Frame {
    Frame {
        host: name(host),
        source: mac(source),
        destination: mac(dest),
        expected: Expected::Admitted {
            accepted_by: vec![name(target)],
        },
    }
}
fn deny(host: &str, source: u8, dest: u8, counted: Option<bool>) -> Frame {
    Frame {
        host: name(host),
        source: mac(source),
        destination: mac(dest),
        expected: Expected::Violation {
            counted,
            operational: true,
        },
    }
}
pub fn task(kind: u8, example: bool) -> Task {
    let prefix = if example { "sample" } else { "assigned" };
    let switch = format!("{prefix}-switch");
    let desk = format!("{prefix}-desk");
    let ops = format!("{prefix}-ops");
    let server = format!("{prefix}-server");
    let offset = if example { 20 } else { 0 };
    let d = offset + 1;
    let o = offset + 2;
    let s = offset + 3;
    let extra = offset + 8;
    let unknown = offset + 9;
    let mut fabric = Fabric {
        switches: BTreeMap::from([(
            switch.clone(),
            Switch {
                label: "Access switch".into(),
                vlans: BTreeSet::from([10]),
                ports: BTreeMap::new(),
            },
        )]),
        hosts: BTreeMap::new(),
        cables: vec![],
    };
    let mut addresses = BTreeMap::new();
    for (port, host, m) in [("desk", &desk, d), ("ops", &ops, o), ("server", &server, s)] {
        fabric.switches.get_mut(&switch).unwrap().ports.insert(
            port.into(),
            Port {
                up: true,
                mode: Mode::Access { vlan: 10 },
            },
        );
        fabric.hosts.insert(
            host.clone(),
            Host {
                label: host.clone(),
                up: true,
            },
        );
        addresses.insert(host.clone(), mac(m));
        fabric.cables.push(Cable {
            id: format!("wire-{port}"),
            a: Jack::Host { host: host.clone() },
            b: Jack::Switch {
                switch: switch.clone(),
                port: port.into(),
            },
            up: true,
        });
    }
    let mut initial = World::new(
        State::new(fabric, addresses, 300).unwrap(),
        &switch,
        &["desk", "ops", "server"],
    )
    .unwrap();
    for (port, m) in [("ops", o), ("server", s)] {
        initial
            .configure(port, true, 1, V::Restrict, vec![mac(m)])
            .unwrap();
    }
    let (maximum, ids, modes, counted, instructions) = match kind {
        1 => (
            1,
            vec![mac(d)],
            vec![V::Protect, V::Restrict],
            None,
            "Bind the approved desk source explicitly with maximum one. Reject an unexpected source even when it arrives first, keep approved traffic working afterward, and leave the neighboring ops and server settings unchanged.",
        ),
        2 => (
            2,
            vec![mac(d), mac(extra)],
            vec![V::Restrict],
            Some(true),
            "Allow both supplied approved desk sources with maximum two. Reject and count an unexpected third source while the port and approved traffic stay working. Leave the neighboring ops and server settings unchanged.",
        ),
        3 => (
            1,
            vec![mac(d)],
            vec![V::Protect, V::Restrict],
            None,
            "Replace the retired configured desk identity with the supplied current identity, retain maximum one and recover the error-disabled port manually. Reject unexpected sources without taking approved traffic offline again; preserve the neighboring ops and server settings.",
        ),
        _ => panic!("kind"),
    };
    match kind {
        1 => initial
            .configure("desk", true, 1, V::Restrict, vec![])
            .unwrap(),
        2 => initial
            .configure("desk", true, 2, V::Protect, ids.clone())
            .unwrap(),
        3 => {
            initial
                .configure("desk", true, 1, V::Shutdown, vec![mac(offset + 7)])
                .unwrap();
            initial.send(&desk, mac(d), mac(s)).unwrap();
        }
        _ => unreachable!(),
    }
    let settings = Settings {
        port: "desk".into(),
        enabled: true,
        maximum,
        mode: modes[0],
        configured: ids,
    };
    let mut reference = vec![Action::Configure {
        settings: settings.clone(),
    }];
    if kind == 3 {
        reference.extend([
            Action::Admin {
                port: name("desk"),
                up: false,
            },
            Action::Admin {
                port: name("desk"),
                up: true,
            },
        ]);
    }
    let mut cases = vec![
        Case {
            label: "Unexpected sender arrives first".into(),
            frames: vec![
                deny(&desk, unknown, s, counted),
                allow(&desk, d, s, &server),
            ],
        },
        Case {
            label: "Approved traffic continues after a rejection".into(),
            frames: vec![
                allow(&desk, d, s, &server),
                deny(&desk, unknown, s, counted),
                allow(&desk, d, s, &server),
            ],
        },
        Case {
            label: "Both directions and the neighbor still work".into(),
            frames: vec![
                allow(&ops, o, d, &desk),
                allow(&desk, d, s, &server),
                allow(&server, s, o, &ops),
            ],
        },
        Case {
            label: "A neighbor's secured address cannot move here".into(),
            frames: vec![deny(&desk, o, s, counted), allow(&ops, o, s, &server)],
        },
    ];
    if kind == 2 {
        cases.push(Case {
            label: "Second approved source also works".into(),
            frames: vec![
                allow(&desk, extra, s, &server),
                deny(&desk, unknown, s, counted),
                allow(&desk, d, s, &server),
            ],
        });
    }
    Task {
        id: name(&format!("security-{kind}-{prefix}")),
        instructions: instructions.into(),
        initial,
        required: vec![RequiredPort { settings, modes }],
        cases,
        reference,
    }
}
pub fn definition(kind: u8) -> Definition {
    Definition {
        initial: task(kind, false),
        example: task(kind, true),
    }
}
