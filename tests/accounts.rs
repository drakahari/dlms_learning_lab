//! Synthetic data only. Never invokes host account utilities or reads host account files.
use dlms_learning_lab::accounts::{
    self, Configuration, Entry, EpochDay, ErrorKind, Group, Kind, Policy, Requirement, State, User,
};
use std::collections::{BTreeMap, BTreeSet};
fn strings(a: &[&str]) -> BTreeSet<String> {
    a.iter().map(|s| s.to_string()).collect()
}
fn dir(uid: u32, gid: u32, mode: u16) -> Entry {
    Entry {
        uid,
        gid,
        mode,
        data: Kind::Directory,
    }
}
fn file(uid: u32, gid: u32, mode: u16, s: &str) -> Entry {
    Entry {
        uid,
        gid,
        mode,
        data: Kind::Text { contents: s.into() },
    }
}
fn user(uid: u32, primary_gid: u32, groups: &[u32], home: &str) -> User {
    User {
        uid,
        primary_gid,
        supplementary: groups.iter().copied().collect(),
        home: home.into(),
        shell: "/bin/bash".into(),
        comment: String::new(),
        expires: None,
    }
}
fn fixture() -> State {
    let users = BTreeMap::from([
        ("root".into(), user(0, 0, &[], "/root")),
        (
            "archive-service".into(),
            User {
                shell: "/usr/sbin/nologin".into(),
                ..user(500, 2003, &[], "/srv/archive")
            },
        ),
        ("rin".into(), user(1101, 1000, &[2001, 2002], "/home/rin")),
        ("casey".into(), user(1102, 1000, &[2000], "/home/casey")),
        ("retire".into(), user(1200, 2002, &[2000], "/home/retire")),
    ]);
    let groups = [
        ("root", 0),
        ("staff", 1000),
        ("review", 2000),
        ("audit", 2001),
        ("old-project", 2002),
        ("archive", 2003),
    ]
    .into_iter()
    .map(|(n, gid)| (n.into(), Group { gid }))
    .collect();
    let mut entries = BTreeMap::new();
    for (p, uid, gid) in [
        ("/", 0, 0),
        ("/root", 0, 0),
        ("/etc", 0, 0),
        ("/home", 0, 0),
        ("/srv", 0, 0),
        ("/srv/archive", 500, 2003),
        ("/archive", 0, 0),
        ("/etc/skel", 0, 0),
        ("/etc/skel/notes", 0, 0),
        ("/etc/review-skel", 0, 0),
        ("/home/rin", 1101, 1000),
        ("/home/casey", 1102, 1000),
        ("/home/retire", 1200, 2002),
    ] {
        entries.insert(p.into(), dir(uid, gid, 0o700));
    }
    for (p, e) in [
        (
            "/etc/skel/.profile",
            file(0, 0, 0o644, "supplied profile\n"),
        ),
        (
            "/etc/skel/notes/private.txt",
            file(0, 0, 0o600, "private starter\n"),
        ),
        (
            "/etc/review-skel/.profile",
            file(0, 0, 0o640, "review profile\n"),
        ),
        (
            "/home/rin/old.txt",
            file(1101, 1000, 0o640, "Rin evidence\n"),
        ),
        (
            "/home/rin/other.txt",
            file(1102, 1000, 0o640, "other UID; old home GID\n"),
        ),
        (
            "/home/rin/custom.txt",
            file(1101, 2001, 0o600, "different group\n"),
        ),
        (
            "/home/casey/evidence.txt",
            file(1102, 1000, 0o600, "Casey unchanged\n"),
        ),
        (
            "/home/retire/notes.txt",
            file(1200, 2002, 0o600, "contract notes\n"),
        ),
        (
            "/archive/receipt.txt",
            file(1200, 2002, 0o440, "signed outside handover\n"),
        ),
        (
            "/archive/rin.txt",
            file(1101, 1000, 0o440, "outside primary group retained\n"),
        ),
    ] {
        entries.insert(p.into(), e);
    }
    let policy = Policy {
        fixture: "account-review".into(),
        today: EpochDay::parse("2026-10-23").unwrap(),
        protected_users: strings(&["root", "archive-service"]),
        protected_groups: strings(&["root", "archive"]),
        protected_entries: strings(&[
            "/",
            "/etc",
            "/home",
            "/root",
            "/srv",
            "/srv/archive",
            "/archive",
        ]),
        skeletons: strings(&["/etc/skel", "/etc/review-skel"]),
        shells: strings(&["/bin/bash", "/usr/sbin/nologin"]),
    };
    State::new(
        policy,
        Configuration {
            users,
            groups,
            entries,
        },
    )
    .unwrap()
}
fn run(s: &mut State, commands: &[&str]) {
    for c in commands {
        let out = s.execute(c).unwrap_or_else(|e| panic!("{c}: {e}"));
        assert_eq!(out.status, 0);
    }
}
fn rejected(s: &mut State, command: &str, kind: ErrorKind) {
    let before = s.clone();
    assert_eq!(s.execute(command).unwrap_err().kind, kind, "{command}");
    assert_eq!(*s, before, "failed command must be atomic: {command}");
}
fn all_pass(initial: &State, s: &State, r: &[Requirement]) -> bool {
    accounts::checks(initial, s, r).iter().all(|(ok, _)| *ok)
}

#[test]
fn onboarding_uses_stable_explicit_ids_and_copies_owned_skeleton_without_modifying_it() {
    let mut s = fixture();
    let initial = s.clone();
    run(
        &mut s,
        &[
            "groupadd -g 3000 project",
            "useradd -N -u 1300 -g staff -G project -d /home/sol -s /bin/bash -m -k /etc/review-skel sol",
        ],
    );
    let u = &s.configuration().users["sol"];
    assert_eq!((u.uid, u.primary_gid, u.expires), (1300, 1000, None));
    assert_eq!(u.supplementary, BTreeSet::from([3000]));
    assert_eq!(
        s.configuration().entries["/home/sol"],
        dir(1300, 1000, 0o700)
    );
    assert_eq!(
        s.configuration().entries["/home/sol/.profile"],
        file(1300, 1000, 0o640, "review profile\n")
    );
    assert_eq!(
        s.configuration().entries["/etc/review-skel/.profile"],
        initial.configuration().entries["/etc/review-skel/.profile"]
    );
    for (name, u) in &initial.configuration().users {
        assert_eq!(s.configuration().users[name], *u);
    }
    assert!(s.valid_against(&initial));
}
#[test]
fn explicit_option_orders_and_long_forms_reach_the_same_configuration() {
    let mut a = fixture();
    let mut b = fixture();
    run(
        &mut a,
        &[
            "groupadd -g 3000 project",
            "useradd -N -u 1300 -g staff -G project,audit -d /home/sol -s /bin/bash -m sol",
        ],
    );
    run(
        &mut b,
        &[
            "groupadd --gid 3000 project",
            "useradd --shell /bin/bash --create-home --home-dir /home/sol --groups audit,project,audit --gid 1000 --uid 1300 --no-user-group sol",
        ],
    );
    assert_eq!(a, b);
}
#[test]
fn metadata_only_creation_has_no_home_and_no_implicit_private_group() {
    let mut s = fixture();
    run(
        &mut s,
        &["useradd -N -u 1300 -g staff -d /home/sol -s /bin/bash -M sol"],
    );
    assert!(s.configuration().users.contains_key("sol"));
    assert!(!s.configuration().entries.contains_key("/home/sol"));
    assert!(!s.configuration().groups.contains_key("sol"));
}
#[test]
fn append_and_replacement_are_independent_of_primary_membership() {
    let mut s = fixture();
    run(&mut s, &["usermod -aG review rin"]);
    assert_eq!(
        s.configuration().users["rin"].supplementary,
        BTreeSet::from([2000, 2001, 2002])
    );
    run(&mut s, &["usermod -G review,audit,review rin"]);
    assert_eq!(
        s.configuration().users["rin"].supplementary,
        BTreeSet::from([2000, 2001])
    );
    run(&mut s, &["usermod -G '' rin"]);
    assert!(s.configuration().users["rin"].supplementary.is_empty());
    assert_eq!(s.configuration().users["rin"].primary_gid, 1000);
    assert!(
        s.execute("id rin")
            .unwrap()
            .stdout
            .contains("gid=1000(staff) groups=1000(staff)")
    );
}
#[test]
fn group_database_does_not_invent_primary_members_and_id_deduplicates() {
    let mut s = fixture();
    assert_eq!(
        s.execute("getent group staff").unwrap().stdout,
        "staff:x:1000:\n"
    );
    run(&mut s, &["usermod -a -G staff rin"]);
    assert_eq!(
        s.execute("getent group 1000").unwrap().stdout,
        "staff:x:1000:rin\n"
    );
    assert_eq!(
        s.execute("id -Gn rin").unwrap().stdout,
        "staff audit old-project\n"
    );
    assert_eq!(
        s.execute("id -nG rin").unwrap().stdout,
        "staff audit old-project\n"
    );
}
#[test]
fn primary_gid_change_updates_old_group_inside_home_including_other_owners_only() {
    let mut s = fixture();
    let before = s.configuration().clone();
    run(&mut s, &["usermod -g review rin"]);
    assert_eq!(s.configuration().users["rin"].primary_gid, 2000);
    for p in ["/home/rin", "/home/rin/old.txt", "/home/rin/other.txt"] {
        assert_eq!(s.configuration().entries[p].gid, 2000);
    }
    for p in [
        "/home/rin/custom.txt",
        "/archive/rin.txt",
        "/home/casey/evidence.txt",
    ] {
        assert_eq!(s.configuration().entries[p], before.entries[p]);
    }
    assert_eq!(s.configuration().entries["/home/rin/other.txt"].uid, 1102);
}
#[test]
fn foreign_owned_home_group_change_and_removal_are_honestly_unsupported() {
    let s = fixture();
    let mut c = s.configuration().clone();
    c.entries.get_mut("/home/rin").unwrap().uid = 1102;
    let mut s = State::new(s.policy().clone(), c).unwrap();
    rejected(&mut s, "usermod -g review rin", ErrorKind::Unsupported);
    rejected(&mut s, "userdel -r rin", ErrorKind::Unsupported);
}
#[test]
fn group_rename_preserves_gid_membership_and_file_identity() {
    let mut s = fixture();
    let before = s.configuration().clone();
    run(&mut s, &["groupmod -n reviews review"]);
    assert!(!s.configuration().groups.contains_key("review"));
    assert_eq!(s.configuration().groups["reviews"].gid, 2000);
    assert_eq!(s.configuration().users, before.users);
    assert_eq!(s.configuration().entries, before.entries);
    assert!(
        s.execute("id casey")
            .unwrap()
            .stdout
            .contains("2000(reviews)")
    );
}
#[test]
fn primary_group_deletion_is_refused_then_lifecycle_cleanup_can_succeed() {
    let mut s = fixture();
    rejected(&mut s, "groupdel old-project", ErrorKind::Conflict);
    run(&mut s, &["userdel retire", "groupdel old-project"]);
    assert!(!s.configuration().groups.contains_key("old-project"));
    assert!(!s.configuration().users["rin"].supplementary.contains(&2002));
    assert_eq!(s.configuration().entries["/archive/receipt.txt"].gid, 2002);
}
#[test]
fn deletion_retains_numeric_ownership_and_never_auto_deletes_primary_group() {
    let mut s = fixture();
    let old = s.configuration().entries.clone();
    run(&mut s, &["userdel retire"]);
    assert!(!s.configuration().users.contains_key("retire"));
    assert_eq!(s.configuration().entries, old);
    assert!(s.configuration().groups.contains_key("old-project"));
    assert!(
        s.execute("ls -l /archive")
            .unwrap()
            .stdout
            .contains("1200 old-project")
    );
    rejected(
        &mut s,
        "useradd -N -u 1200 -g staff -d /home/new -s /bin/bash -M new",
        ErrorKind::Conflict,
    );
}
#[test]
fn retirement_handover_and_whole_configuration_preservation_have_multiple_valid_orders() {
    let initial = fixture();
    let mut a = initial.clone();
    let mut b = initial.clone();
    run(
        &mut a,
        &[
            "chown archive-service:archive /archive/receipt.txt",
            "userdel -r retire",
            "groupdel old-project",
            "useradd -N -u 1300 -g staff -d /home/new -s /bin/bash -m new",
        ],
    );
    run(
        &mut b,
        &[
            "useradd -N -u 1300 -g staff -d /home/new -s /bin/bash -m new",
            "userdel -r retire",
            "chown archive-service:archive /archive/receipt.txt",
            "groupdel old-project",
        ],
    );
    assert_eq!(a, b);
    assert_eq!(
        a.configuration().entries["/archive/receipt.txt"],
        file(500, 2003, 0o440, "signed outside handover\n")
    );
    assert!(
        !a.configuration()
            .entries
            .keys()
            .any(|p| p.starts_with("/home/retire"))
    );
    let r = vec![Requirement::Configuration {
        expected: a.configuration().clone(),
    }];
    assert!(accounts::valid_requirements(&initial, &r));
    assert!(all_pass(&initial, &a, &r));
    assert!(!all_pass(&initial, &initial, &r));
    let mut wrong = a.clone();
    run(&mut wrong, &["usermod -G '' casey"]);
    assert!(!all_pass(&initial, &wrong, &r));
}
#[test]
fn root_service_records_skeleton_and_protected_entries_cannot_be_changed() {
    let mut s = fixture();
    for command in [
        "userdel root",
        "userdel -r archive-service",
        "usermod -e 1 root",
        "usermod -G '' archive-service",
        "groupmod -n admins root",
        "groupdel archive",
        "chown rin:staff /etc/skel/.profile",
        "chown rin:staff /",
    ] {
        rejected(&mut s, command, ErrorKind::Protected);
    }
}
#[test]
fn expiry_is_exact_metadata_in_three_relations_and_can_be_cleared_without_authentication_claims() {
    let mut s = fixture();
    let before = s.configuration().clone();
    for (date, relation) in [
        ("2026-10-22", "before"),
        ("2026-10-23", "on"),
        ("2026-10-24", "after"),
    ] {
        run(&mut s, &[&format!("usermod -e {date} rin")]);
        assert_eq!(s.configuration().users["rin"].expires.unwrap().iso(), date);
        assert!(
            s.execute("show")
                .unwrap()
                .stdout
                .contains(&format!("{relation} supplied date"))
        );
        let shadow = s.execute("getent shadow rin").unwrap().stdout;
        assert_eq!(shadow.trim_end().split(':').count(), 9);
        assert_eq!(shadow.split(':').nth(1), Some("!"));
    }
    run(&mut s, &["usermod -e -1 rin"]);
    assert_eq!(s.configuration(), &before);
    run(&mut s, &["usermod -e 1 rin", "usermod -e \"\" rin"]);
    assert_eq!(s.configuration(), &before);
    assert!(!s.execute("show").unwrap().stdout.contains("can log in"));
}
#[test]
fn original_gregorian_math_handles_leaps_centuries_endpoints_and_unambiguous_numeric_days() {
    for (date, day) in [
        ("1970-01-02", 1),
        ("2000-02-29", 11016),
        ("2024-02-29", 19782),
        ("2026-10-23", 20749),
        ("2100-03-01", 47541),
        ("9999-12-31", 2932896),
    ] {
        let d = EpochDay::parse(date).unwrap();
        assert_eq!(d.number(), day);
        assert_eq!(d.iso(), date);
        assert_eq!(EpochDay::parse(&day.to_string()).unwrap(), d);
    }
    for bad in [
        "0",
        "00",
        "01",
        "-1",
        "2932897",
        "4294967296",
        "1970-01-01",
        "1969-12-31",
        "10000-01-01",
        "2023-02-29",
        "2100-02-29",
        "2000-13-01",
        "2000-00-01",
        "2000-01-00",
        "2000-04-31",
        "2024-2-29",
        "ééééé",
        "2024/02/29",
        "2024-02-29Z",
    ] {
        assert!(EpochDay::parse(bad).is_err(), "{bad}");
    }
}
#[test]
fn date_and_command_errors_are_atomic_and_unsupported_language_never_executes() {
    let mut s = fixture();
    for date in ["0", "01", "2026-02-30", "2932897"] {
        rejected(
            &mut s,
            &format!("usermod -e {date} rin"),
            ErrorKind::Invalid,
        );
    }
    for command in [
        "passwd rin",
        "usermod -L rin",
        "usermod -U rin",
        "userdel -f rin",
        "groupadd -g 01 project",
        "id rin; userdel rin",
        "id $(printf rin)",
        "getent passwd | cat",
        "cat /etc/passwd > copy",
        "useradd -u 1300 -g staff -d /home/new -s /bin/bash -m new",
        "useradd -N -u 1300 -g staff -d /home/new -s /bin/bash -M -k /etc/skel new",
        "usermod -G audit -G review rin",
    ] {
        let before = s.clone();
        assert!(s.execute(command).is_err(), "{command}");
        assert_eq!(s, before);
    }
    assert!(
        s.execute("getent passwd")
            .unwrap()
            .stdout
            .contains("rin:x:1101:1000:")
    );
}
#[test]
fn missing_or_colliding_targets_never_leave_half_created_records() {
    let mut s = fixture();
    for command in [
        "useradd -N -u 1300 -g missing -d /home/new -s /bin/bash -m new",
        "useradd -N -u 1300 -g staff -G missing -d /home/new -s /bin/bash -m new",
        "useradd -N -u 1300 -g staff -d /home/new -s /bin/bash -m -k /etc/missing new",
        "groupadd -g 1000 new",
        "groupmod -n staff review",
        "usermod -aG missing rin",
        "chown rin:missing /archive/receipt.txt",
    ] {
        let before = s.clone();
        assert!(s.execute(command).is_err());
        assert_eq!(s, before);
    }
    let s0 = fixture();
    let mut c = s0.configuration().clone();
    c.entries.insert("/home/new".into(), dir(1300, 1000, 0o700));
    let mut s = State::new(s0.policy().clone(), c).unwrap();
    rejected(
        &mut s,
        "useradd -N -u 1400 -g staff -d /home/new -s /bin/bash -m new",
        ErrorKind::Unsupported,
    );
}
#[test]
fn queries_are_read_only_local_and_shadow_has_only_a_noncredential_sentinel() {
    let mut s = fixture();
    let before = s.clone();
    for command in [
        "getent passwd",
        "getent passwd rin",
        "getent passwd 1101",
        "getent group audit",
        "getent shadow",
        "cat /etc/passwd",
        "cat /etc/group",
        "cat /etc/shadow",
        "id",
        "ls -ld /home/rin",
        "ls -ln /archive",
        "cat /archive/receipt.txt",
        "show",
    ] {
        assert_eq!(s.execute(command).unwrap().status, 0);
        assert_eq!(s, before);
    }
    assert_eq!(s.execute("getent passwd absent").unwrap().status, 2);
    assert_eq!(s.execute("getent group 9999").unwrap().stdout, "");
    for line in s.execute("getent shadow").unwrap().stdout.lines() {
        assert_eq!(line.split(':').count(), 9);
        assert_eq!(line.split(':').nth(1), Some("!"));
    }
}
#[test]
fn serialized_state_pins_protected_records_and_does_not_trust_imported_policy() {
    let initial = fixture();
    let mut v = serde_json::to_value(&initial).unwrap();
    v["policy"]["fixture"] = "different".into();
    let changed: State = serde_json::from_value(v).unwrap();
    assert!(!changed.valid_against(&initial));
    let mut v = serde_json::to_value(&initial).unwrap();
    v["configuration"]["users"]["archive-service"]["comment"] = "changed".into();
    let changed: State = serde_json::from_value(v).unwrap();
    assert!(!changed.valid_against(&initial));
    let mut v = serde_json::to_value(&initial).unwrap();
    v["configuration"]["users"]["rin"]["uid"] = 9999.into();
    let changed: State = serde_json::from_value(v).unwrap();
    assert!(!changed.valid_against(&initial));
}
#[test]
fn strict_decoding_rejects_duplicate_maps_sets_fields_unknowns_and_invalid_links() {
    let s = fixture();
    let text = serde_json::to_string(&s).unwrap();
    assert!(State::from_json(&text.replacen("\"rin\":{", "\"rin\":{\"uid\":1101,", 1)).is_err());
    assert!(
        State::from_json(&text.replacen(
            "\"staff\":{\"gid\":1000}",
            "\"staff\":{\"gid\":1000},\"staff\":{\"gid\":1001}",
            1
        ))
        .is_err()
    );
    assert!(
        State::from_json(&text.replacen(
            "\"supplementary\":[2001,2002]",
            "\"supplementary\":[2001,2001]",
            1
        ))
        .is_err()
    );
    for (target, value) in [
        ("uid", serde_json::json!(0)),
        ("primary_gid", serde_json::json!(9999)),
        ("home", serde_json::json!("/home/../rin")),
        ("shell", serde_json::json!("/bin/missing")),
        ("comment", serde_json::json!("colon:injection")),
        ("expires", serde_json::json!(0)),
    ] {
        let mut v = serde_json::to_value(&s).unwrap();
        v["configuration"]["users"]["rin"][target] = value;
        assert!(serde_json::from_value::<State>(v).is_err(), "{target}");
    }
    let mut v = serde_json::to_value(&s).unwrap();
    v["unexpected"] = true.into();
    assert!(serde_json::from_value::<State>(v).is_err());
}
#[test]
fn bounded_decoders_reject_oversized_keys_values_and_excess_records() {
    let s = fixture();
    let mut v = serde_json::to_value(&s).unwrap();
    v["configuration"]["entries"]["/archive/receipt.txt"]["data"]["contents"] =
        "x".repeat(accounts::MAX_FILE_BYTES + 1).into();
    assert!(serde_json::from_value::<State>(v).is_err());
    let mut v = serde_json::to_value(&s).unwrap();
    let users = v["configuration"]["users"].as_object_mut().unwrap();
    for i in 0..33 {
        users.insert(format!("new{i}"),serde_json::json!({"uid":3000+i,"primary_gid":1000,"supplementary":[],"home":format!("/home/new{i}"),"shell":"/bin/bash","comment":"","expires":null}));
    }
    assert!(serde_json::from_value::<State>(v).is_err());
    assert!(State::from_json(&" ".repeat(accounts::MAX_STATE_BYTES + 1)).is_err());
    assert!(accounts::parse(&"x".repeat(2049)).is_err());
    assert!(accounts::parse(&vec!["id"; 33].join(" ")).is_err());
}
#[test]
fn maximum_shape_and_worst_legal_escaping_roundtrip_compact_and_pretty() {
    let s = fixture();
    let mut c = s.configuration().clone();
    let keys: Vec<_> = c
        .entries
        .iter()
        .filter(|(_, e)| !e.directory())
        .map(|(p, _)| p.clone())
        .collect();
    for p in keys {
        c.entries.remove(&p);
    }
    for i in 0..4 {
        c.entries.insert(
            format!("/archive/big{i}"),
            file(0, 0, 0o600, &"\u{0001}".repeat(accounts::MAX_FILE_BYTES)),
        );
    }
    while c.entries.len() < accounts::MAX_ENTRIES {
        c.entries.insert(
            format!("/archive/{}{:03}", "p".repeat(108), c.entries.len()),
            dir(0, 0, 0o700),
        );
    }
    while c.groups.len() < accounts::MAX_GROUPS {
        let i = c.groups.len();
        c.groups.insert(
            format!("group{i}"),
            Group {
                gid: 3000 + i as u32,
            },
        );
    }
    let gids: Vec<_> = c
        .groups
        .values()
        .filter(|g| g.gid >= 3000)
        .take(16)
        .map(|g| g.gid)
        .collect();
    while c.users.len() < accounts::MAX_USERS {
        let i = c.users.len();
        let mut u = user(3000 + i as u32, 1000, &gids, &format!("/home/user{i}"));
        u.comment = "\\".repeat(80);
        c.users.insert(format!("user{i}"), u);
    }
    let s = State::new(s.policy().clone(), c).unwrap();
    for text in [
        serde_json::to_string(&s).unwrap(),
        serde_json::to_string_pretty(&s).unwrap(),
    ] {
        assert!(text.len() > 1_500_000);
        assert!(text.len() < accounts::MAX_STATE_BYTES);
        assert_eq!(State::from_json(&text).unwrap(), s);
    }
}
#[test]
fn bounded_query_output_and_post_copy_state_overflow_fail_without_mutation() {
    let s = fixture();
    let mut c = s.configuration().clone();
    c.entries.insert(
        "/archive/large".into(),
        file(0, 0, 0o600, &"x".repeat(accounts::MAX_OUTPUT_BYTES + 1)),
    );
    let mut s = State::new(s.policy().clone(), c).unwrap();
    let before = s.clone();
    assert_eq!(
        s.execute("cat /archive/large").unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(s, before);
    let mut c = s.configuration().clone();
    while c.entries.len() < accounts::MAX_ENTRIES {
        c.entries
            .insert(format!("/archive/f{}", c.entries.len()), dir(0, 0, 0o700));
    }
    let mut s = State::new(s.policy().clone(), c).unwrap();
    let before = s.clone();
    assert!(
        s.execute("useradd -N -u 1300 -g staff -d /home/new -s /bin/bash -m new")
            .is_err()
    );
    assert_eq!(s, before);
}
#[test]
fn current_outcomes_reject_wrong_peer_partial_work_wrong_id_and_unrelated_damage() {
    let initial = fixture();
    let mut good = initial.clone();
    run(
        &mut good,
        &["usermod -G audit,review rin", "usermod -e 1 rin"],
    );
    let requirements = vec![
        Requirement::User {
            name: "rin".into(),
            expected: good.configuration().users["rin"].clone(),
        },
        Requirement::Configuration {
            expected: good.configuration().clone(),
        },
    ];
    assert!(all_pass(&initial, &good, &requirements));
    assert!(!all_pass(&initial, &initial, &requirements));
    let mut partial = initial.clone();
    run(&mut partial, &["usermod -G audit,review rin"]);
    assert!(!all_pass(&initial, &partial, &requirements));
    let mut wrong = initial.clone();
    run(&mut wrong, &["usermod -e 1 casey"]);
    assert!(!all_pass(&initial, &wrong, &requirements));
    let mut damaged = good.clone();
    run(&mut damaged, &["chown rin:staff /archive/receipt.txt"]);
    assert!(!all_pass(&initial, &damaged, &requirements));
    let mut invalid = requirements.clone();
    if let Requirement::User { expected, .. } = &mut invalid[0] {
        expected.uid = 9999;
    }
    assert!(!accounts::valid_requirements(&initial, &invalid));
}
#[test]
fn absence_expiry_range_and_requirements_json_are_typed_current_checks() {
    let initial = fixture();
    let mut s = initial.clone();
    run(
        &mut s,
        &[
            "usermod -e 2 rin",
            "userdel -r retire",
            "groupdel old-project",
        ],
    );
    let rs = vec![
        Requirement::UserAbsent {
            name: "retire".into(),
            uid: 1200,
        },
        Requirement::GroupAbsent {
            name: "old-project".into(),
            gid: 2002,
        },
        Requirement::AbsentPath {
            path: "/home/retire".into(),
        },
        Requirement::ExpiryRange {
            name: "rin".into(),
            first: EpochDay::parse("1").unwrap(),
            last: EpochDay::parse("2").unwrap(),
        },
    ];
    assert!(all_pass(&initial, &s, &rs));
    assert!(!all_pass(&initial, &initial, &rs));
    assert_eq!(
        accounts::requirements_from_json(&initial, &serde_json::to_string(&rs).unwrap()).unwrap(),
        rs
    );
    run(&mut s, &["usermod -e 3 rin"]);
    assert!(!all_pass(&initial, &s, &rs));
    let oversized = vec![rs[0].clone(); accounts::MAX_REQUIREMENTS + 1];
    assert!(
        accounts::requirements_from_json(&initial, &serde_json::to_string(&oversized).unwrap())
            .is_err()
    );
}

#[test]
fn directly_constructed_oversized_configuration_outcomes_are_rejected_before_bounded_clone() {
    let initial = fixture();
    let before = initial.clone();
    let mut count = initial.configuration().clone();
    while count.entries.len() <= accounts::MAX_ENTRIES {
        count.entries.insert(
            format!("/archive/extra{}", count.entries.len()),
            dir(0, 0, 0o700),
        );
    }
    let mut individual = initial.configuration().clone();
    individual.entries.insert(
        "/archive/oversized".into(),
        file(0, 0, 0o600, &"x".repeat(accounts::MAX_FILE_BYTES + 1)),
    );
    let mut aggregate = initial.configuration().clone();
    for i in 0..5 {
        aggregate.entries.insert(
            format!("/archive/large{i}"),
            file(0, 0, 0o600, &"x".repeat(accounts::MAX_FILE_BYTES)),
        );
    }
    let mut users = initial.configuration().clone();
    for i in 0..accounts::MAX_USERS {
        users.users.insert(
            format!("added{i}"),
            user(3000 + i as u32, 1000, &[], &format!("/home/added{i}")),
        );
    }
    let mut groups = initial.configuration().clone();
    for i in 0..accounts::MAX_GROUPS {
        groups.groups.insert(
            format!("added{i}"),
            Group {
                gid: 3000 + i as u32,
            },
        );
    }
    for expected in [count, individual, aggregate, users, groups] {
        let rs = [Requirement::Configuration { expected }];
        assert!(!accounts::valid_requirements(&initial, &rs));
        assert!(!all_pass(&initial, &initial, &rs));
        assert_eq!(initial, before);
    }
}

#[test]
fn intrinsic_lifecycle_and_fixed_context_pins_are_separate_and_adapter_preflight_keeps_valid_state()
{
    let initial = fixture();
    let mut current = initial.clone();
    run(&mut current, &["userdel -r rin"]);
    assert!(current.valid_against(&initial));
    let before = current.clone();
    let mut candidate = current.clone();
    // A standalone account database can create the now-free name at a new UID.
    run(
        &mut candidate,
        &["useradd -N -u 1300 -g staff -d /home/rin -s /bin/bash -m rin"],
    );
    assert!(candidate.valid());
    assert!(!candidate.valid_against(&initial));
    // Future fixed-fixture adapters execute on a bounded candidate and publish
    // only after BOTH intrinsic execution and the original context check pass.
    if candidate.valid_against(&initial) {
        current = candidate;
    }
    assert_eq!(current, before);
    assert!(current.valid_against(&initial));
}

#[test]
fn unknown_field_and_variant_errors_keep_a_small_utf8_safe_message_and_original_data() {
    let initial = fixture();
    let before = serde_json::to_vec(&initial).unwrap();
    let large_name = "é".repeat(40_000);
    let encoded_name = serde_json::to_string(&large_name).unwrap();
    let unknown_state_field = format!("{{{encoded_name}:true}}");
    let mut state_variant = serde_json::to_value(&initial).unwrap();
    state_variant["configuration"]["entries"]["/archive/receipt.txt"]["data"]["kind"] =
        large_name.clone().into();
    let unknown_state_variant = serde_json::to_string(&state_variant).unwrap();
    let unknown_outcome_field = format!("[{{\"kind\":\"ExpiryRange\",{encoded_name}:true}}]");
    let unknown_outcome_variant = format!("[{{\"kind\":{encoded_name}}}]");
    for (text, outcome, context, detail) in [
        (
            unknown_state_field,
            false,
            "Invalid account JSON:",
            "unknown field",
        ),
        (
            unknown_state_variant,
            false,
            "Invalid account JSON:",
            "unknown variant",
        ),
        (
            unknown_outcome_field,
            true,
            "Invalid account outcomes:",
            "unknown field",
        ),
        (
            unknown_outcome_variant,
            true,
            "Invalid account outcomes:",
            "unknown variant",
        ),
    ] {
        assert!(text.len() < accounts::MAX_STATE_BYTES);
        let original_input = text.clone();
        let error = if outcome {
            accounts::requirements_from_json(&initial, &text).unwrap_err()
        } else {
            State::from_json(&text).unwrap_err()
        };
        assert_eq!(error.kind, ErrorKind::Invalid);
        assert!(error.message.starts_with(context), "{}", error.message);
        assert!(error.message.contains(detail), "{}", error.message);
        assert!(error.message.len() <= accounts::MAX_ERROR_BYTES);
        assert!(error.message.capacity() <= accounts::MAX_ERROR_BYTES);
        assert!(error.message.ends_with('…'));
        assert_eq!(text, original_input);
        assert_eq!(serde_json::to_vec(&initial).unwrap(), before);
    }
}
