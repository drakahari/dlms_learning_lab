//! Actual registered cache lessons and exact430-origin/progress preservation.
use dlms_learning_lab::{
    course,
    engine::{self, State},
    http_cache_workspace as w, pack,
    progress::{self, Attempt},
    workspace,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = [
    "lpic2-cache-shared-privacy",
    "lpic2-cache-freshness-handover",
];
fn catalog() -> &'static Vec<pack::Scenario> {
    static DATA: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    DATA.get_or_init(|| pack::current().unwrap())
}
fn source(n: usize) -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == IDS[n]).unwrap()
}
fn simulation(a: &mut Attempt) -> &mut w::Session {
    let State::HttpCache { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn retained432_preserves_all430_scenarios_166_unit_values_and_real_prerequisite_order() {
    let frozen = pack::accepted432().unwrap();
    let before = pack::accepted430().unwrap();
    assert_eq!((before.len(), frozen.len()), (430, 432));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&frozen[..430]).unwrap()
    );
    let old = course::accepted430(&before).unwrap();
    let now = course::accepted432(&frozen).unwrap();
    assert_eq!((old.len(), now.len()), (166, 167));
    let at = now
        .iter()
        .position(|u| u.id == "linux-shared-http-cache")
        .unwrap();
    assert_eq!(now[at].labs, IDS);
    assert!(
        now.iter()
            .position(|u| u.id == "linux-proxy-access-policy")
            .unwrap()
            < at
    );
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(
            now.iter()
                .filter(|u| u.id != "linux-shared-http-cache")
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let fixture: pack::Pack =
        serde_json::from_str(include_str!("fixtures/http-cache-preview-v1.json")).unwrap();
    for s in fixture.scenarios {
        assert_eq!(
            serde_json::to_value(frozen.iter().find(|v| v.id == s.id).unwrap()).unwrap(),
            serde_json::to_value(s).unwrap()
        );
    }
}
#[test]
fn taught_references_and_alternate_supported_policies_pass_without_example_credit() {
    for n in 0..2 {
        let s = source(n);
        let d = w::source(s).unwrap();
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        {
            let m = simulation(&mut a);
            m.practice
                .edit(&d.example, d.example.reference_headers())
                .unwrap();
            assert!(m.practice.check(&d.example).unwrap().passed());
        }
        assert!(!progress::current_solved(&a, s));
        for action in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, action).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        let m = simulation(&mut a);
        let mut headers = d.initial.reference_headers();
        headers[0].value = if n == 0 {
            "max-age=30"
        } else {
            "s-maxage=30, max-age=600"
        }
        .into();
        if n == 0 {
            headers[1].value = "no-store".into();
        }
        m.assignment.edit(&d.initial, headers).unwrap();
        assert!(m.assignment.check(&d.initial).unwrap().passed());
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn wrong_storage_ignored_age_overlong_lifetimes_and_partial_repairs_fail_usefully() {
    let s = source(0);
    let d = w::source(s).unwrap();
    let mut a = Attempt::new(s, 1);
    let mut h = d.initial.reference_headers();
    h[1].value = "no-cache, max-age=60".into();
    let m = simulation(&mut a);
    m.assignment.edit(&d.initial, h).unwrap();
    let r = m.assignment.check(&d.initial).unwrap();
    assert!(!r.passed());
    assert!(r.traces[0].steps.iter().all(|s| s.passed));
    assert!(r.traces[1].steps.iter().any(|s| !s.passed));
    assert!(!progress::current_solved(&a, s));
    let s = source(1);
    let d = w::source(s).unwrap();
    let mut a = Attempt::new(s, 1);
    for wrong in [
        "max-age=600",
        "max-age=600, s-maxage=20",
        "max-age=600, s-maxage=60",
        "max-age=600, no-store",
        "max-age=600, no-cache",
        "max-age=wrong",
        "immutable",
    ] {
        let mut h = d.initial.reference_headers();
        h[0].value = wrong.into();
        let m = simulation(&mut a);
        m.assignment.edit(&d.initial, h).unwrap();
        assert!(!m.assignment.check(&d.initial).unwrap().passed(), "{wrong}");
        assert!(!progress::current_solved(&a, s));
    }
    let m = simulation(&mut a);
    m.assignment
        .edit(&d.initial, d.initial.reference_headers())
        .unwrap();
    assert!(m.assignment.check(&d.initial).unwrap().passed());
    assert!(progress::current_solved(&a, s));
    let m = simulation(&mut a);
    m.assignment.headers[0].value = "no-store".into();
    m.assignment.checked_stamp = Some(w::Stamp {
        headers: m.assignment.headers.clone(),
    });
    assert!(
        !progress::current_solved(&a, s),
        "A forged checked stamp cannot turn broken behavior into success."
    );
}
#[test]
fn all430_prior_attempts_favorites_and_two_isolated_drafts_survive_disk_resume_and_selected_reset()
{
    let before = pack::accepted430().unwrap();
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &before {
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        if dlms_learning_lab::command_location_workspace::has_source(s) {
            dlms_learning_lab::command_location_workspace::prepare(&mut a, s, true).unwrap();
        }
        if dlms_learning_lab::accounts_template_workspace::has_source(s) {
            dlms_learning_lab::accounts_template_workspace::prepare(&mut a, s, true).unwrap();
        }
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let retained = serde_json::to_value(&p).unwrap();
    for n in 0..2 {
        let s = source(n);
        let d = w::source(s).unwrap();
        let mut a = Attempt::new(s, 2);
        workspace::prepare(&mut a);
        let m = simulation(&mut a);
        m.practice
            .edit(&d.example, d.example.reference_headers())
            .unwrap();
        m.practice.check(&d.example).unwrap();
        let mut h = m.assignment.headers.clone();
        h[0].value = "max-age=unfinished".into();
        m.assignment.edit(&d.initial, h).unwrap();
        m.view = dlms_learning_lab::script_teaching::View::Assignment;
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let mut old: progress::Progress =
        serde_json::from_value(serde_json::to_value(&loaded).unwrap()).unwrap();
    for id in IDS {
        old.attempts.remove(id);
        old.favorites.remove(id);
    }
    assert_eq!(serde_json::to_value(old).unwrap(), retained);
    for n in 0..2 {
        let s = source(n);
        assert!(!progress::current_solved(&loaded.attempts[&s.id], s));
    }
    let mut bad: progress::Progress =
        serde_json::from_value(serde_json::to_value(&loaded).unwrap()).unwrap();
    bad.attempts.get_mut(IDS[0]).unwrap().scenario_version += 1;
    assert!(progress::save_with_catalog(&path, &bad, catalog()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let mut reset: progress::Progress =
        serde_json::from_value(serde_json::to_value(&loaded).unwrap()).unwrap();
    reset
        .attempts
        .insert(IDS[0].into(), Attempt::new(source(0), 3));
    assert!(reset.favorites.contains(IDS[0]));
    assert_eq!(
        serde_json::to_value(&reset.attempts[IDS[1]]).unwrap(),
        serde_json::to_value(&loaded.attempts[IDS[1]]).unwrap()
    );
    for s in &before {
        assert_eq!(
            serde_json::to_value(&reset.attempts[&s.id]).unwrap(),
            serde_json::to_value(&loaded.attempts[&s.id]).unwrap()
        );
    }
}
#[test]
fn malformed_unreachable_or_prechecked_authored_packs_do_not_get_admitted() {
    let good: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/http-cache-preview-v1.json")).unwrap();
    pack::load(&serde_json::to_string(&good).unwrap()).unwrap();
    let last = good["scenarios"].as_array().unwrap().len() - 1;
    for target in ["goal", "initial"] {
        let mut bad = good.clone();
        if target == "goal" {
            bad["scenarios"][last]["goal"]["definition"]["initial"]["traces"][0]["expected"] =
                serde_json::json!([]);
        } else {
            bad["scenarios"][last]["initial"]["simulation"]["assignment"]["headers"][0]["path"] =
                serde_json::json!("/not-supplied");
        }
        assert!(pack::load(&serde_json::to_string(&bad).unwrap()).is_err());
    }
    let mut bad = good.clone();
    let headers = bad["scenarios"][last]["initial"]["simulation"]["assignment"]["headers"].clone();
    bad["scenarios"][last]["initial"]["simulation"]["assignment"]["checked_stamp"] =
        serde_json::json!({"headers":headers});
    assert!(pack::load(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut bad = good;
    bad["scenarios"][last]["initial"]["simulation"]["host_file"] = serde_json::json!("/etc/passwd");
    assert!(pack::load(&serde_json::to_string(&bad).unwrap()).is_err());
}
