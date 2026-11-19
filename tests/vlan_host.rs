use dlms_learning_lab::{nm_policy as n, switching::Mode, vlan_host};
#[path = "support/vlan_host.rs"]
mod f;
#[test]
fn correct_l3_and_real_two_direction_tags_are_both_required() {
    let env = f::environment();
    env.validate().unwrap();
    let selected = f::choose(&env, &f::files());
    for target in ["10.20.0.1", "10.20.0.50", "172.16.5.10", "10.21.0.50"] {
        let p = selected.probe(&env, f::ip(target));
        assert!(p.pass, "{target}: {}", p.reason);
        assert!(
            p.reason
                .contains("return to this software interface delivered")
        );
    }
}
#[test]
fn native_parent_has_working_outbound_but_not_a_valid_vlan_return() {
    let env = f::environment();
    let mut files = f::files();
    let text = files["reports.nmconnection"].replace("parent=enp1s0", "parent=enp2s0");
    files.insert("reports.nmconnection".into(), text);
    let s = f::choose(&env, &files);
    let p = s.probe(&env, f::ip("10.20.0.1"));
    assert!(!p.pass);
    assert!(p.reason.contains("outward to 10.20.0.1 delivered"));
    assert!(
        p.reason
            .contains("return to this software interface not delivered")
    );
    assert!(p.reason.contains("untagged frame"));
    assert!(env.vlan.as_ref().unwrap().fabric.hosts["native"].up);
    assert!(env.physical.links[0].carrier);
}
#[test]
fn valid_ip_with_wrong_tag_fails_without_faking_carrier_down() {
    let env = f::environment();
    let mut files = f::files();
    let text = files["reports.nmconnection"].replace("id=30\n", "id=40\n");
    files.insert("reports.nmconnection".into(), text);
    let s = f::choose(&env, &files);
    let p = s.probe(&env, f::ip("10.20.0.50"));
    assert!(!p.pass);
    assert!(p.reason.contains("outward to 10.20.0.50 not delivered"));
    assert!(env.physical.links[0].carrier);
}
#[test]
fn correct_tag_does_not_override_an_incorrect_ip_network() {
    let env = f::environment();
    let mut files = f::files();
    let text = files["reports.nmconnection"].replace("10.20.0.2/24", "10.22.0.2/24");
    files.insert("reports.nmconnection".into(), text);
    let p = n::Policy::from_texts_for(&env, &files).unwrap();
    if let Ok((s, _)) = n::Choices::empty().automatic(&env, &p) {
        assert!(!s.probe(&env, f::ip("10.20.0.50")).pass);
    }
}
#[test]
fn physical_parent_and_wire_failures_are_real_drops() {
    for cable in [false, true] {
        let mut env = f::environment();
        let facts = env.vlan.as_mut().unwrap();
        if cable {
            facts
                .fabric
                .cables
                .iter_mut()
                .find(|c| c.id == "link-primary")
                .unwrap()
                .up = false;
        } else {
            facts.fabric.hosts.get_mut("primary").unwrap().up = false;
        }
        let s = f::choose(&env, &f::files());
        let p = s.probe(&env, f::ip("10.20.0.1"));
        assert!(!p.pass);
        assert!(p.reason.contains("down") || p.reason.contains("stopped"));
    }
}
#[test]
fn configuration_changes_recompute_paths_and_old_snapshots_are_not_preempted() {
    let env = f::environment();
    let correct = f::files();
    let mut wrong = correct.clone();
    let text = wrong["reports.nmconnection"].replace("parent=enp1s0", "parent=enp2s0");
    wrong.insert("reports.nmconnection".into(), text);
    let initial = f::choose(&env, &wrong);
    let p = n::Policy::from_texts_for(&env, &correct).unwrap();
    let warm = initial.automatic(&env, &p).unwrap().0;
    assert_eq!(warm.for_device("reports0"), initial.for_device("reports0"));
    assert!(!warm.probe(&env, f::ip("10.20.0.1")).pass);
    let future = n::Choices::empty().automatic(&env, &p).unwrap().0;
    assert!(future.probe(&env, f::ip("10.20.0.1")).pass);
}
#[test]
fn distinct_vlans_can_share_a_parent_but_duplicate_parent_and_id_cannot() {
    let env = f::environment();
    let mut files = f::files();
    let text = files["audit.nmconnection"].replace("parent=enp2s0", "parent=enp1s0");
    files.insert("audit.nmconnection".into(), text);
    let s = f::choose(&env, &files);
    assert!(s.probe(&env, f::ip("10.21.0.50")).pass);
    assert!(s.probe(&env, f::ip("10.20.0.50")).pass);
    let text = files["audit.nmconnection"].replace("id=40\n", "id=30\n");
    files.insert("audit.nmconnection".into(), text);
    let p = n::Policy::from_texts_for(&env, &files).unwrap();
    assert!(n::Choices::empty().automatic(&env, &p).is_err());
}
#[test]
fn missing_peer_duplicate_parent_and_unsupported_attachments_are_rejected() {
    for kind in 0..4 {
        let mut env = f::environment();
        let c = env.vlan.as_mut().unwrap();
        match kind {
            0 => {
                c.peers.pop();
            }
            1 => c.parents[1] = c.parents[0].clone(),
            2 => {
                c.fabric
                    .switches
                    .get_mut("sw")
                    .unwrap()
                    .ports
                    .get_mut("primary")
                    .unwrap()
                    .mode = Mode::Access { vlan: 30 }
            }
            _ => c.version = 2,
        }
        assert!(env.validate().is_err());
    }
}
#[test]
fn closed_bounded_facts_decode_without_host_paths_or_unknown_fields() {
    let env = f::environment();
    let raw = serde_json::to_string(env.vlan.as_ref().unwrap()).unwrap();
    let c: vlan_host::Facts = serde_json::from_str(&raw).unwrap();
    c.validate(&env.physical).unwrap();
    assert!(
        serde_json::from_str::<vlan_host::Facts>(&raw.replacen("{", "{\"unknown\":true,", 1))
            .is_err()
    );
    assert!(
        serde_json::from_str::<vlan_host::Facts>(&raw.replacen("{", "{\"version\":1,", 1)).is_err()
    );
    let mut v = serde_json::to_value(&c).unwrap();
    v["parents"] = serde_json::json!([
        c.parents[0].clone(),
        c.parents[1].clone(),
        c.parents[0].clone()
    ]);
    assert!(serde_json::from_value::<vlan_host::Facts>(v).is_err());
}

#[test]
fn a_selected_vlan_snapshot_cannot_be_reused_without_its_source_capability() {
    let env = f::environment();
    let s = f::choose(&env, &f::files());
    let mut old = env.clone();
    old.vlan = None;
    let legacy = f::files()
        .into_iter()
        .map(|(n, s)| {
            (
                n,
                s.replace("type=vlan", "type=ethernet")
                    .lines()
                    .scan(false, |v, line| {
                        if line == "[vlan]" {
                            *v = true;
                            return Some(None);
                        }
                        if line.starts_with('[') {
                            *v = false;
                        }
                        Some((!*v).then_some(line))
                    })
                    .flatten()
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        })
        .collect();
    let p = n::Policy::from_texts(&legacy).unwrap();
    assert!(s.automatic(&old, &p).is_err());
}
