//! Native represented-host and VRF configuration; no real virtualization/runtime.
use super::*;
use dlms_learning_lab::{
    virtual_isolation_source as binding, virtual_isolation_workspace as work,
    virtual_workloads as hosts,
};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub focus: Vec<(String, egui::Id)>,
    pub cards: Vec<egui::Rect>,
    pub results: Option<egui::Rect>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
        self.focus.push((key.into(), r.id));
    }
}
fn choice(
    ui: &mut egui::Ui,
    key: &str,
    value: &mut String,
    options: &[(String, String)],
    width: f32,
    m: &mut Metrics,
) {
    let selected = options
        .iter()
        .find(|(id, _)| id == value)
        .map_or("Choose a supplied value", |(_, label)| label.as_str());
    let r = controls::dropdown(ui, key, width)
        .selected_text(selected)
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            let start = m.controls.len();
            for (id, label) in options {
                let r = ui.selectable_value(value, id.clone(), label);
                m.record(ui, &format!("{key}:{id}"), &r);
            }
            let clip = controls::popup_content_clip(ui);
            for r in &mut m.controls[start..] {
                r.2 = clip;
            }
        });
    controls::retain_dropdown_focus(ui, &r.response);
    m.record(ui, key, &r.response);
}
fn workload_cards(ui: &mut egui::Ui, f: &hosts::Facts, c: &hosts::Configuration, m: &mut Metrics) {
    let line = ui.text_style_height(&egui::TextStyle::Body);
    let gap = ui.spacing().item_spacing.x;
    let cols = ((ui.available_width() + gap) / (line * 15. + gap))
        .floor()
        .clamp(1., 3.) as usize;
    let width = ((ui.available_width() - gap * (cols - 1) as f32) / cols as f32).max(1.);
    let inner = (width - 2. * controls::PAD_X as f32).max(1.);
    let texts:Vec<_>=f.hosts.iter().map(|h|{
  let workloads=c.placements.iter().filter(|p|p.host==h.id).map(|p|format!("{}{}",p.workload,if p.up{""}else{" (off)"})).collect::<Vec<_>>();
  format!("{}\n{} host kernel\nHypervisor: {} · Linux runtime: {}\nBudget: {} MiB / {} CPU units\nPlaced here: {}",h.id,h.kernel.label(),if h.hypervisor{"available"}else{"not supplied"},if h.container_runtime{"available"}else{"not supplied"},h.memory_mib,h.cpu_units,if workloads.is_empty(){"none".into()}else{workloads.join(", ")})
 }).collect();
    let h = texts
        .iter()
        .map(|s| {
            ui.painter()
                .layout(
                    s.clone(),
                    egui::TextStyle::Body.resolve(ui.style()),
                    palette(ui).text,
                    inner,
                )
                .size()
                .y
        })
        .fold(line, f32::max);
    for row in texts.chunks(cols) {
        ui.horizontal(|ui| {
            for text in row {
                let r = controls::topology_frame(ui).show(ui, |ui| {
                    ui.set_width(inner);
                    ui.set_min_height(h);
                    ui.add(egui::Label::new(text).wrap());
                });
                m.cards.push(r.response.rect);
            }
        });
        ui.add_space(gap);
    }
}
fn configuration(
    ui: &mut egui::Ui,
    t: &work::Task,
    a: &mut work::Attempt,
    m: &mut Metrics,
) -> Result<(), String> {
    let mut draft = a.configuration.clone();
    match (&t.exercise, &mut draft) {
        (work::Exercise::Workloads { facts, .. }, work::Draft::Workloads(c)) => {
            workload_cards(ui, facts, c, m);
            ui.strong("Workload placement");
            let options = facts
                .hosts
                .iter()
                .map(|h| (h.id.clone(), h.id.clone()))
                .collect::<Vec<_>>();
            for w in &facts.workloads {
                ui.push_id(&w.id, |ui| {
                    let p = c
                        .placements
                        .iter_mut()
                        .find(|p| p.workload == w.id)
                        .unwrap();
                    let environment = match w.environment {
                        hosts::Environment::VirtualMachine { kernel } => {
                            format!("Virtual machine · own {} kernel", kernel.label())
                        }
                        hosts::Environment::LinuxContainer => {
                            "Linux container · shares host kernel".into()
                        }
                    };
                    controls::form_row(ui, &w.id, |ui, width| {
                        choice(
                            ui,
                            &format!("placement:{}", w.id),
                            &mut p.host,
                            &options,
                            width,
                            m,
                        );
                    });
                    lesson_presentation::prose(
                        ui,
                        &format!(
                            "{environment} · {} MiB / {} CPU units · service group: {}",
                            w.memory_mib, w.cpu_units, w.service
                        ),
                    );
                    let r = ui.checkbox(&mut p.up, "Workload switched on");
                    m.record(ui, &format!("up:{}", w.id), &r);
                    ui.add_space(ui.spacing().item_spacing.y);
                });
            }
        }
        (work::Exercise::Routing { facts, .. }, work::Draft::Routing(c)) => {
            ui.strong(format!("{} · one physical router", facts.router));
            lesson_presentation::prose(
                ui,
                "Each interface uses one routing context. Supplied addresses stay fixed; each context derives its own connected routes.",
            );
            let r = ui.checkbox(&mut c.router_up, "Router available");
            m.record(ui, "router-up", &r);
            let options = facts
                .contexts
                .iter()
                .map(|v| (v.id.clone(), v.id.clone()))
                .collect::<Vec<_>>();
            for b in &mut c.bindings {
                ui.push_id(&b.interface, |ui| {
                    let i = &facts.network.devices[&facts.router].interfaces[&b.interface];
                    let ip = i.address.as_ref().unwrap();
                    controls::form_row(ui, &b.interface, |ui, width| {
                        choice(
                            ui,
                            &format!("context:{}", b.interface),
                            &mut b.context,
                            &options,
                            width,
                            m,
                        );
                    });
                    for (id, device) in &facts.network.devices {
                        if id == &facts.router {
                            continue;
                        }
                        for peer in device
                            .interfaces
                            .values()
                            .filter(|peer| peer.link == i.link)
                        {
                            if let Some(address) = &peer.address {
                                lesson_presentation::monospace(
                                    ui,
                                    &format!(
                                        "Router {}/{} ↔ {id} {}/{}",
                                        ip.address, ip.prefix, address.address, address.prefix
                                    ),
                                    true,
                                );
                            }
                        }
                    }
                });
            }
            controls::collapsing_header(ui, "Inspect supplied endpoints and connected tables")
                .show(ui, |ui| {
                    for (id, d) in &facts.network.devices {
                        if id == &facts.router {
                            continue;
                        }
                        for i in d.interfaces.values() {
                            if let Some(ip) = &i.address {
                                lesson_presentation::monospace(
                                    ui,
                                    &format!(
                                        "{id} · {}/{} · link {}",
                                        ip.address, ip.prefix, i.link
                                    ),
                                    true,
                                );
                            }
                        }
                    }
                    match facts.project(c) {
                        Ok((network, instances)) => {
                            for (id, (router, context)) in instances {
                                ui.strong(format!("{router} [{context}]"));
                                for i in network.devices[&id].interfaces.values() {
                                    if let Some(a) = &i.address {
                                        let prefix = dlms_learning_lab::ipv4_routing::Prefix::new(
                                            a.address, a.prefix,
                                        )
                                        .unwrap();
                                        lesson_presentation::monospace(
                                            ui,
                                            &format!(
                                                "Connected {} · link {}",
                                                prefix.text(),
                                                i.link
                                            ),
                                            true,
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            lesson_presentation::prose(ui, &e);
                        }
                    }
                });
        }
        _ => return Err("Configuration belongs to another workspace.".into()),
    }
    a.edit(t, draft)
}
fn trial(
    ui: &mut egui::Ui,
    t: &work::Task,
    a: &mut work::Attempt,
    m: &mut Metrics,
) -> Result<(), String> {
    ui.strong("Try the current configuration");
    let mut selection = a.selection.clone();
    match (&t.exercise, &mut selection) {
        (work::Exercise::Workloads { facts, .. }, work::Selection::Workloads { experiment }) => {
            let mut selected = match experiment {
                hosts::Experiment::Normal => "normal".into(),
                hosts::Experiment::HostOutage { host } => format!("outage/{host}"),
            };
            let mut options = vec![("normal".into(), "All supplied hosts available".into())];
            options.extend(
                facts
                    .hosts
                    .iter()
                    .map(|h| (format!("outage/{}", h.id), format!("{} unavailable", h.id))),
            );
            controls::form_row(ui, "Experiment", |ui, width| {
                choice(ui, "experiment", &mut selected, &options, width, m)
            });
            *experiment = if selected == "normal" {
                hosts::Experiment::Normal
            } else {
                hosts::Experiment::HostOutage {
                    host: selected.strip_prefix("outage/").unwrap().into(),
                }
            };
        }
        (work::Exercise::Routing { facts, .. }, work::Selection::Routing { from, to }) => {
            let options = facts
                .network
                .devices
                .keys()
                .filter(|id| *id != &facts.router)
                .map(|id| (id.clone(), id.clone()))
                .collect::<Vec<_>>();
            controls::form_row(ui, "From endpoint", |ui, width| {
                choice(ui, "from", from, &options, width, m)
            });
            controls::form_row(ui, "Intended endpoint", |ui, width| {
                choice(ui, "to", to, &options, width, m)
            });
        }
        _ => return Err("Trial belongs to another workspace.".into()),
    }
    a.choose(t, selection)?;
    let r = ui.add(button("Run experiment"));
    m.record(ui, "run", &r);
    if r.clicked() {
        let _ = a.run(t);
    }
    let heading = ui.strong("Actual observation");
    m.record(ui, "trial-title", &heading);
    if let Some(lines) = &a.actual {
        for line in lines {
            lesson_presentation::prose(ui, line);
        }
    } else {
        lesson_presentation::prose(
            ui,
            "Not run for these settings. Select Run experiment to observe them.",
        );
    }
    Ok(())
}
fn results(
    ui: &mut egui::Ui,
    t: &work::Task,
    a: &work::Attempt,
    s: &Scenario,
    practice: bool,
    m: &mut Metrics,
) {
    if !a.checked {
        let r = lesson_presentation::prose(
            ui,
            "Unchecked · Check answers verifies the complete task, including required separation and failure cases.",
        );
        m.record(ui, "unchecked", &r);
        return;
    }
    let checks = t.checks(&a.configuration);
    let passed = checks.iter().filter(|c| c.passed).count();
    let ok = passed == checks.len();
    let r = ui.strong(if ok {
        if practice {
            "Example checks pass"
        } else {
            "Assignment complete"
        }
    } else {
        "Configuration needs attention"
    });
    m.results = Some(r.rect);
    m.record(ui, "result-title", &r);
    lesson_presentation::prose(
        ui,
        &format!("{passed}/{} required outcomes pass.", checks.len()),
    );
    for c in checks.iter().filter(|c| !c.passed) {
        let r =
            lesson_presentation::prose(ui, &format!("Needs attention · {}\n{}", c.label, c.detail));
        m.record(ui, "failed-case", &r);
    }
    controls::collapsing_header(ui, "Inspect passing checks").show(ui, |ui| {
        for c in checks.iter().filter(|c| c.passed) {
            lesson_presentation::prose(ui, &format!("Passed · {}\n{}", c.label, c.detail));
        }
    });
    if ok {
        lesson_presentation::prose(ui, &s.explanation);
    }
}
impl LabApp {
    pub(super) fn virtual_isolation_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let s = self.scenarios[i].clone();
        let original = &self.progress.attempts[&s.id];
        if let Err(e) = binding::context(original, &s) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (
            engine::Goal::VirtualIsolation { definition },
            engine::State::VirtualIsolation { simulation },
        ) = (&s.goal, &original.state)
        else {
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
        lesson_presentation::rich(ui, RichText::new(&s.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    self.metrics.virtual_isolation.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
            next.view = view;
        }
        let practice = view == teaching::View::Example;
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(ui, &s, "", Vec::<(String, String)>::new());
            lesson_presentation::worked(
                ui,
                "Separate guide equipment · read the example, then try its configuration.",
            );
            let worked = ui.scope(|ui| lesson_presentation::worked_rows(ui, &s));
            self.metrics
                .virtual_isolation
                .record(ui, "worked-table", &worked.response);
            let r = ui.add_enabled(active, primary(ui, "Try this example"));
            self.metrics.virtual_isolation.record(ui, "example", &r);
            self.metrics.example_action = Some(r.rect);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            let t = if practice {
                &definition.example
            } else {
                &definition.assignment
            };
            let a = if practice {
                &mut next.practice
            } else {
                &mut next.assignment
            };
            if practice {
                lesson_presentation::practice_step(ui, &s, 0);
            } else {
                lesson_presentation::assignment(ui, &s);
            }
            ui.push_id(("virtual-network-work", &s.id, practice), |ui| {
                ui.add_enabled_ui(active, |ui| {
                    if let Err(e) = configuration(ui, t, a, &mut self.metrics.virtual_isolation) {
                        self.notice = Some(e);
                    }
                    if practice {
                        lesson_presentation::practice_step(ui, &s, 1);
                    }
                    if let Err(e) = trial(ui, t, a, &mut self.metrics.virtual_isolation) {
                        self.notice = Some(e);
                    }
                    if practice {
                        lesson_presentation::practice_step(ui, &s, 2);
                    }
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        let r = ui.add(primary(ui, "Check answers"));
                        self.metrics.check = Some(r.rect);
                        self.metrics.virtual_isolation.record(ui, "check", &r);
                        if r.clicked()
                            && let Err(e) = a.check(t)
                        {
                            self.notice = Some(e);
                        }
                        if practice {
                            let r = ui.add(button("Reset example only"));
                            self.metrics
                                .virtual_isolation
                                .record(ui, "reset-example", &r);
                            if r.clicked() {
                                match work::Attempt::new(t) {
                                    Ok(reset) => *a = reset,
                                    Err(e) => self.notice = Some(e),
                                }
                            }
                        }
                    });
                    results(ui, t, a, &s, practice, &mut self.metrics.virtual_isolation);
                });
            });
            ui.push_id(("virtual-network-help", &s.id, practice, challenge), |ui| {
                let r = controls::collapsing_header(ui, "Optional help and simulation scope").show(
                    ui,
                    |ui| {
                        if !practice {
                            let a = self.progress.attempts.get_mut(&s.id).unwrap();
                            if ui
                                .add_enabled(
                                    active && a.hints < s.hints.len(),
                                    button("Show a hint"),
                                )
                                .clicked()
                            {
                                a.hints += 1;
                                self.dirty = true;
                            }
                            for hint in s.hints.iter().take(a.hints) {
                                lesson_presentation::prose(ui, hint);
                            }
                            controls::collapsing_header(ui, "Worked approach · reveals reasoning")
                                .show(ui, |ui| {
                                    lesson_presentation::prose(ui, &s.explanation);
                                });
                        }
                        lesson_presentation::prose(ui, binding::HELP);
                        lesson_presentation::prose(ui, &s.simplifications);
                        for source in &s.sources {
                            ui.hyperlink_to(source, source);
                        }
                    },
                );
                self.metrics
                    .virtual_isolation
                    .record(ui, "help", &r.header_response);
            });
        }
        if active && next != before {
            match definition.context(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&s.id).unwrap().state =
                        engine::State::VirtualIsolation {
                            simulation: Box::new(next),
                        };
                    self.completion_index.forget(&s.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
            }
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&s.id], &s);
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
    }
}
