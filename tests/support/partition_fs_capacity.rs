//! Private bounded admission fixtures; never normal curriculum.
#![allow(dead_code)] // UI and backend binaries exercise different subsets.
use dlms_learning_lab::{
    pack::{self, LessonStep, Scenario},
    partition_fs_workspace as work,
    progress::Attempt,
};
pub fn padded_source(mut s: Scenario) -> Scenario {
    s.steps = vec!["Private disk capacity text ".to_owned() + &"x".repeat(3900); 50];
    s.hints = vec!["Read current applied configuration; drafts have not been applied.".into(); 50];
    s.sources = vec!["https://example.invalid/".to_owned() + &"s".repeat(3900); 50];
    // Keep every instruction paired with valid guidance while retaining the same
    // 120,000 bytes of text pressure (50 * 3 * 800, previously 20 * 3 * 2,000).
    s.guide = (0..s.steps.len())
        .map(|_| LessonStep {
            title: "t".repeat(800),
            why: "w".repeat(800),
            action: "a".repeat(800),
            commands: vec![],
        })
        .collect();
    let raw = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios: vec![s],
    })
    .unwrap();
    assert!(raw.len() < pack::MAX_PACK_BYTES);
    let s = pack::load(&raw).unwrap().remove(0);
    work::binding::source(&s).unwrap();
    s
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
        "Capacity fixture must fit unchanged120x8192 transcript bounds"
    );
    assert_eq!(size(s, a), target);
    work::binding::context(a, s).unwrap();
}
