//! Closed native campus fixture; actual clicks, source-held choices and private data.
use super::*;
use dlms_learning_lab::{
    network_architecture_source as binding, network_architecture_workspace as work,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = [
    "ccna-collapsed-campus-uplinks",
    "ccna-distribution-core-paths",
];
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Click(String),
    Choose(String, String),
    Check(bool, bool),
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
    popup_reveals: usize,
}
fn repair(ops: &mut Vec<Op>, t: &work::Task) {
    for c in &t.reference.connections {
        if t.editable.iter().any(|w| w.id == c.wire) {
            for (end, j) in [("a", &c.a), ("b", &c.b)] {
                let value = j
                    .as_ref()
                    .map(|j| format!("{}/{}", j.node, j.port))
                    .unwrap_or_else(|| "unplugged".into());
                ops.push(Op::Choose(format!("{}:{end}", c.wire), value));
            }
        }
    }
}
fn plan() -> &'static Vec<Op> {
    static PLAN: OnceLock<Vec<Op>> = OnceLock::new();
    PLAN.get_or_init(|| build_plan(false))
}
fn active_plan() -> &'static Vec<Op> {
    if std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("leaf-spine-batch") {
        static LEAF: OnceLock<Vec<Op>> = OnceLock::new();
        LEAF.get_or_init(|| build_plan(true))
    } else {
        plan()
    }
}
fn build_plan(leaf_spine: bool) -> Vec<Op> {
    let labs = pack::load(if leaf_spine {
        include_str!("../../../scenarios/leaf-spine-lessons-v1.json")
    } else {
        include_str!("../../../scenarios/campus-architecture-lessons-v1.json")
    })
    .unwrap();
    let mut ops = vec![];
    let ids: &[&str] = if leaf_spine {
        &["ccna-leaf-spine-rack-expansion"]
    } else {
        &IDS
    };
    for (n, id) in ids.iter().enumerate() {
        let s = labs.iter().find(|s| s.id == *id).unwrap();
        let d = binding::source(s).unwrap();
        let tag = |suffix: &str| {
            format!(
                "{}-{n}-{suffix}",
                if leaf_spine { "leaf-spine" } else { "campus" }
            )
        };
        let wire = &d.assignment.editable[0].id;
        ops.extend([
            Op::Style(1., false),
            Op::Open(id.to_string()),
            Op::Shot(tag("introduction"), "introduction".into()),
            Op::Shot(tag("worked"), "worked-table".into()),
            Op::Click("example".into()),
            Op::Shot(tag("practice"), "check".into()),
            Op::Click("check".into()),
            Op::Check(false, true),
            Op::Shot(tag("practice-mistake"), "failed-row".into()),
            Op::Click("worked".into()),
            Op::Click("check".into()),
            Op::Check(true, true),
            Op::Uncredited,
            Op::Shot(tag("practice-path"), "card-first".into()),
            Op::Click("assignment".into()),
            Op::Shot(tag("assignment"), format!("{wire}:a")),
            Op::Click("check".into()),
            Op::Check(false, false),
        ]);
        repair(&mut ops, &d.assignment);
        ops.extend([
            Op::Click("check".into()),
            Op::Check(true, false),
            Op::Save,
            Op::Shot(tag("complete"), "result-title".into()),
            Op::Choose(format!("{wire}:b"), "unplugged".into()),
            Op::Check(false, false),
            Op::Click("check".into()),
            Op::Check(false, false),
            Op::Shot(tag("broken-again"), "failed-row".into()),
        ]);
        repair(&mut ops, &d.assignment);
        if leaf_spine {
            ops.extend([
                Op::Choose("pair".into(), "2".into()),
                Op::Shot(tag("actual-spine-count"), "spine-count".into()),
            ]);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Check(true, false),
            Op::Choose("case".into(), format!("case:{}", d.assignment.cases[1].id)),
            Op::Shot(tag("outage"), "card-last".into()),
            Op::Style(1.5, true),
            Op::Shot(tag("light150-controls"), format!("{wire}:b")),
            Op::Shot(tag("light150-topology"), "card-first".into()),
            Op::Style(2., false),
            Op::Shot(tag("dark200-controls"), format!("{wire}:b")),
            Op::Shot(tag("dark200-topology"), "card-last".into()),
            Op::Save,
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Fresh,
        ]);
    }
    ops
}

fn current(a: &LabApp) -> (&work::Definition, &work::Session) {
    let s = &a.scenarios[a.selected.unwrap()];
    let a = &a.progress.attempts[&s.id];
    binding::context(a, s).unwrap();
    let (
        engine::Goal::CampusArchitecture { definition },
        engine::State::CampusArchitecture { simulation },
    ) = (&s.goal, &a.state)
    else {
        panic!()
    };
    (definition, simulation)
}
fn target(a: &LabApp, key: &str) -> Option<egui::Rect> {
    match key {
        "reset-header" => a.metrics.reset,
        "confirm-reset" => a.metrics.confirm_reset,
        "card-first" => a.metrics.network_architecture.cards.first().copied(),
        "card-last" => a.metrics.network_architecture.cards.last().copied(),
        _ => a
            .metrics
            .network_architecture
            .controls
            .iter()
            .find(|c| c.0 == key)
            .map(|c| c.1),
    }
}
impl LabApp {
    fn campus_parent_wheel(&mut self, ctx: &egui::Context) {
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
    pub(in crate::ui) fn network_architecture_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("campus-native-flow");
        let mut f = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap_or_default();
        let ops = active_plan();
        if f.at == ops.len() {
            println!("CAMPUS_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let began = f.began.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed() < std::time::Duration::from_secs(15),
            "Campus native deadline at {} {:?}",
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
        assert!(f.polls < 1200, "Campus eligible-poll limit");
        if f.polls == 1 {
            println!("CAMPUS_NATIVE_STEP {} {:?}", f.at, ops[f.at]);
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
                            .network_architecture
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
                            assert!(f.popup_reveals <= 20, "Campus popup reveal limit");
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
            self.campus_parent_wheel(ctx);
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
fn campus_native_plan_keeps_independent_practice_real_controls_failures_and_unique_captures() {
    let p = plan();
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
                >= 2
        );
    }
    let shots = p
        .iter()
        .filter_map(|o| {
            if let Op::Shot(n, _) = o {
                Some(n)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(shots.len(), 26);
    assert_eq!(
        shots
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        shots.len()
    );
    assert!(p.iter().any(|o| matches!(o, Op::Style(2., false))));
    assert!(p.iter().any(|o| matches!(o, Op::Style(1.5, true))));
    assert!(matches!(p.last(), Some(Op::Fresh)));
}
#[test]
fn campus_capture_rechecks_real_anchor_clip_before_requesting_a_screenshot() {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    let clip = egui::Rect::from_min_max(egui::pos2(10., 10.), egui::pos2(450., 350.));
    a.metrics.main_viewport = Some(clip);
    a.metrics.main_scroll_region = Some(clip.expand2(egui::vec2(10., 0.)));
    a.metrics.network_architecture.controls.push((
        "worked-table".into(),
        egui::Rect::from_min_size(egui::pos2(20., 400.), egui::vec2(200., 100.)),
        clip,
    ));
    let at = plan()
        .iter()
        .position(|o| matches!(o,Op::Shot(_,k)if k=="worked-table"))
        .unwrap();
    let key = egui::Id::new("campus-native-flow");
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
        |ctx| a.network_architecture_tick(ctx),
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
    a.metrics.network_architecture.controls[0].1 =
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
        a.network_architecture_tick(ctx)
    });
    assert!(a.capture_pending);
}

#[test]
fn leaf_spine_plan_is_a_separate_closed_fixture_with_actual_pair_inspection() {
    let p = build_plan(true);
    assert_eq!(
        p.iter()
            .filter_map(|o| if let Op::Open(id) = o {
                Some(id.as_str())
            } else {
                None
            })
            .collect::<Vec<_>>(),
        vec!["ccna-leaf-spine-rack-expansion"]
    );
    assert!(
        p.iter()
            .any(|o| matches!(o,Op::Choose(k,v)if k=="pair"&&v=="2"))
    );
    assert!(p.iter().any(|o| matches!(o, Op::Uncredited)));
    assert!(p.iter().any(|o| matches!(o, Op::Fresh)));
    assert!(p.iter().any(|o| matches!(o, Op::Style(2., false))));
    assert!(p.iter().any(|o| matches!(o, Op::Style(1.5, true))));
    assert_eq!(p.iter().filter(|o| matches!(o, Op::Shot(..))).count(), 14);
}
