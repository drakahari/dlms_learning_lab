use super::*;
use dlms_learning_lab::ipv4::{self, Exercise, Section};
impl LabApp {
    fn ipv4_seed(&mut self, section: Section, answers: &[(usize, &str)], reveal: bool) {
        let a = self.progress.attempts.get_mut("ccna-ipv4").unwrap();
        if let engine::State::Subnet {
            answers: current, ..
        } = &mut a.state
        {
            for (index, value) in answers {
                current[*index] = value.to_string();
            }
        }
        let e = Exercise::from_state(&a.state).unwrap();
        let l = a.ipv4.as_mut().unwrap();
        l.section = section;
        l.submit(&e);
        if reveal {
            l.reveal(section, &e);
        }
        self.checked = true;
        self.dirty = true;
    }
    fn ipv4_click(&mut self, rect: egui::Rect) {
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
    pub(super) fn ipv4_smoke_tick(&mut self, ctx: &egui::Context) {
        if self.capture_pending || self.started.elapsed().as_millis() < 500 {
            return;
        }
        let mut shot = None;
        match self.smoke_stage {
            0 => {
                self.launch(0);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            }
            1 => {
                assert!(self.metrics.ipv4_explain);
                assert_eq!(self.metrics.inputs.len(), 4);
                assert!(!self.metrics.ipv4_worked);
                shot = Some("01-original-and-mask");
            }
            2 => {
                self.progress
                    .attempts
                    .get_mut("ccna-ipv4")
                    .unwrap()
                    .ipv4
                    .as_mut()
                    .unwrap()
                    .section = Section::Network;
                self.pending_events
                    .push(egui::Event::Text("192.168.10.77".into()));
            }
            3 => self.ipv4_click(self.metrics.check.unwrap()),
            4 => {
                assert!(!self.metrics.ipv4_worked);
                let e = Exercise::from_state(&self.progress.attempts["ccna-ipv4"].state).unwrap();
                assert_eq!(ipv4::diagnose(&e, 0), ipv4::Diagnosis::HostInsteadOfNetwork);
                shot = Some("02-host-versus-network-feedback");
            }
            5 => self.ipv4_click(self.metrics.ipv4_reveal.unwrap()),
            6 => {
                assert!(self.metrics.ipv4_worked);
                shot = Some("03-requested-network-result");
            }
            7 => self.ipv4_seed(Section::Broadcast, &[(1, "192.168.10.90")], true),
            8 => {
                assert!(self.metrics.ipv4_worked);
                shot = Some("04-requested-broadcast-result");
            }
            9 => self.ipv4_seed(
                Section::Usable,
                &[(2, "192.168.10.65"), (3, "192.168.10.94")],
                true,
            ),
            10 => {
                assert!(self.metrics.ipv4_worked);
                shot = Some("05-requested-usable-range");
            }
            11 => {
                self.ipv4_seed(Section::Original, &[], false);
                self.checked = false;
                self.pending_events.extend([
                    egui::Event::PointerMoved(egui::pos2(700., 600.)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., -900.),
                        modifiers: Default::default(),
                    },
                ]);
            }
            12 => {
                let rect = self
                    .metrics
                    .ipv4_bits
                    .iter()
                    .find(|(rect, p, sandbox)| {
                        *sandbox
                            && *p == 31
                            && ctx.input(|i| i.content_rect().contains(rect.center()))
                    })
                    .expect("sandbox host bit visible after scrolling")
                    .0;
                self.ipv4_click(rect);
            }
            13 => {
                assert_eq!(
                    self.progress.attempts["ccna-ipv4"]
                        .ipv4
                        .as_ref()
                        .unwrap()
                        .sandbox_host,
                    12
                );
                assert!(self.ipv4_focus.is_some());
                shot = Some("06-host-bit-sandbox");
            }
            14 => {
                let a = self.progress.attempts.get_mut("ccna-ipv4").unwrap();
                assert!(a.ipv4.as_mut().unwrap().fresh());
                a.challenge = true;
                self.ipv4_focus = None;
                self.ipv4_challenge_revealed = [false; 3];
                self.checked = false;
                self.pending_events.extend([
                    egui::Event::PointerMoved(egui::pos2(700., 500.)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., 2000.),
                        modifiers: Default::default(),
                    },
                ]);
            }
            15 => {
                assert!(!self.metrics.ipv4_worked);
                assert!(
                    self.progress.attempts["ccna-ipv4"]
                        .ipv4
                        .as_ref()
                        .unwrap()
                        .use_variant
                );
                assert_eq!(self.scenarios.len(), 9);
                shot = Some("07-fresh-challenge-variant");
            }
            16 => {
                self.scale = 1.5;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
            17 => {
                let bounds = ctx.input(|i| i.content_rect());
                for (rect, _, _) in &self.metrics.ipv4_bits {
                    assert!(
                        rect.right() <= bounds.right() + 1.,
                        "IPv4 bit clips at 150%: {rect:?} vs {bounds:?}"
                    );
                }
                shot = Some("08-stacked-150-percent");
            }
            18 => {
                self.selected = None;
                self.scale = 1.;
                self.filter.search = "report".into();
            }
            19 => shot = Some("09-permissions-description"),
            20 => {
                println!(
                    "PASS native IPv4 lesson: explicit IP/mask, typed prediction and native Check/Reveal clicks, requested worked views, sandbox host-bit click and decimal update, independent Challenge variant, 150% octet bounds, Alex-specific library description. Broadcast/range states seeded through tested lesson model; no physical-device claim."
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
