//! Package compatibility teaching through actual native controls; not a novice study.
use super::debian_contents::{check_work, command, find, painted, point_click, run};
use super::*;
use dlms_learning_lab::{debian_packages_terminal::Session, script_teaching::View};
const IDS: [&str; 2] = [
    "lpic1-package-conflict-retirement",
    "lpic1-package-conflict-upgrade",
];
fn opened(id: &str, v: View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a, i) = super::debug_recovery::opened(id);
    a.set_debian_packages_view(i, v);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn terminal<'a>(a: &'a LabApp, id: &str, practice: bool) -> &'a Session {
    let w = a.progress.attempts[id].workspace.as_ref().unwrap();
    if practice {
        w.debian_packages_learning
            .as_ref()
            .unwrap()
            .terminal
            .as_ref()
            .unwrap()
    } else {
        w.debian_packages_terminal.as_ref().unwrap()
    }
}
#[test]
fn conflicts_three_views_teach_metadata_and_hide_independent_recipes() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id, View::Introduction);
        let s = a.scenarios[i].clone();
        let text = painted(&ctx, &mut a);
        assert!(text.contains("Conflicts"));
        assert!(text.contains("Worked example"));
        assert!(text.contains("database records"));
        let worked = if id.ends_with("retirement") {
            "gallery-delivery"
        } else {
            "studio-dashboard"
        };
        assert!(text.contains(worked));
        a.set_debian_packages_view(i, View::Example);
        let text = painted(&ctx, &mut a);
        assert!(text.contains("Investigate the unfinished sample"));
        assert!(text.contains("Actual") || text.contains("actual"));
        assert!(a.metrics.package_command.is_some());
        a.set_debian_packages_view(i, View::Assignment);
        let text = painted(&ctx, &mut a);
        assert!(text.contains("Requirements"));
        assert!(!text.contains(worked));
        for recipe in &s.solution {
            assert!(
                !text.contains(recipe),
                "solution exposed by default: {recipe}"
            );
        }
        assert!(terminal(&a, id, false).command.is_empty());
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        a.set_debian_packages_view(i, View::Introduction);
        let text = painted(&ctx, &mut a);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(!text.contains(worked));
        assert!(a.metrics.package_command.is_some());
    }
}
#[test]
fn conflicts_native_commands_refuse_atomically_then_grade_partial_correct_broken_and_resume() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id, View::Assignment);
        let s = a.scenarios[i].clone();
        let target = if id.ends_with("retirement") {
            "ops-delivery"
        } else {
            "branch-dashboard"
        };
        let original = a.progress.attempts[id].state.clone();
        run(&ctx, &mut a, &format!("apt-cache show {target}"));
        assert_eq!(a.progress.attempts[id].state, original);
        assert!(
            terminal(&a, id, false)
                .entries
                .last()
                .unwrap()
                .stdout
                .contains("Conflicts:")
        );
        run(&ctx, &mut a, &format!("apt-get -y install {target}"));
        assert_eq!(a.progress.attempts[id].state, original);
        assert_eq!(terminal(&a, id, false).entries.last().unwrap().status, 100);
        let feedback = painted(&ctx, &mut a);
        assert!(feedback.contains("conflicts with installed"));
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        run(&ctx, &mut a, &s.solution[0]);
        check_work(&ctx, &mut a);
        assert!(!a.metrics.completion);
        run(&ctx, &mut a, &s.solution[1]);
        check_work(&ctx, &mut a);
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        a.save();
        assert!(a.storage_error.is_none());
        let p = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&p.attempts[id]).unwrap(),
            serde_json::to_value(&a.progress.attempts[id]).unwrap()
        );
        run(&ctx, &mut a, &format!("dpkg --remove {target}"));
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        run(&ctx, &mut a, &format!("apt-get -y install {target}"));
        check_work(&ctx, &mut a);
        assert!(a.metrics.completion);
    }
}
#[test]
fn conflicts_insert_and_real_practice_preserve_assignment_draft_then_keyboard_and_scaling() {
    let id = IDS[0];
    let (ctx, mut a, i) = opened(id, View::Assignment);
    let original = a.progress.attempts[id].state.clone();
    command(&ctx, &mut a, 1200., 850., "apt-cache show ops-delivery");
    a.set_debian_packages_view(i, View::Example);
    draw(&ctx, &mut a);
    let r = find(&ctx, &mut a, 1200., 850., |m| m.command_insert);
    point_click(&ctx, &mut a, 1200., 850., r);
    assert_eq!(a.progress.attempts[id].state, original);
    assert_eq!(
        terminal(&a, id, true).command,
        "dpkg --remove gallery-legacy"
    );
    let steps = a.scenarios[i]
        .debian_packages_teaching
        .as_ref()
        .unwrap()
        .example_actions
        .clone();
    for step in &steps {
        run(&ctx, &mut a, step.command.as_ref().unwrap());
    }
    check_work(&ctx, &mut a);
    assert!(!progress::current_solved(
        &a.progress.attempts[id],
        &a.scenarios[i]
    ));
    a.set_debian_packages_view(i, View::Assignment);
    draw(&ctx, &mut a);
    assert_eq!(
        terminal(&a, id, false).command,
        "apt-cache show ops-delivery"
    );
    command(&ctx, &mut a, 1200., 850., "apt-cache show ops-delivery");
    let input = a.metrics.package_command.unwrap().1;
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    draw(&ctx, &mut a);
    assert!(ctx.memory(|m| m.focused()).is_some_and(|id| id != input));
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, true)]);
    draw(&ctx, &mut a);
    assert_eq!(ctx.memory(|m| m.focused()), Some(input));
    for scale in [1., 1.5, 2.] {
        for width in [760., 1500.] {
            a.scale = scale;
            for _ in 0..4 {
                frame(&ctx, &mut a, width, 5000., vec![]);
            }
            for r in [
                a.metrics.package_command.unwrap().0,
                a.metrics.package_run.unwrap().0,
                a.metrics.check.unwrap(),
            ] {
                assert!(r.is_positive());
                assert!(
                    r.right() <= ctx.content_rect().right() + 2.,
                    "control clipped at {scale}/{width}: {r:?}"
                );
            }
        }
    }
}
