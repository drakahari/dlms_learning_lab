//! Opt-in native known-template editing. Learner text never becomes a host command.
use super::*;
use dlms_learning_lab::{
    accounts,
    accounts_template::{Drafts, Step},
};
pub(super) fn clear_save_status(ctx: &egui::Context, lab: &str, scope: &str) {
    let id = egui::Id::new(("account-template-save-status", (lab, scope)));
    ctx.data_mut(|d| d.remove::<Result<(), String>>(id));
}

pub(super) fn worked(ui: &mut egui::Ui, steps: &[Step]) {
    if steps.is_empty() {
        return;
    }
    ui.strong("Worked template changes");
    for step in steps {
        match step {
            Step::Save {
                path,
                contents,
                mode,
            } => {
                lesson_presentation::monospace(ui, &format!("{path} · mode {mode:04o}"), true);
                lesson_presentation::monospace(ui, contents, true);
            }
            Step::Remove { path } => {
                lesson_presentation::monospace(ui, &format!("Remove {path}"), true);
            }
        }
    }
}

/// Returns true only for an edit or Save attempt, not ordinary selection changes.
/// Callers clear stale grading; the source-aware progress guard also checks drafts.
pub(super) fn editor(
    ui: &mut egui::Ui,
    key: (&str, &str),
    source: &accounts::State,
    current: &mut accounts::State,
    drafts: &mut Drafts,
    metrics: &mut Metrics,
) -> bool {
    if !drafts.valid_for(source, current) {
        ui.colored_label(
            palette(ui).error,
            "Template context differs; saved work retained.",
        );
        return false;
    }
    ui.strong("Starter-file editor");
    let status_id = egui::Id::new(("account-template-save-status", key));
    let previous_selection = drafts.selected.clone();
    controls::form_row(ui, "Template file", |ui, width| {
        let selected = drafts.selected.clone();
        let r = controls::dropdown(ui, ("account-template-path", key), width)
            .selected_text(&selected)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for path in drafts.leaves.keys() {
                    let row = ui.selectable_value(&mut drafts.selected, path.clone(), path);
                    metrics
                        .seed_fields
                        .push((format!("template-choice:{path}"), row.rect, row.id));
                }
                metrics.accounts_template_popup = Some(controls::popup_content_clip(ui));
            });
        name_control(&r.response, "Starter file to edit");
        metrics
            .seed_fields
            .push(("template-selector".into(), r.response.rect, r.response.id));
    });
    if previous_selection != drafts.selected {
        ui.ctx()
            .data_mut(|d| d.remove::<Result<(), String>>(status_id));
    }
    let path = drafts.selected.clone();
    let draft = drafts.leaves.get_mut(&path).expect("validated known leaf");
    let mut changed = false;
    let optional = source.policy().editable_skeleton_leaves[&path].allow_remove;
    let r = ui.add_enabled(
        optional,
        egui::Checkbox::new(&mut draft.included, "Include in new homes"),
    );
    name_control(&r, "Include selected starter file in new homes");
    metrics
        .seed_fields
        .push(("template-include".into(), r.rect, r.id));
    changed |= r.changed();
    ui.add_enabled_ui(draft.included, |ui| {
        controls::form_row(ui, "Permissions (octal)", |ui, width| {
            let r = ui.add_sized(
                [width.min(170.), controls::height(ui)],
                controls::singleline(ui, &mut draft.mode)
                    .id_salt(("account-template-mode", key, &path))
                    .hint_text("0000–0777")
                    .char_limit(4),
            );
            name_control(&r, "Starter file permissions in octal");
            metrics
                .seed_fields
                .push(("template-mode".into(), r.rect, r.id));
            changed |= r.changed();
        });
        ui.label("File contents");
        let height = ui.text_style_height(&egui::TextStyle::Monospace) * 6. + 16.;
        let area = controls::nested_scroll(
            ui,
            egui::ScrollArea::both()
                .id_salt(("account-template-scroll", key, &path))
                .max_height(height)
                .min_scrolled_height(height)
                .auto_shrink([false, false]),
            |ui| {
                let r = ui.add(
                    egui::TextEdit::multiline(&mut draft.contents)
                        .id_salt(("account-template-body", key, &path))
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(6)
                        .char_limit(dlms_learning_lab::accounts_template::MAX_TEXT_BYTES),
                );
                name_control(&r, "Starter file contents");
                metrics
                    .seed_fields
                    .push(("template-body".into(), r.rect, r.id));
                changed |= r.changed();
            },
        );
        metrics.editor_viewport = Some(area.inner_rect);
        workspace::bound(
            &mut draft.contents,
            dlms_learning_lab::accounts_template::MAX_TEXT_BYTES,
        );
        if draft.mode.is_empty()
            || !draft.mode.bytes().all(|b| (b'0'..=b'7').contains(&b))
            || u16::from_str_radix(&draft.mode, 8).is_ok_and(|m| m > 0o777)
        {
            ui.colored_label(
                palette(ui).error,
                "Use octal digits 0–7, from 0000 to 0777.",
            );
        }
        if draft.contents.contains('\0') {
            ui.colored_label(palette(ui).error, "Remove the NUL character before saving.");
        }
    });
    let included = draft.included;
    if changed {
        ui.ctx()
            .data_mut(|d| d.remove::<Result<(), String>>(status_id));
    }
    let r = ui.add(button(if included {
        "Save selected file"
    } else {
        "Remove selected file"
    }));
    metrics.editor_save = Some(r.rect);
    if r.clicked() {
        let result = drafts.save_selected(source, current);
        ui.ctx().data_mut(|d| d.insert_temp(status_id, result));
        changed = true;
    }
    if let Some(result) = ui
        .ctx()
        .data(|d| d.get_temp::<Result<(), String>>(status_id))
    {
        match result {
            Ok(()) => {
                ui.label("Saved to the simulated template. Existing homes are unchanged.");
            }
            Err(error) => {
                ui.colored_label(palette(ui).error, error);
            }
        }
    }
    if !drafts.matches_state(source, current) {
        ui.label("Unsaved changes · account creation uses the last saved template.");
    }
    changed
}

impl LabApp {
    pub(super) fn accounts_assignment_templates(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = &self.scenarios[i];
        let engine::State::Accounts { simulation: source } = &s.initial else {
            return;
        };
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        let engine::State::Accounts { simulation } = &mut a.state else {
            return;
        };
        let w = a.workspace.as_mut().unwrap();
        let Some(drafts) = &mut w.accounts_template_drafts else {
            return;
        };
        let before_selection = drafts.selected.clone();
        let changed = editor(
            ui,
            (&s.id, "assignment"),
            source,
            simulation,
            drafts,
            &mut self.metrics,
        );
        self.dirty |= changed || before_selection != drafts.selected;
        if changed {
            self.checked = false;
            w.checked = false;
            self.metrics.completion = false;
            self.completion_index.forget(&s.id);
        }
        if self.help_open {
            controls::collapsing_header(ui, "Show worked template changes")
                .id_salt(("account-template-worked-assignment", &s.id))
                .show(ui, |ui| {
                    let native = s
                        .accounts_teaching
                        .as_ref()
                        .unwrap()
                        .template_teaching
                        .as_ref()
                        .unwrap();
                    worked(ui, &native.assignment);
                });
        }
    }
}
