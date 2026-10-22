use dlms_learning_lab::{
    engine::{self, Goal, State},
    filesystem::Requirement,
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
    terminal_teaching::{self, Learning},
    workspace,
};
fn labs() -> Vec<pack::Scenario> {
    pack::bundled()
        .unwrap()
        .into_iter()
        .filter(|s| s.terminal_teaching.is_some() && s.unit == "linux-file-foundations")
        .collect()
}
fn run(state: &mut State, commands: &[&str]) {
    for cmd in commands {
        engine::execute(state, cmd).unwrap();
    }
}
#[test]
fn every_file_assignment_accepts_distinct_supported_sequences_and_rejects_plausible_mistakes() {
    let alternates: Vec<Vec<&str>> = vec![
        vec![
            "cp /home/alex/projects/aurora/welcome.txt /home/alex/projects/aurora/ready.txt",
            "cd /home/alex/projects/aurora",
        ],
        vec![
            "mkdir \"release notes\"",
            "cd 'release notes'",
            "cp '../draft notes.txt' notes.txt",
        ],
        vec![
            "cp inbox/receipt.txt processed/receipt.txt",
            "rm inbox/receipt.txt",
            "cp ledger.txt archive/snapshot.txt",
        ],
        vec!["cd staging", "rmdir old", "rm .scratch", "rm draft.tmp"],
        vec![
            "umask 0077",
            "mkdir /home/alex/private/",
            "cd private",
            "touch ./notes.txt",
        ],
        vec![
            "umask 077",
            "mkdir private/",
            "cp reports/final.txt private/delivery.txt",
            "cp queue/job.txt done/job.txt",
            "rm queue/job.txt",
            "rm scratch.tmp",
            "cd done",
        ],
    ];
    let mistakes: Vec<Vec<&str>> = vec![
        vec![
            "cp projects/beacon/welcome.txt projects/aurora/ready.txt",
            "cd projects/aurora",
        ],
        vec![
            "mkdir 'release notes'",
            "mv 'draft notes.txt' 'release notes/notes.txt'",
        ],
        vec![
            "mv ledger.txt archive/snapshot.txt",
            "cp inbox/receipt.txt processed",
        ],
        vec![
            "rm staging/draft.tmp",
            "rmdir staging/old",
            "rm staging/.approved",
        ],
        vec!["mkdir private", "touch private/notes.txt", "umask 077"],
        vec![
            "umask 077",
            "mkdir private",
            "cp reports/final.txt private/delivery.txt",
            "mv queue/job.txt done",
            "rm scratch.tmp",
            "rm reports/final.txt",
            "cd done",
        ],
    ];
    let labs = labs();
    assert_eq!(labs.len(), 6);
    for ((lab, alternate), wrong) in labs.iter().zip(alternates).zip(mistakes) {
        let mut state = lab.initial.clone();
        run(&mut state, &alternate);
        assert!(
            engine::solved(&state, &lab.goal),
            "{} alternate: {:?}",
            lab.id,
            engine::checks(&state, &lab.goal)
        );
        let mut state = lab.initial.clone();
        run(&mut state, &wrong);
        assert!(!engine::solved(&state, &lab.goal), "{} wrong", lab.id);
        assert!(!workspace::issues(&state, &lab.goal).is_empty());
        let mut partial = lab.initial.clone();
        engine::execute(&mut partial, &lab.solution[0]).unwrap();
        assert!(!engine::solved(&partial, &lab.goal));
    }
}
#[test]
fn each_objective_is_current_state_not_historical_credit() {
    for lab in labs() {
        let mut state = lab.initial.clone();
        for cmd in &lab.solution {
            engine::execute(&mut state, cmd).unwrap();
        }
        assert!(engine::solved(&state, &lab.goal));
        let Goal::Filesystem { requirements } = &lab.goal else {
            panic!()
        };
        for requirement in requirements {
            let mut broken = state.clone();
            let State::Filesystem { tree } = &mut broken else {
                panic!()
            };
            match requirement {
                Requirement::Contents { path, .. } => {
                    tree.files.insert(path.clone(), "broken".into());
                }
                Requirement::Entry { path, .. } => {
                    tree.nodes.remove(path);
                    tree.files.remove(path);
                }
                Requirement::Absent { path } => {
                    let mut node = tree.nodes["/home/alex"].clone();
                    node.directory = false;
                    tree.nodes.insert(path.clone(), node);
                    tree.files.insert(path.clone(), String::new());
                }
                Requirement::Directory { .. } => tree.cwd = "/home/alex".into(),
                Requirement::Mode { path, mode } => {
                    tree.nodes.get_mut(path).unwrap().mode = mode ^ 0o004
                }
                Requirement::Mask { mask } => tree.umask = mask ^ 0o004,
                Requirement::Ownership { path, .. } => {
                    tree.nodes.remove(path);
                    tree.files.remove(path);
                }
                Requirement::Access { path, .. } => {
                    tree.nodes.remove(path);
                    tree.files.remove(path);
                }
            }
            assert!(
                !engine::solved(&broken, &lab.goal),
                "{} {:?}",
                lab.id,
                requirement
            );
        }
    }
}
#[test]
fn examples_are_runnable_editable_ungraded_and_both_drafts_resume() {
    let all = pack::bundled().unwrap();
    let mut progress = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for lab in labs() {
        let lesson = lab.terminal_teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        let mut attempt = Attempt::new(&lab, 1);
        workspace::prepare(&mut attempt);
        terminal_teaching::prepare(&mut attempt, lesson, true);
        attempt.workspace.as_mut().unwrap().command = "saved assignment input".into();
        let original = attempt.state.clone();
        let learning = attempt
            .workspace
            .as_mut()
            .unwrap()
            .terminal_learning
            .as_mut()
            .unwrap();
        for cmd in &lesson.example {
            learning.command = cmd.clone();
            learning.execute();
        }
        let before = learning.tree.clone();
        learning.command = "touch extra-practice-file".into();
        learning.execute();
        assert_ne!(learning.tree, before);
        learning.command = "retained practice input".into();
        learning.view = View::Example;
        assert_eq!(attempt.state, original);
        assert!(attempt.transcript.is_empty());
        assert!(!engine::solved(&attempt.state, &lab.goal));
        assert!(learning_valid(&attempt));
        progress.attempts.insert(lab.id.clone(), attempt);
    }
    let legacy = progress::decode(include_bytes!("fixtures/progress-v1.json"), &all).unwrap();
    for (id, attempt) in legacy.attempts {
        progress.attempts.insert(id, attempt);
    }
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("progress.json");
    progress::save(&path, &progress).unwrap();
    assert_eq!(
        serde_json::to_value(&progress).unwrap(),
        serde_json::to_value(progress::load(&path, &all).unwrap()).unwrap()
    );
}
fn learning_valid(a: &Attempt) -> bool {
    a.workspace
        .as_ref()
        .unwrap()
        .terminal_learning
        .as_ref()
        .unwrap()
        .valid()
}
#[test]
fn invalid_examples_and_practice_drafts_cannot_enter_saved_content() {
    let mut lesson = labs()[0].terminal_teaching.clone().unwrap();
    lesson.example = vec!["cat /host-personal-file".into()];
    assert!(lesson.validate().is_err());
    let mut learning = Learning::new(labs()[0].terminal_teaching.as_ref().unwrap());
    learning.command = "x".repeat(4097);
    assert!(!learning.valid());
    learning.command.clear();
    learning.transcript = vec!["x".repeat(40_001)];
    assert!(!learning.valid());
    learning.transcript.clear();
    learning
        .tree
        .files
        .insert("/unmodeled".into(), "data".into());
    assert!(!learning.valid());
}

#[test]
fn every_suggested_practice_change_has_verified_behavior() {
    let changes: Vec<Vec<&str>> = vec![
        vec!["ls", "cat copied.txt", "cd ..", "pwd"],
        vec!["mkdir 'second pack'", "cp 'guest list.txt' 'second pack'"],
        vec!["mv saved.txt second-copy.txt", "ls"],
        vec!["ls -a practice"],
        vec![
            "umask 022",
            "mkdir shared",
            "touch shared/note",
            "ls -l shared",
        ],
        vec!["cd ../secure", "cat copy.txt", "cd ../completed", "pwd"],
    ];
    for (index, (lab, change)) in labs().into_iter().zip(changes).enumerate() {
        let lesson = lab.terminal_teaching.as_ref().unwrap();
        let mut tree = lesson.fixture.clone();
        for command in &lesson.example {
            tree.execute(command).unwrap();
        }
        for requirement in &lesson.outcomes {
            assert!(requirement.check(&tree).0);
        }
        for command in change {
            tree.execute(command).unwrap();
        }
        match index {
            0 => assert_eq!(tree.cwd, "/home/alex"),
            1 => assert_eq!(
                tree.files["/home/alex/second pack/guest list.txt"],
                "Guests: Lee and Jo.\n"
            ),
            2 => {
                assert!(!tree.nodes.contains_key("/home/alex/saved.txt"));
                assert_eq!(
                    tree.files["/home/alex/second-copy.txt"],
                    tree.files["/home/alex/schedule.txt"]
                );
            }
            3 => {
                let before = tree.clone();
                assert!(
                    tree.execute("rmdir practice")
                        .unwrap_err()
                        .contains("not empty")
                );
                assert_eq!(tree, before);
            }
            4 => {
                assert_eq!(tree.nodes["/home/alex/shared"].mode, 0o755);
                assert_eq!(tree.nodes["/home/alex/shared/note"].mode, 0o644);
                assert_eq!(tree.nodes["/home/alex/journal"].mode, 0o700);
            }
            _ => assert_eq!(tree.cwd, "/home/alex/completed"),
        }
    }
    let mut lesson = labs()[0].terminal_teaching.clone().unwrap();
    lesson.example.pop();
    assert!(lesson.validate().unwrap_err().contains("declared outcome"));
}
