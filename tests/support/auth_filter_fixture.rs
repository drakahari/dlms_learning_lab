use dlms_learning_lab::auth_filter_task::{Case, Expected as E, Task};
pub fn task(ignore_health: bool) -> Task {
    let line = |app: &str, event: &str, user: &str, ip: &str, port: u16, source: &str| {
        format!("{app} {event} user={user} from {ip} port {port} source={source}")
    };
    let cases = vec![
        Case {
            id: "first-client".into(),
            label: "Ordinary denied and accepted logins".into(),
            lines: vec![
                line(
                    "portal",
                    "rejected",
                    "alex",
                    "192.0.2.9",
                    53142,
                    "interactive",
                ),
                line(
                    "portal",
                    "accepted",
                    "alex",
                    "192.0.2.9",
                    53143,
                    "interactive",
                ),
                line("portal", "rejected", "probe", "192.0.2.10", 99, "health"),
                "ordinary notice".into(),
            ],
            expected: vec![
                E::Matched,
                E::Missed,
                if ignore_health {
                    E::Ignored
                } else {
                    E::Matched
                },
                E::Missed,
            ],
        },
        Case {
            id: "changed-client".into(),
            label: "Changing usernames, addresses and ports".into(),
            lines: vec![
                line(
                    "portal",
                    "rejected",
                    "Ops_1",
                    "203.0.113.88",
                    65535,
                    "interactive",
                ),
                line(
                    "portal",
                    "rejected",
                    "web-team",
                    "198.51.100.6",
                    22,
                    "interactive",
                ),
                line("portal", "accepted", "probe", "192.0.2.10", 9000, "health"),
                line("portal", "rejected", "alex", "203.0.113.1", 43, "health"),
            ],
            expected: vec![
                E::Matched,
                E::Matched,
                E::Missed,
                if ignore_health {
                    E::Ignored
                } else {
                    E::Matched
                },
            ],
        },
        Case {
            id: "lookalikes".into(),
            label: "Real users and unrelated notices".into(),
            lines: vec![
                line(
                    "portal",
                    "rejected",
                    "probe",
                    "192.0.2.10",
                    100,
                    "interactive",
                ),
                line(
                    "audit-portal",
                    "rejected",
                    "alex",
                    "192.0.2.9",
                    99,
                    "interactive",
                ),
                line(
                    "portal",
                    "accepted",
                    "mira",
                    "203.0.113.5",
                    122,
                    "interactive",
                ),
                "portal rejected user=alex without a recorded address".into(),
            ],
            expected: vec![E::Matched, E::Missed, E::Missed, E::Missed],
        },
    ];
    Task {
        id: if ignore_health {
            "health-filter"
        } else {
            "failure-filter"
        }
        .into(),
        log_path: "/var/log/portal.log".into(),
        filter_path: "/etc/fail2ban/filter.d/portal.conf".into(),
        initial_filter: if ignore_health {
            format!(
                "{}ignoreregex = ^portal rejected user=probe \n",
                canonical(false)
            )
        } else {
            "[Definition]\nfailregex = ^portal accepted user=[A-Za-z0-9_-]+ from <HOST> port [0-9]+ source=[a-z]+$\n".into()
        },
        cases,
    }
}
pub fn canonical(ignore_health: bool) -> String {
    format!(
        "[Definition]\nfailregex = ^portal rejected user=[A-Za-z0-9_-]+ from <HOST> port [0-9]+ source=[a-z]+$\n{}",
        if ignore_health {
            "ignoreregex = ^portal rejected user=[A-Za-z0-9_-]+ from [0-9.]+ port [0-9]+ source=health$\n"
        } else {
            ""
        }
    )
}
pub fn alternative(ignore_health: bool) -> String {
    format!(
        "# Equivalent ordinary grouping and repetition\n[Definition]\nfailregex = ^portal (rejected) user=[A-Za-z0-9_-][A-Za-z0-9_-]* from <HOST> port [0-9][0-9]* source=[a-z][a-z]*$\n{}",
        if ignore_health {
            "ignoreregex = ^portal rejected user=[A-Za-z0-9_-]+ from [0-9.]+ port [0-9]+ source=(health)$\n"
        } else {
            ""
        }
    )
}
