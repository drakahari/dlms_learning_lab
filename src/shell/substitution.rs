//! Original bounded lexical matching for the native dollar-parenthesis parser.
//! No execution or Bash source is used here. Only the existing supported grammar
//! is scanned; unsupported groups/heredocs/keywords remain parser errors.
use super::arithmetic;

pub(super) const MAX_DEPTH: usize = 8;
pub(super) const MAX_SITES: usize = 128;

/// Recognize an opener after removing intervening backslash-LF continuations.
/// The caller handles escaped dollar signs before asking about this position.
pub(super) fn body_start(chars: &[char], dollar: usize) -> Option<usize> {
    if chars.get(dollar) != Some(&'$') {
        return None;
    }
    let mut open = dollar + 1;
    while chars.get(open..)?.starts_with(&['\\', '\n']) {
        open += 2;
    }
    (chars.get(open) == Some(&'(') && chars.get(open + 1) != Some(&'(')).then_some(open + 1)
}

/// An arithmetic span remains atomic to the shell lexer. Command substitution
/// inside arithmetic is deliberately excluded; nested arithmetic is unchanged.
pub(super) fn arithmetic_end(chars: &[char], start: usize) -> Result<usize, String> {
    // Parentheses in source still participate in arithmetic lexical matching,
    // even in an unselected parameter fallback. Never make a braced WORD opaque.
    let end = arithmetic::end(chars, start)?;
    let expression_end = end - 2;
    let expression = &chars[..expression_end];
    let mut i = start + 3;
    while i < expression_end {
        if expression[i..].starts_with(&['$', '(', '(']) {
            i = arithmetic_end(expression, i)?;
            continue;
        }
        if expression[i..].starts_with(&['$', '{']) {
            // A brace after this arithmetic close belongs to a different span.
            // Reject it while compiling, before any previous command can run.
            i = parameter_end(expression, i)?;
            continue;
        }
        if body_start(expression, i).is_some() {
            return Err("Unsupported command substitution inside arithmetic expansion".into());
        }
        i += 1;
    }
    Ok(end)
}

/// Skip a parameter span. The read-only default/alternate extension validates
/// its literal WORD while compiling, before any commands or redirects run.
pub(super) fn parameter_end(chars: &[char], start: usize) -> Result<usize, String> {
    let mut i = start + 2;
    while i < chars.len() {
        if chars[i] == '}' {
            let key = chars[start + 2..i].iter().collect::<String>();
            super::parameters::parse(&key)?;
            return Ok(i + 1);
        }
        if body_start(chars, i).is_some() {
            return Err(
                "Unsupported command substitution within a parameter or array index".into(),
            );
        }
        i += 1;
    }
    Err("Close ${parameter} before running".into())
}

/// A lexical case frame distinguishes a pattern-closing `)` from the capture's
/// closing `)`. This scanner locates spans only; Parser validates actual grammar.
#[derive(Clone, Copy)]
enum CasePhase {
    Selector,
    In,
    Pattern(bool),
    Body,
}
struct Delimiter {
    cases: Vec<CasePhase>,
    command: bool,
    word: String,
    bare: bool,
    started: bool,
    redirect: bool,
    array_depth: Option<usize>,
}
impl Delimiter {
    fn new() -> Self {
        Self {
            cases: Vec::new(),
            command: true,
            word: String::new(),
            bare: true,
            started: false,
            redirect: false,
            array_depth: None,
        }
    }
    fn opaque(&mut self) {
        self.started = true;
        self.bare = false;
    }
    fn word(&mut self) -> Result<(), String> {
        if !self.started {
            return Ok(());
        }
        // Array elements are data even when their bare spelling is a keyword.
        if self.array_depth.is_some() {
            self.word.clear();
            self.started = false;
            self.bare = true;
            return Ok(());
        }
        if std::mem::take(&mut self.redirect) {
            self.word.clear();
            self.started = false;
            self.bare = true;
            return Ok(());
        }
        let text = if self.bare { self.word.as_str() } else { "" };
        match self.cases.last_mut() {
            Some(phase @ CasePhase::Selector) => *phase = CasePhase::In,
            Some(phase @ CasePhase::In) if text == "in" => *phase = CasePhase::Pattern(true),
            Some(CasePhase::In) => {} // invalid grammar is diagnosed by Parser
            Some(CasePhase::Pattern(first)) => {
                if *first && text == "esac" {
                    self.cases.pop();
                    self.command = false;
                } else {
                    *first = false;
                }
            }
            _ => {
                if self.command && text == "case" {
                    if self.cases.len() == 24 {
                        return Err("Syntax nesting exceeds 24 levels".into());
                    }
                    self.cases.push(CasePhase::Selector);
                    self.command = false;
                } else if self.command
                    && text == "esac"
                    && matches!(self.cases.last(), Some(CasePhase::Body))
                {
                    self.cases.pop();
                    self.command = false;
                } else {
                    self.command = self.command
                        && matches!(text, "if" | "while" | "then" | "do" | "else" | "{");
                }
            }
        }
        self.word.clear();
        self.started = false;
        self.bare = true;
        Ok(())
    }
}

/// Return the character position following the matching close parenthesis.
/// Child source has its own quote domain even inside an outer double quote.
pub(super) fn end(chars: &[char], start: usize, depth: usize) -> Result<usize, String> {
    if depth > MAX_DEPTH {
        return Err("Command substitution depth limit exceeded (8)".into());
    }
    let mut i = body_start(chars, start).ok_or("Invalid command substitution opener")?;
    let mut parens = 1usize;
    let mut quote = None;
    let mut scan = Delimiter::new();
    while i < chars.len() {
        let c = chars[i];
        if quote == Some('\'') {
            if c == '\'' {
                quote = None;
            }
            i += 1;
            continue;
        }
        if c == '\\' {
            let next = chars
                .get(i + 1)
                .ok_or("Unfinished escape in command substitution")?;
            if quote.is_none() || matches!(next, '$' | '`' | '"' | '\\' | '\n') {
                if *next != '\n' {
                    scan.opaque();
                }
                i += 2;
                continue;
            }
        }
        if quote == Some('"') && c == '"' {
            quote = None;
            i += 1;
            continue;
        }
        if quote.is_none() {
            if c == '#' && !scan.started {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
            if matches!(c, '\'' | '"') {
                quote = Some(c);
                scan.opaque();
                i += 1;
                continue;
            }
        }
        if chars[i..].starts_with(&['$', '(', '(']) {
            i = arithmetic_end(chars, i)?;
            scan.opaque();
            continue;
        }
        if body_start(chars, i).is_some() {
            i = end(chars, i, depth + 1)?;
            scan.opaque();
            continue;
        }
        if chars[i..].starts_with(&['$', '{']) {
            i = parameter_end(chars, i)?;
            scan.opaque();
            continue;
        }
        if quote.is_some() {
            i += 1;
            continue;
        }
        if c.is_ascii_whitespace() {
            scan.word()?;
            if c == '\n'
                && scan.array_depth.is_none()
                && !matches!(
                    scan.cases.last(),
                    Some(CasePhase::Selector | CasePhase::In | CasePhase::Pattern(_))
                )
            {
                scan.command = true;
            }
            i += 1;
            continue;
        }
        if matches!(c, '(' | ')' | ';' | '|' | '&' | '<' | '>') {
            // Same lexical boundary as the supported dense array grammar:
            // one literal name=( at command position, with no intervening space.
            let array_open = c == '('
                && scan.command
                && scan.started
                && scan.bare
                && !scan.redirect
                && scan.array_depth.is_none()
                && !matches!(
                    scan.cases.last(),
                    Some(CasePhase::Selector | CasePhase::In | CasePhase::Pattern(_))
                )
                && scan.word.strip_suffix('=').is_some_and(super::name);
            scan.word()?;
            if let Some(array_depth) = scan.array_depth {
                if c == '(' {
                    parens += 1;
                    if parens > 24 {
                        return Err("Syntax nesting exceeds 24 levels".into());
                    }
                } else if c == ')' {
                    if parens == array_depth {
                        scan.array_depth = None;
                    }
                    parens -= 1;
                }
                // Unsupported raw operators/nested parentheses still go to the
                // real Parser, but cannot turn array data into a case frame.
                scan.command = false;
                i += 1;
                continue;
            }
            if array_open {
                parens += 1;
                if parens > 24 {
                    return Err("Syntax nesting exceeds 24 levels".into());
                }
                scan.array_depth = Some(parens);
                scan.command = false;
                i += 1;
                continue;
            }
            if matches!(c, '<' | '>') {
                scan.redirect = true;
            }
            if c == '(' && matches!(scan.cases.last(), Some(CasePhase::Pattern(_))) {
                // Optional opening parenthesis prevents bare esac from closing this arm.
                *scan.cases.last_mut().unwrap() = CasePhase::Pattern(false);
            } else if c == ')' && matches!(scan.cases.last(), Some(CasePhase::Pattern(_))) {
                *scan.cases.last_mut().unwrap() = CasePhase::Body;
                scan.command = true;
            } else if c == '(' {
                parens += 1;
                if parens > 24 {
                    return Err("Syntax nesting exceeds 24 levels".into());
                }
                scan.command = true;
            } else if c == ')' {
                parens -= 1;
                if parens == 0 {
                    return Ok(i + 1);
                }
                scan.command = true;
            } else if c == ';'
                && chars.get(i + 1) == Some(&';')
                && matches!(scan.cases.last(), Some(CasePhase::Body))
            {
                *scan.cases.last_mut().unwrap() = CasePhase::Pattern(true);
                i += 1;
                scan.command = false;
            } else if matches!(c, ';' | '|' | '&')
                && !matches!(scan.cases.last(), Some(CasePhase::Pattern(_)))
            {
                scan.command = true;
            }
            i += 1;
            continue;
        }
        scan.started = true;
        scan.word.push(c);
        i += 1;
    }
    Err("Close command substitution with ) before running".into())
}

#[cfg(test)]
mod tests {
    use super::super::{Code, PartValue, Program, Quote, Word, compile};
    use std::ops::Range;

    fn words(code: &Code) -> Vec<&Word> {
        match code {
            Code::Pipe(commands) => commands
                .iter()
                .flat_map(|c| {
                    c.words
                        .iter()
                        .chain(c.redirects.iter().filter_map(|(_, w)| w.as_ref()))
                })
                .collect(),
            Code::ArrayAssign(_, words) | Code::For(_, words, _) => words.iter().collect(),
            Code::WhileStart(Some(word), _) => vec![word],
            Code::Case(definition) => std::iter::once(&definition.selector)
                .chain(definition.arms.iter().flat_map(|arm| &arm.patterns))
                .collect(),
            _ => vec![],
        }
    }
    fn targets_stay_in_range(program: &Program, range: Range<usize>) {
        for code in &program.code[range.clone()] {
            let target = match code {
                Code::Jump(pc)
                | Code::JumpIf(_, pc)
                | Code::For(_, _, pc)
                | Code::Next(pc)
                | Code::WhileStart(_, pc) => Some(pc),
                Code::Case(definition) => {
                    assert!((range.start..=range.end).contains(&definition.end));
                    for arm in &definition.arms {
                        assert!(range.contains(&arm.entry));
                    }
                    None
                }
                Code::Define(_, entry, end) => {
                    assert!(range.contains(entry));
                    assert!(entry < end);
                    Some(end)
                }
                _ => None,
            };
            if let Some(pc) = target {
                assert!(
                    (range.start..=range.end).contains(pc),
                    "target {pc} outside {range:?}"
                );
            }
        }
    }
    #[test]
    fn nested_quote_domains_create_typed_nodes_in_one_immutable_arena() {
        let p = compile(r#"value="$(printf '%s' "$(printf 'a)b(c')")""#).unwrap();
        assert_eq!(p.bodies.len(), 2);
        let ids: Vec<_> = p
            .code
            .iter()
            .flat_map(words)
            .flat_map(|w| &w.0)
            .filter_map(|part| match part.value {
                PartValue::Substitution(id) => {
                    assert_eq!(part.quote, Quote::Double);
                    Some(id.0)
                }
                _ => None,
            })
            .collect();
        assert_eq!(ids, [1, 0]);
        assert!(p.main.end <= p.bodies[0].start && p.bodies[0].end <= p.bodies[1].start);
        assert!(ids.iter().all(|id| *id < p.bodies.len()));
        for source in [r#"printf '%s' '$(false)'"#, r#"printf '%s' "\$(false)""#] {
            assert!(compile(source).unwrap().bodies.is_empty());
        }
    }
    #[test]
    fn parent_and_child_function_and_control_targets_keep_their_own_ranges() {
        let p = compile("parent() { printf parent; }; value=$(child() { if true; then for item in a b; do :; done; else :; fi; }; while false; do :; done; child); printf after").unwrap();
        assert_eq!(p.bodies.len(), 1);
        targets_stay_in_range(&p, p.main.clone());
        targets_stay_in_range(&p, p.bodies[0].clone());
        let definitions: Vec<_> = p
            .code
            .iter()
            .filter_map(|c| match c {
                Code::Define(name, entry, _) => Some((name.as_str(), *entry)),
                _ => None,
            })
            .collect();
        assert_eq!(definitions.len(), 2);
        assert!(p.main.contains(&definitions[0].1));
        assert!(p.bodies[0].contains(&definitions[1].1));
    }
    #[test]
    fn case_descriptors_and_nested_captures_relocate_with_function_and_main_boundaries() {
        let p = compile("f() { case x in x) printf parent;; esac; }; result=$(g() { case \"$(printf x)\" in \"$(printf x)\") case y in y) printf child;; esac;; esac; }; g); printf main").unwrap();
        targets_stay_in_range(&p, p.main.clone());
        for range in &p.bodies {
            targets_stay_in_range(&p, range.clone());
        }
        let definitions: Vec<_> = p
            .code
            .iter()
            .filter_map(|code| {
                if let Code::Case(definition) = code {
                    Some(definition)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(definitions.len(), 3);
        assert!(
            definitions
                .iter()
                .any(|definition| p.main.contains(&definition.arms[0].entry))
        );
        assert!(definitions.iter().any(|definition| {
            p.bodies
                .iter()
                .any(|range| range.contains(&definition.arms[0].entry))
        }));
    }
    #[test]
    fn main_end_never_falls_into_an_appended_child_body() {
        use super::super::{Case, Vm};
        use std::collections::BTreeMap;
        let fixture = Case {
            name: "Arena boundary".into(),
            args: vec![],
            stdin: String::new(),
            env: BTreeMap::new(),
            files: BTreeMap::new(),
            stdout: String::new(),
            stderr: String::new(),
            status: 0,
            after: BTreeMap::new(),
        };
        let mut vm = Vm::new("", &fixture).unwrap();
        // Private test injection audits the main boundary only. The function
        // containing a typed substitution is never called; no capture executes.
        vm.program = compile("unused() { value=$(printf child); }; printf main").unwrap();
        vm.range = vm.program.main.clone();
        vm.program_bytes = vm.program.text_bytes();
        assert_eq!(vm.program.bodies.len(), 1);
        for _ in 0..100 {
            vm.tick(1);
            if vm.done() {
                break;
            }
        }
        assert!(vm.done());
        assert_eq!(vm.outcome.stdout, "main");
        assert_eq!(vm.outcome.error, None);
        assert_eq!(vm.pc, vm.program.main.end);
    }
    #[test]
    fn comments_escapes_arrays_functions_and_arithmetic_do_not_close_capture_early() {
        for source in [
            "value=$(# ) ignored\nprintf value\n)",
            r#"value="$(printf '%s' a\)b)""#,
            r#"value="$(render() { parts=(a 'b)c'); printf '%s' "${parts[0]}"; }; render)""#,
            r#"value="$(printf '%s' "$((1 + (2 * 3)))")""#,
        ] {
            assert_eq!(compile(source).unwrap().bodies.len(), 1, "{source}");
        }
    }
    #[test]
    fn continued_openers_keep_their_quote_domain_and_empty_bodies_are_bounded() {
        for source in [
            "value=$\\\n(printf x)",
            "value=\"$\\\n(printf '%s' \"$\\\n(printf ')')\")\"",
        ] {
            let program = compile(source).unwrap();
            assert!(!program.bodies.is_empty());
        }
        assert_eq!(
            compile("value=$()$(# only a comment\n)")
                .unwrap()
                .bodies
                .len(),
            2
        );
        assert!(
            compile("printf '%s' \"\\$\\\n(printf x)\"")
                .unwrap()
                .bodies
                .is_empty()
        );
    }
    #[test]
    fn input_only_shorthand_is_excluded_by_ast_not_spelling_or_a_second_redirect() {
        for body in [
            "< source.txt",
            "< source.txt;",
            "< source.txt\n",
            "< source.txt # ) ignored\n",
        ] {
            let error = compile(&format!("value=$({body})")).unwrap_err();
            assert!(
                error.contains("Unsupported Bash input-only"),
                "{body}: {error}"
            );
        }
        for body in [
            "< source.txt 2> error.txt",
            "< source.txt < other.txt",
            "< source.txt; :",
        ] {
            assert_eq!(
                compile(&format!("value=$({body})")).unwrap().bodies.len(),
                1
            );
        }
    }
    #[test]
    fn excluded_arithmetic_parameter_and_unbalanced_forms_fail_at_compilation() {
        for source in [
            "value=$((1 + $(printf 2)))",
            r#"value="${items[$(printf 0)]}""#,
            r#"value="${label:-$(printf x)}""#,
        ] {
            assert!(compile(source).unwrap_err().contains("Unsupported"));
        }
        for source in [
            "value=$(printf x",
            "value=$(printf 'x)",
            "value=$(printf x))",
        ] {
            assert!(compile(source).is_err());
        }
    }
    fn nested(n: usize) -> String {
        let mut source = "printf value".to_string();
        for _ in 0..n {
            source = format!("printf '%s' \"$({source})\"");
        }
        source
    }
    #[test]
    fn substitution_depth_and_sites_have_exact_bounded_edges() {
        assert_eq!(compile(&nested(8)).unwrap().bodies.len(), 8);
        assert!(compile(&nested(9)).unwrap_err().contains("depth limit"));
        assert_eq!(
            compile(&format!("value=\"{}\"", "$()".repeat(128)))
                .unwrap()
                .bodies
                .len(),
            128
        );
        assert!(
            compile(&format!("value=\"{}\"", "$()".repeat(129)))
                .unwrap_err()
                .contains("site limit")
        );
    }
    #[test]
    fn syntax_depth_includes_parent_compounds_and_child_bodies() {
        for (depth, accepted) in [(22, true), (23, false)] {
            let source = format!(
                "{}value=$(printf x); {}",
                "if true; then ".repeat(depth),
                "fi; ".repeat(depth)
            );
            let result = compile(&source);
            assert_eq!(result.is_ok(), accepted, "{depth}: {result:?}");
        }
    }
    #[test]
    fn tokens_are_aggregated_before_allocation_across_root_and_child_bodies() {
        assert!(compile(&format!("value=$({});", ":;".repeat(2047))).is_ok());
        assert!(
            compile(&format!("value=$({});", ":;".repeat(2048)))
                .unwrap_err()
                .contains("4096 tokens")
        );
        assert!(
            compile(&format!(
                "value=$({})$({})",
                ":;".repeat(1200),
                ":;".repeat(1200)
            ))
            .unwrap_err()
            .contains("4096 tokens")
        );
    }
}
