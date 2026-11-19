//! Opt-in mount editor beside the existing real simulated terminal. No host I/O.
use super::*;
use model::mount_flow as flow;
pub(super) fn worked(ui: &mut egui::Ui, op: &flow::Operation, explanation: &str) {
    match op {
        flow::Operation::Command { text } => lesson_presentation::command(ui, text, explanation),
        flow::Operation::SavePolicy { text } => {
            ui.strong("Save simulated mount configuration");
            lesson_presentation::monospace(ui, text, true);
            lesson_presentation::prose(ui, explanation);
        }
    }
}
pub(super) fn tools(
    ui: &mut egui::Ui,
    d: &model::Definition,
    s: &mut model::Session,
    practice: bool,
    flags: (bool, bool),
    identity: (&str, u32, u64, u64),
    metrics: &mut Metrics,
) -> device_reference::ResultState {
    let mut out = device_reference::ResultState::default();
    let (enabled, packet) = flags;
    let Some(c) = d.mount_context(practice) else {
        return out;
    };
    let live = if practice {
        &s.mount_runtime.as_ref().unwrap().example
    } else {
        &s.mount_runtime.as_ref().unwrap().assignment
    };
    ui.strong("Simulated /etc/fstab");
    lesson_presentation::prose(
        ui,
        "Edit the saved mount policy here. Save configuration changes future mount requests; it does not replace a filesystem already mounted. Use the terminal below to inspect and apply the change.",
    );
    let id = egui::Id::new(("lvm-mount-editor", identity, practice));
    let previous = egui::text_edit::TextEditState::load(ui.ctx(), id).map(|state| {
        let undo = state.undoer();
        NativeBefore { state, undo }
    });
    let edit = shared_libraries_page::configuration_input(
        ui,
        &live.draft,
        id,
        4096,
        enabled,
        "source  mountpoint  type  options  dump  pass",
    );
    out.received = packet && (edit.response.has_focus() || edit.value.is_some() || edit.rejected);
    metrics
        .fields
        .push(("mount-editor".into(), edit.response.rect, id, edit.clip));
    out.rejected = edit.rejected;
    if let Some(text) = edit.value {
        match s.edit_mount(d, text, practice) {
            Ok(()) => out.changed = true,
            Err(e) => {
                out.rejected = true;
                out.error = Some(e);
                if let Some(p) = previous {
                    p.restore(ui.ctx(), id);
                }
            }
        }
    }
    if edit.rejected {
        out.error=Some("Use at most 4096 bytes/64 lines of mount configuration. The previous draft, saved policy and current mounts are retained; nothing ran.".into());
    }
    let live = if practice {
        &s.mount_runtime.as_ref().unwrap().example
    } else {
        &s.mount_runtime.as_ref().unwrap().assignment
    };
    if live.draft != live.mounted.storage.configuration() {
        ui.label("Unsaved configuration · current mounts and saved policy are unchanged");
    }
    let save = ui.add_enabled(
        enabled && !packet && !out.rejected,
        button("Save configuration"),
    );
    metrics
        .controls
        .push(("mount-save".into(), save.rect, ui.clip_rect()));
    if save.clicked() {
        match s.save_mount(d, practice) {
            Ok(_) => out.changed = true,
            Err(e) => out.error = Some(e),
        }
    }
    let live = if practice {
        &s.mount_runtime.as_ref().unwrap().example
    } else {
        &s.mount_runtime.as_ref().unwrap().assignment
    };
    let actual = live.mounted.storage.live();
    let current = if practice { &s.example } else { &s.current };
    ui.strong("Current mounts · actual simulated state");
    let rows = c
        .inventory
        .mountpoints
        .iter()
        .map(|p| {
            let text = match actual.get(&p.0) {
                None => "Not mounted".to_owned(),
                Some(m) => format!(
                    "{} · ext4 · {}",
                    live.mounted
                        .storage
                        .inventory()
                        .device_for(&m.volume)
                        .unwrap_or("unavailable device"),
                    if m.read_only {
                        "read-only"
                    } else {
                        "read/write"
                    }
                ),
            };
            (p.0.clone(), text)
        })
        .collect::<Vec<_>>();
    if ui.available_width() < ui.text_style_height(&egui::TextStyle::Body) * 38. {
        for (p, text) in rows {
            ui.monospace(p);
            ui.label(text);
        }
    } else {
        egui::Grid::new(("lvm-mount-actual", identity, practice))
            .num_columns(2)
            .spacing([16., 8.])
            .show(ui, |ui| {
                for (p, text) in rows {
                    ui.monospace(p);
                    ui.label(text);
                    ui.end_row();
                }
            });
    }
    #[cfg(test)]
    for t in &c.targets {
        let p = format!("{}{}", t.target, t.file);
        let r = live.mounted.read_path(&c.inventory, current, &p);
        metrics
            .inspected_files
            .push((p, r.unwrap_or("Reader unavailable").into()));
    }
    #[cfg(not(test))]
    let _ = current;
    controls::collapsing_header(ui,"Mount reference · format and supported commands").show(ui,|ui|{
        lesson_presentation::prose(ui,"Each entry has six space-separated fields: filesystem source, mountpoint, filesystem type, options, dump field, and check order. This exercise supplies ordinary ext4 filesystems and fixed mountpoints. Use blkid to inspect their device names and filesystem UUIDs.");
        ui.monospace("source  mountpoint  ext4  defaults  0  2");
        lesson_presentation::prose(ui,"Supported: blkid; findmnt [supplied mountpoint]; mount supplied-mountpoint; umount supplied-mountpoint; cat supplied-mountpoint/file; pvs, vgs, lvs and the supplied vgchange/vgrename/lvrename forms. Configuration accepts canonical /dev/VG/LV or UUID= sources and defaults, rw/ro, auto/noauto options. Other syntax is reported as unsupported.");
        lesson_presentation::prose(ui,"Simulator limits: one current mount per filesystem; unmount before changing a live mount; supplied nonbusy filesystems only. Real Linux also supports multiple and overlaid mounts, remounts, busy-file handling and many other options. No real mount, host file, device or service is accessed.");
    });
    out
}
pub(super) fn history(input: &str) -> String {
    if input == "Save simulated mount configuration" {
        input.into()
    } else {
        format!("virtual $ {input}")
    }
}

/// Keep complete grading; show only a short initial failure list by default.
pub(super) fn feedback(ui: &mut egui::Ui, rows: &[(bool, String)], metrics: &mut Metrics) {
    for (_, text) in rows.iter().filter(|r| !r.0).take(3) {
        let r = ui.colored_label(palette(ui).error, format!("Needs attention · {text}"));
        metrics.rows.push((false, r.rect));
        #[cfg(test)]
        metrics.feedback_texts.push(text.clone());
    }
    controls::collapsing_header(ui, "All objective results").show(ui, |ui| {
        for (ok, text) in rows {
            let r = ui.colored_label(
                if *ok {
                    palette(ui).success
                } else {
                    palette(ui).error
                },
                format!(
                    "{} · {text}",
                    if *ok { "Passed" } else { "Needs attention" }
                ),
            );
            metrics.rows.push((*ok, r.rect));
        }
    });
}
