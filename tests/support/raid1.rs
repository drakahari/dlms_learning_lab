#![allow(dead_code)]
use dlms_learning_lab::raid1::*;
use std::collections::BTreeMap;
pub fn data(prefix: &str) -> Vec<Segment> {
    (0..8)
        .map(|i| Segment {
            file: "/records.txt".into(),
            index: i,
            text: format!("{prefix}:line{i}\n"),
        })
        .collect()
}
pub fn disk(media: Media, content: Vec<Segment>) -> Device {
    Device {
        raw_sectors: 4096,
        usable_sectors: 2048,
        available: true,
        read_only: false,
        protected: false,
        media,
        content,
    }
}
pub fn world() -> State {
    let md = |s: &str| Media::Md { array: s.into() };
    let mut devices = BTreeMap::from([
        ("/dev/vda1".into(), disk(md("/dev/md0"), data("records"))),
        (
            "/dev/vdb1".into(),
            disk(md("/dev/md0"), data("old records")),
        ),
        ("/dev/vdc1".into(), disk(Media::Blank, vec![])),
        ("/dev/vdd1".into(), disk(Media::Blank, vec![])),
        ("/dev/vde1".into(), disk(Media::Blank, vec![])),
        ("/dev/vdf1".into(), disk(Media::Foreign, data("foreign"))),
        ("/dev/vdg1".into(), disk(md("/dev/md1"), data("protected"))),
        ("/dev/vdh1".into(), disk(md("/dev/md1"), data("protected"))),
    ]);
    devices.get_mut("/dev/vde1").unwrap().usable_sectors = 512;
    devices.get_mut("/dev/vdg1").unwrap().protected = true;
    devices.get_mut("/dev/vdh1").unwrap().protected = true;
    let member = |d: &str, s, role| Member {
        device: d.into(),
        slot: Some(s),
        role,
    };
    State {
        version: VERSION,
        devices,
        arrays: BTreeMap::from([
            (
                "/dev/md0".into(),
                Array {
                    component_sectors: 1024,
                    members: vec![
                        member("/dev/vda1", 0, Role::Synced),
                        member("/dev/vdb1", 1, Role::Faulty),
                    ],
                },
            ),
            (
                "/dev/md1".into(),
                Array {
                    component_sectors: 1024,
                    members: vec![
                        member("/dev/vdg1", 0, Role::Synced),
                        member("/dev/vdh1", 1, Role::Synced),
                    ],
                },
            ),
        ]),
    }
}
pub fn cmd(w: &mut State, c: &str) {
    let o = w.execute(c).unwrap();
    assert_eq!(o.status, 0, "{}", o.output);
}
pub fn ready(candidate: &str) -> State {
    let mut w = world();
    cmd(&mut w, "mdadm /dev/md0 --remove /dev/vdb1");
    cmd(&mut w, &format!("mdadm /dev/md0 --add {candidate}"));
    w
}
