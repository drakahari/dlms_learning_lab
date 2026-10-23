//! Actual Fedora native renderer/input exercise; private unbundled policy fixture.
use super::*;
impl LabApp {
    pub(super) fn acl_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
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
        let id = "test-acl-workspace";
        if matches!(stage, 7600 | 7660) {
            self.scale = 1.;
            self.theme_id = "dlms-dark".into();
            theme::apply(ctx, &self.theme_id);
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            let i = self.add_acl_fixture();
            self.launch(i);
            if stage == 7660 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.set_acl_view(i, teaching::View::Assignment);
            }
            return (true, None);
        }
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        let s = self.scenarios[i].clone();
        match stage {
            7601 => {
                assert_eq!(self.acl_view(i), teaching::View::Introduction);
                shot = Some("acl-introduction".into());
            }
            7602 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.example_action) {
                    return (false, None);
                }
            }
            7603 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.insert) {
                    return (false, None);
                }
            }
            7604 => {
                let l = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .acl_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(l.network, s.acl_teaching.as_ref().unwrap().fixture);
                assert!(!l.checked);
                assert_eq!(self.progress.attempts[id].state, s.initial);
                shot = Some("acl-example-filled-unapplied".into());
            }
            7605 | 7609 | 7613 | 7617 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            7606 => {
                let l = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .acl_learning
                    .as_ref()
                    .unwrap();
                assert!(
                    l.checked
                        && l.checks(s.acl_teaching.as_ref().unwrap())
                            .iter()
                            .all(|c| c.0)
                );
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
                shot = Some("acl-example-ungraded-success".into());
            }
            7607 | 7615 | 7619 | 7624 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.editor_viewport) {
                    return (false, None);
                }
            }
            7608 => {
                let text = s
                    .acl_teaching
                    .as_ref()
                    .unwrap()
                    .try_change
                    .documents
                    .routers
                    .values()
                    .next()
                    .unwrap();
                self.pending_events = vec![
                    ctrl(egui::Key::A),
                    egui::Event::Text(text.clone()),
                    egui::Event::Key {
                        key: egui::Key::Backspace,
                        physical_key: None,
                        pressed: text.is_empty(),
                        repeat: false,
                        modifiers: Default::default(),
                    },
                ];
            }
            7610 => {
                let lesson = s.acl_teaching.as_ref().unwrap();
                let l = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .acl_learning
                    .as_ref()
                    .unwrap();
                let checks = l.checks(lesson);
                assert!(checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0));
                assert_eq!(self.progress.attempts[id].state, s.initial);
                shot = Some("acl-example-investigation-feedback".into());
            }
            7611 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.assignment_action) {
                    return (false, None);
                }
            }
            7612 => {
                assert_eq!(self.acl_view(i), teaching::View::Assignment);
                shot = Some("acl-full-assignment".into());
            }
            7614 => {
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
                shot = Some("acl-initial-feedback".into());
            }
            7616 => {
                self.pending_events=vec![ctrl(egui::Key::A),egui::Event::Text("ip access-list extended partial\n permit tcp any any eq 443\nexit\ninterface office\n ip access-group partial in".into())];
            }
            7618 => {
                let engine::State::Ipv4Acl { network } = &self.progress.attempts[id].state else {
                    panic!()
                };
                let engine::Goal::Ipv4Acl { definition } = &s.goal else {
                    panic!()
                };
                let w = self.progress.attempts[id].workspace.as_ref().unwrap();
                let checks = dlms_learning_lab::acl_workspace::checks(
                    network,
                    definition,
                    &w.acl_draft.as_ref().unwrap().documents,
                );
                assert!(checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0));
                shot = Some("acl-partial-feedback".into());
            }
            7620 => {
                self.pending_events=vec![ctrl(egui::Key::A),egui::Event::Text("ip access-list extended alternative\n deny tcp any any eq 22\n permit ip any any\nexit\ninterface office\n ip access-group alternative in".into())];
            }
            7621 => {
                self.pending_events = vec![ctrl(egui::Key::Enter)];
            }
            7622 => {
                assert!(progress::current_solved(&self.progress.attempts[id], &s));
                assert!(self.metrics.completion);
                let bytes = serde_json::to_vec(&self.progress.attempts[id]).unwrap();
                progress::save(&self.path, &self.progress).unwrap();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_vec(&self.progress.attempts[id]).unwrap(),
                    bytes
                );
                self.launch(i);
                self.set_acl_view(i, teaching::View::Assignment);
            }
            7623 => {
                assert!(self.metrics.completion);
                shot = Some("acl-complete-and-resumed".into());
            }
            7625 => {
                self.pending_events = vec![
                    ctrl(egui::Key::A),
                    egui::Event::Text("pending invalid edit".into()),
                ];
            }
            7626 => {
                assert!(!self.checked && !self.metrics.completion);
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
                shot = Some("acl-pending-edit-invalidates-success".into());
            }
            7627 => {
                self.progress.attempts.get_mut(id).unwrap().challenge = true;
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 900.)));
            }
            7628 => {
                assert!(self.metrics.lesson_tabs.is_empty() && !self.metrics.help_visible);
                shot = Some("acl-challenge-light-150".into());
            }
            7629 => {
                self.scale = 2.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            7630 => {
                shot = Some("acl-challenge-dark-200".into());
            }
            7631 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            7632 => {
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
                shot = Some("acl-narrow-error-feedback-dark-200".into());
            }
            7633 => {
                self.reset_attempt(i);
                assert_eq!(self.progress.attempts[id].state, s.initial);
                println!(
                    "PASS native ACL workspace: separate example fill/apply/change, full assignment, partial and alternate grading, native Ctrl+Enter, exact save/resume, stale-success invalidation, selected reset, Challenge light150/dark200. Private unbundled fixture; no new authored lab count."
                );
                self.progress.attempts.get_mut(id).unwrap().challenge = false;
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.set_acl_view(i, teaching::View::Assignment);
            }
            7634 => {
                shot = Some("acl-wide-full-assignment-100".into());
            }
            7635 => {
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            7636 => {
                shot = Some("acl-wide-full-assignment-light-150".into());
            }
            7637 => {
                self.scale = 2.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            7638 => {
                shot = Some("acl-wide-full-assignment-dark-200".into());
            }
            7639 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            7661 => {
                shot = Some("acl-readable-table-wide-100".into());
            }
            7662 => {
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            7663 => {
                shot = Some("acl-readable-table-light-150".into());
            }
            7664 => {
                self.scale = 2.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            7665 => {
                shot = Some("acl-readable-table-dark-200".into());
            }
            7666 => {
                println!(
                    "PASS native ACL readable table: source and destination use bundled-font plain text at100/150/200; task rows visible by default."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => unreachable!(),
        }
        (true, shot)
    }
}
