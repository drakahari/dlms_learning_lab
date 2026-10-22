//! Application-only Fedora renderer/input check with isolated /tmp progress.
use super::*;
impl LabApp {
    pub(super) fn capture_events(&mut self, ctx: &egui::Context) {
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot {
                image, user_data, ..
            } = event
            {
                let name = user_data
                    .data
                    .as_ref()
                    .and_then(|d| d.downcast_ref::<String>())
                    .cloned()
                    .unwrap_or("capture".into());
                let dir = std::path::Path::new("screenshots/workspaces");
                std::fs::create_dir_all(dir).unwrap();
                let mut encoder = png::Encoder::new(
                    std::fs::File::create(dir.join(format!("{name}.png"))).unwrap(),
                    image.width() as u32,
                    image.height() as u32,
                );
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                encoder
                    .write_header()
                    .unwrap()
                    .write_image_data(
                        &image
                            .pixels
                            .iter()
                            .flat_map(|p| p.to_array())
                            .collect::<Vec<_>>(),
                    )
                    .unwrap();
                self.capture_pending = false;
                println!("Captured {name}: {}x{}", image.width(), image.height());
            }
        }
    }
    pub(super) fn smoke_tick(&mut self, ctx: &egui::Context) {
        if self.capture_pending || self.started.elapsed().as_millis() < 350 {
            return;
        }
        let key = |key, command| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: command,
                command,
                ..Default::default()
            },
        };
        let click = |rect: egui::Rect| {
            vec![
                egui::Event::PointerMoved(rect.center()),
                egui::Event::PointerButton {
                    pos: rect.center(),
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
                egui::Event::PointerButton {
                    pos: rect.center(),
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                },
            ]
        };
        let mut shot: Option<String> = None;
        match self.smoke_stage {
            2400 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                let source = format!("printf '%s\\n' {}\n", "$spaces\"\"".repeat(32));
                let cases: Vec<_> = [32768, 65536, 98304]
                    .into_iter()
                    .map(|size| shell::Case {
                        name: format!("Controlled prior fixture {size}"),
                        args: vec![],
                        stdin: String::new(),
                        env: std::collections::BTreeMap::from([(
                            "spaces".into(),
                            " ".repeat(size),
                        )]),
                        files: Default::default(),
                        stdout: "\n".repeat(32),
                        stderr: String::new(),
                        status: 0,
                        after: Default::default(),
                    })
                    .collect();
                let mut lab = self.scenarios[9].clone();
                lab.id = "test-expansion-resource".into();
                lab.title = "Expansion bounds · development fixture".into();
                lab.objective="Controlled simulator resource check. Retain earlier revision 3 credit as history; run the current script to inspect its resource diagnostic. Repair or stop a run using the normal native editor. This fixture is not curriculum.".into();
                lab.unit.clear();
                lab.prerequisites.clear();
                lab.teaching = None;
                lab.steps = vec![
                    "Inspect the normal editor, current feedback and retained historical result."
                        .into(),
                ];
                lab.guide.clear();
                lab.goal = engine::Goal::Script {
                    cases: cases.clone(),
                };
                lab.initial = engine::State::Script {
                    source: source.clone(),
                    run: None,
                };
                lab.solution = vec![format!("printf '%s\\n' {}", "\"\" ".repeat(32))];
                let mut a = Attempt::new(&lab, 1);
                a.state = engine::State::Script {
                    source: source.clone(),
                    run: Some(shell::Record {
                        engine_revision: 3,
                        source,
                        results: cases
                            .iter()
                            .map(|c| shell::Outcome {
                                stdout: c.stdout.clone(),
                                stderr: String::new(),
                                status: 0,
                                files: Default::default(),
                                error: None,
                            })
                            .collect(),
                    }),
                };
                self.progress = Progress {
                    schema_version: 1,
                    ..Default::default()
                };
                self.progress.attempts.insert(lab.id.clone(), a);
                self.scenarios.push(lab);
                let i = self.scenarios.len() - 1;
                let root = tempfile::Builder::new()
                    .prefix("dlms-expansion-native-")
                    .tempdir()
                    .unwrap()
                    .keep();
                self.path = root.join("progress-v1.json");
                let original = serde_json::to_vec_pretty(&self.progress).unwrap();
                std::fs::write(&self.path, &original).unwrap();
                std::fs::write(root.join("exact-original.json"), original).unwrap();
                self.launch(i);
            }
            2401 => {
                let i = self.selected.unwrap();
                assert!(progress::legacy_script_credit(
                    &self.progress.attempts[&self.scenarios[i].id],
                    &self.scenarios[i]
                ));
                assert!(!self.metrics.completion);
                shot = Some("expansion-revision3-history".into());
            }
            2402 => self.pending_events = click(self.metrics.check.unwrap()),
            2403 => {}
            2404 => {
                if self.script_job.is_some() {
                    return;
                }
                let i = self.selected.unwrap();
                let a = &self.progress.attempts[&self.scenarios[i].id];
                let engine::State::Script { run: Some(r), .. } = &a.state else {
                    panic!()
                };
                assert!(r.results.iter().all(|r| {
                    r.error
                        .as_ref()
                        .is_some_and(|e| e.contains("Expansion work limit"))
                }));
                assert!(progress::legacy_script_credit(a, &self.scenarios[i]));
                assert!(!self.metrics.completion);
                assert_eq!(
                    std::fs::read(self.path.with_extension(format!(
                        "before-shell-v{}.json",
                        shell::EVALUATOR_REVISION
                    )))
                    .unwrap(),
                    std::fs::read(self.path.parent().unwrap().join("exact-original.json")).unwrap()
                );
                shot = Some("expansion-current-limit-feedback".into());
            }
            2405 | 2416 => self.pending_events = click(self.metrics.inputs[0]),
            2406 => {
                self.pending_events = vec![
                    key(egui::Key::A, true),
                    egui::Event::Text("while true; do :; done".into()),
                ]
            }
            2407 | 2412 => self.pending_events = click(self.metrics.check.unwrap()),
            2408 => {
                assert!(self.script_job.is_some());
                let pos = self.metrics.stop.unwrap().center();
                assert!(ctx.content_rect().contains(pos));
                // Let the enabled Stop control enter egui's previous-frame hit map
                // before injecting a click. Run creates its job after drawing actions.
                shot = Some("expansion-native-running".into());
            }
            2409 => self.pending_events = click(self.metrics.stop.unwrap()),
            2410 => {
                assert!(self.script_job.is_none());
                assert!(!self.metrics.completion);
                shot = Some("expansion-native-stop".into());
                self.pending_events = click(self.metrics.inputs[0]);
            }
            2411 => {
                let source = self.scenarios[self.selected.unwrap()].solution[0].clone();
                self.pending_events = vec![key(egui::Key::A, true), egui::Event::Text(source)];
            }
            2413 => {}
            2414 => {
                if self.script_job.is_some() {
                    return;
                }
                assert!(self.metrics.completion);
                shot = Some("expansion-current-repair".into());
            }
            2415 => {
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                let i = self.selected.unwrap();
                let a = &self.progress.attempts[&self.scenarios[i].id];
                assert!(progress::current_solved(a, &self.scenarios[i]));
                assert!(progress::legacy_script_credit(a, &self.scenarios[i]));
            }
            2417 => {
                self.pending_events = vec![
                    key(egui::Key::A, true),
                    egui::Event::Text("printf wrong".into()),
                ]
            }
            2418 => {
                assert!(!self.metrics.completion);
                let i = self.selected.unwrap();
                assert!(progress::legacy_script_credit(
                    &self.progress.attempts[&self.scenarios[i].id],
                    &self.scenarios[i]
                ));
                shot = Some("expansion-edited-history-retained".into());
            }
            2419 => {
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.scale = 1.5;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(940., 760.)));
            }
            2420 => shot = Some("expansion-history-light-150".into()),
            2421 => {
                self.pending_events = vec![
                    egui::Event::PointerMoved(ctx.content_rect().center()),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., -680.),
                        modifiers: Default::default(),
                    },
                ]
            }
            2422 => shot = Some("expansion-history-light-150-scrolled".into()),
            2423 => {
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                let i = self.selected.unwrap();
                self.progress
                    .attempts
                    .get_mut(&self.scenarios[i].id)
                    .unwrap()
                    .challenge = true;
                self.help_open = false;
                self.lesson_scroll_reset = true;
            }
            2424 => shot = Some("expansion-challenge-dark-150".into()),
            2425 => {
                println!(
                    "PASS native expansion boundary: actual prior fixture history, current resource feedback, exact revision-specific backup, keyboard loop/Stop, repair/all3current cases, reload, edit invalidation and retained history, resized light/dark150. Development fixture only; no new curriculum or novice acceptance."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            2320 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                let i = self
                    .scenarios
                    .iter()
                    .position(|s| s.id == "ccna-vlan-access")
                    .unwrap();
                self.launch(i);
                self.set_switching_view(i, teaching::View::Assignment);
                let s = &self.scenarios[i];
                let a = self.progress.attempts.get_mut(&s.id).unwrap();
                for cmd in &s.solution {
                    engine::execute(&mut a.state, cmd).unwrap();
                }
                let w = a.workspace.as_mut().unwrap();
                w.switching_draft
                    .as_mut()
                    .unwrap()
                    .switches
                    .get_mut("office")
                    .unwrap()
                    .ports
                    .get_mut("staff-b")
                    .unwrap()
                    .access = "bad".into();
                w.checked = true;
                self.checked = true;
                self.dirty = true;
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                self.launch(i);
            }
            2321 => {
                assert!(!self.metrics.completion);
                shot = Some("vlan-resumed-divergent-draft".into());
            }
            2322 => self.pending_events = click(self.metrics.inputs[4]),
            2323 => {
                self.pending_events = vec![key(egui::Key::A, true), egui::Event::Text("10".into())]
            }
            2324 => self.pending_events = click(self.metrics.check.unwrap()),
            2325 => {
                assert!(self.metrics.completion);
                shot = Some("vlan-resumed-draft-repaired".into());
            }
            2326 => {
                println!(
                    "PASS native resumed switching draft: applied state passes, accepted saved draft differs; no current success, keyboard repair and Check restores verified completion."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            n @ 2200..=2299 => {
                let unit = (n - 2200) / 25;
                let step = (n - 2200) % 25;
                let ids = [
                    "ccna-vlan-access",
                    "ccna-vlan-trunk",
                    "ccna-vlan-native",
                    "ccna-vlan-handover",
                ];
                let i = self
                    .scenarios
                    .iter()
                    .position(|s| s.id == ids[unit])
                    .unwrap();
                match step {
                    0 => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                        self.launch(i);
                        assert_eq!(self.switching_view(i), teaching::View::Introduction);
                    }
                    1 => shot = Some(format!("{}-introduction", ids[unit])),
                    2 => self.pending_events = click(self.metrics.example_action.unwrap()),
                    3 => {
                        assert_eq!(self.switching_view(i), teaching::View::Example);
                        let s = &self.scenarios[i];
                        self.progress
                            .attempts
                            .get_mut(&s.id)
                            .unwrap()
                            .workspace
                            .as_mut()
                            .unwrap()
                            .switching_learning
                            .as_mut()
                            .unwrap()
                            .fill_example(s.switching_teaching.as_ref().unwrap())
                            .unwrap();
                        self.dirty = true;
                    }
                    4 => self.pending_events = click(self.metrics.check.unwrap()),
                    5 => {
                        let s = &self.scenarios[i];
                        let a = self.progress.attempts.get_mut(&s.id).unwrap();
                        assert!(!progress::current_solved(a, s));
                        let l = a
                            .workspace
                            .as_mut()
                            .unwrap()
                            .switching_learning
                            .as_mut()
                            .unwrap();
                        assert!(
                            l.checks(s.switching_teaching.as_ref().unwrap())
                                .iter()
                                .all(|c| c.0)
                        );
                        let action = [
                            "access office studio-b 40",
                            "trunk east uplink 90 30,90",
                            "trunk east uplink 40 30,40,90",
                            "vlan east 70 absent",
                        ][unit];
                        let mut candidate = l.fabric.clone();
                        dlms_learning_lab::switching_workspace::adapter(&mut candidate, action)
                            .unwrap();
                        l.draft = dlms_learning_lab::switching_workspace::Draft::new(&candidate);
                        l.checked = false;
                        self.dirty = true;
                    }
                    6 => self.pending_events = click(self.metrics.check.unwrap()),
                    7 => {
                        let s = &self.scenarios[i];
                        let a = &self.progress.attempts[&s.id];
                        assert!(
                            a.workspace
                                .as_ref()
                                .unwrap()
                                .switching_learning
                                .as_ref()
                                .unwrap()
                                .checks(s.switching_teaching.as_ref().unwrap())
                                .iter()
                                .any(|c| !c.0)
                        );
                        shot = Some(format!("{}-practice-retry", ids[unit]));
                    }
                    8 => self.set_switching_view(i, teaching::View::Assignment),
                    9 => self.pending_events = click(self.metrics.check.unwrap()),
                    10 => {
                        assert!(!self.metrics.completion);
                        shot = Some(format!("{}-assignment-retry", ids[unit]));
                    }
                    11 => {
                        let s = &self.scenarios[i];
                        let mut partial = s.initial.clone();
                        for (index, action) in s.solution.iter().enumerate() {
                            if index != [0, 2, 1, 0][unit] {
                                engine::execute(&mut partial, action).unwrap();
                            }
                        }
                        let engine::State::Switching { fabric } = partial else {
                            panic!()
                        };
                        let mut d = dlms_learning_lab::switching_workspace::Draft::new(&fabric);
                        d.selected_switch = ["office", "east", "east", "west"][unit].into();
                        self.progress
                            .attempts
                            .get_mut(&s.id)
                            .unwrap()
                            .workspace
                            .as_mut()
                            .unwrap()
                            .switching_draft = Some(d);
                        self.checked = false;
                        self.dirty = true;
                    }
                    12 => {}
                    13 => self.pending_events = click(self.metrics.check.unwrap()),
                    14 => {
                        assert!(!self.metrics.completion);
                        shot = Some(format!("{}-partial", ids[unit]));
                    }
                    15 => self.pending_events = click(self.metrics.inputs[[4, 4, 3, 2][unit]]),
                    16 => {
                        self.pending_events = vec![
                            key(egui::Key::A, true),
                            egui::Event::Text(["10", "10,20,99", "99", "30"][unit].into()),
                        ]
                    }
                    17 => self.pending_events = click(self.metrics.check.unwrap()),
                    18 => {
                        assert!(self.metrics.completion, "{} native repair", ids[unit]);
                        shot = Some(format!("{}-complete", ids[unit]));
                        self.save();
                    }
                    19 => {
                        let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                        assert!(progress::current_solved(
                            &loaded.attempts[&self.scenarios[i].id],
                            &self.scenarios[i]
                        ));
                        self.progress = loaded;
                    }
                    20 => self.pending_events = click(self.metrics.inputs[[4, 4, 3, 2][unit]]),
                    21 => {
                        self.pending_events =
                            vec![key(egui::Key::A, true), egui::Event::Text("0".into())]
                    }
                    22 => {
                        assert!(!self.metrics.completion);
                        assert!(!progress::current_solved(
                            &self.progress.attempts[&self.scenarios[i].id],
                            &self.scenarios[i]
                        ));
                        shot = Some(format!("{}-draft-invalidation", ids[unit]));
                    }
                    23 => self.pending_events = click(self.metrics.check.unwrap()),
                    24 => {
                        assert!(
                            self.notice
                                .as_ref()
                                .is_some_and(|s| s.contains("1 to 4094"))
                        );
                        assert!(!self.metrics.completion);
                        shot = Some(format!("{}-invalid-field", ids[unit]));
                    }
                    _ => {}
                }
            }
            2300 => {
                let i = self
                    .scenarios
                    .iter()
                    .position(|s| s.id == "ccna-vlan-handover")
                    .unwrap();
                self.set_switching_view(i, teaching::View::Introduction);
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.scale = 1.5;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(940., 760.)));
            }
            2301 => shot = Some("vlan-capstone-introduction-light-150".into()),
            2302 => self.set_switching_view(self.selected.unwrap(), teaching::View::Assignment),
            2303 => shot = Some("vlan-capstone-assignment-light-150".into()),
            2304 => {
                self.pending_events = vec![
                    egui::Event::PointerMoved(ctx.content_rect().center()),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., -720.),
                        modifiers: Default::default(),
                    },
                ]
            }
            2305 => shot = Some("vlan-capstone-scrolled-light-150".into()),
            2306 => {
                let i = self.selected.unwrap();
                self.progress
                    .attempts
                    .get_mut(&self.scenarios[i].id)
                    .unwrap()
                    .challenge = true;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                self.help_open = false;
                self.lesson_scroll_reset = true;
            }
            2307 => shot = Some("vlan-capstone-challenge-dark-150".into()),
            2308 => {
                let i = self.selected.unwrap();
                self.reset_attempt(i);
                assert_eq!(
                    self.progress.attempts[&self.scenarios[i].id].state,
                    self.scenarios[i].initial
                );
                self.selected = None;
                self.filter = Filter {
                    track: "CCNA".into(),
                    unit: "vlan-data-foundations".into(),
                    ..Default::default()
                };
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
            }
            2309 => {}
            2310 => {
                assert_eq!(self.metrics.card_actions.len(), 4);
                shot = Some("vlan-unit-library-light-150".into());
            }
            2311 => {
                println!(
                    "PASS native authored VLAN unit: all four introductions, isolated example experiments, wrong/partial feedback, keyboard final repair, native Check/completion, save/reload, edit invalidation, selected reset and light/dark150 resizing plus unit library. No equipment, novice-user or other-platform runtime acceptance claimed."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            2100 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                let fixture =
                    pack::load(include_str!("../../tests/fixtures/switching-preview.json"))
                        .unwrap()
                        .remove(0);
                self.scenarios.push(fixture);
                self.launch(self.scenarios.len() - 1);
            }
            2101 => shot = Some("switching-fixture-introduction".into()),
            2102 => self.pending_events = click(self.metrics.example_action.unwrap()),
            2103 => {
                let i = self.selected.unwrap();
                assert_eq!(self.switching_view(i), teaching::View::Example);
                let s = &self.scenarios[i];
                let l = self
                    .progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .switching_learning
                    .as_mut()
                    .unwrap();
                l.fill_example(s.switching_teaching.as_ref().unwrap())
                    .unwrap();
                self.dirty = true;
            }
            2104 => self.pending_events = click(self.metrics.check.unwrap()),
            2105 => {
                let i = self.selected.unwrap();
                let s = &self.scenarios[i];
                let a = &self.progress.attempts[&s.id];
                assert!(
                    a.workspace
                        .as_ref()
                        .unwrap()
                        .switching_learning
                        .as_ref()
                        .unwrap()
                        .checks(s.switching_teaching.as_ref().unwrap())
                        .iter()
                        .all(|c| c.0)
                );
                assert!(!progress::current_solved(a, s));
                shot = Some("switching-fixture-practice".into());
            }
            2106 => {
                let i = self.selected.unwrap();
                let l = self
                    .progress
                    .attempts
                    .get_mut(&self.scenarios[i].id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .switching_learning
                    .as_mut()
                    .unwrap();
                l.draft.selected_switch = "right".into();
                l.draft
                    .switches
                    .get_mut("right")
                    .unwrap()
                    .ports
                    .get_mut("staff")
                    .unwrap()
                    .access = "40".into();
                self.dirty = true;
            }
            2107 => self.pending_events = click(self.metrics.check.unwrap()),
            2108 => {
                let i = self.selected.unwrap();
                let s = &self.scenarios[i];
                assert!(
                    self.progress.attempts[&s.id]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .switching_learning
                        .as_ref()
                        .unwrap()
                        .checks(s.switching_teaching.as_ref().unwrap())
                        .iter()
                        .any(|c| !c.0)
                );
                shot = Some("switching-fixture-practice-retry".into());
            }
            2109 => self.set_switching_view(self.selected.unwrap(), teaching::View::Assignment),
            2110 => shot = Some("switching-fixture-assignment".into()),
            2111 => self.pending_events = click(self.metrics.check.unwrap()),
            2112 => {
                assert!(!self.metrics.completion);
                shot = Some("switching-fixture-assignment-retry".into());
            }
            2113 => {
                let i = self.selected.unwrap();
                self.progress
                    .attempts
                    .get_mut(&self.scenarios[i].id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .switching_draft
                    .as_mut()
                    .unwrap()
                    .selected_switch = "right".into();
                self.dirty = true;
            }
            2114 => self.pending_events = click(self.metrics.inputs[2]),
            2115 => {
                self.pending_events = vec![key(egui::Key::A, true), egui::Event::Text("10".into())]
            }
            2116 => self.pending_events = click(self.metrics.check.unwrap()),
            2117 => {
                assert!(self.metrics.completion);
                shot = Some("switching-fixture-complete".into());
                self.pending_events = click(self.metrics.probe.unwrap());
            }
            2118 => {
                self.save();
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                let i = self.selected.unwrap();
                assert!(progress::current_solved(
                    &loaded.attempts[&self.scenarios[i].id],
                    &self.scenarios[i]
                ));
                self.progress = loaded;
                shot = Some("switching-fixture-broadcast-inspection".into());
                self.pending_events = click(self.metrics.inputs[2]);
            }
            2119 => {
                self.pending_events = vec![key(egui::Key::A, true), egui::Event::Text("20".into())]
            }
            2120 => {
                assert!(!self.metrics.completion);
                let i = self.selected.unwrap();
                assert!(!progress::current_solved(
                    &self.progress.attempts[&self.scenarios[i].id],
                    &self.scenarios[i]
                ));
                shot = Some("switching-fixture-edited-invalidation".into());
            }
            2121 => {
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.scale = 1.5;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(940., 760.)));
            }
            2122 => shot = Some("switching-fixture-light-150".into()),
            2123 => {
                self.pending_events = vec![
                    egui::Event::PointerMoved(ctx.content_rect().center()),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., -620.),
                        modifiers: Default::default(),
                    },
                ]
            }
            2124 => shot = Some("switching-fixture-light-150-scrolled".into()),
            2125 => {
                let i = self.selected.unwrap();
                self.progress
                    .attempts
                    .get_mut(&self.scenarios[i].id)
                    .unwrap()
                    .challenge = true;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                self.help_open = false;
                self.lesson_scroll_reset = true;
            }
            2126 => shot = Some("switching-fixture-challenge-150".into()),
            2127 => {
                let i = self.selected.unwrap();
                assert_eq!(self.switching_view(i), teaching::View::Assignment);
                assert!(!self.help_open);
                self.reset_attempt(i);
                assert_eq!(
                    self.progress.attempts[&self.scenarios[i].id].state,
                    self.scenarios[i].initial
                );
                println!(
                    "PASS native switching fixture: introduction, isolated example/experiment, wrong retry, keyboard repair, Check, save/reload, edit invalidation, reset and light/dark150 resizing. Development fixture only; bundled catalog unchanged."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            0 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            }
            1 => shot = Some("00-library".into()),
            n @ 2..=37 => {
                let i = (n - 2) / 4;
                match (n - 2) % 4 {
                    0 => self.launch(i),
                    1 => {
                        assert!(!self.metrics.guide);
                        shot = Some(format!("{}-learn", self.scenarios[i].id));
                    }
                    2 => {
                        self.progress
                            .attempts
                            .get_mut(&self.scenarios[i].id)
                            .unwrap()
                            .challenge = true;
                        self.help_open = false;
                        if let Some(view) = &mut self.ipv4_view {
                            view.hide_guidance();
                        }
                    }
                    _ => {
                        assert!(!self.metrics.guide);
                        shot = Some(format!("{}-challenge", self.scenarios[i].id));
                    }
                }
            }
            38 => {
                self.selected = None;
                self.filter.track = "LPIC-2".into();
                self.filter.topic = "Services".into();
            }
            39 => shot = Some("library-topic".into()),
            40 => self.extra_filters = true,
            41 => shot = Some("library-extra-filters".into()),
            42 => {
                self.filter = Filter::default();
                self.extra_filters = false;
                self.launch(0);
                self.progress
                    .attempts
                    .get_mut("ccna-ipv4")
                    .unwrap()
                    .challenge = false;
            }
            43 => {}
            n @ 44..=58 => {
                let index = (n - 44) / 3;
                let answers = [
                    "192.168.10.64",
                    "192.168.10.95",
                    "192.168.10.65",
                    "192.168.10.94",
                    "192.168.10.80",
                ];
                match (n - 44) % 3 {
                    0 => self
                        .pending_events
                        .extend(click(self.metrics.inputs[index])),
                    1 => self.pending_events.extend([
                        key(egui::Key::A, true),
                        egui::Event::Text(answers[index].into()),
                    ]),
                    _ => {}
                }
            }
            59 => self
                .pending_events
                .extend(click(self.metrics.check.unwrap())),
            60 => {
                assert!(self.metrics.completion);
                shot = Some("ipv4-verified-topology".into());
            }
            61 => self.pending_events.extend(click(self.metrics.inputs[4])),
            62 => self.pending_events.extend([
                key(egui::Key::A, true),
                egui::Event::Text("192.168.10.77".into()),
            ]),
            63 => {
                assert!(!self.metrics.completion);
                shot = Some("ipv4-edit-invalidates-success".into());
            }
            64 => self
                .pending_events
                .extend(click(self.metrics.check.unwrap())),
            65 => shot = Some("ipv4-checked-invalid-host".into()),
            66 => self.pending_events.extend(click(self.metrics.why.unwrap())),
            67 => shot = Some("ipv4-binary-explanation".into()),
            68 => {
                self.launch(6);
                self.progress
                    .attempts
                    .get_mut("lpic2-service")
                    .unwrap()
                    .challenge = false;
            }
            69 => self.pending_events.extend(click(self.metrics.inputs[0])),
            70 => self
                .pending_events
                .extend([key(egui::Key::A, true), egui::Event::Text("8080".into())]),
            71 => self
                .pending_events
                .extend(click(self.metrics.editor_save.unwrap())),
            72 => {
                assert!(
                    matches!(&self.progress.attempts["lpic2-service"].state,engine::State::Service{port,running:false,..}if port=="8080")
                );
                for c in [
                    "systemctl restart atlas",
                    "systemctl enable atlas",
                    "systemctl status atlas",
                ] {
                    self.input = c.into();
                    self.submit(6);
                }
            }
            73 => {
                self.checked = true;
                self.progress
                    .attempts
                    .get_mut("lpic2-service")
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .checked = true;
            }
            74 => {
                assert!(self.metrics.completion);
                shot = Some("service-configured-and-verified".into());
            }
            75 => {
                self.launch(5);
                self.progress
                    .attempts
                    .get_mut("lpic1-logs")
                    .unwrap()
                    .challenge = false;
            }
            76 => self
                .pending_events
                .push(egui::Event::Text("grep ERROR /var/log/app.log".into())),
            77 => self.pending_events.push(key(egui::Key::Enter, false)),
            78 => {
                self.checked = true;
                self.progress
                    .attempts
                    .get_mut("lpic1-logs")
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .checked = true;
            }
            79 => {
                assert!(self.metrics.completion);
                shot = Some("terminal-filtered-log".into());
            }
            80 => self
                .pending_events
                .push(egui::Event::Text("cat /var/log/app.log".into())),
            81 => self.pending_events.push(key(egui::Key::Enter, false)),
            82 => {
                assert!(!self.metrics.completion);
                shot = Some("terminal-state-invalidates-success".into());
            }
            83 => {
                self.selected = None;
                self.scale = 1.5;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
            84 => shot = Some("library-150-percent".into()),
            85 => self.launch(0),
            86 => shot = Some("ipv4-150-percent".into()),
            87 => self.launch(6),
            88 => shot = Some("service-150-percent".into()),
            89 => {
                self.selected = None;
                self.scale = 1.;
                self.developer = true;
                self.synthetic = true;
            }
            90 => {
                assert!(self.metrics.pagination);
                shot = Some("developer-large-catalog".into());
            }
            91 => {
                self.synthetic = false;
                self.developer = false;
                self.scale = 1.;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            }
            n @ 92..=175 => {
                let index = (n - 92) / 3;
                match (n - 92) % 3 {
                    0 => {
                        self.theme_id = theme::inventory()[index].id.clone();
                        theme::apply(ctx, &self.theme_id);
                        self.launch(9);
                        self.theme_preview = true;
                    }
                    1 => {}
                    _ => shot = Some(format!("theme-{}", self.theme_id)),
                }
            }
            n @ 176..=231 => {
                let i = 9 + (n - 176) / 8;
                match (n - 176) % 8 {
                    0 => {
                        if i == 9 {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                        }
                        self.theme_preview = false;
                        self.settings_open = false;
                        self.theme_id = "lab-navy".into();
                        theme::apply(ctx, &self.theme_id);
                        self.launch(i);
                        self.set_script_view(i, teaching::View::Assignment);
                        self.progress
                            .attempts
                            .get_mut(&self.scenarios[i].id)
                            .unwrap()
                            .challenge = false;
                    }
                    1 => shot = Some(format!("{}-learn", self.scenarios[i].id)),
                    2 => {
                        self.pending_events.extend([
                            key(egui::Key::A, true),
                            egui::Event::Text(self.scenarios[i].solution[0].clone()),
                        ]);
                    }
                    3 => self
                        .pending_events
                        .extend(click(self.metrics.check.unwrap())),
                    4 => {}
                    5 => {
                        assert!(
                            self.metrics.completion,
                            "{} did not complete",
                            self.scenarios[i].id
                        );
                        shot = Some(format!("{}-complete", self.scenarios[i].id));
                    }
                    6 => {
                        self.progress
                            .attempts
                            .get_mut(&self.scenarios[i].id)
                            .unwrap()
                            .challenge = true
                    }
                    _ => shot = Some(format!("{}-challenge", self.scenarios[i].id)),
                }
            }
            232 => {
                self.focus_input = true;
            }
            233 => self.pending_events.extend([
                key(egui::Key::A, true),
                egui::Event::Text("while true; do :; done".into()),
            ]),
            234 => {
                assert!(!self.metrics.completion);
                self.pending_events
                    .extend(click(self.metrics.check.unwrap()));
            }
            235 => {
                assert!(self.script_job.is_some());
                self.pending_events
                    .extend(click(self.metrics.stop.unwrap()));
            }
            236 => {
                assert!(self.script_job.is_none());
                assert!(!self.metrics.completion);
                shot = Some("shell-stopped-and-invalidated".into());
            }
            237 => {
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.scale = 1.5;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
            238 => shot = Some("shell-light-150-percent".into()),
            239 => {
                self.theme_id = "omarchy-catppuccin".into();
                theme::apply(ctx, &self.theme_id);
            }
            240 => shot = Some("shell-dark-150-percent".into()),
            241 => {
                self.selected = None;
                self.filter = Filter::default();
                self.filter.track = "Bash/Shell".into();
                self.theme_id = "omarchy-catppuccin-latte".into();
                theme::apply(ctx, &self.theme_id);
            }
            242 => shot = Some("shell-library-light-150-percent".into()),
            243 => {
                let scope = if std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("scripts") {
                    "scripting capture pass"
                } else {
                    "full milestone: original nine and all 28 theme previews plus scripting"
                };
                println!(
                    "PASS native {scope}: seven scripting workspaces, typed source and Run, complete behavior cases, Challenge, Stop and invalidation, light/dark 150% resize. Synthetic progress only; injected native input, not physical-device acceptance."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            n @ (250..=375 | 700..=771 | 900..=971 | 1600..=1653 | 1700..=1771 | 1900..=1971) => {
                let (start, first) = if n >= 1900 {
                    (1900, 43)
                } else if n >= 1700 {
                    (1700, 39)
                } else if n >= 1600 {
                    (1600, 36)
                } else if n >= 900 {
                    (900, 20)
                } else if n >= 700 {
                    (700, 16)
                } else {
                    (250, 9)
                };
                let i = first + (n - start) / 18;
                let id = self.scenarios[i].id.clone();
                match (n - start) % 18 {
                    0 => {
                        if i == first {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                        }
                        self.launch(i);
                    }
                    1 => {
                        assert_eq!(self.script_view(i), teaching::View::Introduction);
                        shot = Some(format!("{id}-introduction"));
                    }
                    2 => self
                        .pending_events
                        .extend(click(self.metrics.example_action.unwrap())),
                    3 => {
                        assert_eq!(self.script_view(i), teaching::View::Example);
                        self.pending_events
                            .extend(click(self.metrics.check.unwrap()));
                    }
                    4 => {}
                    5 => {
                        if self.script_job.is_some() {
                            return;
                        }
                        let a = &self.progress.attempts[&id];
                        let l = a
                            .workspace
                            .as_ref()
                            .unwrap()
                            .script_learning
                            .as_ref()
                            .unwrap();
                        assert!(
                            self.scenarios[i]
                                .teaching
                                .as_ref()
                                .unwrap()
                                .fixture
                                .accepts(l.example_result.as_ref().unwrap())
                        );
                        assert_eq!(a.state, self.scenarios[i].initial);
                        assert!(!self.metrics.completion);
                        assert!(self.metrics.editor_viewport.unwrap().height() >= 175.);
                        shot = Some(format!("{id}-example"));
                        self.focus_input = true;
                    }
                    6 => {
                        let lesson = self.scenarios[i].teaching.as_ref().unwrap();
                        let (from, to) = if i >= 43 {
                            [
                                ("visits + 1", "visits + 2"),
                                ("boxes * price + fee", "boxes * (price + fee)"),
                                ("tray=5", "tray=0"),
                                ("total + count", "total + 1"),
                            ][i - 43]
                        } else if i >= 39 {
                            [("1,3", "2,3"), ("-u", "-r"), ("-c", "-d"), ("-d\n", "-u\n")][i - 39]
                        } else if i >= 36 {
                            [
                                ("local title=", "title="),
                                ("loud", "busy"),
                                ("local destination=", "destination="),
                            ][i - 36]
                        } else {
                            [
                                ("Mina", "Rowan"),
                                ("For:", "Recipient:"),
                                ("Note:", "Reminder:"),
                                ("-z", "-n"),
                                ("OPEN", "CLOSED"),
                                ("Reading", "Menu:"),
                                ("-f", "-z"),
                                ("2>", ">"),
                                ("OPEN", "CLOSED"),
                                (r#" || [ -n "$name" ]"#, ""),
                                ("found_problem=yes", "found_problem=''"),
                                ("Rowan Reed", "Tess Ford"),
                                ("return 1", "exit 1"),
                                ("shift\n", "shift 0\n"),
                                (r#"if available "$@""#, "if available menu.txt"),
                            ][i - 9]
                        };
                        self.pending_events.extend([
                            key(egui::Key::A, true),
                            egui::Event::Text(lesson.example.replacen(from, to, 1)),
                        ]);
                    }
                    7 => self
                        .pending_events
                        .extend(click(self.metrics.check.unwrap())),
                    8 => {}
                    9 => {
                        if self.script_job.is_some() {
                            return;
                        }
                        let a = &self.progress.attempts[&id];
                        let result = a
                            .workspace
                            .as_ref()
                            .unwrap()
                            .script_learning
                            .as_ref()
                            .unwrap()
                            .example_result
                            .as_ref()
                            .unwrap();
                        assert!(result.error.is_none());
                        assert!(
                            !self.scenarios[i]
                                .teaching
                                .as_ref()
                                .unwrap()
                                .fixture
                                .accepts(result)
                        );
                        assert_eq!(a.state, self.scenarios[i].initial);
                        self.pending_events
                            .extend(click(self.metrics.assignment_action.unwrap()));
                    }
                    10 => {
                        assert_eq!(self.script_view(i), teaching::View::Assignment);
                        shot = Some(format!("{id}-assignment"));
                    }
                    11 => self.pending_events.extend([
                        key(egui::Key::A, true),
                        egui::Event::Text("printf 'Not the requested output'".into()),
                    ]),
                    12 => self
                        .pending_events
                        .extend(click(self.metrics.check.unwrap())),
                    13 => {
                        if self.script_job.is_some() {
                            return;
                        }
                        assert!(!self.metrics.completion);
                        if i == 9 || i >= 16 {
                            shot = Some(format!("{id}-retry-feedback"));
                        }
                        self.focus_input = true;
                    }
                    14 => self.pending_events.extend([
                        key(egui::Key::A, true),
                        egui::Event::Text(self.scenarios[i].solution[0].clone()),
                    ]),
                    15 => self
                        .pending_events
                        .extend(click(self.metrics.check.unwrap())),
                    16 => {}
                    _ => {
                        if self.script_job.is_some() {
                            return;
                        }
                        assert!(self.metrics.completion, "{id} retry");
                        if i == 9 || i >= 16 {
                            shot = Some(format!("{id}-teaching-complete"));
                        }
                    }
                }
            }
            376 => {
                self.launch(9);
                self.set_script_view(9, teaching::View::Introduction);
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.scale = 1.5;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
            377 => shot = Some("shell-introduction-light-150".into()),
            378 => {
                self.set_script_view(9, teaching::View::Example);
            }
            379 => shot = Some("shell-example-light-150".into()),
            380 => {
                self.theme_id = "omarchy-catppuccin".into();
                theme::apply(ctx, &self.theme_id);
                self.set_script_view(9, teaching::View::Assignment);
            }
            381 => shot = Some("shell-assignment-dark-150".into()),
            382 => {
                self.scale = 1.;
                self.settings_open = true;
                self.theme_preview = true;
            }
            383 => shot = Some("settings-appearance-theme-samples".into()),
            384 => {
                self.save();
                let saved = progress::load(&self.path, &self.scenarios).unwrap();
                self.progress = saved;
                self.launch(9);
                assert_eq!(self.script_view(9), teaching::View::Assignment);
                assert!(engine::solved(
                    &self.progress.attempts["shell-handover"].state,
                    &self.scenarios[9].goal
                ));
                println!(
                    "PASS native teaching walkthrough: fresh profile; seven introductions, example runs and suggested edits, isolated assignment drafts, wrong output and successful retry, save/load resume, light/dark 150% and Settings samples. Injected application events; not a novice user study."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            400 => {
                self.launch(9);
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.set_script_view(9, teaching::View::Introduction);
            }
            401 | 405 => {
                self.pending_events.extend([
                    egui::Event::PointerMoved(egui::pos2(30., 300.)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., -1500.),
                        modifiers: Default::default(),
                    },
                ]);
            }
            402 => {}
            403 => {
                let r = self.metrics.example_action.unwrap();
                assert!(
                    ctx.content_rect().contains(r.center()),
                    "Introduction actions must be reachable by scrolling"
                );
                shot = Some("shell-introduction-light-150-scrolled".into());
                self.pending_events.extend(click(r));
            }
            404 => {
                assert_eq!(self.script_view(9), teaching::View::Example);
            }
            406 => {}
            407 => {
                let r = self.metrics.check.unwrap();
                assert!(
                    ctx.content_rect().contains(r.center()),
                    "Example Run must be reachable by scrolling"
                );
                shot = Some("shell-example-light-150-scrolled".into());
                self.pending_events.extend(click(r));
            }
            408 => {
                assert!(
                    self.progress.attempts["shell-handover"]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .script_learning
                        .as_ref()
                        .unwrap()
                        .example_result
                        .is_some()
                );
            }
            409 => {
                self.pending_events.extend([
                    egui::Event::PointerMoved(egui::pos2(30., 300.)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., -1500.),
                        modifiers: Default::default(),
                    },
                ]);
            }
            410 => {
                shot = Some("shell-example-light-150-output".into());
            }
            411 => {
                self.set_script_view(9, teaching::View::Assignment);
                self.theme_id = "omarchy-catppuccin".into();
                theme::apply(ctx, &self.theme_id);
            }
            412 => {
                self.pending_events.extend([
                    egui::Event::PointerMoved(egui::pos2(30., 300.)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., -1500.),
                        modifiers: Default::default(),
                    },
                ]);
            }
            413 => {}
            414 => {
                let r = self.metrics.check.unwrap();
                assert!(
                    ctx.content_rect().contains(r.center()),
                    "Assignment Run must be reachable by scrolling"
                );
                shot = Some("shell-assignment-dark-150-scrolled".into());
            }
            415 => {
                println!(
                    "PASS native teaching layout: light/dark 150% outer scrolling reaches introduction actions and example/assignment Run controls in a tiled Fedora window; completed example output inspected."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            500 => {
                let _ = std::fs::remove_file(
                    self.path
                        .parent()
                        .unwrap()
                        .join("dlms-learning-backup.json"),
                );
                self.launch(9);
                self.set_script_view(9, teaching::View::Assignment);
                if let engine::State::Script { source, .. } = &mut self
                    .progress
                    .attempts
                    .get_mut("shell-handover")
                    .unwrap()
                    .state
                {
                    *source = "visitor='unfinished draft'".into();
                }
                self.progress.favorites.insert("shell-handover".into());
                self.dirty = true;
                self.settings_open = true;
                self.settings_data = true;
            }
            501 => shot = Some("settings-your-data".into()),
            502 => self
                .pending_events
                .extend(click(self.metrics.data_export.unwrap())),
            503 => {
                shot = Some("export-native-file-picker".into());
                self.pending_events
                    .extend(click(self.metrics.data_file_action.unwrap()));
            }
            504 => {}
            505 => {
                assert!(
                    self.path
                        .parent()
                        .unwrap()
                        .join("dlms-learning-backup.json")
                        .is_file()
                );
                shot = Some("export-backup-saved".into());
                self.reset_attempt(9);
            }
            506 => self
                .pending_events
                .extend(click(self.metrics.data_restore.unwrap())),
            507 => self
                .pending_events
                .extend(click(self.metrics.data_filename.unwrap())),
            508 => self
                .pending_events
                .push(egui::Event::Text("dlms-learning-backup.json".into())),
            509 => self
                .pending_events
                .extend(click(self.metrics.data_file_action.unwrap())),
            510 => {}
            511 => {
                assert!(self.metrics.data_confirm.is_some());
                shot = Some("restore-review-preview".into());
            }
            512 => self
                .pending_events
                .extend(click(self.metrics.data_confirm.unwrap())),
            513 => {}
            514 => {
                assert!(
                    matches!(&self.progress.attempts["shell-handover"].state,engine::State::Script{source,..}if source=="visitor='unfinished draft'")
                );
                assert!(self.progress.favorites.contains("shell-handover"));
                shot = Some("restore-complete-with-safety-copy".into());
            }
            515 => {
                self.settings_open = false;
                let s = &self.scenarios[0];
                let mut a = Attempt::new(s, 1);
                if let engine::State::Subnet {
                    answers, second, ..
                } = &mut a.state
                {
                    *answers = [
                        "192.168.10.64",
                        "192.168.10.95",
                        "192.168.10.65",
                        "192.168.10.94",
                    ]
                    .map(String::from);
                    *second = None;
                }
                self.progress.attempts.insert(s.id.clone(), a);
                self.launch(0);
            }
            516 => {
                assert!(!self.metrics.completion);
                assert!(progress::legacy_credit(
                    &self.progress.attempts["ccna-ipv4"]
                ));
                shot = Some("legacy-analysis-new-requirement".into());
            }
            517 => {
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.scale = 1.5;
                self.settings_open = true;
                self.settings_data = true;
            }
            518 => shot = Some("settings-data-light-150".into()),
            519 => {
                println!(
                    "PASS native recovery: export picker, backup, reset, import typed filename, verified preview, confirmed restore, saved draft/favorite recovery, legacy new-requirement disclosure and light 150% Settings."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            772 => {
                self.launch(18);
                self.set_script_view(18, teaching::View::Assignment);
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.scale = 1.5;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
            773 => shot = Some("shell-composition-light-150".into()),
            774 => {
                self.progress
                    .attempts
                    .get_mut("shell-eof")
                    .unwrap()
                    .challenge = true;
                self.help_open = false;
                self.theme_id = "omarchy-catppuccin".into();
                theme::apply(ctx, &self.theme_id);
            }
            775 => {
                assert_eq!(self.script_view(18), teaching::View::Assignment);
                assert!(self.metrics.lesson_tabs.is_empty());
                shot = Some("shell-composition-challenge-dark-150".into());
            }
            776 => {
                self.pending_events
                    .push(egui::Event::PointerMoved(egui::pos2(500., 500.)));
                self.pending_events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -450.),
                    modifiers: egui::Modifiers::default(),
                });
            }
            777 => {}
            778 => shot = Some("shell-composition-challenge-scrolled-150".into()),
            779 => {
                println!(
                    "PASS native composition: four introductions, editable ungraded examples and suggested changes, assignment drafts, incorrect retry feedback, typed reference solutions and grading, light/dark 150% responsive rendering and independent Challenge. Synthetic profile only."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            972 => {
                self.launch(22);
                self.set_script_view(22, teaching::View::Assignment);
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.scale = 1.5;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
            973 => shot = Some("shell-reusable-light-150".into()),
            974 => {
                self.progress
                    .attempts
                    .get_mut("shell-shift")
                    .unwrap()
                    .challenge = true;
                self.help_open = false;
                self.theme_id = "omarchy-catppuccin".into();
                theme::apply(ctx, &self.theme_id);
            }
            975 => {
                assert_eq!(self.script_view(22), teaching::View::Assignment);
                assert!(self.metrics.lesson_tabs.is_empty());
                shot = Some("shell-reusable-challenge-dark-150".into());
            }
            976 => {
                self.pending_events
                    .push(egui::Event::PointerMoved(egui::pos2(500., 500.)));
                self.pending_events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -450.),
                    modifiers: egui::Modifiers::default(),
                });
            }
            977 => {}
            978 => shot = Some("shell-reusable-challenge-scrolled-150".into()),
            979 => {
                println!(
                    "PASS native reusable functions: four introductions, editable ungraded examples and suggested changes, assignment drafts, incorrect retry feedback, typed reference solutions and grading, light/dark 150% responsive rendering and independent Challenge. Synthetic profile only."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            1800 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.launch(9);
                self.set_script_view(9, teaching::View::Assignment);
            }
            1801 | 1807 | 1813 | 1819 => self.pending_events.extend(click(self.metrics.inputs[0])),
            1802 | 1820 => {
                let source = format!(
                    "total=0\nfor amount in 2 3; do total=$((total+amount)); done\nif [ \"$total\" -ne 5 ]; then exit 2; fi\n{}",
                    self.scenarios[9].solution[0]
                );
                self.pending_events
                    .extend([key(egui::Key::A, true), egui::Event::Text(source)]);
            }
            1803 | 1809 | 1815 | 1821 => self
                .pending_events
                .extend(click(self.metrics.check.unwrap())),
            1804 | 1805 | 1810 | 1811 | 1822 | 1823 => {}
            1806 => {
                assert!(self.metrics.completion);
                shot = Some("arithmetic-native-current-run".into());
            }
            1808 => self.pending_events.extend([
                key(egui::Key::A, true),
                egui::Event::Text(format!("total=$((1/0))\n{}", self.scenarios[9].solution[0])),
            ]),
            1812 => {
                assert!(!self.metrics.completion);
                shot = Some("arithmetic-native-division-feedback".into());
            }
            1814 => self.pending_events.extend([
                key(egui::Key::A, true),
                egui::Event::Text("total=0\nwhile true; do total=$((total+1)); done".into()),
            ]),
            1816 => {
                assert!(self.script_job.is_some());
                self.pending_events
                    .extend(click(self.metrics.stop.unwrap()));
            }
            1817 => {}
            1818 => {
                assert!(self.script_job.is_none());
                assert!(!self.metrics.completion);
                shot = Some("arithmetic-native-stopped-counter".into());
            }
            1824 => {
                assert!(self.metrics.completion);
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert!(progress::current_solved(
                    &self.progress.attempts["shell-handover"],
                    &self.scenarios[9]
                ));
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.scale = 1.5;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 840.)));
            }
            1825 => shot = Some("arithmetic-native-light-150".into()),
            1826 => {
                println!(
                    "PASS native arithmetic capability: keyboard sum/guard/current grading, division feedback/current invalidation, responsive counter Stop, repair/reload and light150 rendering. Synthetic developer script in an existing lab; no new curriculum or novice acceptance claimed."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            2090 => {
                self.selected = None;
                self.filter = Filter {
                    track: "LPIC-1".into(),
                    unit: "linux-access-policy".into(),
                    ..Default::default()
                };
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            }
            2091 => shot = Some("access-unit-library".into()),
            2092 => {
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 840.)));
            }
            2093 => shot = Some("access-unit-library-light-150".into()),
            2094 => {
                self.pending_events
                    .push(egui::Event::PointerMoved(egui::pos2(500., 500.)));
                self.pending_events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -500.),
                    modifiers: Default::default(),
                });
            }
            2095 => {}
            2096 => shot = Some("access-unit-library-scrolled-150".into()),
            2097 => {
                println!(
                    "PASS native access-unit library: certification/unit selection, large-text light/dark responsive cards and clean scrolling. Render check; no desktop platform acceptance implied."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            2072 => {
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                for i in 47..51 {
                    assert!(!progress::current_solved(
                        &self.progress.attempts[&self.scenarios[i].id],
                        &self.scenarios[i]
                    ));
                }
                self.launch(47);
                self.set_terminal_view(47, teaching::View::Introduction);
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.scale = 1.5;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 840.)));
            }
            2073 => shot = Some("access-introduction-light-150".into()),
            2074 => self.set_terminal_view(47, teaching::View::Example),
            2075 => shot = Some("access-practice-light-150".into()),
            2076 => {
                self.set_terminal_view(47, teaching::View::Assignment);
                self.progress
                    .attempts
                    .get_mut("lpic1-team-brief")
                    .unwrap()
                    .challenge = true;
                self.theme_id = "omarchy-catppuccin".into();
                theme::apply(ctx, &self.theme_id);
            }
            2077 => {
                assert!(self.metrics.lesson_tabs.is_empty());
                shot = Some("access-challenge-dark-150".into());
            }
            2078 => {
                self.pending_events
                    .push(egui::Event::PointerMoved(egui::pos2(500., 500.)));
                self.pending_events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -650.),
                    modifiers: Default::default(),
                });
            }
            2079 => {}
            2080 => shot = Some("access-challenge-scrolled-150".into()),
            2081 => {
                println!(
                    "PASS native Linux access: four introductions, keyboard first-example commands, isolated practice outcomes, partial-policy feedback, normal command repairs/Check, success invalidation, saved reload and light/dark150 scrolling/Challenge. Synthetic agent input; not novice, real multiuser or Windows/macOS desktop acceptance."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            1980 => {
                self.launch(46);
                self.set_script_view(46, teaching::View::Assignment);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            }
            1981 | 1987 => self.pending_events.extend(click(self.metrics.inputs[0])),
            1982 => self.pending_events.extend([
                key(egui::Key::A, true),
                egui::Event::Text("while true; do :; done < alpha.txt".into()),
            ]),
            1983 | 1989 => self
                .pending_events
                .extend(click(self.metrics.check.unwrap())),
            1984 => {
                assert!(self.script_job.is_some());
                self.pending_events
                    .extend(click(self.metrics.stop.unwrap()));
            }
            1985 => {}
            1986 => {
                assert!(self.script_job.is_none());
                assert!(!self.metrics.completion);
                shot = Some("shell-counts-loop-stopped".into());
            }
            1988 => self.pending_events.extend([
                key(egui::Key::A, true),
                egui::Event::Text(self.scenarios[46].solution[0].clone()),
            ]),
            1990 => {
                if self.script_job.is_some() {
                    return;
                }
            }
            1991 => {
                assert!(self.metrics.completion);
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert!(progress::current_solved(
                    &self.progress.attempts["shell-count-capstone"],
                    &self.scenarios[46]
                ));
                shot = Some("shell-counts-after-stop-repaired".into());
            }
            1992 => {
                println!(
                    "PASS native while-file input: responsive Stop on an infinite file loop, no credit, keyboard repair/Run through all eight inputs, current success and saved reload. Synthetic profile only."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            1972 => {
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                for i in 43..47 {
                    assert!(progress::current_solved(
                        &self.progress.attempts[&self.scenarios[i].id],
                        &self.scenarios[i]
                    ));
                }
                self.launch(46);
                self.set_script_view(46, teaching::View::Assignment);
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.scale = 1.5;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 840.)));
            }
            1973 => shot = Some("shell-counts-light-150".into()),
            1974 => {
                self.progress
                    .attempts
                    .get_mut("shell-count-capstone")
                    .unwrap()
                    .challenge = true;
                self.help_open = false;
                self.theme_id = "omarchy-catppuccin".into();
                theme::apply(ctx, &self.theme_id);
            }
            1975 => {
                assert!(self.metrics.lesson_tabs.is_empty());
                shot = Some("shell-counts-challenge-dark-150".into());
            }
            1976 => {
                self.pending_events
                    .push(egui::Event::PointerMoved(egui::pos2(500., 500.)));
                self.pending_events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -650.),
                    modifiers: Default::default(),
                });
            }
            1977 => {}
            1978 => shot = Some("shell-counts-challenge-scrolled-150".into()),
            1979 => {
                if let engine::State::Script { source, .. } = &mut self
                    .progress
                    .attempts
                    .get_mut("shell-count-capstone")
                    .unwrap()
                    .state
                {
                    source.push_str("\nprintf broken");
                }
                assert!(!progress::current_solved(
                    &self.progress.attempts["shell-count-capstone"],
                    &self.scenarios[46]
                ));
                println!(
                    "PASS native counters/totals: four introductions, isolated example Runs/edits, full assignments, wrong retries, keyboard repair/current Run, saved reload, invalidation and light/dark150 scrolling. Synthetic profile only; no novice or Windows/macOS desktop acceptance."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            1772 => {
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                for i in 39..43 {
                    assert!(progress::current_solved(
                        &self.progress.attempts[&self.scenarios[i].id],
                        &self.scenarios[i]
                    ));
                }
                self.launch(41);
                self.set_script_view(41, teaching::View::Assignment);
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.scale = 1.5;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 840.)));
            }
            1773 => shot = Some("shell-text-light-150".into()),
            1774 => {
                self.progress
                    .attempts
                    .get_mut("shell-uniq-counts")
                    .unwrap()
                    .challenge = true;
                self.help_open = false;
                self.theme_id = "omarchy-catppuccin".into();
                theme::apply(ctx, &self.theme_id);
            }
            1775 => {
                assert!(self.metrics.lesson_tabs.is_empty());
                shot = Some("shell-text-challenge-dark-150".into());
            }
            1776 => {
                self.pending_events
                    .push(egui::Event::PointerMoved(egui::pos2(500., 500.)));
                self.pending_events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -650.),
                    modifiers: Default::default(),
                });
            }
            1777 => {}
            1778 => shot = Some("shell-text-challenge-scrolled-150".into()),
            1779 => {
                if let engine::State::Script { source, .. } = &mut self
                    .progress
                    .attempts
                    .get_mut("shell-uniq-counts")
                    .unwrap()
                    .state
                {
                    source.push_str("\nprintf broken");
                }
                assert!(!progress::current_solved(
                    &self.progress.attempts["shell-uniq-counts"],
                    &self.scenarios[41]
                ));
                println!(
                    "PASS native text processing: four introductions, isolated example Runs/edits, full assignments, wrong retries, keyboard repair/current Run, saved reload, invalidation and light/dark150 scrolling. Synthetic profile only; no novice or Windows/macOS desktop acceptance."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            1654 => {
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                for i in 36..39 {
                    assert!(progress::current_solved(
                        &self.progress.attempts[&self.scenarios[i].id],
                        &self.scenarios[i]
                    ));
                }
                self.launch(38);
                self.set_script_view(38, teaching::View::Assignment);
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.scale = 1.5;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 840.)));
            }
            1655 => shot = Some("shell-scope-light-150".into()),
            1656 => {
                self.progress
                    .attempts
                    .get_mut("shell-scope-capstone")
                    .unwrap()
                    .challenge = true;
                self.help_open = false;
                self.theme_id = "omarchy-catppuccin".into();
                theme::apply(ctx, &self.theme_id);
            }
            1657 => {
                assert!(self.metrics.lesson_tabs.is_empty());
                shot = Some("shell-scope-challenge-dark-150".into());
            }
            1658 => {
                self.pending_events
                    .push(egui::Event::PointerMoved(egui::pos2(500., 500.)));
                self.pending_events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -650.),
                    modifiers: Default::default(),
                });
            }
            1659 => {}
            1660 => shot = Some("shell-scope-challenge-scrolled-150".into()),
            1661 => {
                if let engine::State::Script { source, .. } = &mut self
                    .progress
                    .attempts
                    .get_mut("shell-scope-capstone")
                    .unwrap()
                    .state
                {
                    source.push_str("\nprintf broken");
                }
                assert!(!progress::current_solved(
                    &self.progress.attempts["shell-scope-capstone"],
                    &self.scenarios[38]
                ));
                println!(
                    "PASS native Bash scope: three introductions, isolated runnable examples and exact suggested edits, full assignments, incorrect retries, keyboard reference repair/current Run, saved reload, invalidation and light/dark150 scrolling. Synthetic profile only; not novice or Windows/macOS desktop acceptance."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            1500 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.launch(9);
                let s = self.scenarios[9].clone();
                let engine::Goal::Script { cases } = &s.goal else {
                    panic!()
                };
                let original = s.solution[0].clone();
                let source = format!("IFS=,\n{original}");
                self.progress.attempts.get_mut(&s.id).unwrap().state = engine::State::Script {
                    source: source.clone(),
                    run: Some(shell::Record {
                        engine_revision: 0,
                        source,
                        results: cases.iter().map(|c| shell::run(&original, c)).collect(),
                    }),
                };
                model::prepare(self.progress.attempts.get_mut(&s.id).unwrap());
                // Model a second upgrade: an older failed history and a later
                // earned result must both survive current verification.
                let a = self.progress.attempts.get_mut(&s.id).unwrap();
                a.workspace
                    .as_mut()
                    .unwrap()
                    .script_history
                    .as_mut()
                    .unwrap()
                    .results[0]
                    .stdout = "Earlier incomplete output".into();
                a.state = engine::State::Script {
                    source: original.clone(),
                    run: Some(shell::Record {
                        engine_revision: 1,
                        source: original.clone(),
                        results: cases.iter().map(|c| shell::run(&original, c)).collect(),
                    }),
                };
                self.launch(9);
                self.set_script_view(9, teaching::View::Assignment);
                self.selected = None;
                self.filter.track = "Bash/Shell".into();
                self.filter.search = s.title;
            }
            1501 => shot = Some("script-earlier-credit-library".into()),
            1502 => self.launch(9),
            1503 => {
                assert!(!self.metrics.completion);
                assert!(progress::legacy_script_credit(
                    &self.progress.attempts["shell-handover"],
                    &self.scenarios[9]
                ));
                shot = Some("script-earlier-output-retained".into());
            }
            1504 => {
                // The saved revision1 program was valid. An edit now introduces
                // unsupported syntax; earlier earned credit remains separate.
                if let engine::State::Script { source, run } = &mut self
                    .progress
                    .attempts
                    .get_mut("shell-handover")
                    .unwrap()
                    .state
                {
                    *source = format!("IFS=,\n{source}");
                    *run = None;
                }
                self.pending_events
                    .extend(click(self.metrics.check.unwrap()));
            }
            1505 | 1506 => {}
            1507 => {
                assert!(!self.metrics.completion);
                assert!(self.script_job.is_none());
                shot = Some("script-current-simulator-retry".into());
            }
            1508 => {
                let engine::State::Script { source, run } = &mut self
                    .progress
                    .attempts
                    .get_mut("shell-handover")
                    .unwrap()
                    .state
                else {
                    panic!()
                };
                *source =
                    "scope_test() { local label=inside; while true; do :; done; }\nscope_test"
                        .into();
                *run = None;
            }
            1509 => self
                .pending_events
                .extend(click(self.metrics.check.unwrap())),
            1510 => {
                assert!(self.script_job.is_some());
                self.pending_events
                    .extend(click(self.metrics.stop.unwrap()));
            }
            1511 => {
                assert!(self.script_job.is_none());
                assert!(!self.metrics.completion);
                shot = Some("script-scoped-loop-stopped".into());
            }
            1512 => self.pending_events.extend(click(self.metrics.inputs[0])),
            1513 => self.pending_events.extend([
                key(egui::Key::A, true),
                egui::Event::Text(self.scenarios[9].solution[0].clone()),
            ]),
            1514 => self
                .pending_events
                .extend(click(self.metrics.check.unwrap())),
            1515 | 1516 => {}
            1517 => {
                assert!(self.metrics.completion);
                assert!(progress::legacy_script_credit(
                    &self.progress.attempts["shell-handover"],
                    &self.scenarios[9]
                ));
                shot = Some("script-current-run-and-earlier-history".into());
            }
            1518 => {
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert!(progress::current_solved(
                    &self.progress.attempts["shell-handover"],
                    &self.scenarios[9]
                ));
                self.selected = None;
            }
            1519 => shot = Some("script-current-completion-library".into()),
            1520 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 840.)));
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.launch(9);
            }
            1521 => shot = Some("script-history-light-150".into()),
            1522 => {
                self.pending_events
                    .push(egui::Event::PointerMoved(egui::pos2(400., 400.)));
                self.pending_events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -900.),
                    modifiers: Default::default(),
                });
            }
            1523 => {}
            1524 => shot = Some("script-history-scrolled-light-150".into()),
            1525 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.scale = 1.;
                self.theme_id = "lab-navy".into();
                theme::apply(ctx, &self.theme_id);
                self.launch(9);
            }
            1526 => {}
            1527 => self
                .pending_events
                .extend(click(self.metrics.script_history.unwrap())),
            1528 => shot = Some("script-successive-history-open".into()),
            1529 => {
                assert_eq!(
                    self.progress.attempts["shell-handover"]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .script_history_more
                        .len(),
                    1
                );
                println!(
                    "PASS native evaluator upgrade: earlier script/output/credit retained separately; current unsupported retry, scoped-loop Stop, keyboard repair/current Run, saved reload and light150 scroll. Synthetic legacy fixture only; no personal progress or novice-user acceptance."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            n @ 1300..=1407 => {
                let i = 30 + (n - 1300) / 18;
                let id = self.scenarios[i].id.clone();
                match (n - 1300) % 18 {
                    0 => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                        self.launch(i);
                    }
                    1 => {
                        assert_eq!(self.ipv6_view(i), teaching::View::Introduction);
                        shot = Some(format!("{id}-introduction"));
                    }
                    2 => self.set_ipv6_view(i, teaching::View::Example),
                    3 => {
                        let lesson = self.scenarios[i].ipv6_teaching.clone().unwrap();
                        self.progress
                            .attempts
                            .get_mut(&id)
                            .unwrap()
                            .workspace
                            .as_mut()
                            .unwrap()
                            .ipv6_learning
                            .as_mut()
                            .unwrap()
                            .fill_example(&lesson)
                            .unwrap();
                    }
                    4 => self
                        .pending_events
                        .extend(click(self.metrics.check.unwrap())),
                    5 => {
                        let a = &self.progress.attempts[&id];
                        let l = a
                            .workspace
                            .as_ref()
                            .unwrap()
                            .ipv6_learning
                            .as_ref()
                            .unwrap();
                        assert!(
                            l.checks(self.scenarios[i].ipv6_teaching.as_ref().unwrap())
                                .iter()
                                .all(|(p, _)| *p)
                        );
                        assert_eq!(a.state, self.scenarios[i].initial);
                        shot = Some(format!("{id}-practice"));
                    }
                    6 => self
                        .pending_events
                        .extend(click(self.metrics.assignment_action.unwrap())),
                    7 => {
                        assert_eq!(self.ipv6_view(i), teaching::View::Assignment);
                        shot = Some(format!("{id}-assignment"));
                    }
                    8 => {
                        if i == 30 {
                            self.pending_events
                                .push(egui::Event::Text("2001:db8:44::23/64".into()));
                        } else {
                            self.pending_events
                                .extend(click(self.metrics.check.unwrap()));
                        }
                    }
                    9 => {
                        if i != 30 {
                            assert!(!self.metrics.completion);
                            shot = Some(format!("{id}-retry"));
                        }
                    }
                    10 => {
                        if i != 30 {
                            let engine::State::Ipv6 { mut topology } =
                                self.scenarios[i].initial.clone()
                            else {
                                panic!()
                            };
                            for action in &self.scenarios[i].solution {
                                dlms_learning_lab::ipv6_workspace::adapter(&mut topology, action)
                                    .unwrap();
                            }
                            let draft = self
                                .progress
                                .attempts
                                .get_mut(&id)
                                .unwrap()
                                .workspace
                                .as_mut()
                                .unwrap()
                                .ipv6_draft
                                .as_mut()
                                .unwrap();
                            let selected = draft.selected_device.clone();
                            *draft = dlms_learning_lab::ipv6_workspace::Draft::new(&topology);
                            draft.selected_device = selected;
                        }
                    }
                    11 => self
                        .pending_events
                        .extend(click(self.metrics.check.unwrap())),
                    12 => {
                        assert!(self.metrics.completion, "native {id} completion");
                        shot = Some(format!("{id}-complete"));
                    }
                    13 => {
                        let draft = self
                            .progress
                            .attempts
                            .get_mut(&id)
                            .unwrap()
                            .workspace
                            .as_mut()
                            .unwrap()
                            .ipv6_draft
                            .as_mut()
                            .unwrap();
                        for device in draft.devices.values_mut() {
                            for iface in device.interfaces.values_mut() {
                                iface.up = false;
                            }
                        }
                    }
                    14 => {
                        assert!(!self.metrics.completion);
                        assert!(!progress::current_solved(
                            &self.progress.attempts[&id],
                            &self.scenarios[i]
                        ));
                        self.save();
                    }
                    15 => {
                        self.launch(9);
                        self.launch(i);
                    }
                    16 => {
                        assert!(!progress::current_solved(
                            &self.progress.attempts[&id],
                            &self.scenarios[i]
                        ));
                    }
                    _ => self.selected = None,
                }
            }
            1408 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 840.)));
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.launch(31);
                self.reset_attempt(31);
                self.set_ipv6_view(31, teaching::View::Introduction);
            }
            1409 => shot = Some("ipv6-introduction-light-150".into()),
            1410 => self.set_ipv6_view(31, teaching::View::Assignment),
            1411 => shot = Some("ipv6-address-light-150".into()),
            1412 => {
                self.progress
                    .attempts
                    .get_mut("ccna-ipv6-workstation")
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .ipv6_draft
                    .as_mut()
                    .unwrap()
                    .show_routes = true;
            }
            1413 => shot = Some("ipv6-routes-light-150".into()),
            1414 => {
                self.pending_events
                    .push(egui::Event::PointerMoved(egui::pos2(400., 400.)));
                self.pending_events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -900.),
                    modifiers: Default::default(),
                });
            }
            1415 => {}
            1416 => shot = Some("ipv6-routes-scrolled-light-150".into()),
            1417 => {
                let r = self.metrics.check.unwrap();
                assert!(r.top() >= 0. && r.bottom() < ctx.content_rect().bottom());
                self.pending_events.extend(click(r));
            }
            1418 => {
                assert!(self.checked);
                assert!(!self.metrics.completion);
                shot = Some("ipv6-retry-scrolled-light-150".into());
            }
            1419 => {
                self.progress
                    .attempts
                    .get_mut("ccna-ipv6-workstation")
                    .unwrap()
                    .challenge = true;
            }
            1420 => {
                assert!(self.metrics.lesson_tabs.is_empty());
                shot = Some("ipv6-challenge-light-150".into());
            }
            1421 => {
                if std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("ipv6-layout") {
                    println!(
                        "PASS native IPv6 layout: resized light150 address/route views, scrolled Check action and retry feedback, independent Challenge. Synthetic profile only."
                    );
                } else {
                    println!(
                        "PASS native IPv6: six introductions, separate example draft/apply checks, assignment error/retry and reference-draft checks, manual keyboard address in first lab, invalidation and resume. Resized light150 address/route scrolling and Challenge. Synthetic profile only; not novice-user or physical networking acceptance."
                    );
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            n @ 2000..=2071 => {
                let ordinal = (n - 2000) / 18;
                let i = 47 + ordinal;
                let id = self.scenarios[i].id.clone();
                match (n - 2000) % 18 {
                    0 => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                        self.launch(i);
                    }
                    1 => {
                        assert_eq!(self.terminal_view(i), teaching::View::Introduction);
                        shot = Some(format!("{id}-introduction"));
                    }
                    2 => self.set_terminal_view(i, teaching::View::Example),
                    3 => {
                        let command = self.scenarios[i]
                            .terminal_teaching
                            .as_ref()
                            .unwrap()
                            .example[0]
                            .clone();
                        self.pending_events.push(egui::Event::Text(command));
                        self.pending_events.push(key(egui::Key::Enter, false));
                    }
                    4 => {}
                    5 => {
                        let a = &self.progress.attempts[&id];
                        let l = a
                            .workspace
                            .as_ref()
                            .unwrap()
                            .terminal_learning
                            .as_ref()
                            .unwrap();
                        assert_eq!(l.transcript.len(), 1);
                        assert_eq!(a.state, self.scenarios[i].initial);
                        assert!(!engine::solved(&a.state, &self.scenarios[i].goal));
                    }
                    6 => {
                        let commands = self.scenarios[i]
                            .terminal_teaching
                            .as_ref()
                            .unwrap()
                            .example
                            .clone();
                        let l = self
                            .progress
                            .attempts
                            .get_mut(&id)
                            .unwrap()
                            .workspace
                            .as_mut()
                            .unwrap()
                            .terminal_learning
                            .as_mut()
                            .unwrap();
                        for command in commands.into_iter().skip(1) {
                            l.command = command;
                            l.execute();
                        }
                        for requirement in &self.scenarios[i]
                            .terminal_teaching
                            .as_ref()
                            .unwrap()
                            .outcomes
                        {
                            assert!(requirement.check(&l.tree).0);
                        }
                    }
                    7 => shot = Some(format!("{id}-practice")),
                    8 => self.set_terminal_view(i, teaching::View::Assignment),
                    9 => {
                        let mistakes: &[&str] = match ordinal {
                            0 => &["chmod 644 brief.txt"],
                            1 => &["chmod g=w drop"],
                            2 => &["cp incoming/roster-old.txt publish/roster.txt"],
                            _ => &[
                                "chmod u+w staging",
                                "mv staging/ready.txt staging/delivered.txt",
                            ],
                        };
                        for command in mistakes {
                            self.input = (*command).into();
                            self.submit(i);
                        }
                        self.checked = true;
                        self.progress
                            .attempts
                            .get_mut(&id)
                            .unwrap()
                            .workspace
                            .as_mut()
                            .unwrap()
                            .checked = true;
                    }
                    10 => {
                        assert!(!self.metrics.completion);
                        shot = Some(format!("{id}-assignment-retry"));
                    }
                    11 => {
                        let commands = if ordinal == 3 {
                            vec!["chmod u-w staging".into()]
                        } else {
                            self.scenarios[i].solution.clone()
                        };
                        for command in commands {
                            self.input = command;
                            self.submit(i);
                        }
                    }
                    12 => self
                        .pending_events
                        .extend(click(self.metrics.check.unwrap())),
                    13 => {
                        assert!(self.metrics.completion);
                        shot = Some(format!("{id}-complete"));
                    }
                    14 => {
                        self.input = [
                            "chmod o+r brief.txt",
                            "chmod g+r drop",
                            "chmod u+w publish",
                            "chmod u+w staging",
                        ][ordinal]
                            .into();
                        self.submit(i);
                    }
                    15 => {
                        assert!(!self.metrics.completion);
                    }
                    16 => {
                        self.save();
                    }
                    _ => self.selected = None,
                }
            }
            n @ 1100..=1207 => {
                let ordinal = (n - 1100) / 18;
                let i = self
                    .scenarios
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.terminal_teaching.is_some())
                    .nth(ordinal)
                    .unwrap()
                    .0;
                let id = self.scenarios[i].id.clone();
                match (n - 1100) % 18 {
                    0 => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                        self.launch(i);
                    }
                    1 => {
                        assert_eq!(self.terminal_view(i), teaching::View::Introduction);
                        shot = Some(format!("{id}-introduction"));
                    }
                    2 => self.set_terminal_view(i, teaching::View::Example),
                    3 => {
                        let command = self.scenarios[i]
                            .terminal_teaching
                            .as_ref()
                            .unwrap()
                            .example[0]
                            .clone();
                        self.pending_events.push(egui::Event::Text(command));
                        self.pending_events.push(key(egui::Key::Enter, false));
                    }
                    4 => {}
                    5 => {
                        let a = &self.progress.attempts[&id];
                        let l = a
                            .workspace
                            .as_ref()
                            .unwrap()
                            .terminal_learning
                            .as_ref()
                            .unwrap();
                        assert_eq!(l.transcript.len(), 1);
                        assert_eq!(a.state, self.scenarios[i].initial);
                        assert!(!engine::solved(&a.state, &self.scenarios[i].goal));
                    }
                    6 => {
                        let commands = self.scenarios[i]
                            .terminal_teaching
                            .as_ref()
                            .unwrap()
                            .example
                            .clone();
                        let l = self
                            .progress
                            .attempts
                            .get_mut(&id)
                            .unwrap()
                            .workspace
                            .as_mut()
                            .unwrap()
                            .terminal_learning
                            .as_mut()
                            .unwrap();
                        for command in commands.into_iter().skip(1) {
                            l.command = command;
                            l.execute();
                        }
                        for requirement in &self.scenarios[i]
                            .terminal_teaching
                            .as_ref()
                            .unwrap()
                            .outcomes
                        {
                            assert!(requirement.check(&l.tree).0);
                        }
                    }
                    7 => shot = Some(format!("{id}-practice")),
                    8 => self.set_terminal_view(i, teaching::View::Assignment),
                    9 => {
                        self.input = "cat absent-file".into();
                        self.submit(i);
                    }
                    10 => {
                        assert!(!self.metrics.completion);
                        shot = Some(format!("{id}-assignment-retry"));
                    }
                    11 => {
                        for command in self.scenarios[i].solution.clone() {
                            self.input = command;
                            self.submit(i);
                        }
                    }
                    12 => {
                        self.checked = true;
                        self.progress
                            .attempts
                            .get_mut(&id)
                            .unwrap()
                            .workspace
                            .as_mut()
                            .unwrap()
                            .checked = true;
                    }
                    13 => {
                        assert!(self.metrics.completion);
                        shot = Some(format!("{id}-complete"));
                    }
                    14 => {
                        self.input = [
                            "rm ready.txt",
                            "rm 'release notes/notes.txt'",
                            "rm archive/snapshot.txt",
                            "touch staging/draft.tmp",
                            "umask 022",
                            "rm ../private/delivery.txt",
                        ][ordinal]
                            .into();
                        self.submit(i);
                    }
                    15 => {
                        assert!(!self.metrics.completion);
                    }
                    16 => {
                        self.save();
                    }
                    _ => self.selected = None,
                }
            }
            1208 => {}
            1209 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 840.)));
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.launch(24);
                self.set_terminal_view(24, teaching::View::Introduction);
            }
            1210 => shot = Some("files-introduction-light-150".into()),
            1211 => self.set_terminal_view(24, teaching::View::Example),
            1212 => shot = Some("files-practice-light-150".into()),
            1213 => {
                self.progress
                    .attempts
                    .get_mut("lpic1-file-navigation")
                    .unwrap()
                    .challenge = true;
            }
            1214 => {
                assert!(self.metrics.lesson_tabs.is_empty());
                shot = Some("files-challenge-light-150".into());
            }
            1215 => {
                self.pending_events
                    .push(egui::Event::PointerMoved(egui::pos2(500., 500.)));
                self.pending_events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -450.),
                    modifiers: Default::default(),
                });
            }
            1216 => {}
            1217 => shot = Some("files-challenge-scrolled-light-150".into()),
            1218 => {
                if std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("files-layout") {
                    println!(
                        "PASS native file layout: focused practice input and resized light150 Challenge with optional help closed. Synthetic profile only."
                    );
                } else {
                    println!(
                        "PASS native file foundations: six introductions, keyboard practice command, isolated example outcomes, assignment error/retry, all state solutions, success invalidation and saved drafts. Light150 resized Challenge with optional help closed. Synthetic profile only; not novice-user acceptance."
                    );
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            600 => {
                self.filter.track = "Bash/Shell".into();
            }
            601 => {
                assert!(self.metrics.course_start.is_some());
                shot = Some("course-shell-entry".into());
            }
            602 => self
                .pending_events
                .extend(click(self.metrics.course_start.unwrap())),
            603 => {
                assert_eq!(self.selected, Some(9));
                assert_eq!(self.script_view(9), teaching::View::Introduction);
                shot = Some("course-start-opens-introduction".into());
            }
            604 => self
                .pending_events
                .extend(click(self.metrics.back.unwrap())),
            605 => {
                assert_eq!(self.filter.track, "Bash/Shell");
                self.filter.unit = "shell-foundations".into();
            }
            606 => shot = Some("unit-shell-foundations".into()),
            607 => {
                self.filter.track = "LPIC-2".into();
                self.filter.unit = "linux-service-recovery".into();
            }
            608 => shot = Some("unit-linux-prerequisite-guidance".into()),
            609 => {
                for i in 0..3 {
                    let s = &self.scenarios[i];
                    let mut a = Attempt::new(s, 1);
                    for command in &s.solution {
                        engine::execute(&mut a.state, command).unwrap();
                    }
                    self.progress.attempts.insert(s.id.clone(), a);
                }
                self.launch(2);
                self.checked = true;
                self.progress
                    .attempts
                    .get_mut("ccna-gateway")
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .checked = true;
            }
            610 => {
                assert!(self.metrics.completion);
                shot = Some("unit-completion-no-cross-track-jump".into());
            }
            611 => {
                self.selected = None;
                self.filter = Filter {
                    unit: "shell-foundations".into(),
                    ..Default::default()
                };
                self.developer = true;
                self.synthetic = true;
            }
            612 => shot = Some("synthetic-unit-filter-navigation".into()),
            613 => {
                self.synthetic = false;
                self.developer = false;
                self.filter = Filter {
                    track: "Bash/Shell".into(),
                    unit: "shell-foundations".into(),
                    ..Default::default()
                };
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.scale = 1.5;
            }
            614 => shot = Some("unit-library-light-150".into()),
            615 => {
                println!(
                    "PASS native courses: course entry opens Learn introduction, return preserves track, units and advisory prerequisites, unit completion stays in track, synthetic unit filter and light 150% library."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => {}
        }
        if let Some(name) = shot {
            self.capture_pending = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(name)));
        }
        self.smoke_stage += 1;
        self.started = Instant::now();
    }
}
