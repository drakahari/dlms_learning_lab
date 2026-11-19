//! Native request workspace; learner fields never leave the in-process model.
use super::*;
use dlms_learning_lab::{rest_controller as api, rest_workspace as work};

#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub ids: Vec<(String, egui::Id)>,
    pub shown_outcomes: Vec<(bool, String)>,
    pub practice_level: Option<usize>,
    pub shown_diagnostic: Option<String>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, response: &egui::Response) {
        self.controls
            .push((key.into(), response.rect, ui.clip_rect()));
        self.ids.push((key.into(), response.id));
    }
}
pub(super) fn practice_level(task: &work::Task, draft: &work::Draft) -> usize {
    if draft.checked && draft.checks(task).iter().all(|c| c.0) {
        return 2;
    }
    if task
        .requirements
        .iter()
        .any(|r| matches!(r, work::Requirement::AbsentProfile { .. }))
    {
        let targets: Vec<_> = task
            .requirements
            .iter()
            .filter(|r| {
                matches!(
                    r,
                    work::Requirement::Network {
                        requirement: dlms_learning_lab::switching::Requirement::AccessVlan { .. },
                    }
                )
            })
            .collect();
        let moved = !targets.is_empty()
            && draft
                .store(task)
                .and_then(|s| task.api.project(&s))
                .is_ok_and(|fabric| {
                    targets.iter().all(|r| match r {
                        work::Requirement::Network { requirement } => requirement.check(&fabric).0,
                        _ => false,
                    })
                });
        return if moved {
            2
        } else if draft.changes.is_empty() {
            0
        } else {
            1
        };
    }
    // Guide the body repair before it succeeds; this never grants completion.
    let identity = draft
        .request
        .authorization
        .split_once(' ')
        .is_some_and(|(scheme, name)| {
            scheme.eq_ignore_ascii_case("Bearer") && name == "lab-operator"
        });
    let media = match draft.request.method.as_str() {
        "PATCH" => "application/merge-patch+json",
        "PUT" => "application/json",
        _ => return 0,
    };
    let path = task.requirements.iter().find_map(|r| {
        if let work::Requirement::PreserveDescription { name } = r {
            Some(format!("/profiles/{name}"))
        } else {
            None
        }
    });
    usize::from(
        identity
            && draft
                .request
                .content_type
                .trim_matches(' ')
                .eq_ignore_ascii_case(media)
            && path.as_deref() == Some(draft.request.path.as_str()),
    )
}
pub(super) fn response_notice(response: api::Response) -> String {
    format!(
        "Actual response · {} {}\n{}{}",
        response.status,
        match response.status {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            409 => "Conflict",
            415 => "Unsupported Media Type",
            422 => "Unprocessable Content",
            501 => "Not Implemented",
            _ => "",
        },
        response
            .headers
            .iter()
            .map(|(k, v)| format!("{k}: {v}\n"))
            .collect::<String>(),
        if response.body.is_empty() {
            "(empty body)".into()
        } else {
            response.body
        }
    )
}
fn field(
    ui: &mut egui::Ui,
    title: &str,
    key: &str,
    text: &mut String,
    max: usize,
    metrics: &mut Metrics,
) {
    controls::form_row(ui, title, |ui, width| {
        let response = ui.add(
            controls::singleline_fitting(ui, text, width)
                .char_limit(max)
                .hint_text(match key {
                    "path" => "/profiles/name",
                    "authorization" => "Bearer lab-reader",
                    _ => "application/json",
                }),
        );
        metrics.record(ui, key, &response);
    });
}
fn request_editor(
    ui: &mut egui::Ui,
    draft: &mut work::Draft,
    task: &work::Task,
    notice: &mut Option<String>,
    metrics: &mut Metrics,
) {
    let mut request = draft.request.clone();
    controls::form_row(ui, "Method", |ui, width| {
        let result = controls::dropdown(ui, "rest-method", width)
            .selected_text(&request.method)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for value in ["GET", "POST", "PUT", "PATCH", "DELETE"] {
                    let choice = ui.selectable_value(&mut request.method, value.into(), value);
                    metrics.record(ui, &format!("method:{value}"), &choice);
                }
            });
        metrics.record(ui, "method", &result.response);
    });
    field(ui, "Resource path", "path", &mut request.path, 96, metrics);
    field(
        ui,
        "Authorization",
        "authorization",
        &mut request.authorization,
        96,
        metrics,
    );
    field(
        ui,
        "Content-Type",
        "content-type",
        &mut request.content_type,
        96,
        metrics,
    );
    let label = ui.strong("Body · JSON (empty for GET/DELETE)");
    let width = ui.available_width();
    let height = ui.text_style_height(&egui::TextStyle::Monospace).ceil() * 5.
        + 2. * f32::from(controls::PAD_Y);
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let color = palette(ui).text;
    let mut layouter = move |ui: &egui::Ui, text: &dyn egui::TextBuffer, _: f32| {
        ui.fonts_mut(|fonts| fonts.layout_no_wrap(text.as_str().into(), font.clone(), color))
    };
    let (viewport, response) = controls::fixed_viewport(ui, egui::vec2(width, height), |ui| {
        controls::nested_scroll(
            ui,
            egui::ScrollArea::both()
                .id_salt(("rest-body-scroll", &task.id))
                .max_width(width)
                .min_scrolled_width(width)
                .max_height(height)
                .auto_shrink([false, false]),
            |ui| {
                egui::TextEdit::multiline(&mut request.body)
                    .code_editor()
                    .lock_focus(false)
                    .layouter(&mut layouter)
                    .id(egui::Id::new(("rest-body", &task.id)))
                    .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                    .desired_width(
                        (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(24.),
                    )
                    .desired_rows(5)
                    .char_limit(api::MAX_BODY)
                    .show(ui)
                    .response
                    .labelled_by(label.id)
            },
        )
        .inner
    });
    metrics
        .controls
        .push(("body".into(), viewport, ui.clip_rect()));
    metrics.ids.push(("body".into(), response.id));
    if request != draft.request {
        *notice = draft.edit(request).err();
    }
    ui.horizontal_wrapped(|ui| {
        let send = ui.add(primary(ui, "Send request"));
        metrics.record(ui, "send", &send);
        if send.clicked() {
            *notice = Some(match draft.send(task) {
                Ok(response) => response_notice(response),
                Err(e) => format!("Request not sent · {e}"),
            });
        }
        let check = ui.add(button("Check network"));
        metrics.record(ui, "check", &check);
        if check.clicked()
            && let Err(e) = draft.test(task)
        {
            *notice = Some(e);
        }
    });
}
fn feedback(
    ui: &mut egui::Ui,
    draft: &work::Draft,
    task: &work::Task,
    notice: &Option<String>,
    metrics: &mut Metrics,
) {
    if let Some(text) = notice {
        let actual = text.starts_with("Actual response · ");
        let response = lesson_presentation::prose(
            ui,
            if actual {
                "ACTUAL REQUEST RESULT"
            } else {
                "REQUEST FEEDBACK"
            },
        );
        metrics.record(ui, "response", &response);
        if let Some((status, message)) = response_diagnostic(text) {
            lesson_presentation::prose(ui, status);
            let response = lesson_presentation::prose(ui, &message);
            metrics.record(ui, "diagnostic", &response);
            metrics.shown_diagnostic = Some(message);
            controls::collapsing_header(ui, "Raw response · optional")
                .show(ui, |ui| raw_response(ui, task, text));
        } else if actual {
            raw_response(ui, task, text);
        } else {
            let response = lesson_presentation::prose(ui, text);
            metrics.record(ui, "diagnostic", &response);
            metrics.shown_diagnostic = Some(text.clone());
        }
    } else {
        lesson_presentation::prose(
            ui,
            "No request response shown. Send request to inspect its result. Applied changes are retained.",
        );
    }
    ui.add_space(controls::GAP);
    if draft.checked {
        let checks = draft.checks(task);
        let passed = checks.iter().filter(|c| c.0).count();
        let response = ui.strong(format!(
            "{} · {passed}/{} requirements",
            if passed == checks.len() {
                "Network passed"
            } else {
                "Needs repair"
            },
            checks.len()
        ));
        metrics.record(ui, "result", &response);
        for (ok, message) in checks.iter().filter(|c| !c.0) {
            lesson_presentation::prose(ui, &format!("Needs repair · {message}"));
            metrics.shown_outcomes.push((*ok, message.clone()));
        }
        controls::collapsing_header(ui, "All network and preservation results · optional").show(
            ui,
            |ui| {
                for (ok, message) in &checks {
                    lesson_presentation::prose(
                        ui,
                        &format!("{} · {message}", if *ok { "Pass" } else { "Needs repair" }),
                    );
                }
            },
        );
    } else {
        lesson_presentation::prose(
            ui,
            "Network unchecked. Check network evaluates the currently applied resources, separately from request status.",
        );
    }
}
fn raw_response(ui: &mut egui::Ui, task: &work::Task, text: &str) {
    controls::nested_scroll(
        ui,
        egui::ScrollArea::both()
            .id_salt(("rest-response", &task.id))
            .max_height(ui.text_style_height(&egui::TextStyle::Monospace) * 10.)
            .auto_shrink([false, true]),
        |ui| {
            ui.monospace(text);
        },
    );
}
fn response_diagnostic(text: &str) -> Option<(&str, String)> {
    if !text.starts_with("Actual response · ") {
        return None;
    }
    let at = text.find("\n{")?;
    let body: serde_json::Value = serde_json::from_str(&text[at + 1..]).ok()?;
    let message = body.get("error")?.as_str()?;
    let readable = if message.contains("expected a boolean") {
        "The up field needs true or false without quotes. Quoted text is a different type; no resource changed.".into()
    } else if message.contains("expected u16") {
        "Use a whole-number VLAN ID without quotes or a decimal point. No resource changed.".into()
    } else {
        message.to_owned()
    };
    Some((text.lines().next()?, readable))
}
fn mapping(ui: &mut egui::Ui, task: &work::Task) {
    controls::collapsing_header(ui, "Supplied port map and supported behavior · optional").show(
        ui,
        |ui| {
            for (name, slot) in &task.api.ports {
                lesson_presentation::monospace(
                    ui,
                    &format!("{name} → {}/{}", slot.switch, slot.port),
                    true,
                );
            }
            for (name, switch) in &task.api.inventory.fabric.switches {
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "{name} approved VLANs: {}",
                        switch
                            .vlans
                            .iter()
                            .map(u16::to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                );
            }
            lesson_presentation::prose(ui, work::HELP);
        },
    );
}
impl LabApp {
    pub(super) fn rest_controller_page(&mut self, ui: &mut egui::Ui, index: usize) {
        let previous = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, index);
        if previous != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[index].clone();
        let attempt = &self.progress.attempts[&source.id];
        let definition =
            match work::source(&source).and_then(|d| work::context(attempt, &source).map(|()| d)) {
                Ok(d) => d,
                Err(e) => {
                    self.notice = Some(e);
                    return;
                }
            };
        let engine::State::RestController { simulation } = &attempt.state else {
            return;
        };
        let mut next = (**simulation).clone();
        let before = next.clone();
        let active = ui.is_enabled()
            && !(self.reset_pending
                || self.settings_open
                || self.theme_preview
                || self.data_tools.modal_open());
        let mut view = if challenge {
            teaching::View::Assignment
        } else {
            next.view
        };
        self.metrics.rest_controller = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (choice, title, key) in [
                    (teaching::View::Introduction, "Introduction", "introduction"),
                    (teaching::View::Example, "Try example · ungraded", "example"),
                    (teaching::View::Assignment, "Assignment", "assignment"),
                ] {
                    let response = ui.add_enabled(active, button(title).selected(choice == view));
                    self.metrics.rest_controller.record(ui, key, &response);
                    self.metrics.lesson_tabs.push(response.rect);
                    if response.clicked() {
                        view = choice;
                        next.view = choice;
                        self.lesson_scroll_reset = true;
                        self.notice = None;
                    }
                }
            });
        }
        ui.push_id((&source.id,view as u8,challenge),|ui| {
            if view==teaching::View::Introduction {
                lesson_presentation::concepts(ui,&source,"Review JSON inventories and VLAN paths before sending controller requests.",std::iter::empty());lesson_presentation::wrapped_worked_commands(ui,&source);
                ui.horizontal_wrapped(|ui| {for (choice,title) in [(teaching::View::Example,"Try this example"),(teaching::View::Assignment,"Open full assignment")] {if ui.add_enabled(active,button(title)).clicked(){next.view=choice;self.lesson_scroll_reset=true;self.notice=None;}}});return;
            }
            let practice=view==teaching::View::Example;
            let (task,draft)=if practice {(&definition.example,&mut next.practice)}else{(&definition.initial,&mut next.assignment)};
            if practice {
                lesson_presentation::lead(ui,"PRACTICE TASK",&task.instructions);
                let step=practice_level(task,draft);
                self.metrics.rest_controller.practice_level=Some(step);lesson_presentation::practice_step(ui,&source,step);
            }else{lesson_presentation::assignment(ui,&source);}
            ui.add_enabled_ui(active,|ui| {
                if ui.available_width()>=ui.text_style_height(&egui::TextStyle::Body)*54. {
                    ui.columns(2,|cols|{request_editor(&mut cols[0],draft,task,&mut self.notice,&mut self.metrics.rest_controller);feedback(&mut cols[1],draft,task,&self.notice,&mut self.metrics.rest_controller);});
                }else{request_editor(ui,draft,task,&mut self.notice,&mut self.metrics.rest_controller);ui.add_space(controls::GAP);feedback(ui,draft,task,&self.notice,&mut self.metrics.rest_controller);}
                mapping(ui,task);
                controls::collapsing_header(ui,"Hints · optional").show(ui,|ui|{for hint in &source.hints {lesson_presentation::prose(ui,hint);}});
                if practice {controls::collapsing_header(ui,"Worked practice requests · optional").show(ui,|ui| {
                    lesson_presentation::prose(ui,"Insert one request, then use Send request yourself. Inserting does not run it.");
                    for (i,request) in task.reference.iter().enumerate() {
                        lesson_presentation::monospace(ui,&format!("{} {}\n{}",request.method,request.path,request.body),true);
                        if ui.add(button(&format!("Insert worked request {}",i+1))).clicked(){self.notice=draft.edit(request.clone()).err();}
                    }
                    if ui.add(button("Reset example only")).clicked(){*draft=work::Draft::new(task);self.notice=None;}
                });}
            });
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state =
                engine::State::RestController {
                    simulation: Box::new(next),
                };
            self.completion_index.forget(&source.id);
            self.dirty = true;
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
        if self.metrics.completion {
            lesson_presentation::prose(ui, &source.explanation);
            self.next_unit_lab(ui, index);
        }
    }
}
