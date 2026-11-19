//! Closed developer fixture: actual native controls on disposable review data.
use super::*;
use dlms_learning_lab::{
    network_transfer as core, network_transfer_source as binding,
    network_transfer_workspace as work,
};
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "ccna-tftp-recovery-copy",
    "ccna-ftp-archive-channel",
    "ccna-transfer-file-handover",
];
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Click(String),
    Choose(String, String),
    Text(String, String),
    Policy(String),
    Check(bool, bool),
    Actual(bool),
    Uncredited,
    Save,
    Fresh,
    Style(f32, bool),
    Shot(String, String),
}
#[derive(Clone, Default)]
struct Flow {
    at: usize,
    phase: u8,
    next: f64,
    began: Option<Instant>,
    polls: usize,
    before_text: Option<String>,
    before_checked: Option<bool>,
}
fn recipe(ops: &mut Vec<Op>, r: &work::Draft) {
    ops.push(Op::Choose(
        "protocol".into(),
        if r.request.protocol == core::Protocol::Tftp {
            "TFTP"
        } else {
            "FTP · passive"
        }
        .into(),
    ));
    ops.push(Op::Choose(
        "direction".into(),
        if r.request.direction == core::Direction::Download {
            "Download · server to device"
        } else {
            "Upload · device to server"
        }
        .into(),
    ));
    ops.push(Op::Choose("server".into(), r.request.server.clone()));
    if let Some(login) = &r.request.login {
        ops.push(Op::Choose("login".into(), login.clone()));
    }
    ops.push(Op::Text("local-path".into(), r.request.local_path.clone()));
    ops.push(Op::Text(
        "remote-path".into(),
        r.request.remote_path.clone(),
    ));
    if let Some(doc) = r.documents.routers.get("R1") {
        ops.push(Op::Policy(doc.clone()));
    }
}
fn plan() -> &'static Vec<Op> {
    static P: OnceLock<Vec<Op>> = OnceLock::new();
    P.get_or_init(|| {
        let labs = pack::load(include_str!(
            "../../../scenarios/network-transfers-lessons-v1.json"
        ))
        .unwrap();
        let mut ops = vec![];
        for (n, id) in IDS.iter().enumerate() {
            let s = labs.iter().find(|s| s.id == *id).unwrap();
            let d = binding::source(s).unwrap();
            let tag = |name: &str| format!("transfer-{n}-{name}");
            ops.extend([
                Op::Style(1., false),
                Op::Open(id.to_string()),
                Op::Shot(tag("introduction"), "example".into()),
                Op::Shot(tag("worked-example"), "worked-example".into()),
                Op::Click("example".into()),
                Op::Shot(tag("practice"), "local-path".into()),
                Op::Click("transfer".into()),
                Op::Click("check".into()),
                Op::Check(false, true),
                Op::Shot(tag("practice-feedback"), "failure-reason".into()),
            ]);
            if n == 0 {
                ops.extend([
                    Op::Click("files".into()),
                    Op::Click("file:Training/lesson.cfg".into()),
                    Op::Shot(
                        tag("practice-file-evidence"),
                        "file:Training/lesson.cfg".into(),
                    ),
                    Op::Click("files".into()),
                ]);
            } else {
                ops.extend([
                    Op::Click("stages".into()),
                    Op::Shot(tag("practice-packet-evidence"), "stages".into()),
                    Op::Click("stages".into()),
                ]);
            }
            for r in &d.worked_example {
                if n == 2 {
                    ops.push(Op::Choose(
                        "protocol".into(),
                        if r.request.protocol == core::Protocol::Tftp {
                            "TFTP"
                        } else {
                            "FTP · passive"
                        }
                        .into(),
                    ));
                    ops.push(Op::Choose("server".into(), r.request.server.clone()));
                }
                ops.extend([Op::Click("worked".into()), Op::Click("transfer".into())]);
            }
            ops.extend([
                Op::Click("check".into()),
                Op::Check(true, true),
                Op::Uncredited,
                Op::Shot(tag("practice-pass"), "result-title".into()),
            ]);
            if n == 0 {
                ops.extend([
                    Op::Choose("direction".into(), "Upload · device to server".into()),
                    Op::Click("transfer".into()),
                    Op::Actual(false),
                    Op::Shot(tag("practice-exploration"), "stages".into()),
                    Op::Choose("direction".into(), "Download · server to device".into()),
                ]);
            } else if n == 1 {
                ops.extend([
                    Op::Choose("login".into(), "reader".into()),
                    Op::Click("transfer".into()),
                    Op::Actual(false),
                    Op::Shot(tag("practice-exploration"), "stages".into()),
                    Op::Choose("login".into(), "backup-operator".into()),
                ]);
            } else {
                ops.extend([
                    Op::Choose("protocol".into(), "FTP · passive".into()),
                    Op::Choose("direction".into(), "Upload · device to server".into()),
                    Op::Choose("server".into(), "PracticeVault".into()),
                    Op::Choose("login".into(), "archivist".into()),
                    Op::Text("local-path".into(), "lesson.cfg".into()),
                    Op::Text("remote-path".into(), "archive/demo.cfg".into()),
                    Op::Click("transfer".into()),
                    Op::Actual(true),
                    Op::Click("check".into()),
                    Op::Check(false, true),
                    Op::Shot(tag("practice-exploration"), "result-title".into()),
                    Op::Text("local-path".into(), "current.cfg".into()),
                ]);
            }
            ops.extend([
                Op::Click("transfer".into()),
                Op::Actual(true),
                Op::Click("check".into()),
                Op::Check(true, true),
                Op::Uncredited,
                Op::Click("assignment".into()),
                Op::Shot(tag("assignment"), "local-path".into()),
                Op::Click("check".into()),
                Op::Check(false, false),
            ]);
            let recipes: Vec<_> = s
                .solution
                .iter()
                .filter_map(|e| {
                    match serde_json::from_str::<binding::Event>(
                        e.strip_prefix(binding::PREFIX).unwrap(),
                    )
                    .unwrap()
                    {
                        binding::Event::Edit { draft } => Some(draft),
                        _ => None,
                    }
                })
                .collect();
            for r in &recipes {
                recipe(&mut ops, r);
                ops.push(Op::Click("transfer".into()));
            }
            ops.extend([
                Op::Click("check".into()),
                Op::Check(true, false),
                Op::Save,
                Op::Shot(tag("complete"), "result-title".into()),
            ]);
            ops.push(if n == 1 {
                Op::Text("local-path".into(), "note.txt".into())
            } else {
                Op::Text("remote-path".into(), "public/outdated.cfg".into())
            });
            ops.extend([
                Op::Check(false, false),
                Op::Click("transfer".into()),
                Op::Click("check".into()),
                Op::Check(false, false),
                Op::Shot(tag("wrong-source"), "failure-reason".into()),
            ]);
            recipe(&mut ops, recipes.last().unwrap());
            ops.extend([
                Op::Click("transfer".into()),
                Op::Click("check".into()),
                Op::Check(true, false),
                Op::Style(1.5, true),
                Op::Shot(tag("light150-fields"), "local-path".into()),
            ]);
            if n > 0 {
                ops.push(Op::Shot(tag("light150-editor"), "policy-R1".into()));
            }
            ops.extend([
                Op::Style(2., false),
                Op::Shot(tag("dark200-fields"), "local-path".into()),
            ]);
            if n > 0 {
                ops.push(Op::Shot(tag("dark200-editor"), "policy-R1".into()));
            }
            ops.extend([
                Op::Click("reset-header".into()),
                Op::Click("confirm-reset".into()),
                Op::Fresh,
            ]);
        }
        ops
    })
}
fn current(a: &LabApp) -> (&work::Definition, &work::Session) {
    let s = &a.scenarios[a.selected.unwrap()];
    let attempt = &a.progress.attempts[&s.id];
    binding::context(attempt, s).unwrap();
    let (
        engine::Goal::NetworkTransfer { definition },
        engine::State::NetworkTransfer { simulation },
    ) = (&s.goal, &attempt.state)
    else {
        panic!()
    };
    (definition, simulation)
}
fn target(a: &LabApp, key: &str) -> Option<egui::Rect> {
    match key {
        "reset-header" => a.metrics.reset,
        "confirm-reset" => a.metrics.confirm_reset,
        "worked-example" => a.metrics.example_action,
        _ => a
            .metrics
            .network_transfer
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1),
    }
}
fn replace_events(text: String) -> [egui::Event; 2] {
    [
        egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                command: true,
                ..egui::Modifiers::CTRL
            },
        },
        egui::Event::Text(text),
    ]
}
impl LabApp {
    fn transfer_parent_wheel(&mut self, ctx: &egui::Context) {
        if !self
            .pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        {
            return;
        }
        let view = self.metrics.main_viewport.expect("actual parent viewport");
        let region = self
            .metrics
            .main_scroll_region
            .expect("actual parent scroll region");
        let gutter = egui::Rect::from_min_max(
            egui::pos2(view.right(), view.top()),
            egui::pos2(region.right(), view.bottom()),
        )
        .intersect(ctx.content_rect());
        assert!(gutter.width() * ctx.pixels_per_point() > 1.);
        for event in &mut self.pending_events {
            if let egui::Event::PointerMoved(p) = event {
                *p = gutter.center();
            }
        }
    }
    pub(in crate::ui) fn network_transfer_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("network-transfer-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap_or_default();
        let ops = plan();
        if f.at == ops.len() {
            println!("NETWORK_TRANSFER_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let began = f.began.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed() < std::time::Duration::from_secs(15),
            "Transfer native deadline at {} {:?}",
            f.at,
            ops[f.at]
        );
        let now = ctx.input(|i| i.time);
        if now < f.next || self.capture_pending {
            ctx.data_mut(|d| d.insert_temp(key, f));
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        f.next = now + 0.1;
        f.polls += 1;
        assert!(f.polls < 1200, "Transfer native eligible-poll limit");
        if f.polls == 1 {
            println!("NETWORK_TRANSFER_NATIVE_STEP {} {:?}", f.at, ops[f.at]);
        }
        let done = match ops[f.at].clone() {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                true
            }
            Op::Click(k) => self.click_visible_smoke_control(ctx, target(self, &k)),
            Op::Choose(k, label) => {
                if f.phase == 0 {
                    if self.click_visible_smoke_control(ctx, target(self, &k)) {
                        f.phase = 1;
                    }
                    false
                } else {
                    let acknowledged = egui::Id::new(("native-click-phases", self.smoke_stage));
                    if ctx.data(|d| d.get_temp::<u8>(acknowledged)).unwrap_or(0) >= 2 {
                        self.click_smoke_control(ctx, None)
                    } else {
                        let clip = self
                            .metrics
                            .network_transfer
                            .popup
                            .expect("actual dropdown viewport")
                            .intersect(ctx.content_rect());
                        self.click_smoke_control(
                            ctx,
                            target(self, &format!("{k}:{label}")).map(|r| r.intersect(clip)),
                        )
                    }
                }
            }
            Op::Text(k, text) => {
                if f.phase == 0 {
                    if self.click_visible_smoke_control(ctx, target(self, &k)) {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    let id = self
                        .metrics
                        .network_transfer
                        .fields
                        .iter()
                        .find(|r| r.0 == k)
                        .unwrap()
                        .2;
                    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
                    let (_, s) = current(self);
                    let a = s.attempt(s.view == teaching::View::Example);
                    f.before_text = Some(if k == "local-path" {
                        a.draft.request.local_path.clone()
                    } else {
                        a.draft.request.remote_path.clone()
                    });
                    f.before_checked = Some(a.checked);
                    self.pending_events.extend(replace_events(text.clone()));
                    f.phase = 2;
                    false
                } else {
                    let (_, s) = current(self);
                    let a = s.attempt(s.view == teaching::View::Example);
                    assert_eq!(
                        if k == "local-path" {
                            &a.draft.request.local_path
                        } else {
                            &a.draft.request.remote_path
                        },
                        &text
                    );
                    if f.before_text.as_deref() != Some(text.as_str()) {
                        assert!(!a.checked, "Changed input clears stale feedback");
                    } else {
                        assert_eq!(
                            Some(a.checked),
                            f.before_checked,
                            "Unchanged input preserves its check state"
                        );
                    }
                    true
                }
            }
            Op::Policy(text) => {
                let r = target(self, "policy-R1");
                if r.is_none() {
                    true
                } else if f.phase == 0 {
                    if self.click_visible_smoke_control(ctx, r) {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    let id = self.metrics.network_transfer.editor.unwrap().2;
                    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
                    self.pending_events.extend(replace_events(text.clone()));
                    f.phase = 2;
                    false
                } else {
                    let (_, s) = current(self);
                    assert_eq!(
                        s.attempt(s.view == teaching::View::Example)
                            .draft
                            .documents
                            .routers["R1"],
                        text
                    );
                    true
                }
            }
            Op::Check(pass, practice) => {
                let (d, s) = current(self);
                assert_eq!(
                    s.checks(d, practice).iter().all(|r| r.0),
                    pass,
                    "{:?}",
                    s.checks(d, practice)
                );
                if !practice {
                    assert_eq!(self.metrics.completion, pass);
                }
                true
            }
            Op::Actual(success) => {
                let (d, s) = current(self);
                let practice = s.view == teaching::View::Example;
                let report = s
                    .attempt(practice)
                    .last_report(s.task(d, practice))
                    .unwrap()
                    .unwrap();
                assert_eq!(report.success, success, "{}", report.detail);
                true
            }
            Op::Uncredited => {
                let (d, s) = current(self);
                assert_eq!(s.assignment, d.new_session().unwrap().assignment);
                assert!(!s.checks(d, false).iter().all(|r| r.0));
                true
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none());
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(loaded).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Fresh => {
                let (d, s) = current(self);
                assert_eq!(*s, d.new_session().unwrap());
                true
            }
            Op::Style(scale, light) => {
                self.scale = scale;
                ctx.set_zoom_factor(scale);
                self.theme_id = theme::inventory()
                    .iter()
                    .find(|t| t.light == light)
                    .unwrap()
                    .id
                    .clone();
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
                    let rect = target(self, &k).expect("settled capture anchor");
                    let visible = self
                        .metrics
                        .main_viewport
                        .unwrap()
                        .intersect(ctx.content_rect());
                    if !smoke_control_visible(rect, visible, ctx.pixels_per_point()) {
                        f.phase = 0;
                        self.reveal_smoke_control(ctx, rect);
                        false
                    } else {
                        self.capture_pending = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                            egui::UserData::new(name),
                        ));
                        f.phase = 2;
                        false
                    }
                } else {
                    !self.capture_pending
                }
            }
        };
        self.transfer_parent_wheel(ctx);
        if done {
            let scroll = egui::Id::new(("bounded-native-scroll", self.smoke_stage));
            ctx.data_mut(|d| {
                d.remove::<usize>(scroll);
            });
            f.at += 1;
            f.phase = 0;
            f.polls = 0;
            f.began = None;
            f.before_text = None;
            f.before_checked = None;
        }
        ctx.data_mut(|d| d.insert_temp(key, f));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}

#[test]
fn transfer_native_plan_keeps_real_requests_examples_wrong_copies_and_unique_views() {
    let p = plan();
    let opened: Vec<_> = p
        .iter()
        .filter_map(|op| {
            if let Op::Open(id) = op {
                Some(id.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(opened, IDS);
    for expected in [
        Op::Check(true, true),
        Op::Check(false, true),
        Op::Check(true, false),
        Op::Check(false, false),
    ] {
        let count = p
            .iter()
            .filter(|op| match (op, &expected) {
                (Op::Check(a, b), Op::Check(c, d)) => a == c && b == d,
                _ => false,
            })
            .count();
        assert!(count >= 3);
    }
    assert_eq!(
        p.iter().filter(|op| matches!(op, Op::Uncredited)).count(),
        6
    );
    assert_eq!(p.iter().filter(|op| matches!(op, Op::Save)).count(), 3);
    assert_eq!(p.iter().filter(|op| matches!(op, Op::Fresh)).count(), 3);
    assert!(
        p.iter()
            .any(|op| matches!(op, Op::Policy(text) if text.contains("access-list")))
    );
    assert!(
        p.iter()
            .any(|op| matches!(op, Op::Text(_,text) if text=="public/outdated.cfg"))
    );
    assert!(p.iter().any(|op| matches!(op, Op::Style(2., false))));
    assert!(p.iter().any(|op| matches!(op, Op::Style(1.5, true))));
    let names: Vec<_> = p
        .iter()
        .filter_map(|op| {
            if let Op::Shot(n, _) = op {
                Some(n)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        names
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        names.len()
    );
    for n in 0..3 {
        for part in [
            "introduction",
            "practice",
            "assignment",
            "practice-feedback",
            "complete",
            "wrong-source",
        ] {
            assert!(
                names
                    .iter()
                    .any(|name| **name == format!("transfer-{n}-{part}"))
            );
        }
    }
    assert!(matches!(p.last(), Some(Op::Fresh)));
}
#[test]
fn transfer_capture_rechecks_clip_and_targets_parent_gutter_not_nested_editor() {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    let clip = egui::Rect::from_min_max(egui::pos2(10., 10.), egui::pos2(450., 350.));
    a.metrics.main_viewport = Some(clip);
    a.metrics.main_scroll_region = Some(clip.expand2(egui::vec2(10., 0.)));
    a.metrics.network_transfer.controls.push((
        "policy-R1".into(),
        egui::Rect::from_min_size(egui::pos2(20., 400.), egui::vec2(200., 100.)),
    ));
    let at = plan()
        .iter()
        .position(|p| matches!(p,Op::Shot(_,k)if k=="policy-R1"))
        .unwrap();
    let id = egui::Id::new("network-transfer-native-flow");
    let raw = || egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(500., 400.),
        )),
        ..Default::default()
    };
    ctx.data_mut(|d| {
        d.insert_temp(
            id,
            Flow {
                at,
                phase: 1,
                ..Default::default()
            },
        )
    });
    let _ = ctx.run(raw(), |ctx| a.network_transfer_tick(ctx));
    let f = ctx.data(|d| d.get_temp::<Flow>(id)).unwrap();
    assert_eq!((f.at, f.phase), (at, 0));
    assert!(!a.capture_pending);
    assert!(
        a.pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
    );
    let point = a
        .pending_events
        .iter()
        .find_map(|e| {
            if let egui::Event::PointerMoved(p) = e {
                Some(*p)
            } else {
                None
            }
        })
        .unwrap();
    assert!(point.x > clip.right() && a.metrics.main_scroll_region.unwrap().contains(point));
    a.pending_events.clear();
    a.metrics.network_transfer.controls[0].1 =
        egui::Rect::from_min_size(egui::pos2(20., 20.), egui::vec2(200., 100.));
    ctx.data_mut(|d| {
        d.insert_temp(
            id,
            Flow {
                at,
                phase: 1,
                ..Default::default()
            },
        )
    });
    let _ = ctx.run(raw(), |ctx| a.network_transfer_tick(ctx));
    assert!(a.capture_pending);
}
