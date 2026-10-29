//! Closed actual-input acceptance. Reviewed public text is trusted input, not
//! a novice study or permission to install solved State or execute host commands.
use super::*;
use dlms_learning_lab::{
    apache_static::{self as web, Action, Method, Outcome},
    apache_static_workspace::{self as work, Draft, EditorKind, Requirement},
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
pub(in crate::ui) const FIRST: usize = 36000;
const END: usize = 37000;
const UNIT: &str = "static-web-services";
const IDS: [&str; 4] = [
    "lpic2-web-routing",
    "lpic2-web-pages",
    "lpic2-web-access",
    "lpic2-web-handover",
];
const RAW: &str = "Listen 8080\n<VirtualHost *:8080>\n";
const ASSIGNMENT_DRAFT: &str = "unfinished assignment document\n";
const PRACTICE_DRAFT: &str = "different unfinished gallery document\n";
#[derive(Clone, Debug)]
enum Target {
    Control(&'static str),
    Example,
    Assignment,
    File(String),
    Back,
    Favorite,
    Reset,
    Cancel,
    Confirm,
    Challenge,
}
#[derive(Clone, Debug)]
enum Shot {
    Heading,
    Result,
    Field(&'static str),
    Response,
}
#[derive(Clone, Debug)]
enum Step {
    Select(usize),
    Open,
    Wait,
    Click(Target),
    Field(&'static str, String),
    SelectedFile(String),
    Ack(Action, bool),
    RecordInsert,
    InsertOnly,
    Practice(bool),
    Current(bool),
    Partial,
    RecordApplied,
    ReloadRejected,
    RecordReadOnly,
    ReadOnly,
    RecordDrafts,
    SaveReadback,
    Filter,
    ResumeExact,
    RecordReset,
    CancelExact,
    ResetExact,
    Theme(bool),
    Geometry(f32),
    Challenge,
    Capture(String, Shot),
    Library,
    Finish,
}
fn parse_reference(s: &Scenario) -> Vec<Action> {
    assert!(!s.solution.is_empty() && s.solution.len() <= 8);
    s.solution
        .iter()
        .map(|c| {
            assert!(c.len() <= web::MAX_CONFIG + 256);
            let a: Action =
                serde_json::from_str(c.strip_prefix(work::PREFIX).expect("reviewed typed input"))
                    .unwrap();
            a.validate().unwrap();
            a
        })
        .collect()
}
fn actions(p: &mut Vec<Step>, actions: &[Action], practice: bool) {
    assert!(!actions.is_empty() && actions.len() <= 32);
    for action in actions {
        match action {
            Action::SaveConfig { text } => p.extend([
                Step::Click(Target::Control("config-tab")),
                Step::Field("config", text.clone()),
                Step::Click(Target::Control("save-config")),
            ]),
            Action::WriteFile { path, text } => p.extend([
                Step::Click(Target::Control("file-tab")),
                Step::Click(Target::Control("file-picker")),
                Step::Click(Target::File(path.clone())),
                Step::SelectedFile(path.clone()),
                Step::Field("file", text.clone()),
                Step::Click(Target::Control("save-file")),
            ]),
            Action::Start {} => p.push(Step::Click(Target::Control("start"))),
            Action::Reload {} => p.push(Step::Click(Target::Control("reload"))),
            Action::Stop {} => p.push(Step::Click(Target::Control("stop"))),
        }
        p.push(Step::Ack(action.clone(), practice));
    }
}
fn checked(p: &mut Vec<Step>, predicate: Step) {
    p.extend([Step::Click(Target::Control("check")), Step::Wait, predicate]);
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
fn alternate(reference: &[Action]) -> Vec<Action> {
    let mut out = reference
        .iter()
        .filter(|a| matches!(a, Action::WriteFile { .. }))
        .cloned()
        .rev()
        .collect::<Vec<_>>();
    let text = reference
        .iter()
        .find_map(|a| {
            if let Action::SaveConfig { text } = a {
                Some(text)
            } else {
                None
            }
        })
        .unwrap();
    out.extend([
        Action::SaveConfig {
            text: format!(
                "# equivalent supported spelling\n\n{}\n",
                text.replace("ServerName ", "servername ")
                    .replace("Require all ", "require all ")
                    .replace("Listen 8080", "listen 8080")
            ),
        },
        Action::Stop {},
        Action::Start {},
    ]);
    out
}
fn public_request(s: &Scenario) -> web::Request {
    let engine::Goal::ApacheStatic { definition } = &s.goal else {
        panic!("Apache goal")
    };
    definition
        .requirements
        .iter()
        .find_map(|r| {
            if let Requirement::CurrentHttp { request, expected } = r {
                (expected.status == 200).then(|| request.clone())
            } else {
                None
            }
        })
        .unwrap()
}
fn pending(p: &mut Vec<Step>, file: &str, text: &str) {
    p.extend([
        Step::Click(Target::Control("config-tab")),
        Step::Field("config", text.into()),
        Step::Click(Target::Control("file-tab")),
        Step::Click(Target::Control("file-picker")),
        Step::Click(Target::File(file.into())),
        Step::SelectedFile(file.into()),
        Step::Field("file", format!("{text}file draft\n")),
        Step::Field("port", "bad!".into()),
    ]);
}
fn plan(scenarios: &[Scenario]) -> Vec<Step> {
    let mut p = vec![];
    for (n, id) in IDS.iter().enumerate() {
        let s = scenarios
            .iter()
            .find(|s| s.id == *id)
            .expect("normal Apache IDs, no developer replacement");
        assert_eq!(s.unit, UNIT);
        assert!(matches!(s.initial, engine::State::ApacheStatic { .. }));
        let l = s.apache_static_teaching.as_ref().unwrap();
        let reference = parse_reference(s);
        let alt = alternate(&reference);
        let name = |suffix: &str| format!("apache-{n}-{suffix}");
        p.extend([
            Step::Select(n),
            Step::Geometry(1.),
            Step::Open,
            Step::Wait,
            Step::Capture(name("introduction"), Shot::Heading),
            Step::Click(Target::Control("try-example")),
            Step::Wait,
            Step::RecordInsert,
            Step::Click(Target::Control("insert")),
            Step::InsertOnly,
            Step::Capture(name("fill-only"), Shot::Field("config")),
            Step::Click(Target::Control("save-config")),
            Step::Ack(l.example_actions[0].action.clone(), true),
        ]);
        assert!(matches!(
            l.example_actions[0].action,
            Action::SaveConfig { .. }
        ));
        actions(
            &mut p,
            &l.example_actions[1..]
                .iter()
                .map(|s| s.action.clone())
                .collect::<Vec<_>>(),
            true,
        );
        checked(&mut p, Step::Practice(true));
        p.push(Step::Capture(name("example-complete"), Shot::Result));
        actions(&mut p, &l.try_change.actions, true);
        checked(&mut p, Step::Practice(false));
        p.push(Step::Capture(name("example-experiment"), Shot::Result));
        actions(&mut p, &l.repair, true);
        checked(&mut p, Step::Practice(true));
        p.extend([
            Step::Click(Target::Assignment),
            Step::Wait,
            Step::Capture(name("assignment"), Shot::Heading),
        ]);
        checked(&mut p, Step::Current(false));
        p.push(Step::Capture(name("initial-feedback"), Shot::Result));
        actions(&mut p, &reference[..1], false);
        checked(&mut p, Step::Partial);
        p.push(Step::Capture(name("saved-only"), Shot::Result));
        reset(&mut p);
        actions(&mut p, &reference, false);
        checked(&mut p, Step::Current(true));
        p.push(Step::Capture(name("reference"), Shot::Result));
        let q = public_request(s);
        assert_eq!(q.method, Method::Get);
        assert_eq!(q.hosts.len(), 1);
        p.extend([
            Step::RecordReadOnly,
            Step::Field("port", q.port.to_string()),
            Step::Field("host-0", q.hosts[0].clone()),
            Step::Field("path", q.path.clone()),
            Step::Click(Target::Control("send-request")),
            Step::ReadOnly,
            Step::Capture(name("virtual-request"), Shot::Response),
            Step::Field("port", "bad!".into()),
            Step::Click(Target::Control("send-request")),
            Step::ReadOnly,
        ]);
        actions(&mut p, &[Action::SaveConfig { text: RAW.into() }], false);
        checked(&mut p, Step::Partial);
        p.extend([
            Step::RecordApplied,
            Step::Click(Target::Control("reload")),
            Step::ReloadRejected,
        ]);
        checked(&mut p, Step::Partial);
        p.push(Step::Capture(name("invalid-reload"), Shot::Result));
        actions(&mut p, &reference, false);
        checked(&mut p, Step::Current(true));
        reset(&mut p);
        actions(&mut p, &alt, false);
        checked(&mut p, Step::Current(true));
        p.push(Step::Capture(name("alternate"), Shot::Result));
        for light in [true, false] {
            p.extend([
                Step::Theme(light),
                Step::Geometry(if light { 1.5 } else { 2. }),
            ]);
            let edits = if light { &reference } else { &alt };
            actions(&mut p, edits, false);
            checked(&mut p, Step::Current(true));
            let text = edits
                .iter()
                .find_map(|a| {
                    if let Action::SaveConfig { text } = a {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
                .unwrap();
            p.extend([
                Step::Click(Target::Control("config-tab")),
                Step::Field("config", text),
                Step::Capture(
                    name(if light {
                        "light150-focused"
                    } else {
                        "dark200-focused"
                    }),
                    Shot::Field("config"),
                ),
                Step::Click(Target::Control("save-config")),
                Step::Click(Target::Control("reload")),
            ]);
            checked(&mut p, Step::Current(true));
            p.push(Step::Capture(
                name(if light {
                    "light150-complete"
                } else {
                    "dark200-complete"
                }),
                Shot::Result,
            ));
        }
        let file = public_request(s);
        let engine::Goal::ApacheStatic { definition } = &s.goal else {
            unreachable!()
        };
        let path = definition
            .requirements
            .iter()
            .find_map(|r| {
                if let Requirement::CurrentHttp { request, expected } = r {
                    (request == &file)
                        .then(|| expected.resource.clone())
                        .flatten()
                } else {
                    None
                }
            })
            .unwrap();
        pending(&mut p, &path, ASSIGNMENT_DRAFT);
        p.extend([Step::Click(Target::Example), Step::Wait]);
        let example_file = l
            .definition
            .initial
            .inventory
            .entries
            .iter()
            .find_map(|(p, k)| (*k == web::EntryKind::File).then(|| p.clone()))
            .unwrap();
        pending(&mut p, &example_file, PRACTICE_DRAFT);
        p.extend([
            Step::RecordDrafts,
            Step::SaveReadback,
            Step::Click(Target::Back),
            Step::Filter,
            Step::Wait,
            Step::Click(Target::Favorite),
            Step::SaveReadback,
            Step::Open,
            Step::Wait,
            Step::ResumeExact,
            Step::Capture(name("resumed-drafts"), Shot::Field("file")),
            Step::Click(Target::Assignment),
            Step::Wait,
            Step::RecordReset,
            Step::Click(Target::Reset),
            Step::Click(Target::Cancel),
            Step::CancelExact,
        ]);
        reset(&mut p);
        actions(&mut p, &reference, false);
        checked(&mut p, Step::Current(true));
        p.extend([
            Step::SaveReadback,
            Step::Click(Target::Challenge),
            Step::Wait,
            Step::Challenge,
            Step::Capture(name("challenge"), Shot::Heading),
            Step::Click(Target::Back),
        ]);
    }
    p.extend([Step::Library, Step::Wait, Step::Finish]);
    assert!(p.len() < END - FIRST, "closed Apache stage range");
    p
}
fn draft(a: &Attempt, practice: bool) -> &Draft {
    let w = a.workspace.as_ref().unwrap();
    if practice {
        &w.apache_static_learning.as_ref().unwrap().draft
    } else {
        w.apache_static_draft.as_ref().unwrap()
    }
}
fn simulation(a: &Attempt, practice: bool) -> &web::State {
    if practice {
        &a.workspace
            .as_ref()
            .unwrap()
            .apache_static_learning
            .as_ref()
            .unwrap()
            .simulation
    } else {
        let engine::State::ApacheStatic { simulation } = &a.state else {
            panic!("Apache State")
        };
        simulation
    }
}
fn field<'a>(d: &'a Draft, key: &str) -> &'a str {
    match key {
        "config" => &d.config_text,
        "file" => &d.file_text,
        "port" => &d.request_port,
        "path" => &d.request_path,
        "host-0" => d.request_hosts.first().map_or("", String::as_str),
        _ => panic!("closed field"),
    }
}
fn same(a: &impl serde::Serialize, b: &impl serde::Serialize, label: &'static str) {
    assert!(
        serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap(),
        "{label}: complete fields differ, values withheld"
    );
}
type ResetRecord = (Value, BTreeMap<String, Value>, BTreeSet<String>);
fn key_a(pressed: bool) -> egui::Event {
    egui::Event::Key {
        key: egui::Key::A,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: if pressed {
            egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            }
        } else {
            Default::default()
        },
    }
}
impl LabApp {
    fn apache_lesson_reveal(&mut self, ctx: &egui::Context, r: egui::Rect) -> bool {
        let rect = ctx.content_rect();
        let ppp = ctx.pixels_per_point();
        let clip = self.metrics.main_viewport.unwrap_or(rect).intersect(rect);
        if smoke_control_visible(r, clip, ppp) {
            return true;
        }
        let k = egui::Id::new(("apache-reveal", self.smoke_stage));
        let tries = ctx.data_mut(|d| {
            let n = d.get_temp::<u8>(k).unwrap_or(0) + 1;
            d.insert_temp(k, n);
            n
        });
        assert!(
            tries <= 20,
            "Apache target {r:?} outside actual clip {clip:?}"
        );
        let dy = if r.top() < clip.top() {
            clip.top() - r.top() + controls::GAP
        } else if r.bottom() > clip.bottom() {
            clip.bottom() - r.bottom() - controls::GAP
        } else {
            0.
        };
        self.pending_events.extend([
            // The main viewport gutter is outside both multiline editors and
            // the response's inner ScrollAreas, so only the outer page scrolls.
            egui::Event::PointerMoved(egui::pos2(clip.left() + 1., clip.center().y)),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., dy.clamp(-400., 400.)),
                modifiers: Default::default(),
            },
        ]);
        self.started = Instant::now();
        ctx.request_repaint();
        false
    }
    fn apache_lesson_clicked(&self, ctx: &egui::Context) -> bool {
        ctx.data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))))
            .unwrap_or(0)
            >= 2
    }
    fn apache_lesson_control(&self, target: &Target) -> Option<egui::Rect> {
        match target {
            Target::Control(key) => self
                .metrics
                .apache_controls
                .iter()
                .find(|(k, _, _)| k == key)
                .map(|(_, r, _)| *r),
            Target::Example => self.metrics.lesson_tabs.get(1).copied(),
            Target::Assignment => self
                .metrics
                .assignment_action
                .or_else(|| self.metrics.lesson_tabs.get(2).copied()),
            Target::File(path) => self
                .metrics
                .apache_file_items
                .iter()
                .find(|(p, _)| p == path)
                .map(|(_, r)| *r),
            Target::Back => self.metrics.back,
            Target::Favorite => self.metrics.card_actions.first().map(|r| r.1),
            Target::Reset => self.metrics.reset,
            Target::Cancel => self.metrics.reset_cancel,
            Target::Confirm => self.metrics.confirm_reset,
            Target::Challenge => self.metrics.mode_challenge,
        }
    }
    fn apache_lesson_click(&mut self, ctx: &egui::Context, t: &Target) -> bool {
        if self.apache_lesson_clicked(ctx) {
            return self.click_smoke_control(ctx, None);
        }
        let rect = self.apache_lesson_control(t);
        if matches!(t, Target::File(_)) {
            return self.mac_popup_choice(
                ctx,
                rect,
                self.metrics
                    .apache_popup
                    .as_ref()
                    .filter(|(k, _)| k == "file")
                    .map(|(_, r)| *r),
            );
        }
        let r = rect.expect("actual Apache control");
        self.apache_lesson_reveal(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    fn apache_lesson_open(&mut self, ctx: &egui::Context, id: &str) -> bool {
        if self.apache_lesson_clicked(ctx) {
            let done = self.click_smoke_control(ctx, None);
            assert_eq!(self.scenarios[self.selected.unwrap()].id, id);
            return done;
        }
        assert!(self.selected.is_none());
        let matching = self
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
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].id, id);
        assert_eq!(self.metrics.card_actions.len(), 1);
        let r = self.metrics.card_actions[0].0;
        self.apache_lesson_reveal(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    fn apache_lesson_type(&mut self, ctx: &egui::Context, i: usize, key: &str, text: &str) -> bool {
        let cap = match key {
            "config" => web::MAX_CONFIG,
            "file" => web::MAX_FILE,
            "port" => 8,
            "path" => 256,
            "host-0" => 260,
            _ => panic!("closed editor"),
        };
        assert!(text.len() <= cap);
        let k = egui::Id::new(("apache-type", self.smoke_stage));
        let phase = ctx.data(|d| d.get_temp::<u8>(k)).unwrap_or(0);
        let (_, r, id, enabled) = self
            .metrics
            .apache_fields
            .iter()
            .find(|(k, _, _, _)| k == key)
            .expect("actual selected editor");
        let (r, id, enabled) = (*r, *id, *enabled);
        assert!(enabled);
        match phase {
            0 => {
                let target = if matches!(key, "config" | "file") {
                    let (_, v, _, _) = self.metrics.apache_editor.unwrap();
                    egui::Rect::from_center_size(
                        v.center(),
                        egui::vec2(
                            v.width(),
                            ctx.style().spacing.interact_size.y.min(v.height()),
                        ),
                    )
                } else {
                    r
                };
                if !self.apache_lesson_clicked(ctx) && !self.apache_lesson_reveal(ctx, target) {
                    return false;
                }
                if !self.click_smoke_control(ctx, Some(target)) {
                    return false;
                }
                assert!(ctx.memory(|m| m.has_focus(id)));
                ctx.data_mut(|d| d.insert_temp(k, 1u8));
                false
            }
            1 => {
                assert!(ctx.memory(|m| m.has_focus(id)));
                self.pending_events =
                    vec![key_a(true), egui::Event::Text(text.into()), key_a(false)];
                ctx.data_mut(|d| d.insert_temp(k, 2u8));
                false
            }
            2 => {
                let practice = self.apache_static_view(i) == teaching::View::Example;
                assert!(
                    field(
                        draft(&self.progress.attempts[&self.scenarios[i].id], practice),
                        key
                    ) == text,
                    "actual editor retains exact typed bytes"
                );
                assert!(ctx.memory(|m| m.has_focus(id)));
                true
            }
            _ => unreachable!(),
        }
    }
    pub(in crate::ui) fn apache_static_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert!((FIRST..END).contains(&stage));
        assert_eq!(
            self.scenarios.len(),
            171,
            "normal143 registration is a runtime prerequisite"
        );
        assert!(self._smoke_profile.is_some());
        assert_eq!(
            self.path.parent(),
            Some(self._smoke_profile.as_ref().unwrap().path())
        );
        let unit = self
            .units
            .iter()
            .find(|u| u.id == UNIT)
            .expect("normal Apache unit, never a developer fixture");
        assert_eq!(unit.labs, IDS);
        let p = plan(&self.scenarios);
        let offset = stage - FIRST;
        let step = p.get(offset).expect("closed Apache phase");
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
        let lesson = s.apache_static_teaching.as_ref().unwrap();
        let practice =
            self.selected == Some(i) && self.apache_static_view(i) == teaching::View::Example;
        if std::env::var_os("DLMS_SMOKE_DIAGNOSTICS").is_some() {
            eprintln!("Apache phase {stage} {id}");
        }
        match step {
            Step::Select(_) => {
                assert!(self.selected.is_none());
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
                self.filter = Filter::default();
                self.filter.search = s.title.clone();
                self.filter.max_minutes = 120;
                self.page = 0;
            }
            Step::Open => {
                if !self.apache_lesson_open(ctx, id) {
                    return (false, None);
                }
            }
            Step::Wait => {}
            Step::Click(t) => {
                if matches!(t, Target::Favorite) {
                    assert!(self.selected.is_none());
                    assert_eq!(self.filter.search, s.title);
                    assert_eq!(self.metrics.card_actions.len(), 1);
                }
                if !self.apache_lesson_click(ctx, t) {
                    return (false, None);
                }
            }
            Step::Field(key, text) => {
                if !self.apache_lesson_type(ctx, i, key, text) {
                    return (false, None);
                }
            }
            Step::SelectedFile(path) => {
                assert_eq!(
                    draft(&self.progress.attempts[id], practice)
                        .selected_file
                        .as_ref(),
                    Some(path)
                );
                assert_eq!(
                    draft(&self.progress.attempts[id], practice).editor_kind,
                    Some(EditorKind::File)
                );
            }
            Step::Ack(action, was_practice) => {
                assert_eq!(practice, *was_practice);
                let a = &self.progress.attempts[id];
                let state = simulation(a, practice);
                match action {
                    Action::SaveConfig { text } => assert!(state.saved == *text),
                    Action::WriteFile { path, text } => {
                        assert!(state.files.get(path) == Some(text))
                    }
                    Action::Reload {} | Action::Start {} => assert!(
                        state
                            .active
                            .as_ref()
                            .is_some_and(|a| a.source == state.saved)
                    ),
                    Action::Stop {} => assert!(state.active.is_none()),
                }
                if practice {
                    same(&a.state, &s.initial, "practice retains assignment");
                    assert!(!progress::current_solved(a, &s));
                }
            }
            Step::RecordInsert => {
                let a = &self.progress.attempts[id];
                let v = (
                    serde_json::to_value(simulation(a, true)).unwrap(),
                    serde_json::to_value(&a.state).unwrap(),
                );
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("apache-insert"), v));
            }
            Step::InsertOnly => {
                let a = &self.progress.attempts[id];
                let v = ctx
                    .data(|d| d.get_temp::<(Value, Value)>(egui::Id::new("apache-insert")))
                    .unwrap();
                same(simulation(a, true), &v.0, "Insert does not execute");
                same(&a.state, &v.1, "Insert does not grade assignment");
                let Action::SaveConfig { text } = &lesson.example_actions[0].action else {
                    unreachable!()
                };
                assert!(draft(a, true).config_text == *text);
                assert!(!progress::current_solved(a, &s));
            }
            Step::Practice(pass) => {
                let a = &self.progress.attempts[id];
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .apache_static_learning
                    .as_ref()
                    .unwrap();
                assert!(l.checked);
                let rows = l.checks(lesson);
                if *pass {
                    assert!(
                        rows.iter().all(|r| r.0),
                        "practice requires every actual row"
                    );
                } else {
                    assert!(rows[0].0);
                    same(
                        &rows[1..].iter().map(|r| r.0).collect::<Vec<_>>(),
                        &lesson.try_change.expected,
                        "practice experiment rows",
                    );
                }
                same(&a.state, &s.initial, "practice isolation");
                assert!(!progress::current_solved(a, &s));
            }
            Step::Current(pass) => {
                let a = &self.progress.attempts[id];
                let rows = progress::current_apache_static_checks(a, &s);
                assert_eq!(rows.iter().all(|r| r.0), *pass);
                assert_eq!(progress::current_solved(a, &s), *pass);
                assert_eq!(self.metrics.completion, *pass);
            }
            Step::Partial => {
                let a = &self.progress.attempts[id];
                let rows = progress::current_apache_static_checks(a, &s);
                assert!(rows.iter().any(|r| r.0) && rows.iter().any(|r| !r.0));
                assert!(!progress::current_solved(a, &s) && !self.metrics.completion);
            }
            Step::RecordApplied => {
                let v = serde_json::to_value(&self.progress.attempts[id].state).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("apache-applied"), v));
            }
            Step::ReloadRejected => {
                let v = ctx
                    .data(|d| d.get_temp::<Value>(egui::Id::new("apache-applied")))
                    .unwrap();
                same(
                    &self.progress.attempts[id].state,
                    &v,
                    "invalid Reload preserves exact saved and active state",
                );
                assert_eq!(simulation(&self.progress.attempts[id], false).saved, RAW);
                assert!(draft(&self.progress.attempts[id], false).notice.is_some());
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
            }
            Step::RecordReadOnly => {
                let v = serde_json::to_value(&self.progress.attempts[id].state).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("apache-query"), v));
            }
            Step::ReadOnly => {
                let a = &self.progress.attempts[id];
                let v = ctx
                    .data(|d| d.get_temp::<Value>(egui::Id::new("apache-query")))
                    .unwrap();
                same(&a.state, &v, "request cannot change applied server");
                assert!(progress::current_solved(a, &s) && self.metrics.completion);
                let q = draft(a, false).query(simulation(a, false));
                if draft(a, false).request_port == "bad!" {
                    assert!(q.is_err());
                } else {
                    assert!(matches!(q,Ok(Outcome::Http(r))if r.status.code()==200));
                }
                assert!(self.metrics.apache_response.is_some());
            }
            Step::RecordDrafts => {
                let a = &self.progress.attempts[id];
                assert_eq!(draft(a, false).config_text, ASSIGNMENT_DRAFT);
                assert_eq!(draft(a, true).config_text, PRACTICE_DRAFT);
                for pr in [false, true] {
                    assert_eq!(draft(a, pr).editor_kind, Some(EditorKind::File));
                    assert_eq!(draft(a, pr).request_port, "bad!");
                    assert!(!draft(a, pr).matches(simulation(a, pr)));
                }
                assert!(!progress::current_solved(a, &s));
                let v = serde_json::to_value(a).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("apache-resume"), v));
            }
            Step::SaveReadback => {
                self.save();
                assert!(self.storage_error.is_none());
                let raw = progress::read_bounded(&self.path, 16_000_000).unwrap();
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                let portable = serde_json::to_value(&self.progress).unwrap();
                same(&loaded, &portable, "complete portable readback");
                same(
                    &loaded.attempts,
                    &self.progress.attempts,
                    "all Attempts readback",
                );
                let mut full = portable;
                full["migration_receipts"] =
                    serde_json::to_value(&loaded.migration_receipts).unwrap();
                same(
                    &serde_json::from_slice::<Value>(&raw).unwrap(),
                    &full,
                    "main raw exact plus actual receipts",
                );
                assert!(
                    progress::read_bounded(&self.path, 16_000_000).unwrap() == raw,
                    "read-only load keeps raw bytes"
                );
                self.progress = loaded;
            }
            Step::Filter => {
                assert!(self.selected.is_none());
                self.filter = Filter::default();
                self.filter.search = s.title.clone();
                self.filter.max_minutes = 120;
                self.page = 0;
            }
            Step::ResumeExact => {
                let mut before = ctx
                    .data(|d| d.get_temp::<Value>(egui::Id::new("apache-resume")))
                    .unwrap();
                assert!(before["visited"].as_u64().unwrap() < self.progress.clock);
                before["visited"] = serde_json::json!(self.progress.clock);
                same(
                    &self.progress.attempts[id],
                    &before,
                    "warm Continue retains full Attempt except actual visit clock",
                );
                assert_eq!(self.apache_static_view(i), teaching::View::Example);
                assert!(self.progress.favorites.contains(id));
            }
            Step::RecordReset => {
                let v: ResetRecord = (
                    serde_json::to_value(&self.progress.attempts[id]).unwrap(),
                    self.progress
                        .attempts
                        .iter()
                        .filter(|(k, _)| k.as_str() != id)
                        .map(|(k, a)| (k.clone(), serde_json::to_value(a).unwrap()))
                        .collect(),
                    self.progress.favorites.clone(),
                );
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("apache-reset"), v));
            }
            Step::CancelExact => {
                let v = ctx
                    .data(|d| d.get_temp::<ResetRecord>(egui::Id::new("apache-reset")))
                    .unwrap();
                same(
                    &self.progress.attempts[id],
                    &v.0,
                    "Reset Cancel keeps full selected Attempt",
                );
                assert!(!self.reset_pending);
            }
            Step::ResetExact => {
                let a = &self.progress.attempts[id];
                same(&a.state, &s.initial, "selected Reset real initial");
                assert!(a.transcript.is_empty());
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .apache_static_learning
                    .as_ref()
                    .unwrap();
                same(
                    &l.simulation,
                    &lesson.definition.initial,
                    "reset only selected practice",
                );
                assert!(draft(a, false).matches(simulation(a, false)));
                assert!(draft(a, true).matches(simulation(a, true)));
                assert!(!progress::current_solved(a, &s));
                let v = ctx
                    .data(|d| d.get_temp::<ResetRecord>(egui::Id::new("apache-reset")))
                    .unwrap();
                for (k, a) in v.1 {
                    same(
                        &self.progress.attempts[&k],
                        &a,
                        "reset keeps unrelated full Attempt",
                    );
                }
                same(&self.progress.favorites, &v.2, "reset keeps favorites");
            }
            Step::Theme(light) => {
                self.scale = if *light { 1.5 } else { 2. };
                self.theme_id = if *light { "dlms-light" } else { "dlms-dark" }.into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(*light));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(if *light {
                    egui::vec2(1920., 1080.)
                } else {
                    egui::vec2(946., 929.)
                }));
                self.lesson_scroll_reset = true;
            }
            Step::Geometry(scale) => {
                let rect = ctx.content_rect();
                let ppp = ctx.pixels_per_point();
                let zoom = ctx.zoom_factor();
                assert!(
                    rect.is_finite()
                        && rect.width() > 0.
                        && rect.height() > 0.
                        && ppp.is_finite()
                        && ppp > 0.
                );
                let sample = (
                    rect.width().to_bits(),
                    rect.height().to_bits(),
                    ppp.to_bits(),
                    zoom.to_bits(),
                );
                let k = egui::Id::new(("apache-geometry", stage));
                let prior = ctx.data(|d| d.get_temp::<(u8, u32, u32, u32, u32)>(k));
                let tries = prior.map_or(1, |x| x.0 + 1);
                assert!(tries <= 20, "Apache geometry did not settle");
                ctx.data_mut(|d| d.insert_temp(k, (tries, sample.0, sample.1, sample.2, sample.3)));
                if (zoom - *scale).abs() > 0.01
                    || prior.is_none_or(|x| (x.1, x.2, x.3, x.4) != sample)
                {
                    ctx.request_repaint();
                    return (false, None);
                }
                println!(
                    "Apache {id} actual content={rect:?} ppp={ppp} zoom={zoom}; requested window sizes are not measured sizes"
                );
            }
            Step::Challenge => {
                assert!(self.progress.attempts[id].challenge);
                assert!(
                    self.metrics.lesson_tabs.is_empty()
                        && self.metrics.insert.is_none()
                        && self.metrics.example_action.is_none()
                );
                assert!(!self.help_open);
                assert!(
                    progress::current_solved(&self.progress.attempts[id], &s)
                        && self.metrics.completion
                );
            }
            Step::Capture(name, shot) => {
                let r = match shot {
                    Shot::Heading => self.metrics.back,
                    Shot::Result => self.metrics.apache_result,
                    Shot::Response => self.metrics.apache_response,
                    Shot::Field(key) => {
                        let (_, r, id, enabled) = self
                            .metrics
                            .apache_fields
                            .iter()
                            .find(|(k, _, _, _)| k == key)
                            .unwrap();
                        assert!(*enabled && ctx.memory(|m| m.has_focus(*id)));
                        if matches!(*key, "config" | "file") {
                            let (_, v, _, _) = self.metrics.apache_editor.unwrap();
                            Some(egui::Rect::from_center_size(
                                v.center(),
                                egui::vec2(
                                    v.width(),
                                    ctx.style().spacing.interact_size.y.min(v.height()),
                                ),
                            ))
                        } else {
                            Some(*r)
                        }
                    }
                }
                .expect("actual capture region");
                if !self.apache_lesson_reveal(ctx, r) {
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
                    "PASS four normal static-web tasks: real library Start/Continue, separate Insert-only/Save/lifecycle/practice changes, full initial/partial/reference/alternative, malformed saved-vs-active retry, read-only virtual requests, exact independent raw drafts/readback/reset/favorite/Challenge and Light150/Dark200 focused editing. Trusted reviewed text inputs; no solved State/host server/novice/cold/platform claim."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return (false, None);
            }
        }
        (true, None)
    }
}
#[cfg(test)]
pub(in crate::ui) fn phases_for_test(scenarios: &[Scenario]) -> (usize, usize, usize, usize) {
    let p = plan(scenarios);
    assert_eq!(p.iter().filter(|s| matches!(s, Step::Select(_))).count(), 4);
    assert_eq!(p.iter().filter(|s| matches!(s, Step::Finish)).count(), 1);
    assert!(matches!(p.last(), Some(Step::Finish)));
    assert_eq!(
        p.iter().filter(|s| matches!(s, Step::Geometry(_))).count(),
        12
    );
    (
        FIRST + p.len() - 1,
        FIRST
            + p.iter()
                .position(|s| matches!(s, Step::InsertOnly))
                .unwrap(),
        p.len(),
        p.iter()
            .filter(|s| matches!(s, Step::Capture(_, _)))
            .count(),
    )
}
