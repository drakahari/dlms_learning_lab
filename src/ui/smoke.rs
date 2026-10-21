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
                println!(
                    "PASS native normal app: all nine Learn/Challenge workspaces captured; actual IPv4 field editing/check/invalidation; service editor save and modeled restart/enable; terminal Enter/filter/invalidation; real resize, 150% text and synthetic catalog. Injected input, not a physical-device acceptance claim."
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
