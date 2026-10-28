//! Compact NFS editors and explicit virtual actions; no terminal/host bridge.
use super::*;
use dlms_learning_lab::{nfs as core, nfs_teaching as learn, nfs_workspace as work};
pub(super) mod memo;
#[derive(Default)]
pub(super) struct NfsMetrics {
    pub heading: Option<egui::Rect>,
    pub read_output: Option<(egui::Rect, egui::Rect)>,
    pub row_clips: Vec<(usize, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, bool, egui::Rect)>,
    pub documents: Vec<(String, egui::Rect, egui::Rect, egui::Vec2)>,
    pub controls: Vec<(String, egui::Rect, bool, egui::Rect)>,
    pub choices: Vec<(String, egui::Rect)>,
    pub selections: Vec<(String, String)>,
    pub popup: Option<(String, egui::Rect)>,
    pub tabs: Vec<(teaching::View, egui::Rect)>,
    pub rows: Vec<(usize, bool, egui::Rect)>,
    pub result: Option<egui::Rect>,
    pub current: Option<egui::Rect>,
    pub columns: bool,
    pub state_text: Vec<String>,
    pub check_evaluated: bool,
    pub submitted: Option<&'static str>,
    pub queued: Option<&'static str>,
    pub read_shown: bool,
    pub notice_shown: bool,
    pub pending_text: bool,
    pub received_text: bool,
    pub dropped_text: bool,
}
#[derive(Default)]
struct Document {
    key: Option<egui::Id>,
    value: String,
    reset_undo: bool,
}
#[derive(Clone, Default, PartialEq, Eq)]
struct RequestFields {
    server: String,
    client: String,
    mountpoint: String,
    profile: String,
    relative: String,
    write_text: String,
}
impl RequestFields {
    fn from_session(t: &work::Session) -> Self {
        Self {
            server: t.server.clone(),
            client: t.client.clone(),
            mountpoint: t.mountpoint.clone(),
            profile: t.profile.clone(),
            relative: t.relative.clone(),
            write_text: t.write_text.clone(),
        }
    }
    fn matches(&self, t: &work::Session) -> bool {
        self.server == t.server
            && self.client == t.client
            && self.mountpoint == t.mountpoint
            && self.profile == t.profile
            && self.relative == t.relative
            && self.write_text == t.write_text
    }
    fn defaults(&mut self, s: &core::State) {
        if self.server.is_empty() {
            self.server = s
                .inventory()
                .servers
                .first()
                .map(|s| s.id.clone())
                .unwrap_or_default();
        }
        if self.client.is_empty() {
            self.client = s
                .inventory()
                .clients
                .first()
                .map(|c| c.id.clone())
                .unwrap_or_default();
        }
        if let Some(c) = s.inventory().clients.iter().find(|c| c.id == self.client) {
            if self.mountpoint.is_empty() {
                self.mountpoint = c.mountpoints.first().cloned().unwrap_or_default();
            }
            if self.profile.is_empty() {
                self.profile = c.profiles.first().cloned().unwrap_or_default();
            }
        }
    }
    fn reconcile_client(&mut self, s: &core::State) {
        if let Some(c) = s.inventory().clients.iter().find(|c| c.id == self.client) {
            if !c.mountpoints.contains(&self.mountpoint) {
                self.mountpoint.clear();
            }
            if !c.profiles.contains(&self.profile) {
                self.profile.clear();
            }
        } else {
            self.mountpoint.clear();
            self.profile.clear();
        }
    }
}
#[derive(Default)]
pub(super) struct Presentation {
    pub memo: memo::Memo,
    epoch: u64,
    owner: Option<egui::Id>,
    request: RequestFields,
    synced: RequestFields,
    exports: Document,
    fstab: Document,
    root: String,
    access: Option<core::Access>,
    checked: [bool; 2],
    notice: Option<String>,
    last_read: Option<String>,
    focus: Option<egui::Id>,
}
impl Presentation {
    #[cfg(test)]
    pub(in crate::ui) fn observed_write_request(&self) -> (&str, Option<&str>) {
        (&self.request.write_text, self.notice.as_deref())
    }
    #[cfg(test)]
    pub(in crate::ui) fn observed_request_selection(&self) -> (&str, &str, &str, &str) {
        (
            &self.request.server,
            &self.request.client,
            &self.request.mountpoint,
            &self.request.profile,
        )
    }
    pub fn clear(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.owner = None;
        self.exports = Document::default();
        self.fstab = Document::default();
        self.request = RequestFields::default();
        self.synced = RequestFields::default();
        self.root.clear();
        self.access = None;
        self.checked = [false; 2];
        self.notice = None;
        self.last_read = None;
        self.focus = None;
        self.memo.clear();
    }
    fn bind(&mut self, scope: egui::Id, s: &core::State, t: &work::Session) {
        if self.owner != Some(scope) {
            self.owner = Some(scope);
            self.exports = Document::default();
            self.fstab = Document::default();
            self.root.clear();
            self.access = None;
            self.focus = None;
            self.notice = None;
            self.last_read = None;
            self.synced = RequestFields::from_session(t);
            self.request = self.synced.clone();
            self.request.defaults(s);
        } else if !self.synced.matches(t) {
            self.synced = RequestFields::from_session(t);
            self.request = self.synced.clone();
            self.request.defaults(s);
        }
        if !s
            .inventory()
            .roots
            .iter()
            .any(|r| r.server == self.request.server && r.path == self.root)
        {
            self.root = s
                .inventory()
                .roots
                .iter()
                .find(|r| r.server == self.request.server)
                .map(|r| r.path.clone())
                .unwrap_or_default();
        }
        if self.access.is_none() {
            self.access = Some(core::Access::Ro);
        }
    }
}
/// Bounded local edit shadow only; persisted State is never edited then rolled back.
struct Buffer<'a> {
    value: &'a mut String,
    cap: usize,
    before: Option<String>,
    rejected: bool,
    received: bool,
}
struct Edit {
    before: Option<String>,
    rejected: bool,
    received: bool,
    id: Option<egui::Id>,
}
impl Buffer<'_> {
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
                before: None,
                rejected: true,
                received: self.received,
                id: None,
            }
        } else {
            Edit {
                before: self
                    .before
                    .filter(|old| old.as_str() != self.value.as_str()),
                rejected: false,
                received: self.received,
                id: None,
            }
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
        // Observe actual bounded-buffer delivery independently of value change.
        // Identical Paste still reaches this insertion; rejected text is observed
        // too, but remains vetoed through the separate admission/rejection flag.
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
    fn delete_char_range(&mut self, r: std::ops::Range<usize>) {
        if !self.rejected && !r.is_empty() {
            self.remember();
            egui::TextBuffer::delete_char_range(self.value, r);
        }
    }
}
fn button_action(ui: &mut egui::Ui, m: &mut NfsMetrics, key: &str, text: &str, main: bool) -> bool {
    let r = ui.add_sized(
        [controls::button_width(ui, text), controls::height(ui)],
        if main {
            primary(ui, text)
        } else {
            button(text)
        },
    );
    m.controls
        .push((key.into(), r.rect, r.enabled(), ui.clip_rect()));
    r.clicked()
}
fn pick(
    ui: &mut egui::Ui,
    m: &mut NfsMetrics,
    scope: egui::Id,
    key: &str,
    label: &str,
    value: &mut String,
    options: impl IntoIterator<Item = String>,
) -> bool {
    let mut changed = false;
    controls::form_row(ui, label, |ui, width| {
        let selected = if value.is_empty() {
            "Choose"
        } else {
            value.as_str()
        };
        let response = controls::dropdown(ui, scope.with(key), width)
            .selected_text(selected)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for option in options {
                    let r = ui.selectable_label(*value == option, &option);
                    if r.clicked() && *value != option {
                        *value = option.clone();
                        changed = true;
                    }
                    m.choices.push((format!("{key}:{option}"), r.rect));
                }
                m.popup = Some((key.into(), controls::popup_content_clip(ui)));
            });
        m.selections.push((key.into(), value.clone()));
        m.controls.push((
            key.into(),
            response.response.rect,
            response.response.enabled(),
            ui.clip_rect(),
        ));
    });
    changed
}
struct FieldSpec<'a> {
    scope: egui::Id,
    key: &'a str,
    label: &'a str,
    cap: usize,
    multiline: bool,
}
fn field(
    ui: &mut egui::Ui,
    m: &mut NfsMetrics,
    spec: FieldSpec<'_>,
    value: &mut String,
    focus: &mut Option<egui::Id>,
) -> Edit {
    let FieldSpec {
        scope,
        key,
        label,
        cap,
        multiline,
    } = spec;
    controls::form_row(ui, label, |ui, width| {
        let id = scope.with(key);
        let mut b = Buffer {
            value,
            cap,
            before: None,
            rejected: false,
            received: false,
        };
        let r = if multiline {
            ui.add_sized(
                [width, controls::height(ui) * 2.],
                egui::TextEdit::multiline(&mut b)
                    .id(id)
                    .font(egui::TextStyle::Body)
                    .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y)),
            )
        } else {
            ui.add_sized(
                [width, controls::height(ui)],
                controls::singleline(ui, &mut b).id(id),
            )
        };
        r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, r.enabled(), label));
        if *focus == Some(id) && r.enabled() {
            r.request_focus();
            *focus = None;
        }
        m.fields
            .push((key.into(), r.rect, r.id, r.enabled(), ui.clip_rect()));
        let mut edit = b.finish();
        edit.id = Some(r.id);
        edit
    })
    .inner
}
struct DocumentSpec<'a> {
    scope: egui::Id,
    key: &'a str,
    owner: &'a str,
    current: &'a str,
}
fn document(
    ui: &mut egui::Ui,
    m: &mut NfsMetrics,
    spec: DocumentSpec<'_>,
    doc: &mut Document,
    focus: &mut Option<egui::Id>,
) -> Edit {
    let DocumentSpec {
        scope,
        key,
        owner,
        current,
    } = spec;
    let id = scope.with((key, owner));
    let replaced_selected_document = doc.key == Some(id) && doc.value != current;
    if doc.key != Some(id) || doc.value != current {
        if current.len() > core::MAX_DOCUMENT || current.contains('\0') {
            return Edit {
                before: None,
                rejected: true,
                received: false,
                id: Some(id),
            };
        }
        doc.key = Some(id);
        doc.value = current.into();
    }
    // External document replacement establishes a fresh native edit generation.
    // Reflow and returning to the same unchanged live document keep their Undo.
    if (std::mem::take(&mut doc.reset_undo) || replaced_selected_document)
        && let Some(mut state) = egui::text_edit::TextEditState::load(ui.ctx(), id)
    {
        state.clear_undoer();
        state.store(ui.ctx(), id);
    }
    let height = (ui.text_style_height(&egui::TextStyle::Monospace) * 3.
        + 2. * f32::from(controls::PAD_Y))
    .max(controls::height(ui) * 2.);
    let editor_width = (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(1.);
    let mut b = Buffer {
        value: &mut doc.value,
        cap: core::MAX_DOCUMENT,
        before: None,
        rejected: false,
        received: false,
    };
    // The outer viewport includes a style-derived scrollbar gutter. Reserve it
    // before rendering either editor: egui scrollbar transitions may change the
    // inner size, but must never change the following Save row's position.
    let outer_size = egui::vec2(
        ui.available_width(),
        height + ui.spacing().scroll.allocated_width(),
    );
    let (outer, scrolled) = controls::fixed_viewport(ui, outer_size, |ui| {
        egui::ScrollArea::both()
            .id_salt(id.with("scroll"))
            .max_height(outer_size.y)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut b)
                        .id(id)
                        .font(egui::TextStyle::Monospace)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .desired_width(editor_width)
                        .desired_rows(3),
                )
            })
    });
    m.documents.push((
        key.into(),
        outer,
        scrolled.inner_rect,
        scrolled.content_size,
    ));
    let r = scrolled.inner;
    r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, r.enabled(), key));
    if *focus == Some(id) && r.enabled() {
        r.request_focus();
        *focus = None;
    }
    m.fields.push((
        key.into(),
        r.rect.intersect(scrolled.inner_rect),
        r.id,
        r.enabled(),
        scrolled.inner_rect.intersect(ui.clip_rect()),
    ));
    let mut edit = b.finish();
    edit.id = Some(r.id);
    edit
}
#[derive(Default)]
struct Edits {
    q: Option<Submit>,
    pending_target: Option<egui::Id>,
    received: bool,
    dropped: bool,
    server: bool,
    client: bool,
    mountpoint: bool,
    profile: bool,
    relative: bool,
    write_text: bool,
    exports: Option<Edit>,
    fstab: Option<Edit>,
    rejected: bool,
}
impl Edits {
    fn observe(&mut self, edit: &Edit) {
        if self.pending_target.is_some() && edit.id == self.pending_target {
            self.received |= edit.received;
        }
    }
}
#[derive(Clone, Copy)]
enum Submit {
    SaveExports,
    ApplyExports,
    SaveFstab,
    MountSaved,
    MountExplicit,
    Unmount,
    Read,
    Write,
    Check,
    Fill,
    ResetExample,
}
impl Submit {
    fn key(self) -> &'static str {
        match self {
            Self::SaveExports => "save-exports",
            Self::ApplyExports => "apply-exports",
            Self::SaveFstab => "save-fstab",
            Self::MountSaved => "mount-saved",
            Self::MountExplicit => "mount-explicit",
            Self::Unmount => "unmount",
            Self::Read => "read",
            Self::Write => "write",
            Self::Check => "check",
            Self::Fill => "fill",
            Self::ResetExample => "reset-example",
        }
    }
}
fn queue(q: &mut Option<Submit>, clicked: bool, action: Submit) {
    if clicked && q.is_none() {
        *q = Some(action);
    }
}
fn selected(a: &Attempt, practice: bool) -> Option<(&core::State, &work::Session)> {
    if practice {
        let l = a.workspace.as_ref()?.nfs_learning.as_ref()?;
        Some((&l.simulation, &l.session))
    } else {
        let engine::State::Nfs { simulation } = &a.state else {
            return None;
        };
        Some((simulation, a.workspace.as_ref()?.nfs_session.as_ref()?))
    }
}
fn given_address(ui: &mut egui::Ui, label: &str, address: std::net::Ipv4Addr) {
    controls::form_row(ui, label, |ui, width| {
        controls::cell(ui, width, |ui| {
            ui.add(
                egui::Label::new(address.to_string())
                    .wrap()
                    .selectable(true),
            );
        });
    });
}
fn docs(
    ui: &mut egui::Ui,
    a: &Attempt,
    practice: bool,
    p: &mut Presentation,
    m: &mut NfsMetrics,
    scope: egui::Id,
    edits: &mut Edits,
) {
    let Some((s, _)) = selected(a, practice) else {
        return;
    };
    let minimum = controls::form_label_width(ui)
        + ui.text_style_height(&egui::TextStyle::Body) * 9.
        + 2. * f32::from(controls::PAD_X)
        + controls::GAP;
    let wide = ui.available_width() >= 2. * minimum + controls::GAP;
    m.columns = wide;
    let mut panel = |ui: &mut egui::Ui, exports: bool| {
        ui.push_id(
            scope.with(if exports {
                "exports-panel"
            } else {
                "mount-panel"
            }),
            |ui| {
                if exports {
                    ui.strong("Exports document");
                    edits.server |= pick(
                        ui,
                        m,
                        scope,
                        "server",
                        "Server",
                        &mut p.request.server,
                        s.inventory().servers.iter().map(|x| x.id.clone()),
                    );
                    if let Some(server) = s
                        .inventory()
                        .servers
                        .iter()
                        .find(|x| x.id == p.request.server)
                    {
                        given_address(ui, "Server IPv4", server.address);
                    }
                    if let Some(layer) = s
                        .runtime()
                        .servers
                        .iter()
                        .find(|x| x.server == p.request.server)
                    {
                        let edit = document(
                            ui,
                            m,
                            DocumentSpec {
                                scope,
                                key: "exports",
                                owner: &p.request.server,
                                current: &layer.draft,
                            },
                            &mut p.exports,
                            &mut p.focus,
                        );
                        edits.observe(&edit);
                        edits.exports = Some(edit);
                    }
                    ui.horizontal_wrapped(|ui| {
                        queue(
                            &mut edits.q,
                            button_action(ui, m, "save-exports", "Save exports", false),
                            Submit::SaveExports,
                        );
                        queue(
                            &mut edits.q,
                            button_action(ui, m, "apply-exports", "Apply saved exports", false),
                            Submit::ApplyExports,
                        );
                    });
                } else {
                    ui.strong("Client mount document");
                    edits.client |= pick(
                        ui,
                        m,
                        scope,
                        "client",
                        "Client",
                        &mut p.request.client,
                        s.inventory().clients.iter().map(|x| x.id.clone()),
                    );
                    if edits.client {
                        p.request.reconcile_client(s);
                        edits.profile = true;
                        edits.mountpoint = true;
                    }
                    if let Some(client) = s
                        .inventory()
                        .clients
                        .iter()
                        .find(|x| x.id == p.request.client)
                    {
                        given_address(ui, "Client IPv4", client.address);
                    }
                    if let Some(layer) = s
                        .runtime()
                        .clients
                        .iter()
                        .find(|x| x.client == p.request.client)
                    {
                        let edit = document(
                            ui,
                            m,
                            DocumentSpec {
                                scope,
                                key: "fstab",
                                owner: &p.request.client,
                                current: &layer.draft,
                            },
                            &mut p.fstab,
                            &mut p.focus,
                        );
                        edits.observe(&edit);
                        edits.fstab = Some(edit);
                    }
                    queue(
                        &mut edits.q,
                        button_action(ui, m, "save-fstab", "Save mount document", false),
                        Submit::SaveFstab,
                    );
                }
            },
        );
    };
    if wide {
        ui.columns(2, |cols| {
            panel(&mut cols[0], true);
            panel(&mut cols[1], false);
        });
    } else {
        panel(ui, true);
        ui.add_space(controls::GAP);
        panel(ui, false);
    }
}
fn requests(
    ui: &mut egui::Ui,
    a: &Attempt,
    practice: bool,
    p: &mut Presentation,
    m: &mut NfsMetrics,
    scope: egui::Id,
    edits: &mut Edits,
) {
    let Some((s, _)) = selected(a, practice) else {
        return;
    };
    ui.separator();
    let client = s
        .inventory()
        .clients
        .iter()
        .find(|c| c.id == p.request.client);
    edits.mountpoint |= pick(
        ui,
        m,
        scope,
        "mountpoint",
        "Local mountpoint",
        &mut p.request.mountpoint,
        client
            .into_iter()
            .flat_map(|c| c.mountpoints.iter().cloned()),
    );
    ui.horizontal_wrapped(|ui| {
        queue(
            &mut edits.q,
            button_action(ui, m, "mount-saved", "Mount saved entry", false),
            Submit::MountSaved,
        );
        queue(
            &mut edits.q,
            button_action(ui, m, "unmount", "Unmount", false),
            Submit::Unmount,
        );
    });
    let header = egui::CollapsingHeader::new("Manual mount source · Optional").id_salt(scope.with("manual-mount")).show(ui,|ui|{edits.server|=pick(ui,m,scope,"manual-server","Source server",&mut p.request.server,s.inventory().servers.iter().map(|x|x.id.clone()));pick(ui,m,scope,"root","Export root",&mut p.root,s.inventory().roots.iter().filter(|r|r.server==p.request.server).map(|r|r.path.clone()));ui.horizontal_wrapped(|ui|{ui.label("Client mount mode");let r=ui.selectable_value(&mut p.access,Some(core::Access::Ro),"Read only");m.controls.push(("access-ro".into(),r.rect,r.enabled(),ui.clip_rect()));let r=ui.selectable_value(&mut p.access,Some(core::Access::Rw),"Read/write");m.controls.push(("access-rw".into(),r.rect,r.enabled(),ui.clip_rect()));queue(&mut edits.q,button_action(ui,m,"mount-explicit","Mount explicit source",false),Submit::MountExplicit);});ui.label(RichText::new("Manual source/mode selections are temporary; document and request drafts are saved.").small());});
    m.controls.push((
        "manual-toggle".into(),
        header.header_response.rect,
        header.header_response.enabled(),
        ui.clip_rect(),
    ));
    edits.profile |= pick(
        ui,
        m,
        scope,
        "profile",
        "Numeric profile",
        &mut p.request.profile,
        client.into_iter().flat_map(|c| c.profiles.iter().cloned()),
    );
    let e = field(
        ui,
        m,
        FieldSpec {
            scope,
            key: "relative",
            label: "Relative file",
            cap: 160,
            multiline: false,
        },
        &mut p.request.relative,
        &mut p.focus,
    );
    edits.observe(&e);
    edits.relative |= e.before.is_some();
    edits.rejected |= e.rejected;
    queue(
        &mut edits.q,
        button_action(ui, m, "read", "Read existing file", false),
        Submit::Read,
    );
}
fn write_editor(
    ui: &mut egui::Ui,
    p: &mut Presentation,
    m: &mut NfsMetrics,
    scope: egui::Id,
    edits: &mut Edits,
    required: bool,
) {
    let header = egui::CollapsingHeader::new(if required { "Replace existing file text" } else { "Replace existing file text · Optional" }).id_salt(scope.with("write")).default_open(required).show(ui,|ui|{let e=field(ui,m,FieldSpec{scope,key:"write-text",label:"Replacement text",cap:4096,multiline:true},&mut p.request.write_text,&mut p.focus);edits.observe(&e);edits.write_text|=e.before.is_some();edits.rejected|=e.rejected;queue(&mut edits.q,button_action(ui,m,"write","Write existing file",false),Submit::Write);ui.label(RichText::new("File permissions, owner and paths are supplied by the lab; editing this text does not change them.").small());});
    m.controls.push((
        "write-toggle".into(),
        header.header_response.rect,
        header.header_response.enabled(),
        ui.clip_rect(),
    ));
}
fn operation(
    a: &mut Attempt,
    s: &Scenario,
    practice: bool,
    action: &core::Action,
) -> Result<core::Decision, String> {
    if practice {
        work::perform_practice(a, s, action)
    } else {
        work::perform_attempt(a, s, action)
    }
}
fn changed_notice(p: &mut Presentation, decision: core::Decision) {
    p.last_read = None;
    p.notice = Some(match decision {
        core::Decision::AllowedRead { text, .. } => {
            p.last_read = Some(text);
            "Read succeeded for this request".into()
        }
        core::Decision::AllowedWrite { .. } => "Existing virtual file written".into(),
        core::Decision::Mounted {} => "Virtual mount added".into(),
        core::Decision::Unmounted {} => "Virtual mount removed".into(),
        core::Decision::Denied { stage, .. } => match stage {
            core::DenialStage::MountAbsent => "Request denied: this client has no current mount",
            core::DenialStage::ExportAdmission => {
                "Request denied: the applied export does not admit this client"
            }
            core::DenialStage::ClientReadOnly => {
                "Write denied: the current client mount is read only"
            }
            core::DenialStage::ExportReadOnly => "Write denied: the applied export is read only",
            core::DenialStage::DirectorySearch => {
                "Request denied: directory search permission is missing"
            }
            core::DenialStage::FilePermission => {
                "Request denied: existing file permissions do not allow this access"
            }
        }
        .into(),
        core::Decision::Changed { changed } => if changed {
            "Virtual document updated"
        } else {
            "Virtual document already matched; no change"
        }
        .into(),
    });
}
fn apply_edits(
    a: &mut Attempt,
    s: &Scenario,
    practice: bool,
    p: &mut Presentation,
    e: &mut Edits,
) -> bool {
    let mut dirty = false;
    for (exports, edit) in [(true, e.exports.take()), (false, e.fstab.take())] {
        if let Some(edit) = edit {
            e.rejected |= edit.rejected;
            if let Some(old) = edit.before {
                let action = if exports {
                    core::Action::SetExportsDraft {
                        server: p.request.server.clone(),
                        text: p.exports.value.clone(),
                    }
                } else {
                    core::Action::SetFstabDraft {
                        client: p.request.client.clone(),
                        text: p.fstab.value.clone(),
                    }
                };
                match operation(a, s, practice, &action) {
                    Ok(_) => {
                        dirty = true;
                        p.checked[usize::from(practice)] = false;
                    }
                    Err(error) => {
                        if exports {
                            p.exports.value = old;
                        } else {
                            p.fstab.value = old;
                        }
                        e.rejected = true;
                        p.notice = Some(work::notice(error));
                    }
                }
            }
        }
    }
    if e.server || e.client || e.mountpoint || e.profile || e.relative || e.write_text {
        let fields = work::SessionEdit {
            server: Some(&p.request.server),
            client: Some(&p.request.client),
            mountpoint: Some(&p.request.mountpoint),
            profile: Some(&p.request.profile),
            relative: Some(&p.request.relative),
            write_text: Some(&p.request.write_text),
        };
        let result = if practice {
            work::edit_practice_session(a, s, fields)
        } else {
            work::edit_assignment_session(a, s, fields)
        };
        match result {
            Ok(()) => {
                dirty = true;
            }
            Err(error) => {
                p.request = p.synced.clone();
                e.rejected = true;
                p.notice = Some(work::notice(error));
            }
        }
    }
    if e.rejected {
        p.notice=Some(if e.dropped {
            "The pending text or paste was not applied. Enter it again in the field before retrying the action; no action was run."
        } else {
            "Your previous text is still saved and nothing ran. Shorten the edit and try again."
        }.into());
    }
    dirty
}
fn submit(
    a: &mut Attempt,
    s: &Scenario,
    practice: bool,
    p: &mut Presentation,
    scope: egui::Id,
    q: Submit,
) -> Result<bool, String> {
    if matches!(q, Submit::Check) {
        p.checked[usize::from(practice)] = true;
        return Ok(false);
    }
    if matches!(q, Submit::Fill) {
        work::fill_practice(a, s)?;
        p.notice = None;
        p.last_read = None;
        let l = a.workspace.as_ref().unwrap().nfs_learning.as_ref().unwrap();
        let source = s.nfs_teaching.as_ref().unwrap();
        p.focus = Some(match &source.example_actions[l.selected_example].action {
            core::Action::SetExportsDraft { server, .. } => {
                p.request.server = server.clone();
                p.exports.reset_undo = true;
                scope.with(("exports", server.as_str()))
            }
            core::Action::SetFstabDraft { client, .. } => {
                p.request.client = client.clone();
                p.request.reconcile_client(&l.simulation);
                p.fstab.reset_undo = true;
                scope.with(("fstab", client.as_str()))
            }
            core::Action::WriteExisting { .. } => scope.with("write-text"),
            _ => scope.with("relative"),
        });
        p.checked[1] = false;
        return Ok(true);
    }
    if matches!(q, Submit::ResetExample) {
        work::reset_practice(a, s)?;
        p.epoch = p.epoch.wrapping_add(1);
        p.owner = None;
        p.notice = None;
        p.last_read = None;
        p.checked[1] = false;
        return Ok(true);
    }
    let action = match q {
        Submit::SaveExports => core::Action::SaveExportsDraft {
            server: p.request.server.clone(),
        },
        Submit::ApplyExports => core::Action::ApplySavedExports {
            server: p.request.server.clone(),
        },
        Submit::SaveFstab => core::Action::SaveFstabDraft {
            client: p.request.client.clone(),
        },
        Submit::MountSaved => core::Action::MountSaved {
            client: p.request.client.clone(),
            mountpoint: p.request.mountpoint.clone(),
        },
        Submit::Unmount => core::Action::Unmount {
            client: p.request.client.clone(),
            mountpoint: p.request.mountpoint.clone(),
        },
        Submit::MountExplicit => core::Action::MountExplicit {
            client: p.request.client.clone(),
            specification: core::MountSpec {
                server: p.request.server.clone(),
                root: p.root.clone(),
                mountpoint: p.request.mountpoint.clone(),
                access: p.access.unwrap_or(core::Access::Ro),
            },
        },
        Submit::Read => core::Action::ReadExisting {
            client: p.request.client.clone(),
            mountpoint: p.request.mountpoint.clone(),
            profile: p.request.profile.clone(),
            relative: p.request.relative.clone(),
        },
        Submit::Write => core::Action::WriteExisting {
            client: p.request.client.clone(),
            mountpoint: p.request.mountpoint.clone(),
            profile: p.request.profile.clone(),
            relative: p.request.relative.clone(),
            text: p.request.write_text.clone(),
        },
        _ => unreachable!(),
    };
    let before_check = p.checked[usize::from(practice)];
    let result = operation(a, s, practice, &action)?;
    changed_notice(p, result);
    if !matches!(q, Submit::Read) {
        p.checked[usize::from(practice)] = false;
    } else {
        p.checked[usize::from(practice)] = before_check;
    }
    Ok(true)
}
fn access_label(a: core::Access) -> &'static str {
    match a {
        core::Access::Ro => "read only",
        core::Access::Rw => "read/write",
    }
}
fn live(ui: &mut egui::Ui, a: &Attempt, practice: bool, p: &Presentation, m: &mut NfsMetrics) {
    let Some((s, _)) = selected(a, practice) else {
        return;
    };
    let r=ui.vertical(|ui|{ui.strong("Actual current layers");if let Some(server)=s.runtime().servers.iter().find(|x|x.server==p.request.server){let text=format!("{} · editor {} saved text · {} applied export roots",server.server,if server.draft==server.saved{"matches"}else{"differs from"},server.applied.len());m.state_text.push(text.clone());ui.label(text);}
 if let Some(client)=s.runtime().clients.iter().find(|x|x.client==p.request.client){let text=format!("{} · mount editor {} saved text",client.client,if client.draft==client.saved{"matches"}else{"differs from"});m.state_text.push(text.clone());ui.label(text);}
 for mount in &s.runtime().mounts{let text=format!("{}: {}:{} to {} · client {}, export {}",mount.client,mount.specification.server,mount.specification.root,mount.specification.mountpoint,access_label(mount.specification.access),access_label(mount.binding.access));m.state_text.push(text.clone());ui.label(text);}
 ui.label(RichText::new("Saved text equality is not proof of applied policy; Check evaluates actual current and independent new-mount outcomes.").small());
 egui::CollapsingHeader::new("Inventory and identities · Optional").id_salt("nfs-inventory").show(ui,|ui|{for server in &s.inventory().servers{ui.add(egui::Label::new(format!("Server {} · IPv4 {}",server.id,server.address)).wrap().selectable(true));}for client in &s.inventory().clients{ui.add(egui::Label::new(format!("Client {} · IPv4 {}",client.id,client.address)).wrap().selectable(true));ui.add(egui::Label::new(format!("Mountpoints: {}; numeric profiles: {}",client.mountpoints.join(", "),client.profiles.join(", "))).wrap().selectable(true));}for profile in &s.inventory().profiles{ui.add(egui::Label::new(format!("{} ({}) · UID {}, GID {}, additional groups {:?}",profile.id,profile.label,profile.uid,profile.gid,profile.groups)).wrap().selectable(true));}for root in &s.inventory().roots{ui.label(format!("{}:{}",root.server,root.path));for inode in root.directories.iter().chain(&root.files){ui.label(format!("{} · {:o} · UID {} GID {}",inode.path,inode.mode,inode.uid,inode.gid));}}});});
    m.current = Some(r.response.rect);
}

fn view(a: &Attempt) -> teaching::View {
    if a.challenge {
        teaching::View::Assignment
    } else {
        a.workspace
            .as_ref()
            .and_then(|w| w.nfs_learning.as_ref())
            .map_or(teaching::View::Assignment, |l| l.view)
    }
}
fn page(
    ui: &mut egui::Ui,
    s: &Scenario,
    a: &mut Attempt,
    p: &mut Presentation,
    m: &mut Metrics,
    active: bool,
) -> bool {
    // Capture the actual focused NFS field before ANY page widget can surrender
    // focus. Inspect borrowed pending events only; never clone their payloads or
    // change egui's focus/input policy. Exact newline Text events are deliberately
    // ignored by pinned egui; nonempty Paste remains an actual insertion request.
    let entry_focus = ui.memory(|memory| memory.focused());
    let pending_text = ui.input(|input| {
        input.events.iter().any(|event| match event {
            egui::Event::Text(text) => !text.is_empty() && text != "\n" && text != "\r",
            egui::Event::Paste(text) => !text.is_empty(),
            _ => false,
        })
    });
    let pending_target = entry_focus.filter(|id| {
        active
            && pending_text
            && (p.exports.key == Some(*id)
                || p.fstab.key == Some(*id)
                || p.owner.is_some_and(|owner| {
                    *id == owner.with("relative") || *id == owner.with("write-text")
                }))
    });
    m.nfs.heading = Some(ui.heading(&s.title).rect);
    ui.label(RichText::new(&s.objective).strong());
    ui.label(RichText::new("Supplied virtual NFS · Existing files and numeric identities only; nothing runs on the host").small());
    let old = view(a);
    let mut current = old;
    let mut dirty = false;
    let mut rejected = false;
    if s.nfs_teaching.is_some() && !a.challenge {
        ui.add_enabled_ui(active, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · Ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_value(&mut current, v, label);
                    m.nfs.tabs.push((v, r.rect));
                }
            });
        });
    }
    if current != old {
        match work::set_practice_presentation(a, s, Some(current), None) {
            Ok(()) => {
                dirty = true;
                p.owner = None;
                p.notice = None;
            }
            Err(e) => {
                p.notice = Some(work::notice(e));
                current = old;
                rejected = true;
            }
        }
    }
    if current == teaching::View::Introduction {
        if let Some(l) = &s.nfs_teaching {
            ui.label(&l.prerequisite_note);
            for c in &l.concepts {
                ui.strong(&c.title);
                ui.label(&c.text);
            }
            ui.strong("A separate example to investigate");
            ui.label(&l.example_description);
            ui.add_enabled_ui(active && !rejected, |ui| {
                if button_action(ui, &mut m.nfs, "try-example", "Try this example", true) {
                    match work::set_practice_presentation(a, s, Some(teaching::View::Example), None)
                    {
                        Ok(()) => {
                            dirty = true;
                            p.owner = None;
                        }
                        Err(e) => p.notice = Some(work::notice(e)),
                    }
                }
            });
        }
        if dirty {
            p.memo.evaluate(a, s, false);
        }
        return dirty;
    }
    let practice = current == teaching::View::Example;
    let scope = egui::Id::new(("nfs", &s.id, s.version, a.visited, practice, p.epoch));
    // Prepass admission is fresh or an exact complete-key hit, not a persisted checked flag.
    if !p.memo.admitted(&s.id) {
        ui.colored_label(palette(ui).error,"This NFS context or saved metadata cannot be opened safely; existing work is retained.");
        return dirty;
    }
    let Some((simulation, session)) = selected(a, practice) else {
        ui.label("Saved capacity prevented preparation. Export your retained work before choosing a reset.");
        return dirty;
    };
    p.bind(scope, simulation, session);
    let mut edits = Edits {
        rejected,
        pending_target,
        ..Default::default()
    };
    if practice && let Some(l) = &s.nfs_teaching {
        ui.strong("Separate example · Ungraded");
        ui.label(&l.example_description);
        ui.add_enabled_ui(active,|ui|{let selected=a.workspace.as_ref().unwrap().nfs_learning.as_ref().unwrap().selected_example;let mut choice=selected;controls::form_row(ui,"Example action",|ui,width|{let combo=controls::dropdown(ui,scope.with("example-action"),width).selected_text(&l.example_actions[selected].title).show_ui(ui,|ui|{controls::fit_popup_contents(ui);for(n,step)in l.example_actions.iter().enumerate(){let r=ui.selectable_value(&mut choice,n,&step.title);m.nfs.choices.push((format!("example-action:{n}"),r.rect));}m.nfs.popup=Some(("example-action".into(),controls::popup_content_clip(ui)));});m.nfs.controls.push(("example-action".into(),combo.response.rect,combo.response.enabled(),ui.clip_rect()));});if choice!=selected{match work::set_practice_presentation(a,s,None,Some(choice)){Ok(())=>dirty=true,Err(e)=>{edits.rejected=true;p.notice=Some(work::notice(e));}}}
   let chosen=a.workspace.as_ref().unwrap().nfs_learning.as_ref().unwrap().selected_example;ui.label(&l.example_actions[chosen].explanation);ui.horizontal_wrapped(|ui|{queue(&mut edits.q,button_action(ui,&mut m.nfs,"fill","Fill example fields only",false),Submit::Fill);queue(&mut edits.q,button_action(ui,&mut m.nfs,"reset-example","Reset example",false),Submit::ResetExample);});
   egui::CollapsingHeader::new("Experiment and repair · Optional").id_salt(scope.with("experiment")).show(ui,|ui|{ui.label(&l.try_change.description);ui.label("Try the described change with the actual editor/actions, Check, then repair it.");});
  });
    } else if !a.challenge {
        egui::CollapsingHeader::new("Task stages and help · Optional")
            .id_salt(scope.with("help"))
            .show(ui, |ui| {
                for (n, step) in s.steps.iter().enumerate() {
                    ui.label(format!("{}. {step}", n + 1));
                }
                for hint in &s.hints {
                    ui.label(hint);
                }
                ui.label(&s.simplifications);
            });
    }
    // Consume this explicit Check shortcut before multiline editors can handle Enter.
    let focused = [
        scope.with(("exports", p.request.server.as_str())),
        scope.with(("fstab", p.request.client.as_str())),
        scope.with("relative"),
        scope.with("write-text"),
    ]
    .iter()
    .any(|id| ui.memory(|mem| mem.has_focus(*id)));
    let shortcut = active
        && focused
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter));
    let required_write = if practice {
        s.nfs_teaching.as_ref().is_some_and(|l| {
            l.definition
                .requirements
                .iter()
                .any(|r| matches!(r, work::Requirement::CurrentFileText { .. }))
        })
    } else {
        matches!(&s.goal, engine::Goal::Nfs { definition } if definition.requirements.iter().any(|r| matches!(r, work::Requirement::CurrentFileText { .. })))
    };
    ui.add_enabled_ui(active, |ui| {
        docs(ui, a, practice, p, &mut m.nfs, scope, &mut edits);
        requests(ui, a, practice, p, &mut m.nfs, scope, &mut edits);
        write_editor(ui, p, &mut m.nfs, scope, &mut edits, required_write);
    });
    ui.add_enabled_ui(active, |ui| {
        queue(
            &mut edits.q,
            button_action(ui, &mut m.nfs, "check", "Check", true) || shortcut,
            Submit::Check,
        );
    });
    m.nfs.queued = edits.q.map(Submit::key);
    edits.dropped = edits.pending_target.is_some() && !edits.received;
    edits.rejected |= edits.dropped;
    m.nfs.pending_text = edits.pending_target.is_some();
    m.nfs.received_text = edits.received;
    m.nfs.dropped_text = edits.dropped;
    if active {
        dirty |= apply_edits(a, s, practice, p, &mut edits);
    }
    let checked = active && !edits.rejected && matches!(edits.q, Some(Submit::Check));
    if active
        && !edits.rejected
        && let Some(q) = edits.q
    {
        m.nfs.submitted = Some(q.key());
        match submit(a, s, practice, p, scope, q) {
            Ok(changed) => dirty |= changed,
            Err(e) => p.notice = Some(work::notice(e)),
        }
    }
    // Refresh after the final admitted edit/action, before any state/result/completion paint.
    if dirty {
        p.memo.evaluate(a, s, false);
    }
    m.nfs.check_evaluated = checked;
    live(ui, a, practice, p, &mut m.nfs);
    if let Some(notice) = &p.notice {
        m.nfs.notice_shown = true;
        ui.add(egui::Label::new(notice).wrap());
    }
    if let Some(text) = &p.last_read {
        m.nfs.read_shown = true;
        ui.strong("Last successful read · Contents at that request");
        let scrolled = egui::ScrollArea::both()
            .id_salt(scope.with("read-result"))
            .max_height(controls::height(ui) * 3.)
            .show(ui, |ui| {
                ui.add(
                    egui::Label::new(RichText::new(text).monospace())
                        .extend()
                        .selectable(true),
                )
            });
        m.nfs.read_output = Some((
            scrolled.inner.rect,
            scrolled.inner_rect.intersect(ui.clip_rect()),
        ));
    }
    if p.checked[usize::from(practice)]
        && let Some(rows) = p.memo.rows(&s.id, practice)
    {
        let result = controls::feedback(ui, checked, |ui| {
            ui.strong("Check results");
            for (n, (ok, text)) in rows.iter().enumerate() {
                let r = ui.colored_label(
                    if *ok {
                        palette(ui).success
                    } else {
                        palette(ui).error
                    },
                    text,
                );
                m.nfs.rows.push((n, *ok, r.rect));
                m.nfs.row_clips.push((n, ui.clip_rect()));
            }
            let pass = !rows.is_empty() && rows.iter().all(|r| r.0);
            if pass {
                ui.strong(if practice {
                    "Example checks pass · Assignment is separate"
                } else {
                    "Assignment complete"
                });
            }
            m.completion = pass && !practice;
        });
        m.nfs.result = Some(result.response.rect);
    }
    dirty
}
pub(super) fn solved_with_memos(
    nfs: &memo::Memo,
    rpm: &rpm_packages::memo::Memo,
    nat: &nat_workspace::memo::Memo,
    a: &Attempt,
    s: &Scenario,
) -> bool {
    if matches!(s.initial, engine::State::Nfs { .. }) {
        nfs.prepared(&s.id)
            .unwrap_or_else(|| progress::current_solved(a, s))
    } else {
        rpm_packages::solved_with_memos(rpm, nat, a, s)
    }
}
impl LabApp {
    pub(super) fn nfs_visible_solved(&self, a: &Attempt, s: &Scenario) -> bool {
        solved_with_memos(
            &self.nfs_presentation.memo,
            &self.rpm_presentation.memo,
            &self.nat_presentation.memo,
            a,
            s,
        )
    }
    pub(super) fn nfs_prepass(&mut self) {
        let sources = [
            self.scenarios.as_slice(),
            if self.developer && self.synthetic {
                self.synthetic_catalog.as_slice()
            } else {
                &[]
            },
        ];
        self.nfs_presentation.memo.retain(|id| {
            self.progress.attempts.contains_key(id)
                && sources.iter().any(|ss| {
                    ss.iter()
                        .any(|s| s.id == id && matches!(s.initial, engine::State::Nfs { .. }))
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
                        && matches!(s.initial, engine::State::Nfs { .. })
                        && let Some(a) = self.progress.attempts.get(&s.id)
                    {
                        self.nfs_presentation.memo.evaluate(a, s, false);
                    }
                }
            }
        }
    }
    pub(super) fn prepare_nfs(a: &mut Attempt, s: &Scenario, new: bool) {
        if matches!(s.initial, engine::State::Nfs { .. }) {
            let _ = learn::prepare(a, s, new); /* An admitted but full old Attempt remains readable; no panic/mutation on refusal. */
        }
    }
    pub(super) fn nfs_page(&mut self, ui: &mut egui::Ui, i: usize) {
        self.workspace_header(ui, i);
        if let Some(notice) = &self.notice {
            ui.label(work::notice(notice));
        }
        let active = self.selected == Some(i)
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let source = &self.scenarios[i];
        let a = self.progress.attempts.get_mut(&source.id).unwrap();
        if self.nfs_presentation.memo.prepared(&source.id).is_none() {
            self.nfs_presentation.memo.evaluate(a, source, false);
        }
        let dirty = card(ui)
            .show(ui, |ui| {
                page(
                    ui,
                    source,
                    a,
                    &mut self.nfs_presentation,
                    &mut self.metrics,
                    active,
                )
            })
            .inner;
        self.dirty |= dirty;
        self.checked =
            self.nfs_presentation.checked[usize::from(view(a) == teaching::View::Example)];
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
    }
}

#[cfg(test)]
mod buffer_storage_tests {
    use super::*;
    #[test]
    fn bounded_buffer_growth_and_owned_shadows_fit_reserved_storage() {
        let mut text = "x".repeat(core::MAX_DOCUMENT - 1);
        let mut b = Buffer {
            value: &mut text,
            cap: core::MAX_DOCUMENT,
            before: None,
            rejected: false,
            received: false,
        };
        assert_eq!(
            egui::TextBuffer::insert_text(&mut b, "x", core::MAX_DOCUMENT - 1),
            1
        );
        let edit = b.finish();
        assert!(edit.before.is_some());
        assert!(text.capacity() <= core::MAX_DOCUMENT);
        let current = 2 * text.capacity();
        let shadows = 2 * edit.before.as_ref().unwrap().capacity();
        let request = 2 * (32 + 32 + 160 + 32 + 160 + 4096);
        let request_shadow = 4096;
        let last_read = 4096;
        let notice = 512;
        let scope_and_structs = 4096;
        assert!(
            current + shadows + request + request_shadow + last_read + notice + scope_and_structs
                <= memo::EDITOR_RESERVE
        );
        let before = text.clone();
        let mut b = Buffer {
            value: &mut text,
            cap: core::MAX_DOCUMENT,
            before: None,
            rejected: false,
            received: false,
        };
        egui::TextBuffer::delete_char_range(&mut b, 0..core::MAX_DOCUMENT);
        assert_eq!(
            egui::TextBuffer::insert_text(&mut b, &"x".repeat(core::MAX_DOCUMENT + 1), 0),
            0
        );
        assert!(b.finish().rejected);
        assert_eq!(text, before);
    }
}
