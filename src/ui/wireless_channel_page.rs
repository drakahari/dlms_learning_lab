//! Native channel configuration and supplied-survey relationships.
use super::*;
use dlms_learning_lab::{engine::State, wireless_channel_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub summary: Option<String>,
    pub worked_answer_visible: bool,
    pub surveys_deferred: bool,
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub field_feedback: Vec<(String, String)>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
    }
}
fn measure(ui: &egui::Ui, text: &str) -> f32 {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let color = palette(ui).text;
    ui.fonts_mut(|f| f.layout_no_wrap(text.into(), font, color).size().x)
}
fn channel_strip(ui: &mut egui::Ui, m: &mut Metrics) {
    ui.strong("The site’s separated choices: channels 1, 6 and 11");
    let height = ui.text_style_height(&egui::TextStyle::Body) * 2.;
    let (r, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    let p = palette(ui);
    let font = egui::TextStyle::Body.resolve(ui.style());
    let x = |mhz: f32| r.left() + (mhz - 2400.) / 83. * r.width();
    for (channel, centre) in [(1, 2412.), (6, 2437.), (11, 2462.)] {
        let b = egui::Rect::from_min_max(
            egui::pos2(x(centre - 10.), r.top()),
            egui::pos2(x(centre + 10.), r.bottom()),
        );
        ui.painter().rect_filled(b, 4., p.accent);
        ui.painter().rect_stroke(
            b,
            4.,
            egui::Stroke::new(1_f32, p.border),
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            b.center(),
            egui::Align2::CENTER_CENTER,
            channel.to_string(),
            font.clone(),
            p.on_accent,
        );
    }
    m.record(ui, "channel-strip", &response);
    lesson_presentation::prose(
        ui,
        "Each block is one 20 MHz channel in the supplied 2.4 GHz plan. Nearby channel numbers can occupy overlapping space; this is a planning illustration, not a spectrum measurement.",
    );
}
fn radios(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &mut work::Draft,
    active: bool,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    ui.strong("Channel configuration");
    let field_width = measure(ui, "99999999") + 2. * controls::PAD_X as f32;
    let label_width = t
        .network
        .radios
        .iter()
        .map(|r| measure(ui, &r.label))
        .fold(measure(ui, "Access point"), f32::max)
        + ui.spacing().item_spacing.x;
    let role_width = measure(ui, "Fixed neighbour") + ui.spacing().item_spacing.x;
    let wide = ui.available_width()
        >= label_width + field_width + role_width + ui.spacing().item_spacing.x * 2.;
    if wide {
        ui.horizontal(|ui| {
            for (text, width) in [
                ("Access point", label_width),
                ("Channel", field_width),
                ("Authority", role_width),
            ] {
                controls::cell(ui, width, |ui| ui.strong(text));
            }
        });
    }
    for radio in &t.network.radios {
        ui.push_id(&radio.id, |ui| {
            let label_id;
            if wide {
                label_id = ui
                    .horizontal(|ui| {
                        let label =
                            controls::cell(ui, label_width, |ui| ui.label(&radio.label)).inner;
                        if radio.managed {
                            let field = d.fields.iter().find(|f| f.radio == radio.id).unwrap();
                            let mut value = field.value.clone();
                            let r = controls::cell(ui, field_width, |ui| {
                                ui.add_enabled(
                                    active,
                                    controls::singleline_fitting(ui, &mut value, field_width)
                                        .hint_text("1–11")
                                        .char_limit(8),
                                )
                            })
                            .inner
                            .labelled_by(label.id);
                            m.record(ui, &format!("field-{}", radio.id), &r);
                            if r.changed() {
                                *notice = d.set(t, &radio.id, value).err();
                            }
                        } else {
                            let r = controls::cell(ui, field_width, |ui| {
                                ui.monospace(radio.initial_channel.to_string())
                            })
                            .inner;
                            m.record(ui, &format!("fixed-{}", radio.id), &r);
                        }
                        let role = controls::cell(ui, role_width, |ui| {
                            ui.label(if radio.managed {
                                "Managed"
                            } else {
                                "Fixed neighbour"
                            })
                        })
                        .inner;
                        m.record(ui, &format!("role-{}", radio.id), &role);
                        label
                    })
                    .inner;
            } else {
                label_id = ui.label(RichText::new(&radio.label).strong());
                if radio.managed {
                    let role = ui.label("Channel · managed radio");
                    m.record(ui, &format!("role-{}", radio.id), &role);
                    let mut value = d
                        .fields
                        .iter()
                        .find(|f| f.radio == radio.id)
                        .unwrap()
                        .value
                        .clone();
                    let r = ui
                        .add_enabled(
                            active,
                            controls::singleline_fitting(
                                ui,
                                &mut value,
                                field_width.min(ui.available_width()),
                            )
                            .hint_text("1–11")
                            .char_limit(8),
                        )
                        .labelled_by(label_id.id);
                    m.record(ui, &format!("field-{}", radio.id), &r);
                    if r.changed() {
                        *notice = d.set(t, &radio.id, value).err();
                    }
                } else {
                    let r = ui.label(format!(
                        "Fixed neighbour · channel {}",
                        radio.initial_channel
                    ));
                    m.record(ui, &format!("fixed-{}", radio.id), &r);
                }
            }
            m.record(ui, &format!("label-{}", radio.id), &label_id);
            if d.checked()
                && let Ok(rows) = t.rows(&d.fields)
            {
                for row in rows.iter().filter(|r| {
                    !r.correct
                        && (r.id == format!("field-{}", radio.id)
                            || r.id == format!("radio-{}", radio.id))
                }) {
                    lesson_presentation::rich(
                        ui,
                        RichText::new(&row.feedback).color(palette(ui).error),
                    );
                    m.field_feedback
                        .push((radio.id.clone(), row.feedback.clone()));
                }
            }
            if !wide {
                ui.add_space(ui.spacing().item_spacing.y);
            }
        });
    }
}
fn surveys(ui: &mut egui::Ui, t: &work::Task, m: &mut Metrics) {
    let r = ui.strong("Supplied survey relationships");
    m.record(ui, "surveys", &r);
    for survey in &t.network.surveys {
        ui.strong(&survey.label);
        for pair in &survey.neighbours {
            let a = &t
                .network
                .radios
                .iter()
                .find(|r| r.id == pair.a)
                .unwrap()
                .label;
            let b = &t
                .network
                .radios
                .iter()
                .find(|r| r.id == pair.b)
                .unwrap()
                .label;
            lesson_presentation::prose(ui, &format!("{a} ↔ {b}"));
        }
    }
    lesson_presentation::prose(
        ui,
        "Each shown pair can hear one another. Check plan evaluates every pair in every supplied survey.",
    );
}
fn feedback(ui: &mut egui::Ui, t: &work::Task, d: &work::Draft, practice: bool, m: &mut Metrics) {
    if !d.checked() {
        ui.label("Not checked since editing");
        return;
    }
    let rows = match t.rows(&d.fields) {
        Ok(r) => r,
        Err(e) => {
            ui.label(e);
            return;
        }
    };
    let count = rows.iter().filter(|r| !r.correct).count();
    let summary = if count == 0 {
        if practice {
            "Example meets the plan".into()
        } else {
            "Assignment complete".into()
        }
    } else {
        format!(
            "Needs attention · {count} condition{} fail{}",
            if count == 1 { "" } else { "s" },
            if count == 1 { "s" } else { "" }
        )
    };
    m.summary = Some(summary.clone());
    let r = lesson_presentation::rich(
        ui,
        RichText::new(summary).color(if count == 0 {
            palette(ui).success
        } else {
            palette(ui).error
        }),
    );
    m.record(ui, "result", &r);
    for row in rows
        .iter()
        .filter(|r| !r.correct && !r.id.starts_with("radio-") && !r.id.starts_with("field-"))
    {
        let r = lesson_presentation::rich(
            ui,
            RichText::new(format!("{}: {}", row.label, row.feedback)).color(palette(ui).error),
        );
        m.record(ui, &format!("feedback-{}", row.id), &r);
    }
    if rows.iter().any(|r| r.id.starts_with("field-")) {
        m.surveys_deferred = true;
        ui.label("Survey checks will run after every managed radio has a valid channel number.");
    }
    let passed = rows.len() - count;
    if passed > 0 {
        let r = controls::collapsing_header(ui, &format!("Show {passed} passing conditions"))
            .id_salt("wireless-passing")
            .show(ui, |ui| {
                for row in rows.iter().filter(|r| r.correct) {
                    lesson_presentation::prose(
                        ui,
                        &format!("Verified · {}: {}", row.label, row.feedback),
                    );
                }
            });
        m.record(ui, "passing", &r.header_response);
    }
}
impl LabApp {
    pub(super) fn wireless_channel_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let previous = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if previous != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let attempt = &self.progress.attempts[&source.id];
        let definition =
            match work::source(&source).and_then(|d| work::context(attempt, &source).map(|()| d)) {
                Ok(d) => d,
                Err(e) => {
                    self.notice = Some(e);
                    return;
                }
            };
        let State::WirelessChannels { simulation } = &attempt.state else {
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
        self.metrics.wireless_channel = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (choice, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == choice));
                    self.metrics.wireless_channel.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = choice;
                        self.lesson_scroll_reset = true;
                        self.notice = None;
                    }
                }
            });
            next.view = view;
        }
        ui.push_id((i,view as u8),|ui|{
            if view==teaching::View::Introduction{
                lesson_presentation::concepts(ui,&source,"Read the supplied radios and surveys; no real Wi-Fi setup is needed.",Vec::<(String,String)>::new());
                channel_strip(ui,&mut self.metrics.wireless_channel);lesson_presentation::worked(ui,"A different supplied site demonstrates the reasoning before independent work.");
                lesson_presentation::worked_rows(ui,&source);
                ui.horizontal_wrapped(|ui|{for(choice,key,label)in[(teaching::View::Example,"try","Try this example"),(teaching::View::Assignment,"start","Open full assignment")]{let r=ui.add_enabled(active,button(label));self.metrics.wireless_channel.record(ui,key,&r);if r.clicked(){next.view=choice;self.lesson_scroll_reset=true;}}});
            }else{
                let practice=view==teaching::View::Example;if !practice{lesson_presentation::assignment(ui,&source);}
                let(task,draft)=if practice{(&definition.example,&mut next.practice)}else{(&definition.initial,&mut next.assignment)};
                ui.add_enabled_ui(active,|ui|{
                    if practice{lesson_presentation::practice_step(ui,&source,0);}
                    if practice { ui.separator(); }
                    if ui.available_width()>ui.text_style_height(&egui::TextStyle::Body)*50.{ui.columns(2,|cols|{radios(&mut cols[0],task,draft,active,&mut self.metrics.wireless_channel,&mut self.notice);surveys(&mut cols[1],task,&mut self.metrics.wireless_channel);});}else{radios(ui,task,draft,active,&mut self.metrics.wireless_channel,&mut self.notice);surveys(ui,task,&mut self.metrics.wireless_channel);}
                    if practice{lesson_presentation::practice_step(ui,&source,1);let r=ui.add_enabled(active,button("Fill worked plan"));self.metrics.wireless_channel.record(ui,"fill",&r);if r.clicked(){self.notice=draft.fill_reference(task).err();}}
                    let r=ui.add_enabled(active,button("Check plan").fill(palette(ui).accent));self.metrics.wireless_channel.record(ui,"check",&r);if r.clicked(){self.notice=draft.check(task).err();}
                    feedback(ui,task,draft,practice,&mut self.metrics.wireless_channel);
                    if practice{lesson_presentation::practice_step(ui,&source,2);let r=ui.add_enabled(active,button("Reset example only"));self.metrics.wireless_channel.record(ui,"reset-example",&r);if r.clicked(){*draft=work::Draft::new(task);self.notice=None;}}
                    controls::collapsing_header(ui,"Hints and explanation · optional").show(ui,|ui|{
                        let attempt=self.progress.attempts.get_mut(&source.id).unwrap();let r=ui.add_enabled(active&&attempt.hints<source.hints.len(),button("Show a hint"));if r.clicked(){attempt.hints+=1;self.dirty=true;}for hint in source.hints.iter().take(attempt.hints){lesson_presentation::prose(ui,hint);}
                        controls::collapsing_header(ui,"Why the channel numbers matter").show(ui,|ui|channel_strip(ui,&mut self.metrics.wireless_channel));
                        controls::collapsing_header(ui,"Show worked answer · reveals one plan").show(ui,|ui|{self.metrics.wireless_channel.worked_answer_visible=true;for field in &task.reference{let radio=task.network.radios.iter().find(|r|r.id==field.radio).unwrap();ui.label(format!("{}: channel {}",radio.label,field.value));}lesson_presentation::prose(ui,"This is one possible plan, not a required sequence. Check plan grades the supplied relationships and site requirements.");});
                        controls::collapsing_header(ui,"Simulation scope and references").show(ui,|ui|{lesson_presentation::prose(ui,work::HELP);lesson_presentation::prose(ui,&source.simplifications);for url in &source.sources{ui.hyperlink_to(url,url);}});
                    });
                });
            }
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state = State::WirelessChannels {
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
