//! Focused virtual inode-listing, bounds and compatibility regressions.
use dlms_learning_lab::{
    links::{ErrorKind, State},
    links_workspace as work,
};
fn fixture() -> State {
    let mut s = State::new(&["/", "/work", "/archive", "/empty", "/work/-dir"], "/work").unwrap();
    s.create_file("/work/original", "shared data\n").unwrap();
    s.hard_link("/work/original", "/work/alias").unwrap();
    s.copy_file("/work/original", "/work/copy").unwrap();
    s.symbolic_link("original", "/work/shortcut").unwrap();
    s.symbolic_link("missing", "/work/dangling").unwrap();
    s.create_file("/work/.hidden", "hidden\n").unwrap();
    s
}
fn parsed(text: &str) -> Vec<(String, u32)> {
    text.lines()
        .map(|line| {
            let (id, name) = line.trim_start().split_once(' ').unwrap();
            (name.to_owned(), id.parse().unwrap())
        })
        .collect()
}
#[test]
fn typed_listing_reports_own_child_inode_with_aliases_copies_and_dangling_links() {
    let mut s = fixture();
    s.hard_link("/work/original", "/archive/pin").unwrap();
    let before = s.to_json().unwrap();
    let rows = s.directory_inode_entries("/work").unwrap();
    let get = |name: &str| rows.iter().find(|(n, _)| n == name).unwrap().1;
    assert_eq!(get("original"), get("alias"));
    assert_eq!(
        s.directory_inode_entries("/archive").unwrap(),
        [("pin".into(), get("original"))]
    );
    assert_ne!(get("original"), get("copy"));
    assert_ne!(get("original"), get("shortcut"));
    assert_eq!(get("shortcut"), s.stat("/work/shortcut").unwrap().inode);
    assert_ne!(
        get("shortcut"),
        s.stat_follow("/work/shortcut").unwrap().inode
    );
    assert_eq!(get("dangling"), s.stat("/work/dangling").unwrap().inode);
    assert!(rows.iter().any(|(n, _)| n == ".hidden"));
    assert_eq!(s.to_json().unwrap(), before);
}
#[test]
fn terminal_i_has_actual_visible_names_and_ids_without_reading_or_following_children() {
    let mut s = fixture();
    let before = s.clone();
    let text = work::execute(&mut s, "ls -i").unwrap();
    let rows = parsed(&text);
    assert!(!rows.iter().any(|(n, _)| n.starts_with('.')));
    assert_eq!(
        rows.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
        ["-dir", "alias", "copy", "dangling", "original", "shortcut"]
    );
    for (name, id) in rows {
        assert_eq!(id, s.stat(&format!("/work/{name}")).unwrap().inode);
    }
    assert_eq!(s, before);
}
#[test]
fn added_i_omits_hidden_names_and_empty_visible_directory_has_no_newline() {
    let mut s = fixture();
    s.create_file("/empty/.only-hidden", "inert").unwrap();
    let before = s.clone();
    assert_eq!(work::execute(&mut s, "ls -i /empty").unwrap(), "");
    assert_eq!(work::execute(&mut s, "ls -i /archive").unwrap(), "");
    // Compatibility: legacy bare listing and typed names view are not silently changed.
    assert_eq!(
        work::execute(&mut s, "ls /empty").unwrap(),
        ".only-hidden\n"
    );
    assert_eq!(s.directory_entries("/empty").unwrap(), [".only-hidden"]);
    assert_eq!(s, before);
}
#[test]
fn directory_operand_follows_its_symbolic_prefix_but_children_still_keep_own_ids() {
    let mut s = fixture();
    s.symbolic_link("/work", "/portal").unwrap();
    let before = s.clone();
    let direct = work::execute(&mut s, "ls -i /work").unwrap();
    assert_eq!(work::execute(&mut s, "ls -i /portal").unwrap(), direct);
    assert_eq!(
        work::execute(&mut s, "ls -i /portal/../work/.").unwrap(),
        direct
    );
    assert_eq!(s, before);
}
#[test]
fn file_and_symlink_to_file_operands_fail_with_honest_directory_only_diagnostic() {
    let mut s = fixture();
    let before = s.clone();
    for p in ["/work/original", "/work/shortcut"] {
        let e = work::execute(&mut s, &format!("ls -i {p}")).unwrap_err();
        assert!(e.contains("directory operands only") && e.contains("outside this subset"));
        assert_eq!(
            s.directory_inode_entries(p).unwrap_err().kind,
            ErrorKind::NotDirectory
        );
    }
    assert_eq!(s, before);
}
#[test]
fn options_quotes_literal_dash_names_and_composition_keep_closed_parser_scope() {
    let mut s = fixture();
    s.create_file("/work/-dir/name with space", "text").unwrap();
    let before = s.clone();
    assert_eq!(
        work::execute(&mut s, "ls -i -- -dir").unwrap(),
        work::execute(&mut s, "ls -i '/work/-dir'").unwrap()
    );
    assert!(
        work::execute(&mut s, "ls -i -- -dir")
            .unwrap()
            .ends_with(" name with space\n")
    );
    for bad in [
        "ls -li /work",
        "ls -i -a /work",
        "ls -i -L /work",
        "ls --inode /work",
        "ls -i /work /archive",
        "ls -i /work | cat",
        "ls -i $(pwd)",
        "ls -i /work > /work/result",
        "ls -i /work; rm /work/original",
    ] {
        assert!(work::execute(&mut s, bad).is_err(), "{bad}");
    }
    assert_eq!(s, before);
}
#[test]
fn directory_loop_missing_and_resource_errors_leave_every_state_counter_unchanged() {
    let mut s = fixture();
    s.symbolic_link("loop", "/work/loop").unwrap();
    let before = s.to_json().unwrap();
    assert_eq!(
        s.directory_inode_entries("/work/loop").unwrap_err().kind,
        ErrorKind::Loop
    );
    for command in ["ls -i /work/loop", "ls -i /absent"] {
        assert!(work::execute(&mut s, command).is_err());
    }
    assert_eq!(s.to_json().unwrap(), before);
}
#[test]
fn new_output_checks_final_inode_column_bytes_without_changing_raw_directory_api() {
    let mut s = State::new(&["/"], "/").unwrap();
    for i in 0..127 {
        s.create_file(&format!("/{i:03}-{}", "x".repeat(251)), "x")
            .unwrap();
    }
    let before = s.to_json().unwrap();
    assert_eq!(s.directory_entries("/").unwrap().len(), 127);
    assert_eq!(s.directory_inode_entries("/").unwrap().len(), 127);
    let error = work::execute(&mut s, "ls -i /").unwrap_err();
    assert!(error.contains("32KiB"));
    assert_eq!(s.to_json().unwrap(), before);
}
#[test]
fn candidate_alignment_is_derived_from_visible_id_width_not_screen_offsets() {
    let mut s = fixture();
    // Allocation grows IDs naturally; no expected host inode number is invented.
    for i in 0..12 {
        s.create_file(&format!("/work/extra-{i}"), "x").unwrap();
    }
    let text = work::execute(&mut s, "ls -i /work").unwrap();
    let rows = parsed(&text);
    let width = rows
        .iter()
        .map(|(_, id)| id.to_string().len())
        .max()
        .unwrap();
    for (line, (name, id)) in text.lines().zip(rows.iter()) {
        assert_eq!(line, format!("{id:>width$} {name}"));
    }
}
#[test]
fn readonly_session_queries_preserve_world_history_wire_and_independent_drafts() {
    let initial = fixture();
    let mut example = State::new(&["/", "/demo"], "/demo").unwrap();
    example.create_file("/demo/brief", "example\n").unwrap();
    let d = work::Definition {
        initial: initial.clone(),
        example,
        requirements: vec![work::Requirement::Content {
            path: "/work/original".into(),
            text: "shared data\n".into(),
        }],
        example_requirements: vec![work::Requirement::Content {
            path: "/demo/brief".into(),
            text: "example\n".into(),
        }],
        example_commands: vec!["cat /demo/brief".into()],
    };
    d.validate().unwrap();
    let mut s = work::Session::new(&d);
    s.assignment.command = "unfinished assignment draft".into();
    s.practice.command = "unfinished practice draft".into();
    let old_world = s.assignment.world.clone();
    let old_practice = s.practice.clone();
    let checks = d.checks(&s, false);
    let history = s.assignment.history.clone();
    let out = s.run(&d, "ls -i /work", false).unwrap();
    assert!(out.contains(" original\n"));
    assert_eq!(s.assignment.world, old_world);
    assert_eq!(s.assignment.history, history);
    assert_eq!(s.practice, old_practice);
    assert_eq!(s.assignment.command, "unfinished assignment draft");
    assert_eq!(d.checks(&s, false), checks);
    assert_eq!(s.version, 1);
    let raw = serde_json::to_vec(&s).unwrap();
    let restored: work::Session = serde_json::from_slice(&raw).unwrap();
    assert_eq!(restored, s);
    d.bound(&restored).unwrap();
    let mut probe = restored.clone();
    probe.run(&d, "ls -i /demo", true).unwrap();
    assert_eq!(probe.assignment, restored.assignment);
    assert!(probe.practice.history.is_empty());
}
