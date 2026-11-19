//! Fixed private cold-reopen activity; observations never inject learner state.
//! A fixed two-process Environment case using normal startup and native events.
use super::*;
use dlms_learning_lab::environment_workspace as env;
use std::collections::BTreeSet;

const COMPLETE: &str = "lpic1-env-child-settings";
const UNFINISHED: &str = "lpic1-env-parent-note";
const FIRST_EXAMPLE: &str =
    "# Kept separate from the verified setup\nLOCATION=\"$1\"\nSTYLE='unfinished gallery";
const SECOND_EXAMPLE: &str =
    "# Reminder experiment to resume later\nLOCATION=\"$1\"\nSTYLE='unfinished reminder";
const SECOND_ASSIGNMENT: &str =
    "# Planning-note repair to resume later\nSITE=\"$1\"\nMODE='unfinished assignment";

pub(super) const CAPTURES: [&str; 15] = [
    "reopen-env-write-library",
    "reopen-env-write-first-introduction",
    "reopen-env-write-first-example-verified",
    "reopen-env-write-first-example-unfinished",
    "reopen-env-write-first-assignment-verified",
    "reopen-env-write-second-example-unfinished",
    "reopen-env-write-second-assignment-unfinished",
    "reopen-env-write-saved-appearance",
    "reopen-env-read-loaded-library",
    "reopen-env-read-first-assignment-reverified",
    "reopen-env-read-first-example-preserved",
    "reopen-env-read-second-assignment-preserved",
    "reopen-env-read-second-example-preserved",
    "reopen-env-read-second-example-repaired",
    "reopen-env-read-second-assignment-repaired",
];

#[derive(Clone, Copy)]
enum Text {
    Worked(&'static str),
    Reference(&'static str),
    Draft(&'static str),
}
#[derive(Clone, Copy)]
enum Step {
    Filter(&'static str),
    Favorite,
    Open(&'static str),
    Try,
    Assignment,
    Tab(usize),
    Focus,
    Type(Text),
    Run,
    Wait(&'static str, bool),
    Back,
    CheckSaved(bool),
    LoadedExact,
    ReplayExact,
    ScaleOpen,
    ScaleChoice,
    ThemeOpen,
    ThemeChoice,
    Geometry,
    Shot(usize),
    Finish,
}
fn steps(role: Role) -> Vec<Step> {
    use Step::*;
    let result = match role {
        Role::Write => vec![
            Filter(COMPLETE),
            Shot(0),
            Favorite,
            Open(COMPLETE),
            Shot(1),
            Try,
            Run,
            Wait(COMPLETE, true),
            Shot(2),
            Focus,
            Type(Text::Draft(FIRST_EXAMPLE)),
            Shot(3),
            Assignment,
            Focus,
            Type(Text::Reference(COMPLETE)),
            Run,
            Wait(COMPLETE, false),
            Shot(4),
            Back,
            Filter(UNFINISHED),
            Open(UNFINISHED),
            Try,
            Focus,
            Type(Text::Draft(SECOND_EXAMPLE)),
            Shot(5),
            Assignment,
            Focus,
            Type(Text::Draft(SECOND_ASSIGNMENT)),
            CheckSaved(true),
            Shot(6),
            ScaleOpen,
            ScaleChoice,
            ThemeOpen,
            ThemeChoice,
            Geometry,
            Shot(7),
            Finish,
        ],
        Role::Read => vec![
            LoadedExact,
            Shot(8),
            Wait(COMPLETE, false),
            ReplayExact,
            Filter(COMPLETE),
            Open(COMPLETE),
            CheckSaved(true),
            Shot(9),
            Tab(1),
            Shot(10),
            Back,
            Filter(UNFINISHED),
            Open(UNFINISHED),
            Shot(11),
            Tab(1),
            Shot(12),
            Focus,
            Type(Text::Worked(UNFINISHED)),
            Run,
            Wait(UNFINISHED, true),
            Shot(13),
            Tab(2),
            Focus,
            Type(Text::Reference(UNFINISHED)),
            Run,
            Wait(UNFINISHED, false),
            Shot(14),
            Finish,
        ],
    };
    assert!(result.len() <= 64, "fixed environment action-phase ceiling");
    result
}
fn state(a: &Attempt) -> &env::Session {
    let engine::State::Environment { simulation } = &a.state else {
        panic!("fixed Environment activity")
    };
    simulation
}

impl LabApp {
    fn reopen_env_private(&self, session: &Session) {
        assert!(!self.smoke && self._smoke_profile.is_none());
        assert!(self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        assert!(self.storage_error.is_none() && self.preferences_error.is_none());
        session.verify_profile_paths(&self.path).unwrap();
        assert_eq!(session.case, Case::Environment);
        assert_eq!(shell::EVALUATOR_REVISION, 19);
    }
    fn reopen_env_scenario(&self, id: &str) -> &Scenario {
        self.scenarios
            .iter()
            .find(|s| s.id == id)
            .expect("fixed environment lab")
    }
    fn reopen_env_saved(&self, verified: bool) {
        assert_eq!(self.progress.attempts.len(), 2);
        assert_eq!(
            self.progress.favorites,
            BTreeSet::from([COMPLETE.to_owned()])
        );
        let first = &self.progress.attempts[COMPLETE];
        let first_source = self.reopen_env_scenario(COMPLETE);
        env::context(first, first_source).unwrap();
        let a = state(first);
        assert_eq!(a.assignment.source, first_source.solution[0]);
        let run = a
            .assignment
            .result
            .as_ref()
            .expect("actual writer run retained");
        assert_eq!(run.revision, shell::EVALUATOR_REVISION);
        assert_eq!(run.results.len(), a.definition.assignment.cases.len());
        assert_eq!(run.verified(), verified);
        assert_eq!(progress::current_solved(first, first_source), verified);
        assert_eq!(a.example.source, FIRST_EXAMPLE);
        assert!(a.example.result.is_none());
        let second = &self.progress.attempts[UNFINISHED];
        env::context(second, self.reopen_env_scenario(UNFINISHED)).unwrap();
        let b = state(second);
        assert_eq!(b.assignment.source, SECOND_ASSIGNMENT);
        assert_eq!(b.example.source, SECOND_EXAMPLE);
        assert!(b.assignment.result.is_none() && b.example.result.is_none());
        assert!(!progress::current_solved(
            second,
            self.reopen_env_scenario(UNFINISHED)
        ));
    }
    /// Root must call ONCE after ordinary profile/preferences loading and lock
    /// acquisition, before the first render/advance_environment. Observation only:
    /// no injected State, token, flag, suppression, completed record or UI event.
    pub(in crate::ui) fn reopen_environment_loaded(&self, session: &Session) {
        if session.case != Case::Environment || session.role != Role::Read {
            return;
        }
        self.reopen_env_private(session);
        self.reopen_env_saved(false);
        assert!(
            state(&self.progress.attempts[COMPLETE])
                .assignment
                .needs_reverification(
                    &state(&self.progress.attempts[COMPLETE])
                        .definition
                        .assignment
                )
        );
        assert_eq!(self.theme_id, THEME);
        assert_eq!(self.scale, SCALE);
        let expected: Observed = serde_json::from_slice(
            &bounded_file(&session.root.join("writer-observed.json"), JSON_CAP).unwrap(),
        )
        .unwrap();
        let (loaded, raw, prefs) = session.observed(self);
        assert_eq!(
            raw,
            bounded_file(&session.root.join("writer-progress.raw"), 16_000_000).unwrap()
        );
        assert_eq!(
            prefs,
            bounded_file(&session.root.join("writer-preferences.raw"), 4096).unwrap()
        );
        assert_eq!(
            loaded, expected,
            "exact normal raw/portable/provenance/preferences load"
        );
        session.write("reader-loaded.json", &loaded).unwrap();
        let first = &self.progress.attempts[COMPLETE];
        let second = state(&self.progress.attempts[UNFINISHED]);
        session.write("reader-before-replay.json", &serde_json::json!({
            "evaluator_revision": shell::EVALUATOR_REVISION,
            "saved_record_verified": state(first).assignment.result.as_ref().unwrap().verified(),
            "current_assignment_credit": progress::current_solved(first, self.reopen_env_scenario(COMPLETE)),
            "drafts_preserved": state(first).example.source == FIRST_EXAMPLE
                && second.assignment.source == SECOND_ASSIGNMENT
                && second.example.source == SECOND_EXAMPLE
                && state(first).example.result.is_none()
                && second.assignment.result.is_none() && second.example.result.is_none(),
            "note": "Observed after ordinary cold load, before the first cooperative replay slice"
        })).unwrap();
    }
    fn reopen_env_wait(&self, ctx: &egui::Context, id: &str, example: bool) -> bool {
        let a = &self.progress.attempts[id];
        let s = state(a);
        let (task, draft) = if example {
            (&s.definition.example, &s.example)
        } else {
            (&s.definition.assignment, &s.assignment)
        };
        if draft.result.as_ref().is_some_and(|r| r.verified()) {
            assert!(env::checks(task, draft).iter().all(|r| r.0));
            if !example {
                assert!(progress::current_solved(a, self.reopen_env_scenario(id)));
            }
            return true;
        }
        let key = egui::Id::new(("reopen-env-wait", self.smoke_stage));
        let started = ctx.data_mut(|data| {
            let value = data.get_temp::<Instant>(key).unwrap_or_else(Instant::now);
            data.insert_temp(key, value);
            value
        });
        assert!(
            started.elapsed().as_secs() < 15,
            "bounded Environment replay/run deadline"
        );
        assert!(
            self.environment_run.is_some(),
            "actual requested run ended without a verified result"
        );
        ctx.request_repaint();
        false
    }
    fn reopen_env_reader_repaired(&self) {
        let first = &self.progress.attempts[COMPLETE];
        assert!(progress::current_solved(
            first,
            self.reopen_env_scenario(COMPLETE)
        ));
        assert_eq!(state(first).example.source, FIRST_EXAMPLE);
        assert!(state(first).example.result.is_none());
        let second = &self.progress.attempts[UNFINISHED];
        let s = state(second);
        assert!(progress::current_solved(
            second,
            self.reopen_env_scenario(UNFINISHED)
        ));
        assert_eq!(s.example.source, s.definition.example.starter);
        assert!(s.example.result.as_ref().unwrap().verified());
        assert!(
            env::checks(&s.definition.example, &s.example)
                .iter()
                .all(|r| r.0)
        );
        assert_eq!(
            self.progress.favorites,
            BTreeSet::from([COMPLETE.to_owned()])
        );
    }
    pub(super) fn reopen_environment_tick(&mut self, ctx: &egui::Context, session: &Session) {
        self.reopen_env_private(session);
        assert!(self.smoke_stage >= 17000);
        let plan = steps(session.role);
        let phase = self.smoke_stage - 17000;
        let step = *plan.get(phase).expect("closed Environment reopen phase");
        match step {
            Step::Filter(id) => self.reopen_filter(id),
            Step::Favorite => {
                assert_eq!(self.metrics.card_actions.len(), 1);
                if !self.click_visible_smoke_control(
                    ctx,
                    self.metrics.card_actions.first().map(|r| r.1),
                ) {
                    return;
                }
            }
            Step::Open(id) => {
                if !self.reopen_start_card(ctx, id) {
                    return;
                }
            }
            Step::Try => {
                if !self.click_visible_smoke_control(ctx, self.metrics.example_action) {
                    return;
                }
            }
            Step::Assignment => {
                if !self.click_visible_smoke_control(ctx, self.metrics.assignment_action) {
                    return;
                }
            }
            Step::Tab(index) => {
                if !self
                    .click_visible_smoke_control(ctx, self.metrics.lesson_tabs.get(index).copied())
                {
                    return;
                }
            }
            Step::Focus => {
                if !self.reopen_focus(ctx) {
                    return;
                }
            }
            Step::Type(value) => {
                let source = match value {
                    Text::Draft(text) => text.to_owned(),
                    Text::Worked(id) => self
                        .reopen_env_scenario(id)
                        .teaching
                        .as_ref()
                        .unwrap()
                        .example
                        .clone(),
                    Text::Reference(id) => self.reopen_env_scenario(id).solution[0].clone(),
                };
                assert!(source.len() <= env::MAX_SOURCE);
                self.reopen_type(&source);
            }
            Step::Run => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return;
                }
            }
            Step::Wait(id, example) => {
                if !self.reopen_env_wait(ctx, id, example) {
                    return;
                }
            }
            Step::Back => {
                if !self.click_visible_smoke_control(ctx, self.metrics.back) {
                    return;
                }
            }
            Step::CheckSaved(verified) => self.reopen_env_saved(verified),
            Step::LoadedExact => {
                // The mandatory pre-render hook already compared actual loaded
                // observations before automatic replay could finish.
                let expected: Observed = serde_json::from_slice(
                    &bounded_file(&session.root.join("writer-observed.json"), JSON_CAP).unwrap(),
                )
                .unwrap();
                let loaded: Observed = serde_json::from_slice(
                    &bounded_file(&session.root.join("reader-loaded.json"), JSON_CAP).unwrap(),
                )
                .unwrap();
                assert_eq!(loaded, expected);
                assert_eq!(
                    bounded_file(&self.path, 16_000_000).unwrap(),
                    bounded_file(&session.root.join("writer-progress.raw"), 16_000_000).unwrap()
                );
            }
            Step::ReplayExact => {
                self.reopen_env_saved(true);
                let expected: Observed = serde_json::from_slice(
                    &bounded_file(&session.root.join("writer-observed.json"), JSON_CAP).unwrap(),
                )
                .unwrap();
                let (replayed, _, _) = session.observed(self);
                assert_eq!(
                    replayed, expected,
                    "same source replay preserves every serialized outcome, draft and provenance field"
                );
                let first = &self.progress.attempts[COMPLETE];
                session.write("reader-replayed.json", &serde_json::json!({
                    "observed": replayed,
                    "record_verified": state(first).assignment.result.as_ref().unwrap().verified(),
                    "current_assignment_credit": progress::current_solved(first, self.reopen_env_scenario(COMPLETE)),
                })).unwrap();
            }
            Step::ScaleOpen => {
                if !self.reopen_popup(ctx, self.metrics.reopen_scale) {
                    return;
                }
            }
            Step::ScaleChoice => {
                let rect = self
                    .metrics
                    .reopen_scale_items
                    .iter()
                    .find(|(v, _)| *v == SCALE)
                    .map(|(_, r)| *r);
                if !self.reopen_popup(ctx, rect) {
                    return;
                }
            }
            Step::ThemeOpen => {
                if !self.reopen_popup(ctx, self.metrics.reopen_theme) {
                    return;
                }
            }
            Step::ThemeChoice => {
                let rect = self
                    .metrics
                    .reopen_theme_items
                    .iter()
                    .find(|(id, _)| id == THEME)
                    .map(|(_, r)| *r);
                if !self.reopen_popup(ctx, rect) {
                    return;
                }
            }
            Step::Geometry => {
                if !self.reopen_geometry(ctx) {
                    return;
                }
            }
            Step::Shot(index) => {
                if !self.reopen_geometry(ctx) {
                    return;
                }
                self.capture_pending = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                    CAPTURES[index].to_owned(),
                )));
            }
            Step::Finish => {
                let key = egui::Id::new(("reopen-env-finished", session.role == Role::Write));
                if !ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false) {
                    if session.role == Role::Write {
                        self.reopen_env_saved(true);
                    } else {
                        self.reopen_env_reader_repaired();
                    }
                    assert_eq!(self.theme_id, THEME);
                    assert_eq!(self.scale, SCALE);
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
                        "PASS native reopen {}: Environment actual setup/child runs, separate unfinished drafts and normal persistence",
                        if session.role == Role::Write {
                            "writer"
                        } else {
                            "reader"
                        }
                    );
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
        }
        self.smoke_stage += 1;
        self.started = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn environment_cold_reopen_plan_covers_both_actual_runs_distinct_drafts_and_replay() {
        let w = steps(Role::Write);
        let r = steps(Role::Read);
        assert!(w.len() <= 64 && r.len() <= 64);
        assert_eq!(w.iter().filter(|s| matches!(s, Step::Run)).count(), 2);
        assert_eq!(r.iter().filter(|s| matches!(s, Step::Run)).count(), 2);
        assert!(matches!(r.first(), Some(Step::LoadedExact)));
        assert!(r.iter().any(|s| matches!(s, Step::ReplayExact)));
        assert_ne!(FIRST_EXAMPLE, SECOND_EXAMPLE);
        assert_ne!(SECOND_ASSIGNMENT, SECOND_EXAMPLE);
        let names: std::collections::BTreeSet<_> = CAPTURES.iter().collect();
        assert_eq!(names.len(), 15);
    }
}
