//! Actual cold restoration of partial build/staging work in normal isolated profiles.
use super::*;
use crate::ui::smoke::kernel_build::{Op, expand};
use dlms_learning_lab::{kernel_source as binding, kernel_workspace as work};
const ID: &str = "lpic2-kernel-staged-handover";
pub(super) const CAPTURES: [&str; 8] = [
    "reopen-kernel-writer-example-config",
    "reopen-kernel-writer-assignment-config",
    "reopen-kernel-reader-assignment-config",
    "reopen-kernel-reader-assignment-feedback",
    "reopen-kernel-reader-example-config",
    "reopen-kernel-reader-example-feedback",
    "reopen-kernel-repaired-assignment",
    "reopen-kernel-repaired-example",
];
fn config(t: &work::Task) -> String {
    t.reference
        .iter()
        .find_map(|a| {
            if let work::Action::Config { text } = a {
                Some(text.clone())
            } else {
                None
            }
        })
        .unwrap()
}
fn entry(practice: bool) -> (String, String) {
    if practice {
        (
            "unfinished-practice.conf".into(),
            "title Practice draft\nlinux /unfinished-practice\n".into(),
        )
    } else {
        (
            "unfinished-assignment.conf".into(),
            "title Assignment draft\nlinux /unfinished-assignment\n".into(),
        )
    }
}
fn command(practice: bool) -> String {
    if practice {
        "make mod"
    } else {
        "make modules_inst"
    }
    .into()
}
fn expected(source: &pack::Scenario) -> work::Session {
    let d = binding::source(source).unwrap();
    let mut s = work::Session::new(d).unwrap();
    for practice in [true, false] {
        let t = if practice { &d.example } else { &d.assignment };
        let (filename, text) = entry(practice);
        for a in [
            work::Action::Source {
                index: t.wanted_source,
            },
            work::Action::Config { text: config(t) },
            work::Action::SaveConfig,
            work::Action::Run {
                input: "make bzImage".into(),
            },
            work::Action::StageImage,
            work::Action::EntryDraft { filename, text },
            work::Action::SaveEntry,
            work::Action::CommandDraft {
                text: command(practice),
            },
            work::Action::Check,
        ] {
            s.apply(d, practice, &a).unwrap();
        }
    }
    s.view = teaching::View::Assignment;
    s
}
fn verify_partial(a: &LabApp) {
    let source = a.scenarios.iter().find(|s| s.id == ID).unwrap();
    let attempt = &a.progress.attempts[ID];
    binding::context(attempt, source).unwrap();
    let engine::State::KernelBuild { simulation } = &attempt.state else {
        panic!()
    };
    assert_eq!(**simulation, expected(source));
    let d = binding::source(source).unwrap();
    for (scope, t) in [
        (&simulation.assignment, &d.assignment),
        (&simulation.example, &d.example),
    ] {
        assert!(scope.work.checked);
        assert!(!t.solved(&scope.work));
        assert!(scope.work.outputs.image.is_some());
        assert!(scope.work.outputs.modules.is_none());
        assert_eq!(scope.work.store.images.len(), 2);
    }
    assert_ne!(
        simulation.assignment.work.entry_draft,
        simulation.example.work.entry_draft
    );
    assert_ne!(
        simulation.assignment.work.command_draft,
        simulation.example.work.command_draft
    );
    assert!(!progress::current_solved(attempt, source));
}
fn plan(read: bool) -> Vec<Op> {
    let all = serde_json::from_str::<pack::Pack>(include_str!(
        "../../../../scenarios/kernel-build-lessons-v1.json"
    ))
    .unwrap();
    let source = all.scenarios.iter().find(|s| s.id == ID).unwrap();
    let d = binding::source(source).unwrap();
    let mut p = vec![];
    if !read {
        p.push(Op::ColdAppearance);
    }
    p.push(Op::Open(ID.into()));
    for practice in if read { [false, true] } else { [true, false] } {
        let t = if practice { &d.example } else { &d.assignment };
        p.push(Op::Click(
            if practice { "example" } else { "assignment" }.into(),
        ));
        if read {
            p.extend([
                Op::Check(false, practice),
                Op::Shot(
                    CAPTURES[if practice { 4 } else { 2 }].into(),
                    "config".into(),
                ),
                Op::Shot(
                    CAPTURES[if practice { 5 } else { 3 }].into(),
                    "failed-case".into(),
                ),
            ]);
            p.extend(expand(t, practice));
            p.push(Op::Shot(
                CAPTURES[if practice { 7 } else { 6 }].into(),
                "result".into(),
            ));
        } else {
            let (filename, text) = entry(practice);
            p.extend([
                Op::Choose(
                    "source".into(),
                    format!("source-option:{}", t.wanted_source),
                ),
                Op::Input("config".into(), config(t)),
                Op::KeyboardActivate("save-config".into()),
                Op::Input("command".into(), "make bzImage".into()),
                Op::Click("run".into()),
                Op::Click("stage-image".into()),
                Op::Input("entry-name".into(), filename),
                Op::Input("entry-text".into(), text),
                Op::Click("save-entry".into()),
                Op::Input("command".into(), command(practice)),
                Op::Click("check".into()),
                Op::Check(false, practice),
                Op::Shot(
                    CAPTURES[if practice { 0 } else { 1 }].into(),
                    "config".into(),
                ),
            ]);
        }
    }
    if read {
        p.push(Op::Click("assignment".into()));
    }
    p.push(Op::ColdFinish);
    p
}
impl LabApp {
    pub(super) fn reopen_kernel_build_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::KernelBuild);
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        session.verify_profile_paths(&self.path).unwrap();
        if session.role == Role::Read {
            let key = egui::Id::new("kernel_build-normal-load-observed");
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
        self.kernel_build_flow(ctx, &plan(session.role == Role::Read));
    }
    pub(in crate::ui) fn finish_kernel_build_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::KernelBuild);
        if session.role == Role::Write {
            verify_partial(self);
        } else {
            let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
            let a = &self.progress.attempts[ID];
            binding::context(a, source).unwrap();
            assert!(progress::current_solved(a, source));
            let d = binding::source(source).unwrap();
            let engine::State::KernelBuild { simulation } = &a.state else {
                panic!()
            };
            assert!(
                d.assignment.solved(&simulation.assignment.work)
                    && d.example.solved(&simulation.example.work)
            );
        }
        let key = egui::Id::new("kernel_build-cold-finished");
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
                "PASS native reopen {}: distinct partial kernel workspaces, source snapshots, checks and exact raw preferences/progress restored before native repair",
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
fn kernel_build_cold_plan_retains_partial_real_artifacts_before_independent_repairs() {
    let labs = pack::current().unwrap();
    let source = labs.iter().find(|s| s.id == ID).unwrap();
    let d = binding::source(source).unwrap();
    let mut s = expected(source);
    s.bind(d).unwrap();
    assert!(!s.current_solved(d));
    for read in [false, true] {
        let ops = plan(read);
        assert!(matches!(ops.last(), Some(Op::ColdFinish)));
        assert!(!ops.iter().any(|o| matches!(o, Op::Style(..))));
    }
    for practice in [false, true] {
        let t = if practice { &d.example } else { &d.assignment };
        for a in &t.reference {
            s.apply(d, practice, a).unwrap();
        }
    }
    assert!(s.current_solved(d));
    assert!(d.example.solved(&s.example.work));
}
