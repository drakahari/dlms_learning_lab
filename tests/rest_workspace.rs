use dlms_learning_lab::{rest_controller as api, rest_workspace as work, switching};
#[path = "support/rest_controller_fixture.rs"]
mod fixture;
use fixture::{fixture, request};

fn task(id: &str) -> work::Task {
    let (api, mut initial) = fixture();
    initial.profiles.get_mut("staff").unwrap().up = false;
    work::Task {
        id: id.into(),
        instructions: "Restore the staff path, preserve visitors and keep the staff description."
            .into(),
        api,
        initial,
        starter: api::Request {
            authorization: "Bearer lab-reader".into(),
            ..request("PATCH", "/profiles/staff", r#"{"up":"true"}"#)
        },
        reference: vec![request("PATCH", "/profiles/staff", r#"{"up":true}"#)],
        requirements: vec![
            work::Requirement::Network {
                requirement: switching::Requirement::Visibility {
                    label: "Staff conversation".into(),
                    from: "staff-a".into(),
                    to: "staff-b".into(),
                    visible: true,
                },
            },
            work::Requirement::Network {
                requirement: switching::Requirement::Visibility {
                    label: "Visitor conversation".into(),
                    from: "guest-a".into(),
                    to: "guest-b".into(),
                    visible: true,
                },
            },
            work::Requirement::Network {
                requirement: switching::Requirement::Visibility {
                    label: "Separate groups".into(),
                    from: "staff-a".into(),
                    to: "guest-a".into(),
                    visible: false,
                },
            },
            work::Requirement::PreserveProfile {
                name: "visitors".into(),
            },
            work::Requirement::PreserveDescription {
                name: "staff".into(),
            },
        ],
    }
}
fn definition() -> work::Definition {
    let initial = task("rest-update");
    let mut example = task("rest-example");
    example
        .initial
        .profiles
        .get_mut("staff")
        .unwrap()
        .description = Some("Practice staff".into());
    example
        .api
        .inventory
        .fabric
        .switches
        .get_mut("access")
        .unwrap()
        .vlans
        .insert(40);
    work::Definition {
        version: 1,
        initial,
        example,
    }
}
fn apply(draft: &mut work::Draft, task: &work::Task, req: api::Request) -> api::Response {
    draft.edit(req).unwrap();
    draft.send(task).unwrap()
}

#[test]
fn reachable_source_works_and_example_actions_do_not_grade_or_overwrite_assignment() {
    let def = definition();
    def.validate().unwrap();
    let mut session = work::Session::new(&def);
    let assignment = session.assignment.clone();
    assert!(!def.checks(&session).iter().all(|c| c.0));
    for req in &def.example.reference {
        assert_eq!(
            apply(&mut session.practice, &def.example, req.clone()).status,
            204
        );
    }
    session.practice.test(&def.example).unwrap();
    assert!(session.practice.checks(&def.example).iter().all(|c| c.0));
    assert_eq!(session.assignment, assignment);
    assert!(!def.checks(&session).iter().all(|c| c.0));
    for req in &def.initial.reference {
        apply(&mut session.assignment, &def.initial, req.clone());
    }
    session.assignment.test(&def.initial).unwrap();
    assert!(def.checks(&session).iter().all(|c| c.0));
}

#[test]
fn status_success_gets_and_created_unused_profiles_cannot_substitute_for_required_outcomes() {
    let task = task("rest-update");
    task.validate().unwrap();
    let mut draft = work::Draft::new(&task);
    assert_eq!(
        apply(&mut draft, &task, request("GET", "/profiles", "")).status,
        200
    );
    assert!(draft.changes.is_empty());
    draft.test(&task).unwrap();
    let partial = draft.checks(&task);
    assert!(partial.iter().any(|c| c.0) && partial.iter().any(|c| !c.0));
    assert_eq!(
        apply(
            &mut draft,
            &task,
            request(
                "POST",
                "/profiles",
                r#"{"name":"perfect-but-unused","up":true,"vlan":10}"#
            )
        )
        .status,
        201
    );
    draft.test(&task).unwrap();
    assert!(!draft.checks(&task).iter().all(|c| c.0));
    assert_eq!(
        apply(
            &mut draft,
            &task,
            request("PATCH", "/profiles/staff", r#"{"up":true}"#)
        )
        .status,
        204
    );
    draft.test(&task).unwrap();
    assert!(draft.checks(&task).iter().all(|c| c.0));
}

#[test]
fn equivalent_full_put_passes_but_lost_description_and_blanket_vlan_repairs_fail() {
    let task = task("rest-update");
    let mut draft = work::Draft::new(&task);
    assert_eq!(
        apply(
            &mut draft,
            &task,
            request(
                "PUT",
                "/profiles/staff",
                r#"{"up":true,"vlan":10,"description":"Staff workgroup"}"#
            )
        )
        .status,
        204
    );
    draft.test(&task).unwrap();
    assert!(draft.checks(&task).iter().all(|c| c.0));
    apply(
        &mut draft,
        &task,
        request("PUT", "/profiles/staff", r#"{"up":true,"vlan":10}"#),
    );
    draft.test(&task).unwrap();
    assert!(
        draft
            .checks(&task)
            .iter()
            .any(|(ok, msg)| !ok && msg.contains("description"))
    );
    apply(
        &mut draft,
        &task,
        request(
            "PATCH",
            "/profiles/staff",
            r#"{"description":"Staff workgroup","vlan":20}"#,
        ),
    );
    draft.test(&task).unwrap();
    assert!(
        draft
            .checks(&task)
            .iter()
            .any(|(ok, msg)| !ok && msg.contains("Separate groups"))
    );
    apply(
        &mut draft,
        &task,
        request("PATCH", "/profiles/visitors", r#"{"vlan":10}"#),
    );
    draft.test(&task).unwrap();
    assert!(
        draft
            .checks(&task)
            .iter()
            .any(|(ok, msg)| !ok && msg.contains("protects another"))
    );
}

#[test]
fn changes_invalidate_success_failed_requests_are_atomic_and_readonly_repeats_do_not_inflate_history()
 {
    let task = task("rest-update");
    let mut draft = work::Draft::new(&task);
    apply(&mut draft, &task, task.reference[0].clone());
    draft.test(&task).unwrap();
    assert!(draft.checked);
    let applied = draft.store(&task).unwrap();
    draft
        .edit(request("PATCH", "/profiles/staff", r#"{"up":false}"#))
        .unwrap();
    assert!(!draft.checked);
    assert!(!draft.checks(&task).iter().all(|c| c.0));
    assert_eq!(draft.store(&task).unwrap(), applied);
    draft.send(&task).unwrap();
    draft.test(&task).unwrap();
    assert!(!draft.checks(&task).iter().all(|c| c.0));
    let broken = draft.store(&task).unwrap();
    let response = apply(
        &mut draft,
        &task,
        request("PATCH", "/profiles/staff", r#"{"up":true,"vlan":999}"#),
    );
    assert_eq!(response.status, 422);
    assert_eq!(draft.store(&task).unwrap(), broken);
    assert!(!draft.checked);
    let len = draft.changes.len();
    for _ in 0..70 {
        assert_eq!(
            apply(&mut draft, &task, request("GET", "/profiles", "")).status,
            200
        );
    }
    assert_eq!(draft.changes.len(), len);
}

#[test]
fn serialized_resume_replays_source_changes_reset_is_selected_and_forged_history_refuses() {
    let def = definition();
    let mut session = work::Session::new(&def);
    apply(
        &mut session.assignment,
        &def.initial,
        def.initial.reference[0].clone(),
    );
    session.assignment.test(&def.initial).unwrap();
    session
        .practice
        .edit(request("GET", "/bindings", ""))
        .unwrap();
    let bytes = serde_json::to_vec(&session).unwrap();
    let mut resumed: work::Session = serde_json::from_slice(&bytes).unwrap();
    resumed.bound(&def).unwrap();
    assert!(def.checks(&resumed).iter().all(|c| c.0));
    assert_eq!(resumed, session);
    let practice = resumed.practice.clone();
    resumed.assignment = work::Draft::new(&def.initial);
    assert_eq!(resumed.practice, practice);
    assert!(!def.checks(&resumed).iter().all(|c| c.0));
    let mut forged = session.clone();
    forged
        .assignment
        .changes
        .push(request("GET", "/profiles", ""));
    assert!(forged.bound(&def).is_err());
    let mut forged = session.clone();
    forged.assignment.changes[0].authorization = "Bearer lab-reader".into();
    assert!(forged.bound(&def).is_err());
    let mut forged = session;
    forged.context = "different-task".into();
    assert!(forged.bound(&def).is_err());
}

#[test]
fn history_limits_are_atomic_and_source_validation_refuses_unreachable_or_unprotected_goals() {
    let task = task("rest-update");
    let mut draft = work::Draft::new(&task);
    for i in 0..64 {
        let up = i % 2 == 0;
        assert_eq!(
            apply(
                &mut draft,
                &task,
                request("PATCH", "/profiles/staff", &format!(r#"{{"up":{up}}}"#))
            )
            .status,
            204
        );
    }
    assert_eq!(draft.changes.len(), 64);
    let applied = draft.store(&task).unwrap();
    draft
        .edit(request("PATCH", "/profiles/staff", r#"{"up":true}"#))
        .unwrap();
    assert!(draft.send(&task).unwrap_err().contains("64"));
    assert_eq!(draft.store(&task).unwrap(), applied);
    let mut bad = task.clone();
    bad.reference.clear();
    assert!(bad.validate().is_err());
    let mut bad = task.clone();
    bad.reference = vec![request("GET", "/profiles", "")];
    assert!(bad.validate().is_err());
    let mut bad = task;
    bad.requirements.retain(|r| {
        !matches!(
            r,
            work::Requirement::Network {
                requirement: switching::Requirement::Visibility { visible: true, .. }
            }
        )
    });
    assert!(bad.validate().is_err());
}
