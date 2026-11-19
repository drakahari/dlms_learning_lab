//! Fixed native administration capstone; every mutation uses the actual controls.
use super::mailbox::{Op, translate};
use super::*;
use dlms_learning_lab::{mailbox_source as binding, mailbox_workspace as work};
const ID: &str = "lpic2-mailbox-administration-recovery";
fn run(p: &mut Vec<Op>, text: &str) {
    p.extend([
        Op::Text("command".into(), text.into()),
        Op::Click("run-admin".into()),
    ]);
}
fn plan() -> &'static Vec<Op> {
    static P: std::sync::OnceLock<Vec<Op>> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        let labs = pack::current().unwrap();
        let source = labs.iter().find(|s| s.id == ID).unwrap();
        let d = binding::source(source).unwrap();
        let mut p = vec![
            Op::Style(1., false),
            Op::Open(ID.into()),
            Op::Shot("mail-admin-introduction".into(), "introduction".into()),
            Op::Shot("mail-admin-worked".into(), "worked".into()),
            Op::Click("example".into()),
            Op::Shot("mail-admin-practice".into(), "editor-viewport".into()),
            Op::Click("check".into()),
            Op::Check(false, true),
            Op::Click("fill".into()),
            Op::NotRun(true),
            Op::Click("insert-command".into()),
            Op::NotRun(true),
            Op::Click("run-admin".into()),
            Op::OutputContains("/srv/practice/elin/Maildir\n".into(), true),
            Op::Shot(
                "mail-admin-saved-before-apply".into(),
                "output-viewport".into(),
            ),
        ];
        let worked: Vec<work::Action> = serde_json::from_str(include_str!(
            "../../../tests/fixtures/mailbox-administration-worked-v1.json"
        ))
        .unwrap();
        for a in &worked {
            translate(&mut p, a);
        }
        p.extend([
            Op::Check(true, true),
            Op::Uncredited,
            Op::Shot("mail-admin-practice-complete".into(), "result".into()),
        ]);
        run(
            &mut p,
            "doveadm search -u elin mailbox INBOX subject '[DRILL]'",
        );
        p.push(Op::OutputContains(" 44\n".into(), true));
        p.extend([
            Op::Click("check".into()),
            Op::Check(true, true),
            Op::Shot(
                "mail-admin-counterexample-preview".into(),
                "output-viewport".into(),
            ),
            Op::Click("assignment".into()),
            Op::Shot("mail-admin-assignment".into(), "assignment".into()),
            Op::Shot(
                "mail-admin-assignment-workspace".into(),
                "editor-viewport".into(),
            ),
            Op::Click("check".into()),
            Op::Check(false, false),
            Op::Shot("mail-admin-partial-feedback".into(), "failed-case".into()),
        ]);
        run(&mut p, "doveconf -n");
        p.extend([
            Op::OutputContains("[Unsupported]".into(), false),
            Op::Shot("mail-admin-unsupported".into(), "output-viewport".into()),
        ]);
        let actions = source
            .solution
            .iter()
            .map(|l| {
                serde_json::from_str::<work::Action>(l.strip_prefix(binding::PREFIX).unwrap())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        for a in &actions {
            translate(&mut p, a);
        }
        p.extend([
            Op::Check(true, false),
            Op::Save,
            Op::Shot("mail-admin-complete".into(), "result".into()),
            Op::Text("editor".into(), "unknown_setting = yes".into()),
            Op::NotRun(false),
            Op::Click("check".into()),
            Op::Check(false, false),
            Op::Shot(
                "mail-admin-stale-success-cleared".into(),
                "failed-case".into(),
            ),
        ]);
        let text = actions
            .iter()
            .find_map(|a| {
                if let work::Action::Edit { text } = a {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .unwrap();
        for a in [
            work::Action::Edit { text },
            work::Action::Save,
            work::Action::Restart,
            work::Action::Enable,
            work::Action::Check,
        ] {
            translate(&mut p, &a);
        }
        p.extend([Op::Check(true, false), Op::Save]);
        for (scale, light, name) in [(1.5, true, "light150"), (2., false, "navy200")] {
            p.extend([Op::Style(scale, light), Op::Narrow]);
            for (anchor, label) in [
                ("editor-viewport", "editor"),
                ("save", "save"),
                ("command", "input"),
                ("run-admin", "run"),
                ("result", "result"),
            ] {
                p.push(Op::Shot(
                    format!("mail-admin-{name}-{label}"),
                    anchor.into(),
                ));
            }
        }
        p.push(Op::Style(1., false));
        run(&mut p, "doveadm expunge -u tess mailbox INBOX all");
        p.extend([
            Op::Click("check".into()),
            Op::Check(false, false),
            Op::Shot(
                "mail-admin-protected-mail-loss".into(),
                "failed-case".into(),
            ),
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Fresh,
        ]);
        assert_eq!(d.source_id, ID);
        p
    })
}
impl LabApp {
    pub(in crate::ui) fn mailbox_admin_tick(&mut self, ctx: &egui::Context) {
        self.mailbox_flow(ctx, plan());
    }
}
#[test]
fn admin_native_plan_retains_complete_assignment_separate_practice_and_real_failure_paths() {
    let p = plan();
    assert_eq!(p.iter().filter(|o| matches!(o, Op::Open(_))).count(), 1);
    assert!(matches!(p.last(), Some(Op::Fresh)));
    for name in [
        "mail-admin-introduction",
        "mail-admin-practice",
        "mail-admin-assignment",
        "mail-admin-unsupported",
        "mail-admin-protected-mail-loss",
        "mail-admin-navy200-run",
    ] {
        assert!(p.iter().any(|o| matches!(o,Op::Shot(n,_)if n==name)));
    }
    assert!(p.iter().any(|o| matches!(o, Op::Uncredited)));
    assert!(p.iter().any(|o| matches!(o, Op::Check(false, false))));
    assert!(p.iter().any(|o| matches!(o, Op::Check(true, false))));
}
