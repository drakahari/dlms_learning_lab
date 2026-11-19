//! Real authored preview outcomes. Registration is tested separately on integration.
use dlms_learning_lab::{engine, pack, progress, rest_controller as api, rest_workspace as work};
fn request(method: &str, path: &str, body: &str) -> api::Request {
    api::Request {
        method: method.into(),
        path: path.into(),
        authorization: "Bearer lab-operator".into(),
        content_type: if method == "PATCH" {
            "application/merge-patch+json".into()
        } else {
            "application/json".into()
        },
        body: body.into(),
    }
}
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!("fixtures/rest-controller-preview-v1.json")).unwrap()
}
fn apply(draft: &mut work::Draft, task: &work::Task, request: api::Request) -> u16 {
    draft.edit(request).unwrap();
    draft.send(task).unwrap().status
}
fn all(draft: &mut work::Draft, task: &work::Task) -> bool {
    draft.test(task).unwrap();
    draft.checks(task).iter().all(|c| c.0)
}

#[test]
fn taught_method_works_in_both_worlds_and_practice_cannot_complete_assignment() {
    for source in labs() {
        let d = work::source(&source).unwrap();
        let mut session = work::Session::new(d);
        let assignment = session.assignment.clone();
        for r in &d.example.reference {
            assert!((200..300).contains(&apply(&mut session.practice, &d.example, r.clone())));
        }
        assert!(all(&mut session.practice, &d.example));
        assert_eq!(session.assignment, assignment);
        assert!(!d.checks(&session).iter().all(|c| c.0));
        let mut state = source.initial.clone();
        for line in &source.solution {
            engine::execute_for_goal(&mut state, &source.goal, line).unwrap();
        }
        assert!(engine::solved(&state, &source.goal));
    }
}

#[test]
fn update_assignment_diagnoses_permission_encoding_types_and_preserves_fields() {
    let source = labs().remove(0);
    let task = &work::source(&source).unwrap().initial;
    let mut draft = work::Draft::new(task);
    let before = draft.store(task).unwrap();
    assert_eq!(draft.send(task).unwrap().status, 403);
    assert_eq!(draft.store(task).unwrap(), before);
    let mut r = draft.request.clone();
    r.authorization = "Bearer lab-operator".into();
    assert_eq!(apply(&mut draft, task, r.clone()), 415);
    r.content_type = "application/merge-patch+json".into();
    assert_eq!(apply(&mut draft, task, r), 422);
    assert!(!all(&mut draft, task));
    assert_eq!(draft.store(task).unwrap(), before);
    assert_eq!(
        apply(
            &mut draft,
            task,
            request(
                "PUT",
                "/profiles/staff",
                r#"{"description":"Staff desk service","up":true,"vlan":10}"#
            )
        ),
        204
    );
    assert!(all(&mut draft, task));
    apply(
        &mut draft,
        task,
        request("PUT", "/profiles/staff", r#"{"up":true,"vlan":10}"#),
    );
    assert!(!all(&mut draft, task));
    assert!(
        draft
            .checks(task)
            .iter()
            .any(|(ok, msg)| !ok && msg.contains("description"))
    );
    apply(
        &mut draft,
        task,
        request("PATCH", "/profiles/staff", r#"{"up":false}"#),
    );
    assert!(!draft.checked);
    assert!(!all(&mut draft, task));
}

#[test]
fn handover_accepts_another_new_name_and_put_sequence_but_partial_move_and_deletion_fail() {
    let source = labs().remove(1);
    let task = &work::source(&source).unwrap().initial;
    let mut draft = work::Draft::new(task);
    assert_eq!(
        apply(
            &mut draft,
            task,
            request("DELETE", "/profiles/visitors", "")
        ),
        409
    );
    assert_eq!(
        apply(
            &mut draft,
            task,
            request(
                "PUT",
                "/profiles/different-new-name",
                r#"{"vlan":30,"up":true,"description":"Equivalent handover"}"#
            )
        ),
        201
    );
    assert!(!all(&mut draft, task)); // unused perfect profile cannot earn credit
    assert_eq!(
        apply(
            &mut draft,
            task,
            request(
                "PUT",
                "/bindings/desk-3",
                r#"{"profile":"different-new-name"}"#
            )
        ),
        204
    );
    assert!(!all(&mut draft, task));
    assert!(
        draft
            .checks(task)
            .iter()
            .any(|(ok, msg)| !ok && msg.contains("Guest pair"))
    );
    assert_eq!(
        apply(
            &mut draft,
            task,
            request(
                "PUT",
                "/bindings/desk-4",
                r#"{"profile":"different-new-name"}"#
            )
        ),
        204
    );
    assert!(!all(&mut draft, task));
    assert!(
        draft
            .checks(task)
            .iter()
            .any(|(ok, msg)| !ok && msg.contains("Retire"))
    );
    assert_eq!(
        apply(
            &mut draft,
            task,
            request("DELETE", "/profiles/visitors", "")
        ),
        204
    );
    assert!(all(&mut draft, task));
    apply(
        &mut draft,
        task,
        request("PATCH", "/profiles/staff", r#"{"vlan":30}"#),
    );
    assert!(!all(&mut draft, task));
}

#[test]
fn saved_assignment_and_unfinished_example_survive_disk_resume_and_reset_is_selected() {
    let catalog = labs();
    let mut profile = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for source in &catalog {
        let def = work::source(source).unwrap();
        let mut attempt = progress::Attempt::new(source, 1);
        for line in &source.solution {
            engine::execute_for_goal(&mut attempt.state, &source.goal, line).unwrap();
        }
        let engine::State::RestController { simulation } = &mut attempt.state else {
            panic!()
        };
        simulation
            .practice
            .edit(request("GET", "/bindings", ""))
            .unwrap();
        profile.attempts.insert(source.id.clone(), attempt);
        assert!(progress::current_solved(
            &profile.attempts[&source.id],
            source
        ));
        simulation_is_distinct(def);
    }
    let root = tempfile::tempdir().unwrap().keep();
    let path = root.join("profile.json");
    progress::save_with_catalog(&path, &profile, &catalog).unwrap();
    let loaded = progress::load(&path, &catalog).unwrap();
    for source in &catalog {
        assert!(progress::current_solved(
            &loaded.attempts[&source.id],
            source
        ));
        let engine::State::RestController { simulation } = &loaded.attempts[&source.id].state
        else {
            panic!()
        };
        assert_eq!(simulation.practice.request.path, "/bindings");
    }
}
fn simulation_is_distinct(def: &work::Definition) {
    assert_ne!(def.initial.api, def.example.api);
    assert_ne!(def.initial.initial, def.example.initial);
}

#[test]
fn actual_registration_preserves_all439_scenarios_and172_units_in_prerequisite_order() {
    let old = pack::accepted439().unwrap();
    let current = pack::ccna_pilot().unwrap();
    assert_eq!((old.len(), current.len()), (439, 441));
    for source in &old {
        let retained = current.iter().find(|s| s.id == source.id).unwrap();
        assert_eq!(
            serde_json::to_value(retained).unwrap(),
            serde_json::to_value(source).unwrap()
        );
    }
    let old_units = dlms_learning_lab::course::accepted439(&old).unwrap();
    let units = dlms_learning_lab::course::ccna_pilot(&current).unwrap();
    assert_eq!((old_units.len(), units.len()), (172, 173));
    let without_new: Vec<_> = units
        .iter()
        .filter(|u| u.id != "rest-network-changes")
        .collect();
    assert_eq!(
        serde_json::to_value(without_new).unwrap(),
        serde_json::to_value(old_units).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "rest-network-changes")
        .unwrap();
    assert_eq!(units[at - 1].id, "structured-network-inventory");
    assert_eq!(
        units[at].labs,
        vec!["ccna-rest-change-request", "ccna-rest-profile-handover"]
    );
    for preview in labs() {
        let registered = current.iter().find(|s| s.id == preview.id).unwrap();
        let mut expected = preview;
        if expected.id == "ccna-rest-change-request" {
            expected.prerequisites = vec!["ccna-json-inventory-handover".into()];
        }
        assert_eq!(
            serde_json::to_value(registered).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
    }
}
