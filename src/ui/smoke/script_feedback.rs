//! Closed capture of overall failure versus selected passing/failing inputs.
//! Actual editor/Run/result-row events only; no grading record is installed.
use super::*;
const FIRST: usize = 47000;
const SOURCE: &str = "printf 'A\\nb\\nb\\nz\\n'\n";
impl LabApp {
    pub(in crate::ui) fn script_feedback_stage(smoke: bool, name: Option<&str>) -> Option<usize> {
        (smoke && name == Some("script-feedback")).then_some(FIRST)
    }
    pub(in crate::ui) fn script_feedback_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert!(self.smoke && self._smoke_profile.is_some());
        let phase = stage - FIRST;
        let i = self
            .scenarios
            .iter()
            .position(|s| s.id == "shell-command-options")
            .unwrap();
        let id = self.scenarios[i].id.clone();
        let snapshot = egui::Id::new("script-feedback-unchanged-progress");
        match phase {
            0 => {
                assert_eq!(
                    (
                        one_off_boundary::before_len(&self.scenarios),
                        one_off_boundary::before_len(&self.units)
                    ),
                    (176, 46)
                );
                assert!(self.progress.attempts.is_empty());
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.launch(i);
                self.set_script_view(i, teaching::View::Assignment);
            }
            1 => {
                if !self.case_lesson_focus(ctx) {
                    return (false, None);
                }
            }
            2 => self.case_lesson_type(SOURCE),
            3 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            4 | 6 | 9 | 12 => {
                if !self.case_lesson_wait(ctx) {
                    return (false, None);
                }
                self.case_lesson_assignment(i, Some(SOURCE), false);
                let checks =
                    engine::checks(&self.progress.attempts[&id].state, &self.scenarios[i].goal);
                assert_eq!(checks.len(), 12);
                assert_eq!(
                    checks.iter().map(|c| c.0).collect::<Vec<_>>(),
                    [
                        true, false, false, false, false, false, false, false, false, false, false,
                        false
                    ]
                );
                let (case, pass, rect) = self
                    .metrics
                    .script_case_status
                    .expect("Actual painted selected-case status");
                assert_eq!(
                    (case, pass),
                    if phase == 4 { (1, false) } else { (0, true) }
                );
                if !self.reveal_smoke_control(ctx, rect) {
                    return (false, None);
                }
                let before = serde_json::to_value(&self.progress).unwrap();
                if phase == 4 {
                    ctx.data_mut(|d| d.insert_temp(snapshot, before));
                } else {
                    assert_eq!(
                        ctx.data(|d| d.get_temp::<serde_json::Value>(snapshot))
                            .unwrap(),
                        before
                    );
                }
                let content = ctx.content_rect();
                let ppp = ctx.pixels_per_point();
                let zoom = ctx.zoom_factor();
                println!(
                    "script-feedback phase{phase} actual_content={content:?} PPP={ppp} zoom={zoom}; overall11fail, selected_case={case}, pass={pass}"
                );
                return (
                    true,
                    Some(format!(
                        "script-feedback-{}-{}",
                        if phase == 4 {
                            "selected-fail"
                        } else {
                            "selected-pass"
                        },
                        match phase {
                            4 | 6 => "dark100",
                            9 => "light150",
                            _ => "dark200",
                        }
                    )),
                );
            }
            5 => {
                let r = self
                    .metrics
                    .script_case_rows
                    .iter()
                    .find(|r| r.0 == 0)
                    .unwrap()
                    .1;
                if !self.click_visible_smoke_control(ctx, Some(r)) {
                    return (false, None);
                }
            }
            7 | 10 | 13 => {
                assert_eq!(self.script_case, 0);
                let rect = self.metrics.script_feedback.expect("Actual overall result");
                if !self.reveal_smoke_control(ctx, rect) {
                    return (false, None);
                }
                assert_eq!(
                    ctx.data(|d| d.get_temp::<serde_json::Value>(snapshot))
                        .unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                return (
                    true,
                    Some(format!(
                        "script-feedback-overall-fail-{}",
                        match phase {
                            7 => "dark100",
                            10 => "light150",
                            _ => "dark200",
                        }
                    )),
                );
            }
            8 | 11 => {
                self.scale = if phase == 8 { 1.5 } else { 2. };
                self.theme_id = if phase == 8 {
                    "dlms-light"
                } else {
                    "dlms-dark"
                }
                .into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(946., 929.)));
            }
            14 => {
                assert_eq!(
                    ctx.data(|d| d.get_temp::<serde_json::Value>(snapshot))
                        .unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                println!(
                    "PASS native script-feedback: actual Run, automatic first failure, actual passing-case selection, explicit per-case versus overall results, unchanged full progress, dark100/light150/dark200 observed geometry. Synthetic technical evidence only."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return (false, None);
            }
            _ => unreachable!(),
        }
        (true, None)
    }
}
