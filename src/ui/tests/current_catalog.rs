use super::*;
#[test]
fn normal_sessions_select_the_complete_catalog_without_a_review_flag() {
    for pilot in [false, true] {
        for stage in [None, Some("unrelated"), Some("eui64-batch")] {
            assert!(LabApp::current_catalog_for_session(false, pilot, stage));
        }
    }
    assert!(!LabApp::current_catalog_for_session(true, false, None));
    assert!(LabApp::current_catalog_for_session(
        true,
        false,
        Some("eui64-batch")
    ));
    assert!(LabApp::current_catalog_for_session(true, true, None));
    assert!(LabApp::current_catalog_for_session(
        true,
        false,
        Some("leaf-spine-batch")
    ));
    assert!(LabApp::current_catalog_for_session(
        true,
        false,
        Some("network-transfers-batch")
    ));
    assert!(LabApp::current_catalog_for_session(
        true,
        false,
        Some("wlan-profile-batch")
    ));
}
#[test]
fn normal_catalog_ui_retains_private_test_progress_and_all_current_units() {
    let ctx = egui::Context::default();
    let current = LabApp::current_catalog_for_session(false, false, None);
    // Exercise that exact normal catalog selection with synthetic storage.
    // A unit test must never lock/read/write the owner's actual normal profile.
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, current);
    a.smoke = false;
    assert!(a._smoke_profile.is_some());
    assert_eq!(a.scenarios.len(), 543);
    assert_eq!(a.units.len(), 224);
    assert!(a.progress.attempts.is_empty() && a.progress.favorites.is_empty());
    for id in [
        "lpic2-smart-short-diagnosis",
        "lpic2-smart-monitor-identity",
        "lpic2-iscsi-target-identity",
        "lpic2-iscsi-startup-policy",
        "lpic2-pxe-firmware-selection",
        "lpic2-pxelinux-client-lookup",
        "lpic2-dkms-kernel-recovery",
        "lpic2-dkms-version-handover",
        "lpic2-initramfs-root-drivers",
        "lpic2-initramfs-kernel-handover",
        "lpic2-kernel-root-build",
        "lpic2-kernel-staged-handover",
        "ccna-poe-compatible-ports",
        "ccna-poe-budget-recovery",
        "lpic2-auth-filter-detection",
        "lpic2-auth-filter-exceptions",
        "lpic2-vendor-backport",
        "lpic2-vendor-patch-set",
        "lpic2-vpn-peer-profile",
        "lpic2-vpn-lan-return",
        "lpic2-exposure-client-policy",
        "lpic2-exposure-listener-retirement",
        "lpic2-ipv6-control-header-policy",
        "lpic2-ipv6-tracked-service-policy",
        "lpic2-router-local-transit-policy",
        "lpic2-router-forwarding-recovery",
        "lpic2-router-saved-policy-recovery",
        "lpic2-dovecot-secure-access",
        "lpic2-imap-mailbox-recovery",
        "lpic2-pop-transaction-recovery",
        "lpic2-sieve-vacation-safety",
        "lpic2-sieve-vacation-routing-recovery",
        "lpic2-smtp-destination-role",
        "lpic2-smtp-relay-boundary",
        "lpic2-smtp-queue-recovery",
        "lpic1-guest-driver-migration",
        "lpic1-guest-object-recovery",
        "lpic1-guest-image-handover",
        "lpic2-smb-identity-access",
        "lpic2-smb-team-policy",
        "lpic2-smb-change-handover",
        "ccna-leaf-spine-rack-expansion",
        "ccna-tftp-recovery-copy",
        "ccna-ftp-archive-channel",
        "ccna-transfer-file-handover",
        "lpic1-modal-setting-repair",
        "lpic1-modal-record-maintenance",
        "lpic1-modal-recovery",
        "lpic1-editor-handover",
        "ccna-json-port-records",
        "ccna-json-inventory-handover",
        "ccna-ipv6-modified-eui64",
        "lpic1-grub-default-fallback",
        "lpic1-grub-recovery-entry",
        "lpic1-grub-persistent-default",
        "lpic1-grub-argument-scope",
        "lpic2-ldap-schema-consumer",
        "shell-log-owner-triage",
        "lpic2-capacity-trend-plan",
        "lpic2-capacity-import-handover",
        "lpic2-maintenance-login-notices",
        "lpic2-maintenance-restart-handover",
        "lpic1-locale-report-handover",
        "lpic1-encoding-import-repair",
        "lpic1-timezone-system-display",
        "lpic1-timezone-process-override",
        "ccna-wireless-neighbour-channels",
        "ccna-wireless-multisurvey-plan",
        "ccna-wlan-ssid-repair",
        "ccna-wlan-wpa2-personal",
        "ccna-wlan-wpa3-handover",
        "ccna-wlc-campus-forwarding",
        "ccna-flexconnect-local-vlan",
        "ccna-snmp-link-monitor",
        "ccna-snmp-counter-monitor",
        "lpic2-proxy-client-boundary",
        "lpic2-proxy-connect-order",
        "lpic2-cache-shared-privacy",
        "lpic2-cache-freshness-handover",
        "lpic2-withdraw-obsolete-nfs-export",
        "ccna-route-reading-backup",
        "ccna-ospf-broadcast-handover",
    ] {
        let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
        a.launch(i);
        draw(&ctx, &mut a);
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
    }
}

#[test]
fn retained_json_capture_opens_both_real_tasks_against_the_exact_current_catalog() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    for id in ["ccna-json-port-records", "ccna-json-inventory-handover"] {
        app.open_json_inventory_capture(id);
        let i = app.selected.expect("capture must open the real lab");
        assert_eq!(app.scenarios[i].id, id);
        assert!(matches!(
            app.progress.attempts[id].state,
            State::JsonInventory { .. }
        ));
        assert!(!progress::current_solved(
            &app.progress.attempts[id],
            &app.scenarios[i]
        ));
    }
}

#[test]
fn source_capture_stage_routes_to_the_real_catalog_and_uses_its_actual_opener() {
    let ctx = egui::Context::default();
    let current = LabApp::current_catalog_for_session(true, false, Some("source-install-batch"));
    assert!(
        current,
        "New source capture must not select the closed legacy201catalog"
    );
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, current);
    assert!(app._smoke_profile.is_some());
    app.smoke = false;
    for id in [
        "lpic2-source-project-build",
        "lpic2-source-stage-install",
        "lpic2-source-stale-handover",
    ] {
        app.open_source_install_capture(id);
        let i = app.selected.unwrap();
        assert_eq!(app.scenarios[i].id, id);
        assert!(matches!(
            app.progress.attempts[id].state,
            State::SourceInstall { .. }
        ));
        assert!(!progress::current_solved(
            &app.progress.attempts[id],
            &app.scenarios[i]
        ));
    }
}

#[test]
fn rest_capture_opens_each_actual_task_with_the_expanded_normal_catalog() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    assert_eq!((app.scenarios.len(), app.units.len()), (543, 224));
    for id in ["ccna-rest-change-request", "ccna-rest-profile-handover"] {
        app.open_rest_controller_capture(id);
        let i = app
            .selected
            .expect("capture opens the actual registered task");
        assert_eq!(app.scenarios[i].id, id);
        assert!(matches!(
            app.progress.attempts[id].state,
            State::RestController { .. }
        ));
        assert!(!progress::current_solved(
            &app.progress.attempts[id],
            &app.scenarios[i]
        ));
    }
}
