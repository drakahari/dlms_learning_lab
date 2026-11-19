//! Synthetic shared DNS-policy data; never host networking or learner execution.
use dlms_learning_lab::{
    dns::{Data, QueryType, Record, Server, Zone},
    dns_access::world::{Facts, Key, ZoneSource},
};
pub fn zone(origin: &str, address: &str) -> Zone {
    Zone {
        origin: origin.into(),
        serial: 10,
        records: vec![
            Record {
                owner: origin.into(),
                ttl: 300,
                data: Data::Ns {
                    target: format!("ns.{origin}"),
                },
            },
            Record {
                owner: format!("ns.{origin}"),
                ttl: 300,
                data: Data::A {
                    address: "192.0.2.53".parse().unwrap(),
                },
            },
            Record {
                owner: format!("www.{origin}"),
                ttl: 300,
                data: Data::A {
                    address: address.parse().unwrap(),
                },
            },
            Record {
                owner: format!("api.{origin}"),
                ttl: 300,
                data: Data::A {
                    address: address.parse().unwrap(),
                },
            },
        ],
    }
}
pub fn facts() -> Facts {
    let mut server = Server::new(zone("public.example.", "203.0.113.10")).unwrap();
    server.secondary.serial = 9;
    server.secondary.records.last_mut().unwrap().data = Data::A {
        address: "203.0.113.9".parse().unwrap(),
    };
    Facts {
        version: 1,
        zones: vec![
            ZoneSource {
                file: "db.public".into(),
                secondary_client: "198.51.100.2".parse().unwrap(),
                server,
            },
            ZoneSource {
                file: "db.private".into(),
                secondary_client: "198.51.100.3".parse().unwrap(),
                server: Server::new(zone("private.example.", "203.0.113.20")).unwrap(),
            },
        ],
        upstream: vec![zone("external.example.", "198.51.100.7")],
        warm_cache: vec![Key::new("www.external.example", QueryType::A).unwrap()],
    }
}
pub fn config(
    recursion: bool,
    cache: &str,
    recursive: &str,
    transfer: &str,
    public_override: &str,
    private_override: &str,
) -> String {
    format!(
        r#"options {{
 allow-query {{ any; }};
 recursion {};
 allow-recursion {{ {recursive} }};
 allow-query-cache {{ {cache} }};
 allow-transfer {{ {transfer} }};
}};
zone "public.example" {{ type primary; file "db.public"; {public_override} }};
zone "private.example" {{ type primary; file "db.private"; {private_override} }};
"#,
        if recursion { "yes" } else { "no" }
    )
}
