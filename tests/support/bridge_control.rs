#![allow(dead_code)]
use dlms_learning_lab::{bond_control::Binding, bridge_control as b};
use std::collections::BTreeMap;
pub fn source() -> b::Source {
    let bind = |file: &str, n: u8, interface: &str| Binding {
        file: file.into(),
        uuid: format!("30000000-0000-4000-8000-{n:012}"),
        interface: interface.into(),
    };
    b::Source {
        version: 1,
        controller: bind("bridge.nmconnection", 1, "br0"),
        members: vec![
            bind("left.nmconnection", 2, "enp1s0"),
            bind("right.nmconnection", 3, "enp2s0"),
        ],
    }
}
pub fn files() -> BTreeMap<String, String> {
    let source = source();
    let mut files = BTreeMap::new();
    files.insert(source.controller.file.clone(),format!("[connection]\nid=Data bridge\nuuid={}\ntype=bridge\ninterface-name=br0\nautoconnect=true\nautoconnect-ports=1\n[ethernet]\n[bridge]\n[ipv4]\nmethod=manual\naddress1=10.220.0.20/24\n[ipv6]\nmethod=disabled\naddr-gen-mode=default\n[proxy]\n",source.controller.uuid));
    for port in &source.members {
        files.insert(port.file.clone(),format!("[connection]\nid={}\nuuid={}\ntype=ethernet\ninterface-name={}\nautoconnect=false\ncontroller=br0\nport-type=bridge\n[ethernet]\n[bridge-port]\n",port.interface,port.uuid,port.interface));
    }
    files
}
