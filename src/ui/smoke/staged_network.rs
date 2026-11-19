//! Fixed Wayland check for the actual rollout and intent native controls.
use super::*;
use dlms_learning_lab::automation_workshop::{session::Session, source, workspace::Work};
use std::{
    sync::OnceLock,
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Appearance(f32, bool),
    Button(&'static str),
    Choice(&'static str),
    PlanInput(String),
    NfsConfigInput(String),
    FieldInput(&'static str, String),
    TerminalEnter,
    Observed(&'static str),
    ObservedAbsent(&'static str),
    Check { count: usize, practice: bool },
    Unchecked,
    Capture(String, &'static str),
    SaveReadback,
}

#[derive(Clone, Debug, Default)]
struct Flow {
    at: usize,
    phase: u8,
    since: Option<Instant>,
    next: f64,
    previous_area: Option<egui::Rect>,
    requested_pixels: Option<egui::Vec2>,
    resize_reported: bool,
}

fn select_steps(steps: &mut Vec<Op>, area: &'static str, choice: &'static str) {
    steps.extend([Op::Button(area), Op::Choice(choice)]);
}

fn routine(ops: &mut Vec<Op>, index: usize) {
    if index == 0 {
        select_steps(ops, "rollout-method", "rollout-method-2");
        select_steps(ops, "rollout-profile", "rollout-profile-0");
        ops.extend([
            Op::Button("rollout-preview"),
            Op::Button("rollout-apply"),
            Op::Button("rollout-verify-0"),
        ]);
        select_steps(ops, "rollout-slot", "rollout-slot-1");
        ops.extend([Op::Button("rollout-preview"), Op::Button("rollout-apply")]);
        select_steps(ops, "rollout-method", "rollout-method-3");
        ops.extend([Op::Button("rollout-preview"), Op::Button("rollout-apply")]);
    } else {
        select_steps(ops, "intent-from", "intent-from-1");
        select_steps(ops, "intent-to", "intent-to-0");
        ops.extend([
            Op::Button("intent-submit"),
            Op::Button("intent-publish"),
            Op::Button("intent-uplink"),
        ]);
    }
}

fn plan() -> &'static [Op] {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let pack: pack::Pack = serde_json::from_str(include_str!(
            "../../../scenarios/staged-network-lessons-v1.json"
        ))
        .unwrap();
        let mut ops = vec![Op::Appearance(1., false)];
        for (index, lab) in pack.scenarios.iter().enumerate() {
            let snap =
                |part: &str, target| Op::Capture(format!("staged-{}-{part}", index + 1), target);
            let entry = ["rollout-method", "intent-from"][index];
            ops.extend([
                Op::Open(lab.id.clone()),
                snap("introduction", "introduction-content"),
            ]);
            for practice in [true, false] {
                ops.extend([
                    Op::Button(if practice { "example" } else { "assignment" }),
                    if practice {
                        snap("practice", entry)
                    } else {
                        snap("assignment", "assignment-content")
                    },
                    Op::Button("check"),
                    Op::Check {
                        count: [4, 3][index],
                        practice,
                    },
                ]);
                routine(&mut ops, index);
                ops.extend([
                    Op::Button("check"),
                    Op::Check {
                        count: [9, 6][index],
                        practice,
                    },
                    snap(
                        if practice {
                            "example-complete"
                        } else {
                            "complete"
                        },
                        "result",
                    ),
                ]);
            }
            if index == 0 {
                select_steps(&mut ops, "rollout-method", "rollout-method-0");
            } else {
                ops.push(Op::Button("intent-uplink"));
            }
            ops.extend([
                Op::Unchecked,
                Op::Button("check"),
                Op::Check {
                    count: [8, 3][index],
                    practice: false,
                },
                snap("after-change", "result"),
            ]);
        }
        ops.extend([
            Op::Appearance(1.5, true),
            Op::Capture("staged-light150-table".into(), "intent-uplink"),
            Op::Appearance(2.0, false),
            Op::Capture("staged-navy200-feedback".into(), "check"),
            Op::SaveReadback,
        ]);
        ops
    })
}

fn review_plan() -> &'static [Op] {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let pack: pack::Pack = serde_json::from_str(include_str!(
            "../../../scenarios/network-evidence-review-lessons-v1.json"
        ))
        .unwrap();
        let fixtures: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/network_evidence_review_tasks_v1.json"
        ))
        .unwrap();
        let mut ops = vec![Op::Appearance(1., false)];
        for (index, lesson) in pack.scenarios.iter().enumerate() {
            let shot = |part: &str, anchor| {
                Op::Capture(format!("review-network-{}-{part}", index + 1), anchor)
            };
            ops.extend([
                Op::Open(lesson.id.clone()),
                shot("introduction", "introduction-content"),
            ]);
            for practice in [true, false] {
                ops.push(Op::Button(if practice { "example" } else { "assignment" }));
                ops.push(if practice {
                    shot("practice", ["incident-probe", "plan-draft"][index])
                } else {
                    shot("assignment", "assignment-content")
                });
                ops.extend([
                    Op::Button("check"),
                    Op::Check {
                        count: [2, 6][index],
                        practice,
                    },
                ]);
                if index == 0 {
                    ops.push(Op::Button("incident-probe"));
                    select_steps(
                        &mut ops,
                        "diagnosis",
                        if practice {
                            "diagnosis-1"
                        } else {
                            "diagnosis-2"
                        },
                    );
                    ops.push(Op::Button("next-reading"));
                    select_steps(
                        &mut ops,
                        "forecast",
                        if practice { "forecast-1" } else { "forecast-2" },
                    );
                    select_steps(&mut ops, "response", "response-2");
                    ops.push(Op::Button("apply-response"));
                } else {
                    let text = fixtures[if practice { "example" } else { "assignment" }]["config"]
                        ["valid_document"]
                        .as_str()
                        .unwrap();
                    ops.extend([
                        Op::Button("preview-plan"),
                        shot(
                            if practice {
                                "unsafe-example-preview"
                            } else {
                                "unsafe-preview"
                            },
                            "preview-plan",
                        ),
                        Op::PlanInput(text.into()),
                        Op::Button("preview-plan"),
                        Op::Button("apply-plan"),
                        Op::Button("preview-plan"),
                    ]);
                }
                ops.extend([
                    Op::Button("check"),
                    Op::Check {
                        count: [6, 10][index],
                        practice,
                    },
                    shot(
                        if practice {
                            "example-complete"
                        } else {
                            "complete"
                        },
                        "result",
                    ),
                ]);
            }
            if index == 0 {
                select_steps(&mut ops, "diagnosis", "diagnosis-1");
            } else {
                ops.extend([
                    Op::PlanInput("{ pending malformed draft".into()),
                    Op::Button("preview-plan"),
                    shot("invalid-preview", "plan-error"),
                ]);
            }
            ops.extend([
                Op::Unchecked,
                Op::Button("check"),
                Op::Check {
                    count: [5, 9][index],
                    practice: false,
                },
                shot("after-change", "result"),
            ]);
        }
        ops.extend([
            Op::Appearance(1.5, true),
            Op::Capture("review-network-light150-editor".into(), "plan-draft"),
            Op::Appearance(2.0, false),
            Op::Capture("review-network-navy200-feedback".into(), "check"),
            Op::SaveReadback,
        ]);
        ops
    })
}

fn site_plan() -> &'static [Op] {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let pack: pack::Pack = serde_json::from_str(include_str!(
            "../../../scenarios/site-connectivity-lessons-v1.json"
        ))
        .unwrap();
        let mut ops = vec![Op::Appearance(1., false)];
        for (index, lesson) in pack.scenarios.iter().enumerate() {
            let capture =
                |phase, anchor| Op::Capture(format!("site-path-{}-{phase}", index + 1), anchor);
            ops.extend([
                Op::Open(lesson.id.clone()),
                capture("introduction", "introduction-content"),
                Op::Button("example"),
                capture("practice", "site-trial"),
                Op::Button("site-trace"),
            ]);
            select_steps(
                &mut ops,
                "site-trial",
                if index == 0 {
                    "site-trial-1"
                } else {
                    "site-trial-3"
                },
            );
            ops.extend([
                Op::Button("site-trace"),
                capture("outage-sample", "site-trace"),
                Op::Button("check"),
                Op::Check {
                    count: [4, 7][index],
                    practice: true,
                },
            ]);
            if index == 0 {
                ops.extend([Op::Button("site-link-5"), Op::Button("site-link-6")]);
            } else {
                ops.push(Op::Button("site-link-7"));
            }
            ops.extend([
                Op::Button("site-trace"),
                Op::Button("check"),
                Op::Check {
                    count: [6, 8][index],
                    practice: true,
                },
                capture("example-complete", "result"),
                Op::Button("assignment"),
                capture("assignment", "assignment-content"),
                Op::Button("check"),
                Op::Check {
                    count: [3, 4][index],
                    practice: false,
                },
            ]);
            if index == 0 {
                ops.extend([Op::Button("site-link-8"), Op::Button("site-link-6")]);
            } else {
                ops.extend([
                    Op::Button("site-link-8"),
                    Op::Button("site-link-4"),
                    Op::Button("site-link-7"),
                ]);
            }
            ops.extend([
                Op::Button("check"),
                Op::Check {
                    count: [6, 8][index],
                    practice: false,
                },
                capture("complete", "result"),
            ]);
            ops.push(Op::Button(if index == 0 {
                "site-link-8"
            } else {
                "site-link-7"
            }));
            ops.extend([
                Op::Unchecked,
                Op::Button("check"),
                Op::Check {
                    count: [4, 7][index],
                    practice: false,
                },
                capture("after-change", "result"),
            ]);
        }
        ops.extend([
            Op::Appearance(1.5, true),
            Op::Capture("site-path-light150-circuit".into(), "site-link-6"),
            Op::Appearance(2.0, false),
            Op::Capture("site-path-navy200-feedback".into(), "check"),
            Op::SaveReadback,
        ]);
        ops
    })
}

fn host_security_plan() -> &'static [Op] {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let pack: pack::Pack = serde_json::from_str(include_str!(
            "../../../scenarios/host-and-dns-security-lessons-v1.json"
        ))
        .unwrap();
        let mut ops = vec![Op::Appearance(1., false)];
        for (index, lesson) in pack.scenarios.iter().enumerate() {
            let shot =
                |phase, anchor| Op::Capture(format!("host-security-{}-{phase}", index + 1), anchor);
            ops.extend([
                Op::Open(lesson.id.clone()),
                shot("introduction", "introduction-content"),
            ]);
            for practice in [true, false] {
                let task = source::source(lesson).unwrap();
                let supplied = if practice {
                    &task.example
                } else {
                    &task.assignment
                };
                ops.extend([
                    Op::Button(if practice { "example" } else { "assignment" }),
                    if practice {
                        shot("practice", ["retirement-command", "dns-rule"][index])
                    } else {
                        shot("assignment", "assignment-content")
                    },
                    Op::Button("check"),
                    Op::Check {
                        count: [1, 3][index],
                        practice,
                    },
                ]);
                use dlms_learning_lab::automation_workshop::workspace::Task;
                if let Task::SocketRetirement { supplied } = supplied {
                    let command = |verb: &str, name: &str| {
                        [
                            Op::FieldInput(
                                "retirement-command",
                                format!("systemctl {verb} {name}"),
                            ),
                            Op::TerminalEnter,
                        ]
                    };
                    ops.extend(command("status", &supplied.activating_socket));
                    ops.push(Op::Observed("active (listening)"));
                    if practice {
                        ops.extend(command("stop", &supplied.obsolete_service));
                        ops.push(Op::Button("retirement-obsolete-present"));
                        ops.push(Op::Observed("socket started the service"));
                        ops.push(shot("socket-reactivated", "retirement-obsolete-present"));
                    }
                    for name in [&supplied.activating_socket, &supplied.obsolete_service] {
                        ops.extend(command("disable --now", name));
                    }
                    ops.extend([
                        Op::Button("retirement-obsolete-boot"),
                        Op::Observed("received no response"),
                        Op::Button("retirement-required-boot"),
                        Op::Observed("answered the supplied request"),
                    ]);
                } else if let Task::KeyedTransfer { supplied } = supplied {
                    ops.extend([
                        Op::Button("dns-unsigned"),
                        Op::Observed("policy exposed"),
                        Op::Button("dns-outdated"),
                        Op::Observed("does not match a current key"),
                        Op::FieldInput(
                            "dns-rule",
                            format!("allow-transfer {{ key {}; }};", supplied.approved[0].name),
                        ),
                    ]);
                    select_steps(&mut ops, "dns-key", "dns-key-1");
                    ops.extend([
                        Op::Button("dns-apply"),
                        Op::Button("dns-refresh"),
                        Op::Observed("received a fresh copy"),
                        Op::Button("dns-unsigned"),
                        Op::Observed("refused the request"),
                        Op::Button("dns-protected"),
                        Op::Observed("refused the request"),
                        Op::Button("dns-secondary"),
                        Op::Observed("Secondary lookup"),
                        Op::Button("dns-public"),
                        Op::Observed("Public lookup"),
                    ]);
                } else {
                    panic!("Expected actual Linux security work order");
                }
                ops.extend([
                    Op::Button("check"),
                    Op::Check { count: 6, practice },
                    shot(
                        if practice {
                            "example-complete"
                        } else {
                            "complete"
                        },
                        "result",
                    ),
                ]);
            }
            if index == 0 {
                ops.extend([
                    Op::FieldInput(
                        "retirement-command",
                        "systemctl start legacy-import.socket".into(),
                    ),
                    Op::TerminalEnter,
                ]);
            } else {
                ops.extend([
                    Op::FieldInput("dns-rule", "allow-transfer { unfinished".into()),
                    Op::Button("dns-apply"),
                    shot("rejected-policy", "dns-apply"),
                ]);
            }
            ops.extend([
                Op::Unchecked,
                Op::Button("check"),
                Op::Check {
                    count: [4, 5][index],
                    practice: false,
                },
                shot("after-change", "result"),
            ]);
        }
        ops.extend([
            Op::Appearance(1.5, true),
            Op::Capture("host-security-light150-rule".into(), "dns-rule"),
            Op::Appearance(2., false),
            Op::Capture("host-security-navy200-feedback".into(), "check"),
            Op::SaveReadback,
        ]);
        ops
    })
}

fn vlan_gateway_plan() -> &'static [Op] {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let mut ops = vec![Op::Appearance(1., false)];
        for (index, id) in [
            "ccna-default-access-membership",
            "ccna-switched-gateway-repair",
        ]
        .into_iter()
        .enumerate()
        {
            let capture =
                |stage, anchor| Op::Capture(format!("vlan-gateway-{}-{stage}", index + 1), anchor);
            ops.extend([
                Op::Open(id.into()),
                capture("introduction", "introduction-content"),
            ]);
            for practice in [true, false] {
                ops.extend([
                    Op::Button(if practice { "example" } else { "assignment" }),
                    if practice {
                        capture("practice", ["default-access", "svi-address-0"][index])
                    } else {
                        capture("assignment", "assignment-content")
                    },
                    Op::Button("check"),
                    Op::Check {
                        count: if index == 0 {
                            1
                        } else if practice {
                            3
                        } else {
                            2
                        },
                        practice,
                    },
                ]);
                if index == 0 {
                    if practice {
                        select_steps(&mut ops, "default-access", "default-access-0");
                    }
                    select_steps(&mut ops, "default-pair", "default-pair-1");
                    ops.extend([
                        Op::Button("default-ethernet"),
                        Op::Observed("each received"),
                    ]);
                    select_steps(&mut ops, "default-access", "default-access-3");
                    ops.extend([Op::Button("check"), Op::Check { count: 3, practice }]);
                    select_steps(&mut ops, "default-access", "default-access-2");
                    ops.extend([
                        Op::Button("default-ethernet"),
                        Op::Observed("Neither Ethernet broadcast"),
                    ]);
                    select_steps(&mut ops, "default-pair", "default-pair-0");
                    ops.extend([
                        Op::Button("default-ethernet"),
                        Op::Observed("each received"),
                    ]);
                    select_steps(&mut ops, "default-pair", "default-pair-3");
                    ops.extend([
                        Op::Button("default-ethernet"),
                        Op::Observed("each received"),
                    ]);
                } else {
                    ops.push(Op::Button("svi-routing"));
                    if practice {
                        ops.push(Op::FieldInput("svi-address-1", "198.51.100.1/24".into()));
                        select_steps(&mut ops, "svi-trial", "svi-trial-1");
                        ops.extend([
                            Op::Button("svi-ipv4"),
                            Op::Observed("no up neighbor"),
                            capture("blocked-path", "svi-ipv4"),
                        ]);
                    } else {
                        ops.extend([
                            Op::Button("svi-enable-0"),
                            Op::FieldInput("svi-address-0", "10.45.1.254/24".into()),
                            Op::FieldInput("svi-address-1", "10.70.1.254/24".into()),
                        ]);
                    }
                    ops.push(Op::Button("svi-allowed-1"));
                    for (i, key) in [
                        "station-gateway-0",
                        "station-gateway-1",
                        "station-gateway-2",
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let value = if practice {
                            ["192.0.2.1", "192.0.2.1", "198.51.100.1"][i]
                        } else {
                            ["10.45.1.254", "10.45.1.254", "10.70.1.254"][i]
                        };
                        ops.push(Op::FieldInput(key, value.into()));
                    }
                    for choice in ["svi-trial-1", "svi-trial-2"] {
                        select_steps(&mut ops, "svi-trial", choice);
                        ops.extend([
                            Op::Button("svi-ipv4"),
                            Op::Observed("IPv4 delivery and reply worked"),
                            Op::Button("svi-ethernet"),
                            Op::Observed("Neither Ethernet broadcast"),
                        ]);
                    }
                    select_steps(&mut ops, "svi-trial", "svi-trial-0");
                    ops.extend([Op::Button("svi-ethernet"), Op::Observed("each received")]);
                }
                ops.extend([
                    Op::Button("check"),
                    Op::Check {
                        count: [4, 6][index],
                        practice,
                    },
                    capture(
                        if practice {
                            "example-complete"
                        } else {
                            "complete"
                        },
                        "result",
                    ),
                ]);
            }
            if index == 0 {
                select_steps(&mut ops, "default-access", "default-access-0");
            } else {
                ops.push(Op::FieldInput("station-gateway-2", "10.70.1.".into()));
            }
            ops.extend([
                Op::Unchecked,
                Op::Button("check"),
                Op::Check {
                    count: [1, 4][index],
                    practice: false,
                },
                capture("after-change", "result"),
            ]);
        }
        ops.extend([
            Op::Appearance(1.5, true),
            Op::Capture("vlan-gateway-light150-address".into(), "svi-address-1"),
            Op::FieldInput("station-gateway-2", "10.70.1.254".into()),
            Op::Button("check"),
            Op::Check {
                count: 6,
                practice: false,
            },
            Op::Appearance(2., false),
            Op::FieldInput("station-gateway-2", "10.70.1.".into()),
            Op::Unchecked,
            Op::Button("check"),
            Op::Check {
                count: 4,
                practice: false,
            },
            Op::Capture("vlan-gateway-navy200-feedback".into(), "check"),
            Op::SaveReadback,
        ]);
        ops
    })
}

fn nfs_export_plan() -> &'static [Op] {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| {
        use dlms_learning_lab::automation_workshop::workspace::Task;
        let authored: pack::Pack = serde_json::from_str(include_str!(
            "../../../scenarios/nfs-export-operations-v1.json"
        ))
        .unwrap();
        let lesson = &authored.scenarios[0];
        let definition = source::source(lesson).unwrap();
        let mut ops = vec![
            Op::Appearance(1., false),
            Op::Open(lesson.id.clone()),
            Op::Capture("nfs-export-introduction".into(), "introduction-content"),
        ];
        for practice in [true, false] {
            let task = if practice {
                &definition.example
            } else {
                &definition.assignment
            };
            let Task::NfsExportRetirement { supplied } = task else {
                panic!("Use the authored NFS re-export task")
            };
            let state = &supplied.definition.initial;
            let address = state.inventory().servers[0].address;
            let preserved = state.runtime().servers[0]
                .draft
                .lines()
                .filter(|line| {
                    line.split_ascii_whitespace().next() != Some(supplied.withdrawn_root.as_str())
                })
                .collect::<Vec<_>>()
                .join("\n")
                + "\n";
            let where_at = if practice { "practice" } else { "assignment" };
            let anchor = if practice {
                "nfs-exports"
            } else {
                "assignment-content"
            };
            let retired = if practice {
                "/srv/demo-drop"
            } else {
                "/srv/old-bundle"
            };
            ops.extend([
                Op::Button(if practice { "example" } else { "assignment" }),
                Op::Capture(format!("nfs-export-{where_at}"), anchor),
                Op::Button("check"),
                Op::Check { count: 6, practice },
                Op::FieldInput("nfs-command", "exportfs -v".into()),
                Op::TerminalEnter,
                Op::Observed(retired),
                Op::NfsConfigInput(preserved),
                Op::Button("nfs-save"),
                Op::FieldInput("nfs-command", format!("showmount -e {address}")),
                Op::TerminalEnter,
                Op::Observed("Export list for"),
                Op::Observed(retired),
                Op::FieldInput("nfs-command", "exportfs -r".into()),
                Op::TerminalEnter,
                Op::Observed("A current mount depends"),
                Op::Capture(format!("nfs-export-{where_at}-conflict"), "nfs-command"),
                Op::Button("nfs-unmount"),
                Op::Observed("Removed the current mount"),
                Op::FieldInput(
                    "nfs-command",
                    if practice {
                        "exportfs -ra"
                    } else {
                        "exportfs -ar"
                    }
                    .into(),
                ),
                Op::TerminalEnter,
                Op::FieldInput("nfs-command", format!("showmount -e {address}")),
                Op::TerminalEnter,
                Op::Observed("Export list for"),
                Op::ObservedAbsent(retired),
                Op::Observed("/srv/archive"),
                Op::Button("nfs-mount-0"),
                Op::Observed("outside the applied export"),
                Op::Button("nfs-mount-1"),
                Op::Observed("now has a current mount"),
                Op::Button("nfs-read"),
                Op::Observed("handover record"),
                Op::Button("check"),
                Op::Check {
                    count: 12,
                    practice,
                },
                Op::Capture(format!("nfs-export-{where_at}-complete"), "result"),
            ]);
        }
        ops.extend([
            Op::Appearance(1.5, true),
            Op::NfsConfigInput("this is an unfinished export file\n".into()),
            Op::Unchecked,
            Op::Button("nfs-save"),
            Op::FieldInput("nfs-command", "exportfs -ra".into()),
            Op::TerminalEnter,
            Op::Capture("nfs-export-light150-rejected".into(), "nfs-command"),
            Op::Button("check"),
            Op::Check {
                count: 10,
                practice: false,
            },
            Op::Appearance(2., false),
            Op::FieldInput("nfs-command", "showmount -e 198.51.100.10".into()),
            Op::TerminalEnter,
            Op::ObservedAbsent("/srv/old-bundle"),
            Op::Observed("/srv/archive"),
            Op::Check {
                count: 10,
                practice: false,
            },
            Op::Capture("nfs-export-navy200-partial".into(), "check"),
            Op::SaveReadback,
        ]);
        ops
    })
}

fn route_reading_plan() -> &'static [Op] {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let mut ops = vec![
            Op::Appearance(1., false),
            Op::Open("ccna-route-reading-backup".into()),
            Op::Capture("route-reading-introduction".into(), "introduction-content"),
            Op::Button("example"),
            Op::Capture("route-reading-practice".into(), "route-distance"),
            Op::Button("check"),
            Op::Check {
                count: 3,
                practice: true,
            },
            Op::FieldInput("route-distance", "145".into()),
            Op::Button("check"),
            Op::Check {
                count: 4,
                practice: true,
            },
            Op::Button("route-branch-costs"),
            Op::Capture("route-reading-compare-costs".into(), "route-branch-costs"),
            Op::FieldInput("route-cost", "3".into()),
            Op::FieldInput("route-mask", "255.255.255.224".into()),
            Op::Button("check"),
            Op::Check {
                count: 5,
                practice: true,
            },
            Op::FieldInput("route-mask", "255.255.255.192".into()),
            Op::Button("check"),
            Op::Check {
                count: 6,
                practice: true,
            },
        ];
        select_steps(&mut ops, "route-lookup", "route-lookup-2");
        ops.push(Op::Capture(
            "route-reading-outside".into(),
            "route-active-decision",
        ));
        select_steps(&mut ops, "route-lookup", "route-lookup-1");
        ops.extend([
            Op::Button("route-withdrawal"),
            Op::Capture("route-reading-withdrawal".into(), "route-withdrawal"),
            Op::Button("assignment"),
            Op::Capture("route-reading-assignment".into(), "assignment-content"),
            Op::Button("check"),
            Op::Check {
                count: 3,
                practice: false,
            },
            Op::FieldInput("route-cost", "6".into()),
            Op::FieldInput("route-distance", "162".into()),
            Op::FieldInput("route-mask", "255.255.255.192".into()),
            Op::Button("check"),
            Op::Check {
                count: 6,
                practice: false,
            },
            Op::Capture("route-reading-assignment-complete".into(), "result"),
            Op::Appearance(1.5, true),
            Op::FieldInput("route-mask", "255.255.0.255".into()),
            Op::Unchecked,
            Op::Capture("route-reading-light150-mask".into(), "route-mask"),
            Op::Button("check"),
            Op::Check {
                count: 5,
                practice: false,
            },
            Op::Appearance(2., false),
            Op::FieldInput("route-mask", "255.255.255.192".into()),
            Op::FieldInput("route-distance", "255".into()),
            Op::Button("check"),
            Op::Check {
                count: 5,
                practice: false,
            },
            Op::Capture("route-reading-navy200-fallback".into(), "check"),
            Op::SaveReadback,
        ]);
        ops
    })
}

fn ospf_broadcast_plan() -> &'static [Op] {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| {
        vec![
            Op::Appearance(1., false),
            Op::Open("ccna-ospf-broadcast-handover".into()),
            Op::Capture("ospf-broadcast-introduction".into(), "introduction-content"),
            Op::Button("example"),
            Op::Capture("ospf-broadcast-practice".into(), "broadcast-priority-0"),
            Op::Button("check"),
            Op::Check {
                count: 1,
                practice: true,
            },
            Op::FieldInput("broadcast-priority-0", "110".into()),
            Op::FieldInput("broadcast-priority-1", "70".into()),
            Op::FieldInput("broadcast-priority-2", "0".into()),
            Op::Capture("ospf-broadcast-fresh".into(), "broadcast-roles"),
            Op::Button("broadcast-view-1"),
            Op::Capture("ospf-broadcast-arrival".into(), "broadcast-roles"),
            Op::Button("broadcast-view-2"),
            Op::Capture("ospf-broadcast-outage".into(), "broadcast-roles"),
            Op::Button("check"),
            Op::Check {
                count: 5,
                practice: true,
            },
            Op::Button("assignment"),
            Op::Capture("ospf-broadcast-assignment".into(), "assignment-content"),
            Op::Button("check"),
            Op::Check {
                count: 0,
                practice: false,
            },
            Op::FieldInput("broadcast-priority-0", "100".into()),
            Op::FieldInput("broadcast-priority-1", "100".into()),
            Op::FieldInput("broadcast-priority-2", "0".into()),
            Op::Button("broadcast-view-1"),
            Op::Button("broadcast-view-2"),
            Op::Button("check"),
            Op::Check {
                count: 5,
                practice: false,
            },
            Op::Capture("ospf-broadcast-complete".into(), "result"),
            Op::Appearance(1.5, true),
            Op::FieldInput("broadcast-priority-0", "25x".into()),
            Op::Unchecked,
            Op::Capture(
                "ospf-broadcast-light150-input".into(),
                "broadcast-priority-0",
            ),
            Op::Button("check"),
            Op::Check {
                count: 0,
                practice: false,
            },
            Op::FieldInput("broadcast-priority-0", "100".into()),
            Op::Appearance(2., false),
            Op::FieldInput("broadcast-priority-2", "1".into()),
            Op::Button("check"),
            Op::Check {
                count: 4,
                practice: false,
            },
            Op::Capture("ospf-broadcast-navy200-partial".into(), "check"),
            Op::SaveReadback,
        ]
    })
}

fn session(a: &LabApp) -> &Session {
    let lab = &a.scenarios[a.selected.unwrap()];
    let engine::State::AutomationWorkshop { simulation } = &a.progress.attempts[&lab.id].state
    else {
        panic!()
    };
    simulation
}

fn control(a: &LabApp, name: &str) -> Option<egui::Rect> {
    a.metrics
        .automation_workshop
        .controls
        .iter()
        .find(|row| row.0 == name)
        .map(|row| row.1)
}

fn editor_scroll_point(a: &LabApp, ctx: &egui::Context) -> Option<egui::Pos2> {
    let editor = control(a, "plan-draft").or_else(|| control(a, "nfs-exports"))?;
    let view = a.metrics.main_viewport?.intersect(ctx.content_rect());
    let inside_parent = egui::pos2(view.left() + 1., view.center().y);
    if !editor.contains(inside_parent) {
        return None;
    }
    let region = a.metrics.main_scroll_region?.intersect(ctx.content_rect());
    let gutter = region.right() - view.right();
    if gutter * ctx.pixels_per_point() > 1. {
        return Some(egui::pos2(view.right() + gutter / 2., view.center().y));
    }
    let offset = 1. / ctx.pixels_per_point();
    [editor.top() - offset, editor.bottom() + offset]
        .into_iter()
        .map(|y| egui::pos2(inside_parent.x, y))
        .find(|point| view.contains(*point) && !editor.contains(*point))
}

#[derive(Clone, Copy, Hash)]
pub(in crate::ui) enum NetworkFlow {
    Rollout,
    Evidence,
    SitePaths,
    HostSecurity,
    VlanGateways,
    NfsExportOps,
    RouteReading,
    OspfBroadcast,
}

impl LabApp {
    pub(in crate::ui) fn staged_network_tick(
        &mut self,
        ctx: &egui::Context,
        selected: NetworkFlow,
    ) {
        let id = egui::Id::new(("staged-network-native-flow", selected));
        let mut f = ctx.data(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        let ops = match selected {
            NetworkFlow::Rollout => plan(),
            NetworkFlow::Evidence => review_plan(),
            NetworkFlow::SitePaths => site_plan(),
            NetworkFlow::HostSecurity => host_security_plan(),
            NetworkFlow::VlanGateways => vlan_gateway_plan(),
            NetworkFlow::NfsExportOps => nfs_export_plan(),
            NetworkFlow::RouteReading => route_reading_plan(),
            NetworkFlow::OspfBroadcast => ospf_broadcast_plan(),
        };
        let Some(op) = ops.get(f.at) else {
            println!("STAGED_NETWORK_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        };
        let now = ctx.input(|i| i.time);
        if now < f.next || self.capture_pending {
            ctx.request_repaint_after(Duration::from_millis(25));
            return;
        }
        let since = f.since.get_or_insert_with(|| {
            println!("STAGED_NETWORK_NATIVE_STEP {} {op:?}", f.at);
            Instant::now()
        });
        assert!(
            since.elapsed() < Duration::from_secs(15),
            "Native rollout control deadline: {op:?}"
        );
        f.next = now + 0.12;
        let done = match op {
            Op::Open(lab) => {
                self.launch(self.scenarios.iter().position(|s| &s.id == lab).unwrap());
                self.lesson_scroll_reset = true;
                true
            }
            Op::Appearance(scale, light) => {
                use crate::NetworkReviewWindow::{Automatic, Fixed, Wide};
                let mode = crate::network_review_window();
                let narrow = match mode {
                    Automatic => *scale != 1.,
                    Fixed => true,
                    Wide => false,
                };
                if f.phase == 0 {
                    if let Some(focused) = ctx.memory(|memory| memory.focused()) {
                        ctx.memory_mut(|memory| memory.surrender_focus(focused));
                    }
                    self.scale = *scale;
                    ctx.set_zoom_factor(*scale);
                    self.theme_id = theme::inventory()
                        .iter()
                        .find(|t| t.light == *light)
                        .unwrap()
                        .id
                        .clone();
                    theme::apply(ctx, &self.theme_id);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!narrow));
                    if narrow {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
                    }
                    self.lesson_scroll_reset = true;
                    f.phase = 1;
                    false
                } else {
                    let area = ctx.content_rect();
                    let stable = f.previous_area == Some(area);
                    f.previous_area = Some(area);
                    let pixels = area.size() * ctx.pixels_per_point();
                    let (fullscreen, maximized, native) = ctx.input(|i| {
                        let viewport = i.viewport();
                        (
                            viewport.fullscreen,
                            viewport.maximized,
                            viewport.native_pixels_per_point,
                        )
                    });
                    let ready = fullscreen == Some(!narrow)
                        && (!narrow || maximized != Some(true))
                        && (ctx.zoom_factor() - *scale).abs() < 0.01;
                    if narrow && f.requested_pixels.is_none() && ready && stable {
                        let request = if mode == Fixed {
                            let [width, height] = crate::NETWORK_REVIEW_SIZE;
                            egui::vec2(width, height) * native.unwrap_or(1.)
                        } else {
                            capture_size::request(ctx)
                        };
                        f.requested_pixels = Some(request);
                        f.previous_area = None;
                        false
                    } else {
                        let proper_size = !narrow
                            || f.requested_pixels.is_some_and(|requested| {
                                capture_size::fits(pixels, requested, native.unwrap_or_default())
                            });
                        let settled = stable && ready && proper_size;
                        if settled {
                            println!(
                                "STAGED_NETWORK_NATIVE_GEOMETRY actual={pixels:?} zoom={}",
                                ctx.zoom_factor()
                            );
                        } else if !f.resize_reported && since.elapsed().as_secs_f32() > 2. {
                            println!(
                                "STAGED_NETWORK_WAITING_FOR_GEOMETRY actual={pixels:?} requested={:?} fullscreen={fullscreen:?} maximized={maximized:?} zoom={}",
                                f.requested_pixels,
                                ctx.zoom_factor()
                            );
                            f.resize_reported = true;
                        }
                        settled
                    }
                }
            }
            Op::Button(name) => self.click_visible_smoke_control(ctx, control(self, name)),
            Op::Choice(name) => {
                let item = self
                    .metrics
                    .automation_workshop
                    .controls
                    .iter()
                    .find(|row| row.0 == *name);
                self.mac_popup_choice(ctx, item.map(|row| row.1), item.map(|row| row.2))
            }
            Op::PlanInput(text) | Op::NfsConfigInput(text) => {
                let key = if matches!(op, Op::PlanInput(_)) {
                    "plan-draft"
                } else {
                    "nfs-exports"
                };
                match f.phase {
                    0 => {
                        let cell = control(self, key).map(|rect| {
                            egui::Rect::from_center_size(
                                rect.center(),
                                rect.size().min(egui::vec2(4., 4.)),
                            )
                        });
                        if self.click_visible_smoke_control(ctx, cell) {
                            f.phase = 1;
                        }
                        false
                    }
                    1 => {
                        let editor = self
                            .metrics
                            .automation_workshop
                            .focus
                            .iter()
                            .find(|entry| entry.0 == key)
                            .map(|entry| entry.1)
                            .unwrap();
                        assert_eq!(
                            ctx.memory(|memory| memory.focused()),
                            Some(editor),
                            "The click must focus the visible proposed configuration editor"
                        );
                        let modifiers = egui::Modifiers {
                            command: true,
                            ..egui::Modifiers::CTRL
                        };
                        self.pending_events.extend([
                            egui::Event::Key {
                                key: egui::Key::A,
                                physical_key: None,
                                pressed: true,
                                repeat: false,
                                modifiers,
                            },
                            egui::Event::Paste(text.clone()),
                            egui::Event::Key {
                                key: egui::Key::A,
                                physical_key: None,
                                pressed: false,
                                repeat: false,
                                modifiers,
                            },
                        ]);
                        f.phase = 2;
                        false
                    }
                    _ => {
                        let s = session(self);
                        let w = if s.view == teaching::View::Example {
                            &s.example.work
                        } else {
                            &s.assignment.work
                        };
                        use dlms_learning_lab::automation_workshop::workspace::Scope;
                        let draft = match &w.current {
                            Scope::DesiredConfig { edit } => &edit.draft,
                            Scope::NfsExportRetirement { terminal } => {
                                &terminal.state.runtime().servers[0].draft
                            }
                            _ => panic!("Expected the actual proposed configuration editor"),
                        };
                        assert_eq!(draft, text);
                        true
                    }
                }
            }
            Op::FieldInput(key, text) => match f.phase {
                0 => {
                    if self.click_visible_smoke_control(ctx, control(self, key)) {
                        f.phase = 1;
                    }
                    false
                }
                1 => {
                    let input = self
                        .metrics
                        .automation_workshop
                        .focus
                        .iter()
                        .find(|entry| entry.0 == *key)
                        .map(|entry| entry.1)
                        .unwrap();
                    assert_eq!(ctx.memory(|memory| memory.focused()), Some(input));
                    let modifiers = egui::Modifiers {
                        command: true,
                        ..egui::Modifiers::CTRL
                    };
                    self.pending_events.extend([
                        egui::Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers,
                        },
                        egui::Event::Paste(text.clone()),
                        egui::Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed: false,
                            repeat: false,
                            modifiers,
                        },
                    ]);
                    f.phase = 2;
                    false
                }
                _ => {
                    let state = session(self);
                    let w = if state.view == teaching::View::Example {
                        &state.example.work
                    } else {
                        &state.assignment.work
                    };
                    use dlms_learning_lab::automation_workshop::workspace::Scope;
                    let draft = match &w.current {
                        Scope::SocketRetirement { console } => &console.draft,
                        Scope::KeyedTransfer { edit } => {
                            if *key == "dns-other-rule" {
                                &edit.protected_fragment
                            } else {
                                &edit.served_fragment
                            }
                        }
                        Scope::RoutedVlans { draft } => {
                            if let Some(index) = key.strip_prefix("svi-address-") {
                                &draft.interfaces[index.parse::<usize>().unwrap()].0
                            } else if let Some(index) = key.strip_prefix("station-gateway-") {
                                &draft.gateways[index.parse::<usize>().unwrap()].0
                            } else {
                                panic!("Expected one of the actual gateway fields")
                            }
                        }
                        Scope::NfsExportRetirement { terminal } => &terminal.draft,
                        Scope::RouteReading { draft } => match *key {
                            "route-distance" => &draft.distance,
                            "route-cost" => &draft.cost,
                            "route-mask" => &draft.mask_query,
                            _ => panic!("Use an actual route table field"),
                        },
                        Scope::OspfBroadcast { draft } => {
                            let index = key
                                .strip_prefix("broadcast-priority-")
                                .expect("Use a supplied priority field")
                                .parse::<usize>()
                                .unwrap();
                            &draft.priorities[index]
                        }
                        _ => panic!("Use one of the supplied terminal and configuration fields"),
                    };
                    assert_eq!(draft, text);
                    true
                }
            },
            Op::TerminalEnter => {
                if f.phase == 0 {
                    self.pending_events
                        .extend([true, false].map(|pressed| egui::Event::Key {
                            key: egui::Key::Enter,
                            physical_key: None,
                            pressed,
                            repeat: false,
                            modifiers: Default::default(),
                        }));
                    f.phase = 1;
                    false
                } else {
                    let state = session(self);
                    let w = if state.view == teaching::View::Example {
                        &state.example.work
                    } else {
                        &state.assignment.work
                    };
                    use dlms_learning_lab::automation_workshop::{
                        nfs_tools::workspace::Origin, workspace::Scope,
                    };
                    match &w.current {
                        Scope::SocketRetirement { console } => {
                            assert!(console.draft.is_empty());
                            assert_eq!(console.history.last().unwrap().status, 0);
                        }
                        Scope::NfsExportRetirement { terminal } => {
                            assert!(terminal.draft.is_empty());
                            assert_eq!(terminal.history.last().unwrap().origin, Origin::Command);
                        }
                        _ => panic!("Expected an actual command terminal"),
                    }
                    true
                }
            }
            Op::Observed(phrase) | Op::ObservedAbsent(phrase) => {
                let state = session(self);
                let w = if state.view == teaching::View::Example {
                    &state.example.work
                } else {
                    &state.assignment.work
                };
                use dlms_learning_lab::automation_workshop::workspace::Scope;
                let note = match &w.current {
                    Scope::SocketRetirement { console } => {
                        console.history.last().map(|line| line.output.clone())
                    }
                    Scope::KeyedTransfer { edit } => edit.observation.clone(),
                    Scope::AccessDefault { .. } | Scope::RoutedVlans { .. } => {
                        crate::ui::automation_workshop_page::last_vlan_observation(
                            ctx,
                            &state.source_id,
                            state.view == teaching::View::Example,
                            matches!(w.current, Scope::RoutedVlans { .. }),
                        )
                    }
                    Scope::NfsExportRetirement { terminal } => {
                        terminal.history.last().map(|entry| entry.reply.clone())
                    }
                    _ => panic!("Expected an actual simulated observation"),
                };
                assert!(
                    note.as_ref().is_some_and(
                        |output| output.contains(phrase) == matches!(op, Op::Observed(_))
                    ),
                    "Failed {op:?}; observed {note:?}"
                );
                true
            }
            Op::Check { count, practice } => {
                let s = session(self);
                let lab = &self.scenarios[self.selected.unwrap()];
                let d = source::source(lab).unwrap();
                let (task, work) = if *practice {
                    (&d.example, &s.example.work)
                } else {
                    (&d.assignment, &s.assignment.work)
                };
                let checks = work.outcomes(task);
                assert!(work.checked);
                assert_eq!(
                    checks.iter().filter(|c| c.passed).count(),
                    *count,
                    "{checks:?}"
                );
                assert_eq!(
                    self.metrics.completion,
                    !*practice && *count == checks.len()
                );
                if *practice {
                    assert_eq!(s.assignment.work, Work::new(&d.assignment));
                }
                true
            }
            Op::Unchecked => {
                assert!(!session(self).assignment.work.checked);
                assert!(!self.metrics.completion);
                true
            }
            Op::Capture(name, anchor) => {
                if f.phase == 0 {
                    let r = control(self, anchor).expect("Actual rendered capture anchor");
                    if self.reveal_smoke_control(ctx, r) {
                        self.capture_pending = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                            egui::UserData::new(name.clone()),
                        ));
                        f.phase = 1;
                    }
                    false
                } else {
                    !self.capture_pending
                }
            }
            Op::SaveReadback => {
                self.save();
                assert!(self.storage_error.is_none());
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(loaded).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
        };
        // Only redirect page scrolling when it would land in the nested editor.
        // Leave popup scrolling on the popup itself.
        if !matches!(op, Op::Choice(_))
            && self
                .pending_events
                .iter()
                .any(|event| matches!(event, egui::Event::MouseWheel { .. }))
            && let Some(destination) = editor_scroll_point(self, ctx)
        {
            for event in &mut self.pending_events {
                if let egui::Event::PointerMoved(point) = event {
                    *point = destination;
                }
            }
        }
        if done {
            let scroll = egui::Id::new(("bounded-native-scroll", self.smoke_stage));
            let popup_scroll = egui::Id::new(("mac-popup-scroll", self.smoke_stage));
            ctx.data_mut(|d| {
                d.remove::<usize>(scroll);
                d.remove::<u8>(popup_scroll);
            });
            f.at += 1;
            f.phase = 0;
            f.since = None;
            f.previous_area = None;
            f.requested_pixels = None;
            f.resize_reported = false;
        }
        ctx.data_mut(|d| d.insert_temp(id, f));
        ctx.request_repaint_after(Duration::from_millis(25));
    }
}
