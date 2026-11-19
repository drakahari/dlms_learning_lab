//! Fixed normal-process restoration; all mutations use actual native RPM controls.
use super::*;
use crate::ui::smoke::rpm_patch::Op;
use dlms_learning_lab::{rpm_packages_terminal as terminal, rpm_packages_workspace as work};
const ID: &str = "lpic2-vendor-patch-set";
const ASSIGNMENT: &str = "rpm -U '/media/dispatch/unfinished assignment";
const EXAMPLE: &str = "rpm -q 'separate unfinished example";
pub(super) const CAPTURES: [&str; 6] = [
    "reopen-rpm-patch-writer-example",
    "reopen-rpm-patch-writer-assignment",
    "reopen-rpm-patch-reader-assignment",
    "reopen-rpm-patch-repaired-assignment",
    "reopen-rpm-patch-reader-example",
    "reopen-rpm-patch-repaired-example",
];
fn definition(source: &pack::Scenario) -> &work::Definition {
    let engine::Goal::RpmPackages { definition } = &source.goal else {
        panic!()
    };
    definition
}
fn expected(source: &pack::Scenario, practice: bool) -> dlms_learning_lab::rpm_packages::State {
    let lesson = source.rpm_packages_teaching.as_ref().unwrap();
    let d = if practice {
        &lesson.definition
    } else {
        definition(source)
    };
    let mut state = d.initial.clone();
    let commands = if practice {
        lesson
            .example_commands
            .iter()
            .map(|s| s.command.as_str())
            .chain(std::iter::once("rpm -e gallery-notes"))
            .collect::<Vec<_>>()
    } else {
        vec!["rpm -U /media/dispatch/dispatch-codec-3.0-2.train8.noarch.rpm"]
    };
    for command in commands {
        assert!(matches!(
            terminal::execute(d, &mut state, command),
            terminal::Outcome::Utility { status: 0, .. }
        ));
    }
    state
}
fn verify_partial(app: &LabApp) {
    let source = app.scenarios.iter().find(|s| s.id == ID).unwrap();
    let attempt = &app.progress.attempts[ID];
    let engine::State::RpmPackages { simulation } = &attempt.state else {
        panic!()
    };
    assert_eq!(
        simulation,
        &expected(source, false),
        "Actual partial installed set restored before repair"
    );
    let workspace = attempt.workspace.as_ref().unwrap();
    let practice = workspace.rpm_learning.as_ref().unwrap();
    let lesson = source.rpm_packages_teaching.as_ref().unwrap();
    assert_eq!(practice.context, lesson.definition);
    assert_eq!(practice.simulation, expected(source, true));
    assert_eq!(workspace.rpm_session.as_ref().unwrap().command, ASSIGNMENT);
    assert_eq!(practice.terminal.command, EXAMPLE);
    assert!(
        workspace.checked && practice.checked,
        "Pending commands do not change the checked package state"
    );
    assert!(!progress::current_solved(attempt, source));
    assert!(!definition(source).checks(simulation)[0].0);
    assert!(definition(source).checks(simulation)[1].0);
    assert!(
        practice
            .simulation
            .runtime()
            .files
            .contains_key("/srv/gallery/agent.conf.rpmnew")
    );
    assert!(
        !simulation
            .runtime()
            .files
            .contains_key("/srv/dispatch/agent.conf.rpmnew")
    );
}
fn plan(read: bool, source: &pack::Scenario) -> &'static Vec<Op> {
    static WRITE: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    static READ: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    (if read { &READ } else { &WRITE }).get_or_init(|| {
        let mut ops = vec![];
        if !read {
            ops.push(Op::ColdAppearance);
        }
        ops.push(Op::Open(ID));
        if read {
            ops.extend([
                Op::Tab(teaching::View::Assignment),
                Op::FocusCommand,
                Op::Shot(CAPTURES[2].into(), false),
                Op::Command("rpm -U /media/dispatch/dispatch-agent-1.0-4.train8.noarch.rpm".into()),
                Op::Check(false, true),
                Op::Shot(CAPTURES[3].into(), true),
                Op::Tab(teaching::View::Example),
                Op::FocusCommand,
                Op::Shot(CAPTURES[4].into(), false),
                Op::Command("rpm -i /media/gallery/gallery-notes-1.0-1.train8.noarch.rpm".into()),
                Op::Check(true, true),
                Op::Shot(CAPTURES[5].into(), true),
            ]);
        } else {
            ops.push(Op::Tab(teaching::View::Example));
            for step in &source
                .rpm_packages_teaching
                .as_ref()
                .unwrap()
                .example_commands
            {
                ops.push(Op::Command(step.command.clone()));
            }
            ops.extend([
                Op::Check(true, true),
                Op::Command("rpm -e gallery-notes".into()),
                Op::Check(true, false),
                Op::Draft(EXAMPLE.into()),
                Op::Shot(CAPTURES[0].into(), false),
                Op::Tab(teaching::View::Assignment),
                Op::Command("rpm -U /media/dispatch/dispatch-codec-3.0-2.train8.noarch.rpm".into()),
                Op::Check(false, false),
                Op::Draft(ASSIGNMENT.into()),
                Op::Shot(CAPTURES[1].into(), false),
            ]);
        }
        ops.push(Op::ColdFinish);
        ops
    })
}
impl LabApp {
    pub(super) fn reopen_rpm_patch_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::RpmPatch);
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        session.verify_profile_paths(&self.path).unwrap();
        if session.role == Role::Read {
            let key = egui::Id::new("rpm-patch-normal-load-observed");
            if !ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false) {
                let expected: Observed = serde_json::from_slice(
                    &bounded_file(&session.root.join("writer-observed.json"), JSON_CAP).unwrap(),
                )
                .unwrap();
                let (loaded, raw, prefs) = session.observed(self);
                assert_eq!(loaded, expected);
                assert_eq!(
                    raw,
                    bounded_file(&session.root.join("writer-progress.raw"), 16_000_000).unwrap()
                );
                assert_eq!(
                    prefs,
                    bounded_file(&session.root.join("writer-preferences.raw"), 4096).unwrap()
                );
                verify_partial(self);
                assert_eq!(self.scale, SCALE);
                assert_eq!(self.theme_id, THEME);
                session.write("reader-loaded.json", &loaded).unwrap();
                ctx.data_mut(|d| d.insert_temp(key, true));
            }
        }
        let source = self.scenarios.iter().find(|s| s.id == ID).unwrap().clone();
        self.rpm_patch_flow(ctx, Some(plan(session.role == Role::Read, &source)));
    }
    pub(in crate::ui) fn finish_rpm_patch_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::RpmPatch);
        if session.role == Role::Write {
            verify_partial(self);
        } else {
            let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
            let attempt = &self.progress.attempts[ID];
            assert!(progress::current_solved(attempt, source));
            let practice = attempt
                .workspace
                .as_ref()
                .unwrap()
                .rpm_learning
                .as_ref()
                .unwrap();
            assert!(
                practice.checked
                    && practice
                        .checks(source.rpm_packages_teaching.as_ref().unwrap())
                        .iter()
                        .all(|r| r.0)
            );
        }
        let key = egui::Id::new("rpm-patch-cold-finished");
        if !ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false) {
            self.save();
            assert!(!self.dirty && self.storage_error.is_none());
            let (observed, raw, prefs) = session.observed(self);
            let names = if session.role == Role::Write {
                (
                    "writer-progress.raw",
                    "writer-preferences.raw",
                    "writer-observed.json",
                )
            } else {
                (
                    "reader-progress.raw",
                    "reader-preferences.raw",
                    "reader-repaired.json",
                )
            };
            session.write_raw(names.0, &raw, 16_000_000).unwrap();
            session.write_raw(names.1, &prefs, 4096).unwrap();
            session.write(names.2, &observed).unwrap();
            ctx.data_mut(|d| d.insert_temp(key, true));
            println!(
                "PASS native reopen {}: actual patch set/site files, source policies and distinct unfinished terminal drafts restored before native repair",
                if session.role == Role::Write {
                    "writer"
                } else {
                    "reader"
                }
            );
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}
#[test]
fn patch_cold_plan_preserves_partial_package_work_and_distinct_drafts() {
    let pack: pack::Pack = serde_json::from_str(include_str!(
        "../../../../scenarios/advisory-patch-lessons-v1.json"
    ))
    .unwrap();
    let source = pack.scenarios.iter().find(|s| s.id == ID).unwrap();
    let assignment = expected(source, false);
    let example = expected(source, true);
    assert_eq!(
        definition(source)
            .checks(&assignment)
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>(),
        [false, true, true, true, true, true, true]
    );
    assert_eq!(
        source
            .rpm_packages_teaching
            .as_ref()
            .unwrap()
            .definition
            .checks(&example)
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>(),
        [true, true, true, false, true, true, true]
    );
    assert_ne!(ASSIGNMENT, EXAMPLE);
    assert!(
        plan(false, source)
            .iter()
            .any(|o| matches!(o,Op::Draft(s)if s==ASSIGNMENT))
    );
    assert!(
        plan(false, source)
            .iter()
            .any(|o| matches!(o,Op::Draft(s)if s==EXAMPLE))
    );
    assert!(
        plan(true, source)
            .iter()
            .any(|o| matches!(o, Op::Check(false, true)))
    );
    assert!(
        plan(true, source)
            .iter()
            .any(|o| matches!(o, Op::Check(true, true)))
    );
}
