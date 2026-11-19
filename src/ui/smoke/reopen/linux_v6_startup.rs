//! Normal cold processes restore actual native work before any repair.
use super::*;
use crate::ui::smoke::linux_v6_startup::{Op, translate};
use dlms_learning_lab::{linux_v6_startup_source as binding, linux_v6_startup_workspace as work};
const ID: &str = "lpic2-ipv6-startup-policy-recovery";
pub(super) const CAPTURES: [&str; 8] = [
    "reopen-v6-startup-writer-example-editor",
    "reopen-v6-startup-writer-assignment-editor",
    "reopen-v6-startup-reader-assignment-editor",
    "reopen-v6-startup-reader-assignment-command",
    "reopen-v6-startup-reader-example-editor",
    "reopen-v6-startup-reader-example-command",
    "reopen-v6-startup-repaired-assignment",
    "reopen-v6-startup-repaired-example",
];
fn pending(source: &pack::Scenario, practice: bool) -> Vec<work::Action> {
    let d = binding::source(source).unwrap();
    let t = if practice { &d.example } else { &d.assignment };
    let all: Vec<work::Action> = if practice {
        d.worked.clone()
    } else {
        source
            .solution
            .iter()
            .map(|s| serde_json::from_str(s.strip_prefix(binding::PREFIX).unwrap()).unwrap())
            .collect()
    };
    let mut probe = work::Scope::new(t);
    let mut actions = vec![];
    for action in all.iter().take(6) {
        probe.reference(t, action).unwrap();
        actions.push(action.clone());
    }
    actions.extend([
        work::Action::EditFile {
            text: format!(
                "{}# Unfinished {} note\n",
                probe.world.saved_filter,
                if practice { "example" } else { "assignment" }
            ),
        },
        work::Action::Input {
            text: if practice {
                "systemctl  enable lab-ipv6-filter.service".into()
            } else {
                "systemctl enable lab-ipv6-filter.service".into()
            },
        },
        work::Action::RunCase,
        work::Action::RunStartup,
    ]);
    actions
}
fn verify_partial(a: &LabApp) {
    let source = a.scenarios.iter().find(|s| s.id == ID).unwrap();
    let attempt = &a.progress.attempts[ID];
    binding::context(attempt, source).unwrap();
    let d = binding::source(source).unwrap();
    let engine::State::LinuxV6Startup { simulation } = &attempt.state else {
        panic!()
    };
    for (scope, t, practice) in [
        (&simulation.assignment, &d.assignment, false),
        (&simulation.practice, &d.example, true),
    ] {
        let mut expected = work::Scope::new(t);
        for action in pending(source, practice) {
            expected.apply(t, &action).unwrap();
        }
        assert_eq!(scope, &expected);
        assert!(!scope.checked && !scope.world.enabled);
        assert_ne!(scope.editor, scope.world.saved_filter);
        assert!(
            scope
                .last_trial
                .as_ref()
                .is_some_and(|v| v.steps.iter().all(|s| s.check.passed))
        );
        assert!(
            scope.checks(t).iter().any(|c| c.passed) && scope.checks(t).iter().any(|c| !c.passed)
        );
    }
    assert_ne!(simulation.assignment.input, simulation.practice.input);
    assert!(!progress::current_solved(attempt, source));
}
fn plan(read: bool) -> &'static Vec<Op> {
    static W: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    static R: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    (if read { &R } else { &W }).get_or_init(|| {
        let labs = pack::current().unwrap();
        let source = labs.iter().find(|s| s.id == ID).unwrap();
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
                        "editor".into(),
                    ),
                    Op::Shot(
                        CAPTURES[if practice { 5 } else { 3 }].into(),
                        "command".into(),
                    ),
                    Op::Click("save".into()),
                    Op::Click("run".into()),
                    Op::Text(
                        "command".into(),
                        "systemctl restart lab-ipv6-filter.service".into(),
                    ),
                    Op::Click("run".into()),
                    Op::Click("startup".into()),
                    Op::Click("run-traffic".into()),
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
                        "editor".into(),
                    ),
                ]);
            }
        }
        p.push(Op::ColdFinish);
        p
    })
}
impl LabApp {
    pub(super) fn reopen_v6_startup_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::V6Startup);
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        session.verify_profile_paths(&self.path).unwrap();
        if session.role == Role::Read {
            let key = egui::Id::new("v6-startup-normal-load-observed");
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
        self.v6_startup_flow(ctx, plan(session.role == Role::Read));
    }
    pub(in crate::ui) fn finish_v6_startup_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::V6Startup);
        if session.role == Role::Write {
            verify_partial(self);
        } else {
            let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
            let a = &self.progress.attempts[ID];
            let d = binding::source(source).unwrap();
            let engine::State::LinuxV6Startup { simulation } = &a.state else {
                panic!()
            };
            assert!(progress::current_solved(a, source));
            assert!(
                simulation
                    .practice
                    .checks(&d.example)
                    .iter()
                    .all(|r| r.passed)
            );
        }
        let key = egui::Id::new("v6-startup-cold-finished");
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
                "PASS native reopen {}: separate saved IPv6 rules, separate unsaved editor/command drafts, current policy and failed startup restored before native Save/Enable/Restart/Test/Check repair",
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
fn v6_startup_cold_plan_preserves_unsaved_drafts_and_requires_enable_and_load() {
    for read in [false, true] {
        let p = plan(read);
        assert!(matches!(p.last(), Some(Op::ColdFinish)));
        assert!(!p.iter().any(|o| matches!(o, Op::Style(..))));
    }
    let labs = pack::current().unwrap();
    let s = labs.iter().find(|s| s.id == ID).unwrap();
    let d = binding::source(s).unwrap();
    for (practice, t) in [(false, &d.assignment), (true, &d.example)] {
        let mut scope = work::Scope::new(t);
        for a in pending(s, practice) {
            scope.apply(t, &a).unwrap();
        }
        assert!(!scope.checked && !scope.world.enabled);
        assert_ne!(scope.editor, scope.world.saved_filter);
        let input = scope.input.clone();
        for a in [
            work::Action::SaveFile,
            work::Action::Command { input },
            work::Action::Command {
                input: "systemctl restart lab-ipv6-filter.service".into(),
            },
            work::Action::RunStartup,
            work::Action::Check,
        ] {
            scope.apply(t, &a).unwrap();
        }
        assert!(scope.checks(t).iter().all(|c| c.passed));
    }
}
