//! Paired-counter arithmetic and rejection boundaries; no host observations.
use dlms_learning_lab::resource_measurements::*;
fn sample() -> Snapshot {
    Snapshot {
        boot: "sample-boot".into(),
        at_ms: 1000,
        cores: 2,
        cpu: Cpu {
            ticks: [100, 0, 50, 300, 10, 5, 5, 0],
        },
        memory: Memory {
            total_kib: 8 * 1024 * 1024,
            free_kib: 100 * 1024,
            available_kib: 4 * 1024 * 1024,
            swap_total_kib: 2 * 1024 * 1024,
            swap_free_kib: 1024 * 1024,
        },
        disk: Disk {
            name: "vda".into(),
            read_ops: 10,
            write_ops: 20,
            read_sectors: 100,
            write_sectors: 200,
            read_ms: 50,
            write_ms: 100,
        },
        network: Network {
            name: "eth0".into(),
            rx_bytes: 10000,
            tx_bytes: 20000,
        },
    }
}
fn pair() -> (Snapshot, Snapshot) {
    let a = sample();
    let mut b = a.clone();
    b.at_ms = 3000;
    b.cpu.ticks = [300, 0, 100, 400, 30, 15, 15, 10];
    b.disk.read_ops += 8;
    b.disk.write_ops += 4;
    b.disk.read_sectors += 64;
    b.disk.write_sectors += 32;
    b.disk.read_ms += 24;
    b.disk.write_ms += 24;
    b.network.rx_bytes += 5000;
    b.network.tx_bytes += 1000;
    (a, b)
}
#[test]
fn interval_deltas_normalize_cpu_and_keep_idle_wait_steal_separate() {
    let (a, b) = pair();
    let w = measure(&a, &b).unwrap();
    let deltas = [200u128, 0, 50, 100, 20, 10, 10, 10];
    for (i, share) in w.cpu_shares.iter().enumerate() {
        assert_eq!(share.numerator, deltas[i]);
        assert_eq!(share.denominator, 400);
    }
    assert_eq!(w.cpu_work_share.scaled_floor(10000).unwrap(), 6750);
    assert_eq!(w.available_kib, 4 * 1024 * 1024);
    assert_eq!(w.swap_used_kib, 1024 * 1024);
}
#[test]
fn disk_sectors_always_mean_512_bytes_and_elapsed_time_controls_rates() {
    let (a, b) = pair();
    let w = measure(&a, &b).unwrap();
    assert_eq!(w.read_ops_per_second.scaled_floor(1000).unwrap(), 4000);
    assert_eq!(w.write_ops_per_second.scaled_floor(1000).unwrap(), 2000);
    assert_eq!(w.read_bytes_per_second.scaled_floor(1).unwrap(), 16384);
    assert_eq!(w.write_bytes_per_second.scaled_floor(1).unwrap(), 8192);
    assert_eq!(
        w.mean_completion_ms.unwrap().scaled_floor(100).unwrap(),
        400
    );
    assert_eq!(w.rx_bytes_per_second.scaled_floor(1).unwrap(), 2500);
    assert_eq!(w.tx_bytes_per_second.scaled_floor(1).unwrap(), 500);
    let mut longer = b;
    longer.at_ms = 5000;
    let w = measure(&a, &longer).unwrap();
    assert_eq!(w.read_bytes_per_second.scaled_floor(1).unwrap(), 8192);
    assert_eq!(w.cpu_work_share.scaled_floor(10000).unwrap(), 6750);
}
#[test]
fn idle_zero_traffic_is_distinct_from_an_unmeasured_cpu_interval() {
    let a = sample();
    let mut b = a.clone();
    b.at_ms += 1000;
    assert_eq!(measure(&a, &b), Err(Error::NoCpuInterval));
    b.cpu.ticks[3] += 200;
    let w = measure(&a, &b).unwrap();
    assert_eq!(w.cpu_work_share.scaled_floor(100).unwrap(), 0);
    assert_eq!(w.rx_bytes_per_second.scaled_floor(1).unwrap(), 0);
    assert!(w.mean_completion_ms.is_none());
}
#[test]
fn counter_decrease_is_honest_and_never_silently_changed_to_zero() {
    let (a, b) = pair();
    for field in 0..8 {
        let mut bad = b.clone();
        bad.cpu.ticks[field] = a.cpu.ticks[field].saturating_sub(1);
        if a.cpu.ticks[field] == 0 {
            continue;
        }
        assert_eq!(measure(&a, &bad), Err(Error::Reset));
    }
    for field in 0..8 {
        let mut bad = b.clone();
        match field {
            0 => bad.disk.read_ops = 0,
            1 => bad.disk.write_ops = 0,
            2 => bad.disk.read_sectors = 0,
            3 => bad.disk.write_sectors = 0,
            4 => bad.disk.read_ms = 0,
            5 => bad.disk.write_ms = 0,
            6 => bad.network.rx_bytes = 0,
            _ => bad.network.tx_bytes = 0,
        };
        assert_eq!(measure(&a, &bad), Err(Error::Reset));
    }
    assert_eq!(a, sample());
}
#[test]
fn incompatible_boot_device_interface_or_cpu_inventory_cannot_be_compared() {
    let (a, b) = pair();
    for field in 0..4 {
        let mut bad = b.clone();
        match field {
            0 => bad.boot = "after-reboot".into(),
            1 => bad.cores = 4,
            2 => bad.disk.name = "vdb".into(),
            _ => bad.network.name = "eth1".into(),
        };
        assert_eq!(measure(&a, &bad), Err(Error::Identity));
    }
    for time in [a.at_ms, a.at_ms - 1, a.at_ms + 86_400_001] {
        let mut bad = b.clone();
        bad.at_ms = time;
        assert_eq!(measure(&a, &bad), Err(Error::Time));
    }
}
#[test]
fn memory_availability_is_an_independent_supplied_estimate_and_bounds_are_checked() {
    let (a, mut b) = pair();
    b.memory.available_kib = 0;
    let w = measure(&a, &b).unwrap();
    assert_eq!(w.available_kib, 0);
    assert_eq!(w.swap_used_kib, 1024 * 1024);
    for which in 0..4 {
        let mut bad = b.clone();
        match which {
            0 => bad.memory.free_kib = bad.memory.total_kib + 1,
            1 => bad.memory.available_kib = bad.memory.total_kib + 1,
            2 => bad.memory.swap_free_kib = bad.memory.swap_total_kib + 1,
            _ => bad.memory.total_kib = 0,
        };
        assert_eq!(measure(&a, &bad), Err(Error::Invalid));
    }
}
#[test]
fn exact_large_ratios_and_display_scaling_cannot_overflow_or_accept_zero_denominators() {
    let (mut a, mut b) = pair();
    a.disk.read_sectors = 0;
    b.disk.read_sectors = MAX_COUNTER;
    let w = measure(&a, &b).unwrap();
    assert_eq!(
        w.read_bytes_per_second.numerator,
        u128::from(MAX_COUNTER) * 512 * 1000
    );
    assert_eq!(w.read_bytes_per_second.denominator, 2000);
    assert_eq!(
        w.read_bytes_per_second.scaled_floor(u64::MAX),
        Err(Error::Bound)
    );
    assert_eq!(
        Ratio {
            numerator: 1,
            denominator: 0
        }
        .scaled_floor(100),
        Err(Error::Invalid)
    );
    b.cpu.ticks[0] = MAX_COUNTER + 1;
    assert_eq!(measure(&a, &b), Err(Error::Bound));
}
#[test]
fn bounded_wire_input_rejects_unknown_fields_oversized_series_and_forged_facts() {
    let (a, b) = pair();
    let valid = Series {
        version: 1,
        samples: vec![a, b],
    };
    let text = serde_json::to_string(&valid).unwrap();
    let decoded = Series::from_json(&text).unwrap();
    assert_eq!(decoded, valid);
    assert!(decoded.window(0, 1).is_ok());
    assert!(decoded.window(1, 0).is_err());
    let mut value = serde_json::to_value(&valid).unwrap();
    value["samples"][0]["cpu"]["guest"] = 1.into();
    assert_eq!(Series::from_json(&value.to_string()), Err(Error::Invalid));
    let mut value = serde_json::to_value(&valid).unwrap();
    value["version"] = 2.into();
    assert_eq!(Series::from_json(&value.to_string()), Err(Error::Invalid));
    let oversized = Series {
        version: 1,
        samples: vec![sample(); 33],
    };
    assert!(Series::from_json(&serde_json::to_string(&oversized).unwrap()).is_err());
    assert_eq!(
        Series::from_json(&" ".repeat(MAX_INPUT + 1)),
        Err(Error::Bound)
    );
    let mut value = serde_json::to_value(&valid).unwrap();
    value["samples"][0]["boot"] = "x".repeat(49).into();
    assert_eq!(Series::from_json(&value.to_string()), Err(Error::Invalid));
}

#[test]
fn distant_endpoints_cannot_hide_an_observed_intermediate_reset() {
    let (a, mut b) = pair();
    b.at_ms = 2000;
    b.disk.read_ops = 40;
    let mut c = b.clone();
    c.at_ms = 3000;
    c.disk.read_ops = 5;
    let mut d = c.clone();
    d.at_ms = 4000;
    d.disk.read_ops = 50;
    d.cpu.ticks[0] += 200;
    assert!(measure(&a, &d).is_ok());
    let series = Series {
        version: 1,
        samples: vec![a.clone(), b, c, d],
    };
    assert_eq!(series.window(0, 3), Err(Error::Reset));
    let mut sub_tick = a.clone();
    sub_tick.at_ms += 1;
    let mut after = sub_tick.clone();
    after.at_ms += 1000;
    after.cpu.ticks[3] += 200;
    let okay = Series {
        version: 1,
        samples: vec![a.clone(), sub_tick.clone(), after.clone()],
    };
    assert!(okay.window(0, 2).is_ok());
    sub_tick.network.rx_bytes = 0;
    let bad = Series {
        version: 1,
        samples: vec![a, sub_tick, after],
    };
    assert_eq!(bad.window(0, 2), Err(Error::Reset));
}
