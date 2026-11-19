//! Standalone configuration/scan capability only: no new authored coverage.
use dlms_learning_lab::{
    lvm::{self, config_scan as c},
    lvm_workspace,
};
use serde_json::{Value, json};
fn state() -> lvm::State {
    let raw: Value =
        serde_json::from_str(include_str!("../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    let mut s: lvm::State =
        serde_json::from_value(raw["scenarios"][0]["goal"]["definition"]["initial"].clone())
            .unwrap();
    s.devices.insert(
        "/dev/noise".into(),
        lvm::Device {
            bytes: 8 * lvm::MIB,
            role: lvm::DeviceRole::Foreign,
            pv_bytes: None,
            group: String::new(),
        },
    );
    s.validate().unwrap();
    s
}
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
fn policy(flag: bool, rules: &[&str]) -> c::Policy {
    c::parse(&config(flag, rules)).unwrap()
}
fn allowed(r: &c::Report) -> Vec<&str> {
    r.records
        .iter()
        .filter(|r| r.allowed)
        .map(|r| r.path.as_str())
        .collect()
}
fn file(paths: &[&str]) -> c::DevicesFile {
    c::DevicesFile::new(&paths.iter().map(|s| s.to_string()).collect::<Vec<_>>()).unwrap()
}
#[test]
fn real_parser_handles_comments_setting_order_delimiters_and_inner_alternation() {
    let source = "# given policy\r\ndevices {\nfilter = [ \"a#^/dev/(teama|reservea)$#\", # explanation\n\"r|.*|\" ]\nuse_devicesfile = 0 # explicit, no distribution-default guess\n}\n";
    let p = c::parse(source).unwrap();
    assert!(!p.uses_devices_file());
    assert_eq!(p.rules().len(), 2);
    assert_eq!(p.rules()[0].pattern, "^/dev/(teama|reservea)$");
    assert_eq!(
        allowed(&c::scan(&p, &state(), None).unwrap()),
        ["/dev/reservea", "/dev/teama"]
    );
    for entry in [
        "a(^/dev/teama$)",
        "a[^/dev/teama$]",
        "a{^/dev/teama$}",
        "a|^/dev/(team|reserve)a$|",
        "a/^/dev/teama$/",
    ] {
        let p = policy(false, &[entry, "r|.*|"]);
        let r = c::scan(&p, &state(), None).unwrap();
        assert!(!allowed(&r).is_empty());
        assert!(!allowed(&r).contains(&"/dev/noise"));
    }
}
#[test]
fn first_matching_rule_wins_and_unmatched_devices_are_allowed_instead_of_faked_safe() {
    let s = state();
    let denied = c::scan(&policy(false, &["r|.*|", "a|^/dev/teama$|"]), &s, None).unwrap();
    assert!(allowed(&denied).is_empty());
    assert!(denied.records.iter().all(|r| r.metadata.is_none()
        && r.reason
            == c::Reason::FilterRule {
                index: 0,
                action: c::Action::Reject
            }));
    let fixed = c::scan(&policy(false, &["a|^/dev/teama$|", "r|.*|"]), &s, None).unwrap();
    assert_eq!(allowed(&fixed), ["/dev/teama"]);
    assert_eq!(fixed.mode, c::Mode::FilterSelected);
    let open = c::scan(&policy(false, &["r|^/dev/not_present$|"]), &s, None).unwrap();
    assert_eq!(allowed(&open).len(), s.devices.len());
    assert!(
        open.records
            .iter()
            .all(|r| r.reason == c::Reason::NoFilterMatched)
    );
    let noise = open
        .records
        .iter()
        .find(|r| r.path == "/dev/noise")
        .unwrap();
    assert_eq!(noise.metadata, Some(c::Metadata::NoPhysicalVolumeLabel));
}
#[test]
fn metadata_is_read_from_actual_virtual_state_and_follows_real_modeled_group_renames() {
    let mut s = state();
    let p = policy(false, &["a|^/dev/teama$|", "r|.*|"]);
    let before = s.clone();
    let r = c::scan(&p, &s, None).unwrap();
    assert_eq!(
        r.records
            .iter()
            .find(|r| r.path == "/dev/teama")
            .unwrap()
            .metadata,
        Some(c::Metadata::PhysicalVolume {
            group: Some("team".into())
        })
    );
    assert_eq!(s, before);
    s.execute("vgrename team renamed").unwrap();
    let r = c::scan(&p, &s, None).unwrap();
    assert_eq!(
        r.records
            .iter()
            .find(|r| r.path == "/dev/teama")
            .unwrap()
            .metadata,
        Some(c::Metadata::PhysicalVolume {
            group: Some("renamed".into())
        })
    );
    assert_eq!(
        s.groups["renamed"].volumes["draft"].filesystem,
        before.groups["team"].volumes["draft"].filesystem
    );
}
#[test]
fn present_devices_file_controls_selection_and_filter_is_ignored_but_not_the_reverse() {
    let s = state();
    let f = file(&["/dev/reservea"]);
    let p = policy(true, &["a|^/dev/teama$|", "r|.*|"]);
    let r = c::scan_limited(&p, &s, Some(&f), 0).unwrap();
    assert_eq!(r.mode, c::Mode::DevicesFile);
    assert_eq!(allowed(&r), ["/dev/reservea"]);
    assert_eq!(r.matching_work, 0);
    let no_file = c::scan(&p, &s, None).unwrap();
    assert_eq!(no_file.mode, c::Mode::FilterBecauseFileMissing);
    assert_eq!(allowed(&no_file), ["/dev/teama"]);
    let disabled = c::scan(&policy(false, &["a|^/dev/teama$|", "r|.*|"]), &s, Some(&f)).unwrap();
    assert_eq!(disabled.mode, c::Mode::FilterSelected);
    assert_eq!(allowed(&disabled), ["/dev/teama"]);
    let empty = c::scan(&p, &s, Some(&file(&[]))).unwrap();
    assert!(allowed(&empty).is_empty());
    assert!(
        empty
            .records
            .iter()
            .all(|r| r.reason == c::Reason::NotInDevicesFile)
    );
}
#[test]
fn absent_device_file_entries_are_reported_without_reading_or_inventing_devices() {
    let s = state();
    let f = file(&["/dev/missing", "/dev/teama"]);
    let r = c::scan(&policy(true, &["r|.*|"]), &s, Some(&f)).unwrap();
    assert_eq!(allowed(&r), ["/dev/teama"]);
    assert_eq!(r.unresolved_file_entries, ["/dev/missing"]);
    assert_eq!(r.records.len(), s.devices.len());
    assert!(r.records.iter().all(|r| r.path != "/dev/missing"));
}
#[test]
fn equivalent_supported_expressions_and_independent_rule_order_have_same_actual_access() {
    let s = state();
    let a = c::scan(
        &policy(false, &["a|^/dev/teama$|", "a|^/dev/reservea$|", "r|.*|"]),
        &s,
        None,
    )
    .unwrap();
    for rules in [
        vec!["a|^/dev/(reserve|team)a$|", "r|.*|"],
        vec!["a|^/dev/reservea$|", "a|^/dev/teama$|", "r|.*|"],
    ] {
        let b = c::scan(&policy(false, &rules), &s, None).unwrap();
        assert_eq!(allowed(&a), allowed(&b));
        assert_eq!(
            a.records.iter().map(|r| &r.metadata).collect::<Vec<_>>(),
            b.records.iter().map(|r| &r.metadata).collect::<Vec<_>>()
        );
    }
}
#[test]
fn malformed_syntax_is_distinct_from_unsupported_configuration_or_expression_features() {
    for src in [
        "devices { use_devicesfile = 0 filter = [\"a|.*|\" }",
        "devices {use_devicesfile=0 filter=[\"x|.*|\"]}",
        "devices {use_devicesfile=0 filter=[\"a|.*#\"]}",
        "devices { use_devicesfile=1filter=[\"a|.*|\"] }",
        "devices {use_devicesfile=0 filter=[]}",
    ] {
        assert_eq!(c::parse(src).unwrap_err().kind, c::Kind::Syntax, "{src}");
    }
    for src in [
        "devices {use_devicesfile=2 filter=[\"a|.*|\"]}",
        "devices {filter=[\"a|.*|\"]}",
        "devices {use_devicesfile=0 filter=[\"a|.*|\"] scan_lvs=1}",
        "devices {use_devicesfile=0 use_devicesfile=1 filter=[\"a|.*|\"]}",
        "devices {use_devicesfile=0 filter=[\"a|.*|\",]}",
        "devices {use_devicesfile=0 filter=[\"a|.*|\"]} backup {backup=1}",
    ] {
        assert_eq!(
            c::parse(src).unwrap_err().kind,
            c::Kind::Unsupported,
            "{src}"
        );
    }
    for rule in ["a|(?=x)|", "a||", r"a|\d+|"] {
        assert_eq!(
            c::parse(&config(false, &[rule])).unwrap_err().kind,
            c::Kind::Unsupported,
            "{rule}"
        );
    }
    assert_eq!(
        c::parse("devices {use_devicesfile=0 filter=[\"a|é|\"]}")
            .unwrap_err()
            .kind,
        c::Kind::Unsupported
    );
}
#[test]
fn admission_limits_precede_excess_values_and_expression_growth_is_bounded() {
    assert_eq!(
        c::parse(&" ".repeat(c::MAX_CONFIG + 1)).unwrap_err().kind,
        c::Kind::Limit
    );
    let long = format!("a|{}|", "a".repeat(c::MAX_PATTERN + 1));
    assert_eq!(
        c::parse(&config(false, &[&long])).unwrap_err().kind,
        c::Kind::Limit
    );
    let mut src = format!(
        "devices {{use_devicesfile=0 filter=[{},",
        ["\"a|.*|\""; 8].join(",")
    );
    src.push_str("not-even-a-quoted-entry");
    assert_eq!(c::parse(&src).unwrap_err().kind, c::Kind::Limit);
    let exploding = "a|((a{32}){32}){32}|";
    assert_eq!(
        c::parse(&config(false, &[exploding])).unwrap_err().kind,
        c::Kind::Limit
    );
}
#[test]
fn one_matching_budget_covers_the_entire_scan_and_failure_leaves_source_unchanged() {
    let s = state();
    let before = s.clone();
    let p = policy(false, &["a|^/dev/teama$|", "r|.*|"]);
    let r = c::scan(&p, &s, None).unwrap();
    assert!(r.matching_work > 0);
    assert_eq!(c::scan_limited(&p, &s, None, r.matching_work).unwrap(), r);
    for limit in [0, r.matching_work - 1, c::MAX_MATCH_WORK + 1] {
        assert_eq!(
            c::scan_limited(&p, &s, None, limit).unwrap_err().kind,
            c::Kind::Limit
        );
        assert_eq!(s, before);
    }
}
#[test]
fn device_file_and_inventory_bounds_reject_alias_traversal_duplicates_and_foreign_state() {
    for paths in [
        vec!["/dev/teama", "/dev/teama"],
        vec!["/home/private"],
        vec!["/dev/../private"],
        vec!["/dev/disk/by-id/x"],
        vec!["/dev/-bad"],
    ] {
        let p = paths.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            c::DevicesFile::new(&p).unwrap_err().kind,
            c::Kind::InvalidContext
        );
    }
    assert_eq!(
        c::DevicesFile::new(&vec!["/dev/a".to_string(); 9])
            .unwrap_err()
            .kind,
        c::Kind::Limit
    );
    let mut s = state();
    s.devices.get_mut("/dev/noise").unwrap().group = "team".into();
    assert_eq!(
        c::scan(&policy(false, &["a|.*|"]), &s, None)
            .unwrap_err()
            .kind,
        c::Kind::InvalidContext
    );
    let mut s = state();
    s.groups
        .get_mut("team")
        .unwrap()
        .volumes
        .get_mut("draft")
        .unwrap()
        .filesystem
        .as_mut()
        .unwrap()
        .files
        .get_mut("/keep.txt")
        .unwrap()
        .contents = "x".repeat(65536);
    assert_eq!(
        c::scan(&policy(false, &["a|.*|"]), &s, None)
            .unwrap_err()
            .kind,
        c::Kind::Limit
    );
}
#[test]
fn normal_commands_malformed_goal_fields_and_host_execution_remain_refused() {
    let mut s = state();
    let before = s.clone();
    for text in [
        "lvmconfig --type current",
        "lvmdevices --adddev /dev/teama",
        "sh -c pvs",
        "config set filter accept-all",
    ] {
        assert!(s.execute(text).is_err());
        assert_eq!(s, before);
    }
    let raw: Value =
        serde_json::from_str(include_str!("../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    let mut d = raw["scenarios"][0]["goal"]["definition"].clone();
    d["config_scan_task"] = json!({});
    assert!(serde_json::from_value::<lvm_workspace::Definition>(d).is_err());
    let mut value = serde_json::to_value(&s).unwrap();
    value["scan_policy"] = json!({});
    assert!(serde_json::from_value::<lvm::State>(value).is_err());
    let harmless = "# $(touch /not-a-real-command)\ndevices {use_devicesfile=0 filter=[\"a|.*|\"]}";
    assert!(
        c::scan(&c::parse(harmless).unwrap(), &s, None)
            .unwrap()
            .records
            .iter()
            .all(|r| r.allowed)
    );
    assert!(
        c::parse("devices {use_devicesfile=0 filter=[\"a|.*|\"]}; touch /not-a-real-command")
            .is_err()
    );
}

#[test]
fn parser_feedback_points_to_a_line_and_column_without_exposing_engine_jargon() {
    let source = "devices {\n use_devicesfile=0\n filter=[\"a|.*|\" \"r|.*|\"]\n}";
    let error = c::parse(source).unwrap_err();
    assert_eq!(error.kind, c::Kind::Syntax);
    assert_eq!(error.line, Some(3));
    assert!(error.column.unwrap() > 1);
    let message = error.to_string();
    assert!(message.contains("line 3") && message.contains("column"));
    assert!(!message.contains("byte"));
    let error = c::parse(&config(false, &["a|((a{32}){32}){32}|"])).unwrap_err();
    assert_eq!(error.kind, c::Kind::Limit);
    assert!(error.message.contains("simpler pattern"));
    assert!(!error.message.contains("states"));
}
