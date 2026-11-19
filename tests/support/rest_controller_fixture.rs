use dlms_learning_lab::{
    json_inventory as inventory, rest_controller as api, switching as ethernet,
};
use std::collections::{BTreeMap, BTreeSet};

pub fn inventory() -> inventory::Spec {
    let endpoints = ["staff-a", "staff-b", "guest-a", "guest-b"];
    let fabric = ethernet::Fabric {
        switches: BTreeMap::from([(
            "access".into(),
            ethernet::Switch {
                label: "Access switch".into(),
                vlans: BTreeSet::from([10, 20, 30]),
                ports: (1..=4)
                    .map(|i| {
                        (
                            format!("p{i}"),
                            ethernet::Port {
                                up: true,
                                mode: ethernet::Mode::Access { vlan: 10 },
                            },
                        )
                    })
                    .collect(),
            },
        )]),
        hosts: endpoints
            .into_iter()
            .map(|name| {
                (
                    name.into(),
                    ethernet::Host {
                        label: name.into(),
                        up: true,
                    },
                )
            })
            .collect(),
        cables: endpoints
            .into_iter()
            .enumerate()
            .map(|(i, endpoint)| ethernet::Cable {
                id: format!("c{i}"),
                a: ethernet::Jack::Host {
                    host: endpoint.into(),
                },
                b: ethernet::Jack::Switch {
                    switch: "access".into(),
                    port: format!("p{}", i + 1),
                },
                up: true,
            })
            .collect(),
    };
    inventory::Spec {
        fabric,
        editable: BTreeMap::from([("access".into(), (1..=4).map(|i| format!("p{i}")).collect())]),
    }
}

pub fn fixture() -> (api::Spec, api::Store) {
    let spec = api::Spec {
        inventory: inventory(),
        ports: (1..=4)
            .map(|i| {
                (
                    format!("desk-{i}"),
                    api::Slot {
                        switch: "access".into(),
                        port: format!("p{i}"),
                    },
                )
            })
            .collect(),
    };
    let store = api::Store {
        profiles: BTreeMap::from([
            (
                "staff".into(),
                api::Profile {
                    up: true,
                    vlan: 10,
                    description: Some("Staff workgroup".into()),
                },
            ),
            (
                "visitors".into(),
                api::Profile {
                    up: true,
                    vlan: 20,
                    description: Some("Visitor workgroup".into()),
                },
            ),
            (
                "retired".into(),
                api::Profile {
                    up: false,
                    vlan: 30,
                    description: None,
                },
            ),
        ]),
        bindings: (1..=4)
            .map(|i| {
                (
                    format!("desk-{i}"),
                    api::Binding {
                        profile: if i < 3 {
                            "staff".into()
                        } else {
                            "visitors".into()
                        },
                    },
                )
            })
            .collect(),
    };
    (spec, store)
}
pub fn request(method: &str, path: &str, body: &str) -> api::Request {
    api::Request {
        method: method.into(),
        path: path.into(),
        authorization: "Bearer lab-operator".into(),
        content_type: if method == "PATCH" {
            "application/merge-patch+json".into()
        } else {
            "application/json".into()
        },
        body: body.into(),
    }
}
