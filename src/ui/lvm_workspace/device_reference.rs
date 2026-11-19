//! Actual application-reference drafts/readers; opt-in, no host device access.
use super::*;
#[derive(Default)]
pub(super) struct ResultState {
    pub changed: bool,
    pub received: bool,
    pub rejected: bool,
    pub error: Option<String>,
}
pub(super) fn tools(
    ui: &mut egui::Ui,
    d: &model::Definition,
    s: &mut model::Session,
    practice: bool,
    flags: (bool, bool),
    identity: (&str, u32, u64, u64),
    metrics: &mut Metrics,
) -> ResultState {
    let (enabled, packet) = flags;
    let mut result = ResultState::default();
    if d.reference_context(practice).is_none() {
        return result;
    }
    let live = if practice {
        &s.reference_runtime.as_ref().unwrap().example
    } else {
        &s.reference_runtime.as_ref().unwrap().assignment
    };
    let fields = live
        .configuration
        .callers
        .iter()
        .map(|(key, c)| (key.clone(), c.clone(), live.drafts[key].0.clone()))
        .collect::<Vec<_>>();
    ui.strong("Application device references");
    lesson_presentation::prose(
        ui,
        "Edit the application's simulated configuration, then Save. The reader uses the saved reference; typing alone does not change it. These fields never access this computer's devices.",
    );
    for (key, caller, mut draft) in fields {
        ui.strong(&caller.label);
        ui.label(format!("Virtual file: {}", caller.file));
        let id = egui::Id::new(("lvm-reference", identity, practice, &key));
        let old_native = ui
            .input(|i| !i.events.is_empty())
            .then(|| {
                egui::text_edit::TextEditState::load(ui.ctx(), id).map(|state| {
                    let undo = state.undoer();
                    NativeBefore { state, undo }
                })
            })
            .flatten();
        let invalid=ui.memory(|m|m.has_focus(id)) && ui.input(|i|i.events.iter().any(|e|matches!(e,egui::Event::Text(t)|egui::Event::Paste(t) if t.len()>256 || t.chars().any(char::is_control))));
        let mut buffer = NativeEditBuffer {
            value: &mut draft,
            rejected: invalid,
            received: false,
            limit: 256,
        };
        // Width reserves Save's actual font-derived size. Narrow layouts stack.
        let save_width = controls::button_width(ui, "Save");
        let wide = ui.available_width() > ui.text_style_height(&egui::TextStyle::Body) * 34.;
        let mut edit = None;
        let mut save = None;
        let mut row = |ui: &mut egui::Ui| {
            ui.label("Device reference");
            let width = if wide {
                (ui.available_width() - save_width - ui.spacing().item_spacing.x).max(1.)
            } else {
                ui.available_width()
            };
            let r = ui.add_enabled(
                enabled,
                controls::singleline_fitting(ui, &mut buffer, width)
                    .id(id)
                    .hint_text("Absolute virtual device path"),
            );
            edit = Some(r);
            let r = ui.add_enabled(enabled && !packet && !result.rejected, button("Save"));
            save = Some(r);
        };
        if wide {
            ui.horizontal(|ui| {
                row(ui);
            });
        } else {
            row(ui);
        }
        let edit = edit.unwrap();
        let save = save.unwrap();
        let received = buffer.received;
        let mut rejected = buffer.rejected;
        result.received |= received;
        metrics
            .fields
            .push((format!("reference-{key}"), edit.rect, id, ui.clip_rect()));
        metrics
            .controls
            .push((format!("reference-save-{key}"), save.rect, ui.clip_rect()));
        if edit.changed() && !rejected {
            match s.edit_reference(d, &key, draft, practice) {
                Ok(()) => result.changed = true,
                Err(e) => {
                    rejected = true;
                    result.error = Some(e);
                }
            }
        }
        if rejected {
            if let Some(old) = old_native {
                old.restore(ui.ctx(), id);
            }
            result.rejected = true;
            result.error=Some("Use a single device reference up to 256 bytes without control characters. The previous draft, saved reference and selection are retained; nothing ran.".into());
        }
        if save.clicked() && !rejected && !packet {
            match s.save_reference(d, &key, practice) {
                Ok(_) => result.changed = true,
                Err(e) => result.error = Some(e),
            }
        }
        let live = if practice {
            &s.reference_runtime.as_ref().unwrap().example
        } else {
            &s.reference_runtime.as_ref().unwrap().assignment
        };
        let saved = &live.configuration.callers[&key].path;
        if live.drafts[&key].0 != *saved {
            ui.label("Unsaved edit · reader still uses the saved reference");
            ui.label("Saved device reference");
            ui.monospace(if saved.is_empty() { "(blank)" } else { saved });
        } else {
            ui.label("Saved reference matches the field above");
        }
        let storage = if practice { &s.example } else { &s.current };
        let read = live.configuration.inspect(storage, &key);
        let ok = read.is_ok();
        let text = match read {
            Ok(r) => format!("Actual reader result · readable\n{}", r.content),
            Err(e) => format!("Actual reader result · unavailable\n{e}"),
        };
        let r = ui.add(
            egui::Label::new(RichText::new(&text).monospace())
                .wrap()
                .selectable(true),
        );
        metrics
            .reader_records
            .push((format!("reference-{key}"), ok));
        metrics
            .reader_rows
            .push((format!("reference-reader-{key}"), r.rect, ui.clip_rect()));
        #[cfg(test)]
        metrics
            .inspected_files
            .push((format!("reference-{key}"), text));
        ui.add_space(ui.spacing().item_spacing.y);
    }
    result
}
/// Keep native configuration actions distinct from supported Linux commands.
/// The stored bounded packet remains available to persistence, never to a shell.
pub(super) fn action_label(
    d: &model::Definition,
    input: &str,
    practice: bool,
    command_prompt: bool,
) -> Option<String> {
    let context = d.reference_context(practice)?;
    Some(match model::device_reference_session::decode(input) {
        Ok(op) => {
            let label = op.description(context);
            if command_prompt
                && matches!(op, model::device_reference_flow::Operation::Command { .. })
            {
                format!("virtual $ {label}")
            } else {
                label
            }
        }
        Err(_) => "Simulator configuration action · saved description unavailable".into(),
    })
}
