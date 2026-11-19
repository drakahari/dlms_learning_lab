//! Outcome-based FHS practice using the existing bounded virtual filesystem.
use dlms_learning_lab::{
    course,
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
    terminal_teaching, workspace,
};
use std::sync::OnceLock;

fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted283().expect("FHS authored reachability"))
}
fn labs() -> Vec<&'static pack::Scenario> {
    catalog()
        .iter()
        .filter(|s| s.unit == "linux-file-locations")
        .collect()
}
fn commands(state: &mut State, lines: &[&str]) {
    for line in lines {
        engine::execute(state, line).unwrap_or_else(|e| panic!("{line}: {e}"));
    }
}
fn solution(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for line in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, line).unwrap();
    }
    assert!(
        engine::solved(&state, &s.goal),
        "{} {:?}",
        s.id,
        engine::checks(&state, &s.goal)
    );
    state
}

#[test]
fn fhs_catalog_preserves_all_281_labs_85_units_and_prerequisite_order() {
    let old = pack::accepted281().unwrap();
    let current = catalog();
    assert_eq!(current.len(), 283);
    assert_eq!(
        serde_json::to_value(&current[..281]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let old_units = course::accepted281(&old).unwrap();
    let units = course::accepted283(current).unwrap();
    assert_eq!(units.len(), 86);
    let kept: Vec<_> = units
        .iter()
        .filter(|u| u.id != "linux-file-locations")
        .collect();
    assert_eq!(
        serde_json::to_value(kept).unwrap(),
        serde_json::to_value(old_units).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-file-locations")
        .unwrap();
    for id in ["linux-file-foundations", "linux-file-search"] {
        assert!(units.iter().position(|u| u.id == id).unwrap() < at);
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| current.iter().filter(|s| s.track == track).count()),
        [72, 93, 45, 73]
    );
    assert_eq!(
        labs().iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        [
            "lpic1-fhs-application-layout",
            "lpic1-fhs-locate-active-files"
        ]
    );
}

#[test]
fn fhs_both_taught_examples_and_assignments_work_without_initial_credit() {
    for s in labs() {
        assert!(!engine::solved(&s.initial, &s.goal));
        solution(s);
        let lesson = s.terminal_teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        let State::Filesystem { tree } = &s.initial else {
            panic!()
        };
        assert_ne!(tree, &lesson.fixture);
        assert_eq!(tree.user, "alex");
        assert_eq!(lesson.fixture.user, "alex");
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        terminal_teaching::prepare(&mut a, lesson, true);
        let original = serde_json::to_value(&a.state).unwrap();
        let learning = a
            .workspace
            .as_mut()
            .unwrap()
            .terminal_learning
            .as_mut()
            .unwrap();
        for line in &lesson.example {
            learning.command = line.clone();
            learning.execute();
        }
        assert!(lesson.outcomes.iter().all(|r| r.check(&learning.tree).0));
        assert_eq!(serde_json::to_value(&a.state).unwrap(), original);
        assert!(!progress::current_solved(&a, s));
    }
}

#[test]
fn fhs_relative_moves_and_copy_remove_alternatives_grade_the_same_outcomes() {
    let s = labs()[0];
    let mut relative = s.initial.clone();
    commands(
        &mut relative,
        &[
            "cd /var/log/beacon",
            "mv settings.conf /etc/beacon/settings.conf",
            "cd /var/lib/beacon",
            "mv labels.txt /usr/local/share/beacon/labels.txt",
            "cd /usr/local/share/beacon",
            "mv cursor.db /var/lib/beacon/cursor.db",
            "cd /etc/beacon",
            "mv activity.log /var/log/beacon/activity.log",
        ],
    );
    assert!(engine::solved(&relative, &s.goal));
    let mut alternate = s.initial.clone();
    commands(
        &mut alternate,
        &[
            "cp /work/intake/activity.log /var/log/beacon/activity.log",
            "rm /etc/beacon/activity.log",
            "cp /work/intake/cursor.db /var/lib/beacon/cursor.db",
            "rm /usr/local/share/beacon/cursor.db",
            "cp /work/intake/labels.txt /usr/local/share/beacon/labels.txt",
            "rm /var/lib/beacon/labels.txt",
            "cp /work/intake/settings.conf /etc/beacon/settings.conf",
            "rm /var/log/beacon/settings.conf",
        ],
    );
    assert!(engine::solved(&alternate, &s.goal));
}

#[test]
fn fhs_independent_copy_order_and_narrower_find_are_supported_alternatives() {
    let s = labs()[1];
    let mut state = s.initial.clone();
    commands(
        &mut state,
        &[
            "find /etc/harbor -type f -name 'site.conf'",
            "cat /etc/harbor/site.conf",
            "cd /var/log/harbor",
            "cp events.log /work/support/activity.txt",
            "cd /var/lib/harbor",
            "cp cursor.db /work/support/checkpoint.txt",
            "cd /usr/local/share/harbor",
            "cp routes.tsv /work/support/reference.txt",
            "cd /etc/harbor",
            "cp site.conf /work/support/config.txt",
        ],
    );
    assert!(engine::solved(&state, &s.goal));
}

#[test]
fn fhs_copying_correct_contents_without_removing_wrong_roles_stays_partial() {
    let s = labs()[0];
    let mut state = s.initial.clone();
    commands(
        &mut state,
        &[
            "cp /work/intake/settings.conf /etc/beacon/settings.conf",
            "cp /work/intake/labels.txt /usr/local/share/beacon/labels.txt",
            "cp /work/intake/cursor.db /var/lib/beacon/cursor.db",
            "cp /work/intake/activity.log /var/log/beacon/activity.log",
        ],
    );
    let checks = engine::checks(&state, &s.goal);
    assert!(checks[..4].iter().all(|r| r.0));
    assert!(checks[4..8].iter().all(|r| !r.0));
    assert!(checks[4..8].iter().all(|r| r.1.contains("unrequested")));
    assert!(!engine::solved(&state, &s.goal));
}

#[test]
fn fhs_state_in_log_location_invalidates_success_and_retry_restores_it() {
    let s = labs()[0];
    let mut a = Attempt::new(s, 1);
    a.state = solution(s);
    assert!(progress::current_solved(&a, s));
    commands(
        &mut a.state,
        &["mv /var/lib/beacon/cursor.db /var/log/beacon/cursor.db"],
    );
    assert!(!progress::current_solved(&a, s));
    assert!(!engine::checks(&a.state, &s.goal)[2].0);
    commands(
        &mut a.state,
        &["mv /var/log/beacon/cursor.db /var/lib/beacon/cursor.db"],
    );
    assert!(progress::current_solved(&a, s));
}

#[test]
fn fhs_archive_or_neighbour_basename_is_not_an_active_support_file() {
    let s = labs()[1];
    for wrong in [
        "cp /var/log/harbor/archive/events.log /work/support/activity.txt",
        "cp /etc/neighbour/site.conf /work/support/config.txt",
        "cp /usr/local/share/harbor/archive/routes.tsv /work/support/reference.txt",
        "cp /var/lib/harbor/archive/cursor.db /work/support/checkpoint.txt",
    ] {
        let mut state = solution(s);
        commands(&mut state, &[wrong]);
        assert!(!engine::solved(&state, &s.goal));
        let checks = engine::checks(&state, &s.goal);
        assert!(checks[..4].iter().any(|r| !r.0));
        assert!(
            checks[4].0,
            "Same inventory must not make wrong source contents pass"
        );
    }
}

#[test]
fn fhs_partial_outputs_hidden_extras_and_moved_sources_do_not_pass() {
    let s = labs()[1];
    for damage in [
        "rm /work/support/config.txt",
        "cp /home/alex/private-note.txt /work/support/.notes",
        "mv /var/lib/harbor/cursor.db /work/support/checkpoint.txt",
    ] {
        let mut state = solution(s);
        commands(&mut state, &[damage]);
        assert!(!engine::solved(&state, &s.goal));
        let checks = engine::checks(&state, &s.goal);
        assert!(checks.iter().any(|r| r.0) && checks.iter().any(|r| !r.0));
    }
}

#[test]
fn fhs_intake_and_private_policy_are_required_even_after_correct_deployment() {
    for (s, wrong) in [
        (labs()[0], "rm /work/intake/settings.conf"),
        (labs()[0], "chmod 644 /home/alex/private-note.txt"),
        (labs()[1], "chmod 644 /home/alex/private-note.txt"),
    ] {
        let mut state = solution(s);
        commands(&mut state, &[wrong]);
        assert!(!engine::solved(&state, &s.goal));
    }
}

#[test]
fn fhs_no_sudo_host_dispatch_expansion_or_protected_parent_mutation() {
    for s in labs() {
        for input in [
            "sudo mv /etc/site-policy.txt /work/policy.txt",
            "sh -c whoami",
            "/bin/sh",
            "cat /etc/site-policy.txt | sh",
            "touch $(whoami)",
            "touch /etc/unauthorized",
            "rm /etc/site-policy.txt",
        ] {
            let mut state = s.initial.clone();
            let before = state.clone();
            assert!(engine::execute(&mut state, input).is_err(), "{input}");
            assert_eq!(
                state, before,
                "Refused operation must not mutate virtual state"
            );
        }
    }
}

#[test]
fn fhs_both_drafts_favorites_partial_state_resume_and_selected_reset_preserve_work() {
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in labs() {
        let mut a = Attempt::new(s, 7);
        workspace::prepare(&mut a);
        terminal_teaching::prepare(&mut a, s.terminal_teaching.as_ref().unwrap(), true);
        a.workspace.as_mut().unwrap().command = "cat /etc/unfinished".into();
        let learning = a
            .workspace
            .as_mut()
            .unwrap()
            .terminal_learning
            .as_mut()
            .unwrap();
        learning.command = "find /var/lib -type f ".into();
        learning.view = View::Example;
        learning.command = s.terminal_teaching.as_ref().unwrap().example[0].clone();
        learning.execute();
        learning.command = "cat /work/unsubmitted".into();
        let command = s.solution.last().unwrap();
        engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
        assert!(!progress::current_solved(&a, s));
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let mut loaded = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let other = serde_json::to_value(&loaded.attempts[&labs()[1].id]).unwrap();
    loaded
        .attempts
        .insert(labs()[0].id.clone(), Attempt::new(labs()[0], 8));
    progress::save_with_catalog(&path, &loaded, catalog()).unwrap();
    let restored = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&restored.attempts[&labs()[1].id]).unwrap(),
        other
    );
    assert_eq!(restored.attempts[&labs()[0].id].state, labs()[0].initial);
    assert_eq!(restored.favorites, p.favorites);
}

#[test]
fn fhs_teaching_and_source_boundaries_do_not_claim_unsupported_discovery_commands() {
    for s in labs() {
        assert!(s.simplifications.contains("locate"));
        assert!(s.simplifications.contains("updatedb"));
        assert!(!s.retry_guidance.trim().is_empty());
        let mut invalid = s.terminal_teaching.clone().unwrap();
        invalid.example = vec!["sudo true".into()];
        assert!(invalid.validate().is_err());
        let mut invalid = s.terminal_teaching.clone().unwrap();
        invalid.fixture.user = "root".into();
        assert!(invalid.validate().is_err());
    }
}
