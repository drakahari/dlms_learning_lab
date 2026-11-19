use dlms_learning_lab::{
    course, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
#[test]
fn current_application_catalog_extends_the_complete_legacy_catalog_without_rewriting_it() {
    let old = pack::bundled().unwrap();
    let current = pack::current().unwrap();
    assert_eq!(old.len(), 201);
    assert_eq!(current.len(), 424);
    assert_eq!(
        serde_json::to_value(&current[..old.len()]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let old_units = course::bundled(&old).unwrap();
    let current_units = course::current(&current).unwrap();
    assert_eq!(old_units.len(), 54);
    assert_eq!(current_units.len(), 163);
    for old in old_units {
        let new = current_units.iter().find(|u| u.id == old.id).unwrap();
        assert_eq!(
            serde_json::to_value(new).unwrap(),
            serde_json::to_value(old).unwrap()
        );
    }
    let source = old.iter().find(|s| s.id == "lpic1-permissions").unwrap();
    let mut attempt = Attempt::new(source, 1);
    workspace::prepare(&mut attempt);
    attempt.workspace.as_mut().unwrap().command = "unfinished terminal draft".into();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.favorites.insert(source.id.clone());
    p.attempts.insert(source.id.clone(), attempt.clone());
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".build-artifacts/continuation/resume-20261110/next-eui64/normal-profile");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!(
        "{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    progress::save_with_catalog(&path, &p, &old).unwrap();
    let upgraded = progress::load(&path, &current).unwrap();
    assert_eq!(
        serde_json::to_value(&upgraded.attempts[&source.id]).unwrap(),
        serde_json::to_value(&attempt).unwrap()
    );
    assert_eq!(upgraded.favorites, p.favorites);
    assert_eq!(upgraded.attempts.len(), 1);
    assert!(!upgraded.attempts.contains_key("ccna-ipv6-modified-eui64"));
}
