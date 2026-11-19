//! Normal-process persistence of distinct partial power plans, then native repair.
use super::*;
use crate::ui::smoke::poe::Op;
use dlms_learning_lab::{poe_source as binding, poe_workspace as work};
const ID: &str = "ccna-poe-budget-recovery";
pub(super) const CAPTURES: [&str; 8] = [
    "reopen-poe-writer-example-plan",
    "reopen-poe-writer-assignment-plan",
    "reopen-poe-reader-assignment-plan",
    "reopen-poe-reader-assignment-feedback",
    "reopen-poe-reader-example-plan",
    "reopen-poe-reader-example-feedback",
    "reopen-poe-repaired-assignment",
    "reopen-poe-repaired-example",
];
fn expected(source: &pack::Scenario) -> work::Session {
    let d = binding::source(source).unwrap();
    let mut s = work::Session::new(d).unwrap();
    for practice in [true, false] {
        let t = if practice { &d.example } else { &d.assignment };
        let mut p = t.initial.clone();
        p.attachments[0] = Some(3);
        if practice {
            p.enabled[3] = true;
        }
        s.action(d, &work::Action::Edit { plan: p }, practice)
            .unwrap();
        s.action(d, &work::Action::Check, practice).unwrap();
    }
    s.view = teaching::View::Assignment;
    s
}
fn verify_partial(a: &LabApp) {
    let source = a.scenarios.iter().find(|s| s.id == ID).unwrap();
    let attempt = &a.progress.attempts[ID];
    binding::context(attempt, source).unwrap();
    let engine::State::Poe { simulation } = &attempt.state else {
        panic!()
    };
    assert_eq!(**simulation, expected(source));
    let d = binding::source(source).unwrap();
    assert!(simulation.assignment.checked && simulation.example.checked);
    assert!(!simulation.assignment.solved(&d.assignment) && !simulation.example.solved(&d.example));
    assert_ne!(
        simulation.assignment.plan.enabled,
        simulation.example.plan.enabled
    );
    assert!(!progress::current_solved(attempt, source));
}
fn plan(read: bool) -> Vec<Op> {
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
                Op::Check(false, practice),
                Op::Shot(
                    CAPTURES[if practice { 4 } else { 2 }].into(),
                    "port-choice:0".into(),
                ),
                Op::Shot(
                    CAPTURES[if practice { 5 } else { 3 }].into(),
                    "failed-case".into(),
                ),
            ]);
            if !practice {
                p.extend([Op::Click("enabled:3".into()), Op::Unchecked(false)]);
            }
            p.extend([
                Op::Choose(2, None),
                Op::Unchecked(practice),
                Op::Click("check".into()),
                Op::Check(true, practice),
                Op::Shot(
                    CAPTURES[if practice { 7 } else { 6 }].into(),
                    "result".into(),
                ),
            ]);
        } else {
            p.push(Op::Choose(0, Some(3)));
            if practice {
                p.push(Op::Click("enabled:3".into()));
            }
            p.extend([
                Op::Click("check".into()),
                Op::Check(false, practice),
                Op::Shot(
                    CAPTURES[if practice { 0 } else { 1 }].into(),
                    "enabled:3".into(),
                ),
            ]);
        }
    }
    p.push(Op::ColdFinish);
    p
}
impl LabApp {
    pub(super) fn reopen_poe_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::Poe);
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        session.verify_profile_paths(&self.path).unwrap();
        if session.role == Role::Read {
            let key = egui::Id::new("poe-normal-load-observed");
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
        self.poe_flow(ctx, &plan(session.role == Role::Read));
    }
    pub(in crate::ui) fn finish_poe_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::Poe);
        if session.role == Role::Write {
            verify_partial(self);
        } else {
            let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
            let a = &self.progress.attempts[ID];
            binding::context(a, source).unwrap();
            assert!(progress::current_solved(a, source));
            let d = binding::source(source).unwrap();
            let engine::State::Poe { simulation } = &a.state else {
                panic!()
            };
            assert!(
                simulation.assignment.solved(&d.assignment)
                    && simulation.example.solved(&d.example)
            );
        }
        let key = egui::Id::new("poe-cold-finished");
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
                "PASS native reopen {}: distinct partial power plans, source snapshots, checks and exact raw preferences/progress restored before native repair",
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
fn poe_cold_plan_preserves_distinct_partial_plans_before_the_native_repairs() {
    let labs = pack::current().unwrap();
    let source = labs.iter().find(|s| s.id == ID).unwrap();
    let d = binding::source(source).unwrap();
    let mut s = expected(source);
    d.bound(&s).unwrap();
    assert!(!s.assignment.solved(&d.assignment) && !s.example.solved(&d.example));
    assert_ne!(s.assignment.plan.enabled, s.example.plan.enabled);
    for read in [false, true] {
        let ops = plan(read);
        assert!(matches!(ops.last(), Some(Op::ColdFinish)));
        assert!(!ops.iter().any(|o| matches!(o, Op::Style(..))));
    }
    for practice in [false, true] {
        let mut p = if practice {
            s.example.plan.clone()
        } else {
            s.assignment.plan.clone()
        };
        p.enabled[3] = true;
        p.attachments[2] = None;
        s.action(d, &work::Action::Edit { plan: p }, practice)
            .unwrap();
        s.action(d, &work::Action::Check, practice).unwrap();
    }
    assert!(s.assignment.solved(&d.assignment) && s.example.solved(&d.example));
}
