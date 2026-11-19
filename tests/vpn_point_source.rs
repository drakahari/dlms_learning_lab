//! Fixed unregistered source fixture; no invented authored-lab credit.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    vpn_point_source as binding,
    vpn_point_workspace::{Action, Session},
};
#[path = "support/vpn_point_fixture.rs"]
mod fixture;
#[path = "support/vpn_point_task_fixture.rs"]
mod task_fixture;
#[path = "support/vpn_point_workspace_fixture.rs"]
mod work_fixture;
fn reference(a: &Action, practice: bool) -> String {
    format!(
        "{}{}",
        binding::PREFIX,
        serde_json::json!({"practice":practice,"action":a})
    )
}
fn scenario() -> Scenario {
    let d = work_fixture::definition();
    serde_json::from_value(serde_json::json!({
  "id":d.source_id,"version":1,"track":"LPIC-2","topic":"Fixed VPN source fixture","title":"Virtual peer repair fixture","summary":"Unregistered source test, not curriculum.","objective":"Repair the supplied peer policy and preserve both full LAN route requirements.","difficulty":"Intermediate","minutes":20,
  "steps":["Inspect supplied facts","Edit peer profiles","Apply and verify"],
  "guide":[
   {"title":"Outside and tunnel addresses","why":"The outside endpoint and virtual tunnel address have different roles.","action":"Inspect the two peer editors and supplied facts.","commands":[]},
   {"title":"Supplied identity facts","why":"Authority, identity and key-pair references are different checks.","action":"Choose the provided profile references for each peer.","commands":[]},
   {"title":"Apply and verify","why":"Saved text and applied settings are distinct.","action":"Save, Apply profiles and Check configuration.","commands":[]}
  ],"hints":["Compare the peer endpoint and its configured port."],"sources":["https://openvpn.net/community-docs/community-articles/openvpn-2-6-manual/"],"explanation":"Evaluate the supplied peer facts and actual virtual route state, not exact profile spelling.","simplifications":"Finite metadata/route model, not TLS, host interfaces, real packets or a deployable OpenVPN configuration.",
  "initial":State::VpnPoint{simulation:Box::new(Session::new(&d).unwrap())},"goal":Goal::VpnPoint{definition:Box::new(d)},"solution":work_fixture::repair().iter().map(|a|reference(a,false)).collect::<Vec<_>>()
 })).unwrap()
}
fn act(state: &mut State, s: &Scenario, a: &Action, practice: bool) {
    engine::execute_for_goal(state, &s.goal, &reference(a, practice)).unwrap();
}
fn session(state: &mut State) -> &mut Session {
    let State::VpnPoint { simulation } = state else {
        panic!()
    };
    simulation
}
#[test]
fn fixture_loads_and_real_taught_actions_keep_practice_ungraded() {
    let s = scenario();
    binding::source(&s).unwrap();
    let d = work_fixture::definition();
    let loaded = pack::load(
        &serde_json::to_string(&pack::Pack {
            schema_version: 1,
            scenarios: vec![s.clone()],
        })
        .unwrap(),
    )
    .unwrap();
    assert_eq!(loaded.len(), 1);
    let mut state = s.initial.clone();
    let assignment = session(&mut state).assignment.clone();
    for a in &d.worked {
        act(&mut state, &s, a, true);
    }
    assert_eq!(session(&mut state).assignment, assignment);
    assert!(!engine::solved(&state, &s.goal));
    for raw in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, raw).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
    assert_eq!(
        session(&mut state).assignment.world,
        task_fixture::repaired(&d.assignment)
    );
    assert_eq!(
        session(&mut state).assignment.world.applied,
        Some(fixture::configs())
    );
}
#[test]
fn partial_unchecked_edited_and_broken_results_cannot_remain_successful() {
    let s = scenario();
    let mut state = s.initial.clone();
    let actions = work_fixture::repair();
    act(&mut state, &s, &actions[0], false);
    act(&mut state, &s, &Action::Check, false);
    assert!(!engine::solved(&state, &s.goal));
    for a in &actions[..3] {
        act(&mut state, &s, a, false);
    }
    assert!(!engine::solved(&state, &s.goal));
    act(&mut state, &s, &Action::Check, false);
    assert!(engine::solved(&state, &s.goal));
    act(
        &mut state,
        &s,
        &Action::Edit {
            peer: 1,
            text: fixture::peer(false).replace("1194", "1195"),
        },
        false,
    );
    assert!(!engine::solved(&state, &s.goal));
    for a in [Action::Save { peer: 1 }, Action::Apply, Action::Check] {
        act(&mut state, &s, &a, false);
    }
    assert!(!engine::solved(&state, &s.goal));
}
#[test]
fn wrong_sources_fake_reports_and_internal_envelope_abuse_are_refused_atomically() {
    let s = scenario();
    let mut state = s.initial.clone();
    let before = serde_json::to_value(&state).unwrap();
    for text in [
        "openvpn --config /etc/passwd",
        "sh -c anything",
        "vpn-point-workspace {\"practice\":false,\"action\":{\"kind\":\"Check\"},\"script\":\"escape\"}",
    ] {
        assert!(engine::execute_for_goal(&mut state, &s.goal, text).is_err());
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
    }
    assert!(engine::execute(&mut state, "anything").is_err());
    let mut wrong = s.clone();
    wrong.track = "CCNA".into();
    assert!(binding::source(&wrong).is_err());
    wrong = s.clone();
    wrong.guide[0].commands.push(pack::CommandExample {
        command: "openvpn --config /etc/openvpn/client.conf".into(),
        explanation: "Not a supported terminal workflow.".into(),
        simulator_only: false,
    });
    assert!(binding::source(&wrong).is_err());
}
#[test]
fn progress_decode_preserves_favorites_drafts_history_and_real_current_credit() {
    let s = scenario();
    let mut a = Attempt::new(&s, 0);
    for raw in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, raw).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    assert_eq!(a.step, 0, "Passing is independent of walkthrough position");
    act(
        &mut a.state,
        &s,
        &Action::Edit {
            peer: 0,
            text: fixture::peer(true) + "# unfinished assignment\n",
        },
        false,
    );
    act(
        &mut a.state,
        &s,
        &Action::Edit {
            peer: 0,
            text: fixture::peer(true) + "# separate example\n",
        },
        true,
    );
    let mut p = Progress {
        schema_version: 1,
        ..Progress::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let raw = serde_json::to_vec(&p).unwrap();
    let restored = progress::decode(&raw, std::slice::from_ref(&s)).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(!progress::current_solved(&restored.attempts[&s.id], &s));
    let mut bad = restored;
    session(&mut bad.attempts.get_mut(&s.id).unwrap().state)
        .assignment
        .source_task = "{}".into();
    assert!(
        progress::decode(&serde_json::to_vec(&bad).unwrap(), std::slice::from_ref(&s)).is_err()
    );
}
#[test]
fn catalog_aware_safe_writes_reject_invalid_state_and_preserve_last_good_bytes() {
    let s = scenario();
    let mut p = Progress {
        schema_version: 1,
        ..Progress::default()
    };
    p.attempts.insert(s.id.clone(), Attempt::new(&s, 0));
    let root = std::path::PathBuf::from(
        std::env::var_os("TMPDIR").expect("MAIN disk-backed runner required"),
    )
    .join(format!(
        "vpn-source-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("progress-v1.json");
    std::fs::write(
        root.join("source-fixture.json"),
        serde_json::to_vec_pretty(&s).unwrap(),
    )
    .unwrap();
    assert!(progress::save(&path, &p).is_err());
    assert!(!path.exists());
    progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
    let before = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, std::slice::from_ref(&s)).unwrap();
    binding::context(&loaded.attempts[&s.id], &s).unwrap();
    session(&mut p.attempts.get_mut(&s.id).unwrap().state)
        .assignment
        .source_task = "{}".into();
    assert!(progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    // Tiny exact-path synthetic fixture retained; no personal profile or cleanup.
}
