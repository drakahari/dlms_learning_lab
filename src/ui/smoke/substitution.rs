//! Synthetic-profile native regression for the actual current public evaluator.
//! This flow migrates frozen6 data; genuine7→8 is a separate acceptance target.
//! The frozen revision-six records are development fixtures, never learner data.
use super::*;
impl LabApp {
    pub(super) fn reveal_file_action(&mut self, ctx: &egui::Context) -> bool {
        let action = self.metrics.data_file_action.expect("file chooser action");
        let visible = self
            .metrics
            .data_file_viewport
            .expect("file chooser clip")
            .intersect(ctx.content_rect());
        // egui's scroll clip can round one point inside the painted border.
        // Pointer/text must be visible; do not keep scrolling at its lower limit.
        if visible.expand(1.).contains_rect(action) {
            return true;
        }
        let id = egui::Id::new(("capture-file-scroll", self.smoke_stage));
        let tries = ctx.data_mut(|d| {
            let n = d.get_temp::<usize>(id).unwrap_or(0) + 1;
            d.insert_temp(id, n);
            n
        });
        assert!(
            tries <= 20,
            "Chooser action did not become visible: {action:?} in {visible:?}"
        );
        let delta = if action.bottom() > visible.bottom() {
            visible.bottom() - action.bottom() - controls::GAP
        } else {
            visible.top() - action.top() + controls::GAP
        };
        self.pending_events.extend([
            egui::Event::PointerMoved(egui::pos2(visible.left() + 1., visible.center().y)),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., delta),
                modifiers: Default::default(),
            },
        ]);
        self.started = Instant::now();
        ctx.request_repaint();
        false
    }
    pub(super) fn substitution_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        let i = self
            .scenarios
            .iter()
            .position(|s| s.id == "shell-handover")
            .unwrap();
        let id = self.scenarios[i].id.clone();
        let mut shot = None;
        let key = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        };
        if std::env::var_os("DLMS_SMOKE_DIAGNOSTICS").is_some() {
            eprintln!("capture-native stage {stage}");
        }
        match stage {
            7000 => {
                const { assert!(shell::EVALUATOR_REVISION >= 7); };
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                let old: Vec<serde_json::Value> = serde_json::from_str(include_str!(
                    "../../../tests/fixtures/shell-v6-authored-records.json"
                )).unwrap();
                let mut p = Progress { schema_version: 1, ..Default::default() };
                for old in old {
                    let s = self.scenarios.iter().find(|s| s.id == old["id"].as_str().unwrap()).unwrap();
                    let record: shell::Record = serde_json::from_value(old["record"].clone()).unwrap();
                    assert_eq!(record.engine_revision, 6);
                    let mut a = Attempt::new(s, 1);
                    a.state = engine::State::Script { source: record.source.clone(), run: Some(record) };
                    p.attempts.insert(s.id.clone(), a);
                }
                assert_eq!(p.attempts.len(), 33);
                p.favorites.insert(id.clone());
                let bytes = serde_json::to_vec_pretty(&p).unwrap();
                std::fs::write(&self.path, &bytes).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("capture-native-original6"), bytes));
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                self.launch_assignment_fixture(i);
                self.set_script_view(i, teaching::View::Assignment);
            }
            7001 => {
                let bytes = ctx.data(|d| d.get_temp::<Vec<u8>>(egui::Id::new("capture-native-original6"))).unwrap();
                assert_eq!(std::fs::read(self.path.with_extension(format!("before-shell-v{}.json",shell::EVALUATOR_REVISION))).unwrap(), bytes);
                let a = &self.progress.attempts[&id];
                assert!(!progress::current_solved(a, &self.scenarios[i]));
                assert!(progress::legacy_script_credit(a, &self.scenarios[i]));
                assert_eq!(a.workspace.as_ref().unwrap().script_history.as_ref().unwrap().engine_revision, 6);
                shot = Some("capture-native-actual6-history".into());
            }
            7002 | 7010 | 7016 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.editor_viewport) { return (false, None); }
            }
            7003 | 7017 => self.pending_events = vec![key(egui::Key::A), egui::Event::Text(
                "site=$(printf '%s' \"$SITE\")\ntask=$(printf '%s' \"$TASK\")\nprintf 'Site: %s\\nTask: %s\\n' \"$site\" \"$task\"\n".into()
            )],
            7004 | 7007 | 7018 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) { return (false, None); }
            }
            7005 | 7019 => {
                if self.script_job.is_some() { return (false, None); }
                let a = &self.progress.attempts[&id];
                assert!(progress::current_solved(a, &self.scenarios[i]));
                assert!(progress::legacy_script_credit(a, &self.scenarios[i]));
                let engine::State::Script { run: Some(r), .. } = &a.state else { panic!() };
                assert_eq!(r.engine_revision, shell::EVALUATOR_REVISION);
                assert_eq!(a.workspace.as_ref().unwrap().script_history.as_ref().unwrap().engine_revision, 6);
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert!(progress::current_solved(&self.progress.attempts[&id], &self.scenarios[i]));
                shot = Some(format!("capture-native-current-{}", if stage == 7005 { "complete" } else { "repaired" }));
            }
            7006 => {
                self.set_script_view(i, teaching::View::Example);
                let l = self.progress.attempts.get_mut(&id).unwrap().workspace.as_mut().unwrap().script_learning.as_mut().unwrap();
                l.example_source = format!("practice_marker=$(printf ready)\n{}", self.scenarios[i].teaching.as_ref().unwrap().example);
                l.example_result = None;
                self.dirty = true;
            }
            7008 => {
                if self.script_job.is_some() { return (false, None); }
                let a = &self.progress.attempts[&id];
                let l = a.workspace.as_ref().unwrap().script_learning.as_ref().unwrap();
                assert_eq!(l.example_revision, shell::EVALUATOR_REVISION);
                assert!(self.scenarios[i].teaching.as_ref().unwrap().fixture.accepts(l.example_result.as_ref().unwrap()));
                assert!(progress::current_solved(a, &self.scenarios[i]));
                shot = Some("capture-native-separate-example-current".into());
            }
            7009 => self.set_script_view(i, teaching::View::Assignment),
            7011 => self.pending_events = vec![key(egui::Key::A), egui::Event::Text(
                "pending=$(while true; do :; done)\nprintf parent-must-not-run\n".into()
            )],
            7012 => {
                assert!(self.metrics.focus.is_some_and(|id| ctx.memory(|m| m.has_focus(id))), "edited script retains keyboard focus");
                self.pending_events = vec![key(egui::Key::Enter)];
            }
            7013 => {
                let phase = ctx.data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage)))).unwrap_or(0);
                if phase < 2 {
                    assert!(self.script_job.is_some(), "Stop begins while the child loop is active");
                }
                if !self.click_visible_smoke_control(ctx, self.metrics.stop) { return (false, None); }
            }
            7014 => {
                assert!(self.script_job.is_none());
                assert!(!progress::current_solved(&self.progress.attempts[&id], &self.scenarios[i]));
                assert!(matches!(&self.progress.attempts[&id].state, engine::State::Script { run: None, source } if source.contains("parent-must-not-run")));
                assert!(self.notice.as_deref().is_some_and(|s| s.starts_with("Stopped")));
                shot = Some("capture-native-child-loop-stopped".into());
            }
            7015 => {
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert!(self.progress.favorites.contains(&id));
                assert!(!progress::current_solved(&self.progress.attempts[&id], &self.scenarios[i]));
            }
            7020 => {
                self.scale = 1.5; self.theme_id = "dlms-light".into(); theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 740.)));
                self.lesson_scroll_reset = true;
            }
            7021 => shot = Some("capture-native-light150".into()),
            7022 | 7025 => {
                if !self.reveal_smoke_control(ctx, self.metrics.check.unwrap()) { return (false, None); }
            }
            7023 => shot = Some("capture-native-light150-run-visible".into()),
            7024 => {
                self.scale = 2.; self.theme_id = "dlms-dark".into(); theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            7026 => shot = Some("capture-native-dark200-run-visible".into()),
            7027 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.scale = 1.; self.theme_id = "dlms-light".into(); theme::apply(ctx, &self.theme_id);
            }
            7028 => {
                if ctx.content_rect().width() < 1200. || (ctx.zoom_factor() - 1.).abs() > 0.01 {
                    let attempts = ctx.data_mut(|d| { let id=egui::Id::new("capture-native-geometry-settle"); let n=d.get_temp::<usize>(id).unwrap_or(0)+1; d.insert_temp(id,n); n });
                    assert!(attempts <= 20, "Native geometry did not settle before Settings");
                    return (false, None);
                }
                self.settings_open = true; self.settings_data = true;
            }
            7029 => shot = Some("capture-native-settings-data-before-export".into()),
            7030 => {
                let r = self.metrics.data_export.expect("rendered Settings export control");
                assert!(ctx.content_rect().contains_rect(r), "floating control must be visible in its window");
                if !self.click_smoke_control(ctx, Some(r)) { return (false, None); }
                assert!(self.data_tools.modal_open(), "Export must actually open the file chooser");
            }
            7031 => {
                assert!(self.metrics.data_file_action.is_some() || ctx.data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage)))).unwrap_or(0) >= 2, "stage {stage} file action absent; modal {}, exported {}", self.data_tools.modal_open(), self.path.parent().unwrap().join("dlms-learning-backup.json").exists());
                if ctx.data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases",self.smoke_stage)))).unwrap_or(0)<2 && !self.reveal_file_action(ctx) { return (false,None); }
                if !self.click_smoke_control(ctx, self.metrics.data_file_action) { return (false, None); }
            }
            7032 => {
                assert!(self.path.parent().unwrap().join("dlms-learning-backup.json").is_file());
                self.save();
                let bytes = std::fs::read(&self.path).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("capture-native-before-cancel"), bytes));
            }
            7033 => {
                let r = self.metrics.data_restore.expect("rendered Settings restore control");
                assert!(ctx.content_rect().contains_rect(r));
                if !self.click_smoke_control(ctx, Some(r)) { return (false, None); }
                assert!(self.data_tools.modal_open(), "Restore must actually open the file chooser");
            }
            7034 => {
                if !self.click_smoke_control(ctx, self.metrics.data_filename) { return (false, None); }
            }
            7035 => {
                assert!(self.metrics.data_filename_id.is_some_and(|id| ctx.memory(|m| m.has_focus(id))), "Restore filename must actually receive keyboard focus: {}", self.data_tools.restore_diagnostic());
                self.pending_events = vec![egui::Event::Text("dlms-learning-backup.json".into())];
                shot=Some("capture-native-restore-chooser-typed".into());
            }
            7036 => {
                let phase = ctx.data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage)))).unwrap_or(0);
                if phase < 2 { assert_eq!(self.data_tools.restore_filename(), Some("dlms-learning-backup.json"), "{}", self.data_tools.restore_diagnostic()); }
                if std::env::var_os("DLMS_SMOKE_DIAGNOSTICS").is_some() { eprintln!("before restore Check: {}", self.data_tools.restore_diagnostic()); }
                assert!(self.metrics.data_file_action.is_some() || ctx.data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage)))).unwrap_or(0) >= 2, "stage {stage} file action absent; modal {}, exported {}", self.data_tools.modal_open(), self.path.parent().unwrap().join("dlms-learning-backup.json").exists());
                if ctx.data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases",self.smoke_stage)))).unwrap_or(0)<2 && !self.reveal_file_action(ctx) { return (false,None); }
                if !self.click_smoke_control(ctx, self.metrics.data_file_action) { return (false, None); }
            }
            7037 => {
                if self.metrics.data_confirm.is_none() {
                    let started = ctx.data_mut(|d| {
                        let id = egui::Id::new("capture-native-preview-start");
                        let value = d.get_temp::<Instant>(id).unwrap_or_else(Instant::now);
                        d.insert_temp(id, value);
                        value
                    });
                    assert!(started.elapsed().as_secs() < 10, "Restore preview unavailable: {}", self.data_tools.restore_diagnostic());
                    return (false, None);
                }
                shot = Some("capture-native-restore-review-current".into());
            }
            7038 => {
                if !self.click_smoke_control(ctx, self.metrics.data_cancel) { return (false, None); }
            }
            7039 => {
                assert!(!self.data_tools.modal_open());
                let bytes = ctx.data(|d| d.get_temp::<Vec<u8>>(egui::Id::new("capture-native-before-cancel"))).unwrap();
                assert_eq!(std::fs::read(&self.path).unwrap(), bytes);
                assert!(progress::current_solved(&self.progress.attempts[&id], &self.scenarios[i]));
                shot = Some("capture-native-restore-cancel-keeps-work".into());
            }
            7040 => {
                let bytes = ctx.data(|d| d.get_temp::<Vec<u8>>(egui::Id::new("capture-native-original6"))).unwrap();
                assert_eq!(std::fs::read(self.path.with_extension(format!("before-shell-v{}.json",shell::EVALUATOR_REVISION))).unwrap(), bytes);
                println!("PASS native public evaluator{}: genuine frozen33 revision6 records, exact first backup/history and separate current credit, keyboard capture repair/Run, independent example, active-child Stop leaving no graded result, resume/retry, light150/dark200 reachable Run, native backup export/review/cancel with exact retained main bytes. Synthetic fixtures; active inspection cancellation is independently backend-tested, not a novice/physical-device/platform study.",shell::EVALUATOR_REVISION);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => unreachable!(),
        }
        (true, shot)
    }
}
