//! Native template drafts and author-only typed reference steps. No CLI/host dispatch.
use crate::{
    accounts::{self, Action, Kind, State},
    accounts_teaching,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const MAX_LEAVES: usize = 4;
pub const MAX_TEXT_BYTES: usize = 8192;
pub const MAX_JSON_BYTES: usize =
    accounts::MAX_STATE_BYTES + MAX_LEAVES * (MAX_TEXT_BYTES * 6 + 2048) + 8192;
fn text<'de, D: serde::Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    struct Text<const N: usize>;
    impl<const N: usize> serde::de::Visitor<'_> for Text<N> {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {N} UTF-8 bytes")
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<String, E> {
            if v.len() > N {
                Err(E::custom("Template text limit"))
            } else {
                Ok(v.into())
            }
        }
    }
    d.deserialize_str(Text::<N>)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Step {
    Save {
        #[serde(deserialize_with = "text::<_,128>")]
        path: String,
        #[serde(deserialize_with = "text::<_,MAX_TEXT_BYTES>")]
        contents: String,
        mode: u16,
    },
    Remove {
        #[serde(deserialize_with = "text::<_,128>")]
        path: String,
    },
}
impl Step {
    fn bounded(&self) -> bool {
        match self {
            Self::Save {
                path,
                contents,
                mode,
            } => {
                path.len() <= 128
                    && contents.len() <= MAX_TEXT_BYTES
                    && !contents.contains('\0')
                    && *mode <= 0o777
            }
            Self::Remove { path } => path.len() <= 128,
        }
    }
    fn action(&self) -> Action {
        match self {
            Self::Save {
                path,
                contents,
                mode,
            } => Action::SaveSkeletonLeaf {
                path: path.clone(),
                contents: contents.clone(),
                mode: *mode,
            },
            Self::Remove { path } => Action::RemoveSkeletonLeaf { path: path.clone() },
        }
    }
    pub fn path(&self) -> &str {
        match self {
            Self::Save { path, .. } | Self::Remove { path } => path,
        }
    }
    fn valid_for(&self, source: &State) -> bool {
        self.bounded()
            && source
                .policy()
                .editable_skeleton_leaves
                .get(self.path())
                .is_some_and(|leaf| !matches!(self, Self::Remove { .. }) || leaf.allow_remove)
    }
}
fn steps<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Step>, D::Error> {
    use serde::de::{Error as _, SeqAccess, Visitor};
    struct Steps;
    impl<'de> Visitor<'de> for Steps {
        type Value = Vec<Step>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("at most4 native template steps")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while out.len() < MAX_LEAVES {
                match a.next_element()? {
                    None => return Ok(out),
                    Some(v) => out.push(v),
                }
            }
            if a.next_element::<serde::de::IgnoredAny>()?.is_some() {
                Err(A::Error::custom("Template step count limit"))
            } else {
                Ok(out)
            }
        }
    }
    d.deserialize_seq(Steps)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Teaching {
    #[serde(default, deserialize_with = "steps")]
    pub assignment: Vec<Step>,
    #[serde(default, deserialize_with = "steps")]
    pub example: Vec<Step>,
    #[serde(default, deserialize_with = "steps")]
    pub try_steps: Vec<Step>,
    #[serde(default, deserialize_with = "steps")]
    pub repair: Vec<Step>,
}
impl Teaching {
    pub fn valid_shape(&self) -> bool {
        [
            &self.assignment,
            &self.example,
            &self.try_steps,
            &self.repair,
        ]
        .iter()
        .all(|v| v.len() <= MAX_LEAVES && v.iter().all(Step::bounded))
            && [
                &self.assignment,
                &self.example,
                &self.try_steps,
                &self.repair,
            ]
            .iter()
            .any(|v| !v.is_empty())
    }
    pub fn validate_example(&self, source: &State) -> Result<(), String> {
        if !self.valid_shape()
            || !source_valid(source)
            || [&self.example, &self.try_steps, &self.repair]
                .iter()
                .any(|v| v.iter().any(|s| !s.valid_for(source)))
        {
            return Err("Native example steps differ from their known template source".into());
        }
        Ok(())
    }
    pub fn validate_assignment(&self, source: &State) -> Result<(), String> {
        if !self.valid_shape()
            || !source_valid(source)
            || self.assignment.iter().any(|s| !s.valid_for(source))
        {
            return Err(
                "Native assignment reference steps differ from their known template source".into(),
            );
        }
        Ok(())
    }
    /// Author/pack validation only. Never call when constructing learner work.
    pub fn assignment_reference(&self, source: &State) -> Result<State, String> {
        self.validate_assignment(source)?;
        let mut current = source.clone();
        apply_phase(source, &mut current, &self.assignment)?;
        Ok(current)
    }
}
/// Atomic author reference phase; ordinary terminal commands follow it separately.
pub fn apply_phase(source: &State, current: &mut State, steps: &[Step]) -> Result<(), String> {
    if !source.valid_initial()
        || !current.valid_against(source)
        || steps.len() > MAX_LEAVES
        || steps.iter().any(|s| !s.valid_for(source))
    {
        return Err("Native template phase/source is invalid; previous state retained".into());
    }
    let mut candidate = current.clone();
    for step in steps {
        accounts_teaching::apply_in_context(source, &mut candidate, &step.action())?;
    }
    *current = candidate;
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeafDraft {
    pub included: bool,
    #[serde(deserialize_with = "text::<_,MAX_TEXT_BYTES>")]
    pub contents: String,
    #[serde(deserialize_with = "text::<_,4>")]
    pub mode: String,
}
fn leaves<'de, D: serde::Deserializer<'de>>(d: D) -> Result<BTreeMap<String, LeafDraft>, D::Error> {
    use serde::de::{Error as _, MapAccess, Visitor};
    struct Key(String);
    impl<'de> Deserialize<'de> for Key {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            text::<D, 128>(d).map(Self)
        }
    }
    struct Leaves;
    impl<'de> Visitor<'de> for Leaves {
        type Value = BTreeMap<String, LeafDraft>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("at most4 distinct known template drafts")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(Key(key)) = a.next_key()? {
                if out.len() >= MAX_LEAVES || out.contains_key(&key) {
                    return Err(A::Error::custom("Duplicate template key or count limit"));
                }
                out.insert(key, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Leaves)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Drafts {
    pub version: u32,
    /// Exact retained source, never authority by itself; valid_for compares held source.
    pub source: State,
    #[serde(deserialize_with = "text::<_,128>")]
    pub selected: String,
    #[serde(deserialize_with = "leaves")]
    pub leaves: BTreeMap<String, LeafDraft>,
}
fn source_valid(source: &State) -> bool {
    source.valid_initial()
        && !source.policy().editable_skeleton_leaves.is_empty()
        && source.policy().editable_skeleton_leaves.len() <= MAX_LEAVES
        && source.policy().editable_skeleton_leaves.keys().all(|p| {
            source.configuration().entries.get(p).is_some_and(
                |e| matches!(&e.data,Kind::Text{contents}if contents.len()<=MAX_TEXT_BYTES),
            )
        })
}
fn mode(raw: &str) -> Option<u16> {
    if raw.is_empty() || raw.len() > 4 || !raw.bytes().all(|b| (b'0'..=b'7').contains(&b)) {
        None
    } else {
        u16::from_str_radix(raw, 8).ok().filter(|m| *m <= 0o777)
    }
}
impl Drafts {
    pub fn new(source: &State, current: &State) -> Result<Self, String> {
        if !source_valid(source) || !current.valid_against(source) {
            return Err("Template draft source/context is invalid or exceeds editor bounds".into());
        }
        let mut leaves = BTreeMap::new();
        for key in source.policy().editable_skeleton_leaves.keys() {
            let actual = current.configuration().entries.get(key);
            let e = actual.unwrap_or(&source.configuration().entries[key]);
            let Kind::Text { contents } = &e.data else {
                return Err("Known template is not ordinary text".into());
            };
            if contents.len() > MAX_TEXT_BYTES {
                return Err("Applied template exceeds the 8 KiB editor limit".into());
            }
            leaves.insert(
                key.clone(),
                LeafDraft {
                    included: actual.is_some(),
                    contents: contents.clone(),
                    mode: format!("{:04o}", e.mode),
                },
            );
        }
        let selected = leaves
            .keys()
            .next()
            .cloned()
            .ok_or("No editable templates")?;
        Ok(Self {
            version: 1,
            source: source.clone(),
            selected,
            leaves,
        })
    }
    /// Shape validity intentionally permits invalid octal/NUL drafts for correction.
    pub fn valid(&self) -> bool {
        self.version == 1
            && source_valid(&self.source)
            && self.selected.len() <= 128
            && self.leaves.contains_key(&self.selected)
            && self.leaves.len() == self.source.policy().editable_skeleton_leaves.len()
            && self
                .leaves
                .keys()
                .eq(self.source.policy().editable_skeleton_leaves.keys())
            && self
                .leaves
                .values()
                .all(|d| d.contents.len() <= MAX_TEXT_BYTES && d.mode.len() <= 4)
    }
    pub fn valid_for(&self, source: &State, current: &State) -> bool {
        self.valid() && &self.source == source && current.valid_against(source)
    }
    pub fn from_json(source: &State, current: &State, raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_JSON_BYTES {
            return Err("Template draft JSON size limit".into());
        }
        let d: Self = serde_json::from_str(raw)
            .map_err(|_| "Invalid bounded template draft JSON".to_string())?;
        if d.valid_for(source, current) {
            Ok(d)
        } else {
            Err("Saved template drafts differ from their held source/context".into())
        }
    }
    pub fn matches_state(&self, source: &State, current: &State) -> bool {
        self.valid_for(source, current)
            && self
                .leaves
                .iter()
                .all(|(key, d)| match current.configuration().entries.get(key) {
                    None => !d.included,
                    Some(e) => {
                        d.included
                            && mode(&d.mode) == Some(e.mode)
                            && matches!(&e.data,Kind::Text{contents}if contents==&d.contents)
                    }
                })
    }
    /// Saves ONE selected draft; all other pending drafts remain byte-for-byte.
    pub fn save_selected(&mut self, source: &State, current: &mut State) -> Result<(), String> {
        if !self.valid_for(source, current) {
            return Err("Template drafts differ from this source; previous work retained".into());
        }
        let draft = &self.leaves[&self.selected];
        let action = if draft.included {
            let parsed = mode(&draft.mode)
                .ok_or("Enter an octal mode from 0000 to 0777 using digits 0–7")?;
            if draft.contents.contains('\0') {
                return Err("Template text contains a NUL character; remove it and retry".into());
            }
            Action::SaveSkeletonLeaf {
                path: self.selected.clone(),
                contents: draft.contents.clone(),
                mode: parsed,
            }
        } else {
            if !source.policy().editable_skeleton_leaves[&self.selected].allow_remove {
                return Err("This template can be edited but must remain present".into());
            }
            if !current.configuration().entries.contains_key(&self.selected) {
                return Ok(());
            }
            Action::RemoveSkeletonLeaf {
                path: self.selected.clone(),
            }
        };
        accounts_teaching::apply_in_context(source, current, &action)?;
        if let Action::SaveSkeletonLeaf { mode, .. } = action {
            self.leaves.get_mut(&self.selected).unwrap().mode = format!("{mode:04o}");
        }
        Ok(())
    }
}
