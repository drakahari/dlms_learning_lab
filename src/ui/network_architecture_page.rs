//! Native campus cable work; all paths come from the current bounded graph.
use super::*;
use dlms_learning_lab::{
    network_architecture as core, network_architecture_source as binding,
    network_architecture_workspace as work,
};
use std::collections::BTreeMap;
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub focus: Vec<(String, egui::Id)>,
    pub cards: Vec<egui::Rect>,
    pub diagram: Option<egui::Rect>,
    pub results: Option<egui::Rect>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
    }
}
fn jack_label(t: &work::Task, j: &Option<core::Jack>) -> String {
    j.as_ref()
        .map(|j| {
            format!(
                "{} / {}",
                t.facts.node(&j.node).map_or(j.node.as_str(), |n| &n.label),
                j.port
            )
        })
        .unwrap_or_else(|| "Unplugged".into())
}
/// Preserve keyboard focus when this native control reflows. A deliberate key,
/// pointer press, another focused widget or disabled/modal state takes precedence.
fn retain_focus(ui: &egui::Ui, r: &egui::Response) {
    let key = r.id.with("campus-focus");
    let previous = ui
        .ctx()
        .data(|d| d.get_temp::<(egui::Rect, bool, bool)>(key));
    let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
    let navigating = ui.input(|i| {
        i.events.iter().any(|e| {
            matches!(
                e,
                egui::Event::Key { pressed: true, .. }
                    | egui::Event::PointerButton { pressed: true, .. }
            )
        })
    });
    let reflow = previous.is_some_and(|(rect, focused, _)| focused && rect != r.rect)
        && ui.ctx().memory(|m| m.focused().is_none())
        && !navigating
        && ui.input(|i| i.focused);
    let escape_return = escape && previous.is_some_and(|(_, _, open)| open);
    if ui.is_enabled() && (escape_return || reflow) {
        r.request_focus();
    }
    let open = egui::ComboBox::is_open(ui.ctx(), r.id);
    let focused = r.has_focus();
    ui.ctx()
        .data_mut(|d| d.insert_temp(key, (r.rect, focused, open)));
}
fn endpoint(
    ui: &mut egui::Ui,
    t: &work::Task,
    key: &str,
    j: &mut Option<core::Jack>,
    width: f32,
    m: &mut Metrics,
) {
    let r = controls::dropdown(ui, key, width)
        .selected_text(jack_label(t, j))
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            let begin = m.controls.len();
            let r = ui.selectable_value(j, None, "Unplugged");
            m.record(ui, &format!("{key}:unplugged"), &r);
            for n in &t.facts.nodes {
                for p in &n.ports {
                    let option = Some(core::Jack {
                        node: n.id.clone(),
                        port: p.id.clone(),
                    });
                    let r = ui.selectable_value(j, option.clone(), jack_label(t, &option));
                    m.record(ui, &format!("{key}:{}/{}", n.id, p.id), &r);
                }
            }
            let clip = controls::popup_content_clip(ui);
            for c in &mut m.controls[begin..] {
                c.2 = clip;
            }
        });
    retain_focus(ui, &r.response);
    m.focus.push((key.into(), r.response.id));
    m.record(ui, key, &r.response);
}
fn wiring(ui: &mut egui::Ui, t: &work::Task, draft: &mut core::Configuration, m: &mut Metrics) {
    ui.strong("Cable endpoints");
    let wide = ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 43.;
    let gap = ui.spacing().item_spacing.x;
    let label = ui.text_style_height(&egui::TextStyle::Body) * 7.;
    let field = ((ui.available_width() - label - 2. * gap) / 2.).max(1.);
    if wide {
        ui.horizontal(|ui| {
            ui.add_sized([label, controls::height(ui)], egui::Label::new("Cable"));
            ui.add_sized(
                [field, controls::height(ui)],
                egui::Label::new("End 1 · device / port"),
            );
            ui.add_sized(
                [field, controls::height(ui)],
                egui::Label::new("End 2 · device / port"),
            );
        });
    }
    for c in draft
        .connections
        .iter_mut()
        .filter(|c| t.editable.iter().any(|w| w.id == c.wire))
    {
        let editable = t.editable.iter().any(|w| w.id == c.wire);
        let name = format!("{}{}", c.wire, if editable { "" } else { " · fixed" });
        ui.push_id(&c.wire, |ui| {
            if wide {
                ui.horizontal(|ui| {
                    ui.add_sized([label, controls::height(ui)], egui::Label::new(&name));
                    ui.add_enabled_ui(editable, |ui| {
                        endpoint(ui, t, &format!("{}:a", c.wire), &mut c.a, field, m);
                        endpoint(ui, t, &format!("{}:b", c.wire), &mut c.b, field, m);
                    });
                });
            } else {
                ui.strong(&name);
                ui.add_enabled_ui(editable, |ui| {
                    for (label, end, jack) in [("End 1", "a", &mut c.a), ("End 2", "b", &mut c.b)] {
                        if ui.available_width() < ui.text_style_height(&egui::TextStyle::Body) * 28.
                        {
                            ui.label(label);
                            let width = ui.available_width();
                            endpoint(ui, t, &format!("{}:{end}", c.wire), jack, width, m);
                        } else {
                            controls::form_row(ui, label, |ui, width| {
                                endpoint(ui, t, &format!("{}:{end}", c.wire), jack, width, m)
                            });
                        }
                    }
                });
            }
        });
        ui.add_space(4.);
    }
    let fixed = controls::collapsing_header(ui, "Inspect fixed cables · preserved endpoints").show(
        ui,
        |ui| {
            for c in draft
                .connections
                .iter()
                .filter(|c| !t.editable.iter().any(|w| w.id == c.wire))
            {
                ui.strong(&c.wire);
                lesson_presentation::prose(
                    ui,
                    &format!("{} <-> {}", jack_label(t, &c.a), jack_label(t, &c.b)),
                );
            }
        },
    );
    m.record(ui, "fixed-cables", &fixed.header_response);
}
fn topology(
    ui: &mut egui::Ui,
    t: &work::Task,
    c: &core::Configuration,
    e: &core::Experiment,
    m: &mut Metrics,
) {
    let line = ui.text_style_height(&egui::TextStyle::Body);
    let gap = controls::GAP * 2.;
    let columns = ((ui.available_width() + gap) / (line * 15. + gap))
        .floor()
        .clamp(1., 4.) as usize;
    let width = ((ui.available_width() - gap * (columns - 1) as f32) / columns as f32).max(1.);
    let mut nodes = t.facts.nodes.iter().collect::<Vec<_>>();
    nodes.sort_by_key(|n| match n.role {
        core::Role::Core | core::Role::CollapsedCore | core::Role::Spine => 0,
        core::Role::Distribution => 1,
        core::Role::Access | core::Role::Leaf => 2,
    });
    let colors = palette(ui);
    let text_width = (width - 2. * controls::PAD_X as f32).max(1.);
    let layouts: Vec<_> = nodes
        .iter()
        .map(|n| {
            let mut details = n.role.label().to_string();
            if let Some(block) = &n.block {
                details.push_str(&format!(
                    "\n{}: {block}",
                    if n.role == core::Role::Leaf {
                        "Rack"
                    } else {
                        "Building"
                    }
                ));
            }
            let down = e
                .unavailable
                .iter()
                .any(|f| matches!(f,core::Failure::Node{id}if id==&n.id));
            if down {
                details.push_str("\nUnavailable in this case");
            }
            let text = format!("{}\n{details}", n.label);
            ui.painter().layout(
                text,
                egui::TextStyle::Body.resolve(ui.style()),
                colors.text,
                text_width,
            )
        })
        .collect();
    let h = layouts.iter().map(|g| g.size().y).fold(line, f32::max) + 2. * controls::PAD_Y as f32;
    // Preserve tier rows instead of allowing a wide equipment grid to mix roles.
    let rank = |n: &core::Node| match n.role {
        core::Role::Core | core::Role::CollapsedCore | core::Role::Spine => 0,
        core::Role::Distribution => 1,
        core::Role::Access | core::Role::Leaf => 2,
    };
    let mut slots = Vec::new();
    let mut rows = 0;
    for tier in 0..=2 {
        let group = nodes.iter().filter(|n| rank(n) == tier).collect::<Vec<_>>();
        for chunk in group.chunks(columns) {
            for (col, n) in chunk.iter().enumerate() {
                slots.push((n.id.as_str(), rows, col, chunk.len()));
            }
            rows += 1;
        }
    }
    let (area, response) = ui.allocate_exact_size(
        egui::vec2(
            ui.available_width(),
            rows as f32 * h + (rows - 1) as f32 * gap,
        ),
        egui::Sense::hover(),
    );
    response.widget_info(||egui::WidgetInfo::labeled(egui::WidgetType::Image,ui.is_enabled(),"Current campus tier roles. Exact cable endpoints are also available as native text and selectors in the cable table."));
    let rects: BTreeMap<_, _> = slots
        .into_iter()
        .map(|(id, row, col, count)| {
            let used = count as f32 * width + (count - 1) as f32 * gap;
            (
                id,
                egui::Rect::from_min_size(
                    area.min
                        + egui::vec2(
                            (area.width() - used) / 2. + col as f32 * (width + gap),
                            row as f32 * (h + gap),
                        ),
                    egui::vec2(width, h),
                ),
            )
        })
        .collect();
    if columns >= 2 {
        for c in &c.connections {
            if let (Some(a), Some(b)) = (&c.a, &c.b) {
                ui.painter().line_segment(
                    [
                        rects[a.node.as_str()].center(),
                        rects[b.node.as_str()].center(),
                    ],
                    egui::Stroke::new(1.5_f32, colors.muted),
                );
            }
        }
    }
    for (i, n) in nodes.iter().enumerate() {
        let r = rects[n.id.as_str()];
        ui.painter().rect(
            r,
            6.,
            colors.panel,
            egui::Stroke::new(1_f32, colors.border),
            egui::StrokeKind::Inside,
        );
        ui.painter().galley(
            r.min + egui::vec2(controls::PAD_X as f32, controls::PAD_Y as f32),
            layouts[i].clone(),
            colors.text,
        );
        m.cards.push(r);
    }
    m.diagram = Some(area);
    lesson_presentation::prose(
        ui,
        if columns >= 2 {
            "Lines show complete chosen cables, including cables that may violate a role requirement. A labeled unavailable device is excluded from the case’s path below."
        } else {
            "The cable table gives the exact chosen connections. A labeled unavailable device is excluded from the case’s path below."
        },
    );
}
fn observe(
    ui: &mut egui::Ui,
    t: &work::Task,
    a: &work::Attempt,
    case: &mut usize,
    pair_index: &mut usize,
    m: &mut Metrics,
) {
    *case = (*case).min(t.cases.len() - 1);
    ui.strong("Inspect a stated case");
    let r = controls::dropdown(ui, "campus-case", ui.available_width())
        .selected_text(&t.cases[*case].label)
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            let begin = m.controls.len();
            for (i, c) in t.cases.iter().enumerate() {
                let r = ui.selectable_value(case, i, &c.label);
                m.record(ui, &format!("case:{}", c.id), &r);
            }
            let clip = controls::popup_content_clip(ui);
            for c in &mut m.controls[begin..] {
                c.2 = clip;
            }
        });
    retain_focus(ui, &r.response);
    m.record(ui, "case", &r.response);
    *pair_index = (*pair_index).min(t.pairs.len() - 1);
    if t.facts.design == core::Design::SpineLeaf {
        let label = |p: &work::Pair| {
            format!(
                "{} to {}",
                t.facts.node(&p.from).unwrap().label,
                t.facts.node(&p.to).unwrap().label
            )
        };
        ui.label("Rack pair to inspect");
        let r = controls::dropdown(ui, "fabric-pair", ui.available_width())
            .selected_text(label(&t.pairs[*pair_index]))
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (i, pair) in t.pairs.iter().enumerate() {
                    let r = ui.selectable_value(pair_index, i, label(pair));
                    m.record(ui, &format!("pair:{i}"), &r);
                }
            });
        retain_focus(ui, &r.response);
        m.record(ui, "pair", &r.response);
    }
    let pair = &t.pairs[*pair_index];
    let e = &t.cases[*case].experiment;
    topology(ui, t, &a.draft, e, m);
    ui.strong("Actual current layout · structural observation");
    match t.facts.inspect(&a.draft, e, &pair.from, &pair.to) {
        Ok(o) => {
            if let Some(paths) = &o.spine_alternatives {
                let r = lesson_presentation::prose(
                    ui,
                    &format!(
                        "Actual distinct internal spine alternatives: {}",
                        paths.len()
                    ),
                );
                m.record(ui, "spine-count", &r);
                for p in paths {
                    lesson_presentation::prose(
                        ui,
                        &p.nodes
                            .iter()
                            .map(|id| t.facts.node(id).unwrap().label.as_str())
                            .collect::<Vec<_>>()
                            .join(" > "),
                    );
                }
                if let Some(minimum) = t.cases[*case].minimum_paths {
                    lesson_presentation::prose(
                        ui,
                        &format!(
                            "Required in this case: at least {minimum} distinct internal spines for this pair."
                        ),
                    );
                }
                if o.hierarchy.is_none() && o.physical.is_some() {
                    lesson_presentation::prose(
                        ui,
                        "A physical path exists, but it does not follow the required leaf–spine–leaf shape. Inspect the chosen endpoints.",
                    );
                }
            }
            let path = |p: Option<core::Path>| {
                p.map(|p| {
                    p.nodes
                        .iter()
                        .map(|id| t.facts.node(id).unwrap().label.clone())
                        .collect::<Vec<_>>()
                        .join(" > ")
                })
                .unwrap_or_else(|| "No path".into())
            };
            if o.spine_alternatives.is_none() {
                lesson_presentation::prose(ui, &format!("Physical path: {}", path(o.physical)));
                lesson_presentation::prose(
                    ui,
                    &format!("Required hierarchy path: {}", path(o.hierarchy)),
                );
            }
        }
        Err(e) => {
            lesson_presentation::prose(ui, &e);
        }
    }
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
        lesson_presentation::prose(
            ui,
            "Unchecked · select Check answers to evaluate the full task, not only the displayed case.",
        );
        return;
    }
    let checks = t.checks(&a.draft);
    let ok = checks.iter().all(|c| c.passed);
    let title = ui.strong(if ok {
        if practice {
            "Example checks pass"
        } else {
            "Assignment complete"
        }
    } else {
        "Layout needs attention"
    });
    m.record(ui, "result-title", &title);
    m.results = Some(title.rect);
    let compact = t.facts.design == core::Design::SpineLeaf;
    let prefix = 1 + t.uplinks.len();
    for c in checks
        .iter()
        .take(if compact { prefix } else { checks.len() })
    {
        let r = lesson_presentation::prose(
            ui,
            &format!(
                "{} · {}",
                if c.passed {
                    "Passed"
                } else {
                    "Needs attention"
                },
                c.detail
            ),
        );
        m.record(ui, if c.passed { "passed-row" } else { "failed-row" }, &r);
    }
    if compact {
        if !ok {
            lesson_presentation::prose(
                ui,
                "Each pair needs enough shared, available spines; parallel cables to one spine count once.",
            );
        }
        for (case, rows) in t
            .cases
            .iter()
            .zip(checks[prefix.min(checks.len())..].chunks(t.pairs.len()))
        {
            let passed = rows.iter().filter(|r| r.passed).count();
            let r = lesson_presentation::prose(
                ui,
                &format!(
                    "{} · {}: {passed}/{} rack pairs meet the path requirement.",
                    if passed == t.pairs.len() {
                        "Passed"
                    } else {
                        "Needs attention"
                    },
                    case.label,
                    t.pairs.len()
                ),
            );
            m.record(ui, "case-summary", &r);
            for (pair, _) in t.pairs.iter().zip(rows).filter(|(_, r)| !r.passed) {
                let r = lesson_presentation::prose(ui, &pair_failure(t, a, case, pair));
                m.record(ui, "failed-row", &r);
            }
            ui.push_id(&case.id, |ui| {
                controls::collapsing_header(ui, "Inspect pair paths and diagnostics").show(
                    ui,
                    |ui| {
                        for row in rows {
                            lesson_presentation::prose(ui, &row.detail);
                        }
                    },
                );
            });
        }
    }
    if ok {
        lesson_presentation::prose(ui, &s.explanation);
    }
}
/// Keep essential failed counts visible; full paths remain available on demand.
/// Counts come from the current graph, never parsed from diagnostic strings.
pub(super) fn pair_failure(
    t: &work::Task,
    a: &work::Attempt,
    case: &work::Case,
    pair: &work::Pair,
) -> String {
    let names = format!(
        "{} to {}",
        t.facts.node(&pair.from).unwrap().label,
        t.facts.node(&pair.to).unwrap().label
    );
    match t
        .facts
        .inspect(&a.draft, &case.experiment, &pair.from, &pair.to)
    {
        Ok(o) => format!(
            "{names}: {} of {} required internal spines available.",
            o.spine_alternatives.as_ref().map_or(0, Vec::len),
            case.minimum_paths.unwrap_or(1)
        ),
        Err(e) => format!("{names}: {e}"),
    }
}
impl LabApp {
    pub(super) fn network_architecture_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
            engine::Goal::CampusArchitecture { definition },
            engine::State::CampusArchitecture { simulation },
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
                    self.metrics.network_architecture.record(ui, key, &r);
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
                "Separate guide equipment: trace the roles, then build its missing connections.",
            );
            let worked = ui.scope(|ui| lesson_presentation::worked_rows(ui, &s));
            self.metrics
                .network_architecture
                .record(ui, "worked-table", &worked.response);
            let r = ui.add_enabled(active, primary(ui, "Try this example"));
            self.metrics.network_architecture.record(ui, "example", &r);
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
            if !practice {
                lesson_presentation::assignment(ui, &s);
            } else {
                lesson_presentation::practice_step(ui, &s, 0);
            }
            let scope = egui::Id::new(("campus-work", &s.id, practice));
            let mut selected = ui
                .ctx()
                .data(|d| d.get_temp::<usize>(scope.with("case")).unwrap_or(0));
            let mut pair_index = ui
                .ctx()
                .data(|d| d.get_temp::<usize>(scope.with("pair")).unwrap_or(0));
            let a = next.attempt_mut(practice);
            let mut draft = a.draft.clone();
            ui.push_id(scope, |ui| {
                ui.add_enabled_ui(active, |ui| {
                    if practice {
                        lesson_presentation::practice_step(ui, &s, 1);
                    }
                    wiring(ui, t, &mut draft, &mut self.metrics.network_architecture);
                    if draft != a.draft
                        && let Err(e) = a.edit(t, draft)
                    {
                        self.notice = Some(e);
                    }
                    ui.horizontal_wrapped(|ui| {
                        let r = ui.add_enabled(active, primary(ui, "Check answers"));
                        self.metrics.check = Some(r.rect);
                        self.metrics.network_architecture.record(ui, "check", &r);
                        if r.clicked()
                            && let Err(e) = a.check(t)
                        {
                            self.notice = Some(e);
                        }
                        if practice {
                            let r = ui.add_enabled(active, button("Show worked layout"));
                            self.metrics.network_architecture.record(ui, "worked", &r);
                            if r.clicked()
                                && let Err(e) = a.edit(t, t.reference.clone())
                            {
                                self.notice = Some(e);
                            }
                            let r = ui.add_enabled(active, button("Reset example only"));
                            self.metrics
                                .network_architecture
                                .record(ui, "reset-example", &r);
                            if r.clicked() {
                                *a = work::Attempt {
                                    draft: t.initial.clone(),
                                    checked: false,
                                };
                            }
                        }
                    });
                    results(
                        ui,
                        t,
                        a,
                        &s,
                        practice,
                        &mut self.metrics.network_architecture,
                    );
                    if practice {
                        lesson_presentation::practice_step(ui, &s, 2);
                    }
                    observe(
                        ui,
                        t,
                        a,
                        &mut selected,
                        &mut pair_index,
                        &mut self.metrics.network_architecture,
                    );
                });
            });
            ui.ctx()
                .data_mut(|d| d.insert_temp(scope.with("case"), selected));
            ui.ctx()
                .data_mut(|d| d.insert_temp(scope.with("pair"), pair_index));
            ui.push_id(("campus-help", &s.id, challenge, practice), |ui| {
                let help = controls::collapsing_header(ui, "Optional help and simulation scope")
                    .show(ui, |ui| {
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
                            let reasoning = controls::collapsing_header(
                                ui,
                                "Worked approach · reveals reasoning",
                            )
                            .show(ui, |ui| {
                                lesson_presentation::prose(ui, &s.explanation);
                            });
                            self.metrics.network_architecture.record(
                                ui,
                                "reasoning",
                                &reasoning.header_response,
                            );
                        }
                        lesson_presentation::prose(ui, &s.simplifications);
                        for link in &s.sources {
                            ui.hyperlink_to(link, link);
                        }
                    });
                self.metrics
                    .network_architecture
                    .record(ui, "help", &help.header_response);
            });
        }
        if active && next != before {
            match definition.context(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&s.id).unwrap().state =
                        engine::State::CampusArchitecture {
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
