//! Fixed public-fixture native review, not a learner execution route.
use super::*;
use dlms_learning_lab::ssh_workspace::{self as work, Document, Event};
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "lpic1-ssh-server-trust",
    "lpic2-ssh-key-authorization",
    "lpic2-ssh-policy-handover",
];
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Click(String),
    Select(String, String),
    Field(String, String),
    Check(bool, bool),
    Save,
    Shot(String, String),
    Style(f32, bool),
    Fresh,
    Inert,
}
#[derive(Clone, Default)]
struct Flow {
    at: usize,
    phase: u8,
    polls: usize,
    next: f64,
    began: Option<Instant>,
}
fn edit(ops: &mut Vec<Op>, f: &work::Fixture, e: &Event) {
    let mut select = |doc: &Document| {
        let n = f.documents.iter().position(|d| d == doc).unwrap();
        ops.push(Op::Select("document".into(), format!("document:{n}")));
    };
    match e {
        Event::Text { document, text } => {
            select(document);
            ops.push(Op::Field("editor".into(), text.clone()));
        }
        Event::Metadata {
            path,
            owner,
            group,
            mode,
        } => {
            let doc=f.documents.iter().find(|d|matches!(d,Document::KeyFile{path:p}|Document::Attributes{path:p}if p==path)).unwrap();
            select(doc);
            ops.extend([
                Op::Select("owner".into(), format!("owner:{owner}")),
                Op::Select("group".into(), format!("group:{group}")),
                Op::Field("mode".into(), mode.clone()),
            ]);
        }
        Event::Save { document } => {
            select(document);
            ops.push(Op::Click("save".into()));
        }
        Event::ApplyPolicy {} => {
            select(&Document::ServerConfig);
            ops.push(Op::Click("apply".into()));
        }
        Event::Check {} => ops.push(Op::Click("check".into())),
        Event::Select { document } => select(document),
    }
}
fn plan() -> &'static Vec<Op> {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let labs = pack::load(include_str!(
            "../../../scenarios/ssh-access-lessons-v1.json"
        ))
        .unwrap();
        let mut ops = vec![];
        for (n, id) in IDS.iter().enumerate() {
            let s = labs.iter().find(|s| s.id == *id).unwrap();
            let d = work::source(s).unwrap();
            let tag = |name: &str| format!("ssh-{n}-{name}");
            ops.extend([
                Op::Style(1., false),
                Op::Open(id.to_string()),
                Op::Shot(tag("introduction"), "example".into()),
                Op::Shot(tag("worked"), "worked".into()),
                Op::Click("example".into()),
                Op::Shot(tag("practice"), "editor".into()),
                Op::Click("fill".into()),
                Op::Inert,
            ]);
            for e in &d.example_solution {
                edit(&mut ops, &d.example, e);
            }
            ops.extend([
                Op::Check(true, true),
                Op::Shot(tag("practice-pass"), "results".into()),
                Op::Click("assignment".into()),
                Op::Shot(tag("assignment"), "editor".into()),
                Op::Click("check".into()),
                Op::Check(false, false),
                Op::Shot(tag("needs-attention"), "results".into()),
            ]);
            for input in &s.solution {
                let e = serde_json::from_str(input.strip_prefix("ssh-lab ").unwrap()).unwrap();
                edit(&mut ops, &d.initial, &e);
            }
            ops.extend([
                Op::Check(true, false),
                Op::Save,
                Op::Shot(tag("assignment-pass"), "results".into()),
            ]);
            let doc = d
                .initial
                .documents
                .iter()
                .find(|d| matches!(d, Document::KnownHosts | Document::KeyFile { .. }))
                .unwrap()
                .clone();
            edit(
                &mut ops,
                &d.initial,
                &Event::Text {
                    document: doc.clone(),
                    text: "unfinished record".into(),
                },
            );
            ops.extend([
                Op::Check(false, false),
                Op::Click("save".into()),
                Op::Shot(tag("invalid-save"), "save".into()),
            ]);
            for input in &s.solution {
                let e = serde_json::from_str(input.strip_prefix("ssh-lab ").unwrap()).unwrap();
                edit(&mut ops, &d.initial, &e);
            }
            ops.extend([Op::Check(true, false), Op::Save]);
            if n == 1 {
                ops.extend([
                    Op::Style(1.5, true),
                    Op::Shot(tag("light150-permissions"), "mode".into()),
                    Op::Style(2., false),
                    Op::Shot(tag("dark200-permissions"), "save".into()),
                ]);
            }
            ops.extend([
                Op::Style(1., false),
                Op::Click("reset-header".into()),
                Op::Click("confirm-reset".into()),
                Op::Fresh,
            ]);
        }
        ops
    })
}
fn target(a: &LabApp, key: &str) -> Option<egui::Rect> {
    match key {
        "reset-header" => a.metrics.reset,
        "confirm-reset" => a.metrics.confirm_reset,
        "results" => a.metrics.ssh.output,
        "worked" => a.metrics.example_action,
        _ => a
            .metrics
            .ssh
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1)
            .or_else(|| {
                a.metrics
                    .ssh
                    .fields
                    .iter()
                    .find(|r| r.0 == key)
                    .map(|r| r.1)
            }),
    }
}
fn current(a: &LabApp) -> (&work::Definition, &work::Session) {
    let s = &a.scenarios[a.selected.unwrap()];
    let engine::State::SshAccess { simulation } = &a.progress.attempts[&s.id].state else {
        panic!()
    };
    (work::source(s).unwrap(), simulation)
}
fn field(a: &LabApp, key: &str) -> String {
    let (_, s) = current(a);
    let a = if s.view == teaching::View::Example {
        &s.practice
    } else {
        &s.assignment
    };
    match key {
        "editor" => match &a.drafts.selected {
            Document::KnownHosts => a.drafts.known_hosts.clone(),
            Document::ServerConfig => a.drafts.config.clone(),
            Document::KeyFile { path } => a
                .drafts
                .files
                .iter()
                .find(|d| &d.path == path)
                .unwrap()
                .text
                .clone(),
            _ => panic!(),
        },
        "mode" => {
            let (Document::KeyFile { path } | Document::Attributes { path }) = &a.drafts.selected
            else {
                panic!()
            };
            a.drafts
                .metadata
                .iter()
                .find(|d| &d.path == path)
                .unwrap()
                .mode
                .clone()
        }
        _ => panic!(),
    }
}
fn selected(a: &LabApp, key: &str) -> String {
    let (d, s) = current(a);
    let practice = s.view == teaching::View::Example;
    let f = if practice { &d.example } else { &d.initial };
    let a = if practice { &s.practice } else { &s.assignment };
    if key == "document" {
        return format!(
            "document:{}",
            f.documents
                .iter()
                .position(|doc| doc == &a.drafts.selected)
                .unwrap()
        );
    }
    let (Document::KeyFile { path } | Document::Attributes { path }) = &a.drafts.selected else {
        panic!()
    };
    let m = a.drafts.metadata.iter().find(|m| &m.path == path).unwrap();
    format!("{key}:{}", if key == "owner" { &m.owner } else { &m.group })
}
impl LabApp {
    pub(in crate::ui) fn ssh_access_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("ssh-access-native");
        let mut f = ctx
            .data_mut(|d| d.get_temp::<Flow>(key))
            .unwrap_or_default();
        if f.at == plan().len() {
            println!(
                "SSH_ACCESS_NATIVE_COMPLETE: {} real-widget actions; virtual public fixtures only",
                plan().len()
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 190000 + f.at;
        let now = ctx.input(|i| i.time);
        let due = native_action_due(
            &mut f.polls,
            now,
            f.next,
            f.began.get_or_insert_with(Instant::now).elapsed(),
        );
        assert!(due.is_ok(), "SSH step {} {:?}: {due:?}", f.at, plan()[f.at]);
        if !due.unwrap() {
            ctx.data_mut(|d| d.insert_temp(key, f));
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        f.next = now + 0.1;
        if f.polls == 1 {
            println!("SSH_NATIVE_STEP {} {:?}", f.at, plan()[f.at]);
        }
        let complete = match plan()[f.at].clone() {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                true
            }
            Op::Click(k) => self.click_visible_smoke_control(ctx, target(self, &k)),
            Op::Select(k, choice) => {
                if f.phase == 0 {
                    if selected(self, &k) == choice {
                        true
                    } else {
                        if self.click_visible_smoke_control(ctx, target(self, &k)) {
                            f.phase = 1;
                        }
                        false
                    }
                } else {
                    let acknowledged = egui::Id::new(("native-click-phases", self.smoke_stage));
                    if ctx.data(|d| d.get_temp::<u8>(acknowledged)).unwrap_or(0) >= 2 {
                        assert!(self.click_smoke_control(ctx, None));
                        assert_eq!(selected(self, &k), choice);
                        ctx.data_mut(|d| {
                            d.insert_temp(
                                key,
                                Flow {
                                    at: f.at + 1,
                                    ..Default::default()
                                },
                            )
                        });
                        ctx.request_repaint();
                        return;
                    }
                    let rect = target(self, &choice).expect("source-listed popup item");
                    let popup = self.metrics.ssh.popup.unwrap();
                    if smoke_control_visible(
                        rect,
                        popup.intersect(ctx.content_rect()),
                        ctx.pixels_per_point(),
                    ) && self.click_smoke_control(ctx, Some(rect))
                    {
                        assert_eq!(selected(self, &k), choice);
                        true
                    } else {
                        false
                    }
                }
            }
            Op::Field(k, value) => {
                if f.phase == 0 {
                    if self.click_visible_smoke_control(ctx, target(self, &k)) {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    let id = self.metrics.ssh.fields.iter().find(|r| r.0 == k).unwrap().2;
                    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
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
                    self.pending_events.push(egui::Event::Paste(value));
                    f.phase = 2;
                    false
                } else {
                    assert_eq!(field(self, &k), value);
                    true
                }
            }
            Op::Check(pass, practice) => {
                let (d, s) = current(self);
                let actual = if practice {
                    s.practice.solved(&d.example)
                } else {
                    progress::current_solved(
                        &self.progress.attempts[&d.id],
                        &self.scenarios[self.selected.unwrap()],
                    )
                };
                assert_eq!(
                    actual,
                    pass,
                    "SSH grading rows {:?}",
                    if practice {
                        s.practice.checks(&d.example)
                    } else {
                        s.assignment.checks(&d.initial)
                    }
                );
                true
            }
            Op::Inert => {
                let (d, s) = current(self);
                assert_eq!(s.assignment, work::Session::new(d).assignment);
                assert_eq!(s.practice.world, d.example.world);
                assert!(!s.practice.checked);
                true
            }
            Op::Fresh => {
                let (d, s) = current(self);
                assert_eq!(*s, work::Session::new(d));
                true
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none(), "{:?}", self.storage_error);
                let read = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(read).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Style(scale, light) => {
                self.scale = scale;
                ctx.set_zoom_factor(scale);
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
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(scale == 1.));
                if scale != 1. {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                        1100., 850.,
                    )));
                }
                self.lesson_scroll_reset = true;
                true
            }
            Op::Shot(name, k) => {
                if f.phase == 0 {
                    if self.reveal_smoke_control(ctx, target(self, &k).expect("capture anchor")) {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name,
                    )));
                    f.phase = 2;
                    false
                } else {
                    !self.capture_pending
                }
            }
        };
        if complete {
            f.at += 1;
            f.phase = 0;
            f.polls = 0;
            f.began = None;
        }
        ctx.data_mut(|d| d.insert_temp(key, f));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}
#[test]
fn ssh_fixed_native_plan_exercises_every_authored_assignment_without_host_execution() {
    let p = plan();
    assert_eq!(p.iter().filter(|op| matches!(op, Op::Open(_))).count(), 3);
    assert_eq!(
        p.iter()
            .filter(|op| matches!(op, Op::Check(true, true)))
            .count(),
        3
    );
    assert_eq!(p.iter().filter(|op| matches!(op, Op::Fresh)).count(), 3);
    assert!(p.iter().any(|op| matches!(op,Op::Field(k,_)if k=="mode")));
}
