//! Tick-owned command/pipeline continuations. A suspended preparation must never
//! execute a command or expand its head again when it resumes.
use super::*;
use preparation::Words;

enum CommandPhase {
    Begin,
    Arguments(Words),
    Locals(Words),
    Assignment(String, Words),
    Redirect,
    Target(String, Words),
    Execute,
}
struct CommandTask {
    command: Command,
    phase: CommandPhase,
    args: Vec<String>,
    declarations: Option<Vec<LocalDeclaration>>,
    assignment: usize,
    redirect: usize,
    stdout: Dest,
    stderr: Dest,
    saved_input: Option<(String, usize, usize)>,
    input_guard_base: Option<usize>,
}
impl CommandTask {
    fn bytes(&self) -> usize {
        command_bytes(&self.command)
            + strings_bytes(&self.args)
            + self.declarations.as_ref().map_or(0, |values| {
                values
                    .iter()
                    .map(|(key, value)| {
                        key.len()
                            + value.as_ref().map_or(0, String::len)
                            + std::mem::size_of::<LocalDeclaration>()
                    })
                    .sum()
            })
            + self
                .saved_input
                .as_ref()
                .map_or(0, |(input, _, _)| input.len())
            + match &self.phase {
                CommandPhase::Arguments(words) | CommandPhase::Locals(words) => words.bytes(),
                CommandPhase::Assignment(key, words) | CommandPhase::Target(key, words) => {
                    key.len() + words.bytes()
                }
                _ => 0,
            }
            + [&self.stdout, &self.stderr]
                .into_iter()
                .map(|dest| {
                    if let Dest::File(file) = dest {
                        file.len()
                    } else {
                        0
                    }
                })
                .sum::<usize>()
    }
    fn new(command: Command) -> Self {
        Self {
            command,
            phase: CommandPhase::Begin,
            args: Vec::new(),
            declarations: None,
            assignment: 0,
            redirect: 0,
            stdout: Dest::Out,
            stderr: Dest::Err,
            saved_input: None,
            input_guard_base: None,
        }
    }
    fn next_assignment(&mut self, vm: &Vm) -> Result<(), String> {
        if let Some(word) = self.command.words.get(self.assignment) {
            let (key, value) = word.0[0].text().unwrap().split_once('=').unwrap();
            vm.reserve_memory(word_bytes(word) + key.len() + value.len())?;
            let mut word = word.clone();
            let (key, value) = word.0[0].text().unwrap().split_once('=').unwrap();
            let key = key.to_string();
            variable_target(&key)?;
            *word.0[0].text_mut()? = value.to_string();
            self.assignment += 1;
            self.phase = CommandPhase::Assignment(key, Words::new(vec![(word, false)]));
        } else {
            self.phase = CommandPhase::Redirect;
        }
        Ok(())
    }
    fn restore_input(&mut self, vm: &mut Vm) -> Result<(), String> {
        if let Some(base) = self.input_guard_base.take() {
            vm.ordinary_inputs.truncate(base);
        }
        if let Some((input, pos, depth)) = self.saved_input.take() {
            vm.release_pending(input.len())?;
            if vm.inputs.len() == depth {
                vm.input = input;
                vm.input_pos = pos;
            }
        }
        Ok(())
    }
    /// Complete one preparation unit, redirect, or builtin. Some(output) ends
    /// this command; None suspends it without restarting any prior phase.
    fn advance(&mut self, vm: &mut Vm, pipeline: bool) -> Result<Option<String>, String> {
        match std::mem::replace(&mut self.phase, CommandPhase::Begin) {
            CommandPhase::Begin => {
                if self.command.words.first().and_then(Word::literal) == Some("local") {
                    if pipeline {
                        return Err(pipeline_context_error());
                    }
                    if vm.calls.is_empty() {
                        return Err("Unsupported local outside a function; local variables belong to a function call".into());
                    }
                    if self.command.words.len() == 1 {
                        return Err(
                            "Unsupported local listing; supply a variable name or NAME=value"
                                .into(),
                        );
                    }
                    vm.reserve_memory(self.command.words[1..].iter().map(word_bytes).sum())?;
                    self.phase = CommandPhase::Locals(Words::new(
                        self.command.words[1..]
                            .iter()
                            .cloned()
                            .map(|word| {
                                let split = !word.assignment();
                                (word, split)
                            })
                            .collect(),
                    ));
                } else if self.command.words.iter().all(Word::assignment) {
                    self.next_assignment(vm)?;
                } else {
                    vm.reserve_memory(self.command.words.iter().map(word_bytes).sum())?;
                    self.phase = CommandPhase::Arguments(Words::new(
                        self.command
                            .words
                            .iter()
                            .cloned()
                            .map(|w| (w, true))
                            .collect(),
                    ));
                }
            }
            CommandPhase::Arguments(mut words) => {
                if let Some(args) = words.advance(vm)? {
                    let head = args.first();
                    if pipeline
                        && head.is_some_and(|head| {
                            vm.functions.contains_key(head) || head == "return" || head == "local"
                        })
                    {
                        return Err(pipeline_context_error());
                    }
                    if !pipeline
                        && let Some(start) = args
                            .first()
                            .and_then(|head| vm.functions.get(head))
                            .copied()
                    {
                        vm.release_pending(strings_bytes(&args))?;
                        vm.enter_function(&self.command, args, start)?;
                        return Ok(Some(String::new()));
                    }
                    self.args = args;
                    self.phase = CommandPhase::Redirect;
                } else {
                    self.phase = CommandPhase::Arguments(words);
                }
            }
            CommandPhase::Locals(mut words) => {
                if let Some(values) = words.advance(vm)? {
                    self.declarations = Some(local_declarations(values, vm)?);
                    self.phase = CommandPhase::Redirect;
                } else {
                    self.phase = CommandPhase::Locals(words);
                }
            }
            CommandPhase::Assignment(key, mut words) => {
                if let Some(mut values) = words.advance(vm)? {
                    let owned = strings_bytes(&values) + key.len();
                    let value = values.pop().unwrap_or_default();
                    debug_assert!(values.is_empty());
                    vm.release_pending(owned)?;
                    vm.assign_scalar(key, value)?;
                    vm.limits()?;
                    self.next_assignment(vm)?;
                } else {
                    self.phase = CommandPhase::Assignment(key, words);
                }
            }
            CommandPhase::Redirect => {
                if let Some((op, word)) = self.command.redirects.get(self.redirect) {
                    self.redirect += 1;
                    match op.as_str() {
                        ">&2" | "1>&2" => {
                            if let Dest::File(file) = &self.stderr {
                                vm.reserve_memory(file.len())?;
                            }
                            self.stdout = self.stderr.clone();
                            self.phase = CommandPhase::Redirect;
                        }
                        "2>&1" => {
                            if let Dest::File(file) = &self.stdout {
                                vm.reserve_memory(file.len())?;
                            }
                            self.stderr = self.stdout.clone();
                            self.phase = CommandPhase::Redirect;
                        }
                        _ => {
                            vm.reserve_memory(op.len() + word.as_ref().map_or(0, word_bytes))?;
                            self.phase = CommandPhase::Target(
                                op.clone(),
                                Words::new(vec![(word.as_ref().unwrap().clone(), true)]),
                            );
                        }
                    }
                } else {
                    self.phase = CommandPhase::Execute;
                }
            }
            CommandPhase::Target(op, mut words) => {
                if let Some(fields) = words.advance(vm)? {
                    if fields.len() != 1 {
                        return Err("Ambiguous redirection: quote a single file path".into());
                    }
                    vm.reserve_memory(fields[0].len().min(256) + 6)?;
                    let file = path(&fields[0])?;
                    vm.rebase_pending(0, file.len())?;
                    if op == "<" {
                        let Some(text) = vm.outcome.files.get(&file) else {
                            vm.outcome.status = 1;
                            vm.reserve_memory(fields[0].len() + 22)?;
                            let error = format!("shell: {}: No such file\n", fields[0]);
                            vm.rebase_pending(0, error.len())?;
                            vm.emit(&self.stderr, &error)?;
                            self.restore_input(vm)?;
                            return Ok(Some(String::new()));
                        };
                        // Bind now, before preparing the next redirect target.
                        // A later child must inherit precisely this cursor/binding.
                        vm.reserve_memory(text.len())?;
                        let text = text.clone();
                        let old = std::mem::replace(&mut vm.input, text);
                        let pos = std::mem::replace(&mut vm.input_pos, 0);
                        if self.saved_input.is_none() {
                            vm.rebase_pending(0, old.len())?;
                            self.saved_input = Some((old, pos, vm.inputs.len()));
                        }
                        if let Some(base) = self.input_guard_base {
                            vm.release_pending(file.len())?;
                            vm.ordinary_inputs[base] = file;
                        } else {
                            vm.reserve_memory(std::mem::size_of::<String>())?;
                            self.input_guard_base = Some(vm.ordinary_inputs.len());
                            vm.release_pending(file.len())?;
                            vm.ordinary_inputs.push(file);
                        }
                    } else {
                        vm.writable_file(&file)?;
                        if !vm.outcome.files.contains_key(&file) && vm.outcome.files.len() >= 64 {
                            return Err("Virtual file limit exceeded".into());
                        }
                        if !op.ends_with(">>") {
                            vm.reserve_memory(file.len())?;
                            vm.outcome.files.insert(file.clone(), String::new());
                        }
                        if op.starts_with('2') {
                            self.stderr = Dest::File(file);
                        } else {
                            self.stdout = Dest::File(file);
                        }
                    }
                    self.phase = CommandPhase::Redirect;
                } else {
                    self.phase = CommandPhase::Target(op, words);
                }
            }
            CommandPhase::Execute => {
                let (out, err, status) = if let Some(declarations) = self.declarations.take() {
                    vm.apply_local_declarations(declarations)?;
                    (String::new(), String::new(), 0)
                } else if self.command.words.iter().all(Word::assignment) {
                    (String::new(), String::new(), 0)
                } else {
                    if self
                        .args
                        .first()
                        .is_some_and(|a| a.split_once('=').is_some_and(|(key, _)| name(key)))
                    {
                        return Err("Assignments before a command are unsupported; put the assignment on its own line".into());
                    }
                    vm.builtin(&self.args)?
                };
                vm.rebase_pending(0, out.len() + err.len())?;
                self.restore_input(vm)?;
                vm.outcome.status = status;
                let mut output = if matches!(self.stdout, Dest::Out) {
                    out
                } else {
                    vm.emit(&self.stdout, &out)?;
                    String::new()
                };
                if matches!(self.stderr, Dest::Out) {
                    vm.reserve_memory(err.len())?;
                    output.push_str(&err);
                } else {
                    vm.emit(&self.stderr, &err)?;
                }
                return Ok(Some(output));
            }
        }
        Ok(None)
    }
}
fn pipeline_context_error() -> String {
    "Unsupported function call, return or local declaration in a pipeline; use separate commands"
        .into()
}
fn local_declarations(values: Vec<String>, vm: &Vm) -> Result<Vec<LocalDeclaration>, String> {
    let original = strings_bytes(&values);
    let mut memory = vm.scratch(0)?;
    let mut declarations = Vec::new();
    for value in values {
        let (key, new_value) = value
            .split_once('=')
            .map_or((value.as_str(), None), |(key, value)| (key, Some(value)));
        variable_target(key)?;
        if !name(key) {
            return Err("Unsupported local argument; use a valid variable name or NAME=value, without options or attributes".into());
        }
        memory.grow(
            vm,
            key.len() + new_value.map_or(0, str::len) + std::mem::size_of::<LocalDeclaration>(),
        )?;
        declarations.push((key.to_string(), new_value.map(str::to_owned)));
        if declarations.len() > 128 {
            return Err("Local declaration argument limit exceeded".into());
        }
    }
    if declarations.is_empty() {
        return Err("Unsupported local listing; supply a valid variable argument".into());
    }
    let retained = declarations
        .iter()
        .map(|(key, value)| {
            key.len()
                + value.as_ref().map_or(0, String::len)
                + std::mem::size_of::<LocalDeclaration>()
        })
        .sum();
    drop(memory);
    vm.rebase_pending(original, retained)?;
    Ok(declarations)
}

struct Snapshot {
    env: BTreeMap<String, Value>,
    args: Vec<String>,
    status: i32,
    input: String,
    pos: usize,
    bytes: usize,
}
pub(super) struct Pipeline {
    commands: std::vec::IntoIter<Command>,
    active: Option<CommandTask>,
    snapshot: Option<Snapshot>,
    multiple: bool,
    started: bool,
    stages: usize,
    consumed: usize,
    pipe: String,
}
impl Pipeline {
    fn bytes(&self) -> usize {
        self.commands
            .as_slice()
            .iter()
            .map(command_bytes)
            .sum::<usize>()
            + self.active.as_ref().map_or(0, CommandTask::bytes)
            + self.snapshot.as_ref().map_or(0, |snapshot| snapshot.bytes)
            + self.pipe.len()
    }
    pub(super) fn new(commands: Vec<Command>) -> Self {
        Self {
            multiple: commands.len() > 1,
            commands: commands.into_iter(),
            active: None,
            snapshot: None,
            started: false,
            stages: 0,
            consumed: 0,
            pipe: String::new(),
        }
    }
    pub(super) fn advance(&mut self, vm: &mut Vm) -> Result<bool, String> {
        if !self.started {
            self.started = true;
            if self.multiple {
                let bytes = vm
                    .env
                    .iter()
                    .map(|(n, v)| n.len() + v.bytes())
                    .sum::<usize>()
                    + strings_bytes(&vm.args)
                    + vm.input.len();
                if vm
                    .memory_bytes()
                    .checked_add(bytes)
                    .is_none_or(|n| n > MAX_TEXT)
                {
                    return Err(
                        "Virtual memory/file limit exceeded while retaining pipeline variables"
                            .into(),
                    );
                }
                vm.charge_expansion(bytes)?;
                self.snapshot = Some(Snapshot {
                    env: vm.env.clone(),
                    args: vm.args.clone(),
                    status: vm.outcome.status,
                    input: vm.input.clone(),
                    pos: vm.input_pos,
                    bytes,
                });
            }
            return Ok(false);
        }
        if let Some(command) = &mut self.active {
            if let Some(output) = command.advance(vm, self.multiple)? {
                self.active = None;
                if self.stages == 1 && self.multiple {
                    self.consumed = vm.input_pos;
                }
                self.pipe = output;
                if self.multiple {
                    vm.done = false; // exit terminates this isolated pipeline stage.
                }
            }
            return Ok(false);
        }
        if let Some(command) = self.commands.next() {
            if let Some(saved) = &self.snapshot {
                vm.charge_expansion(
                    saved.bytes + if self.stages == 0 { 0 } else { self.pipe.len() },
                )?;
                // The completed isolated stage can be released before copying
                // the retained parent snapshot for the next stage.
                vm.env.clear();
                vm.args.clear();
                vm.input.clear();
                let cloned = saved.bytes - saved.input.len()
                    + if self.stages == 0 {
                        saved.input.len() - saved.pos
                    } else {
                        0
                    };
                vm.reserve_memory(cloned)?;
                vm.env = saved.env.clone();
                vm.args = saved.args.clone();
                vm.outcome.status = saved.status;
                vm.input = if self.stages == 0 {
                    saved.input[saved.pos..].to_string()
                } else {
                    vm.release_pending(self.pipe.len())?;
                    std::mem::take(&mut self.pipe)
                };
                vm.input_pos = 0;
            }
            self.stages += 1;
            self.active = Some(CommandTask::new(command));
            return Ok(false);
        }
        vm.emit(&Dest::Out, &self.pipe)?;
        if let Some(saved) = self.snapshot.take() {
            vm.release_pending(saved.bytes)?;
            vm.env = saved.env;
            vm.args = saved.args;
            vm.input = saved.input;
            vm.input_pos = saved.pos + self.consumed;
        }
        Ok(true)
    }
}

pub(super) enum Pending {
    Pipeline(Box<Pipeline>),
    Array(String, Words),
    For(String, usize, Words),
    While(usize, Words),
}
impl Pending {
    pub(super) fn bytes(&self) -> usize {
        match self {
            Self::Pipeline(pipeline) => pipeline.bytes(),
            Self::Array(key, words) | Self::For(key, _, words) => key.len() + words.bytes(),
            Self::While(_, words) => words.bytes(),
        }
    }
    pub(super) fn advance(&mut self, vm: &mut Vm) -> Result<bool, String> {
        match self {
            Self::Pipeline(pipeline) => pipeline.advance(vm),
            Self::Array(key, words) => {
                let Some(elements) = words.advance(vm)? else {
                    return Ok(false);
                };
                vm.release_pending(key.len() + strings_bytes(&elements))?;
                vm.store_value(std::mem::take(key), Value::Indexed(elements))?;
                vm.outcome.status = 0;
                Ok(true)
            }
            Self::For(key, end, words) => {
                let Some(values) = words.advance(vm)? else {
                    return Ok(false);
                };
                if values.is_empty() {
                    vm.pc = *end;
                    vm.outcome.status = 0;
                } else {
                    vm.charge_expansion(values[0].len())?;
                    vm.reserve_memory(key.len() + values[0].len())?;
                    vm.assign_scalar(key.clone(), values[0].clone())?;
                    vm.release_pending(key.len() + strings_bytes(&values))?;
                    vm.reserve_memory(key.len() + strings_bytes(&values))?;
                    vm.loops.push((std::mem::take(key), values, 1));
                }
                Ok(true)
            }
            Self::While(end, words) => {
                let Some(fields) = words.advance(vm)? else {
                    return Ok(false);
                };
                if fields.len() != 1 {
                    return Err("Ambiguous loop input redirection: quote a single file path".into());
                }
                vm.reserve_memory(fields[0].len().min(256) + 6)?;
                let file = path(&fields[0])?;
                vm.rebase_pending(0, file.len())?;
                if let Some(text) = vm.outcome.files.get(&file) {
                    vm.reserve_memory(text.len())?;
                    let text = text.clone();
                    let input = std::mem::replace(&mut vm.input, text);
                    let pos = std::mem::replace(&mut vm.input_pos, 0);
                    vm.release_pending(file.len())?;
                    vm.inputs.push(InputFrame { input, pos, file });
                    vm.while_status.push((0, true));
                } else {
                    vm.outcome.status = 1;
                    vm.reserve_memory(fields[0].len() + 22)?;
                    let error = format!("shell: {}: No such file\n", fields[0]);
                    vm.rebase_pending(0, error.len())?;
                    vm.emit(&Dest::Err, &error)?;
                    vm.pc = *end;
                }
                Ok(true)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Case {
        Case {
            name: "Execution continuation".into(),
            args: vec![],
            stdin: "parent\n".into(),
            env: BTreeMap::from([
                ("head".into(), "render".into()),
                ("value".into(), "one two".into()),
            ]),
            files: BTreeMap::from([("/work/source.txt".into(), "source\n".into())]),
            stdout: String::new(),
            stderr: String::new(),
            status: 0,
            after: BTreeMap::new(),
        }
    }
    #[test]
    fn function_head_and_arguments_are_evaluated_exactly_once() {
        let mut vm = Vm::new("render() { :; }; \"$head\" \"$value\"", &fixture()).unwrap();
        let command = vm
            .program
            .code
            .iter()
            .rev()
            .find_map(|code| {
                if let Code::Pipe(commands) = code {
                    Some(commands[0].clone())
                } else {
                    None
                }
            })
            .unwrap();
        let expected_vm = Vm::new("", &fixture()).unwrap();
        let mut preparation =
            Words::new(command.words.into_iter().map(|word| (word, true)).collect());
        expected_vm.pending_bytes.set(preparation.bytes());
        let expected = loop {
            if let Some(fields) = preparation.advance(&expected_vm).unwrap() {
                break fields;
            }
        };
        assert_eq!(expected, ["render", "one two"]);
        for _ in 0..100 {
            vm.tick(1);
            if !vm.calls.is_empty() {
                break;
            }
        }
        assert_eq!(vm.calls.len(), 1);
        assert_eq!(vm.args, ["one two"]);
        assert_eq!(vm.expansion_work.get(), expected_vm.expansion_work.get());
    }
    #[test]
    fn an_input_redirect_is_live_before_the_next_target_is_prepared() {
        let mut vm = Vm::new("", &fixture()).unwrap();
        let program = compile(": < source.txt > output.txt").unwrap();
        let Code::Pipe(commands) = &program.code[0] else {
            panic!("expected pipe")
        };
        let mut command = CommandTask::new(commands[0].clone());
        for _ in 0..100 {
            vm.pending_bytes.set(command.bytes());
            assert!(command.advance(&mut vm, false).unwrap().is_none());
            if vm.input == "source\n" {
                break;
            }
        }
        assert_eq!(vm.input, "source\n");
        assert_eq!(vm.input_pos, 0);
        assert!(!vm.outcome.files.contains_key("/work/output.txt"));
        for _ in 0..100 {
            vm.pending_bytes.set(command.bytes());
            if command.advance(&mut vm, false).unwrap().is_some() {
                break;
            }
        }
        assert_eq!(vm.input, "parent\n");
        assert!(vm.outcome.files.contains_key("/work/output.txt"));
    }
}
