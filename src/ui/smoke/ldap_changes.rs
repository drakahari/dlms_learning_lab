//! Fixed native management policy actions with private smoke progress; no real network.
use super::*;
use dlms_learning_lab::ldap_change_workspace as work;
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Style(f32, bool, bool),
    Click(String),
    Field(String, String),
    Grade {
        practice: bool,
        passed: Option<bool>,
    },
    Notice(String),
    Trial(String),
    Cases(usize, usize),
    Fresh,
    Save,
    Shot(String),
    ShotAt(String, String),
}

#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    done: bool,
    frames: usize,
    style_rect: Option<egui::Rect>,
    stable_frames: usize,
    reveal_after: f64,
    action_started: Option<Instant>,
}
pub(in crate::ui) const IDS: [&str; 2] =
    ["lpic2-ldap-person-import", "lpic2-ldap-contact-maintenance"];
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    for (i, id) in IDS.into_iter().enumerate() {
        let source = labs.iter().find(|s| s.id == id).unwrap();
        let d = work::source(source).unwrap();
        let t = &d.initial;
        let alternate = if i == 0 {
            let mut blocks = t.reference.split("\n\n");
            let version = blocks.next().unwrap();
            let mut entries: Vec<_> = blocks
                .map(|s| s.replacen("\nobjectClass:", "\nchangetype: add\nobjectClass:", 1))
                .collect();
            entries.reverse();
            format!("{version}\n\n{}", entries.join("\n\n"))
        } else {
            t.reference
                .replace("replace: mail\nmail:", "replace: mail\n-\nadd: mail\nmail:")
        };
        let partial = if i == 0 {
            t.reference.replace("nova@cedar.test", "wrong@cedar.test")
        } else {
            t.reference
                .replace("replace: mail", "delete: mail\n-\nadd: mail")
        };
        ops.extend([
            Op::Style(1., false, false),
            Op::Open(id.into()),
            Op::Shot(format!("{id}-introduction")),
            Op::ShotAt(format!("{id}-introduction-worked"), "start".into()),
            Op::Click("example".into()),
            Op::ShotAt(format!("{id}-practice-workspace"), "editor".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: true,
                passed: Some(false),
            },
            Op::Cases(0, 3),
            Op::Click("worked".into()),
            Op::Click("fill-example".into()),
            Op::Grade {
                practice: true,
                passed: None,
            },
            Op::Click("apply".into()),
            Op::Trial("Record 1".into()),
            Op::ShotAt(format!("{id}-practice-actual"), "output".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: true,
                passed: Some(true),
            },
            Op::Click("assignment".into()),
            Op::ShotAt(format!("{id}-assignment-workspace"), "editor".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(false),
            },
            Op::Cases(0, 3),
            Op::Field("editor".into(), "version: 1\n\ndn: unfinished".into()),
            Op::Click("apply".into()),
            Op::Notice("Syntax".into()),
            Op::ShotAt(format!("{id}-syntax-feedback"), "apply".into()),
            Op::Field("editor".into(), t.reference.clone()),
            Op::Click("apply".into()),
            Op::Trial("Record 1".into()),
            Op::ShotAt(format!("{id}-assignment-actual"), "output".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(true),
            },
            Op::Field("editor".into(), alternate),
            Op::Grade {
                practice: false,
                passed: None,
            },
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(true),
            },
            Op::Field("editor".into(), partial),
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(false),
            },
            Op::Cases(if i == 0 { 0 } else { 2 }, 3),
            Op::ShotAt(format!("{id}-partial-feedback"), "check".into()),
            Op::Field("editor".into(), t.reference.clone()),
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(true),
            },
            Op::Save,
            Op::Style(1.5, false, false),
            Op::ShotAt(format!("{id}-dark150-wide"), "editor".into()),
            Op::Style(2., true, true),
            Op::ShotAt(format!("{id}-light200-editor"), "editor".into()),
            Op::ShotAt(format!("{id}-light200-result"), "help".into()),
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Fresh,
        ]);
    }
    ops
}

pub(in crate::ui) const UTILITY_IDS: [&str; 2] =
    ["lpic2-ldap-import-command", "lpic2-ldap-stopped-update"];
fn utility_route(labs: &[Scenario]) -> Vec<Op> {
    utility_route_for(labs, false)
}
fn retirement_route(labs: &[Scenario]) -> Vec<Op> {
    utility_route_for(labs, true)
}
fn utility_route_for(labs: &[Scenario], retirement: bool) -> Vec<Op> {
    let mut ops = vec![];
    let ids = if retirement {
        vec!["lpic2-ldap-account-retirement"]
    } else {
        UTILITY_IDS.to_vec()
    };
    for (index, id) in ids.into_iter().enumerate() {
        let d = work::source(labs.iter().find(|s| s.id == id).unwrap()).unwrap();
        let correct = work::utility::Draft::parse(&d.initial.reference).unwrap();
        let mut alternate = correct.clone();
        alternate.command = if retirement {
            correct.command.replace(" -f /work/retire.dn", "")
        } else if index == 0 {
            alternate.command.replacen("ldapadd", "ldapmodify -a", 1)
        } else {
            alternate.command.replacen("ldapmodify -a", "ldapadd", 1)
        };
        let mut partial = correct.clone();
        partial.ldif = if retirement {
            correct.ldif.clone() + "uid=avery,ou=Archive,dc=retirement,dc=test\n"
        } else if index == 0 {
            partial.ldif.replace("@orchard.test", "@incorrect.test")
        } else {
            partial.ldif.replace("replace: mail", "add: mail")
        };
        ops.extend([
            Op::Style(1., false, false),
            Op::Open(id.into()),
            Op::Shot(format!("{id}-introduction")),
            Op::ShotAt(format!("{id}-introduction-worked"), "start".into()),
            Op::Click("example".into()),
            Op::ShotAt(format!("{id}-practice-workspace"), "editor".into()),
            Op::Click("apply".into()),
            Op::Trial(if index == 0 { "50" } else { "68" }.into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: true,
                passed: Some(false),
            },
            Op::Cases(0, 3),
            Op::Click("worked".into()),
            Op::Click("fill-example".into()),
            Op::Grade {
                practice: true,
                passed: None,
            },
            Op::Click("apply".into()),
            Op::Trial("Exit status: 0".into()),
            Op::ShotAt(format!("{id}-practice-actual"), "output".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: true,
                passed: Some(true),
            },
            Op::Click("assignment".into()),
            Op::ShotAt(format!("{id}-assignment-workspace"), "editor".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(false),
            },
            Op::Cases(0, 3),
            Op::Click("credential".into()),
            Op::Click("credential-operator".into()),
            Op::Field("command".into(), correct.command.clone()),
            Op::Field("editor".into(), correct.ldif.clone() + "\nnot a dn\n"),
            Op::Click("apply".into()),
            Op::Trial("Record 3".into()),
            Op::ShotAt(format!("{id}-stopped-feedback"), "output".into()),
            Op::Field("editor".into(), correct.ldif.clone()),
            Op::Click("apply".into()),
            Op::Trial("Exit status: 0".into()),
            Op::ShotAt(format!("{id}-assignment-actual"), "output".into()),
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(true),
            },
            Op::Field("command".into(), alternate.command),
            Op::Grade {
                practice: false,
                passed: None,
            },
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(true),
            },
            Op::Field("editor".into(), partial.ldif),
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(false),
            },
            Op::Cases(if index == 0 { 0 } else { 1 }, 3),
            Op::ShotAt(format!("{id}-partial-feedback"), "check".into()),
            Op::Field("command".into(), correct.command),
            Op::Field("editor".into(), correct.ldif),
            Op::Click("check".into()),
            Op::Grade {
                practice: false,
                passed: Some(true),
            },
            Op::Save,
            Op::Style(1.5, false, false),
            Op::ShotAt(format!("{id}-dark150-wide"), "command".into()),
            Op::Style(2., true, true),
            Op::ShotAt(format!("{id}-light200-command"), "command".into()),
            Op::ShotAt(
                format!("{id}-light200-editor"),
                if retirement { "editor-start" } else { "editor" }.into(),
            ),
            Op::ShotAt(format!("{id}-light200-result"), "help".into()),
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Fresh,
        ]);
    }
    ops
}

impl LabApp {
    /// A collapsing body's widget can exist before its hit area is revealed.
    /// Wait for the actual child clip, rather than clicking through its animation.
    fn ldap_changes_click(
        &mut self,
        ctx: &egui::Context,
        target: Option<(egui::Rect, egui::Rect)>,
        popup: bool,
    ) -> Option<bool> {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            // A popup or disclosure target can disappear after a delivered click.
            return Some(self.click_smoke_control(ctx, None));
        }
        let (rect, clip) = target?;
        if popup {
            let clip = clip.intersect(ctx.content_rect());
            if !smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
                let point = egui::pos2(
                    rect.center().x.clamp(clip.left(), clip.right()),
                    clip.center().y,
                );
                assert_eq!(
                    ctx.layer_id_at(point).map(|l| l.order),
                    Some(egui::Order::Foreground),
                    "Popup wheel must reach the actual menu layer"
                );
                self.pending_events.extend([
                    egui::Event::PointerMoved(point),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(
                            0.,
                            (clip.center().y - rect.center().y).clamp(-500., 500.),
                        ),
                        modifiers: Default::default(),
                    },
                ]);
                self.started = Instant::now();
                ctx.request_repaint();
                return Some(false);
            }
            return Some(self.click_smoke_control(ctx, Some(rect)));
        }
        if !self.reveal_smoke_control(ctx, rect) {
            return Some(false);
        }
        if !clip.expand(1. / ctx.pixels_per_point()).contains_rect(rect) {
            return None;
        }
        Some(self.click_smoke_control(ctx, Some(rect)))
    }

    pub(in crate::ui) fn ldap_changes_tick(&mut self, ctx: &egui::Context) {
        self.ldap_directory_tick(ctx, 0);
    }
    pub(in crate::ui) fn ldap_utility_tick(&mut self, ctx: &egui::Context) {
        self.ldap_directory_tick(ctx, 1);
    }
    pub(in crate::ui) fn ldap_retirement_tick(&mut self, ctx: &egui::Context) {
        self.ldap_directory_tick(ctx, 2);
    }
    fn ldap_directory_tick(&mut self, ctx: &egui::Context, kind: u8) {
        let utility = kind != 0;
        let key = egui::Id::new(match kind {
            2 => "ldap_retirement-native-route",
            1 => "ldap_utility-native-route",
            _ => "ldap_changes-native-route",
        });
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: if kind == 2 {
                    retirement_route(&self.scenarios)
                } else if utility {
                    utility_route(&self.scenarios)
                } else {
                    route(&self.scenarios)
                },
                ..Default::default()
            });
        if r.done {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if r.at == r.ops.len() {
            if kind == 2 {
                println!(
                    "LDAP_RETIREMENT_NATIVE_FLOW_COMPLETE: one DN-list assignment; separate practice, branch/access error, over-deletion, editor-input alternative, invalidation, save-read/reset; no host execution"
                );
            } else if utility {
                println!(
                    "LDAP_UTILITIES_NATIVE_FLOW_COMPLETE: two utility assignments; supplied virtual input and fictional profiles, actual command/request/state outcomes, alternate/partial/stopped/retry/save-read/reset; no OS or network execution"
                );
            } else {
                println!(
                    "LDAP_CHANGES_NATIVE_FLOW_COMPLETE: two directory-data assignments; separate practice, actual LDIF requests and state, varied directories, alternate/partial/broken/retry/save-read/reset; no host LDAP, credentials, shell or network"
                );
            }
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 100000 + r.at;
        let op = r.ops[r.at].clone();
        let began = *r.action_started.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed().as_secs() < 15,
            "LdapChanges native action exceeded15seconds: {} {:?}",
            r.at,
            op
        );
        // Optimized rendering can yield thousands of callbacks during a short
        // asynchronous settle delay. Waiting consumes time, not a work attempt.
        // The independent wall deadline still bounds even a stalled egui clock.
        if matches!(&op, Op::Shot(_) | Op::ShotAt(_, _) if (r.phase==0 && ctx.input(|i|i.time)<r.reveal_after)
            || (r.phase==2 && self.capture_pending))
            || matches!(&op, Op::Click(_) if ctx.input(|i| i.time) < r.reveal_after)
        {
            ctx.data_mut(|d| d.insert_temp(key, r));
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        if r.frames == 0 {
            println!("LDAP_CHANGES_NATIVE_STEP {} {:?}", r.at, op);
        }
        // A resize acknowledgement still needs geometry observations, but
        // those observations are waiting rather than new work attempts.
        if !matches!(&op, Op::Style(..) if r.phase==1) {
            r.frames += 1;
            assert!(
                r.frames < 1200,
                "LdapChanges native action did not complete: {} {:?}",
                r.at,
                op
            );
        }
        let complete = match op {
            Op::Style(scale, light, narrow) => {
                if r.phase == 0 {
                    self.scale = scale;
                    ctx.set_zoom_factor(scale);
                    self.theme_id = if light {
                        theme::inventory()
                            .iter()
                            .find(|t| t.light)
                            .unwrap()
                            .id
                            .clone()
                    } else {
                        theme::Preferences::default().theme
                    };
                    theme::apply(ctx, &self.theme_id);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!narrow));
                    if narrow {
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                            1268., 1308.,
                        )));
                    }
                    self.lesson_scroll_reset = true;
                    r.phase = 1;
                } else {
                    let rect = ctx.content_rect();
                    let ppp = ctx.pixels_per_point();
                    let dimensions = if narrow {
                        (rect.width() * ppp - 1268.).abs() <= 4.
                            && (rect.height() * ppp - 1308.).abs() <= 4.
                    } else {
                        ctx.input(|i| i.viewport().fullscreen) != Some(false)
                    };
                    if dimensions
                        && (ctx.zoom_factor() - scale).abs() < 0.01
                        && r.style_rect == Some(rect)
                    {
                        r.stable_frames += 1;
                    } else {
                        r.stable_frames = 0;
                    }
                    r.style_rect = Some(rect);
                    if r.stable_frames >= 2 {
                        r.phase = 2;
                    }
                }
                if r.phase != 2 {
                    assert!(
                        self.started.elapsed().as_secs() < 15,
                        "LdapChanges native window geometry did not settle"
                    );
                    ctx.data_mut(|d| d.insert_temp(key, r));
                    ctx.request_repaint_after(std::time::Duration::from_millis(25));
                    return;
                }
                true
            }

            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                true
            }
            Op::Click(name) => {
                let viewport = self.metrics.main_viewport.unwrap_or(ctx.content_rect());
                let target = if name == "reset-header" {
                    self.metrics.reset.map(|r| (r, ctx.content_rect()))
                } else if name == "confirm-reset" {
                    self.metrics.confirm_reset.map(|r| (r, viewport))
                } else {
                    self.metrics
                        .ldap_changes
                        .controls
                        .iter()
                        .find(|c| c.0 == name)
                        .map(|c| (c.1, c.2))
                };
                match self.ldap_changes_click(
                    ctx,
                    target,
                    name.starts_with("case-")
                        || name.starts_with("line-")
                        || name.starts_with("credential-"),
                ) {
                    Some(done) => done,
                    None => {
                        r.reveal_after = ctx.input(|i| i.time) + 0.025;
                        false
                    }
                }
            }
            Op::Field(name, value) => {
                assert!(matches!(name.as_str(), "editor" | "command"));
                let field = if name == "command" {
                    self.metrics.ldap_changes.command
                } else {
                    self.metrics.ldap_changes.editor
                }
                .map(|v| (v.0, v.1));
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, field.map(|f| f.0)) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    assert_eq!(
                        ctx.memory(|m| m.focused()),
                        field.map(|f| f.1),
                        "native directory query document field focus"
                    );
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
                    if value.is_empty() {
                        for pressed in [true, false] {
                            self.pending_events.push(egui::Event::Key {
                                key: egui::Key::Backspace,
                                physical_key: None,
                                pressed,
                                repeat: false,
                                modifiers: Default::default(),
                            });
                        }
                    } else {
                        self.pending_events.push(egui::Event::Paste(value));
                    }
                    r.phase = 2;
                    false
                } else {
                    let source = &self.scenarios[self.selected.unwrap()];
                    let engine::State::LdapChanges { simulation } =
                        &self.progress.attempts[&source.id].state
                    else {
                        unreachable!()
                    };
                    let scope = if simulation.view == teaching::View::Example {
                        &simulation.practice
                    } else {
                        &simulation.assignment
                    };
                    assert_eq!(
                        if utility {
                            let p = work::utility::Draft::parse(&scope.draft).unwrap();
                            if name == "command" { p.command } else { p.ldif }
                        } else {
                            scope.draft.clone()
                        },
                        value,
                        "Native paste must reach the actual directory query document field"
                    );
                    true
                }
            }
            Op::Grade { practice, passed } => {
                let source = &self.scenarios[self.selected.unwrap()];
                let definition = work::source(source).unwrap();
                let engine::State::LdapChanges { simulation } =
                    &self.progress.attempts[&source.id].state
                else {
                    unreachable!()
                };
                let (task, scope) = if practice {
                    (&definition.example, &simulation.practice)
                } else {
                    (&definition.initial, &simulation.assignment)
                };
                assert_eq!(scope.checked, passed.is_some());
                if let Some(expected) = passed {
                    let rows = scope.checks(task);
                    let count = rows.iter().filter(|r| r.0).count();
                    assert_eq!(
                        self.metrics.ldap_changes.result.map(|r| (r.0, r.1)),
                        Some((count, rows.len()))
                    );
                    assert_eq!(
                        count == rows.len(),
                        expected,
                        "Actual configuration/ordered-frame grade: {rows:?}"
                    );
                }
                assert_eq!(
                    progress::current_solved(&self.progress.attempts[&source.id], source),
                    !practice && passed == Some(true)
                );
                if practice {
                    assert_eq!(
                        simulation.assignment,
                        work::Session::new(definition).unwrap().assignment
                    );
                }
                true
            }
            Op::Cases(passed, total) => {
                assert_eq!(
                    self.metrics.ldap_changes.result.map(|r| (r.0, r.1)),
                    Some((passed, total))
                );
                true
            }
            Op::Trial(text) => {
                let source = &self.scenarios[self.selected.unwrap()];
                let engine::State::LdapChanges { simulation } =
                    &self.progress.attempts[&source.id].state
                else {
                    unreachable!()
                };
                let scope = if simulation.view == teaching::View::Example {
                    &simulation.practice
                } else {
                    &simulation.assignment
                };
                assert!(
                    scope
                        .output
                        .as_ref()
                        .is_some_and(|r| r.actual.contains(&text) || r.feedback.contains(&text)),
                    "Native trial feedback must match actual observation: {:?}",
                    scope.output
                );
                true
            }
            Op::Notice(text) => {
                assert!(
                    self.notice.as_deref().is_some_and(|s| s.contains(&text)),
                    "Specific native retry guidance must be visible: {:?}",
                    self.notice
                );
                true
            }
            Op::Fresh => {
                let source = &self.scenarios[self.selected.unwrap()];
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[&source.id].state).unwrap(),
                    serde_json::to_value(&source.initial).unwrap()
                );
                true
            }
            Op::Save => {
                self.save();
                assert!(
                    self.storage_error.is_none(),
                    "native save {:?}",
                    self.storage_error
                );
                let restored = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(&restored).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            op @ (Op::Shot(_) | Op::ShotAt(_, _)) => {
                let (name, target_key) = match op {
                    Op::Shot(n) => (n, None),
                    Op::ShotAt(n, k) => (n, Some(k)),
                    _ => unreachable!(),
                };
                if r.phase == 0 {
                    let now = ctx.input(|i| i.time);
                    r.reveal_after = now + 0.15;
                    let target = if let Some(key) = target_key {
                        if key == "editor" {
                            self.metrics.ldap_changes.editor.map(|v| v.0)
                        } else if key == "editor-start" {
                            // Capture the first two of the font-sized twelve rows,
                            // rather than centering a mostly empty editor's bottom.
                            self.metrics.ldap_changes.editor.map(|v| {
                                egui::Rect::from_min_size(
                                    v.0.min,
                                    egui::vec2(v.0.width(), v.0.height() / 6.),
                                )
                            })
                        } else if key == "output" {
                            self.metrics.ldap_changes.output
                        } else {
                            self.metrics
                                .ldap_changes
                                .controls
                                .iter()
                                .find(|v| v.0 == key)
                                .map(|v| v.1)
                        }
                    } else {
                        self.metrics.ldap_changes.result.map(|r| r.2)
                    };
                    if target.is_none_or(|target| self.reveal_smoke_control(ctx, target)) {
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
            r.style_rect = None;
            r.stable_frames = 0;
            r.reveal_after = 0.;
            r.action_started = None;
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
    fn actual_capture_phase_requests_a_screenshot_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic2-ldap-person-import")
            .unwrap();
        app.launch(i);
        let engine::State::LdapChanges { simulation } = &mut app
            .progress
            .attempts
            .get_mut("lpic2-ldap-person-import")
            .unwrap()
            .state
        else {
            panic!()
        };
        simulation.view = teaching::View::Assignment;
        for _ in 0..3 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1500., 5000.),
                    )),
                    ..Default::default()
                },
                |ctx| app.render(ctx),
            );
        }
        assert!(app.metrics.ldap_changes.editor.is_some());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("ldap_changes-native-route"),
                Route {
                    ops: vec![Op::ShotAt(
                        "ldap-change-regression-capture".into(),
                        "editor".into(),
                    )],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.ldap_changes_tick(ctx));
        assert!(app.capture_pending);
    }
}

#[cfg(test)]
mod utility_capture_regression {
    use super::*;
    #[test]
    fn actual_utility_capture_phase_requests_screenshot_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == UTILITY_IDS[0])
            .unwrap();
        app.launch(i);
        let engine::State::LdapChanges { simulation } =
            &mut app.progress.attempts.get_mut(UTILITY_IDS[0]).unwrap().state
        else {
            panic!()
        };
        simulation.view = teaching::View::Assignment;
        for _ in 0..3 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1500., 5000.),
                    )),
                    ..Default::default()
                },
                |ctx| app.render(ctx),
            );
        }
        assert!(app.metrics.ldap_changes.command.is_some());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("ldap_utility-native-route"),
                Route {
                    ops: vec![Op::ShotAt(
                        "ldap-utility-capture-regression".into(),
                        "command".into(),
                    )],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.ldap_utility_tick(ctx));
        assert!(app.capture_pending);
    }
}

#[cfg(test)]
mod retirement_capture_regression {
    use super::*;
    #[test]
    fn actual_retirement_capture_phase_requests_screenshot_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic2-ldap-account-retirement")
            .unwrap();
        app.launch(i);
        let engine::State::LdapChanges { simulation } = &mut app
            .progress
            .attempts
            .get_mut("lpic2-ldap-account-retirement")
            .unwrap()
            .state
        else {
            panic!()
        };
        simulation.view = teaching::View::Assignment;
        for _ in 0..3 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1500., 5000.),
                    )),
                    ..Default::default()
                },
                |ctx| app.render(ctx),
            );
        }
        assert!(app.metrics.ldap_changes.command.is_some());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("ldap_retirement-native-route"),
                Route {
                    ops: vec![Op::ShotAt(
                        "ldap-retirement-capture-regression".into(),
                        "editor-start".into(),
                    )],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.ldap_retirement_tick(ctx));
        assert!(app.capture_pending);
    }
}
