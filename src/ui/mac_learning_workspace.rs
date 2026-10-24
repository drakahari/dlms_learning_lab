//! Native Ethernet-learning workspace; no learner terminal or hidden host execution.
use super::*;
use dlms_learning_lab::{
    mac_learning as net, mac_learning_teaching as lesson, mac_learning_workspace as work,
};
use engine::State;
impl LabApp {
    pub(super) fn mac_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            return teaching::View::Assignment;
        }
        a.workspace
            .as_ref()
            .and_then(|w| w.mac_learning.as_ref())
            .map_or(teaching::View::Assignment, |l| l.view)
    }
    pub(super) fn set_mac_view(&mut self, i: usize, view: teaching::View) {
        if let Some(l) = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .and_then(|a| a.workspace.as_mut())
            .and_then(|w| w.mac_learning.as_mut())
        {
            l.view = view
        }
        self.notice = None;
        self.help_open = false;
        self.focus_input = true;
        self.lesson_scroll_reset = true;
        self.dirty = true;
    }
    pub(super) fn mac_learning_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        if self.mac_trace_reset == Some(i) {
            ui.ctx().data_mut(|d| {
                for practice in [false, true] {
                    d.remove::<net::Observation>(egui::Id::new(("mac-trace", i, practice)));
                }
            });
            self.mac_trace_reset = None;
        }
        let active = !self.reset_pending && !self.settings_open && !self.data_tools.modal_open();
        if !active {
            self.focus_input = false;
        }
        ui.add_enabled_ui(active, |ui| self.mac_contents(ui, i));
    }
    fn mac_contents(&mut self, ui: &mut egui::Ui, i: usize) {
        let scenario = self.scenarios[i].clone();
        let view = self.mac_view(i);
        ui.label(RichText::new(&scenario.title).strong().size(23.));
        if !self.progress.attempts[&scenario.id].challenge && scenario.mac_teaching.is_some() {
            ui.horizontal_wrapped(|ui| {
                for (v, text) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, text);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_mac_view(i, v)
                    }
                }
            });
        }
        match view {
            teaching::View::Introduction => {
                let l = scenario.mac_teaching.as_ref().expect("validated teaching");
                ui.label(&l.prerequisite_note);
                for c in &l.concepts {
                    ui.label(RichText::new(&c.title).strong());
                    ui.label(&c.text);
                }
                ui.label(RichText::new("An Ethernet example to try").strong());
                ui.label(&l.example_description);
                ui.label(format!("Try a change: {}", l.try_change));
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_mac_view(i, teaching::View::Example)
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_mac_view(i, teaching::View::Assignment)
                    }
                });
            }
            teaching::View::Example => {
                let l = scenario.mac_teaching.as_ref().unwrap();
                ui.label("Practice only · Sends and settings never change your assignment or earn completion.");
                ui.label(&l.example_description);
                let mut learning = self.progress.attempts[&scenario.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .mac_learning
                    .clone()
                    .unwrap();
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(button("Fill worked example inputs"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        if let Err(e) = learning.fill_example(l) {
                            self.notice = Some(e)
                        }
                        self.dirty = true
                    }
                    let r = ui.add(button("Reset example only"));
                    self.metrics.example_reset = Some(r.rect);
                    if r.clicked() {
                        learning = lesson::Learning::new(l).expect("validated lesson");
                        learning.view = teaching::View::Example;
                        self.dirty = true;
                        self.notice = None;
                        ui.ctx().data_mut(|d| {
                            d.remove::<net::Observation>(egui::Id::new(("mac-trace", i, true)))
                        });
                    }
                });
                let insert_width =
                    controls::button_width(ui, "Insert example action").min(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    ui.label("Example action");
                    let width = 300_f32.min(ui.available_width());
                    let picker = controls::cell(ui, width, |ui| {
                        egui::ComboBox::from_id_salt(("mac-example-action", i))
                            .width(width)
                            .truncate()
                            .selected_text(l.example_solution[learning.selected_action].label())
                            .show_ui(ui, |ui| {
                                for (n, a) in l.example_solution.iter().enumerate() {
                                    let choice = ui.selectable_value(
                                        &mut learning.selected_action,
                                        n,
                                        a.label(),
                                    );
                                    self.metrics.mac_example_choices.push((n, choice.rect));
                                    if choice.changed() {
                                        self.dirty = true;
                                    }
                                }
                                self.metrics.mac_action_popup =
                                    Some(controls::popup_content_clip(ui));
                            })
                    })
                    .inner;
                    self.metrics.mac_example_action = Some(picker.response.rect);
                    name_control(&picker.response, "Ungraded MAC example action");
                    let r = ui.add_sized(
                        [insert_width, controls::height(ui)],
                        button("Insert example action").wrap(),
                    );
                    self.metrics.command_insert = Some(r.rect);
                    if r.clicked() {
                        self.notice = l.example_solution[learning.selected_action]
                            .insert(&learning.simulation, &mut learning.draft)
                            .err();
                        learning.checked = false;
                        self.dirty = true;
                    }
                });
                ui.label("Insert fills the matching native controls only. Use the labeled action button to perform it.");
                ui.label(format!("Try a change: {}", l.try_change));
                let config = config_needed(&l.outcomes, &l.fixture);
                let before = learning.draft.clone();
                let changed = mac_controls(
                    ui,
                    i,
                    true,
                    &mut learning.simulation,
                    &mut learning.draft,
                    config,
                    &mut self.metrics,
                    &mut self.focus_input,
                    &mut self.notice,
                );
                if before != learning.draft || changed {
                    learning.checked = false;
                    self.dirty = true;
                }
                let r = ui.add(primary(ui, "Check example"));
                self.metrics.check = Some(r.rect);
                if r.clicked() {
                    learning.checked = true;
                    self.dirty = true;
                }
                let feedback = controls::feedback(ui, r.clicked(), |ui| {
                    if let Some(e) = &self.notice {
                        ui.colored_label(palette(ui).error, e);
                    }
                    let checks = learning.checks(l);
                    if !checks.is_empty() {
                        mac_feedback(
                            ui,
                            &checks,
                            "Example checks pass · Your assignment is unchanged.",
                            (i, true),
                        );
                    } else if learning.checked {
                        ui.label("Practice draft not applied · Apply settings before checking the current settings.");
                    }
                });
                self.metrics.feedback = Some(feedback.response.rect);
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                if r.clicked() {
                    learning.view = teaching::View::Assignment;
                    self.set_mac_view(i, teaching::View::Assignment)
                }
                self.progress
                    .attempts
                    .get_mut(&scenario.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .mac_learning = Some(learning);
            }
            teaching::View::Assignment => {
                ui.label(RichText::new("Understand the task").strong());
                ui.label(&scenario.objective);
                let State::MacLearning { mut simulation } =
                    self.progress.attempts[&scenario.id].state.clone()
                else {
                    return;
                };
                let engine::Goal::MacLearning {
                    initial,
                    requirements,
                } = &scenario.goal
                else {
                    return;
                };
                let mut draft = self.progress.attempts[&scenario.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .mac_draft
                    .clone()
                    .unwrap();
                let before = draft.clone();
                let changed = mac_controls(
                    ui,
                    i,
                    false,
                    &mut simulation,
                    &mut draft,
                    config_needed(requirements, initial),
                    &mut self.metrics,
                    &mut self.focus_input,
                    &mut self.notice,
                );
                if before != draft || changed {
                    self.dirty = true;
                }
                if !draft.matches(&simulation) {
                    self.checked = false;
                }
                let r = ui.add(primary(ui, "Check work"));
                self.metrics.check = Some(r.rect);
                if r.clicked() {
                    self.checked = true;
                    self.dirty = true;
                }
                let a = self.progress.attempts.get_mut(&scenario.id).unwrap();
                a.state = State::MacLearning { simulation };
                let w = a.workspace.as_mut().unwrap();
                w.mac_draft = Some(draft);
                w.checked = self.checked;
                let feedback = controls::feedback(ui, r.clicked(), |ui| {
                    if let Some(e) = &self.notice {
                        ui.colored_label(palette(ui).error, e);
                    }
                    if self.checked {
                        let a = &self.progress.attempts[&scenario.id];
                        let okay = progress::current_solved(a, &scenario);
                        self.metrics.completion = okay;
                        if a.workspace.as_ref().and_then(|w|w.mac_draft.as_ref()).is_some_and(|d|matches!(&a.state,State::MacLearning{simulation}if !d.matches(simulation))){ui.label("Draft not verified · Apply settings, then Check work.");}
                        else {let checks=engine::checks(&a.state,&scenario.goal);mac_feedback(ui,&checks,"Objective complete · All current checks pass.",(i,false));if okay{self.next_unit_lab(ui,i)}}
                    } else if !before.matches(match &self.progress.attempts[&scenario.id].state {
                        State::MacLearning { simulation } => simulation,
                        _ => unreachable!(),
                    }) {
                        ui.label("Pending settings · Apply settings changes before checking work.");
                    }
                });
                self.metrics.feedback = Some(feedback.response.rect);
            }
        }
    }
}
/// Presentation comes from the authored problem, never from changing current state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ConfigNeeds {
    ports: bool,
    lifetime: bool,
}
fn config_needed(reqs: &[work::Requirement], initial: &net::State) -> ConfigNeeds {
    let mut needs = ConfigNeeds::default();
    for r in reqs {
        match r {
            work::Requirement::Port {
                switch,
                port,
                settings,
            } => {
                needs.ports |= initial
                    .fabric()
                    .switches
                    .get(switch)
                    .and_then(|s| s.ports.get(port))
                    != Some(settings);
            }
            work::Requirement::ActiveVlans { switch, vlans } => {
                needs.ports |= initial
                    .fabric()
                    .switches
                    .get(switch)
                    .is_none_or(|s| &s.vlans != vlans);
            }
            work::Requirement::Outcome {
                outcome: net::Requirement::Aging { minimum, maximum },
            } => {
                needs.lifetime |= !(*minimum..=*maximum).contains(&initial.aging_seconds());
            }
            _ => {}
        }
    }
    needs
}
/// Reopen unapplied work even in a new UI context; keep IDs stable while typing.
fn settings_header(
    text: &str,
    salt: (&'static str, usize, bool),
    pending: bool,
) -> egui::CollapsingHeader {
    egui::CollapsingHeader::new(text)
        .id_salt(salt)
        .open(pending.then_some(true))
}
/// A rejected Send is an action error; existing objective truth is unchanged.
fn frame_not_sent(error: &str) -> String {
    const PREFIX: &str = "Frame not sent: ";
    let room = net::MAX_ERROR_BYTES - PREFIX.len();
    let clipped = error.len() > room;
    let mut end = error.len().min(room - if clipped { 3 } else { 0 });
    while !error.is_char_boundary(end) {
        end -= 1;
    }
    let mut text = String::with_capacity(PREFIX.len() + end + if clipped { 3 } else { 0 });
    text.push_str(PREFIX);
    text.push_str(&error[..end]);
    if clipped {
        text.push_str("...");
    }
    text
}
fn mac_feedback(ui: &mut egui::Ui, checks: &[(bool, String)], complete: &str, id: (usize, bool)) {
    let passed = checks.iter().filter(|c| c.0).count();
    if passed == checks.len() {
        ui.colored_label(palette(ui).success, complete);
    } else {
        for (_, s) in checks.iter().filter(|c| !c.0).take(2) {
            ui.colored_label(palette(ui).error, format!("Retry · {s}"));
        }
    }
    egui::CollapsingHeader::new(format!(
        "Objective results · {passed}/{} passing",
        checks.len()
    ))
    .id_salt(("mac-results", id))
    .show(ui, |ui| {
        for (ok, s) in checks {
            ui.colored_label(
                if *ok {
                    palette(ui).success
                } else {
                    palette(ui).error
                },
                format!("{} · {s}", if *ok { "Pass" } else { "Retry" }),
            );
        }
    });
}
#[allow(clippy::too_many_arguments)]
fn mac_controls(
    ui: &mut egui::Ui,
    i: usize,
    practice: bool,
    s: &mut net::State,
    d: &mut work::Draft,
    needs: ConfigNeeds,
    metrics: &mut Metrics,
    focus: &mut bool,
    notice: &mut Option<String>,
) -> bool {
    let mut changed = false;
    let before = d.clone();
    topology(ui, s, &d.configuration.selected_switch);
    let ports_pending = !d.configuration.matches(s.fabric());
    let lifetime_pending =
        !work::decimal(&d.aging, 3600).is_ok_and(|n| n == u64::from(s.aging_seconds()));
    let ports =
        |ui: &mut egui::Ui, d: &mut work::Draft, metrics: &mut Metrics, focus: &mut bool| {
            super::switching_workspace::configuration_editor(
                ui,
                i,
                practice,
                s.fabric(),
                &mut d.configuration,
                metrics,
                focus,
                "Pending settings · Apply settings before checking work",
            );
        };
    let config_open = if needs.ports {
        ports(ui, d, metrics, focus);
        true
    } else {
        let salt = ("mac-settings", i, practice);
        let response = settings_header("Optional port and VLAN settings", salt, ports_pending)
            .show(ui, |ui| ports(ui, d, metrics, focus));
        metrics.mac_settings = Some(response.header_response.rect);
        response.body_returned.is_some()
    };
    let lifetime =
        |ui: &mut egui::Ui, d: &mut work::Draft, metrics: &mut Metrics, focus: &mut bool| {
            super::ipv6_workspace::field(
                ui,
                "Entry lifetime (seconds; 0 disables expiry)",
                &mut d.aging,
                4,
                ("mac-aging", i, practice),
                metrics,
                focus,
            );
            metrics.mac_aging = metrics.inputs.last().copied();
        };
    let lifetime_open = if needs.lifetime {
        lifetime(ui, d, metrics, focus);
        true
    } else {
        let salt = ("mac-lifetime-settings", i, practice);
        let response = settings_header("Optional entry lifetime", salt, lifetime_pending)
            .show(ui, |ui| lifetime(ui, d, metrics, focus));
        metrics.mac_lifetime = Some(response.header_response.rect);
        response.body_returned.is_some()
    };
    if !d.matches(s) {
        ui.label("Pending settings · Apply changes before sending frames with them.");
    }
    if config_open || lifetime_open || !d.matches(s) {
        let r = ui.add(button("Apply settings"));
        metrics.editor_save = Some(r.rect);
        if r.clicked() {
            match d.apply(s) {
                Ok(()) => {
                    *notice = None;
                    changed = true
                }
                Err(e) => *notice = Some(e),
            }
        }
    }
    ui.label(RichText::new("Send one Ethernet metadata frame").strong());
    ui.horizontal(|ui| {
        let label_width = controls::form_label_width(ui);
        controls::label(ui, "From workstation", label_width);
        let width = 240_f32.min(ui.available_width());
        let picker = controls::cell(ui, width, |ui| {
            egui::ComboBox::from_id_salt(("mac-from", i, practice))
                .width(width)
                .truncate()
                .selected_text(&d.source)
                .show_ui(ui, |ui| {
                    for (h, v) in &s.fabric().hosts {
                        let choice = ui.selectable_value(
                            &mut d.source,
                            h.clone(),
                            format!("{} ({h}) · {}", v.label, s.address(h).unwrap().text()),
                        );
                        metrics.mac_sources.push((h.clone(), choice.rect));
                    }
                    metrics.mac_source_popup = Some(controls::popup_content_clip(ui));
                })
        })
        .inner;
        name_control(
            &picker.response,
            if practice {
                "Practice frame source"
            } else {
                "Assignment frame source"
            },
        );
        metrics.mac_source = Some(picker.response.rect);
        metrics.inputs.push(picker.response.rect);
    });
    super::ipv6_workspace::field(
        ui,
        "Destination MAC (ff:ff:ff:ff:ff:ff is broadcast)",
        &mut d.destination,
        17,
        ("mac-destination", i, practice),
        metrics,
        focus,
    );
    metrics.mac_destination = metrics.inputs.last().copied();
    let send = ui.add(button("Send frame"));
    metrics.mac_send = Some(send.rect);
    metrics.mac_send_id = Some(send.id);
    metrics.mac_send_enabled = Some(send.enabled());
    if send.clicked() {
        match net::Mac::parse(&d.destination).and_then(|address| s.send(&d.source, address)) {
            Ok(result) => {
                ui.ctx().data_mut(|data| {
                    data.insert_temp(egui::Id::new(("mac-trace", i, practice)), result)
                });
                *notice = None;
                changed = true
            }
            Err(e) => *notice = Some(frame_not_sent(&e)),
        }
    }
    ui.label(format!(
        "Applied state · {} simulated seconds · entry lifetime {}s",
        s.now(),
        s.aging_seconds()
    ));
    ui.label(RichText::new("Learned MAC entries · applied state").strong());
    if !config_open {
        ui.horizontal(|ui| {
            let label_width = controls::form_label_width(ui);
            controls::label(ui, "Inspect switch", label_width);
            let width = 240_f32.min(ui.available_width());
            let r = controls::cell(ui, width, |ui| {
                egui::ComboBox::from_id_salt(("mac-inspect-switch", i, practice))
                    .width(width)
                    .truncate()
                    .selected_text(&d.configuration.selected_switch)
                    .show_ui(ui, |ui| {
                        for (id, switch) in &s.fabric().switches {
                            ui.selectable_value(
                                &mut d.configuration.selected_switch,
                                id.clone(),
                                format!("{} ({id})", switch.label),
                            );
                        }
                    })
            })
            .inner;
            name_control(&r.response, "Switch whose learned MAC table is displayed");
            metrics.mac_inspect = Some(r.response.rect);
        });
    }
    let selected = &d.configuration.selected_switch;
    ui.label(format!("Switch {selected}. Sending learns its source; looking up a destination does not refresh it."));
    egui::ScrollArea::horizontal()
        .id_salt(("mac-table", i, practice))
        .show(ui, |ui| {
            egui::Grid::new(("mac-entries", i, practice))
                .striped(true)
                .show(ui, |ui| {
                    for label in ["VLAN", "MAC address", "Port", "Age (seconds)"] {
                        ui.label(RichText::new(label).strong());
                    }
                    ui.end_row();
                    for e in s.entries().iter().filter(|e| &e.switch == selected) {
                        ui.label(e.vlan.to_string());
                        ui.monospace(e.address.text());
                        ui.label(&e.port);
                        ui.label((s.now() - e.learned_at).to_string());
                        ui.end_row();
                    }
                });
        });
    if !s.entries().iter().any(|e| &e.switch == selected) {
        ui.label("No learned entries on this switch. Send a frame from a workstation to learn its source.");
    }
    let tools=egui::CollapsingHeader::new("Time and selective clear tools").id_salt(("mac-tools",i,practice)).show(ui,|ui|{
        ui.label("Time advances only when requested. Clear removes dynamic entries; it does not change ports or cables.");
        super::ipv6_workspace::field(ui,"Advance seconds (0–86400)",&mut d.advance_seconds,20,("mac-advance",i,practice),metrics,focus);
        metrics.mac_time=metrics.inputs.last().copied();
        let r=ui.add(button("Advance time"));metrics.mac_advance=Some(r.rect);if r.clicked(){match work::decimal(&d.advance_seconds,86400).and_then(|n|s.advance(n)){Ok(())=>{*notice=None;changed=true},Err(e)=>*notice=Some(e)}}
        ui.horizontal(|ui|{let label_width=controls::form_label_width(ui);controls::label(ui,"Clear on switch",label_width);let width=220_f32.min(ui.available_width());let r=controls::cell(ui,width,|ui|egui::ComboBox::from_id_salt(("mac-clear-switch",i,practice)).width(width).truncate().selected_text(&d.clear_switch).show_ui(ui,|ui|{for id in s.fabric().switches.keys(){let choice=ui.selectable_value(&mut d.clear_switch,id.clone(),id);metrics.mac_clear_choices.push((id.clone(),choice.rect));}metrics.mac_clear_popup=Some(controls::popup_content_clip(ui));})).inner;name_control(&r.response,"Switch for selective MAC clear");metrics.mac_clear_switch=Some(r.response.rect);});
        super::ipv6_workspace::field(ui,"Clear VLAN (empty means all VLANs on this switch)",&mut d.clear_vlan,4,("mac-clear-vlan",i,practice),metrics,focus);
        metrics.mac_clear_vlan=metrics.inputs.last().copied();
        let r=ui.add(button("Clear dynamic entries"));metrics.mac_clear=Some(r.rect);if r.clicked(){let vlan=if d.clear_vlan.is_empty(){Ok(None)}else{work::decimal(&d.clear_vlan,4094).and_then(|n|if n>0{Ok(Some(n as u16))}else{Err("Use a VLAN from 1 to 4094, or leave it empty".into())})};match vlan.and_then(|v|s.clear_dynamic(&d.clear_switch,v)){Ok(n)=>{*notice=Some(format!("Cleared {n} dynamic entries on {}.",d.clear_switch));changed=true},Err(e)=>*notice=Some(e)}}
    });
    metrics.mac_tools = Some(tools.header_response.rect);
    egui::CollapsingHeader::new("Last frame trace · observation only").id_salt(("mac-trace-panel",i,practice)).show(ui,|ui|{
        if let Some(trace)=ui.ctx().data(|data|data.get_temp::<net::Observation>(egui::Id::new(("mac-trace",i,practice)))){ui.label(format!("Copies arrived at: {:?}. Accepted by: {:?}.",trace.arrivals,trace.accepted_by));egui::ScrollArea::vertical().id_salt(("mac-trace-scroll",i,practice)).max_height(220.).show(ui,|ui|{for e in &trace.events{ui.add(egui::Label::new(format!("{e:?}")).wrap());}});}else{ui.label("Send a frame to inspect its learning and forwarding decisions. This temporary trace is not saved or used to award completion.");}
    });
    if before.configuration.switches != d.configuration.switches || before.aging != d.aging {
        *notice = None;
    }
    changed
}
fn topology(ui: &mut egui::Ui, s: &net::State, selected: &str) {
    use dlms_learning_lab::switching::Jack;
    let f = s.fabric();
    let mut cards = Vec::<[String; 3]>::new();
    let mut links = Vec::new();
    for c in &f.cables {
        let pair = match (&c.a, &c.b) {
            (Jack::Switch { switch, port }, peer) if switch == selected => Some((port, peer)),
            (peer, Jack::Switch { switch, port }) if switch == selected => Some((port, peer)),
            _ => None,
        };
        if let Some((port, peer)) = pair {
            match peer {
                Jack::Host { host } => {
                    cards.push([
                        f.hosts[host].label.clone(),
                        host.clone(),
                        s.address(host).unwrap().text(),
                    ]);
                    links.push(format!(
                        "{selected}/{port} to {host} ({})",
                        if c.up { "cable up" } else { "cable down" }
                    ));
                }
                Jack::Switch {
                    switch,
                    port: other,
                } => {
                    cards.push([
                        f.switches[switch].label.clone(),
                        switch.clone(),
                        "Virtual switch".into(),
                    ]);
                    links.push(format!(
                        "{selected}/{port} to {switch}/{other} ({})",
                        if c.up { "cable up" } else { "cable down" }
                    ));
                }
            }
        }
    }
    let refs = cards
        .iter()
        .map(|c| [c[0].as_str(), c[1].as_str(), c[2].as_str()])
        .collect::<Vec<_>>();
    let caption = links.join("; ");
    if !controls::topology_star(
        ui,
        [&f.switches[selected].label, selected, "Virtual switch"],
        &refs,
        &caption,
    ) {
        ui.label(
            RichText::new(format!(
                "{} ({selected}) · physical connections",
                f.switches[selected].label
            ))
            .strong(),
        );
        for line in &links {
            ui.label(line);
        }
    }
}
