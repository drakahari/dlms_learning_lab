#![allow(dead_code)] // Native QA and integration binaries use different helper subsets.
#[path = "../../tests/support/loop_registration.rs"]
pub mod loop_registration;
// Developer/native-QA and test source-boundary helper. No progress or grading.
// Keep old predecessor-count assertions while their loops still inspect all180.
use dlms_learning_lab::{
    course::Unit,
    engine::{Goal, State},
    pack::{Pack, Scenario},
};
use serde_json::Value;
use std::sync::OnceLock;
struct Expected {
    ids: Vec<(String, u32)>,
    units: Vec<Unit>,
    units_value: Vec<Value>,
    unit: Unit,
    unit_value: Value,
    new: Vec<Scenario>,
}
fn expected() -> &'static Expected {
    static EXPECTED: OnceLock<Expected> = OnceLock::new();
    EXPECTED.get_or_init(|| {
        let r: Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/catalog180-registration.json"
        ))
        .unwrap();
        let raw48 = include_bytes!("../../scenarios/units-v1.json");
        let raw47 = loop_registration::before_units(raw48).unwrap();
        let raw = std::str::from_utf8(&raw47).unwrap();
        assert_eq!(raw.len() as u64, r["after_units47_bytes"].as_u64().unwrap());
        let prefix = raw
            .strip_suffix(r["append_suffix_utf8"].as_str().unwrap())
            .expect("Exact47append");
        let original = format!("{prefix}{}", r["old_tail_utf8"].as_str().unwrap());
        assert_eq!(
            original.as_bytes(),
            include_bytes!("../../tests/fixtures/one-off-units-before176.json")
        );
        assert_eq!(
            original.len() as u64,
            r["before_units46_bytes"].as_u64().unwrap()
        );
        let old: Value = serde_json::from_str(&original).unwrap();
        assert_eq!(old, r["before_units46"]);
        let mut after = old.clone();
        after["units"]
            .as_array_mut()
            .unwrap()
            .push(r["one_off_unit"].clone());
        assert_eq!(after, serde_json::from_str::<Value>(raw).unwrap());
        let ids = r["before_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                (
                    v["id"].as_str().unwrap().to_owned(),
                    v["version"].as_u64().unwrap() as u32,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(ids.len(), 176);
        let new: Pack =
            serde_json::from_str(include_str!("../../scenarios/linux-one-off-v1.json")).unwrap();
        assert_eq!(new.scenarios.len(), 4);
        Expected {
            ids,
            units: serde_json::from_value(old["units"].clone()).unwrap(),
            units_value: old["units"].as_array().unwrap().clone(),
            unit: serde_json::from_value(r["one_off_unit"].clone()).unwrap(),
            unit_value: r["one_off_unit"].clone(),
            new: new.scenarios,
        }
    })
}
fn same_unit(a: &Unit, b: &Unit) {
    assert_eq!(
        (
            &a.id,
            &a.track,
            &a.title,
            &a.summary,
            &a.prerequisites,
            &a.labs
        ),
        (
            &b.id,
            &b.track,
            &b.title,
            &b.summary,
            &b.prerequisites,
            &b.labs
        )
    );
}
pub trait Boundary {
    fn check(&self) -> usize;
}
pub fn before_len<T: Boundary + ?Sized>(items: &T) -> usize {
    items.check()
}
impl Boundary for [Scenario] {
    fn check(&self) -> usize {
        let items = loop_registration::before_scenarios(self);
        let e = expected();
        assert_eq!(items.len(), 180);
        for (s, (id, version)) in items[..176].iter().zip(&e.ids) {
            assert_eq!((&s.id, s.version), (id, *version));
        }
        for (a, b) in items[176..].iter().zip(&e.new) {
            assert!(dlms_learning_lab::one_off_workspace::source_shape(a));
            macro_rules! same { ($($field:ident),+ $(,)?) => {$(assert_eq!(&a.$field, &b.$field);)+}; }
            same!(
                id,
                version,
                track,
                topic,
                unit,
                title,
                summary,
                objective,
                retry_guidance,
                difficulty,
                minutes,
                steps,
                explanation,
                hints,
                sources,
                simplifications,
                solution,
                prerequisites,
                one_off_teaching
            );
            assert!(a.guide.is_empty() && b.guide.is_empty());
            match (&a.initial, &b.initial) {
                (State::OneOff { simulation: x }, State::OneOff { simulation: y }) => {
                    assert_eq!(x, y)
                }
                _ => panic!("Canonical newOneOff kind"),
            }
            match (&a.goal, &b.goal) {
                (Goal::OneOff { definition: x }, Goal::OneOff { definition: y }) => {
                    assert_eq!(x, y)
                }
                _ => panic!("Canonical newOneOff goal"),
            }
        }
        176
    }
}
impl Boundary for [Unit] {
    fn check(&self) -> usize {
        let items = loop_registration::before_typed_units(self);
        let e = expected();
        assert_eq!(items.len(), 47);
        for (a, b) in items[..46].iter().zip(&e.units) {
            same_unit(a, b);
        }
        same_unit(&items[46], &e.unit);
        46
    }
}
impl Boundary for [Value] {
    fn check(&self) -> usize {
        let items = loop_registration::before_values(self);
        let e = expected();
        assert_eq!(items.len(), 47);
        assert_eq!(&items[..46], &e.units_value);
        assert_eq!(&items[46], &e.unit_value);
        46
    }
}

impl<T> Boundary for Vec<T>
where
    [T]: Boundary,
{
    fn check(&self) -> usize {
        self.as_slice().check()
    }
}
impl Boundary for std::collections::BTreeMap<String, dlms_learning_lab::progress::Attempt> {
    fn check(&self) -> usize {
        let normal182 = loop_registration::lvm_registration::before_attempts_len(self);
        assert_eq!(normal182, 182);
        for id in loop_registration::IDS {
            let a = &self[id];
            assert_eq!(a.scenario_version, 1);
            assert!(matches!(a.state, State::Script { .. }));
        }
        assert_eq!(normal182 - loop_registration::IDS.len(), 180);
        for id in [
            "lpic1-at-report",
            "lpic1-at-capture-repair",
            "lpic1-at-owner-queue",
            "lpic1-at-handover",
        ] {
            let a = &self[id];
            assert_eq!(a.scenario_version, 1);
            assert!(matches!(&a.state, State::OneOff { .. }));
        }
        176
    }
}

impl<T: Boundary + ?Sized> Boundary for &T {
    fn check(&self) -> usize {
        (**self).check()
    }
}
impl<T: Boundary + ?Sized> Boundary for &mut T {
    fn check(&self) -> usize {
        (**self).check()
    }
}
