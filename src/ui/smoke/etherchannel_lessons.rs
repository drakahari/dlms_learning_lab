//! Closed native LACP authoring acceptance; real controls only, never installed grades.
use super::*;
use dlms_learning_lab::{etherchannel as core, etherchannel_workspace as work};
use serde::Deserialize;
const FIRST: usize = 49000;
const STRIDE: usize = 200;
const LAST: usize = 78;
const ALIGN: usize = 49600;
const END: usize = 49680;
const IDS: [&str; 3] = [
    "ccna-lacp-initiation",
    "ccna-lacp-member-service",
    "ccna-lacp-resilient-handover",
];
const SCALES: [f32; 5] = [1., 1.15, 1.25, 1.5, 2.];
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Review {
    id: String,
    reason: String,
    reference: Vec<core::Action>,
    alternatives: Vec<Vec<core::Action>>,
    wrong: Vec<core::Action>,
    wrong_expected: Vec<bool>,
    partial: Vec<core::Action>,
    partial_expected: Vec<bool>,
    initial_expected: Vec<bool>,
    adverse: Vec<core::Action>,
    adverse_expected: Vec<bool>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reviews {
    schema_version: u32,
    origin: String,
    lessons: Vec<Review>,
}
fn reviews() -> &'static [Review] {
    static R: std::sync::OnceLock<Vec<Review>> = std::sync::OnceLock::new();
    R.get_or_init(|| {
        let r: Reviews = serde_json::from_str(include_str!(
            "../../../tests/fixtures/etherchannel-content-review.json"
        ))
        .unwrap();
        assert_eq!(r.schema_version, 1);
        assert!(r.origin.contains("not recorded"));
        assert_eq!(r.lessons.len(), 3);
        for (n, c) in r.lessons.iter().enumerate() {
            assert_eq!(c.id, IDS[n]);
            assert!(!c.reason.is_empty());
            assert_eq!(c.alternatives.len(), 2);
            assert!(!c.adverse.is_empty());
            assert!(c.adverse_expected.iter().any(|v| !*v));
        }
        r.lessons
    })
}
#[derive(Clone)]
enum Op {
    Expand { key: String, probe: String },
    Choice { key: String, label: String },
    Text { key: String, text: String },
    Toggle { key: String },
}
#[derive(Clone, Default)]
struct Cursor {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    wait: usize,
    scroll: usize,
}
impl Cursor {
    fn push(&mut self, op: Op) {
        assert!(self.ops.len() < 300, "Bounded native LACP operation plan");
        self.ops.push(op);
    }
    fn advance(&mut self) {
        self.at += 1;
        self.phase = 0;
        self.wait = 0;
        self.scroll = 0;
    }
}
fn group_key(device: &str, g: &str, property: &str) -> String {
    format!("{device}:Group({g:?}):{property}")
}
fn port_key(device: &str, p: &str, property: &str) -> String {
    format!("{device}:Port({p:?}):{property}")
}
fn rate(r: core::Rate) -> &'static str {
    match r {
        core::Rate::R100 => "100 Mb/s",
        core::Rate::R1000 => "1 Gb/s",
        core::Rate::R10000 => "10 Gb/s",
    }
}
fn text(c: &mut Cursor, key: String, before: &str, after: &str) {
    if before != after {
        c.push(Op::Text {
            key,
            text: after.into(),
        });
    }
}
fn choice(c: &mut Cursor, key: String, before: bool, label: &str) {
    if before {
        c.push(Op::Choice {
            key,
            label: label.into(),
        });
    }
}
fn profile(c: &mut Cursor, prefix: &str, before: &work::ProfileDraft, after: &work::ProfileDraft) {
    choice(
        c,
        format!("{prefix}:Mode"),
        before.mode != after.mode,
        match after.mode {
            work::ProfileMode::Access => "Access",
            work::ProfileMode::Trunk => "Trunk",
        },
    );
    match after.mode {
        work::ProfileMode::Access => {
            text(c, format!("{prefix}:Access"), &before.access, &after.access)
        }
        work::ProfileMode::Trunk => {
            text(c, format!("{prefix}:Native"), &before.native, &after.native);
            text(
                c,
                format!("{prefix}:Allowed"),
                &before.allowed,
                &after.allowed,
            );
        }
    }
}
fn plan(before: &work::Draft, after: &work::Draft) -> Cursor {
    assert_eq!(
        before.devices.keys().collect::<Vec<_>>(),
        after.devices.keys().collect::<Vec<_>>()
    );
    let mut c = Cursor::default();
    for (device, want) in &after.devices {
        let old = &before.devices[device];
        c.push(Op::Choice {
            key: ":Selection:SelectedDevice".into(),
            label: device.clone(),
        });
        text(
            &mut c,
            format!("{device}:Device:Vlans"),
            &old.vlans,
            &want.vlans,
        );
        for (g, want) in &want.groups {
            let old = &old.groups[g];
            if old.up != want.up {
                c.push(Op::Toggle {
                    key: group_key(device, g, "Up"),
                });
            }
            choice(
                &mut c,
                group_key(device, g, "Rate"),
                old.rate != want.rate,
                rate(want.rate),
            );
            if old.profile != want.profile {
                c.push(Op::Expand {
                    key: group_key(device, g, "Details"),
                    probe: group_key(device, g, "Mode"),
                });
                profile(
                    &mut c,
                    &format!("{device}:Group({g:?})"),
                    &old.profile,
                    &want.profile,
                );
            }
        }
        for (p, want) in &want.ports {
            let old = &old.ports[p];
            if old.up != want.up {
                c.push(Op::Toggle {
                    key: port_key(device, p, "Up"),
                });
            }
            choice(
                &mut c,
                port_key(device, p, "Membership"),
                old.group != want.group,
                want.group.as_deref().unwrap_or("Not bundled"),
            );
            choice(
                &mut c,
                port_key(device, p, "Lacp"),
                old.mode != want.mode,
                match want.mode {
                    core::LacpMode::Active => "Active",
                    core::LacpMode::Passive => "Passive",
                },
            );
            if old.rate != want.rate || old.duplex != want.duplex || old.profile != want.profile {
                c.push(Op::Expand {
                    key: port_key(device, p, "Details"),
                    probe: port_key(device, p, "Rate"),
                });
                choice(
                    &mut c,
                    port_key(device, p, "Rate"),
                    old.rate != want.rate,
                    rate(want.rate),
                );
                choice(
                    &mut c,
                    port_key(device, p, "Duplex"),
                    old.duplex != want.duplex,
                    match want.duplex {
                        core::Duplex::Full => "Full duplex",
                        core::Duplex::Half => "Half duplex",
                    },
                );
                profile(
                    &mut c,
                    &format!("{device}:Port({p:?})"),
                    &old.profile,
                    &want.profile,
                );
            }
        }
    }
    assert!(c.ops.len() <= 300);
    c
}
fn keys(key: &str) -> egui::Event {
    egui::Event::Key {
        key: match key {
            "a" => egui::Key::A,
            "z" => egui::Key::Z,
            "tab" | "shift-tab" => egui::Key::Tab,
            "space" => egui::Key::Space,
            _ => panic!("Closed native key"),
        },
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: matches!(key, "a" | "z"),
            command: matches!(key, "a" | "z"),
            shift: key == "shift-tab",
            ..Default::default()
        },
    }
}
fn simulation(a: &Attempt, practice: bool) -> &core::State {
    if practice {
        &a.workspace
            .as_ref()
            .unwrap()
            .etherchannel_learning
            .as_ref()
            .unwrap()
            .simulation
    } else {
        let engine::State::EtherChannel { simulation } = &a.state else {
            panic!()
        };
        simulation
    }
}
fn draft(a: &Attempt, practice: bool) -> &work::Draft {
    let w = a.workspace.as_ref().unwrap();
    if practice {
        &w.etherchannel_learning.as_ref().unwrap().draft
    } else {
        w.etherchannel_draft.as_ref().unwrap()
    }
}
fn portable(p: &Progress) -> serde_json::Value {
    serde_json::to_value(p).unwrap()
}
fn full(p: &Progress) -> serde_json::Value {
    let mut v = portable(p);
    v["migration_receipts"] = serde_json::to_value(&p.migration_receipts).unwrap();
    v
}
#[derive(Clone, Default)]
struct Warm {
    raw: Vec<u8>,
    portable: serde_json::Value,
    typed: serde_json::Value,
    loaded_typed: serde_json::Value,
}
impl LabApp {
    pub(in crate::ui) fn etherchannel_stage(smoke: bool, name: Option<&str>) -> Option<usize> {
        if !smoke {
            return None;
        }
        match name {
            Some("etherchannel-lessons") => Some(FIRST),
            Some("etherchannel-spacing") => Some(49700),
            _ => None,
        }
    }
    fn ether_index(&self, n: usize) -> usize {
        self.scenarios
            .iter()
            .position(|s| s.id == IDS[n])
            .expect("Actual authored LACP source")
    }
    fn ether_click_completed(&self, ctx: &egui::Context) -> bool {
        ctx.data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))))
            .unwrap_or(0)
            >= 2
    }
    fn ether_click(&mut self, ctx: &egui::Context, rect: Option<egui::Rect>) -> bool {
        self.click_visible_smoke_control(ctx, rect)
    }
    fn ether_control(&mut self, ctx: &egui::Context, key: &str) -> bool {
        let rect = self
            .metrics
            .ether
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| {
                assert!(r.2);
                r.1
            });
        self.ether_click(ctx, rect)
    }
    fn ether_expander(&mut self, ctx: &egui::Context, key: &str, probe: &str) -> bool {
        if self.ether_click_completed(ctx) {
            return self.click_smoke_control(ctx, None);
        }
        if self.metrics.ether.controls.iter().any(|r| r.0 == probe) {
            return true;
        }
        let r = self
            .metrics
            .ether
            .expanders
            .iter()
            .find(|r| r.0 == key)
            .expect("Actual discoverable LACP detail control");
        assert!(r.2);
        self.click_visible_smoke_control(ctx, Some(r.1))
    }
    fn ether_tab(&mut self, ctx: &egui::Context, wanted: teaching::View) -> bool {
        let r = self
            .metrics
            .ether
            .tabs
            .iter()
            .find(|r| r.0 == wanted)
            .map(|r| r.1);
        self.ether_click(ctx, r)
    }
    fn ether_popup_choice(
        &mut self,
        ctx: &egui::Context,
        key: &str,
        label: &str,
        c: &mut Cursor,
    ) -> bool {
        if self.ether_click_completed(ctx) {
            return self.click_smoke_control(ctx, None);
        }
        let name = format!("{key}:{label}");
        let r = self
            .metrics
            .ether
            .choices
            .iter()
            .find(|r| r.0 == name)
            .map(|r| r.1);
        let clip = self
            .metrics
            .ether
            .popup
            .as_ref()
            .filter(|r| r.0 == key)
            .map(|r| r.1.intersect(ctx.content_rect()));
        let (Some(r), Some(clip)) = (r, clip) else {
            c.wait += 1;
            assert!(c.wait <= 60, "LACP popup did not open: {key}");
            return false;
        };
        if !smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            c.scroll += 1;
            assert!(c.scroll <= 20, "LACP option remains clipped: {name}");
            let delta = if r.bottom() > clip.bottom() {
                clip.bottom() - r.bottom()
            } else {
                clip.top() - r.top()
            };
            self.pending_events.extend([
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta.clamp(-250., 250.)),
                    modifiers: Default::default(),
                },
            ]);
            return false;
        }
        self.click_smoke_control(ctx, Some(r))
    }
    fn ether_cursor(&mut self, ctx: &egui::Context, c: &mut Cursor) -> bool {
        if c.at == c.ops.len() {
            return true;
        }
        let op = c.ops[c.at].clone();
        let old_at = c.at;
        match op {
            Op::Expand { key, probe } => {
                if self.ether_expander(ctx, &key, &probe) {
                    c.advance();
                }
            }
            Op::Toggle { key } => {
                if self.ether_control(ctx, &key) {
                    c.advance();
                }
            }
            Op::Choice { key, label } => {
                if c.phase == 0 {
                    if self.ether_control(ctx, &key) {
                        c.phase = 1;
                    }
                } else if self.ether_popup_choice(ctx, &key, &label, c) {
                    c.advance();
                }
            }
            Op::Text { key, text } => {
                let field = self
                    .metrics
                    .ether
                    .fields
                    .iter()
                    .find(|r| r.0 == key)
                    .expect("Visible actual LACP text field");
                assert!(field.3);
                let (rect, id) = (field.1, field.2);
                if c.phase == 0 {
                    if self.click_visible_smoke_control(ctx, Some(rect)) {
                        c.phase = 1;
                    }
                } else if c.phase == 1 {
                    assert!(
                        ctx.memory(|m| m.has_focus(id)),
                        "Real click must focus {key}"
                    );
                    self.pending_events
                        .extend([keys("a"), egui::Event::Paste(text)]);
                    c.phase = 2;
                } else {
                    assert!(
                        !self.metrics.ether.rejected,
                        "Valid bounded text must reach its native field"
                    );
                    c.advance();
                }
            }
        }
        if c.at != old_at {
            let key = egui::Id::new(("bounded-native-scroll", self.smoke_stage));
            ctx.data_mut(|data| data.remove::<usize>(key));
        }
        assert!(
            self.pending_events.len() <= 300,
            "Bounded native event queue"
        );
        false
    }
    fn ether_plan(&mut self, ctx: &egui::Context, n: usize, actions: &[core::Action]) -> bool {
        assert_eq!(
            actions.len(),
            1,
            "Authored plans have one complete target, not mutation history authority"
        );
        let core::Action::Configure { configuration } = &actions[0];
        let i = self.ether_index(n);
        let a = &self.progress.attempts[IDS[n]];
        let desired =
            work::Draft::from_configuration(configuration, simulation(a, false).topology())
                .unwrap();
        let key = egui::Id::new(("etherchannel-native-plan", self.smoke_stage));
        let saved = ctx.data_mut(|data| data.remove_temp::<Cursor>(key));
        let mut cursor = saved.unwrap_or_else(|| plan(draft(a, false), &desired));
        if self.ether_cursor(ctx, &mut cursor) {
            let actual = draft(&self.progress.attempts[IDS[n]], false)
                .parsed(simulation(&self.progress.attempts[IDS[n]], false).topology())
                .unwrap();
            assert_eq!(
                &actual, configuration,
                "Actual native edits, not installed configuration, must produce target"
            );
            assert_eq!(self.selected, Some(i));
            true
        } else {
            ctx.data_mut(|data| data.insert_temp(key, cursor));
            false
        }
    }
    fn ether_one_op(&mut self, ctx: &egui::Context, op: Op) -> bool {
        self.ether_ops(ctx, vec![op])
    }
    fn ether_ops(&mut self, ctx: &egui::Context, ops: Vec<Op>) -> bool {
        assert!(ops.len() <= 300);
        let done = egui::Id::new(("etherchannel-native-single-done", self.smoke_stage));
        if ctx
            .data(|data| data.get_temp::<bool>(done))
            .unwrap_or(false)
        {
            return true;
        }
        let key = egui::Id::new(("etherchannel-native-single", self.smoke_stage));
        let mut c = ctx
            .data_mut(|d| d.remove_temp::<Cursor>(key))
            .unwrap_or_else(|| Cursor {
                ops,
                ..Default::default()
            });
        if self.ether_cursor(ctx, &mut c) {
            ctx.data_mut(|data| data.insert_temp(done, true));
            true
        } else {
            ctx.data_mut(|d| d.insert_temp(key, c));
            false
        }
    }
    fn ether_rows(&self, n: usize, expected: &[bool]) {
        let i = self.ether_index(n);
        let a = &self.progress.attempts[IDS[n]];
        let actual = progress::current_etherchannel_checks(a, &self.scenarios[i]);
        assert_eq!(actual.iter().map(|r| r.0).collect::<Vec<_>>(), expected);
        assert_eq!(
            self.metrics
                .ether
                .rows
                .iter()
                .map(|r| r.1)
                .collect::<Vec<_>>(),
            expected,
            "Actual painted rows must follow same-frame current state"
        );
        assert_eq!(self.metrics.completion, expected.iter().all(|v| *v));
    }
    fn ether_result_shot(
        &mut self,
        ctx: &egui::Context,
        n: usize,
        label: &str,
    ) -> (bool, Option<String>) {
        self.ether_row_shot(ctx, n, label, 1)
    }
    fn ether_row_shot(
        &mut self,
        ctx: &egui::Context,
        n: usize,
        label: &str,
        row: usize,
    ) -> (bool, Option<String>) {
        let r = self
            .metrics
            .ether
            .rows
            .iter()
            .find(|r| r.0 == row)
            .expect("Actual participating-member result")
            .2;
        if !self.reveal_smoke_control(ctx, r) {
            return (false, None);
        }
        (true, Some(format!("{}-{label}", IDS[n])))
    }
    fn ether_warm(&mut self) -> Warm {
        let before = portable(&self.progress);
        let typed = full(&self.progress);
        self.save();
        assert!(self.storage_error.is_none());
        let raw = progress::read_bounded(&self.path, 16_000_000).unwrap();
        let loaded = progress::decode(&raw, &self.scenarios).unwrap();
        assert_eq!(portable(&loaded), before);
        let raw_value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(
            raw_value["migration_receipts"],
            serde_json::to_value(&loaded.migration_receipts).unwrap()
        );
        let loaded_typed = full(&loaded);
        assert_eq!(full(&self.progress), typed);
        assert_eq!(
            portable(&self.progress),
            before,
            "Readback never installs loaded Progress"
        );
        Warm {
            raw,
            portable: before,
            typed,
            loaded_typed,
        }
    }
    fn ether_install_sources(&mut self) {
        let present = IDS
            .iter()
            .filter(|id| self.scenarios.iter().any(|s| s.id == **id))
            .count();
        assert!(
            present == 0 || present == 3,
            "Never mix a partial registration with injected source"
        );
        if present == 0 {
            let old = serde_json::to_vec(&self.scenarios).unwrap();
            let old_units = serde_json::to_vec(&self.units).unwrap();
            let mut supplied = pack::load_catalog(&[
                include_str!("../../../scenarios/starter-v1.json"),
                include_str!("../../../scenarios/vlan-data-v1.json"),
                include_str!("../../../scenarios/etherchannel-v1.json"),
            ])
            .unwrap();
            supplied.retain(|s| IDS.contains(&s.id.as_str()));
            assert_eq!(supplied.len(), 3);
            let before = self.scenarios.len();
            self.scenarios.extend(supplied);
            assert_eq!(serde_json::to_vec(&self.scenarios[..before]).unwrap(), old);
            let count = self.units.len();
            self.units.push(course::Unit {
                id: "ccna-lacp".into(),
                track: "CCNA".into(),
                title: "LACP: form, diagnose and safely migrate a bundle".into(),
                summary:
                    "Supplied Layer 2 members, VLAN admission and independent cable-loss checks."
                        .into(),
                prerequisites: vec!["vlan-data-foundations".into()],
                labs: IDS.iter().map(|id| (*id).into()).collect(),
            });
            assert_eq!(serde_json::to_vec(&self.units[..count]).unwrap(), old_units);
            println!(
                "Native authoring fixture: injected three unregistered LACP assignments and one advisory unit; no normal catalog credit."
            );
        } else {
            let units: Vec<_> = self.units.iter().filter(|u| u.id == "ccna-lacp").collect();
            assert_eq!(units.len(), 1);
            assert_eq!(units[0].labs, IDS);
            assert_eq!(units[0].prerequisites, ["vlan-data-foundations"]);
            println!(
                "Native normal LACP: three bundled authored assignments and their real course unit; no fixture injection"
            );
        }
    }
    fn ether_header(&mut self, ctx: &egui::Context, key: &str) -> bool {
        let r = self
            .metrics
            .workspace_header_controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1);
        self.ether_click(ctx, r)
    }
    fn ether_probe(&self, n: usize, practice: bool, pass: bool) {
        let i = self.ether_index(n);
        let a = &self.progress.attempts[IDS[n]];
        if practice {
            let l = a
                .workspace
                .as_ref()
                .unwrap()
                .etherchannel_learning
                .as_ref()
                .unwrap();
            let rows = l.checks(self.scenarios[i].etherchannel_teaching.as_ref().unwrap());
            assert_eq!(rows.iter().all(|r| r.0), pass);
            assert!(!self.metrics.completion);
        } else {
            assert_eq!(progress::current_solved(a, &self.scenarios[i]), pass);
            assert_eq!(self.metrics.completion, pass);
        }
    }
    fn ether_scroll_idle(&mut self, ctx: &egui::Context, n: usize) -> bool {
        #[derive(Clone, Copy, Default)]
        struct Observation {
            frames: usize,
            base: etherchannel_workspace::memo::Counters,
            start: f32,
            saw_scroll: bool,
            scrollable: bool,
            admissions: usize,
        }
        let key = egui::Id::new(("ether-scroll-idle", n));
        let counters = self.ether_presentation.memo.counters;
        let offset = self.metrics.main_scroll_offset.y;
        let scrollable = self
            .metrics
            .main_viewport
            .is_some_and(|v| self.metrics.main_content_size.y > v.height() + 1.);
        let mut observation = ctx
            .data_mut(|data| data.remove_temp::<Observation>(key))
            .unwrap_or(Observation {
                base: counters,
                start: offset,
                scrollable,
                admissions: work::json_admission_calls(),
                ..Default::default()
            });
        assert_eq!(
            counters.evaluations, observation.base.evaluations,
            "Real scroll/idle must not regrade an exact input"
        );
        assert_eq!(
            counters.observations, observation.base.observations,
            "Idle/scroll must not calculate pair observations"
        );
        assert_eq!(
            counters.meters, observation.base.meters,
            "No idle memo metering"
        );
        assert_eq!(
            work::json_admission_calls(),
            observation.admissions,
            "No idle LACP adapter JSON admission calls, including live display/navigation"
        );
        assert_eq!(
            counters.snapshots, observation.base.snapshots,
            "No idle source/Attempt copies"
        );
        observation.saw_scroll |= (offset - observation.start).abs() > 0.01;
        if observation.frames == 80 {
            assert!(
                !observation.scrollable || observation.saw_scroll,
                "Scrollable workspace must observe actual movement from the20 wheel inputs"
            );
            println!(
                "PASS native LACP{} scroll/idle:20 real wheel inputs then60 idle observations; model evaluations/meters/snapshots unchanged, scrollable={} moved={}",
                IDS[n], observation.scrollable, observation.saw_scroll
            );
            return true;
        }
        if observation.frames < 20 {
            let view = self
                .metrics
                .main_viewport
                .unwrap()
                .intersect(ctx.content_rect());
            self.pending_events.extend([
                egui::Event::PointerMoved(egui::pos2(view.left() + 1., view.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., if observation.frames < 10 { 80. } else { -80. }),
                    modifiers: Default::default(),
                },
            ]);
        }
        observation.frames += 1;
        ctx.data_mut(|data| data.insert_temp(key, observation));
        ctx.request_repaint();
        false
    }
    fn ether_native_inner(&mut self, ctx: &egui::Context, stage: usize) -> (bool, Option<String>) {
        assert!(self.smoke && self._smoke_profile.is_some());
        assert!((FIRST..=END).contains(&stage));
        assert!(self.pending_events.len() <= 300);
        if stage >= ALIGN {
            return self.ether_alignment(ctx, stage);
        }
        let n = (stage - FIRST) / STRIDE;
        let phase = (stage - FIRST) % STRIDE;
        assert!(n < 3 && phase <= LAST, "Closed native LACP phase");
        let r = &reviews()[n];
        if phase == 0 {
            if n == 0 {
                assert!(self.progress.attempts.is_empty());
                self.ether_install_sources();
            }
            self.scale = 1.;
            self.theme_id = "dlms-dark".into();
            theme::apply(ctx, &self.theme_id);
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            assert!(self.selected.is_none());
            let i = self.ether_index(n);
            self.filter = Filter {
                track: "CCNA".into(),
                search: self.scenarios[i].title.clone(),
                ..Default::default()
            };
            self.page = 0;
            return (true, None);
        }
        let i = self.ether_index(n);
        let id = IDS[n];
        match phase {
            1 => {
                assert_eq!(self.metrics.card_actions.len(), 1);
                if !self.click_visible_smoke_control(ctx, Some(self.metrics.card_actions[0].1)) {
                    return (false, None);
                }
            }
            2 => {
                assert!(self.progress.favorites.contains(id));
                if !self.ether_click(ctx, self.metrics.card_actions.first().map(|r| r.0)) {
                    return (false, None);
                }
            }
            3 => {
                assert_eq!(self.selected, Some(i));
                assert_eq!(
                    self.progress.attempts[id]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .etherchannel_learning
                        .as_ref()
                        .unwrap()
                        .view,
                    teaching::View::Introduction
                );
                assert_eq!(
                    &self.progress.attempts[id].state,
                    &self.scenarios[i].initial
                );
                assert!(!self.metrics.completion);
                return (true, Some(format!("{id}-introduction")));
            }
            4 => {
                if !self.ether_control(ctx, "try-example") {
                    return (false, None);
                }
            }
            5 => {
                assert_eq!(
                    &self.progress.attempts[id].state,
                    &self.scenarios[i].initial
                );
                return (true, Some(format!("{id}-example-initial")));
            }
            6 => {
                if !self.ether_control(ctx, "fill") {
                    return (false, None);
                }
            }
            7 => {
                let a = &self.progress.attempts[id];
                assert_eq!(&a.state, &self.scenarios[i].initial);
                assert!(!draft(a, true).matches(simulation(a, true)));
                if !self.ether_control(ctx, "check") {
                    return (false, None);
                }
            }
            8 => {
                if !self.ether_click_completed(ctx) {
                    self.ether_probe(n, true, false);
                }
                if !self.ether_control(ctx, "apply") {
                    return (false, None);
                }
            }
            9 => {
                if !self.ether_control(ctx, "check") {
                    return (false, None);
                }
            }
            10 => {
                self.ether_probe(n, true, true);
                assert_eq!(
                    &self.progress.attempts[id].state,
                    &self.scenarios[i].initial
                );
                return self.ether_result_shot(ctx, n, "example-verified");
            }
            11 | 15 => {
                if !self.ether_one_op(
                    ctx,
                    Op::Choice {
                        key: ":Selection:SelectedDevice".into(),
                        label: "example-a".into(),
                    },
                ) {
                    return (false, None);
                }
                if phase == 15 {
                    self.ether_probe(n, true, false);
                    let l = self.progress.attempts[id]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .etherchannel_learning
                        .as_ref()
                        .unwrap();
                    let lesson = self.scenarios[i].etherchannel_teaching.as_ref().unwrap();
                    assert_eq!(
                        l.checks(lesson).iter().map(|r| r.0).collect::<Vec<_>>(),
                        lesson.try_change.expected
                    );
                    return self.ether_result_shot(ctx, n, "example-adverse-retry");
                }
            }
            12 | 16 => {
                if !self.ether_one_op(
                    ctx,
                    Op::Toggle {
                        key: port_key("example-a", "p1", "Up"),
                    },
                ) {
                    return (false, None);
                }
            }
            13 | 17 => {
                if !self.ether_control(ctx, "apply") {
                    return (false, None);
                }
            }
            14 => {
                if !self.ether_control(ctx, "check") {
                    return (false, None);
                }
            }
            18 => {
                if !self.ether_control(ctx, "check") {
                    return (false, None);
                }
            }
            19 => {
                self.ether_probe(n, true, true);
                if !self.ether_tab(ctx, teaching::View::Assignment) {
                    return (false, None);
                }
            }
            20 => {
                if !self.ether_control(ctx, "check") {
                    return (false, None);
                }
            }
            21 => {
                self.ether_rows(n, &r.initial_expected);
                return self.ether_result_shot(ctx, n, "assignment-initial");
            }
            22 => {
                if !self.ether_plan(ctx, n, &r.wrong) {
                    return (false, None);
                }
            }
            23 => {
                if !self.ether_control(ctx, "apply") {
                    return (false, None);
                }
            }
            24 => {
                if !self.ether_control(ctx, "check") {
                    return (false, None);
                }
            }
            25 => {
                self.ether_rows(n, &r.wrong_expected);
                return self.ether_result_shot(ctx, n, "wrong-specific-feedback");
            }
            26 => {
                if !self.ether_plan(ctx, n, &r.partial) {
                    return (false, None);
                }
            }
            27 => {
                if !self.ether_control(ctx, "apply") {
                    return (false, None);
                }
            }
            28 => {
                if !self.ether_control(ctx, "check") {
                    return (false, None);
                }
            }
            29 => {
                self.ether_rows(n, &r.partial_expected);
                return self.ether_result_shot(ctx, n, "partial-feedback");
            }
            30 => {
                if !self.ether_plan(ctx, n, &r.reference) {
                    return (false, None);
                }
            }
            31 => {
                if !self.ether_control(ctx, "apply") {
                    return (false, None);
                }
            }
            32 => {
                if !self.ether_control(ctx, "check") {
                    return (false, None);
                }
            }
            33 => {
                self.ether_probe(n, false, true);
                return self.ether_result_shot(ctx, n, "reference-verified");
            }
            34 => {
                if n == 2 {
                    if !self.ether_ops(
                        ctx,
                        vec![
                            Op::Expand {
                                key: "inspect-options".into(),
                                probe: ":Selection:SelectedCondition".into(),
                            },
                            Op::Choice {
                                key: ":Selection:SelectedCondition".into(),
                                label: "If loss-1".into(),
                            },
                        ],
                    ) {
                        return (false, None);
                    }
                    self.ether_probe(n, false, true);
                    return self.ether_row_shot(ctx, n, "independent-cable-loss-observation", 4);
                }
                let device = simulation(&self.progress.attempts[id], false)
                    .topology()
                    .devices
                    .keys()
                    .next()
                    .unwrap()
                    .clone();
                if !self.ether_one_op(
                    ctx,
                    Op::Choice {
                        key: ":Selection:SelectedDevice".into(),
                        label: device,
                    },
                ) {
                    return (false, None);
                }
            }
            35 => {
                let d = draft(&self.progress.attempts[id], false);
                let device = d.selected_device.clone();
                let altered = d.devices[&device]
                    .vlans
                    .split(',')
                    .filter(|s| *s != "30")
                    .collect::<Vec<_>>()
                    .join(",");
                if !self.ether_one_op(
                    ctx,
                    Op::Text {
                        key: format!("{device}:Device:Vlans"),
                        text: altered,
                    },
                ) {
                    return (false, None);
                }
            }
            36 => {
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &self.scenarios[i]
                ));
                assert!(!self.metrics.completion);
                return self.ether_result_shot(ctx, n, "pending-edit-invalidates-success");
            }
            37 => {
                let field = self
                    .metrics
                    .ether
                    .fields
                    .iter()
                    .find(|r| r.0.ends_with(":Device:Vlans"))
                    .unwrap();
                assert!(
                    ctx.memory(|m| m.has_focus(field.2)),
                    "Native edit retained actual focus"
                );
                let state = egui::TextEdit::load_state(ctx, field.2).unwrap();
                if state.undoer().is_in_flux() {
                    let k = egui::Id::new(("ether-undo-settle", stage));
                    let tries = ctx.data_mut(|d| {
                        let n = d.get_temp::<usize>(k).unwrap_or(0) + 1;
                        d.insert_temp(k, n);
                        n
                    });
                    assert!(tries <= 60, "Real native Undo checkpoint must settle");
                    return (false, None);
                }
            }
            38 => self.pending_events.push(keys("z")),
            39 => {
                self.ether_probe(n, false, true);
                return self.ether_result_shot(ctx, n, "native-undo-restores-current-success");
            }
            40 => {
                let field = self
                    .metrics
                    .ether
                    .fields
                    .iter()
                    .find(|r| r.0.ends_with(":Device:Vlans"))
                    .unwrap();
                if !self.click_visible_smoke_control(ctx, Some(field.1)) {
                    return (false, None);
                }
            }
            41 => self.pending_events.push(keys("tab")),
            42 => {
                let d = draft(&self.progress.attempts[id], false);
                let dev = &d.selected_device;
                let g = d.devices[dev].groups.keys().next().unwrap();
                let k = group_key(dev, g, "Up");
                let control = self
                    .metrics
                    .ether
                    .control_ids
                    .iter()
                    .find(|r| r.0 == k)
                    .unwrap()
                    .1;
                assert!(
                    ctx.memory(|m| m.has_focus(control)),
                    "Tab must focus the actual next native checkbox"
                );
                self.pending_events.push(keys("space"));
            }
            43 => {
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &self.scenarios[i]
                ));
                self.pending_events.push(keys("space"));
            }
            44 => self.pending_events.push(keys("shift-tab")),
            45 => {
                // Pinned egui delivers Previous focus via id_next_frame. Observe
                // that actual next frame before testing focus; never request it.
                let delivered = egui::Id::new(("ether-shift-tab-delivery", stage));
                if !ctx
                    .data(|data| data.get_temp::<bool>(delivered))
                    .unwrap_or(false)
                {
                    ctx.data_mut(|data| data.insert_temp(delivered, true));
                    ctx.request_repaint();
                    return (false, None);
                }
                let key = egui::Id::new(("ether-shift-tab-checked", stage));
                if !ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false) {
                    let field = self
                        .metrics
                        .ether
                        .fields
                        .iter()
                        .find(|r| r.0.ends_with(":Device:Vlans"))
                        .unwrap();
                    assert!(
                        ctx.memory(|m| m.has_focus(field.2)),
                        "Shift-Tab returns to the actual field"
                    );
                    ctx.data_mut(|d| d.insert_temp(key, true));
                }
                if !self.ether_control(ctx, "apply") {
                    return (false, None);
                }
            }
            46 => {
                if !self.ether_control(ctx, "check") {
                    return (false, None);
                }
            }
            47 => {
                let key = egui::Id::new(("ether-before-alternate", stage));
                if !ctx.data(|data| data.get_temp::<bool>(key)).unwrap_or(false) {
                    self.ether_probe(n, false, true);
                    ctx.data_mut(|data| data.insert_temp(key, true));
                }
                if !self.ether_plan(ctx, n, &r.alternatives[0]) {
                    return (false, None);
                }
            }
            48 => {
                if !self.ether_control(ctx, "apply") {
                    return (false, None);
                }
            }
            49 => {
                if !self.ether_control(ctx, "check") {
                    return (false, None);
                }
            }
            50 => {
                self.ether_probe(n, false, true);
                return self.ether_result_shot(ctx, n, "alternate-one-verified");
            }
            51 => {
                if !self.ether_plan(ctx, n, &r.alternatives[1]) {
                    return (false, None);
                }
            }
            52 => {
                if !self.ether_control(ctx, "apply") {
                    return (false, None);
                }
            }
            53 => {
                if !self.ether_control(ctx, "check") {
                    return (false, None);
                }
            }
            54 => {
                self.ether_probe(n, false, true);
                return self.ether_result_shot(ctx, n, "alternate-two-verified");
            }
            55 => {
                if !self.ether_header(ctx, "reset") {
                    return (false, None);
                }
            }
            56 => {
                assert!(self.reset_pending);
                assert!(self.metrics.ether.fields.iter().all(|r| !r.3));
                assert!(self.metrics.ether.controls.iter().all(|r| !r.2));
                let snapshot = portable(&self.progress);
                ctx.data_mut(|d| d.insert_temp(egui::Id::new(("ether-modal", n)), snapshot));
                self.pending_events.push(egui::Event::Paste("20".into()));
            }
            57 => {
                let before = ctx
                    .data(|d| d.get_temp::<serde_json::Value>(egui::Id::new(("ether-modal", n))))
                    .unwrap();
                assert_eq!(
                    portable(&self.progress),
                    before,
                    "Modal prevents hidden draft edits"
                );
                if !self.click_visible_smoke_control(ctx, self.metrics.reset_cancel) {
                    return (false, None);
                }
            }
            58 => {
                self.ether_probe(n, false, true);
                let warm = self.ether_warm();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new(("ether-warm", n)), warm));
            }
            59 => {
                let warm = ctx
                    .data(|d| d.get_temp::<Warm>(egui::Id::new(("ether-warm", n))))
                    .unwrap();
                assert_eq!(portable(&self.progress), warm.portable);
                assert_eq!(full(&self.progress), warm.typed);
                assert_eq!(std::fs::read(&self.path).unwrap(), warm.raw);
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(portable(&loaded), warm.portable);
                assert_eq!(full(&loaded), warm.loaded_typed);
                assert_eq!(portable(&self.progress), warm.portable);
                assert_eq!(full(&self.progress), warm.typed);
                let out = self
                    .path
                    .parent()
                    .unwrap()
                    .join(format!("lacp-{n}-backup.json"));
                assert!(
                    dlms_learning_lab::backup::export(&out, &self.progress, &self.scenarios)
                        .unwrap()
                        .is_none()
                );
                let candidate = dlms_learning_lab::backup::inspect(
                    &out,
                    &self.scenarios,
                    &std::sync::atomic::AtomicBool::new(false),
                    &std::sync::atomic::AtomicUsize::new(0),
                )
                .unwrap();
                assert_eq!(portable(&candidate.progress), warm.portable);
                let restored = self
                    .path
                    .parent()
                    .unwrap()
                    .join(format!("lacp-{n}-restore.json"));
                dlms_learning_lab::backup::restore(&restored, &candidate, &self.scenarios).unwrap();
                assert_eq!(
                    portable(&progress::load(&restored, &self.scenarios).unwrap()),
                    warm.portable
                );
                assert_eq!(portable(&self.progress), warm.portable);
            }
            60 => {
                if !self.ether_header(ctx, "back") {
                    return (false, None);
                }
            }
            61 => {
                assert!(self.progress.favorites.contains(id));
                assert!(self.selected.is_none() || self.selected == Some(i));
                if self.selected.is_none() {
                    assert_eq!(self.metrics.card_actions.len(), 1);
                }
                if !self.ether_click(ctx, self.metrics.card_actions.first().map(|r| r.0)) {
                    return (false, None);
                }
            }
            62 => {
                self.ether_probe(n, false, true);
                if !self.ether_header(ctx, "challenge") {
                    return (false, None);
                }
            }
            63 => {
                assert!(self.progress.attempts[id].challenge);
                assert!(self.metrics.ether.tabs.is_empty());
                self.ether_probe(n, false, true);
                return self.ether_result_shot(ctx, n, "challenge-keeps-full-task");
            }
            64 => {
                let key = egui::Id::new(("ether-other-before-reset", n));
                if ctx
                    .data(|data| data.get_temp::<serde_json::Value>(key))
                    .is_none()
                {
                    let mut others = portable(&self.progress);
                    others["attempts"].as_object_mut().unwrap().remove(id);
                    ctx.data_mut(|data| data.insert_temp(key, others));
                }
                if !self.ether_header(ctx, "reset") {
                    return (false, None);
                }
            }
            65 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.confirm_reset) {
                    return (false, None);
                }
            }
            66 => {
                let mut others = portable(&self.progress);
                others["attempts"].as_object_mut().unwrap().remove(id);
                let expected = ctx
                    .data(|data| {
                        data.get_temp::<serde_json::Value>(egui::Id::new((
                            "ether-other-before-reset",
                            n,
                        )))
                    })
                    .unwrap();
                assert_eq!(
                    others, expected,
                    "Reset must preserve all other labs, drafts, visits and favorites"
                );
                let a = &self.progress.attempts[id];
                assert_eq!(a.state, self.scenarios[i].initial);
                assert!(!progress::current_solved(a, &self.scenarios[i]));
                assert!(a.workspace.as_ref().unwrap().etherchannel_draft.is_some());
                assert!(
                    a.workspace
                        .as_ref()
                        .unwrap()
                        .etherchannel_learning
                        .is_some()
                );
                assert!(!a.challenge);
                assert!(self.progress.favorites.contains(id));
                assert_eq!(
                    a.workspace
                        .as_ref()
                        .unwrap()
                        .etherchannel_learning
                        .as_ref()
                        .unwrap()
                        .view,
                    teaching::View::Introduction
                );
                return (true, Some(format!("{id}-reset-prepared-introduction")));
            }
            67 => {
                if !self.ether_tab(ctx, teaching::View::Assignment) {
                    return (false, None);
                }
            }
            68 => {
                if !self.ether_control(ctx, "check") {
                    return (false, None);
                }
            }
            69 => {
                self.ether_rows(n, &r.initial_expected);
                if n == 2
                    && !self.ether_expander(ctx, "inspect-options", ":Selection:SelectedCondition")
                {
                    return (false, None);
                }
            }
            70 => {
                if n == 2
                    && !self.ether_one_op(
                        ctx,
                        Op::Choice {
                            key: ":Selection:SelectedCondition".into(),
                            label: "If loss-1".into(),
                        },
                    )
                {
                    return (false, None);
                }
            }
            71 => {}
            72 => {
                if n == 2
                    && !self.ether_one_op(
                        ctx,
                        Op::Choice {
                            key: ":Selection:SelectedCondition".into(),
                            label: "Current cables".into(),
                        },
                    )
                {
                    return (false, None);
                }
            }
            73 => {
                let finished = egui::Id::new(("ether-observation-finished", n));
                if !ctx
                    .data(|data| data.get_temp::<bool>(finished))
                    .unwrap_or(false)
                {
                    if !self.ether_scroll_idle(ctx, n) {
                        return (false, None);
                    }
                    ctx.data_mut(|data| data.insert_temp(finished, true));
                }
            }
            74 => {
                let key = egui::Id::new(("ether-details-progress", n));
                if ctx.data(|d| d.get_temp::<serde_json::Value>(key)).is_none() {
                    let before = portable(&self.progress);
                    ctx.data_mut(|d| d.insert_temp(key, before));
                }
                if !self.ether_control(ctx, "objective-details") {
                    return (false, None);
                }
            }
            75 => {
                let expected = &r.initial_expected;
                self.ether_rows(n, expected);
                assert_eq!(
                    self.metrics
                        .ether
                        .detail_rows
                        .iter()
                        .map(|r| (r.0, r.1))
                        .collect::<Vec<_>>(),
                    expected.iter().copied().enumerate().collect::<Vec<_>>(),
                    "Actual requested details contain every original result exactly once"
                );
                let before = ctx
                    .data(|d| {
                        d.get_temp::<serde_json::Value>(egui::Id::new((
                            "ether-details-progress",
                            n,
                        )))
                    })
                    .unwrap();
                assert_eq!(
                    portable(&self.progress),
                    before,
                    "Requested details never change saved work or grading"
                );
                let first = self.metrics.ether.detail_rows.first().unwrap().2;
                if !self.reveal_smoke_control(ctx, first) {
                    return (false, None);
                }
                return (true, Some(format!("{id}-objective-details-requested")));
            }
            76 => {
                if !self.ether_control(ctx, "objective-details") {
                    return (false, None);
                }
            }
            77 => {
                if !self.metrics.ether.detail_rows.is_empty() {
                    ctx.request_repaint();
                    return (false, None);
                }
                if n < 2 && !self.ether_header(ctx, "back") {
                    return (false, None);
                }
                println!(
                    "PASS native LACP {id}: independent practice, actual member/profile edits, literal wrong/partial vectors, reference/two alternatives, invalidation/Undo/keyboard, modal, raw/typed restore, reset. Unregistered source is not curriculum acceptance."
                );
            }
            78 => {
                self.smoke_stage = if n < 2 {
                    FIRST + (n + 1) * STRIDE - 1
                } else {
                    ALIGN - 1
                };
            }
            _ => unreachable!("Closed LACP phase"),
        }
        assert!(self.pending_events.len() <= 300);
        (true, None)
    }
    pub(in crate::ui) fn etherchannel_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        let result = self.ether_native_inner(ctx, stage);
        if result.0 {
            let done = egui::Id::new(("etherchannel-native-single-done", stage));
            ctx.data_mut(|data| data.remove::<bool>(done));
        }
        if let Some(tag) = &result.1 {
            let key = egui::Id::new("ether-native-capture-tags");
            let mut tags = ctx
                .data_mut(|d| d.remove_temp::<Vec<String>>(key))
                .unwrap_or_default();
            assert!(
                !tags.contains(tag),
                "Capture tags are unique and cannot be reissued"
            );
            assert!(tags.len() < 128);
            tags.push(tag.clone());
            ctx.data_mut(|d| d.insert_temp(key, tags));
        }
        result
    }
    fn ether_alignment(&mut self, ctx: &egui::Context, stage: usize) -> (bool, Option<String>) {
        let offset = stage - ALIGN;
        if offset == 80 {
            let before = ctx
                .data(|d| {
                    d.get_temp::<serde_json::Value>(egui::Id::new("ether-alignment-portable"))
                })
                .unwrap();
            assert_eq!(
                portable(&self.progress),
                before,
                "Appearance and scrolling must not change progress"
            );
            let tags = ctx
                .data(|d| d.get_temp::<Vec<String>>(egui::Id::new("ether-native-capture-tags")))
                .unwrap();
            assert!(tags.len() <= 128);
            assert_eq!(
                tags.iter()
                    .filter(|s| s.starts_with("lacp-alignment-"))
                    .count(),
                60
            );
            println!(
                "PASS native LACP closed acceptance: three authored contexts, meaningful actual configuration edits, references/two alternatives, wrong/partial/adverse feedback, current success invalidation, Undo/Tab/Shift-Tab, modal/reset/warm/validated backup, all five text scales wide/narrow light/dark with60 measured captures (fields, members and compact feedback). Native source fixtures alone do not grant registered curriculum coverage."
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return (false, None);
        }
        assert!(offset < 80);
        let case = offset / 4;
        let phase = offset % 4;
        let scale = SCALES[case % 5];
        let light = (case / 5) % 2 == 1;
        let narrow = case >= 10;
        if phase == 0 {
            self.scale = scale;
            self.theme_id = if light { "dlms-light" } else { "dlms-dark" }.into();
            theme::apply(ctx, &self.theme_id);
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!narrow));
            if narrow {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(946., 929.)));
            }
            if case == 0 {
                let before = portable(&self.progress);
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("ether-alignment-portable"), before));
            }
            return (true, None);
        }
        // Read Context geometry before entering a data closure. A window-size request
        // is never a claim about the actual physical window.
        let pixels = ctx.content_rect().size() * ctx.pixels_per_point();
        let fullscreen = ctx.input(|i| i.viewport().fullscreen);
        let ppp = ctx.pixels_per_point();
        let native_ppp = ctx
            .input(|input| input.viewport().native_pixels_per_point)
            .unwrap_or(1.);
        let ready = (ctx.zoom_factor() - scale).abs() < 0.01
            && (ppp - native_ppp * scale).abs() < 0.01
            && fullscreen == Some(!narrow)
            && if narrow {
                (650. ..=1300.).contains(&pixels.x)
            } else {
                pixels.x >= 1500.
            };
        // egui applies a new zoom on the next pass and briefly rescales the old
        // content rectangle. Require consecutive actual geometry, not a request
        // or merely a broad width class, before checking/capturing controls.
        type GeometrySample = (egui::Rect, u32, u32, Option<bool>, Option<egui::Rect>);
        #[derive(Clone, Copy, Default)]
        struct Settlement {
            sample: Option<GeometrySample>,
            same: usize,
            tries: usize,
        }
        let sample = (
            ctx.content_rect(),
            ppp.to_bits(),
            ctx.zoom_factor().to_bits(),
            fullscreen,
            self.metrics.main_viewport,
        );
        let key = egui::Id::new(("ether-geometry-settlement", stage));
        let mut settled = ctx
            .data_mut(|data| data.remove_temp::<Settlement>(key))
            .unwrap_or_default();
        settled.tries += 1;
        settled.same = if ready && settled.sample == Some(sample) {
            settled.same + 1
        } else {
            usize::from(ready)
        };
        settled.sample = Some(sample);
        let stable = ready && settled.same >= 2;
        let tries = settled.tries;
        ctx.data_mut(|data| data.insert_temp(key, settled));
        assert!(
            tries <= 60,
            "LACP window failed to settle: actual_pixels={pixels:?}, PPP={ppp}, native_PPP={native_ppp}, fullscreen={fullscreen:?}"
        );
        if !stable {
            ctx.request_repaint();
            return (false, None);
        }
        let i = self.ether_index(2);
        assert_eq!(self.selected, Some(i));
        assert!(!self.metrics.ether.rejected);
        let screen = ctx.content_rect();
        for (name, r, _, _, _) in &self.metrics.ether.fields {
            assert!(
                r.left() >= screen.left() - 1. && r.right() <= screen.right() + 1.,
                "LACP field horizontal clipping: {name} {r:?} screen{screen:?}"
            );
        }
        for (name, r, _, _) in &self.metrics.ether.controls {
            assert!(
                r.left() >= screen.left() - 1. && r.right() <= screen.right() + 1.,
                "LACP control horizontal clipping: {name} {r:?} screen{screen:?}"
            );
        }
        // A long native disclosure title must not expand its child beyond the
        // settled viewport. Check the actual titles as well as their controls.
        for (name, r, _, _) in &self.metrics.ether.expanders {
            assert!(
                r.left() >= screen.left() - 1. && r.right() <= screen.right() + 1.,
                "LACP disclosure horizontal clipping: {name} {r:?} screen{screen:?}"
            );
        }
        for (n, _, r) in &self.metrics.ether.rows {
            assert!(
                r.left() >= screen.left() - 1. && r.right() <= screen.right() + 1.,
                "LACP compact outcome horizontal clipping: {n} {r:?} screen{screen:?}"
            );
        }
        assert!(
            self.metrics.ether.detail_rows.is_empty(),
            "Original diagnostics stay optional across appearance changes"
        );
        let rect = if phase == 1 {
            self.metrics
                .ether
                .fields
                .iter()
                .find(|r| r.0.ends_with(":Device:Vlans"))
                .unwrap()
                .1
        } else if phase == 2 {
            let d = draft(&self.progress.attempts[IDS[2]], false);
            let name = port_key(&d.selected_device, "p1", "Details");
            self.metrics
                .ether
                .expanders
                .iter()
                .find(|r| r.0 == name)
                .unwrap()
                .1
        } else {
            let rows = &self.metrics.ether.rows;
            assert_eq!(rows.len(), 21);
            self.metrics
                .ether
                .first_failure
                .expect("Actual first specific capstone failure")
                .1
        };
        if !self.reveal_smoke_control(ctx, rect) {
            return (false, None);
        }
        let before = ctx
            .data(|d| d.get_temp::<serde_json::Value>(egui::Id::new("ether-alignment-portable")))
            .unwrap();
        assert_eq!(portable(&self.progress), before);
        println!(
            "LACP alignment case{case}/phase{phase}: actual_pixels={pixels:?} actual_PPP={} zoom={} columns={}",
            ctx.pixels_per_point(),
            ctx.zoom_factor(),
            self.metrics.ether.columns
        );
        (
            true,
            Some(format!(
                "lacp-alignment-{}-{}-{}-{}",
                if narrow { "narrow" } else { "wide" },
                if light { "light" } else { "dark" },
                (scale * 100.).round() as u32,
                match phase {
                    1 => "fields",
                    2 => "members",
                    3 => "feedback",
                    _ => unreachable!("Closed geometry phase"),
                }
            )),
        )
    }
    fn ether_spacing_ready(
        &self,
        ctx: &egui::Context,
        stage: usize,
        scale: f32,
        narrow: bool,
    ) -> bool {
        // Precompute Context observations before touching its data lock.
        let ppp = ctx.pixels_per_point();
        let pixels = ctx.content_rect().size() * ppp;
        let fullscreen = ctx.input(|i| i.viewport().fullscreen);
        let native = ctx
            .input(|i| i.viewport().native_pixels_per_point)
            .unwrap_or(1.);
        let ready = (ctx.zoom_factor() - scale).abs() < 0.01
            && (ppp - native * scale).abs() < 0.01
            && fullscreen == Some(!narrow)
            && if narrow {
                (650. ..=1300.).contains(&pixels.x)
            } else {
                pixels.x >= 1500.
            };
        type SpacingGeometry = (egui::Rect, u32, Option<egui::Rect>);
        #[derive(Clone, Copy, Default)]
        struct Stable {
            sample: Option<SpacingGeometry>,
            same: usize,
            tries: usize,
        }
        let sample = (
            ctx.content_rect(),
            ppp.to_bits(),
            self.metrics.main_viewport,
        );
        let key = egui::Id::new(("ether-spacing-settled", stage));
        let mut state = ctx
            .data_mut(|d| d.remove_temp::<Stable>(key))
            .unwrap_or_default();
        state.tries += 1;
        state.same = if ready && state.sample == Some(sample) {
            state.same + 1
        } else {
            usize::from(ready)
        };
        state.sample = Some(sample);
        assert!(
            state.tries <= 60,
            "Spacing viewport did not settle: {pixels:?}/{ppp}/{fullscreen:?}"
        );
        let settled = ready && state.same >= 2;
        ctx.data_mut(|d| d.insert_temp(key, state));
        settled
    }
    /// Narrow follow-up for the two tasks with no retired-members requirement.
    /// Actual Start/Assignment/Check controls establish initial failure; no
    /// learner answers, results or loaded Progress are installed by this route.
    pub(in crate::ui) fn etherchannel_spacing_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert!(self.smoke && self._smoke_profile.is_some());
        assert!(self.pending_events.len() <= 300);
        assert!((49700..=49724).contains(&stage));
        let frames = ctx.data_mut(|d| {
            let key = egui::Id::new(("ether-spacing-stage-frames", stage));
            let n = d.get_temp::<usize>(key).unwrap_or(0) + 1;
            d.insert_temp(key, n);
            n
        });
        assert!(
            frames <= 900,
            "Closed spacing stage{stage} exceeded its frame budget"
        );
        let offset = stage - 49700;
        if offset == 24 {
            let tags = ctx
                .data(|d| d.get_temp::<Vec<String>>(egui::Id::new("ether-spacing-tags")))
                .unwrap();
            assert_eq!(tags.len(), 8);
            assert_eq!(
                tags.iter().collect::<std::collections::BTreeSet<_>>().len(),
                8
            );
            println!(
                "PASS native LACP spacing:8 controlled first-two-task frames,100/200 light/dark, actual Start/Assignment/Check, no empty retired row, singular pair, unchanged graded inputs."
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return (false, None);
        }
        let n = offset / 12;
        let phase = offset % 12;
        assert!(n < 2);
        match phase {
            0 => {
                self.ether_install_sources();
                assert!(self.selected.is_none());
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.filter = Filter {
                    track: "CCNA".into(),
                    search: self.scenarios[self.ether_index(n)].title.clone(),
                    ..Default::default()
                };
                self.page = 0;
            }
            1 => {
                if !self.ether_spacing_ready(ctx, stage, 1., false) {
                    return (false, None);
                }
                if !self.ether_click_completed(ctx) {
                    assert_eq!(self.metrics.card_actions.len(), 1);
                }
                if !self.ether_click(ctx, self.metrics.card_actions.first().map(|r| r.0)) {
                    return (false, None);
                }
            }
            2 => {
                if !self.ether_tab(ctx, teaching::View::Assignment) {
                    return (false, None);
                }
            }
            3 => {
                if !self.ether_control(ctx, "check") {
                    return (false, None);
                }
            }
            5 | 7 | 9 => {
                self.scale = if phase == 5 { 1. } else { 2. };
                self.theme_id = if phase == 7 {
                    "dlms-dark"
                } else {
                    "dlms-light"
                }
                .into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(phase == 5));
                if phase != 5 {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(946., 929.)));
                }
            }
            4 | 6 | 8 | 10 => {
                let narrow = phase >= 8;
                let scale = if narrow { 2. } else { 1. };
                if !self.ether_spacing_ready(ctx, stage, scale, narrow) {
                    return (false, None);
                }
                assert!(!self.metrics.completion);
                assert_eq!(self.metrics.ether.rows.len(), 7);
                assert!(self.metrics.ether.detail_rows.is_empty());
                assert!(
                    self.metrics.ether.compact_retired.is_none(),
                    "No empty retired-members row may reserve space"
                );
                let current = self.metrics.ether.compact_current.unwrap();
                let preserved = self.metrics.ether.compact_preserved.unwrap();
                let expected_gap = ctx.style().spacing.item_spacing.y;
                assert!(
                    ((preserved.top() - current.bottom()) - expected_gap).abs()
                        <= 2. / ctx.pixels_per_point(),
                    "Only the shared row spacing belongs between actual groups"
                );
                if !self.reveal_smoke_control(ctx, current.union(preserved)) {
                    return (false, None);
                }
                let i = self.ether_index(n);
                let attempt = &self.progress.attempts[IDS[n]];
                assert_eq!(attempt.state, self.scenarios[i].initial);
                assert!(!progress::current_solved(attempt, &self.scenarios[i]));
                let before_key = egui::Id::new(("ether-spacing-progress", n));
                let now = portable(&self.progress);
                let before = ctx.data(|d| d.get_temp::<serde_json::Value>(before_key));
                if let Some(before) = before {
                    assert_eq!(now, before, "Appearance cannot edit assignment/progress");
                } else {
                    ctx.data_mut(|d| d.insert_temp(before_key, now));
                }
                let tag = format!(
                    "lacp-spacing-{}-{}-{}-{}",
                    IDS[n],
                    if narrow { "narrow" } else { "wide" },
                    if phase == 4 || phase == 8 {
                        "dark"
                    } else {
                        "light"
                    },
                    (scale * 100.).round() as usize
                );
                let key = egui::Id::new("ether-spacing-tags");
                let mut tags = ctx
                    .data_mut(|d| d.remove_temp::<Vec<String>>(key))
                    .unwrap_or_default();
                assert!(tags.len() < 8 && !tags.contains(&tag));
                tags.push(tag.clone());
                ctx.data_mut(|d| d.insert_temp(key, tags));
                return (true, Some(tag));
            }
            11 => {
                if !self.ether_header(ctx, "back") {
                    return (false, None);
                }
            }
            _ => unreachable!("Closed spacing phase"),
        }
        assert!(self.pending_events.len() <= 300);
        (true, None)
    }
}
