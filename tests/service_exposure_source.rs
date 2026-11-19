//! Fixed source-binding fixture; these tests add no authored catalog credit.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    service_exposure_source as binding,
    service_exposure_workspace::{Action, Event, Session},
};
#[path = "support/service_exposure_fixture.rs"]
mod server_fixture;
#[path = "support/service_exposure_task_fixture.rs"]
mod task_fixture;
#[path = "support/service_exposure_workspace_fixture.rs"]
mod workspace_fixture;
fn reference(a: &Action, practice: bool) -> String {
    format!(
        "{}{}",
        binding::PREFIX,
        serde_json::json!({"practice":practice,"action":a})
    )
}
fn fixture() -> Scenario {
    let d = workspace_fixture::definition();
    serde_json::from_value(serde_json::json!({
        "id":d.source_id,"version":1,"track":"LPIC-2","topic":"Service-exposure test fixture",
        "title":"Fixed service-exposure source fixture","summary":"Unregistered source binding fixture.",
        "objective":"Repair the supplied virtual service access while preserving required owners.",
        "difficulty":"Intermediate","minutes":20,"steps":["Inspect","Compare","Repair"],
        "guide":[
            {"title":"Inspect listeners","why":"A retained socket and a successful application are different facts.","action":"Run ss in Server terminal.","commands":[{"command":"ss -lntp","explanation":"Inspect the supplied listener metadata.","simulator_only":false}]},
            {"title":"Compare viewpoints","why":"A dropped probe does not identify whether a listener exists.","action":"Run the bounded example scan from Observer terminal.","commands":[{"command":"nmap -n -Pn -sT -p 22,80,443,9000 203.0.113.42","explanation":"Observe only the supplied virtual example server.","simulator_only":false}]},
            {"title":"Repair and verify","why":"The full client range must meet the access requirement.","action":"Repair the example policy and check both observers.","commands":[{"command":"iptables -A INPUT -s 192.0.2.0/24 -p tcp --dport 22 -j DROP","explanation":"Apply a numeric source-and-port filter to the simulated host.","simulator_only":false}]}
        ],
        "hints":["Keep server administration separate from observer commands."],
        "sources":["https://nmap.org/book/man-port-scanning-basics.html"],
        "explanation":"Use the modeled state and policy requirements rather than exact command spelling.",
        "simplifications":"Only the supplied path and bounded numeric TCP contexts; no host calls.",
        "initial":State::ServiceExposure{simulation:Box::new(Session::new(&d).unwrap())},
        "goal":Goal::ServiceExposure{definition:Box::new(d)},
        "solution":workspace_fixture::repair().iter().map(|a|reference(a,false)).collect::<Vec<_>>()
    })).unwrap()
}
fn execute(state: &mut State, s: &Scenario, a: &Action, practice: bool) {
    engine::execute_for_goal(state, &s.goal, &reference(a, practice)).unwrap();
}
fn session(s: &mut State) -> &mut Session {
    let State::ServiceExposure { simulation } = s else {
        panic!()
    };
    simulation
}
#[test]
fn fixed_pack_taught_methods_and_separate_example_are_reachable_without_credit_leakage() {
    let s = fixture();
    binding::source(&s).unwrap();
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
    for a in workspace_fixture::repair() {
        execute(&mut state, &s, &a, true);
    }
    assert!(!engine::solved(&state, &s.goal));
    for c in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
    assert_eq!(
        session(&mut state).assignment.world,
        task_fixture::solve(&workspace_fixture::definition().assignment)
    );
}
#[test]
fn partial_solutions_edits_and_unchecked_current_state_cannot_remain_successful() {
    let s = fixture();
    let mut state = s.initial.clone();
    let actions = workspace_fixture::repair();
    execute(&mut state, &s, &actions[0], false);
    execute(
        &mut state,
        &s,
        &workspace_fixture::action(Event::Check),
        false,
    );
    assert!(!engine::solved(&state, &s.goal));
    execute(&mut state, &s, &actions[1], false);
    assert!(!engine::solved(&state, &s.goal));
    execute(&mut state, &s, &actions[2], false);
    assert!(engine::solved(&state, &s.goal));
    execute(
        &mut state,
        &s,
        &workspace_fixture::action(Event::Admin {
            input: "iptables -F INPUT".into(),
        }),
        false,
    );
    assert!(!engine::solved(&state, &s.goal));
}
#[test]
fn malformed_source_unsupported_guidance_and_wrong_native_actions_fail_honestly() {
    let s = fixture();
    let mut bad = s.clone();
    bad.track = "CCNA".into();
    assert!(binding::source(&bad).is_err());
    let mut bad = s.clone();
    bad.guide[0].commands[0].command = "sh -c touch nope".into();
    assert!(binding::source(&bad).is_err());
    let mut bad = s.clone();
    let Goal::ServiceExposure { definition } = &mut bad.goal else {
        panic!()
    };
    definition.worked = vec![workspace_fixture::action(Event::Check)];
    assert!(binding::source(&bad).is_err());
    let mut state = s.initial.clone();
    let before = state.clone();
    assert!(engine::execute_for_goal(&mut state, &s.goal, "ss -lntp").is_err());
    assert_eq!(state, before);
    assert!(
        engine::execute_for_goal(
            &mut state,
            &s.goal,
            "service-exposure-workspace {\"practice\":false,\"action\":{\"kind\":\"HostExec\"}}"
        )
        .is_err()
    );
    assert_eq!(state, before);
}
#[test]
fn progress_decode_retains_favorites_drafts_and_history_and_rejects_forged_results() {
    let s = fixture();
    let mut a = Attempt::new(&s, 0);
    execute(
        &mut a.state,
        &s,
        &Action::AdminInput {
            text: "ss -lntp".into(),
        },
        false,
    );
    execute(
        &mut a.state,
        &s,
        &workspace_fixture::action(Event::Select { viewpoint: 1 }),
        true,
    );
    let mut p = Progress {
        schema_version: 1,
        ..Progress::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    assert!(
        progress::decode(
            &serde_json::to_vec(&Progress::default()).unwrap(),
            std::slice::from_ref(&s)
        )
        .is_err()
    );
    let bytes = serde_json::to_vec(&p).unwrap();
    let restored = progress::decode(&bytes, std::slice::from_ref(&s)).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let mut bad = restored;
    session(&mut bad.attempts.get_mut(&s.id).unwrap().state)
        .assignment
        .admin_output = "Forged result".into();
    assert!(
        progress::decode(&serde_json::to_vec(&bad).unwrap(), std::slice::from_ref(&s)).is_err()
    );
    binding::context(&p.attempts[&s.id], &s).unwrap();
}
#[test]
fn catalog_aware_safe_writes_preserve_last_good_file_on_rejected_state() {
    let s = fixture();
    let mut p = Progress {
        schema_version: 1,
        ..Progress::default()
    };
    p.attempts.insert(s.id.clone(), Attempt::new(&s, 0));
    let root = std::path::PathBuf::from(
        std::env::var_os("TMPDIR").expect("MAIN project-disk runner required"),
    )
    .join(format!(
        "exposure-source-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("progress-v1.json");
    assert!(progress::save(&path, &p).is_err());
    assert!(!path.exists());
    std::fs::write(
        root.join("source-fixture.json"),
        serde_json::to_vec_pretty(&s).unwrap(),
    )
    .unwrap();
    progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
    let before = std::fs::read(&path).unwrap();
    let restored = progress::load(&path, std::slice::from_ref(&s)).unwrap();
    binding::context(&restored.attempts[&s.id], &s).unwrap();
    session(&mut p.attempts.get_mut(&s.id).unwrap().state)
        .assignment
        .checked = true;
    assert!(progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    // Retain the tiny exact-path synthetic progress fixture; never owner progress.
}
