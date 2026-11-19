//! Serialize fixed authored model fixtures using the actual runtime schema.
//! The normal test checks round trips; --nocapture is a development authoring aid.
#[path = "support/initramfs_fixture.rs"]
mod fixture;
use dlms_learning_lab::initramfs_workspace::{Definition, Kind, Session};
#[test]
fn fixed_authoring_fixtures_serialize_the_actual_source_bound_schema() {
    let mut values = vec![];
    for (kind, id) in [
        (Kind::Drivers, "lpic2-initramfs-root-drivers"),
        (Kind::Handover, "lpic2-initramfs-kernel-handover"),
    ] {
        let mut assignment = fixture::task(kind, false);
        assignment.id = id.into();
        assignment.instructions=if kind==Kind::Drivers{"Save the driver-selection fragment and regenerate early userspace for supplied kernel 6.6.61. Make the selected candidate reach the supplied NVMe/ext4 root and keep every working fallback file unchanged."}else{"Prepare early userspace matching supplied kernel 6.6.61 and select it with a saved candidate entry. Verify root and init handoff; preserve the working 6.6.60 fallback files."}.into();
        let mut example = fixture::task(kind, true);
        example.id = format!("{id}-example");
        let definition = Definition {
            version: 1,
            source_id: id.into(),
            assignment,
            example,
        };
        let simulation = Session::new(&definition).unwrap();
        let raw = serde_json::to_string(&simulation).unwrap();
        let roundtrip: Session = serde_json::from_str(&raw).unwrap();
        assert_eq!(roundtrip, simulation);
        roundtrip.bind(&definition).unwrap();
        values.push(serde_json::json!({"definition":definition,"simulation":simulation}));
    }
    println!(
        "INITRAMFS_AUTHORED_FIXTURE={}",
        serde_json::to_string(&values).unwrap()
    );
}
