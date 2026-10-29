use dlms_learning_lab::{
    physical_link as core, physical_link_assignment as work, physical_link_teaching as teaching,
};
#[path = "support/physical_assignment_fixture.rs"]
mod fixture;
fn lesson() -> (work::Definition, teaching::Lesson) {
    let assignment = fixture::fixture("assignment");
    let example = fixture::fixture("example");
    assert!(
        example
            .definition
            .validate_reference(&example.neighbor_broken)
            .is_err()
    );
    let lesson=teaching::Lesson{version:1,introduction:"Inspect both endpoints and use fresh sample traffic before trusting old counters.".into(),concepts:vec![teaching::Concept{title:"Apply and check".into(),text:"Editing prepares a configuration; Apply changes only these simulated ports. Check uses independent sample traffic.".into()}],practice:example.definition,reference:example.reference,try_change:example.partial,repair:example.alternate,experiment_description:"Try the other supplied cable, Apply, then compare fresh receive errors.".into()};
    (assignment.definition, lesson)
}
#[test]
fn worked_fill_only_edits_separate_draft_then_apply_and_reset_preserve_assignment() {
    let (assignment, lesson) = lesson();
    let before = assignment.initial.clone();
    let mut learning = teaching::Learning::new(&lesson, &assignment).unwrap();
    let initial = learning.current.clone();
    learning.fill_worked_example(&lesson).unwrap();
    assert_eq!(learning.current, initial);
    assert!(!learning.checks(&lesson).iter().all(|r| r.0));
    assert_eq!(assignment.initial, before);
    learning.apply(&lesson).unwrap();
    assert!(learning.checks(&lesson).iter().all(|r| r.0));
    learning.reset(&lesson, &assignment).unwrap();
    assert_eq!(learning.current, initial);
    assert_eq!(assignment.initial, before);
}
#[test]
fn adverse_retry_drafts_resume_full_source_pin_and_real_probe_are_independent() {
    let (assignment, lesson) = lesson();
    let mut learning = teaching::Learning::new(&lesson, &assignment).unwrap();
    learning.fill_worked_example(&lesson).unwrap();
    learning.apply(&lesson).unwrap();
    let report = learning.probe(&lesson, "example-uplink", 8, 8).unwrap();
    assert_eq!(report.after["example-left"].received, 8);
    let mut draft = learning.draft.clone();
    draft.configuration = lesson.try_change.clone();
    learning.set_draft(&lesson, &draft).unwrap();
    assert!(!learning.checks(&lesson).iter().all(|r| r.0));
    let raw = serde_json::to_vec(&learning).unwrap();
    let mut resumed: teaching::Learning = serde_json::from_slice(&raw).unwrap();
    assert!(resumed.valid_for(&lesson));
    resumed.apply(&lesson).unwrap();
    assert!(!resumed.checks(&lesson).iter().all(|r| r.0));
    draft.configuration = lesson.repair.clone();
    resumed.set_draft(&lesson, &draft).unwrap();
    resumed.apply(&lesson).unwrap();
    assert!(resumed.checks(&lesson).iter().all(|r| r.0));
    let mut changed = lesson.clone();
    changed.introduction.push_str(" Different source.");
    assert!(!resumed.valid_for(&changed));
    let before = serde_json::to_value(&resumed).unwrap();
    assert!(resumed.fill_worked_example(&changed).is_err());
    assert_eq!(serde_json::to_value(&resumed).unwrap(), before);
}
#[test]
fn foreign_example_identity_invalid_partial_and_excessive_content_never_replace_work() {
    let (assignment, mut lesson) = lesson();
    lesson.practice = assignment.clone();
    assert!(lesson.validate(&assignment).is_err());
    let (assignment, lesson) = self::lesson();
    let mut learning = teaching::Learning::new(&lesson, &assignment).unwrap();
    let before = serde_json::to_value(&learning).unwrap();
    let mut draft = learning.draft.clone();
    draft.selected_port = "unknown".into();
    assert!(learning.set_draft(&lesson, &draft).is_err());
    assert_eq!(serde_json::to_value(&learning).unwrap(), before);
    assert!(
        teaching::Lesson::from_json(&vec![b' '; teaching::MAX_LESSON + 1], &assignment).is_err()
    );
    let mut bad = lesson.clone();
    bad.experiment_description = "x".repeat(1201);
    assert!(bad.validate(&assignment).is_err());
    let mut draft = learning.draft.clone();
    draft
        .configuration
        .ports
        .get_mut("example-left")
        .unwrap()
        .negotiation = core::Negotiation::Auto {
        advertise: Default::default(),
    };
    learning.set_draft(&lesson, &draft).unwrap();
    let retained = serde_json::to_value(&learning).unwrap();
    assert!(learning.apply(&lesson).is_err());
    assert_eq!(serde_json::to_value(&learning).unwrap(), retained);
}
#[test]
fn oversized_concept_body_refuses_before_loading_extra_instructional_content() {
    let (assignment, lesson) = lesson();
    let mut value = serde_json::to_value(&lesson).unwrap();
    let concept = value["concepts"][0].clone();
    let mut concepts = vec![concept; 6];
    concepts.push(serde_json::Value::Null);
    value["concepts"] = concepts.into();
    let e =
        teaching::Lesson::from_json(&serde_json::to_vec(&value).unwrap(), &assignment).unwrap_err();
    assert!(e.contains("before body"), "{e}");
}
