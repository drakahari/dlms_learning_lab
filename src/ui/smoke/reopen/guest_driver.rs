//! Normal-process save/exit/load; native actions create and repair independent image drafts.
use super::*;
use crate::ui::smoke::guest_driver::{Op, action};
use dlms_learning_lab::{
    guest_driver as core, guest_driver_source as binding, guest_driver_workspace as work,
};
const ID: &str = "lpic1-guest-object-recovery";
pub(super) const CAPTURES: [&str; 6] = [
    "reopen-guest-writer-example",
    "reopen-guest-writer-assignment",
    "reopen-guest-reader-assignment",
    "reopen-guest-reader-example",
    "reopen-guest-repaired-assignment",
    "reopen-guest-repaired-example",
];
fn partial(t: &work::Task, a: &work::Action) -> core::Runtime {
    let mut r = t.initial.clone();
    let work::Action::Package { action } = a else {
        panic!()
    };
    r.package(&t.facts, action).unwrap();
    r
}
fn verify_partial(a: &LabApp) {
    let source = a.scenarios.iter().find(|s| s.id == ID).unwrap();
    let attempt = &a.progress.attempts[ID];
    binding::context(attempt, source).unwrap();
    let d = binding::source(source).unwrap();
    let engine::State::GuestDrivers { simulation } = &attempt.state else {
        panic!()
    };
    for (scope, t, method, input) in [
        (
            &simulation.assignment,
            &d.assignment,
            &d.reference,
            "modprobe ",
        ),
        (
            &simulation.practice,
            &d.example,
            &d.example_steps,
            "modinfo ",
        ),
    ] {
        assert_eq!(scope.runtime, partial(t, &method[0]));
        assert_eq!(scope.input, input);
        assert!(scope.checked);
        assert_eq!(scope.operation, Some(work::PackageOperation::Reinstall));
        let work::Action::Package {
            action: dlms_learning_lab::rpm_packages::Action::ReinstallExact { archive },
        } = &method[0]
        else {
            panic!()
        };
        assert_eq!(&scope.selected_archive, archive);
        assert!(!scope.output.is_empty());
        assert!(t.checks(&scope.runtime).iter().any(|r| !r.passed));
    }
    assert!(!progress::current_solved(attempt, source));
}
fn plan(read: bool) -> &'static Vec<Op> {
    static W: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    static R: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    (if read { &R } else { &W }).get_or_init(|| {
        let labs = pack::load(include_str!(
            "../../../../scenarios/guest-driver-lessons-v1.json"
        ))
        .unwrap();
        let source = labs.iter().find(|s| s.id == ID).unwrap();
        let d = binding::source(source).unwrap();
        let mut p = vec![];
        if !read {
            p.push(Op::ColdAppearance);
        }
        p.push(Op::Open(ID.into()));
        if read {
            p.push(Op::Shot(CAPTURES[2].into(), "command".into()));
            for a in &d.reference[1..] {
                action(&mut p, a);
            }
            p.extend([
                Op::Check(true, false),
                Op::Shot(CAPTURES[4].into(), "result-title".into()),
                Op::Click("example".into()),
                Op::Shot(CAPTURES[3].into(), "command".into()),
            ]);
            for a in &d.example_steps[1..] {
                action(&mut p, a);
            }
            p.extend([
                Op::Check(true, true),
                Op::Shot(CAPTURES[5].into(), "result-title".into()),
            ]);
        } else {
            p.push(Op::Click("example".into()));
            action(
                &mut p,
                &work::Action::Run {
                    input: "lsmod".into(),
                },
            );
            action(&mut p, &d.example_steps[0]);
            p.extend([
                Op::Click("check".into()),
                Op::Check(false, true),
                Op::Text("command".into(), "modinfo ".into()),
                Op::Shot(CAPTURES[0].into(), "command".into()),
                Op::Click("assignment".into()),
            ]);
            action(
                &mut p,
                &work::Action::Run {
                    input: "lspci -k".into(),
                },
            );
            action(&mut p, &d.reference[0]);
            p.extend([
                Op::Click("check".into()),
                Op::Check(false, false),
                Op::Text("command".into(), "modprobe ".into()),
                Op::Shot(CAPTURES[1].into(), "command".into()),
            ]);
        }
        p.push(Op::ColdFinish);
        p
    })
}
impl LabApp {
    pub(super) fn reopen_guest_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::GuestDrivers);
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        session.verify_profile_paths(&self.path).unwrap();
        if session.role == Role::Read {
            let key = egui::Id::new("virtual-normal-load-observed");
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
                assert_eq!(self.scale, 1.5);
                assert_eq!(self.theme_id, THEME);
                session.write("reader-loaded.json", &loaded).unwrap();
                ctx.data_mut(|d| d.insert_temp(key, true));
            }
        }
        self.guest_flow(ctx, plan(session.role == Role::Read));
    }
    pub(in crate::ui) fn finish_guest_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::GuestDrivers);
        if session.role == Role::Write {
            verify_partial(self);
        } else {
            let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
            let a = &self.progress.attempts[ID];
            let d = binding::source(source).unwrap();
            let engine::State::GuestDrivers { simulation } = &a.state else {
                panic!()
            };
            assert!(progress::current_solved(a, source));
            assert!(simulation.practice.checked);
            assert!(
                d.example
                    .checks(&simulation.practice.runtime)
                    .iter()
                    .all(|r| r.passed)
            );
        }
        let key = egui::Id::new("guest-driver-cold-finished");
        if !ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false) {
            self.save();
            assert!(!self.dirty && self.storage_error.is_none());
            let (observed, raw, prefs) = session.observed(self);
            let (progress, preferences, observation) = if session.role == Role::Write {
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
            session.write_raw(progress, &raw, 16_000_000).unwrap();
            session.write_raw(preferences, &prefs, 4096).unwrap();
            session.write(observation, &observed).unwrap();
            ctx.data_mut(|d| d.insert_temp(key, true));
            println!(
                "PASS native reopen {}: two independent package/module image drafts and native repairs",
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
fn guest_cold_plan_uses_native_controls_and_real_preferences_without_style_injection() {
    for read in [false, true] {
        let p = plan(read);
        assert!(!p.iter().any(|o| matches!(o, Op::Style(..))));
        assert!(matches!(p.last(), Some(Op::ColdFinish)));
        assert_eq!(
            p.iter().filter(|o| matches!(o, Op::ColdAppearance)).count(),
            usize::from(!read)
        );
    }
    for text in ["modprobe ", "modinfo "] {
        assert!(
            plan(false)
                .iter()
                .any(|o| matches!(o,Op::Text(_,v)if v==text))
        );
    }
}
