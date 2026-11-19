//! Native read-only MIB investigation beside meaningful monitor configuration.
use super::wlan_page::row;
use super::*;
use dlms_learning_lab::{
    engine::State,
    snmp::{
        Reply, Value, Width,
        monitor::{Direction, LinkRule, Outcome, UnknownCause},
    },
    snmp_workspace as work,
};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub choices: Vec<(String, egui::Rect, egui::Rect)>,
    pub summary: Option<String>,
    pub worked_answer_visible: bool,
    pub query_text: Option<String>,
    pub case_results: Vec<(String, bool)>,
    pub case_feedback: Vec<(String, String)>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
    }
}
fn field(
    ui: &mut egui::Ui,
    key: &str,
    label: &str,
    value: &mut String,
    chars: usize,
    hint: &str,
    m: &mut Metrics,
) -> bool {
    let r = ui
        .push_id(key, |ui| {
            row(ui, label, |ui, width, id| {
                ui.add(
                    controls::singleline_fitting(ui, value, width.min(284.))
                        .font(egui::TextStyle::Monospace)
                        .char_limit(chars)
                        .hint_text(hint),
                )
                .labelled_by(id)
            })
        })
        .inner;
    m.record(ui, key, &r);
    m.fields.push((key.into(), r.rect, r.id, ui.clip_rect()));
    r.changed()
}
fn choice<T: Copy + PartialEq + std::fmt::Debug>(
    ui: &mut egui::Ui,
    key: &str,
    label: &str,
    value: &mut T,
    options: &[(T, &str)],
    m: &mut Metrics,
) -> bool {
    let old = *value;
    row(ui, label, |ui, width, id| {
        let r = controls::dropdown(ui, key, width.min(284.))
            .selected_text(
                options
                    .iter()
                    .find(|o| o.0 == *value)
                    .map(|o| o.1)
                    .unwrap_or("Choose"),
            )
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (v, text) in options {
                    let r = ui.selectable_value(value, *v, *text);
                    m.choices
                        .push((format!("{key}-{v:?}"), r.rect, ui.clip_rect()));
                }
            });
        r.response.clone().labelled_by(id);
        m.record(ui, key, &r.response);
    });
    old != *value
}
fn unknown(c: UnknownCause) -> &'static str {
    match c {
        UnknownCause::UnavailableAgent => "agent unavailable",
        UnknownCause::MissingInterface => "interface absent",
        UnknownCause::AmbiguousDescription => "description ambiguous",
        UnknownCause::ChangedInterface => "interface changed",
        UnknownCause::MissingMeasurement => "sample missing",
        UnknownCause::ChangedCapacity => "capacity changed",
        UnknownCause::UnusableCounter => "counter not comparable",
    }
}
fn outcome(v: Outcome) -> String {
    match v {
        Outcome::Quiet => "No alert".into(),
        Outcome::Alert => "Alert".into(),
        Outcome::Unknown(c) => format!("Unknown · {}", unknown(c)),
    }
}
fn rule(v: LinkRule) -> &'static str {
    match v {
        LinkRule::OperationalDown => "Any operationally Down link",
        LinkRule::UnplannedDown => "Admin Up and Oper Down",
    }
}
fn value(v: &Value) -> String {
    match v {
        Value::Text(t) => format!("Text: {t}"),
        Value::Integer(v) => format!("Integer: {v}"),
        Value::Gauge32(v) => format!("Gauge32: {v}"),
        Value::Counter32(v) => format!("Counter32: {v}"),
        Value::Counter64(v) => format!("Counter64: {v}"),
        Value::TimeTicks(v) => format!("TimeTicks: {v} · 100 ticks = 1 second"),
    }
}
fn reply(r: &Reply) -> String {
    match r{Reply::Value{oid,value:v}=>format!("{}\n{}",oid.dotted(),value(v)),Reply::NoSuchObject=>"No such object: this virtual agent does not implement that measurement.".into(),Reply::NoSuchInstance=>"No such instance: the measurement exists, but not at this row. Scalar values use .0; interface values use an index from ifDescr.".into(),Reply::EndOfMibView=>"End of the supported MIB view: there is no later object in this virtual table.".into()}
}
fn settings(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &mut work::Draft,
    m: &mut Metrics,
    practice: bool,
    notice: &mut Option<String>,
) {
    ui.strong("Monitor configuration");
    let before = d.fields.clone();
    field(
        ui,
        "description",
        "Interface description",
        &mut d.fields.description,
        64,
        "Exact interface description",
        m,
    );
    match t.activity {
        work::Activity::Link => {
            choice(
                ui,
                "rule",
                "Alert rule",
                &mut d.fields.link_rule,
                &[
                    (LinkRule::UnplannedDown, rule(LinkRule::UnplannedDown)),
                    (LinkRule::OperationalDown, rule(LinkRule::OperationalDown)),
                ],
                m,
            );
        }
        work::Activity::Traffic => {
            choice(
                ui,
                "direction",
                "Traffic direction",
                &mut d.fields.direction,
                &[
                    (Direction::Incoming, "Incoming"),
                    (Direction::Outgoing, "Outgoing"),
                ],
                m,
            );
            choice(
                ui,
                "width",
                "Counter width",
                &mut d.fields.width,
                &[
                    (Width::Counter32, "32-bit"),
                    (Width::Counter64, "64-bit high-capacity"),
                ],
                m,
            );
            field(
                ui,
                "threshold",
                "Warning at %",
                &mut d.fields.threshold,
                3,
                "1–100",
                m,
            );
        }
    }
    if before != d.fields {
        d.changed()
    }
    ui.horizontal_wrapped(|ui| {
        if practice {
            let r = ui.add(button("Fill worked monitor"));
            m.record(ui, "fill", &r);
            if r.clicked() {
                *notice = d.fill(t).err();
            }
        }
        let r = ui.add(primary(ui, "Check monitor"));
        m.record(ui, "check", &r);
        if r.clicked() {
            *notice = d.check(t).err();
        }
    });
    let report = if d.checked() {
        t.report(&d.fields).ok()
    } else {
        None
    };
    let (status, color) = if let Some(r) = &report {
        if r.passed() {
            (
                "All monitoring requirements verified".to_string(),
                palette(ui).success,
            )
        } else {
            let count = if r.configuration_error.is_some() {
                t.cases.len()
            } else {
                r.cases.iter().filter(|c| !c.passed).count()
            };
            (
                format!(
                    "Needs attention · {count} {}",
                    if count == 1 {
                        "case remains"
                    } else {
                        "cases remain"
                    }
                ),
                palette(ui).error,
            )
        }
    } else {
        (
            "Configuration not checked since editing".into(),
            palette(ui).muted,
        )
    };
    m.summary = Some(
        if !d.checked() {
            "unchecked"
        } else if report.as_ref().is_some_and(|r| r.passed()) {
            "pass"
        } else {
            "fail"
        }
        .into(),
    );
    let r = ui.label(RichText::new(status).color(color));
    m.record(ui, "result", &r);
    if let Some(error) = report.and_then(|r| r.configuration_error) {
        let r = lesson_presentation::prose(ui, &error);
        m.record(ui, "configuration-feedback", &r);
    }
}
fn browser(ui: &mut egui::Ui, t: &work::Task, d: &mut work::Draft, m: &mut Metrics) {
    ui.strong("Inspect supplied agent · simulated GET / GETNEXT");
    row(ui, "Case", |ui, width, id| {
        let selected = t
            .cases
            .iter()
            .find(|c| c.id == d.query.case)
            .map(|c| c.label.as_str())
            .unwrap_or("Choose");
        let r = controls::dropdown(ui, "mib-case", width.min(284.))
            .selected_text(selected)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for c in &t.cases {
                    let r = ui.selectable_value(&mut d.query.case, c.id.clone(), &c.label);
                    m.choices
                        .push((format!("case-{}", c.id), r.rect, ui.clip_rect()));
                }
            });
        r.response.clone().labelled_by(id);
        m.record(ui, "case", &r.response);
    });
    let has_before = t
        .cases
        .iter()
        .find(|c| c.id == d.query.case)
        .is_some_and(|c| c.stream.before.is_some());
    if let Some(c) = t.cases.iter().find(|c| c.id == d.query.case)
        && let Some(earlier) = &c.stream.before
    {
        ui.label(format!(
            "Samples: earlier {} s, current {} s ({} seconds apart)",
            earlier.at_seconds,
            c.stream.after.at_seconds,
            c.stream.after.at_seconds.saturating_sub(earlier.at_seconds)
        ));
    }
    row(ui, "Sample", |ui, width, id| {
        let r = controls::dropdown(ui, "mib-sample", width.min(284.))
            .selected_text(if d.query.before {
                "Earlier sample"
            } else {
                "Current sample"
            })
            .show_ui(ui, |ui| {
                for (v, label, enabled) in [
                    (false, "Current sample", true),
                    (true, "Earlier sample", has_before),
                ] {
                    let r = ui.add_enabled(enabled, button(label).selected(d.query.before == v));
                    if r.clicked() {
                        d.query.before = v;
                        ui.close();
                    }
                    m.choices
                        .push((format!("sample-{v}"), r.rect, ui.clip_rect()));
                }
            });
        r.response.clone().labelled_by(id);
        m.record(ui, "sample", &r.response);
    });
    choice(
        ui,
        "operation",
        "Operation",
        &mut d.query.next,
        &[
            (false, "GET · one exact object"),
            (true, "GETNEXT · next object"),
        ],
        m,
    );
    field(
        ui,
        "query-oid",
        "Object ID",
        &mut d.query.oid,
        96,
        "Dotted numbers",
        m,
    );
    ui.horizontal_wrapped(|ui| {
        for (key, label, oid, next) in [
            ("names", "Names column", "1.3.6.1.2.1.2.2.1.2", true),
            ("uptime", "Uptime", "1.3.6.1.2.1.1.3.0", false),
        ] {
            let r = ui.add(button(label));
            m.record(ui, key, &r);
            if r.clicked() {
                d.query.oid = oid.into();
                d.query.next = next;
            }
        }
        let r = ui.add(button("Poll object"));
        m.record(ui, "poll", &r);
        if r.clicked() {
            let _ = d.poll(t);
        }
    });
    let output=ui.vertical(|ui|{
  ui.strong("Actual query result");
  if let Some(result)=d.observed(t){match result{
   Ok(r)=>{let text=reply(&r);m.query_text=Some(text.clone());for line in text.lines(){ui.add(egui::Label::new(RichText::new(line).monospace()).wrap());}if matches!(r,Reply::Value{value:Value::Integer(_),..}){ui.label("State objects: 1 = Up; 2 = Down.");}},
   Err(e)=>{m.query_text=Some(format!("Query error: {e}"));ui.label(RichText::new("Query needs attention").color(palette(ui).error));lesson_presentation::prose(ui,&e);}
  }}else{ui.label("Not polled since choosing these inputs. Presets fill the request; Poll object reads it.");}
 });
    m.record(ui, "query-output", &output.response);
}
fn mistake(t: &work::Task, d: &work::Draft, r: &work::CaseResult) -> String {
    if d.fields.description != t.reference.description {
        return format!(
            "This monitor targets {:?}; the task names {:?}. Read the interface descriptions and select the requested target.",
            d.fields.description, t.reference.description
        );
    }
    if matches!(r.actual.outcome, Outcome::Unknown(_)) {
        return r.actual.explanation.clone();
    }
    if t.activity == work::Activity::Link
        && r.expected == Outcome::Quiet
        && r.actual.outcome == Outcome::Alert
        && d.fields.link_rule == LinkRule::OperationalDown
    {
        return "This interface is administratively Down: a shutdown was requested. The down-only rule mistakes planned maintenance for an unexpected failure.".into();
    }
    if t.activity == work::Activity::Traffic {
        if d.fields.direction != t.reference.direction {
            return "The selected counter measures traffic leaving the interface. This task asks for traffic entering it; compare the Incoming and Outgoing measurements.".into();
        }
        let advice = if r.expected == Outcome::Quiet && r.actual.outcome == Outcome::Alert {
            "A threshold that includes this ordinary load creates an unnecessary warning."
        } else {
            "The threshold is too high to catch this required busy case."
        };
        return format!("{} {advice}", r.actual.explanation);
    }
    r.actual.explanation.clone()
}
fn cases(ui: &mut egui::Ui, t: &work::Task, d: &work::Draft, m: &mut Metrics) {
    ui.strong("Required outcomes");
    let report = if d.checked() {
        t.report(&d.fields).ok()
    } else {
        None
    };
    if report
        .as_ref()
        .is_some_and(|r| r.configuration_error.is_some())
    {
        ui.label("No cases evaluated until the configuration error above is corrected.");
    }
    for c in &t.cases {
        let row = ui.vertical(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong(&c.label);
                ui.label(format!("Required: {}", outcome(c.expected)));
            });
            if let Some(actual) = report
                .as_ref()
                .and_then(|r| r.cases.iter().find(|v| v.id == c.id))
            {
                m.case_results.push((c.id.clone(), actual.passed));
                ui.label(
                    RichText::new(format!(
                        "{} · actual: {}",
                        if actual.passed {
                            "Meets requirement"
                        } else {
                            "Needs attention"
                        },
                        outcome(actual.actual.outcome)
                    ))
                    .color(if actual.passed {
                        palette(ui).success
                    } else {
                        palette(ui).error
                    }),
                );
                if !actual.passed {
                    let detail = mistake(t, d, actual);
                    m.case_feedback.push((c.id.clone(), detail.clone()));
                    lesson_presentation::prose(ui, &detail);
                }
                ui.push_id(&c.id, |ui| {
                    controls::collapsing_header(ui, "Show measurement and MIB reads").show(
                        ui,
                        |ui| {
                            lesson_presentation::prose(ui, &actual.actual.explanation);
                            for p in &actual.actual.polls {
                                ui.monospace(format!(
                                    "{} {} {}",
                                    if p.before { "Earlier" } else { "Current" },
                                    if p.next { "GETNEXT" } else { "GET" },
                                    p.requested.dotted()
                                ));
                                lesson_presentation::prose(ui, &reply(&p.reply));
                            }
                        },
                    )
                });
            }
            ui.separator();
        });
        m.record(ui, &format!("case-{}", c.id), &row.response);
    }
}
fn worked(ui: &mut egui::Ui, t: &work::Task) {
    ui.monospace(format!(
        "Interface description: {}",
        t.reference.description
    ));
    match t.activity {
        work::Activity::Link => {
            ui.label(format!("Alert rule: {}", rule(t.reference.link_rule)));
        }
        work::Activity::Traffic => {
            ui.label(format!(
                "Direction: {:?} · counter: {:?} · warning: {}%",
                t.reference.direction, t.reference.width, t.reference.threshold
            ));
        }
    }
    ui.label(
        "One supported configuration. The checker compares behavior across all supplied cases.",
    );
}
impl LabApp {
    pub(super) fn snmp_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let previous = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if previous != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let attempt = &self.progress.attempts[&source.id];
        let def =
            match work::source(&source).and_then(|d| work::context(attempt, &source).map(|()| d)) {
                Ok(d) => d,
                Err(e) => {
                    self.notice = Some(e);
                    return;
                }
            };
        let State::SnmpMonitor { simulation } = &attempt.state else {
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
        self.metrics.snmp = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    self.metrics.snmp.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                        self.notice = None;
                    }
                }
            });
            next.view = view;
        }
        ui.push_id((i,view as u8,challenge,"snmp-monitor"),|ui|{
   if view==teaching::View::Introduction{
    lesson_presentation::concepts(ui,&source,"Read supplied measurements and configure a useful monitor.",Vec::<(String,String)>::new());
    lesson_presentation::worked(ui,"A different site demonstrates the method.");lesson_presentation::worked_rows(ui,&source);
    ui.horizontal_wrapped(|ui|{for(v,key,label)in[(teaching::View::Example,"try","Try this example"),(teaching::View::Assignment,"start","Open full assignment")]{let r=ui.add_enabled(active,button(label));self.metrics.snmp.record(ui,key,&r);if r.clicked(){next.view=v;self.lesson_scroll_reset=true;}}});
   }else{
    let practice=view==teaching::View::Example;let(t,d)=if practice{(&def.example,&mut next.practice)}else{(&def.initial,&mut next.assignment)};
    if !practice{
     ui.label(RichText::new("YOUR TASK").small().color(palette(ui).accent));
     let goal=lesson_presentation::entry(&source).map(|e|e.goal.as_str()).unwrap_or(&source.summary);ui.label(RichText::new(goal).size(22.));
     if let Some(e)=lesson_presentation::entry(&source) && let Some(first)=e.requirements.first(){lesson_presentation::prose(ui,first);}
     ui.label("Meet every required outcome below. The supplied agent measurements are read-only.");
    }
    ui.add_enabled_ui(active,|ui|{
     let minimum=controls::form_label_width(ui)+ui.text_style_height(&egui::TextStyle::Body)*9.+2.*controls::PAD_X as f32+controls::GAP;
     if ui.available_width()>=minimum*2.+controls::GAP{
      ui.columns(2,|cols|{
       if practice{lesson_presentation::practice_step(&mut cols[0],&source,0);}
       browser(&mut cols[0],t,d,&mut self.metrics.snmp);
       if practice{lesson_presentation::practice_step(&mut cols[1],&source,1);}
       settings(&mut cols[1],t,d,&mut self.metrics.snmp,practice,&mut self.notice);
      });
     }else{
      if practice{lesson_presentation::practice_step(ui,&source,0);}
      browser(ui,t,d,&mut self.metrics.snmp);ui.separator();
      if practice{lesson_presentation::practice_step(ui,&source,1);}
      settings(ui,t,d,&mut self.metrics.snmp,practice,&mut self.notice);
     }
     ui.add_space(controls::GAP);cases(ui,t,d,&mut self.metrics.snmp);
     if practice{lesson_presentation::practice_step(ui,&source,2);let r=ui.add(button("Reset example only"));self.metrics.snmp.record(ui,"reset-example",&r);if r.clicked(){d.reset(t);self.notice=None;}}
     controls::collapsing_header(ui,"Hints and explanation · optional").show(ui,|ui|{
      let a=self.progress.attempts.get_mut(&source.id).unwrap();let r=ui.add_enabled(active&&a.hints<source.hints.len(),button("Show a hint"));if r.clicked(){a.hints+=1;self.dirty=true;}
      for hint in source.hints.iter().take(a.hints){lesson_presentation::prose(ui,hint);}
      controls::collapsing_header(ui,"Why does this work?").show(ui,|ui|lesson_presentation::prose(ui,&source.explanation));
      controls::collapsing_header(ui,"Show worked answer · reveals one monitor policy").show(ui,|ui|{self.metrics.snmp.worked_answer_visible=true;worked(ui,t);});
      controls::collapsing_header(ui,"Supported objects, scope and references").show(ui,|ui|{
       lesson_presentation::prose(ui,work::HELP);lesson_presentation::prose(ui,&source.simplifications);
       ui.label("Object-ID input accepts up to 96 characters; GET/GETNEXT use only the supplied virtual table. Names column fills an ifDescr GETNEXT request; Uptime fills a scalar GET request. Neither preset polls automatically.");
       for url in &source.sources{ui.hyperlink_to(url,url);}
      });
     });
    });
   }
  });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state = State::SnmpMonitor {
                simulation: Box::new(next),
            };
            self.completion_index.forget(&source.id);
            self.dirty = true;
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
        if self.metrics.completion {
            lesson_presentation::prose(ui, &source.explanation);
            self.next_unit_lab(ui, i);
        }
    }
}
