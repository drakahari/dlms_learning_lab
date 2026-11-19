use dlms_learning_lab::automation_workshop::{
    claim_review::{Assessment, Response, Task as ReviewTask},
    desired_config::Task as ConfigTask,
    plan_editor::Difference,
    workspace::{Action, Scope, Task, Work},
};

fn fixture(case: &str) -> (ReviewTask, ConfigTask, String) {
    let content: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/network_evidence_review_tasks_v1.json"
    ))
    .unwrap();
    let context = &content[case];
    (
        serde_json::from_value(context["review"].clone()).unwrap(),
        serde_json::from_value(context["config"]["task"].clone()).unwrap(),
        context["config"]["valid_document"].as_str().unwrap().into(),
    )
}

#[test]
fn reports_require_actual_evidence_and_distinguish_different_forecast_samples() {
    for (scope, expected) in [
        ("assignment", Assessment::Contradicted),
        ("example", Assessment::Supported),
    ] {
        let (review, _, _) = fixture(scope);
        review.validate().unwrap();
        let task = Task::ClaimReview {
            supplied: Box::new(review),
        };
        let mut work = Work::new(&task);
        for action in [
            Action::AssessClaim {
                assessment: Some(expected),
            },
            Action::AssessForecast {
                assessment: Some(expected),
            },
        ] {
            work.apply(&task, action).unwrap();
        }
        assert_eq!(
            work.outcomes(&task)
                .iter()
                .filter(|result| result.passed)
                .count(),
            2,
            "Guessing alone must not earn evidence credit"
        );
        for action in [
            Action::InspectClaim,
            Action::ReadForecastSample,
            Action::ClaimResponse {
                response: Response::RestoreCarrier,
            },
            Action::ApplyClaimResponse,
            Action::Check,
        ] {
            work.apply(&task, action).unwrap();
        }
        assert!(
            !work.outcomes(&task)[3].passed,
            "Changing the working carrier does not publish policy."
        );
        for action in [
            Action::ClaimResponse {
                response: Response::PublishRules,
            },
            Action::ApplyClaimResponse,
            Action::Check,
        ] {
            work.apply(&task, action).unwrap();
        }
        assert!(work.outcomes(&task).iter().all(|result| result.passed));
    }
}

#[test]
fn altered_report_clears_result_and_a_supplied_reading_starts_pending() {
    let (review, _, _) = fixture("assignment");
    let forecast = review.forecast;
    assert_eq!(forecast.assess(false), Assessment::Pending);
    assert_eq!(forecast.assess(true), Assessment::Contradicted);
    let mut border = forecast;
    border.next_sample = forecast.threshold_percent;
    assert_eq!(border.assess(true), Assessment::Supported);
    let task = Task::ClaimReview {
        supplied: Box::new(review),
    };
    let mut work = Work::new(&task);
    work.apply(&task, Action::Check).unwrap();
    work.apply(
        &task,
        Action::AssessClaim {
            assessment: Some(Assessment::Supported),
        },
    )
    .unwrap();
    assert!(!work.checked);
    let Scope::ClaimReview { review, .. } = work.current else {
        panic!();
    };
    assert!(!review.inspected_incident);
}

#[test]
fn desired_preview_detects_unapproved_removal_and_requires_a_fresh_second_check() {
    let (_, config, valid) = fixture("assignment");
    config.validate().unwrap();
    let mut edit = config.new_edit();
    let first = config.plan(&edit.applied, &edit.draft).unwrap();
    assert!(
        first
            .changes
            .iter()
            .any(|row| row.difference == Difference::Remove
                && row.resource.ends_with(&config.retained_profile))
    );
    assert!(edit.apply(&config).is_err());
    assert_eq!(edit.applied, config.initial);
    edit.preview(&config).unwrap();
    edit.change(valid).unwrap();
    assert!(!edit.previewed);
    assert!(edit.apply(&config).is_err());
    edit.preview(&config).unwrap();
    edit.apply(&config).unwrap();
    assert!(
        !config
            .outcomes(&edit)
            .unwrap()
            .iter()
            .find(|row| row.name.contains("remaining changes"))
            .unwrap()
            .passed
    );
    edit.preview(&config).unwrap();
    assert!(
        config
            .outcomes(&edit)
            .unwrap()
            .iter()
            .all(|finding| finding.passed)
    );
    let saved = edit.applied.clone();
    edit.change("{ broken draft".into()).unwrap();
    assert!(edit.preview(&config).is_err());
    assert!(edit.apply(&config).is_err());
    assert_eq!(edit.applied, saved);
    let checked = config.outcomes(&edit).unwrap();
    assert!(!checked[4].passed);
    assert!(
        checked[4]
            .detail
            .contains("Cannot read the supplied configuration")
    );
}

#[test]
fn equivalent_named_profile_passes_but_unapproved_changes_fail_actual_grading() {
    let (_, config, valid) = fixture("example");
    let mut document: dlms_learning_lab::rest_controller::Store =
        serde_json::from_str(&valid).unwrap();
    document.profiles.insert(
        "renamed-approved".into(),
        document.profiles["approved-service"].clone(),
    );
    for slot in &config.approved_slots {
        document.bindings.get_mut(slot).unwrap().profile = "renamed-approved".into();
    }
    let mut edit = config.new_edit();
    let apply = |edit: &mut dlms_learning_lab::automation_workshop::desired_config::Edit,
                 document: &dlms_learning_lab::rest_controller::Store| {
        edit.change(serde_json::to_string(document).unwrap())
            .unwrap();
        edit.preview(&config).unwrap();
        edit.apply(&config).unwrap();
        edit.preview(&config).unwrap();
    };
    apply(&mut edit, &document);
    assert!(
        config
            .outcomes(&edit)
            .unwrap()
            .iter()
            .all(|finding| finding.passed)
    );
    document.profiles.remove(&config.retained_profile);
    apply(&mut edit, &document);
    let checks = config.outcomes(&edit).unwrap();
    assert!(
        !checks[0].passed,
        "An unbound but necessary rule must remain."
    );
    assert!(
        checks[5..].iter().all(|finding| finding.passed),
        "A path-only grade misses the unrelated removal"
    );
}
