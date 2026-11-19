use dlms_learning_lab::chrony::{Assessment, Sample, Source, observe, parse};
use std::net::Ipv4Addr;

fn fixtures() -> [Source; 3] {
    [
        Source {
            address: Ipv4Addr::new(192, 0, 2, 9),
            stratum: 2,
            poll: 6,
            reach: 0xff,
            assessment: Assessment::Eligible,
            last_good: Some(Sample {
                seconds_ago: 21,
                offset_us: 140,
                uncertainty_us: 820,
            }),
        },
        Source {
            address: Ipv4Addr::new(192, 0, 2, 17),
            stratum: 2,
            poll: 6,
            reach: 3,
            assessment: Assessment::InsufficientSamples,
            last_good: None,
        },
        Source {
            address: Ipv4Addr::new(198, 51, 100, 20),
            stratum: 0,
            poll: 6,
            reach: 0,
            assessment: Assessment::Unreachable,
            last_good: None,
        },
    ]
}

#[test]
fn a_reachable_server_can_be_ineligible_and_a_selected_source_affects_tracking() {
    let inventory = fixtures();
    let initial = observe("server 192.0.2.17 iburst\n", &inventory).unwrap();
    assert_eq!(initial.selected, None);
    assert!(initial.report_sources().lines().any(
        |line| line.starts_with("^? 192.0.2.17") && line.split_whitespace().nth(4) == Some("3")
    ));
    assert!(initial.report_tracking().contains("Not synchronised"));
    let recovered = observe(
        "server 192.0.2.9 iburst\nserver 198.51.100.20\n",
        &inventory,
    )
    .unwrap();
    assert_eq!(recovered.selected, Some(inventory[0].address));
    assert!(recovered.report_sources().contains("^* 192.0.2.9"));
    assert!(recovered.report_sources().contains("377"));
    assert!(recovered.report_sources().contains("+140us"));
    assert!(recovered.report_tracking().contains("C0000209 (192.0.2.9)"));
    assert!(recovered.report_tracking().contains("Stratum         : 3"));
}

#[test]
fn literal_configuration_and_alternate_whitespace_are_bounded() {
    let config = parse("# approved test source\n\n server\t192.0.2.9\tiburst\n").unwrap();
    assert_eq!(config.len(), 1);
    assert_eq!(config[0].address, Ipv4Addr::new(192, 0, 2, 9));
    assert!(config[0].iburst);
    for source in [
        "include /etc/shadow",
        "server ../../host",
        "pool 192.0.2.9",
        "server 192.0.2.9 makestep",
        "server 192.0.2.9\nserver 192.0.2.9",
        "server 192.0.2.1\nserver 192.0.2.2\nserver 192.0.2.3\nserver 192.0.2.4",
        "server 192.0.2.9\0",
    ] {
        assert!(parse(source).is_err(), "{source:?}");
    }
    assert!(parse(&" ".repeat(2049)).is_err());
    assert!(
        observe("server 192.0.2.1", &fixtures())
            .unwrap_err()
            .contains("no supplied observation")
    );
}

#[test]
fn multiple_eligible_sources_and_malformed_observations_do_not_fake_selection() {
    let mut peers = fixtures();
    peers[1] = Source {
        address: peers[1].address,
        ..peers[0]
    };
    let error = observe("server 192.0.2.9\nserver 192.0.2.17\n", &peers).unwrap_err();
    assert!(error.contains("does not compare several eligible servers"));
    peers[1].reach = 0;
    assert!(observe("server 192.0.2.9", &peers).is_err());
    let mut peers = fixtures();
    peers[1].address = peers[0].address;
    assert!(observe("server 192.0.2.9", &peers).is_err());
}
