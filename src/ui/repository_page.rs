//! Native repository configuration and actual cached-metadata inspection.
use super::*;
use dlms_learning_lab::{repository_workspace as work, rpm_repositories as repo};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub headers: Vec<egui::Rect>,
    pub rows: Vec<[egui::Rect; 4]>,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn heading(ui: &mut egui::Ui, title: &str) {
    ui.add_space(8.);
    lesson_presentation::rich(ui, RichText::new(title).strong());
}
fn status(s: repo::CacheStatus) -> &'static str {
    match s {
        repo::CacheStatus::Missing => "No cached metadata",
        repo::CacheStatus::WrongSource => "Different source",
        repo::CacheStatus::Older => "Older snapshot",
        repo::CacheStatus::Current => "Current published snapshot",
    }
}
fn inspection(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    src: &repo::Source,
    m: &mut Metrics,
    _active: bool,
) {
    heading(ui, "Applied configuration and cache");
    let rows: Vec<_> = s
        .world
        .inspect_all(src)
        .unwrap()
        .into_iter()
        .map(|r| {
            (
                r.configuration.alias.clone(),
                r.configuration.enabled,
                r.endpoint
                    .map(|e| e.id.clone())
                    .unwrap_or_else(|| "No supplied source".into()),
                r.cached
                    .map(|c| format!("{} · {}", c.endpoint, c.snapshot.as_str()))
                    .unwrap_or_else(|| "None".into()),
                status(r.cache_status),
            )
        })
        .collect();
    controls::nested_scroll(
        ui,
        egui::ScrollArea::horizontal().id_salt("repository-state"),
        |ui| {
            egui::Grid::new("repository-inspection")
                .num_columns(4)
                .spacing(egui::vec2(14., 8.))
                .striped(true)
                .show(ui, |ui| {
                    for title in [
                        "Alias",
                        "Queries",
                        "Configured source",
                        "Cached source / snapshot",
                    ] {
                        m.headers.push(ui.strong(title).rect);
                    }
                    ui.end_row();
                    for (alias, enabled, endpoint, cached, label) in rows {
                        let a = ui.monospace(&alias).rect;
                        let b = ui.label(if enabled { "Enabled" } else { "Disabled" }).rect;
                        let c = ui.monospace(endpoint).rect;
                        let d = ui
                            .vertical(|ui| {
                                ui.monospace(cached);
                                ui.label(label);
                            })
                            .response
                            .rect;
                        m.rows.push([a, b, c, d]);
                        ui.end_row();
                    }
                });
        },
    );
}
fn editor(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    src: &repo::Source,
    m: &mut Metrics,
    active: bool,
    actions: &mut Vec<work::Event>,
) {
    heading(ui, "Repository settings");
    let aliases: Vec<_> = s
        .world
        .inspect_all(src)
        .unwrap()
        .iter()
        .map(|r| r.configuration.alias.clone())
        .collect();
    ui.horizontal_wrapped(|ui| {
        for alias in aliases {
            let r = ui.add_enabled(
                active && !s.dirty,
                egui::Button::new(&alias).selected(s.alias == alias),
            );
            record(m, &format!("edit-{alias}"), &r, ui);
            if r.clicked() {
                s.load(src, &alias).expect("known alias");
            }
        }
    });
    controls::form_row(ui, "Alias", |ui, _| {
        ui.monospace(&s.alias);
    });
    controls::form_row(ui, "Used in queries", |ui, _| {
        let r = ui.add_enabled(active, egui::Checkbox::new(&mut s.enabled, "Enabled"));
        record(m, "enabled", &r, ui);
        if r.changed() {
            s.dirty = true;
            s.checked = false;
        }
    });
    controls::form_row(ui, "Source URI label", |ui, width| {
        let r = ui.add_enabled(
            active,
            controls::singleline(ui, &mut s.uri)
                .desired_width((width - 2. * f32::from(controls::PAD_X)).max(1.))
                .hint_text("file:///virtual-repositories/name")
                .char_limit(160),
        );
        m.fields.push(("uri".into(), r.rect, r.id));
        record(m, "uri", &r, ui);
        if r.changed() {
            s.dirty = true;
            s.checked = false;
        }
    });
    ui.horizontal_wrapped(|ui| {
        let r = ui.add_enabled(active, button("Apply settings"));
        record(m, "apply", &r, ui);
        if r.clicked() {
            actions.push(work::Event::Edit {
                edit: work::Edit::Settings {
                    alias: s.alias.clone(),
                    enabled: s.enabled,
                    uri: s.uri.clone(),
                },
            });
        }
        let r = ui.add_enabled(active && s.dirty, button("Discard draft"));
        record(m, "discard", &r, ui);
        if r.clicked() {
            let alias = s.alias.clone();
            s.load(src, &alias).expect("known alias");
        }
        let r = ui.add_enabled(active && !s.dirty, button("Refresh metadata"));
        record(m, "refresh", &r, ui);
        if r.clicked() {
            actions.push(work::Event::Edit {
                edit: work::Edit::Refresh {
                    alias: s.alias.clone(),
                },
            });
        }
    });
    if s.dirty {
        lesson_presentation::prose(
            ui,
            "Unapplied settings · Apply settings or Discard draft before switching aliases or refreshing. Queries still inspect the applied configuration.",
        );
    }
    heading(ui, "Supplied virtual sources");
    for e in &src.endpoints {
        lesson_presentation::rich(
            ui,
            RichText::new(format!(
                "{} · {} · publishes {}",
                e.id,
                if e.delivery == repo::Delivery::Available {
                    "Available"
                } else {
                    "Unavailable"
                },
                e.published.as_str()
            ))
            .strong(),
        );
        lesson_presentation::monospace(ui, &e.uri, true);
        let r = ui.add_enabled(active, button("Use address in draft"));
        record(m, &format!("use-source-{}", e.id), &r, ui);
        if r.clicked() {
            s.uri = e.uri.clone();
            s.dirty = true;
            s.checked = false;
        }
    }
    lesson_presentation::prose(
        ui,
        "Use address in draft fills the field. Apply settings changes the selected alias.",
    );
}
fn query_input(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    m: &mut Metrics,
    active: bool,
    actions: &mut Vec<work::Event>,
) {
    heading(ui, "Inspect cached package candidates");
    lesson_presentation::prose(
        ui,
        "Reads all enabled sources’ caches. Selecting an alias above chooses which settings you edit; it does not filter this query.",
    );
    controls::form_row(ui, "Exact package name", |ui, width| {
        let r = ui.add_enabled(
            active,
            controls::singleline(ui, &mut s.package)
                .desired_width((width.min(400.) - 2. * f32::from(controls::PAD_X)).max(1.))
                .hint_text("Literal package name")
                .char_limit(32),
        );
        m.fields.push(("package".into(), r.rect, r.id));
        record(m, "package", &r, ui);
        if r.changed() {
            s.checked = false;
        }
    });
    let r = ui.add_enabled(active, button("Query exact package"));
    record(m, "query", &r, ui);
    if r.clicked() {
        actions.push(work::Event::Query {
            package: s.package.clone(),
        });
    }
}
fn actual(ui: &mut egui::Ui, d: &work::Definition, s: &work::Session, practice: bool) {
    let Some((package, world, fresh)) = d.observed(s, practice).expect("bound history") else {
        lesson_presentation::prose(
            ui,
            "No actual query yet. Enter a name and choose Query exact package.",
        );
        return;
    };
    let src = if practice {
        &d.example_source
    } else {
        &d.initial_source
    };
    let q = world
        .query_exact(src, &package)
        .expect("recorded successful query");
    heading(ui, &format!("Actual query · {package}"));
    if !fresh {
        lesson_presentation::prose(
            ui,
            "Previous configuration/cache state · run this query again after the applied changes for current handover evidence.",
        );
    }
    if q.rows.is_empty() {
        lesson_presentation::prose(
            ui,
            "No candidate rows in this query. Read any source omissions below; an empty result is not an installation result.",
        );
    } else {
        controls::nested_scroll(
            ui,
            egui::ScrollArea::horizontal().id_salt("repository-candidates"),
            |ui| {
                egui::Grid::new("repository-query-rows")
                    .num_columns(6)
                    .spacing(egui::vec2(14., 8.))
                    .striped(true)
                    .show(ui, |ui| {
                        for text in [
                            "Alias",
                            "Source",
                            "Snapshot",
                            "Package",
                            "Version / release",
                            "Architecture",
                        ] {
                            ui.strong(text);
                        }
                        ui.end_row();
                        for row in &q.rows {
                            for text in [
                                row.alias,
                                row.endpoint,
                                row.snapshot.as_str(),
                                row.archive.identity.name.as_str(),
                                row.archive.evr.as_str(),
                                match row.archive.identity.arch {
                                    dlms_learning_lab::rpm_packages::Architecture::X86_64 => {
                                        "x86_64"
                                    }
                                    dlms_learning_lab::rpm_packages::Architecture::Noarch => {
                                        "noarch"
                                    }
                                },
                            ] {
                                ui.monospace(text);
                            }
                            ui.end_row();
                        }
                    });
            },
        );
    }
    for (alias, omission) in q.omitted {
        lesson_presentation::prose(
            ui,
            &format!(
                "Omitted {alias}: {}.",
                match omission {
                    repo::Omission::Disabled => "disabled for queries",
                    repo::Omission::UnknownSource => "configured address has no supplied source",
                    repo::Omission::MissingCache =>
                        "no cached metadata; enable and refresh the intended available source",
                    repo::Omission::WrongSource =>
                        "cached source differs from the configured source; inspect provenance before refreshing",
                }
            ),
        );
    }
}
fn apply_actions(
    next: &mut work::Session,
    definition: &work::Definition,
    practice: bool,
    actions: Vec<work::Event>,
) {
    for event in actions {
        if let Err(e) = next.run(definition, &event, practice) {
            let s = if practice {
                &mut next.practice
            } else {
                &mut next.assignment
            };
            s.message = e;
            s.checked = false;
        } else {
            let s = if practice {
                &mut next.practice
            } else {
                &mut next.assignment
            };
            s.message=match event{work::Event::Query{..}=>"Actual cached-metadata query recorded.".into(),work::Event::Edit{edit:work::Edit::Settings{..}}=>"Applied repository settings. Cached provenance remains separate; run fresh queries for current evidence.".into(),work::Event::Edit{edit:work::Edit::Refresh{..}}=>"Refreshed the selected simulated metadata cache. Installed packages are unchanged.".into()};
        }
    }
}

impl LabApp {
    pub(super) fn repository_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let engine::Goal::Repositories { definition } = &source.goal else {
            return;
        };
        let engine::State::Repositories { simulation } = &a.state else {
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
                    let r = ui.add_enabled(active, egui::Button::new(label).selected(v == view));
                    record(&mut self.metrics.repository, key, &r, ui);
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
                "Builds on RPM records and metadata.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(
                ui,
                "A different supplied repository demonstrates the reasoning before the independent task.",
            );
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.repository, key, &r, ui);
                    if r.clicked() {
                        next.view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        } else {
            if practice {
                lesson_presentation::practice_step(ui, &source, 0);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            if practice {
                let e = &definition.example_actions[next.example_action];
                let description = match e {
                    work::Event::Edit {
                        edit:
                            work::Edit::Settings {
                                alias,
                                enabled,
                                uri,
                            },
                    } => format!(
                        "Worked action · configure {alias}: {}, URI {uri}; then Apply settings.",
                        if *enabled { "Enabled" } else { "Disabled" }
                    ),
                    work::Event::Edit {
                        edit: work::Edit::Refresh { alias },
                    } => format!("Worked action · select {alias}, then Refresh metadata."),
                    work::Event::Query { package } => {
                        format!("Worked action · enter {package}, then Query exact package.")
                    }
                };
                lesson_presentation::prose(ui, &description);
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add_enabled(active, button("Prepare worked action"));
                    record(&mut self.metrics.repository, "fill-example", &r, ui);
                    if r.clicked() {
                        let s = &mut next.practice;
                        match e {
                            work::Event::Edit {
                                edit:
                                    work::Edit::Settings {
                                        alias,
                                        enabled,
                                        uri,
                                    },
                            } => {
                                s.load(&definition.example_source, alias).unwrap();
                                s.enabled = *enabled;
                                s.uri = uri.clone();
                                s.dirty = true;
                                s.checked = false;
                            }
                            work::Event::Edit {
                                edit: work::Edit::Refresh { alias },
                            } => s.load(&definition.example_source, alias).unwrap(),
                            work::Event::Query { package } => {
                                s.package = package.clone();
                                s.checked = false;
                            }
                        }
                    }
                    let r = ui.add_enabled(
                        active && next.example_action + 1 < definition.example_actions.len(),
                        button("Next worked action"),
                    );
                    record(&mut self.metrics.repository, "next-action", &r, ui);
                    if r.clicked() {
                        next.example_action += 1;
                    }
                });
                lesson_presentation::practice_step(ui, &source, 1);
            }
            let mut actions = vec![];
            let src = if practice {
                &definition.example_source
            } else {
                &definition.initial_source
            };
            if ui.available_width() > 920. * self.scale {
                ui.columns(2, |cols| {
                    cols[0].with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        let s = if practice {
                            &mut next.practice
                        } else {
                            &mut next.assignment
                        };
                        editor(
                            ui,
                            s,
                            src,
                            &mut self.metrics.repository,
                            active,
                            &mut actions,
                        );
                        query_input(ui, s, &mut self.metrics.repository, active, &mut actions);
                    });
                    apply_actions(
                        &mut next,
                        definition,
                        practice,
                        std::mem::take(&mut actions),
                    );
                    cols[1].with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        let s = if practice {
                            &mut next.practice
                        } else {
                            &mut next.assignment
                        };
                        inspection(ui, s, src, &mut self.metrics.repository, active);
                        actual(ui, definition, &next, practice);
                    });
                });
            } else {
                let s = if practice {
                    &mut next.practice
                } else {
                    &mut next.assignment
                };
                inspection(ui, s, src, &mut self.metrics.repository, active);
                editor(
                    ui,
                    s,
                    src,
                    &mut self.metrics.repository,
                    active,
                    &mut actions,
                );
                query_input(ui, s, &mut self.metrics.repository, active, &mut actions);
                apply_actions(&mut next, definition, practice, actions);
                actual(ui, definition, &next, practice);
            }
            let s = if practice {
                &mut next.practice
            } else {
                &mut next.assignment
            };
            if !s.message.is_empty() {
                lesson_presentation::prose(ui, &s.message);
            }
            let r = ui.add_enabled(
                active,
                primary(
                    ui,
                    if practice {
                        "Check example"
                    } else {
                        "Check repository"
                    },
                ),
            );
            record(&mut self.metrics.repository, "check", &r, ui);
            self.metrics.check = Some(r.rect);
            if r.clicked() {
                s.checked = true;
            }
            if s.checked {
                let rows = definition.checks(&next, practice);
                let failed = rows.iter().filter(|r| !r.0).count();
                let r = lesson_presentation::rich(
                    ui,
                    RichText::new(if failed == 0 {
                        format!("Passed · all {} outcome checks", rows.len())
                    } else {
                        format!("Needs attention · {failed} of {} checks", rows.len())
                    })
                    .strong(),
                );
                self.metrics.repository.result = Some((rows.len(), failed, r.rect));
                for (ok, text) in rows {
                    lesson_presentation::rich(
                        ui,
                        RichText::new(format!(
                            "{} · {text}",
                            if ok { "Passed" } else { "Needs attention" }
                        ))
                        .color(if ok {
                            palette(ui).success
                        } else {
                            palette(ui).error
                        }),
                    );
                }
            } else {
                lesson_presentation::prose(
                    ui,
                    if practice {
                        "Not checked · use Check example after the required queries."
                    } else {
                        "Not checked · use Check repository after the required queries."
                    },
                );
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                let r = ui.add_enabled(active, button("Reset example only"));
                record(&mut self.metrics.repository, "reset-example", &r, ui);
                if r.clicked() {
                    next.practice = work::Session::new(definition).practice;
                    next.example_action = 0;
                }
            }
            controls::collapsing_header(ui, "Help and supported simulation").show(ui, |ui| {
                lesson_presentation::prose(ui, work::HELP);
                if !practice {
                    let a = self.progress.attempts.get_mut(&source.id).unwrap();
                    let r = ui.add_enabled(
                        active && a.hints < source.hints.len(),
                        button("Show a hint"),
                    );
                    if r.clicked() {
                        a.hints += 1;
                        self.dirty = true;
                    }
                    for hint in source.hints.iter().take(a.hints) {
                        lesson_presentation::prose(ui, hint);
                    }
                    controls::collapsing_header(ui, "Worked approach · reveals a solution").show(
                        ui,
                        |ui| {
                            lesson_presentation::prose(ui, &source.explanation);
                        },
                    );
                }
                for link in &source.sources {
                    ui.hyperlink_to(link, link);
                }
            });
        }
        if next != before && active {
            match definition.bound(&next) {
                Ok(()) => {
                    let a = self.progress.attempts.get_mut(&source.id).unwrap();
                    a.state = engine::State::Repositories {
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
