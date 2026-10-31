//! Exact201→199/54→53 developer predecessor. Never learner grading or mastery.
#![allow(dead_code)] // Each test/native binary imports different closed subsets.
use dlms_learning_lab::developer_digest as digest;
use dlms_learning_lab::{
    course::Unit,
    engine::State,
    pack::{Pack, Scenario},
    progress::Attempt,
};
use serde_json::Value;
use std::{collections::BTreeMap, sync::OnceLock};
pub const IDS: [&str; 2] = ["shell-snapshot-audit", "shell-conditional-rollout"];
fn recipe() -> &'static Value {
    static R: OnceLock<Value> = OnceLock::new();
    R.get_or_init(|| {
        let raw = include_bytes!("../fixtures/catalog201-registration.json");
        assert_eq!(
            digest::hash(raw),
            "e59b6f5ba29d84d886b322c5f1297047d4e9b6bf744a21fcc1f82c78f51244ae"
        );
        serde_json::from_slice(raw).unwrap()
    })
}
pub fn original199() -> Vec<Scenario> {
    let mut old = super::original197();
    old.extend(super::new_sources().iter().cloned());
    assert_eq!(old.len(), 199);
    old
}

fn old_values() -> &'static Vec<Value> {
    static V: OnceLock<Vec<Value>> = OnceLock::new();
    V.get_or_init(|| {
        original199()
            .iter()
            .map(|s| serde_json::to_value(s).unwrap())
            .collect()
    })
}
fn new_sources() -> &'static Vec<Scenario> {
    static S: OnceLock<Vec<Scenario>> = OnceLock::new();
    S.get_or_init(|| {
        let raw = include_bytes!("../../scenarios/shell-snapshot-rollout-draft.json");
        assert_eq!(
            raw.len() as u64,
            recipe()["canonical_snapshot_pack_bytes"].as_u64().unwrap()
        );
        assert_eq!(
            digest::hash(raw),
            recipe()["canonical_snapshot_pack_sha256"].as_str().unwrap()
        );
        let p: Pack = serde_json::from_slice(raw).unwrap();
        assert_eq!(
            p.scenarios
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            IDS
        );
        p.scenarios
    })
}
fn new_values() -> &'static Vec<Value> {
    static V: OnceLock<Vec<Value>> = OnceLock::new();
    V.get_or_init(|| {
        new_sources()
            .iter()
            .map(|s| serde_json::to_value(s).unwrap())
            .collect()
    })
}
pub fn before_scenarios(all: &[Scenario]) -> &[Scenario] {
    assert_eq!(all.len(), 201);
    for (actual, expected) in all.iter().zip(source_tapes()) {
        assert!(
            event_serialized_equal(actual, expected),
            "Complete source differs: {}",
            actual.id
        );
    }
    &all[..199]
}
pub fn prefix_scenarios(all: &[Scenario]) -> Option<&[Scenario]> {
    (all.len() == 201).then(|| before_scenarios(all))
}
pub fn before_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    let r = recipe();
    if raw.len() as u64 != r["after_units54_bytes"].as_u64().unwrap()
        || digest::hash(raw) != r["after_units54_sha256"].as_str().unwrap()
    {
        return Err("Exact53 raw unit bytes differ".into());
    }
    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
    let head = text
        .strip_suffix(r["append_suffix_utf8"].as_str().unwrap())
        .ok_or("Exact snapshot unit append differs")?;
    let old = format!("{head}{}", r["old_tail_utf8"].as_str().unwrap()).into_bytes();
    if old != include_bytes!("../fixtures/snapshot-units-before199.json")
        || old.len() as u64 != r["before_units53_bytes"].as_u64().unwrap()
        || digest::hash(&old) != r["before_units53_sha256"].as_str().unwrap()
    {
        return Err("Complete previous53 raw bytes differ".into());
    }
    let mut expected = r["before_units53"].clone();
    expected["units"]
        .as_array_mut()
        .unwrap()
        .push(r["snapshot_unit"].clone());
    if serde_json::from_slice::<Value>(raw).map_err(|e| e.to_string())? != expected {
        return Err("Change outside exact appended unit".into());
    }
    Ok(old)
}
pub fn prefix_units(raw: &[u8]) -> Result<Option<Vec<u8>>, String> {
    if raw.len() as u64 == recipe()["after_units54_bytes"].as_u64().unwrap()
        && digest::hash(raw) == recipe()["after_units54_sha256"].as_str().unwrap()
    {
        before_units(raw).map(Some)
    } else {
        Ok(None)
    }
}
pub fn before_typed_units(all: &[Unit]) -> &[Unit] {
    assert_eq!(all.len(), 54);
    for (actual, expected) in all.iter().zip(unit_tapes()) {
        assert!(
            event_serialized_equal(actual, expected),
            "Complete unit differs: {}",
            actual.id
        );
    }
    &all[..53]
}
pub fn prefix_typed_units(all: &[Unit]) -> Option<&[Unit]> {
    (all.len() == 54).then(|| before_typed_units(all))
}
pub fn before_values(all: &[Value]) -> &[Value] {
    assert_eq!(all.len(), 54);
    assert_eq!(
        &all[..53],
        recipe()["before_units53"]["units"].as_array().unwrap()
    );
    assert_eq!(&all[53], &recipe()["snapshot_unit"]);
    &all[..53]
}
pub fn prefix_values(all: &[Value]) -> Option<&[Value]> {
    (all.len() == 54).then(|| before_values(all))
}
/// Count only; validates every key/version and both bounded Script contexts.
pub fn before_attempts_len(all: &BTreeMap<String, Attempt>) -> usize {
    assert_eq!(all.len(), 201);
    for v in old_values().iter().chain(new_values()) {
        assert_eq!(
            all[v["id"].as_str().unwrap()].scenario_version as u64,
            v["version"].as_u64().unwrap()
        );
    }
    for s in new_sources() {
        let a = &all[&s.id];
        assert!(matches!(a.state, State::Script { .. }));
        assert!(a.ipv4.is_none());
        assert!(a.step < s.steps.len() && a.hints <= s.hints.len());
        dlms_learning_lab::pack::validate_state(&a.state).unwrap();
        assert!(a.workspace.as_ref().is_none_or(|w| w.valid()));
    }
    199
}

// Compare the complete current serialization, not a digest or a remembered
// actual object. Expected bytes come only from independent fixed typed sources.
// This is a test/developer source guard, never learner grading.
struct CompareBytes<'a> {
    expected: &'a [u8],
    at: usize,
    failed: bool,
}
impl std::io::Write for CompareBytes<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let end = self.at.checked_add(bytes.len());
        if self.failed || end.is_none() || self.expected.get(self.at..end.unwrap()) != Some(bytes) {
            self.failed = true;
            return Err(std::io::Error::other("Complete serialized source differs"));
        }
        self.at = end.unwrap();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        if self.failed {
            Err(std::io::Error::other("Complete serialized source differs"))
        } else {
            Ok(())
        }
    }
}
/// Exact compact serde_json byte equality. Only the immutable expected bytes
/// are cached. Callers use identical concrete typed schemas and serializers;
/// this does not claim order-independent equality of arbitrary custom maps.
pub fn canonical_serialized_equal<T: ?Sized + serde::Serialize>(
    actual: &T,
    expected: &[u8],
) -> bool {
    let mut compare = CompareBytes {
        expected,
        at: 0,
        failed: false,
    };
    serde_json::to_writer(&mut compare, actual).is_ok()
        && !compare.failed
        && compare.at == expected.len()
}
fn source_bytes() -> &'static Vec<Vec<u8>> {
    static BYTES: OnceLock<Vec<Vec<u8>>> = OnceLock::new();
    BYTES.get_or_init(|| {
        original199()
            .iter()
            .chain(new_sources())
            .map(|s| serde_json::to_vec(s).unwrap())
            .collect()
    })
}
fn unit_bytes() -> &'static Vec<Vec<u8>> {
    static BYTES: OnceLock<Vec<Vec<u8>>> = OnceLock::new();
    BYTES.get_or_init(|| {
        recipe()["before_units53"]["units"]
            .as_array()
            .unwrap()
            .iter()
            .chain(std::iter::once(&recipe()["snapshot_unit"]))
            .map(|v| {
                // Serialize the independent expected concrete Unit, not a
                // Value map: struct declaration order is part of this format.
                let unit: Unit = serde_json::from_value(v.clone()).unwrap();
                serde_json::to_vec(&unit).unwrap()
            })
            .collect()
    })
}

// Expected events come only from independent fixed typed sources. Actual
// serialization is traversed completely on every call, without JSON escaping,
// a Value-map lookup, a hash, or an actual-object identity/cache.
mod source_events {
    use serde::{Serialize, Serializer, ser};
    use std::fmt;

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Atom {
        Bool(bool),
        Signed(i128),
        Unsigned(u128),
        F32(u32),
        F64(u64),
        Char(char),
        None,
        Some,
        Unit,
        UnitStruct,
        UnitVariant(u32),
        NewtypeStruct,
        NewtypeVariant(u32),
        Seq(Option<usize>),
        Tuple(usize),
        TupleStruct(usize),
        TupleVariant(u32, usize),
        Map(Option<usize>),
        Struct(usize),
        StructVariant(u32, usize),
        Element,
        Key,
        Value,
        Field,
        EndSome,
        EndNewtype,
        EndSeq,
        EndTuple,
        EndTupleStruct,
        EndTupleVariant,
        EndMap,
        EndStruct,
        EndStructVariant,
    }
    #[derive(Debug)]
    enum Token {
        Atom(Atom),
        Text(String),
        Bytes(Vec<u8>),
    }
    /// Immutable expected event sequence. No actual source is retained here.
    #[derive(Debug)]
    pub struct Tape(Vec<Token>);
    #[derive(Debug)]
    struct Different;
    impl fmt::Display for Different {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("Complete typed serialization event differs")
        }
    }
    impl std::error::Error for Different {}
    impl ser::Error for Different {
        fn custom<T: fmt::Display>(_: T) -> Self {
            Self
        }
    }
    type Result<T> = std::result::Result<T, Different>;
    struct Sink<'a> {
        expected: &'a [Token],
        recorded: Option<Vec<Token>>,
        at: usize,
        failed: bool,
    }
    impl Sink<'_> {
        fn require(&mut self, same: bool) -> Result<()> {
            if self.failed || !same {
                self.failed = true;
                Err(Different)
            } else {
                self.at += 1;
                Ok(())
            }
        }
        fn atom(&mut self, v: Atom) -> Result<()> {
            if let Some(recorded) = &mut self.recorded {
                recorded.push(Token::Atom(v));
                return Ok(());
            }
            self.require(matches!(self.expected.get(self.at), Some(Token::Atom(e)) if *e == v))
        }
        fn text(&mut self, v: &str) -> Result<()> {
            if let Some(recorded) = &mut self.recorded {
                recorded.push(Token::Text(v.to_owned()));
                return Ok(());
            }
            self.require(matches!(self.expected.get(self.at), Some(Token::Text(e)) if e == v))
        }
        fn bytes(&mut self, v: &[u8]) -> Result<()> {
            if let Some(recorded) = &mut self.recorded {
                recorded.push(Token::Bytes(v.to_vec()));
                return Ok(());
            }
            self.require(matches!(self.expected.get(self.at), Some(Token::Bytes(e)) if e == v))
        }
        fn child<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
            if self.failed {
                return Err(Different);
            }
            let result = value.serialize(&mut *self);
            if result.is_err() {
                self.failed = true;
            }
            result
        }
        fn display<T: ?Sized + fmt::Display>(&mut self, value: &T) -> Result<()> {
            if self.recorded.is_some() {
                return self.text(&value.to_string());
            }
            // Compare Display chunks without allocating a current string.
            let same = match self.expected.get(self.at) {
                Some(Token::Text(expected)) => {
                    use fmt::Write;
                    let mut compare = DisplayText {
                        expected,
                        at: 0,
                        failed: false,
                    };
                    write!(&mut compare, "{value}").is_ok()
                        && !compare.failed
                        && compare.at == expected.len()
                }
                _ => false,
            };
            self.require(same)
        }
    }
    struct DisplayText<'a> {
        expected: &'a str,
        at: usize,
        failed: bool,
    }
    impl fmt::Write for DisplayText<'_> {
        fn write_str(&mut self, text: &str) -> fmt::Result {
            let end = self.at.checked_add(text.len());
            if self.failed
                || end.is_none()
                || self.expected.as_bytes().get(self.at..end.unwrap()) != Some(text.as_bytes())
            {
                self.failed = true;
                return Err(fmt::Error);
            }
            self.at = end.unwrap();
            Ok(())
        }
    }
    pub fn record<T: ?Sized + Serialize>(expected: &T) -> Tape {
        let mut sink = Sink {
            expected: &[],
            recorded: Some(Vec::new()),
            at: 0,
            failed: false,
        };
        sink.child(expected)
            .expect("Independent typed source serialization failed");
        assert!(!sink.failed);
        Tape(sink.recorded.unwrap())
    }
    pub fn equal<T: ?Sized + Serialize>(actual: &T, tape: &Tape) -> bool {
        let mut sink = Sink {
            expected: &tape.0,
            recorded: None,
            at: 0,
            failed: false,
        };
        sink.child(actual).is_ok() && !sink.failed && sink.at == tape.0.len()
    }
    struct Compound<'a, 'b> {
        sink: &'a mut Sink<'b>,
        end: Atom,
    }
    impl Compound<'_, '_> {
        fn item<T: ?Sized + Serialize>(&mut self, tag: Atom, value: &T) -> Result<()> {
            self.sink.atom(tag)?;
            self.sink.child(value)
        }
        fn field<T: ?Sized + Serialize>(&mut self, name: &'static str, value: &T) -> Result<()> {
            self.sink.atom(Atom::Field)?;
            self.sink.text(name)?;
            self.sink.child(value)
        }
        fn finish(self) -> Result<()> {
            self.sink.atom(self.end)
        }
    }
    macro_rules! signed { ($($fn:ident($ty:ty)),+) => { $(
        fn $fn(self, v: $ty) -> Result<()> { self.atom(Atom::Signed(i128::from(v))) }
    )+ }; }
    macro_rules! unsigned { ($($fn:ident($ty:ty)),+) => { $(
        fn $fn(self, v: $ty) -> Result<()> { self.atom(Atom::Unsigned(u128::from(v))) }
    )+ }; }
    impl<'a, 'b> Serializer for &'a mut Sink<'b> {
        type Ok = ();
        type Error = Different;
        type SerializeSeq = Compound<'a, 'b>;
        type SerializeTuple = Compound<'a, 'b>;
        type SerializeTupleStruct = Compound<'a, 'b>;
        type SerializeTupleVariant = Compound<'a, 'b>;
        type SerializeMap = Compound<'a, 'b>;
        type SerializeStruct = Compound<'a, 'b>;
        type SerializeStructVariant = Compound<'a, 'b>;
        signed!(
            serialize_i8(i8),
            serialize_i16(i16),
            serialize_i32(i32),
            serialize_i64(i64),
            serialize_i128(i128)
        );
        unsigned!(
            serialize_u8(u8),
            serialize_u16(u16),
            serialize_u32(u32),
            serialize_u64(u64),
            serialize_u128(u128)
        );
        fn serialize_bool(self, v: bool) -> Result<()> {
            self.atom(Atom::Bool(v))
        }
        fn serialize_f32(self, v: f32) -> Result<()> {
            self.atom(Atom::F32(v.to_bits()))
        }
        fn serialize_f64(self, v: f64) -> Result<()> {
            self.atom(Atom::F64(v.to_bits()))
        }
        fn serialize_char(self, v: char) -> Result<()> {
            self.atom(Atom::Char(v))
        }
        fn serialize_str(self, v: &str) -> Result<()> {
            self.text(v)
        }
        fn serialize_bytes(self, v: &[u8]) -> Result<()> {
            self.bytes(v)
        }
        fn serialize_none(self) -> Result<()> {
            self.atom(Atom::None)
        }
        fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<()> {
            self.atom(Atom::Some)?;
            self.child(v)?;
            self.atom(Atom::EndSome)
        }
        fn serialize_unit(self) -> Result<()> {
            self.atom(Atom::Unit)
        }
        fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
            self.atom(Atom::UnitStruct)?;
            self.text(name)
        }
        fn serialize_unit_variant(
            self,
            name: &'static str,
            index: u32,
            variant: &'static str,
        ) -> Result<()> {
            self.atom(Atom::UnitVariant(index))?;
            self.text(name)?;
            self.text(variant)
        }
        fn serialize_newtype_struct<T: ?Sized + Serialize>(
            self,
            name: &'static str,
            v: &T,
        ) -> Result<()> {
            self.atom(Atom::NewtypeStruct)?;
            self.text(name)?;
            self.child(v)?;
            self.atom(Atom::EndNewtype)
        }
        fn serialize_newtype_variant<T: ?Sized + Serialize>(
            self,
            name: &'static str,
            index: u32,
            variant: &'static str,
            v: &T,
        ) -> Result<()> {
            self.atom(Atom::NewtypeVariant(index))?;
            self.text(name)?;
            self.text(variant)?;
            self.child(v)?;
            self.atom(Atom::EndNewtype)
        }
        fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
            self.atom(Atom::Seq(len))?;
            Ok(Compound {
                sink: self,
                end: Atom::EndSeq,
            })
        }
        fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
            self.atom(Atom::Tuple(len))?;
            Ok(Compound {
                sink: self,
                end: Atom::EndTuple,
            })
        }
        fn serialize_tuple_struct(
            self,
            name: &'static str,
            len: usize,
        ) -> Result<Self::SerializeTupleStruct> {
            self.atom(Atom::TupleStruct(len))?;
            self.text(name)?;
            Ok(Compound {
                sink: self,
                end: Atom::EndTupleStruct,
            })
        }
        fn serialize_tuple_variant(
            self,
            name: &'static str,
            index: u32,
            variant: &'static str,
            len: usize,
        ) -> Result<Self::SerializeTupleVariant> {
            self.atom(Atom::TupleVariant(index, len))?;
            self.text(name)?;
            self.text(variant)?;
            Ok(Compound {
                sink: self,
                end: Atom::EndTupleVariant,
            })
        }
        fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
            self.atom(Atom::Map(len))?;
            Ok(Compound {
                sink: self,
                end: Atom::EndMap,
            })
        }
        fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
            self.atom(Atom::Struct(len))?;
            self.text(name)?;
            Ok(Compound {
                sink: self,
                end: Atom::EndStruct,
            })
        }
        fn serialize_struct_variant(
            self,
            name: &'static str,
            index: u32,
            variant: &'static str,
            len: usize,
        ) -> Result<Self::SerializeStructVariant> {
            self.atom(Atom::StructVariant(index, len))?;
            self.text(name)?;
            self.text(variant)?;
            Ok(Compound {
                sink: self,
                end: Atom::EndStructVariant,
            })
        }
        fn collect_str<T: ?Sized + fmt::Display>(self, v: &T) -> Result<()> {
            self.display(v)
        }
    }
    impl ser::SerializeSeq for Compound<'_, '_> {
        type Ok = ();
        type Error = Different;
        fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
            self.item(Atom::Element, v)
        }
        fn end(self) -> Result<()> {
            self.finish()
        }
    }
    impl ser::SerializeTuple for Compound<'_, '_> {
        type Ok = ();
        type Error = Different;
        fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
            self.item(Atom::Element, v)
        }
        fn end(self) -> Result<()> {
            self.finish()
        }
    }
    impl ser::SerializeTupleStruct for Compound<'_, '_> {
        type Ok = ();
        type Error = Different;
        fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
            self.item(Atom::Element, v)
        }
        fn end(self) -> Result<()> {
            self.finish()
        }
    }
    impl ser::SerializeTupleVariant for Compound<'_, '_> {
        type Ok = ();
        type Error = Different;
        fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
            self.item(Atom::Element, v)
        }
        fn end(self) -> Result<()> {
            self.finish()
        }
    }
    impl ser::SerializeMap for Compound<'_, '_> {
        type Ok = ();
        type Error = Different;
        fn serialize_key<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
            self.item(Atom::Key, v)
        }
        fn serialize_value<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
            self.item(Atom::Value, v)
        }
        fn end(self) -> Result<()> {
            self.finish()
        }
    }
    impl ser::SerializeStruct for Compound<'_, '_> {
        type Ok = ();
        type Error = Different;
        fn serialize_field<T: ?Sized + Serialize>(
            &mut self,
            name: &'static str,
            v: &T,
        ) -> Result<()> {
            self.field(name, v)
        }
        fn end(self) -> Result<()> {
            self.finish()
        }
    }
    impl ser::SerializeStructVariant for Compound<'_, '_> {
        type Ok = ();
        type Error = Different;
        fn serialize_field<T: ?Sized + Serialize>(
            &mut self,
            name: &'static str,
            v: &T,
        ) -> Result<()> {
            self.field(name, v)
        }
        fn end(self) -> Result<()> {
            self.finish()
        }
    }
}
pub use source_events::{
    Tape as SourceTape, equal as event_serialized_equal, record as expected_source_tape,
};
fn source_tapes() -> &'static Vec<SourceTape> {
    static TAPES: OnceLock<Vec<SourceTape>> = OnceLock::new();
    TAPES.get_or_init(|| {
        original199()
            .iter()
            .chain(new_sources())
            .map(expected_source_tape)
            .collect()
    })
}
fn unit_tapes() -> &'static Vec<SourceTape> {
    static TAPES: OnceLock<Vec<SourceTape>> = OnceLock::new();
    TAPES.get_or_init(|| {
        recipe()["before_units53"]["units"]
            .as_array()
            .unwrap()
            .iter()
            .chain(std::iter::once(&recipe()["snapshot_unit"]))
            .map(|v| expected_source_tape(&serde_json::from_value::<Unit>(v.clone()).unwrap()))
            .collect()
    })
}
