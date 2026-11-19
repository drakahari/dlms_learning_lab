//! Ordinary scalar export attributes. No host environment or process access.
//! Environment activities use fixed C/POSIX behavior; locale-control names are
//! outside this subset, without changing ordinary script locale handling.
use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentVariable {
    pub value: Option<String>,
    pub exported: bool,
}

/// Actual completed VM state; not an Outcome/Record migration or grading token.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentSnapshot {
    pub parent: BTreeMap<String, EnvironmentVariable>,
    pub child: BTreeMap<String, String>,
}

struct Target {
    key: String,
    assignment: bool,
}
pub(super) struct ExportPlan {
    pub remove: bool,
    targets: Vec<Target>,
}
fn locale_control(key: &str) -> bool {
    matches!(key, "LANG" | "LANGUAGE") || key.starts_with("LC_")
}
fn locale_limitation() -> String {
    "Unsupported environment locale controls: this activity fixes locale to C/POSIX; LANG, LANGUAGE and LC_* are outside the export/observation subset".into()
}
fn ordinary_target(key: &str) -> Result<(), String> {
    if locale_control(key) {
        return Err(locale_limitation());
    }
    if !name(key) || !ordinary_variable_name(key) || variable_target(key).is_err() {
        return Err("Unsupported export target: use an ordinary scalar variable name; special names, arrays and attributes are outside this subset".into());
    }
    Ok(())
}
fn target(word: &Word, vm: &Vm) -> Result<Target, String> {
    let _temporary = vm.scratch(word_bytes(word))?;
    let mut key = String::new();
    for part in &word.0 {
        let text = part
            .text()
            .ok_or("Unsupported command substitution in export")?;
        for c in text.chars() {
            if c == '=' {
                ordinary_target(&key)?;
                if !word.assignment() {
                    return Err("Unsupported export assignment spelling: use literal NAME=value, with quotes around the value".into());
                }
                return Ok(Target {
                    key,
                    assignment: true,
                });
            }
            if c == '$' {
                return Err(
                    "Unsupported computed export name/operand: write a literal NAME or NAME=value"
                        .into(),
                );
            }
            key.push(c);
        }
    }
    ordinary_target(&key)?;
    Ok(Target {
        key,
        assignment: false,
    })
}
impl ExportPlan {
    pub fn new(words: &[Word], vm: &Vm) -> Result<(Self, Vec<(Word, bool)>), String> {
        let remove = words.first().and_then(Word::literal) == Some("-n");
        let words = &words[usize::from(remove)..];
        if words.is_empty() {
            return Err("Unsupported export listing/options: supply NAME or NAME=value; Bash export -n accepts names only".into());
        }
        let mut memory = vm.scratch(0)?;
        let mut targets = Vec::new();
        for word in words {
            let t = target(word, vm)?;
            if remove && t.assignment {
                return Err("Unsupported export -n assignment; supply literal names only".into());
            }
            memory.grow(vm, t.key.len() + std::mem::size_of::<Target>())?;
            targets.push(t);
        }
        memory.grow(vm, words.iter().map(word_bytes).sum())?;
        let prepared = words.iter().cloned().map(|w| (w, false)).collect();
        Ok((Self { remove, targets }, prepared))
    }
    pub fn bytes(&self) -> usize {
        self.targets
            .iter()
            .map(|t| t.key.len() + std::mem::size_of::<Target>())
            .sum()
    }
    pub fn declarations(
        &self,
        values: Vec<String>,
        vm: &Vm,
    ) -> Result<Vec<LocalDeclaration>, String> {
        if values.len() != self.targets.len() {
            return Err("Unsupported expanded export operand count".into());
        }
        let original = strings_bytes(&values);
        let mut memory = vm.scratch(0)?;
        let mut out = Vec::new();
        for (value, target) in values.into_iter().zip(&self.targets) {
            let (key, assignment) = if target.assignment {
                let (key, value) = value
                    .split_once('=')
                    .ok_or("Invalid prepared export assignment")?;
                (key, Some(value))
            } else {
                (value.as_str(), None)
            };
            if key != target.key {
                return Err("Unsupported computed export target".into());
            }
            memory.grow(
                vm,
                key.len()
                    + assignment.map_or(0, str::len)
                    + std::mem::size_of::<LocalDeclaration>(),
            )?;
            out.push((key.to_owned(), assignment.map(str::to_owned)));
        }
        let retained = out
            .iter()
            .map(|(key, value)| {
                key.len()
                    + value.as_ref().map_or(0, String::len)
                    + std::mem::size_of::<LocalDeclaration>()
            })
            .sum();
        drop(memory);
        vm.rebase_pending(original, retained)?;
        Ok(out)
    }
}
impl Vm {
    fn environment_shape(&self) -> Result<(), String> {
        if self
            .env
            .values()
            .any(|v| matches!(v, Value::Associative(_)))
        {
            return Err("Unsupported associative maps with export/environment observation".into());
        }
        if self
            .env
            .keys()
            .chain(self.exported.iter())
            .any(|n| locale_control(n))
        {
            return Err(locale_limitation());
        }
        let supported = self.program.bodies.is_empty()
            && self.program.code.iter().all(|code| match code {
                Code::Jump(_) | Code::JumpIf(_, _) | Code::Status(_) => true,
                Code::Pipe(commands) => {
                    commands.len() == 1
                        && commands.iter().all(|c| {
                            c.words.iter().all(Word::assignment)
                                || !c.words.first().is_some_and(Word::assignment)
                        })
                }
                _ => false,
            });
        if !supported {
            return Err("Unsupported environment program shape: use global scalar assignments, simple commands and if/else or &&/||. Functions, pipelines, loops, case, arrays, command substitution and prefixed commands need broader environment scope support; ordinary non-environment programs remain available.".into());
        }
        if self.xtrace || self.environment_trace_seen {
            return Err("Unsupported export/environment observation with xtrace; remove the enabling set -x or set -o xtrace and rerun".into());
        }
        Ok(())
    }
    pub(super) fn begin_export(&mut self) -> Result<(), String> {
        self.environment_shape()?;
        self.environment_active = true;
        Ok(())
    }
    pub(super) fn apply_exports(
        &mut self,
        declarations: Vec<LocalDeclaration>,
        remove: bool,
    ) -> Result<(), String> {
        let mut names = self.live_variable_names();
        for (key, _) in &declarations {
            ordinary_target(key)?;
            // Removing an absent attribute neither declares nor allocates a name.
            if !remove {
                names.insert(key.as_str());
            }
        }
        if !variable_names_within_limit(&names) {
            return Err(
                "Variable limit exceeded (256 ordinary names including unset export attributes)"
                    .into(),
            );
        }
        drop(names);
        for (key, value) in declarations {
            // IntoIter retains the declaration Vec allocation until completion.
            // Release moved payload only; the pending owner reconciles slots.
            let retained = key.len() + value.as_ref().map_or(0, String::len);
            if remove {
                self.exported.remove(&key);
            } else if !self.exported.contains(&key) {
                self.reserve_memory(key.len() + std::mem::size_of::<String>())?;
                self.exported.insert(key.clone());
            }
            self.release_pending(retained)?;
            if let Some(value) = value {
                self.assign_scalar(key, value)?;
            }
            self.limits()?;
        }
        Ok(())
    }
    /// Bounded read-only evidence from an error-free finished run. An exported
    /// unset name is visible in parent state but contributes nothing to child.
    pub fn environment_snapshot(&self) -> Result<EnvironmentSnapshot, String> {
        if !self.done || self.outcome.error.is_some() {
            return Err(
                "Environment is unchecked: finish an error-free run before observing its state"
                    .into(),
            );
        }
        self.environment_shape()?;
        let names: BTreeSet<&str> = self
            .env
            .keys()
            .map(String::as_str)
            .chain(self.exported.iter().map(String::as_str))
            .filter(|n| ordinary_variable_name(n))
            .collect();
        if !variable_names_within_limit(&names) {
            return Err("Environment observation name limit exceeded".into());
        }
        let mut bytes = 0usize;
        for key in &names {
            let value = match self.env.get(*key) {
                Some(Value::Scalar(value)) => Some(value),
                Some(Value::Associative(_)) => {
                    return Err("Unsupported associative map environment observation".into());
                }
                Some(Value::Indexed(_)) => {
                    return Err("Unsupported indexed array environment observation".into());
                }
                None => None,
            };
            let size = key.len()
                + value.map_or(0, String::len)
                + std::mem::size_of::<EnvironmentVariable>();
            bytes = bytes
                .checked_add(size)
                .ok_or("Environment observation limit exceeded")?;
            if self.exported.contains(*key) && value.is_some() {
                bytes = bytes
                    .checked_add(key.len() + value.map_or(0, String::len))
                    .ok_or("Environment observation limit exceeded")?;
            }
        }
        self.reserve_memory(bytes)?;
        let mut parent = BTreeMap::new();
        let mut child = BTreeMap::new();
        for key in names {
            let value = self.env.get(key).map(|v| v.scalar().to_owned());
            let exported = self.exported.contains(key);
            if exported && let Some(value) = &value {
                child.insert(key.to_owned(), value.clone());
            }
            parent.insert(key.to_owned(), EnvironmentVariable { value, exported });
        }
        Ok(EnvironmentSnapshot { parent, child })
    }
}

#[cfg(test)]
mod memory_tests {
    use super::*;
    #[test]
    fn export_payload_transfer_keeps_declaration_slots_charged_until_owner_reconciles() {
        let c = Case {
            name: "Export ownership".into(),
            args: vec![],
            stdin: String::new(),
            env: BTreeMap::new(),
            files: BTreeMap::new(),
            stdout: String::new(),
            stderr: String::new(),
            status: 0,
            after: BTreeMap::new(),
        };
        let mut vm = Vm::new("export A B=value", &c).unwrap();
        let declarations = vec![
            ("A".to_owned(), None),
            ("B".to_owned(), Some("value".to_owned())),
        ];
        let slots = declarations.len() * std::mem::size_of::<LocalDeclaration>();
        let payload = declarations
            .iter()
            .map(|(key, value)| key.len() + value.as_ref().map_or(0, String::len))
            .sum::<usize>();
        vm.rebase_pending(0, payload + slots).unwrap();
        vm.begin_export().unwrap();
        vm.apply_exports(declarations, false).unwrap();
        assert_eq!(
            vm.pending_bytes.get(),
            slots,
            "Do not undercharge a still-owned declaration allocation"
        );
        assert!(vm.exported.contains("A"));
        assert_eq!(vm.env["B"].scalar(), "value");
        vm.rebase_pending(slots, 0).unwrap();
        assert_eq!(vm.pending_bytes.get(), 0);
        vm.limits().unwrap();
    }
}
