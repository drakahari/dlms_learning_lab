//! Fixed native development inputs only; never a host router/shell session.
use super::*;
impl LabApp {
    fn ospf_picker_click(&mut self, ctx: &egui::Context, rect: egui::Rect) -> bool {
        let id = egui::Id::new(("ospf-picker-physical-phases", self.smoke_stage));
        let phase = ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0);
        if phase == 3 {
            ctx.data_mut(|d| d.remove::<u8>(id));
            return true;
        }
        if !self.reveal_smoke_control(ctx, rect) {
            return false;
        }
        self.pending_events
            .push(egui::Event::PointerMoved(rect.center()));
        if phase > 0 {
            self.pending_events.push(egui::Event::PointerButton {
                pos: rect.center(),
                button: egui::PointerButton::Primary,
                pressed: phase == 1,
                modifiers: Default::default(),
            });
        }
        ctx.data_mut(|d| d.insert_temp(id, phase + 1));
        self.started = Instant::now();
        ctx.request_repaint();
        false
    }
    pub(super) fn ospf_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        let id = "test-ospf-workspace";
        let mut shot = None;
        let ctrl = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        };
        if stage == 10000 {
            self.scale = 1.;
            self.theme_id = "dlms-dark".into();
            theme::apply(ctx, &self.theme_id);
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            let i = self.add_ospf_fixture();
            self.launch(i);
            return (true, None);
        }
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        let s = self.scenarios[i].clone();
        match stage {
            10001 => shot = Some("ospf-introduction".into()),
            10002 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.example_action) {
                    return (false, None);
                }
            }
            10003 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.insert) {
                    return (false, None);
                }
            }
            10004 => {
                let l = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .ospf_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(l.simulation, s.ospf_teaching.as_ref().unwrap().fixture);
                assert!(!l.checked && !l.draft.matches(&l.simulation));
                shot = Some("ospf-example-filled-unapplied".into());
            }
            10005 | 10013 | 10017 | 10021 | 10028 | 10032 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            10006 => {
                let l = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .ospf_learning
                    .as_ref()
                    .unwrap();
                assert!(
                    l.checked
                        && l.checks(s.ospf_teaching.as_ref().unwrap())
                            .iter()
                            .all(|c| c.0)
                );
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
                shot = Some("ospf-example-ungraded-success".into());
            }
            10007 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.assignment_action) {
                    return (false, None);
                }
            }
            10008 => {
                assert_eq!(self.progress.attempts[id].state, s.initial);
                shot = Some("ospf-full-assignment".into());
            }
            10009 => {
                if !self.ospf_picker_click(ctx, self.metrics.ospf_router.unwrap()) {
                    return (false, None);
                }
                shot = Some("ospf-router-picker-open".into());
            }
            10010 => {
                let target = self
                    .metrics
                    .ospf_router_items
                    .iter()
                    .find(|(name, _)| name == "Core")
                    .map(|(_, r)| *r);
                // Selection closes the popup before the final click phase.
                if !self.click_smoke_control(ctx, target) {
                    return (false, None);
                }
            }
            10011 | 10015 | 10026 | 10030 => {
                let target = self
                    .metrics
                    .ospf_fields
                    .iter()
                    .find(|(name, _, _)| name == "Core/branch-core:hello")
                    .expect("selected Core/branch-core")
                    .1;
                if !self.click_visible_smoke_control(ctx, Some(target)) {
                    return (false, None);
                }
            }
            10012 | 10016 | 10020 | 10027 | 10031 => {
                self.pending_events = vec![
                    ctrl(egui::Key::A),
                    egui::Event::Text(
                        match stage {
                            10012 => "malformed",
                            10020 => "23",
                            10027 => "11",
                            _ => "10",
                        }
                        .into(),
                    ),
                ];
            }
            10014 => {
                assert_eq!(self.progress.attempts[id].state, s.initial);
                assert!(!self.metrics.completion);
                shot = Some("ospf-field-error".into());
            }
            10018 => {
                assert!(self.metrics.completion);
                shot = Some("ospf-repaired-current-outcomes".into());
            }
            10019 => {
                let target = self
                    .metrics
                    .ospf_fields
                    .iter()
                    .find(|(name, _, _)| name == "Core/branch-core:cost")
                    .unwrap()
                    .1;
                if !self.click_visible_smoke_control(ctx, Some(target)) {
                    return (false, None);
                }
            }
            10022 => {
                assert!(self.metrics.completion);
                let bytes = serde_json::to_vec(&self.progress.attempts[id]).unwrap();
                progress::save(&self.path, &self.progress).unwrap();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_vec(&self.progress.attempts[id]).unwrap(),
                    bytes
                );
                self.launch(i);
                self.set_ospf_view(i, teaching::View::Assignment);
            }
            10023 => {
                assert!(self.metrics.completion);
                shot = Some("ospf-alternate-resumed".into());
            }
            10024 => {
                if !self.ospf_picker_click(ctx, self.metrics.ospf_interface.unwrap()) {
                    return (false, None);
                }
            }
            10025 => {
                let target = self
                    .metrics
                    .ospf_interface_items
                    .iter()
                    .find(|(name, _)| name == "branch-core")
                    .map(|(_, r)| *r);
                if !self.click_smoke_control(ctx, target) {
                    return (false, None);
                }
                assert!(
                    self.metrics.completion,
                    "selection is not a config mutation"
                );
            }
            10029 => {
                let results = engine::checks(&self.progress.attempts[id].state, &s.goal);
                assert!(results.iter().any(|c| c.0) && results.iter().any(|c| !c.0));
                assert!(!self.metrics.completion);
                shot = Some("ospf-partial-retry-feedback".into());
            }
            10033 => {
                assert!(self.metrics.completion);
                self.progress.attempts.get_mut(id).unwrap().challenge = true;
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 900.)));
                self.lesson_scroll_reset = true;
            }
            10034 => {
                assert!(self.metrics.lesson_tabs.is_empty() && !self.metrics.help_visible);
                shot = Some("ospf-challenge-light-150".into());
            }
            10035 => {
                self.scale = 2.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            10036 => shot = Some("ospf-challenge-dark-200".into()),
            10037 => {
                if !self.reveal_smoke_control(ctx, self.metrics.ospf_fields[0].1) {
                    return (false, None);
                }
            }
            10038 => shot = Some("ospf-fields-dark-200".into()),
            10039 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            10040 => {
                assert!(self.metrics.completion);
                shot = Some("ospf-completion-dark-200".into());
            }
            10041 => {
                self.reset_attempt(i);
                assert_eq!(self.progress.attempts[id].state, s.initial);
                self.scale = 1.;
                self.progress.attempts.get_mut(id).unwrap().challenge = false;
                self.set_ospf_view(i, teaching::View::Assignment);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            }
            10042 => shot = Some("ospf-reset-full-assignment".into()),
            10043 => {
                println!(
                    "PASS native private OSPF fixture: separate fill/check example; native router/interface pickers and field edits; malformed/partial/alternate retry; exact save/resume; selected reset; Challenge light150/dark200. No bundled lesson or live protocol acceptance."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => unreachable!(),
        }
        (true, shot)
    }
}
