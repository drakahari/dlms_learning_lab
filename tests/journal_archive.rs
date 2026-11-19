use dlms_learning_lab::journal_archive::{Archive, Boot, Observation, Query, Record};

const PATH: &str = "/recovery/copied-journal";
const FAILED: &str = "64071fd4a12548cbb5a6b686d7de48a1";
const LATER: &str = "1b6429a038174859aa9b7dfdb68b0013";

fn record(timestamp: &str, unit: &str, priority: u8, message: &str) -> Record {
    Record {
        timestamp: timestamp.into(),
        unit: unit.into(),
        priority,
        message: message.into(),
    }
}

fn archive() -> Archive {
    Archive {
        directory: PATH.into(),
        boots: vec![
            Boot {
                id: FAILED.into(),
                records: vec![
                    record("2026-02-10T11:10:00Z", "reader.service", 6, "Starting"),
                    record(
                        "2026-02-10T11:10:01Z",
                        "reader.service",
                        3,
                        "Input unavailable",
                    ),
                    record("2026-02-10T11:10:02Z", "writer.service", 2, "Other failure"),
                    record(
                        "2026-02-10T11:10:03Z",
                        "reader.service",
                        2,
                        "Startup aborted",
                    ),
                ],
            },
            Boot {
                id: LATER.into(),
                records: vec![record("2026-02-10T11:14:00Z", "reader.service", 5, "Ready")],
            },
        ],
    }
}

#[test]
fn copied_boot_selection_and_priority_match_semantically() {
    let archive = archive();
    archive.validate().unwrap();
    let list = format!("journalctl --no-pager --directory={PATH} --list-boots");
    let Observation::Boots(boots) = archive.inspect(&Query::parse(&list).unwrap()).unwrap() else {
        panic!("expected a boot list");
    };
    assert_eq!((&*boots[0].id, &*boots[1].id), (FAILED, LATER));
    let short = format!("journalctl -D {PATH} -b {FAILED} -u reader.service -p err -o cat");
    let long = format!(
        "journalctl --unit=reader.service --priority=3 --boot={FAILED} --output=cat --directory={PATH}"
    );
    assert_eq!(Query::parse(&short), Query::parse(&long));
    let Observation::Messages { boot, records } =
        archive.inspect(&Query::parse(&long).unwrap()).unwrap()
    else {
        panic!("expected matching journal entries");
    };
    assert_eq!(boot.id, FAILED);
    assert_eq!(
        records
            .iter()
            .map(|row| row.message.as_str())
            .collect::<Vec<_>>(),
        ["Input unavailable", "Startup aborted"]
    );

    let wrong_boot = short.replace(FAILED, LATER);
    let Observation::Messages { records, .. } = archive
        .inspect(&Query::parse(&wrong_boot).unwrap())
        .unwrap()
    else {
        panic!("expected an empty message result");
    };
    assert!(records.is_empty());
}

#[test]
fn unsupported_and_host_paths_cannot_open_a_journal() {
    let archive = archive();
    for input in [
        "journalctl --list-boots",
        "journalctl -D /recovery/copied-journal --rotate",
        "journalctl -D /recovery/copied-journal --list-boots; echo unsafe",
        "journalctl -D /recovery/copied-journal --list-boots -p err",
        "journalctl -D /recovery/copied-journal --list-boots --list-boots",
        "journalctl -D /recovery/copied-journal -b -1",
        "journalctl -D=/recovery/copied-journal --list-boots",
    ] {
        assert!(Query::parse(input).is_err(), "{input}");
    }
    for path in ["/var/log/journal", "/recovery/../copied-journal"] {
        let query = format!("journalctl -D {path} --list-boots");
        assert!(archive.inspect(&Query::parse(&query).unwrap()).is_err());
    }
}

#[test]
fn malformed_archives_fail_before_use_and_decoding_is_bounded() {
    let mut duplicate = archive();
    duplicate.boots[1].id = FAILED.into();
    assert!(duplicate.validate().is_err());
    let mut timestamp = archive();
    timestamp.boots[0].records[0].timestamp = "2026-02-29T11:10:00Z".into();
    assert!(timestamp.validate().is_err());
    let mut unordered = archive();
    unordered.boots.swap(0, 1);
    assert!(unordered.validate().is_err());
    let mut oversize = archive();
    oversize.boots[0].records = vec![oversize.boots[0].records[0].clone(); 33];
    let serialized = serde_json::to_string(&oversize).unwrap();
    assert!(serde_json::from_str::<Archive>(&serialized).is_err());
}
