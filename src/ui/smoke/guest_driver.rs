//! Fixed authored guest-image package/module workflow. Private virtual data only.
use super::*;
use dlms_learning_lab::{guest_driver_source as binding, guest_driver_workspace as work};
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "lpic1-guest-driver-migration",
    "lpic1-guest-object-recovery",
    "lpic1-guest-image-handover",
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
    ColdAppearance,
    ColdFinish,
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
    popup_reveals: usize,
}
fn package(p: &mut Vec<Op>, archive: &str, operation: &str) {
    p.extend([
        Op::Choose("archive".into(), archive.into()),
        Op::Choose("operation".into(), operation.into()),
        Op::Click("apply-package".into()),
    ]);
}
pub(super) fn action(p: &mut Vec<Op>, a: &work::Action) {
    use dlms_learning_lab::rpm_packages::Action as R;
    match a {
        work::Action::Package { action } => match action {
            R::UpgradeSet { archives } if archives.len() == 1 => {
                package(p, &archives[0], "Replace")
            }
            R::ReinstallExact { archive } => package(p, archive, "Reinstall"),
            _ => panic!("Fixed authored package action"),
        },
        work::Action::Run { input } => p.extend([
            Op::Text("command".into(), input.clone()),
            Op::Click("run".into()),
        ]),
        work::Action::Check => p.push(Op::Click("check".into())),
    }
}
fn active_plan() -> &'static Vec<Op> {
    static P: OnceLock<Vec<Op>> = OnceLock::new();
    P.get_or_init(|| {
        let labs = pack::load(include_str!(
            "../../../scenarios/guest-driver-lessons-v1.json"
        ))
        .unwrap();
        let mut p = vec![];
        for (n, id) in IDS.iter().enumerate() {
            let s = labs.iter().find(|s| s.id == *id).unwrap();
            let d = binding::source(s).unwrap();
            let tag = |name: &str| format!("guest-{n}-{name}");
            p.extend([
                Op::Style(1., false),
                Op::Open((*id).into()),
                Op::Shot(tag("introduction"), "introduction".into()),
                Op::Shot(tag("worked"), "worked".into()),
                Op::Click("example".into()),
                Op::Shot(tag("practice"), "archive".into()),
                Op::Click("check".into()),
                Op::Check(false, true),
            ]);
            for a in &d.example_steps {
                action(&mut p, a);
            }
            p.extend([
                Op::Check(true, true),
                Op::Uncredited,
                Op::Shot(tag("practice-complete"), "result-title".into()),
                Op::Click("assignment".into()),
                Op::Shot(tag("assignment"), "archive".into()),
                Op::Click("check".into()),
                Op::Check(false, false),
                Op::Shot(tag("assignment-failure"), "result-title".into()),
                Op::Shot(tag("failure-detail"), "failed-case".into()),
            ]);
            for (at, a) in d.reference.iter().enumerate() {
                action(&mut p, a);
                if at == 0 {
                    p.extend([Op::Click("check".into()), Op::Check(false, false)]);
                }
            }
            p.extend([
                Op::Check(true, false),
                Op::Shot(tag("complete"), "result-title".into()),
                Op::Save,
            ]);
            package(&mut p, "guest-support-new", "Remove");
            p.extend([
                Op::Click("check".into()),
                Op::Check(false, false),
                Op::Shot(tag("broken-again"), "result-title".into()),
            ]);
            package(&mut p, "guest-support-new", "Install");
            action(
                &mut p,
                &work::Action::Run {
                    input: "depmod -a".into(),
                },
            );
            action(
                &mut p,
                &work::Action::Run {
                    input: "modprobe guest_net".into(),
                },
            );
            p.extend([Op::Click("check".into()), Op::Check(true, false)]);
            for (scale, light, label) in [(1.5, true, "light150"), (2., false, "navy200")] {
                p.extend([
                    Op::Style(scale, light),
                    Op::Text(
                        "command".into(),
                        "modinfo /lib/modules/6.8.0-1/kernel/drivers/guest_net.ko".into(),
                    ),
                    Op::Shot(tag(&format!("{label}-packages")), "archive".into()),
                    Op::Shot(tag(&format!("{label}-terminal")), "command".into()),
                ]);
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
            .guest_driver
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1),
    }
}
fn session(a: &LabApp) -> &work::Session {
    let s = &a.scenarios[a.selected.unwrap()];
    let engine::State::GuestDrivers { simulation } = &a.progress.attempts[&s.id].state else {
        panic!("Guest source state")
    };
    simulation
}
fn field_value(a: &LabApp, _: &str) -> String {
    let s = session(a);
    if s.view == teaching::View::Example {
        s.practice.input.clone()
    } else {
        s.assignment.input.clone()
    }
}
fn checks(a: &LabApp, practice: bool) -> Vec<(bool, String)> {
    let source = &a.scenarios[a.selected.unwrap()];
    let d = binding::source(source).unwrap();
    let s = session(a);
    let (t, scope) = if practice {
        (&d.example, &s.practice)
    } else {
        (&d.assignment, &s.assignment)
    };
    assert!(scope.checked);
    t.checks(&scope.runtime)
        .into_iter()
        .map(|r| (r.passed, r.detail))
        .collect()
}
impl LabApp {
    fn guest_parent_wheel(&mut self, ctx: &egui::Context) {
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
    pub(in crate::ui) fn guest_driver_tick(&mut self, ctx: &egui::Context) {
        self.guest_flow(ctx, active_plan());
    }
    pub(super) fn guest_flow(&mut self, ctx: &egui::Context, ops: &[Op]) {
        let key = egui::Id::new("guest-driver-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap_or_default();
        if f.at == ops.len() {
            println!("GUEST_DRIVER_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let began = f.began.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed() < std::time::Duration::from_secs(15),
            "Guest-image native deadline at {} {:?}",
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
        assert!(f.polls < 1200, "Guest-image eligible-poll limit");
        if f.polls == 1 {
            println!("GUEST_DRIVER_NATIVE_STEP {} {:?}", f.at, ops[f.at]);
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
                        let option = format!("{k}:{value}");
                        let (_, rect, clip) = self
                            .metrics
                            .guest_driver
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
                            assert!(f.popup_reveals <= 20, "Guest-image popup reveal limit");
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
                        .guest_driver
                        .fields
                        .iter()
                        .find(|(_, r, _)| *r == rect)
                        .expect("Actual guest text field")
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
                let checks = checks(self, practice);
                assert_eq!(checks.iter().all(|c| c.0), pass, "{checks:?}");
                if !practice {
                    assert_eq!(self.metrics.completion, pass);
                    let s = &self.scenarios[self.selected.unwrap()];
                    assert_eq!(
                        progress::current_solved(&self.progress.attempts[&s.id], s),
                        pass
                    );
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
            Op::ColdAppearance => self.samba_reopen_appearance(ctx),
            Op::ColdFinish => {
                self.finish_guest_reopen(ctx);
                false
            }
            Op::Fresh => {
                let s = &self.scenarios[self.selected.unwrap()];
                let a = &self.progress.attempts[&s.id];
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, s));
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
            self.guest_parent_wheel(ctx);
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
fn guest_native_plan_retains_all_three_authored_workspaces_and_failure_paths() {
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
    let names = p
        .iter()
        .filter_map(|o| {
            if let Op::Shot(n, _) = o {
                Some(n)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 39);
    assert_eq!(
        names
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        names.len()
    );
}

#[test]
fn guest_capture_rechecks_real_anchor_clip_before_requesting_a_screenshot() {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    let clip = egui::Rect::from_min_max(egui::pos2(10., 10.), egui::pos2(450., 350.));
    a.metrics.main_viewport = Some(clip);
    a.metrics.main_scroll_region = Some(clip.expand2(egui::vec2(10., 0.)));
    a.metrics.guest_driver.controls.push((
        "worked".into(),
        egui::Rect::from_min_size(egui::pos2(20., 400.), egui::vec2(200., 100.)),
        clip,
    ));
    let at = active_plan()
        .iter()
        .position(|o| matches!(o,Op::Shot(_,k)if k=="worked"))
        .unwrap();
    let key = egui::Id::new("guest-driver-native-flow");
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
        |ctx| a.guest_driver_tick(ctx),
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
    a.metrics.guest_driver.controls[0].1 =
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
    let _ = ctx.run(egui::RawInput::default(), |ctx| a.guest_driver_tick(ctx));
    assert!(a.capture_pending);
}
