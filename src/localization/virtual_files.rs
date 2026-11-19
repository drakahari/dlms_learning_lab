//! Bounded byte documents for conversion practice. No host path is opened.
use super::encoding::{self, Conversion, Encoding};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
type Result<T> = std::result::Result<T, String>;
pub const MAX_OUTPUT_FILE: usize = 32 * 1024;
pub const MAX_OUTPUT_TOTAL: usize = 128 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,u8,16384>")]
    pub bytes: Vec<u8>,
}
fn canonical(path: &str) -> Result<String> {
    let name = path.strip_prefix("/work/").unwrap_or(path);
    if path.len() > 128
        || name.is_empty()
        || name.len() > 64
        || matches!(name, "." | "..")
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
    {
        return Err("Use a supplied /work/name or a simple filename; directories, traversal and host paths are outside this activity.".into());
    }
    Ok(format!("/work/{name}"))
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Files {
    inputs: Vec<Input>,
    outputs: BTreeMap<String, Vec<u8>>,
}
impl Files {
    pub fn new(inputs: Vec<Input>) -> Result<Self> {
        let state = Self {
            inputs,
            outputs: BTreeMap::new(),
        };
        state.validate()?;
        Ok(state)
    }
    pub fn validate(&self) -> Result<()> {
        if self.inputs.is_empty()
            || self.inputs.len() > 8
            || self.inputs.iter().map(|f| f.bytes.len()).sum::<usize>() > 64 * 1024
        {
            return Err("Supply one to eight read-only documents, at most64KiB total.".into());
        }
        for (n, file) in self.inputs.iter().enumerate() {
            if canonical(&file.path)? != file.path
                || file.label.is_empty()
                || file.label.len() > 128
                || file.label.chars().any(char::is_control)
                || file.bytes.len() > encoding::MAX_INPUT
                || self.inputs[..n].iter().any(|old| old.path == file.path)
            {
                return Err("Supplied documents need distinct canonical paths, readable labels and at most16KiB each.".into());
            }
        }
        if self.outputs.len() > 8
            || self.outputs.values().map(Vec::len).sum::<usize>() > MAX_OUTPUT_TOTAL
        {
            return Err("Simulator output limit: eight files and128KiB total.".into());
        }
        for (path, bytes) in &self.outputs {
            if canonical(path)? != *path
                || self.inputs.iter().any(|f| f.path == *path)
                || bytes.len() > MAX_OUTPUT_FILE
            {
                return Err("Invalid saved output document.".into());
            }
        }
        Ok(())
    }
    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }
    pub fn outputs(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.outputs
    }
    pub fn read(&self, path: &str) -> Result<&[u8]> {
        let path = canonical(path)?;
        if let Some(file) = self.inputs.iter().find(|f| f.path == path) {
            return Ok(&file.bytes);
        }
        self.outputs.get(&path).map(Vec::as_slice).ok_or_else(|| {
            format!(
                "No simulated document {path}. Inspect the supplied files and your saved outputs."
            )
        })
    }
    pub fn write(&mut self, path: &str, bytes: Vec<u8>) -> Result<()> {
        let path = canonical(path)?;
        if self.inputs.iter().any(|f| f.path == path) {
            return Err("Supplied inputs are read-only in this activity. Choose a different /work/name for the output.".into());
        }
        if bytes.len() > MAX_OUTPUT_FILE
            || (!self.outputs.contains_key(&path) && self.outputs.len() == 8)
        {
            return Err(
                "Simulator output limit: eight files,32KiB each. No output was replaced.".into(),
            );
        }
        let total = self
            .outputs
            .iter()
            .filter(|(name, _)| **name != path)
            .map(|(_, b)| b.len())
            .sum::<usize>();
        if bytes.len() > MAX_OUTPUT_TOTAL - total {
            return Err("Simulator output limit:128KiB total. No output was replaced.".into());
        }
        self.outputs.insert(path, bytes);
        Ok(())
    }
    /// Failed conversion can still write its valid prefix; report failure and
    /// grade the actual bytes rather than treating every failure as atomic.
    pub fn convert(
        &mut self,
        input: &str,
        output: Option<&str>,
        from: Encoding,
        to: Encoding,
    ) -> Result<Conversion> {
        if let Some(path) = output {
            let path = canonical(path)?;
            if self.inputs.iter().any(|f| f.path == path) {
                return Err(
                    "Supplied inputs are read-only in this activity. Choose a new output path."
                        .into(),
                );
            }
        }
        let converted = encoding::convert(self.read(input)?, from, to)?;
        if let Some(path) = output {
            self.write(path, converted.bytes.clone())?;
        }
        Ok(converted)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> Input {
        Input {
            path: "/work/feed.txt".into(),
            label: "Supplied Latin-1 feed".into(),
            bytes: b"Caf\xe9\n".to_vec(),
        }
    }
    #[test]
    fn actual_conversion_creates_different_bytes_and_preserves_original() {
        let mut f = Files::new(vec![input()]).unwrap();
        let r = f
            .convert(
                "feed.txt",
                Some("fixed.txt"),
                Encoding::Latin1,
                Encoding::Utf8,
            )
            .unwrap();
        assert!(r.failure.is_none());
        assert_eq!(f.read("fixed.txt").unwrap(), "Café\n".as_bytes());
        assert_eq!(f.read("feed.txt").unwrap(), b"Caf\xe9\n");
        f.validate().unwrap();
    }
    #[test]
    fn failed_conversion_saves_actual_prefix_and_retry_replaces_it() {
        let mut f = Files::new(vec![input()]).unwrap();
        let r = f
            .convert(
                "feed.txt",
                Some("fixed.txt"),
                Encoding::Latin1,
                Encoding::Ascii,
            )
            .unwrap();
        assert!(r.failure.is_some());
        assert_eq!(f.read("fixed.txt").unwrap(), b"Caf");
        f.convert(
            "feed.txt",
            Some("fixed.txt"),
            Encoding::Latin1,
            Encoding::Utf8,
        )
        .unwrap();
        assert_eq!(f.read("fixed.txt").unwrap(), "Café\n".as_bytes());
    }
    #[test]
    fn viewing_conversion_does_not_create_an_output_or_modify_other_files() {
        let mut f = Files::new(vec![input()]).unwrap();
        let before = f.clone();
        let r = f
            .convert("feed.txt", None, Encoding::Latin1, Encoding::Utf8)
            .unwrap();
        assert_eq!(r.bytes, "Café\n".as_bytes());
        assert_eq!(f, before);
        assert!(f.read("fixed.txt").is_err());
    }
    #[test]
    fn host_paths_traversal_readonly_sources_and_file_limits_refuse_without_mutation() {
        let mut f = Files::new(vec![input()]).unwrap();
        for path in [
            "../feed.txt",
            "/etc/passwd",
            "/work/../private",
            "/work/sub/file",
            "~/.profile",
            "/work/",
            "..",
            "a b",
        ] {
            let before = f.clone();
            assert!(f.write(path, vec![1]).is_err());
            assert!(f.read(path).is_err());
            assert_eq!(f, before);
        }
        let before = f.clone();
        assert!(
            f.convert(
                "feed.txt",
                Some("/work/feed.txt"),
                Encoding::Latin1,
                Encoding::Utf8
            )
            .is_err()
        );
        assert_eq!(f, before);
        assert!(f.write("huge.txt", vec![0; MAX_OUTPUT_FILE + 1]).is_err());
        assert_eq!(f, before);
        for n in 0..8 {
            f.write(&format!("part{n}.txt"), vec![n]).unwrap();
        }
        let before = f.clone();
        assert!(f.write("ninth.txt", vec![9]).is_err());
        assert_eq!(f, before);
        f.write("part0.txt", vec![7]).unwrap();
        assert_eq!(f.read("part0.txt").unwrap(), [7]);
    }
    #[test]
    fn total_byte_bound_and_malformed_source_or_oversized_deserialization_refuse() {
        let mut f = Files::new(vec![input()]).unwrap();
        for n in 0..4 {
            f.write(&format!("out{n}"), vec![0; MAX_OUTPUT_FILE])
                .unwrap();
        }
        let before = f.clone();
        assert!(f.write("extra", vec![1]).is_err());
        assert_eq!(f, before);
        let mut bad = input();
        bad.path = "/work/../feed.txt".into();
        assert!(Files::new(vec![bad]).is_err());
        assert!(Files::new(vec![input(), input()]).is_err());
        let mut value = serde_json::to_value(input()).unwrap();
        value["bytes"] = serde_json::json!(vec![0; encoding::MAX_INPUT + 1]);
        assert!(serde_json::from_value::<Input>(value).is_err());
    }
}
