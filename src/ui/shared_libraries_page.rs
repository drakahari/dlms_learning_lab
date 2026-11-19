//! Native shared-library editor and investigation workspace.
//! No renderer/model injection, host utility, subprocess or filesystem access.
use super::*;
use dlms_learning_lab::{engine::State, shared_libraries_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub goal: Option<egui::Rect>,
    pub goal_anchor: Option<egui::Rect>,
    pub intro_goal: Option<egui::Rect>,
    pub intro_context: Option<egui::Rect>,
    pub inventory: Option<egui::Rect>,
    pub history: Option<egui::Rect>,
    pub inspection: Option<egui::Rect>,
    pub result: Option<(usize, usize, egui::Rect)>,
}
fn record(m: &mut Metrics, key: impl Into<String>, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn world(session: &work::Session, scope: work::Scope) -> &work::Workspace {
    match scope {
        work::Scope::Assignment => &session.assignment,
        work::Scope::Example => &session.example,
    }
}
fn scope_name(scope: work::Scope) -> &'static str {
    match scope {
        work::Scope::Assignment => "assignment",
        work::Scope::Example => "example",
    }
}
#[derive(Clone)]
enum Edit {
    View(work::View),
    SelectStep(usize),
    Configuration(String),
    SaveConfiguration,
    Launch(String, String),
    SaveLaunch(String),
    InspectLaunch(String),
    Command(String),
    Run(String),
    Check,
    ResetExample,
}
struct Buffer<'a> {
    value: &'a mut String,
    cap: usize,
    multiline: bool,
    rejected: bool,
    received: bool,
}
impl egui::TextBuffer for Buffer<'_> {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Buffer<'static>>()
    }
    fn is_mutable(&self) -> bool {
        true
    }
    fn as_str(&self) -> &str {
        self.value
    }
    fn insert_text(&mut self, text: &str, at: usize) -> usize {
        self.received = true;
        if self.rejected
            || text.len() > self.cap
            || self
                .value
                .len()
                .checked_add(text.len())
                .is_none_or(|n| n > self.cap)
            || text.contains('\0')
            || (!self.multiline && text.contains(['\n', '\r']))
        {
            self.rejected = true;
            return 0;
        }
        egui::TextBuffer::insert_text(self.value, text, at)
    }
    fn delete_char_range(&mut self, range: std::ops::Range<usize>) {
        if !self.rejected {
            egui::TextBuffer::delete_char_range(self.value, range);
        }
    }
}
pub(super) struct Edited {
    pub(super) response: egui::Response,
    pub(super) clip: egui::Rect,
    pub(super) viewport: egui::Rect,
    pub(super) value: Option<String>,
    pub(super) rejected: bool,
}
pub(super) fn text_input(
    ui: &mut egui::Ui,
    old: &str,
    id: egui::Id,
    cap: usize,
    multiline: bool,
    enabled: bool,
    hint: &str,
) -> Edited {
    text_input_behavior(ui, old, id, cap, (multiline, true, false), enabled, hint)
}
/// Opt-in configuration fragments use Tab for native focus navigation. Existing
/// code editors retain Tab indentation and all prior layout/edit behavior.
pub(super) fn configuration_input(
    ui: &mut egui::Ui,
    old: &str,
    id: egui::Id,
    cap: usize,
    enabled: bool,
    hint: &str,
) -> Edited {
    text_input_behavior(ui, old, id, cap, (true, false, false), enabled, hint)
}
/// Opt-in byte-bounded single-line input whose full click target fits the row.
/// Existing input/editor call sites retain their original dimensions.
pub(super) fn fitting_input(
    ui: &mut egui::Ui,
    old: &str,
    id: egui::Id,
    cap: usize,
    enabled: bool,
    hint: &str,
) -> Edited {
    text_input_behavior(ui, old, id, cap, (false, false, true), enabled, hint)
}
fn text_input_behavior(
    ui: &mut egui::Ui,
    old: &str,
    id: egui::Id,
    cap: usize,
    behavior: (bool, bool, bool),
    enabled: bool,
    hint: &str,
) -> Edited {
    let (multiline, indent_with_tab, fitting) = behavior;
    let mut value = old.to_owned();
    let invalid_packet = ui.input(|i| {
        i.events.iter().any(|e| {
            matches!(e, egui::Event::Text(t) | egui::Event::Paste(t)
        if t.len() > cap || t.contains('\0') || (!multiline && t.contains(['\n','\r'])))
        })
    });
    let previous = egui::text_edit::TextEditState::load(ui.ctx(), id).map(|s| {
        let undo = s.undoer();
        (s, undo)
    });
    let mut buffer = Buffer {
        value: &mut value,
        cap,
        multiline,
        rejected: invalid_packet,
        received: false,
    };
    let (response, viewport) = if multiline {
        let height =
            ui.text_style_height(&egui::TextStyle::Monospace) * 7. + 2. * controls::PAD_Y as f32;
        let scroll = controls::nested_scroll(
            ui,
            egui::ScrollArea::vertical()
                .id_salt(("shared-library-editor-scroll", id))
                .max_height(height)
                .min_scrolled_height(height)
                .auto_shrink([false, false]),
            |ui| {
                ui.add_enabled(
                    enabled,
                    egui::TextEdit::multiline(&mut buffer)
                        .id(id)
                        .code_editor()
                        .lock_focus(indent_with_tab)
                        .desired_rows(7)
                        .desired_width(ui.available_width())
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .hint_text(hint),
                )
            },
        );
        (scroll.inner, scroll.inner_rect)
    } else {
        let editor = if fitting {
            controls::singleline_fitting(ui, &mut buffer, ui.available_width())
        } else {
            controls::singleline(ui, &mut buffer).desired_width(ui.available_width())
        };
        let r = ui.add_enabled(enabled, editor.id(id).hint_text(hint));
        let viewport = r.rect;
        (r, viewport)
    };
    let clip = if multiline {
        viewport.intersect(ui.clip_rect())
    } else {
        ui.clip_rect()
    };
    let rejected = enabled && buffer.rejected && (buffer.received || response.has_focus());
    if rejected && let Some((mut previous, undo)) = previous {
        previous.set_undoer(undo);
        previous.store(ui.ctx(), id);
    }
    let changed = enabled && !rejected && response.changed() && value != old;
    Edited {
        response,
        clip,
        viewport,
        value: changed.then_some(value),
        rejected,
    }
}
// A fixed two-slot UI observation cache, not persisted progress. It stores at
// most one latest native inspection per scope; source/context/program mismatch
// is never displayed. An exact bounded input snapshot marks changed state stale.
#[derive(Clone)]
struct Inspection {
    lab: String,
    version: u32,
    visit: u64,
    program: String,
    signature: Vec<u8>,
    output: String,
    completed: bool,
}
fn inspection_key(scope: work::Scope) -> egui::Id {
    egui::Id::new(("shared-library-latest-native-inspection", scope_name(scope)))
}
fn signature(w: &work::Workspace, program: &str) -> Result<Vec<u8>, String> {
    let recipe = w
        .recipes
        .iter()
        .find(|r| r.program == program)
        .ok_or("Select a supplied application")?;
    let bytes = serde_json::to_vec(&(&w.state, recipe)).map_err(|e| e.to_string())?;
    if bytes.len() > 70 * 1024 {
        return Err("Inspection snapshot exceeds its bound".into());
    }
    Ok(bytes)
}
/// Called only after an explicit successful reset. No model or focus mutation.
/// Removes this source/scope's ephemeral inspection, selection and TextEdit undo
/// state so an identical fresh model does not inherit old output or editor undo.
pub(super) fn clear_scope_ui(
    ctx: &egui::Context,
    source: &Scenario,
    visit: u64,
    scope: work::Scope,
    programs: &[String],
) {
    let mut fields = vec![
        egui::Id::new((
            "shared-library-configuration",
            &source.id,
            source.version,
            visit,
            scope_name(scope),
        )),
        egui::Id::new((
            "shared-library-command",
            &source.id,
            source.version,
            visit,
            scope_name(scope),
        )),
    ];
    fields.extend(programs.iter().map(|program| {
        egui::Id::new((
            "shared-library-launch",
            &source.id,
            source.version,
            visit,
            scope_name(scope),
            program,
        ))
    }));
    let selected = egui::Id::new((
        "shared-library-selected-program",
        &source.id,
        source.version,
        visit,
        scope_name(scope),
    ));
    let observed = inspection_key(scope);
    ctx.data_mut(|data| {
        for id in fields {
            data.remove::<egui::text_edit::TextEditState>(id);
        }
        data.remove::<String>(selected);
        if data
            .get_temp::<Inspection>(observed)
            .is_some_and(|o| o.lab == source.id)
        {
            data.remove::<Inspection>(observed);
        }
    });
}
#[derive(Clone, Copy)]
struct PageContext<'a> {
    source: &'a Scenario,
    visit: u64,
    scope: work::Scope,
    world: &'a work::Workspace,
    enabled: bool,
}
fn settings(
    ui: &mut egui::Ui,
    context: PageContext<'_>,
    m: &mut Metrics,
    edits: &mut Vec<Edit>,
    reject_frame: &mut bool,
) {
    let PageContext {
        source,
        visit,
        scope,
        world: w,
        enabled,
    } = context;
    lesson_presentation::rich(
        ui,
        RichText::new("Application search settings · simulated").strong(),
    );
    let key = egui::Id::new((
        "shared-library-selected-program",
        &source.id,
        source.version,
        visit,
        scope_name(scope),
    ));
    let mut selected = ui
        .ctx()
        .data(|d| d.get_temp::<String>(key))
        .filter(|p| w.recipes.iter().any(|r| &r.program == p))
        .unwrap_or_else(|| w.recipes[0].program.clone());
    let picker = controls::dropdown(ui, key, ui.available_width())
        .selected_text(&selected)
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            let start = m.controls.len();
            for recipe in &w.recipes {
                let r = ui.selectable_value(&mut selected, recipe.program.clone(), &recipe.program);
                record(m, format!("program:{}", recipe.program), &r, ui);
            }
            let clip = controls::popup_content_clip(ui);
            for row in &mut m.controls[start..] {
                row.2 = clip;
            }
        });
    record(m, "program-picker", &picker.response, ui);
    ui.ctx().data_mut(|d| d.insert_temp(key, selected.clone()));
    let program = w
        .state
        .catalog
        .programs
        .iter()
        .find(|p| p.path == selected)
        .expect("Validated source roster");
    lesson_presentation::monospace(
        ui,
        &format!("Requests: {}", program.needed.join(", ")),
        true,
    );
    let draft = w
        .launch_drafts
        .iter()
        .find(|r| r.program == selected)
        .expect("Validated source drafts");
    let recipe = w.recipes.iter().find(|r| r.program == selected).unwrap();
    ui.label("Library search directories · leave empty for a clean environment");
    let id = egui::Id::new((
        "shared-library-launch",
        &source.id,
        source.version,
        visit,
        scope_name(scope),
        &selected,
    ));
    let edited = text_input(
        ui,
        &draft.value,
        id,
        1024,
        false,
        enabled,
        "Absolute directory paths separated by :",
    );
    m.fields.push((
        format!("launch:{selected}"),
        edited.response.rect,
        id,
        edited.clip,
    ));
    *reject_frame |= edited.rejected;
    if let Some(value) = edited.value {
        edits.push(Edit::Launch(selected.clone(), value));
    }
    lesson_presentation::prose(
        ui,
        &format!(
            "Saved setting: {}",
            recipe
                .environment
                .as_deref()
                .unwrap_or("Clean · no extra directories")
        ),
    );
    if draft.value != recipe.environment.as_deref().unwrap_or_default() {
        ui.label("Draft not applied · inspections use the saved setting.");
    }
    ui.horizontal_wrapped(|ui| {
        let save = ui.add_enabled(enabled, button("Apply launch settings"));
        record(m, format!("apply-launch:{selected}"), &save, ui);
        if save.clicked() {
            edits.push(Edit::SaveLaunch(selected.clone()));
        }
        let inspect = ui.add_enabled(enabled, button("Inspect launch settings"));
        record(m, format!("inspect-launch:{selected}"), &inspect, ui);
        if inspect.clicked() {
            edits.push(Edit::InspectLaunch(selected.clone()));
        }
    });
    let observation = ui
        .ctx()
        .data(|d| d.get_temp::<Inspection>(inspection_key(scope)))
        .filter(|o| {
            o.lab == source.id
                && o.version == source.version
                && o.visit == visit
                && o.program == selected
        });
    if let Some(observation) = observation {
        let fresh = signature(w, &selected).is_ok_and(|s| s == observation.signature);
        ui.strong(if !fresh {
            "Earlier inspection · saved state has changed"
        } else if observation.completed {
            "Actual saved-setting inspection"
        } else {
            "Inspection could not complete · read the diagnostic"
        });
        if !fresh {
            ui.label("Inspect again to observe the current saved settings.");
        }
        let h = ui.text_style_height(&egui::TextStyle::Monospace) * 5.;
        let out = controls::nested_scroll(
            ui,
            egui::ScrollArea::vertical()
                .id_salt(("shared-library-native-output", key))
                .max_height(h)
                .auto_shrink([false, true]),
            |ui| {
                lesson_presentation::monospace(ui, &observation.output, true);
            },
        );
        m.inspection = Some(out.inner_rect);
    } else {
        ui.label("Not inspected yet · choose Inspect launch settings.");
    }
}
fn configuration(
    ui: &mut egui::Ui,
    context: PageContext<'_>,
    m: &mut Metrics,
    edits: &mut Vec<Edit>,
    reject_frame: &mut bool,
) {
    let PageContext {
        source,
        visit,
        scope,
        world: w,
        enabled,
    } = context;
    lesson_presentation::rich(
        ui,
        RichText::new("System search configuration · virtual").strong(),
    );
    lesson_presentation::monospace(ui, "/etc/ld.so.conf", true);
    let id = egui::Id::new((
        "shared-library-configuration",
        &source.id,
        source.version,
        visit,
        scope_name(scope),
    ));
    let edited = text_input(
        ui,
        &w.configuration_draft,
        id,
        4096,
        true,
        enabled,
        "One supplied absolute directory per line",
    );
    m.fields.push((
        "configuration".into(),
        edited.response.rect,
        id,
        edited.clip,
    ));
    *reject_frame |= edited.rejected;
    if let Some(value) = edited.value {
        edits.push(Edit::Configuration(value));
    }
    // Save stays outside the editor scrollbar; its row width never follows it.
    let save = ui.add_enabled(enabled, button("Save configuration"));
    record(m, "save-configuration", &save, ui);
    if save.clicked() {
        edits.push(Edit::SaveConfiguration);
    }
    ui.label(if w.configuration_draft == w.state.configuration {
        "Editor matches the saved virtual file."
    } else {
        "Unsaved draft · cache operations use the saved file."
    });
    let current = w.state.cache_matches_configuration().unwrap_or(false);
    ui.label(if current {
        "Cache: built from the current saved directory list."
    } else {
        "Cache: not rebuilt for the current saved directory list."
    });
    ui.label("Save changes the virtual file. Run ldconfig to rebuild its cache.");
}
fn investigation(
    ui: &mut egui::Ui,
    context: PageContext<'_>,
    m: &mut Metrics,
    edits: &mut Vec<Edit>,
    reject_frame: &mut bool,
) {
    let PageContext {
        source,
        visit,
        scope,
        world: w,
        enabled,
    } = context;
    lesson_presentation::rich(ui, RichText::new("Supplied library inventory").strong());
    let rows = ui.text_style_height(&egui::TextStyle::Body) * 7.;
    let inventory = controls::nested_scroll(
        ui,
        egui::ScrollArea::vertical()
            .id_salt(("shared-library-inventory", &source.id, scope_name(scope)))
            .max_height(rows)
            .auto_shrink([false, true]),
        |ui| {
            let wide = ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 35.;
            if wide {
                let width = (ui.available_width() - ui.spacing().item_spacing.x * 2.) / 3.;
                egui::Grid::new((
                    "shared-library-inventory-grid",
                    &source.id,
                    scope_name(scope),
                ))
                .num_columns(3)
                .max_col_width(width)
                .show(ui, |ui| {
                    for title in ["Library name", "Supplied file", "Also needs"] {
                        ui.strong(title);
                    }
                    ui.end_row();
                    for o in &w.state.catalog.objects {
                        lesson_presentation::monospace(ui, &o.soname, true);
                        lesson_presentation::monospace(ui, &o.path, true);
                        let needed = if o.needed.is_empty() {
                            "—".into()
                        } else {
                            o.needed.join(", ")
                        };
                        lesson_presentation::monospace(ui, &needed, true);
                        ui.end_row();
                    }
                });
            } else {
                for o in &w.state.catalog.objects {
                    lesson_presentation::monospace(ui, &o.soname, true);
                    lesson_presentation::monospace(ui, &o.path, true);
                    if !o.needed.is_empty() {
                        ui.label(format!("Also needs: {}", o.needed.join(", ")));
                    }
                    ui.add_space(6.);
                }
            }
        },
    );
    m.inventory = Some(inventory.inner_rect);
    ui.add_space(8.);
    lesson_presentation::rich(
        ui,
        RichText::new("Investigation terminal · simulated").strong(),
    );
    let h = ui.text_style_height(&egui::TextStyle::Monospace) * 7.;
    let output = controls::nested_scroll(
        ui,
        egui::ScrollArea::vertical()
            .id_salt((
                "shared-library-history",
                &source.id,
                visit,
                scope_name(scope),
            ))
            .max_height(h)
            .min_scrolled_height(h)
            .auto_shrink([false, false])
            .stick_to_bottom(true),
        |ui| {
            if w.transcript.is_empty() {
                ui.label("No terminal commands run yet.");
            }
            for row in &w.transcript {
                let kind = match row.outcome {
                    work::Outcome::Supported => "Supported simulated operation",
                    work::Outcome::Unsupported => "Unsupported syntax or behavior",
                    work::Outcome::Invalid => "Invalid input",
                    work::Outcome::NotFound => "Unknown supplied item",
                    work::Outcome::Limit => "Simulation limit",
                };
                lesson_presentation::monospace(
                    ui,
                    &format!("virtual $ {}\n{kind}\n{}", row.input, row.output),
                    true,
                );
            }
        },
    );
    m.history = Some(output.inner_rect);
    ui.label("Actual command history · a supported inspection can still show a missing library.");
    let id = egui::Id::new((
        "shared-library-command",
        &source.id,
        source.version,
        visit,
        scope_name(scope),
    ));
    let edited = text_input(
        ui,
        &w.command,
        id,
        work::MAX_COMMAND,
        false,
        enabled,
        "ldd PROGRAM",
    );
    m.fields
        .push(("command".into(), edited.response.rect, id, edited.clip));
    *reject_frame |= edited.rejected;
    let command = edited.value.clone().unwrap_or_else(|| w.command.clone());
    if let Some(value) = edited.value {
        edits.push(Edit::Command(value));
    }
    let after_enter = ui.input(|i| {
        let mut entered = false;
        i.events.iter().any(|e| {
            if matches!(
                e,
                egui::Event::Key {
                    key: egui::Key::Enter,
                    pressed: true,
                    ..
                }
            ) {
                entered = true;
            }
            entered && matches!(e,egui::Event::Text(t)|egui::Event::Paste(t)if !t.is_empty())
        })
    });
    let enter = enabled
        && !edited.rejected
        && edited.response.lost_focus()
        && ui.input(|i| i.key_pressed(egui::Key::Enter));
    if enter && after_enter {
        *reject_frame = true;
    }
    let run = ui.add_enabled(enabled && !edited.rejected, button("Run command"));
    record(m, "run", &run, ui);
    if run.clicked() || (enter && !after_enter) {
        edits.push(Edit::Run(command));
    }
}
fn worked_intro(ui: &mut egui::Ui, d: &work::Definition) {
    lesson_presentation::worked(ui, &d.lesson.description);
    for (n, step) in d.lesson.worked.iter().enumerate() {
        lesson_presentation::rich(
            ui,
            RichText::new(format!("{}. {}", n + 1, step.title)).strong(),
        );
        match &step.action {
            work::WorkedAction::Inspect { command } => {
                lesson_presentation::monospace(ui, command, true)
            }
            work::WorkedAction::Configuration { text } => {
                lesson_presentation::monospace(ui, text, true)
            }
            work::WorkedAction::LaunchRecipe {
                program,
                environment,
            } => lesson_presentation::monospace(
                ui,
                &format!(
                    "Simulated setting for {program}: {}",
                    environment
                        .as_deref()
                        .unwrap_or("empty · clean environment")
                ),
                true,
            ),
            work::WorkedAction::RebuildCache => {
                lesson_presentation::monospace(ui, "ldconfig", true)
            }
        };
        lesson_presentation::prose(ui, &step.explanation);
        ui.add_space(6.);
    }
}
impl LabApp {
    pub(super) fn shared_libraries_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let before = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if before != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let attempt = &self.progress.attempts[&source.id];
        let (State::SharedLibraries { simulation }, engine::Goal::SharedLibraries { definition }) =
            (&attempt.state, &source.goal)
        else {
            lesson_presentation::prose(
                ui,
                "This activity requires its supplied shared-library workspace. No operation was applied.",
            );
            return;
        };
        if let Err(e) = simulation.bound(definition) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let original = (**simulation).clone();
        let mut next = original.clone();
        let visit = attempt.visited;
        let allowed = ui.is_enabled()
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let mut view = if challenge {
            work::View::Assignment
        } else {
            next.view
        };
        let mut edits = vec![];
        let mut reject_frame = false;
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (work::View::Introduction, "Introduction"),
                    (work::View::Example, "Try example · ungraded"),
                    (work::View::Assignment, "Assignment"),
                ] {
                    let r = ui.add_enabled(allowed, egui::Button::selectable(view == v, label));
                    self.metrics.lesson_tabs.push(r.rect);
                    record(
                        &mut self.metrics.shared_libraries,
                        format!("view:{v:?}"),
                        &r,
                        ui,
                    );
                    if r.clicked() {
                        view = v;
                        edits.push(Edit::View(v));
                    }
                }
            });
        }
        let scope = if view == work::View::Example {
            work::Scope::Example
        } else {
            work::Scope::Assignment
        };
        if view == work::View::Introduction {
            let intro = lesson_presentation::concepts(
                ui,
                &source,
                &definition.lesson.prerequisite_note,
                definition
                    .lesson
                    .concepts
                    .iter()
                    .map(|c| (c.title.clone(), c.text.clone())),
            );
            self.metrics.shared_libraries.intro_goal = Some(intro.objective.rect);
            self.metrics.shared_libraries.intro_context = Some(intro.context.rect);
            worked_intro(ui, definition);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (work::View::Example, "start-example", "Try this example"),
                    (
                        work::View::Assignment,
                        "start-assignment",
                        "Go straight to assignment",
                    ),
                ] {
                    let r = ui.add_enabled(allowed, button(label));
                    record(&mut self.metrics.shared_libraries, key, &r, ui);
                    if r.clicked() {
                        edits.push(Edit::View(v));
                    }
                }
            });
        } else {
            let practice = scope == work::Scope::Example;
            if practice {
                lesson_presentation::worked(ui, &definition.lesson.description);
                let mut selected = next.selected_example;
                let picker = controls::dropdown(
                    ui,
                    ("shared-library-guided", &source.id, source.version, visit),
                    ui.available_width(),
                )
                .selected_text(&definition.lesson.worked[selected].title)
                .show_ui(ui, |ui| {
                    controls::fit_popup_contents(ui);
                    let first = self.metrics.shared_libraries.controls.len();
                    for (n, step) in definition.lesson.worked.iter().enumerate() {
                        let r = ui.selectable_value(&mut selected, n, &step.title);
                        record(
                            &mut self.metrics.shared_libraries,
                            format!("step:{n}"),
                            &r,
                            ui,
                        );
                    }
                    let clip = controls::popup_content_clip(ui);
                    for row in &mut self.metrics.shared_libraries.controls[first..] {
                        row.2 = clip;
                    }
                });
                record(
                    &mut self.metrics.shared_libraries,
                    "step-picker",
                    &picker.response,
                    ui,
                );
                if selected != next.selected_example {
                    edits.push(Edit::SelectStep(selected));
                }
                let step = &definition.lesson.worked[selected];
                lesson_presentation::rich(ui, RichText::new("Current guided action").strong());
                lesson_presentation::prose(ui, &step.explanation);
                match &step.action {
                    work::WorkedAction::Inspect { command } => {
                        lesson_presentation::monospace(ui, command, true);
                        let r = ui.add_enabled(allowed, button("Insert command"));
                        record(&mut self.metrics.shared_libraries, "insert-command", &r, ui);
                        if r.clicked() {
                            edits.push(Edit::Command(command.clone()));
                        }
                    }
                    work::WorkedAction::RebuildCache => {
                        lesson_presentation::monospace(ui, "ldconfig", true);
                        let r = ui.add_enabled(allowed, button("Insert command"));
                        record(&mut self.metrics.shared_libraries, "insert-command", &r, ui);
                        if r.clicked() {
                            edits.push(Edit::Command("ldconfig".into()));
                        }
                    }
                    work::WorkedAction::Configuration { text } => {
                        let r = ui.add_enabled(allowed, button("Fill practice configuration"));
                        record(
                            &mut self.metrics.shared_libraries,
                            "fill-configuration",
                            &r,
                            ui,
                        );
                        if r.clicked() {
                            edits.push(Edit::Configuration(text.clone()));
                        }
                    }
                    work::WorkedAction::LaunchRecipe {
                        program,
                        environment,
                    } => {
                        lesson_presentation::monospace(ui, program, true);
                        let r = ui.add_enabled(allowed, button("Fill practice launch field"));
                        record(&mut self.metrics.shared_libraries, "fill-launch", &r, ui);
                        if r.clicked() {
                            let selected_key = egui::Id::new((
                                "shared-library-selected-program",
                                &source.id,
                                source.version,
                                visit,
                                scope_name(scope),
                            ));
                            ui.ctx()
                                .data_mut(|d| d.insert_temp(selected_key, program.clone()));
                            edits.push(Edit::Launch(
                                program.clone(),
                                environment.clone().unwrap_or_default(),
                            ));
                        }
                    }
                }
            } else {
                let task = lesson_presentation::assignment_responses(ui, &source);
                self.metrics.shared_libraries.goal = Some(task.whole.rect);
                self.metrics.shared_libraries.goal_anchor = Some(task.goal.rect);
            }
            let w = world(&next, scope);
            let plan = if practice {
                &definition.lesson.plan
            } else {
                &definition.plan
            };
            let settings_first = plan.preserve_configuration_and_cache;
            // Width derives from readable font rows; larger text naturally stacks.
            let wide = ui.available_width()
                >= ui.text_style_height(&egui::TextStyle::Body) * 52. + controls::GAP;
            let render_context = PageContext {
                source: &source,
                visit,
                scope,
                world: w,
                enabled: allowed,
            };
            let editing = |ui: &mut egui::Ui,
                           m: &mut Metrics,
                           edits: &mut Vec<Edit>,
                           reject_frame: &mut bool| {
                if settings_first {
                    settings(ui, render_context, m, edits, reject_frame);
                    ui.push_id(
                        (
                            "shared-library-global-config",
                            &source.id,
                            source.version,
                            visit,
                            scope_name(scope),
                        ),
                        |ui| {
                            controls::collapsing_header(
                                ui,
                                "System configuration · keep unchanged for this task",
                            )
                            .show(ui, |ui| {
                                configuration(ui, render_context, m, edits, reject_frame);
                            });
                        },
                    );
                } else {
                    configuration(ui, render_context, m, edits, reject_frame);
                    ui.add_space(10.);
                    settings(ui, render_context, m, edits, reject_frame);
                }
            };
            if wide {
                ui.columns(2, |columns| {
                    editing(
                        &mut columns[0],
                        &mut self.metrics.shared_libraries,
                        &mut edits,
                        &mut reject_frame,
                    );
                    investigation(
                        &mut columns[1],
                        render_context,
                        &mut self.metrics.shared_libraries,
                        &mut edits,
                        &mut reject_frame,
                    );
                });
            } else {
                editing(
                    ui,
                    &mut self.metrics.shared_libraries,
                    &mut edits,
                    &mut reject_frame,
                );
                ui.add_space(10.);
                investigation(
                    ui,
                    render_context,
                    &mut self.metrics.shared_libraries,
                    &mut edits,
                    &mut reject_frame,
                );
            }
            ui.horizontal_wrapped(|ui| {
                let check = ui.add_enabled(allowed && !reject_frame, button("Check configuration"));
                record(&mut self.metrics.shared_libraries, "check", &check, ui);
                self.metrics.check = Some(check.rect);
                if check.clicked() {
                    edits.push(Edit::Check);
                }
                if practice {
                    let reset = ui.add_enabled(allowed, button("Reset example only"));
                    record(
                        &mut self.metrics.shared_libraries,
                        "reset-example",
                        &reset,
                        ui,
                    );
                    if reset.clicked() {
                        edits.push(Edit::ResetExample);
                    }
                    let assignment = ui.add_enabled(allowed, button("Open full assignment"));
                    record(
                        &mut self.metrics.shared_libraries,
                        "assignment",
                        &assignment,
                        ui,
                    );
                    if assignment.clicked() {
                        edits.push(Edit::View(work::View::Assignment));
                    }
                }
            });
            if practice
                && let Some(step) =
                    lesson_presentation::entry(&source).and_then(|e| e.practice.first())
            {
                lesson_presentation::rich(ui, RichText::new(&step.title).strong());
                lesson_presentation::prose(ui, &step.action);
                lesson_presentation::prose(ui, &format!("Observe: {}", step.observe));
            }
        }
        // Rejected text never publishes a partial draft or executes another queued
        // action from the same malformed input packet. Cursor/undo restoration is
        // handled by the native TextEdit wrapper; no focus is forced.
        if reject_frame {
            edits.clear();
            self.notice=Some("Nothing was applied or run. The text exceeds this field's limit or contains a disallowed line break; your earlier draft was retained.".into());
        }
        if allowed {
            for edit in edits {
                let result = match edit {
                    Edit::View(v) => next.set_view(definition, v),
                    Edit::SelectStep(n) => next.select_example(definition, n),
                    Edit::Configuration(text) => next.edit_configuration(definition, scope, text),
                    Edit::SaveConfiguration => next.save_configuration(definition, scope),
                    Edit::Launch(program, text) => {
                        next.edit_launch(definition, scope, &program, text)
                    }
                    Edit::SaveLaunch(program) => next.save_launch(definition, scope, &program),
                    Edit::Command(command) => next.set_command(definition, scope, command),
                    Edit::Run(command) => next.run(definition, scope, &command).map(|_| ()),
                    Edit::Check => next.check(definition, scope).map(|_| ()),
                    Edit::ResetExample => next.reset(definition, work::Scope::Example).map(|()| {
                        let programs = definition
                            .lesson
                            .initial
                            .catalog
                            .programs
                            .iter()
                            .map(|p| p.path.clone())
                            .collect::<Vec<_>>();
                        clear_scope_ui(ui.ctx(), &source, visit, work::Scope::Example, &programs);
                    }),
                    Edit::InspectLaunch(program) => {
                        match signature(world(&next, scope), &program) {
                            Err(e) => Err(e),
                            Ok(signature) => {
                                let actual =
                                    next.inspect_recipe(definition, scope, &program).and_then(
                                        |resolved| resolved.report().map_err(|e| e.to_string()),
                                    );
                                let (output, completed) = match actual {
                                    Ok(output) => (output, true),
                                    Err(e) => (e, false),
                                };
                                if output.len() > 32 * 1024 {
                                    Err("Native inspection output exceeded its bound; no partial result displayed".into())
                                } else {
                                    let observation = Inspection {
                                        lab: source.id.clone(),
                                        version: source.version,
                                        visit,
                                        program,
                                        signature,
                                        output,
                                        completed,
                                    };
                                    ui.ctx().data_mut(|d| {
                                        d.insert_temp(inspection_key(scope), observation)
                                    });
                                    Ok(())
                                }
                            }
                        }
                    }
                };
                if let Err(e) = result {
                    self.notice = Some(e);
                }
            }
        }
        if view != work::View::Introduction {
            let w = world(&next, scope);
            if w.checked {
                match next.current_report(definition, scope) {
                    Ok(report) => {
                        let passed = report.checks.iter().filter(|r| r.passed).count();
                        let success = report.passed();
                        let title = format!(
                            "{} · {passed}/{} requirements{}",
                            if success { "Passed" } else { "Needs work" },
                            report.checks.len(),
                            if scope == work::Scope::Example {
                                " · example, ungraded"
                            } else {
                                ""
                            }
                        );
                        let r = lesson_presentation::rich(
                            ui,
                            RichText::new(title).strong().color(if success {
                                palette(ui).success
                            } else {
                                palette(ui).error
                            }),
                        );
                        self.metrics.shared_libraries.result =
                            Some((passed, report.checks.len(), r.rect));
                        for check in report.checks.iter().filter(|r| !r.passed) {
                            lesson_presentation::rich(ui, RichText::new(&check.label).strong());
                            lesson_presentation::prose(ui, &check.feedback);
                        }
                        ui.push_id(
                            (
                                "shared-library-objectives",
                                &source.id,
                                source.version,
                                visit,
                                scope_name(scope),
                                challenge,
                            ),
                            |ui| {
                                controls::collapsing_header(ui, "Objective results").show(
                                    ui,
                                    |ui| {
                                        for check in &report.checks {
                                            lesson_presentation::prose(
                                                ui,
                                                &format!(
                                                    "{} · {}: {}",
                                                    if check.passed {
                                                        "Passed"
                                                    } else {
                                                        "Needs work"
                                                    },
                                                    check.label,
                                                    check.feedback
                                                ),
                                            );
                                        }
                                    },
                                );
                            },
                        );
                        if scope == work::Scope::Assignment
                            && success
                            && next.current_success(definition)
                        {
                            self.metrics.completion = true;
                            lesson_presentation::prose(ui, &source.explanation);
                            let back =
                                ui.add_enabled(allowed && !reject_frame, button("Back to library"));
                            record(&mut self.metrics.shared_libraries, "library", &back, ui);
                            if back.clicked() {
                                self.selected = None;
                            }
                            ui.add_enabled_ui(allowed && !reject_frame, |ui| {
                                self.next_unit_lab(ui, i)
                            });
                        }
                    }
                    Err(e) => {
                        lesson_presentation::prose(ui, &e);
                    }
                }
            } else {
                let text = if w.pending() {
                    "Not checked · apply or restore your pending drafts, then Check configuration."
                } else {
                    "Not checked · inspect or change the saved settings, then Check configuration."
                };
                let r = lesson_presentation::prose(ui, text);
                record(&mut self.metrics.shared_libraries, "unchecked", &r, ui);
            }
            ui.push_id(
                (
                    "shared-library-help",
                    &source.id,
                    source.version,
                    visit,
                    scope_name(scope),
                    challenge,
                ),
                |ui| {
                    let help =
                        controls::collapsing_header(ui, "Optional help and simulation limits")
                            .show(ui, |ui| {
                                if scope == work::Scope::Assignment {
                                    let a = self.progress.attempts.get_mut(&source.id).unwrap();
                                    let hint = ui.add_enabled(
                                        allowed && !reject_frame && a.hints < source.hints.len(),
                                        button("Show a hint"),
                                    );
                                    record(&mut self.metrics.shared_libraries, "hint", &hint, ui);
                                    if hint.clicked() {
                                        a.hints += 1;
                                        self.dirty = true;
                                    }
                                    for hint in source.hints.iter().take(a.hints) {
                                        lesson_presentation::prose(ui, hint);
                                    }
                                    let worked = controls::collapsing_header(
                                        ui,
                                        "Worked approach · reveals a solution",
                                    )
                                    .show(ui, |ui| {
                                        // Human-authored explanation only; never show serialized backend actions as Linux commands.
                                        lesson_presentation::prose(ui, &source.explanation);
                                    });
                                    record(
                                        &mut self.metrics.shared_libraries,
                                        "worked-help",
                                        &worked.header_response,
                                        ui,
                                    );
                                }
                                lesson_presentation::prose(ui, work::HELP);
                                controls::collapsing_header(ui, "References and boundaries").show(
                                    ui,
                                    |ui| {
                                        for url in &source.sources {
                                            ui.hyperlink_to(url, url);
                                        }
                                        lesson_presentation::prose(ui, &source.simplifications);
                                    },
                                );
                            });
                    record(
                        &mut self.metrics.shared_libraries,
                        "help",
                        &help.header_response,
                        ui,
                    );
                },
            );
        }
        if next != original {
            if let Err(e) = next.bound(definition) {
                self.notice = Some(e);
                return;
            }
            if let State::SharedLibraries { simulation } =
                &mut self.progress.attempts.get_mut(&source.id).unwrap().state
            {
                **simulation = next;
                self.dirty = true;
                self.completion_index.forget(&source.id);
                ui.ctx().request_repaint();
            }
        }
    }
}

#[cfg(test)]
mod viewport_tests {
    use super::*;

    #[test]
    fn long_configuration_target_fits_the_scroll_viewport() {
        let context = egui::Context::default();
        let draft = "Some supplied configuration.\n".repeat(30);
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(480., 420.));
        let _ = context.run(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let input = configuration_input(
                        ui,
                        &draft,
                        egui::Id::new("viewport-example"),
                        8192,
                        true,
                        "Config",
                    );
                    assert!(input.response.rect.height() > input.viewport.height());
                    assert!(!input.clip.contains(input.response.rect.center()));
                    assert!(input.clip.contains(input.viewport.center()));
                });
            },
        );
    }
}
