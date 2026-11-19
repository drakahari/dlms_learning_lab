//! Reviewed synthetic policy task; not registered authored curriculum.
#[path = "dns_access_fixture.rs"]
mod dataset;
use dlms_learning_lab::{
    dns::{Data, QueryType},
    dns_access::{
        acl::{Acl, Network},
        world::World,
    },
    dns_access_workspace::{
        self as work, Action, Cache, Case, Config, Expected, Name, Required, Task,
        TransferRequirement, session::Definition,
    },
};
fn acl(text: &str) -> Acl {
    Acl::Networks {
        networks: vec![Network::parse(text).unwrap()],
    }
}
pub fn task() -> Task {
    let facts = dataset::facts();
    let initial = dataset::config(true, "any;", "any;", "any;", "", "");
    let solution = dataset::config(
        true,
        "192.0.2.0/24;",
        "192.0.2.0/24;",
        "none;",
        "allow-transfer { 198.51.100.2; };",
        "",
    );
    let positive = |label: &str, client: &str, name: &str, cache: Cache, address: &str| Case {
        label: label.into(),
        client: client.parse().unwrap(),
        name: Name(name.into()),
        record_type: QueryType::A,
        recursive: true,
        cache,
        expected: Expected::Data {
            data: vec![Data::A {
                address: address.parse().unwrap(),
            }],
        },
    };
    let negative = |label: &str, name: &str, cache: Cache| Case {
        label: label.into(),
        client: "198.51.100.2".parse().unwrap(),
        name: Name(name.into()),
        record_type: QueryType::A,
        recursive: true,
        cache,
        expected: Expected::NoExternalAnswer {},
    };
    Task {
        id: "staff-dns".into(),
        facts,
        initial_config: Config(initial),
        starter: vec![],
        required: Required {
            public_queries: Acl::Any,
            cached_answers: acl("192.0.2.0/24"),
            recursive_lookups: acl("192.0.2.0/24"),
            transfers: vec![
                TransferRequirement {
                    zone: Name("public.example.".into()),
                    clients: acl("198.51.100.2"),
                    synchronized: true,
                },
                TransferRequirement {
                    zone: Name("private.example.".into()),
                    clients: Acl::None,
                    synchronized: false,
                },
            ],
        },
        cases: vec![
            positive(
                "Public outside",
                "198.51.100.2",
                "www.public.example.",
                Cache::Cold,
                "203.0.113.10",
            ),
            positive(
                "Staff cold",
                "192.0.2.7",
                "api.external.example.",
                Cache::Cold,
                "198.51.100.7",
            ),
            positive(
                "Staff warm",
                "192.0.2.8",
                "www.external.example.",
                Cache::SuppliedWarm,
                "198.51.100.7",
            ),
            negative("Outside cold", "api.external.example.", Cache::Cold),
            negative("Outside warm", "www.external.example.", Cache::SuppliedWarm),
        ],
        reference: vec![
            Action::Save {
                config: Config(solution),
            },
            Action::Reload {},
            Action::Transfer {
                client: "198.51.100.2".parse().unwrap(),
                zone: Name("public.example.".into()),
            },
        ],
    }
}
pub fn definition() -> Definition {
    let initial = task();
    let json = serde_json::to_string(&initial)
        .unwrap()
        .replace("staff-dns", "demo-dns")
        .replace("public.example", "demo.example")
        .replace("private.example", "hidden.example")
        .replace("external.example", "sample.example")
        .replace("db.public", "db.demo")
        .replace("db.private", "db.hidden");
    Definition {
        initial,
        example: serde_json::from_str(&json).unwrap(),
    }
}
pub fn reached(task: &Task) -> World {
    let mut world = task.initial().unwrap();
    for action in &task.reference {
        work::apply(&mut world, action).unwrap();
    }
    world
}
pub fn alternate_config() -> String {
    dataset::config(
        true,
        "192.0.2.0/25; 192.0.2.128/25;",
        "any;",
        "198.51.100.2;",
        "",
        "allow-transfer { none; };",
    )
}
