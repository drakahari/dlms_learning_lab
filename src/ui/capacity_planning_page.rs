//! Native planning graph, quote table and editable policy; no learner-host I/O.
use super::*;
use dlms_learning_lab::{
    capacity_planning_workspace as work,
    engine::{Goal, State},
};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub choices: Vec<(String, egui::Rect, egui::Rect)>,
    pub graph: Option<egui::Rect>,
    pub result: Option<(usize, usize, egui::Rect)>,
}
fn record(m: &mut Metrics, key: impl Into<String>, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn gib(n: u64) -> String {
    let v = n as f64 / 1048576.;
    if n.is_multiple_of(1048576) {
        format!("{v:.0}")
    } else {
        format!("{v:.2}")
    }
}
fn sample(t: &work::Task, i: usize) -> String {
    let s = &t.series.samples[i];
    format!("Day {} · {} GiB occupied", s.day, gib(s.used_kib))
}
fn plot(ui: &mut egui::Ui, t: &work::Task, d: &work::Draft, m: &mut Metrics) {
    lesson_presentation::rich(
        ui,
        RichText::new("Occupancy and the selected growth assumption").strong(),
    );
    let color = palette(ui);
    let font = egui::TextStyle::Body.resolve(ui.style());
    let line = ui.text_style_height(&egui::TextStyle::Body);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), line * 14.),
        egui::Sense::hover(),
    );
    m.graph = Some(rect);
    let anchor = t.series.samples.last().unwrap();
    let through = anchor.day + t.policy.horizon_days;
    let growth = t.window(d).ok().and_then(|w| t.series.growth(w).ok());
    let mut projection = vec![];
    let mut projection_error = None;
    if let Some(g) = growth {
        for day in anchor.day..=through {
            match g.at(anchor, day) {
                Ok(n) => projection.push((day, n)),
                Err(e) => {
                    projection_error = Some(e.to_string());
                    break;
                }
            }
        }
    }
    let assessment = if d.checked { Some(t.assess(d)) } else { None };
    let tested = assessment
        .as_ref()
        .and_then(|a| a.cases.get(d.selected_case));
    let maximum = t
        .series
        .capacity_kib
        .max(t.series.samples.iter().map(|s| s.used_kib).max().unwrap())
        .max(projection.iter().map(|s| s.1).max().unwrap_or(0))
        .max(
            tested
                .map(|c| {
                    c.days
                        .iter()
                        .map(|d| d.used_kib.max(d.capacity_kib))
                        .max()
                        .unwrap()
                })
                .unwrap_or(0),
        );
    let axis_width = controls::button_width(ui, &format!("{} GiB", gib(maximum)));
    let canvas = egui::Rect::from_min_max(
        rect.left_top() + egui::vec2(axis_width, line),
        rect.right_bottom() - egui::vec2(line, line * 2.5),
    );
    if canvas.width() < line * 3. {
        lesson_presentation::prose(ui, "Use the observation table below at this width.");
        return;
    }
    let xmin = t.series.samples[0].day;
    let xmax = through;
    let xy = |day: u32, n: u64| {
        egui::pos2(
            canvas.left() + canvas.width() * (day - xmin) as f32 / (xmax - xmin) as f32,
            canvas.bottom() - canvas.height() * (n as f64 / maximum.max(1) as f64) as f32,
        )
    };
    let painter = ui.painter();
    let mono = egui::TextStyle::Monospace.resolve(ui.style());
    for n in [0, maximum / 2, maximum] {
        let y = xy(xmin, n).y;
        painter.line_segment(
            [egui::pos2(canvas.left(), y), egui::pos2(canvas.right(), y)],
            egui::Stroke::new(1f32, color.muted.gamma_multiply(0.4)),
        );
        painter.text(
            egui::pos2(canvas.left() - line * 0.4, y),
            egui::Align2::RIGHT_CENTER,
            format!("{} GiB", gib(n)),
            mono.clone(),
            color.text,
        );
    }
    let ticks = if canvas.width() > line * 20. {
        vec![xmin, anchor.day, (anchor.day + through) / 2, through]
    } else {
        vec![xmin, anchor.day, through]
    };
    for day in ticks {
        painter.text(
            egui::pos2(xy(day, 0).x, canvas.bottom() + line * 0.5),
            egui::Align2::CENTER_TOP,
            day.to_string(),
            mono.clone(),
            color.text,
        );
    }
    painter.text(
        egui::pos2(canvas.center().x, rect.bottom()),
        egui::Align2::CENTER_BOTTOM,
        "Recording day",
        font,
        color.text,
    );
    painter.line_segment(
        [canvas.left_bottom(), canvas.left_top()],
        egui::Stroke::new(1f32, color.text),
    );
    painter.line_segment(
        [canvas.left_bottom(), canvas.right_bottom()],
        egui::Stroke::new(1f32, color.text),
    );
    let observed: Vec<_> = t
        .series
        .samples
        .iter()
        .map(|s| xy(s.day, s.used_kib))
        .collect();
    for pair in observed.windows(2) {
        painter.line_segment([pair[0], pair[1]], egui::Stroke::new(2f32, color.text));
    }
    for p in observed {
        painter.circle_filled(p, 3., color.text);
    }
    let cap = xy(xmin, t.series.capacity_kib).y;
    painter.line_segment(
        [
            egui::pos2(canvas.left(), cap),
            egui::pos2(canvas.right(), cap),
        ],
        egui::Stroke::new(1f32, color.muted),
    );
    for pair in projection.windows(2).enumerate() {
        if pair.0 % 2 == 0 {
            painter.line_segment(
                [xy(pair.1[0].0, pair.1[0].1), xy(pair.1[1].0, pair.1[1].1)],
                egui::Stroke::new(2f32, color.accent),
            );
        }
    }
    if let Some(case) = tested {
        let case_color = if case.first_failure.is_some() {
            color.error
        } else {
            color.success
        };
        for pair in case.days.windows(2) {
            painter.line_segment(
                [
                    xy(pair[0].day, pair[0].used_kib),
                    xy(pair[1].day, pair[1].used_kib),
                ],
                egui::Stroke::new(2f32, case_color),
            );
        }
        for row in case.days.iter().step_by((case.days.len() / 8).max(1)) {
            painter.rect_filled(
                egui::Rect::from_center_size(xy(row.day, row.used_kib), egui::vec2(5., 5.)),
                0.,
                case_color,
            );
        }
    }
    if let Some(e) = projection_error {
        lesson_presentation::prose(ui, &format!("Projection limited · {e}"));
    }
    lesson_presentation::prose(
        ui,
        &format!(
            "Solid circles: observed occupancy. Dashed line: projection. Squares: selected simulated workload after testing. Thin horizontal line: original {} GiB capacity.",
            gib(t.series.capacity_kib)
        ),
    );
    if let Some(g) = growth {
        let rate = g.change_kib as f64 / f64::from(g.days) / 1048576.;
        let window = t.window(d).unwrap();
        let first = &t.series.samples[window.first];
        let last = &t.series.samples[window.last];
        lesson_presentation::prose(
            ui,
            &format!(
                "Selected window: ({} − {}) GiB / {} days = {rate:.2} GiB/day. This assumption does not include the published workload additions.",
                gib(last.used_kib),
                gib(first.used_kib),
                g.days
            ),
        );
    } else {
        lesson_presentation::prose(
            ui,
            "Choose both growth-window observations to draw a projection. This is a planning assumption, not a measurement of the future.",
        );
    }
    if let Some(case) = tested {
        lesson_presentation::prose(
            ui,
            &format!(
                "Displayed workload: {} · {}.",
                case.id,
                case.first_failure
                    .map(|day| format!("reserve first fails on day {day}"))
                    .unwrap_or_else(|| "reserve retained throughout".into())
            ),
        );
    }
    controls::collapsing_header(ui, "Observation data · exact KiB").show(ui, |ui| {
        for s in &t.series.samples {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("Day {}", s.day));
                ui.monospace(format!("{} KiB ({} GiB)", s.used_kib, gib(s.used_kib)));
            });
        }
    });
}
fn quotes(ui: &mut egui::Ui, t: &work::Task) {
    lesson_presentation::rich(
        ui,
        RichText::new("Supplied quotes · fictional credits").strong(),
    );
    let widths = [0.37, 0.23, 0.20, 0.20].map(|f| (ui.available_width() - 3. * controls::GAP) * f);
    ui.horizontal(|ui| {
        for (label, width) in ["Quote", "Capacity GiB", "Lead days", "Credits"]
            .into_iter()
            .zip(widths)
        {
            controls::table_text(ui, RichText::new(label).strong(), width);
        }
    });
    for o in &t.offers {
        ui.horizontal(|ui| {
            for (value, width) in [
                o.id.clone(),
                gib(o.capacity_kib),
                o.lead_days.to_string(),
                o.cost.to_string(),
            ]
            .into_iter()
            .zip(widths)
            {
                controls::table_text(ui, value, width);
            }
        });
    }
    controls::collapsing_header(ui, "Published workload cases").show(ui, |ui| {
        for c in &t.demands {
            lesson_presentation::rich(ui, RichText::new(&c.id).strong());
            ui.monospace(format!(
                "{} KiB/day ({} GiB/day)",
                c.daily_kib,
                gib(c.daily_kib)
            ));
            for e in &c.changes {
                ui.monospace(format!(
                    "Day {}: add {} KiB ({} GiB)",
                    e.day,
                    e.added_kib,
                    gib(e.added_kib)
                ));
            }
        }
    });
}
fn window_picker(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &mut work::Draft,
    m: &mut Metrics,
    active: bool,
    first: bool,
) {
    let (label, key, value) = if first {
        ("Growth window start", "window-first", &mut d.inputs.first)
    } else {
        ("Growth window end", "window-last", &mut d.inputs.last)
    };
    controls::form_row(ui, label, |ui, width| {
        let old = *value;
        let r = ui
            .add_enabled_ui(active, |ui| {
                controls::dropdown(ui, key, width)
                    .selected_text(
                        value
                            .map(|i| sample(t, i))
                            .unwrap_or("Choose an observation".into()),
                    )
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        let start = m.choices.len();
                        for i in 0..t.series.samples.len() {
                            let r = ui.selectable_value(value, Some(i), sample(t, i));
                            m.choices
                                .push((format!("{key}-{i}"), r.rect, ui.clip_rect()));
                        }
                        let clip = controls::popup_content_clip(ui);
                        for choice in &mut m.choices[start..] {
                            choice.2 = clip;
                        }
                    })
                    .response
            })
            .inner;
        record(m, key, &r, ui);
        if old != *value {
            d.checked = false;
        }
    });
}
fn input(
    ui: &mut egui::Ui,
    m: &mut Metrics,
    d: &mut work::Draft,
    key: &str,
    label: &str,
    units: &str,
    active: bool,
) -> bool {
    let mut submit = false;
    controls::form_row(ui, label, |ui, width| {
        ui.horizontal_wrapped(|ui| {
            let unit_width = controls::button_width(ui, units);
            let width = (width - unit_width - controls::GAP).clamp(80., 180.);
            let value = if key == "order-day" {
                &mut d.inputs.order_day
            } else {
                &mut d.inputs.reserve_kib
            };
            let r = ui.add_enabled(
                active,
                controls::singleline_fitting(ui, value, width)
                    .char_limit(24)
                    .hint_text(if key == "order-day" {
                        "Recording day"
                    } else {
                        "Whole KiB"
                    }),
            );
            m.fields.push((key.into(), r.rect, r.id, ui.clip_rect()));
            if r.changed() {
                d.checked = false;
            }
            submit = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            ui.label(units);
        });
    });
    submit
}
fn plan_controls(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &mut work::Draft,
    m: &mut Metrics,
    active: bool,
    practice: bool,
    s: &pack::Scenario,
) {
    if practice {
        lesson_presentation::practice_step(ui, s, 0);
    }
    window_picker(ui, t, d, m, active, true);
    window_picker(ui, t, d, m, active, false);
    if practice {
        lesson_presentation::practice_step(ui, s, 1);
    }
    controls::form_row(ui, "Upgrade quote", |ui, width| {
        let old = d.inputs.offer.clone();
        let r = ui
            .add_enabled_ui(active, |ui| {
                controls::dropdown(ui, "quote", width)
                    .selected_text(d.inputs.offer.as_deref().unwrap_or("Choose a quote"))
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        let start = m.choices.len();
                        for o in &t.offers {
                            let r = ui.selectable_value(
                                &mut d.inputs.offer,
                                Some(o.id.clone()),
                                format!(
                                    "{} · {} GiB · {} days · {} credits",
                                    o.id,
                                    gib(o.capacity_kib),
                                    o.lead_days,
                                    o.cost
                                ),
                            );
                            m.choices
                                .push((format!("quote-{}", o.id), r.rect, ui.clip_rect()));
                        }
                        let clip = controls::popup_content_clip(ui);
                        for choice in &mut m.choices[start..] {
                            choice.2 = clip;
                        }
                    })
                    .response
            })
            .inner;
        record(m, "quote", &r, ui);
        if old != d.inputs.offer {
            d.checked = false;
        }
    });
    if let Some(o) = d
        .inputs
        .offer
        .as_ref()
        .and_then(|id| t.offers.iter().find(|o| &o.id == id))
    {
        lesson_presentation::prose(
            ui,
            &format!(
                "Selected quote: {} GiB, {} lead days, {} credits.",
                gib(o.capacity_kib),
                o.lead_days,
                o.cost
            ),
        );
    }
    let mut submit = input(ui, m, d, "order-day", "Order day", "day", active);
    submit |= input(ui, m, d, "reserve", "Keep free", "KiB", active);
    lesson_presentation::prose(
        ui,
        &format!(
            "Required free space: at least {} KiB ({} GiB). Budget: {} credits. Test through day {}.",
            t.policy.minimum_reserve_kib,
            gib(t.policy.minimum_reserve_kib),
            t.policy.budget,
            t.series.samples.last().unwrap().day + t.policy.horizon_days
        ),
    );
    let r = ui.add_enabled(
        active,
        primary(
            ui,
            if practice {
                "Test example plan"
            } else {
                "Test plan"
            },
        ),
    );
    record(m, "test", &r, ui);
    if active && (r.clicked() || submit) {
        d.apply(t, &work::Action::Test).unwrap();
    }
    if !d.checked {
        lesson_presentation::prose(
            ui,
            "Plan not tested. Editing a planning field clears its previous result.",
        );
    }
}
fn results(ui: &mut egui::Ui, t: &work::Task, d: &mut work::Draft, m: &mut Metrics, active: bool) {
    if !d.checked {
        return;
    }
    let a = t.assess(d);
    let failed = a.checks.iter().filter(|r| !r.0).count();
    let r=lesson_presentation::rich(ui,RichText::new(if a.passed(){format!("Plan verified · All {} policy and workload checks pass.",a.checks.len())}else{format!("Plan needs attention · {failed} of {} checks fail. A passing workload alone does not pass the plan.",a.checks.len())}).strong());
    m.result = Some((a.checks.len(), failed, r.rect));
    for (i, (ok, text)) in a.checks.iter().enumerate() {
        let r = lesson_presentation::rich(
            ui,
            RichText::new(format!(
                "{} · {text}",
                if *ok {
                    "Meets requirement"
                } else {
                    "Needs attention"
                }
            ))
            .color(if *ok {
                palette(ui).success
            } else {
                palette(ui).error
            }),
        );
        record(m, format!("feedback-{i}"), &r, ui);
    }
    if !a.cases.is_empty() {
        controls::form_row(ui, "Graph workload", |ui, width| {
            let r = ui
                .add_enabled_ui(active, |ui| {
                    controls::dropdown(ui, "case", width)
                        .selected_text(&a.cases[d.selected_case].id)
                        .show_ui(ui, |ui| {
                            controls::fit_popup_contents(ui);
                            let start = m.choices.len();
                            for (i, c) in a.cases.iter().enumerate() {
                                let r = ui.selectable_value(
                                    &mut d.selected_case,
                                    i,
                                    format!(
                                        "{} · {}",
                                        c.id,
                                        if c.first_failure.is_none() {
                                            "reserve retained"
                                        } else {
                                            "reserve fails"
                                        }
                                    ),
                                );
                                m.choices
                                    .push((format!("case-{i}"), r.rect, ui.clip_rect()));
                            }
                            let clip = controls::popup_content_clip(ui);
                            for choice in &mut m.choices[start..] {
                                choice.2 = clip;
                            }
                        })
                        .response
                })
                .inner;
            record(m, "case", &r, ui);
        });
    }
}
impl LabApp {
    pub(super) fn capacity_planning_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = work::context(a, &source) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (Goal::CapacityPlanning { definition }, State::CapacityPlanning { simulation }) =
            (&source.goal, &a.state)
        else {
            return;
        };
        let before = (**simulation).clone();
        let mut next = before.clone();
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
        lesson_presentation::rich(ui, RichText::new(&source.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, egui::Button::new(label).selected(view == v));
                    record(&mut self.metrics.capacity_planning, key, &r, ui);
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
            lesson_presentation::concepts(
                ui,
                &source,
                "Builds on recorded-host measurements and filesystem capacity.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(
                ui,
                "A different supplied volume demonstrates the calculation and planning decisions. Use Try example to test it without changing the assignment.",
            );
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.capacity_planning, key, &r, ui);
                    if r.clicked() {
                        next.view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        } else {
            if !practice {
                lesson_presentation::assignment(ui, &source);
            }
            let (t, d): (&work::Task, &mut work::Draft) = if practice {
                (&definition.example, &mut next.practice)
            } else {
                (&definition.initial, &mut next.assignment)
            };
            ui.push_id((i, practice), |ui| {
                if ui.available_width() > 940. {
                    let width = ui.available_width();
                    let left = (width - controls::GAP) * 0.52;
                    let right = width - controls::GAP - left;
                    ui.horizontal_top(|ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(left, 0.),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_width(left);
                                plot(ui, t, d, &mut self.metrics.capacity_planning);
                                quotes(ui, t);
                            },
                        );
                        ui.allocate_ui_with_layout(
                            egui::vec2(right, 0.),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_width(right);
                                plan_controls(
                                    ui,
                                    t,
                                    d,
                                    &mut self.metrics.capacity_planning,
                                    active,
                                    practice,
                                    &source,
                                );
                            },
                        );
                    });
                } else {
                    plot(ui, t, d, &mut self.metrics.capacity_planning);
                    quotes(ui, t);
                    ui.separator();
                    plan_controls(
                        ui,
                        t,
                        d,
                        &mut self.metrics.capacity_planning,
                        active,
                        practice,
                        &source,
                    );
                }
                results(ui, t, d, &mut self.metrics.capacity_planning, active);
            });
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                let r = ui.add_enabled(active, button("Reset example only"));
                record(&mut self.metrics.capacity_planning, "reset-example", &r, ui);
                if r.clicked() {
                    next.practice = work::Draft::new(&definition.example);
                }
            }
            controls::collapsing_header(ui, "Help and references").show(ui, |ui| {
                lesson_presentation::prose(ui, work::HELP);
                let a = self.progress.attempts.get_mut(&source.id).unwrap();
                let r = ui.add_enabled(
                    active && a.hints < source.hints.len(),
                    button("Show a hint"),
                );
                if r.clicked() {
                    a.hints += 1;
                    self.dirty = true;
                }
                for h in source.hints.iter().take(a.hints) {
                    lesson_presentation::prose(ui, h);
                }
                controls::collapsing_header(ui, "Worked approach · reveals one solution").show(
                    ui,
                    |ui| {
                        lesson_presentation::prose(ui, &source.explanation);
                    },
                );
                for url in &source.sources {
                    ui.hyperlink_to(url, url);
                }
            });
        }
        if next != before && active {
            match definition.bound(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        State::CapacityPlanning {
                            simulation: Box::new(next),
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

#[cfg(test)]
#[test]
fn projection_size_limit_is_painted_as_a_diagnostic_not_silently_omitted() {
    use dlms_learning_lab::resource_measurements::capacity::MAX_KIB;
    let definition: work::Definition = serde_json::from_str(include_str!(
        "../../tests/fixtures/capacity-planning-workspace-v1.json"
    ))
    .unwrap();
    let mut task = definition.initial;
    task.series.capacity_kib = MAX_KIB;
    for (point, used) in
        task.series
            .samples
            .iter_mut()
            .zip([0, MAX_KIB / 2, MAX_KIB - 200, MAX_KIB - 100])
    {
        point.used_kib = used;
    }
    for quote in &mut task.offers {
        quote.capacity_kib = MAX_KIB;
        quote.lead_days = 0;
    }
    for case in &mut task.demands {
        case.daily_kib = 0;
        case.changes.clear();
    }
    task.validate().unwrap();
    let mut draft = work::Draft::new(&task);
    draft.inputs.first = Some(1);
    draft.inputs.last = Some(3);
    let ctx = egui::Context::default();
    let mut metrics = Metrics::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900., 800.),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| plot(ui, &task, &draft, &mut metrics));
        },
    );
    fn diagnostic(s: &egui::Shape) -> bool {
        match s {
            egui::Shape::Vec(shapes) => shapes.iter().any(diagnostic),
            egui::Shape::Text(t) => t.galley.text().contains("Projection limited"),
            _ => false,
        }
    }
    assert!(output.shapes.iter().any(|s| diagnostic(&s.shape)));
    assert!(!draft.checked);
}
