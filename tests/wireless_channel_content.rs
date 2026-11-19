//! Real authored channel plans; native/inventory admission remains a separate gate.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack, progress, wireless_channel_workspace as work,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = [
    "ccna-wireless-neighbour-channels",
    "ccna-wireless-multisurvey-plan",
];
fn authored() -> &'static Vec<pack::Scenario> {
    static DATA: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    DATA.get_or_init(|| {
        let raw: pack::Pack =
            serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v24.json")).unwrap();
        assert_eq!(
            raw.scenarios[46..]
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            IDS
        );
        let mut all = pack::accepted419().unwrap();
        assert_eq!(all.len(), 419);
        all.extend(
            raw.scenarios
                .into_iter()
                .filter(|s| IDS.contains(&s.id.as_str())),
        );
        let texts = all
            .chunks(8)
            .map(|rows| {
                serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":rows}))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>())
            .unwrap()
            .into_iter()
            .filter(|s| IDS.contains(&s.id.as_str()))
            .collect()
    })
}
fn session(a: &mut progress::Attempt) -> &mut work::Session {
    let State::WirelessChannels { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn def(s: &pack::Scenario) -> &work::Definition {
    let Goal::WirelessChannels { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn solve(a: &mut progress::Attempt, s: &pack::Scenario) {
    for action in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, action).unwrap();
    }
}
#[test]
fn both_actual_methods_and_distinct_examples_reach_derived_outcomes_without_example_credit() {
    for s in authored() {
        let d = def(s);
        d.validate().unwrap();
        assert_ne!(d.initial.network, d.example.network);
        let mut a = progress::Attempt::new(s, 1);
        session(&mut a).practice.fill_reference(&d.example).unwrap();
        assert!(!session(&mut a).practice.checked());
        assert!(!progress::current_solved(&a, s));
        assert!(
            session(&mut a)
                .practice
                .check(&d.example)
                .unwrap()
                .iter()
                .all(|r| r.correct)
        );
        assert!(!progress::current_solved(&a, s));
        let example = session(&mut a).practice.clone();
        solve(&mut a, s);
        assert!(progress::current_solved(&a, s));
        assert_eq!(session(&mut a).practice, example);
    }
}
#[test]
fn six_equivalent_multi_survey_allocations_pass_and_one_survey_only_plan_fails_specifically() {
    let s = &authored()[1];
    let d = def(s);
    let mut a = progress::Attempt::new(s, 1);
    let mut accepted = 0;
    for north in [1, 6, 11] {
        for east in [1, 6, 11] {
            for south in [1, 6, 11] {
                for west in [1, 6, 11] {
                    let mut draft = work::Draft::new(&d.initial);
                    for (id, value) in [
                        ("north", north),
                        ("east", east),
                        ("south", south),
                        ("west", west),
                    ] {
                        draft.set(&d.initial, id, value.to_string()).unwrap();
                    }
                    draft.check(&d.initial).unwrap();
                    if draft.solved(&d.initial) {
                        accepted += 1;
                        assert!(
                            north != east
                                && east != south
                                && south != west
                                && west != north
                                && north != south
                        );
                    }
                }
            }
        }
    }
    assert_eq!(accepted, 6);
    for (id, value) in [("north", "1"), ("east", "6"), ("south", "1"), ("west", "6")] {
        session(&mut a)
            .assignment
            .set(&d.initial, id, value.into())
            .unwrap();
    }
    let rows = session(&mut a).assignment.check(&d.initial).unwrap();
    assert_eq!(rows.iter().filter(|r| !r.correct).count(), 1);
    let failure = rows.iter().find(|r| !r.correct).unwrap();
    assert!(failure.label.contains("Partition open"));
    assert!(failure.feedback.contains("share airtime"));
    assert!(!progress::current_solved(&a, s));
}
#[test]
fn plausible_wrong_single_radio_plans_and_bad_formats_explain_the_specific_mistake() {
    let s = &authored()[0];
    let d = def(s);
    let mut a = progress::Attempt::new(s, 1);
    for (value, needle) in [
        ("", "Enter a channel"),
        ("2412MHz", "whole channel"),
        ("12", "1 to 11"),
        ("1", "share airtime"),
        ("2", "Different channel numbers"),
        ("6", "share airtime"),
    ] {
        session(&mut a)
            .assignment
            .set(&d.initial, "reception", value.into())
            .unwrap();
        let rows = session(&mut a).assignment.check(&d.initial).unwrap();
        assert!(
            rows.iter()
                .any(|r| !r.correct && r.feedback.contains(needle)),
            "{value}: {rows:?}"
        );
        assert!(!progress::current_solved(&a, s));
    }
    assert!(
        session(&mut a)
            .assignment
            .set(&d.initial, "neighbour-west", "11".into())
            .is_err()
    );
    solve(&mut a, s);
    assert!(progress::current_solved(&a, s));
}
#[test]
fn edits_reset_and_roundtrip_preserve_separate_drafts_without_stale_completion() {
    for s in authored() {
        let d = def(s);
        let mut a = progress::Attempt::new(s, 1);
        solve(&mut a, s);
        assert!(progress::current_solved(&a, s));
        let id = d.initial.reference[0].radio.clone();
        session(&mut a)
            .assignment
            .set(&d.initial, &id, "2".into())
            .unwrap();
        assert!(!progress::current_solved(&a, s));
        session(&mut a).practice.fill_reference(&d.example).unwrap();
        let bytes = serde_json::to_vec(&a).unwrap();
        let mut restored: progress::Attempt = serde_json::from_slice(&bytes).unwrap();
        work::context(&restored, s).unwrap();
        assert_eq!(
            serde_json::to_value(&restored).unwrap(),
            serde_json::to_value(&a).unwrap()
        );
        assert!(!progress::current_solved(&restored, s));
        let example = session(&mut restored).practice.clone();
        session(&mut restored).reset_assignment(d);
        assert_eq!(session(&mut restored).practice, example);
        assert!(!progress::current_solved(&restored, s));
        solve(&mut restored, s);
        assert!(progress::current_solved(&restored, s));
    }
}
#[test]
fn authored_tail_retains_all_46_previous_values_and_bad_or_unreachable_definitions_refuse() {
    let old: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v23.json")).unwrap();
    let new: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v24.json")).unwrap();
    assert_eq!((old.scenarios.len(), new.scenarios.len()), (46, 48));
    assert_eq!(
        serde_json::to_value(&old.scenarios).unwrap(),
        serde_json::to_value(&new.scenarios[..46]).unwrap()
    );
    for s in authored() {
        let mut bad = s.clone();
        if let Goal::WirelessChannels { definition } = &mut bad.goal {
            definition.initial.reference[0].value = "2".into();
        }
        assert!(work::source(&bad).is_err());
        let mut bad = s.clone();
        if let State::WirelessChannels { simulation } = &mut bad.initial {
            simulation.assignment.last_check = Some(simulation.assignment.fields.clone());
        }
        assert!(work::source(&bad).is_err());
        let mut bad = s.clone();
        if let Goal::WirelessChannels { definition } = &mut bad.goal {
            definition.initial.network.surveys[0].neighbours[0].b = "unknown".into();
        }
        assert!(work::source(&bad).is_err());
    }
}

#[test]
fn normal_424_catalog_keeps_all_419_scenarios_and_161_units_exact() {
    use dlms_learning_lab::course;
    let old = pack::accepted419().unwrap();
    let current = pack::current().unwrap();
    assert_eq!((old.len(), current.len()), (419, 424));
    for source in &old {
        let now = current.iter().find(|s| s.id == source.id).unwrap();
        assert_eq!(
            serde_json::to_value(now).unwrap(),
            serde_json::to_value(source).unwrap()
        );
    }
    let old_units = course::accepted419(&old).unwrap();
    let units = course::current(&current).unwrap();
    assert_eq!((old_units.len(), units.len()), (161, 163));
    for before in old_units {
        let now = units.iter().find(|u| u.id == before.id).unwrap();
        assert_eq!(
            serde_json::to_value(now).unwrap(),
            serde_json::to_value(before).unwrap()
        );
    }
    let unit = units
        .iter()
        .position(|u| u.id == "ccna-wireless-channel-foundations")
        .unwrap();
    let anchor = units
        .iter()
        .position(|u| u.id == "ccna-wired-device-paths")
        .unwrap();
    assert_eq!(unit, anchor + 1);
    assert_eq!(
        units[unit]
            .labs
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        IDS
    );
}
