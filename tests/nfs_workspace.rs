//! Prospective adapter checks, using supplied data only; no NFS utility or host mount.
use dlms_learning_lab::{nfs as core, nfs_workspace as work};
use serde::Deserialize;
use serde_json::json;
#[derive(Deserialize)]
struct Case {
    definition: work::Definition,
    reference: Vec<core::Action>,
    alternatives: Vec<Vec<core::Action>>,
}
fn cases() -> Vec<Case> {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/nfs-workbench-v1.json")).unwrap();
    serde_json::from_value(v["cases"].clone()).unwrap()
}
fn run(d: &work::Definition, s: &mut core::State, t: &mut work::Session, actions: &[core::Action]) {
    for a in actions {
        assert!(!matches!(
            work::perform(d, s, t, a).unwrap(),
            core::Decision::Denied { .. }
        ));
    }
}
fn solved(c: &Case) -> (core::State, work::Session) {
    let mut s = c.definition.initial.clone();
    let mut t = work::Session::default();
    run(&c.definition, &mut s, &mut t, &c.reference);
    assert!(c.definition.checks(&s).iter().all(|r| r.0));
    (s, t)
}
fn exact<T: serde::Serialize>(a: &T, b: &T) {
    assert!(
        serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap(),
        "full serialized caller changed"
    );
}
#[test]
fn four_meaningful_shapes_reference_and_two_independent_plans_keep_current_truth() {
    for c in cases() {
        assert!(
            !c.definition
                .checks(&c.definition.initial)
                .iter()
                .all(|r| r.0)
        );
        let (s, _) = solved(&c);
        assert_eq!(c.definition.checks(&s).len(), c.definition.row_count());
        for plan in &c.alternatives {
            let mut s = c.definition.initial.clone();
            let mut t = work::Session::default();
            run(&c.definition, &mut s, &mut t, plan);
            assert!(c.definition.checks(&s).iter().all(|r| r.0));
        }
    }
}
#[test]
fn pending_relevant_layers_fail_but_unsubmitted_queries_and_unrelated_editor_do_not() {
    let c = cases().remove(0);
    let (mut s, mut t) = solved(&c);
    let before = s.clone();
    t.relative = "../unfinished request".into();
    t.write_text = "unsubmitted existing-file replacement".into();
    assert!(t.valid_for(&s));
    assert!(c.definition.checks(&s).iter().all(|r| r.0));
    exact(&s, &before);
    let outsider = s
        .inventory()
        .clients
        .iter()
        .find(|c| c.id.ends_with("outside"))
        .unwrap()
        .id
        .clone();
    work::perform(
        &c.definition,
        &mut s,
        &mut t,
        &core::Action::SetFstabDraft {
            client: outsider,
            text: "bounded malformed unrelated draft\n".into(),
        },
    )
    .unwrap();
    assert!(c.definition.checks(&s).iter().all(|r| r.0));
    let server = s.inventory().servers[0].id.clone();
    let old = s.runtime().servers[0].clone();
    work::perform(
        &c.definition,
        &mut s,
        &mut t,
        &core::Action::SetExportsDraft {
            server: server.clone(),
            text: "malformed pending\n".into(),
        },
    )
    .unwrap();
    assert_eq!(s.runtime().servers[0].saved, old.saved);
    assert_eq!(s.runtime().servers[0].applied, old.applied);
    assert!(!c.definition.checks(&s).iter().all(|r| r.0));
    work::perform(
        &c.definition,
        &mut s,
        &mut t,
        &core::Action::SaveExportsDraft { server },
    )
    .unwrap();
    assert_eq!(s.runtime().servers[0].applied, old.applied);
}
#[test]
fn saved_new_mount_failure_does_not_revoke_actual_read_but_cannot_pass_as_denial() {
    let c = cases().remove(0);
    let (mut s, mut session) = solved(&c);
    let server = s.inventory().servers[0].id.clone();
    let client = s.inventory().clients[0].id.clone();
    work::perform(
        &c.definition,
        &mut s,
        &mut session,
        &core::Action::SetExportsDraft {
            server: server.clone(),
            text: "not an exports record\n".into(),
        },
    )
    .unwrap();
    work::perform(
        &c.definition,
        &mut s,
        &mut session,
        &core::Action::SaveExportsDraft {
            server: server.clone(),
        },
    )
    .unwrap();
    let mut d = c.definition.clone();
    d.fresh[0].base = core::TrialBase::Saved {
        servers: vec![server],
        client,
    };
    d.fresh[0].steps[0].expected = core::Expected::Denied {
        stage: core::DenialStage::ExportAdmission,
    };
    let rows = d.checks(&s);
    assert!(rows[2].0, "actual current read still works");
    assert!(
        !rows[1 + d.requirements.len()].0,
        "malformed saved source is not expected denial"
    );
    let before = s.clone();
    let _ = d.checks(&s);
    exact(&s, &before);
}
#[test]
fn goal_rejects_vacuous_unknown_and_oversized_before_any_candidate_operation() {
    let c = cases().remove(0);
    let mut d = c.definition.clone();
    let client = d.initial.inventory().clients[0].clone();
    d.requirements = vec![work::Requirement::NoCurrentMount {
        client: client.id,
        mountpoint: client.mountpoints[0].clone(),
    }];
    assert!(d.validate().is_err());
    let mut d = c.definition.clone();
    d.requirements.push(work::Requirement::CurrentFileText {
        server: "foreign".into(),
        path: "/not/supplied".into(),
        text: "".into(),
    });
    assert!(d.validate().is_err());
    let mut d = c.definition.clone();
    if let work::Requirement::CurrentAccess {
        expected: core::Expected::Read { text },
        ..
    } = &mut d.requirements[1]
    {
        *text = "x".repeat(core::MAX_FILE + 1);
    }
    assert!(d.validate().is_err());
    let mut d = c.definition.clone();
    d.fresh.clear();
    assert!(d.validate().is_err());
    let mut d = c.definition.clone();
    d.requirements.push(d.requirements[0].clone());
    assert!(d.validate().is_err());
}
#[test]
fn unsquashed_uid_zero_fault_never_satisfies_current_or_fresh_expected_denial() {
    let c = cases().remove(0);
    let (mut s, mut t) = solved(&c);
    let server = s.inventory().servers[0].id.clone();
    let client = s.inventory().clients[0].id.clone();
    let mountpoint = s.inventory().clients[0].mountpoints[1].clone();
    let root = s
        .inventory()
        .profiles
        .iter()
        .find(|p| p.id.ends_with("root"))
        .unwrap()
        .id
        .clone();
    let source = s.runtime().servers[0]
        .saved
        .replace("(ro)", "(ro,no_root_squash)");
    // Retire both old option bindings explicitly before this permitted policy repair.
    let existing = s
        .runtime()
        .mounts
        .iter()
        .map(|m| (m.client.clone(), m.specification.clone()))
        .collect::<Vec<_>>();
    for (cn, sp) in &existing {
        work::perform(
            &c.definition,
            &mut s,
            &mut t,
            &core::Action::Unmount {
                client: cn.clone(),
                mountpoint: sp.mountpoint.clone(),
            },
        )
        .unwrap();
    }
    run(
        &c.definition,
        &mut s,
        &mut t,
        &[
            core::Action::SetExportsDraft {
                server: server.clone(),
                text: source,
            },
            core::Action::SaveExportsDraft {
                server: server.clone(),
            },
            core::Action::ApplySavedExports { server },
        ],
    );
    for (cn, sp) in existing {
        work::perform(
            &c.definition,
            &mut s,
            &mut t,
            &core::Action::MountExplicit {
                client: cn,
                specification: sp,
            },
        )
        .unwrap();
    }
    let mut d = c.definition.clone();
    d.requirements.push(work::Requirement::CurrentAccess {
        request: core::Request::Read {
            client: client.clone(),
            mountpoint: mountpoint.clone(),
            profile: root.clone(),
            relative: "notice.txt".into(),
        },
        expected: core::Expected::Denied {
            stage: core::DenialStage::FilePermission,
        },
    });
    let mut root_trial = d.fresh[0].clone();
    if let core::Action::ReadExisting { profile, .. } = &mut root_trial.steps[1].action {
        *profile = root;
    }
    root_trial.steps[1].expected = core::Expected::Denied {
        stage: core::DenialStage::FilePermission,
    };
    d.fresh.push(root_trial);
    let rows = d.checks(&s);
    assert!(!rows[d.requirements.len()].0);
    assert!(!rows[1 + d.requirements.len() + d.fresh.len() - 1].0);
    let before = s.clone();
    d.checks(&s);
    exact(&s, &before);
}
#[test]
fn missing_search_is_repaired_by_numeric_selection_or_supported_mapping_not_chmod() {
    let c = cases().remove(1);
    let mut s = c.definition.initial.clone();
    let mut session = work::Session::default();
    let inv = s.inventory();
    let server = inv.servers[0].id.clone();
    let client = inv.clients[0].id.clone();
    let mp = inv.clients[0].mountpoints[1].clone();
    let root = inv
        .roots
        .iter()
        .find(|r| r.path.ends_with("mapped"))
        .unwrap()
        .path
        .clone();
    let writer = inv
        .profiles
        .iter()
        .find(|p| p.id.ends_with("writer"))
        .unwrap()
        .id
        .clone();
    let mapped = inv
        .profiles
        .iter()
        .find(|p| p.id.ends_with("mapped"))
        .unwrap()
        .id
        .clone();
    let address = inv.clients[0].address;
    let old_source = s.runtime().servers[0].saved.clone();
    let source = format!("{old_source}{root} {address}(rw,anongid=65534)\n");
    run(
        &c.definition,
        &mut s,
        &mut session,
        &[
            core::Action::SetExportsDraft {
                server: server.clone(),
                text: source,
            },
            core::Action::SaveExportsDraft {
                server: server.clone(),
            },
            core::Action::ApplySavedExports {
                server: server.clone(),
            },
        ],
    );
    let sp = core::MountSpec {
        server: server.clone(),
        root: root.clone(),
        mountpoint: mp.clone(),
        access: core::Access::Rw,
    };
    work::perform(
        &c.definition,
        &mut s,
        &mut session,
        &core::Action::MountExplicit {
            client: client.clone(),
            specification: sp.clone(),
        },
    )
    .unwrap();
    let request = |profile: String| core::Action::ReadExisting {
        client: client.clone(),
        mountpoint: mp.clone(),
        profile,
        relative: "notice.txt".into(),
    };
    let inventory = s.inventory().clone();
    assert!(matches!(
        work::perform(&c.definition, &mut s, &mut session, &request(writer)).unwrap(),
        core::Decision::Denied {
            stage: core::DenialStage::DirectorySearch,
            ..
        }
    ));
    assert!(matches!(
        work::perform(
            &c.definition,
            &mut s,
            &mut session,
            &request(mapped.clone())
        )
        .unwrap(),
        core::Decision::AllowedRead { .. }
    ));
    work::perform(
        &c.definition,
        &mut s,
        &mut session,
        &core::Action::Unmount {
            client: client.clone(),
            mountpoint: mp.clone(),
        },
    )
    .unwrap();
    run(
        &c.definition,
        &mut s,
        &mut session,
        &[
            core::Action::SetExportsDraft {
                server: server.clone(),
                text: format!("{old_source}{root} {address}(rw,anongid=777)\n"),
            },
            core::Action::SaveExportsDraft {
                server: server.clone(),
            },
            core::Action::ApplySavedExports {
                server: server.clone(),
            },
            core::Action::MountExplicit {
                client: client.clone(),
                specification: sp.clone(),
            },
        ],
    );
    assert!(matches!(
        work::perform(
            &c.definition,
            &mut s,
            &mut session,
            &request(mapped.clone())
        )
        .unwrap(),
        core::Decision::Denied {
            stage: core::DenialStage::DirectorySearch,
            ..
        }
    ));
    work::perform(
        &c.definition,
        &mut s,
        &mut session,
        &core::Action::Unmount {
            client: client.clone(),
            mountpoint: mp.clone(),
        },
    )
    .unwrap();
    run(
        &c.definition,
        &mut s,
        &mut session,
        &[
            core::Action::SetExportsDraft {
                server: server.clone(),
                text: format!("{old_source}{root} {address}(rw,anongid=65534)\n"),
            },
            core::Action::SaveExportsDraft {
                server: server.clone(),
            },
            core::Action::ApplySavedExports { server },
            core::Action::MountExplicit {
                client: client.clone(),
                specification: sp,
            },
        ],
    );
    assert!(matches!(
        work::perform(&c.definition, &mut s, &mut session, &request(mapped)).unwrap(),
        core::Decision::AllowedRead { .. }
    ));
    assert_eq!(s.inventory(), &inventory);
}
#[test]
fn strict_new_definition_and_session_raw_unknown_duplicates_and_n_plus_one() {
    let c = cases().remove(0);
    let raw = serde_json::to_vec(&c.definition).unwrap();
    assert_eq!(work::Definition::from_json(&raw).unwrap(), c.definition);
    assert!(work::Definition::from_json(&vec![b' '; work::MAX_DEFINITION + 1]).is_err());
    let mut v = serde_json::to_value(&c.definition).unwrap();
    v["extra"] = json!(1);
    assert!(serde_json::from_value::<work::Definition>(v).is_err());
    let raw = String::from_utf8(raw).unwrap();
    assert!(
        work::Definition::from_json(
            format!(
                "{{\"initial\":{},{}",
                serde_json::to_string(&c.definition.initial).unwrap(),
                &raw[1..]
            )
            .as_bytes()
        )
        .is_err()
    );
    let mut v = serde_json::to_value(&c.definition).unwrap();
    v["requirements"] = json!([]);
    for _ in 0..32 {
        v["requirements"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::to_value(&c.definition.requirements[0]).unwrap());
    }
    v["requirements"]
        .as_array_mut()
        .unwrap()
        .push(json!({"poison":[]}));
    let e = serde_json::from_value::<work::Definition>(v)
        .unwrap_err()
        .to_string();
    assert!(e.contains("collection limit before typed body"));
    let t = work::Session::default();
    let raw = serde_json::to_string(&t).unwrap();
    assert_eq!(work::Session::from_json(raw.as_bytes()).unwrap(), t);
    assert!(work::Session::from_json(format!("{{\"version\":1,{}", &raw[1..]).as_bytes()).is_err());
    let mut v = serde_json::to_value(t).unwrap();
    v["foreign"] = json!(true);
    assert!(serde_json::from_value::<work::Session>(v).is_err());
    assert!(work::Session::from_json(&vec![b' '; work::MAX_SESSION + 1]).is_err());
}
#[test]
fn session_nul_roundtrip_parity_sixteen_rows_and_escape_heavy_aggregate_are_closed() {
    let mut t = work::Session {
        entries: (0..16)
            .map(|n| work::Entry {
                operation: format!("Observed {n}"),
                detail: "x".repeat(3900),
            })
            .collect(),
        ..Default::default()
    };
    assert!(t.valid());
    assert_eq!(
        work::Session::from_json(&serde_json::to_vec(&t).unwrap()).unwrap(),
        t
    );
    t.entries.push(work::Entry {
        operation: "overflow".into(),
        detail: "".into(),
    });
    assert!(!t.valid());
    t.entries.pop();
    for e in &mut t.entries {
        e.detail = "\u{1}".repeat(4096);
    }
    assert!(!t.valid());
    let t = work::Session {
        relative: "bad\0query".into(),
        ..Default::default()
    };
    assert!(!t.valid());
    assert!(work::Session::from_json(&serde_json::to_vec(&t).unwrap()).is_err());
    let mut t = work::Session::default();
    t.entries.push(work::Entry {
        operation: "Read".into(),
        detail: "bad\0output".into(),
    });
    assert!(!t.valid());
    assert!(work::Session::from_json(&serde_json::to_vec(&t).unwrap()).is_err());
}
#[test]
fn foreign_context_active_mount_conflict_and_oversized_actions_preserve_full_callers() {
    let c = cases().remove(2);
    let mut s = c.definition.initial.clone();
    let mut t = work::Session::default();
    let before = s.clone();
    let old = t.clone();
    let server = s.inventory().servers[0].id.clone();
    let raw = c
        .reference
        .iter()
        .find_map(|a| {
            if let core::Action::SetExportsDraft { text, .. } = a {
                Some(text.clone())
            } else {
                None
            }
        })
        .unwrap();
    run(
        &c.definition,
        &mut s,
        &mut t,
        &[
            core::Action::SetExportsDraft {
                server: server.clone(),
                text: raw,
            },
            core::Action::SaveExportsDraft {
                server: server.clone(),
            },
        ],
    );
    let saved = s.clone();
    let session = t.clone();
    assert!(
        work::perform(
            &c.definition,
            &mut s,
            &mut t,
            &core::Action::ApplySavedExports {
                server: server.clone()
            }
        )
        .is_err()
    );
    exact(&s, &saved);
    exact(&t, &session);
    assert!(
        work::perform(
            &c.definition,
            &mut s,
            &mut t,
            &core::Action::SetExportsDraft {
                server,
                text: "x".repeat(core::MAX_DOCUMENT + 1)
            }
        )
        .is_err()
    );
    exact(&s, &saved);
    exact(&t, &session);
    let other = cases().remove(0);
    assert!(work::perform(&other.definition, &mut s, &mut t, &c.reference[0]).is_err());
    exact(&s, &saved);
    exact(&t, &session);
    assert_eq!(before.inventory(), s.inventory());
    assert_eq!(old.entries.len(), 0);
}
#[test]
fn full_inventory_and_positive_rows_cannot_be_replaced_by_fixture_name_or_display_history() {
    let c = cases().remove(0);
    let (s, mut t) = solved(&c);
    t.entries.push(work::Entry {
        operation: "Past success".into(),
        detail: "All checks passed".into(),
    });
    assert!(
        !c.definition
            .checks(&c.definition.initial)
            .iter()
            .all(|r| r.0)
    );
    let mut i = s.inventory().clone();
    i.profiles[0].uid += 10;
    let foreign = core::State::new(i, s.runtime().clone()).unwrap();
    assert_eq!(foreign.inventory().fixture, s.inventory().fixture);
    assert!(!c.definition.same_assignment(&foreign));
    assert!(!c.definition.checks(&foreign)[0].0);
    assert!(t.valid_for(&s));
}
#[test]
fn escaped_expected_frontier_and_fresh_evaluation_bound_reject_without_repairing_current() {
    let c = cases().remove(0);
    let mut d = c.definition.clone();
    let server = d.initial.inventory().servers[0].id.clone();
    let path = d.initial.inventory().roots[0].files[0].path.clone();
    for n in 0..26 {
        d.requirements.push(work::Requirement::CurrentFileText {
            server: server.clone(),
            path: path.clone(),
            text: format!("{n}{}", "\u{1}".repeat(4090)),
        });
    }
    assert!(d.requirements.len() <= 32);
    assert!(d.validate().is_err());
    let before = c.definition.initial.clone();
    assert!(!d.checks(&before)[0].0);
    exact(&before, &c.definition.initial);
}

#[test]
fn committed_exports_ignore_root_and_disjoint_clause_order_but_not_options() {
    let c = cases().remove(0);
    let (mut s, mut session) = solved(&c);
    let server = s.inventory().servers[0].id.clone();
    let root = s.inventory().roots[1].path.clone();
    let old = s.inventory().clients[0].address;
    let new = s.inventory().clients[1].address;
    let archive = s.runtime().servers[0]
        .saved
        .lines()
        .find(|l| l.contains("archive"))
        .unwrap()
        .to_owned();
    let source = format!("{root} {old}(ro) {new}(ro)\n{archive}\n");
    run(
        &c.definition,
        &mut s,
        &mut session,
        &[
            core::Action::SetExportsDraft {
                server: server.clone(),
                text: source,
            },
            core::Action::SaveExportsDraft {
                server: server.clone(),
            },
            core::Action::ApplySavedExports {
                server: server.clone(),
            },
        ],
    );
    assert!(c.definition.checks(&s).iter().all(|r| r.0));
    let applied = s.runtime().servers[0].applied.clone();
    let source = format!("{archive}\n{root} {new}(ro) {old}(ro)\n");
    run(
        &c.definition,
        &mut s,
        &mut session,
        &[
            core::Action::SetExportsDraft {
                server: server.clone(),
                text: source,
            },
            core::Action::SaveExportsDraft {
                server: server.clone(),
            },
        ],
    );
    assert_eq!(s.runtime().servers[0].applied, applied);
    assert!(
        c.definition.checks(&s).iter().all(|r| r.0),
        "semantic policy equality allows root and clause reordering without Apply"
    );
    run(
        &c.definition,
        &mut s,
        &mut session,
        &[
            core::Action::SetExportsDraft {
                server: server.clone(),
                text: format!("{archive}\n{root} {new}(ro) {old}(rw)\n"),
            },
            core::Action::SaveExportsDraft { server },
        ],
    );
    let rows = c.definition.checks(&s);
    assert!(
        rows[1].0 && rows[2].0,
        "actual current mount/read remain healthy"
    );
    assert!(
        !rows[3].0,
        "saved changed option needs Apply and cannot be committed"
    );
}
