use dlms_learning_lab::debian_sources as s;
const ONE: &str =
    "Types: deb\nURIs: https://repo.workshop.test/debian\nSuites: studio\nComponents: main tools\n";
fn keys(text: &str) -> s::Selection {
    s::select(&s::parse(text).unwrap()).unwrap()
}
#[test]
fn modern_sources_select_ordered_targets_and_normalize_only_the_trailing_slash() {
    let r = keys(ONE);
    assert_eq!(r.keys.len(), 2);
    assert_eq!(
        r.keys[0],
        s::Key {
            uri: "https://repo.workshop.test/debian/".into(),
            suite: "studio".into(),
            component: "main".into()
        }
    );
    assert_eq!(r.keys[1].component, "tools");
    assert_eq!(r.repeated_targets, 0);
    for k in &r.keys {
        k.validate().unwrap();
    }
    assert_eq!(r, keys(&ONE.replace("debian\n", "debian/\n")));
    assert_eq!(r, keys(&ONE.replace('\n', "\r\n")));
    assert!(
        s::uri("https://repo.workshop.test/debian//")
            .unwrap()
            .ends_with("//")
    );
}
#[test]
fn case_fields_folding_comments_and_value_products_keep_their_real_relationship() {
    let text = "# Separate sources, not shell commands\nTyPeS: deb\nuRiS: https://one.workshop.test/repo\n https://two.workshop.test/repo\nSuites: studio archive\nComponents: main\n tools\nEnabled: YES\n";
    let r = keys(text);
    assert_eq!(r.keys.len(), 8);
    assert_eq!(r.keys[0].suite, "studio");
    assert_eq!(r.keys[1].component, "tools");
    assert_eq!(r.keys[2].suite, "archive");
    assert_eq!(r.keys[4].uri, "https://two.workshop.test/repo/");
}
#[test]
fn enabled_selection_changes_eligibility_without_fabricating_fetched_data() {
    assert!(keys("").keys.is_empty());
    assert!(keys("# no configured binary sources\n").keys.is_empty());
    assert!(keys(&(ONE.to_owned() + "Enabled: no\n")).keys.is_empty());
    assert!(keys("Enabled: no\n").keys.is_empty());
    let fields = s::parse(&(ONE.to_owned() + "Enabled: no\n")).unwrap();
    assert_eq!(fields[0].uris, ["https://repo.workshop.test/debian"]);
    assert!(!fields[0].enabled);
    assert_eq!(keys(ONE).keys.len(), 2);
}
#[test]
fn repeated_targets_share_one_index_without_silently_changing_preference() {
    let r = keys(&(ONE.to_owned() + "\n" + ONE));
    assert_eq!(r.keys.len(), 2);
    assert_eq!(r.repeated_targets, 2);
    assert_eq!(r.keys, keys(ONE).keys);
    let r = keys(&ONE.replace("Components: main tools", "Components: tools main tools"));
    assert_eq!(r.keys[0].component, "tools");
    assert_eq!(r.keys[1].component, "main");
    assert_eq!(r.repeated_targets, 1);
}
#[test]
fn missing_and_unsupported_fields_have_honest_different_diagnostics() {
    for text in [
        "Types: deb\nURIs: https://one.test/repo\nSuites: studio\n",
        "URIs: https://one.test/repo\nSuites: studio\nComponents: main\n",
        " Components: main\n",
    ] {
        assert_eq!(s::parse(text).unwrap_err().code, s::Code::Syntax, "{text}");
    }
    for text in [
        ONE.to_owned() + "Trusted: yes\n",
        ONE.replace("Types: deb", "Types: deb-src"),
        ONE.replace("Types: deb", "Types: deb deb-src"),
        ONE.to_owned() + "Enabled: true\n",
        ONE.to_owned() + "Suites: archive\n",
        ONE.to_owned() + "sUiTeS: archive\n",
        "deb https://one.test/repo studio main\n".into(),
        ONE.to_owned() + "Signed-By: /home/user/key\n",
    ] {
        assert_eq!(
            s::parse(&text).unwrap_err().code,
            s::Code::Unsupported,
            "{text}"
        );
    }
}
#[test]
fn input_lines_field_continuations_products_and_direct_api_stay_bounded() {
    for text in [
        "x".repeat(s::MAX_INPUT + 1),
        "#comment\n".repeat(s::MAX_LINES + 1),
        (0..=s::MAX_STANZAS)
            .map(|_| ONE)
            .collect::<Vec<_>>()
            .join("\n"),
        ONE.replace("studio", "one two three"),
        ONE.replace("main tools", "one two three four"),
        format!("URIs: {}\n {}\n", "x".repeat(400), "y".repeat(200)),
    ] {
        assert_eq!(s::parse(&text).unwrap_err().code, s::Code::Limit);
    }
    let mut direct = s::parse(ONE).unwrap();
    direct[0].uris = vec!["x".repeat(161)];
    assert_eq!(s::select(&direct).unwrap_err().code, s::Code::Limit);
    let mut direct = s::parse(ONE).unwrap();
    direct[0].components = vec!["a".into(); 4];
    assert_eq!(s::select(&direct).unwrap_err().code, s::Code::Limit);
    let text = "Types: deb\nURIs: https://one.test/repo https://two.test/repo\nSuites: alpha beta\nComponents: one two three\n";
    assert_eq!(
        keys(
            &(text.to_owned()
                + "\n"
                + &text
                    .replace("one.test", "three.test")
                    .replace("two.test", "four.test"))
        )
        .keys
        .len(),
        24
    );
}
#[test]
fn unsafe_or_unmodeled_uri_forms_refuse_without_opening_anything() {
    for uri in [
        "file:///home/secret",
        "https://user:password@one.test/repo",
        "https://one.test/../repo",
        "https://one.test/a?token=x",
        "https://one.test/a#tag",
        "https://one.test/%2e%2e/",
        "https://One.test/repo",
        "https://one.test:443/repo",
        "https://[::1]/repo",
        "https://one.test/repo\0",
    ] {
        assert!(s::uri(uri).is_err(), "{uri}");
    }
    assert_eq!(
        s::parse(&ONE.replace("studio", "studio\0"))
            .unwrap_err()
            .code,
        s::Code::Unsupported
    );
    let k = s::Key {
        uri: "https://one.test/repo".into(),
        suite: "studio".into(),
        component: "main".into(),
    };
    assert_eq!(k.validate().unwrap_err().code, s::Code::Syntax);
}

#[test]
fn separate_target_stanzas_and_a_retained_disabled_stanza_are_supported_but_not_unbounded() {
    let one = "Types: deb\nURIs: https://one.test/repo\nSuites: studio\nComponents: main\n";
    let four = [
        one.to_owned(),
        one.replace("main", "tools"),
        one.replace("studio", "studio-updates"),
        one.replace("studio", "archive") + "Enabled: no\n",
    ]
    .join("\n");
    let selected = keys(&four);
    assert_eq!(s::parse(&four).unwrap().len(), 4);
    assert_eq!(selected.keys.len(), 3);
    assert!(!selected.keys.iter().any(|k| k.suite == "archive"));
    let product = "Types: deb\nURIs: https://one.test/repo https://two.test/repo\nSuites: alpha beta\nComponents: one two three\n";
    let four = (0..4)
        .map(|i| {
            product
                .replace("one.test", &format!("one{i}.test"))
                .replace("two.test", &format!("two{i}.test"))
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        s::select(&s::parse(&four).unwrap()).unwrap_err().code,
        s::Code::Limit,
        "Active targets remain independently bounded at24"
    );
}
