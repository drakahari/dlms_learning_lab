#![allow(dead_code)]
use dlms_learning_lab::bond_control as b;
use std::collections::BTreeMap;
pub fn source() -> b::Source {
    b::Source {
        version: 1,
        controller: b::Binding {
            file: "bond.nmconnection".into(),
            uuid: "10000000-0000-4000-8000-000000000001".into(),
            interface: "bond0".into(),
        },
        members: [
            ("primary.nmconnection", "enp1s0", 2),
            ("backup.nmconnection", "enp2s0", 3),
        ]
        .into_iter()
        .map(|(file, interface, n)| b::Binding {
            file: file.into(),
            uuid: format!("10000000-0000-4000-8000-{n:012}"),
            interface: interface.into(),
        })
        .collect(),
    }
}
pub fn files() -> BTreeMap<String, String> {
    let s = source();
    std::iter::once(&s.controller).chain(&s.members).map(|x|{let controller=x.interface=="bond0";(x.file.clone(),format!("[connection]\nid={}\nuuid={}\ninterface-name={}\ntype={}\nautoconnect={}\n{}[ipv4]\nmethod=disabled\n[ipv6]\nmethod=disabled\n{}",x.interface,x.uuid,x.interface,if controller{"bond"}else{"ethernet"},if controller{"true"}else{"false"},if controller{"autoconnect-ports=1\n"}else{"controller=bond0\nport-type=bond\n"},if controller{"[bond]\nmode=active-backup\nmiimon=100\nprimary=enp1s0\nprimary_reselect=always\n"}else{""}))}).collect()
}
pub fn carrier(a: bool, c: bool) -> BTreeMap<String, bool> {
    BTreeMap::from([("enp1s0".into(), a), ("enp2s0".into(), c)])
}
