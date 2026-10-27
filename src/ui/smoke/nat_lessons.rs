//! Three NORMAL authored lessons. Only real GUI events can mutate their workspaces.
use super::nat::{Op, Queue, Target, Warm};
use super::*;
use dlms_learning_lab::{nat as core, nat_workspace as work};
use serde_json::Value;
pub(in crate::ui) const FIRST: usize = 39000;
pub(in crate::ui) const STRIDE: usize = 72;
pub(in crate::ui) const LAST: usize = FIRST + 3 * STRIDE - 1;
pub(in crate::ui) const IDS: [&str; 3] = ["ccna-nat-static", "ccna-nat-pool", "ccna-nat-handover"];
const _: () = assert!(FIRST > super::nat::LAST && LAST == 39215 && LAST < 40000);
#[derive(Clone, Default)]
struct Actions {
    at: usize,
    sub: u8,
    ops: Option<Vec<Op>>,
    before: Option<Value>,
}
#[derive(Clone)]
struct CancelProof {
    progress: Value,
    receipts: Value,
    files: std::collections::BTreeMap<String, Vec<u8>>,
}
#[derive(Clone, Copy)]
enum Shot {
    Header,
    Rows,
    Failure,
    Focused,
    Observation,
    Library(usize),
}
fn key(stage: usize, name: &str) -> egui::Id {
    egui::Id::new(("nat-lessons", stage, name))
}
fn click(t: Target) -> Op {
    Op::Click(t)
}
fn text(k: &'static str, v: impl Into<String>) -> Op {
    Op::Text(k, v.into())
}
fn exact<A: serde::Serialize + ?Sized, B: serde::Serialize + ?Sized>(a: &A, b: &B, label: &str) {
    super::nat::exact(a, b, label)
}
impl LabApp {
    pub(in crate::ui) fn nat_lessons_stage(smoke: bool, name: Option<&str>) -> Option<usize> {
        (smoke && name == Some("nat-lessons")).then_some(FIRST)
    }
    fn nl_index(&self, id: &str) -> usize {
        self.scenarios
            .iter()
            .position(|s| s.id == id)
            .expect("normal authored NAT task")
    }
    fn nl_live(&self, id: &str, practice: bool) -> &core::State {
        let a = &self.progress.attempts[id];
        if practice {
            &a.workspace
                .as_ref()
                .unwrap()
                .nat_learning
                .as_ref()
                .unwrap()
                .simulation
        } else {
            let engine::State::BasicNat { simulation } = &a.state else {
                panic!("actual NAT State")
            };
            simulation
        }
    }
    fn nl_learning(&self, id: &str) -> &dlms_learning_lab::nat_teaching::Learning {
        self.progress.attempts[id]
            .workspace
            .as_ref()
            .unwrap()
            .nat_learning
            .as_ref()
            .unwrap()
    }
    fn nl_review(&self, id: &str) -> Value {
        let rows: Vec<Value> = serde_json::from_str(include_str!(
            "../../../tests/fixtures/nat-basic-review.json"
        ))
        .unwrap();
        rows.into_iter().find(|r| r["id"] == id).unwrap()
    }
    fn nl_plan(&self, id: &str, section: &str) -> Vec<core::Action> {
        let r = self.nl_review(id);
        serde_json::from_value(match section {
            "wrong" => r["wrong"][0]["actions"].clone(),
            "partial" => r["partial"]["actions"].clone(),
            "broken" => r["broken"]["actions"].clone(),
            "alt0" => r["alternatives"][0]["actions"].clone(),
            "alt1" => r["alternatives"][1]["actions"].clone(),
            _ => panic!("closed plan"),
        })
        .unwrap()
    }
    fn nl_reference(&self, id: &str) -> Vec<core::Action> {
        self.scenarios[self.nl_index(id)]
            .solution
            .iter()
            .map(|s| {
                core::Action::from_json(s.strip_prefix(work::PREFIX).unwrap().as_bytes()).unwrap()
            })
            .collect()
    }
    fn nl_value(&self, id: &str, practice: bool) -> Value {
        serde_json::to_value(self.nl_live(id, practice)).unwrap()
    }
    fn nl_put<T: Clone + Send + Sync + 'static>(
        &self,
        ctx: &egui::Context,
        lesson: usize,
        name: &str,
        value: T,
    ) {
        ctx.data_mut(|d| d.insert_temp(key(lesson, name), value));
    }
    fn nl_get<T: Clone + Send + Sync + 'static>(
        &self,
        ctx: &egui::Context,
        lesson: usize,
        name: &str,
    ) -> T {
        ctx.data(|d| d.get_temp::<T>(key(lesson, name)))
            .expect("actual earlier observation")
    }
    fn nl_group(
        &mut self,
        ctx: &egui::Context,
        id: &str,
        practice: bool,
        name: &str,
        ops: Vec<Op>,
    ) -> bool {
        assert!(ops.len() <= 24);
        let k = key(self.smoke_stage, name);
        if ctx.data(|d| d.get_temp::<bool>(k)) == Some(true) {
            return true;
        }
        if !self.nat_queue_for(ctx, ops, id, practice) {
            return false;
        }
        self.nat_reset_queue(ctx);
        ctx.data_mut(|d| d.insert_temp(k, true));
        // A completed click can change the view after this frame was painted.
        // Only the next render may acknowledge this group to a dependent group
        // (notably Example/Assignment tabs in phases42/44/69).
        ctx.request_repaint();
        false
    }
    fn nl_ops(&self, id: &str, practice: bool, a: &core::Action) -> Vec<Op> {
        let d = self.nat_event_draft(id, practice);
        let mut ops = Vec::new();
        match a {
            core::Action::Configure { configuration: c } => {
                assert!(d.statics.len() <= 32 && c.statics.len() <= 32);
                let mut rows = d
                    .statics
                    .iter()
                    .map(|r| (r.inside.clone(), r.global.clone()))
                    .collect::<Vec<_>>();
                for n in (0..rows.len()).rev() {
                    if !c.statics.contains_key(&rows[n].0) {
                        ops.push(click(Target::RemoveRow(n)));
                        rows.remove(n);
                    }
                }
                for (inside, g) in &c.statics {
                    let n = if let Some(n) = rows.iter().position(|r| &r.0 == inside) {
                        n
                    } else {
                        let n = rows.len();
                        ops.push(click(Target::Control("add-static")));
                        ops.push(Op::RowText(n, false, inside.clone()));
                        rows.push((inside.clone(), String::new()));
                        n
                    };
                    let value = g.to_string();
                    if rows[n].1 != value {
                        ops.push(Op::RowText(n, true, value));
                    }
                }
                if d.dynamic_enabled != c.dynamic.is_some() {
                    ops.push(click(Target::Control("dynamic")));
                }
                if let Some(r) = c.dynamic {
                    for (k, value, old) in [
                        ("network", r.source.network.to_string(), &d.network),
                        ("prefix", r.source.length.to_string(), &d.prefix),
                        ("first", r.pool.first.to_string(), &d.first),
                        ("last", r.pool.last.to_string(), &d.last),
                    ] {
                        if old != &value {
                            ops.push(text(k, value));
                        }
                    }
                }
                ops.push(click(Target::Control("apply")));
            }
            core::Action::ClearDynamic {} => ops.push(click(Target::Control("clear"))),
            core::Action::Outbound { inside, peer } => {
                if d.inside.as_ref() != Some(inside) {
                    ops.push(Op::Select("inside", inside.clone()));
                }
                if d.peer.as_ref() != Some(peer) {
                    ops.push(Op::Select("peer", peer.clone()));
                }
                ops.push(click(Target::Control("outbound")));
            }
            core::Action::Inbound { peer, global } => {
                if d.peer.as_ref() != Some(peer) {
                    ops.push(Op::Select("peer", peer.clone()));
                }
                let g = global.to_string();
                if d.inbound_global != g {
                    ops.push(text("inbound-global", g));
                }
                ops.push(click(Target::Control("inbound")));
            }
        }
        assert!(
            ops.len() <= 24,
            "split bounded GUI action, never silently skip"
        );
        ops
    }
    fn nl_action_proof(
        &self,
        id: &str,
        practice: bool,
        action: &core::Action,
        before: &Value,
        error: bool,
    ) {
        let live = self.nl_live(id, practice);
        let d = self.nat_event_draft(id, practice);
        let effect = self.nat_presentation.observed_effect();
        if error {
            assert!(matches!(action, core::Action::Configure { .. }));
            exact(live, before, "Rejected Apply leaves full State unchanged");
            assert!(
                d.notice
                    .as_ref()
                    .is_some_and(|s| s.contains("Clear dynamic bindings"))
            );
            assert!(!d.matches(live));
            if let core::Action::Configure { configuration } = action {
                assert_eq!(d.configuration().unwrap(), *configuration);
            }
            return;
        }
        match action {
            core::Action::Configure { configuration } => {
                assert_eq!(live.configuration(), configuration);
                assert_eq!(d.configuration().unwrap(), *configuration);
                assert!(matches!(effect, Some(core::Effect::Configured)));
            }
            core::Action::ClearDynamic {} => {
                assert!(live.bindings().is_empty());
                exact(
                    live.configuration(),
                    &before["configuration"],
                    "Clear preserves complete policy/static maps",
                );
                assert!(matches!(effect, Some(core::Effect::Cleared { .. })));
            }
            core::Action::Outbound { inside, peer } => {
                let Some(core::Effect::Probe {
                    result:
                        core::ProbeResult::Translated {
                            before: b,
                            after,
                            kind,
                        },
                }) = effect
                else {
                    panic!("actual positive outbound observation")
                };
                let (g, k) = live
                    .configuration()
                    .statics
                    .get(inside)
                    .map(|g| (*g, core::MappingKind::Static))
                    .or_else(|| {
                        live.bindings()
                            .get(inside)
                            .map(|g| (*g, core::MappingKind::Dynamic))
                    })
                    .unwrap();
                assert_eq!(
                    (
                        *kind,
                        b.source,
                        b.destination,
                        after.source,
                        after.destination
                    ),
                    (
                        k,
                        live.inventory().inside[inside],
                        live.inventory().outside[peer],
                        g,
                        b.destination
                    )
                );
                exact(
                    live.configuration(),
                    &before["configuration"],
                    "Outbound does not edit policy",
                );
            }
            core::Action::Inbound { peer, global } => {
                exact(
                    live,
                    before,
                    "Reverse never allocates or changes any State field",
                );
                let Some(core::Effect::Probe {
                    result:
                        core::ProbeResult::Translated {
                            before: b, after, ..
                        },
                }) = effect
                else {
                    panic!("actual positive reverse observation")
                };
                let local = live
                    .configuration()
                    .statics
                    .iter()
                    .chain(live.bindings())
                    .find_map(|(i, g)| (g == global).then_some(live.inventory().inside[i]))
                    .unwrap();
                assert_eq!(
                    (b.source, b.destination, after.source, after.destination),
                    (live.inventory().outside[peer], *global, b.source, local)
                );
            }
        }
    }
    fn nl_actions(
        &mut self,
        ctx: &egui::Context,
        id: &str,
        practice: bool,
        name: &str,
        items: &[core::Action],
        options: (Option<usize>, bool),
    ) -> bool {
        let (annotation, error) = options;
        assert!(!items.is_empty() && items.len() <= 16);
        let k = key(self.smoke_stage, name);
        if ctx.data(|d| d.get_temp::<bool>(k.with("done"))) == Some(true) {
            return true;
        }
        let mut c = ctx.data(|d| d.get_temp::<Actions>(k)).unwrap_or_default();
        assert!(c.at < items.len());
        let item = &items[c.at];
        if c.before.is_none() {
            c.before = Some(self.nl_value(id, practice));
        }
        if let Some(start) = annotation
            && c.sub == 0
        {
            let index = start + c.at;
            assert!(practice && index < 16);
            let ops = vec![Op::Example(index), click(Target::Insert)];
            if !self.nat_queue_for(ctx, ops, id, true) {
                ctx.data_mut(|d| d.insert_temp(k, c));
                return false;
            }
            exact(
                self.nl_live(id, true),
                c.before.as_ref().unwrap(),
                "Actual Insert does not execute any action",
            );
            self.nat_reset_queue(ctx);
            c.sub = 1;
        }
        if c.ops.is_none() {
            c.ops = Some(self.nl_ops(id, practice, item));
        }
        if !self.nat_queue_for(ctx, c.ops.as_ref().unwrap().clone(), id, practice) {
            ctx.data_mut(|d| d.insert_temp(k, c));
            return false;
        }
        self.nl_action_proof(id, practice, item, c.before.as_ref().unwrap(), error);
        self.nat_reset_queue(ctx);
        c.at += 1;
        c.sub = 0;
        c.ops = None;
        c.before = None;
        if c.at == items.len() {
            ctx.data_mut(|d| {
                d.remove::<Actions>(k);
                d.insert_temp(k.with("done"), true);
            });
            return true;
        }
        ctx.data_mut(|d| d.insert_temp(k, c));
        self.started = Instant::now();
        ctx.request_repaint();
        false
    }
    fn nl_rows(&self, id: &str, practice: bool) -> Vec<bool> {
        let i = self.nl_index(id);
        if practice {
            self.nl_learning(id)
                .checks(self.scenarios[i].nat_teaching.as_ref().unwrap())
                .iter()
                .map(|r| r.0)
                .collect()
        } else {
            progress::current_nat_checks(&self.progress.attempts[id], &self.scenarios[i])
                .iter()
                .map(|r| r.0)
                .collect()
        }
    }
    fn nl_pass(&self, id: &str, practice: bool, expected: bool) {
        let rows = self.nl_rows(id, practice);
        assert!(!rows.is_empty());
        assert_eq!(rows.iter().all(|r| *r), expected);
        if practice {
            assert!(!self.metrics.completion);
        } else {
            assert_eq!(self.metrics.completion, expected);
            assert_eq!(
                progress::current_solved(
                    &self.progress.attempts[id],
                    &self.scenarios[self.nl_index(id)]
                ),
                expected
            );
        }
    }
    fn nl_capture(&mut self, ctx: &egui::Context, id: &str, name: &str, target: Target) -> bool {
        assert!(name.len() <= 64);
        let k = key(self.smoke_stage, name);
        if ctx.data(|d| d.get_temp::<bool>(k.with("captured"))) == Some(true) {
            return true;
        }
        let rect = ctx.content_rect();
        let zoom = ctx.zoom_factor();
        let ppp = ctx.pixels_per_point();
        assert!(ppp.is_finite() && ppp > 0.);
        let geometry = ctx.data_mut(|d| {
            let (old, n, start) = d
                .get_temp::<(Option<(egui::Rect, f32, f32)>, usize, Instant)>(k.with("geometry"))
                .unwrap_or((None, 0, Instant::now()));
            assert!(
                start.elapsed().as_secs() < 15,
                "capture geometry must settle"
            );
            let n = if old == Some((rect, zoom, ppp)) && (zoom - self.scale).abs() < 0.01 {
                n + 1
            } else {
                0
            };
            d.insert_temp(k.with("geometry"), (Some((rect, zoom, ppp)), n, start));
            n >= 2
        });
        if !geometry {
            ctx.request_repaint();
            return false;
        }
        let (r, v, inner) = self.nat_target(ctx, &target);
        let mut q = ctx
            .data(|d| d.get_temp::<Queue>(k.with("reveal")))
            .unwrap_or_default();
        let visible = self.nat_reveal(ctx, r, v, inner, &mut q);
        ctx.data_mut(|d| d.insert_temp(k.with("reveal"), q));
        if !visible {
            return false;
        }
        if let Target::Field(ref field) = target {
            let input = self
                .metrics
                .nat_fields
                .iter()
                .find(|r| &r.0 == field)
                .unwrap();
            assert!(ctx.memory(|m| m.has_focus(input.2)));
        }
        let outer = ctx.input(|i| i.viewport().outer_rect);
        let fullscreen = ctx.input(|i| i.viewport().fullscreen);
        println!(
            "NAT lesson {id} {name}: actual_content={rect:?} actual_outer={outer:?} fullscreen={fullscreen:?} zoom={zoom} PPP={ppp}"
        );
        ctx.data_mut(|d| d.insert_temp(k.with("captured"), true));
        self.capture_pending = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
            format!("nat-lesson-{id}-{name}-zoom{zoom:.2}-ppp{ppp:.2}"),
        )));
        false
    }
    fn nl_shot(&mut self, ctx: &egui::Context, id: &str, name: &str, shot: Shot) -> bool {
        match shot {
            Shot::Header => self.nl_capture(ctx, id, name, Target::Back),
            Shot::Observation => self.nl_capture(ctx, id, name, Target::Observation),
            Shot::Library(index) => {
                assert_eq!(
                    self.metrics.card_actions.len(),
                    if self.filter.unit == "basic-nat" {
                        3
                    } else {
                        1
                    }
                );
                self.nl_capture(ctx, id, name, Target::CardAt(index))
            }
            Shot::Focused => {
                let field = self.nl_focus_key(id);
                self.nl_capture(ctx, id, name, Target::Field(field))
            }
            Shot::Failure => {
                let row = self
                    .metrics
                    .nat_result_rows
                    .iter()
                    .find(|r| !r.1)
                    .expect("actual false painted row")
                    .0;
                self.nl_capture(ctx, id, name, Target::ResultRow(row))
            }
            Shot::Rows => {
                assert!(!self.metrics.nat_result_rows.is_empty());
                let first = self.metrics.nat_result_rows.first().unwrap().2;
                let last_rect = self.metrics.nat_result_rows.last().unwrap().2;
                let span = first.union(last_rect);
                let root = self
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                let mode = key(self.smoke_stage, &format!("{name}-fits"));
                let fits = ctx.data_mut(|d| {
                    if let Some(v) = d.get_temp::<bool>(mode) {
                        v
                    } else {
                        let v = span.height() <= root.height() && span.width() <= root.width();
                        d.insert_temp(mode, v);
                        v
                    }
                });
                if fits {
                    return self.nl_capture(
                        ctx,
                        id,
                        &format!("{name}-all-rows"),
                        Target::ResultsSpan,
                    );
                }
                let last = self.metrics.nat_result_rows.last().unwrap().0;
                if !self.nl_capture(ctx, id, &format!("{name}-first"), Target::ResultRow(0)) {
                    return false;
                }
                self.nl_capture(ctx, id, &format!("{name}-last"), Target::ResultRow(last))
            }
        }
    }
    fn nl_focus_key(&self, id: &str) -> String {
        if self.nat_event_draft(id, false).dynamic_enabled {
            "prefix".into()
        } else {
            let d = self.nat_event_draft(id, false);
            let row = d.statics.iter().position(|r| r.inside == "server").unwrap();
            self.metrics
                .nat_fields
                .iter()
                .filter(|r| r.0.starts_with("static:") && r.0.ends_with(":global"))
                .nth(row)
                .unwrap()
                .0
                .clone()
        }
    }
    fn nl_scale_text(&self, id: &str, dark: bool) -> (Op, String) {
        let d = self.nat_event_draft(id, false);
        if let Some(r) = self.nl_live(id, false).configuration().dynamic {
            let value = if dark {
                format!(" {}", r.source.length)
            } else {
                format!("0{}", r.source.length)
            };
            (text("prefix", value.clone()), value)
        } else {
            let row = d.statics.iter().position(|r| r.inside == "server").unwrap();
            let global = self.nl_live(id, false).configuration().statics["server"];
            let value = if dark {
                format!(" {global}")
            } else {
                format!("{global} ")
            };
            (Op::RowText(row, true, value.clone()), value)
        }
    }
    fn nl_scale_proof(&self, id: &str, dark: bool) {
        let (_, expected) = self.nl_scale_text(id, dark);
        let d = self.nat_event_draft(id, false);
        if d.dynamic_enabled {
            assert_eq!(d.prefix, expected);
            assert_eq!(
                d.configuration().unwrap().dynamic,
                self.nl_live(id, false).configuration().dynamic
            );
        } else {
            let row = d.statics.iter().find(|r| r.inside == "server").unwrap();
            assert_eq!(row.global, expected);
            assert_eq!(
                d.configuration().unwrap().statics,
                self.nl_live(id, false).configuration().statics
            );
        }
    }
    fn nl_check(&mut self, ctx: &egui::Context, id: &str, practice: bool, name: &str) -> bool {
        let k = key(self.smoke_stage, &format!("{name}-before"));
        if ctx.data(|d| d.get_temp::<Value>(k)).is_none() {
            let value = self.nl_value(id, practice);
            ctx.data_mut(|d| d.insert_temp(k, value));
        }
        if !self.nl_group(
            ctx,
            id,
            practice,
            name,
            vec![click(Target::Control("check"))],
        ) {
            return false;
        }
        let before = ctx.data(|d| d.get_temp::<Value>(k)).unwrap();
        exact(
            self.nl_live(id, practice),
            &before,
            "Actual Check does not allocate or mutate any current State field",
        );
        true
    }
    fn nl_fixed_context(&self, lesson: usize) {
        self.nat_private();
        assert!(!self.synthetic);
        assert_eq!((self.scenarios.len(), self.units.len()), (143, 36));
        assert_eq!(
            self.scenarios[140..]
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            IDS
        );
        let unit = self.units.iter().find(|u| u.id == "basic-nat").unwrap();
        assert_eq!(unit.labs, IDS.map(String::from));
        assert!(self.progress.attempts.len() <= 3);
        assert!(
            self.progress
                .attempts
                .keys()
                .all(|k| IDS[..=lesson].contains(&k.as_str()))
        );
        for (id, a) in &self.progress.attempts {
            let i = self.nl_index(id);
            let engine::State::BasicNat {
                simulation: initial,
            } = &self.scenarios[i].initial
            else {
                panic!()
            };
            let engine::State::BasicNat { simulation } = &a.state else {
                panic!()
            };
            assert_eq!(simulation.inventory(), initial.inventory());
            assert!(a.transcript.is_empty());
            let w = a.workspace.as_ref().unwrap();
            assert!(w.script_history.is_none() && w.script_history_more.is_empty());
            assert_eq!(
                w.nat_learning.as_ref().unwrap().fixture,
                self.scenarios[i].nat_teaching.as_ref().unwrap().fixture
            );
        }
    }
    fn nl_pending_ops(&self, id: &str, practice: bool) -> Vec<Op> {
        let d = self.nat_event_draft(id, practice);
        if d.dynamic_enabled {
            vec![text(
                "prefix",
                if practice {
                    "example raw"
                } else {
                    "unfinished"
                },
            )]
        } else {
            let row = d
                .statics
                .iter()
                .position(|r| r.inside == if practice { "archive" } else { "server" })
                .unwrap();
            vec![Op::RowText(
                row,
                true,
                if practice {
                    "practice raw"
                } else {
                    "not-an-ip"
                }
                .into(),
            )]
        }
    }
    pub(in crate::ui) fn nat_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert!((FIRST..=LAST).contains(&stage));
        let lesson = (stage - FIRST) / STRIDE;
        let phase = (stage - FIRST) % STRIDE;
        let id = IDS[lesson];
        self.nl_fixed_context(lesson);
        let i = self.nl_index(id);
        let example = self.scenarios[i].nat_teaching.as_ref().unwrap().clone();
        let done = match phase {
            0 => {
                assert!(self.selected.is_none());
                assert_eq!(self.progress.attempts.len(), lesson);
                for previous in &IDS[..lesson] {
                    assert!(progress::current_solved(
                        &self.progress.attempts[*previous],
                        &self.scenarios[self.nl_index(previous)]
                    ));
                }
                self.filter = catalog::Filter {
                    search: self.scenarios[i].title.clone(),
                    ..Default::default()
                };
                self.page = 0;
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
                true
            }
            1 => self.nl_group(
                ctx,
                id,
                false,
                "favorite-start",
                vec![click(Target::Card(true)), click(Target::Card(false))],
            ),
            2 => {
                assert_eq!(self.selected, Some(i));
                assert_eq!(self.nl_learning(id).view, teaching::View::Introduction);
                assert_eq!(self.progress.attempts[id].state, self.scenarios[i].initial);
                assert!(self.progress.favorites.contains(id));
                let initial = serde_json::to_value(&self.progress.attempts[id]).unwrap();
                self.nl_put(ctx, lesson, "initial", initial);
                self.nl_shot(ctx, id, "intro", Shot::Header)
            }
            3 => {
                let k = key(stage, "isolation");
                if ctx.data(|d| d.get_temp::<bool>(k)).is_none() {
                    self.nl_put(
                        ctx,
                        lesson,
                        "before-practice",
                        serde_json::to_value(&self.progress).unwrap(),
                    );
                    ctx.data_mut(|d| d.insert_temp(k, true));
                }
                // One bounded queue yields after the tab click, so the reset
                // target comes from the actual next rendered Example view.
                let done = self.nl_group(
                    ctx,
                    id,
                    true,
                    "example-tab-reset",
                    vec![click(Target::LessonTab(1)), click(Target::ExampleReset)],
                );
                if done {
                    assert_eq!(self.nl_learning(id).view, teaching::View::Example);
                    assert!(self.metrics.example_reset.is_some());
                }
                done
            }
            4 => self.nl_check(ctx, id, true, "initial-example-check"),
            5 => {
                self.nl_pass(id, true, false);
                self.nl_shot(ctx, id, "example-initial-failure", Shot::Failure)
            }
            6 => {
                let k = key(stage, "insert-before");
                if ctx.data(|d| d.get_temp::<bool>(k)).is_none() {
                    self.nl_put(ctx, lesson, "insert-before", self.nl_value(id, true));
                    ctx.data_mut(|d| d.insert_temp(k, true));
                }
                if !self.nl_group(
                    ctx,
                    id,
                    true,
                    "first-insert",
                    vec![Op::Example(0), click(Target::Insert)],
                ) {
                    false
                } else {
                    let before: Value = self.nl_get(ctx, lesson, "insert-before");
                    exact(
                        self.nl_live(id, true),
                        &before,
                        "First selected Insert is not execution",
                    );
                    true
                }
            }
            7 => self.nl_actions(
                ctx,
                id,
                true,
                "worked-first",
                &[example.example_actions[0].action.clone()],
                (None, false),
            ),
            8 => {
                let rest = example.example_actions[1..]
                    .iter()
                    .map(|s| s.action.clone())
                    .collect::<Vec<_>>();
                self.nl_actions(ctx, id, true, "worked-rest", &rest, (Some(1), false))
            }
            9 => self.nl_check(ctx, id, true, "worked-example-check"),
            10 => {
                self.nl_pass(id, true, true);
                self.nl_shot(ctx, id, "worked-example", Shot::Rows)
            }
            11 => self.nl_actions(
                ctx,
                id,
                true,
                "experiment",
                &example.try_change.actions,
                (None, false),
            ),
            12 => self.nl_check(ctx, id, true, "adverse-example-check"),
            13 => {
                assert_eq!(self.nl_rows(id, true), example.try_change.expected);
                self.nl_shot(ctx, id, "example-adverse", Shot::Rows)
            }
            14 => self.nl_actions(
                ctx,
                id,
                true,
                "repair-example",
                &example.repair,
                (None, false),
            ),
            15 => self.nl_check(ctx, id, true, "repair-example-check"),
            16 => {
                self.nl_pass(id, true, true);
                let mut before: Value = self.nl_get(ctx, lesson, "before-practice");
                before["attempts"][id]["workspace"]["nat_learning"] =
                    serde_json::to_value(self.nl_learning(id)).unwrap();
                exact(
                    &self.progress,
                    &before,
                    "Practice changes only own Learning; all assignment/other/favorite/clock fields exact",
                );
                self.nl_shot(ctx, id, "example-repaired", Shot::Rows)
            }
            17 => self.nl_group(
                ctx,
                id,
                false,
                "assignment-tab",
                vec![click(Target::LessonTab(2))],
            ),
            18 => self.nl_shot(ctx, id, "full-assignment", Shot::Header),
            19 => {
                let actions = self.nl_plan(id, "wrong");
                self.nl_actions(ctx, id, false, "wrong-plan", &actions, (None, lesson == 2))
            }
            20 => self.nl_check(ctx, id, false, "wrong-check"),
            21 => {
                let r = self.nl_review(id);
                let expected: Vec<bool> =
                    serde_json::from_value(r["wrong"][0]["expected"].clone()).unwrap();
                assert_eq!(self.nl_rows(id, false), expected);
                self.nl_pass(id, false, false);
                self.nl_shot(ctx, id, "wrong-feedback", Shot::Failure)
            }
            22 => {
                let actions = self.nl_plan(id, "partial");
                self.nl_actions(ctx, id, false, "partial-plan", &actions, (None, false))
            }
            23 => self.nl_check(ctx, id, false, "partial-check"),
            24 => {
                let r = self.nl_review(id);
                let expected: Vec<bool> =
                    serde_json::from_value(r["partial"]["expected"].clone()).unwrap();
                assert_eq!(self.nl_rows(id, false), expected);
                self.nl_pass(id, false, false);
                self.nl_shot(ctx, id, "partial-feedback", Shot::Rows)
            }
            25 => {
                let actions = self.nl_reference(id);
                self.nl_actions(ctx, id, false, "reference", &actions, (None, false))
            }
            26 => self.nl_check(ctx, id, false, "reference-check"),
            27 => {
                self.nl_pass(id, false, true);
                self.nl_shot(ctx, id, "reference-complete", Shot::Rows)
            }
            28 => {
                if lesson != 1 {
                    true
                } else {
                    let inside = self
                        .nl_live(id, false)
                        .inventory()
                        .inside
                        .keys()
                        .find(|k| k.as_str() == "alpha")
                        .unwrap()
                        .clone();
                    let peer = self
                        .nl_live(id, false)
                        .inventory()
                        .outside
                        .keys()
                        .next()
                        .unwrap()
                        .clone();
                    let k = key(stage, "before-optional-probe");
                    if ctx.data(|d| d.get_temp::<bool>(k)).is_none() {
                        assert!(self.nl_live(id, false).bindings().is_empty());
                        ctx.data_mut(|d| d.insert_temp(k, true));
                    }
                    self.nl_actions(
                        ctx,
                        id,
                        false,
                        "optional-current-observation",
                        &[core::Action::Outbound { inside, peer }],
                        (None, false),
                    )
                }
            }
            29 => {
                let live = self.nl_live(id, false);
                let inside = if lesson == 0 { "server" } else { "alpha" };
                let global = *live
                    .configuration()
                    .statics
                    .get(inside)
                    .or_else(|| live.bindings().get(inside))
                    .unwrap();
                let peer = live.inventory().outside.keys().next().unwrap().clone();
                self.nl_actions(
                    ctx,
                    id,
                    false,
                    "reverse-actual-map",
                    &[core::Action::Inbound { peer, global }],
                    (None, false),
                )
            }
            30 => self.nl_shot(ctx, id, "actual-reverse-observation", Shot::Observation),
            31 => {
                let actions = self.nl_plan(id, "broken");
                self.nl_actions(ctx, id, false, "break-again", &actions, (None, false))
            }
            32 => self.nl_check(ctx, id, false, "broken-check"),
            33 => {
                self.nl_pass(id, false, false);
                self.nl_shot(ctx, id, "success-invalidated", Shot::Failure)
            }
            34 => {
                let actions = self.nl_plan(id, "alt0");
                self.nl_actions(ctx, id, false, "alternate0", &actions, (None, false))
            }
            35 => self.nl_check(ctx, id, false, "alternate0-check"),
            36 => {
                self.nl_pass(id, false, true);
                self.nl_shot(ctx, id, "alternate-policy-one", Shot::Rows)
            }
            37 => {
                let mut actions = self.nl_plan(id, "alt1");
                if lesson == 1 {
                    actions.insert(0, core::Action::ClearDynamic {});
                }
                self.nl_actions(ctx, id, false, "alternate1", &actions, (None, false))
            }
            38 => self.nl_check(ctx, id, false, "alternate1-check"),
            39 => {
                self.nl_pass(id, false, true);
                self.nl_shot(ctx, id, "alternate-policy-two", Shot::Rows)
            }
            40 => {
                let ops = self.nl_pending_ops(id, false);
                if !self.nl_group(ctx, id, false, "assignment-pending", ops) {
                    false
                } else {
                    self.nl_check(ctx, id, false, "assignment-pending-check")
                }
            }
            41 => {
                self.nl_pass(id, false, false);
                self.nl_shot(ctx, id, "assignment-pending-before-save", Shot::Failure)
            }
            42 => {
                if !self.nl_group(
                    ctx,
                    id,
                    true,
                    "practice-tab",
                    vec![click(Target::LessonTab(1))],
                ) {
                    false
                } else {
                    let mut ops = self.nl_pending_ops(id, true);
                    ops.push(text("inbound-global", format!("practice query {lesson}")));
                    if !self.nl_group(ctx, id, true, "practice-pending", ops) {
                        false
                    } else {
                        self.nl_check(ctx, id, true, "practice-pending-check")
                    }
                }
            }
            43 => {
                self.nl_pass(id, true, false);
                self.nl_shot(ctx, id, "separate-practice-pending", Shot::Failure)
            }
            44 => {
                if !self.nl_group(
                    ctx,
                    id,
                    false,
                    "return-assignment",
                    vec![click(Target::LessonTab(2))],
                ) {
                    false
                } else {
                    self.nl_group(
                        ctx,
                        id,
                        false,
                        "assignment-readonly",
                        vec![text("inbound-global", format!("assignment query {lesson}"))],
                    )
                }
            }
            45 => {
                let warm = self.nat_warm();
                self.nl_put(ctx, lesson, "warm", warm);
                true
            }
            46 => self.nl_group(ctx, id, false, "back-pending", vec![click(Target::Back)]),
            47 => self.nl_shot(ctx, id, "library-pending-saved", Shot::Library(0)),
            48 => self.nl_group(ctx, id, false, "continue", vec![click(Target::Card(false))]),
            49 => {
                let warm: Warm = self.nl_get(ctx, lesson, "warm");
                let mut expected = warm.portable.clone();
                let clock = expected["clock"].as_u64().unwrap() + 1;
                expected["clock"] = clock.into();
                expected["attempts"][id]["visited"] = clock.into();
                exact(
                    &self.progress,
                    &expected,
                    "Real Continue changes only clock+visited; both drafts/current maps and all others exact",
                );
                exact(
                    &self.progress.migration_receipts,
                    &warm.caller_receipts,
                    "Continue keeps caller receipts",
                );
                let mut raw: Value = serde_json::from_slice(&warm.raw).unwrap();
                let receipts = raw
                    .as_object_mut()
                    .unwrap()
                    .remove("migration_receipts")
                    .unwrap();
                exact(
                    &raw,
                    &warm.portable,
                    "Original raw/portable full projection anchor",
                );
                exact(&receipts, &warm.raw_receipts, "Original raw receipt anchor");
                let current = self.nat_warm();
                exact(
                    &current.portable,
                    &expected,
                    "Continued whole normal-load readback",
                );
                assert_eq!(
                    self.nat_event_draft(id, false).inbound_global,
                    format!("assignment query {lesson}")
                );
                assert_eq!(
                    self.nat_event_draft(id, true).inbound_global,
                    format!("practice query {lesson}")
                );
                self.nl_shot(ctx, id, "warm-resumed-pending", Shot::Failure)
            }
            50 => {
                let mut actions = self.nl_reference(id);
                if lesson == 1 {
                    actions.insert(0, core::Action::ClearDynamic {});
                }
                self.nl_actions(ctx, id, false, "repair-assignment", &actions, (None, false))
            }
            51 => {
                if !self.nl_check(ctx, id, false, "repaired-check") {
                    false
                } else {
                    self.nl_pass(id, false, true);
                    assert!(!self.nl_rows(id, true).iter().all(|v| *v));
                    true
                }
            }
            52 => self.nl_group(ctx, id, false, "challenge", vec![click(Target::Challenge)]),
            53 => {
                assert!(self.progress.attempts[id].challenge);
                assert!(
                    self.metrics.lesson_tabs.is_empty()
                        && self.metrics.insert.is_none()
                        && !self.metrics.help_visible
                );
                self.nl_shot(ctx, id, "challenge-full-assignment", Shot::Header)
            }
            54 => self.nl_group(ctx, id, false, "learn", vec![click(Target::Learn)]),
            55 | 60 | 65 => {
                let (scale, light, width) = if phase == 55 {
                    (1.5, true, 1400.)
                } else if phase == 60 {
                    (2., false, 946.)
                } else {
                    (1., false, 1200.)
                };
                self.scale = scale;
                self.theme_id = if light { "dlms-light" } else { "dlms-dark" }.into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(width, 929.)));
                true
            }
            56 | 61 => {
                let (op, _) = self.nl_scale_text(id, phase == 61);
                self.nl_group(ctx, id, false, "scaled-text", vec![op])
            }
            57 | 62 => {
                self.nl_scale_proof(id, phase == 62);
                self.nl_shot(
                    ctx,
                    id,
                    if phase == 57 {
                        "light150-focused-input"
                    } else {
                        "dark200-focused-input"
                    },
                    Shot::Focused,
                )
            }
            58 | 63 => {
                if !self.nl_group(
                    ctx,
                    id,
                    false,
                    "scaled-apply",
                    vec![click(Target::Control("apply"))],
                ) {
                    false
                } else {
                    self.nl_check(ctx, id, false, "scaled-check")
                }
            }
            59 | 64 => {
                self.nl_scale_proof(id, phase == 64);
                self.nl_pass(id, false, true);
                self.nl_shot(
                    ctx,
                    id,
                    if phase == 59 {
                        "light150-applied-pass"
                    } else {
                        "dark200-applied-pass"
                    },
                    Shot::Rows,
                )
            }
            66 => {
                let k = key(stage, "before-cancel");
                if ctx.data(|d| d.get_temp::<CancelProof>(k)).is_none() {
                    let _ = self.nat_warm();
                    let proof = CancelProof {
                        progress: serde_json::to_value(&self.progress).unwrap(),
                        receipts: serde_json::to_value(&self.progress.migration_receipts).unwrap(),
                        files: self.nat_files(),
                    };
                    ctx.data_mut(|d| d.insert_temp(k, proof));
                }
                if !self.nl_group(
                    ctx,
                    id,
                    false,
                    "reset-cancel",
                    vec![click(Target::Reset), click(Target::CancelReset)],
                ) {
                    false
                } else {
                    let before = ctx.data(|d| d.get_temp::<CancelProof>(k)).unwrap();
                    exact(
                        &self.progress,
                        &before.progress,
                        "Reset Cancel keeps whole Progress",
                    );
                    exact(
                        &self.progress.migration_receipts,
                        &before.receipts,
                        "Reset Cancel keeps all receipts",
                    );
                    assert!(
                        self.nat_files() == before.files,
                        "Reset Cancel changed private file bytes (contents withheld)"
                    );
                    self.nl_put(ctx, lesson, "before-reset", before.progress);
                    true
                }
            }
            67 => self.nl_group(
                ctx,
                id,
                false,
                "reset-confirm",
                vec![click(Target::Reset), click(Target::ConfirmReset)],
            ),
            68 => {
                let mut expected: Value = self.nl_get(ctx, lesson, "before-reset");
                let visited = expected["attempts"][id]["visited"].clone();
                let mut initial: Value = self.nl_get(ctx, lesson, "initial");
                initial["visited"] = visited;
                expected["attempts"][id] = initial;
                exact(
                    &self.progress,
                    &expected,
                    "Selected Confirm reset equals actual original Start; all others/clock/favorites exact",
                );
                assert_eq!(self.nl_learning(id).view, teaching::View::Introduction);
                self.nl_shot(ctx, id, "selected-reset-intro", Shot::Header)
            }
            69 => {
                if !self.nl_group(
                    ctx,
                    id,
                    false,
                    "assignment-after-reset",
                    vec![click(Target::LessonTab(2))],
                ) {
                    false
                } else {
                    let actions = self.nl_reference(id);
                    if !self.nl_actions(
                        ctx,
                        id,
                        false,
                        "reference-after-reset",
                        &actions,
                        (None, false),
                    ) {
                        false
                    } else {
                        self.nl_check(ctx, id, false, "final-check")
                    }
                }
            }
            70 => {
                let shown = key(stage, "repair-feedback-shown");
                if ctx.data(|d| d.get_temp::<bool>(shown)) != Some(true) {
                    self.nl_pass(id, false, true);
                    if !self.nl_shot(ctx, id, "after-reset-repaired", Shot::Rows) {
                        return (false, None);
                    }
                    ctx.data_mut(|d| d.insert_temp(shown, true));
                }
                if !self.nl_group(ctx, id, false, "final-back", vec![click(Target::Back)]) {
                    false
                } else {
                    let k = key(stage, "unit-filter");
                    if ctx.data(|d| d.get_temp::<bool>(k)).is_none() {
                        let _ = self.nat_warm();
                        self.filter = catalog::Filter {
                            unit: "basic-nat".into(),
                            ..Default::default()
                        };
                        self.page = 0;
                        ctx.data_mut(|d| d.insert_temp(k, true));
                        ctx.request_repaint();
                        return (false, None);
                    }
                    assert_eq!(self.metrics.card_actions.len(), 3);
                    if !self.nl_shot(ctx, id, "unit-library-first", Shot::Library(0)) {
                        false
                    } else {
                        self.nl_shot(ctx, id, "unit-library-last", Shot::Library(2))
                    }
                }
            }
            71 => {
                let k = key(stage, "final-proof-done");
                if ctx.data(|d| d.get_temp::<bool>(k)) == Some(true) {
                    assert_eq!(stage, LAST);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return (false, None);
                }
                assert!(self.selected.is_none());
                assert_eq!(self.progress.attempts.len(), lesson + 1);
                let _ = self.nat_warm();
                for old in &IDS[..=lesson] {
                    assert!(progress::current_solved(
                        &self.progress.attempts[*old],
                        &self.scenarios[self.nl_index(old)]
                    ));
                    assert!(self.progress.favorites.contains(*old));
                }
                exact(
                    &self.scenarios,
                    &pack::bundled().unwrap(),
                    "All143 authored specs remain exact, no injected fixture",
                );
                ctx.data_mut(|d| {
                    d.insert_temp(k, true);
                    for name in [
                        "initial",
                        "before-practice",
                        "insert-before",
                        "before-reset",
                    ] {
                        d.remove::<Value>(key(lesson, name));
                    }
                    d.remove::<Warm>(key(lesson, "warm"));
                    d.remove::<CancelProof>(key(FIRST + lesson * STRIDE + 66, "before-cancel"));
                });
                if lesson == 2 {
                    println!(
                        "PASS normal three Basic NAT lessons: actual Start/Intro/separate example/Insert/actions/adverse/repair/full assignment/wrong+partial/ref/two alternatives/invalidation/current+new requests/warm whole-value readback/reset/Challenge/observed100wide150Light200Dark. No PAT, routing, host, novice, other-OS or cold-process claim."
                    );
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return (false, None);
                }
                true
            }
            _ => unreachable!("closed72phase lesson"),
        };
        (done, None)
    }
}
