//! Authored content is exercised before registration. Numeric variants are not new labs.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack, progress, recursive_tree_source as source,
    recursive_tree_workspace::{self as work, Session},
};
fn authored() -> Vec<pack::Scenario> {
    let old = pack::accepted298().unwrap();
    let mut texts = old
        .chunks(8)
        .map(|chunk| {
            serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":chunk}))
                .unwrap()
        })
        .collect::<Vec<_>>();
    texts.push(include_str!("../scenarios/find-size-lessons-v1.json").into());
    pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>())
        .unwrap()
        .into_iter()
        .filter(|s| s.id.starts_with("lpic1-find-"))
        .collect()
}
fn def(s: &pack::Scenario) -> &work::Definition {
    source::source(s).unwrap()
}
fn run(s: &mut State, goal: &Goal, commands: &[String]) {
    for c in commands {
        engine::execute_for_goal(s, goal, c).unwrap();
    }
}

#[test]
fn both_authored_assignments_and_separate_worked_examples_are_complete_reachable_tasks() {
    let labs = authored();
    assert_eq!(labs.len(), 2);
    for lab in labs {
        let d = def(&lab);
        d.validate_reachability().unwrap();
        assert!(!engine::solved(&lab.initial, &lab.goal));
        let mut state = lab.initial.clone();
        run(&mut state, &lab.goal, &lab.solution);
        assert!(engine::solved(&state, &lab.goal));
        let mut s = Session::new(d).unwrap();
        let initial = s.assignment.clone();
        for c in &d.example_commands {
            s.run(d, c, true).unwrap();
        }
        assert!(s.check(d, true).unwrap().iter().all(|r| r.0));
        assert_eq!(s.assignment, initial);
        assert!(!d.checks(&s).iter().all(|r| r.0));
        assert!(
            lab.guide
                .iter()
                .all(|g| !g.why.is_empty() && !g.action.is_empty() && !g.commands.is_empty())
        );
        assert!(d.example_commands.iter().all(|c| !c.contains("/work/")));
        assert!(lab.solution.last().unwrap() == work::CHECK_ACTION);
    }
}

#[test]
fn bytes_not_letters_strict_boundary_partial_content_and_extra_files_receive_specific_failures() {
    let lab = authored().remove(0);
    let d = def(&lab);
    let mut s = Session::new(d).unwrap();
    let source = d.assignment.task.source();
    let dest = d.assignment.task.destination();
    assert_eq!(
        d.assignment.initial.files[&format!("{source}/multibyte.log")]
            .chars()
            .count(),
        61
    );
    assert_eq!(
        d.assignment.initial.files[&format!("{source}/multibyte.log")].len(),
        122
    );
    assert_eq!(
        d.assignment.initial.files[&format!("{source}/exact-boundary.log")].len(),
        120
    );
    let result = s
        .run(d, &format!("find {source} -type f -size +120c"), false)
        .unwrap();
    assert!(result.contains("multibyte.log") && result.contains("remote/error.log"));
    assert!(!result.contains("exact-boundary.log"));
    for c in [
        "umask 077".into(),
        format!("mkdir {dest}/remote"),
        format!("cp {source}/remote/error.log {dest}/remote/error.log"),
    ] {
        s.run(d, &c, false).unwrap();
    }
    let c = s.check(d, false).unwrap();
    assert!(!c[0].0 && c[0].1.contains("multibyte.log"));
    s.run(
        d,
        &format!("cp {source}/multibyte.log {dest}/multibyte.log"),
        false,
    )
    .unwrap();
    assert!(s.check(d, false).unwrap().iter().all(|r| r.0));
    s.run(
        d,
        &format!("cp {source}/exact-boundary.log {dest}/exact-boundary.log"),
        false,
    )
    .unwrap();
    assert!(!s.assignment.checked);
    let c = s.check(d, false).unwrap();
    assert!(!c[0].0 && c[0].1.contains("Unexpected"));
    s.run(d, &format!("rm {dest}/exact-boundary.log"), false)
        .unwrap();
    assert!(s.check(d, false).unwrap().iter().all(|r| r.0));
    s.assignment
        .tree
        .files
        .get_mut(&format!("{dest}/multibyte.log"))
        .unwrap()
        .push('\n');
    assert!(!d.checks(&s)[1].0);
}

#[test]
fn rounded_unit_misconception_and_initial_wrong_copies_cannot_pass() {
    let lab = authored().remove(1);
    let d = def(&lab);
    let mut s = Session::new(d).unwrap();
    let src = d.assignment.task.source();
    let dst = d.assignment.task.destination();
    let wrong = s
        .run(d, &format!("find {src} -type f -size -1024c"), false)
        .unwrap();
    let correct = s
        .run(d, &format!("find {src} -type f -size -1k"), false)
        .unwrap();
    assert!(wrong.contains("tiny.log") && wrong.contains("unicode.log"));
    assert!(!correct.contains("tiny.log") && !correct.contains("unicode.log"));
    assert!(correct.contains(".unfinished") && correct.contains("nested/empty.log"));
    assert!(s.check(d, false).unwrap().iter().any(|r| !r.0));
    for c in &d.reference {
        s.run(d, c, false).unwrap();
    }
    assert!(s.check(d, false).unwrap().iter().all(|r| r.0));
    s.run(d, &format!("cp {src}/tiny.log {dst}/tiny.log"), false)
        .unwrap();
    let c = s.check(d, false).unwrap();
    assert!(!c[0].0 && c[0].1.contains("1024-byte units") && c[0].1.contains("nonempty"));
    s.run(d, &format!("rm {dst}/tiny.log"), false).unwrap();
    assert!(s.check(d, false).unwrap().iter().all(|r| r.0));
    s.run(d, &format!("rm {src}/tiny.log"), false).unwrap();
    assert!(!s.assignment.checked);
    assert!(!s.check(d, false).unwrap()[4].0);
}

#[test]
fn unfinished_drafts_favorites_actual_save_read_and_checked_states_remain_source_bound() {
    let labs = authored();
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for lab in &labs {
        let mut a = progress::Attempt::new(lab, 1);
        run(&mut a.state, &lab.goal, &lab.solution);
        let State::RecursiveTree { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.command = "find /work ".into();
        simulation.practice.command = "unfinished practice query".into();
        p.attempts.insert(lab.id.clone(), a);
        p.favorites.insert(lab.id.clone());
    }
    let root = std::env::temp_dir().join(format!("find-size-lessons-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("progress.json");
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    let read = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&read).unwrap()
    );
    for lab in &labs {
        source::context(&read.attempts[&lab.id], lab).unwrap();
        assert!(progress::current_solved(&read.attempts[&lab.id], lab));
    }
    let mut wrong = read.attempts[&labs[0].id].clone();
    let State::RecursiveTree { simulation } = &mut wrong.state else {
        panic!()
    };
    simulation.practice.role = work::Role::Assignment;
    assert!(source::context(&wrong, &labs[0]).is_err());
    assert!(!progress::current_solved(&wrong, &labs[0]));
}

#[test]
fn supported_explicit_permission_repairs_and_equivalent_empty_file_method_pass() {
    let labs = authored();
    let lab = &labs[0];
    let d = def(lab);
    let mut s = Session::new(d).unwrap();
    let src = d.assignment.task.source();
    let dst = d.assignment.task.destination();
    for c in [
        format!("mkdir {dst}/remote"),
        format!("cp -- {src}/multibyte.log {dst}/multibyte.log"),
        format!("cp -- {src}/remote/error.log {dst}/remote/error.log"),
        format!("chmod 700 {dst}/remote"),
        format!("chmod 600 {dst}/multibyte.log"),
        format!("chmod 600 {dst}/remote/error.log"),
        "umask 0077".into(),
    ] {
        s.run(d, &c, false).unwrap();
    }
    assert!(s.check(d, false).unwrap().iter().all(|r| r.0));
    let lab = &labs[1];
    let d = def(lab);
    let mut s = Session::new(d).unwrap();
    let src = d.assignment.task.source();
    let dst = d.assignment.task.destination();
    let matches = s
        .run(d, &format!("find {src} -type f -size 0c"), false)
        .unwrap();
    assert!(matches.contains(".unfinished") && matches.contains("empty.log"));
    for c in [
        "umask 077".into(),
        format!("rm {dst}/tiny.log"),
        format!("rm {dst}/exact-block.log"),
        format!("mkdir {dst}/nested"),
        format!("touch {dst}/.unfinished"),
        format!("touch {dst}/nested/empty.log"),
        format!("chmod 400 {dst}/nested/empty.log"),
    ] {
        s.run(d, &c, false).unwrap();
    }
    // A correct resulting empty file is equivalent; the grader never requires a command string.
    assert!(s.check(d, false).unwrap().iter().all(|r| r.0));
}

#[test]
fn registered_catalog_and_unit_values_preserve_all298_before_adding_only_the_authored_pair() {
    let old = pack::accepted298().unwrap();
    let all = pack::accepted300().unwrap();
    assert_eq!((old.len(), all.len()), (298, 300));
    assert_eq!(
        serde_json::to_value(&all[..298]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let before = dlms_learning_lab::course::accepted298(&old).unwrap();
    let units = dlms_learning_lab::course::accepted300(&all).unwrap();
    assert_eq!((before.len(), units.len()), (97, 98));
    let retained = units
        .iter()
        .filter(|u| u.id != "linux-size-evidence")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(retained).unwrap(),
        serde_json::to_value(before).unwrap()
    );
    for expected in authored() {
        let actual = all.iter().find(|s| s.id == expected.id).unwrap();
        assert_eq!(
            serde_json::to_value(&expected).unwrap(),
            serde_json::to_value(actual).unwrap()
        );
    }
    for (track, n) in [
        ("CCNA", 74),
        ("LPIC-1", 105),
        ("LPIC-2", 45),
        ("Bash/Shell", 76),
    ] {
        assert_eq!(
            all.iter().filter(|s| s.track == track).count(),
            n,
            "{track}"
        );
    }
}
