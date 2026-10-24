//! Fixed native input plan for the original client seed. No reference State publication.
use super::*;
use dlms_learning_lab::client_teaching::Client;
const START: usize = 23000;
const ID: &str = "ccna-gateway";
#[derive(Clone, Debug)]
enum Target {
    Try,
    Fill,
    Apply,
    Check,
    Command,
    Run,
    Field(&'static str),
    Action,
    Choice(usize),
    Insert,
    Assignment,
    Example,
    Back,
    Reset,
    Cancel,
    Confirm,
    Challenge,
    Learn,
}
#[derive(Clone, Debug)]
enum Step {
    Setup,
    Settle,
    Start,
    StartOther,
    RememberOthers,
    Filter,
    Wait,
    Click(Target),
    Text(&'static str),
    Enter,
    Observe(&'static str),
    Remember(&'static str),
    SaveReadback,
    Reload,
    Scale(f32, &'static str),
    Shot(&'static str, bool),
    Finish,
}
fn click(p: &mut Vec<Step>, t: Target) {
    p.push(Step::Click(t));
}
fn edit(p: &mut Vec<Step>, field: &'static str, value: &'static str) {
    p.extend([
        Step::Click(Target::Field(field)),
        Step::Text(value),
        Step::Wait,
    ]);
}
fn command(p: &mut Vec<Step>, text: &'static str, enter: bool) {
    p.extend([Step::Click(Target::Command), Step::Text(text)]);
    p.push(if enter {
        Step::Enter
    } else {
        Step::Click(Target::Run)
    });
    p.push(Step::Wait);
}
fn check(p: &mut Vec<Step>, result: &'static str, image: &'static str) {
    p.extend([
        Step::Click(Target::Check),
        Step::Wait,
        Step::Observe(result),
        Step::Shot(image, true),
    ]);
}
fn plan() -> Vec<Step> {
    let mut p = vec![
        Step::Setup,
        Step::Settle,
        Step::StartOther,
        Step::Wait,
        Step::RememberOthers,
        Step::Click(Target::Back),
        Step::Filter,
        Step::Wait,
        Step::Start,
        Step::Wait,
        Step::Observe("introduction"),
        Step::Shot("introduction", false),
    ];
    click(&mut p, Target::Try);
    p.extend([Step::Wait, Step::Remember("before-fill")]);
    click(&mut p, Target::Fill);
    p.extend([Step::Wait, Step::Observe("fill-only")]);
    click(&mut p, Target::Apply);
    check(&mut p, "example-pass", "example-complete-ungraded");
    click(&mut p, Target::Action);
    click(&mut p, Target::Choice(1));
    p.push(Step::Remember("before-insert"));
    click(&mut p, Target::Insert);
    p.extend([Step::Wait, Step::Observe("insert-only")]);
    click(&mut p, Target::Command);
    p.extend([Step::Enter, Step::Wait, Step::Observe("read-only-example")]);
    command(&mut p, "ping 198.51.100.129", true);
    command(&mut p, "ping 203.0.113.90", false);
    p.push(Step::Observe("reply-example"));
    edit(&mut p, "gateway", "198.51.100.130");
    click(&mut p, Target::Apply);
    check(&mut p, "example-partial", "example-wrong-gateway");
    command(&mut p, "ping 198.51.100.129", true);
    p.push(Step::Observe("reply-example"));
    command(&mut p, "ping 203.0.113.90", false);
    p.push(Step::Observe("unreachable-example"));
    edit(&mut p, "gateway", "198.51.100.129");
    click(&mut p, Target::Apply);
    check(&mut p, "example-pass", "example-repaired");
    click(&mut p, Target::Assignment);
    p.extend([
        Step::Wait,
        Step::Observe("assignment-initial"),
        Step::Shot("full-assignment", false),
    ]);
    command(&mut p, "ping 203.0.113.10", true);
    check(&mut p, "assignment-initial", "initial-feedback");
    edit(&mut p, "prefix", "26");
    click(&mut p, Target::Apply);
    check(&mut p, "assignment-partial", "prefix-only-partial");
    edit(&mut p, "gateway", "192.168.40.65");
    click(&mut p, Target::Apply);
    check(&mut p, "assignment-pass", "complete");
    p.push(Step::Remember("before-query"));
    command(&mut p, "ip route", true);
    p.push(Step::Observe("query-preserves"));
    p.push(Step::Remember("before-malformed"));
    edit(&mut p, "prefix", "bad");
    p.push(Step::Observe("pending"));
    click(&mut p, Target::Apply);
    p.extend([Step::Wait, Step::Observe("rejected")]);
    check(&mut p, "pending", "unapplied-malformed");
    edit(&mut p, "prefix", "26");
    click(&mut p, Target::Apply);
    check(&mut p, "assignment-pass", "retry-complete");
    command(&mut p, "ip addr set 192.168.40.71/24", false);
    p.push(Step::Observe("assignment-fails"));
    // The retained, disclosed simulator shortcut remains a legitimate alternative.
    command(&mut p, "ip route replace default via 192.168.40.65", true);
    command(&mut p, "ip addr set 192.168.40.70/26", true);
    check(&mut p, "assignment-pass", "terminal-alternative");
    p.extend([Step::Scale(1.5, "dlms-light"), Step::Settle]);
    edit(&mut p, "prefix", "25");
    p.push(Step::Shot("light-150-focused-input", false));
    click(&mut p, Target::Apply);
    command(&mut p, "ping 203.0.113.10", true);
    p.push(Step::Shot("light-150-terminal", false));
    check(&mut p, "assignment-fails", "light-150-retry");
    edit(&mut p, "prefix", "26");
    click(&mut p, Target::Apply);
    command(&mut p, "ping 203.0.113.10", false);
    check(&mut p, "assignment-pass", "light-150-complete");
    p.extend([Step::Scale(2., "dlms-dark"), Step::Settle]);
    click(&mut p, Target::Challenge);
    p.extend([Step::Wait, Step::Observe("challenge")]);
    edit(&mut p, "gateway", "192.168.40.66");
    p.push(Step::Shot("dark-200-focused-input", false));
    click(&mut p, Target::Apply);
    command(&mut p, "ping 203.0.113.10", false);
    p.push(Step::Shot("dark-200-terminal", false));
    check(&mut p, "assignment-fails", "dark-200-retry");
    edit(&mut p, "gateway", "192.168.40.65");
    click(&mut p, Target::Apply);
    command(&mut p, "ping 203.0.113.10", true);
    check(&mut p, "assignment-pass", "dark-200-complete");
    p.extend([Step::Scale(1., "dlms-dark"), Step::Settle]);
    click(&mut p, Target::Learn);
    click(&mut p, Target::Example);
    edit(&mut p, "gateway", "unfinished example");
    click(&mut p, Target::Command);
    p.push(Step::Text("ip ro"));
    click(&mut p, Target::Assignment);
    edit(&mut p, "prefix", "unfinished");
    click(&mut p, Target::Command);
    p.push(Step::Text("ping 203.0."));
    p.extend([Step::Remember("resume"), Step::SaveReadback]);
    click(&mut p, Target::Back);
    p.extend([
        Step::Filter,
        Step::Wait,
        Step::Reload,
        Step::Start,
        Step::Wait,
        Step::Observe("resume"),
        Step::Shot("resume-assignment", false),
    ]);
    click(&mut p, Target::Example);
    p.extend([
        Step::Wait,
        Step::Observe("resume-example"),
        Step::Shot("resume-example", false),
    ]);
    click(&mut p, Target::Assignment);
    p.push(Step::Remember("cancel"));
    click(&mut p, Target::Reset);
    click(&mut p, Target::Cancel);
    p.extend([Step::Wait, Step::Observe("cancel")]);
    click(&mut p, Target::Reset);
    click(&mut p, Target::Confirm);
    p.extend([
        Step::Wait,
        Step::Observe("reset"),
        Step::Shot("reset-introduction", false),
    ]);
    click(&mut p, Target::Challenge);
    p.extend([Step::Wait, Step::Observe("challenge")]);
    check(&mut p, "assignment-initial", "challenge-initial");
    p.push(Step::Finish);
    assert!(p.len() < 1000);
    p
}
fn ctrl_a() -> Vec<egui::Event> {
    [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        })
        .collect()
}
impl LabApp {
    fn client_native_settled(&self, ctx: &egui::Context) -> bool {
        // Context geometry MUST be read before entering any data lock.
        let r = ctx.content_rect();
        let ppp = ctx.pixels_per_point();
        let zoom = ctx.zoom_factor();
        let viewport = self.metrics.main_viewport.unwrap_or(r);
        let sample = (
            r.width().to_bits(),
            r.height().to_bits(),
            ppp.to_bits(),
            self.scale.to_bits(),
            viewport.top().to_bits(),
            viewport.bottom().to_bits(),
            zoom.to_bits(),
        );
        let key = egui::Id::new(("client-native-settle", self.smoke_stage));
        let same =
            ctx.data(|d| d.get_temp::<(u32, u32, u32, u32, u32, u32, u32)>(key)) == Some(sample);
        ctx.data_mut(|d| d.insert_temp(key, sample));
        if same && zoom == self.scale {
            return true;
        }
        let attempts_key = egui::Id::new(("client-native-settle-attempts", self.smoke_stage));
        let attempts = ctx.data_mut(|d| {
            let n = d.get_temp::<usize>(attempts_key).unwrap_or(0) + 1;
            d.insert_temp(attempts_key, n);
            n
        });
        assert!(
            attempts < 20,
            "Client geometry/zoom did not settle within20 attempts: actual {r:?}, viewport {viewport:?}, pixels-per-point {ppp}, actual zoom {zoom}, requested zoom {}",
            self.scale
        );
        false
    }
    fn client_native_start(&mut self, ctx: &egui::Context, i: usize) -> bool {
        let key = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0) >= 2 {
            let done = self.click_visible_smoke_control(ctx, None);
            assert_eq!(self.selected, Some(i));
            return done;
        }
        assert!(self.selected.is_none());
        let mut matches = self.scenarios.iter().filter(|s| {
            let a = self.progress.attempts.get(&s.id);
            catalog::matches(
                s,
                &self.filter,
                a.is_some(),
                a.is_some_and(|a| progress::current_solved(a, s)),
                self.progress.favorites.contains(&s.id),
            )
        });
        assert!(
            matches.next().is_some_and(|s| s.id == self.scenarios[i].id)
                && matches.next().is_none()
        );
        assert_eq!(self.metrics.card_actions.len(), 1);
        self.click_visible_smoke_control(ctx, self.metrics.card_actions.first().map(|r| r.0))
    }
    fn client_native_choice(&mut self, ctx: &egui::Context, n: usize) -> bool {
        let key = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0) >= 2 {
            let done = self.click_smoke_control(ctx, None);
            let l = self.progress.attempts[ID]
                .workspace
                .as_ref()
                .unwrap()
                .client_learning
                .as_ref()
                .unwrap();
            assert_eq!(l.selected_action, n);
            return done;
        }
        let r = self
            .metrics
            .client_actions
            .iter()
            .find(|(v, _)| *v == n)
            .expect("actual client action row")
            .1;
        let pane = self
            .metrics
            .client_popup
            .expect("actual content popup bounds")
            .intersect(ctx.content_rect());
        if !smoke_control_visible(r, pane, ctx.pixels_per_point()) {
            let tries_key = egui::Id::new(("client-popup-scroll", self.smoke_stage));
            let tries = ctx.data_mut(|d| {
                let n = d.get_temp::<usize>(tries_key).unwrap_or(0) + 1;
                d.insert_temp(tries_key, n);
                n
            });
            assert!(
                tries <= 20,
                "client action row {r:?} remains clipped in {pane:?}"
            );
            assert!(pane.width() > 0. && pane.height() > 0. && pane.contains(pane.center()));
            let dy = if r.bottom() > pane.bottom() {
                pane.bottom() - r.bottom()
            } else if r.top() < pane.top() {
                pane.top() - r.top()
            } else {
                0.
            };
            self.pending_events.extend([
                egui::Event::PointerMoved(pane.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., dy),
                    modifiers: Default::default(),
                },
            ]);
            self.started = Instant::now();
            ctx.request_repaint();
            return false;
        }
        self.click_smoke_control(ctx, Some(r))
    }
    fn client_native_click(&mut self, ctx: &egui::Context, t: &Target) -> bool {
        if let Target::Choice(n) = t {
            return self.client_native_choice(ctx, *n);
        }
        let r = match t {
            Target::Try | Target::Fill => self.metrics.example_action,
            Target::Apply => self.metrics.editor_save,
            Target::Check => self.metrics.check,
            Target::Command => self.metrics.terminal,
            Target::Run => self.metrics.client_run,
            Target::Field(key) => self
                .metrics
                .client_fields
                .iter()
                .find(|(k, _, _)| k.as_str() == *key)
                .map(|(_, _, r)| *r),
            Target::Action => self.metrics.client_action,
            Target::Insert => self.metrics.command_insert,
            Target::Assignment => self
                .metrics
                .assignment_action
                .or_else(|| self.metrics.lesson_tabs.get(2).copied()),
            Target::Example => self.metrics.lesson_tabs.get(1).copied(),
            Target::Back => self.metrics.back,
            Target::Reset => self.metrics.reset,
            Target::Cancel => self.metrics.reset_cancel,
            Target::Confirm => self.metrics.confirm_reset,
            Target::Challenge => self.metrics.mode_challenge,
            Target::Learn => self.metrics.mode_learn,
            Target::Choice(_) => unreachable!(),
        };
        let done = self.click_visible_smoke_control(ctx, r);
        if done {
            let id = match t {
                Target::Field(key) => self
                    .metrics
                    .client_fields
                    .iter()
                    .find(|(k, _, _)| k.as_str() == *key)
                    .map(|(_, id, _)| *id),
                Target::Command => self.metrics.focus,
                _ => None,
            };
            if let Some(id) = id {
                assert!(
                    ctx.memory(|m| m.has_focus(id)),
                    "the actual clicked client input must own focus"
                );
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("client-native-input-focus"), id));
            }
        }
        done
    }
    fn client_native_observe(&self, ctx: &egui::Context, i: usize, kind: &str) {
        let s = &self.scenarios[i];
        let a = &self.progress.attempts[ID];
        let w = a.workspace.as_ref().unwrap();
        let l = w.client_learning.as_ref().unwrap();
        let lesson = s.client_teaching.as_ref().unwrap();
        let prior = |label: &str| {
            ctx.data(|d| {
                d.get_temp::<serde_json::Value>(egui::Id::new(("client-native-snapshot", label)))
            })
            .unwrap()
        };
        match kind {
            "introduction" => {
                assert_eq!(self.client_view(i), teaching::View::Introduction);
                assert!(self.metrics.check.is_none() && self.metrics.terminal.is_none());
            }
            "fill-only" => {
                let before = prior("before-fill");
                assert_eq!(
                    serde_json::to_value(l.client).unwrap(),
                    before["workspace"]["client_learning"]["client"]
                );
                assert_eq!(a.state, s.initial);
                assert!(!l.draft.matches(&l.client));
                assert!(!progress::current_solved(a, s));
            }
            "insert-only" => {
                let before = prior("before-insert");
                assert_eq!(l.command, "ip route");
                assert_eq!(
                    serde_json::to_value(l.client).unwrap(),
                    before["workspace"]["client_learning"]["client"]
                );
                assert_eq!(
                    serde_json::to_value(&l.transcript).unwrap(),
                    before["workspace"]["client_learning"]["transcript"]
                );
                assert_eq!(a.state, s.initial);
            }
            "example-pass" => {
                assert!(l.checked && l.checks(lesson).iter().all(|c| c.0));
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, s));
            }
            "read-only-example" => {
                let before = prior("before-insert");
                assert_eq!(
                    serde_json::to_value(l.client).unwrap(),
                    before["workspace"]["client_learning"]["client"]
                );
                assert!(!l.transcript.is_empty() && l.checked);
            }
            "reply-example" => assert!(l.transcript.last().unwrap().contains("Reply")),
            "unreachable-example" => assert!(l.transcript.last().unwrap().contains("unreachable")),
            "example-partial" => {
                assert!(l.checked);
                assert_eq!(
                    l.checks(lesson).iter().map(|c| c.0).collect::<Vec<_>>(),
                    [true, true, false]
                );
                assert_eq!(a.state, s.initial);
            }
            "assignment-initial" => {
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, s));
            }
            "assignment-partial" => {
                assert_eq!(
                    engine::checks(&a.state, &s.goal)
                        .iter()
                        .map(|c| c.0)
                        .collect::<Vec<_>>(),
                    [true, true, false]
                );
                assert!(!progress::current_solved(a, s));
            }
            "assignment-pass" => assert!(progress::current_solved(a, s) && self.metrics.completion),
            "assignment-fails" => assert!(!progress::current_solved(a, s)),
            "query-preserves" => {
                assert_eq!(
                    serde_json::to_value(&a.state).unwrap(),
                    prior("before-query")["state"]
                );
                assert!(progress::current_solved(a, s));
                assert!(a.transcript.last().unwrap().contains("default"));
            }
            "pending" => assert!(
                dlms_learning_lab::client_teaching::pending_draft(a)
                    && !progress::current_solved(a, s)
            ),
            "rejected" => {
                assert_eq!(
                    serde_json::to_value(&a.state).unwrap(),
                    prior("before-malformed")["state"]
                );
                assert_eq!(w.client_draft.as_ref().unwrap().prefix, "bad");
                assert!(self.notice.as_ref().unwrap().contains("Prefix"));
                assert!(!progress::current_solved(a, s));
            }
            "resume" => {
                let before = prior("resume");
                assert_eq!(serde_json::to_value(&a.state).unwrap(), before["state"]);
                assert_eq!(serde_json::to_value(w).unwrap(), before["workspace"]);
                assert_eq!(self.client_view(i), teaching::View::Assignment);
                assert_eq!(self.input, "ping 203.0.");
                assert_eq!(w.client_draft.as_ref().unwrap().prefix, "unfinished");
                assert!(!progress::current_solved(a, s));
            }
            "resume-example" => {
                assert_eq!(l.draft.gateway, "unfinished example");
                assert_eq!(l.command, "ip ro");
                let before = prior("resume");
                assert_eq!(serde_json::to_value(&a.state).unwrap(), before["state"]);
                for key in ["client", "draft", "command", "transcript"] {
                    assert_eq!(
                        serde_json::to_value(l).unwrap()[key],
                        before["workspace"]["client_learning"][key]
                    );
                }
                assert!(!progress::current_solved(a, s));
            }
            "cancel" => {
                assert!(!self.reset_pending);
                assert_eq!(serde_json::to_value(a).unwrap(), prior("cancel"));
            }
            "reset" => {
                assert_eq!(a.state, s.initial);
                assert_eq!(l.fixture, lesson.fixture);
                assert_eq!(l.client, lesson.fixture);
                assert!(
                    l.transcript.is_empty()
                        && l.command.is_empty()
                        && a.transcript.is_empty()
                        && w.command.is_empty()
                );
                assert_eq!(
                    w.client_draft.as_ref().unwrap(),
                    &dlms_learning_lab::client_teaching::Draft::new(
                        &Client::from_state(&s.initial).unwrap()
                    )
                );
                assert_eq!(self.client_view(i), teaching::View::Introduction);
                assert!(!progress::current_solved(a, s));
                let others = ctx
                    .data(|d| {
                        d.get_temp::<std::collections::BTreeMap<String, serde_json::Value>>(
                            egui::Id::new("client-native-other-attempts"),
                        )
                    })
                    .unwrap();
                for (id, expected) in others {
                    assert_eq!(
                        serde_json::to_value(&self.progress.attempts[&id]).unwrap(),
                        expected
                    );
                }
            }
            "challenge" => assert!(
                a.challenge
                    && self.client_view(i) == teaching::View::Assignment
                    && self.metrics.lesson_tabs.is_empty()
                    && self.metrics.client_action.is_none()
                    && !self.metrics.help_visible
            ),
            _ => panic!("unknown fixed native client observation"),
        }
    }
    pub(super) fn client_learning_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        let steps = plan();
        let step = steps
            .get(stage - START)
            .expect("bounded fixed client native stage");
        let i = self
            .scenarios
            .iter()
            .position(|s| s.id == ID)
            .expect("normal original client lab");
        match step {
            Step::Setup => {
                assert!(self.smoke);
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
                self.selected = None;
                let other = self.scenarios.iter().find(|s| s.id == "ccna-ipv4").unwrap();
                self.filter = Filter {
                    search: other.title.clone(),
                    ..Default::default()
                };
                self.page = 0;
                self.help_open = false;
            }
            Step::Settle => {
                if !self.client_native_settled(ctx) {
                    return (false, None);
                }
            }
            Step::Filter => {
                assert!(self.selected.is_none());
                self.filter = Filter {
                    search: self.scenarios[i].title.clone(),
                    ..Default::default()
                };
                self.page = 0;
            }
            Step::Start => {
                if !self.client_native_start(ctx, i) {
                    return (false, None);
                }
            }
            Step::StartOther => {
                let other = self
                    .scenarios
                    .iter()
                    .position(|s| s.id == "ccna-ipv4")
                    .unwrap();
                if !self.client_native_start(ctx, other) {
                    return (false, None);
                }
            }
            Step::RememberOthers => {
                let others = self
                    .progress
                    .attempts
                    .iter()
                    .filter(|(id, _)| id.as_str() != ID)
                    .map(|(id, a)| (id.clone(), serde_json::to_value(a).unwrap()))
                    .collect::<std::collections::BTreeMap<_, _>>();
                assert!(!others.is_empty());
                ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new("client-native-other-attempts"), others)
                });
            }
            Step::Wait => {}
            Step::Click(t) => {
                if !self.client_native_click(ctx, t) {
                    return (false, None);
                }
            }
            Step::Text(text) => {
                assert!(text.len() <= 256);
                let id = ctx
                    .data(|d| d.get_temp::<egui::Id>(egui::Id::new("client-native-input-focus")))
                    .expect("prior real input click");
                assert!(ctx.memory(|m| m.has_focus(id)));
                self.pending_events = ctrl_a();
                self.pending_events.push(egui::Event::Text((*text).into()));
            }
            Step::Enter => {
                self.pending_events = [true, false]
                    .into_iter()
                    .map(|pressed| egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: Default::default(),
                    })
                    .collect();
            }
            Step::Observe(kind) => self.client_native_observe(ctx, i, kind),
            Step::Remember(label) => {
                let value = serde_json::to_value(&self.progress.attempts[ID]).unwrap();
                ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new(("client-native-snapshot", *label)), value)
                });
            }
            Step::SaveReadback => {
                self.save();
                assert!(self.storage_error.is_none());
                let before = progress::read_bounded(&self.path, 16_000_000).unwrap();
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                let expected = ctx
                    .data(|d| {
                        d.get_temp::<serde_json::Value>(egui::Id::new((
                            "client-native-snapshot",
                            "resume",
                        )))
                    })
                    .unwrap();
                assert_eq!(
                    serde_json::to_value(&loaded.attempts[ID]).unwrap(),
                    expected
                );
                assert_eq!(
                    progress::read_bounded(&self.path, 16_000_000).unwrap(),
                    before
                );
            }
            Step::Reload => {
                assert!(self.selected.is_none());
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
            }
            Step::Scale(scale, theme_id) => {
                self.scale = *scale;
                self.theme_id = (*theme_id).into();
                theme::apply(ctx, &self.theme_id);
                ctx.set_zoom_factor(*scale);
                self.lesson_scroll_reset = true;
            }
            Step::Shot(name, feedback) => {
                if *feedback
                    && !self.reveal_smoke_control(
                        ctx,
                        self.metrics
                            .feedback
                            .expect("actual current feedback block"),
                    )
                {
                    return (false, None);
                }
                return (true, Some(format!("client-foundation-{name}")));
            }
            Step::Finish => {
                println!(
                    "PASS native original client teaching: actual library Start/Continue, isolated Fill/Insert/Apply/Run, wrong-gateway local/remote experiment and repair, full assignment wrong/partial/reference/terminal alternate, rejected-input/pending truth,150Light/200Dark real edits+Run, exact separated drafts save/readback/resume, real Reset Cancel/Confirm and Challenge. No new lab/physicalnetwork/novice/platform claim."
                );
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
    fn client_native_plan_is_bounded_and_covers_actual_large_text_retry_and_resets() {
        let p = plan();
        assert!(p.len() < 1000);
        for wanted in [
            "fill-only",
            "insert-only",
            "example-partial",
            "assignment-partial",
            "assignment-pass",
            "pending",
            "rejected",
            "query-preserves",
            "resume",
            "resume-example",
            "cancel",
            "reset",
            "challenge",
        ] {
            assert!(
                p.iter().any(|s| matches!(s,Step::Observe(v)if *v==wanted)),
                "{wanted}"
            );
        }
        for scale in [1.5, 2.] {
            let begin = p
                .iter()
                .position(|s| matches!(s,Step::Scale(v,_)if *v==scale))
                .unwrap();
            let end = p[begin + 1..]
                .iter()
                .position(|s| matches!(s, Step::Scale(..)))
                .map_or(p.len(), |n| begin + 1 + n);
            let part = &p[begin..end];
            assert!(
                part.iter()
                    .any(|s| matches!(s, Step::Click(Target::Field(_))))
            );
            assert!(part.iter().any(|s| matches!(s, Step::Click(Target::Run))));
            assert!(
                part.iter()
                    .any(|s| matches!(s, Step::Observe("assignment-pass")))
            );
        }
        assert!(
            p.iter()
                .filter(|s| matches!(s, Step::Click(Target::Reset)))
                .count()
                == 2
        );
        assert!(p.iter().any(|s| matches!(s, Step::Click(Target::Cancel))));
        for s in &p {
            if let Step::Text(text) = s {
                assert!(text.len() <= 256);
            }
        }
    }
    #[test]
    fn client_native_actual_settle_phase_uses_precomputed_context_geometry() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context(&ctx, true, false);
        a.smoke = false;
        a.smoke_stage = START + 1;
        let mut first = None;
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(900., 700.),
                )),
                ..Default::default()
            },
            |ctx| {
                first = Some(a.client_learning_native(ctx, START + 1));
            },
        );
        assert_eq!(first, Some((false, None)));
        let sample = ctx
            .data(|d| {
                d.get_temp::<(u32, u32, u32, u32, u32, u32, u32)>(egui::Id::new((
                    "client-native-settle",
                    START + 1,
                )))
            })
            .unwrap();
        assert_eq!(sample.0, ctx.content_rect().width().to_bits());
        assert_eq!(sample.2, ctx.pixels_per_point().to_bits());
        assert_eq!(sample.6, ctx.zoom_factor().to_bits());
        assert_eq!(ctx.zoom_factor(), a.scale);
        let mut second = None;
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(900., 700.),
                )),
                ..Default::default()
            },
            |ctx| {
                second = Some(a.client_learning_native(ctx, START + 1));
            },
        );
        assert_eq!(second, Some((true, None)));
    }
    #[test]
    fn client_native_actual_settle_phase_rejects_unstable_geometry_or_wrong_zoom_within_twenty_attempts()
     {
        for wrong_zoom in [false, true] {
            let ctx = egui::Context::default();
            let mut a = LabApp::from_context(&ctx, true, false);
            a.smoke = false;
            a.smoke_stage = START + 1;
            if wrong_zoom {
                a.scale = 2.;
            }
            let before = serde_json::to_value(&a.progress).unwrap();
            for n in 0..19 {
                let w = if wrong_zoom {
                    900.
                } else {
                    900. + (n % 2) as f32
                };
                let mut result = None;
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(w, 700.),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        result = Some(a.client_learning_native(ctx, START + 1));
                    },
                );
                assert_eq!(result, Some((false, None)));
            }
            let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(if wrong_zoom { 900. } else { 901. }, 700.),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        a.client_learning_native(ctx, START + 1);
                    },
                );
            }))
            .expect_err("twentieth unsettled actual phase must diagnose");
            let message = rejected
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| rejected.downcast_ref::<&str>().copied())
                .unwrap();
            assert!(message.contains("within20 attempts"));
            assert!(a.pending_events.is_empty());
            assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
        }
    }
}
