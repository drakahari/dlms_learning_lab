//! Original scalar case preparation and resumable C-byte matching. No file lookup.
//! Definitions share the immutable instruction arena; tasks own only reached words.
use super::*;

#[derive(Debug)]
pub(super) struct Arm {
    pub patterns: Vec<Word>,
    pub entry: usize,
}
#[derive(Debug)]
pub(super) struct Definition {
    pub selector: Word,
    pub arms: Vec<Arm>,
    pub end: usize,
}
impl Definition {
    pub fn text_bytes(&self) -> usize {
        std::iter::once(&self.selector)
            .chain(self.arms.iter().flat_map(|arm| &arm.patterns))
            .flat_map(|word| &word.0)
            .map(|part| part.text().map_or(0, str::len))
            .sum()
    }
}
/// Ordinary command/pattern source retains the old compile-first bracket boundary.
/// A selector is scalar data and deliberately bypasses this filename-only rule.
pub(super) fn check_static_word(word: &Word) -> Result<(), String> {
    for part in &word.0 {
        if part.quote == Quote::Bare
            && let Some(text) = part.text()
            && values::bracket_glob(text)?
            && text != "["
            && text != "]"
        {
            return Err("Bracket globbing is unsupported; quote literal brackets".into());
        }
    }
    Ok(())
}

struct Scalar {
    parts: std::vec::IntoIter<Part>,
    text: String,
    active: Vec<u8>,
    pattern: bool,
    waiting: Option<Quote>,
}
impl Scalar {
    fn new(vm: &Vm, word: &Word, pattern: bool) -> Result<Self, String> {
        vm.reserve_memory(word_bytes(word))?;
        Ok(Self {
            parts: word.0.clone().into_iter(),
            text: String::new(),
            active: Vec::new(),
            pattern,
            waiting: None,
        })
    }
    fn bytes(&self) -> usize {
        self.parts.as_slice().iter().map(part_bytes).sum::<usize>()
            + self.text.len()
            + self.active.len()
    }
    fn append(&mut self, vm: &Vm, text: &str, quote: Quote) -> Result<(), String> {
        bounded_expansion_bytes(self.text.len(), text.len())?;
        if self.pattern && self.text.len() + text.len() > 256 {
            return Err("Case pattern limit exceeded (256 bytes)".into());
        }
        vm.charge_expansion(text.len())?;
        vm.reserve_memory(text.len() * if self.pattern { 2 } else { 1 })?;
        self.text.push_str(text);
        if self.pattern {
            self.active.extend(std::iter::repeat_n(
                u8::from(quote == Quote::Bare),
                text.len(),
            ));
        }
        Ok(())
    }
    /// One part per preparation unit. No sole_fields, splitting or filename globbing.
    fn advance(&mut self, vm: &Vm) -> Result<bool, String> {
        if self.waiting.is_some() {
            return Err("Missing case capture continuation".into());
        }
        let Some(part) = self.parts.next() else {
            return Ok(true);
        };
        if let PartValue::Substitution(body) = &part.value {
            if vm.capture_request.get().is_some() {
                return Err("Conflicting capture request".into());
            }
            self.waiting = Some(part.quote);
            vm.capture_request.set(Some(*body));
            return Ok(false);
        }
        let source = part.text().ok_or("Invalid case scalar part")?;
        if part.quote == Quote::Single {
            self.append(vm, source, part.quote)?;
        } else {
            // Existing scalar parameter reader rejects all list-valued forms,
            // including sole quoted $@ / array-all (ordinary command usage unchanged).
            let expanded = vm.expand_text(source)?;
            self.append(vm, &expanded.text, part.quote)?;
        }
        Ok(false)
    }
    fn resume(&mut self, vm: &Vm, value: String) -> Result<(), String> {
        let quote = self.waiting.take().ok_or("Missing case capture quote")?;
        // Returned capture bytes currently live outside pending; charge that live
        // source as scratch until the appended destination is in the ledger.
        let _source = vm.scratch(value.len())?;
        self.append(vm, &value, quote)
    }
}

#[derive(Default)]
struct Matcher {
    pattern: usize,
    subject: usize,
    star: Option<usize>,
    retry: usize,
}
impl Matcher {
    fn validate(vm: &Vm, pattern: &Scalar) -> Result<(), String> {
        vm.charge_expansion(pattern.text.len())?;
        for (i, &byte) in pattern.text.as_bytes().iter().enumerate() {
            if pattern.active[i] == 0 {
                continue;
            }
            if matches!(byte, b'\\' | b'[' | b']') {
                return Err(
                    "Unsupported active backslash or bracket case pattern; quote literal text"
                        .into(),
                );
            }
            if matches!(byte, b'@' | b'!' | b'+' | b'?' | b'*')
                && pattern.text.as_bytes().get(i + 1) == Some(&b'(')
                && pattern.active.get(i + 1) == Some(&1)
            {
                return Err("Unsupported extended case pattern; use * or ? only".into());
            }
        }
        Ok(())
    }
    /// At most256 charged transitions, independent of the semantic instruction budget.
    fn advance(
        &mut self,
        vm: &Vm,
        subject: &str,
        pattern: &Scalar,
    ) -> Result<Option<bool>, String> {
        let text = pattern.text.as_bytes();
        let subject = subject.as_bytes();
        let star = |p: usize| text.get(p) == Some(&b'*') && pattern.active.get(p) == Some(&1);
        for _ in 0..256 {
            vm.charge_expansion(1)?;
            if self.subject == subject.len() {
                if star(self.pattern) {
                    self.pattern += 1;
                    continue;
                }
                return Ok(Some(self.pattern == text.len()));
            }
            if star(self.pattern) {
                self.star = Some(self.pattern);
                self.pattern += 1;
                self.retry = self.subject;
            } else if text.get(self.pattern).is_some_and(|byte| {
                (*byte == b'?' && pattern.active[self.pattern] == 1)
                    || *byte == subject[self.subject]
            }) {
                self.pattern += 1;
                self.subject += 1;
            } else if let Some(star) = self.star {
                self.retry += 1;
                self.subject = self.retry;
                self.pattern = star + 1;
            } else {
                return Ok(Some(false));
            }
        }
        Ok(None)
    }
}

pub(super) struct Task {
    definition: Arc<Definition>,
    subject: String,
    scalar: Option<Scalar>,
    arm: usize,
    alternative: usize,
    selecting: bool,
    matcher: Option<Matcher>,
}
impl Task {
    pub fn new(vm: &Vm, definition: Arc<Definition>) -> Result<Self, String> {
        let scalar = Scalar::new(vm, &definition.selector, false)?;
        Ok(Self {
            definition,
            subject: String::new(),
            scalar: Some(scalar),
            arm: 0,
            alternative: 0,
            selecting: true,
            matcher: None,
        })
    }
    pub fn bytes(&self) -> usize {
        self.subject.len() + self.scalar.as_ref().map_or(0, Scalar::bytes)
    }
    pub fn resume(&mut self, vm: &Vm, value: String) -> Result<(), String> {
        let old = self.bytes();
        self.scalar
            .as_mut()
            .ok_or("Missing case scalar")?
            .resume(vm, value)?;
        vm.rebase_pending(old, self.bytes())
    }
    pub fn advance(&mut self, vm: &mut Vm) -> Result<bool, String> {
        let old = self.bytes();
        let done = self.advance_inner(vm)?;
        vm.rebase_pending(old, self.bytes())?;
        Ok(done)
    }
    fn advance_inner(&mut self, vm: &mut Vm) -> Result<bool, String> {
        if self.selecting {
            let scalar = self.scalar.as_mut().ok_or("Missing case selector")?;
            if !scalar.advance(vm)? {
                return Ok(false);
            }
            self.subject = std::mem::take(&mut scalar.text);
            self.scalar = None;
            self.selecting = false;
            return Ok(false);
        }
        let Some(arm) = self.definition.arms.get(self.arm) else {
            self.scalar = None;
            self.subject = String::new();
            vm.outcome.status = 0;
            vm.pc = self.definition.end;
            return Ok(true);
        };
        if self.scalar.is_none() {
            self.scalar = Some(Scalar::new(vm, &arm.patterns[self.alternative], true)?);
            return Ok(false);
        }
        let pattern = self.scalar.as_mut().unwrap();
        if self.matcher.is_none() {
            if !pattern.advance(vm)? {
                return Ok(false);
            }
            Matcher::validate(vm, pattern)?;
            self.matcher = Some(Matcher::default());
            return Ok(false);
        }
        let Some(matched) = self
            .matcher
            .as_mut()
            .unwrap()
            .advance(vm, &self.subject, pattern)?
        else {
            return Ok(false);
        };
        if matched {
            self.scalar = None;
            self.subject = String::new();
            vm.pc = arm.entry;
            return Ok(true);
        }
        self.scalar = None;
        self.matcher = None;
        self.alternative += 1;
        if self.alternative == arm.patterns.len() {
            self.arm += 1;
            self.alternative = 0;
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Case {
        Case {
            name: "Private case ledger fixture".into(),
            args: vec![],
            stdin: String::new(),
            env: BTreeMap::new(),
            files: BTreeMap::new(),
            stdout: String::new(),
            stderr: String::new(),
            status: 0,
            after: BTreeMap::new(),
        }
    }
    #[test]
    fn matcher_charges_transitions_and_yields_before_large_match_finishes() {
        let vm = Vm::new("", &fixture()).unwrap();
        let scalar = Scalar {
            parts: vec![].into_iter(),
            text: "*b".into(),
            active: vec![1, 1],
            pattern: true,
            waiting: None,
        };
        let mut matcher = Matcher::default();
        let subject = "a".repeat(10_000);
        let before = vm.expansion_work.get();
        assert_eq!(matcher.advance(&vm, &subject, &scalar).unwrap(), None);
        assert_eq!(vm.expansion_work.get() - before, 256);
        assert!(matcher.subject < subject.len());
        let mut result = None;
        for _ in 0..1000 {
            result = matcher.advance(&vm, &subject, &scalar).unwrap();
            if result.is_some() {
                break;
            }
        }
        assert_eq!(result, Some(false));
    }
    #[test]
    fn mixed_quote_mask_matches_bytes_without_filename_rules() {
        let vm = Vm::new("", &fixture()).unwrap();
        for (text, active, subject, expected) in [
            ("a**", vec![1, 0, 1], "a*rest", true),
            ("a**", vec![1, 0, 1], "axrest", false),
            ("*", vec![1], ".hidden/a\nb", true),
            ("?", vec![1], "é", false),
            ("??", vec![1, 1], "é", true),
            ("é", vec![0, 0], "é", true),
            ("***", vec![1, 1, 1], "", true),
            ("?", vec![0], "?", true),
            ("?", vec![0], "x", false),
        ] {
            let scalar = Scalar {
                parts: vec![].into_iter(),
                text: text.into(),
                active,
                pattern: true,
                waiting: None,
            };
            assert_eq!(
                Matcher::default().advance(&vm, subject, &scalar).unwrap(),
                Some(expected),
                "{text:?} / {subject:?}"
            );
        }
    }
    #[test]
    fn short_patterns_match_an_independent_exhaustive_dynamic_programming_oracle() {
        // Distinct algorithm: each table cell states whether a pattern prefix
        // matches a subject prefix. There is no greedy star/retry state here.
        fn oracle(pattern: &[(u8, bool)], subject: &[u8]) -> bool {
            assert!(pattern.len() <= 3 && subject.len() <= 3);
            let mut prefixes = [[false; 4]; 4];
            prefixes[0][0] = true;
            for (i, &(byte, active)) in pattern.iter().enumerate() {
                if active && byte == b'*' {
                    for j in 0..=subject.len() {
                        prefixes[i + 1][j] = prefixes[i][j] || (j > 0 && prefixes[i + 1][j - 1]);
                    }
                } else {
                    for j in 1..=subject.len() {
                        prefixes[i + 1][j] = prefixes[i][j - 1]
                            && ((active && byte == b'?') || byte == subject[j - 1]);
                    }
                }
            }
            prefixes[pattern.len()][subject.len()]
        }
        fn sequences<T: Copy>(alphabet: &[T]) -> Vec<Vec<T>> {
            let mut all = vec![vec![]];
            let mut level = vec![vec![]];
            for _ in 0..3 {
                let mut next = Vec::new();
                for prefix in &level {
                    for &symbol in alphabet {
                        let mut word = prefix.clone();
                        word.push(symbol);
                        next.push(word);
                    }
                }
                all.extend(next.iter().cloned());
                level = next;
            }
            all
        }
        // Every length0..3 sequence, including every quoted/active choice for
        // * and ?. Slash is subject data; literal wildcard subjects are included.
        let patterns = sequences(&[
            (b'a', false),
            (b'b', false),
            (b'*', true),
            (b'?', true),
            (b'*', false),
            (b'?', false),
        ]);
        let subjects = sequences(b"ab*?/");
        assert_eq!(patterns.len(), 259);
        assert_eq!(subjects.len(), 156);
        let vm = Vm::new("", &fixture()).unwrap();
        let mut comparisons = 0;
        for pattern in patterns {
            // Each isolated short-pattern group gets an independent VM work
            // counter; no case can mask a matcher bug behind aggregate rejection.
            vm.expansion_work.set(0);
            let scalar = Scalar {
                parts: vec![].into_iter(),
                text: String::from_utf8(pattern.iter().map(|&(byte, _)| byte).collect()).unwrap(),
                active: pattern
                    .iter()
                    .map(|&(_, active)| u8::from(active))
                    .collect(),
                pattern: true,
                waiting: None,
            };
            for subject in &subjects {
                let expected = oracle(&pattern, subject);
                let subject = std::str::from_utf8(subject).unwrap();
                assert_eq!(
                    Matcher::default().advance(&vm, subject, &scalar).unwrap(),
                    Some(expected),
                    "pattern {pattern:?}, subject {subject:?}"
                );
                comparisons += 1;
            }
        }
        assert_eq!(comparisons, 40_404);
    }
    #[test]
    fn stop_during_resumable_match_drops_all_pending_and_cannot_resume() {
        let mut fixture = fixture();
        fixture.env.insert("subject".into(), "a".repeat(50_000));
        let mut vm = Vm::new(
            "case $subject in *b) printf wrong;; *) printf fallback;; esac",
            &fixture,
        )
        .unwrap();
        vm.tick(1);
        vm.tick(1);
        assert!(
            matches!(vm.pending.as_ref(), Some(execution::Pending::Case(task)) if task.matcher.is_some())
        );
        assert!(vm.pending_bytes.get() >= 50_000);
        assert!(vm.pending_bytes.get() <= vm.memory_bytes());
        vm.stop();
        assert_eq!(vm.pending_bytes.get(), 0);
        assert!(vm.pending.is_none());
        let stopped = vm.outcome.clone();
        for _ in 0..100 {
            vm.tick(64);
        }
        assert_eq!(vm.outcome, stopped);
        assert!(vm.outcome.stdout.is_empty());
    }
    #[test]
    fn joined_preparation_exposes_exact_quote_provenance_without_sole_fields() {
        let mut fixture = fixture();
        fixture.env.insert("suffix".into(), "*?".into());
        let vm = Vm::new("", &fixture).unwrap();
        let word = Word(vec![
            Part {
                value: PartValue::Text("a*".into()),
                quote: Quote::Single,
            },
            Part {
                value: PartValue::Text("$suffix".into()),
                quote: Quote::Bare,
            },
        ]);
        let mut scalar = Scalar::new(&vm, &word, true).unwrap();
        while !scalar.advance(&vm).unwrap() {}
        assert_eq!(scalar.text, "a**?");
        assert_eq!(scalar.active, [0, 0, 1, 1]);
        let quoted = Word(vec![Part {
            value: PartValue::Text("$@".into()),
            quote: Quote::Double,
        }]);
        assert!(
            Scalar::new(&vm, &quoted, false)
                .unwrap()
                .advance(&vm)
                .is_err()
        );
    }
    #[test]
    fn finite_instruction_and_function_call_limits_are_shared_through_case_captures() {
        let fixture = fixture();
        let result = run("f() { case x in x) value=$(f);; esac; }; f", &fixture);
        assert_eq!(result.status, 2);
        assert!(
            result
                .error
                .as_deref()
                .is_some_and(|error| error.contains("depth") || error.contains("call"))
        );
        let result = run(
            "f() { if test \"$1\" -gt 0; then f \"$(($1-1))\"; else case x in x) value=$(g 20);; esac; fi; }; g() { if test \"$1\" -gt 0; then g \"$(($1-1))\"; else :; fi; }; f 20",
            &fixture,
        );
        assert!(
            result
                .error
                .as_deref()
                .is_some_and(|error| error.contains("Function call depth limit"))
        );
        let result = run("while true; do case x in x) :;; esac; done", &fixture);
        assert!(
            result
                .error
                .as_deref()
                .is_some_and(|error| error.contains("Execution limit"))
        );
    }
}
