//! Combined154 NFS boundary: exact original147 plus Bash3, then NFS4; raw37 retained.
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use dlms_learning_lab::{course, engine, nfs, nfs_teaching, nfs_workspace, pack, progress};
use serde_json::Value;
#[path = "support/combined_catalog.rs"]
mod combined_catalog;
#[path = "support/cron_registration.rs"]
mod cron_registration;
#[path = "../examples/support/automation_digest.rs"]
mod digest;
const INDEX_SHA: &str = "d1781b990368d1e5cecf871c2b11032659756cd67e39f6f5facc51f7998472ef";
const IDS: [&str; 4] = [
    "lpic2-nfs-reference",
    "lpic2-nfs-publication",
    "lpic2-nfs-handover",
    "lpic2-nfs-capstone",
];
fn index() -> Value {
    let text = include_str!("fixtures/nfs-original147-source-index.json");
    assert_eq!(digest::hash(text.as_bytes()), INDEX_SHA);
    serde_json::from_str(text).unwrap()
}
fn exact(a: &impl serde::Serialize, b: &impl serde::Serialize) {
    assert_eq!(
        serde_json::to_value(a).unwrap(),
        serde_json::to_value(b).unwrap()
    );
}
fn original_units(raw: &str) -> Option<String> {
    String::from_utf8(combined_catalog::old_unit_bytes(raw.as_bytes()).ok()?).ok()
}

#[test]
fn complete_old147_pack_bytes_typed_specs_order_and_raw37_units_remain_exact() {
    let i = index();
    let mut texts = Vec::new();
    for pin in i["packs"].as_array().unwrap() {
        let path = pin["path"].as_str().unwrap();
        assert!(path.starts_with("scenarios/") && !path.contains(".."));
        let text =
            std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
                .unwrap();
        assert_eq!(text.len() as u64, pin["bytes"].as_u64().unwrap());
        assert_eq!(
            digest::hash(text.as_bytes()),
            pin["sha256"].as_str().unwrap()
        );
        texts.push(text);
    }
    assert_eq!(texts.len(), 35);
    let old = pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    let all = pack::bundled().unwrap();
    assert_eq!((old.len(), one_off_boundary::before_len(&all)), (147, 176));
    exact(&all[..147].to_vec(), &old);
    exact(
        &old.iter()
            .map(|s| serde_json::json!({"id":s.id,"version":s.version}))
            .collect::<Vec<_>>(),
        &i["catalog"],
    );
    assert_eq!(
        all[150..154]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    let declared: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/nfs-v1.json")).unwrap();
    exact(&all[150..154].to_vec(), &declared.scenarios);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [53, 40, 32, 55]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all[..176].iter().filter(|s| s.track == t).count()),
        [53, 36, 32, 55]
    );
    let current_raw = include_str!("../scenarios/units-v1.json");
    let raw39 = cron_registration::previous_units(
        &cron_registration::private_cron_units(current_raw.as_bytes()).unwrap(),
    )
    .unwrap();
    let raw = std::str::from_utf8(&raw39).unwrap();
    let reconstructed = original_units(raw).unwrap();
    exact(
        &serde_json::from_str::<Value>(&reconstructed).unwrap(),
        &i["units"],
    );
    let original = course::load(&reconstructed, &old).unwrap();
    let units = course::bundled(&all).unwrap();
    assert_eq!(
        (original.len(), one_off_boundary::before_len(&units)),
        (37, 46)
    );
    exact(&units[..37].to_vec(), &original);
    exact(&units[38], &i["appended_unit"]);
    assert_eq!(
        units[38].prerequisites,
        ["persistent-mounts", "linux-file-access"]
    );
}
#[test]
fn raw_unit_reconstruction_accepts_only_the_closed_append_and_unchanged_prefix() {
    let current_raw = include_str!("../scenarios/units-v1.json");
    let raw39 = cron_registration::previous_units(
        &cron_registration::private_cron_units(current_raw.as_bytes()).unwrap(),
    )
    .unwrap();
    let raw = std::str::from_utf8(&raw39).unwrap();
    let original = original_units(raw).unwrap();
    assert!(original_units(&original).is_none());
    assert!(original_units(&(raw.to_owned() + " ")).is_none());
    assert!(original_units(&raw.replacen("Shared files:", "Changed files:", 1)).is_none());
    assert!(original_units(&raw.replacen("IPv4", "IPvX", 1)).is_none());
    let i = index();
    // raw is already the exact original39 projection above. Apply the
    // original missing-NFS mutation to those bytes, not a second current inverse.
    let bash_only = format!(
        "{}{}",
        raw.strip_suffix(i["unit_new_suffix"].as_str().unwrap())
            .unwrap(),
        i["unit_old_tail"].as_str().unwrap()
    );
    assert!(
        original_units(&bash_only).is_none(),
        "Missing final NFS append must fail"
    );
    let nfs_only = format!(
        "{}{}",
        original
            .strip_suffix(i["unit_old_tail"].as_str().unwrap())
            .unwrap(),
        i["unit_new_suffix"].as_str().unwrap()
    );
    assert!(
        original_units(&nfs_only).is_none(),
        "Missing Bash intermediate must fail"
    );
    let mut reordered: Value = serde_json::from_str(raw).unwrap();
    reordered["units"].as_array_mut().unwrap().swap(37, 38);
    assert!(
        original_units(&serde_json::to_string_pretty(&reordered).unwrap()).is_none(),
        "Swapping reviewed unit order must fail"
    );
}
#[test]
fn four_registered_assignments_have_real_prerequisites_and_current_outcome_grading() {
    let all = pack::bundled().unwrap();
    for (n, s) in all[150..154].iter().enumerate() {
        assert_eq!(s.id, IDS[n]);
        assert_eq!(
            s.prerequisites,
            if n == 0 {
                vec![
                    "lpic2-mount-policy".to_owned(),
                    "lpic1-traversal".to_owned(),
                ]
            } else {
                vec![IDS[n - 1].to_owned()]
            }
        );
        let mut a = progress::Attempt::new(s, 1);
        nfs_teaching::prepare(&mut a, s, true).unwrap();
        assert!(!progress::current_solved(&a, s));
        for text in &s.solution {
            let action =
                nfs::Action::from_json(text.strip_prefix(nfs_workspace::PREFIX).unwrap()).unwrap();
            nfs_workspace::perform_attempt(&mut a, s, &action).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        assert!(matches!(a.state, engine::State::Nfs { .. }));
    }
}
#[test]
fn unchanged_old_profiles_never_gain_new_nfs_answers_attempts_or_favorites() {
    let all = pack::bundled().unwrap();
    let raw = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let before: progress::Progress = serde_json::from_slice(raw).unwrap();
    let loaded = progress::decode(raw, &all).unwrap();
    exact(&loaded.attempts, &before.attempts);
    exact(&loaded.favorites, &before.favorites);
    assert!(
        IDS.iter()
            .all(|id| !loaded.attempts.contains_key(*id) && !loaded.favorites.contains(*id))
    );
}
