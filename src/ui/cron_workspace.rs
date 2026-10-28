//! Native scheduling workspace: full assignment, real document editing,
//! explicit installation, bounded virtual Run/Stop and fresh behavior checks.
use super::*;
use dlms_learning_lab::{cron as core, cron_workspace as work, script_teaching::View};
pub(super) mod memo;
#[derive(Default)]
pub(super) struct CronMetrics {
    pub fields: Vec<(String, egui::Rect, egui::Id, bool, egui::Rect)>,
    pub controls: Vec<(String, egui::Rect, bool, egui::Rect)>,
    pub choices: Vec<(String, egui::Rect)>,
    pub popup: Option<(String, egui::Rect)>,
    pub documents: Vec<(egui::Rect, egui::Rect, egui::Vec2)>,
    pub rows: Vec<(usize, bool, egui::Rect)>,
    pub labels: Vec<(String, egui::Rect)>,
    pub output: Option<(egui::Rect, egui::Rect)>,
    pub result: Option<egui::Rect>,
}
struct Job {
    source: Scenario,
    input: Attempt,
    practice: bool,
    start: core::Moment,
    run: core::Run,
}
#[derive(Default)]
struct Document {
    key: Option<egui::Id>,
    value: String,
    reset_undo: bool,
}
const EDITOR_UNDOS: usize = 16;
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
            ctx.data_mut(|data| data.remove::<egui::text_edit::TextEditState>(id));
        }
    }
    fn track(&mut self, ctx: &egui::Context, id: egui::Id) {
        if !self.active.contains(&id) {
            if self.active.len() >= 18 {
                let old = self.active.remove(0);
                ctx.data_mut(|data| data.remove::<egui::text_edit::TextEditState>(old));
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
#[derive(Default)]
pub(super) struct Presentation {
    pub memo: memo::Memo,
    native: NativeStore,
    epoch: u64,
    owner: Option<egui::Id>,
    actor: String,
    selected: Option<work::Document>,
    end: String,
    synced_actor: String,
    synced_selected: Option<work::Document>,
    synced_end: String,
    document: Document,
    job: Option<Job>,
    ran_this_frame: bool,
    checked: [bool; 2],
    notice: Option<String>,
    file: String,
    focus_document: bool,
}
impl Presentation {
    pub(in crate::ui) fn running(&self) -> bool {
        self.job.is_some()
    }
    #[cfg(test)]
    pub(in crate::ui) fn observed_checked(&self, practice: bool) -> bool {
        self.checked[usize::from(practice)]
    }
    pub(in crate::ui) fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }
    pub fn clear(&mut self) {
        self.native.clear();
        self.epoch = self.epoch.wrapping_add(1);
        self.owner = None;
        self.actor.clear();
        self.selected = None;
        self.end.clear();
        self.synced_actor.clear();
        self.synced_selected = None;
        self.synced_end.clear();
        self.document = Document::default();
        self.job = None;
        self.ran_this_frame = false;
        self.checked = [false; 2];
        self.notice = None;
        self.file.clear();
        self.focus_document = false;
        self.memo.clear();
    }
    fn bind(&mut self, scope: egui::Id, session: &work::Session) {
        if self.owner != Some(scope) {
            self.owner = Some(scope);
            self.document = Document::default();
            self.checked = [false; 2];
            self.notice = None;
            self.file.clear();
        }
        if self.actor.is_empty()
            || self.synced_actor != session.actor
            || self.synced_selected.as_ref() != Some(&session.selected)
            || self.synced_end != session.window_end
        {
            self.actor = session.actor.clone();
            self.selected = Some(session.selected.clone());
            self.end = session.window_end.clone();
            self.synced_actor = session.actor.clone();
            self.synced_selected = Some(session.selected.clone());
            self.synced_end = session.window_end.clone();
        }
    }
    fn rollback(&mut self, session: &work::Session) {
        self.actor = session.actor.clone();
        self.selected = Some(session.selected.clone());
        self.end = session.window_end.clone();
        self.synced_actor = session.actor.clone();
        self.synced_selected = Some(session.selected.clone());
        self.synced_end = session.window_end.clone();
        self.document = Document::default();
    }
}
fn selected(a: &Attempt, practice: bool) -> Option<(&core::State, &work::Session)> {
    let w = a.workspace.as_ref()?;
    if practice {
        let l = w.cron_learning.as_ref()?;
        Some((&l.simulation, &l.session))
    } else {
        let engine::State::Cron { simulation } = &a.state else {
            return None;
        };
        Some((simulation, w.cron_session.as_ref()?))
    }
}
fn view(a: &Attempt) -> View {
    if a.challenge {
        View::Assignment
    } else {
        a.workspace
            .as_ref()
            .and_then(|w| w.cron_learning.as_ref())
            .map_or(View::Assignment, |l| l.view)
    }
}
fn definition(s: &Scenario, practice: bool) -> Option<&work::Definition> {
    if practice {
        Some(&s.cron_teaching.as_ref()?.definition)
    } else {
        let engine::Goal::Cron { definition } = &s.goal else {
            return None;
        };
        Some(definition)
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
fn action(ui: &mut egui::Ui, m: &mut CronMetrics, key: &str, text: &str, main: bool) -> bool {
    action_enabled(ui, m, key, text, main, true)
}
fn action_enabled(
    ui: &mut egui::Ui,
    m: &mut CronMetrics,
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

fn label(ui: &mut egui::Ui, m: &mut CronMetrics, text: impl Into<String>) {
    let text = text.into();
    let r = ui.add(egui::Label::new(&text).wrap());
    m.labels.push((text, r.rect));
}
fn editor(
    ui: &mut egui::Ui,
    m: &mut CronMetrics,
    scope: egui::Id,
    target: &work::Document,
    current: &str,
    doc: &mut Document,
    focus: &mut bool,
) -> Edit {
    let id = scope.with(("document", target.key()));
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
        cap: core::MAX_TABLE_BYTES,
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
            "Scheduling document editor",
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
fn end_field(ui: &mut egui::Ui, m: &mut CronMetrics, scope: egui::Id, end: &mut String) -> Edit {
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
    Install,
    Remove,
    Run,
    Stop,
    Check,
    Fill,
    ResetExample,
    RestoreSamples,
}
#[derive(Default)]
struct Edits {
    changed: bool,
    rejected: bool,
    received: bool,
    pending: Option<egui::Id>,
    submit: Option<Submit>,
    target: Option<work::Document>,
    example_choice: Option<usize>,
    navigation: Option<View>,
    native: Vec<NativeBefore>,
}
impl Edits {
    fn observe(&mut self, mut e: Edit) {
        if let Some(native) = e.native.take() {
            self.native.push(native);
        }
        self.changed |= e.changed;
        self.rejected |= e.rejected;
        if self.pending.is_some() && self.pending == e.id {
            self.received |= e.received;
        }
    }
    fn queue(&mut self, clicked: bool, value: Submit) {
        if clicked && self.submit.is_none() {
            self.submit = Some(value);
        }
    }
}
struct Live<'a> {
    state: &'a core::State,
    session: &'a work::Session,
}
fn edit_workspace(
    ui: &mut egui::Ui,
    d: &work::Definition,
    live: Live<'_>,
    p: &mut Presentation,
    m: &mut CronMetrics,
    scope: egui::Id,
    e: &mut Edits,
) {
    let Live { state, session } = live;
    controls::form_row(ui, "Simulated account", |ui, width| {
        let mut value = p.actor.clone();
        let r = controls::dropdown(ui, scope.with("actor"), width)
            .selected_text(&value)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for name in &d.initial.users {
                    let r = ui.selectable_value(&mut value, name.clone(), name);
                    m.choices.push((format!("actor:{name}"), r.rect));
                }
                m.popup = Some(("actor".into(), controls::popup_content_clip(ui)));
            });
        if value != p.actor {
            p.actor = value;
            e.changed = true;
        }
        m.controls.push((
            "actor".into(),
            r.response.rect,
            r.response.enabled(),
            ui.clip_rect(),
        ));
    });
    controls::form_row(ui, "Document", |ui, width| {
        let before = p.selected.as_ref().unwrap().clone();
        let mut value = before.clone();
        let r = controls::dropdown(ui, scope.with("target"), width)
            .selected_text(value.label())
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for target in &d.editable {
                    let r = ui.selectable_value(&mut value, target.clone(), target.label());
                    m.choices.push((format!("target:{}", target.key()), r.rect));
                }
                m.popup = Some(("target".into(), controls::popup_content_clip(ui)));
            });
        if value != before {
            p.selected = Some(value);
            e.changed = true;
        }
        m.controls.push((
            "target".into(),
            r.response.rect,
            r.response.enabled(),
            ui.clip_rect(),
        ));
    });
    let target = p.selected.as_ref().unwrap().clone();
    let current = session
        .drafts
        .get(&target.key())
        .map(String::as_str)
        .unwrap_or("");
    p.native
        .track(ui.ctx(), scope.with(("document", target.key())));
    e.observe(editor(
        ui,
        m,
        scope,
        &target,
        current,
        &mut p.document,
        &mut p.focus_document,
    ));
    e.target = Some(target.clone());
    let applied = session.installed_matches(d, state);
    label(
        ui,
        m,
        if applied {
            "Installed documents match these edits. Run uses the installed schedule."
        } else {
            "Pending edits · Install each changed document before running."
        },
    );
    ui.horizontal_wrapped(|ui| {
        e.queue(
            action(ui, m, "install", "Install document", !applied),
            Submit::Install,
        );
        e.queue(
            action_enabled(
                ui,
                m,
                "remove",
                "Remove installed document",
                false,
                target.exists(state),
            ),
            Submit::Remove,
        );
    });
    if let Some(notice) = p.notice() {
        label(ui, m, notice);
    }
    label(ui, m, format!("Virtual clock: {} UTC", state.clock.text()));
    p.native.track(ui.ctx(), scope.with("end"));
    e.observe(end_field(ui, m, scope, &mut p.end));
    ui.horizontal_wrapped(|ui| {
        e.queue(
            action(ui, m, "run", "Run", applied && session.last_run.is_none()),
            Submit::Run,
        );
    });
}
fn output(
    ui: &mut egui::Ui,
    d: &work::Definition,
    live: Live<'_>,
    p: &mut Presentation,
    m: &mut CronMetrics,
    scope: egui::Id,
) {
    let Live { state, session } = live;
    ui.strong("Output and virtual files");
    controls::form_row(ui, "View file", |ui, width| {
        if !state.files.contains_key(&p.file) {
            p.file = d
                .outputs
                .iter()
                .find(|path| state.files.contains_key(*path))
                .cloned()
                .or_else(|| {
                    state
                        .files
                        .keys()
                        .find(|path| path.contains("source"))
                        .cloned()
                })
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
    let h =
        ui.text_style_height(&egui::TextStyle::Monospace) * 7. + 2. * f32::from(controls::PAD_Y);
    let text = state
        .files
        .get(&p.file)
        .map(String::as_str)
        .unwrap_or("Run the installed schedule to create its output file.");
    let shown = egui::ScrollArea::both()
        .id_salt(scope.with("file-output"))
        .max_height(h)
        .min_scrolled_height(h)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(RichText::new(text).monospace())
                    .wrap()
                    .selectable(true),
            )
        });
    m.output = Some((shown.inner.rect, shown.inner_rect));
    if let Some(log) = &session.last_run {
        controls::collapsing_header(ui, "Last simulated run")
            .id_salt(scope.with("run-output"))
            .default_open(true)
            .show(ui, |ui| {
                label(
                    ui,
                    m,
                    format!(
                        "{} job starts · {}–{} UTC",
                        log.events.len(),
                        log.start.text(),
                        log.end.text()
                    ),
                );
                egui::ScrollArea::vertical()
                    .id_salt(scope.with("job-lines"))
                    .max_height(h)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for event in &log.events {
                            label(
                                ui,
                                m,
                                format!(
                                    "{} · {} · Exit {}",
                                    event.when.text(),
                                    event.owner,
                                    event.status
                                ),
                            );
                            if !event.stdout.is_empty() {
                                ui.monospace(&event.stdout);
                            }
                            if !event.stderr.is_empty() {
                                ui.colored_label(palette(ui).error, &event.stderr);
                            }
                        }
                        if log.events.is_empty() {
                            ui.label("No job matched this interval.");
                        }
                    });
            });
    } else {
        ui.label("Run shows each job’s actual owner, output and exit status (0 means success).");
    }
}

fn introduction(
    ui: &mut egui::Ui,
    s: &Scenario,
    a: &mut Attempt,
    p: &mut Presentation,
    m: &mut CronMetrics,
    active: bool,
) -> bool {
    let Some(lesson) = &s.cron_teaching else {
        return false;
    };
    let mut dirty = false;
    label(ui, m, &s.title);
    ui.strong("Introduction");
    label(ui, m, &lesson.prerequisite_note);
    for concept in &lesson.concepts {
        ui.strong(&concept.title);
        label(ui, m, &concept.text);
    }
    ui.add_enabled_ui(active, |ui| {
        ui.horizontal_wrapped(|ui| {
            for (key, text, view, main) in [
                (
                    "try-example",
                    "Try the separate example",
                    View::Example,
                    true,
                ),
                ("assignment", "Start assignment", View::Assignment, false),
            ] {
                if action(ui, m, key, text, main) {
                    let selected = a
                        .workspace
                        .as_ref()
                        .unwrap()
                        .cron_learning
                        .as_ref()
                        .unwrap()
                        .selected_example;
                    match work::budget::set_presentation(a, s, view, selected) {
                        Ok(()) => {
                            dirty = true;
                            p.owner = None;
                            p.focus_document = view == View::Assignment;
                        }
                        Err(error) => p.notice = Some(error),
                    }
                }
            }
        })
    });
    dirty
}
fn page(
    ui: &mut egui::Ui,
    s: &Scenario,
    a: &mut Attempt,
    p: &mut Presentation,
    m: &mut Metrics,
    active: bool,
) -> bool {
    if !p.memo.admitted(a, s) {
        ui.colored_label(palette(ui).error,"This saved work does not match the lab or exceeds its supported limits. Existing work is retained; export a backup before choosing a reset.");
        return false;
    }
    let mut dirty = false;
    let current = view(a);
    if current == View::Introduction {
        return introduction(ui, s, a, p, &mut m.cron, active);
    }
    let practice = current == View::Example;
    let scope = egui::Id::new(("cron", &s.id, s.version, a.visited, practice, p.epoch));
    let Some((_, session)) = selected(a, practice) else {
        ui.label("The saved-work limit prevented preparation. Existing work is retained.");
        return false;
    };
    p.bind(scope, session);
    let r = ui.add(egui::Label::new(RichText::new(&s.title).strong().size(22.)).wrap());
    m.cron.labels.push((s.title.clone(), r.rect));
    if practice {
        ui.strong("Separate example · Ungraded");
        label(
            ui,
            &mut m.cron,
            &s.cron_teaching.as_ref().unwrap().example_description,
        );
    } else {
        label(ui, &mut m.cron, &s.objective);
    }
    // Capture pending input and focus separately before widgets. This observes
    // actual delivery, even when Paste replaces a selection with identical text.
    // It never calls another Context API from inside an input/memory closure.
    let edit_enabled = active && !p.running();
    let focused = ui.memory(|memory| memory.focused());
    let has_text = ui.input(|input| {
        input.events.iter().any(|event| match event {
            egui::Event::Paste(text) => !text.is_empty(),
            egui::Event::Text(text) => !text.is_empty() && text != "\n" && text != "\r",
            _ => false,
        })
    });
    let doc_id = scope.with(("document", session.selected.key()));
    let end_id = scope.with("end");
    let pending = if edit_enabled && has_text {
        focused.filter(|id| *id == doc_id || *id == end_id)
    } else {
        None
    };
    let shortcut = edit_enabled
        && focused.is_some_and(|id| id == doc_id || id == end_id)
        && ui.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::Enter));
    let mut edits = Edits {
        pending,
        ..Edits::default()
    };
    if practice {
        let lesson = s.cron_teaching.as_ref().unwrap();
        let old = a
            .workspace
            .as_ref()
            .unwrap()
            .cron_learning
            .as_ref()
            .unwrap()
            .selected_example;
        let mut choice = old;
        ui.add_enabled_ui(edit_enabled, |ui| {
            controls::form_row(ui, "Example action", |ui, width| {
                let r = controls::dropdown(ui, scope.with("example"), width)
                    .selected_text(&lesson.example_actions[old].title)
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        for (n, step) in lesson.example_actions.iter().enumerate() {
                            let r = ui.selectable_value(&mut choice, n, &step.title);
                            m.cron.choices.push((format!("example:{n}"), r.rect));
                        }
                        m.cron.popup = Some(("example".into(), controls::popup_content_clip(ui)));
                    });
                m.cron.controls.push((
                    "example".into(),
                    r.response.rect,
                    r.response.enabled(),
                    ui.clip_rect(),
                ));
            });
            label(ui, &mut m.cron, &lesson.example_actions[choice].explanation);
            ui.horizontal_wrapped(|ui| {
                edits.queue(
                    action(ui, &mut m.cron, "fill", "Fill example only", false),
                    Submit::Fill,
                );
                edits.queue(
                    action(ui, &mut m.cron, "reset-example", "Reset example", false),
                    Submit::ResetExample,
                );
            });
        });
        if choice != old {
            edits.example_choice = Some(choice);
        }
    }
    // Rendering reads actual current state only. No model/grade is installed by
    // this layout, and both modes retain the complete same practical assignment.
    let d = definition(s, practice).unwrap();
    let Some((state, session)) = selected(a, practice) else {
        return dirty;
    };
    let split = ui.available_width()
        > 2. * (ui.text_style_height(&egui::TextStyle::Body) * 24.
            + 2. * f32::from(controls::PAD_X))
            + controls::GAP;
    ui.add_enabled_ui(active, |ui| {
        if split {
            ui.columns(2, |uis| {
                uis[0].add_enabled_ui(edit_enabled, |ui| {
                    edit_workspace(
                        ui,
                        d,
                        Live { state, session },
                        p,
                        &mut m.cron,
                        scope,
                        &mut edits,
                    )
                });
                output(
                    &mut uis[1],
                    d,
                    Live { state, session },
                    p,
                    &mut m.cron,
                    scope,
                );
            });
        } else {
            ui.add_enabled_ui(edit_enabled, |ui| {
                edit_workspace(
                    ui,
                    d,
                    Live { state, session },
                    p,
                    &mut m.cron,
                    scope,
                    &mut edits,
                )
            });
            ui.separator();
            output(ui, d, Live { state, session }, p, &mut m.cron, scope);
        }
    });
    ui.add_enabled_ui(active, |ui| {
        ui.horizontal_wrapped(|ui| {
            edits.queue(
                action(ui, &mut m.cron, "check", "Check work", true),
                Submit::Check,
            );
            if p.job.is_some() || p.memo.pending(a, s) {
                edits.queue(action(ui, &mut m.cron, "stop", "Stop", false), Submit::Stop);
            }
            if !a.challenge && s.cron_teaching.is_some() {
                for (key, text, target) in [
                    ("introduction", "Introduction", View::Introduction),
                    ("assignment", "Assignment", View::Assignment),
                    ("practice", "Try example", View::Example),
                ] {
                    if target != current && action(ui, &mut m.cron, key, text, false) {
                        edits.navigation = Some(target);
                    }
                }
            }
        })
    });
    ui.label("Each check starts with its own sample dates and reset simulated files. Your draft and current workspace stay unchanged.");
    let retry = ui.add_enabled_ui(active,|ui| { controls::collapsing_header(ui, "Retry a run")
        .id_salt(scope.with("retry"))
        .show(ui, |ui| {
            ui.label("Restore only this assignment/example’s supplied files and starting virtual clock. Generated output and this activity’s run history are cleared; installed schedules, access policies and your drafts stay saved. This is a simulator helper, not a Linux command.");
            ui.add_enabled_ui(active, |ui| {
                edits.queue(
                    action(ui, &mut m.cron, "restore-samples", "Restore sample files and clock", false),
                    Submit::RestoreSamples,
                )
            });
        }) }).inner;
    m.cron.controls.push((
        "retry".into(),
        retry.header_response.rect,
        retry.header_response.enabled(),
        ui.clip_rect(),
    ));
    if shortcut && edits.submit.is_none() {
        edits.submit = Some(Submit::Check);
    }
    let dropped = edits.pending.is_some() && !edits.received;
    if edits.rejected || dropped {
        // Refuse the complete edit/action packet, never just its last field.
        if let Some((_, session)) = selected(a, practice) {
            p.rollback(session)
        }
        p.checked[usize::from(practice)] = false;
        for saved in edits.native.drain(..) {
            saved.restore(ui.ctx());
        }
        p.notice=Some(if dropped{"The paste or text was not applied. Click the intended field, retry the edit, then choose the action again."}else{"That edit exceeds the supported field limit. Previous fields are retained and nothing ran; shorten the edit and retry."}.into());
        return dirty;
    }
    if edits.changed || edits.example_choice.is_some() || edits.navigation.is_some() {
        let edit = work::budget::SessionEdit {
            selected: p.selected.as_ref(),
            actor: Some(&p.actor),
            draft: edits
                .target
                .as_ref()
                .map(|target| (target, p.document.value.as_str())),
            window_end: Some(&p.end),
        };
        let old = a
            .workspace
            .as_ref()
            .and_then(|w| w.cron_learning.as_ref())
            .map_or(0, |l| l.selected_example);
        let presentation = if edits.example_choice.is_some() || edits.navigation.is_some() {
            Some((
                edits.navigation.unwrap_or(current),
                edits.example_choice.unwrap_or(old),
            ))
        } else {
            None
        };
        match work::budget::edit_batch(a, s, edit, practice, presentation) {
            Ok(()) => {
                dirty = true;
                p.checked[usize::from(practice)] = false;
                p.memo.invalidate(&s.id);
                p.synced_actor = p.actor.clone();
                p.synced_selected = p.selected.clone();
                p.synced_end = p.end.clone();
            }
            Err(e) => {
                if let Some((_, session)) = selected(a, practice) {
                    p.rollback(session)
                }
                p.notice = Some(e);
                p.checked[usize::from(practice)] = false;
                for saved in edits.native.drain(..) {
                    saved.restore(ui.ctx());
                }
                return dirty;
            }
        }
    }
    if edits.navigation.is_some() {
        p.job = None;
        p.owner = None;
        return dirty;
    }
    if let Some(submit) = edits.submit {
        let result = match submit {
            Submit::Fill => work::budget::fill_example(a, s),
            Submit::ResetExample => work::budget::reset_example(a, s),
            Submit::RestoreSamples => {
                if let Some(mut job) = p.job.take() {
                    job.run.cancel();
                }
                p.memo.invalidate(&s.id);
                let result = work::budget::restore_samples(a, s, practice);
                if result.is_ok() {
                    p.notice=Some("Sample files and virtual clock restored. Your installed schedules, policies and drafts are unchanged.".into());
                }
                result
            }
            Submit::Install | Submit::Remove => {
                let target = p.selected.as_ref().unwrap().clone();
                let action = if matches!(submit, Submit::Install) {
                    work::Action::Install { target }
                } else {
                    work::Action::Remove { target }
                };
                work::budget::perform_attempt(a, s, &action, practice)
            }
            Submit::Run => {
                if p.job.is_some() {
                    Err("Stop the current run before starting another".into())
                } else if !memo::run_admitted(a, s) {
                    Err("This run exceeds the native working-memory or supplied-context limit; existing work is retained".into())
                } else {
                    let (state, session) = selected(a, practice).unwrap();
                    let d = definition(s, practice).unwrap();
                    if !session.applied(d, state) {
                        Err("Install or discard pending edits before Run. A draft is not the installed schedule.".into())
                    } else {
                        match core::Moment::parse(&session.window_end)
                            .and_then(|end| core::Run::begin(state, end))
                        {
                            Ok(run) => {
                                p.checked[usize::from(practice)] = false;
                                p.job = Some(Job {
                                    source: s.clone(),
                                    input: a.clone(),
                                    practice,
                                    start: state.clock,
                                    run,
                                });
                                Ok(())
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    }
                }
            }
            Submit::Stop => {
                if let Some(mut job) = p.job.take() {
                    job.run.cancel();
                }
                p.memo.evaluate(a, s, false);
                p.memo.pause(a, s);
                p.notice=Some("Stopped. Unfinished simulated changes were discarded; drafts and saved files remain.".into());
                Ok(())
            }
            Submit::Check => {
                p.checked[usize::from(practice)] = true;
                p.memo.evaluate(a, s, true);
                Ok(())
            }
        };
        match result {
            Ok(()) => {
                if !matches!(submit, Submit::Run | Submit::Check | Submit::Stop) {
                    dirty = true;
                    p.checked[usize::from(practice)] = false;
                    p.memo.invalidate(&s.id);
                    if matches!(submit, Submit::ResetExample) {
                        p.owner = None;
                        p.epoch = p.epoch.wrapping_add(1);
                    }
                    if matches!(submit, Submit::RestoreSamples) {
                        p.memo.evaluate(a, s, false);
                        p.memo.pause(a, s);
                    }
                }
                if !matches!(submit, Submit::Stop | Submit::RestoreSamples) {
                    p.notice = None;
                }
            }
            Err(e) => p.notice = Some(e),
        }
    }
    if edits.submit.is_some() {
        ui.ctx().request_repaint();
    }
    if p.job.is_some() {
        ui.label("Running only inside the simulator · Stop discards unfinished changes.");
        ui.ctx().request_repaint();
    }
    if p.checked[usize::from(practice)] {
        let result = card(ui).show(ui, |ui| {
            if let Some(rows) = p.memo.rows(a, s, practice) {
                let pass = !rows.is_empty() && rows.iter().all(|r| r.0);
                ui.strong(if pass {
                    if practice {
                        "Example checks pass · Assignment is separate"
                    } else {
                        "Assignment complete"
                    }
                } else {
                    "Some outcomes need attention"
                });
                for (n, (ok, text)) in rows.iter().take(1 + d.requirements.len() + 1).enumerate() {
                    let r = ui.colored_label(
                        if *ok {
                            palette(ui).success
                        } else {
                            palette(ui).error
                        },
                        format!("{} · {text}", if *ok { "Pass" } else { "Needs work" }),
                    );
                    m.cron.rows.push((n, *ok, r.rect));
                }
                let first_fresh_failure = rows
                    .iter()
                    .enumerate()
                    .skip(2 + d.requirements.len())
                    .find(|(_, r)| !r.0)
                    .map(|(n, _)| n);
                if let Some(n) = first_fresh_failure {
                    let r = ui.add(
                        egui::Label::new(
                            RichText::new(format!("Needs work · {}", rows[n].1))
                                .color(palette(ui).error),
                        )
                        .wrap(),
                    );
                    m.cron.rows.push((n, false, r.rect));
                }
                let samples = ui
                    .add_enabled_ui(active, |ui| {
                        controls::collapsing_header(ui, "Fresh sample results")
                            .id_salt(scope.with("case-results"))
                            .show(ui, |ui| {
                                for (n, (ok, text)) in rows
                                    .iter()
                                    .enumerate()
                                    .skip(2 + d.requirements.len())
                                    .filter(|(n, _)| Some(*n) != first_fresh_failure)
                                {
                                    let r =
                                        ui.add(
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
                                    m.cron.rows.push((n, *ok, r.rect));
                                }
                            })
                    })
                    .inner;
                m.cron.controls.push((
                    "cases".into(),
                    samples.header_response.rect,
                    samples.header_response.enabled(),
                    ui.clip_rect(),
                ));
                m.completion = pass && !practice;
            } else if p.memo.paused(a, s) {
                ui.label("Check stopped. Choose Check work when ready.");
            } else {
                ui.label("Checking fresh sample inputs and quiet dates…");
                ui.ctx().request_repaint();
            }
        });
        m.cron.result = Some(result.response.rect);
    }
    ui.add_enabled_ui(active, |ui| {
        controls::collapsing_header(ui, "Help and limits")
            .id_salt(scope.with("help"))
            .show(ui, |ui| {
                if practice {
                    ui.label(&s.cron_teaching.as_ref().unwrap().try_change);
                } else {
                    for text in s.hints.iter().take(a.hints) {
                        ui.label(text);
                    }
                    ui.add_enabled_ui(active && p.job.is_none() && a.hints < s.hints.len(), |ui| {
                        if action(ui, &mut m.cron, "hint", "Show next hint", false) {
                            a.hints += 1;
                            dirty = true;
                            p.memo.invalidate(&s.id);
                        }
                    });
                }
                controls::collapsing_header(ui, "Worked explanation")
                    .id_salt(scope.with("why"))
                    .show(ui, |ui| {
                        ui.label(&s.explanation);
                    });
                ui.label(&s.simplifications);
                for source in &s.sources {
                    ui.hyperlink_to(source, source);
                }
            });
    });
    dirty
}

impl LabApp {
    pub(super) fn prepare_cron(a: &mut Attempt, s: &Scenario) {
        if matches!(s.initial, engine::State::Cron { .. }) {
            let _ = work::budget::prepare(a, s);
        }
    }
    pub(super) fn cron_prepass(&mut self, ctx: &egui::Context) {
        self.cron_presentation.native.purge(ctx);
        self.cron_presentation.memo.retain(|id| {
            self.progress.attempts.contains_key(id)
                && self
                    .scenarios
                    .iter()
                    .any(|s| s.id == id && matches!(s.initial, engine::State::Cron { .. }))
        });
        for s in &self.scenarios {
            if matches!(s.initial, engine::State::Cron { .. })
                && let Some(a) = self.progress.attempts.get(&s.id)
            {
                self.cron_presentation.memo.evaluate(a, s, false);
            }
        }
        let budget = if self.cron_presentation.ran_this_frame {
            core::MAX_TICK / 2
        } else {
            core::MAX_TICK
        };
        if self.cron_presentation.memo.advance(budget) {
            ctx.request_repaint();
        }
    }
    pub(super) fn advance_cron(&mut self, ctx: &egui::Context) {
        self.cron_presentation.ran_this_frame = false;
        let Some(mut job) = self.cron_presentation.job.take() else {
            return;
        };
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
            job.run.cancel();
            self.cron_presentation.notice=Some("Run stopped because the workspace changed. Unfinished simulated changes were discarded.".into());
            return;
        }
        self.cron_presentation.ran_this_frame = true;
        job.run.tick(core::MAX_TICK / 2);
        if !job.run.done() {
            self.cron_presentation.job = Some(job);
            ctx.request_repaint();
            return;
        }
        let outcome = job.run.take_result().unwrap();
        match outcome {
            Err(e) => self.cron_presentation.notice = Some(e.to_string()),
            Ok(result) => {
                let id = &job.source.id;
                let a = self.progress.attempts.get_mut(id).unwrap();
                let count = result.records.len();
                match work::budget::publish_completed(
                    a,
                    &job.source,
                    job.start,
                    result,
                    job.practice,
                ) {
                    Err(e) => self.cron_presentation.notice = Some(e),
                    Ok(()) => {
                        self.dirty = true;
                        self.cron_presentation.memo.invalidate(id);
                        self.cron_presentation.checked[usize::from(job.practice)] = false;
                        self.cron_presentation.notice = Some(format!(
                            "Run finished · {count} job starts. Inspect the actual output, then Check work."
                        ));
                        let d = definition(&job.source, job.practice).unwrap();
                        if let Some((state, _)) = selected(a, job.practice) {
                            self.cron_presentation.file = d
                                .outputs
                                .iter()
                                .find(|p| state.files.contains_key(*p))
                                .cloned()
                                .unwrap_or_default();
                        }
                    }
                }
            }
        }
    }
    pub(super) fn cron_page(&mut self, ui: &mut egui::Ui, i: usize) {
        self.workspace_header(ui, i);
        if let Some(notice) = &self.notice {
            ui.label(notice);
        }
        let active = self.selected == Some(i)
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let source = &self.scenarios[i];
        let a = self.progress.attempts.get_mut(&source.id).unwrap();
        self.cron_presentation.memo.evaluate(a, source, false);
        self.dirty |= card(ui)
            .show(ui, |ui| {
                page(
                    ui,
                    source,
                    a,
                    &mut self.cron_presentation,
                    &mut self.metrics,
                    active,
                )
            })
            .inner;
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
    }
}

/// Borrow only the memo fields while library cards mutate independent favorites
/// and navigation. Cron never falls back to synchronous fresh-case grading.
pub(super) fn solved_with_memos(
    cron: &memo::Memo,
    ether: &etherchannel_workspace::memo::Memo,
    nfs: &nfs_workspace::memo::Memo,
    rpm: &rpm_packages::memo::Memo,
    nat: &nat_workspace::memo::Memo,
    a: &Attempt,
    s: &Scenario,
) -> bool {
    if matches!(s.initial, engine::State::Cron { .. }) {
        cron.prepared(&s.id).unwrap_or(false)
    } else {
        etherchannel_workspace::solved_with_memos(ether, nfs, rpm, nat, a, s)
    }
}
