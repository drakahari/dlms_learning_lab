// Same-project synthetic Ethernet fixture; no host interfaces or I/O.
use dlms_learning_lab::{
    mac_learning::{Mac, State},
    switching::Fabric,
};
use std::collections::{BTreeMap, BTreeSet};
pub fn mac(n: u8) -> Mac {
    Mac([2, 0, 0, 0, 0, n])
}
pub fn fixture() -> State {
    let fabric:Fabric=serde_json::from_str(r#"{
  "switches":{
   "s1":{"label":"Desk switch","vlans":[10,20,99],"ports":{
    "a":{"up":true,"mode":{"kind":"access","vlan":10}},
    "b":{"up":true,"mode":{"kind":"access","vlan":10}},
    "uplink":{"up":true,"mode":{"kind":"trunk","native":99,"allowed":[10,20,99]}}}},
   "s2":{"label":"Room switch","vlans":[10,20,99],"ports":{
    "c":{"up":true,"mode":{"kind":"access","vlan":10}},
    "d":{"up":true,"mode":{"kind":"access","vlan":20}},
    "uplink":{"up":true,"mode":{"kind":"trunk","native":99,"allowed":[10,20,99]}}}}},
  "hosts":{"a":{"label":"Alex","up":true},"b":{"label":"Blair","up":true},"c":{"label":"Casey","up":true},"d":{"label":"Drew","up":true}},
  "cables":[
   {"id":"wire-a","a":{"kind":"host","host":"a"},"b":{"kind":"switch","switch":"s1","port":"a"},"up":true},
   {"id":"wire-b","a":{"kind":"host","host":"b"},"b":{"kind":"switch","switch":"s1","port":"b"},"up":true},
   {"id":"wire-c","a":{"kind":"host","host":"c"},"b":{"kind":"switch","switch":"s2","port":"c"},"up":true},
   {"id":"wire-d","a":{"kind":"host","host":"d"},"b":{"kind":"switch","switch":"s2","port":"d"},"up":true},
   {"id":"between","a":{"kind":"switch","switch":"s1","port":"uplink"},"b":{"kind":"switch","switch":"s2","port":"uplink"},"up":true}]}
 "#).unwrap();
    State::new(
        fabric,
        BTreeMap::from([
            ("a".into(), mac(1)),
            ("b".into(), mac(2)),
            ("c".into(), mac(3)),
            ("d".into(), mac(4)),
        ]),
        300,
    )
    .unwrap()
}
pub fn names(xs: &[&str]) -> BTreeSet<String> {
    xs.iter().map(|s| s.to_string()).collect()
}
