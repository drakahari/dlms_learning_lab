//! Private deterministic admission fixtures, never bundled curriculum.
#![allow(dead_code)] // Different integration/native binaries exercise subsets.
use dlms_learning_lab::{
    pack::{self, LessonStep, Scenario},
    progress::Attempt,
    syslog_workspace as work,
};
pub fn padded_source(mut s: Scenario) -> Scenario {
    // Every field remains under existing authoring limits; the pack remains <2 MB.
    s.steps = vec!["Capacity fixture task text".to_owned() + &"x".repeat(3974); 50];
    s.hints = vec!["A private admission hint".into(); 50];
    s.sources = vec!["https://example.invalid/".to_owned() + &"s".repeat(3976); 50];
    s.guide = (0..50)
        .map(|_| LessonStep {
            title: "t".repeat(3500),
            why: "w".repeat(3500),
            action: "a".repeat(3500),
            commands: vec![pack::CommandExample {
                command: "Private capacity reference note".into(),
                explanation: "e".repeat(4000),
                simulator_only: true,
            }],
        })
        .collect();
    let raw = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios: vec![s],
    })
    .unwrap();
    assert!(raw.len() < pack::MAX_PACK_BYTES);
    pack::load(&raw).unwrap().remove(0)
}
pub fn size(s: &Scenario, a: &Attempt) -> usize {
    serde_json::to_vec(&(s, a)).unwrap().len()
}
pub fn fill_to(s: &Scenario, a: &mut Attempt, target: usize) {
    assert!(size(s, a) < target);
    let old = a.transcript.len();
    a.transcript.resize(120, String::new());
    let used = size(s, a);
    assert!(used <= target);
    let mut remaining = target - used;
    for row in &mut a.transcript[old..] {
        let n = remaining.min(8192);
        row.push_str(&"x".repeat(n));
        remaining -= n;
    }
    assert_eq!(
        remaining, 0,
        "Fixed fixture must fit existing120×8192 transcript bound"
    );
    assert_eq!(size(s, a), target);
    work::context(a, s).unwrap();
}
