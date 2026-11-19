//! Focused native profile editors and in-process path investigation.
use super::linux_v6_page::Metrics;
use super::*;
use dlms_learning_lab::{
    vpn_point::{Material, Role},
    vpn_point_source as binding,
    vpn_point_task::Task,
    vpn_point_workspace::{self as work, Action},
};
fn apply(s: &mut work::Scope, t: &Task, a: Action, notice: &mut Option<String>) {
    if let Err(e) = s.apply(t, &a) {
        *notice = Some(e);
    }
}
fn editor(
    ui: &mut egui::Ui,
    t: &Task,
    s: &mut work::Scope,
    peer: usize,
    m: &mut Metrics,
    n: &mut Option<String>,
) {
    let label = ui.strong(format!(
        "{} · virtual profile",
        t.initial.supplied.peers[peer].label
    ));
    let mut text = s.editors[peer].clone();
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let color = ui.visuals().text_color();
    let mut layouter = |ui: &egui::Ui, buffer: &dyn egui::TextBuffer, _width: f32| {
        ui.fonts_mut(|f| f.layout_no_wrap(buffer.as_str().to_owned(), font.clone(), color))
    };
    let height =
        ui.text_style_height(&egui::TextStyle::Monospace) * 10. + 2. * f32::from(controls::PAD_Y);
    let size = egui::vec2(
        ui.available_width(),
        height + ui.spacing().scroll.allocated_width(),
    );
    let (outer, r) = controls::fixed_viewport(ui, size, |ui| {
        egui::ScrollArea::both()
            .id_salt(("vpn-profile-scroll", peer))
            .auto_shrink([false, false])
            .min_scrolled_height(height)
            .max_height(size.y)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .id_salt(("vpn-profile-editor", peer))
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(10)
                        .char_limit(8192)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .lock_focus(false)
                        .layouter(&mut layouter),
                )
            })
    });
    m.controls
        .push((format!("editor-fixed-{peer}"), outer, ui.clip_rect()));
    m.controls.push((
        format!("editor-{peer}"),
        r.inner.rect.intersect(r.inner_rect),
        ui.clip_rect(),
    ));
    m.controls.push((
        format!("editor-viewport-{peer}"),
        r.inner_rect,
        ui.clip_rect(),
    ));
    let response = r.inner.labelled_by(label.id);
    m.fields
        .push((format!("editor-{peer}"), response.rect, response.id));
    m.focus.push((format!("editor-{peer}"), response.id));
    if text != s.editors[peer] {
        apply(s, t, Action::Edit { peer, text }, n);
    }
    ui.horizontal_wrapped(|ui| {
        let r = ui.add(button("Save profile"));
        m.record(ui, &format!("save-{peer}"), &r);
        if r.clicked() {
            apply(s, t, Action::Save { peer }, n);
        }
        ui.label(if s.editors[peer] == s.world.saved[peer] {
            "Saved"
        } else {
            "Unsaved changes"
        });
    });
}
fn facts(ui: &mut egui::Ui, t: &Task) {
    for f in &t.initial.supplied.files {
        let detail = match &f.material {
            Material::Ca { id } => format!("Authority · {id}"),
            Material::Certificate {
                subject,
                issuer,
                key_id,
                role,
                valid,
            } => format!(
                "Certificate · name {subject} · issuer {issuer} · {} purpose · pair {key_id} · {} supplied validity",
                if *role == Role::Server {
                    "server"
                } else {
                    "client"
                },
                if *valid { "valid" } else { "not valid" }
            ),
            Material::Key { key_id } => {
                format!("Private-key reference · pair {key_id}; no actual key material")
            }
        };
        if ui.available_width() > ui.text_style_height(&egui::TextStyle::Body) * 48. {
            ui.columns(2, |cols| {
                lesson_presentation::monospace(&mut cols[0], &f.path, true);
                lesson_presentation::prose(&mut cols[1], &detail);
            });
        } else {
            lesson_presentation::monospace(ui, &f.path, true);
            lesson_presentation::prose(ui, &detail);
        }
        ui.add_space(4.);
    }
}
fn query(
    ui: &mut egui::Ui,
    t: &Task,
    s: &mut work::Scope,
    m: &mut Metrics,
    n: &mut Option<String>,
) {
    ui.strong("Test a modeled path");
    let mut q = s.query.clone();
    let before = q.clone();
    controls::form_row(ui, "From LAN", |ui, w| {
        let r = controls::dropdown(ui, "vpn-query-source", w)
            .selected_text(&t.initial.supplied.peers[q.from].label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (i, p) in t.initial.supplied.peers.iter().enumerate() {
                    let r = ui.selectable_value(&mut q.from, i, &p.label);
                    m.record(ui, &format!("from-choice:{i}"), &r);
                }
            });
        controls::retain_dropdown_focus(ui, &r.response);
        m.record(ui, "from-choice", &r.response);
    });
    if q.from != before.from
        && let Some(c) = t.cases.iter().find(|c| {
            c.from == q.from && c.expected == dlms_learning_lab::vpn_point_task::Outcome::Reachable
        })
    {
        q.source = c.source.to_string();
        q.destination = c.destination.to_string();
    }
    for (key, label, value) in [
        ("source", "Source host", &mut q.source),
        ("destination", "Destination", &mut q.destination),
    ] {
        controls::form_row(ui, label, |ui, _| {
            let r = shared_libraries_page::fitting_input(
                ui,
                value,
                egui::Id::new(("vpn-path-field", key)),
                64,
                ui.is_enabled(),
                "Dotted-decimal IPv4 address",
            );
            m.record(ui, key, &r.response);
            m.fields.push((key.into(), r.response.rect, r.response.id));
            if let Some(v) = r.value {
                *value = v;
            }
        });
    }
    if q != before {
        apply(s, t, Action::Query { query: q }, n);
    }
    let r = ui.add(button("Test path"));
    m.record(ui, "probe", &r);
    if r.clicked() {
        apply(s, t, Action::Probe, n);
    }
    if let Some(p) = s.visible_probe() {
        let r = ui.strong(format!(
            "Actual model result · {}",
            match p.outcome {
                dlms_learning_lab::vpn_point_task::Outcome::Reachable =>
                    "Outward and return routes pass",
                dlms_learning_lab::vpn_point_task::Outcome::OutsideTunnel =>
                    "Stays outside the VPN",
                dlms_learning_lab::vpn_point_task::Outcome::SelectedTunnel =>
                    "Unexpected tunnel selection",
                dlms_learning_lab::vpn_point_task::Outcome::MissingOutbound =>
                    "Outward route missing",
                dlms_learning_lab::vpn_point_task::Outcome::MissingReturn => "Return route missing",
                dlms_learning_lab::vpn_point_task::Outcome::Unavailable =>
                    "Peer prerequisites need attention",
            }
        ));
        m.record(ui, "probe-result", &r);
        lesson_presentation::prose(ui, &p.detail);
    }
}
fn results(ui: &mut egui::Ui, s: &work::Scope, t: &Task, practice: bool, m: &mut Metrics) {
    if !s.checked {
        return;
    }
    let checks = s.checks(t);
    let passed = checks.iter().filter(|c| c.passed).count();
    let r = ui.strong(if passed == checks.len() {
        if practice {
            "Example checks passed".into()
        } else {
            "Assignment complete".into()
        }
    } else if s.world.applied.is_none() {
        "Not complete · apply saved profiles to evaluate their behavior".into()
    } else {
        format!("Not complete · {passed}/{} outcomes pass", checks.len())
    });
    m.result = Some((passed, checks.len(), r.rect));
    m.record(ui, "result", &r);
    let mut groups: Vec<(&str, Vec<&str>)> = vec![];
    for c in checks.iter().filter(|c| !c.passed) {
        if let Some((_, labels)) = groups.iter_mut().find(|(detail, _)| *detail == c.detail) {
            labels.push(&c.label);
        } else {
            groups.push((&c.detail, vec![&c.label]));
        }
    }
    for (i, (detail, labels)) in groups.iter().enumerate() {
        ui.strong(if labels.len() == 1 {
            format!("Needs work · {}", labels[0])
        } else {
            format!("Needs work · {} outcomes share this cause", labels.len())
        });
        let r = lesson_presentation::prose(ui, detail);
        m.record(ui, "failed-case", &r);
        if labels.len() > 1 {
            controls::collapsing_header(ui, &format!("Affected outcomes ({})", labels.len()))
                .id_salt(("vpn-failure", i))
                .show(ui, |ui| {
                    for label in labels {
                        lesson_presentation::prose(ui, &format!("Not passed · {label}"));
                    }
                });
        }
    }
    if passed > 0 {
        controls::collapsing_header(ui, &format!("Inspect passing checks ({passed})")).show(
            ui,
            |ui| {
                for c in checks.iter().filter(|c| c.passed) {
                    lesson_presentation::prose(ui, &format!("Passed · {}\n{}", c.label, c.detail));
                }
            },
        );
    }
}
impl LabApp {
    pub(super) fn vpn_point_lesson(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase=(self.selected,self.reset_pending);
        let challenge=self.workspace_header(ui,i);
        if phase!=(self.selected,self.reset_pending){return;}
        let source = self.scenarios[i].clone();
        let original = &self.progress.attempts[&source.id];
        let d = match binding::source(&source) {
            Ok(d) => d,
            Err(e) => {
                lesson_presentation::prose(ui, &e);
                return;
            }
        };
        if let Err(e) = binding::context(original, &source) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let engine::State::VpnPoint { simulation } = &original.state else {
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
        lesson_presentation::rich(ui, RichText::new(&source.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    self.metrics.vpn_point.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        }
        next.view = view;
        let practice = view == teaching::View::Example;
        if view == teaching::View::Introduction {
            let r = lesson_presentation::concepts(ui, &source, "", Vec::<(String, String)>::new());
            self.metrics
                .vpn_point
                .record(ui, "introduction-content", &r.objective);
            lesson_presentation::worked(
                ui,
                "Expected example settings and observations; the assignment uses separate peers.",
            );
            let r = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &source));
            self.metrics.vpn_point.record(ui, "worked", &r.response);
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            self.metrics.vpn_point.record(ui, "try-example", &r);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            let (t, s) = if practice {
                (&d.example, &mut next.example)
            } else {
                (&d.assignment, &mut next.assignment)
            };
            if !practice {
                let r = lesson_presentation::assignment(ui, &source);
                self.metrics.vpn_point.record(ui, "assignment-content", &r);
            }
            ui.add_enabled_ui(active,|ui|{
                let labels=t.initial.supplied.peers.each_ref().map(|p|p.label.as_str());
                let endpoints=t.initial.supplied.peers.each_ref().map(|p|p.endpoint.to_string());
                let lans=t.initial.lans.map(|n|format!("LAN {}/{}",n.address,n.prefix));
                let r=controls::topology_chain(ui,&[[labels[0],&endpoints[0],&lans[0]],[labels[1],&endpoints[1],&lans[1]]],"Supplied peers · simulated IPv4 paths, not an actual tunnel.");self.metrics.vpn_point.record(ui,"topology",&r);
                if ui.available_width()>=ui.text_style_height(&egui::TextStyle::Body)*54.{
                    // A real guidance row reserves the taller column before either editor.
                    if practice{ui.columns(2,|cols|{for(peer,col)in cols.iter_mut().enumerate(){lesson_presentation::practice_step(col,&source,peer);}});}
                    ui.columns(2,|cols|{for(peer,col)in cols.iter_mut().enumerate(){editor(col,t,s,peer,&mut self.metrics.vpn_point,&mut self.notice);}});
                }else{for peer in 0..2{if practice{lesson_presentation::practice_step(ui,&source,peer);}editor(ui,t,s,peer,&mut self.metrics.vpn_point,&mut self.notice);ui.add_space(12.);}}
                controls::collapsing_header(ui,"Supplied virtual identity files · inspect names and pairs").id_salt(("vpn-identity-files",&source.id,practice)).default_open(source.id=="lpic2-vpn-peer-profile").show(ui,|ui|facts(ui,t));
                ui.horizontal_wrapped(|ui|{
                    let r=ui.add(button("Apply profiles"));self.metrics.vpn_point.record(ui,"apply",&r);if r.clicked(){apply(s,t,Action::Apply,&mut self.notice);}
                    ui.label(if s.world.applied.is_none(){"Not applied"}else if s.world.applied_source.as_ref()!=Some(&s.world.saved){"Saved changes not applied"}else if s.editors!=s.world.saved{"Applied saved profiles · editor changes unsaved"}else{"Saved profiles applied"});
                });
                if let Some(n)=s.visible_notice(){let r=ui.strong(&n.kind);self.metrics.vpn_point.record(ui,"notice",&r);lesson_presentation::prose(ui,&n.message);}
                if practice{lesson_presentation::practice_step(ui,&source,2);}
                query(ui,t,s,&mut self.metrics.vpn_point,&mut self.notice);
                ui.separator();let r=ui.add(primary(ui,if practice{"Check example"}else{"Check configuration"}));self.metrics.vpn_point.record(ui,"check",&r);if r.clicked(){apply(s,t,Action::Check,&mut self.notice);}
                results(ui,s,t,practice,&mut self.metrics.vpn_point);
                if s.solved(t){lesson_presentation::prose(ui,if practice{"Apply this reasoning independently to the assignment."}else{&source.explanation});}
                if practice{controls::collapsing_header(ui,"Show worked answer · example only").id_salt(("vpn-example-worked",&source.id)).show(ui,|ui|{
                    for peer in 0..2{if let Some(text)=d.worked.iter().find_map(|a|match a{Action::Edit{peer:p,text}if *p==peer=>Some(text),_=>None}){ui.strong(&t.initial.supplied.peers[peer].label);lesson_presentation::monospace(ui,text,true);let r=ui.add(button("Fill worked example draft"));self.metrics.vpn_point.record(ui,&format!("fill-worked-{peer}"),&r);if r.clicked(){apply(s,t,Action::Edit{peer,text:text.clone()},&mut self.notice);}}}
                    lesson_presentation::prose(ui,"Filling changes only this example editor. Save each changed profile, Apply profiles, then test both directions.");
                });}
            });
            controls::collapsing_header(ui, "Help, supported subset and references").id_salt(("vpn-help",&source.id)).show(
                ui,
                |ui| {
                    if !challenge {
                        if ui.add(button("Show a hint")).clicked() {
                            let a = self.progress.attempts.get_mut(&source.id).unwrap();
                            a.hints = (a.hints + 1).min(source.hints.len());
                            self.dirty = true;
                        }
                        for hint in source
                            .hints
                            .iter()
                            .take(self.progress.attempts[&source.id].hints)
                        {
                            lesson_presentation::prose(ui, hint);
                        }
                        controls::collapsing_header(
                            ui,
                            "Worked approach · reveals assignment guidance",
                        )
                        .id_salt(("vpn-assignment-worked",&source.id))
                        .show(ui, |ui| {
                            lesson_presentation::prose(ui, &source.explanation);
                        });
                    }
                    lesson_presentation::prose(ui, binding::HELP);
                    lesson_presentation::prose(ui, &source.simplifications);
                    for url in &source.sources {
                        ui.hyperlink_to(url, url);
                    }
                },
            );
        }
        if active && next != before {
            match d.bound(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::VpnPoint {
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
        Self::workspace_footer(ui);
    }
}
