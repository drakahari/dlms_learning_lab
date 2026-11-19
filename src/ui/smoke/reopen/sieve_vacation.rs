//! Normal-process save/exit/load; native actions create and repair independent mail drafts.
use super::*;
use crate::ui::smoke::sieve::Op;
use dlms_learning_lab::{sieve_source as binding, sieve_workspace as work};
const ID: &str = "lpic2-sieve-vacation-routing-recovery";
pub(super) const CAPTURES: [&str; 6] = [
    "reopen-sieve-vacation-writer-example",
    "reopen-sieve-vacation-writer-assignment",
    "reopen-sieve-vacation-reader-assignment",
    "reopen-sieve-vacation-reader-example",
    "reopen-sieve-vacation-repaired-assignment",
    "reopen-sieve-vacation-repaired-example",
];
fn draft(t: &work::Task, practice: bool) -> String {
    let partial = if practice {
        t.reference.replace(":days 2", ":days 3")
    } else {
        t.reference.replace(":days 1", ":days 2")
    };
    assert_ne!(
        partial, t.reference,
        "Keep the authored longer-interval fault"
    );
    format!(
        "# {} repair in progress\n{partial}",
        if practice { "Guided" } else { "Assignment" }
    )
}
fn verify_partial(a: &LabApp) {
    let source = a.scenarios.iter().find(|s| s.id == ID).unwrap();
    let attempt = &a.progress.attempts[ID];
    binding::context(attempt, source).unwrap();
    let d = binding::source(source).unwrap();
    let engine::State::SieveFiltering { simulation } = &attempt.state else {
        panic!()
    };
    for (scope, t, practice) in [
        (&simulation.assignment, &d.assignment, false),
        (&simulation.practice, &d.example, true),
    ] {
        assert_eq!(scope.draft, draft(t, practice));
        assert_eq!(scope.selected, t.cases.len() - 1);
        assert!(scope.checked);
        assert!(!scope.running && !scope.stopped && scope.error.is_none());
        let parsed = t.parse(&scope.draft).unwrap();
        let expected = t
            .cases
            .iter()
            .map(|c| t.evaluate(c, &parsed))
            .collect::<Vec<_>>();
        assert_eq!(scope.results, expected);
        assert_eq!(scope.results.len(), 10);
        assert_eq!(scope.results.iter().filter(|r| r.passed).count(), 9);
        assert!(matches!(
            scope.results[1].vacation,
            Some(dlms_learning_lab::sieve_vacation_policy::RunResult::Recent { last_sent: 0, .. })
        ));
        assert!(scope.results.iter().any(|r| r.passed));
        assert!(scope.results.iter().any(|r| !r.passed));
    }
    assert!(!progress::current_solved(attempt, source));
}
fn plan(read: bool) -> &'static Vec<Op> {
    static W: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    static R: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    (if read { &R } else { &W }).get_or_init(|| {
        let labs = pack::load_catalog(&[
            include_str!("../../../../scenarios/sieve-filter-lessons-v1.json"),
            include_str!("../../../../scenarios/sieve-address-lessons-v1.json"),
            include_str!("../../../../scenarios/sieve-delivery-lessons-v1.json"),
            include_str!("../../../../scenarios/sieve-vacation-lessons-v1.json"),
        ])
        .unwrap();
        let source = labs.iter().find(|s| s.id == ID).unwrap();
        let d = binding::source(source).unwrap();
        let mut p = vec![];
        if !read {
            p.push(Op::ColdAppearance);
        }
        p.push(Op::Open(ID.into()));
        if read {
            p.extend([
                Op::Click("assignment".into()),
                Op::Shot(CAPTURES[2].into(), "editor-viewport".into()),
                Op::Text("editor".into(), d.assignment.reference.clone()),
                Op::Click("run".into()),
                Op::Check(true, false),
                Op::Shot(CAPTURES[4].into(), "overall".into()),
                Op::Click("example".into()),
                Op::Shot(CAPTURES[3].into(), "editor-viewport".into()),
                Op::Text("editor".into(), d.example.reference.clone()),
                Op::Click("run".into()),
                Op::Check(true, true),
                Op::Shot(CAPTURES[5].into(), "overall".into()),
            ]);
        } else {
            for (practice, t, capture) in [
                (true, &d.example, CAPTURES[0]),
                (false, &d.assignment, CAPTURES[1]),
            ] {
                p.extend([
                    Op::Click(if practice { "example" } else { "assignment" }.into()),
                    Op::Text("editor".into(), draft(t, practice)),
                    Op::Click("run".into()),
                    Op::Check(false, practice),
                    Op::Choose("sample".into(), t.cases.last().unwrap().id.clone()),
                    Op::Shot(capture.into(), "overall".into()),
                ]);
            }
        }
        p.push(Op::ColdFinish);
        p
    })
}
impl LabApp {
    pub(super) fn reopen_sieve_vacation_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::SieveVacation);
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
        self.sieve_flow(ctx, plan(session.role == Role::Read));
    }
    pub(in crate::ui) fn finish_sieve_vacation_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::SieveVacation);
        if session.role == Role::Write {
            verify_partial(self);
        } else {
            let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
            let a = &self.progress.attempts[ID];
            let d = binding::source(source).unwrap();
            let engine::State::SieveFiltering { simulation } = &a.state else {
                panic!()
            };
            assert!(progress::current_solved(a, source));
            assert!(simulation.practice.checked);
            assert!(simulation.practice.solved(&d.example));
        }
        let key = egui::Id::new("sieve-vacation-driver-cold-finished");
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
                "PASS native reopen {}: two independent Sieve script drafts/results and native repairs",
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
fn sieve_vacation_cold_plan_uses_native_controls_and_real_preferences_without_style_injection() {
    for read in [false, true] {
        let p = plan(read);
        assert!(!p.iter().any(|o| matches!(o, Op::Style(..))));
        assert!(matches!(p.last(), Some(Op::ColdFinish)));
        assert_eq!(
            p.iter().filter(|o| matches!(o, Op::ColdAppearance)).count(),
            usize::from(!read)
        );
    }
    let edits = plan(false)
        .iter()
        .filter_map(|o| {
            if let Op::Text(k, t) = o {
                assert_eq!(k, "editor");
                Some(t)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(edits.len(), 2);
    assert_ne!(edits[0], edits[1]);
    assert!(edits.iter().all(|t| t.contains("repair in progress")));
}
