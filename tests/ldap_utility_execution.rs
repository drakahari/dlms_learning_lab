use dlms_learning_lab::{ldap_identity as i, ldap_ldif as l, ldap_query as q, ldap_utility as u};
#[path = "support/ldap_directory.rs"]
mod fixture;
use fixture::{attr, fixture};
const URI: &str = "ldap://directory.example.test";
const BASE: &str = "ou=People,dc=example,dc=test";
const ALEX: &str = "uid=alex,ou=People,dc=example,dc=test";
const SAM: &str = "uid=sam,ou=People,dc=example,dc=test";
const SAMPLE: &str = "Only-a-training-value-52";
fn dn(raw: &str) -> q::Dn {
    q::Dn::parse(raw).unwrap()
}
fn realm() -> i::Realm {
    i::Realm::new(
        URI,
        &fixture(),
        &[
            i::Identity::new(
                dn(ALEX),
                SAMPLE,
                dn(BASE),
                true,
                true,
                &["mail", "description"],
            )
            .unwrap(),
            i::Identity::new(dn(SAM), SAMPLE, dn(BASE), false, false, &["description"]).unwrap(),
        ],
    )
    .unwrap()
}
fn command(verb: &str, principal: &str, extra: &str) -> String {
    format!("{verb} -x -W -H {URI} -D '{principal}' {extra}")
}
fn modify(target: &str, value: &str) -> String {
    format!("dn: {target}\nchangetype: modify\nreplace: description\ndescription: {value}\n-\n")
}
fn field(r: &i::Realm, target: &str, name: &str) -> Vec<String> {
    r.directory()
        .source()
        .entries
        .iter()
        .find(|e| dn(&e.dn).equivalent(&dn(target)))
        .unwrap()
        .attributes
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case(name))
        .map_or_else(Vec::new, |a| a.values.clone())
}
fn run(r: &mut i::Realm, raw: &str) -> u::Report {
    u::run(
        &command("ldapmodify", ALEX, ""),
        SAMPLE,
        &u::Inputs::new(raw, &[]).unwrap(),
        r,
    )
}
#[test]
fn commands_execute_real_typed_changes_and_return_a_truthful_exit_status() {
    let mut r = realm();
    let raw = format!("version: 1\n{}", modify(ALEX, "verified"));
    let report = run(&mut r, &raw);
    assert!(report.succeeded());
    assert_eq!(report.exit_status(), 0);
    assert_eq!(report.applied.len(), 1);
    assert_eq!(field(&r, ALEX, "description"), ["verified"]);
    assert_eq!(
        field(&r, ALEX, "mail"),
        ["alex@example.test", "a.rivera@example.test"]
    );
}
#[test]
fn editor_and_virtual_file_inputs_are_selected_without_touching_each_other() {
    let mut r = realm();
    let editor = format!("version: 1\n{}", modify(ALEX, "editor draft"));
    let file = format!("version: 1\n{}", modify(ALEX, "selected file"));
    let inputs = u::Inputs::new(&editor, &[("/work/update.ldif", &file)]).unwrap();
    let report = u::run(
        &command("ldapmodify", ALEX, "-f /work/update.ldif"),
        SAMPLE,
        &inputs,
        &mut r,
    );
    assert!(report.succeeded());
    assert_eq!(field(&r, ALEX, "description"), ["selected file"]);
    assert!(u::run(&command("ldapmodify", ALEX, ""), SAMPLE, &inputs, &mut r).succeeded());
    assert_eq!(field(&r, ALEX, "description"), ["editor draft"]);
    assert!(editor.contains("editor draft"));
    assert!(file.contains("selected file"));
}
#[test]
fn absent_virtual_resources_wrong_password_and_denied_permissions_have_no_effect() {
    let raw = format!("version: 1\n{}", modify(ALEX, "must not happen"));
    let inputs = u::Inputs::new(&raw, &[]).unwrap();
    for (command, password, expected) in [
        (
            command("ldapmodify", ALEX, "-f /work/absent.ldif"),
            SAMPLE,
            1,
        ),
        (
            command("ldapmodify", ALEX, "").replace(URI, "ldap://not-supplied.example.test"),
            SAMPLE,
            1,
        ),
        (command("ldapmodify", ALEX, ""), "wrong", 49),
        (
            command("ldapmodify", "uid=absent,ou=People,dc=example,dc=test", ""),
            SAMPLE,
            49,
        ),
    ] {
        let mut r = realm();
        let before = r.directory().clone();
        let report = u::run(&command, password, &inputs, &mut r);
        assert!(!report.succeeded());
        assert_eq!(report.exit_status(), expected);
        assert!(report.applied.is_empty());
        assert_eq!(r.directory(), &before);
    }
    let mut r = realm();
    let before = r.directory().clone();
    let denied = format!(
        "version: 1\ndn: {ALEX}\nchangetype: modify\nreplace: mail\nmail: denied@example.test\n-\n"
    );
    let report = u::run(
        &command("ldapmodify", SAM, ""),
        SAMPLE,
        &u::Inputs::new(&denied, &[]).unwrap(),
        &mut r,
    );
    assert_eq!(report.exit_status(), 50);
    assert!(!report.succeeded());
    assert_eq!(r.directory(), &before);
}
#[test]
fn later_syntax_error_keeps_prior_success_and_stops_the_following_record() {
    let mut r = realm();
    let raw = format!(
        "version: 1\n{}\nnot a dn\n\n{}",
        modify(ALEX, "first request kept"),
        modify(SAM, "must not run")
    );
    let report = run(&mut r, &raw);
    assert_eq!(report.applied.len(), 1);
    assert!(matches!(
        report.failure,
        Some(u::Failure::Record { number: 2, .. })
    ));
    assert_eq!(report.exit_status(), 1);
    assert!(!report.succeeded());
    assert_eq!(field(&r, ALEX, "description"), ["first request kept"]);
    assert!(field(&r, SAM, "description").is_empty());
    // The old editor action deliberately parses its entire document first.
    assert!(l::parse(&raw).is_err());
}
#[test]
fn later_request_failure_keeps_prior_requests_but_rolls_back_its_own_edits() {
    let mut r = realm();
    let raw = format!(
        "version: 1\n{}\ndn: {SAM}\nchangetype: modify\nreplace: description\ndescription: must roll back\n-\ndelete: mail\nmail: absent@example.test\n-\n\n{}",
        modify(ALEX, "keep earlier"),
        modify(ALEX, "do not run later")
    );
    let report = run(&mut r, &raw);
    assert_eq!(report.applied.len(), 1);
    assert!(matches!(
        report.failure,
        Some(u::Failure::Request { number: 2, .. })
    ));
    assert_eq!(report.exit_status(), 16);
    assert_eq!(field(&r, ALEX, "description"), ["keep earlier"]);
    assert!(field(&r, SAM, "description").is_empty());
}
#[test]
fn add_default_and_explicit_delete_are_actual_operations_not_name_matches() {
    for verb in ["ldapadd", "ldapmodify"] {
        let mut r = realm();
        let entry = format!(
            "dn: uid=new,{BASE}\nobjectClass: inetOrgPerson\nuid: new\ncn: New Person\nsn: Person\n"
        );
        let raw = format!("version: 1\n{entry}");
        let extra = if verb == "ldapmodify" { "-a" } else { "" };
        let report = u::run(
            &command(verb, ALEX, extra),
            SAMPLE,
            &u::Inputs::new(&raw, &[]).unwrap(),
            &mut r,
        );
        assert!(report.succeeded());
        assert_eq!(r.directory().source().entries.len(), 8);
        let deletion = format!("version: 1\ndn: uid=new,{BASE}\nchangetype: delete\n");
        let report = u::run(
            &command(verb, ALEX, extra),
            SAMPLE,
            &u::Inputs::new(&deletion, &[]).unwrap(),
            &mut r,
        );
        assert!(report.succeeded());
        assert_eq!(r.directory().source().entries.len(), 7);
    }
}
#[test]
fn duplicates_are_failures_and_cannot_masquerade_as_successful_retries() {
    let mut r = realm();
    let raw = format!(
        "version: 1\ndn: uid=new,{BASE}\nchangetype: add\nobjectClass: inetOrgPerson\ncn: New Person\nsn: Person\n"
    );
    assert!(run(&mut r, &raw).succeeded());
    let before = r.directory().clone();
    let repeat = run(&mut r, &raw);
    assert_eq!(repeat.exit_status(), 68);
    assert!(!repeat.succeeded());
    assert!(repeat.applied.is_empty());
    assert_eq!(r.directory(), &before);
}
#[test]
fn empty_input_and_unimplemented_default_modify_never_report_success() {
    for raw in [
        "",
        "version: 1\n",
        &format!("version: 1\ndn: {ALEX}\ndescription: implicit replace\n"),
    ] {
        let mut r = realm();
        let before = r.directory().clone();
        let report = run(&mut r, raw);
        assert!(!report.succeeded());
        assert_eq!(report.exit_status(), 1);
        assert_eq!(r.directory(), &before);
    }
}
#[test]
fn virtual_input_identifiers_count_and_combined_size_are_validated_before_copying() {
    assert!(u::Inputs::new("", &[("/work/a", "one"), ("/work/a", "two")]).is_err());
    for name in [
        "/etc/passwd",
        "/work/../secret",
        "/work/a//b",
        "file:///work/a",
    ] {
        assert!(u::Inputs::new("", &[(name, "data")]).is_err());
    }
    assert!(u::Inputs::new("", &[("/work/a", "data"); 9]).is_err());
    let max = "x".repeat(l::MAX_TEXT);
    assert!(u::Inputs::new(&max, &[("/work/a", &max)]).is_ok());
    assert!(u::Inputs::new(&max, &[("/work/a", &max), ("/work/b", "x")]).is_err());
    assert!(u::Inputs::new(&format!("{max}x"), &[]).is_err());
}
#[test]
fn unsupported_host_escape_and_password_arguments_fail_without_side_effects() {
    let raw = format!("version: 1\n{}", modify(ALEX, "unchanged"));
    let inputs = u::Inputs::new(&raw, &[]).unwrap();
    for extra in [
        "-w not-allowed",
        "-f /etc/passwd",
        "; touch /tmp/no",
        "$(id)",
        "| sh",
        "> /work/out",
        "-c",
        "-Z",
        "-E txn=commit",
    ] {
        let mut r = realm();
        let before = r.directory().clone();
        let report = u::run(&command("ldapmodify", ALEX, extra), SAMPLE, &inputs, &mut r);
        assert!(matches!(report.failure, Some(u::Failure::Invocation(_))));
        assert!(!report.succeeded());
        assert_eq!(r.directory(), &before);
    }
}
#[test]
fn failed_login_cannot_inherit_success_or_expose_sample_passwords() {
    let mut r = realm();
    let raw = format!("version: 1\n{}", modify(ALEX, "first"));
    assert!(run(&mut r, &raw).succeeded());
    let before = r.directory().clone();
    let inputs = u::Inputs::new(&raw, &[]).unwrap();
    let failed = u::run(
        &command("ldapmodify", ALEX, ""),
        "second-wrong-value",
        &inputs,
        &mut r,
    );
    assert_eq!(failed.exit_status(), 49);
    assert!(!failed.succeeded());
    assert_eq!(r.directory(), &before);
    for text in [format!("{failed:?}"), failed.failure.unwrap().to_string()] {
        assert!(!text.contains(SAMPLE));
        assert!(!text.contains("second-wrong-value"));
    }
}
#[test]
fn semantically_equivalent_edits_and_changed_source_values_are_not_canned_answers() {
    for value in ["sample A", "different contact", "third run"] {
        let mut r = realm();
        let first = format!("version: 1\n{}", modify(ALEX, value));
        assert!(run(&mut r, &first).succeeded());
        let alternate = format!(
            "version: 1\ndn: {ALEX}\nchangetype: modify\ndelete: description\n-\nadd: description\ndescription: {value}\n-\n"
        );
        assert!(run(&mut r, &alternate).succeeded());
        assert_eq!(field(&r, ALEX, "description"), [value]);
    }
    // A supplied source can contain different values; no fixed fixture answer is recognized.
    let mut source = fixture();
    source.entries[2]
        .attributes
        .push(attr("description", &["preexisting"]));
    let id = i::Identity::new(dn(ALEX), SAMPLE, dn(BASE), true, true, &["description"]).unwrap();
    let mut r = i::Realm::new(URI, &source, &[id]).unwrap();
    let raw = format!("version: 1\n{}", modify(ALEX, "newly requested"));
    assert!(run(&mut r, &raw).succeeded());
    assert_eq!(field(&r, ALEX, "description"), ["newly requested"]);
}
