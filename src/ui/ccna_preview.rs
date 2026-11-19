//! Fixed CCNA review captures. Private smoke profile; fill/check use native controls.
use super::*;
#[derive(Clone, Default)]
struct Capture {
    index: usize,
    phase: usize,
}
impl LabApp {
    pub(super) fn ccna_preview_tick(&mut self, ctx: &egui::Context) {
        if std::env::var("DLMS_CCNA_GUIDANCE_CAPTURE").as_deref() == Ok("1") {
            self.magic_guidance_preview_tick(ctx);
            return;
        }
        let key = egui::Id::new("ccna-pilot-capture");
        let mut c = ctx.data_mut(|d| d.get_temp::<Capture>(key).unwrap_or_default());
        let cases = [
            ("ccna-library", 0, "ccna-library-start"),
            ("ccna-magic-blocks", 0, "magic-introduction"),
            ("ccna-magic-blocks", 1, "magic-example"),
            ("ccna-magic-blocks", 2, "magic-assignment"),
            ("ccna-ipv4", 0, "existing-ipv4-introduction"),
            ("ccna-ipv4", 1, "existing-ipv4-example"),
            ("ccna-ipv4", 2, "existing-ipv4-assignment"),
        ];
        if c.index > cases.len() * 2 {
            println!("CCNA_REVISION_PREVIEWS_COMPLETE");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let enlarged = c.index >= cases.len() && c.index < cases.len() * 2;
        let (id, view, name) = if c.index < cases.len() * 2 {
            cases[c.index % cases.len()]
        } else {
            ("ccna-ipv4", 1, "existing-ipv4-example-checked")
        };
        if c.phase == 0 {
            let i = if id == "ccna-library" {
                0
            } else {
                self.scenarios
                    .iter()
                    .position(|s| s.id == id)
                    .expect("fixed preview lab")
            };
            if id == "ccna-library" {
                self.selected = None;
            } else {
                self.launch(i);
            }
            let view = match view {
                0 => teaching::View::Introduction,
                1 => teaching::View::Example,
                _ => teaching::View::Assignment,
            };
            if id == "ccna-library" {
            } else if let engine::State::MagicSubnet { session } = &mut self
                .progress
                .attempts
                .get_mut(&self.scenarios[i].id)
                .unwrap()
                .state
            {
                session.view = view;
            } else if id.starts_with("ccna-vlan") {
                self.set_switching_view(i, view);
            } else if id == "ccna-ipv4" {
                self.set_address_view(i, view);
            }
            self.scale = if enlarged { 2. } else { 1. };
            ctx.set_zoom_factor(self.scale);
            self.theme_id = if enlarged {
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
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!enlarged));
            if enlarged {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1100., 850.)));
            }
            self.lesson_scroll_reset = true;
        }
        if name == "existing-ipv4-example-checked" {
            let target = if c.phase == 7 || c.phase == 8 {
                self.metrics.insert
            } else if c.phase == 15 || c.phase == 16 {
                self.metrics.check
            } else {
                None
            };
            if let Some(rect) = target {
                assert!(smoke::smoke_control_visible(
                    rect,
                    self.metrics.main_viewport.unwrap(),
                    ctx.pixels_per_point()
                ));
                self.pending_events = vec![
                    egui::Event::PointerMoved(rect.center()),
                    egui::Event::PointerButton {
                        pos: rect.center(),
                        button: egui::PointerButton::Primary,
                        pressed: c.phase == 7 || c.phase == 15,
                        modifiers: Default::default(),
                    },
                ];
            }
        }
        if name == "existing-ipv4-example-checked" {
            if [28, 29, 45, 46].contains(&c.phase) {
                let r = self.metrics.inputs[4];
                self.pending_events = vec![
                    egui::Event::PointerMoved(r.center()),
                    egui::Event::PointerButton {
                        pos: r.center(),
                        button: egui::PointerButton::Primary,
                        pressed: c.phase == 28 || c.phase == 45,
                        modifiers: Default::default(),
                    },
                ];
            }
            if c.phase == 31 || c.phase == 48 {
                self.pending_events = vec![egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    },
                }];
            }
            if c.phase == 32 || c.phase == 49 {
                let addresses = self
                    .address_example
                    .as_ref()
                    .unwrap()
                    .exercise
                    .expected()
                    .unwrap();
                let value = addresses[if c.phase == 32 { 2 } else { 1 }].to_string();
                self.pending_events = vec![
                    egui::Event::Key {
                        key: egui::Key::A,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers: Default::default(),
                    },
                    egui::Event::Text(value),
                ];
            }
            if [35, 36, 52, 53].contains(&c.phase) {
                let r = self.metrics.check.unwrap();
                self.pending_events = vec![
                    egui::Event::PointerMoved(r.center()),
                    egui::Event::PointerButton {
                        pos: r.center(),
                        button: egui::PointerButton::Primary,
                        pressed: c.phase == 35 || c.phase == 52,
                        modifiers: Default::default(),
                    },
                ];
            }
            if c.phase == 40 {
                assert!(
                    self.address_example.as_ref().unwrap().verified(),
                    "alternate usable workstation address must pass; fields={:?}, second={:?}",
                    self.address_example.as_ref().unwrap().exercise.answers,
                    self.address_example.as_ref().unwrap().exercise.second
                );
            }
            if c.phase == 58 {
                let example = self.address_example.as_ref().unwrap();
                assert!(!example.verified(), "broadcast workstation must fail");
                assert_eq!(
                    example.checked_answers.as_ref().unwrap()[4],
                    example.exercise.expected().unwrap()[1].to_string()
                );
            }
            if c.phase == 42 || c.phase == 60 {
                self.capture_pending = true;
                let suffix = if c.phase == 42 {
                    "alternate-valid"
                } else {
                    "broadcast-feedback"
                };
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                    format!("ccna-pilot-existing-ipv4-{suffix}-dark-100"),
                )));
            }
        }
        if name == "existing-ipv4-example-checked" && c.phase == 12 {
            assert!(
                self.address_example.as_ref().unwrap().exercise.solved(),
                "native Fill worked example did not populate the practice fields"
            );
        }
        if name == "existing-ipv4-example-checked" && c.phase == 22 {
            assert!(
                self.address_example.as_ref().unwrap().verified(),
                "native Check configuration did not verify the worked example"
            );
        }
        if c.phase
            == if name == "existing-ipv4-example-checked" {
                26
            } else {
                12
            }
        {
            let suffix = if enlarged { "light-200" } else { "dark-100" };
            let geometry = ctx.content_rect();
            println!(
                "CCNA_PREVIEW {name}-{suffix} geometry={geometry:?} scale={}",
                self.scale
            );
            self.capture_pending = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                format!("ccna-pilot-{name}-{suffix}"),
            )));
        }
        if enlarged && c.phase == 14 {
            let visible = self.metrics.main_viewport.expect("rendered viewport");
            self.pending_events = vec![
                egui::Event::PointerMoved(visible.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -650.),
                    modifiers: Default::default(),
                },
            ];
        }
        if enlarged && c.phase == 30 {
            println!(
                "CCNA_SCROLLED {name} offset={:?} content={:?}",
                self.metrics.main_scroll_offset, self.metrics.main_content_size
            );
            self.capture_pending = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                format!("ccna-pilot-{name}-light-200-scrolled"),
            )));
        }
        if enlarged && c.phase == 32 {
            let visible = self.metrics.main_viewport.unwrap();
            self.pending_events = vec![
                egui::Event::PointerMoved(visible.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -4000.),
                    modifiers: Default::default(),
                },
            ];
        }
        if enlarged && c.phase == 48 {
            println!(
                "CCNA_CONTROLS {name} offset={:?}",
                self.metrics.main_scroll_offset
            );
            self.capture_pending = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                format!("ccna-pilot-{name}-light-200-controls"),
            )));
        }
        if c.phase
            > if enlarged {
                48
            } else if name == "existing-ipv4-example-checked" {
                60
            } else {
                12
            }
            && !self.capture_pending
        {
            c.index += 1;
            c.phase = 0;
        } else {
            c.phase += 1;
        }
        ctx.data_mut(|d| d.insert_temp(key, c));
    }
}

impl LabApp {
    fn magic_guidance_preview_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("magic-guidance-capture");
        let mut c = ctx.data_mut(|d| d.get_temp::<Capture>(key).unwrap_or_default());
        let cases = [
            (0, "introduction"),
            (1, "guided-practice"),
            (2, "assignment"),
            (3, "practice-feedback"),
            (4, "worked-answer"),
            (5, "practice-prefix20"),
            (6, "practice-prefix24"),
        ];
        if c.index >= cases.len() * 2 {
            println!("CCNA_GUIDANCE_PREVIEWS_COMPLETE");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let enlarged = c.index >= cases.len();
        let (kind, name) = cases[c.index % cases.len()];
        if c.phase == 0 {
            let i = self
                .scenarios
                .iter()
                .position(|s| s.id == "ccna-magic-blocks")
                .unwrap();
            self.launch(i);
            let engine::State::MagicSubnet { session } = &mut self
                .progress
                .attempts
                .get_mut("ccna-magic-blocks")
                .unwrap()
                .state
            else {
                panic!("magic lesson");
            };
            **session =
                dlms_learning_lab::magic_subnet::Session::new(session.problem, session.stage);
            session.view = match kind {
                0 => teaching::View::Introduction,
                2 => teaching::View::Assignment,
                _ => teaching::View::Example,
            };
            if kind == 3 {
                session.example.answers[1] = "4".into();
                session.example.answers[2] = "192".into();
            }
            if kind == 5 || kind == 6 {
                // Fixed synthetic exploratory draft, not personal work. It stays
                // unchecked so the containing interval must remain concealed.
                session.example.address = "172.19.143.9".into();
                session.example.prefix = if kind == 5 { "20" } else { "24" }.into();
                let p = session.example_problem().unwrap();
                let calc = p.calculate().unwrap();
                session.example.answers[0] = calc.mask.to_string();
                session.example.answers[1] = calc.mask_octet.to_string();
            }
            self.scale = if enlarged { 2. } else { 1. };
            ctx.set_zoom_factor(self.scale);
            self.theme_id = if enlarged {
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
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!enlarged));
            if enlarged {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1100., 850.)));
            }
            self.lesson_scroll_reset = true;
        }
        if !enlarged && (kind == 3 || kind == 4) && (c.phase == 7 || c.phase == 8) {
            let target = if kind == 3 {
                self.metrics.check
            } else {
                self.metrics.insert
            }
            .unwrap();
            self.pending_events = vec![
                egui::Event::PointerMoved(target.center()),
                egui::Event::PointerButton {
                    pos: target.center(),
                    button: egui::PointerButton::Primary,
                    pressed: c.phase == 7,
                    modifiers: Default::default(),
                },
            ];
        }
        if c.phase == 14 {
            if !enlarged && (kind == 3 || kind == 4) {
                let engine::State::MagicSubnet { session } =
                    &self.progress.attempts["ccna-magic-blocks"].state
                else {
                    panic!()
                };
                assert_eq!(
                    session.example_checked.as_ref(),
                    Some(&session.example),
                    "native action did not check/show the example"
                );
            }
            let suffix = if enlarged { "light-200" } else { "dark-100" };
            println!(
                "CCNA_GUIDANCE {name}-{suffix} geometry={:?}",
                ctx.content_rect()
            );
            self.capture_pending = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                format!("ccna-guidance-{name}-{suffix}"),
            )));
        }
        if enlarged && (c.phase == 16 || c.phase == 34) {
            let visible = self.metrics.main_viewport.unwrap();
            self.pending_events = vec![
                egui::Event::PointerMoved(visible.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., if c.phase == 16 { -500. } else { -4000. }),
                    modifiers: Default::default(),
                },
            ];
        }
        if enlarged && (c.phase == 32 || c.phase == 50) {
            self.capture_pending = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                format!(
                    "ccna-guidance-{name}-light-200-{}",
                    if c.phase == 32 {
                        "scrolled"
                    } else {
                        "controls"
                    }
                ),
            )));
        }
        if c.phase > if enlarged { 50 } else { 14 } && !self.capture_pending {
            c.index += 1;
            c.phase = 0;
        } else {
            c.phase += 1;
        }
        ctx.data_mut(|d| d.insert_temp(key, c));
    }
}
