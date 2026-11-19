use dlms_learning_lab::local_mail as mail;
use mail::{Directory, parse_aliases, parse_forward};
use std::collections::{BTreeMap, BTreeSet};

fn users() -> BTreeSet<String> {
    ["alex", "jules", "sam"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

#[test]
fn alias_order_case_and_helper_paths_expand_to_the_same_mailboxes() {
    let direct = Directory {
        users: users(),
        aliases: parse_aliases("Team: ALEX, Jules\nadmin: sam\n").unwrap(),
        forwards: BTreeMap::new(),
    };
    let helper = Directory {
        users: users(),
        aliases: parse_aliases("admin: sam\nteam: helper\nhelper: jules, alex, alex\n").unwrap(),
        forwards: BTreeMap::new(),
    };
    let left = direct.deliver("TEAM").unwrap();
    assert_eq!(left, helper.deliver("team").unwrap());
    assert_eq!(
        left.mailboxes,
        ["alex", "jules"].into_iter().map(str::to_owned).collect()
    );
    assert!(left.failures.is_empty());
    assert_eq!(
        direct.deliver("admin").unwrap().mailboxes,
        ["sam".to_owned()].into()
    );
}

#[test]
fn forwarding_replaces_delivery_and_explicit_self_keeps_a_copy() {
    let mut directory = Directory {
        users: users(),
        aliases: parse_aliases("team: alex\n").unwrap(),
        forwards: BTreeMap::from([("alex".into(), parse_forward("jules\n").unwrap())]),
    };
    assert_eq!(
        directory.deliver("team").unwrap().mailboxes,
        ["jules".to_owned()].into()
    );
    directory
        .forwards
        .insert("alex".into(), parse_forward("alex, jules\n").unwrap());
    let report = directory.deliver("team").unwrap();
    assert_eq!(
        report.mailboxes,
        ["alex", "jules"].into_iter().map(str::to_owned).collect()
    );
    assert!(report.failures.is_empty());
    directory.forwards.insert(
        "alex".into(),
        parse_forward("# deliberately empty\n").unwrap(),
    );
    assert_eq!(
        directory.deliver("team").unwrap().mailboxes,
        ["alex".to_owned()].into()
    );
}

#[test]
fn cycles_and_missing_recipients_preserve_independent_deliveries() {
    let directory = Directory {
        users: users(),
        aliases: parse_aliases("team: alex, first, nobody\nfirst: second\nsecond: first\n")
            .unwrap(),
        forwards: BTreeMap::new(),
    };
    let report = directory.deliver("team").unwrap();
    assert_eq!(report.mailboxes, ["alex".to_owned()].into());
    assert!(
        report
            .failures
            .iter()
            .any(|failure| failure.contains("loop"))
    );
    assert!(
        report
            .failures
            .iter()
            .any(|failure| failure.contains("nobody"))
    );
}

#[test]
fn unsupported_forms_are_errors_and_cannot_run_programs_or_open_paths() {
    for text in [
        "team: |touch /tmp/example\n",
        "team: /etc/passwd\n",
        "team: :include:/etc/aliases\n",
        "team: \"alex\"\n",
        "team: alex@example.invalid\n",
        "team: alex\n other: jules\n",
        "team: alex\nteam: jules\n",
        "team: alex\u{a0}\n",
    ] {
        assert!(parse_aliases(text).is_err(), "{text:?}");
    }
    assert!(parse_forward("alex,").is_err());
    assert!(parse_aliases(&"x".repeat(mail::MAX_TEXT + 1)).is_err());
    let directory = Directory {
        users: users(),
        aliases: parse_aliases("alex: alex\n").unwrap(),
        forwards: BTreeMap::new(),
    };
    assert!(directory.validate().is_err());
}

#[test]
fn expansion_budget_bounds_acyclic_fanout() {
    let mut aliases = BTreeMap::new();
    for index in 0..15 {
        aliases.insert(format!("a{index}"), vec![format!("a{}", index + 1); 8]);
    }
    aliases.insert("a15".into(), vec!["sam".into()]);
    let directory = Directory {
        users: users(),
        aliases,
        forwards: BTreeMap::new(),
    };
    let report = directory.deliver("a0").unwrap();
    assert!(
        report
            .failures
            .iter()
            .any(|failure| failure.contains("128-step"))
    );
}
