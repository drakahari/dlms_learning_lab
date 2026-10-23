//! Four fixed teaching fixtures, exercised through the native editor and Run controls.
use super::*;
impl LabApp {
    pub(super) fn capture_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        const IDS: [&str; 4] = [
            "shell-capture-preview",
            "shell-capture-evidence",
            "shell-capture-decision",
            "shell-capture-handover",
        ];
        let ordinal = (stage - 7200) / 100;
        let phase = (stage - 7200) % 100;
        assert!(
            IDS.iter()
                .all(|id| self.scenarios.iter().any(|s| &s.id == id)),
            "Authored capture lessons must be bundled in the normal app"
        );
        assert!(self.units.iter().any(|u| u.id == "shell-command-capture"));
        let id = IDS[ordinal];
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        let scenario = self.scenarios[i].clone();
        let lesson = scenario.teaching.as_ref().unwrap();
        let rows: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/shell-capture-content-review.json"
        ))
        .unwrap();
        let review = rows
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
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
        let mut shot = None;
        if std::env::var_os("DLMS_SMOKE_DIAGNOSTICS").is_some() {
            eprintln!("capture-lesson {id} phase {phase}");
        }
        match phase {
            0 => {
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.launch(i);
                self.set_script_view(i, teaching::View::Introduction);
                self.progress.favorites.insert(id.into());
            }
            1 => {
                assert_eq!(self.script_view(i), teaching::View::Introduction);
                shot = Some(format!("{id}-introduction"));
            }
            2 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.example_action) {
                    return (false, None);
                }
            }
            3 => {
                assert_eq!(self.script_view(i), teaching::View::Example);
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
            }
            4 | 8 | 14 | 18 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            5 => {
                if self.script_job.is_some() {
                    return (false, None);
                }
                let learning = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap();
                assert!(
                    lesson
                        .fixture
                        .accepts(learning.example_result.as_ref().unwrap())
                );
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
                shot = Some(format!("{id}-ungraded-example"));
            }
            6 | 12 | 16 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.editor_viewport) {
                    return (false, None);
                }
            }
            7 => {
                self.pending_events = vec![
                    ctrl(egui::Key::A),
                    egui::Event::Text(review["try_change"]["source"].as_str().unwrap().into()),
                ];
            }
            9 => {
                if self.script_job.is_some() {
                    return (false, None);
                }
                let expected: shell::Case =
                    serde_json::from_value(review["try_change"]["expected"].clone()).unwrap();
                let learning = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap();
                assert!(expected.accepts(learning.example_result.as_ref().unwrap()));
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
            }
            10 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.assignment_action) {
                    return (false, None);
                }
            }
            11 => {
                assert_eq!(self.script_view(i), teaching::View::Assignment);
                shot = Some(format!("{id}-full-assignment"));
            }
            13 => {
                let partial = review["wrong"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|w| w["name"] == review["partial"])
                    .unwrap()["source"]
                    .as_str()
                    .unwrap();
                self.pending_events = vec![ctrl(egui::Key::A), egui::Event::Text(partial.into())];
            }
            15 => {
                if self.script_job.is_some() {
                    return (false, None);
                }
                let engine::State::Script {
                    run: Some(record), ..
                } = &self.progress.attempts[id].state
                else {
                    panic!("partial run missing")
                };
                let engine::Goal::Script { cases } = &scenario.goal else {
                    panic!()
                };
                let passes: Vec<_> = cases
                    .iter()
                    .zip(&record.results)
                    .map(|(c, r)| c.accepts(r))
                    .collect();
                assert!(
                    passes.contains(&true) && passes.contains(&false),
                    "{id} partial {passes:?}"
                );
                assert!(!self.metrics.completion);
                shot = Some(format!("{id}-partial-feedback"));
            }
            17 => {
                self.pending_events = vec![
                    ctrl(egui::Key::A),
                    egui::Event::Text(scenario.solution[0].clone()),
                ];
            }
            19 => {
                if self.script_job.is_some() {
                    return (false, None);
                }
                assert!(progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                assert_eq!(self.progress.attempts[id].step, 0);
                shot = Some(format!("{id}-complete"));
            }
            20 => {
                let before = serde_json::to_vec(&self.progress.attempts).unwrap();
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(serde_json::to_vec(&self.progress.attempts).unwrap(), before);
                assert!(self.progress.favorites.contains(id));
                self.set_script_view(i, teaching::View::Example);
            }
            21 => {
                let learning = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(
                    learning.example_source,
                    review["try_change"]["source"].as_str().unwrap()
                );
                assert!(learning.example_result.is_some());
                assert!(progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                self.set_script_view(i, teaching::View::Assignment);
                self.progress.attempts.get_mut(id).unwrap().challenge = true;
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 840.)));
            }
            22 => {
                assert!(self.metrics.lesson_tabs.is_empty());
                assert!(!self.metrics.help_visible);
                shot = Some(format!("{id}-challenge-light150"));
            }
            23 | 26 => {
                if !self.reveal_smoke_control(ctx, self.metrics.check.unwrap()) {
                    return (false, None);
                }
            }
            24 => {
                shot = Some(format!("{id}-challenge-light150-run"));
            }
            25 => {
                self.scale = 2.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            27 => {
                shot = Some(format!("{id}-challenge-dark200-run"));
            }
            28 => {
                let a = self.progress.attempts.get_mut(id).unwrap();
                let engine::State::Script { source, .. } = &mut a.state else {
                    panic!()
                };
                source.push_str("\nprintf broken");
                assert!(!progress::current_solved(a, &scenario));
                self.dirty = true;
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                self.scale = 1.;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.lesson_scroll_reset = true;
            }
            29 => {
                let others: std::collections::BTreeMap<_, _> = self
                    .progress
                    .attempts
                    .iter()
                    .filter(|(other, _)| other.as_str() != id)
                    .map(|(id, a)| (id.clone(), serde_json::to_vec(a).unwrap()))
                    .collect();
                ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new("capture-unit-other-attempts"), others)
                });
            }
            30 => {
                if !self.click_smoke_control(ctx, self.metrics.reset) {
                    return (false, None);
                }
            }
            31 => {
                if !self.click_smoke_control(ctx, self.metrics.confirm_reset) {
                    return (false, None);
                }
            }
            32 => {
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                let others = ctx
                    .data(|d| {
                        d.get_temp::<std::collections::BTreeMap<String, Vec<u8>>>(egui::Id::new(
                            "capture-unit-other-attempts",
                        ))
                    })
                    .unwrap();
                for (id, bytes) in others {
                    assert_eq!(
                        serde_json::to_vec(&self.progress.attempts[&id]).unwrap(),
                        bytes
                    );
                }
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
            }
            33 => {
                self.selected = None;
                self.filter.track = "Bash/Shell".into();
                self.filter.unit = "shell-command-capture".into();
            }
            34 => {
                shot = Some(format!("{id}-library-resume"));
            }
            35 => {
                if ordinal == 3 {
                    println!(
                        "PASS native four command-capture lessons: prerequisite teaching, actual separate example Run/change, full assignment, partial feedback, keyboard repair/Run, Step0 success, exact resume and example drafts, current invalidation, selected Reset, Challenge light150/dark200 and library. Synthetic authored inputs and injected keyboard events; no novice/physical-device/screen-reader/other-platform acceptance."
                    );
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                } else {
                    self.smoke_stage = 7200 + (ordinal + 1) * 100 - 1;
                }
            }
            _ => panic!("Unexpected capture lesson phase {phase}"),
        }
        (true, shot)
    }
}
