//! Technical foreign-family guards only; no physical native-workspace acceptance.
use super::*;
#[path = "../../../tests/support/physical_scenario.rs"]
mod fixture;
pub(in crate::ui) fn source() -> Scenario {
    fixture::scenario()
}
pub(in crate::ui) fn prepared() -> (Scenario, Attempt) {
    let s = source();
    let mut a = Attempt::new(&s, 9);
    dlms_learning_lab::physical_link_workspace::prepare_attempt(&mut a, &s, true).unwrap();
    (s, a)
}
#[test]
fn discovery_prepare_retains_real_physical_kind_and_valid_metadata_on_old_kind() {
    let (physical, prepared) = prepared();
    let all = pack::bundled().unwrap();
    let source = all.iter().find(|s| s.discovery_teaching.is_some()).unwrap();
    let lesson = source.discovery_teaching.as_ref().unwrap();
    for with in [false, true] {
        for new in [false, true] {
            let mut a = if with {
                prepared.clone()
            } else {
                Attempt::new(&physical, 9)
            };
            let old = serde_json::to_value(&a).unwrap();
            discovery_workspace::prepare(&mut a, lesson, new);
            assert_eq!(serde_json::to_value(&a).unwrap(), old);
        }
    }
    let mut own = Attempt::new(source, 3);
    model::prepare(&mut own);
    discovery_workspace::prepare(&mut own, lesson, true);
    assert!(own.workspace.as_ref().unwrap().discovery_learning.is_some());
    let pw = prepared.workspace.as_ref().unwrap();
    for mask in 1..4 {
        let mut incoming = own.clone();
        let w = incoming.workspace.as_mut().unwrap();
        if mask & 1 != 0 {
            w.physical_link_draft = pw.physical_link_draft.clone();
        }
        if mask & 2 != 0 {
            w.physical_link_learning = pw.physical_link_learning.clone();
        }
        assert!(w.valid());
        let before = serde_json::to_value(&incoming).unwrap();
        for new in [false, true] {
            discovery_workspace::prepare(&mut incoming, lesson, new);
            assert_eq!(serde_json::to_value(&incoming).unwrap(), before);
        }
    }
}
