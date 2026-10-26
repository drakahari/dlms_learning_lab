//! Closed normal-course steady-state tasks; references identify UI edits only.
use super::*;
use dlms_learning_lab::{
    spanning_tree::{Action, State as Simulation},
    spanning_tree_workspace::Draft,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
const FIRST: usize = 35000;
const END: usize = 36000;
const UNIT: &str = "spanning-tree-steady-state";
const IDS: [&str; 4] = [
    "ccna-stp-root",
    "ccna-stp-local-cost",
    "ccna-stp-parallel",
    "ccna-stp-handover",
];
const RAW_ASSIGNMENT: &str = "unfinished assignment";
const RAW_EXAMPLE: &str = "different unfinished example";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Review {
    id: String,
    wrong: Vec<Action>,
    partial: Vec<Action>,
    alternate: Vec<Action>,
    break_current: Vec<Action>,
    warm_priority: u16,
}
fn reviews() -> Vec<Review> {
    let r: Vec<Review> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/spanning-tree-native-review.json"
    ))
    .unwrap();
    assert_eq!(r.len(), 4);
    for (row, id) in r.iter().zip(IDS) {
        assert_eq!(row.id, id);
        for a in [&row.wrong, &row.partial, &row.alternate, &row.break_current] {
            assert!(!a.is_empty() && a.len() <= 8);
            for a in a {
                assert!(matches!(
                    a,
                    Action::BridgePriority { .. } | Action::Port { .. }
                ));
            }
        }
    }
    r
}
#[derive(Clone, Debug, PartialEq)]
enum Target {
    Try,
    Example,
    Assignment,
    Insert,
    Apply,
    Check,
    BridgeMenu,
    Bridge(String),
    PortMenu,
    Port(String),
    Back,
    Favorite,
    Reset,
    Cancel,
    Confirm,
    Challenge,
}
#[derive(Clone, Debug, PartialEq)]
enum Shot {
    Heading,
    Result,
    Field(String),
}
#[derive(Clone, Debug)]
enum Step {
    Select(usize),
    Wait,
    Click(Target),
    SelectedBridge(String),
    SelectedPort(String),
    Field(String, String),
    Applied(Vec<Action>, bool),
    RecordInsert,
    InsertOnly,
    Practice(bool),
    Current(bool),
    Partial,
    RecordReset,
    CancelExact,
    ResetExact,
    RecordState,
    Rejected,
    RecordDrafts,
    SaveReadback,
    Filter,
    Resume,
    ResumeExact,
    Theme(bool),
    Geometry,
    Challenge,
    Capture(String, Shot),
    Library,
    Finish,
}
fn actions(p: &mut Vec<Step>, actions: &[Action], practice: bool) {
    assert!(!actions.is_empty() && actions.len() <= 32);
    for a in actions {
        let bridge = match a {
            Action::BridgePriority { bridge, .. } | Action::Port { bridge, .. } => bridge,
            _ => panic!("closed lessons use priorities/costs only"),
        };
        p.extend([
            Step::Click(Target::BridgeMenu),
            Step::Click(Target::Bridge(bridge.clone())),
            Step::SelectedBridge(bridge.clone()),
        ]);
        match a {
            Action::BridgePriority { priority, .. } => p.push(Step::Field(
                format!("{bridge}:priority"),
                priority.to_string(),
            )),
            Action::Port { port, settings, .. } => {
                assert!(
                    settings.up,
                    "closed native plans retain supplied physical availability"
                );
                p.extend([
                    Step::Click(Target::PortMenu),
                    Step::Click(Target::Port(port.clone())),
                    Step::SelectedPort(port.clone()),
                    Step::Field(
                        format!("{bridge}/{port}:priority"),
                        settings.priority.to_string(),
                    ),
                    Step::Field(format!("{bridge}/{port}:cost"), settings.cost.to_string()),
                ]);
            }
            _ => unreachable!(),
        }
    }
    p.extend([
        Step::Click(Target::Apply),
        Step::Applied(actions.to_vec(), practice),
    ]);
}
fn check(p: &mut Vec<Step>, predicate: Step) {
    p.extend([Step::Click(Target::Check), Step::Wait, predicate]);
}
fn reset(p: &mut Vec<Step>) {
    p.extend([
        Step::RecordReset,
        Step::Click(Target::Reset),
        Step::Click(Target::Confirm),
        Step::Wait,
        Step::ResetExact,
        Step::Click(Target::Assignment),
        Step::Wait,
    ]);
}
fn plan(scenarios: &[Scenario]) -> Vec<Step> {
    let mut p = vec![];
    for (n, r) in reviews().iter().enumerate() {
        let s = scenarios
            .iter()
            .find(|s| s.id == r.id)
            .expect("normal registered STP lesson");
        assert_eq!(s.unit, UNIT);
        let l = s.spanning_tree_teaching.as_ref().unwrap();
        let reference: Vec<Action> = s
            .solution
            .iter()
            .map(|s| {
                serde_json::from_str(
                    s.strip_prefix("spanning-tree ")
                        .expect("typed reference metadata"),
                )
                .unwrap()
            })
            .collect();
        let name = |suffix: &str| format!("stp-lesson-{n}-{suffix}");
        p.extend([
            Step::Select(n),
            Step::Geometry,
            Step::Capture(name("introduction"), Shot::Heading),
            Step::Click(Target::Try),
            Step::Wait,
            Step::RecordInsert,
            Step::Click(Target::Insert),
            Step::InsertOnly,
        ]);
        let first = &l.example_actions[0].action;
        let bridge = match first {
            Action::BridgePriority { bridge, .. } | Action::Port { bridge, .. } => bridge,
            _ => panic!("closed example action"),
        };
        p.extend([
            Step::Click(Target::BridgeMenu),
            Step::Click(Target::Bridge(bridge.clone())),
            Step::SelectedBridge(bridge.clone()),
        ]);
        let fill_field = if let Action::Port { port, .. } = first {
            p.extend([
                Step::Click(Target::PortMenu),
                Step::Click(Target::Port(port.clone())),
                Step::SelectedPort(port.clone()),
            ]);
            format!("{bridge}/{port}:cost")
        } else {
            format!("{bridge}:priority")
        };
        p.extend([
            Step::Capture(name("example-fill-only"), Shot::Field(fill_field)),
            Step::Click(Target::Apply),
        ]);
        check(&mut p, Step::Practice(true));
        p.push(Step::Capture(name("example-complete"), Shot::Result));
        actions(&mut p, &l.try_change.actions, true);
        check(&mut p, Step::Practice(false));
        p.push(Step::Capture(name("example-experiment"), Shot::Result));
        actions(&mut p, &l.repair, true);
        check(&mut p, Step::Practice(true));
        p.extend([
            Step::Click(Target::Assignment),
            Step::Wait,
            Step::Capture(name("assignment"), Shot::Heading),
        ]);
        check(&mut p, Step::Current(false));
        actions(&mut p, &r.wrong, false);
        check(&mut p, Step::Current(false));
        reset(&mut p);
        actions(&mut p, &r.partial, false);
        check(&mut p, Step::Partial);
        p.push(Step::Capture(name("partial"), Shot::Result));
        reset(&mut p);
        actions(&mut p, &reference, false);
        check(&mut p, Step::Current(true));
        p.push(Step::Capture(name("reference"), Shot::Result));
        actions(&mut p, &r.break_current, false);
        check(&mut p, Step::Current(false));
        actions(&mut p, &reference, false);
        check(&mut p, Step::Current(true));
        reset(&mut p);
        actions(&mut p, &r.alternate, false);
        check(&mut p, Step::Current(true));
        p.push(Step::Capture(name("alternate"), Shot::Result));
        for light in [true, false] {
            p.extend([Step::Theme(light), Step::Geometry]);
            let edits = if light { &reference } else { &r.alternate };
            actions(&mut p, edits, false);
            check(&mut p, Step::Current(true));
            let field = match &edits[0] {
                Action::BridgePriority { bridge, .. } => format!("{bridge}:priority"),
                Action::Port { bridge, port, .. } => format!("{bridge}/{port}:cost"),
                _ => unreachable!(),
            };
            // Re-enter the exact reviewed value at the actual scale and retain real focus.
            let value = match &edits[0] {
                Action::BridgePriority { priority, .. } => priority.to_string(),
                Action::Port { settings, .. } => settings.cost.to_string(),
                _ => unreachable!(),
            };
            let bridge = match &edits[0] {
                Action::BridgePriority { bridge, .. } | Action::Port { bridge, .. } => bridge,
                _ => unreachable!(),
            };
            p.extend([
                Step::Click(Target::BridgeMenu),
                Step::Click(Target::Bridge(bridge.clone())),
                Step::SelectedBridge(bridge.clone()),
            ]);
            if let Action::Port { port, .. } = &edits[0] {
                p.extend([
                    Step::Click(Target::PortMenu),
                    Step::Click(Target::Port(port.clone())),
                    Step::SelectedPort(port.clone()),
                ]);
            }
            p.extend([
                Step::Field(field.clone(), value),
                Step::Capture(
                    name(if light {
                        "light150-focused-field"
                    } else {
                        "dark200-focused-field"
                    }),
                    Shot::Field(field),
                ),
            ]);
        }
        // A malformed pending edit must invalidate current credit without changing the
        // applied graph; a real Apply rejects it, then typing+Apply repairs the draft.
        p.extend([
            Step::Click(Target::BridgeMenu),
            Step::Click(Target::Bridge("core".into())),
            Step::SelectedBridge("core".into()),
            Step::RecordState,
            Step::Field("core:priority".into(), RAW_ASSIGNMENT.into()),
        ]);
        check(&mut p, Step::Current(false));
        p.extend([
            Step::Click(Target::Apply),
            Step::Rejected,
            Step::Field("core:priority".into(), r.warm_priority.to_string()),
            Step::Click(Target::Apply),
        ]);
        check(&mut p, Step::Current(true));
        p.extend([
            Step::Field("core:priority".into(), RAW_ASSIGNMENT.into()),
            Step::Click(Target::Example),
            Step::Wait,
            Step::Click(Target::BridgeMenu),
            Step::Click(Target::Bridge("sample-core".into())),
            Step::SelectedBridge("sample-core".into()),
            Step::Field("sample-core:priority".into(), RAW_EXAMPLE.into()),
            Step::RecordDrafts,
            Step::SaveReadback,
            Step::Click(Target::Back),
            Step::Filter,
            Step::Wait,
            Step::Click(Target::Favorite),
            Step::SaveReadback,
            Step::Resume,
            Step::Wait,
            Step::ResumeExact,
            Step::Capture(
                name("resumed-independent-drafts"),
                Shot::Field("sample-core:priority".into()),
            ),
            Step::Click(Target::Assignment),
            Step::Wait,
            Step::RecordReset,
            Step::Click(Target::Reset),
            Step::Click(Target::Cancel),
            Step::CancelExact,
        ]);
        // Clear only the pending field through real typing; the reference can coexist
        // with permitted alternate settings. Reset was already exercised three times.
        p.extend([
            Step::Click(Target::BridgeMenu),
            Step::Click(Target::Bridge("core".into())),
            Step::SelectedBridge("core".into()),
            Step::Field("core:priority".into(), r.warm_priority.to_string()),
            Step::Click(Target::Apply),
        ]);
        check(&mut p, Step::Current(true));
        p.extend([
            Step::SaveReadback,
            Step::Click(Target::Challenge),
            Step::Wait,
            Step::Challenge,
            Step::Capture(name("challenge"), Shot::Heading),
        ]);
    }
    p.extend([
        Step::Click(Target::Back),
        Step::Library,
        Step::Wait,
        Step::Finish,
    ]);
    assert!(
        p.len() < END - FIRST,
        "bounded normal STP range before36000"
    );
    p
}
fn draft(a: &progress::Attempt, practice: bool) -> &Draft {
    let w = a.workspace.as_ref().unwrap();
    if practice {
        &w.spanning_tree_learning.as_ref().unwrap().draft
    } else {
        w.spanning_tree_draft.as_ref().unwrap()
    }
}
fn simulation(a: &progress::Attempt, practice: bool) -> &Simulation {
    if practice {
        &a.workspace
            .as_ref()
            .unwrap()
            .spanning_tree_learning
            .as_ref()
            .unwrap()
            .simulation
    } else {
        let engine::State::SpanningTree { simulation } = &a.state else {
            panic!()
        };
        simulation
    }
}
fn field<'a>(d: &'a Draft, key: &str) -> &'a str {
    if let Some(bridge) = key.strip_suffix(":priority").filter(|s| !s.contains('/')) {
        return &d.bridges[bridge].priority;
    }
    let (endpoint, what) = key.rsplit_once(':').unwrap();
    let (bridge, port) = endpoint.split_once('/').unwrap();
    let p = &d.bridges[bridge].ports[port];
    match what {
        "priority" => &p.priority,
        "cost" => &p.cost,
        _ => panic!("closed field"),
    }
}
type Record = (Value, BTreeMap<String, Value>, BTreeSet<String>);
impl LabApp {
    fn stp_lesson_target(&self, t: &Target) -> Option<egui::Rect> {
        match t {
            Target::Try => self.metrics.example_action,
            Target::Example => self.metrics.lesson_tabs.get(1).copied(),
            Target::Assignment => self
                .metrics
                .assignment_action
                .or_else(|| self.metrics.lesson_tabs.get(2).copied()),
            Target::Insert => self.metrics.insert,
            Target::Apply => self.metrics.stp_apply,
            Target::Check => self.metrics.check,
            Target::BridgeMenu => self.metrics.stp_bridge,
            Target::Bridge(n) => self
                .metrics
                .stp_bridge_items
                .iter()
                .find(|v| v.0 == *n)
                .map(|v| v.1),
            Target::PortMenu => self.metrics.stp_port,
            Target::Port(n) => self
                .metrics
                .stp_port_items
                .iter()
                .find(|v| v.0 == *n)
                .map(|v| v.1),
            Target::Back => self.metrics.back,
            Target::Favorite => self.metrics.card_actions.first().map(|v| v.1),
            Target::Reset => self.metrics.reset,
            Target::Cancel => self.metrics.reset_cancel,
            Target::Confirm => self.metrics.confirm_reset,
            Target::Challenge => self.metrics.mode_challenge,
        }
    }
    fn stp_lesson_reveal(&mut self, ctx: &egui::Context, r: egui::Rect) -> bool {
        let rect = ctx.content_rect();
        let ppp = ctx.pixels_per_point();
        let clip = self.metrics.main_viewport.unwrap().intersect(rect);
        if smoke_control_visible(r, clip, ppp) {
            return true;
        }
        let count = ctx.data_mut(|d| {
            let k = egui::Id::new(("stp-lessons-reveal", self.smoke_stage));
            let n = d.get_temp::<u8>(k).unwrap_or(0) + 1;
            d.insert_temp(k, n);
            n
        });
        assert!(
            count <= 20,
            "STP lesson control clipped: {r:?}, clip {clip:?}"
        );
        let dy = if r.top() < clip.top() {
            clip.top() - r.top() + controls::GAP
        } else if r.bottom() > clip.bottom() {
            clip.bottom() - r.bottom() - controls::GAP
        } else {
            0.
        };
        let x = r.center().x.clamp(clip.left() + 1., clip.right() - 1.);
        self.pending_events.extend([
            egui::Event::PointerMoved(egui::pos2(x, clip.center().y)),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., dy.clamp(-400., 400.)),
                modifiers: Default::default(),
            },
        ]);
        self.started = Instant::now();
        ctx.request_repaint();
        false
    }
    fn stp_lesson_clicked(&self, ctx: &egui::Context) -> bool {
        ctx.data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))))
            .unwrap_or(0)
            >= 2
    }
    fn stp_lesson_click(&mut self, ctx: &egui::Context, t: &Target) -> bool {
        if self.stp_lesson_clicked(ctx) {
            return self.click_smoke_control(ctx, None);
        }
        if matches!(t, Target::Bridge(_) | Target::Port(_)) {
            let popup = if matches!(t, Target::Bridge(_)) {
                self.metrics.stp_bridge_popup
            } else {
                self.metrics.stp_port_popup
            };
            return self.mac_popup_choice(ctx, self.stp_lesson_target(t), popup);
        }
        let r = self
            .stp_lesson_target(t)
            .expect("actual STP lesson control");
        self.stp_lesson_reveal(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    fn stp_lesson_type(&mut self, ctx: &egui::Context, key: &str, text: &str, i: usize) -> bool {
        assert!(text.len() <= 64);
        let k = egui::Id::new(("stp-lesson-type", self.smoke_stage));
        let phase = ctx.data(|d| d.get_temp::<u8>(k)).unwrap_or(0);
        match phase {
            0 => {
                let (_, r, id) = self
                    .metrics
                    .stp_fields
                    .iter()
                    .find(|v| v.0 == key)
                    .expect("selected native field");
                let (r, id) = (*r, *id);
                if !self.stp_lesson_clicked(ctx) && !self.stp_lesson_reveal(ctx, r) {
                    return false;
                }
                if !self.click_smoke_control(ctx, Some(r)) {
                    return false;
                }
                assert!(ctx.memory(|m| m.has_focus(id)));
                ctx.data_mut(|d| d.insert_temp(k, 1u8));
                false
            }
            1 => {
                self.pending_events = vec![
                    egui::Event::Key {
                        key: egui::Key::A,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers {
                            ctrl: true,
                            command: true,
                            ..Default::default()
                        },
                    },
                    egui::Event::Text(text.into()),
                    egui::Event::Key {
                        key: egui::Key::A,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers: Default::default(),
                    },
                ];
                ctx.data_mut(|d| d.insert_temp(k, 2u8));
                false
            }
            2 => {
                assert_eq!(
                    field(
                        draft(
                            &self.progress.attempts[&self.scenarios[i].id],
                            self.spanning_tree_view(i) == teaching::View::Example
                        ),
                        key
                    ),
                    text
                );
                true
            }
            _ => unreachable!(),
        }
    }
    pub(super) fn spanning_tree_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert!((FIRST..END).contains(&stage));
        assert!(
            self._smoke_profile.is_some(),
            "private synthetic native profile"
        );
        assert_eq!(
            self.path.parent(),
            Some(self._smoke_profile.as_ref().unwrap().path())
        );
        let p = plan(&self.scenarios);
        let offset = stage - FIRST;
        let step = p.get(offset).expect("closed STP phase");
        let u = self
            .units
            .iter()
            .find(|u| u.id == UNIT)
            .expect("normal STP unit registered");
        assert_eq!(u.labs, IDS);
        let n = p[..=offset]
            .iter()
            .rev()
            .find_map(|s| {
                if let Step::Select(n) = s {
                    Some(*n)
                } else {
                    None
                }
            })
            .unwrap();
        let id = IDS[n];
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        let s = self.scenarios[i].clone();
        let l = s.spanning_tree_teaching.as_ref().unwrap();
        let practice =
            self.selected == Some(i) && self.spanning_tree_view(i) == teaching::View::Example;
        if std::env::var_os("DLMS_SMOKE_DIAGNOSTICS").is_some() {
            eprintln!("STP lesson {stage} {id} {step:?}");
        }
        match step {
            Step::Select(_) => {
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
                self.launch(i);
                assert_eq!(self.spanning_tree_view(i), teaching::View::Introduction);
            }
            Step::Wait => {}
            Step::Click(t) => {
                if matches!(t, Target::Favorite) {
                    assert!(self.selected.is_none());
                    assert_eq!(self.filter.search, s.title);
                    assert_eq!(self.metrics.card_actions.len(), 1);
                }
                if !self.stp_lesson_click(ctx, t) {
                    return (false, None);
                }
            }
            Step::SelectedBridge(b) => assert_eq!(
                draft(&self.progress.attempts[id], practice).selected_bridge,
                *b
            ),
            Step::SelectedPort(p) => assert_eq!(
                draft(&self.progress.attempts[id], practice)
                    .selected_port
                    .as_ref(),
                Some(p)
            ),
            Step::Field(key, text) => {
                if !self.stp_lesson_type(ctx, key, text, i) {
                    return (false, None);
                }
            }
            Step::Applied(actions, was_practice) => {
                assert_eq!(practice, *was_practice);
                let a = &self.progress.attempts[id];
                let sim = simulation(a, practice);
                let d = draft(a, practice);
                assert!(d.matches(sim));
                for a in actions {
                    match a {
                        Action::BridgePriority { bridge, priority } => {
                            assert_eq!(sim.configuration().bridges[bridge].priority, *priority)
                        }
                        Action::Port {
                            bridge,
                            port,
                            settings,
                        } => assert_eq!(sim.configuration().bridges[bridge].ports[port], *settings),
                        _ => unreachable!(),
                    }
                }
                if practice {
                    assert_eq!(a.state, s.initial);
                }
            }
            Step::RecordInsert => {
                let a = &self.progress.attempts[id];
                let before = (
                    serde_json::to_value(simulation(a, true)).unwrap(),
                    serde_json::to_value(&a.state).unwrap(),
                );
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("stp-lessons-insert"), before));
            }
            Step::InsertOnly => {
                let a = &self.progress.attempts[id];
                let before = ctx
                    .data(|d| d.get_temp::<(Value, Value)>(egui::Id::new("stp-lessons-insert")))
                    .unwrap();
                assert_eq!(serde_json::to_value(simulation(a, true)).unwrap(), before.0);
                assert_eq!(serde_json::to_value(&a.state).unwrap(), before.1);
                assert!(!draft(a, true).matches(simulation(a, true)));
                assert!(!progress::current_solved(a, &s));
            }
            Step::Practice(pass) => {
                let a = &self.progress.attempts[id];
                let p = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .spanning_tree_learning
                    .as_ref()
                    .unwrap();
                let rows = p.checks(l);
                assert!(p.checked);
                if *pass {
                    assert!(rows.iter().all(|r| r.0), "{id} {rows:?}");
                } else {
                    assert!(rows[0].0);
                    assert_eq!(
                        rows[1..].iter().map(|r| r.0).collect::<Vec<_>>(),
                        l.try_change.expected
                    );
                }
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, &s));
            }
            Step::Current(pass) => {
                let a = &self.progress.attempts[id];
                let rows = progress::current_spanning_tree_checks(a, &s);
                assert_eq!(rows.iter().all(|r| r.0), *pass, "{id} {rows:?}");
                assert_eq!(progress::current_solved(a, &s), *pass);
                assert_eq!(self.metrics.completion, *pass);
            }
            Step::Partial => {
                let a = &self.progress.attempts[id];
                let rows = progress::current_spanning_tree_checks(a, &s);
                assert!(
                    rows.iter().any(|r| r.0) && rows.iter().any(|r| !r.0),
                    "{id}: {rows:?}"
                );
                assert!(!progress::current_solved(a, &s) && !self.metrics.completion);
            }
            Step::RecordState => {
                let v = serde_json::to_value(&self.progress.attempts[id].state).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("stp-lessons-state"), v));
            }
            Step::Rejected => {
                let v = ctx
                    .data(|d| d.get_temp::<Value>(egui::Id::new("stp-lessons-state")))
                    .unwrap();
                let a = &self.progress.attempts[id];
                assert_eq!(serde_json::to_value(&a.state).unwrap(), v);
                assert_eq!(draft(a, false).bridges["core"].priority, RAW_ASSIGNMENT);
                assert!(draft(a, false).notice.is_some());
                assert!(!progress::current_solved(a, &s));
            }
            Step::RecordReset => {
                let record: Record = (
                    serde_json::to_value(&self.progress.attempts[id]).unwrap(),
                    self.progress
                        .attempts
                        .iter()
                        .filter(|(k, _)| k.as_str() != id)
                        .map(|(k, a)| (k.clone(), serde_json::to_value(a).unwrap()))
                        .collect(),
                    self.progress.favorites.clone(),
                );
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("stp-lessons-reset"), record));
            }
            Step::CancelExact => {
                let before = ctx
                    .data(|d| d.get_temp::<Record>(egui::Id::new("stp-lessons-reset")))
                    .unwrap();
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[id]).unwrap(),
                    before.0
                );
                assert!(!self.reset_pending);
            }
            Step::ResetExact => {
                let a = &self.progress.attempts[id];
                assert_eq!(a.state, s.initial);
                let p = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .spanning_tree_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(p.simulation, l.fixture);
                assert!(p.draft.matches(&p.simulation));
                assert!(draft(a, false).matches(simulation(a, false)));
                assert!(!progress::current_solved(a, &s));
                let before = ctx
                    .data(|d| d.get_temp::<Record>(egui::Id::new("stp-lessons-reset")))
                    .unwrap();
                for (k, v) in before.1 {
                    assert_eq!(
                        serde_json::to_value(&self.progress.attempts[&k]).unwrap(),
                        v
                    );
                }
                assert_eq!(self.progress.favorites, before.2);
            }
            Step::RecordDrafts => {
                let a = &self.progress.attempts[id];
                assert_eq!(draft(a, false).bridges["core"].priority, RAW_ASSIGNMENT);
                assert_eq!(draft(a, true).bridges["sample-core"].priority, RAW_EXAMPLE);
                assert!(!progress::current_solved(a, &s));
                let value = serde_json::to_value(a).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("stp-lessons-resume"), value));
            }
            Step::SaveReadback => {
                self.save();
                assert!(self.storage_error.is_none());
                let raw = progress::read_bounded(&self.path, 16_000_000).unwrap();
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                let portable = serde_json::to_value(&self.progress).unwrap();
                assert_eq!(serde_json::to_value(&loaded).unwrap(), portable);
                assert_eq!(
                    serde_json::to_value(&loaded.attempts).unwrap(),
                    serde_json::to_value(&self.progress.attempts).unwrap()
                );
                let mut expected = portable;
                expected["migration_receipts"] =
                    serde_json::to_value(&loaded.migration_receipts).unwrap();
                assert_eq!(serde_json::from_slice::<Value>(&raw).unwrap(), expected);
                assert_eq!(progress::read_bounded(&self.path, 16_000_000).unwrap(), raw);
                self.progress = loaded;
            }
            Step::Filter => {
                assert!(self.selected.is_none());
                self.filter = Filter::default();
                self.filter.search = s.title.clone();
                self.filter.max_minutes = 120;
                self.page = 0;
            }
            Step::Resume => {
                if self.stp_lesson_clicked(ctx) {
                    let done = self.click_smoke_control(ctx, None);
                    assert_eq!(self.selected, Some(i));
                    return (done, None);
                }
                assert!(self.selected.is_none());
                let matches: Vec<_> = self
                    .scenarios
                    .iter()
                    .filter(|s| {
                        catalog::matches(
                            s,
                            &self.filter,
                            self.progress.attempts.contains_key(&s.id),
                            self.progress
                                .attempts
                                .get(&s.id)
                                .is_some_and(|a| progress::current_solved(a, s)),
                            self.progress.favorites.contains(&s.id),
                        )
                    })
                    .collect();
                assert_eq!(matches.len(), 1);
                assert_eq!(matches[0].id, id);
                assert_eq!(self.metrics.card_actions.len(), 1);
                let r = self.metrics.card_actions[0].0;
                if !self.stp_lesson_reveal(ctx, r) || !self.click_smoke_control(ctx, Some(r)) {
                    return (false, None);
                }
            }
            Step::ResumeExact => {
                let mut expected = ctx
                    .data(|d| d.get_temp::<Value>(egui::Id::new("stp-lessons-resume")))
                    .unwrap();
                assert!(expected["visited"].as_u64().unwrap() < self.progress.clock);
                expected["visited"] = serde_json::json!(self.progress.clock);
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[id]).unwrap(),
                    expected
                );
                assert_eq!(self.spanning_tree_view(i), teaching::View::Example);
                assert!(self.progress.favorites.contains(id));
            }
            Step::Theme(light) => {
                self.scale = if *light { 1.5 } else { 2. };
                self.theme_id = if *light { "dlms-light" } else { "dlms-dark" }.into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(*light));
                if !light {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(946., 929.)));
                }
                self.lesson_scroll_reset = true;
            }
            Step::Geometry => {
                let rect = ctx.content_rect();
                let zoom = ctx.zoom_factor();
                let ppp = ctx.pixels_per_point();
                assert!(rect.is_finite() && ppp.is_finite() && ppp > 0.);
                let sample = (
                    rect.width().to_bits(),
                    rect.height().to_bits(),
                    zoom.to_bits(),
                    ppp.to_bits(),
                );
                let key = egui::Id::new(("stp-lessons-geometry", stage));
                let old = ctx.data(|d| d.get_temp::<(u8, (u32, u32, u32, u32))>(key));
                let tries = old.map_or(1, |v| v.0 + 1);
                assert!(tries <= 20, "bounded native STP geometry");
                ctx.data_mut(|d| d.insert_temp(key, (tries, sample)));
                if (zoom - self.scale).abs() > 0.01 || old.is_none_or(|v| v.1 != sample) {
                    ctx.request_repaint();
                    return (false, None);
                }
                println!(
                    "STP lesson {id} actual content={rect:?}, ppp={ppp}, zoom={zoom}; requested size is not observed geometry"
                );
                let clip = self.metrics.main_viewport.unwrap().intersect(rect);
                for (_, r, _) in &self.metrics.stp_fields {
                    assert!(
                        r.left() >= clip.left() - 1. / ppp && r.right() <= clip.right() + 1. / ppp,
                        "horizontal field bounds"
                    );
                }
            }
            Step::Challenge => {
                assert!(self.progress.attempts[id].challenge);
                assert!(
                    self.metrics.lesson_tabs.is_empty()
                        && self.metrics.insert.is_none()
                        && !self.help_open
                );
                assert!(progress::current_solved(&self.progress.attempts[id], &s));
                assert!(self.metrics.completion);
            }
            Step::Capture(name, target) => {
                let r = match target {
                    Shot::Heading => self.metrics.stp_heading,
                    Shot::Result => self.metrics.stp_result,
                    Shot::Field(k) => {
                        let (_, r, id) = self
                            .metrics
                            .stp_fields
                            .iter()
                            .find(|v| v.0 == *k)
                            .expect("measured selected field");
                        if name.contains("focused-field") {
                            assert!(ctx.memory(|m| m.has_focus(*id)));
                        }
                        Some(*r)
                    }
                }
                .expect("actual existing widget capture target");
                if !self.stp_lesson_reveal(ctx, r) {
                    return (false, None);
                }
                return (true, Some(name.clone()));
            }
            Step::Library => {
                assert!(self.selected.is_none());
                self.filter = Filter::default();
                self.filter.unit = UNIT.into();
                self.filter.max_minutes = 120;
                self.page = 0;
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
            Step::Finish => {
                assert_eq!(self.metrics.card_actions.len(), 4);
                assert_eq!(self.filter.unit, UNIT);
                assert!(IDS.iter().all(|id| {
                    let s = self.scenarios.iter().find(|s| s.id == *id).unwrap();
                    progress::current_solved(&self.progress.attempts[*id], s)
                }));
                println!(
                    "PASS normal four STP lessons: actual isolated Fill-only/Apply/check/experiment/repair, current wrong/partial/reference/alternate/invalidation, real priority/cost typing150Light200Dark, exact both-draft save/readback/Continue/Favorite/resetCancelConfirm and Challenge. Reference-driven fixed synthetic inputs, no solved State publication, BPDU/convergence/packet/novice/cold-process/platform claim."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return (false, None);
            }
        }
        (true, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn opened() -> (egui::Context, LabApp) {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context(&ctx, true, false);
        a.smoke = false;
        // Trusted private headless setup only. Runtime never appends these scenarios.
        let labs = pack::load(include_str!("../../../scenarios/spanning-tree-v1.json")).unwrap();
        assert_eq!(labs.len(), 4);
        for s in labs {
            if let Some(old) = a.scenarios.iter().find(|old| old.id == s.id) {
                assert_eq!(
                    serde_json::to_value(old).unwrap(),
                    serde_json::to_value(&s).unwrap()
                );
            } else {
                a.scenarios.push(s);
            }
        }
        if let Some(u) = a.units.iter().find(|u| u.id == UNIT) {
            assert_eq!(u.labs, IDS);
        } else {
            a.units.push(course::Unit {
                id: UNIT.into(),
                track: "CCNA".into(),
                title: "Spanning-tree review setup".into(),
                summary: "Headless trusted source, not normal registration evidence".into(),
                prerequisites: vec![],
                labs: IDS.into_iter().map(str::to_string).collect(),
            });
        }
        a.smoke_stage = FIRST;
        (ctx, a)
    }
    fn frame(ctx: &egui::Context, a: &mut LabApp, drive: bool, router: bool) -> Option<String> {
        let events = std::mem::take(&mut a.pending_events);
        let modifiers = events
            .iter()
            .rev()
            .find_map(|e| {
                if let egui::Event::Key { modifiers, .. } = e {
                    Some(*modifiers)
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let key = egui::Id::new("normal-stp-test-frame");
        let n = ctx.data(|d| d.get_temp::<u64>(key)).unwrap_or(0) + 1;
        ctx.data_mut(|d| d.insert_temp(key, n));
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(946. / 1.15 / a.scale, 929. / 1.15 / a.scale),
            )),
            time: Some(n as f64 / 20.),
            events,
            modifiers,
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(1.15);
        let mut shot = None;
        let _ = ctx.run(input, |ctx| {
            a.render(ctx);
            a.save();
            if drive {
                a.started = Instant::now() - std::time::Duration::from_secs(1);
                if router {
                    a.smoke_tick(ctx);
                } else {
                    let (next, capture) = a.spanning_tree_lessons_native(ctx, a.smoke_stage);
                    if next {
                        a.smoke_stage += 1;
                    }
                    shot = capture;
                }
            }
        });
        shot
    }
    fn until(ctx: &egui::Context, a: &mut LabApp, target: usize, limit: usize) -> Vec<String> {
        let mut shots = vec![];
        for n in 0..limit {
            if a.smoke_stage == target {
                return shots;
            }
            assert!(a.smoke_stage < target);
            if let Some(s) = frame(ctx, a, n % 4 == 0, false) {
                shots.push(s);
            }
        }
        panic!("normal STP phase {} failed to reach{target}", a.smoke_stage);
    }
    #[test]
    fn stp_lessons_native_plan_closed_normal_ids_gui_edits_and_twelve_shots_each() {
        let (_, a) = opened();
        let p = plan(&a.scenarios);
        assert!(p.len() < END - FIRST);
        assert_eq!(p.iter().filter(|s| matches!(s, Step::Select(_))).count(), 4);
        assert_eq!(
            p.iter().filter(|s| matches!(s, Step::Capture(..))).count(),
            48
        );
        assert_eq!(
            p.iter().filter(|s| matches!(s, Step::InsertOnly)).count(),
            4
        );
        assert_eq!(
            p.iter().filter(|s| matches!(s, Step::ResumeExact)).count(),
            4
        );
        assert_eq!(
            p.iter().filter(|s| matches!(s, Step::ResetExact)).count(),
            12
        );
        assert_eq!(p.iter().filter(|s| matches!(s, Step::Rejected)).count(), 4);
        assert!(p.iter().all(|s| !matches!(s,Step::Field(_,v)if v.len()>64)));
        assert!(matches!(p.last(), Some(Step::Finish)));
        let production = include_str!("spanning_tree_lessons.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        for forbidden in [
            "engine::execute",
            ".state =",
            ".fill_example(",
            "scenarios.push",
            ".configuration =",
        ] {
            assert!(!production.contains(forbidden), "{forbidden}");
        }
    }
    #[test]
    fn stp_lessons_native_actual_smoketick_open_geometry_and_missing_unit_rejection() {
        let (ctx, mut a) = opened();
        a.smoke = true;
        frame(&ctx, &mut a, true, true);
        assert_eq!(a.smoke_stage, FIRST + 1);
        assert_eq!(
            a.spanning_tree_view(a.selected.unwrap()),
            teaching::View::Introduction
        );
        for _ in 0..5 {
            frame(&ctx, &mut a, true, false);
            if a.smoke_stage > FIRST + 1 {
                break;
            }
        }
        assert_eq!(a.smoke_stage, FIRST + 2);
        let (ctx, mut bad) = opened();
        bad.units.retain(|u| u.id != UNIT);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || bad.spanning_tree_lessons_native(&ctx, FIRST)
            ))
            .is_err()
        );
    }
    #[test]
    fn stp_lessons_native_real_fill_apply_check_idle_scroll_and_practice_repair() {
        let (ctx, mut a) = opened();
        let p = plan(&a.scenarios);
        let target = FIRST
            + p.iter()
                .position(|s| matches!(s,Step::Capture(n,_)if n=="stp-lesson-0-assignment"))
                .unwrap();
        let shots = until(&ctx, &mut a, target, 4200);
        assert!(shots.contains(&"stp-lesson-0-example-fill-only".into()));
        assert!(shots.contains(&"stp-lesson-0-example-experiment".into()));
        let s = a.scenarios.iter().find(|s| s.id == IDS[0]).unwrap();
        let l = a.progress.attempts[IDS[0]]
            .workspace
            .as_ref()
            .unwrap()
            .spanning_tree_learning
            .as_ref()
            .unwrap();
        assert!(
            l.checks(s.spanning_tree_teaching.as_ref().unwrap())
                .iter()
                .all(|r| r.0)
        );
        assert_eq!(a.progress.attempts[IDS[0]].state, s.initial);
        // Revisit the checked example through its actual tab, then stop the controller
        // and render idle frames: feedback visibility must not keep scrolling to itself.
        let mut switched = false;
        for _ in 0..60 {
            if a.stp_lesson_click(&ctx, &Target::Example) {
                switched = true;
                break;
            }
            for _ in 0..4 {
                frame(&ctx, &mut a, false, false);
            }
        }
        assert!(switched);
        assert_eq!(
            a.spanning_tree_view(a.selected.unwrap()),
            teaching::View::Example
        );
        for _ in 0..16 {
            frame(&ctx, &mut a, false, false);
        }
        let mut visible = false;
        for _ in 0..20 {
            let r = a.metrics.stp_fields[0].1;
            if a.stp_lesson_reveal(&ctx, r) {
                visible = true;
                break;
            }
            for _ in 0..8 {
                frame(&ctx, &mut a, false, false);
            }
        }
        assert!(visible);
        for _ in 0..16 {
            frame(&ctx, &mut a, false, false);
        }
        assert!(
            a.progress.attempts[IDS[0]]
                .workspace
                .as_ref()
                .unwrap()
                .spanning_tree_learning
                .as_ref()
                .unwrap()
                .checked
        );
        assert!(smoke_control_visible(
            a.metrics.stp_fields[0].1,
            a.metrics
                .main_viewport
                .unwrap()
                .intersect(ctx.content_rect()),
            ctx.pixels_per_point()
        ));
    }
    #[test]
    fn stp_lessons_native_all_four_real_autosaved_flows_resume_reset_and_repeated_finish() {
        let (ctx, mut a) = opened();
        let p = plan(&a.scenarios);
        let end = FIRST + p.len() - 1;
        let shots = until(&ctx, &mut a, end, 30000);
        assert_eq!(shots.len(), 48);
        for n in 0..4 {
            for name in [
                "introduction",
                "example-fill-only",
                "example-complete",
                "example-experiment",
                "assignment",
                "partial",
                "reference",
                "alternate",
                "light150-focused-field",
                "dark200-focused-field",
                "resumed-independent-drafts",
                "challenge",
            ] {
                assert!(shots.contains(&format!("stp-lesson-{n}-{name}")));
            }
        }
        let loaded = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&loaded.attempts).unwrap(),
            serde_json::to_value(&a.progress.attempts).unwrap()
        );
        assert!(IDS.iter().all(|id| loaded.favorites.contains(*id)));
        a.smoke = true;
        for _ in 0..2 {
            frame(&ctx, &mut a, true, true);
            assert_eq!(a.smoke_stage, end);
            assert!(!a.capture_pending && a.pending_events.is_empty());
        }
    }
}
