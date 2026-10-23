//! Opt-in native lifecycle observations. Never log learner input, profile paths or content.
use std::sync::{
    OnceLock,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Instant;
static START: OnceLock<Instant> = OnceLock::new();
static ENABLED: OnceLock<bool> = OnceLock::new();
static FRAMES: AtomicUsize = AtomicUsize::new(0);
fn enabled() -> bool {
    *ENABLED.get_or_init(|| std::env::var("DLMS_STARTUP_TRACE").as_deref() == Ok("1"))
}
pub fn mark(event: &str) {
    if enabled() {
        eprintln!(
            "native-startup {}ms {event}",
            START.get_or_init(Instant::now).elapsed().as_millis()
        );
    }
}
pub fn frame(ctx: &eframe::egui::Context, completed: bool) {
    if !enabled() {
        return;
    }
    let frame = if completed {
        FRAMES.fetch_add(1, Ordering::Relaxed)
    } else {
        FRAMES.load(Ordering::Relaxed)
    };
    if frame < 4 {
        let (focused, minimized) = ctx.input(|i| (i.focused, i.viewport().minimized));
        mark(&format!(
            "update-{} frame={frame} focused={focused} minimized={minimized:?}",
            if completed { "end" } else { "begin" }
        ));
    }
}
