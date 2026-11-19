use dlms_learning_lab::resource_measurements::capacity::*;
fn series() -> Series {
    Series {
        version: 1,
        volume: "reports".into(),
        capacity_kib: 1000,
        samples: vec![
            Sample {
                day: 0,
                used_kib: 400,
            },
            Sample {
                day: 7,
                used_kib: 470,
            },
            Sample {
                day: 14,
                used_kib: 610,
            },
        ],
    }
}
fn offer() -> Offer {
    Offer {
        id: "larger".into(),
        capacity_kib: 1400,
        lead_days: 7,
        cost: 100,
    }
}
fn plan() -> Plan {
    Plan {
        offer: "larger".into(),
        order_day: 14,
    }
}
fn demand() -> Demand {
    Demand {
        id: "expected".into(),
        daily_kib: 20,
        changes: vec![],
    }
}
#[test]
fn recent_and_whole_period_windows_make_different_honest_forecasts() {
    let s = series();
    let recent = s.growth(Window { first: 1, last: 2 }).unwrap();
    let whole = s.growth(Window { first: 0, last: 2 }).unwrap();
    assert_eq!((recent.change_kib, recent.days), (140, 7));
    assert_eq!((whole.change_kib, whole.days), (210, 14));
    assert_eq!(recent.at(&s.samples[2], 24).unwrap(), 810);
    assert_eq!(whole.at(&s.samples[2], 24).unwrap(), 760);
    assert_eq!(
        recent.first_breach(&s.samples[2], 800, 40).unwrap(),
        Some(24)
    );
    assert_eq!(
        whole.first_breach(&s.samples[2], 800, 40).unwrap(),
        Some(27)
    );
}
#[test]
fn projections_round_up_instead_of_repeatedly_truncating_growth() {
    let g = Growth {
        change_kib: 1,
        days: 3,
    };
    let a = Sample {
        day: 5,
        used_kib: 10,
    };
    assert_eq!(
        (6..=9).map(|d| g.at(&a, d).unwrap()).collect::<Vec<_>>(),
        vec![11, 11, 11, 12]
    );
    assert_eq!(g.first_breach(&a, 11, 9).unwrap(), Some(9));
}
#[test]
fn first_breach_matches_bounded_direct_projection_including_equal_reserve_days() {
    for change in -5..=12 {
        for elapsed in 1..=7 {
            for limit in 0..=20 {
                let g = Growth {
                    change_kib: change,
                    days: elapsed,
                };
                let a = Sample {
                    day: 4,
                    used_kib: 10,
                };
                let actual = (4..=40).find(|&d| g.at(&a, d).unwrap() > limit);
                assert_eq!(
                    g.first_breach(&a, limit, 40).unwrap(),
                    actual,
                    "{g:?}, limit{limit}"
                );
            }
        }
    }
}
#[test]
fn flat_and_falling_occupancy_are_stocks_not_counter_reset_errors() {
    let mut s = series();
    s.samples[2].used_kib = 400;
    let g = s.growth(Window { first: 1, last: 2 }).unwrap();
    assert_eq!(g.change_kib, -70);
    assert_eq!(g.at(&s.samples[2], 21).unwrap(), 330);
    assert_eq!(g.first_breach(&s.samples[2], 450, 60).unwrap(), None);
    assert_eq!(
        Growth {
            change_kib: 0,
            days: 7
        }
        .at(&s.samples[2], 30)
        .unwrap(),
        400
    );
    assert_eq!(
        Growth {
            change_kib: -100,
            days: 1
        }
        .at(&s.samples[2], 30)
        .unwrap(),
        0
    );
}
#[test]
fn invalid_windows_dates_sizes_and_public_numeric_extremes_fail_without_panicking() {
    let s = series();
    for w in [
        Window { first: 2, last: 1 },
        Window { first: 1, last: 1 },
        Window { first: 0, last: 99 },
    ] {
        assert_eq!(s.growth(w), Err(Error::Window));
    }
    let a = Sample {
        day: 14,
        used_kib: 610,
    };
    for g in [
        Growth {
            change_kib: 1,
            days: 0,
        },
        Growth {
            change_kib: i128::MIN,
            days: 1,
        },
        Growth {
            change_kib: i128::MAX,
            days: 1,
        },
    ] {
        assert_eq!(g.at(&a, 20), Err(Error::Bound));
    }
    let g = Growth {
        change_kib: 10,
        days: 1,
    };
    assert_eq!(g.at(&a, 13), Err(Error::Bound));
    assert_eq!(g.at(&a, MAX_DAY + 1), Err(Error::Bound));
    assert_eq!(
        g.at(
            &Sample {
                day: 0,
                used_kib: MAX_KIB
            },
            1
        ),
        Err(Error::Bound)
    );
}
#[test]
fn wire_is_bounded_closed_and_validates_every_observation_before_acceptance() {
    let text = serde_json::to_string(&series()).unwrap();
    assert_eq!(Series::parse(&text).unwrap(), series());
    assert_eq!(Series::parse(&" ".repeat(32769)), Err(Error::Bound));
    assert!(
        Series::parse(&text.replace("\"version\":1", "\"version\":1,\"script\":\"rm -rf /\""))
            .is_err()
    );
    for n in 0..6 {
        let mut s = series();
        match n {
            0 => s.version = 2,
            1 => s.samples[1].day = 0,
            2 => s.samples[1].used_kib = 1001,
            3 => s.samples[1].day = MAX_DAY + 1,
            4 => s.volume = "../../private".into(),
            _ => s.samples = vec![],
        };
        assert!(s.validate().is_err());
    }
}
#[test]
fn arrival_lead_time_and_required_reserve_are_checked_for_every_virtual_day() {
    let s = series();
    let rows = replay(&s, &offer(), &plan(), &demand(), 200, 20).unwrap();
    assert_eq!(rows.len(), 21);
    assert_eq!(rows[6].capacity_kib, 1000);
    assert_eq!(rows[7].capacity_kib, 1400);
    assert!(rows.iter().all(|d| d.reserve_met));
    let mut p = plan();
    p.order_day = 20;
    let bad = replay(&s, &offer(), &p, &demand(), 200, 20).unwrap();
    assert_eq!(bad.iter().find(|d| !d.reserve_met).unwrap().day, 24);
    assert_eq!(
        bad.last().unwrap().capacity_kib,
        1400,
        "Eventually arriving does not erase an earlier failure"
    );
}
#[test]
fn known_future_workload_steps_are_not_silently_folded_into_a_daily_average() {
    let s = series();
    let mut d = demand();
    d.changes = vec![
        Change {
            day: 18,
            added_kib: 200,
        },
        Change {
            day: 18,
            added_kib: 10,
        },
    ];
    let rows = replay(&s, &offer(), &plan(), &d, 200, 20).unwrap();
    assert_eq!(rows[3].used_kib, 670);
    assert_eq!(rows[4].used_kib, 900);
    assert!(!rows[4].reserve_met);
    let faster = Offer {
        lead_days: 3,
        capacity_kib: 1500,
        ..offer()
    };
    assert!(
        replay(&s, &faster, &plan(), &d, 200, 20)
            .unwrap()
            .iter()
            .all(|d| d.reserve_met)
    );
}
#[test]
fn bigger_and_faster_alternatives_can_both_be_valid_but_one_easy_case_does_not_prove_a_plan() {
    let s = series();
    let cases = [
        demand(),
        Demand {
            id: "busier".into(),
            daily_kib: 30,
            changes: vec![Change {
                day: 22,
                added_kib: 30,
            }],
        },
    ];
    for cap in [1400, 1500] {
        let o = Offer {
            capacity_kib: cap,
            lead_days: 3,
            ..offer()
        };
        assert!(cases.iter().all(|d| {
            replay(&s, &o, &plan(), d, 100, 20)
                .unwrap()
                .iter()
                .all(|r| r.reserve_met)
        }));
    }
    let small = Offer {
        capacity_kib: 1100,
        lead_days: 3,
        ..offer()
    };
    assert!(
        replay(&s, &small, &plan(), &cases[0], 50, 20)
            .unwrap()
            .iter()
            .all(|d| d.reserve_met)
    );
    assert!(
        replay(&s, &small, &plan(), &cases[1], 50, 20)
            .unwrap()
            .iter()
            .any(|d| !d.reserve_met)
    );
}
#[test]
fn replay_rejects_impossible_orders_unknown_offers_and_unbounded_work() {
    let s = series();
    let mut p = plan();
    p.order_day = 13;
    assert!(replay(&s, &offer(), &p, &demand(), 200, 20).is_err());
    p = plan();
    p.offer = "not-an-offer".into();
    assert!(replay(&s, &offer(), &p, &demand(), 200, 20).is_err());
    for horizon in [0, 91, u32::MAX] {
        assert!(replay(&s, &offer(), &plan(), &demand(), 200, horizon).is_err());
    }
    for reserve in [0, MAX_KIB + 1, u64::MAX] {
        assert!(replay(&s, &offer(), &plan(), &demand(), reserve, 20).is_err());
    }
    let d = Demand {
        daily_kib: u64::MAX,
        ..demand()
    };
    assert!(replay(&s, &offer(), &plan(), &d, 200, 20).is_err());
    let o = Offer {
        capacity_kib: 999,
        ..offer()
    };
    assert!(replay(&s, &o, &plan(), &demand(), 200, 20).is_err());
}

#[test]
fn a_larger_reserve_is_valid_when_instant_capacity_really_supports_it() {
    let s = series();
    let immediate = Offer {
        capacity_kib: 2400,
        lead_days: 0,
        ..offer()
    };
    assert!(
        replay(&s, &immediate, &plan(), &demand(), 1200, 20)
            .unwrap()
            .iter()
            .all(|d| d.reserve_met)
    );
    let late = Offer {
        lead_days: 1,
        ..immediate
    };
    let rows = replay(&s, &late, &plan(), &demand(), 1200, 20).unwrap();
    assert!(!rows[0].reserve_met);
    assert!(rows.iter().skip(1).all(|d| d.reserve_met));
    assert!(
        replay(&s, &offer(), &plan(), &demand(), 1001, 20)
            .unwrap()
            .iter()
            .any(|d| !d.reserve_met)
    );
}
