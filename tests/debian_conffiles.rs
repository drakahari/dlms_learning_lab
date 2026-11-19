//! Source-derived logical conffile vectors, NOT an executed dpkg comparison.
use dlms_learning_lab::debian_packages::*;
use serde_json::Value;
const CFG: &str = "/etc/museum/report.conf";
fn fixture() -> State {
    let mut value: Value =
        serde_json::from_str(include_str!("fixtures/debian-packages-declared.json")).unwrap();
    let state = &mut value["state"];
    state["runtime"]["cached"] = "current".into();
    for suffix in [".dpkg-old", ".dpkg-dist", ".dpkg-new", ".dpkg-tmp"] {
        state["catalog"]["extra_paths"]
            .as_array_mut()
            .unwrap()
            .push(format!("{CFG}{suffix}").into());
    }
    serde_json::from_value(state.clone()).unwrap()
}
fn change(s: &State, f: impl FnOnce(&mut Catalog, &mut Runtime)) -> State {
    let (mut c, mut r) = (s.catalog().clone(), s.runtime().clone());
    f(&mut c, &mut r);
    State::new(c, r).unwrap()
}
fn action(s: &State, decision: Option<ConffileDecision>) -> Action {
    let mut upgrade = s.review_conffiles("museum-report").unwrap();
    upgrade.files[0].decision = decision;
    Action::UpgradeConffiles { upgrade }
}
fn reject(s: &mut State, a: &Action, code: ErrorCode) {
    let before = serde_json::to_vec(s).unwrap();
    assert_eq!(s.plan(a).unwrap_err().code, code);
    assert_eq!(s.apply(a).unwrap_err().code, code);
    assert_eq!(serde_json::to_vec(s).unwrap(), before);
}
#[test]
fn explicit_choices_update_payload_and_baseline_but_preserve_correct_live_and_sidecar_bytes() {
    for decision in [ConffileDecision::KeepLocal, ConffileDecision::InstallNew] {
        let mut s = fixture();
        let old = s.clone();
        let a = action(&s, Some(decision));
        let preview = s.plan(&a).unwrap();
        assert_eq!(s, old);
        let applied = s.apply(&a).unwrap();
        assert_eq!(
            serde_json::to_value(preview).unwrap(),
            serde_json::to_value(applied).unwrap()
        );
        assert_eq!(s.runtime().records["museum-report"].archive, "report-2");
        assert_eq!(s.runtime().records["museum-lib"].archive, "lib-2");
        assert_eq!(s.runtime().files["/usr/bin/museum-report"], "report two\n");
        assert_eq!(
            s.runtime().files["/work/note.txt"],
            old.runtime().files["/work/note.txt"]
        );
        let (live, suffix, saved) = match decision {
            ConffileDecision::KeepLocal => ("style=local\n", ".dpkg-dist", "style=bright\n"),
            ConffileDecision::InstallNew => ("style=bright\n", ".dpkg-old", "style=local\n"),
        };
        assert_eq!(s.runtime().files[CFG], live);
        assert_eq!(s.runtime().files[&format!("{CFG}{suffix}")], saved);
        assert!(
            s.recorded_owner(&format!("{CFG}{suffix}"))
                .unwrap()
                .is_none()
        );
        assert!(!s.runtime().files.contains_key(&format!(
            "{CFG}{}",
            if suffix == ".dpkg-old" {
                ".dpkg-dist"
            } else {
                ".dpkg-old"
            }
        )));
        let round: State = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
        assert_eq!(s, round);
    }
}
#[test]
fn automatic_comparisons_and_empty_text_are_distinct_from_missing_files() {
    for (old, local, new, want, disposition) in [
        (
            "same",
            "same",
            "same",
            "same",
            ConffileDisposition::MatchesIncoming,
        ),
        (
            "old",
            "new",
            "new",
            "new",
            ConffileDisposition::MatchesIncoming,
        ),
        (
            "old",
            "old",
            "new",
            "new",
            ConffileDisposition::UpdateUnmodified,
        ),
        (
            "old",
            "site",
            "old",
            "site",
            ConffileDisposition::KeepLocalOnly,
        ),
    ] {
        assert_eq!(conffile_disposition(old, local, new), disposition);
        let mut s = change(&fixture(), |c, r| {
            c.archives
                .get_mut("report-1")
                .unwrap()
                .files
                .get_mut(CFG)
                .unwrap()
                .contents = old.into();
            c.archives
                .get_mut("report-2")
                .unwrap()
                .files
                .get_mut(CFG)
                .unwrap()
                .contents = new.into();
            r.files.insert(CFG.into(), local.into());
        });
        let a = action(&s, None);
        s.apply(&a).unwrap();
        assert_eq!(s.runtime().files[CFG], want);
        assert!(!s.runtime().files.contains_key(&format!("{CFG}.dpkg-old")));
        assert!(!s.runtime().files.contains_key(&format!("{CFG}.dpkg-dist")));
    }
    for decision in [ConffileDecision::KeepLocal, ConffileDecision::InstallNew] {
        let mut s = change(&fixture(), |_, r| {
            r.files.insert(CFG.into(), String::new());
        });
        let a = action(&s, Some(decision));
        s.apply(&a).unwrap();
        assert!(s.runtime().files.contains_key(CFG));
        let key = if decision == ConffileDecision::KeepLocal {
            CFG.to_owned()
        } else {
            format!("{CFG}.dpkg-old")
        };
        assert_eq!(s.runtime().files[&key], "");
    }
    let missing = change(&fixture(), |_, r| {
        r.files.remove(CFG);
    });
    assert_eq!(
        missing.review_conffiles("museum-report").unwrap_err().code,
        ErrorCode::Unsupported
    );
}
#[test]
fn legacy_install_retains_its_exact_refusal_including_already_matching_local_content() {
    for local in ["style=local\n", "style=bright\n"] {
        let mut s = change(&fixture(), |_, r| {
            r.files.insert(CFG.into(), local.into());
        });
        let a = Action::Install {
            packages: vec![Request {
                name: "museum-report".into(),
                version: None,
            }],
        };
        reject(&mut s, &a, ErrorCode::Conffile);
    }
}
#[test]
fn missing_unnecessary_duplicate_foreign_or_stale_decisions_cannot_publish_any_changes() {
    let mut s = fixture();
    let none = action(&s, None);
    reject(&mut s, &none, ErrorCode::Conffile);
    let a = action(&s, Some(ConffileDecision::KeepLocal));
    let Action::UpgradeConffiles { upgrade } = a.clone() else {
        panic!()
    };
    for kind in 0..5 {
        let mut u = upgrade.clone();
        match kind {
            0 => u.expected_archive = "report-2".into(),
            1 => u.incoming_archive = "report-1".into(),
            2 => u.files[0].expected_local = "different".into(),
            3 => u.files.push(u.files[0].clone()),
            _ => u.files[0].path = "/work/note.txt".into(),
        };
        let a = Action::UpgradeConffiles { upgrade: u };
        let before = s.clone();
        assert!(s.plan(&a).is_err());
        assert!(s.apply(&a).is_err());
        assert_eq!(s, before);
    }
    let mut automatic = change(&s, |_, r| {
        r.files.insert(CFG.into(), "style=plain\n".into());
    });
    let a = action(&automatic, Some(ConffileDecision::InstallNew));
    reject(&mut automatic, &a, ErrorCode::Conffile);
    let mut changed = change(&s, |_, r| {
        r.files.insert(CFG.into(), "after review\n".into());
    });
    reject(&mut changed, &a_for(&upgrade), ErrorCode::Conffile);
    s.apply(&a_for(&upgrade)).unwrap();
    reject(&mut s, &a_for(&upgrade), ErrorCode::Conffile);
}
fn a_for(u: &ConffileUpgrade) -> Action {
    Action::UpgradeConffiles { upgrade: u.clone() }
}
#[test]
fn each_existing_neighbor_and_any_manifest_or_missing_admission_refuses_before_mutation() {
    for suffix in [".dpkg-old", ".dpkg-dist", ".dpkg-new", ".dpkg-tmp"] {
        let mut s = change(&fixture(), |_, r| {
            r.files.insert(format!("{CFG}{suffix}"), String::new());
        });
        let original = s.clone();
        assert_eq!(
            s.review_conffiles("museum-report").unwrap_err().code,
            ErrorCode::Collision
        );
        let a = action(&fixture(), Some(ConffileDecision::KeepLocal));
        reject(&mut s, &a, ErrorCode::Collision);
        assert_eq!(s, original);
    }
    let mut collision = change(&fixture(), |c, _| {
        c.extra_paths.remove(&format!("{CFG}.dpkg-old"));
        c.archives.get_mut("common-1").unwrap().files.insert(
            format!("{CFG}.dpkg-old"),
            Payload {
                contents: "protected future payload".into(),
                conffile: false,
            },
        );
    });
    let a = action(&fixture(), Some(ConffileDecision::KeepLocal));
    reject(&mut collision, &a, ErrorCode::Collision);
    let mut missing = change(&fixture(), |c, _| {
        c.extra_paths.remove(&format!("{CFG}.dpkg-dist"));
    });
    reject(&mut missing, &a, ErrorCode::Unsupported);
}
#[test]
fn current_resolver_architecture_and_dependencies_are_never_bypassed() {
    let mut s = fixture();
    let a = action(&s, Some(ConffileDecision::InstallNew));
    let mut wrong = change(&s, |c, _| {
        c.archives.get_mut("report-2").unwrap().architecture = Architecture::Arm64;
    });
    reject(&mut wrong, &a, ErrorCode::Architecture);
    let mut dependencies = change(&s, |c, _| {
        c.archives.get_mut("report-2").unwrap().depends[0].constraint = Some(Constraint {
            relation: Relation::Ge,
            version: Version::parse("99").unwrap(),
        });
    });
    reject(&mut dependencies, &a, ErrorCode::Dependency);
    let mut old = change(&s, |_, r| r.cached = Some("old".into()));
    reject(&mut old, &a, ErrorCode::Conffile);
    let mut absent = change(&s, |_, r| r.cached = None);
    assert!(absent.plan(&a).is_err());
    let before = absent.clone();
    assert!(absent.apply(&a).is_err());
    assert_eq!(absent, before);
    // A changed conffile path set is not an adoption shortcut.
    let mut paths = change(&s, |c, _| {
        let cfg = c
            .archives
            .get_mut("report-2")
            .unwrap()
            .files
            .remove(CFG)
            .unwrap();
        c.archives
            .get_mut("report-2")
            .unwrap()
            .files
            .insert("/etc/museum/other.conf".into(), cfg);
    });
    reject(&mut paths, &a, ErrorCode::Unsupported);
    assert_eq!(s, fixture());
    let _ = &mut s;
}
#[test]
fn action_decoding_bounds_and_closed_shapes_reject_without_host_execution() {
    let a = action(&fixture(), Some(ConffileDecision::KeepLocal));
    let bytes = serde_json::to_vec(&a).unwrap();
    assert_eq!(Action::decode(&bytes).unwrap(), a);
    for change in 0..4 {
        let mut v = serde_json::to_value(&a).unwrap();
        match change {
            0 => v["upgrade"]["files"][0]["decision"] = "UseHost".into(),
            1 => v["upgrade"]["files"][0]["expected_local"] = "x".repeat(2049).into(),
            2 => v["upgrade"]["files"][0]["path"] = "/etc/../personal".into(),
            _ => v["upgrade"]["source"] = "/home/private".into(),
        };
        assert!(Action::decode(&serde_json::to_vec(&v).unwrap()).is_err());
    }
    assert!(Action::decode(&vec![b' '; 16385]).is_err());
}
#[test]
fn four_row_mixed_review_and_late_errors_preserve_every_earlier_file() {
    let base = change(&fixture(), |c, r| {
        for (path, old, live, new) in [
            ("/etc/museum/default.conf", "old", "old", "new"),
            ("/etc/museum/site.conf", "old", "site", "old"),
            ("/etc/museum/ready.conf", "old", "new", "new"),
        ] {
            c.archives.get_mut("report-1").unwrap().files.insert(
                path.into(),
                Payload {
                    contents: old.into(),
                    conffile: true,
                },
            );
            c.archives.get_mut("report-2").unwrap().files.insert(
                path.into(),
                Payload {
                    contents: new.into(),
                    conffile: true,
                },
            );
            r.files.insert(path.into(), live.into());
            for suffix in [".dpkg-old", ".dpkg-dist", ".dpkg-new", ".dpkg-tmp"] {
                c.extra_paths.insert(format!("{path}{suffix}"));
            }
        }
    });
    let mut review = base.review_conffiles("museum-report").unwrap();
    assert_eq!(review.files.len(), 4);
    for row in &mut review.files {
        if row.path == CFG {
            row.decision = Some(ConffileDecision::KeepLocal);
        }
    }
    let mut next = base.clone();
    next.apply(&a_for(&review)).unwrap();
    for (p, text) in [
        (CFG, "style=local\n"),
        ("/etc/museum/default.conf", "new"),
        ("/etc/museum/site.conf", "site"),
        ("/etc/museum/ready.conf", "new"),
    ] {
        assert_eq!(next.runtime().files[p], text);
    }
    assert_eq!(
        next.runtime().files[&format!("{CFG}.dpkg-dist")],
        "style=bright\n"
    );
    assert_eq!(next.runtime().files.len(), base.runtime().files.len() + 1);
    let mut bad = review.clone();
    assert_eq!(bad.files.last().unwrap().path, "/etc/museum/site.conf");
    bad.files.last_mut().unwrap().decision = Some(ConffileDecision::InstallNew);
    let mut unchanged = base.clone();
    reject(&mut unchanged, &a_for(&bad), ErrorCode::Conffile);
    let mut collision = change(&base, |_, r| {
        r.files.insert(
            "/etc/museum/site.conf.dpkg-tmp".into(),
            "keep late neighbor".into(),
        );
    });
    reject(&mut collision, &a_for(&review), ErrorCode::Collision);
}
fn ballast() -> State {
    change(&fixture(), |c, r| {
        let mut at = 0;
        let mut archives = 0;
        let mut rows: usize = c.archives.values().map(|a| a.files.len()).sum();
        while rows < 256 {
            let id = format!("padding-{archives}");
            let mut archive = c.archives["common-1"].clone();
            archive.name = id.clone();
            archive.files.clear();
            for _ in 0..32 {
                if rows == 256 {
                    break;
                }
                let p = format!("/work/padding-{at}");
                archive.files.insert(
                    p.clone(),
                    Payload {
                        contents: String::new(),
                        conffile: false,
                    },
                );
                r.files.insert(p, String::new());
                at += 1;
                rows += 1;
            }
            c.archives.insert(id, archive);
            archives += 1;
        }
    })
}
#[test]
fn finite_universe_admission_and_text_preflight_never_publish_partial_upgrade() {
    let full = ballast();
    assert_eq!(
        full.catalog()
            .archives
            .values()
            .map(|a| a.files.len())
            .sum::<usize>(),
        256
    );
    assert!(full.runtime().files.len() < MAX_FILES);
    let before = full.clone();
    let mut catalog = full.catalog().clone();
    catalog.archives.get_mut("common-1").unwrap().files.insert(
        "/work/too-many".into(),
        Payload {
            contents: String::new(),
            conffile: false,
        },
    );
    assert_eq!(
        State::new(catalog, full.runtime().clone())
            .unwrap_err()
            .code,
        ErrorCode::Limit
    );
    assert_eq!(full, before);
    let mut runtime = full.runtime().clone();
    while runtime.files.len() <= MAX_FILES {
        let n = runtime.files.len();
        runtime
            .files
            .insert(format!("/work/excess-{n}"), String::new());
    }
    assert_eq!(
        State::new(full.catalog().clone(), runtime)
            .unwrap_err()
            .code,
        ErrorCode::Limit
    );
    let mut text = change(&full, |_, r| {
        r.files
            .insert("/usr/bin/museum-report".into(), "report two\n".into());
        let used: usize = r.files.values().map(String::len).sum();
        let mut left = MAX_TEXT - 1 - used;
        for (p, t) in &mut r.files {
            if p.starts_with("/work/padding-") {
                let n = left.min(2048);
                *t = "x".repeat(n);
                left -= n;
            }
        }
        assert_eq!(left, 0);
    });
    assert_eq!(
        text.runtime()
            .files
            .values()
            .map(String::len)
            .sum::<usize>(),
        MAX_TEXT - 1
    );
    let a = action(&text, Some(ConffileDecision::KeepLocal));
    reject(&mut text, &a, ErrorCode::Limit);
}
#[test]
fn typed_action_counts_the_tagged_wrapper_in_the_existing16kib_limit() {
    let mut u = fixture().review_conffiles("museum-report").unwrap();
    u.files = (0..4)
        .map(|n| ConffileSelection {
            path: format!("/etc/config-{n}"),
            expected_local: if n < 3 {
                "\n".repeat(2048)
            } else {
                String::new()
            },
            decision: None,
        })
        .collect();
    let base = serde_json::to_vec(&u).unwrap().len();
    let n = (16384 - base) / 2;
    assert!(n <= 2048);
    u.files[3].expected_local = "\n".repeat(n);
    assert!(u.valid_shape());
    let a = a_for(&u);
    assert!(serde_json::to_vec(&a).unwrap().len() > 16384);
    assert!(Action::decode(&serde_json::to_vec(&a).unwrap()).is_err());
    let mut s = fixture();
    reject(&mut s, &a, ErrorCode::Limit);
}
fn definition(
    s: &State,
    decision: ConffileDecision,
) -> dlms_learning_lab::debian_packages_workspace::Definition {
    use dlms_learning_lab::debian_packages_workspace as w;
    let d = w::Definition {
        initial: s.clone(),
        requirements: vec![
            w::Requirement::ConfigurationUpgrade {
                outcome: w::ConfigurationUpgradeGoal {
                    package: "museum-report".into(),
                    incoming_archive: "report-2".into(),
                    choices: vec![w::ConfigurationTarget {
                        path: CFG.into(),
                        decision,
                    }],
                },
            },
            w::Requirement::FilePreserved {
                path: "/work/note.txt".into(),
            },
        ],
    };
    d.validate().unwrap();
    d
}
#[test]
fn source_goal_grades_realized_live_and_backup_files_without_awarding_a_choice() {
    use dlms_learning_lab::debian_packages_workspace as w;
    for decision in [ConffileDecision::KeepLocal, ConffileDecision::InstallNew] {
        let initial = fixture();
        let d = definition(&initial, decision);
        assert!(d.checks(&initial).iter().any(|r| !r.0));
        let a = action(&initial, Some(decision));
        let mut s = initial.clone();
        w::apply(&d, &mut s, &a).unwrap();
        assert!(d.checks(&s).iter().all(|r| r.0), "{:?}", d.checks(&s));
        // A saved enum or a correct version cannot compensate for damaged files.
        let side = format!(
            "{CFG}{}",
            if decision == ConffileDecision::KeepLocal {
                ".dpkg-dist"
            } else {
                ".dpkg-old"
            }
        );
        let bad = change(&s, |_, r| {
            r.files.insert(side.clone(), "wrong recovery text".into());
        });
        let checks = d.checks(&bad);
        assert!(!checks[0].0 && checks[0].1.contains(&side));
        let bad = change(&s, |_, r| {
            r.files.insert(CFG.into(), "wrong live text".into());
        });
        assert!(!d.checks(&bad)[0].0);
        let foreign = change(&s, |c, _| {
            c.archives
                .get_mut("report-1")
                .unwrap()
                .files
                .get_mut(CFG)
                .unwrap()
                .contents = "changed source under same ID".into();
        });
        assert!(!d.same_assignment(&foreign));
        assert!(d.checks(&foreign).iter().any(|r| !r.0));
        let raw = serde_json::to_vec(&d).unwrap();
        let loaded: w::Definition = serde_json::from_slice(&raw).unwrap();
        assert_eq!(loaded, d);
    }
}
#[test]
fn new_activity_admission_and_sidecar_lifecycle_are_explicit_and_preserve_legacy_scope() {
    use dlms_learning_lab::debian_packages_workspace as w;
    let initial = fixture();
    let a = action(&initial, Some(ConffileDecision::KeepLocal));
    let ordinary = w::Definition {
        initial: initial.clone(),
        requirements: vec![w::Requirement::Installed {
            package: "museum-report".into(),
            constraint: None,
        }],
    };
    ordinary.validate().unwrap();
    let mut before = initial.clone();
    assert!(
        w::apply(&ordinary, &mut before, &a)
            .unwrap_err()
            .contains("does not admit")
    );
    assert_eq!(before, initial);
    let d = definition(&initial, ConffileDecision::KeepLocal);
    let mut s = initial.clone();
    w::apply(&d, &mut s, &a).unwrap();
    for action in [
        Action::Remove {
            name: "museum-report".into(),
        },
        Action::Purge {
            name: "museum-report".into(),
        },
    ] {
        let before = s.clone();
        assert!(
            w::apply(&d, &mut s, &action)
                .unwrap_err()
                .contains("outside this activity")
        );
        assert_eq!(s, before);
    }
    let wrong = definition(&initial, ConffileDecision::InstallNew);
    assert!(!wrong.checks(&s)[0].0);
}
#[test]
fn optional_review_drafts_preserve_legacy_wire_format_and_reopen_without_execution() {
    use dlms_learning_lab::debian_packages_workspace as w;
    let initial = fixture();
    let definition = definition(&initial, ConffileDecision::KeepLocal);
    let mut draft = w::Draft::new(&initial).unwrap();
    let legacy = serde_json::to_value(&draft).unwrap();
    assert!(legacy.get("conffile_upgrade").is_none());
    assert_eq!(
        serde_json::from_value::<w::Draft>(legacy.clone()).unwrap(),
        draft
    );
    w::prepare_configuration(&definition, &initial, &mut draft, "museum-report").unwrap();
    assert_eq!(draft.operation, w::Operation::UpgradeConffiles);
    assert!(
        draft
            .conffile_upgrade
            .as_ref()
            .unwrap()
            .files
            .iter()
            .all(|r| r.decision.is_none())
    );
    assert_eq!(initial, fixture());
    assert!(!definition.checks(&initial)[0].0);
    draft.conffile_upgrade.as_mut().unwrap().files[0].decision = Some(ConffileDecision::KeepLocal);
    let raw = serde_json::to_vec(&draft).unwrap();
    let loaded: w::Draft = serde_json::from_slice(&raw).unwrap();
    assert_eq!(draft, loaded);
    assert_eq!(initial, fixture());
    assert!(loaded.valid_for(&initial));
    let changed = change(&initial, |_, r| {
        r.files.insert(CFG.into(), "new local edit\n".into());
    });
    assert!(
        loaded.valid_for(&changed),
        "Keep a stale but bounded draft for explanation/retry"
    );
    assert!(
        changed
            .conffile_review_current(loaded.conffile_upgrade.as_ref().unwrap())
            .is_err()
    );
    let mut unchanged = changed.clone();
    assert!(w::apply(&definition, &mut unchanged, &loaded.action().unwrap()).is_err());
    assert_eq!(unchanged, changed);
    let foreign = change(&initial, |c, _| {
        c.archives
            .get_mut("report-1")
            .unwrap()
            .files
            .get_mut(CFG)
            .unwrap()
            .contents = "different catalog, same IDs".into();
    });
    assert!(!loaded.valid_for(&foreign));
    let mut draft = loaded.clone();
    assert!(w::prepare_configuration(&definition, &foreign, &mut draft, "museum-report").is_err());
    assert_eq!(draft, loaded);
    let mut unknown = serde_json::to_value(&loaded).unwrap();
    unknown["conffile_upgrade"]["expected_archive"] = "unknown".into();
    assert!(serde_json::from_value::<w::Draft>(unknown).is_err());
}
#[test]
fn choosing_a_real_conflict_is_a_source_bound_saved_draft_not_an_applied_upgrade() {
    use dlms_learning_lab::debian_packages_workspace as w;
    let s = fixture();
    let d = definition(&s, ConffileDecision::KeepLocal);
    let mut draft = w::Draft::new(&s).unwrap();
    w::prepare_configuration(&d, &s, &mut draft, "museum-report").unwrap();
    w::choose_configuration(&d, &s, &mut draft, CFG, Some(ConffileDecision::KeepLocal)).unwrap();
    assert_eq!(s, fixture());
    assert_eq!(
        draft.conffile_upgrade.as_ref().unwrap().files[0].decision,
        Some(ConffileDecision::KeepLocal)
    );
    assert!(!d.checks(&s)[0].0);
    let original = draft.clone();
    assert!(
        w::choose_configuration(
            &d,
            &s,
            &mut draft,
            "/work/note.txt",
            Some(ConffileDecision::InstallNew)
        )
        .is_err()
    );
    assert_eq!(draft, original);
    let changed = change(&s, |_, r| {
        r.files.insert(CFG.into(), "edited since review\n".into());
    });
    assert!(w::choose_configuration(&d, &changed, &mut draft, CFG, None).is_err());
    assert_eq!(draft, original);
    w::choose_configuration(&d, &s, &mut draft, CFG, None).unwrap();
    assert_eq!(
        draft.conffile_upgrade.as_ref().unwrap().files[0].decision,
        None
    );
    let mut copy = s.clone();
    assert!(w::apply(&d, &mut copy, &draft.action().unwrap()).is_err());
    assert_eq!(s, copy);
    let mut duplicate = d.clone();
    duplicate.requirements.push(d.requirements[0].clone());
    assert!(duplicate.validate().is_err());
}
