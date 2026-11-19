use super::*;
use serde_json::json;
pub(crate) fn world() -> World {
    let names = ["ap", "controller", "staff", "guest"];
    let mut hosts = serde_json::Map::new();
    let mut ports = serde_json::Map::new();
    let mut cables = Vec::new();
    for name in names {
        hosts.insert(name.into(), json!({"label":name,"up":true}));
        let mode = match name {
            "ap" | "controller" => json!({"kind":"trunk","native":10,"allowed":[10,20,30]}),
            "staff" => json!({"kind":"access","vlan":20}),
            "guest" => json!({"kind":"access","vlan":30}),
            _ => json!({"kind":"access","vlan":10}),
        };
        ports.insert(name.into(), json!({"up":true,"mode":mode}));
        cables.push(json!({"id":format!("cable-{name}"),"a":{"kind":"host","host":name},"b":{"kind":"switch","switch":"access","port":name},"up":true}));
    }
    serde_json::from_value(json!({
        "fabric":{"switches":{"access":{"label":"Access switch","vlans":[10,20,30],"ports":ports}},"hosts":hosts,"cables":cables},
        "ap":"ap","controller":"controller",
        "mode":"local","ap_native":10,"controller_native":10,
        "wlans":[{"id":"staff-wlan","label":"Staff WLAN","forwarding":"local","local_vlan":20,"controller_vlan":20},
            {"id":"guest-wlan","label":"Guest WLAN","forwarding":"central","local_vlan":30,"controller_vlan":30}]
    })).unwrap()
}
