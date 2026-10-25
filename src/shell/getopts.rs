//! A bounded getopts scanner, owned by the current execution context. No I/O.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Identity {
    declaration: String,
    arguments: Vec<String>,
}
impl Identity {
    fn bytes(&self) -> usize {
        self.declaration.len() + strings_bytes(&self.arguments)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Scanner {
    identity: Option<Identity>,
    index: usize,
    offset: usize,
}
impl Default for Scanner {
    fn default() -> Self {
        Self {
            identity: None,
            index: 1,
            offset: 0,
        }
    }
}
impl Scanner {
    #[cfg(test)]
    pub(super) fn active(&self) -> bool {
        self.identity.is_some()
    }
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.identity.as_ref().map_or(0, Identity::bytes)
    }
}

pub(super) fn local_target(key: &str, value: Option<&str>) -> Result<(), String> {
    if key == "OPTIND" {
        if value == Some("1") {
            Ok(())
        } else {
            Err("Unsupported local OPTIND; supply local OPTIND=1 inside a function".into())
        }
    } else {
        variable_target(key)
    }
}

struct Spec {
    options: [u8; 128], // 0 absent, 1 flag, 2 required argument
    silent: bool,
}
impl Spec {
    fn parse(text: &str) -> Result<Self, String> {
        let mut result = Self {
            options: [0; 128],
            silent: text.starts_with(':'),
        };
        let bytes = text.as_bytes();
        let mut i = usize::from(result.silent);
        while i < bytes.len() {
            let option = bytes[i];
            if !option.is_ascii_alphanumeric() || result.options[option as usize] != 0 {
                return Err("Unsupported getopts declaration; use unique ASCII letters/digits, optional required-argument colons, and optional initial colon".into());
            }
            i += 1;
            let needs = bytes.get(i) == Some(&b':');
            result.options[option as usize] = if needs { 2 } else { 1 };
            i += usize::from(needs);
        }
        Ok(result)
    }
}
struct Next<'a> {
    option: &'a str,
    argument: Option<&'a str>,
    index: usize,
    offset: usize,
    status: i32,
    problem: Option<(bool, u8)>, // missing argument rather than unknown option
}
fn step<'a>(scanner: &Scanner, spec: &Spec, arguments: &'a [String]) -> Result<Next<'a>, String> {
    let mut index = scanner.index;
    let mut offset = scanner.offset;
    let end = |index| Next {
        option: "?",
        argument: None,
        index,
        offset: 0,
        status: 1,
        problem: None,
    };
    let Some(word) = arguments.get(index - 1) else {
        return Ok(end(index));
    };
    if offset == 0 {
        if word == "--" {
            return Ok(end(index + 1));
        }
        if !word.starts_with('-') || word.len() == 1 {
            return Ok(end(index));
        }
        offset = 1;
    }
    let option = *word
        .as_bytes()
        .get(offset)
        .ok_or("Invalid getopts cluster offset")?;
    if !option.is_ascii() || option == 0 {
        return Err(
            "Unsupported non-ASCII option byte; option values and operands may contain UTF8".into(),
        );
    }
    let option_text = &word[offset..offset + 1];
    offset += 1;
    let kind = spec.options[option as usize];
    let mut result = Next {
        option: option_text,
        argument: None,
        index,
        offset,
        status: 0,
        problem: None,
    };
    if kind == 2 {
        if offset < word.len() {
            result.argument = Some(&word[offset..]);
            index += 1;
        } else if let Some(value) = arguments.get(index) {
            result.argument = Some(value);
            index += 2;
        } else {
            result.option = if spec.silent { ":" } else { "?" };
            result.argument = spec.silent.then_some(option_text);
            result.problem = Some((true, option));
            index += 1;
        }
        offset = 0;
    } else {
        if kind == 0 {
            result.option = "?";
            result.argument = spec.silent.then_some(option_text);
            result.problem = Some((false, option));
        }
        if offset == word.len() {
            index += 1;
            offset = 0;
        }
    }
    result.index = index;
    result.offset = offset;
    Ok(result)
}

struct Transition {
    identity: Option<Identity>,
    name: String,
    option: String,
    ind: String,
    argument: Option<String>,
    argument_key: Option<String>,
    diagnostic: String,
    index: usize,
    offset: usize,
    status: i32,
}
impl Vm {
    fn getopts_plan(&self, operands: &[String]) -> Result<Transition, String> {
        let [declaration, target, ..] = operands else {
            return Err(
                "Unsupported getopts arguments; use getopts OPTSTRING NAME [ARG...]".into(),
            );
        };
        variable_target(target)?;
        if !name(target) || target == "OPTERR" {
            return Err("Unsupported getopts output name; use an ordinary identifier, not a special-variable alias".into());
        }
        let errors = match self.env.get("OPTERR") {
            Some(Value::Scalar(value)) if value == "0" || value == "1" => value == "1",
            _ => return Err(
                "Unsupported OPTERR for getopts; use scalar OPTERR=0 or OPTERR=1 before parsing"
                    .into(),
            ),
        };
        self.charge_expansion(declaration.len())?;
        let spec = Spec::parse(declaration)?;
        let arguments = if operands.len() == 2 {
            self.args.as_slice()
        } else {
            &operands[2..]
        };
        let identity_bytes = declaration.len() + strings_bytes(arguments);
        // Charge all exact-identity comparisons before work; no fingerprint or
        // pointer identity can admit changed contents. Equal rebuilt lists work.
        self.charge_expansion(identity_bytes)?;
        if arguments.iter().any(|s| s.contains('\0')) {
            return Err("Unsupported NUL in getopts arguments".into());
        }
        if let Some(identity) = &self.scanner.identity {
            self.charge_expansion(identity_bytes)?;
            if identity.declaration != *declaration || identity.arguments.as_slice() != arguments {
                return Err("Unsupported changed getopts declaration or argument list; reset OPTIND=1 before a new scan".into());
            }
        }
        self.charge_expansion(1)?;
        let next = step(&self.scanner, &spec, arguments)?;
        let new_argument = next.argument.is_some();
        let argument_key = new_argument && !self.env.contains_key("OPTARG");
        // Aggregate names after the whole write set, including all hidden parent
        // names. Independent single-write preflights would admit overflow.
        let names: BTreeSet<&str> = self
            .env
            .keys()
            .map(String::as_str)
            .filter(|key| *key != "OPTARG" || new_argument)
            .chain(
                self.calls
                    .iter()
                    .flat_map(|c| c.locals.keys().map(String::as_str)),
            )
            .chain(
                self.frames
                    .iter()
                    .flat_map(|f| f.env.keys().map(String::as_str)),
            )
            .chain(self.frames.iter().flat_map(|f| {
                f.calls
                    .iter()
                    .flat_map(|c| c.locals.keys().map(String::as_str))
            }))
            .chain([target.as_str(), "OPTIND"])
            .chain(new_argument.then_some("OPTARG"))
            .collect();
        if !variable_names_within_limit(&names) {
            return Err(
                "Variable limit exceeded (256 ordinary names plus three fixed getopts names)"
                    .into(),
            );
        }
        let new_identity = self.scanner.identity.is_none();
        let empty_array_slot =
            matches!(self.env.get(target), Some(Value::Indexed(v)) if v.is_empty());
        // Preflight all simultaneous owned transaction copies while existing
        // command arguments and old values are still live. No allocations yet.
        let extra = std::mem::size_of::<Transition>()
            + target.len()
            + next.option.len()
            + decimal_bytes(next.index as i64)
            + next.argument.map_or(0, str::len)
            + usize::from(argument_key) * "OPTARG".len()
            + usize::from(empty_array_slot) * std::mem::size_of::<String>()
            + if new_identity { identity_bytes } else { 0 }
            + 96;
        let memory = self.scratch(extra)?;
        self.charge_expansion(extra)?;
        let diagnostic = if errors && !spec.silent {
            next.problem.map_or_else(String::new, |(missing, option)| {
                if missing {
                    format!(
                        "getopts: option requires an argument: {:?}\n",
                        char::from(option)
                    )
                } else {
                    format!("getopts: invalid option: {:?}\n", char::from(option))
                }
            })
        } else {
            String::new()
        };
        let transition = Transition {
            identity: new_identity.then(|| Identity {
                declaration: declaration.clone(),
                arguments: arguments.to_vec(),
            }),
            name: target.clone(),
            option: next.option.to_string(),
            ind: next.index.to_string(),
            argument: next.argument.map(str::to_string),
            argument_key: argument_key.then(|| "OPTARG".into()),
            diagnostic,
            index: next.index,
            offset: next.offset,
            status: next.status,
        };
        drop(memory);
        Ok(transition)
    }
    pub(super) fn getopts_builtin(
        &mut self,
        operands: &[String],
    ) -> Result<(String, String, i32), String> {
        let next = self.getopts_plan(operands)?;
        // All validation/aggregate reservation precedes publication. These
        // assignments move already allocated values; no fallible second scan.
        if let Some(identity) = next.identity {
            self.scanner.identity = Some(identity);
        }
        self.scanner.index = next.index;
        self.scanner.offset = next.offset;
        if let Some(value) = self.env.get_mut(&next.name) {
            value.assign_scalar(next.option);
        } else {
            self.env.insert(next.name, Value::Scalar(next.option));
        }
        *self
            .env
            .get_mut("OPTIND")
            .expect("initialized or scoped OPTIND") = Value::Scalar(next.ind);
        match next.argument {
            None => {
                self.env.remove("OPTARG");
            }
            Some(argument) => {
                if let Some(value) = self.env.get_mut("OPTARG") {
                    *value = Value::Scalar(argument);
                } else {
                    self.env.insert(
                        next.argument_key.expect("new OPTARG key"),
                        Value::Scalar(argument),
                    );
                }
            }
        }
        Ok((String::new(), next.diagnostic, next.status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn vm() -> Vm {
        Vm::new(
            "",
            &Case {
                name: "getopts private state".into(),
                args: vec![],
                stdin: String::new(),
                env: BTreeMap::new(),
                files: BTreeMap::new(),
                stdout: String::new(),
                stderr: String::new(),
                status: 0,
                after: BTreeMap::new(),
            },
        )
        .unwrap()
    }
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| (*s).into()).collect()
    }
    #[test]
    fn aggregate_name_write_set_rejects_before_any_scanner_or_variable_publication() {
        let mut vm = vm();
        for i in 0..256 {
            vm.env.insert(format!("v{i}"), Value::Scalar(String::new()));
        }
        let before = vm.scanner.clone();
        let error = vm
            .getopts_builtin(&args(&["o:", "choice", "-o", "value"]))
            .unwrap_err();
        assert!(error.contains("Variable limit"));
        assert_eq!(vm.scanner, before);
        assert_eq!(vm.scalar("OPTIND"), Some("1"));
        assert!(!vm.env.contains_key("choice") && !vm.env.contains_key("OPTARG"));
        assert_eq!(vm.env.len(), 258);
        vm.env.remove("v255");
        vm.getopts_builtin(&args(&["o:", "choice", "-o", "value"]))
            .unwrap();
        assert_eq!(vm.env.len(), 259);
        assert_eq!(vm.scalar("choice"), Some("o"));
        assert_eq!(vm.scalar("OPTARG"), Some("value"));
    }
    #[test]
    fn identity_is_exact_not_pointer_based_and_failure_keeps_prior_cursor() {
        let mut vm = vm();
        vm.getopts_builtin(&args(&["abc", "choice", "-abc"]))
            .unwrap();
        let prior = vm.scanner.clone();
        for changed in [
            args(&["abc", "other", "-abd"]),
            args(&["ab", "other", "-abc"]),
        ] {
            assert!(
                vm.getopts_builtin(&changed)
                    .unwrap_err()
                    .contains("changed getopts")
            );
            assert_eq!(vm.scanner, prior);
            assert!(!vm.env.contains_key("other"));
            assert_eq!(vm.scalar("choice"), Some("a"));
        }
        vm.getopts_builtin(&args(&["abc", "other", "-abc"]))
            .unwrap();
        assert_eq!(vm.scalar("other"), Some("b"));
        vm.assign_scalar("OPTIND".into(), "1".into()).unwrap();
        vm.getopts_builtin(&args(&["x:", "other", "-x", ""]))
            .unwrap();
        assert_eq!(vm.scalar("OPTARG"), Some(""));
    }
    #[test]
    fn aliases_and_memory_failure_publish_no_partial_transition() {
        let mut vm = vm();
        for alias in ["OPTIND", "OPTARG", "OPTERR", "IFS", "BASH_ENV", "x[0]"] {
            assert!(vm.getopts_builtin(&args(&["a", alias, "-a"])).is_err());
            assert!(vm.scanner.identity.is_none());
            assert_eq!(vm.scalar("OPTIND"), Some("1"));
        }
        let available = MAX_TEXT - vm.memory_bytes() - 64;
        vm.env
            .insert("pad".into(), Value::Scalar("p".repeat(available)));
        assert!(vm.memory_bytes() < MAX_TEXT);
        assert!(
            vm.getopts_builtin(&args(&["a", "choice", "-a"]))
                .unwrap_err()
                .contains("memory")
        );
        assert!(vm.scanner.identity.is_none());
        assert!(!vm.env.contains_key("choice") && !vm.env.contains_key("OPTARG"));
        assert_eq!(vm.scalar("OPTIND"), Some("1"));
        assert_eq!(vm.scalar("pad").unwrap().len(), available);
    }
    #[test]
    fn removing_optarg_does_not_free_an_ordinary_slot_or_partially_publish_a_failed_write_set() {
        let mut vm = vm();
        vm.getopts_builtin(&args(&["ao:", "choice", "-o", "value", "-a"]))
            .unwrap();
        for i in 0..255 {
            vm.env.insert(format!("v{i}"), Value::Scalar(String::new()));
        }
        assert_eq!(vm.env.len(), 259);
        let scanner = vm.scanner.clone();
        assert!(
            vm.getopts_builtin(&args(&["ao:", "replacement", "-o", "value", "-a"]))
                .unwrap_err()
                .contains("Variable limit")
        );
        assert_eq!(vm.scanner, scanner);
        assert_eq!(vm.scalar("choice"), Some("o"));
        assert_eq!(vm.scalar("OPTARG"), Some("value"));
        assert_eq!(vm.scalar("OPTIND"), Some("3"));
        assert!(!vm.env.contains_key("replacement"));
        vm.getopts_builtin(&args(&["ao:", "choice", "-o", "value", "-a"]))
            .unwrap();
        assert_eq!(vm.env.len(), 258);
        assert!(!vm.env.contains_key("OPTARG"));
        assert_eq!(vm.scalar("choice"), Some("a"));
        vm.limits().unwrap();
    }
}
