//! Normal147 registry: original143 remains exact apart from one closed approved NAT Introduction leaf.
use dlms_learning_lab::{
    course, engine, pack,
    progress::{self, Attempt, Progress},
    rpm_packages_teaching, workspace,
};
#[path = "../examples/support/automation_digest.rs"]
mod digest;
const IDS: [&str; 4] = [
    "lpic1-rpm-provider",
    "lpic1-rpm-epoch",
    "lpic1-rpm-payload",
    "lpic1-rpm-handover",
];
const INDEX_SHA: &str = "9d53aa48d601283bf35bf5940571fe92a1e934dd4ba651311e2b9188f12272bd";
const ORIGINAL_PACKS: &[(&str, &str)] = &[
    (
        "scenarios/starter-v1.json",
        include_str!("../scenarios/starter-v1.json"),
    ),
    (
        "scenarios/shell-unit-v1.json",
        include_str!("../scenarios/shell-unit-v1.json"),
    ),
    (
        "scenarios/shell-composition-v1.json",
        include_str!("../scenarios/shell-composition-v1.json"),
    ),
    (
        "scenarios/shell-reusable-v1.json",
        include_str!("../scenarios/shell-reusable-v1.json"),
    ),
    (
        "scenarios/linux-files-v1.json",
        include_str!("../scenarios/linux-files-v1.json"),
    ),
    (
        "scenarios/ipv6-static-v1.json",
        include_str!("../scenarios/ipv6-static-v1.json"),
    ),
    (
        "scenarios/shell-scope-v1.json",
        include_str!("../scenarios/shell-scope-v1.json"),
    ),
    (
        "scenarios/shell-text-v1.json",
        include_str!("../scenarios/shell-text-v1.json"),
    ),
    (
        "scenarios/shell-counts-v1.json",
        include_str!("../scenarios/shell-counts-v1.json"),
    ),
    (
        "scenarios/linux-access-v1.json",
        include_str!("../scenarios/linux-access-v1.json"),
    ),
    (
        "scenarios/vlan-data-v1.json",
        include_str!("../scenarios/vlan-data-v1.json"),
    ),
    (
        "scenarios/shell-file-batches-v1.json",
        include_str!("../scenarios/shell-file-batches-v1.json"),
    ),
    (
        "scenarios/ipv4-static-v1.json",
        include_str!("../scenarios/ipv4-static-v1.json"),
    ),
    (
        "scenarios/dns-forward-v1.json",
        include_str!("../scenarios/dns-forward-v1.json"),
    ),
    (
        "scenarios/shell-arrays-v1.json",
        include_str!("../scenarios/shell-arrays-v1.json"),
    ),
    (
        "scenarios/linux-processes-v1.json",
        include_str!("../scenarios/linux-processes-v1.json"),
    ),
    (
        "scenarios/discovery-v1.json",
        include_str!("../scenarios/discovery-v1.json"),
    ),
    (
        "scenarios/storage-v1.json",
        include_str!("../scenarios/storage-v1.json"),
    ),
    (
        "scenarios/shell-capture-v1.json",
        include_str!("../scenarios/shell-capture-v1.json"),
    ),
    (
        "scenarios/acl-policy-v1.json",
        include_str!("../scenarios/acl-policy-v1.json"),
    ),
    (
        "scenarios/accounts-lifecycle-v1.json",
        include_str!("../scenarios/accounts-lifecycle-v1.json"),
    ),
    (
        "scenarios/ospf-p2p-v1.json",
        include_str!("../scenarios/ospf-p2p-v1.json"),
    ),
    (
        "scenarios/unit-manager-v1.json",
        include_str!("../scenarios/unit-manager-v1.json"),
    ),
    (
        "scenarios/shell-case-v1.json",
        include_str!("../scenarios/shell-case-v1.json"),
    ),
    (
        "scenarios/mac-learning-v1.json",
        include_str!("../scenarios/mac-learning-v1.json"),
    ),
    (
        "scenarios/linux-find-v1.json",
        include_str!("../scenarios/linux-find-v1.json"),
    ),
    (
        "scenarios/dns-reverse-v1.json",
        include_str!("../scenarios/dns-reverse-v1.json"),
    ),
    (
        "scenarios/shell-getopts-v1.json",
        include_str!("../scenarios/shell-getopts-v1.json"),
    ),
    (
        "scenarios/dhcpv4-client-v1.json",
        include_str!("../scenarios/dhcpv4-client-v1.json"),
    ),
    (
        "scenarios/debian-packages-v1.json",
        include_str!("../scenarios/debian-packages-v1.json"),
    ),
    (
        "scenarios/spanning-tree-v1.json",
        include_str!("../scenarios/spanning-tree-v1.json"),
    ),
    (
        "scenarios/shell-safe-automation-v1.json",
        include_str!("../scenarios/shell-safe-automation-v1.json"),
    ),
    (
        "scenarios/apache-static-v1.json",
        include_str!("../scenarios/apache-static-v1.json"),
    ),
    (
        "scenarios/nat-basic-v1.json",
        include_str!("../scenarios/nat-basic-v1.json"),
    ),
];

fn index() -> serde_json::Value {
    let raw = include_bytes!("fixtures/rpm-original143-source-index.json");
    assert_eq!(digest::hash(raw), INDEX_SHA);
    serde_json::from_slice(raw).unwrap()
}
fn original_nat(raw: &str, pin: &serde_json::Value) -> Result<String, &'static str> {
    let hash = digest::hash(raw.as_bytes());
    let before = pin["before"].as_str().ok_or("missing before")?;
    let after = pin["after"].as_str().ok_or("missing after")?;
    let parsed: serde_json::Value = serde_json::from_str(raw).map_err(|_| "invalid JSON")?;
    if parsed["scenarios"][0]["id"] != "ccna-nat-static" {
        return Err("wrong exact task");
    }
    let leaf = parsed["scenarios"][0]["nat_teaching"]["concepts"][0]["text"]
        .as_str()
        .ok_or("missing exact leaf")?;
    if hash == pin["old_pack_sha256"].as_str().ok_or("missing old pin")? {
        if leaf != before {
            return Err("wrong exact old leaf");
        }
        return Ok(raw.into());
    }
    if hash != pin["new_pack_sha256"].as_str().ok_or("missing new pin")? || leaf != after {
        return Err("unknown pack or leaf");
    }
    let old = serde_json::to_string(before).unwrap();
    let new = serde_json::to_string(after).unwrap();
    if raw.matches(new.as_str()).count() != 1 {
        return Err("ambiguous amendment");
    }
    let restored = raw.replacen(new.as_str(), old.as_str(), 1);
    if digest::hash(restored.as_bytes()) != pin["old_pack_sha256"].as_str().unwrap() {
        return Err("reversal does not reproduce original bytes");
    }
    Ok(restored)
}
fn exact(a: &impl serde::Serialize, b: &impl serde::Serialize) {
    assert!(
        serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap(),
        "Complete specification/value differs"
    )
}
#[test]
fn original143_typed_fields_and36units_have_only_the_closed_nat_intro_exception() {
    let index = index();
    let pins = index["packs"].as_array().unwrap();
    assert_eq!(pins.len(), 34);
    assert_eq!(ORIGINAL_PACKS.len(), 34);
    let mut texts = Vec::new();
    for ((path, text), pin) in ORIGINAL_PACKS.iter().zip(pins) {
        assert_eq!(*path, pin["path"].as_str().unwrap());
        let original = if *path == "scenarios/nat-basic-v1.json" {
            original_nat(text, &index["nat_teaching_amendment"]).unwrap()
        } else {
            (*text).to_owned()
        };
        assert_eq!(original.len() as u64, pin["bytes"].as_u64().unwrap());
        assert_eq!(
            digest::hash(original.as_bytes()),
            pin["sha256"].as_str().unwrap()
        );
        texts.push(original);
    }
    let mut original =
        pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(original.len(), 143);
    let all = pack::bundled().unwrap();
    assert_eq!(all.len(), 164);
    let nat_raw = ORIGINAL_PACKS.last().unwrap().1;
    if digest::hash(nat_raw.as_bytes())
        == index["nat_teaching_amendment"]["new_pack_sha256"]
            .as_str()
            .unwrap()
    {
        assert_eq!(original[140].id, "ccna-nat-static");
        assert_eq!(
            original[140].nat_teaching.as_ref().unwrap().concepts[0].text,
            index["nat_teaching_amendment"]["before"].as_str().unwrap()
        );
        original[140].nat_teaching.as_mut().unwrap().concepts[0].text =
            index["nat_teaching_amendment"]["after"]
                .as_str()
                .unwrap()
                .to_owned();
    }
    exact(&&all[..143], &original);
    let ids = all[..143]
        .iter()
        .map(|s| serde_json::json!({"id":s.id,"version":s.version}))
        .collect::<Vec<_>>();
    exact(&ids, &index["catalog"]);
    assert_eq!(
        all[143..147]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [48, 36, 28, 52]
    );
    let units = course::bundled(&all).unwrap();
    assert_eq!(units.len(), 42);
    let old_units: Vec<course::Unit> =
        serde_json::from_value(index["units"]["units"].clone()).unwrap();
    assert_eq!(old_units.len(), 36);
    exact(&&units[..36], &old_units);
    assert_eq!(units[36].id, "rpm-package-investigation");
    assert_eq!(units[36].labs, IDS.map(String::from));
    assert_eq!(units[36].prerequisites, ["linux-file-foundations"]);
}
#[test]
fn closed_nat_exception_rejects_other_teaching_goal_and_unreviewed_byte_changes() {
    let index = index();
    let pin = &index["nat_teaching_amendment"];
    let raw = ORIGINAL_PACKS.last().unwrap().1;
    assert!(original_nat(raw, pin).is_ok());
    assert!(original_nat(&(raw.to_owned() + " "), pin).is_err());
    let mut changed: serde_json::Value = serde_json::from_str(raw).unwrap();
    changed["scenarios"][0]["nat_teaching"]["concepts"][0]["text"] =
        serde_json::json!("unreviewed replacement");
    assert!(original_nat(&changed.to_string(), pin).is_err());
    let mut changed: serde_json::Value = serde_json::from_str(raw).unwrap();
    changed["scenarios"][0]["objective"] = serde_json::json!("different task");
    assert!(original_nat(&changed.to_string(), pin).is_err());
}
#[test]
fn normal_rpm_prerequisites_references_and_truth_are_real_registered_scenarios() {
    let all = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for (i, s) in all[143..147].iter().enumerate() {
        assert_eq!(s.id, IDS[i]);
        assert_eq!(s.track, "LPIC-1");
        assert_eq!(s.unit, "rpm-package-investigation");
        assert_eq!(
            s.prerequisites,
            vec![if i == 0 {
                "lpic1-file-navigation".to_owned()
            } else {
                IDS[i - 1].to_owned()
            }]
        );
        assert_eq!(s.steps.len(), 3);
        assert_eq!(s.guide.len(), 3);
        let mut a = Attempt::new(s, 2);
        workspace::prepare(&mut a);
        let engine::Goal::RpmPackages { definition } = &s.goal else {
            panic!()
        };
        rpm_packages_teaching::prepare(
            &mut a,
            definition,
            s.rpm_packages_teaching.as_ref().unwrap(),
            true,
        )
        .unwrap();
        assert!(!progress::current_solved(&a, s));
        for c in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    assert_eq!(p.favorites.len(), 4);
}
#[test]
fn genuine_old_profiles_do_not_gain_new_rpm_attempts_answers_or_favorites() {
    let all = pack::bundled().unwrap();
    for (raw, n) in [
        (
            include_bytes!("fixtures/shell-v7-synthetic-writer-progress.json").as_slice(),
            102,
        ),
        (
            include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json").as_slice(),
            116,
        ),
    ] {
        let original: Progress = serde_json::from_slice(raw).unwrap();
        let loaded = progress::decode(raw, &all).unwrap();
        assert_eq!(loaded.attempts.len(), n);
        exact(&loaded.attempts, &original.attempts);
        exact(&loaded.migration_receipts, &original.migration_receipts);
        for id in IDS {
            assert!(!loaded.attempts.contains_key(id));
            assert!(!loaded.favorites.contains(id));
        }
    }
}
