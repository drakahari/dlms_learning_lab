use dlms_learning_lab::{ldap_identity as i, ldap_query as q, ldap_utility as u};
#[path = "support/ldap_directory.rs"]
mod fixture;
use fixture::{attr, fixture};
const URI: &str = "ldap://directory.example.test";
const BASE: &str = "ou=People,dc=example,dc=test";
const ALEX: &str = "uid=alex,ou=People,dc=example,dc=test";
const SAM: &str = "uid=sam,ou=People,dc=example,dc=test";
const LEE: &str = "uid=lee,ou=Remote,ou=People,dc=example,dc=test";
const SAMPLE: &str = "Fictional-delete-only-71";
fn dn(s: &str) -> q::Dn {
    q::Dn::parse(s).unwrap()
}
fn context(source: &q::Source) -> i::Realm {
    i::Realm::new(
        URI,
        source,
        &[
            i::Identity::new(dn(ALEX), SAMPLE, dn(BASE), false, true, &[]).unwrap(),
            i::Identity::new(dn(SAM), SAMPLE, dn(BASE), false, false, &[]).unwrap(),
        ],
    )
    .unwrap()
}
fn realm() -> i::Realm {
    context(&fixture())
}
fn command(extra: &str) -> String {
    format!("ldapdelete -x -W -H {URI} -D '{ALEX}' {extra}")
}
fn exists(r: &i::Realm, name: &str) -> bool {
    r.directory()
        .read()
        .entries()
        .iter()
        .any(|e| e.dn().equivalent(&dn(name)))
}
fn run(raw: &str, r: &mut i::Realm) -> u::Report {
    u::run_delete(&command(""), SAMPLE, &u::Inputs::new(raw, &[]).unwrap(), r)
}
#[test]
fn positional_names_file_and_editor_inputs_are_distinct_and_do_not_lookup_resources() {
    let a = u::parse_delete(&command(&format!("'{SAM}' '{LEE}'"))).unwrap();
    assert_eq!(a.uri(), URI);
    assert!(a.bind().equivalent(&dn(ALEX)));
    assert_eq!(
        a.input(),
        &u::DeleteInput::Names(vec![SAM.into(), LEE.into()])
    );
    assert_eq!(
        u::parse_delete(&command("-f /work/list.dn"))
            .unwrap()
            .input(),
        &u::DeleteInput::Data(u::Input::VirtualFile("/work/list.dn".into()))
    );
    assert_eq!(
        u::parse_delete(&command("")).unwrap().input(),
        &u::DeleteInput::Data(u::Input::Editor)
    );
    assert!(
        u::parse(&command("")).is_err(),
        "The existing LDIF-only parser must retain its supported names"
    );
}
#[test]
fn option_order_end_marker_and_quoted_dn_text_are_supported_without_expansion() {
    let raw =
        format!("ldapdelete -D '{ALEX}' -H ldap://DIRECTORY.EXAMPLE.TEST:389/ -W -x -- '{SAM}'");
    let parsed = u::parse_delete(&raw).unwrap();
    assert_eq!(parsed.uri(), URI);
    assert_eq!(parsed.input(), &u::DeleteInput::Names(vec![SAM.into()]));
    for extra in [
        "-r",
        "-c",
        "-n",
        "-v",
        "-a",
        "-w not-accepted",
        "-y /work/secret",
        "-Z",
        "-ZZ",
        "-E noop",
        "--recursive",
    ] {
        assert!(
            matches!(
                u::parse_delete(&command(extra)),
                Err(q::Error::Unsupported(_))
            ),
            "{extra}"
        );
    }
    for extra in [format!("'{SAM}' -x"), format!("-f /work/list.dn '{SAM}'")] {
        assert!(u::parse_delete(&command(&extra)).is_err());
    }
}
#[test]
fn duplicate_missing_and_invalid_options_are_diagnosed_before_any_execution() {
    for extra in [
        "-x",
        "-W",
        "-D uid=other,dc=test",
        "-H ldap://other.test",
        "-f /work/a -f /work/b",
        "-f",
        "-D",
        "-H",
    ] {
        assert!(u::parse_delete(&command(extra)).is_err(), "{extra}");
    }
    for raw in [
        "ldapdelete",
        "ldapdelete -x -W",
        "ldapdelete -x -W -H ldap://directory.example.test",
        "ldapdelete -W -H ldap://directory.example.test -D uid=alex,dc=test",
        "ldapmodify -x -W",
        "sudo ldapdelete -x -W",
    ] {
        assert!(u::parse_delete(raw).is_err());
    }
    for name in [
        "/etc/passwd",
        "/work/../secret",
        "file:///work/list",
        "/work/a//b",
        "/work/.",
    ] {
        assert!(u::parse_delete(&command(&format!("-f {name}"))).is_err());
    }
}
#[test]
fn command_bytes_words_dn_count_and_component_limits_remain_bounded() {
    assert!(matches!(
        u::parse_delete(&"x".repeat(4097)),
        Err(q::Error::Limit(_))
    ));
    assert!(matches!(
        u::parse_delete(&command(&vec![SAM; 17].join(" "))),
        Err(q::Error::Limit(_))
    ));
    assert!(matches!(
        u::parse_delete(&command(&"x".repeat(1025))),
        Err(q::Error::Limit(_))
    ));
    let raw = command(&vec!["x"; 65].join(" "));
    let e = u::parse_delete(&raw).unwrap_err().to_string();
    assert!(e.contains("32"));
    assert!(!e.contains("64 command"));
}
#[test]
fn named_and_list_forms_delete_real_leaf_entries_and_preserve_other_data() {
    let mut named = realm();
    let mut listed = realm();
    let inputs = u::Inputs::new("", &[]).unwrap();
    let report = u::run_delete(
        &command(&format!("'{SAM}' '{LEE}'")),
        SAMPLE,
        &inputs,
        &mut named,
    );
    assert!(report.succeeded());
    assert_eq!(report.exit_status(), 0);
    assert_eq!(report.applied.len(), 2);
    let report = run(&format!("{SAM}\n{LEE}\n"), &mut listed);
    assert!(report.succeeded());
    assert_eq!(named.directory(), listed.directory());
    assert_eq!(listed.directory().source().entries.len(), 5);
    assert!(!exists(&listed, SAM) && !exists(&listed, LEE));
    assert!(exists(&listed, ALEX));
    assert!(exists(&listed, BASE));
}
#[test]
fn the_virtual_file_selects_its_data_and_missing_input_or_target_has_no_effect() {
    let list = format!("{SAM}\n{LEE}\n");
    let editor = "not a dn";
    let inputs = u::Inputs::new(editor, &[("/work/retire.dn", &list)]).unwrap();
    let mut r = realm();
    assert!(u::run_delete(&command("-f /work/retire.dn"), SAMPLE, &inputs, &mut r).succeeded());
    assert!(!exists(&r, SAM));
    for cmd in [
        command("-f /work/absent.dn"),
        command("").replace(URI, "ldap://not-provided.example.test"),
    ] {
        let mut r = realm();
        let before = r.directory().clone();
        let report = u::run_delete(&cmd, SAMPLE, &inputs, &mut r);
        assert!(!report.succeeded());
        assert_eq!(r.directory(), &before);
    }
}
#[test]
fn syntax_errors_are_sequential_including_positional_dn_values() {
    for (cmd, raw) in [
        (command(""), format!("{SAM}\nnot a dn\n{LEE}")),
        (
            command(&format!("'{SAM}' 'not a dn' '{LEE}'")),
            String::new(),
        ),
    ] {
        let mut r = realm();
        let report = u::run_delete(&cmd, SAMPLE, &u::Inputs::new(&raw, &[]).unwrap(), &mut r);
        assert_eq!(report.applied.len(), 1);
        assert!(!report.succeeded());
        assert!(matches!(
            report.failure,
            Some(u::Failure::Record { number: 2, .. })
        ));
        assert!(!exists(&r, SAM));
        assert!(exists(&r, LEE));
    }
}
#[test]
fn a_missing_or_duplicate_entry_stops_the_list_without_undoing_prior_success() {
    for second in [SAM, "uid=missing,ou=People,dc=example,dc=test"] {
        let mut r = realm();
        let report = run(&format!("{SAM}\n{second}\n{LEE}"), &mut r);
        assert_eq!(report.exit_status(), 32);
        assert_eq!(report.applied.len(), 1);
        assert!(matches!(
            report.failure,
            Some(u::Failure::Request { number: 2, .. })
        ));
        assert!(!exists(&r, SAM));
        assert!(exists(&r, LEE));
        let before = r.directory().clone();
        let repeat = run(&format!("{SAM}\n"), &mut r);
        assert!(!repeat.succeeded());
        assert_eq!(repeat.exit_status(), 32);
        assert_eq!(r.directory(), &before);
    }
}
#[test]
fn credentials_permissions_and_subtree_boundaries_are_checked_separately() {
    for (cmd, password, code) in [
        (command(&format!("'{SAM}'")), "wrong", 49),
        (
            command(&format!("'{LEE}'")).replace(&format!("-D '{ALEX}'"), &format!("-D '{SAM}'")),
            SAMPLE,
            50,
        ),
        (command("'ou=Teams,dc=example,dc=test'"), SAMPLE, 50),
    ] {
        let mut r = realm();
        let before = r.directory().clone();
        let report = u::run_delete(&cmd, password, &u::Inputs::new("", &[]).unwrap(), &mut r);
        assert_eq!(report.exit_status(), code);
        assert!(!report.succeeded());
        assert_eq!(r.directory(), &before);
    }
}
#[test]
fn container_deletion_is_honestly_unsupported_and_nonleaf_people_return66() {
    let mut r = realm();
    let before = r.directory().clone();
    let report = run(&format!("{BASE}\n"), &mut r);
    assert!(!report.succeeded());
    assert_eq!(report.exit_status(), 1);
    assert_eq!(r.directory(), &before);
    let mut source = fixture();
    source.entries.push(q::EntrySource {
        dn: format!("uid=kid,{SAM}"),
        attributes: vec![
            attr("uid", &["kid"]),
            attr("cn", &["Kid Person"]),
            attr("sn", &["Person"]),
            attr("objectClass", &["inetOrgPerson"]),
        ],
    });
    let mut r = context(&source);
    let before = r.directory().clone();
    let report = run(&format!("{SAM}\n"), &mut r);
    assert_eq!(report.exit_status(), 66);
    assert!(!report.succeeded());
    assert_eq!(r.directory(), &before);
}
#[test]
fn unsupported_list_formats_cannot_be_mistaken_for_ldif_or_successful_empty_work() {
    for raw in [
        "",
        "\n",
        "# comment",
        "version: 1",
        "dn: uid=sam,ou=People,dc=example,dc=test",
        &format!("{SAM}\r\n"),
    ] {
        let mut r = realm();
        let before = r.directory().clone();
        let report = run(raw, &mut r);
        assert!(!report.succeeded(), "{raw:?}");
        assert_eq!(report.exit_status(), 1);
        assert_eq!(r.directory(), &before);
    }
    let mut r = realm();
    let report = run(&format!("{SAM}\n\n{LEE}"), &mut r);
    assert_eq!(report.applied.len(), 1);
    assert!(!exists(&r, SAM));
    assert!(exists(&r, LEE));
}
#[test]
fn line_limits_preflight_and_request_limits_preserve_truthful_partial_state() {
    let mut r = realm();
    let before = r.directory().clone();
    let report = run(&"\n".repeat(1025), &mut r);
    assert!(!report.succeeded());
    assert!(report.applied.is_empty());
    assert_eq!(r.directory(), &before);
    let mut source = fixture();
    let mut names = vec![];
    for n in 0..17 {
        let uid = format!("retired{n}");
        let name = format!("uid={uid},{BASE}");
        source.entries.push(q::EntrySource {
            dn: name.clone(),
            attributes: vec![
                attr("uid", &[&uid]),
                attr("cn", &["Retired Person"]),
                attr("sn", &["Person"]),
                attr("objectClass", &["inetOrgPerson"]),
            ],
        });
        names.push(name);
    }
    let mut r = context(&source);
    let report = run(&names.join("\n"), &mut r);
    assert_eq!(report.applied.len(), 16);
    assert!(matches!(
        report.failure,
        Some(u::Failure::Record { number: 17, .. })
    ));
    assert!(!report.succeeded());
    assert!(exists(&r, &names[16]));
    assert!(exists(&r, ALEX));
}
#[test]
fn shell_escape_attempts_have_no_effect_and_the_existing_ldif_runner_stays_closed() {
    for extra in ["; id", "$(touch /tmp/no)", "| sh", "> /work/out", "`id`"] {
        let mut r = realm();
        let before = r.directory().clone();
        let report = u::run_delete(
            &command(extra),
            SAMPLE,
            &u::Inputs::new(SAM, &[]).unwrap(),
            &mut r,
        );
        assert!(!report.succeeded());
        assert_eq!(r.directory(), &before);
    }
    let mut r = realm();
    let before = r.directory().clone();
    let report = u::run(
        &command(&format!("'{SAM}'")),
        SAMPLE,
        &u::Inputs::new("", &[]).unwrap(),
        &mut r,
    );
    assert!(!report.succeeded());
    assert_eq!(r.directory(), &before);
}
#[test]
fn truncated_literal_commands_are_bounded_and_reports_do_not_echo_sample_passwords() {
    let cmd = command(&format!("'{SAM}'"));
    for end in 0..cmd.len() {
        let _ = u::parse_delete(&cmd[..end]);
    }
    let mut r = realm();
    let report = u::run_delete(
        &cmd,
        "wrong-training-value",
        &u::Inputs::new("", &[]).unwrap(),
        &mut r,
    );
    let rendered = format!("{report:?}");
    assert!(!rendered.contains(SAMPLE));
    assert!(!rendered.contains("wrong-training-value"));
}
