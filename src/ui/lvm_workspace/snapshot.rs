//! Opt-in record/application controls; no effect on prior LVM workspaces.
use super::*;
use dlms_learning_lab::lvm_workspace::snapshot_flow::Operation;
#[derive(Default)]
pub(super) struct ResultState {
    pub changed: bool,
    pub error: Option<String>,
}
pub(super) fn tools(
    ui: &mut egui::Ui,
    d: &model::Definition,
    s: &mut model::Session,
    practice: bool,
    enabled: bool,
    metrics: &mut Metrics,
) -> ResultState {
    let mut result = ResultState::default();
    let Some(context) = d.snapshot_context(practice) else {
        return result;
    };
    ui.strong("Application records and saved views");
    lesson_presentation::prose(
        ui,
        "These controls simulate a small application's fixed-size records. They are not Linux commands or a mounted filesystem. LVM commands reserve and manage the COW space below.",
    );
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        for (index, update) in context.updates.iter().enumerate() {
            let r = ui.add_enabled(
                enabled,
                button(&format!("Apply simulated update · {}", update.label)),
            );
            metrics
                .controls
                .push((format!("snapshot-update-{index}"), r.rect, ui.clip_rect()));
            if r.clicked() {
                action = Some(Operation::ApplyUpdate { index });
            }
            r.on_hover_text(&update.explanation);
        }
    });
    let storage = if practice { &s.example } else { &s.current };
    if let Some(profile) = &storage.snapshots {
        for (origin, bank) in &profile.origins {
            ui.strong(origin);
            let keys = bank.layout().keys().cloned().collect::<Vec<_>>();
            let id = egui::Id::new(("snapshot-record", origin, practice));
            let mut selected = ui
                .ctx()
                .data(|data| data.get_temp::<String>(id))
                .filter(|v| keys.contains(v))
                .unwrap_or_else(|| keys[0].clone());
            ui.horizontal_wrapped(|ui| {
                ui.label("Record");
                let r =
                    controls::dropdown(ui, id, ui.text_style_height(&egui::TextStyle::Body) * 11.)
                        .selected_text(&selected)
                        .show_ui(ui, |ui| {
                            controls::fit_popup_contents(ui);
                            for key in &keys {
                                let r = ui.selectable_value(&mut selected, key.clone(), key);
                                metrics.controls.push((
                                    format!("snapshot-record-choice-{origin}-{key}"),
                                    r.rect,
                                    ui.clip_rect(),
                                ));
                            }
                        });
                metrics.controls.push((
                    format!("snapshot-record-{origin}"),
                    r.response.rect,
                    ui.clip_rect(),
                ));
            });
            ui.ctx()
                .data_mut(|data| data.insert_temp(id, selected.clone()));
            // Records are intentionally selectable text, never an invented file command.
            record(
                ui,
                metrics,
                &format!("{origin}:current:{selected}"),
                "Current application value",
                bank.current(&selected)
                    .map(str::to_owned)
                    .map_err(|e| e.to_string()),
            );
            for name in bank.snapshot_names() {
                let status = bank.status(name).unwrap();
                ui.label(format!(
                    "{name} · {} · COW {} reserved · {} of {} data chunks copied · {} per chunk",
                    if status.valid {
                        "Readable saved view"
                    } else {
                        "Invalid · old data unavailable"
                    },
                    capacity(status.capacity_bytes),
                    status.copied_chunks,
                    status.usable_chunks,
                    capacity(u64::from(status.chunk_bytes))
                ));
                let value = bank.read(name, &selected).map_err(|e| e.to_string());
                let can_copy = value.is_ok();
                record(
                    ui,
                    metrics,
                    &format!("{origin}:{name}:{selected}"),
                    "Saved value",
                    value,
                );
                let r = ui.add_enabled(
                    enabled && can_copy,
                    button(if can_copy {
                        "Copy saved record to current · simulator"
                    } else {
                        "Copy unavailable · invalid saved view"
                    }),
                );
                metrics.controls.push((
                    format!("snapshot-copy-{origin}-{name}"),
                    r.rect,
                    ui.clip_rect(),
                ));
                if r.clicked() {
                    action = Some(Operation::CopyRecord {
                        origin: origin.clone(),
                        snapshot: name.into(),
                        record: selected.clone(),
                    });
                }
            }
            if bank.snapshot_names().next().is_none() {
                ui.label("No saved view yet. Reserve one with the simulated LVM terminal.");
            }
        }
    }
    if let Some(op) = action {
        match s.apply_snapshot(d, &op, practice) {
            Ok(_) => result.changed = true,
            Err(e) => result.error = Some(e),
        }
    }
    result
}
fn record(
    ui: &mut egui::Ui,
    metrics: &mut Metrics,
    key: &str,
    label: &str,
    value: Result<String, String>,
) {
    ui.label(label);
    let ok = value.is_ok();
    let text = match value {
        Ok(v) => v,
        Err(e) => format!("Unavailable · {e}"),
    };
    let r = ui.add(
        egui::Label::new(RichText::new(&text).monospace())
            .wrap()
            .selectable(true),
    );
    metrics.reader_records.push((key.into(), ok));
    metrics
        .reader_rows
        .push((key.into(), r.rect, ui.clip_rect()));
    #[cfg(test)]
    metrics.inspected_files.push((key.into(), text));
}
