use dlms_learning_lab::{
    ids_rule::{Event, Protocol},
    ids_rule_task::{Case, Task},
};

pub fn record(source: &str, destination: &str, port: u16, payload: &str) -> Event {
    Event {
        protocol: Protocol::Tcp,
        source: source.parse().unwrap(),
        source_port: 41000,
        destination: destination.parse().unwrap(),
        destination_port: port,
        payload: payload.into(),
    }
}
pub fn task() -> Task {
    Task {
        id: "local-api-detection".into(),
        rule_path: "/etc/snort/rules/local.rules".into(),
        initial_rule: "alert tcp any any -> 198.51.100.8 80 (content:\"POST\";sid:1000100;)".into(),
        cases: vec![
            Case {
                id: "primary".into(),
                label: "API destination and payload".into(),
                records: vec![
                    record(
                        "192.0.2.4",
                        "198.51.100.8",
                        8080,
                        "POST /admin HTTP/1.1\r\n",
                    ),
                    record(
                        "192.0.2.5",
                        "198.51.100.8",
                        8080,
                        "GET /health HTTP/1.1\r\n",
                    ),
                    record(
                        "192.0.2.4",
                        "198.51.100.9",
                        8080,
                        "POST /admin HTTP/1.1\r\n",
                    ),
                ],
                expected_alerts: vec![true, false, false],
            },
            Case {
                id: "contrasts".into(),
                label: "Different clients and traffic".into(),
                records: vec![
                    record(
                        "192.0.2.29",
                        "198.51.100.8",
                        8080,
                        "POST /admin HTTP/1.0\r\n",
                    ),
                    record("192.0.2.4", "198.51.100.8", 80, "POST /admin HTTP/1.1\r\n"),
                    record(
                        "192.0.2.4",
                        "198.51.100.8",
                        8080,
                        "POST /public HTTP/1.1\r\n",
                    ),
                ],
                expected_alerts: vec![true, false, false],
            },
        ],
    }
}
pub const SOLUTION: &str =
    "alert tcp any any -> 198.51.100.8 8080 (content:\"POST /admin\";sid:1000100;)";
pub const ALTERNATIVE: &str = "alert tcp any any -> 198.51.100.8/32 8080 (sid:1000101;msg:\"API action\";content:\"POST\";content:\"/admin\";)";
