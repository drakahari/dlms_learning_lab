//! Opt-in networking presentation. Never changes another track's UI or grading.
use super::*;
pub(super) fn enabled(s: &Scenario) -> bool {
    s.track == "CCNA"
}
pub(super) fn lead(ui: &mut egui::Ui, eyebrow: &str, text: &str) {
    lesson_presentation::lead(ui, eyebrow, text);
}
pub(super) fn concepts(
    ui: &mut egui::Ui,
    s: &Scenario,
    note: &str,
    items: impl IntoIterator<Item = (String, String)>,
) {
    lesson_presentation::concepts(ui, s, note, items);
}
pub(super) fn prose(ui: &mut egui::Ui, text: &str) -> egui::Response {
    lesson_presentation::prose(ui, text)
}
pub(super) fn assignment(ui: &mut egui::Ui, s: &Scenario) -> egui::Response {
    lesson_presentation::assignment(ui, s)
}
pub(super) fn example(ui: &mut egui::Ui, s: &Scenario, workspace_heading: bool) -> egui::Response {
    let heading = ui.strong("GUIDED PRACTICE");
    let count = lesson_presentation::entry(s).map_or(0, |e| e.practice.len());
    if ui.available_width() >= 900. && count > 1 {
        ui.columns(count, |columns| {
            for (n, column) in columns.iter_mut().enumerate() {
                lesson_presentation::practice_step(column, s, n);
            }
        });
    } else {
        for n in 0..count {
            lesson_presentation::practice_step(ui, s, n);
        }
    }
    ui.add_space(6.);
    ui.separator();
    ui.add_space(6.);
    if workspace_heading {
        ui.strong("Practice workspace")
    } else {
        heading
    }
}
