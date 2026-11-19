//! Fixed normal-profile administration reopen. Only native controls create work.
use super::*;
use crate::ui::smoke::linux_service_router::{Op, translate};
use dlms_learning_lab::{
    linux_service_router as work, linux_service_router_source as binding,
    linux_v6_workspace as router,
};
const ID: &str = "lpic2-router-service-name-recovery";
pub(super) const CAPTURES: [&str; 8] = [
    "reopen-services-writer-example-editor",
    "reopen-services-writer-assignment-editor",
    "reopen-services-reader-assignment-editor",
    "reopen-services-reader-assignment-command",
    "reopen-services-reader-example-editor",
    "reopen-services-reader-example-command",
    "reopen-services-repaired-assignment",
    "reopen-services-repaired-example",
];
fn repairs(source: &pack::Scenario, practice: bool) -> Vec<work::Action> {
    if practice {
        binding::source(source).unwrap().worked.clone()
    } else {
        source
            .solution
            .iter()
            .map(|raw| serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap())
            .collect()
    }
}
fn pending(source: &pack::Scenario, practice: bool) -> Vec<work::Action> {
    let all = repairs(source, practice);
    assert!(matches!(
        all.last(),
        Some(work::Action::Router {
            action: router::Action::Check
        })
    ));
    let work::Action::Router {
        action: router::Action::Command { line },
    } = &all[all.len() - 2]
    else {
        panic!("Final pending repair is a named reverse permit")
    };
    let d = binding::source(source).unwrap();
    let t = if practice { &d.example } else { &d.assignment };
    let mut result = all[..all.len() - 2].to_vec();
    result.extend([
        work::Action::Edit {
            text: format!(
                "{}# Unfinished {} handover note\n",
                t.required_services,
                if practice { "example" } else { "assignment" }
            ),
        },
        work::Action::Router {
            action: router::Action::Input { text: line.clone() },
        },
        work::Action::Router {
            action: router::Action::SelectCase { index: 1 },
        },
        work::Action::Router {
            action: router::Action::RunTraffic,
        },
    ]);
    result
}

fn verify_partial(a: &LabApp) {
    let source = a.scenarios.iter().find(|s| s.id == ID).unwrap();
    let attempt = &a.progress.attempts[ID];
    binding::context(attempt, source).unwrap();
    let d = binding::source(source).unwrap();
    let engine::State::LinuxServiceRouter { simulation } = &attempt.state else {
        panic!()
    };
    for (scope, t, practice) in [
        (&simulation.assignment, &d.assignment, false),
        (&simulation.practice, &d.example, true),
    ] {
        let mut expected = work::Scope::new(t).unwrap();
        for action in pending(source, practice) {
            expected.apply(t, &action).unwrap();
        }
        assert_eq!(scope, &expected);
        assert!(!scope.router.checked);
        let checks = t.checks(scope);
        assert!(checks.iter().any(|r| r.passed));
        assert!(checks.iter().any(|r| !r.passed));
        assert!(scope.router.last_trial.is_some());
        assert!(
            scope
                .router
                .input
                .starts_with("ip6tables -A FORWARD -i wan0 -o lan0")
        );
        assert_ne!(scope.editor, scope.saved_services);
        assert_eq!(scope.saved_services, t.required_services);
    }
    assert_ne!(
        simulation.assignment.router.input,
        simulation.practice.router.input
    );
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
                p.push(Op::Shot(
                    CAPTURES[if practice { 4 } else { 2 }].into(),
                    "editor".into(),
                ));
                p.push(Op::Shot(
                    CAPTURES[if practice { 5 } else { 3 }].into(),
                    "command".into(),
                ));
                p.extend([
                    Op::Click("save".into()),
                    Op::Click("run".into()),
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
    pub(super) fn reopen_services_router_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::ServiceRouter);
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        session.verify_profile_paths(&self.path).unwrap();
        if session.role == Role::Read {
            let key = egui::Id::new("v6-normal-load-observed");
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
        self.services_router_flow(ctx, plan(session.role == Role::Read));
    }
    pub(in crate::ui) fn finish_services_router_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::ServiceRouter);
        if session.role == Role::Write {
            verify_partial(self);
        } else {
            let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
            let a = &self.progress.attempts[ID];
            let d = binding::source(source).unwrap();
            let engine::State::LinuxServiceRouter { simulation } = &a.state else {
                panic!()
            };
            assert!(progress::current_solved(a, source));
            assert!(
                d.example
                    .checks(&simulation.practice)
                    .iter()
                    .all(|r| r.passed)
            );
        }
        let key = egui::Id::new("services-router-cold-finished");
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
                "PASS native reopen {}: separate saved services, unsaved editor drafts, installed rules, queued named permits and exact traffic observations restored before native Save/Run/Check repair",
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
fn services_router_cold_plan_restores_unsaved_files_and_queued_names_before_repair() {
    for read in [false, true] {
        let p = plan(read);
        assert!(matches!(p.last(), Some(Op::ColdFinish)));
        assert_eq!(
            p.iter().filter(|o| matches!(o, Op::ColdAppearance)).count(),
            usize::from(!read)
        );
        assert!(!p.iter().any(|o| matches!(o, Op::Style(..))));
    }
    let labs = pack::current().unwrap();
    let source = labs.iter().find(|s| s.id == ID).unwrap();
    let d = binding::source(source).unwrap();
    for (practice, t) in [(false, &d.assignment), (true, &d.example)] {
        let mut scope = work::Scope::new(t).unwrap();
        for a in pending(source, practice) {
            scope.apply(t, &a).unwrap();
        }
        assert!(!scope.router.checked);
        assert_ne!(scope.editor, scope.saved_services);
        assert!(scope.router.last_trial.is_some());
        assert!(t.checks(&scope).iter().any(|c| !c.passed));
        let line = scope.router.input.clone();
        for a in [
            work::Action::Save,
            work::Action::Router {
                action: router::Action::Command { line },
            },
            work::Action::Router {
                action: router::Action::RunTraffic,
            },
            work::Action::Router {
                action: router::Action::Check,
            },
        ] {
            scope.apply(t, &a).unwrap();
        }
        assert!(t.checks(&scope).iter().all(|c| c.passed));
    }
    assert_eq!(
        CAPTURES
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        8
    );
}
