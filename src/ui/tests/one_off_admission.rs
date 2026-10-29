//! Synthetic source-admission fixture only; no bundled lab or installed learner state.
use dlms_learning_lab::{cron::Moment, one_off as core, one_off_workspace as work};
use std::collections::{BTreeMap, BTreeSet};
fn time(s: &str) -> Moment {
    Moment::parse(s).unwrap()
}
fn initial() -> core::State {
    core::State {
        version: 1,
        clock: time("2026-01-06 10:00"),
        next_id: 1,
        users: BTreeSet::from(["root".into(), "alex".into()]),
        files: BTreeMap::from([
            (
                "/work/job.sh".into(),
                "cat /work/input.txt > /work/output.txt\n".into(),
            ),
            ("/work/input.txt".into(), "live first\n".into()),
            ("/work/keep.txt".into(), "preserve\n".into()),
        ]),
        jobs: BTreeMap::new(),
        records: vec![],
    }
}
fn selector() -> work::Selector {
    work::Selector {
        owner: "alex".into(),
        script_path: "/work/job.sh".into(),
        timing: work::Timing::AfterSubmission { minutes: 5 },
    }
}
fn definition() -> work::Definition {
    let initial = initial();
    let mut probes = Vec::new();
    for (name, input) in [
        ("Original live input", "live first\n"),
        ("Different live input", "second sample\n"),
    ] {
        let mut files = initial.files.clone();
        files.insert("/work/input.txt".into(), input.into());
        let mut after = files.clone();
        after.insert("/work/output.txt".into(), input.into());
        probes.push(work::Probe {
            name: name.into(),
            source: work::CaptureSource::LastRun,
            job: selector(),
            window: work::ProbeWindow::AfterSubmission {
                start_minutes: 0,
                end_minutes: 6,
            },
            files,
            expected: Some(work::ExpectedEvent {
                owner: "alex".into(),
                when: work::ExpectedWhen::Dispatch,
                status: core::Status::Completed,
                exit_status: Some(0),
                stdout: String::new(),
                stderr: String::new(),
                error: None,
            }),
            after,
        });
    }
    probes.push(work::Probe {
        name: "Before the due minute".into(),
        source: work::CaptureSource::LastRun,
        job: selector(),
        window: work::ProbeWindow::AfterSubmission {
            start_minutes: 0,
            end_minutes: 4,
        },
        files: initial.files.clone(),
        expected: None,
        after: initial.files.clone(),
    });
    work::Definition {
        initial,
        editable: vec!["/work/job.sh".into(), "/work/input.txt".into()],
        outputs: vec!["/work/output.txt".into()],
        preserve_jobs: vec![],
        requirements: vec![
            work::Requirement::QueueExactly {
                source: work::CaptureSource::Queued,
                jobs: vec![],
            },
            work::Requirement::QueueExactly {
                source: work::CaptureSource::LastRun,
                jobs: vec![selector()],
            },
            work::Requirement::CurrentFile {
                path: "/work/output.txt".into(),
                text: "live first\n".into(),
            },
            work::Requirement::Dispatched { job: selector() },
        ],
        probes,
    }
}
fn scenario() -> dlms_learning_lab::pack::Scenario {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("../../../scenarios/starter-v1.json")).unwrap();
    let mut value = pack["scenarios"][0].clone();
    let object = value.as_object_mut().unwrap();
    object.retain(|k, _| k != "teaching" && !k.ends_with("_teaching"));
    object.insert("id".into(), "synthetic-one-off-binding".into());
    object.insert("unit".into(), "".into());
    object.insert("track".into(), "LPIC-1".into());
    object.insert("topic".into(), "Scheduling".into());
    object.insert("title".into(), "Synthetic one-off binding fixture".into());
    object.insert(
        "steps".into(),
        serde_json::json!(["Use the virtual queue and explicit clock"]),
    );
    object.insert("guide".into(), serde_json::json!([]));
    object.insert("prerequisites".into(), serde_json::json!([]));
    let d = definition();
    object.insert(
        "initial".into(),
        serde_json::to_value(dlms_learning_lab::engine::State::OneOff {
            simulation: Box::new(work::Assignment::new(&d.initial).unwrap()),
        })
        .unwrap(),
    );
    object.insert(
        "goal".into(),
        serde_json::to_value(dlms_learning_lab::engine::Goal::OneOff {
            definition: Box::new(d),
        })
        .unwrap(),
    );
    object.insert("solution".into(),serde_json::json!([
        format!("one-off {}",serde_json::json!({"kind":"Command","actor":"alex","environment":{},"text":"at -f /work/job.sh now + 5 minutes"})),
        format!("one-off {}",serde_json::json!({"kind":"Run","end":time("2026-01-06 10:06")}))
    ]));
    serde_json::from_value(value).unwrap()
}
fn lesson() -> dlms_learning_lab::one_off_teaching::Lesson {
    use dlms_learning_lab::one_off_teaching::{AnnotatedAction, Concept, Lesson};
    let mut d: work::Definition = serde_json::from_str(
        &serde_json::to_string(&definition())
            .unwrap()
            .replace("/work/", "/work/example-")
            .replace("\"alex\"", "\"sam\"")
            .replace("live first", "example one")
            .replace("second sample", "example two"),
    )
    .unwrap();
    let mut wanted = selector();
    wanted.owner = "sam".into();
    wanted.script_path = "/work/example-job.sh".into();
    for p in &mut d.probes {
        p.source = work::CaptureSource::Queued;
    }
    d.requirements = vec![work::Requirement::QueueExactly {
        source: work::CaptureSource::Queued,
        jobs: vec![wanted],
    }];
    let command = |text: &str| work::Action::Command { text: text.into() };
    Lesson{prerequisite_note:"You can inspect a supplied virtual text file; the example script is already supplied.".into(),concepts:vec![Concept{title:"A queued copy".into(),text:"Submitting captures this script and its variable values. Editing the original file later does not rewrite the queued copy.".into()},Concept{title:"An explicit clock".into(),text:"The clock changes only when you choose a simulated window. Due times here use UTC, independently of the Fedora desktop clock.".into()}],example_description:"Queue Sam's separate example job for five minutes from now. Check uses other sample inputs without changing this queue.".into(),definition:d,example_actions:vec![AnnotatedAction{title:"Choose Sam".into(),explanation:"The selected simulated account owns its new queued work.".into(),action:work::Action::Actor{actor:"sam".into()}},AnnotatedAction{title:"Submit the supplied script".into(),explanation:"The five-minute delay belongs to this example, independently of the assignment.".into(),action:command("at -f /work/example-job.sh now + 5 minutes")}],try_change:"Remove the example job and queue it for six minutes; inspect the due-time feedback.".into(),try_actions:vec![command("atrm 1"),command("at -f /work/example-job.sh now + 6 minutes")],try_rows:vec![true,true,false,true,false,false,false],repair:vec![command("atrm 2"),command("at -f /work/example-job.sh now + 5 minutes")]}
}

pub(in crate::ui) fn prepared() -> (
    dlms_learning_lab::pack::Scenario,
    dlms_learning_lab::progress::Attempt,
) {
    let mut source = scenario();
    source.one_off_teaching = Some(lesson());
    let mut a = dlms_learning_lab::progress::Attempt::new(&source, 5);
    work::budget::prepare(&mut a, &source).unwrap();
    let w = a.workspace.as_ref().unwrap();
    assert!(w.valid());
    assert!(w.one_off_session.is_some() && w.one_off_learning.is_some());
    (source, a)
}
