//! Focused native controls for three separate workstation activities.
use super::*;
use dlms_learning_lab::{
    desktop::{
        remote::{Client, Port, Sharing},
        source,
        typing_assist::{self, Filter, Key, TextSize},
        workspace::{self, Action, Task, Work, Workspace},
        xorg,
    },
    engine::State,
};
type Metrics = super::linux_v6_page::Metrics;

fn apply(task: &Task, work: &mut Work, action: Action, notice: &mut Option<String>) {
    if let Err(error) = work.apply(task, action) {
        *notice = Some(error);
    }
}

fn guide(ui: &mut egui::Ui, s: Option<&Scenario>, n: usize) {
    if let Some(s) = s {
        lesson_presentation::practice_step(ui, s, n);
    }
}

fn keyboard_ui(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    guidance: Option<&Scenario>,
    metrics: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Task::Keyboard { devices, .. } = task else {
        return;
    };
    ui.strong("Supplied Xorg session");
    controls::form_row(ui, "Display", |ui, _| {
        lesson_presentation::monospace(ui, "DISPLAY=:0", true)
    });
    for target in devices {
        lesson_presentation::monospace(ui, &target.device.product, true);
    }
    ui.separator();
    guide(ui, guidance, 1);
    let Workspace::Keyboard { draft, .. } = &work.workspace else {
        return;
    };
    ui.strong("Virtual xorg.conf.d fragment");
    let r = shared_libraries_page::configuration_input(
        ui,
        draft,
        id.with("keyboard-editor"),
        xorg::LIMIT,
        ui.is_enabled(),
        "Edit the supplied InputClass rules",
    );
    metrics.record(ui, "keyboard-editor", &r.response);
    if let Some(contents) = r.value {
        apply(task, work, Action::EditKeyboard { contents }, notice);
    }
    if r.rejected {
        ui.label("The edit exceeded the configuration limit. Your earlier draft was kept.");
    }
    let r = ui.add(button("Apply to virtual session"));
    metrics.record(ui, "apply-keyboard", &r);
    if r.clicked() {
        apply(task, work, Action::ApplyKeyboard, notice);
    }
    let Workspace::Keyboard { draft, applied } = &work.workspace else {
        return;
    };
    ui.label(if draft == applied {
        "The virtual session uses this fragment."
    } else {
        "The draft has edits. The virtual session still uses the previous fragment."
    });
    ui.separator();
    guide(ui, guidance, 2);
    ui.strong("Keyboard probe · applied layout");
    let Ok(rules) = xorg::parse(applied) else {
        return;
    };
    let store = id.with("physical-key");
    let mut picked = ui.data(|d| d.get_temp::<usize>(store)).unwrap_or(0);
    ui.horizontal_wrapped(|ui| {
        for (i, (key, _)) in xorg::Layout::Us.letters().iter().enumerate() {
            let r = ui.add(button(&format!("Physical {key}")).selected(picked == i));
            metrics.record(ui, &format!("key-{key}"), &r);
            if r.clicked() {
                picked = i;
            }
        }
    });
    ui.data_mut(|d| d.insert_temp(store, picked));
    for target in devices {
        let actual = xorg::effective(&target.device, &rules);
        let (_, output) = actual.layout.letters()[picked];
        controls::form_row(ui, &target.device.product, |ui, _| {
            lesson_presentation::monospace(
                ui,
                &format!("{} · {output}", actual.layout.name()),
                true,
            );
        });
        if !actual.matched.is_empty() {
            lesson_presentation::prose(
                ui,
                &format!("Matched in order. {}", actual.matched.join(" → ")),
            );
        }
    }
}

fn choose<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    id: egui::Id,
    label: &str,
    state: &mut T,
    choices: &[(T, &str)],
    metrics: &mut Metrics,
    key: &str,
) {
    controls::form_row(ui, label, |ui, width| {
        let current = choices
            .iter()
            .find(|(value, _)| value == state)
            .map_or("Select", |(_, label)| *label);
        let r = controls::dropdown(ui, id.with(key), width)
            .selected_text(current)
            .show_ui(ui, |ui| {
                for (index, (value, name)) in choices.iter().enumerate() {
                    let r = ui.selectable_value(state, *value, *name);
                    metrics.record(ui, &format!("{key}-{index}"), &r);
                }
            });
        metrics.record(ui, key, &r.response);
    });
}

fn remote_ui(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    guidance: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Workspace::Remote { draft: initial } = work.workspace else {
        return;
    };
    let mut draft = initial;
    ui.strong("Supplied GNOME remote desktop profile");
    choose(
        ui,
        id,
        "Desktop Sharing",
        &mut draft.services.sharing,
        &[
            (Sharing::Off, "Off"),
            (Sharing::ViewOnly, "View only"),
            (Sharing::AllowControl, "Allow input control"),
        ],
        m,
        "sharing",
    );
    let r = ui.checkbox(&mut draft.services.remote_login, "Enable Remote Login");
    m.record(ui, "remote-login", &r);
    let port = draft.services.sharing_port();
    ui.label(match port {
        Some(Port::Standard) => "Active sharing endpoint · TCP 3389",
        Some(Port::Secondary) => "Active sharing endpoint · TCP 3390",
        None => "Desktop Sharing is off",
    });
    ui.separator();
    guide(ui, guidance, 1);
    let clients = [
        (Client::RdpReady, "RDP-compatible client"),
        (Client::OtherProtocolOnly, "VNC-only client"),
    ];
    let ports = [(Port::Standard, "3389"), (Port::Secondary, "3390")];
    for (label, route, client_key, port_key) in [
        (
            "Observer · existing demonstration",
            &mut draft.observer,
            "observer-client",
            "observer-port",
        ),
        (
            "Employee · individual login",
            &mut draft.employee,
            "employee-client",
            "employee-port",
        ),
    ] {
        ui.strong(label);
        choose(
            ui,
            id,
            "Supplied client",
            &mut route.client,
            &clients,
            m,
            client_key,
        );
        choose(
            ui,
            id,
            "Connection port",
            &mut route.port,
            &ports,
            m,
            port_key,
        );
    }
    if draft != initial {
        apply(task, work, Action::SetRemote { draft }, notice);
    }
    ui.separator();
    guide(ui, guidance, 2);
    ui.strong("Virtual connection trials");
    for (label, trial) in [
        "Observer with sharing access",
        "Employee with a remote login grant",
        "Unauthorized observer",
    ]
    .into_iter()
    .zip(workspace::remote_trials(draft))
    {
        ui.strong(label);
        lesson_presentation::prose(ui, trial.detail());
    }
}

fn typing_probe(
    ui: &mut egui::Ui,
    settings: typing_assist::Settings,
    title: &str,
    events: &[typing_assist::PhysicalTap],
) {
    use typing_assist::Event::*;
    ui.strong(title);
    if let Some(observations) = typing_assist::preview(settings, events) {
        for (tap, event) in events.iter().zip(observations) {
            let key = match tap.key {
                Key::LeftShift => "Shift",
                Key::A => "A",
                Key::B => "B",
            };
            let label = format!(
                "{key} at {} ms · held {} ms",
                tap.pressed_at_ms,
                tap.released_at_ms - tap.pressed_at_ms
            );
            let description = match event {
                PendingModifier => "Shift applies to the next letter".into(),
                ReleasedModifier => "Shift released".into(),
                Printed(c) => format!("Printed {c}"),
                RejectedTooShort => "Ignored short press".into(),
                RejectedRepeated => "Ignored quick repeat".into(),
            };
            controls::form_row(ui, &label, |ui, _| {
                lesson_presentation::prose(ui, &description);
            });
        }
    }
}

fn typing_ui(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    guidance: Option<&Scenario>,
    metrics: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Workspace::Typing { current: initial } = work.workspace else {
        return;
    };
    let Task::Typing { filter, .. } = task else {
        return;
    };
    let mut current = initial;
    ui.strong("Reading sample settings");
    choose(
        ui,
        id,
        "Text size",
        &mut current.text,
        &[(TextSize::Standard, "Standard"), (TextSize::Large, "Large")],
        metrics,
        "text-size",
    );
    let contrast = ui.checkbox(&mut current.high_contrast, "High contrast");
    metrics.record(ui, "contrast", &contrast);
    let sample = RichText::new("Training appointment · Wednesday, 10:00").size(
        if current.text == TextSize::Large {
            22.
        } else {
            15.
        },
    );
    if current.high_contrast {
        egui::Frame::new()
            .fill(egui::Color32::BLACK)
            .inner_margin(8)
            .show(ui, |ui| {
                lesson_presentation::rich(ui, sample.color(egui::Color32::WHITE));
            });
    } else {
        lesson_presentation::rich(ui, sample);
    }
    ui.separator();
    guide(ui, guidance, 1);
    let sticky = ui.checkbox(&mut current.sticky_keys, "Sticky Keys");
    metrics.record(ui, "sticky", &sticky);
    typing_probe(
        ui,
        current,
        "Separate Shift and two letter presses",
        &workspace::case_probe(),
    );
    ui.separator();
    guide(ui, guidance, 2);
    choose(
        ui,
        id,
        "Typing filter",
        &mut current.typing_filter,
        &[
            (Filter::None, "Off"),
            (Filter::SlowKeys300Ms, "Slow Keys · 300 ms"),
            (Filter::BounceKeys300Ms, "Bounce Keys · 300 ms"),
        ],
        metrics,
        "typing-filter",
    );
    typing_probe(
        ui,
        current,
        "Supplied timing trial",
        &workspace::filter_probe(*filter),
    );
    if current != initial {
        apply(task, work, Action::SetTyping { current }, notice);
    }
}
fn results(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &Work,
    example: bool,
    m: &mut Metrics,
) -> bool {
    let outcomes = work.checks(task);
    let passed = outcomes.iter().filter(|outcome| outcome.passed).count();
    let solved = passed == outcomes.len();
    let headline = ui.strong(if !work.checked {
        "Not checked".into()
    } else if solved && example {
        "Example verified".into()
    } else if solved {
        "Assignment complete".into()
    } else {
        format!("Needs work · {passed}/{} requirements met", outcomes.len())
    });
    m.record(ui, "result", &headline);
    m.result = Some((passed, outcomes.len(), headline.rect));
    if !work.checked {
        return false;
    }
    for outcome in outcomes.iter().filter(|o| !o.passed) {
        ui.strong(format!("Needs work · {}", outcome.label));
        lesson_presentation::prose(ui, &outcome.detail);
    }
    controls::collapsing_header(
        ui,
        if solved {
            "Objective results"
        } else {
            "Requirements already met"
        },
    )
    .id_salt(id.with("details"))
    .show(ui, |ui| {
        for outcome in outcomes.iter().filter(|o| o.passed) {
            ui.strong(format!("Passed · {}", outcome.label));
            lesson_presentation::prose(ui, &outcome.detail);
        }
    });
    solved
}

impl LabApp {
    pub(super) fn desktop_profile_lesson(&mut self, ui: &mut egui::Ui, index: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, index);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let scenario = self.scenarios[index].clone();
        let original = &self.progress.attempts[&scenario.id];
        let definition = match source::source(&scenario) {
            Ok(d) => d,
            Err(error) => {
                lesson_presentation::prose(ui, &error);
                return;
            }
        };
        if let Err(error) = source::context(original, &scenario) {
            lesson_presentation::prose(ui, &error);
            return;
        }
        let State::Desktop { simulation } = &original.state else {
            return;
        };
        let mut next = (**simulation).clone();
        let before = next.clone();
        let active = ui.is_enabled()
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let mut view = if challenge {
            teaching::View::Assignment
        } else {
            next.view
        };
        let metrics = &mut self.metrics.desktop_profile;
        lesson_presentation::rich(ui, RichText::new(&scenario.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (candidate, name, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == candidate));
                    metrics.record(ui, name, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = candidate;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
            next.view = view;
        }
        if view == teaching::View::Introduction {
            let intro =
                lesson_presentation::concepts(ui, &scenario, "", Vec::<(String, String)>::new());
            metrics.record(ui, "introduction-content", &intro.objective);
            lesson_presentation::worked(ui, "Example using a different workstation");
            let r = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &scenario));
            metrics.record(ui, "worked", &r.response);
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            metrics.record(ui, "try-example", &r);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            let practice = view == teaching::View::Example;
            let (task, work) = if practice {
                (&definition.example, &mut next.example.work)
            } else {
                (&definition.assignment, &mut next.assignment.work)
            };
            let id = egui::Id::new(("desktop-profile", &scenario.id, practice));
            if practice {
                lesson_presentation::practice_step(ui, &scenario, 0);
            } else {
                let assignment = lesson_presentation::assignment_responses(ui, &scenario);
                metrics.record(ui, "assignment-content", &assignment.goal);
            }
            let mut reset = false;
            ui.add_enabled_ui(active, |ui| {
                let guidance = practice.then_some(&scenario);
                let workspace = match task {
                    Task::Keyboard { .. } => keyboard_ui,
                    Task::Remote { .. } => remote_ui,
                    Task::Typing { .. } => typing_ui,
                };
                workspace(ui, id, task, work, guidance, metrics, &mut self.notice);
                ui.separator();
                let check = ui.add(primary(ui, "Check workspace"));
                metrics.record(ui, "check", &check);
                if check.clicked() {
                    apply(task, work, Action::Check, &mut self.notice);
                }
                if results(ui, id, task, work, practice, metrics) && !practice {
                    lesson_presentation::prose(ui, &scenario.explanation);
                }
                if practice {
                    let r = ui.add(button("Reset example"));
                    metrics.record(ui, "reset-example", &r);
                    reset = r.clicked();
                }
            });
            if reset && let Err(error) = next.reset_example(definition) {
                self.notice = Some(error);
            }
            controls::collapsing_header(ui, "Supported operations, hints and references")
                .id_salt(id.with("help"))
                .show(ui, |ui| {
                    let subset = match task {
                    Task::Keyboard { .. } => "Edit InputClass sections with Identifier, MatchProduct, MatchIsKeyboard and Option \"XkbLayout\". Use quoted values and choose us, de or fr.",
                    Task::Remote { .. } => "Select a supplied client and port for each role. The profile models Desktop Sharing and Remote Login with fixed access grants.",
                    Task::Typing { .. } => "The samples show one Sticky Keys modifier and two timing filters. The settings only change this virtual desk.",
                };
                lesson_presentation::prose(ui, subset);
                if ui.add_enabled(active, button("Show a hint")).clicked() {
                        let a = self.progress.attempts.get_mut(&scenario.id).unwrap();
                        a.hints = (a.hints + 1).min(scenario.hints.len());
                        self.dirty = true;
                    }
                    for hint in scenario
                        .hints
                        .iter()
                        .take(self.progress.attempts[&scenario.id].hints)
                    {
                        lesson_presentation::prose(ui, hint);
                    }
                    lesson_presentation::prose(ui, &scenario.simplifications);
                    for url in &scenario.sources {
                        ui.hyperlink_to(url, url);
                    }
                });
        }
        if active && next != before {
            match next.bind(definition) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&scenario.id).unwrap().state = State::Desktop {
                        simulation: Box::new(next),
                    };
                    self.completion_index.forget(&scenario.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(error) => self.notice = Some(error),
            }
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&scenario.id], &scenario);
        if self.metrics.completion {
            self.next_unit_lab(ui, index);
        }
        Self::workspace_footer(ui);
    }
}
