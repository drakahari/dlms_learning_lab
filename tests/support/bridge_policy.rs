#![allow(dead_code)]
#[path = "bridge_host.rs"]
pub mod model;
use dlms_learning_lab::nm_policy as n;
pub fn environment() -> n::Environment {
    n::Environment {
        bridge: Some(model::facts()),
        bond: None,
        vlan: None,
        physical: model::physical(),
        default_metrics: [("br0".into(), 100)].into(),
        dhcp: Default::default(),
    }
}
pub fn files() -> std::collections::BTreeMap<String, String> {
    model::files()
}
pub fn future(
    env: &n::Environment,
    files: &std::collections::BTreeMap<String, String>,
) -> n::Choices {
    n::Choices::empty()
        .automatic(env, &n::Policy::from_texts_for(env, files).unwrap())
        .unwrap()
        .0
}
