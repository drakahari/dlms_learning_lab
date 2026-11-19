//! Fixed application-specific recovery work through real native terminal events.
use super::*;
const ID: &str = "lpic2-backup-directory-selection";
#[derive(Clone, Debug)]
enum Op {
    Open,
    Tab(teaching::View),
    Click(String),
    Command(String, bool),
    Checked(bool, bool),
    Inert,
    Save,
    Fresh,
    Shot(String),
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    frames: usize,
    before_count: usize,
    done: bool,
}
fn check(p: &mut Vec<Op>, pass: bool, practice: bool) {
    p.extend([Op::Click("check".into()), Op::Checked(pass, practice)]);
}
fn command(p: &mut Vec<Op>, text: &str, practice: bool) {
    p.push(Op::Command(text.into(), practice));
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    let s = labs.iter().find(|s| s.id == ID).unwrap();
    let lesson = s.file_backup_teaching.as_ref().unwrap();
    let mut p = vec![Op::Open, Op::Tab(teaching::View::Example)];
    check(&mut p, false, true);
    p.extend([Op::Click("insert".into()), Op::Inert]);
    for c in &lesson.example_commands {
        command(&mut p, &c.command, true);
    }
    check(&mut p, true, true);
    p.push(Op::Shot("backup-selection-practice-passed".into()));
    for c in &lesson.try_change.commands {
        command(&mut p, &c.command, true);
    }
    check(&mut p, false, true);
    p.push(Op::Shot("backup-selection-practice-extra-feedback".into()));
    for c in &lesson.repair {
        command(&mut p, &c.command, true);
    }
    check(&mut p, true, true);
    p.extend([Op::Save, Op::Tab(teaching::View::Assignment)]);
    check(&mut p, false, false);
    for c in [
        "tar -cf /backups/orchard-config.tar -C /var/cache orchard",
        "tar -cf /backups/orchard-data.tar -C /run orchard",
        "tar -xf /backups/orchard-config.tar -C /recovery/etc",
        "tar -xf /backups/orchard-data.tar -C /recovery/lib",
    ] {
        command(&mut p, c, false);
    }
    check(&mut p, false, false);
    p.push(Op::Shot("backup-selection-wrong-source-feedback".into()));
    for c in &s.solution {
        command(&mut p, c, false);
    }
    check(&mut p, true, false);
    p.extend([Op::Save, Op::Shot("backup-selection-passed".into())]);
    command(&mut p, "chmod 777 /recovery/etc/orchard/payload.txt", false);
    check(&mut p, false, false);
    p.push(Op::Shot("backup-selection-damage-feedback".into()));
    command(
        &mut p,
        "tar -xf /backups/orchard-config.tar -C /recovery/etc",
        false,
    );
    check(&mut p, true, false);
    p.extend([
        Op::Save,
        Op::Click("reset-header".into()),
        Op::Click("confirm-reset".into()),
        Op::Fresh,
    ]);
    p
}
impl LabApp {
    pub(in crate::ui) fn backup_selection_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("backup-selection-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: route(&self.scenarios),
                ..Default::default()
            });
        if r.done {
            self.teaching_preview_tick(ctx);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "BACKUP_SELECTION_NATIVE_FLOW_COMPLETE: separate example/inert insert/extra delivery/repair; assignment/wrong source/solution/damage/retry/save-read/reset; no host commands"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 150000 + r.at;
        let op = r.ops[r.at].clone();
        r.frames += 1;
        if r.frames == 1 {
            println!("BACKUP_SELECTION_NATIVE_STEP {} {:?}", r.at, op);
        }
        assert!(
            r.frames < 1200,
            "Backup selection step{} phase{} {:?}",
            r.at,
            r.phase,
            op
        );
        let done = match op {
            Op::Open => {
                let i = self.scenarios.iter().position(|s| s.id == ID).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                true
            }
            Op::Tab(v) => {
                let rect = self
                    .metrics
                    .backup
                    .tabs
                    .iter()
                    .find(|t| t.0 == v)
                    .map(|t| t.1);
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Click(name) => {
                let rect = match name.as_str() {
                    "reset-header" => self.metrics.reset,
                    "confirm-reset" => self.metrics.confirm_reset,
                    "insert" => self.metrics.backup.inserts.first().map(|r| r.1),
                    _ => self
                        .metrics
                        .backup
                        .controls
                        .iter()
                        .find(|c| c.0 == name)
                        .map(|c| c.1),
                };
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Command(text, practice) => {
                if r.phase == 0 {
                    let a = &self.progress.attempts[ID];
                    r.before_count = if practice {
                        a.workspace
                            .as_ref()
                            .unwrap()
                            .file_backup_learning
                            .as_ref()
                            .unwrap()
                            .transcript
                            .len()
                    } else {
                        a.transcript.len()
                    };
                    if self.click_visible_smoke_control(
                        ctx,
                        self.metrics.backup.terminal.input.map(|f| f.0),
                    ) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    let modifiers = egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    };
                    for pressed in [true, false] {
                        self.pending_events.push(egui::Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed,
                            repeat: false,
                            modifiers,
                        });
                    }
                    self.pending_events.push(egui::Event::Paste(text.clone()));
                    r.phase = 2;
                    false
                } else if r.phase == 2 {
                    let a = &self.progress.attempts[ID];
                    let w = a.workspace.as_ref().unwrap();
                    let draft = if practice {
                        &w.file_backup_learning.as_ref().unwrap().command
                    } else {
                        &w.command
                    };
                    assert_eq!(draft, &text, "actual typed command");
                    if self.click_visible_smoke_control(ctx, self.metrics.backup.terminal.run) {
                        r.phase = 3;
                    }
                    false
                } else {
                    let a = &self.progress.attempts[ID];
                    let history = if practice {
                        &a.workspace
                            .as_ref()
                            .unwrap()
                            .file_backup_learning
                            .as_ref()
                            .unwrap()
                            .transcript
                    } else {
                        &a.transcript
                    };
                    if history.len() > r.before_count {
                        assert!(history.last().unwrap().contains(&text));
                        true
                    } else {
                        false
                    }
                }
            }
            Op::Inert => {
                let s = &self.scenarios[self.selected.unwrap()];
                let a = &self.progress.attempts[ID];
                assert_eq!(a.state, s.initial);
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .file_backup_learning
                    .as_ref()
                    .unwrap();
                assert!(l.transcript.is_empty());
                assert!(!l.command.is_empty());
                assert!(!progress::current_solved(a, s));
                true
            }
            Op::Checked(expected, practice) => {
                if self.backup_presentation.checking() || self.metrics.backup.rows.len() != 13 {
                    false
                } else {
                    let s = &self.scenarios[self.selected.unwrap()];
                    let a = &self.progress.attempts[ID];
                    let actual = self.metrics.backup.rows.iter().all(|r| r.1);
                    assert_eq!(actual, expected, "native objective rows");
                    if practice {
                        assert_eq!(a.state, s.initial);
                        assert!(!progress::current_solved(a, s));
                    } else {
                        assert_eq!(progress::current_solved(a, s), expected);
                        assert_eq!(self.metrics.backup.complete, expected);
                    }
                    true
                }
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none(), "{:?}", self.storage_error);
                let got = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(got).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Fresh => {
                let s = &self.scenarios[self.selected.unwrap()];
                let a = &self.progress.attempts[ID];
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, s));
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .file_backup_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(
                    l.state,
                    s.file_backup_teaching.as_ref().unwrap().definition.initial
                );
                true
            }
            Op::Shot(name) => {
                if r.phase == 0 {
                    let rect = self.metrics.backup.rows.first().map(|r| r.2);
                    if rect.is_some_and(|r| self.reveal_smoke_control(ctx, r)) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name,
                    )));
                    r.phase = 2;
                    false
                } else {
                    !self.capture_pending
                }
            }
        };
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
    fn backup_selection_capture_phase_does_not_reenter_context() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
        a.launch(i);
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("backup-selection-native-route"),
                Route {
                    ops: vec![Op::Shot("fixed-backup-selection".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| a.backup_selection_tick(ctx));
        assert!(a.capture_pending);
    }
}
