//! Root-owned cooperative capture state. No child Vm, host I/O, or fresh budget.
use super::*;

pub(super) const MAX_CAPTURES: usize = 8;
const MAX_BUFFERS: usize = MAX_CAPTURES * 2 + 1;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BufferId(pub(super) usize);
pub(super) struct Suspended {
    pub pc: usize,
    pub range: Range<usize>,
    pub executed: bool,
    pub env: BTreeMap<String, Value>,
    pub scanner: getopts::Scanner,
    pub pipefail: bool,
    pub args: Vec<String>,
    pub loops: Vec<(String, Vec<String>, usize)>,
    pub whiles: Vec<(i32, bool)>,
    pub inputs: Vec<InputFrame>,
    pub functions: BTreeMap<String, usize>,
    pub calls: Vec<CallFrame>,
    pub pending: execution::Pending,
    pub stdout: Dest,
    pub stderr: Dest,
    pub input_id: usize,
    pub guard_depth: usize,
    pub output: BufferId,
    resident_bytes: usize,
}
impl Suspended {
    pub fn bytes(&self) -> usize {
        self.resident_bytes
    }
    fn measure_bytes(&self) -> usize {
        context_bytes(
            &self.env,
            &self.args,
            &self.loops,
            &self.inputs,
            &self.functions,
            &self.calls,
        ) + self.scanner.bytes()
            + self.pending.bytes()
            + dest_bytes(&self.stdout)
            + dest_bytes(&self.stderr)
    }
}
pub(super) fn dest_bytes(dest: &Dest) -> usize {
    if let Dest::File(path) = dest {
        path.len()
    } else {
        0
    }
}
pub(super) fn context_bytes(
    env: &BTreeMap<String, Value>,
    args: &[String],
    loops: &[(String, Vec<String>, usize)],
    inputs: &[InputFrame],
    functions: &BTreeMap<String, usize>,
    calls: &[CallFrame],
) -> usize {
    env.iter()
        .map(|(key, value)| key.len() + value.bytes())
        .sum::<usize>()
        + strings_bytes(args)
        + loops
            .iter()
            .map(|(key, values, _)| key.len() + strings_bytes(values))
            .sum::<usize>()
        + inputs
            .iter()
            .map(|input| input.input.len() + input.file.len())
            .sum::<usize>()
        + functions.keys().map(String::len).sum::<usize>()
        + calls
            .iter()
            .map(|call| {
                strings_bytes(&call.args)
                    + call.scanner.as_ref().map_or(0, getopts::Scanner::bytes)
                    + call
                        .locals
                        .iter()
                        .map(|(key, value)| key.len() + value.as_ref().map_or(0, Value::bytes))
                        .sum::<usize>()
            })
            .sum::<usize>()
}
impl Vm {
    pub(super) fn advance_pending(
        &mut self,
        mut pending: execution::Pending,
    ) -> Result<Option<execution::Pending>, String> {
        let complete = pending.advance(self)?;
        self.pending_bytes
            .set(if complete { 0 } else { pending.bytes() });
        self.pending_output_bytes
            .set(if complete { 0 } else { pending.output_bytes() });
        if let Some(body) = self.capture_request.take() {
            if complete {
                return Err("Capture requested after command completion".into());
            }
            let stderr = pending.capture_stderr(self)?;
            // The copied descriptor path is live before a child buffer or
            // environment is allocated, even though it is not part of Pending.
            self.rebase_pending(0, dest_bytes(&stderr))?;
            self.start_capture(body, pending, stderr)?;
            return Ok(None);
        }
        self.limits()?;
        Ok((!complete).then_some(pending))
    }
    fn start_capture(
        &mut self,
        body: BodyId,
        pending: execution::Pending,
        stderr: Dest,
    ) -> Result<(), String> {
        if self.frames.len() >= MAX_CAPTURES {
            return Err("Command-substitution depth limit exceeded (8)".into());
        }
        let range = self
            .program
            .bodies
            .get(body.0)
            .cloned()
            .ok_or("Invalid capture body")?;
        let output = self.new_buffer()?;
        let bytes = self
            .env
            .iter()
            .map(|(key, value)| key.len() + value.bytes())
            .sum::<usize>()
            + strings_bytes(&self.args)
            + self.functions.keys().map(String::len).sum::<usize>()
            + self.scanner.bytes();
        self.reserve_memory(bytes)?;
        self.charge_expansion(bytes + dest_bytes(&stderr))?;
        let env = self.env.clone();
        let args = self.args.clone();
        let functions = self.functions.clone();
        let scanner = self.scanner.clone();
        let mut frame = Suspended {
            pc: self.pc,
            range: std::mem::replace(&mut self.range, range.clone()),
            executed: std::mem::replace(&mut self.frame_executed, false),
            env: std::mem::replace(&mut self.env, env),
            scanner: std::mem::replace(&mut self.scanner, scanner),
            pipefail: self.pipefail,
            args: std::mem::replace(&mut self.args, args),
            functions: std::mem::replace(&mut self.functions, functions),
            loops: std::mem::take(&mut self.loops),
            whiles: std::mem::take(&mut self.while_status),
            inputs: std::mem::take(&mut self.inputs),
            calls: std::mem::take(&mut self.calls),
            pending,
            stdout: std::mem::replace(&mut self.base_stdout, Dest::Buffer(output)),
            stderr: std::mem::replace(&mut self.base_stderr, stderr),
            input_id: self.input_id,
            guard_depth: self.ordinary_inputs.len(),
            output,
            resident_bytes: 0,
        };
        frame.resident_bytes = frame.measure_bytes();
        self.pc = range.start;
        self.pending_bytes.set(0);
        self.pending_output_bytes.set(0);
        self.scope_exit = None;
        self.frames.push(frame);
        self.limits()
    }
    pub(super) fn finish_capture(&mut self) -> Result<(), String> {
        let frame = self.frames.last().ok_or("Missing capture parent")?;
        let status = self.scope_exit.unwrap_or(if self.frame_executed {
            self.outcome.status
        } else {
            0
        });
        let output = frame.output;
        let inherited_id = frame.input_id;
        let guard_depth = frame.guard_depth;
        while !self.inputs.is_empty() {
            self.restore_input()?;
        }
        if self.input_id != inherited_id {
            return Err("Capture did not restore its inherited input binding".into());
        }
        self.ordinary_inputs.truncate(guard_depth);
        self.env.clear();
        self.scanner = getopts::Scanner::default();
        self.args.clear();
        self.functions.clear();
        self.calls.clear();
        self.loops.clear();
        self.while_status.clear();
        let raw = self.buffer(output)?;
        self.charge_expansion(raw.len())?;
        let trimmed = raw.trim_end_matches('\n').len();
        let value = if trimmed == raw.len() {
            self.take_buffer(output)?
        } else {
            self.reserve_memory(trimmed)?;
            self.charge_expansion(trimmed)?;
            let value = raw[..trimmed].to_owned();
            self.take_buffer(output)?;
            value
        };
        let frame = self.frames.pop().ok_or("Missing capture parent")?;
        self.pc = frame.pc;
        self.range = frame.range;
        self.frame_executed = frame.executed;
        self.env = frame.env;
        self.scanner = frame.scanner;
        self.pipefail = frame.pipefail;
        self.args = frame.args;
        self.functions = frame.functions;
        self.calls = frame.calls;
        self.loops = frame.loops;
        self.while_status = frame.whiles;
        self.inputs = frame.inputs;
        self.base_stdout = frame.stdout;
        self.base_stderr = frame.stderr;
        self.scope_exit = None;
        self.outcome.status = status;
        let mut pending = frame.pending;
        self.pending_bytes.set(pending.bytes());
        self.pending_output_bytes.set(pending.output_bytes());
        pending.resume_capture(self, value, status)?;
        self.pending_bytes.set(pending.bytes());
        self.pending = Some(pending);
        self.limits()
    }
    pub(super) fn new_buffer(&mut self) -> Result<BufferId, String> {
        self.reserve_memory(std::mem::size_of::<String>())?;
        if let Some(index) = self.buffers.iter().position(Option::is_none) {
            self.buffers[index] = Some(String::new());
            return Ok(BufferId(index));
        }
        if self.buffers.len() == MAX_BUFFERS {
            return Err("Active output-buffer limit exceeded".into());
        }
        self.buffers.push(Some(String::new()));
        Ok(BufferId(self.buffers.len() - 1))
    }
    pub(super) fn buffer(&self, id: BufferId) -> Result<&String, String> {
        self.buffers
            .get(id.0)
            .and_then(Option::as_ref)
            .ok_or_else(|| "Invalid output binding".into())
    }
    pub(super) fn take_buffer(&mut self, id: BufferId) -> Result<String, String> {
        self.buffers
            .get_mut(id.0)
            .and_then(Option::take)
            .ok_or_else(|| "Invalid output binding".into())
    }
    pub(super) fn buffer_bytes(&self) -> usize {
        self.buffers
            .iter()
            .flatten()
            .map(|text| text.len() + std::mem::size_of::<String>())
            .sum()
    }
    pub(super) fn output_bytes(&self) -> usize {
        self.outcome.stdout.len()
            + self.outcome.stderr.len()
            + self
                .buffers
                .iter()
                .flatten()
                .map(String::len)
                .sum::<usize>()
            + self.pending_output_bytes.get()
            + self
                .frames
                .iter()
                .map(|frame| frame.pending.output_bytes())
                .sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Deserialize)]
    struct Reference {
        name: String,
        source: String,
        case: Case,
    }
    #[derive(Deserialize)]
    struct OtherReferences {
        native_unsupported_observations: Vec<Reference>,
        supported_diagnostic_observations: Vec<Reference>,
    }
    fn fixture() -> Case {
        Case {
            name: "Native-only capture fixture".into(),
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
    fn vm(source: &str, case: &Case) -> Result<Vm, String> {
        if !case.valid() {
            return Err("Invalid simulation fixture".into());
        }
        Vm::from_program(compile(source)?, case)
    }
    fn internal(source: &str, case: &Case, slice: usize) -> Outcome {
        let mut vm = match vm(source, case) {
            Ok(vm) => vm,
            Err(error) => {
                return Outcome {
                    stdout: String::new(),
                    stderr: String::new(),
                    status: 2,
                    files: case.files.clone(),
                    error: Some(diagnostic(error)),
                };
            }
        };
        for _ in 0..20_000 {
            vm.tick(slice);
            if vm.done() {
                return vm.outcome;
            }
        }
        panic!("Internal capture fixture did not finish");
    }
    #[derive(Deserialize)]
    struct NativeCases {
        cases: Vec<NativeCase>,
    }
    #[derive(Deserialize)]
    struct NativeCase {
        name: String,
        source: String,
        #[serde(default)]
        env: BTreeMap<String, String>,
        #[serde(default)]
        files: BTreeMap<String, String>,
        expected_error: Option<String>,
    }
    #[test]
    fn all_twenty_two_native_only_cases_obey_shared_limits_and_exclusions() {
        let cases: NativeCases = serde_json::from_str(include_str!(
            "../../tests/fixtures/shell-substitution-vm-only.json"
        ))
        .unwrap();
        assert_eq!(cases.cases.len(), 22);
        for native in cases.cases {
            let mut case = fixture();
            case.env = native.env;
            case.files = native.files;
            let mut across_slices = None;
            for slice in [1, 16, 32, 64] {
                if native.name.starts_with("stop-") {
                    let mut vm = vm(&native.source, &case).unwrap();
                    for _ in 0..100 {
                        vm.tick(1);
                        if !vm.frames.is_empty()
                            && (native.name.contains("loop") || !vm.calls.is_empty())
                        {
                            break;
                        }
                    }
                    assert!(
                        !vm.frames.is_empty(),
                        "{} must stop inside capture",
                        native.name
                    );
                    assert!(!vm.done());
                    vm.stop();
                    let stopped = vm.outcome.clone();
                    vm.tick(slice);
                    assert_eq!(vm.outcome, stopped);
                    assert!(vm.frames.is_empty() && vm.buffers.is_empty() && vm.pending.is_none());
                    assert!(vm.outcome.error.as_deref().unwrap().contains("Stopped"));
                    assert!(!vm.outcome.stdout.contains("wrong"));
                    continue;
                }
                let result = internal(&native.source, &case, slice);
                if let Some(expected) = &across_slices {
                    assert_eq!(
                        &result, expected,
                        "{} changes with slice {slice}",
                        native.name
                    );
                } else {
                    across_slices = Some(result.clone());
                }
                if let Some(error) = &native.expected_error {
                    assert_eq!(result.status, 2, "{}: {result:?}", native.name);
                    let actual = result.error.as_deref().unwrap();
                    if native.name == "compile-before-first-side-effect" {
                        assert_eq!(result.files, case.files);
                    } else {
                        assert!(
                            actual
                                .to_ascii_lowercase()
                                .contains(&error.to_ascii_lowercase()),
                            "{}: {result:?}",
                            native.name
                        );
                    }
                    if native.name == "ancestor-input-write-guard" {
                        assert_eq!(result.files, case.files);
                    }
                } else {
                    assert_eq!(result.error, None, "{}: {result:?}", native.name);
                    assert_eq!(result.status, 0);
                    let expected = match native.name.as_str() {
                        "depth-eight" => "value",
                        "sites-128" => "",
                        "capture-data-not-source" => "literal$label; (not syntax)",
                        other => panic!("Unspecified native success: {other}"),
                    };
                    assert_eq!(result.stdout, expected);
                }
            }
        }
    }
    #[test]
    fn function_calls_are_aggregated_over_parent_and_child_frames() {
        let source = "recur() { printf '%s' x; recur; }; outer() { value=$(recur); }; outer";
        let mut vm = vm(source, &fixture()).unwrap();
        let mut maximum = 0;
        for _ in 0..2000 {
            let before = vm.steps;
            vm.tick(1);
            assert!(vm.steps - before <= 1);
            maximum = maximum.max(
                vm.calls.len()
                    + vm.frames
                        .iter()
                        .map(|frame| frame.calls.len())
                        .sum::<usize>(),
            );
            if vm.done() {
                break;
            }
        }
        assert_eq!(maximum, 32);
        assert!(
            vm.outcome
                .error
                .as_deref()
                .unwrap()
                .contains("Function call depth")
        );
    }
    #[test]
    fn descendants_use_the_requested_instruction_slice_and_one_instruction_ceiling() {
        let source = format!(
            "for index in {}; do value=$(for item in {}; do :; done); done",
            "x ".repeat(100),
            "y ".repeat(100)
        );
        for slice in [1, 16, 32, 64, 128] {
            let mut vm = vm(&source, &fixture()).unwrap();
            for _ in 0..10_000 {
                let before = vm.steps;
                vm.tick(slice);
                assert!(vm.steps - before <= slice.min(64));
                if vm.done() {
                    break;
                }
            }
            assert_eq!(vm.steps, MAX_STEPS);
            assert!(
                vm.outcome
                    .error
                    .as_deref()
                    .unwrap()
                    .contains("Execution limit")
            );
        }
    }
    #[test]
    fn unset_parent_locals_still_consume_the_tree_wide_variable_name_limit() {
        let declarations = (0..255).map(|n| format!("v{n}")).collect::<Vec<_>>();
        let source = format!(
            "outer() {{ local {}; local {}; local v254; v0=$(fresh=x; other=y; printf wrong > touched.txt); }}; outer",
            declarations[..127].join(" "),
            declarations[127..254].join(" ")
        );
        let result = internal(&source, &fixture(), 1);
        assert!(result.error.as_deref().unwrap().contains("Variable limit"));
        assert!(!result.files.contains_key("/work/touched.txt"));
    }
    #[test]
    fn raw_newline_boundary_and_nested_output_share_one_retained_output_ceiling() {
        let mut case = fixture();
        case.env.insert("lines".into(), "\n".repeat(1024));
        for (count, pass) in [(32, true), (33, false)] {
            let source = format!(
                "value=$(for item in {}; do printf '%s' \"$lines\"; done); printf '<%s>' \"$value\"",
                "x ".repeat(count)
            );
            for slice in [1, 16, 32, 64] {
                let result = internal(&source, &case, slice);
                if pass {
                    assert_eq!(result.error, None);
                    assert_eq!(result.stdout, "<>");
                } else {
                    assert!(result.error.as_deref().unwrap().contains("Output limit"));
                    assert!(result.stdout.is_empty());
                }
            }
        }
        let one_over = format!(
            "value=$(for item in {}; do printf '%s' \"$lines\"; done; printf '\\n'); printf wrong",
            "x ".repeat(32)
        );
        let result = internal(&one_over, &case, 1);
        assert!(result.error.as_deref().unwrap().contains("Output limit"));
        assert!(result.stdout.is_empty());
        let mut case = fixture();
        case.env.insert("blob".into(), "x".repeat(20_000));
        let result = internal(
            "value=$(printf '%s' \"$blob\"; printf '%s' \"$(printf '%s' \"$blob\")\"); printf wrong",
            &case,
            1,
        );
        assert!(result.error.as_deref().unwrap().contains("Output limit"));
        assert!(result.stdout.is_empty());
    }
    #[test]
    fn child_snapshots_have_a_positive_boundary_and_preflight_before_child_file_effects() {
        for (bytes, pass) in [(60_000, true), (85_000, false)] {
            let mut case = fixture();
            case.env.insert("pad".into(), "x".repeat(bytes));
            let source = "copy=$pad; value=$(printf changed > touched.txt)";
            let result = internal(source, &case, 1);
            if pass {
                assert_eq!(result.error, None);
                assert_eq!(result.files["/work/touched.txt"], "changed");
            } else {
                assert!(result.error.as_deref().unwrap().contains("memory"));
                assert_eq!(result.files, case.files);
            }
        }
    }
    #[test]
    fn the_inherited_input_binding_moves_once_and_returns_its_consumed_cursor() {
        fn send<T: Send>() {}
        send::<Vm>();
        let mut case = fixture();
        case.env.insert("pad".into(), "x".repeat(65_500));
        case.stdin = "x\n".to_owned() + &"z".repeat(65_498);
        let mut vm = vm("value=$(read -r line); test -n \"$pad\"", &case).unwrap();
        let identity = vm.input_id;
        let mut saw_capture = false;
        for _ in 0..2000 {
            vm.tick(1);
            if !vm.frames.is_empty() {
                saw_capture = true;
                assert_eq!(vm.frames[0].input_id, identity);
                assert_eq!(vm.input_id, identity);
                assert!(vm.memory_bytes() < 210_000);
            }
            if vm.done() {
                break;
            }
        }
        assert!(saw_capture);
        assert_eq!(vm.outcome.error, None);
        assert_eq!(vm.input_id, identity);
        assert_eq!(vm.input_pos, 2);
    }
    #[test]
    fn child_return_and_exit_unwind_redirected_inputs_before_returning_the_binding() {
        let mut case = fixture();
        case.stdin = "parent\n".into();
        case.files.insert("/work/source.txt".into(), "one\n".into());
        case.files
            .insert("/work/return.txt".into(), "unused\n".into());
        for body in [
            "f() { while read -r line; do return 7 < return.txt; done < source.txt; }; f",
            "while read -r line; do exit 7 < return.txt; done < source.txt",
        ] {
            let source = format!(
                "value=$({body}); status=$?; read -r line; printf '%s:%s' \"$status\" \"$line\""
            );
            for slice in [1, 16, 32, 64] {
                let result = internal(&source, &case, slice);
                assert_eq!(result.error, None);
                assert_eq!(result.stdout, "7:parent");
                assert_eq!(result.files, case.files);
            }
        }
    }
    #[test]
    fn ordinary_input_guards_and_partial_redirect_bindings_survive_child_frames() {
        let mut case = fixture();
        case.files
            .insert("/work/source.txt".into(), "source\n".into());
        case.files
            .insert("/work/other.txt".into(), "other\n".into());
        let denied = internal(
            ": < source.txt >\"$(printf changed > source.txt; printf destination.txt)\"",
            &case,
            1,
        );
        assert!(
            denied
                .error
                .as_deref()
                .unwrap()
                .contains("active input source")
        );
        assert_eq!(denied.files, case.files);
        let allowed = internal(
            ": < source.txt < other.txt >\"$(printf changed > source.txt; printf destination.txt)\"",
            &case,
            1,
        );
        assert_eq!(allowed.error, None);
        assert_eq!(allowed.files["/work/source.txt"], "changed");
        assert_eq!(allowed.files["/work/other.txt"], "other\n");
        assert_eq!(allowed.files["/work/destination.txt"], "");
    }
    #[test]
    fn serial_children_share_the_file_map_and_file_ceiling() {
        let mut case = fixture();
        for n in 0..63 {
            case.files
                .insert(format!("/work/keep{n}.txt"), String::new());
        }
        let result = internal(
            "a=$(printf first > new.txt); b=$(printf second > other.txt); printf wrong",
            &case,
            1,
        );
        assert!(result.error.as_deref().unwrap().contains("file limit"));
        assert_eq!(result.files.len(), 64);
        assert_eq!(result.files["/work/new.txt"], "first");
        assert!(!result.files.contains_key("/work/other.txt"));
        assert!(result.stdout.is_empty());
    }
    #[test]
    fn prepared_capture_data_stays_charged_while_later_children_execute() {
        let mut case = fixture();
        case.files
            .insert("/work/data.txt".into(), "x".repeat(30_000));
        case.files.insert("/work/trace.txt".into(), String::new());
        for (count, pass) in [(5, true), (8, false)] {
            let source = "printf '' ".to_owned()
                + &"\"$(printf x >> trace.txt; cat data.txt)\" ".repeat(count);
            let result = internal(&source, &case, 1);
            if pass {
                assert_eq!(result.error, None);
                assert_eq!(result.files["/work/trace.txt"].len(), count);
            } else {
                assert!(
                    result.error.as_deref().unwrap().contains("memory"),
                    "{:?}",
                    result.error
                );
                assert!(result.files["/work/trace.txt"].len() < count);
            }
            assert!(result.stdout.is_empty());
        }
    }
    #[test]
    fn runtime_recursion_cannot_reuse_one_compiled_site_past_eight_active_captures() {
        let mut vm = vm("recur() { value=$(recur); }; recur", &fixture()).unwrap();
        let mut maximum = 0;
        for _ in 0..2000 {
            vm.tick(1);
            maximum = maximum.max(vm.frames.len());
            if vm.done() {
                break;
            }
        }
        assert_eq!(maximum, 8);
        assert!(vm.outcome.error.as_deref().unwrap().contains("depth limit"));
    }
    #[test]
    fn cooperative_driver_compares_selected_reference_cases_at_all_tick_slices() {
        let cases: Vec<Reference> = serde_json::from_str(include_str!(
            "../../tests/fixtures/shell-substitution-reference.json"
        ))
        .unwrap();
        let mut failures = Vec::new();
        for reference in cases {
            for slice in [1, 16, 32, 64] {
                let result = internal(&reference.source, &reference.case, slice);
                if !reference.case.accepts(&result) {
                    failures.push((reference.name.clone(), slice, result));
                }
            }
        }
        assert!(failures.is_empty(), "{failures:#?}");
    }
    #[test]
    fn excluded_real_bash_forms_are_honest_and_supported_missing_input_skips_capture() {
        let cases: OtherReferences = serde_json::from_str(include_str!(
            "../../tests/fixtures/shell-substitution-excluded-diagnostic.json"
        ))
        .unwrap();
        assert_eq!(cases.native_unsupported_observations.len(), 8);
        assert_eq!(cases.supported_diagnostic_observations.len(), 1);
        for reference in cases.native_unsupported_observations {
            for slice in [1, 16, 32, 64] {
                let result = internal(&reference.source, &reference.case, slice);
                assert_eq!(result.status, 2, "{}: {result:?}", reference.name);
                assert!(
                    result
                        .error
                        .as_deref()
                        .unwrap()
                        .to_ascii_lowercase()
                        .contains("unsupported"),
                    "{}: {result:?}",
                    reference.name
                );
                assert_eq!(result.files, reference.case.files);
                assert!(result.stdout.is_empty());
            }
        }
        for reference in cases.supported_diagnostic_observations {
            for slice in [1, 16, 32, 64] {
                let result = internal(&reference.source, &reference.case, slice);
                assert_eq!(result.error, None);
                assert_eq!(result.status, reference.case.status);
                assert_eq!(result.stdout, reference.case.stdout);
                assert_eq!(result.files, reference.case.after);
                assert!(
                    result.stderr.contains("missing.txt") && result.stderr.contains("No such file")
                );
                assert!(reference.case.stderr.contains("missing.txt"));
                assert!(!result.files.contains_key("/work/side.txt"));
            }
        }
    }
}
