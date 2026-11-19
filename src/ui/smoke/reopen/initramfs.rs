//! Cold restoration of separate partially repaired early-userspace workspaces.
use super::*;
use crate::ui::smoke::initramfs::{Op, expand};
use dlms_learning_lab::{initramfs_source as binding, initramfs_workspace as work};
const ID: &str = "lpic2-initramfs-kernel-handover";
pub(super) const CAPTURES: [&str; 8] = [
    "reopen-initramfs-writer-example-config",
    "reopen-initramfs-writer-assignment-config",
    "reopen-initramfs-reader-assignment-config",
    "reopen-initramfs-reader-assignment-feedback",
    "reopen-initramfs-reader-example-config",
    "reopen-initramfs-reader-example-feedback",
    "reopen-initramfs-repaired-assignment",
    "reopen-initramfs-repaired-example",
];
fn command(practice: bool) -> String {
    if practice {
        "dracut --kver 6.1.121"
    } else {
        "dracut --kver 6.6.61"
    }
    .into()
}
fn partial_ops(t: &work::Task, practice: bool) -> Vec<work::Action> {
    vec![
        t.reference[0].clone(),
        t.reference[1].clone(),
        work::Action::CommandDraft {
            text: command(practice),
        },
        work::Action::Check,
    ]
}
fn expected(source: &pack::Scenario) -> work::Session {
    let d = binding::source(source).unwrap();
    let mut s = work::Session::new(d).unwrap();
    for practice in [true, false] {
        let t = if practice { &d.example } else { &d.assignment };
        for a in partial_ops(t, practice) {
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
    let engine::State::Initramfs { simulation } = &attempt.state else {
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
        assert!(
            scope
                .work
                .store
                .initramfs
                .iter()
                .any(|image| image.path.ends_with("/repaired.img"))
        );
        assert_eq!(scope.work.entry_filename, "trial.conf");
        assert_eq!(scope.work.saved_entry, work::Work::new(t).saved_entry);
        assert!(
            !scope
                .work
                .store
                .entries
                .iter()
                .any(|entry| entry.path.ends_with("/trial.conf"))
        );
    }
    assert_ne!(
        simulation.assignment.work.command_draft,
        simulation.example.work.command_draft
    );
    assert!(!progress::current_solved(attempt, source));
}
fn plan(read: bool) -> Vec<Op> {
    let all = serde_json::from_str::<pack::Pack>(include_str!(
        "../../../../scenarios/initramfs-lessons-v1.json"
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
                    "command".into(),
                ),
                Op::Shot(
                    CAPTURES[if practice { 5 } else { 3 }].into(),
                    "failed-case".into(),
                ),
            ]);
            // The generated image was restored with the unfinished draft.
            // Save the entry that selects it rather than generating it again.
            p.extend(expand(&t.reference[2..], practice));
            p.push(Op::Shot(
                CAPTURES[if practice { 7 } else { 6 }].into(),
                "result".into(),
            ));
        } else {
            let actions = partial_ops(t, practice);
            p.extend(expand(&actions[..actions.len() - 1], practice));
            p.extend([
                Op::Click("check".into()),
                Op::Check(false, practice),
                Op::Shot(
                    CAPTURES[if practice { 0 } else { 1 }].into(),
                    "command".into(),
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
    pub(super) fn reopen_initramfs_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::Initramfs);
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        session.verify_profile_paths(&self.path).unwrap();
        if session.role == Role::Read {
            let key = egui::Id::new("initramfs-normal-load-observed");
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
        self.initramfs_flow(ctx, &plan(session.role == Role::Read));
    }
    pub(in crate::ui) fn finish_initramfs_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::Initramfs);
        if session.role == Role::Write {
            verify_partial(self);
        } else {
            let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
            let a = &self.progress.attempts[ID];
            binding::context(a, source).unwrap();
            assert!(progress::current_solved(a, source));
            let d = binding::source(source).unwrap();
            let engine::State::Initramfs { simulation } = &a.state else {
                panic!()
            };
            assert!(
                d.assignment.solved(&simulation.assignment.work)
                    && d.example.solved(&simulation.example.work)
            );
        }
        let key = egui::Id::new("initramfs-cold-finished");
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
                "PASS native reopen {}: separate generated images, unsaved entries and command drafts restored with exact progress and preferences before repair",
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
fn initramfs_cold_plan_retains_partial_records_before_independent_native_repairs() {
    let raw = serde_json::from_str::<pack::Pack>(include_str!(
        "../../../../scenarios/initramfs-lessons-v1.json"
    ))
    .unwrap();
    let source = raw.scenarios.iter().find(|s| s.id == ID).unwrap();
    let s = expected(source);
    let d = binding::source(source).unwrap();
    s.bind(d).unwrap();
    assert!(!d.assignment.solved(&s.assignment.work));
    assert!(!d.example.solved(&s.example.work));
    let writer = plan(false);
    assert!(matches!(writer.last(), Some(Op::ColdFinish)));
    assert!(
        writer
            .iter()
            .any(|op| matches!(op, Op::Input(k, _) if k == "command"))
    );
    let reader = plan(true);
    assert!(matches!(reader.last(), Some(Op::ColdFinish)));
    assert_eq!(
        reader
            .iter()
            .filter(|op| matches!(op, Op::KeyboardActivate(k) if k == "save-entry"))
            .count(),
        2
    );
    assert!(
        !reader
            .iter()
            .any(|op| matches!(op, Op::Click(k) if k == "run"))
    );
    assert_eq!(CAPTURES.len(), 8);
}
