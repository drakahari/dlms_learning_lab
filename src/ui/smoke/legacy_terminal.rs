//! Fixed real-input acceptance for the three original access/log assignments.
//! Author reference commands are trusted test inputs, never State or grading authority.
use super::*;
use serde_json::Value;
const START: usize = 28000;
const END: usize = 29000;
const IDS: [&str; 3] = ["lpic1-permissions", "lpic1-traversal", "lpic1-logs"];
#[derive(Clone, Copy, Debug, PartialEq)]
enum Target {
    Try,
    Assignment,
    Example,
    Input,
    Run,
    Insert,
    Check,
    Reset,
    Confirm,
    Cancel,
    Challenge,
    Learn,
}
#[derive(Clone, Debug, PartialEq)]
enum Step {
    Library,
    Filter(usize),
    Start,
    Wait,
    Click(Target),
    Text(String),
    BeforeInsert,
    InsertOnly,
    Ran(String, bool),
    Output(String, bool),
    Rows(bool, Vec<bool>),
    Remember,
    Resume,
    Save,
    Snapshot,
    CancelExact,
    ResetExact,
    Theme(bool, f32, bool),
    Geometry(f32, bool),
    Mode(bool),
    Shot(String, bool),
    Finish,
}
fn commands(s: &Scenario, group: usize) -> Vec<String> {
    if let Some(l) = &s.legacy_access_teaching {
        match group {
            0 => l.reference.clone(),
            1 => l.try_commands.clone(),
            _ => l.repair_commands.clone(),
        }
    } else {
        let l = s.legacy_logs_teaching.as_ref().unwrap();
        match group {
            0 => l.reference.clone(),
            1 => l.try_commands.clone(),
            _ => l.repair_commands.clone(),
        }
    }
}
fn command(p: &mut Vec<Step>, text: &str, practice: bool) {
    p.extend([
        Step::Click(Target::Input),
        Step::Text(text.into()),
        Step::Click(Target::Run),
        Step::Ran(text.into(), practice),
    ]);
}
fn sequence(p: &mut Vec<Step>, commands: &[String], practice: bool) {
    for c in commands {
        command(p, c, practice);
    }
}
fn check(p: &mut Vec<Step>, practice: bool, rows: &[bool], shot: String) {
    p.extend([
        Step::Click(Target::Check),
        Step::Wait,
        Step::Rows(practice, rows.to_vec()),
        Step::Shot(shot, true),
    ]);
}
fn scale(p: &mut Vec<Step>, light: bool, value: f32, wide: bool) {
    p.extend([Step::Theme(light, value, wide), Step::Geometry(value, wide)]);
}
fn plan(scenarios: &[Scenario]) -> Vec<Step> {
    let mut p = Vec::new();
    for (n, id) in IDS.iter().enumerate() {
        let s = scenarios
            .iter()
            .find(|s| s.id == *id)
            .expect("three normal IDs, no fixture append");
        assert_eq!(s.legacy_access_teaching.is_some(), n < 2);
        assert_eq!(s.legacy_logs_teaching.is_some(), n == 2);
        let good = if n == 2 { vec![true] } else { vec![true, true] };
        let bad = if n == 2 {
            vec![false]
        } else {
            vec![false, true]
        };
        let shot = |suffix: &str| format!("legacy-terminal-{n}-{suffix}");
        p.extend([
            Step::Library,
            Step::Filter(n),
            Step::Wait,
            Step::Start,
            Step::Wait,
            Step::Shot(shot("introduction"), false),
            Step::Click(Target::Try),
            Step::Wait,
            Step::BeforeInsert,
            Step::Click(Target::Insert),
            Step::InsertOnly,
        ]);
        // One real Insert proof; the rest are typed, not repeated popup traversal.
        let first = if let Some(l) = &s.legacy_access_teaching {
            &l.commands[0].command
        } else {
            &s.legacy_logs_teaching.as_ref().unwrap().commands[0].command
        };
        p.extend([Step::Click(Target::Run), Step::Ran(first.clone(), true)]);
        sequence(&mut p, &commands(s, 0), true);
        check(&mut p, true, &good, shot("example-complete-ungraded"));
        sequence(&mut p, &commands(s, 1), true);
        check(&mut p, true, &bad, shot("example-experiment"));
        sequence(&mut p, &commands(s, 2), true);
        check(&mut p, true, &good, shot("example-repaired"));
        p.extend([
            Step::Click(Target::Assignment),
            Step::Wait,
            Step::Shot(shot("assignment"), false),
        ]);
        check(&mut p, false, &bad, shot("initial-feedback"));
        let partial = match n {
            0 => "chmod 640 /srv/report.txt",
            1 => "usermod -aG editors blair",
            _ => "grep timeout /var/log/app.log",
        };
        command(&mut p, partial, false);
        check(&mut p, false, &bad, shot("partial-feedback"));
        sequence(&mut p, &s.solution, false);
        check(&mut p, false, &good, shot("reference-complete"));
        if n == 0 {
            scale(&mut p, true, 1.5, false);
        } else if n == 1 {
            scale(&mut p, false, 2., false);
        } else {
            scale(&mut p, true, 1.5, true);
        }
        let alternative: Vec<String> = match n {
            0 => vec![
                "chown alex:editors /srv/report.txt",
                "chmod 600 /srv/report.txt",
            ],
            1 => vec![
                "chown blair:editors /srv/project/notes.txt",
                "chmod 600 /srv/project/notes.txt",
                "chown blair:editors /srv/project",
                "chmod 700 /srv/project",
            ],
            _ => vec!["grep ERROR /var/log/app.log"],
        }
        .into_iter()
        .map(str::to_owned)
        .collect();
        sequence(&mut p, &alternative, false);
        check(&mut p, false, &good, shot("typed-alternate-complete"));
        if n == 2 {
            scale(&mut p, false, 2., true);
        }
        let (broken, repair) = match n {
            0 => ("chmod 400 /srv/report.txt", "chmod 600 /srv/report.txt"),
            1 => ("chmod 600 /srv/project", "chmod 700 /srv/project"),
            _ => ("cat /var/log/app.log", "grep ERROR /var/log/app.log"),
        };
        command(&mut p, broken, false);
        p.extend([
            Step::Rows(false, bad.clone()),
            Step::Shot(shot("fresh-invalidated"), true),
        ]);
        command(&mut p, repair, false);
        check(&mut p, false, &good, shot("typed-repair-complete"));
        if n == 2 {
            scale(&mut p, false, 2., false);
            command(&mut p, repair, false);
            check(&mut p, false, &good, shot("dark200-narrow-typed-complete"));
        }
        let query = match n {
            0 => "id alex",
            1 => "namei /srv/project/notes.txt",
            _ => "find /var/log -name app.log",
        };
        command(&mut p, query, false);
        p.push(Step::Rows(false, good.clone()));
        p.push(Step::Output(
            match n {
                0 => "uid=",
                1 => "755 root:root /",
                _ => "/var/log/app.log",
            }
            .into(),
            false,
        ));
        p.extend([
            Step::Click(Target::Challenge),
            Step::Wait,
            Step::Mode(true),
            Step::Shot(shot("challenge"), false),
            Step::Click(Target::Learn),
            Step::Wait,
            Step::Mode(false),
        ]);
        p.extend([
            Step::Click(Target::Input),
            Step::Text("unfinished assignment '".into()),
            Step::Click(Target::Example),
            Step::Wait,
            Step::Click(Target::Input),
            Step::Text("unfinished practice '".into()),
            Step::Remember,
            Step::Save,
            Step::Library,
            Step::Filter(n),
            Step::Wait,
            Step::Start,
            Step::Wait,
            Step::Resume,
            Step::Shot(shot("separate-commands-resumed"), false),
            Step::Click(Target::Assignment),
            Step::Wait,
            Step::Snapshot,
            Step::Click(Target::Reset),
            Step::Click(Target::Cancel),
            Step::CancelExact,
            Step::Click(Target::Reset),
            Step::Click(Target::Confirm),
            Step::Wait,
            Step::ResetExact,
            Step::Click(Target::Assignment),
            Step::Wait,
        ]);
        sequence(&mut p, &s.solution, false);
        check(&mut p, false, &good, shot("reset-repaired"));
        p.push(Step::Save);
    }
    p.push(Step::Finish);
    assert!(p.len() < END - START);
    assert!(p.iter().all(|s| !matches!(s,Step::Text(t)if t.len()>4096)));
    p
}
fn hold_close(ctx: &egui::Context) -> (bool, Option<String>) {
    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    (false, None)
}
impl LabApp {
    fn legacy_native_practice(&self, i: usize) -> Value {
        let w = self.progress.attempts[&self.scenarios[i].id]
            .workspace
            .as_ref()
            .unwrap();
        if let Some(l) = &w.legacy_access_learning {
            serde_json::to_value(l).unwrap()
        } else {
            serde_json::to_value(w.legacy_logs_learning.as_ref().unwrap()).unwrap()
        }
    }
    fn legacy_native_rows(&self, i: usize) -> Vec<bool> {
        let s = &self.scenarios[i];
        let w = self.progress.attempts[&s.id].workspace.as_ref().unwrap();
        let checks = if let Some(l) = &w.legacy_access_learning {
            l.checks(s.legacy_access_teaching.as_ref().unwrap())
        } else {
            w.legacy_logs_learning
                .as_ref()
                .unwrap()
                .checks(s.legacy_logs_teaching.as_ref().unwrap())
        };
        checks.iter().map(|r| r.0).collect()
    }
    fn legacy_native_target(&self, t: Target) -> Option<egui::Rect> {
        match t {
            Target::Try => self.metrics.example_action,
            Target::Assignment => self
                .metrics
                .assignment_action
                .or_else(|| self.metrics.lesson_tabs.get(2).copied()),
            Target::Example => self.metrics.lesson_tabs.get(1).copied(),
            Target::Input => self.metrics.terminal,
            Target::Run => self.metrics.terminal_run,
            Target::Insert => self.metrics.insert,
            Target::Check => self.metrics.check,
            Target::Reset => self.metrics.reset,
            Target::Confirm => self.metrics.confirm_reset,
            Target::Cancel => self.metrics.reset_cancel,
            Target::Challenge => self.metrics.mode_challenge,
            Target::Learn => self.metrics.mode_learn,
        }
    }
    fn legacy_native_click(&mut self, ctx: &egui::Context, t: Target) -> bool {
        self.click_visible_smoke_control(ctx, self.legacy_native_target(t))
    }
    fn legacy_native_start(&mut self, ctx: &egui::Context, id: &str) -> bool {
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        if ctx
            .data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))))
            .unwrap_or(0)
            >= 2
        {
            let done = self.click_visible_smoke_control(ctx, None);
            assert_eq!(self.selected, Some(i));
            return done;
        }
        assert!(self.selected.is_none());
        let visible: Vec<_> = self
            .scenarios
            .iter()
            .filter(|s| {
                catalog::matches(
                    s,
                    &self.filter,
                    self.progress.attempts.contains_key(&s.id),
                    self.progress
                        .attempts
                        .get(&s.id)
                        .is_some_and(|a| progress::current_solved(a, s)),
                    self.progress.favorites.contains(&s.id),
                )
            })
            .collect();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].id, id);
        assert_eq!(self.metrics.card_actions.len(), 1);
        self.click_visible_smoke_control(ctx, self.metrics.card_actions.first().map(|r| r.0))
    }
    fn legacy_native_settled(&self, ctx: &egui::Context, scale: f32, wide: bool) -> bool {
        let r = ctx.content_rect();
        let zoom = ctx.zoom_factor();
        let ppp = ctx.pixels_per_point();
        let native_width = r.width() * zoom;
        assert!(r.is_finite() && ppp.is_finite() && ppp > 0.);
        let (tries, stable) = ctx.data_mut(|d| {
            let key = egui::Id::new(("legacy-native-settle", self.smoke_stage));
            let (tries, old, stable) =
                d.get_temp::<(u8, egui::Rect, u8)>(key)
                    .unwrap_or((0, egui::Rect::NOTHING, 0));
            let width = if wide {
                native_width >= 1600.
            } else {
                native_width <= 1500.
            };
            let stable = if old == r && (zoom - scale).abs() < 0.01 && width {
                stable + 1
            } else {
                0
            };
            d.insert_temp(key, (tries + 1, r, stable));
            (tries + 1, stable)
        });
        assert!(
            tries <= 30,
            "Legacy native viewport/zoom did not settle: {r:?} {zoom} wide{wide}"
        );
        if stable < 2 {
            ctx.request_repaint();
        }
        stable >= 2
    }
    pub(super) fn legacy_terminal_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert!((START..END).contains(&stage));
        let p = plan(&self.scenarios);
        let offset = stage - START;
        let step = p.get(offset).expect("fixed legacy terminal stage");
        let n = p[..=offset]
            .iter()
            .rev()
            .find_map(|s| {
                if let Step::Filter(n) = s {
                    Some(*n)
                } else {
                    None
                }
            })
            .unwrap_or(0);
        let id = IDS[n];
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        if std::env::var_os("DLMS_SMOKE_DIAGNOSTICS").is_some() {
            eprintln!("legacy-terminal-native {stage} {id} {step:?}");
        }
        match step {
            Step::Library => {
                let done = if ctx
                    .data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", stage))))
                    .unwrap_or(0)
                    >= 2
                {
                    self.click_visible_smoke_control(ctx, None)
                } else if self.selected.is_some() {
                    self.click_visible_smoke_control(ctx, self.metrics.back)
                } else {
                    true
                };
                if !done {
                    return (false, None);
                }
                assert!(self.selected.is_none());
            }
            Step::Filter(_) => {
                assert!(self.selected.is_none());
                self.filter = Filter::default();
                self.filter.search = self.scenarios[i].title.clone();
                self.filter.max_minutes = 120;
                self.page = 0;
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1268., 632.)));
            }
            Step::Start => {
                if !self.legacy_native_start(ctx, id) {
                    return (false, None);
                }
            }
            Step::Wait => {}
            Step::Click(t) => {
                if !self.legacy_native_click(ctx, *t) {
                    return (false, None);
                }
                assert_eq!(self.selected, Some(i));
                match t {
                    Target::Try | Target::Example => {
                        assert_eq!(self.legacy_view(i), teaching::View::Example)
                    }
                    Target::Assignment => {
                        assert_eq!(self.legacy_view(i), teaching::View::Assignment)
                    }
                    _ => {}
                }
            }
            Step::Text(text) => {
                assert!(self.metrics.focus.is_some());
                assert_eq!(
                    ctx.memory(|m| m.focused()),
                    self.metrics.focus,
                    "actual terminal focus before typing"
                );
                self.pending_events = vec![
                    egui::Event::Key {
                        key: egui::Key::A,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers {
                            ctrl: true,
                            command: true,
                            ..Default::default()
                        },
                    },
                    egui::Event::Text(text.clone()),
                ];
            }
            Step::BeforeInsert => {
                let a = &self.progress.attempts[id];
                let value = (
                    self.legacy_native_practice(i),
                    serde_json::to_value(&a.state).unwrap(),
                    a.transcript.clone(),
                    a.workspace.as_ref().unwrap().command.clone(),
                );
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("legacy-native-insert"), value));
            }
            Step::InsertOnly => {
                let before = ctx
                    .data(|d| {
                        d.get_temp::<(Value, Value, Vec<String>, String)>(egui::Id::new(
                            "legacy-native-insert",
                        ))
                    })
                    .unwrap();
                let current = self.legacy_native_practice(i);
                let s = &self.scenarios[i];
                let first = if let Some(l) = &s.legacy_access_teaching {
                    &l.commands[0].command
                } else {
                    &s.legacy_logs_teaching.as_ref().unwrap().commands[0].command
                };
                assert_eq!(current["selected_action"], 0);
                assert_eq!(current["command"], first.as_str());
                assert_eq!(current["current"], before.0["current"]);
                assert_eq!(current["transcript"], before.0["transcript"]);
                let a = &self.progress.attempts[id];
                assert_eq!(serde_json::to_value(&a.state).unwrap(), before.1);
                assert_eq!(a.transcript, before.2);
                assert_eq!(a.workspace.as_ref().unwrap().command, before.3);
            }
            Step::Ran(command, practice) => {
                let a = &self.progress.attempts[id];
                let (input, transcript) = if *practice {
                    let v = self.legacy_native_practice(i);
                    assert_eq!(a.state, self.scenarios[i].initial);
                    assert!(!progress::current_solved(a, &self.scenarios[i]));
                    (
                        v["command"].as_str().unwrap().to_owned(),
                        v["transcript"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|x| x.as_str().unwrap().to_owned())
                            .collect::<Vec<_>>(),
                    )
                } else {
                    assert!(a.workspace.as_ref().unwrap().command.is_empty());
                    (self.input.clone(), a.transcript.clone())
                };
                assert!(input.is_empty());
                let last = transcript.last().expect("actual Run appended output");
                assert!(last.starts_with(&format!(
                    "{} # {command}",
                    if *practice { "practice" } else { "lab" }
                )));
                assert!(!last.contains("Command failed:"));
            }
            Step::Output(text, practice) => {
                let lines = if *practice {
                    self.legacy_native_practice(i)["transcript"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|x| x.as_str().unwrap().to_owned())
                        .collect::<Vec<_>>()
                } else {
                    self.progress.attempts[id].transcript.clone()
                };
                let (_, output) = lines
                    .last()
                    .unwrap()
                    .split_once('\n')
                    .expect("actual command returned output");
                assert!(output.contains(text));
            }
            Step::Rows(practice, expected) => {
                let a = &self.progress.attempts[id];
                let s = &self.scenarios[i];
                let rows = if *practice {
                    assert!(!progress::current_solved(a, s));
                    let v = self.legacy_native_practice(i);
                    assert_eq!(v["checked"], true);
                    assert_eq!(a.state, s.initial);
                    assert_eq!(
                        self.metrics
                            .seed_feedback
                            .iter()
                            .map(|r| r.0)
                            .collect::<Vec<_>>(),
                        *expected
                    );
                    self.legacy_native_rows(i)
                } else {
                    let pass = expected.iter().all(|v| *v);
                    assert_eq!(progress::current_solved(a, s), pass);
                    assert_eq!(self.metrics.completion, pass);
                    assert!(self.metrics.legacy_feedback.is_some());
                    engine::checks(&a.state, &s.goal)
                        .iter()
                        .map(|r| r.0)
                        .collect()
                };
                assert_eq!(rows, *expected);
            }
            Step::Remember => {
                let a = &self.progress.attempts[id];
                assert_eq!(
                    a.workspace.as_ref().unwrap().command,
                    "unfinished assignment '"
                );
                let v = self.legacy_native_practice(i);
                assert_eq!(v["command"], "unfinished practice '");
                assert!(
                    progress::current_solved(a, &self.scenarios[i]),
                    "unsubmitted terminal text is not applied configuration"
                );
                let v = serde_json::to_value(a).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("legacy-native-resume"), v));
            }
            Step::Resume => {
                let mut before = ctx
                    .data(|d| d.get_temp::<Value>(egui::Id::new("legacy-native-resume")))
                    .unwrap();
                before["visited"] = serde_json::json!(self.progress.clock);
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[id]).unwrap(),
                    before
                );
                assert_eq!(self.legacy_view(i), teaching::View::Example);
            }
            Step::Save => {
                self.save();
                assert!(self.storage_error.is_none());
                let bytes = progress::read_bounded(&self.path, 16_000_000).unwrap();
                let raw: Value = serde_json::from_slice(&bytes).unwrap();
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                let portable = serde_json::to_value(&self.progress).unwrap();
                assert_eq!(serde_json::to_value(&loaded).unwrap(), portable);
                let mut exact = portable;
                exact["migration_receipts"] =
                    serde_json::to_value(loaded.migration_receipts).unwrap();
                assert_eq!(raw, exact);
                assert_eq!(
                    progress::read_bounded(&self.path, 16_000_000).unwrap(),
                    bytes
                );
            }
            Step::Snapshot => {
                let v = serde_json::to_value(&self.progress).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("legacy-native-reset"), v));
            }
            Step::CancelExact | Step::ResetExact => {
                let before = ctx
                    .data(|d| d.get_temp::<Value>(egui::Id::new("legacy-native-reset")))
                    .unwrap();
                assert!(!self.reset_pending);
                let current = serde_json::to_value(&self.progress).unwrap();
                assert_eq!(current["favorites"], before["favorites"]);
                for (key, value) in before["attempts"].as_object().unwrap() {
                    if key != id {
                        assert_eq!(&current["attempts"][key], value);
                    }
                }
                let a = &self.progress.attempts[id];
                if matches!(step, Step::CancelExact) {
                    assert_eq!(current["attempts"][id], before["attempts"][id]);
                } else {
                    assert_eq!(a.state, self.scenarios[i].initial);
                    assert!(
                        a.transcript.is_empty() && a.workspace.as_ref().unwrap().command.is_empty()
                    );
                    assert!(!progress::current_solved(a, &self.scenarios[i]));
                    assert_eq!(self.legacy_view(i), teaching::View::Introduction);
                    let v = self.legacy_native_practice(i);
                    let fixture = if let Some(l) = &self.scenarios[i].legacy_access_teaching {
                        serde_json::to_value(&l.fixture).unwrap()
                    } else {
                        serde_json::to_value(
                            &self.scenarios[i]
                                .legacy_logs_teaching
                                .as_ref()
                                .unwrap()
                                .fixture,
                        )
                        .unwrap()
                    };
                    assert_eq!(v["current"], fixture);
                    assert_eq!(v["command"], "");
                    assert_eq!(v["transcript"], serde_json::json!([]));
                }
            }
            Step::Theme(light, scale, wide) => {
                self.scale = *scale;
                self.theme_id = if *light { "dlms-light" } else { "dlms-dark" }.into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(*wide));
                if !wide {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                        1268., 632.,
                    )));
                }
                self.lesson_scroll_reset = true;
            }
            Step::Geometry(value, wide) => {
                if !self.legacy_native_settled(ctx, *value, *wide) {
                    return (false, None);
                }
            }
            Step::Mode(challenge) => {
                assert_eq!(self.progress.attempts[id].challenge, *challenge);
                assert_eq!(self.legacy_view(i), teaching::View::Assignment);
                if *challenge {
                    assert!(
                        self.metrics.lesson_tabs.is_empty()
                            && !self.metrics.guide
                            && !self.help_open
                    );
                }
            }
            Step::Shot(name, feedback) => {
                if name.ends_with("introduction") {
                    assert_eq!(self.legacy_view(i), teaching::View::Introduction);
                    assert!(self.metrics.terminal.is_none());
                }
                if name.ends_with("assignment") {
                    assert_eq!(self.legacy_view(i), teaching::View::Assignment);
                }
                if *feedback {
                    let r = if self.legacy_view(i) == teaching::View::Example {
                        self.metrics.seed_feedback.first().map(|r| r.1)
                    } else {
                        self.metrics.legacy_feedback
                    }
                    .expect("actual current feedback label");
                    if !self.reveal_smoke_control(ctx, r) {
                        return (false, None);
                    }
                }
                return (true, Some(name.clone()));
            }
            Step::Finish => {
                for id in IDS {
                    let s = self.scenarios.iter().find(|s| s.id == id).unwrap();
                    assert!(progress::current_solved(&self.progress.attempts[id], s));
                }
                let key = egui::Id::new("legacy-native-finish");
                if !ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false) {
                    println!(
                        "PASS three original Unix/Logs native teaching flows: actual controls, separate Insert/Run practice and experiments, current wrong/partial/reference/alternatives/invalidation, Light150/Dark200 wide+narrow typing, exact separated drafts/navigation/save/reset/Challenge. No novice/cold/platform claim."
                    );
                    ctx.data_mut(|d| d.insert_temp(key, true));
                }
                return hold_close(ctx);
            }
        }
        (true, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
        let modifiers = events
            .iter()
            .find_map(|e| {
                if let egui::Event::Key { modifiers, .. } = e {
                    Some(*modifiers)
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1268., 1600.),
                )),
                events,
                modifiers,
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
    }
    fn click(ctx: &egui::Context, a: &mut LabApp, t: Target) {
        a.smoke_stage += 1;
        for _ in 0..24 {
            let done = a.legacy_native_click(ctx, t);
            let events = std::mem::take(&mut a.pending_events);
            frame(ctx, a, events);
            if done {
                return;
            }
        }
        panic!("real legacy control did not complete {t:?}");
    }
    #[test]
    fn legacy_terminal_native_plan_is_normal_bounded_and_covers_independent_current_work() {
        let all = pack::bundled().unwrap();
        let p = plan(&all);
        assert!(START + p.len() < END);
        assert_eq!(p.last(), Some(&Step::Finish));
        for n in 0..3 {
            assert_eq!(
                p.iter()
                    .filter(|s| matches!(s,Step::Filter(i)if *i==n))
                    .count(),
                2
            );
            let start = p
                .iter()
                .position(|s| matches!(s,Step::Filter(i)if *i==n))
                .unwrap();
            let end = p[start + 1..]
                .iter()
                .position(|s| matches!(s,Step::Filter(i)if *i!=n))
                .map_or(p.len(), |i| start + 1 + i);
            let part = &p[start..end];
            for step in [
                Step::BeforeInsert,
                Step::InsertOnly,
                Step::Remember,
                Step::Resume,
                Step::CancelExact,
                Step::ResetExact,
                Step::Mode(true),
                Step::Mode(false),
            ] {
                assert!(part.contains(&step), "{n}:{step:?}");
            }
            assert_eq!(
                part.iter()
                    .filter(|s| matches!(s, Step::Click(Target::Insert)))
                    .count(),
                1
            );
            assert!(part.iter().any(|s| matches!(s, Step::Ran(_, true))));
            assert!(part.iter().any(|s| matches!(s, Step::Ran(_, false))));
            assert!(
                part.iter()
                    .any(|s| matches!(s,Step::Rows(false,rows)if !rows.iter().all(|x|*x)))
            );
            assert!(
                part.iter()
                    .any(|s| matches!(s,Step::Rows(false,rows)if rows.iter().all(|x|*x)))
            );
        }
        for (scale, wide) in [(1.5, false), (1.5, true), (2., false), (2., true)] {
            let start = p
                .iter()
                .position(|s| matches!(s,Step::Geometry(v,w)if (*v,*w)==(scale,wide)))
                .unwrap();
            let end = p[start + 1..]
                .iter()
                .position(|s| matches!(s, Step::Theme(..) | Step::Filter(_)))
                .map_or(p.len(), |i| start + 1 + i);
            assert!(p[start..end].iter().any(|s| matches!(s, Step::Text(_))));
            assert!(p[start..end].contains(&Step::Click(Target::Run)));
        }
        assert!(p.iter().all(|s| !matches!(s,Step::Text(t)if t.len()>4096)));
        let incomplete: Vec<_> = all.into_iter().filter(|s| s.id != IDS[2]).collect();
        assert!(std::panic::catch_unwind(|| plan(&incomplete)).is_err());
    }
    #[test]
    fn legacy_terminal_native_real_geometry_phase_uses_precomputed_context_and_finite_retries() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context(&ctx, true, false);
        a.smoke = false;
        ctx.set_zoom_factor(1.5);
        // Zoom is applied on the next pass and rescales that pass's input rect.
        // Settle it before testing the production geometry sample counter.
        for _ in 0..2 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800., 600.),
                    )),
                    ..Default::default()
                },
                |_| {},
            );
        }
        assert!((ctx.zoom_factor() - 1.5).abs() < 0.01);
        let p = plan(&a.scenarios);
        let offset = p
            .iter()
            .position(|s| *s == Step::Geometry(1.5, false))
            .unwrap();
        a.smoke_stage = START + offset;
        for expected in 1..=3 {
            let mut done = false;
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800., 600.),
                    )),
                    ..Default::default()
                },
                |ctx| done = a.legacy_terminal_native(ctx, a.smoke_stage).0,
            );
            let sample = ctx
                .data(|d| {
                    d.get_temp::<(u8, egui::Rect, u8)>(egui::Id::new((
                        "legacy-native-settle",
                        a.smoke_stage,
                    )))
                })
                .unwrap();
            assert_eq!(sample.0, expected);
            assert_eq!(sample.1, ctx.content_rect());
            assert_eq!(done, expected == 3);
        }
        a.smoke_stage += 1;
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                for _ in 0..31 {
                    let _ = a.legacy_native_settled(&ctx, 2., false);
                }
            }))
            .is_err(),
            "wrong actual zoom must fail within30 attempts"
        );
    }
    #[test]
    fn legacy_terminal_native_click_completion_does_not_require_disappeared_controls() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context(&ctx, true, false);
        a.smoke = false;
        a.smoke_stage = START + 2;
        ctx.data_mut(|d| {
            d.insert_temp(egui::Id::new(("native-click-phases", a.smoke_stage)), 2_u8)
        });
        assert!(a.metrics.terminal.is_none());
        assert!(a.legacy_native_click(&ctx, Target::Input));
        assert!(a.pending_events.is_empty());
    }
    #[test]
    fn legacy_terminal_native_real_insert_typing_run_and_feedback_use_normal_practice() {
        for id in IDS {
            let ctx = egui::Context::default();
            let mut a = LabApp::from_context(&ctx, true, false);
            a.smoke = false;
            let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
            a.launch(i);
            a.set_legacy_view(i, teaching::View::Example);
            for _ in 0..3 {
                frame(&ctx, &mut a, vec![]);
            }
            let p = plan(&a.scenarios);
            let begin = p
                .iter()
                .position(|s| matches!(s,Step::Filter(n)if IDS[*n]==id))
                .unwrap();
            let offset = begin
                + p[begin..]
                    .iter()
                    .position(|s| matches!(s, Step::BeforeInsert))
                    .unwrap();
            a.smoke_stage = START + offset;
            assert!(a.legacy_terminal_native(&ctx, a.smoke_stage).0);
            let before = a.legacy_native_practice(i);
            click(&ctx, &mut a, Target::Insert);
            assert_eq!(a.legacy_native_practice(i)["current"], before["current"]);
            assert_eq!(
                a.legacy_native_practice(i)["transcript"],
                before["transcript"]
            );
            let inserted = begin
                + p[begin..]
                    .iter()
                    .position(|s| *s == Step::InsertOnly)
                    .unwrap();
            a.smoke_stage = START + inserted;
            assert!(a.legacy_terminal_native(&ctx, a.smoke_stage).0);
            click(&ctx, &mut a, Target::Input);
            let expected = a.metrics.focus.unwrap();
            assert_eq!(ctx.memory(|m| m.focused()), Some(expected));
            let text_offset = begin
                + p[begin..]
                    .iter()
                    .position(|s| matches!(s, Step::Text(_)))
                    .unwrap();
            a.smoke_stage = START + text_offset;
            assert!(a.legacy_terminal_native(&ctx, a.smoke_stage).0);
            let events = std::mem::take(&mut a.pending_events);
            frame(&ctx, &mut a, events);
            click(&ctx, &mut a, Target::Run);
            let now = a.legacy_native_practice(i);
            assert_eq!(now["command"], "");
            assert_eq!(now["transcript"].as_array().unwrap().len(), 1);
            assert_eq!(a.progress.attempts[id].state, a.scenarios[i].initial);
            assert!(!progress::current_solved(
                &a.progress.attempts[id],
                &a.scenarios[i]
            ));
            click(&ctx, &mut a, Target::Check);
            assert_eq!(
                a.metrics
                    .seed_feedback
                    .iter()
                    .map(|r| r.0)
                    .collect::<Vec<_>>(),
                a.legacy_native_rows(i)
            );
            assert!(!a.metrics.seed_feedback.is_empty());
        }
    }
    #[test]
    fn legacy_terminal_native_finish_checks_real_models_and_holds_async_close() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context(&ctx, true, false);
        a.smoke = false;
        let p = plan(&a.scenarios);
        for id in IDS {
            let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
            a.launch(i);
        }
        let stage = START + p.len() - 1;
        a.smoke_stage = stage;
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || a.legacy_terminal_native(&ctx, stage)
            ))
            .is_err(),
            "pristine models cannot claimFinish"
        );
        for _ in 0..3 {
            let mut result = (true, None);
            let out = ctx.run(egui::RawInput::default(), |ctx| result = hold_close(ctx));
            assert_eq!(result, (false, None));
            assert_eq!(a.smoke_stage, stage);
            assert!(
                out.viewport_output[&egui::ViewportId::ROOT]
                    .commands
                    .contains(&egui::ViewportCommand::Close)
            );
        }
    }
}
