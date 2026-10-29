//! Technical fixture only; no registered lab/course or learner profile.
//! Raw values below are synthetic schema/grading controls, not a hardware capture.
use dlms_learning_lab::{physical_link as core, physical_link_assignment as work};
use std::collections::{BTreeMap, BTreeSet};
pub struct Fixture {
    pub definition: work::Definition,
    pub reference: core::Configuration,
    pub alternate: core::Configuration,
    pub partial: core::Configuration,
    pub neighbor_broken: core::Configuration,
}
pub fn fixture(prefix: &str) -> Fixture {
    let ability = |rate, duplex| core::Ability { rate, duplex };
    let abilities = BTreeSet::from([
        ability(core::Rate::M10, core::Duplex::Half),
        ability(core::Rate::M10, core::Duplex::Full),
        ability(core::Rate::M100, core::Duplex::Half),
        ability(core::Rate::M100, core::Duplex::Full),
        ability(core::Rate::M1000, core::Duplex::Full),
    ]);
    let automatic = core::PortConfig {
        admin_up: true,
        profile: core::Profile::CopperRj45,
        negotiation: core::Negotiation::Auto {
            advertise: abilities.clone(),
        },
    };
    let fixed = core::PortConfig {
        negotiation: core::Negotiation::Fixed {
            ability: ability(core::Rate::M100, core::Duplex::Full),
        },
        ..automatic.clone()
    };
    let names = ["left", "right", "ops-left", "ops-right"].map(|n| format!("{prefix}-{n}"));
    let target = format!("{prefix}-uplink");
    let operations = format!("{prefix}-operations");
    let ports = names
        .iter()
        .map(|name| {
            (
                name.clone(),
                core::PortSpec {
                    device: format!("{name}-device"),
                    label: name.clone(),
                    profiles: BTreeSet::from([core::Profile::CopperRj45]),
                    abilities: abilities.clone(),
                    parallel_detection: true,
                },
            )
        })
        .collect();
    let cables = [
        ("clean", core::Condition::Clean),
        ("bad", core::Condition::ReceiveImpairment { every: 4 }),
        ("ops-wire", core::Condition::Clean),
    ]
    .into_iter()
    .map(|(name, condition)| {
        (
            format!("{prefix}-{name}"),
            core::Cable {
                label: format!("Technical {name} cable"),
                kind: core::CableKind::Cat5e,
                length_m: 30,
                condition,
            },
        )
    })
    .collect();
    let links = BTreeMap::from([
        (
            target.clone(),
            core::LinkSpec {
                a: names[0].clone(),
                b: names[1].clone(),
                required_span_m: 10,
                pattern: core::ProbePattern::DuplexMismatchContention,
            },
        ),
        (
            operations.clone(),
            core::LinkSpec {
                a: names[2].clone(),
                b: names[3].clone(),
                required_span_m: 5,
                pattern: core::ProbePattern::Serialized,
            },
        ),
    ]);
    let topology = core::Topology {
        version: 1,
        ports,
        cables,
        links,
    };
    let mut configuration = core::Configuration {
        ports: names
            .iter()
            .map(|n| (n.clone(), automatic.clone()))
            .collect(),
        cables: BTreeMap::from([
            (target.clone(), format!("{prefix}-bad")),
            (operations.clone(), format!("{prefix}-ops-wire")),
        ]),
    };
    configuration.ports.insert(names[1].clone(), fixed.clone());
    let initial = core::State::new(topology, configuration.clone()).unwrap();
    let observation = work::PortObservation {
        port: names[0].clone(),
        reading: core::SuppliedReading {
            origin: core::ReadingOrigin::ReportedMeasurement,
            note: "Synthetic reported-reading schema control; no actual hardware capture".into(),
            counters: core::ObservedCounters {
                input_packets: 900,
                output_packets: 800,
                crc_errors: 37,
                collisions: 0,
                late_collisions: 7,
            },
        },
    };
    let definition = work::Definition {
        version: 1,
        initial,
        requirements: vec![work::LinkRequirement {
            id: format!("{prefix}-service"),
            label: "Technical bidirectional service".into(),
            link: target.clone(),
            minimum_rate: core::Rate::M100,
            full_duplex: true,
            probes: vec![
                work::ProbeCase {
                    a_frames: 4,
                    b_frames: 8,
                },
                work::ProbeCase {
                    a_frames: 9,
                    b_frames: 12,
                },
            ],
        }],
        protected_ports: BTreeSet::from([names[2].clone(), names[3].clone()]),
        protected_links: BTreeSet::from([operations]),
        retired_ports: BTreeSet::new(),
        observations: vec![observation],
    };
    let mut reference = configuration;
    reference.ports.insert(names[1].clone(), automatic);
    reference
        .cables
        .insert(target.clone(), format!("{prefix}-clean"));
    let mut alternate = reference.clone();
    for name in &names[..2] {
        alternate.ports.insert(name.clone(), fixed.clone());
    }
    let mut partial = reference.clone();
    partial.cables.insert(target, format!("{prefix}-bad"));
    let mut neighbor_broken = reference.clone();
    neighbor_broken.ports.get_mut(&names[2]).unwrap().admin_up = false;
    Fixture {
        definition,
        reference,
        alternate,
        partial,
        neighbor_broken,
    }
}
