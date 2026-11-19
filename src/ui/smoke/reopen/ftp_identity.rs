//! Normal cold processes restore actual native work before any repair.
use super::*;
use crate::ui::smoke::ftp_identity::{translate, Op};
use dlms_learning_lab::{ftp_identity_source as binding, ftp_identity_workspace as work};
const ID: &str = "lpic2-ftp-upload-rights";
pub(super) const CAPTURES: [&str; 8] = [
    "reopen-ftp-identity-writer-example-editor",
    "reopen-ftp-identity-writer-assignment-editor",
    "reopen-ftp-identity-reader-assignment-editor",
    "reopen-ftp-identity-reader-assignment-command",
    "reopen-ftp-identity-reader-example-editor",
    "reopen-ftp-identity-reader-example-command",
    "reopen-ftp-identity-repaired-assignment",
    "reopen-ftp-identity-repaired-example",
];
fn pending(source: &pack::Scenario, practice: bool) -> Vec<work::Action> {
    let d = binding::source(source).unwrap();
    let (t, inputs) = if practice {
        (&d.example, &d.example_inputs)
    } else {
        (&d.assignment, &d.assignment_inputs)
    };
    let all: Vec<work::Action> = if practice {
        d.worked.clone()
    } else {
        source
            .solution
            .iter()
            .map(|r| serde_json::from_str(r.strip_prefix(binding::PREFIX).unwrap()).unwrap())
            .collect()
    };
    // This fixture's login list already has its correct sole user. Omit that
    // redundant save, keeping all real configuration/permission/client repairs.
    let mut actions: Vec<_> = all
        .into_iter()
        .filter(|a| {
            !matches!(
                a,
                work::Action::Event {
                    event: work::Event::SaveFile { .. }
                }
            )
        })
        .collect();
    let mut probe = work::Scope::new(t, inputs);
    for a in &actions {
        probe.apply(t, inputs, a).unwrap();
    }
    actions.extend([
        work::Action::Event {
            event: work::Event::Connect { client: 0 },
        },
        work::Action::Event {
            event: work::Event::Client {
                input: "cd inbox".into(),
            },
        },
        work::Action::Event {
            event: work::Event::Client {
                input: format!(
                    "put {} cold-report.txt",
                    if practice { "sample.txt" } else { "report.txt" }
                ),
            },
        },
        work::Action::Event {
            event: work::Event::Client {
                input: "get ../brief.txt cold-brief.txt".into(),
            },
        },
        work::Action::Edit {
            text: format!(
                "{}# Unfinished {} note\n",
                probe.world.saved,
                if practice { "example" } else { "assignment" }
            ),
        },
        work::Action::AdminInput {
            text: format!("ls -l {}/inbox/cold-report.txt", t.initial.accounts[0].home),
        },
        work::Action::ClientInput {
            text: "get cold-report.txt recovered.txt".into(),
        },
    ]);
    actions
}
fn verify_partial(a: &LabApp) {
    let source = a.scenarios.iter().find(|s| s.id == ID).unwrap();
    let attempt = &a.progress.attempts[ID];
    binding::context(attempt, source).unwrap();
    let d = binding::source(source).unwrap();
    let engine::State::FtpIdentity { simulation } = &attempt.state else {
        panic!()
    };
    for (scope, t, inputs, practice) in [
        (
            &simulation.assignment,
            &d.assignment,
            &d.assignment_inputs,
            false,
        ),
        (&simulation.example, &d.example, &d.example_inputs, true),
    ] {
        let mut expected = work::Scope::new(t, inputs);
        for a in pending(source, practice) {
            expected.apply(t, inputs, &a).unwrap();
        }
        assert_eq!(scope, &expected);
        assert!(!scope.checked);
        assert_ne!(scope.editor, scope.world.saved);
        assert_eq!(scope.local_files["cold-brief.txt"], "Partner briefing\n");
        assert!(scope
            .world
            .files
            .files
            .contains_key(&(t.initial.accounts[0].home.clone() + "/inbox/cold-report.txt")));
        assert!(t.checks(&scope.world).iter().all(|c| c.passed));
    }
    assert_ne!(
        simulation.assignment.admin_input,
        simulation.example.admin_input
    );
    assert_ne!(
        simulation.assignment.local_files,
        simulation.example.local_files
    );
    assert!(!progress::current_solved(attempt, source));
}
fn plan(read: bool, source: &pack::Scenario) -> &'static Vec<Op> {
    static W: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    static R: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    (if read { &R } else { &W }).get_or_init(|| {
        assert_eq!(source.id, ID);
        let mut p = vec![];
        if !read {
            p.push(Op::ColdAppearance);
        }
        p.push(Op::Open(ID.into()));
        for practice in if read { [false, true] } else { [true, false] } {
            p.push(Op::Click(
                if practice { "example" } else { "assignment" }.into(),
            ));
            if read {
                p.extend([
                    Op::Shot(
                        CAPTURES[if practice { 4 } else { 2 }].into(),
                        "editor-viewport".into(),
                    ),
                    Op::Shot(
                        CAPTURES[if practice { 5 } else { 3 }].into(),
                        "client-input".into(),
                    ),
                    Op::Click("client-run".into()),
                    Op::Click("admin-run".into()),
                    Op::Click("save".into()),
                    Op::Click("check".into()),
                    Op::Check(true, practice),
                    Op::Shot(
                        CAPTURES[if practice { 7 } else { 6 }].into(),
                        "result".into(),
                    ),
                ]);
            } else {
                for a in pending(source, practice) {
                    translate(&mut p, &a);
                }
                p.extend([
                    Op::NotRun(practice),
                    Op::Shot(
                        CAPTURES[if practice { 0 } else { 1 }].into(),
                        "editor-viewport".into(),
                    ),
                ]);
            }
        }
        p.push(Op::ColdFinish);
        p
    })
}
impl LabApp {
    pub(super) fn reopen_ftp_identity_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::FtpIdentity);
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        session.verify_profile_paths(&self.path).unwrap();
        if session.role == Role::Read {
            let key = egui::Id::new("ftp-identity-normal-load-observed");
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
        self.ftp_identity_flow(ctx, plan(session.role == Role::Read, &source));
    }
    pub(in crate::ui) fn finish_ftp_identity_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::FtpIdentity);
        if session.role == Role::Write {
            verify_partial(self);
        } else {
            let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
            let a = &self.progress.attempts[ID];
            let d = binding::source(source).unwrap();
            let engine::State::FtpIdentity { simulation } = &a.state else {
                panic!()
            };
            assert!(progress::current_solved(a, source));
            for (scope, inputs) in [
                (&simulation.assignment, &d.assignment_inputs),
                (&simulation.example, &d.example_inputs),
            ] {
                let original = inputs
                    .get(if inputs.contains_key("report.txt") {
                        "report.txt"
                    } else {
                        "sample.txt"
                    })
                    .unwrap();
                assert_eq!(
                    scope.local_files["recovered.txt"], *original,
                    "Restored cwd/session must download actual earlier upload"
                );
                assert!(scope.client_input.is_empty() && scope.admin_input.is_empty());
            }
            assert!(simulation
                .example
                .checks(&d.example, &d.example_inputs)
                .iter()
                .all(|r| r.passed));
        }
        let key = egui::Id::new("ftp-identity-cold-finished");
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
                "PASS native reopen {}: separate saved FTP policy, unsaved editors, queued admin/client commands, actual uploads/download buffers and client cwd restored before native Run/Save/Check repair",
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
fn ftp_identity_cold_plan_preserves_real_uploads_cwd_buffers_and_separate_drafts() {
    let labs = pack::current().unwrap();
    let source = labs.iter().find(|s| s.id == ID).unwrap();
    for read in [false, true] {
        let p = plan(read, source);
        assert!(matches!(p.last(), Some(Op::ColdFinish)));
        assert!(!p.iter().any(|o| matches!(o, Op::Style(..))));
    }
    let d = binding::source(source).unwrap();
    for (practice, t, inputs) in [
        (false, &d.assignment, &d.assignment_inputs),
        (true, &d.example, &d.example_inputs),
    ] {
        let mut scope = work::Scope::new(t, inputs);
        for a in pending(source, practice) {
            scope.apply(t, inputs, &a).unwrap();
        }
        assert!(!scope.checked);
        assert_ne!(scope.editor, scope.world.saved);
        for a in [
            work::Action::Event {
                event: work::Event::Client {
                    input: scope.client_input.clone(),
                },
            },
            work::Action::Event {
                event: work::Event::Admin {
                    input: scope.admin_input.clone(),
                },
            },
            work::Action::Save,
            work::Action::Check,
        ] {
            scope.apply(t, inputs, &a).unwrap();
        }
        assert!(scope.checks(t, inputs).iter().all(|c| c.passed));
        assert_eq!(
            scope.local_files["recovered.txt"],
            inputs[if practice { "sample.txt" } else { "report.txt" }]
        );
    }
}
