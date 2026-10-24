//! Fixed, normal-bundle Find acceptance. Only actual input events execute commands.
use super::*;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

const START: usize = 21000;
const END: usize = 22000;
const IDS: [&str; 3] = [
    "lpic1-search-release-set",
    "lpic1-search-incomplete-intake",
    "lpic1-search-private-handover",
];
#[derive(Deserialize)]
struct Review {
    id: String,
    alternates: Vec<Vec<String>>,
    partial: Vec<String>,
    wrong: Vec<Wrong>,
    example: Experiment,
}
#[derive(Deserialize)]
struct Wrong {
    commands: Vec<String>,
}
#[derive(Deserialize)]
struct Experiment {
    experiment: Vec<String>,
    repair: Vec<String>,
    failed_rows: Vec<usize>,
}
#[derive(Clone, Debug, PartialEq)]
enum Target {
    Try,
    ExampleTab,
    Assignment,
    Input,
    Run,
    Insert,
    Picker,
    Choice(usize),
    Check,
    Reset,
    Confirm,
    Cancel,
    Challenge,
    Back,
}
#[derive(Clone, Debug, PartialEq)]
enum Step {
    Select(usize),
    Wait,
    Click(Target),
    Type(String),
    ExampleChoice(usize),
    RecordPractice,
    InsertOnly(usize),
    Ran(String, bool),
    Practice(bool),
    Current(bool),
    Partial,
    Record,
    CancelExact,
    ResetExact,
    SaveReadback,
    Drafts,
    DraftsExact(bool),
    Filter,
    Resume,
    ResumeExact,
    Theme(bool),
    Geometry(f32),
    Challenge,
    Shot(String, bool),
    LibraryFilter,
    Library,
    Finish,
}
fn commands(p: &mut Vec<Step>, commands: &[String], practice: bool) {
    for command in commands {
        p.extend([
            Step::Click(Target::Input),
            Step::Type(command.clone()),
            Step::Click(Target::Run),
            Step::Ran(command.clone(), practice),
        ]);
    }
}
fn checked(p: &mut Vec<Step>, check: Step, shot: String) {
    p.extend([
        Step::Click(Target::Check),
        Step::Wait,
        check,
        Step::Shot(shot, true),
    ]);
}
fn reset(p: &mut Vec<Step>) {
    p.extend([
        Step::Record,
        Step::Click(Target::Reset),
        Step::Click(Target::Confirm),
        Step::Wait,
        Step::ResetExact,
        Step::Click(Target::Assignment),
        Step::Wait,
    ]);
}
fn plan(scenarios: &[Scenario]) -> Vec<Step> {
    let reviews: Vec<Review> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/linux-find-content-review.json"
    ))
    .unwrap();
    let mut p = Vec::new();
    for (n, id) in IDS.iter().enumerate() {
        let s = scenarios
            .iter()
            .find(|s| s.id == *id)
            .expect("normal registered Find lesson");
        let lesson = s.terminal_teaching.as_ref().unwrap();
        let review = reviews.iter().find(|r| r.id == *id).unwrap();
        let name = |suffix: &str| format!("find-{n}-{suffix}");
        p.extend([
            Step::Select(n),
            Step::Wait,
            Step::Shot(name("introduction"), false),
            Step::Click(Target::Try),
            Step::Wait,
        ]);
        for (index, command) in lesson.example.iter().enumerate() {
            p.extend([
                Step::Click(Target::Picker),
                Step::Click(Target::Choice(index)),
                Step::ExampleChoice(index),
                Step::RecordPractice,
                Step::Click(Target::Insert),
                Step::InsertOnly(index),
                Step::Click(Target::Run),
                Step::Ran(command.clone(), true),
            ]);
        }
        p.extend([
            Step::Practice(true),
            Step::Shot(name("example-complete"), false),
        ]);
        commands(&mut p, &review.example.experiment, true);
        p.extend([
            Step::Practice(false),
            Step::Shot(name("example-experiment"), false),
        ]);
        commands(&mut p, &review.example.repair, true);
        p.extend([
            Step::Practice(true),
            Step::Shot(name("example-repaired"), false),
            Step::Click(Target::Assignment),
            Step::Wait,
            Step::Shot(name("assignment"), false),
        ]);
        commands(&mut p, &review.wrong[0].commands, false);
        checked(&mut p, Step::Current(false), name("misconception"));
        reset(&mut p);
        commands(&mut p, &review.partial, false);
        checked(&mut p, Step::Partial, name("partial"));
        reset(&mut p);
        commands(&mut p, &s.solution, false);
        checked(&mut p, Step::Current(true), name("complete"));
        p.push(Step::SaveReadback);
        // Exercise real typing and Run at both requested scales, without changing
        // the product layout or publishing a reference State.
        for light in [true, false] {
            p.extend([
                Step::Theme(light),
                Step::Geometry(if light { 1.5 } else { 2. }),
            ]);
            commands(&mut p, &["find /work -maxdepth 0".into()], false);
            checked(
                &mut p,
                Step::Current(true),
                name(if light {
                    "light150-typed"
                } else {
                    "dark200-typed"
                }),
            );
        }
        let engine::Goal::Filesystem { requirements } = &s.goal else {
            panic!("file goal")
        };
        let dlms_learning_lab::filesystem::Requirement::DirectoryEntries { path, .. } =
            &requirements[0]
        else {
            panic!("exact named handover folder")
        };
        commands(
            &mut p,
            &[format!("touch {path}/.unrequested-native")],
            false,
        );
        checked(&mut p, Step::Current(false), name("current-invalidated"));
        commands(&mut p, &[format!("rm {path}/.unrequested-native")], false);
        checked(&mut p, Step::Current(true), name("current-repaired"));
        p.extend([
            Step::Click(Target::Input),
            Step::Type("cp 'unfinished assignment".into()),
            Step::Click(Target::ExampleTab),
            Step::Wait,
            Step::Click(Target::Input),
            Step::Type("find . -name 'unfinished practice".into()),
            Step::Drafts,
            Step::SaveReadback,
            Step::Click(Target::Back),
            Step::Filter,
            Step::Wait,
            Step::Resume,
            Step::Wait,
            Step::ResumeExact,
            Step::DraftsExact(true),
            Step::Shot(name("resumed-practice-draft"), false),
            Step::Click(Target::Assignment),
            Step::Wait,
            Step::DraftsExact(false),
        ]);
        p.extend([
            Step::Record,
            Step::Click(Target::Reset),
            Step::Click(Target::Cancel),
            Step::CancelExact,
        ]);
        reset(&mut p);
        commands(&mut p, &review.alternates[0], false);
        checked(&mut p, Step::Current(true), name("manual-alternate"));
        p.extend([
            Step::SaveReadback,
            Step::Click(Target::Challenge),
            Step::Wait,
            Step::Challenge,
            Step::Shot(name("challenge"), false),
        ]);
    }
    p.extend([
        Step::Click(Target::Back),
        Step::LibraryFilter,
        Step::Geometry(1.),
        Step::Wait,
        Step::Library,
        Step::Shot("find-unit-library".into(), false),
        Step::Finish,
    ]);
    assert!(p.len() < END - START, "separate bounded range before22000");
    p
}
fn ctrl_a() -> egui::Event {
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
    }
}
impl LabApp {
    fn find_target(&self, t: &Target) -> Option<egui::Rect> {
        match t {
            Target::Try => self.metrics.example_action,
            Target::ExampleTab => self.metrics.lesson_tabs.get(1).copied(),
            Target::Assignment => self.metrics.assignment_action,
            Target::Input => self.metrics.terminal,
            Target::Run => self.metrics.terminal_run,
            Target::Insert => self.metrics.insert,
            Target::Picker => self.metrics.terminal_example_picker,
            Target::Choice(index) => self
                .metrics
                .terminal_example_choices
                .iter()
                .find(|(i, _)| i == index)
                .map(|(_, r)| *r),
            Target::Check => self.metrics.check,
            Target::Reset => self.metrics.reset,
            Target::Confirm => self.metrics.confirm_reset,
            Target::Cancel => self.metrics.reset_cancel,
            Target::Challenge => self.metrics.mode_challenge,
            Target::Back => self.metrics.back,
        }
    }
    fn find_start_card(&mut self, ctx: &egui::Context, id: &str) -> bool {
        let expected = self.scenarios.iter().position(|s| s.id == id).unwrap();
        let key = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0) >= 2 {
            let done = self.click_visible_smoke_control(ctx, None);
            assert_eq!(
                self.selected,
                Some(expected),
                "real Start must resume the requested lab"
            );
            return done;
        }
        assert!(self.selected.is_none());
        let matching: Vec<_> = self
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
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].id, id);
        assert_eq!(
            self.metrics.card_actions.len(),
            1,
            "one rendered real card before clicking"
        );
        self.click_visible_smoke_control(ctx, self.metrics.card_actions.first().map(|r| r.0))
    }
    pub(super) fn find_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert!((START..END).contains(&stage));
        let p = plan(&self.scenarios);
        let step = &p[stage - START];
        let n = p[..=stage - START]
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
        let l = s.terminal_teaching.as_ref().unwrap();
        if std::env::var_os("DLMS_SMOKE_DIAGNOSTICS").is_some() {
            eprintln!("find-native {stage} {id} {step:?}");
        }
        match step {
            Step::Select(_) => {
                self.theme_id = "dlms-dark".into();
                self.scale = 1.;
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
                self.launch(i);
                self.progress.favorites.insert(id.into());
                assert_eq!(self.terminal_view(i), teaching::View::Introduction);
            }
            Step::Wait => {}
            Step::Click(target) => {
                // Popup choices are expected only before dispatching their final
                // click; complete the helper even when the popup has closed.
                let ready = if matches!(target, Target::Choice(_)) {
                    self.mac_popup_choice(
                        ctx,
                        self.find_target(target),
                        self.metrics.terminal_example_popup,
                    )
                } else {
                    self.click_visible_smoke_control(ctx, self.find_target(target))
                };
                if !ready {
                    return (false, None);
                }
            }
            Step::Type(command) => {
                assert!(self.metrics.focus.is_some());
                assert_eq!(
                    ctx.memory(|m| m.focused()),
                    self.metrics.focus,
                    "clicked terminal owns keyboard focus"
                );
                self.pending_events = vec![ctrl_a(), egui::Event::Text(command.clone())];
            }
            Step::ExampleChoice(index) => assert_eq!(
                self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .terminal_learning
                    .as_ref()
                    .unwrap()
                    .example_command,
                *index
            ),
            Step::RecordPractice => {
                let a = &self.progress.attempts[id];
                let practice = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .terminal_learning
                    .as_ref()
                    .unwrap();
                let before = (
                    serde_json::to_value(&practice.tree).unwrap(),
                    practice.transcript.clone(),
                    serde_json::to_value(&a.state).unwrap(),
                );
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("find-native-practice"), before));
            }
            Step::InsertOnly(index) => {
                let before = ctx
                    .data(|d| {
                        d.get_temp::<(Value, Vec<String>, Value)>(egui::Id::new(
                            "find-native-practice",
                        ))
                    })
                    .unwrap();
                let a = &self.progress.attempts[id];
                let practice = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .terminal_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(practice.command, l.example[*index]);
                assert_eq!(serde_json::to_value(&practice.tree).unwrap(), before.0);
                assert_eq!(practice.transcript, before.1);
                assert_eq!(serde_json::to_value(&a.state).unwrap(), before.2);
                assert!(!progress::current_solved(a, &s));
            }
            Step::Ran(command, practice) => {
                let a = &self.progress.attempts[id];
                let transcript = if *practice {
                    let learning = a
                        .workspace
                        .as_ref()
                        .unwrap()
                        .terminal_learning
                        .as_ref()
                        .unwrap();
                    assert!(learning.command.is_empty());
                    assert_eq!(a.state, s.initial);
                    &learning.transcript
                } else {
                    assert!(self.input.is_empty());
                    &a.transcript
                };
                assert!(
                    transcript
                        .last()
                        .is_some_and(|line| line.starts_with(&format!("alex $ {command}"))),
                    "actual Run consumed the exact command"
                );
            }
            Step::Practice(pass) => {
                let a = &self.progress.attempts[id];
                let practice = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .terminal_learning
                    .as_ref()
                    .unwrap();
                let checks: Vec<_> = l.outcomes.iter().map(|r| r.check(&practice.tree)).collect();
                if *pass {
                    assert!(checks.iter().all(|c| c.0), "{id} {checks:?}");
                } else {
                    let reviews: Vec<Review> = serde_json::from_str(include_str!(
                        "../../../tests/fixtures/linux-find-content-review.json"
                    ))
                    .unwrap();
                    for row in &reviews
                        .iter()
                        .find(|r| r.id == id)
                        .unwrap()
                        .example
                        .failed_rows
                    {
                        assert!(!checks[*row].0);
                    }
                }
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, &s));
            }
            Step::Current(pass) => {
                let a = &self.progress.attempts[id];
                assert_eq!(progress::current_solved(a, &s), *pass);
                assert_eq!(self.metrics.completion, *pass);
                let checks = engine::checks(&a.state, &s.goal);
                assert_eq!(checks.iter().all(|c| c.0), *pass);
                if !pass {
                    assert!(!model::issues(&a.state, &s.goal).is_empty());
                }
            }
            Step::Partial => {
                let a = &self.progress.attempts[id];
                let checks = engine::checks(&a.state, &s.goal);
                assert!(checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0));
                assert!(!self.metrics.completion && !progress::current_solved(a, &s));
            }
            Step::Record => {
                let value = serde_json::to_value(&self.progress.attempts[id]).unwrap();
                let others: BTreeMap<_, _> = self
                    .progress
                    .attempts
                    .iter()
                    .filter(|(key, _)| key.as_str() != id)
                    .map(|(key, a)| (key.clone(), serde_json::to_value(a).unwrap()))
                    .collect();
                let favorites = self.progress.favorites.clone();
                ctx.data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new("find-native-reset"),
                        (value, others, favorites),
                    )
                });
            }
            Step::CancelExact => {
                let record = ctx
                    .data(|d| {
                        d.get_temp::<(
                            Value,
                            BTreeMap<String, Value>,
                            std::collections::BTreeSet<String>,
                        )>(egui::Id::new("find-native-reset"))
                    })
                    .unwrap();
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[id]).unwrap(),
                    record.0
                );
                assert!(!self.reset_pending);
            }
            Step::ResetExact => {
                let a = &self.progress.attempts[id];
                assert_eq!(a.state, s.initial);
                assert!(a.transcript.is_empty());
                let w = a.workspace.as_ref().unwrap();
                let practice = w.terminal_learning.as_ref().unwrap();
                assert_eq!(practice.tree, l.fixture);
                assert!(
                    practice.command.is_empty()
                        && practice.transcript.is_empty()
                        && w.command.is_empty()
                );
                assert!(!progress::current_solved(a, &s));
                let record = ctx
                    .data(|d| {
                        d.get_temp::<(
                            Value,
                            BTreeMap<String, Value>,
                            std::collections::BTreeSet<String>,
                        )>(egui::Id::new("find-native-reset"))
                    })
                    .unwrap();
                for (id, expected) in record.1 {
                    assert_eq!(
                        serde_json::to_value(&self.progress.attempts[&id]).unwrap(),
                        expected
                    );
                }
                assert_eq!(self.progress.favorites, record.2);
            }
            Step::SaveReadback => {
                self.save();
                assert!(self.storage_error.is_none());
                let bytes = progress::read_bounded(&self.path, 16_000_000).unwrap();
                let raw: Value = serde_json::from_slice(&bytes).unwrap();
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                let portable = serde_json::to_value(&self.progress).unwrap();
                assert_eq!(serde_json::to_value(&loaded).unwrap(), portable);
                // Main-file receipts are deliberately excluded from portable
                // Progress serialization. Compare their complete decoded value
                // explicitly and account for every raw key, without ignores.
                let mut exact = portable;
                exact["migration_receipts"] =
                    serde_json::to_value(&loaded.migration_receipts).unwrap();
                assert_eq!(raw, exact);
                assert_eq!(
                    progress::read_bounded(&self.path, 16_000_000).unwrap(),
                    bytes
                );
            }
            Step::Drafts => {
                let a = &self.progress.attempts[id];
                let w = a.workspace.as_ref().unwrap();
                assert_eq!(w.command, "cp 'unfinished assignment");
                assert_eq!(
                    w.terminal_learning.as_ref().unwrap().command,
                    "find . -name 'unfinished practice"
                );
                let before = serde_json::to_value(a).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("find-native-resume"), before));
            }
            Step::DraftsExact(_) | Step::ResumeExact => {
                let before = ctx
                    .data(|d| d.get_temp::<Value>(egui::Id::new("find-native-resume")))
                    .unwrap();
                let actual = serde_json::to_value(&self.progress.attempts[id]).unwrap();
                // Navigation may update the displayed view and visited clock;
                // all learner simulation/draft/history/credit fields stay exact.
                let mut wanted = before;
                assert!(wanted["visited"].as_u64().unwrap() < self.progress.clock);
                wanted["visited"] = serde_json::json!(self.progress.clock);
                let expected_view = if matches!(step, Step::DraftsExact(false)) {
                    teaching::View::Assignment
                } else {
                    teaching::View::Example
                };
                wanted["workspace"]["terminal_learning"]["view"] =
                    serde_json::to_value(expected_view).unwrap();
                assert_eq!(self.terminal_view(i), expected_view);
                assert_eq!(actual, wanted);
                assert_eq!(
                    self.progress.attempts[id]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .terminal_learning
                        .as_ref()
                        .unwrap()
                        .command,
                    "find . -name 'unfinished practice"
                );
                if matches!(step, Step::ResumeExact) {
                    assert_eq!(self.selected, Some(i));
                    assert_eq!(self.terminal_view(i), teaching::View::Example);
                }
            }
            Step::Filter => {
                assert!(self.selected.is_none());
                self.filter = Filter::default();
                self.filter.search = s.title.clone();
                self.filter.max_minutes = 120;
                self.page = 0;
            }
            Step::Resume => {
                if !self.find_start_card(ctx, id) {
                    return (false, None);
                }
            }
            Step::Theme(light) => {
                self.theme_id = if *light { "dlms-light" } else { "dlms-dark" }.into();
                self.scale = if *light { 1.5 } else { 2. };
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            Step::Geometry(scale) => {
                // Context reads happen before data closures (never nested locks).
                let zoom = ctx.zoom_factor();
                let pixels = ctx.pixels_per_point();
                let rect = ctx.content_rect();
                assert!(pixels.is_finite() && pixels > 0. && rect.is_finite());
                if (zoom - *scale).abs() > 0.01 {
                    let attempts = ctx.data_mut(|d| {
                        let key = egui::Id::new(("find-native-scale", stage));
                        let n = d.get_temp::<u8>(key).unwrap_or(0) + 1;
                        d.insert_temp(key, n);
                        n
                    });
                    assert!(attempts < 20, "bounded actual scale settling");
                    ctx.request_repaint();
                    return (false, None);
                }
            }
            Step::Challenge => {
                assert!(self.progress.attempts[id].challenge);
                assert!(self.metrics.lesson_tabs.is_empty() && self.metrics.insert.is_none());
                assert!(self.metrics.completion);
            }
            Step::Shot(name, feedback) => {
                let target = if *feedback {
                    self.metrics.feedback.map(|r| {
                        egui::Rect::from_min_size(
                            r.min,
                            egui::vec2(
                                r.width(),
                                r.height().min(
                                    self.metrics.check.map_or(r.height(), |c| 2. * c.height()),
                                ),
                            ),
                        )
                    })
                } else {
                    None
                };
                if let Some(target) = target
                    && !self.reveal_smoke_control(ctx, target)
                {
                    return (false, None);
                }
                return (true, Some(name.clone()));
            }
            Step::LibraryFilter => {
                assert!(self.selected.is_none());
                self.filter = Filter::default();
                self.filter.unit = "linux-file-search".into();
                self.filter.max_minutes = 120;
                self.page = 0;
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
            }
            Step::Library => {
                assert!(self.selected.is_none());
                assert!(self.filter.search.is_empty());
                assert_eq!(self.filter.unit, "linux-file-search");
                assert_eq!(
                    self.metrics.card_actions.len(),
                    3,
                    "all three actual unit cards rendered"
                );
                let unit = self
                    .units
                    .iter()
                    .find(|u| u.id == "linux-file-search")
                    .unwrap();
                assert_eq!(unit.labs, IDS);
            }
            Step::Finish => {
                assert!(IDS.iter().all(|id| {
                    let s = self.scenarios.iter().find(|s| s.id == *id).unwrap();
                    progress::current_solved(&self.progress.attempts[*id], s)
                }));
                println!(
                    "PASS normal three Find lessons: actual Intro, example picker/Insert-only/Run, experiment/repair, full assignment wrong/partial/reference/manual alternatives, current invalidation/repair, exact save/readback, independent drafts/resume, selected Reset/Cancel, Challenge, actual Light150/Dark200 typing. Fixed synthetic inputs only; no reference-State publication, cold-process, novice or other-platform claim."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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
            .rev()
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
                    egui::vec2(1200. / a.scale, 850. / a.scale),
                )),
                events,
                modifiers,
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
    }
    fn draw(ctx: &egui::Context, a: &mut LabApp) {
        for _ in 0..3 {
            frame(ctx, a, vec![]);
        }
    }
    fn ui_click(ctx: &egui::Context, a: &mut LabApp, target: Target) {
        for _ in 0..24 {
            let r = a.find_target(&target).expect("real terminal control");
            let viewport = a
                .metrics
                .main_viewport
                .unwrap()
                .intersect(ctx.content_rect());
            if smoke_control_visible(r, viewport, ctx.pixels_per_point()) {
                for pressed in [true, false] {
                    frame(
                        ctx,
                        a,
                        vec![
                            egui::Event::PointerMoved(r.center()),
                            egui::Event::PointerButton {
                                pos: r.center(),
                                button: egui::PointerButton::Primary,
                                pressed,
                                modifiers: Default::default(),
                            },
                        ],
                    );
                }
                draw(ctx, a);
                return;
            }
            let delta = if r.bottom() > viewport.bottom() {
                viewport.bottom() - r.bottom()
            } else {
                viewport.top() - r.top()
            };
            frame(
                ctx,
                a,
                vec![
                    egui::Event::PointerMoved(viewport.center()),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., delta),
                        modifiers: Default::default(),
                    },
                ],
            );
            draw(ctx, a);
        }
        panic!("actual control must become visible");
    }
    #[test]
    fn find_native_plan_is_bounded_separate_and_uses_real_controls_for_every_task() {
        let all = pack::bundled().unwrap();
        let p = plan(&all);
        assert!(START > 20099 && START + p.len() < END);
        assert_eq!(p.iter().filter(|s| matches!(s, Step::Select(_))).count(), 3);
        for light in [true, false] {
            assert_eq!(
                p.iter()
                    .filter(|s| matches!(s,Step::Theme(v) if *v==light))
                    .count(),
                3
            );
        }
        for id in IDS {
            assert!(all.iter().any(|s| s.id == id));
        }
        assert_eq!(p.iter().filter(|s| matches!(s, Step::Resume)).count(), 3);
        assert_eq!(
            p.iter().filter(|s| matches!(s, Step::ResetExact)).count(),
            9
        );
        assert_eq!(p.iter().filter(|s| matches!(s, Step::Challenge)).count(), 3);
        assert!(p.iter().all(|s| !matches!(s,Step::Type(x) if x.len()>4096)));
        let source = include_str!("find_lessons.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        assert!(!source.contains("engine::execute") && !source.contains(".state ="));
    }
    #[test]
    fn find_native_actual_geometry_phase_runs_without_context_lock_reentry() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context(&ctx, true, false);
        a.smoke = false;
        let p = plan(&a.scenarios);
        for scale in [1.5, 2.] {
            let offset = p
                .iter()
                .position(|s| matches!(s,Step::Geometry(v) if *v==scale))
                .unwrap();
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            a.smoke_stage = START + offset;
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                assert!(a.find_lessons_native(ctx, START + offset).0)
            });
        }
    }
    #[test]
    fn find_native_real_insert_fills_only_then_run_and_assignment_remain_independent() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context(&ctx, true, false);
        a.smoke = false;
        let i = a.scenarios.iter().position(|s| s.id == IDS[0]).unwrap();
        a.launch(i);
        draw(&ctx, &mut a);
        assert_eq!(a.terminal_view(i), teaching::View::Introduction);
        assert!(a.metrics.terminal.is_none());
        ui_click(&ctx, &mut a, Target::Try);
        assert_eq!(a.terminal_view(i), teaching::View::Example);
        let lesson = a.scenarios[i].terminal_teaching.as_ref().unwrap().clone();
        ui_click(&ctx, &mut a, Target::Insert);
        let practice = a.progress.attempts[IDS[0]]
            .workspace
            .as_ref()
            .unwrap()
            .terminal_learning
            .as_ref()
            .unwrap();
        assert_eq!(practice.command, lesson.example[0]);
        assert_eq!(practice.tree, lesson.fixture);
        assert!(practice.transcript.is_empty());
        ui_click(&ctx, &mut a, Target::Run);
        let practice = a.progress.attempts[IDS[0]]
            .workspace
            .as_ref()
            .unwrap()
            .terminal_learning
            .as_ref()
            .unwrap();
        assert!(practice.command.is_empty());
        assert_eq!(practice.transcript.len(), 1);
        assert!(practice.transcript[0].contains(".queue.log"));
        assert_eq!(a.progress.attempts[IDS[0]].state, a.scenarios[i].initial);
        ui_click(&ctx, &mut a, Target::Assignment);
        ui_click(&ctx, &mut a, Target::Input);
        frame(
            &ctx,
            &mut a,
            vec![
                ctrl_a(),
                egui::Event::Text("find /work/intake -maxdepth 0".into()),
            ],
        );
        ui_click(&ctx, &mut a, Target::Run);
        assert_eq!(a.progress.attempts[IDS[0]].transcript.len(), 1);
        ui_click(&ctx, &mut a, Target::Check);
        assert!(!a.metrics.completion);
        assert_eq!(
            a.progress.attempts[IDS[0]]
                .workspace
                .as_ref()
                .unwrap()
                .terminal_learning
                .as_ref()
                .unwrap()
                .transcript
                .len(),
            1
        );
    }
    #[test]
    fn find_native_focused_enter_cannot_execute_behind_reset_settings_or_data_modal() {
        for view in [teaching::View::Example, teaching::View::Assignment] {
            for blocked in ["reset", "settings", "data"] {
                let ctx = egui::Context::default();
                let mut a = LabApp::from_context(&ctx, true, false);
                a.smoke = false;
                let i = a.scenarios.iter().position(|s| s.id == IDS[0]).unwrap();
                a.launch(i);
                a.set_terminal_view(i, view);
                draw(&ctx, &mut a);
                ui_click(&ctx, &mut a, Target::Input);
                frame(
                    &ctx,
                    &mut a,
                    vec![
                        ctrl_a(),
                        egui::Event::Text("touch must-not-run-behind-dialog".into()),
                    ],
                );
                let field = a.metrics.focus.unwrap();
                match blocked {
                    "reset" => a.reset_pending = true,
                    "settings" => a.settings_open = true,
                    "data" => {
                        // The real chooser opens only the tool-created synthetic
                        // profile's parent, never a personal default folder.
                        a.smoke = true;
                        a.settings_open = true;
                        a.settings_data = true;
                        draw(&ctx, &mut a);
                        let export = a.metrics.data_export.expect("real Your data export action");
                        for pressed in [true, false] {
                            frame(
                                &ctx,
                                &mut a,
                                vec![
                                    egui::Event::PointerMoved(export.center()),
                                    egui::Event::PointerButton {
                                        pos: export.center(),
                                        button: egui::PointerButton::Primary,
                                        pressed,
                                        modifiers: Default::default(),
                                    },
                                ],
                            );
                        }
                        assert!(a.data_tools.modal_open());
                        a.settings_open = false;
                    }
                    _ => unreachable!(),
                }
                let before = serde_json::to_value(&a.progress.attempts[IDS[0]]).unwrap();
                ctx.memory_mut(|m| m.request_focus(field));
                frame(
                    &ctx,
                    &mut a,
                    vec![egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: Default::default(),
                    }],
                );
                assert_eq!(
                    serde_json::to_value(&a.progress.attempts[IDS[0]]).unwrap(),
                    before,
                    "{view:?}/{blocked}: background terminal must stay exact"
                );
                assert_eq!(
                    a.metrics.terminal_run_enabled,
                    Some(false),
                    "inactive Run must be rendered disabled"
                );
            }
        }
    }
    #[test]
    fn find_native_painted_run_labels_fit_practice_and_assignment_at_every_scale_and_width() {
        fn render(
            ctx: &egui::Context,
            a: &mut LabApp,
            w: f32,
            h: f32,
            events: Vec<egui::Event>,
        ) -> egui::FullOutput {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(w / a.scale, h / a.scale),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            )
        }
        fn text(shape: &egui::Shape, found: &mut Vec<egui::Rect>) {
            match shape {
                egui::Shape::Text(t) if t.galley.text() == "Run" => {
                    found.push(egui::Rect::from_min_size(t.pos, t.galley.size()))
                }
                egui::Shape::Vec(shapes) => {
                    for shape in shapes {
                        text(shape, found);
                    }
                }
                _ => {}
            }
        }
        for id in IDS {
            for scale in [1., 1.5, 2.] {
                for (w, h) in [(1200., 850.), (720., 560.)] {
                    for view in [teaching::View::Example, teaching::View::Assignment] {
                        let ctx = egui::Context::default();
                        let mut a = LabApp::from_context(&ctx, true, false);
                        a.smoke = false;
                        let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
                        a.launch(i);
                        a.set_terminal_view(i, view);
                        a.scale = scale;
                        for _ in 0..6 {
                            let _ = render(&ctx, &mut a, w, h, vec![]);
                        }
                        let mut visible = false;
                        for _ in 0..20 {
                            let r = a.metrics.terminal_run.expect("existing Run control");
                            let viewport = a
                                .metrics
                                .main_viewport
                                .unwrap()
                                .intersect(ctx.content_rect());
                            if smoke_control_visible(r, viewport, ctx.pixels_per_point()) {
                                visible = true;
                                break;
                            }
                            let delta = if r.bottom() > viewport.bottom() {
                                viewport.bottom() - r.bottom()
                            } else {
                                viewport.top() - r.top()
                            };
                            let _ = render(
                                &ctx,
                                &mut a,
                                w,
                                h,
                                vec![
                                    egui::Event::PointerMoved(egui::pos2(
                                        viewport.left() + 1.,
                                        viewport.center().y,
                                    )),
                                    egui::Event::MouseWheel {
                                        unit: egui::MouseWheelUnit::Point,
                                        delta: egui::vec2(0., delta),
                                        modifiers: Default::default(),
                                    },
                                ],
                            );
                            for _ in 0..4 {
                                let _ = render(&ctx, &mut a, w, h, vec![]);
                            }
                        }
                        assert!(visible, "actual Run visible at {id}/{view:?}/{scale}/{w}");
                        let output = render(&ctx, &mut a, w, h, vec![]);
                        let mut galleys = Vec::new();
                        for s in &output.shapes {
                            text(&s.shape, &mut galleys);
                        }
                        assert_eq!(galleys.len(), 1, "one actual painted Run");
                        let run = a.metrics.terminal_run.unwrap();
                        assert!(
                            run.contains_rect(galleys[0]),
                            "Run text fits actual Response: {galleys:?} vs {run:?}"
                        );
                        let input = a.metrics.terminal.unwrap();
                        assert!(
                            (run.center().y - input.center().y).abs()
                                <= 1. / ctx.pixels_per_point()
                        );
                        assert!(input.right() < run.left());
                    }
                }
            }
        }
    }
}
