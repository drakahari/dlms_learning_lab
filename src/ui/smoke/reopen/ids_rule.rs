//! Fixed normal-process rule restoration; mutations use actual native controls.
use super::*;
use crate::ui::smoke::ids_rule::{Op, translate};
use dlms_learning_lab::{ids_rule_source as binding, ids_rule_workspace as work};

const ID: &str = "lpic2-ids-content-specificity";
pub(super) const CAPTURES: [&str; 6] = [
    "reopen-ids-rule-writer-example",
    "reopen-ids-rule-writer-assignment",
    "reopen-ids-rule-reader-assignment",
    "reopen-ids-rule-repaired-assignment",
    "reopen-ids-rule-reader-example",
    "reopen-ids-rule-repaired-example",
];

fn correct(source: &pack::Scenario, practice: bool) -> String {
    let d = binding::source(source).unwrap();
    if practice {
        d.worked
            .iter()
            .find_map(|a| match a {
                work::Action::Edit { text } => Some(text.clone()),
                _ => None,
            })
            .unwrap()
    } else {
        source
            .solution
            .iter()
            .find_map(|input| {
                let value: serde_json::Value =
                    serde_json::from_str(input.strip_prefix(binding::PREFIX).unwrap()).unwrap();
                match serde_json::from_value::<work::Action>(value["action"].clone()).unwrap() {
                    work::Action::Edit { text } => Some(text),
                    _ => None,
                }
            })
            .unwrap()
    }
}
fn partial_actions(source: &pack::Scenario, practice: bool) -> Vec<work::Action> {
    let partial = if practice {
        correct(source, true).replace("172.20.16.0/20", "172.20.16.0/24")
    } else {
        correct(source, false).replace("10.24.32.0/20", "10.24.32.0/24")
    };
    vec![
        work::Action::Edit {
            text: partial.clone(),
        },
        work::Action::Save,
        work::Action::Select { case: 1 },
        work::Action::Replay,
        work::Action::Check,
        work::Action::Edit {
            text: partial
                + if practice {
                    "\n# Separate unfinished example\nalert tcp "
                } else {
                    "\n# Unfinished assignment\nalert "
                },
        },
    ]
}
fn expected(source: &pack::Scenario) -> work::Session {
    let d = binding::source(source).unwrap();
    let mut s = work::Session::new(d).unwrap();
    for practice in [true, false] {
        for a in partial_actions(source, practice) {
            s.action(d, &a, practice).unwrap();
        }
    }
    s.view = teaching::View::Assignment;
    s
}
fn verify_partial(app: &LabApp) {
    let source = app.scenarios.iter().find(|s| s.id == ID).unwrap();
    let attempt = &app.progress.attempts[ID];
    let engine::State::IdsRule { simulation } = &attempt.state else {
        panic!()
    };
    assert_eq!(
        simulation.as_ref(),
        &expected(source),
        "Actual saved rules, histories, record tests and distinct unfinished drafts restored before repair"
    );
    assert!(!progress::current_solved(attempt, source));
}
fn plan(read: bool, source: &pack::Scenario) -> &'static Vec<Op> {
    static WRITE: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    static READ: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    (if read { &READ } else { &WRITE }).get_or_init(|| {
        let mut ops = vec![];
        if !read {
            ops.push(Op::ColdAppearance);
        }
        ops.push(Op::Open(ID.into()));
        for practice in if read { [false, true] } else { [true, false] } {
            ops.push(Op::Click(
                if practice { "example" } else { "assignment" }.into(),
            ));
            if read {
                let first = if practice { 4 } else { 2 };
                ops.push(Op::Shot(CAPTURES[first].into(), "editor-viewport".into()));
                for a in [
                    work::Action::Edit {
                        text: correct(source, practice),
                    },
                    work::Action::Save,
                    work::Action::Replay,
                    work::Action::Check,
                ] {
                    translate(&mut ops, &a);
                }
                ops.push(Op::Check(true, practice));
                ops.push(Op::Shot(CAPTURES[first + 1].into(), "result".into()));
            } else {
                for a in partial_actions(source, practice) {
                    translate(&mut ops, &a);
                }
                ops.push(Op::Check(false, practice));
                ops.push(Op::Shot(
                    CAPTURES[usize::from(!practice)].into(),
                    "editor-viewport".into(),
                ));
            }
        }
        ops.push(Op::ColdFinish);
        ops
    })
}
impl LabApp {
    pub(super) fn reopen_ids_rule_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::IdsRule);
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        session.verify_profile_paths(&self.path).unwrap();
        if session.role == Role::Read {
            let key = egui::Id::new("ids-rule-normal-load-observed");
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
        self.ids_rule_flow(ctx, plan(session.role == Role::Read, &source));
    }
    pub(in crate::ui) fn finish_ids_rule_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::IdsRule);
        if session.role == Role::Write {
            verify_partial(self);
        } else {
            let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
            let d = binding::source(source).unwrap();
            let attempt = &self.progress.attempts[ID];
            assert!(progress::current_solved(attempt, source));
            let engine::State::IdsRule { simulation } = &attempt.state else {
                panic!()
            };
            for (s, t) in [
                (&simulation.assignment, &d.assignment),
                (&simulation.example, &d.example),
            ] {
                assert!(s.checked && s.checks(t).iter().all(|r| r.passed));
                assert_eq!(s.editor, s.saved);
                assert_eq!(s.last_run.as_ref().unwrap().rule, s.saved);
                assert_eq!(s.last_run.as_ref().unwrap().case, 1);
            }
        }
        let key = egui::Id::new("ids-rule-cold-finished");
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
                "PASS native reopen {}: saved rules, actual tests and distinct unfinished drafts restored before native repair",
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
fn ids_rule_cold_plan_retains_real_partial_results_and_distinct_pending_work() {
    let p: pack::Pack = serde_json::from_str(include_str!(
        "../../../../scenarios/ids-rule-lessons-v1.json"
    ))
    .unwrap();
    let source = p.scenarios.iter().find(|s| s.id == ID).unwrap();
    let d = binding::source(source).unwrap();
    let s = expected(source);
    d.bound(&s).unwrap();
    assert_ne!(s.assignment.editor, s.example.editor);
    for (s, t) in [(&s.assignment, &d.assignment), (&s.example, &d.example)] {
        assert_ne!(s.saved, s.editor);
        assert!(s.checked && s.checks(t).iter().any(|r| !r.passed));
        let actual = t.checks(&s.saved);
        assert_eq!(actual.iter().filter(|r| r.passed).count(), 2);
        assert_eq!(actual.len(), 3);
        assert_eq!(s.last_run.as_ref().unwrap().case, 1);
    }
    assert!(
        plan(false, source)
            .iter()
            .any(|o| matches!(o, Op::ColdFinish))
    );
    for practice in [true, false] {
        assert!(
            plan(true, source)
                .iter()
                .any(|o| matches!(o, Op::Check(true, p) if *p == practice))
        );
    }
}
