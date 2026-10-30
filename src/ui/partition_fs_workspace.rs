//! Explicit native disk configuration. No terminal, host paths or disk I/O.
use super::*;
use dlms_learning_lab::{
    engine::State, partition_fs as core, partition_fs_workspace as work, script_teaching::View,
};
use work::binding;
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub choices: Vec<(String, egui::Rect, egui::Rect)>,
    pub rows: Vec<(bool, egui::Rect)>,
    pub table: Option<egui::Rect>,
    pub goal: Option<egui::Rect>,
    pub notice: Option<egui::Rect>,
    pub field_text: Vec<(String, egui::Rect, egui::Rect)>,
    pub help_bodies: Vec<(String, egui::Rect, egui::Rect)>,
    pub text_rects: Vec<(String, egui::Rect, egui::Rect)>,
    capture_text: bool,
    #[cfg(test)]
    pub painted_text: Vec<String>,
    #[cfg(test)]
    pub guidance_intent: bool,
    #[cfg(test)]
    pub pending_action: Option<&'static str>,
    #[cfg(test)]
    pub staged_draft_changed: bool,
    #[cfg(test)]
    pub received_edit_bytes: usize,
    #[cfg(test)]
    pub edit_vetoed: bool,
}
#[derive(Default)]
pub(super) struct Presentation {
    veto_root: Option<u64>,
    held_veto: bool,
    generation: [u64; 2],
}
impl Presentation {
    #[cfg(test)]
    pub(in crate::ui) fn held_for_test(&self) -> bool {
        self.held_veto
    }
    pub(super) fn reset_all(&mut self) {
        for n in &mut self.generation {
            *n = n.wrapping_add(1);
        }
        self.veto_root = None;
        self.held_veto = false;
    }
    fn reset_practice(&mut self) {
        self.generation[1] = self.generation[1].wrapping_add(1);
    }
}
struct Buffer<'a> {
    value: &'a mut String,
    max: usize,
    received: usize,
    rejected: bool,
}
impl egui::TextBuffer for Buffer<'_> {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Buffer<'static>>()
    }
    fn is_mutable(&self) -> bool {
        true
    }
    fn as_str(&self) -> &str {
        self.value
    }
    fn insert_text(&mut self, text: &str, at: usize) -> usize {
        self.received = self.received.saturating_add(text.len());
        if self.rejected
            || text
                .chars()
                .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
            || self
                .value
                .len()
                .checked_add(text.len())
                .is_none_or(|n| n > self.max)
        {
            self.rejected = true;
            return 0;
        }
        egui::TextBuffer::insert_text(self.value, text, at)
    }
    fn delete_char_range(&mut self, range: std::ops::Range<usize>) {
        if !self.rejected {
            egui::TextBuffer::delete_char_range(self.value, range);
        }
    }
}
struct NativeBefore {
    id: egui::Id,
    state: egui::text_edit::TextEditState,
    undo: egui::util::undoer::Undoer<(egui::text::CCursorRange, String)>,
}
impl NativeBefore {
    fn restore(mut self, ctx: &egui::Context) {
        self.state.set_undoer(self.undo);
        self.state.store(ctx, self.id);
    }
}
struct PendingDraft {
    disk: String,
    draft: work::Draft,
    practice: bool,
    changed: bool,
    rejected: bool,
    native: Vec<NativeBefore>,
}
#[derive(Default)]
struct Packet {
    text: bool,
    bytes: usize,
    invalid: bool,
}
fn packet(ui: &egui::Ui) -> Packet {
    ui.input(|i| {
        let mut p = Packet::default();
        let mut after_return = false;
        for e in &i.events {
            match e {
                egui::Event::Key {
                    key: egui::Key::Enter,
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.matches_logically(egui::Modifiers::NONE) => after_return = true,
                egui::Event::Text(s) | egui::Event::Paste(s) if !s.is_empty() => {
                    p.text = true;
                    p.bytes = p.bytes.saturating_add(s.len());
                    p.invalid |= after_return
                        || p.bytes > 36
                        || s.chars()
                            .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'));
                }
                _ => {}
            }
        }
        p
    })
}
#[derive(Clone)]
enum Action {
    View(View),
    Disk(String),
    Example(usize),
    Insert,
    ResetPractice,
    Create,
    Add,
    Format,
    Check,
    Hint,
}
fn control(ui: &mut egui::Ui, m: &mut Metrics, key: &str, label: &str, enabled: bool) -> bool {
    let r = ui.add_enabled(enabled, button(label));
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
    r.clicked()
}
fn guidance_intent(ui: &egui::Ui, r: &egui::Response) -> bool {
    r.clicked()
        || ui.input(|i| {
            (i.pointer.primary_pressed() || i.pointer.primary_released())
                && i.pointer
                    .interact_pos()
                    .is_some_and(|p| r.rect.contains(p) && ui.clip_rect().contains(p))
        })
}
fn line(ui: &mut egui::Ui, m: &mut Metrics, text: impl Into<String>) -> egui::Response {
    let text = text.into();
    #[cfg(test)]
    m.painted_text.push(text.clone());
    let r = ui.label(&text);
    if m.capture_text {
        m.text_rects.push((text, r.rect, ui.clip_rect()));
    }
    r
}
#[derive(Default)]
struct FieldEdits {
    native: Vec<NativeBefore>,
    received: usize,
    rejected: bool,
}
fn field(
    ui: &mut egui::Ui,
    m: &mut Metrics,
    spec: (&str, &str, usize, egui::Id),
    value: &mut String,
    active: bool,
    edits: &mut FieldEdits,
) {
    let (key, label, max, id) = spec;
    let state = egui::text_edit::TextEditState::load(ui.ctx(), id).unwrap_or_default();
    let undo = state.undoer();
    edits.native.push(NativeBefore { id, state, undo });
    ui.label(label);
    let mut b = Buffer {
        value,
        max,
        received: 0,
        rejected: false,
    };
    let out = ui
        .add_enabled_ui(active, |ui| {
            controls::singleline(ui, &mut b)
                .id(id)
                .desired_width(ui.available_width())
                .show(ui)
        })
        .inner;
    let r = out.response;
    m.field_text.push((
        key.into(),
        r.rect,
        egui::Rect::from_min_size(out.galley_pos, out.galley.size()),
    ));
    edits.received = edits.received.saturating_add(b.received);
    edits.rejected |= b.rejected;
    m.fields.push((key.into(), r.rect, id, ui.clip_rect()));
}
fn capacity(n: u64) -> String {
    if n.is_multiple_of(1024 * 1024) {
        format!("{} MiB", n / (1024 * 1024))
    } else {
        format!("{n} B")
    }
}
fn signature(p: &core::Partition) -> String {
    match &p.content {
        core::Content::Blank => "Unformatted".into(),
        core::Content::Foreign { description } => format!("Protected signature: {description}"),
        core::Content::Ext4 { label, .. } => format!("ext4 · label {label}"),
        core::Content::Swap { label, .. } => format!("swap · label {label}"),
    }
}
fn table(ui: &mut egui::Ui, m: &mut Metrics, path: &str, d: &core::Disk) {
    let r = ui.vertical(|ui| {
        line(ui, m, format!("Applied disk {path} · {} · {} sectors of {} B · alignment {} sectors", capacity(d.sectors * d.sector_bytes), d.sectors, d.sector_bytes, d.optimal_alignment));
        let mut protection=Vec::new();
        if !d.writable { protection.push("Read-only"); }
        if d.in_use { protection.push("In use"); }
        if d.occupied_without_table { protection.push("Existing data to preserve"); }
        if !protection.is_empty() { line(ui,m,format!("Protection · {}. Destructive replacement is unavailable.",protection.join(" · "))); }
        let Some(t) = &d.table else { line(ui,m,"No partition table. Creating GPT is permitted only on a supplied blank, writable, unused disk."); return; };
        line(ui,m,format!("GPT usable sectors: 34–{} inclusive. Type hint and actual content are separate.",d.sectors-34));
        if t.partitions.is_empty() { line(ui,m,"The GPT is empty; no partition or filesystem has been created."); }
        let font = egui::TextStyle::Body.resolve(ui.style());
        let text_color=palette(ui).text;
        let widths: Vec<f32> = ["Partition / name", "Start–end sectors", "Capacity", "Type hint", "Actual content"].iter().enumerate().map(|(i,h)| {
            let mut texts=vec![h.to_string()];
            for p in &t.partitions { texts.push(match i {0=>format!("{path}{} / {}",p.number,p.name),1=>format!("{}–{}",p.start,p.end),2=>capacity((p.end-p.start+1)*512),3=>work::type_hint_name(p.type_hint).into(),_=>signature(p)}); }
            texts.iter().map(|s|ui.fonts_mut(|f| f.layout_no_wrap(s.clone(),font.clone(),text_color).size().x)).fold(0.,f32::max)+12.
        }).collect();
        let wide = widths.iter().sum::<f32>() + 4.*ui.spacing().item_spacing.x <= ui.available_width();
        if wide {
            egui::Grid::new(("partition-applied",path)).num_columns(5).striped(true).show(ui, |ui| {
                for (i,h) in ["Partition / name", "Start–end sectors", "Capacity", "Type hint", "Actual content"].iter().enumerate() { controls::cell(ui,widths[i],|ui|{ui.strong(*h);}); } ui.end_row();
                for p in &t.partitions {
                    for (i,text) in [format!("{path}{} / {}",p.number,p.name),format!("{}–{}",p.start,p.end),capacity((p.end-p.start+1)*512),work::type_hint_name(p.type_hint).into(),signature(p)].into_iter().enumerate() { controls::cell(ui,widths[i],|ui|{line(ui,m,text);}); } ui.end_row();
                }
            });
        } else {
            for p in &t.partitions { ui.group(|ui| {
                line(ui,m,format!("{path}{} · {} · {}",p.number,p.name,capacity((p.end-p.start+1)*512)));
                line(ui,m,format!("Sectors {}–{} inclusive · hint {}",p.start,p.end,work::type_hint_name(p.type_hint)));
                line(ui,m,signature(p));
            }); }
        }
    });
    m.table = Some(r.response.rect);
}
impl LabApp {
    pub(super) fn partition_fs_page(&mut self, ui: &mut egui::Ui, i: usize) {
        // Borrowed admission before source copy or native state restoration.
        let source = &self.scenarios[i];
        if let Err(e) = binding::context(&self.progress.attempts[&source.id], source) {
            ui.colored_label(palette(ui).error, e);
            return;
        }
        self.metrics.partition_fs.capture_text = self.smoke;
        let source = source.clone();
        let engine::Goal::PartitionFs { definition: d } = &source.goal else {
            return;
        };
        let p = packet(ui);
        let root = ui.ctx().cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let inherited = self.partition_fs_presentation.held_veto;
        if !ui.input(|i| i.pointer.primary_down()) {
            self.partition_fs_presentation.held_veto = false;
        }
        let blocked = inherited || self.partition_fs_presentation.veto_root == Some(root);
        let phase = (
            self.selected,
            self.reset_pending,
            self.progress.attempts[&source.id].challenge,
        );
        let challenge = ui
            .add_enabled_ui(!p.text && !blocked, |ui| self.workspace_header(ui, i))
            .inner;
        if phase
            != (
                self.selected,
                self.reset_pending,
                self.progress.attempts[&source.id].challenge,
            )
        {
            return;
        }
        let a = &self.progress.attempts[&source.id];
        let State::PartitionFs { simulation: w } = &a.state else {
            return;
        };
        let view = if challenge { View::Assignment } else { w.view };
        let practice = view == View::Example;
        let active = ui.is_enabled()
            && !blocked
            && !p.invalid
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let actions = active && !p.text;
        let mut action = None;
        let mut guidance = false;
        let mut pending_draft = None;
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, label, key) in [
                    (View::Introduction, "Introduction", "introduction"),
                    (View::Example, "Separate practice · ungraded", "practice"),
                    (View::Assignment, "Assignment", "assignment"),
                ] {
                    if control(ui, &mut self.metrics.partition_fs, key, label, actions) && view != v
                    {
                        action = Some(Action::View(v));
                    }
                }
            });
        }
        line(
            ui,
            &mut self.metrics.partition_fs,
            "Virtual disks only · configuration changes never touch your computer.",
        );
        if view == View::Introduction {
            line(
                ui,
                &mut self.metrics.partition_fs,
                &d.lesson.prerequisite_note,
            );
            for c in &d.lesson.concepts {
                ui.strong(&c.title);
                line(ui, &mut self.metrics.partition_fs, &c.text);
            }
            line(
                ui,
                &mut self.metrics.partition_fs,
                "Protect the supplied data before adding anything. The model cannot delete, resize or overwrite. In practice, a valid unwanted allocation is repaired by Reset practice and manual replay; reset never affects the assignment.",
            );
        } else {
            let editor = if practice {
                &w.example_editor
            } else {
                &w.assignment_editor
            };
            let world = if practice { &w.example } else { &w.current };

            let checked = if practice {
                w.example_checked
            } else {
                w.checked
            };
            let goal = ui.label(if practice {
                &d.lesson.description
            } else {
                &source.objective
            });
            self.metrics.partition_fs.goal = Some(goal.rect);
            let plan = if practice { &d.lesson.plan } else { &d.plan };
            for target in &plan.targets {
                let content = match &target.content {
                    work::DesiredContent::Blank => "unformatted content".into(),
                    work::DesiredContent::Ext4 { label } => format!(
                        "actual ext4{}",
                        label
                            .as_ref()
                            .map(|s| format!(" labeled {s}"))
                            .unwrap_or_default()
                    ),
                    work::DesiredContent::Swap { label } => format!(
                        "actual swap{}",
                        label
                            .as_ref()
                            .map(|s| format!(" labeled {s}"))
                            .unwrap_or_default()
                    ),
                };
                line(
                    ui,
                    &mut self.metrics.partition_fs,
                    format!(
                        "{}: {} to {} inclusive on {}; {} hint; {}; {}. Keep all other media and supplied data unchanged.",
                        target.name,
                        capacity(target.minimum_bytes),
                        capacity(target.maximum_bytes),
                        target.eligible_disks.join(" or "),
                        work::type_hint_name(target.type_hint),
                        content,
                        if target.aligned_start {
                            "optimal start alignment required"
                        } else {
                            "any valid non-overlapping start"
                        }
                    ),
                );
            }
            if practice {
                line(
                    ui,
                    &mut self.metrics.partition_fs,
                    "Independent practice. Fill a worked step, then choose its explicit action. Filling never applies it. Reset practice restores only this example; replay the steps yourself.",
                );
                ui.add_enabled_ui(actions, |ui| {
                    let pick = controls::dropdown(
                        ui,
                        (&source.id, "partition-example", a.visited),
                        ui.available_width(),
                    )
                    .selected_text(format!("Worked step {}", w.selected_example + 1))
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        for (n, step) in d.lesson.worked.iter().enumerate() {
                            let r = ui.selectable_label(
                                w.selected_example == n,
                                format!("{} · {}", n + 1, step.operation.description()),
                            );
                            self.metrics.partition_fs.choices.push((
                                format!("example-{n}"),
                                r.rect,
                                ui.clip_rect(),
                            ));
                            if r.clicked() {
                                action = Some(Action::Example(n));
                            }
                        }
                    });
                    self.metrics.partition_fs.controls.push((
                        "example-picker".into(),
                        pick.response.rect,
                        ui.clip_rect(),
                    ));
                    if control(
                        ui,
                        &mut self.metrics.partition_fs,
                        "fill-example",
                        "Fill selected step · does not apply",
                        true,
                    ) {
                        action = Some(Action::Insert);
                    }
                    if control(
                        ui,
                        &mut self.metrics.partition_fs,
                        "reset-practice",
                        "Reset practice",
                        true,
                    ) {
                        action = Some(Action::ResetPractice);
                    }
                });
                line(
                    ui,
                    &mut self.metrics.partition_fs,
                    &d.lesson.worked[w.selected_example].explanation,
                );
                line(ui, &mut self.metrics.partition_fs, &d.lesson.try_change);
                line(
                    ui,
                    &mut self.metrics.partition_fs,
                    format!(
                        "Practice experiment: {}. Use its explicit native action; Reset practice then replay the worked steps to repair.",
                        d.lesson.experiment.description()
                    ),
                );
            }
            ui.add_enabled_ui(actions, |ui| {
                let picker = controls::dropdown(
                    ui,
                    (&source.id, "partition-disk", practice, a.visited),
                    ui.available_width(),
                )
                .selected_text(&editor.selected)
                .show_ui(ui, |ui| {
                    controls::fit_popup_contents(ui);
                    for path in world.disks.keys() {
                        let r = ui.selectable_label(path == &editor.selected, path);
                        self.metrics.partition_fs.choices.push((
                            path.clone(),
                            r.rect,
                            ui.clip_rect(),
                        ));
                        if r.clicked() {
                            action = Some(Action::Disk(path.clone()));
                        }
                    }
                });
                self.metrics.partition_fs.controls.push((
                    "disk".into(),
                    picker.response.rect,
                    ui.clip_rect(),
                ));
            });
            table(
                ui,
                &mut self.metrics.partition_fs,
                &editor.selected,
                &world.disks[&editor.selected],
            );
            let old_draft = &editor.drafts[&editor.selected];
            let mut draft = old_draft.clone();
            line(
                ui,
                &mut self.metrics.partition_fs,
                "Unapplied editor values · only the named action changes the disk. Check grades the applied table and signatures, including protected disks, not these draft fields.",
            );
            let mut edits = FieldEdits {
                rejected: p.invalid,
                ..Default::default()
            };
            let salt = egui::Id::new((
                "partition-fields",
                &source.id,
                source.version,
                a.visited,
                practice,
                &editor.selected,
                self.partition_fs_presentation.generation[usize::from(practice)],
            ));
            let m = &mut self.metrics.partition_fs;
            field(
                ui,
                m,
                ("name", "Partition name (36 bytes)", 36, salt.with("name")),
                &mut draft.name,
                active,
                &mut edits,
            );
            field(
                ui,
                m,
                ("start", "Start sector (inclusive)", 32, salt.with("start")),
                &mut draft.start,
                active,
                &mut edits,
            );
            field(
                ui,
                m,
                ("end", "End sector (inclusive)", 32, salt.with("end")),
                &mut draft.end,
                active,
                &mut edits,
            );
            ui.add_enabled_ui(actions, |ui| {
                let pick = controls::dropdown(ui, salt.with("hint"), ui.available_width())
                    .selected_text(format!(
                        "Partition type hint: {}",
                        work::type_hint_name(draft.type_hint)
                    ))
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        for value in [core::TypeHint::LinuxData, core::TypeHint::LinuxSwap] {
                            let r = ui.selectable_value(
                                &mut draft.type_hint,
                                value,
                                work::type_hint_name(value),
                            );
                            m.choices
                                .push((format!("hint-{value:?}"), r.rect, ui.clip_rect()));
                        }
                    });
                m.controls
                    .push(("hint".into(), pick.response.rect, ui.clip_rect()));
            });
            ui.horizontal_wrapped(|ui| {
                if control(ui, m, "create-gpt", "Create GPT on blank disk", actions) {
                    action = Some(Action::Create);
                }
                if control(ui, m, "add", "Add partition using draft", actions) {
                    action = Some(Action::Add);
                }
            });
            ui.add_enabled_ui(actions, |ui| {
                let pick = controls::dropdown(ui, salt.with("partition"), ui.available_width())
                    .selected_text(
                        draft
                            .selected_partition
                            .map(|n| format!("Initialize {0}{n}", editor.selected))
                            .unwrap_or_else(|| "Choose a partition to initialize".into()),
                    )
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        if let Some(t) = &world.disks[&editor.selected].table {
                            for part in &t.partitions {
                                let r = ui.selectable_value(
                                    &mut draft.selected_partition,
                                    Some(part.number),
                                    format!("{}{} · {}", editor.selected, part.number, part.name),
                                );
                                m.choices.push((
                                    format!("partition-{}", part.number),
                                    r.rect,
                                    ui.clip_rect(),
                                ));
                            }
                        }
                    });
                m.controls
                    .push(("partition".into(), pick.response.rect, ui.clip_rect()));
                let pick = controls::dropdown(ui, salt.with("format"), ui.available_width())
                    .selected_text(format!(
                        "Content to initialize: {}",
                        work::format_name(draft.format)
                    ))
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        for value in [work::FormatChoice::Ext4, work::FormatChoice::Swap] {
                            let r = ui.selectable_value(
                                &mut draft.format,
                                value,
                                work::format_name(value),
                            );
                            m.choices
                                .push((format!("format-{value:?}"), r.rect, ui.clip_rect()));
                        }
                    });
                m.controls
                    .push(("content-type".into(), pick.response.rect, ui.clip_rect()));
            });
            field(
                ui,
                m,
                ("label", "Content label (16 bytes)", 16, salt.with("label")),
                &mut draft.label,
                active,
                &mut edits,
            );
            field(
                ui,
                m,
                (
                    "uuid",
                    "UUID · blank generates a unique virtual UUID",
                    36,
                    salt.with("uuid"),
                ),
                &mut draft.uuid,
                active,
                &mut edits,
            );
            ui.horizontal_wrapped(|ui| {
                if control(
                    ui,
                    m,
                    "format",
                    "Initialize selected blank partition",
                    actions,
                ) {
                    action = Some(Action::Format);
                }
                if control(ui, m, "check", "Check current disk configuration", actions) {
                    action = Some(Action::Check);
                }
            });
            self.metrics.check = m.controls.iter().find(|r| r.0 == "check").map(|r| r.1);
            if let Some(error) = &self.notice {
                m.notice = Some(
                    ui.colored_label(palette(ui).error, format!("Needs attention · {error}"))
                        .rect,
                );
            }
            let rows = if practice {
                d.practice_checks(w)
            } else {
                d.checks(w)
            };
            if checked {
                let failures = rows.iter().filter(|r| !r.0).count();
                if failures > 0 {
                    line(
                        ui,
                        m,
                        format!(
                            "{failures} of {} checks need attention. See the results below.",
                            rows.len()
                        ),
                    );
                } else {
                    line(ui, m, "All current configuration checks pass.");
                }
                for (ok, text) in rows {
                    let r = ui.colored_label(
                        if ok {
                            palette(ui).success
                        } else {
                            palette(ui).error
                        },
                        format!("{} · {text}", if ok { "Passed" } else { "Needs attention" }),
                    );
                    m.rows.push((ok, r.rect));
                }
            }
            let detail = ui
                .add_enabled_ui(actions, |ui| {
                    controls::collapsing_header(ui, "Inspect protected files and action history")
                        .id_salt(salt.with("detail"))
                        .show(ui, |ui| {
                            let h = ui.text_style_height(&egui::TextStyle::Body) * 6.;
                            egui::ScrollArea::vertical()
                                .id_salt(salt.with("files"))
                                .min_scrolled_height(h)
                                .max_height(h)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    for (path, disk) in &world.disks {
                                        if let Some(t) = &disk.table {
                                            for part in &t.partitions {
                                                if let core::Content::Ext4 { files, .. } =
                                                    &part.content
                                                {
                                                    for (name, file) in files {
                                                        line(
                                                            ui,
                                                            m,
                                                            format!(
                                                                "{path}{} · {name}: {}",
                                                                part.number, file.text
                                                            ),
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    let rows = if practice {
                                        &w.example_transcript
                                    } else {
                                        &w.transcript
                                    };
                                    for row in rows {
                                        line(ui, m, format!("{}\n{}", row.input, row.output));
                                    }
                                });
                        })
                })
                .inner;
            if guidance_intent(ui, &detail.header_response) {
                guidance = true;
            }
            m.controls.push((
                "inspect".into(),
                detail.header_response.rect,
                ui.clip_rect(),
            ));
            // Keep only the bounded edited draft and its existing native receipt.
            // Publication waits until every later guidance response has been read.
            edits.rejected |= p.text && edits.received != p.bytes;
            let changed = draft != *old_draft;
            #[cfg(test)]
            {
                m.staged_draft_changed = changed;
                m.received_edit_bytes = edits.received;
            }
            let rejected = edits.rejected || draft.validate().is_err();
            if p.text || changed || rejected {
                pending_draft = Some(PendingDraft {
                    disk: editor.selected.clone(),
                    draft,
                    practice,
                    changed,
                    rejected,
                    native: edits.native,
                });
            }
        }
        // Optional assistance is separate from essential introduction and from
        // applied configuration. One hint never opens the worked answer.
        let a = &self.progress.attempts[&source.id];
        let scope = egui::Id::new((
            "partition-guidance",
            &source.id,
            source.version,
            a.visited,
            challenge,
            practice,
        ));
        let help=ui.add_enabled_ui(actions,|ui|controls::collapsing_header(ui,"Optional help and references").id_salt(scope).show(ui,|ui|{
            let hint = ui.add_enabled(a.hints < source.hints.len(), button("Show the next hint"));
            self.metrics.partition_fs.controls.push(("show-hint".into(), hint.rect, ui.clip_rect()));
            if hint.clicked() { action = Some(Action::Hint); }
            guidance |= guidance_intent(ui, &hint);
            for text in source.hints.iter().take(a.hints) {let r=line(ui,&mut self.metrics.partition_fs,text);self.metrics.partition_fs.help_bodies.push(("hint".into(),r.rect,ui.clip_rect()));}
            let worked=controls::collapsing_header(ui,"Show the worked approach · contains assignment answers").id_salt(scope.with("worked")).show(ui,|ui|{
                for operation in &d.reference {line(ui,&mut self.metrics.partition_fs,operation.description());}
                for text in &source.solution {line(ui,&mut self.metrics.partition_fs,text);}
                line(ui,&mut self.metrics.partition_fs,&source.explanation);
            });guidance |= guidance_intent(ui, &worked.header_response);self.metrics.partition_fs.controls.push(("worked".into(),worked.header_response.rect,ui.clip_rect()));
            let refs=controls::collapsing_header(ui,"Tool names, primary references and simulation limits").id_salt(scope.with("refs")).show(ui,|ui|{
                line(ui,&mut self.metrics.partition_fs,work::HELP);
                line(ui,&mut self.metrics.partition_fs,"Native configuration actions teach modeled outcomes. They do not establish coverage of parted, mkfs.ext4 or mkswap command syntax. The pure subset has no deletion, resize, MBR/logical partitions, mount or swap activation.");
                for text in &source.sources {ui.hyperlink(text);}
                line(ui,&mut self.metrics.partition_fs,&source.simplifications);
            });guidance |= guidance_intent(ui, &refs.header_response);self.metrics.partition_fs.controls.push(("references".into(),refs.header_response.rect,ui.clip_rect()));
        })).inner;
        if guidance_intent(ui, &help.header_response) {
            guidance = true;
        }
        self.metrics.partition_fs.controls.push((
            "help".into(),
            help.header_response.rect,
            ui.clip_rect(),
        ));
        #[cfg(test)]
        {
            self.metrics.partition_fs.guidance_intent = guidance;
            self.metrics.partition_fs.pending_action = action.as_ref().map(|a| match a {
                Action::Create => "create-gpt",
                Action::Add => "add",
                Action::Format => "format",
                Action::Check => "check",
                Action::Hint => "show-hint",
                _ => "navigation",
            });
        }
        // Guidance always blocks disk/Check/Next in this frame. Latch across a
        // held release only for competing input; a plain hint/header press must
        // leave its own later release enabled so an ordinary split click works.
        let competing = p.text
            || p.invalid
            || blocked
            || pending_draft.as_ref().is_some_and(|edit| edit.changed)
            || action.as_ref().is_some_and(|a| !matches!(a, Action::Hint))
            || ui.input(|i| {
                i.key_pressed(egui::Key::Enter)
                    && i.modifiers.matches_logically(egui::Modifiers::NONE)
            });
        if guidance {
            if competing {
                self.partition_fs_presentation.veto_root = Some(root);
                self.partition_fs_presentation.held_veto |= ui.input(|i| i.pointer.primary_down());
            }
            ui.ctx().request_repaint();
        }
        // Backspace/Cut/Undo/Redo can edit without a Text/Paste event. Any
        // actual draft edit is separate from releasing an old held action.
        if p.text || pending_draft.as_ref().is_some_and(|edit| edit.changed) {
            self.partition_fs_presentation.held_veto |= ui.input(|i| i.pointer.primary_down());
        }
        if let Some(edit) = pending_draft {
            let veto = edit.rejected || guidance || blocked || !active;
            if veto {
                for before in edit.native {
                    before.restore(ui.ctx());
                }
                #[cfg(test)]
                {
                    self.metrics.partition_fs.edit_vetoed = true;
                }
                self.partition_fs_presentation.veto_root = Some(root);
                self.partition_fs_presentation.held_veto |= ui.input(|i| i.pointer.primary_down());
                self.notice = Some(if edit.rejected {
                    "That whole edit was refused. Use one bounded single-line field; the previous draft, selection and undo history remain. Nothing was applied."
                } else {
                    "Help and editing were requested together. The previous draft, selection and undo history remain; make the edit in a separate action. Nothing was applied."
                }.into());
                ui.ctx().request_repaint();
                return;
            }
            if edit.changed {
                if let Err(error) = binding::set_draft(
                    self.progress.attempts.get_mut(&source.id).unwrap(),
                    &source,
                    &edit.disk,
                    edit.draft,
                    edit.practice,
                ) {
                    for before in edit.native {
                        before.restore(ui.ctx());
                    }
                    self.notice = Some(error);
                    self.partition_fs_presentation.veto_root = Some(root);
                    self.partition_fs_presentation.held_veto |=
                        ui.input(|i| i.pointer.primary_down());
                    ui.ctx().request_repaint();
                    return;
                }
                self.dirty = true;
                self.notice = None;
            }
            // Even a text edit whose resulting draft is identical is separate
            // from operation/Check/Next. No per-frame full-session copy occurs.
            ui.ctx().request_repaint();
            return;
        }
        let can_dispatch =
            actions && !guidance && self.partition_fs_presentation.veto_root != Some(root);
        // A hint is the only allowed guidance-side mutation. Every other action
        // waits for a later clean root frame; no model/grade side effect occurs.
        if let Some(next) = action.clone()
            && (can_dispatch || matches!(next, Action::Hint) && guidance && actions)
        {
            let a = self.progress.attempts.get_mut(&source.id).unwrap();
            let result = match &next {
                Action::View(v) => binding::set_view(a, &source, *v),
                Action::Disk(path) => binding::select_disk(a, &source, path, practice),
                Action::Example(n) => binding::select_example(a, &source, *n),
                Action::Insert => {
                    let State::PartitionFs { simulation } = &a.state else {
                        unreachable!()
                    };
                    let n = simulation.selected_example;
                    binding::insert(a, &source, n)
                }
                Action::ResetPractice => binding::reset_example(a, &source),
                Action::Create | Action::Add | Action::Format => {
                    let State::PartitionFs { simulation } = &a.state else {
                        unreachable!()
                    };
                    let e = if practice {
                        &simulation.example_editor
                    } else {
                        &simulation.assignment_editor
                    };
                    let operation = match &next {
                        Action::Create => Ok(work::Operation::CreateGpt {
                            disk: e.selected.clone(),
                        }),
                        Action::Add => e.drafts[&e.selected].add_operation(&e.selected),
                        _ => e.drafts[&e.selected].format_operation(&e.selected),
                    };
                    operation.and_then(|o| binding::apply(a, &source, &o, practice))
                }
                Action::Check => binding::check(a, &source, practice),
                Action::Hint => binding::request_hint(a, &source),
            };
            match result {
                Ok(()) => {
                    self.dirty = true;
                    self.notice = None;
                    if matches!(next, Action::ResetPractice) {
                        self.partition_fs_presentation.reset_practice();
                    }
                }
                Err(e) => self.notice = Some(e),
            }
            ui.ctx().request_repaint();
            return;
        }
        self.metrics.completion =
            progress::current_solved(&self.progress.attempts[&source.id], &source);
        if view == View::Assignment
            && self.metrics.completion
            && self.selected == Some(i)
            && actions
            && !guidance
            && self.partition_fs_presentation.veto_root != Some(root)
            && action.is_none()
        {
            self.next_unit_lab(ui, i);
        }
    }
}
