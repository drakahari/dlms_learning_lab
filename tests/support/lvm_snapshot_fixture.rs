//! Owned virtual fixture; no host access.
use dlms_learning_lab::lvm::{
    cow::{Bank, Slot},
    snapshot::Model,
    *,
};
use std::collections::BTreeMap;
pub fn plain() -> State {
    State {
        version: 1,
        provisioning: None,
        activation: None,
        identity: None,
        pv_capacity: None,
        snapshots: None,
        devices: BTreeMap::from([
            (
                "/dev/store".into(),
                Device {
                    bytes: 13 * MIB,
                    pv_bytes: None,
                    role: DeviceRole::PhysicalVolume,
                    group: "work".into(),
                },
            ),
            (
                "/dev/foreign".into(),
                Device {
                    bytes: 9 * MIB,
                    pv_bytes: None,
                    role: DeviceRole::Foreign,
                    group: String::new(),
                },
            ),
        ]),
        groups: BTreeMap::from([(
            "work".into(),
            VolumeGroup {
                extent_bytes: MIB,
                volumes: BTreeMap::from([
                    (
                        "report".into(),
                        LogicalVolume {
                            extents: (0..2)
                                .map(|index| Extent {
                                    pv: "/dev/store".into(),
                                    index,
                                })
                                .collect(),
                            filesystem: None,
                        },
                    ),
                    (
                        "reference".into(),
                        LogicalVolume {
                            extents: vec![Extent {
                                pv: "/dev/store".into(),
                                index: 2,
                            }],
                            filesystem: Some(Filesystem {
                                blocks: 256,
                                files: BTreeMap::from([(
                                    "/keep.txt".into(),
                                    File {
                                        contents: "Keep this source record.\n".into(),
                                    },
                                )]),
                            }),
                        },
                    ),
                ]),
            },
        )]),
    }
}
pub fn initial() -> State {
    let mut s = plain();
    let records = (0..8)
        .map(|i| {
            (
                format!("row{i}"),
                (
                    Slot {
                        offset: i * 128 * 1024 + 100,
                        length: 8,
                    },
                    format!("record-{i}"),
                ),
            )
        })
        .collect();
    s.snapshots = Some(Model {
        origins: BTreeMap::from([("work/report".into(), Bank::new(2 * MIB, records).unwrap())]),
    });
    s.validate().unwrap();
    s
}
pub fn bank(s: &State) -> &Bank {
    &s.snapshots.as_ref().unwrap().origins["work/report"]
}
pub fn same_neighbors(s: &State, before: &State) {
    assert_eq!(s.devices, before.devices);
    assert_eq!(
        s.groups["work"].volumes["reference"],
        before.groups["work"].volumes["reference"]
    );
    assert_eq!(
        s.groups["work"].volumes["report"],
        before.groups["work"].volumes["report"]
    );
}
