use dlms_learning_lab::{
    resource_measurements::{Cpu, Disk, Memory, Network, Snapshot},
    resource_observations::{Error, MAX_INPUT, Observation, Recording},
};
fn observation(at: u64) -> Observation {
    Observation {
        counters: Snapshot {
            boot: "boot-a".into(),
            at_ms: at,
            cores: 2,
            cpu: Cpu {
                ticks: [180, 30, 60, 240, 60, 15, 15, 0],
            },
            memory: Memory {
                total_kib: 8192,
                free_kib: 1024,
                available_kib: 4096,
                swap_total_kib: 2048,
                swap_free_kib: 1536,
            },
            disk: Disk {
                name: "vda".into(),
                read_ops: 10,
                write_ops: 20,
                read_sectors: 100,
                write_sectors: 200,
                read_ms: 30,
                write_ms: 40,
            },
            network: Network {
                name: "eth0".into(),
                rx_bytes: 100,
                tx_bytes: 200,
            },
        },
        runnable: 1,
        blocked: 0,
        buffers_kib: 512,
        cache_kib: 2048,
        reclaimable_kib: 256,
        guest_ticks: [30, 0],
        page_kib: 4,
        swap_in_pages: 75,
        swap_out_pages: 38,
        block_in_kib: 900,
        block_out_kib: 600,
        interrupts: 1200,
        context_switches: 600,
    }
}
fn recording() -> Recording {
    let a = observation(3000);
    let mut b = a.clone();
    b.counters.at_ms = 4000;
    b.counters.cpu.ticks = [260, 50, 100, 260, 80, 25, 25, 0];
    b.guest_ticks = [50, 0];
    b.swap_in_pages += 25;
    b.swap_out_pages += 50;
    b.block_in_kib += 300;
    b.block_out_kib += 400;
    b.interrupts += 500;
    b.context_switches += 600;
    b.counters.memory.free_kib = 1500;
    b.runnable = 3;
    b.blocked = 2;
    Recording {
        version: 1,
        observations: vec![a, b],
    }
}
fn rows(text: &str) -> Vec<Vec<u64>> {
    text.lines()
        .skip(2)
        .map(|l| l.split_whitespace().map(|v| v.parse().unwrap()).collect())
        .collect()
}
#[test]
fn boot_average_interval_deltas_guest_and_instantaneous_fields_are_distinct() {
    let r = recording();
    let out = r.vmstat(0, "vmstat 1 2").unwrap();
    let v = rows(&out);
    assert_eq!(v.len(), 2);
    assert_eq!(
        &v[0][..12],
        &[1, 0, 512, 1024, 512, 2048, 100, 50, 300, 200, 400, 200]
    );
    assert_eq!(&v[0][12..], &[30, 15, 40, 10, 0, 5]);
    assert_eq!(
        &v[1][..12],
        &[3, 2, 512, 1500, 512, 2048, 100, 200, 300, 400, 500, 600]
    );
    assert_eq!(&v[1][12..], &[40, 30, 10, 10, 0, 10]);
    assert_eq!(out, r.vmstat(0, "vmstat -S K 1 2").unwrap());
    assert_eq!(r, recording(), "readings do not mutate the recording");
}
#[test]
fn first_rates_truncate_and_later_rates_round_but_cpu_percentages_round() {
    let mut r = recording();
    r.observations[0].swap_in_pages = 1;
    r.observations[1].swap_in_pages = 2;
    r.observations[1].counters.at_ms = 11000;
    r.observations[1].counters.cpu.ticks = [820, 190, 380, 400, 220, 95, 95, 0];
    r.observations[1].guest_ticks = [190, 0];
    let v = rows(&r.vmstat(0, "vmstat 8 2").unwrap());
    assert_eq!(v[0][6], 1);
    assert_eq!(v[1][6], 1);
}
#[test]
fn finite_requests_reject_shell_syntax_unimplemented_options_and_missing_samples() {
    let r = recording();
    for c in [
        "sh -c vmstat",
        "vmstat; cat /etc/passwd",
        "vmstat $(id)",
        "vmstat 1",
        "vmstat -x",
        "vmstat -S M 1 2",
        "vmstat 0 2",
        "vmstat 1 -1",
        "vmstat\n1 2",
        "vmstat\u{a0}1 2",
    ] {
        assert_eq!(r.vmstat(0, c), Err(Error::Unsupported), "{c}")
    }
    assert_eq!(r.vmstat(0, "vmstat 1 9"), Err(Error::Bound));
    assert_eq!(r.vmstat(0, "vmstat 2 2"), Err(Error::MissingInterval));
    assert_eq!(r.vmstat(32, "vmstat"), Err(Error::Invalid));
    assert_eq!(r.vmstat(0, &"x".repeat(129)), Err(Error::Bound));
}
#[test]
fn source_and_ancillary_resets_never_become_a_rate() {
    for field in 0..4 {
        let mut r = recording();
        match field {
            0 => r.observations[1].counters.boot = "boot-b".into(),
            1 => r.observations[1].swap_in_pages = 0,
            2 => r.observations[1].interrupts = 0,
            _ => r.observations[1].guest_ticks = [0, 0],
        };
        assert_eq!(
            r.vmstat(0, "vmstat 1 2"),
            Err(if field == 0 {
                Error::ChangedSource
            } else {
                Error::Decreased
            })
        );
    }
}
#[test]
fn a_distant_pair_cannot_hide_an_observed_intermediate_counter_reset() {
    let mut r = recording();
    let mut middle = r.observations[0].clone();
    middle.counters.at_ms = 3500;
    middle.context_switches = 0;
    r.observations.insert(1, middle);
    assert_eq!(r.vmstat(0, "vmstat 1 2"), Err(Error::Decreased));
}
#[test]
fn unmeasured_cpu_and_inconsistent_guest_changes_fail_explicitly() {
    let mut r = recording();
    r.observations[1].counters.cpu = r.observations[0].counters.cpu.clone();
    r.observations[1].guest_ticks = r.observations[0].guest_ticks;
    assert_eq!(r.vmstat(0, "vmstat 1 2"), Err(Error::NoCpuInterval));
    let mut r = recording();
    r.observations[1].guest_ticks = [130, 0];
    assert_eq!(r.vmstat(0, "vmstat 1 2"), Err(Error::Invalid));
}
#[test]
fn recording_parse_is_closed_bounded_and_allows_teaching_a_reset() {
    let r = recording();
    let raw = serde_json::to_string(&r).unwrap();
    assert_eq!(Recording::parse(&raw).unwrap(), r);
    assert!(
        Recording::parse(&raw.replacen(
            "\"version\":1",
            "\"version\":1,\"host_path\":\"/etc/passwd\"",
            1
        ))
        .is_err()
    );
    assert_eq!(
        Recording::parse(&" ".repeat(MAX_INPUT + 1)),
        Err(Error::Bound)
    );
    let mut invalid = r.clone();
    invalid.observations = vec![r.observations[0].clone(); 33];
    assert!(Recording::parse(&serde_json::to_string(&invalid).unwrap()).is_err());
    let mut reset = r.clone();
    reset.observations[1].context_switches = 0;
    assert!(Recording::parse(&serde_json::to_string(&reset).unwrap()).is_ok());
}
#[test]
fn invalid_memory_inventory_and_guest_total_are_not_printed_as_measurements() {
    for field in 0..4 {
        let mut r = recording();
        match field {
            0 => r.observations[0].cache_kib = 8192,
            1 => r.observations[0].guest_ticks = [181, 0],
            2 => r.observations[0].counters.at_ms = 0,
            _ => r.version = 2,
        };
        assert!(r.vmstat(0, "vmstat").is_err());
    }
}
#[test]
fn supplied_page_size_changes_swap_rates_without_changing_block_io_units() {
    let mut r = recording();
    for o in &mut r.observations {
        o.page_kib = 64;
    }
    let v = rows(&r.vmstat(0, "vmstat 1 2").unwrap());
    assert_eq!(&v[1][6..10], &[1600, 3200, 300, 400]);
    r.observations[1].page_kib = 4;
    assert_eq!(r.vmstat(0, "vmstat 1 2"), Err(Error::ChangedSource));
    r.observations[0].page_kib = 3;
    assert!(r.validate().is_err());
}

#[test]
fn memory_subset_keeps_free_available_cache_and_reclaimable_meanings_separate() {
    let r = recording();
    let out = r.meminfo(0).unwrap();
    assert!(out.contains("MemAvailable:         4096 kB"));
    assert!(out.contains("MemFree:         1024 kB"));
    assert!(out.contains("Cached:         1792 kB"));
    assert!(out.contains("SReclaimable:          256 kB"));
    assert_eq!(r.meminfo(32), Err(Error::Invalid));
    let mut invalid = r.clone();
    invalid.observations[0].reclaimable_kib = 4096;
    assert!(invalid.meminfo(0).is_err());
}

#[test]
fn large_value_columns_align_with_their_headings_and_remain_separate() {
    fn spans(s: &str) -> Vec<(String, usize)> {
        let mut out = vec![];
        let mut start = None;
        for (i, b) in s.bytes().enumerate() {
            if b.is_ascii_whitespace() {
                if let Some(a) = start.take() {
                    out.push((s[a..i].to_owned(), i));
                }
            } else if start.is_none() {
                start = Some(i);
            }
        }
        if let Some(a) = start {
            out.push((s[a..].to_owned(), s.len()));
        }
        out
    }
    let mut r = recording();
    for o in &mut r.observations {
        o.counters.memory.total_kib = 83_886_080;
        o.counters.memory.free_kib = 12_345_678;
        o.counters.memory.available_kib = 20_000_000;
        o.counters.memory.swap_total_kib = 16_777_216;
        o.counters.memory.swap_free_kib = 1_000_000;
    }
    let text = r.vmstat(0, "vmstat 1 2").unwrap();
    let lines: Vec<_> = text.lines().collect();
    let headings = spans(lines[1]);
    assert_eq!(headings.len(), 18);
    for line in &lines[2..] {
        let values = spans(line);
        assert_eq!(values.len(), 18);
        for (v, h) in values.iter().zip(&headings) {
            assert_eq!(v.1, h.1, "Column {} misaligned: {text}", h.0);
        }
    }
}
