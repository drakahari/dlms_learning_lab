//! Closed NFS cold-reopen fixture. Every model change uses normal native widgets.
use super::*;
use dlms_learning_lab::nfs as core;
const LAB: &str = "lpic2-nfs-reference";
const ASSIGNMENT: &str = "# unfinished assignment exports\nnot-an-export";
const EXAMPLE: &str = "# separate unfinished example\nalso-not-an-export";
pub(super) const CAPTURES: [&str; 12] = [
    "reopen-nfs-write-library",
    "reopen-nfs-write-introduction",
    "reopen-nfs-write-working-example",
    "reopen-nfs-write-example-draft",
    "reopen-nfs-write-current-assignment",
    "reopen-nfs-write-unfinished-assignment",
    "reopen-nfs-write-saved-appearance",
    "reopen-nfs-read-loaded-library",
    "reopen-nfs-read-unfinished-assignment",
    "reopen-nfs-read-unfinished-example",
    "reopen-nfs-read-example-repaired",
    "reopen-nfs-read-assignment-repaired",
];
#[derive(Clone, Debug)]
enum Step {
    Filter,
    Favorite,
    Open,
    Try,
    Tab(usize),
    Action(core::Action, bool),
    Check,
    Practice(bool),
    Assignment(bool),
    Pending,
    LoadExact,
    ScaleOpen,
    ScaleChoice,
    ThemeOpen,
    ThemeChoice,
    Geometry,
    Shot(usize),
    Finish,
}
fn solution(s: &Scenario) -> Vec<core::Action> {
    s.solution
        .iter()
        .map(|line| {
            core::Action::from_json(line.strip_prefix("nfs ").expect("authored codec"))
                .expect("validated authored NFS action")
        })
        .collect()
}
fn edit(s: &Scenario, practice: bool, text: &str) -> core::Action {
    let actions = if practice {
        s.nfs_teaching
            .as_ref()
            .unwrap()
            .example_actions
            .iter()
            .map(|step| step.action.clone())
            .collect()
    } else {
        solution(s)
    };
    let server = actions
        .iter()
        .find_map(|a| match a {
            core::Action::SetExportsDraft { server, .. } => Some(server.clone()),
            _ => None,
        })
        .expect("reference fixture edits exports");
    core::Action::SetExportsDraft {
        server,
        text: text.into(),
    }
}
fn steps(s: &Scenario, role: Role) -> Vec<Step> {
    use Step::*;
    let example = || {
        s.nfs_teaching
            .as_ref()
            .unwrap()
            .example_actions
            .iter()
            .map(|step| Action(step.action.clone(), true))
            .collect::<Vec<_>>()
    };
    let assignment = || {
        solution(s)
            .into_iter()
            .map(|a| Action(a, false))
            .collect::<Vec<_>>()
    };
    match role {
        Role::Write => {
            let mut p = vec![Filter, Shot(0), Favorite, Open, Shot(1), Try];
            p.extend(example());
            p.extend([
                Check,
                Practice(true),
                Assignment(false),
                Shot(2),
                Action(edit(s, true, EXAMPLE), true),
                Check,
                Practice(false),
                Shot(3),
                Tab(2),
            ]);
            p.extend(assignment());
            p.extend([
                Check,
                Assignment(true),
                Shot(4),
                Action(edit(s, false, ASSIGNMENT), false),
                Check,
                Pending,
                Shot(5),
                ScaleOpen,
                ScaleChoice,
                ThemeOpen,
                ThemeChoice,
                Geometry,
                Shot(6),
                Finish,
            ]);
            p
        }
        Role::Read => {
            let mut p = vec![
                LoadExact,
                Shot(7),
                Filter,
                Open,
                Pending,
                Shot(8),
                Tab(1),
                Pending,
                Shot(9),
            ];
            // Live policy/mounts already survived the exit. Repair only the
            // unsaved document; mounting over an occupied mountpoint is an error.
            p.extend(
                example().into_iter().filter(|step| {
                    matches!(step, Action(core::Action::SetExportsDraft { .. }, true))
                }),
            );
            p.extend([Check, Practice(true), Assignment(false), Shot(10), Tab(2)]);
            p.extend(assignment().into_iter().filter(|step| {
                matches!(
                    step,
                    Action(
                        core::Action::SetExportsDraft { .. } | core::Action::ReadExisting { .. },
                        false
                    )
                )
            }));
            p.extend([Check, Practice(true), Assignment(true), Shot(11), Finish]);
            p
        }
    }
}
impl LabApp {
    fn reopen_nfs_pending(&self, s: &Scenario) {
        let a = &self.progress.attempts[LAB];
        let engine::State::Nfs { simulation } = &a.state else {
            panic!("NFS fixture")
        };
        let learning = a.workspace.as_ref().unwrap().nfs_learning.as_ref().unwrap();
        assert_eq!(simulation.runtime().servers[0].draft, ASSIGNMENT);
        assert_eq!(learning.simulation.runtime().servers[0].draft, EXAMPLE);
        assert!(!progress::current_solved(a, s));
        assert!(
            learning
                .checks(s.nfs_teaching.as_ref().unwrap())
                .iter()
                .any(|row| !row.0)
        );
        // Live mounts and applied policy persist even with unfinished editor work.
        assert!(!simulation.runtime().mounts.is_empty());
        assert!(!learning.simulation.runtime().mounts.is_empty());
        assert!(!simulation.runtime().servers[0].applied.is_empty());
        assert!(!learning.simulation.runtime().servers[0].applied.is_empty());
        assert_eq!(self.progress.attempts.len(), 1);
        assert_eq!(
            self.progress.favorites,
            std::collections::BTreeSet::from([LAB.into()])
        );
    }
    pub(super) fn reopen_nfs_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        assert!(self.storage_error.is_none() && self.preferences_error.is_none());
        session.verify_profile_paths(&self.path).unwrap();
        assert_eq!(session.case, Case::Nfs);
        let i = self.scenarios.iter().position(|s| s.id == LAB).unwrap();
        let s = self.scenarios[i].clone();
        assert_eq!(s.version, 1);
        let plan = steps(&s, session.role);
        let phase = self.smoke_stage.checked_sub(17000).unwrap();
        let step = plan.get(phase).expect("closed bounded NFS reopen phase");
        if phase == 0 && !self.reopen_geometry(ctx) {
            return;
        }
        match step {
            Step::Filter => self.reopen_filter(LAB),
            Step::Favorite => {
                if !self.click_visible_smoke_control(
                    ctx,
                    self.metrics.card_actions.first().map(|r| r.1),
                ) {
                    return;
                }
            }
            Step::Open => {
                if !self.reopen_start_card(ctx, LAB) {
                    return;
                }
            }
            Step::Try => {
                if !self.nfs_reopen_try(ctx) {
                    return;
                }
            }
            Step::Tab(tab) => {
                if !self.nfs_reopen_tab(ctx, *tab) {
                    return;
                }
            }
            Step::Action(action, practice) => {
                if !self.nfs_reopen_action(ctx, LAB, action, *practice) {
                    return;
                }
            }
            Step::Check => {
                if !self.nfs_reopen_check(ctx) {
                    return;
                }
            }
            Step::Practice(pass) => {
                let l = self.progress.attempts[LAB]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .nfs_learning
                    .as_ref()
                    .unwrap();
                let rows = l.checks(s.nfs_teaching.as_ref().unwrap());
                assert_eq!(
                    rows.iter().all(|r| r.0),
                    *pass,
                    "independent example outcomes: {rows:?}"
                );
            }
            Step::Assignment(pass) => assert_eq!(
                progress::current_solved(&self.progress.attempts[LAB], &s),
                *pass
            ),
            Step::Pending => self.reopen_nfs_pending(&s),
            Step::LoadExact => {
                let expected: Observed = serde_json::from_slice(
                    &bounded_file(&session.root.join("writer-observed.json"), JSON_CAP).unwrap(),
                )
                .unwrap();
                assert_eq!(
                    bounded_file(&self.path, 16_000_000).unwrap(),
                    bounded_file(&session.root.join("writer-progress.raw"), 16_000_000).unwrap()
                );
                assert_eq!(
                    bounded_file(&self.path.with_file_name("preferences-v1.json"), 4096).unwrap(),
                    bounded_file(&session.root.join("writer-preferences.raw"), 4096).unwrap()
                );
                let (loaded, _, _) = session.observed(self);
                assert_eq!(
                    loaded, expected,
                    "every raw/portable/provenance/preference field must cold-load exactly"
                );
                assert_eq!(self.theme_id, THEME);
                assert_eq!(self.scale, SCALE);
                self.reopen_nfs_pending(&s);
                session.write("reader-loaded.json", &loaded).unwrap();
            }
            Step::ScaleOpen => {
                if !self.reopen_popup(ctx, self.metrics.reopen_scale) {
                    return;
                }
            }
            Step::ScaleChoice => {
                let r = self
                    .metrics
                    .reopen_scale_items
                    .iter()
                    .find(|(v, _)| *v == SCALE)
                    .map(|(_, r)| *r);
                if !self.reopen_popup(ctx, r) {
                    return;
                }
            }
            Step::ThemeOpen => {
                if !self.reopen_popup(ctx, self.metrics.reopen_theme) {
                    return;
                }
            }
            Step::ThemeChoice => {
                let r = self
                    .metrics
                    .reopen_theme_items
                    .iter()
                    .find(|(id, _)| id == THEME)
                    .map(|(_, r)| *r);
                if !self.reopen_popup(ctx, r) {
                    return;
                }
            }
            Step::Geometry => {
                if !self.reopen_geometry(ctx) {
                    return;
                }
            }
            Step::Shot(n) => {
                if !matches!(*n, 0 | 1 | 7)
                    && !self.nfs_reopen_reveal(ctx, matches!(*n, 2 | 4 | 10 | 11))
                {
                    return;
                }
                self.capture_pending = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                    CAPTURES[*n].to_owned(),
                )));
            }
            Step::Finish => {
                let key = egui::Id::new(("nfs-cold-finished", session.role == Role::Write));
                if !ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false) {
                    if session.role == Role::Write {
                        self.reopen_nfs_pending(&s);
                    } else {
                        assert!(progress::current_solved(&self.progress.attempts[LAB], &s));
                        let l = self.progress.attempts[LAB]
                            .workspace
                            .as_ref()
                            .unwrap()
                            .nfs_learning
                            .as_ref()
                            .unwrap();
                        assert!(
                            l.checks(s.nfs_teaching.as_ref().unwrap())
                                .iter()
                                .all(|r| r.0)
                        );
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
                        "PASS native reopen {}: actual NFS work, separate drafts, mounts/files, grading and normal persistence",
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

#[cfg(all(test, target_os = "linux"))]
pub(super) fn finish(s: &Scenario, write: bool) -> usize {
    steps(s, if write { Role::Write } else { Role::Read }).len() - 1 + 17000
}
