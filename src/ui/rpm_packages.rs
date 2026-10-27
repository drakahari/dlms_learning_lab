//! Literal RPM terminal over declared virtual data; no host bridge or saved grade cache.
use super::*;
use dlms_learning_lab::{
    rpm_packages as core, rpm_packages_teaching as learn,
    rpm_packages_terminal::{self as terminal, Outcome, Session},
    rpm_packages_workspace as work,
};
pub(super) mod memo;
/// Count exact compact JSON without owning a serialized copy. Action/edit preflight only.
struct CompactSize {
    bytes: usize,
    limit: usize,
}
impl std::io::Write for CompactSize {
    fn write(&mut self, input: &[u8]) -> std::io::Result<usize> {
        let next = self
            .bytes
            .checked_add(input.len())
            .filter(|n| *n <= self.limit)
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "RPM compact JSON limit")
            })?;
        self.bytes = next;
        Ok(input.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn within<T: serde::Serialize + ?Sized>(value: &T, limit: usize) -> bool {
    serde_json::to_writer(CompactSize { bytes: 0, limit }, value).is_ok()
}
#[derive(Default)]
pub(super) struct RpmMetrics {
    pub command: Option<(egui::Rect, egui::Id, bool)>,
    pub run: Option<(egui::Rect, bool)>,
    pub check: Option<egui::Rect>,
    pub check_enabled: bool,
    pub output: Option<egui::Rect>,
    pub result: Option<egui::Rect>,
    pub result_rows: Vec<(usize, bool, egui::Rect, egui::Rect)>,
    pub output_rows: Vec<(String, egui::Rect, egui::Rect)>,
    pub inspect_selected: Option<(String, Option<String>)>,
    pub inspector: Option<egui::Rect>,
    pub inspect_header: Option<egui::Rect>,
    pub inspect_available: Option<egui::Rect>,
    pub inspect_path: Option<egui::Rect>,
    pub inspect_result: Option<egui::Rect>,
    pub inspect_choices: Vec<(String, egui::Rect)>,
    pub inspect_popup: Option<egui::Rect>,
    pub example: Option<egui::Rect>,
    pub insert: Option<egui::Rect>,
    pub choices: Vec<(usize, egui::Rect)>,
    pub popup: Option<egui::Rect>,
    pub tabs: Vec<(teaching::View, egui::Rect)>,
    pub example_reset: Option<egui::Rect>,
    pub next: Option<egui::Rect>,
    pub notice: Option<egui::Rect>,
    pub inspection_text: Vec<String>,
}
#[derive(Default)]
pub(super) struct Presentation {
    pub memo: memo::Memo,
    epoch: u64,
    notice: Option<String>,
    inspect: Inspector,
}
impl Presentation {
    pub fn clear(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.notice = None;
        self.inspect = Inspector::default();
        self.memo.retain(|_| false);
    }
}
#[derive(Default)]
struct Inspector {
    scope: Option<egui::Id>,
    archive: Option<String>,
    local: bool,
    path: Option<String>,
}
struct Edit {
    previous: Option<String>,
    rejected: bool,
}
struct InputBuffer<'a> {
    value: &'a mut String,
    before: Option<String>,
    rejected: bool,
}
impl InputBuffer<'_> {
    fn remember(&mut self) {
        if self.before.is_none() {
            self.before = Some(self.value.clone());
        }
    }
    fn finish(mut self) -> Edit {
        if self.rejected {
            if let Some(old) = self.before.take() {
                *self.value = old;
            }
            Edit {
                previous: None,
                rejected: true,
            }
        } else {
            let previous = self
                .before
                .filter(|old| old.as_str() != self.value.as_str());
            Edit {
                previous,
                rejected: false,
            }
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
    fn insert_text(&mut self, text: &str, index: usize) -> usize {
        if self.rejected || text.is_empty() {
            return 0;
        }
        if text.contains(['\0', '\n', '\r'])
            || self
                .value
                .len()
                .checked_add(text.len())
                .is_none_or(|n| n > terminal::MAX_COMMAND)
        {
            self.rejected = true;
            return 0;
        }
        self.remember();
        egui::TextBuffer::insert_text(self.value, text, index)
    }
    fn delete_char_range(&mut self, range: std::ops::Range<usize>) {
        if self.rejected || range.is_empty() {
            return;
        }
        self.remember();
        egui::TextBuffer::delete_char_range(self.value, range);
    }
}
fn display(text: &str) -> std::borrow::Cow<'_, str> {
    if !text
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
    {
        return std::borrow::Cow::Borrowed(text);
    }
    let mut out = String::new();
    for c in text.chars() {
        if c.is_control() && !matches!(c, '\n' | '\t') {
            out.extend(c.escape_default());
        } else {
            out.push(c)
        }
    }
    std::borrow::Cow::Owned(out)
}
fn console(
    ui: &mut egui::Ui,
    scope: egui::Id,
    t: &mut Session,
    metrics: &mut Metrics,
    focus: &mut bool,
) -> (Edit, bool) {
    ui.strong("Restricted RPM terminal · Virtual data");
    let height = (metrics.main_available_height * 0.30)
        .max(controls::height(ui) * 2.)
        .min(controls::height(ui) * 7.);
    // An inner ScrollArea in the outer auto-sized workspace can otherwise
    // receive zero height despite retained output. Reserve readable rows from
    // shared native control metrics, without exceeding the existing maximum.
    let minimum_output = (controls::height(ui) * 3.).min(height);
    let output = egui::ScrollArea::both().id_salt(scope.with("output")).max_height(height).min_scrolled_height(minimum_output).auto_shrink([false,false]).stick_to_bottom(true).show(ui, |ui| {
        if t.entries.is_empty(){ ui.label("Read installed/local metadata with the disclosed rpm queries. Nothing runs on the host."); }
        for (index,e) in t.entries.iter().enumerate() {
            let r=ui.add(egui::Label::new(RichText::new(format!("$ {}",display(&e.command))).monospace()).extend().selectable(true));
            metrics.rpm.output_rows.push((format!("{index}:command"),r.rect,ui.clip_rect()));
            match &e.output {
                Outcome::Utility{stdout,stderr,status,summary} => {
                    if !stdout.is_empty() {
                        let r=ui.add(egui::Label::new(RichText::new(display(stdout).as_ref()).monospace()).extend().selectable(true));
                        metrics.rpm.output_rows.push((format!("{index}:stdout"),r.rect,ui.clip_rect()));
                    }
                    if !stderr.is_empty() {
                        let r=ui.label(RichText::new(display(stderr).as_ref()).color(palette(ui).error));
                        metrics.rpm.output_rows.push((format!("{index}:stderr"),r.rect,ui.clip_rect()));
                    }
                    let r=ui.label(RichText::new(format!("RPM command success · status {status}")).small());
                    metrics.rpm.output_rows.push((format!("{index}:status"),r.rect,ui.clip_rect()));
                    if let Some(text)=summary {
                        let r=ui.add(egui::Label::new(RichText::new(format!("Simulation operation summary: {text}")).small()).wrap());
                        metrics.rpm.output_rows.push((format!("{index}:summary"),r.rect,ui.clip_rect()));
                    }
                }
                Outcome::SimulatorUnsupported{reason} => {
                    let r=ui.add(egui::Label::new(RichText::new(format!("Command not run by this simulation: {reason}")).color(palette(ui).error)).wrap());
                    metrics.rpm.output_rows.push((format!("{index}:unsupported"),r.rect,ui.clip_rect()));
                }
            }
        }
    });
    metrics.rpm.output = Some(output.inner_rect);
    metrics.output = Some(output.inner_rect);
    let id = scope.with("command");
    let focused = ui.memory(|m| m.has_focus(id));
    let enter = ui.is_enabled()
        && focused
        && ui.input_mut(|i| {
            !i.modifiers.command
                && !i.modifiers.ctrl
                && !i.modifiers.mac_cmd
                && i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
        });
    let mut buffer = InputBuffer {
        value: &mut t.command,
        before: None,
        rejected: false,
    };
    let mut run = false;
    ui.horizontal_wrapped(|ui| {
        let width = ui.available_width();
        let button_width = controls::button_width(ui, "Run command");
        let field_width = if width
            >= button_width + controls::GAP + ui.text_style_height(&egui::TextStyle::Body) * 8.
        {
            width - button_width - controls::GAP
        } else {
            width
        };
        let r = ui.add(
            controls::singleline(ui, &mut buffer)
                .id(id)
                .desired_width((field_width - 2. * f32::from(controls::PAD_X)).max(0.))
                .hint_text("rpm … · Enter to run"),
        );
        name_control(&r, "RPM command for the current example or assignment");
        metrics.rpm.command = Some((r.rect, id, r.enabled()));
        metrics.terminal = Some(r.rect);
        metrics.inputs.push(r.rect);
        if *focus && ui.is_enabled() && !ui.input(|i| i.pointer.any_down()) {
            r.request_focus();
            *focus = false;
        }
        let button = ui.add_enabled(!buffer.value.trim().is_empty(), button("Run command"));
        metrics.rpm.run = Some((button.rect, button.enabled()));
        run = ui.is_enabled() && !buffer.value.trim().is_empty() && (button.clicked() || enter);
        if run {
            r.request_focus();
        }
    });
    let edit = buffer.finish();
    (edit, run)
}
fn relation(r: core::Relation) -> &'static str {
    match r {
        core::Relation::Lt => "<",
        core::Relation::Le => "<=",
        core::Relation::Eq => "=",
        core::Relation::Ge => ">=",
        core::Relation::Gt => ">",
    }
}
fn simulated_inspector(
    ui: &mut egui::Ui,
    scope: egui::Id,
    state: &core::State,
    definition: &work::Definition,
    selection: &mut Inspector,
    metrics: &mut Metrics,
) {
    if selection.scope != Some(scope) {
        *selection = Inspector {
            scope: Some(scope),
            ..Default::default()
        };
    }
    let panel = egui::CollapsingHeader::new("Inspect package data · Read-only")
        .id_salt(scope.with("inspect"))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(&mut selection.local, false, "Installed records");
                let available =
                    ui.selectable_value(&mut selection.local, true, "Available local archives");
                metrics.rpm.inspect_available = Some(available.rect);
            });
            controls::form_row(ui, "Package header", |ui, width| {
                let response = controls::dropdown(ui, scope.with("header"), width)
                    .selected_text(
                        selection
                            .archive
                            .as_ref()
                            .and_then(|id| {
                                if selection.local {
                                    definition.archive_paths.iter().find_map(|(path, archive)| {
                                        (archive == id && state.runtime().available.contains(id))
                                            .then_some(path.as_str())
                                    })
                                } else {
                                    state
                                        .runtime()
                                        .installed
                                        .iter()
                                        .find(|i| &i.archive == id)
                                        .map(|i| i.identity.name.as_str())
                                }
                            })
                            .unwrap_or("Choose declared metadata"),
                    )
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        if selection.local {
                            for (path, id) in definition
                                .archive_paths
                                .iter()
                                .filter(|(_, id)| state.runtime().available.contains(*id))
                                .take(12)
                            {
                                let r = ui.selectable_value(
                                    &mut selection.archive,
                                    Some(id.clone()),
                                    path,
                                );
                                metrics.rpm.inspect_choices.push((path.clone(), r.rect));
                            }
                        } else {
                            for i in state.runtime().installed.iter().take(8) {
                                let r = ui.selectable_value(
                                    &mut selection.archive,
                                    Some(i.archive.clone()),
                                    &i.identity.name,
                                );
                                metrics
                                    .rpm
                                    .inspect_choices
                                    .push((i.archive.clone(), r.rect));
                            }
                        }
                        metrics.rpm.inspect_popup = Some(controls::popup_content_clip(ui));
                    });
                metrics.rpm.inspect_header = Some(response.response.rect);
            });
            let Some(id) = selection.archive.as_ref() else {
                return;
            };
            let eligible = if selection.local {
                state.runtime().available.contains(id)
            } else {
                state.runtime().installed.iter().any(|i| &i.archive == id)
            };
            let Some(a) = state.catalog().archives.get(id).filter(|_| eligible) else {
                ui.label("This declared header is not present in the selected source.");
                return;
            };
            ui.label("Declared simulation header · No binary archive or signature verification");
            for (label, value) in [("Name", a.identity.name.as_str()), ("EVR", a.evr.as_str())] {
                controls::form_row(ui, label, |ui, _| {
                    ui.label(value);
                });
            }
            ui.label(format!(
                "Architecture: {}",
                match a.identity.arch {
                    core::Architecture::Noarch => "noarch",
                    core::Architecture::X86_64 => "x86_64",
                }
            ));
            ui.label(format!(
                "Numeric epoch: {}",
                a.evr
                    .as_str()
                    .split_once(':')
                    .map_or(0, |(p, _)| p.parse::<u32>().unwrap_or(0))
            ));
            ui.label("Provides (declared, plus own package name)");
            ui.label(format!("{} = {}", a.identity.name, a.evr.as_str()));
            for (cap, version) in &a.provides {
                ui.label(format!(
                    "{cap}{}",
                    version
                        .as_ref()
                        .map_or_else(String::new, |v| format!(" = {}", v.as_str()))
                ));
            }
            ui.label("Requires (all must be satisfied)");
            if a.requires.is_empty() {
                ui.label("No declared requirements");
            }
            for r in &a.requires {
                ui.label(format!(
                    "{}{}",
                    r.capability,
                    r.constraint.as_ref().map_or_else(String::new, |c| format!(
                        " {} {}",
                        relation(c.relation),
                        c.evr.as_str()
                    ))
                ));
            }
            controls::form_row(ui, "Payload path", |ui, width| {
                let response = controls::dropdown(ui, scope.with("payload"), width)
                    .selected_text(
                        selection
                            .path
                            .as_deref()
                            .unwrap_or("Choose a recorded path"),
                    )
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        for p in a.files.keys() {
                            let r = ui.selectable_value(&mut selection.path, Some(p.clone()), p);
                            metrics.rpm.inspect_choices.push((p.clone(), r.rect));
                        }
                        metrics.rpm.inspect_popup = Some(controls::popup_content_clip(ui));
                    });
                metrics.rpm.inspect_path = Some(response.response.rect);
            });
            metrics.rpm.inspect_selected = Some((id.clone(), selection.path.clone()));
            if let Some(path) = selection.path.as_ref()
                && let Some(expected) = a.files.get(path)
            {
                ui.strong("Simulated payload comparison · This is not rpm -V");
                ui.label(format!("Declared path: {path}"));
                ui.label(format!(
                    "Configuration file: {}",
                    expected.kind != core::FileKind::Ordinary
                ));
                let current = state.runtime().files.get(path);
                let owned = state
                    .runtime()
                    .installed
                    .iter()
                    .filter_map(|i| state.catalog().archives.get(&i.archive))
                    .any(|a| a.files.contains_key(path));
                let status = match current {
                    None => "Current virtual file is missing",
                    Some(f) if f == &expected.file => {
                        "Current virtual bytes and attributes match this declared payload"
                    }
                    Some(_) => {
                        "Current virtual bytes or attributes differ from this declared payload"
                    }
                };
                metrics.rpm.inspect_result = Some(ui.label(status).rect);
                metrics.rpm.inspection_text.push(status.into());
                ui.label(if owned {
                    "A current installed record claims this path"
                } else {
                    "No current installed record claims this path"
                });
                for (label, value) in [("Declared", Some(&expected.file)), ("Current", current)] {
                    ui.strong(label);
                    if let Some(f) = value {
                        ui.label(format!(
                            "Mode {:o} · owner {} · group {} · mtime {}",
                            f.mode, f.owner, f.group, f.mtime
                        ));
                        ui.add(
                            egui::Label::new(
                                RichText::new(display(&f.contents).as_ref()).monospace(),
                            )
                            .wrap()
                            .selectable(true),
                        );
                    } else {
                        ui.label("Missing");
                    }
                }
            }
        });
    metrics.rpm.inspector = Some(panel.header_response.rect);
}

fn session(a: &Attempt, practice: bool) -> Option<&Session> {
    let w = a.workspace.as_ref()?;
    if practice {
        Some(&w.rpm_learning.as_ref()?.terminal)
    } else {
        w.rpm_session.as_ref()
    }
}
fn session_mut(a: &mut Attempt, practice: bool) -> Option<&mut Session> {
    let w = a.workspace.as_mut()?;
    if practice {
        Some(&mut w.rpm_learning.as_mut()?.terminal)
    } else {
        w.rpm_session.as_mut()
    }
}
fn simulation(a: &Attempt, practice: bool) -> Option<&core::State> {
    if practice {
        Some(&a.workspace.as_ref()?.rpm_learning.as_ref()?.simulation)
    } else {
        let engine::State::RpmPackages { simulation } = &a.state else {
            return None;
        };
        Some(simulation)
    }
}
fn command_fit(a: &Attempt, practice: bool) -> bool {
    session(a, practice).is_some_and(Session::valid)
        && within(a, work::MAX_ATTEMPT)
        && (!practice
            || a.workspace
                .as_ref()
                .and_then(|w| w.rpm_learning.as_ref())
                .is_some_and(|l| within(l, learn::MAX_LEARNING)))
}
fn admit_action(a: &Attempt, s: &Scenario) -> bool {
    let (engine::State::RpmPackages { simulation }, engine::Goal::RpmPackages { definition }) =
        (&a.state, &s.goal)
    else {
        return false;
    };
    if !matches!(&s.initial,engine::State::RpmPackages{simulation:authored}if authored==&definition.initial)
        || definition.validate().is_err()
        || !definition.same_assignment(simulation)
        || !work::admitted(simulation)
        || !within(a, work::MAX_ATTEMPT)
    {
        return false;
    }
    a.workspace.as_ref().is_some_and(|w| {
        !memo::foreign(w)
            && w.valid()
            && w.rpm_session
                .as_ref()
                .is_none_or(|t| t.valid_for(simulation))
            && w.rpm_learning.as_ref().is_none_or(|l| {
                s.rpm_packages_teaching.as_ref().is_some_and(|source| {
                    source.separate_from_definition(definition) && l.valid_for(source)
                })
            })
    })
}
fn run(a: &mut Attempt, s: &Scenario, practice: bool) -> Result<(), String> {
    if !admit_action(a, s) {
        return Err("RPM context or saved bounds differ; command not run".into());
    }
    let old_state = simulation(a, practice).ok_or("RPM model missing")?.clone();
    let old_session = session(a, practice).ok_or("RPM session missing")?.clone();
    let outcome = if practice {
        a.workspace
            .as_mut()
            .unwrap()
            .rpm_learning
            .as_mut()
            .unwrap()
            .run(s.rpm_packages_teaching.as_ref().unwrap())
    } else {
        let engine::Goal::RpmPackages { definition } = &s.goal else {
            unreachable!()
        };
        let engine::State::RpmPackages { simulation } = &mut a.state else {
            unreachable!()
        };
        a.workspace
            .as_mut()
            .unwrap()
            .rpm_session
            .as_mut()
            .unwrap()
            .run(definition, simulation)
    };
    if !command_fit(a, practice) {
        if practice {
            let l = a.workspace.as_mut().unwrap().rpm_learning.as_mut().unwrap();
            l.simulation = old_state;
            l.terminal = old_session;
        } else {
            let engine::State::RpmPackages { simulation } = &mut a.state else {
                unreachable!()
            };
            *simulation = old_state;
            a.workspace.as_mut().unwrap().rpm_session = Some(old_session);
        }
        return Err(
            "Run exceeds the complete saved capacity; previous command history and model retained"
                .into(),
        );
    }
    outcome.map(|_| ())
}
pub(super) fn solved_with_memos(
    rpm: &memo::Memo,
    nat: &nat_workspace::memo::Memo,
    a: &Attempt,
    s: &Scenario,
) -> bool {
    if matches!(s.initial, engine::State::RpmPackages { .. }) {
        rpm.prepared(&s.id)
            .unwrap_or_else(|| progress::current_solved(a, s))
    } else {
        nat_workspace::solved_with_memo(nat, a, s)
    }
}
impl LabApp {
    pub(super) fn rpm_visible_solved(&self, a: &Attempt, s: &Scenario) -> bool {
        if matches!(s.initial, engine::State::RpmPackages { .. }) {
            self.rpm_presentation
                .memo
                .prepared(&s.id)
                .unwrap_or_else(|| progress::current_solved(a, s))
        } else {
            self.visible_solved(a, s)
        }
    }
    pub(super) fn rpm_prepass(&mut self) {
        let sources = [
            self.scenarios.as_slice(),
            if self.developer && self.synthetic {
                self.synthetic_catalog.as_slice()
            } else {
                &[]
            },
        ];
        self.rpm_presentation.memo.retain(|id| {
            self.progress.attempts.contains_key(id)
                && sources.iter().any(|source| {
                    source.iter().any(|s| {
                        s.id == id && matches!(s.initial, engine::State::RpmPackages { .. })
                    })
                })
        });
        let selected = self
            .selected
            .and_then(|i| self.scenarios.get(i))
            .map(|s| s.id.as_str());
        for selected_pass in [false, true] {
            for source in sources {
                for s in source {
                    if (selected == Some(s.id.as_str())) == selected_pass
                        && matches!(s.initial, engine::State::RpmPackages { .. })
                        && let Some(a) = self.progress.attempts.get(&s.id)
                    {
                        self.rpm_presentation.memo.evaluate(a, s, false);
                    }
                }
            }
        }
    }
    pub(super) fn prepare_rpm(a: &mut Attempt, s: &Scenario, new: bool) {
        if let (Some(l), engine::Goal::RpmPackages { definition }) =
            (&s.rpm_packages_teaching, &s.goal)
        {
            learn::prepare(a, definition, l, new).expect("validated separate RPM lesson");
        }
    }
    pub(super) fn rpm_page(&mut self, ui: &mut egui::Ui, i: usize) {
        self.workspace_header(ui, i);
        let active = !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let source = &self.scenarios[i];
        let attempt = self.progress.attempts.get_mut(&source.id).unwrap();
        let mut effect = PageEffect {
            focus: self.focus_input,
            active,
            ..Default::default()
        };
        card(ui).show(ui, |ui| {
            page_body(
                ui,
                source,
                attempt,
                &mut self.rpm_presentation,
                &mut self.metrics,
                &mut effect,
            )
        });
        if effect.dirty && !effect.refreshed {
            self.rpm_presentation.memo.evaluate(attempt, source, false);
        }
        self.dirty |= effect.dirty;
        self.focus_input = effect.focus;
        self.checked = attempt.workspace.as_ref().is_some_and(|w| w.checked);
        if self.metrics.completion {
            let source = &self.scenarios[i];
            let next = self
                .units
                .iter()
                .find(|u| u.id == source.unit)
                .and_then(|u| {
                    let at = u.labs.iter().position(|id| id == &source.id)?;
                    u.labs.iter().skip(at + 1).find_map(|id| {
                        self.scenarios.iter().position(|s| {
                            &s.id == id
                                && self
                                    .progress
                                    .attempts
                                    .get(id)
                                    .is_none_or(|a| !self.rpm_visible_solved(a, s))
                        })
                    })
                });
            if let Some(next) = next {
                let response = ui.add_enabled(active, button("Next in unit"));
                self.metrics.rpm.next = Some(response.rect);
                if response.clicked() {
                    self.launch(next);
                }
            }
        }
    }
}
#[derive(Default)]
struct PageEffect {
    dirty: bool,
    focus: bool,
    active: bool,
    notice: Option<String>,
    inserted: bool,
    refreshed: bool,
}
fn set_view(a: &mut Attempt, view: teaching::View) -> bool {
    let Some(l) = a.workspace.as_mut().and_then(|w| w.rpm_learning.as_mut()) else {
        return false;
    };
    if l.view == view {
        return false;
    }
    let old = l.view;
    l.view = view;
    if within(a, work::MAX_ATTEMPT) {
        true
    } else {
        a.workspace
            .as_mut()
            .unwrap()
            .rpm_learning
            .as_mut()
            .unwrap()
            .view = old;
        false
    }
}
fn page_body(
    ui: &mut egui::Ui,
    s: &Scenario,
    a: &mut Attempt,
    p: &mut Presentation,
    m: &mut Metrics,
    e: &mut PageEffect,
) {
    let active = e.active;
    m.package_heading = Some(ui.heading(&s.title).rect);
    ui.label(RichText::new(&s.objective).strong());
    ui.label(
        RichText::new(
            "Script-free RPM simulation · Commands operate only on declared virtual data",
        )
        .small()
        .color(palette(ui).muted),
    );
    let old_view = if a.challenge {
        teaching::View::Assignment
    } else {
        a.workspace
            .as_ref()
            .and_then(|w| w.rpm_learning.as_ref())
            .map_or(teaching::View::Assignment, |l| l.view)
    };
    let mut view = old_view;
    if s.rpm_packages_teaching.is_some() && !a.challenge {
        ui.add_enabled_ui(active, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · Ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let response = ui.selectable_value(&mut view, v, label);
                    m.rpm.tabs.push((v, response.rect));
                }
            })
        });
    }
    if view != old_view {
        if set_view(a, view) {
            e.dirty = true;
            e.focus = true;
            p.notice = None;
        } else {
            view = old_view;
            p.notice = Some("View change exceeds saved capacity; previous view retained".into());
        }
    }
    if view == teaching::View::Introduction {
        if let Some(l) = &s.rpm_packages_teaching {
            ui.label(&l.prerequisite_note);
            for c in &l.concepts {
                ui.strong(&c.title);
                ui.label(&c.text);
            }
            ui.strong("A separate example to investigate");
            ui.label(&l.example_description);
            ui.add_enabled_ui(active, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (label, target) in [
                        ("Try this example", teaching::View::Example),
                        ("Go straight to assignment", teaching::View::Assignment),
                    ] {
                        let response = ui.add(button(label));
                        if target == teaching::View::Example {
                            m.example_action = Some(response.rect);
                        } else {
                            m.assignment_action = Some(response.rect);
                        }
                        if response.clicked() && set_view(a, target) {
                            e.dirty = true;
                            e.focus = true;
                        }
                    }
                })
            });
        }
        return;
    }
    let practice = view == teaching::View::Example;
    let check_key =
        active && ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter));
    let scope = egui::Id::new(("rpm-workspace", &s.id, practice, p.epoch));
    if practice && let Some(source) = &s.rpm_packages_teaching {
        if a.workspace
            .as_ref()
            .and_then(|w| w.rpm_learning.as_ref())
            .is_none_or(|l| l.selected_example >= source.example_commands.len())
        {
            ui.colored_label(palette(ui).error,"The separate example selection differs from its authored source; previous work retained");
            return;
        }
        ui.label("Practice example · Ungraded; separate packages and command history");
        ui.label(&source.example_description);
        example_picker(ui, scope, a, source, m, e);
        if e.inserted {
            p.notice = None;
        }
        if let Some(notice) = e.notice.take() {
            p.notice = Some(notice);
        }
        let selected = a
            .workspace
            .as_ref()
            .unwrap()
            .rpm_learning
            .as_ref()
            .unwrap()
            .selected_example;
        ui.label(&source.example_commands[selected].explanation);
        ui.label(
            RichText::new(
                "Insert fills the example command only. Read it, then press Run command.",
            )
            .small(),
        );
        egui::CollapsingHeader::new("Try a change, then repair")
            .id_salt(scope.with("experiment"))
            .show(ui, |ui| {
                ui.label(&source.try_change.description);
                for c in &source.try_change.commands {
                    ui.monospace(&c.command);
                    ui.label(&c.explanation);
                }
                ui.strong("Repair the example");
                for c in &source.repair {
                    ui.monospace(&c.command);
                    ui.label(&c.explanation);
                }
            });
    }
    let Some(t) = session_mut(a, practice) else {
        ui.label("RPM session unavailable; return to the library and reopen the activity.");
        return;
    };
    let (edit, submitted) = ui
        .add_enabled_ui(active, |ui| console(ui, scope, t, m, &mut e.focus))
        .inner;
    let mut edit_rejected = edit.rejected;
    if edit_rejected {
        p.notice=Some("Command paste exceeds2048 bytes or contains a line break/NUL; complete prior draft retained".into());
    } else if let Some(old) = edit.previous {
        if command_fit(a, practice) {
            e.dirty = true;
            p.notice = None;
        } else {
            session_mut(a, practice).unwrap().command = old;
            edit_rejected = true;
            p.notice =
                Some("Command exceeds saved aggregate capacity; previous draft retained".into());
        }
    }
    if submitted && !edit_rejected {
        match run(a, s, practice) {
            Ok(()) => p.notice = None,
            Err(reason) => p.notice = Some(work::notice(reason)),
        }
        e.dirty = true;
    }
    let definition = if practice {
        s.rpm_packages_teaching
            .as_ref()
            .map(|source| &source.definition)
    } else if let engine::Goal::RpmPackages { definition } = &s.goal {
        Some(definition.as_ref())
    } else {
        None
    };
    if let (Some(state), Some(definition)) = (simulation(a, practice), definition) {
        ui.add_enabled_ui(active, |ui| {
            simulated_inspector(ui, scope, state, definition, &mut p.inspect, m)
        });
    }
    let response = ui.add_enabled(
        active,
        primary(
            ui,
            if practice {
                "Check example"
            } else {
                "Check work"
            },
        ),
    );
    m.rpm.check = Some(response.rect);
    m.rpm.check_enabled = response.enabled();
    m.check = Some(response.rect);
    let checked = active && (response.clicked() || check_key);
    if checked {
        if practice {
            a.workspace
                .as_mut()
                .unwrap()
                .rpm_learning
                .as_mut()
                .unwrap()
                .checked = true;
        } else {
            a.workspace.as_mut().unwrap().checked = true;
        }
        e.dirty = true;
    }
    if e.dirty || checked || p.memo.rows(&s.id, practice).is_none() {
        p.memo.evaluate(a, s, checked);
        e.refreshed = true;
    }
    let show = if practice {
        a.workspace
            .as_ref()
            .unwrap()
            .rpm_learning
            .as_ref()
            .unwrap()
            .checked
    } else {
        a.workspace.as_ref().unwrap().checked
    };
    let feedback = controls::feedback(ui, checked, |ui| {
        if show && let Some(rows) = p.memo.rows(&s.id, practice) {
            let passed = rows.iter().all(|r| r.0);
            let title = if passed {
                if practice {
                    "Example checks pass · No assignment credit"
                } else {
                    "Objective complete"
                }
            } else {
                "Check results · Needs attention"
            };
            let mut result = ui.strong(title).rect;
            result = result.union(
                ui.label(format!(
                    "{} of {} checks pass",
                    rows.iter().filter(|r| r.0).count(),
                    rows.len()
                ))
                .rect,
            );
            let mut first_failure = true;
            for (index, (ok, text)) in rows.iter().enumerate() {
                let row = ui.colored_label(
                    if *ok {
                        palette(ui).success
                    } else {
                        palette(ui).error
                    },
                    text,
                );
                m.rpm
                    .result_rows
                    .push((index, *ok, row.rect, ui.clip_rect()));
                if !ok && first_failure {
                    result = result.union(row.rect);
                    first_failure = false;
                }
            }
            m.rpm.result = Some(result);
            if !practice {
                m.completion = passed;
            }
        }
    });
    m.feedback = Some(feedback.response.rect);
    if let Some(notice) = &p.notice {
        m.rpm.notice = Some(ui.colored_label(palette(ui).error, notice).rect);
    }
    if m.completion && !a.challenge {
        egui::CollapsingHeader::new("Why this configuration works")
            .id_salt(scope.with("why"))
            .show(ui, |ui| {
                ui.label(&s.explanation);
            });
    }
    if !a.challenge && !practice {
        egui::CollapsingHeader::new("Optional guidance")
            .id_salt(scope.with("guidance"))
            .show(ui, |ui| {
                for hint in &s.hints {
                    ui.label(hint);
                }
                for step in &s.guide {
                    ui.strong(&step.title);
                    ui.label(&step.why);
                    ui.label(&step.action);
                    for c in &step.commands {
                        ui.monospace(&c.command);
                        ui.label(&c.explanation);
                    }
                }
            });
    }
    if practice {
        let response = ui.add_enabled(active, button("Reset example only"));
        m.rpm.example_reset = Some(response.rect);
        if response.clicked()
            && let Some(source) = &s.rpm_packages_teaching
        {
            match learn::Learning::new(source) {
                Ok(mut fresh) => {
                    fresh.view = teaching::View::Example;
                    let old = a.workspace.as_mut().unwrap().rpm_learning.replace(fresh);
                    if within(a, work::MAX_ATTEMPT) {
                        e.dirty = true;
                        e.refreshed = false;
                        p.epoch = p.epoch.wrapping_add(1);
                        p.inspect = Inspector::default();
                        p.notice = None;
                        e.focus = true;
                    } else {
                        a.workspace.as_mut().unwrap().rpm_learning = old;
                        p.notice = Some(
                            "Example reset exceeds saved capacity; previous example retained"
                                .into(),
                        );
                    }
                }
                Err(reason) => p.notice = Some(reason),
            }
        }
    }
}
fn example_picker(
    ui: &mut egui::Ui,
    scope: egui::Id,
    a: &mut Attempt,
    source: &learn::Lesson,
    m: &mut Metrics,
    e: &mut PageEffect,
) {
    let active = e.active;
    let old = a
        .workspace
        .as_ref()
        .unwrap()
        .rpm_learning
        .as_ref()
        .unwrap()
        .selected_example;
    let mut selected = old;
    let mut insert = false;
    ui.add_enabled_ui(active, |ui| {
        ui.horizontal_wrapped(|ui| {
            let width = (ui.available_width()
                - controls::button_width(ui, "Insert example")
                - controls::GAP)
                .max(0.);
            controls::cell(ui, width, |ui| {
                let response = controls::dropdown(ui, scope.with("example"), width)
                    .selected_text(format!(
                        "Example command {} of {}",
                        selected + 1,
                        source.example_commands.len()
                    ))
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        for (n, c) in source.example_commands.iter().enumerate() {
                            let r = ui.selectable_value(&mut selected, n, &c.command);
                            m.rpm.choices.push((n, r.rect));
                        }
                        m.rpm.popup = Some(controls::popup_content_clip(ui));
                    });
                m.rpm.example = Some(response.response.rect);
            });
            let response = ui.add(button("Insert example"));
            m.rpm.insert = Some(response.rect);
            insert = response.clicked();
        })
    });
    if selected != old {
        a.workspace
            .as_mut()
            .unwrap()
            .rpm_learning
            .as_mut()
            .unwrap()
            .selected_example = selected;
        if within(a, work::MAX_ATTEMPT) {
            e.dirty = true;
        } else {
            a.workspace
                .as_mut()
                .unwrap()
                .rpm_learning
                .as_mut()
                .unwrap()
                .selected_example = old;
            selected = old;
        }
    }
    if insert {
        let old = session(a, true).unwrap().command.clone();
        let result = a
            .workspace
            .as_mut()
            .unwrap()
            .rpm_learning
            .as_mut()
            .unwrap()
            .insert(source, &source.example_commands[selected].command);
        match result {
            Ok(()) if command_fit(a, true) => {
                e.dirty = true;
                e.focus = true;
                e.inserted = true;
            }
            Ok(()) => {
                session_mut(a, true).unwrap().command = old;
                e.notice = Some(
                    "Example insertion exceeds saved aggregate capacity; prior draft retained"
                        .into(),
                );
            }
            Err(reason) => e.notice = Some(work::notice(reason)),
        }
    }
}
