//! Authored expectations for a closed model; none are actual Apache observations.
use dlms_learning_lab::apache_static::*;
use std::collections::BTreeMap;

fn vhost(name: &str, root: &str) -> String {
    format!(
        "<VirtualHost *:8080>\nServerName {name}\nDocumentRoot \"{root}\"\n<Directory \"{root}\">\nRequire all granted\n</Directory>\n</VirtualHost>\n"
    )
}
fn baseline() -> &'static str {
    "Listen 8080\n<Directory \"/\">\nOptions None\nAllowOverride None\nRequire all denied\n</Directory>\n"
}
fn config() -> String {
    format!(
        "{}<VirtualHost *:8080>\nServerName gallery.example.test\nServerAlias art.example.test shared.example.test\nDocumentRoot \"/srv/gallery\"\nDirectoryIndex welcome.html index.html\n<Directory \"/srv/gallery\">\nRequire all granted\n</Directory>\n<Directory \"/srv/gallery/private\">\nRequire all denied\n</Directory>\n</VirtualHost>\n{}",
        baseline(),
        vhost("archive.example.test", "/srv/archive")
    )
}
fn fixture() -> State {
    let mut entries = BTreeMap::new();
    for p in [
        "/",
        "/srv",
        "/srv/gallery",
        "/srv/gallery/private",
        "/srv/gallery/private-notes",
        "/srv/gallery/empty",
        "/srv/gallery/section",
        "/srv/archive",
    ] {
        entries.insert(p.into(), EntryKind::Directory);
    }
    let mut files = BTreeMap::new();
    for (p, t) in [
        ("/srv/gallery/welcome.html", "Gallery home\n"),
        ("/srv/gallery/index.html", "Second choice\n"),
        ("/srv/gallery/private/secret.txt", "Not public\n"),
        ("/srv/gallery/private-notes/note.txt", "Public note\n"),
        ("/srv/gallery/section/index.html", "Café — 展示\n"),
        ("/srv/archive/index.html", "Archive intact\n"),
    ] {
        entries.insert(p.into(), EntryKind::File);
        files.insert(p.into(), t.into());
    }
    State::new(
        Inventory {
            fixture: "museum-http-v1".into(),
            server: "museum".into(),
            config_path: "/config/httpd.conf".into(),
            entries,
        },
        config(),
        files,
    )
    .unwrap()
}
fn running() -> State {
    let mut s = fixture();
    assert!(s.apply(&Action::Start {}).unwrap());
    s
}
fn req(host: &str, path: &str) -> Request {
    Request {
        server: "museum".into(),
        port: 8080,
        method: Method::Get,
        hosts: vec![host.into()],
        path: path.into(),
    }
}
fn hit(s: &State, h: &str, p: &str) -> Response {
    match request(s, &req(h, p)).unwrap() {
        Outcome::Http(r) => r,
        Outcome::NoListener => panic!("Expected an actual HTTP observation"),
    }
}
fn save_load(s: &mut State, text: String) {
    s.apply(&Action::SaveConfig { text }).unwrap();
    s.apply(&Action::Reload {}).unwrap();
}

#[test]
fn ordered_names_aliases_and_first_default_use_actual_destination_port() {
    let s = running();
    for host in [
        "gallery.example.test",
        "ART.EXAMPLE.TEST",
        "gallery.example.test:9999",
        "unknown.example.test",
    ] {
        let r = hit(&s, host, "/");
        assert_eq!((r.status.code(), r.body.as_str()), (200, "Gallery home\n"));
        assert_eq!(r.vhost.as_deref(), Some("gallery.example.test"));
    }
    assert_eq!(
        hit(&s, "archive.example.test", "/").body,
        "Archive intact\n"
    );
    let mut r = req("gallery.example.test:8080", "/");
    r.port = 9999;
    assert_eq!(request(&s, &r).unwrap(), Outcome::NoListener);
    r.port = 8080;
    r.server = "another-server".into();
    assert_eq!(request(&s, &r).unwrap(), Outcome::NoListener);
}

#[test]
fn duplicate_name_alias_shadowing_is_source_order_not_name_priority() {
    let first = vhost("first.example.test", "/srv/archive").replace(
        "DocumentRoot",
        "ServerAlias gallery.example.test\nDocumentRoot",
    );
    let mut s = running();
    save_load(
        &mut s,
        format!(
            "{}{}{}",
            baseline(),
            first,
            vhost("gallery.example.test", "/srv/gallery")
        ),
    );
    assert_eq!(
        hit(&s, "gallery.example.test", "/index.html").body,
        "Archive intact\n"
    );
    save_load(
        &mut s,
        format!(
            "{}{}{}",
            baseline(),
            vhost("gallery.example.test", "/srv/gallery"),
            first
        ),
    );
    assert_eq!(
        hit(&s, "gallery.example.test", "/index.html").body,
        "Second choice\n"
    );
    assert_eq!(
        hit(&s, "unlisted.example.test", "/index.html")
            .vhost
            .as_deref(),
        Some("gallery.example.test")
    );
}

#[test]
fn only_verified_bad_host_syntax_is_400_valid_excluded_reg_names_stay_unsupported() {
    let s = running();
    for raw in [
        "gallery example.test",
        "gallery.example.test:abc",
        "user@gallery.example.test",
        "gallery.example.test/path",
        "bad\rhost",
        "a#fragment",
    ] {
        let r = hit(&s, raw, "/");
        assert_eq!(
            (r.status, r.cause),
            (HttpStatus::BadRequest, Cause::InvalidHost)
        );
        assert!(r.vhost.is_none());
    }
    for raw in [
        "under_score.example",
        "pct%61.example",
        "gallery.example.test.",
        "[::1]",
        "a!b.example",
        "127.0.0.1",
        "",
        "host:",
        "host:999999",
    ] {
        assert_eq!(
            request(&s, &req(raw, "/")).unwrap_err().kind,
            FaultKind::Unsupported,
            "{raw}"
        );
    }
    assert_eq!(
        hit(&s, " \tART.example.test\t ", "/").body,
        "Gallery home\n"
    );
    let mut r = req("gallery.example.test", "/");
    r.hosts.clear();
    assert!(matches!(
        request(&s, &r).unwrap(),
        Outcome::Http(Response {
            status: HttpStatus::BadRequest,
            ..
        })
    ));
    r.hosts = vec!["gallery.example.test".into(), "archive.example.test".into()];
    assert!(matches!(
        request(&s, &r).unwrap(),
        Outcome::Http(Response {
            status: HttpStatus::BadRequest,
            ..
        })
    ));
}

#[test]
fn directory_index_order_defaults_disabled_and_no_recursive_index_claim() {
    let mut s = running();
    assert_eq!(
        hit(&s, "gallery.example.test", "/").resource.as_deref(),
        Some("/srv/gallery/welcome.html")
    );
    save_load(
        &mut s,
        config().replace("DirectoryIndex welcome.html index.html\n", ""),
    );
    assert_eq!(hit(&s, "gallery.example.test", "/").body, "Second choice\n");
    save_load(
        &mut s,
        config().replace("welcome.html index.html", "missing.html index.html"),
    );
    assert_eq!(hit(&s, "gallery.example.test", "/").body, "Second choice\n");
    // Sole disabled is a case-insensitive keyword, even if a file with that
    // exact spelling would otherwise be present and readable in this directory.
    for name in ["disabled", "DISABLED", "DiSaBlEd"] {
        let p = format!("/srv/gallery/{name}");
        s.inventory.entries.insert(p.clone(), EntryKind::File);
        s.files
            .insert(p, "Must never become the directory index\n".into());
    }
    for keyword in ["disabled", "DISABLED", "DiSaBlEd"] {
        let text = config().replace("welcome.html index.html", keyword);
        assert!(parse(&text).unwrap().hosts[0].indexes.is_empty());
        save_load(&mut s, text);
        let r = hit(&s, "gallery.example.test", "/");
        assert_eq!((r.status.code(), r.cause), (403, Cause::NoIndex));
        assert!(r.body.is_empty());
        assert_eq!(r.resource.as_deref(), Some("/srv/gallery"));
    }
    for mixed in [
        "index.html disabled",
        "DISABLED index.html",
        "index.html DiSaBlEd",
    ] {
        assert_eq!(
            parse(&config().replace("welcome.html index.html", mixed))
                .unwrap_err()
                .kind,
            FaultKind::Unsupported
        );
    }
    save_load(
        &mut s,
        config().replace("welcome.html index.html", "section"),
    );
    assert_eq!(
        request(&s, &req("gallery.example.test", "/"))
            .unwrap_err()
            .kind,
        FaultKind::Unsupported
    );
}

#[test]
fn directory_redirect_missing_file_and_no_listing_are_distinct() {
    let s = running();
    let r = hit(&s, "gallery.example.test", "/section");
    assert_eq!(
        (r.status.code(), r.cause, r.redirect_path.as_deref()),
        (301, Cause::DirectorySlash, Some("/section/"))
    );
    assert!(r.body.is_empty());
    assert_eq!(
        hit(&s, "gallery.example.test", "/section/").status.code(),
        200
    );
    assert_eq!(
        hit(&s, "gallery.example.test", "/empty/").cause,
        Cause::NoIndex
    );
    let r = hit(&s, "gallery.example.test", "/absent.txt");
    assert_eq!((r.status.code(), r.cause), (404, Cause::MissingResource));
    assert_eq!(
        hit(&s, "gallery.example.test", "/index.html/")
            .status
            .code(),
        404
    );
}

#[test]
fn access_uses_directory_ancestry_not_filename_or_sibling_string_prefix() {
    let mut s = running();
    assert_eq!(
        hit(&s, "gallery.example.test", "/private/secret.txt")
            .status
            .code(),
        403
    );
    assert_eq!(
        hit(&s, "gallery.example.test", "/private-notes/note.txt")
            .status
            .code(),
        200
    );
    assert_eq!(
        request(&s, &req("gallery.example.test", "/private/missing.txt"))
            .unwrap_err()
            .kind,
        FaultKind::Unsupported
    );
    let extra = "<Directory \"/srv/gallery/welcome.html\">\nRequire all denied\n</Directory>\n";
    let changed = config().replacen("</VirtualHost>", &format!("{extra}</VirtualHost>"), 1);
    save_load(&mut s, changed);
    assert_eq!(
        hit(&s, "gallery.example.test", "/welcome.html")
            .status
            .code(),
        200,
        "Directory does not match a filename itself"
    );
}

#[test]
fn child_require_replaces_inherited_default_independent_of_section_source_order() {
    let mut s = running();
    let a = "<Directory \"/srv/gallery\">\nRequire all denied\n</Directory>\n<Directory \"/srv/gallery/private\">\nRequire all granted\n</Directory>";
    let old = "<Directory \"/srv/gallery\">\nRequire all granted\n</Directory>\n<Directory \"/srv/gallery/private\">\nRequire all denied\n</Directory>";
    save_load(&mut s, config().replace(old, a));
    assert_eq!(
        hit(&s, "gallery.example.test", "/welcome.html")
            .status
            .code(),
        403
    );
    assert_eq!(
        hit(&s, "gallery.example.test", "/private/secret.txt")
            .status
            .code(),
        200
    );
    let reordered = a
        .split_once("<Directory \"/srv/gallery/private\">")
        .unwrap();
    let reverse = format!(
        "<Directory \"/srv/gallery/private\">{}\n{}",
        reordered.1, reordered.0
    );
    save_load(&mut s, config().replace(old, &reverse));
    assert_eq!(
        hit(&s, "gallery.example.test", "/private/secret.txt")
            .status
            .code(),
        200
    );
}

#[test]
fn head_has_no_body_and_length_counts_utf8_bytes_not_characters() {
    let s = running();
    let before = serde_json::to_vec(&s).unwrap();
    let get = hit(&s, "gallery.example.test", "/section/");
    assert_eq!(get.body, "Café — 展示\n");
    assert!(get.body.len() > get.body.chars().count());
    let mut r = req("gallery.example.test", "/section/");
    r.method = Method::Head;
    let Outcome::Http(head) = request(&s, &r).unwrap() else {
        panic!()
    };
    assert_eq!(head.status, get.status);
    assert_eq!(head.resource, get.resource);
    assert!(head.body.is_empty());
    assert_eq!(head.representation_bytes, Some("Café — 展示\n".len()));
    assert_eq!(serde_json::to_vec(&s).unwrap(), before);
}

#[test]
fn saved_check_reload_and_live_page_content_have_independent_effects() {
    let mut s = running();
    let before_active = s.active.clone();
    let changed = config().replace("welcome.html index.html", "index.html");
    s.apply(&Action::SaveConfig {
        text: changed.clone(),
    })
    .unwrap();
    assert_eq!(hit(&s, "gallery.example.test", "/").body, "Gallery home\n");
    let before_check = s.clone();
    assert_eq!(
        s.check_config().unwrap().hosts[0].indexes,
        vec!["index.html"]
    );
    assert_eq!(s, before_check);
    assert_eq!(s.active, before_active);
    assert!(!s.apply(&Action::Start {}).unwrap());
    s.apply(&Action::WriteFile {
        path: "/srv/gallery/welcome.html".into(),
        text: "New live page\n".into(),
    })
    .unwrap();
    assert_eq!(hit(&s, "gallery.example.test", "/").body, "New live page\n");
    assert_eq!(s.active, before_active);
    assert!(s.apply(&Action::Reload {}).unwrap());
    assert_eq!(s.active.as_ref().unwrap().source, changed);
    assert_eq!(hit(&s, "gallery.example.test", "/").body, "Second choice\n");
    assert!(!s.apply(&Action::Reload {}).unwrap());
    assert_eq!(
        hit(&s, "archive.example.test", "/").body,
        "Archive intact\n"
    );
}

#[test]
fn bad_saved_config_roundtrips_and_failed_reload_preserves_active_and_all_files() {
    let mut s = running();
    s.apply(&Action::SaveConfig {
        text: "<VirtualHost *:8080>\n".into(),
    })
    .unwrap();
    let exact = s.clone();
    assert_eq!(
        s.apply(&Action::Reload {}).unwrap_err().kind,
        FaultKind::Syntax
    );
    assert_eq!(s, exact);
    assert_eq!(
        hit(&s, "archive.example.test", "/").body,
        "Archive intact\n"
    );
    let decoded = State::decode(&serde_json::to_vec(&s).unwrap(), &s.inventory).unwrap();
    assert_eq!(decoded, s);
    assert!(s.apply(&Action::Stop {}).unwrap());
    assert_eq!(
        request(&s, &req("gallery.example.test", "/")).unwrap(),
        Outcome::NoListener
    );
    let stopped = s.clone();
    assert!(s.apply(&Action::Start {}).is_err());
    assert_eq!(s, stopped);
    assert_eq!(
        s.apply(&Action::Reload {}).unwrap_err().kind,
        FaultKind::Unsupported
    );
    assert_eq!(s, stopped);
    s.apply(&Action::SaveConfig { text: config() }).unwrap();
    s.apply(&Action::Start {}).unwrap();
    assert_eq!(hit(&s, "archive.example.test", "/").status.code(), 200);
}

#[test]
fn wellformed_missing_document_root_is_not_syntax_failure_or_host_file_access() {
    let mut s = fixture();
    let text = config().replace("/srv/gallery", "/srv/not-supplied");
    assert!(parse(&text).is_ok());
    s.apply(&Action::SaveConfig { text }).unwrap();
    s.apply(&Action::Start {}).unwrap();
    assert_eq!(
        hit(&s, "gallery.example.test", "/index.html").status.code(),
        404
    );
    assert_eq!(
        hit(&s, "archive.example.test", "/").body,
        "Archive intact\n"
    );
    assert!(request(&s, &req("gallery.example.test", "/../../etc/passwd")).is_err());
}

#[test]
fn canonical_uri_closure_diagnoses_excluded_forms_without_fake_security_success() {
    let s = running();
    let before = s.clone();
    for method in [Method::Get, Method::Head] {
        let mut r = req("gallery.example.test", "/");
        r.method = method;
        assert!(matches!(
            request(&s, &r).unwrap(),
            Outcome::Http(Response {
                status: HttpStatus::Ok,
                ..
            })
        ));
        for spelling in ["//", "///", "/section//", "//index.html"] {
            r.path = spelling.into();
            assert_eq!(
                request(&s, &r).unwrap_err().kind,
                FaultKind::Unsupported,
                "{method:?} {spelling}"
            );
            assert_eq!(s, before);
        }
    }
    for p in [
        "relative",
        "/./index.html",
        "/../index.html",
        "//index.html",
        "/%69ndex.html",
        "/index.html?q=1",
        "/index.html#part",
        "/a\\b",
        "/展示",
    ] {
        assert_eq!(
            request(&s, &req("gallery.example.test", p))
                .unwrap_err()
                .kind,
            FaultKind::Unsupported,
            "{p}"
        );
        assert_eq!(s, before);
    }
}

#[test]
fn closed_parser_rejects_real_unmodeled_forms_instead_of_silent_approximation() {
    for s in [
        config().replace("*:8080", "127.0.0.1:8080"),
        config().replacen("*:8080", "*:8081", 1),
        config().replace("Listen 8080", "Listen 8080\nListen 8081"),
        config().replace(
            "welcome.html index.html",
            "index.html\nDirectoryIndex welcome.html",
        ),
        config().replace(
            "ServerName gallery.example.test",
            "ServerName gallery.example.test\nServerName later.example.test",
        ),
        config().replace("art.example.test", "*.example.test"),
        config().replace("Require all granted", "Require ip 192.0.2.0/24"),
        config().replace("Options None", "Options Indexes"),
        config().replace("AllowOverride None", "AllowOverride All"),
        config().replace("DocumentRoot \"/srv/gallery\"", "DocumentRoot ${SITE}"),
        config().replace("Listen 8080", "Include more.conf\nListen 8080"),
        config().replace("Listen 8080", "Listen 8080 # inline"),
        config().replace("\"/srv/gallery\"", "'/srv/gallery'"),
        config().replace(
            "DocumentRoot \"/srv/gallery\"",
            "DocumentRoot \"/srv/\"\"gallery\"",
        ),
    ] {
        assert_eq!(parse(&s).unwrap_err().kind, FaultKind::Unsupported, "{s}");
    }
    for s in [
        "<VirtualHost *:8080>\n",
        "</Directory>\n",
        "<Directory \"/\">\n<Directory \"/srv\">\n",
        "Listen \"8080\n",
    ] {
        assert_eq!(parse(s).unwrap_err().kind, FaultKind::Syntax);
    }
    assert!(
        parse(
            &config()
                .replace("Listen", "lIsTeN")
                .replace("VirtualHost", "vIrTuAlHoSt")
                .replace("DirectoryIndex", "dIrEcToRyInDeX")
        )
        .is_ok()
    );
}

#[test]
fn active_snapshot_and_inventory_context_are_validated_independently_of_saved_draft() {
    let mut s = running();
    let inv = s.inventory.clone();
    let mut foreign = inv.clone();
    foreign.server = "other".into();
    assert_eq!(s.valid_in(&foreign).unwrap_err().kind, FaultKind::Context);
    assert!(State::decode(&serde_json::to_vec(&s).unwrap(), &foreign).is_err());
    s.saved = "unclosed \" draft\n".into();
    assert!(s.validate().is_ok());
    s.active.as_mut().unwrap().source = "Listen 8080\n".into();
    assert!(s.validate().is_err());
    assert!(State::decode(&serde_json::to_vec(&s).unwrap(), &inv).is_err());
    assert!(serde_json::from_slice::<State>(&serde_json::to_vec(&s).unwrap()).is_err());
}

#[test]
fn immutable_file_keys_types_parents_and_atomic_bad_actions_are_not_repaired_silently() {
    let mut s = running();
    let before = s.clone();
    for action in [
        Action::WriteFile {
            path: "/etc/passwd".into(),
            text: "foreign".into(),
        },
        Action::WriteFile {
            path: "/srv/gallery".into(),
            text: "directory".into(),
        },
        Action::WriteFile {
            path: "/srv/gallery/index.html".into(),
            text: "x".repeat(MAX_FILE + 1),
        },
    ] {
        assert!(s.apply(&action).is_err());
        assert_eq!(s, before);
    }
    let mut wrong = s.clone();
    wrong.files.remove("/srv/gallery/index.html");
    assert!(wrong.validate().is_err());
    let mut wrong = s.clone();
    wrong
        .inventory
        .entries
        .insert("/orphan/file".into(), EntryKind::File);
    assert!(wrong.validate().is_err());
    let mut wrong = s.clone();
    wrong
        .inventory
        .entries
        .insert("/srv".into(), EntryKind::File);
    assert!(wrong.validate().is_err());
    let mut huge = before.clone();
    huge.files
        .get_mut("/srv/gallery/index.html")
        .unwrap()
        .push_str(&"x".repeat(MAX_FILE));
    let exact = huge.clone();
    assert!(huge.apply(&Action::Stop {}).is_err());
    assert_eq!(huge, exact);
}

#[test]
fn strict_json_rejects_unknown_duplicate_fields_maps_and_oversized_inputs() {
    let s = running();
    let value = serde_json::to_value(&s).unwrap();
    let mut extra = value.clone();
    extra["cached_success"] = true.into();
    assert!(State::decode(&serde_json::to_vec(&extra).unwrap(), &s.inventory).is_err());
    let raw = serde_json::to_string(&s).unwrap();
    let dup = raw.replacen("\"saved\":", "\"saved\":\"\",\"saved\":", 1);
    assert!(State::decode(dup.as_bytes(), &s.inventory).is_err());
    // Both duplicate Inventory values are valid EntryKind values, so rejection
    // must come from duplicate-key detection rather than an invalid enum value.
    let inventory_raw = serde_json::to_string(&s.inventory).unwrap();
    let entry = "\"/srv/gallery/index.html\":\"File\"";
    let duplicate_inventory = inventory_raw.replacen(entry, &format!("{entry},{entry}"), 1);
    assert_ne!(duplicate_inventory, inventory_raw);
    let error = serde_json::from_str::<Inventory>(&duplicate_inventory)
        .unwrap_err()
        .to_string();
    assert!(error.contains("duplicate"), "{error}");
    // Target the actual files map, not the identically named Inventory key.
    let split = raw.find("\"files\":").unwrap() + "\"files\":".len();
    let (prefix, tail) = raw.split_at(split);
    let file = format!(
        "\"/srv/gallery/index.html\":{}",
        serde_json::to_string(&s.files["/srv/gallery/index.html"]).unwrap()
    );
    let duplicate_tail = tail.replacen(&file, &format!("{file},{file}"), 1);
    assert_ne!(duplicate_tail, tail);
    let duplicate_files = format!("{prefix}{duplicate_tail}");
    let error = serde_json::from_str::<State>(&duplicate_files)
        .unwrap_err()
        .to_string();
    assert!(error.contains("duplicate"), "{error}");
    assert!(State::decode(duplicate_files.as_bytes(), &s.inventory).is_err());
    for json in [
        r#"{"kind":"Start","extra":true}"#,
        r#"{"kind":"Stop","extra":true}"#,
        r#"{"kind":"Reload","extra":true}"#,
        r#"{"kind":"SaveConfig","text":"","extra":true}"#,
        r#"{"kind":"WriteFile","path":"/a","text":"","extra":true}"#,
        r#"{"kind":"Start","kind":"Stop"}"#,
    ] {
        assert!(Action::decode(json.as_bytes()).is_err(), "{json}");
    }
    assert_eq!(
        Action::decode(br#"{"kind":"Start"}"#).unwrap(),
        Action::Start {}
    );
    assert!(State::decode(&vec![b' '; MAX_STATE_JSON + 1], &s.inventory).is_err());
    let mut r = req("gallery.example.test", "/");
    r.hosts.push("a".into());
    r.hosts.push("b".into());
    assert!(Request::decode(&serde_json::to_vec(&r).unwrap()).is_err());
    let mut r = serde_json::to_value(req("gallery.example.test", "/")).unwrap();
    r["method"] = "POST".into();
    assert!(Request::decode(&serde_json::to_vec(&r).unwrap()).is_err());
}

fn maximal() -> State {
    let mut entries = BTreeMap::from([("/".into(), EntryKind::Directory)]);
    let mut files = BTreeMap::new();
    for i in 0..31 {
        entries.insert(format!("/d{i}"), EntryKind::Directory);
    }
    for i in 0..MAX_FILES {
        let p = format!("/d{}/f{i}", i % 31);
        entries.insert(p.clone(), EntryKind::File);
        files.insert(p, "\0".repeat(2730));
    }
    State::new(
        Inventory {
            fixture: "maximum".into(),
            server: "maximum".into(),
            config_path: "/config/httpd.conf".into(),
            entries,
        },
        config(),
        files,
    )
    .unwrap()
}
#[test]
fn maximum_escaped_state_roundtrips_compact_pretty_and_aggregate_write_boundary() {
    let mut s = maximal();
    assert_eq!(s.inventory.entries.len(), MAX_ENTRIES);
    assert_eq!(s.files.len(), MAX_FILES);
    for raw in [
        serde_json::to_vec(&s).unwrap(),
        serde_json::to_vec_pretty(&s).unwrap(),
    ] {
        assert!(raw.len() < MAX_STATE_JSON);
        assert_eq!(State::decode(&raw, &s.inventory).unwrap(), s);
    }
    let first = s.files.keys().next().unwrap().clone();
    let used: usize = s.files.values().map(String::len).sum();
    let exact = s.files[&first].clone() + &"\0".repeat(MAX_FILE_TEXT - used);
    s.apply(&Action::WriteFile {
        path: first.clone(),
        text: exact.clone(),
    })
    .unwrap();
    assert_eq!(
        s.files.values().map(String::len).sum::<usize>(),
        MAX_FILE_TEXT
    );
    let before = s.clone();
    assert_eq!(
        s.apply(&Action::WriteFile {
            path: first,
            text: exact + "x"
        })
        .unwrap_err()
        .kind,
        FaultKind::Limit
    );
    assert_eq!(s, before);
    let mut over = before.clone();
    over.inventory
        .entries
        .insert("/extra".into(), EntryKind::Directory);
    assert!(over.validate().is_err());
}

#[test]
fn maximum_parser_shape_and_physical_line_limits_are_checked_without_expansion() {
    let mut text = baseline().to_owned();
    for h in 0..MAX_VHOSTS {
        text.push_str(&format!(
            "<VirtualHost *:8080>\nServerName s{h}.example\nServerAlias"
        ));
        for a in 0..8 {
            text.push_str(&format!(" a{h}-{a}.example"));
        }
        text.push_str(&format!("\nDocumentRoot /s{h}\n"));
        for d in 0..if h == 7 { 3 } else { 4 } {
            text.push_str(&format!(
                "<Directory /s{h}/d{d}>\nRequire all granted\n</Directory>\n"
            ));
        }
        text.push_str("</VirtualHost>\n");
    }
    let c = parse(&text).unwrap();
    assert_eq!(c.hosts.len(), 8);
    assert_eq!(c.hosts.iter().map(|h| h.aliases.len()).sum::<usize>(), 64);
    assert_eq!(
        1 + c.hosts.iter().map(|h| h.directories.len()).sum::<usize>(),
        32
    );
    let ninth = text.clone() + &vhost("ninth.example", "/ninth");
    assert_eq!(parse(&ninth).unwrap_err().kind, FaultKind::Limit);
    let extra_dir = text.replacen(
        "</VirtualHost>",
        "<Directory /s0/extra>\nRequire all denied\n</Directory>\n</VirtualHost>",
        1,
    );
    assert_eq!(parse(&extra_dir).unwrap_err().kind, FaultKind::Limit);
    let mut lines = config();
    let used = lines.lines().count();
    for _ in used..MAX_LINES {
        lines.push_str("#\n");
    }
    assert!(parse(&lines).is_ok());
    assert_eq!(parse(&(lines + "#\n")).unwrap_err().kind, FaultKind::Limit);
    assert!(parse(&(config() + "#" + &"a".repeat(MAX_LINE - 1) + "\n")).is_ok());
    assert_eq!(
        parse(&(config() + "#" + &"a".repeat(MAX_LINE) + "\n"))
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
    assert_eq!(
        parse(&"#".repeat(MAX_CONFIG + 1)).unwrap_err().kind,
        FaultKind::Limit
    );
}

#[test]
fn malformed_bounded_drafts_and_action_bytes_preserve_raw_work_exactly() {
    let mut s = fixture();
    let malformed = "Listen\0 deliberately unfinished\n<Directory \"/\n".to_owned();
    assert!(
        s.apply(&Action::SaveConfig {
            text: malformed.clone()
        })
        .unwrap()
    );
    assert_eq!(s.saved, malformed);
    assert!(s.check_config().is_err());
    let raw = serde_json::to_vec(&s).unwrap();
    let restored = State::decode(&raw, &s.inventory).unwrap();
    assert_eq!(restored, s);
    let before = s.clone();
    let too_long = Action::SaveConfig {
        text: "x".repeat(MAX_CONFIG + 1),
    };
    assert!(s.apply(&too_long).is_err());
    assert_eq!(s, before);
    assert!(Action::decode(&serde_json::to_vec(&too_long).unwrap()).is_err());
    assert!(Action::decode(&vec![b' '; MAX_ACTION_JSON + 1]).is_err());
    assert!(Request::decode(&vec![b' '; MAX_REQUEST_JSON + 1]).is_err());
}

#[test]
fn stopped_or_deny_all_is_not_a_substitute_for_positive_healthy_host_outcomes() {
    let mut s = running();
    let positive = |s: &State| {
        hit(s, "archive.example.test", "/").status == HttpStatus::Ok
            && hit(s, "gallery.example.test", "/").status == HttpStatus::Ok
    };
    assert!(positive(&s));
    save_load(
        &mut s,
        config().replace("Require all granted", "Require all denied"),
    );
    assert!(!positive(&s));
    assert_eq!(
        hit(&s, "gallery.example.test", "/private/secret.txt").cause,
        Cause::AccessDenied
    );
    s.apply(&Action::Stop {}).unwrap();
    assert_eq!(
        request(&s, &req("gallery.example.test", "/private/secret.txt")).unwrap(),
        Outcome::NoListener
    );
    // A fresh start consumes saved text, not an old successful response or loaded snapshot.
    s.apply(&Action::SaveConfig { text: config() }).unwrap();
    s.apply(&Action::Start {}).unwrap();
    assert!(positive(&s));
}
