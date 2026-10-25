//! Private technical-fixture native flow. Initial launch only; no solved-State install.
use super::*;
const FIRST: usize = 30000;
const LAST: usize = 30095;
const ID: &str = "dhcpv4-workspace-preview";
impl LabApp {
    pub(in crate::ui) fn add_dhcpv4_fixture(&mut self) -> usize {
        assert!(
            self.smoke || cfg!(test),
            "private fixture is developer/test only"
        );
        if let Some(i) = self.scenarios.iter().position(|s| s.id == ID) {
            return i;
        }
        let scenario = pack::load(include_str!(
            "../../../tests/fixtures/dhcpv4-workspace-preview.json"
        ))
        .expect("validated unbundled technical fixture")
        .remove(0);
        let i = self.scenarios.len();
        self.scenarios.push(scenario);
        i
    }
    fn dhcpv4_geometry(&self, ctx: &egui::Context) -> bool {
        let rect = ctx.content_rect();
        let zoom = ctx.zoom_factor();
        let ppp = ctx.pixels_per_point();
        let stable = ctx.data_mut(|d| {
            let key = egui::Id::new(("dhcp-native-geometry", self.smoke_stage));
            let (old, count, started) = d
                .get_temp::<(Option<egui::Rect>, usize, Instant)>(key)
                .unwrap_or((None, 0, Instant::now()));
            assert!(
                started.elapsed().as_secs() < 15,
                "DHCP native geometry did not settle"
            );
            assert!(ppp.is_finite() && ppp > 0.);
            let n = if old == Some(rect) && (zoom - self.scale).abs() < 0.01 {
                count + 1
            } else {
                0
            };
            d.insert_temp(key, (Some(rect), n, started));
            n >= 2
        });
        if !stable {
            ctx.request_repaint();
        }
        stable
    }
    fn dhcpv4_control(&mut self, ctx: &egui::Context, key: &str) -> bool {
        let phase = ctx
            .data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))))
            .unwrap_or(0);
        if phase >= 2 {
            return self.click_visible_smoke_control(ctx, None);
        }
        let r = self
            .metrics
            .dhcp_controls
            .iter()
            .find(|(id, _, _)| id == key)
            .map(|(_, r, enabled)| {
                assert!(*enabled, "native control must be enabled: {key}");
                *r
            });
        self.click_visible_smoke_control(ctx, r)
    }
    fn dhcpv4_field(&mut self, ctx: &egui::Context, key: &str) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            let field = self
                .metrics
                .dhcp_fields
                .iter()
                .find(|(name, _, _)| name == key)
                .expect("actual focused DHCP field")
                .2;
            assert!(
                ctx.memory(|m| m.has_focus(field)),
                "native field click must own TextEdit focus before typing"
            );
            return self.click_visible_smoke_control(ctx, None);
        }
        let (_, rect, field) = self
            .metrics
            .dhcp_fields
            .iter()
            .find(|(id, _, _)| id == key)
            .expect("actual DHCP field");
        let (rect, field) = (*rect, *field);
        let done = self.click_visible_smoke_control(ctx, Some(rect));
        if done {
            assert!(ctx.memory(|m| m.has_focus(field)));
        }
        done
    }
    fn dhcpv4_type(&mut self, text: &str) {
        assert!(text.len() <= 64);
        self.pending_events = [true, false]
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
            .collect();
        self.pending_events.push(egui::Event::Text(text.into()));
    }
    fn dhcpv4_show_check(&mut self, ctx: &egui::Context, pass: bool, i: usize) -> bool {
        assert_eq!(
            progress::current_solved(&self.progress.attempts[ID], &self.scenarios[i]),
            pass
        );
        assert_eq!(self.metrics.completion, pass);
        self.reveal_smoke_control(ctx, self.metrics.feedback.expect("current DHCP feedback"))
    }
    pub(super) fn dhcpv4_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert!((FIRST..=LAST).contains(&stage));
        assert!(self.smoke && self._smoke_profile.is_some());
        assert_eq!(
            self.path.parent(),
            Some(self._smoke_profile.as_ref().unwrap().path())
        );
        let i = self.add_dhcpv4_fixture();
        let scenario = self.scenarios[i].clone();
        let lesson = scenario.dhcpv4_teaching.as_ref().unwrap();
        let mut shot = None;
        let practice = |a: &LabApp| {
            a.progress.attempts[ID]
                .workspace
                .as_ref()
                .unwrap()
                .dhcpv4_learning
                .as_ref()
                .unwrap()
                .clone()
        };
        match stage - FIRST {
            0 => {
                self.launch(i);
                self.progress.favorites.insert(ID.into());
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, "dlms-dark");
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            }
            1 | 49 | 57 | 83 => {
                if !self.dhcpv4_geometry(ctx) {
                    return (false, None);
                }
            }
            2 => {
                assert_eq!(self.dhcpv4_view(i), teaching::View::Introduction);
                shot = Some("dhcpv4-private-introduction".into());
            }
            3 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.example_action) {
                    return (false, None);
                }
            }
            4 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.insert) {
                    return (false, None);
                }
            }
            5 => {
                let l = practice(self);
                assert_eq!(l.simulation, lesson.fixture);
                assert!(!l.draft.matches(&l.simulation));
                assert_eq!(self.progress.attempts[ID].state, scenario.initial);
                shot = Some("dhcpv4-example-fill-only".into());
            }
            6 | 32 | 45 | 52 | 60 => {
                if !self.dhcpv4_control(ctx, "apply") {
                    return (false, None);
                }
            }
            7 => {
                let l = practice(self);
                assert_ne!(l.simulation.policy(), lesson.fixture.policy());
                assert!(!l.simulation.leases().contains_key("museum-branch"));
            }
            8 | 17 | 33 => {
                if !self.dhcpv4_control(ctx, "discover") {
                    return (false, None);
                }
            }
            9 => {
                let l = practice(self);
                assert!(l.simulation.offers().contains_key("museum-branch"));
                assert!(!l.simulation.leases().contains_key("museum-branch"));
            }
            10 | 18 | 36 => {
                if !self.dhcpv4_control(ctx, "request") {
                    return (false, None);
                }
            }
            11 | 20 => {
                let l = practice(self);
                assert!(l.checks(lesson).iter().all(|r| r.0));
                assert_eq!(self.progress.attempts[ID].state, scenario.initial);
                if stage - FIRST == 20 {
                    shot = Some("dhcpv4-example-repaired".into());
                }
            }
            12 | 15 | 19 | 22 | 30 | 35 | 37 | 41 | 46 | 54 | 62 | 67 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            13 => {
                assert!(practice(self).checked);
                shot = Some("dhcpv4-example-configured".into());
            }
            14 => {
                if !self.dhcpv4_control(ctx, "release") {
                    return (false, None);
                }
            }
            16 => {
                let l = practice(self);
                assert!(!l.simulation.leases().contains_key("museum-branch"));
                assert_eq!(
                    l.checks(lesson).iter().map(|r| r.0).collect::<Vec<_>>(),
                    lesson.try_change.expected
                );
                shot = Some("dhcpv4-example-released-not-configured".into());
            }
            21 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.lesson_tabs.get(2).copied())
                {
                    return (false, None);
                }
            }
            23 => {
                if !self.dhcpv4_field(ctx, "relay-target") {
                    return (false, None);
                }
            }
            24 => self.dhcpv4_type("192.0.2.1"),
            25 => {
                if !self.dhcpv4_field(ctx, "router") {
                    return (false, None);
                }
            }
            26 => self.dhcpv4_type("10.20.0.1"),
            27 => {
                if !self.dhcpv4_control(ctx, "dns-header") {
                    return (false, None);
                }
            }
            28 => {
                if !self.dhcpv4_field(ctx, "dns-0") {
                    return (false, None);
                }
            }
            29 => self.dhcpv4_type("203.0.113.53"),
            31 => {
                if !self.dhcpv4_show_check(ctx, false, i) {
                    return (false, None);
                }
                shot = Some("dhcpv4-pending-policy-no-credit".into());
            }
            34 => {
                let engine::State::DhcpV4 { simulation } = &self.progress.attempts[ID].state else {
                    unreachable!()
                };
                assert!(simulation.offers().contains_key("branch"));
                assert!(!simulation.leases().contains_key("branch"));
                shot = Some("dhcpv4-offer-is-not-lease".into());
            }
            38 | 47 | 55 | 63 | 68 => {
                if !self.dhcpv4_show_check(ctx, true, i) {
                    return (false, None);
                }
                if stage - FIRST == 68 {
                    let engine::State::DhcpV4 { simulation } = &self.progress.attempts[ID].state
                    else {
                        unreachable!()
                    };
                    assert_eq!(simulation.now(), 1);
                    assert_eq!(simulation.leases()["branch"].remaining(1), 159);
                }
                shot = Some(format!("dhcpv4-current-and-fresh-passing-{stage}"));
            }
            39 | 43 | 50 | 58 | 73 => {
                if !self.dhcpv4_field(ctx, "lease-seconds") {
                    return (false, None);
                }
            }
            40 => self.dhcpv4_type("bad duration"),
            44 => self.dhcpv4_type("96"),
            51 => self.dhcpv4_type("128"),
            59 => self.dhcpv4_type("160"),
            74 => self.dhcpv4_type("unfinished"),
            42 => {
                if !self.dhcpv4_show_check(ctx, false, i) {
                    return (false, None);
                }
                shot = Some("dhcpv4-invalid-draft-retains-lease".into());
            }
            48 | 56 | 82 => {
                let (scale, theme, size) = match stage - FIRST {
                    48 => (1.5, "dlms-light", egui::vec2(960., 850.)),
                    56 => (2., "dlms-dark", egui::vec2(960., 850.)),
                    _ => (1., "dlms-dark", egui::vec2(1200., 850.)),
                };
                self.scale = scale;
                self.theme_id = theme.into();
                theme::apply(ctx, theme);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            }
            53 | 61 => {
                if !self.dhcpv4_control(ctx, "renew") {
                    return (false, None);
                }
            }
            64 => {
                if !self.dhcpv4_field(ctx, "time") {
                    return (false, None);
                }
            }
            65 => self.dhcpv4_type("1"),
            66 => {
                if !self.dhcpv4_control(ctx, "advance") {
                    return (false, None);
                }
            }
            69 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.mode_challenge) {
                    return (false, None);
                }
            }
            70 => {
                assert!(self.progress.attempts[ID].challenge);
                assert!(self.metrics.lesson_tabs.is_empty());
                assert!(!self.metrics.help_visible);
                shot = Some("dhcpv4-independent-challenge".into());
            }
            71 | 75 => {
                let before = serde_json::to_vec(&self.progress.attempts).unwrap();
                self.save();
                assert!(self.storage_error.is_none());
                let bytes = progress::read_bounded(&self.path, 16_000_000).unwrap();
                self.progress = progress::decode(&bytes, &self.scenarios).unwrap();
                assert_eq!(serde_json::to_vec(&self.progress.attempts).unwrap(), before);
                if stage - FIRST == 75 {
                    ctx.data_mut(|d| d.insert_temp(egui::Id::new("dhcp-before-reset"), before));
                }
                shot = Some(format!("dhcpv4-exact-save-resume-{stage}"));
            }
            72 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.mode_learn) {
                    return (false, None);
                }
            }
            76 | 79 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.reset) {
                    return (false, None);
                }
            }
            77 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.reset_cancel) {
                    return (false, None);
                }
            }
            78 => {
                let before = ctx
                    .data(|d| d.get_temp::<Vec<u8>>(egui::Id::new("dhcp-before-reset")))
                    .unwrap();
                assert_eq!(serde_json::to_vec(&self.progress.attempts).unwrap(), before);
                assert!(!self.reset_pending);
            }
            80 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.confirm_reset) {
                    return (false, None);
                }
            }
            81 => {
                assert_eq!(self.progress.attempts[ID].state, scenario.initial);
                assert!(self.progress.favorites.contains(ID));
                assert!(!progress::current_solved(
                    &self.progress.attempts[ID],
                    &scenario
                ));
            }
            84 => {
                shot = Some("dhcpv4-selected-reset".into());
                self.smoke_stage = LAST - 1;
            }
            95 => {
                println!(
                    "PASS private DHCP native controls: separate fill/apply/exchange/experiment, pending and offer-only rejection, configured current/fresh state, actual150Light/200Dark field typing+renewal, explicit clock, raw draft save/resume and Reset Cancel/Confirm. No bundled activity, protocol packets, routed/DNS-service, cold restart, novice or other-platform claim."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return (false, None);
            }
            _ => unreachable!("closed private DHCP native phase"),
        }
        (true, shot)
    }
}
