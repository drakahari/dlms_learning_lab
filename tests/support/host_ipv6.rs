//! Synthetic source-only IPv6 facts; never catalog content or host networking.
use dlms_learning_lab::linux_host as host;
use serde_json::{Value, json};
fn raw() -> Value {
    let pack: Value =
        serde_json::from_str(include_str!("../../scenarios/linux-host-lessons-v1.json")).unwrap();
    pack["scenarios"][0]["goal"]["definition"]["initial"].clone()
}
pub fn fixture() -> host::State {
    let mut v = raw();
    v["profile"]["id"] = "ipv6-proof".into();
    v["profile"]["ipv6"] = json!({"lans":[{"device":"ens3","network":{"address":"2001:db8:10::","length":64},"link_local":"fe80::a"},{"device":"ens4","network":{"address":"2001:db8:20::","length":64},"link_local":"fe80::a"}],"gateways":[{"device":"ens3","address":"fe80::1","available":true,"exports":[{"address":"2001:db8:90::","length":64}]},{"device":"ens4","address":"fe80::1","available":false,"exports":[{"address":"2001:db8:90::","length":64}]}],"nodes":[{"address":"2001:db8:10::20","available":true},{"address":"2001:db8:20::20","available":true},{"address":"2001:db8:90::80","available":true},{"address":"2001:db8:10:8::80","available":true}]});
    v["configuration"]["interfaces"][0]["up"] = true.into();
    v["configuration"]["interfaces"][0]["address"] = json!({"address":"192.0.2.40","prefix":24});
    v["configuration"]["ipv6"] = json!({"interfaces":[{"device":"ens3","address":{"address":"2001:db8:10::40","prefix":64}},{"device":"ens4","address":{"address":"2001:db8:20::40","prefix":64}}],"routes":[]});
    let w: host::State = serde_json::from_value(v).unwrap();
    w.validate().unwrap();
    w
}
