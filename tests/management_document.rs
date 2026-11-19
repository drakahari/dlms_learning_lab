use dlms_learning_lab::{
    acl_editor::Category, management_document as doc, management_policy as policy,
};
#[path = "support/management_fixture.rs"]
mod fixture;
use fixture::{network, settings};

fn supplied() -> policy::Policy {
    let mut p = settings();
    let row = p.lines[0].clone();
    p.lines = (0..16)
        .map(|number| policy::Line {
            number,
            ..row.clone()
        })
        .collect();
    p
}
fn document(split: bool, number: &str) -> String {
    let body = |first, last| {
        format!("line vty {first} {last}\n transport input ssh\n access-class {number} in\nexit\n")
    };
    let groups = if split {
        body(0, 4) + &body(5, 15)
    } else {
        body(0, 15)
    };
    format!(
        "ip ssh version 2\n{groups}ip access-list standard {number}\n permit 192.0.2.0 0.0.0.127\nexit\n"
    )
}

#[test]
fn complete_policy_documents_reach_real_router_paths_and_restrict_each_supplied_line() {
    let n = network();
    let context = supplied();
    let p = doc::parse(&n, &context, &document(true, "10")).unwrap();
    assert_eq!(p.host_identity, context.host_identity);
    assert_eq!(p.lines.len(), 16);
    for line in 0..16 {
        assert!(
            n.inspect(&p, "admin", line, policy::Protocol::Ssh2)
                .unwrap()
                .allowed_to_authenticate()
        );
        assert!(
            !n.inspect(&p, "guest", line, policy::Protocol::Ssh2)
                .unwrap()
                .allowed_to_authenticate()
        );
        assert!(
            !n.inspect(&p, "admin", line, policy::Protocol::Ssh1)
                .unwrap()
                .allowed_to_authenticate()
        );
        assert!(
            !n.inspect(&p, "admin", line, policy::Protocol::Telnet)
                .unwrap()
                .allowed_to_authenticate()
        );
    }
}

#[test]
fn equivalent_line_groups_standard_numbers_and_rule_decompositions_work() {
    let n = network();
    let context = supplied();
    let a = doc::parse(&n, &context, &document(true, "10")).unwrap();
    let b = doc::parse(
        &n,
        &context,
        &document(false, "1999").replace(
            "permit 192.0.2.0 0.0.0.127",
            "permit 192.0.2.0 0.0.0.63\n permit 192.0.2.64 0.0.0.63",
        ),
    )
    .unwrap();
    for line in 0..16 {
        for client in ["admin", "guest"] {
            assert_eq!(
                n.inspect(&a, client, line, policy::Protocol::Ssh2)
                    .unwrap()
                    .allowed_to_authenticate(),
                n.inspect(&b, client, line, policy::Protocol::Ssh2)
                    .unwrap()
                    .allowed_to_authenticate()
            );
        }
    }
}

#[test]
fn version_compatibility_and_explicit_protocol_choices_do_not_become_canned_ssh_success() {
    let n = network();
    let context = supplied();
    let source = document(false, "10")
        .replace("ip ssh version 2", "no ip ssh version")
        .replace("transport input ssh", "transport input telnet ssh");
    let p = doc::parse(&n, &context, &source).unwrap();
    for protocol in [
        policy::Protocol::Ssh1,
        policy::Protocol::Ssh2,
        policy::Protocol::Telnet,
    ] {
        assert!(
            n.inspect(&p, "admin", 0, protocol)
                .unwrap()
                .allowed_to_authenticate()
        );
        assert!(
            !n.inspect(&p, "guest", 0, protocol)
                .unwrap()
                .allowed_to_authenticate()
        );
    }
    let p = doc::parse(
        &n,
        &context,
        &source.replace("transport input telnet ssh", "transport input none"),
    )
    .unwrap();
    assert!(
        !n.inspect(&p, "admin", 0, policy::Protocol::Ssh2)
            .unwrap()
            .allowed_to_authenticate()
    );
}

#[test]
fn source_line_roster_cannot_be_reduced_overlapped_or_replaced_by_a_new_line() {
    let n = network();
    let context = supplied();
    let missing = "ip ssh version 2\nline vty 0 4\ntransport input ssh\nexit\n";
    assert_eq!(
        doc::parse(&n, &context, missing).unwrap_err().category,
        Category::FixedContext
    );
    let overlap = document(true, "10").replace("line vty 5 15", "line vty 4 15");
    assert_eq!(
        doc::parse(&n, &context, &overlap).unwrap_err().category,
        Category::Unsupported
    );
    let unknown = document(true, "10").replace("line vty 5 15", "line vty 5 16");
    assert_eq!(
        doc::parse(&n, &context, &unknown).unwrap_err().category,
        Category::FixedContext
    );
    let mut sparse = context.clone();
    sparse.lines = vec![context.lines[9].clone(), context.lines[0].clone()];
    let source = "ip ssh version 2\nline vty 0\ntransport input ssh\nexit\nline vty 9\ntransport input ssh\nexit\n";
    let p = doc::parse(&n, &sparse, source).unwrap();
    assert_eq!(
        p.lines.iter().map(|l| l.number).collect::<Vec<_>>(),
        vec![9, 0]
    );
}

#[test]
fn missing_settings_and_duplicate_overrides_explain_the_complete_document_boundary() {
    let n = network();
    let context = supplied();
    for (text, category) in [
        (
            document(false, "10").replacen("ip ssh version 2\n", "", 1),
            Category::Syntax,
        ),
        (
            document(false, "10").replace("transport input ssh\n", ""),
            Category::Syntax,
        ),
        (
            format!("ip ssh version 2\n{}", document(false, "10")),
            Category::Unsupported,
        ),
        (
            document(false, "10").replace(
                "transport input ssh",
                "transport input ssh\ntransport input telnet",
            ),
            Category::Unsupported,
        ),
        (
            document(false, "10").replace(
                "access-class 10 in",
                "access-class 10 in\naccess-class 10 in",
            ),
            Category::Unsupported,
        ),
    ] {
        assert_eq!(
            doc::parse(&n, &context, &text).unwrap_err().category,
            category,
            "{text}"
        );
    }
}

#[test]
fn unsupported_protocols_named_classes_outbound_vrfs_and_session_commands_fail_honestly() {
    let n = network();
    let context = supplied();
    for source in [
        document(false, "10").replace("transport input ssh", "transport input all"),
        document(false, "10").replace("transport input ssh", "transport input rlogin"),
        document(false, "10").replace("transport input ssh", "transport input ssh ssh"),
        document(false, "10").replace("ip ssh version 2", "ip ssh version 1"),
        document(false, "10").replace("access-class 10 in", "access-class MGMT in"),
        document(false, "10").replace("access-class 10 in", "access-class 10 out"),
        document(false, "10").replace("access-class 10 in", "access-class 10 in vrf-also"),
        format!("configure terminal\n{}", document(false, "10")),
        format!(
            "ip ssh rsa keypair-name supplied-lab-identity\n{}",
            document(false, "10")
        ),
    ] {
        assert_eq!(
            doc::parse(&n, &context, &source).unwrap_err().category,
            Category::Unsupported,
            "{source}"
        );
    }
}

#[test]
fn standard_number_ranges_and_missing_references_are_not_silently_approximated() {
    let n = network();
    let context = supplied();
    for number in ["1", "99", "1300", "1999"] {
        doc::parse(&n, &context, &document(false, number)).unwrap();
    }
    for number in ["0", "100", "1299", "2000"] {
        assert_eq!(
            doc::parse(&n, &context, &document(false, number))
                .unwrap_err()
                .category,
            Category::Unsupported
        );
    }
    let missing =
        "ip ssh version 2\nline vty 0 15\ntransport input ssh\naccess-class 10 in\nexit\n";
    let e = doc::parse(&n, &context, missing).unwrap_err();
    assert_eq!(e.category, Category::Resolution);
    assert_eq!(e.line, Some(4));
    assert_eq!(
        doc::parse(
            &n,
            &context,
            &document(false, "10").replace("standard 10", "extended 10")
        )
        .unwrap_err()
        .category,
        Category::Unsupported
    );
}

#[test]
fn shared_acl_diagnostics_keep_original_document_line_numbers() {
    let source = "ip ssh version 2\nline vty 0 15\n transport input ssh\n access-class 10 in\nexit\n! explanation\n\nip access-list standard 10\n permit not-an-address 0.0.0.127\nexit\n";
    let n = network();
    let context = supplied();
    let e = doc::parse(&n, &context, source).unwrap_err();
    assert_eq!(e.line, Some(9));
    assert!(!e.message.is_empty());
    let empty = document(false, "10").replace(" permit 192.0.2.0 0.0.0.127\n", "");
    assert_eq!(
        doc::parse(&n, &context, &empty).unwrap_err().category,
        Category::Unsupported
    );
}

#[test]
fn malformed_drafts_do_not_change_supplied_policy_identity_or_topology() {
    let n = network();
    let before = n.clone();
    let context = supplied();
    let original = context.clone();
    for source in [
        "",
        "exit\n",
        "line vty 98 0\n",
        "$(touch /tmp/never)\n",
        "ip ssh version 2\n\0",
    ] {
        assert!(doc::parse(&n, &context, source).is_err());
        assert_eq!(n, before);
        assert_eq!(context, original);
    }
    let doc = document(false, "10").replace("\n", "\r\n");
    let p = doc::parse(&n, &context, &doc).unwrap();
    assert_eq!(p.host_identity, context.host_identity);
    assert_eq!(n, before);
}

#[test]
fn resource_limits_bound_text_lines_tokens_and_expanded_rule_attachments() {
    let n = network();
    let context = supplied();
    for source in [
        " ".repeat(65537),
        format!("!{}", "x".repeat(512)),
        "\n".repeat(1025),
        format!("{}\n", vec!["x"; 17].join(" ")),
    ] {
        assert_eq!(
            doc::parse(&n, &context, &source).unwrap_err().category,
            Category::Limit
        );
    }
    let mut source = document(false, "10");
    source = source.replace(
        " permit 192.0.2.0 0.0.0.127",
        &(0..9)
            .map(|i| format!(" permit host 192.0.2.{}", i + 1))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    assert_eq!(
        doc::parse(&n, &context, &source).unwrap_err().category,
        Category::Limit
    );
}

#[test]
fn incomplete_supported_settings_are_syntax_errors_and_keep_the_actual_line_number() {
    let network = fixture::network();
    let supplied = fixture::settings();
    for (text, line, detail) in [
        ("! Read the setting\nip ssh version\n", 2, "version number"),
        (
            "ip ssh version 2\nline vty 0\n transport input\n",
            3,
            "protocol choice",
        ),
        (
            "ip ssh version 2\nline vty 0\n access-class 10\n",
            3,
            "NUMBER in",
        ),
    ] {
        let e =
            dlms_learning_lab::management_document::parse(&network, &supplied, text).unwrap_err();
        assert_eq!(e.category, dlms_learning_lab::acl_editor::Category::Syntax);
        assert_eq!(e.line, Some(line));
        assert!(e.to_string().contains(detail));
    }
}
