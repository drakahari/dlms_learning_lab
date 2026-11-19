//! Fixed cold-load observations and real native mount controls, never injected state.
use super::*;
use dlms_learning_lab::user_mounts_workspace as work;
const COMPLETE: &str = "lpic1-removable-personal-session";
const UNFINISHED: &str = "lpic1-removable-shared-handover";
const FIRST_EXAMPLE: &str = "# Separate gallery notes, unfinished\n";
const SECOND_EXAMPLE: &str = "# Separate shared gallery draft, unfinished\n";
const SECOND_ASSIGNMENT: &str = "# Archive handover draft, unfinished\n";
pub(super) const CAPTURES: [&str; 8] = [
    "reopen-mount-write-complete",
    "reopen-mount-write-independent-draft",
    "reopen-mount-write-second-drafts",
    "reopen-mount-write-saved",
    "reopen-mount-read-loaded",
    "reopen-mount-read-replayed",
    "reopen-mount-read-example-repaired",
    "reopen-mount-read-assignment-repaired",
];
#[derive(Clone)]
enum Step {
    Filter(&'static str),
    Open(&'static str),
    Favorite,
    Try,
    Assignment,
    Tab(usize),
    Focus,
    Text(String),
    Click(String),
    Run,
    Wait(&'static str, bool),
    Back,
    ScaleOpen,
    ScaleChoice,
    ThemeOpen,
    ThemeChoice,
    Loaded,
    Replayed,
    Shot(usize),
    Finish,
}
fn scenario<'a>(labs: &'a [Scenario], id: &str) -> &'a Scenario {
    labs.iter().find(|s| s.id == id).unwrap()
}
fn state(a: &Attempt) -> &work::Session {
    let engine::State::UserMounts { simulation } = &a.state else {
        panic!("fixed mount activity")
    };
    simulation
}
fn actions(steps: &mut Vec<Step>, actions: &[work::Action]) {
    for a in actions {
        match a {
            work::Action::Save { configuration } => steps.extend([
                Step::Focus,
                Step::Text(configuration.clone()),
                Step::Click("save".into()),
            ]),
            work::Action::Mount { actor, target } | work::Action::Unmount { actor, target } => {
                steps.extend([
                    Step::Click("actor".into()),
                    Step::Click(format!("actor:{actor:?}")),
                    Step::Click("target".into()),
                    Step::Click(format!("target:{target}")),
                    Step::Click(
                        if matches!(a, work::Action::Mount { .. }) {
                            "mount"
                        } else {
                            "unmount"
                        }
                        .into(),
                    ),
                ])
            }
        }
    }
}
fn reference(labs: &[Scenario], id: &str) -> Vec<work::Action> {
    scenario(labs, id)
        .solution
        .iter()
        .map(|s| serde_json::from_str(s.strip_prefix(work::PREFIX).unwrap()).unwrap())
        .collect()
}
fn steps(role: Role, labs: &[Scenario]) -> Vec<Step> {
    use Step::*;
    let mut p = vec![];
    if role == Role::Write {
        p.extend([Filter(COMPLETE), Favorite, Open(COMPLETE), Assignment]);
        actions(&mut p, &reference(labs, COMPLETE));
        p.extend([
            Run,
            Wait(COMPLETE, false),
            Shot(0),
            Tab(1),
            Focus,
            Text(FIRST_EXAMPLE.into()),
            Shot(1),
            Back,
            Filter(UNFINISHED),
            Open(UNFINISHED),
            Try,
            Focus,
            Text(SECOND_EXAMPLE.into()),
            Assignment,
            Focus,
            Text(SECOND_ASSIGNMENT.into()),
            Shot(2),
            ScaleOpen,
            ScaleChoice,
            ThemeOpen,
            ThemeChoice,
            Shot(3),
            Finish,
        ]);
    } else {
        p.extend([
            Loaded,
            Shot(4),
            Wait(COMPLETE, false),
            Replayed,
            Filter(COMPLETE),
            Open(COMPLETE),
            Tab(2),
            Shot(5),
            Back,
            Filter(UNFINISHED),
            Open(UNFINISHED),
            Tab(1),
        ]);
        let engine::Goal::UserMounts { definition } = &scenario(labs, UNFINISHED).goal else {
            panic!()
        };
        actions(&mut p, &definition.example_actions);
        p.extend([Run, Wait(UNFINISHED, true), Shot(6), Tab(2)]);
        actions(&mut p, &reference(labs, UNFINISHED));
        p.extend([Run, Wait(UNFINISHED, false), Shot(7), Finish]);
    }
    assert!(p.len() <= 96);
    p
}
impl LabApp {
    fn mount_private(&self, s: &Session) {
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        assert!(self.storage_error.is_none() && self.preferences_error.is_none());
        s.verify_profile_paths(&self.path).unwrap();
        assert_eq!(s.case, Case::UserMounts);
    }
    fn mount_saved(&self, verified: bool) {
        assert_eq!(self.progress.attempts.len(), 2);
        assert_eq!(self.progress.favorites, BTreeSet::from([COMPLETE.into()]));
        let a = &self.progress.attempts[COMPLETE];
        let source = scenario(&self.scenarios, COMPLETE);
        work::context(a, source).unwrap();
        let x = state(a);
        let mut expected = work::Work::new(&x.definition.assignment);
        for action in reference(&self.scenarios, COMPLETE) {
            expected.apply(&x.definition.assignment, action).unwrap();
        }
        assert_eq!(x.assignment.draft, expected.draft);
        assert_eq!(x.assignment.history, expected.history);
        assert_eq!(x.assignment.result.as_ref().unwrap().verified(), verified);
        assert_eq!(progress::current_solved(a, source), verified);
        assert_eq!(x.example.draft, FIRST_EXAMPLE);
        assert!(x.example.history.is_empty() && x.example.result.is_none());
        let b = &self.progress.attempts[UNFINISHED];
        work::context(b, scenario(&self.scenarios, UNFINISHED)).unwrap();
        let y = state(b);
        assert_eq!(y.example.draft, SECOND_EXAMPLE);
        assert_eq!(y.assignment.draft, SECOND_ASSIGNMENT);
        assert!(y.assignment.history.is_empty() && y.example.history.is_empty());
        assert!(y.assignment.result.is_none() && y.example.result.is_none());
        assert!(!progress::current_solved(
            b,
            scenario(&self.scenarios, UNFINISHED)
        ));
    }
    /// Observation after ordinary load, before any cooperative verification frame.
    pub(in crate::ui) fn reopen_user_mounts_loaded(&self, s: &Session) {
        if s.case != Case::UserMounts || s.role != Role::Read {
            return;
        }
        self.mount_private(s);
        self.mount_saved(false);
        assert_eq!(self.theme_id, THEME);
        assert_eq!(self.scale, SCALE);
        let expected: Observed = serde_json::from_slice(
            &bounded_file(&s.root.join("writer-observed.json"), JSON_CAP).unwrap(),
        )
        .unwrap();
        let (loaded, raw, prefs) = s.observed(self);
        assert_eq!(loaded, expected);
        assert_eq!(
            raw,
            bounded_file(&s.root.join("writer-progress.raw"), 16_000_000).unwrap()
        );
        assert_eq!(
            prefs,
            bounded_file(&s.root.join("writer-preferences.raw"), 4096).unwrap()
        );
        s.write("reader-loaded.json", &loaded).unwrap();
        let x = state(&self.progress.attempts[COMPLETE]);
        assert!(x.assignment.needs_verification(&x.definition.assignment));
        s.write("reader-before-replay.json",&serde_json::json!({"activity_revision":work::REVISION,"saved_record_verified":false,"current_assignment_credit":false,"drafts_preserved":true,"note":"Assertions above observe ordinary loaded state before cooperative verification"})).unwrap();
    }
    pub(super) fn reopen_user_mounts_tick(&mut self, ctx: &egui::Context, s: &Session) {
        self.mount_private(s);
        let plan = steps(s.role, &self.scenarios);
        let phase = self.smoke_stage - 17000;
        let op = plan.get(phase).expect("fixed mount reopen phase").clone();
        match op {
            Step::Filter(id) => self.reopen_filter(id),
            Step::Open(id) => {
                if !self.reopen_start_card(ctx, id) {
                    return;
                }
            }
            Step::Favorite => {
                assert_eq!(self.metrics.card_actions.len(), 1);
                if !self.click_visible_smoke_control(
                    ctx,
                    self.metrics.card_actions.first().map(|r| r.1),
                ) {
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
            Step::Tab(n) => {
                if !self.click_visible_smoke_control(ctx, self.metrics.lesson_tabs.get(n).copied())
                {
                    return;
                }
            }
            Step::Focus => {
                if !self.reopen_focus(ctx) {
                    return;
                }
            }
            Step::Text(text) => self.reopen_type(&text),
            Step::Click(key) => {
                let popup = key.starts_with("actor:") || key.starts_with("target:");
                let rect = self
                    .metrics
                    .user_mounts
                    .controls
                    .iter()
                    .find(|r| r.0 == key)
                    .map(|r| r.1);
                if popup {
                    if !self.reopen_popup(ctx, rect) {
                        return;
                    }
                } else if !self.click_visible_smoke_control(ctx, rect) {
                    return;
                }
            }
            Step::Run => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return;
                }
            }
            Step::Wait(id, example) => {
                let x = state(&self.progress.attempts[id]);
                let (t, w) = if example {
                    (&x.definition.example, &x.example)
                } else {
                    (&x.definition.assignment, &x.assignment)
                };
                if !w.solved(t) {
                    assert!(
                        self.user_mounts_run.is_some(),
                        "requested run ended without success: {:?}",
                        w.result
                    );
                    ctx.request_repaint();
                    return;
                }
            }
            Step::Back => {
                if !self.click_visible_smoke_control(ctx, self.metrics.back) {
                    return;
                }
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
                    .find(|r| r.0 == SCALE)
                    .map(|r| r.1);
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
                    .find(|r| r.0 == THEME)
                    .map(|r| r.1);
                if !self.reopen_popup(ctx, rect) {
                    return;
                }
            }
            Step::Loaded => {
                let loaded: Observed = serde_json::from_slice(
                    &bounded_file(&s.root.join("reader-loaded.json"), JSON_CAP).unwrap(),
                )
                .unwrap();
                let expected: Observed = serde_json::from_slice(
                    &bounded_file(&s.root.join("writer-observed.json"), JSON_CAP).unwrap(),
                )
                .unwrap();
                assert_eq!(loaded, expected);
            }
            Step::Replayed => {
                self.mount_saved(true);
                let expected: Observed = serde_json::from_slice(
                    &bounded_file(&s.root.join("writer-observed.json"), JSON_CAP).unwrap(),
                )
                .unwrap();
                let (observed, _, _) = s.observed(self);
                assert_eq!(observed, expected);
                s.write("reader-replayed.json",&serde_json::json!({"observed":observed,"record_verified":state(&self.progress.attempts[COMPLETE]).assignment.result.as_ref().unwrap().verified(),"current_assignment_credit":progress::current_solved(&self.progress.attempts[COMPLETE],scenario(&self.scenarios,COMPLETE))})).unwrap();
            }
            Step::Shot(n) => {
                if !self.reopen_geometry(ctx) {
                    return;
                }
                self.capture_pending = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                    CAPTURES[n].to_owned(),
                )));
            }
            Step::Finish => {
                let key = egui::Id::new(("mount-reopen-finished", s.role == Role::Write));
                if !ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false) {
                    if s.role == Role::Write {
                        self.mount_saved(true);
                    } else {
                        assert!(progress::current_solved(
                            &self.progress.attempts[COMPLETE],
                            scenario(&self.scenarios, COMPLETE)
                        ));
                        assert_eq!(
                            state(&self.progress.attempts[COMPLETE]).example.draft,
                            FIRST_EXAMPLE
                        );
                        let x = state(&self.progress.attempts[UNFINISHED]);
                        assert!(
                            x.example.solved(&x.definition.example)
                                && x.assignment.solved(&x.definition.assignment)
                        );
                    }
                    assert_eq!(self.theme_id, THEME);
                    assert_eq!(self.scale, SCALE);
                    self.save();
                    assert!(!self.dirty && self.storage_error.is_none());
                    let (observed, raw, prefs) = s.observed(self);
                    let (a, b, c) = if s.role == Role::Write {
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
                    s.write_raw(a, &raw, 16_000_000).unwrap();
                    s.write_raw(b, &prefs, 4096).unwrap();
                    s.write(c, &observed).unwrap();
                    ctx.data_mut(|d| d.insert_temp(key, true));
                    println!(
                        "PASS native reopen {}: actual caller-aware mounts, independent drafts and normal saved history",
                        if s.role == Role::Write {
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
    fn user_mount_cold_plan_uses_native_actions_and_independent_drafts() {
        let ctx = egui::Context::default();
        let app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let labs = &app.scenarios;
        let w = steps(Role::Write, labs);
        let r = steps(Role::Read, labs);
        assert!(w.len() <= 96 && r.len() <= 96);
        assert!(matches!(r.first(), Some(Step::Loaded)));
        assert!(r.iter().any(|r| matches!(r, Step::Replayed)));
        assert_eq!(w.iter().filter(|s| matches!(s, Step::Run)).count(), 1);
        assert_eq!(r.iter().filter(|s| matches!(s, Step::Run)).count(), 2);
        assert_ne!(FIRST_EXAMPLE, SECOND_EXAMPLE);
        assert_ne!(SECOND_EXAMPLE, SECOND_ASSIGNMENT);
        assert_eq!(CAPTURES.iter().collect::<BTreeSet<_>>().len(), 8);
    }
}
