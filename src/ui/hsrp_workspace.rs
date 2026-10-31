//! Native configuration form for the settled HSRP model; no IOS/host terminal.
use super::controls::GAP;
use super::*;
use dlms_learning_lab::{hsrp as core, hsrp_workspace as work, script_teaching::View};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub rows: Vec<(bool, egui::Rect)>,
    pub topology: Option<(egui::Rect, egui::Rect)>,
    pub goal: Option<egui::Rect>,
    pub summary: Option<(usize, usize, egui::Rect)>,
    pub probe: Option<egui::Rect>,
    #[cfg(test)]
    pub intents: (bool, bool, bool),
    #[cfg(test)]
    pub guidance: bool,
    #[cfg(test)]
    pub scope_intents: (bool, bool),
    #[cfg(test)]
    pub disclosures: Vec<&'static str>,
}
#[derive(Default)]
pub(super) struct Presentation {
    generation: [u64; 2],
    #[cfg(test)]
    pub(super) fields: Vec<egui::Id>,
    #[cfg(not(test))]
    fields: Vec<egui::Id>,
    retired: Vec<egui::Id>,
    held: bool,
    pub(super) pending_mode: Option<bool>,
    veto: Option<u64>,
    #[cfg(test)]
    pub(super) events: Vec<core::FaultEvent>,
    #[cfg(not(test))]
    events: Vec<core::FaultEvent>,
    owner: Option<(String, u32, u64, bool)>,
    rows: Vec<(bool, String)>,
    memo: Option<(work::Definition, work::Session, bool)>,
}
impl Presentation {
    pub(super) fn reset_all(&mut self) {
        self.generation[0] = self.generation[0].wrapping_add(1);
        self.generation[1] = self.generation[1].wrapping_add(1);
        self.retired.append(&mut self.fields);
        self.events.clear();
        self.owner = None;
        self.memo = None;
        self.rows.clear();
        self.held = false;
        self.pending_mode = None;
        self.veto = None;
    }
    fn reset_example(&mut self) {
        self.generation[1] = self.generation[1].wrapping_add(1);
        self.retired.append(&mut self.fields);
        self.events.clear();
        self.memo = None;
    }
    fn rows(
        &mut self,
        d: &work::Definition,
        s: &work::Session,
        practice: bool,
    ) -> &[(bool, String)] {
        if self
            .memo
            .as_ref()
            .is_none_or(|(old, session, p)| old != d || session != s || *p != practice)
        {
            self.rows = if practice {
                d.practice_checks(s)
            } else {
                d.checks(s)
            };
            self.memo = Some((d.clone(), s.clone(), practice));
        }
        &self.rows
    }
}
struct NativeEditBuffer<'a> {
    value: &'a mut String,
    rejected: bool,
    received: bool,
}
impl egui::TextBuffer for NativeEditBuffer<'_> {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<NativeEditBuffer<'static>>()
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
                .is_none_or(|n| n > 16)
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
    state: egui::text_edit::TextEditState,
    undo: egui::util::undoer::Undoer<(egui::text::CCursorRange, String)>,
}
#[derive(Default)]
struct Packet {
    received: bool,
    rejected: bool,
    before: Vec<(egui::Id, Option<NativeBefore>)>,
}
impl Packet {
    fn restore(&mut self, ctx: &egui::Context) {
        for (id, before) in self.before.drain(..) {
            if let Some(mut n) = before {
                n.state.set_undoer(n.undo);
                n.state.store(ctx, id);
            } else {
                ctx.data_mut(|d| d.remove::<egui::text_edit::TextEditState>(id));
            }
        }
    }
}
fn field(
    ui: &mut egui::Ui,
    value: &mut String,
    key: &str,
    id: egui::Id,
    allowed: bool,
    packet: &mut Packet,
    m: &mut Metrics,
) -> egui::Response {
    let before = egui::text_edit::TextEditState::load(ui.ctx(), id).map(|state| {
        let undo = state.undoer();
        NativeBefore { state, undo }
    });
    packet.before.push((id, before));
    let mut buffer = NativeEditBuffer {
        value,
        rejected: packet.rejected,
        received: false,
    };
    let r = ui.add_enabled(
        allowed,
        controls::singleline(ui, &mut buffer)
            .id(id)
            .desired_width(ui.available_width()),
    );
    packet.received |= buffer.received;
    packet.rejected |= buffer.rejected;
    m.fields.push((key.into(), r.rect, id, ui.clip_rect()));
    r
}
fn router_name(id: Option<core::RouterId>) -> String {
    id.map_or_else(|| "None available".into(), |id| format!("Router {id:?}"))
}
fn connectivity(value: core::Connectivity) -> &'static str {
    match value {
        core::Connectivity::ForwardingAvailable => "Modeled forwarding available",
        core::Connectivity::ClientLinkDown => "Client link is down",
        core::Connectivity::GatewayOutsideLan => "Gateway is outside this LAN",
        core::Connectivity::GatewayNotUsableHost => "Gateway is not a usable host address",
        core::Connectivity::UnknownGateway => "Gateway is not a supplied router or virtual address",
        core::Connectivity::NoActiveRouter => "No Active router is available",
        core::Connectivity::RouterPoweredOff => "Router is powered off",
        core::Connectivity::RouterLanDown => "Router LAN link is down",
        core::Connectivity::WanLinkDown => "Router WAN link is down",
        core::Connectivity::NoUpstreamRoute => "Router has no modeled upstream route",
    }
}
fn role(s: &core::State, id: core::RouterId) -> String {
    s.router_status(id)
        .map(|r| {
            format!(
                "{:?} · priority {} (effective {})",
                r.role, r.priority, r.effective_priority
            )
        })
        .unwrap_or_else(|e| e.to_string())
}
fn action_text(a: &work::Action) -> String {
    match a {
        work::Action::Router { router, config } => format!(
            "Router {router:?}: participation {}, priority {}, preempt {}, WAN decrement {}",
            if config.participating {
                "Enabled"
            } else {
                "Disabled"
            },
            config.priority,
            if config.preempt {
                "Enabled"
            } else {
                "Disabled"
            },
            config
                .track_wan_decrement
                .map(|n| n.to_string())
                .unwrap_or_else(|| "off".into())
        ),
        work::Action::Gateway { client, gateway } => {
            format!("Client {client}: default gateway {gateway}")
        }
    }
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
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
fn notice_result(
    notice: &mut Option<String>,
    dirty: &mut bool,
    result: Result<(), String>,
) -> bool {
    match result {
        Ok(()) => {
            *dirty = true;
            *notice = None;
            true
        }
        Err(e) => {
            *notice = Some(e);
            false
        }
    }
}
fn publish_scope(
    attempt: &mut progress::Attempt,
    source: &pack::Scenario,
    change: Option<work::binding::Change>,
    mode: Option<bool>,
) -> Result<(), String> {
    match (change, mode) {
        (Some(change), None) => work::binding::change(attempt, source, change),
        (None, Some(challenge)) => work::binding::set_challenge(attempt, source, challenge),
        _ => Err("Choose one navigation or mode action at a time; previous work retained".into()),
    }
}
impl LabApp {
    fn hsrp_mode_applied(&mut self, i: usize) {
        let _ = self.completion_index.rekey_metadata(
            &self.progress.attempts[&self.scenarios[i].id],
            &self.scenarios[i],
        );
        // Preserve the accepted shared-header cleanup, after admission succeeds.
        self.help_open = false;
        self.script_job = None;
        if let Some(view) = &mut self.ipv4_view {
            view.hide_guidance();
        }
        self.invalidate_apache_presentation();
        self.nat_presentation.clear();
        self.rpm_presentation.clear();
        self.nfs_presentation.clear();
        self.ether_presentation.clear();
        self.physical_presentation.clear();
        self.cron_presentation.clear();
        self.one_off_presentation.clear();
        self.backup_presentation.clear();
    }
    pub(super) fn hsrp_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let borrowed = &self.scenarios[i];
        if let Err(e) = work::context(&self.progress.attempts[&borrowed.id], borrowed) {
            ui.label(format!("HSRP work retained: {e}"));
            return;
        }
        let source_id = borrowed.id.clone();
        let root = ui.ctx().cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let text_packet = ui.input(|i| {
            i.events
                .iter()
                .any(|e| matches!(e, egui::Event::Text(_) | egui::Event::Paste(_)))
        });
        // Inspect borrowed raw payloads before egui normalizes CR/LF or stops
        // consuming single-line events at Enter. No event text is copied.
        let invalid_packet = ui.input(|input| {
            input.events.iter().any(|event| match event {
                egui::Event::Text(text) | egui::Event::Paste(text) => {
                    text.len() > 16
                        || text
                            .chars()
                            .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
                }
                _ => false,
            })
        });
        let after_return = ui.input(|input| {
            let mut returned = false;
            input.events.iter().any(|event| {
                if matches!(event, egui::Event::Key { key:egui::Key::Enter, pressed:true, modifiers, .. }
                    if modifiers.matches_logically(egui::Modifiers::NONE)) {
                    returned = true;
                }
                returned && matches!(event, egui::Event::Text(text) | egui::Event::Paste(text) if !text.is_empty())
            })
        });
        let inherited = self.hsrp_presentation.held;
        if !ui.input(|i| i.pointer.primary_down()) {
            self.hsrp_presentation.held = false;
        }
        let blocked = inherited || self.hsrp_presentation.veto == Some(root);
        for id in self.hsrp_presentation.retired.drain(..) {
            ui.ctx()
                .data_mut(|d| d.remove::<egui::text_edit::TextEditState>(id));
        }
        let phase = (
            self.selected,
            self.reset_pending,
            self.progress.attempts[&source_id].challenge,
        );
        self.hsrp_presentation.pending_mode = None;
        let challenge = ui
            .add_enabled_ui(!text_packet && !blocked, |ui| self.workspace_header(ui, i))
            .inner;
        let mode_requested = self.hsrp_presentation.pending_mode.take();
        if phase
            != (
                self.selected,
                self.reset_pending,
                self.progress.attempts[&source_id].challenge,
            )
        {
            return;
        }
        let source = &self.scenarios[i];
        let engine::Goal::Hsrp { definition } = &source.goal else {
            return;
        };
        let active = ui.is_enabled()
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let attempt = self.progress.attempts.get_mut(&source.id).unwrap();
        if let Err(e) = work::context(attempt, source) {
            ui.label(e);
            return;
        }
        let engine::State::Hsrp { simulation: s } = &attempt.state else {
            return;
        };
        let view = if challenge { View::Assignment } else { s.view };
        let practice = view == View::Example;
        let owner = (source.id.clone(), source.version, attempt.visited, practice);
        if self.hsrp_presentation.owner.as_ref() != Some(&owner) {
            self.hsrp_presentation.owner = Some(owner);
            self.hsrp_presentation.events.clear();
            self.hsrp_presentation.memo = None;
        }
        let mut scope_requested = None;
        let mut scope_conflict = false;
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (View::Introduction, "introduction", "Introduction"),
                    (View::Example, "example", "Separate example · ungraded"),
                    (View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(
                        active && !text_packet && !blocked,
                        egui::Button::new(label).selected(view == v),
                    );
                    record(&mut self.metrics.hsrp, key, &r, ui);
                    if r.clicked() {
                        scope_conflict |= scope_requested
                            .replace(work::binding::Change::View(v))
                            .is_some();
                    }
                }
            });
        }
        if view == View::Introduction {
            ui.label(&definition.lesson.prerequisite_note);
            for c in &definition.lesson.concepts {
                ui.strong(&c.title);
                ui.label(&c.text);
            }
            ui.label("Use the separate example, then configure the complete assignment. Real HSRP uses timers and packet delivery; these displayed roles are already settled simulator outcomes.");
            // Introduction currently has no help disclosures or configuration
            // controls. Its complete visible scope has now been rendered.
            #[cfg(test)]
            {
                self.metrics.hsrp.scope_intents =
                    (scope_requested.is_some(), mode_requested.is_some());
            }
            if scope_conflict {
                self.notice =
                    Some("Choose one navigation action at a time; previous work retained".into());
            } else if active
                && !text_packet
                && !blocked
                && (scope_requested.is_some() || mode_requested.is_some())
            {
                let mode = mode_requested.is_some();
                if notice_result(
                    &mut self.notice,
                    &mut self.dirty,
                    publish_scope(attempt, source, scope_requested, mode_requested),
                ) && mode
                {
                    self.hsrp_mode_applied(i);
                }
                ui.ctx().request_repaint();
            }
            return;
        }
        let policy = if practice {
            &definition.lesson.policy
        } else {
            &definition.policy
        };
        if practice {
            ui.label(&definition.lesson.description);
            let mut selected = s.selected_example;
            let picker = ui
                .add_enabled_ui(active && !text_packet && !blocked, |ui| {
                    controls::dropdown(ui, ("hsrp-worked", &source.id), ui.available_width())
                        .selected_text(&definition.lesson.worked[selected].title)
                        .show_ui(ui, |ui| {
                            controls::fit_popup_contents(ui);
                            for (n, w) in definition.lesson.worked.iter().enumerate() {
                                let r = ui.selectable_value(&mut selected, n, &w.title);
                                record(
                                    &mut self.metrics.hsrp,
                                    &format!("worked-choice-{n}"),
                                    &r,
                                    ui,
                                );
                            }
                        })
                })
                .inner;
            self.metrics.terminal_example_picker = Some(picker.response.rect);
            record(
                &mut self.metrics.hsrp,
                "worked-picker",
                &picker.response,
                ui,
            );
            if selected != s.selected_example {
                scope_conflict |= scope_requested
                    .replace(work::binding::Change::SelectExample(selected))
                    .is_some();
            }
            ui.label(&definition.lesson.worked[s.selected_example].why);
            let mut action = None;
            ui.horizontal_wrapped(|ui| {
                let r = ui.add_enabled(
                    active && !text_packet && !blocked,
                    button("Insert selected settings"),
                );
                record(&mut self.metrics.hsrp, "insert", &r, ui);
                self.metrics.insert = Some(r.rect);
                if r.clicked() {
                    action = Some(false);
                }
                let r = ui.add_enabled(
                    active && !text_packet && !blocked,
                    button("Reset example only"),
                );
                record(&mut self.metrics.hsrp, "reset-example", &r, ui);
                if r.clicked() {
                    action = Some(true);
                }
            });
            if let Some(reset) = action {
                // Use the held saved selection until any picker intent is
                // independently admitted; never insert an uncommitted choice.
                if scope_requested.is_some() {
                    scope_conflict = true;
                    self.notice =
                        Some("Choose one example action at a time; previous work retained".into());
                } else {
                    scope_requested = Some(if reset {
                        work::binding::Change::ResetExample
                    } else {
                        work::binding::Change::InsertExample(s.selected_example)
                    });
                }
            }
            ui.label(format!("Try a change: {}", definition.lesson.try_change));
            ui.label(action_text(&definition.lesson.experiment));
            ui.label(format!(
                "Repair: {}",
                action_text(&definition.lesson.repair)
            ));
        } else {
            let r = ui.label(&source.objective);
            self.metrics.hsrp.goal = Some(r.rect);
            record(&mut self.metrics.hsrp, "goal", &r, ui);
        }
        let current = if practice { &s.example } else { &s.current };
        let a = current.router(core::RouterId::A);
        let b = current.router(core::RouterId::B);
        let left = [
            a.label.clone(),
            a.address.to_string(),
            role(current, core::RouterId::A),
        ];
        let right = [
            b.label.clone(),
            b.address.to_string(),
            role(current, core::RouterId::B),
        ];
        let topology = controls::topology_frame(ui).show(ui, |ui| {
            controls::topology_pair(
                ui,
                [&left[0], &left[1], &left[2]],
                [&right[0], &right[1], &right[2]],
                palette(ui).muted,
                &format!(
                    "Connected LAN /{} · HSRPv1 group {} · virtual gateway {}",
                    current.prefix(),
                    current.group(),
                    current.virtual_ip()
                ),
            )
        });
        self.metrics.hsrp.topology = Some((topology.response.rect, ui.clip_rect()));
        let original = if practice {
            s.example_draft.clone()
        } else {
            s.draft.clone()
        };
        let mut draft = original.clone();
        let mut packet = Packet {
            rejected: invalid_packet,
            ..Default::default()
        };
        let generation = self.hsrp_presentation.generation[usize::from(practice)];
        self.hsrp_presentation.fields.clear();
        for (index, id) in core::RouterId::ALL.into_iter().enumerate() {
            let editable = policy.editable_routers.contains(&id) && active && !blocked;
            let name = &current.router(id).label;
            ui.strong(format!(
                "{name} · router {id:?}{}",
                if editable { "" } else { " · protected" }
            ));
            let router = &mut draft.routers[index];
            egui::Grid::new(("hsrp-router", &source.id, practice, index))
                .num_columns(2)
                .max_col_width((ui.available_width() - GAP) * 0.5)
                .spacing([GAP, 6.])
                .show(ui, |ui| {
                    ui.label("Participation");
                    ui.add_enabled_ui(editable && !text_packet, |ui| {
                        let r = ui.checkbox(&mut router.participating, "Enabled");
                        record(
                            &mut self.metrics.hsrp,
                            &format!("router-{id:?}-participating"),
                            &r,
                            ui,
                        );
                    });
                    ui.end_row();
                    for (key, label, value) in [
                        ("priority", "Priority 1–255", &mut router.priority),
                        ("decrement", "WAN tracking decrement", &mut router.decrement),
                    ] {
                        ui.label(label);
                        let k = format!("router-{id:?}-{key}");
                        let field_id = egui::Id::new((
                            "hsrp-field",
                            &source.id,
                            source.version,
                            attempt.visited,
                            practice,
                            generation,
                            &k,
                        ));
                        field(
                            ui,
                            value,
                            &k,
                            field_id,
                            editable,
                            &mut packet,
                            &mut self.metrics.hsrp,
                        );
                        self.hsrp_presentation.fields.push(field_id);
                        ui.end_row();
                    }
                    ui.label("Preemption");
                    ui.add_enabled_ui(editable && !text_packet, |ui| {
                        let r = ui.checkbox(&mut router.preempt, "Preempt");
                        record(
                            &mut self.metrics.hsrp,
                            &format!("router-{id:?}-preempt"),
                            &r,
                            ui,
                        );
                    });
                    ui.end_row();
                    ui.label("WAN link tracking");
                    ui.add_enabled_ui(editable && !text_packet, |ui| {
                        let r = ui.checkbox(&mut router.track, "Track link");
                        record(
                            &mut self.metrics.hsrp,
                            &format!("router-{id:?}-track"),
                            &r,
                            ui,
                        );
                    });
                    ui.end_row();
                });
        }
        ui.strong("Client default gateways");
        egui::Grid::new(("hsrp-clients", &source.id, practice))
            .num_columns(2)
            .max_col_width((ui.available_width() - GAP) * 0.5)
            .spacing([GAP, 6.])
            .show(ui, |ui| {
                for c in &mut draft.clients {
                    let info = current.clients().iter().find(|x| x.id == c.id).unwrap();
                    ui.label(format!("{} · {}", info.label, info.address));
                    let key = format!("client-{}", c.id);
                    let id = egui::Id::new((
                        "hsrp-field",
                        &source.id,
                        source.version,
                        attempt.visited,
                        practice,
                        generation,
                        &key,
                    ));
                    field(
                        ui,
                        &mut c.gateway,
                        &key,
                        id,
                        policy.editable_clients.contains(&c.id) && active && !blocked,
                        &mut packet,
                        &mut self.metrics.hsrp,
                    );
                    self.hsrp_presentation.fields.push(id);
                    ui.end_row();
                }
            });
        let rejected = packet.rejected || (text_packet && (!packet.received || after_return));
        if rejected {
            packet.restore(ui.ctx());
            self.hsrp_presentation.veto = Some(root);
            self.hsrp_presentation.held |= ui.input(|i| i.pointer.primary_down());
            self.notice=Some("That edit was not applied. Previous fields and selection are retained; no settings or probe ran.".into());
        }
        let draft_changed = draft != original;
        // All actions wait for a later clean gesture; no text packet can apply an old prefix.
        let allowed = active
            && !text_packet
            && !blocked
            && !rejected
            && !draft_changed
            && self.hsrp_presentation.veto != Some(root);
        let mut apply_requested = false;
        let mut check_requested = false;
        ui.horizontal_wrapped(|ui| {
            let r = ui.add_enabled(allowed, primary(ui, "Apply configuration"));
            record(&mut self.metrics.hsrp, "apply", &r, ui);
            apply_requested = r.clicked();
            let r = ui.add_enabled(
                allowed,
                primary(
                    ui,
                    if practice {
                        "Check example"
                    } else {
                        "Check work"
                    },
                ),
            );
            self.metrics.check = Some(r.rect);
            record(&mut self.metrics.hsrp, "check", &r, ui);
            check_requested = r.clicked();
        });
        let current_completion = if practice {
            false
        } else {
            let rows = self.hsrp_presentation.rows(definition, s, false);
            !rows.is_empty() && rows.iter().all(|r| r.0)
        };
        let is_checked = if practice {
            s.example_checked
        } else {
            s.checked
        };
        if is_checked {
            let rows = self.hsrp_presentation.rows(definition, s, practice);
            let failed = rows.iter().filter(|r| !r.0).count();
            let summary = ui.strong(if failed == 0 {
                format!(
                    "Passed · all{} current and failure-probe checks",
                    rows.len()
                )
            } else {
                format!("Needs attention · {failed} of{} checks", rows.len())
            });
            self.metrics.hsrp.summary = Some((rows.len(), failed, summary.rect));
            for (ok, text) in rows.iter().filter(|r| !r.0).take(3) {
                let r = ui.colored_label(palette(ui).error, format!("Needs attention · {text}"));
                self.metrics.hsrp.rows.push((*ok, r.rect));
            }
            controls::collapsing_header(ui, "All current and failure checks").show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt(("hsrp-rows", &source.id, practice))
                    .max_height(ui.text_style_height(&egui::TextStyle::Body) * 10.)
                    .show(ui, |ui| {
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
                            self.metrics.hsrp.rows.push((*ok, r.rect));
                        }
                    });
            });
        }
        if let Some(e) = &self.notice {
            ui.label(e);
        }
        let mut probe_requested = None;
        let inspection=controls::collapsing_header(ui,"Temporary failure inspection · does not change configured work").show(ui,|ui|{
            ui.horizontal_wrapped(|ui|{
                for id in core::RouterId::ALL {for(up,label)in[(false,"fail"),(true,"return")] {for(kind,name)in[(0,"power"),(1,"LAN"),(2,"WAN")] {
                    let key=format!("probe-{id:?}-{name}-{up}");
                    let r=ui.add_enabled(allowed&&self.hsrp_presentation.events.len()<8,button(&format!("{id:?} {name} {label}")));
                    record(&mut self.metrics.hsrp,&key,&r,ui);
                    if r.clicked(){let mut events=self.hsrp_presentation.events.clone();events.push(match kind{0=>core::FaultEvent::RouterPower{router:id,up},1=>core::FaultEvent::LanLink{router:id,up},_=>core::FaultEvent::WanLink{router:id,up}});probe_requested=Some(events);}
                }}}
                let r=ui.add_enabled(allowed,button("Clear temporary probe"));record(&mut self.metrics.hsrp,"clear-probe",&r,ui);if r.clicked(){probe_requested=Some(vec![]);}
            });
            if !self.hsrp_presentation.events.is_empty(){match s.inspect(definition,&self.hsrp_presentation.events,practice){
                Ok(observed)=>{let r=ui.strong(format!("Temporary settled observation: Active {}, Standby {}. Configured work remains Active {}.",router_name(observed.active()),router_name(observed.standby()),router_name(if practice{s.example.active()}else{s.current.active()})));self.metrics.hsrp.probe=Some(r.rect);for c in observed.clients(){if let Ok(row)=observed.client_status(c.id){ui.label(format!("{}: {} via {}",c.label,connectivity(row.outcome),router_name(row.via)));}}},Err(e)=>{ui.label(e);}
            }}
        });
        record(
            &mut self.metrics.hsrp,
            "inspection",
            &inspection.header_response,
            ui,
        );
        #[cfg(test)]
        if inspection.body_returned.is_some() {
            self.metrics.hsrp.disclosures.push("inspection");
        }
        let mut guidance = guidance_intent(ui, &inspection.header_response);
        let mut hint_requested = false;
        ui.push_id(
            (
                "hsrp-help",
                &source.id,
                source.version,
                attempt.visited,
                challenge,
            ),
            |ui| {
                let help = controls::collapsing_header(ui, "Optional help and model limits").show(
                    ui,
                    |ui| {
                        ui.label(core::HELP);
                        if !practice {
                            if attempt.hints < source.hints.len() {
                                let r = ui.add_enabled(allowed, button("Show a hint"));
                                record(&mut self.metrics.hsrp, "show-hint", &r, ui);
                                guidance |= guidance_intent(ui, &r);
                                hint_requested = r.clicked();
                            }
                            for (n, h) in source.hints.iter().take(attempt.hints).enumerate() {
                                ui.label(format!("Hint {}: {h}", n + 1));
                            }
                            let worked = controls::collapsing_header(
                                ui,
                                "Worked approach · reveals a complete solution",
                            )
                            .show(ui, |ui| {
                                ui.label(&source.explanation);
                                for command in &source.solution {
                                    if let Some(raw) = command.strip_prefix(work::PREFIX)
                                        && let Ok(action) =
                                            serde_json::from_str::<work::Action>(raw)
                                    {
                                        ui.label(action_text(&action));
                                    }
                                }
                            });
                            record(
                                &mut self.metrics.hsrp,
                                "worked-approach",
                                &worked.header_response,
                                ui,
                            );
                            #[cfg(test)]
                            if worked.body_returned.is_some() {
                                self.metrics.hsrp.disclosures.push("worked-approach");
                            }
                            guidance |= guidance_intent(ui, &worked.header_response);
                            let refs =
                                controls::collapsing_header(ui, "References").show(ui, |ui| {
                                    for link in &source.sources {
                                        ui.hyperlink_to(link, link);
                                    }
                                });
                            record(
                                &mut self.metrics.hsrp,
                                "references",
                                &refs.header_response,
                                ui,
                            );
                            #[cfg(test)]
                            if refs.body_returned.is_some() {
                                self.metrics.hsrp.disclosures.push("references");
                            }
                            guidance |= guidance_intent(ui, &refs.header_response);
                        }
                    },
                );
                record(
                    &mut self.metrics.hsrp,
                    "optional-help",
                    &help.header_response,
                    ui,
                );
                #[cfg(test)]
                if help.body_returned.is_some() {
                    self.metrics.hsrp.disclosures.push("optional-help");
                }
                guidance |= guidance_intent(ui, &help.header_response);
            },
        );
        // The held Session was read-only for this render. Only now may a request
        // create a bounded candidate, after the complete guidance/input barrier.
        let scope_action = scope_requested.is_some() || mode_requested.is_some();
        let requested = scope_action
            || draft_changed
            || apply_requested
            || check_requested
            || hint_requested
            || probe_requested.is_some();
        #[cfg(test)]
        {
            self.metrics.hsrp.intents =
                (apply_requested, check_requested, probe_requested.is_some());
            self.metrics.hsrp.guidance = guidance;
            self.metrics.hsrp.scope_intents = (scope_requested.is_some(), mode_requested.is_some());
        }
        let conflicting = scope_conflict
            || usize::from(scope_requested.is_some())
                + usize::from(mode_requested.is_some())
                + usize::from(apply_requested)
                + usize::from(check_requested)
                + usize::from(probe_requested.is_some())
                > 1;
        // A normal help press must remain enabled for its later release. Latch
        // only competing work/input; current dispatch is vetoed by guidance either way.
        if guidance
            && (text_packet
                || rejected
                || draft_changed
                || scope_action
                || apply_requested
                || check_requested
                || probe_requested.is_some()
                || blocked)
        {
            self.hsrp_presentation.veto = Some(root);
            self.hsrp_presentation.held |= ui.input(|i| i.pointer.primary_down());
        }
        if conflicting {
            self.notice = Some("Choose one action at a time; previous work retained".into());
        }
        if allowed
            && !guidance
            && !conflicting
            && let Some(events) = probe_requested
        {
            match s.inspect(definition, &events, practice) {
                Ok(_) => {
                    self.hsrp_presentation.events = events;
                    self.notice = None;
                }
                Err(e) => self.notice = Some(e),
            }
        }
        // End the render's read-only Session borrow before source-bound mutation.
        if !rejected && draft_changed && !guidance && !scope_action {
            let result = work::binding::change(
                attempt,
                source,
                work::binding::Change::Draft { draft, practice },
            );
            if !notice_result(&mut self.notice, &mut self.dirty, result) {
                packet.restore(ui.ctx());
                self.hsrp_presentation.veto = Some(root);
                self.hsrp_presentation.held |= ui.input(|i| i.pointer.primary_down());
            }
        } else if allowed && !guidance && !conflicting {
            if scope_action {
                let reset = matches!(&scope_requested, Some(work::binding::Change::ResetExample));
                let mode = mode_requested.is_some();
                if notice_result(
                    &mut self.notice,
                    &mut self.dirty,
                    publish_scope(attempt, source, scope_requested, mode_requested),
                ) {
                    if reset {
                        self.hsrp_presentation.reset_example();
                    }
                    if mode {
                        self.hsrp_mode_applied(i);
                        ui.ctx().request_repaint();
                        return;
                    }
                }
            } else if apply_requested {
                if notice_result(
                    &mut self.notice,
                    &mut self.dirty,
                    work::binding::change(
                        attempt,
                        source,
                        work::binding::Change::Apply { practice },
                    ),
                ) {
                    self.hsrp_presentation.events.clear();
                    self.hsrp_presentation.memo = None;
                }
            } else if check_requested {
                notice_result(
                    &mut self.notice,
                    &mut self.dirty,
                    work::binding::change(
                        attempt,
                        source,
                        work::binding::Change::Check { practice },
                    ),
                );
            }
        }
        if draft_changed && (guidance || scope_action) {
            packet.restore(ui.ctx());
            self.notice = Some(
                "Choose one field change or help action at a time; previous work retained".into(),
            );
        }
        if hint_requested && allowed && !conflicting && !scope_action {
            notice_result(
                &mut self.notice,
                &mut self.dirty,
                work::binding::request_hint(attempt, source),
            );
        }
        if requested || guidance || rejected {
            ui.ctx().request_repaint();
        }
        if !practice {
            self.metrics.completion = if requested {
                progress::current_solved(attempt, source)
            } else {
                current_completion
            };
            if self.metrics.completion && allowed && !requested && !guidance {
                self.next_unit_lab(ui, i);
                if let Some(r) = self.metrics.next_unit {
                    self.metrics
                        .hsrp
                        .controls
                        .push(("next".into(), r, ui.clip_rect()));
                }
            }
        }
    }
}
