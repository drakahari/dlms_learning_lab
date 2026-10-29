use super::*;
use dlms_learning_lab::{
    discovery::{self as net, Endpoint, Outcome},
    discovery_teaching::{Learning, Lesson},
    discovery_workspace::{Action, ActionResult, Draft},
};

pub(super) fn prepare(a: &mut Attempt, lesson: &Lesson, new_attempt: bool) {
    if model::is_file_backup_workspace(a) {
        return;
    }
    if dlms_learning_lab::workspace::is_etherchannel_workspace(a)
        || dlms_learning_lab::workspace::is_cron_workspace(a)
    {
        return;
    }
    if dlms_learning_lab::workspace::has_nfs_metadata(a)
        || dlms_learning_lab::workspace::has_cron_metadata(a)
    {
        return;
    }
    let w = a.workspace.as_mut().expect("workspace prepared");
    if w.discovery_learning.is_none() {
        let mut l = Learning::new(lesson);
        if !new_attempt {
            l.view = teaching::View::Assignment;
        }
        w.discovery_learning = Some(l);
    }
}
impl LabApp {
    pub(super) fn discovery_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            return teaching::View::Assignment;
        }
        a.workspace
            .as_ref()
            .and_then(|w| w.discovery_learning.as_ref())
            .map_or(teaching::View::Assignment, |l| l.view)
    }
    pub(super) fn set_discovery_view(&mut self, i: usize, view: teaching::View) {
        self.progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .discovery_learning
            .as_mut()
            .unwrap()
            .view = view;
        self.help_open = false;
        self.lesson_scroll_reset = true;
        self.dirty = true;
    }
    pub(super) fn discovery_learning_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let lesson = s
            .discovery_teaching
            .as_ref()
            .expect("validated discovery lesson");
        let view = self.discovery_view(i);
        ui.heading(&s.title);
        if !self.progress.attempts[&s.id].challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_discovery_view(i, v);
                    }
                }
            });
        }
        match view {
            teaching::View::Introduction => {
                ui.label(RichText::new(&lesson.prerequisite_note).color(palette(ui).muted));
                for c in &lesson.concepts {
                    ui.label(RichText::new(&c.title).strong());
                    ui.label(&c.text);
                }
                ui.label(RichText::new("A separate example to try").strong());
                ui.label(&lesson.example_description);
                ui.label(format!("Try a change: {}", lesson.try_change.description));
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_discovery_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_discovery_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => {
                ui.label("Practice example · Not graded; your assignment stays separate.");
                ui.label(&lesson.example_description);
                let mut l = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .discovery_learning
                    .as_ref()
                    .unwrap()
                    .clone();
                let before = serde_json::to_vec(&l).unwrap();
                let reset=ui.horizontal_wrapped(|ui| {
                    let fill=ui.add(button("Fill example settings")); self.metrics.command_insert=Some(fill.rect);
                    if fill.clicked(){ l.notice=l.fill_example(lesson).err(); }
                    ui.label("Fills editor settings only. Apply them, then send advertisements explicitly.");
                    let r=ui.add(button("Reset example only"));self.metrics.example_reset=Some(r.rect);r.clicked()
                }).inner;
                if reset {
                    l = Learning::new(lesson);
                    l.view = teaching::View::Example;
                }
                editor(
                    ui,
                    (i, true),
                    &mut l.simulation,
                    &mut l.draft,
                    &mut l.notice,
                    &mut self.metrics,
                );
                let r = ui.add(primary(ui, "Check example · ungraded"));
                self.metrics.check = Some(r.rect);
                if r.clicked() {
                    let _ = l.check(lesson);
                }
                for (pass, message) in l.checks(lesson) {
                    ui.colored_label(
                        if pass {
                            palette(ui).success
                        } else {
                            palette(ui).error
                        },
                        message,
                    );
                }
                ui.label(format!("Try a change: {}", lesson.try_change.description));
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                let assignment = r.clicked();
                if before != serde_json::to_vec(&l).unwrap() {
                    self.dirty = true;
                }
                self.progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .discovery_learning = Some(l);
                if assignment {
                    self.set_discovery_view(i, teaching::View::Assignment);
                }
            }
            teaching::View::Assignment => {
                ui.label(&s.objective);
                let a = &self.progress.attempts[&s.id];
                let engine::State::Discovery { simulation } = &a.state else {
                    unreachable!()
                };
                let mut sim = simulation.clone();
                let mut draft = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .discovery_draft
                    .as_ref()
                    .unwrap()
                    .clone();
                let original_draft = draft.clone();
                let mut notice = self.notice.clone();
                editor(
                    ui,
                    (i, false),
                    &mut sim,
                    &mut draft,
                    &mut notice,
                    &mut self.metrics,
                );
                if &sim != simulation || draft != original_draft {
                    let a = self.progress.attempts.get_mut(&s.id).unwrap();
                    a.state = engine::State::Discovery { simulation: sim };
                    a.workspace.as_mut().unwrap().discovery_draft = Some(draft);
                    self.dirty = true;
                }
                self.notice = notice;
                let r = ui.add_sized(
                    [220_f32.min(ui.available_width()), controls::height(ui)],
                    primary(ui, "Check configuration"),
                );
                self.metrics.check = Some(r.rect);
                if r.clicked() {
                    self.checked = true;
                    self.progress
                        .attempts
                        .get_mut(&s.id)
                        .unwrap()
                        .workspace
                        .as_mut()
                        .unwrap()
                        .checked = true;
                    self.dirty = true;
                }
                let result = controls::feedback(ui, r.clicked(), |ui| {
                    let a = &self.progress.attempts[&s.id];
                    let now = progress::current_solved(a, &s);
                    self.metrics.completion = self.checked && now;
                    if self.checked {
                        let engine::State::Discovery { simulation } = &a.state else {
                            unreachable!()
                        };
                        if !a
                            .workspace
                            .as_ref()
                            .unwrap()
                            .discovery_draft
                            .as_ref()
                            .unwrap()
                            .matches(simulation)
                        {
                            ui.colored_label(palette(ui).error,"Unsaved settings · Apply the editor changes before checking current neighbors.");
                        } else if now {
                            ui.colored_label(
                                palette(ui).success,
                                "Objective complete · All current checks pass.",
                            );
                            self.next_unit_lab(ui, i);
                        } else {
                            for issue in model::issues(&a.state, &s.goal) {
                                ui.colored_label(palette(ui).error, issue);
                            }
                        }
                    }
                });
                self.metrics.feedback = Some(result.response.rect);
            }
        }
    }
}

fn editor(
    ui: &mut egui::Ui,
    id: (usize, bool),
    simulation: &mut net::State,
    draft: &mut Draft,
    notice: &mut Option<String>,
    metrics: &mut Metrics,
) {
    let before = draft.clone();
    // Capture the parent column before wrapped rows can grow their minimum rect.
    // Use this same width for all fields and the actual table columns.
    let width = ui.available_width();
    let topology = simulation.topology().clone();
    ui.label(
        RichText::new("Applied topology · simulated LLDP")
            .small()
            .color(palette(ui).muted),
    );
    topology_cards(ui, &topology);
    ui.horizontal_wrapped(|ui| {
        ui.label("Configure device");
        let r = egui::ComboBox::from_id_salt(("discovery-device", id))
            .width(220_f32.min(ui.available_width()))
            .truncate()
            .selected_text(&draft.selected_device)
            .show_ui(ui, |ui| {
                for name in topology.devices.keys() {
                    let option =
                        ui.selectable_value(&mut draft.selected_device, name.clone(), name);
                    metrics.discovery_options.push((name.clone(), option.rect));
                }
            })
            .response;
        name_control(&r, "LLDP device to configure");
        metrics.discovery_picker = Some(r.rect);
    });
    let selected = draft.selected_device.clone();
    let d = draft.devices.get_mut(&selected).unwrap();
    ui.horizontal_wrapped(|ui| {
        let r = ui.checkbox(&mut d.enabled, "LLDP enabled");
        metrics
            .discovery_toggles
            .push((format!("{selected}:enabled"), r.rect));
        ui.label("Advertised lifetime (seconds)");
        field(
            ui,
            &mut d.ttl,
            105.,
            40,
            format!("{selected} advertised lifetime"),
            (id, &selected, "ttl"),
            metrics,
        );
    });
    ui.horizontal_wrapped(|ui| {
        ui.label("Advertised name");
        optional_field(
            ui,
            &mut d.system_name,
            320_f32.min(width),
            format!("{selected} advertised name"),
            (id, &selected, "name"),
            metrics,
        );
    });
    let mut send = None;
    let wide = width >= 790.;
    egui::Grid::new(("discovery-port-grid", id, &selected))
        .num_columns(if wide { 6 } else { 2 })
        .spacing([12., 8.])
        .show(ui, |ui| {
            if wide {
                for (label, w) in [
                    ("Port / peer", 160.),
                    ("Up", 50.),
                    ("Transmit", 84.),
                    ("Receive", 84.),
                    ("Port description", (width - 533.).max(100.)),
                    ("Advertise", 95.),
                ] {
                    controls::label(ui, RichText::new(label).strong(), w);
                }
                ui.end_row();
            }
            for (name, p) in &mut d.ports {
                let ep = Endpoint {
                    device: selected.clone(),
                    port: name.clone(),
                };
                let peer = topology.links.iter().find_map(|l| {
                    if l.a == ep {
                        Some(&l.b)
                    } else if l.b == ep {
                        Some(&l.a)
                    } else {
                        None
                    }
                });
                let label = peer.map_or_else(
                    || format!("{name} · unattached"),
                    |other| format!("{name} -> {}/{}", other.device, other.port),
                );
                if wide {
                    controls::label(ui, label, 160.);
                } else {
                    controls::label(ui, RichText::new(label).strong(), 130.);
                    ui.end_row();
                }
                for (tag, value, w) in [
                    ("Up", &mut p.up, 50.),
                    ("Transmit", &mut p.transmit, 84.),
                    ("Receive", &mut p.receive, 84.),
                ] {
                    let r = controls::cell(ui, if wide { w } else { 130. }, |ui| {
                        ui.checkbox(value, if wide { "" } else { tag })
                    })
                    .inner;
                    name_control(&r, format!("{selected}/{name} {tag}"));
                    metrics
                        .discovery_toggles
                        .push((format!("{selected}/{name}:{tag}"), r.rect));
                    if !wide {
                        ui.end_row();
                    }
                }
                if !wide {
                    ui.label("Port description");
                }
                optional_field(
                    ui,
                    &mut p.description,
                    if wide {
                        (width - 533.).max(100.)
                    } else {
                        (width - 155.).max(80.)
                    },
                    format!("{selected}/{name} description"),
                    (id, &selected, name, "description"),
                    metrics,
                );
                if !wide {
                    ui.end_row();
                    controls::label(ui, "Send one advertisement", 130.);
                }
                let r = ui.add_sized(
                    [95_f32.min(ui.available_width()), controls::height(ui)],
                    button("Send now"),
                );
                name_control(
                    &r,
                    format!("Send LLDP advertisement from {selected}/{name}"),
                );
                metrics.discovery_send.push((ep.clone(), r.rect));
                if r.clicked() {
                    send = Some(Action::Advertise { from: ep });
                }
                ui.end_row();
            }
        });
    let pending = !draft.matches(simulation);
    ui.horizontal_wrapped(|ui|{
        let r=ui.add(button("Apply settings"));metrics.editor_save=Some(r.rect);
        if r.clicked(){*notice=Some(match draft.apply(simulation){Ok(())=>"Applied simulated settings. A fresh information advertisement is still a separate action.".into(),Err(e)=>e});}
        ui.label(if pending{"Editor has unapplied changes."}else{"Editor matches the applied settings."});
    });
    ui.label(RichText::new("Send now uses applied settings. This exercise sends events manually; advancing time does not send packets.").small().color(palette(ui).muted));
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("Simulated time: {}s", simulation.now()));
        ui.label("Advance by seconds");
        field(ui, &mut draft.advance_seconds, 105., 40, "Seconds to advance the simulated LLDP clock".into(), (id, "advance"), metrics);
        let r = ui.add(button("Advance time"));
        metrics.discovery_time = Some(r.rect);
        if r.clicked() {
            match draft.advance_seconds.parse::<u64>().ok().filter(|_| !draft.advance_seconds.is_empty() && draft.advance_seconds.bytes().all(|b| b.is_ascii_digit())) {
                Some(seconds) => send = Some(Action::Advance { seconds }),
                None => *notice = Some("Enter a whole number of simulated seconds, from 0 through 1000000000. No real time elapses.".into()),
            }
        }
    });
    if let Some(action) = send {
        *notice = Some(if !draft.matches(simulation) {
            "Apply or correct the editor changes before sending an advertisement or advancing time. Your draft stays saved.".into()
        } else {
            match action.apply(simulation) {
                Ok(result) => action_notice(result),
                Err(e) => e,
            }
        });
    }
    if draft != &before {
        *notice = None;
    }
    if let Some(text) = notice {
        ui.label(text.as_str());
    }
    ui.label(RichText::new(format!("Neighbor records received by {selected}")).strong());
    let neighbors = simulation.neighbors(&selected).unwrap_or_default();
    if neighbors.is_empty() {
        ui.label("No unexpired neighbor records. Inspect the sender and receiver settings before sending a fresh advertisement.");
    }
    for n in neighbors {
        let status = if n.can_receive && n.matches_current {
            "Current information"
        } else {
            "Historical information · refresh or expiry needed"
        };
        ui.label(format!(
            "{} <- {}/{} · {}s remaining · {status}",
            n.record.local.port, n.record.peer.device, n.record.peer.port, n.remaining_seconds
        ));
        ui.label(format!(
            "Advertised name: {} · Port description: {}",
            n.record
                .advertisement
                .system_name
                .as_deref()
                .unwrap_or("not supplied"),
            n.record
                .advertisement
                .port_description
                .as_deref()
                .unwrap_or("not supplied")
        ));
        egui::CollapsingHeader::new("Record details · optional")
            .id_salt(("lldp-record-details", id, &n.record.local.device, &n.record.local.port))
            .show(ui, |ui| {
                ui.add(egui::Label::new(format!(
                    "Advertised chassis ID: {} · Advertised port ID: {}. These are locally assigned identifiers in this model.\nReceived {} simulated seconds ago; advertised lifetime {} seconds.",
                    n.record.advertisement.chassis_id,
                    n.record.advertisement.port_id,
                    n.age_seconds,
                    n.record.advertisement.ttl
                )).wrap());
            });
    }
}
fn field(
    ui: &mut egui::Ui,
    text: &mut String,
    width: f32,
    limit: usize,
    label: String,
    id: impl std::hash::Hash,
    metrics: &mut Metrics,
) {
    let r = ui.add_sized(
        [
            width.min(ui.available_width()).max(40.),
            controls::height(ui),
        ],
        controls::singleline(ui, text)
            .id_salt(id)
            .char_limit(limit)
            .hint_text("Not supplied"),
    );
    name_control(&r, &label);
    metrics.discovery_fields.push((label, r.rect));
    metrics.inputs.push(r.rect);
    metrics.focus.get_or_insert(r.id);
    if r.changed() {
        workspace::bound(text, limit);
    }
}
fn optional_field(
    ui: &mut egui::Ui,
    value: &mut Option<String>,
    width: f32,
    label: String,
    id: impl std::hash::Hash,
    metrics: &mut Metrics,
) {
    let mut text = value.clone().unwrap_or_default();
    field(ui, &mut text, width, 256, label, id, metrics);
    *value = if text.is_empty() { None } else { Some(text) };
}
fn action_notice(result: ActionResult) -> String {
    match result {
        ActionResult::ConfigurationApplied => "Applied simulated LLDP settings.".into(),
        ActionResult::TimeAdvanced { now } => format!(
            "Simulated time is {now}s. Expired records were removed; no advertisement was sent."
        ),
        ActionResult::Transmission(t) => format!(
            "{}/{}: {}",
            t.sender.device,
            t.sender.port,
            match t.outcome {
                Outcome::Recorded => "the attached receiver recorded this advertisement.",
                Outcome::Withdrawn => "the attached receiver removed this neighbor.",
                Outcome::TransmitDisabled =>
                    "transmit is disabled. Inspect the sending device and port.",
                Outcome::ReceiveDisabled =>
                    "the attached receiver is disabled. Inspect its device and receive setting.",
                Outcome::LinkDown => "a physical port or link is down.",
                Outcome::Unattached => "this port has no attached peer.",
            }
        ),
    }
}
fn topology_cards(ui: &mut egui::Ui, t: &net::Topology) {
    if (2..=3).contains(&t.links.len())
        && let Some((hub, device)) = t.devices.iter().find(|(name, _)| {
            t.links
                .iter()
                .all(|l| &l.a.device == *name || &l.b.device == *name)
        })
    {
        let peers: Vec<_> = t
            .links
            .iter()
            .map(|l| {
                if &l.a.device == hub {
                    &l.b.device
                } else {
                    &l.a.device
                }
            })
            .collect();
        let unique: std::collections::BTreeSet<_> = peers.iter().copied().collect();
        if unique.len() == peers.len() && peers.len() + 1 == t.devices.len() {
            let labels: Vec<_> = peers
                .iter()
                .map(|name| {
                    let d = &t.devices[*name];
                    [
                        name.as_str(),
                        d.system_name.as_deref().unwrap_or("Name not supplied"),
                        if d.enabled {
                            "LLDP enabled"
                        } else {
                            "LLDP disabled"
                        },
                    ]
                })
                .collect();
            let caption = t
                .links
                .iter()
                .map(|l| {
                    format!(
                        "{}/{} - {}/{} ({})",
                        l.a.device,
                        l.a.port,
                        l.b.device,
                        l.b.port,
                        if l.up
                            && t.devices[&l.a.device].ports[&l.a.port].up
                            && t.devices[&l.b.device].ports[&l.b.port].up
                        {
                            "Up"
                        } else {
                            "Down"
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join(" · ");
            if controls::topology_star(
                ui,
                [
                    hub,
                    device.system_name.as_deref().unwrap_or("Name not supplied"),
                    if device.enabled {
                        "LLDP enabled"
                    } else {
                        "LLDP disabled"
                    },
                ],
                &labels,
                &caption,
            ) {
                return;
            }
        }
    }
    for l in &t.links {
        let a = &t.devices[&l.a.device];
        let b = &t.devices[&l.b.device];
        let left = format!("{}/{}", l.a.device, l.a.port);
        let right = format!("{}/{}", l.b.device, l.b.port);
        controls::topology_pair(
            ui,
            [
                &left,
                a.system_name.as_deref().unwrap_or("Name not supplied"),
                if a.enabled {
                    "LLDP enabled"
                } else {
                    "LLDP disabled"
                },
            ],
            [
                &right,
                b.system_name.as_deref().unwrap_or("Name not supplied"),
                if b.enabled {
                    "LLDP enabled"
                } else {
                    "LLDP disabled"
                },
            ],
            palette(ui).muted,
            if l.up && a.ports[&l.a.port].up && b.ports[&l.b.port].up {
                "Direct physical link · Up"
            } else {
                "Direct physical link · Down"
            },
        );
    }
}

#[cfg(test)]
mod nfs_prepare_guard_tests {
    use super::*;
    #[test]
    fn direct_discovery_prepare_has_normal_control_and_atomic_nfs_refusal() {
        let s = dlms_learning_lab::pack::bundled()
            .unwrap()
            .into_iter()
            .find(|s| s.discovery_teaching.is_some())
            .unwrap();
        let lesson = s.discovery_teaching.as_ref().unwrap();
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/nfs-workbench-v1.json"))
                .unwrap();
        let nfs: dlms_learning_lab::nfs_teaching::Lesson =
            serde_json::from_value(raw["cases"][0]["lesson"].clone()).unwrap();
        let learning = dlms_learning_lab::nfs_teaching::Learning::new(&nfs).unwrap();
        for new in [false, true] {
            for existing in [false, true] {
                let mut a = Attempt::new(&s, 7);
                model::prepare(&mut a);
                if existing {
                    prepare(&mut a, lesson, new);
                }
                let mut normal = a.clone();
                prepare(&mut normal, lesson, new);
                assert!(
                    normal
                        .workspace
                        .as_ref()
                        .unwrap()
                        .discovery_learning
                        .is_some()
                );
                for (session, practice) in [(true, false), (false, true), (true, true)] {
                    let mut mixed = a.clone();
                    let w = mixed.workspace.as_mut().unwrap();
                    if session {
                        w.nfs_session = Some(dlms_learning_lab::nfs_workspace::Session::default());
                    }
                    if practice {
                        w.nfs_learning = Some(learning.clone());
                    }
                    assert!(w.valid());
                    let before = serde_json::to_value(&mixed).unwrap();
                    prepare(&mut mixed, lesson, new);
                    assert!(
                        serde_json::to_value(&mixed).unwrap() == before,
                        "complete caller unchanged"
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod backup_family_refusal_tests {
    use super::*;
    #[test]
    fn real_discovery_prepare_preserves_backup_kind_and_valid_foreign_learning_in_both_modes() {
        let all = pack::bundled().unwrap();
        let source = all.iter().find(|s| s.discovery_teaching.is_some()).unwrap();
        let lesson = source.discovery_teaching.as_ref().unwrap();
        let extra: pack::Pack =
            serde_json::from_str(include_str!("../../scenarios/file-backup-v1.json")).unwrap();
        let backup = &extra.scenarios[0];
        let owned = backup.file_backup_teaching.as_ref().unwrap();
        owned.validate().unwrap();
        let mut prepared = Attempt::new(backup, 11);
        model::prepare(&mut prepared);
        dlms_learning_lab::file_backup_teaching::prepare(&mut prepared, owned, true).unwrap();
        for has_learning in [false, true] {
            for new in [false, true] {
                let mut a = if has_learning {
                    prepared.clone()
                } else {
                    Attempt::new(backup, 11)
                };
                let before = serde_json::to_vec(&a).unwrap();
                prepare(&mut a, lesson, new);
                assert_eq!(serde_json::to_vec(&a).unwrap(), before);
            }
        }
        let mut own = Attempt::new(source, 7);
        model::prepare(&mut own);
        prepare(&mut own, lesson, true);
        assert!(own.workspace.as_ref().unwrap().discovery_learning.is_some());
        own.workspace.as_mut().unwrap().file_backup_learning = prepared
            .workspace
            .as_ref()
            .unwrap()
            .file_backup_learning
            .clone();
        assert!(own.workspace.as_ref().unwrap().valid());
        let before = serde_json::to_vec(&own).unwrap();
        for new in [false, true] {
            prepare(&mut own, lesson, new);
            assert_eq!(serde_json::to_vec(&own).unwrap(), before);
        }
    }
}
