//! Normal two-process restoration of source-bound server and observer work.
use super::*;
use crate::ui::smoke::vpn_point::{Op, translate};
use dlms_learning_lab::{vpn_point_source as binding, vpn_point_workspace as work};
const ID: &str = "lpic2-vpn-peer-profile";
pub(super) const CAPTURES: [&str; 8] = [
    "reopen-vpn-point-writer-example-command",
    "reopen-vpn-point-writer-assignment-command",
    "reopen-vpn-point-reader-assignment-command",
    "reopen-vpn-point-reader-assignment-observer",
    "reopen-vpn-point-reader-example-command",
    "reopen-vpn-point-reader-example-observer",
    "reopen-vpn-point-repaired-assignment",
    "reopen-vpn-point-repaired-example",
];
fn repairs(source: &pack::Scenario, practice: bool) -> Vec<work::Action> {
    let d = binding::source(source).unwrap();
    let t = if practice { &d.example } else { &d.assignment };
    let actions: Vec<work::Action> = if practice {
        d.worked.clone()
    } else {
        source
            .solution
            .iter()
            .map(|raw| {
                let e: serde_json::Value =
                    serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
                serde_json::from_value(e["action"].clone()).unwrap()
            })
            .collect()
    };
    let mut good = t.initial.saved.clone();
    for a in actions {
        if let work::Action::Edit { peer, text } = a {
            good[peer] = text;
        }
    }
    let mut result = vec![];
    for (peer, text) in good.into_iter().enumerate() {
        result.extend([
            work::Action::Edit { peer, text },
            work::Action::Save { peer },
        ]);
    }
    let c = t
        .cases
        .iter()
        .find(|c| {
            c.from == 1 && c.expected == dlms_learning_lab::vpn_point_task::Outcome::Reachable
        })
        .unwrap();
    result.extend([
        work::Action::Apply,
        work::Action::Query {
            query: work::Query {
                from: c.from,
                source: c.source.to_string(),
                destination: c.destination.to_string(),
            },
        },
        work::Action::Probe,
        work::Action::Check,
    ]);
    result
}
fn pending(source: &pack::Scenario, practice: bool) -> Vec<work::Action> {
    let d = binding::source(source).unwrap();
    let t = if practice { &d.example } else { &d.assignment };
    let mut actions = repairs(source, practice);
    let mut trial = work::Scope::new(t).unwrap();
    for a in &actions {
        trial.apply(t, a).unwrap();
    }
    let broken = trial.world.saved[0]
        .lines()
        .filter(|l| !l.starts_with("route "))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let draft = format!(
        "{}# Distinct unfinished {} profile draft\n",
        trial.world.saved[1],
        if practice { "example" } else { "assignment" }
    );
    let mut query = trial.query.clone();
    query.destination = if practice {
        "198.51.100."
    } else {
        "203.0.113."
    }
    .into();
    actions.extend([
        work::Action::Edit {
            peer: 0,
            text: broken,
        },
        work::Action::Save { peer: 0 },
        work::Action::Apply,
        work::Action::Probe,
        work::Action::Edit {
            peer: 1,
            text: draft,
        },
        work::Action::Query { query },
    ]);
    actions
}
fn verify_partial(a: &LabApp) {
    let source = a.scenarios.iter().find(|s| s.id == ID).unwrap();
    let attempt = &a.progress.attempts[ID];
    binding::context(attempt, source).unwrap();
    let d = binding::source(source).unwrap();
    let engine::State::VpnPoint { simulation } = &attempt.state else {
        panic!()
    };
    for (scope, t, practice) in [
        (&simulation.assignment, &d.assignment, false),
        (&simulation.example, &d.example, true),
    ] {
        let mut expected = work::Scope::new(t).unwrap();
        for a in pending(source, practice) {
            expected.apply(t, &a).unwrap();
        }
        assert_eq!(
            scope, &expected,
            "Restore actual saved/applied profiles, observations, sources and distinct unfinished editors/query drafts BEFORE repair"
        );
        assert!(!scope.checked && !scope.solved(t));
        assert!(scope.last_probe.is_some());
        assert!(scope.visible_probe().is_none());
        assert_ne!(scope.editors, scope.world.saved);
        assert_eq!(
            scope.world.applied_source.as_ref(),
            Some(&scope.world.saved)
        );
    }
    assert_ne!(simulation.assignment.query, simulation.example.query);
    assert_ne!(simulation.assignment.editors, simulation.example.editors);
    assert!(!progress::current_solved(attempt, source));
}
fn plan(read: bool, source: &pack::Scenario) -> &'static Vec<Op> {
    static W: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    static R: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    (if read { &R } else { &W }).get_or_init(|| {
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
                        "editor-viewport-1".into(),
                    ),
                    Op::Shot(
                        CAPTURES[if practice { 5 } else { 3 }].into(),
                        "destination".into(),
                    ),
                ]);
                for a in repairs(source, practice) {
                    translate(&mut p, &a);
                }
                p.extend([
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
                        "editor-viewport-1".into(),
                    ),
                ]);
            }
        }
        p.push(Op::ColdFinish);
        p
    })
}
impl LabApp {
    pub(super) fn reopen_vpn_point_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::VpnPoint);
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        session.verify_profile_paths(&self.path).unwrap();
        if session.role == Role::Read {
            let key = egui::Id::new("vpn-point-normal-load-observed");
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
        self.vpn_point_flow(ctx, plan(session.role == Role::Read, &source));
    }
    pub(in crate::ui) fn finish_vpn_point_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::VpnPoint);
        if session.role == Role::Write {
            verify_partial(self);
        } else {
            let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
            let attempt = &self.progress.attempts[ID];
            let d = binding::source(source).unwrap();
            let engine::State::VpnPoint { simulation } = &attempt.state else {
                panic!()
            };
            assert!(progress::current_solved(attempt, source));
            for (scope, t) in [
                (&simulation.assignment, &d.assignment),
                (&simulation.example, &d.example),
            ] {
                assert!(scope.solved(t));
                assert_eq!(scope.editors, scope.world.saved);
                assert_eq!(
                    scope.world.applied_source.as_ref(),
                    Some(&scope.world.saved)
                );
                assert!(scope.visible_probe().is_some());
            }
        }
        let key = egui::Id::new("vpn-point-cold-finished");
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
                "PASS native reopen {}: actual saved/applied profiles, path evidence and separate unfinished editors/queries restored before native repair",
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
fn vpn_cold_plan_restores_saved_applied_and_unsaved_work_before_native_repair() {
    let labs = pack::current().unwrap();
    let source = labs.iter().find(|s| s.id == ID).unwrap();
    let d = binding::source(source).unwrap();
    for read in [false, true] {
        let p = plan(read, source);
        assert!(matches!(p.last(), Some(Op::ColdFinish)));
        assert!(!p.iter().any(|o| matches!(o, Op::Style(..))));
    }
    for (practice, t) in [(false, &d.assignment), (true, &d.example)] {
        let mut scope = work::Scope::new(t).unwrap();
        for a in pending(source, practice) {
            scope.apply(t, &a).unwrap();
        }
        assert!(!scope.checked && !scope.solved(t));
        assert_ne!(scope.editors, scope.world.saved);
        assert!(scope.last_probe.is_some() && scope.visible_probe().is_none());
        for a in repairs(source, practice) {
            scope.apply(t, &a).unwrap();
        }
        assert!(scope.solved(t));
        assert!(scope.visible_probe().is_some());
    }
}
