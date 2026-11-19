//! Native saved-WLAN configuration and supplied client-profile investigation.
use super::*;
use dlms_learning_lab::{
    engine::State,
    wlan_identity::{Authentication, Compatibility, ManagementProtection},
    wlan_workspace as work,
};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub field_hints: Vec<(String, String)>,
    pub choices: Vec<(String, egui::Rect, egui::Rect)>,
    pub summary: Option<String>,
    pub worked_answer_visible: bool,
    pub security_facts_visible: bool,
    pub case_results: Vec<(String, bool)>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
    }
}
fn auth(a: Authentication) -> &'static str {
    match a {
        Authentication::Open => "Open",
        Authentication::Wpa2Psk => "WPA2-Personal",
        Authentication::Wpa3Sae => "WPA3-Personal",
    }
}
fn pmf(p: ManagementProtection) -> &'static str {
    match p {
        ManagementProtection::Disabled => "Disabled",
        ManagementProtection::Optional => "Optional",
        ManagementProtection::Required => "Required",
    }
}
/// Reuse the shared cell metrics and reflow rule, retaining the actual label ID
/// so native inputs expose their visible labels to accessibility consumers.
pub(super) fn row<R>(
    ui: &mut egui::Ui,
    text: &str,
    body: impl FnOnce(&mut egui::Ui, f32, egui::Id) -> R,
) -> R {
    let label_width = controls::form_label_width(ui);
    let minimum = ui.text_style_height(&egui::TextStyle::Body) * 8. + 2. * controls::PAD_X as f32;
    if ui.available_width() < label_width + ui.spacing().item_spacing.x + minimum {
        let label = ui.label(text);
        body(ui, ui.available_width(), label.id)
    } else {
        let right = ui.next_widget_position().x + ui.available_width();
        ui.horizontal(|ui| {
            let label =
                controls::cell(ui, label_width, |ui| ui.add(egui::Label::new(text).wrap())).inner;
            let width = (right - ui.next_widget_position().x).max(1.);
            controls::cell(ui, width, |ui| body(ui, width, label.id)).inner
        })
        .inner
    }
}
fn security_task(t: &work::Task) -> bool {
    t.policies.iter().any(|p| {
        p.editable.iter().any(|f| {
            matches!(
                f,
                work::Editable::Authentication
                    | work::Editable::ManagementProtection
                    | work::Editable::Passphrase
            )
        })
    })
}
fn configuration(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &mut work::Draft,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let heading = ui.strong("WLAN configuration");
    m.record(ui, "configuration", &heading);
    for wlan in &t.network.wlans {
        ui.push_id(&wlan.id,|ui|{
            if !wlan.managed {
                ui.separator();
                let mode=if security_task(t){format!(" · {}",auth(wlan.initial.authentication))}else{String::new()};
                lesson_presentation::prose(ui,&format!("{} · read-only: {} · {}{}",wlan.label,wlan.initial.ssid,if wlan.initial.enabled{"Enabled"}else{"Disabled"},mode));
                return;
            }
            ui.label(RichText::new(&wlan.label).strong());
            let policy=t.policies.iter().find(|p|p.wlan==wlan.id).unwrap();
            let before=d.fields.iter().find(|f|f.wlan==wlan.id).unwrap().configuration.clone();
            let mut value=before.clone();
            for field in &policy.editable {
                match field {
                    work::Editable::Enabled=>{
                        let text=if value.enabled{"Enabled"}else{"Disabled"};
                        let r=row(ui,"Availability",|ui,_width,label|ui.checkbox(&mut value.enabled,text).labelled_by(label));
                        m.record(ui,&format!("enabled-{}",wlan.id),&r);
                    }
                    work::Editable::Ssid|work::Editable::Passphrase=>{
                        let (key,label,hint,text)=if *field==work::Editable::Ssid {("ssid","Network name","Exact network name",&mut value.ssid)}else{("passphrase","Training passphrase","Training value",&mut value.training_passphrase)};
                        let r=row(ui,label,|ui,width,id|ui.add(controls::singleline_fitting(ui,text,width.min(284.)).font(egui::TextStyle::Monospace).hint_text(hint).char_limit(128)).labelled_by(id));
                        m.record(ui,&format!("{key}-{}",wlan.id),&r);m.fields.push((format!("{key}-{}",wlan.id),r.rect,r.id,ui.clip_rect()));m.field_hints.push((format!("{key}-{}",wlan.id),hint.into()));
                    }
                    work::Editable::Authentication=>{
                        let response=row(ui,"Personal security",|ui,width,id|{
                            let combo=controls::dropdown(ui,"authentication",width.min(284.)).selected_text(auth(value.authentication)).show_ui(ui,|ui|{
                                for choice in [Authentication::Open,Authentication::Wpa2Psk,Authentication::Wpa3Sae]{let r=ui.selectable_value(&mut value.authentication,choice,auth(choice));m.choices.push((format!("auth-{}",auth(choice)),r.rect,controls::popup_content_clip(ui)));}
                            });combo.response.labelled_by(id)
                        });m.record(ui,&format!("authentication-{}",wlan.id),&response);
                    }
                    work::Editable::ManagementProtection=>{
                        let response=row(ui,"Management protection",|ui,width,id|{
                            let combo=controls::dropdown(ui,"management-protection",width.min(284.)).selected_text(pmf(value.management_protection)).show_ui(ui,|ui|{
                                for choice in [ManagementProtection::Disabled,ManagementProtection::Optional,ManagementProtection::Required]{let r=ui.selectable_value(&mut value.management_protection,choice,pmf(choice));m.choices.push((format!("pmf-{}",pmf(choice)),r.rect,controls::popup_content_clip(ui)));}
                            });combo.response.labelled_by(id)
                        });m.record(ui,&format!("protection-{}",wlan.id),&response);
                    }
                }
            }
            if !policy.editable.contains(&work::Editable::Ssid) {row(ui,"Network name",|ui,_,_|{ui.add(egui::Label::new(RichText::new(&value.ssid).monospace()).wrap());});}
            if !policy.editable.contains(&work::Editable::Authentication){lesson_presentation::prose(ui,"Security and training passphrase are preconfigured and read-only in this task.");}
            if value!=before{*notice=d.set(t,&wlan.id,value).err();}
            if d.checked()&&let Ok(rows)=t.rows(&d.fields){
                for r in rows.iter().filter(|r|!r.correct&&r.id.starts_with(&format!("field-{}-",wlan.id))){lesson_presentation::rich(ui,RichText::new(&r.feedback).color(palette(ui).error));}
            }
        });
    }
}
fn actual_label(result: &Compatibility, detailed: bool) -> String {
    if !detailed && matches!(result, Compatibility::Compatible { .. }) {
        return "Configuration compatible".into();
    }
    match result {
        Compatibility::Compatible {
            authentication,
            protected_management,
            ..
        } => format!(
            "Configuration compatible · {} · management frames {}",
            auth(*authentication),
            if *protected_management {
                "protected"
            } else {
                "not protected"
            }
        ),
        Compatibility::Deferred => "Not evaluated — fix the configuration fields".into(),
        Compatibility::NameNotFound => "No matching network name".into(),
        Compatibility::Disabled => "Matching WLAN is disabled".into(),
        Compatibility::AuthenticationMismatch => "Authentication method does not match".into(),
        Compatibility::ManagementProtectionRequired => {
            "Required management protection is unavailable".into()
        }
        Compatibility::PassphraseMismatch => "Training passphrase differs".into(),
    }
}
fn profiles(ui: &mut egui::Ui, t: &work::Task, d: &work::Draft, m: &mut Metrics) {
    let detailed = security_task(t);
    m.security_facts_visible = detailed;
    let r = ui.strong("Supplied client profiles");
    m.record(ui, "profiles", &r);
    let rows = if d.checked() {
        t.rows(&d.fields).ok()
    } else {
        None
    };
    let report = if d.checked() {
        t.preview(&d.fields)
            .and_then(|v| t.network.inspect(&v))
            .ok()
    } else {
        None
    };
    for c in &t.network.clients {
        let card = ui.push_id(&c.id, |ui| {
            let r = ui.label(RichText::new(&c.label).strong());
            m.record(ui, &format!("client-{}", c.id), &r);
            ui.add(
                egui::Label::new(RichText::new(format!("Saved name: {}", c.ssid)).monospace())
                    .wrap(),
            );
            if detailed {
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "Allows {} · {}",
                        c.allowed_authentication
                            .iter()
                            .map(|a| auth(*a))
                            .collect::<Vec<_>>()
                            .join(" / "),
                        if c.management_protection_capable {
                            "offers PMF"
                        } else {
                            "does not offer PMF"
                        }
                    ),
                );
                if c.allowed_authentication
                    .iter()
                    .any(|a| *a != Authentication::Open)
                {
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!("Training value: {}", c.training_passphrase))
                                .monospace(),
                        )
                        .wrap(),
                    );
                }
            }
            let case = t.cases.iter().find(|case| case.client == c.id).unwrap();
            let expected = match &case.expected {
                work::Expected::Compatible { authentication, .. } => {
                    if detailed {
                        format!("Required: compatible using {}", auth(*authentication))
                    } else {
                        "Required: configuration compatible".into()
                    }
                }
                work::Expected::Rejected => "Required: incompatible".into(),
            };
            lesson_presentation::prose(ui, &expected);
            if let Some(report) = &report {
                let actual = &report.clients[&c.id];
                let row = rows
                    .as_ref()
                    .and_then(|rows| rows.iter().find(|r| r.id == format!("client-{}", c.id)));
                let correct = row.is_some_and(|r| r.correct);
                lesson_presentation::rich(
                    ui,
                    RichText::new(format!(
                        "{} · Actual: {}",
                        if matches!(actual, Compatibility::Deferred) {
                            "Not evaluated"
                        } else if correct {
                            "Meets requirement"
                        } else {
                            "Needs attention"
                        },
                        actual_label(actual, detailed)
                    ))
                    .color(if matches!(actual, Compatibility::Deferred) {
                        palette(ui).text
                    } else if correct {
                        palette(ui).success
                    } else {
                        palette(ui).error
                    }),
                );
                if let Some(row) = row {
                    m.case_results.push((c.id.clone(), correct));
                    if !correct {
                        lesson_presentation::prose(ui, &row.feedback);
                    }
                }
            }
            ui.separator();
        });
        m.record(ui, &format!("client-card-{}", c.id), &card.response);
    }
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
    let failed = rows.iter().filter(|r| !r.correct).count();
    let summary = if failed == 0 {
        if practice {
            "Example meets the requirements".into()
        } else {
            "Assignment complete".into()
        }
    } else {
        format!(
            "Needs attention · {failed} requirement{} fail{}",
            if failed == 1 { "" } else { "s" },
            if failed == 1 { "s" } else { "" }
        )
    };
    let r = lesson_presentation::rich(
        ui,
        RichText::new(&summary).strong().color(if failed == 0 {
            palette(ui).success
        } else {
            palette(ui).error
        }),
    );
    m.record(ui, "result", &r);
    m.summary = Some(summary);
    for row in rows
        .iter()
        .filter(|r| !r.correct && r.id.starts_with("policy-"))
    {
        lesson_presentation::rich(
            ui,
            RichText::new(format!("{}: {}", row.label, row.feedback)).color(palette(ui).error),
        );
    }
    if rows.iter().any(|r| r.id.starts_with("field-")) {
        ui.label("Client checks wait for valid configuration fields.");
    }
}
fn workspace_actions(
    ui: &mut egui::Ui,
    task: &work::Task,
    draft: &mut work::Draft,
    source: &Scenario,
    practice: bool,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    configuration(ui, task, draft, m, notice);
    if practice {
        lesson_presentation::practice_step(ui, source, 1);
        let r = ui.add(button("Fill worked configuration"));
        m.record(ui, "fill", &r);
        if r.clicked() {
            *notice = draft.fill_reference(task).err();
        }
    }
    let r = ui.add(primary(ui, "Check configuration"));
    m.record(ui, "check", &r);
    if r.clicked() {
        *notice = draft.check(task).err();
    }
    feedback(ui, task, draft, practice, m);
}
impl LabApp {
    pub(super) fn wlan_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let State::WlanProfiles { simulation } = &attempt.state else {
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
        self.metrics.wlan = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (choice, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == choice));
                    self.metrics.wlan.record(ui, key, &r);
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
        ui.push_id((i, view as u8, "wlan-workspace"), |ui| {
            if view == teaching::View::Introduction {
                lesson_presentation::concepts(
                    ui,
                    &source,
                    "Read the supplied WLAN and client profiles before editing.",
                    Vec::<(String, String)>::new(),
                );
                lesson_presentation::worked(
                    ui,
                    "A different supplied site demonstrates the method.",
                );
                lesson_presentation::worked_rows(ui, &source);
                ui.horizontal_wrapped(|ui| {
                    for (choice, key, label) in [
                        (teaching::View::Example, "try", "Try this example"),
                        (teaching::View::Assignment, "start", "Open full assignment"),
                    ] {
                        let r = ui.add_enabled(active, button(label));
                        self.metrics.wlan.record(ui, key, &r);
                        if r.clicked() {
                            next.view = choice;
                            self.lesson_scroll_reset = true;
                        }
                    }
                });
            } else {
                let practice = view == teaching::View::Example;
                let (task, draft) = if practice {
                    (&definition.example, &mut next.practice)
                } else {
                    (&definition.initial, &mut next.assignment)
                };
                if practice {
                    lesson_presentation::practice_step(ui, &source, 0);
                } else {
                    lesson_presentation::assignment(ui, &source);
                }
                if practice {
                    ui.separator();
                }
                ui.add_enabled_ui(active, |ui| {
                    let column_min = controls::form_label_width(ui)
                        + ui.spacing().item_spacing.x
                        + ui.text_style_height(&egui::TextStyle::Body) * 8.
                        + 2. * controls::PAD_X as f32;
                    if ui.available_width() >= column_min * 2. + ui.spacing().item_spacing.x {
                        ui.columns(2, |cols| {
                            workspace_actions(
                                &mut cols[0],
                                task,
                                draft,
                                &source,
                                practice,
                                &mut self.metrics.wlan,
                                &mut self.notice,
                            );
                            profiles(&mut cols[1], task, draft, &mut self.metrics.wlan);
                        });
                    } else {
                        workspace_actions(
                            ui,
                            task,
                            draft,
                            &source,
                            practice,
                            &mut self.metrics.wlan,
                            &mut self.notice,
                        );
                        ui.separator();
                        profiles(ui, task, draft, &mut self.metrics.wlan);
                    }
                    if practice {
                        lesson_presentation::practice_step(ui, &source, 2);
                        let r = ui.add_enabled(active, button("Reset example only"));
                        self.metrics.wlan.record(ui, "reset-example", &r);
                        if r.clicked() {
                            *draft = work::Draft::new(task);
                            self.notice = None;
                        }
                    }
                    controls::collapsing_header(ui, "Hints and explanation · optional").show(
                        ui,
                        |ui| {
                            let attempt = self.progress.attempts.get_mut(&source.id).unwrap();
                            if ui
                                .add_enabled(
                                    active && attempt.hints < source.hints.len(),
                                    button("Show a hint"),
                                )
                                .clicked()
                            {
                                attempt.hints += 1;
                                self.dirty = true;
                            }
                            for hint in source.hints.iter().take(attempt.hints) {
                                lesson_presentation::prose(ui, hint);
                            }
                            controls::collapsing_header(ui, "Why does this work?").show(ui, |ui| {
                                lesson_presentation::prose(ui, &source.explanation);
                            });
                            controls::collapsing_header(
                                ui,
                                "Show worked answer · reveals one configuration",
                            )
                            .show(ui, |ui| {
                                self.metrics.wlan.worked_answer_visible = true;
                                for field in &task.reference {
                                    let w = task
                                        .network
                                        .wlans
                                        .iter()
                                        .find(|w| w.id == field.wlan)
                                        .unwrap();
                                    ui.strong(&w.label);
                                    for kind in &task
                                        .policies
                                        .iter()
                                        .find(|p| p.wlan == field.wlan)
                                        .unwrap()
                                        .editable
                                    {
                                        let text = match kind {
                                            work::Editable::Enabled => format!(
                                                "Availability: {}",
                                                if field.configuration.enabled {
                                                    "Enabled"
                                                } else {
                                                    "Disabled"
                                                }
                                            ),
                                            work::Editable::Ssid => format!(
                                                "Network name: {}",
                                                field.configuration.ssid
                                            ),
                                            work::Editable::Authentication => format!(
                                                "Personal security: {}",
                                                auth(field.configuration.authentication)
                                            ),
                                            work::Editable::ManagementProtection => format!(
                                                "Management protection: {}",
                                                pmf(field.configuration.management_protection)
                                            ),
                                            work::Editable::Passphrase => format!(
                                                "Training passphrase: {}",
                                                field.configuration.training_passphrase
                                            ),
                                        };
                                        lesson_presentation::prose(ui, &text);
                                    }
                                }
                            });
                            controls::collapsing_header(ui, "Simulation scope and references")
                                .show(ui, |ui| {
                                    lesson_presentation::prose(ui, work::HELP);
                                    lesson_presentation::prose(ui, &source.simplifications);
                                    for url in &source.sources {
                                        ui.hyperlink_to(url, url);
                                    }
                                });
                        },
                    );
                });
            }
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state = State::WlanProfiles {
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
