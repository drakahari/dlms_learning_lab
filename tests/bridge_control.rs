//! Bounded bridge startup dependencies/current snapshots; no host activation.
#[path = "support/bridge_control.rs"]
mod fixture;
use dlms_learning_lab::{bridge_control as b, bridge_fabric::Interface, nm_profiles::ErrorKind};
fn selected() -> b::Current {
    b::Current {
        controller: true,
        members: vec![Interface("enp1s0".into()), Interface("enp2s0".into())],
    }
}
#[test]
fn forced_bridge_ports_start_together_despite_their_own_false_autoconnect() {
    let plan = b::Plan::from_files(&fixture::source(), &fixture::files()).unwrap();
    assert!(plan.controller().stp);
    assert!(plan.members().all(|(_, p)| !p.autoconnect));
    assert_eq!(plan.automatic().unwrap(), selected());
    let snapshot = b::Snapshot::automatic(plan).unwrap();
    assert_eq!(snapshot.projection().unwrap().members, selected().members);
}
#[test]
fn unbound_candidate_is_not_a_port_and_half_a_binding_is_unsupported() {
    let mut files = fixture::files();
    let r = files.get_mut("right.nmconnection").unwrap();
    *r = r.replace("controller=br0\nport-type=bridge\n", "").replace(
        "[bridge-port]",
        "[ipv4]\nmethod=disabled\n[ipv6]\nmethod=disabled",
    );
    let plan = b::Plan::from_files(&fixture::source(), &files).unwrap();
    assert_eq!(
        plan.automatic().unwrap().members,
        vec![Interface("enp1s0".into())]
    );
    assert!(plan.selected(&selected()).is_err());
    files
        .get_mut("right.nmconnection")
        .unwrap()
        .push_str("\n[connection]\ncontroller=br0\n");
    assert_eq!(
        b::Plan::from_files(&fixture::source(), &files)
            .unwrap_err()
            .kind,
        ErrorKind::Unsupported
    );
}
#[test]
fn independent_port_startup_never_silently_becomes_a_missing_path() {
    let mut files = fixture::files();
    files
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[connection]\nautoconnect-ports=0\n");
    let plan = b::Plan::from_files(&fixture::source(), &files).unwrap();
    assert!(plan.automatic().unwrap().members.is_empty());
    files
        .get_mut("right.nmconnection")
        .unwrap()
        .push_str("\n[connection]\nautoconnect=true\n");
    let err = b::Plan::from_files(&fixture::source(), &files)
        .unwrap()
        .automatic()
        .unwrap_err();
    assert_eq!(err.kind, ErrorKind::Unsupported);
    assert!(err.message.contains("Independent"));
}
#[test]
fn forced_true_port_flags_are_also_valid_and_disabled_controller_cannot_invent_members() {
    let mut files = fixture::files();
    for n in ["left.nmconnection", "right.nmconnection"] {
        files
            .get_mut(n)
            .unwrap()
            .push_str("\n[connection]\nautoconnect=true\n");
    }
    assert_eq!(
        b::Plan::from_files(&fixture::source(), &files)
            .unwrap()
            .automatic()
            .unwrap(),
        selected()
    );
    let mut files = fixture::files();
    files
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[connection]\nautoconnect=false\n");
    let plan = b::Plan::from_files(&fixture::source(), &files).unwrap();
    assert_eq!(
        plan.automatic().unwrap(),
        b::Current {
            controller: false,
            members: vec![]
        }
    );
    assert!(
        plan.selected(&b::Current {
            controller: false,
            members: selected().members
        })
        .is_err()
    );
}
#[test]
fn owned_current_plan_and_members_do_not_follow_edited_future_files() {
    let source = fixture::source();
    let mut files = fixture::files();
    let current =
        b::Snapshot::supplied(b::Plan::from_files(&source, &files).unwrap(), selected()).unwrap();
    let old = current.clone();
    files
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[connection]\nautoconnect-ports=0\n");
    let future = b::Snapshot::automatic(b::Plan::from_files(&source, &files).unwrap()).unwrap();
    assert!(future.current.members.is_empty());
    assert_eq!(current, old);
    assert_eq!(current.current, selected());
    assert_ne!(current.plan, future.plan);
}
#[test]
fn equivalent_text_and_normalized_port_groups_preserve_semantic_signatures() {
    let source = fixture::source();
    let a = b::Plan::from_files(&source, &fixture::files()).unwrap();
    let mut files = fixture::files();
    for name in ["left.nmconnection", "right.nmconnection"] {
        let text = files.get_mut(name).unwrap();
        *text = text.replace("type=ethernet", "type=802-3-ethernet");
        text.push_str("\n[ipv4]\nmethod=disabled\n[ipv6]\nmethod=disabled\naddr-gen-mode=default\n# same configuration\n");
    }
    files
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[bridge]\nstp=true\nvlan-filtering=false\n");
    let z = b::Plan::from_files(&source, &files).unwrap();
    assert_eq!(a, z);
    assert_eq!(
        a.profile("right.nmconnection"),
        z.profile("right.nmconnection")
    );
    assert!(a.profile("unknown.nmconnection").is_none());
}
#[test]
fn source_identity_foreign_controller_and_nondefault_port_settings_fail() {
    for (name, old, new) in [
        (
            "bridge.nmconnection",
            "interface-name=br0",
            "interface-name=other",
        ),
        (
            "left.nmconnection",
            "uuid=30000000-0000-4000-8000-000000000002",
            "uuid=30000000-0000-4000-8000-999999999999",
        ),
        (
            "right.nmconnection",
            "controller=br0",
            "controller=some-other",
        ),
        ("right.nmconnection", "port-type=bridge", "port-type=bond"),
        (
            "right.nmconnection",
            "[bridge-port]",
            "[bridge-port]\nhairpin-mode=true",
        ),
        (
            "right.nmconnection",
            "[bridge-port]",
            "[ipv4]\nmethod=manual\naddress1=10.220.0.8/24",
        ),
    ] {
        let mut files = fixture::files();
        *files.get_mut(name).unwrap() = files[name].replace(old, new);
        assert!(
            b::Plan::from_files(&fixture::source(), &files).is_err(),
            "{name}/{new}"
        );
    }
}
#[test]
fn current_names_and_closed_bounded_sources_are_checked() {
    let plan = b::Plan::from_files(&fixture::source(), &fixture::files()).unwrap();
    for members in [
        vec![Interface("foreign".into())],
        vec![Interface("enp1s0".into()); 2],
        vec![Interface("enp1s0".into()); 3],
    ] {
        assert!(
            b::Snapshot::supplied(
                plan.clone(),
                b::Current {
                    controller: true,
                    members
                }
            )
            .is_err()
        );
    }
    let mut s = fixture::source();
    s.members[1] = s.members[0].clone();
    assert!(s.validate().is_err());
    let text = serde_json::to_string(&fixture::source()).unwrap();
    assert_eq!(
        serde_json::from_str::<b::Source>(&text).unwrap(),
        fixture::source()
    );
    assert!(
        serde_json::from_str::<b::Source>(
            &text.replace("\"version\":1", "\"version\":1,\"exec\":\"sh\"")
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<b::Current>(
            r#"{"controller":true,"members":["enp1s0","enp2s0","third"]}"#
        )
        .is_err()
    );
}
#[test]
fn control_file_limits_comments_and_path_boundaries_are_real_failures() {
    let source = fixture::source();
    for tail in [
        "\n# hidden\0command".into(),
        "\n".repeat(257),
        "#".repeat(16384),
        "\n[connection]\ncontroller=br\\0\n".into(),
        "\n[connection]\nautoconnect-ports=1\n".into(),
    ] {
        let mut files = fixture::files();
        files.get_mut("left.nmconnection").unwrap().push_str(&tail);
        assert!(b::Plan::from_files(&source, &files).is_err());
    }
    let mut files = fixture::files();
    files.insert("../../real.nmconnection".into(), "unused".into());
    assert!(b::Plan::from_files(&source, &files).is_err());
    let mut files = fixture::files();
    files.remove("left.nmconnection");
    assert!(b::Plan::from_files(&source, &files).is_err());
}
