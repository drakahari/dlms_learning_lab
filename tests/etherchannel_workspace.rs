//! Independent adapter, teaching and persistence closure. Runtime is Root-only.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    etherchannel as core, etherchannel_teaching as teach, etherchannel_workspace as work, pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};
use serde_json::{Value, json};
fn original_catalog_prefix(all: &[pack::Scenario]) {
    const ORIGINAL154: &[&str] = &[
        "ccna-ipv4",
        "ccna-vlsm",
        "ccna-gateway",
        "lpic1-permissions",
        "lpic1-traversal",
        "lpic1-logs",
        "lpic2-service",
        "lpic2-dns",
        "lpic2-storage",
        "shell-handover",
        "shell-arguments",
        "shell-input",
        "shell-validation",
        "shell-report",
        "shell-loop",
        "shell-capstone",
        "shell-streams",
        "shell-pipelines",
        "shell-eof",
        "shell-batch-audit",
        "shell-functions",
        "shell-return",
        "shell-shift",
        "shell-bundle",
        "lpic1-file-navigation",
        "lpic1-file-names",
        "lpic1-copy-move",
        "lpic1-file-cleanup",
        "lpic1-creation-mask",
        "lpic1-file-handover",
        "ccna-ipv6-neighbor",
        "ccna-ipv6-workstation",
        "ccna-ipv6-scope",
        "ccna-ipv6-specific-route",
        "ccna-ipv6-backup",
        "ccna-ipv6-handover",
        "shell-local-heading",
        "shell-scope-review",
        "shell-scope-capstone",
        "shell-cut-inventory",
        "shell-sort-manifest",
        "shell-uniq-counts",
        "shell-text-capstone",
        "shell-number-records",
        "shell-order-total",
        "shell-pack-items",
        "shell-count-capstone",
        "lpic1-team-brief",
        "lpic1-drop-directory",
        "lpic1-update-existing",
        "lpic1-retire-handover",
        "ccna-vlan-access",
        "ccna-vlan-trunk",
        "ccna-vlan-native",
        "ccna-vlan-handover",
        "shell-file-manifest",
        "shell-collect-errors",
        "shell-file-batch-capstone",
        "ccna-ipv4-network-route",
        "ccna-ipv4-return-route",
        "ccna-ipv4-specific-route",
        "ccna-ipv4-host-exception",
        "ccna-ipv4-floating-route",
        "ccna-ipv4-branch-handover",
        "lpic2-dns-dualstack",
        "lpic2-dns-aliases",
        "lpic2-dns-mail",
        "lpic2-dns-rrsets",
        "lpic2-dns-secondary",
        "lpic2-dns-handover",
        "shell-array-intake",
        "shell-array-priority",
        "shell-array-package",
        "shell-array-handover",
        "lpic1-process-pause",
        "lpic1-process-pending",
        "lpic1-process-priority",
        "lpic1-process-handover",
        "ccna-lldp-receive",
        "ccna-lldp-inventory",
        "ccna-lldp-transmit-policy",
        "ccna-lldp-handover",
        "lpic2-mount-identity",
        "lpic2-mount-policy",
        "lpic2-mount-manual",
        "lpic2-mount-handover",
        "shell-capture-preview",
        "shell-capture-evidence",
        "shell-capture-decision",
        "shell-capture-handover",
        "ccna-acl-source-policy",
        "ccna-acl-service-order",
        "ccna-acl-direction-repair",
        "ccna-acl-policy-handover",
        "lpic1-account-onboarding",
        "lpic1-account-membership",
        "lpic1-account-retirement",
        "lpic1-account-team-handover",
        "ccna-ospf-router-identity",
        "ccna-ospf-p2p-parameters",
        "ccna-ospf-directed-ecmp",
        "ccna-ospf-passive-handover",
        "lpic2-unit-loaded-recovery",
        "lpic2-unit-preparation-order",
        "lpic2-unit-optional-wants",
        "lpic2-unit-target-handover",
        "shell-case-interface",
        "shell-case-routing",
        "shell-case-publication",
        "ccna-mac-learn-paths",
        "ccna-mac-stale-location",
        "ccna-mac-source-refresh",
        "ccna-mac-handover",
        "lpic1-search-release-set",
        "lpic1-search-incomplete-intake",
        "lpic1-search-private-handover",
        "lpic2-dns-reverse-investigation",
        "lpic2-dns-reverse-ipv6",
        "lpic2-dns-reverse-handover",
        "shell-options-report",
        "shell-options-jobs",
        "shell-options-publication",
        "ccna-dhcp-held-options",
        "ccna-dhcp-relay-scope",
        "ccna-dhcp-lease-handover",
        "lpic1-package-install",
        "lpic1-package-metadata",
        "lpic1-package-payload",
        "lpic1-package-handover",
        "ccna-stp-root",
        "ccna-stp-local-cost",
        "ccna-stp-parallel",
        "ccna-stp-handover",
        "shell-preserve-lines",
        "shell-pipeline-status",
        "shell-safe-handover",
        "lpic2-web-routing",
        "lpic2-web-pages",
        "lpic2-web-access",
        "lpic2-web-handover",
        "ccna-nat-static",
        "ccna-nat-pool",
        "ccna-nat-handover",
        "lpic1-rpm-provider",
        "lpic1-rpm-epoch",
        "lpic1-rpm-payload",
        "lpic1-rpm-handover",
        "shell-command-options",
        "shell-command-dispatch",
        "shell-command-plan",
        "lpic2-nfs-reference",
        "lpic2-nfs-publication",
        "lpic2-nfs-handover",
        "lpic2-nfs-capstone",
    ];
    assert_eq!(ORIGINAL154.len(), 154);
    assert!(all.len() >= ORIGINAL154.len());
    assert_eq!(
        all[..ORIGINAL154.len()]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        ORIGINAL154
    );
}
fn fixture(n: usize) -> Value {
    let v: Value =
        serde_json::from_str(include_str!("fixtures/etherchannel-workbench-v1.json")).unwrap();
    v["cases"][n].clone()
}
fn scenario(n: usize) -> pack::Scenario {
    let c = fixture(n);
    let solution = c["reference"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| format!("{}{}", work::PREFIX, serde_json::to_string(a).unwrap()))
        .collect::<Vec<_>>();
    let v = json!({"schema_version":1,"scenarios":[{"id":format!("lacp-technical-{n}"),"version":1,"track":"CCNA","topic":"L2 LACP technical closure","unit":"","title":"Unregistered LACP workbench closure","summary":"Supplied member and VLAN outcomes","objective":"Repair the target bundle and both data VLANs; preserve the named working settings","retry_guidance":"Inspect member exclusion and VLAN admission separately","difficulty":"Intermediate","minutes":20,"steps":["Inspect supplied groups","Edit and apply configuration","Check member and VLAN outcomes"],"guide":[],"explanation":"This technical fixture is not a registered curriculum assignment","hints":["Active and Passive are distinct roles","Local consistency and VLAN service are separate"],"sources":["https://1.ieee802.org/tsn/802-1ax-rev/"],"simplifications":"Static bounded L2 LACP eligibility and logical VLAN admission only; no packets, timer or routed channel","initial":{"kind":"EtherChannel","simulation":c["definition"]["initial"]},"goal":{"kind":"EtherChannel","definition":c["definition"]},"solution":solution,"prerequisites":[],"etherchannel_teaching":c["lesson"]}]});
    pack::load(&v.to_string()).unwrap().remove(0)
}
fn sim(a: &Attempt) -> &core::State {
    let State::EtherChannel { simulation } = &a.state else {
        panic!()
    };
    simulation
}
fn def(s: &pack::Scenario) -> &work::Definition {
    let Goal::EtherChannel { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn exact<T: serde::Serialize>(a: &T, b: &Value) {
    assert!(
        serde_json::to_value(a).unwrap() == *b,
        "complete caller changed"
    );
}
fn prepared(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 9);
    teach::prepare(&mut a, s, true).unwrap();
    a
}
fn actions(n: usize, key: &str) -> Vec<core::Action> {
    serde_json::from_value(fixture(n)[key].clone()).unwrap()
}
fn apply(a: &mut Attempt, s: &pack::Scenario, action: &core::Action) {
    let core::Action::Configure { configuration } = action;
    let draft = work::Draft::from_configuration(configuration, sim(a).topology()).unwrap();
    work::set_assignment_draft(a, s, &draft).unwrap();
    work::apply_assignment(a, s).unwrap();
}
fn vector(a: &Attempt, s: &pack::Scenario) -> Vec<bool> {
    progress::current_etherchannel_checks(a, s)
        .iter()
        .map(|r| r.0)
        .collect()
}
fn profile(s: &pack::Scenario, a: Attempt) -> Progress {
    let mut p = Progress {
        schema_version: 1,
        clock: 9,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    p
}
fn fill_attempt(a: &mut Attempt) {
    let mut lo = 0usize;
    let mut hi = work::MAX_ATTEMPT;
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        a.transcript = vec!["x".repeat(mid)];
        if serde_json::to_vec(a).unwrap().len() <= work::MAX_ATTEMPT {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    a.transcript = vec!["x".repeat(lo)];
    assert_eq!(serde_json::to_vec(a).unwrap().len(), work::MAX_ATTEMPT);
}
#[test]
fn three_contexts_initial_wrong_partial_references_two_alternates_and_broken_again_match_literal_vectors()
 {
    for n in 0..3 {
        let s = scenario(n);
        let c = fixture(n);
        let mut a = prepared(&s);
        let initial: Vec<bool> = serde_json::from_value(c["initial_expected"].clone()).unwrap();
        assert_eq!(vector(&a, &s), initial);
        assert!(!progress::current_solved(&a, &s));
        for (plan, expected) in [("wrong", "wrong_expected"), ("partial", "partial_expected")] {
            let mut wrong = prepared(&s);
            for action in actions(n, plan) {
                apply(&mut wrong, &s, &action);
            }
            let v: Vec<bool> = serde_json::from_value(c[expected].clone()).unwrap();
            assert_eq!(vector(&wrong, &s), v);
            assert!(!progress::current_solved(&wrong, &s));
        }
        for action in actions(n, "reference") {
            apply(&mut a, &s, &action);
        }
        assert!(progress::current_solved(&a, &s));
        let before = serde_json::to_value(&a).unwrap();
        let rows = progress::current_etherchannel_checks(&a, &s);
        assert!(rows.iter().all(|r| r.0 && r.1.len() <= 512));
        exact(&a, &before);
        for action in actions(n, "adverse") {
            apply(&mut a, &s, &action);
        }
        let v: Vec<bool> = serde_json::from_value(c["adverse_expected"].clone()).unwrap();
        assert_eq!(vector(&a, &s), v);
        assert!(!progress::current_solved(&a, &s));
        for action in actions(n, "reference") {
            apply(&mut a, &s, &action);
        }
        assert!(progress::current_solved(&a, &s));
        let alternatives: Vec<Vec<core::Action>> =
            serde_json::from_value(c["alternatives"].clone()).unwrap();
        assert_eq!(alternatives.len(), 2);
        for plan in alternatives {
            let mut alternate = prepared(&s);
            for a in plan {
                apply(&mut alternate, &s, &a);
            }
            assert!(progress::current_solved(&alternate, &s));
        }
    }
}
#[test]
fn same_pair_same_source_two_vlan_anchors_and_real_member_domains_are_required() {
    let s = scenario(2);
    let original = def(&s);
    original.validate().unwrap();
    let mut wrong = original.clone();
    wrong
        .requirements
        .retain(|r| !matches!(r,work::Requirement::VlanService{query,..}if query=="data-two"));
    assert!(wrong.validate().is_err());
    let mut wrong = original.clone();
    let work::Requirement::BundlePair { source, .. } = &mut wrong.requirements[0] else {
        panic!()
    };
    *source = work::Source::Fault {
        id: "loss-1".into(),
    };
    assert!(wrong.validate().is_err());
    let mut wrong = original.clone();
    wrong.queries[0].query.a.group = "task2-keep".into();
    wrong.queries[0].query.b.group = "task2-keep".into();
    assert!(
        wrong.validate().is_err(),
        "healthy neighbor cannot stand in for target service"
    );
    let mut wrong = original.clone();
    wrong.faults[0].id = "unused-loss".into();
    assert!(wrong.validate().is_err());
    let mut wrong = original.clone();
    wrong.queries.push(wrong.queries[0].clone());
    assert!(wrong.validate().is_err());
    let mut wrong = original.clone();
    let work::Requirement::BundlePair { cables, .. } = &mut wrong.requirements[0] else {
        panic!()
    };
    cables[0] = "task2-link7".into(); // Same devices, but not a target member; outcome must still fail domain.
    assert!(wrong.validate().is_ok());
    let mut a = prepared(&s);
    for action in actions(2, "reference") {
        apply(&mut a, &s, &action);
    }
    assert!(!wrong.checks(sim(&a))[1].0);
    let before = serde_json::to_value(sim(&a)).unwrap();
    let rows = original.checks(sim(&a));
    assert!(rows.iter().all(|r| r.0));
    exact(sim(&a), &before);
}
#[test]
fn one_ready_link_is_not_two_member_success_and_vlan_truth_is_independent() {
    let s = scenario(0);
    let mut a = prepared(&s);
    for action in actions(0, "partial") {
        apply(&mut a, &s, &action);
    }
    let snap = sim(&a).snapshot().unwrap();
    assert!(
        snap.groups()
            .iter()
            .any(|g| matches!(g.status, core::GroupStatus::Ready { .. })
                && g.eligible_members.len() == 1)
    );
    assert!(!progress::current_solved(&a, &s));
    for action in actions(0, "wrong") {
        apply(&mut a, &s, &action);
    }
    let v = vector(&a, &s);
    assert!(v[1] && v[2] && !v[3]);
    let mut draft = a
        .workspace
        .as_ref()
        .unwrap()
        .etherchannel_draft
        .as_ref()
        .unwrap()
        .clone();
    draft.devices.get_mut("task0-left").unwrap().vlans.clear();
    work::set_assignment_draft(&mut a, &s, &draft).unwrap();
    work::apply_assignment(&mut a, &s).unwrap();
    let rows = progress::current_etherchannel_checks(&a, &s);
    assert!(rows[1].0 && !rows[2].0 && rows[2].1.contains("not active"));
}
#[test]
fn raw_inactive_values_and_selectors_do_not_break_current_truth_but_pending_active_values_do() {
    let s = scenario(0);
    let mut a = prepared(&s);
    for action in actions(0, "reference") {
        apply(&mut a, &s, &action);
    }
    let state = serde_json::to_value(&a.state).unwrap();
    let mut d = a
        .workspace
        .as_ref()
        .unwrap()
        .etherchannel_draft
        .as_ref()
        .unwrap()
        .clone();
    let p = &mut d
        .devices
        .get_mut("task0-left")
        .unwrap()
        .ports
        .get_mut("p1")
        .unwrap()
        .profile;
    p.access = "bad".into();
    work::set_assignment_draft(&mut a, &s, &d).unwrap();
    assert!(progress::current_solved(&a, &s));
    exact(&a.state, &state);
    d.selected_device = "task0-right".into();
    d.selected_group = "task0-target".into();
    d.selected_service = "data-two".into();
    work::set_assignment_draft(&mut a, &s, &d).unwrap();
    assert!(progress::current_solved(&a, &s));
    d.devices
        .get_mut("task0-right")
        .unwrap()
        .ports
        .get_mut("p2")
        .unwrap()
        .profile
        .native = "unfinished".into();
    assert!(!d.valid()); // Per-field8 byte frontier precedes aggregate.
    d.devices
        .get_mut("task0-right")
        .unwrap()
        .ports
        .get_mut("p2")
        .unwrap()
        .profile
        .native = "?".into();
    work::set_assignment_draft(&mut a, &s, &d).unwrap();
    assert!(!progress::current_solved(&a, &s));
    let before = serde_json::to_value(&a).unwrap();
    assert!(work::apply_assignment(&mut a, &s).is_err());
    exact(&a, &before);
    d.devices
        .get_mut("task0-right")
        .unwrap()
        .ports
        .get_mut("p2")
        .unwrap()
        .profile
        .native = "1".into();
    work::set_assignment_draft(&mut a, &s, &d).unwrap();
    work::apply_assignment(&mut a, &s).unwrap();
    assert!(progress::current_solved(&a, &s));
}
#[test]
fn separate_fill_apply_experiment_repair_and_reset_never_publish_assignment_answers() {
    for n in 0..3 {
        let s = scenario(n);
        let mut a = prepared(&s);
        let assignment = serde_json::to_value(&a.state).unwrap();
        let source = s.etherchannel_teaching.as_ref().unwrap();
        let before = serde_json::to_value(
            &a.workspace
                .as_ref()
                .unwrap()
                .etherchannel_learning
                .as_ref()
                .unwrap()
                .simulation,
        )
        .unwrap();
        work::fill_practice(&mut a, &s).unwrap();
        exact(
            &a.workspace
                .as_ref()
                .unwrap()
                .etherchannel_learning
                .as_ref()
                .unwrap()
                .simulation,
            &before,
        );
        assert!(!progress::current_solved(&a, &s));
        work::apply_practice(&mut a, &s).unwrap();
        assert!(
            a.workspace
                .as_ref()
                .unwrap()
                .etherchannel_learning
                .as_ref()
                .unwrap()
                .checks(source)
                .iter()
                .all(|r| r.0)
        );
        for action in &source.try_change.actions {
            let core::Action::Configure { configuration } = action;
            let l = a
                .workspace
                .as_ref()
                .unwrap()
                .etherchannel_learning
                .as_ref()
                .unwrap();
            let d =
                work::Draft::from_configuration(configuration, l.simulation.topology()).unwrap();
            work::set_practice_draft(&mut a, &s, &d).unwrap();
            work::apply_practice(&mut a, &s).unwrap();
        }
        assert_eq!(
            a.workspace
                .as_ref()
                .unwrap()
                .etherchannel_learning
                .as_ref()
                .unwrap()
                .checks(source)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            source.try_change.expected
        );
        work::fill_practice(&mut a, &s).unwrap();
        work::apply_practice(&mut a, &s).unwrap();
        assert!(
            a.workspace
                .as_ref()
                .unwrap()
                .etherchannel_learning
                .as_ref()
                .unwrap()
                .checks(source)
                .iter()
                .all(|r| r.0)
        );
        exact(&a.state, &assignment);
        work::reset_practice(&mut a, &s).unwrap();
        assert_eq!(
            a.workspace
                .as_ref()
                .unwrap()
                .etherchannel_learning
                .as_ref()
                .unwrap()
                .simulation,
            source.definition.initial
        );
        assert_eq!(
            a.workspace
                .as_ref()
                .unwrap()
                .etherchannel_learning
                .as_ref()
                .unwrap()
                .view,
            View::Example
        );
        exact(&a.state, &assignment);
    }
}
#[test]
fn full_source_initial_topology_environment_lesson_and_version_are_pinned() {
    let s = scenario(0);
    let mut a = prepared(&s);
    for action in actions(0, "reference") {
        apply(&mut a, &s, &action);
    }
    let before = serde_json::to_value(&a).unwrap();
    let mut wrong = s.clone();
    wrong.version += 1;
    assert!(!progress::current_solved(&a, &wrong));
    assert!(teach::prepare(&mut a, &wrong, false).is_err());
    exact(&a, &before);
    let mut wrong = s.clone();
    wrong.etherchannel_teaching = scenario(1).etherchannel_teaching;
    assert!(!progress::current_solved(&a, &wrong));
    assert!(work::fill_practice(&mut a, &wrong).is_err());
    exact(&a, &before);
    let mut wrong = serde_json::to_value(&s).unwrap();
    wrong["initial"]["simulation"]["environment"]["cables"]["task0-link1"] = json!(false);
    let wrong: pack::Scenario = serde_json::from_value(wrong).unwrap();
    assert!(!progress::current_solved(&a, &wrong));
    let mut bad_state = serde_json::to_value(&a.state).unwrap();
    bad_state["simulation"]["environment"]["cables"]["task0-link1"] = json!(false);
    let mut forged = a.clone();
    forged.state = serde_json::from_value(bad_state).unwrap();
    assert!(!progress::current_solved(&forged, &s));
    let mut wrong = s.clone();
    wrong
        .etherchannel_teaching
        .as_mut()
        .unwrap()
        .definition
        .queries[0]
        .query
        .vlan = 30;
    assert!(!progress::current_solved(&a, &wrong));
    assert!(teach::prepare(&mut a, &wrong, false).is_err());
    exact(&a, &before);
}
#[test]
fn malformed_packs_and_strict_bounded_wire_do_not_become_success() {
    let s = scenario(0);
    let d = a_draft(&s);
    let mut raw = serde_json::to_value(&d).unwrap();
    raw["unknown"] = json!(0);
    assert!(work::Draft::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
    raw = serde_json::to_value(&d).unwrap();
    raw["version"] = json!(2);
    assert!(work::Draft::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
    let mut raw = serde_json::to_value(def(&s)).unwrap();
    raw["requirements"] = json!(vec![raw["requirements"][0].clone(); 33]);
    assert!(work::Definition::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
    let source = s.etherchannel_teaching.as_ref().unwrap();
    let mut raw = serde_json::to_value(source).unwrap();
    raw["example_actions"] = json!([]);
    assert!(teach::Lesson::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
    let mut wrong = s.clone();
    wrong.guide = vec![pack::LessonStep {
        title: "Bad command teaching".into(),
        why: "Should remain native".into(),
        action: "Edit the table".into(),
        commands: vec![pack::CommandExample {
            command: "sim-lacp {}".into(),
            explanation: "Internal bridge must not be taught".into(),
            simulator_only: true,
        }],
    }];
    assert!(pack::load(&json!({"schema_version":1,"scenarios":[wrong]}).to_string()).is_err());
    assert!(work::Draft::from_json(&vec![b' '; work::MAX_DRAFT + 1]).is_err());
    assert!(work::Definition::from_json(&vec![b' '; work::MAX_DEFINITION + 1]).is_err());
    assert!(teach::Lesson::from_json(&vec![b' '; teach::MAX_LESSON + 1]).is_err());
}
fn a_draft(s: &pack::Scenario) -> work::Draft {
    work::Draft::from_state(&def(s).initial).unwrap()
}
#[test]
fn whole_attempt_escaped_budget_and_digit_growth_refuse_atomically_without_erasing_drafts() {
    let s = scenario(0);
    let mut low = prepared(&s);
    let mut high = low.clone();
    work::set_practice_presentation(&mut high, &s, Some(View::Example), None, None).unwrap();
    fill_attempt(&mut high);
    let before = serde_json::to_value(&high).unwrap();
    let mut proposed = a_draft(&s);
    proposed
        .devices
        .get_mut("task0-left")
        .unwrap()
        .ports
        .get_mut("p1")
        .unwrap()
        .profile
        .access = "\u{1}".repeat(8);
    assert!(proposed.valid());
    work::set_assignment_draft(&mut low, &s, &proposed).unwrap();
    assert!(work::set_assignment_draft(&mut high, &s, &proposed).is_err());
    exact(&high, &before);
    work::record_visit(&mut low, &s, 10).unwrap();
    assert!(work::record_visit(&mut high, &s, 10).is_err());
    exact(&high, &before);
    work::record_visit(&mut high, &s, 8).unwrap();
    let mut expected = before.clone();
    expected["visited"] = json!(8);
    exact(&high, &expected);
    let before = serde_json::to_value(&high).unwrap();
    assert!(
        work::set_practice_presentation(&mut high, &s, Some(View::Assignment), None, None).is_err()
    );
    exact(&high, &before);
    let mut a = Attempt::new(&s, 9);
    let before = serde_json::to_value(&a).unwrap();
    assert!(work::reset_practice(&mut a, &s).is_err());
    exact(&a, &before);
}
#[test]
fn current_and_saved_progress_roundtrip_preserves_complete_raw_work_and_genuine_records() {
    let mut all = pack::bundled().unwrap();
    original_catalog_prefix(&all);
    let old_count = all.len();
    assert_eq!(
        all.iter()
            .filter(|s| matches!(s.initial, State::EtherChannel { .. }))
            .count(),
        3
    );
    let s = scenario(0);
    all.push(s.clone());
    let mut a = prepared(&s);
    for action in actions(0, "reference") {
        apply(&mut a, &s, &action);
    }
    let record: Value = serde_json::from_str(include_str!(
        "fixtures/shell-v8-synthetic-writer-progress.json"
    ))
    .unwrap();
    let r = record["attempts"]
        .as_object()
        .unwrap()
        .values()
        .find_map(|a| (!a["state"]["run"].is_null()).then(|| a["state"]["run"].clone()))
        .unwrap();
    let r: dlms_learning_lab::shell::Record = serde_json::from_value(r).unwrap();
    assert_eq!(r.engine_revision, 8);
    a.workspace.as_mut().unwrap().script_history = Some(r);
    let mut draft = a
        .workspace
        .as_ref()
        .unwrap()
        .etherchannel_draft
        .as_ref()
        .unwrap()
        .clone();
    draft
        .devices
        .get_mut("task0-left")
        .unwrap()
        .ports
        .get_mut("p1")
        .unwrap()
        .profile
        .access = "?".into();
    work::set_assignment_draft(&mut a, &s, &draft).unwrap();
    work::set_practice_presentation(&mut a, &s, Some(View::Example), None, Some(true)).unwrap();
    assert!(progress::current_solved(&a, &s));
    let before = serde_json::to_value(&a).unwrap();
    let p = profile(&s, a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.json");
    progress::save(&path, &p).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let decoded = progress::decode(&bytes, &all).unwrap();
    exact(&decoded.attempts[&s.id], &before);
    let loaded = progress::load(&path, &all).unwrap();
    exact(&loaded.attempts[&s.id], &before);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    for old in all[..old_count]
        .iter()
        .filter(|s| !matches!(s.initial, State::EtherChannel { .. }))
    {
        let a = Attempt::new(old, 1);
        let raw = serde_json::to_value(&a).unwrap();
        assert!(raw.get("etherchannel_draft").is_none());
        let raw = serde_json::to_value(old).unwrap();
        assert!(raw.get("etherchannel_teaching").is_none());
    }
}
#[test]
fn internal_action_bridge_is_goal_aware_not_a_learner_host_command() {
    let s = scenario(0);
    let mut state = s.initial.clone();
    let before = serde_json::to_value(&state).unwrap();
    assert!(engine::execute(&mut state, &s.solution[0]).is_err());
    exact(&state, &before);
    assert!(
        engine::execute_for_goal(
            &mut state,
            &s.goal,
            &format!("{}{}", work::PREFIX, " ".repeat(core::MAX_ACTION_BYTES + 1))
        )
        .is_err()
    );
    exact(&state, &before);
    engine::execute_for_goal(&mut state, &s.goal, &s.solution[0]).unwrap();
    assert!(engine::checks(&state, &s.goal).iter().all(|r| r.0));
}

fn invoke_legacy(
    site: usize,
    a: &mut Attempt,
    s: &pack::Scenario,
    new: bool,
) -> Result<(), String> {
    match site {
        0 => {
            dlms_learning_lab::accounts_teaching::prepare(
                a,
                s.accounts_teaching.as_ref().unwrap(),
                new,
            );
            Ok(())
        }
        1 => dlms_learning_lab::acl_teaching::prepare(a, s.acl_teaching.as_ref().unwrap(), new),
        2 => {
            dlms_learning_lab::address_teaching::prepare(
                a,
                s.address_teaching.as_ref().unwrap(),
                new,
            );
            Ok(())
        }
        3 => dlms_learning_lab::apache_static_teaching::prepare(
            a,
            s.apache_static_teaching.as_ref().unwrap(),
        ),
        4 => {
            dlms_learning_lab::client_teaching::prepare(a, s.client_teaching.as_ref().unwrap(), new)
        }
        5 => dlms_learning_lab::debian_packages_teaching::prepare(
            a,
            s.debian_packages_teaching.as_ref().unwrap(),
            new,
        ),
        6 => {
            dlms_learning_lab::dhcpv4_teaching::prepare(a, s.dhcpv4_teaching.as_ref().unwrap(), new)
        }
        7 => {
            dlms_learning_lab::dns_teaching::prepare(a, s.dns_teaching.as_ref().unwrap(), new);
            Ok(())
        }
        8 => {
            dlms_learning_lab::ipv4_routing_teaching::prepare(
                a,
                s.ipv4_routing_teaching.as_ref().unwrap(),
                new,
            );
            Ok(())
        }
        9 => {
            dlms_learning_lab::ipv6_teaching::prepare(a, s.ipv6_teaching.as_ref().unwrap(), new);
            Ok(())
        }
        10 => dlms_learning_lab::legacy_access_teaching::prepare(
            a,
            s.legacy_access_teaching.as_ref().unwrap(),
            new,
        ),
        11 => dlms_learning_lab::legacy_logs_teaching::prepare(
            a,
            s.legacy_logs_teaching.as_ref().unwrap(),
            new,
        ),
        12 => {
            dlms_learning_lab::mac_learning_teaching::prepare(
                a,
                s.mac_teaching.as_ref().unwrap(),
                new,
            );
            Ok(())
        }
        13 => dlms_learning_lab::nat_teaching::prepare(a, s.nat_teaching.as_ref().unwrap(), new),
        14 => dlms_learning_lab::ospf_teaching::prepare(a, s.ospf_teaching.as_ref().unwrap(), new),
        15 => {
            dlms_learning_lab::process_teaching::prepare(
                a,
                s.process_teaching.as_ref().unwrap(),
                new,
            );
            Ok(())
        }
        16 => dlms_learning_lab::rpm_packages_teaching::prepare(
            a,
            match &s.goal {
                dlms_learning_lab::engine::Goal::RpmPackages { definition } => definition.as_ref(),
                _ => panic!(),
            },
            s.rpm_packages_teaching.as_ref().unwrap(),
            new,
        ),
        17 => {
            dlms_learning_lab::script_teaching::prepare(a, s.teaching.as_ref().unwrap(), new);
            Ok(())
        }
        18 => dlms_learning_lab::seed_dns_teaching::prepare(
            a,
            s.seed_dns_teaching.as_ref().unwrap(),
            new,
        ),
        19 => dlms_learning_lab::seed_mount_teaching::prepare(
            a,
            s.seed_mount_teaching.as_ref().unwrap(),
            new,
        ),
        20 => {
            dlms_learning_lab::service_teaching::prepare(
                a,
                s.service_teaching.as_ref().unwrap(),
                new,
            );
            Ok(())
        }
        21 => dlms_learning_lab::spanning_tree_teaching::prepare(
            a,
            s.spanning_tree_teaching.as_ref().unwrap(),
        ),
        22 => {
            dlms_learning_lab::storage_teaching::prepare(
                a,
                s.storage_teaching.as_ref().unwrap(),
                new,
            );
            Ok(())
        }
        23 => {
            dlms_learning_lab::switching_teaching::prepare(
                a,
                s.switching_teaching.as_ref().unwrap(),
                new,
            );
            Ok(())
        }
        24 => {
            dlms_learning_lab::terminal_teaching::prepare(
                a,
                s.terminal_teaching.as_ref().unwrap(),
                new,
            );
            Ok(())
        }
        25 => dlms_learning_lab::unit_manager_teaching::prepare(
            a,
            s.unit_manager_teaching.as_ref().unwrap(),
            new,
        ),
        26 => {
            workspace::prepare(a);
            Ok(())
        }
        _ => panic!("unknown closed prepare site"),
    }
}

#[test]
fn every_legacy_prepare_keeps_full_caller_exact_with_valid_new_family_fields() {
    let all = pack::bundled().unwrap();
    let source = scenario(0);
    let draft = a_draft(&source);
    let learning =
        teach::Learning::new(source.etherchannel_teaching.as_ref().unwrap(), true).unwrap();
    let sites: Vec<(usize, &pack::Scenario, bool)> = vec![
        (
            0,
            all.iter().find(|s| s.accounts_teaching.is_some()).unwrap(),
            false,
        ),
        (
            1,
            all.iter().find(|s| s.acl_teaching.is_some()).unwrap(),
            true,
        ),
        (
            2,
            all.iter().find(|s| s.address_teaching.is_some()).unwrap(),
            false,
        ),
        (
            3,
            all.iter()
                .find(|s| s.apache_static_teaching.is_some())
                .unwrap(),
            true,
        ),
        (
            4,
            all.iter().find(|s| s.client_teaching.is_some()).unwrap(),
            true,
        ),
        (
            5,
            all.iter()
                .find(|s| s.debian_packages_teaching.is_some())
                .unwrap(),
            true,
        ),
        (
            6,
            all.iter().find(|s| s.dhcpv4_teaching.is_some()).unwrap(),
            true,
        ),
        (
            7,
            all.iter().find(|s| s.dns_teaching.is_some()).unwrap(),
            false,
        ),
        (
            8,
            all.iter()
                .find(|s| s.ipv4_routing_teaching.is_some())
                .unwrap(),
            false,
        ),
        (
            9,
            all.iter().find(|s| s.ipv6_teaching.is_some()).unwrap(),
            false,
        ),
        (
            10,
            all.iter()
                .find(|s| s.legacy_access_teaching.is_some())
                .unwrap(),
            true,
        ),
        (
            11,
            all.iter()
                .find(|s| s.legacy_logs_teaching.is_some())
                .unwrap(),
            true,
        ),
        (
            12,
            all.iter().find(|s| s.mac_teaching.is_some()).unwrap(),
            false,
        ),
        (
            13,
            all.iter().find(|s| s.nat_teaching.is_some()).unwrap(),
            true,
        ),
        (
            14,
            all.iter().find(|s| s.ospf_teaching.is_some()).unwrap(),
            true,
        ),
        (
            15,
            all.iter().find(|s| s.process_teaching.is_some()).unwrap(),
            false,
        ),
        (
            16,
            all.iter()
                .find(|s| s.rpm_packages_teaching.is_some())
                .unwrap(),
            true,
        ),
        (
            17,
            all.iter().find(|s| s.teaching.is_some()).unwrap(),
            false,
        ),
        (
            18,
            all.iter().find(|s| s.seed_dns_teaching.is_some()).unwrap(),
            true,
        ),
        (
            19,
            all.iter()
                .find(|s| s.seed_mount_teaching.is_some())
                .unwrap(),
            true,
        ),
        (
            20,
            all.iter().find(|s| s.service_teaching.is_some()).unwrap(),
            false,
        ),
        (
            21,
            all.iter()
                .find(|s| s.spanning_tree_teaching.is_some())
                .unwrap(),
            true,
        ),
        (
            22,
            all.iter().find(|s| s.storage_teaching.is_some()).unwrap(),
            false,
        ),
        (
            23,
            all.iter().find(|s| s.switching_teaching.is_some()).unwrap(),
            false,
        ),
        (
            24,
            all.iter().find(|s| s.terminal_teaching.is_some()).unwrap(),
            false,
        ),
        (
            25,
            all.iter()
                .find(|s| s.unit_manager_teaching.is_some())
                .unwrap(),
            true,
        ),
        (26, &all[0], false),
    ];

    assert_eq!(sites.len(), 27);
    for (site, s, result) in sites {
        for new in [false, true] {
            for existing in [false, true] {
                let mut base = Attempt::new(s, 11);
                workspace::prepare(&mut base);
                if existing {
                    invoke_legacy(site, &mut base, s, new).unwrap();
                }
                let mut good = base.clone();
                invoke_legacy(site, &mut good, s, new).unwrap();
                assert!(good.workspace.as_ref().unwrap().valid());
                for (d, l) in [(true, false), (false, true), (true, true)] {
                    let mut mixed = base.clone();
                    let w = mixed.workspace.as_mut().unwrap();
                    if d {
                        w.etherchannel_draft = Some(draft.clone());
                    }
                    if l {
                        w.etherchannel_learning = Some(learning.clone());
                    }
                    assert!(w.valid());
                    let before = serde_json::to_value(&mixed).unwrap();
                    let outcome = invoke_legacy(site, &mut mixed, s, new);
                    if result {
                        assert!(outcome.is_err());
                    }
                    exact(&mixed, &before);
                    assert!(!progress::current_solved(&mixed, s));
                }
                if site != 26 {
                    let mut own = Attempt::new(&source, 1);
                    let before = serde_json::to_value(&own).unwrap();
                    let _ = invoke_legacy(site, &mut own, s, new);
                    exact(&own, &before);
                }
            }
        }
    }
}

fn old_prepared(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 9);
    workspace::prepare(&mut a);
    let names = [
        "accounts_teaching",
        "acl_teaching",
        "address_teaching",
        "apache_static_teaching",
        "client_teaching",
        "debian_packages_teaching",
        "dhcpv4_teaching",
        "dns_teaching",
        "ipv4_routing_teaching",
        "ipv6_teaching",
        "legacy_access_teaching",
        "legacy_logs_teaching",
        "mac_teaching",
        "nat_teaching",
        "ospf_teaching",
        "process_teaching",
        "rpm_packages_teaching",
        "teaching",
        "seed_dns_teaching",
        "seed_mount_teaching",
        "service_teaching",
        "spanning_tree_teaching",
        "storage_teaching",
        "switching_teaching",
        "terminal_teaching",
        "unit_manager_teaching",
    ];
    if matches!(a.state, State::Cron { .. }) {
        // New scheduling workspaces require their real authored Goal context;
        // retain all four in the positive/foreign-negative matrix.
        dlms_learning_lab::cron_workspace::budget::prepare(&mut a, s).unwrap();
    }
    if let Some(lesson) = &s.file_backup_teaching {
        // Preserve this real new-family positive in the cross-family matrix.
        dlms_learning_lab::file_backup_teaching::prepare(&mut a, lesson, true).unwrap();
    }
    let raw = serde_json::to_value(s).unwrap();
    for (site, key) in names.iter().enumerate() {
        if raw.get(*key).is_some() {
            invoke_legacy(site, &mut a, s, true).unwrap();
        }
    }
    if let Some(l) = &s.discovery_teaching {
        let State::Discovery { simulation } = &a.state else {
            panic!()
        };
        let w = a.workspace.as_mut().unwrap();
        w.discovery_draft = Some(dlms_learning_lab::discovery_workspace::Draft::new(
            simulation,
        ));
        w.discovery_learning = Some(dlms_learning_lab::discovery_teaching::Learning::new(l));
    }
    if matches!(a.state, State::UnitManager { .. }) {
        a.workspace.as_mut().unwrap().unit_manager_terminal = Some(Default::default());
    }
    if matches!(a.state, State::DebianPackages { .. }) {
        a.workspace.as_mut().unwrap().debian_packages_terminal = Some(Default::default());
    }
    assert!(a.workspace.as_ref().unwrap().valid());
    a
}
fn nfs_source() -> pack::Scenario {
    let raw: Value = serde_json::from_str(include_str!("fixtures/nfs-workbench-v1.json")).unwrap();
    let c = &raw["cases"][0];
    let solution = c["reference"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            format!(
                "{}{}",
                dlms_learning_lab::nfs_workspace::PREFIX,
                serde_json::to_string(a).unwrap()
            )
        })
        .collect::<Vec<_>>();
    let v = json!({"schema_version":1,"scenarios":[{"id":"nfs-interop-closure","version":1,"track":"LPIC-2","topic":"NFS technical fixture","unit":"","title":"Unregistered NFS boundary fixture","summary":"Existing independent technical source","objective":"Check isolation from LACP work","retry_guidance":"Keep kinds separate","difficulty":"Intermediate","minutes":15,"steps":["Inspect","Edit","Check"],"guide":[],"explanation":"Virtual supplied file model","hints":["Save differs from Apply"],"sources":["https://www.rfc-editor.org/rfc/rfc1813.html"],"simplifications":"Existing supplied NFS ACCESS-style fixture","initial":{"kind":"Nfs","simulation":c["definition"]["initial"]},"goal":{"kind":"Nfs","definition":c["definition"]},"solution":solution,"prerequisites":[],"nfs_teaching":c["lesson"]}]});
    pack::load(&v.to_string()).unwrap().remove(0)
}
#[test]
fn valid_old_family_metadata_is_rejected_on_lacp_for_current_prepare_helpers_decode_and_save() {
    let source = scenario(0);
    let mut good = prepared(&source);
    for action in actions(0, "reference") {
        apply(&mut good, &source, &action);
    }
    let mut all = pack::bundled().unwrap();
    all.push(source.clone());
    let nfs = nfs_source();
    all.push(nfs.clone());
    original_catalog_prefix(&all);
    let mut contexts = all[..147].iter().map(old_prepared).collect::<Vec<_>>();
    contexts.extend(
        all[147..]
            .iter()
            .filter(|s| !matches!(s.initial, State::EtherChannel { .. } | State::Nfs { .. }))
            .map(old_prepared),
    );
    let mut n = Attempt::new(&nfs, 9);
    dlms_learning_lab::nfs_teaching::prepare(&mut n, &nfs, true).unwrap();
    contexts.push(n);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.json");
    progress::save(&path, &profile(&source, good.clone())).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mut fields = std::collections::BTreeSet::new();
    for old in contexts {
        let raw = serde_json::to_value(old.workspace.as_ref().unwrap()).unwrap();
        for (name, value) in raw.as_object().unwrap() {
            if value.is_null()
                || !(name.ends_with("_draft")
                    || name.ends_with("_learning")
                    || name.ends_with("_session")
                    || name.ends_with("_terminal"))
            {
                continue;
            }
            if !fields.insert(name.clone()) {
                continue;
            }
            let mut mixed = serde_json::to_value(&good).unwrap();
            mixed["workspace"][name] = value.clone();
            let mut mixed: Attempt = serde_json::from_value(mixed).unwrap();
            assert!(mixed.workspace.as_ref().unwrap().valid());
            let before = serde_json::to_value(&mixed).unwrap();
            assert!(!progress::current_solved(&mixed, &source), "{name}");
            assert!(teach::prepare(&mut mixed, &source, false).is_err());
            assert!(work::fill_practice(&mut mixed, &source).is_err());
            assert!(work::reset_practice(&mut mixed, &source).is_err());
            assert!(
                work::set_assignment_draft(
                    &mut mixed,
                    &source,
                    good.workspace
                        .as_ref()
                        .unwrap()
                        .etherchannel_draft
                        .as_ref()
                        .unwrap()
                )
                .is_err()
            );
            assert!(progress::save(&path, &profile(&source, mixed.clone())).is_err());
            assert_eq!(std::fs::read(&path).unwrap(), original);
            assert!(
                progress::decode(
                    &serde_json::to_vec(&profile(&source, mixed.clone())).unwrap(),
                    &all
                )
                .is_err()
            );
            exact(&mixed, &before);
        }
    }
    for key in [
        "nfs_session",
        "nfs_learning",
        "rpm_session",
        "rpm_learning",
        "nat_draft",
        "nat_learning",
        "discovery_draft",
        "discovery_learning",
        "script_learning",
        "unit_manager_terminal",
        "debian_packages_terminal",
    ] {
        assert!(
            fields.contains(key),
            "missing actual foreign control: {key}"
        );
    }
}
#[test]
fn new_draft_and_learning_are_refused_on_every_registered_kind_and_valid_nfs() {
    let source = scenario(0);
    let draft = a_draft(&source);
    let learning =
        teach::Learning::new(source.etherchannel_teaching.as_ref().unwrap(), true).unwrap();
    let mut all = pack::bundled().unwrap();
    all.push(source);
    all.push(nfs_source());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.json");
    for s in all
        .iter()
        .filter(|s| !matches!(s.initial, State::EtherChannel { .. }))
    {
        let mut good = if matches!(s.initial, State::Nfs { .. }) {
            let mut a = Attempt::new(s, 9);
            dlms_learning_lab::nfs_teaching::prepare(&mut a, s, true).unwrap();
            a
        } else {
            old_prepared(s)
        };
        // All admitted old kinds are controls, including incomplete work; state grading is never manufactured.
        progress::save(&path, &profile(s, good.clone())).unwrap();
        let original = std::fs::read(&path).unwrap();
        assert!(progress::decode(&original, &all).is_ok());
        for (d, l) in [(true, false), (false, true), (true, true)] {
            let mut bad = good.clone();
            let w = bad.workspace.as_mut().unwrap();
            if d {
                w.etherchannel_draft = Some(draft.clone());
            }
            if l {
                w.etherchannel_learning = Some(learning.clone());
            }
            assert!(w.valid());
            let before = serde_json::to_value(&bad).unwrap();
            assert!(!progress::current_solved(&bad, s));
            assert!(progress::save(&path, &profile(s, bad.clone())).is_err());
            assert_eq!(std::fs::read(&path).unwrap(), original);
            assert!(
                progress::decode(&serde_json::to_vec(&profile(s, bad.clone())).unwrap(), &all)
                    .is_err()
            );
            workspace::prepare(&mut bad);
            exact(&bad, &before);
        }
        // A valid source-only foreign Lesson also rejects current truth, independently of Attempt fields.
        let mut bad_source = s.clone();
        bad_source.etherchannel_teaching = Some(learning_source());
        assert!(!progress::current_solved(&good, &bad_source));
        let before = serde_json::to_value(&good).unwrap();
        assert!(teach::prepare(&mut good, &bad_source, true).is_err());
        exact(&good, &before);
    }
}
fn learning_source() -> teach::Lesson {
    scenario(0).etherchannel_teaching.unwrap()
}
#[test]
fn backup_restore_selected_reset_and_legacy_first_open_preserve_full_work() {
    use dlms_learning_lab::backup;
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    let mut all = pack::bundled().unwrap();
    let source = scenario(0);
    all.push(source.clone());
    let mut a = prepared(&source);
    for action in actions(0, "reference") {
        apply(&mut a, &source, &action);
    }
    work::fill_practice(&mut a, &source).unwrap();
    let selected = serde_json::to_value(&a).unwrap();
    let mut p = profile(&source, a);
    p.favorites.insert(source.id.clone());
    let other = &all[0];
    p.attempts.insert(other.id.clone(), Attempt::new(other, 1));
    let unrelated = serde_json::to_value(&p.attempts[&other.id]).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let export = dir.path().join("export.json");
    progress::save(&path, &p).unwrap();
    backup::export(&export, &p, &all).unwrap();
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    exact(&candidate.progress.attempts[&source.id], &selected);
    backup::restore(&path, &candidate, &all).unwrap();
    let mut restored = progress::load(&path, &all).unwrap();
    exact(&restored.attempts[&source.id], &selected);
    exact(&restored.attempts[&other.id], &unrelated);
    restored
        .attempts
        .insert(source.id.clone(), Attempt::new(&source, 1));
    teach::prepare(
        restored.attempts.get_mut(&source.id).unwrap(),
        &source,
        true,
    )
    .unwrap();
    exact(&restored.attempts[&other.id], &unrelated);
    assert!(restored.favorites.contains(&source.id));
    assert!(!progress::current_solved(
        &restored.attempts[&source.id],
        &source
    ));
    let mut old = Attempt::new(&source, 1);
    teach::prepare(&mut old, &source, false).unwrap();
    assert_eq!(
        old.workspace
            .as_ref()
            .unwrap()
            .etherchannel_learning
            .as_ref()
            .unwrap()
            .view,
        View::Assignment
    );
    let before = serde_json::to_value(&old).unwrap();
    teach::prepare(&mut old, &source, true).unwrap();
    exact(&old, &before);
}

#[test]
fn maximum_supplied_topology_draft_fits_while_escape_heavy_aggregate_refuses() {
    use std::collections::{BTreeMap, BTreeSet};
    let allowed = (1..=64).collect::<BTreeSet<u16>>();
    let profile = core::Profile::Trunk {
        native: 1,
        allowed: allowed.clone(),
    };
    let mut devices = BTreeMap::new();
    let mut configured = BTreeMap::new();
    let mut cables = BTreeMap::new();
    for d in 0..8 {
        let name = format!("d{d}");
        devices.insert(
            name.clone(),
            core::Device {
                mac: core::Mac::parse(&format!("02:00:00:00:01:{:02x}", d + 1)).unwrap(),
                ports: (0..8)
                    .map(|p| {
                        (
                            format!("p{p}"),
                            core::PortSpec {
                                rates: BTreeSet::from([core::Rate::R1000]),
                            },
                        )
                    })
                    .collect(),
                groups: (0..2)
                    .map(|g| (format!("g{g}"), core::GroupSlot {}))
                    .collect(),
            },
        );
        configured.insert(
            name,
            core::DeviceConfig {
                vlans: allowed.clone(),
                groups: (0..2)
                    .map(|g| {
                        (
                            format!("g{g}"),
                            core::GroupConfig {
                                up: true,
                                template: core::Template {
                                    rate: core::Rate::R1000,
                                    profile: profile.clone(),
                                },
                            },
                        )
                    })
                    .collect(),
                ports: (0..8)
                    .map(|p| {
                        (
                            format!("p{p}"),
                            core::PortConfig {
                                up: true,
                                group: Some(format!("g{}", p / 4)),
                                mode: core::LacpMode::Active,
                                rate: core::Rate::R1000,
                                duplex: core::Duplex::Full,
                                profile: profile.clone(),
                            },
                        )
                    })
                    .collect(),
            },
        );
    }
    for d in (0..8).step_by(2) {
        for p in 0..8 {
            cables.insert(
                format!("c{d}-{p}"),
                core::Cable {
                    a: core::Endpoint {
                        device: format!("d{d}"),
                        port: format!("p{p}"),
                    },
                    b: core::Endpoint {
                        device: format!("d{}", d + 1),
                        port: format!("p{p}"),
                    },
                },
            );
        }
    }
    let environment = core::Environment {
        cables: cables.keys().map(|n| (n.clone(), true)).collect(),
    };
    let state = core::State::new(
        core::Topology {
            fixture: "max-draft".into(),
            devices,
            cables,
        },
        core::Configuration {
            devices: configured,
        },
        environment,
    )
    .unwrap();
    let draft = work::Draft::from_state(&state).unwrap();
    assert!(draft.valid_for(&state));
    assert_eq!(
        draft.parsed(state.topology()).unwrap(),
        *state.configuration()
    );
    assert_eq!(draft.devices.len(), 8);
    assert_eq!(
        draft.devices.values().map(|d| d.ports.len()).sum::<usize>(),
        64
    );
    let mut escaped = draft.clone();
    for d in escaped.devices.values_mut() {
        for p in d.ports.values_mut() {
            p.profile.allowed = "\u{1}".repeat(384);
        }
        for g in d.groups.values_mut() {
            g.profile.allowed = "\u{1}".repeat(384);
        }
    }
    assert!(!escaped.valid());
    assert!(work::Draft::from_json(&serde_json::to_vec(&escaped).unwrap()).is_err());
    assert!(draft.valid());
}
#[test]
fn first_fixture_configuration_and_example_values_do_not_solve_another_context() {
    let first = actions(0, "reference").remove(0);
    for n in [1, 2] {
        let s = scenario(n);
        let mut a = prepared(&s);
        let before = serde_json::to_value(&a).unwrap();
        let core::Action::Configure { configuration } = &first;
        assert!(work::Draft::from_configuration(configuration, sim(&a).topology()).is_err());
        assert!(
            engine::execute_for_goal(
                &mut a.state,
                &s.goal,
                &format!("{}{}", work::PREFIX, serde_json::to_string(&first).unwrap())
            )
            .is_err()
        );
        exact(&a, &before);
        let example =
            teach::Learning::new(s.etherchannel_teaching.as_ref().unwrap(), true).unwrap();
        let mut source = serde_json::to_value(&a).unwrap();
        source["state"]["simulation"] = serde_json::to_value(example.simulation).unwrap();
        let mut forged: Attempt = serde_json::from_value(source).unwrap();
        assert!(!progress::current_solved(&forged, &s));
        assert!(teach::prepare(&mut forged, &s, false).is_err());
    }
}

#[test]
fn presentation_bits_and_hint_digit_growth_use_whole_attempt_budget_without_credit() {
    let mut s = scenario(0);
    s.hints = (0..11)
        .map(|n| format!("Independent bounded hint {n}"))
        .collect();
    let mut a = prepared(&s);
    work::record_mode(&mut a, &s, true).unwrap();
    work::record_hint(&mut a, &s, 9).unwrap();
    work::set_saved_check(&mut a, &s, true).unwrap();
    assert!(!progress::current_solved(&a, &s));
    fill_attempt(&mut a);
    let before = serde_json::to_value(&a).unwrap();
    assert!(work::record_mode(&mut a, &s, false).is_err());
    exact(&a, &before);
    assert!(work::record_hint(&mut a, &s, 10).is_err());
    exact(&a, &before);
    assert!(work::set_saved_check(&mut a, &s, false).is_err());
    exact(&a, &before);
    work::record_hint(&mut a, &s, 8).unwrap();
    let mut expected = before;
    expected["hints"] = json!(8);
    exact(&a, &expected);
}
