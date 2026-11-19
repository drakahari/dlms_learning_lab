use dlms_learning_lab::{
    course,
    engine::{self, State},
    ipv6,
    ipv6_teaching::Learning,
    ipv6_workspace::Draft,
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::OnceLock;
const ID: &str = "ccna-ipv6-modified-eui64";
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::ccna_pilot().unwrap())
}
fn lab() -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == ID).unwrap()
}
fn solved(state: &State) -> bool {
    engine::checks(state, &lab().goal).iter().all(|c| c.0)
}
fn run(state: &mut State, commands: &[&str]) {
    for c in commands {
        engine::execute(state, c).unwrap();
    }
}
#[test]
fn new_source_and_unit_preserve_all372_old_scenarios_and137_unit_values() {
    let c = catalog();
    assert_eq!(c.len(), 402);
    assert_eq!(c[372].id, ID);
    let old = pack::accepted372().unwrap();
    assert_eq!(old.len(), 372);
    assert_eq!(
        serde_json::to_value(&c[..372]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let units = course::ccna_pilot(c).unwrap();
    assert_eq!(units.len(), 154);
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| !matches!(
                    u.id.as_str(),
                    "ipv6-interface-identifiers"
                        | "linux-boot-entry-recovery"
                        | "linux-grub-menu-recovery"
                        | "linux-grub-persistent-configuration"
                        | "linux-bios-loader-recovery"
                        | "linux-efi-loader-recovery"
                        | "linux-removable-efi-recovery"
                        | "linux-temporary-grub-recovery" | "linux-rescue-target-repair" | "linux-clock-source-diagnosis" | "linux-copied-journal" | "linux-local-mail-routing" | "ccna-wired-device-paths" | "structured-network-inventory" | "linux-workload-disk-layout" | "shell-measurement-reports" | "linux-resource-observations"
                ))
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(course::accepted372(&old).unwrap()).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "ipv6-interface-identifiers")
        .unwrap();
    assert_eq!(units[at - 1].id, "ipv6-static-foundations");
    assert_eq!(units[at].labs, [ID]);
    let rows: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/presentation/ccna.json")).unwrap();
    let row = rows.as_array().unwrap().iter().find(|row| row["id"] == ID).unwrap();
    assert_eq!(row["id"], ID);
    assert_eq!(
        row["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap())
            .collect::<Vec<_>>()
            .join(" "),
        lab().objective
    );
    assert_eq!(lab().prerequisites, ["ccna-ipv6-workstation"]);
    assert!(matches!(lab().initial, State::Ipv6 { .. }));
}
#[test]
fn taught_solution_and_equivalent_address_spelling_and_next_hops_work() {
    let mut state = lab().initial.clone();
    assert!(!solved(&state));
    for action in &lab().solution {
        engine::execute(&mut state, action).unwrap();
    }
    assert!(solved(&state));
    let mut alternate = lab().initial.clone();
    run(
        &mut alternate,
        &[
            "address Client lan 2001:0DB8:0074:0000:0425:96FF:FEAB:CDEF/64",
            "remove-route Client 0",
            "route Client ::/0 lan 2001:db8:74::1 19",
        ],
    );
    assert!(solved(&alternate));
    assert_eq!(
        ipv6::interface_id::address(
            "06:25:96:ab:cd:ef",
            &ipv6::Prefix::parse("2001:db8:74::/64").unwrap()
        )
        .unwrap()
        .to_string(),
        "2001:db8:74:0:425:96ff:feab:cdef"
    );
}
#[test]
fn plausible_mistakes_and_partial_repairs_fail_without_erasing_correct_case_results() {
    let mut correct = lab().initial.clone();
    for action in &lab().solution {
        engine::execute(&mut correct, action).unwrap();
    }
    for action in [
        "address Client lan 2001:db8:74:0:625:96ff:feab:cdef/64",
        "address Client lan 2001:db8:74:0:425:96ab:cdff:feef/64",
        "address Client lan 2001:db8:75:0:425:96ff:feab:cdef/64",
        "address Client lan 2001:db8:74:0:425:96ff:feab:cdef/48",
        "link Client lan down",
        "address R1 office 2001:db8:74::2/64",
        "remove-route Server 0",
    ] {
        let mut state = correct.clone();
        engine::execute(&mut state, action).unwrap();
        assert!(!solved(&state), "{action}");
    }
    let mut address_only = lab().initial.clone();
    engine::execute(&mut address_only, &lab().solution[0]).unwrap();
    let checks = engine::checks(&address_only, &lab().goal);
    assert!(checks[0].0);
    assert!(checks.iter().any(|c| !c.0));
    let mut no_flip = correct.clone();
    engine::execute(
        &mut no_flip,
        "address Client lan 2001:db8:74:0:625:96ff:feab:cdef/64",
    )
    .unwrap();
    let checks = engine::checks(&no_flip, &lab().goal);
    assert!(!checks[0].0);
    assert!(checks[0].1.contains("not toggled"));
    assert!(checks.iter().skip(1).all(|c| c.0));
    let before = serde_json::to_value(&correct).unwrap();
    assert!(engine::execute(&mut correct, "address Client lan malformed").is_err());
    assert_eq!(serde_json::to_value(&correct).unwrap(), before);
    assert!(engine::execute(&mut correct, "sudo ip addr flush dev eth0").is_err());
    assert!(solved(&correct));
}
#[test]
fn practice_worked_fill_experiment_and_reset_are_separate_from_assignment() {
    let s = lab();
    let lesson = s.ipv6_teaching.as_ref().unwrap();
    lesson.validate().unwrap();
    let initial = s.initial.clone();
    let mut learning = Learning::new(lesson);
    let old = learning.topology.clone();
    learning.fill_example(lesson).unwrap();
    assert_eq!(learning.topology, old);
    assert!(learning.checks(lesson).is_empty());
    learning.apply().unwrap();
    assert!(learning.checks(lesson).iter().all(|c| c.0));
    assert_eq!(s.initial, initial);
    learning
        .draft
        .devices
        .get_mut("Client")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = "fd64:7::16:3eff:fe12:3456/64".into();
    assert!(learning.checks(lesson).is_empty());
    learning.apply().unwrap();
    let checks = learning.checks(lesson);
    assert!(!checks[0].0);
    assert!(checks[0].1.contains("not toggled"));
    let reset = Learning::new(lesson);
    assert_eq!(reset.topology, old);
    assert!(!reset.checked);
    assert_eq!(s.initial, initial);
}
#[test]
fn private_save_resume_preserves_both_drafts_and_old_work_and_regrades_after_damage() {
    let s = lab();
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    let lesson = s.ipv6_teaching.as_ref().unwrap();
    dlms_learning_lab::ipv6_teaching::prepare(&mut a, lesson, true);
    let w = a.workspace.as_mut().unwrap();
    w.ipv6_learning
        .as_mut()
        .unwrap()
        .fill_example(lesson)
        .unwrap();
    w.ipv6_draft
        .as_mut()
        .unwrap()
        .devices
        .get_mut("Client")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = "unfinished assignment draft".into();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(ID.into(), a);
    p.favorites.insert(ID.into());
    let old = catalog()
        .iter()
        .find(|s| s.id == "ccna-ipv6-workstation")
        .unwrap();
    let mut old_attempt = Attempt::new(old, 1);
    workspace::prepare(&mut old_attempt);
    dlms_learning_lab::ipv6_teaching::prepare(
        &mut old_attempt,
        old.ipv6_teaching.as_ref().unwrap(),
        true,
    );
    p.attempts.insert(old.id.clone(), old_attempt.clone());
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".build-artifacts/continuation/resume-20261110/next-eui64/progress");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!(
        "profile-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let restored = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&restored.attempts[ID].workspace).unwrap(),
        serde_json::to_value(&p.attempts[ID].workspace).unwrap()
    );
    assert!(restored.favorites.contains(ID));
    assert_eq!(
        serde_json::to_value(&restored.attempts[&old.id]).unwrap(),
        serde_json::to_value(&old_attempt).unwrap()
    );
    assert!(!progress::current_solved(&restored.attempts[ID], s));
    let mut a = restored.attempts[ID].clone();
    let State::Ipv6 { topology } = &mut a.state else {
        panic!()
    };
    for action in &s.solution {
        dlms_learning_lab::ipv6_workspace::adapter(topology, action).unwrap();
    }
    a.workspace.as_mut().unwrap().ipv6_draft = Some(Draft::new(topology));
    assert!(progress::current_solved(&a, s));
    a.workspace
        .as_mut()
        .unwrap()
        .ipv6_draft
        .as_mut()
        .unwrap()
        .devices
        .get_mut("Client")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .address = "2001:db8:74::123/64".into();
    assert!(!progress::current_solved(&a, s));
}

#[test]
fn invalid_mac_prefix_device_or_unreachable_taught_solution_rejects_the_pack() {
    let mut valid: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/ipv6-interface-id-v1.json")).unwrap();
    // Isolate intrinsic pack validation; registered prerequisite membership is
    // independently verified against the complete373 source catalog above.
    valid["scenarios"][0]["prerequisites"] = serde_json::json!([]);
    assert!(pack::load(&valid.to_string()).is_ok());
    for (field, value) in [
        ("mac", serde_json::json!("01:25:96:ab:cd:ef")),
        ("interface", serde_json::json!("absent")),
        (
            "network",
            serde_json::json!({"address":"2001:db8:74::1","length":64}),
        ),
    ] {
        let mut bad = valid.clone();
        bad["scenarios"][0]["goal"]["requirements"][0][field] = value;
        assert!(pack::load(&bad.to_string()).is_err(), "{field}");
    }
    let mut bad = valid;
    bad["scenarios"][0]["ipv6_teaching"]["example_solution"][0] =
        serde_json::json!("address Client lan fd64:7::16:3eff:fe12:3456/64");
    assert!(
        pack::load(&bad.to_string())
            .unwrap_err()
            .contains("does not reach")
    );
}
