//! Source-bound virtual modal-editing activities. Paths are labels in an owned
//! map, never host paths. No filesystem, process, clipboard or network bridge.
use crate::{
    modal_editing::{Editor, Key, Mode},
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
const MAX_FILES: usize = 4;
const MAX_FILE: usize = 4096;
fn bounded_map<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<BTreeMap<String, T>, D::Error> {
    use serde::de::{Error, MapAccess, Visitor};
    struct Name(String);
    impl<'de> Deserialize<'de> for Name {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            crate::cron::text::<D, 120>(d).map(Self)
        }
    }
    struct Entries<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Entries<T, N> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {N} unique bounded keys")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut rows = BTreeMap::new();
            while let Some(Name(key)) = map.next_key::<Name>()? {
                if rows.len() == N || rows.contains_key(&key) {
                    return Err(A::Error::custom(
                        "Duplicate key or virtual editing map limit",
                    ));
                }
                rows.insert(key, map.next_value()?);
            }
            Ok(rows)
        }
    }
    d.deserialize_map(Entries::<T, N>(std::marker::PhantomData))
}
fn optional_text<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    struct Optional;
    impl<'de> serde::de::Visitor<'de> for Optional {
        type Value = Option<String>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("optional bounded text")
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: serde::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            crate::cron::text::<D, 4096>(d).map(Some)
        }
    }
    d.deserialize_option(Optional)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub text: String,
    pub writable: bool,
    #[serde(default, deserialize_with = "optional_text")]
    pub draft: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Settings {
        path: String,
        values: BTreeMap<String, String>,
    },
    Records {
        path: String,
        counts: BTreeMap<String, usize>,
    },
    Unchanged {
        path: String,
    },
}
impl Requirement {
    pub fn path(&self) -> &str {
        match self {
            Self::Settings { path, .. } | Self::Records { path, .. } | Self::Unchanged { path } => {
                path
            }
        }
    }
    pub fn label(&self) -> String {
        match self {
            Self::Settings { path, .. } => format!("Requested settings in {path}"),
            Self::Records { path, .. } => format!("Requested records in {path}"),
            Self::Unchanged { path } => format!("Preserve {path}"),
        }
    }
    fn check(&self, initial: &str, actual: &str) -> (bool, String) {
        let result: Result<(bool, String), String> = (|| {
            Ok(match self {
                Self::Settings { values, .. } => {
                    let before = settings(initial)?;
                    let after = settings(actual)?;
                    let keys = before.keys().eq(after.keys());
                    let requested = values.iter().all(|(k, v)| after.get(k) == Some(v));
                    let other = before
                        .iter()
                        .filter(|(k, _)| !values.contains_key(*k))
                        .all(|(k, v)| after.get(k) == Some(v));
                    let pass = keys && requested && other;
                    let mut expected = before.clone();
                    expected.extend(values.clone());
                    let names: std::collections::BTreeSet<_> =
                        expected.keys().chain(after.keys()).collect();
                    let why = if pass {
                        "Requested saved values match; other keys and values are preserved.".into()
                    } else {
                        let differences = names
                            .into_iter()
                            .filter(|k| expected.get(*k) != after.get(*k))
                            .take(8)
                            .map(|k| match (expected.get(k), after.get(k)) {
                                (Some(want), Some(got)) => {
                                    format!("{k}: expected {}, saved {}", short(want), short(got))
                                }
                                (Some(_), None) => format!("{k}: missing saved setting"),
                                _ => format!("{k}: unexpected added setting"),
                            })
                            .collect::<Vec<_>>();
                        format!(
                            "{}. Repair these saved values; spaces around = and reordered keys are allowed.",
                            differences.join("; ")
                        )
                    };
                    (pass, why)
                }
                Self::Records {
                    counts: targets, ..
                } => {
                    let before = records(initial);
                    let after = records(actual);
                    let mut expected = before;
                    for (line, n) in targets {
                        if *n == 0 {
                            expected.remove(line);
                        } else {
                            expected.insert(line.clone(), *n);
                        }
                    }
                    let pass = after == expected;
                    let why = if pass {
                        "Saved complete-line counts match; all other records are preserved.".into()
                    } else {
                        let lines: std::collections::BTreeSet<_> =
                            expected.keys().chain(after.keys()).collect();
                        let differences = lines
                            .into_iter()
                            .filter(|l| expected.get(*l) != after.get(*l))
                            .take(8)
                            .map(|l| {
                                format!(
                                    "{}: expected {}, saved {}",
                                    short(l),
                                    expected.get(l).copied().unwrap_or(0),
                                    after.get(l).copied().unwrap_or(0)
                                )
                            })
                            .collect::<Vec<_>>();
                        format!(
                            "{}. Check the whole line before copying or deleting it.",
                            differences.join("; ")
                        )
                    };
                    (pass, why)
                }
                Self::Unchanged { .. } => {
                    if actual == initial {
                        (true, "Original saved contents are preserved.".into())
                    } else {
                        (false,"The saved contents changed. Undoing a buffer does not rewrite saved bytes; restore the intended original before writing it. :q! discards an unsaved draft only.".into())
                    }
                }
            })
        })();
        result.unwrap_or_else(|why| (false, why))
    }
}
fn short(text: &str) -> String {
    let out: String = text.chars().take(72).collect();
    format!("{out:?}{}", if text.len() > 72 { "..." } else { "" })
}
fn settings(text: &str) -> Result<BTreeMap<String, String>, String> {
    let mut result = BTreeMap::new();
    for line in text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let (k, v) = line
            .split_once('=')
            .ok_or("A setting needs key=value; preserve complete lines.")?;
        let (k, v) = (k.trim(), v.trim());
        if k.is_empty()
            || !k
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
            || v.is_empty()
            || result.insert(k.into(), v.into()).is_some()
        {
            return Err(
                "A setting is empty, repeated or malformed. Each key needs one nonempty value."
                    .into(),
            );
        }
    }
    Ok(result)
}
fn records(text: &str) -> BTreeMap<String, usize> {
    let mut rows = BTreeMap::new();
    for line in text.lines() {
        *rows.entry(line.into()).or_insert(0) += 1;
    }
    rows
}
fn valid_path(path: &str) -> bool {
    path.starts_with('/')
        && path.len() <= 120
        && !path.ends_with('/')
        && path.split('/').skip(1).all(|c| {
            !c.is_empty()
                && c != "."
                && c != ".."
                && c.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    #[serde(deserialize_with = "bounded_map::<_,File,4>")]
    pub files: BTreeMap<String, File>,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub start: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,12>")]
    pub requirements: Vec<Requirement>,
}
impl Fixture {
    pub fn validate(&self) -> Result<(), String> {
        if self.files.is_empty()
            || self.files.len() > MAX_FILES
            || !self.files.contains_key(&self.start)
            || self.requirements.is_empty()
            || self.requirements.len() > 12
        {
            return Err("Editing activity needs bounded supplied files and requirements.".into());
        }
        for (path, file) in &self.files {
            if !valid_path(path) || file.text.len() > MAX_FILE {
                return Err("Invalid virtual file label or file over4 KiB.".into());
            }
            let e = if let Some(draft) = &file.draft {
                if draft.len() > MAX_FILE || draft == &file.text {
                    return Err("Recovery starter needs a distinct, bounded unsaved buffer.".into());
                }
                Editor::with_draft(&file.text, draft)?
            } else {
                Editor::new(&file.text)?
            };
            if file.draft.is_some() && !e.modified() {
                return Err(
                    "Recovery starter must actually differ from the saved logical buffer.".into(),
                );
            }
            if e.contents().len() > MAX_FILE {
                return Err("Logical file with its final line ending exceeds4 KiB.".into());
            }
        }
        for r in &self.requirements {
            let file = self
                .files
                .get(r.path())
                .ok_or("Requirement references an unknown virtual file.")?;
            match r {
                Requirement::Settings { values, .. } => {
                    let source = settings(&file.text)?;
                    if !file.writable
                        || values.is_empty()
                        || values.len() > 16
                        || values.iter().any(|(k, v)| {
                            !source.contains_key(k)
                                || v.is_empty()
                                || v.len() > 120
                                || !v.bytes().all(|b| (32..=126).contains(&b))
                        })
                    {
                        return Err("Invalid setting requirement.".into());
                    }
                }
                Requirement::Records { counts, .. } => {
                    if !file.writable
                        || counts.is_empty()
                        || counts.len() > 32
                        || counts.iter().any(|(s, n)| {
                            s.is_empty()
                                || s.len() > 200
                                || s.contains('\n')
                                || !s.bytes().all(|b| (32..=126).contains(&b))
                                || *n > 8
                        })
                    {
                        return Err("Invalid record requirement.".into());
                    }
                }
                Requirement::Unchanged { .. } => {}
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Event {
    Open { path: String },
    Key { key: Key },
    Insert { text: String },
    Check {},
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    #[serde(deserialize_with = "bounded_map::<_,Editor,4>")]
    pub editors: BTreeMap<String, Editor>,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub selected: String,
    pub checked: bool,
}
impl Attempt {
    pub fn new(f: &Fixture) -> Result<Self, String> {
        f.validate()?;
        Ok(Self {
            editors: f
                .files
                .iter()
                .map(|(p, file)| {
                    Ok((
                        p.clone(),
                        if let Some(draft) = &file.draft {
                            Editor::with_draft(&file.text, draft)?
                        } else {
                            Editor::new(&file.text)?
                        },
                    ))
                })
                .collect::<Result<_, String>>()?,
            selected: f.start.clone(),
            checked: false,
        })
    }
    pub fn valid_for(&self, f: &Fixture) -> Result<(), String> {
        f.validate()?;
        if !self.editors.keys().eq(f.files.keys()) || !self.editors.contains_key(&self.selected) {
            return Err("Saved editor files do not match this activity.".into());
        }
        for (path, e) in &self.editors {
            e.validate()?;
            if e.file_bytes().len() > MAX_FILE || e.contents().len() > MAX_FILE {
                return Err("Activity file exceeds4 KiB.".into());
            }
            if !f.files[path].writable && e.file_bytes() != f.files[path].text {
                return Err(
                    "A read-only virtual file was changed; retain this data before resetting."
                        .into(),
                );
            }
        }
        Ok(())
    }
    pub fn editor(&self) -> &Editor {
        &self.editors[&self.selected]
    }
    pub fn checks(&self, f: &Fixture) -> Vec<(bool, String)> {
        if let Err(e) = self.valid_for(f) {
            return vec![(false, e)];
        }
        let dirty: Vec<_> = self
            .editors
            .iter()
            .filter(|(_, e)| e.modified())
            .map(|(path, _)| path.as_str())
            .collect();
        let mut rows = vec![(
            dirty.is_empty(),
            if dirty.is_empty() {
                "All supplied buffers are saved or clean.".into()
            } else {
                format!(
                    "Unsaved buffer: {}. Save or discard its edits before checking; the check reads saved virtual files.",
                    dirty.join(", ")
                )
            },
        )];
        rows.extend(f.requirements.iter().map(|r| {
            let (pass, why) = r.check(&f.files[r.path()].text, self.editors[r.path()].file_bytes());
            (pass, format!("{} · {why}", r.label()))
        }));
        rows
    }
    pub fn solved(&self, f: &Fixture) -> bool {
        self.checked && self.checks(f).iter().all(|r| r.0)
    }
    pub fn apply(&mut self, f: &Fixture, event: &Event) -> Result<(), String> {
        self.valid_for(f)?;
        let mut next = self.clone();
        match event {
            Event::Open { path } => {
                if !f.files.contains_key(path) {
                    return Err(
                        "Choose a supplied virtual file; arbitrary paths are unavailable.".into(),
                    );
                }
                if self.editor().modified() && !self.editor().closed() {
                    return Err("Save with :w or discard with :q! before changing files. Your current draft is retained.".into());
                }
                next.selected = path.clone();
                if next.editor().closed() {
                    next.editors
                        .insert(path.clone(), Editor::new(next.editor().file_bytes())?);
                }
            }
            Event::Key { key } => {
                if !f.files[&self.selected].writable
                    && *key == Key::Enter
                    && self.editor().mode() == Mode::Command
                    && matches!(self.editor().command().trim(), "w" | "wq")
                {
                    return Err("This supplied virtual file is read-only. Use :q! to discard its draft; writing it is not allowed.".into());
                }
                next.editors
                    .get_mut(&self.selected)
                    .ok_or("No selected virtual editor.")?
                    .key(*key)?;
                let before = self.editor();
                let after = next.editor();
                if before.contents() != after.contents()
                    || before.file_bytes() != after.file_bytes()
                    || before.modified() != after.modified()
                {
                    next.checked = false;
                }
            }
            Event::Insert { text } => {
                next.editors
                    .get_mut(&self.selected)
                    .ok_or("No selected virtual editor.")?
                    .insert_text(text)?;
                let before = self.editor();
                let after = next.editor();
                if before.contents() != after.contents()
                    || before.file_bytes() != after.file_bytes()
                    || before.modified() != after.modified()
                {
                    next.checked = false;
                }
            }
            Event::Check {} => next.checked = true,
        }
        next.valid_for(f)?;
        *self = next;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub id: String,
    pub initial: Fixture,
    pub example: Fixture,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Event,512>")]
    pub example_solution: Vec<Event>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub id: String,
    pub version: u32,
    pub view: View,
    pub assignment: Attempt,
    pub practice: Attempt,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.id.is_empty()
            || self.id.len() > 80
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || self.example_solution.is_empty()
            || self.example_solution.len() > 512
        {
            return Err("Invalid authored editing activity identity or worked sequence.".into());
        }
        self.initial.validate()?;
        self.example.validate()?;
        if Attempt::new(&self.initial)?
            .checks(&self.initial)
            .iter()
            .all(|r| r.0)
        {
            return Err("Start the assignment with a concrete incomplete editing task or an unsaved recovery draft.".into());
        }

        let mut p = Attempt::new(&self.example)?;
        for event in &self.example_solution {
            p.apply(&self.example, event)?;
        }
        p.apply(&self.example, &Event::Check {})?;
        if !p.solved(&self.example) {
            return Err("The authored editing example does not meet its requirements.".into());
        }
        Ok(())
    }
    pub fn new_session(&self) -> Result<Session, String> {
        self.validate()?;
        Ok(Session {
            id: self.id.clone(),
            version: self.version,
            view: View::Introduction,
            assignment: Attempt::new(&self.initial)?,
            practice: Attempt::new(&self.example)?,
        })
    }
    pub fn context(&self, s: &Session) -> Result<(), String> {
        self.validate()?;
        if s.version != self.version || s.id != self.id {
            return Err("Saved editing activity identity does not match its source.".into());
        }
        s.assignment.valid_for(&self.initial)?;
        s.practice.valid_for(&self.example)
    }
}
impl Session {
    pub fn apply(&mut self, d: &Definition, practice: bool, event: &Event) -> Result<(), String> {
        d.context(self)?;
        if practice {
            self.practice.apply(&d.example, event)
        } else {
            self.assignment.apply(&d.initial, event)
        }
    }
    pub fn solved(&self, d: &Definition) -> bool {
        d.context(self).is_ok() && self.assignment.solved(&d.initial)
    }
    pub fn reset_assignment(&mut self, d: &Definition) -> Result<(), String> {
        d.context(self)?;
        self.assignment = Attempt::new(&d.initial)?;
        Ok(())
    }
    pub fn reset_practice(&mut self, d: &Definition) -> Result<(), String> {
        d.context(self)?;
        self.practice = Attempt::new(&d.example)?;
        Ok(())
    }
    pub fn show_worked(&mut self, d: &Definition) -> Result<(), String> {
        d.context(self)?;
        let mut p = Attempt::new(&d.example)?;
        for event in &d.example_solution {
            p.apply(&d.example, event)?;
        }
        p.checked = false; // Showing an answer never presses Check for the learner.
        self.practice = p;
        Ok(())
    }
}
