use crate::pack::Scenario;
#[derive(Default)]
pub struct Filter {
    pub search: String,
    pub track: String,
    pub topic: String,
    pub difficulty: String,
    pub status: String,
    pub max_minutes: u16,
    pub favorites: bool,
    pub recent: bool,
}
pub fn matches(s: &Scenario, f: &Filter, started: bool, complete: bool, favorite: bool) -> bool {
    let q = f.search.to_lowercase();
    (q.is_empty()
        || format!("{} {} {} {}", s.title, s.summary, s.topic, s.track)
            .to_lowercase()
            .contains(&q))
        && (f.track.is_empty() || f.track == s.track)
        && (f.topic.is_empty() || f.topic == s.topic)
        && (f.difficulty.is_empty() || f.difficulty == s.difficulty)
        && (f.max_minutes == 0 || s.minutes <= f.max_minutes)
        && (!f.favorites || favorite)
        && (!f.recent || started)
        && match f.status.as_str() {
            "Complete" => complete,
            "In progress" => started && !complete,
            "Not started" => !started,
            _ => true,
        }
}
/// Explicitly synthetic metadata stress fixture, never counted or launched as authored content.
pub fn synthetic_catalog(seed: &[Scenario], count: usize) -> Vec<Scenario> {
    (0..count)
        .map(|i| {
            let mut s = seed[i % seed.len()].clone();
            s.id = format!("synthetic-{i}");
            s.title = format!("SYNTHETIC {:04} • {}", i, s.title);
            s
        })
        .collect()
}
