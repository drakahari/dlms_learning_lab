//! Fixed authored template activity through real native editor/terminal controls.
use super::*;
#[cfg(test)]
use crate::ui::accounts_template_fixture as fixture;
use dlms_learning_lab::accounts_template::Step;
#[derive(Clone, Debug)]
enum Op {
    Open,
    Click(&'static str),
    Select(String),
    Edit(&'static str, String),
    Include(bool),
    Command(String),
    Check(bool),
    Practice(bool),
    Save,
    Shot(String),
    Theme(bool),
    Fresh,
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    frames: usize,
    until: f64,
    done: bool,
}
fn steps(out: &mut Vec<Op>, steps: &[Step]) {
    for step in steps {
        out.push(Op::Select(step.path().into()));
        match step {
            Step::Save { contents, mode, .. } => out.extend([
                Op::Include(true),
                Op::Edit("template-mode", format!("{mode:04o}")),
                Op::Edit("template-body", contents.clone()),
                Op::Click("save-file"),
            ]),
            Step::Remove { .. } => out.extend([Op::Include(false), Op::Click("save-file")]),
        }
    }
}
fn plan(s: &Scenario) -> Vec<Op> {
    let l = s.accounts_teaching.as_ref().unwrap();
    let t = l.template_teaching.as_ref().unwrap();
    let mode_path = t
        .assignment
        .iter()
        .find_map(|step| match step {
            Step::Save { path, .. } => Some(path.clone()),
            Step::Remove { .. } => None,
        })
        .expect("Fixed template lesson has an editable retained file");
    let mut p = vec![
        Op::Open,
        Op::Shot("skeleton-introduction".into()),
        Op::Click("example"),
    ];
    steps(&mut p, &t.example);
    p.extend(l.example.iter().cloned().map(Op::Command));
    p.extend([
        Op::Practice(true),
        Op::Shot("skeleton-worked-practice".into()),
    ]);
    steps(&mut p, &t.try_steps);
    p.extend(l.try_actions.iter().cloned().map(Op::Command));
    p.extend([
        Op::Practice(false),
        Op::Shot("skeleton-copy-once-feedback".into()),
    ]);
    steps(&mut p, &t.repair);
    p.extend(l.repair.iter().cloned().map(Op::Command));
    p.extend([
        Op::Practice(true),
        Op::Click("assignment"),
        Op::Shot("skeleton-assignment".into()),
        Op::Check(false),
    ]);
    // Diagnose early creation: correcting the template alone cannot repair its old copy.
    p.extend(s.solution.iter().cloned().map(Op::Command));
    p.push(Op::Check(false));
    steps(&mut p, &t.assignment);
    p.extend([
        Op::Check(false),
        Op::Shot("skeleton-early-home-still-wrong".into()),
        Op::Click("reset"),
        Op::Click("confirm"),
        Op::Fresh,
        Op::Click("assignment"),
    ]);
    steps(&mut p, &t.assignment);
    p.extend(s.solution.iter().cloned().map(Op::Command));
    p.extend([
        Op::Check(true),
        Op::Save,
        Op::Shot("skeleton-completed".into()),
    ]);
    // A pending invalid edit loses current credit before any Save is attempted.
    p.extend([
        Op::Select(mode_path.clone()),
        Op::Edit("template-mode", "0888".into()),
        Op::Check(false),
        Op::Click("save-file"),
        Op::Check(false),
        Op::Shot("skeleton-invalid-mode-retry".into()),
    ]);
    steps(&mut p, &t.assignment);
    p.extend([Op::Check(true), Op::Save]);
    for light in [true, false] {
        p.extend([
            Op::Theme(light),
            Op::Select(mode_path.clone()),
            Op::Edit("template-mode", "0600".into()),
            Op::Shot(
                if light {
                    "skeleton-light150-edit"
                } else {
                    "skeleton-dark200-edit"
                }
                .into(),
            ),
        ]);
    }
    p
}
fn field(m: &Metrics, key: &str) -> Option<egui::Rect> {
    m.seed_fields
        .iter()
        .find(|(k, _, _)| k == key)
        .map(|(_, r, _)| *r)
}
impl LabApp {
    pub(in crate::ui) fn accounts_template_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("accounts-template-native-route");
        let gate = egui::Id::new("accounts-template-native-root");
        let frame = ctx.cumulative_frame_nr();
        if ctx.data(|d| d.get_temp::<u64>(gate)) == Some(frame) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(gate, frame));
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: plan(
                    self.scenarios
                        .iter()
                        .find(|s| s.id == "lpic1-skeleton-handover")
                        .unwrap(),
                ),
                ..Default::default()
            });
        if r.done {
            self.teaching_preview_tick(ctx);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "ACCOUNT_TEMPLATE_NATIVE_COMPLETE: actual editors/Save, commands, copy-once failure, separate practice, pending invalidation, reset, disk save/load, light150/dark200"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        let now = ctx.input(|i| i.time);
        if now < r.until {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        self.smoke_stage = 150000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("ACCOUNT_TEMPLATE_STEP {} {op:?}", r.at);
        }
        r.frames += 1;
        assert!(r.frames < 10000, "Template activity timed out: {op:?}");
        let done = match op {
            Op::Open => {
                let i = self
                    .scenarios
                    .iter()
                    .position(|s| s.id == "lpic1-skeleton-handover")
                    .unwrap();
                self.launch(i);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                true
            }
            Op::Click(k) => {
                let target = match k {
                    "example" => self.metrics.example_action,
                    "assignment" => self
                        .metrics
                        .assignment_action
                        .or_else(|| self.metrics.lesson_tabs.get(2).copied()),
                    "save-file" => self.metrics.editor_save,
                    "reset" => self.metrics.reset,
                    "confirm" => self.metrics.confirm_reset,
                    _ => panic!("Unknown fixed template control"),
                };
                self.click_visible_smoke_control(ctx, target)
            }
            Op::Select(path) => {
                let a = &self.progress.attempts[&self.scenarios[self.selected.unwrap()].id];
                let w = a.workspace.as_ref().unwrap();
                let selected =
                    if self.accounts_view(self.selected.unwrap()) == teaching::View::Example {
                        &w.accounts_learning
                            .as_ref()
                            .unwrap()
                            .template_drafts
                            .as_ref()
                            .unwrap()
                            .selected
                    } else {
                        &w.accounts_template_drafts.as_ref().unwrap().selected
                    };
                if *selected == path {
                    true
                } else if r.phase == 0 {
                    if self
                        .click_visible_smoke_control(ctx, field(&self.metrics, "template-selector"))
                    {
                        r.phase = 1;
                    }
                    false
                } else {
                    self.mac_popup_choice(
                        ctx,
                        field(&self.metrics, &format!("template-choice:{path}")),
                        self.metrics.accounts_template_popup,
                    );
                    false
                }
            }
            Op::Include(want) => {
                let i = self.selected.unwrap();
                let w = self.progress.attempts[&self.scenarios[i].id]
                    .workspace
                    .as_ref()
                    .unwrap();
                let d = if self.accounts_view(i) == teaching::View::Example {
                    w.accounts_learning
                        .as_ref()
                        .unwrap()
                        .template_drafts
                        .as_ref()
                        .unwrap()
                } else {
                    w.accounts_template_drafts.as_ref().unwrap()
                };
                if d.leaves[&d.selected].included == want {
                    true
                } else {
                    self.click_visible_smoke_control(ctx, field(&self.metrics, "template-include"));
                    false
                }
            }
            Op::Edit(k, text) => {
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, field(&self.metrics, k)) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    self.case_lesson_type(&text);
                    r.phase = 2;
                    false
                } else {
                    let i = self.selected.unwrap();
                    let w = self.progress.attempts[&self.scenarios[i].id]
                        .workspace
                        .as_ref()
                        .unwrap();
                    let d = if self.accounts_view(i) == teaching::View::Example {
                        w.accounts_learning
                            .as_ref()
                            .unwrap()
                            .template_drafts
                            .as_ref()
                            .unwrap()
                    } else {
                        w.accounts_template_drafts.as_ref().unwrap()
                    };
                    let actual = if k == "template-mode" {
                        &d.leaves[&d.selected].mode
                    } else {
                        &d.leaves[&d.selected].contents
                    };
                    assert_eq!(
                        actual, &text,
                        "actual native input must match before continuing"
                    );
                    true
                }
            }
            Op::Command(text) => {
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, self.metrics.terminal) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    self.case_lesson_type(&text);
                    r.phase = 2;
                    false
                } else if r.phase == 2 {
                    let i = self.selected.unwrap();
                    let a = &self.progress.attempts[&self.scenarios[i].id];
                    let actual = if self.accounts_view(i) == teaching::View::Example {
                        &a.workspace
                            .as_ref()
                            .unwrap()
                            .accounts_learning
                            .as_ref()
                            .unwrap()
                            .command
                    } else {
                        &self.input
                    };
                    assert_eq!(actual, &text);
                    self.pending_events = [true, false]
                        .into_iter()
                        .map(|pressed| egui::Event::Key {
                            key: egui::Key::Enter,
                            physical_key: None,
                            pressed,
                            repeat: false,
                            modifiers: Default::default(),
                        })
                        .collect();
                    r.phase = 3;
                    false
                } else {
                    let i = self.selected.unwrap();
                    let a = &self.progress.attempts[&self.scenarios[i].id];
                    let transcript = if self.accounts_view(i) == teaching::View::Example {
                        &a.workspace
                            .as_ref()
                            .unwrap()
                            .accounts_learning
                            .as_ref()
                            .unwrap()
                            .transcript
                    } else {
                        &a.transcript
                    };
                    assert!(
                        transcript
                            .iter()
                            .rev()
                            .take(2)
                            .any(|line| line.contains(&text)),
                        "actual terminal command missing: {text}"
                    );
                    true
                }
            }
            Op::Check(want) | Op::Practice(want) => {
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, self.metrics.check) {
                        r.phase = 1;
                    }
                    false
                } else {
                    let s = &self.scenarios[self.selected.unwrap()];
                    let a = &self.progress.attempts[&s.id];
                    if self.accounts_view(self.selected.unwrap()) == teaching::View::Example {
                        let l = a
                            .workspace
                            .as_ref()
                            .unwrap()
                            .accounts_learning
                            .as_ref()
                            .unwrap();
                        assert_eq!(
                            l.checks(s.accounts_teaching.as_ref().unwrap())
                                .iter()
                                .all(|c| c.0),
                            want
                        );
                        assert_eq!(a.state, s.initial);
                        assert!(!progress::current_solved(a, s));
                    } else {
                        assert_eq!(progress::current_solved(a, s), want);
                        assert_eq!(self.metrics.completion, want);
                    }
                    true
                }
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none(), "{:?}", self.storage_error);
                let p = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(p).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Shot(name) => {
                if r.phase == 0 {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name,
                    )));
                    r.phase = 1;
                    false
                } else {
                    !self.capture_pending
                }
            }
            Op::Fresh => {
                let s = &self.scenarios[self.selected.unwrap()];
                assert_eq!(self.progress.attempts[&s.id].state, s.initial);
                true
            }
            Op::Theme(light) => {
                self.scale = if light { 1.5 } else { 2. };
                ctx.set_zoom_factor(self.scale);
                self.theme_id = if light {
                    theme::inventory()
                        .iter()
                        .find(|t| t.light)
                        .unwrap()
                        .id
                        .clone()
                } else {
                    theme::Preferences::default().theme
                };
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1100., 850.)));
                self.lesson_scroll_reset = true;
                true
            }
        };
        if self
            .pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        {
            r.until = now + 0.15;
        }
        if done {
            r.at += 1;
            r.phase = 0;
            r.frames = 0;
            self.started = Instant::now();
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn account_template_native_plan_keeps_actual_input_wrong_order_reset_and_save() {
        let s = fixture::scenario();
        let p = plan(&s);
        assert!(matches!(p.first(), Some(Op::Open)));
        assert_eq!(p.iter().filter(|o| matches!(o, Op::Practice(_))).count(), 3);
        assert!(p.iter().filter(|o| matches!(o, Op::Check(false))).count() >= 4);
        assert_eq!(p.iter().filter(|o| matches!(o, Op::Fresh)).count(), 1);
        assert_eq!(p.iter().filter(|o| matches!(o, Op::Save)).count(), 2);
        assert!(
            p.iter()
                .any(|o| matches!(o, Op::Edit("template-mode",v) if v=="0888"))
        );
        for c in &s.solution {
            assert_eq!(
                p.iter()
                    .filter(|o| matches!(o, Op::Command(v) if v==c))
                    .count(),
                2
            );
        }
        // A removed final row must not become the target of a later mode edit.
        let mut with_remove = s.clone();
        with_remove
            .accounts_teaching
            .as_mut()
            .unwrap()
            .template_teaching
            .as_mut()
            .unwrap()
            .assignment
            .push(Step::Remove {
                path: "/etc/skel-field/optional-note.txt".into(),
            });
        let p = plan(&with_remove);
        for (i, o) in p.iter().enumerate() {
            if matches!(o,Op::Edit("template-mode",v) if v=="0888"||v=="0600") {
                assert!(matches!(&p[i-1],Op::Select(path) if path==fixture::ASSIGNMENT_PATH));
            }
        }
    }
    #[test]
    fn account_template_capture_emits_real_screenshot_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a.scenarios.len();
        a.scenarios.push(fixture::scenario());
        a.launch(i);
        a.set_accounts_view(i, teaching::View::Example);
        let raw = || egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1500., 7000.),
            )),
            ..Default::default()
        };
        let _ = ctx.run(raw(), |ctx| a.render(ctx));
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("accounts-template-native-route"),
                Route {
                    ops: vec![Op::Shot("skeleton-capture-regression".into())],
                    ..Default::default()
                },
            )
        });
        let output = ctx.run(raw(), |ctx| {
            a.render(ctx);
            a.accounts_template_tick(ctx);
        });
        assert!(a.capture_pending);
        assert!(output.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
}
