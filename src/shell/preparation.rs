//! Internal resumable expansion preparation. Source evaluation precedes splitting
//! and globbing for the complete word list; evaluated text is never re-parsed.
use super::*;
use std::collections::VecDeque;

struct SourceWord {
    parts: std::vec::IntoIter<Part>,
    text: Vec<Part>,
    split: bool,
    sole: bool,
}
enum EvaluatedWord {
    Parts(Word, bool),
    Fields(Vec<String>),
}
pub(super) struct Words {
    source: VecDeque<SourceWord>,
    evaluated: VecDeque<EvaluatedWord>,
    fields: Vec<String>,
    ready: Option<Vec<String>>,
    retained: usize,
    splitting: bool,
}
impl Words {
    pub(super) fn new(words: Vec<(Word, bool)>) -> Self {
        Self {
            source: words
                .into_iter()
                .map(|(word, split)| SourceWord {
                    sole: word.0.len() == 1,
                    parts: word.0.into_iter(),
                    text: Vec::new(),
                    split,
                })
                .collect(),
            evaluated: VecDeque::new(),
            fields: Vec::new(),
            ready: None,
            retained: 0,
            splitting: false,
        }
    }
    pub(super) fn bytes(&self) -> usize {
        self.source
            .iter()
            .map(|word| {
                word.parts.as_slice().iter().map(part_bytes).sum::<usize>()
                    + word.text.iter().map(part_bytes).sum::<usize>()
            })
            .sum::<usize>()
            + self
                .evaluated
                .iter()
                .map(|word| match word {
                    EvaluatedWord::Parts(word, _) => word_bytes(word),
                    EvaluatedWord::Fields(fields) => strings_bytes(fields),
                })
                .sum::<usize>()
            + strings_bytes(&self.fields)
            + self
                .ready
                .as_ref()
                .map_or(0, |fields| strings_bytes(fields))
    }
    /// At most one source part or one prepared word is processed per call.
    /// Capture nodes remain unsupported even for internal callers at this stage.
    pub(super) fn advance(&mut self, vm: &Vm) -> Result<Option<Vec<String>>, String> {
        let old = self.bytes();
        let result = self.advance_inner(vm)?;
        let returned = result.as_ref().map_or(0, |fields| strings_bytes(fields));
        vm.rebase_pending(old, self.bytes() + returned)?;
        Ok(result)
    }
    fn advance_inner(&mut self, vm: &Vm) -> Result<Option<Vec<String>>, String> {
        if !self.splitting {
            if let Some(word) = self.source.front_mut() {
                if std::mem::take(&mut word.sole)
                    && let Some(fields) = vm.sole_fields(&word.parts.as_slice()[0])?
                {
                    self.retained =
                        bounded_expansion_bytes(self.retained, expansion_fields_size(&fields)?)?;
                    self.evaluated.push_back(EvaluatedWord::Fields(fields));
                    self.source.pop_front();
                    return Ok(None);
                }
                if let Some(part) = word.parts.next() {
                    let source = part.text().ok_or("Command substitution is unsupported")?;
                    let Expanded {
                        mut text,
                        memory: _memory,
                    } = if part.quote == Quote::Single {
                        vm.charge_expansion(source.len())?;
                        Expanded {
                            memory: vm.scratch(source.len())?,
                            text: source.to_string(),
                        }
                    } else {
                        vm.expand_text(source)?
                    };
                    vm.charge_expansion(text.len())?;
                    let mut compressed_memory = None;
                    if word.split && part.quote == Quote::Bare {
                        // Preserve delimiter presence without retaining an arbitrarily
                        // long run of default-IFS whitespace. No fields/globs are
                        // resolved yet; work is charged for the complete original text.
                        let original = text.len();
                        let mut whitespace = false;
                        text.retain(|c| {
                            let next = matches!(c, ' ' | '\t' | '\n');
                            let retain = !next || !whitespace;
                            whitespace = next;
                            retain
                        });
                        if text.len() != original {
                            // Release the large raw allocation rather than keeping
                            // its old capacity behind a one-byte delimiter.
                            compressed_memory = Some(vm.scratch(text.len())?);
                            text = text.as_str().to_owned();
                        }
                    }
                    self.retained = bounded_expansion_bytes(self.retained, text.len())?;
                    vm.reserve_memory(std::mem::size_of::<Part>())?;
                    word.text.push(Part {
                        value: PartValue::Text(text),
                        quote: part.quote,
                    });
                    drop(compressed_memory);
                    return Ok(None);
                }
                let word = self.source.pop_front().unwrap();
                self.evaluated
                    .push_back(EvaluatedWord::Parts(Word(word.text), word.split));
                return Ok(None);
            }
            self.splitting = true;
        }
        if let Some(fields) = self.ready.take() {
            // Moving payload does not copy it. The source and destination
            // String slots coexist only during this bounded transfer.
            vm.reserve_memory(std::mem::size_of_val(fields.as_slice()))?;
            self.fields.extend(fields);
            return Ok(None);
        }
        if let Some(word) = self.evaluated.pop_front() {
            let (fields, old_bytes) = match word {
                EvaluatedWord::Parts(mut word, split) => {
                    let bytes = word.0.iter().map(|part| part.text().unwrap().len()).sum();
                    let joined = !split || word.0.iter().all(|part| part.quote != Quote::Bare);
                    let simple = word.0.len() == 1
                        && (joined
                            || !word.0[0]
                                .text()
                                .unwrap()
                                .contains([' ', '\t', '\n', '*', '?', '[']));
                    let fields = if joined {
                        vm.reserve_memory(std::mem::size_of::<String>())?;
                        let mut parts = word.0.into_iter();
                        let mut output = parts.next().map_or_else(String::new, |part| {
                            let PartValue::Text(text) = part.value else {
                                unreachable!()
                            };
                            text
                        });
                        for part in parts {
                            let PartValue::Text(text) = part.value else {
                                unreachable!()
                            };
                            // The root ledger already owns all source payload. Only
                            // this appended copy coexists until its source drops.
                            vm.reserve_memory(text.len())?;
                            output.push_str(&text);
                        }
                        vec![output]
                    } else if simple {
                        vm.reserve_memory(std::mem::size_of::<String>())?;
                        let part = word.0.pop().unwrap();
                        let PartValue::Text(text) = part.value else {
                            unreachable!()
                        };
                        if split && part.quote == Quote::Bare && text.is_empty() {
                            Vec::new()
                        } else {
                            vec![text]
                        }
                    } else {
                        vm.split_word(&word, split)?
                    };
                    (fields, bytes)
                }
                EvaluatedWord::Fields(fields) => {
                    let bytes = expansion_fields_size(&fields)?;
                    (fields, bytes)
                }
            };
            if self.fields.len() + fields.len() > 128 {
                return Err("Argument limit exceeded (128 arguments)".into());
            }
            self.retained = bounded_expansion_bytes(
                self.retained - old_bytes,
                expansion_fields_size(&fields)?,
            )?;
            self.ready = Some(fields);
            return Ok(None);
        }
        Ok(Some(std::mem::take(&mut self.fields)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decimal_scratch_sizes_cover_signed_extremes_without_overflow() {
        for value in [i64::MIN, -10, -1, 0, 9, 10, 255, i64::MAX] {
            assert_eq!(decimal_bytes(value), value.to_string().len());
        }
    }
    fn vm() -> Vm {
        let fixture = Case {
            name: "Preparation continuation".into(),
            args: vec![],
            stdin: String::new(),
            env: BTreeMap::new(),
            files: BTreeMap::new(),
            stdout: String::new(),
            stderr: String::new(),
            status: 0,
            after: BTreeMap::new(),
        };
        Vm::new("", &fixture).unwrap()
    }
    fn word(text: &str, quote: Quote) -> Word {
        Word(vec![Part {
            value: PartValue::Text(text.into()),
            quote,
        }])
    }
    fn finish(words: &mut Words, vm: &Vm) -> Vec<String> {
        for _ in 0..100 {
            if let Some(fields) = words.advance(vm).unwrap() {
                return fields;
            }
        }
        panic!("preparation did not complete");
    }
    #[test]
    fn all_sources_are_prepared_before_any_filename_is_resolved() {
        let mut vm = vm();
        let mut words = Words::new(vec![
            (word("*.txt", Quote::Bare), true),
            (word("later", Quote::Double), true),
        ]);
        vm.pending_bytes.set(words.bytes());
        while !words.source.is_empty() {
            assert!(words.advance(&vm).unwrap().is_none());
            assert!(words.fields.is_empty());
        }
        // Models a later source-part side effect, without executing a capture.
        vm.outcome
            .files
            .insert("/work/new.txt".into(), String::new());
        assert_eq!(finish(&mut words, &vm), ["new.txt", "later"]);
    }
    #[test]
    fn resume_uses_saved_text_once_and_never_reinterprets_generated_syntax() {
        let mut vm = vm();
        vm.env
            .insert("value".into(), Value::Scalar("$other; 'quoted'".into()));
        let mut words = Words::new(vec![
            (word("$value", Quote::Double), true),
            (word("$value", Quote::Double), true),
        ]);
        vm.pending_bytes.set(words.bytes());
        words.advance(&vm).unwrap();
        vm.env
            .insert("value".into(), Value::Scalar("changed".into()));
        assert_eq!(finish(&mut words, &vm), ["$other; 'quoted'", "changed"]);
    }
    #[test]
    fn preparing_status_in_a_later_part_does_not_restart_prior_parts() {
        let mut vm = vm();
        let mut words = Words::new(vec![(
            Word(vec![
                Part {
                    value: PartValue::Text("$?".into()),
                    quote: Quote::Double,
                },
                Part {
                    value: PartValue::Text(":$?".into()),
                    quote: Quote::Double,
                },
            ]),
            true,
        )]);
        vm.pending_bytes.set(words.bytes());
        words.advance(&vm).unwrap();
        vm.outcome.status = 7;
        assert_eq!(finish(&mut words, &vm), ["0:7"]);
    }
}
