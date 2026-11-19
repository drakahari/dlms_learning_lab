//! One fixed ordinary-profile cold load. Repairs enter the same native widgets.
use super::*;
use crate::ui::smoke::samba_shares::{Op, repair};
use dlms_learning_lab::samba_share_source as binding;
const ID: &str = "lpic2-smb-change-handover";
const ASSIGNMENT: &str = "# unfinished independent policy\n[team]\npath = ";
const EXAMPLE: &str = "# separate unfinished guide policy\n[notes]\npath = ";
pub(super) const CAPTURES: [&str; 4] = [
    "reopen-samba-writer-draft",
    "reopen-samba-reader-assignment",
    "reopen-samba-reader-example",
    "reopen-samba-reader-repaired",
];
fn plan(read: bool) -> &'static Vec<Op> {
    static W: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    static R: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    (if read { &R } else { &W }).get_or_init(|| {
        let s = pack::load(include_str!(
            "../../../../scenarios/samba-standalone-lessons-v1.json"
        ))
        .unwrap()
        .into_iter()
        .find(|s| s.id == ID)
        .unwrap();
        let d = binding::source(&s).unwrap();
        let mut p = vec![Op::Open(ID.into())];
        if read {
            p.push(Op::Shot(CAPTURES[1].into(), "config".into()));
            p.extend([
                Op::Text(
                    "config".into(),
                    d.assignment.reference.saved_config().unwrap().into(),
                ),
                Op::Click("save".into()),
                Op::Click("apply".into()),
                Op::Click("check".into()),
                Op::Check(true, false),
                Op::Click("example".into()),
                Op::Shot(CAPTURES[2].into(), "config".into()),
                Op::Text(
                    "config".into(),
                    d.example.reference.saved_config().unwrap().into(),
                ),
                Op::Click("save".into()),
                Op::Click("apply".into()),
                Op::Click("check".into()),
                Op::Check(true, true),
                Op::Shot(CAPTURES[3].into(), "result-title".into()),
            ]);
        } else {
            p.push(Op::ColdAppearance);
            p.push(Op::Click("example".into()));
            repair(&mut p, &d.example, true);
            p.extend([
                Op::Click("check".into()),
                Op::Check(true, true),
                Op::Uncredited,
                Op::Text("config".into(), EXAMPLE.into()),
                Op::Check(false, true),
                Op::Click("assignment".into()),
            ]);
            repair(&mut p, &d.assignment, true);
            p.extend([
                Op::Click("check".into()),
                Op::Check(true, false),
                Op::Text("terminal".into(), "ls -ld /srv/live-team".into()),
                Op::Text("config".into(), ASSIGNMENT.into()),
                Op::Check(false, false),
                Op::Shot(CAPTURES[0].into(), "config".into()),
            ]);
        }
        p.push(Op::ColdFinish);
        p
    })
}
impl LabApp {
    pub(in crate::ui) fn samba_reopen_appearance(&mut self, ctx: &egui::Context) -> bool {
        let key = egui::Id::new("samba-cold-appearance-controls");
        let phase = ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0);
        if phase == 4 {
            assert_eq!(self.scale, 1.5);
            assert_eq!(self.theme_id, THEME);
            assert!(self.preferences_error.is_none());
            return true;
        }
        let target = match phase {
            0 => self.metrics.reopen_scale,
            1 => self
                .metrics
                .reopen_scale_items
                .iter()
                .find(|(v, _)| *v == 1.5)
                .map(|(_, r)| *r),
            2 => self.metrics.reopen_theme,
            3 => self
                .metrics
                .reopen_theme_items
                .iter()
                .find(|(id, _)| id == THEME)
                .map(|(_, r)| *r),
            _ => panic!("closed toolbar appearance phase"),
        };
        if self.reopen_popup(ctx, target) {
            ctx.data_mut(|d| d.insert_temp(key, phase + 1));
        }
        false
    }
    pub(super) fn reopen_samba_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::Samba);
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        session.verify_profile_paths(&self.path).unwrap();
        if session.role == Role::Read {
            let key = egui::Id::new("samba-normal-load-observed");
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
                let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
                let a = &self.progress.attempts[ID];
                binding::context(a, source).unwrap();
                let engine::State::SambaShare { simulation } = &a.state else {
                    panic!()
                };
                assert_eq!(simulation.assignment.editor, ASSIGNMENT);
                assert_eq!(simulation.practice.editor, EXAMPLE);
                assert_eq!(simulation.assignment.terminal, "ls -ld /srv/live-team");
                assert!(!progress::current_solved(a, source));
                assert!(
                    simulation.assignment.last_trial.is_none()
                        && simulation.practice.last_trial.is_none()
                );
                assert!(
                    simulation.assignment.output.is_empty()
                        && simulation.practice.output.is_empty()
                );
                assert!(
                    simulation
                        .assignment
                        .world
                        .accounts
                        .iter()
                        .all(|u| u.enabled)
                );
                assert_eq!(
                    simulation.assignment.world.files.nodes["/srv/live-team"].mode,
                    0o750
                );
                assert_eq!(
                    simulation.assignment.world.saved_config().unwrap(),
                    binding::source(source)
                        .unwrap()
                        .assignment
                        .reference
                        .saved_config()
                        .unwrap()
                );
                assert_eq!(self.scale, 1.5);
                assert!(
                    theme::inventory()
                        .iter()
                        .find(|t| t.id == self.theme_id)
                        .unwrap()
                        .light
                );
                session.write("reader-loaded.json", &loaded).unwrap();
                ctx.data_mut(|d| d.insert_temp(key, true));
            }
        }
        self.samba_flow(ctx, plan(session.role == Role::Read));
    }
    pub(in crate::ui) fn finish_samba_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::Samba);
        let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
        let a = &self.progress.attempts[ID];
        let engine::State::SambaShare { simulation } = &a.state else {
            panic!()
        };
        let d = binding::source(source).unwrap();
        if session.role == Role::Write {
            assert_eq!(simulation.assignment.editor, ASSIGNMENT);
            assert_eq!(simulation.practice.editor, EXAMPLE);
            assert!(!progress::current_solved(a, source));
        } else {
            assert!(progress::current_solved(a, source));
            assert!(simulation.practice.solved(&d.example));
        }
        let key = egui::Id::new("samba-cold-finished");
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
                "PASS native reopen {}: actual standalone-sharing accounts, modes, saved/applied settings and independent drafts",
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
fn cold_sharing_uses_real_appearance_controls_and_both_independent_drafts() {
    let writer = plan(false);
    assert!(writer.iter().any(|o| matches!(o, Op::ColdAppearance)));
    assert!(!writer.iter().any(|o| matches!(o, Op::Style(..))));
    assert!(
        writer
            .iter()
            .any(|o| matches!(o,Op::Text(key,text)if key=="config"&&text==ASSIGNMENT))
    );
    assert!(
        writer
            .iter()
            .any(|o| matches!(o,Op::Text(key,text)if key=="config"&&text==EXAMPLE))
    );
    assert!(matches!(writer.last(), Some(Op::ColdFinish)));
    assert!(matches!(plan(true).last(), Some(Op::ColdFinish)));
}
