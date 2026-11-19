use dlms_learning_lab::automation_workshop::nfs_tools::workspace::{Origin, Task, Work};

fn tasks() -> Vec<Task> {
    let values: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/withdraw-export-fixtures.json")).unwrap();
    ["example", "assignment"]
        .into_iter()
        .map(|stage| serde_json::from_value(values[stage].clone()).unwrap())
        .collect()
}

fn corrected(task: &Task) -> String {
    task.definition.initial.runtime().servers[0]
        .draft
        .lines()
        .filter(|line| line.split_ascii_whitespace().next() != Some(task.withdrawn_root.as_str()))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

#[test]
fn saved_policy_must_be_applied_and_only_the_conflicting_mount_needs_retirement() {
    for task in tasks() {
        task.validate().unwrap();
        for args in ["exportfs -r", "exportfs -ar"] {
            let mut w = task.start();
            assert!(!w.outcomes(&task).unwrap().iter().all(|r| r.passed));
            assert!(!w.outcomes(&task).unwrap()[0].passed);
            w.edit_exports(&task, corrected(&task)).unwrap();
            w.save_exports(&task).unwrap();
            let held = w.state.clone();
            let address = w.state.inventory().servers[0].address;
            let remote = format!("showmount -e {address}");
            w.command(&task, remote.clone()).unwrap();
            assert!(
                w.history
                    .last()
                    .unwrap()
                    .reply
                    .contains(&task.withdrawn_root)
            );
            assert_eq!(w.state, held);
            w.command(&task, args.into()).unwrap();
            assert!(!w.history.last().unwrap().success);
            assert_eq!(w.state, held);
            w.unmount_retired(&task).unwrap();
            assert!(w.history.last().unwrap().success);
            w.command(&task, args.into()).unwrap();
            assert!(w.history.last().unwrap().success);
            w.command(&task, remote).unwrap();
            assert!(
                !w.history
                    .last()
                    .unwrap()
                    .reply
                    .contains(&task.withdrawn_root)
            );
            w.mount(&task, 0).unwrap();
            assert!(!w.history.last().unwrap().success);
            w.mount(&task, 1).unwrap();
            assert!(w.history.last().unwrap().success);
            w.read(&task, 1).unwrap();
            assert_eq!(w.history.last().unwrap().origin, Origin::Client);
            let result = w.outcomes(&task).unwrap();
            assert_eq!(result.len(), 12);
            assert!(result.iter().all(|r| r.passed), "{result:?}");
            assert!(w.history.len() <= 6);
            let mut malformed = w.clone();
            malformed
                .edit_exports(&task, "this is an unfinished export file\n".into())
                .unwrap();
            assert_eq!(
                malformed
                    .outcomes(&task)
                    .unwrap()
                    .iter()
                    .filter(|r| r.passed)
                    .count(),
                11,
            );
            malformed.save_exports(&task).unwrap();
            let before = malformed.state.clone();
            malformed.command(&task, args.into()).unwrap();
            assert!(!malformed.history.last().unwrap().success);
            assert_eq!(malformed.state, before);
            let failed = malformed
                .outcomes(&task)
                .unwrap()
                .into_iter()
                .filter(|row| !row.passed)
                .map(|row| row.name)
                .collect::<Vec<_>>();
            assert_eq!(
                failed,
                [
                    "Saved export rules are applied",
                    "A fresh approved client can mount and read"
                ]
            );
            let mut widened = w.clone();
            let profile = task.definition.initial.inventory().clients[1].address;
            let selector = format!("{}(ro,root_squash)", profile);
            let network = format!(
                "{}/24",
                std::net::Ipv4Addr::from(u32::from(profile) & 0xffff_ff00)
            );
            let broad = corrected(&task).replace(&selector, &format!("{network}(ro,root_squash)"));
            assert_ne!(broad, corrected(&task));
            widened.edit_exports(&task, broad).unwrap();
            widened.save_exports(&task).unwrap();
            widened.command(&task, args.into()).unwrap();
            assert!(widened.history.last().unwrap().success);
            assert!(widened.outcomes(&task).unwrap()[0].passed);
            assert!(!widened.outcomes(&task).unwrap()[1].passed);
            let mut altered = w.clone();
            altered
                .edit_exports(&task, "# removed everything\n".into())
                .unwrap();
            assert!(!altered.outcomes(&task).unwrap().iter().all(|r| r.passed));
            altered.save_exports(&task).unwrap();
            let held = altered.state.clone();
            altered.command(&task, args.into()).unwrap();
            assert!(!altered.history.last().unwrap().success);
            assert_eq!(altered.state, held);
            let restored: Work =
                serde_json::from_str(&serde_json::to_string(&altered).unwrap()).unwrap();
            assert_eq!(restored, altered);
            restored.validate(&task).unwrap();
        }
    }
}

#[test]
fn command_output_and_packet_free_discovery_are_observations_only() {
    for task in tasks() {
        let mut w = task.start();
        let before = w.state.clone();
        for hostile in [
            "showmount -e 192.0.2.99",
            "cat /etc/exports",
            "exportfs -v | mail someone",
        ] {
            w.command(&task, hostile.into()).unwrap();
            assert!(!w.history.last().unwrap().success);
            assert_eq!(w.state, before);
        }
        w.command(&task, "exportfs -v".into()).unwrap();
        assert_eq!(w.state, before);
        assert!(
            w.history
                .last()
                .unwrap()
                .reply
                .contains(&task.withdrawn_root)
        );
    }
}
