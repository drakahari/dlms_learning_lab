//! Prospective authored-unit gates for the private normal catalog overlay; no native acceptance.
use dlms_learning_lab::{
    physical_link as core, physical_link_assignment as work, physical_link_teaching as teaching,
};
use serde_json::Value;
const RAW: &str = include_str!("fixtures/physical-link-author-review.json");
const IDS: [&str; 4] = [
    "ccna-physical-media-span",
    "ccna-physical-duplex-diagnosis",
    "ccna-physical-counter-baseline",
    "ccna-physical-uplink-handover",
];
fn tasks() -> Vec<Value> {
    serde_json::from_str::<Value>(RAW).unwrap()["tasks"]
        .as_array()
        .unwrap()
        .clone()
}
fn config(v: &Value) -> core::Configuration {
    serde_json::from_value(v.clone()).unwrap()
}
fn apply(d: &work::Definition, c: &core::Configuration) -> core::State {
    let mut state = d.initial.clone();
    state
        .perform(&core::Action::Configure {
            configuration: c.clone(),
        })
        .unwrap();
    state
}
#[test]
fn four_distinct_meaningful_tasks_have_reachable_alternatives_and_independent_failures() {
    let rows = tasks();
    assert_eq!(rows.len(), 4);
    let mut seen = std::collections::BTreeSet::new();
    for (i, row) in rows.iter().enumerate() {
        assert_eq!(row["id"], IDS[i]);
        assert!(seen.insert(IDS[i]));
        for prerequisite in row["prerequisites"].as_array().unwrap() {
            assert!(IDS[..i].contains(&prerequisite.as_str().unwrap()));
        }
        let d =
            work::Definition::from_json(&serde_json::to_vec(&row["definition"]).unwrap()).unwrap();
        assert!(!d.solved(&d.initial));
        let refs = row["references"].as_array().unwrap();
        assert_eq!(refs.len(), 3);
        assert_eq!(
            refs.iter()
                .map(|v| serde_json::to_string(v).unwrap())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            3,
            "Reference plus two distinct supported alternatives"
        );
        for reference in refs {
            let c = config(reference);
            d.validate_reference(&c).unwrap();
            let state = apply(&d, &c);
            assert!(d.solved(&state));
            assert_eq!(state.topology(), d.initial.topology());
        }
        for name in ["wrong", "partial"] {
            let c = config(&row[name]["configuration"]);
            c.validate(d.initial.topology()).unwrap();
            assert!(!d.solved(&apply(&d, &c)), "{}/{name}", IDS[i]);
            assert!(!row[name]["reason"].as_str().unwrap().is_empty());
        }
        let mut state = apply(&d, &config(&refs[0]));
        assert!(d.solved(&state));
        state
            .perform(&core::Action::Configure {
                configuration: config(&row["break_after_success"]),
            })
            .unwrap();
        assert!(
            !d.solved(&state),
            "Current success must invalidate: {}",
            IDS[i]
        );
    }
    // Distinct work: media alternatives cross profile families; mismatch uses a
    // clean cable; receive investigation starts with agreed modes and impairment;
    // handover adds retired interfaces, not merely another cable value.
    assert_ne!(
        rows[0]["references"][0]["ports"]["campus-left"]["profile"],
        rows[0]["references"][1]["ports"]["campus-left"]["profile"]
    );
    assert_eq!(
        rows[1]["definition"]["initial"]["topology"]["cables"]["training-initial"]["condition"]["kind"],
        "Clean"
    );
    assert_eq!(
        rows[2]["definition"]["initial"]["topology"]["cables"]["archive-initial"]["condition"]["kind"],
        "ReceiveImpairment"
    );
    assert_eq!(
        rows[3]["definition"]["retired_ports"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}
#[test]
fn every_lesson_teaches_prerequisites_and_runs_an_isolated_example_with_adverse_retry() {
    for row in tasks() {
        let d =
            work::Definition::from_json(&serde_json::to_vec(&row["definition"]).unwrap()).unwrap();
        let lesson =
            teaching::Lesson::from_json(&serde_json::to_vec(&row["introduction"]).unwrap(), &d)
                .unwrap();
        assert_eq!(lesson.concepts.len(), 3);
        assert!(lesson.separate_from(&d));
        let assignment = d.initial.clone();
        let mut l = teaching::Learning::new(&lesson, &d).unwrap();
        let example_initial = l.current.clone();
        l.fill_worked_example(&lesson).unwrap();
        assert_eq!(l.current, example_initial);
        assert_eq!(d.initial, assignment);
        assert!(!l.checked);
        l.apply(&lesson).unwrap();
        assert!(l.checks(&lesson).iter().all(|r| r.0));
        let target = &lesson.practice.requirements[0];
        l.probe(&lesson, &target.link, 12, 18).unwrap();
        let mut draft = l.draft.clone();
        draft.configuration = lesson.try_change.clone();
        l.set_draft(&lesson, &draft).unwrap();
        l.apply(&lesson).unwrap();
        assert!(l.checks(&lesson).iter().any(|r| !r.0));
        draft.configuration = lesson.repair.clone();
        l.set_draft(&lesson, &draft).unwrap();
        l.apply(&lesson).unwrap();
        assert!(l.checks(&lesson).iter().all(|r| r.0));
        let bytes = serde_json::to_vec(&l).unwrap();
        let resumed: teaching::Learning = serde_json::from_slice(&bytes).unwrap();
        assert!(resumed.valid_for(&lesson));
        assert_eq!(resumed, l);
        l.reset(&lesson, &d).unwrap();
        assert_eq!(l.current, lesson.practice.initial);
        assert_eq!(d.initial, assignment);
        assert_eq!(row["hints"].as_array().unwrap().len(), 3);
        assert!(!row["worked_explanation"].as_str().unwrap().is_empty());
    }
}
#[test]
fn counter_lesson_uses_real_deltas_not_zero_totals_raw_crc_or_replayed_history() {
    let row = tasks().remove(2);
    let d = work::Definition::from_json(&serde_json::to_vec(&row["definition"]).unwrap()).unwrap();
    let mut state = d.initial.clone();
    let link = &state.topology().links["archive-link"];
    let (a, b) = (link.a.clone(), link.b.clone());
    assert_eq!(
        (
            state.counters()[&a].crc_errors,
            state.counters()[&b].crc_errors
        ),
        (32, 32)
    );
    let report = state
        .perform(&core::Action::Probe {
            link: "archive-link".into(),
            a_frames: 12,
            b_frames: 18,
        })
        .unwrap()
        .unwrap();
    // Independently reasoned literals: each third offered frame is impaired.
    // Access receives18, of which6 fail; distribution receives12, of which4 fail.
    assert_eq!(
        (
            report.after[&a].crc_errors - report.before[&a].crc_errors,
            report.after[&b].crc_errors - report.before[&b].crc_errors
        ),
        (6, 4)
    );
    assert_eq!(
        (
            report.after[&a].received - report.before[&a].received,
            report.after[&b].received - report.before[&b].received
        ),
        (12, 8)
    );
    for port in [&a, &b] {
        state
            .perform(&core::Action::ClearCounters { port: port.clone() })
            .unwrap();
    }
    assert_eq!(
        (
            state.counters()[&a].crc_errors,
            state.counters()[&b].crc_errors
        ),
        (0, 0)
    );
    assert!(!d.solved(&state), "Clearing is not repair");
    let fixed = apply(&d, &config(&row["references"][0]));
    assert_eq!(fixed.counters()[&a].crc_errors, 32);
    assert!(
        d.solved(&fixed),
        "Historical errors may remain after genuine repair"
    );
    let mut changed_source = d.clone();
    changed_source.observations[0].reading.counters.crc_errors = 0;
    assert!(!changed_source.solved(&changed_source.initial));
}
#[test]
fn handover_requires_retirement_and_preserved_neighbor_while_media_rejects_short_spare() {
    let rows = tasks();
    let d =
        work::Definition::from_json(&serde_json::to_vec(&rows[3]["definition"]).unwrap()).unwrap();
    let partial = apply(&d, &config(&rows[3]["partial"]["configuration"]));
    assert!(
        d.checks(&partial)
            .iter()
            .any(|(ok, text)| !ok && text.starts_with("Retire the old interface"))
    );
    let wrong = apply(&d, &config(&rows[3]["wrong"]["configuration"]));
    assert!(
        d.checks(&wrong)
            .iter()
            .any(|(ok, text)| !ok && text.starts_with("Preserve the configuration"))
    );
    for ref_value in rows[3]["references"].as_array().unwrap() {
        let state = apply(&d, &config(ref_value));
        for p in &d.protected_ports {
            assert_eq!(
                state.configuration().ports[p],
                d.initial.configuration().ports[p]
            );
        }
    }
    let media =
        work::Definition::from_json(&serde_json::to_vec(&rows[0]["definition"]).unwrap()).unwrap();
    let short = apply(&media, &config(&rows[0]["wrong"]["configuration"]));
    assert_eq!(
        short.report("campus-link").unwrap().qualification,
        core::Qualification::TooShortForRoute
    );
}
