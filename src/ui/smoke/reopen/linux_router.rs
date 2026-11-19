//! Fixed normal-profile administration reopen. Only native controls create work.
use super::*;
use crate::ui::smoke::linux_router::{Op, translate};
use dlms_learning_lab::{linux_router_source as binding, linux_router_workspace as work};
const ID: &str = "lpic2-router-saved-policy-recovery";
pub(super) const CAPTURES: [&str; 6] = [
    "reopen-router-writer-example",
    "reopen-router-writer-assignment",
    "reopen-router-reader-assignment",
    "reopen-router-reader-example",
    "reopen-router-repaired-assignment",
    "reopen-router-repaired-example",
];
fn pending(source: &pack::Scenario, practice: bool) -> Vec<work::Action> {
    let d = binding::source(source).unwrap();
    let t = if practice { &d.example } else { &d.assignment };
    assert!(t.persistence && t.files);
    vec![
        work::Action::Command {
            line: "iptables -S".into(),
        },
        work::Action::Command {
            line: "iptables-save -t filter -f /etc/lab-filter.rules".into(),
        },
        work::Action::File {
            file: work::File::Forwarding,
        },
        work::Action::Edit {
            text: format!(
                "# {} draft not saved yet\nnet.ipv4.ip_forward=1\n",
                if practice { "Guided" } else { "Assignment" }
            ),
        },
        work::Action::Check,
        work::Action::Input {
            text: "sysctl -p /etc/sysctl.d/lab-forwarding.conf".into(),
        },
    ]
}
fn verify_partial(a: &LabApp) {
    let source = a.scenarios.iter().find(|s| s.id == ID).unwrap();
    let attempt = &a.progress.attempts[ID];
    binding::context(attempt, source).unwrap();
    let d = binding::source(source).unwrap();
    let engine::State::LinuxRouter { simulation } = &attempt.state else {
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
        assert_ne!(scope.forward_draft, scope.state.forward_file);
        assert_eq!(scope.filter_draft, scope.state.filter_file);
        assert!(scope.state.forwarding);
        assert_eq!(scope.file, work::File::Forwarding);
        assert!(scope.input.starts_with("sysctl -p "));
    }
    assert_ne!(
        simulation.assignment.forward_draft,
        simulation.practice.forward_draft
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
                    CAPTURES[if practice { 3 } else { 2 }].into(),
                    "command".into(),
                ));
                p.extend([
                    Op::Click("save".into()),
                    Op::Click("run".into()),
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
    pub(super) fn reopen_router_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::LinuxRouter);
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        session.verify_profile_paths(&self.path).unwrap();
        if session.role == Role::Read {
            let key = egui::Id::new("router-normal-load-observed");
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
        self.router_flow(ctx, plan(session.role == Role::Read));
    }
    pub(in crate::ui) fn finish_router_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::LinuxRouter);
        if session.role == Role::Write {
            verify_partial(self);
        } else {
            let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
            let a = &self.progress.attempts[ID];
            let d = binding::source(source).unwrap();
            let engine::State::LinuxRouter { simulation } = &a.state else {
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
        let key = egui::Id::new("router-cold-finished");
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
                "PASS native reopen {}: separate router drafts, live/saved differences and unsent command restored before native repair",
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
fn router_cold_plan_restores_dirty_drafts_before_saving_or_applying() {
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
            work::Action::Save,
            work::Action::Command { line },
            work::Action::Check,
        ] {
            scope.apply(t, &action).unwrap();
        }
        assert!(t.checks(&scope).iter().all(|c| c.passed));
    }
}
