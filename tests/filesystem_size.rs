//! Explicit size boundaries, UTF8 byte length and malformed operands.
use dlms_learning_lab::filesystem::size_predicate::Criterion;
use dlms_learning_lab::{
    engine::Node,
    filesystem::{Tree, size_predicate},
};
use std::collections::BTreeMap;

fn tree() -> Tree {
    let dir = Node {
        directory: true,
        mode: 0o755,
        owner: "alex".into(),
        group: "staff".into(),
    };
    let mut t = Tree {
        nodes: BTreeMap::from([("/".into(), dir.clone()), ("/work".into(), dir)]),
        files: BTreeMap::new(),
        users: BTreeMap::from([
            ("root".into(), vec!["root".into()]),
            ("alex".into(), vec!["staff".into()]),
        ]),
        user: "alex".into(),
        home: "/work".into(),
        cwd: "/work".into(),
        previous: None,
        umask: 0o022,
        clock: 0,
        modified: BTreeMap::new(),
        command_location: None,
    };
    for (name, body) in [
        ("empty", String::new()),
        ("one", "x".into()),
        ("utf8", "é".into()),
        ("edge", "x".repeat(1024)),
        ("over", "x".repeat(1025)),
    ] {
        let p = format!("/work/{name}");
        t.nodes.insert(
            p.clone(),
            Node {
                directory: false,
                mode: 0o644,
                owner: "alex".into(),
                group: "staff".into(),
            },
        );
        t.files.insert(p, body);
    }
    assert!(t.valid());
    t
}
#[test]
fn strict_comparison_and_rounded_unit_boundaries() {
    for (s, yes, no) in [
        ("1c", vec![1], vec![0, 2]),
        ("+1c", vec![2, 512], vec![0, 1]),
        ("-1c", vec![0], vec![1, 2]),
        ("1k", vec![1, 1023, 1024], vec![0, 1025]),
        ("+1k", vec![1025, 2048], vec![0, 1, 1024]),
        ("-1k", vec![0], vec![1, 1023, 1024]),
        ("0k", vec![0], vec![1, 1024]),
        ("1", vec![1, 511, 512], vec![0, 513]),
        ("1b", vec![1, 512], vec![0, 513]),
        ("1w", vec![1, 2], vec![0, 3]),
        ("1M", vec![1, 1_048_576], vec![0, 1_048_577]),
        ("-1M", vec![0], vec![1, 1_048_575]),
        ("-1048576c", vec![0, 1, 1_048_575], vec![1_048_576]),
        ("+0G", vec![1, 1024], vec![0]),
    ] {
        let c = Criterion::parse(s).unwrap();
        for n in yes {
            assert!(c.matches(n), "{s}/{n}");
        }
        for n in no {
            assert!(!c.matches(n), "{s}/{n}");
        }
    }
}
#[test]
fn byte_lengths_not_character_counts_and_no_arithmetic_overflow() {
    assert!(Criterion::parse("2c").unwrap().matches("é".len()));
    assert!(!Criterion::parse("1c").unwrap().matches("é".len()));
    assert!(
        !Criterion::parse("18446744073709551615G")
            .unwrap()
            .matches(262_144)
    );
    assert!(Criterion::parse("+0b").unwrap().matches(usize::MAX));
}
#[test]
fn malformed_or_unsupported_operands_are_explicit_errors() {
    for s in [
        "",
        "+",
        "-",
        "c",
        "1K",
        "1m",
        "1.5k",
        "--1c",
        "+-1c",
        " 1c",
        "1 c",
        "1c ",
        "1cc",
        "1é",
        "é1c",
        "18446744073709551616c",
        "$(id)",
    ] {
        assert!(Criterion::parse(s).is_err(), "{s}");
    }
}

#[test]
fn actual_opt_in_queries_use_metadata_and_keep_original_console_unchanged() {
    let mut t = tree();
    let old = t.clone();
    for (q, expected) in [
        ("find . -type f -size 0c", "./empty\n"),
        ("find . -type f -size 2c", "./utf8\n"),
        ("find . -type f -size 1k", "./edge\n./one\n./utf8\n"),
        ("find . -type f -size +1k", "./over\n"),
        ("find . -type f -size -1k", "./empty\n"),
        (
            "find . -maxdepth 1 -type f -size +0c -size -3c -name '*e' -print",
            "./one\n",
        ),
        ("find . -maxdepth 0 -type f -size +0c", ""),
    ] {
        assert_eq!(size_predicate::execute(&mut t, q).unwrap(), expected, "{q}");
        assert_eq!(t, old);
        assert!(t.execute(q).is_err(), "Original console gained {q}");
        assert_eq!(t, old);
    }
    for q in [
        "find . -size 1k",
        "find . -type d -size 1k",
        "find . -type f -size 1K",
        "find . -type f -mtime -1",
        "find . -type f -size +1k -delete",
    ] {
        assert!(size_predicate::execute(&mut t, q).is_err(), "{q}");
        assert_eq!(t, old);
    }
    for q in [
        "find . -type f",
        "find . -maxdepth 0",
        "find . -type f -name '*e'",
        "find missing -type f",
    ] {
        let mut copy = t.clone();
        assert_eq!(size_predicate::execute(&mut t, q), copy.execute(q));
        assert_eq!(t, old);
    }
}

#[test]
fn filtering_does_not_drop_permission_diagnostics_or_invent_inaccessible_results() {
    let mut t = tree();
    t.execute("mkdir /work/closed").unwrap();
    t.execute("cp /work/over /work/closed/large").unwrap();
    t.execute("chmod 000 /work/closed").unwrap();
    let args = [".", "-type", "f", "-size", "+1k"].map(str::to_owned);
    let report = t.find_sized(&args).unwrap();
    assert!(!report.complete());
    assert_eq!(report.paths(), ["./over"]);
    assert!(!report.diagnostics().is_empty());
    assert!(size_predicate::execute(&mut t, "find . -type f -size +1k").is_err());
}
