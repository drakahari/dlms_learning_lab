//! Native saved Ethernet profile editing over bounded startup projections.
use super::*;
use dlms_learning_lab::nm_workspace as work;
use std::sync::Arc;
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub running: bool,
    pub editor_scroll: Option<(f32, f32)>,
}
pub(super) struct ActiveRun {
    pub scenario: String,
    pub practice: bool,
    draft: work::Files,
    run: work::Run,
}
#[derive(Clone)]
struct Report {
    task: work::Task,
    draft: work::Files,
    rows: Vec<work::CaseResult>,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn bridge_facts(ui: &mut egui::Ui, b: &dlms_learning_lab::bridge_host::Facts, details: bool) {
    use dlms_learning_lab::switching::Jack;
    lesson_presentation::rich(ui, RichText::new("Supplied untagged bridge links").strong());
    let sides = b
        .lan
        .ports
        .iter()
        .map(|p| {
            let jack = Jack::Host {
                host: p.endpoint.clone(),
            };
            let attachment = b.lan.fabric.cables.iter().find_map(|c| {
                let other = if c.a == jack {
                    &c.b
                } else if c.b == jack {
                    &c.a
                } else {
                    return None;
                };
                let Jack::Switch { switch, port } = other else {
                    return None;
                };
                let sw = b.lan.fabric.switches.get(switch)?;
                let remote = sw.ports.get(port)?;
                Some((
                    sw.label.clone(),
                    c.up && remote.up && b.lan.fabric.hosts.get(&p.endpoint)?.up,
                ))
            });
            (
                attachment
                    .as_ref()
                    .map(|a| a.0.clone())
                    .unwrap_or_else(|| "No supplied segment".into()),
                p.interface.0.clone(),
                if attachment.is_some_and(|a| a.1) {
                    "Carrier available"
                } else {
                    "Physical path unavailable"
                },
            )
        })
        .collect::<Vec<_>>();
    if sides.len() == 2 {
        controls::topology_chain(
            ui,
            &[
                [&sides[0].0, &sides[0].1, sides[0].2],
                [
                    "Software bridge",
                    &b.profiles.controller.interface,
                    "Ports checked by tests",
                ],
                [&sides[1].0, &sides[1].1, sides[1].2],
            ],
            "Supplied physical connections; actual current and future membership appears after testing.",
        );
    }
    if !details {
        return;
    }
    lesson_presentation::monospace(
        ui,
        &format!(
            "{} · controller file {}",
            b.profiles.controller.interface, b.profiles.controller.file
        ),
        true,
    );
    for p in &b.profiles.members {
        lesson_presentation::monospace(
            ui,
            &format!("{} · port file {}", p.interface, p.file),
            true,
        );
    }
    lesson_presentation::rich(
        ui,
        RichText::new("Provided LAN peers · keep the bridge address distinct").strong(),
    );
    for p in &b.peers {
        let label = b
            .lan
            .fabric
            .hosts
            .get(&p.endpoint)
            .map(|h| h.label.as_str())
            .unwrap_or(&p.endpoint);
        lesson_presentation::monospace(ui, &format!("{} · {label}", p.address), true);
    }
}
fn facts(ui: &mut egui::Ui, task: &work::Task, selected: &mut usize, m: &mut Metrics) {
    lesson_presentation::rich(
        ui,
        RichText::new("Supplied inputs · before each test").strong(),
    );
    let r = controls::dropdown(ui, "nm-input-case", ui.available_width())
        .selected_text(&task.cases[*selected].label)
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            let start = m.controls.len();
            for (n, c) in task.cases.iter().enumerate() {
                let r = ui.selectable_value(selected, n, &c.label);
                record(m, &format!("case:{n}"), &r, ui);
            }
            let clip = controls::popup_content_clip(ui);
            for c in &mut m.controls[start..] {
                c.2 = clip;
            }
        });
    record(m, "case", &r.response, ui);
    let c = &task.cases[*selected];
    if let Some(bridge) = &c.environment.bridge {
        bridge_facts(ui, bridge, true);
    }
    if let Some(bond) = &c.environment.bond {
        lesson_presentation::rich(
            ui,
            RichText::new("Supplied bond and physical candidates").strong(),
        );
        lesson_presentation::monospace(
            ui,
            &format!(
                "{} · controller file {}",
                bond.profiles.controller.interface, bond.profiles.controller.file
            ),
            true,
        );
        for member in &bond.profiles.members {
            lesson_presentation::monospace(
                ui,
                &format!("{} · port file {}", member.interface, member.file),
                true,
            );
        }
    }
    if let Some(vlan) = &c.environment.vlan {
        lesson_presentation::rich(ui, RichText::new("Supplied VLAN attachments").strong());
        for parent in &vlan.parents {
            let endpoint = dlms_learning_lab::switching::Jack::Host {
                host: parent.endpoint.clone(),
            };
            let cable = vlan
                .fabric
                .cables
                .iter()
                .find(|c| c.a == endpoint || c.b == endpoint);
            if let Some(cable) = cable {
                let other = if cable.a == endpoint {
                    &cable.b
                } else {
                    &cable.a
                };
                if let dlms_learning_lab::switching::Jack::Switch { switch, port } = other {
                    let p = &vlan.fabric.switches[switch].ports[port];
                    if let dlms_learning_lab::switching::Mode::Trunk { native, allowed } = &p.mode {
                        lesson_presentation::monospace(
                            ui,
                            &format!(
                                "{} → {switch}/{port}\nnative {native} · allowed {} · {}",
                                parent.device,
                                allowed
                                    .iter()
                                    .map(u16::to_string)
                                    .collect::<Vec<_>>()
                                    .join(", "),
                                if cable.up && p.up && vlan.fabric.hosts[&parent.endpoint].up {
                                    "carrier available"
                                } else {
                                    "physical path unavailable"
                                }
                            ),
                            true,
                        );
                    }
                }
            }
        }
        for peer in &vlan.peers {
            let endpoint = dlms_learning_lab::switching::Jack::Host {
                host: peer.endpoint.clone(),
            };
            if let Some(cable) = vlan
                .fabric
                .cables
                .iter()
                .find(|c| c.a == endpoint || c.b == endpoint)
            {
                let other = if cable.a == endpoint {
                    &cable.b
                } else {
                    &cable.a
                };
                if let dlms_learning_lab::switching::Jack::Switch { switch, port } = other
                    && let dlms_learning_lab::switching::Mode::Access { vlan } =
                        &vlan.fabric.switches[switch].ports[port].mode
                {
                    lesson_presentation::monospace(
                        ui,
                        &format!("{} · {} · access VLAN{vlan}", peer.address, peer.endpoint),
                        true,
                    );
                }
            }
        }
    }
    for link in &c.environment.physical.links {
        lesson_presentation::rich(
            ui,
            RichText::new(format!(
                "{} · {}/{}",
                link.name, link.network.address, link.network.length
            ))
            .strong(),
        );
        lesson_presentation::prose(
            ui,
            &format!(
                "Automatic route metric: {}",
                c.environment.default_metrics[&link.name]
            ),
        );
    }
    for (device, outcome) in &c.environment.dhcp {
        let value = match outcome {
            dlms_learning_lab::nm_dhcp::Outcome::Bound { address, router } => format!(
                "Supplied DHCP for {device}: {}/{} · router {}",
                address.address,
                address.prefix,
                router
                    .map(|r| r.to_string())
                    .unwrap_or_else(|| "none".into())
            ),
            dlms_learning_lab::nm_dhcp::Outcome::Unavailable {} => {
                format!("Supplied DHCP for {device}: unavailable; no address supplied")
            }
        };
        lesson_presentation::monospace(ui, &value, true);
    }
    controls::collapsing_header(ui, "Supplied gateways and destinations")
        .id_salt("nm-physical-facts")
        .show(ui, |ui| {
            for g in &c.environment.physical.gateways {
                lesson_presentation::monospace(
                    ui,
                    &format!("{} via {}", g.address, g.device),
                    true,
                );
            }
            for n in &c.environment.physical.nodes {
                lesson_presentation::monospace(
                    ui,
                    &format!("Virtual destination: {}", n.address),
                    true,
                );
            }
        });
}
fn report(
    ui: &mut egui::Ui,
    rows: &[work::CaseResult],
    task: &work::Task,
    selected: usize,
    m: &mut Metrics,
) {
    let passed = rows.iter().filter(|r| r.passed()).count();
    let color = if passed == rows.len() {
        palette(ui).success
    } else {
        palette(ui).error
    };
    let r = lesson_presentation::rich(
        ui,
        RichText::new(format!(
            "{} · {passed}/{} startup cases pass",
            if passed == rows.len() {
                "Profiles meet every requirement"
            } else {
                "Profiles need work"
            },
            rows.len()
        ))
        .color(color)
        .strong(),
    );
    m.result = Some((passed, rows.len(), r.rect));
    for row in rows {
        lesson_presentation::rich(
            ui,
            RichText::new(format!(
                "{} · {}",
                if row.passed() { "Passed" } else { "Needs work" },
                row.label
            ))
            .strong(),
        );
        if let Some((_, text)) = row.checks.iter().find(|r| !r.0) {
            lesson_presentation::prose(ui, text);
        }
        controls::collapsing_header(ui, "Objective results")
            .id_salt(("nm-objectives", &row.id))
            .show(ui, |ui| {
                for (ok, text) in &row.checks {
                    lesson_presentation::prose(
                        ui,
                        &format!("{} · {text}", if *ok { "Pass" } else { "Needs work" }),
                    );
                }
            });
    }
    let row = &rows[selected];
    for (title, choices) in [
        (
            "Actual current settings · after re-reading files",
            &row.current,
        ),
        (
            "Actual next startup · separate fresh selection",
            &row.future,
        ),
    ] {
        if let Some(choices) = choices {
            ui.separator();
            lesson_presentation::rich(ui, RichText::new(title).strong());
            if let (Some(snapshot), Some(bond), Some(vlan)) = (
                choices.controls(),
                task.cases[selected].environment.bond.as_ref(),
                task.cases[selected].environment.vlan.as_ref(),
            ) {
                match bond.selection(vlan, snapshot) {
                    Ok(state) => {
                        lesson_presentation::monospace(
                            ui,
                            &format!(
                                "{} · {} · active member {}\nAttached NICs: {}",
                                snapshot.plan.controller.name,
                                if state.controller {
                                    "Selected"
                                } else {
                                    "Not selected"
                                },
                                state.active.as_deref().unwrap_or("none"),
                                if state.members.is_empty() {
                                    "none".into()
                                } else {
                                    state.members.into_iter().collect::<Vec<_>>().join(", ")
                                }
                            ),
                            true,
                        );
                    }
                    Err(e) => {
                        lesson_presentation::prose(ui, &e.to_string());
                    }
                }
            }
            if let Some(snapshot) = choices.bridge_controls() {
                lesson_presentation::monospace(
                    ui,
                    &format!(
                        "{} · {} · STP {} (settled tree only)\nAttached ports: {}",
                        snapshot.plan.source().controller.interface,
                        if snapshot.current.controller {
                            "Selected"
                        } else {
                            "Not selected"
                        },
                        if snapshot.plan.controller().stp {
                            "Enabled"
                        } else {
                            "Disabled"
                        },
                        if snapshot.current.members.is_empty() {
                            "none".into()
                        } else {
                            snapshot
                                .current
                                .members
                                .iter()
                                .map(|m| m.0.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    ),
                    true,
                );
            }
            for (device, p) in choices.selected() {
                if let Some(vlan) = &p.vlan {
                    lesson_presentation::monospace(
                        ui,
                        &format!("{device} ← {} · VLAN{}", vlan.parent, vlan.id),
                        true,
                    );
                }
                if choices.uses_dhcp(device) {
                    lesson_presentation::prose(
                        ui,
                        "Address and router received from this test's supplied DHCP outcome.",
                    );
                }
                lesson_presentation::monospace(
                    ui,
                    &format!(
                        "{device}: {}\n{}/{} · gateway {}",
                        p.id,
                        p.address.address,
                        p.address.prefix,
                        p.gateway
                            .map(|x| x.to_string())
                            .unwrap_or_else(|| "none".into())
                    ),
                    true,
                );
            }
            if choices.selected().is_empty() {
                lesson_presentation::prose(ui, "No profile selected.");
            }
        }
    }
    if let Some(choices) = &row.future {
        controls::collapsing_header(ui, "Actual next-startup routes")
            .id_salt("nm-actual-routes")
            .show(ui, |ui| {
                match choices.route_table(&task.cases[selected].environment) {
                    Err(e) => {
                        lesson_presentation::prose(ui, &e.to_string());
                    }
                    Ok(routes) => {
                        for r in routes {
                            lesson_presentation::monospace(
                                ui,
                                &format!(
                                    "{}/{} via {} dev {} metric {}",
                                    r.destination.address,
                                    r.destination.length,
                                    r.via
                                        .map(|x| x.to_string())
                                        .unwrap_or_else(|| "direct".into()),
                                    r.device,
                                    r.metric
                                ),
                                true,
                            );
                        }
                    }
                }
            });
    }
    controls::collapsing_header(ui, "Selection details · profile identities")
        .id_salt("nm-decisions")
        .show(ui, |ui| {
            for decision in &row.decisions {
                lesson_presentation::prose(
                    ui,
                    &format!("{} · {}", decision.device, decision.reason),
                );
            }
            if let Some(choices) = &row.future {
                for (device, p) in choices.selected() {
                    lesson_presentation::monospace(ui, &format!("{device}: {}", p.uuid), true);
                }
            }
        });
}
struct EditContext<'a> {
    source: &'a Scenario,
    definition: &'a work::Definition,
    visited: u64,
    practice: bool,
    active: bool,
    running: bool,
}
impl EditContext<'_> {
    fn editor(
        &self,
        ui: &mut egui::Ui,
        scope: &mut work::Scope,
        m: &mut Metrics,
    ) -> Result<(bool, bool), String> {
        let Self {
            source,
            definition,
            visited,
            practice,
            active,
            running,
        } = *self;
        let mut start = false;
        let mut stop = false;
        lesson_presentation::rich(
            ui,
            RichText::new("Saved profile editor · virtual files").strong(),
        );
        let r = controls::dropdown(ui, "nm-file", ui.available_width())
            .selected_text(&scope.selected)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                let start = m.controls.len();
                for name in scope.draft.keys() {
                    let r = ui.selectable_value(&mut scope.selected, name.clone(), name);
                    record(m, &format!("file:{name}"), &r, ui);
                }
                let clip = controls::popup_content_clip(ui);
                for c in &mut m.controls[start..] {
                    c.2 = clip;
                }
            });
        record(m, "file", &r.response, ui);
        if practice {
            let r = ui.add_enabled(active && !running, button("Insert worked profiles"));
            record(m, "insert", &r, ui);
            if r.clicked() {
                scope
                    .replace(&definition.example_worked)
                    .expect("validated example");
            }
        }
        let selected_file = scope.selected.clone();
        let mut draft = scope.draft[&selected_file].clone();
        let editor_height = ui.text_style_height(&egui::TextStyle::Monospace) * 10.
            + 2. * f32::from(controls::PAD_Y);
        let mut visible_editor = egui::Rect::NOTHING;
        let area = controls::nested_scroll(
            ui,
            egui::ScrollArea::vertical()
                .id_salt((
                    "nm-editor-scroll",
                    &source.id,
                    visited,
                    practice,
                    &selected_file,
                ))
                .max_height(editor_height)
                .min_scrolled_height(editor_height)
                .auto_shrink([false, false]),
            |ui| {
                visible_editor = ui.clip_rect();
                ui.add_enabled(
                    active && !running,
                    egui::TextEdit::multiline(&mut draft)
                        .code_editor()
                        .id(egui::Id::new((
                            "nm-config-editor",
                            &source.id,
                            visited,
                            practice,
                            &selected_file,
                        )))
                        .desired_width(f32::INFINITY)
                        .desired_rows(9)
                        .char_limit(16384),
                )
            },
        );
        m.editor_scroll = Some((area.inner_rect.height(), area.state.offset.y));
        let r = area.inner;
        name_control(
            &r,
            if definition.initial.cases[0].environment.bridge.is_some() {
                "Simulated saved bridge controller or port profile"
            } else if definition.initial.cases[0].environment.bond.is_some() {
                "Simulated saved bond, member or VLAN profile"
            } else if definition.initial.cases[0].environment.vlan.is_some() {
                "Simulated saved VLAN profile"
            } else {
                "Simulated saved Ethernet profile"
            },
        );
        m.editor = Some((r.rect.intersect(visible_editor), r.id));
        let shortcut = r.has_focus()
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter));
        if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            r.surrender_focus();
        }
        if r.changed() {
            workspace::bound(&mut draft, 16384);
            scope.edit(&selected_file, &draft)?;
        }
        lesson_presentation::rich(
            ui,
            RichText::new("Tab indents · Ctrl+Enter runs · Esc releases focus").small(),
        );
        if practice {
            lesson_presentation::practice_step(ui, source, 1);
        }
        ui.horizontal_wrapped(|ui| {
            let r = ui.add_enabled(active && !running, primary(ui, "Test startup profiles"));
            record(m, "run", &r, ui);
            start = r.clicked() || shortcut;

            let r = ui.add_enabled(active && running, button("Stop"));
            record(m, "stop", &r, ui);
            stop = r.clicked();
        });
        if running {
            lesson_presentation::prose(ui, "Checking saved profiles and startup paths…");
        } else if scope.tested.as_ref() != Some(&scope.draft) {
            lesson_presentation::prose(
                ui,
                "Not checked · run the current saved profiles for actual results.",
            );
        }
        Ok((start, stop))
    }
}

impl LabApp {
    pub(super) fn nm_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            self.nm_profiles_run = None;
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = work::context(a, &source) {
            self.nm_profiles_run = None;
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (engine::Goal::NmProfiles { definition }, engine::State::NmProfiles { simulation }) =
            (&source.goal, &a.state)
        else {
            return;
        };
        let mut candidate = (**simulation).clone();
        let before = candidate.clone();
        let visited = a.visited;
        let active = ui.is_enabled()
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let mut view = if challenge {
            teaching::View::Assignment
        } else {
            candidate.view
        };
        lesson_presentation::rich(ui, RichText::new(&source.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, egui::Button::new(label).selected(view == v));
                    self.metrics.lesson_tabs.push(r.rect);
                    record(&mut self.metrics.nm_profiles, key, &r, ui);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
            candidate.view = view;
        }
        let practice = view == teaching::View::Example;
        let task = if practice {
            &definition.example
        } else {
            &definition.initial
        };
        if self.nm_profiles_run.as_ref().is_some_and(|job| {
            job.scenario != source.id
                || job.practice != practice
                || view == teaching::View::Introduction
                || &job.draft
                    != if practice {
                        &candidate.practice.draft
                    } else {
                        &candidate.assignment.draft
                    }
        }) {
            self.nm_profiles_run = None;
        }
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                "Builds on IPv4 interfaces and route selection. Edit saved virtual profiles without touching real host configuration.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(ui, &definition.example.instructions);
            if let Some(bridge) = &definition.example.cases[0].environment.bridge {
                bridge_facts(ui, bridge, false);
            }
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.nm_profiles, key, &r, ui);
                    if r.clicked() {
                        candidate.view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        } else {
            if practice {
                lesson_presentation::lead(ui, "PRACTICE TASK", &task.instructions);
                lesson_presentation::practice_step(ui, &source, 0);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            let scope = if practice {
                &mut candidate.practice
            } else {
                &mut candidate.assignment
            };
            let running = self.nm_profiles_run.is_some();
            let mut start = false;
            let mut stop = false;
            let edit_context = EditContext {
                source: &source,
                definition,
                visited,
                practice,
                active,
                running,
            };
            let selected_id = ui.id().with(("nm-selected-case", practice));
            let mut selected = ui
                .ctx()
                .data_mut(|d| d.get_temp::<usize>(selected_id).unwrap_or(0))
                .min(task.cases.len() - 1);
            let mut edit_error = None;
            if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 50. {
                ui.columns(2, |columns| {
                    match edit_context.editor(&mut columns[0], scope, &mut self.metrics.nm_profiles)
                    {
                        Ok((a, b)) => {
                            start = a;
                            stop = b;
                        }
                        Err(e) => edit_error = Some(e),
                    }
                    facts(
                        &mut columns[1],
                        task,
                        &mut selected,
                        &mut self.metrics.nm_profiles,
                    );
                });
            } else {
                match edit_context.editor(ui, scope, &mut self.metrics.nm_profiles) {
                    Ok((a, b)) => {
                        start = a;
                        stop = b;
                    }
                    Err(e) => edit_error = Some(e),
                }
                ui.separator();
                facts(ui, task, &mut selected, &mut self.metrics.nm_profiles);
            }
            ui.ctx().data_mut(|d| d.insert_temp(selected_id, selected));
            if let Some(e) = edit_error {
                self.notice = Some(e);
            }
            self.metrics.check = self
                .metrics
                .nm_profiles
                .controls
                .iter()
                .find(|r| r.0 == "run")
                .map(|r| r.1);
            if stop {
                if let Some(mut job) = self.nm_profiles_run.take() {
                    job.run.cancel();
                }
                self.notice =
                    Some("Stopped · draft retained; unfinished tests award no credit.".into());
            }
            if start && !running && active {
                match scope.begin(task) {
                    Ok(run) => {
                        self.nm_profiles_run = Some(ActiveRun {
                            scenario: source.id.clone(),
                            practice,
                            draft: scope.draft.clone(),
                            run,
                        })
                    }
                    Err(e) => self.notice = Some(e),
                }
            }
            if active && let Some(mut job) = self.nm_profiles_run.take() {
                if job.run.tick() {
                    if let Err(e) = scope.finish(task, &job.run) {
                        self.notice = Some(e);
                    }
                } else {
                    self.nm_profiles_run = Some(job);
                }
                ui.ctx().request_repaint();
            }
            self.metrics.nm_profiles.running = self.nm_profiles_run.is_some();
            if scope.tested.as_ref() == Some(&scope.draft) {
                let cache_id = ui.id().with(("nm-report", &source.id, practice));
                let cached = ui.ctx().data_mut(|d| d.get_temp::<Arc<Report>>(cache_id));
                let cached =
                    if let Some(r) = cached.filter(|r| r.task == *task && r.draft == scope.draft) {
                        r
                    } else {
                        let r = Arc::new(Report {
                            task: task.clone(),
                            draft: scope.draft.clone(),
                            rows: task.run(&scope.draft).expect("validated bounded test"),
                        });
                        ui.ctx().data_mut(|d| d.insert_temp(cache_id, r.clone()));
                        r
                    };
                ui.separator();
                report(
                    ui,
                    &cached.rows,
                    task,
                    selected,
                    &mut self.metrics.nm_profiles,
                );
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                let r = ui.add_enabled(
                    active && !self.metrics.nm_profiles.running,
                    button("Reset example only"),
                );
                record(&mut self.metrics.nm_profiles, "reset-example", &r, ui);
                if r.clicked() {
                    candidate.practice = work::Session::new(definition).unwrap().practice;
                }
            }
            ui.push_id(("nm-help", &source.id, visited, challenge), |ui| {
                controls::collapsing_header(ui, "Help and simulation scope").show(ui, |ui| {
                    lesson_presentation::prose(
                        ui,
                        if task.cases[0].environment.bridge.is_some() {
                            work::BRIDGE_HELP
                        } else if task.cases[0].environment.bond.is_some() {
                            work::BOND_HELP
                        } else if task.cases[0].environment.vlan.is_some() {
                            work::VLAN_HELP
                        } else {
                            work::HELP
                        },
                    );
                    if !practice {
                        let a = self.progress.attempts.get_mut(&source.id).unwrap();
                        let r = ui.add_enabled(
                            active && a.hints < source.hints.len(),
                            button("Show a hint"),
                        );
                        record(&mut self.metrics.nm_profiles, "hint", &r, ui);
                        if r.clicked() {
                            a.hints += 1;
                            self.dirty = true;
                        }
                        for hint in source.hints.iter().take(a.hints) {
                            lesson_presentation::prose(ui, hint);
                        }
                        controls::collapsing_header(ui, "Worked approach · reveals a solution")
                            .show(ui, |ui| {
                                lesson_presentation::prose(ui, &source.explanation);
                                for answer in &source.solution {
                                    lesson_presentation::monospace(
                                        ui,
                                        &answer
                                            .strip_prefix(work::PREFIX)
                                            .and_then(|s| {
                                                serde_json::from_str::<work::Files>(s).ok()
                                            })
                                            .map(|files| {
                                                files
                                                    .into_iter()
                                                    .map(|(name, text)| format!("{name}\n{text}"))
                                                    .collect::<Vec<_>>()
                                                    .join("\n")
                                            })
                                            .unwrap_or_else(|| answer.clone()),
                                        false,
                                    );
                                }
                            });
                    }
                    lesson_presentation::prose(ui, &source.simplifications);
                    for url in &source.sources {
                        ui.hyperlink_to(url, url);
                    }
                });
            });
        }
        if candidate != before && active {
            match definition.bound(&candidate) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::NmProfiles {
                            simulation: Box::new(candidate),
                        };
                    self.completion_index.forget(&source.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
            }
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
    }
}
