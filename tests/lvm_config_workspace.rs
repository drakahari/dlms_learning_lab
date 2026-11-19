//! Typed capability proof only; catalog/source/native admission stays closed.
use dlms_learning_lab::{lvm, lvm_workspace::config_scan as w};
use serde_json::{Value, json};
fn config(flag: bool, rules: &[&str]) -> String {
    format!(
        "devices {{ use_devicesfile = {} filter = [{}] }}",
        u8::from(flag),
        rules
            .iter()
            .map(|s| serde_json::to_string(s).unwrap())
            .collect::<Vec<_>>()
            .join(", ")
    )
}
fn selection(paths: &[&str]) -> w::Selection {
    w::Selection {
        entries: paths.iter().map(|p| w::DeviceName((*p).into())).collect(),
    }
}
fn context(file: bool) -> w::Context {
    let raw: Value =
        serde_json::from_str(include_str!("../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    let mut inventory: lvm::State =
        serde_json::from_value(raw["scenarios"][0]["goal"]["definition"]["initial"].clone())
            .unwrap();
    inventory.devices.insert(
        "/dev/noise".into(),
        lvm::Device {
            bytes: 8 * lvm::MIB,
            role: lvm::DeviceRole::Foreign,
            pv_bytes: None,
            group: String::new(),
        },
    );
    let first = w::Case {
        label: "Current supplied inventory".into(),
        expected: inventory
            .devices
            .keys()
            .map(|p| w::Expected {
                path: p.clone(),
                allowed: p == "/dev/teama",
            })
            .collect(),
        inventory: inventory.clone(),
    };
    inventory.devices.insert(
        "/dev/teamb".into(),
        lvm::Device {
            bytes: 8 * lvm::MIB,
            role: lvm::DeviceRole::PhysicalVolume,
            pv_bytes: None,
            group: String::new(),
        },
    );
    let second = w::Case {
        label: "Provided arrival check".into(),
        expected: inventory
            .devices
            .keys()
            .map(|p| w::Expected {
                path: p.clone(),
                allowed: p.starts_with("/dev/team"),
            })
            .collect(),
        inventory,
    };
    w::Context {
        initial_config: w::Text(config(file, &["r|.*|", "a|^/dev/team[a-z]$|"])),
        initial_file: file.then(|| selection(&["/dev/reservea"])),
        selection_editable: file,
        required_mode: Some(if file {
            w::RequiredMode::DevicesFile
        } else {
            w::RequiredMode::Filters
        }),
        cases: vec![first, second],
    }
}
fn passed(l: &w::Live, c: &w::Context) -> bool {
    l.checks(c).iter().all(|r| r.0)
}
fn fix(l: &mut w::Live, c: &w::Context, pattern: &str) {
    l.edit(c, config(false, &[pattern, "r|.*|"])).unwrap();
    l.save_config(c).unwrap();
    l.run(c).unwrap();
}
#[test]
fn actual_multiple_cases_reject_one_inventory_hardcoding_and_accept_equivalent_filters() {
    let c = context(false);
    c.shape().unwrap();
    let mut l = w::Live::new(&c).unwrap();
    assert!(!passed(&l, &c));
    fix(&mut l, &c, "a|^/dev/teama$|");
    assert!(!passed(&l, &c));
    let rows = l.checks(&c);
    assert!(rows.iter().any(|r| !r.0
        && r.1.contains("Provided arrival")
        && r.1.contains("/dev/teamb")
        && r.1.contains("rule2 rejects first")));
    for p in [
        "a|^/dev/team[a-z]$|",
        "a#^/dev/(teama|teamb)$#",
        "a|^/dev/team[ab]$|",
    ] {
        fix(&mut l, &c, p);
        assert!(passed(&l, &c), "{:?}", l.checks(&c));
    }
    let reports = l.last_results(&c).unwrap().unwrap().unwrap();
    assert_eq!(
        reports[1]
            .records
            .iter()
            .find(|r| r.path == "/dev/teamb")
            .unwrap()
            .metadata,
        Some(lvm::config_scan::Metadata::PhysicalVolume { group: None })
    );
}
#[test]
fn drafts_are_inert_save_and_run_are_distinct_and_edits_clear_current_success() {
    let c = context(false);
    let mut l = w::Live::new(&c).unwrap();
    fix(&mut l, &c, "a|^/dev/team[a-z]$|");
    assert!(passed(&l, &c));
    let saved = l.saved.clone();
    let old = l.last_results(&c).unwrap();
    l.edit(&c, config(false, &["r|.*|"])).unwrap();
    assert_eq!(l.saved, saved);
    assert_eq!(l.last_results(&c).unwrap(), old);
    assert!(l.changed_since_run());
    assert!(!passed(&l, &c));
    l.save_config(&c).unwrap();
    assert!(!passed(&l, &c));
    l.run(&c).unwrap();
    assert!(!l.changed_since_run());
    assert!(!passed(&l, &c));
    assert!(
        l.checks(&c)
            .iter()
            .any(|r| !r.0 && r.1.contains("Needs to be allowed"))
    );
}
#[test]
fn malformed_saved_policy_is_retained_with_diagnostic_and_success_can_be_repaired() {
    let c = context(false);
    let mut l = w::Live::new(&c).unwrap();
    fix(&mut l, &c, "a|^/dev/team[a-z]$|");
    l.edit(
        &c,
        "devices {\nfilter = [\"r|.*|\"]\nuse_devicesfile = 0\n".into(),
    )
    .unwrap();
    l.save_config(&c).unwrap();
    let e = l.run(&c).unwrap_err();
    assert!(e.starts_with("Syntax:"));
    assert!(e.contains("line 4"));
    assert!(!passed(&l, &c));
    assert!(l.last_results(&c).unwrap().unwrap().is_err());
    l.edit(
        &c,
        "devices { use_devicesfile = 0 global_filter = [\"a|.*|\"] }".into(),
    )
    .unwrap();
    l.save_config(&c).unwrap();
    assert!(l.run(&c).unwrap_err().starts_with("Unsupported:"));
    fix(&mut l, &c, "a|^/dev/team[a-z]$|");
    assert!(passed(&l, &c));
}
#[test]
fn feedback_explains_default_acceptance_instead_of_merely_saying_incorrect() {
    let c = context(false);
    let mut l = w::Live::new(&c).unwrap();
    l.edit(&c, config(false, &["a|^/dev/team[a-z]$|"])).unwrap();
    l.save_config(&c).unwrap();
    l.run(&c).unwrap();
    let rows = l.checks(&c);
    assert!(rows.iter().any(|r| !r.0
        && r.1.contains("/dev/noise")
        && r.1.contains("no filter matched, so the default allows it")));
    assert!(rows.iter().any(|r| r.0 && r.1.contains("/dev/teama")));
}
#[test]
fn enabled_file_selection_really_wins_and_filter_only_workaround_cannot_pass() {
    let mut c = context(true);
    // All device-file cases share one real provided selection universe; the
    // arrival-check device must therefore be present in both supplied snapshots.
    let extra = c.cases[1].inventory.devices["/dev/teamb"].clone();
    c.cases[0]
        .inventory
        .devices
        .insert("/dev/teamb".into(), extra);
    c.cases[0].expected.push(w::Expected {
        path: "/dev/teamb".into(),
        allowed: true,
    });
    let mut l = w::Live::new(&c).unwrap();
    l.run(&c).unwrap();
    assert!(!passed(&l, &c));
    assert!(
        l.checks(&c)
            .iter()
            .any(|r| !r.0 && r.1.contains("filters are ignored"))
    );
    l.edit(&c, config(false, &["a|^/dev/team[a-z]$|", "r|.*|"]))
        .unwrap();
    l.save_config(&c).unwrap();
    l.run(&c).unwrap();
    assert!(!passed(&l, &c));
    assert!(
        l.checks(&c)
            .iter()
            .any(|r| !r.0 && r.1.contains("Use the provided devices file"))
    );
    l.edit(&c, config(true, &["r|.*|"])).unwrap();
    l.save_config(&c).unwrap();
    l.edit_selection(&c, selection(&["/dev/teamb", "/dev/teama"]))
        .unwrap();
    assert!(!passed(&l, &c));
    l.save_selection(&c).unwrap();
    l.run(&c).unwrap();
    assert!(passed(&l, &c), "{:?}", l.checks(&c));
    l.edit_selection(&c, selection(&["/dev/reservea"])).unwrap();
    assert!(!passed(&l, &c));
}
#[test]
fn roundtrip_retains_unfinished_drafts_historical_scan_and_does_not_fabricate_success() {
    let c = context(false);
    let mut assignment = w::Live::new(&c).unwrap();
    let mut example = w::Live::new(&c).unwrap();
    fix(&mut example, &c, "a|^/dev/team[a-z]$|");
    assert!(!passed(&assignment, &c));
    assignment
        .edit(&c, "# unfinished\ndevices {".into())
        .unwrap();
    let original = assignment.clone();
    let bytes = serde_json::to_vec(&assignment).unwrap();
    let loaded: w::Live = serde_json::from_slice(&bytes).unwrap();
    loaded.validate(&c).unwrap();
    assert_eq!(loaded, original);
    assert!(!passed(&loaded, &c));
    assert!(passed(&example, &c));
    let reset = w::Live::new(&c).unwrap();
    assert_eq!(reset.draft, c.initial_config);
    assert_eq!(example.last_results(&c).unwrap().unwrap().unwrap().len(), 2);
}
#[test]
fn input_and_import_bounds_reject_excess_and_unknown_fields_without_touching_prior_draft() {
    let c = context(false);
    let mut l = w::Live::new(&c).unwrap();
    let before = l.clone();
    for bad in ["a".repeat(4097), "nul\0byte".into()] {
        assert!(l.edit(&c, bad).is_err());
        assert_eq!(l, before);
    }
    let mut wire = serde_json::to_value(&l).unwrap();
    wire["success"] = json!(true);
    assert!(serde_json::from_value::<w::Live>(wire).is_err());
    let mut wire = serde_json::to_value(&l).unwrap();
    wire["draft"] = json!("a".repeat(4097));
    assert!(serde_json::from_value::<w::Live>(wire).is_err());
    let mut wire = serde_json::to_value(&c).unwrap();
    wire["cases"] = json!([wire["cases"][0].clone(),wire["cases"][0].clone(),wire["cases"][0].clone(), {"malformed":"excess must reject before value"}]);
    let e = serde_json::from_value::<w::Context>(wire)
        .unwrap_err()
        .to_string();
    assert!(e.contains("limit"), "{e}");
    let entries = format!(
        "{{\"entries\":[{},{{\"not\":\"a device name\"}}]}}",
        (0..8)
            .map(|i| format!("\"/dev/a{i}\""))
            .collect::<Vec<_>>()
            .join(",")
    );
    let e = serde_json::from_str::<w::Selection>(&entries)
        .unwrap_err()
        .to_string();
    assert!(e.contains("limit"), "{e}");
}
#[test]
fn selection_edits_cannot_invent_paths_create_a_file_or_modify_a_readonly_fixture() {
    let c = context(false);
    let mut l = w::Live::new(&c).unwrap();
    assert!(l.edit_selection(&c, selection(&["/dev/teama"])).is_err());
    assert!(l.save_selection(&c).is_err());
    let c = context(true);
    let mut l = w::Live::new(&c).unwrap();
    let before = l.clone();
    for paths in [
        &["/dev/../../etc/passwd"][..],
        &["/dev/unknown"][..],
        &["/dev/teama", "/dev/teama"][..],
    ] {
        assert!(l.edit_selection(&c, selection(paths)).is_err());
        assert_eq!(l, before);
    }
    let mut forged = l.clone();
    forged.saved.file = None;
    assert!(forged.validate(&c).is_err());
    let mut readonly = c.clone();
    readonly.selection_editable = false;
    assert!(l.edit_selection(&readonly, selection(&[])).is_err());
}
#[test]
fn invalid_contexts_and_invented_requirements_are_rejected_instead_of_graded() {
    let c = context(false);
    for change in 0..6 {
        let mut bad = c.clone();
        match change {
            0 => bad.cases[1].label = bad.cases[0].label.clone(),
            1 => bad.cases[0].expected.pop().map(|_| ()).unwrap(),
            2 => bad.cases[0].expected[0].path = "/dev/unknown".into(),
            3 => {
                bad.cases[0]
                    .inventory
                    .devices
                    .values_mut()
                    .next()
                    .unwrap()
                    .bytes = 0
            }
            4 => bad.required_mode = Some(w::RequiredMode::DevicesFile),
            _ => bad.cases.truncate(1),
        }
        assert!(bad.shape().is_err());
        assert!(w::Live::new(&bad).is_err());
    }
}
#[test]
fn serialized_run_position_cannot_substitute_for_current_correct_behavior() {
    let c = context(false);
    let mut l = w::Live::new(&c).unwrap();
    l.last_run = Some(l.saved.clone());
    assert!(!passed(&l, &c));
    fix(&mut l, &c, "a|^/dev/team[a-z]$|");
    assert!(passed(&l, &c));
    l.saved.config = w::Text(config(false, &["a|.*|"]));
    l.draft = l.saved.config.clone();
    l.last_run = Some(l.saved.clone());
    assert!(!passed(&l, &c));
    assert!(
        l.checks(&c)
            .iter()
            .any(|r| !r.0 && r.1.contains("Needs to be excluded"))
    );
    assert_eq!(w::Live::new(&c).unwrap().saved.config, c.initial_config);
}
#[test]
fn typed_reference_requires_actual_reachability_not_just_a_scan_or_plausible_text() {
    let c = context(false);
    let task = w::Task {
        context: c.clone(),
        reference: vec![
            w::Operation::SaveConfig {
                text: w::Text(config(false, &["a|^/dev/team[a-z]$|", "r|.*|"])),
            },
            w::Operation::Scan,
        ],
    };
    task.validate().unwrap();
    let mut wrong = task.clone();
    wrong.reference[0] = w::Operation::SaveConfig {
        text: w::Text(config(false, &["a|^/dev/teama$|", "r|.*|"])),
    };
    assert!(wrong.validate().is_err());
    let mut wrong = task.clone();
    wrong.reference.pop();
    assert!(wrong.shape().is_err());
    let mut already = task.clone();
    already.context.initial_config = match &task.reference[0] {
        w::Operation::SaveConfig { text } => text.clone(),
        _ => unreachable!(),
    };
    assert!(
        already
            .validate()
            .unwrap_err()
            .contains("real configuration")
    );
    let mut invalid = task.clone();
    invalid.reference = vec![
        w::Operation::SaveSelection {
            selection: selection(&["/dev/teama"]),
        },
        w::Operation::Scan,
    ];
    assert!(invalid.shape().is_err());
    let before = serde_json::to_vec(&c).unwrap();
    let mut l = w::Live::new(&c).unwrap();
    for op in &task.reference {
        l.apply(&c, op).unwrap();
    }
    assert!(passed(&l, &c));
    assert_eq!(serde_json::to_vec(&c).unwrap(), before);
}
fn task() -> w::Task {
    w::Task {
        context: context(false),
        reference: vec![
            w::Operation::SaveConfig {
                text: w::Text(config(false, &["a|^/dev/team[a-z]$|", "r|.*|"])),
            },
            w::Operation::Scan,
        ],
    }
}
fn practice() -> w::Practice {
    let mut c = context(false);
    for case in &mut c.cases {
        case.inventory.groups.clear();
        case.inventory.identity = None;
        case.inventory.activation = None;
        case.inventory.provisioning = None;
        let keys = case.inventory.devices.keys().cloned().collect::<Vec<_>>();
        for key in keys {
            let mut dev = case.inventory.devices.remove(&key).unwrap();
            dev.group.clear();
            case.inventory
                .devices
                .insert(key.replace("/dev/", "/dev/example-"), dev);
        }
        for e in &mut case.expected {
            e.path = e.path.replace("/dev/", "/dev/example-");
        }
    }
    let good = w::Operation::SaveConfig {
        text: w::Text(config(false, &["a|^/dev/example-team[a-z]$|", "r|.*|"])),
    };
    w::Practice{context:c,worked:vec![w::Step{operation:good.clone(),explanation:"Allow the supplied example data PVs before excluding all other supplied paths. Save, then scan and compare the two inventories.".into()},w::Step{operation:w::Operation::Scan,explanation:"The actual scan should read both example data PVs and exclude the unrelated device in each inventory.".into()}],experiment:vec![w::Operation::SaveConfig{text:w::Text(config(false,&["r|.*|","a|^/dev/example-team[a-z]$|"]))},w::Operation::Scan],repair:vec![good,w::Operation::Scan]}
}
#[test]
fn guided_practice_verifies_worked_change_repair_and_cannot_be_a_cosmetic_checked_flag() {
    let p = practice();
    p.validate().unwrap();
    let task = task();
    task.validate().unwrap();
    let mut runtime = w::Runtime::new(&task, &p).unwrap();
    let original = runtime.assignment.clone();
    for step in &p.worked {
        runtime.example.apply(&p.context, &step.operation).unwrap();
    }
    assert!(passed(&runtime.example, &p.context));
    assert_eq!(runtime.assignment, original);
    for op in &p.experiment {
        runtime.example.apply(&p.context, op).unwrap();
    }
    assert!(!passed(&runtime.example, &p.context));
    for op in &p.repair {
        runtime.example.apply(&p.context, op).unwrap();
    }
    assert!(passed(&runtime.example, &p.context));
    let mut invalid = p.clone();
    invalid.experiment = vec![w::Operation::Scan];
    assert!(
        invalid
            .validate()
            .unwrap_err()
            .contains("real saved-policy")
    );
    let mut invalid = p.clone();
    invalid.repair = vec![w::Operation::Scan];
    assert!(invalid.validate().is_err());
    let mut invalid = p.clone();
    invalid.worked[0].explanation.clear();
    assert!(invalid.shape().is_err());
    let mut invalid = p.clone();
    invalid.worked.pop();
    assert!(invalid.shape().is_err());
    let mut invalid = p.clone();
    invalid.context = task.context.clone();
    assert!(w::Runtime::new(&task, &invalid).is_err());
}
#[test]
fn paired_runtime_roundtrip_and_selected_reset_preserve_other_work_and_both_drafts() {
    let task = task();
    let p = practice();
    let mut r = w::Runtime::new(&task, &p).unwrap();
    r.assignment
        .edit(&task.context, "# unfinished assignment\ndevices {".into())
        .unwrap();
    r.example
        .edit(&p.context, "# unfinished example\ndevices {".into())
        .unwrap();
    let bytes = serde_json::to_vec(&r).unwrap();
    let mut loaded: w::Runtime = serde_json::from_slice(&bytes).unwrap();
    loaded.validate(&task, &p).unwrap();
    assert_eq!(loaded, r);
    let assignment = loaded.assignment.clone();
    loaded.reset_example(&task, &p).unwrap();
    assert_eq!(loaded.assignment, assignment);
    assert_eq!(loaded.example.draft, p.context.initial_config);
    let example = loaded.example.clone();
    loaded.reset_assignment(&task, &p).unwrap();
    assert_eq!(loaded.example, example);
    assert_eq!(loaded.assignment.draft, task.context.initial_config);
    let mut forged = loaded.clone();
    forged.version = 2;
    assert!(forged.validate(&task, &p).is_err());
    let mut forged = loaded;
    forged.assignment.saved.file = Some(selection(&["/dev/teama"]));
    assert!(forged.validate(&task, &p).is_err());
}
