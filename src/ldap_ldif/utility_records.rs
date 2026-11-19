//! Record-at-a-time utility input; the existing full-document API is unchanged.
use super::*;

/// Bounded borrowed input. A later record error does not pre-parse earlier ones.
/// Aggregate byte/physical-line bounds are checked before any record is yielded.
pub struct UtilityRecords<'a> {
    remaining: &'a str,
    line: usize,
    header: bool,
    records: usize,
    add_default: bool,
    done: bool,
}
impl<'a> UtilityRecords<'a> {
    pub fn new(raw: &'a str, add_default: bool) -> Result<Self> {
        if raw.len() > MAX_TEXT || raw.split_terminator('\n').count() > MAX_LINES {
            return Err(error(
                Category::Limit,
                1,
                "Utility input allows at most 64 KiB and 1024 physical lines; no requests were attempted.",
            ));
        }
        Ok(Self {
            remaining: raw,
            line: 1,
            header: false,
            records: 0,
            add_default,
            done: false,
        })
    }
    fn record(&mut self) -> Result<Option<Change>> {
        while !self.remaining.is_empty() {
            let start_line = self.line;
            let mut end = 0;
            for physical in self.remaining.split_inclusive('\n') {
                end += physical.len();
                self.line += 1;
                let line = physical.strip_suffix('\n').unwrap_or(physical);
                if line.strip_suffix('\r').unwrap_or(line).is_empty() {
                    break;
                }
            }
            let chunk = &self.remaining[..end];
            self.remaining = &self.remaining[end..];
            let mut lines = unfold(chunk).map_err(|mut e| {
                e.line += start_line - 1;
                e
            })?;
            for line in &mut lines {
                line.number += start_line - 1;
            }
            lines.retain(|l| !l.text.is_empty());
            if lines.is_empty() {
                continue;
            }
            if !self.header {
                let (name, value) = field(&lines[0])?;
                if name != "version" || value != "1" {
                    return Err(error(
                        Category::Unsupported,
                        lines[0].number,
                        "This utility subset requires explicit version: 1 before its records.",
                    ));
                }
                self.header = true;
                lines.remove(0);
            }
            if lines.is_empty() {
                continue;
            }
            if self.records >= MAX_RECORDS {
                return Err(error(
                    Category::Limit,
                    lines[0].number,
                    "At most 16 utility records; earlier successful requests remain applied.",
                ));
            }
            self.records += 1;
            let name = dn(&lines[0])?;
            let second = lines.get(1).map(field).transpose()?;
            if let Some((field, value)) = second.filter(|(n, _)| n == "changetype") {
                debug_assert_eq!(field, "changetype");
                return change_record(&lines, name, &value).map(Some);
            }
            if !self.add_default {
                return Err(error(
                    Category::Unsupported,
                    lines[0].number,
                    "Without -a, this ldapmodify subset requires an explicit changetype. Implicit attribute replacement is not implemented.",
                ));
            }
            return Ok(Some(Change::Add(EntrySource {
                dn: name.text().into(),
                attributes: attributes(&lines[1..])?,
            })));
        }
        if self.records == 0 {
            return Err(error(
                Category::Syntax,
                self.line.saturating_sub(1).max(1),
                "Start with version: 1 and include at least one directory record.",
            ));
        }
        Ok(None)
    }
}
impl Iterator for UtilityRecords<'_> {
    type Item = Result<Change>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        match self.record() {
            Ok(Some(record)) => Some(Ok(record)),
            Ok(None) => {
                self.done = true;
                None
            }
            Err(e) => {
                self.done = true;
                Some(Err(e))
            }
        }
    }
}
impl std::iter::FusedIterator for UtilityRecords<'_> {}
