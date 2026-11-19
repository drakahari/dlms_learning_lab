use dlms_learning_lab::security_investigation::{
    access_plan::{Access, Reports},
    change_evidence::{self, Report},
    session::Definition,
    workspace::{Action, Scope, Task, Work},
};

fn fixture(id: &str) -> (Definition, serde_json::Value) {
    let all: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/security_investigation_tasks_v1.json"
    ))
    .unwrap();
    let task = &all[id];
    (
        serde_json::from_value(task["definition"].clone()).unwrap(),
        task["answers"].clone(),
    )
}

const CHANGE: &str = "ccna-management-change-evidence";
const FACILITY: &str = "ccna-facility-access-plan";

#[test]
fn journal_records_only_count_for_the_affected_device_client_and_time() {
    let (d, answers) = fixture(CHANGE);
    let Task::Change { supplied } = &d.assignment else {
        panic!()
    };
    let valid: Report = serde_json::from_value(answers["assignment"]["report"].clone()).unwrap();
    let evaluate = |report: &Report| {
        change_evidence::findings(
            &supplied.journal,
            &supplied.network.management.device,
            &supplied.affected_trial,
            report,
        )
    };
    assert!(evaluate(&valid).iter().all(|row| row.passed));
    let mut wrong = valid.clone();
    wrong.approval = Some(supplied.journal[0].id);
    let failures = evaluate(&wrong);
    assert!(!failures[0].passed && !failures[1].passed);
    wrong = valid.clone();
    wrong.successful_use_recorded = None;
    assert!(
        !evaluate(&wrong)[3].passed,
        "blank judgment earns no credit"
    );
    wrong.successful_use_recorded = Some(true);
    assert!(
        !evaluate(&wrong)[3].passed,
        "successful use by a different client is separate"
    );
    wrong = valid.clone();
    wrong.observed_extra_access = Some(supplied.journal[4].id);
    assert!(
        !evaluate(&wrong)[2].passed,
        "an approved client is a different observation"
    );
}

#[test]
fn repairing_policy_preserves_legitimate_path_and_reports_save_mismatches() {
    let (d, answers) = fixture(CHANGE);
    d.validate().unwrap();
    let task = &d.assignment;
    let mut work = Work::new(task);
    let Task::Change { supplied } = task else {
        panic!()
    };
    let expected = &answers["assignment"];
    let before = change_evidence::trials(
        &supplied.network,
        &supplied.parse(&supplied.starter).unwrap(),
        &supplied.trials,
    )
    .unwrap();
    assert_eq!(
        before
            .iter()
            .map(|trial| (trial.expected, trial.reached))
            .collect::<Vec<_>>(),
        [(true, true), (true, true), (false, true), (false, true)]
    );
    work.apply(
        task,
        Action::Report {
            report: serde_json::from_value(expected["report"].clone()).unwrap(),
        },
    )
    .unwrap();
    let alternate = expected["alternate"].as_str().unwrap();
    work.apply(
        task,
        Action::Draft {
            contents: alternate.into(),
        },
    )
    .unwrap();
    assert!(work.outcomes(task).iter().any(|row| !row.passed));
    work.apply(task, Action::Apply).unwrap();
    assert!(work.outcomes(task).iter().all(|row| row.passed));
    work.apply(task, Action::Check).unwrap();
    let Scope::Change { applied, .. } = &work.current else {
        panic!()
    };
    let active = applied.clone();
    work.apply(
        task,
        Action::Draft {
            contents: "unsupported directive".into(),
        },
    )
    .unwrap();
    assert!(!work.checked);
    assert!(work.apply(task, Action::Apply).is_err());
    let Scope::Change { applied, .. } = &work.current else {
        panic!()
    };
    assert_eq!(applied, &active);
}

#[test]
fn facility_rejects_blocking_every_visit_and_allows_equivalent_report_routes() {
    let (d, answers) = fixture(FACILITY);
    d.validate().unwrap();
    let task = &d.assignment;
    let mut work = Work::new(task);
    let mut plan = serde_json::from_value::<
        dlms_learning_lab::security_investigation::access_plan::Policy,
    >(answers["assignment"].clone())
    .unwrap();
    plan.suspicious_contact_reports = Reports::BothDesks;
    work.apply(task, Action::Facility { plan }).unwrap();
    assert!(work.outcomes(task).iter().all(|row| row.passed));
    work.apply(task, Action::Check).unwrap();
    plan.visitor = Access::Denied;
    work.apply(task, Action::Facility { plan }).unwrap();
    assert!(!work.checked);
    let rows = work.outcomes(task);
    assert_eq!(rows.iter().filter(|row| !row.passed).count(), 1);
    assert_eq!(
        rows.iter().find(|row| !row.passed).unwrap().label,
        "Escorted registered contractor"
    );
    plan.record_denials = false;
    work.apply(task, Action::Facility { plan }).unwrap();
    assert!(
        work.outcomes(task)
            .iter()
            .any(|row| row.detail.contains("requires an access log entry"))
    );
}

#[test]
fn practice_uses_its_own_network_facts_and_never_changes_the_assignment() {
    for id in [CHANGE, FACILITY] {
        let (d, answers) = fixture(id);
        let mut session =
            dlms_learning_lab::security_investigation::session::Session::new(&d).unwrap();
        let old = session.assignment.clone();
        let sample = &answers["example"];
        match d.example {
            Task::Change { .. } => {
                let report: Report = serde_json::from_value(sample["report"].clone()).unwrap();
                assert_eq!(report.successful_use_recorded, Some(true));
                session
                    .example
                    .work
                    .apply(&d.example, Action::Report { report })
                    .unwrap();
                session
                    .example
                    .work
                    .apply(
                        &d.example,
                        Action::Draft {
                            contents: sample["contents"].as_str().unwrap().into(),
                        },
                    )
                    .unwrap();
                session
                    .example
                    .work
                    .apply(&d.example, Action::Apply)
                    .unwrap();
            }
            Task::Facility { .. } => {
                session
                    .example
                    .work
                    .apply(
                        &d.example,
                        Action::Facility {
                            plan: serde_json::from_value(sample.clone()).unwrap(),
                        },
                    )
                    .unwrap();
            }
            _ => panic!("This historical fixture retains only the two investigation activities"),
        }
        session
            .example
            .work
            .apply(&d.example, Action::Check)
            .unwrap();
        assert!(
            session
                .example
                .work
                .outcomes(&d.example)
                .iter()
                .all(|row| row.passed),
            "{id}"
        );
        assert_eq!(session.assignment, old);
    }
}
