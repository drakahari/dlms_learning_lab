use dlms_learning_lab::{bios_loader as b, boot_entry as boot, partition_fs as p};
use std::collections::BTreeMap;
pub const ROOT: &str = "12345678-9abc-def0-1234-56789abcdef0";
const DATA: &str = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
pub fn facts() -> b::Facts {
    let disk = || p::Disk {
        sectors: 262144,
        sector_bytes: 512,
        optimal_alignment: 2048,
        writable: true,
        in_use: false,
        occupied_without_table: false,
        table: None,
        mbr: None,
    };
    let mut geometry = p::State {
        version: 1,
        mbr_profile: Some(p::mbr::Profile::Dos512),
        next_uuid: 1,
        disks: BTreeMap::from([("/dev/vda".into(), disk()), ("/dev/vdb".into(), disk())]),
    };
    for (path, uuid) in [("/dev/vda", ROOT), ("/dev/vdb", DATA)] {
        for line in [
            format!("parted -s {path} mklabel msdos"),
            format!("parted -s {path} unit s mkpart primary ext4 2048s 65535s"),
            format!("mkfs.ext4 -U {uuid} {path}1"),
        ] {
            geometry.execute(&line).unwrap();
        }
        geometry.disks.get_mut(path).unwrap().in_use = true;
    }
    let inventory = boot::Inventory {
        kernels: vec![boot::Kernel {
            path: "/linux".into(),
            release: "7.0".into(),
            builtin_drivers: vec!["ext4".into()],
        }],
        initramfs: vec![boot::Initramfs {
            path: "/initrd".into(),
            release: "7.0".into(),
            drivers: vec!["virtio_blk".into()],
            uuid_lookup: true,
        }],
        roots: vec![boot::Root {
            uuid: ROOT.into(),
            label: "System".into(),
            required_drivers: vec!["ext4".into(), "virtio_blk".into()],
            init_present: true,
        }],
    };
    let menu = format!(
        "set default=system\nmenuentry 'System Linux' --id=system {{\nlinux /linux root=UUID={ROOT} ro\ninitrd /initrd\n}}\n"
    );
    b::Facts {
        geometry,
        boot_disk: "/dev/vda".into(),
        core_sectors: 512,
        directories: vec![
            b::Directory {
                path: "/target/boot".into(),
                disk: "/dev/vda".into(),
                partition: 1,
                writable: true,
                configuration: Some(b::Configuration { menu, inventory }),
            },
            b::Directory {
                path: "/boot".into(),
                disk: "/dev/vdb".into(),
                partition: 1,
                writable: true,
                configuration: None,
            },
        ],
    }
}
