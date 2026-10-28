//! Native L2 LACP configuration workspace; all actions stay inside the typed simulator.
use super::*;
use dlms_learning_lab::{
    etherchannel as core, etherchannel_teaching as learn, etherchannel_workspace as work,
};
pub(super) mod memo;

#[derive(Default)]
pub(super) struct EtherMetrics {
    pub fields: Vec<(String, egui::Rect, egui::Id, bool, egui::Rect)>,
    pub controls: Vec<(String, egui::Rect, bool, egui::Rect)>,
    pub choices: Vec<(String, egui::Rect)>,
    pub control_ids: Vec<(String, egui::Id)>,
    pub expanders: Vec<(String, egui::Rect, bool, egui::Rect)>,
    pub popup: Option<(String, egui::Rect)>,
    pub tabs: Vec<(teaching::View, egui::Rect)>,
    pub rows: Vec<(usize, bool, egui::Rect)>,
    pub row_clips: Vec<(usize, egui::Rect)>,
    pub detail_rows: Vec<(usize, bool, egui::Rect)>,
    pub details_header: Option<egui::Rect>,
    pub compact_current: Option<egui::Rect>,
    pub compact_retired: Option<egui::Rect>,
    pub compact_preserved: Option<egui::Rect>,
    pub first_failure: Option<(usize, egui::Rect)>,
    pub result: Option<egui::Rect>,
    pub reasoning_header: Option<egui::Rect>,
    pub reasoning_text: Option<egui::Rect>,
    pub next_lab: Option<(String, egui::Rect)>,
    pub columns: bool,
    pub rejected: bool,
    pub applied: bool,
    pub checked: bool,
}
#[derive(Default)]
pub(super) struct Presentation {
    pub memo: memo::Memo,
    owner: Option<egui::Id>,
    source: Option<Scenario>,
    device: String,
    epoch: u64,
    draft: Option<work::Draft>,
    field_ids: Vec<egui::Id>,
    notice: Option<String>,
}
impl Presentation {
    #[cfg(test)]
    pub(super) fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }
    fn clear_fields(&mut self, ctx: &egui::Context) {
        let ids = std::mem::take(&mut self.field_ids);
        ctx.data_mut(|d| {
            for id in ids {
                d.remove::<egui::text_edit::TextEditState>(id);
            }
        });
        self.epoch = self.epoch.wrapping_add(1);
    }
    pub fn clear(&mut self) {
        self.owner = None;
        self.source = None;
        self.draft = None;
        self.device.clear();
        self.notice = None;
        self.memo.clear();
        // Retain the small ID list until the next bind can remove actual Context states.
    }
    fn bind(
        &mut self,
        ctx: &egui::Context,
        owner: egui::Id,
        source: &Scenario,
        draft: &work::Draft,
    ) -> Result<(), String> {
        let source_changed = self
            .source
            .as_ref()
            .is_none_or(|old| !memo::same_source(old, source));
        let owner_changed = self.owner != Some(owner) || source_changed;
        if owner_changed || self.draft.as_ref() != Some(draft) {
            if memo::charged(draft, 512 * 1024).is_none()
                || (source_changed && memo::charged(source, memo::EDITOR_RESERVE / 2).is_none())
            {
                return Err("This source/draft cannot fit the bounded editing workspace; existing work is retained".into());
            }
            self.clear_fields(ctx);
            self.owner = Some(owner);
            self.device = draft.selected_device.clone();
            self.draft = Some(draft.clone());
            if source_changed {
                self.source = Some(source.clone());
            }
            if owner_changed {
                self.notice = None;
            }
        } else if self.device != draft.selected_device {
            self.clear_fields(ctx);
            self.device = draft.selected_device.clone();
        }
        Ok(())
    }
    fn field(&mut self, ctx: &egui::Context, id: egui::Id) -> bool {
        if self.field_ids.contains(&id) {
            return true;
        }
        if self.field_ids.len() >= 48 {
            return false;
        }
        let mut state = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
        state.set_undoer(egui::util::undoer::Undoer::with_settings(
            egui::util::undoer::Settings {
                max_undos: 16,
                ..Default::default()
            },
        ));
        state.store(ctx, id);
        self.field_ids.push(id);
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Row {
    Device,
    Group(String),
    Port(String),
    Selection,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Property {
    Vlans,
    Up,
    Rate,
    Membership,
    Lacp,
    Duplex,
    Mode,
    Access,
    Native,
    Allowed,
    SelectedDevice,
    SelectedGroup,
    SelectedService,
    SelectedCondition,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Key {
    device: String,
    row: Row,
    property: Property,
}
impl Key {
    fn name(&self) -> String {
        format!("{}:{:?}:{:?}", self.device, self.row, self.property)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    Text(String),
    Up(bool),
    Rate(core::Rate),
    Membership(Option<String>),
    Lacp(core::LacpMode),
    Duplex(core::Duplex),
    Mode(work::ProfileMode),
}
fn profile<'a>(d: &'a work::Draft, k: &Key) -> &'a work::ProfileDraft {
    match &k.row {
        Row::Group(g) => &d.devices[&k.device].groups[g].profile,
        Row::Port(p) => &d.devices[&k.device].ports[p].profile,
        _ => unreachable!("Typed profile field"),
    }
}
fn profile_mut<'a>(d: &'a mut work::Draft, k: &Key) -> &'a mut work::ProfileDraft {
    match &k.row {
        Row::Group(g) => {
            &mut d
                .devices
                .get_mut(&k.device)
                .unwrap()
                .groups
                .get_mut(g)
                .unwrap()
                .profile
        }
        Row::Port(p) => {
            &mut d
                .devices
                .get_mut(&k.device)
                .unwrap()
                .ports
                .get_mut(p)
                .unwrap()
                .profile
        }
        _ => unreachable!("Typed profile field"),
    }
}
fn raw_mut<'a>(d: &'a mut work::Draft, k: &Key) -> &'a mut String {
    match k.property {
        Property::Vlans => &mut d.devices.get_mut(&k.device).unwrap().vlans,
        Property::Access => &mut profile_mut(d, k).access,
        Property::Native => &mut profile_mut(d, k).native,
        Property::Allowed => &mut profile_mut(d, k).allowed,
        Property::SelectedDevice => &mut d.selected_device,
        Property::SelectedGroup => &mut d.selected_group,
        Property::SelectedService => &mut d.selected_service,
        Property::SelectedCondition => &mut d.selected_condition,
        _ => unreachable!("Typed text field"),
    }
}
fn value(d: &work::Draft, k: &Key) -> Value {
    match k.property {
        Property::Up => Value::Up(match &k.row {
            Row::Group(g) => d.devices[&k.device].groups[g].up,
            Row::Port(p) => d.devices[&k.device].ports[p].up,
            _ => unreachable!(),
        }),
        Property::Rate => Value::Rate(match &k.row {
            Row::Group(g) => d.devices[&k.device].groups[g].rate,
            Row::Port(p) => d.devices[&k.device].ports[p].rate,
            _ => unreachable!(),
        }),
        Property::Membership => {
            let Row::Port(p) = &k.row else { unreachable!() };
            Value::Membership(d.devices[&k.device].ports[p].group.clone())
        }
        Property::Lacp => {
            let Row::Port(p) = &k.row else { unreachable!() };
            Value::Lacp(d.devices[&k.device].ports[p].mode)
        }
        Property::Duplex => {
            let Row::Port(p) = &k.row else { unreachable!() };
            Value::Duplex(d.devices[&k.device].ports[p].duplex)
        }
        Property::Mode => Value::Mode(profile(d, k).mode),
        Property::Vlans => Value::Text(d.devices[&k.device].vlans.clone()),
        Property::Access => Value::Text(profile(d, k).access.clone()),
        Property::Native => Value::Text(profile(d, k).native.clone()),
        Property::Allowed => Value::Text(profile(d, k).allowed.clone()),
        Property::SelectedDevice => Value::Text(d.selected_device.clone()),
        Property::SelectedGroup => Value::Text(d.selected_group.clone()),
        Property::SelectedService => Value::Text(d.selected_service.clone()),
        Property::SelectedCondition => Value::Text(d.selected_condition.clone()),
    }
}
fn put(d: &mut work::Draft, k: &Key, v: Value) {
    match v {
        Value::Text(v) => *raw_mut(d, k) = v,
        Value::Up(v) => match &k.row {
            Row::Group(g) => {
                d.devices
                    .get_mut(&k.device)
                    .unwrap()
                    .groups
                    .get_mut(g)
                    .unwrap()
                    .up = v
            }
            Row::Port(p) => {
                d.devices
                    .get_mut(&k.device)
                    .unwrap()
                    .ports
                    .get_mut(p)
                    .unwrap()
                    .up = v
            }
            _ => unreachable!(),
        },
        Value::Rate(v) => match &k.row {
            Row::Group(g) => {
                d.devices
                    .get_mut(&k.device)
                    .unwrap()
                    .groups
                    .get_mut(g)
                    .unwrap()
                    .rate = v
            }
            Row::Port(p) => {
                d.devices
                    .get_mut(&k.device)
                    .unwrap()
                    .ports
                    .get_mut(p)
                    .unwrap()
                    .rate = v
            }
            _ => unreachable!(),
        },
        Value::Membership(v) => {
            let Row::Port(p) = &k.row else { unreachable!() };
            d.devices
                .get_mut(&k.device)
                .unwrap()
                .ports
                .get_mut(p)
                .unwrap()
                .group = v;
        }
        Value::Lacp(v) => {
            let Row::Port(p) = &k.row else { unreachable!() };
            d.devices
                .get_mut(&k.device)
                .unwrap()
                .ports
                .get_mut(p)
                .unwrap()
                .mode = v;
        }
        Value::Duplex(v) => {
            let Row::Port(p) = &k.row else { unreachable!() };
            d.devices
                .get_mut(&k.device)
                .unwrap()
                .ports
                .get_mut(p)
                .unwrap()
                .duplex = v;
        }
        Value::Mode(v) => profile_mut(d, k).mode = v,
    }
}
#[derive(Default)]
struct Edits {
    old: Vec<(Key, Value)>,
    rejected: bool,
    rejection: Option<&'static str>,
    received: bool,
    pending: Option<egui::Id>,
    apply: bool,
    check: bool,
    fill: bool,
    reset_example: bool,
}
impl Edits {
    fn change(&mut self, d: &mut work::Draft, k: Key, v: Value) {
        let old = value(d, &k);
        if old == v {
            return;
        }
        if !self.old.iter().any(|(key, _)| *key == k) {
            self.old.push((k.clone(), old));
        }
        put(d, &k, v);
    }
    fn rollback(&mut self, d: &mut work::Draft) {
        for (k, v) in self.old.drain(..).rev() {
            put(d, &k, v);
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
            }
        } else {
            Edit {
                before: self
                    .before
                    .filter(|old| old.as_str() != self.value.as_str()),
                rejected: false,
                received: self.received,
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
fn selected<'a>(
    a: &'a Attempt,
    s: &'a Scenario,
    practice: bool,
) -> Option<(&'a core::State, &'a work::Draft, &'a work::Definition)> {
    let w = a.workspace.as_ref()?;
    if practice {
        let l = w.etherchannel_learning.as_ref()?;
        Some((&l.simulation, &l.draft, &l.context))
    } else {
        let (engine::State::EtherChannel { simulation }, engine::Goal::EtherChannel { definition }) =
            (&a.state, &s.goal)
        else {
            return None;
        };
        Some((simulation, w.etherchannel_draft.as_ref()?, definition))
    }
}
fn key(device: &str, row: Row, property: Property) -> Key {
    Key {
        device: device.into(),
        row,
        property,
    }
}
struct Editor<'a> {
    p: &'a mut Presentation,
    e: &'a mut Edits,
    m: &'a mut EtherMetrics,
    scope: egui::Id,
}
fn field(
    ui: &mut egui::Ui,
    label: &str,
    width: f32,
    k: Key,
    d: &mut work::Draft,
    cx: &mut Editor<'_>,
) {
    let id = cx.scope.with((cx.p.epoch, k.name()));
    if !cx.p.field(ui.ctx(), id) {
        cx.e.rejected = true;
        cx.e.rejection = Some(
            "The bounded editing workspace is full. Reopen this section and retry; nothing was applied or checked.",
        );
        return;
    }
    let cap = if matches!(k.property, Property::Allowed | Property::Vlans) {
        384
    } else {
        8
    };
    let mut b = Buffer {
        value: raw_mut(d, &k),
        cap,
        before: None,
        rejected: false,
        received: false,
    };
    let response = ui.add_sized(
        [width.max(1.), controls::height(ui)],
        controls::singleline(ui, &mut b).id(id),
    );
    name_control(&response, label);
    let edit = b.finish();
    if cx.e.pending == Some(response.id) {
        cx.e.received |= edit.received;
    }
    cx.e.rejected |= edit.rejected;
    if edit.rejected {
        cx.e.rejection = Some(
            "This edit is too long or contains a NUL character. Shorten it and retry; nothing was applied or checked.",
        );
    }
    if let Some(old) = edit.before
        && !cx.e.old.iter().any(|(oldk, _)| *oldk == k)
    {
        cx.e.old.push((k.clone(), Value::Text(old)));
    }
    cx.m.fields.push((
        k.name(),
        response.rect,
        response.id,
        response.enabled(),
        ui.clip_rect(),
    ));
}
fn dropdown(
    ui: &mut egui::Ui,
    label: &str,
    k: Key,
    d: &mut work::Draft,
    cx: &mut Editor<'_>,
    choices: Vec<(String, Value)>,
) {
    let old = value(d, &k);
    let mut current = old.clone();
    let name = k.name();
    let selected = choices
        .iter()
        .find(|(_, v)| *v == old)
        .map(|(n, _)| n.as_str())
        .unwrap_or("Choose supplied value");
    let r = controls::dropdown(ui, cx.scope.with(&name), ui.available_width())
        .selected_text(selected)
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            for (label, v) in choices {
                let r = ui.selectable_value(&mut current, v, &label);
                cx.m.choices.push((format!("{name}:{label}"), r.rect));
            }
            cx.m.popup = Some((name.clone(), controls::popup_content_clip(ui)));
        })
        .response;
    name_control(&r, label);
    cx.m.control_ids.push((name.clone(), r.id));
    cx.m.controls
        .push((name, r.rect, r.enabled(), ui.clip_rect()));
    if current != old {
        cx.e.change(d, k, current);
    }
}
fn up(ui: &mut egui::Ui, label: &str, k: Key, d: &mut work::Draft, cx: &mut Editor<'_>) {
    let Value::Up(mut yes) = value(d, &k) else {
        unreachable!()
    };
    let r = ui.checkbox(&mut yes, "Up");
    name_control(&r, label);
    cx.m.control_ids.push((k.name(), r.id));
    cx.m.controls
        .push((k.name(), r.rect, r.enabled(), ui.clip_rect()));
    if r.changed() {
        cx.e.change(d, k, Value::Up(yes));
    }
}
fn action(ui: &mut egui::Ui, m: &mut EtherMetrics, id: &str, label: &str, main: bool) -> bool {
    let r = if main {
        ui.add(primary(ui, label))
    } else {
        ui.add(button(label))
    };
    name_control(&r, label);
    m.control_ids.push((id.into(), r.id));
    m.controls
        .push((id.into(), r.rect, r.enabled(), ui.clip_rect()));
    r.clicked()
}
fn rates() -> Vec<(String, Value)> {
    [
        ("100 Mb/s", core::Rate::R100),
        ("1 Gb/s", core::Rate::R1000),
        ("10 Gb/s", core::Rate::R10000),
    ]
    .into_iter()
    .map(|(n, v)| (n.into(), Value::Rate(v)))
    .collect()
}
fn profile_summary(p: &work::ProfileDraft) -> String {
    match p.mode {
        work::ProfileMode::Access => format!("Access · VLAN {}", p.access),
        work::ProfileMode::Trunk => format!(
            "Trunk · native {} · {} allowed values",
            p.native,
            p.allowed.split(',').count()
        ),
    }
}
fn profile_fields(
    ui: &mut egui::Ui,
    device: &str,
    row: Row,
    d: &mut work::Draft,
    cx: &mut Editor<'_>,
) {
    controls::form_row(ui, "Link type", |ui, _| {
        dropdown(
            ui,
            "Access or trunk profile",
            key(device, row.clone(), Property::Mode),
            d,
            cx,
            vec![
                ("Access".into(), Value::Mode(work::ProfileMode::Access)),
                ("Trunk".into(), Value::Mode(work::ProfileMode::Trunk)),
            ],
        )
    });
    let mode = profile(d, &key(device, row.clone(), Property::Mode)).mode;
    match mode {
        work::ProfileMode::Access => {
            controls::form_row(ui, "Access VLAN", |ui, width| {
                field(
                    ui,
                    "Access VLAN",
                    width,
                    key(device, row.clone(), Property::Access),
                    d,
                    cx,
                )
            });
        }
        work::ProfileMode::Trunk => {
            controls::form_row(ui, "Native VLAN", |ui, width| {
                field(
                    ui,
                    "Native VLAN",
                    width,
                    key(device, row.clone(), Property::Native),
                    d,
                    cx,
                )
            });
            controls::form_row(ui, "Allowed VLANs", |ui, width| {
                field(
                    ui,
                    "Comma-separated allowed VLANs",
                    width,
                    key(device, row.clone(), Property::Allowed),
                    d,
                    cx,
                )
            });
        }
    }
}
fn peer(s: &core::State, device: &str, port: &str) -> String {
    s.topology()
        .cables
        .iter()
        .find_map(|(id, c)| {
            if c.a.device == device && c.a.port == port {
                Some(format!("{}:{} · {}", c.b.device, c.b.port, id))
            } else if c.b.device == device && c.b.port == port {
                Some(format!("{}:{} · {}", c.a.device, c.a.port, id))
            } else {
                None
            }
        })
        .unwrap_or_else(|| "No cable supplied".into())
}
fn group_rows(ui: &mut egui::Ui, device: &str, d: &mut work::Draft, cx: &mut Editor<'_>) {
    ui.strong("Logical groups");
    let unit = ui.text_style_height(&egui::TextStyle::Body);
    let widths = [unit * 5. + 24., unit * 3. + 24., unit * 7. + 24.];
    let wide = ui.available_width() >= widths.iter().sum::<f32>() + unit * 12. + 3. * controls::GAP;
    if wide {
        ui.horizontal(|ui| {
            for (label, width) in ["Group", "State", "Line rate"].into_iter().zip(widths) {
                controls::label(ui, RichText::new(label).strong(), width);
            }
            ui.strong("VLAN profile");
        });
    }
    let groups: Vec<_> = d.devices[device].groups.keys().cloned().collect();
    for group in groups {
        let row = Row::Group(group.clone());
        if wide {
            ui.horizontal(|ui| {
                controls::label(ui, &group, widths[0]);
                controls::cell(ui, widths[1], |ui| {
                    up(
                        ui,
                        "Group administrative state",
                        key(device, row.clone(), Property::Up),
                        d,
                        cx,
                    )
                });
                controls::cell(ui, widths[2], |ui| {
                    dropdown(
                        ui,
                        "Group line rate",
                        key(device, row.clone(), Property::Rate),
                        d,
                        cx,
                        rates(),
                    )
                });
                ui.add(
                    egui::Label::new(profile_summary(&d.devices[device].groups[&group].profile))
                        .wrap(),
                );
            });
        } else {
            ui.strong(&group);
            controls::form_row(ui, "State", |ui, _| {
                up(
                    ui,
                    "Group administrative state",
                    key(device, row.clone(), Property::Up),
                    d,
                    cx,
                )
            });
            controls::form_row(ui, "Line rate", |ui, _| {
                dropdown(
                    ui,
                    "Group line rate",
                    key(device, row.clone(), Property::Rate),
                    d,
                    cx,
                    rates(),
                )
            });
            ui.label(profile_summary(&d.devices[device].groups[&group].profile));
        }
        let details = controls::collapsing_header(ui, &format!("Edit VLAN profile · {group}"))
            .id_salt(cx.scope.with((device, &group, "profile")))
            .show(ui, |ui| profile_fields(ui, device, row, d, cx));
        cx.m.expanders.push((
            format!("{device}:Group({group:?}):Details"),
            details.header_response.rect,
            details.header_response.enabled(),
            ui.clip_rect(),
        ));
    }
}
fn port_rows(
    ui: &mut egui::Ui,
    s: &core::State,
    device: &str,
    d: &mut work::Draft,
    cx: &mut Editor<'_>,
) {
    ui.strong("Physical members");
    let unit = ui.text_style_height(&egui::TextStyle::Body);
    let widths = [
        unit * 11. + 24.,
        unit * 3. + 24.,
        unit * 7. + 24.,
        unit * 7. + 24.,
    ];
    let wide = ui.available_width() >= widths.iter().sum::<f32>() + 3. * controls::GAP;
    cx.m.columns = wide;
    if wide {
        ui.horizontal(|ui| {
            for (label, width) in ["Port / physical peer", "State", "Local group", "LACP mode"]
                .into_iter()
                .zip(widths)
            {
                controls::label(ui, RichText::new(label).strong(), width);
            }
        });
    }
    let ports: Vec<_> = d.devices[device].ports.keys().cloned().collect();
    for port in ports {
        let row = Row::Port(port.clone());
        let memberships = || {
            std::iter::once(("Not bundled".into(), Value::Membership(None)))
                .chain(
                    s.topology().devices[device]
                        .groups
                        .keys()
                        .map(|g| (g.clone(), Value::Membership(Some(g.clone())))),
                )
                .collect()
        };
        let modes = || {
            vec![
                ("Active".into(), Value::Lacp(core::LacpMode::Active)),
                ("Passive".into(), Value::Lacp(core::LacpMode::Passive)),
            ]
        };
        if wide {
            ui.horizontal(|ui| {
                controls::cell(ui, widths[0], |ui| {
                    ui.vertical(|ui| {
                        ui.strong(&port);
                        ui.add(
                            egui::Label::new(RichText::new(peer(s, device, &port)).small()).wrap(),
                        );
                    });
                });
                controls::cell(ui, widths[1], |ui| {
                    up(
                        ui,
                        "Port administrative state",
                        key(device, row.clone(), Property::Up),
                        d,
                        cx,
                    )
                });
                controls::cell(ui, widths[2], |ui| {
                    dropdown(
                        ui,
                        "Local member group",
                        key(device, row.clone(), Property::Membership),
                        d,
                        cx,
                        memberships(),
                    )
                });
                controls::cell(ui, widths[3], |ui| {
                    dropdown(
                        ui,
                        "LACP initiation mode",
                        key(device, row.clone(), Property::Lacp),
                        d,
                        cx,
                        modes(),
                    )
                });
            });
        } else {
            ui.strong(&port);
            ui.label(peer(s, device, &port));
            controls::form_row(ui, "State", |ui, _| {
                up(
                    ui,
                    "Port administrative state",
                    key(device, row.clone(), Property::Up),
                    d,
                    cx,
                )
            });
            controls::form_row(ui, "Local group", |ui, _| {
                dropdown(
                    ui,
                    "Local member group",
                    key(device, row.clone(), Property::Membership),
                    d,
                    cx,
                    memberships(),
                )
            });
            controls::form_row(ui, "LACP mode", |ui, _| {
                dropdown(
                    ui,
                    "LACP initiation mode",
                    key(device, row.clone(), Property::Lacp),
                    d,
                    cx,
                    modes(),
                )
            });
        }
        let details = controls::collapsing_header(ui, &format!("Rate, duplex & VLANs · {port}"))
            .id_salt(cx.scope.with((device, &port, "details")))
            .show(ui, |ui| {
                let supported = s.topology().devices[device].ports[&port]
                    .rates
                    .iter()
                    .map(|r| {
                        let name = match r {
                            core::Rate::R100 => "100 Mb/s",
                            core::Rate::R1000 => "1 Gb/s",
                            core::Rate::R10000 => "10 Gb/s",
                        };
                        (name.into(), Value::Rate(*r))
                    })
                    .collect();
                controls::form_row(ui, "Line rate", |ui, _| {
                    dropdown(
                        ui,
                        "Port line rate",
                        key(device, row.clone(), Property::Rate),
                        d,
                        cx,
                        supported,
                    )
                });
                controls::form_row(ui, "Duplex", |ui, _| {
                    dropdown(
                        ui,
                        "Port duplex",
                        key(device, row.clone(), Property::Duplex),
                        d,
                        cx,
                        vec![
                            ("Full duplex".into(), Value::Duplex(core::Duplex::Full)),
                            ("Half duplex".into(), Value::Duplex(core::Duplex::Half)),
                        ],
                    )
                });
                profile_fields(ui, device, row, d, cx);
            });
        cx.m.expanders.push((
            format!("{device}:Port({port:?}):Details"),
            details.header_response.rect,
            details.header_response.enabled(),
            ui.clip_rect(),
        ));
    }
}
fn editors(ui: &mut egui::Ui, s: &core::State, d: &mut work::Draft, cx: &mut Editor<'_>) {
    let before = d.selected_device.clone();
    let chosen = if before.is_empty() {
        d.devices.keys().next().cloned().unwrap_or_default()
    } else {
        before.clone()
    };
    if before.is_empty() {
        cx.e.change(
            d,
            key("", Row::Selection, Property::SelectedDevice),
            Value::Text(chosen),
        );
    }
    let devices = d
        .devices
        .keys()
        .map(|n| (n.clone(), Value::Text(n.clone())))
        .collect();
    controls::form_row(ui, "Configure switch", |ui, _| {
        dropdown(
            ui,
            "Configure switch",
            key("", Row::Selection, Property::SelectedDevice),
            d,
            cx,
            devices,
        )
    });
    if d.selected_device != before {
        cx.e.change(
            d,
            key("", Row::Selection, Property::SelectedGroup),
            Value::Text(String::new()),
        );
        cx.p.clear_fields(ui.ctx());
        cx.p.device = d.selected_device.clone();
    }
    let device = d.selected_device.clone();
    controls::form_row(ui, "Active VLANs", |ui, width| {
        field(
            ui,
            "Active VLANs on this switch",
            width,
            key(&device, Row::Device, Property::Vlans),
            d,
            cx,
        )
    });
    group_rows(ui, &device, d, cx);
    ui.add_space(controls::GAP);
    port_rows(ui, s, &device, d, cx);
}
fn view(a: &Attempt) -> teaching::View {
    if a.challenge {
        teaching::View::Assignment
    } else {
        a.workspace
            .as_ref()
            .and_then(|w| w.etherchannel_learning.as_ref())
            .map(|l| l.view)
            .unwrap_or(teaching::View::Assignment)
    }
}
fn checked(a: &Attempt, practice: bool) -> bool {
    a.workspace.as_ref().is_some_and(|w| {
        if practice {
            w.etherchannel_learning.as_ref().is_some_and(|l| l.checked)
        } else {
            w.checked
        }
    })
}
fn status_cell(ui: &mut egui::Ui, m: &mut EtherMetrics, n: usize, ok: bool, label: &str) {
    let color = if ok {
        palette(ui).success
    } else {
        palette(ui).error
    };
    let response = ui.add(
        egui::Label::new(
            RichText::new(format!("{label} · {}", if ok { "Pass" } else { "Fail" })).color(color),
        )
        .wrap(),
    );
    m.rows.push((n, ok, response.rect));
    m.row_clips.push((n, ui.clip_rect()));
}
fn source_matches(source: &work::Source, at: usize, definition: &work::Definition) -> bool {
    match source {
        work::Source::Current {} => at == 0,
        work::Source::Fault { id } => {
            at > 0 && definition.faults.get(at - 1).is_some_and(|f| f.id == *id)
        }
    }
}
fn compact_checks(
    ui: &mut egui::Ui,
    definition: &work::Definition,
    rows: &[(bool, String)],
    id: &str,
    practice: bool,
    p: &Presentation,
    m: &mut EtherMetrics,
) {
    if let Some((ok, _)) = rows.first() {
        status_cell(ui, m, 0, *ok, "Applied fields and supplied network");
    }
    for at in 0..=definition.faults.len() {
        let multiple_pairs = definition.requirements.iter().filter(|r| matches!(r, work::Requirement::BundlePair {source, ..} if source_matches(source, at, definition))).count() > 1;
        let title = if at == 0 {
            "Current cables".into()
        } else {
            format!(
                "Without {}",
                definition.faults[at - 1]
                    .condition
                    .down
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let group = ui.horizontal_wrapped(|ui| {
            ui.strong(title);
            for (n, requirement) in definition.requirements.iter().enumerate() {
                let Some((ok, _)) = rows.get(n + 1) else {
                    continue;
                };
                let label = match requirement {
                    work::Requirement::BundlePair {
                        source,
                        a,
                        b,
                        minimum_members,
                        ..
                    } if source_matches(source, at, definition) => {
                        let observed = p.memo.pairs(id, practice, n);
                        let count = observed
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "Unavailable".into());
                        let noun = if observed == Some(1) { "pair" } else { "pairs" };
                        if multiple_pairs {
                            format!(
                                "{}:{} ↔ {}:{}: {count} {noun}; need {minimum_members}",
                                a.device, a.group, b.device, b.group
                            )
                        } else {
                            format!("Links: {count} {noun}; need {minimum_members}")
                        }
                    }
                    work::Requirement::VlanService { source, query }
                        if source_matches(source, at, definition) =>
                    {
                        let q = definition
                            .queries
                            .iter()
                            .find(|q| q.id == *query)
                            .expect("Admitted query identity");
                        if multiple_pairs {
                            format!("VLAN {} ({query})", q.query.vlan)
                        } else {
                            format!("VLAN {}", q.query.vlan)
                        }
                    }
                    _ => continue,
                };
                status_cell(ui, m, n + 1, *ok, &label);
            }
        });
        if at == 0 {
            m.compact_current = Some(group.response.rect);
        }
    }
    if definition
        .requirements
        .iter()
        .any(|r| matches!(r, work::Requirement::RetiredMembership { .. }))
    {
        let retired = ui.horizontal_wrapped(|ui| {
            for (n, requirement) in definition.requirements.iter().enumerate() {
                if let work::Requirement::RetiredMembership { group } = requirement
                    && let Some((ok, _)) = rows.get(n + 1)
                {
                    status_cell(
                        ui,
                        m,
                        n + 1,
                        *ok,
                        &format!("Retired {}:{} empty", group.device, group.group),
                    );
                }
            }
        });
        m.compact_retired = Some(retired.response.rect);
    }
    let preserved = ui.horizontal_wrapped(|ui| {
        for (n, requirement) in definition.requirements.iter().enumerate() {
            let label = match requirement {
                work::Requirement::PreservePorts { .. } => "Working ports unchanged",
                work::Requirement::PreserveGroups { .. } => "Working groups unchanged",
                work::Requirement::PreserveVlans { .. } => "VLAN databases unchanged",
                _ => continue,
            };
            if let Some((ok, _)) = rows.get(n + 1) {
                status_cell(ui, m, n + 1, *ok, label);
            }
        }
    });
    m.compact_preserved = Some(preserved.response.rect);
    // Definition order is the grading contract; grouping changes only where
    // actual painted status cells appear, never their identities or values.
    m.rows.sort_unstable_by_key(|r| r.0);
    m.row_clips.sort_unstable_by_key(|r| r.0);
}
fn inspect(
    ui: &mut egui::Ui,
    s: &Scenario,
    a: &Attempt,
    practice: bool,
    p: &Presentation,
    m: &mut EtherMetrics,
    active: bool,
) {
    let Some((simulation, d, definition)) = selected(a, s, practice) else {
        return;
    };
    let Some(pending) = p.memo.pending(&s.id, practice) else {
        ui.colored_label(
            palette(ui).error,
            "No current admitted configuration report is available; saved work is retained.",
        );
        return;
    };
    let show_checks = checked(a, practice);
    ui.strong("Applied network · Current simulated state");
    if pending {
        ui.label("Edits are saved but have not been applied. Apply configuration to update the network; Check never applies changes for you.");
    }
    let source = if d.selected_condition.is_empty() {
        work::Source::Current {}
    } else {
        work::Source::Fault {
            id: d.selected_condition.clone(),
        }
    };
    let rows = p.memo.rows(&s.id, practice).unwrap_or(&[]);
    if let Some((n, work::Requirement::BundlePair { a, b, .. })) = definition
        .requirements
        .iter()
        .enumerate()
        .find(|(_, r)| matches!(r,work::Requirement::BundlePair{source:at,..}if *at==source))
    {
        let left = format!("Group {}", a.group);
        let right = format!("Group {}", b.group);
        let leftstate = if simulation.configuration().devices[&a.device].groups[&a.group].up {
            "Enabled"
        } else {
            "Stopped"
        };
        let rightstate = if simulation.configuration().devices[&b.device].groups[&b.group].up {
            "Enabled"
        } else {
            "Stopped"
        };
        let (ok, caption) = rows
            .get(n + 1)
            .map(|(ok, text)| (*ok, text.as_str()))
            .unwrap_or((false, "No verified observation available"));
        let short_caption = match &source {
            work::Source::Current {} => "Current cables".into(),
            work::Source::Fault { id } => format!("Cable failure view · {id}"),
        };
        controls::topology_pair(
            ui,
            [&a.device, &left, leftstate],
            [&b.device, &right, rightstate],
            if ok {
                palette(ui).success
            } else {
                palette(ui).error
            },
            if show_checks { &short_caption } else { caption },
        );
    }
    for (n, r) in definition.requirements.iter().enumerate() {
        if !show_checks
            && matches!(r,work::Requirement::VlanService{source:at,query}if *at==source&&(d.selected_service.is_empty()||query==&d.selected_service))
            && let Some((ok, text)) = rows.get(n + 1)
        {
            ui.label(format!(
                "{} {text}",
                if *ok { "Ready ·" } else { "Not ready ·" }
            ));
        }
    }
    if show_checks {
        let result = controls::feedback(ui, m.checked, |ui| {
            let pass = !rows.is_empty() && rows.iter().all(|r| r.0);
            ui.strong(if pass {
                if practice {
                    "Example checks pass · Assignment is separate"
                } else {
                    "Assignment complete"
                }
            } else {
                "Configuration needs attention"
            });
            ui.label(format!(
                "{} of {} checks pass",
                rows.iter().filter(|r| r.0).count(),
                rows.len()
            ));
            if let Some((n, (_, text))) = rows.iter().enumerate().find(|(_, r)| !r.0) {
                let response = ui.add(
                    egui::Label::new(
                        RichText::new(format!(
                            "First thing to fix: {}",
                            text.replace(" 1 participating pairs", " 1 participating pair")
                        ))
                        .color(palette(ui).error),
                    )
                    .wrap(),
                );
                m.first_failure = Some((n, response.rect));
            }
            compact_checks(ui, definition, rows, &s.id, practice, p, m);
            let details = ui
                .add_enabled_ui(active, |ui| {
                    controls::collapsing_header(ui, "Objective details · All original checks")
                        .id_salt((
                            "etherchannel-objective-details",
                            &s.id,
                            a.visited,
                            practice,
                            a.challenge,
                        ))
                        .show(ui, |ui| {
                            for (n, (ok, text)) in rows.iter().enumerate() {
                                let response = ui.add(
                                    egui::Label::new(
                                        RichText::new(format!(
                                            "{} {text}",
                                            if *ok { "Pass ·" } else { "Check ·" }
                                        ))
                                        .color(if *ok {
                                            palette(ui).success
                                        } else {
                                            palette(ui).error
                                        }),
                                    )
                                    .wrap(),
                                );
                                m.detail_rows.push((n, *ok, response.rect));
                            }
                        })
                })
                .inner;
            m.details_header = Some(details.header_response.rect);
            m.controls.push((
                "objective-details".into(),
                details.header_response.rect,
                details.header_response.enabled(),
                ui.clip_rect(),
            ));
            m.control_ids
                .push(("objective-details".into(), details.header_response.id));
            if pass && !practice {
                let explanation = ui
                    .add_enabled_ui(active, |ui| {
                        controls::collapsing_header(ui, "Why this works · Worked reasoning")
                            .id_salt(("etherchannel-reasoning", &s.id, a.visited, a.challenge))
                            .show(ui, |ui| {
                                m.reasoning_text =
                                    Some(ui.add(egui::Label::new(&s.explanation).wrap()).rect);
                            })
                    })
                    .inner;
                m.reasoning_header = Some(explanation.header_response.rect);
            }
        });
        m.result = Some(result.response.rect);
    }
}
fn inspection_choices(
    ui: &mut egui::Ui,
    definition: &work::Definition,
    d: &mut work::Draft,
    cx: &mut Editor<'_>,
) {
    ui.vertical(|ui| {
        controls::form_row(ui, "Observe", |ui, width| {
            controls::cell(ui, width, |ui| {
                dropdown(
                    ui,
                    "Observe current state or a supplied cable failure",
                    key("", Row::Selection, Property::SelectedCondition),
                    d,
                    cx,
                    std::iter::once(("Current cables".into(), Value::Text(String::new())))
                        .chain(
                            definition
                                .faults
                                .iter()
                                .map(|f| (format!("If {}", f.id), Value::Text(f.id.clone()))),
                        )
                        .collect(),
                )
            })
        });
        controls::form_row(ui, "Service", |ui, width| {
            controls::cell(ui, width, |ui| {
                dropdown(
                    ui,
                    "VLAN service to inspect",
                    key("", Row::Selection, Property::SelectedService),
                    d,
                    cx,
                    std::iter::once(("All required VLANs".into(), Value::Text(String::new())))
                        .chain(definition.queries.iter().map(|q| {
                            (
                                format!("{} · VLAN {}", q.id, q.query.vlan),
                                Value::Text(q.id.clone()),
                            )
                        }))
                        .collect(),
                )
            })
        });
    });
}
fn page(
    ui: &mut egui::Ui,
    s: &Scenario,
    a: &mut Attempt,
    p: &mut Presentation,
    m: &mut Metrics,
    interaction: (bool, Option<egui::Id>),
) -> bool {
    let (active, entry_focus) = interaction;
    let pending_text = ui.input(|input| {
        input.events.iter().any(|event| match event {
            egui::Event::Text(text) => !text.is_empty() && text != "\n" && text != "\r",
            egui::Event::Paste(text) => !text.is_empty(),
            _ => false,
        })
    });
    let pending = entry_focus.filter(|id| active && pending_text && p.field_ids.contains(id));
    let mut edits = Edits {
        pending,
        ..Default::default()
    };
    let mut dirty = false;
    ui.heading(&s.title);
    ui.label(RichText::new(&s.objective).strong());
    ui.label(
        RichText::new(
            "Simulated Layer 2 LACP · Configuration controls affect this supplied network only",
        )
        .small(),
    );
    let current = view(a);
    let mut requested = current;
    let scope = egui::Id::new((
        "etherchannel",
        &s.id,
        s.version,
        a.visited,
        current == teaching::View::Example,
    ));
    ui.add_enabled_ui(active, |ui| {
        if s.etherchannel_teaching.is_some() && !a.challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · Ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_value(&mut requested, v, label);
                    m.ether.tabs.push((v, r.rect));
                }
            });
        }
    });
    if !p.memo.admitted(&s.id) {
        ui.colored_label(palette(ui).error,"This LACP context or saved metadata cannot be opened safely; existing work is retained.");
        return false;
    }
    if current == teaching::View::Introduction {
        if let Some(lesson) = &s.etherchannel_teaching {
            ui.label(&lesson.prerequisite_note);
            for concept in &lesson.concepts {
                ui.strong(&concept.title);
                ui.label(&concept.text);
            }
            ui.strong("A separate network to try");
            ui.label(&lesson.example_description);
            ui.add_enabled_ui(active, |ui| {
                if action(ui, &mut m.ether, "try-example", "Try this example", true) {
                    requested = teaching::View::Example;
                }
            });
        }
        if active && requested != current {
            match work::set_practice_presentation(a, s, Some(requested), None, None) {
                Ok(()) => {
                    dirty = true;
                    p.owner = None;
                    p.notice = None;
                }
                Err(err) => p.notice = Some(work::notice(err)),
            }
        }
        if dirty {
            p.memo.evaluate(a, s, false);
        }
        return dirty;
    }
    let practice = current == teaching::View::Example;
    let Some((_, draft, _)) = selected(a, s, practice) else {
        ui.label("The saved workspace could not be prepared. Export your retained work before choosing a reset.");
        return false;
    };
    if let Err(err) = p.bind(ui.ctx(), scope, s, draft) {
        ui.colored_label(palette(ui).error, err);
        return false;
    }
    let mut d = p.draft.take().unwrap();
    let mut example_choice = None;
    ui.add_enabled_ui(active,|ui|{
        if practice&&let Some(lesson)=&s.etherchannel_teaching {
            ui.strong("Separate example · Ungraded");ui.label(&lesson.example_description);
            let selected=a.workspace.as_ref().unwrap().etherchannel_learning.as_ref().unwrap().selected_example;
            let mut choice=selected;
            controls::form_row(ui,"Worked action",|ui,width|{
                let response=controls::dropdown(ui,scope.with("example-action"),width).selected_text(&lesson.example_actions[selected].title).show_ui(ui,|ui|{
                    controls::fit_popup_contents(ui);for(n,step)in lesson.example_actions.iter().enumerate(){let r=ui.selectable_value(&mut choice,n,&step.title);m.ether.choices.push((format!("example-action:{n}"),r.rect));}
                }).response;
                name_control(&response,"Choose a worked action for the separate example");m.ether.controls.push(("example-action".into(),response.rect,response.enabled(),ui.clip_rect()));
            });
            if choice!=selected{example_choice=Some(choice);}
            ui.label(&lesson.example_actions[selected].explanation);
            ui.horizontal_wrapped(|ui|{
                edits.fill=action(ui,&mut m.ether,"fill","Fill example fields only",false);
                edits.reset_example=action(ui,&mut m.ether,"reset-example","Reset example",false);
            });
            ui.label("Fill changes the example's editable fields only. Inspect them, then Apply configuration and Check configuration. Assignment work stays separate.");
            controls::collapsing_header(ui, "Try a change · Optional").id_salt(scope.with("experiment")).show(ui,|ui|{ui.label(&lesson.try_change.description);});
        }
        if !practice {
            controls::collapsing_header(ui, if a.challenge{"Hints and references · Optional"}else{"Task stages and help · Optional"}).id_salt(scope.with("help")).show(ui,|ui|{
                if !a.challenge {for(n,step)in s.steps.iter().enumerate(){ui.label(format!("{}. {step}",n+1));}}
                // Deliberately opened help is available in either mode; no unsolicited solution.
                for hint in &s.hints{ui.label(hint);}
                ui.label(&s.simplifications);
                for source in &s.sources{ui.hyperlink(source);}
            });
        }
        let (simulation,_,definition)=selected(a,s,practice).unwrap();
        let mut editor=Editor {p,e:&mut edits,m:&mut m.ether,scope};
        editors(ui, simulation, &mut d, &mut editor);
        let details=controls::collapsing_header(ui, "Inspect a supplied cable failure or VLAN · Optional").id_salt(scope.with("inspect")).show(ui,|ui|inspection_choices(ui, definition, &mut d, &mut editor));
        editor.m.expanders.push(("inspect-options".into(),details.header_response.rect,details.header_response.enabled(),ui.clip_rect()));
        ui.horizontal_wrapped(|ui|{
            edits.apply=action(ui,&mut m.ether,"apply","Apply configuration",false);
            edits.check=action(ui,&mut m.ether,"check","Check configuration",true);
        });
    });
    // A delivered input event must reach the field which owned it on entry.
    // Switching panels/collapsing a focused field cannot silently check old data.
    let dropped = edits.pending.is_some() && !edits.received;
    edits.rejected |= dropped;
    if dropped {
        edits.rejection.get_or_insert("The pending text did not reach its field. Focus the visible field and retry; nothing was applied or checked.");
    }
    let navigation =
        requested != current || example_choice.is_some() || edits.fill || edits.reset_example;
    if navigation && !edits.old.is_empty() {
        edits.rejected = true;
        edits.rejection.get_or_insert("Finish this edit before changing sections or filling/resetting the example. Your previous saved work is retained.");
    }
    if !active || edits.rejected {
        edits.rollback(&mut d);
        if edits.rejected {
            p.clear_fields(ui.ctx());
            p.notice = Some(
                edits
                    .rejection
                    .unwrap_or(
                        "This edit could not be accepted. Retry; nothing was applied or checked.",
                    )
                    .into(),
            );
        }
    } else {
        let persisted = if edits.old.is_empty() {
            Ok(())
        } else if practice {
            work::set_practice_draft(a, s, &d)
        } else {
            work::set_assignment_draft(a, s, &d)
        };
        match persisted {
            Err(err) => {
                edits.rollback(&mut d);
                edits.rejected = true;
                p.clear_fields(ui.ctx());
                p.notice = Some(work::notice(err));
            }
            Ok(()) => {
                dirty |= !edits.old.is_empty();
                if !edits.old.is_empty() {
                    p.notice = None;
                }
                let mut applied = true;
                if edits.apply {
                    let result = if practice {
                        work::apply_practice(a, s)
                    } else {
                        work::apply_assignment(a, s)
                    };
                    match result {
                        Ok(()) => {
                            dirty = true;
                            m.ether.applied = true;
                            p.notice=Some("Configuration applied to the simulated network. Check configuration to verify the full assignment.".into());
                        }
                        Err(err) => {
                            applied = false;
                            p.notice = Some(work::notice(err));
                        }
                    }
                }
                if edits.check && applied {
                    let result = if practice {
                        work::set_practice_presentation(a, s, None, None, Some(true))
                    } else {
                        work::set_saved_check(a, s, true)
                    };
                    match result {
                        Ok(()) => {
                            dirty = true;
                            m.ether.checked = true;
                            p.notice = None;
                        }
                        Err(err) => p.notice = Some(work::notice(err)),
                    }
                }
                if navigation {
                    let result = if edits.fill {
                        work::fill_practice(a, s)
                    } else if edits.reset_example {
                        work::reset_practice(a, s)
                    } else {
                        work::set_practice_presentation(
                            a,
                            s,
                            (requested != current).then_some(requested),
                            example_choice,
                            None,
                        )
                    };
                    match result {
                        Ok(()) => {
                            dirty = true;
                            if edits.fill || edits.reset_example {
                                p.clear_fields(ui.ctx());
                            }
                            if requested != current {
                                p.owner = None;
                            }
                            p.notice = if edits.fill {
                                Some("Example fields filled only. Apply configuration to change the example network.".into())
                            } else {
                                None
                            };
                        }
                        Err(err) => p.notice = Some(work::notice(err)),
                    }
                }
            }
        }
    }
    p.draft = Some(d);
    m.ether.rejected = edits.rejected;
    if dirty {
        p.memo.evaluate(a, s, false);
    }
    inspect(ui, s, a, practice, p, &mut m.ether, active);
    if let Some(notice) = &p.notice {
        ui.add(egui::Label::new(notice).wrap());
    }
    m.completion = !practice && p.memo.prepared(&s.id) == Some(true) && checked(a, false);
    dirty
}
pub(super) fn solved_with_memos(
    ether: &memo::Memo,
    nfs: &nfs_workspace::memo::Memo,
    rpm: &rpm_packages::memo::Memo,
    nat: &nat_workspace::memo::Memo,
    a: &Attempt,
    s: &Scenario,
) -> bool {
    if matches!(s.initial, engine::State::EtherChannel { .. }) {
        ether
            .prepared(&s.id)
            .unwrap_or_else(|| progress::current_solved(a, s))
    } else {
        nfs_workspace::solved_with_memos(nfs, rpm, nat, a, s)
    }
}
impl LabApp {
    pub(super) fn ether_prepass(&mut self) {
        let sources = [
            self.scenarios.as_slice(),
            if self.developer && self.synthetic {
                self.synthetic_catalog.as_slice()
            } else {
                &[]
            },
        ];
        self.ether_presentation.memo.retain(|id| {
            self.progress.attempts.contains_key(id)
                && sources.iter().any(|ss| {
                    ss.iter().any(|s| {
                        s.id == id && matches!(s.initial, engine::State::EtherChannel { .. })
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
                        && matches!(s.initial, engine::State::EtherChannel { .. })
                        && let Some(a) = self.progress.attempts.get(&s.id)
                    {
                        self.ether_presentation.memo.evaluate(a, s, false);
                    }
                }
            }
        }
    }
    pub(super) fn prepare_ether(a: &mut Attempt, s: &Scenario, new: bool) {
        if matches!(s.initial, engine::State::EtherChannel { .. }) {
            let _ = learn::prepare(a, s, new);
        }
    }
    fn ether_next_unit_lab(&mut self, ui: &mut egui::Ui, i: usize) {
        let Some(unit) = self
            .units
            .iter()
            .find(|unit| unit.id == self.scenarios[i].unit)
        else {
            return;
        };
        let Some(position) = unit.labs.iter().position(|id| id == &self.scenarios[i].id) else {
            return;
        };
        // The prepass has validated every retained full input for this frame.
        // Missing admission never counts as solved; no navigation stamps credit.
        let next = unit
            .labs
            .iter()
            .skip(position + 1)
            .chain(unit.labs.iter().take(position))
            .filter_map(|id| self.scenarios.iter().position(|s| &s.id == id))
            .find(|at| {
                self.progress
                    .attempts
                    .get(&self.scenarios[*at].id)
                    .is_none_or(|a| {
                        if matches!(
                            self.scenarios[*at].initial,
                            engine::State::EtherChannel { .. }
                        ) {
                            self.ether_presentation
                                .memo
                                .prepared(&self.scenarios[*at].id)
                                != Some(true)
                        } else {
                            !self.nfs_visible_solved(a, &self.scenarios[*at])
                        }
                    })
            });
        if let Some(next) = next {
            let response = ui.link(format!("Next in unit: {}", self.scenarios[next].title));
            self.metrics.ether.next_lab = Some((self.scenarios[next].id.clone(), response.rect));
            if response.clicked() {
                self.launch(next);
            }
        } else {
            ui.label("Unit complete · All its current assignments pass.");
            if ui.link("Back to this unit").clicked() {
                self.filter.track = unit.track.clone();
                self.filter.unit = unit.id.clone();
                self.filter.topic.clear();
                self.filter.search.clear();
                self.selected = None;
                self.invalidate_apache_presentation();
                self.nat_presentation.clear();
                self.rpm_presentation.clear();
                self.nfs_presentation.clear();
                self.ether_presentation.clear();
            }
        }
    }
    pub(super) fn ether_page(&mut self, ui: &mut egui::Ui, i: usize) {
        // Observe field ownership before header controls can change focus.
        let focus = ui.memory(|m| m.focused());
        let owns_pending = focus.is_some_and(|id| self.ether_presentation.field_ids.contains(&id))
            && ui.input(|input| {
                input.events.iter().any(|event| match event {
                    egui::Event::Text(text) => !text.is_empty() && text != "\n" && text != "\r",
                    egui::Event::Paste(text) => !text.is_empty(),
                    _ => false,
                })
            });
        // Deliver a pending edit before navigation may replace its field/context.
        // This defers only the header's actions for that input frame.
        ui.add_enabled_ui(!owns_pending, |ui| self.workspace_header(ui, i));
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
        if self.ether_presentation.memo.prepared(&source.id).is_none() {
            self.ether_presentation.memo.evaluate(a, source, false);
        }
        self.dirty |= card(ui)
            .show(ui, |ui| {
                page(
                    ui,
                    source,
                    a,
                    &mut self.ether_presentation,
                    &mut self.metrics,
                    (active, focus),
                )
            })
            .inner;
        self.checked = checked(a, view(a) == teaching::View::Example);
        if self.metrics.completion {
            self.ether_next_unit_lab(ui, i);
        }
    }
}

#[cfg(test)]
mod buffer_storage_tests {
    use super::*;
    #[test]
    fn bounded_paste_rolls_back_native_replace_and_typed_frame_journal() {
        let mut value = "10,20".to_owned();
        let old = value.clone();
        let mut b = Buffer {
            value: &mut value,
            cap: 384,
            before: None,
            rejected: false,
            received: false,
        };
        egui::TextBuffer::delete_char_range(&mut b, 0..5);
        assert_eq!(
            egui::TextBuffer::insert_text(&mut b, &"x".repeat(385), 0),
            0
        );
        let edit = b.finish();
        assert!(edit.rejected && edit.received);
        assert!(edit.before.is_none());
        assert_eq!(value, old);
        let s = crate::ui::tests::etherchannel_workspace::sources().remove(0);
        let engine::State::EtherChannel { simulation } = &s.initial else {
            panic!()
        };
        let mut d = work::Draft::from_state(simulation).unwrap();
        let old = d.clone();
        let device = d.selected_device.clone();
        let mut edits = Edits::default();
        edits.change(
            &mut d,
            key(&device, Row::Device, Property::Vlans),
            Value::Text("1,10,20".into()),
        );
        edits.change(
            &mut d,
            key(&device, Row::Port("p1".into()), Property::Up),
            Value::Up(false),
        );
        edits.rejected = true;
        edits.rollback(&mut d);
        assert_eq!(d, old);
    }
    #[test]
    fn editor_source_draft_undo_and_frame_shadows_fit_explicit_reservations() {
        // At most 37 actual text fields per supplied switch; retain at most48 IDs.
        // Native Undo/Redo are each capped16, plus current/flux and generous192-byte
        // cursor/container overhead for each <=384-byte string. Invalid text never
        // enters a checkpoint. Source/draft structural charges include map storage.
        let undo_bound = 48usize * 34 * (384 + 192);
        let source_and_draft = memo::EDITOR_RESERVE / 2 + 512 * 1024;
        let field_ids = 48 * std::mem::size_of::<egui::Id>();
        assert!(source_and_draft + undo_bound + field_ids + 128 * 1024 < memo::EDITOR_RESERVE);
        for s in crate::ui::tests::etherchannel_workspace::sources() {
            let engine::State::EtherChannel { simulation } = &s.initial else {
                panic!()
            };
            let d = work::Draft::from_state(simulation).unwrap();
            assert!(memo::charged(&s, memo::EDITOR_RESERVE / 2).is_some());
            assert!(memo::charged(&d, 512 * 1024).is_some());
            let strings = d
                .devices
                .values()
                .map(|d| 1 + 3 * d.groups.len() + 3 * d.ports.len())
                .max()
                .unwrap();
            assert!(strings <= 37);
        }
    }
}
