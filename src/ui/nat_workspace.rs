//! Native Basic NAT address workbench. No host networking or command bridge.
use super::*;
use dlms_learning_lab::{nat as core, nat_workspace as work};
pub(super) mod memo;

#[derive(Default)]
pub(super) struct Presentation {
    pub memo: memo::Memo,
    row_scope: Option<(String, bool, u64)>,
    generation: u64,
    next_token: u64,
    tokens: Vec<u64>,
    effect: Option<core::Effect>,
}
impl Presentation {
    pub(super) fn observed_effect(&self) -> Option<&core::Effect> {
        self.effect.as_ref()
    }
    pub fn clear(&mut self) {
        self.row_scope = None;
        self.tokens.clear();
        self.effect = None;
        self.generation = self.generation.wrapping_add(1);
        // Entries remain optimization-only: every access compares complete inputs.
    }
    fn bind(&mut self, id: &str, challenge: bool, visited: u64, rows: usize) {
        if self
            .row_scope
            .as_ref()
            .is_none_or(|k| k.0 != id || k.1 != challenge || k.2 != visited)
        {
            self.clear();
            self.row_scope = Some((id.into(), challenge, visited));
        }
        while self.tokens.len() < rows {
            self.tokens.push(self.next_token);
            self.next_token += 1;
        }
        self.tokens.truncate(rows);
    }
}
/// A rejected replacement paste rolls back its selection deletion too. No
/// snapshot is taken on idle paints, or before an actual text mutation.
struct InputBuffer<'a> {
    value: &'a mut String,
    cap: usize,
    old: Option<String>,
    rejected: bool,
}
impl InputBuffer<'_> {
    fn remember(&mut self) {
        if self.old.is_none() {
            self.old = Some(self.value.clone());
        }
    }
    fn finish(mut self) -> bool {
        if self.rejected {
            if let Some(old) = self.old.take() {
                *self.value = old;
            }
            false
        } else {
            self.old.is_some_and(|s| s != *self.value)
        }
    }
}
impl egui::TextBuffer for InputBuffer<'_> {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<InputBuffer<'static>>()
    }
    fn is_mutable(&self) -> bool {
        true
    }
    fn as_str(&self) -> &str {
        self.value
    }
    fn insert_text(&mut self, s: &str, at: usize) -> usize {
        if s.is_empty() {
            return 0;
        }
        if self
            .value
            .len()
            .checked_add(s.len())
            .is_none_or(|n| n > self.cap)
        {
            self.rejected = true;
            return 0;
        }
        self.remember();
        egui::TextBuffer::insert_text(self.value, s, at)
    }
    fn delete_char_range(&mut self, r: std::ops::Range<usize>) {
        if !r.is_empty() {
            self.remember();
            egui::TextBuffer::delete_char_range(self.value, r);
        }
    }
}
struct FieldSpec<'a> {
    key: &'a str,
    label: &'a str,
    cap: usize,
}
fn edit(
    ui: &mut egui::Ui,
    metrics: &mut Metrics,
    scope: egui::Id,
    field: FieldSpec<'_>,
    value: &mut String,
    focus: &mut bool,
) -> bool {
    let FieldSpec { key, label, cap } = field;
    controls::form_row(ui, label, |ui, width| {
        let id = scope.with(key);
        let mut b = InputBuffer {
            value,
            cap,
            old: None,
            rejected: false,
        };
        let r = ui.add_sized(
            [width, controls::height(ui)],
            controls::singleline(ui, &mut b).id(id),
        );
        r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, r.enabled(), label));
        if *focus && r.enabled() {
            r.request_focus();
            *focus = false;
        }
        metrics
            .nat_fields
            .push((key.into(), r.rect, r.id, r.enabled(), ui.clip_rect()));
        b.finish()
    })
    .inner
}
fn action(
    ui: &mut egui::Ui,
    metrics: &mut Metrics,
    key: &str,
    text: &str,
    primary_button: bool,
) -> bool {
    let width = controls::button_width(ui, text);
    let r = if primary_button {
        ui.add_sized([width, controls::height(ui)], primary(ui, text))
    } else {
        ui.add_sized([width, controls::height(ui)], button(text))
    };
    metrics
        .nat_controls
        .push((key.into(), r.rect, r.enabled(), ui.clip_rect()));
    r.clicked()
}
fn selector(
    ui: &mut egui::Ui,
    metrics: &mut Metrics,
    scope: egui::Id,
    key: &str,
    label: &str,
    value: &mut Option<String>,
    options: &std::collections::BTreeMap<String, std::net::Ipv4Addr>,
) -> bool {
    let mut changed = false;
    controls::form_row(ui, label, |ui, width| {
        let combo = controls::dropdown(ui, scope.with(key), width)
            .selected_text(value.as_deref().unwrap_or("Choose"));
        let inner = combo.show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            for (id, addr) in options {
                let r = ui.selectable_label(value.as_ref() == Some(id), format!("{id} · {addr}"));
                if r.clicked() && value.as_ref() != Some(id) {
                    *value = Some(id.clone());
                    changed = true;
                }
                metrics.nat_choices.push((format!("{key}:{id}"), r.rect));
            }
            metrics.nat_popup = Some(controls::popup_content_clip(ui));
        });
        metrics.nat_controls.push((
            key.into(),
            inner.response.rect,
            inner.response.enabled(),
            ui.clip_rect(),
        ));
    });
    changed
}
// One complete static map should be inspectable without inner scrolling.
// Measure the same label/control layout and reserve the actual scrollbar width.
fn static_row_height(ui: &egui::Ui) -> f32 {
    let control = controls::height(ui);
    let gap = ui.spacing().item_spacing.y;
    let width = (ui.available_width() - ui.spacing().scroll.allocated_width()).max(1.);
    let labels = (width * 0.4).clamp(90., 165.);
    let minimum_value =
        ui.text_style_height(&egui::TextStyle::Body) * 6. + 2. * f32::from(controls::PAD_X);
    let stacked = width < labels + ui.spacing().item_spacing.x + minimum_value;
    let font = egui::TextStyle::Body.resolve(ui.style());
    let color = palette(ui).text;
    let row = |text: &str| {
        let label = ui.fonts_mut(|f| {
            f.layout(
                text.into(),
                font.clone(),
                color,
                if stacked { width } else { labels },
            )
            .size()
            .y
            .ceil()
        });
        if stacked {
            label + gap + control
        } else {
            label.max(control)
        }
    };
    let heading = if width
        < controls::button_width(ui, "Remove")
            + ui.spacing().item_spacing.x
            + ui.fonts_mut(|f| {
                f.layout_no_wrap("Map 32".into(), font.clone(), color)
                    .size()
                    .x
            }) {
        control + gap + ui.text_style_height(&egui::TextStyle::Body)
    } else {
        control
    };
    heading + row("Inside identity") + row("Global address") + 4. * gap
}
fn policy_panel(
    ui: &mut egui::Ui,
    d: &mut work::Draft,
    p: &mut Presentation,
    metrics: &mut Metrics,
    scope: egui::Id,
    focus: &mut bool,
) -> (bool, bool) {
    ui.strong("Policy editor");
    ui.label("Apply changes policy only. Existing dynamic mappings stay until explicitly cleared; incompatible changes can require clearing them first.");
    let mut changed = false;
    let mut remove = None;
    ui.horizontal_wrapped(|ui| {
        ui.strong("Static maps");
        ui.add_enabled_ui(d.statics.len() < 32, |ui| {
            if action(ui, metrics, "add-static", "Add static map", false) {
                d.statics.push(work::StaticRow {
                    inside: String::new(),
                    global: String::new(),
                });
                p.tokens.push(p.next_token);
                p.next_token += 1;
                changed = true;
            }
        });
    });
    let row_height = static_row_height(ui);
    let static_list = egui::ScrollArea::vertical()
        .id_salt(scope.with("static-list"))
        .min_scrolled_height(row_height)
        .max_height(row_height.max(240.))
        .show(ui, |ui| {
            for (n, row) in d.statics.iter_mut().enumerate() {
                let token = p.tokens[n];
                ui.push_id(scope.with(("static", token)), |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("Map {}", n + 1));
                        if action(ui, metrics, &format!("remove:{token}"), "Remove", false) {
                            remove = Some(n);
                        }
                    });
                    changed |= edit(
                        ui,
                        metrics,
                        scope.with(("static", token)),
                        FieldSpec {
                            key: &format!("static:{token}:inside"),
                            label: "Inside identity",
                            cap: 32,
                        },
                        &mut row.inside,
                        focus,
                    );
                    changed |= edit(
                        ui,
                        metrics,
                        scope.with(("static", token)),
                        FieldSpec {
                            key: &format!("static:{token}:global"),
                            label: "Global address",
                            cap: 64,
                        },
                        &mut row.global,
                        focus,
                    );
                    ui.separator();
                });
            }
        });
    metrics.nat_static_viewport = Some(static_list.inner_rect);
    metrics.nat_static_scroll = Some((
        static_list.id,
        static_list.state.offset,
        static_list.content_size,
    ));
    if let Some(n) = remove {
        d.statics.remove(n);
        p.tokens.remove(n);
        d.selected_row = None;
        changed = true;
    }
    let toggle = ui.checkbox(&mut d.dynamic_enabled, "Enable dynamic address mapping");
    changed |= toggle.changed();
    metrics.nat_controls.push((
        "dynamic".into(),
        toggle.rect,
        toggle.enabled(),
        ui.clip_rect(),
    ));
    // Retain disabled text verbatim. It has no policy meaning until enabled.
    ui.add_enabled_ui(d.dynamic_enabled, |ui| {
        changed |= edit(
            ui,
            metrics,
            scope,
            FieldSpec {
                key: "network",
                label: "Eligible network",
                cap: 64,
            },
            &mut d.network,
            focus,
        );
        changed |= edit(
            ui,
            metrics,
            scope,
            FieldSpec {
                key: "prefix",
                label: "Prefix length (0–32)",
                cap: 16,
            },
            &mut d.prefix,
            focus,
        );
        changed |= edit(
            ui,
            metrics,
            scope,
            FieldSpec {
                key: "first",
                label: "Pool first address",
                cap: 64,
            },
            &mut d.first,
            focus,
        );
        changed |= edit(
            ui,
            metrics,
            scope,
            FieldSpec {
                key: "last",
                label: "Pool last address",
                cap: 64,
            },
            &mut d.last,
            focus,
        );
    });
    let apply = action(ui, metrics, "apply", "Apply policy", false);
    (changed, apply)
}
fn requests_panel(
    ui: &mut egui::Ui,
    d: &mut work::Draft,
    metrics: &mut Metrics,
    scope: egui::Id,
    focus: &mut bool,
) -> (bool, Option<Result<core::Action, String>>) {
    ui.strong("Address observations");
    ui.label("Outbound may create a dynamic mapping. Reverse never allocates. These are simulated address rewrites, not packet delivery or connectivity tests.");
    let mut changed = selector(
        ui,
        metrics,
        scope,
        "inside",
        "Inside identity",
        &mut d.inside,
        &d.context.inside,
    );
    changed |= selector(
        ui,
        metrics,
        scope,
        "peer",
        "Outside peer",
        &mut d.peer,
        &d.context.outside,
    );
    changed |= edit(
        ui,
        metrics,
        scope,
        FieldSpec {
            key: "inbound-global",
            label: "Reverse global address",
            cap: 64,
        },
        &mut d.inbound_global,
        focus,
    );
    let mut request = None;
    ui.horizontal_wrapped(|ui| {
        if action(ui, metrics, "outbound", "Outbound", false) {
            request = Some(
                d.inside
                    .as_ref()
                    .zip(d.peer.as_ref())
                    .map(|(i, p)| core::Action::Outbound {
                        inside: i.clone(),
                        peer: p.clone(),
                    })
                    .ok_or_else(|| "Choose an inside identity and outside peer".into()),
            );
        }
        if action(ui, metrics, "inbound", "Reverse", false) {
            request = Some(
                d.peer
                    .as_ref()
                    .ok_or_else(|| "Choose an outside peer".to_owned())
                    .and_then(|p| {
                        d.inbound_global
                            .trim()
                            .parse()
                            .map(|global| core::Action::Inbound {
                                peer: p.clone(),
                                global,
                            })
                            .map_err(|_| "Reverse global needs an IPv4 address".into())
                    }),
            );
        }
        if action(ui, metrics, "clear", "Clear dynamic maps", false) {
            request = Some(Ok(core::Action::ClearDynamic {}));
        }
    });
    (changed, request)
}
pub(super) fn check_text(ok: bool, message: &str) -> String {
    let body = if ok {
        message.strip_prefix("Satisfied: ")
    } else {
        message.strip_prefix("Needs attention: ")
    }
    .unwrap_or(message);
    format!(
        "{} · {body}",
        if ok { "Verified" } else { "Needs attention" }
    )
}
pub(super) fn observation_lines(effect: &core::Effect) -> Vec<String> {
    match effect {
        core::Effect::Configured => {
            vec!["Policy applied; no new address probe was performed.".into()]
        }
        core::Effect::Cleared { removed } => vec![format!(
            "Cleared {removed} dynamic maps. Static maps are unchanged."
        )],
        core::Effect::Probe { result } => match result {
            core::ProbeResult::Translated {
                before,
                after,
                kind,
            } => vec![
                format!(
                    "{} rewrite",
                    match kind {
                        core::MappingKind::Static => "Static",
                        core::MappingKind::Dynamic => "Dynamic",
                    }
                ),
                format!("Before: {} -> {}", before.source, before.destination),
                format!("After: {} -> {}", after.source, after.destination),
            ],
            core::ProbeResult::Untranslated { addresses } => vec![format!(
                "Untranslated: {} -> {}",
                addresses.source, addresses.destination
            )],
            core::ProbeResult::NoMapping { addresses } => vec![format!(
                "No reverse mapping: {} -> {}",
                addresses.source, addresses.destination
            )],
            core::ProbeResult::PoolExhausted { addresses } => vec![format!(
                "Pool exhausted: {} -> {}",
                addresses.source, addresses.destination
            )],
        },
    }
}
fn state_panel(ui: &mut egui::Ui, s: &core::State, p: &Presentation, metrics: &mut Metrics) {
    ui.strong("Applied policy and existing maps");
    if let Some(r) = s.configuration().dynamic {
        ui.label(format!(
            "Dynamic eligibility: {}/{} · pool {}–{}",
            r.source.network, r.source.length, r.pool.first, r.pool.last
        ));
    } else {
        ui.label("Dynamic mapping disabled");
    }
    // Borrow existing state; observing or checking never creates a binding.
    let mapping = |id: &str| {
        if let Some(g) = s.configuration().statics.get(id) {
            format!("{g} · Static")
        } else if let Some(g) = s.bindings().get(id) {
            format!("{g} · Dynamic")
        } else {
            "No current mapping".into()
        }
    };
    if ui.available_width() < controls::height(ui) * 15. {
        for (id, addr) in &s.inventory().inside {
            ui.label(format!("{id} · {addr}"));
            ui.label(mapping(id));
            ui.separator();
        }
    } else {
        egui::Grid::new("nat-mapping-table")
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Inside");
                ui.strong("Local address");
                ui.strong("Existing global / type");
                ui.end_row();
                for (id, addr) in &s.inventory().inside {
                    ui.label(id);
                    ui.label(addr.to_string());
                    ui.label(mapping(id));
                    ui.end_row();
                }
            });
    }
    if let Some(effect) = &p.effect {
        ui.strong("Last explicit action");
        let r = ui.vertical(|ui| {
            for line in observation_lines(effect) {
                ui.label(line);
            }
        });
        metrics.nat_observation = Some(r.response.rect);
    }
}
// Borrow only the memo in card closures, leaving unrelated mutable UI fields free.
pub(super) fn solved_with_memo(memo: &memo::Memo, a: &Attempt, s: &Scenario) -> bool {
    if matches!(s.initial, engine::State::BasicNat { .. }) {
        memo.prepared(&s.id)
            .unwrap_or_else(|| progress::current_solved(a, s))
    } else {
        progress::current_solved(a, s)
    }
}
impl LabApp {
    pub(super) fn nat_library_prepass(&mut self) {
        let sources = [
            self.scenarios.as_slice(),
            if self.developer && self.synthetic {
                self.synthetic_catalog.as_slice()
            } else {
                &[]
            },
        ];
        self.nat_presentation.memo.retain(|id| {
            self.progress.attempts.contains_key(id)
                && sources.iter().any(|source| {
                    source
                        .iter()
                        .any(|s| s.id == id && matches!(s.initial, engine::State::BasicNat { .. }))
                })
        });
        for source in sources {
            for s in source {
                if matches!(s.initial, engine::State::BasicNat { .. })
                    && let Some(a) = self.progress.attempts.get(&s.id)
                {
                    self.nat_presentation.memo.evaluate(a, s, false);
                }
            }
        }
    }
    pub(super) fn visible_solved(&self, a: &Attempt, s: &Scenario) -> bool {
        solved_with_memo(&self.nat_presentation.memo, a, s)
    }
    pub(super) fn nat_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let challenge = self.workspace_header(ui, i);
        let active = ui.is_enabled()
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        if !active {
            self.focus_input = false;
        }
        let s = &self.scenarios[i];
        ui.heading(&s.title);
        ui.label(&s.objective);
        ui.label("Basic NAT: address-only simulation. No PAT, ports, sessions, timers, routing or security verdict.");
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        let (
            engine::State::BasicNat {
                simulation: initial,
            },
            engine::State::BasicNat { simulation },
        ) = (&s.initial, &mut a.state)
        else {
            return;
        };
        let Some(d) = a.workspace.as_mut().and_then(|w| w.nat_draft.as_mut()) else {
            ui.label(
                "The saved NAT editor is missing; reset this selected workspace to prepare it.",
            );
            return;
        };
        if !memo::draft_shape(d) {
            ui.label("The retained NAT editor exceeds its bounds or has an unknown selector. Reset this selected workspace to repair it; its original draft is retained.");
            self.nat_presentation.memo.evaluate(a, s, false);
            return;
        }
        self.nat_presentation
            .bind(&s.id, challenge, a.visited, d.statics.len());
        let scope = egui::Id::new((
            "nat-workspace",
            &s.id,
            challenge,
            self.nat_presentation.generation,
        ));
        let mut apply = false;
        let mut request = None;
        let mut changed = false;
        ui.push_id(scope, |ui| {
            ui.add_enabled_ui(active, |ui| {
                if ui.available_width() >= controls::height(ui) * 18. {
                    ui.columns(2, |columns| {
                        let (c, p) = policy_panel(
                            &mut columns[0],
                            d,
                            &mut self.nat_presentation,
                            &mut self.metrics,
                            scope,
                            &mut self.focus_input,
                        );
                        changed |= c;
                        apply = p;
                        let (c, r) = requests_panel(
                            &mut columns[1],
                            d,
                            &mut self.metrics,
                            scope,
                            &mut self.focus_input,
                        );
                        changed |= c;
                        request = r;
                    });
                } else {
                    let (c, p) = policy_panel(
                        ui,
                        d,
                        &mut self.nat_presentation,
                        &mut self.metrics,
                        scope,
                        &mut self.focus_input,
                    );
                    changed |= c;
                    apply = p;
                    ui.separator();
                    let (c, r) =
                        requests_panel(ui, d, &mut self.metrics, scope, &mut self.focus_input);
                    changed |= c;
                    request = r;
                }
            });
        });
        self.dirty |= changed;
        if active && apply {
            // Always dirty on an actual Apply, including identical notice text.
            self.dirty = true;
            match d.apply(initial, simulation) {
                Ok(_) => {
                    d.notice = Some("Policy applied".into());
                    self.nat_presentation.effect = Some(core::Effect::Configured);
                }
                Err(e) => {
                    d.notice = Some(work::notice(&e));
                    self.nat_presentation.effect = None;
                }
            }
        }
        if active && let Some(request) = request {
            self.dirty = true;
            match request.and_then(|r| work::perform(initial, simulation, &r)) {
                Ok((_, effect)) => {
                    d.notice = None;
                    self.nat_presentation.effect = Some(effect);
                }
                Err(e) => {
                    d.notice = Some(work::notice(&e));
                    self.nat_presentation.effect = None;
                }
            }
        }
        if let Some(n) = &d.notice {
            ui.label(n);
        }
        ui.push_id(scope, |ui| {
            state_panel(ui, simulation, &self.nat_presentation, &mut self.metrics)
        });
        let check = ui
            .add_enabled_ui(active, |ui| {
                action(ui, &mut self.metrics, "check", "Check work", true)
            })
            .inner
            || active && ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter));
        self.metrics.check = self
            .metrics
            .nat_controls
            .iter()
            .find(|r| r.0 == "check")
            .map(|r| r.1);
        // Evaluate AFTER every possible editor/action mutation. Never reuse a
        // library prepass from earlier in this frame as a post-action result.
        if check {
            a.workspace.as_mut().unwrap().checked = true;
            self.dirty = true;
        }
        self.nat_presentation.memo.evaluate(a, s, check);
        let checked = a.workspace.as_ref().is_some_and(|w| w.checked);
        if checked {
            let rows = self.nat_presentation.memo.rows(&s.id).unwrap();
            let r = controls::feedback(ui, check, |ui| {
                for (ok, message) in rows {
                    ui.label(check_text(*ok, message));
                }
            });
            self.metrics.nat_feedback = Some(r.response.rect);
            self.metrics.feedback = Some(r.response.rect);
            self.metrics.completion = rows.iter().all(|r| r.0);
        }
        if !challenge {
            egui::CollapsingHeader::new("Why these controls?").id_salt(scope.with("why")).show(ui,|ui| {
                ui.label("Applying edits does not make a dynamic map. Outbound can allocate one for an eligible supplied identity. Reverse observes an existing static or dynamic map. Check reads current maps and tests copied policy in independent fresh orders without changing this workspace.");
            });
        }
    }
}
