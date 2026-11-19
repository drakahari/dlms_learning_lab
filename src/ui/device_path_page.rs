//! Native cable topology and aligned connection editing for wired-device practice.
use super::*;
use dlms_learning_lab::{device_path_workspace as work, switching::Jack};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, response: &egui::Response) {
        self.controls
            .push((key.into(), response.rect, ui.clip_rect()));
    }
}

fn jack_title(task: &work::Task, jack: &Jack) -> String {
    match jack {
        Jack::Host { host } => task.facts.fabric.hosts[host].label.clone(),
        Jack::Switch { switch, port } => {
            format!("{} {port}", task.facts.fabric.switches[switch].label)
        }
    }
}

fn node(task: &work::Task, jack: &Jack) -> String {
    match jack {
        Jack::Switch { switch, .. } => format!("switch-{switch}"),
        Jack::Host { host } => task
            .facts
            .attachments
            .iter()
            .find(|binding| &binding.endpoint == host)
            .map(|binding| binding.device.clone())
            .unwrap_or_default(),
    }
}

fn topology(ui: &mut egui::Ui, task: &work::Task, draft: &work::Draft, metrics: &mut Metrics) {
    ui.strong("Equipment and fixed addressing");
    let mut cards = Vec::new();
    for (id, device) in &task.facts.network.devices {
        let title = match id.as_str() {
            "client" => "Workstation",
            "edge" => "Router edge",
            "monitor" => "Monitor",
            _ => "Server",
        };
        let addresses = device
            .interfaces
            .iter()
            .map(|(name, interface)| {
                let address = interface
                    .address
                    .as_ref()
                    .expect("admitted fixed addressing");
                format!("{name}: {}/{}", address.address, address.prefix)
            })
            .collect::<Vec<_>>()
            .join("\n");
        cards.push((id.clone(), title.to_owned(), addresses));
    }
    for (id, switch) in &task.facts.fabric.switches {
        cards.push((
            format!("switch-{id}"),
            switch.label.clone(),
            "Layer 2 · untagged VLAN 1".into(),
        ));
    }
    // A responsive equipment grid depicts the current cable endpoints. Card size
    // comes from measured text and shared padding, with connections at centers.
    let line = ui.text_style_height(&egui::TextStyle::Body);
    let gap = controls::GAP * 3.;
    let columns = ((ui.available_width() + gap) / (line * 15. + gap))
        .floor()
        .clamp(1., 3.) as usize;
    let order = [
        "client",
        "switch-office",
        "edge",
        "monitor",
        "server",
        "switch-servers",
    ];
    cards.sort_by_key(|card| {
        order
            .iter()
            .position(|id| *id == card.0)
            .unwrap_or(usize::MAX)
    });
    if columns < 3 {
        let titles: BTreeMap<_, _> = cards
            .iter()
            .map(|card| (card.0.clone(), card.1.clone()))
            .collect();
        for (id, _, value) in &mut cards {
            let mut neighbors = std::collections::BTreeSet::new();
            for cable in draft.cables.iter().filter(|cable| cable.up) {
                let a = node(task, &cable.a);
                let b = node(task, &cable.b);
                if &a == id {
                    neighbors.insert(titles[&b].clone());
                }
                if &b == id {
                    neighbors.insert(titles[&a].clone());
                }
            }
            let links = if neighbors.is_empty() {
                "No operational cable".into()
            } else {
                neighbors.into_iter().collect::<Vec<_>>().join(", ")
            };
            value.push_str(&format!("\nConnected to · {links}"));
        }
    }
    let width = ((ui.available_width() - gap * (columns - 1) as f32) / columns as f32).max(1.);
    let palette = palette(ui);
    let text_width = (width - 2. * f32::from(controls::PAD_X)).max(1.);
    let layouts: Vec<_> = cards
        .iter()
        .map(|(_, title, addresses)| {
            let title = ui.painter().layout(
                title.clone(),
                egui::TextStyle::Body.resolve(ui.style()),
                palette.text,
                text_width,
            );
            let value = ui.painter().layout(
                addresses.clone(),
                egui::TextStyle::Small.resolve(ui.style()),
                palette.text,
                text_width,
            );
            (title, value)
        })
        .collect();
    let title_height = layouts
        .iter()
        .map(|row| row.0.size().y)
        .fold(line, f32::max);
    let value_height = layouts
        .iter()
        .map(|row| row.1.size().y)
        .fold(line, f32::max);
    let height = title_height + value_height + controls::GAP + 2. * f32::from(controls::PAD_Y);
    let rows = cards.len().div_ceil(columns);
    let (area, response) = ui.allocate_exact_size(
        egui::vec2(
            ui.available_width(),
            rows as f32 * height + (rows - 1) as f32 * gap,
        ),
        egui::Sense::hover(),
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Image, ui.is_enabled(), "Current simulated cable connections. Equipment addresses and cable ends are also available as text in the connection table."));
    metrics.record(ui, "topology", &response);
    let positions: BTreeMap<_, _> = cards
        .iter()
        .enumerate()
        .map(|(index, card)| {
            let origin = area.min
                + egui::vec2(
                    (index % columns) as f32 * (width + gap),
                    (index / columns) as f32 * (height + gap),
                );
            (
                card.0.clone(),
                egui::Rect::from_min_size(origin, egui::vec2(width, height)),
            )
        })
        .collect();
    for cable in draft.cables.iter().filter(|cable| cable.up && columns >= 3) {
        if let (Some(a), Some(b)) = (
            positions.get(&node(task, &cable.a)),
            positions.get(&node(task, &cable.b)),
        ) {
            ui.painter().line_segment(
                [a.center(), b.center()],
                egui::Stroke::new(1.5_f32, palette.muted),
            );
        }
    }
    for (index, (id, _, _)) in cards.iter().enumerate() {
        let rect = positions[id];
        if index == 0 {
            metrics
                .controls
                .push(("topology-start".into(), rect, ui.clip_rect()));
        }
        if index + 1 == cards.len() {
            metrics
                .controls
                .push(("topology-end".into(), rect, ui.clip_rect()));
        }
        ui.painter().rect_filled(rect, 6., palette.panel);
        ui.painter().rect_stroke(
            rect,
            6.,
            egui::Stroke::new(1_f32, palette.muted),
            egui::StrokeKind::Inside,
        );
        let origin = rect.min + egui::vec2(f32::from(controls::PAD_X), f32::from(controls::PAD_Y));
        ui.painter()
            .galley(origin, layouts[index].0.clone(), palette.text);
        ui.painter().galley(
            origin + egui::vec2(0., title_height + controls::GAP),
            layouts[index].1.clone(),
            palette.text,
        );
    }
    lesson_presentation::prose(
        ui,
        if columns >= 3 {
            "Lines show currently operational cables. The table identifies the exact interface and jack at each end. Addresses and routes stay fixed."
        } else {
            "Each card names its current operational neighbors. The table identifies the exact interface and jack at each end. Addresses and routes stay fixed."
        },
    );
}

fn choose(ui: &mut egui::Ui, task: &work::Task, key: &str, jack: &mut Jack, metrics: &mut Metrics) {
    let response = controls::dropdown(ui, key, ui.available_width())
        .selected_text(jack_title(task, jack))
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            let start = metrics.controls.len();
            for (index, choice) in task.choices().into_iter().enumerate() {
                let response = ui.selectable_value(jack, choice.clone(), jack_title(task, &choice));
                metrics.record(ui, &format!("{key}-choice-{index}"), &response);
            }
            let clip = controls::popup_content_clip(ui);
            for control in &mut metrics.controls[start..] {
                control.2 = clip;
            }
        });
    metrics.record(ui, key, &response.response);
}

fn wiring(
    ui: &mut egui::Ui,
    task: &work::Task,
    draft: &mut work::Draft,
    notice: &mut Option<String>,
    metrics: &mut Metrics,
) {
    let mut cables = draft.cables.clone();
    let wide = ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 43.;
    if wide {
        let widths = [0.18, 0.34, 0.34, 0.14];
        let available = ui.available_width() - controls::GAP * 3.;
        egui::Grid::new("device-cables")
            .spacing([controls::GAP, controls::GAP])
            .show(ui, |ui| {
                for (text, fraction) in ["Cable", "End A", "End B", "Operational"]
                    .into_iter()
                    .zip(widths)
                {
                    ui.add_sized(
                        [available * fraction, controls::height(ui)],
                        egui::Label::new(RichText::new(text).strong()),
                    );
                }
                ui.end_row();
                for cable in &mut cables {
                    ui.add_sized(
                        [available * widths[0], controls::height(ui)],
                        egui::Label::new(&cable.id),
                    );
                    for (side, jack) in [("a", &mut cable.a), ("b", &mut cable.b)] {
                        controls::cell(ui, available * widths[1], |ui| {
                            choose(ui, task, &format!("{}-{side}", cable.id), jack, metrics);
                        });
                    }
                    let response = ui.checkbox(&mut cable.up, "Up");
                    metrics.record(ui, &format!("{}-up", cable.id), &response);
                    ui.end_row();
                }
            });
    } else {
        for cable in &mut cables {
            ui.strong(&cable.id);
            for (label, side, jack) in [("End A", "a", &mut cable.a), ("End B", "b", &mut cable.b)]
            {
                ui.label(label);
                choose(ui, task, &format!("{}-{side}", cable.id), jack, metrics);
            }
            let response = ui.checkbox(&mut cable.up, "Operational · Up");
            metrics.record(ui, &format!("{}-up", cable.id), &response);
            ui.separator();
        }
    }
    if cables != draft.cables {
        *notice = draft.edit(task, cables).err();
    }
    let response = ui.add(primary(ui, "Check paths"));
    metrics.record(ui, "check", &response);
    if response.clicked() {
        *notice = draft.test(task).err();
    }
    if !draft.checked {
        ui.label("Paths unchecked");
    } else {
        let checks = draft.checks(task);
        let passed = checks.iter().filter(|row| row.0).count();
        let summary = if checks.len() == task.probes.len() {
            format!("{passed}/{} path requirements passed", checks.len())
        } else {
            "Wiring needs repair before path tests can run".into()
        };
        let response = ui.strong(summary);
        metrics.record(ui, "result", &response);
        for (ok, text) in checks {
            lesson_presentation::prose(
                ui,
                &format!("{} · {text}", if ok { "Pass" } else { "Needs repair" }),
            );
        }
    }
}

impl LabApp {
    pub(super) fn device_path_page(&mut self, ui: &mut egui::Ui, index: usize) {
        let previous = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, index);
        if previous != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[index].clone();
        let attempt = &self.progress.attempts[&source.id];
        let definition = match work::source(&source)
            .and_then(|definition| work::context(attempt, &source).map(|()| definition))
        {
            Ok(value) => value,
            Err(error) => {
                self.notice = Some(error);
                return;
            }
        };
        let engine::State::DevicePath { simulation } = &attempt.state else {
            return;
        };
        let mut next = (**simulation).clone();
        let before = next.clone();
        let active = ui.is_enabled()
            && !(self.reset_pending
                || self.settings_open
                || self.theme_preview
                || self.data_tools.modal_open());
        let mut view = if challenge {
            teaching::View::Assignment
        } else {
            next.view
        };
        self.metrics.device_path = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (choice, title, key) in [
                    (teaching::View::Introduction, "Introduction", "introduction"),
                    (teaching::View::Example, "Try example · ungraded", "example"),
                    (teaching::View::Assignment, "Assignment", "assignment"),
                ] {
                    let response = ui.add_enabled(active, button(title).selected(choice == view));
                    self.metrics.device_path.record(ui, key, &response);
                    self.metrics.lesson_tabs.push(response.rect);
                    if response.clicked() {
                        view = choice;
                        next.view = choice;
                        self.lesson_scroll_reset = true;
                        self.notice = None;
                    }
                }
            });
        }
        ui.push_id((&source.id, view as u8, challenge), |ui| {
            if view == teaching::View::Introduction {
                lesson_presentation::concepts(
                    ui,
                    &source,
                    "Review IPv4 subnets and default gateways.",
                    std::iter::empty(),
                );
                ui.strong("Worked example");
                lesson_presentation::wrapped_worked_commands(ui, &source);
                for (choice, title) in [
                    (teaching::View::Example, "Try this example"),
                    (teaching::View::Assignment, "Open full assignment"),
                ] {
                    if ui.add_enabled(active, button(title)).clicked() {
                        next.view = choice;
                        self.lesson_scroll_reset = true;
                    }
                }
                return;
            }
            let practice = view == teaching::View::Example;
            let (task, draft) = if practice {
                (&definition.example, &mut next.practice)
            } else {
                (&definition.initial, &mut next.assignment)
            };
            if practice {
                lesson_presentation::lead(ui, "PRACTICE TASK", &task.instructions);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            ui.add_enabled_ui(active, |ui| {
                topology(ui, task, draft, &mut self.metrics.device_path);
                ui.add_space(controls::GAP);
                if practice {
                    let step = if draft.checked && draft.checks(task).iter().all(|row| row.0) {
                        2
                    } else if draft.checked || draft.cables != task.initial {
                        1
                    } else {
                        0
                    };
                    lesson_presentation::practice_step(ui, &source, step);
                }
                wiring(
                    ui,
                    task,
                    draft,
                    &mut self.notice,
                    &mut self.metrics.device_path,
                );
                if let Some(notice) = &self.notice {
                    lesson_presentation::prose(ui, notice);
                }
                controls::collapsing_header(ui, "Wiring hints and simulation limits · optional")
                    .show(ui, |ui| {
                        for hint in &source.hints {
                            lesson_presentation::prose(ui, hint);
                        }
                        lesson_presentation::prose(ui, work::HELP);
                    });
                if practice {
                    controls::collapsing_header(ui, "Worked practice wiring · optional").show(
                        ui,
                        |ui| {
                            for cable in &task.reference {
                                lesson_presentation::prose(
                                    ui,
                                    &format!(
                                        "{} · {} / {} · {}",
                                        cable.id,
                                        jack_title(task, &cable.a),
                                        jack_title(task, &cable.b),
                                        if cable.up { "Up" } else { "Down" }
                                    ),
                                );
                            }
                            let response = ui.add(button("Fill worked example"));
                            self.metrics.device_path.record(ui, "fill", &response);
                            if response.clicked() {
                                self.notice = draft.edit(task, task.reference.clone()).err();
                            }
                        },
                    );
                }
            });
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state = engine::State::DevicePath {
                simulation: Box::new(next),
            };
            self.completion_index.forget(&source.id);
            self.dirty = true;
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
        if self.metrics.completion {
            lesson_presentation::prose(ui, &source.explanation);
            self.next_unit_lab(ui, index);
        }
    }
}
