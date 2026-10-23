//! Normal bundled account assignments; native input and private progress only.
use super::*;
use dlms_learning_lab::accounts;
use std::collections::BTreeMap;
impl LabApp {
    fn accounts_lesson_command(
        &mut self,
        ctx: &egui::Context,
        command: &str,
        phase: usize,
        keyboard: bool,
    ) -> bool {
        match phase {
            0 => return self.click_visible_smoke_control(ctx, self.metrics.terminal),
            1 => {
                self.pending_events = vec![
                    egui::Event::Key {
                        key: egui::Key::A,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers {
                            ctrl: true,
                            command: true,
                            ..Default::default()
                        },
                    },
                    egui::Event::Text(command.into()),
                ];
            }
            2 if keyboard => {
                self.pending_events = vec![
                    egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: Default::default(),
                    },
                    egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers: Default::default(),
                    },
                ];
            }
            2 => return self.click_visible_smoke_control(ctx, self.metrics.accounts_run),
            3 => {}
            _ => unreachable!(),
        }
        true
    }
    pub(super) fn accounts_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        const IDS: [&str; 4] = [
            "lpic1-account-onboarding",
            "lpic1-account-membership",
            "lpic1-account-retirement",
            "lpic1-account-team-handover",
        ];
        let ordinal = (stage - 9000) / 200;
        let phase = (stage - 9000) % 200;
        let id = IDS[ordinal];
        let i = self
            .scenarios
            .iter()
            .position(|s| s.id == id)
            .expect("normal bundled account lesson");
        let s = self.scenarios[i].clone();
        let lesson = s.accounts_teaching.as_ref().unwrap();
        assert!(
            self.units
                .iter()
                .any(|u| u.id == "local-account-lifecycle" && u.labs.iter().any(|v| v == id))
        );
        let plans: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/accounts-content-review.json"
        ))
        .unwrap();
        let plan = plans["plans"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == id)
            .unwrap();
        let wrong: Vec<String> =
            serde_json::from_value(plan["negatives"][0]["commands"].clone()).unwrap();
        let alternate: Vec<String> =
            serde_json::from_value(plan["alternatives"][0].clone()).unwrap();
        assert!(wrong.len() <= 6 && alternate.len() <= 6 && lesson.example.len() <= 8);
        let mut shot = None;
        let command = match phase {
            5..=36 => lesson
                .example
                .get((phase - 5) / 4)
                .map(|s| (s.as_str(), (phase - 5) % 4, false)),
            39..=42 => Some((lesson.try_actions[0].as_str(), phase - 39, false)),
            45..=48 => Some((lesson.repair[0].as_str(), phase - 45, false)),
            55..=58 => Some(("usermod -G", phase - 55, false)),
            60..=83 => wrong
                .get((phase - 60) / 4)
                .map(|s| (s.as_str(), (phase - 60) % 4, false)),
            89..=112 => alternate
                .get((phase - 89) / 4)
                .map(|s| (s.as_str(), (phase - 89) % 4, true)),
            116..=119 => Some((plan["break_command"].as_str().unwrap(), phase - 116, false)),
            124..=127 => Some((plan["repair_command"].as_str().unwrap(), phase - 124, true)),
            132..=135 => Some(("passwd unavailable", phase - 132, false)),
            _ => None,
        };
        if let Some((command, part, keyboard)) = command {
            if !self.accounts_lesson_command(ctx, command, part, keyboard) {
                return (false, None);
            }
            return (true, None);
        }
        match phase {
            0 => {
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.filter = Filter {
                    track: "LPIC-1".into(),
                    unit: "local-account-lifecycle".into(),
                    ..Default::default()
                };
                self.launch(i);
                self.progress.favorites.insert(id.into());
            }
            1 => {
                assert_eq!(self.accounts_view(i), teaching::View::Introduction);
                shot = Some(format!("{id}-introduction"));
            }
            2 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.example_action) {
                    return (false, None);
                }
            }
            3 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.insert) {
                    return (false, None);
                }
            }
            4 => {
                let a = &self.progress.attempts[id];
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .accounts_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(l.simulation, lesson.fixture);
                assert_eq!(l.command, lesson.example[0]);
                assert!(l.transcript.is_empty());
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, &s));
            }
            37 | 43 | 49 | 53 | 84 | 113 | 120 | 128 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            38 | 44 | 50 => {
                let a = &self.progress.attempts[id];
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .accounts_learning
                    .as_ref()
                    .unwrap();
                assert!(l.checked);
                if phase == 44 {
                    assert!(l.checks(lesson).iter().any(|c| !c.0));
                    assert!(
                        accounts::checks(&lesson.fixture, &l.simulation, &lesson.try_outcomes)
                            .iter()
                            .all(|c| c.0)
                    );
                } else {
                    assert!(l.checks(lesson).iter().all(|c| c.0));
                }
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, &s));
                if phase != 50 {
                    shot = Some(format!(
                        "{id}-example-{}",
                        if phase == 44 {
                            "experiment"
                        } else {
                            "ungraded"
                        }
                    ));
                }
            }
            51 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.assignment_action) {
                    return (false, None);
                }
            }
            52 => {
                assert_eq!(self.accounts_view(i), teaching::View::Assignment);
                shot = Some(format!("{id}-full-assignment"));
            }
            54 => {
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
                shot = Some(format!("{id}-initial-feedback"));
            }
            59 => {
                let a = &self.progress.attempts[id];
                assert_eq!(a.state, s.initial);
                assert!(
                    a.transcript
                        .last()
                        .unwrap()
                        .contains("requires a GROUP_LIST and USER")
                );
                shot = Some(format!("{id}-missing-operands"));
            }
            85 => {
                let a = &self.progress.attempts[id];
                let checks = engine::checks(&a.state, &s.goal);
                assert!(checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0));
                assert!(!progress::current_solved(a, &s));
                shot = Some(format!("{id}-partial-feedback"));
            }
            86 | 138 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.reset) {
                    return (false, None);
                }
            }
            87 | 139 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.confirm_reset) {
                    return (false, None);
                }
            }
            88 => {
                assert_eq!(self.progress.attempts[id].state, s.initial);
                self.set_accounts_view(i, teaching::View::Assignment);
            }
            114 => {
                assert!(
                    self.metrics.completion
                        && progress::current_solved(&self.progress.attempts[id], &s)
                );
                assert_eq!(self.progress.attempts[id].step, 0);
                let exact = serde_json::to_vec(&self.progress.attempts[id]).unwrap();
                progress::save(&self.path, &self.progress).unwrap();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_vec(&self.progress.attempts[id]).unwrap(),
                    exact
                );
                self.launch(i);
                self.set_accounts_view(i, teaching::View::Assignment);
            }
            115 => {
                assert!(self.metrics.completion);
                shot = Some(format!("{id}-alternate-complete-resumed"));
            }
            121 => {
                assert!(
                    !self.metrics.completion
                        && !progress::current_solved(&self.progress.attempts[id], &s)
                );
                shot = Some(format!("{id}-success-invalidated"));
            }
            122 => {
                self.progress.attempts.get_mut(id).unwrap().challenge = true;
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            123 => {
                assert!(self.metrics.lesson_tabs.is_empty() && !self.metrics.help_visible);
                shot = Some(format!("{id}-challenge-light-150"));
            }
            129 => {
                assert!(
                    self.metrics.completion
                        && progress::current_solved(&self.progress.attempts[id], &s)
                );
                assert!(self.metrics.lesson_tabs.is_empty() && !self.metrics.help_visible);
                shot = Some(format!("{id}-challenge-repaired-light-150"));
            }
            130 => {
                self.scale = 2.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                self.lesson_scroll_reset = true;
            }
            131 => shot = Some(format!("{id}-challenge-dark-200")),
            136 => {
                assert!(progress::current_solved(&self.progress.attempts[id], &s));
                assert!(
                    self.progress.attempts[id]
                        .transcript
                        .last()
                        .unwrap()
                        .contains("not supported")
                );
                shot = Some(format!("{id}-unsupported-command-200"));
            }
            137 => {
                let exact = serde_json::to_vec(&self.progress.attempts[id]).unwrap();
                progress::save(&self.path, &self.progress).unwrap();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_vec(&self.progress.attempts[id]).unwrap(),
                    exact
                );
                let others: BTreeMap<_, _> = self
                    .progress
                    .attempts
                    .iter()
                    .filter(|(key, _)| key.as_str() != id)
                    .collect();
                ctx.data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new(("accounts-unit-others", id)),
                        serde_json::to_vec(&others).unwrap(),
                    )
                });
            }
            140 => {
                assert_eq!(self.progress.attempts[id].state, s.initial);
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
                assert!(self.progress.favorites.contains(id));
                let others: BTreeMap<_, _> = self
                    .progress
                    .attempts
                    .iter()
                    .filter(|(key, _)| key.as_str() != id)
                    .collect();
                assert_eq!(serde_json::to_vec(&others).unwrap(), ctx.data(|d| d.get_temp::<Vec<u8>>(egui::Id::new(("accounts-unit-others", id)))).unwrap());
            }
            141 => {
                println!(
                    "PASS native authored accounts {id}: introduction, separate insert/run/experiment/repair, full task, missing operands, partial/alternate grading, keyboard Enter, Step0 completion, exact resume, invalidation, Challenge Light150/Dark200, repair and selected reset."
                );
                if ordinal < 3 {
                    self.smoke_stage = 9000 + (ordinal + 1) * 200 - 1;
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            _ => {}
        }
        (true, shot)
    }
}
