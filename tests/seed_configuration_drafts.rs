use dlms_learning_lab::{
    engine::{self, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    workspace,
};

fn solved(id: &str) -> (Scenario, Attempt) {
    let s = pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.id == id)
        .unwrap();
    let mut a = Attempt::new(&s, 7);
    for command in &s.solution {
        engine::execute(&mut a.state, command).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    workspace::prepare(&mut a);
    (s, a)
}

#[test]
fn missing_legacy_drafts_preserve_applied_credit_without_inventing_edits() {
    for id in ["lpic2-dns", "lpic2-storage"] {
        let (s, mut a) = solved(id);
        assert!(!workspace::pending_seed_configuration_draft(&a));
        assert!(progress::current_solved(&a, &s));
        a.workspace = None;
        assert!(!workspace::pending_seed_configuration_draft(&a));
        assert!(progress::current_solved(&a, &s));
    }
}

#[test]
fn every_visible_dns_field_invalidates_pending_credit_without_changing_served_state() {
    let (s, mut a) = solved("lpic2-dns");
    let before = a.state.clone();
    let State::Dns {
        records, serial, ..
    } = &before
    else {
        panic!()
    };
    for value in ["", "not-an-address", "192.0.2.81", " 192.0.2.80 "] {
        a.workspace.as_mut().unwrap().dns_address = Some(value.into());
        assert!(workspace::pending_seed_configuration_draft(&a));
        assert!(!progress::current_solved(&a, &s));
        assert_eq!(a.state, before);
        a.workspace.as_mut().unwrap().dns_address = records.values().next().cloned();
        assert!(progress::current_solved(&a, &s));
    }
    for value in ["", "bad", "2026102103", " 2026102102 "] {
        a.workspace.as_mut().unwrap().dns_serial = Some(value.into());
        assert!(workspace::pending_seed_configuration_draft(&a));
        assert!(!progress::current_solved(&a, &s));
        assert_eq!(a.state, before);
        a.workspace.as_mut().unwrap().dns_serial = Some(serial.to_string());
        assert!(progress::current_solved(&a, &s));
    }
    workspace::edit_dns(&mut a.state, "192.0.2.81", "2026102103").unwrap();
    let w = a.workspace.as_mut().unwrap();
    w.dns_address = Some("192.0.2.81".into());
    w.dns_serial = Some("2026102103".into());
    assert!(!workspace::pending_seed_configuration_draft(&a));
    assert!(
        !progress::current_solved(&a, &s),
        "Saving a wrong value cannot earn credit"
    );
}

#[test]
fn device_drafts_invalidate_credit_and_saved_wrong_devices_still_fail() {
    let (s, mut a) = solved("lpic2-storage");
    let before = a.state.clone();
    let State::Mount {
        configured_device, ..
    } = &before
    else {
        panic!()
    };
    for value in ["", "not-a-device", "/dev/vdz9", "/dev/../secret"] {
        a.workspace.as_mut().unwrap().mount_device = Some(value.into());
        assert!(workspace::pending_seed_configuration_draft(&a));
        assert!(!progress::current_solved(&a, &s));
        assert_eq!(a.state, before);
        a.workspace.as_mut().unwrap().mount_device = Some(configured_device.clone());
        assert!(progress::current_solved(&a, &s));
    }
    workspace::edit_mount(&mut a.state, "/dev/vdz9").unwrap();
    a.workspace.as_mut().unwrap().mount_device = Some("/dev/vdz9".into());
    assert!(!workspace::pending_seed_configuration_draft(&a));
    assert!(!progress::current_solved(&a, &s));
}

#[test]
fn pending_drafts_resume_exactly_without_rewriting_files_or_other_attempts() {
    let labs = pack::bundled().unwrap();
    let (_, mut dns) = solved("lpic2-dns");
    let (_, mut mount) = solved("lpic2-storage");
    dns.workspace.as_mut().unwrap().dns_address = Some("unfinished DNS draft".into());
    mount.workspace.as_mut().unwrap().mount_device = Some("unfinished device draft".into());
    let mut p = Progress {
        schema_version: 1,
        clock: 20,
        ..Default::default()
    };
    p.favorites.insert("ccna-ipv4".into());
    p.attempts.insert("lpic2-dns".into(), dns);
    p.attempts.insert("lpic2-storage".into(), mount);
    let unrelated = Attempt::new(&labs[0], 1);
    p.attempts.insert(labs[0].id.clone(), unrelated.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic-progress.json");
    progress::save(&path, &p).unwrap();
    let raw = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, &labs).unwrap();
    assert_eq!(std::fs::read(path).unwrap(), raw);
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    for id in ["lpic2-dns", "lpic2-storage"] {
        let s = labs.iter().find(|s| s.id == id).unwrap();
        assert!(!progress::current_solved(&loaded.attempts[id], s));
    }
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&labs[0].id]).unwrap(),
        serde_json::to_value(unrelated).unwrap()
    );
}
