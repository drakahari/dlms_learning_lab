//! Locale, byte conversion, malformed source and resource-bound regressions.
use super::*;
const AVAILABLE: [Locale; 4] = [Locale::C, Locale::Cutf8, Locale::EnUsUtf8, Locale::DeDeUtf8];
#[test]
fn explicit_override_and_empty_values_follow_category_precedence() {
    let mut env = configuration(
        "LANG=de_DE.UTF-8\nLC_NUMERIC=C\nLC_CTYPE=C.UTF-8\n",
        &AVAILABLE,
    )
    .unwrap();
    assert_eq!(env.effective(Category::Messages), Locale::DeDeUtf8);
    assert_eq!(env.effective(Category::Numeric), Locale::C);
    assert_eq!(env.effective(Category::Ctype), Locale::Cutf8);
    env.set(Variable::All, Some(Locale::EnUsUtf8));
    for c in Category::ALL {
        assert_eq!(env.effective(c), Locale::EnUsUtf8);
    }
    env.set(Variable::All, None);
    assert_eq!(env.effective(Category::Numeric), Locale::C);
    env.unset(Variable::Category(Category::Numeric));
    assert_eq!(env.effective(Category::Numeric), Locale::DeDeUtf8);
    env.set(Variable::Lang, None);
    assert_eq!(env.effective(Category::Numeric), Locale::C);
}
#[test]
fn literal_file_does_not_execute_or_accept_process_wide_override() {
    assert_eq!(
        configuration(
            "# supplied\nLANG='de_DE.UTF-8' # display\nLC_NUMERIC=\"C\"\n",
            &AVAILABLE
        )
        .unwrap()
        .effective(Category::Numeric),
        Locale::C
    );
    for text in [
        "LC_ALL=C\n",
        "LANG=$(cat private)",
        "LANG=`id`",
        "export LANG=C",
        "LANG=C;id",
        "LANG=C\nLANG=de_DE.UTF-8",
        "LANG=fr_FR.UTF-8",
        "LANG=C\0",
        "LC_PAPER=C",
    ] {
        assert!(configuration(text, &AVAILABLE).is_err(), "{text:?}");
    }
    assert!(configuration("LANG=de_DE.UTF-8", &[Locale::C]).is_err());
    assert!(configuration("LANG=C", &[Locale::C, Locale::C]).is_err());
    assert!(configuration(&"#".repeat(MAX_CONFIG + 1), &AVAILABLE).is_err());
}
#[test]
fn fixed_point_report_uses_actual_numeric_category_with_zero_negative_and_large_samples() {
    let mut env = configuration("LANG=de_DE.UTF-8", &AVAILABLE).unwrap();
    for (n, expected) in [
        (0, "0,00"),
        (19, "0,19"),
        (-107, "-1,07"),
        (1012500, "10125,00"),
    ] {
        assert_eq!(report_number(n, &env).unwrap(), expected);
    }
    env.set(Variable::Category(Category::Numeric), Some(Locale::Cutf8));
    assert_eq!(report_number(-107, &env).unwrap(), "-1.07");
    assert!(report_number(i64::MIN, &env).is_err());
    assert_eq!(Locale::C.charmap(), "ANSI_X3.4-1968");
    assert_eq!(Locale::Cutf8.charmap(), "UTF-8");
}
#[test]
fn conversion_matches_known_bytes_and_roundtrips_all_latin1_values() {
    use encoding::{Encoding::*, convert};
    let result = convert(b"Caf\xe9;ni\xf1o\n", Latin1, Utf8).unwrap();
    assert_eq!(result.bytes, "Café;niño\n".as_bytes());
    assert!(result.failure.is_none());
    let every = (0..=255).collect::<Vec<u8>>();
    let utf8 = convert(&every, Latin1, Utf8).unwrap();
    assert!(utf8.failure.is_none());
    assert_eq!(convert(&utf8.bytes, Utf8, Latin1).unwrap().bytes, every);
    assert_eq!(convert(b"plain\n", Ascii, Utf8).unwrap().bytes, b"plain\n");
    assert_eq!(
        convert("こんにちは".as_bytes(), Utf8, Utf8).unwrap().bytes,
        "こんにちは".as_bytes()
    );
}
#[test]
fn failed_conversion_keeps_only_actual_prefix_and_reports_the_first_input_offset() {
    use encoding::{Encoding::*, Failure, FailureKind::*, convert};
    for (bytes, from, to, position, kind, prefix) in [
        (
            b"ok\xfftail".as_slice(),
            Utf8,
            Utf8,
            2,
            InvalidInput,
            b"ok".as_slice(),
        ),
        (
            b"a\xe2\x82".as_slice(),
            Utf8,
            Utf8,
            1,
            IncompleteInput,
            b"a".as_slice(),
        ),
        (
            "ab€end".as_bytes(),
            Utf8,
            Latin1,
            2,
            Unrepresentable,
            b"ab".as_slice(),
        ),
        (
            b"yes\xe9".as_slice(),
            Latin1,
            Ascii,
            3,
            Unrepresentable,
            b"yes".as_slice(),
        ),
        (
            b"x\x80".as_slice(),
            Ascii,
            Utf8,
            1,
            InvalidInput,
            b"x".as_slice(),
        ),
        (
            b"\xc0\xaf".as_slice(),
            Utf8,
            Utf8,
            0,
            InvalidInput,
            b"".as_slice(),
        ),
        (
            b"\xed\xa0\x80".as_slice(),
            Utf8,
            Utf8,
            0,
            InvalidInput,
            b"".as_slice(),
        ),
    ] {
        let r = convert(bytes, from, to).unwrap();
        assert_eq!(r.bytes, prefix);
        assert_eq!(r.failure, Some(Failure { position, kind }));
    }
    assert!(convert(&vec![b'A'; encoding::MAX_INPUT + 1], Ascii, Utf8).is_err());
    assert!(encoding::Encoding::parse("UTF-8//IGNORE").is_err());
    assert!(encoding::Encoding::parse("WINDOWS-1252").is_err());
}

#[test]
fn every_category_has_its_own_override_and_nonempty_all_wins_over_each() {
    for selected in Category::ALL {
        let mut env = Environment::default();
        env.set(Variable::Lang, Some(Locale::DeDeUtf8));
        env.set(Variable::Category(selected), Some(Locale::EnUsUtf8));
        for c in Category::ALL {
            assert_eq!(
                env.effective(c),
                if c == selected {
                    Locale::EnUsUtf8
                } else {
                    Locale::DeDeUtf8
                }
            );
        }
        env.set(Variable::All, Some(Locale::C));
        for c in Category::ALL {
            assert_eq!(env.effective(c), Locale::C);
        }
        env.unset(Variable::All);
        assert_eq!(env.effective(selected), Locale::EnUsUtf8);
    }
}

#[test]
fn bounded_saved_bindings_refuse_oversized_unknown_and_duplicate_state() {
    let env = configuration("LANG=C.UTF-8\nLC_NUMERIC=C\n", &AVAILABLE).unwrap();
    let value = serde_json::to_value(&env).unwrap();
    assert_eq!(
        serde_json::from_value::<Environment>(value.clone()).unwrap(),
        env
    );
    let row = value[0].clone();
    assert!(
        serde_json::from_value::<Environment>(serde_json::Value::Array(vec![row.clone(); 9]))
            .is_err()
    );
    let duplicate: Environment =
        serde_json::from_value(serde_json::Value::Array(vec![row.clone(), row])).unwrap();
    assert!(duplicate.validate(&AVAILABLE).is_err());
    assert!(
        serde_json::from_value::<Environment>(
            serde_json::json!([{"variable":{"kind":"lang"},"value":"fr_fr_utf8"}])
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<Environment>(
            serde_json::json!([{"variable":{"kind":"lang"},"value":"c","invented":true}])
        )
        .is_err()
    );
}

#[test]
fn empty_values_comments_and_exact_size_boundary_do_not_invent_a_locale() {
    let env = configuration(
        "LANG= # empty default\nLC_NUMERIC='' # inherit\n",
        &AVAILABLE,
    )
    .unwrap();
    assert_eq!(env.effective(Category::Numeric), Locale::C);
    assert_eq!(
        configuration(&format!("#{}", "x".repeat(MAX_CONFIG - 1)), &AVAILABLE).unwrap(),
        Environment::default()
    );
    assert!(configuration("LANG='C' garbage", &AVAILABLE).is_err());
    assert!(configuration("LANG='C", &AVAILABLE).is_err());
}

#[test]
fn conversion_earliest_error_uses_input_byte_offsets_after_multibyte_characters() {
    use encoding::{Encoding::*, Failure, FailureKind::*, convert};
    let r = convert("é€end".as_bytes(), Utf8, Latin1).unwrap();
    assert_eq!(r.bytes, [0xe9]);
    assert_eq!(
        r.failure,
        Some(Failure {
            position: 2,
            kind: Unrepresentable
        })
    );
    let r = convert(b"\xc3\xa9\xe2\x82\xac\xff", Utf8, Ascii).unwrap();
    assert!(r.bytes.is_empty());
    assert_eq!(
        r.failure,
        Some(Failure {
            position: 0,
            kind: Unrepresentable
        })
    );
    let r = convert(b"\xc3\xa9\xff", Utf8, Utf8).unwrap();
    assert_eq!(r.bytes, "é".as_bytes());
    assert_eq!(
        r.failure,
        Some(Failure {
            position: 2,
            kind: InvalidInput
        })
    );
}

#[test]
fn declared_codec_aliases_and_maximum_input_have_exact_supported_behavior() {
    use encoding::{Encoding, convert};
    for alias in ["UTF-8", "utf8"] {
        assert_eq!(Encoding::parse(alias).unwrap(), Encoding::Utf8);
    }
    for alias in ["ASCII", "US-ASCII", "ANSI_X3.4-1968"] {
        assert_eq!(Encoding::parse(alias).unwrap(), Encoding::Ascii);
    }
    for alias in ["LATIN1", "iso-8859-1", "ISO8859-1"] {
        assert_eq!(Encoding::parse(alias).unwrap(), Encoding::Latin1);
    }
    for alias in ["UTF-8//TRANSLIT", "ISO-8859-15", "WINDOWS-1252", ""] {
        assert!(Encoding::parse(alias).is_err());
    }
    let input = vec![0xff; encoding::MAX_INPUT];
    let result = convert(&input, Encoding::Latin1, Encoding::Utf8).unwrap();
    assert_eq!(result.bytes.len(), encoding::MAX_INPUT * 2);
    assert!(result.failure.is_none());
    assert_eq!(
        convert(
            &result.bytes[..encoding::MAX_INPUT],
            Encoding::Utf8,
            Encoding::Latin1
        )
        .unwrap()
        .bytes
        .len(),
        encoding::MAX_INPUT / 2
    );
}

#[test]
fn display_representation_is_explicit_and_does_not_convert_saved_bytes() {
    let raw = b"Caf\xe9\n".to_vec();
    let before = raw.clone();
    assert_eq!(readable(&raw), "Caf\\xe9\n");
    assert_eq!(raw, before);
    assert_eq!(readable("Café\n".as_bytes()), "Café\n");
    assert_eq!(readable(b"a\0b"), "a\\u{0}b");
}
