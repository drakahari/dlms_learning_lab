//! Native, in-process static serving. Display snapshots are never grading authority.
use super::*;
use dlms_learning_lab::{
    apache_static::{self as web, Action, Method, Outcome},
    apache_static_teaching::{Learning, Lesson},
    apache_static_workspace::{self as work, Draft, EditorKind},
};

#[derive(Default)]
pub(super) struct Presentation {
    epoch: u64,
    key: Option<(String, bool, u64)>,
    rows: Option<Vec<(bool, String)>>,
    request_visible: bool,
}
impl Presentation {
    pub(super) fn clear(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.key = None;
        self.rows = None;
        self.request_visible = false;
    }
    fn bind(&mut self, id: &str, practice: bool) {
        if self
            .key
            .as_ref()
            .is_some_and(|k| k.0 == id && k.1 == practice && k.2 == self.epoch)
        {
            return;
        }
        self.key = Some((id.into(), practice, self.epoch));
        self.rows = None;
        self.request_visible = false;
    }
}
/// Snapshot lazily on actual edits, not on unchanged frames. Rejected paste,
/// including replacement of a selection, restores the complete previous string.
struct InputBuffer<'a> {
    value: &'a mut String,
    limit: usize,
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
                changed: false,
                rejected: true,
                previous: None,
            }
        } else {
            let changed = self
                .before
                .as_ref()
                .is_some_and(|s| s.as_str() != self.value.as_str());
            Edit {
                changed,
                rejected: false,
                previous: self.before,
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
        if text.is_empty() {
            return 0;
        }
        if self
            .value
            .len()
            .checked_add(text.len())
            .is_none_or(|n| n > self.limit)
        {
            self.rejected = true;
            return 0;
        }
        self.remember();
        egui::TextBuffer::insert_text(self.value, text, index)
    }
    fn delete_char_range(&mut self, range: std::ops::Range<usize>) {
        if range.is_empty() {
            return;
        }
        self.remember();
        egui::TextBuffer::delete_char_range(self.value, range);
    }
}
struct Edit {
    changed: bool,
    rejected: bool,
    previous: Option<String>,
}
#[derive(Clone, Copy)]
enum Field {
    Config,
    File,
    Port,
    Host(usize),
    Path,
}
impl Field {
    fn value(self, d: &mut Draft) -> &mut String {
        match self {
            Self::Config => &mut d.config_text,
            Self::File => &mut d.file_text,
            Self::Port => &mut d.request_port,
            Self::Host(n) => &mut d.request_hosts[n],
            Self::Path => &mut d.request_path,
        }
    }
    fn key(self) -> String {
        match self {
            Self::Config => "config".into(),
            Self::File => "file".into(),
            Self::Port => "port".into(),
            Self::Host(n) => format!("host-{n}"),
            Self::Path => "path".into(),
        }
    }
    fn limit(self) -> usize {
        match self {
            Self::Config => web::MAX_CONFIG,
            Self::File => web::MAX_FILE,
            Self::Port => 8,
            Self::Host(_) => 260,
            Self::Path => 256,
        }
    }
    fn changes_configuration(self) -> bool {
        matches!(self, Self::Config | Self::File)
    }
}
enum Session<'a> {
    Practice {
        learning: &'a mut Learning,
        lesson: &'a Lesson,
    },
    Assignment {
        attempt: &'a mut Attempt,
        scenario: &'a Scenario,
    },
}
impl Session<'_> {
    fn parts(&mut self) -> (&web::State, &mut Draft) {
        match self {
            Self::Practice { learning, .. } => (&learning.simulation, &mut learning.draft),
            Self::Assignment { attempt, .. } => {
                let engine::State::ApacheStatic { simulation } = &attempt.state else {
                    unreachable!()
                };
                (
                    simulation,
                    attempt
                        .workspace
                        .as_mut()
                        .unwrap()
                        .apache_static_draft
                        .as_mut()
                        .unwrap(),
                )
            }
        }
    }
    fn state(&self) -> &web::State {
        match self {
            Self::Practice { learning, .. } => &learning.simulation,
            Self::Assignment { attempt, .. } => match &attempt.state {
                engine::State::ApacheStatic { simulation } => simulation,
                _ => unreachable!(),
            },
        }
    }
    fn draft(&self) -> &Draft {
        match self {
            Self::Practice { learning, .. } => &learning.draft,
            Self::Assignment { attempt, .. } => attempt
                .workspace
                .as_ref()
                .unwrap()
                .apache_static_draft
                .as_ref()
                .unwrap(),
        }
    }
    fn draft_mut(&mut self) -> &mut Draft {
        self.parts().1
    }
    fn valid(&self) -> bool {
        match self {
            Self::Practice { learning, lesson } => learning.valid_for(lesson),
            Self::Assignment { .. } => self.draft().valid_for(self.state()),
        }
    }
    fn checked(&self) -> bool {
        match self {
            Self::Practice { learning, .. } => learning.checked,
            Self::Assignment { attempt, .. } => attempt.workspace.as_ref().unwrap().checked,
        }
    }
    fn set_checked(&mut self, value: bool) {
        match self {
            Self::Practice { learning, .. } => learning.checked = value,
            Self::Assignment { attempt, .. } => attempt.workspace.as_mut().unwrap().checked = value,
        }
    }
    fn checks(&self) -> Vec<(bool, String)> {
        match self {
            Self::Practice { learning, lesson } => learning.checks(lesson),
            Self::Assignment { attempt, scenario } => {
                progress::current_apache_static_checks(attempt, scenario)
            }
        }
    }
    fn notice(&mut self, text: &str) {
        if let Self::Practice { learning, .. } = self {
            learning.notice = None;
        }
        self.draft_mut().notice = Some(work::notice(text));
    }
    fn editor(&mut self, kind: EditorKind) -> bool {
        let before = self.draft().editor_kind;
        self.draft_mut().editor_kind = Some(kind);
        if !self.valid() {
            self.draft_mut().editor_kind = before;
            return false;
        }
        before != Some(kind)
    }
    fn select_file(&mut self, path: &str) -> Result<(), String> {
        let before = (
            self.draft().selected_file.clone(),
            self.draft().file_text.clone(),
        );
        let result = {
            let (state, draft) = self.parts();
            draft.select_file(state, path)
        };
        result?;
        if !self.valid() {
            self.draft_mut().selected_file = before.0;
            self.draft_mut().file_text = before.1;
            return Err("File selection exceeds this example's retained draft bound".into());
        }
        Ok(())
    }
    fn perform(&mut self, action: &Action) -> Result<(), String> {
        match self {
            Self::Practice { learning, lesson } => learning.perform(lesson, action).map(|_| ()),
            Self::Assignment { attempt, scenario } => {
                let engine::Goal::ApacheStatic { definition } = &scenario.goal else {
                    unreachable!()
                };
                let engine::State::ApacheStatic { simulation } = &mut attempt.state else {
                    unreachable!()
                };
                work::apply(&definition.initial, simulation, action).map(|_| ())
            }
        }
    }
    fn revert(&mut self, file: bool) -> Result<(), String> {
        let field = if file { Field::File } else { Field::Config };
        let text = if file {
            let path = self
                .draft()
                .selected_file
                .as_ref()
                .ok_or("Select a supplied file first")?;
            self.state()
                .files
                .get(path)
                .cloned()
                .ok_or("The selected live file is absent")?
        } else {
            self.state().saved.clone()
        };
        let before = std::mem::replace(field.value(self.draft_mut()), text);
        if !self.valid() {
            *field.value(self.draft_mut()) = before;
            return Err("Revert exceeds retained example bounds".into());
        }
        Ok(())
    }
}
fn control(
    ui: &mut egui::Ui,
    metrics: &mut Metrics,
    key: &str,
    text: &str,
    main: bool,
) -> egui::Response {
    let r = if main {
        ui.add(primary(ui, text))
    } else {
        ui.add(button(text))
    };
    name_control(&r, text);
    metrics
        .apache_controls
        .push((key.into(), r.rect, r.enabled()));
    r
}
fn edit(
    ui: &mut egui::Ui,
    session: &mut Session<'_>,
    field: Field,
    scope: egui::Id,
    metrics: &mut Metrics,
    focus: &mut bool,
    available: f32,
) -> Edit {
    let key = field.key();
    let path = if matches!(field, Field::File) {
        session.draft().selected_file.as_deref().unwrap_or("")
    } else if matches!(field, Field::Config) {
        &session.state().inventory.config_path
    } else {
        "request"
    };
    let id = scope.with(("apache-field", &key, path));
    let multiline = matches!(field, Field::Config | Field::File);
    let absent_host = matches!(field, Field::Host(0)) && session.draft().request_hosts.is_empty();
    // A zero-header legacy draft stays zero on unchanged/disabled frames. The
    // visible main field commits its first header only after an actual edit.
    let mut absent_text = String::new();
    let mut buffer = InputBuffer {
        value: if absent_host {
            &mut absent_text
        } else {
            field.value(session.draft_mut())
        },
        limit: field.limit(),
        before: None,
        rejected: false,
    };
    let response = if multiline {
        let height =
            ui.text_style_height(&egui::TextStyle::Monospace) * 7. + 2. * controls::PAD_Y as f32;
        let width =
            (available - 2. * controls::PAD_X as f32 - ui.spacing().scroll.allocated_width())
                .max(0.);
        let out = egui::ScrollArea::both()
            .id_salt(id.with("scroll"))
            .min_scrolled_height(height)
            .max_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut buffer)
                        .id(id)
                        .code_editor()
                        .lock_focus(false)
                        .desired_width(width)
                        .desired_rows(7)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y)),
                )
            });
        metrics.apache_editor = Some((out.inner.rect, out.inner_rect, id, out.inner.enabled()));
        metrics.editor_viewport = Some(out.inner_rect);
        metrics.inputs.push(out.inner_rect);
        out.inner
    } else {
        let width = (available - 2. * controls::PAD_X as f32).max(0.);
        let r = ui.add(
            controls::singleline(ui, &mut buffer)
                .id(id)
                .desired_width(width),
        );
        metrics.inputs.push(r.rect);
        r
    };
    name_control(&response, format!("Apache {key} editor"));
    metrics
        .apache_fields
        .push((key, response.rect, id, response.enabled()));
    if *focus && multiline && ui.is_enabled() {
        response.request_focus();
        *focus = false;
    }
    let mut result = buffer.finish();
    if result.changed {
        if absent_host {
            session.draft_mut().request_hosts.push(absent_text);
        }
        if !session.valid() {
            if absent_host {
                session.draft_mut().request_hosts.pop();
            } else {
                *field.value(session.draft_mut()) = result.previous.take().unwrap();
            }
            result.changed = false;
            result.rejected = true;
        }
    }
    if result.rejected {
        session.notice("Edit not retained: this field or example exceeds its UTF-8 byte bound. Previous text is unchanged.");
    }
    result
}

#[derive(Default)]
struct Changes {
    dirty: bool,
    rejected: bool,
}
fn changed_edit(
    session: &mut Session<'_>,
    presentation: &mut Presentation,
    field: Field,
    result: &Edit,
    changes: &mut Changes,
) {
    changes.dirty |= result.changed || result.rejected;
    changes.rejected |= result.rejected;
    if field.changes_configuration() && (result.changed || result.rejected) {
        session.set_checked(false);
        presentation.clear();
    } else if result.changed || result.rejected {
        presentation.request_visible = false;
    }
}
fn editor_panel(
    ui: &mut egui::Ui,
    session: &mut Session<'_>,
    presentation: &mut Presentation,
    scope: egui::Id,
    metrics: &mut Metrics,
    focus: &mut bool,
    changes: &mut Changes,
) {
    ui.label(RichText::new("Saved configuration / live files").strong());
    let selected = session.draft().editor_kind.unwrap_or(EditorKind::Config);
    ui.horizontal_wrapped(|ui| {
        for (kind, text, key) in [
            (EditorKind::Config, "Configuration", "config-tab"),
            (EditorKind::File, "Files", "file-tab"),
        ] {
            let r = ui.selectable_label(selected == kind, text);
            metrics
                .apache_controls
                .push((key.into(), r.rect, r.enabled()));
            if r.clicked() {
                changes.dirty |= session.editor(kind);
                *focus = true;
            }
        }
    });
    let file = session.draft().editor_kind == Some(EditorKind::File);
    if file {
        let mut choice = None;
        controls::form_row(ui, "Live file", |ui, width| {
            let r = controls::dropdown(ui, "apache-file", width)
                .selected_text(
                    session
                        .draft()
                        .selected_file
                        .as_deref()
                        .unwrap_or("Select supplied file"),
                )
                .show_ui(ui, |ui| {
                    controls::fit_popup_contents(ui);
                    for (path, kind) in &session.state().inventory.entries {
                        if *kind == web::EntryKind::File {
                            let r = ui.selectable_label(
                                session.draft().selected_file.as_ref() == Some(path),
                                path,
                            );
                            metrics.apache_file_items.push((path.clone(), r.rect));
                            if r.clicked() {
                                choice = Some(path.clone());
                            }
                        }
                    }
                    metrics.apache_popup = Some(("file".into(), controls::popup_content_clip(ui)));
                });
            metrics.apache_controls.push((
                "file-picker".into(),
                r.response.rect,
                r.response.enabled(),
            ));
        });
        if let Some(path) = choice {
            changes.dirty = true;
            if let Err(e) = session.select_file(&path) {
                session.notice(&format!("File not selected: {e}"));
            } else {
                *focus = true;
            }
        }
    } else {
        ui.label(&session.state().inventory.config_path);
    }
    let field = if file { Field::File } else { Field::Config };
    if !file || session.draft().selected_file.is_some() {
        let available = ui.available_width();
        let result = edit(ui, session, field, scope, metrics, focus, available);
        changed_edit(session, presentation, field, &result, changes);
        let mut save = false;
        let mut revert = false;
        ui.horizontal_wrapped(|ui| {
            save = control(
                ui,
                metrics,
                if file { "save-file" } else { "save-config" },
                if file {
                    "Save file"
                } else {
                    "Save configuration"
                },
                false,
            )
            .clicked();
            revert = control(
                ui,
                metrics,
                if file { "revert-file" } else { "revert-config" },
                if file {
                    "Revert file edits"
                } else {
                    "Revert configuration edits"
                },
                false,
            )
            .clicked();
        });
        if (save || revert) && !changes.rejected {
            presentation.clear();
            session.set_checked(false);
            changes.dirty = true;
            let result = if revert {
                session.revert(file)
            } else {
                let action = if file {
                    Action::WriteFile {
                        path: session.draft().selected_file.clone().unwrap(),
                        text: session.draft().file_text.clone(),
                    }
                } else {
                    Action::SaveConfig {
                        text: session.draft().config_text.clone(),
                    }
                };
                session.perform(&action)
            };
            match result {
                Ok(())=>session.notice(if revert{"Draft restored from actual saved/live text. Check work again when ready."}else if file{"File saved · Requests use these live contents; configuration activation is separate."}else{"Configuration saved · Start or Reload separately to activate it."}),
                Err(e)=>session.notice(&format!("{} not applied: {e}",if revert{"Revert"}else if file{"File save"}else{"Configuration save"})),
            }
        }
    }
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        for (key, label, value) in [
            ("start", "Start", Action::Start {}),
            ("reload", "Reload", Action::Reload {}),
            ("stop", "Stop", Action::Stop {}),
        ] {
            if control(ui, metrics, key, label, false).clicked() {
                action = Some((label, value));
            }
        }
    });
    if let Some((label, action)) = action.filter(|_| !changes.rejected) {
        changes.dirty = true;
        presentation.clear();
        session.set_checked(false);
        match session.perform(&action) {
            Ok(()) => session.notice(&format!(
                "{label} applied to the virtual server. Check work when ready."
            )),
            Err(e) => session.notice(&format!("{label} not applied: {e}")),
        }
    }
    let state = session.state();
    ui.label(format!(
        "Active configuration: {}",
        if state.active.is_some() {
            "serving"
        } else {
            "stopped"
        }
    ));
    if state.active.is_some() {
        ui.label(
            if state
                .active
                .as_ref()
                .is_some_and(|a| a.source == state.saved)
            {
                "Saved document matches active configuration"
            } else {
                "Saved document differs from active configuration"
            },
        );
    }
    if session.draft().config_text != state.saved {
        ui.label(
            RichText::new("Pending configuration text · Save configuration before checking.")
                .color(palette(ui).muted),
        );
    }
    if session
        .draft()
        .selected_file
        .as_ref()
        .is_some_and(|p| state.files.get(p) != Some(&session.draft().file_text))
    {
        ui.label(
            RichText::new("Pending file text · Save file before checking.")
                .color(palette(ui).muted),
        );
    }
    egui::CollapsingHeader::new("Active configuration · actual source")
        .id_salt(scope.with("active-source"))
        .show(ui, |ui| {
            if let Some(active) = &state.active {
                egui::ScrollArea::both()
                    .id_salt(scope.with("active-scroll"))
                    .max_height(160.)
                    .show(ui, |ui| {
                        ui.add(egui::Label::new(RichText::new(&active.source).monospace()).wrap());
                    });
            } else {
                ui.label("No active configuration while stopped.");
            }
        });
}
fn request_panel(
    ui: &mut egui::Ui,
    session: &mut Session<'_>,
    presentation: &mut Presentation,
    scope: egui::Id,
    metrics: &mut Metrics,
    changes: &mut Changes,
) {
    ui.label(RichText::new("Virtual request · applied server").strong());
    ui.label("These are in-process request fields. No socket or HTTP packet is sent.");
    ui.label(format!("Server: {}", session.state().inventory.server));
    for (label, field) in [
        ("Destination port", Field::Port),
        ("Request path", Field::Path),
    ] {
        controls::form_row(ui, label, |ui, width| {
            let result = edit(ui, session, field, scope, metrics, &mut false, width);
            changed_edit(session, presentation, field, &result, changes);
        });
    }
    controls::form_row(ui, "Method", |ui, width| {
        let old = session.draft().request_method;
        let r = controls::dropdown(ui, "apache-method", width)
            .selected_text(if old == Method::Get { "GET" } else { "HEAD" })
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (method, label) in [(Method::Get, "GET"), (Method::Head, "HEAD")] {
                    ui.selectable_value(&mut session.draft_mut().request_method, method, label);
                }
            });
        metrics
            .apache_controls
            .push(("method".into(), r.response.rect, r.response.enabled()));
        if old != session.draft().request_method {
            changes.dirty = true;
            presentation.request_visible = false;
        }
    });
    controls::form_row(ui, "Host name", |ui, width| {
        let result = edit(
            ui,
            session,
            Field::Host(0),
            scope,
            metrics,
            &mut false,
            width,
        );
        changed_edit(session, presentation, Field::Host(0), &result, changes);
    });
    if session.draft().request_hosts.len() == 2 {
        ui.label("Two Host fields are active · inspect Advanced request fields.");
    }
    let mut change = 0;
    let advanced = egui::CollapsingHeader::new("Advanced request fields")
        .id_salt(scope.with("advanced-hosts"))
        .default_open(false)
        .show(ui, |ui| {
            ui.label(format!(
                "{} Host fields sent · zero, one or two are retained exactly",
                session.draft().request_hosts.len()
            ));
            if session.draft().request_hosts.len() == 2 {
                controls::form_row(ui, "Second Host", |ui, width| {
                    let result = edit(
                        ui,
                        session,
                        Field::Host(1),
                        scope,
                        metrics,
                        &mut false,
                        width,
                    );
                    changed_edit(session, presentation, Field::Host(1), &result, changes);
                });
            }
            ui.horizontal_wrapped(|ui| {
                ui.add_enabled_ui(session.draft().request_hosts.len() < 2, |ui| {
                    if control(ui, metrics, "host-add", "Add Host field", false).clicked() {
                        change = 1;
                    }
                });
                ui.add_enabled_ui(!session.draft().request_hosts.is_empty(), |ui| {
                    if control(ui, metrics, "host-remove", "Remove last Host field", false)
                        .clicked()
                    {
                        change = -1;
                    }
                });
            });
        });
    metrics.apache_controls.push((
        "advanced-hosts".into(),
        advanced.header_response.rect,
        advanced.header_response.enabled(),
    ));
    if change != 0 {
        let before = session.draft().request_hosts.clone();
        if change > 0 {
            session.draft_mut().request_hosts.push(String::new());
        } else {
            session.draft_mut().request_hosts.pop();
        }
        if !session.valid() {
            session.draft_mut().request_hosts = before;
            session.notice("Host field change not retained: example draft bound");
        }
        changes.dirty = true;
        presentation.request_visible = false;
    }
    if control(ui, metrics, "send-request", "Send virtual request", false).clicked()
        && !changes.rejected
    {
        presentation.request_visible = true;
        changes.dirty = true;
    }
    if presentation.request_visible {
        // Actual applied state is observed at most once, never cached as credit.
        match session.draft().query(session.state()) {
            Err(e) => {
                metrics.apache_response = Some(
                    ui.label(
                        RichText::new(format!("Request not completed: {e}"))
                            .color(palette(ui).error),
                    )
                    .rect,
                );
            }
            Ok(Outcome::NoListener) => {
                metrics.apache_response = Some(ui.label("No active virtual listener for this server/port. This is not an HTTP denial response.").rect);
            }
            Ok(Outcome::Http(response)) => {
                let cause = match response.cause {
                    web::Cause::File => "file response",
                    web::Cause::DirectorySlash => "directory slash redirect",
                    web::Cause::InvalidHost => "invalid Host fields",
                    web::Cause::AccessDenied => "access denied",
                    web::Cause::MissingResource => "resource not found",
                    web::Cause::NoIndex => "no available index",
                };
                metrics.apache_response = Some(
                    ui.label(format!("HTTP {} · {cause}", response.status.code()))
                        .rect,
                );
                if let Some(site) = response.vhost {
                    ui.label(format!("Selected site: {site}"));
                }
                if let Some(path) = response.resource {
                    ui.label(format!("Mapped resource: {path}"));
                }
                if let Some(path) = response.redirect_path {
                    ui.label(format!("Redirect path: {path}"));
                }
                if let Some(bytes) = response.representation_bytes {
                    ui.label(format!("Representation length: {bytes} UTF-8 bytes"));
                }
                if !response.body.is_empty() {
                    let text = response
                        .body
                        .chars()
                        .map(|c| {
                            if c == '\n' || c == '\t' {
                                c.to_string()
                            } else {
                                c.escape_debug().to_string()
                            }
                        })
                        .collect::<String>();
                    egui::ScrollArea::both()
                        .id_salt(scope.with("response-body"))
                        .max_height(180.)
                        .show(ui, |ui| {
                            ui.add(
                                egui::Label::new(RichText::new(text).monospace())
                                    .selectable(true)
                                    .wrap(),
                            );
                        });
                }
            }
        }
    }
}
fn workbench(
    ui: &mut egui::Ui,
    session: &mut Session<'_>,
    presentation: &mut Presentation,
    id: &str,
    practice: bool,
    metrics: &mut Metrics,
    focus: &mut bool,
) -> (bool, bool) {
    presentation.bind(id, practice);
    let d = session.draft();
    if d.version != 1
        || d.config_text.len() > web::MAX_CONFIG
        || d.file_text.len() > web::MAX_FILE
        || d.request_port.len() > 8
        || d.request_path.len() > 256
        || d.request_hosts.len() > 2
        || d.request_hosts.iter().any(|h| h.len() > 260)
        || d.notice.as_ref().is_some_and(|n| n.len() > 512)
        || d.context.validate().is_err()
        || match &d.selected_file {
            Some(path) => {
                path.len() > 160 || d.context.entries.get(path) != Some(&web::EntryKind::File)
            }
            None => !d.file_text.is_empty(),
        }
        || d.context.entries.len() > web::MAX_ENTRIES
        || d.context != session.state().inventory
    {
        presentation.clear();
        ui.label("This editor context or retained text is outside its admitted bounds. Existing data was not changed.");
        return (false, false);
    }
    let scope = egui::Id::new(("apache-workbench", id, practice));
    let mut changes = Changes::default();
    // Stable field IDs were made before allocating responsive columns.
    if ui.available_width() >= 720. {
        ui.columns(2, |cols| {
            editor_panel(
                &mut cols[0],
                session,
                presentation,
                scope,
                metrics,
                focus,
                &mut changes,
            );
            request_panel(
                &mut cols[1],
                session,
                presentation,
                scope,
                metrics,
                &mut changes,
            );
        });
    } else {
        editor_panel(
            ui,
            session,
            presentation,
            scope,
            metrics,
            focus,
            &mut changes,
        );
        ui.separator();
        request_panel(ui, session, presentation, scope, metrics, &mut changes);
    }
    if let Some(notice) = &session.draft().notice {
        ui.label(notice);
    }
    let r = control(
        ui,
        metrics,
        "check",
        if practice {
            "Check example"
        } else {
            "Check work"
        },
        true,
    );
    metrics.check = Some(r.rect);
    let pressed = r.clicked()
        || (ui.is_enabled()
            && ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter)));
    let checked = pressed && !changes.rejected;
    if checked {
        session.set_checked(true);
        changes.dirty = true;
    }
    presentation.bind(id, practice);
    if !session.checked() {
        presentation.rows = None;
    }
    if ui.is_enabled() && session.checked() && (checked || presentation.rows.is_none()) {
        metrics.apache_check_evaluated = true;
        presentation.rows = Some(session.checks());
    }
    let out = controls::feedback(ui, checked, |ui| {
        if let Some(rows) = &presentation.rows {
            ui.label("Check results · from the last explicit check (or restored checked view)");
            let passed = rows.iter().all(|r| r.0);
            let mut target = ui
                .label(
                    RichText::new(if passed {
                        if practice {
                            "Example checks pass · No assignment credit"
                        } else {
                            "Objective complete · Check results pass"
                        }
                    } else {
                        "Check work needs attention"
                    })
                    .color(if passed {
                        palette(ui).success
                    } else {
                        palette(ui).error
                    }),
                )
                .rect;
            ui.label(format!(
                "{} of {} checks pass",
                rows.iter().filter(|r| r.0).count(),
                rows.len()
            ));
            let mut first = true;
            for (_, text) in rows.iter().filter(|r| !r.0) {
                let r =
                    ui.add(egui::Label::new(RichText::new(text).color(palette(ui).error)).wrap());
                if first {
                    target = target.union(r.rect);
                    first = false;
                }
            }
            let passing = rows.iter().filter(|r| r.0).count();
            if passing > 0 {
                egui::CollapsingHeader::new(format!("View {passing} passing checks"))
                    .id_salt(scope.with("pass-details"))
                    .show(ui, |ui| {
                        for (_, text) in rows.iter().filter(|r| r.0) {
                            ui.label(text);
                        }
                    });
            }
            metrics.apache_result = Some(target);
            metrics.completion = passed && !practice;
        }
    });
    metrics.feedback = Some(out.response.rect);
    (changes.dirty, checked)
}

impl LabApp {
    pub(super) fn invalidate_apache_presentation(&mut self) {
        self.apache_presentation.clear();
    }
    pub(super) fn prepare_apache_static(a: &mut Attempt, s: &Scenario, new: bool) {
        if let Some(lesson) = &s.apache_static_teaching {
            let missing = a
                .workspace
                .as_ref()
                .is_none_or(|w| w.apache_static_learning.is_none());
            dlms_learning_lab::apache_static_teaching::prepare(a, lesson)
                .expect("validated Apache lesson");
            if missing && !new {
                a.workspace
                    .as_mut()
                    .unwrap()
                    .apache_static_learning
                    .as_mut()
                    .unwrap()
                    .view = teaching::View::Assignment;
            }
        }
    }
    pub(super) fn apache_static_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            teaching::View::Assignment
        } else {
            a.workspace
                .as_ref()
                .and_then(|w| w.apache_static_learning.as_ref())
                .map_or(teaching::View::Assignment, |l| l.view)
        }
    }
    pub(super) fn set_apache_static_view(&mut self, i: usize, view: teaching::View) {
        let w = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap();
        if let Some(l) = &mut w.apache_static_learning {
            l.view = view;
        }
        self.checked = view == teaching::View::Assignment && w.checked;
        self.help_open = false;
        self.notice = None;
        self.focus_input = true;
        self.lesson_scroll_reset = true;
        self.dirty = true;
        self.invalidate_apache_presentation();
    }
    pub(super) fn apache_static_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let challenge = self.workspace_header(ui, i);
        if self.selected != Some(i) {
            return;
        }
        ui.add_space(8.);
        card(ui).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            let active = !self.reset_pending
                && !self.settings_open
                && !self.theme_preview
                && !self.data_tools.modal_open();
            if !active {
                self.focus_input = false;
            }
            ui.add_enabled_ui(active, |ui| self.apache_static_contents(ui, i));
            if self.apache_static_view(i) == teaching::View::Assignment {
                let r = ui.link(if self.help_open {
                    "Hide help"
                } else if challenge {
                    "Request help & references"
                } else {
                    "Hints, walkthrough & references"
                });
                self.metrics.help = Some(r.rect);
                if r.clicked() {
                    self.help_open = !self.help_open;
                }
                if self.help_open {
                    self.apache_static_help(ui, i);
                }
            }
        });
    }
    fn apache_static_contents(&mut self, ui: &mut egui::Ui, i: usize) {
        ui.heading(&self.scenarios[i].title);
        ui.label(
            RichText::new(
                "Virtual Apache-style static serving · No process, socket, TLS or host filesystem",
            )
            .small()
            .color(palette(ui).muted),
        );
        let id = self.scenarios[i].id.clone();
        let mut view = self.apache_static_view(i);
        let mut next = None;
        if !self.progress.attempts[&id].challenge
            && self.scenarios[i].apache_static_teaching.is_some()
        {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        next = Some(v);
                    }
                }
            });
        }
        if let Some(v) = next {
            self.set_apache_static_view(i, v);
            view = v;
        }
        match view {
            teaching::View::Introduction => {
                let lesson = self.scenarios[i].apache_static_teaching.as_ref().unwrap();
                ui.label(&lesson.prerequisite_note);
                for c in &lesson.concepts {
                    ui.label(RichText::new(&c.title).strong());
                    ui.label(&c.text);
                }
                ui.label(RichText::new("A different virtual site to try").strong());
                ui.label(&lesson.example_description);
                let mut go = None;
                ui.horizontal_wrapped(|ui| {
                    let r = control(
                        ui,
                        &mut self.metrics,
                        "try-example",
                        "Try the separate example",
                        true,
                    );
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        go = Some(teaching::View::Example);
                    }
                    let r = control(
                        ui,
                        &mut self.metrics,
                        "open-assignment",
                        "Open full assignment",
                        false,
                    );
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        go = Some(teaching::View::Assignment);
                    }
                });
                if let Some(v) = go {
                    self.set_apache_static_view(i, v);
                }
            }
            teaching::View::Example => {
                let s = &self.scenarios[i];
                let lesson = s.apache_static_teaching.as_ref().unwrap();
                let a = self.progress.attempts.get_mut(&id).unwrap();
                let l = a
                    .workspace
                    .as_mut()
                    .unwrap()
                    .apache_static_learning
                    .as_mut()
                    .unwrap();
                if l.selected_action >= lesson.example_actions.len() {
                    ui.label("Example action context differs; existing data was not changed.");
                    self.apache_presentation.clear();
                    return;
                }
                ui.label("Separate practice · Changes here never complete the assignment.");
                ui.label(&lesson.example_description);
                let mut dirty = false;
                let mut defer_focus = false;
                ui.push_id(("apache-example-action",&id),|ui|{
                    let before=l.selected_action;
                    let r=controls::dropdown(ui,"action",ui.available_width()).selected_text(&lesson.example_actions[l.selected_action].title).show_ui(ui,|ui|{
                        controls::fit_popup_contents(ui);
                        for (n,step) in lesson.example_actions.iter().enumerate(){let r=ui.selectable_value(&mut l.selected_action,n,&step.title);self.metrics.apache_action_items.push((n,r.rect));}
                        self.metrics.apache_popup=Some(("action".into(),controls::popup_content_clip(ui)));
                    });self.metrics.apache_controls.push(("example-action".into(),r.response.rect,r.response.enabled()));dirty|=before!=l.selected_action;
                    let step=&lesson.example_actions[l.selected_action];ui.label(&step.explanation);
                    let can_insert=matches!(step.action,Action::SaveConfig{..}|Action::WriteFile{..});
                    let mut insert=false;let mut reset=false;
                    ui.horizontal_wrapped(|ui|{
                        ui.add_enabled_ui(can_insert,|ui|{let r=control(ui,&mut self.metrics,"insert","Insert editor text",false);self.metrics.insert=Some(r.rect);insert=r.clicked();});
                        let r=control(ui,&mut self.metrics,"reset-example","Reset example only",false);self.metrics.example_reset=Some(r.rect);reset=r.clicked();
                    });
                    if insert {
                        self.apache_presentation.clear();dirty=true;
                        match l.insert_selected(lesson){
                            Ok(())=>{let editor=if matches!(step.action,Action::WriteFile{..}){EditorKind::File}else{EditorKind::Config};let prior=l.draft.editor_kind;l.draft.editor_kind=Some(editor);if !l.valid_for(lesson){l.draft.editor_kind=prior;}self.focus_input=true;defer_focus=true;}
                            Err(e)=>l.notice=Some(work::notice(&format!("Text not inserted: {e}"))),
                        }
                    }
                    if reset {match Learning::new(lesson){Ok(mut fresh)=>{fresh.view=teaching::View::Example;*l=fresh;self.apache_presentation.clear();self.focus_input=true;defer_focus=true;dirty=true;},Err(e)=>l.notice=Some(work::notice(&e))}}
                    ui.label(RichText::new("Insert fills draft text only. Save configuration or Save file applies it; Start/Reload/Stop are separate labeled controls.").small().color(palette(ui).muted));
                    if !can_insert {ui.label("This step has no text to insert. Use its labeled virtual server control.");}
                });
                let mut session = Session::Practice {
                    learning: l,
                    lesson,
                };
                // The pointer release still belongs to Insert/Reset this frame.
                // Give the editor focus on the following frame, after dispatch.
                let mut panel_focus = self.focus_input && !defer_focus;
                let (changed, _) = workbench(
                    ui,
                    &mut session,
                    &mut self.apache_presentation,
                    &id,
                    true,
                    &mut self.metrics,
                    &mut panel_focus,
                );
                if !defer_focus {
                    self.focus_input = panel_focus;
                }
                dirty |= changed;
                if let Session::Practice { learning, .. } = &session
                    && let Some(n) = &learning.notice
                {
                    ui.label(n);
                }
                ui.label(format!("Try a change: {}", lesson.try_change.description));
                self.dirty |= dirty;
                let r = control(
                    ui,
                    &mut self.metrics,
                    "open-assignment",
                    "Open full assignment",
                    false,
                );
                self.metrics.assignment_action = Some(r.rect);
                if r.clicked() {
                    self.set_apache_static_view(i, teaching::View::Assignment);
                }
            }
            teaching::View::Assignment => {
                let s = &self.scenarios[i];
                ui.label(RichText::new("Full task").strong());
                ui.label(&s.objective);
                let a = self.progress.attempts.get_mut(&id).unwrap();
                if !matches!(a.state, engine::State::ApacheStatic { .. })
                    || a.workspace
                        .as_ref()
                        .is_none_or(|w| w.apache_static_draft.is_none())
                {
                    ui.label(
                        "Assignment state/editor context differs; existing data was not changed.",
                    );
                    self.apache_presentation.clear();
                    return;
                }
                let mut session = Session::Assignment {
                    attempt: a,
                    scenario: s,
                };
                let (changed, _) = workbench(
                    ui,
                    &mut session,
                    &mut self.apache_presentation,
                    &id,
                    false,
                    &mut self.metrics,
                    &mut self.focus_input,
                );
                self.checked = session.checked();
                self.dirty |= changed;
                let passed = self
                    .apache_presentation
                    .rows
                    .as_ref()
                    .is_some_and(|rows| rows.iter().all(|r| r.0));
                if passed {
                    if !self.progress.attempts[&id].challenge {
                        egui::CollapsingHeader::new("Why this configuration works")
                            .id_salt(("apache-why", &id))
                            .show(ui, |ui| {
                                ui.label(&self.scenarios[i].explanation);
                            });
                    }
                    self.next_unit_lab(ui, i);
                }
            }
        }
    }
    pub(super) fn apache_static_help(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = &self.scenarios[i];
        ui.label(work::HELP);
        for hint in &s.hints {
            ui.label(hint);
        }
        egui::CollapsingHeader::new("Walkthrough")
            .id_salt((
                "apache-walkthrough",
                &s.id,
                self.progress.attempts[&s.id].challenge,
            ))
            .show(ui, |ui| {
                for step in &s.steps {
                    ui.label(step);
                }
            });
        for source in &s.sources {
            ui.hyperlink(source);
        }
        ui.label(&s.simplifications);
    }
}

#[cfg(test)]
mod buffer_tests {
    use super::*;
    use egui::TextBuffer;
    #[test]
    fn apache_editor_byte_bound_restores_selection_and_preserves_utf8() {
        let mut value = "éab".to_owned();
        let mut b = InputBuffer {
            value: &mut value,
            limit: 4,
            before: None,
            rejected: false,
        };
        b.delete_char_range(0..3);
        assert_eq!(b.insert_text("ééé", 0), 0);
        let result = b.finish();
        assert!(result.rejected && !result.changed);
        assert_eq!(value, "éab");
        let mut b = InputBuffer {
            value: &mut value,
            limit: 6,
            before: None,
            rejected: false,
        };
        assert_eq!(b.insert_text("é", 1), 1);
        assert!(b.finish().changed);
        assert_eq!(value, "ééab");
    }
    #[test]
    fn apache_unchanged_editor_has_no_retained_snapshot() {
        let mut value = "unchanged".repeat(1000);
        let mut b = InputBuffer {
            value: &mut value,
            limit: web::MAX_CONFIG,
            before: None,
            rejected: false,
        };
        assert_eq!(b.insert_text("", 0), 0);
        b.delete_char_range(0..0);
        let result = b.finish();
        assert!(!result.changed && !result.rejected && result.previous.is_none());
    }
}
