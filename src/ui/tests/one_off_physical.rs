//! Reciprocal integration guards; no fixture/state installation in a native run.
use super::*;
use dlms_learning_lab::{one_off_workspace as one, physical_link_workspace as physical};
fn one_off() -> (Scenario, Attempt) {
    let (s, _) = super::one_off_admission::prepared();
    let mut a = Attempt::new(&s, 17);
    for text in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, text).unwrap();
    }
    one::budget::prepare(&mut a, &s).unwrap();
    assert!(progress::current_solved(&a, &s));
    (s, a)
}
fn physical() -> (Scenario, Attempt) {
    let s = pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| matches!(s.initial, State::PhysicalLink { .. }))
        .unwrap();
    let mut a = Attempt::new(&s, 23);
    for text in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, text).unwrap();
    }
    physical::prepare_attempt(&mut a, &s, true).unwrap();
    assert!(progress::current_solved(&a, &s));
    (s, a)
}
fn add_physical(mask: usize, a: &mut Attempt, s: &mut Scenario, p: &Attempt, source: &Scenario) {
    if mask == 0 {
        s.physical_link_teaching = source.physical_link_teaching.clone();
    } else {
        let w = a.workspace.as_mut().unwrap();
        let f = p.workspace.as_ref().unwrap();
        if mask & 1 != 0 {
            w.physical_link_draft = f.physical_link_draft.clone();
        }
        if mask & 2 != 0 {
            w.physical_link_learning = f.physical_link_learning.clone();
        }
        assert!(w.valid());
    }
}
fn add_one_off(mask: usize, a: &mut Attempt, s: &mut Scenario, p: &Attempt, source: &Scenario) {
    if mask == 0 {
        s.one_off_teaching = source.one_off_teaching.clone();
    } else {
        let w = a.workspace.as_mut().unwrap();
        let f = p.workspace.as_ref().unwrap();
        if mask & 1 != 0 {
            w.one_off_session = f.one_off_session.clone();
        }
        if mask & 2 != 0 {
            w.one_off_learning = f.one_off_learning.clone();
        }
        assert!(w.valid());
    }
}
#[test]
fn one_off_typed_refuses_valid_physical_source_or_each_optional_field_before_publication() {
    let (source, good) = one_off();
    let (foreign, p) = physical();
    for mask in 0..4 {
        let mut s = source.clone();
        let mut a = good.clone();
        add_physical(mask, &mut a, &mut s, &p, &foreign);
        let before = serde_json::to_value(&a).unwrap();
        let authored = serde_json::to_value(&s).unwrap();
        assert!(one::context(&a, &s).is_err());
        assert!(!progress::current_solved(&a, &s));
        assert!(one::budget::prepare(&mut a, &s).is_err());
        assert!(
            one::budget::apply_attempt(
                &mut a,
                &s,
                &one::Action::Command { text: "atq".into() },
                false
            )
            .is_err()
        );
        assert!(
            one::budget::PendingAttempt::begin(
                &a,
                &s,
                dlms_learning_lab::cron::Moment::parse("2026-01-06 10:07").unwrap(),
                false
            )
            .is_err()
        );
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
        assert_eq!(serde_json::to_value(&s).unwrap(), authored);
    }
    assert!(progress::current_solved(&good, &source));
}
#[test]
fn physical_typed_refuses_valid_one_off_source_or_each_optional_field_before_publication() {
    let (source, good) = physical();
    let (foreign, p) = one_off();
    for mask in 0..4 {
        let mut s = source.clone();
        let mut a = good.clone();
        add_one_off(mask, &mut a, &mut s, &p, &foreign);
        let before = serde_json::to_value(&a).unwrap();
        let authored = serde_json::to_value(&s).unwrap();
        assert!(physical::context(&a, &s).is_err());
        assert!(!progress::current_solved(&a, &s));
        assert!(physical::prepare_attempt(&mut a, &s, false).is_err());
        assert!(physical::apply(&mut a, &s, physical::Scope::Assignment).is_err());
        assert!(physical::reset(&mut a, &s, physical::Scope::Practice).is_err());
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
        assert_eq!(serde_json::to_value(&s).unwrap(), authored);
    }
    assert!(progress::current_solved(&good, &source));
}
fn finish(m: &mut crate::ui::one_off_workspace::memo::Memo, a: &Attempt, s: &Scenario) {
    m.evaluate(a, s, false);
    for _ in 0..10000 {
        if !m.advance(32) {
            break;
        }
    }
    m.publish_current(a, s).unwrap();
    assert_eq!(m.passed(a, s), Some(true));
}
#[test]
fn one_off_warm_success_cannot_survive_physical_four_masks_or_copy_grade_meter() {
    let (source, good) = one_off();
    let (foreign, p) = physical();
    for mask in 0..4 {
        let mut m = crate::ui::one_off_workspace::memo::Memo::default();
        finish(&mut m, &good, &source);
        let counters = m.counters;
        let mut s = source.clone();
        let mut a = good.clone();
        add_physical(mask, &mut a, &mut s, &p, &foreign);
        let before = serde_json::to_value(&a).unwrap();
        let authored = serde_json::to_value(&s).unwrap();
        assert_eq!(m.passed(&a, &s), None);
        m.evaluate(&a, &s, false);
        assert_eq!(m.passed(&a, &s), None);
        assert!(!m.advance(32));
        assert_eq!(m.counters.snapshots, counters.snapshots);
        assert_eq!(m.counters.meters, counters.meters);
        assert_eq!(m.counters.steps, counters.steps);
        assert_eq!(m.counters.hits, counters.hits);
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
        assert_eq!(serde_json::to_value(&s).unwrap(), authored);
        finish(&mut m, &good, &source);
    }
}
#[test]
fn physical_warm_success_cannot_survive_one_off_four_masks_or_copy_grade_meter() {
    let (source, good) = physical();
    let (foreign, p) = one_off();
    for mask in 0..4 {
        let mut m = crate::ui::physical_link_workspace::memo::Memo::default();
        m.prepare(&good, &source, false).unwrap();
        assert_eq!(m.passed(&good, &source), Some(true));
        let counters = m.counters;
        let mut s = source.clone();
        let mut a = good.clone();
        add_one_off(mask, &mut a, &mut s, &p, &foreign);
        let before = serde_json::to_value(&a).unwrap();
        let authored = serde_json::to_value(&s).unwrap();
        assert_eq!(m.passed(&a, &s), None);
        assert!(m.prepare(&a, &s, false).is_err());
        assert_eq!(m.passed(&a, &s), None);
        assert_eq!(m.counters, counters);
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
        assert_eq!(serde_json::to_value(&s).unwrap(), authored);
        m.prepare(&good, &source, false).unwrap();
        assert_eq!(m.passed(&good, &source), Some(true));
    }
}
#[test]
fn complete_one_off_wire_projection_accounts_for_each_new_physical_field_before_refusing() {
    let (source, good) = one_off();
    let (foreign, p) = physical();
    for mask in 0..4 {
        let mut s = source.clone();
        let mut a = good.clone();
        add_physical(mask, &mut a, &mut s, &p, &foreign);
        let State::OneOff { simulation } = &a.state else {
            panic!()
        };
        let w = a.workspace.as_ref().unwrap();
        let session = one::budget::projected(
            &a,
            simulation,
            w.one_off_session
                .as_ref()
                .map(|v| serde_json::to_vec(v).unwrap().len()),
            w.one_off_learning
                .as_ref()
                .map(|v| serde_json::to_vec(v).unwrap().len()),
        );
        assert_eq!(session, Some(serde_json::to_vec(&a).unwrap().len()));
        assert!(one::context(&a, &s).is_err());
    }
}
