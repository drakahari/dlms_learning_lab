//! Normal registered Ethernet lessons: actual native inputs, never reference-State publication.
use super::mac_learning::{Control as Base, ctrl_a};
use super::*;
use dlms_learning_lab::{engine::State, mac_learning_workspace as work};
use serde::Deserialize;
use std::collections::BTreeMap;
const START: usize = 18000;
const IDS: [&str; 4] = [
    "ccna-mac-learn-paths",
    "ccna-mac-stale-location",
    "ccna-mac-source-refresh",
    "ccna-mac-handover",
];
#[derive(Clone, Deserialize)]
struct Review {
    id: String,
    alternates: Vec<Vec<String>>,
    partial: Vec<String>,
    wrong: Vec<Wrong>,
    experiment_actions: Vec<work::Action>,
    repair_actions: Vec<work::Action>,
}
#[derive(Clone, Deserialize)]
struct Wrong {
    actions: Vec<String>,
}
#[derive(Clone, Debug, PartialEq)]
enum Target {
    Base(Base),
    Back,
    ExampleReset,
    Lifetime,
    ClearSwitch,
    ClearChoice(String),
    ClearVlan,
    Switch,
    SwitchChoice(String),
    Field(String, String, &'static str),
}
#[derive(Clone, Debug, PartialEq)]
enum Step {
    Select(usize),
    Wait,
    Click(Target),
    OpenPorts,
    OpenLifetime,
    OpenTools,
    Type(String),
    AddVlan(String),
    RestoreLifetime,
    RecordExample,
    InsertOnly,
    ExamplePass,
    Experiment,
    Current(bool),
    Partial,
    Rejected,
    RecordApplied,
    SelectedSource(String),
    SelectedAction(usize),
    SelectedSwitch(String),
    SelectedClear(String),
    Pending,
    SaveReadback,
    Record,
    CancelExact,
    ResetExact,
    Challenge,
    Library,
    Shot(String, bool),
    Theme(usize),
    Scale(f32),
    Finish,
}
fn click(p: &mut Vec<Step>, b: Base) {
    p.push(Step::Click(Target::Base(b)));
}
fn type_field(p: &mut Vec<Step>, t: Target, value: String) {
    p.extend([Step::Click(t), Step::Type(value)]);
}
fn decoded(inputs: &[String]) -> Vec<work::Action> {
    inputs
        .iter()
        .map(|x| {
            serde_json::from_str(x.strip_prefix("@mac ").expect("author metadata only")).unwrap()
        })
        .collect()
}
fn actions(p: &mut Vec<Step>, actions: &[work::Action]) {
    for action in actions {
        match action {
            work::Action::Send {
                source,
                destination,
            } => {
                click(p, Base::SourcePicker);
                click(p, Base::Source(source.clone()));
                p.push(Step::SelectedSource(source.clone()));
                type_field(p, Target::Base(Base::Destination), destination.text());
                click(p, Base::Send);
            }
            work::Action::Aging { seconds } => {
                p.push(Step::OpenLifetime);
                type_field(p, Target::Lifetime, seconds.to_string());
                click(p, Base::Apply);
            }
            work::Action::Advance { seconds } => {
                p.push(Step::OpenTools);
                type_field(p, Target::Base(Base::Time), seconds.to_string());
                click(p, Base::Advance);
            }
            work::Action::Clear { switch, vlan } => {
                p.push(Step::OpenTools);
                p.extend([
                    Step::Click(Target::ClearSwitch),
                    Step::Click(Target::ClearChoice(switch.clone())),
                    Step::SelectedClear(switch.clone()),
                ]);
                type_field(
                    p,
                    Target::ClearVlan,
                    vlan.map(|x| x.to_string()).unwrap_or_default(),
                );
                // No author action is executed here; this is the existing native Clear button.
                p.push(Step::Click(Target::Field(
                    String::new(),
                    String::new(),
                    "clear",
                )));
            }
            work::Action::Settings { operation } => {
                let a = operation.split_whitespace().collect::<Vec<_>>();
                let switch = a[1].to_string();
                p.extend([
                    Step::OpenPorts,
                    Step::Click(Target::Switch),
                    Step::Click(Target::SwitchChoice(switch.clone())),
                    Step::SelectedSwitch(switch.clone()),
                ]);
                match a.as_slice() {
                    ["access", _, port, value] => type_field(
                        p,
                        Target::Field(switch, (*port).into(), "access"),
                        (*value).into(),
                    ),
                    ["trunk", _, port, native, allowed] => {
                        type_field(
                            p,
                            Target::Field(switch.clone(), (*port).into(), "native"),
                            (*native).into(),
                        );
                        type_field(
                            p,
                            Target::Field(switch, (*port).into(), "allowed"),
                            (*allowed).into(),
                        );
                    }
                    ["vlan", _, value, "present"] => {
                        p.push(Step::Click(Target::Field(switch, String::new(), "vlans")));
                        p.push(Step::AddVlan((*value).into()));
                    }
                    _ => panic!("normal MAC source plan has an unsupported native edit"),
                }
                click(p, Base::Apply);
            }
        }
        p.push(Step::Wait);
    }
}
fn check(p: &mut Vec<Step>, assertion: Step, name: String) {
    click(p, Base::Check);
    p.extend([Step::Wait, assertion, Step::Shot(name, true)]);
}
fn reset(p: &mut Vec<Step>) {
    click(p, Base::Reset);
    click(p, Base::Confirm);
    p.extend([Step::Wait, Step::ResetExact]);
    click(p, Base::Assignment);
    p.push(Step::Wait);
}
fn plan(scenarios: &[Scenario]) -> Vec<Step> {
    plan_for(scenarios, &[0, 1, 2, 3])
}
fn plan_for(scenarios: &[Scenario], scope: &[usize]) -> Vec<Step> {
    assert!(
        scope == [0, 1, 2, 3] || scope == [3],
        "fixed developer scope only"
    );
    let review: Vec<Review> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/mac-content-review.json"
    ))
    .unwrap();
    let mut p = Vec::new();
    for (n, id) in IDS.iter().enumerate() {
        if !scope.contains(&n) {
            continue;
        }
        let s = scenarios
            .iter()
            .find(|s| s.id == *id)
            .expect("normal registered lesson");
        let row = review.iter().find(|r| r.id == *id).unwrap();
        let l = s.mac_teaching.as_ref().unwrap();
        let shot = |suffix: &str| format!("ccna-mac-{n}-{suffix}");
        p.extend([
            Step::Select(n),
            Step::Wait,
            Step::Shot(shot("introduction"), false),
        ]);
        click(&mut p, Base::Try);
        p.push(Step::RecordExample);
        click(&mut p, Base::Try); // Fill button in Example view.
        p.push(Step::InsertOnly);
        p.push(Step::Click(Target::ExampleReset));
        p.push(Step::Wait);
        for (index, action) in l.example_solution.iter().enumerate() {
            click(&mut p, Base::ActionPicker);
            click(&mut p, Base::Action(index));
            p.push(Step::SelectedAction(index));
            p.push(Step::RecordExample);
            click(&mut p, Base::Insert);
            p.push(Step::InsertOnly);
            match action {
                work::Action::Settings { .. } | work::Action::Aging { .. } => {
                    click(&mut p, Base::Apply)
                }
                work::Action::Send { .. } => click(&mut p, Base::Send),
                work::Action::Clear { .. } => {
                    p.push(Step::OpenTools);
                    p.push(Step::Click(Target::Field(
                        String::new(),
                        String::new(),
                        "clear",
                    )));
                }
                work::Action::Advance { .. } => {
                    p.push(Step::OpenTools);
                    click(&mut p, Base::Advance);
                }
            }
            p.push(Step::Wait);
        }
        check(&mut p, Step::ExamplePass, shot("practice-complete"));
        actions(&mut p, &row.experiment_actions);
        check(&mut p, Step::Experiment, shot("practice-experiment"));
        actions(&mut p, &row.repair_actions);
        check(&mut p, Step::ExamplePass, shot("practice-repaired"));
        click(&mut p, Base::Assignment);
        p.extend([Step::Wait, Step::Shot(shot("assignment"), false)]);
        p.push(Step::RecordApplied);
        type_field(&mut p, Target::Base(Base::Destination), "not-a-mac".into());
        click(&mut p, Base::Send);
        check(&mut p, Step::Rejected, shot("invalid-input"));
        actions(&mut p, &decoded(&row.wrong[0].actions));
        check(&mut p, Step::Current(false), shot("misconception"));
        reset(&mut p);
        actions(&mut p, &decoded(&row.partial));
        check(&mut p, Step::Partial, shot("partial"));
        reset(&mut p);
        actions(&mut p, &decoded(&s.solution));
        check(&mut p, Step::Current(true), shot("complete"));
        p.push(Step::SaveReadback);
        p.extend([
            Step::Theme(n),
            Step::Scale(if n % 2 == 0 { 1.5 } else { 2. }),
        ]);
        p.push(Step::RecordApplied);
        type_field(&mut p, Target::Base(Base::Destination), "not-a-mac".into());
        click(&mut p, Base::Send);
        check(&mut p, Step::Rejected, shot("scaled-typing-error"));
        p.push(Step::OpenLifetime);
        type_field(&mut p, Target::Lifetime, "oops".into());
        p.extend([
            Step::Wait,
            Step::Pending,
            Step::Shot(shot("scaled-pending"), false),
        ]);
        p.extend([Step::Click(Target::Lifetime), Step::RestoreLifetime]);
        click(&mut p, Base::Apply);
        check(&mut p, Step::Current(true), shot("scaled-repaired"));
        p.push(Step::SaveReadback);
        p.push(Step::Record);
        click(&mut p, Base::Reset);
        click(&mut p, Base::Cancel);
        p.push(Step::CancelExact);
        reset(&mut p);
        actions(&mut p, &decoded(&row.alternates[0]));
        check(&mut p, Step::Current(true), shot("alternate"));
        p.push(Step::SaveReadback);
        click(&mut p, Base::Challenge);
        p.extend([
            Step::Wait,
            Step::Challenge,
            Step::Shot(shot("challenge"), false),
        ]);
    }
    p.extend([
        Step::Click(Target::Back),
        Step::Wait,
        Step::Library,
        Step::Shot("mac-unit-library".into(), false),
        Step::Finish,
    ]);
    assert!(p.len() < 2000);
    p
}
impl LabApp {
    fn mac_lessons_target(&mut self, ctx: &egui::Context, t: &Target) -> bool {
        if let Target::Base(base) = t {
            return self.mac_native_click(ctx, base);
        }
        let rect = match t {
            Target::Base(_) => unreachable!(),
            Target::Back => self.metrics.back,
            Target::ExampleReset => self.metrics.example_reset,
            Target::Lifetime => self.metrics.mac_aging,
            Target::ClearSwitch => self.metrics.mac_clear_switch,
            Target::ClearChoice(name) => self
                .metrics
                .mac_clear_choices
                .iter()
                .find(|(x, _)| x == name)
                .map(|(_, r)| *r),
            Target::ClearVlan => self.metrics.mac_clear_vlan,
            Target::Switch => self.metrics.vlan_switch_picker,
            Target::SwitchChoice(name) => self
                .metrics
                .vlan_switch_choices
                .iter()
                .find(|(x, _)| x == name)
                .map(|(_, r)| *r),
            Target::Field(_, _, "clear") => self.metrics.mac_clear,
            Target::Field(switch, port, kind) => self
                .metrics
                .vlan_named_fields
                .iter()
                .find(|(s, p, k, _)| s == switch && p == port && k == kind)
                .map(|(_, _, _, r)| *r),
        };
        if matches!(t, Target::SwitchChoice(_) | Target::ClearChoice(_)) {
            let clip = if matches!(t, Target::SwitchChoice(_)) {
                self.metrics.vlan_switch_popup
            } else {
                self.metrics.mac_clear_popup
            };
            self.mac_popup_choice(ctx, rect, clip)
        } else {
            self.click_visible_smoke_control(ctx, rect)
        }
    }
    pub(super) fn mac_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        let handover = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("mac-handover");
        let scope: &[usize] = if handover { &[3] } else { &[0, 1, 2, 3] };
        let steps = if handover {
            plan_for(&self.scenarios, scope)
        } else {
            plan(&self.scenarios)
        };
        let step = steps.get(stage - START).expect("bounded normal MAC stage");
        if let Step::Select(n) = step {
            assert!(self.smoke && self._smoke_profile.is_some());
            assert!(
                self.path
                    .starts_with(self._smoke_profile.as_ref().unwrap().path())
            );
            let i = self.scenarios.iter().position(|s| s.id == IDS[*n]).unwrap();
            let other = self
                .scenarios
                .iter()
                .find(|s| !IDS.contains(&s.id.as_str()))
                .unwrap()
                .clone();
            self.progress
                .attempts
                .entry(other.id.clone())
                .or_insert_with(|| Attempt::new(&other, 1));
            let others = self
                .progress
                .attempts
                .iter()
                .filter(|(id, _)| *id != IDS[*n])
                .map(|(id, a)| (id.clone(), serde_json::to_value(a).unwrap()))
                .collect::<BTreeMap<_, _>>();
            ctx.data_mut(|d| {
                d.insert_temp(egui::Id::new("mac-lessons-other"), others);
                d.insert_temp(egui::Id::new("mac-lessons-selected"), i);
            });
            self.launch(i);
            self.set_mac_view(i, teaching::View::Introduction);
            self.theme_id = "dlms-dark".into();
            self.scale = 1.;
            theme::apply(ctx, &self.theme_id);
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            return (true, None);
        }
        let i = ctx
            .data(|d| d.get_temp::<usize>(egui::Id::new("mac-lessons-selected")))
            .unwrap();
        let s = &self.scenarios[i];
        let id = s.id.clone();
        let clicking = ctx
            .data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", stage))))
            .is_some();
        match step {
            Step::Select(_) => unreachable!(),
            Step::Wait => {}
            Step::Click(t) => {
                if !self.mac_lessons_target(ctx, t) {
                    return (false, None);
                }
            }
            Step::OpenPorts => {
                if (self.metrics.vlan_named_fields.is_empty() || clicking)
                    && !self.mac_native_click(ctx, &Base::Settings)
                {
                    return (false, None);
                }
            }
            Step::OpenLifetime => {
                if (self.metrics.mac_aging.is_none() || clicking)
                    && !self.click_visible_smoke_control(ctx, self.metrics.mac_lifetime)
                {
                    return (false, None);
                }
            }
            Step::OpenTools => {
                if (self.metrics.mac_clear.is_none() || clicking)
                    && !self.mac_native_click(ctx, &Base::Tools)
                {
                    return (false, None);
                }
            }
            Step::Type(text) => {
                assert!(
                    ctx.memory(|m| m.focused()).is_some(),
                    "actual clicked field must own keyboard focus"
                );
                assert!(text.len() <= 384);
                self.pending_events = ctrl_a();
                self.pending_events.push(egui::Event::Text(text.clone()));
            }
            Step::AddVlan(vlan) => {
                let a = &self.progress.attempts[&id];
                let w = a.workspace.as_ref().unwrap();
                let d = if self.mac_view(i) == teaching::View::Example {
                    &w.mac_learning.as_ref().unwrap().draft
                } else {
                    w.mac_draft.as_ref().unwrap()
                };
                let mut text = d.configuration.switches[&d.configuration.selected_switch]
                    .vlans
                    .clone();
                if !text.split(',').any(|v| v.trim() == vlan) {
                    if !text.is_empty() {
                        text.push(',');
                    }
                    text.push_str(vlan);
                }
                assert!(text.len() <= 384);
                self.pending_events = ctrl_a();
                self.pending_events.push(egui::Event::Text(text));
            }
            Step::RestoreLifetime => {
                let State::MacLearning { simulation } = &self.progress.attempts[&id].state else {
                    panic!()
                };
                self.pending_events = ctrl_a();
                self.pending_events
                    .push(egui::Event::Text(simulation.aging_seconds().to_string()));
            }
            Step::RecordExample => {
                let a = &self.progress.attempts[&id];
                let l = a.workspace.as_ref().unwrap().mac_learning.as_ref().unwrap();
                ctx.data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new("mac-lessons-example"),
                        (
                            serde_json::to_value(&a.state).unwrap(),
                            serde_json::to_value(&l.simulation).unwrap(),
                        ),
                    )
                });
            }
            Step::InsertOnly => {
                let a = &self.progress.attempts[&id];
                let l = a.workspace.as_ref().unwrap().mac_learning.as_ref().unwrap();
                let before = ctx
                    .data(|d| {
                        d.get_temp::<(serde_json::Value, serde_json::Value)>(egui::Id::new(
                            "mac-lessons-example",
                        ))
                    })
                    .unwrap();
                assert_eq!(serde_json::to_value(&a.state).unwrap(), before.0);
                assert_eq!(serde_json::to_value(&l.simulation).unwrap(), before.1);
                assert!(!progress::current_solved(a, s));
            }
            Step::ExamplePass | Step::Experiment => {
                let a = &self.progress.attempts[&id];
                let l = a.workspace.as_ref().unwrap().mac_learning.as_ref().unwrap();
                let lesson = s.mac_teaching.as_ref().unwrap();
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, s));
                if matches!(step, Step::ExamplePass) {
                    let c = l.checks(lesson);
                    assert!(!c.is_empty() && c.iter().all(|c| c.0), "{} {c:?}", s.id);
                } else {
                    let c = work::checks(&lesson.fixture, &l.simulation, &lesson.try_outcomes);
                    assert!(c.iter().all(|c| c.0), "{} {c:?}", s.id);
                    assert!(l.checks(lesson).iter().any(|c| !c.0));
                }
            }
            Step::Current(ok) => {
                assert_eq!(
                    progress::current_solved(&self.progress.attempts[&id], s),
                    *ok,
                    "{} {:?}",
                    id,
                    engine::checks(&self.progress.attempts[&id].state, &s.goal)
                );
                if *ok {
                    assert!(self.metrics.completion);
                }
            }
            Step::Partial => {
                let c = engine::checks(&self.progress.attempts[&id].state, &s.goal);
                assert!(c.iter().any(|c| c.0) && c.iter().any(|c| !c.0));
                assert!(!progress::current_solved(&self.progress.attempts[&id], s));
            }
            Step::SelectedSource(expected)
            | Step::SelectedSwitch(expected)
            | Step::SelectedClear(expected) => {
                let w = self.progress.attempts[&id].workspace.as_ref().unwrap();
                let draft = if self.mac_view(i) == teaching::View::Example {
                    &w.mac_learning.as_ref().unwrap().draft
                } else {
                    w.mac_draft.as_ref().unwrap()
                };
                let actual = match step {
                    Step::SelectedSource(_) => &draft.source,
                    Step::SelectedSwitch(_) => &draft.configuration.selected_switch,
                    _ => &draft.clear_switch,
                };
                assert_eq!(
                    actual, expected,
                    "actual native popup selection must match requested action before typing/sending"
                );
            }
            Step::SelectedAction(expected) => assert_eq!(
                self.progress.attempts[&id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .mac_learning
                    .as_ref()
                    .unwrap()
                    .selected_action,
                *expected,
                "actual example action selection"
            ),
            Step::RecordApplied => {
                let value = serde_json::to_value(&self.progress.attempts[&id].state).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("mac-lessons-applied"), value));
            }
            Step::Rejected => {
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[&id].state).unwrap(),
                    ctx.data(
                        |d| d.get_temp::<serde_json::Value>(egui::Id::new("mac-lessons-applied"))
                    )
                    .unwrap()
                );
                assert!(
                    self.notice
                        .as_ref()
                        .is_some_and(|x| x.contains("hexadecimal"))
                );
            }
            Step::Pending => {
                let a = &self.progress.attempts[&id];
                assert!(engine::solved(&a.state, &s.goal));
                assert_eq!(
                    a.workspace
                        .as_ref()
                        .unwrap()
                        .mac_draft
                        .as_ref()
                        .unwrap()
                        .aging,
                    "oops"
                );
                assert!(!progress::current_solved(a, s));
            }
            Step::SaveReadback => {
                self.save();
                assert!(self.storage_error.is_none());
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(&loaded).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                let others = ctx
                    .data(|d| {
                        d.get_temp::<BTreeMap<String, serde_json::Value>>(egui::Id::new(
                            "mac-lessons-other",
                        ))
                    })
                    .unwrap();
                for (id, expected) in others {
                    assert_eq!(
                        serde_json::to_value(&self.progress.attempts[&id]).unwrap(),
                        expected
                    );
                }
            }
            Step::Record => ctx.data_mut(|d| {
                d.insert_temp(
                    egui::Id::new("mac-lessons-record"),
                    serde_json::to_value(&self.progress.attempts[&id]).unwrap(),
                )
            }),
            Step::CancelExact => assert_eq!(
                serde_json::to_value(&self.progress.attempts[&id]).unwrap(),
                ctx.data(|d| d.get_temp::<serde_json::Value>(egui::Id::new("mac-lessons-record")))
                    .unwrap()
            ),
            Step::ResetExact => {
                let a = &self.progress.attempts[&id];
                assert_eq!(a.state, s.initial);
                assert!(
                    a.workspace
                        .as_ref()
                        .unwrap()
                        .mac_draft
                        .as_ref()
                        .unwrap()
                        .matches(match &a.state {
                            State::MacLearning { simulation } => simulation,
                            _ => panic!(),
                        })
                );
                assert!(!progress::current_solved(a, s));
            }
            Step::Challenge => {
                assert!(self.progress.attempts[&id].challenge);
                assert!(
                    self.metrics.lesson_tabs.is_empty() && self.metrics.command_insert.is_none()
                );
            }
            Step::Shot(name, feedback) => {
                if *feedback
                    && !self.reveal_smoke_control(
                        ctx,
                        self.metrics.feedback.expect("actual MAC feedback"),
                    )
                {
                    return (false, None);
                }
                return (true, Some(name.clone()));
            }
            Step::Theme(n) => {
                self.theme_id = if n % 2 == 0 {
                    "dlms-light"
                } else {
                    "dlms-dark"
                }
                .into();
                self.scale = if n % 2 == 0 { 1.5 } else { 2. };
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(if n % 2 == 0 {
                    egui::vec2(1600., 1000.)
                } else {
                    egui::vec2(960., 900.)
                }));
            }
            Step::Scale(scale) => {
                if (ctx.zoom_factor() - *scale).abs() > 0.01 {
                    let count = ctx.data_mut(|d| {
                        let key = egui::Id::new(("mac-lessons-scale", stage));
                        let count = d.get_temp::<u8>(key).unwrap_or(0) + 1;
                        d.insert_temp(key, count);
                        count
                    });
                    assert!(count < 20, "text scale must settle");
                    return (false, None);
                }
            }
            Step::Library => {
                assert!(self.selected.is_none());
                let unit = self
                    .units
                    .iter()
                    .find(|u| u.id == "ethernet-learning")
                    .unwrap();
                assert_eq!(
                    unit.labs,
                    IDS.iter().map(|id| (*id).to_string()).collect::<Vec<_>>()
                );
            }
            Step::Finish => {
                assert!(scope.iter().all(|n| {
                    let id = &IDS[*n];
                    let i = self.scenarios.iter().position(|s| s.id == *id).unwrap();
                    progress::current_solved(&self.progress.attempts[*id], &self.scenarios[i])
                }));
                if handover {
                    println!(
                        "PASS scoped MAC handover lesson only: same full-task example/experiment/wrong/partial/reference/alternate/scaledediting/save/reset/Challenge inputs and37requirements; other3lessons notexecuted bythisscope."
                    );
                } else {
                    println!(
                        "PASS normal four MAC lessons: real native example Fill/Insert/Apply/Send/Check, independent experiment/repair, assignment malformed/conceptual/partial/reference/alternate, actual150light/200dark typing, freshpending invalidation and repair, exact save/readback, selectedReset Cancel/Confirm, Challenge and library. Trusted authored input targets only; no referenceState publication, coldrestart, novice or otherplatform claim."
                    );
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        (true, None)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normal_mac_lesson_plan_uses_registered_ids_finite_real_inputs_and_every_task() {
        let scenarios = pack::bundled().unwrap();
        let p = plan(&scenarios);
        assert!(p.len() < 2000);
        assert_eq!(p.iter().filter(|s| matches!(s, Step::Select(_))).count(), 4);
        assert_eq!(p.iter().filter(|s| matches!(s, Step::Finish)).count(), 1);
        assert_eq!(p.iter().filter(|s| matches!(s, Step::Theme(_))).count(), 4);
        assert!(p.iter().all(|s| !matches!(s,Step::Type(x)if x.len()>384)));
        for id in IDS {
            assert!(scenarios.iter().any(|s| s.id == id));
        }
        assert!(!scenarios.iter().any(|s| s.id == "development-mac-learning"));
    }
    #[test]
    fn mac_handover_scope_is_exactly_the_full_plan_capstone_suffix_and_truthfully_scoped() {
        let scenarios = pack::bundled().unwrap();
        let all = plan(&scenarios);
        let start = all
            .iter()
            .position(|s| matches!(s, Step::Select(3)))
            .unwrap();
        let scoped = plan_for(&scenarios, &[3]);
        assert_eq!(scoped, all[start..]);
        assert_eq!(
            scoped
                .iter()
                .filter(|s| matches!(s, Step::Select(_)))
                .count(),
            1
        );
        assert!(scoped.iter().any(|s| matches!(s, Step::Current(true))));
        assert!(scoped.len() < all.len());
    }
}
