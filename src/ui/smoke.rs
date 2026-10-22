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
            n @ 250..=375 => {
                let i = 9 + (n - 250) / 18;
                let id = self.scenarios[i].id.clone();
                match (n - 250) % 18 {
                    0 => {
                        if i == 9 {
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
                        let (from, to) = [
                            ("Mina", "Rowan"),
                            ("For:", "Recipient:"),
                            ("Note:", "Reminder:"),
                            ("-z", "-n"),
                            ("OPEN", "CLOSED"),
                            ("Reading", "Menu:"),
                            ("-f", "-z"),
                        ][i - 9];
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
                        assert!(!self.metrics.completion);
                        if i == 9 {
                            shot = Some("shell-handover-retry-feedback".into());
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
                        assert!(self.metrics.completion, "{id} retry");
                        if i == 9 {
                            shot = Some("shell-handover-teaching-complete".into());
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
