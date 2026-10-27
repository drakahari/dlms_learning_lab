use dlms_learning_lab::{
    engine, nfs_teaching as teach, nfs_workspace as work, pack,
    progress::{self, Attempt, Progress},
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
fn solved(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 7);
    for c in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    a
}
fn profile(s: &pack::Scenario, a: Attempt) -> Progress {
    let mut p = Progress {
        schema_version: 1,
        clock: 7,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    p
}
fn refusal(
    a: &Attempt,
    s: &pack::Scenario,
    all: &[pack::Scenario],
    path: &std::path::Path,
    prior: &[u8],
) {
    let before = serde_json::to_value(a).unwrap();
    let rows = match s.goal {
        engine::Goal::Nfs { .. } => progress::current_nfs_checks(a, s),
        engine::Goal::RpmPackages { .. } => progress::current_rpm_checks(a, s),
        engine::Goal::BasicNat { .. } => progress::current_nat_checks(a, s),
        engine::Goal::DhcpV4 { .. } => progress::current_dhcpv4_checks(a, s),
        engine::Goal::SpanningTree { .. } => progress::current_spanning_tree_checks(a, s),
        engine::Goal::ApacheStatic { .. } => progress::current_apache_static_checks(a, s),
        engine::Goal::DebianPackages { .. } => progress::current_debian_packages_checks(a, s),
        _ => panic!(),
    };
    assert!(!rows.is_empty() && rows.iter().any(|r| !r.0));
    assert!(rows.iter().all(|r| r.1.len() <= 512));
    assert!(!progress::current_solved(a, s));
    let p = profile(s, a.clone());
    assert!(progress::save(path, &p).is_err());
    assert_eq!(std::fs::read(path).unwrap(), prior);
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), all).is_err());
    exact(a, &before);
}
#[test]
fn nfs_fields_cannot_reuse_other_families_applied_success_or_save_admission() {
    let mut all = pack::bundled().unwrap();
    let nfs = scenario(0);
    let learning = teach::Learning::new(nfs.nfs_teaching.as_ref().unwrap()).unwrap();
    all.push(nfs);
    let mut count = 0;
    for s in &all {
        if !matches!(
            s.goal,
            engine::Goal::RpmPackages { .. }
                | engine::Goal::BasicNat { .. }
                | engine::Goal::DhcpV4 { .. }
                | engine::Goal::SpanningTree { .. }
                | engine::Goal::ApacheStatic { .. }
                | engine::Goal::DebianPackages { .. }
        ) {
            continue;
        }
        let mut good = solved(s);
        good.workspace = Some(workspace::WorkspaceData::new());
        assert!(progress::current_solved(&good, s));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("synthetic.json");
        progress::save(&path, &profile(s, good.clone())).unwrap();
        let prior = std::fs::read(&path).unwrap();
        assert!(progress::decode(&prior, &all).is_ok());
        for (session, practice) in [(true, false), (false, true), (true, true)] {
            let mut a = good.clone();
            let w = a.workspace.as_mut().unwrap();
            if session {
                w.nfs_session = Some(work::Session::default());
            }
            if practice {
                w.nfs_learning = Some(learning.clone());
            }
            assert!(w.valid());
            refusal(&a, s, &all, &path, &prior);
        }
        count += 1;
    }
    assert_eq!(count, 22); //4RPM+3NAT+3DHCP+4STP+4Apache+4Debian.
}
#[test]
fn genuine_rpm_nat_sessions_and_learning_are_rejected_on_nfs_without_mutation() {
    let nfs = scenario(0);
    let mut all = pack::bundled().unwrap();
    all.push(nfs.clone());
    let mut good = solved(&nfs);
    teach::prepare(&mut good, &nfs, true).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.json");
    progress::save(&path, &profile(&nfs, good.clone())).unwrap();
    let prior = std::fs::read(&path).unwrap();
    for rpm in [false, true] {
        let source = all
            .iter()
            .find(|s| {
                if rpm {
                    s.rpm_packages_teaching.is_some()
                } else {
                    s.nat_teaching.is_some()
                }
            })
            .unwrap();
        let mut normal = Attempt::new(source, 7);
        workspace::prepare(&mut normal);
        if rpm {
            let engine::Goal::RpmPackages { definition } = &source.goal else {
                panic!()
            };
            dlms_learning_lab::rpm_packages_teaching::prepare(
                &mut normal,
                definition,
                source.rpm_packages_teaching.as_ref().unwrap(),
                true,
            )
            .unwrap();
        } else {
            dlms_learning_lab::nat_teaching::prepare(
                &mut normal,
                source.nat_teaching.as_ref().unwrap(),
                true,
            )
            .unwrap();
        }
        let source_w = normal.workspace.as_ref().unwrap();
        assert!(source_w.valid());
        progress::save(
            &dir.path().join("normal.json"),
            &profile(source, normal.clone()),
        )
        .unwrap();
        for (draft, practice) in [(true, false), (false, true), (true, true)] {
            let mut a = good.clone();
            let w = a.workspace.as_mut().unwrap();
            if rpm {
                if draft {
                    w.rpm_session = source_w.rpm_session.clone();
                    assert!(w.rpm_session.is_some());
                }
                if practice {
                    w.rpm_learning = source_w.rpm_learning.clone();
                    assert!(w.rpm_learning.is_some());
                }
            } else {
                if draft {
                    w.nat_draft = source_w.nat_draft.clone();
                    assert!(w.nat_draft.is_some());
                }
                if practice {
                    w.nat_learning = source_w.nat_learning.clone();
                    assert!(w.nat_learning.is_some());
                }
            }
            assert!(w.valid());
            refusal(&a, &nfs, &all, &path, &prior);
            let before = serde_json::to_value(&a).unwrap();
            assert!(teach::prepare(&mut a, &nfs, false).is_err());
            exact(&a, &before);
            assert!(work::fill_practice(&mut a, &nfs).is_err());
            exact(&a, &before);
            assert!(work::reset_practice(&mut a, &nfs).is_err());
            exact(&a, &before);
        }
    }
}
#[test]
fn request_draft_decode_bounds_and_unknown_keys_reject_without_overwriting_target() {
    let s = scenario(0);
    let a = solved(&s);
    let mut all = pack::bundled().unwrap();
    all.push(s.clone());
    let mut p = profile(&s, a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.json");
    progress::save(&path, &p).unwrap();
    let prior = std::fs::read(&path).unwrap();
    teach::prepare(p.attempts.get_mut(&s.id).unwrap(), &s, true).unwrap();
    let valid = serde_json::to_value(&p).unwrap();
    assert!(progress::decode(&serde_json::to_vec(&valid).unwrap(), &all).is_ok());
    p.attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .nfs_session
        .as_mut()
        .unwrap()
        .write_text = "\0".into();
    let before = serde_json::to_value(&p).unwrap();
    assert!(progress::save(&path, &p).is_err());
    exact(&p, &before);
    assert_eq!(std::fs::read(&path).unwrap(), prior);
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &all).is_err());
    let mut unknown = valid;
    unknown["attempts"][&s.id]["workspace"]["nfs_session"] =
        serde_json::to_value(work::Session::default()).unwrap();
    unknown["attempts"][&s.id]["workspace"]["nfs_session"]["future_authority"] = json!(true);
    assert!(progress::decode(&serde_json::to_vec(&unknown).unwrap(), &all).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), prior);
}
