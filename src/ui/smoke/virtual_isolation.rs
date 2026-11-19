//! Fixed native virtual-workload/vNIC/VRF fixture. Private data, no host guests.
use super::*;
use dlms_learning_lab::{virtual_isolation_source as binding, virtual_isolation_workspace as work};
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "ccna-virtual-host-resilience",
    "ccna-virtual-switch-uplink",
    "ccna-vrf-overlap-repair",
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
    ColdAppearance,
    ColdFinish,
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
pub(super) fn repair(p: &mut Vec<Op>, t: &work::Task) {
    match &t.exercise {
        work::Exercise::Workloads { reference, .. } => {
            for placement in &reference.placements {
                p.push(Op::Choose(
                    format!("placement:{}", placement.workload),
                    placement.host.clone(),
                ));
            }
        }
        work::Exercise::Routing { reference, .. } => {
            for b in &reference.bindings {
                p.push(Op::Choose(
                    format!("context:{}", b.interface),
                    b.context.clone(),
                ));
            }
        }
    }
}
fn active_plan() -> &'static Vec<Op> {
    static P: OnceLock<Vec<Op>> = OnceLock::new();
    P.get_or_init(|| {
        let labs = pack::load(include_str!(
            "../../../scenarios/virtual-isolation-lessons-v1.json"
        ))
        .unwrap();
        let mut p = vec![];
        for (n, id) in IDS.iter().enumerate() {
            let s = labs.iter().find(|s| s.id == *id).unwrap();
            let tag = |name: &str| format!("virtual-{n}-{name}");
            p.extend([
                Op::Style(1., false),
                Op::Open((*id).into()),
                Op::Shot(tag("introduction"), "introduction".into()),
                Op::Shot(
                    tag("worked"),
                    if n == 1 {
                        "example-action"
                    } else {
                        "worked-table"
                    }
                    .into(),
                ),
                Op::Click("example".into()),
            ]);
            if n == 1 {
                let field = |kind: &str| {
                    format!(
                        "field:guide-south-vswitch:{}:{kind}",
                        if kind == "access" { "studio" } else { "uplink" }
                    )
                };
                p.extend([
                    Op::Choose("switch".into(), "guide-south-vswitch".into()),
                    Op::Shot(tag("practice"), field("access")),
                    Op::Click("check".into()),
                    Op::Check(false, true),
                    Op::Text(field("access"), "30".into()),
                    Op::Click("check".into()),
                    Op::Check(false, true),
                    Op::Text(field("allowed"), "30,40,90".into()),
                    Op::Click("check".into()),
                    Op::Check(true, true),
                    Op::Uncredited,
                    Op::Shot(tag("practice-complete"), "check".into()),
                    Op::Click("assignment".into()),
                    Op::Choose("switch".into(), "south-vswitch".into()),
                ]);
                let field = |kind: &str| {
                    format!(
                        "field:south-vswitch:{}:{kind}",
                        if kind == "access" {
                            "accounting"
                        } else {
                            "uplink"
                        }
                    )
                };
                p.extend([
                    Op::Shot(tag("assignment"), field("access")),
                    Op::Click("check".into()),
                    Op::Check(false, false),
                    Op::Text(field("access"), "10".into()),
                    Op::Click("check".into()),
                    Op::Check(false, false),
                    Op::Text(field("allowed"), "10,20,99".into()),
                    Op::Click("check".into()),
                    Op::Check(true, false),
                    Op::Save,
                    Op::Shot(tag("complete"), "result-title".into()),
                    Op::Text(field("access"), "20".into()),
                    Op::Click("check".into()),
                    Op::Check(false, false),
                    Op::Shot(tag("broken-again"), "result-title".into()),
                ]);
                for (scale, light, name) in [(1.5, true, "light150"), (2., false, "navy200")] {
                    p.extend([
                        Op::Style(scale, light),
                        Op::Shot(tag(name), field("access")),
                        Op::Shot(tag(&format!("{name}-allowed")), field("allowed")),
                    ]);
                }
            } else {
                let d = binding::source(s).unwrap();
                let field = |practice: bool| match if practice {
                    &d.example.exercise
                } else {
                    &d.assignment.exercise
                } {
                    work::Exercise::Workloads { initial, .. } => {
                        format!("placement:{}", initial.placements[0].workload)
                    }
                    work::Exercise::Routing { initial, .. } => {
                        format!("context:{}", initial.bindings[0].interface)
                    }
                };
                p.extend([
                    Op::Shot(tag("practice"), field(true)),
                    Op::Click("run".into()),
                    Op::Shot(tag("practice-observation"), "trial-title".into()),
                    Op::Click("check".into()),
                    Op::Check(false, true),
                    Op::Shot(tag("practice-failure"), "failed-case".into()),
                ]);
                repair(&mut p, &d.example);
                p.extend([
                    Op::Click("run".into()),
                    Op::Click("check".into()),
                    Op::Check(true, true),
                    Op::Uncredited,
                    Op::Shot(tag("practice-complete"), "result-title".into()),
                    Op::Click("assignment".into()),
                    Op::Shot(tag("assignment"), field(false)),
                    Op::Click("check".into()),
                    Op::Check(false, false),
                ]);
                repair(&mut p, &d.assignment);
                p.extend([
                    Op::Click("run".into()),
                    Op::Shot(tag("assignment-observation"), "trial-title".into()),
                    Op::Click("check".into()),
                    Op::Check(true, false),
                    Op::Save,
                    Op::Shot(tag("complete"), "result-title".into()),
                    Op::Click(if n == 0 { "up:ledger-a" } else { "router-up" }.into()),
                    Op::Click("check".into()),
                    Op::Check(false, false),
                    Op::Shot(tag("broken-again"), "failed-case".into()),
                ]);
                for (scale, light, name) in [(1.5, true, "light150"), (2., false, "navy200")] {
                    p.extend([
                        Op::Style(scale, light),
                        Op::Shot(tag(name), field(false)),
                        Op::Shot(tag(&format!("{name}-trial")), "run".into()),
                    ]);
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
        "switch" => a.metrics.vlan_switch_picker,
        "example-action" => a.metrics.example_action,
        _ => a
            .metrics
            .virtual_isolation
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1)
            .or_else(|| match key {
                "introduction" => a.metrics.lesson_tabs.first().copied(),
                "example" => a.metrics.lesson_tabs.get(1).copied(),
                "assignment" => a.metrics.lesson_tabs.get(2).copied(),
                "check" => a.metrics.check,
                "result-title" => a.metrics.feedback,
                _ => {
                    let parts = key.split(':').collect::<Vec<_>>();
                    if parts.len() == 4 && parts[0] == "field" {
                        a.metrics
                            .vlan_named_fields
                            .iter()
                            .find(|(s, p, k, _)| s == parts[1] && p == parts[2] && *k == parts[3])
                            .map(|r| r.3)
                    } else {
                        None
                    }
                }
            }),
    }
}
fn field_value(a: &LabApp, key: &str) -> String {
    let s = &a.scenarios[a.selected.unwrap()];
    let w = a.progress.attempts[&s.id].workspace.as_ref().unwrap();
    let draft = if a.switching_view(a.selected.unwrap()) == teaching::View::Example {
        &w.switching_learning.as_ref().unwrap().draft
    } else {
        w.switching_draft.as_ref().unwrap()
    };
    let parts = key.split(':').collect::<Vec<_>>();
    let port = &draft.switches[parts[1]].ports[parts[2]];
    match parts[3] {
        "access" => port.access.clone(),
        "allowed" => port.allowed.clone(),
        _ => panic!("Fixed native VLAN field"),
    }
}
fn checks(a: &LabApp, practice: bool) -> Vec<(bool, String)> {
    let s = &a.scenarios[a.selected.unwrap()];
    let attempt = &a.progress.attempts[&s.id];
    if let (
        engine::Goal::VirtualIsolation { definition },
        engine::State::VirtualIsolation { simulation },
    ) = (&s.goal, &attempt.state)
    {
        simulation.checks(definition, practice)
    } else if practice {
        let l = attempt
            .workspace
            .as_ref()
            .unwrap()
            .switching_learning
            .as_ref()
            .unwrap();
        assert!(l.checked);
        l.checks(s.switching_teaching.as_ref().unwrap())
    } else {
        engine::checks(&attempt.state, &s.goal)
    }
}
impl LabApp {
    fn virtual_parent_wheel(&mut self, ctx: &egui::Context) {
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
    pub(in crate::ui) fn virtual_isolation_tick(&mut self, ctx: &egui::Context) {
        self.virtual_flow(ctx, active_plan());
    }
    pub(super) fn virtual_flow(&mut self, ctx: &egui::Context, ops: &[Op]) {
        let key = egui::Id::new("virtual-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap_or_default();
        if f.at == ops.len() {
            println!("VIRTUAL_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let began = f.began.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed() < std::time::Duration::from_secs(15),
            "Virtual-network native deadline at {} {:?}",
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
        assert!(f.polls < 1200, "Virtual-network eligible-poll limit");
        if f.polls == 1 {
            println!("VIRTUAL_NATIVE_STEP {} {:?}", f.at, ops[f.at]);
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
                        let (rect, clip) = if k == "switch" {
                            let rect = self
                                .metrics
                                .vlan_switch_choices
                                .iter()
                                .find(|(id, _)| id == &value)
                                .expect("Actual switch option")
                                .1;
                            (
                                rect,
                                self.metrics
                                    .vlan_switch_popup
                                    .expect("actual switch popup clip"),
                            )
                        } else {
                            let option = format!("{k}:{value}");
                            let (_, rect, clip) = self
                                .metrics
                                .virtual_isolation
                                .controls
                                .iter()
                                .find(|c| c.0 == option)
                                .expect("Actual supplied popup option")
                                .clone();
                            (rect, clip)
                        };
                        let clip = clip.intersect(ctx.content_rect());
                        if smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
                            self.click_smoke_control(ctx, Some(rect))
                        } else {
                            f.popup_reveals += 1;
                            assert!(f.popup_reveals <= 20, "Virtual-network popup reveal limit");
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
                        .vlan_fields
                        .iter()
                        .find(|(_, r)| *r == rect)
                        .expect("Actual VLAN text field")
                        .0;
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
                if let (
                    engine::State::VirtualIsolation { simulation },
                    engine::Goal::VirtualIsolation { definition },
                ) = (&a.state, &s.goal)
                {
                    assert_eq!(
                        simulation.assignment,
                        definition.new_session().unwrap().assignment
                    );
                } else {
                    assert_eq!(a.state, s.initial);
                }
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
                self.finish_virtual_reopen(ctx);
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
            self.virtual_parent_wheel(ctx);
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
fn virtual_native_plan_has_three_real_workspaces_and_unique_screens() {
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
            if let Op::Shot(name, _) = o {
                Some(name)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert!(names.len() >= 35);
    assert_eq!(
        names.len(),
        names
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    );
    assert!(
        p.iter()
            .any(|o| matches!(o,Op::Text(k,_)if k.contains("allowed")))
    );
    assert!(
        p.iter()
            .any(|o| matches!(o,Op::Choose(k,_)if k.starts_with("context:")))
    );
    assert!(
        p.iter()
            .any(|o| matches!(o,Op::Choose(k,_)if k.starts_with("placement:")))
    );
}

#[test]
fn virtual_capture_rechecks_real_anchor_clip_before_requesting_a_screenshot() {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    let clip = egui::Rect::from_min_max(egui::pos2(10., 10.), egui::pos2(450., 350.));
    a.metrics.main_viewport = Some(clip);
    a.metrics.main_scroll_region = Some(clip.expand2(egui::vec2(10., 0.)));
    a.metrics.virtual_isolation.controls.push((
        "worked-table".into(),
        egui::Rect::from_min_size(egui::pos2(20., 400.), egui::vec2(200., 100.)),
        clip,
    ));
    let at = active_plan()
        .iter()
        .position(|o| matches!(o,Op::Shot(_,k)if k=="worked-table"))
        .unwrap();
    let key = egui::Id::new("virtual-native-flow");
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
        |ctx| a.virtual_isolation_tick(ctx),
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
    a.metrics.virtual_isolation.controls[0].1 =
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
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        a.virtual_isolation_tick(ctx)
    });
    assert!(a.capture_pending);
}
