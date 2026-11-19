//! Fixed developer-only native route; actual evidence requires a guarded run.
//! No model mutation, execute_reference, host utility or supplied success results.
use super::*;
use dlms_learning_lab::shared_libraries_workspace as work;
use std::collections::{BTreeMap, BTreeSet};
pub(in crate::ui) const FIRST: usize = 95_000;
const RESERVE: usize = 1000;
const IDS: [&str; 2] = ["lpic1-library-search", "lpic1-library-cache-handover"];
#[derive(Clone, Debug)]
enum Target {
    Control(String),
    Field(String),
    Track,
    Search,
    Start,
    Back,
    Reset,
    Cancel,
    Confirm,
    Settings,
    SettingsClose,
    Challenge,
    Learn,
    Scale,
    Theme,
    HeaderChoice(String),
    Intro,
    Goal,
    Result,
    Unchecked,
    History,
    Inspection,
}
fn control(s: impl Into<String>) -> Target {
    Target::Control(s.into())
}
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Click(Target),
    Replace(String, String),
    SelectProgram(String),
    SelectStep(usize),
    InsertUnrun(String),
    Run(String, work::Outcome, Option<String>, bool),
    Check(bool),
    Current(bool),
    Snapshot(String, u8),
    Same(String, u8),
    AppliedUnchanged(String),
    SaveRead,
    AssertDraft(String, String),
    AssertSaved(String, Option<String>),
    AssertConfig(String),
    AssertCacheSame(String),
    AssertPartial,
    AssertModal(bool),
    AssertSettings(bool),
    AssertChallenge(bool),
    TabTo(Target),
    Enter,
    InvalidPaste(String, usize),
    EnterTail(String),
    Fresh,
    Layout(bool),
    Shot(String, Target),
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    frames: usize,
    done: bool,
    started: Option<f64>,
    wait: f64,
    stable: u8,
    last: Option<egui::Rect>,
    wheels: usize,
    mouse: u8,
    pressed_at: Option<egui::Pos2>,
    snapshots: BTreeMap<String, serde_json::Value>,
    shots: BTreeSet<String>,
}
fn session(a: &LabApp) -> &work::Session {
    let id = &a.scenarios[a.selected.expect("Actual opened lab")].id;
    let engine::State::SharedLibraries { simulation } = &a.progress.attempts[id].state else {
        panic!("Actual library state")
    };
    simulation
}
fn example(a: &LabApp) -> bool {
    session(a).view == work::View::Example
        && !a.progress.attempts[&a.scenarios[a.selected.unwrap()].id].challenge
}
fn active(a: &LabApp) -> &work::Workspace {
    if example(a) {
        &session(a).example
    } else {
        &session(a).assignment
    }
}
fn value<T: serde::Serialize + ?Sized>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).unwrap()
}
fn snap(a: &LabApp, kind: u8) -> serde_json::Value {
    match kind {
        0 => value(&session(a).assignment),
        1 => value(&(
            active(a).state.clone(),
            active(a).recipes.clone(),
            active(a).checked,
        )),
        2 => value(session(a)),
        3 => value(&active(a).state.cache),
        5 => value(&(active(a).state.clone(), active(a).recipes.clone())),
        4 => {
            let selected = &a.scenarios[a.selected.unwrap()].id;
            value(
                &a.progress
                    .attempts
                    .iter()
                    .filter(|(id, _)| *id != selected)
                    .collect::<BTreeMap<_, _>>(),
            )
        }
        _ => panic!("Closed snapshot kind"),
    }
}
fn field_value<'a>(a: &'a LabApp, key: &str) -> &'a str {
    let w = active(a);
    if key == "command" {
        &w.command
    } else if key == "configuration" {
        &w.configuration_draft
    } else {
        let program = key
            .strip_prefix("launch:")
            .expect("Closed native field key");
        &w.launch_drafts
            .iter()
            .find(|r| r.program == program)
            .unwrap()
            .value
    }
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut ops = vec![Op::Layout(false)];
    for id in IDS {
        let source = labs
            .iter()
            .find(|s| s.id == id)
            .expect("Both authored lessons must be integrated before route");
        let engine::Goal::SharedLibraries { definition: d } = &source.goal else {
            panic!("Authored library Goal")
        };
        ops.extend([
            Op::Open(id.into()),
            Op::Snapshot("unrelated-attempts".into(), 4),
            Op::Shot(format!("{id}-introduction"), Target::Intro),
            Op::Snapshot("assignment-before-example".into(), 0),
            Op::Click(control("view:Example")),
            Op::InvalidPaste("command".into(), work::MAX_COMMAND + 1),
            Op::Replace("command".into(), "ldd /usr/bin/brief".into()),
            Op::EnterTail("command".into()),
        ]);
        for (n, step) in d.lesson.worked.iter().enumerate() {
            ops.push(Op::SelectStep(n));
            match &step.action {
                work::WorkedAction::Inspect { command } => {
                    ops.extend([
                        Op::Click(control("insert-command")),
                        Op::InsertUnrun(command.clone()),
                        Op::Run(command.clone(), work::Outcome::Supported, None, n == 0),
                    ]);
                }
                work::WorkedAction::RebuildCache => ops.extend([
                    Op::Click(control("insert-command")),
                    Op::InsertUnrun("ldconfig".into()),
                    Op::Run("ldconfig".into(), work::Outcome::Supported, None, false),
                ]),
                work::WorkedAction::Configuration { text } => ops.extend([
                    Op::Snapshot("cache-before-save".into(), 3),
                    Op::Click(control("fill-configuration")),
                    Op::AssertDraft("configuration".into(), text.clone()),
                    Op::Click(control("save-configuration")),
                    Op::AssertConfig(text.clone()),
                    Op::AssertCacheSame("cache-before-save".into()),
                ]),
                work::WorkedAction::LaunchRecipe {
                    program,
                    environment,
                } => {
                    ops.extend([
                        Op::Click(control("fill-launch")),
                        Op::AssertDraft(
                            format!("launch:{program}"),
                            environment.clone().unwrap_or_default(),
                        ),
                        Op::Click(control(format!("apply-launch:{program}"))),
                        Op::AssertSaved(program.clone(), environment.clone()),
                        Op::Click(control(format!("inspect-launch:{program}"))),
                    ]);
                }
            }
        }
        ops.extend([
            Op::Check(true),
            Op::Same("assignment-before-example".into(), 0),
            Op::SaveRead,
            Op::Snapshot("before-settings".into(), 2),
            Op::Click(Target::Settings),
            Op::AssertSettings(true),
            Op::Click(Target::SettingsClose),
            Op::AssertSettings(false),
            Op::Same("before-settings".into(), 2),
            Op::Shot(format!("{id}-practice-passed"), Target::Result),
            Op::Shot(
                format!("{id}-practice-workspace"),
                Target::Field(if id == IDS[0] {
                    "launch:/usr/bin/brief".into()
                } else {
                    "configuration".into()
                }),
            ),
            Op::Click(control("view:Assignment")),
            Op::Same("assignment-before-example".into(), 0),
            Op::Shot(format!("{id}-assignment-initial"), Target::Goal),
            Op::Click(Target::Challenge),
            Op::AssertChallenge(true),
            Op::Shot(format!("{id}-challenge-assignment"), Target::Goal),
            Op::Click(Target::Learn),
            Op::AssertChallenge(false),
            Op::Snapshot("diagnostic-model".into(), 1),
            Op::Run(
                "env LD_LIBRARY_PATH=/opt/site/lib:/opt/common/lib ldd /usr/bin/report".into(),
                work::Outcome::Supported,
                None,
                false,
            ),
            Op::AppliedUnchanged("diagnostic-model".into()),
            Op::Check(false),
            Op::SelectProgram("/usr/bin/report".into()),
            Op::Replace("launch:/usr/bin/report".into(), "/not-supplied".into()),
            Op::Snapshot("before-wrong-apply".into(), 1),
            Op::Click(control("apply-launch:/usr/bin/report")),
            Op::AppliedUnchanged("before-wrong-apply".into()),
            Op::AssertDraft("launch:/usr/bin/report".into(), "/not-supplied".into()),
            Op::Check(false),
            Op::Shot(format!("{id}-wrong-feedback"), Target::Result),
        ]);
        if id == IDS[0] {
            ops.extend([
                Op::Replace("launch:/usr/bin/report".into(), "/opt/site/lib".into()),
                Op::TabTo(control("apply-launch:/usr/bin/report")),
                Op::Enter,
                Op::AssertSaved("/usr/bin/report".into(), Some("/opt/site/lib".into())),
                Op::Check(false),
                Op::AssertPartial,
                Op::Shot(format!("{id}-partial-feedback"), Target::Result),
                Op::Replace(
                    "launch:/usr/bin/report".into(),
                    "/opt/site/lib:/opt/common/lib".into(),
                ),
                Op::Click(control("apply-launch:/usr/bin/report")),
                Op::SelectProgram("/usr/bin/audit".into()),
                Op::Replace("launch:/usr/bin/audit".into(), "/opt/common/lib".into()),
                Op::Click(control("apply-launch:/usr/bin/audit")),
            ]);
        } else {
            ops.extend([
                Op::Replace("launch:/usr/bin/report".into(), "/opt/old/lib".into()),
                Op::Click(control("apply-launch:/usr/bin/report")),
                Op::Replace(
                    "configuration".into(),
                    "include /etc/ld.so.conf.d/*.conf\n".into(),
                ),
                Op::Snapshot("invalid-config-applied".into(), 1),
                Op::Click(control("save-configuration")),
                Op::AppliedUnchanged("invalid-config-applied".into()),
                Op::Replace(
                    "configuration".into(),
                    "/opt/keep/lib\n/opt/site/lib\n".into(),
                ),
                Op::Click(control("save-configuration")),
                Op::Run("ldconfig".into(), work::Outcome::Supported, None, false),
                Op::Check(false),
                Op::AssertPartial,
                Op::Shot(format!("{id}-partial-feedback"), Target::Result),
                Op::Replace(
                    "configuration".into(),
                    "# Preserve viewer\n/opt/keep/lib\n/opt/site/lib\n/opt/common/lib\n".into(),
                ),
                Op::Click(control("save-configuration")),
                Op::Run("ldconfig".into(), work::Outcome::Supported, None, false),
                Op::Check(false),
                Op::SelectProgram("/usr/bin/report".into()),
                Op::Replace("launch:/usr/bin/report".into(), "".into()),
                Op::Click(control("apply-launch:/usr/bin/report")),
                Op::SelectProgram("/usr/bin/audit".into()),
                Op::Replace("launch:/usr/bin/audit".into(), "".into()),
                Op::Click(control("apply-launch:/usr/bin/audit")),
            ]);
        }
        ops.extend([
            Op::Check(true),
            Op::Current(true),
            Op::SaveRead,
            Op::Shot(format!("{id}-assignment-passed"), Target::Result),
            Op::SelectProgram("/usr/bin/report".into()),
            Op::Click(control("inspect-launch:/usr/bin/report")),
            Op::Shot(format!("{id}-actual-saved-inspection"), Target::Inspection),
            Op::SelectProgram("/usr/bin/report".into()),
            Op::Replace("launch:/usr/bin/report".into(), "/not-supplied".into()),
            Op::Current(false),
            Op::Shot(format!("{id}-success-invalidated"), Target::Unchecked),
            Op::Replace(
                "launch:/usr/bin/report".into(),
                if id == IDS[0] {
                    "/opt/site/lib:/opt/common/lib".into()
                } else {
                    "".into()
                },
            ),
            Op::Check(true),
            Op::Replace("launch:/usr/bin/report".into(), "/opt/old/lib".into()),
            Op::Click(control("apply-launch:/usr/bin/report")),
            Op::Current(false),
            Op::Shot(format!("{id}-earlier-inspection"), Target::Inspection),
            Op::Replace(
                "launch:/usr/bin/report".into(),
                if id == IDS[0] {
                    "/opt/site/lib:/opt/common/lib".into()
                } else {
                    "".into()
                },
            ),
            Op::Click(control("apply-launch:/usr/bin/report")),
            Op::Check(true),
            Op::Snapshot("before-modal".into(), 2),
            Op::Click(Target::Reset),
            Op::AssertModal(true),
            Op::Click(Target::Cancel),
            Op::AssertModal(false),
            Op::Same("before-modal".into(), 2),
        ]);
        if id == IDS[1] {
            ops.extend([
                Op::Layout(true),
                Op::Shot(
                    format!("{id}-light200-narrow-editor"),
                    Target::Field("configuration".into()),
                ),
                Op::Shot(format!("{id}-light200-narrow-history"), Target::History),
                Op::Layout(false),
            ]);
        }
        ops.extend([
            Op::Click(Target::Reset),
            Op::Click(Target::Confirm),
            Op::Fresh,
            Op::Same("unrelated-attempts".into(), 4),
        ]);
    }
    ops
}
impl LabApp {
    pub(in crate::ui) fn shared_libraries_capture_active(&self) -> bool {
        (FIRST..FIRST + RESERVE).contains(&self.smoke_stage)
    }
    fn libraries_target(
        &self,
        ctx: &egui::Context,
        t: &Target,
    ) -> Option<(egui::Rect, egui::Rect, bool)> {
        let m = &self.metrics.shared_libraries;
        let outer = self
            .metrics
            .main_viewport
            .unwrap_or(ctx.content_rect())
            .intersect(ctx.content_rect());
        let global = ctx.content_rect();
        Some(match t {
            Target::Control(key) => {
                let r = m.controls.iter().find(|r| &r.0 == key)?;
                let popup = key.starts_with("program:") || key.starts_with("step:");
                (
                    r.1,
                    r.2.intersect(if popup { global } else { outer }),
                    popup,
                )
            }
            Target::Field(key) => {
                let r = m.fields.iter().find(|r| &r.0 == key)?;
                (r.1, r.3.intersect(outer), false)
            }
            Target::Track => {
                let r = self
                    .metrics
                    .library_track_choices
                    .iter()
                    .find(|r| r.0 == "LPIC-1")?;
                (r.1, outer, false)
            }
            Target::Search => (self.metrics.library_search?, outer, false),
            Target::Start => (self.metrics.card_actions.first()?.0, outer, false),
            Target::Back => (self.metrics.back?, outer, false),
            Target::Reset => (self.metrics.reset?, outer, false),
            Target::Cancel => (self.metrics.reset_cancel?, outer, false),
            Target::Confirm => (self.metrics.confirm_reset?, outer, false),
            Target::Settings => (self.metrics.settings?, global, false),
            Target::SettingsClose => {
                let r = self.metrics.settings_close?;
                (r.0, r.3.intersect(global), false)
            }
            Target::Challenge => (self.metrics.mode_challenge?, outer, false),
            Target::Learn => (self.metrics.mode_learn?, outer, false),
            Target::Scale => (self.metrics.reopen_scale?, global, false),
            Target::Theme => (self.metrics.reopen_theme?, global, false),
            Target::HeaderChoice(key) => {
                let r = self
                    .metrics
                    .physical_picker_choices
                    .iter()
                    .find(|r| &r.0 == key)?;
                (r.1, r.2.intersect(global), true)
            }
            Target::Intro => (m.intro_goal?, outer, false),
            Target::Goal => (m.goal?, outer, false),
            Target::Result => (m.result?.2, outer, false),
            Target::Unchecked => {
                let r = m.controls.iter().find(|r| r.0 == "unchecked")?;
                (r.1, r.2.intersect(outer), false)
            }
            Target::History => (m.history?, outer, false),
            Target::Inspection => (m.inspection?, outer, false),
        })
    }
    fn libraries_reveal(&mut self, ctx: &egui::Context, t: &Target, r: &mut Route) -> bool {
        let Some((rect, clip, popup)) = self.libraries_target(ctx, t) else {
            ctx.request_repaint();
            return false;
        };
        let now = ctx.input(|i| i.time);
        if now < r.wait {
            return false;
        }
        let tol = 1. / ctx.pixels_per_point();
        let same = r.last.is_some_and(|old| {
            old.min.distance(rect.min) < tol * 0.25 && old.max.distance(rect.max) < tol * 0.25
        });
        r.stable = if same { r.stable.saturating_add(1) } else { 0 };
        r.last = Some(rect);
        if r.stable < 2 {
            return false;
        }
        if smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
            return true;
        }
        let main = self
            .metrics
            .main_viewport
            .unwrap_or(ctx.content_rect())
            .intersect(ctx.content_rect());
        let area = if popup { clip } else { main };
        assert!(area.is_positive(), "Actual reveal viewport: {t:?}");
        assert!(
            rect.left() >= area.left() - tol && rect.right() <= area.right() + tol,
            "Horizontal clipping {t:?}: {rect:?}/{area:?}"
        );
        assert!(
            rect.height() <= area.height() + tol,
            "A whole target cannot fit: {t:?}: {rect:?}/{area:?}"
        );
        assert!(
            r.wheels < 20,
            "Bounded20-wheel native reveal failed: {t:?} stage{} {rect:?}/{area:?}",
            r.at
        );
        r.wheels += 1;
        let dy = if rect.bottom() > area.bottom() {
            area.bottom() - rect.bottom()
        } else {
            area.top() - rect.top()
        };
        self.pending_events.extend([
            egui::Event::PointerMoved(if popup {
                area.center()
            } else {
                shared_libraries_outer_wheel_point(self, area, ctx.pixels_per_point())
            }),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., dy.clamp(-area.height() * 0.75, area.height() * 0.75)),
                modifiers: Default::default(),
            },
        ]);
        r.wait = now + 0.15;
        r.stable = 0;
        r.last = None;
        false
    }
    fn libraries_click(&mut self, ctx: &egui::Context, t: &Target, r: &mut Route) -> bool {
        if r.mouse == 3 {
            r.mouse = 0;
            r.pressed_at = None;
            r.wheels = 0;
            r.last = None;
            r.stable = 0;
            return true;
        }
        if r.mouse == 0 && !self.libraries_reveal(ctx, t, r) {
            return false;
        }
        let (rect, clip, _) = self
            .libraries_target(ctx, t)
            .expect("Actual target while clicking");
        assert!(
            smoke_control_visible(rect, clip, ctx.pixels_per_point()),
            "No offscreen click substitution"
        );
        let tol = 1. / ctx.pixels_per_point();
        if r.mouse == 1
            && r.last.is_some_and(|old| {
                old.min.distance(rect.min) > tol * 0.25 || old.max.distance(rect.max) > tol * 0.25
            })
        {
            r.mouse = 0;
            r.stable = 0;
            r.last = None;
            return false; // settle actual geometry again before any press
        }
        let point = if r.mouse == 2 {
            r.pressed_at.expect("Release at the actual press point")
        } else {
            rect.center()
        };
        assert!(
            rect.contains(point) && clip.contains(point),
            "A moving target must not be chased as a drag"
        );
        self.pending_events.push(egui::Event::PointerMoved(point));
        if r.mouse > 0 {
            self.pending_events.push(egui::Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: r.mouse == 1,
                modifiers: Default::default(),
            });
        }
        if r.mouse == 1 {
            r.pressed_at = Some(point);
        }
        r.mouse += 1;
        false
    }
    pub(in crate::ui) fn shared_libraries_lessons_tick(&mut self, ctx: &egui::Context) {
        if ctx.current_pass_index() != 0 || ctx.will_discard() {
            return;
        }
        let root = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let marker = egui::Id::new("shared-libraries-native-root");
        if ctx.data(|d| d.get_temp::<u64>(marker)) == Some(root) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(marker, root));
        let key = egui::Id::new("shared-libraries-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: route(&self.scenarios),
                ..Default::default()
            });
        if r.done {
            return;
        }
        assert!(r.ops.len() < RESERVE, "Fixed route namespace bound");
        if r.at == r.ops.len() {
            println!(
                "SHARED_LIBRARIES_NATIVE_COMPLETE: {} real event operations, {} requested native screenshots; no host library utilities, independent novice or other-platform acceptance",
                r.ops.len(),
                r.shots.len()
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = FIRST + r.at;
        let op = r.ops[r.at].clone();
        let now = ctx.input(|i| i.time);
        let start = *r.started.get_or_insert(now);
        assert!(
            now - start < 15.,
            "Native action elapsed bound {op:?}, phase{}, wheels{}; selected{:?} actions{:?} stable{} last{:?} stage{} viewport{:?}",
            r.phase,
            r.wheels,
            self.selected,
            self.metrics.card_actions,
            r.stable,
            r.last,
            self.smoke_stage,
            ctx.content_rect()
        );
        if now < r.wait {
            ctx.request_repaint_after(std::time::Duration::from_millis(10));
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "Native frame bound {op:?} offset{:?}",
            self.metrics.main_scroll_offset
        );
        if r.frames == 1 {
            println!("SHARED_LIBRARIES_NATIVE_STEP {} {op:?}", r.at);
        }
        let done = match op {
            Op::Open(id) => {
                let source = self.scenarios.iter().find(|s| s.id == id).unwrap();
                let title = source.title.clone();
                if r.phase == 0 {
                    if self.selected.is_some() || r.mouse == 3 {
                        if self.libraries_click(ctx, &Target::Back, &mut r) {
                            r.phase = 1;
                        }
                        false
                    } else {
                        r.phase = 1;
                        false
                    }
                } else if r.phase == 1 {
                    if self.libraries_click(ctx, &Target::Track, &mut r) {
                        r.phase = 2;
                    }
                    false
                } else if r.phase == 2 {
                    assert_eq!(self.filter.track, "LPIC-1");
                    if self.libraries_click(ctx, &Target::Search, &mut r) {
                        r.phase = 3;
                    }
                    false
                } else if r.phase == 3 {
                    paste(self, &title);
                    r.phase = 4;
                    false
                } else if r.phase == 4 {
                    // Native eframe may render the newly opened lesson before
                    // this controller acknowledges the already delivered release.
                    // Acknowledge first; the next phase verifies its actual ID.
                    if r.mouse != 3 {
                        assert_eq!(self.filter.search, title);
                        assert_eq!(
                            self.metrics.card_actions.len(),
                            1,
                            "Exactly one actual searched Start control before pressing it"
                        );
                    }
                    if self.libraries_click(ctx, &Target::Start, &mut r) {
                        r.phase = 5;
                    }
                    false
                } else {
                    assert_eq!(self.scenarios[self.selected.unwrap()].id, id);
                    true
                }
            }
            Op::Click(t) => {
                if r.frames == 1
                    && let Target::Control(name) = &t
                {
                    if name == "insert-command" {
                        r.snapshots
                            .insert("before-insert".into(), value(&active(self).transcript));
                    }
                    if matches!(name.as_str(), "fill-configuration" | "fill-launch") {
                        r.snapshots.insert("before-fill".into(), snap(self, 5));
                    }
                }
                self.libraries_click(ctx, &t, &mut r)
            }
            Op::SelectProgram(program) => {
                if self
                    .metrics
                    .shared_libraries
                    .fields
                    .iter()
                    .any(|f| f.0 == format!("launch:{program}"))
                {
                    true
                } else if r.phase == 0 {
                    if self.libraries_click(ctx, &control("program-picker"), &mut r) {
                        r.phase = 1;
                    }
                    false
                } else {
                    self.libraries_click(ctx, &control(format!("program:{program}")), &mut r)
                }
            }
            Op::SelectStep(n) => {
                if r.phase == 0 {
                    if self.libraries_click(ctx, &control("step-picker"), &mut r) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    if self.libraries_click(ctx, &control(format!("step:{n}")), &mut r) {
                        r.phase = 2;
                    }
                    false
                } else {
                    assert_eq!(session(self).selected_example, n);
                    true
                }
            }
            Op::Replace(field, text) => replace(self, ctx, &field, &text, &mut r),
            Op::InsertUnrun(text) => {
                assert_eq!(active(self).command, text);
                assert_eq!(
                    value(&active(self).transcript),
                    r.snapshots["before-insert"],
                    "Insert did not execute or rewrite any previous result"
                );
                true
            }
            Op::Run(text, outcome, needle, enter) => {
                if r.phase < 3 {
                    replace(self, ctx, "command", &text, &mut r)
                } else if r.phase == 3 {
                    if enter {
                        key_event(self, egui::Key::Enter, Default::default());
                        r.phase = 4;
                    } else if self.libraries_click(ctx, &control("run"), &mut r) {
                        r.phase = 4;
                    }
                    false
                } else {
                    let row = active(self)
                        .transcript
                        .last()
                        .expect("Actual Run appended history");
                    assert_eq!(row.input, text);
                    assert_eq!(row.outcome, outcome);
                    if let Some(needle) = needle {
                        assert!(
                            row.output.contains(&needle),
                            "Actual output: {}",
                            row.output
                        );
                    }
                    true
                }
            }
            Op::Check(expected) => {
                if r.phase == 0 {
                    if self.libraries_click(ctx, &control("check"), &mut r) {
                        r.phase = 1;
                    }
                    false
                } else {
                    let (passed, total, _) = self
                        .metrics
                        .shared_libraries
                        .result
                        .expect("Real checked feedback");
                    assert!(total > 0);
                    assert_eq!(passed == total, expected);
                    if example(self) {
                        assert!(!progress::current_solved(
                            &self.progress.attempts[&self.scenarios[self.selected.unwrap()].id],
                            &self.scenarios[self.selected.unwrap()]
                        ));
                    }
                    true
                }
            }
            Op::Current(expected) => {
                let source = &self.scenarios[self.selected.unwrap()];
                assert_eq!(
                    progress::current_solved(&self.progress.attempts[&source.id], source),
                    expected
                );
                if !expected {
                    assert!(
                        self.metrics.shared_libraries.result.is_none(),
                        "No stale checked panel after draft edit"
                    );
                }
                true
            }
            Op::Snapshot(name, kind) => {
                r.snapshots.insert(name, snap(self, kind));
                true
            }
            Op::Same(name, kind) => {
                assert_eq!(snap(self, kind), r.snapshots[&name]);
                true
            }
            Op::AppliedUnchanged(name) => {
                assert_eq!(snap(self, 1), r.snapshots[&name]);
                true
            }
            Op::AssertDraft(field, text) => {
                assert_eq!(field_value(self, &field), text);
                if r.at > 0
                    && matches!(&r.ops[r.at-1],Op::Click(Target::Control(name))if matches!(name.as_str(),"fill-configuration"|"fill-launch"))
                {
                    assert_eq!(
                        snap(self, 5),
                        r.snapshots["before-fill"],
                        "Practice Fill changed only its raw draft"
                    );
                }
                true
            }
            Op::AssertSaved(program, environment) => {
                assert_eq!(
                    active(self)
                        .recipes
                        .iter()
                        .find(|v| v.program == program)
                        .unwrap()
                        .environment,
                    environment
                );
                true
            }
            Op::AssertConfig(text) => {
                assert_eq!(active(self).state.configuration, text);
                true
            }
            Op::AssertCacheSame(name) => {
                assert_eq!(value(&active(self).state.cache), r.snapshots[&name]);
                true
            }
            Op::AssertPartial => {
                let (passed, total, _) = self.metrics.shared_libraries.result.unwrap();
                assert!(passed > 0 && passed < total);
                true
            }
            Op::AssertModal(open) => {
                assert_eq!(self.reset_pending, open);
                true
            }
            Op::AssertSettings(open) => {
                assert_eq!(self.settings_open, open);
                true
            }
            Op::AssertChallenge(on) => {
                let id = &self.scenarios[self.selected.unwrap()].id;
                assert_eq!(
                    self.progress.attempts[id].challenge,
                    on,
                    "actual header response {:?}; main clip {:?}; pointer {:?}",
                    self.metrics.mode_challenge,
                    self.metrics.main_viewport,
                    ctx.input(|i| i.pointer.hover_pos())
                );
                assert_eq!(self.metrics.lesson_tabs.is_empty(), on);
                true
            }
            Op::TabTo(target) => {
                if r.phase == 0 {
                    key_event(self, egui::Key::Tab, Default::default());
                    r.phase = 1;
                    false
                } else {
                    let id = ctx
                        .memory(|m| m.focused())
                        .expect("Native Tab focuses a widget");
                    let response = ctx
                        .read_response(id)
                        .expect("Current-pass focused native widget");
                    let (rect, _, _) = self.libraries_target(ctx, &target).unwrap();
                    assert_eq!(response.rect, rect, "Tab reaches intended actual control");
                    assert!(response.enabled());
                    true
                }
            }
            Op::Enter => {
                key_event(self, egui::Key::Enter, Default::default());
                true
            }
            Op::InvalidPaste(field, size) => {
                if r.phase == 0 {
                    r.snapshots.insert("invalid-packet".into(), snap(self, 2));
                    if self.libraries_click(ctx, &Target::Field(field), &mut r) {
                        self.pending_events
                            .push(egui::Event::Paste("x".repeat(size)));
                        r.phase = 1;
                    }
                    false
                } else {
                    assert_eq!(snap(self, 2), r.snapshots["invalid-packet"]);
                    true
                }
            }
            Op::EnterTail(field) => {
                if r.phase == 0 {
                    if self.libraries_click(ctx, &Target::Field(field), &mut r) {
                        r.snapshots.insert("enter-tail".into(), snap(self, 2));
                        key_event(self, egui::Key::Enter, Default::default());
                        self.pending_events
                            .push(egui::Event::Text("not-run".into()));
                        r.phase = 1;
                    }
                    false
                } else {
                    assert_eq!(snap(self, 2), r.snapshots["enter-tail"]);
                    true
                }
            }
            Op::SaveRead => {
                save_read(self);
                true
            }
            Op::Fresh => {
                let source = &self.scenarios[self.selected.unwrap()];
                assert_eq!(
                    value(&self.progress.attempts[&source.id].state),
                    value(&source.initial)
                );
                assert!(!progress::current_solved(
                    &self.progress.attempts[&source.id],
                    source
                ));
                true
            }
            Op::Layout(light) => layout(self, ctx, light, &mut r),
            Op::Shot(name, target) => {
                if r.phase == 0 {
                    if self.libraries_reveal(ctx, &target, &mut r) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    assert!(
                        r.shots.insert(name.clone()),
                        "Unique actual screenshot request"
                    );
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
        // replace() finishes raw editing at phase3; only plain Replace completes
        // there. Run must continue to an actual Run/Enter phase on a later frame.
        let done = if matches!(r.ops[r.at], Op::Run(..)) && r.phase == 3 {
            false
        } else {
            done
        };
        if done {
            r.at += 1;
            r.phase = 0;
            r.mouse = 0;
            r.frames = 0;
            r.started = None;
            r.stable = 0;
            r.last = None;
            r.wheels = 0;
        }
        r.wait = r.wait.max(now + 0.01);
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint_after(std::time::Duration::from_millis(10));
    }
}
fn key_event(a: &mut LabApp, key: egui::Key, modifiers: egui::Modifiers) {
    for pressed in [true, false] {
        a.pending_events.push(egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        });
    }
}
fn paste(a: &mut LabApp, text: &str) {
    key_event(
        a,
        egui::Key::A,
        egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    );
    if text.is_empty() {
        key_event(a, egui::Key::Backspace, Default::default());
    } else {
        a.pending_events.push(egui::Event::Paste(text.into()));
    }
}
fn replace(a: &mut LabApp, ctx: &egui::Context, field: &str, text: &str, r: &mut Route) -> bool {
    if r.phase == 0 {
        if a.libraries_click(ctx, &Target::Field(field.into()), r) {
            r.phase = 1;
        }
        false
    } else if r.phase == 1 {
        let id = a
            .metrics
            .shared_libraries
            .fields
            .iter()
            .find(|f| f.0 == field)
            .unwrap()
            .2;
        assert_eq!(ctx.memory(|m| m.focused()), Some(id));
        paste(a, text);
        r.phase = 2;
        false
    } else {
        assert_eq!(field_value(a, field), text);
        r.phase = 3;
        true
    }
}
fn save_read(a: &LabApp) {
    let disk =
        std::path::PathBuf::from(std::env::var_os("TMPDIR").expect("MAIN disk-backed runner"));
    assert!(
        disk.is_absolute()
            && disk
                .components()
                .any(|c| c.as_os_str() == ".build-artifacts")
    );
    let dir = tempfile::Builder::new()
        .prefix("shared-library-native-progress-")
        .tempdir_in(disk)
        .unwrap()
        .keep();
    let path = dir.join("progress.json");
    let before = value(&a.progress);
    progress::save_with_catalog(&path, &a.progress, &a.scenarios).unwrap();
    let restored = progress::load(&path, &a.scenarios).unwrap();
    assert_eq!(
        value(&restored),
        before,
        "Actual saved state/drafts/history survive catalog-aware read"
    );
    assert!(restored.migration_receipts.is_some());
    println!("SHARED_LIBRARIES_PROGRESS_ROUNDTRIP {}", path.display());
}
fn layout(a: &mut LabApp, ctx: &egui::Context, light: bool, r: &mut Route) -> bool {
    let theme = if light {
        theme::inventory()
            .iter()
            .find(|t| t.light)
            .unwrap()
            .id
            .clone()
    } else {
        "lab-navy".into()
    };
    let scale = if light { 2. } else { 1. };
    match r.phase {
        0 => {
            if a.libraries_click(ctx, &Target::Theme, r) {
                r.phase = 1;
            }
            false
        }
        1 => {
            if a.libraries_click(ctx, &Target::HeaderChoice(format!("theme:{theme}")), r) {
                r.phase = 2;
            }
            false
        }
        2 => {
            if a.libraries_click(ctx, &Target::Scale, r) {
                r.phase = 3;
            }
            false
        }
        3 => {
            if a.libraries_click(ctx, &Target::HeaderChoice(format!("scale:{scale:.2}")), r) {
                r.phase = 4;
            }
            false
        }
        4 => {
            assert_eq!(a.theme_id, theme);
            assert!((a.scale - scale).abs() < 0.001);
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!light));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(if light {
                egui::vec2(820., 1100.)
            } else {
                egui::vec2(1250., 1000.)
            }));
            r.wait = ctx.input(|i| i.time) + 0.2;
            r.phase = 5;
            false
        }
        _ => {
            let physical = ctx.content_rect().size() * ctx.pixels_per_point();
            assert!(
                physical.x > 300. && physical.y > 400.,
                "Actual resized native surface"
            );
            true
        }
    }
}

pub(in crate::ui) fn shared_libraries_expected_captures(labs: &[Scenario]) -> BTreeSet<String> {
    route(labs)
        .into_iter()
        .filter_map(|o| {
            if let Op::Shot(name, _) = o {
                Some(name)
            } else {
                None
            }
        })
        .collect()
}
/// A real outer-wheel hit point outside all recorded nested inspection/editor
/// viewports; no direct scroll mutation and no assumed gutter pixel.
pub(in crate::ui) fn shared_libraries_outer_wheel_point(
    a: &LabApp,
    area: egui::Rect,
    ppp: f32,
) -> egui::Pos2 {
    // The real parent allocation includes its scrollbar gutter; child content
    // clips cannot consume a wheel there. inner_rect alone excludes that gutter.
    if let Some(outer) = a.metrics.main_scroll_region
        && outer.right() > area.right() + 1. / ppp
    {
        let point = egui::pos2((outer.right() + area.right()) / 2., area.center().y);
        assert!(outer.contains(point) && !area.contains(point));
        return point;
    }
    let m = &a.metrics.shared_libraries;
    let mut nested = vec![];
    nested.extend([m.inventory, m.history, m.inspection].into_iter().flatten());
    nested.extend(
        m.fields
            .iter()
            .filter(|f| f.0 == "configuration")
            .map(|f| f.3),
    );
    let epsilon = 1. / ppp;
    let mut xs = vec![
        area.left() + epsilon,
        area.center().x,
        area.right() - epsilon,
    ];
    let mut ys = vec![
        area.top() + epsilon,
        area.center().y,
        area.bottom() - epsilon,
    ];
    for r in &nested {
        xs.extend([r.left() - epsilon, r.right() + epsilon]);
        ys.extend([r.top() - epsilon, r.bottom() + epsilon]);
    }
    for y in ys {
        for &x in &xs {
            let p = egui::pos2(x, y);
            if area.contains(p) && !nested.iter().any(|r| r.contains(p)) {
                return p;
            }
        }
    }
    panic!("No actual outer-scroll hit area outside nested viewports: {area:?} {nested:?}")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_recipe_retains_two_real_labs_paste_scope_checks_wrong_partial_and_reset() {
        let (labs, _) = crate::ui::tests::ccna_pilot::catalog();
        let ops = route(&labs);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Open(_))).count(), 2);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Fresh)).count(), 2);
        assert_eq!(
            ops.iter()
                .filter(|o| matches!(o, Op::InvalidPaste(..)))
                .count(),
            2
        );
        assert_eq!(
            ops.iter()
                .filter(|o| matches!(o, Op::EnterTail(..)))
                .count(),
            2
        );
        assert_eq!(
            ops.iter()
                .filter(|o| matches!(o, Op::AssertPartial))
                .count(),
            2
        );
        assert_eq!(
            ops.iter().filter(|o| matches!(o, Op::Layout(true))).count(),
            1
        );
        assert!(ops.len() < RESERVE);
        let shots = shared_libraries_expected_captures(&labs);
        assert_eq!(
            shots.len(),
            ops.iter().filter(|o| matches!(o, Op::Shot(..))).count()
        );
        for id in IDS {
            for suffix in [
                "introduction",
                "practice-passed",
                "practice-workspace",
                "assignment-initial",
                "challenge-assignment",
                "wrong-feedback",
                "partial-feedback",
                "assignment-passed",
                "actual-saved-inspection",
                "success-invalidated",
                "earlier-inspection",
            ] {
                assert!(shots.contains(&format!("{id}-{suffix}")));
            }
        }
    }
}
