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
    let end = arithmetic::end(chars, start)?;
    for i in start + 3..end {
        if body_start(chars, i).is_some() {
            return Err("Unsupported command substitution inside arithmetic expansion".into());
        }
    }
    Ok(end)
}

/// Skip the existing simple parameter span without admitting a new parameter
/// operator or computed-index execution context by accident.
pub(super) fn parameter_end(chars: &[char], start: usize) -> Result<usize, String> {
    let mut i = start + 2;
    while i < chars.len() {
        if chars[i] == '}' {
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

/// Return the character position following the matching close parenthesis.
/// Child source has its own quote domain even inside an outer double quote.
pub(super) fn end(chars: &[char], start: usize, depth: usize) -> Result<usize, String> {
    if depth > MAX_DEPTH {
        return Err("Command substitution depth limit exceeded (8)".into());
    }
    let mut i = body_start(chars, start).ok_or("Invalid command substitution opener")?;
    let mut parens = 1usize;
    let mut quote = None;
    let mut word_start = true;
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
            let Some(next) = chars.get(i + 1) else {
                return Err("Unfinished escape in command substitution".into());
            };
            if quote.is_none() || matches!(next, '$' | '`' | '"' | '\\' | '\n') {
                if *next != '\n' {
                    word_start = false;
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
            if c == '#' && word_start {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
            if matches!(c, '\'' | '"') {
                quote = Some(c);
                word_start = false;
                i += 1;
                continue;
            }
        }
        if chars[i..].starts_with(&['$', '(', '(']) {
            i = arithmetic_end(chars, i)?;
            word_start = false;
            continue;
        }
        if body_start(chars, i).is_some() {
            i = end(chars, i, depth + 1)?;
            word_start = false;
            continue;
        }
        if chars[i..].starts_with(&['$', '{']) {
            i = parameter_end(chars, i)?;
            word_start = false;
            continue;
        }
        if quote.is_none() {
            if c == '(' {
                parens += 1;
                if parens > 24 {
                    return Err("Syntax nesting exceeds 24 levels".into());
                }
            } else if c == ')' {
                parens -= 1;
                if parens == 0 {
                    return Ok(i + 1);
                }
            }
            word_start =
                c.is_ascii_whitespace() || matches!(c, ';' | '|' | '&' | '<' | '>' | '(' | ')');
        }
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
        assert_eq!(vm.program.bodies.len(), 1);
        for _ in 0..10 {
            vm.tick(1);
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
