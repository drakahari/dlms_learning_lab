//! Bounded Bash startup files in one existing VM; never reads host startup files.
use super::*;
use serde::de::{Error as _, MapAccess, Visitor};
use std::fmt;
pub const MAX_FILES_BYTES: usize = 24576;
pub const PATHS: [&str; 6] = [
    "/etc/profile",
    "~/.bash_profile",
    "~/.bash_login",
    "~/.profile",
    "~/.bashrc",
    "/work/startup.env",
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
#[serde(transparent)]
pub struct Files(pub BTreeMap<String, File>);
impl<'de> Deserialize<'de> for Files {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Bounded;
        impl<'de> Visitor<'de> for Bounded {
            type Value = Files;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("at most six known virtual startup files")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Files, A::Error> {
                #[derive(Deserialize)]
                struct Key(#[serde(deserialize_with = "crate::cron::text::<_,32>")] String);
                let mut out = BTreeMap::new();
                while let Some(Key(key)) = a.next_key()? {
                    if out.len() == 6 || !PATHS.contains(&key.as_str()) || out.contains_key(&key) {
                        return Err(A::Error::custom(
                            "Unknown, duplicate or excess virtual startup file",
                        ));
                    }
                    out.insert(key, a.next_value()?);
                }
                let files = Files(out);
                files.validate().map_err(A::Error::custom)?;
                Ok(files)
            }
        }
        d.deserialize_map(Bounded)
    }
}
impl Files {
    pub fn validate(&self) -> Result<(), String> {
        if self.0.len() > 6 {
            return Err("Startup file limit: six known virtual paths".into());
        }
        let mut bytes = 0usize;
        for (path, file) in &self.0 {
            if !PATHS.contains(&path.as_str()) {
                return Err(
                    "Unsupported startup path: use only this exercise's supplied virtual files"
                        .into(),
                );
            }
            bytes = bytes
                .checked_add(path.len() + file.text.len())
                .ok_or("Startup source-size overflow")?;
            if file.text.len() > 8192 || file.text.contains('\0') {
                return Err("Startup file exceeds8KiB or contains a null byte".into());
            }
        }
        if bytes > MAX_FILES_BYTES {
            return Err("All startup files together exceed24KiB".into());
        }
        Ok(())
    }
    pub fn decode(raw: &str) -> Result<Self, String> {
        if raw.len() > 65536 {
            return Err("Startup file JSON exceeds64KiB".into());
        }
        serde_json::from_str(raw).map_err(|e| format!("Invalid virtual startup files: {e}"))
    }
}
fn env_file<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Text(#[serde(deserialize_with = "crate::cron::text::<_,32>")] String);
    Option::<Text>::deserialize(d).map(|s| s.map(|v| v.0))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Invocation {
    InteractiveLogin {
        noprofile: bool,
    },
    InteractiveNonLogin {
        norc: bool,
    },
    NonInteractive {
        #[serde(default, deserialize_with = "env_file")]
        bash_env: Option<String>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selected {
    pub path: String,
    pub text: String,
}
pub fn select(invocation: &Invocation, files: &Files) -> Result<Vec<Selected>, String> {
    files.validate()?;
    let mut paths = vec![];
    match invocation {
        Invocation::InteractiveLogin { noprofile: false } => {
            if files.0.contains_key("/etc/profile") {
                paths.push("/etc/profile")
            }
            if let Some(path) = PATHS[1..4].iter().find(|p| files.0.contains_key(**p)) {
                paths.push(*path)
            }
        }
        Invocation::InteractiveNonLogin { norc: false } => {
            if files.0.contains_key("~/.bashrc") {
                paths.push("~/.bashrc")
            }
        }
        Invocation::NonInteractive { bash_env } => {
            if let Some(path) = bash_env.as_ref().filter(|p| !p.is_empty()) {
                if path != "/work/startup.env" {
                    return Err("Unsupported BASH_ENV value: this exercise accepts only the literal supplied /work/startup.env; expansion and other paths are outside this subset".into());
                }
                if files.0.contains_key(path) {
                    paths.push("/work/startup.env")
                }
            }
        }
        _ => {}
    }
    Ok(paths
        .into_iter()
        .map(|path| Selected {
            path: path.into(),
            text: files.0[path].text.clone(),
        })
        .collect())
}
fn controlled(key: &str) -> bool {
    matches!(
        key,
        "HOME" | "BASH_ENV" | "OPTIND" | "OPTERR" | "OPTARG" | "LANG" | "LANGUAGE"
    ) || key.starts_with("LC_")
}
fn assignment_key(word: &Word) -> Option<String> {
    if !word.assignment() {
        return None;
    }
    let mut key = String::new();
    for part in &word.0 {
        for c in part.text()?.chars() {
            if c == '=' {
                return Some(key);
            }
            key.push(c);
        }
    }
    None
}
fn constant_word(word: &Word) -> Option<String> {
    let mut value = String::new();
    for part in &word.0 {
        let text = part.text()?;
        if part.quote != Quote::Single && text.contains('$') {
            return None;
        }
        if part.quote == Quote::Bare && text.contains(['*', '?', '[']) {
            return None;
        }
        value.push_str(text);
    }
    Some(value)
}
fn startup_program(source: &str) -> Result<Arc<Program>, String> {
    let program = compile(source).map_err(|error| {
        if error.to_ascii_lowercase().contains("unsupported") {
            format!("Unsupported startup syntax: {error}")
        } else {
            error
        }
    })?;
    if !program.bodies.is_empty() {
        return Err("Unsupported startup script: command substitution and function bodies need a broader session model".into());
    }
    for code in &program.code {
        match code{
            Code::Jump(_)|Code::JumpIf(_,_)|Code::Status(_)=>{},
            Code::Pipe(commands)if commands.len()==1=>{
                let command=&commands[0];
                if !command.redirects.is_empty(){return Err("Unsupported startup redirection: use assignments, printing and export in this activity".into())}
                if command.words.iter().all(Word::assignment){
                    if command.words.iter().filter_map(assignment_key).any(|k|controlled(&k)){return Err("Unsupported change to a supplied shell-control setting in this startup activity".into())}
                    continue
                }
                let head=command.words.first().and_then(Word::literal);
                if !matches!(head,Some("export"|"printf"|"test"|"["|"true"|"false"|":"|"exit")){
                    return Err("Unsupported startup command: use ordinary variable assignments, export, printf, test/[ and if/else or &&/||; advanced shell tools remain available in the scripting track".into())
                }
                if head==Some("printf")&&command.words.get(1).and_then(constant_word).is_none_or(|format|format.starts_with('-')){
                    return Err("Unsupported startup printf format: use a literal format such as '%s\\n', followed by quoted variable values; printf options and computed formats are outside this activity".into())
                }
                if head==Some("export")&&command.words.iter().skip(1).any(|w|assignment_key(w).or_else(||constant_word(w)).is_some_and(|k|controlled(&k))){return Err("Unsupported export of a supplied shell-control setting in this startup activity".into())}
            },
            _=>return Err("Unsupported startup script shape: this activity supports ordinary variables and conditions, without functions, loops, arrays or pipelines".into()),
        }
    }
    Ok(program)
}
/// One VM carries variables, attributes, status, output and budgets between files.
/// Its existing program arena is replaced only at a completed top-level boundary.
pub struct Run {
    selected: Vec<Selected>,
    programs: Vec<Arc<Program>>,
    at: usize,
    vm: Vm,
    stopped: bool,
    finished: bool,
}
impl Run {
    pub fn new(
        invocation: &Invocation,
        files: &Files,
        input: &Case,
        command: &str,
    ) -> Result<Self, String> {
        if !input.valid() {
            return Err("Invalid supplied startup inputs".into());
        }
        if input
            .env
            .keys()
            .any(|n| matches!(n.as_str(), "LANG" | "LANGUAGE") || n.starts_with("LC_"))
        {
            return Err(
                "Unsupported locale controls: this startup exercise uses fixed C/POSIX behavior"
                    .into(),
            );
        }
        if command.len() > 2048 {
            return Err("Supplied session command exceeds2KiB".into());
        }
        if input
            .env
            .get("HOME")
            .is_some_and(|home| home != "/home/learner")
        {
            return Err(
                "Unsupported virtual HOME: this activity fixes its simulated home at /home/learner"
                    .into(),
            );
        }
        let mut prepared = input.clone();
        prepared.env.insert("HOME".into(), "/home/learner".into());
        if !prepared.valid() {
            return Err("Startup inputs leave no room for the supplied virtual HOME".into());
        }
        let selected = select(invocation, files)?;
        let bytes = selected.iter().map(|s| s.text.len()).sum::<usize>() + command.len();
        if bytes > MAX_SOURCE {
            return Err("Selected startup files and session command exceed16KiB".into());
        }
        // Every file parses separately. A quote/control block cannot accidentally
        // become valid by consuming text from the next file. Admission is stricter
        // than real Bash's incremental startup: all selected inputs preflight first.
        let mut programs = vec![];
        for file in &selected {
            programs.push(startup_program(&file.text).map_err(|e| format!("{}: {e}", file.path))?);
        }
        programs
            .push(startup_program(command).map_err(|e| format!("Supplied session command: {e}"))?);
        let mut vm = Vm::from_program(programs[0].clone(), &prepared)?;
        // Account for all retained arenas and source labels, not only the active one.
        vm.program_bytes = programs.iter().map(|p| p.text_bytes()).sum::<usize>()
            + std::mem::size_of_val(programs.as_slice())
            + selected
                .iter()
                .map(|s| s.path.len() + s.text.len() + std::mem::size_of::<Selected>())
                .sum::<usize>();
        vm.limits()?;
        Ok(Self {
            selected,
            programs,
            at: 0,
            vm,
            stopped: false,
            finished: false,
        })
    }
    pub fn selected(&self) -> &[Selected] {
        &self.selected
    }
    pub fn tick(&mut self) {
        if self.stopped || self.finished {
            return;
        }
        self.vm.tick(16);
        if !self.vm.done() {
            return;
        }
        if self.vm.outcome.error.is_none()
            && self.vm.scope_exit.is_none()
            && self.vm.range.is_empty()
        {
            self.vm.outcome.status = 0;
        }
        if self.vm.outcome.error.is_some()
            || self.vm.scope_exit.is_some()
            || self.at + 1 == self.programs.len()
        {
            self.finished = true;
            return;
        }
        if !self.vm.calls.is_empty() || !self.vm.frames.is_empty() || self.vm.pending.is_some() {
            self.vm.fail("Invalid startup execution boundary".into());
            self.finished = true;
            return;
        }
        // A sourced empty file returns zero, instead of inheriting a previous
        // failure. Nonempty files start with the previous status still visible.
        self.at += 1;
        self.vm.program = self.programs[self.at].clone();
        self.vm.range = self.vm.program.main.clone();
        self.vm.pc = self.vm.range.start;
        self.vm.frame_executed = false;
        self.vm.done = false;
        if let Err(e) = self.vm.limits() {
            self.vm.fail(e);
            self.finished = true;
        }
    }
    pub fn stop(&mut self) {
        self.stopped = true
    }
    pub fn done(&self) -> bool {
        !self.stopped && self.finished
    }
    pub fn outcome(&self) -> Option<&Outcome> {
        self.done().then_some(&self.vm.outcome)
    }
    /// A look-alike print followed by exit must not stand in for the supplied
    /// session command. This evidence is derived from private execution state.
    pub fn session_command_finished(&self) -> bool {
        self.done()
            && self.at + 1 == self.programs.len()
            && self.vm.scope_exit.is_none()
            && self.vm.outcome.error.is_none()
    }
    pub fn environment(&self) -> Result<EnvironmentSnapshot, String> {
        if !self.done() {
            return Err("Startup run is unfinished or stopped".into());
        }
        self.vm.environment_snapshot()
    }
}
