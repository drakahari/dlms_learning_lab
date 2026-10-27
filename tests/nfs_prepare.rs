use dlms_learning_lab::{
    nfs_teaching as teach, nfs_workspace as work, pack,
    progress::{self, Attempt},
    workspace,
};
use serde_json::json;
fn scenario(index: usize) -> pack::Scenario {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/nfs-workbench-v1.json")).unwrap();
    let c = &raw["cases"][index];
    let solution = c["reference"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| format!("{}{}", work::PREFIX, serde_json::to_string(a).unwrap()))
        .collect::<Vec<_>>();
    let v = json!({"schema_version":1,"scenarios":[{"id":format!("nfs-technical-{index}"),"version":1,"track":"LPIC-2","topic":"Virtual NFS","unit":"","title":"Technical NFS adapter","summary":"Inspect supplied NFS layers","objective":"Complete actual current and new-mount outcomes","retry_guidance":"Inspect saved, applied and live values","difficulty":"Intermediate","minutes":15,"steps":["Inspect","Edit","Check"],"guide":[],"explanation":"Virtual existing files only","hints":["Save differs from Apply"],"sources":["https://www.rfc-editor.org/rfc/rfc1813.html"],"simplifications":"Supplied virtual NFSv3 ACCESS-style model; no host mount or RPC","initial":{"kind":"Nfs","simulation":c["definition"]["initial"]},"goal":{"kind":"Nfs","definition":c["definition"]},"solution":solution,"prerequisites":[],"nfs_teaching":c["lesson"]}]});
    pack::load(&v.to_string()).unwrap().remove(0)
}
fn exact<T: serde::Serialize>(a: &T, b: &serde_json::Value) {
    assert!(
        serde_json::to_value(a).unwrap() == *b,
        "complete caller differs"
    );
}
fn invoke(site: usize, a: &mut Attempt, s: &pack::Scenario, new: bool) -> Result<(), String> {
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
fn every_legacy_prepare_rejects_new_fields_before_any_existing_or_new_mutation() {
    let all = pack::bundled().unwrap();
    let nfs = scenario(0);
    let learning = teach::Learning::new(nfs.nfs_teaching.as_ref().unwrap()).unwrap();
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
                    invoke(site, &mut base, s, new).unwrap();
                }
                let mut normal = base.clone();
                invoke(site, &mut normal, s, new).unwrap();
                assert!(normal.workspace.as_ref().unwrap().valid());
                assert!(!workspace::has_nfs_metadata(&normal));
                for (session, practice) in [(true, false), (false, true), (true, true)] {
                    let mut mixed = base.clone();
                    let w = mixed.workspace.as_mut().unwrap();
                    if session {
                        w.nfs_session = Some(work::Session::default());
                    }
                    if practice {
                        w.nfs_learning = Some(learning.clone());
                    }
                    assert!(w.valid());
                    let before = serde_json::to_value(&mixed).unwrap();
                    let outcome = invoke(site, &mut mixed, s, new);
                    if result {
                        assert!(outcome.is_err());
                    }
                    exact(&mixed, &before);
                }
            }
        }
    }
}
#[test]
fn nfs_new_intro_old_assignment_and_existing_two_drafts_are_separate_and_atomic() {
    let s = scenario(0);
    let mut a = Attempt::new(&s, 1);
    let original = serde_json::to_value(&a.state).unwrap();
    teach::prepare(&mut a, &s, true).unwrap();
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .nfs_learning
            .as_ref()
            .unwrap()
            .view,
        dlms_learning_lab::script_teaching::View::Introduction
    );
    let l = s.nfs_teaching.as_ref().unwrap();
    for step in &l.example_actions {
        work::perform_practice(&mut a, &s, &step.action).unwrap();
    }
    let w = a.workspace.as_mut().unwrap();
    w.nfs_session.as_mut().unwrap().relative = "unrun assigned query".into();
    w.nfs_learning.as_mut().unwrap().session.relative = "unrun example query".into();
    let before = serde_json::to_value(&a).unwrap();
    teach::prepare(&mut a, &s, false).unwrap();
    workspace::prepare(&mut a);
    exact(&a, &before);
    exact(&a.state, &original);
    let mut old = Attempt::new(&s, 2);
    teach::prepare(&mut old, &s, false).unwrap();
    assert_eq!(
        old.workspace
            .as_ref()
            .unwrap()
            .nfs_learning
            .as_ref()
            .unwrap()
            .view,
        dlms_learning_lab::script_teaching::View::Assignment
    );
    let mut wrong = old.clone();
    wrong.scenario_version += 1;
    let before = serde_json::to_value(&wrong).unwrap();
    assert!(teach::prepare(&mut wrong, &s, true).is_err());
    exact(&wrong, &before);
    let unix = pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| matches!(s.initial, dlms_learning_lab::engine::State::Unix { .. }))
        .unwrap();
    let mut old = Attempt::new(&unix, 3);
    let before = serde_json::to_value(&old).unwrap();
    assert!(teach::prepare(&mut old, &s, true).is_err());
    exact(&old, &before);
}
#[test]
fn new_practice_whole_attempt_admission_precedes_fixture_copy() {
    let s = scenario(0);
    let mut small = Attempt::new(&s, 1);
    teach::prepare(&mut small, &s, true).unwrap();
    let mut large = Attempt::new(&s, 1);
    let mut lo = 0usize;
    let mut hi = work::MAX_ATTEMPT;
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        large.transcript = vec!["x".repeat(mid)];
        if serde_json::to_vec(&large).unwrap().len() <= work::MAX_ATTEMPT {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    large.transcript = vec!["x".repeat(lo)];
    assert_eq!(serde_json::to_vec(&large).unwrap().len(), work::MAX_ATTEMPT);
    let before = serde_json::to_value(&large).unwrap();
    assert!(teach::prepare(&mut large, &s, true).is_err());
    exact(&large, &before);
    // Same accepted context and action with room genuinely creates separate state/drafts.
    assert!(small.workspace.as_ref().unwrap().nfs_learning.is_some());
    assert!(
        progress::current_nfs_checks(&small, &s)
            .iter()
            .any(|r| !r.0)
    );
}

#[test]
fn practice_fill_whole_attempt_frontier_rejects_before_copying_with_real_positive_control() {
    let s = scenario(0);
    let mut base = Attempt::new(&s, 1);
    teach::prepare(&mut base, &s, true).unwrap();
    let mut positive = base.clone();
    let initial = serde_json::to_value(&base).unwrap();
    let bytes = serde_json::to_vec(&positive).unwrap().len();
    work::fill_practice(&mut positive, &s).unwrap();
    assert!(serde_json::to_vec(&positive).unwrap().len() > bytes);
    assert!(
        serde_json::to_value(
            &positive
                .workspace
                .as_ref()
                .unwrap()
                .nfs_learning
                .as_ref()
                .unwrap()
                .simulation
        )
        .unwrap()
            != initial["workspace"]["nfs_learning"]["simulation"]
    );
    exact(&positive.state, &initial["state"]);
    let mut lo = 0usize;
    let mut hi = work::MAX_ATTEMPT;
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        base.transcript = vec!["x".repeat(mid)];
        if serde_json::to_vec(&base).unwrap().len() <= work::MAX_ATTEMPT {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    base.transcript = vec!["x".repeat(lo)];
    assert_eq!(serde_json::to_vec(&base).unwrap().len(), work::MAX_ATTEMPT);
    let before = serde_json::to_value(&base).unwrap();
    assert!(work::fill_practice(&mut base, &s).is_err());
    exact(&base, &before);
}
