//! Closed197→193/52→51 predecessor for tests and developer QA; never grading.
#![allow(dead_code)] // Each integration/native binary imports different closed subsets.
use dlms_learning_lab::{
    course::Unit,
    developer_digest as digest,
    engine::{Goal, State},
    pack::{Pack, Scenario},
};
use serde_json::Value;
use std::sync::OnceLock;
pub const IDS: [&str; 4] = [
    "lpic1-disk-safe-gap",
    "lpic1-disk-hints-versus-content",
    "lpic1-disk-eligible-media",
    "lpic1-disk-delivery",
];
struct Expected {
    ids: Vec<(String, u32)>,
    old_units: Vec<Unit>,
    old_values: Vec<Value>,
    unit: Unit,
    unit_value: Value,
    new: Vec<Scenario>,
}
fn recipe() -> &'static Value {
    static R: OnceLock<Value> = OnceLock::new();
    R.get_or_init(|| {
        let raw = include_bytes!("../fixtures/catalog197-registration.json");
        assert_eq!(
            digest::hash(raw),
            "ddc64d8b6c5dce8d4066f0531700de253063c943bb347b1543ed4d4a106390ee"
        );
        serde_json::from_slice(raw).unwrap()
    })
}
pub fn before_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let r = recipe();
    if raw.len() as u64 != r["after_units52_bytes"].as_u64().ok_or("52length")?
        || digest::hash(raw) != r["after_units52_sha256"].as_str().ok_or("52pin")?
    {
        return Err("Exact disk52 raw unit bytes differ".into());
    }
    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
    let head = text
        .strip_suffix(r["append_suffix_utf8"].as_str().ok_or("52suffix")?)
        .ok_or("Exact disk unit append differs")?;
    let before = format!("{head}{}", r["old_tail_utf8"].as_str().ok_or("51tail")?).into_bytes();
    if before.len() as u64 != r["before_units51_bytes"].as_u64().ok_or("51length")?
        || digest::hash(&before) != r["before_units51_sha256"].as_str().ok_or("51pin")?
        || before.as_slice() != include_bytes!("../fixtures/disk-units-before193.json")
    {
        return Err("Complete reviewed193 raw51 differs".into());
    }
    let mut expected: Value = serde_json::from_slice(&before).map_err(|e| e.to_string())?;
    if expected != r["before_units51"] {
        return Err("Complete51 source objects differ".into());
    }
    let units = expected["units"].as_array_mut().ok_or("51units")?;
    if units.len() != 51 {
        return Err("51count".into());
    }
    units.push(r["disk_unit"].clone());
    if serde_json::from_slice::<Value>(raw).map_err(|e| e.to_string())? != expected {
        return Err("Change outside complete disk unit append".into());
    }
    Ok(before)
}
pub fn prefix_units(raw: &[u8]) -> Result<Option<Vec<u8>>, String> {
    if raw.len() as u64 == recipe()["after_units52_bytes"].as_u64().unwrap()
        && digest::hash(raw) == recipe()["after_units52_sha256"].as_str().unwrap()
    {
        before_units(raw).map(Some)
    } else {
        Ok(None)
    }
}
fn expected() -> &'static Expected {
    static E: OnceLock<Expected> = OnceLock::new();
    E.get_or_init(|| {
        let r = recipe();
        let raw = include_bytes!("../../scenarios/disk-preparation-v1.json");
        assert_eq!(
            raw.len() as u64,
            r["canonical_disk_pack_bytes"].as_u64().unwrap()
        );
        assert_eq!(
            digest::hash(raw),
            r["canonical_disk_pack_sha256"].as_str().unwrap()
        );
        let pack: Pack = serde_json::from_slice(raw).unwrap();
        assert_eq!(pack.scenarios.len(), 4);
        let ids = r["before_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                (
                    v["id"].as_str().unwrap().into(),
                    v["version"].as_u64().unwrap() as u32,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(ids.len(), 193);
        Expected {
            ids,
            old_units: serde_json::from_value(r["before_units51"]["units"].clone()).unwrap(),
            old_values: r["before_units51"]["units"].as_array().unwrap().clone(),
            unit: serde_json::from_value(r["disk_unit"].clone()).unwrap(),
            unit_value: r["disk_unit"].clone(),
            new: pack.scenarios,
        }
    })
}
/// Independent complete193 reconstruction, never pack::bundled/current197.
/// Reuse the canonical historical owner rather than loading duplicate modules.
pub fn original193() -> Vec<Scenario> {
    let raw = include_bytes!("../fixtures/catalog193-registration.json");
    assert_eq!(
        digest::hash(raw),
        "c26dec73f0727ce2ee24d76b991204e3114be028be41cd562397714b7c1ced55"
    );
    let r: Value = serde_json::from_slice(raw).unwrap();
    let mut all = super::original190::load();
    assert_eq!(all.len(), 190);
    let bytes = include_bytes!("../../scenarios/shell-ere-evidence-v1.json");
    assert_eq!(
        digest::hash(bytes),
        r["canonical_ere_pack_sha256"].as_str().unwrap()
    );
    let pack: Pack = serde_json::from_slice(bytes).unwrap();
    assert_eq!(pack.scenarios.len(), 3);
    all.extend(pack.scenarios);
    assert_eq!(all.len(), 193);
    assert_eq!(
        all.iter().map(|s| (&s.id, s.version)).collect::<Vec<_>>(),
        expected()
            .ids
            .iter()
            .map(|(id, v)| (id, *v))
            .collect::<Vec<_>>()
    );
    all
}
fn same_scenario(a: &Scenario, b: &Scenario) {
    // Exhaustive current normal197 source schema; reject every foreign family.
    // Schema additions require an explicit new source review.
    let Scenario {
        physical_link_teaching: _,
        one_off_teaching: _,
        file_backup_teaching: _,
        etherchannel_teaching: _,
        cron_teaching: _,
        nfs_teaching: _,
        nat_teaching: _,
        rpm_packages_teaching: _,
        apache_static_teaching: _,
        spanning_tree_teaching: _,
        dhcpv4_teaching: _,
        id: _,
        version: _,
        track: _,
        topic: _,
        unit: _,
        title: _,
        summary: _,
        objective: _,
        retry_guidance: _,
        difficulty: _,
        minutes: _,
        steps: _,
        guide: _,
        explanation: _,
        hints: _,
        sources: _,
        simplifications: _,
        initial: _,
        goal: _,
        solution: _,
        prerequisites: _,
        teaching: _,
        terminal_teaching: _,
        process_teaching: _,
        accounts_teaching: _,
        unit_manager_teaching: _,
        ospf_teaching: _,
        discovery_teaching: _,
        storage_teaching: _,
        acl_teaching: _,
        ipv6_teaching: _,
        ipv4_routing_teaching: _,
        address_teaching: _,
        service_teaching: _,
        client_teaching: _,
        seed_dns_teaching: _,
        seed_mount_teaching: _,
        legacy_access_teaching: _,
        legacy_logs_teaching: _,
        dns_teaching: _,
        switching_teaching: _,
        mac_teaching: _,
        debian_packages_teaching: _,
    } = a;
    assert!(
        a.physical_link_teaching.is_none()
            && b.physical_link_teaching.is_none()
            && a.one_off_teaching.is_none()
            && b.one_off_teaching.is_none()
            && a.file_backup_teaching.is_none()
            && b.file_backup_teaching.is_none()
            && a.etherchannel_teaching.is_none()
            && b.etherchannel_teaching.is_none()
            && a.cron_teaching.is_none()
            && b.cron_teaching.is_none()
            && a.nfs_teaching.is_none()
            && b.nfs_teaching.is_none()
            && a.nat_teaching.is_none()
            && b.nat_teaching.is_none()
            && a.rpm_packages_teaching.is_none()
            && b.rpm_packages_teaching.is_none()
            && a.apache_static_teaching.is_none()
            && b.apache_static_teaching.is_none()
            && a.spanning_tree_teaching.is_none()
            && b.spanning_tree_teaching.is_none()
            && a.dhcpv4_teaching.is_none()
            && b.dhcpv4_teaching.is_none()
            && a.terminal_teaching.is_none()
            && b.terminal_teaching.is_none()
            && a.process_teaching.is_none()
            && b.process_teaching.is_none()
            && a.accounts_teaching.is_none()
            && b.accounts_teaching.is_none()
            && a.unit_manager_teaching.is_none()
            && b.unit_manager_teaching.is_none()
            && a.ospf_teaching.is_none()
            && b.ospf_teaching.is_none()
            && a.discovery_teaching.is_none()
            && b.discovery_teaching.is_none()
            && a.storage_teaching.is_none()
            && b.storage_teaching.is_none()
            && a.acl_teaching.is_none()
            && b.acl_teaching.is_none()
            && a.ipv6_teaching.is_none()
            && b.ipv6_teaching.is_none()
            && a.ipv4_routing_teaching.is_none()
            && b.ipv4_routing_teaching.is_none()
            && a.address_teaching.is_none()
            && b.address_teaching.is_none()
            && a.service_teaching.is_none()
            && b.service_teaching.is_none()
            && a.client_teaching.is_none()
            && b.client_teaching.is_none()
            && a.seed_dns_teaching.is_none()
            && b.seed_dns_teaching.is_none()
            && a.seed_mount_teaching.is_none()
            && b.seed_mount_teaching.is_none()
            && a.legacy_access_teaching.is_none()
            && b.legacy_access_teaching.is_none()
            && a.legacy_logs_teaching.is_none()
            && b.legacy_logs_teaching.is_none()
            && a.dns_teaching.is_none()
            && b.dns_teaching.is_none()
            && a.switching_teaching.is_none()
            && b.switching_teaching.is_none()
            && a.mac_teaching.is_none()
            && b.mac_teaching.is_none()
            && a.debian_packages_teaching.is_none()
            && b.debian_packages_teaching.is_none()
    );
    macro_rules! same{($($f:ident),+)=>{$(assert_eq!(&a.$f,&b.$f);)+};}
    same!(
        id,
        version,
        track,
        topic,
        unit,
        title,
        summary,
        objective,
        retry_guidance,
        difficulty,
        minutes,
        steps,
        explanation,
        hints,
        sources,
        simplifications,
        solution,
        prerequisites
    );
    assert_eq!(a.guide.len(), b.guide.len());
    for (a, b) in a.guide.iter().zip(&b.guide) {
        assert_eq!((&a.title, &a.why, &a.action), (&b.title, &b.why, &b.action));
        assert_eq!(a.commands.len(), b.commands.len());
        for (a, b) in a.commands.iter().zip(&b.commands) {
            assert_eq!(
                (&a.command, &a.explanation, a.simulator_only),
                (&b.command, &b.explanation, b.simulator_only)
            );
        }
    }
    assert!(
        matches!(a.initial, State::PartitionFs { .. })
            && matches!(b.initial, State::PartitionFs { .. })
    );
    assert_eq!(a.initial, b.initial);
    match (&a.goal, &b.goal) {
        (Goal::PartitionFs { definition: a }, Goal::PartitionFs { definition: b }) => {
            assert_eq!(
                serde_json::to_value(a).unwrap(),
                serde_json::to_value(b).unwrap()
            )
        }
        _ => panic!("Canonical virtual disk Goal kind"),
    }
    assert!(a.teaching.is_none() && b.teaching.is_none());
}

fn original193_values() -> &'static Vec<Value> {
    static V: OnceLock<Vec<Value>> = OnceLock::new();
    V.get_or_init(|| {
        original193()
            .iter()
            .map(|s| serde_json::to_value(s).unwrap())
            .collect()
    })
}
pub fn before_scenarios(all: &[Scenario]) -> &[Scenario] {
    assert_eq!(all.len(), 197);
    // Cold expected objects are cached. Compare one bounded old Scenario at a
    // time rather than cloning/serializing the complete catalog per call.
    for (actual, original) in all[..193].iter().zip(original193_values()) {
        assert!(
            serialized_equal(actual, original),
            "Complete prior source differs for {}",
            actual.id
        );
    }
    let e = expected();
    for (a, (id, v)) in all[..193].iter().zip(&e.ids) {
        assert_eq!((&a.id, a.version), (id, *v));
    }
    assert_eq!(
        all[193..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    for (a, b) in all[193..].iter().zip(&e.new) {
        same_scenario(a, b);
    }
    &all[..193]
}
pub fn prefix_scenarios(all: &[Scenario]) -> Option<&[Scenario]> {
    if all.len() == 197 {
        Some(before_scenarios(all))
    } else {
        None
    }
}
fn same_unit(a: &Unit, b: &Unit) {
    let Unit {
        id: _,
        track: _,
        title: _,
        summary: _,
        prerequisites: _,
        labs: _,
    } = a;
    assert_eq!(
        (
            &a.id,
            &a.track,
            &a.title,
            &a.summary,
            &a.prerequisites,
            &a.labs
        ),
        (
            &b.id,
            &b.track,
            &b.title,
            &b.summary,
            &b.prerequisites,
            &b.labs
        )
    );
}
pub fn before_typed_units(all: &[Unit]) -> &[Unit] {
    assert_eq!(all.len(), 52);
    let e = expected();
    for (a, b) in all[..51].iter().zip(&e.old_units) {
        same_unit(a, b);
    }
    same_unit(&all[51], &e.unit);
    &all[..51]
}
pub fn prefix_typed_units(all: &[Unit]) -> Option<&[Unit]> {
    if all.len() == 52 {
        Some(before_typed_units(all))
    } else {
        None
    }
}
pub fn before_values(all: &[Value]) -> &[Value] {
    assert_eq!(all.len(), 52);
    let e = expected();
    assert_eq!(&all[..51], &e.old_values);
    assert_eq!(&all[51], &e.unit_value);
    &all[..51]
}
pub fn prefix_values(all: &[Value]) -> Option<&[Value]> {
    if all.len() == 52 {
        Some(before_values(all))
    } else {
        None
    }
}
/// Count only: inspect every current key/source version and new family context.
/// Never replace the whole map or treat this count as completion evidence.
pub fn before_attempts_len(
    all: &std::collections::BTreeMap<String, dlms_learning_lab::progress::Attempt>,
) -> usize {
    assert_eq!(all.len(), 197);
    let e = expected();
    for (id, v) in &e.ids {
        assert_eq!(all[id].scenario_version, *v);
    }
    for s in &e.new {
        let a = &all[&s.id];
        assert_eq!(a.scenario_version, s.version);
        assert!(matches!(a.state, State::PartitionFs { .. }));
        assert!(a.ipv4.is_none());
        dlms_learning_lab::partition_fs_workspace::context(a, s).unwrap();
    }
    193
}

// Compare every serialized field on every call, without constructing the actual
// Value tree. Cached Value is only the independent immutable expected source.
// This test/developer helper does not run in learner rendering or grading.
mod borrowed_json {
    use serde::{Serialize, Serializer, ser};
    use serde_json::{Map, Value};
    use std::fmt;

    #[derive(Debug)]
    pub struct Different;
    impl fmt::Display for Different {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("Serialized value differs from the complete expected JSON")
        }
    }
    impl std::error::Error for Different {}
    impl ser::Error for Different {
        fn custom<T: fmt::Display>(_: T) -> Self {
            Self
        }
    }
    type Result<T> = std::result::Result<T, Different>;
    fn require(ok: bool) -> Result<()> {
        if ok { Ok(()) } else { Err(Different) }
    }
    pub(super) fn equal<T: ?Sized + Serialize>(actual: &T, expected: &Value) -> bool {
        actual.serialize(Against(expected)).is_ok()
    }
    struct DisplayAgainst<'a> {
        expected: &'a str,
        at: usize,
        failed: bool,
    }
    impl fmt::Write for DisplayAgainst<'_> {
        fn write_str(&mut self, text: &str) -> fmt::Result {
            if self.failed {
                return Err(fmt::Error);
            }
            let Some(end) = self.at.checked_add(text.len()) else {
                self.failed = true;
                return Err(fmt::Error);
            };
            if self.expected.as_bytes().get(self.at..end) != Some(text.as_bytes()) {
                self.failed = true;
                return Err(fmt::Error);
            }
            self.at = end;
            Ok(())
        }
    }
    fn same_display<T: ?Sized + fmt::Display>(value: &T, expected: &str) -> bool {
        use fmt::Write;
        let mut compare = DisplayAgainst {
            expected,
            at: 0,
            failed: false,
        };
        write!(&mut compare, "{value}").is_ok() && !compare.failed && compare.at == expected.len()
    }
    pub struct Against<'a>(&'a Value);
    impl Against<'_> {
        fn scalar(self, value: Value) -> Result<()> {
            require(*self.0 == value)
        }
    }
    fn variant<'a>(expected: &'a Value, name: &str) -> Result<&'a Value> {
        let map = expected.as_object().ok_or(Different)?;
        require(map.len() == 1)?;
        map.get(name).ok_or(Different)
    }
    pub struct Sequence<'a> {
        expected: &'a [Value],
        at: usize,
        failed: bool,
    }
    impl<'a> Sequence<'a> {
        fn new(expected: &'a Value) -> Result<Self> {
            Ok(Self {
                expected: expected.as_array().ok_or(Different)?,
                at: 0,
                failed: false,
            })
        }
        fn item<T: ?Sized + Serialize>(&mut self, item: &T) -> Result<()> {
            if self.failed {
                return Err(Different);
            }
            let result = self
                .expected
                .get(self.at)
                .ok_or(Different)
                .and_then(|expected| item.serialize(Against(expected)));
            if result.is_ok() {
                self.at += 1;
            } else {
                self.failed = true;
            }
            result
        }
        fn finish(self) -> Result<()> {
            require(!self.failed && self.at == self.expected.len())
        }
    }
    // Derived Scenario/model serializers have unique keys. Also reject duplicate
    // custom keys explicitly instead of accepting last-write-wins comparisons.
    // Most source objects fit64 inline slots; only a larger supplied map spills.
    pub struct Object<'a> {
        expected: &'a Map<String, Value>,
        seen: [Option<&'a str>; 64],
        extra: Vec<&'a str>,
        count: usize,
        pending: Option<&'a str>,
        failed: bool,
    }
    impl<'a> Object<'a> {
        fn new(expected: &'a Value) -> Result<Self> {
            Ok(Self {
                expected: expected.as_object().ok_or(Different)?,
                seen: [None; 64],
                extra: Vec::new(),
                count: 0,
                pending: None,
                failed: false,
            })
        }
        fn key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
            if self.failed {
                return Err(Different);
            }
            let result = (|| {
                require(self.pending.is_none())?;
                let key = key.serialize(Key(self.expected))?;
                require(
                    !self.seen[..self.count.min(self.seen.len())]
                        .iter()
                        .flatten()
                        .chain(self.extra.iter())
                        .any(|seen| *seen == key),
                )?;
                if self.count < self.seen.len() {
                    self.seen[self.count] = Some(key);
                } else {
                    self.extra.push(key);
                }
                self.count += 1;
                self.pending = Some(key);
                Ok(())
            })();
            if result.is_err() {
                self.failed = true;
            }
            result
        }
        fn value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
            if self.failed {
                return Err(Different);
            }
            let result =
                self.pending.take().ok_or(Different).and_then(|key| {
                    value.serialize(Against(self.expected.get(key).ok_or(Different)?))
                });
            if result.is_err() {
                self.failed = true;
            }
            result
        }
        fn field<T: ?Sized + Serialize>(&mut self, key: &str, value: &T) -> Result<()> {
            self.key(key)?;
            self.value(value)
        }
        fn finish(self) -> Result<()> {
            require(!self.failed && self.pending.is_none() && self.count == self.expected.len())
        }
    }
    struct Key<'a>(&'a Map<String, Value>);
    impl<'a> Key<'a> {
        fn named(self, key: &str) -> Result<&'a str> {
            self.0
                .get_key_value(key)
                .map(|(k, _)| k.as_str())
                .ok_or(Different)
        }
        fn display<T: ?Sized + fmt::Display>(self, key: &T) -> Result<&'a str> {
            // Formatted map keys are normally tiny numeric/IP values. Format
            // once (as serde_json does), never replay a stateful Display.
            self.named(&key.to_string())
        }
    }
    macro_rules! signed { ($($name:ident($ty:ty)),+) => { $(fn $name(self, v: $ty) -> Result<()> { self.scalar(Value::from(v)) })+ }; }
    macro_rules! key_integer { ($($name:ident($ty:ty)),+) => { $(fn $name(self, v: $ty) -> Result<Self::Ok> { self.display(&v) })+ }; }
    impl<'a> Serializer for Against<'a> {
        type Ok = ();
        type Error = Different;
        type SerializeSeq = Sequence<'a>;
        type SerializeTuple = Sequence<'a>;
        type SerializeTupleStruct = Sequence<'a>;
        type SerializeTupleVariant = Sequence<'a>;
        type SerializeMap = Object<'a>;
        type SerializeStruct = Object<'a>;
        type SerializeStructVariant = Object<'a>;
        fn serialize_bool(self, v: bool) -> Result<()> {
            self.scalar(Value::Bool(v))
        }
        signed!(
            serialize_i8(i8),
            serialize_i16(i16),
            serialize_i32(i32),
            serialize_i64(i64),
            serialize_u8(u8),
            serialize_u16(u16),
            serialize_u32(u32),
            serialize_u64(u64),
            serialize_f32(f32),
            serialize_f64(f64)
        );
        fn serialize_i128(self, v: i128) -> Result<()> {
            if let Ok(v) = u64::try_from(v) {
                self.serialize_u64(v)
            } else {
                self.serialize_i64(i64::try_from(v).map_err(|_| Different)?)
            }
        }
        fn serialize_u128(self, v: u128) -> Result<()> {
            self.serialize_u64(u64::try_from(v).map_err(|_| Different)?)
        }
        fn serialize_char(self, v: char) -> Result<()> {
            self.serialize_str(v.encode_utf8(&mut [0; 4]))
        }
        fn serialize_str(self, v: &str) -> Result<()> {
            require(self.0.as_str() == Some(v))
        }
        fn serialize_bytes(self, v: &[u8]) -> Result<()> {
            let expected = self.0.as_array().ok_or(Different)?;
            require(expected.len() == v.len())?;
            for (v, expected) in v.iter().zip(expected) {
                Against(expected).serialize_u8(*v)?;
            }
            Ok(())
        }
        fn serialize_none(self) -> Result<()> {
            require(self.0.is_null())
        }
        fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<()> {
            v.serialize(self)
        }
        fn serialize_unit(self) -> Result<()> {
            self.serialize_none()
        }
        fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
            self.serialize_none()
        }
        fn serialize_unit_variant(self, _: &'static str, _: u32, v: &'static str) -> Result<()> {
            self.serialize_str(v)
        }
        fn serialize_newtype_struct<T: ?Sized + Serialize>(
            self,
            _: &'static str,
            v: &T,
        ) -> Result<()> {
            v.serialize(self)
        }
        fn serialize_newtype_variant<T: ?Sized + Serialize>(
            self,
            _: &'static str,
            _: u32,
            name: &'static str,
            v: &T,
        ) -> Result<()> {
            v.serialize(Against(variant(self.0, name)?))
        }
        fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq> {
            Sequence::new(self.0)
        }
        fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple> {
            Sequence::new(self.0)
        }
        fn serialize_tuple_struct(
            self,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeTupleStruct> {
            Sequence::new(self.0)
        }
        fn serialize_tuple_variant(
            self,
            _: &'static str,
            _: u32,
            name: &'static str,
            _: usize,
        ) -> Result<Self::SerializeTupleVariant> {
            Sequence::new(variant(self.0, name)?)
        }
        fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap> {
            Object::new(self.0)
        }
        fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct> {
            Object::new(self.0)
        }
        fn serialize_struct_variant(
            self,
            _: &'static str,
            _: u32,
            name: &'static str,
            _: usize,
        ) -> Result<Self::SerializeStructVariant> {
            Object::new(variant(self.0, name)?)
        }
        fn collect_str<T: ?Sized + fmt::Display>(self, value: &T) -> Result<()> {
            require(self.0.as_str().is_some_and(|s| same_display(value, s)))
        }
    }
    impl<'a> Serializer for Key<'a> {
        type Ok = &'a str;
        type Error = Different;
        type SerializeSeq = ser::Impossible<Self::Ok, Different>;
        type SerializeTuple = ser::Impossible<Self::Ok, Different>;
        type SerializeTupleStruct = ser::Impossible<Self::Ok, Different>;
        type SerializeTupleVariant = ser::Impossible<Self::Ok, Different>;
        type SerializeMap = ser::Impossible<Self::Ok, Different>;
        type SerializeStruct = ser::Impossible<Self::Ok, Different>;
        type SerializeStructVariant = ser::Impossible<Self::Ok, Different>;
        key_integer!(
            serialize_i8(i8),
            serialize_i16(i16),
            serialize_i32(i32),
            serialize_i64(i64),
            serialize_i128(i128),
            serialize_u8(u8),
            serialize_u16(u16),
            serialize_u32(u32),
            serialize_u64(u64),
            serialize_u128(u128)
        );
        fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
            self.named(if v { "true" } else { "false" })
        }
        fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
            if !v.is_finite() {
                return Err(Different);
            }
            self.named(&serde_json::to_string(&v).map_err(|_| Different)?)
        }
        fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
            if !v.is_finite() {
                return Err(Different);
            }
            self.named(&serde_json::to_string(&v).map_err(|_| Different)?)
        }
        fn serialize_char(self, v: char) -> Result<Self::Ok> {
            self.named(v.encode_utf8(&mut [0; 4]))
        }
        fn serialize_str(self, v: &str) -> Result<Self::Ok> {
            self.named(v)
        }
        fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok> {
            Err(Different)
        }
        fn serialize_none(self) -> Result<Self::Ok> {
            Err(Different)
        }
        fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<Self::Ok> {
            Err(Different)
        }
        fn serialize_unit(self) -> Result<Self::Ok> {
            Err(Different)
        }
        fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok> {
            Err(Different)
        }
        fn serialize_unit_variant(
            self,
            _: &'static str,
            _: u32,
            v: &'static str,
        ) -> Result<Self::Ok> {
            self.named(v)
        }
        fn serialize_newtype_struct<T: ?Sized + Serialize>(
            self,
            _: &'static str,
            v: &T,
        ) -> Result<Self::Ok> {
            v.serialize(self)
        }
        fn serialize_newtype_variant<T: ?Sized + Serialize>(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: &T,
        ) -> Result<Self::Ok> {
            Err(Different)
        }
        fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq> {
            Err(Different)
        }
        fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple> {
            Err(Different)
        }
        fn serialize_tuple_struct(
            self,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeTupleStruct> {
            Err(Different)
        }
        fn serialize_tuple_variant(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeTupleVariant> {
            Err(Different)
        }
        fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap> {
            Err(Different)
        }
        fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct> {
            Err(Different)
        }
        fn serialize_struct_variant(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeStructVariant> {
            Err(Different)
        }
        fn collect_str<T: ?Sized + fmt::Display>(self, v: &T) -> Result<Self::Ok> {
            self.display(v)
        }
    }
    macro_rules! sequence_impl {
        ($trait:ident,$method:ident) => {
            impl ser::$trait for Sequence<'_> {
                type Ok = ();
                type Error = Different;
                fn $method<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
                    self.item(v)
                }
                fn end(self) -> Result<()> {
                    self.finish()
                }
            }
        };
    }
    sequence_impl!(SerializeSeq, serialize_element);
    sequence_impl!(SerializeTuple, serialize_element);
    sequence_impl!(SerializeTupleStruct, serialize_field);
    sequence_impl!(SerializeTupleVariant, serialize_field);
    impl ser::SerializeMap for Object<'_> {
        type Ok = ();
        type Error = Different;
        fn serialize_key<T: ?Sized + Serialize>(&mut self, k: &T) -> Result<()> {
            self.key(k)
        }
        fn serialize_value<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
            self.value(v)
        }
        fn end(self) -> Result<()> {
            self.finish()
        }
    }
    impl ser::SerializeStruct for Object<'_> {
        type Ok = ();
        type Error = Different;
        fn serialize_field<T: ?Sized + Serialize>(&mut self, k: &'static str, v: &T) -> Result<()> {
            self.field(k, v)
        }
        fn end(self) -> Result<()> {
            self.finish()
        }
    }
    impl ser::SerializeStructVariant for Object<'_> {
        type Ok = ();
        type Error = Different;
        fn serialize_field<T: ?Sized + Serialize>(&mut self, k: &'static str, v: &T) -> Result<()> {
            self.field(k, v)
        }
        fn end(self) -> Result<()> {
            self.finish()
        }
    }
}
/// Full serialized-value equality without allocating the actual container/string
/// tree. Public only within this test/developer support module for parity tests.
pub fn serialized_equal<T: ?Sized + serde::Serialize>(actual: &T, expected: &Value) -> bool {
    borrowed_json::equal(actual, expected)
}
