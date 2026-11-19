//! Real state transitions in a supplied virtual oneshot; never host systemd/firewall.
use dlms_learning_lab::{
    engine::{self, State as EngineState},
    linux_v6_filter::{Class, Verdict},
    linux_v6_saved_filter as file,
    linux_v6_startup::{FILE, State, Status, UNIT},
    linux_v6_state_filter::Table,
    pack,
};
const SETTINGS: &str = "net.ipv6.conf.all.forwarding=1\n";
fn fixture() -> State {
    let source = serde_json::from_str::<pack::Pack>(include_str!(
        "../scenarios/linux-v6-admin-lessons-v1.json"
    ))
    .unwrap()
    .scenarios
    .remove(0);
    let mut state = source.initial.clone();
    for command in &source.solution {
        engine::execute_for_goal(&mut state, &source.goal, command).unwrap();
    }
    let EngineState::LinuxV6Administration { simulation } = state else {
        panic!()
    };
    let mut filter = Table::new(Verdict::Drop);
    filter.command("ip6tables -P OUTPUT ACCEPT").unwrap();
    filter.command("ip6tables -A FORWARD -p tcp --dport 9443 -m conntrack --ctstate NEW,ESTABLISHED -j ACCEPT").unwrap();
    let saved_filter = file::save(&filter).unwrap();
    let s = State {
        version: 1,
        router: simulation.assignment.world,
        filter,
        saved_filter,
        enabled: false,
        status: Status::Inactive,
    };
    s.validate().unwrap();
    s
}
#[test]
fn enabling_is_distinct_from_loading_and_fresh_startup_uses_saved_rules() {
    let mut s = fixture();
    let expected = s.filter.clone();
    s.command("ip6tables -F FORWARD").unwrap();
    let live = s.filter.clone();
    assert_ne!(live, expected);
    let disabled = s.startup_trial(SETTINGS).unwrap();
    assert_eq!(disabled.filter, Table::new(Verdict::Accept));
    assert_eq!(disabled.status, Status::Inactive);
    s.command(&format!("systemctl enable {UNIT}")).unwrap();
    assert_eq!(s.filter, live);
    assert_eq!(s.status, Status::Inactive);
    let repeated = s.command(&format!("systemctl enable {UNIT}")).unwrap();
    assert!(!repeated.changed && repeated.text.is_empty());
    let boot = s.startup_trial(SETTINGS).unwrap();
    assert_eq!(boot.filter, expected);
    assert_eq!(boot.status, Status::Loaded);
    assert!(boot.router.settings.global_forwarding);
    assert_eq!(s.filter, live, "trial cannot mutate current work");
}
#[test]
fn save_does_not_enable_or_start_and_accepts_both_option_orders() {
    for options in [
        format!("-t filter -f {FILE}"),
        format!("--file {FILE} --table filter"),
    ] {
        let mut s = fixture();
        s.command("ip6tables -P FORWARD ACCEPT").unwrap();
        let text = s.command(&format!("ip6tables-save {options}")).unwrap();
        assert_eq!(text.exit, 0);
        assert_eq!(file::restore(&s.saved_filter).unwrap(), s.filter);
        assert!(!s.enabled);
        assert_eq!(s.status, Status::Inactive);
        assert_eq!(
            s.command(&format!("cat {FILE}")).unwrap().text,
            s.saved_filter
        );
    }
}
#[test]
fn start_is_oneshot_idempotent_restart_reloads_and_stop_disable_retain_rules() {
    let mut s = fixture();
    let original = s.filter.clone();
    s.command("systemctl start lab-ipv6-filter").unwrap();
    assert_eq!(s.status, Status::Loaded);
    s.command("ip6tables -F FORWARD").unwrap();
    let changed = s.filter.clone();
    s.command(&format!("systemctl start {UNIT}")).unwrap();
    assert_eq!(
        s.filter, changed,
        "active oneshot start must not pretend to reload a file"
    );
    s.command(&format!("systemctl restart {UNIT}")).unwrap();
    assert_eq!(s.filter, original);
    s.command(&format!("systemctl enable {UNIT}")).unwrap();
    s.command(&format!("systemctl disable {UNIT}")).unwrap();
    assert_eq!(s.status, Status::Loaded);
    assert!(!s.enabled);
    s.command(&format!("systemctl stop {UNIT}")).unwrap();
    assert_eq!(s.filter, original);
    assert_eq!(s.status, Status::Inactive);
}
#[test]
fn malformed_saved_data_can_resume_and_failed_loader_does_not_replace_current_table() {
    let mut s = fixture();
    let old = s.filter.clone();
    s.command(&format!("systemctl enable {UNIT}")).unwrap();
    s.save_draft("*filter\n:INPUT DROP [0:0]\n").unwrap();
    let result = s.command(&format!("systemctl restart {UNIT}")).unwrap();
    assert_eq!(result.exit, 1);
    assert!(result.text.contains("retained"));
    assert!(matches!(s.status, Status::Failed { .. }));
    assert_eq!(s.filter, old);
    assert!(s.enabled);
    let active = s.command(&format!("systemctl is-active {UNIT}")).unwrap();
    assert_eq!((active.exit, active.text.as_str()), (3, "failed\n"));
    let loaded: State = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    assert_eq!(loaded, s);
    loaded.validate().unwrap();
    let boot = s.startup_trial(SETTINGS).unwrap();
    assert!(matches!(boot.status, Status::Failed { .. }));
    assert_eq!(boot.filter, Table::new(Verdict::Accept));
}
#[test]
fn manual_restore_test_and_failure_are_atomic_and_do_not_forge_unit_status() {
    let mut s = fixture();
    let wanted = s.filter.clone();
    s.command("ip6tables -F FORWARD").unwrap();
    let before = s.clone();
    s.command(&format!("ip6tables-restore --test {FILE}"))
        .unwrap();
    assert_eq!(s, before);
    s.command(&format!("ip6tables-restore {FILE}")).unwrap();
    assert_eq!(s.filter, wanted);
    assert_eq!(s.status, Status::Inactive);
    s.save_draft("not a rules file").unwrap();
    let before = s.clone();
    assert!(s.command(&format!("ip6tables-restore {FILE}")).is_err());
    assert_eq!(s, before);
}
#[test]
fn unit_inspection_is_readable_and_successful_loader_is_not_a_policy_grade() {
    let mut s = fixture();
    assert!(
        s.command(&format!("systemctl cat {UNIT}"))
            .unwrap()
            .text
            .contains("RemainAfterExit=yes")
    );
    let disabled = s.command(&format!("systemctl is-enabled {UNIT}")).unwrap();
    assert_eq!((disabled.exit, disabled.text.as_str()), (1, "disabled\n"));
    assert!(
        s.command(&format!("systemctl status {UNIT}"))
            .unwrap()
            .text
            .contains("inactive (dead)")
    );
    let good = s.filter.clone();
    s.save_draft(&file::save(&Table::new(Verdict::Accept)).unwrap())
        .unwrap();
    s.command(&format!("systemctl restart {UNIT}")).unwrap();
    assert_eq!(s.status, Status::Loaded);
    assert_ne!(
        s.filter, good,
        "an active loader can load a bad policy; grade behavior separately"
    );
    assert!(
        s.command(&format!("systemctl status {UNIT}"))
            .unwrap()
            .text
            .contains("active (exited)")
    );
}
#[test]
fn unsupported_commands_paths_modules_and_idempotent_host_writes_leave_state_exact() {
    let mut s = fixture();
    for command in [
        "bash -c 'touch /etc/host'",
        "cat ../../etc/shadow",
        "ip6tables-save -M /bin/sh",
        "ip6tables-save -f /tmp/escape",
        "ip6tables-restore /etc/another-file",
        "systemctl start unrelated.service",
        "sysctl -w net.ipv6.conf.all.forwarding=1",
        "ip -6 route replace 2001:db8:99::/64 via fe80::1 dev wan0 metric 10",
    ] {
        let before = s.clone();
        assert!(s.command(command).is_err(), "{command}");
        assert_eq!(s, before, "{command}");
    }
    assert!(
        s.command("ip -6 route show")
            .unwrap()
            .text
            .contains("2001:db8:99::/64")
    );
}
#[test]
fn byte_caps_and_source_settings_fail_without_changing_current_work() {
    let mut s = fixture();
    let before = s.clone();
    assert_eq!(
        s.save_draft(&"x".repeat(32769)).unwrap_err().class,
        Class::Limit
    );
    assert_eq!(s, before);
    assert_eq!(s.command(&"x".repeat(513)).unwrap_err().class, Class::Limit);
    assert_eq!(s, before);
    assert!(s.startup_trial("net.ipv6.conf.unknown=1\n").is_err());
    assert_eq!(s, before);
    let mut bad = s.clone();
    bad.version = 2;
    assert!(bad.validate().is_err());
}
