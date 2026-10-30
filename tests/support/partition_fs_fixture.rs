// Different test binaries use subsets of this developer-only fixture helper.
#![allow(dead_code)]
use dlms_learning_lab::{
    pack, partition_fs as core, partition_fs_workspace as work, script_teaching::Concept,
};
use std::collections::BTreeMap;
pub const MIB: u64 = 1024 * 1024;
fn blank() -> core::Disk {
    core::Disk {
        sectors: 131072,
        sector_bytes: 512,
        optimal_alignment: 2048,
        writable: true,
        in_use: false,
        occupied_without_table: false,
        table: None,
    }
}
fn world(first: char, word: &str) -> core::State {
    let names = (0..4)
        .map(|n| format!("/dev/vd{}", char::from_u32(first as u32 + n).unwrap()))
        .collect::<Vec<_>>();
    let mut peer = blank();
    peer.table = Some(core::Gpt {
        partitions: vec![core::Partition {
            number: 1,
            name: "retained".into(),
            start: 2048,
            end: 18431,
            type_hint: core::TypeHint::LinuxData,
            content: core::Content::Ext4 {
                label: "archive".into(),
                uuid: if first == 'a' {
                    "11111111-1111-4111-8111-111111111111"
                } else {
                    "22222222-2222-4222-8222-222222222222"
                }
                .into(),
                blocks: 2048,
                files: BTreeMap::from([(
                    "/keep.txt".into(),
                    core::File {
                        text: format!("{word} original data\n"),
                    },
                )]),
            },
        }],
    });
    core::State {
        version: 1,
        next_uuid: 1,
        disks: BTreeMap::from([
            (names[0].clone(), blank()),
            (names[1].clone(), peer),
            (names[2].clone(), blank()),
            (names[3].clone(), blank()),
        ]),
    }
}
fn target(
    name: &str,
    disks: &[&str],
    bytes: u64,
    hint: core::TypeHint,
    format: work::DesiredContent,
) -> work::Target {
    work::Target {
        name: name.into(),
        eligible_disks: disks.iter().map(|s| (*s).into()).collect(),
        type_hint: hint,
        minimum_bytes: bytes,
        maximum_bytes: bytes * 2,
        aligned_start: true,
        content: format,
    }
}
pub fn operations(
    disk: &str,
    data_name: &str,
    swap_name: &str,
    data_mib: u64,
    swap_mib: u64,
    label: &str,
) -> Vec<work::Operation> {
    let start = 2048;
    let data_end = start + data_mib * MIB / 512 - 1;
    let swap_start = data_end + 1;
    let swap_end = swap_start + swap_mib * MIB / 512 - 1;
    vec![
        work::Operation::CreateGpt { disk: disk.into() },
        work::Operation::AddPartition {
            disk: disk.into(),
            name: data_name.into(),
            start,
            end: data_end,
            type_hint: core::TypeHint::LinuxData,
        },
        work::Operation::Format {
            partition: format!("{disk}1"),
            format: work::FormatChoice::Ext4,
            label: label.into(),
            uuid: None,
        },
        work::Operation::AddPartition {
            disk: disk.into(),
            name: swap_name.into(),
            start: swap_start,
            end: swap_end,
            type_hint: core::TypeHint::LinuxSwap,
        },
        work::Operation::Format {
            partition: format!("{disk}2"),
            format: work::FormatChoice::Swap,
            label: format!("{label}_swap"),
            uuid: None,
        },
    ]
}
pub fn definition() -> work::Definition {
    let initial = world('a', "Assignment");
    let fixture = world('e', "Separate practice");
    let plan = work::Plan {
        targets: vec![
            target(
                "reports",
                &["/dev/vda", "/dev/vdc"],
                8 * MIB,
                core::TypeHint::LinuxData,
                work::DesiredContent::Ext4 {
                    label: Some("reports".into()),
                },
            ),
            target(
                "paging",
                &["/dev/vda", "/dev/vdc"],
                2 * MIB,
                core::TypeHint::LinuxSwap,
                work::DesiredContent::Swap {
                    label: Some("reports_swap".into()),
                },
            ),
        ],
    };
    let practice_plan = work::Plan {
        targets: vec![
            target(
                "sample",
                &["/dev/vde", "/dev/vdg"],
                4 * MIB,
                core::TypeHint::LinuxData,
                work::DesiredContent::Ext4 {
                    label: Some("sample".into()),
                },
            ),
            target(
                "pool",
                &["/dev/vde", "/dev/vdg"],
                MIB,
                core::TypeHint::LinuxSwap,
                work::DesiredContent::Swap {
                    label: Some("sample_swap".into()),
                },
            ),
        ],
    };
    let worked=operations("/dev/vde","sample","pool",4,1,"sample").into_iter().map(|operation|work::WorkedStep{explanation:match &operation{work::Operation::CreateGpt{..}=>"Create a virtual partition table; this does not create any filesystem.",work::Operation::AddPartition{..}=>"Allocate the inclusive sector range, keeping reserved areas and retained peer data unchanged.",work::Operation::Format{format:work::FormatChoice::Ext4,..}=>"Initialize the blank selected partition as ext4; a type hint alone was not formatting.",work::Operation::Format{..}=>"Initialize a swap area; it is not activated or mounted."}.into(),operation}).collect();
    work::Definition{initial,plan,reference:operations("/dev/vda","reports","paging",8,2,"reports"),lesson:work::Lesson{prerequisite_note:"Read the supplied disk geometry and retained-data facts before using virtual preparation tools.".into(),concepts:vec![Concept{title:"Partitions and content are different".into(),text:"A sector is512bytes. Inclusive range size is(end-start+1)times512. An ext4 type hint allocates space; initializing ext4 is a separate operation.".into()},Concept{title:"Protection and recovery".into(),text:"Existing data cannot be overwritten here. A wrong extra practice allocation can be repaired by Reset practice then replaying the native worked steps. This does not claim real disk undo.".into()}],description:"Use different media and smaller capacities in this isolated practice. The retained peer and spare must remain unchanged.".into(),fixture,plan:practice_plan,worked,try_change:"Add an unwanted extra partition after the worked result. Check again, then Reset practice and manually replay the worked steps.".into(),experiment:work::Operation::AddPartition{disk:"/dev/vde".into(),name:"extra".into(),start:12288,end:14335,type_hint:core::TypeHint::LinuxData}}}
}
pub fn scenario() -> pack::Scenario {
    let d = definition();
    let session = work::Session::new(&d).unwrap();
    serde_json::from_value(serde_json::json!({"id":"private-partition-capability","version":1,"track":"LPIC-1","topic":"Virtual disk preparation","unit":"","title":"Prepare supplied virtual storage","summary":"Capability fixture only; not a registered authored lesson.","objective":"Prepare the requested reports and paging areas on eligible blank media while preserving peer data and every unused disk.","retry_guidance":"Compare current partition hints and actual signatures separately; reset if an unwanted allocation cannot be removed by this bounded subset.","difficulty":"Intermediate","minutes":20,"steps":["Inspect supplied media","Allocate and initialize","Check current result"],"hints":["A partition hint is not a filesystem.","Check inclusive sector sizes and actual signatures."],"sources":["https://www.lpi.org/our-certifications/exam-101-102-objectives/"],"simplifications":"Virtual512-byte-sector GPT/ext4/swap preparation only; no host devices, overwrite, deletion, resizing, mounts or swap activation.","initial":{"kind":"PartitionFs","simulation":session},"goal":{"kind":"PartitionFs","definition":d},"solution":["Choose eligible blank virtual media, create GPT, allocate reports and paging ranges, then initialize their distinct ext4 and swap signatures."]})).unwrap()
}
pub fn fresh() -> (pack::Scenario, dlms_learning_lab::progress::Attempt) {
    let s = scenario();
    let mut a = dlms_learning_lab::progress::Attempt::new(&s, 1);
    work::binding::prepare(&mut a, &s).unwrap();
    (s, a)
}
pub fn session(a: &dlms_learning_lab::progress::Attempt) -> &work::Session {
    let dlms_learning_lab::engine::State::PartitionFs { simulation } = &a.state else {
        panic!("Typed disk fixture")
    };
    simulation
}
