//! Fixed normal-profile administration reopen. Only native controls create work.
use super::*;
use crate::ui::smoke::linux_flow::{Op, translate};
use dlms_learning_lab::{linux_flow_source as binding, linux_flow_workspace as work};
const ID: &str = "lpic2-nat-published-service-policy";
pub(super) const CAPTURES: [&str; 6] = [
    "reopen-flow-writer-example",
    "reopen-flow-writer-assignment",
    "reopen-flow-reader-assignment",
    "reopen-flow-reader-example",
    "reopen-flow-repaired-assignment",
    "reopen-flow-repaired-example",
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
    assert!(matches!(all.last(), Some(work::Action::Check)));
    let work::Action::Command { line } = &all[all.len() - 2] else {
        panic!("Final pending repair is a literal NAT rule")
    };
    let mut result = all[..all.len() - 2].to_vec();
    result.extend([
        work::Action::Input { text: line.clone() },
        work::Action::RunTraffic,
    ]);
    result
}
fn verify_partial(a: &LabApp) {
    let source = a.scenarios.iter().find(|s| s.id == ID).unwrap();
    let attempt = &a.progress.attempts[ID];
    binding::context(attempt, source).unwrap();
    let d = binding::source(source).unwrap();
    let engine::State::LinuxFlow { simulation } = &attempt.state else {
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
        assert!(!scope.checked);
        let checks = t.checks(scope);
        assert!(checks.iter().any(|r| r.passed));
        assert!(checks.iter().any(|r| !r.passed));
        assert!(scope.last_trial.is_some());
        assert!(scope.input.starts_with("iptables -t nat -A PREROUTING"));
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
                p.push(Op::Shot(
                    CAPTURES[if practice { 3 } else { 2 }].into(),
                    "command".into(),
                ));
                p.extend([
                    Op::Click("run".into()),
                    Op::Click("run-traffic".into()),
                    Op::Click("check".into()),
                    Op::Check(true, practice),
                    Op::Shot(
                        CAPTURES[if practice { 5 } else { 4 }].into(),
                        "result".into(),
                    ),
                ]);
            } else {
                for action in pending(source, practice) {
                    translate(&mut p, &action);
                }
                p.extend([
                    Op::NotRun(practice),
                    Op::Shot(
                        CAPTURES[if practice { 0 } else { 1 }].into(),
                        "command".into(),
                    ),
                ]);
            }
        }
        p.push(Op::ColdFinish);
        p
    })
}
impl LabApp {
    pub(super) fn reopen_flow_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::LinuxFlow);
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        session.verify_profile_paths(&self.path).unwrap();
        if session.role == Role::Read {
            let key = egui::Id::new("flow-normal-load-observed");
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
        self.connection_flow(ctx, plan(session.role == Role::Read));
    }
    pub(in crate::ui) fn finish_flow_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::LinuxFlow);
        if session.role == Role::Write {
            verify_partial(self);
        } else {
            let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
            let a = &self.progress.attempts[ID];
            let d = binding::source(source).unwrap();
            let engine::State::LinuxFlow { simulation } = &a.state else {
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
        let key = egui::Id::new("flow-cold-finished");
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
                "PASS native reopen {}: separate unfinished NAT policies, queued commands and exact traffic results restored before native repair",
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
fn flow_cold_plan_restores_policy_and_unsent_commands_before_running() {
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
        for action in pending(source, practice) {
            scope.apply(t, &action).unwrap();
        }
        assert!(!scope.checked);
        assert!(t.checks(&scope).iter().any(|c| !c.passed));
        let line = scope.input.clone();
        for action in [
            work::Action::RunTraffic,
            work::Action::Command { line },
            work::Action::Check,
        ] {
            scope.apply(t, &action).unwrap();
        }
        assert!(t.checks(&scope).iter().all(|c| c.passed));
    }
}
