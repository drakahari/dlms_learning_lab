//! Thread-local sizing-pass diagnostics only; no learner data, telemetry or grade.
std::thread_local! {static JSON_SIZING:std::cell::Cell<[usize;2]>=const{std::cell::Cell::new([0;2])};}
pub fn json_admission_calls() -> [usize; 2] {
    JSON_SIZING.with(std::cell::Cell::get)
}
pub(crate) fn physical_json() {
    JSON_SIZING.with(|c| {
        let mut n = c.get();
        n[0] = n[0].saturating_add(1);
        c.set(n);
    });
}
pub(crate) fn backup_json() {
    JSON_SIZING.with(|c| {
        let mut n = c.get();
        n[1] = n[1].saturating_add(1);
        c.set(n);
    });
}
