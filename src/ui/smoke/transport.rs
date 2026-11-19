//! Fixed review inputs through actual native controls; no learner host execution.
use super::*;
use dlms_learning_lab::transport_workspace as work;
#[derive(Clone, Debug)]
enum Op {
    Open(&'static str),
    Click(&'static str),
    Input(&'static str, &'static str),
    Repair,
    Check(bool, bool),
    Unchecked(bool),
    Shot(&'static str, bool),
    Save,
    Fresh,
}
#[derive(Clone, Default)]
struct Route {
    at: usize,
    phase: u8,
    frames: usize,
    until: f64,
    done: bool,
}
fn plan() -> Vec<Op> {
    vec![
        Op::Open("ccna-transport-port-pairing"),
        Op::Shot("transport-pairing-introduction", false),
        Op::Click("example"),
        Op::Shot("transport-pairing-practice", false),
        Op::Check(true, false),
        Op::Click("fill-example"),
        Op::Unchecked(true),
        Op::Check(true, true),
        Op::Input("listener-0-port", "8090"),
        Op::Input("report-main-server", "8090"),
        Op::Input("report-retry-server", "8090"),
        Op::Check(true, true),
        Op::Input("report-retry-server", "8091"),
        Op::Check(true, false),
        Op::Shot("transport-pairing-partial-feedback", true),
        Op::Click("assignment"),
        Op::Check(false, false),
        Op::Input("listener-0-port", "8443"),
        Op::Unchecked(false),
        Op::Check(false, true),
        Op::Save,
        Op::Input("report-retry-server", "1"),
        Op::Unchecked(false),
        Op::Check(false, false),
        Op::Click("reset"),
        Op::Click("confirm"),
        Op::Fresh,
        Op::Open("ccna-transport-return-policy"),
        Op::Shot("transport-policy-introduction", false),
        Op::Click("example"),
        Op::Check(true, false),
        Op::Repair,
        Op::Unchecked(true),
        Op::Check(true, true),
        Op::Shot("transport-policy-repaired-example", true),
        Op::Click("assignment"),
        Op::Check(false, false),
        Op::Shot("transport-policy-synack-failure", true),
        Op::Repair,
        Op::Unchecked(false),
        Op::Check(false, true),
        Op::Save,
        Op::Input(
            "config-R1",
            "ip access-list extended RETURN\n 10 permit ip any any\nexit\ninterface office\n ip access-group RETURN out\nexit\n",
        ),
        Op::Unchecked(false),
        Op::Check(false, false),
        Op::Shot("transport-policy-broad-permit-feedback", true),
        Op::Save,
        Op::Click("challenge"),
        Op::Click("learn"),
    ]
}
fn current(a: &LabApp) -> (&Scenario, &work::Session) {
    let s = &a.scenarios[a.selected.unwrap()];
    let engine::State::Transport { simulation } = &a.progress.attempts[&s.id].state else {
        panic!()
    };
    (s, simulation)
}
fn scope(s: &work::Session, p: bool) -> &work::Scope {
    if p { &s.practice } else { &s.assignment }
}
impl LabApp {
    pub(in crate::ui) fn transport_tick(&mut self, ctx: &egui::Context) {
        let root = ctx.cumulative_frame_nr();
        let gate = egui::Id::new("transport-native-root");
        if ctx.data(|d| d.get_temp::<u64>(gate)) == Some(root) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(gate, root));
        let key = egui::Id::new("transport-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_default();
        let ops = plan();
        if r.done {
            self.teaching_preview_tick(ctx);
            return;
        }
        if r.at == ops.len() {
            println!(
                "TRANSPORT_NATIVE_COMPLETE: {} actual input/check/capture/save/reset actions; assisted technical verification, not novice acceptance",
                ops.len()
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        let now = ctx.input(|i| i.time);
        if now < r.until {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        self.smoke_stage = 190000 + r.at;
        let op = ops[r.at].clone();
        if r.frames == 0 {
            println!("TRANSPORT_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(r.frames < 10000, "Transport native step timed out: {op:?}");
        let done = match op {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                true
            }
            Op::Click(k) => {
                let rect = match k {
                    "reset" => self.metrics.reset,
                    "confirm" => self.metrics.confirm_reset,
                    "challenge" => self.metrics.mode_challenge,
                    "learn" => self.metrics.mode_learn,
                    _ => self
                        .metrics
                        .transport
                        .controls
                        .iter()
                        .find(|c| c.0 == k)
                        .map(|c| c.1),
                };
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Input(..) | Op::Repair => {
                let (field, text) = if let Op::Input(field, text) = op {
                    (field, text.to_owned())
                } else {
                    let (s, session) = current(self);
                    let engine::Goal::Transport { definition } = &s.goal else {
                        panic!()
                    };
                    (
                        "config-R1",
                        if session.view == teaching::View::Example {
                            &definition.worked_example
                        } else {
                            &definition.worked_assignment
                        }
                        .documents
                        .routers["R1"]
                            .clone(),
                    )
                };
                if r.phase == 0 {
                    let rect = self
                        .metrics
                        .transport
                        .fields
                        .iter()
                        .find(|f| f.0 == field)
                        .map(|f| {
                            egui::Rect::from_min_size(
                                f.1.min + egui::vec2(3., 3.),
                                egui::vec2((f.1.width() - 6.).max(1.), 22.),
                            )
                        });
                    if self.click_visible_smoke_control(ctx, rect) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    self.case_lesson_type(&text);
                    r.phase = 2;
                    false
                } else {
                    let (_, session) = current(self);
                    let current = scope(session, session.view == teaching::View::Example);
                    let actual = match field {
                        "listener-0-port" => &current.draft.listeners[0].port,
                        "report-main-server" => &current.draft.exchanges[0].server_port,
                        "report-retry-server" => &current.draft.exchanges[1].server_port,
                        "config-R1" => &current.draft.documents.routers["R1"],
                        _ => panic!(),
                    };
                    assert_eq!(actual, &text);
                    true
                }
            }
            Op::Check(p, want) => {
                if r.phase == 0 {
                    let rect = self
                        .metrics
                        .transport
                        .controls
                        .iter()
                        .find(|c| c.0 == "check")
                        .map(|c| c.1);
                    if self.click_visible_smoke_control(ctx, rect) {
                        r.phase = 1;
                    }
                    false
                } else {
                    let (source, session) = current(self);
                    let engine::Goal::Transport { definition } = &source.goal else {
                        panic!()
                    };
                    let task = if p {
                        &definition.example
                    } else {
                        &definition.assignment
                    };
                    assert!(scope(session, p).tested.is_some());
                    assert_eq!(scope(session, p).checks(task).iter().all(|r| r.0), want);
                    assert_eq!(
                        self.metrics.transport.result.map(|r| r.0 == r.1),
                        Some(want)
                    );
                    if p {
                        assert!(!progress::current_solved(
                            &self.progress.attempts[&source.id],
                            source
                        ));
                    } else {
                        assert_eq!(
                            progress::current_solved(&self.progress.attempts[&source.id], source),
                            want
                        );
                    }
                    true
                }
            }
            Op::Unchecked(p) => {
                let (_, s) = current(self);
                assert!(scope(s, p).tested.is_none());
                assert!(self.metrics.transport.result.is_none());
                true
            }
            Op::Shot(name, result) => {
                if result && r.phase == 0 {
                    let target = self
                        .metrics
                        .transport
                        .result
                        .expect("Checked feedback exists")
                        .2;
                    if self.reveal_smoke_control(ctx, target) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == u8::from(result) {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name.to_owned(),
                    )));
                    r.phase = 2;
                    false
                } else {
                    !self.capture_pending
                }
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none(), "{:?}", self.storage_error);
                let reloaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(&reloaded).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Fresh => {
                let (source, s) = current(self);
                let engine::State::Transport { simulation } = &source.initial else {
                    panic!()
                };
                assert_eq!(s, &**simulation);
                true
            }
        };
        if self
            .pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        {
            r.until = now + 0.15;
        }
        if done {
            r.at += 1;
            r.phase = 0;
            r.frames = 0;
            r.until = now + 0.06;
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint();
    }
}
