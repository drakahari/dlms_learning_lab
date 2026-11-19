use dlms_learning_lab::ssh_identity::{
    self as identity, AuthorizedKeys, KnownHosts, PublicKey, Trust,
};
// Public-only RFC8032 section7.1 TEST1 data. No private key or crypto code imported.
const A: &str = "AAAAC3NzaC1lZDI1NTE5AAAAINdamAGCsQq31Uv+08lkBzoO4XLz2qYjJa8CGmj3B1Ea";
fn key() -> PublicKey {
    PublicKey::parse("ssh-ed25519", A).unwrap()
}
fn different() -> PublicKey {
    let mut k = key();
    k.bytes[31] ^= 1;
    k
}
#[test]
fn public_wire_shape_matches_the_published_vector_not_only_a_roundtrip() {
    let k = key();
    let public_hex = k
        .bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    assert_eq!(
        public_hex,
        "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"
    );
    assert_eq!(k.encoded(), A);
    for changed in [
        format!("B{}", &A[1..]),
        format!("{}A", &A[..67]),
        format!("{A}="),
        "A".repeat(68),
    ] {
        // A different final public byte is valid shape, unlike corrupt wire headers.
        if changed == format!("{}A", &A[..67]) {
            assert!(PublicKey::parse("ssh-ed25519", &changed).is_ok());
        } else {
            assert!(PublicKey::parse("ssh-ed25519", &changed).is_err());
        }
    }
    assert!(
        PublicKey::parse("ssh-rsa", A)
            .unwrap_err()
            .contains("Unsupported")
    );
}
#[test]
fn host_trust_is_readonly_distinguishes_unknown_changed_and_accepts_any_matching_entry() {
    let text = format!(
        "# supplied identities\nserver.lab.test,192.0.2.10 {} stale\nserver.lab.test {} current\nother.lab.test {}\n",
        different().line(),
        key().line(),
        different().line()
    );
    let h = KnownHosts::parse(&text).unwrap();
    let before = h.clone();
    assert_eq!(h.check("server.lab.test", key()).unwrap(), Trust::Trusted);
    assert_eq!(h.check("192.0.2.10", key()).unwrap(), Trust::ChangedKey);
    assert_eq!(h.check("new.lab.test", key()).unwrap(), Trust::UnknownHost);
    assert_eq!(h, before);
    assert!(Trust::ChangedKey.explanation().contains("verify"));
}
#[test]
fn authorized_user_identity_is_not_server_identity_or_a_comment() {
    let text = format!(
        "# unrelated label\n{} server.lab.test approved-admin\n",
        different().line()
    );
    let keys = AuthorizedKeys::parse(&text).unwrap();
    assert!(!keys.contains(key()));
    assert!(keys.contains(different()));
    let alternate =
        AuthorizedKeys::parse(&format!("  {}\tany comment # ignored\n", key().line())).unwrap();
    assert!(alternate.contains(key()));
    assert!(
        AuthorizedKeys::parse(&format!("from=\"192.0.2.1\" {}", key().line()))
            .unwrap_err()
            .contains("Unsupported")
    );
}
#[test]
fn unsupported_host_grammar_private_material_and_shell_text_fail_honestly() {
    for host in [
        "*.lab.test",
        "!server.lab.test",
        "|1|hash|value",
        "[server.lab.test]:22",
        "SERVER.lab.test",
        "server.lab.test.",
        "192.000.2.1",
        "a/../../etc/passwd",
        "a;id",
        "$(id)",
        "2001:db8::1",
    ] {
        assert!(identity::target(host).is_err(), "{host}");
        assert!(KnownHosts::parse(&format!("{host} {}", key().line())).is_err());
    }
    for text in [
        "-----BEGIN OPENSSH PRIVATE KEY-----",
        "ssh-ed25519 $(cat /etc/passwd)",
        "ssh-ed25519 missing\n",
    ] {
        assert!(AuthorizedKeys::parse(text).is_err());
    }
    for host in ["server.lab.test", "192.0.2.10", "gateway-2.lab.test"] {
        assert!(identity::target(host).is_ok());
    }
}
#[test]
fn parsing_is_bounded_without_allocating_from_encoded_wire_lengths() {
    assert!(AuthorizedKeys::parse(&"A".repeat(identity::MAX_TEXT + 1)).is_err());
    assert!(
        KnownHosts::parse(&format!(
            "server.lab.test {} {}",
            key().line(),
            "x".repeat(1024)
        ))
        .is_err()
    );
    assert!(
        AuthorizedKeys::parse(&format!("{}\n", key().line()).repeat(identity::MAX_ENTRIES + 1))
            .is_err()
    );
    assert!(KnownHosts::parse(&format!("a,b,c,d,e,f,g,h,i {}", key().line())).is_err());
    assert!(AuthorizedKeys::parse(&format!("{}\u{0}", key().line())).is_err());
    assert_eq!(
        KnownHosts::parse("")
            .unwrap()
            .check("server.lab.test", key())
            .unwrap(),
        Trust::UnknownHost
    );
}
