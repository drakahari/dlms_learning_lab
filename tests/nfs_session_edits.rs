use dlms_learning_lab::{
    engine::State, nfs as core, nfs_teaching as learn, nfs_workspace as work, pack,
    progress::Attempt,
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

fn exact(a: &Attempt, b: &serde_json::Value) {
    assert!(
        serde_json::to_value(a).unwrap() == *b,
        "complete Attempt retained"
    );
}
#[test]
fn source_aware_request_edits_never_write_and_preserve_separate_sessions() {
    for n in 0..4 {
        let s = scenario(n);
        let mut a = Attempt::new(&s, 1);
        learn::prepare(&mut a, &s, true).unwrap();
        let initial = serde_json::to_value(&a.state).unwrap();
        let State::Nfs { simulation } = &a.state else {
            panic!()
        };
        let c = simulation.inventory().clients[0].clone();
        let server = simulation.inventory().servers[0].id.clone();
        let source = s.nfs_teaching.as_ref().unwrap();
        let gallery = &source.definition.initial.inventory().clients[0];
        let gallery_server = &source.definition.initial.inventory().servers[0].id;
        work::edit_assignment_session(
            &mut a,
            &s,
            work::SessionEdit {
                server: Some(&server),
                client: Some(&c.id),
                profile: Some(&c.profiles[0]),
                mountpoint: Some(&c.mountpoints[0]),
                relative: Some("../unfinished"),
                write_text: Some("unsubmitted replacement\n"),
            },
        )
        .unwrap();
        let assigned = serde_json::to_value(&a.workspace.as_ref().unwrap().nfs_session).unwrap();
        work::edit_practice_session(
            &mut a,
            &s,
            work::SessionEdit {
                server: Some(gallery_server),
                client: Some(&gallery.id),
                profile: Some(&gallery.profiles[0]),
                mountpoint: Some(&gallery.mountpoints[0]),
                relative: Some("unfinished/example"),
                write_text: Some("unsubmitted gallery\n"),
            },
        )
        .unwrap();
        assert!(serde_json::to_value(&a.state).unwrap() == initial);
        assert!(
            serde_json::to_value(&a.workspace.as_ref().unwrap().nfs_session).unwrap() == assigned
        );
        assert!(
            a.workspace
                .as_ref()
                .unwrap()
                .nfs_learning
                .as_ref()
                .unwrap()
                .session
                .entries
                .is_empty()
        );
        let before = serde_json::to_value(&a).unwrap();
        assert!(
            work::edit_practice_session(
                &mut a,
                &s,
                work::SessionEdit {
                    client: Some(&c.id),
                    ..Default::default()
                }
            )
            .is_err()
        );
        exact(&a, &before);
        assert!(
            work::edit_assignment_session(
                &mut a,
                &s,
                work::SessionEdit {
                    write_text: Some("\0"),
                    ..Default::default()
                }
            )
            .is_err()
        );
        exact(&a, &before);
    }
}
#[test]
fn changed_client_clears_only_invalid_dependents_and_explicit_values_win() {
    let mut s = scenario(0);
    let State::Nfs { simulation } = &s.initial else {
        panic!()
    };
    let mut inventory = simulation.inventory().clone();
    let mut runtime = simulation.runtime().clone();
    let mut profile = inventory.profiles[0].clone();
    profile.id = "p-isolated".into();
    inventory.profiles.push(profile);
    // The technical topology already uses all16 mount slots. Replace one
    // unmounted, unreferenced outside-client slot before adding this independent
    // selector context; retain the real core bound and all exercised paths.
    let outside = inventory
        .clients
        .iter_mut()
        .find(|c| c.id.ends_with("-outside"))
        .unwrap();
    let spare = outside
        .mountpoints
        .iter()
        .position(|p| p.ends_with("/reports"))
        .unwrap();
    outside.mountpoints.remove(spare);
    inventory.clients.push(core::Client {
        id: "c-isolated".into(),
        address: "198.51.100.99".parse().unwrap(),
        mountpoints: vec!["/mnt/isolated".into()],
        profiles: vec!["p-isolated".into()],
    });
    runtime.clients.push(core::ClientLayers {
        client: "c-isolated".into(),
        draft: String::new(),
        saved: String::new(),
    });
    assert_eq!(
        inventory
            .clients
            .iter()
            .map(|c| c.mountpoints.len())
            .sum::<usize>(),
        16,
        "setup stays within the actual total mount-point limit"
    );
    let state = core::State::new(inventory, runtime).unwrap();
    s.initial = State::Nfs {
        simulation: state.clone(),
    };
    let dlms_learning_lab::engine::Goal::Nfs { definition } = &mut s.goal else {
        panic!()
    };
    definition.initial = state;
    definition.validate().unwrap();
    let mut a = Attempt::new(&s, 1);
    learn::prepare(&mut a, &s, true).unwrap();
    let State::Nfs { simulation } = &a.state else {
        panic!()
    };
    let c = simulation.inventory().clients[0].clone();
    work::edit_assignment_session(
        &mut a,
        &s,
        work::SessionEdit {
            client: Some(&c.id),
            mountpoint: Some(&c.mountpoints[0]),
            profile: Some(&c.profiles[0]),
            relative: Some("keep.txt"),
            write_text: Some("keep replacement"),
            ..Default::default()
        },
    )
    .unwrap();
    work::edit_assignment_session(
        &mut a,
        &s,
        work::SessionEdit {
            client: Some("c-isolated"),
            ..Default::default()
        },
    )
    .unwrap();
    let t = a.workspace.as_ref().unwrap().nfs_session.as_ref().unwrap();
    assert!(t.profile.is_empty() && t.mountpoint.is_empty());
    assert_eq!(t.relative, "keep.txt");
    assert_eq!(t.write_text, "keep replacement");
    work::edit_assignment_session(
        &mut a,
        &s,
        work::SessionEdit {
            profile: Some("p-isolated"),
            mountpoint: Some("/mnt/isolated"),
            ..Default::default()
        },
    )
    .unwrap();
    let before = serde_json::to_value(&a).unwrap();
    assert!(
        work::edit_assignment_session(
            &mut a,
            &s,
            work::SessionEdit {
                client: Some(&c.id),
                profile: Some("p-isolated"),
                ..Default::default()
            }
        )
        .is_err()
    );
    exact(&a, &before);
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
fn complete_escaped_attempt_frontier_and_presentation_refusals_are_atomic() {
    let s = scenario(0);
    let mut low = Attempt::new(&s, 1);
    learn::prepare(&mut low, &s, true).unwrap();
    let text = "\u{1}".repeat(4096);
    work::edit_assignment_session(
        &mut low,
        &s,
        work::SessionEdit {
            write_text: Some(&text),
            ..Default::default()
        },
    )
    .unwrap();
    let mut high = Attempt::new(&s, 1);
    learn::prepare(&mut high, &s, true).unwrap();
    fill_attempt(&mut high);
    let before = serde_json::to_value(&high).unwrap();
    assert!(
        work::edit_assignment_session(
            &mut high,
            &s,
            work::SessionEdit {
                write_text: Some(&text),
                ..Default::default()
            }
        )
        .is_err()
    );
    exact(&high, &before);
    assert!(
        work::edit_practice_session(
            &mut high,
            &s,
            work::SessionEdit {
                write_text: Some(&text),
                ..Default::default()
            }
        )
        .is_err()
    );
    exact(&high, &before);
    work::set_practice_presentation(
        &mut low,
        &s,
        Some(dlms_learning_lab::script_teaching::View::Assignment),
        None,
    )
    .unwrap();
    let l = low
        .workspace
        .as_ref()
        .unwrap()
        .nfs_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.view, dlms_learning_lab::script_teaching::View::Assignment);
    let before = serde_json::to_value(&low).unwrap();
    assert!(work::set_practice_presentation(&mut low, &s, None, Some(16)).is_err());
    exact(&low, &before);
}

#[test]
fn recency_digit_boundary_substitutes_all_attempt_bytes_and_preserves_every_other_field() {
    let s = scenario(0);
    for metadata in [false, true] {
        let mut low = Attempt::new(&s, 9);
        if metadata {
            learn::prepare(&mut low, &s, true).unwrap();
        }
        let before = serde_json::to_value(&low).unwrap();
        let mut expected = before.clone();
        expected["visited"] = json!(10);
        work::record_visit(&mut low, &s, 10).unwrap();
        exact(&low, &expected);
        let mut high = Attempt::new(&s, 9);
        if metadata {
            learn::prepare(&mut high, &s, true).unwrap();
        }
        fill_attempt(&mut high);
        let before = serde_json::to_value(&high).unwrap();
        assert!(work::record_visit(&mut high, &s, 10).is_err());
        exact(&high, &before);
        work::record_visit(&mut high, &s, 8).unwrap();
        let mut expected = before;
        expected["visited"] = json!(8);
        exact(&high, &expected);
    }
}
