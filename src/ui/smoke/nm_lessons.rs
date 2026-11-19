//! Fixed DHCP editor flows: real widget events and private smoke progress only.
use super::*;
use dlms_learning_lab::nm_workspace as work;
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Style(f32, bool, bool),
    Click(String),
    Field(String, String),
    Solved(bool),
    Cases(usize, usize),
    Example(bool),
    Fresh,
    Save,
    Shot(String),
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    done: bool,
    frames: usize,
}
fn route(labs: &[Scenario], dhcp: bool) -> Vec<Op> {
    route_for(labs, dhcp, false)
}
fn route_for(labs: &[Scenario], dhcp: bool, vlan: bool) -> Vec<Op> {
    fn edit(ops: &mut Vec<Op>, files: &work::Files) {
        for (name, text) in files {
            ops.extend([
                Op::Click("file".into()),
                Op::Click(format!("file:{name}")),
                Op::Field(name.clone(), text.clone()),
            ]);
        }
    }
    let mut ops = vec![];
    let mut count = 0;
    for s in labs.iter().filter(|s| {
        s.unit
            == if vlan {
                "linux-vlan-profiles"
            } else if dhcp {
                "linux-automatic-ethernet"
            } else {
                "linux-persistent-networking"
            }
    }) {
        count += 1;
        let engine::Goal::NmProfiles { definition } = &s.goal else {
            unreachable!()
        };
        let solution: work::Files =
            serde_json::from_str(s.solution[0].strip_prefix(work::PREFIX).unwrap()).unwrap();
        if vlan {
            ops.extend([
                Op::Style(1., false, false),
                Op::Open(s.id.clone()),
                Op::Shot(format!("vlan-{}-introduction", s.id)),
            ]);
        } else {
            ops.push(Op::Open(s.id.clone()));
        }
        ops.extend([
            Op::Click("example".into()),
            Op::Click("run".into()),
            Op::Example(false),
            Op::Click("case".into()),
            Op::Click("case:1".into()),
            Op::Click("insert".into()),
            Op::Example(false),
            Op::Click("run".into()),
            Op::Example(true),
            Op::Shot(format!("nm-{}-practice-passed", s.id)),
        ]);
        if vlan {
            let experiment = definition.example_worked["reports.nmconnection"]
                .replace("parent=enp1s0", "parent=enp2s0");
            ops.extend([
                Op::Click("file".into()),
                Op::Click("file:reports.nmconnection".into()),
                Op::Field("reports.nmconnection".into(), experiment),
                Op::Click("run".into()),
                Op::Example(false),
                Op::Shot(format!("vlan-{}-practice-return-failure", s.id)),
                Op::Click("insert".into()),
                Op::Click("run".into()),
                Op::Example(true),
            ]);
        }
        if dhcp {
            let experiment = if s.id.ends_with("local-only") {
                definition.example_worked["app.nmconnection"]
                    .replace("never-default=true", "never-default=false")
                    .replace("route-metric=100", "route-metric=400")
            } else {
                let dlms_learning_lab::nm_dhcp::Outcome::Bound { address, router } =
                    &definition.example.cases[0].environment.dhcp["enp1s0"]
                else {
                    unreachable!()
                };
                definition.example_worked["app.nmconnection"].replace(
                    "method=auto",
                    &format!(
                        "method=manual\naddress1={}/{}\ngateway={}",
                        address.address,
                        address.prefix,
                        router.unwrap()
                    ),
                )
            };
            ops.extend([
                Op::Click("file".into()),
                Op::Click("file:app.nmconnection".into()),
                Op::Field("app.nmconnection".into(), experiment),
                Op::Click("run".into()),
                Op::Example(false),
                Op::Shot(format!("nm-{}-practice-incomplete", s.id)),
                Op::Click("insert".into()),
                Op::Click("run".into()),
                Op::Example(true),
            ]);
        }
        ops.push(Op::Click("assignment".into()));
        if vlan {
            ops.push(Op::Shot(format!("vlan-{}-assignment-unchecked", s.id)));
        }
        ops.extend([
            Op::Click("run".into()),
            Op::Solved(false),
            Op::Shot(format!("nm-{}-starter-feedback", s.id)),
        ]);
        edit(&mut ops, &solution);
        ops.extend([
            Op::Click("run".into()),
            Op::Solved(true),
            Op::Save,
            Op::Shot(format!("nm-{}-passed", s.id)),
        ]);
        ops.extend([
            Op::Click("file".into()),
            Op::Click(
                if vlan {
                    "file:reports.nmconnection"
                } else {
                    "file:app.nmconnection"
                }
                .into(),
            ),
            Op::Field(
                if vlan {
                    "reports.nmconnection"
                } else {
                    "app.nmconnection"
                }
                .into(),
                "[connection]\nunknown=unsupported".into(),
            ),
            Op::Solved(false),
            Op::Click("run".into()),
            Op::Solved(false),
            Op::Shot(format!("nm-{}-damage-feedback", s.id)),
        ]);
        edit(&mut ops, &solution);
        ops.extend([Op::Click("run".into()), Op::Solved(true)]);
        if vlan {
            ops.extend([
                Op::Style(2., true, true),
                Op::Shot(format!("vlan-{}-light200-narrow", s.id)),
                Op::Style(1.5, false, false),
                Op::Shot(format!("vlan-{}-dark150-wide", s.id)),
                Op::Style(1., false, false),
            ]);
        }
        ops.extend([
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Fresh,
        ]);
        assert_ne!(definition.example_worked, definition.initial.starter);
    }
    assert_eq!(count, if vlan || dhcp { 2 } else { 3 });
    ops
}
fn bond_route(labs: &[Scenario]) -> Vec<Op> {
    let source = labs
        .iter()
        .find(|s| s.id == "lpic2-bonded-vlan-handover")
        .unwrap();
    let engine::Goal::NmProfiles { definition } = &source.goal else {
        unreachable!()
    };
    let solution: work::Files =
        serde_json::from_str(source.solution[0].strip_prefix(work::PREFIX).unwrap()).unwrap();
    let controller = &definition.initial.cases[0]
        .environment
        .bond
        .as_ref()
        .unwrap()
        .profiles
        .controller
        .file;
    let backup = &definition.example.cases[0]
        .environment
        .bond
        .as_ref()
        .unwrap()
        .profiles
        .members[1]
        .file;
    let partial = definition.example.starter[controller]
        .replace("autoconnect-ports=0", "autoconnect-ports=1");
    let mut ops = vec![
        Op::Style(1., false, false),
        Op::Open(source.id.clone()),
        Op::Shot("bond-vlan-introduction".into()),
        Op::Click("example".into()),
        Op::Click("run".into()),
        Op::Cases(0, 3),
        Op::Example(false),
        Op::Shot("bond-vlan-example-unfinished".into()),
        Op::Field(controller.clone(), partial),
        Op::Click("run".into()),
        Op::Cases(2, 3),
        Op::Example(false),
        Op::Shot("bond-vlan-example-partial".into()),
        Op::Click("case".into()),
        Op::Click("case:1".into()),
        Op::Shot("bond-vlan-example-missing-backup".into()),
        Op::Click("file".into()),
        Op::Click(format!("file:{backup}")),
        Op::Field(backup.clone(), definition.example_worked[backup].clone()),
        Op::Click("run".into()),
        Op::Cases(3, 3),
        Op::Example(true),
        Op::Shot("bond-vlan-example-passed".into()),
        Op::Click("assignment".into()),
        Op::Shot("bond-vlan-assignment-unchecked".into()),
        Op::Click("run".into()),
        Op::Cases(0, 3),
        Op::Solved(false),
        Op::Shot("bond-vlan-assignment-unfinished".into()),
    ];
    fn edit(ops: &mut Vec<Op>, files: &work::Files) {
        for (name, text) in files {
            ops.extend([
                Op::Click("file".into()),
                Op::Click(format!("file:{name}")),
                Op::Field(name.clone(), text.clone()),
            ]);
        }
    }
    edit(&mut ops, &solution);
    ops.extend([
        Op::Click("run".into()),
        Op::Cases(3, 3),
        Op::Solved(true),
        Op::Save,
        Op::Shot("bond-vlan-assignment-passed".into()),
        Op::Click("file".into()),
        Op::Click(format!("file:{controller}")),
        Op::Field(
            controller.clone(),
            solution[controller].replace("mode=active-backup", "mode=802.3ad"),
        ),
        Op::Solved(false),
        Op::Click("run".into()),
        Op::Cases(0, 3),
        Op::Solved(false),
        Op::Shot("bond-vlan-unsupported-mode".into()),
        Op::Field(controller.clone(), solution[controller].clone()),
        Op::Click("run".into()),
        Op::Cases(3, 3),
        Op::Solved(true),
        Op::Save,
        Op::Style(2., true, true),
        Op::Shot("bond-vlan-light200-narrow".into()),
        Op::Style(1.5, false, false),
        Op::Shot("bond-vlan-dark150-wide".into()),
        Op::Style(1., false, false),
        Op::Click("reset-header".into()),
        Op::Click("confirm-reset".into()),
        Op::Fresh,
    ]);
    ops
}
fn bridge_route(labs: &[Scenario]) -> Vec<Op> {
    let source = labs
        .iter()
        .find(|s| s.id == "lpic2-bridge-lan-commission")
        .unwrap();
    let engine::Goal::NmProfiles { definition } = &source.goal else {
        unreachable!()
    };
    let solution: work::Files =
        serde_json::from_str(source.solution[0].strip_prefix(work::PREFIX).unwrap()).unwrap();
    let controller = &definition.example.cases[0]
        .environment
        .bridge
        .as_ref()
        .unwrap()
        .profiles
        .controller
        .file;
    let right = &definition.example.cases[0]
        .environment
        .bridge
        .as_ref()
        .unwrap()
        .profiles
        .members[1]
        .file;
    let mut ops = vec![
        Op::Style(1., false, false),
        Op::Open(source.id.clone()),
        Op::Shot("bridge-lan-introduction".into()),
        Op::Click("example".into()),
        Op::Click("run".into()),
        Op::Cases(0, 3),
        Op::Example(false),
        Op::Shot("bridge-lan-example-unfinished".into()),
        Op::Field(
            controller.clone(),
            definition.example_worked[controller].clone(),
        ),
        Op::Click("run".into()),
        Op::Cases(1, 3),
        Op::Example(false),
        Op::Shot("bridge-lan-example-partial".into()),
        Op::Click("case".into()),
        Op::Click("case:0".into()),
        Op::Shot("bridge-lan-example-missing-port".into()),
        Op::Click("file".into()),
        Op::Click(format!("file:{right}")),
        Op::Field(right.clone(), definition.example_worked[right].clone()),
        Op::Click("run".into()),
        Op::Cases(3, 3),
        Op::Example(true),
        Op::Shot("bridge-lan-example-passed".into()),
        Op::Click("assignment".into()),
        Op::Shot("bridge-lan-assignment-unchecked".into()),
        Op::Click("run".into()),
        Op::Cases(0, 3),
        Op::Solved(false),
        Op::Shot("bridge-lan-assignment-unfinished".into()),
    ];
    fn edit(ops: &mut Vec<Op>, files: &work::Files) {
        for (name, text) in files {
            ops.extend([
                Op::Click("file".into()),
                Op::Click(format!("file:{name}")),
                Op::Field(name.clone(), text.clone()),
            ]);
        }
    }
    edit(&mut ops, &solution);
    ops.extend([
        Op::Click("run".into()),
        Op::Cases(3, 3),
        Op::Solved(true),
        Op::Save,
        Op::Shot("bridge-lan-assignment-passed".into()),
        Op::Click("file".into()),
        Op::Click(format!("file:{controller}")),
        Op::Field(
            controller.clone(),
            solution[controller].replace("10.240.0.20/24", "10.240.0.20/25"),
        ),
        Op::Solved(false),
        Op::Click("run".into()),
        Op::Cases(0, 3),
        Op::Shot("bridge-lan-wrong-mask".into()),
        Op::Click("file".into()),
        Op::Click(format!("file:{right}")),
        Op::Field(
            right.clone(),
            format!("{}\n[bridge-port]\nhairpin-mode=true\n", solution[right]),
        ),
        Op::Click("run".into()),
        Op::Cases(0, 3),
        Op::Solved(false),
        Op::Shot("bridge-lan-unsupported-port".into()),
    ]);
    edit(&mut ops, &solution);
    ops.extend([
        Op::Click("run".into()),
        Op::Cases(3, 3),
        Op::Solved(true),
        Op::Save,
        Op::Style(2., true, true),
        Op::Shot("bridge-lan-light200-narrow".into()),
        Op::Style(1.5, false, false),
        Op::Shot("bridge-lan-dark150-wide".into()),
        Op::Style(1., false, false),
        Op::Click("reset-header".into()),
        Op::Click("confirm-reset".into()),
        Op::Fresh,
    ]);
    ops
}
impl LabApp {
    pub(in crate::ui) fn nm_profiles_lessons_tick(&mut self, ctx: &egui::Context) {
        let bridge = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("nm-bridge-batch");
        let bond = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("nm-bond-batch");
        let vlan = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("nm-vlan-batch");
        let dhcp = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("nm-dhcp-batch");
        let key = egui::Id::new(if bridge {
            "nm-bridge-native-route"
        } else if bond {
            "nm-bond-native-route"
        } else if vlan {
            "nm-vlan-native-route"
        } else if dhcp {
            "nm-dhcp-native-route"
        } else {
            "nm-profiles-native-route"
        });
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: if bridge {
                    bridge_route(&self.scenarios)
                } else if bond {
                    bond_route(&self.scenarios)
                } else if vlan {
                    route_for(&self.scenarios, false, true)
                } else {
                    route(&self.scenarios, dhcp)
                },
                ..Default::default()
            });
        if r.done {
            self.teaching_preview_tick(ctx);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "NM_PROFILES_UNIT_NATIVE_FLOW_COMPLETE: {}",
                if bridge {
                    "one untagged bridge capstone: actual0/3→1/3→3/3practice, all physical cases, current/future/IP/membership, wrong-mask/unsupported-port/retry/save-read/reset and light200narrow/dark150wide; no host networking"
                } else if bond {
                    "one bonded VLAN capstone: actual0/3→2/3→3/3practice, both current/future views, assignment/damage/retry/save-read/reset and light200narrow/dark150wide; no host networking"
                } else if vlan {
                    "two VLAN profile/current-future/directional/starter/solution/damage/retry/save-read/reset flows and isolated practice; no host activity"
                } else if dhcp {
                    "two DHCP-profile starter/solution/damage/retry/save-read/reset flows, separate guided practice and intentional incomplete repairs; no daemon, host commands or network operations"
                } else {
                    "all three static-profile starter/solution/damage/retry/save-read/reset flows; three separate guided examples pass; no daemon, host commands or network operations"
                }
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 90000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("NM_PROFILES_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "NmProfiles native action did not complete: {} {:?}",
            r.at,
            op
        );
        let complete = match op {
            Op::Style(scale, light, narrow) => {
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
                true
            }

            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                true
            }
            Op::Click(key) => {
                if key.starts_with("file:") || key.starts_with("case:") {
                    let queued = ctx
                        .data(|d| {
                            d.get_temp::<u8>(egui::Id::new((
                                "native-click-phases",
                                self.smoke_stage,
                            )))
                        })
                        .unwrap_or(0);
                    if queued >= 2 {
                        self.click_smoke_control(ctx, None)
                    } else {
                        let (_, rect, clip) = self
                            .metrics
                            .nm_profiles
                            .controls
                            .iter()
                            .find(|c| c.0 == key)
                            .expect("Open popup choice");
                        let (rect, clip) = (*rect, clip.intersect(ctx.content_rect()));
                        if r.phase > 0 {
                            r.phase -= 1;
                            false
                        } else if smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
                            self.click_smoke_control(ctx, Some(rect))
                        } else {
                            self.pending_events.extend([
                                egui::Event::PointerMoved(clip.center()),
                                egui::Event::MouseWheel {
                                    unit: egui::MouseWheelUnit::Point,
                                    delta: egui::vec2(
                                        0.,
                                        (clip.center().y - rect.center().y).clamp(
                                            -clip.height().max(0.) * 0.75,
                                            clip.height().max(0.) * 0.75,
                                        ),
                                    ),
                                    modifiers: Default::default(),
                                },
                            ]);
                            r.phase = 6;
                            false
                        }
                    }
                } else {
                    let rect = if key == "reset-header" {
                        self.metrics.reset
                    } else if key == "confirm-reset" {
                        self.metrics.confirm_reset
                    } else {
                        self.metrics
                            .nm_profiles
                            .controls
                            .iter()
                            .find(|c| c.0 == key)
                            .map(|c| c.1)
                    };
                    self.click_visible_smoke_control(ctx, rect)
                }
            }
            Op::Field(key, value) => {
                let field = self.metrics.nm_profiles.editor;
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, field.map(|f| f.0)) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    assert_eq!(
                        ctx.memory(|m| m.focused()),
                        field.map(|f| f.1),
                        "native field focus {key}"
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
                    self.pending_events.push(egui::Event::Paste(value));
                    r.phase = 2;
                    false
                } else {
                    let source = &self.scenarios[self.selected.unwrap()];
                    let engine::State::NmProfiles { simulation } =
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
                        scope.selected, key,
                        "Native file picker must select the requested profile before editing"
                    );
                    assert_eq!(
                        scope.draft[&key], value,
                        "Native paste must reach the selected profile"
                    );
                    true
                }
            }
            Op::Cases(passed, total) => {
                if self.nm_profiles_run.is_some() {
                    ctx.data_mut(|d| d.insert_temp(key, r));
                    ctx.request_repaint();
                    return;
                }
                assert_eq!(
                    self.metrics.nm_profiles.result.map(|x| (x.0, x.1)),
                    Some((passed, total)),
                    "Actual native case count"
                );
                true
            }
            Op::Solved(expected) => {
                if self.nm_profiles_run.is_some() {
                    ctx.data_mut(|d| d.insert_temp(key, r));
                    ctx.request_repaint();
                    return;
                }
                let source = &self.scenarios[self.selected.unwrap()];
                assert_eq!(
                    progress::current_solved(&self.progress.attempts[&source.id], source),
                    expected,
                    "native current result {} notice={:?} checks={:?} state={:?}",
                    source.id,
                    self.notice,
                    engine::checks(&self.progress.attempts[&source.id].state, &source.goal),
                    self.progress.attempts[&source.id].state
                );
                true
            }
            Op::Example(expected) => {
                if self.nm_profiles_run.is_some() {
                    ctx.data_mut(|d| d.insert_temp(key, r));
                    ctx.request_repaint();
                    return;
                }
                let source = &self.scenarios[self.selected.unwrap()];
                let (
                    engine::Goal::NmProfiles { definition },
                    engine::State::NmProfiles { simulation },
                ) = (&source.goal, &self.progress.attempts[&source.id].state)
                else {
                    unreachable!()
                };
                assert_eq!(simulation.practice.passed(&definition.example), expected);
                assert_eq!(
                    simulation.assignment,
                    work::Session::new(definition).unwrap().assignment
                );
                assert!(!progress::current_solved(
                    &self.progress.attempts[&source.id],
                    source
                ));
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
            Op::Shot(name) => {
                if r.phase == 0 {
                    let target = self
                        .metrics
                        .nm_profiles
                        .result
                        .map(|r| r.2)
                        .or_else(|| self.metrics.nm_profiles.editor.map(|r| r.0));
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
    fn nm_profiles_native_phases_choose_the_third_file_before_paste_and_run() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        app.smoke = true;
        let source = app
            .scenarios
            .iter()
            .find(|s| s.id == "lpic1-nm-autoconnect-choice")
            .unwrap()
            .clone();
        let files: work::Files =
            serde_json::from_str(source.solution[0].strip_prefix(work::PREFIX).unwrap()).unwrap();
        let ops = vec![
            Op::Open(source.id.clone()),
            Op::Click("assignment".into()),
            Op::Click("file".into()),
            Op::Click("file:replacement.nmconnection".into()),
            Op::Field(
                "replacement.nmconnection".into(),
                files["replacement.nmconnection"].clone(),
            ),
            Op::Click("run".into()),
            Op::Solved(true),
        ];
        let len = ops.len();
        let key = egui::Id::new("nm-profiles-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops,
                    ..Default::default()
                },
            )
        });
        for n in 0..300 {
            let mut input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(2560., 1440.),
                )),
                time: Some(n as f64 / 60.),
                ..Default::default()
            };
            eframe::App::raw_input_hook(&mut app, &ctx, &mut input);
            let _ = ctx.run(input, |ctx| {
                app.render(ctx);
                app.nm_profiles_lessons_tick(ctx);
            });
            if ctx.data(|d| d.get_temp::<Route>(key)).unwrap().at == len {
                assert!(progress::current_solved(
                    &app.progress.attempts[&source.id],
                    &source
                ));
                return;
            }
        }
        panic!("Native third-file phases did not complete");
    }
    #[test]
    fn nm_profiles_native_capture_uses_real_context_safe_screenshot_request() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-nm-persistent-profile")
            .unwrap();
        app.launch(i);
        if let engine::State::NmProfiles { simulation } = &mut app
            .progress
            .attempts
            .get_mut("lpic1-nm-persistent-profile")
            .unwrap()
            .state
        {
            simulation.view = teaching::View::Assignment;
            simulation.assignment.tested = Some(simulation.assignment.draft.clone());
        }
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
        assert!(app.metrics.nm_profiles.result.is_some());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("nm-profiles-native-route"),
                Route {
                    ops: vec![Op::Shot("nm-profiles-regression-capture".into())],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.nm_profiles_lessons_tick(ctx));
        assert!(app.capture_pending);
    }
}

#[cfg(test)]
mod dhcp_tests {
    use super::*;
    #[test]
    fn nm_dhcp_native_routes_select_two_distinct_tasks_and_keep_three_static_flows() {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let new = route(&a.scenarios, true);
        let old = route(&a.scenarios, false);
        assert_eq!(new.iter().filter(|x| matches!(x, Op::Open(_))).count(), 2);
        assert_eq!(old.iter().filter(|x| matches!(x, Op::Open(_))).count(), 3);
        assert_eq!(
            new.iter()
                .filter(|x| matches!(x,Op::Shot(n)if n.ends_with("practice-incomplete")))
                .count(),
            2
        );
    }
}

#[cfg(test)]
mod vlan_tests {
    use super::*;
    #[test]
    fn vlan_introduction_capture_phase_without_results_requests_native_screenshot() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = true;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic2-vlan-profile-commission")
            .unwrap();
        a.launch(i);
        assert!(a.metrics.nm_profiles.result.is_none());
        // Exercise actual capture-phase code with no result/editor target.
        // Store only data; never re-enter Context from a data_mut closure.
        let key = egui::Id::new("nm-profiles-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("vlan-introduction-capture-regression".into())],
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| a.nm_profiles_lessons_tick(ctx));
        let _ = ctx.run(Default::default(), |ctx| a.nm_profiles_lessons_tick(ctx));
        assert!(a.capture_pending);
    }

    #[test]
    fn vlan_native_route_preserves_old_routes_and_includes_current_capture_paths() {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route_for(&a.scenarios, false, true);
        assert_eq!(ops.iter().filter(|x| matches!(x, Op::Open(_))).count(), 2);
        assert_eq!(ops.iter().filter(|x| matches!(x, Op::Shot(_))).count(), 18);
        assert_eq!(ops.iter().filter(|x| matches!(x, Op::Style(..))).count(), 8);
        for name in [
            "introduction",
            "assignment-unchecked",
            "practice-return-failure",
            "light200-narrow",
            "dark150-wide",
        ] {
            assert_eq!(
                ops.iter()
                    .filter(|x| matches!(x,Op::Shot(n)if n.ends_with(name)))
                    .count(),
                2
            );
        }
        assert_eq!(
            route(&a.scenarios, false)
                .iter()
                .filter(|x| matches!(x, Op::Open(_)))
                .count(),
            3
        );
        assert_eq!(
            route(&a.scenarios, true)
                .iter()
                .filter(|x| matches!(x, Op::Open(_)))
                .count(),
            2
        );
    }
}

#[cfg(test)]
mod bond_tests {
    use super::*;
    #[test]
    fn bond_native_route_checks_the_real_partial_sequence_and_keeps_old_routes() {
        let ctx = egui::Context::default();
        let app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = bond_route(&app.scenarios);
        assert_eq!(ops.iter().filter(|x| matches!(x, Op::Open(_))).count(), 1);
        assert_eq!(
            ops.iter()
                .filter_map(|x| if let Op::Cases(p, n) = x {
                    Some((*p, *n))
                } else {
                    None
                })
                .collect::<Vec<_>>(),
            [(0, 3), (2, 3), (3, 3), (0, 3), (3, 3), (0, 3), (3, 3)]
        );
        assert_eq!(ops.iter().filter(|x| matches!(x, Op::Shot(_))).count(), 11);
        assert_eq!(ops.iter().filter(|x| matches!(x, Op::Style(..))).count(), 4);
        assert!(ops.iter().any(|x|matches!(x,Op::Field(name,text)if name=="port-enp2s0.nmconnection"&&text.contains("autoconnect=false")&&text.contains("controller=bond0"))));
        assert_eq!(
            route_for(&app.scenarios, false, true)
                .iter()
                .filter(|x| matches!(x, Op::Open(_)))
                .count(),
            2
        );
        assert_eq!(
            route(&app.scenarios, false)
                .iter()
                .filter(|x| matches!(x, Op::Open(_)))
                .count(),
            3
        );
        assert_eq!(
            route(&app.scenarios, true)
                .iter()
                .filter(|x| matches!(x, Op::Open(_)))
                .count(),
            2
        );
    }
}

#[cfg(test)]
mod bridge_tests {
    use super::*;
    #[test]
    fn bridge_native_route_checks_partial_mask_errors_and_keeps_all_old_routes() {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = bridge_route(&a.scenarios);
        assert_eq!(ops.iter().filter(|x| matches!(x, Op::Open(_))).count(), 1);
        assert_eq!(
            ops.iter()
                .filter_map(|x| if let Op::Cases(p, n) = x {
                    Some((*p, *n))
                } else {
                    None
                })
                .collect::<Vec<_>>(),
            [
                (0, 3),
                (1, 3),
                (3, 3),
                (0, 3),
                (3, 3),
                (0, 3),
                (0, 3),
                (3, 3)
            ]
        );
        assert_eq!(ops.iter().filter(|x| matches!(x, Op::Shot(_))).count(), 12);
        assert_eq!(ops.iter().filter(|x| matches!(x, Op::Style(..))).count(), 4);
        assert!(
            ops.iter()
                .any(|x| matches!(x,Op::Field(_,text)if text.contains("hairpin-mode=true")))
        );
        assert_eq!(
            bond_route(&a.scenarios)
                .iter()
                .filter(|x| matches!(x, Op::Shot(_)))
                .count(),
            11
        );
        assert_eq!(
            route_for(&a.scenarios, false, true)
                .iter()
                .filter(|x| matches!(x, Op::Open(_)))
                .count(),
            2
        );
    }
}
