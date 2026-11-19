//! Native policy editors over supplied events; no IOS terminal or host logging.
use super::*;
use dlms_learning_lab::{
    engine::State, script_teaching::View, syslog as core, syslog_workspace as model,
};
use std::net::Ipv4Addr;
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub choices: Vec<(String, egui::Rect, egui::Rect)>,
    pub rows: Vec<(bool, egui::Rect)>,
    #[cfg(test)]
    pub painted_rows: Vec<(usize, bool, egui::Rect, egui::Rect)>,
    pub texts: Vec<String>,
    pub help_bodies: Vec<(String, egui::Rect, egui::Rect)>,
    pub goal: Option<egui::Rect>,
    pub history: Option<(egui::Rect, f32)>,
    pub topology: Option<egui::Rect>,
    #[cfg(test)]
    pub entered: bool,
    #[cfg(test)]
    pub intents: (bool, bool, bool),
    #[cfg(test)]
    pub guidance: bool,
    #[cfg(test)]
    pub disclosures: Vec<&'static str>,
}
#[derive(Default)]
pub(super) struct Presentation {
    owner: Option<(String, u32, u64, bool)>,
    pub generation: [u64; 2],
    pub veto_root: Option<u64>,
    pub held_veto: bool,
    pub notice: Option<(bool, String)>,
}
impl Presentation {
    pub(super) fn reset_all(&mut self) {
        for g in &mut self.generation {
            *g = g.wrapping_add(1);
        }
        self.veto_root = None;
        self.held_veto = false;
        self.notice = None;
        self.owner = None;
    }
    fn reset_example(&mut self) {
        self.generation[1] = self.generation[1].wrapping_add(1);
        self.notice = None;
    }
}
fn metric(m: &mut Metrics, key: &str, r: &egui::Response, clip: egui::Rect) {
    m.controls.push((key.into(), r.rect, clip));
}
// Observe actual press/release intent as well as completed clicks. Enter in a
// focused field/button cannot dispatch while the learner is opening help.
fn guidance_intent(ui: &egui::Ui, r: &egui::Response) -> bool {
    r.clicked()
        || ui.input(|i| {
            (i.pointer.primary_pressed() || i.pointer.primary_released())
                && i.pointer
                    .interact_pos()
                    .is_some_and(|p| r.rect.contains(p) && ui.clip_rect().contains(p))
        })
}
// Render previous/current checked policy feedback before action dispatch, so
// opening the complete-check disclosure participates in the same guidance veto.
// Apply/Check actions below request repaint and reveal their new feedback next frame.
fn current_results(
    ui: &mut egui::Ui,
    d: &model::Definition,
    s: &model::Session,
    practice: bool,
    id: egui::Id,
    m: &mut Metrics,
) -> bool {
    if !(if practice {
        s.example_checked
    } else {
        s.checked
    }) {
        return false;
    }
    let rows = if practice {
        d.practice_checks(s)
    } else {
        d.checks(s)
    };
    let passed = rows.iter().filter(|r| r.0).count();
    let summary = format!(
        "{} · {} of {} checks pass",
        if passed == rows.len() {
            "Configuration passes"
        } else {
            "Configuration needs attention"
        },
        passed,
        rows.len()
    );
    let r = ui.strong(&summary);
    metric(m, "result-summary", &r, ui.clip_rect());
    m.texts.push(summary);
    let details = controls::collapsing_header(ui, "All configuration checks · optional details")
        .id_salt(id)
        .default_open(false)
        .show(ui, |ui| {
            for (index, (ok, text)) in rows.iter().enumerate() {
                let r = ui.label(format!(
                    "{} · {text}",
                    if *ok { "Passed" } else { "Needs attention" }
                ));
                // Only real, visible label responses are painted-row metrics.
                // Hidden/cull-clipped checks stay model truth, never synthetic rects.
                if ui.is_rect_visible(r.rect) {
                    metric(m, &format!("check-row-{index}"), &r, ui.clip_rect());
                    m.rows.push((*ok, r.rect));
                    m.texts.push(text.clone());
                    #[cfg(test)]
                    m.painted_rows.push((index, *ok, r.rect, ui.clip_rect()));
                }
            }
        });
    metric(m, "check-details", &details.header_response, ui.clip_rect());
    #[cfg(test)]
    if details.body_returned.is_some() {
        m.disclosures.push("check-details");
    }
    // Do not repeat the same failing sentence above its row in expanded details.
    if details.body_returned.is_none()
        && let Some((_, why)) = rows.iter().find(|r| !r.0)
    {
        let r = lesson_presentation::rich(
            ui,
            RichText::new(format!("Needs attention · {why}")).color(palette(ui).error),
        );
        metric(m, "first-issue", &r, ui.clip_rect());
        m.texts.push(why.clone());
    }
    guidance_intent(ui, &details.header_response)
}
fn finish_choices(ui: &egui::Ui, metrics: &mut Metrics, start: usize) {
    let clip = controls::popup_content_clip(ui);
    for choice in &mut metrics.choices[start..] {
        choice.2 = clip;
    }
}
fn choice_facility(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash,
    value: &mut Option<core::Facility>,
    m: &mut Metrics,
    key: &str,
) {
    let r = controls::dropdown(ui, id, ui.available_width())
        .selected_text(value.map_or_else(|| "Choose facility".into(), model::facility_label))
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            let first = m.choices.len();
            for n in 0..24 {
                let f = core::Facility::new(n).unwrap();
                let r = ui.selectable_value(value, Some(f), model::facility_label(f));
                m.choices.push((
                    format!("{key}-{n}"),
                    r.rect,
                    controls::popup_content_clip(ui),
                ));
            }
            finish_choices(ui, m, first);
        })
        .response;
    name_control(
        &r,
        if key == "facility" {
            "Export facility"
        } else {
            "Collector selector facility"
        },
    );
    metric(m, key, &r, ui.clip_rect());
}
fn choice_severity(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash,
    value: &mut Option<core::Severity>,
    off: bool,
    m: &mut Metrics,
    key: &str,
) {
    let label = value.map_or_else(
        || "Off".into(),
        |s| format!("0 through {} · {}", s.number(), s.name()),
    );
    let r = controls::dropdown(ui, id, ui.available_width())
        .selected_text(label)
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            let first = m.choices.len();
            if off {
                let r = ui.selectable_value(value, None, "Off");
                m.choices.push((
                    format!("{key}-off"),
                    r.rect,
                    controls::popup_content_clip(ui),
                ));
            }
            for n in 0..8 {
                let s = core::Severity::from_number(n).unwrap();
                let r =
                    ui.selectable_value(value, Some(s), format!("0 through {} · {}", n, s.name()));
                m.choices.push((
                    format!("{key}-{n}"),
                    r.rect,
                    controls::popup_content_clip(ui),
                ));
            }
            finish_choices(ui, m, first);
        })
        .response;
    name_control(
        &r,
        match key {
            "buffer" => "Local buffer severity threshold",
            "remote" => "Remote export severity threshold",
            _ => "Collector severity threshold",
        },
    );
    metric(m, key, &r, ui.clip_rect());
}
struct AddressBuffer<'a> {
    value: &'a mut String,
    rejected: bool,
    received: bool,
}
impl egui::TextBuffer for AddressBuffer<'_> {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<AddressBuffer<'static>>()
    }
    fn is_mutable(&self) -> bool {
        true
    }
    fn as_str(&self) -> &str {
        self.value
    }
    fn insert_text(&mut self, text: &str, at: usize) -> usize {
        if text.is_empty() || self.rejected {
            return 0;
        }
        self.received = true;
        if text
            .chars()
            .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
            || self
                .value
                .len()
                .checked_add(text.len())
                .is_none_or(|n| n > 15)
        {
            self.rejected = true;
            return 0;
        }
        egui::TextBuffer::insert_text(self.value, text, at)
    }
    fn delete_char_range(&mut self, r: std::ops::Range<usize>) {
        if !self.rejected {
            egui::TextBuffer::delete_char_range(self.value, r);
        }
    }
}
struct NativeBefore {
    state: egui::text_edit::TextEditState,
    undo: egui::util::undoer::Undoer<(egui::text::CCursorRange, String)>,
}
impl NativeBefore {
    fn restore(mut self, ctx: &egui::Context, id: egui::Id) {
        self.state.set_undoer(self.undo);
        self.state.store(ctx, id);
    }
}
#[derive(Default)]
struct Edits {
    source: Option<(Ipv4Addr, model::SourceDraft)>,
    collector: Option<(Ipv4Addr, model::CollectorDraft)>,
    rejected: bool,
    received: bool,
    enter: bool,
    native: Vec<(egui::Id, NativeBefore)>,
}
fn source_editor(
    ui: &mut egui::Ui,
    ip: Ipv4Addr,
    mut draft: model::SourceDraft,
    scope: egui::Id,
    active: bool,
    invalid_packet: bool,
    m: &mut Metrics,
) -> Edits {
    let before = draft.clone();
    let mut edit = Edits::default();
    controls::form_row(ui, "Logging process", |ui, _| {
        let r = ui.checkbox(&mut draft.enabled, "Enabled");
        metric(m, "process", &r, ui.clip_rect());
    });
    controls::form_row(ui, "Local buffer accepts", |ui, _| {
        choice_severity(ui, "buffer", &mut draft.buffer, true, m, "buffer")
    });
    controls::form_row(ui, "Remote export accepts", |ui, _| {
        choice_severity(ui, "remote", &mut draft.remote, true, m, "remote")
    });
    controls::form_row(ui, "Export facility", |ui, _| {
        let mut f = Some(draft.facility);
        choice_facility(ui, "export-facility", &mut f, m, "facility");
        draft.facility = f.unwrap();
    });
    ui.label("Remote collector addresses · edits are not applied until Apply settings.");
    let mut remove = None;
    for (n, value) in draft.hosts.iter_mut().enumerate() {
        let font = egui::TextStyle::Body.resolve(ui.style());
        let color = palette(ui).text;
        let min_address = ui.fonts_mut(|f| {
            f.layout_no_wrap("255.255.255.255".into(), font, color)
                .size()
                .x
        }) + 2. * f32::from(controls::PAD_X);
        let narrow = ui.available_width()
            < controls::form_label_width(ui)
                + min_address
                + controls::button_width(ui, "Remove")
                + 2. * controls::GAP;
        let mut row = |ui: &mut egui::Ui, width: f32| {
            let id = scope.with(("host", n));
            if let Some(state) = egui::text_edit::TextEditState::load(ui.ctx(), id) {
                let undo = state.undoer();
                edit.native.push((id, NativeBefore { state, undo }));
            }
            let mut buf = AddressBuffer {
                value: &mut *value,
                rejected: invalid_packet,
                received: false,
            };
            let r = ui.add_enabled(
                active,
                controls::singleline(ui, &mut buf)
                    .id(id)
                    .hint_text("192.0.2.100")
                    .desired_width(
                        (width
                            - controls::button_width(ui, "Remove")
                            - ui.spacing().item_spacing.x
                            - 2. * f32::from(controls::PAD_X))
                        .max(1.),
                    ),
            );
            edit.rejected |= buf.rejected;
            edit.received |= buf.received;
            edit.enter |= (r.has_focus() || r.lost_focus())
                && ui.input(|i| {
                    i.key_pressed(egui::Key::Enter)
                        && i.modifiers.matches_logically(egui::Modifiers::NONE)
                });
            m.fields
                .push((format!("host-{n}"), r.rect, id, ui.clip_rect()));
            name_control(&r, format!("Collector address row{}", n + 1));
            let r = ui.add_enabled(active, button("Remove"));
            metric(m, &format!("host-remove-{n}"), &r, ui.clip_rect());
            if r.clicked() {
                remove = Some(n);
            }
        };
        if narrow {
            ui.label(format!("Collector {}", n + 1));
            ui.horizontal(|ui| {
                let width = ui.available_width();
                row(ui, width);
            });
        } else {
            controls::form_row(ui, format!("Collector {}", n + 1), row);
        }
    }
    if let Some(n) = remove {
        draft.hosts.remove(n);
    }
    let r = ui.add_enabled(
        active && draft.hosts.len() < 8,
        button("Add collector address"),
    );
    metric(m, "host-add", &r, ui.clip_rect());
    if r.clicked() {
        draft.hosts.push(String::new());
    }
    if draft != before {
        edit.source = Some((ip, draft));
    }
    edit
}
fn collector_editor(
    ui: &mut egui::Ui,
    ip: Ipv4Addr,
    mut draft: model::CollectorDraft,
    active: bool,
    m: &mut Metrics,
) -> Edits {
    let before = draft.clone();
    let mut remove = None;
    ui.label("Accept a matching facility at this severity or any more urgent level.");
    let row_height = controls::height(ui) + ui.spacing().item_spacing.y;
    let height = row_height * 6.;
    egui::ScrollArea::vertical()
        .id_salt("selectors")
        .min_scrolled_height(height)
        .max_height(height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (n, row) in draft.rows.iter_mut().enumerate() {
                let narrow = ui.available_width() < controls::height(ui) * 13.;
                let widths = [
                    ui.available_width() * 0.46,
                    ui.available_width() * 0.34,
                    ui.available_width() * 0.16,
                ];
                if narrow {
                    ui.label(format!("Selector {}", n + 1));
                    controls::form_row(ui, "Facility", |ui, _| {
                        choice_facility(
                            ui,
                            ("selector-facility", n),
                            &mut row.facility,
                            m,
                            &format!("selector-facility-{n}"),
                        )
                    });
                    controls::form_row(ui, "Accept severity", |ui, _| {
                        let mut s = Some(row.threshold);
                        choice_severity(
                            ui,
                            ("selector-threshold", n),
                            &mut s,
                            false,
                            m,
                            &format!("selector-threshold-{n}"),
                        );
                        row.threshold = s.unwrap();
                    });
                    let r = ui.add_enabled(active, button("Remove"));
                    metric(m, &format!("selector-remove-{n}"), &r, ui.clip_rect());
                    if r.clicked() {
                        remove = Some(n);
                    }
                } else {
                    ui.horizontal(|ui| {
                        controls::cell(ui, widths[0], |ui| {
                            choice_facility(
                                ui,
                                ("selector-facility", n),
                                &mut row.facility,
                                m,
                                &format!("selector-facility-{n}"),
                            )
                        });
                        controls::cell(ui, widths[1], |ui| {
                            let mut s = Some(row.threshold);
                            choice_severity(
                                ui,
                                ("selector-threshold", n),
                                &mut s,
                                false,
                                m,
                                &format!("selector-threshold-{n}"),
                            );
                            row.threshold = s.unwrap();
                        });
                        controls::cell(ui, widths[2], |ui| {
                            let r = ui.add_enabled(active, button("Remove"));
                            metric(m, &format!("selector-remove-{n}"), &r, ui.clip_rect());
                            if r.clicked() {
                                remove = Some(n);
                            }
                        });
                    });
                }
            }
        });
    if let Some(n) = remove {
        draft.rows.remove(n);
    }
    let r = ui.add_enabled(
        active && draft.rows.len() < 24,
        button("Add facility selector"),
    );
    metric(m, "selector-add", &r, ui.clip_rect());
    if r.clicked() {
        draft.rows.push(model::SelectorDraft {
            facility: None,
            threshold: core::Severity::Warning,
        });
    }
    Edits {
        collector: (draft != before).then_some((ip, draft)),
        ..Default::default()
    }
}
fn selected_editor(
    ui: &mut egui::Ui,
    editor: &model::Editor,
    scope: egui::Id,
    active: bool,
    invalid: bool,
    m: &mut Metrics,
) -> Edits {
    ui.push_id(scope, |ui| {
        ui.add_enabled_ui(active, |ui| match editor.selected {
            model::Component::Source(ip) => source_editor(
                ui,
                ip,
                editor.sources[&ip].clone(),
                scope,
                active,
                invalid,
                m,
            ),
            model::Component::Collector(ip) => {
                collector_editor(ui, ip, editor.collectors[&ip].clone(), active, m)
            }
        })
        .inner
    })
    .inner
}
fn evidence(ui: &mut egui::Ui, world: &core::State, editor: &model::Editor, m: &mut Metrics) {
    let env = world.environment().unwrap();
    let event = &env.events[&editor.event];
    ui.label(format!(
        "Selected event: {} · severity {} ({})",
        editor.event.as_str(),
        event.severity.number(),
        event.severity.name()
    ));
    ui.label(event.cisco_style_text().unwrap());
    if let model::Component::Source(device) = editor.selected {
        let policy = world.device_policy(device).unwrap();
        let target = policy
            .hosts
            .iter()
            .next()
            .copied()
            .unwrap_or(*env.collectors.keys().next().unwrap());
        let decision = world.current_remote(device, &editor.event, target).unwrap();
        let remote_name = env
            .collectors
            .get(&target)
            .map_or("Unknown supplied collector", |e| e.name.as_str());
        let color = if matches!(decision, core::RemoteDisposition::Accepted { .. }) {
            palette(ui).success
        } else {
            palette(ui).error
        };
        let r=ui.scope(|ui|{controls::topology_pair(
            ui,
            [
                &env.devices[&device].name,
                &device.to_string(),
                &format!("Export: {}", model::facility_label(policy.export_facility)),
            ],
            [remote_name, &target.to_string(), &decision.explanation()],
            color,
            "Preview of one collector · Check evaluates every required case, using supplied delivery facts",
        );});
        m.topology = Some(r.response.rect);
    }
    let title = match editor.selected {
        model::Component::Source(_) => "Historical local buffer",
        model::Component::Collector(_) => "Historical collector inbox",
    };
    ui.strong(title);
    ui.label(
        "Older records stay after policy changes. Check configuration tests current settings.",
    );
    let mut lines = Vec::new();
    match editor.selected {
        model::Component::Source(ip) => {
            for r in world.local_history(ip).unwrap() {
                lines.push(format!(
                    "#{} {}",
                    r.sequence,
                    env.events[&r.event].cisco_style_text().unwrap()
                ));
            }
        }
        model::Component::Collector(ip) => {
            for r in world.collector_history(ip).unwrap() {
                lines.push(format!(
                    "#{} from {} · {} · {}",
                    r.sequence,
                    r.device,
                    model::facility_label(r.facility),
                    env.events[&r.event].cisco_style_text().unwrap()
                ));
            }
        }
    }
    let text = if lines.is_empty() {
        "No accepted events in this historical log yet.".into()
    } else {
        lines.join("\n")
    };
    let line = ui.text_style_height(&egui::TextStyle::Monospace) + ui.spacing().item_spacing.y;
    let height = line * 6.;
    let out = egui::ScrollArea::vertical()
        .id_salt("history")
        .min_scrolled_height(height)
        .max_height(height)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.monospace(&text);
        });
    m.history = Some((out.inner_rect, height));
}
impl LabApp {
    pub(super) fn syslog_page(&mut self, ui: &mut egui::Ui, i: usize) {
        if let Err(e) = model::context(
            &self.progress.attempts[&self.scenarios[i].id],
            &self.scenarios[i],
        ) {
            ui.colored_label(palette(ui).error, format!("Syslog work retained: {e}"));
            return;
        }
        let source = &self.scenarios[i];
        let a = &self.progress.attempts[&source.id];
        let owner = (source.id.clone(), source.version, a.visited, a.challenge);
        if self.syslog_presentation.owner.as_ref() != Some(&owner) {
            self.syslog_presentation.notice = None;
            self.syslog_presentation.owner = Some(owner);
        }
        let root = ui.ctx().cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let packet = ui.input(|i| {
            i.events
                .iter()
                .any(|e| matches!(e, egui::Event::Text(_) | egui::Event::Paste(_)))
        });
        let invalid=ui.input(|i|i.events.iter().any(|e|matches!(e,egui::Event::Text(t)|egui::Event::Paste(t)if t.len()>15||t.chars().any(|c|c.is_control()||matches!(c,'\u{2028}'|'\u{2029}')))));
        let after_enter = ui.input(|i| {
            let mut enter = false;
            i.events.iter().any(|e| {
                if matches!(
                    e,
                    egui::Event::Key {
                        key: egui::Key::Enter,
                        pressed: true,
                        ..
                    }
                ) {
                    enter = true;
                }
                enter && matches!(e,egui::Event::Text(t)|egui::Event::Paste(t)if !t.is_empty())
            })
        });
        let inherited = self.syslog_presentation.held_veto;
        if !ui.input(|i| i.pointer.primary_down()) {
            self.syslog_presentation.held_veto = false;
        }
        let blocked = inherited || self.syslog_presentation.veto_root == Some(root);
        let before = (
            self.selected,
            self.reset_pending,
            self.progress.attempts[&self.scenarios[i].id].challenge,
        );
        let challenge = ui
            .add_enabled_ui(!packet && !blocked, |ui| self.workspace_header(ui, i))
            .inner;
        if before
            != (
                self.selected,
                self.reset_pending,
                self.progress.attempts[&self.scenarios[i].id].challenge,
            )
        {
            return;
        }
        let source = &self.scenarios[i];
        let engine::Goal::Syslog { definition: d } = &source.goal else {
            return;
        };
        let attempt = self.progress.attempts.get_mut(&source.id).unwrap();
        let State::Syslog { simulation } = &attempt.state else {
            return;
        };
        // Stage this frame's edits privately. The held Attempt is untouched until
        // the entire candidate (including common metadata) has been admitted.
        let mut staged = simulation.as_ref().clone();
        let s = &mut staged;
        let mut hints = attempt.hints;
        let dirty_before = self.dirty;
        let generation_before = self.syslog_presentation.generation;
        let view = if challenge { View::Assignment } else { s.view };
        let mut navigation = None;
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (View::Introduction, "introduction", "Introduction"),
                    (View::Example, "example", "Try example · ungraded"),
                    (View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(
                        !packet && !blocked,
                        egui::Button::selectable(view == v, label),
                    );
                    metric(&mut self.metrics.syslog, key, &r, ui.clip_rect());
                    if r.clicked() {
                        navigation = Some(v);
                    }
                }
            });
        }
        if let Some(v) = navigation {
            let result = s
                .set_view(d, v)
                .and_then(|()| model::binding::publish(attempt, source, s, hints));
            match result {
                Ok(()) => {
                    self.dirty = true;
                    self.syslog_presentation.notice = None;
                }
                Err(e) => self.syslog_presentation.notice = Some((false, e)),
            }
            ui.ctx().request_repaint();
            return;
        }
        if view == View::Introduction {
            if lesson_presentation::entry(source).is_some() {
                lesson_presentation::concepts(
                    ui,
                    source,
                    &d.lesson.prerequisite_note,
                    d.lesson
                        .concepts
                        .iter()
                        .map(|c| (c.title.clone(), c.text.clone())),
                );
                lesson_presentation::worked(ui, &d.lesson.description);
                lesson_presentation::worked_rows(ui, source);
            } else {
                ui.heading("Understand the task");
                ui.label(&d.lesson.prerequisite_note);
                for c in &d.lesson.concepts {
                    ui.strong(&c.title);
                    ui.label(&c.text);
                }
            }
            return;
        }
        let practice = view == View::Example && !challenge;
        let generation = self.syslog_presentation.generation[usize::from(practice)];
        let scope = egui::Id::new((
            "syslog-editor",
            &source.id,
            source.version,
            attempt.visited,
            practice,
            generation,
        ));
        let mut fill = None;
        let mut reset_example = false;
        if practice {
            if lesson_presentation::entry(source).is_some() {
                lesson_presentation::lead(ui, "Practice task", &d.lesson.description);
                lesson_presentation::practice_step(ui, source, 0);
            } else {
                ui.heading("Separate example · no assignment credit");
                ui.label(&d.lesson.description);
                ui.label(&d.lesson.try_change);
            }
            ui.horizontal_wrapped(|ui| {
                let r = controls::dropdown(ui, "worked-step", ui.available_width() * 0.6)
                    .selected_text(format!("Example action {}", s.selected_example + 1))
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        let first = self.metrics.syslog.choices.len();
                        for (n, w) in d.lesson.worked.iter().enumerate() {
                            let r = ui.selectable_label(s.selected_example == n, &w.explanation);
                            self.metrics.syslog.choices.push((
                                format!("worked-{n}"),
                                r.rect,
                                controls::popup_content_clip(ui),
                            ));
                            if r.clicked() {
                                fill = Some((n, false));
                            }
                        }
                        finish_choices(ui, &mut self.metrics.syslog, first);
                    })
                    .response;
                metric(
                    &mut self.metrics.syslog,
                    "worked-select",
                    &r,
                    ui.clip_rect(),
                );
                let r = ui.add_enabled(!packet && !blocked, button("Fill example settings"));
                metric(&mut self.metrics.syslog, "fill-example", &r, ui.clip_rect());
                if r.clicked() {
                    fill = Some((s.selected_example, true));
                }
                let r = ui.add_enabled(!packet && !blocked, button("Reset example"));
                metric(
                    &mut self.metrics.syslog,
                    "reset-example",
                    &r,
                    ui.clip_rect(),
                );
                reset_example = r.clicked();
            });
            lesson_presentation::prose(ui, &d.lesson.worked[s.selected_example].explanation);
            if lesson_presentation::entry(source).is_some() {
                lesson_presentation::practice_step(ui, source, 1);
            }
        } else {
            ui.heading(&source.title);
            if lesson_presentation::entry(source).is_some() {
                lesson_presentation::assignment(ui, source);
                self.metrics.syslog.goal = Some(ui.min_rect());
            } else {
                self.metrics.syslog.goal = Some(ui.label(&source.objective).rect);
            }
        }
        let active = ui.is_enabled()
            && !blocked
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let (world, editor) = s.inspect(d, practice).unwrap();
        let mut selected = editor.selected;
        let mut event = editor.event.clone();
        let env = world.environment().unwrap();
        controls::form_row(ui, "Selected component", |ui, _| {
            let label = match selected {
                model::Component::Source(ip) => {
                    format!("Source · {} ({ip})", env.devices[&ip].name)
                }
                model::Component::Collector(ip) => {
                    format!("Collector · {} ({ip})", env.collectors[&ip].name)
                }
            };
            let r = controls::dropdown(ui, "syslog-component", ui.available_width())
                .selected_text(label)
                .show_ui(ui, |ui| {
                    controls::fit_popup_contents(ui);
                    let first = self.metrics.syslog.choices.len();
                    for (ip, e) in &env.devices {
                        let r = ui.selectable_value(
                            &mut selected,
                            model::Component::Source(*ip),
                            format!("Source · {} ({ip})", e.name),
                        );
                        self.metrics.syslog.choices.push((
                            format!("source-{ip}"),
                            r.rect,
                            controls::popup_content_clip(ui),
                        ));
                    }
                    for (ip, e) in &env.collectors {
                        let r = ui.selectable_value(
                            &mut selected,
                            model::Component::Collector(*ip),
                            format!("Collector · {} ({ip})", e.name),
                        );
                        self.metrics.syslog.choices.push((
                            format!("collector-{ip}"),
                            r.rect,
                            controls::popup_content_clip(ui),
                        ));
                    }
                    finish_choices(ui, &mut self.metrics.syslog, first);
                })
                .response;
            name_control(&r, "Selected logging component");
            metric(&mut self.metrics.syslog, "component", &r, ui.clip_rect());
        });
        let mut edit = Edits::default();
        let component_scope = scope.with(editor.selected);
        let wide = ui.available_width() >= controls::height(ui) * 24.;
        if wide {
            ui.columns(2, |columns| {
                columns[0].with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    edit = selected_editor(
                        ui,
                        editor,
                        component_scope,
                        active,
                        invalid,
                        &mut self.metrics.syslog,
                    );
                });
                columns[1].with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.push_id((scope, editor.selected), |ui| {
                        evidence(ui, world, editor, &mut self.metrics.syslog)
                    });
                });
            });
        } else {
            edit = selected_editor(
                ui,
                editor,
                component_scope,
                active,
                invalid,
                &mut self.metrics.syslog,
            );
        }
        let rejected = edit.rejected || invalid || after_enter || (packet && !edit.received);
        if rejected {
            for (id, before) in edit.native.drain(..) {
                before.restore(ui.ctx(), id);
            }
            self.syslog_presentation.veto_root = Some(root);
            self.syslog_presentation.held_veto |= ui.input(|i| i.pointer.primary_down());
            self.syslog_presentation.notice=Some((false,"That address edit was not applied. The full prior draft and selection were retained; retry in the address field before choosing an action.".into()));
        }
        let selection_changed = selected != editor.selected || event != editor.event;
        let draft_changed = edit.source.is_some() || edit.collector.is_some();
        let mut result: Option<Result<(), String>> = None;
        if !rejected {
            if let Some((ip, draft)) = edit.source {
                result = Some(s.set_source_draft(d, practice, ip, draft));
            }
            if let Some((ip, draft)) = edit.collector {
                result = Some(s.set_collector_draft(d, practice, ip, draft));
            }
        }

        let mut apply = false;
        let mut generate = false;
        let mut check = false;
        let mut clear = false;
        let allowed = active && !packet && !rejected && !draft_changed && !selection_changed;
        let (world, editor) = s.inspect(d, practice).unwrap();
        let env = world.environment().unwrap();
        controls::form_row(ui, "Supplied event", |ui, _| {
            let r = controls::dropdown(ui, "syslog-event", ui.available_width())
                .selected_text(event.as_str())
                .show_ui(ui, |ui| {
                    controls::fit_popup_contents(ui);
                    let first = self.metrics.syslog.choices.len();
                    for (id, e) in &env.events {
                        let r = ui.selectable_value(
                            &mut event,
                            id.clone(),
                            format!(
                                "{} · severity {} {}",
                                id.as_str(),
                                e.severity.number(),
                                e.severity.name()
                            ),
                        );
                        self.metrics.syslog.choices.push((
                            format!("event-{}", id.as_str()),
                            r.rect,
                            controls::popup_content_clip(ui),
                        ));
                    }
                    finish_choices(ui, &mut self.metrics.syslog, first);
                })
                .response;
            name_control(&r, "Selected supplied event");
            metric(&mut self.metrics.syslog, "event", &r, ui.clip_rect());
        });
        if matches!(editor.selected, model::Component::Source(_)) {
            let r = ui.add_enabled(allowed, button("Generate simulated event"));
            metric(&mut self.metrics.syslog, "generate", &r, ui.clip_rect());
            generate = r.clicked();
        }
        if practice && lesson_presentation::entry(source).is_some() {
            lesson_presentation::practice_step(ui, source, 2);
        }
        let mut help_action = false;
        let help = controls::collapsing_header(ui, "Optional help and references")
            .id_salt((scope, "help", challenge))
            .show(ui, |ui| {
                let r = ui.add_enabled(
                    active && !packet && hints < source.hints.len(),
                    button("Show a hint"),
                );
                metric(&mut self.metrics.syslog, "show-hint", &r, ui.clip_rect());
                help_action |= guidance_intent(ui, &r);
                if r.clicked() {
                    hints = (hints + 1).min(source.hints.len());
                    self.dirty = true;
                    help_action = true;
                    ui.ctx().request_repaint();
                }
                for (n, h) in source.hints.iter().take(hints).enumerate() {
                    let text = format!("Hint {}: {h}", n + 1);
                    let r = ui.label(&text);
                    self.metrics
                        .syslog
                        .help_bodies
                        .push((text, r.rect, ui.clip_rect()));
                }
                let worked =
                    controls::collapsing_header(ui, "Worked approach · reveal when requested")
                        .id_salt((scope, "worked", challenge))
                        .show(ui, |ui| {
                            let r = ui.label(&source.explanation);
                            self.metrics.syslog.help_bodies.push((
                                source.explanation.clone(),
                                r.rect,
                                ui.clip_rect(),
                            ));
                        });
                metric(
                    &mut self.metrics.syslog,
                    "worked-approach",
                    &worked.header_response,
                    ui.clip_rect(),
                );
                #[cfg(test)]
                if worked.body_returned.is_some() {
                    self.metrics.syslog.disclosures.push("worked-approach");
                }
                help_action |= guidance_intent(ui, &worked.header_response);
                let references =
                    controls::collapsing_header(ui, "References and simulation limits")
                        .id_salt((scope, "refs", challenge))
                        .show(ui, |ui| {
                            ui.label(&source.simplifications);
                            for url in &source.sources {
                                let r = ui.hyperlink(url);
                                self.metrics.syslog.help_bodies.push((
                                    url.clone(),
                                    r.rect,
                                    ui.clip_rect(),
                                ));
                            }
                        });
                metric(
                    &mut self.metrics.syslog,
                    "references",
                    &references.header_response,
                    ui.clip_rect(),
                );
                #[cfg(test)]
                if references.body_returned.is_some() {
                    self.metrics.syslog.disclosures.push("references");
                }
                help_action |= guidance_intent(ui, &references.header_response);
                let r = ui.add_enabled(allowed, button("Clear selected historical log"));
                metric(
                    &mut self.metrics.syslog,
                    "clear-history",
                    &r,
                    ui.clip_rect(),
                );
                clear = r.clicked();
            });
        metric(
            &mut self.metrics.syslog,
            "optional-help",
            &help.header_response,
            ui.clip_rect(),
        );
        #[cfg(test)]
        if help.body_returned.is_some() {
            self.metrics.syslog.disclosures.push("optional-help");
        }
        help_action |= guidance_intent(ui, &help.header_response);
        if s.has_unapplied(d, practice).unwrap() {
            ui.label("Unapplied changes · Apply settings to update the policy. Generated events still use the previously applied settings.");
        }
        ui.horizontal_wrapped(|ui| {
            let r = ui.add_enabled(allowed, button("Apply settings"));
            metric(&mut self.metrics.syslog, "apply", &r, ui.clip_rect());
            apply = r.clicked() || edit.enter && allowed;
            let r = ui.add_enabled(
                allowed,
                primary(
                    ui,
                    if practice {
                        "Check separate example"
                    } else {
                        "Check configuration"
                    },
                ),
            );
            metric(&mut self.metrics.syslog, "check", &r, ui.clip_rect());
            self.metrics.check = Some(r.rect);
            check = r.clicked();
        });
        if let Some(r) = result {
            self.dirty |= r.is_ok();
            self.syslog_presentation.notice = r.err().map(|e| (false, e));
        }
        if let Some((ok, text)) = &self.syslog_presentation.notice {
            let r = ui.colored_label(
                if *ok {
                    palette(ui).success
                } else {
                    palette(ui).error
                },
                format!("{} · {text}", if *ok { "Done" } else { "Needs attention" }),
            );
            self.metrics.syslog.texts.push(text.clone());
            metric(&mut self.metrics.syslog, "notice", &r, ui.clip_rect());
        }
        help_action |= current_results(
            ui,
            d,
            s,
            practice,
            scope.with(("check-details", challenge)),
            &mut self.metrics.syslog,
        );
        #[cfg(test)]
        {
            self.metrics.syslog.entered = edit.enter;
            self.metrics.syslog.intents = (apply, generate, check);
            self.metrics.syslog.guidance = help_action;
        }
        // Match the accepted NTP intent rule: all guidance blocks dispatch now,
        // but normal hint/header presses must remain enabled for later release.
        if help_action
            && (packet
                || rejected
                || edit.enter
                || apply
                || generate
                || check
                || clear
                || blocked
                || draft_changed
                || selection_changed)
        {
            self.syslog_presentation.veto_root = Some(root);
            self.syslog_presentation.held_veto |= ui.input(|i| i.pointer.primary_down());
        }
        let conflicting =
            usize::from(apply) + usize::from(generate) + usize::from(check) + usize::from(clear)
                > 1;
        if conflicting {
            self.syslog_presentation.notice = Some((
                false,
                "Choose one action at a time. Previous applied settings were retained.".into(),
            ));
        }
        let dispatch = allowed
            && !conflicting
            && !help_action
            && fill.is_none()
            && !reset_example
            && selected == s.inspect(d, practice).unwrap().1.selected
            && event == s.inspect(d, practice).unwrap().1.event;
        if active && !packet && !rejected {
            if selected != s.inspect(d, practice).unwrap().1.selected {
                result_notice(
                    &mut self.syslog_presentation,
                    &mut self.dirty,
                    s.select(d, practice, selected)
                        .map(|_| "Component selected; all drafts retained.".into()),
                );
            } else if event != s.inspect(d, practice).unwrap().1.event {
                result_notice(
                    &mut self.syslog_presentation,
                    &mut self.dirty,
                    s.select_event(d, practice, event)
                        .map(|_| "Event selected; nothing generated.".into()),
                );
            }
        }
        if dispatch {
            if apply {
                result_notice(
                    &mut self.syslog_presentation,
                    &mut self.dirty,
                    s.apply_selected(d, practice),
                );
            } else if generate {
                result_notice(
                    &mut self.syslog_presentation,
                    &mut self.dirty,
                    s.generate(d, practice),
                );
            } else if check {
                result_notice(&mut self.syslog_presentation,&mut self.dirty,s.check(d,practice).map(|_|"Current configuration checked; historical records do not determine success.".into()));
            } else if clear {
                result_notice(
                    &mut self.syslog_presentation,
                    &mut self.dirty,
                    s.clear_selected(d, practice),
                );
            }
        }
        if active && !packet && !rejected {
            if let Some((index, insert)) = fill {
                if insert {
                    result_notice(
                        &mut self.syslog_presentation,
                        &mut self.dirty,
                        s.fill_worked(d, index),
                    );
                } else if index < d.lesson.worked.len() {
                    s.selected_example = index;
                    self.dirty = true;
                }
            } else if reset_example {
                let result = s
                    .reset_example(d)
                    .map(|_| "Separate example reset; assignment retained.".into());
                if result.is_ok() {
                    self.syslog_presentation.reset_example();
                }
                result_notice(&mut self.syslog_presentation, &mut self.dirty, result);
            }
        }
        // Hints, selections, example reset/fill, policy actions and checked flags
        // all share one complete Attempt admission. Nothing is partially published.
        let admission = model::binding::publish(attempt, source, s, hints);
        if let Err(e) = admission {
            let State::Syslog { simulation } = &attempt.state else {
                unreachable!()
            };
            s.clone_from(simulation.as_ref());
            self.dirty = dirty_before;
            self.syslog_presentation.generation = generation_before;
            for (id, before) in edit.native.drain(..) {
                before.restore(ui.ctx(), id);
            }
            self.syslog_presentation.veto_root = Some(root);
            self.syslog_presentation.held_veto |= ui.input(|i| i.pointer.primary_down());
            self.syslog_presentation.notice = Some((false, e));
            ui.ctx().request_repaint();
            // Do not offer Next on a frame whose prospective change was refused.
            return;
        }
        if self.dirty
            || conflicting
            || rejected
            || (dispatch && (apply || generate || check || clear))
            || fill.is_some()
            || reset_example
            || selection_changed
            || help_action
        {
            ui.ctx().request_repaint();
        }
        let (world, editor) = s.inspect(d, practice).unwrap();
        if !wide {
            ui.push_id((scope, editor.selected), |ui| {
                evidence(ui, world, editor, &mut self.metrics.syslog)
            });
        }
        self.metrics.completion = !practice && progress::current_solved(attempt, source);
        if self.metrics.completion
            && active
            && !packet
            && !rejected
            && !draft_changed
            && !selection_changed
            && !apply
            && !generate
            && !check
            && !clear
            && !help_action
            && !reset_example
            && fill.is_none()
        {
            self.next_unit_lab(ui, i);
        }
    }
}
fn result_notice(p: &mut Presentation, dirty: &mut bool, result: Result<String, String>) {
    match result {
        Ok(text) => {
            *dirty = true;
            p.notice = Some((true, text));
        }
        Err(e) => p.notice = Some((false, e)),
    }
}
