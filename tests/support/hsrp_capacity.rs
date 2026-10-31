//! Fixed private admission fixtures, never curriculum.
#![allow(dead_code)] // Native and integration binaries use different fixture helpers.
use dlms_learning_lab::{
    hsrp_workspace as work,
    pack::{self, LessonStep, Scenario},
    progress::Attempt,
};
pub fn source(mut s: Scenario) -> Scenario {
    // This detached size/admission fixture is loaded alone, without curriculum
    // navigation. Authored prerequisite validation remains tested separately.
    s.prerequisites.clear();
    s.steps = vec!["x".repeat(2000); 50];
    s.hints = vec!["Private admission hint".into(); 50];
    let text = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios: vec![s],
    })
    .unwrap();
    assert!(text.len() < pack::MAX_PACK_BYTES);
    pack::load(&text).unwrap().remove(0)
}
pub fn size(s: &Scenario, a: &Attempt) -> usize {
    serde_json::to_vec(&(s, a)).unwrap().len()
}
pub fn fill(s: &Scenario, a: &mut Attempt, target: usize) {
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
    assert_eq!(remaining, 0);
    assert_eq!(size(s, a), target);
    work::context(a, s).unwrap();
}
pub fn oversized_fresh(mut s: Scenario) -> Scenario {
    s.steps = vec!["x".repeat(4000); 50];
    s.hints = vec!["hint".into(); 50];
    s.sources = vec!["https://example.invalid/".into(); 50];
    s.guide = (0..50)
        .map(|_| LessonStep {
            title: "t".repeat(4000),
            why: "w".repeat(4000),
            action: "a".repeat(4000),
            commands: vec![],
        })
        .collect();
    let target = work::MAX_CONTEXT - 256;
    let used = serde_json::to_vec(&s).unwrap().len();
    assert!(used < target);
    let mut left = target - used;
    for h in &mut s.hints {
        let n = left.min(4000 - h.len());
        h.push_str(&"h".repeat(n));
        left -= n;
    }
    for h in &mut s.sources {
        let n = left.min(4000 - h.len());
        h.push_str(&"s".repeat(n));
        left -= n;
    }
    assert_eq!(left, 0);
    assert_eq!(serde_json::to_vec(&s).unwrap().len(), target);
    s
}
