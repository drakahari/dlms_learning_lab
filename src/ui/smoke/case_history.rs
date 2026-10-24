//! App-only genuine frozen7→8 native acceptance. Controlled data; no reference State.
use super::*;
use dlms_learning_lab::shell;
use serde::Deserialize;
use std::io::Write;
const WRITER7: &[u8] =
    include_bytes!("../../../tests/fixtures/shell-v7-synthetic-writer-progress.json");
const ID: &str = "shell-handover";
const SOLUTION: &str = "site=$SITE\ntask=$TASK\ncase \"$task\" in\n '') printf 'Site: %s\\nTask: \\n' \"$site\";;\n *) printf 'Site: %s\\nTask: %s\\n' \"$site\" \"$task\";;\nesac\n";
const LOOP: &str = "case \"$(printf entered > native-case-entered.txt; while true; do :; done)\" in *) printf parent-must-not-run;; esac\n";
const INCOMPLETE: &str = "case \"$TASK\" in\n";
const LAST: usize = 15075;
#[derive(Deserialize)]
struct Captured {
    id: String,
    record: shell::Record,
}
fn records() -> Vec<Captured> {
    include_str!("../../../tests/fixtures/shell-v7-authored-records.jsonl")
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}
fn shortcut(key: egui::Key) -> Vec<egui::Event> {
    [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        })
        .collect()
}
fn case8_editor_band(viewport: egui::Rect, control_height: f32) -> egui::Rect {
    egui::Rect::from_center_size(
        viewport.center(),
        egui::vec2(viewport.width(), control_height.min(viewport.height())),
    )
}
impl LabApp {
    fn case8_focus_editor(&mut self, ctx: &egui::Context) -> bool {
        let phase = ctx
            .data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))))
            .unwrap_or(0);
        let clicked = if phase >= 2 {
            self.click_smoke_control(ctx, None)
        } else {
            let viewport = self
                .metrics
                .editor_viewport
                .expect("actual script editor viewport");
            let band = case8_editor_band(viewport, ctx.style().spacing.interact_size.y);
            assert!(viewport.contains_rect(band));
            if !self.reveal_smoke_control(ctx, band) {
                return false;
            }
            self.click_smoke_control(ctx, Some(band))
        };
        if clicked {
            assert!(
                self.metrics
                    .focus
                    .is_some_and(|id| ctx.memory(|m| m.has_focus(id))),
                "native band click must focus the actual script TextEdit"
            );
        }
        clicked
    }
    fn case8_input(&mut self, text: &str) {
        assert!(text.len() <= shell::MAX_SOURCE);
        self.pending_events = shortcut(egui::Key::A);
        self.pending_events.push(egui::Event::Text(text.into()));
    }
    fn case8_history(&self) {
        let original: Progress = serde_json::from_slice(WRITER7).unwrap();
        let mut six = 0;
        let mut seven = 0;
        for row in records() {
            let a = &self.progress.attempts[&row.id];
            let w = a.workspace.as_ref().unwrap();
            if let Some(old) = &original.attempts[&row.id]
                .workspace
                .as_ref()
                .unwrap()
                .script_history
            {
                assert_eq!(old.engine_revision, 6);
                assert_eq!(w.script_history.as_ref(), Some(old));
            }
            let actual_six = w
                .script_history
                .as_ref()
                .into_iter()
                .chain(&w.script_history_more)
                .filter(|r| r.engine_revision == 6)
                .count();
            assert_eq!(
                actual_six,
                usize::from(
                    original.attempts[&row.id]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .script_history
                        .is_some()
                )
            );
            six += actual_six;
            let historical = w
                .script_history
                .as_ref()
                .into_iter()
                .chain(&w.script_history_more)
                .filter(|r| r.engine_revision == 7)
                .collect::<Vec<_>>();
            let current_old = match &a.state {
                engine::State::Script { run: Some(r), .. } if r.engine_revision == 7 => Some(r),
                _ => None,
            };
            assert_eq!(
                historical.len() + usize::from(current_old.is_some()),
                1,
                "one genuine7 record {}",
                row.id
            );
            assert!(
                historical
                    .into_iter()
                    .chain(current_old)
                    .all(|r| r == &row.record)
            );
            seven += 1;
        }
        assert_eq!((six, seven), (33, 37));
    }
    fn case8_wait(&self, ctx: &egui::Context, label: &'static str) -> bool {
        if self.script_job.is_none() {
            return true;
        }
        let started = ctx.data_mut(|d| {
            let key = egui::Id::new(("case8-native-wait", self.smoke_stage, label));
            let v = d.get_temp::<Instant>(key).unwrap_or_else(Instant::now);
            d.insert_temp(key, v);
            v
        });
        assert!(
            started.elapsed().as_secs() < 15,
            "bounded native job must finish: {label}"
        );
        false
    }
    fn case8_pass(&self, i: usize) {
        let a = &self.progress.attempts[ID];
        assert!(progress::current_solved(a, &self.scenarios[i]));
        let engine::State::Script {
            source,
            run: Some(r),
        } = &a.state
        else {
            panic!()
        };
        assert_eq!(r.engine_revision, 8);
        assert_eq!(r.source, *source);
        assert_eq!(source, SOLUTION);
        self.case8_history();
    }
    pub(super) fn case_history_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert_eq!(shell::EVALUATOR_REVISION, 8);
        let i = self.scenarios.iter().position(|s| s.id == ID).unwrap();
        let mut shot = None;
        match stage {
            15000 => {
                assert!(self.smoke && self._smoke_profile.is_some());
                assert_eq!(
                    self.scenarios.len(),
                    113,
                    "native acceptance requires the merged normal113 library"
                );
                assert_eq!(
                    self.path.parent(),
                    Some(self._smoke_profile.as_ref().unwrap().path())
                );
                assert!(!self.path.with_extension("before-shell-v8.json").exists());
                std::fs::write(&self.path, WRITER7).unwrap();
                let mut f = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(self.path.parent().unwrap().join("original-writer7.json"))
                    .unwrap();
                f.write_all(WRITER7).unwrap();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(self.progress.attempts.len(), 102);
                assert_eq!(
                    self.progress.migration_receipts.as_ref().unwrap().shell,
                    vec![7]
                );
                self.launch(i);
                self.set_script_view(i, teaching::View::Assignment);
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            }
            15001 => {
                if !self.click_smoke_control(ctx, self.metrics.mode_learn) {
                    return (false, None);
                }
            }
            15002 => {
                self.save();
                assert!(self.storage_error.is_none());
                assert_eq!(
                    std::fs::read(self.path.with_extension("before-shell-v8.json")).unwrap(),
                    WRITER7
                );
                assert!(!progress::current_solved(
                    &self.progress.attempts[ID],
                    &self.scenarios[i]
                ));
                self.case8_history();
            }
            15003 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.script_history) {
                    return (false, None);
                }
            }
            15004 => shot = Some("case8-genuine6-and7-earlier-output".into()),
            15005 | 15012 | 15017 | 15024 | 15028 | 15034 | 15038 => {
                if !self.case8_focus_editor(ctx) {
                    return (false, None);
                }
            }
            15006 | 15018 | 15029 | 15039 => self.case8_input(SOLUTION),
            15007 | 15019 | 15026 | 15030 | 15036 | 15040 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            15008 | 15020 | 15031 | 15041 => {
                if !self.case8_wait(ctx, "assignment") {
                    return (false, None);
                }
                self.case8_pass(i);
                if !self.reveal_smoke_control(
                    ctx,
                    self.metrics
                        .script_completion
                        .expect("actual current completion block"),
                ) {
                    return (false, None);
                }
                shot = Some(format!("case8-current8-passing-{stage}"));
            }
            15009 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.lesson_tabs.get(1).copied())
                {
                    return (false, None);
                }
            }
            15010 => {
                let a = &self.progress.attempts[ID];
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(l.example_revision, 7);
                let original: Progress = serde_json::from_slice(WRITER7).unwrap();
                assert_eq!(
                    l.example_source,
                    original.attempts[ID]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .script_learning
                        .as_ref()
                        .unwrap()
                        .example_source
                );
                assert!(progress::current_solved(a, &self.scenarios[i]));
                self.case8_history();
                shot = Some("case8-example-still-separate-earlier7".into());
            }
            15011 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.lesson_tabs.get(2).copied())
                {
                    return (false, None);
                }
            }
            15013 => self.case8_input(LOOP),
            15014 => {
                assert!(
                    self.metrics
                        .focus
                        .is_some_and(|id| ctx.memory(|m| m.has_focus(id)))
                );
                self.pending_events = shortcut(egui::Key::Enter);
            }
            15015 => {
                let phase = ctx
                    .data(|d| {
                        d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage)))
                    })
                    .unwrap_or(0);
                if phase < 2 {
                    let job = self
                        .script_job
                        .as_ref()
                        .expect("Stop must occur while actual job runs");
                    if !job.has_case_native_child_marker() {
                        return (false, None);
                    }
                }
                if !self.click_visible_smoke_control(ctx, self.metrics.stop) {
                    return (false, None);
                }
            }
            15016 => {
                assert!(self.script_job.is_none());
                assert!(!progress::current_solved(
                    &self.progress.attempts[ID],
                    &self.scenarios[i]
                ));
                assert!(
                    matches!(&self.progress.attempts[ID].state,engine::State::Script{run:None,source}if source==LOOP)
                );
                assert!(
                    self.notice
                        .as_deref()
                        .is_some_and(|n| n.starts_with("Stopped"))
                );
                self.case8_history();
                shot = Some("case8-stopped-reached-selector-capture".into());
            }
            15021 => {
                self.save();
                assert!(self.storage_error.is_none());
                let before = serde_json::to_value(&self.progress.attempts).unwrap();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts).unwrap(),
                    before
                );
                self.case8_pass(i);
                let original: Progress = serde_json::from_slice(WRITER7).unwrap();
                for (id, old) in original.attempts.iter().filter(|(id, _)| id.as_str() != ID) {
                    assert_eq!(
                        serde_json::to_value(&self.progress.attempts[id]).unwrap(),
                        serde_json::to_value(old).unwrap(),
                        "unopened context {id}"
                    );
                }
                assert_eq!(self.progress.favorites, original.favorites);
                assert_eq!(
                    self.progress.migration_receipts.as_ref().unwrap().shell,
                    vec![7, 8]
                );
            }
            15022 | 15032 | 15042 => {
                let (scale, name, size) = match stage {
                    15022 => (1.5, "dlms-light", egui::vec2(960., 900.)),
                    15032 => (2., "dlms-dark", egui::vec2(960., 900.)),
                    _ => (1., "dlms-light", egui::vec2(1200., 850.)),
                };
                self.scale = scale;
                self.theme_id = name.into();
                theme::apply(ctx, name);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            }
            15023 | 15033 => {
                let expected = if stage == 15023 { 1.5 } else { 2. };
                if (ctx.zoom_factor() - expected).abs() > 0.01 {
                    let n = ctx.data_mut(|d| {
                        let key = egui::Id::new(("case8-zoom-wait", stage));
                        let n = d.get_temp::<u8>(key).unwrap_or(0) + 1;
                        d.insert_temp(key, n);
                        n
                    });
                    assert!(n < 20, "native text scale must settle");
                    return (false, None);
                }
            }
            15025 | 15035 => self.case8_input(INCOMPLETE),
            15027 | 15037 => {
                if !self.case8_wait(ctx, "malformed input") {
                    return (false, None);
                }
                let a = &self.progress.attempts[ID];
                assert!(!progress::current_solved(a, &self.scenarios[i]));
                let engine::State::Script {
                    run: Some(run),
                    source,
                } = &a.state
                else {
                    panic!()
                };
                assert_eq!(source, INCOMPLETE);
                assert_eq!(run.results.len(), 3);
                assert!(run.results.iter().all(|r| {
                    r.error
                        .as_deref()
                        .is_some_and(|e| teaching::diagnostic(e) == teaching::Diagnostic::Syntax)
                }));
                self.case8_history();
                if !self.reveal_smoke_control(
                    ctx,
                    self.metrics
                        .script_diagnostic
                        .expect("actual current diagnostic and retry block"),
                ) {
                    return (false, None);
                }
                shot = Some(format!("case8-native-syntax-feedback-{stage}"));
            }
            15043 => {
                self.save();
                assert!(self.storage_error.is_none());
                self.settings_open = true;
                self.settings_data = true;
            }
            15044 => {
                if !self.click_smoke_control(ctx, self.metrics.data_export) {
                    return (false, None);
                }
            }
            15045 | 15050 | 15057 => {
                if !self.click_smoke_control(ctx, self.metrics.data_filename) {
                    return (false, None);
                }
            }
            15046 => self.case8_input("current8-native-backup.json"),
            15047 | 15052 | 15059 => {
                if ctx
                    .data(|d| {
                        d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage)))
                    })
                    .unwrap_or(0)
                    < 2
                    && !self.reveal_file_action(ctx)
                {
                    return (false, None);
                }
                if !self.click_smoke_control(ctx, self.metrics.data_file_action) {
                    return (false, None);
                }
            }
            15048 => {
                let out = self
                    .path
                    .parent()
                    .unwrap()
                    .join("current8-native-backup.json");
                assert!(out.is_file());
                let json: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(out).unwrap()).unwrap();
                assert!(json["progress"].get("migration_receipts").is_none());
                self.save();
                let bytes = std::fs::read(&self.path).unwrap();
                ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new("case8-current-before-preview"), bytes)
                });
            }
            15049 | 15056 => {
                if !self.click_smoke_control(ctx, self.metrics.data_restore) {
                    return (false, None);
                }
            }
            15051 | 15058 => self.case8_input("original-writer7.json"),
            15053 | 15060 => {
                if self.metrics.data_confirm.is_none() {
                    let began = ctx.data_mut(|d| {
                        let key = egui::Id::new(("case8-restore-wait", stage));
                        let t = d.get_temp::<Instant>(key).unwrap_or_else(Instant::now);
                        d.insert_temp(key, t);
                        t
                    });
                    assert!(
                        began.elapsed().as_secs() < 20,
                        "Review must become available: {}",
                        self.data_tools.restore_diagnostic()
                    );
                    return (false, None);
                }
                let n = ctx.data_mut(|d| {
                    let key = egui::Id::new(("case8-review-settle", stage));
                    let n = d.get_temp::<u8>(key).unwrap_or(0) + 1;
                    d.insert_temp(key, n);
                    n
                });
                if n < 3 {
                    return (false, None);
                }
                shot = Some(format!("case8-original7-review-{stage}"));
            }
            15054 => {
                if !self.click_smoke_control(ctx, self.metrics.data_cancel) {
                    return (false, None);
                }
            }
            15055 => {
                assert!(!self.data_tools.modal_open());
                let bytes = ctx
                    .data(|d| d.get_temp::<Vec<u8>>(egui::Id::new("case8-current-before-preview")))
                    .unwrap();
                assert_eq!(std::fs::read(&self.path).unwrap(), bytes);
                assert!(
                    !self
                        .path
                        .parent()
                        .unwrap()
                        .join("progress-before-restore-1.json")
                        .exists()
                );
                assert_eq!(
                    std::fs::read(self.path.with_extension("before-shell-v8.json")).unwrap(),
                    WRITER7
                );
                self.case8_pass(i);
                shot = Some("case8-cancel-keeps-current-work".into());
            }
            15061 => {
                if !self.click_smoke_control(ctx, self.metrics.data_confirm) {
                    return (false, None);
                }
            }
            15062 => {
                assert!(!self.data_tools.modal_open());
                assert!(self.selected.is_none());
                let bytes = ctx
                    .data(|d| d.get_temp::<Vec<u8>>(egui::Id::new("case8-current-before-preview")))
                    .unwrap();
                assert_eq!(
                    std::fs::read(
                        self.path
                            .parent()
                            .unwrap()
                            .join("progress-before-restore-1.json")
                    )
                    .unwrap(),
                    bytes
                );
                assert_eq!(
                    std::fs::read(self.path.with_extension("before-shell-v8.json")).unwrap(),
                    WRITER7
                );
                self.case8_history();
                let a = &self.progress.attempts[ID];
                assert!(!progress::current_solved(a, &self.scenarios[i]));
                let engine::State::Script { run: Some(run), .. } = &a.state else {
                    panic!()
                };
                assert_eq!(run.engine_revision, 8);
                self.settings_open = false;
                self.launch(i);
                self.set_script_view(i, teaching::View::Assignment);
            }
            15063 => {
                shot = Some("case8-restored-original-draft-rechecked-honestly".into());
                let others = self
                    .progress
                    .attempts
                    .iter()
                    .filter(|(id, _)| id.as_str() != ID)
                    .map(|(id, a)| (id.clone(), serde_json::to_value(a).unwrap()))
                    .collect::<std::collections::BTreeMap<_, _>>();
                ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new("case8-before-selected-reset-others"), others)
                });
                let selected = serde_json::to_value(&self.progress.attempts[ID]).unwrap();
                ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new("case8-before-reset-selected"), selected)
                });
            }
            15064 | 15067 => {
                if !self.click_smoke_control(ctx, self.metrics.reset) {
                    return (false, None);
                }
            }
            15065 => {
                if !self.click_smoke_control(ctx, self.metrics.reset_cancel) {
                    return (false, None);
                }
            }
            15066 => {
                assert!(!self.reset_pending);
                let value = ctx
                    .data(|d| {
                        d.get_temp::<serde_json::Value>(egui::Id::new(
                            "case8-before-reset-selected",
                        ))
                    })
                    .unwrap();
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[ID]).unwrap(),
                    value
                );
            }
            15068 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.confirm_reset) {
                    return (false, None);
                }
            }
            15069 => {
                let a = &self.progress.attempts[ID];
                assert_eq!(a.state, self.scenarios[i].initial);
                assert!(!progress::current_solved(a, &self.scenarios[i]));
                let others = ctx
                    .data(|d| {
                        d.get_temp::<std::collections::BTreeMap<String, serde_json::Value>>(
                            egui::Id::new("case8-before-selected-reset-others"),
                        )
                    })
                    .unwrap();
                for (id, value) in others {
                    assert_eq!(
                        serde_json::to_value(&self.progress.attempts[&id]).unwrap(),
                        value
                    );
                }
                assert_eq!(
                    std::fs::read(self.path.with_extension("before-shell-v8.json")).unwrap(),
                    WRITER7
                );
                shot = Some("case8-selected-reset-preserves-other-contexts".into());
            }
            15070 => {
                if !self.click_smoke_control(ctx, self.metrics.mode_challenge) {
                    return (false, None);
                }
            }
            15071 => {
                assert!(self.progress.attempts[ID].challenge);
                assert!(self.metrics.lesson_tabs.is_empty());
                assert!(!self.metrics.help_visible);
                shot = Some("case8-independent-challenge".into());
            }
            15072 => {
                self.save();
                assert!(self.storage_error.is_none());
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert!(!progress::current_solved(
                    &self.progress.attempts[ID],
                    &self.scenarios[i]
                ));
                assert_eq!(
                    self.progress.migration_receipts.as_ref().unwrap().shell,
                    vec![7, 8]
                );
            }
            15073 | 15074 => {}
            LAST => {
                println!(
                    "PASS scoped native genuine7→8: real6+7 history, actual typed case8 Run/Stop/retry, separate example, exact first backup/readback,150/200 actual syntax error/repair, export/old7 preview Cancel+Confirm with current recheck/raw safety copy, selected Reset/Challenge. Synthetic data only; no novice/other-platform/screen-reader claim."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => unreachable!("bounded dedicated genuine7→8 stage"),
        }
        (true, shot)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn genuine7_native_plan_uses_real_frozen_data_and_bounded_case8_input_targets_only() {
        assert_eq!(shell::EVALUATOR_REVISION, 8);
        for height in [1., 18., 180., 360.] {
            let viewport =
                egui::Rect::from_min_size(egui::pos2(40., 100.), egui::vec2(400., height));
            let band = case8_editor_band(viewport, 35.);
            assert!(viewport.contains_rect(band));
            assert_eq!(band.center(), viewport.center());
            assert_eq!(band.height(), height.min(35.));
            assert_eq!(band.width(), viewport.width());
        }
        let labs = pack::bundled().unwrap();
        assert_eq!(labs.len(), 113);
        let p = progress::decode(WRITER7, &labs).unwrap();
        assert_eq!(p.attempts.len(), 102);
        assert_eq!(records().len(), 37);
        assert!(SOLUTION.len() < shell::MAX_SOURCE && LOOP.len() < shell::MAX_SOURCE);
        const {
            assert!(LAST - 15000 < 256);
        };
        let s = labs.iter().find(|s| s.id == ID).unwrap();
        let engine::Goal::Script { cases } = &s.goal else {
            panic!()
        };
        assert_eq!(cases.len(), 3);
        for c in cases {
            assert!(c.accepts(&shell::run(SOLUTION, c)));
        }
        assert!(records().iter().all(|r| r.record.engine_revision == 7));
    }
}
