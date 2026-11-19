//! Standalone configuration/outcome proof; not learner/session admission.
use dlms_learning_lab::{
    lvm::{
        self, device_path,
        device_reference::{Caller, Configuration},
    },
    lvm_workspace::device_reference::{self as goals, Plan, Target},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
fn state() -> lvm::State {
    let v: Value =
        serde_json::from_str(include_str!("../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    let s: lvm::State =
        serde_json::from_value(v["scenarios"][0]["goal"]["definition"]["initial"].clone()).unwrap();
    s.validate().unwrap();
    s
}
fn file(s: &lvm::State, volume: &str) -> String {
    s.volume(volume)
        .unwrap()
        .1
        .filesystem
        .as_ref()
        .unwrap()
        .files
        .keys()
        .next()
        .unwrap()
        .clone()
}
fn configuration(s: &lvm::State) -> Configuration {
    Configuration {
        callers: BTreeMap::from([
            (
                "reporting".into(),
                Caller {
                    label: "Reporting application".into(),
                    file: file(s, "team/draft"),
                    path: device_path::modeled_mapper("team", "draft").unwrap(),
                },
            ),
            (
                "reference".into(),
                Caller {
                    label: "Reference reader".into(),
                    file: file(s, "team/reference"),
                    path: device_path::canonical("team", "reference").unwrap(),
                },
            ),
        ]),
    }
}
fn plan() -> Plan {
    Plan {
        targets: vec![Target {
            caller: "reporting".into(),
            original_volume: "team/draft".into(),
            canonical_required: true,
        }],
        renaming: None,
    }
}
fn all(
    s: &lvm::State,
    current: &lvm::State,
    c: &Configuration,
    now: &Configuration,
    p: &Plan,
) -> bool {
    goals::outcomes(s, current, c, now, p).iter().all(|r| r.0)
}
fn solved(c: &Configuration) -> Configuration {
    let mut now = c.clone();
    now.save("reporting", "/dev/team/draft").unwrap();
    now
}
#[test]
fn canonical_repair_reads_actual_original_data_without_changing_storage_or_needing_host_files() {
    let s = state();
    let before = s.clone();
    let c = configuration(&s);
    let p = plan();
    p.validate(&s, &c).unwrap();
    assert!(!all(&s, &s, &c, &c, &p));
    let rows = goals::outcomes(&s, &s, &c, &c, &p);
    assert!(rows[0].1.contains("internal mapper"));
    let now = solved(&c);
    assert!(all(&s, &s, &c, &now, &p));
    let read = now.inspect(&s, "reporting").unwrap();
    assert_eq!(read.form, device_path::Form::Canonical);
    assert_eq!(
        read.content,
        s.read_file("team/draft", &c.callers["reporting"].file)
            .unwrap()
    );
    assert_eq!(s, before);
    let mut alternate = s.clone();
    alternate.execute("vgrename team temporary").unwrap();
    alternate.execute("vgrename temporary team").unwrap();
    assert!(all(&s, &alternate, &c, &now, &p));
    let mut either = p.clone();
    either.targets[0].canonical_required = false;
    assert!(
        all(&s, &s, &c, &c, &either),
        "Explicit observed-form task may allow either actual reference"
    );
}
#[test]
fn valid_alias_with_identical_output_from_wrong_object_cannot_fake_original_volume_credit() {
    let mut s = state();
    let source_file = file(&s, "team/draft");
    let contents = s.read_file("team/draft", &source_file).unwrap().to_owned();
    s.groups
        .get_mut("team")
        .unwrap()
        .volumes
        .get_mut("reference")
        .unwrap()
        .filesystem
        .as_mut()
        .unwrap()
        .files
        .insert(source_file.clone(), lvm::File { contents });
    s.validate().unwrap();
    let c = configuration(&s);
    let mut now = solved(&c);
    now.save("reporting", "/dev/team/reference").unwrap();
    assert!(now.inspect(&s, "reporting").is_ok());
    let rows = goals::outcomes(&s, &s, &c, &now, &plan());
    assert!(!rows[0].0 && rows[0].1.contains("another volume"));
    assert!(!all(&s, &s, &c, &now, &plan()));
}
#[test]
fn blank_malformed_unknown_and_inactive_references_have_specific_retryable_reader_failures() {
    let s = state();
    let c = configuration(&s);
    let mut now = c.clone();
    for (path, fragment) in [
        ("", "blank"),
        ("/dev/team/../draft", "names"),
        ("/dev/no_group/draft", "no current modeled volume group"),
        ("/dev/team/no_lv", "no current modeled logical volume"),
    ] {
        now.save("reporting", path).unwrap();
        assert!(goals::same_context(&s, &s, &c, &now));
        let text = goals::outcomes(&s, &s, &c, &now, &plan())[0].1.clone();
        assert!(text.contains(fragment), "{path}: {text}");
        assert!(!all(&s, &s, &c, &now, &plan()));
        now.save("reporting", "/dev/team/draft").unwrap();
        assert!(all(&s, &s, &c, &now, &plan()));
    }
    let mut inactive = s.clone();
    inactive
        .activation
        .as_mut()
        .unwrap()
        .active
        .insert("team/draft".into(), false);
    inactive.validate().unwrap();
    assert!(now.inspect(&inactive, "reporting").is_err());
    assert!(!all(&s, &inactive, &c, &now, &plan()));
}
#[test]
fn source_roster_and_unrequested_configurations_stay_protected_but_bad_paths_remain_repairable() {
    let s = state();
    let c = configuration(&s);
    let mut now = solved(&c);
    now.save("reference", "/dev/team/draft").unwrap();
    assert!(goals::same_context(&s, &s, &c, &now));
    let rows = goals::outcomes(&s, &s, &c, &now, &plan());
    assert!(rows[0].0 && !rows[1].0);
    now.save("reference", &c.callers["reference"].path).unwrap();
    assert!(all(&s, &s, &c, &now, &plan()));
    let mut bad = now.clone();
    bad.callers.get_mut("reference").unwrap().label = "Forged caller role".into();
    assert!(!goals::same_context(&s, &s, &c, &bad));
    let mut bad = now;
    bad.callers.get_mut("reference").unwrap().file = "/different.txt".into();
    assert!(!goals::same_context(&s, &s, &c, &bad));
}
#[test]
fn actual_original_renames_require_both_saved_references_to_be_repaired() {
    let s = state();
    let c = configuration(&s);
    let mut p = plan();
    p.targets.push(Target {
        caller: "reference".into(),
        original_volume: "team/reference".into(),
        canonical_required: true,
    });
    p.renaming=Some(serde_json::from_value(json!({"groups":[{"source":"team","name":"archive-pool"}],"volumes":[{"source":"team/draft","name":"reports-live"}]})).unwrap());
    p.validate(&s, &c).unwrap();
    let mut current = s.clone();
    current.execute("lvrename team draft reports-live").unwrap();
    current.execute("vgrename team archive-pool").unwrap();
    assert!(goals::same_context(&s, &current, &c, &c));
    assert!(c.inspect(&current, "reporting").is_err());
    assert!(!all(&s, &current, &c, &c, &p));
    let mut now = c.clone();
    now.save("reporting", "/dev/archive-pool/reports-live")
        .unwrap();
    let rows = goals::outcomes(&s, &current, &c, &now, &p);
    assert!(rows[0].0 && !rows[1].0);
    now.save("reference", "/dev/archive-pool/reference")
        .unwrap();
    assert!(
        all(&s, &current, &c, &now, &p),
        "{:?}",
        goals::outcomes(&s, &current, &c, &now, &p)
    );
    let mut reverse = s.clone();
    reverse.execute("vgrename team archive-pool").unwrap();
    reverse
        .execute("lvrename archive-pool draft reports-live")
        .unwrap();
    assert!(all(&s, &reverse, &c, &now, &p));
}
#[test]
fn extra_storage_changed_bytes_and_later_wrong_reference_invalidate_success() {
    let s = state();
    let c = configuration(&s);
    let mut now = solved(&c);
    assert!(all(&s, &s, &c, &now, &plan()));
    now.save("reporting", "/dev/not_present/no_volume").unwrap();
    assert!(!all(&s, &s, &c, &now, &plan()));
    now = solved(&c);
    let mut current = s.clone();
    current.execute("lvcreate -l 1 -n extra team").unwrap();
    assert!(goals::same_context(&s, &current, &c, &now));
    assert!(!all(&s, &current, &c, &now, &plan()));
    current.execute("lvremove -y team/extra").unwrap();
    assert!(all(&s, &current, &c, &now, &plan()));
    let file = file(&s, "team/draft");
    current
        .groups
        .get_mut("team")
        .unwrap()
        .volumes
        .get_mut("draft")
        .unwrap()
        .filesystem
        .as_mut()
        .unwrap()
        .files
        .get_mut(&file)
        .unwrap()
        .contents = "Changed virtual record\n".into();
    current.validate().unwrap();
    assert!(!all(&s, &current, &c, &now, &plan()));
}
#[test]
fn bounded_configuration_roundtrip_atomic_refusals_and_unknown_fields_never_execute_text() {
    let s = state();
    let c = configuration(&s);
    c.validate().unwrap();
    assert_eq!(
        serde_json::from_slice::<Configuration>(&serde_json::to_vec(&c).unwrap()).unwrap(),
        c
    );
    for path in [
        "x".repeat(257),
        "/dev/team/draft\nsh".into(),
        "/dev/team/draft\0".into(),
    ] {
        let mut now = c.clone();
        assert!(now.save("reporting", &path).is_err());
        assert_eq!(now, c);
    }
    let mut now = c.clone();
    assert!(now.save("unknown", "/dev/team/draft").is_err());
    assert_eq!(now, c);
    now.save("reporting", "$(touch /personal-file)").unwrap();
    assert!(now.inspect(&s, "reporting").is_err());
    assert!(!all(&s, &s, &c, &now, &plan()));
    let mut value = serde_json::to_value(&c).unwrap();
    value["callers"]["reporting"]["execute"] = "sh".into();
    assert!(serde_json::from_value::<Configuration>(value).is_err());
    let caller = serde_json::to_value(&c.callers["reporting"]).unwrap();
    let value = json!({"callers":(0..5).map(|i|(format!("app{i}"),caller.clone())).collect::<BTreeMap<_,_>>()});
    assert!(serde_json::from_value::<Configuration>(value).is_err());
}
#[test]
fn malformed_or_unavailable_source_goals_are_not_accepted_as_teaching_content() {
    let s = state();
    let c = configuration(&s);
    let mut empty = plan();
    empty.targets.clear();
    assert!(empty.validate(&s, &c).is_err());
    let mut p = plan();
    p.targets[0].caller = "no_caller".into();
    assert!(p.validate(&s, &c).is_err());
    let mut p = plan();
    p.targets[0].original_volume = "team/no_volume".into();
    assert!(p.validate(&s, &c).is_err());
    let mut p = plan();
    p.targets.push(p.targets[0].clone());
    assert!(p.validate(&s, &c).is_err());
    let mut wrong = s.clone();
    wrong.identity = None;
    assert!(plan().validate(&wrong, &c).is_err());
    let mut wrong = c;
    wrong.callers.get_mut("reporting").unwrap().file = "/../host".into();
    assert!(plan().validate(&s, &wrong).is_err());
}
