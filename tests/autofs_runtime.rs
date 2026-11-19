//! Demand mounts inspect actual virtual filesystem identity/data, never a host daemon.
#[path = "support/autofs_fixture.rs"]
mod fixture;
use dlms_learning_lab::{
    autofs::{maps::ErrorKind, runtime::*},
    lvm,
};
use fixture::*;
use serde_json::{Value, json};
fn assert_unchanged(
    a: &mut State,
    c: &Context,
    s: &lvm::State,
    f: impl FnOnce(&mut State) -> Result<(), dlms_learning_lab::autofs::maps::Error>,
) -> String {
    let before = a.clone();
    let e = f(a).unwrap_err();
    assert_eq!(*a, before);
    a.validate(c, s).unwrap();
    e.to_string()
}
#[test]
fn first_lookup_demand_mounts_actual_filesystem_not_eager_startup() {
    for is_direct in [false, true] {
        let (s, c, mut a) = fixture(is_direct);
        assert!(a.mounted.storage.live().is_empty());
        assert!(a.leases.is_empty());
        let path = if is_direct { DIRECT_REPORT } else { REPORT };
        let r = a.read(&c, &s, path).unwrap();
        assert_eq!(r.content, s.read_file("team/draft", "/keep.txt").unwrap());
        assert_eq!(
            r.original_volume,
            s.identity.as_ref().unwrap().volumes["team/draft"]
        );
        assert_eq!(a.leases.len(), 1);
        assert_eq!(a.mounted.storage.live().len(), 1);
        assert_eq!(a.clock, 0);
    }
}
#[test]
fn indirect_edits_apply_to_new_lookups_without_reload_but_not_existing_mounts() {
    let (s, c, mut a) = fixture(false);
    let first = a.read(&c, &s, REPORT).unwrap();
    let loaded = a.loaded.clone();
    a.save(&c, &s, INDIRECT, indirect("/dev/team/reference"))
        .unwrap();
    assert_eq!(a.loaded, loaded);
    assert_eq!(a.read(&c, &s, REPORT).unwrap(), first);
    assert_eq!(a.advance(&c, &s, 29).unwrap(), Vec::<String>::new());
    assert_eq!(a.advance(&c, &s, 1).unwrap(), vec!["/srv/teams/report"]);
    let second = a.read(&c, &s, REPORT).unwrap();
    assert_eq!(
        second.content,
        s.read_file("team/reference", "/keep.txt").unwrap()
    );
    assert_ne!(second.original_volume, first.original_volume);
    assert_eq!(a.loaded, loaded);
}
#[test]
fn direct_file_save_needs_refresh_and_existing_association_survives_refresh() {
    let (s, c, mut a) = fixture(true);
    let first = a.read(&c, &s, DIRECT_REPORT).unwrap();
    a.save(&c, &s, DIRECT, direct("/dev/team/reference"))
        .unwrap();
    a.advance(&c, &s, 30).unwrap();
    assert_eq!(a.read(&c, &s, DIRECT_REPORT).unwrap(), first); // save alone did not refresh direct map
    a.reload(&c, &s).unwrap();
    assert_eq!(a.read(&c, &s, DIRECT_REPORT).unwrap(), first); // already mounted
    a.advance(&c, &s, 30).unwrap();
    let second = a.read(&c, &s, DIRECT_REPORT).unwrap();
    assert_ne!(first.original_volume, second.original_volume);
    assert_eq!(
        second.content,
        s.read_file("team/reference", "/keep.txt").unwrap()
    );
}
#[test]
fn saved_master_is_separate_from_active_master_and_bad_refresh_is_atomic() {
    let (s, c, mut a) = fixture(false);
    a.save(&c, &s, MASTER, String::new()).unwrap();
    a.reload(&c, &s).unwrap();
    assert_eq!(a.read(&c, &s, REPORT).unwrap_err().kind, ErrorKind::Lookup);
    assert!(a.leases.is_empty());
    a.save(&c, &s, MASTER, config(false).master.0).unwrap();
    assert!(a.read(&c, &s, REPORT).is_err());
    a.reload(&c, &s).unwrap();
    a.read(&c, &s, REPORT).unwrap();
    a.save(
        &c,
        &s,
        MASTER,
        "/srv/teams program:/etc/auto.teams --timeout=30".into(),
    )
    .unwrap();
    let e = assert_unchanged(&mut a, &c, &s, |a| a.reload(&c, &s));
    assert!(e.contains("program"));
    assert!(a.read(&c, &s, REPORT).is_ok());
}
#[test]
fn settled_idle_checks_respect_last_access_and_each_independent_mount() {
    let (s, c, mut a) = fixture(false);
    a.read(&c, &s, REPORT).unwrap();
    a.advance(&c, &s, 20).unwrap();
    a.read(&c, &s, REPORT).unwrap();
    a.read(&c, &s, "/srv/teams/audit/keep.txt").unwrap();
    a.advance(&c, &s, 20).unwrap();
    a.read(&c, &s, REPORT).unwrap();
    assert_eq!(a.advance(&c, &s, 10).unwrap(), vec!["/srv/teams/audit"]);
    assert!(a.mounted.storage.live().contains_key("/srv/teams/report"));
    assert_eq!(a.advance(&c, &s, 20).unwrap(), vec!["/srv/teams/report"]);
    assert!(a.mounted.storage.live().is_empty());
}
#[test]
fn missing_file_still_mounts_backing_filesystem_but_invalid_source_does_not() {
    let (s, c, mut a) = fixture(false);
    assert_eq!(
        a.read(&c, &s, "/srv/teams/report/missing.txt")
            .unwrap_err()
            .kind,
        ErrorKind::Lookup
    );
    assert_eq!(a.leases.len(), 1);
    assert!(a.mounted.storage.live().contains_key("/srv/teams/report"));
    a.advance(&c, &s, 30).unwrap();
    a.save(&c, &s, INDIRECT, indirect("/dev/missing/volume"))
        .unwrap();
    assert_unchanged(&mut a, &c, &s, |a| a.read(&c, &s, REPORT).map(|_| ()));
    assert!(a.mounted.storage.live().is_empty());
}
#[test]
fn inactive_source_fails_without_inventing_data_or_a_live_mount() {
    let (mut s, c, mut a) = fixture(false);
    s.activation.as_mut().unwrap().open.clear();
    s.execute("vgchange -an team").unwrap();
    let e = assert_unchanged(&mut a, &c, &s, |a| a.read(&c, &s, REPORT).map(|_| ()));
    assert!(e.contains("inactive"));
    s.execute("vgchange -ay team").unwrap();
    assert!(a.read(&c, &s, REPORT).is_ok());
}
#[test]
fn malformed_saved_indirect_map_diagnoses_next_new_lookup_and_can_be_repaired() {
    let (s, c, mut a) = fixture(false);
    a.save(
        &c,
        &s,
        INDIRECT,
        "report -fstype=ext4 /dev/team/draft".into(),
    )
    .unwrap();
    let e = assert_unchanged(&mut a, &c, &s, |a| a.read(&c, &s, REPORT).map(|_| ()));
    assert!(e.contains("colon"));
    assert!(e.contains(INDIRECT));
    assert!(e.contains("Line 1"));
    a.save(&c, &s, INDIRECT, indirect("/dev/team/draft"))
        .unwrap();
    assert!(a.read(&c, &s, REPORT).is_ok());
}
#[test]
fn removed_indirect_key_does_not_break_existing_mount_but_fails_after_expiry() {
    let (s, c, mut a) = fixture(false);
    a.read(&c, &s, REPORT).unwrap();
    a.save(&c, &s, INDIRECT, String::new()).unwrap();
    assert!(a.read(&c, &s, REPORT).is_ok());
    a.advance(&c, &s, 30).unwrap();
    assert_unchanged(&mut a, &c, &s, |a| a.read(&c, &s, REPORT).map(|_| ()));
}
#[test]
fn changing_trigger_structure_while_live_has_honest_subset_diagnostic() {
    let (s, c, mut a) = fixture(true);
    a.read(&c, &s, DIRECT_REPORT).unwrap();
    a.save(&c, &s, MASTER, config(true).master.0.replace("30", "60"))
        .unwrap();
    let e = assert_unchanged(&mut a, &c, &s, |a| a.reload(&c, &s));
    assert!(e.contains("not a universal Linux reload rule"));
    a.advance(&c, &s, 30).unwrap();
    a.reload(&c, &s).unwrap();
    assert_eq!(a.loaded.roots(&c).unwrap()[0].timeout, 60);
}
#[test]
fn read_only_map_options_are_actual_mount_options_and_equivalent_forms_work() {
    let (s, c, base) = fixture(false);
    let mut outputs = vec![];
    for text in [
        "report -fstype=ext4,ro :/dev/team/draft",
        "# comment\n report\t-ro,fstype=ext4\t:/dev/team/draft\r\n",
    ] {
        let mut a = base.clone();
        a.save(&c, &s, INDIRECT, text.into()).unwrap();
        let r = a.read(&c, &s, REPORT).unwrap();
        assert!(a.mounted.storage.live()["/srv/teams/report"].read_only);
        outputs.push((r, a.mounted.storage.live().clone()));
    }
    assert_eq!(outputs[0], outputs[1]);
}
#[test]
fn clock_limits_and_invalid_reads_do_not_mutate_state_or_reach_host_paths() {
    let (s, c, mut a) = fixture(false);
    for n in [0, 3601, u32::MAX] {
        assert_unchanged(&mut a, &c, &s, |a| a.advance(&c, &s, n).map(|_| ()));
    }
    for path in [
        "/etc/passwd",
        "/srv/teams/report/../keep.txt",
        "/srv/teams/report//keep.txt",
        "/srv/teams/report/keep.txt;id",
        "/srv/teams/report/$(id)",
        "/srv/teams/report/'keep.txt'",
        "/srv/teams/report/keep.txt\0",
    ] {
        let e = assert_unchanged(&mut a, &c, &s, |a| a.read(&c, &s, path).map(|_| ()));
        assert!(!e.is_empty());
    }
    for _ in 0..24 {
        a.advance(&c, &s, 3600).unwrap();
    }
    assert_eq!(a.clock, MAX_CLOCK);
    assert_unchanged(&mut a, &c, &s, |a| a.advance(&c, &s, 1).map(|_| ()));
}
#[test]
fn reset_and_round_trip_preserve_actual_distinct_saved_loaded_and_live_state() {
    let (s, c, mut a) = fixture(true);
    let initial = a.clone();
    a.read(&c, &s, DIRECT_REPORT).unwrap();
    a.save(&c, &s, DIRECT, direct("/dev/team/reference"))
        .unwrap();
    a.advance(&c, &s, 7).unwrap();
    let loaded = a.loaded.clone();
    let raw = serde_json::to_string(&a).unwrap();
    let mut restored: State = serde_json::from_str(&raw).unwrap();
    restored.validate(&c, &s).unwrap();
    assert_eq!(restored, a);
    assert_eq!(restored.loaded, loaded);
    assert!(
        restored
            .read(&c, &s, DIRECT_REPORT)
            .unwrap()
            .content
            .starts_with("team/draft:")
    );
    let reset = State::new(&c, &s, &config(true), config(true)).unwrap();
    assert_eq!(reset, initial);
    assert_ne!(reset, restored);
}
#[test]
fn invalid_serialized_state_rosters_versions_leases_and_bounds_are_rejected() {
    let (s, c, mut a) = fixture(false);
    a.read(&c, &s, REPORT).unwrap();
    let good = serde_json::to_value(&a).unwrap();
    for (path, val) in [
        ("version", json!(2)),
        ("clock", json!(86401)),
        ("leases", json!([])),
    ] {
        let mut v = good.clone();
        v[path] = val;
        let result = serde_json::from_value::<State>(v);
        assert!(result.is_err() || result.unwrap().validate(&c, &s).is_err());
    }
    let mut v = good.clone();
    v["leases"][0]["root"] = json!("/another");
    assert!(
        serde_json::from_value::<State>(v)
            .unwrap()
            .validate(&c, &s)
            .is_err()
    );
    let mut v = good.clone();
    v["leases"][0]["last_used"] = json!(1);
    assert!(
        serde_json::from_value::<State>(v)
            .unwrap()
            .validate(&c, &s)
            .is_err()
    );
    let mut v = good.clone();
    v["unexpected"] = json!("ignored?");
    assert!(serde_json::from_value::<State>(v).is_err());
    let mut v = good.clone();
    v["saved"]["master"] = json!("x".repeat(4097));
    assert!(serde_json::from_value::<State>(v).is_err());
    let mut v = good.clone();
    v["leases"] = Value::Array(vec![v["leases"][0].clone(); 5]);
    assert!(serde_json::from_value::<State>(v).is_err());
    let mut b = a.clone();
    b.saved.maps.swap(0, 1);
    b.validate(&c, &s).unwrap();
    b.saved.maps[0] = b.saved.maps[1].clone();
    assert!(b.validate(&c, &s).is_err());
}
#[test]
fn ambiguous_direct_indirect_namespace_is_rejected_before_loading() {
    let (s, c, _) = fixture(false);
    let mut cfg = config(false);
    cfg.master
        .0
        .push_str(&format!("/- {DIRECT} --timeout=30\n"));
    cfg.maps
        .iter_mut()
        .find(|f| f.name.0 == DIRECT)
        .unwrap()
        .text
        .0 = "/srv/teams/report -fstype=ext4 :/dev/team/draft".into();
    assert_eq!(
        State::new(&c, &s, &cfg, cfg.clone()).unwrap_err().kind,
        ErrorKind::Unsupported
    );
}
#[test]
fn oversized_or_rebound_context_never_admits_foreign_world_files() {
    let (s, c, a) = fixture(false);
    let mut wrong = c.clone();
    wrong.inventory.filesystems[0].original_volume = u32::MAX;
    assert!(a.validate(&wrong, &s).is_err());
    let mut wrong = c.clone();
    wrong.map_files.push(Name(EXTRA.into()));
    assert!(a.validate(&wrong, &s).is_err());
    let mut wrong = c.clone();
    wrong.inventory.mountpoints[0].0 = "/srv/teams/../report".into();
    assert!(a.validate(&wrong, &s).is_err());
    let mut wrong = c.clone();
    wrong.inventory.filesystems[0].uuid = wrong.inventory.filesystems[1].uuid.clone();
    assert!(a.validate(&wrong, &s).is_err());
}
