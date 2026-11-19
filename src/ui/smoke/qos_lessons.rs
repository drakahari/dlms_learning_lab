//! Fixed authored native inputs. Policies supply widget values, never installed State.
use super::*;
use dlms_learning_lab::{qos as core, qos_workspace as work};
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Click(String),
    Number(String, u8),
    Example(bool),
    Solved(bool),
    Save,
    Fresh,
    Shot(String),
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    frames: usize,
    done: bool,
}
fn policy(out: &mut Vec<Op>, from: &core::Policy, to: &core::Policy) {
    for (flow, mark) in &to.marks {
        match mark {
            core::Mark::Keep => {
                if !matches!(from.marks[flow], core::Mark::Keep) {
                    out.push(Op::Click(format!("mark:{flow}:mode")));
                    out.push(Op::Click(format!("mark:{flow}:keep")));
                }
            }
            core::Mark::Set(code) => {
                if matches!(from.marks[flow], core::Mark::Keep) {
                    out.push(Op::Click(format!("mark:{flow}:mode")));
                    out.push(Op::Click(format!("mark:{flow}:rewrite")));
                }
                out.push(Op::Number(format!("mark:{flow}:code"), *code));
            }
        }
    }
    if from.scheduler != to.scheduler {
        out.push(Op::Click("scheduler".into()));
        out.push(Op::Click(
            if to.scheduler == core::Scheduler::Fifo {
                "fifo"
            } else {
                "priority"
            }
            .into(),
        ));
    }
    if to.scheduler == core::Scheduler::StrictPriority {
        assert_eq!(to.priority_dscp.len(), 1);
        if from.priority_dscp.is_empty() {
            out.push(Op::Click("add-high".into()));
        }
        out.push(Op::Number(
            "high-code:0".into(),
            *to.priority_dscp.first().unwrap(),
        ));
        out.push(Op::Number(
            "high-places".into(),
            to.high_buffer.try_into().unwrap(),
        ));
        out.push(Op::Number(
            "standard-places".into(),
            to.standard_buffer.try_into().unwrap(),
        ));
    }
}
fn break_field(d: &work::Definition, practice: bool) -> (String, u8, u8) {
    let t = if practice { &d.example } else { &d.initial };
    let code = if practice {
        *d.example_worked.priority_dscp.first().unwrap_or(&12)
    } else {
        46
    };
    if t.streams.len() == 2 {
        ("high-code:0".into(), code.wrapping_add(1), code)
    } else if t
        .requirements
        .iter()
        .any(|r| matches!(r, work::Requirement::AvoidPriority { .. }))
    {
        let flow = &t
            .streams
            .iter()
            .find(|s| s.id.ends_with("-guest"))
            .unwrap()
            .id;
        (format!("mark:{flow}:code"), code, 0)
    } else {
        let flow = &t
            .streams
            .iter()
            .find(|s| s.id.ends_with("-interactive"))
            .unwrap()
            .id;
        let good = if practice { 12 } else { 46 };
        (format!("mark:{flow}:code"), 0, good)
    }
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut p = vec![];
    let mut n = 0;
    for s in labs.iter().filter(|s| s.id.starts_with("ccna-qos-")) {
        n += 1;
        let engine::Goal::Qos { definition: d } = &s.goal else {
            unreachable!()
        };
        let wanted = core::Policy::from_json(
            s.solution[0].strip_prefix(work::PREFIX).unwrap(),
            &d.initial.workloads[0].fixture,
        )
        .unwrap();
        let (ef, bad, _) = break_field(d, true);
        let (af, broken, repaired) = break_field(d, false);
        p.extend([
            Op::Open(s.id.clone()),
            Op::Click("example".into()),
            Op::Click("run".into()),
            Op::Example(false),
            Op::Click("insert".into()),
            Op::Example(false),
            Op::Click("run".into()),
            Op::Example(true),
            Op::Shot(format!("qos-{}-practice-passed", s.id)),
            Op::Number(ef, bad),
            Op::Example(false),
            Op::Click("run".into()),
            Op::Example(false),
            Op::Click("insert".into()),
            Op::Example(false),
            Op::Click("run".into()),
            Op::Example(true),
            Op::Click("assignment".into()),
            Op::Click("run".into()),
            Op::Solved(false),
            Op::Shot(format!("qos-{}-starter-feedback", s.id)),
        ]);
        policy(&mut p, &d.initial.starter, &wanted);
        p.extend([
            Op::Click("run".into()),
            Op::Solved(true),
            Op::Save,
            Op::Shot(format!("qos-{}-passed", s.id)),
            Op::Number(af.clone(), broken),
            Op::Solved(false),
            Op::Click("run".into()),
            Op::Solved(false),
            Op::Shot(format!("qos-{}-damage-feedback", s.id)),
            Op::Number(af, repaired),
            Op::Click("run".into()),
            Op::Solved(true),
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Fresh,
        ]);
    }
    assert_eq!(n, 3);
    p
}
fn value(s: &work::Session, key: &str) -> Option<u8> {
    let p = if s.view == teaching::View::Example {
        &s.practice.draft
    } else {
        &s.assignment.draft
    };
    if key == "high-places" {
        return p.high_buffer.try_into().ok();
    }
    if key == "standard-places" {
        return p.standard_buffer.try_into().ok();
    }
    if let Some(index) = key.strip_prefix("high-code:") {
        return p.priority_dscp.iter().nth(index.parse().ok()?).copied();
    }
    let flow = key.strip_prefix("mark:")?.strip_suffix(":code")?;
    match p.marks.get(flow)? {
        core::Mark::Set(n) => Some(*n),
        core::Mark::Keep => None,
    }
}
impl LabApp {
    pub(in crate::ui) fn qos_lessons_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("qos-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: route(&self.scenarios),
                ..Default::default()
            });
        if r.done {
            self.teaching_preview_tick(ctx);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "QOS_NATIVE_FLOW_COMPLETE: all3 separate examples/change/retry; assignments/partial/solution/edit invalidation/damage/retry/save-read/reset; no host packets, network or service operations"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 140000 + r.at;
        let op = r.ops[r.at].clone();
        r.frames += 1;
        if r.frames == 1 {
            println!("QOS_NATIVE_STEP {} {:?}", r.at, op);
        }
        assert!(
            r.frames < 1200,
            "QoS native step {} phase{} {:?}",
            r.at,
            r.phase,
            op
        );
        let complete = match op {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                true
            }
            Op::Click(name) => {
                let rect = if name == "reset-header" {
                    self.metrics.reset
                } else if name == "confirm-reset" {
                    self.metrics.confirm_reset
                } else {
                    self.metrics
                        .qos
                        .controls
                        .iter()
                        .find(|r| r.0 == name)
                        .map(|r| r.1)
                };
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Number(name, wanted) => {
                let field = self
                    .metrics
                    .qos
                    .fields
                    .iter()
                    .find(|f| f.0 == name)
                    .map(|f| (f.1, f.2));
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, field.map(|f| f.0)) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    let modifiers = egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    };
                    for pressed in [true, false] {
                        self.pending_events.push(egui::Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed,
                            repeat: false,
                            modifiers,
                        });
                    }
                    self.pending_events
                        .push(egui::Event::Paste(wanted.to_string()));
                    r.phase = 2;
                    false
                } else if r.phase == 2 {
                    for pressed in [true, false] {
                        self.pending_events.push(egui::Event::Key {
                            key: egui::Key::Enter,
                            physical_key: None,
                            pressed,
                            repeat: false,
                            modifiers: Default::default(),
                        });
                    }
                    r.phase = 3;
                    false
                } else {
                    let id = &self.scenarios[self.selected.unwrap()].id;
                    let engine::State::Qos { simulation } = &self.progress.attempts[id].state
                    else {
                        unreachable!()
                    };
                    if value(simulation, &name) == Some(wanted) {
                        true
                    } else {
                        assert!(
                            r.frames < 100,
                            "Numeric edit {name}: wanted{wanted},actual {:?}; focused {:?}",
                            value(simulation, &name),
                            ctx.memory(|m| m.focused())
                        );
                        false
                    }
                }
            }
            Op::Example(expected) => {
                let s = &self.scenarios[self.selected.unwrap()];
                let (engine::Goal::Qos { definition }, engine::State::Qos { simulation }) =
                    (&s.goal, &self.progress.attempts[&s.id].state)
                else {
                    unreachable!()
                };
                assert_eq!(simulation.practice.passed(&definition.example), expected);
                assert_eq!(
                    simulation.assignment,
                    work::Session::new(definition).unwrap().assignment
                );
                assert!(!progress::current_solved(&self.progress.attempts[&s.id], s));
                true
            }
            Op::Solved(expected) => {
                let s = &self.scenarios[self.selected.unwrap()];
                assert_eq!(
                    progress::current_solved(&self.progress.attempts[&s.id], s),
                    expected,
                    "{} notice={:?}",
                    s.id,
                    self.notice
                );
                true
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none(), "{:?}", self.storage_error);
                let got = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(got).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Fresh => {
                let s = &self.scenarios[self.selected.unwrap()];
                assert_eq!(self.progress.attempts[&s.id].state, s.initial);
                assert!(!progress::current_solved(&self.progress.attempts[&s.id], s));
                true
            }
            Op::Shot(name) => {
                if r.phase == 0 {
                    if self.reveal_smoke_control(
                        ctx,
                        self.metrics.qos.result.expect("result summary").2,
                    ) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name,
                    )));
                    r.phase = 2;
                    false
                } else {
                    !self.capture_pending
                }
            }
        };
        if complete {
            r.at += 1;
            r.frames = 0;
            r.phase = 0;
            self.started = Instant::now();
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn qos_native_capture_phase_uses_context_without_reentry() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "ccna-qos-mark-boundary")
            .unwrap();
        a.launch(i);
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("qos-native-route"),
                Route {
                    ops: vec![Op::Shot("fixed-qos-capture".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| a.qos_lessons_tick(ctx));
        assert!(a.capture_pending);
    }
}
