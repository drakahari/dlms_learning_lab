//! Private fixture native-input exercise; no bundled account curriculum.
use super::*;
impl LabApp {
    pub(super) fn accounts_output_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        if stage == 7900 {
            self.scale = 1.;
            self.theme_id = "dlms-dark".into();
            theme::apply(ctx, &self.theme_id);
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            let i = self.add_accounts_fixture();
            self.launch(i);
            self.set_accounts_view(i, teaching::View::Assignment);
            return (true, None);
        }
        let i = self
            .scenarios
            .iter()
            .position(|s| s.id == "test-accounts-workspace")
            .unwrap();
        let phase = match stage {
            7901..=7940 => Some((stage - 7901) % 4),
            7941..=7944 => Some(stage - 7941),
            7947..=7986 => Some((stage - 7947) % 4),
            7987..=7990 => Some(stage - 7987),
            _ => None,
        };
        if let Some(phase) = phase {
            match phase {
                0 => {
                    if !self.click_visible_smoke_control(ctx, self.metrics.terminal) {
                        return (false, None);
                    }
                }
                1 => {
                    self.pending_events = vec![egui::Event::Text(
                        if matches!(stage, 7942 | 7988) {
                            "usermod -G"
                        } else {
                            "getent passwd"
                        }
                        .into(),
                    )]
                }
                2 => {
                    if !self.click_visible_smoke_control(ctx, self.metrics.accounts_run) {
                        return (false, None);
                    }
                }
                3 => {
                    if matches!(stage, 7944 | 7990) {
                        let (offset, viewport) = self
                            .metrics
                            .accounts_output_scroll
                            .expect("account output viewport");
                        let latest = self
                            .metrics
                            .accounts_latest_output
                            .expect("latest error row");
                        assert!(
                            offset > 0. && viewport.expand(1.).contains_rect(latest),
                            "latest error must be visible after a long transcript: offset={offset} viewport={viewport:?} row={latest:?}"
                        );
                        return (
                            true,
                            Some(
                                if stage == 7944 {
                                    "accounts-assignment-latest-error"
                                } else {
                                    "accounts-practice-latest-error"
                                }
                                .into(),
                            ),
                        );
                    }
                }
                _ => unreachable!(),
            }
        } else if stage == 7945 {
            self.set_accounts_view(i, teaching::View::Example);
        } else if stage == 7946 {
            assert!(
                self.progress.attempts[&self.scenarios[i].id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .accounts_learning
                    .as_ref()
                    .unwrap()
                    .transcript
                    .is_empty()
            );
        } else if stage == 7991 {
            println!(
                "PASS native virtual accounts output: repeated actual queries and latest errors follow in separate assignment/practice terminals."
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            unreachable!();
        }
        (true, None)
    }
    pub(super) fn accounts_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        if stage == 7800 {
            self.scale = 1.;
            self.theme_id = "dlms-dark".into();
            theme::apply(ctx, &self.theme_id);
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            let i = self.add_accounts_fixture();
            self.launch(i);
            return (true, None);
        }
        let i = self
            .scenarios
            .iter()
            .position(|s| s.id == "test-accounts-workspace")
            .unwrap();
        let s = self.scenarios[i].clone();
        let mut shot = None;
        match stage {
            7801 => {
                assert_eq!(self.accounts_view(i), teaching::View::Introduction);
                shot = Some("accounts-introduction".into());
            }
            7802 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.example_action) {
                    return (false, None);
                }
            }
            7803 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.insert) {
                    return (false, None);
                }
            }
            7804 => {
                let l = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .accounts_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(l.simulation, s.accounts_teaching.as_ref().unwrap().fixture);
                assert_eq!(l.command, "id tess");
                assert_eq!(self.progress.attempts[&s.id].state, s.initial);
                shot = Some("accounts-example-filled-only".into());
            }
            7805 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.accounts_run) {
                    return (false, None);
                }
            }
            7806 => {
                assert_eq!(self.progress.attempts[&s.id].state, s.initial);
                assert!(!progress::current_solved(
                    &self.progress.attempts[&s.id],
                    &s
                ));
            }
            7807 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.assignment_action) {
                    return (false, None);
                }
            }
            7808 | 7812 | 7816 | 7821 | 7826 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.terminal) {
                    return (false, None);
                }
            }
            7809 | 7813 | 7817 | 7822 | 7827 => {
                let command = match stage {
                    7809 => s.solution[0].clone(),
                    7813 => s.solution[1].clone(),
                    7817 => s.solution[2].clone(),
                    7822 => "usermod -G reports mira".into(),
                    _ => "usermod -G audit,reports mira".into(),
                };
                self.pending_events = vec![egui::Event::Text(command)];
            }
            7810 | 7814 | 7818 | 7823 | 7828 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.accounts_run) {
                    return (false, None);
                }
            }
            7811 => assert!(!progress::current_solved(
                &self.progress.attempts[&s.id],
                &s
            )),
            7815 => assert!(!progress::current_solved(
                &self.progress.attempts[&s.id],
                &s
            )),
            7819 | 7824 | 7829 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            7820 => {
                assert!(self.metrics.completion);
                shot = Some("accounts-assignment-current-success".into());
            }
            7825 => {
                assert!(!self.metrics.completion);
                shot = Some("accounts-required-membership-lost".into());
            }
            7830 => {
                assert!(self.metrics.completion);
                self.progress.attempts.get_mut(&s.id).unwrap().challenge = true;
            }
            7831 => {
                assert!(self.metrics.completion);
                assert!(self.metrics.lesson_tabs.is_empty());
                assert!(!self.metrics.help_visible);
                shot = Some("accounts-challenge-completion-retained".into());
            }
            7832 => {
                self.scale = 1.5;
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(720., 560.)));
            }
            7833 => shot = Some("accounts-assignment-150".into()),
            7834 => self.scale = 2.,
            7835 => shot = Some("accounts-assignment-200".into()),
            7836 => {
                self.progress.attempts.get_mut(&s.id).unwrap().challenge = false;
                self.set_accounts_view(i, teaching::View::Example);
            }
            7837 => shot = Some("accounts-separate-example-200".into()),
            7838 => {
                assert_eq!(
                    self.progress.attempts[&s.id]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .accounts_learning
                        .as_ref()
                        .unwrap()
                        .simulation,
                    s.accounts_teaching.as_ref().unwrap().fixture
                );
                println!(
                    "PASS native virtual accounts: input-only fill, separate practice, partial repair, current invalidation and optional Challenge at100/150/200."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => unreachable!(),
        }
        (true, shot)
    }
}
