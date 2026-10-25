//! Fixed authored DHCP lesson acceptance. Only actual UI events publish changes.
use super::*;
use dlms_learning_lab::{
    dhcpv4::{Policy, State as Simulation},
    dhcpv4_workspace::{Action, Draft},
};
const FIRST: usize = 31000;
const STRIDE: usize = 82;
const LAST: usize = FIRST + STRIDE * 3;
const MAX_OPS: usize = 256;
const IDS: [&str; 3] = [
    "ccna-dhcp-held-options",
    "ccna-dhcp-relay-scope",
    "ccna-dhcp-lease-handover",
];
#[derive(Clone, Debug)]
enum Op {
    Client(String),
    Pool(String),
    Annotation(usize),
    Header(&'static str, &'static str),
    Control(String),
    Text(String, String),
    Policy(Policy),
}
#[derive(Clone)]
struct Queue {
    ops: Vec<Op>,
    next: usize,
    field_phase: u8,
    started: Instant,
    frames: usize,
}
fn simulation(s: &engine::State) -> &Simulation {
    let engine::State::DhcpV4 { simulation } = s else {
        panic!("DHCP lesson state")
    };
    simulation
}
fn draft(a: &Attempt, practice: bool) -> &Draft {
    let w = a.workspace.as_ref().unwrap();
    if practice {
        &w.dhcpv4_learning.as_ref().unwrap().draft
    } else {
        w.dhcpv4_draft.as_ref().unwrap()
    }
}
fn read_actions(s: &Scenario, key: &str) -> Vec<Action> {
    let rows: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/dhcpv4-content-review.json"
    ))
    .unwrap();
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == s.id)
        .unwrap();
    let v = if key == "alternative" {
        row["alternatives"][0].clone()
    } else {
        row[key].clone()
    };
    serde_json::from_value(v).expect("closed authored typed plan")
}
fn text(ops: &mut Vec<Op>, key: impl Into<String>, value: impl Into<String>) {
    let value = value.into();
    assert!(value.len() <= 64);
    ops.push(Op::Text(key.into(), value));
}
fn policy_ops(current: &Draft, wanted: &Policy) -> Vec<Op> {
    // Intentionally closed to these three tasks: no automatic pool/relay creation,
    // server toggles or arbitrarily large collection editing.
    assert_eq!(current.server_enabled, wanted.server_enabled);
    assert!(current.pools.keys().eq(wanted.pools.keys()));
    assert!(current.relays.keys().eq(wanted.relays.keys()));
    let mut ops = Vec::new();
    for (id, p) in &wanted.pools {
        let now = &current.pools[id];
        let mut edits = Vec::new();
        for (key, before, after) in [
            ("network", now.network.clone(), p.subnet.network.to_string()),
            ("prefix", now.prefix.clone(), p.subnet.prefix.to_string()),
            ("first", now.first.clone(), p.first.to_string()),
            ("last", now.last.clone(), p.last.to_string()),
            (
                "router",
                now.router.clone(),
                p.router.map_or(String::new(), |v| v.to_string()),
            ),
            (
                "lease-seconds",
                now.lease_seconds.clone(),
                p.lease_seconds.to_string(),
            ),
        ] {
            if before != after {
                text(&mut edits, key, after);
            }
        }
        assert_eq!(
            now.dns.len(),
            p.dns.len(),
            "review a real DNS add/remove route before extending this closed plan"
        );
        for (n, (a, b)) in now.dns.iter().zip(&p.dns).enumerate() {
            if *a != b.to_string() {
                edits.push(Op::Header("dns-header", "dns-0"));
                text(&mut edits, format!("dns-{n}"), b.to_string());
            }
        }
        assert!(
            now.excluded.len() <= 4
                && p.excluded.len() <= 4
                && now.reservations.len() <= 4
                && p.reservations.len() <= 4
        );
        let wanted_excluded: Vec<_> = p.excluded.iter().map(ToString::to_string).collect();
        if now.excluded != wanted_excluded {
            edits.push(Op::Header("excluded-header", "add-exclusion"));
            for n in (wanted_excluded.len()..now.excluded.len()).rev() {
                edits.push(Op::Control(format!("remove-exclusion-{n}")));
            }
            for (n, value) in wanted_excluded.iter().enumerate() {
                if n >= now.excluded.len() {
                    edits.push(Op::Control("add-exclusion".into()));
                }
                if now.excluded.get(n) != Some(value) {
                    text(&mut edits, format!("excluded-{n}"), value);
                }
            }
        }
        let wanted_reserved: Vec<_> = p
            .reservations
            .iter()
            .map(|(c, a)| (c.to_owned(), a.to_string()))
            .collect();
        let now_reserved: Vec<_> = now
            .reservations
            .iter()
            .map(|r| (r.client.clone(), r.address.clone()))
            .collect();
        if now_reserved != wanted_reserved {
            edits.push(Op::Header("reservations-header", "add-reservation"));
            for n in (wanted_reserved.len()..now_reserved.len()).rev() {
                edits.push(Op::Control(format!("remove-reservation-{n}")));
            }
            for (n, (client, address)) in wanted_reserved.iter().enumerate() {
                if n >= now_reserved.len() {
                    edits.push(Op::Control("add-reservation".into()));
                }
                if now_reserved.get(n).map(|r| &r.0) != Some(client) {
                    text(&mut edits, format!("reserved-client-{n}"), client);
                }
                if now_reserved.get(n).map(|r| &r.1) != Some(address) {
                    text(&mut edits, format!("reserved-address-{n}"), address);
                }
            }
        }
        if !edits.is_empty() {
            ops.push(Op::Pool(id.clone()));
            ops.extend(edits);
        }
    }
    for (id, r) in &wanted.relays {
        let now = &current.relays[id];
        assert_eq!(now.enabled, r.enabled);
        if now.target != r.target.to_string() || now.giaddr != r.giaddr.to_string() {
            let client = current
                .context
                .clients
                .iter()
                .find(|(_, c)| c.segment == *id)
                .unwrap()
                .0
                .clone();
            ops.push(Op::Client(client));
            if now.target != r.target.to_string() {
                text(&mut ops, "relay-target", r.target.to_string());
            }
            if now.giaddr != r.giaddr.to_string() {
                text(&mut ops, "relay-giaddr", r.giaddr.to_string());
            }
        }
    }
    ops.push(Op::Control("apply".into()));
    ops.push(Op::Policy(wanted.clone()));
    assert!(ops.len() <= MAX_OPS);
    ops
}
fn click_phase(ctx: &egui::Context, stage: usize) -> u8 {
    ctx.data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", stage))))
        .unwrap_or(0)
}
impl LabApp {
    #[cfg(test)]
    pub(in crate::ui) fn check_dhcpv4_lesson_catalog(&mut self) {
        assert!(self._smoke_profile.is_some());
        let labs = pack::load_catalog(&[
            include_str!("../../../scenarios/starter-v1.json"),
            include_str!("../../../scenarios/dhcpv4-client-v1.json"),
        ])
        .unwrap();
        for s in labs.into_iter().filter(|s| IDS.contains(&s.id.as_str())) {
            let existing = self
                .scenarios
                .iter()
                .find(|x| x.id == s.id)
                .expect("authored DHCP lessons must be in the normal catalog");
            assert_eq!(existing.initial, s.initial);
            assert_eq!(
                serde_json::to_value(&existing.goal).unwrap(),
                serde_json::to_value(&s.goal).unwrap()
            );
            assert_eq!(existing.solution, s.solution);
            assert_eq!(existing.unit, "dhcpv4-client-foundations");
        }
    }
    fn dhcp_lessons_geometry(&self, ctx: &egui::Context) -> bool {
        let rect = ctx.content_rect();
        let zoom = ctx.zoom_factor();
        let ppp = ctx.pixels_per_point();
        let scale = self.scale;
        assert!(ppp.is_finite() && ppp > 0.);
        let stable = ctx.data_mut(|d| {
            let id = egui::Id::new(("dhcp-lessons-geometry", self.smoke_stage));
            let (old, n, start) = d
                .get_temp::<(Option<egui::Rect>, usize, Instant)>(id)
                .unwrap_or((None, 0, Instant::now()));
            assert!(start.elapsed().as_secs() < 20);
            let n = if old == Some(rect) && (zoom - scale).abs() < 0.01 {
                n + 1
            } else {
                0
            };
            d.insert_temp(id, (Some(rect), n, start));
            n >= 2
        });
        if !stable {
            ctx.request_repaint();
        }
        stable
    }
    fn dhcp_lessons_target_stable(
        &self,
        ctx: &egui::Context,
        kind: &str,
        rect: egui::Rect,
        clip: egui::Rect,
    ) -> bool {
        let frame = ctx.cumulative_frame_nr();
        let tolerance = 1. / ctx.pixels_per_point();
        let key = egui::Id::new(("dhcp-lessons-stable", self.smoke_stage, kind));
        let ready = ctx.data_mut(|d| {
            let previous = d.get_temp::<(egui::Rect, egui::Rect, u8, u64)>(key);
            let n = previous.map_or(0, |(old, old_clip, n, old_frame)| {
                let same = (old.min - rect.min).length() <= tolerance
                    && (old.max - rect.max).length() <= tolerance
                    && (old_clip.min - clip.min).length() <= tolerance
                    && (old_clip.max - clip.max).length() <= tolerance;
                if same && old_frame != frame {
                    n.saturating_add(1)
                } else {
                    0
                }
            });
            d.insert_temp(key, (rect, clip, n, frame));
            n >= 2
        });
        if !ready {
            ctx.request_repaint();
        }
        ready
    }
    fn dhcp_lessons_click(&mut self, ctx: &egui::Context, rect: Option<egui::Rect>) -> bool {
        if click_phase(ctx, self.smoke_stage) >= 2 {
            return self.click_visible_smoke_control(ctx, None);
        }
        let rect = rect.expect("actual DHCP click target");
        let clip = self
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if !self.dhcp_lessons_target_stable(ctx, "click", rect, clip) {
            return false;
        }
        self.click_visible_smoke_control(ctx, Some(rect))
    }
    fn dhcp_lessons_control(&mut self, ctx: &egui::Context, key: &str) -> bool {
        if click_phase(ctx, self.smoke_stage) >= 2 {
            return self.dhcp_lessons_click(ctx, None);
        }
        let (_, r, enabled) = self
            .metrics
            .dhcp_controls
            .iter()
            .find(|(k, _, _)| k == key)
            .unwrap_or_else(|| panic!("actual DHCP control {key}"));
        assert!(*enabled, "actual DHCP control is enabled: {key}");
        self.dhcp_lessons_click(ctx, Some(*r))
    }
    pub(in crate::ui::smoke) fn dhcp_lessons_field(
        &mut self,
        ctx: &egui::Context,
        key: &str,
    ) -> bool {
        let (_, rect, id) = self
            .metrics
            .dhcp_fields
            .iter()
            .find(|(k, _, _)| k == key)
            .unwrap_or_else(|| panic!("actual DHCP field {key}"));
        let (rect, id) = (*rect, *id);
        if click_phase(ctx, self.smoke_stage) >= 2 {
            assert!(
                ctx.memory(|m| m.has_focus(id)),
                "DHCP field focus: stage={} key={key} actual={:?}",
                self.smoke_stage,
                ctx.memory(|m| m.focused())
            );
            return self.click_visible_smoke_control(ctx, None);
        }
        let root = self
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let clip = self.metrics.dhcp_field_clips[key].intersect(root);
        if !self.dhcp_lessons_target_stable(ctx, "field", rect, clip) {
            return false;
        }
        if !smoke_control_visible(rect, root, ctx.pixels_per_point()) {
            self.reveal_smoke_control(ctx, rect);
            return false;
        }
        let inner_key = egui::Id::new(("dhcp-lessons-inner-scroll", self.smoke_stage, key));
        if !smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
            // Rows inside exclusion/reservation lists need their own real wheel
            // target; a visible outer rectangle does not prove an editable row.
            if clip.height() < rect.height() {
                ctx.request_repaint();
                return false;
            }
            let tries = ctx.data_mut(|d| {
                let n = d.get_temp::<usize>(inner_key).unwrap_or(0) + 1;
                d.insert_temp(inner_key, n);
                n
            });
            assert!(
                tries <= 20,
                "DHCP inner field did not become visible: {key} rect={rect:?} clip={clip:?} stage={}",
                self.smoke_stage
            );
            let delta = if rect.bottom() > clip.bottom() {
                clip.bottom() - rect.bottom()
            } else {
                clip.top() - rect.top()
            };
            self.pending_events.extend([
                // A vertical area's painter clip may span the full outer UI.
                // Aim at this field's column, not the space between columns.
                egui::Event::PointerMoved(egui::pos2(rect.center().x, clip.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta.clamp(-200., 200.)),
                    modifiers: Default::default(),
                },
            ]);
            ctx.request_repaint();
            return false;
        }
        ctx.data_mut(|d| d.remove::<usize>(inner_key));
        self.click_smoke_control(ctx, Some(rect))
    }
    pub(in crate::ui::smoke) fn dhcp_lessons_type(&mut self, text: &str) {
        assert!(text.len() <= 64);
        for pressed in [true, false] {
            self.pending_events.push(egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            });
        }
        self.pending_events.push(egui::Event::Text(text.into()));
    }
    fn dhcp_lessons_open_menu(&mut self, ctx: &egui::Context, rect: egui::Rect) -> bool {
        // Use the shared hover/click/release acknowledgement. Splitting press
        // and release across scrolling frames can turn the opener into a drag.
        self.dhcp_lessons_click(ctx, Some(rect))
    }
    fn dhcp_lessons_picker(
        &mut self,
        ctx: &egui::Context,
        i: usize,
        practice: bool,
        op: &Op,
        ordinal: usize,
    ) -> bool {
        let id = egui::Id::new(("dhcp-lessons-picker", self.smoke_stage, ordinal));
        let phase = ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0);
        let current = draft(&self.progress.attempts[&self.scenarios[i].id], practice);
        let matched = match op {
            Op::Client(c) => current.selected_client == *c,
            Op::Pool(p) => current.selected_pool.as_ref() == Some(p),
            Op::Annotation(n) => {
                self.progress.attempts[&self.scenarios[i].id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .dhcpv4_learning
                    .as_ref()
                    .unwrap()
                    .selected_action
                    == *n
            }
            _ => unreachable!(),
        };
        if phase == 0 && matched {
            return true;
        }
        if phase == 0 {
            let rect = match op {
                Op::Client(_) => self.metrics.dhcp_client,
                Op::Pool(_) => self.metrics.dhcp_pool,
                Op::Annotation(_) => self
                    .metrics
                    .dhcp_controls
                    .iter()
                    .find(|(k, _, _)| k == "example-action-menu")
                    .map(|(_, r, _)| *r),
                _ => unreachable!(),
            };
            if !self.dhcp_lessons_open_menu(ctx, rect.expect("actual picker opener")) {
                return false;
            }
            ctx.data_mut(|d| d.insert_temp(id, 1u8));
            return false;
        }
        // A selected item disappears on release. Drain that completion before
        // inspecting changed selection or popup metrics.
        if click_phase(ctx, self.smoke_stage) >= 2 {
            let done = self.click_smoke_control(ctx, None);
            assert!(done && matched);
            ctx.data_mut(|d| d.remove::<u8>(id));
            return true;
        }
        let rect = match op {
            Op::Client(c) => self
                .metrics
                .dhcp_clients
                .iter()
                .find(|(k, _)| k == c)
                .map(|(_, r)| *r),
            Op::Pool(p) => self
                .metrics
                .dhcp_pools
                .iter()
                .find(|(k, _)| k == p)
                .map(|(_, r)| *r),
            Op::Annotation(n) => self
                .metrics
                .dhcp_example_actions
                .iter()
                .find(|(k, _)| k == n)
                .map(|(_, r)| *r),
            _ => unreachable!(),
        };
        let Some(rect) = rect else {
            ctx.request_repaint();
            return false;
        };
        let clip = self
            .metrics
            .dhcp_popup
            .expect("actual DHCP popup clip")
            .intersect(ctx.content_rect());
        if !self.dhcp_lessons_target_stable(ctx, "picker", rect, clip) {
            return false;
        }
        if !smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
            let delta = if rect.bottom() > clip.bottom() {
                clip.bottom() - rect.bottom()
            } else {
                clip.top() - rect.top()
            };
            self.pending_events
                .push(egui::Event::PointerMoved(clip.center()));
            self.pending_events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., delta.clamp(-280., 280.)),
                modifiers: Default::default(),
            });
            ctx.request_repaint();
            return false;
        }
        self.click_smoke_control(ctx, Some(rect))
    }
    fn dhcp_lessons_ops(
        &mut self,
        ctx: &egui::Context,
        i: usize,
        practice: bool,
        ops: Vec<Op>,
        salt: usize,
    ) -> bool {
        assert!(ops.len() <= MAX_OPS);
        let key = egui::Id::new(("dhcp-lessons-ops", self.smoke_stage, salt));
        let mut q = ctx.data(|d| d.get_temp::<Queue>(key)).unwrap_or(Queue {
            ops,
            next: 0,
            field_phase: 0,
            started: Instant::now(),
            frames: 0,
        });
        q.frames += 1;
        assert!(
            q.frames <= 1800 && q.started.elapsed().as_secs() < 120,
            "bounded fixed DHCP native queue: stage={} salt={salt} next={} op={:?}",
            self.smoke_stage,
            q.next,
            q.ops.get(q.next)
        );
        if q.next == q.ops.len() {
            ctx.data_mut(|d| d.remove::<Queue>(key));
            return true;
        }
        let op = q.ops[q.next].clone();
        let done = match &op {
            Op::Client(_) | Op::Pool(_) | Op::Annotation(_) => {
                self.dhcp_lessons_picker(ctx, i, practice, &op, q.next)
            }
            Op::Control(k) => self.dhcp_lessons_control(ctx, k),
            Op::Header(header, child) => {
                if click_phase(ctx, self.smoke_stage) >= 2 {
                    self.dhcp_lessons_click(ctx, None)
                } else if self
                    .metrics
                    .dhcp_controls
                    .iter()
                    .any(|(k, _, _)| k == child)
                    || self.metrics.dhcp_fields.iter().any(|(k, _, _)| k == child)
                {
                    true
                } else {
                    self.dhcp_lessons_control(ctx, header)
                }
            }
            Op::Text(field, value) => match q.field_phase {
                0 => {
                    if self.dhcp_lessons_field(ctx, field) {
                        q.field_phase = 1;
                    }
                    false
                }
                1 => {
                    self.dhcp_lessons_type(value);
                    q.field_phase = 2;
                    false
                }
                _ => {
                    let id = self
                        .metrics
                        .dhcp_fields
                        .iter()
                        .find(|(k, _, _)| k == field)
                        .unwrap()
                        .2;
                    assert!(ctx.memory(|m| m.has_focus(id)));
                    q.field_phase = 0;
                    true
                }
            },
            Op::Policy(wanted) => {
                let actual = if practice {
                    self.progress.attempts[&self.scenarios[i].id]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .dhcpv4_learning
                        .as_ref()
                        .unwrap()
                        .simulation
                        .policy()
                } else {
                    simulation(&self.progress.attempts[&self.scenarios[i].id].state).policy()
                };
                assert_eq!(
                    actual, wanted,
                    "policy must originate from native typed fields and Apply"
                );
                true
            }
        };
        if done {
            q.next += 1;
            // This stage contains multiple verified operations. Keep the shared
            // twenty-scroll limit per completed target, plus the queue's fixed
            // operation/frame/time ceilings; never reset a stuck target.
            ctx.data_mut(|d| {
                d.remove::<usize>(egui::Id::new(("bounded-native-scroll", self.smoke_stage)))
            });
        }
        ctx.data_mut(|d| d.insert_temp(key, q));
        ctx.request_repaint();
        false
    }
    pub(in crate::ui::smoke) fn dhcp_lessons_actions(
        &mut self,
        ctx: &egui::Context,
        i: usize,
        practice: bool,
        actions: &[Action],
        annotations: bool,
    ) -> bool {
        assert!(actions.len() <= 16);
        let id = egui::Id::new(("dhcp-lessons-action", self.smoke_stage));
        let next = ctx.data(|d| d.get_temp::<usize>(id)).unwrap_or(0);
        if next == actions.len() {
            ctx.data_mut(|d| d.remove::<usize>(id));
            return true;
        }
        let mut ops = if annotations {
            vec![Op::Annotation(next)]
        } else {
            Vec::new()
        };
        match &actions[next] {
            Action::ApplyPolicy { policy } => ops.extend(policy_ops(
                draft(&self.progress.attempts[&self.scenarios[i].id], practice),
                policy,
            )),
            Action::Advance { seconds } => {
                text(&mut ops, "time", seconds.to_string());
                ops.push(Op::Control("advance".into()));
            }
            action => {
                let (client, key) = match action {
                    Action::Discover { client } => (client, "discover"),
                    Action::Request { client, .. } => (client, "request"),
                    Action::Renew { client } => (client, "renew"),
                    Action::Rebind { client } => (client, "rebind"),
                    Action::Release { client } => (client, "release"),
                    _ => unreachable!(),
                };
                ops.push(Op::Client(client.clone()));
                ops.push(Op::Control(key.into()));
            }
        }
        // Request clicks the current actual returned offer; authored token values
        // are never installed or substituted for the UI's own token selection.
        if self.dhcp_lessons_ops(ctx, i, practice, ops, next) {
            ctx.data_mut(|d| d.insert_temp(id, next + 1));
        }
        false
    }
    pub(in crate::ui::smoke) fn dhcp_lessons_feedback(
        &mut self,
        ctx: &egui::Context,
        i: usize,
        practice: bool,
        expected: bool,
    ) -> bool {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if practice {
            let l = a
                .workspace
                .as_ref()
                .unwrap()
                .dhcpv4_learning
                .as_ref()
                .unwrap();
            assert!(l.checked);
            assert_eq!(
                l.checks(self.scenarios[i].dhcpv4_teaching.as_ref().unwrap())
                    .iter()
                    .all(|r| r.0),
                expected
            );
        } else {
            assert_eq!(progress::current_solved(a, &self.scenarios[i]), expected);
            assert_eq!(self.metrics.completion, expected);
        }
        let full = self.metrics.feedback.expect("actual DHCP feedback");
        let clip = self
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        // At large text sizes the notice, results and unit navigation may need
        // normal scrolling. Capture the complete current result through the
        // footer together, without requiring an older operation notice to fit
        // in that same screenshot. Failed rows always retain the full target.
        let rect = if expected && full.height() > clip.height() {
            let summary = self
                .metrics
                .dhcp_result_summary
                .expect("actual result summary");
            egui::Rect::from_min_max(egui::pos2(full.left(), summary.top()), full.max)
        } else {
            full
        };
        if !self.dhcp_lessons_target_stable(ctx, "feedback", rect, clip) {
            return false;
        }
        self.reveal_smoke_control(ctx, rect)
    }
    pub(in crate::ui) fn dhcpv4_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert!((FIRST..=LAST).contains(&stage));
        // egui may rerender a frame for measurement. Deliver at most one input
        // batch per real frame, never advance a queue before that input renders.
        let frame = ctx.cumulative_frame_nr();
        let fresh = ctx.data_mut(|d| {
            let key = egui::Id::new("dhcp-lessons-controller-frame");
            if d.get_temp::<u64>(key) == Some(frame) {
                false
            } else {
                d.insert_temp(key, frame);
                true
            }
        });
        if !fresh {
            ctx.request_repaint();
            return (false, None);
        }
        assert!(self.smoke && self._smoke_profile.is_some());
        assert_eq!(
            self.path.parent(),
            Some(self._smoke_profile.as_ref().unwrap().path())
        );
        if stage == LAST {
            #[cfg(not(test))]
            println!(
                "PASS native three authored DHCP lessons through actual fields, selection, Apply, exchanges, Check, examples, experiments, alternates, invalidation, warm save/readback/resume/reset and light150/dark200 typing. Not cold-process, novice, physical network, routed/DNS connectivity or cross-platform acceptance."
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return (false, None);
        }
        let ordinal = (stage - FIRST) / STRIDE;
        let phase = (stage - FIRST) % STRIDE;
        let id = IDS[ordinal];
        let i =
            self.scenarios.iter().position(|s| s.id == id).expect(
                "authored DHCP lessons must actually be registered; no normal fixture append",
            );
        let s = self.scenarios[i].clone();
        let lesson = s.dhcpv4_teaching.as_ref().unwrap();
        let reference = read_actions(&s, "reference");
        let desired = match &reference[0] {
            Action::ApplyPolicy { policy } => policy.clone(),
            _ => unreachable!(),
        };
        let metadata: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/dhcpv4-content-review.json"
        ))
        .unwrap();
        let row = metadata
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == id)
            .unwrap();
        let work = row["work_pool"].as_str().unwrap();
        let first = row["first_client"].as_str().unwrap();
        let current_key = egui::Id::new(("dhcp-lessons-resume", id));
        let mut shot = None;
        match phase {
            0 => {
                assert!(
                    !self.progress.attempts.contains_key(id),
                    "fixed flow starts each lesson fresh"
                );
                self.launch(i);
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                self.progress.favorites.insert(id.into());
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            }
            1 | 45 | 52 => {
                if !self.dhcp_lessons_geometry(ctx) {
                    return (false, None);
                }
            }
            2 => {
                assert_eq!(self.dhcpv4_view(i), teaching::View::Introduction);
                shot = Some(format!("{id}-introduction"));
            }
            3 => {
                if !self.dhcp_lessons_click(ctx, self.metrics.example_action) {
                    return (false, None);
                }
            }
            4 => {
                assert_eq!(self.dhcpv4_view(i), teaching::View::Example);
                shot = Some(format!("{id}-separate-example"));
            }
            5 => {
                if !self.dhcp_lessons_click(ctx, self.metrics.insert) {
                    return (false, None);
                }
            }
            6 => {
                let l = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .dhcpv4_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(l.simulation, lesson.fixture);
                assert!(!l.draft.matches(&l.simulation));
                assert_eq!(self.progress.attempts[id].state, s.initial);
                shot = Some(format!("{id}-fill-only"));
            }
            7 => {
                let actions: Vec<_> = lesson
                    .example_actions
                    .iter()
                    .map(|a| a.action.clone())
                    .collect();
                if !self.dhcp_lessons_actions(ctx, i, true, &actions, true) {
                    return (false, None);
                }
            }
            8 | 11 | 14 | 17 | 20 | 26 | 32 | 36 | 39 | 42 | 49 | 56 | 73 => {
                if !self.dhcp_lessons_click(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            9 | 15 => {
                if !self.dhcp_lessons_feedback(ctx, i, true, true) {
                    return (false, None);
                }
                assert_eq!(self.progress.attempts[id].state, s.initial);
                shot = Some(format!(
                    "{id}-example-{}",
                    if phase == 9 { "worked" } else { "repaired" }
                ));
            }
            10 => {
                if !self.dhcp_lessons_actions(ctx, i, true, &lesson.try_change.actions, false) {
                    return (false, None);
                }
            }
            12 => {
                let l = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .dhcpv4_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(
                    l.checks(lesson).iter().map(|r| r.0).collect::<Vec<_>>(),
                    lesson.try_change.expected
                );
                if !self.dhcp_lessons_feedback(ctx, i, true, false) {
                    return (false, None);
                }
                shot = Some(format!("{id}-example-experiment"));
            }
            13 => {
                if !self.dhcp_lessons_actions(ctx, i, true, &lesson.repair, false) {
                    return (false, None);
                }
            }
            16 | 24 | 30 | 71 => {
                if phase != 16 {
                    assert_eq!(self.progress.attempts[id].state, s.initial);
                }
                if phase == 71 {
                    let a = &self.progress.attempts[id];
                    let l = a
                        .workspace
                        .as_ref()
                        .unwrap()
                        .dhcpv4_learning
                        .as_ref()
                        .unwrap();
                    assert_eq!(l.simulation, lesson.fixture);
                    assert!(l.draft.matches(&l.simulation));
                    assert!(draft(a, false).matches(simulation(&s.initial)));
                    let (_, _, others) = ctx
                        .data(|d| {
                            d.get_temp::<(serde_json::Value, u64, serde_json::Value)>(current_key)
                        })
                        .unwrap();
                    let mut now = serde_json::to_value(&self.progress.attempts).unwrap();
                    now.as_object_mut().unwrap().remove(id);
                    assert_eq!(now, others);
                }
                if !self.dhcp_lessons_click(ctx, self.metrics.lesson_tabs.get(2).copied()) {
                    return (false, None);
                }
                if phase == 16 {
                    shot = Some(format!("{id}-full-assignment"));
                }
            }
            18 | 21 | 27 | 37 | 40 => {
                if phase == 40 {
                    let engine::Goal::DhcpV4 { requirements, .. } = &s.goal else {
                        unreachable!()
                    };
                    let rows = engine::checks(&self.progress.attempts[id].state, &s.goal);
                    assert!(rows[..requirements.len()].iter().all(|r| r.0));
                    assert!(rows[requirements.len()..].iter().any(|r| !r.0));
                }
                if phase == 21 && ordinal == 1 {
                    assert_eq!(
                        simulation(&self.progress.attempts[id].state).leases()[first]
                            .parameters
                            .subnet
                            .prefix,
                        16
                    );
                }
                if !self.dhcp_lessons_feedback(ctx, i, false, false) {
                    return (false, None);
                }
                shot = Some(format!(
                    "{id}-{}",
                    match phase {
                        18 => "initial-feedback",
                        21 => "wrong-feedback",
                        27 => "partial-feedback",
                        37 => "pending-invalidates",
                        _ => "new-client-policy-fails",
                    }
                ));
            }
            19 => {
                let wrong = if ordinal == 0 {
                    vec![Action::Renew {
                        client: first.into(),
                    }]
                } else if ordinal == 1 {
                    let mut p = simulation(&s.initial).policy().clone();
                    p.relays.get_mut(work).unwrap().target =
                        simulation(&s.initial).topology().server;
                    vec![
                        Action::ApplyPolicy { policy: p },
                        Action::Discover {
                            client: first.into(),
                        },
                        Action::Request {
                            client: first.into(),
                            token: 0,
                        },
                    ]
                } else {
                    let mut p = desired.clone();
                    p.pools.get_mut(work).unwrap().dns = vec!["203.0.113.200".parse().unwrap()];
                    let mut actions = reference.clone();
                    actions[0] = Action::ApplyPolicy { policy: p };
                    actions
                };
                if !self.dhcp_lessons_actions(ctx, i, false, &wrong, false) {
                    return (false, None);
                }
            }
            22 | 28 | 66 | 69 => {
                if !self.dhcp_lessons_click(ctx, self.metrics.reset) {
                    return (false, None);
                }
            }
            23 | 29 | 70 => {
                if !self.dhcp_lessons_click(ctx, self.metrics.confirm_reset) {
                    return (false, None);
                }
            }
            25 => {
                if !self.dhcp_lessons_actions(ctx, i, false, &read_actions(&s, "partial"), false) {
                    return (false, None);
                }
            }
            31 => {
                if !self.dhcp_lessons_actions(ctx, i, false, &reference, false) {
                    return (false, None);
                }
            }
            33 | 43 | 50 | 57 | 74 => {
                if phase == 50 || phase == 57 {
                    let value =
                        desired.pools[work].lease_seconds + if phase == 50 { 8 } else { 16 };
                    let a = &self.progress.attempts[id];
                    let raw = draft(a, false);
                    assert_eq!(raw.selected_pool.as_deref(), Some(work));
                    assert_eq!(
                        raw.pools[work].lease_seconds,
                        value.to_string(),
                        "scale proof requires actual new editor text"
                    );
                    assert_eq!(
                        simulation(&a.state).policy().pools[work].lease_seconds,
                        value,
                        "scale proof requires Apply to publish the typed duration"
                    );
                    let key = egui::Id::new((
                        "dhcp-lessons-scale-held",
                        id,
                        if phase == 50 { 44usize } else { 51usize },
                    ));
                    let before = ctx
                        .data(|d| d.get_temp::<serde_json::Value>(key))
                        .expect("observed held grants before policy typing");
                    assert_eq!(
                        serde_json::to_value(simulation(&a.state).leases()).unwrap(),
                        before,
                        "Apply must not refresh held leases"
                    );
                    let log_key = egui::Id::new(("dhcp-scale-proof-log", id, phase));
                    let first = ctx.data_mut(|d| {
                        if d.get_temp::<bool>(log_key).unwrap_or(false) {
                            false
                        } else {
                            d.insert_temp(log_key, true);
                            true
                        }
                    });
                    if first {
                        let rect = ctx.content_rect();
                        let ppp = ctx.pixels_per_point();
                        let zoom = ctx.zoom_factor();
                        println!(
                            "DHCP_TYPED_SCALE id={id} phase={phase} duration={value} actual_logical_rect={rect:?} actual_ppp={ppp} actual_zoom={zoom}"
                        );
                    }
                }
                if !self.dhcp_lessons_feedback(ctx, i, false, true) {
                    return (false, None);
                }
                shot = Some(format!(
                    "{id}-{}",
                    match phase {
                        33 => "reference-complete",
                        43 => "repaired",
                        50 => "light150-typed",
                        57 => "dark200-typed",
                        _ => "alternate-complete",
                    }
                ));
            }
            34 => {
                if !self.dhcp_lessons_ops(
                    ctx,
                    i,
                    false,
                    vec![
                        Op::Pool(work.into()),
                        Op::Text(
                            "router".into(),
                            desired.pools[work]
                                .subnet
                                .network
                                .to_string()
                                .rsplit_once('.')
                                .map(|(b, _)| format!("{b}.254"))
                                .unwrap(),
                        ),
                    ],
                    0,
                ) {
                    return (false, None);
                }
            }
            35 => {
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
            }
            38 => {
                if !self.dhcp_lessons_control(ctx, "apply") {
                    return (false, None);
                }
            }
            41 => {
                if !self.dhcp_lessons_actions(
                    ctx,
                    i,
                    false,
                    &[Action::ApplyPolicy {
                        policy: desired.clone(),
                    }],
                    false,
                ) {
                    return (false, None);
                }
            }
            44 | 51 => {
                let before =
                    serde_json::to_value(simulation(&self.progress.attempts[id].state).leases())
                        .unwrap();
                let key = egui::Id::new(("dhcp-lessons-scale-held", id, phase));
                ctx.data_mut(|d| d.insert_temp(key, before));
                self.scale = if phase == 44 { 1.5 } else { 2. };
                self.theme_id = if phase == 44 {
                    "dlms-light"
                } else {
                    "dlms-dark"
                }
                .into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            }
            46 | 53 | 58 => {
                if !self.dhcp_lessons_field(ctx, "lease-seconds") {
                    return (false, None);
                }
                if phase != 58 {
                    shot = Some(format!(
                        "{id}-{}-focused-policy-field",
                        if phase == 46 { "light150" } else { "dark200" }
                    ));
                }
            }
            47 | 54 => self.dhcp_lessons_type(
                &(desired.pools[work].lease_seconds + if phase == 47 { 8 } else { 16 }).to_string(),
            ),
            48 | 55 => {
                if !self.dhcp_lessons_control(ctx, "apply") {
                    return (false, None);
                }
            }
            59 => self.dhcp_lessons_type("unfinished"),
            60 => {
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
                self.save();
                let loaded = progress::load(&self.path, &self.scenarios)
                    .expect("actual saved private profile");
                assert_eq!(
                    serde_json::to_value(&loaded).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                let snapshot = serde_json::to_value(&self.progress.attempts[id]).unwrap();
                let mut others = serde_json::to_value(&self.progress.attempts).unwrap();
                others.as_object_mut().unwrap().remove(id);
                let clock = self.progress.clock;
                ctx.data_mut(|d| d.insert_temp(current_key, (snapshot, clock, others)));
            }
            61 | 78 => {
                if !self.dhcp_lessons_click(ctx, self.metrics.back) {
                    return (false, None);
                }
            }
            62 => {
                assert!(self.selected.is_none());
                self.filter = Filter {
                    track: "CCNA".into(),
                    ..Default::default()
                };
                self.filter.search = s.title.clone();
                self.page = 0;
            }
            63 => {
                if click_phase(ctx, self.smoke_stage) >= 2 {
                    if !self.dhcp_lessons_click(ctx, None) {
                        return (false, None);
                    }
                } else {
                    assert!(self.selected.is_none());
                    assert_eq!(
                        self.metrics.card_actions.len(),
                        1,
                        "one actual filtered resume card"
                    );
                    if !self.dhcp_lessons_click(ctx, self.metrics.card_actions.first().map(|r| r.0))
                    {
                        return (false, None);
                    }
                }
            }
            64 => {
                assert_eq!(self.selected, Some(i));
                let (mut saved, clock, _) = ctx
                    .data(|d| {
                        d.get_temp::<(serde_json::Value, u64, serde_json::Value)>(current_key)
                    })
                    .unwrap();
                assert_eq!(self.progress.clock, clock + 1);
                saved["visited"] = (clock + 1).into();
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[id]).unwrap(),
                    saved
                );
            }
            65 => {
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
                shot = Some(format!("{id}-pending-warm-resume"));
            }
            67 => {
                if !self.dhcp_lessons_click(ctx, self.metrics.reset_cancel) {
                    return (false, None);
                }
            }
            68 => {
                assert_eq!(
                    draft(&self.progress.attempts[id], false).pools[work].lease_seconds,
                    "unfinished"
                );
            }
            72 => {
                assert!(self.progress.favorites.contains(id));
                if !self.dhcp_lessons_actions(
                    ctx,
                    i,
                    false,
                    &read_actions(&s, "alternative"),
                    false,
                ) {
                    return (false, None);
                }
            }
            75 => {
                if !self.dhcp_lessons_click(ctx, self.metrics.mode_challenge) {
                    return (false, None);
                }
            }
            76 => {
                assert!(self.progress.attempts[id].challenge);
                assert!(self.metrics.lesson_tabs.is_empty());
                assert!(!self.metrics.help_visible);
                assert!(progress::current_solved(&self.progress.attempts[id], &s));
            }
            77 => shot = Some(format!("{id}-challenge")),
            79 => {
                assert!(self.selected.is_none());
                assert!(self.progress.favorites.contains(id));
            }
            80 => {
                self.filter = Filter {
                    track: "CCNA".into(),
                    ..Default::default()
                };
            }
            81 => {
                assert!(progress::current_solved(&self.progress.attempts[id], &s));
            }
            _ => unreachable!("closed DHCP lesson phase"),
        }
        (true, shot)
    }
}
