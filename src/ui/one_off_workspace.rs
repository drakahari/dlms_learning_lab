//! Focused native one-off scheduler. Every action stays inside bounded simulation.
use super::*;
use dlms_learning_lab::{one_off as core, one_off_workspace as work, script_teaching::View};
mod input;
pub(super) mod memo;
#[derive(Default)]
pub(super) struct OneOffMetrics {
    pub fields: Vec<(String, egui::Rect, egui::Id, bool, egui::Rect)>,
    pub controls: Vec<(String, egui::Rect, bool, egui::Rect)>,
    pub choices: Vec<(String, egui::Rect)>,
    pub popup: Option<(String, egui::Rect)>,
    pub documents: Vec<(egui::Rect, egui::Rect, egui::Vec2)>,
    pub labels: Vec<(String, egui::Rect)>,
    pub rows: Vec<(usize, bool, egui::Rect)>,
    pub output: Option<(egui::Rect, egui::Rect)>,
    pub result: Option<egui::Rect>,
    pub goal: Option<egui::Rect>,
}
const EDITOR_UNDOS: usize = 16;
#[derive(Default)]
struct Document {
    key: Option<egui::Id>,
    value: String,
    reset_undo: bool,
}
#[derive(Default)]
struct NativeStore {
    active: Vec<egui::Id>,
    retired: Vec<egui::Id>,
}
impl NativeStore {
    fn clear(&mut self) {
        self.retired.append(&mut self.active);
    }
    fn purge(&mut self, ctx: &egui::Context) {
        for id in self.retired.drain(..) {
            ctx.data_mut(|d| d.remove::<egui::text_edit::TextEditState>(id));
        }
    }
    fn track(&mut self, ctx: &egui::Context, id: egui::Id) {
        if !self.active.contains(&id) {
            // One document, command, UTC end, and at most16 literal variable fields.
            if self.active.len() >= 19 {
                let old = self.active.remove(0);
                ctx.data_mut(|d| d.remove::<egui::text_edit::TextEditState>(old));
            }
            self.active.push(id);
        }
        if egui::text_edit::TextEditState::load(ctx, id).is_none() {
            let mut state = egui::text_edit::TextEditState::default();
            state.set_undoer(Undo::with_settings(egui::util::undoer::Settings {
                max_undos: EDITOR_UNDOS,
                ..Default::default()
            }));
            state.store(ctx, id);
        }
    }
}
struct Job {
    source: Scenario,
    input: Attempt,
    practice: bool,
    pending: work::budget::PendingAttempt,
}
#[derive(Default)]
pub(super) struct Presentation {
    pub memo: memo::Memo,
    native: NativeStore,
    owner: Option<egui::Id>,
    epoch: u64,
    actor: String,
    selected: String,
    bound_selected: String,
    command: String,
    end: String,
    environment: std::collections::BTreeMap<String, String>,
    document: Document,
    file: String,
    job: Option<Job>,
    advanced: Option<u64>,
    veto_frame: Option<u64>,
    blocked_gesture: bool,
    checked: [bool; 2],
    notice: Option<String>,
    focus_document: bool,
    command_executions: u64,
}
impl Presentation {
    pub(super) fn with_index(certificates: completion_index::Handle) -> Self {
        let mut p = Self::default();
        p.memo.certificates = certificates;
        p
    }
    pub(in crate::ui) fn command_executions(&self) -> u64 {
        self.command_executions
    }
    #[cfg(test)]
    pub(in crate::ui) fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }
    #[cfg(test)]
    pub(in crate::ui) fn gesture_blocked(&self) -> bool {
        self.blocked_gesture
    }
    #[cfg(test)]
    pub(in crate::ui) fn command_value(&self) -> &str {
        &self.command
    }
    pub fn clear(&mut self) {
        self.native.clear();
        self.owner = None;
        self.epoch = self.epoch.wrapping_add(1);
        self.document = Document::default();
        self.job = None;
        self.checked = [false; 2];
        self.notice = None;
        self.memo.clear();
    }
    pub fn running(&self) -> bool {
        self.job.is_some()
    }
    fn pending_text(&self, ctx: &egui::Context) -> Option<egui::Id> {
        let focused = ctx.memory(|m| m.focused());
        let text = ctx.input(|i| {
            i.events.iter().any(|e| match e {
                egui::Event::Paste(t) => !t.is_empty(),
                egui::Event::Text(t) => !t.is_empty() && t != "\n" && t != "\r",
                _ => false,
            })
        });
        if text {
            focused.filter(|id| self.native.active.contains(id))
        } else {
            None
        }
    }
    fn bind(&mut self, scope: egui::Id, s: &work::Session) {
        if self.owner != Some(scope) {
            self.native.clear();
            self.owner = Some(scope);
            self.document = Document::default();
            self.checked = [false; 2];
        }
        if self.bound_selected != s.selected {
            self.native.clear();
            self.document = Document::default();
            self.bound_selected = s.selected.clone();
        }
        self.actor = s.actor.clone();
        self.selected = s.selected.clone();
        self.command = s.command.clone();
        self.end = s.window_end.clone();
        self.environment = s.environment.clone();
    }
}
fn selected(a: &Attempt, practice: bool) -> Option<(&work::Assignment, &work::Session)> {
    let w = a.workspace.as_ref()?;
    if practice {
        let l = w.one_off_learning.as_ref()?;
        Some((&l.assignment, &l.session))
    } else {
        let engine::State::OneOff { simulation } = &a.state else {
            return None;
        };
        Some((simulation, w.one_off_session.as_ref()?))
    }
}
fn definition(s: &Scenario, practice: bool) -> Option<&work::Definition> {
    if practice {
        Some(&s.one_off_teaching.as_ref()?.definition)
    } else {
        let engine::Goal::OneOff { definition } = &s.goal else {
            return None;
        };
        Some(definition)
    }
}
fn view(a: &Attempt) -> View {
    if a.challenge {
        View::Assignment
    } else {
        a.workspace
            .as_ref()
            .and_then(|w| w.one_off_learning.as_ref())
            .map_or(View::Assignment, |l| l.view)
    }
}
type Undo = egui::util::undoer::Undoer<(egui::text::CCursorRange, String)>;
struct NativeBefore {
    id: egui::Id,
    state: egui::text_edit::TextEditState,
    undo: Undo,
}
impl NativeBefore {
    fn capture(ui: &egui::Ui, id: egui::Id) -> Option<Self> {
        if ui.memory(|memory| memory.focused()) != Some(id) {
            return None;
        }
        let editing = ui.input(|input| {
            input.events.iter().any(|event| match event {
                egui::Event::Text(v) | egui::Event::Paste(v) => !v.is_empty(),
                egui::Event::Key {
                    key, pressed: true, ..
                } => matches!(
                    key,
                    egui::Key::Backspace | egui::Key::Delete | egui::Key::Z | egui::Key::Y
                ),
                _ => false,
            })
        });
        if !editing {
            return None;
        }
        let state = egui::text_edit::TextEditState::load(ui.ctx(), id)?;
        let undo = state.undoer();
        Some(Self { id, state, undo })
    }
    fn restore(mut self, ctx: &egui::Context) {
        self.state.set_undoer(self.undo);
        self.state.store(ctx, self.id);
    }
}
#[derive(Default)]
struct Edit {
    changed: bool,
    rejected: bool,
    received: bool,
    id: Option<egui::Id>,
    native: Option<NativeBefore>,
}
struct Buffer<'a> {
    value: &'a mut String,
    cap: usize,
    before: Option<String>,
    rejected: bool,
    received: bool,
}
impl Buffer<'_> {
    fn remember(&mut self) {
        if self.before.is_none() {
            self.before = Some(self.value.clone());
        }
    }
    fn finish(mut self) -> Edit {
        if self.rejected
            && let Some(old) = self.before.take()
        {
            *self.value = old;
        }
        Edit {
            changed: !self.rejected && self.before.as_ref().is_some_and(|old| old != self.value),
            rejected: self.rejected,
            received: self.received,
            id: None,
            native: None,
        }
    }
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
        if self.rejected || text.is_empty() {
            return 0;
        }
        self.received = true;
        if text.contains('\0')
            || self
                .value
                .len()
                .checked_add(text.len())
                .is_none_or(|n| n > self.cap)
        {
            self.rejected = true;
            return 0;
        }
        self.remember();
        self.value.reserve_exact(text.len());
        egui::TextBuffer::insert_text(self.value, text, at)
    }
    fn delete_char_range(&mut self, range: std::ops::Range<usize>) {
        if !self.rejected && !range.is_empty() {
            self.remember();
            egui::TextBuffer::delete_char_range(self.value, range);
        }
    }
}
fn action(ui: &mut egui::Ui, m: &mut OneOffMetrics, key: &str, text: &str, main: bool) -> bool {
    action_enabled(ui, m, key, text, main, true)
}
fn action_enabled(
    ui: &mut egui::Ui,
    m: &mut OneOffMetrics,
    key: &str,
    text: &str,
    main: bool,
    enabled: bool,
) -> bool {
    // Add the button directly to the wrapping parent. A nested child scope can
    // claim the remaining row before its full width is known and expand the form.
    let size = egui::vec2(controls::button_width(ui, text), controls::height(ui));
    let widget = if main {
        primary(ui, text)
    } else {
        button(text)
    };
    let r = ui.add_enabled(enabled, widget.min_size(size));
    m.controls
        .push((key.into(), r.rect, r.enabled(), ui.clip_rect()));
    r.clicked()
}

fn label(ui: &mut egui::Ui, m: &mut OneOffMetrics, text: impl Into<String>) {
    let text = text.into();
    let r = ui
        .with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.add(egui::Label::new(&text).wrap())
        })
        .inner;
    m.labels.push((text, r.rect));
}
fn editor(
    ui: &mut egui::Ui,
    m: &mut OneOffMetrics,
    scope: egui::Id,
    target: &str,
    current: &str,
    doc: &mut Document,
    focus: &mut bool,
) -> Edit {
    let id = scope.with(("document", target));
    let replaced = doc.key == Some(id) && doc.value != current;
    if doc.key != Some(id) || doc.value != current {
        doc.key = Some(id);
        doc.value = current.into();
    }
    if (std::mem::take(&mut doc.reset_undo) || replaced)
        && let Some(mut state) = egui::text_edit::TextEditState::load(ui.ctx(), id)
    {
        state.set_undoer(Undo::with_settings(egui::util::undoer::Settings {
            max_undos: EDITOR_UNDOS,
            ..Default::default()
        }));
        state.store(ui.ctx(), id);
    }
    let height =
        ui.text_style_height(&egui::TextStyle::Monospace) * 5. + 2. * f32::from(controls::PAD_Y);
    let size = egui::vec2(
        ui.available_width(),
        height + ui.spacing().scroll.allocated_width(),
    );
    let width = (size.x - 2. * f32::from(controls::PAD_X)).max(1.);
    let native = NativeBefore::capture(ui, id);
    let mut buffer = Buffer {
        value: &mut doc.value,
        cap: core::MAX_FILES_BYTES,
        before: None,
        rejected: false,
        received: false,
    };
    let (outer, scrolled) = controls::fixed_viewport(ui, size, |ui| {
        egui::ScrollArea::both()
            .id_salt(id.with("scroll"))
            .max_height(size.y)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut buffer)
                        .id(id)
                        .font(egui::TextStyle::Monospace)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .desired_width(width)
                        .desired_rows(5),
                )
            })
    });
    let r = scrolled.inner;
    r.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::TextEdit,
            r.enabled(),
            "Virtual script or input file editor",
        )
    });
    if *focus && r.enabled() {
        r.request_focus();
        *focus = false;
    }
    m.documents
        .push((outer, scrolled.inner_rect, scrolled.content_size));
    m.fields.push((
        "document".into(),
        r.rect.intersect(scrolled.inner_rect),
        r.id,
        r.enabled(),
        scrolled.inner_rect.intersect(ui.clip_rect()),
    ));
    let mut edit = buffer.finish();
    edit.id = Some(r.id);
    edit.native = native;
    edit
}
fn end_field(ui: &mut egui::Ui, m: &mut OneOffMetrics, scope: egui::Id, end: &mut String) -> Edit {
    controls::form_row(ui, "Run through (UTC)", |ui, width| {
        let native = NativeBefore::capture(ui, scope.with("end"));
        let mut b = Buffer {
            value: end,
            cap: 32,
            before: None,
            rejected: false,
            received: false,
        };
        let r = ui.add_sized(
            [width, controls::height(ui)],
            controls::singleline(ui, &mut b)
                .id(scope.with("end"))
                .hint_text("YYYY-MM-DD HH:MM"),
        );
        r.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::TextEdit,
                r.enabled(),
                "Virtual run end in UTC",
            )
        });
        m.fields
            .push(("end".into(), r.rect, r.id, r.enabled(), ui.clip_rect()));
        let mut edit = b.finish();
        edit.id = Some(r.id);
        edit.native = native;
        edit
    })
    .inner
}
#[derive(Clone, Copy)]
enum Submit {
    Save,
    Command,
    Run,
    Stop,
    Check,
    Insert,
    ResetExample,
}
#[derive(Default)]
struct Edits {
    changed: bool,
    rejected: bool,
    received: bool,
    pending: Option<egui::Id>,
    submit: Option<Submit>,
    navigation: Option<View>,
    example_choice: Option<usize>,
    target: String,
    native: Vec<NativeBefore>,
}
impl Edits {
    fn observe(&mut self, mut e: Edit) {
        self.changed |= e.changed;
        self.rejected |= e.rejected;
        if self.pending == e.id && self.pending.is_some() {
            self.received |= e.received;
        }
        if let Some(n) = e.native.take() {
            self.native.push(n);
        }
    }
    fn queue(&mut self, clicked: bool, s: Submit) {
        if clicked && self.submit.is_none() {
            self.submit = Some(s);
        }
    }
}
fn scalar(
    ui: &mut egui::Ui,
    m: &mut OneOffMetrics,
    id: egui::Id,
    key: &str,
    text: &mut String,
    cap: usize,
    hint: &str,
) -> Edit {
    let native = NativeBefore::capture(ui, id);
    let mut b = Buffer {
        value: text,
        cap,
        before: None,
        rejected: false,
        received: false,
    };
    let r = ui.add_sized(
        [ui.available_width(), controls::height(ui)],
        controls::singleline(ui, &mut b).id(id).hint_text(hint),
    );
    r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, r.enabled(), key));
    m.fields
        .push((key.into(), r.rect, r.id, r.enabled(), ui.clip_rect()));
    let mut e = b.finish();
    e.id = Some(r.id);
    e.native = native;
    e
}
fn editing(
    ui: &mut egui::Ui,
    context: (&work::Definition, &work::Assignment),
    session: &work::Session,
    p: &mut Presentation,
    m: &mut OneOffMetrics,
    scope: egui::Id,
    e: &mut Edits,
) {
    let (d, simulation) = context;
    controls::form_row(ui, "Simulated account", |ui, width| {
        let before = p.actor.clone();
        let r = controls::dropdown(ui, scope.with("actor"), width)
            .selected_text(&p.actor)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for name in &d.initial.users {
                    let r = ui.selectable_value(&mut p.actor, name.clone(), name);
                    m.choices.push((format!("actor:{name}"), r.rect));
                }
                m.popup = Some(("actor".into(), controls::popup_content_clip(ui)));
            });
        e.changed |= p.actor != before;
        m.controls.push((
            "actor".into(),
            r.response.rect,
            r.response.enabled(),
            ui.clip_rect(),
        ));
    });
    controls::form_row(ui, "Virtual file", |ui, width| {
        let before = p.selected.clone();
        let r = controls::dropdown(ui, scope.with("target"), width)
            .selected_text(&p.selected)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for name in &d.editable {
                    let r = ui.selectable_value(&mut p.selected, name.clone(), name);
                    m.choices.push((format!("target:{name}"), r.rect));
                }
                m.popup = Some(("target".into(), controls::popup_content_clip(ui)));
            });
        e.changed |= p.selected != before;
        m.controls.push((
            "target".into(),
            r.response.rect,
            r.response.enabled(),
            ui.clip_rect(),
        ));
    });
    // A selector click changes the next frame's document. Save applies to the
    // actual document painted this frame, after every delivered text edit.
    e.target = session.selected.clone();
    p.native.track(
        ui.ctx(),
        scope.with(("document", session.selected.as_str())),
    );
    e.observe(editor(
        ui,
        m,
        scope,
        &session.selected,
        &session.drafts[&session.selected],
        &mut p.document,
        &mut p.focus_document,
    ));
    ui.horizontal_wrapped(|ui| {
        e.queue(
            action(
                ui,
                m,
                "save",
                "Save simulated file",
                !session.applied(d, simulation),
            ),
            Submit::Save,
        );
    });
    label(
        ui,
        m,
        "Save updates this virtual file. Queued jobs keep their saved copy; input files are read when a job runs.",
    );
    let variables_header=controls::collapsing_header(ui,"Submission variables · Literal values").id_salt(scope.with("variables")).default_open(d.requirements.iter().any(|r|matches!(r,work::Requirement::CapturedEnvironment{..}))).show(ui,|ui|{
        label(ui,m,"These settings are captured by the next at submission. They do not change existing jobs and are not shell export commands. An unchecked name is absent; a checked blank value is an empty value.");
        for name in input::variable_names(d,session).unwrap_or_default(){
            let mut present=p.environment.contains_key(&name);
            ui.horizontal_wrapped(|ui|{let r=ui.checkbox(&mut present,format!("Include {name}"));if r.changed(){e.changed=true;if present{p.environment.insert(name.clone(),String::new());}else{p.environment.remove(&name);}}m.controls.push((format!("include:{name}"),r.rect,r.enabled(),ui.clip_rect()));});
            if present {let id=scope.with(("variable",name.as_str()));p.native.track(ui.ctx(),id);controls::form_row(ui,&name,|ui,_|{let edit=scalar(ui,m,id,&format!("variable:{name}"),p.environment.get_mut(&name).unwrap(),1024,"Literal captured value");e.observe(edit);});}
        }
    });
    m.controls.push((
        "variables".into(),
        variables_header.header_response.rect,
        variables_header.header_response.enabled(),
        ui.clip_rect(),
    ));
    ui.strong("Simulated scheduler terminal");
    label(
        ui,
        m,
        format!(
            "{} · Virtual UTC {}",
            session.actor,
            simulation.simulation.clock.text()
        ),
    );
    p.native.track(ui.ctx(), scope.with("command"));
    e.observe(scalar(
        ui,
        m,
        scope.with("command"),
        "command",
        &mut p.command,
        2048,
        "atq",
    ));
    ui.horizontal_wrapped(|ui| {
        e.queue(
            action(ui, m, "command-run", "Run command", false),
            Submit::Command,
        );
    });
    let height =
        ui.text_style_height(&egui::TextStyle::Monospace) * 5. + 2. * f32::from(controls::PAD_Y);
    let shown = egui::ScrollArea::both()
        .id_salt(scope.with("terminal-output"))
        .max_height(height)
        .min_scrolled_height(height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(if session.output.is_empty() {
                        "Supported queue tools: at, atq, atrm. Open Help for supported forms."
                    } else {
                        &session.output
                    })
                    .monospace(),
                )
                .wrap()
                .selectable(true),
            )
        });
    m.output = Some((shown.inner.rect, shown.inner_rect));
    label(
        ui,
        m,
        "All jobs are virtual. Submitting a job queues it; only Run through advances this activity’s clock. Closing the app does not advance time.",
    );
    p.native.track(ui.ctx(), scope.with("end"));
    e.observe(end_field(ui, m, scope, &mut p.end));
    ui.horizontal_wrapped(|ui| {
        e.queue(action(ui, m, "run", "Run through", true), Submit::Run);
    });
}
fn inspect(
    ui: &mut egui::Ui,
    d: &work::Definition,
    simulation: &work::Assignment,
    session: &work::Session,
    p: &mut Presentation,
    m: &mut OneOffMetrics,
    scope: egui::Id,
) {
    let state = &simulation.simulation;
    ui.strong("Queue and current result");
    let visible = |job: &&core::Job| session.actor == "root" || job.owner == session.actor;
    if !state
        .jobs
        .values()
        .any(|job| session.actor == "root" || job.owner == session.actor)
    {
        label(ui, m, "No queued jobs visible to this simulated account");
    }
    for job in state.jobs.values().filter(visible) {
        label(
            ui,
            m,
            format!(
                "Job {} · {} · due {} UTC · captured {}",
                job.id,
                job.owner,
                job.due.text(),
                job.script_path
            ),
        );
    }
    label(
        ui,
        m,
        "Use at -c ID to inspect a queued job’s saved script and variables.",
    );
    controls::form_row(ui, "View virtual file", |ui, width| {
        if !state.files.contains_key(&p.file) {
            p.file = d
                .outputs
                .iter()
                .find(|v| state.files.contains_key(*v))
                .cloned()
                .or_else(|| state.files.keys().next().cloned())
                .unwrap_or_default();
        }
        let r = controls::dropdown(ui, scope.with("file"), width)
            .selected_text(if p.file.is_empty() {
                "No file"
            } else {
                &p.file
            })
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for name in state.files.keys() {
                    let r = ui.selectable_value(&mut p.file, name.clone(), name);
                    m.choices.push((format!("file:{name}"), r.rect));
                }
                m.popup = Some(("file".into(), controls::popup_content_clip(ui)));
            });
        m.controls.push((
            "file".into(),
            r.response.rect,
            r.response.enabled(),
            ui.clip_rect(),
        ));
    });
    let h = ui.text_style_height(&egui::TextStyle::Monospace) * 6.;
    egui::ScrollArea::both()
        .id_salt(scope.with("file-result"))
        .max_height(h)
        .min_scrolled_height(ui.text_style_height(&egui::TextStyle::Monospace))
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(
                        state
                            .files
                            .get(&p.file)
                            .map(String::as_str)
                            .unwrap_or("No output file yet"),
                    )
                    .monospace(),
                )
                .wrap()
                .selectable(true),
            );
        });
    controls::collapsing_header(ui, "Recent virtual job history · Display only")
        .id_salt(scope.with("history"))
        .show(ui, |ui| {
            label(ui, m, "History is bounded. Check work verifies current state, replay and fresh sample inputs; history lines do not award credit.");
            egui::ScrollArea::vertical()
                .id_salt(scope.with("history-scroll"))
                .max_height(h)
                .show(ui, |ui| {
                    for record in state.records.iter().rev() {
                        label(ui, m, format!(
                            "Job {} · {} · {} UTC · {:?} · exit {}",
                            record.id, record.owner, record.when.text(), record.status,
                            record.exit_status.map_or("—".into(), |n| n.to_string())
                        ));
                        if !record.stdout.is_empty() {
                            ui.monospace(&record.stdout);
                        }
                        if !record.stderr.is_empty() {
                            ui.colored_label(palette(ui).error, &record.stderr);
                        }
                        if let Some(error) = &record.error {
                            ui.colored_label(palette(ui).error, error);
                        }
                    }
                });
        });
}
fn insert_example(a: &mut Attempt, s: &Scenario) -> Result<String, String> {
    work::context(a, s)?;
    let l = a
        .workspace
        .as_ref()
        .and_then(|w| w.one_off_learning.as_ref())
        .ok_or("Prepare separate example")?;
    let step = &s
        .one_off_teaching
        .as_ref()
        .ok_or("No separate example")?
        .example_actions[l.selected_example];
    let (action, message) = match &step.action {
        work::Action::Command { text } => (
            work::Action::CommandDraft { text: text.clone() },
            "Command inserted into the example only. Choose Run command to execute it.",
        ),
        work::Action::Run { end } => (
            work::Action::WindowDraft { text: end.text() },
            "Example time inserted. Choose Run through to advance its virtual clock.",
        ),
        work::Action::Save { path } => (
            work::Action::Select { path: path.clone() },
            "Example file selected. Review its draft and choose Save simulated file.",
        ),
        other => (
            other.clone(),
            "Example settings inserted only; no queue command or timed job ran.",
        ),
    };
    work::budget::apply_attempt(a, s, &action, true)?;
    Ok(message.into())
}
fn introduction(
    ui: &mut egui::Ui,
    s: &Scenario,
    a: &mut Attempt,
    p: &mut Presentation,
    m: &mut OneOffMetrics,
    active: bool,
) -> bool {
    let Some(l) = &s.one_off_teaching else {
        return false;
    };
    ui.heading(&s.title);
    ui.strong("Introduction");
    label(ui, m, &l.prerequisite_note);
    for c in &l.concepts {
        ui.strong(&c.title);
        label(ui, m, &c.text);
    }
    let mut target = None;
    ui.add_enabled_ui(active, |ui| {
        ui.horizontal_wrapped(|ui| {
            for (key, text, v, main) in [
                (
                    "try-example",
                    "Try the separate example",
                    View::Example,
                    true,
                ),
                ("assignment", "Start assignment", View::Assignment, false),
            ] {
                if action(ui, m, key, text, main) {
                    target = Some(v);
                }
            }
        })
    });
    if let Some(target) = target {
        let n = a
            .workspace
            .as_ref()
            .unwrap()
            .one_off_learning
            .as_ref()
            .unwrap()
            .selected_example;
        match work::budget::set_view(a, s, target, n) {
            Ok(()) => {
                p.owner = None;
                p.focus_document = true;
                return true;
            }
            Err(e) => p.notice = Some(e),
        }
    }
    false
}
fn page(
    ui: &mut egui::Ui,
    s: &Scenario,
    a: &mut Attempt,
    p: &mut Presentation,
    m: &mut Metrics,
    active: bool,
    pending: Option<egui::Id>,
) -> bool {
    if !p.memo.admitted(a, s) {
        label(
            ui,
            &mut m.one_off,
            "This saved activity does not match its source or exceeds a supported native limit. Existing work is retained; export a backup before choosing Reset.",
        );
        return false;
    }
    let frame = ui.ctx().cumulative_frame_nr_for(egui::ViewportId::ROOT);
    let blocked = p.blocked_gesture || p.veto_frame == Some(frame);
    let active = active && !blocked;
    let current = view(a);
    if current == View::Introduction {
        return introduction(ui, s, a, p, &mut m.one_off, active);
    }
    let practice = current == View::Example;
    let scope = egui::Id::new(("one-off", &s.id, s.version, a.visited, practice, p.epoch));
    let Some((_, session)) = selected(a, practice) else {
        return false;
    };
    p.bind(scope, session);
    ui.heading(&s.title);
    if practice {
        ui.strong("Separate example · Ungraded");
        label(
            ui,
            &mut m.one_off,
            &s.one_off_teaching.as_ref().unwrap().example_description,
        );
    } else {
        let r = ui.add(egui::Label::new(&s.objective).wrap());
        m.one_off.goal = Some(r.rect);
    }
    let edit_enabled = active && !p.running();
    let focused = ui.memory(|m| m.focused());
    let shortcut = edit_enabled
        && focused.is_some_and(|id| p.native.active.contains(&id))
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter));
    let terminal_enter = edit_enabled
        && focused == Some(scope.with("command"))
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
    let mut e = Edits {
        pending,
        ..Default::default()
    };
    if practice {
        let l = s.one_off_teaching.as_ref().unwrap();
        let old = a
            .workspace
            .as_ref()
            .unwrap()
            .one_off_learning
            .as_ref()
            .unwrap()
            .selected_example;
        let mut choice = old;
        ui.add_enabled_ui(edit_enabled, |ui| {
            controls::form_row(ui, "Example action", |ui, width| {
                let r = controls::dropdown(ui, scope.with("example"), width)
                    .selected_text(&l.example_actions[choice].title)
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        for (n, step) in l.example_actions.iter().enumerate() {
                            let r = ui.selectable_value(&mut choice, n, &step.title);
                            m.one_off.choices.push((format!("example:{n}"), r.rect));
                        }
                        m.one_off.popup =
                            Some(("example".into(), controls::popup_content_clip(ui)));
                    });
                m.one_off.controls.push((
                    "example".into(),
                    r.response.rect,
                    r.response.enabled(),
                    ui.clip_rect(),
                ));
            });
            label(ui, &mut m.one_off, &l.example_actions[choice].explanation);
            ui.horizontal_wrapped(|ui| {
                e.queue(
                    action(
                        ui,
                        &mut m.one_off,
                        "insert",
                        "Insert into example only",
                        false,
                    ),
                    Submit::Insert,
                );
                e.queue(
                    action(ui, &mut m.one_off, "reset-example", "Reset example", false),
                    Submit::ResetExample,
                );
            });
        });
        if choice != old {
            e.example_choice = Some(choice);
        }
    }
    let d = definition(s, practice).unwrap();
    let (simulation, session) = selected(a, practice).unwrap();
    let split = ui.available_width()
        > 2. * (ui.text_style_height(&egui::TextStyle::Body) * 24.
            + 2. * f32::from(controls::PAD_X))
            + controls::GAP;
    ui.add_enabled_ui(active, |ui| {
        if split {
            ui.columns(2, |uis| {
                uis[0].add_enabled_ui(edit_enabled, |ui| {
                    editing(
                        ui,
                        (d, simulation),
                        session,
                        p,
                        &mut m.one_off,
                        scope,
                        &mut e,
                    )
                });
                inspect(
                    &mut uis[1],
                    d,
                    simulation,
                    session,
                    p,
                    &mut m.one_off,
                    scope,
                );
            });
        } else {
            ui.add_enabled_ui(edit_enabled, |ui| {
                editing(
                    ui,
                    (d, simulation),
                    session,
                    p,
                    &mut m.one_off,
                    scope,
                    &mut e,
                )
            });
            ui.separator();
            inspect(ui, d, simulation, session, p, &mut m.one_off, scope);
        }
    });
    ui.add_enabled_ui(active, |ui| {
        ui.horizontal_wrapped(|ui| {
            e.queue(
                action_enabled(
                    ui,
                    &mut m.one_off,
                    "check",
                    "Check work",
                    true,
                    !p.running(),
                ),
                Submit::Check,
            );
            if p.running() || p.memo.pending(a, s) {
                e.queue(
                    action(ui, &mut m.one_off, "stop", "Stop", false),
                    Submit::Stop,
                );
            }
            if !a.challenge && s.one_off_teaching.is_some() {
                for (key, text, target) in [
                    ("introduction", "Introduction", View::Introduction),
                    ("assignment", "Assignment", View::Assignment),
                    ("practice", "Try example", View::Example),
                ] {
                    if target != current
                        && action_enabled(ui, &mut m.one_off, key, text, false, !p.running())
                    {
                        e.navigation = Some(target);
                    }
                }
            }
        })
    });
    label(
        ui,
        &mut m.one_off,
        "Stop cancels the unfinished virtual interval; none of its changes are applied. Check uses separate sample inputs; your draft and current files stay unchanged.",
    );
    if shortcut && e.submit.is_none() {
        e.submit = Some(Submit::Check);
    }
    if terminal_enter && e.submit.is_none() {
        e.submit = Some(Submit::Command);
    }
    let dropped = e.pending.is_some() && !e.received;
    if e.rejected || dropped {
        p.veto_frame = Some(frame);
        p.blocked_gesture |= ui.input(|i| i.pointer.primary_down());
        for native in e.native.drain(..) {
            native.restore(ui.ctx());
        }
        p.document = Document::default();
        p.checked[usize::from(practice)] = false;
        p.notice=Some(if dropped{"The text was not applied. Click the field, retry the edit, then choose the action again."}else{"That edit exceeds its supported limit. Previous fields are retained and nothing ran."}.into());
        return false;
    }
    // Keep a refused press blocked through its actual release and every layout
    // pass in the release's ROOT frame. A later valid gesture may retry.
    let gesture_blocked = blocked || p.veto_frame == Some(frame);
    if p.blocked_gesture && !ui.input(|i| i.pointer.primary_down()) {
        p.blocked_gesture = false;
        p.veto_frame = Some(frame);
    }
    if gesture_blocked {
        for native in e.native.drain(..) {
            native.restore(ui.ctx());
        }
        p.document = Document::default();
        return false;
    }
    let mut dirty = false;
    if e.changed || e.example_choice.is_some() || e.navigation.is_some() {
        let old = a
            .workspace
            .as_ref()
            .and_then(|w| w.one_off_learning.as_ref())
            .map_or(0, |l| l.selected_example);
        let packet = input::Packet {
            selected: Some(&p.selected),
            actor: Some(&p.actor),
            document: (!e.target.is_empty())
                .then_some((e.target.as_str(), p.document.value.as_str())),
            command: Some(&p.command),
            end: Some(&p.end),
            environment: Some(&p.environment),
            presentation: if e.example_choice.is_some() || e.navigation.is_some() {
                Some((
                    e.navigation.unwrap_or(current),
                    e.example_choice.unwrap_or(old),
                ))
            } else {
                None
            },
        };
        match input::commit(a, s, practice, packet) {
            Ok(()) => {
                dirty = true;
                p.notice = None;
                if e.changed {
                    p.memo.changed(&s.id);
                } else {
                    p.memo.invalidate(&s.id);
                }
                p.checked[usize::from(practice)] = false;
            }
            Err(error) => {
                p.veto_frame = Some(frame);
                p.blocked_gesture |= ui.input(|i| i.pointer.primary_down());
                for n in e.native.drain(..) {
                    n.restore(ui.ctx());
                }
                p.document = Document::default();
                p.notice = Some(error);
                return false;
            }
        }
    }
    if e.navigation.is_some() {
        p.owner = None;
        p.job = None;
        return dirty;
    }
    if let Some(submit) = e.submit {
        let result = match submit {
            Submit::Save => {
                work::budget::apply_attempt(a, s, &work::Action::Save { path: e.target }, practice)
            }
            Submit::Command => work::budget::apply_attempt(
                a,
                s,
                &work::Action::Command {
                    text: p.command.clone(),
                },
                practice,
            ),
            Submit::Run => {
                if !memo::run_admitted(a, s) {
                    Err("This Run exceeds the native working-memory or context limit; all work retained".into())
                } else {
                    match work::budget::PendingAttempt::from_draft(a, s, practice) {
                        Ok(pending) => {
                            p.job = Some(Job {
                                source: s.clone(),
                                input: a.clone(),
                                practice,
                                pending,
                            });
                            Ok(
                                "Running only in the simulator. Stop discards unfinished changes."
                                    .into(),
                            )
                        }
                        Err(e) => Err(e),
                    }
                }
            }
            Submit::Stop => {
                if let Some(mut job) = p.job.take() {
                    job.pending.cancel();
                }
                p.memo.pause(a, s);
                Ok("Stopped. Drafts and saved state are retained; unfinished simulated changes were discarded.".into())
            }
            Submit::Check => {
                p.checked[usize::from(practice)] = true;
                p.memo.evaluate(a, s, true);
                Ok(String::new())
            }
            Submit::Insert => insert_example(a, s),
            Submit::ResetExample => input::reset_example(a, s)
                .map(|()| "Separate example reset; assignment is unchanged".into()),
        };
        match result {
            Ok(message) => {
                p.notice = (!message.is_empty()).then_some(message);
                if matches!(submit, Submit::Command) {
                    p.command_executions = p.command_executions.saturating_add(1);
                }
                if matches!(submit, Submit::Run) {
                    p.checked[usize::from(practice)] = false;
                }
                if !matches!(submit, Submit::Run | Submit::Check | Submit::Stop) {
                    dirty = true;
                    p.memo.changed(&s.id);
                    p.checked[usize::from(practice)] = false;
                }
                if matches!(submit, Submit::ResetExample) {
                    p.owner = None;
                    p.epoch = p.epoch.wrapping_add(1);
                }
            }
            Err(e) => p.notice = Some(e),
        }
        ui.ctx().request_repaint();
    }
    if let Some(notice) = &p.notice {
        label(ui, &mut m.one_off, notice);
    }
    if p.memo.paused(a, s) && !p.checked[usize::from(practice)] {
        label(
            ui,
            &mut m.one_off,
            "Check stopped. Drafts and files are retained. Choose Check work to restart it.",
        );
    }
    if p.checked[usize::from(practice)] {
        let result = card(ui).show(ui, |ui| {
            if let Some(rows) = p.memo.rows(a, s, practice) {
                let pass = !rows.is_empty() && rows.iter().all(|r| r.0);
                ui.strong(if pass {
                    if practice {
                        "Example checks pass · Assignment remains separate"
                    } else {
                        "Assignment complete"
                    }
                } else {
                    "Some outcomes need attention"
                });
                let main_rows = 3 + d.requirements.len();
                // Explain the first actual failure before listing routine passes.
                // Keep its original row index; display order never grades work.
                let first = rows.iter().position(|r| !r.0);
                if let Some(n) = first {
                    let r = ui.add(
                        egui::Label::new(
                            RichText::new(format!("Needs work · {}", rows[n].1))
                                .color(palette(ui).error),
                        )
                        .wrap(),
                    );
                    m.one_off.rows.push((n, false, r.rect));
                }
                for (n, (ok, text)) in rows
                    .iter()
                    .take(main_rows)
                    .enumerate()
                    .filter(|(n, _)| Some(*n) != first)
                {
                    let r = ui.add(
                        egui::Label::new(
                            RichText::new(format!(
                                "{} · {text}",
                                if *ok { "Pass" } else { "Needs work" }
                            ))
                            .color(if *ok {
                                palette(ui).success
                            } else {
                                palette(ui).error
                            }),
                        )
                        .wrap(),
                    );
                    m.one_off.rows.push((n, *ok, r.rect));
                }
                controls::collapsing_header(ui, "Fresh sample results")
                    .id_salt(scope.with("sample-results"))
                    .show(ui, |ui| {
                        for (n, (ok, text)) in rows
                            .iter()
                            .enumerate()
                            .skip(main_rows)
                            .filter(|(n, _)| Some(*n) != first)
                        {
                            let r = ui.add(
                                egui::Label::new(
                                    RichText::new(format!(
                                        "{} · {text}",
                                        if *ok { "Pass" } else { "Needs work" }
                                    ))
                                    .color(if *ok {
                                        palette(ui).success
                                    } else {
                                        palette(ui).error
                                    }),
                                )
                                .wrap(),
                            );
                            m.one_off.rows.push((n, *ok, r.rect));
                        }
                    });
                m.completion = pass && !practice;
            } else if p.memo.paused(a, s) {
                ui.label("Check stopped. Choose Check work when ready.");
            } else {
                ui.label("Checking bounded replay and fresh sample inputs…");
                ui.ctx().request_repaint();
            }
        });
        m.one_off.result = Some(result.response.rect);
    }
    ui.add_enabled_ui(active,|ui|{controls::collapsing_header(ui,"Help and supported commands").id_salt(scope.with("help")).show(ui,|ui|{
        ui.label(core::HELP);ui.label("Job scripts use the disclosed portable printf, echo, cat, ordinary scalar expansion, quotes and redirection subset. Other syntax fails honestly before side effects. These queue commands do not run a host shell.");
        ui.label("To use a revised script, remove the old pending job and submit the saved file again. Changing the file or current variables does not edit an already queued copy.");
        ui.label("at -c shows variables as quoted simulator metadata, not shell export commands. Use the submission variables editor for those values.");
        ui.label("Stop cancels only the unfinished simulated interval. Real at cannot undo external work that has already finished. Check starts separate tests with their own sample inputs and reset simulated files; it leaves your draft and current files unchanged.");
        if practice{ui.label(&s.one_off_teaching.as_ref().unwrap().try_change);}else{
            for hint in s.hints.iter().take(a.hints){ui.label(hint);}
            if a.hints<s.hints.len()&&action_enabled(ui,&mut m.one_off,"hint","Show next hint",false,!p.running()){
                let size=dlms_learning_lab::cron_workspace::size(a,work::MAX_ATTEMPT);let next=a.hints+1;
                if size.and_then(|n|n.checked_sub(a.hints.to_string().len())).and_then(|n|n.checked_add(next.to_string().len())).is_some_and(|n|n<=work::MAX_ATTEMPT){a.hints=next;dirty=true;p.memo.invalidate(&s.id);}
            }
        }
        controls::collapsing_header(ui,"Worked explanation · On request").id_salt(scope.with("why")).show(ui,|ui|{ui.label(&s.explanation);});ui.label(&s.simplifications);for source in &s.sources{ui.hyperlink_to(source,source);}
    });});
    dirty
}
impl LabApp {
    pub(super) fn prepare_one_off(a: &mut Attempt, s: &Scenario) {
        if matches!(s.initial, engine::State::OneOff { .. }) {
            let _ = work::budget::prepare(a, s);
        }
    }
    pub(super) fn one_off_prepass(&mut self, ctx: &egui::Context) {
        let frame = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        if self.one_off_presentation.advanced == Some(frame) {
            return;
        }
        self.one_off_presentation.advanced = Some(frame);
        self.one_off_presentation.native.purge(ctx);
        self.one_off_presentation
            .memo
            .reconcile(&self.progress, &self.scenarios);
        let allowed = !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let selected_id = self
            .selected
            .and_then(|i| self.scenarios.get(i))
            .map(|s| s.id.as_str());
        if !allowed || selected_id.is_some() {
            self.one_off_presentation.memo.pause_away_from(if allowed {
                selected_id
            } else {
                None
            });
        }
        let target = if !allowed {
            None
        } else if let Some(i) = self.selected {
            matches!(self.scenarios[i].initial, engine::State::OneOff { .. }).then_some(i)
        } else {
            self.completion_index
                .pending(&self.progress, &self.scenarios)
                .filter(|i| matches!(self.scenarios[*i].initial, engine::State::OneOff { .. }))
                .filter(|_| self.completion_index.take_work())
        };
        if let Some(i) = target {
            let s = &self.scenarios[i];
            if let Some(a) = self.progress.attempts.get(&s.id) {
                self.one_off_presentation.memo.evaluate(a, s, false);
                if !self.one_off_presentation.memo.admitted(a, s) {
                    self.completion_index.refuse(&s.id);
                }
            }
        }
        let mut ran = false;
        if let Some(mut job) = self.one_off_presentation.job.take() {
            let same = self
                .selected
                .and_then(|i| self.scenarios.get(i))
                .is_some_and(|s| {
                    s.id == job.source.id
                        && self.progress.attempts.get(&s.id).is_some_and(|a| {
                            view(a)
                                == (if job.practice {
                                    View::Example
                                } else {
                                    View::Assignment
                                })
                                && memo::same_input(a, s, &job.input, &job.source)
                        })
                });
            if !same
                || self.reset_pending
                || self.settings_open
                || self.theme_preview
                || self.data_tools.modal_open()
            {
                job.pending.cancel();
                self.one_off_presentation.notice=Some("Run stopped because the workspace changed; unfinished simulated changes were discarded.".into());
            } else {
                ran = true;
                job.pending.tick(core::MAX_TICK / 2);
                if !job.pending.done() {
                    self.one_off_presentation.job = Some(job);
                    ctx.request_repaint();
                } else {
                    let a = self.progress.attempts.get_mut(&job.source.id).unwrap();
                    match job.pending.finish(a, &job.source) {
                        Ok(()) => {
                            self.dirty = true;
                            self.one_off_presentation.memo.changed(&job.source.id);
                            self.one_off_presentation.checked[usize::from(job.practice)] = false;
                            self.one_off_presentation.notice = Some(
                                "Virtual run finished. Inspect the actual output, then Check work."
                                    .into(),
                            );
                        }
                        Err(e) => self.one_off_presentation.notice = Some(e),
                    }
                }
            }
        }

        if allowed
            && self.one_off_presentation.memo.advance(if ran {
                core::MAX_TICK / 2
            } else {
                core::MAX_TICK
            })
        {
            ctx.request_repaint();
        }
        for s in &self.scenarios {
            if matches!(s.initial, engine::State::OneOff { .. })
                && let Some(a) = self.progress.attempts.get(&s.id)
            {
                let _ = self.one_off_presentation.memo.publish_current(a, s);
            }
        }
    }
    pub(super) fn one_off_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let pending = self.one_off_presentation.pending_text(ui.ctx());
        let frame = ui.ctx().cumulative_frame_nr_for(egui::ViewportId::ROOT);
        ui.add_enabled_ui(
            pending.is_none()
                && !self.one_off_presentation.blocked_gesture
                && self.one_off_presentation.veto_frame != Some(frame),
            |ui| self.workspace_header(ui, i),
        );
        let active = self.selected == Some(i)
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let s = &self.scenarios[i];
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        self.one_off_presentation.memo.evaluate(a, s, false);
        self.dirty |= card(ui)
            .show(ui, |ui| {
                page(
                    ui,
                    s,
                    a,
                    &mut self.one_off_presentation,
                    &mut self.metrics,
                    active,
                    pending,
                )
            })
            .inner;
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
    }
}
