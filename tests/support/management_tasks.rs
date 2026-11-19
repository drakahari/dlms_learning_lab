//! Synthetic grading fixtures only; these are not authored or registered labs.
#[path = "management_fixture.rs"]
pub mod base;
use dlms_learning_lab::{management_policy as p, management_workspace as w};

pub fn task(source_restriction: bool) -> w::Task {
    let mut network = base::network();
    let mut supplied = base::settings();
    let row = supplied.lines[0].clone();
    supplied.lines = (0..16)
        .map(|number| p::Line {
            number,
            ..row.clone()
        })
        .collect();
    let mut cases = vec![
        w::Case {
            label: "First approved administrator".into(),
            client: "admin".into(),
            protocol: p::Protocol::Ssh2,
            expected: w::Expected::ReadyForAuthentication,
        },
        w::Case {
            label: "Second approved administrator".into(),
            client: if source_restriction {
                "operator"
            } else {
                "guest"
            }
            .into(),
            protocol: p::Protocol::Ssh2,
            expected: w::Expected::ReadyForAuthentication,
        },
        w::Case {
            label: "Telnet must remain unavailable".into(),
            client: "admin".into(),
            protocol: p::Protocol::Telnet,
            expected: w::Expected::ProtocolDenied,
        },
        w::Case {
            label: "Legacy SSH must remain unavailable".into(),
            client: "admin".into(),
            protocol: p::Protocol::Ssh1,
            expected: w::Expected::LegacyVersionDenied,
        },
    ];
    let reference: String = if source_restriction {
        for (id, device, address) in [
            ("operator", "Operator", "192.0.2.11"),
            ("nearby", "Nearby", "192.0.2.12"),
        ] {
            network.topology.devices.insert(
                device.into(),
                base::device(
                    false,
                    vec![("lan", base::port("office", address, 24))],
                    vec![base::route("0.0.0.0/0", "lan", "192.0.2.1")],
                ),
            );
            network
                .clients
                .push(dlms_learning_lab::management_path::Client {
                    id: id.into(),
                    binding: base::binding(device, "lan"),
                });
        }
        cases.extend([
            w::Case {
                label: "Unapproved guest".into(),
                client: "guest".into(),
                protocol: p::Protocol::Ssh2,
                expected: w::Expected::SourceDenied,
            },
            w::Case {
                label: "Unapproved nearby workstation".into(),
                client: "nearby".into(),
                protocol: p::Protocol::Ssh2,
                expected: w::Expected::SourceDenied,
            },
        ]);
        "ip ssh version 2\nline vty 0 15\n transport input ssh\n access-class 10 in\nexit\nip access-list standard 10\n permit host 192.0.2.10\n permit host 192.0.2.11\nexit\n".into()
    } else {
        "ip ssh version 2\nline vty 0 15\n transport input ssh\nexit\n".into()
    };
    let starter = if source_restriction {
        reference.replace("permit host 192.0.2.10", "permit any")
    } else {
        reference
            .replace("ip ssh version 2", "no ip ssh version")
            .replace("transport input ssh", "transport input ssh telnet")
    };
    w::Task{version:1,id:if source_restriction{"synthetic-management-sources"}else{"synthetic-management-protocols"}.into(),
        instructions:if source_restriction{"Retain both approved administrators on SSH version 2 across every supplied line, reject the other supplied clients by source policy, and keep Telnet/legacy SSH unavailable."}else{"Retain both supplied administrators on SSH version 2 across every line while keeping Telnet and legacy SSH unavailable."}.into(),
        network,supplied,starter,reference,cases}
}

pub fn definition(source_restriction: bool) -> w::Definition {
    let initial = task(source_restriction);
    let raw = serde_json::to_string(&initial)
        .unwrap()
        .replace("192.0.2.", "172.18.40.");
    let mut example: w::Task = serde_json::from_str(&raw).unwrap();
    example.id.push_str("-example");
    example.supplied.host_identity = Some("supplied-example-identity".into());
    example.supplied.lines.truncate(3);
    example.starter = example.starter.replace("line vty 0 15", "line vty 0 2");
    example.reference = example.reference.replace("line vty 0 15", "line vty 0 2");
    w::Definition {
        version: 1,
        initial,
        example,
    }
}
