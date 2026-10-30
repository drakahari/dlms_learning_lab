//! Conditional post-evaluator14 candidate: literal-only prefix/suffix removal.
//! No expansion, glob matching, reparsing, mutation or host lookup occurs here.
use super::parameters::Target;
use super::{name, readable_variable};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Edge {
    Prefix,
    Suffix,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Expansion<'a> {
    pub target: Target<'a>,
    literal: &'a str,
    edge: Edge,
}
impl Expansion<'_> {
    /// A successful literal match ends on a UTF-8 boundary because both operands
    /// are valid str. An unset scalar/positional parameter behaves as empty.
    pub(super) fn select<'a>(&'a self, value: Option<&'a str>) -> &'a str {
        let value = value.unwrap_or("");
        match self.edge {
            Edge::Prefix => value.strip_prefix(self.literal).unwrap_or(value),
            Edge::Suffix => value.strip_suffix(self.literal).unwrap_or(value),
        }
    }
}
/// Called only after the existing default/alternate parser returns None. This
/// prevents literal '#'/'%' in an existing fallback WORD from being reclassified.
pub(super) fn parse(key: &str) -> Result<Option<Expansion<'_>>, String> {
    let Some((at, delimiter)) = key.char_indices().find(|(_, c)| matches!(c, '#' | '%')) else {
        return Ok(None);
    };
    // Keep all existing parameter-count/array-count dispatch unchanged.
    if at == 0 {
        return Ok(None);
    }
    let (base, tail) = key.split_at(at);
    let literal = &tail[1..];
    if literal.starts_with(delimiter) {
        return Err("Unsupported longest parameter removal; this subset supports single # or % with a literal operand only".into());
    }
    let target = if name(base) {
        readable_variable(base)?;
        Target::Named(base)
    } else if base.len() == 1 && matches!(base.as_bytes()[0], b'1'..=b'9') {
        Target::Positional(usize::from(base.as_bytes()[0] - b'1'))
    } else {
        return Err("Unsupported literal-removal target; use an ordinary scalar or positional parameter1 through9".into());
    };
    if literal.starts_with('~')
        || literal.chars().any(|c| {
            c.is_control()
                || matches!(
                    c,
                    '*' | '?' | '[' | ']' | '$' | '\'' | '"' | '\\' | '`' | '{' | '}' | '(' | ')'
                )
        })
    {
        return Err("Unsupported removal operand: use literal text only; patterns, nested expansion, inner quotes, escapes, parentheses and control characters are excluded".into());
    }
    Ok(Some(Expansion {
        target,
        literal,
        edge: if delimiter == '#' {
            Edge::Prefix
        } else {
            Edge::Suffix
        },
    }))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn literal_edges_strip_once_and_preserve_absent_or_nonmatching_text() {
        for (key, input, expected) in [
            ("v#--output=", "--output=a=b", "a=b"),
            ("v%.log", "error.log.log", "error.log"),
            ("v#pre", "prepare", "pare"),
            ("v#missing", "kept", "kept"),
            ("v%.log", "file.txt", "file.txt"),
            ("v#", "kept", "kept"),
            ("v%", "kept", "kept"),
            ("v#--name=", "--name=", ""),
        ] {
            let x = parse(key).unwrap().unwrap();
            assert_eq!(x.select(Some(input)), expected);
            assert_eq!(x.select(None), "");
            assert_eq!(x.select(Some("")), "");
        }
    }
    #[test]
    fn borrowed_utf8_values_and_hostile_data_are_never_reparsed() {
        for (key, value, expected) in [
            ("v#é-", "é-$(not-code)*.log", "$(not-code)*.log"),
            ("v%.log", "-two words;literal?.log", "-two words;literal?"),
            ("v%.é", "name.é", "name"),
        ] {
            let x = parse(key).unwrap().unwrap();
            let actual = x.select(Some(value));
            assert_eq!(actual, expected);
            assert!(actual.as_ptr() >= value.as_ptr());
            assert!(
                actual.as_ptr() as usize + actual.len() <= value.as_ptr() as usize + value.len()
            );
        }
    }
    #[test]
    fn new_parser_preserves_existing_dispatch_and_parameter_target_boundaries() {
        for key in ["name", "?", "#", "#array[@]", "array[0]", "name:=x"] {
            assert_eq!(parse(key).unwrap(), None);
        }
        assert_eq!(
            parse("1#--").unwrap().unwrap().target,
            Target::Positional(0)
        );
        for key in [
            "0#x",
            "10%x",
            "array[0]#x",
            "IFS#x",
            "UID%x",
            "BASH_FORBIDDEN#x",
        ] {
            assert!(parse(key).is_err(), "{key}");
        }
    }
    #[test]
    fn unsupported_pattern_or_expansion_is_not_silently_literal_or_lazy() {
        for key in [
            "v##x",
            "v%%x",
            "v#*",
            "v%?",
            "v#[ab]",
            "v#~home",
            "v#$other",
            "v#$(printf x)",
            "v#'x'",
            "v%\"x\"",
            "v#x\\y",
            "v#x`y`",
            "v#x{y}",
            "v#x\ny",
        ] {
            assert!(parse(key).is_err(), "{key}");
        }
    }
    #[test]
    fn caller_must_preserve_old_fallback_words_with_hash_and_percent() {
        for key in ["v:-prefix#tail", "v+suffix%data", "v:-name%.log"] {
            assert!(super::super::parameters::parse(key).unwrap().is_some());
        }
    }
}

#[cfg(test)]
mod vm_ownership_tests {
    use super::super::{Case, MAX_TEXT, Value, Vm};
    use std::collections::BTreeMap;

    fn case() -> Case {
        Case {
            name: "Literal-removal ownership".into(),
            args: vec![],
            stdin: String::new(),
            env: BTreeMap::from([("value".into(), format!("pre{}", "v".repeat(512)))]),
            files: BTreeMap::from([("/work/keep.txt".into(), "retained\n".into())]),
            stdout: String::new(),
            stderr: String::new(),
            status: 0,
            after: BTreeMap::new(),
        }
    }

    #[test]
    fn borrowed_selection_retains_only_the_result_lease_and_refuses_before_oversized_append() {
        let c = case();
        let vm = Vm::new(":", &c).unwrap();
        let expression = "${value#pre}";
        let value = vm.expand_text(expression).unwrap();
        assert_eq!(value.text, "v".repeat(512));
        assert_eq!(vm.scratch_bytes.get(), value.text.len());
        assert_eq!(
            vm.regex_work.get(),
            0,
            "Literal removal cannot renew or consume the ERE ledger"
        );
        drop(value);
        assert_eq!(vm.scratch_bytes.get(), 0);

        // Leave the real character/key leases plus all but one result byte.
        // This forces admission failure at append, rather than allocating first.
        let characters = expression.chars().count() * std::mem::size_of::<char>();
        let key = "value#pre".len();
        let available = characters + key + 511;
        let held = vm
            .scratch(MAX_TEXT - vm.memory_bytes() - available)
            .unwrap();
        let retained = vm.scratch_bytes.get();
        let error = vm
            .expand_text(expression)
            .err()
            .expect("Result does not fit the existing ledger");
        assert!(error.contains("Virtual memory"), "{error}");
        assert_eq!(
            vm.scratch_bytes.get(),
            retained,
            "Rejected temporary leases are released"
        );
        assert_eq!(vm.outcome.files, c.files);
        assert_eq!(vm.regex_work.get(), 0);
        drop(held);
        assert_eq!(vm.scratch_bytes.get(), 0);
        assert_eq!(vm.expand_text(expression).unwrap().text, "v".repeat(512));
    }

    #[test]
    fn stop_follows_actual_trim_execution_and_preserves_the_existing_global_ere_ledger() {
        let c = case();
        let mut vm = Vm::new("while true; do result=\"${value#pre}\"; done", &c).unwrap();
        vm.charge_regex(17).unwrap();
        let mut reached = false;
        for _ in 0..200 {
            vm.tick(1);
            if vm
                .env
                .get("result")
                .is_some_and(|v| matches!(v, Value::Scalar(text) if text == &"v".repeat(512)))
            {
                reached = true;
                break;
            }
            assert!(
                !vm.done(),
                "Actual trim must execute before Stop: {:?}",
                vm.outcome
            );
        }
        assert!(
            reached,
            "A clock-only or never-executed Stop control is insufficient"
        );
        assert!(vm.expansion_work.get() > 512);
        assert_eq!(vm.regex_work.get(), 17);
        assert!(!vm.done());
        vm.stop();
        assert!(vm.done());
        assert!(
            vm.outcome
                .error
                .as_ref()
                .is_some_and(|e| e.starts_with("Stopped"))
        );
        assert_eq!(vm.outcome.files, c.files);
        assert_eq!(vm.regex_work.get(), 17);
        assert_eq!(vm.scratch_bytes.get(), 0);
        assert_eq!(vm.pending_bytes.get(), 0);
        let stopped = vm.outcome.clone();
        vm.tick(64);
        assert_eq!(vm.outcome, stopped);
    }
}
