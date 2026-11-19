//! Fixed two-process normal-profile check. Native controls create every observation.
use super::*;
use crate::ui::smoke::mailbox::Op;
use dlms_learning_lab::{mailbox_source as binding, mailbox_workspace as work};
const ID: &str = "lpic2-pop-transaction-recovery";
pub(super) const CAPTURES: [&str; 6] = [
    "reopen-mailbox-writer-example",
    "reopen-mailbox-writer-assignment",
    "reopen-mailbox-reader-assignment",
    "reopen-mailbox-reader-example",
    "reopen-mailbox-repaired-assignment",
    "reopen-mailbox-repaired-example",
];
fn config(s: &pack::Scenario, d: &work::Definition, practice: bool) -> String {
    if practice {
        return d.worked_config.clone().unwrap();
    }
    s.solution
        .iter()
        .find_map(|line| {
            match serde_json::from_str::<work::Action>(line.strip_prefix(binding::PREFIX)?).unwrap()
            {
                work::Action::Edit { text } => Some(text),
                _ => None,
            }
        })
        .unwrap()
}
fn draft(s: &pack::Scenario, d: &work::Definition, practice: bool) -> String {
    format!(
        "# {} pending transaction: finish QUIT before restarting\n{}",
        if practice { "Guided" } else { "Assignment" },
        config(s, d, practice)
    )
}
fn target(t: &work::Task) -> (&str, u32, usize) {
    let (user, uid) = t
        .requirements
        .iter()
        .find_map(|r| match r {
            work::Requirement::PopCommitted { user, uids } => {
                Some((user.as_str(), *uids.first().unwrap()))
            }
            _ => None,
        })
        .unwrap();
    let position = t.facts.mailboxes[user]["INBOX"]
        .messages
        .iter()
        .position(|m| m.uid == uid)
        .unwrap()
        + 1;
    (user, uid, position)
}
fn verify_partial(a: &LabApp) {
    let source = a.scenarios.iter().find(|s| s.id == ID).unwrap();
    let attempt = &a.progress.attempts[ID];
    binding::context(attempt, source).unwrap();
    let d = binding::source(source).unwrap();
    let engine::State::MailboxAccess { simulation } = &attempt.state else {
        panic!()
    };
    for (scope, t, practice) in [
        (&simulation.assignment, &d.assignment, false),
        (&simulation.practice, &d.example, true),
    ] {
        let (user, uid, position) = target(t);
        assert_eq!(scope.draft, draft(source, d, practice));
        assert_eq!(scope.runtime.saved, config(source, d, practice));
        assert_eq!(scope.input, "QUIT");
        assert_eq!(scope.client, if practice { "pop-tls" } else { "pop-plain" });
        assert!(!scope.checked);
        assert!(scope.runtime.pop_committed.is_empty());
        assert!(
            scope.runtime.store[user]["INBOX"]
                .messages
                .iter()
                .any(|m| m.uid == uid)
        );
        let connection = scope.connection.as_ref().unwrap();
        assert!(connection.link.tls);
        let dlms_learning_lab::mailbox_protocol::Phase::Pop {
            user: signed_in,
            deleted,
            ..
        } = &connection.phase
        else {
            panic!()
        };
        assert_eq!(signed_in.as_deref(), Some(user));
        assert_eq!(deleted, &std::collections::BTreeSet::from([uid]));
        assert!(scope.output.contains(&format!("C: DELE {position}")));
        assert!(scope.checks(t).iter().any(|r| !r.passed));
    }
    assert!(!progress::current_solved(attempt, source));
}
fn command(p: &mut Vec<Op>, text: String) {
    p.extend([Op::Text("command".into(), text), Op::Click("send".into())]);
}
fn plan(read: bool) -> &'static Vec<Op> {
    static W: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    static R: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    (if read { &R } else { &W }).get_or_init(|| {
        let labs = pack::current().unwrap();
        let source = labs.iter().find(|s| s.id == ID).unwrap();
        let d = binding::source(source).unwrap();
        let mut p = vec![];
        if !read {
            p.push(Op::ColdAppearance);
        }
        p.push(Op::Open(ID.into()));
        let scopes = if read {
            [(false, &d.assignment), (true, &d.example)]
        } else {
            [(true, &d.example), (false, &d.assignment)]
        };
        for (practice, t) in scopes {
            p.push(Op::Click(
                if practice { "example" } else { "assignment" }.into(),
            ));
            if read {
                p.push(Op::Shot(
                    CAPTURES[if practice { 3 } else { 2 }].into(),
                    "command".into(),
                ));
                // Use the restored input and authenticated transaction, before any restart.
                p.push(Op::Click("send".into()));
                p.extend([
                    Op::Click("save".into()),
                    Op::Click("restart".into()),
                    Op::Click("check".into()),
                    Op::Check(true, practice),
                    Op::Shot(
                        CAPTURES[if practice { 5 } else { 4 }].into(),
                        "result".into(),
                    ),
                ]);
            } else {
                p.extend([
                    Op::Text("editor".into(), config(source, d, practice)),
                    Op::Click("save".into()),
                    Op::Click("restart".into()),
                    Op::Click("enable".into()),
                    Op::Choose(
                        "client".into(),
                        if practice { "pop-tls" } else { "pop-plain" }.into(),
                    ),
                    Op::Click("connect".into()),
                ]);
                if !practice {
                    command(&mut p, "STLS".into());
                }
                let (user, _, position) = target(t);
                for line in [
                    format!("USER {user}"),
                    format!("PASS {}", t.facts.accounts[user].sample_password),
                    format!("DELE {position}"),
                ] {
                    command(&mut p, line);
                }
                p.extend([
                    Op::Click("check".into()),
                    Op::Check(false, practice),
                    Op::Text("command".into(), "QUIT".into()),
                    Op::Text("editor".into(), draft(source, d, practice)),
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
    pub(super) fn reopen_mailbox_tick(&mut self, ctx: &egui::Context, session: &Session) {
        assert_eq!(session.case, Case::MailboxAccess);
        assert!(!self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        session.verify_profile_paths(&self.path).unwrap();
        if session.role == Role::Read {
            let key = egui::Id::new("mailbox-normal-load-observed");
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
        self.mailbox_flow(ctx, plan(session.role == Role::Read));
    }
    pub(in crate::ui) fn finish_mailbox_reopen(&mut self, ctx: &egui::Context) {
        let session = self.reopen.as_ref().unwrap().clone();
        assert_eq!(session.case, Case::MailboxAccess);
        if session.role == Role::Write {
            verify_partial(self);
        } else {
            let source = self.scenarios.iter().find(|s| s.id == ID).unwrap();
            let a = &self.progress.attempts[ID];
            let d = binding::source(source).unwrap();
            let engine::State::MailboxAccess { simulation } = &a.state else {
                panic!()
            };
            assert!(progress::current_solved(a, source));
            assert!(
                simulation
                    .practice
                    .checks(&d.example)
                    .iter()
                    .all(|r| r.passed)
            );
            for (scope, t) in [
                (&simulation.assignment, &d.assignment),
                (&simulation.practice, &d.example),
            ] {
                let (user, uid, _) = target(t);
                assert!(scope.runtime.pop_committed[user].contains(&uid));
                assert!(
                    !scope.runtime.store[user]["INBOX"]
                        .messages
                        .iter()
                        .any(|m| m.uid == uid)
                );
            }
        }
        let key = egui::Id::new("mailbox-cold-finished");
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
                "PASS native reopen {}: independent drafts and pending POP transactions preserved before native QUIT commit",
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
fn mailbox_cold_plan_preserves_pending_transactions_and_commits_only_after_actual_reopen() {
    for read in [false, true] {
        let p = plan(read);
        assert!(!p.iter().any(|o| matches!(o, Op::Style(..))));
        assert!(matches!(p.last(), Some(Op::ColdFinish)));
        assert_eq!(
            p.iter().filter(|o| matches!(o, Op::ColdAppearance)).count(),
            usize::from(!read)
        );
    }
    let p = plan(false);
    let drafts = p
        .iter()
        .filter_map(|o| match o {
            Op::Text(k, t) if k == "editor" && t.contains("pending transaction") => Some(t),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(drafts.len(), 2);
    assert_ne!(drafts[0], drafts[1]);
    for text in ["DELE 1", "DELE 2", "STLS"] {
        assert!(p.iter().any(|o| matches!(o,Op::Text(_,t)if t==text)));
    }
    let p = plan(true);
    assert_eq!(
        p.iter()
            .filter(|o| matches!(o,Op::Click(k)if k=="send"))
            .count(),
        2
    );
    assert!(
        !p.iter().any(|o| matches!(o, Op::Text(..) | Op::Choose(..))
            || matches!(o,Op::Click(k)if k=="connect"))
    );
}
