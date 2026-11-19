use super::*;

fn rendered(ctx: &egui::Context, a: &mut LabApp) -> String {
    fn collect(shape: &egui::Shape, text: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                text.push_str(&t.galley.job.text);
                text.push('\n');
            }
            egui::Shape::Vec(shapes) => {
                for s in shapes {
                    collect(s, text);
                }
            }
            _ => {}
        }
    }
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200., 5000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut text = String::new();
    for shape in output.shapes {
        collect(&shape.shape, &mut text);
    }
    text
}

#[test]
fn rollout_bash_every_lab_has_individually_reviewed_requirements_without_losing_conditions() {
    let scenarios = pack::bundled().unwrap();
    let bash: Vec<_> = scenarios
        .iter()
        .filter(|s| s.track == "Bash/Shell")
        .collect();
    assert_eq!(bash.len(), 64);
    let raw: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../scenarios/presentation/bash.json")).unwrap();
    assert_eq!(raw.len(), bash.len() + 12);
    assert_eq!(
        raw.iter()
            .filter(|r| r["source"] == "scenarios/shell-content-backup-lessons-v1.json")
            .count(),
        2
    );
    assert_eq!(
        raw.iter()
            .filter(|r| r["source"] == "scenarios/shell-file-lifecycle-lessons-v1.json")
            .count(),
        3
    );
    assert_eq!(
        raw.iter()
            .filter(|r| r["source"] == "scenarios/shell-tracing-lessons-v1.json")
            .count(),
        2
    );
    assert_eq!(
        raw.iter()
            .filter(|r| r["source"] == "scenarios/shell-debug-recovery-v1.json")
            .count(),
        2
    );
    assert_eq!(
        raw.iter()
            .filter(|r| r["source"] == "scenarios/shell-associative-lessons-v1.json")
            .count(),
        2
    );
    assert_eq!(
        raw.iter()
            .filter(|r| r["source"] == "scenarios/shell-managed-config-lessons-v1.json")
            .count(),
        1
    );
    let normalized = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    for s in bash {
        let entry = lesson_presentation::entry(s).expect(&s.id);
        assert_eq!(
            normalized(&entry.requirements.join(" ")),
            normalized(&s.objective),
            "changed requirements: {}",
            s.id
        );
        assert_eq!(
            entry.practice.len(),
            2,
            "{} needs focused run/change guidance",
            s.id
        );
        assert!(
            entry
                .practice
                .iter()
                .all(|p| !p.title.is_empty() && !p.action.is_empty() && !p.observe.is_empty())
        );
        let reviewed = raw.iter().find(|row| row["id"] == s.id).unwrap();
        assert_eq!(reviewed["status"], "updated");
        for view in ["introduction", "example", "assignment"] {
            assert!(
                reviewed["review"][view].as_str().unwrap().len() > 80,
                "missing content review for {} / {view}",
                s.id
            );
        }
    }
}

#[test]
fn rollout_bash_all_three_views_render_their_actual_lesson_content_and_keep_assignment_independent()
{
    let (ctx, mut a) = app();
    let indices: Vec<_> = a
        .scenarios
        .iter()
        .enumerate()
        .filter_map(|(i, s)| (s.track == "Bash/Shell").then_some(i))
        .collect();
    assert_eq!(indices.len(), 64);
    for i in indices {
        a.launch(i);
        let s = a.scenarios[i].clone();
        let state = a.progress.attempts[&s.id].state.clone();
        let lesson = s.teaching.as_ref().unwrap();
        a.set_script_view(i, teaching::View::Introduction);
        let introduction = rendered(&ctx, &mut a);
        assert!(
            introduction.contains("WHAT YOU’LL LEARN"),
            "{} missing objective",
            s.id
        );
        assert!(
            introduction.contains(&lesson.concepts[0].title),
            "{} concept missing",
            s.id
        );
        assert!(
            introduction.contains("Worked example"),
            "{} worked example absent",
            s.id
        );
        assert!(
            introduction.contains("Expected preview · original example"),
            "{} expected/actual ambiguity",
            s.id
        );
        a.set_script_view(i, teaching::View::Example);
        let practice = rendered(&ctx, &mut a);
        assert!(
            practice.contains(&lesson_presentation::entry(&s).unwrap().practice[0].title),
            "{} missing specific practice action",
            s.id
        );
        assert!(
            practice.contains("Actual result"),
            "{} actual output absent",
            s.id
        );
        assert!(
            practice.contains("Not run."),
            "{} needs an honest unchecked state",
            s.id
        );
        assert!(
            !practice
                .contains("Practice example · Not graded; your assignment draft stays separate.")
        );
        a.set_script_view(i, teaching::View::Assignment);
        let assignment = rendered(&ctx, &mut a);
        assert!(
            assignment.contains("Requirements"),
            "{} requirements absent",
            s.id
        );
        assert!(
            !assignment.contains("Expected preview · original example"),
            "{} leaks practice into assignment",
            s.id
        );
        assert_eq!(
            a.progress.attempts[&s.id].state, state,
            "view navigation changed graded state: {}",
            s.id
        );
        assert!(a.metrics.script_goal.is_some());
    }
}

#[test]
fn rollout_bash_expected_preview_never_masquerades_as_actual_output_or_completion() {
    let (ctx, mut a) = app();
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "shell-functions")
        .unwrap();
    a.launch(i);
    let original = a.progress.attempts["shell-functions"].state.clone();
    a.set_script_view(i, teaching::View::Example);
    let text = rendered(&ctx, &mut a);
    assert!(text.contains("Actual result"));
    assert!(text.contains("Not run."));
    assert!(
        a.progress.attempts["shell-functions"]
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_result
            .is_none()
    );
    assert_eq!(a.progress.attempts["shell-functions"].state, original);
    assert!(!a.metrics.completion);
    a.set_script_view(i, teaching::View::Introduction);
    let text = rendered(&ctx, &mut a);
    assert!(text.contains("First call") && text.contains("badge Mina"));
    assert!(text.contains("Second call") && text.contains("Rowan Reed"));
    assert_eq!(a.progress.attempts["shell-functions"].state, original);
}

#[test]
fn rollout_file_lifecycle_three_views_teach_before_assignment_without_installing_answers() {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    for id in [
        "shell-stage-report",
        "shell-cleanup-preview",
        "shell-recover-publication",
    ] {
        let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
        a.launch(i);
        a.set_script_view(i, teaching::View::Introduction);
        let _ = rendered(&ctx, &mut a);
        let intro = rendered(&ctx, &mut a);
        let s = a.scenarios[i].clone();
        let entry = lesson_presentation::entry(&s).unwrap();
        for c in &entry.concepts {
            assert!(intro.contains(&c.title), "{id}: missing {}", c.title);
        }
        a.set_script_view(i, teaching::View::Example);
        let _ = rendered(&ctx, &mut a);
        let example = rendered(&ctx, &mut a);
        assert_eq!(entry.practice.len(), 3);
        for step in &entry.practice {
            assert!(
                example.contains(&step.title),
                "{id}: missing {}",
                step.title
            );
            assert!(
                example.contains(&step.action),
                "{id}: missing action {}",
                step.action
            );
        }
        assert_eq!(a.progress.attempts[id].state, s.initial);
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        a.set_script_view(i, teaching::View::Assignment);
        let _ = rendered(&ctx, &mut a);
        let task = rendered(&ctx, &mut a);
        assert!(task.contains("Requirements"));
        assert!(task.contains("Run script"));
        assert_eq!(a.progress.attempts[id].state, s.initial);
    }
}

#[test]
fn rollout_associative_three_views_teach_before_assignment_without_installing_answers() {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    for id in ["shell-map-lookup", "shell-map-reconcile"] {
        let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
        a.launch(i);
        a.set_script_view(i, teaching::View::Introduction);
        let _ = rendered(&ctx, &mut a);
        let intro = rendered(&ctx, &mut a);
        let s = a.scenarios[i].clone();
        let entry = lesson_presentation::entry(&s).unwrap();
        for c in &entry.concepts {
            assert!(intro.contains(&c.title), "{id}: missing {}", c.title);
        }
        a.set_script_view(i, teaching::View::Example);
        let _ = rendered(&ctx, &mut a);
        let example = rendered(&ctx, &mut a);
        assert_eq!(entry.practice.len(), 3);
        for step in &entry.practice {
            assert!(
                example.contains(&step.title),
                "{id}: missing {}",
                step.title
            );
            assert!(
                example.contains(&step.action),
                "{id}: missing action {}",
                step.action
            );
        }
        assert_eq!(a.progress.attempts[id].state, s.initial);
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        a.set_script_view(i, teaching::View::Assignment);
        let _ = rendered(&ctx, &mut a);
        let task = rendered(&ctx, &mut a);
        assert!(task.contains("Requirements"));
        assert!(task.contains("Run script"));
        assert_eq!(a.progress.attempts[id].state, s.initial);
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        a.help_open = false;
        let challenge = rendered(&ctx, &mut a);
        for c in &entry.concepts {
            assert!(
                !challenge.contains(&c.title),
                "{id}: Challenge reveals teaching by default"
            );
        }
        assert_eq!(a.progress.attempts[id].state, s.initial);
    }
}
