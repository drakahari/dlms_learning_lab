//! Fixed private technical acceptance. Effects only through actual native widgets.
use super::*;
use dlms_learning_lab::{nat as core, nat_workspace as work};
use serde_json::Value;
pub(in crate::ui) const FIRST: usize = 38000;
pub(in crate::ui) const LAST: usize = FIRST + PLAN.len() - 1;
pub(in crate::ui) const ID: &str = "nat-workspace-preview";
const MAX_PROFILE: usize = 512 * 1024;
#[derive(Clone, Copy, Debug)]
enum Step {
    Admit,
    Start,
    Initial,
    Shot(&'static str),
    Check,
    InitialFailure,
    StaticReverse,
    Malformed,
    AtomicFailure,
    Policy,
    Pending,
    Apply,
    AppliedEmpty,
    Alpha,
    Partial,
    Beta,
    Complete,
    ReverseActual,
    PeerReuse,
    ReuseProof,
    ReadonlyError,
    ReadonlyProof,
    Clear,
    Cleared,
    RepairCurrent,
    AlternateText,
    AlternateApply,
    AlternateMaps,
    AlternateProof,
    SettingsBlock,
    SettingsClose,
    SamplesBlock,
    SamplesClose,
    DataCancel,
    ResetCancel,
    PendingText,
    WarmSave,
    Back,
    Continue,
    ResumeProof,
    RepairText,
    Challenge,
    Learn,
    Light,
    LightText,
    LightApply,
    Dark,
    DarkText,
    DarkApply,
    Normal,
    ConfirmReset,
    ResetProof,
    RepairAfterReset,
    FinalSave,
    Finish,
}
const PLAN: [Step; 72] = [
    Step::Admit,
    Step::Start,
    Step::Initial,
    Step::Shot("initial-task"),
    Step::Check,
    Step::InitialFailure,
    Step::StaticReverse,
    Step::Shot("static-reverse"),
    Step::Malformed,
    Step::AtomicFailure,
    Step::Shot("malformed-policy"),
    Step::Policy,
    Step::Pending,
    Step::Shot("pending-feedback"),
    Step::Apply,
    Step::AppliedEmpty,
    Step::Alpha,
    Step::Partial,
    Step::Shot("partial-beta-missing"),
    Step::Beta,
    Step::Complete,
    Step::Shot("current-and-fresh-pass"),
    Step::ReverseActual,
    Step::PeerReuse,
    Step::ReuseProof,
    Step::ReadonlyError,
    Step::ReadonlyProof,
    Step::Clear,
    Step::Cleared,
    Step::Shot("cleared-current-fails"),
    Step::RepairCurrent,
    Step::Shot("repaired-current"),
    Step::Clear,
    Step::AlternateText,
    Step::AlternateApply,
    Step::AlternateMaps,
    Step::AlternateProof,
    Step::Shot("legitimate-alternate"),
    Step::SettingsBlock,
    Step::SettingsClose,
    Step::SamplesBlock,
    Step::SamplesClose,
    Step::DataCancel,
    Step::ResetCancel,
    Step::PendingText,
    Step::Shot("pending-before-save"),
    Step::WarmSave,
    Step::Back,
    Step::Continue,
    Step::ResumeProof,
    Step::RepairText,
    Step::Challenge,
    Step::Shot("challenge-task"),
    Step::Learn,
    Step::Light,
    Step::LightText,
    Step::Shot("light150-focused-input"),
    Step::LightApply,
    Step::Shot("light150-current-pass"),
    Step::Dark,
    Step::DarkText,
    Step::Shot("dark200-focused-input"),
    Step::DarkApply,
    Step::Shot("dark200-current-pass"),
    Step::Normal,
    Step::ConfirmReset,
    Step::ResetProof,
    Step::RepairAfterReset,
    Step::Shot("after-reset-repaired"),
    Step::Back,
    Step::FinalSave,
    Step::Finish,
];
const _: () = assert!(FIRST > 37504 && LAST == 38071 && LAST < 39000);
#[derive(Clone, Debug)]
pub(in crate::ui) enum Target {
    Control(&'static str),
    LessonTab(usize),
    ExamplePicker,
    Insert,
    ExampleReset,
    ResultRow(usize),
    ResultsSpan,
    Observation,
    CardAt(usize),
    Field(String),
    RemoveRow(usize),
    Choice(String),
    Card(bool),
    Back,
    Challenge,
    Learn,
    Reset,
    CancelReset,
    ConfirmReset,
    Settings,
    SettingsClose,
    Tab(&'static str),
    Samples,
    DataExport,
    FileName,
    FileCancel,
}
#[derive(Clone, Debug)]
pub(in crate::ui) enum Op {
    Click(Target),
    Text(&'static str, String),
    Choose(&'static str, &'static str),
    Select(&'static str, String),
    Example(usize),
    FileText(String),
    RowText(usize, bool, String),
    Undo,
    Expect(&'static str, String),
}
#[derive(Clone, Copy, Debug)]
struct ScrollGeometry {
    target: egui::Rect,
    visible: egui::Rect,
    main: egui::Vec2,
    child: Option<(egui::Id, egui::Vec2, egui::Vec2)>,
    ppp: f32,
}
impl ScrollGeometry {
    fn settled_with(self, old: Self) -> bool {
        if self.ppp != old.ppp || !self.ppp.is_finite() || self.ppp <= 0. {
            return false;
        }
        // Stability only: this never expands a clickable/visible rectangle.
        let epsilon = 0.25 / self.ppp;
        let scalar = |a: f32, b: f32| (a - b).abs() <= epsilon;
        let vector = |a: egui::Vec2, b: egui::Vec2| scalar(a.x, b.x) && scalar(a.y, b.y);
        let rect = |a: egui::Rect, b: egui::Rect| {
            vector(a.min.to_vec2(), b.min.to_vec2()) && vector(a.max.to_vec2(), b.max.to_vec2())
        };
        rect(self.target, old.target)
            && rect(self.visible, old.visible)
            && vector(self.main, old.main)
            && match (self.child, old.child) {
                (Some((id, a, b)), Some((other, c, d))) => {
                    id == other && vector(a, c) && vector(b, d)
                }
                (None, None) => true,
                _ => false,
            }
    }
}
#[derive(Clone, Default)]
pub(in crate::ui) struct Queue {
    next: usize,
    sub: u8,
    ticks: usize,
    scrolls: usize,
    stable: Option<(egui::Rect, egui::Rect, f32)>,
    stable_frames: usize,
    list_before: Option<usize>,
    trace: std::collections::VecDeque<String>,
    waiting_wheel: bool,
    wheel_geometry: Option<ScrollGeometry>,
    idle_frames: usize,
    settled_frames: usize,
}
#[derive(Clone)]
pub(in crate::ui) struct Warm {
    pub(in crate::ui) raw: Vec<u8>,
    pub(in crate::ui) portable: Value,
    pub(in crate::ui) raw_receipts: Value,
    pub(in crate::ui) caller_receipts: Value,
}
fn ctrl_key(key: egui::Key, pressed: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    }
}
fn difference(a: &Value, b: &Value) -> Option<String> {
    if a == b {
        return None;
    }
    match (a, b) {
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() {
                return Some("object key count".into());
            }
            for (k, v) in a {
                let Some(w) = b.get(k) else {
                    return Some(format!("missing {:.60}", k));
                };
                if let Some(p) = difference(v, w) {
                    return Some(format!("{:.60}.{:.140}", k, p));
                }
            }
            Some("object".into())
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return Some("array length".into());
            }
            a.iter()
                .zip(b)
                .enumerate()
                .find_map(|(i, (v, w))| difference(v, w).map(|p| format!("[{i}].{p:.140}")))
        }
        _ => Some("scalar/type (values withheld)".into()),
    }
}
pub(in crate::ui) fn exact<A: serde::Serialize + ?Sized, B: serde::Serialize + ?Sized>(
    a: &A,
    b: &B,
    label: &str,
) {
    let a = serde_json::to_value(a).unwrap();
    let b = serde_json::to_value(b).unwrap();
    let d = difference(&a, &b);
    assert!(d.is_none(), "{label}: {}", d.unwrap_or_default());
}
fn click(t: Target) -> Op {
    Op::Click(t)
}
fn text(k: &'static str, v: &str) -> Op {
    Op::Text(k, v.into())
}
fn choice(k: &'static str, v: &'static str) -> Op {
    Op::Choose(k, v)
}
fn mappings(alpha_first: bool) -> Vec<Op> {
    let (first, second) = if alpha_first {
        ("alpha", "beta")
    } else {
        ("beta", "alpha")
    };
    vec![
        choice("peer", "peer"),
        choice("inside", first),
        click(Target::Control("outbound")),
        choice("inside", second),
        click(Target::Control("outbound")),
        click(Target::Control("check")),
    ]
}
// The controller uses this exact geometry decision; it is not a grade or an
// expanded viewport. Tests can replay measured edges without fabricating widgets.
pub(in crate::ui) fn static_root_reveal_piece(
    target: egui::Rect,
    visible: egui::Rect,
    child: egui::Rect,
    root: egui::Rect,
    ppp: f32,
) -> Option<egui::Rect> {
    let already_visible = smoke_control_visible(target, visible, ppp);
    let room_for_target = visible.height() >= target.height() && visible.width() >= target.width();
    if already_visible || room_for_target {
        return None;
    }
    // Reveal enough of a mostly/wholly offscreen child before routing inside.
    // A taller child need never be wholly contained to expose a complete field.
    let height = child.height().min(root.height());
    let width = child.width().min(root.width());
    let top = if child.top() < root.top() {
        child.bottom() - height
    } else {
        child.top()
    };
    Some(egui::Rect::from_min_size(
        egui::pos2(child.left(), top),
        egui::vec2(width, height),
    ))
}
impl LabApp {
    pub(in crate::ui) fn nat_stage(smoke: bool, name: Option<&str>) -> Option<usize> {
        (smoke && name == Some("nat-workspace")).then_some(FIRST)
    }
    pub(in crate::ui) fn nat_private(&self) {
        assert!(
            self.smoke && self._smoke_profile.is_some(),
            "closed smoke-private fixture only"
        );
        assert_eq!(
            self.path.parent(),
            Some(self._smoke_profile.as_ref().unwrap().path())
        );
    }
    fn nat_index(&self) -> usize {
        self.scenarios
            .iter()
            .position(|s| s.id == ID)
            .expect("closed technical fixture")
    }
    fn nat_draft(&self) -> &work::Draft {
        self.progress.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .nat_draft
            .as_ref()
            .unwrap()
    }
    fn nat_live(&self) -> &core::State {
        let engine::State::BasicNat { simulation } = &self.progress.attempts[ID].state else {
            panic!("NAT actual state")
        };
        simulation
    }
    fn nat_raw(&self, key: &str) -> &str {
        let d = self.nat_draft();
        match key {
            "network" => &d.network,
            "prefix" => &d.prefix,
            "first" => &d.first,
            "last" => &d.last,
            "inbound-global" => &d.inbound_global,
            _ => panic!("closed field"),
        }
    }
    fn nat_remember(&self, ctx: &egui::Context, name: &'static str, value: Value) {
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(("nat-observed", name)), value));
    }
    fn nat_observed(&self, ctx: &egui::Context, name: &'static str) -> Value {
        ctx.data(|d| d.get_temp::<Value>(egui::Id::new(("nat-observed", name))))
            .expect("earlier actual observation")
    }
    pub(in crate::ui) fn nat_geometry(&self, ctx: &egui::Context) -> bool {
        let rect = ctx.content_rect();
        let zoom = ctx.zoom_factor();
        let ppp = ctx.pixels_per_point();
        assert!(ppp.is_finite() && ppp > 0.);
        let key = egui::Id::new(("nat-geometry", self.smoke_stage));
        let ready = ctx.data_mut(|d| {
            let (old, n, start) = d
                .get_temp::<(Option<(egui::Rect, f32, f32)>, usize, Instant)>(key)
                .unwrap_or((None, 0, Instant::now()));
            assert!(
                start.elapsed().as_secs() < 15,
                "NAT geometry did not settle"
            );
            let n = if old == Some((rect, zoom, ppp)) && (zoom - self.scale).abs() < 0.01 {
                n + 1
            } else {
                0
            };
            d.insert_temp(key, (Some((rect, zoom, ppp)), n, start));
            n >= 2
        });
        if !ready {
            ctx.request_repaint();
        }
        ready
    }
    pub(in crate::ui) fn nat_target(
        &self,
        ctx: &egui::Context,
        t: &Target,
    ) -> (egui::Rect, egui::Rect, bool) {
        let root = self
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        match t {
            Target::ResultsSpan => {
                let first = self.metrics.nat_result_rows.first().unwrap();
                let last = self.metrics.nat_result_rows.last().unwrap();
                (
                    first.2.union(last.2),
                    first.3.intersect(last.3).intersect(root),
                    false,
                )
            }
            Target::Observation => (
                self.metrics.nat_observation.expect("actual observation"),
                root,
                false,
            ),
            Target::CardAt(n) => (self.metrics.card_actions[*n].0, root, false),
            Target::LessonTab(n) => (self.metrics.lesson_tabs[*n], root, false),
            Target::ExamplePicker => {
                let (r, c) = self
                    .metrics
                    .nat_example_picker
                    .expect("actual example picker");
                (r, c.intersect(root), false)
            }
            Target::Insert => (self.metrics.insert.expect("actual Insert"), root, false),
            Target::ExampleReset => (
                self.metrics.example_reset.expect("actual example reset"),
                root,
                false,
            ),
            Target::ResultRow(n) => {
                let r = self
                    .metrics
                    .nat_result_rows
                    .iter()
                    .find(|r| r.0 == *n)
                    .expect("actual result row");
                (r.2, r.3.intersect(root), false)
            }
            Target::Field(k) => {
                let r = self
                    .metrics
                    .nat_fields
                    .iter()
                    .find(|r| &r.0 == k)
                    .expect("actual field");
                let child = if k.starts_with("static:") {
                    self.metrics.nat_static_viewport.unwrap()
                } else {
                    root
                };
                (
                    r.1,
                    r.4.intersect(child).intersect(root),
                    k.starts_with("static:"),
                )
            }
            Target::Control(k) => {
                let r = self
                    .metrics
                    .nat_controls
                    .iter()
                    .find(|r| r.0 == *k)
                    .expect("actual control");
                let child = if k.starts_with("remove:") {
                    self.metrics.nat_static_viewport.unwrap()
                } else {
                    root
                };
                (
                    r.1,
                    r.3.intersect(child).intersect(root),
                    k.starts_with("remove:"),
                )
            }
            Target::RemoveRow(row) => {
                let r = self
                    .metrics
                    .nat_controls
                    .iter()
                    .filter(|r| r.0.starts_with("remove:"))
                    .nth(*row)
                    .expect("actual row Remove");
                (
                    r.1,
                    r.3.intersect(self.metrics.nat_static_viewport.unwrap())
                        .intersect(root),
                    true,
                )
            }
            Target::Choice(k) => {
                let r = self
                    .metrics
                    .nat_choices
                    .iter()
                    .find(|r| &r.0 == k)
                    .expect("actual option");
                (
                    r.1,
                    (if k.starts_with("example:") {
                        self.metrics.nat_example_popup
                    } else {
                        self.metrics.nat_popup
                    })
                    .unwrap()
                    .intersect(ctx.content_rect()),
                    true,
                )
            }
            Target::Card(favorite) => {
                assert_eq!(
                    self.metrics.card_actions.len(),
                    1,
                    "one actual technical card"
                );
                let r = self.metrics.card_actions[0];
                (if *favorite { r.1 } else { r.0 }, root, false)
            }
            Target::Settings => (self.metrics.settings.unwrap(), ctx.content_rect(), false),
            Target::SettingsClose => {
                let (r, _, enabled, clip) =
                    self.metrics.settings_close.expect("actual Settings close");
                assert!(
                    enabled,
                    "Settings Close must stay blocked until the file/restore modal is closed"
                );
                (r, clip.intersect(ctx.content_rect()), true)
            }
            Target::Tab(name) => {
                let r = self
                    .metrics
                    .settings_tabs
                    .iter()
                    .find(|r| r.0 == *name)
                    .unwrap();
                (r.1, r.2.intersect(ctx.content_rect()), true)
            }
            Target::Samples => {
                let (r, c) = self.metrics.settings_samples.unwrap();
                (r, c.intersect(ctx.content_rect()), true)
            }
            Target::DataExport => (
                self.metrics.data_export.unwrap(),
                self.metrics
                    .settings_tabs
                    .iter()
                    .find(|r| r.0 == "data")
                    .unwrap()
                    .2
                    .intersect(ctx.content_rect()),
                true,
            ),
            Target::FileName => (
                self.metrics.data_filename.unwrap(),
                self.metrics
                    .data_file_viewport
                    .unwrap()
                    .intersect(ctx.content_rect()),
                true,
            ),
            Target::FileCancel => (
                self.metrics.data_file_cancel.unwrap(),
                self.metrics
                    .data_file_viewport
                    .unwrap()
                    .intersect(ctx.content_rect()),
                true,
            ),
            _ => {
                let r = match t {
                    Target::Back => self.metrics.back,
                    Target::Challenge => self.metrics.mode_challenge,
                    Target::Learn => self.metrics.mode_learn,
                    Target::Reset => self.metrics.reset,
                    Target::CancelReset => self.metrics.reset_cancel,
                    Target::ConfirmReset => self.metrics.confirm_reset,
                    _ => unreachable!(),
                }
                .expect("actual header control");
                (r, root, false)
            }
        }
    }
    pub(in crate::ui) fn nat_reveal(
        &mut self,
        ctx: &egui::Context,
        target: egui::Rect,
        visible: egui::Rect,
        inner: bool,
        q: &mut Queue,
    ) -> bool {
        let ppp = ctx.pixels_per_point();
        let geometry = ScrollGeometry {
            target,
            visible,
            main: self.metrics.main_scroll_offset,
            child: self.metrics.nat_static_scroll,
            ppp,
        };
        if q.waiting_wheel {
            q.idle_frames += 1;
            q.settled_frames = if q
                .wheel_geometry
                .is_some_and(|old| geometry.settled_with(old))
            {
                q.settled_frames + 1
            } else {
                0
            };
            q.wheel_geometry = Some(geometry);
            if q.trace.len() == 3 {
                q.trace.pop_front();
            }
            q.trace.push_back(format!(
                "wheel{} idle{} stable{} measured{geometry:?}",
                q.scrolls, q.idle_frames, q.settled_frames
            ));
            assert!(
                q.idle_frames <= 60,
                "NAT wheel did not settle within60 actual rendered frames at stage{} item{}: {:?}",
                self.smoke_stage,
                q.next,
                q.trace
            );
            if q.settled_frames < 3 {
                ctx.request_repaint();
                return false;
            }
            q.waiting_wheel = false;
        }
        if smoke_control_visible(target, visible, ppp) {
            let current = (target, visible, ppp);
            q.stable_frames = if q.stable == Some(current) {
                q.stable_frames + 1
            } else {
                0
            };
            q.stable = Some(current);
            if q.stable_frames >= 1 {
                return true;
            }
            ctx.request_repaint();
            return false;
        }
        q.stable = None;
        q.stable_frames = 0;
        q.scrolls += 1;
        let root = self
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        // A wholly offscreen child needs the root scroller first.
        let child_visible = inner
            && visible.height() >= target.height()
            && visible.width() > 0.
            && root.contains(visible.center());
        let area = if child_visible { visible } else { root };
        let delta = if target.bottom() > area.bottom() {
            area.bottom() - target.bottom() - 4.
        } else if target.top() < area.top() {
            area.top() - target.top() + 4.
        } else {
            0.
        };
        let pointer = if child_visible {
            area.center()
        } else {
            egui::pos2(root.left() + 1., root.center().y)
        };
        let input = ctx.input(|i| {
            (
                i.raw_scroll_delta,
                i.smooth_scroll_delta,
                i.modifiers,
                i.pointer.hover_pos(),
            )
        });
        let focus = ctx.memory(|m| m.focused());
        let focus_field = self
            .metrics
            .nat_fields
            .iter()
            .find(|r| Some(r.2) == focus)
            .map(|r| r.0.as_str());
        if q.trace.len() == 3 {
            q.trace.pop_front();
        }
        q.trace.push_back(format!("try{} target{target:?} visible{visible:?} inner{inner} route_child{child_visible} queued_pointer{pointer:?} queued_delta{:.2} actual_input{input:?} ppp{ppp:.3} main_offset{:?} child(id,offset,content){:?} focused{focus:?}/{focus_field:?}",q.scrolls,delta.clamp(-350.,350.),self.metrics.main_scroll_offset,self.metrics.nat_static_scroll));
        assert!(
            q.scrolls <= 20,
            "NAT bounded reveal stage{} item{}; last3 {:?}",
            self.smoke_stage,
            q.next,
            q.trace
        );
        q.waiting_wheel = true;
        q.wheel_geometry = Some(geometry);
        q.idle_frames = 0;
        q.settled_frames = 0;
        self.pending_events.extend([
            egui::Event::PointerMoved(pointer),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., delta.clamp(-350., 350.)),
                modifiers: Default::default(),
            },
        ]);
        self.started = Instant::now();
        ctx.request_repaint();
        false
    }
    fn nat_click(&mut self, ctx: &egui::Context, t: &Target, q: &mut Queue) -> bool {
        let phase = ctx
            .data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))))
            .unwrap_or(0);
        if phase >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let (r, v, inner) = self.nat_target(ctx, t);
        if !self.nat_reveal(ctx, r, v, inner, q) {
            return false;
        }
        self.click_smoke_control(ctx, Some(r))
    }
    fn nat_queue(&mut self, ctx: &egui::Context, ops: Vec<Op>) -> bool {
        self.nat_queue_for(ctx, ops, ID, false)
    }
    pub(in crate::ui) fn nat_event_draft(&self, id: &str, practice: bool) -> &work::Draft {
        let w = self.progress.attempts[id].workspace.as_ref().unwrap();
        if practice {
            &w.nat_learning.as_ref().unwrap().draft
        } else {
            w.nat_draft.as_ref().unwrap()
        }
    }
    fn nat_event_raw(&self, id: &str, practice: bool, key: &str) -> &str {
        let d = self.nat_event_draft(id, practice);
        match key {
            "network" => &d.network,
            "prefix" => &d.prefix,
            "first" => &d.first,
            "last" => &d.last,
            "inbound-global" => &d.inbound_global,
            _ => panic!("closed field"),
        }
    }
    pub(in crate::ui) fn nat_reset_queue(&self, ctx: &egui::Context) {
        let key = egui::Id::new(("nat-queue", self.smoke_stage));
        ctx.data_mut(|d| d.remove::<Queue>(key));
    }
    fn nat_click_for(&mut self, ctx: &egui::Context, t: &Target, q: &mut Queue, id: &str) -> bool {
        if id != ID && matches!(t, Target::RemoveRow(_) | Target::Field(_)) {
            let is_static = matches!(t, Target::RemoveRow(_))
                || matches!(t,Target::Field(k) if k.starts_with("static:"));
            let phase = ctx
                .data(|d| {
                    d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage)))
                })
                .unwrap_or(0);
            if is_static && phase < 2 {
                let root = self
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                let child = self.metrics.nat_static_viewport.unwrap();
                let (target, visible, _) = self.nat_target(ctx, t);
                if let Some(piece) =
                    static_root_reveal_piece(target, visible, child, root, ctx.pixels_per_point())
                    && !self.nat_reveal(ctx, piece, root, false, q)
                {
                    return false;
                }
            }
        }
        self.nat_click(ctx, t, q)
    }
    pub(in crate::ui) fn nat_queue_for(
        &mut self,
        ctx: &egui::Context,
        ops: Vec<Op>,
        id: &str,
        practice: bool,
    ) -> bool {
        assert!(ops.len() <= 24);
        let key = egui::Id::new(("nat-queue", self.smoke_stage));
        let mut q = ctx.data(|d| d.get_temp::<Queue>(key)).unwrap_or_default();
        q.ticks += 1;
        let click_phase = ctx
            .data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))));
        let focus = ctx.memory(|m| m.focused());
        let pointer = ctx.input(|i| i.pointer.hover_pos());
        let fields = self
            .metrics
            .nat_fields
            .iter()
            .map(|r| (r.0.as_str(), r.1, r.2, r.4))
            .filter(|r| {
                Some(r.2) == focus
                    || match ops.get(q.next) {
                        Some(Op::RowText(row, global, _)) => {
                            let suffix = if *global { ":global" } else { ":inside" };
                            self.metrics
                                .nat_fields
                                .iter()
                                .filter(|r| r.0.starts_with("static:") && r.0.ends_with(suffix))
                                .nth(*row)
                                .is_some_and(|target| target.2 == r.2)
                        }
                        Some(Op::Text(k, _)) => r.0 == *k,
                        _ => false,
                    }
            })
            .collect::<Vec<_>>();
        if q.trace.len() == 3 {
            q.trace.pop_front();
        }
        q.trace.push_back(format!("queue tick{} item{} sub{} op{:?} click_phase{click_phase:?} focus{focus:?} pointer{pointer:?} fields{fields:?} root{:?} child{:?} scroll{:?} stable{:?}/{} waiting_wheel{} idle{} zoom{} PPP{}",q.ticks,q.next,q.sub,ops.get(q.next),self.metrics.main_viewport,self.metrics.nat_static_viewport,self.metrics.nat_static_scroll,q.stable,q.stable_frames,q.waiting_wheel,q.idle_frames,ctx.zoom_factor(),ctx.pixels_per_point()));
        assert!(
            q.ticks <= 300,
            "NAT event queue bounded at stage{} item{}; last3 {:?}",
            self.smoke_stage,
            q.next,
            q.trace
        );
        if q.next >= ops.len() {
            return true;
        }
        let op = &ops[q.next];
        let done = match op {
            Op::Click(t) => {
                let structural = matches!(t, Target::RemoveRow(_) | Target::Control("add-static"));
                if structural && q.list_before.is_none() {
                    q.list_before = Some(self.nat_event_draft(id, practice).statics.len());
                }
                let done = self.nat_click_for(ctx, t, &mut q, id);
                if done && structural {
                    let before = q.list_before.unwrap();
                    let expected = if matches!(t, Target::RemoveRow(_)) {
                        before - 1
                    } else {
                        before + 1
                    };
                    assert_eq!(
                        self.nat_event_draft(id, practice).statics.len(),
                        expected,
                        "Actual row delta before queue advance"
                    );
                }
                done
            }
            Op::Text(k, text) => {
                assert!(text.len() <= 64);
                if q.sub == 0 {
                    if self.nat_click_for(ctx, &Target::Field((*k).into()), &mut q, id) {
                        let id = self
                            .metrics
                            .nat_fields
                            .iter()
                            .find(|r| r.0 == *k)
                            .unwrap()
                            .2;
                        assert!(ctx.memory(|m| m.has_focus(id)));
                        q.sub = 1;
                    }
                    false
                } else if q.sub == 1 {
                    self.pending_events.extend([
                        ctrl_key(egui::Key::A, true),
                        ctrl_key(egui::Key::A, false),
                        egui::Event::Text(text.clone()),
                    ]);
                    q.sub = 2;
                    false
                } else {
                    assert_eq!(self.nat_event_raw(id, practice, k), text);
                    true
                }
            }
            Op::Choose(control, selected_id) => {
                if q.sub == 0 {
                    if self.nat_click_for(ctx, &Target::Control(control), &mut q, id) {
                        q.sub = 1;
                        q.stable = None;
                        q.scrolls = 0;
                    }
                    false
                } else if self.nat_click(
                    ctx,
                    &Target::Choice(format!("{control}:{selected_id}")),
                    &mut q,
                ) {
                    let selected = if *control == "inside" {
                        self.nat_event_draft(id, practice).inside.as_deref()
                    } else {
                        self.nat_event_draft(id, practice).peer.as_deref()
                    };
                    assert_eq!(selected, Some(*selected_id));
                    true
                } else {
                    false
                }
            }
            Op::Select(control, selected_id) => {
                assert!(selected_id.len() <= 32);
                if q.sub == 0 {
                    if self.nat_click_for(ctx, &Target::Control(control), &mut q, id) {
                        q.sub = 1;
                        q.stable = None;
                        q.scrolls = 0;
                    }
                    false
                } else if self.nat_click(
                    ctx,
                    &Target::Choice(format!("{control}:{selected_id}")),
                    &mut q,
                ) {
                    let selected = if *control == "inside" {
                        self.nat_event_draft(id, practice).inside.as_deref()
                    } else {
                        self.nat_event_draft(id, practice).peer.as_deref()
                    };
                    assert_eq!(selected, Some(selected_id.as_str()));
                    true
                } else {
                    false
                }
            }
            Op::Example(index) => {
                assert!(*index < 16);
                if q.sub == 0 {
                    if self.nat_click(ctx, &Target::ExamplePicker, &mut q) {
                        q.sub = 1;
                        q.stable = None;
                        q.scrolls = 0;
                    }
                    false
                } else if self.nat_click(ctx, &Target::Choice(format!("example:{index}")), &mut q) {
                    assert_eq!(
                        self.progress.attempts[id]
                            .workspace
                            .as_ref()
                            .unwrap()
                            .nat_learning
                            .as_ref()
                            .unwrap()
                            .selected_action,
                        *index
                    );
                    true
                } else {
                    false
                }
            }
            Op::RowText(row, global, text) => {
                assert!(text.len() <= if *global { 64 } else { 32 });
                let suffix = if *global { ":global" } else { ":inside" };
                let actual = self
                    .metrics
                    .nat_fields
                    .iter()
                    .filter(|r| r.0.starts_with("static:") && r.0.ends_with(suffix))
                    .nth(*row)
                    .expect("actual static row field");
                let key = actual.0.clone();
                let field_id = actual.2;
                if q.sub == 0 {
                    if self.nat_click_for(ctx, &Target::Field(key), &mut q, id) {
                        assert!(ctx.memory(|m| m.has_focus(field_id)));
                        q.sub = 1;
                    }
                    false
                } else if q.sub == 1 {
                    self.pending_events.extend([
                        ctrl_key(egui::Key::A, true),
                        ctrl_key(egui::Key::A, false),
                        egui::Event::Text(text.clone()),
                    ]);
                    q.sub = 2;
                    false
                } else {
                    let r = &self.nat_event_draft(id, practice).statics[*row];
                    assert_eq!(if *global { &r.global } else { &r.inside }, text);
                    true
                }
            }
            Op::FileText(text) => {
                assert!(text.len() <= 64);
                if q.sub == 0 {
                    if self.nat_click(ctx, &Target::FileName, &mut q) {
                        q.sub = 1;
                    }
                    false
                } else if q.sub == 1 {
                    self.pending_events.extend([
                        ctrl_key(egui::Key::A, true),
                        ctrl_key(egui::Key::A, false),
                        egui::Event::Text(text.clone()),
                    ]);
                    q.sub = 2;
                    false
                } else {
                    assert_eq!(self.data_tools.restore_filename(), Some(text.as_str()));
                    true
                }
            }
            Op::Undo => {
                if q.sub == 0 {
                    self.pending_events
                        .extend([ctrl_key(egui::Key::Z, true), ctrl_key(egui::Key::Z, false)]);
                    q.sub = 1;
                    false
                } else {
                    true
                }
            }
            Op::Expect(k, text) => {
                assert_eq!(self.nat_event_raw(id, practice, k), text);
                true
            }
        };
        if done {
            q.next += 1;
            q.sub = 0;
            q.scrolls = 0;
            q.list_before = None;
            q.trace.clear();
            q.waiting_wheel = false;
            q.wheel_geometry = None;
            q.idle_frames = 0;
            q.settled_frames = 0;
            q.stable = None;
            q.stable_frames = 0;
        }
        let finished = q.next == ops.len();
        ctx.data_mut(|d| d.insert_temp(key, q));
        if !finished {
            self.started = Instant::now();
            ctx.request_repaint();
        }
        finished
    }
    fn nat_pass(&self, pass: bool) {
        let i = self.nat_index();
        assert_eq!(
            progress::current_solved(&self.progress.attempts[ID], &self.scenarios[i]),
            pass
        );
        assert_eq!(self.nat_presentation.memo.prepared(ID), Some(pass));
        assert_eq!(self.metrics.completion, pass);
    }
    pub(in crate::ui) fn nat_files(&self) -> std::collections::BTreeMap<String, Vec<u8>> {
        let mut files = std::collections::BTreeMap::new();
        let mut total = 0usize;
        let mut actual_total = 0usize;
        for (n, item) in std::fs::read_dir(self.path.parent().unwrap())
            .unwrap()
            .enumerate()
        {
            assert!(n < 16, "closed private profile file count");
            let item = item.unwrap();
            let kind = item.file_type().unwrap();
            assert!(kind.is_file() && !kind.is_symlink());
            total = total
                .checked_add(usize::try_from(item.metadata().unwrap().len()).unwrap())
                .unwrap();
            assert!(total <= 1024 * 1024);
            let name = item.file_name().into_string().unwrap();
            assert!(name.len() <= 256);
            let bytes = progress::read_bounded(&item.path(), MAX_PROFILE).unwrap();
            actual_total = actual_total.checked_add(bytes.len()).unwrap();
            assert!(actual_total <= 1024 * 1024);
            files.insert(name, bytes);
        }
        files
    }
    pub(in crate::ui) fn nat_warm(&mut self) -> Warm {
        let portable = serde_json::to_value(&self.progress).unwrap();
        let caller_receipts = serde_json::to_value(&self.progress.migration_receipts).unwrap();
        self.save();
        assert!(self.storage_error.is_none() && !self.dirty);
        let raw = progress::read_bounded(&self.path, MAX_PROFILE).unwrap();
        let mut projected: Value = serde_json::from_slice(&raw).unwrap();
        let receipts = projected
            .as_object_mut()
            .unwrap()
            .remove("migration_receipts")
            .expect("main-only receipts");
        exact(
            &projected,
            &portable,
            "All portable keys equal exact raw projection",
        );
        let loaded = progress::load(&self.path, &self.scenarios).unwrap();
        exact(&loaded, &portable, "Actual normal loaded portable state");
        exact(
            &loaded.migration_receipts,
            &receipts,
            "Loaded/main receipts exact",
        );
        exact(
            &self.progress.migration_receipts,
            &caller_receipts,
            "Normal save keeps caller receipts unchanged",
        );
        assert_eq!(
            progress::read_bounded(&self.path, MAX_PROFILE).unwrap(),
            raw
        );
        Warm {
            raw,
            portable,
            raw_receipts: receipts,
            caller_receipts,
        }
    }
    fn nat_shot(&mut self, ctx: &egui::Context, name: &'static str) -> (bool, Option<String>) {
        if !self.nat_geometry(ctx) {
            return (false, None);
        }
        let key = egui::Id::new(("nat-shot", self.smoke_stage));
        let segment = ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0);
        let focused = name.contains("focused");
        let is_task = name.ends_with("task");
        let root = self
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if name.contains("150") || name.contains("200") {
            let expected = if name.contains("150") {
                "198.51.100.14"
            } else {
                "198.51.100.15"
            };
            assert_eq!(self.nat_raw("last"), expected);
            if name.contains("current-pass") {
                assert_eq!(
                    self.nat_live()
                        .configuration()
                        .dynamic
                        .unwrap()
                        .pool
                        .last
                        .to_string(),
                    expected
                );
                self.nat_pass(true);
            }
        }
        let target = if self.selected.is_none() {
            assert_eq!(self.metrics.card_actions.len(), 1);
            self.metrics.card_actions[0].0
        } else if focused {
            let r = self
                .metrics
                .nat_fields
                .iter()
                .find(|r| r.0 == "last")
                .unwrap();
            assert!(ctx.memory(|m| m.has_focus(r.2)));
            r.1
        } else if is_task {
            self.metrics
                .workspace_header_controls
                .iter()
                .find(|r| r.0 == "back")
                .unwrap()
                .1
        } else {
            self.metrics
                .nat_feedback
                .or(self.metrics.nat_observation)
                .unwrap_or(self.metrics.check.unwrap())
        };
        let tall = target.height() > root.height() - 4.;
        let piece = if tall {
            let h = (root.height() * 0.65).min(target.height());
            if segment == 0 {
                egui::Rect::from_min_size(target.min, egui::vec2(target.width(), h))
            } else {
                egui::Rect::from_min_max(egui::pos2(target.left(), target.bottom() - h), target.max)
            }
        } else {
            target
        };
        let mut q = ctx
            .data(|d| d.get_temp::<Queue>(key.with("reveal")))
            .unwrap_or_default();
        let visible = self.nat_reveal(ctx, piece, root, false, &mut q);
        ctx.data_mut(|d| d.insert_temp(key.with("reveal"), q));
        if !visible {
            return (false, None);
        }
        let rect = ctx.content_rect();
        let zoom = ctx.zoom_factor();
        let ppp = ctx.pixels_per_point();
        let outer = ctx.input(|i| i.viewport().outer_rect);
        let fullscreen = ctx.input(|i| i.viewport().fullscreen);
        println!(
            "NAT capture {name} segment{segment} actual_content={rect:?} actual_outer={outer:?} actual_fullscreen={fullscreen:?} zoom={zoom} PPP={ppp}"
        );
        let label = format!(
            "nat-{name}-{}-zoom{zoom:.2}-ppp{ppp:.2}",
            if tall {
                if segment == 0 { "top" } else { "bottom" }
            } else {
                "visible"
            }
        );
        if tall && segment == 0 {
            ctx.data_mut(|d| {
                d.insert_temp(key, 1u8);
                d.remove::<Queue>(key.with("reveal"));
            });
            self.capture_pending = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                label,
            )));
            return (false, None);
        }
        (true, Some(label))
    }
    pub(in crate::ui) fn nat_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        self.nat_private();
        assert!((FIRST..=LAST).contains(&stage));
        let step = PLAN[stage - FIRST];
        if !matches!(step, Step::Admit | Step::Start) {
            assert!(self.scenarios.iter().any(|s| s.id == ID));
        }
        let mut ops = Vec::new();
        match step {
            Step::Admit => {
                assert_eq!(
                    (
                        one_off_boundary::before_len(&self.scenarios),
                        one_off_boundary::before_len(&self.units)
                    ),
                    (176, 46)
                );
                assert!(self.progress.attempts.is_empty());
                assert!(self.scenarios.iter().all(|s| s.id != ID));
                let s = pack::load(include_str!(
                    "../../../tests/fixtures/nat-workspace-preview.json"
                ))
                .unwrap()
                .remove(0);
                assert_eq!(s.id, ID);
                self.scenarios.push(s);
                self.filter = catalog::Filter {
                    search: "Development address mapping workspace".into(),
                    ..Default::default()
                };
                self.selected = None;
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            }
            Step::Start => {
                ops = vec![click(Target::Card(true)), click(Target::Card(false))];
            }
            Step::Initial => {
                assert_eq!(self.selected, Some(self.nat_index()));
                assert_eq!(self.progress.attempts.len(), 1);
                exact(
                    &self.progress.attempts[ID].state,
                    &self.scenarios[self.nat_index()].initial,
                    "Actual Start initial state",
                );
                assert!(self.progress.favorites.contains(ID));
                assert!(self.nat_live().bindings().is_empty());
            }
            Step::Shot(name) => return self.nat_shot(ctx, name),
            Step::Check => {
                self.nat_remember(
                    ctx,
                    "before-check",
                    serde_json::to_value(self.nat_live()).unwrap(),
                );
                ops = vec![click(Target::Control("check"))];
            }
            Step::InitialFailure => {
                self.nat_pass(false);
                exact(
                    self.nat_live(),
                    &self.nat_observed(ctx, "before-check"),
                    "Check must not allocate",
                );
                return self.nat_shot(ctx, "initial-feedback");
            }
            Step::StaticReverse => {
                ops = vec![
                    click(Target::Control("add-static")),
                    Op::RowText(1, false, "unused".into()),
                    Op::RowText(1, true, "198.51.100.15".into()),
                    click(Target::RemoveRow(1)),
                    choice("inside", "observer"),
                    choice("peer", "peer"),
                    text("inbound-global", "198.51.100.50"),
                    click(Target::Control("inbound")),
                ];
            }
            Step::Malformed => {
                self.nat_remember(
                    ctx,
                    "before-malformed",
                    serde_json::to_value(self.nat_live()).unwrap(),
                );
                ops = vec![
                    text("prefix", "bad"),
                    Op::Undo,
                    Op::Expect("prefix", "24".into()),
                    text("prefix", "bad"),
                    click(Target::Control("apply")),
                ];
            }
            Step::AtomicFailure => {
                exact(
                    self.nat_live(),
                    &self.nat_observed(ctx, "before-malformed"),
                    "Malformed Apply atomic",
                );
                assert!(self.nat_draft().notice.is_some());
                ops = vec![click(Target::Control("check"))];
            }
            Step::Policy => {
                ops = vec![
                    text("network", "10.0.0.8"),
                    text("prefix", "30"),
                    text("first", "198.51.100.10"),
                    text("last", "198.51.100.12"),
                ];
            }
            Step::Pending => {
                ops = vec![click(Target::Control("check"))];
            }
            Step::Apply => {
                ops = vec![
                    click(Target::Control("apply")),
                    click(Target::Control("check")),
                ];
            }
            Step::AppliedEmpty => {
                assert!(self.nat_live().bindings().is_empty());
                assert_eq!(
                    self.nat_live()
                        .configuration()
                        .dynamic
                        .unwrap()
                        .source
                        .length,
                    30
                );
                self.nat_pass(false);
                return self.nat_shot(ctx, "applied-no-current-maps");
            }
            Step::Alpha => {
                ops = vec![
                    choice("inside", "alpha"),
                    click(Target::Control("outbound")),
                    click(Target::Control("check")),
                ];
            }
            Step::Partial => {
                self.nat_pass(false);
                assert!(self.nat_live().bindings().contains_key("alpha"));
                assert!(!self.nat_live().bindings().contains_key("beta"));
                let rows = progress::current_nat_checks(
                    &self.progress.attempts[ID],
                    &self.scenarios[self.nat_index()],
                );
                let engine::Goal::BasicNat { definition } = &self.scenarios[self.nat_index()].goal
                else {
                    unreachable!()
                };
                assert!(
                    rows[1 + definition.requirements.len()..]
                        .iter()
                        .all(|r| r.0),
                    "All fresh trial rows remain healthy; only held beta is missing"
                );
            }
            Step::Beta => {
                ops = vec![
                    choice("inside", "beta"),
                    click(Target::Control("outbound")),
                    click(Target::Control("check")),
                ];
            }
            Step::Complete => {
                self.nat_pass(true);
                assert_eq!(
                    self.nat_live().configuration().statics["observer"].to_string(),
                    "198.51.100.50"
                );
                self.nat_remember(
                    ctx,
                    "held",
                    serde_json::to_value(self.nat_live().bindings()).unwrap(),
                );
            }
            Step::ReverseActual => {
                let global = self.nat_live().bindings()["alpha"].to_string();
                ops = vec![
                    text("inbound-global", &global),
                    click(Target::Control("inbound")),
                ];
            }
            Step::PeerReuse => {
                ops = vec![
                    choice("inside", "alpha"),
                    choice("peer", "peer-two"),
                    click(Target::Control("outbound")),
                ];
            }
            Step::ReuseProof => {
                exact(
                    self.nat_live().bindings(),
                    &self.nat_observed(ctx, "held"),
                    "Peer reuse retains mapping",
                );
                self.nat_pass(true);
                return self.nat_shot(ctx, "peer-reuses-held-map");
            }
            Step::ReadonlyError => {
                ops = vec![
                    text("inbound-global", "invalid"),
                    click(Target::Control("inbound")),
                ];
            }
            Step::ReadonlyProof => {
                self.nat_pass(true);
                assert!(self.nat_draft().notice.is_some());
                exact(
                    self.nat_live().bindings(),
                    &self.nat_observed(ctx, "held"),
                    "Readonly request cannot change mappings",
                );
                return self.nat_shot(ctx, "readonly-error-applied-pass");
            }
            Step::Clear => {
                ops = vec![
                    click(Target::Control("clear")),
                    click(Target::Control("check")),
                ]
            }
            Step::Cleared => {
                assert!(self.nat_live().bindings().is_empty());
                assert_eq!(self.nat_live().configuration().statics.len(), 1);
                self.nat_pass(false);
            }
            Step::RepairCurrent => ops = mappings(false),
            Step::AlternateText => {
                ops = vec![
                    text("network", "10.0.0.10"),
                    text("prefix", "31"),
                    text("first", "198.51.100.12"),
                    text("last", "198.51.100.15"),
                ];
            }
            Step::AlternateApply => ops = vec![click(Target::Control("apply"))],
            Step::AlternateMaps => ops = mappings(false),
            Step::AlternateProof => {
                self.nat_pass(true);
                self.nat_remember(
                    ctx,
                    "alternate-held",
                    serde_json::to_value(self.nat_live().bindings()).unwrap(),
                );
            }
            Step::SettingsBlock | Step::SamplesBlock => {
                let before = serde_json::to_value(&self.progress).unwrap();
                let key = if matches!(step, Step::SettingsBlock) {
                    "settings"
                } else {
                    "samples"
                };
                if ctx
                    .data(|d| d.get_temp::<Value>(egui::Id::new(("nat-modal", key))))
                    .is_none()
                {
                    ctx.data_mut(|d| d.insert_temp(egui::Id::new(("nat-modal", key)), before));
                }
                ops = vec![
                    click(Target::Field("network".into())),
                    click(Target::Settings),
                    click(Target::Tab("appearance")),
                ];
                if matches!(step, Step::SamplesBlock) {
                    ops.push(click(Target::Samples));
                }
            }
            Step::SettingsClose | Step::SamplesClose => {
                if ctx
                    .data(|d| d.get_temp::<Queue>(egui::Id::new(("nat-queue", self.smoke_stage))))
                    .is_none()
                {
                    assert!(self.settings_open);
                }
                let key = if matches!(step, Step::SettingsClose) {
                    "settings"
                } else {
                    "samples"
                };
                let before = ctx
                    .data(|d| d.get_temp::<Value>(egui::Id::new(("nat-modal", key))))
                    .unwrap();
                exact(&self.progress, &before, "Modal open retains all progress");
                ops = vec![click(Target::SettingsClose)];
            }
            Step::DataCancel => {
                let key = egui::Id::new("nat-data-before");
                if ctx.data(|d| d.get_temp::<Value>(key)).is_none() {
                    let before = serde_json::to_value(&self.progress).unwrap();
                    let files = self.nat_files();
                    ctx.data_mut(|d| {
                        d.insert_temp(key, before);
                        d.insert_temp(key.with("files"), files);
                    });
                }
                ops = vec![
                    click(Target::Settings),
                    click(Target::Tab("data")),
                    click(Target::DataExport),
                    Op::FileText("../invalid-destination".into()),
                    click(Target::FileCancel),
                    click(Target::SettingsClose),
                ];
            }
            Step::ResetCancel => {
                let key = egui::Id::new("nat-reset-before");
                if ctx.data(|d| d.get_temp::<Value>(key)).is_none() {
                    let before = serde_json::to_value(&self.progress).unwrap();
                    ctx.data_mut(|d| d.insert_temp(key, before));
                }
                ops = vec![click(Target::Reset), click(Target::CancelReset)];
            }
            Step::PendingText => ops = vec![text("network", "pending invalid network")],
            Step::WarmSave => {
                let warm = self.nat_warm();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("nat-warm"), warm));
            }
            Step::Back => ops = vec![click(Target::Back)],
            Step::Continue => ops = vec![click(Target::Card(false))],
            Step::ResumeProof => {
                let warm = ctx
                    .data(|d| d.get_temp::<Warm>(egui::Id::new("nat-warm")))
                    .unwrap();
                let mut expected = warm.portable.clone();
                let clock = expected["clock"].as_u64().unwrap() + 1;
                expected["clock"] = clock.into();
                expected["attempts"][ID]["visited"] = clock.into();
                exact(
                    &self.progress,
                    &expected,
                    "Only actual Continue clock+visited change",
                );
                exact(
                    &self.progress.migration_receipts,
                    &warm.caller_receipts,
                    "Continue retains caller receipts",
                );
                let mut anchored: Value = serde_json::from_slice(&warm.raw).unwrap();
                let receipts = anchored
                    .as_object_mut()
                    .unwrap()
                    .remove("migration_receipts")
                    .unwrap();
                exact(&anchored, &warm.portable, "Original warm raw anchor exact");
                exact(
                    &receipts,
                    &warm.raw_receipts,
                    "Original warm receipt anchor exact",
                );
                let fresh = self.nat_warm();
                exact(
                    &fresh.portable,
                    &expected,
                    "Actual continued normal readback whole portable",
                );
                assert_eq!(self.nat_raw("network"), "pending invalid network");
                return self.nat_shot(ctx, "warm-resumed-pending");
            }
            Step::RepairText => {
                ops = vec![
                    text("network", "10.0.0.10"),
                    click(Target::Control("check")),
                ]
            }
            Step::Challenge => ops = vec![click(Target::Challenge)],
            Step::Learn => ops = vec![click(Target::Learn)],
            Step::Light | Step::Dark | Step::Normal => {
                let (light, scale, width) = match step {
                    Step::Light => (true, 1.5, 1400.),
                    Step::Dark => (false, 2., 946.),
                    _ => (false, 1., 1200.),
                };
                self.theme_id = if light { "dlms-light" } else { "dlms-dark" }.into();
                self.scale = scale;
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(width, 950.)));
            }
            Step::LightText => ops = vec![text("last", "198.51.100.14")],
            Step::DarkText => ops = vec![text("last", "198.51.100.15")],
            Step::LightApply | Step::DarkApply => {
                ops = vec![
                    click(Target::Control("apply")),
                    click(Target::Control("check")),
                ]
            }
            Step::ConfirmReset => ops = vec![click(Target::Reset), click(Target::ConfirmReset)],
            Step::ResetProof => {
                exact(
                    &self.progress.attempts[ID].state,
                    &self.scenarios[self.nat_index()].initial,
                    "Actual selected reset",
                );
                assert!(self.progress.favorites.contains(ID));
                assert!(self.nat_live().bindings().is_empty());
                self.nat_pass(false);
                return self.nat_shot(ctx, "reset-initial");
            }
            Step::RepairAfterReset => {
                ops = vec![
                    text("network", "10.0.0.8"),
                    text("prefix", "30"),
                    click(Target::Control("apply")),
                ];
                ops.extend(mappings(true));
            }
            Step::FinalSave => {
                assert!(self.selected.is_none());
                assert_eq!(self.progress.attempts.len(), 1);
                self.nat_warm();
                let original = pack::bundled().unwrap();
                exact(
                    &self.scenarios[..original.len()],
                    &original,
                    "All current normal metadata unchanged",
                );
                assert_eq!(one_off_boundary::before_len(&self.units), 46);
                return self.nat_shot(ctx, "technical-library-final");
            }
            Step::Finish => {
                println!(
                    "PASS private technical Basic NAT GUI: actual Start/config/probes/check, pending/readonly/atomic/alternate/retry, strict warm readback/reset and observed100/150/200. No curriculum, packet, PAT, host, novice or cold-process claim."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return (false, None);
            }
        }
        if !ops.is_empty() && !self.nat_queue(ctx, ops) {
            return (false, None);
        }
        // Post-event exact acknowledgements. Never advance on a click pulse alone.
        match step {
            Step::Start => {
                assert_eq!(self.selected, Some(self.nat_index()));
                assert!(self.progress.favorites.contains(ID));
            }
            Step::StaticReverse => {
                assert!(self.nat_live().bindings().is_empty());
                assert_eq!(self.nat_draft().statics.len(), 1);
                let peer = self.nat_live().inventory().outside["peer"];
                let local = self.nat_live().inventory().inside["observer"];
                let global = self.nat_live().configuration().statics["observer"];
                assert!(
                    matches!(self.nat_presentation.observed_effect(),Some(core::Effect::Probe{result:core::ProbeResult::Translated{before,after,kind:core::MappingKind::Static}})if before.source==peer&&before.destination==global&&after.source==peer&&after.destination==local)
                );
            }
            Step::ReverseActual => {
                let peer = self.nat_live().inventory().outside["peer"];
                let local = self.nat_live().inventory().inside["alpha"];
                let global = self.nat_live().bindings()["alpha"];
                assert!(
                    matches!(self.nat_presentation.observed_effect(),Some(core::Effect::Probe{result:core::ProbeResult::Translated{before,after,kind:core::MappingKind::Dynamic}})if before.source==peer&&before.destination==global&&after.source==peer&&after.destination==local)
                );
            }
            Step::PeerReuse => {
                let global = self.nat_live().bindings()["alpha"];
                let local = self.nat_live().inventory().inside["alpha"];
                let peer = self.nat_live().inventory().outside["peer-two"];
                assert!(
                    matches!(self.nat_presentation.observed_effect(),Some(core::Effect::Probe{result:core::ProbeResult::Translated{before,after,kind:core::MappingKind::Dynamic}})if before.source==local&&before.destination==peer&&after.source==global&&after.destination==peer)
                );
            }
            Step::DataCancel => {
                let key = egui::Id::new("nat-data-before");
                let before = ctx.data(|d| d.get_temp::<Value>(key)).unwrap();
                let files = ctx
                    .data(|d| {
                        d.get_temp::<std::collections::BTreeMap<String, Vec<u8>>>(key.with("files"))
                    })
                    .unwrap();
                exact(
                    &self.progress,
                    &before,
                    "Cancel preserves ALL progress fields",
                );
                assert!(
                    self.nat_files() == files,
                    "Cancel must cause zero file changes"
                );
                assert!(!self.data_tools.modal_open() && !self.settings_open);
                assert!(self.metrics.nat_fields.iter().all(|r| r.3
                    || ["network", "prefix", "first", "last"].contains(&r.0.as_str())
                        && !self.nat_draft().dynamic_enabled));
            }
            Step::ResetCancel => {
                let key = egui::Id::new("nat-reset-before");
                let before = ctx.data(|d| d.get_temp::<Value>(key)).unwrap();
                exact(
                    &self.progress,
                    &before,
                    "Actual Reset Cancel retains all fields",
                );
                assert!(!self.reset_pending);
            }
            Step::SettingsClose | Step::SamplesClose => assert!(!self.settings_open),
            Step::Pending => {
                self.nat_pass(false);
                assert!(self.nat_live().bindings().is_empty());
            }
            Step::RepairCurrent
            | Step::AlternateMaps
            | Step::RepairText
            | Step::RepairAfterReset => self.nat_pass(true),
            Step::LightApply | Step::DarkApply => {
                let expected = if matches!(step, Step::LightApply) {
                    "198.51.100.14"
                } else {
                    "198.51.100.15"
                };
                assert_eq!(self.nat_raw("last"), expected);
                assert_eq!(
                    self.nat_live()
                        .configuration()
                        .dynamic
                        .unwrap()
                        .pool
                        .last
                        .to_string(),
                    expected
                );
                exact(
                    self.nat_live().bindings(),
                    &self.nat_observed(ctx, "alternate-held"),
                    "Apply keeps held mappings",
                );
                self.nat_pass(true);
            }
            Step::SettingsBlock | Step::SamplesBlock => {
                assert!(self.settings_open);
                assert!(self.metrics.nat_fields.iter().all(|r| !r.3));
                self.pending_events.extend([
                    egui::Event::Text("blocked".into()),
                    ctrl_key(egui::Key::Enter, true),
                    ctrl_key(egui::Key::Enter, false),
                ]);
            }
            _ => {}
        }
        (true, None)
    }
}
