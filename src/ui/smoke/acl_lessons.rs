//! Normal bundled ACL lessons, actual native controls and a private progress profile.
use super::*;
use dlms_learning_lab::{acl_editor as editor, acl_workspace::Action};
use std::collections::BTreeMap;
impl LabApp {
    fn acl_native_documents(
        &mut self,
        ctx: &egui::Context,
        i: usize,
        practice: bool,
        documents: &editor::Drafts,
        phase: usize,
    ) -> bool {
        assert!(
            documents.routers.len() <= 2,
            "fixed authored-unit smoke bound"
        );
        let Some((router, text)) = documents.routers.iter().nth(phase / 5) else {
            return true;
        };
        let w = self.progress.attempts[&self.scenarios[i].id]
            .workspace
            .as_ref()
            .unwrap();
        let draft = if practice {
            &w.acl_learning.as_ref().unwrap().draft
        } else {
            w.acl_draft.as_ref().unwrap()
        };
        let selected = draft.selected_router.as_deref() == Some(router.as_str());
        match phase % 5 {
            0 => {
                if !selected {
                    return self.click_visible_smoke_control(ctx, self.metrics.acl_router);
                }
            }
            1 => {
                if !selected {
                    let rect = self
                        .metrics
                        .acl_router_items
                        .iter()
                        .find(|(name, _)| name == router)
                        .map(|(_, r)| *r)
                        .expect("native router menu contains the supplied router");
                    return self.click_visible_smoke_control(ctx, Some(rect));
                }
            }
            2 => return self.click_visible_smoke_control(ctx, self.metrics.editor_viewport),
            3 => {
                let ctrl = egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                };
                self.pending_events = vec![egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: ctrl,
                }];
                if text.is_empty() {
                    self.pending_events.push(egui::Event::Key {
                        key: egui::Key::Backspace,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: Default::default(),
                    });
                } else {
                    self.pending_events.push(egui::Event::Text(text.clone()));
                }
            }
            4 => assert_eq!(
                &draft.documents.routers[router], text,
                "native text input must update only the selected document"
            ),
            _ => unreachable!(),
        }
        true
    }
    pub(super) fn acl_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        const IDS: [&str; 4] = [
            "ccna-acl-source-policy",
            "ccna-acl-service-order",
            "ccna-acl-direction-repair",
            "ccna-acl-policy-handover",
        ];
        let ordinal = (stage - 8200) / 100;
        let phase = (stage - 8200) % 100;
        let id = IDS[ordinal];
        let i = self
            .scenarios
            .iter()
            .position(|s| s.id == id)
            .expect("authored ACL lesson must be bundled in the normal app");
        assert!(
            self.units
                .iter()
                .any(|u| u.id == "ipv4-access-policy" && u.labs.iter().any(|x| x == id))
        );
        let s = self.scenarios[i].clone();
        let lesson = s.acl_teaching.as_ref().unwrap();
        let Action::ApplyDocuments {
            documents: reference,
        } = serde_json::from_str(s.solution[0].strip_prefix("acl-author ").unwrap()).unwrap();
        let plans: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/acl-content-review.json"
        ))
        .unwrap();
        let plan = plans["plans"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == id)
            .unwrap();
        let wrong: editor::Drafts =
            serde_json::from_value(plan["negatives"][0]["documents"].clone()).unwrap();
        let alternate: editor::Drafts =
            serde_json::from_value(plan["alternatives"][0]["documents"].clone()).unwrap();
        let mut shot = None;
        match phase {
            0 => {
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.filter = Filter {
                    track: "CCNA".into(),
                    unit: "ipv4-access-policy".into(),
                    ..Default::default()
                };
                self.launch(i);
                self.progress.favorites.insert(id.into());
            }
            1 => {
                assert_eq!(self.acl_view(i), teaching::View::Introduction);
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
                let l = a.workspace.as_ref().unwrap().acl_learning.as_ref().unwrap();
                assert_eq!(l.network, lesson.fixture);
                assert!(!l.checked);
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, &s));
            }
            5 | 20 | 23 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            6 | 24 => {
                let a = &self.progress.attempts[id];
                let l = a.workspace.as_ref().unwrap().acl_learning.as_ref().unwrap();
                assert!(l.checked && l.checks(lesson).iter().all(|c| c.0));
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, &s));
                if phase == 6 {
                    shot = Some(format!("{id}-ungraded-example"));
                }
            }
            10..=19 => {
                if !self.acl_native_documents(
                    ctx,
                    i,
                    true,
                    &lesson.try_change.documents,
                    phase - 10,
                ) {
                    return (false, None);
                }
            }
            21 => {
                let a = &self.progress.attempts[id];
                let l = a.workspace.as_ref().unwrap().acl_learning.as_ref().unwrap();
                for (case, check) in lesson.definition.cases.iter().zip(l.checks(lesson)) {
                    assert_eq!(
                        check.0, lesson.try_change.expected[&case.id],
                        "{id} {}",
                        case.id
                    );
                }
                assert_eq!(a.state, s.initial);
                shot = Some(format!("{id}-example-experiment"));
            }
            22 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.insert) {
                    return (false, None);
                }
            }
            25 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.assignment_action) {
                    return (false, None);
                }
            }
            26 => {
                assert_eq!(self.acl_view(i), teaching::View::Assignment);
                shot = Some(format!("{id}-full-assignment"));
            }
            27 | 40 | 55 | 67 | 72 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            28 => {
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
            }
            30..=39 => {
                if !self.acl_native_documents(ctx, i, false, &wrong, phase - 30) {
                    return (false, None);
                }
            }
            41 => {
                let a = &self.progress.attempts[id];
                let engine::State::Ipv4Acl { network } = &a.state else {
                    panic!()
                };
                let engine::Goal::Ipv4Acl { definition } = &s.goal else {
                    panic!()
                };
                let checks = dlms_learning_lab::acl_workspace::checks(
                    network,
                    definition,
                    &a.workspace
                        .as_ref()
                        .unwrap()
                        .acl_draft
                        .as_ref()
                        .unwrap()
                        .documents,
                );
                assert!(checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0));
                assert!(!progress::current_solved(a, &s));
                shot = Some(format!("{id}-partial-feedback"));
            }
            45..=54 => {
                if !self.acl_native_documents(ctx, i, false, &alternate, phase - 45) {
                    return (false, None);
                }
            }
            56 => {
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
                self.set_acl_view(i, teaching::View::Assignment);
            }
            57 => {
                assert!(self.metrics.completion);
                shot = Some(format!("{id}-alternate-complete-resumed"));
            }
            60 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.editor_viewport) {
                    return (false, None);
                }
            }
            61 => {
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
                    egui::Event::Text("pending invalid document".into()),
                ];
            }
            62 => {
                assert!(
                    !self.checked
                        && !self.metrics.completion
                        && !progress::current_solved(&self.progress.attempts[id], &s)
                );
            }
            63 => {
                self.progress.attempts.get_mut(id).unwrap().challenge = true;
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            64 => {
                assert!(self.metrics.lesson_tabs.is_empty() && !self.metrics.help_visible);
                shot = Some(format!("{id}-challenge-light-150"));
            }
            65 => {
                self.scale = 2.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                self.lesson_scroll_reset = true;
            }
            66 => {
                shot = Some(format!("{id}-challenge-dark-200"));
            }
            68 => {
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
                shot = Some(format!("{id}-invalid-document-feedback-200"));
            }
            70 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.editor_viewport) {
                    return (false, None);
                }
            }
            71 => {
                let router = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .acl_draft
                    .as_ref()
                    .unwrap()
                    .selected_router
                    .as_ref()
                    .unwrap();
                let text = alternate
                    .routers
                    .get(router)
                    .or_else(|| reference.routers.get(router))
                    .expect("edited router belongs to the applied method");
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
                    egui::Event::Text(text.clone()),
                ];
            }
            73 => {
                assert!(progress::current_solved(&self.progress.attempts[id], &s));
                let others: BTreeMap<_, _> = self
                    .progress
                    .attempts
                    .iter()
                    .filter(|(name, _)| name.as_str() != id)
                    .collect();
                ctx.data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new(("acl-unit-others", id)),
                        serde_json::to_vec(&others).unwrap(),
                    )
                });
            }
            74 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.reset) {
                    return (false, None);
                }
            }
            75 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.confirm_reset) {
                    return (false, None);
                }
            }
            76 => {
                assert_eq!(self.progress.attempts[id].state, s.initial);
                assert!(!progress::current_solved(&self.progress.attempts[id], &s));
                assert!(self.progress.favorites.contains(id));
                let others: BTreeMap<_, _> = self
                    .progress
                    .attempts
                    .iter()
                    .filter(|(name, _)| name.as_str() != id)
                    .collect();
                assert_eq!(
                    serde_json::to_vec(&others).unwrap(),
                    ctx.data(|d| d.get_temp::<Vec<u8>>(egui::Id::new(("acl-unit-others", id))))
                        .unwrap()
                );
            }
            80 => {
                println!(
                    "PASS native authored ACL {id}: separate fill/check/experiment/repair, full task, actual router picker and document editing, partial/alternate grading, Step0 completion, exact resume, invalidation, Challenge light150/dark200, repair and selected reset."
                );
                if ordinal < 3 {
                    self.smoke_stage = 8200 + (ordinal + 1) * 100 - 1;
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            _ => {}
        }
        (true, shot)
    }
}

impl LabApp {
    pub(super) fn acl_lessons_layout(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        const IDS: [&str; 4] = [
            "ccna-acl-source-policy",
            "ccna-acl-service-order",
            "ccna-acl-direction-repair",
            "ccna-acl-policy-handover",
        ];
        if stage >= 8736 {
            match stage {
                8736 => {
                    self.selected = None;
                    self.filter = Filter {
                        track: "CCNA".into(),
                        unit: "ipv4-access-policy".into(),
                        ..Default::default()
                    };
                    self.extra_filters = true;
                    self.scale = 1.5;
                    self.theme_id = "dlms-light".into();
                    theme::apply(ctx, &self.theme_id);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                }
                8737 => return (true, Some("acl-library-light-150".into())),
                8738 => {
                    self.filter.difficulty = "Advanced".into();
                    self.page = 0;
                }
                8739 => {
                    assert!(!self.metrics.pagination);
                    return (true, Some("acl-library-advanced-filter".into()));
                }
                8740 => {
                    println!(
                        "PASS native ACL compact table and library layouts: fourfull tasks100/light150/dark200, visibleCheckandfeedback, sharedadvisoryAdvancedfilter."
                    );
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                _ => unreachable!(),
            }
            return (true, None);
        }
        let ordinal = (stage - 8700) / 9;
        let phase = (stage - 8700) % 9;
        let id = IDS[ordinal];
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        let mut shot = None;
        match phase {
            0 => {
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.launch(i);
                self.set_acl_view(i, teaching::View::Assignment);
            }
            1 => {
                assert!(
                    self.metrics.check.unwrap().bottom() <= ctx.content_rect().bottom() + 1.,
                    "{id} default wide Check should fit with compact read-only rows: {:?} vs {:?}",
                    self.metrics.check,
                    ctx.content_rect()
                );
                shot = Some(format!("{id}-compact-full-task-100"));
            }
            2 => {
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            3 => {
                shot = Some(format!("{id}-compact-light-150"));
            }
            4 => {
                self.scale = 2.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                self.lesson_scroll_reset = true;
            }
            5 => {
                shot = Some(format!("{id}-compact-tiled-200"));
            }
            6 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            7 => {
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &self.scenarios[i]
                ));
                shot = Some(format!("{id}-compact-check-feedback-200"));
            }
            8 => {}
            _ => unreachable!(),
        }
        (true, shot)
    }
}
