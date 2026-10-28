use dlms_learning_lab::{
    catalog::{self, Filter},
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
};
fn scenario(id: &str) -> pack::Scenario {
    pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.id == id)
        .unwrap()
}
fn solution(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for cmd in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, cmd).unwrap();
    }
    if let (State::Script { source, run }, engine::Goal::Script { cases }) = (&mut state, &s.goal) {
        *run = Some(dlms_learning_lab::shell::Record {
            engine_revision: dlms_learning_lab::shell::EVALUATOR_REVISION,
            source: source.clone(),
            results: cases
                .iter()
                .map(|c| dlms_learning_lab::shell::run(source, c))
                .collect(),
        });
    }
    state
}
#[test]
fn all_nine_start_unsolved_and_reach_success() {
    let labs = pack::bundled().unwrap();
    assert!(labs.len() >= 9);
    for s in labs.into_iter().take(9) {
        assert!(!engine::solved(&s.initial, &s.goal), "{}", s.id);
        assert!(engine::solved(&solution(&s), &s.goal), "{}", s.id);
    }
}
#[test]
fn success_is_recomputed_after_every_repair_is_broken() {
    let breaks = [
        ("ccna-ipv4", "answer first 192.168.10.64"),
        ("ccna-vlsm", "allocate lab 10.20.0.0/28"),
        ("ccna-gateway", "ip route replace default via 192.168.40.1"),
        ("lpic1-permissions", "chmod 600 /srv/report.txt"),
        ("lpic1-traversal", "chmod 740 /srv/project"),
        ("lpic1-logs", "cat /var/log/app.log"),
        ("lpic2-service", "systemctl stop atlas"),
        ("lpic2-dns", "dns set portal.lab.test 192.0.2.1"),
        ("lpic2-storage", "umount /data"),
    ];
    for (id, cmd) in breaks {
        let s = scenario(id);
        let mut state = solution(&s);
        engine::execute(&mut state, cmd).unwrap();
        assert!(!engine::solved(&state, &s.goal), "{id}");
    }
}
#[test]
fn subnet_math_handles_every_prefix_without_overflow() {
    for p in 0..=32 {
        let (n, b) = engine::bounds("255.255.255.255".parse().unwrap(), p).unwrap();
        assert!(n <= b);
        assert_eq!((b as u64) - (n as u64) + 1, 1u64 << (32 - p));
    }
    assert!(engine::bounds("0.0.0.0".parse().unwrap(), 33).is_none());
    assert!(engine::cidr("10.0.0.7/24").is_none());
    assert_eq!(
        engine::bounds("192.168.10.77".parse().unwrap(), 27),
        Some((0xc0a80a40, 0xc0a80a5f))
    );
}
#[test]
fn alternate_vlsm_layout_and_invalid_capacity_overlap() {
    let s = scenario("ccna-vlsm");
    let mut state = s.initial.clone();
    for c in [
        "allocate design 10.20.0.128/26",
        "allocate support 10.20.0.0/27",
        "allocate lab 10.20.0.32/28",
    ] {
        engine::execute(&mut state, c).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
    engine::execute(&mut state, "allocate support 10.20.0.0/28").unwrap();
    assert!(!engine::solved(&state, &s.goal));
    engine::execute(&mut state, "allocate support 10.20.1.0/27").unwrap();
    assert!(!engine::solved(&state, &s.goal));
    assert!(engine::execute(&mut state, "allocate support 10.20.0.7/27").is_err());
}
#[test]
fn symbolic_permissions_and_ownership_are_valid_alternates() {
    let s = scenario("lpic1-permissions");
    let mut st = s.initial.clone();
    engine::execute(&mut st, "chmod u=rw,g=rw,o= /srv/report.txt").unwrap();
    assert!(engine::solved(&st, &s.goal));
    let mut st = s.initial.clone();
    engine::execute(&mut st, "chown alex:editors /srv/report.txt").unwrap();
    assert!(engine::solved(&st, &s.goal));
    engine::execute(&mut st, "chmod 666 /srv/report.txt").unwrap();
    assert!(!engine::solved(&st, &s.goal));
}
#[test]
fn unix_owner_precedence_and_all_parent_traversal_are_modeled() {
    let s = scenario("lpic1-traversal");
    let mut st = solution(&s);
    engine::execute(&mut st, "chmod 640 /srv/project").unwrap();
    assert!(!engine::solved(&st, &s.goal));
    engine::execute(&mut st, "chmod 750 /srv/project").unwrap();
    engine::execute(&mut st, "chown blair:editors /srv/project/notes.txt").unwrap();
    engine::execute(&mut st, "chmod 040 /srv/project/notes.txt").unwrap();
    assert!(
        !engine::solved(&st, &s.goal),
        "owner cannot fall back to group permission"
    );
}
#[test]
fn service_requires_restart_and_enable_independently() {
    let s = scenario("lpic2-service");
    let mut st = s.initial.clone();
    assert!(engine::execute(&mut st, "systemctl start atlas").is_err());
    engine::execute(&mut st, "systemctl enable atlas").unwrap();
    assert!(!engine::solved(&st, &s.goal));
    engine::execute(&mut st, "config set port 80").unwrap();
    engine::execute(&mut st, "systemctl start atlas").unwrap();
    engine::execute(&mut st, "config set port 8080").unwrap();
    engine::execute(&mut st, "systemctl start atlas").unwrap();
    assert!(!engine::solved(&st, &s.goal));
    engine::execute(&mut st, "systemctl restart atlas").unwrap();
    assert!(engine::solved(&st, &s.goal));
    engine::execute(&mut st, "config set port invalid").unwrap();
    assert!(!engine::solved(&st, &s.goal));
    assert!(engine::execute(&mut st, "systemctl restart atlas").is_err());
}
#[test]
fn dns_edit_serial_reload_and_resolution_are_distinct() {
    let s = scenario("lpic2-dns");
    let mut st = s.initial.clone();
    engine::execute(&mut st, "dns set portal.lab.test 192.0.2.80").unwrap();
    assert!(!engine::solved(&st, &s.goal));
    assert!(engine::execute(&mut st, "systemctl reload named").is_err());
    assert!(
        engine::execute(&mut st, "dig portal.lab.test")
            .unwrap()
            .contains("192.0.2.8")
    );
    engine::execute(&mut st, "dns serial 2026102102").unwrap();
    engine::execute(&mut st, "systemctl reload named").unwrap();
    assert!(engine::solved(&st, &s.goal));
    assert!(engine::execute(&mut st, "dns set portal.lab.test 999.0.0.1").is_err());
}
#[test]
fn storage_requires_writable_mount_and_persistent_device() {
    let s = scenario("lpic2-storage");
    let mut st = s.initial.clone();
    assert!(engine::execute(&mut st, "rm -r /data/cache").is_err());
    assert!(engine::execute(&mut st, "mount /data").is_err());
    engine::execute(&mut st, "mount -o remount,rw /data").unwrap();
    engine::execute(&mut st, "rm -r /data/cache").unwrap();
    assert!(!engine::solved(&st, &s.goal));
    engine::execute(&mut st, "config set device /dev/vdb1").unwrap();
    assert!(engine::solved(&st, &s.goal));
    engine::execute(&mut st, "rm -r /data/cache").unwrap();
    assert!(engine::solved(&st, &s.goal));
    assert!(engine::execute(&mut st, "rm /etc/fstab").is_err());
}
#[test]
fn log_alternates_pipeline_and_limits() {
    let s = scenario("lpic1-logs");
    let mut st = s.initial.clone();
    engine::execute(&mut st, "grep ERROR /var/log/app.log").unwrap();
    assert!(engine::solved(&st, &s.goal));
    engine::execute(&mut st, "cat /var/log/app.log | grep ERROR | tail -n 1").unwrap();
    assert!(!engine::solved(&st, &s.goal));
    assert!(engine::execute(&mut st, "cat /etc/passwd").is_err());
    assert!(engine::execute(&mut st, "cat /var/log/app.log | sh").is_err());
}
#[test]
fn no_shell_escape_or_unsupported_command_succeeds() {
    for s in pack::bundled().unwrap().into_iter().take(9) {
        for c in [
            "sudo whoami",
            "sh -c id",
            "$(touch /tmp/escaped)",
            "help ; id",
            "cat /etc/passwd > /tmp/foo",
            "curl https://example.com",
        ] {
            let mut st = s.initial.clone();
            assert!(engine::execute(&mut st, c).is_err(), "{} {c}", s.id);
            assert_eq!(st, s.initial);
        }
    }
}
#[test]
fn resume_reset_only_selected_and_round_trip_all_states() {
    let labs = pack::bundled().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &labs {
        let mut a = Attempt::new(s, 3);
        a.state = solution(s);
        a.step = 1.min(s.steps.len() - 1);
        a.hints = 1;
        a.challenge = true;
        p.attempts.insert(s.id.clone(), a);
    }
    p.favorites.insert(labs[0].id.clone());
    progress::save(&path, &p).unwrap();
    let mut q = progress::load(&path, &labs).unwrap();
    assert_eq!(q.attempts.len(), labs.len());
    for s in &labs {
        let a = &q.attempts[&s.id];
        assert!(engine::solved(&a.state, &s.goal));
        assert_eq!(a.step, 1.min(s.steps.len() - 1));
        assert!(a.challenge);
    }
    q.attempts
        .insert(labs[0].id.clone(), Attempt::new(&labs[0], 4));
    progress::save(&path, &q).unwrap();
    let q = progress::load(&path, &labs).unwrap();
    assert!(!engine::solved(
        &q.attempts[&labs[0].id].state,
        &labs[0].goal
    ));
    assert!(engine::solved(
        &q.attempts[&labs[1].id].state,
        &labs[1].goal
    ));
    assert!(q.favorites.contains(&labs[0].id));
}
#[test]
fn corrupt_or_future_progress_is_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("p.json");
    for data in [
        "not json",
        r#"{"schema_version":999,"clock":0,"favorites":[],"attempts":{}}"#,
    ] {
        std::fs::write(&path, data).unwrap();
        assert!(progress::load(&path, &pack::bundled().unwrap()).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), data);
    }
}
#[test]
fn packs_reject_bad_schema_ids_metadata_state_and_unreachable_solutions() {
    let source = include_str!("../scenarios/starter-v1.json");
    let base: serde_json::Value = serde_json::from_str(source).unwrap();
    for (pointer, value) in [
        ("/schema_version", serde_json::json!(99)),
        ("/scenarios/0/id", serde_json::json!("../escape")),
        ("/scenarios/1/id", serde_json::json!("ccna-ipv4")),
        ("/scenarios/0/hints", serde_json::json!([])),
        ("/scenarios/0/initial/prefix", serde_json::json!(33)),
        ("/scenarios/0/solution", serde_json::json!(["show"])),
        ("/scenarios/0/goal/kind", serde_json::json!("Mount")),
        (
            "/scenarios/0/sources",
            serde_json::json!(["http://untrusted"]),
        ),
        ("/scenarios/0/initial/extra", serde_json::json!(true)),
    ] {
        let mut v = base.clone();
        if pointer.ends_with("extra") {
            v["scenarios"][0]["initial"]["extra"] = value;
        } else {
            *v.pointer_mut(pointer).unwrap() = value;
        }
        assert!(pack::load(&v.to_string()).is_err(), "{pointer}");
    }
}
#[test]
fn catalog_900_filters_and_synthetic_boundary() {
    let labs = pack::bundled().unwrap();
    let large = catalog::synthetic_catalog(&labs[..9], 900);
    assert_eq!(large.len(), 900);
    assert!(
        large
            .iter()
            .all(|s| s.id.starts_with("synthetic-") && s.title.starts_with("SYNTHETIC"))
    );
    let mut f = Filter {
        track: "CCNA".into(),
        ..Default::default()
    };
    assert_eq!(
        large
            .iter()
            .filter(|s| catalog::matches(s, &f, false, false, false))
            .count(),
        300
    );
    f.search = "boundaries".into();
    f.max_minutes = 6;
    assert_eq!(
        large
            .iter()
            .filter(|s| catalog::matches(s, &f, false, false, false))
            .count(),
        100
    );
    f.favorites = true;
    assert_eq!(
        large
            .iter()
            .filter(|s| catalog::matches(s, &f, false, false, false))
            .count(),
        0
    );
    f.favorites = false;
    f.status = "Complete".into();
    assert_eq!(
        large
            .iter()
            .filter(|s| catalog::matches(s, &f, true, true, false))
            .count(),
        100
    );
}

#[test]
fn broken_attempts_are_resumable_instead_of_becoming_invalid_saves() {
    let labs = pack::bundled().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("p.json");
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &labs {
        let mut a = Attempt::new(s, 0);
        a.state = solution(s);
        let cmd = match s.id.as_str() {
            "ccna-ipv4" => "answer last 0.0.0.0",
            "ccna-vlsm" => "allocate design 10.21.0.0/26",
            "ccna-gateway" => "ip addr set 192.168.40.65/24",
            "lpic1-permissions" => "chmod 000 /srv/report.txt",
            "lpic1-traversal" => "chmod 000 /srv/project",
            "lpic1-logs" => "grep INFO /var/log/app.log",
            "lpic2-service" => "config set port broken",
            "lpic2-dns" => "dns serial 1",
            "lpic2-storage" => "config set device invalid-device",
            _ => "# unfinished script",
        };
        if let State::EtherChannel { simulation } = &a.state {
            // A genuine admitted bad configuration, preserving the source and
            // topology: disable every member after the taught solution passed.
            let mut configuration = simulation.configuration().clone();
            for device in configuration.devices.values_mut() {
                for port in device.ports.values_mut() {
                    port.up = false;
                }
            }
            let action = dlms_learning_lab::etherchannel::Action::Configure { configuration };
            engine::execute_for_goal(
                &mut a.state,
                &s.goal,
                &format!(
                    "{}{}",
                    dlms_learning_lab::etherchannel_workspace::PREFIX,
                    serde_json::to_string(&action).unwrap()
                ),
            )
            .unwrap();
        } else if let State::Nfs { simulation } = &a.state {
            use dlms_learning_lab::{nfs, nfs_workspace};
            let engine::Goal::Nfs { definition } = &s.goal else {
                panic!("NFS goal context");
            };
            let (client, mountpoint) = definition
                .requirements
                .iter()
                .find_map(|r| match r {
                    nfs_workspace::Requirement::CurrentAccess {
                        request:
                            nfs::Request::Read {
                                client, mountpoint, ..
                            },
                        expected: nfs::Expected::Read { .. },
                    } => Some((client.clone(), mountpoint.clone())),
                    _ => None,
                })
                .expect("Authored NFS task requires an actual positive current read");
            assert!(engine::solved(&a.state, &s.goal));
            assert!(
                simulation
                    .runtime()
                    .mounts
                    .iter()
                    .any(|m| m.client == client && m.specification.mountpoint == mountpoint),
                "The required positive read has a real held mount before breaking it"
            );
            let action = nfs::Action::Unmount {
                client: client.clone(),
                mountpoint: mountpoint.clone(),
            };
            let output = engine::execute_for_goal(
                &mut a.state,
                &s.goal,
                &format!(
                    "{}{}",
                    nfs_workspace::PREFIX,
                    serde_json::to_string(&action).unwrap()
                ),
            )
            .unwrap();
            assert_eq!(output, "Virtual mount removed");
            let State::Nfs { simulation } = &a.state else {
                unreachable!()
            };
            assert!(
                !simulation
                    .runtime()
                    .mounts
                    .iter()
                    .any(|m| m.client == client && m.specification.mountpoint == mountpoint)
            );
            assert!(
                a.workspace.is_none(),
                "Typed state-only adapter does not fabricate editor/practice metadata"
            );
        } else if matches!(a.state, State::Cron { .. }) {
            use dlms_learning_lab::cron_workspace::{self as cron_work, Document, ReferenceAction};
            let engine::Goal::Cron { definition } = &s.goal else {
                panic!("Cron context");
            };
            let target = definition
                .editable
                .iter()
                .find(|target| matches!(target, Document::User { .. } | Document::System))
                .expect("Authored Cron task requires an editable running schedule")
                .clone();
            let text = if target == Document::System {
                "* * * * * root echo wrong\n"
            } else {
                "* * * * * echo wrong\n"
            };
            let action = ReferenceAction::Install {
                actor: "root".into(),
                target,
                text: text.into(),
            };
            engine::execute_for_goal(
                &mut a.state,
                &s.goal,
                &format!(
                    "{}{}",
                    cron_work::PREFIX,
                    serde_json::to_string(&action).unwrap()
                ),
            )
            .unwrap();
        } else if let State::BasicNat { simulation } = &a.state {
            use dlms_learning_lab::{nat, nat_workspace};
            let engine::Goal::BasicNat { definition } = &s.goal else {
                panic!("NAT goal context");
            };
            let required = definition.requirements.iter().find_map(|r| match r {
                nat_workspace::Requirement::CurrentMapping {
                    inside,
                    mapping_kind,
                    ..
                } => Some((inside, *mapping_kind)),
                _ => None,
            });
            let action = match required {
                Some((_, nat_workspace::Kind::Dynamic)) => nat::Action::ClearDynamic {},
                Some((inside, nat_workspace::Kind::Static)) => {
                    let mut configuration = simulation.configuration().clone();
                    assert!(configuration.statics.remove(inside).is_some());
                    nat::Action::Configure { configuration }
                }
                None => {
                    let mut configuration = simulation.configuration().clone();
                    assert!(configuration.dynamic.take().is_some());
                    nat::Action::Configure { configuration }
                }
            };
            engine::execute_for_goal(
                &mut a.state,
                &s.goal,
                &format!(
                    "{}{}",
                    nat_workspace::PREFIX,
                    serde_json::to_string(&action).unwrap()
                ),
            )
            .unwrap();
        } else if matches!(a.state, State::RpmPackages { .. }) {
            let engine::Goal::RpmPackages { definition } = &s.goal else {
                panic!("RPM goal context");
            };
            let target = definition
                .requirements
                .iter()
                .find_map(|r| {
                    if let dlms_learning_lab::rpm_packages_workspace::Requirement::Installed {
                        package,
                        ..
                    } = r
                    {
                        Some(package.name.as_str())
                    } else {
                        None
                    }
                })
                .expect("Authored RPM tasks require a consumer record");
            engine::execute_for_goal(&mut a.state, &s.goal, &format!("rpm -e {target}")).unwrap();
        } else if matches!(a.state, State::ApacheStatic { .. }) {
            let action = dlms_learning_lab::apache_static::Action::Stop {};
            engine::execute_for_goal(
                &mut a.state,
                &s.goal,
                &format!(
                    "{}{}",
                    dlms_learning_lab::apache_static_workspace::PREFIX,
                    serde_json::to_string(&action).unwrap()
                ),
            )
            .unwrap();
        } else if matches!(a.state, State::DebianPackages { .. }) {
            let engine::Goal::DebianPackages { definition } = &s.goal else {
                panic!("package assignment goal kind");
            };
            let target = definition
                .requirements
                .iter()
                .find_map(|r| match r {
                    dlms_learning_lab::debian_packages_workspace::Requirement::Installed {
                        package,
                        ..
                    }
                    | dlms_learning_lab::debian_packages_workspace::Requirement::RetainedConfig {
                        package,
                        ..
                    } => Some(package),
                    _ => None,
                })
                .expect("package task requires an installed or retained record");
            // Purge an actual required record via the supported goal-aware CLI.
            engine::execute_for_goal(&mut a.state, &s.goal, &format!("dpkg -P {target}")).unwrap();
        } else if let State::SpanningTree { simulation } = &a.state {
            let bridge = simulation.topology().bridges.keys().next().unwrap().clone();
            let action = dlms_learning_lab::spanning_tree::Action::BridgeOnline {
                bridge,
                online: false,
            };
            // Every normal task explicitly preserves supplied bridge availability.
            engine::execute_for_goal(
                &mut a.state,
                &s.goal,
                &format!(
                    "{}{}",
                    dlms_learning_lab::spanning_tree_workspace::PREFIX,
                    serde_json::to_string(&action).unwrap()
                ),
            )
            .unwrap();
        } else if let State::Ipv4Acl { network } = &mut a.state {
            // Removing the real policies permits every forbidden flow again.
            network.devices.clear();
        } else if let State::UnitManager { simulation } = &a.state {
            // Stop an actually running required program through the pinned adapter.
            let unit = simulation
                .runtime()
                .iter()
                .find_map(|(unit, runtime)| {
                    matches!(
                        runtime,
                        dlms_learning_lab::unit_manager::Runtime::Running { .. }
                    )
                    .then(|| unit.clone())
                })
                .expect("reference has a running application");
            let action = dlms_learning_lab::unit_manager_workspace::EditAction::Manager {
                action: dlms_learning_lab::unit_manager::Action::Stop { unit },
            };
            engine::execute_for_goal(
                &mut a.state,
                &s.goal,
                &format!("unit-manager {}", serde_json::to_string(&action).unwrap()),
            )
            .unwrap();
        } else if let State::Ospf { simulation } = &a.state {
            let (router, current) = simulation.configuration().routers.iter().next().unwrap();
            let mut configuration = current.process.clone();
            configuration.enabled = false;
            let action = dlms_learning_lab::ospf::Action::SetProcess {
                router: router.clone(),
                process: configuration,
            };
            engine::execute_for_goal(
                &mut a.state,
                &s.goal,
                &format!("ospf {}", serde_json::to_string(&action).unwrap()),
            )
            .unwrap();
        } else if matches!(a.state, State::Accounts { .. }) {
            engine::execute_for_goal(
                &mut a.state,
                &s.goal,
                "chown root:root /srv/shared/outside.txt",
            )
            .unwrap();
        } else if let State::Discovery { simulation } = &mut a.state {
            let mut wrong = simulation.topology().clone();
            for device in wrong.devices.values_mut() {
                device.enabled = false;
            }
            simulation.reconfigure(wrong).unwrap();
        } else if let State::PersistentMounts { storage } = &mut a.state {
            storage
                .save_configuration("unfinished mount row\n".into())
                .unwrap();
        } else if let State::Processes { table } = &mut a.state {
            // Every authored process task preserves its current session controller.
            // Breaking that real requirement must still yield a resumable attempt.
            let output = table
                .execute(&format!("renice 19 -p {}", table.session))
                .unwrap();
            assert_eq!(output.status, 0);
        } else if let State::AuthoritativeDns { server } = &mut a.state {
            server.configured.records.clear();
        } else if let State::Switching { fabric } = &mut a.state {
            for s in fabric.switches.values_mut() {
                for p in s.ports.values_mut() {
                    p.up = false;
                }
            }
        } else if let State::Ipv6 { topology } = &mut a.state {
            for d in topology.devices.values_mut() {
                for p in d.interfaces.values_mut() {
                    p.up = false;
                }
            }
        } else if let State::Ipv4Routing { topology } = &mut a.state {
            for device in topology.devices.values_mut() {
                for interface in device.interfaces.values_mut() {
                    interface.up = false;
                }
            }
        } else if let State::MacLearning { simulation } = &mut a.state {
            let engine::Goal::MacLearning { requirements, .. } = &s.goal else {
                panic!()
            };
            let (switch, vlan) = requirements
                .iter()
                .find_map(|r| {
                    if let dlms_learning_lab::mac_learning_workspace::Requirement::Outcome {
                        outcome:
                            dlms_learning_lab::mac_learning::Requirement::Learned {
                                switch, vlan, ..
                            },
                    } = r
                    {
                        Some((switch, *vlan))
                    } else {
                        None
                    }
                })
                .expect("authored source-learning tasks require a learned row");
            simulation.clear_dynamic(switch, Some(vlan)).unwrap();
        } else if let State::DhcpV4 { simulation } = &mut a.state {
            // Reachable wrong policy, retaining the exact supplied topology.
            // DHCP mutations deliberately cannot use the context-free wrapper.
            let mut policy = simulation.policy().clone();
            policy.server_enabled = false;
            simulation.replace_policy(policy).unwrap();
        } else if let State::Filesystem { tree } = &mut a.state {
            for text in tree.files.values_mut() {
                text.push_str("changed");
            }
        } else {
            engine::execute(&mut a.state, cmd).unwrap();
        }
        assert!(!engine::solved(&a.state, &s.goal));
        p.attempts.insert(s.id.clone(), a);
    }
    assert_eq!(p.attempts.len(), labs.len());
    progress::save(&path, &p).unwrap();
    let q = progress::load(&path, &labs).unwrap();
    assert_eq!(q.attempts.len(), labs.len());
    for s in &labs {
        assert!(!engine::solved(&q.attempts[&s.id].state, &s.goal));
    }
}

#[test]
fn legacy_v1_fixture_preserves_all_nine_attempts_without_rewriting_the_original() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("p.json");
    let original = include_bytes!("fixtures/progress-v1.json");
    std::fs::write(&path, original).unwrap();
    let labs = pack::bundled().unwrap();
    let p = progress::load(&path, &labs).unwrap();
    assert_eq!(p.attempts.len(), 9);
    assert_eq!(p.clock, 9);
    assert!(p.favorites.contains("ccna-ipv4"));
    for a in p.attempts.values() {
        assert_eq!(a.scenario_version, 1);
        assert_eq!(a.step, 1);
        assert_eq!(a.hints, 1);
        assert_eq!(
            a.transcript,
            vec!["Synthetic legacy fixture: no learner recording."]
        );
    }
    assert!(p.attempts["ccna-ipv4"].challenge);
    assert!(
        matches!(&p.attempts["lpic2-service"].state,State::Service{port,running:true,loaded_port:Some(80),journal,failed:false,..}if port=="80"&&journal.is_empty())
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
    progress::save(&path, &p).unwrap();
    let q = progress::load(&path, &labs).unwrap();
    for (id, a) in &p.attempts {
        assert_eq!(q.attempts[id].state, a.state);
        assert_eq!(q.attempts[id].challenge, a.challenge);
        assert_eq!(q.attempts[id].visited, a.visited);
    }
}
#[test]
fn malformed_native_answer_drafts_survive_resume_and_do_not_pass() {
    let labs = pack::bundled().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("p.json");
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in labs.iter().take(2) {
        let mut a = Attempt::new(s, 1);
        match &mut a.state {
            State::Subnet { answers, .. } => answers[0] = "999.1.1.1".into(),
            State::Vlsm { allocations, .. } => allocations[0].cidr = "10.20.".into(),
            _ => unreachable!(),
        }
        p.attempts.insert(s.id.clone(), a);
    }
    progress::save(&path, &p).unwrap();
    let q = progress::load(&path, &labs).unwrap();
    for s in labs.iter().take(2) {
        assert_eq!(q.attempts[&s.id].state, p.attempts[&s.id].state);
        assert!(!engine::solved(&q.attempts[&s.id].state, &s.goal));
    }
}
#[test]
fn vlsm_partial_feedback_accepts_a_valid_row_before_the_whole_plan_is_complete() {
    let s = scenario("ccna-vlsm");
    let mut st = s.initial.clone();
    engine::execute(&mut st, "allocate design 10.20.0.0/26").unwrap();
    let c = engine::checks(&st, &s.goal);
    assert_eq!(c.iter().filter(|x| x.0).count(), 1);
    assert!(!engine::solved(&st, &s.goal));
}
#[test]
fn linux_outputs_use_familiar_columns_ids_and_unit_status() {
    let s = scenario("lpic1-permissions");
    let mut st = s.initial.clone();
    let ls = engine::execute(&mut st, "ls -l /srv/report.txt").unwrap();
    assert!(ls.starts_with("-rw------- 1 root editors "));
    assert!(ls.ends_with("/srv/report.txt"));
    let id = engine::execute(&mut st, "id alex").unwrap();
    assert_eq!(
        id,
        "uid=1000(alex) gid=1000(alex) groups=1000(alex),2000(editors)"
    );
    let s = scenario("lpic2-service");
    let mut st = s.initial.clone();
    let status = engine::execute(&mut st, "systemctl status atlas").unwrap();
    assert!(status.contains("Loaded: loaded"));
    assert!(status.contains("Active: failed (Result: exit-code)"));
    let history = engine::execute(&mut st, "journalctl -u atlas").unwrap();
    assert!(history.contains("systemd[1]: atlas.service:"));
    assert!(history.contains("Invalid port 'eighty'"));
    engine::edit_service_port(&mut st, "8080").unwrap();
    assert_eq!(
        engine::execute(&mut st, "journalctl -u atlas").unwrap(),
        history,
        "editing does not rewrite history"
    );
    engine::execute(&mut st, "systemctl restart atlas").unwrap();
    assert!(
        engine::execute(&mut st, "systemctl status atlas")
            .unwrap()
            .contains("Active: active (running)")
    );
    let started = engine::execute(&mut st, "journalctl -u atlas").unwrap();
    assert!(started.contains("Invalid port 'eighty'"));
    assert!(started.contains("Listening on port 8080"));
    engine::execute(&mut st, "systemctl stop atlas").unwrap();
    assert!(
        engine::execute(&mut st, "systemctl status atlas")
            .unwrap()
            .contains("Active: inactive (dead)")
    );
}
#[test]
fn pack_rejects_bad_guidance_and_malformed_authored_allocations() {
    let source = include_str!("../scenarios/starter-v1.json");
    let base: serde_json::Value = serde_json::from_str(source).unwrap();
    for (pointer, value) in [
        ("/scenarios/0/guide/0/why", serde_json::json!("")),
        (
            "/scenarios/1/initial/allocations/0/cidr",
            serde_json::json!("10.20."),
        ),
        (
            "/scenarios/2/guide/0/commands/0/explanation",
            serde_json::json!(""),
        ),
    ] {
        let mut v = base.clone();
        *v.pointer_mut(pointer).unwrap() = value;
        assert!(pack::load(&v.to_string()).is_err(), "{pointer}");
    }
}

#[test]
fn ls_lists_directory_children_while_ld_inspects_the_directory() {
    let s = scenario("lpic1-traversal");
    let mut st = s.initial.clone();
    let listing = engine::execute(&mut st, "ls -l /srv/project").unwrap();
    assert!(listing.starts_with("total 0\n-rw-r-----"));
    assert!(listing.ends_with("notes.txt"));
    let directory = engine::execute(&mut st, "ls -ld /srv/project").unwrap();
    assert!(directory.starts_with("drwxr----- 2 root editors "));
    assert!(directory.ends_with("/srv/project"));
}
