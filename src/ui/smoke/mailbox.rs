//! Fixed authored Mailbox editor workflow. Private virtual data only.
use super::*;
use dlms_learning_lab::{mailbox_source as binding, mailbox_workspace as work};
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "lpic2-dovecot-secure-access",
    "lpic2-imap-mailbox-recovery",
    "lpic2-pop-transaction-recovery",
];
#[derive(Clone, Debug)]
pub(super) enum Op {
    ColdAppearance,
    ColdFinish,
    OutputContains(String, bool),
    Open(String),
    Click(String),
    Choose(String, String),
    Text(String, String),
    Check(bool, bool),
    Uncredited,
    Save,
    NotRun(bool),
    Fresh,
    Style(f32, bool),
    Narrow,
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
    requested: Option<egui::Vec2>,
    stable: u8,
}
fn config_actions(p: &mut Vec<Op>, text: &str) {
    p.extend([
        Op::Text("editor".into(), text.into()),
        Op::Click("save".into()),
        Op::Click("restart".into()),
        Op::Click("enable".into()),
    ]);
}
fn command(p: &mut Vec<Op>, text: &str) {
    p.extend([
        Op::Text("command".into(), text.into()),
        Op::Click("send".into()),
    ]);
}
pub(super) fn translate(p: &mut Vec<Op>, a: &work::Action) {
    match a {
        work::Action::Edit { text } => p.push(Op::Text("editor".into(), text.clone())),
        work::Action::Save => p.push(Op::Click("save".into())),
        work::Action::Restart => p.push(Op::Click("restart".into())),
        work::Action::Enable => p.push(Op::Click("enable".into())),
        work::Action::Client { id } => p.push(Op::Choose("client".into(), id.clone())),
        work::Action::Connect => p.push(Op::Click("connect".into())),
        work::Action::Command { line } => command(p, line),
        work::Action::Input { text } => p.push(Op::Text("command".into(), text.clone())),
        work::Action::Admin { line } => p.extend([
            Op::Text("command".into(), line.clone()),
            Op::Click("run-admin".into()),
        ]),
        work::Action::Check => p.push(Op::Click("check".into())),
        _ => panic!("Unreviewed action in fixed mailbox capture plan"),
    }
}
fn active_plan() -> &'static Vec<Op> {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let labs = pack::current().unwrap();
        let mut p = vec![];
        for (n, id) in IDS.iter().enumerate() {
            let source = labs.iter().find(|s| s.id == *id).unwrap();
            let d = binding::source(source).unwrap();
            let anchor = if d.assignment.editable_config {
                "editor-viewport"
            } else {
                "client"
            };
            let name = |s: &str| format!("mailbox-{n}-{s}");
            p.extend([
                Op::Style(1., false),
                Op::Open((*id).into()),
                Op::Shot(name("introduction"), "introduction".into()),
                Op::Shot(name("worked"), "worked".into()),
                Op::Click("example".into()),
                Op::Shot(name("practice"), anchor.into()),
                Op::Click("check".into()),
                Op::Check(false, true),
                Op::Shot(name("practice-failure"), "failed-case".into()),
            ]);
            if d.worked_config.is_some() {
                p.extend([
                    Op::Click("fill".into()),
                    Op::NotRun(true),
                    Op::Click("save".into()),
                    Op::Click("restart".into()),
                    Op::Click("enable".into()),
                ]);
            }
            let cmds = source
                .guide
                .iter()
                .flat_map(|g| &g.commands)
                .map(|c| c.command.clone())
                .collect::<Vec<_>>();
            p.push(Op::Choose(
                "client".into(),
                if n == 2 { "pop-tls" } else { "imap-tls" }.into(),
            ));
            p.push(Op::Click("connect".into()));
            if n == 0 {
                for c in &cmds[..3] {
                    command(&mut p, c);
                }
            } else if n == 1 {
                for c in &cmds[..7] {
                    command(&mut p, c);
                }
            } else {
                for c in &cmds[..6] {
                    command(&mut p, c);
                }
                p.extend([
                    Op::Click("check".into()),
                    Op::Check(false, true),
                    Op::Shot(name("pending-pop"), "failed-case".into()),
                    Op::Click("disconnect".into()),
                    Op::Click("connect".into()),
                ]);
                for c in [&cmds[0], &cmds[1], &cmds[4], &cmds[5], &cmds[7]] {
                    command(&mut p, c);
                }
            }
            p.extend([
                Op::Click("check".into()),
                Op::Check(true, true),
                Op::Shot(name("practice-complete"), "result".into()),
                Op::Uncredited,
                Op::Click("assignment".into()),
                Op::Shot(name("assignment"), "assignment".into()),
                Op::Shot(name("assignment-workspace"), anchor.into()),
                Op::Click("check".into()),
                Op::Check(false, false),
                Op::Shot(name("assignment-failure"), "failed-case".into()),
            ]);
            let actions = source
                .solution
                .iter()
                .map(|s| {
                    serde_json::from_str::<work::Action>(s.strip_prefix(binding::PREFIX).unwrap())
                        .unwrap()
                })
                .collect::<Vec<_>>();
            for a in &actions {
                translate(&mut p, a);
            }
            p.extend([
                Op::Check(true, false),
                Op::Shot(name("complete"), "result".into()),
                Op::Save,
            ]);
            if let Some(text) = actions.iter().find_map(|a| {
                if let work::Action::Edit { text } = a {
                    Some(text)
                } else {
                    None
                }
            }) {
                p.extend([
                    Op::Text("editor".into(), "bad_setting = yes".into()),
                    Op::NotRun(false),
                    Op::Click("check".into()),
                    Op::Check(false, false),
                    Op::Shot(name("stale-success-cleared"), "failed-case".into()),
                ]);
                config_actions(&mut p, text);
            } else {
                p.push(Op::Click("connect".into()));
                for c in [
                    "a1 LOGIN lee sample-lee",
                    "a2 SELECT INBOX",
                    r"a3 UID STORE 101 -FLAGS \Seen",
                ] {
                    command(&mut p, c);
                }
                p.extend([
                    Op::NotRun(false),
                    Op::Click("check".into()),
                    Op::Check(false, false),
                    Op::Shot(name("stale-success-cleared"), "failed-case".into()),
                ]);
                command(&mut p, r"a4 UID STORE 101 +FLAGS \Seen");
            }
            p.extend([Op::Click("check".into()), Op::Check(true, false), Op::Save]);
            for (scale, light, label) in [(1.5, true, "light150"), (2., false, "navy200")] {
                p.extend([
                    Op::Style(scale, light),
                    Op::Narrow,
                    Op::Shot(name(&format!("{label}-workspace")), anchor.into()),
                    Op::Shot(name(&format!("{label}-input")), "command".into()),
                    Op::Shot(name(&format!("{label}-result")), "result".into()),
                ]);
                if d.assignment.editable_config {
                    p.push(Op::Shot(name(&format!("{label}-save")), "save".into()));
                }
            }
            p.extend([
                Op::Style(1., false),
                Op::Click("reset-header".into()),
                Op::Click("confirm-reset".into()),
                Op::Fresh,
            ]);
        }
        p
    })
}

fn target(a: &LabApp, key: &str) -> Option<egui::Rect> {
    match key {
        "reset-header" => a.metrics.reset,
        "confirm-reset" => a.metrics.confirm_reset,
        _ => a
            .metrics
            .mailbox
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1),
    }
}
fn session(a: &LabApp) -> &work::Session {
    let s = &a.scenarios[a.selected.unwrap()];
    let (engine::State::MailboxAccess { simulation }
    | engine::State::MailboxAdministration { simulation }) = &a.progress.attempts[&s.id].state
    else {
        panic!("Mail source state")
    };
    simulation
}
fn scope(a: &LabApp, practice: bool) -> &work::Scope {
    let s = session(a);
    if practice { &s.practice } else { &s.assignment }
}
fn field_value(a: &LabApp, key: &str) -> String {
    let s = session(a);
    let scope = scope(a, s.view == teaching::View::Example);
    match key {
        "editor" => scope.draft.clone(),
        "command" => scope.input.clone(),
        _ => panic!("Unknown mailbox field"),
    }
}
fn checks(a: &LabApp, practice: bool) -> Option<Vec<(bool, String)>> {
    let source = &a.scenarios[a.selected.unwrap()];
    let d = binding::source(source).unwrap();
    let scope = scope(a, practice);
    assert!(scope.checked, "Click Check before observing a result");
    Some(
        if practice {
            scope.checks(&d.example)
        } else {
            d.checks(session(a))
        }
        .into_iter()
        .map(|c| (c.passed, c.detail))
        .collect(),
    )
}
fn capture_geometry(ctx: &egui::Context) -> Option<(egui::Vec2, f32)> {
    let (extent, native) = ctx.input(|i| {
        (
            i.viewport_rect().size(),
            i.viewport().native_pixels_per_point,
        )
    });
    // Wayland may omit inner_rect. The current rendered viewport still gives
    // the actual UI extent; convert it with the settled pixels-per-point.
    Some((extent * ctx.pixels_per_point(), native?))
}
impl LabApp {
    fn mailbox_parent_wheel(&mut self, ctx: &egui::Context) {
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
    pub(in crate::ui) fn mailbox_tick(&mut self, ctx: &egui::Context) {
        self.mailbox_flow(ctx, active_plan());
    }
    pub(super) fn mailbox_flow(&mut self, ctx: &egui::Context, ops: &[Op]) {
        let key = egui::Id::new("mailbox-filters-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap_or_default();
        if f.at == ops.len() {
            println!("MAILBOX_ACCESS_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let began = f.began.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed() < std::time::Duration::from_secs(15),
            "Mailbox native deadline at {} {:?}",
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
        assert!(f.polls < 1200, "Mailbox eligible-poll limit");
        if f.polls == 1 {
            println!("MAILBOX_ACCESS_NATIVE_STEP {} {:?}", f.at, ops[f.at]);
        }
        let mut popup_wheel = false;
        let done = match ops[f.at].clone() {
            Op::ColdAppearance => self.samba_reopen_appearance(ctx),
            Op::ColdFinish => {
                self.finish_mailbox_cold_reopen(ctx);
                true
            }
            Op::OutputContains(text, practice) => {
                assert!(
                    scope(self, practice).output.contains(&text),
                    "Missing {text:?} in actual console output"
                );
                true
            }
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
                        let option = format!("{k}:{value}");
                        let (_, rect, clip) = self
                            .metrics
                            .mailbox
                            .controls
                            .iter()
                            .find(|c| c.0 == option)
                            .expect("Actual supplied popup option")
                            .clone();
                        let clip = clip.intersect(ctx.content_rect());
                        if smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
                            self.click_smoke_control(ctx, Some(rect))
                        } else {
                            f.popup_reveals += 1;
                            assert!(f.popup_reveals <= 20, "Mailbox popup reveal limit");
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
                    let rect = target(self, &k).unwrap();
                    let id = self
                        .metrics
                        .mailbox
                        .fields
                        .iter()
                        .find(|(_, r, _)| *r == rect)
                        .expect("Actual mail text field")
                        .2;
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
                    assert_eq!(field_value(self, &k), text);
                    true
                }
            }
            Op::Check(pass, practice) => {
                if let Some(checks) = checks(self, practice) {
                    assert_eq!(checks.iter().all(|c| c.0), pass, "{checks:?}");
                    if !practice {
                        assert_eq!(self.metrics.completion, pass);
                        let source = &self.scenarios[self.selected.unwrap()];
                        assert_eq!(
                            progress::current_solved(&self.progress.attempts[&source.id], source),
                            pass
                        );
                    }
                    true
                } else {
                    false
                }
            }
            Op::NotRun(practice) => {
                let s = scope(self, practice);
                assert!(!s.checked);
                if !practice {
                    assert!(!self.metrics.completion);
                }
                true
            }
            Op::Uncredited => {
                let s = &self.scenarios[self.selected.unwrap()];
                let a = &self.progress.attempts[&s.id];
                assert!(!progress::current_solved(a, s));
                let d = binding::source(s).unwrap();
                assert_eq!(
                    session(self).assignment,
                    work::Session::new(d).unwrap().assignment
                );
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
                let s = &self.scenarios[self.selected.unwrap()];
                let a = &self.progress.attempts[&s.id];
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, s));
                true
            }
            Op::Narrow => {
                if f.phase == 0 {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                    f.phase = 1;
                    false
                } else if f.phase == 1 {
                    f.requested = Some(capture_size::request(ctx));
                    f.phase = 2;
                    false
                } else {
                    if let Some((actual, native)) = capture_geometry(ctx) {
                        if capture_size::fits(actual, f.requested.unwrap(), native) {
                            f.stable += 1;
                        } else {
                            f.stable = 0;
                        }
                        if f.stable >= 3 {
                            println!(
                                "MAILBOX_NATIVE_GEOMETRY actual={actual:?} requested={:?} native={native} zoom={}",
                                f.requested,
                                ctx.zoom_factor()
                            );
                            true
                        } else {
                            false
                        }
                    } else {
                        f.stable = 0;
                        false
                    }
                }
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
                    // Only capture navigation surrenders its earlier keyboard
                    // target; normal learner editing retains native focus.
                    ctx.memory_mut(|memory| {
                        if let Some(focused) = memory.focused() {
                            memory.surrender_focus(focused);
                        }
                    });
                    if k == "timeline" {
                        eprintln!(
                            "SIEVE_TIMELINE_CAPTURE target={:?} parent={:?} region={:?} offset={:?} ppp={} focus={:?}",
                            target(self, &k),
                            self.metrics.main_viewport,
                            self.metrics.main_scroll_region,
                            self.metrics.main_scroll_offset,
                            ctx.pixels_per_point(),
                            ctx.memory(|m| m.focused())
                        );
                    }
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
            self.mailbox_parent_wheel(ctx);
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
            f.requested = None;
            f.stable = 0;
            f.began = None;
        }
        ctx.data_mut(|d| d.insert_temp(key, f));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}

#[test]
fn mailbox_native_plan_uses_all_three_real_workspaces_and_current_outcomes() {
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
    let shots = p
        .iter()
        .filter_map(|o| {
            if let Op::Shot(name, _) = o {
                Some(name)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        shots.len(),
        shots
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    );
    for pair in [(false, true), (true, true), (false, false), (true, false)] {
        assert!(
            p.iter()
                .filter(|o| matches!(o,Op::Check(a,b)if(*a,*b)==pair))
                .count()
                >= 3
        );
    }
    assert_eq!(p.iter().filter(|o| matches!(o, Op::Narrow)).count(), 6);
    assert_eq!(p.iter().filter(|o| matches!(o, Op::Uncredited)).count(), 3);
    assert_eq!(p.iter().filter(|o| matches!(o, Op::Fresh)).count(), 3);
    assert!(
        p.iter()
            .filter(|o| matches!(o,Op::Click(k)if k=="send"))
            .count()
            > 30
    );
}

#[test]
fn mailbox_capture_measures_the_rendered_wayland_viewport_when_inner_rect_is_absent() {
    let ctx = egui::Context::default();
    ctx.set_zoom_factor(2.);
    // Apply the queued zoom first, as a settled native capture does. egui scales
    // the current frame's old-coordinate screen_rect when consuming a new zoom.
    let _ = ctx.run(egui::RawInput::default(), |_| {});
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(946., 929.) / 2.3,
        )),
        ..Default::default()
    };
    let viewport = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
    viewport.native_pixels_per_point = Some(1.15);
    viewport.inner_rect = None;
    let _ = ctx.run(input, |ctx| {
        assert!(ctx.input(|i| i.viewport().inner_rect.is_none()));
        let (actual, native) = capture_geometry(ctx).unwrap();
        assert!(
            (actual - egui::vec2(946., 929.)).length() < 0.01,
            "{actual:?}"
        );
        assert_eq!(native, 1.15);
        assert!(capture_size::fits(
            actual,
            egui::vec2(1458.2, 941.8),
            native
        ));
    });
}
