use dlms_learning_lab::{
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    samba_share_source as source, samba_share_workspace as work, workspace,
};
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!(
        "../scenarios/samba-standalone-lessons-v1.json"
    ))
    .unwrap()
}
fn session(a: &mut Attempt) -> &mut work::Session {
    let State::SambaShare { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn authored_methods_partial_work_and_supported_alternatives_use_real_access_layers() {
    for s in labs() {
        let d = source::source(&s).unwrap();
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        assert!(!progress::current_solved(&a, &s));
        for (i, event) in s.solution.iter().enumerate() {
            engine::execute_for_goal(&mut a.state, &s.goal, event)
                .unwrap_or_else(|e| panic!("{} #{i}: {e}", s.id));
            if i + 1 < s.solution.len() {
                assert!(!progress::current_solved(&a, &s));
            }
        }
        assert!(
            progress::current_solved(&a, &s),
            "{} {:?}",
            s.id,
            engine::checks(&a.state, &s.goal)
        );
        let b = session(&mut a);
        b.assignment
            .edit(
                &d.assignment,
                b.assignment
                    .editor
                    .replace("read only = yes", "writable = no")
                    .replace("valid users = @team", "valid users = mira noor @team"),
            )
            .unwrap();
        b.assignment.save(&d.assignment).unwrap();
        b.assignment.publish(&d.assignment).unwrap();
        b.assignment.check(&d.assignment).unwrap();
        assert!(
            progress::current_solved(&a, &s),
            "{} equivalent alias/membership",
            s.id
        );
        let team = dlms_learning_lab::samba_share_config::parse(&d.assignment.reference.published)
            .unwrap()
            .shares
            .get("team")
            .unwrap()
            .path
            .clone();
        session(&mut a)
            .assignment
            .command(&d.assignment, &format!("chmod g-x {team}"))
            .unwrap();
        assert!(!progress::current_solved(&a, &s));
        session(&mut a).assignment.checked = true;
        assert!(
            !progress::current_solved(&a, &s),
            "forged checked does not restore broken state"
        );
    }
}
#[test]
fn separate_practice_disk_resume_keeps_drafts_favorites_and_current_credit() {
    for s in labs() {
        let d = source::source(&s).unwrap();
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        for event in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, event).unwrap();
        }
        let b = session(&mut a);
        b.practice.editor.push_str("\n# unfinished guide draft\n");
        b.practice.request.operation = work::Operation::Read {
            path: String::new(),
        };
        b.practice.terminal = "ls -ld /practice/notes".into();
        d.context(b).unwrap();
        let before = b.clone();
        let mut p = Progress {
            schema_version: 1,
            clock: 1,
            ..Default::default()
        };
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress-v1.json");
        progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
        let mut restored = progress::load(&path, std::slice::from_ref(&s)).unwrap();
        assert_eq!(*session(restored.attempts.get_mut(&s.id).unwrap()), before);
        assert!(progress::current_solved(&restored.attempts[&s.id], &s));
        assert!(restored.favorites.contains(&s.id));
        let raw = std::fs::read(&path).unwrap();
        assert!(progress::save(&path, &p).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), raw);
        let reset = d.new_session().unwrap();
        assert!(!reset.assignment.solved(&d.assignment));
        assert!(reset.practice.editor.ends_with('\n'));
        assert_eq!(reset.assignment.world, d.assignment.initial);
    }
}
#[test]
fn foreign_source_metadata_actions_and_oversized_input_are_rejected_without_host_execution() {
    for s in labs() {
        let mut a = Attempt::new(&s, 1);
        let before = serde_json::to_value(&a.state).unwrap();
        for bad in [
            "smbpasswd -a host-user",
            "smb-share {\"kind\":\"Spawn\"}",
            "smb-share {\"kind\":\"Check\",\"passed\":true}",
            "smb-share {\"kind\":\"Account\",\"user\":\"root\",\"enabled\":true}",
        ] {
            assert!(engine::execute_for_goal(&mut a.state, &s.goal, bad).is_err());
            assert_eq!(serde_json::to_value(&a.state).unwrap(), before);
        }
        assert!(
            engine::execute_for_goal(&mut a.state, &s.goal, &"x".repeat(source::MAX_ACTION + 1))
                .is_err()
        );
        assert!(engine::execute(&mut a.state, "smb-share {\"kind\":\"Check\"}").is_err());
        let mut wrong = s.clone();
        wrong.track = "CCNA".into();
        assert!(source::source(&wrong).is_err());
        assert!(!progress::current_solved(&a, &wrong));
        wrong = s.clone();
        wrong.version += 1;
        assert!(source::source(&wrong).is_err());
        session(&mut a).id = "foreign".into();
        assert!(source::context(&a, &s).is_err());
    }
}
