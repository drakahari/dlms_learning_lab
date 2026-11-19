//! Fixed native-fixture actions only. No learner input, host editor or shell.
use super::*;
use dlms_learning_lab::{
    modal_source,
    modal_workspace::{self as work, Event},
};
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "lpic1-modal-setting-repair",
    "lpic1-modal-record-maintenance",
    "lpic1-modal-recovery",
];
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Click(String),
    Select(String),
    Keys(String),
    Setup(String),
    SetupChecked(bool, bool),
    Check(bool, bool),
    Shot(String, String),
    Style(f32, bool),
    Save,
    Fresh,
    Favorite,
    ReplayExact,
    Uncredited,
}
#[derive(Clone, Default)]
struct Flow {
    at: usize,
    phase: u8,
    polls: usize,
    next: f64,
    began: Option<Instant>,
}
fn edits(ops: &mut Vec<Op>, events: &[Event]) {
    let mut keys = String::new();
    let flush = |ops: &mut Vec<Op>, keys: &mut String| {
        if !keys.is_empty() {
            ops.push(Op::Keys(std::mem::take(keys)));
        }
    };
    for e in events {
        match e {
            Event::Key { key } => keys.push(match key {
                dlms_learning_lab::modal_editing::Key::Character(c) => *c,
                dlms_learning_lab::modal_editing::Key::Escape => '\x1b',
                dlms_learning_lab::modal_editing::Key::Enter => '\r',
                dlms_learning_lab::modal_editing::Key::Backspace => '\x08',
            }),
            Event::Open { path } => {
                flush(ops, &mut keys);
                ops.push(Op::Select(path.clone()));
                ops.push(Op::Click("open".into()));
            }
            Event::Check {} => {
                flush(ops, &mut keys);
                ops.push(Op::Click("check".into()));
            }
            Event::Insert { .. } => panic!(
                "These closed fixtures use actual typed key events, not direct model insertion"
            ),
        }
    }
    flush(ops, &mut keys);
}
fn plan() -> &'static Vec<Op> {
    static P: OnceLock<Vec<Op>> = OnceLock::new();
    P.get_or_init(|| {
        let labs = pack::load(include_str!(
            "../../../scenarios/modal-editing-lessons-v1.json"
        ))
        .unwrap();
        let mut ops = vec![];
        for (n, id) in IDS.iter().enumerate() {
            let s = labs.iter().find(|s| s.id == *id).unwrap();
            let d = modal_source::source(s).unwrap();
            let tag = |name: &str| format!("modal-{n}-{name}");
            ops.extend([
                Op::Style(1., false),
                Op::Open(id.to_string()),
                Op::Shot(tag("introduction"), "introduction".into()),
                Op::Shot(tag("worked"), "worked-table".into()),
                Op::Click("example".into()),
                Op::Shot(tag("practice"), "editor".into()),
            ]);
            if n == 0 {
                ops.extend([
                    Op::Keys("iX".into()),
                    Op::Shot(tag("insert-focus"), "editor".into()),
                    Op::Keys("\x1bu".into()),
                ]);
            }
            edits(&mut ops, &d.example_solution);
            ops.extend([
                Op::Click("check".into()),
                Op::Check(true, true),
                Op::Uncredited,
                Op::Shot(tag("practice-pass"), "results".into()),
                Op::Click("assignment".into()),
                Op::Shot(tag("assignment"), "editor".into()),
                Op::Click("check".into()),
                Op::Check(false, false),
                Op::Shot(tag("needs-attention"), "results".into()),
            ]);
            let events: Vec<Event> = s
                .solution
                .iter()
                .map(|s| serde_json::from_str(s.strip_prefix("modal-lab ").unwrap()).unwrap())
                .collect();
            edits(&mut ops, &events);
            ops.extend([
                Op::Check(true, false),
                Op::Save,
                Op::Shot(tag("assignment-pass"), "results".into()),
                Op::Keys("iBAD\x1b".into()),
                Op::Click("check".into()),
                Op::Check(false, false),
                Op::Shot(tag("unsaved-feedback"), "results".into()),
                Op::Keys("u".into()),
                Op::Click("check".into()),
                Op::Check(true, false),
                Op::Style(1.5, true),
                Op::Shot(tag("light150"), "editor".into()),
                Op::Style(2., false),
                Op::Shot(tag("dark200"), "editor".into()),
                Op::Click("reset-header".into()),
                Op::Click("confirm-reset".into()),
                Op::Fresh,
            ]);
        }
        ops
    })
}
fn handover_plan() -> &'static Vec<Op> {
    static P: OnceLock<Vec<Op>> = OnceLock::new();
    P.get_or_init(|| vec![
        Op::Style(1.,false),Op::Open("lpic1-editor-handover".into()),
        Op::Shot("editor-handover-introduction".into(),"introduction".into()),
        Op::Shot("editor-handover-worked".into(),"worked-table".into()),
        Op::Click("example".into()),Op::Shot("editor-handover-practice-setup".into(),"setup-editor".into()),
        Op::Click("run-setup".into()),Op::SetupChecked(false,true),
        Op::Shot("editor-handover-missing-export".into(),"run-setup".into()),
        Op::Click("actual-setup".into()),Op::Shot("editor-handover-actual-comparison".into(),"actual-setup-body".into()),Op::Click("actual-setup".into()),
        Op::Setup("export EDITOR=\"$PREFERRED_EDITOR\"\n".into()),
        Op::Click("run-setup".into()),Op::SetupChecked(true,true),
        Op::Click("open-handover".into()),Op::Shot("editor-handover-practice-editor".into(),"editor".into()),
        Op::Keys("/OWNER\rddGoOWNER training\rREVIEW practice\x1b:w\r".into()),
        Op::Click("check".into()),Op::Check(true,true),Op::Uncredited,
        Op::Shot("editor-handover-practice-pass".into(),"results".into()),
        Op::Click("assignment".into()),Op::Shot("editor-handover-assignment".into(),"setup-editor".into()),
        Op::Click("check".into()),Op::Check(false,false),
        Op::Setup("EDITOR=$PREFERRED_EDITOR\n".into()),Op::Click("run-setup".into()),Op::SetupChecked(false,false),
        Op::Shot("editor-handover-assignment-feedback".into(),"run-setup".into()),
        Op::Setup("EDITOR=$PREFERRED_EDITOR\nexport EDITOR\n".into()),Op::Click("run-setup".into()),Op::SetupChecked(true,false),
        Op::Click("open-handover".into()),Op::Shot("editor-handover-independent-editor".into(),"editor".into()),
        Op::Keys("/OWNER\rddGoOWNER platform\rREVIEW ready\x1b".into()),Op::Click("check".into()),Op::Check(false,false),
        Op::Shot("editor-handover-unsaved".into(),"results".into()),
        Op::Keys(":w\r".into()),Op::Click("check".into()),Op::Check(true,false),Op::Save,
        Op::Shot("editor-handover-complete".into(),"results".into()),
        Op::Setup("EDITOR=wrong\n".into()),Op::Click("check".into()),Op::Check(false,false),
        Op::Shot("editor-handover-stale-choice-cleared".into(),"results".into()),
        Op::Setup("export EDITOR=\"$PREFERRED_EDITOR\"\n".into()),Op::Click("run-setup".into()),Op::SetupChecked(true,false),
        Op::Click("open-handover".into()),Op::Click("check".into()),Op::Check(true,false),
        Op::Style(1.5,true),Op::Shot("editor-handover-light150".into(),"editor".into()),
        Op::Style(2.,false),Op::Shot("editor-handover-dark200".into(),"editor".into()),
        Op::Click("reset-header".into()),Op::Click("confirm-reset".into()),Op::Fresh,
    ])
}
const HANDOVER_DRAFT: &str = "# Independent practice draft\nEDITOR='unfinished";
fn handover_cold_plan(read: bool) -> &'static Vec<Op> {
    static WRITER: OnceLock<Vec<Op>> = OnceLock::new();
    static READER: OnceLock<Vec<Op>> = OnceLock::new();
    if read { READER.get_or_init(|| vec![
        Op::ReplayExact,
        Op::Shot("reopen-handover-read-library".into(), "library".into()),
        Op::Open("lpic1-editor-handover".into()), Op::Click("assignment".into()),
        Op::Check(true,false), Op::Shot("reopen-handover-read-assignment".into(),"editor".into()),
        Op::Click("example".into()), Op::Shot("reopen-handover-read-draft".into(),"setup-editor".into()),
        Op::Setup("export EDITOR=\"$PREFERRED_EDITOR\"\n".into()),
        Op::Click("run-setup".into()), Op::SetupChecked(true,true), Op::Click("open-handover".into()),
        Op::Shot("reopen-handover-read-partial-buffer".into(),"editor".into()),
        Op::Keys("\x1bu".into()), Op::Click("check".into()), Op::Check(true,true),
        Op::Click("assignment".into()), Op::Check(true,false), Op::Click("example".into()), Op::Save,
        Op::Shot("reopen-handover-read-repaired".into(),"results".into()),
    ]) } else { WRITER.get_or_init(|| {
        let mut ops: Vec<_> = handover_plan().iter().take_while(|op| !matches!(op,Op::Save)).cloned().collect();
        ops.extend([
            Op::Click("example".into()), Op::Keys("iDRAFT".into()),
            Op::Setup(HANDOVER_DRAFT.into()), Op::Favorite, Op::Style(1.5,true), Op::Save,
            Op::Shot("reopen-handover-write-drafts".into(),"setup-editor".into()),
        ]);
        ops
    }) }
}
pub(in crate::ui) fn handover_cold_capture_allowed(name: &str) -> bool {
    [false,true].into_iter().flat_map(|r| handover_cold_plan(r).iter()).any(|op|matches!(op,Op::Shot(n,_)if n==name))
}
fn handover_current(a: &LabApp) -> (&dlms_learning_lab::editor_handover::Definition, &dlms_learning_lab::editor_handover::Session) {
    let source=&a.scenarios[a.selected.unwrap()];let attempt=&a.progress.attempts[&source.id];
    dlms_learning_lab::editor_handover_source::context(attempt,source).unwrap();
    let (engine::Goal::EditorHandover{definition},engine::State::EditorHandover{simulation})=(&source.goal,&attempt.state)else{panic!()};
    (definition,simulation)
}
fn current(a: &LabApp) -> (&work::Definition, &work::Session) {
    let s = &a.scenarios[a.selected.unwrap()];
    let engine::State::ModalEditing { simulation } = &a.progress.attempts[&s.id].state else {
        panic!()
    };
    (modal_source::source(s).unwrap(), simulation)
}
fn target(a: &LabApp, k: &str) -> Option<egui::Rect> {
    match k {
        "library" => a.metrics.main_viewport,
        "reset-header" => a.metrics.reset,
        "confirm-reset" => a.metrics.confirm_reset,
        "editor" => a.metrics.modal.editor.map(|(r, _, _)| r),
        "results" => a.metrics.modal.output,
        "worked-table" => a.metrics.example_action,
        _ => a
            .metrics
            .modal
            .controls
            .iter()
            .find(|r| r.0 == k)
            .map(|r| r.1),
    }
}
fn native_keys(text: &str) -> Vec<egui::Event> {
    let mut events = vec![];
    let mut typed = String::new();
    let flush = |events: &mut Vec<_>, typed: &mut String| {
        if !typed.is_empty() {
            events.push(egui::Event::Text(std::mem::take(typed)));
        }
    };
    for c in text.chars() {
        let key = match c {
            '\x1b' => Some(egui::Key::Escape),
            '\r' => Some(egui::Key::Enter),
            '\x08' => Some(egui::Key::Backspace),
            _ => None,
        };
        if let Some(key) = key {
            flush(&mut events, &mut typed);
            for pressed in [true, false] {
                events.push(egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: Default::default(),
                });
            }
        } else {
            typed.push(c);
        }
    }
    flush(&mut events, &mut typed);
    events
}
impl LabApp {
    fn modal_parent_wheel(&mut self, ctx: &egui::Context) {
        if !self
            .pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        {
            return;
        }
        let v = self
            .metrics
            .main_viewport
            .expect("Measured parent viewport");
        let region = self
            .metrics
            .main_scroll_region
            .expect("Measured parent scrollbar region");
        let gutter = egui::Rect::from_min_max(
            egui::pos2(v.right(), v.top()),
            egui::pos2(region.right(), v.bottom()),
        )
        .intersect(ctx.content_rect());
        assert!(
            gutter.width() * ctx.pixels_per_point() > 1.,
            "Need the actual outer scrollbar gutter"
        );
        for event in &mut self.pending_events {
            if let egui::Event::PointerMoved(point) = event {
                *point = gutter.center();
            }
        }
    }
    pub(in crate::ui) fn modal_lessons_tick(&mut self, ctx: &egui::Context) { self.modal_flow_tick(ctx,plan(),false); }
    pub(in crate::ui) fn editor_handover_tick(&mut self, ctx: &egui::Context) { self.modal_flow_tick(ctx,handover_plan(),true); }
    pub(in crate::ui) fn editor_handover_cold_tick(&mut self, ctx: &egui::Context, read: bool) {
        self.modal_flow_tick(ctx, handover_cold_plan(read), true);
    }
    fn modal_flow_tick(&mut self, ctx: &egui::Context, ops: &[Op], handover: bool) {
        let key = egui::Id::new(if handover {"editor-handover-native"} else {"modal-lessons-native"});
        let mut f = ctx.data(|d| d.get_temp::<Flow>(key)).unwrap_or_default();
        if f.at == ops.len() {
            if self.reopen.is_some() { self.finish_editor_handover_reopen(ctx); return; }
            println!("MODAL_NATIVE_COMPLETE {} fixed actions", f.at);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let now = ctx.input(|i| i.time);
        let began = f.began.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed() < std::time::Duration::from_secs(15),
            "Modal native deadline at {} {:?}",
            f.at,
            ops[f.at]
        );
        if now < f.next {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        f.polls += 1;
        assert!(f.polls < 1200, "Modal native poll limit");
        f.next = now + 0.1;
        if f.polls == 1 {
            println!("MODAL_NATIVE_STEP {} {:?}", f.at, ops[f.at]);
        }
        let complete = match ops[f.at].clone() {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                true
            }
            Op::Click(k) => self.click_visible_smoke_control(ctx, target(self, &k)),
            Op::Select(path) => {
                if f.phase == 0 {
                    if self.click_visible_smoke_control(ctx, target(self, "file")) {
                        f.phase = 1;
                    }
                    false
                } else {
                    let acknowledged = egui::Id::new(("native-click-phases", self.smoke_stage));
                    if ctx.data(|d| d.get_temp::<u8>(acknowledged)).unwrap_or(0) >= 2 {
                        assert!(self.click_smoke_control(ctx, None));
                        true
                    } else {
                        let rect = target(self, &format!("file:{path}"))
                            .expect("source-listed file choice");
                        let clip = self
                            .metrics
                            .modal
                            .popup
                            .unwrap()
                            .intersect(ctx.content_rect());
                        if smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
                            self.click_smoke_control(ctx, Some(rect));
                        }
                        false
                    }
                }
            }
            Op::Keys(text) => {
                if f.phase == 0 {
                    if self.click_visible_smoke_control(ctx, target(self, "editor")) {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    let id = self.metrics.modal.editor.unwrap().2;
                    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
                    self.pending_events.extend(native_keys(&text));
                    f.phase = 2;
                    false
                } else {
                    true
                }
            }
            Op::Setup(source) => {
                if f.phase==0 {if self.click_visible_smoke_control(ctx,target(self,"setup-editor")){f.phase=1;}false}
                else if f.phase==1 {
                    assert_eq!(ctx.memory(|m|m.focused()),self.metrics.focus);
                    self.pending_events.extend([egui::Event::Key{key:egui::Key::A,physical_key:None,pressed:true,repeat:false,modifiers:egui::Modifiers{command:true,..egui::Modifiers::CTRL}},egui::Event::Text(source.clone())]);f.phase=2;false
                }else{let(_,s)=handover_current(self);let practice=s.view==teaching::View::Example;assert_eq!(s.draft(practice).source,source);assert!(s.draft(practice).result.is_none());assert!(s.launch(practice).is_none());true}
            }
            Op::SetupChecked(pass,practice) => {
                if self.editor_handover_run.is_some(){false}else{let(d,s)=handover_current(self);assert!(s.draft(practice).result.is_some(),"Actual run required");assert_eq!(s.ready(d,practice),pass,"{:?}",s.checks(d,practice));true}
            }
            Op::Check(pass, practice) => {
                if handover {let(d,s)=handover_current(self);assert_eq!(s.checks(d,practice).iter().all(|r|r.0),pass,"{:?}",s.checks(d,practice));if !practice{assert_eq!(self.metrics.completion,pass);}true}else{
                let (d, s) = current(self);
                let a = if practice { &s.practice } else { &s.assignment };
                let fixture = if practice { &d.example } else { &d.initial };
                assert_eq!(a.solved(fixture), pass, "{:?}", a.checks(fixture));
                if !practice {
                    assert_eq!(self.metrics.completion, pass);
                }
                true }
            }
            Op::Uncredited => {
                if handover {let(d,s)=handover_current(self);let initial=d.new_session().unwrap();assert_eq!(s.setup.assignment,initial.setup.assignment);assert_eq!(s.files.assignment,initial.files.assignment);assert!(!s.solved(d));true}else{
                let (d, s) = current(self);
                assert_eq!(s.assignment, d.new_session().unwrap().assignment);
                assert!(!s.assignment.checked);
                true }
            }
            Op::Fresh => {
                if handover {let(d,s)=handover_current(self);assert_eq!(*s,d.new_session().unwrap());}else{let (d,s)=current(self);assert_eq!(*s,d.new_session().unwrap());}true
            }
            Op::Favorite => {
                assert!(handover && self.reopen.is_some());
                self.progress.favorites.insert("lpic1-editor-handover".into());
                self.dirty = true;
                true
            }
            Op::ReplayExact => self.editor_handover_replayed(ctx),
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none(), "{:?}", self.storage_error);
                let read = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(read).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Style(scale, light) => {
                self.scale = scale;
                ctx.set_zoom_factor(scale);
                self.theme_id = theme::inventory()
                    .iter()
                    .find(|t| t.light == light)
                    .unwrap()
                    .id
                    .clone();
                theme::apply(ctx, &self.theme_id);
                if self.reopen.is_some() {
                    assert!(!self.preferences_read_only);
                    theme::Preferences {version:1,theme:self.theme_id.clone(),text_scale:self.scale}
                        .save(&self.path.with_file_name("preferences-v1.json")).unwrap();
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(scale == 1.));
                if scale != 1. {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                        1100., 850.,
                    )));
                }
                self.lesson_scroll_reset = true;
                true
            }
            Op::Shot(name, k) => {
                if f.phase == 0 {
                    if self.reveal_smoke_control(ctx, target(self, &k).expect("capture anchor")) {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    let rect = target(self, &k).expect("Settled capture anchor");
                    let visible = self
                        .metrics
                        .main_viewport
                        .unwrap()
                        .intersect(ctx.content_rect());
                    if !smoke_control_visible(rect, visible, ctx.pixels_per_point()) {
                        f.phase = 0;
                        self.reveal_smoke_control(ctx, rect);
                        self.modal_parent_wheel(ctx);
                        ctx.data_mut(|d| d.insert_temp(key, f));
                        ctx.request_repaint_after(std::time::Duration::from_millis(25));
                        return;
                    }
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name,
                    )));
                    f.phase = 2;
                    false
                } else {
                    !self.capture_pending
                }
            }
        };
        self.modal_parent_wheel(ctx);
        if complete {
            // Twenty attempts applies to this reveal/action, not cumulatively
            // to every separate target in the complete fixture.
            let scroll_key = egui::Id::new(("bounded-native-scroll", self.smoke_stage));
            ctx.data_mut(|d| {
                d.remove::<usize>(scroll_key);
            });
            f.at += 1;
            f.phase = 0;
            f.polls = 0;
            f.began = None;
        }
        ctx.data_mut(|d| d.insert_temp(key, f));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}
#[test]
fn editor_handover_native_plan_types_setup_checks_actual_children_and_saved_files() {
    let p=handover_plan();
    assert_eq!(p.iter().filter(|op|matches!(op,Op::Open(_))).count(),1);
    assert!(p.iter().filter(|op|matches!(op,Op::Setup(_))).count()>=4);
    assert!(p.iter().any(|op|matches!(op,Op::SetupChecked(false,true))));
    assert!(p.iter().any(|op|matches!(op,Op::SetupChecked(true,false))));
    assert!(p.iter().any(|op|matches!(op,Op::Keys(s)if s.contains("OWNER platform")&&s.contains("REVIEW ready"))));
    assert!(p.iter().any(|op|matches!(op,Op::Uncredited)));
    assert_eq!(p.iter().filter(|op|matches!(op,Op::Save)).count(),1);
    assert!(matches!(p.last(),Some(Op::Fresh)));
    assert!(p.iter().any(|op|matches!(op,Op::Shot(name,_)if name=="editor-handover-stale-choice-cleared")));
}
#[test]
fn modal_native_plan_has_all_three_assignments_separate_practice_and_actual_key_actions() {
    let p = plan();
    assert_eq!(p.iter().filter(|op| matches!(op, Op::Open(_))).count(), 3);
    assert_eq!(p.iter().filter(|op| matches!(op, Op::Fresh)).count(), 3);
    assert_eq!(
        p.iter()
            .filter(|op| matches!(op, Op::Check(true, true)))
            .count(),
        3
    );
    assert!(p.iter().any(|op| matches!(op, Op::Select(_))));
    assert!(
        p.iter()
            .any(|op| matches!(op,Op::Keys(t)if t.contains("yy")))
    );
}
#[test]
fn modal_capture_rechecks_late_clipping_and_scrolls_the_actual_parent_gutter() {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    let clip = egui::Rect::from_min_max(egui::pos2(10., 10.), egui::pos2(450., 350.));
    a.metrics.main_viewport = Some(clip);
    a.metrics.main_scroll_region = Some(clip.expand2(egui::vec2(10., 0.)));
    a.metrics.modal.editor = Some((
        egui::Rect::from_min_size(egui::pos2(20., 400.), egui::vec2(200., 100.)),
        clip,
        egui::Id::new("editor"),
    ));
    let at = plan()
        .iter()
        .position(|p| matches!(p,Op::Shot(_,k)if k=="editor"))
        .unwrap();
    let id = egui::Id::new("modal-lessons-native");
    ctx.data_mut(|d| {
        d.insert_temp(
            id,
            Flow {
                at,
                phase: 1,
                ..Default::default()
            },
        )
    });
    let raw = || egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(500., 400.),
        )),
        ..Default::default()
    };
    let _ = ctx.run(raw(), |ctx| a.modal_lessons_tick(ctx));
    let flow = ctx.data(|d| d.get_temp::<Flow>(id)).unwrap();
    assert_eq!(flow.at, at);
    assert_eq!(flow.phase, 0);
    assert!(!a.capture_pending);
    assert!(
        a.pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
    );
    let point = a
        .pending_events
        .iter()
        .find_map(|e| {
            if let egui::Event::PointerMoved(p) = e {
                Some(*p)
            } else {
                None
            }
        })
        .unwrap();
    assert!(point.x > clip.right() && a.metrics.main_scroll_region.unwrap().contains(point));
    a.pending_events.clear();
    a.metrics.modal.editor = Some((
        egui::Rect::from_min_size(egui::pos2(20., 20.), egui::vec2(200., 100.)),
        clip,
        egui::Id::new("editor"),
    ));
    ctx.data_mut(|d| {
        d.insert_temp(
            id,
            Flow {
                at,
                phase: 1,
                ..Default::default()
            },
        )
    });
    let _ = ctx.run(raw(), |ctx| a.modal_lessons_tick(ctx));
    assert!(a.capture_pending);
}
#[test]
fn modal_native_scroll_attempts_reset_only_between_completed_actions() {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    let scroll = egui::Id::new(("bounded-native-scroll", a.smoke_stage));
    ctx.data_mut(|d| d.insert_temp(scroll, 19usize));
    let _ = ctx.run(egui::RawInput::default(), |ctx| a.modal_lessons_tick(ctx));
    assert!(ctx.data(|d| d.get_temp::<usize>(scroll)).is_none());
    let flow = ctx
        .data(|d| d.get_temp::<Flow>(egui::Id::new("modal-lessons-native")))
        .unwrap();
    assert_eq!(flow.at, 1);
}

#[test]
fn handover_cold_plans_keep_actual_edits_partial_drafts_and_closed_capture_names() {
    let writer=handover_cold_plan(false);
    let reader=handover_cold_plan(true);
    assert!(writer.len()<64 && reader.len()<32);
    assert!(writer.iter().any(|op|matches!(op,Op::Check(true,false))));
    assert!(writer.iter().any(|op|matches!(op,Op::Keys(s)if s=="iDRAFT")));
    assert!(writer.iter().any(|op|matches!(op,Op::Setup(s)if s==HANDOVER_DRAFT)));
    assert!(matches!(reader.first(),Some(Op::ReplayExact)));
    assert!(reader.iter().any(|op|matches!(op,Op::Keys(s)if s=="\x1bu")));
    let last_check=reader.iter().rposition(|op|matches!(op,Op::Check(true,false))).unwrap();
    assert!(matches!(&reader[last_check-1],Op::Click(k)if k=="assignment"));
    assert!(matches!(&reader[last_check+1],Op::Click(k)if k=="example"));
    assert!(!writer.iter().chain(reader).any(|op|matches!(op,Op::Fresh)));
    let names:std::collections::BTreeSet<_>=writer.iter().chain(reader).filter_map(|op|if let Op::Shot(n,_) = op {Some(n)}else{None}).collect();
    let count=writer.iter().chain(reader).filter(|op|matches!(op,Op::Shot(..))).count();
    assert_eq!(names.len(),count);
    for n in names { assert!(handover_cold_capture_allowed(n)); }
    for bad in ["../capture","reopen-arbitrary","reopen-handover-read-library.png",""] { assert!(!handover_cold_capture_allowed(bad)); }
    assert_eq!(handover_plan().len(),61);
    assert_eq!(plan().len(),114);
}
