//! STAGED, UNMERGED: normal-bundle OSPF unit through native controls only.
//! Target states are developer-owned typed data; no target is published directly.
use super::*;
use dlms_learning_lab::{
    ospf::{Action, State as OspfState},
    ospf_workspace::{self as ospf_model, Draft},
};
use std::collections::BTreeMap;

const OSPF_UNIT_IDS: [&str; 4] = [
    "ccna-ospf-router-identity",
    "ccna-ospf-p2p-parameters",
    "ccna-ospf-directed-ecmp",
    "ccna-ospf-passive-handover",
];
const OSPF_UNIT_START: usize = 11_000;
const OSPF_UNIT_STRIDE: usize = 100;
const OSPF_NATIVE_MAX_EDITS: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
enum OspfNativeValue {
    Text(String),
    Toggle(bool),
}
#[derive(Clone, Debug)]
struct OspfNativeEdit {
    router: String,
    interface: Option<String>,
    key: String,
    value: OspfNativeValue,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OspfNativeStep {
    RouterOpen,
    RouterItem,
    RouterVerify,
    InterfaceOpen,
    InterfaceItem,
    InterfaceVerify,
    Edit,
    Type,
    Verify,
}
#[derive(Clone)]
struct OspfNativeQueue {
    target: Draft,
    edits: Vec<OspfNativeEdit>,
    next: usize,
    step: OspfNativeStep,
    waits: usize,
}
impl OspfNativeQueue {
    fn new(from: &Draft, to: &Draft) -> Self {
        assert!(from.valid() && to.valid());
        assert!(from.routers.keys().eq(to.routers.keys()));
        let mut edits = Vec::new();
        let mut add = |router: &str,
                       interface: Option<&str>,
                       suffix: &str,
                       before: OspfNativeValue,
                       value: OspfNativeValue| {
            if before != value {
                assert!(
                    edits.len() < OSPF_NATIVE_MAX_EDITS,
                    "fixed OSPF unit edit queue exceeds64 changes"
                );
                if let OspfNativeValue::Text(s) = &value {
                    assert!(s.len() <= ospf_model::MAX_TEXT);
                }
                edits.push(OspfNativeEdit {
                    router: router.into(),
                    interface: interface.map(str::to_owned),
                    key: interface.map_or_else(
                        || format!("{router}:{suffix}"),
                        |p| format!("{router}/{p}:{suffix}"),
                    ),
                    value,
                });
            }
        };
        for (router, desired) in &to.routers {
            let current = &from.routers[router];
            assert!(current.interfaces.keys().eq(desired.interfaces.keys()));
            add(
                router,
                None,
                "enabled",
                OspfNativeValue::Toggle(current.enabled),
                OspfNativeValue::Toggle(desired.enabled),
            );
            for (key, before, after) in [
                ("router-id", &current.router_id, &desired.router_id),
                ("process-id", &current.process_id, &desired.process_id),
                ("area", &current.area, &desired.area),
            ] {
                add(
                    router,
                    None,
                    key,
                    OspfNativeValue::Text(before.clone()),
                    OspfNativeValue::Text(after.clone()),
                );
            }
            for (port, wanted) in &desired.interfaces {
                let now = &current.interfaces[port];
                assert_eq!(
                    now.network_type, wanted.network_type,
                    "native unit queue deliberately rejects network-type changes; add/review a real picker path before extending it"
                );
                for (key, before, after) in [
                    ("up", now.up, wanted.up),
                    ("enabled", now.enabled, wanted.enabled),
                    ("passive", now.passive, wanted.passive),
                ] {
                    add(
                        router,
                        Some(port),
                        key,
                        OspfNativeValue::Toggle(before),
                        OspfNativeValue::Toggle(after),
                    );
                }
                for (key, before, after) in [
                    ("address", &now.address, &wanted.address),
                    ("prefix", &now.prefix, &wanted.prefix),
                    ("cost", &now.cost, &wanted.cost),
                    ("hello", &now.hello_seconds, &wanted.hello_seconds),
                    ("dead", &now.dead_seconds, &wanted.dead_seconds),
                ] {
                    add(
                        router,
                        Some(port),
                        key,
                        OspfNativeValue::Text(before.clone()),
                        OspfNativeValue::Text(after.clone()),
                    );
                }
            }
        }
        Self {
            target: to.clone(),
            edits,
            next: 0,
            step: OspfNativeStep::RouterOpen,
            waits: 0,
        }
    }
}
fn ospf_unit_draft(a: &Attempt, practice: bool) -> &Draft {
    let w = a.workspace.as_ref().expect("prepared OSPF workspace");
    if practice {
        &w.ospf_learning.as_ref().unwrap().draft
    } else {
        w.ospf_draft.as_ref().unwrap()
    }
}
fn ospf_unit_sim(state: &engine::State) -> &OspfState {
    let engine::State::Ospf { simulation } = state else {
        panic!("normal OSPF lesson state")
    };
    simulation
}
fn ospf_unit_actions(initial: &OspfState, actions: &[Action]) -> OspfState {
    assert!(
        actions.len() <= 16,
        "fixed typed native target action ceiling"
    );
    let mut target = initial.clone();
    for action in actions {
        target
            .apply_against(initial, action)
            .expect("validated fixed unit action");
    }
    target
}
fn ospf_unit_field(d: &Draft, e: &OspfNativeEdit) -> OspfNativeValue {
    let r = &d.routers[&e.router];
    let suffix = e.key.rsplit_once(':').unwrap().1;
    if let Some(port) = &e.interface {
        let p = &r.interfaces[port];
        match suffix {
            "up" => OspfNativeValue::Toggle(p.up),
            "enabled" => OspfNativeValue::Toggle(p.enabled),
            "passive" => OspfNativeValue::Toggle(p.passive),
            "address" => OspfNativeValue::Text(p.address.clone()),
            "prefix" => OspfNativeValue::Text(p.prefix.clone()),
            "cost" => OspfNativeValue::Text(p.cost.clone()),
            "hello" => OspfNativeValue::Text(p.hello_seconds.clone()),
            "dead" => OspfNativeValue::Text(p.dead_seconds.clone()),
            _ => panic!("unsupported port queue field"),
        }
    } else {
        match suffix {
            "enabled" => OspfNativeValue::Toggle(r.enabled),
            "router-id" => OspfNativeValue::Text(r.router_id.clone()),
            "process-id" => OspfNativeValue::Text(r.process_id.clone()),
            "area" => OspfNativeValue::Text(r.area.clone()),
            _ => panic!("unsupported process queue field"),
        }
    }
}
fn ospf_unit_key(key: egui::Key, pressed: bool, command: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: command,
            command,
            ..Default::default()
        },
    }
}
impl LabApp {
    pub(super) fn ospf_lessons_layout(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        let ordinal = (stage - 11500) / 10;
        let phase = (stage - 11500) % 10;
        let id = OSPF_UNIT_IDS[ordinal];
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        match phase {
            0 => {
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.launch(i);
                self.set_ospf_view(i, teaching::View::Assignment);
            }
            1 | 3 | 5 => {
                let scale = match phase {
                    1 => 1.,
                    3 => 1.5,
                    _ => 2.,
                };
                if !self.ospf_unit_geometry(ctx, scale) {
                    return (false, None);
                }
                if !self.reveal_smoke_control(ctx, self.metrics.ospf_diagram.unwrap()) {
                    return (false, None);
                }
                return (
                    true,
                    Some(format!("{id}-final-task-costs-{}", (scale * 100.) as usize)),
                );
            }
            2 => {
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            4 => {
                self.scale = 2.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            6 => {
                if ordinal < 3 {
                    self.smoke_stage = 11500 + (ordinal + 1) * 10 - 1;
                } else {
                    println!(
                        "PASS OSPF final task/cost legibility snapshots at100/150/200; plain-text directional labels, no new grading claim."
                    );
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            _ => unreachable!(),
        }
        (true, None)
    }
    fn ospf_unit_next_step(
        &mut self,
        ctx: &egui::Context,
        q: &mut OspfNativeQueue,
        step: OspfNativeStep,
    ) {
        q.step = step;
        q.waits = 0;
        // A new target gets its own20-attempt reveal bound; never reset on retry.
        ctx.data_mut(|d| {
            d.remove::<usize>(egui::Id::new(("bounded-native-scroll", self.smoke_stage)))
        });
        ctx.request_repaint();
    }
    fn ospf_unit_picker_open(&mut self, ctx: &egui::Context, rect: egui::Rect) -> bool {
        // Separate physical hover/press/release frames, matching the accepted private flow.
        let id = egui::Id::new(("ospf-unit-picker-phases", self.smoke_stage));
        let phase = ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0);
        if phase == 3 {
            ctx.data_mut(|d| d.remove::<u8>(id));
            return true;
        }
        if !self.reveal_smoke_control(ctx, rect) {
            return false;
        }
        self.pending_events
            .push(egui::Event::PointerMoved(rect.center()));
        if phase > 0 {
            self.pending_events.push(egui::Event::PointerButton {
                pos: rect.center(),
                button: egui::PointerButton::Primary,
                pressed: phase == 1,
                modifiers: Default::default(),
            });
        }
        ctx.data_mut(|d| d.insert_temp(id, phase + 1));
        self.started = Instant::now();
        ctx.request_repaint();
        false
    }
    fn ospf_unit_menu_item(&mut self, ctx: &egui::Context, rect: Option<egui::Rect>) -> bool {
        // Popup selection removes its item metrics before the helper's completion frame.
        // Always drain the pending click with None, rather than skipping on new selection.
        let phase = ctx
            .data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))))
            .unwrap_or(0);
        if phase >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        if rect.is_none() {
            ctx.request_repaint();
            return false;
        }
        self.click_smoke_control(ctx, rect)
    }
    fn ospf_unit_edit_queue(
        &mut self,
        ctx: &egui::Context,
        i: usize,
        practice: bool,
        target: &Draft,
    ) -> bool {
        assert_eq!(
            self.selected,
            Some(i),
            "native queue must edit the selected activity"
        );
        assert_eq!(
            self.ospf_view(i),
            if practice {
                teaching::View::Example
            } else {
                teaching::View::Assignment
            },
            "native queue context must match the actual rendered view"
        );
        let id = egui::Id::new(("ospf-unit-native-queue", self.smoke_stage, practice));
        let lab_id = self.scenarios[i].id.clone();
        let live = ospf_unit_draft(&self.progress.attempts[&lab_id], practice).clone();
        let mut q = ctx
            .data(|d| d.get_temp::<OspfNativeQueue>(id))
            .unwrap_or_else(|| OspfNativeQueue::new(&live, target));
        assert_eq!(
            q.target.routers, target.routers,
            "a native edit target must not change while suspended"
        );
        if q.next == q.edits.len() {
            assert_eq!(
                live.routers, target.routers,
                "all target fields must come from actual native edits"
            );
            ctx.data_mut(|d| d.remove::<OspfNativeQueue>(id));
            return true;
        }
        q.waits += 1;
        assert!(
            q.waits <= 80,
            "native OSPF queue stalled: lesson={lab_id},practice={practice},edit={:?},step={:?},selected={:?},routeritems={:?},portitems={:?}",
            q.edits[q.next],
            q.step,
            live.selected,
            self.metrics.ospf_router_items,
            self.metrics.ospf_interface_items
        );
        let edit = q.edits[q.next].clone();
        match q.step {
            OspfNativeStep::RouterOpen => {
                if live.selected.router == edit.router {
                    self.ospf_unit_next_step(ctx, &mut q, OspfNativeStep::InterfaceOpen);
                } else if self
                    .ospf_unit_picker_open(ctx, self.metrics.ospf_router.expect("router picker"))
                {
                    self.ospf_unit_next_step(ctx, &mut q, OspfNativeStep::RouterItem);
                }
            }
            OspfNativeStep::RouterItem => {
                let rect = self
                    .metrics
                    .ospf_router_items
                    .iter()
                    .find(|(n, _)| n == &edit.router)
                    .map(|(_, r)| *r);
                if self.ospf_unit_menu_item(ctx, rect) {
                    self.ospf_unit_next_step(ctx, &mut q, OspfNativeStep::RouterVerify);
                }
            }
            OspfNativeStep::RouterVerify => {
                if live.selected.router == edit.router {
                    self.ospf_unit_next_step(ctx, &mut q, OspfNativeStep::InterfaceOpen);
                }
            }
            OspfNativeStep::InterfaceOpen => {
                if edit
                    .interface
                    .as_ref()
                    .is_none_or(|p| live.selected.interface == *p)
                {
                    self.ospf_unit_next_step(ctx, &mut q, OspfNativeStep::Edit);
                } else if self.ospf_unit_picker_open(
                    ctx,
                    self.metrics.ospf_interface.expect("interface picker"),
                ) {
                    self.ospf_unit_next_step(ctx, &mut q, OspfNativeStep::InterfaceItem);
                }
            }
            OspfNativeStep::InterfaceItem => {
                let rect = self
                    .metrics
                    .ospf_interface_items
                    .iter()
                    .find(|(n, _)| Some(n) == edit.interface.as_ref())
                    .map(|(_, r)| *r);
                if self.ospf_unit_menu_item(ctx, rect) {
                    self.ospf_unit_next_step(ctx, &mut q, OspfNativeStep::InterfaceVerify);
                }
            }
            OspfNativeStep::InterfaceVerify => {
                if edit.interface.as_ref() == Some(&live.selected.interface) {
                    self.ospf_unit_next_step(ctx, &mut q, OspfNativeStep::Edit);
                }
            }
            OspfNativeStep::Edit => {
                let finished = match &edit.value {
                    OspfNativeValue::Text(_) => {
                        let rect = self
                            .metrics
                            .ospf_fields
                            .iter()
                            .find(|(k, _, _)| k == &edit.key)
                            .map(|(_, r, _)| *r);
                        self.click_visible_smoke_control(ctx, rect)
                    }
                    OspfNativeValue::Toggle(_) => {
                        let rect = self
                            .metrics
                            .ospf_toggles
                            .iter()
                            .find(|(k, _)| k == &edit.key)
                            .map(|(_, r)| *r);
                        self.click_visible_smoke_control(ctx, rect)
                    }
                };
                if finished {
                    let step = if matches!(edit.value, OspfNativeValue::Text(_)) {
                        OspfNativeStep::Type
                    } else {
                        OspfNativeStep::Verify
                    };
                    self.ospf_unit_next_step(ctx, &mut q, step);
                }
            }
            OspfNativeStep::Type => {
                let field_id = self
                    .metrics
                    .ospf_fields
                    .iter()
                    .find(|(k, _, _)| k == &edit.key)
                    .expect("selected native text field")
                    .2;
                assert_eq!(
                    ctx.memory(|m| m.focused()),
                    Some(field_id),
                    "native click must focus the actual selected field before typing"
                );
                let OspfNativeValue::Text(text) = &edit.value else {
                    unreachable!()
                };
                self.pending_events.extend([
                    ospf_unit_key(egui::Key::A, true, true),
                    ospf_unit_key(egui::Key::A, false, true),
                ]);
                if text.is_empty() {
                    self.pending_events.extend([
                        ospf_unit_key(egui::Key::Backspace, true, false),
                        ospf_unit_key(egui::Key::Backspace, false, false),
                    ]);
                } else {
                    self.pending_events.push(egui::Event::Text(text.clone()));
                }
                self.started = Instant::now();
                self.ospf_unit_next_step(ctx, &mut q, OspfNativeStep::Verify);
            }
            OspfNativeStep::Verify => {
                assert_eq!(
                    ospf_unit_field(&live, &edit),
                    edit.value,
                    "native edit must reach the requested raw field exactly"
                );
                q.next += 1;
                self.ospf_unit_next_step(ctx, &mut q, OspfNativeStep::RouterOpen);
            }
        }
        ctx.data_mut(|d| d.insert_temp(id, q));
        false
    }
    fn ospf_unit_feedback_head(&mut self, ctx: &egui::Context) -> bool {
        let r = self.metrics.feedback.expect("native OSPF feedback panel");
        // The full list can exceed a narrow viewport; reveal its first real row.
        let row = (ctx.style().text_styles[&egui::TextStyle::Body].size * 1.6).min(r.height());
        self.reveal_smoke_control(
            ctx,
            egui::Rect::from_min_size(r.min, egui::vec2(r.width(), row)),
        )
    }
    fn ospf_unit_geometry(&mut self, ctx: &egui::Context, scale: f32) -> bool {
        let id = egui::Id::new(("ospf-unit-geometry", self.smoke_stage));
        let current = ctx.content_rect();
        let last = ctx.data(|d| d.get_temp::<egui::Rect>(id));
        ctx.data_mut(|d| d.insert_temp(id, current));
        if (ctx.zoom_factor() - scale).abs() <= 0.01 && last == Some(current) {
            return true;
        }
        let tries = egui::Id::new(("ospf-unit-geometry-count", self.smoke_stage));
        let n = ctx.data_mut(|d| {
            let n = d.get_temp::<usize>(tries).unwrap_or(0) + 1;
            d.insert_temp(tries, n);
            n
        });
        assert!(
            n <= 30,
            "native OSPF geometry did not settle: {current:?},zoom={}",
            ctx.zoom_factor()
        );
        ctx.request_repaint();
        false
    }
    fn ospf_unit_exact_resume(&mut self, i: usize, view: teaching::View) {
        let attempts = serde_json::to_vec(&self.progress.attempts).unwrap();
        let favorites = self.progress.favorites.clone();
        let clock = self.progress.clock;
        progress::save(&self.path, &self.progress).unwrap();
        self.progress = progress::load(&self.path, &self.scenarios).unwrap();
        assert_eq!(
            serde_json::to_vec(&self.progress.attempts).unwrap(),
            attempts,
            "save/load must preserve every attempt, both current states and raw drafts exactly"
        );
        assert_eq!(self.progress.favorites, favorites);
        assert_eq!(self.progress.clock, clock);
        self.launch(i);
        self.set_ospf_view(i, view);
    }
    pub(super) fn ospf_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert!((OSPF_UNIT_START..OSPF_UNIT_START + 4 * OSPF_UNIT_STRIDE).contains(&stage));
        let ordinal = (stage - OSPF_UNIT_START) / OSPF_UNIT_STRIDE;
        let phase = (stage - OSPF_UNIT_START) % OSPF_UNIT_STRIDE;
        let id = OSPF_UNIT_IDS[ordinal];
        let i=self.scenarios.iter().position(|s|s.id==id).expect("OSPF acceptance requires the actual normal bundled lesson, not a private added fixture");
        assert!(
            self.units
                .iter()
                .any(|u| u.id == "ospf-p2p-foundations" && u.labs.iter().any(|s| s == id))
        );
        let s = self.scenarios[i].clone();
        let lesson = s.ospf_teaching.as_ref().unwrap();
        let mut reference = s.initial.clone();
        assert!(s.solution.len() <= 16);
        for c in &s.solution {
            engine::execute_for_goal(&mut reference, &s.goal, c)
                .expect("validated typed reference target");
        }
        let reference = ospf_unit_sim(&reference).clone();
        let plans: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/ospf-content-review.json"
        ))
        .unwrap();
        let plan = plans["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == id)
            .unwrap();
        let variant = |section: &str, name: &str| -> Vec<Action> {
            let row = plan[section]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p["name"] == name)
                .expect("reviewed named native variant");
            assert!(
                row["actions"].as_array().unwrap().len() <= 16,
                "fixed native variant action bound"
            );
            serde_json::from_value(row["actions"].clone()).unwrap()
        };
        let (partial_name, wrong_name, alternate_name) = match ordinal {
            0 => (
                "enable-only",
                "move-the-protected-hub-id",
                "other-unique-identity",
            ),
            1 => ("area-only", "process-number-only", "repair-peer-timers"),
            2 => ("read-only", "wrong-direction", "higher-equal-pair"),
            3 => (
                "only-transport-and-process",
                "disable-instead-of-passive",
                "repair-hub-endpoint",
            ),
            _ => unreachable!(),
        };
        let initial = ospf_unit_sim(&s.initial);
        let partial = ospf_unit_actions(initial, &variant("negatives", partial_name));
        let wrong = ospf_unit_actions(initial, &variant("negatives", wrong_name));
        let alternate = ospf_unit_actions(initial, &variant("alternates", alternate_name));
        let worked = ospf_unit_actions(
            &lesson.fixture,
            &lesson
                .example_actions
                .iter()
                .map(|x| x.action.clone())
                .collect::<Vec<_>>(),
        );
        let experiment = ospf_unit_actions(&worked, &lesson.try_change.actions);
        let repaired_example = ospf_unit_actions(&experiment, &lesson.repair);
        assert_eq!(repaired_example, worked);
        let first_peer = if let engine::Goal::Ospf { requirements, .. } = &s.goal {
            requirements
                .iter()
                .find_map(|r| {
                    if let ospf_model::Requirement::Compatibility { local, .. } = r {
                        Some(local.clone())
                    } else {
                        None
                    }
                })
                .unwrap()
        } else {
            unreachable!()
        };
        let mut broken = alternate.clone();
        let mut cfg = broken.configuration().routers[&first_peer.router].interfaces
            [&first_peer.interface]
            .clone();
        cfg.enabled = false;
        broken
            .apply_against(
                initial,
                &Action::SetPort {
                    endpoint: first_peer,
                    configuration: cfg,
                },
            )
            .unwrap();
        let mut invalid = Draft::new(&alternate).unwrap();
        invalid
            .routers
            .values_mut()
            .next()
            .unwrap()
            .process_id
            .clear();
        let mut unfinished_practice = Draft::new(&worked).unwrap();
        unfinished_practice
            .routers
            .values_mut()
            .next()
            .unwrap()
            .area = "unfinished practice".into();
        let mut unfinished_assignment = Draft::new(&alternate).unwrap();
        unfinished_assignment
            .routers
            .values_mut()
            .next()
            .unwrap()
            .router_id = "unfinished assignment".into();
        let queued = match phase {
            7 => Some((true, Draft::new(&experiment).unwrap())),
            10 => Some((true, Draft::new(&repaired_example).unwrap())),
            17 => Some((false, Draft::new(&partial).unwrap())),
            20 => Some((false, Draft::new(&wrong).unwrap())),
            23 => Some((false, Draft::new(&reference).unwrap())),
            27 | 36 | 45 | 53 => Some((false, Draft::new(&alternate).unwrap())),
            32 => Some((false, invalid)),
            39 | 50 => Some((false, Draft::new(&broken).unwrap())),
            57 => Some((true, unfinished_practice)),
            59 => Some((false, unfinished_assignment)),
            _ => None,
        };
        if let Some((practice, target)) = queued {
            return (self.ospf_unit_edit_queue(ctx, i, practice, &target), None);
        }
        let mut shot = None;
        match phase {
            0 => {
                let others: BTreeMap<_, _> = self
                    .progress
                    .attempts
                    .iter()
                    .filter(|(n, _)| n.as_str() != id)
                    .collect();
                ctx.data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new(("ospf-unit-others", id)),
                        serde_json::to_vec(&others).unwrap(),
                    )
                });
                assert!(
                    !self.progress.attempts.contains_key(id),
                    "normal OSPF unit smoke requires a fresh private profile for its four new lesson IDs"
                );
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.filter = Filter {
                    track: "CCNA".into(),
                    unit: "ospf-p2p-foundations".into(),
                    ..Default::default()
                };
                self.launch(i);
                self.progress.favorites.insert(id.into());
            }
            1 => {
                if !self.ospf_unit_geometry(ctx, 1.) {
                    return (false, None);
                }
                assert_eq!(self.ospf_view(i), teaching::View::Introduction);
                shot = Some(format!("{id}-introduction"));
            }
            2 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.example_action) {
                    return (false, None);
                }
            }
            3 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.insert) {
                    return (false, None);
                }
            }
            4 => {
                let a = &self.progress.attempts[id];
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .ospf_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(l.simulation, lesson.fixture);
                assert!(!l.checked && !l.draft.matches(&l.simulation));
                assert_eq!(a.state, s.initial);
                shot = Some(format!("{id}-worked-fill-unapplied"));
            }
            5 | 8 | 11 | 15 | 18 | 21 | 25 | 29 | 34 | 37 | 41 | 46 | 51 | 54 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            6 | 12 => {
                let a = &self.progress.attempts[id];
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .ospf_learning
                    .as_ref()
                    .unwrap();
                assert!(l.checked && l.checks(lesson).iter().all(|r| r.0));
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, &s));
                if !self.ospf_unit_feedback_head(ctx) {
                    return (false, None);
                }
                shot = Some(format!(
                    "{id}-example-{}",
                    if phase == 6 {
                        "worked-ungraded"
                    } else {
                        "repaired-ungraded"
                    }
                ));
            }
            9 => {
                let l = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .ospf_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(
                    l.checks(lesson).iter().map(|r| r.0).collect::<Vec<_>>(),
                    lesson.try_change.expected
                );
                assert_eq!(self.progress.attempts[id].state, s.initial);
                if !self.ospf_unit_feedback_head(ctx) {
                    return (false, None);
                }
                shot = Some(format!("{id}-example-experiment"));
            }
            13 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.assignment_action) {
                    return (false, None);
                }
            }
            14 => {
                assert_eq!(self.ospf_view(i), teaching::View::Assignment);
                assert_eq!(self.progress.attempts[id].state, s.initial);
                assert!(!self.metrics.help_visible);
                shot = Some(format!("{id}-full-assignment"));
            }
            16 | 19 | 22 | 42 | 52 => {
                let a = &self.progress.attempts[id];
                assert!(!progress::current_solved(a, &s) && !self.metrics.completion);
                let rows = engine::checks(&a.state, &s.goal);
                assert!(rows.iter().any(|r| r.0) && rows.iter().any(|r| !r.0));
                assert!(rows.iter().filter(|r| !r.0).all(|r| !r.1.trim().is_empty()));
                if !self.ospf_unit_feedback_head(ctx) {
                    return (false, None);
                }
                shot = Some(format!(
                    "{id}-{}",
                    match phase {
                        16 => "initial-current-feedback",
                        19 => "partial-repair",
                        22 => "plausible-wrong",
                        42 => "applied-breakage-current",
                        _ => "challenge-dark-200-partial",
                    }
                ));
            }
            24 | 28 | 40 => {
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
                if phase == 40 {
                    shot = Some(format!("{id}-pending-edit-invalidates-success"));
                }
            }
            26 | 31 | 38 | 47 | 55 => {
                assert!(
                    self.metrics.completion
                        && progress::current_solved(&self.progress.attempts[id], &s)
                );
                assert_eq!(self.progress.attempts[id].step, 0);
                if phase == 47 || phase == 55 {
                    assert!(self.metrics.lesson_tabs.is_empty() && !self.metrics.help_visible);
                }
                if !self.ospf_unit_feedback_head(ctx) {
                    return (false, None);
                }
                shot = Some(format!(
                    "{id}-{}",
                    match phase {
                        26 => "reference-current-complete",
                        31 => "alternate-resumed",
                        38 => "malformed-repair",
                        47 => "challenge-light-150-repaired",
                        _ => "challenge-dark-200-repaired",
                    }
                ));
                if phase == 55 {
                    println!(
                        "Native OSPF rendered {id}: content_rect={:?},zoom={},theme={}",
                        ctx.content_rect(),
                        ctx.zoom_factor(),
                        self.theme_id
                    );
                }
            }
            30 => self.ospf_unit_exact_resume(i, teaching::View::Assignment),
            33 => {
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
                assert_eq!(ospf_unit_sim(&self.progress.attempts[id].state), &alternate);
            }
            35 => {
                assert_eq!(
                    ospf_unit_sim(&self.progress.attempts[id].state),
                    &alternate,
                    "invalid fields must not partially apply"
                );
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
                assert!(
                    ospf_unit_draft(&self.progress.attempts[id], false)
                        .notice
                        .is_some()
                );
                let once = egui::Id::new(("ospf-unit-malformed-resume", self.smoke_stage));
                if !ctx.data(|d| d.get_temp::<bool>(once)).unwrap_or(false) {
                    self.ospf_unit_exact_resume(i, teaching::View::Assignment);
                    ctx.data_mut(|d| d.insert_temp(once, true));
                    self.started = Instant::now();
                    ctx.request_repaint();
                    return (false, None);
                }
                if !self.ospf_unit_feedback_head(ctx) {
                    return (false, None);
                }
                shot = Some(format!("{id}-malformed-fields-resumed"));
            }
            43 => {
                self.progress.attempts.get_mut(id).unwrap().challenge = true;
                self.help_open = false;
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 900.)));
                self.lesson_scroll_reset = true;
            }
            44 => {
                if !self.ospf_unit_geometry(ctx, 1.5) {
                    return (false, None);
                }
                assert!(self.metrics.lesson_tabs.is_empty() && !self.metrics.help_visible);
                shot = Some(format!("{id}-challenge-light-150-task"));
            }
            48 => {
                self.scale = 2.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            49 => {
                if !self.ospf_unit_geometry(ctx, 2.) {
                    return (false, None);
                }
                shot = Some(format!("{id}-challenge-dark-200-task"));
            }
            56 => {
                self.progress.attempts.get_mut(id).unwrap().challenge = false;
                self.set_ospf_view(i, teaching::View::Example);
            }
            58 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.assignment_action) {
                    return (false, None);
                }
            }
            60 => {
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
                self.ospf_unit_exact_resume(i, teaching::View::Assignment);
                assert_eq!(
                    ospf_unit_draft(&self.progress.attempts[id], true)
                        .routers
                        .values()
                        .next()
                        .unwrap()
                        .area,
                    "unfinished practice"
                );
                assert_eq!(
                    ospf_unit_draft(&self.progress.attempts[id], false)
                        .routers
                        .values()
                        .next()
                        .unwrap()
                        .router_id,
                    "unfinished assignment"
                );
            }
            61 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.reset) {
                    return (false, None);
                }
            }
            62 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.confirm_reset) {
                    return (false, None);
                }
            }
            63 => {
                assert_eq!(self.progress.attempts[id].state, s.initial);
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
                assert!(self.progress.favorites.contains(id));
                let others: BTreeMap<_, _> = self
                    .progress
                    .attempts
                    .iter()
                    .filter(|(n, _)| n.as_str() != id)
                    .collect();
                assert_eq!(
                    serde_json::to_vec(&others).unwrap(),
                    ctx.data(|d| d.get_temp::<Vec<u8>>(egui::Id::new(("ospf-unit-others", id))))
                        .unwrap(),
                    "only selected OSPF reset may change a prior attempt"
                );
                assert_eq!(
                    self.progress.attempts[id]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .ospf_learning
                        .as_ref()
                        .unwrap()
                        .simulation,
                    lesson.fixture
                );
            }
            64 => {
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.lesson_scroll_reset = true;
            }
            65 => {
                if !self.ospf_unit_geometry(ctx, 1.) {
                    return (false, None);
                }
                shot = Some(format!("{id}-selected-reset"));
            }
            66 => {
                println!(
                    "PASS normal bundled native OSPF {id}: intro, separate fill-only/check/experiment/repair; native router/interface selection+text/toggle edits; partial/wrong/reference/alternate; pending/current invalidation; malformed atomicity; exact both-draft resume; Challenge light150/dark200 and selected reset. Snapshot only, not live protocol/forwarding or novice acceptance."
                );
                if ordinal < 3 {
                    self.smoke_stage = OSPF_UNIT_START + (ordinal + 1) * OSPF_UNIT_STRIDE - 1;
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            _ => panic!("unexpected normal OSPF smoke phase {phase}"),
        }
        (true, shot)
    }
}
