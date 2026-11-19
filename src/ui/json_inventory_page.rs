//! Native JSON inventory editing with actual simulated switching outcomes.
use super::*;
use dlms_learning_lab::{
    json_inventory as inventory, json_inventory_workspace as work,
    switching::{Jack, Mode},
};

#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub ids: Vec<(String, egui::Id)>,
    pub shown_outcomes: Vec<(bool, String)>,
    pub practice_level: Option<usize>,
}

impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, response: &egui::Response) {
        self.controls
            .push((key.into(), response.rect, ui.clip_rect()));
        self.ids.push((key.into(), response.id));
    }
}

fn editor(
    ui: &mut egui::Ui,
    draft: &mut work::Draft,
    task: &work::Task,
    notice: &mut Option<String>,
    metrics: &mut Metrics,
) {
    let label = ui.strong("inventory.json · simulated access-port document");
    let mut text = draft.source.clone();
    let width = ui.available_width();
    let height = ui.text_style_height(&egui::TextStyle::Monospace).ceil() * 10.
        + 2. * f32::from(controls::PAD_Y);
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let color = palette(ui).text;
    let mut layouter = move |ui: &egui::Ui, text: &dyn egui::TextBuffer, _: f32| {
        ui.fonts_mut(|fonts| fonts.layout_no_wrap(text.as_str().into(), font.clone(), color))
    };
    let (viewport, response) = controls::fixed_viewport(ui, egui::vec2(width, height), |ui| {
        controls::nested_scroll(
            ui,
            egui::ScrollArea::both()
                .id_salt(("json-inventory-editor-scroll", &task.id))
                .max_width(width)
                .min_scrolled_width(width)
                .max_height(height)
                .auto_shrink([false, false]),
            |ui| {
                egui::TextEdit::multiline(&mut text)
                    .code_editor()
                    .lock_focus(false)
                    .layouter(&mut layouter)
                    .id(egui::Id::new(("json-inventory-document", &task.id)))
                    .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                    .desired_width(
                        (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(24.),
                    )
                    .desired_rows(10)
                    .char_limit(inventory::MAX_DOCUMENT)
                    .show(ui)
                    .response
                    .labelled_by(label.id)
            },
        )
        .inner
    });
    metrics
        .controls
        .push(("editor".into(), viewport, ui.clip_rect()));
    metrics.ids.push(("editor".into(), response.id));
    if response.changed() {
        *notice = draft.edit(text).err();
    }
    lesson_presentation::prose(
        ui,
        if draft.source == draft.applied {
            "The edited file matches the applied configuration."
        } else {
            "The simulation still uses the previous configuration. Apply the edited file before checking."
        },
    );
    ui.horizontal_wrapped(|ui| {
        let apply = ui.add(button("Apply inventory"));
        metrics.record(ui, "apply", &apply);
        if apply.clicked() {
            *notice = draft.apply(task).err();
        }
        let check = ui.add(primary(ui, "Check network"));
        metrics.record(ui, "check", &check);
        if check.clicked() {
            *notice = draft.test(task).err();
        }
    });
    if let Some(message) = notice.as_ref() {
        let response = lesson_presentation::prose(ui, message);
        metrics.record(ui, "notice", &response);
    }
    if draft.checked {
        let checks = draft.checks(task);
        let passed = checks.iter().filter(|check| check.0).count();
        let summary = if passed == checks.len() {
            format!("Passed · {passed}/{} network requirements", checks.len())
        } else {
            format!(
                "Needs repair · {passed}/{} network requirements passed",
                checks.len()
            )
        };
        let response = ui.strong(summary);
        metrics.record(ui, "result", &response);
        for (passed, message) in checks.iter().filter(|(passed, _)| !*passed) {
            lesson_presentation::prose(ui, &format!("Needs repair · {message}"));
            metrics.shown_outcomes.push((*passed, message.clone()));
        }
        controls::collapsing_header(ui, "All network results · optional").show(ui, |ui| {
            for (passed, message) in &checks {
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "{} · {message}",
                        if *passed { "Pass" } else { "Needs repair" }
                    ),
                );
            }
        });
    }
}

fn mapping(ui: &mut egui::Ui, task: &work::Task, draft: &work::Draft) {
    ui.strong("Supplied connections");
    for (name, switch) in &task.inventory.fabric.switches {
        let vlans = switch
            .vlans
            .iter()
            .map(u16::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        lesson_presentation::prose(ui, &format!("{name} · approved VLAN IDs: {vlans}"));
    }
    let column_width = ((ui.available_width() - 2. * controls::GAP) / 3.).max(1.);
    egui::Grid::new("inventory-port-map")
        .num_columns(3)
        .min_col_width(column_width)
        .spacing([controls::GAP, controls::GAP])
        .show(ui, |ui| {
            for title in ["Switch", "Port", "Endpoint"] {
                ui.strong(title);
            }
            ui.end_row();
            for cable in &task.inventory.fabric.cables {
                let pair = match (&cable.a, &cable.b) {
                    (Jack::Host { host }, Jack::Switch { switch, port })
                    | (Jack::Switch { switch, port }, Jack::Host { host }) => {
                        Some((host, switch, port))
                    }
                    _ => None,
                };
                if let Some((host, switch, port)) = pair {
                    lesson_presentation::monospace(ui, switch, true);
                    lesson_presentation::monospace(ui, port, true);
                    lesson_presentation::prose(ui, &task.inventory.fabric.hosts[host].label);
                    ui.end_row();
                }
            }
        });
    controls::collapsing_header(ui, "Applied port settings · optional").show(ui, |ui| {
        let applied = task
            .inventory
            .apply(&draft.applied)
            .expect("admitted applied inventory");
        for (name, ports) in &task.inventory.editable {
            for name_port in ports {
                let port = &applied.switches[name].ports[name_port];
                let Mode::Access { vlan } = port.mode else {
                    unreachable!("fixed editable access ports")
                };
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "{name}/{name_port} · {} · VLAN {vlan}",
                        if port.up { "Up" } else { "Down" }
                    ),
                );
            }
        }
    });
}

impl LabApp {
    pub(super) fn json_inventory_page(&mut self, ui: &mut egui::Ui, index: usize) {
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
        let engine::State::JsonInventory { simulation } = &attempt.state else {
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
        self.metrics.json_inventory = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (choice, title) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let response = ui.add_enabled(active, button(title).selected(choice == view));
                    let key = match choice {
                        teaching::View::Introduction => "introduction",
                        teaching::View::Example => "example",
                        teaching::View::Assignment => "assignment",
                    };
                    self.metrics.json_inventory.record(ui, key, &response);
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
                    "Review VLAN groups and their connections before editing this inventory.",
                    std::iter::empty(),
                );
                lesson_presentation::wrapped_worked_commands(ui, &source);
                ui.horizontal_wrapped(|ui| {
                    for (choice, title) in [
                        (teaching::View::Example, "Try this example"),
                        (teaching::View::Assignment, "Open full assignment"),
                    ] {
                        if ui.add_enabled(active, button(title)).clicked() {
                            next.view = choice;
                            self.lesson_scroll_reset = true;
                            self.notice = None;
                        }
                    }
                });
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
                let step = if task.inventory.apply(&draft.source).is_err() {
                    0
                } else if draft.checked && draft.checks(task).iter().all(|check| check.0) {
                    2
                } else {
                    1
                };
                self.metrics.json_inventory.practice_level = Some(step);
                lesson_presentation::practice_step(ui, &source, step);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            ui.add_enabled_ui(active, |ui| {
                if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 48. {
                    ui.columns(2, |columns| {
                        editor(
                            &mut columns[0],
                            draft,
                            task,
                            &mut self.notice,
                            &mut self.metrics.json_inventory,
                        );
                        mapping(&mut columns[1], task, draft);
                    });
                } else {
                    mapping(ui, task, draft);
                    ui.add_space(controls::GAP);
                    editor(
                        ui,
                        draft,
                        task,
                        &mut self.notice,
                        &mut self.metrics.json_inventory,
                    );
                }
                controls::collapsing_header(ui, "Hints and supported behavior · optional").show(
                    ui,
                    |ui| {
                        for hint in &source.hints {
                            lesson_presentation::prose(ui, hint);
                        }
                        lesson_presentation::prose(ui, work::HELP);
                    },
                );
                if practice {
                    controls::collapsing_header(ui, "Worked practice · optional").show(ui, |ui| {
                        lesson_presentation::monospace(ui, &task.reference, true);
                        if ui.add(button("Fill worked example")).clicked() {
                            self.notice = draft.edit(task.reference.clone()).err();
                        }
                    });
                }
            });
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state =
                engine::State::JsonInventory {
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
