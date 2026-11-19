//! Fixed development-only native sharing fixture, private progress and actual controls.
use super::*;
use dlms_learning_lab::{samba_share_source as binding, samba_share_workspace as work};
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "lpic2-smb-identity-access",
    "lpic2-smb-team-policy",
    "lpic2-smb-change-handover",
];
#[derive(Clone, Debug)]
pub(super) enum Op {
    Open(String),
    Click(String),
    Choose(String, String),
    Text(String, String),
    Check(bool, bool),
    Uncredited,
    Save,
    Fresh,
    ColdFinish,
    ColdAppearance,
    Trial(bool),
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
    popup_reveals: usize,
}
pub(super) fn repair(ops: &mut Vec<Op>, t: &work::Task, policy: bool) {
    for user in &t.accounts {
        if t.editable_accounts.iter().any(|u| u.id == user.user)
            && !t
                .initial
                .accounts
                .iter()
                .any(|u| u.user == user.user && u.enabled)
        {
            ops.push(Op::Click(format!("account:{}", user.user)));
        }
    }
    if !t.editable_accounts.is_empty() {
        if policy {
            ops.push(Op::Click("terminal-help".into()));
        }
        let folder = t
            .filesystem
            .iter()
            .find_map(|r| {
                if let dlms_learning_lab::filesystem::Requirement::Access {
                    path,
                    operation: dlms_learning_lab::filesystem::Access::ListNames,
                    ..
                } = r
                {
                    Some(path)
                } else {
                    None
                }
            })
            .unwrap();
        ops.extend([
            Op::Text("terminal".into(), format!("chmod g+rx {folder}")),
            Op::Click("command-run".into()),
        ]);
    }
    if policy {
        ops.extend([
            Op::Text("config".into(), t.reference.saved_config().unwrap().into()),
            Op::Click("save".into()),
            Op::Click("syntax".into()),
            Op::Click("apply".into()),
        ]);
    }
}
fn active_plan() -> &'static Vec<Op> {
    static P: OnceLock<Vec<Op>> = OnceLock::new();
    P.get_or_init(|| {
        let labs = pack::load(include_str!(
            "../../../scenarios/samba-standalone-lessons-v1.json"
        ))
        .unwrap();
        let mut ops = vec![];
        for (n, id) in IDS.iter().enumerate() {
            let s = labs.iter().find(|s| s.id == *id).unwrap();
            let d = binding::source(s).unwrap();
            let tag = |name: &str| format!("samba-{n}-{name}");
            let policy = n != 0;
            ops.extend([
                Op::Style(1., false),
                Op::Open((*id).into()),
                Op::Shot(tag("introduction"), "introduction".into()),
                Op::Shot(tag("worked"), "worked-table".into()),
                Op::Click("example".into()),
                Op::Shot(
                    tag("practice"),
                    if policy { "config" } else { "terminal" }.into(),
                ),
                Op::Click("trial-run".into()),
                Op::Click("check".into()),
                Op::Check(false, true),
                Op::Shot(tag("practice-failure"), "failed-case".into()),
            ]);
            repair(&mut ops, &d.example, policy);
            let reviewer = d
                .example
                .clients
                .iter()
                .find(|c| c.id == "reviewer")
                .unwrap()
                .label
                .clone();
            let writer = d
                .example
                .clients
                .iter()
                .find(|c| c.id == "writer")
                .unwrap()
                .label
                .clone();
            ops.extend([
                Op::Choose("client".into(), reviewer),
                Op::Choose("operation".into(), "Replace file".into()),
                Op::Click("trial-run".into()),
                Op::Trial(false),
                Op::Shot(tag("reviewer-denied"), "trial-title".into()),
                Op::Choose("client".into(), writer),
                Op::Choose("operation".into(), "Read file".into()),
            ]);
            ops.extend([
                Op::Click("trial-run".into()),
                Op::Shot(tag("practice-trial"), "trial-title".into()),
                Op::Click("check".into()),
                Op::Check(true, true),
                Op::Uncredited,
                Op::Click("assignment".into()),
                Op::Shot(
                    tag("assignment"),
                    if policy { "config" } else { "terminal" }.into(),
                ),
                Op::Click("trial-run".into()),
                Op::Click("check".into()),
                Op::Check(false, false),
            ]);
            repair(&mut ops, &d.assignment, policy);
            ops.extend([
                Op::Choose("operation".into(), "Replace file".into()),
                Op::Text("payload".into(), "native sharing review\n".into()),
                Op::Click("trial-run".into()),
                Op::Trial(true),
                Op::Shot(tag("written-file"), "trial-title".into()),
                Op::Choose("operation".into(), "Read file".into()),
            ]);
            ops.extend([
                Op::Click("trial-run".into()),
                Op::Shot(tag("assignment-trial"), "trial-title".into()),
                Op::Click("check".into()),
                Op::Check(true, false),
                Op::Save,
                Op::Shot(tag("complete"), "result-title".into()),
            ]);
            if policy {
                ops.extend([
                    Op::Text("config".into(), "[team]\npath = relative\n".into()),
                    Op::Check(false, false),
                    Op::Click("save".into()),
                    Op::Click("apply".into()),
                    Op::Click("check".into()),
                    Op::Check(false, false),
                    Op::Shot(tag("broken-again"), "failed-case".into()),
                ]);
            } else {
                ops.extend([
                    Op::Text("terminal".into(), "chmod g-x /srv/team-reports".into()),
                    Op::Click("command-run".into()),
                    Op::Check(false, false),
                    Op::Click("check".into()),
                    Op::Check(false, false),
                    Op::Shot(tag("broken-again"), "failed-case".into()),
                ]);
            }
            for (scale, light, label) in [(1.5, true, "light150"), (2., false, "navy200")] {
                ops.extend([
                    Op::Style(scale, light),
                    Op::Shot(
                        tag(label),
                        if policy { "config" } else { "terminal" }.into(),
                    ),
                    Op::Shot(tag(&format!("{label}-trial")), "trial-run".into()),
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
fn current(a: &LabApp) -> (&work::Definition, &work::Session) {
    let s = &a.scenarios[a.selected.unwrap()];
    let a = &a.progress.attempts[&s.id];
    let (engine::Goal::SambaShare { definition }, engine::State::SambaShare { simulation }) =
        (&s.goal, &a.state)
    else {
        panic!()
    };
    (definition, simulation)
}
fn target(a: &LabApp, key: &str) -> Option<egui::Rect> {
    match key {
        "reset-header" => a.metrics.reset,
        "confirm-reset" => a.metrics.confirm_reset,
        _ => a
            .metrics
            .samba
            .controls
            .iter()
            .find(|c| c.0 == key)
            .map(|c| c.1),
    }
}
impl LabApp {
    fn samba_parent_wheel(&mut self, ctx: &egui::Context) {
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
            .expect("actual scroll region");
        let gutter = egui::Rect::from_min_max(
            egui::pos2(view.right(), view.top()),
            egui::pos2(region.right(), view.bottom()),
        )
        .intersect(ctx.content_rect());
        assert!(gutter.width() * ctx.pixels_per_point() > 1.);
        for e in &mut self.pending_events {
            if let egui::Event::PointerMoved(p) = e {
                *p = gutter.center();
            }
        }
    }
    pub(in crate::ui) fn samba_tick(&mut self, ctx: &egui::Context) {
        self.samba_flow(ctx, active_plan());
    }
    pub(super) fn samba_flow(&mut self, ctx: &egui::Context, ops: &[Op]) {
        let key = egui::Id::new("samba-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap_or_default();
        if f.at == ops.len() {
            println!("SAMBA_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let began = f.began.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed() < std::time::Duration::from_secs(15),
            "Sharing native deadline at {} {:?}",
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
        assert!(f.polls < 1200, "Sharing eligible-poll limit");
        if f.polls == 1 {
            println!("SAMBA_NATIVE_STEP {} {:?}", f.at, ops[f.at]);
        }
        let mut popup_wheel = false;
        let done = match ops[f.at].clone() {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                true
            }
            Op::Click(k) => self.click_visible_smoke_control(ctx, target(self, &k)),
            Op::Choose(k, value) => {
                if f.phase == 0 {
                    if self.click_visible_smoke_control(ctx, target(self, &k)) {
                        f.phase = 1;
                    }
                    false
                } else {
                    let ack = egui::Id::new(("native-click-phases", self.smoke_stage));
                    if ctx.data(|d| d.get_temp::<u8>(ack)).unwrap_or(0) >= 2 {
                        self.click_smoke_control(ctx, None)
                    } else {
                        let option = if k == "case" {
                            value
                        } else {
                            format!("{k}:{value}")
                        };
                        let (_, rect, clip) = self
                            .metrics
                            .samba
                            .controls
                            .iter()
                            .find(|c| c.0 == option)
                            .expect("actual popup option")
                            .clone();
                        let clip = clip.intersect(ctx.content_rect());
                        if smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
                            self.click_smoke_control(ctx, Some(rect))
                        } else {
                            f.popup_reveals += 1;
                            assert!(f.popup_reveals <= 20, "Sharing popup reveal limit");
                            self.pending_events.extend([
                                egui::Event::PointerMoved(clip.center()),
                                egui::Event::MouseWheel {
                                    unit: egui::MouseWheelUnit::Point,
                                    delta: egui::vec2(
                                        0.,
                                        (clip.center().y - rect.center().y).clamp(-500., 500.),
                                    ),
                                    modifiers: Default::default(),
                                },
                            ]);
                            popup_wheel = true;
                            false
                        }
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
                    let id = if k == "config" {
                        self.metrics.samba.editor.unwrap().2
                    } else {
                        self.metrics
                            .samba
                            .fields
                            .iter()
                            .find(|r| r.0 == k)
                            .unwrap()
                            .2
                    };
                    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
                    self.pending_events.extend([
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
                        egui::Event::Text(text.clone()),
                    ]);
                    f.phase = 2;
                    false
                } else {
                    let (_, s) = current(self);
                    let a = if s.view == teaching::View::Example {
                        &s.practice
                    } else {
                        &s.assignment
                    };
                    let actual = match k.as_str() {
                        "config" => &a.editor,
                        "terminal" => &a.terminal,
                        "payload" => match &a.request.operation {
                            work::Operation::Replace { text, .. } => text,
                            _ => panic!("replacement text"),
                        },
                        _ => panic!("fixed text field"),
                    };
                    assert_eq!(actual, &text);
                    true
                }
            }
            Op::ColdAppearance => self.samba_reopen_appearance(ctx),
            Op::ColdFinish => {
                self.finish_samba_reopen(ctx);
                false
            }
            Op::Trial(pass) => {
                let (_, s) = current(self);
                let a = if s.view == teaching::View::Example {
                    &s.practice
                } else {
                    &s.assignment
                };
                assert_eq!(
                    a.last_trial
                        .as_ref()
                        .expect("actual current trial")
                        .result
                        .is_ok(),
                    pass
                );
                true
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
        if !popup_wheel {
            self.samba_parent_wheel(ctx);
        }
        if done {
            let scroll = egui::Id::new(("bounded-native-scroll", self.smoke_stage));
            ctx.data_mut(|d| {
                d.remove::<usize>(scroll);
            });
            f.at += 1;
            f.phase = 0;
            f.polls = 0;
            f.popup_reveals = 0;
            f.began = None;
        }
        ctx.data_mut(|d| d.insert_temp(key, f));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}
#[test]
fn sharing_native_plan_uses_real_inputs_fresh_trial_outcomes_and_unique_evidence() {
    let p = active_plan();
    assert_eq!(
        p.iter()
            .filter_map(|o| if let Op::Open(id) = o {
                Some(id.as_str())
            } else {
                None
            })
            .collect::<Vec<_>>(),
        IDS
    );
    for pair in [(true, true), (false, true), (true, false), (false, false)] {
        assert!(
            p.iter()
                .filter(|o| matches!(o,Op::Check(a,b)if(*a,*b)==pair))
                .count()
                >= 3
        );
    }
    let shots: Vec<_> = p
        .iter()
        .filter_map(|o| {
            if let Op::Shot(name, _) = o {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    assert!(shots.len() >= 40);
    assert_eq!(
        shots
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        shots.len()
    );
    assert!(
        p.iter()
            .any(|o| matches!(o,Op::Text(key,_)if key=="payload"))
    );
    assert!(p.iter().any(|o| matches!(o, Op::Trial(true))));
    assert!(p.iter().any(|o| matches!(o, Op::Trial(false))));
    assert!(matches!(p.last(), Some(Op::Fresh)));
}

#[test]
fn samba_capture_rechecks_real_anchor_clip_before_requesting_a_screenshot() {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    let clip = egui::Rect::from_min_max(egui::pos2(10., 10.), egui::pos2(450., 350.));
    a.metrics.main_viewport = Some(clip);
    a.metrics.main_scroll_region = Some(clip.expand2(egui::vec2(10., 0.)));
    a.metrics.samba.controls.push((
        "worked-table".into(),
        egui::Rect::from_min_size(egui::pos2(20., 400.), egui::vec2(200., 100.)),
        clip,
    ));
    let at = active_plan()
        .iter()
        .position(|o| matches!(o,Op::Shot(_,k)if k=="worked-table"))
        .unwrap();
    let key = egui::Id::new("samba-native-flow");
    ctx.data_mut(|d| {
        d.insert_temp(
            key,
            Flow {
                at,
                phase: 1,
                ..Default::default()
            },
        )
    });
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(500., 400.),
            )),
            ..Default::default()
        },
        |ctx| a.samba_tick(ctx),
    );
    assert!(!a.capture_pending);
    let f = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap();
    assert_eq!((f.at, f.phase), (at, 0));
    assert!(
        a.pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
    );
    a.pending_events.clear();
    a.metrics.samba.controls[0].1 =
        egui::Rect::from_min_size(egui::pos2(20., 50.), egui::vec2(200., 100.));
    ctx.data_mut(|d| {
        d.insert_temp(
            key,
            Flow {
                at,
                phase: 1,
                ..Default::default()
            },
        )
    });
    let _ = ctx.run(egui::RawInput::default(), |ctx| a.samba_tick(ctx));
    assert!(a.capture_pending);
}
