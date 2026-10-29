//! Closed native acceptance over the four normal registered backup assignments.
//! All changes arrive through real library, terminal and course controls.
use super::nat::{Queue, Warm, exact};
use super::*;
use dlms_learning_lab::{file_backup_teaching as teach, script_teaching::View};
use serde_json::Value;
pub(in crate::ui) const FIRST: usize = 62000;
const END: usize = 63000;
pub(in crate::ui) const IDS: [&str; 4] = [
    "lpic2-backup-selective-recovery",
    "lpic2-backup-incremental-chain",
    "lpic2-backup-snapshot-branch",
    "lpic2-backup-recovery-handover",
];
#[derive(Clone, Debug)]
enum Target {
    Control(&'static str),
    Tab(View),
    Insert(usize),
    Input,
    Run,
    Output,
    Failure,
    Heading,
    GoalStart,
    GoalEnd,
    Card(bool),
    Back,
    Reset,
    Cancel,
    Confirm,
    Challenge,
    UnitPicker,
    UnitChoice,
    Next,
    UnitComplete,
}
#[derive(Clone, Debug)]
enum Step {
    Select(usize),
    Next(usize),
    Click(Target),
    Open,
    BeforePractice,
    BeforeInsert,
    InsertOnly(usize),
    Command(String, bool),
    PracticeExact,
    Check(bool, bool),
    Rows(Vec<bool>),
    Snapshot,
    ResetExact,
    Retained,
    Warm,
    ResumeExact,
    Theme(bool),
    Geometry(f32, bool),
    Shot(String, Target),
    StopActive,
    StopExact,
    StoppedRetained,
    Text(String),
    Keyboard,
    Undo,
    Finish,
}
fn learning(a: &Attempt) -> &teach::Learning {
    a.workspace
        .as_ref()
        .unwrap()
        .file_backup_learning
        .as_ref()
        .unwrap()
}
fn command(a: &Attempt, practice: bool) -> &str {
    if practice {
        &learning(a).command
    } else {
        &a.workspace.as_ref().unwrap().command
    }
}
fn portable(a: &LabApp) -> Value {
    serde_json::to_value(&a.progress).unwrap()
}
fn receipts(a: &LabApp) -> Value {
    serde_json::to_value(&a.progress.migration_receipts).unwrap()
}
fn key(name: &str) -> egui::Id {
    egui::Id::new(("backup-native", name))
}
fn ctrl(k: egui::Key, pressed: bool) -> egui::Event {
    egui::Event::Key {
        key: k,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    }
}
fn check(p: &mut Vec<Step>, pass: bool, practice: bool) {
    p.extend([
        Step::Click(Target::Control("check")),
        Step::Check(pass, practice),
    ]);
}
fn reset(p: &mut Vec<Step>) {
    p.extend([
        Step::Snapshot,
        Step::Click(Target::Reset),
        Step::Click(Target::Confirm),
        Step::ResetExact,
        Step::Click(Target::Tab(View::Assignment)),
    ]);
}
fn commands(p: &mut Vec<Step>, rows: &[String], practice: bool) {
    p.extend(rows.iter().cloned().map(|s| Step::Command(s, practice)));
}
fn sources() -> &'static Vec<Scenario> {
    static ALL: std::sync::OnceLock<Vec<Scenario>> = std::sync::OnceLock::new();
    ALL.get_or_init(|| {
        let all = pack::bundled().unwrap();
        assert_eq!(
            all.len(),
            168,
            "Normal registration is required; never append a private fixture"
        );
        let tail = all[164..168].to_vec();
        assert_eq!(tail.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), IDS);
        tail
    })
}
fn review() -> &'static Value {
    static R: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    R.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../tests/fixtures/file-backup-content-review.json"
        ))
        .unwrap()
    })
}
fn vectors() -> &'static Value {
    static R: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    R.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../tests/fixtures/file-backup-native-rows-v1.json"
        ))
        .unwrap()
    })
}
fn strings(v: &Value) -> Vec<String> {
    serde_json::from_value(v.clone()).unwrap()
}
fn plan() -> &'static Vec<Step> {
    static P: std::sync::OnceLock<Vec<Step>> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        let mut p = Vec::new();
        for (n,s) in sources().iter().enumerate() {
            let r=&review()["labs"][n]; let v=&vectors()["rows"][n];
            assert_eq!(r["id"],s.id);assert_eq!(v["id"],s.id);
            let name=|part:&str|format!("backup-{n}-{part}");
            let lesson=s.file_backup_teaching.as_ref().unwrap();
            p.extend([Step::Select(n),Step::Click(Target::Card(true)),Step::Click(Target::Card(false)),
                Step::Open,Step::Shot(name("introduction"),Target::Heading),
                Step::Click(Target::Control("try-example")),Step::BeforePractice,Step::BeforeInsert,
                Step::Click(Target::Insert(0)),Step::InsertOnly(0)]);
            // Insert is independently proven inert, then actual Run performs the first command.
            for c in &lesson.example_commands {p.push(Step::Command(c.command.clone(),true));}
            check(&mut p,true,true);p.push(Step::Shot(name("example-output"),Target::Output));
            for c in &lesson.try_change.commands {p.push(Step::Command(c.command.clone(),true));}
            check(&mut p,false,true);
            p.push(Step::Rows(lesson.try_change.expected.clone()));
            p.push(Step::Shot(name("practice-mistake"),Target::Failure));
            for c in &lesson.repair {p.push(Step::Command(c.command.clone(),true));}
            check(&mut p,true,true);
            p.extend([Step::PracticeExact,Step::Warm,Step::Click(Target::Tab(View::Assignment)),
                Step::Shot(name("assignment"),Target::Heading)]);
            commands(&mut p,&strings(&r["wrong"]),false);check(&mut p,false,false);
            p.push(Step::Rows(serde_json::from_value(v["wrong"].clone()).unwrap()));
            p.push(Step::Shot(name("wrong-feedback"),Target::Failure));reset(&mut p);
            commands(&mut p,&strings(&r["partial"]),false);check(&mut p,false,false);
            p.push(Step::Rows(serde_json::from_value(v["partial"].clone()).unwrap()));
            p.push(Step::Shot(name("partial-progress"),Target::Failure));reset(&mut p);
            commands(&mut p,&s.solution,false);check(&mut p,true,false);
            p.push(Step::Shot(name("objectives-complete"),Target::Control("check")));
            // Cancel an actual Reset dialog and preserve all portable fields and receipts.
            p.extend([Step::Snapshot,Step::Click(Target::Reset),Step::Click(Target::Cancel),Step::Retained]);
            if n==0 {
                p.push(Step::Keyboard);
                p.push(Step::Undo);
                check(&mut p,true,false);
                // Representative lab0: twenty focused command captures and twenty separate output captures; not an all-lab matrix.
                for scale in [1.,1.15,1.25,1.5,2.] {for narrow in [false,true] {for light in [false,true] {
                    let label=format!("backup-alignment-{}-{}-{:.0}",if narrow{"narrow"}else{"wide"},if light{"light"}else{"dark"},scale*100.);
                    p.extend([Step::Theme(light),Step::Geometry(scale,narrow),Step::Click(Target::Input),
                        Step::Shot(format!("{label}-command"),Target::Input),
                        Step::Shot(format!("{label}-output"),Target::Output)]);
                }}}
                p.extend([Step::Theme(false),Step::Geometry(1.,false)]);
            }
            // A genuine unsaved assignment draft survives back/resume without losing either scope.
            p.extend([Step::Text("tar -tf /backups/unfinished.tar".into()),Step::Warm,
                Step::Click(Target::Back),Step::Click(Target::Card(false)),Step::ResumeExact]);
            check(&mut p,true,false);
            commands(&mut p,&strings(&r["break"]),false);check(&mut p,false,false);
            p.push(Step::Rows(serde_json::from_value(v["damage"].clone()).unwrap()));
            p.push(Step::Shot(name("success-invalidated"),Target::Failure));reset(&mut p);
            commands(&mut p,&strings(&r["alternate"]),false);check(&mut p,true,false);
            reset(&mut p);p.push(Step::Click(Target::Challenge));
            p.push(Step::Shot(name("challenge-full-task"),Target::Heading));
            // Independent supported tokenization: explicit cp -- and reordered long tar options.
            let second:Vec<String>=s.solution.iter().map(|line| {
                if let Some(t)=line.strip_prefix("cp "){format!("cp -- {t}")}
                else if let Some(t)=line.strip_prefix("tar -xf ") {
                    let (archive,tail)=t.split_once(" -C ").unwrap();
                    let (dest,member)=tail.split_once(' ').unwrap();
                    format!("tar --directory {dest} --file {archive} --extract -- {member}")
                } else if let Some(t)=line.strip_prefix("tar -xGf ") {
                    let (archive,dest)=t.split_once(" -C ").unwrap();
                    format!("tar --directory {dest} --incremental --file {archive} --extract")
                } else if let Some(t)=line.strip_prefix("tar -cf ") {
                    let (archive,t)=t.split_once(" -g ").unwrap();let (snapshot,t)=t.split_once(" -C ").unwrap();
                    let (dest,member)=t.split_once(' ').unwrap();
                    format!("tar --directory {dest} --listed-incremental {snapshot} --file {archive} --create -- {member}")
                } else {panic!("Closed authored second alternate must be explicitly classified")}
            }).collect();
            commands(&mut p,&second,false);check(&mut p,true,false);
            if n==3 {
                p.extend([Step::Theme(true),Step::Geometry(2.,true),
                    Step::Shot("backup-capstone-narrow-light-200-goal-start".into(),Target::GoalStart),
                    Step::Shot("backup-capstone-narrow-light-200-goal-end".into(),Target::GoalEnd),
                    Step::Theme(false),
                    Step::Shot("backup-capstone-narrow-dark-200-goal-start".into(),Target::GoalStart),
                    Step::Shot("backup-capstone-narrow-dark-200-goal-end".into(),Target::GoalEnd),
                    Step::Geometry(1.,false)]);
                // This task's independent recovery checker spans enough real ticks to Stop.
                p.extend([Step::Click(Target::Control("check")),Step::StopActive,Step::StopExact,
                    Step::Warm,Step::Click(Target::Back),Step::Click(Target::Card(false)),
                    Step::ResumeExact,Step::StoppedRetained]);
                check(&mut p,true,false);
            }
            p.push(Step::Warm);
            if n<3 {p.extend([Step::Next(n+1),Step::Click(Target::Back)]);}else{
                p.push(Step::Shot("backup-unit-complete".into(),Target::UnitComplete));
            }
        }
        p.push(Step::Finish);
        assert!(p.len()<1000,"Closed backup plan length {} exceeds reserved range",p.len());
        assert_eq!(p.iter().filter(|s|matches!(s,Step::Shot(..))).count(),81);
        p
    })
}
pub(in crate::ui) fn phase_bounds() -> (usize, usize) {
    (FIRST, FIRST + plan().len() - 1)
}
#[cfg(test)]
pub(in crate::ui) fn undo_stage() -> usize {
    let phases = plan()
        .iter()
        .enumerate()
        .filter(|(_, s)| matches!(s, Step::Undo))
        .map(|(n, _)| FIRST + n)
        .collect::<Vec<_>>();
    assert_eq!(phases.len(), 1);
    phases[0]
}
#[cfg(test)]
pub(in crate::ui) fn stop_stage() -> usize {
    let phases = plan()
        .iter()
        .enumerate()
        .filter(|(_, s)| matches!(s, Step::StopActive))
        .map(|(n, _)| FIRST + n)
        .collect::<Vec<_>>();
    assert_eq!(phases.len(), 1);
    phases[0]
}
pub(in crate::ui) fn immediate_stop(stage: usize) -> bool {
    (FIRST..END).contains(&stage)
        && plan().get(stage - FIRST).is_some_and(|s| {
            matches!(
                s,
                Step::StopActive | Step::StopExact | Step::Click(Target::Control("check" | "stop"))
            )
        })
}
impl LabApp {
    pub(in crate::ui) fn file_backup_lessons_stage(
        smoke: bool,
        name: Option<&str>,
    ) -> Option<usize> {
        (smoke && name == Some("file-backup-lessons")).then_some(FIRST)
    }
    pub(in crate::ui) fn file_backup_lessons_contains_stage(&self, n: usize) -> bool {
        (FIRST..END).contains(&n) && n <= phase_bounds().1
    }
    fn bk_put<T: Clone + Send + Sync + 'static>(&self, ctx: &egui::Context, name: &str, v: T) {
        ctx.data_mut(|d| d.insert_temp(key(name), v));
    }
    fn bk_get<T: Clone + Send + Sync + 'static>(&self, ctx: &egui::Context, name: &str) -> T {
        ctx.data(|d| d.get_temp(key(name)))
            .expect("Prior real backup observation")
    }
    fn bk_scope(&self) {
        self.nat_private();
        assert!(!self.synthetic);
        assert_eq!((self.scenarios.len(), self.units.len()), (168, 43));
        assert_eq!(
            self.scenarios[164..168]
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            IDS
        );
        assert!(
            self.progress.attempts.len() <= 4
                && self
                    .progress
                    .attempts
                    .keys()
                    .all(|s| IDS.contains(&s.as_str()))
        );
    }
    fn bk_target(&self, ctx: &egui::Context, t: &Target) -> (egui::Rect, egui::Rect, bool) {
        let root = self
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let m = &self.metrics.backup;
        if matches!(t, Target::UnitChoice) {
            let r = self
                .metrics
                .unit_choices
                .iter()
                .find(|(id, _)| id == "backup-and-recovery")
                .expect("Normal unit choice")
                .1;
            return (
                r,
                self.metrics
                    .unit_popup
                    .unwrap()
                    .intersect(ctx.content_rect()),
                true,
            );
        }
        if matches!(t, Target::GoalStart | Target::GoalEnd) {
            let (goal, body_row) = m
                .goal
                .expect("Actual current objective Label and body row height");
            assert!(
                body_row.is_finite() && body_row > 0. && goal.is_finite() && goal.height() > 0.
            );
            let height = body_row.min(goal.height());
            let top = if matches!(t, Target::GoalStart) {
                goal.top()
            } else {
                goal.bottom() - height
            };
            let edge = egui::Rect::from_min_size(
                egui::pos2(goal.left(), top),
                egui::vec2(goal.width(), height),
            );
            return (edge, root, false);
        }
        if matches!(t, Target::Input) {
            return (
                m.terminal.input.unwrap().0,
                m.terminal.input_clip.unwrap().intersect(root),
                false,
            );
        }
        let r = match t {
            Target::Control(k) => m.controls.iter().find(|r| r.0 == *k).map(|r| r.1),
            Target::Tab(v) => m.tabs.iter().find(|r| r.0 == *v).map(|r| r.1),
            Target::Insert(n) => m.inserts.iter().find(|r| r.0 == *n).map(|r| r.1),
            Target::Run => m.terminal.run,
            Target::Output => m.terminal.output,
            Target::Failure => m.rows.iter().find(|r| !r.1).map(|r| r.2),
            Target::Heading => m.heading,
            Target::Card(f) => {
                assert_eq!(self.metrics.card_actions.len(), 1);
                let r = self.metrics.card_actions[0];
                Some(if *f { r.1 } else { r.0 })
            }
            Target::Back => self.metrics.back,
            Target::Reset => self.metrics.reset,
            Target::Cancel => self.metrics.reset_cancel,
            Target::Confirm => self.metrics.confirm_reset,
            Target::Challenge => self.metrics.mode_challenge,
            Target::UnitPicker => self.metrics.unit_picker,
            Target::Next => self.metrics.next_unit,
            Target::UnitComplete => self.metrics.unit_complete,
            Target::Input | Target::UnitChoice | Target::GoalStart | Target::GoalEnd => {
                unreachable!()
            }
        }
        .unwrap_or_else(|| panic!("Actual backup target {t:?} is absent"));
        (r, root, false)
    }
    fn bk_reveal(&mut self, ctx: &egui::Context, t: &Target) -> bool {
        let (r, v, inner) = self.bk_target(ctx, t);
        let k = egui::Id::new(("backup-reveal", self.smoke_stage));
        let mut q = ctx.data(|d| d.get_temp::<Queue>(k)).unwrap_or_default();
        let done = self.nat_reveal(ctx, r, v, inner, &mut q);
        ctx.data_mut(|d| d.insert_temp(k, q));
        done
    }
    fn bk_click(&mut self, ctx: &egui::Context, t: &Target) -> bool {
        let k = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(k)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        if matches!(t, Target::UnitChoice) {
            let (r, v, _) = self.bk_target(ctx, t);
            return self.mac_popup_choice(ctx, Some(r), Some(v));
        }
        if !self.bk_reveal(ctx, t) {
            return false;
        }
        let (r, _, _) = self.bk_target(ctx, t);
        self.click_smoke_control(ctx, Some(r))
    }
    fn bk_clear_click(&self, ctx: &egui::Context) {
        let k = egui::Id::new(("native-click-phases", self.smoke_stage));
        ctx.data_mut(|d| d.remove::<u8>(k));
    }
    fn bk_text(&mut self, ctx: &egui::Context, text: &str) -> bool {
        let k = egui::Id::new(("backup-type", self.smoke_stage));
        let phase = ctx.data(|d| d.get_temp::<u8>(k)).unwrap_or(0);
        if phase == 0 {
            if !self.bk_click(ctx, &Target::Input) {
                return false;
            }
            let (_, id) = self.metrics.backup.terminal.input.unwrap();
            assert!(ctx.memory(|m| m.has_focus(id)));
            assert!(text.len() <= 4096 && !text.contains(['\n', '\r', '\0']));
            self.pending_events.extend([
                ctrl(egui::Key::A, true),
                ctrl(egui::Key::A, false),
                egui::Event::Paste(text.into()),
            ]);
            ctx.data_mut(|d| d.insert_temp(k, 1u8));
            self.bk_clear_click(ctx);
            return false;
        }
        let a = &self.progress.attempts[&self.scenarios[self.selected.unwrap()].id];
        let practice = !a.challenge && learning(a).view == View::Example;
        assert_eq!(
            command(a, practice),
            text,
            "Actual persisted command equals delivered text"
        );
        true
    }
    fn bk_command(&mut self, ctx: &egui::Context, text: &str, practice: bool) -> bool {
        let k = egui::Id::new(("backup-command", self.smoke_stage));
        let phase = ctx.data(|d| d.get_temp::<u8>(k)).unwrap_or(0);
        if phase == 0 {
            if ctx
                .data(|d| d.get_temp::<Value>(key("command-before")))
                .is_none()
            {
                self.bk_put(ctx, "command-before", portable(self));
                self.bk_put(ctx, "command-receipts", receipts(self));
            }
            if !self.bk_text(ctx, text) {
                return false;
            }
            ctx.data_mut(|d| d.insert_temp(k, 1u8));
            self.bk_clear_click(ctx);
            return false;
        }
        if !self.bk_click(ctx, &Target::Run) {
            return false;
        }
        let id = &self.scenarios[self.selected.unwrap()].id;
        let a = &self.progress.attempts[id];
        assert_eq!(command(a, practice), text);
        let log = if practice {
            &learning(a).transcript
        } else {
            &a.transcript
        };
        let mut expected: Value = self.bk_get(ctx, "command-before");
        let prior = if practice {
            &expected["attempts"][id]["workspace"]["file_backup_learning"]["transcript"]
        } else {
            &expected["attempts"][id]["transcript"]
        }
        .as_array()
        .unwrap();
        let (entries, bytes) = if practice { (16, 65536) } else { (120, 262144) };
        assert!(
            prior.len() < entries,
            "Closed route stays below actual retention count"
        );
        assert_eq!(
            log.len(),
            prior.len() + 1,
            "Run requires exactly one fresh receipt, including repeated identical commands"
        );
        for (before, after) in prior.iter().zip(log) {
            assert_eq!(
                before.as_str().unwrap(),
                after,
                "Run preserves complete prior transcript prefix"
            );
        }
        let receipt = log.last().unwrap();
        assert!(
            receipt.starts_with(&format!("simulated root $ {text}\n")),
            "The sole new receipt must identify THIS Run, never an older matching command"
        );
        assert!(
            prior
                .iter()
                .map(|v| v.as_str().unwrap().len())
                .sum::<usize>()
                + receipt.len()
                <= bytes,
            "Closed route stays below byte-retention cap so whole prefix remains exact"
        );
        if practice {
            expected["attempts"][id]["workspace"]["file_backup_learning"] =
                serde_json::to_value(learning(a)).unwrap();
        } else {
            for field in ["state", "transcript"] {
                expected["attempts"][id][field] = serde_json::to_value(a).unwrap()[field].clone()
            }
            expected["attempts"][id]["workspace"]["command"] = text.into();
            expected["attempts"][id]["workspace"]["checked"] = false.into();
        }
        exact(
            &self.progress,
            &expected,
            "Actual Run preserves all other work, progress and example/assignment scope",
        );
        exact(
            &self.progress.migration_receipts,
            &self.bk_get::<Value>(ctx, "command-receipts"),
            "Run retains receipts",
        );
        ctx.data_mut(|d| d.remove::<Value>(key("command-before")));
        true
    }
    fn bk_type_again(&self, ctx: &egui::Context) {
        let id = egui::Id::new(("backup-type", self.smoke_stage));
        ctx.data_mut(|d| d.remove::<u8>(id));
        self.bk_clear_click(ctx);
    }
    fn bk_history_key(&mut self, redo: bool) {
        for pressed in [true, false] {
            let mut event = ctrl(egui::Key::Z, pressed);
            if let egui::Event::Key { modifiers, .. } = &mut event {
                modifiers.shift = redo;
            }
            self.pending_events.push(event);
        }
    }
    /// Capture-only finite cadence. Incidental fast renders are not productive
    /// Undo polls; the existing300-poll ceiling and actual native history remain.
    fn bk_undo_poll(&self, ctx: &egui::Context) -> bool {
        let now = ctx.input(|i| i.time);
        assert!(now.is_finite());
        let k = key("undo-poll-cadence");
        let (start, next) = ctx
            .data(|d| d.get_temp::<(f64, f64)>(k))
            .unwrap_or((now, now));
        assert!(
            now >= start && now - start <= 15.,
            "Capture-only Undo exceeded finite15s time allowance; current={now},start={start},next={next}"
        );
        if now < next {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64((next - now).min(0.05)));
            return false;
        }
        ctx.data_mut(|d| d.insert_temp(k, (start, now + 0.05)));
        true
    }
    fn bk_undo(&mut self, ctx: &egui::Context, id: &str) -> bool {
        let k = key("undo-phase");
        let phase = ctx.data(|d| d.get_temp::<u8>(k)).unwrap_or(0);
        let now = ctx.input(|i| i.time);
        let (_, field) = self.metrics.backup.terminal.input.unwrap();
        if phase == 0 {
            if ctx
                .data(|d| d.get_temp::<Value>(key("undo-original")))
                .is_none()
            {
                self.bk_put(ctx, "undo-original", portable(self));
                self.bk_put(ctx, "undo-receipts", receipts(self));
                self.bk_put(
                    ctx,
                    "undo-original-command",
                    command(&self.progress.attempts[id], false).to_owned(),
                );
            }
            if !self.bk_text(ctx, "pwd") {
                return false;
            }
            self.bk_type_again(ctx);
            self.bk_put(ctx, "undo-stable-start", now);
        } else if phase == 1 || phase == 3 {
            // Natural egui time creates a stable Undo checkpoint, without direct history injection.
            let start: f64 = self.bk_get(ctx, "undo-stable-start");
            if now - start < 1.2 {
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
                return false;
            }
            assert!(ctx.memory(|m| m.has_focus(field)));
            let text = if phase == 1 { "pwd" } else { "ls" };
            assert_eq!(command(&self.progress.attempts[id], false), text);
            if phase == 1 {
                self.bk_put(ctx, "undo-before", portable(self));
            } else {
                self.bk_put(ctx, "undo-after", portable(self));
                self.bk_put(ctx, "undo-retries", 1u8);
                self.bk_history_key(false);
            }
        } else if phase == 2 {
            if !self.bk_text(ctx, "ls") {
                return false;
            }
            self.bk_type_again(ctx);
            self.bk_put(ctx, "undo-stable-start", now);
        } else if (4..=6).contains(&phase) {
            assert!(
                ctx.memory(|m| m.has_focus(field)),
                "Real Undo/Redo keeps actual command focus"
            );
            let (wanted, redo) = if phase == 5 {
                ("ls", true)
            } else {
                ("pwd", false)
            };
            if command(&self.progress.attempts[id], false) != wanted {
                let n: u8 = self.bk_get(ctx, "undo-retries");
                assert!(
                    n < 3,
                    "Actual Undo/Redo must restore text within three native cursor/text checkpoints"
                );
                self.bk_put(ctx, "undo-retries", n + 1);
                self.bk_history_key(redo);
                ctx.request_repaint();
                return false;
            }
            let expected: Value = self.bk_get(
                ctx,
                if phase == 5 {
                    "undo-after"
                } else {
                    "undo-before"
                },
            );
            exact(
                &self.progress,
                &expected,
                "Actual Undo/Redo restores complete saved draft and all other work without installed state or rerun",
            );
            exact(
                &self.progress.migration_receipts,
                &self.bk_get::<Value>(ctx, "undo-receipts"),
                "Undo/Redo retains full receipts",
            );
            let cursor = egui::text_edit::TextEditState::load(ctx, field)
                .unwrap()
                .cursor
                .char_range()
                .unwrap();
            assert!(cursor.is_empty());
            assert_eq!(cursor.primary.index, wanted.chars().count());
            if phase < 6 {
                self.bk_put(ctx, "undo-retries", 1u8);
                self.bk_history_key(phase == 4);
            }
        } else {
            let text: String = self.bk_get(ctx, "undo-original-command");
            if !self.bk_text(ctx, &text) {
                return false;
            }
            let mut expected: Value = self.bk_get(ctx, "undo-original");
            expected["attempts"][id]["workspace"]["checked"] = false.into();
            exact(
                &self.progress,
                &expected,
                "Restoring earlier text retains all work; genuine editing clears only checked marker",
            );
            exact(
                &self.progress.migration_receipts,
                &self.bk_get::<Value>(ctx, "undo-receipts"),
                "Restored command retains receipts",
            );
            self.bk_put(ctx, "undo-finished", true);
            return true;
        }
        ctx.data_mut(|d| d.insert_temp(k, phase + 1));
        ctx.request_repaint();
        false
    }
    fn bk_wait(&self, ctx: &egui::Context, scale: f32, narrow: bool) -> bool {
        let sample = (
            ctx.content_rect(),
            ctx.pixels_per_point(),
            ctx.zoom_factor(),
            ctx.input(|i| i.viewport().fullscreen),
        );
        let px = sample.0.size() * sample.1;
        let expected = (sample.2 - scale).abs() < 0.01
            && sample.3 == Some(!narrow)
            && if narrow {
                (650. ..=1300.).contains(&px.x)
            } else {
                px.x >= 1500.
            };
        let k = egui::Id::new(("backup-geometry", self.smoke_stage));
        let ready = ctx.data_mut(|d| {
            let (old, n) = d
                .get_temp::<(Option<(egui::Rect, f32, f32, Option<bool>)>, u8)>(k)
                .unwrap_or((None, 0));
            if expected && old == Some(sample) {
                return true;
            }
            assert!(
                n < 60,
                "Backup measured geometry did not settle: {sample:?}"
            );
            d.insert_temp(k, (Some(sample), n + 1));
            false
        });
        if !ready {
            ctx.request_repaint()
        }
        ready
    }
    fn bk_warm(&mut self, ctx: &egui::Context) {
        let portable = portable(self);
        let caller_receipts = receipts(self);
        self.save();
        assert!(!self.dirty && self.storage_error.is_none());
        let raw = progress::read_bounded(&self.path, 2 * 1024 * 1024).unwrap();
        let mut projection: Value = serde_json::from_slice(&raw).unwrap();
        let raw_receipts = projection
            .as_object_mut()
            .unwrap()
            .remove("migration_receipts")
            .unwrap();
        exact(&projection, &portable, "Warm disk complete portable fields");
        let loaded = progress::load(&self.path, &self.scenarios).unwrap();
        exact(
            &loaded,
            &portable,
            "Normal loader complete readback without installing Progress",
        );
        exact(
            &loaded.migration_receipts,
            &raw_receipts,
            "Readback receipt bytes",
        );
        exact(
            &self.progress.migration_receipts,
            &caller_receipts,
            "Save caller receipts unchanged",
        );
        assert_eq!(
            progress::read_bounded(&self.path, 2 * 1024 * 1024).unwrap(),
            raw
        );
        self.bk_put(
            ctx,
            "warm",
            Warm {
                raw,
                portable,
                raw_receipts,
                caller_receipts,
            },
        );
    }
    fn bk_choose_unit(&mut self, ctx: &egui::Context) -> bool {
        let k = key("unit-choice-phase");
        let phase = ctx.data(|d| d.get_temp::<u8>(k)).unwrap_or(0);
        if phase < 2 {
            let target = if phase == 0 {
                Target::UnitPicker
            } else {
                Target::UnitChoice
            };
            if !self.bk_click(ctx, &target) {
                return false;
            }
            ctx.data_mut(|d| d.insert_temp(k, phase + 1));
            ctx.request_repaint();
            return false;
        }
        assert_eq!(self.filter.unit, "backup-and-recovery");
        assert_eq!(
            self.metrics.card_actions.len(),
            4,
            "Actual course selection exposes all four normal cards"
        );
        assert!(
            self.progress.attempts.is_empty(),
            "Choosing a unit creates no work or credit"
        );
        true
    }
    pub(in crate::ui) fn file_backup_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        self.bk_scope();
        let p = plan();
        let offset = stage - FIRST;
        let step = &p[offset];
        let n = p[..=offset]
            .iter()
            .rev()
            .find_map(|s| {
                if let Step::Select(n) = s {
                    Some(*n)
                } else {
                    None
                }
            })
            .unwrap();
        let id = IDS[n];
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        let s = self.scenarios[i].clone();
        if matches!(step, Step::Undo) && !self.bk_undo_poll(ctx) {
            return (false, None);
        }
        let k = egui::Id::new(("backup-step-ticks", stage));
        let ticks = ctx.data_mut(|d| {
            let n = d.get_temp::<u16>(k).unwrap_or(0) + 1;
            d.insert_temp(k, n);
            n
        });
        let undo_observation = if matches!(step, Step::Undo) {
            let phase = ctx
                .data(|d| d.get_temp::<u8>(key("undo-phase")))
                .unwrap_or(0);
            let start = ctx.data(|d| d.get_temp::<f64>(key("undo-stable-start")));
            let now = ctx.input(|i| i.time);
            Some((phase, now, start.map(|t| now - t)))
        } else {
            None
        };
        assert!(
            ticks <= 300,
            "Backup stage{stage}/{step:?} exceeded bounded real-event wait; actualUndo(phase,time,stable_elapsed)={undo_observation:?}"
        );
        match step {
            Step::Select(n) => {
                assert!(self.selected.is_none());
                assert_eq!(
                    self.progress.attempts.len(),
                    if *n == 0 { 0 } else { *n + 1 }
                );
                if *n == 0 && !self.bk_choose_unit(ctx) {
                    return (false, None);
                }
                assert_eq!(self.filter.unit, "backup-and-recovery");
                self.filter.search = s.title.clone();
                self.page = 0;
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.bk_put(ctx, "narrow", false);
            }
            Step::Next(next) => {
                if ctx
                    .data(|d| d.get_temp::<Value>(key("next-prior")))
                    .is_none()
                {
                    assert_eq!(self.selected, Some(i));
                    assert!(self.metrics.completion);
                    assert!(!self.progress.attempts.contains_key(IDS[*next]));
                    self.bk_put(ctx, "next-prior", portable(self));
                    self.bk_put(ctx, "next-receipts", receipts(self));
                }
                if !self.bk_click(ctx, &Target::Next) {
                    return (false, None);
                }
                let mut expected: Value = self.bk_get(ctx, "next-prior");
                let clock = expected["clock"].as_u64().unwrap() + 1;
                let target = self.scenarios.iter().find(|s| s.id == IDS[*next]).unwrap();
                let mut fresh = Attempt::new(target, clock);
                model::prepare(&mut fresh);
                teach::prepare(
                    &mut fresh,
                    target.file_backup_teaching.as_ref().unwrap(),
                    true,
                )
                .unwrap();
                expected["clock"] = clock.into();
                expected["attempts"][IDS[*next]] = serde_json::to_value(fresh).unwrap();
                exact(
                    &self.progress,
                    &expected,
                    "Actual Next creates exactly the fresh normal next task and recency",
                );
                exact(
                    &self.progress.migration_receipts,
                    &self.bk_get::<Value>(ctx, "next-receipts"),
                    "Next preserves receipts",
                );
                assert_eq!(
                    self.selected,
                    self.scenarios.iter().position(|s| s.id == IDS[*next])
                );
                ctx.data_mut(|d| d.remove::<Value>(key("next-prior")));
            }
            Step::Click(t) => {
                if !self.bk_click(ctx, t) {
                    return (false, None);
                }
            }
            Step::Open => {
                let a = &self.progress.attempts[id];
                assert_eq!(self.selected, Some(i));
                assert_eq!(learning(a).view, View::Introduction);
                assert_eq!(a.state, s.initial);
                assert!(!self.metrics.completion);
            }
            Step::BeforePractice => {
                assert_eq!(learning(&self.progress.attempts[id]).view, View::Example);
                self.bk_put(ctx, "practice-before", portable(self));
                self.bk_put(ctx, "practice-receipts", receipts(self));
            }
            Step::BeforeInsert => {
                self.bk_put(ctx, "insert-before", portable(self));
                self.bk_put(ctx, "insert-receipts", receipts(self));
            }
            Step::InsertOnly(n) => {
                let mut expected: Value = self.bk_get(ctx, "insert-before");
                let text = &s.file_backup_teaching.as_ref().unwrap().example_commands[*n].command;
                expected["attempts"][id]["workspace"]["file_backup_learning"]["command"] =
                    text.clone().into();
                // Insert changes only the draft, not example files, log, assignment or credit.
                expected["attempts"][id]["workspace"]["file_backup_learning"]["checked"] =
                    false.into();
                exact(
                    &self.progress,
                    &expected,
                    "Actual Insert is inert example draft population",
                );
                exact(
                    &self.progress.migration_receipts,
                    &self.bk_get::<Value>(ctx, "insert-receipts"),
                    "Insert retains receipts",
                );
                assert!(!self.metrics.completion);
            }
            Step::Command(text, practice) => {
                if !self.bk_command(ctx, text, *practice) {
                    return (false, None);
                }
            }
            Step::PracticeExact => {
                let mut expected: Value = self.bk_get(ctx, "practice-before");
                expected["attempts"][id]["workspace"]["file_backup_learning"] =
                    serde_json::to_value(learning(&self.progress.attempts[id])).unwrap();
                exact(
                    &self.progress,
                    &expected,
                    "All real example commands preserve complete assignment and other work",
                );
                exact(
                    &self.progress.migration_receipts,
                    &self.bk_get::<Value>(ctx, "practice-receipts"),
                    "Practice retains receipts",
                );
                assert!(!self.metrics.completion);
            }
            Step::Check(pass, practice) => {
                if self.backup_presentation.checking() {
                    ctx.request_repaint();
                    return (false, None);
                }
                let a = &self.progress.attempts[id];
                let checked = if *practice {
                    learning(a).checked
                } else {
                    a.workspace.as_ref().unwrap().checked
                };
                assert!(
                    checked,
                    "The actual Check control must persist its check request"
                );
                let rows = &self.metrics.backup.rows;
                assert!(
                    !rows.is_empty(),
                    "Actual current rendered objective results"
                );
                assert_eq!(
                    rows.iter().all(|r| r.1),
                    *pass,
                    "{id} practice={practice}: {rows:?}"
                );
                assert_eq!(self.metrics.backup.complete, !*practice && *pass);
                assert_eq!(self.metrics.completion, !*practice && *pass);
                if !practice {
                    assert_eq!(progress::current_solved(a, &s), *pass)
                }
            }
            Step::Rows(expected) => {
                if self.backup_presentation.checking() {
                    ctx.request_repaint();
                    return (false, None);
                }
                assert_eq!(
                    self.metrics
                        .backup
                        .rows
                        .iter()
                        .map(|r| r.1)
                        .collect::<Vec<_>>(),
                    *expected,
                    "Independent complete row vector {id}"
                );
            }
            Step::Snapshot => {
                self.bk_put(ctx, "snapshot", portable(self));
                self.bk_put(ctx, "snapshot-receipts", receipts(self));
            }
            Step::Retained | Step::StopExact => {
                exact(
                    &self.progress,
                    &self.bk_get::<Value>(ctx, "snapshot"),
                    "Cancel/Stop retains all progress and both drafts",
                );
                exact(
                    &self.progress.migration_receipts,
                    &self.bk_get::<Value>(ctx, "snapshot-receipts"),
                    "Cancel/Stop retains receipts",
                );
                if matches!(step, Step::StopExact) {
                    assert!(!self.backup_presentation.checking());
                    assert!(
                        self.backup_presentation.explicitly_stopped(id),
                        "Only an actual Stop operation satisfies cancellation, not natural completion"
                    );
                }
            }
            Step::StopActive => {
                assert!(
                    self.backup_presentation.checking(),
                    "Stop probe must observe a genuinely active bounded checker"
                );
                let control = self
                    .metrics
                    .backup
                    .controls
                    .iter()
                    .find(|r| r.0 == "stop")
                    .map(|r| r.1);
                let clip = self
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                println!(
                    "Backup real Stop observation stage{stage}: checking={},control={control:?},clip={clip:?},scroll={:?}",
                    self.backup_presentation.checking(),
                    self.metrics.main_scroll_offset
                );
                self.bk_put(ctx, "snapshot", portable(self));
                self.bk_put(ctx, "snapshot-receipts", receipts(self));
                assert!(!self.backup_presentation.explicitly_stopped(id));
                let r = control.expect("Actual visible Stop control while checker is active");
                assert!(
                    smoke_control_visible(r, clip, ctx.pixels_per_point()),
                    "Prepare the real Check viewport before starting; never scroll a short job into completion"
                );
                self.pending_events
                    .push(egui::Event::PointerMoved(r.center()));
                for pressed in [true, false] {
                    self.pending_events.push(egui::Event::PointerButton {
                        pos: r.center(),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    });
                }
            }
            Step::StoppedRetained => {
                assert!(!self.backup_presentation.checking());
                assert!(
                    self.backup_presentation.explicitly_stopped(id),
                    "Back/Resume retains actual Stop intent until explicit Check"
                );
                assert!(
                    !self.metrics.completion,
                    "Stopped checker never publishes a new completion"
                );
            }
            Step::ResetExact => {
                let mut expected: Value = self.bk_get(ctx, "snapshot");
                let a = &self.progress.attempts[id];
                assert!(!self.reset_pending && self.selected == Some(i));
                assert_eq!(a.state, s.initial);
                assert!(!a.challenge && a.step == 0 && a.hints == 0 && a.transcript.is_empty());
                assert_eq!(
                    a.visited,
                    expected["attempts"][id]["visited"].as_u64().unwrap()
                );
                let mut fresh = Attempt::new(&s, a.visited);
                model::prepare(&mut fresh);
                teach::prepare(&mut fresh, s.file_backup_teaching.as_ref().unwrap(), true).unwrap();
                exact(
                    a,
                    &fresh,
                    "Reset creates the complete fresh selected attempt without changing recency",
                );
                expected["attempts"][id] = serde_json::to_value(fresh).unwrap();
                exact(
                    &self.progress,
                    &expected,
                    "Reset changes only the selected attempt, retaining all other work and clock",
                );
                exact(
                    &self.progress.migration_receipts,
                    &self.bk_get::<Value>(ctx, "snapshot-receipts"),
                    "Reset retains full receipts",
                );
            }
            Step::Warm => self.bk_warm(ctx),
            Step::ResumeExact => {
                let warm: Warm = self.bk_get(ctx, "warm");
                let mut expected = warm.portable;
                let clock = expected["clock"].as_u64().unwrap() + 1;
                expected["clock"] = clock.into();
                expected["attempts"][id]["visited"] = clock.into();
                assert_eq!(self.selected, Some(i));
                exact(
                    &self.progress,
                    &expected,
                    "Actual Back/Resume retains drafts, files, histories and other progress; only recency advances",
                );
                exact(
                    &self.progress.migration_receipts,
                    &warm.caller_receipts,
                    "Resume preserves receipts",
                );
                let raw: Value = serde_json::from_slice(&warm.raw).unwrap();
                exact(
                    &raw["migration_receipts"],
                    &warm.raw_receipts,
                    "Retained raw receipts exact",
                );
            }
            Step::Undo => {
                if !self.bk_undo(ctx, id) {
                    return (false, None);
                }
            }
            Step::Keyboard => {
                let k = key("keyboard-phase");
                let phase = ctx.data(|d| d.get_temp::<u8>(k)).unwrap_or(0);
                if phase == 0 {
                    if !self.bk_click(ctx, &Target::Input) {
                        return (false, None);
                    }
                    self.bk_put(ctx, "keyboard-before", portable(self));
                    self.bk_put(ctx, "keyboard-receipts", receipts(self));
                }
                let (_, field) = self.metrics.backup.terminal.input.unwrap();
                match phase {
                    0 => {
                        assert!(ctx.memory(|m| m.has_focus(field)));
                        for pressed in [true, false] {
                            self.pending_events.push(egui::Event::Key {
                                key: egui::Key::Tab,
                                physical_key: Some(egui::Key::Tab),
                                pressed,
                                repeat: false,
                                modifiers: Default::default(),
                            });
                        }
                    }
                    1 | 3 => {
                        // Focus traversal is resolved at end-pass. Observe one
                        // ordinary idle frame before sending the reverse key.
                    }
                    2 => {
                        assert!(
                            ctx.memory(|m| m.focused()).is_some_and(|id| id != field),
                            "Tab must move actual native focus"
                        );
                        for pressed in [true, false] {
                            self.pending_events.push(egui::Event::Key {
                                key: egui::Key::Tab,
                                physical_key: Some(egui::Key::Tab),
                                pressed,
                                repeat: false,
                                modifiers: egui::Modifiers {
                                    shift: true,
                                    ..Default::default()
                                },
                            });
                        }
                    }
                    4 => {
                        assert!(
                            ctx.memory(|m| m.has_focus(field)),
                            "Shift-Tab returns to the actual command field"
                        );
                        for pressed in [true, false] {
                            self.pending_events.push(egui::Event::Key {
                                key: egui::Key::End,
                                physical_key: Some(egui::Key::End),
                                pressed,
                                repeat: false,
                                modifiers: Default::default(),
                            });
                        }
                    }
                    _ => {
                        let edit = egui::text_edit::TextEditState::load(ctx, field).unwrap();
                        let range = edit.cursor.char_range().unwrap();
                        assert!(range.is_empty());
                        assert_eq!(
                            range.primary.index,
                            command(&self.progress.attempts[id], false).chars().count()
                        );
                        exact(
                            &self.progress,
                            &self.bk_get::<Value>(ctx, "keyboard-before"),
                            "Keyboard focus/cursor navigation preserves all saved work",
                        );
                        exact(
                            &self.progress.migration_receipts,
                            &self.bk_get::<Value>(ctx, "keyboard-receipts"),
                            "Keyboard navigation preserves receipts",
                        );
                        return (true, None);
                    }
                }
                ctx.data_mut(|d| d.insert_temp(k, phase + 1));
                ctx.request_repaint();
                return (false, None);
            }
            Step::Text(text) => {
                if !self.bk_text(ctx, text) {
                    return (false, None);
                }
            }
            Step::Theme(light) => {
                self.theme_id = if *light { "dlms-light" } else { "dlms-dark" }.into();
                theme::apply(ctx, &self.theme_id);
            }
            Step::Geometry(scale, narrow) => {
                self.bk_put(ctx, "narrow", *narrow);
                self.scale = *scale;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!narrow));
                if *narrow {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(946., 929.)))
                }
                if !self.bk_wait(ctx, *scale, *narrow) {
                    return (false, None);
                }
            }
            Step::Shot(name, target) => {
                let narrow = ctx
                    .data(|d| d.get_temp::<bool>(key("narrow")))
                    .unwrap_or(false);
                if !self.bk_wait(ctx, self.scale, narrow) || !self.bk_reveal(ctx, target) {
                    return (false, None);
                }
                let (r, v, _) = self.bk_target(ctx, target);
                assert!(smoke_control_visible(r, v, ctx.pixels_per_point()));
                if matches!(target, Target::Input) {
                    let (_, id) = self.metrics.backup.terminal.input.unwrap();
                    assert!(ctx.memory(|m| m.has_focus(id)));
                }
                if matches!(target, Target::GoalStart | Target::GoalEnd) {
                    let (goal, body_row) = self.metrics.backup.goal.unwrap();
                    println!(
                        "Backup capstone current goal label={goal:?}; body row={body_row}; fully visible measured edge={r:?}; clip={v:?}. This proves reachability of this edge, not all vertical text at once."
                    );
                }
                let content = ctx.content_rect();
                let ppp = ctx.pixels_per_point();
                let zoom = ctx.zoom_factor();
                println!(
                    "Backup {id}/{name} actual_content={content:?} PPP={ppp} zoom={zoom}; requested dimensions are not observations"
                );
                return (true, Some(format!("{name}-zoom{zoom:.2}-ppp{ppp:.2}")));
            }
            Step::Finish => {
                if ctx.data(|d| d.get_temp::<bool>(key("finished"))) == Some(true) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return (false, None);
                }
                assert!(
                    self.bk_get::<bool>(ctx, "undo-finished"),
                    "Actual bounded Undo/Redo restoration must complete before Finish"
                );
                for id in IDS {
                    let s = self.scenarios.iter().find(|s| s.id == id).unwrap();
                    assert!(progress::current_solved(&self.progress.attempts[id], s));
                }
                self.bk_warm(ctx);
                println!(
                    "PASS native four normal backup lessons: unit/cards/Next/unit-complete; introductions/separate practice/inert Insert/adverse/repair; real terminal reference/two alternatives/wrong/partial/success invalidation; active checker Stop/reset/cancel/resume/full progress and receipts; five sizes/light/dark/wide/narrow. Render review, novice and accessibility acceptance are separate."
                );
                ctx.data_mut(|d| d.insert_temp(key("finished"), true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return (false, None);
            }
        }
        (true, None)
    }
}
