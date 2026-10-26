//! Fixed normal-course package acceptance, driven only by real native input.
//! Reviewed desired command strings are not a novice/usability study.
use super::*;
use dlms_learning_lab::{debian_packages_teaching::Learning, debian_packages_terminal::Session};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const FIRST: usize = 34000;
const END: usize = 35000;
const IDS: [&str; 4] = [
    "lpic1-package-install",
    "lpic1-package-metadata",
    "lpic1-package-payload",
    "lpic1-package-handover",
];
const UNIT: &str = "debian-package-administration";
const ASSIGNMENT_DRAFT: &str = "dpkg-query -W 'unfinished assignment";
const EXAMPLE_DRAFT: &str = "apt-cache depends 'unfinished example";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Review {
    id: String,
    annotation_commands: Vec<String>,
    partial: Vec<String>,
    alternate: Vec<String>,
    experiment: Vec<String>,
    repair: Vec<String>,
    break_current: Vec<String>,
    repair_current: Vec<String>,
}
fn reviews() -> Vec<Review> {
    let rows: Vec<Review> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/debian-packages-native-review.json"
    ))
    .unwrap();
    assert_eq!(rows.len(), 4);
    for (r, id) in rows.iter().zip(IDS) {
        assert_eq!(r.id, id);
        for group in [
            &r.annotation_commands,
            &r.partial,
            &r.alternate,
            &r.experiment,
            &r.repair,
            &r.break_current,
            &r.repair_current,
        ] {
            assert!(!group.is_empty() && group.len() <= 8);
            assert!(group.iter().all(|c| !c.is_empty() && c.len() <= 2048));
        }
    }
    rows
}
#[derive(Clone, Debug, PartialEq)]
enum Target {
    Try,
    Example,
    Assignment,
    Input,
    Run,
    Picker,
    Choice(usize),
    Insert,
    Check,
    Reset,
    Cancel,
    Confirm,
    Challenge,
    Back,
    Favorite,
}
#[derive(Clone, Debug, PartialEq)]
enum Shot {
    Heading,
    Input,
    Feedback,
}
#[derive(Clone, Debug, PartialEq)]
enum Step {
    Select(usize),
    Wait,
    Click(Target),
    Type(String),
    Ran(String, bool, bool),
    RecordInsert,
    InsertOnly,
    Practice(bool),
    Current(bool),
    Partial,
    RecordReset,
    CancelExact,
    ResetExact,
    SaveReadback,
    RecordDrafts,
    Filter,
    Resume,
    ResumeExact,
    Theme(bool),
    Geometry(f32),
    Challenge,
    Capture(String, Shot),
    Library,
    Finish,
}
fn commands(p: &mut Vec<Step>, commands: &[String], practice: bool) {
    for c in commands {
        p.extend([
            Step::Click(Target::Input),
            Step::Type(c.clone()),
            Step::Click(Target::Run),
            Step::Ran(c.clone(), practice, true),
        ]);
    }
}
fn check(p: &mut Vec<Step>, predicate: Step, name: String) {
    p.extend([
        Step::Click(Target::Check),
        Step::Wait,
        predicate,
        Step::Capture(name, Shot::Feedback),
    ]);
}
fn reset(p: &mut Vec<Step>) {
    p.extend([
        Step::RecordReset,
        Step::Click(Target::Reset),
        Step::Click(Target::Confirm),
        Step::Wait,
        Step::ResetExact,
        Step::Click(Target::Assignment),
        Step::Wait,
    ]);
}
fn plan(scenarios: &[Scenario]) -> Vec<Step> {
    let mut p = vec![];
    for (n, r) in reviews().iter().enumerate() {
        let s = scenarios
            .iter()
            .find(|s| s.id == r.id)
            .expect("registered normal package lesson");
        assert_eq!(s.unit, UNIT);
        assert!(matches!(s.initial, engine::State::DebianPackages { .. }));
        let l = s
            .debian_packages_teaching
            .as_ref()
            .expect("authored separate practice");
        let actual: Vec<_> = l
            .example_actions
            .iter()
            .map(|a| {
                a.command
                    .as_deref()
                    .expect("reviewed real command annotation")
            })
            .collect();
        assert_eq!(
            actual,
            r.annotation_commands
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
        );
        let name = |suffix: &str| format!("packages-{n}-{suffix}");
        p.extend([
            Step::Select(n),
            Step::Wait,
            Step::Capture(name("introduction"), Shot::Heading),
            Step::Click(Target::Try),
            Step::Wait,
        ]);
        // One real picker/Insert proof, followed by ordinary typed commands for
        // all remaining steps. Insert must never execute an example action.
        p.extend([
            Step::Click(Target::Picker),
            Step::Click(Target::Choice(0)),
            Step::RecordInsert,
            Step::Click(Target::Insert),
            Step::InsertOnly,
            Step::Click(Target::Run),
            Step::Ran(r.annotation_commands[0].clone(), true, true),
        ]);
        commands(&mut p, &r.annotation_commands[1..], true);
        check(&mut p, Step::Practice(true), name("example-complete"));
        commands(&mut p, &r.experiment, true);
        check(&mut p, Step::Practice(false), name("example-experiment"));
        commands(&mut p, &r.repair, true);
        check(&mut p, Step::Practice(true), name("example-repaired"));
        p.extend([
            Step::Click(Target::Assignment),
            Step::Wait,
            Step::Capture(name("assignment"), Shot::Heading),
        ]);
        check(&mut p, Step::Current(false), name("initial-evidence"));
        commands(&mut p, &r.partial, false);
        check(&mut p, Step::Partial, name("partial"));
        reset(&mut p);
        commands(&mut p, &s.solution, false);
        check(&mut p, Step::Current(true), name("complete"));
        commands(&mut p, &["dpkg-query -W".into()], false);
        p.push(Step::Current(true));
        p.extend([
            Step::Click(Target::Input),
            Step::Type("dpkg-query -W 'unfinished".into()),
            Step::Click(Target::Run),
            Step::Ran("dpkg-query -W 'unfinished".into(), false, false),
            Step::Current(true),
        ]);
        commands(&mut p, &["dpkg-query -W museum-report".into()], false);
        for light in [true, false] {
            p.extend([
                Step::Theme(light),
                Step::Geometry(if light { 1.5 } else { 2. }),
                Step::Click(Target::Input),
                Step::Type("dpkg-query -W museum-lib".into()),
                Step::Capture(
                    name(if light {
                        "light150-focused-input"
                    } else {
                        "dark200-focused-input"
                    }),
                    Shot::Input,
                ),
                Step::Click(Target::Run),
                Step::Ran("dpkg-query -W museum-lib".into(), false, true),
            ]);
            check(
                &mut p,
                Step::Current(true),
                name(if light {
                    "light150-current"
                } else {
                    "dark200-current"
                }),
            );
        }
        p.extend([Step::Theme(false), Step::Geometry(2.)]);
        commands(&mut p, &r.break_current, false);
        check(&mut p, Step::Current(false), name("invalidated"));
        commands(&mut p, &r.repair_current, false);
        check(&mut p, Step::Current(true), name("repaired"));
        p.extend([
            Step::Click(Target::Input),
            Step::Type(ASSIGNMENT_DRAFT.into()),
            Step::Click(Target::Example),
            Step::Wait,
            Step::Click(Target::Input),
            Step::Type(EXAMPLE_DRAFT.into()),
            Step::RecordDrafts,
            Step::SaveReadback,
            Step::Click(Target::Back),
            Step::Filter,
            Step::Wait,
            Step::Click(Target::Favorite),
            Step::SaveReadback,
            Step::Resume,
            Step::Wait,
            Step::ResumeExact,
            Step::Capture(name("resumed-example-draft"), Shot::Input),
            Step::Click(Target::Assignment),
            Step::Wait,
            Step::RecordReset,
            Step::Click(Target::Reset),
            Step::Click(Target::Cancel),
            Step::CancelExact,
        ]);
        reset(&mut p);
        commands(&mut p, &r.alternate, false);
        check(&mut p, Step::Current(true), name("alternate"));
        p.extend([
            Step::SaveReadback,
            Step::Click(Target::Challenge),
            Step::Wait,
            Step::Challenge,
            Step::Capture(name("challenge"), Shot::Heading),
        ]);
    }
    p.extend([
        Step::Click(Target::Back),
        Step::Library,
        Step::Wait,
        Step::Capture("packages-unit-library".into(), Shot::Heading),
        Step::Finish,
    ]);
    assert!(p.len() < END - FIRST, "closed range before35000");
    p
}
fn session(a: &progress::Attempt, practice: bool) -> &Session {
    let w = a.workspace.as_ref().unwrap();
    if practice {
        w.debian_packages_learning
            .as_ref()
            .unwrap()
            .terminal
            .as_ref()
            .unwrap()
    } else {
        w.debian_packages_terminal.as_ref().unwrap()
    }
}
fn learning(a: &progress::Attempt) -> &Learning {
    a.workspace
        .as_ref()
        .unwrap()
        .debian_packages_learning
        .as_ref()
        .unwrap()
}
fn key_a(pressed: bool) -> egui::Event {
    egui::Event::Key {
        key: egui::Key::A,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: pressed,
            command: pressed,
            ..Default::default()
        },
    }
}
type ResetRecord = (Value, BTreeMap<String, Value>, BTreeSet<String>);
impl LabApp {
    fn package_lesson_target(&self, t: &Target) -> Option<egui::Rect> {
        match t {
            Target::Try => self.metrics.example_action,
            Target::Example => self.metrics.lesson_tabs.get(1).copied(),
            Target::Assignment => self
                .metrics
                .assignment_action
                .or_else(|| self.metrics.lesson_tabs.get(2).copied()),
            Target::Input => self.metrics.package_command.map(|v| v.0),
            Target::Run => self.metrics.package_run.map(|v| v.0),
            Target::Picker => self.metrics.package_example,
            Target::Choice(n) => self
                .metrics
                .package_example_items
                .iter()
                .find(|v| v.0 == *n)
                .map(|v| v.1),
            Target::Insert => self.metrics.command_insert,
            Target::Check => self.metrics.check,
            Target::Reset => self.metrics.reset,
            Target::Cancel => self.metrics.reset_cancel,
            Target::Confirm => self.metrics.confirm_reset,
            Target::Challenge => self.metrics.mode_challenge,
            Target::Back => self.metrics.back,
            Target::Favorite => self.metrics.card_actions.first().map(|r| r.1),
        }
    }
    /// Strict bounded reveal, with the wheel inside the actual target column.
    fn package_lesson_reveal(&mut self, ctx: &egui::Context, r: egui::Rect) -> bool {
        let content = ctx.content_rect();
        let ppp = ctx.pixels_per_point();
        let clip = self
            .metrics
            .main_viewport
            .unwrap_or(content)
            .intersect(content);
        if smoke_control_visible(r, clip, ppp) {
            return true;
        }
        let tries = ctx.data_mut(|d| {
            let k = egui::Id::new(("packages-reveal", self.smoke_stage));
            let n = d.get_temp::<u8>(k).unwrap_or(0) + 1;
            d.insert_temp(k, n);
            n
        });
        assert!(
            tries <= 20,
            "package native control cannot become visible: {r:?}, clip {clip:?}"
        );
        let delta = if r.bottom() > clip.bottom() {
            clip.bottom() - r.bottom() - controls::GAP
        } else if r.top() < clip.top() {
            clip.top() - r.top() + controls::GAP
        } else {
            0.
        };
        let x = r.center().x.clamp(clip.left() + 1., clip.right() - 1.);
        self.pending_events.extend([
            egui::Event::PointerMoved(egui::pos2(x, clip.center().y)),
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
    fn package_lesson_click(&mut self, ctx: &egui::Context, t: &Target) -> bool {
        let phase = ctx
            .data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))))
            .unwrap_or(0);
        if phase >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        if matches!(t, Target::Choice(_)) {
            return self.mac_popup_choice(
                ctx,
                self.package_lesson_target(t),
                self.metrics.package_example_popup,
            );
        }
        let r = self
            .package_lesson_target(t)
            .expect("actual package control before click");
        self.package_lesson_reveal(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    fn package_lesson_resume(&mut self, ctx: &egui::Context, id: &str) -> bool {
        let phase = ctx
            .data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))))
            .unwrap_or(0);
        if phase >= 2 {
            let done = self.click_smoke_control(ctx, None);
            assert_eq!(self.scenarios[self.selected.unwrap()].id, id);
            return done;
        }
        assert!(self.selected.is_none());
        let matches: Vec<_> = self
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
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, id);
        assert_eq!(self.metrics.card_actions.len(), 1);
        let r = self.metrics.card_actions[0].0;
        self.package_lesson_reveal(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    pub(super) fn debian_packages_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert!((FIRST..END).contains(&stage));
        let p = plan(&self.scenarios);
        let offset = stage - FIRST;
        let step = p.get(offset).expect("closed package lesson phase");
        // Normal runtime must have the registered unit and exact four IDs; this
        // controller never appends a fixture or publishes a desired State.
        let u = self
            .units
            .iter()
            .find(|u| u.id == UNIT)
            .expect("normal package unit registered");
        assert_eq!(u.labs, IDS);
        let n = p[..=offset]
            .iter()
            .rev()
            .find_map(|s| {
                if let Step::Select(n) = s {
                    Some(*n)
                } else {
                    None
                }
            })
            .unwrap();
        let id = IDS[n];
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        let s = self.scenarios[i].clone();
        let lesson = s.debian_packages_teaching.as_ref().unwrap();
        if std::env::var_os("DLMS_SMOKE_DIAGNOSTICS").is_some() {
            eprintln!("packages-native {stage} {id} {step:?}");
        }
        match step {
            Step::Select(_) => {
                self.theme_id = "dlms-dark".into();
                self.scale = 1.;
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
                self.launch(i);
                assert_eq!(self.debian_packages_view(i), teaching::View::Introduction);
            }
            Step::Wait => {
                if self.selected.is_none() && self.filter.unit == UNIT {
                    assert_eq!(self.metrics.card_actions.len(), 4);
                }
            }
            Step::Click(t) => {
                if matches!(t, Target::Run) {
                    let key = egui::Id::new(("package-run-before", stage));
                    if ctx.data(|d| d.get_temp::<Value>(key)).is_none() {
                        let before =
                            serde_json::to_value(&self.progress.attempts[id].state).unwrap();
                        ctx.data_mut(|d| d.insert_temp(key, before));
                    }
                }
                if matches!(t, Target::Favorite) {
                    assert!(self.selected.is_none());
                    assert_eq!(self.filter.search, s.title);
                    assert_eq!(self.metrics.card_actions.len(), 1);
                }
                if !self.package_lesson_click(ctx, t) {
                    return (false, None);
                }
            }
            Step::Type(text) => {
                let (_, input, enabled) = self.metrics.package_command.unwrap();
                assert!(enabled);
                assert!(
                    ctx.memory(|m| m.has_focus(input)),
                    "real package command owns focus"
                );
                self.pending_events =
                    vec![key_a(true), egui::Event::Text(text.clone()), key_a(false)];
            }
            Step::Ran(text, practice, ok) => {
                let a = &self.progress.attempts[id];
                let t = session(a, *practice);
                let entry = t
                    .entries
                    .last()
                    .expect("actual Run appends terminal evidence");
                assert_eq!(entry.command, *text);
                assert_eq!(entry.status == 0, *ok, "{id}: {}", entry.stderr);
                assert_eq!(entry.diagnostic.is_none(), *ok);
                if *ok {
                    assert!(t.command.is_empty())
                } else {
                    assert_eq!(t.command, *text);
                    let before = ctx
                        .data(|d| {
                            d.get_temp::<Value>(egui::Id::new(("package-run-before", stage - 1)))
                        })
                        .unwrap();
                    assert_eq!(
                        serde_json::to_value(&a.state).unwrap(),
                        before,
                        "rejected command must preserve the full applied assignment state"
                    );
                }
                if *practice {
                    assert_eq!(a.state, s.initial);
                    assert!(!progress::current_solved(a, &s));
                }
            }
            Step::RecordInsert => {
                let a = &self.progress.attempts[id];
                let l = learning(a);
                let before = (
                    serde_json::to_value(&l.simulation).unwrap(),
                    serde_json::to_value(session(a, true)).unwrap(),
                    serde_json::to_value(&a.state).unwrap(),
                );
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("package-insert-before"), before));
            }
            Step::InsertOnly => {
                let before = ctx
                    .data(|d| {
                        d.get_temp::<(Value, Value, Value)>(egui::Id::new("package-insert-before"))
                    })
                    .unwrap();
                let a = &self.progress.attempts[id];
                let l = learning(a);
                assert_eq!(l.selected_action, 0);
                assert_eq!(
                    session(a, true).command,
                    lesson.example_actions[0].command.as_ref().unwrap().as_str()
                );
                assert_eq!(serde_json::to_value(&l.simulation).unwrap(), before.0);
                let mut wanted = before.1;
                wanted["command"] = session(a, true).command.clone().into();
                assert_eq!(serde_json::to_value(session(a, true)).unwrap(), wanted);
                assert_eq!(serde_json::to_value(&a.state).unwrap(), before.2);
                assert!(!progress::current_solved(a, &s));
            }
            Step::Practice(pass) => {
                let a = &self.progress.attempts[id];
                let l = learning(a);
                let rows = l.checks(lesson);
                let actual: Vec<_> = rows.iter().map(|r| r.0).collect();
                if *pass {
                    assert!(actual.iter().all(|v| *v), "{id}: {rows:?}");
                } else {
                    assert_eq!(actual, lesson.try_change.expected);
                    assert!(actual.iter().any(|v| !*v));
                }
                assert!(l.checked);
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, &s));
            }
            Step::Current(pass) => {
                let a = &self.progress.attempts[id];
                let rows = progress::current_debian_packages_checks(a, &s);
                assert_eq!(rows.iter().all(|r| r.0), *pass, "{id}: {rows:?}");
                assert_eq!(progress::current_solved(a, &s), *pass);
                assert_eq!(self.metrics.completion, *pass);
            }
            Step::Partial => {
                let a = &self.progress.attempts[id];
                let rows = progress::current_debian_packages_checks(a, &s);
                assert!(
                    rows.iter().any(|r| r.0) && rows.iter().any(|r| !r.0),
                    "{id}: {rows:?}"
                );
                assert!(!progress::current_solved(a, &s) && !self.metrics.completion);
            }
            Step::RecordReset => {
                let before: ResetRecord = (
                    serde_json::to_value(&self.progress.attempts[id]).unwrap(),
                    self.progress
                        .attempts
                        .iter()
                        .filter(|(k, _)| k.as_str() != id)
                        .map(|(k, a)| (k.clone(), serde_json::to_value(a).unwrap()))
                        .collect(),
                    self.progress.favorites.clone(),
                );
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("package-reset-before"), before));
            }
            Step::CancelExact => {
                let before = ctx
                    .data(|d| d.get_temp::<ResetRecord>(egui::Id::new("package-reset-before")))
                    .unwrap();
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[id]).unwrap(),
                    before.0
                );
                assert!(!self.reset_pending);
            }
            Step::ResetExact => {
                let a = &self.progress.attempts[id];
                assert_eq!(a.state, s.initial);
                assert!(a.transcript.is_empty());
                let w = a.workspace.as_ref().unwrap();
                let l = w.debian_packages_learning.as_ref().unwrap();
                assert_eq!(l.simulation, lesson.definition.initial);
                assert!(l.terminal.is_none());
                assert!(w.debian_packages_terminal.is_none());
                assert!(!progress::current_solved(a, &s));
                let before = ctx
                    .data(|d| d.get_temp::<ResetRecord>(egui::Id::new("package-reset-before")))
                    .unwrap();
                for (k, v) in before.1 {
                    assert_eq!(
                        serde_json::to_value(&self.progress.attempts[&k]).unwrap(),
                        v
                    );
                }
                assert_eq!(self.progress.favorites, before.2);
            }
            Step::SaveReadback => {
                self.save();
                assert!(self.storage_error.is_none());
                let bytes = progress::read_bounded(&self.path, 16_000_000).unwrap();
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                let portable = serde_json::to_value(&self.progress).unwrap();
                assert_eq!(serde_json::to_value(&loaded).unwrap(), portable);
                assert_eq!(
                    serde_json::to_value(&loaded.attempts).unwrap(),
                    serde_json::to_value(&self.progress.attempts).unwrap()
                );
                let mut full = portable;
                full["migration_receipts"] =
                    serde_json::to_value(&loaded.migration_receipts).unwrap();
                assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), full);
                assert_eq!(
                    progress::read_bounded(&self.path, 16_000_000).unwrap(),
                    bytes
                );
                self.progress = loaded;
            }
            Step::RecordDrafts => {
                let a = &self.progress.attempts[id];
                assert_eq!(session(a, false).command, ASSIGNMENT_DRAFT);
                assert_eq!(session(a, true).command, EXAMPLE_DRAFT);
                assert!(progress::current_solved(a, &s));
                let before = serde_json::to_value(a).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("package-resume-before"), before));
            }
            Step::Filter => {
                assert!(self.selected.is_none());
                self.filter = Filter::default();
                self.filter.search = s.title.clone();
                self.filter.max_minutes = 120;
                self.page = 0;
            }
            Step::Resume => {
                if !self.package_lesson_resume(ctx, id) {
                    return (false, None);
                }
            }
            Step::ResumeExact => {
                let mut before = ctx
                    .data(|d| d.get_temp::<Value>(egui::Id::new("package-resume-before")))
                    .unwrap();
                assert!(before["visited"].as_u64().unwrap() < self.progress.clock);
                before["visited"] = serde_json::json!(self.progress.clock);
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[id]).unwrap(),
                    before
                );
                assert_eq!(self.debian_packages_view(i), teaching::View::Example);
                assert_eq!(
                    session(&self.progress.attempts[id], true).command,
                    EXAMPLE_DRAFT
                );
                assert_eq!(
                    session(&self.progress.attempts[id], false).command,
                    ASSIGNMENT_DRAFT
                );
                assert!(self.progress.favorites.contains(id));
            }
            Step::Theme(light) => {
                self.theme_id = if *light { "dlms-light" } else { "dlms-dark" }.into();
                self.scale = if *light { 1.5 } else { 2. };
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(*light));
                if !light {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(946., 929.)));
                }
                self.lesson_scroll_reset = true;
            }
            Step::Geometry(scale) => {
                let zoom = ctx.zoom_factor();
                let ppp = ctx.pixels_per_point();
                let rect = ctx.content_rect();
                assert!(ppp.is_finite() && ppp > 0. && rect.is_finite());
                let key = egui::Id::new(("package-geometry", stage));
                let prior = ctx.data(|d| d.get_temp::<(u8, u32, u32, u32, u32)>(key));
                let sample = (
                    rect.width().to_bits(),
                    rect.height().to_bits(),
                    ppp.to_bits(),
                    zoom.to_bits(),
                );
                let tries = prior.map_or(1, |v| v.0 + 1);
                assert!(tries <= 20, "bounded package geometry/zoom did not settle");
                ctx.data_mut(|d| {
                    d.insert_temp(key, (tries, sample.0, sample.1, sample.2, sample.3))
                });
                if (zoom - *scale).abs() > 0.01
                    || prior.is_none_or(|v| (v.1, v.2, v.3, v.4) != sample)
                {
                    ctx.request_repaint();
                    return (false, None);
                }
                println!(
                    "package geometry {id}: content={rect:?}, ppp={ppp}, zoom={zoom}; requested window size is not an observed size"
                );
            }
            Step::Challenge => {
                assert!(self.progress.attempts[id].challenge);
                assert!(
                    self.metrics.lesson_tabs.is_empty()
                        && self.metrics.command_insert.is_none()
                        && self.metrics.example_action.is_none()
                );
                assert!(!self.metrics.help_visible);
                assert!(progress::current_solved(&self.progress.attempts[id], &s));
                assert!(self.metrics.completion);
            }
            Step::Capture(name, target) => {
                let rect = match target {
                    Shot::Heading => {
                        if self.selected.is_some() {
                            self.metrics.package_heading
                        } else {
                            None
                        }
                    }
                    Shot::Input => {
                        let (r, id, enabled) = self.metrics.package_command.unwrap();
                        assert!(enabled && ctx.memory(|m| m.has_focus(id)));
                        Some(r)
                    }
                    Shot::Feedback => self.metrics.package_result,
                };
                if let Some(r) = rect
                    && !self.package_lesson_reveal(ctx, r)
                {
                    return (false, None);
                }
                return (true, Some(name.clone()));
            }
            Step::Library => {
                assert!(self.selected.is_none());
                self.filter = Filter::default();
                self.filter.unit = UNIT.into();
                self.filter.max_minutes = 120;
                self.page = 0;
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
            Step::Finish => {
                assert!(IDS.iter().all(|id| {
                    let s = self.scenarios.iter().find(|s| s.id == *id).unwrap();
                    progress::current_solved(&self.progress.attempts[*id], s)
                }));
                assert_eq!(self.filter.unit, UNIT);
                assert_eq!(self.metrics.card_actions.len(), 4);
                println!(
                    "PASS four normal package lessons: actual Intro, isolated real Insert-only/Run/example change/repair, full assignment initial/partial/reference/alternative, current invalidation and retry, independent raw Sessions, exact saved readback before selected reset, real Back/Continue, Challenge, Light150/Dark200 focused typing. Fixed reviewed command inputs; no solved-State publication, real package utilities, cold-process, novice or platform claim."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return (false, None);
            }
        }
        (true, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn opened() -> (egui::Context, LabApp) {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context(&ctx, true, false);
        a.smoke = false;
        // Private trusted review input in headless tests only. Runtime plan()
        // requires the normal registered IDs/unit and never appends fixtures.
        let mut raw: Value =
            serde_json::from_str(include_str!("../../../scenarios/debian-packages-v1.json"))
                .unwrap();
        let metadata = reviews();
        for row in raw["scenarios"].as_array_mut().unwrap() {
            let r = metadata
                .iter()
                .find(|r| r.id == row["id"].as_str().unwrap())
                .unwrap();
            let actions = row["debian_packages_teaching"]["example_actions"]
                .as_array_mut()
                .unwrap();
            assert_eq!(actions.len(), r.annotation_commands.len());
            for (action, command) in actions.iter_mut().zip(&r.annotation_commands) {
                action["command"] = command.clone().into();
            }
        }
        let scenarios: Vec<Scenario> = serde_json::from_value(raw["scenarios"].clone()).unwrap();
        assert_eq!(scenarios.len(), 4);
        for s in scenarios {
            if let Some(existing) = a.scenarios.iter().find(|old| old.id == s.id) {
                assert_eq!(
                    serde_json::to_value(existing).unwrap(),
                    serde_json::to_value(&s).unwrap()
                );
            } else {
                a.scenarios.push(s);
            }
        }
        if let Some(unit) = a.units.iter().find(|u| u.id == UNIT) {
            assert_eq!(unit.track, "LPIC-1");
            assert_eq!(
                unit.labs,
                IDS.into_iter().map(str::to_string).collect::<Vec<_>>()
            );
        } else {
            a.units.push(dlms_learning_lab::course::Unit {
                id: UNIT.into(),
                track: "LPIC-1".into(),
                title: "Package administration review".into(),
                summary: "Private headless setup; normal registration is separate.".into(),
                prerequisites: vec![],
                labs: IDS.into_iter().map(str::to_string).collect(),
            });
        }
        a.smoke_stage = FIRST;
        (ctx, a)
    }
    fn frame(ctx: &egui::Context, a: &mut LabApp, drive: bool, router: bool) -> Option<String> {
        let events = std::mem::take(&mut a.pending_events);
        let modifiers = events
            .iter()
            .rev()
            .find_map(|e| match e {
                egui::Event::Key { modifiers, .. } => Some(*modifiers),
                _ => None,
            })
            .unwrap_or_default();
        let key = egui::Id::new("package-test-frame");
        let n = ctx.data(|d| d.get_temp::<u64>(key)).unwrap_or(0) + 1;
        ctx.data_mut(|d| d.insert_temp(key, n));
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(946. / 1.15 / a.scale, 929. / 1.15 / a.scale),
            )),
            time: Some(n as f64 / 20.),
            events,
            modifiers,
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(1.15);
        let mut shot = None;
        let _ = ctx.run(input, |ctx| {
            a.render(ctx);
            // Match native update ordering, including the autosave that made a
            // prior STP fixture setup's dirty-less save incorrectly appear valid.
            a.save();
            if drive {
                a.started = Instant::now() - std::time::Duration::from_secs(1);
                if router {
                    a.smoke_tick(ctx);
                } else {
                    let (advance, capture) = a.debian_packages_lessons_native(ctx, a.smoke_stage);
                    if advance {
                        a.smoke_stage += 1;
                    }
                    shot = capture;
                }
            }
        });
        shot
    }
    fn until(ctx: &egui::Context, a: &mut LabApp, target: usize, limit: usize) -> Vec<String> {
        let mut shots = vec![];
        for turn in 0..limit {
            if a.smoke_stage == target {
                return shots;
            }
            assert!(a.smoke_stage < target, "closed plan cannot skip phase");
            // Three real idle frames per controller tick expose sticky reveal
            // and autosave errors hidden by advancing immediately after render.
            if let Some(name) = frame(ctx, a, turn % 4 == 0, false) {
                shots.push(name);
            }
        }
        panic!(
            "package native phase {} did not reach {target}",
            a.smoke_stage
        );
    }
    #[test]
    fn package_native_plan_is_closed_bounded_and_covers_each_normal_task() {
        let (_, a) = opened();
        let p = plan(&a.scenarios);
        assert!(p.len() < END - FIRST);
        assert_eq!(p.iter().filter(|s| matches!(s, Step::Select(_))).count(), 4);
        for n in 0..4 {
            assert!(p.contains(&Step::Select(n)));
        }
        for kind in [
            Step::RecordInsert,
            Step::ResumeExact,
            Step::CancelExact,
            Step::Challenge,
        ] {
            assert_eq!(p.iter().filter(|s| **s == kind).count(), 4);
        }
        assert_eq!(
            p.iter().filter(|s| matches!(s, Step::ResetExact)).count(),
            8
        );
        assert_eq!(
            p.iter()
                .filter(|s| matches!(s, Step::Click(Target::Favorite)))
                .count(),
            4
        );
        assert_eq!(
            p.iter()
                .filter(|s| matches!(s, Step::Geometry(1.5)))
                .count(),
            4
        );
        assert_eq!(
            p.iter()
                .filter(|s| matches!(s, Step::Ran(_, false, false)))
                .count(),
            4
        );
        assert!(p.iter().all(|s| !matches!(s,Step::Type(c)if c.len()>2048)));
        assert_eq!(p.last(), Some(&Step::Finish));
        let production = include_str!("debian_packages_lessons.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        assert!(
            !production.contains("engine::execute")
                && !production.contains(".state =")
                && !production.contains("scenarios.push")
        );
    }
    #[test]
    fn package_native_actual_open_geometry_and_missing_registration_fail_closed() {
        let (ctx, mut a) = opened();
        a.smoke = true;
        frame(&ctx, &mut a, true, true);
        assert_eq!(a.smoke_stage, FIRST + 1);
        assert_eq!(
            a.debian_packages_view(a.selected.unwrap()),
            teaching::View::Introduction
        );
        assert!(a.metrics.package_command.is_none());
        // Invoke the actual geometry phase inside a Context frame. Reads happen
        // outside data closures and the second equal sample settles.
        let p = plan(&a.scenarios);
        let offset = p.iter().position(|s| *s == Step::Geometry(1.5)).unwrap();
        a.smoke_stage = FIRST + offset;
        a.scale = 1.5;
        for _ in 0..8 {
            frame(&ctx, &mut a, true, false);
            if a.smoke_stage > FIRST + offset {
                break;
            }
        }
        assert_eq!(a.smoke_stage, FIRST + offset + 1);
        let (ctx, mut missing) = opened();
        missing.units.retain(|u| u.id != UNIT);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = missing.debian_packages_lessons_native(&ctx, FIRST);
            }))
            .is_err()
        );
    }
    #[test]
    fn package_native_real_insert_run_check_and_idle_frames_allow_scrolling_back_to_input() {
        let (ctx, mut a) = opened();
        let p = plan(&a.scenarios);
        let stop = FIRST + p.iter().position(|s| *s == Step::Practice(true)).unwrap() + 1;
        until(&ctx, &mut a, stop, 2400);
        let l = learning(&a.progress.attempts[IDS[0]]);
        assert!(l.checked);
        assert!(
            l.checks(
                a.scenarios
                    .iter()
                    .find(|s| s.id == IDS[0])
                    .unwrap()
                    .debian_packages_teaching
                    .as_ref()
                    .unwrap()
            )
            .iter()
            .all(|r| r.0)
        );
        for _ in 0..12 {
            frame(&ctx, &mut a, false, false);
        }
        let mut revealed = false;
        for _ in 0..20 {
            let r = a.metrics.package_command.unwrap().0;
            if a.package_lesson_reveal(&ctx, r) {
                revealed = true;
                break;
            }
            for _ in 0..8 {
                frame(&ctx, &mut a, false, false);
            }
        }
        assert!(revealed);
        for _ in 0..16 {
            frame(&ctx, &mut a, false, false);
        }
        assert!(learning(&a.progress.attempts[IDS[0]]).checked);
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        assert!(
            smoke_control_visible(
                a.metrics.package_command.unwrap().0,
                clip,
                ctx.pixels_per_point()
            ),
            "displayed checked feedback must not keep forcing scroll on idle frames"
        );
    }
    #[test]
    fn package_native_all_four_real_flows_autosave_resume_reset_and_finish_hold() {
        let (ctx, mut a) = opened();
        let p = plan(&a.scenarios);
        let end = FIRST + p.len() - 1;
        let shots = until(&ctx, &mut a, end, 24000);
        for n in 0..4 {
            for suffix in [
                "example-complete",
                "example-experiment",
                "example-repaired",
                "partial",
                "complete",
                "light150-focused-input",
                "dark200-focused-input",
                "resumed-example-draft",
                "alternate",
                "challenge",
            ] {
                assert!(
                    shots.contains(&format!("packages-{n}-{suffix}")),
                    "task{n} missing {suffix}"
                );
            }
        }
        assert!(IDS.iter().all(|id| a.progress.favorites.contains(*id)));
        let disk = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&disk.attempts).unwrap(),
            serde_json::to_value(&a.progress.attempts).unwrap()
        );
        a.smoke = true;
        for _ in 0..2 {
            frame(&ctx, &mut a, true, true);
            assert_eq!(a.smoke_stage, end);
            assert!(a.pending_events.is_empty() && !a.capture_pending);
        }
    }
}
