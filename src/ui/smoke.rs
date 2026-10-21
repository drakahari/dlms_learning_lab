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
                let dir = std::path::Path::new(if self.ipv4_smoke {
                    "screenshots/ipv4"
                } else {
                    "screenshots/after"
                });
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
        if self.capture_pending || self.started.elapsed().as_millis() < 500 {
            return;
        }
        let key = |key, shift| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                shift,
                ..Default::default()
            },
        };
        let mut shot = None;
        match self.smoke_stage {
            0 => {
                assert!(!self.metrics.pagination);
                shot = Some("01-library");
            }
            1 => {
                self.smoke_width = ctx.input(|i| i.content_rect().width()) * ctx.zoom_factor();
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            }
            2 => {
                assert!(
                    (ctx.input(|i| i.content_rect().width()) * ctx.zoom_factor()
                        - self.smoke_width)
                        .abs()
                        > 10.
                );
                self.launch(0);
            }
            3 => {
                assert_eq!(self.metrics.inputs.len(), 4);
                self.pending_events
                    .push(egui::Event::Text("999.1.1.1".into()));
            }
            4 => {
                assert!(
                    matches!(&self.progress.attempts["ccna-ipv4"].state,engine::State::Subnet{answers,..}if answers[0]=="999.1.1.1")
                );
                self.checked = true;
            }
            5 => shot = Some("02-ipv4-wrong"),
            6 => {
                if let engine::State::Subnet { answers, .. } =
                    &mut self.progress.attempts.get_mut("ccna-ipv4").unwrap().state
                {
                    *answers = [
                        "192.168.10.64".into(),
                        "192.168.10.95".into(),
                        "192.168.10.65".into(),
                        "192.168.10.94".into(),
                    ];
                }
                self.dirty = true;
            }
            7 => {
                assert!(self.metrics.completion);
                assert_eq!(self.progress.attempts["ccna-ipv4"].step, 0);
                shot = Some("03-complete-at-section-one");
            }
            8 => {
                if let engine::State::Subnet { answers, .. } =
                    &mut self.progress.attempts.get_mut("ccna-ipv4").unwrap().state
                {
                    answers[0] = "192.168.10.63".into();
                }
                self.dirty = true;
            }
            9 => {
                assert!(!self.metrics.completion);
                assert!(self.invalidated);
                self.launch(1);
            }
            10 => {
                assert_eq!(self.metrics.inputs.len(), 3);
                shot = Some("04-vlsm-table");
            }
            11 => self.launch(5),
            12 => {
                self.smoke_focus = ctx.memory(|m| m.focused());
                assert!(self.smoke_focus.is_some());
                self.pending_events
                    .push(egui::Event::Text("grep ERROR /var/log/app.log".into()));
            }
            13 => {
                assert_eq!(self.input, "grep ERROR /var/log/app.log");
                self.pending_events.push(key(egui::Key::Enter, false));
            }
            14 => {
                assert!(engine::solved(
                    &self.progress.attempts["lpic1-logs"].state,
                    &self.scenarios[5].goal
                ));
                self.pending_events.push(key(egui::Key::Tab, false));
            }
            15 => {
                assert_ne!(ctx.memory(|m| m.focused()), self.smoke_focus);
                self.pending_events.push(key(egui::Key::Tab, true));
            }
            16 => {
                assert_eq!(ctx.memory(|m| m.focused()), self.smoke_focus);
                self.launch(6);
            }
            17 => self.service_draft = Some("8080".into()),
            18 => {
                let rect = self.metrics.editor_save.unwrap();
                self.pending_events.extend([
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
                ]);
            }
            19 => {
                assert!(
                    matches!(&self.progress.attempts["lpic2-service"].state,engine::State::Service{port,running:false,..}if port=="8080")
                );
                shot = Some("05-service-editor");
            }
            20 => {
                for command in [
                    "systemctl restart atlas",
                    "systemctl enable atlas",
                    "systemctl status atlas",
                ] {
                    self.input = command.into();
                    self.submit(6);
                }
            }
            21 => {
                assert!(self.metrics.completion);
                shot = Some("06-service-complete");
            }
            22 => {
                self.selected = None;
                self.scale = 1.5;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
            23 => {
                let bounds = ctx.input(|i| i.content_rect());
                assert!(ctx.zoom_factor() > 1.4);
                for rect in &self.metrics.filters {
                    assert!(
                        rect.right() <= bounds.right() + 1.,
                        "native filter clips at 150%: {rect:?} in {bounds:?}"
                    );
                }
                shot = Some("07-library-150-percent");
            }
            24 => self.launch(0),
            25 => {
                assert!(!self.metrics.split);
                shot = Some("08-stacked-ipv4-150-percent");
            }
            26 => {
                self.pending_events.extend([
                    egui::Event::PointerMoved(egui::pos2(300., 300.)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., -650.),
                        modifiers: Default::default(),
                    },
                ]);
            }
            27 => shot = Some("09-stacked-inputs-150-percent"),
            28 => {
                self.selected = None;
                self.scale = 1.;
                self.developer = true;
                self.synthetic = true;
            }
            29 => {
                assert!(self.metrics.pagination);
                shot = Some("10-developer-catalog");
            }
            30 => {
                println!(
                    "PASS native Fedora: fields, wrong answers, section-1 completion/invalidation, VLSM table, terminal Enter/Tab/Shift+Tab, editor Save click, service completion, real resize, native 150% filters and stacked scrolling."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => {}
        }
        if let Some(name) = shot {
            self.capture_pending = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                name.to_string(),
            )));
        }
        self.smoke_stage += 1;
        self.started = Instant::now();
    }
}
