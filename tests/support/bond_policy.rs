#![allow(dead_code)]
#[path = "bond_control.rs"]
mod controls;
#[path = "vlan_host.rs"]
mod vlan;
use dlms_learning_lab::{bond_control as c, bond_host, nm_policy as n, nm_workspace as w};
pub fn environment(base: u8) -> n::Environment {
    let mut e = vlan::environment();
    let s = serde_json::to_string(&e)
        .unwrap()
        .replace("10.20.", &format!("10.{base}."))
        .replace("10.21.", &format!("10.{}.", base + 1));
    e = serde_json::from_str(&s).unwrap();
    let mut source = controls::source();
    for b in std::iter::once(&mut source.controller).chain(&mut source.members) {
        b.uuid = b.uuid.replace("10000000", &format!("1000{base:04}"));
    }
    e.bond = Some(bond_host::Facts {
        version: 1,
        profiles: source,
    });
    let dlms_learning_lab::switching::Mode::Trunk { native, .. } = &mut e
        .vlan
        .as_mut()
        .unwrap()
        .fabric
        .switches
        .get_mut("sw")
        .unwrap()
        .ports
        .get_mut("native")
        .unwrap()
        .mode
    else {
        panic!()
    };
    *native = 99;
    e
}
pub fn files(base: u8) -> w::Files {
    let mut f = controls::files()
        .into_iter()
        .map(|(n, t)| (n, t.replace("10000000", &format!("1000{base:04}"))))
        .collect::<w::Files>();
    for (n, t) in vlan::files() {
        f.insert(
            n,
            t.replace("parent=enp1s0", "parent=bond0")
                .replace("parent=enp2s0", "parent=bond0")
                .replace("10.20.", &format!("10.{base}."))
                .replace("10.21.", &format!("10.{}.", base + 1))
                .replace("000000000001", &format!("{base:012}"))
                .replace("000000000002", &format!("{:012}", base + 1)),
        );
    }
    f
}
pub fn current() -> c::Current {
    c::Current {
        controller: true,
        members: vec![
            c::InterfaceName("enp1s0".into()),
            c::InterfaceName("enp2s0".into()),
        ],
    }
}
pub fn future(env: &n::Environment, files: &w::Files) -> n::Choices {
    let p = n::Policy::from_texts_for(env, files).unwrap();
    n::Choices::empty().automatic(env, &p).unwrap().0
}
pub fn initial(env: &n::Environment, files: &w::Files) -> n::Choices {
    let p = n::Policy::from_texts_for(env, files).unwrap();
    let mut c = n::Choices::empty()
        .seed_current_controls(env, &p, &current())
        .unwrap();
    for (_, profile) in p.profiles() {
        c = c.manual(env, &p, &profile.uuid).unwrap();
    }
    c
}
pub fn ip(s: &str) -> std::net::Ipv4Addr {
    s.parse().unwrap()
}
