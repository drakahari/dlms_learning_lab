//! Focused model checks; catalog integration and native acceptance are separate.
use dlms_learning_lab::virtual_workloads::*;
#[path = "support/virtual_workloads_fixture.rs"]
mod setup;
use setup::fixture;
#[test]
fn derives_normal_and_each_outage_from_placements() {
    let (f, c) = fixture();
    for h in &f.hosts {
        for service in ["web", "database"] {
            assert!(
                f.service_ready(&c, &Experiment::HostOutage { host: h.id.clone() }, service)
                    .unwrap()
            );
        }
    }
    assert!(
        f.observe(&c, &Experiment::Normal)
            .unwrap()
            .iter()
            .all(|o| o.ready)
    );
}
#[test]
fn normal_readiness_does_not_prove_outage_readiness() {
    let (f, mut c) = fixture();
    for p in &mut c.placements {
        p.host = "north".into();
    }
    assert!(
        f.observe(&c, &Experiment::Normal)
            .unwrap()
            .iter()
            .all(|o| o.ready)
    );
    for service in ["web", "database"] {
        assert!(
            !f.service_ready(
                &c,
                &Experiment::HostOutage {
                    host: "north".into()
                },
                service
            )
            .unwrap()
        );
    }
}
#[test]
fn accepts_equivalent_rearrangements_and_invalidates_after_edits() {
    let (f, mut c) = fixture();
    c.placements[2].host = "south".into();
    c.placements[3].host = "north".into();
    assert!(
        f.service_ready(
            &c,
            &Experiment::HostOutage {
                host: "north".into()
            },
            "web"
        )
        .unwrap()
    );
    c.placements[2].up = false;
    assert!(
        !f.service_ready(
            &c,
            &Experiment::HostOutage {
                host: "north".into()
            },
            "web"
        )
        .unwrap()
    );
}
#[test]
fn distinguishes_vm_guest_kernel_from_container_host_kernel() {
    let (f, mut c) = fixture();
    c.placements[0].host = "bsd".into();
    c.placements[2].host = "bsd".into();
    let o = f.observe(&c, &Experiment::Normal).unwrap();
    assert!(o[0].ready);
    assert!(o[0].detail.contains("Windows"));
    assert!(!o[2].ready);
    assert!(o[2].detail.contains("Linux host kernel"));
}
#[test]
fn checks_budget_without_claiming_real_hypervisor_policy() {
    let (mut f, c) = fixture();
    f.hosts[0].memory_mib = 512;
    let o = f.observe(&c, &Experiment::Normal).unwrap();
    assert!(!o[0].ready && !o[2].ready && o[1].ready && o[3].ready);
    assert!(o[0].detail.contains("exercise's"));
}
#[test]
fn rejects_forged_shape_unknown_outage_and_oversized_pack() {
    let (f, mut c) = fixture();
    c.placements[1].workload = c.placements[0].workload.clone();
    assert!(f.validate_configuration(&c).is_err());
    let (_, c) = fixture();
    assert!(
        f.observe(
            &c,
            &Experiment::HostOutage {
                host: "outside".into()
            }
        )
        .is_err()
    );
    let mut v = serde_json::to_value(&f).unwrap();
    v["hosts"] = serde_json::json!([f.hosts[0], f.hosts[0], f.hosts[0], f.hosts[0], f.hosts[0]]);
    assert!(serde_json::from_value::<Facts>(v).is_err());
    let mut v = serde_json::to_value(&f).unwrap();
    v["host_shell"] = serde_json::json!("never");
    assert!(serde_json::from_value::<Facts>(v).is_err());
}
#[test]
fn all_eighty_one_placements_match_an_independent_compatibility_and_outage_oracle() {
    let (f, mut c) = fixture();
    for mut n in 0..81 {
        for p in &mut c.placements {
            p.host = f.hosts[n % 3].id.clone();
            n /= 3;
        }
        for e in std::iter::once(Experiment::Normal).chain(
            f.hosts
                .iter()
                .map(|h| Experiment::HostOutage { host: h.id.clone() }),
        ) {
            let expected: Vec<_> = c
                .placements
                .iter()
                .enumerate()
                .map(|(i, p)| p.host != "bsd" || i < 2)
                .zip(c.placements.iter())
                .map(|(compatible, p)| {
                    compatible
                        && match &e {
                            Experiment::Normal => true,
                            Experiment::HostOutage { host } => p.host != *host,
                        }
                })
                .collect();
            let actual = f.observe(&c, &e).unwrap();
            assert_eq!(actual.iter().map(|o| o.ready).collect::<Vec<_>>(), expected);
            for (service, indexes) in [("database", [0, 1]), ("web", [2, 3])] {
                assert_eq!(
                    f.service_ready(&c, &e, service).unwrap(),
                    indexes.iter().any(|i| expected[*i])
                );
            }
        }
    }
}
