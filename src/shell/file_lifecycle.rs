//! Bounded flat virtual-file operations. No host I/O or subprocesses.
use super::{Dest, Vm, path};
const CANDIDATES: usize = 65;
const SEQUENCE_LIMIT: u32 = 1_000_000;
fn aliases(dest: &Dest, file: &str) -> bool {
    matches!(dest,Dest::File(name)if name==file)
}
fn operands<'a>(cmd: &str, args: &'a [String], count: usize) -> Result<&'a [String], String> {
    let (args, ended) = if args.first().is_some_and(|s| s == "--") {
        (&args[1..], true)
    } else {
        (args, false)
    };
    if args.len() != count || !ended && args.iter().any(|s| s.starts_with('-')) {
        return Err(format!(
            "Unsupported {cmd} arguments. Use the documented single-file form with optional --; no directory or other options are simulated."
        ));
    }
    Ok(args)
}
impl Vm {
    fn mutable_file(&self, file: &str, stdout: &Dest, stderr: &Dest) -> Result<(), String> {
        self.writable_file(file)?;
        let opened = aliases(stdout, file)
            || aliases(stderr, file)
            || aliases(&self.base_stdout, file)
            || aliases(&self.base_stderr, file)
            || self
                .pending
                .as_ref()
                .is_some_and(|p| p.output_aliases(file))
            || self.frames.iter().any(|frame| {
                aliases(&frame.stdout, file)
                    || aliases(&frame.stderr, file)
                    || frame.pending.output_aliases(file)
            });
        if opened {
            return Err("Unsupported mutation of an active output path: this simulator does not model renamed/unlinked file descriptors. Earlier redirection effects are not rolled back.".into());
        }
        Ok(())
    }
    pub(super) fn file_lifecycle(
        &mut self,
        cmd: &str,
        args: &[String],
        stdout: &Dest,
        stderr: &Dest,
    ) -> Result<(String, String, i32), String> {
        // Bound transient normalized keys, output/diagnostics and a generated key
        // before allocation. File contents are moved, never cloned on rename.
        self.reserve_memory(6 * 256 + 128)?;
        match cmd {
            "mktemp" => {
                let values = operands(cmd, args, 1)?;
                let template = path(&values[0])?;
                let prefix = template
                    .strip_suffix("XXXXXX")
                    .filter(|p| !p.ends_with('X'))
                    .ok_or("Unsupported mktemp template: use one explicit flat name ending in exactly six X characters")?;
                if self.outcome.files.len() >= 64 {
                    return Err("Virtual file limit exceeded (64 files)".into());
                }
                for _ in 0..CANDIDATES {
                    let n = self.temporary_sequence;
                    if n >= SEQUENCE_LIMIT {
                        return Err("Temporary allocation sequence limit reached".into());
                    }
                    self.temporary_sequence += 1;
                    self.charge_expansion(template.len())?;
                    let candidate = format!("{prefix}{n:06}");
                    if self.outcome.files.contains_key(&candidate) {
                        continue;
                    }
                    self.mutable_file(&candidate, stdout, stderr)?;
                    let printed = if values[0].starts_with('/') {
                        candidate.as_str()
                    } else {
                        candidate.strip_prefix("/work/").unwrap()
                    };
                    let output = format!("{printed}\n");
                    self.outcome.files.insert(candidate, String::new());
                    return Ok((output, String::new(), 0));
                }
                Err("Temporary allocation collision limit reached".into())
            }
            "mv" => {
                let values = operands(cmd, args, 2)?;
                let source = path(&values[0])?;
                let destination = path(&values[1])?;
                self.mutable_file(&source, stdout, stderr)?;
                self.mutable_file(&destination, stdout, stderr)?;
                if !self.outcome.files.contains_key(&source) {
                    return Ok((
                        String::new(),
                        format!("mv: {}: No such virtual file\n", values[0]),
                        1,
                    ));
                }
                if source == destination {
                    return Ok((
                        String::new(),
                        "mv: source and destination are the same virtual file\n".into(),
                        1,
                    ));
                }
                let contents = self.outcome.files.remove(&source).unwrap();
                drop(self.outcome.files.insert(destination, contents));
                Ok((String::new(), String::new(), 0))
            }
            "rm" => {
                let values = operands(cmd, args, 1)?;
                let file = path(&values[0])?;
                self.mutable_file(&file, stdout, stderr)?;
                if self.outcome.files.remove(&file).is_none() {
                    Ok((
                        String::new(),
                        format!("rm: {}: No such virtual file\n", values[0]),
                        1,
                    ))
                } else {
                    Ok((String::new(), String::new(), 0))
                }
            }
            _ => Err("Unknown simulated file operation".into()),
        }
    }
}
