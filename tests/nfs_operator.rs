use dlms_learning_lab::automation_workshop::nfs_tools::operator::{Command, execute, parse};
use dlms_learning_lab::nfs::{Action, State};

fn lab(index: usize) -> State {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/nfs-workbench-v1.json")).unwrap();
    serde_json::from_value(fixtures["cases"][index]["definition"]["initial"].clone()).unwrap()
}

#[test]
fn local_and_remote_show_applied_exports_until_saved_changes_are_republished() {
    let mut state = lab(0);
    let server = &"s-assignment0";
    let remote = "showmount -e 192.0.2.10";
    let starter = state.clone();
    let listing = execute(&mut state, server, remote).unwrap();
    assert!(listing.starts_with("Export list for 192.0.2.10:\n"));
    assert!(listing.contains("reference 198.51.100.30"));
    assert_eq!(state, starter);
    let new = state.runtime().servers[0]
        .saved
        .replace("198.51.100.30(ro)", "192.0.2.21(ro)");
    state
        .apply(&Action::SetExportsDraft {
            server: server.to_string(),
            text: new.clone(),
        })
        .unwrap();
    assert_eq!(execute(&mut state, server, remote).unwrap(), listing);
    state
        .apply(&Action::SaveExportsDraft {
            server: server.to_string(),
        })
        .unwrap();
    assert_eq!(execute(&mut state, server, remote).unwrap(), listing);
    assert!(
        execute(&mut state, server, "exportfs -ra")
            .unwrap()
            .is_empty()
    );
    let now = execute(&mut state, server, remote).unwrap();
    assert!(now.contains("reference 192.0.2.21"));
    assert!(!now.contains("198.51.100.30"));
    assert_eq!(state.runtime().mounts, starter.runtime().mounts);
    assert_eq!(state.runtime().contents, starter.runtime().contents);
    assert!(
        execute(&mut state, server, "exportfs -v")
            .unwrap()
            .contains("192.0.2.21(sync,sec=sys,ro,root_squash")
    );
    assert_eq!(state.runtime().servers[0].saved, new);
    assert_eq!(parse("exportfs -a -r").unwrap(), Command::ReexportSaved);
    assert_eq!(parse("exportfs -r").unwrap(), Command::ReexportSaved);
}

#[test]
fn refusal_preserves_existing_mounts_and_unsupported_syntax_uses_no_host_context() {
    let mut state = lab(1);
    let server = "s-assignment1";
    let document = state.runtime().servers[0]
        .saved
        .replace("192.0.2.20(rw)", "192.0.2.20(ro)");
    state
        .apply(&Action::SetExportsDraft {
            server: server.into(),
            text: document,
        })
        .unwrap();
    state
        .apply(&Action::SaveExportsDraft {
            server: server.into(),
        })
        .unwrap();
    let before = state.clone();
    assert!(execute(&mut state, server, "exportfs -r").is_err());
    assert_eq!(state, before);
    for input in [
        "showmount -e 192.0.2.99",
        "showmount foo",
        "exportfs -ra > /etc/exports",
        "cat /proc/mounts",
        "exportfs -v\nexportfs -r",
    ] {
        assert!(execute(&mut state, server, input).is_err(), "{input}");
        assert_eq!(state, before);
    }
}
