use super::*;
const FILE: &str = "# supplied sample, not the host database\nhttp 80/tcp www\ndomain 53/udp dns\ndomain 53/tcp\nhttps 443/tcp\nhttps 443/udp\n";
#[test]
fn parses_names_aliases_protocols_and_numeric_reverse_lookups() {
    let db = Database::parse(FILE).unwrap();
    assert_eq!(db.records().len(), 5);
    for key in ["http", "www", "80", "00080/tcp"] {
        let r = db.lookup(&Query::parse(key).unwrap()).unwrap();
        assert_eq!(
            (r.name.as_str(), r.port, r.protocol),
            ("http", 80, Protocol::Tcp)
        );
    }
    for key in ["domain/udp", "dns/udp", "53/udp"] {
        let r = db.lookup(&Query::parse(key).unwrap()).unwrap();
        assert_eq!(
            (r.name.as_str(), r.port, r.protocol),
            ("domain", 53, Protocol::Udp)
        );
    }
    assert_eq!(
        db.lookup(&Query::parse("domain/tcp").unwrap())
            .unwrap()
            .protocol,
        Protocol::Tcp
    );
    assert!(db.lookup(&Query::parse("dns/tcp").unwrap()).is_none());
    assert!(db.lookup(&Query::parse("HTTP").unwrap()).is_none());
}
#[test]
fn absent_protocol_uses_supplied_file_order_not_invented_tcp_preference() {
    let db = Database::parse(FILE).unwrap();
    assert_eq!(
        db.lookup(&Query::parse("domain").unwrap())
            .unwrap()
            .protocol,
        Protocol::Udp
    );
    let changed = FILE.replace(
        "domain 53/udp dns\ndomain 53/tcp",
        "domain 53/tcp\ndomain 53/udp dns",
    );
    assert_eq!(
        Database::parse(&changed)
            .unwrap()
            .lookup(&Query::parse("domain").unwrap())
            .unwrap()
            .protocol,
        Protocol::Tcp
    );
}
#[test]
fn comments_tabs_and_crlf_preserve_the_actual_lookup_meaning() {
    let a = Database::parse(FILE).unwrap();
    let file = " \t# a leading comment\r\n\r\nhttp\t80/tcp\twww # notes\r\ndomain 53/udp dns\r\ndomain 53/tcp\r\nhttps 443/tcp\r\nhttps 443/udp\r\n";
    let b = Database::parse(file).unwrap();
    assert_eq!(a.output().unwrap(), b.output().unwrap());
    assert_eq!(
        Database::parse("# empty\n\n").unwrap().output().unwrap(),
        ""
    );
}
#[test]
fn actual_getent_output_status_and_cat_come_from_virtual_file_data() {
    let r = execute(FILE, "getent services www/tcp").unwrap();
    assert_eq!(r.status, 0);
    assert_eq!(r.stdout, "http                  80/tcp www\n");
    assert_eq!(
        execute(FILE, "getent services absent").unwrap(),
        Output {
            stdout: String::new(),
            status: 2
        }
    );
    assert_eq!(execute(FILE, "getent services http/udp").unwrap().status, 2);
    assert_eq!(execute(FILE, "cat /etc/services").unwrap().stdout, FILE);
    assert_eq!(
        execute(FILE, "getent services").unwrap().stdout,
        Database::parse(FILE).unwrap().output().unwrap()
    );
    let changed = FILE.replace("http 80/tcp", "http 8080/tcp");
    let r = execute(&changed, "getent services www").unwrap();
    assert_eq!(r.status, 0);
    assert!(r.stdout.contains("8080/tcp"));
    assert_eq!(
        execute(&changed, "getent services 80/tcp").unwrap().status,
        2
    );
}
#[test]
fn syntax_and_valid_but_unsupported_file_features_are_not_silently_skipped() {
    for text in [
        "http\n",
        "http tcp\n",
        "http 80/tcp/extra\n",
        "http x/tcp\n",
    ] {
        assert_eq!(
            Database::parse(text).unwrap_err().kind,
            FaultKind::Syntax,
            "{text}"
        );
    }
    for text in [
        " http 80/tcp\n",
        "http 80/sctp\n",
        "http 80/tcp\nweb 80/tcp\n",
        "http 80/tcp www\nother 81/tcp www\n",
        "http 80/tcp http\n",
        "bad_name 80/tcp\n",
        "http 80/tcp\rbroken",
    ] {
        assert_eq!(
            Database::parse(text).unwrap_err().kind,
            FaultKind::Unsupported,
            "{text}"
        );
    }
    assert_eq!(
        Database::parse("http 65536/tcp\n").unwrap_err().kind,
        FaultKind::Invalid
    );
    let zero = Database::parse("reserved 0/tcp\n").unwrap();
    assert_eq!(
        zero.lookup(&Query::parse("0/tcp").unwrap()).unwrap().port,
        0
    );
}
#[test]
fn declared_limits_fail_before_unbounded_processing() {
    assert_eq!(
        Database::parse(&" ".repeat(MAX_FILE + 1)).unwrap_err().kind,
        FaultKind::Limit
    );
    assert_eq!(
        Database::parse(&format!("#{}", "x".repeat(MAX_LINE)))
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
    let many = (0..65)
        .map(|i| format!("svc{i} {i}/tcp\n"))
        .collect::<String>();
    assert_eq!(Database::parse(&many).unwrap_err().kind, FaultKind::Limit);
    let aliases = format!(
        "http 80/tcp {}\n",
        (0..9)
            .map(|i| format!("alias{i}"))
            .collect::<Vec<_>>()
            .join(" ")
    );
    assert_eq!(
        Database::parse(&aliases).unwrap_err().kind,
        FaultKind::Limit
    );
    assert!(Query::parse(&"a".repeat(MAX_QUERY + 1)).is_err());
    assert_eq!(
        execute(FILE, &" ".repeat(MAX_COMMAND + 1))
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
}
#[test]
fn hostile_or_unimplemented_commands_are_diagnostics_not_success_or_host_access() {
    let before = FILE.to_owned();
    for cmd in [
        "cat /etc/passwd",
        "cat ../../private",
        "getent passwd root",
        "getent -s files services http",
        "getent services http domain",
        "getent services $(cat /private)",
        "getent services http; touch /private",
        "getent services http | cat",
        "getent services 'http'",
        "getent services http\ncat /etc/passwd",
        "getent services *.tcp",
        "getent services http # comment",
    ] {
        assert_eq!(
            execute(FILE, cmd).unwrap_err().kind,
            FaultKind::Unsupported,
            "{cmd}"
        );
    }
    assert_eq!(FILE, before);
    assert!(
        execute(FILE, "help")
            .unwrap()
            .stdout
            .contains("Simulator reference")
    );
    for query in [
        "http/",
        "http/tcp/udp",
        "/tcp",
        "-1",
        "http/TCP",
        "http/sctp",
    ] {
        assert!(Query::parse(query).is_err(), "{query}");
    }
}
#[test]
fn malformed_file_is_visible_as_data_but_not_a_fake_successful_lookup() {
    let invalid = "# unfinished virtual file\nhttp";
    assert_eq!(
        execute(invalid, "cat /etc/services").unwrap().stdout,
        invalid
    );
    let e = execute(invalid, "getent services http").unwrap_err();
    assert_eq!(e.kind, FaultKind::Syntax);
    assert_eq!(e.line, Some(2));
    let fixed = "http 80/tcp\n";
    assert_eq!(execute(fixed, "getent services http").unwrap().status, 0);
}
