use dlms_learning_lab::network_transfer as work;
#[path = "support/network_transfer_fixture.rs"]
mod fixture;
use fixture::{fixture, policy, request};
#[test]
fn tftp_download_exact_block_boundaries_empty_files_and_split_utf8_are_byte_preserving() {
    for (text, expected) in [
        (String::new(), vec![0]),
        ("A".to_string(), vec![1]),
        ("A".repeat(511), vec![511]),
        ("A".repeat(512), vec![512, 0]),
        ("A".repeat(513), vec![512, 1]),
        ("A".repeat(1024), vec![512, 512, 0]),
        (format!("{}é", "A".repeat(511)), vec![512, 1]),
    ] {
        let (c, mut w) = fixture(&text);
        let before = w.clone();
        let q = request(work::Protocol::Tftp, work::Direction::Download);
        let r = work::transfer(&c, &c.network, &mut w, &q).unwrap();
        assert!(r.success, "{}", r.detail);
        assert_eq!(r.bytes, Some(text.len()));
        assert_eq!(w.files["Client"]["staged.cfg"], text);
        assert_eq!(w.files["Server"], before.files["Server"]);
        assert_eq!(w.files["Client"]["startup.cfg"], "CURRENT\n");
        let sizes: Vec<_> = r
            .events
            .iter()
            .filter(|e| e.stage == work::Stage::Data)
            .map(|e| e.packet.unwrap().payload_bytes)
            .collect();
        assert_eq!(sizes, expected);
        assert_eq!(
            r.events.first().unwrap().packet.unwrap().destination.port,
            69
        );
        for e in r.events.iter().filter(|e| e.stage == work::Stage::Data) {
            let p = e.packet.unwrap();
            assert_eq!((p.source.port, p.destination.port), (50000, 49152));
        }
        for e in r
            .events
            .iter()
            .filter(|e| e.stage == work::Stage::Acknowledgement)
        {
            let p = e.packet.unwrap();
            assert_eq!((p.source.port, p.destination.port), (49152, 50000));
        }
    }
}
#[test]
fn tftp_upload_starts_with_ack_zero_and_exact_multiple_ends_with_empty_data() {
    let (c, mut w) = fixture("unused");
    w.files
        .get_mut("Client")
        .unwrap()
        .insert("startup.cfg".into(), "x".repeat(512));
    let before = w.clone();
    let r = work::transfer(
        &c,
        &c.network,
        &mut w,
        &request(work::Protocol::Tftp, work::Direction::Upload),
    )
    .unwrap();
    assert!(r.success);
    assert_eq!(r.events[1].block, Some(0));
    assert_eq!(r.events[1].stage, work::Stage::Acknowledgement);
    let data: Vec<_> = r
        .events
        .iter()
        .filter(|e| e.stage == work::Stage::Data)
        .collect();
    assert_eq!(data.len(), 2);
    assert_eq!(data[1].packet.unwrap().payload_bytes, 0);
    assert_eq!(data[0].packet.unwrap().source.port, 49152);
    assert_eq!(data[0].packet.unwrap().destination.port, 50000);
    assert_eq!(
        w.files["Server"]["archive/site.cfg"],
        before.files["Client"]["startup.cfg"]
    );
    assert_eq!(w.files["Client"], before.files["Client"]);
}
#[test]
fn ftp_passive_connection_initiator_and_file_direction_are_separate_and_empty_files_work() {
    for direction in [work::Direction::Download, work::Direction::Upload] {
        for text in ["", "line\r\nlast\n"] {
            let (c, mut w) = fixture(text);
            if direction == work::Direction::Upload {
                w.files
                    .get_mut("Client")
                    .unwrap()
                    .insert("startup.cfg".into(), text.into());
            }
            let before = w.clone();
            let r = work::transfer(
                &c,
                &c.network,
                &mut w,
                &request(work::Protocol::Ftp, direction),
            )
            .unwrap();
            assert!(r.success, "{}", r.detail);
            assert_eq!(r.bytes, Some(text.len()));
            let syn = r
                .events
                .iter()
                .find(|e| e.stage == work::Stage::DataConnection)
                .unwrap()
                .packet
                .unwrap();
            assert_eq!((syn.source.port, syn.destination.port), (49152, 50001));
            let file = r.events.iter().find(|e| e.stage == work::Stage::Data);
            if text.is_empty() {
                assert!(file.is_none());
            } else {
                let p = file.unwrap().packet.unwrap();
                assert_eq!(p.payload_bytes, text.len());
                assert_eq!(
                    p.source.port,
                    if direction == work::Direction::Download {
                        50001
                    } else {
                        49152
                    }
                );
            }
            assert!(
                r.events
                    .iter()
                    .filter(|e| matches!(
                        e.stage,
                        work::Stage::Control | work::Stage::DataConnection
                    ))
                    .all(|e| e.packet.unwrap().payload_bytes == 0),
                "No prospective placeholder payload is sent by handshake-only reuse"
            );
            match direction {
                work::Direction::Download => {
                    assert_eq!(w.files["Client"]["staged.cfg"], text);
                    assert_eq!(w.files["Server"], before.files["Server"]);
                }
                work::Direction::Upload => {
                    assert_eq!(w.files["Server"]["archive/site.cfg"], text);
                    assert_eq!(w.files["Client"], before.files["Client"]);
                }
            }
        }
    }
}
#[test]
fn ftp_control_success_is_not_data_success_and_failure_never_creates_partial_files() {
    let (c, mut w) = fixture("recovery\n");
    let before = w.clone();
    let network = policy(
        &c,
        " 10 permit tcp host 192.0.2.10 host 198.51.100.20 eq 21",
        " 10 permit tcp host 198.51.100.20 eq 21 host 192.0.2.10 range 49152 65535",
    );
    let r = work::transfer(
        &c,
        &network,
        &mut w,
        &request(work::Protocol::Ftp, work::Direction::Upload),
    )
    .unwrap();
    assert!(!r.success);
    assert_eq!(r.stage, work::Stage::DataConnection);
    assert!(
        r.events
            .iter()
            .any(|e| e.stage == work::Stage::Login && e.delivered)
    );
    assert!(r.detail.contains("TCP port 21 alone"));
    assert_eq!(w, before);
}
#[test]
fn tftp_udp69_only_fails_at_data_and_one_way_data_fails_at_acknowledgement() {
    let (c, mut w) = fixture("recovery\n");
    let before = w.clone();
    let q = request(work::Protocol::Tftp, work::Direction::Download);
    let network = policy(
        &c,
        " 10 permit udp host 192.0.2.10 host 198.51.100.20 eq 69",
        " 10 permit udp host 198.51.100.20 eq 69 host 192.0.2.10 range 49152 65535",
    );
    let r = work::transfer(&c, &network, &mut w, &q).unwrap();
    assert!(!r.success);
    assert_eq!(r.stage, work::Stage::Data);
    assert!(r.events[0].delivered);
    assert_eq!(w, before);
    let network = policy(
        &c,
        " 10 permit udp host 192.0.2.10 host 198.51.100.20 eq 69",
        " 10 permit udp host 198.51.100.20 eq 50000 host 192.0.2.10 eq 49152",
    );
    let r = work::transfer(&c, &network, &mut w, &q).unwrap();
    assert!(!r.success);
    assert_eq!(r.stage, work::Stage::Acknowledgement);
    assert!(
        r.events
            .iter()
            .any(|e| e.stage == work::Stage::Data && e.delivered)
    );
    assert_eq!(w, before);
}
#[test]
fn ftp_login_permissions_missing_source_and_disabled_services_have_distinct_failure_stages() {
    let (c, mut w) = fixture("recovery\n");
    let before = w.clone();
    let mut q = request(work::Protocol::Ftp, work::Direction::Upload);
    q.login = None;
    assert_eq!(
        work::transfer(&c, &c.network, &mut w, &q).unwrap().stage,
        work::Stage::Login
    );
    q.login = Some("reader".into());
    let r = work::transfer(&c, &c.network, &mut w, &q).unwrap();
    assert!(!r.success && r.detail.contains("cannot write"));
    assert_eq!(r.stage, work::Stage::File);
    q.login = Some("archivist".into());
    q.local_path = "missing.cfg".into();
    let r = work::transfer(&c, &c.network, &mut w, &q).unwrap();
    assert!(!r.success && r.detail.contains("does not exist"));
    assert_eq!(w, before);
    let mut c = c;
    c.servers.get_mut("Server").unwrap().tftp = false;
    let r = work::transfer(
        &c,
        &c.network,
        &mut w,
        &request(work::Protocol::Tftp, work::Direction::Download),
    )
    .unwrap();
    assert!(!r.success && r.detail.contains("does not offer TFTP"));
    assert_eq!(w, before);
}
#[test]
fn malformed_paths_targets_and_unsupported_protocols_never_reach_a_host_or_change_stores() {
    let (c, mut w) = fixture("recovery\n");
    let before = w.clone();
    for p in [
        "",
        "../secret",
        "a/../secret",
        "/etc/passwd",
        "a//b",
        "a/./b",
        "file;touch",
        "$(id)",
        "file:///secret",
        "x\\y",
        "%2e%2e/x",
    ] {
        let mut q = request(work::Protocol::Tftp, work::Direction::Download);
        q.local_path = p.into();
        assert!(work::transfer(&c, &c.network, &mut w, &q).is_err(), "{p}");
        assert_eq!(w, before);
    }
    let mut q = request(work::Protocol::Tftp, work::Direction::Download);
    q.server = "203.0.113.4".into();
    assert!(work::transfer(&c, &c.network, &mut w, &q).is_err());
    let mut q = request(work::Protocol::Tftp, work::Direction::Download);
    q.login = Some("archivist".into());
    assert_eq!(
        work::transfer(&c, &c.network, &mut w, &q)
            .unwrap_err()
            .category,
        work::Category::Unsupported
    );
    let mut raw =
        serde_json::to_value(request(work::Protocol::Ftp, work::Direction::Download)).unwrap();
    raw["protocol"] = "sftp".into();
    assert!(serde_json::from_value::<work::Request>(raw).is_err());
    assert_eq!(w, before);
}
#[test]
fn file_bounds_and_source_held_network_shape_are_enforced_before_mutation() {
    let (c, mut w) = fixture("recovery\n");
    let before = w.clone();
    let mut changed = c.network.clone();
    changed
        .topology
        .devices
        .get_mut("Client")
        .unwrap()
        .interfaces
        .get_mut("lan")
        .unwrap()
        .up = false;
    assert!(
        work::transfer(
            &c,
            &changed,
            &mut w,
            &request(work::Protocol::Tftp, work::Direction::Download)
        )
        .is_err()
    );
    assert_eq!(w, before);
    for n in 0..work::MAX_FILES - 2 {
        w.files
            .get_mut("Client")
            .unwrap()
            .insert(format!("keep-{n}"), "x".into());
    }
    let before = w.clone();
    assert_eq!(w.files["Client"].len(), work::MAX_FILES);
    assert_eq!(
        work::transfer(
            &c,
            &c.network,
            &mut w,
            &request(work::Protocol::Tftp, work::Direction::Download)
        )
        .unwrap_err()
        .category,
        work::Category::Limit
    );
    assert_eq!(w, before);
    let (c, mut bad) = fixture("unused");
    bad.files
        .get_mut("Server")
        .unwrap()
        .insert("public/oversize".into(), "x".repeat(work::MAX_FILE + 1));
    assert!(bad.validate(&c).is_err());
    assert!(work::path("configs/site-1.cfg"));
    assert!(!work::path("configs/"));
}
