//! Real authored policy assignments, current registration and preserved prior work.
use dlms_learning_lab::{
    course,
    engine::{self, State},
    pack,
    progress::{self, Attempt},
    reverse_proxy_workspace as w, workspace,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = ["lpic2-proxy-uri-mapping", "lpic2-proxy-private-paths"];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::current().unwrap())
}
fn source(n: usize) -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == IDS[n]).unwrap()
}
fn simulation(a: &mut Attempt) -> &mut w::Session {
    let State::ReverseProxy { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn apply(a: &mut Attempt, s: &pack::Scenario, text: &str) {
    let d = w::source(s).unwrap();
    let a = &mut simulation(a).assignment;
    a.edit(text.into()).unwrap();
    a.save(&d.initial).unwrap();
    a.apply(&d.initial).unwrap();
    a.check(&d.initial).unwrap();
}
#[test]
fn current434_retains_all432_sources_167_unit_objects_and_prerequisite_order() {
    let before = pack::accepted432().unwrap();
    let retained = pack::accepted434().unwrap();
    let snapshot = &retained;
    assert_eq!((before.len(), snapshot.len()), (432, 434));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&snapshot[..432]).unwrap()
    );
    let old = course::accepted432(&before).unwrap();
    let units = course::accepted434(snapshot).unwrap();
    assert_eq!((old.len(), units.len()), (167, 168));
    let added = units
        .iter()
        .position(|u| u.id == "linux-reverse-proxy-routing")
        .unwrap();
    assert_eq!(units[added].labs, IDS);
    for dependency in [
        "static-web-services",
        "linux-proxy-access-policy",
        "linux-shared-http-cache",
    ] {
        assert!(units.iter().position(|u| u.id == dependency).unwrap() < added);
    }
    let retained = units
        .iter()
        .filter(|u| u.id != "linux-reverse-proxy-routing")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let expected: pack::Pack =
        serde_json::from_str(include_str!("fixtures/nginx-routing-preview-v1.json")).unwrap();
    for s in expected.scenarios {
        assert_eq!(
            serde_json::to_value(snapshot.iter().find(|v| v.id == s.id).unwrap()).unwrap(),
            serde_json::to_value(&s).unwrap()
        );
    }
}
#[test]
fn taught_references_and_distinct_examples_work_without_credit_leaking() {
    for n in 0..2 {
        let s = source(n);
        let d = w::source(s).unwrap();
        assert_eq!(d.initial.cases.len(), if n == 0 { 5 } else { 6 });
        assert_ne!(d.initial.cases, d.example.cases);
        let mut a = Attempt::new(s, 1);
        let before = simulation(&mut a).assignment.clone();
        let practice = &mut simulation(&mut a).practice;
        practice.edit(d.example.required_config.clone()).unwrap();
        practice.save(&d.example).unwrap();
        practice.apply(&d.example).unwrap();
        assert!(practice.check(&d.example).unwrap().passed());
        assert_eq!(simulation(&mut a).assignment, before);
        assert!(!progress::current_solved(&a, s));
        for action in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, action).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        assert!(engine::checks(&a.state, &s.goal).iter().all(|r| r.0));
    }
}
#[test]
fn equivalent_location_partitions_and_unconstrained_error_pages_pass() {
    for n in 0..2 {
        let s = source(n);
        let d = w::source(s).unwrap();
        let text = if n == 0 {
            d.initial.required_config.replace("location /reports/","location ^~ /reports/").replace("return 404;","return 404 'Unavailable';").replace("    location = /healthz", "    location /reports/team { proxy_pass http://192.0.2.40:9000/team; }\n    location = /healthz")
        } else {
            d.initial
                .required_config
                .replace(
                    "    location = /admin {",
                    "    location /admin/reports/ { return 403; }\n    location = /admin {",
                )
                .replace("\"ready\"", "'ready'")
        };
        assert_ne!(text, d.initial.required_config);
        let mut a = Attempt::new(s, 1);
        apply(&mut a, s, &text);
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn broad_refusals_missing_application_and_hidden_path_leaks_fail_usefully() {
    for n in 0..2 {
        let s = source(n);
        let d = w::source(s).unwrap();
        for wrong in [
            "server { listen 8080; location / { return 404; } }",
            "server { listen 8080; location / { return 200 'open'; } }",
        ] {
            let mut a = Attempt::new(s, 1);
            apply(&mut a, s, wrong);
            assert!(!progress::current_solved(&a, s));
            assert!(engine::checks(&a.state, &s.goal).iter().any(|r| !r.0));
        }
        let mut a = Attempt::new(s, 1);
        let p = &mut simulation(&mut a).assignment;
        p.edit(d.initial.required_config.clone()).unwrap();
        p.save(&d.initial).unwrap();
        let r = p.check(&d.initial).unwrap();
        assert!(!r.passed());
        assert!(r.policies[1].passed);
        assert!(!r.policies[0].passed);
        let hidden=d.initial.required_config.replace("    location / {",if n==0{"    location /unlisted { return 200 'unwanted'; }\n    location / {"}else{"    location /admin/unlisted { proxy_pass http://198.51.100.20:8088; }\n    location / {"});
        apply(&mut a, s, &hidden);
        let r = d.initial.report(&simulation(&mut a).assignment).unwrap();
        assert!(r.cases.iter().all(|r| r.passed));
        assert!(r.policies.iter().all(|r| !r.passed));
        assert!(!progress::current_solved(&a, s));
        let copied = d.example.required_config.clone();
        apply(&mut a, s, &copied);
        assert!(
            !progress::current_solved(&a, s),
            "example configuration is not an assignment solution"
        );
    }
}
#[test]
fn all432_prior_attempts_favorites_and_two_new_separate_drafts_survive_safe_disk_resume() {
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &catalog()[..432] {
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        if dlms_learning_lab::command_location_workspace::has_source(s) {
            dlms_learning_lab::command_location_workspace::prepare(&mut a, s, true).unwrap();
        }
        if dlms_learning_lab::accounts_template_workspace::has_source(s) {
            dlms_learning_lab::accounts_template_workspace::prepare(&mut a, s, true).unwrap();
        }
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let original = serde_json::to_value(&p).unwrap();
    for n in 0..2 {
        let s = source(n);
        let mut a = Attempt::new(s, 2);
        workspace::prepare(&mut a);
        let d = w::source(s).unwrap();
        apply(&mut a, s, &d.initial.required_config);
        simulation(&mut a)
            .practice
            .edit("# unfinished practice\nserver {\n".into())
            .unwrap();
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let mut restored = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    for id in original["attempts"].as_object().unwrap().keys() {
        assert_eq!(
            serde_json::to_value(&restored.attempts[id]).unwrap(),
            original["attempts"][id]
        );
        assert!(restored.favorites.contains(id));
    }
    for n in 0..2 {
        let s = source(n);
        assert!(progress::current_solved(&restored.attempts[&s.id], s));
        simulation(restored.attempts.get_mut(&s.id).unwrap())
            .assignment
            .edit("server { listen 8080; location / { return 403; } }".into())
            .unwrap();
        assert!(!progress::current_solved(&restored.attempts[&s.id], s));
    }
    let untouched = serde_json::to_value(&restored.attempts[IDS[1]]).unwrap();
    restored
        .attempts
        .insert(IDS[0].into(), Attempt::new(source(0), 3));
    assert_eq!(
        serde_json::to_value(&restored.attempts[IDS[1]]).unwrap(),
        untouched
    );
    assert!(restored.favorites.contains(IDS[0]));
    restored.attempts.get_mut(IDS[0]).unwrap().scenario_version += 1;
    assert!(progress::save_with_catalog(&path, &restored, catalog()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}
