use dlms_learning_lab::links::*;
fn fixture() -> State {
    let mut s = State::new(
        &[
            "/",
            "/home",
            "/home/desk",
            "/srv",
            "/srv/release",
            "/srv/release/live",
            "/archive",
        ],
        "/home/desk",
    )
    .unwrap();
    for (p, text) in [
        ("/home/desk/item", "desk data\n"),
        ("/home/desk/note", "lexical parent\n"),
        ("/srv/release/note", "physical parent\n"),
        ("/archive/item", "archive data\n"),
    ] {
        s.create_file(p, text).unwrap();
    }
    s
}
fn raw(s: &State) -> serde_json::Value {
    serde_json::from_slice(&s.to_json().unwrap()).unwrap()
}
fn decode(v: &serde_json::Value) -> Result<State> {
    State::from_json(&serde_json::to_vec(v).unwrap())
}
fn ino(s: &State, p: &str) -> u32 {
    s.stat(p).unwrap().inode
}
#[test]
fn hard_alias_is_shared_identity_and_mutation_while_copy_is_independent() {
    let mut s = fixture();
    s.hard_link("item", "alias").unwrap();
    s.copy_file("item", "copy").unwrap();
    assert_eq!(ino(&s, "item"), ino(&s, "alias"));
    assert_ne!(ino(&s, "item"), ino(&s, "copy"));
    assert_eq!(s.stat("item").unwrap().links, 2);
    assert_eq!(s.stat("alias").unwrap().links, 2);
    assert_eq!(s.stat("copy").unwrap().links, 1);
    s.write_file("alias", "updated\n").unwrap();
    assert_eq!(s.read_file("item").unwrap(), "updated\n");
    assert_eq!(s.read_file("alias").unwrap(), "updated\n");
    assert_eq!(s.read_file("copy").unwrap(), "desk data\n");
    s.write_file("copy", "separate\n").unwrap();
    assert_eq!(s.read_file("item").unwrap(), "updated\n");
}
#[test]
fn unlink_preserves_other_aliases_and_collects_only_the_last_unreferenced_inode() {
    let mut s = fixture();
    s.hard_link("item", "alias").unwrap();
    let id = ino(&s, "item");
    s.unlink("item").unwrap();
    assert_eq!(s.read_file("alias").unwrap(), "desk data\n");
    assert_eq!(s.stat("alias").unwrap().links, 1);
    assert_eq!(ino(&s, "alias"), id);
    assert_eq!(s.read_file("item").unwrap_err().kind, ErrorKind::NotFound);
    s.unlink("alias").unwrap();
    assert!(raw(&s)["inodes"].get(id.to_string().as_str()).is_none());
    s.create_file("item", "replacement\n").unwrap();
    assert_ne!(ino(&s, "item"), id);
    s.validate().unwrap();
}
#[test]
fn relative_target_is_anchored_at_link_parent_not_cwd_and_absolute_target_stays_absolute() {
    let mut s = fixture();
    s.symbolic_link("item", "/archive/relative").unwrap();
    s.symbolic_link("/home/desk/item", "/archive/absolute")
        .unwrap();
    assert_eq!(s.read_file("/archive/relative").unwrap(), "archive data\n");
    assert_eq!(s.read_file("/archive/absolute").unwrap(), "desk data\n");
    assert_eq!(s.read_link("/archive/relative").unwrap(), "item");
    assert_eq!(s.read_link("/archive/absolute").unwrap(), "/home/desk/item");
    s.symbolic_link("../desk/item", "/home/desk/parent-relative")
        .unwrap();
    assert_eq!(s.read_file("parent-relative").unwrap(), "desk data\n");
}
#[test]
fn dangling_link_is_inspectable_and_repairs_when_the_target_appears() {
    let mut s = fixture();
    s.symbolic_link("missing", "latest").unwrap();
    assert_eq!(s.stat("latest").unwrap().kind, Kind::SymbolicLink);
    assert_eq!(s.read_link("latest").unwrap(), "missing");
    assert_eq!(
        s.stat_follow("latest").unwrap_err().kind,
        ErrorKind::NotFound
    );
    assert_eq!(s.read_file("latest").unwrap_err().kind, ErrorKind::NotFound);
    let before = s.clone();
    assert!(s.write_file("latest", "not auto-created").is_err());
    assert_eq!(s, before);
    s.create_file("missing", "arrived\n").unwrap();
    assert_eq!(s.read_file("latest").unwrap(), "arrived\n");
    s.unlink("missing").unwrap();
    assert_eq!(s.read_file("latest").unwrap_err().kind, ErrorKind::NotFound);
    assert_eq!(s.read_link("latest").unwrap(), "missing");
}
#[test]
fn physical_resolution_follows_symlink_before_parent_and_handles_relative_link_parents() {
    let mut s = fixture();
    s.symbolic_link("/srv/release/live", "portal").unwrap();
    assert_eq!(s.read_file("portal/../note").unwrap(), "physical parent\n");
    assert_ne!(
        s.read_file("portal/../note").unwrap(),
        s.read_file("note").unwrap()
    );
    assert_eq!(
        s.resolve("portal/..", true).unwrap().physical_path,
        "/srv/release"
    );
    s.symbolic_link("../note", "/srv/release/live/relative-note")
        .unwrap();
    assert_eq!(
        s.read_file("portal/relative-note").unwrap(),
        "physical parent\n"
    );
    assert_eq!(
        s.resolve("//home///desk/./portal/../note", true)
            .unwrap()
            .physical_path,
        "/srv/release/note"
    );
    assert_eq!(s.resolve("/../../", true).unwrap().physical_path, "/");
}
#[test]
fn path_non_directory_trailing_slash_root_and_unmodeled_destination_shorthand_are_honest() {
    let mut s = fixture();
    s.symbolic_link("item", "file-link").unwrap();
    s.symbolic_link("/archive", "dir-link").unwrap();
    for p in [
        "item/",
        "item/..",
        "item/.",
        "file-link/",
        "file-link/../note",
    ] {
        assert_eq!(
            s.resolve(p, true).unwrap_err().kind,
            ErrorKind::NotDirectory,
            "{p}"
        );
    }
    assert_eq!(s.stat("dir-link/").unwrap().kind, Kind::Directory);
    assert_eq!(
        s.read_link("dir-link/").unwrap_err().kind,
        ErrorKind::NotSymlink
    );
    assert_eq!(
        s.read_file("/../../host-only").unwrap_err().kind,
        ErrorKind::NotFound
    );
    let before = s.clone();
    for dest in ["/archive", "/archive/", "dir-link", ".", ".."] {
        assert!(s.hard_link("item", dest).is_err());
        assert_eq!(s, before);
    }
    assert_eq!(s.create_file("", "x").unwrap_err().kind, ErrorKind::Invalid);
    assert_eq!(
        s.create_file("/unknown/item", "x").unwrap_err().kind,
        ErrorKind::NotFound
    );
    assert_eq!(s, before);
}
#[test]
fn stat_default_no_follow_readlink_raw_text_and_editor_following_are_distinct() {
    let mut s = fixture();
    s.symbolic_link("./item", "view").unwrap();
    assert_eq!(s.stat("view").unwrap().kind, Kind::SymbolicLink);
    assert_eq!(s.stat("view").unwrap().size_bytes, Some(6));
    assert_eq!(s.stat_follow("view").unwrap().kind, Kind::Regular);
    assert_eq!(s.stat_follow("view").unwrap().inode, ino(&s, "item"));
    assert_eq!(s.read_link("view").unwrap(), "./item");
    assert_eq!(s.read_link("item").unwrap_err().kind, ErrorKind::NotSymlink);
    s.write_file("view", "from link\n").unwrap();
    assert_eq!(s.read_file("item").unwrap(), "from link\n");
    assert_eq!(s.stat("view").unwrap().kind, Kind::SymbolicLink);
    let root = s.stat("/").unwrap();
    assert_eq!(root.links, 5);
    assert_eq!(root.size_bytes, None);
}
#[test]
fn moving_regular_aliases_and_relative_symlinks_preserves_identity_not_link_target_resolution() {
    let mut s = fixture();
    s.hard_link("item", "alias").unwrap();
    let id = ino(&s, "item");
    s.move_entry("alias", "/archive/alias").unwrap();
    assert_eq!(ino(&s, "/archive/alias"), id);
    assert_eq!(s.stat("item").unwrap().links, 2);
    s.symbolic_link("item", "view").unwrap();
    let link_id = ino(&s, "view");
    assert_eq!(s.read_file("view").unwrap(), "desk data\n");
    s.move_entry("view", "/archive/view").unwrap();
    assert_eq!(ino(&s, "/archive/view"), link_id);
    assert_eq!(s.read_link("/archive/view").unwrap(), "item");
    assert_eq!(s.read_file("/archive/view").unwrap(), "archive data\n");
    s.symbolic_link("/home/desk/item", "absolute").unwrap();
    s.move_entry("absolute", "/archive/absolute").unwrap();
    assert_eq!(s.read_file("/archive/absolute").unwrap(), "desk data\n");
    s.unlink("/archive/view").unwrap();
    assert_eq!(s.read_file("/archive/item").unwrap(), "archive data\n");
}
#[test]
fn symlink_cycles_and_finite_hop_limit_fail_without_mutating_or_disabling_lstat() {
    let mut s = fixture();
    s.symbolic_link("self", "self").unwrap();
    s.symbolic_link("two", "one").unwrap();
    s.symbolic_link("one", "two").unwrap();
    let before = s.clone();
    for p in ["self", "one", "two"] {
        assert_eq!(s.read_file(p).unwrap_err().kind, ErrorKind::Loop);
        assert_eq!(s.stat(p).unwrap().kind, Kind::SymbolicLink);
    }
    assert_eq!(s, before);
    for n in (0..40).rev() {
        let target = if n == 39 {
            "item".into()
        } else {
            format!("s{}", n + 1)
        };
        s.symbolic_link(&target, &format!("s{n}")).unwrap();
    }
    assert_eq!(s.read_file("s0").unwrap(), "desk data\n");
    s.symbolic_link("s0", "s40").unwrap();
    assert_eq!(s.read_file("s40").unwrap_err().kind, ErrorKind::Loop);
    assert_eq!(s.read_link("s40").unwrap(), "s0");
}
#[test]
fn unsupported_hard_symlink_and_directory_operations_never_silently_follow_or_change_state() {
    let mut s = fixture();
    s.symbolic_link("item", "sym").unwrap();
    let before = s.clone();
    assert_eq!(
        s.hard_link("sym", "bad").unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert_eq!(
        s.hard_link("/archive", "bad").unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert_eq!(
        s.move_entry("/archive", "/moved").unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert_eq!(
        s.unlink("/archive").unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert_eq!(
        s.write_file("/archive", "x").unwrap_err().kind,
        ErrorKind::IsDirectory
    );
    assert_eq!(
        s.read_file("/archive").unwrap_err().kind,
        ErrorKind::IsDirectory
    );
    assert_eq!(s, before);
    s.copy_file("sym", "copied-target").unwrap();
    assert_ne!(ino(&s, "copied-target"), ino(&s, "item"));
    assert_eq!(s.read_file("copied-target").unwrap(), "desk data\n");
}
#[test]
fn existing_destination_refuses_replace_and_all_failed_mutations_are_atomic() {
    let mut s = fixture();
    let before = s.clone();
    assert_eq!(
        s.hard_link("item", "note").unwrap_err().kind,
        ErrorKind::Exists
    );
    assert_eq!(
        s.symbolic_link("item", "note").unwrap_err().kind,
        ErrorKind::Exists
    );
    assert_eq!(
        s.move_entry("item", "note").unwrap_err().kind,
        ErrorKind::Exists
    );
    assert_eq!(
        s.copy_file("item", "note").unwrap_err().kind,
        ErrorKind::Exists
    );
    assert_eq!(
        s.create_file("note", "new").unwrap_err().kind,
        ErrorKind::Exists
    );
    assert_eq!(s.unlink("item/").unwrap_err().kind, ErrorKind::Unsupported);
    assert_eq!(
        s.move_entry("item", "note/").unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert_eq!(
        s.symbolic_link("", "new").unwrap_err().kind,
        ErrorKind::Invalid
    );
    assert_eq!(s, before);
}
#[test]
fn behavioral_checks_accept_different_valid_link_repairs_and_invalidate_after_damage() {
    let check = |s: &State| s.read_file("published").is_ok_and(|v| v == "desk data\n");
    let mut hard = fixture();
    assert!(!check(&hard));
    hard.hard_link("item", "published").unwrap();
    assert!(check(&hard));
    let mut relative = fixture();
    relative.symbolic_link("item", "published").unwrap();
    assert!(check(&relative));
    let mut absolute = fixture();
    absolute
        .symbolic_link("/home/desk/item", "published")
        .unwrap();
    assert!(check(&absolute));
    hard.write_file("item", "broken\n").unwrap();
    assert!(!check(&hard));
    relative.unlink("item").unwrap();
    assert!(!check(&relative));
    absolute.unlink("published").unwrap();
    assert!(!check(&absolute));
    // A task requiring shared identity must additionally inspect inode/link counts;
    // equal copied bytes alone are never evidence of a hard link.
    let mut copied = fixture();
    copied.copy_file("item", "published").unwrap();
    assert!(check(&copied));
    assert_ne!(ino(&copied, "published"), ino(&copied, "item"));
}
#[test]
fn codec_resume_preserves_shared_inodes_links_cwd_and_independent_reset() {
    let initial = fixture();
    let mut s = initial.clone();
    s.hard_link("item", "alias").unwrap();
    s.symbolic_link("alias", "view").unwrap();
    s.copy_file("item", "copy").unwrap();
    let mut reopened = State::from_json(&s.to_json().unwrap()).unwrap();
    assert_eq!(reopened, s);
    assert!(reopened.environment_matches(&initial).unwrap());
    reopened.write_file("view", "resumed\n").unwrap();
    assert_eq!(reopened.read_file("alias").unwrap(), "resumed\n");
    assert_eq!(reopened.read_file("item").unwrap(), "resumed\n");
    assert_eq!(reopened.read_file("copy").unwrap(), "desk data\n");
    assert_eq!(reopened.cwd().unwrap(), "/home/desk");
    let before = reopened.clone();
    reopened.stat("view").unwrap();
    reopened.read_link("view").unwrap();
    reopened.directory_entries(".").unwrap();
    reopened.resolve("view", true).unwrap();
    assert_eq!(reopened, before);
    reopened = initial.clone();
    assert_eq!(reopened, initial);
    assert!(reopened.read_link("view").is_err());
    assert_eq!(s.read_file("alias").unwrap(), "desk data\n");
}
#[test]
fn malformed_or_foreign_raw_state_is_rejected_including_orphans_and_alias_directory_inodes() {
    let s = fixture();
    for change in 0..10 {
        let mut v = raw(&s);
        match change {
            0 => v["entries"]["/missing-parent/child"] = serde_json::json!(ino(&s, "item")),
            1 => v["entries"]["/home/desk/foreign"] = serde_json::json!(9999),
            2 => v["entries"]["/home/desk/dir-alias"] = serde_json::json!(ino(&s, "/archive")),
            3 => {
                v["entries"]
                    .as_object_mut()
                    .unwrap()
                    .remove("/home/desk/item");
            }
            4 => v["cwd"] = serde_json::json!("/home/desk/item"),
            5 => v["entries"]["/home/desk/../bad"] = serde_json::json!(ino(&s, "item")),
            6 => v["inodes"]["999"] = serde_json::json!({"Regular":{"data":"orphan"}}),
            7 => v["host_command"] = serde_json::json!("/bin/sh"),
            8 => v["next_inode"] = serde_json::json!(1),
            _ => v["version"] = serde_json::json!(2),
        }
        assert!(decode(&v).is_err(), "change{change}");
    }
    let mut linked = s.clone();
    linked.symbolic_link("item", "sym").unwrap();
    let mut v = raw(&linked);
    v["entries"]["/home/desk/illegal-hard-sym"] = serde_json::json!(ino(&linked, "sym"));
    assert!(decode(&v).is_err());
    let text = String::from_utf8(s.to_json().unwrap()).unwrap();
    assert!(
        State::from_json(
            text.replacen("\"version\":1", "\"version\":1,\"version\":1", 1)
                .as_bytes()
        )
        .is_err()
    );
    assert!(State::from_json(text.replacen("\"/\":1", "\"/\":1,\"/\":1", 1).as_bytes()).is_err());
    assert!(State::from_json(b"{malformed").is_err());
    let mut v = raw(&s);
    v["cwd"] = serde_json::json!("/archive");
    let foreign = decode(&v).unwrap();
    assert!(!foreign.environment_matches(&s).unwrap());
}
#[test]
fn aggregate_data_entry_id_counter_output_and_serialization_limits_do_not_lose_work() {
    let mut s = State::new(&["/", "/work"], "/work").unwrap();
    for i in 0..8 {
        s.create_file(&format!("f{i}"), &"\0".repeat(MAX_FILE_BYTES))
            .unwrap();
    }
    let before = s.clone();
    assert_eq!(
        s.create_file("too-many-bytes", "x").unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(
        s.copy_file("f0", "extra").unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(
        s.write_file("f0", &"x".repeat(MAX_FILE_BYTES + 1))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(s, before);
    assert_eq!(State::from_json(&s.to_json().unwrap()).unwrap(), s); // Worst escaped data still fits the raw1MiB ceiling.
    assert_eq!(
        State::from_json(&vec![b' '; MAX_STATE_BYTES + 1])
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    let mut full = State::new(&["/", "/work"], "/work").unwrap();
    for i in 0..MAX_ENTRIES - 2 {
        full.create_file(&format!("n{i}"), "").unwrap();
    }
    let before = full.clone();
    assert_eq!(
        full.hard_link("n0", "extra").unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(full, before);
    full.move_entry("n0", "renamed").unwrap();
    assert_eq!(ino(&full, "renamed"), ino(&before, "n0"));
    assert_eq!(full.directory_entries(".").unwrap().len(), MAX_ENTRIES - 2);
    for field in ["next_inode", "revision"] {
        let mut v = raw(&s);
        v[field] = if field == "next_inode" {
            serde_json::json!(u32::MAX)
        } else {
            serde_json::json!(u64::MAX)
        };
        let mut limited = decode(&v).unwrap();
        let before = limited.clone();
        assert_eq!(
            limited.create_file("new", "").unwrap_err().kind,
            ErrorKind::Limit
        );
        assert_eq!(limited, before);
    }
}
#[test]
fn path_expansion_bounds_controls_large_targets_and_virtual_escape_inputs_are_safe() {
    let mut s = fixture();
    let before = s.clone();
    for p in [
        "a".repeat(MAX_PATH_BYTES + 1),
        "a".repeat(256),
        "x/".repeat(MAX_COMPONENTS + 1),
        "/bad\nname".into(),
        "/bad\0name".into(),
    ] {
        assert!(s.create_file(&p, "x").is_err());
        assert_eq!(s, before);
    }
    assert!(
        s.symbolic_link(&"x".repeat(MAX_PATH_BYTES + 1), "bad")
            .is_err()
    );
    assert_eq!(s, before);
    // Growing pending tails cannot allocate unboundedly before the40-hop guard.
    let expanding = format!("bomb/{}", ".".to_owned() + &"/.".repeat(62));
    s.symbolic_link(&expanding, "bomb").unwrap();
    let before = s.clone();
    let e = s.read_file("bomb").unwrap_err();
    assert!(matches!(e.kind, ErrorKind::Limit | ErrorKind::Loop));
    assert_eq!(s, before);
    assert_eq!(
        s.read_file("/../../etc/host-personal-file")
            .unwrap_err()
            .kind,
        ErrorKind::NotFound
    );
    assert!(State::new(&["/", "/no-parent/child"], "/").is_err());
    assert!(
        State::new(&["/"], "/")
            .unwrap()
            .directory_entries("/")
            .unwrap()
            .is_empty()
    );
}
