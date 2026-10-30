//! Bounded, pure shell-subset interpreter. No host I/O, subprocesses or network.
mod arithmetic;
mod capture;
mod case;
pub mod cron;
mod execution;
mod getopts;
mod globbing;
mod grep;
mod literal_trim;
mod loop_control;
pub mod one_off;
mod parameters;
mod preparation;
mod substitution;
mod text_tools;
mod values;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
    ops::Range,
    sync::Arc,
};
use values::{ArrayRef, Value};
// Increment when previously saved outcomes can change under corrected evaluator semantics.
pub const MAX_CASES: usize = 12;
pub const EVALUATOR_REVISION: u32 = 15;
pub(crate) fn legacy_revision(revision: &u32) -> bool {
    *revision == 0
}
pub const MAX_SOURCE: usize = 16_384;
const MAX_TEXT: usize = 262_144;
const MAX_OUTPUT: usize = 32_768;
const MAX_STEPS: usize = 2000;
const MAX_ORDINARY_NAMES: usize = 256;
const MAX_SYSTEM_NAMES: usize = 3;
fn ordinary_variable_name(key: &str) -> bool {
    !matches!(key, "OPTIND" | "OPTERR" | "OPTARG")
}
fn variable_names_within_limit(names: &BTreeSet<&str>) -> bool {
    names.len() <= MAX_ORDINARY_NAMES + MAX_SYSTEM_NAMES
        && names
            .iter()
            .filter(|key| ordinary_variable_name(key))
            .count()
            <= MAX_ORDINARY_NAMES
}
const MAX_EXPANSION_WORK: usize = 1_000_000;
const MAX_REGEX_WORK: usize = 1_000_000;
/// Persistable diagnostics are bounded at creation, not only when rendering.
fn diagnostic(mut text: String) -> String {
    if text.len() > 1000 {
        let mut end = 997;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
        text.push_str("...");
    }
    text
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub name: String,
    pub args: Vec<String>,
    pub stdin: String,
    pub env: BTreeMap<String, String>,
    pub files: BTreeMap<String, String>,
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
    pub after: BTreeMap<String, String>,
}
impl Case {
    pub fn valid(&self) -> bool {
        !self.name.is_empty()
            && self.name.len() < 120
            && self.args.len() <= 64
            && self.env.len() <= 64
            && self
                .env
                .keys()
                .all(|k| name(k) && variable_target(k).is_ok())
            && self.files.len() <= 64
            && self.after.len() <= 64
            && self
                .files
                .keys()
                .chain(self.after.keys())
                .all(|k| path(k).is_ok_and(|p| p == *k))
            && self
                .args
                .iter()
                .chain(self.env.values())
                .chain(self.files.values())
                .chain(self.after.values())
                .all(|s| !s.contains('\0'))
            && !self.stdin.contains('\0')
            && self.stdin.len()
                + self
                    .env
                    .iter()
                    .map(|(name, value)| name.len() + value.len())
                    .sum::<usize>()
                + self
                    .files
                    .iter()
                    .map(|(name, value)| name.len() + value.len())
                    .sum::<usize>()
                + self.args.iter().map(String::len).sum::<usize>()
                < MAX_TEXT / 2
            && self.stdout.len() + self.stderr.len() <= MAX_OUTPUT
            && self
                .after
                .iter()
                .map(|(name, value)| name.len() + value.len())
                .sum::<usize>()
                < MAX_TEXT / 2
            && (0..=255).contains(&self.status)
    }
    pub fn accepts(&self, r: &Outcome) -> bool {
        r.error.is_none()
            && r.stdout == self.stdout
            && r.stderr == self.stderr
            && r.status == self.status
            && r.files == self.after
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
    pub files: BTreeMap<String, String>,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Record {
    #[serde(default, skip_serializing_if = "legacy_revision")]
    pub engine_revision: u32,
    pub source: String,
    pub results: Vec<Outcome>,
}
impl Record {
    pub fn current(&self) -> bool {
        self.engine_revision == EVALUATOR_REVISION
    }
    pub fn valid(&self) -> bool {
        self.engine_revision <= EVALUATOR_REVISION
            && self.source.len() <= MAX_SOURCE
            && self.results.len() <= MAX_CASES
            && self.results.iter().all(|r| {
                (0..=255).contains(&r.status)
                    && r.stdout.len() + r.stderr.len() <= MAX_OUTPUT
                    && r.files.len() <= 64
                    && r.files.values().map(String::len).sum::<usize>() <= MAX_TEXT
                    && r.files
                        .keys()
                        .all(|p| path(p).is_ok_and(|normalized| normalized == *p))
                    && r.error.as_ref().is_none_or(|s| s.len() <= 1000)
            })
    }
}
#[derive(Clone, Debug)]
struct Part {
    value: PartValue,
    quote: Quote,
}
#[derive(Clone, Debug)]
enum PartValue {
    Text(String),
    Substitution(BodyId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BodyId(usize);
impl Part {
    fn text(&self) -> Option<&str> {
        match &self.value {
            PartValue::Text(text) => Some(text),
            PartValue::Substitution(_) => None,
        }
    }
    fn text_mut(&mut self) -> Result<&mut String, String> {
        match &mut self.value {
            PartValue::Text(text) => Ok(text),
            PartValue::Substitution(_) => Err("Command substitution is unsupported".into()),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Quote {
    Bare,
    Single,
    Double,
}
#[derive(Clone, Debug)]
struct Word(Vec<Part>);
impl Word {
    fn assignment(&self) -> bool {
        self.0.first().is_some_and(|p| {
            p.quote == Quote::Bare
                && p.text()
                    .and_then(|s| s.split_once('='))
                    .is_some_and(|(n, _)| name(n))
        })
    }
    fn literal(&self) -> Option<&str> {
        if self.0.len() == 1 && self.0[0].quote == Quote::Bare {
            self.0[0].text()
        } else {
            None
        }
    }
}
#[derive(Clone, Debug)]
enum Token {
    Word(Word),
    Op(String),
}
fn name(s: &str) -> bool {
    let mut c = s.chars();
    c.next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && c.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
// These Bash variables have readonly/dynamic/attribute/option behavior that this subset does not model.
fn unmodeled_variable(n: &str) -> bool {
    n.starts_with("BASH")
        || n.starts_with("COMP_")
        || n.starts_with("HIST")
        || [
            "_",
            "UID",
            "EUID",
            "PPID",
            "SHELLOPTS",
            "RANDOM",
            "SRANDOM",
            "SECONDS",
            "EPOCHSECONDS",
            "EPOCHREALTIME",
            "LINENO",
            "FUNCNAME",
            "DIRSTACK",
            "GROUPS",
            "PIPESTATUS",
            "SHLVL",
            "REPLY",
            "MAPFILE",
            "PS0",
            "PS1",
            "PS2",
            "PS3",
            "PS4",
            "PROMPT_COMMAND",
            "IGNOREEOF",
            "TMOUT",
            "GLOBIGNORE",
            "GLOBSORT",
            "CDPATH",
            "FIGNORE",
            "EXECIGNORE",
            "POSIXLY_CORRECT",
        ]
        .contains(&n)
}
fn readable_variable(n: &str) -> Result<(), String> {
    if n == "IFS" {
        return Err(
            "Unsupported custom IFS; this subset uses default space, tab and newline splitting"
                .into(),
        );
    }
    if unmodeled_variable(n) {
        return Err(format!(
            "Unsupported special-variable name or reserved namespace {n:.80}; use an ordinary variable name within this subset"
        ));
    }
    Ok(())
}
fn variable_target(n: &str) -> Result<(), String> {
    readable_variable(n)?;
    if matches!(n, "OPTIND" | "OPTARG") {
        return Err(
            "Unsupported special-variable write; use scalar OPTIND=1 to reset getopts".into(),
        );
    }
    Ok(())
}
fn scalar_assignment_target(n: &str) -> Result<(), String> {
    if n == "OPTIND" {
        Ok(())
    } else {
        variable_target(n)
    }
}
fn reserved_function_name(n: &str) -> bool {
    // Keywords cannot become ordinary calls; no Bash builtin replacement is modeled.
    [
        "if",
        "then",
        "else",
        "elif",
        "fi",
        "for",
        "while",
        "until",
        "do",
        "done",
        "in",
        "case",
        "esac",
        "select",
        "function",
        "time",
        "coproc",
        "alias",
        "bg",
        "bind",
        "break",
        "builtin",
        "caller",
        "cd",
        "command",
        "compgen",
        "complete",
        "compopt",
        "continue",
        "declare",
        "dirs",
        "disown",
        "echo",
        "enable",
        "eval",
        "exec",
        "exit",
        "export",
        "false",
        "fc",
        "fg",
        "getopts",
        "hash",
        "help",
        "history",
        "jobs",
        "kill",
        "let",
        "local",
        "logout",
        "mapfile",
        "popd",
        "printf",
        "pushd",
        "pwd",
        "read",
        "readarray",
        "readonly",
        "return",
        "set",
        "shift",
        "shopt",
        "source",
        "suspend",
        "test",
        "times",
        "trap",
        "true",
        "type",
        "typeset",
        "ulimit",
        "umask",
        "unalias",
        "unset",
        "wait",
        "cat",
        "grep",
    ]
    .contains(&n)
}
fn lex_unit(
    source: &str,
    compiler: &mut Compiler,
    subst_depth: usize,
) -> Result<Vec<Token>, String> {
    if source.len() > MAX_SOURCE {
        return Err("Script exceeds 16 KiB".into());
    }
    if source.contains('\0') {
        return Err("NUL data is unsupported".into());
    }
    let cs: Vec<char> = source.chars().collect();
    let mut i = 0;
    let mut tokens = Vec::new();
    let mut parts = Vec::new();
    let mut bare = String::new();
    fn flush(parts: &mut Vec<Part>, bare: &mut String) {
        if !bare.is_empty() {
            parts.push(Part {
                value: PartValue::Text(std::mem::take(bare)),
                quote: Quote::Bare,
            })
        }
    }
    fn word(
        tokens: &mut Vec<Token>,
        parts: &mut Vec<Part>,
        bare: &mut String,
        compiler: &mut Compiler,
    ) -> Result<(), String> {
        flush(parts, bare);
        if !parts.is_empty() {
            compiler.token(tokens, Token::Word(Word(std::mem::take(parts))))?;
        }
        Ok(())
    }
    while i < cs.len() {
        let c = cs[i];
        if c == '#' && parts.is_empty() && bare.is_empty() {
            while i < cs.len() && cs[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if matches!(c, '<' | '>') && cs.get(i + 1) == Some(&'(') {
            return Err("Unsupported process substitution".into());
        }
        if c == '\r' {
            return Err(
                "Use LF script line breaks; carriage returns outside quotes are unsupported".into(),
            );
        }
        if c == ' ' || c == '\t' {
            word(&mut tokens, &mut parts, &mut bare, compiler)?;
            i += 1;
            continue;
        }
        if c == '\'' || c == '"' {
            flush(&mut parts, &mut bare);
            let quote = if c == '\'' {
                Quote::Single
            } else {
                Quote::Double
            };
            i += 1;
            let mut text = String::new();
            let quote_start = parts.len();
            let mut closed = false;
            while i < cs.len() {
                if quote == Quote::Double && cs[i..].starts_with(&['$', '(', '(']) {
                    let end = substitution::arithmetic_end(&cs, i)?;
                    text.extend(cs[i..end].iter());
                    i = end;
                    continue;
                }
                if quote == Quote::Double
                    && let Some(body_start) = substitution::body_start(&cs, i)
                {
                    if !text.is_empty() {
                        parts.push(Part {
                            value: PartValue::Text(std::mem::take(&mut text)),
                            quote,
                        });
                    }
                    let end = substitution::end(&cs, i, subst_depth + 1)?;
                    let body: String = cs[body_start..end - 1].iter().collect();
                    let body = compiler.body(&body, subst_depth + 1)?;
                    parts.push(Part {
                        value: PartValue::Substitution(body),
                        quote,
                    });
                    i = end;
                    continue;
                }
                if quote == Quote::Double && cs[i..].starts_with(&['$', '{']) {
                    let end = substitution::parameter_end(&cs, i)?;
                    text.extend(cs[i..end].iter());
                    i = end;
                    continue;
                }
                if cs[i] == c {
                    closed = true;
                    i += 1;
                    break;
                }
                if cs[i] == '`' && quote != Quote::Single {
                    return Err("Backtick substitution is unsupported".into());
                }
                if cs[i] == '\\' && quote == Quote::Double {
                    i += 1;
                    if i == cs.len() {
                        return Err("Unfinished escape".into());
                    }
                    let x = cs[i];
                    if x == '\n' {
                        i += 1;
                        continue;
                    }
                    if matches!(x, '$' | '`' | '"' | '\\') {
                        // Escaped expanding characters become literal parts.
                        if !text.is_empty() {
                            parts.push(Part {
                                value: PartValue::Text(std::mem::take(&mut text)),
                                quote,
                            });
                        }
                        parts.push(Part {
                            value: PartValue::Text(x.to_string()),
                            quote: Quote::Single,
                        });
                        i += 1;
                        continue;
                    }
                    text.push('\\');
                }
                text.push(cs[i]);
                i += 1;
            }
            if !closed {
                return Err("Close the quoted string before running".into());
            }
            if !text.is_empty() || parts.len() == quote_start {
                parts.push(Part {
                    value: PartValue::Text(text),
                    quote,
                });
            }
            continue;
        }
        if c == '\\' {
            i += 1;
            if i == cs.len() {
                return Err("Unfinished escape".into());
            }
            if cs[i] != '\n' {
                flush(&mut parts, &mut bare);
                parts.push(Part {
                    value: PartValue::Text(cs[i].to_string()),
                    quote: Quote::Single,
                });
            }
            i += 1;
            continue;
        }
        if c == '`' || (matches!(c, '{' | '}') && (!bare.is_empty() || !parts.is_empty())) {
            // Braces inside ${name} are consumed below.
            return Err(format!(
                "Unsupported syntax: {c}. See the supported shell subset"
            ));
        }
        if c == '$' && cs[i..].starts_with(&['$', '(', '(']) {
            let end = substitution::arithmetic_end(&cs, i)?;
            bare.extend(cs[i..end].iter());
            i = end;
            continue;
        }
        if let Some(body_start) = substitution::body_start(&cs, i) {
            flush(&mut parts, &mut bare);
            let end = substitution::end(&cs, i, subst_depth + 1)?;
            let body: String = cs[body_start..end - 1].iter().collect();
            let body = compiler.body(&body, subst_depth + 1)?;
            parts.push(Part {
                value: PartValue::Substitution(body),
                quote: Quote::Bare,
            });
            i = end;
            continue;
        }
        if c == '$' && cs.get(i + 1) == Some(&'{') {
            let end = substitution::parameter_end(&cs, i)?;
            bare.extend(cs[i..end].iter());
            i = end;
            continue;
        }
        if c == '(' && bare.ends_with('$') {
            return Err("Command substitution is unsupported".into());
        }
        if c == '\n'
            || matches!(c, '(' | ')' | ';' | '|' | '&' | '<' | '>')
            || (matches!(c, '1' | '2')
                && bare.is_empty()
                && parts.is_empty()
                && cs.get(i + 1) == Some(&'>'))
        {
            let adjacent_array = c == '(' && (!bare.is_empty() || !parts.is_empty());
            word(&mut tokens, &mut parts, &mut bare, compiler)?;
            let array_opener = adjacent_array
                && matches!(tokens.last(), Some(Token::Word(w)) if w.literal().is_some_and(|s| s.ends_with('=')));
            if c == '\n' {
                compiler.token(&mut tokens, Token::Op("\n".into()))?;
                i += 1;
                continue;
            }
            let mut op = if array_opener {
                "=(".to_string()
            } else {
                c.to_string()
            };
            i += 1;
            if c == '1' || c == '2' {
                op.push('>');
                i += 1;
            }
            if i < cs.len()
                && ((cs[i] == c && matches!(c, ';' | '|' | '&' | '>' | '<'))
                    || (cs[i] == '>' && op.ends_with('>')))
            {
                op.push(cs[i]);
                i += 1;
            }
            if c == ';' && cs.get(i) == Some(&'&') {
                return Err("Unsupported case fallthrough operator ;& or ;;&".into());
            }
            if op.ends_with('>') && cs.get(i) == Some(&'&') {
                op.push('&');
                i += 1;
                if let Some(x @ ('1' | '2')) = cs.get(i) {
                    op.push(*x);
                    i += 1;
                } else {
                    return Err("Only >&2 and 2>&1 descriptor copies are supported".into());
                }
            }
            if ![
                "=(", "(", ")", ";", ";;", "|", "&&", "||", "<", ">", ">>", "1>", "1>>", "2>",
                "2>>", ">&2", "1>&2", "2>&1",
            ]
            .contains(&op.as_str())
            {
                return Err(format!("Unsupported operator: {op}"));
            }
            compiler.token(&mut tokens, Token::Op(op))?;
            continue;
        }
        if c == '~' {
            return Err(
                "Tilde expansion is unsupported; use an ordinary virtual /work path".into(),
            );
        }
        bare.push(c);
        i += 1;
    }
    word(&mut tokens, &mut parts, &mut bare, compiler)?;
    if tokens.len() > 4096 {
        return Err("Script exceeds 4096 tokens".into());
    }
    Ok(tokens)
}
#[derive(Clone, Debug)]
struct Command {
    words: Vec<Word>,
    redirects: Vec<(String, Option<Word>)>,
}
#[derive(Clone, Debug)]
enum Code {
    Pipe(Vec<Command>),
    Case(Arc<case::Definition>),
    ArrayAssign(String, Vec<Word>),
    Jump(usize),
    JumpIf(bool, usize),
    For(String, Vec<Word>, usize),
    Next(usize),
    Status(i32),
    WhileStart(Option<Word>, usize),
    WhileBody,
    WhileEnd,
    Define(String, usize, usize),
    FunctionEnd,
}
/// One immutable instruction arena keeps parent/child function targets stable.
#[derive(Debug)]
struct Program {
    code: Vec<Code>,
    main: Range<usize>,
    bodies: Vec<Range<usize>>,
    loop_sites: Vec<loop_control::Site>,
}
impl Program {
    fn text_bytes(&self) -> usize {
        let words = |words: &[Word]| {
            words
                .iter()
                .flat_map(|word| &word.0)
                .map(|part| part.text().map_or(0, str::len))
                .sum::<usize>()
        };
        self.code
            .iter()
            .map(|code| match code {
                Code::Pipe(commands) => commands
                    .iter()
                    .map(|command| {
                        words(&command.words)
                            + command
                                .redirects
                                .iter()
                                .map(|(op, word)| {
                                    op.len()
                                        + word
                                            .as_ref()
                                            .map_or(0, |word| words(std::slice::from_ref(word)))
                                })
                                .sum::<usize>()
                    })
                    .sum(),
                Code::ArrayAssign(key, values) | Code::For(key, values, _) => {
                    key.len() + words(values)
                }
                Code::Case(definition) => definition.text_bytes(),
                Code::Define(key, _, _) => key.len(),
                Code::WhileStart(Some(word), _) => words(std::slice::from_ref(word)),
                _ => 0,
            })
            .sum::<usize>()
            + self.loop_sites.capacity() * std::mem::size_of::<loop_control::Site>()
    }
}
struct Unit {
    code: Vec<Code>,
    syntax_depth: usize,
}
#[derive(Default)]
struct Compiler {
    tokens: usize,
    sites: usize,
    bodies: Vec<Unit>,
}
impl Compiler {
    fn token(&mut self, tokens: &mut Vec<Token>, token: Token) -> Result<(), String> {
        if self.tokens >= 4096 {
            return Err("Script exceeds 4096 tokens across all command-substitution bodies".into());
        }
        self.tokens += 1;
        tokens.push(token);
        Ok(())
    }
    fn body(&mut self, source: &str, depth: usize) -> Result<BodyId, String> {
        if depth > substitution::MAX_DEPTH {
            return Err("Command substitution depth limit exceeded (8)".into());
        }
        if self.sites >= substitution::MAX_SITES {
            return Err("Command substitution site limit exceeded (128)".into());
        }
        self.sites += 1;
        let unit = parse_unit(source, self, depth)?;
        // Bash treats this exact AST shape as file-content capture, even with
        // trailing comments/terminators. Two redirects are an ordinary command.
        if matches!(unit.code.as_slice(), [Code::Pipe(commands)]
            if matches!(commands.as_slice(), [Command { words, redirects }]
                if words.is_empty() && matches!(redirects.as_slice(), [(op, Some(_))] if op == "<")))
        {
            return Err("Unsupported Bash input-only command substitution; use the documented virtual file command".into());
        }
        let id = BodyId(self.bodies.len());
        self.bodies.push(unit);
        Ok(id)
    }
    fn finish(self, main: Unit) -> Result<Arc<Program>, String> {
        let main_end = main.code.len();
        let mut program = Program {
            code: main.code,
            main: 0..main_end,
            bodies: Vec::new(),
            loop_sites: Vec::new(),
        };
        for body in self.bodies {
            let start = program.code.len();
            for mut instruction in body.code {
                match &mut instruction {
                    Code::Jump(pc)
                    | Code::JumpIf(_, pc)
                    | Code::For(_, _, pc)
                    | Code::Next(pc)
                    | Code::WhileStart(_, pc) => *pc += start,
                    Code::Case(definition) => {
                        let definition = Arc::get_mut(definition).expect("unique compiled case");
                        definition.end += start;
                        for arm in &mut definition.arms {
                            arm.entry += start;
                        }
                    }
                    Code::Define(_, entry, end) => {
                        *entry += start;
                        *end += start;
                    }
                    _ => {}
                }
                program.code.push(instruction);
            }
            program.bodies.push(start..program.code.len());
        }
        program.loop_sites = loop_control::sites(&program)?;
        Ok(Arc::new(program))
    }
}
struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    code: Vec<Code>,
    depth: usize,
    syntax_depth: usize,
    body_depths: &'a [usize],
}
impl Parser<'_> {
    fn note_word(&mut self, word: &Word) -> Result<(), String> {
        case::check_static_word(word)?;
        self.note_scalar(word)
    }
    fn note_scalar(&mut self, word: &Word) -> Result<(), String> {
        for part in &word.0 {
            if let PartValue::Substitution(BodyId(id)) = &part.value {
                let child = self
                    .body_depths
                    .get(*id)
                    .ok_or("Invalid compiled substitution body")?;
                let total = self.depth + child;
                if total > 24 {
                    return Err(
                        "Syntax nesting exceeds 24 levels across command substitutions".into(),
                    );
                }
                self.syntax_depth = self.syntax_depth.max(total);
            }
        }
        Ok(())
    }
    fn is(&self, s: &str) -> bool {
        match self.tokens.get(self.pos) {
            Some(Token::Word(w)) => w.literal() == Some(s),
            Some(Token::Op(o)) => o == s,
            None => false,
        }
    }
    fn eat(&mut self, s: &str) -> bool {
        if self.is(s) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, s: &str) -> Result<(), String> {
        if self.eat(s) {
            Ok(())
        } else {
            Err(format!("Expected {s} near token {}", self.pos + 1))
        }
    }
    fn patch(&mut self, i: usize) {
        let end = self.code.len();
        match &mut self.code[i] {
            Code::Jump(t) | Code::JumpIf(_, t) | Code::For(_, _, t) => *t = end,
            _ => unreachable!(),
        }
    }
    fn list(&mut self, stops: &[&str]) -> Result<(), String> {
        self.depth += 1;
        if self.depth > 24 {
            return Err("Syntax nesting exceeds 24 levels".into());
        }
        self.syntax_depth = self.syntax_depth.max(self.depth);
        while self.eat("\n") {}
        while self.pos < self.tokens.len() && !stops.iter().any(|s| self.is(s)) {
            self.compound()?;
            while self.is("&&") || self.is("||") {
                let skip = self.eat("||");
                if !skip {
                    self.expect("&&")?;
                }
                while self.eat("\n") {}
                let idx = self.code.len();
                self.code.push(Code::JumpIf(skip, 0));
                self.compound()?;
                self.patch(idx);
            }
            if self.pos < self.tokens.len()
                && !(stops.contains(&";;") && self.is(";;"))
                && !self.eat(";")
                && !self.eat("\n")
            {
                return Err(format!("Expected newline or ; near token {}", self.pos + 1));
            }
            while self.eat("\n") {}
        }
        self.depth -= 1;
        Ok(())
    }
    fn body(&mut self, stops: &[&str]) -> Result<(), String> {
        let start = self.code.len();
        self.list(stops)?;
        if start == self.code.len() {
            return Err(
                "A then/do/else block needs a command. Use : for an intentional no-op.".into(),
            );
        }
        Ok(())
    }
    fn compound(&mut self) -> Result<(), String> {
        if self.eat("case") {
            let Some(Token::Word(selector)) = self.tokens.get(self.pos) else {
                return Err("case needs one selector word".into());
            };
            let selector = selector.clone();
            self.note_scalar(&selector)?;
            self.pos += 1;
            while self.eat("\n") {}
            self.expect("in")?;
            while self.eat("\n") {}
            let instruction = self.code.len();
            self.code.push(Code::Status(0)); // replaced by immutable descriptor
            let mut arms = Vec::new();
            let mut exits = Vec::new();
            let mut alternatives = 0;
            while !self.eat("esac") {
                if arms.len() == 32 {
                    return Err("Case arm limit exceeded (32)".into());
                }
                self.eat("(");
                let mut patterns = Vec::new();
                loop {
                    let Some(Token::Word(pattern)) = self.tokens.get(self.pos) else {
                        return Err("Case arm needs a pattern before )".into());
                    };
                    let pattern = pattern.clone();
                    case::check_static_pattern(&pattern)?;
                    self.note_scalar(&pattern)?;
                    patterns.push(pattern);
                    self.pos += 1;
                    alternatives += 1;
                    if patterns.len() > 8 || alternatives > 128 {
                        return Err(
                            "Case alternative limit exceeded (8 per arm,128 per case)".into()
                        );
                    }
                    if !self.eat("|") {
                        break;
                    }
                    while self.eat("\n") {}
                }
                self.expect(")")?;
                let entry = self.code.len();
                self.list(&[";;", "esac"])?;
                if self.code.len() == entry {
                    self.code.push(Code::Status(0));
                }
                arms.push(case::Arm { patterns, entry });
                exits.push(self.code.len());
                self.code.push(Code::Jump(0));
                if !self.eat(";;") {
                    self.expect("esac")?;
                    break;
                }
                while self.eat("\n") {}
            }
            let end = self.code.len();
            for exit in exits {
                self.patch(exit);
            }
            self.code[instruction] = Code::Case(Arc::new(case::Definition {
                selector,
                arms,
                end,
            }));
            if matches!(self.tokens.get(self.pos),Some(Token::Op(op)) if op == "|" || op.contains('>') || op == "<")
            {
                return Err("Unsupported pipeline or redirection on whole case; redirect commands inside an arm".into());
            }
            return Ok(());
        }
        if let (Some(Token::Word(word)), Some(Token::Op(op))) =
            (self.tokens.get(self.pos), self.tokens.get(self.pos + 1))
            && op == "=("
            && let Some(text) = word.literal()
            && text.ends_with('=')
        {
            let key = text.strip_suffix('=').unwrap();
            if !name(key) {
                return Err("Unsupported array assignment; use a standalone name=(words) without += or indices".into());
            }
            variable_target(key)?;
            let key = key.to_string();
            self.pos += 2;
            let mut words = Vec::new();
            loop {
                match self.tokens.get(self.pos) {
                    Some(Token::Word(word)) => { let word = word.clone(); self.note_word(&word)?; words.push(word); self.pos+=1;}
                    Some(Token::Op(op)) if op == "\n" => self.pos+=1,
                    Some(Token::Op(op)) if op == ")" => {self.pos+=1;break;}
                    None => return Err("Close the compound array assignment with )".into()),
                    _ => return Err("Unsupported array element syntax: use ordinary words, quotes and supported expansions, without indexed writes or nested commands".into()),
                }
                if words.len() > 128 {
                    return Err("Array assignment exceeds128 source elements".into());
                }
            }
            if matches!(self.tokens.get(self.pos), Some(Token::Word(_)))
                || matches!(self.tokens.get(self.pos),Some(Token::Op(op)) if op == "|" || op.contains('>') || op == "<")
            {
                return Err("Unsupported compound array assignment with a command, pipeline or redirection; put it on its own line".into());
            }
            self.code.push(Code::ArrayAssign(key, words));
            return Ok(());
        }
        let function = match (
            self.tokens.get(self.pos),
            self.tokens.get(self.pos + 1),
            self.tokens.get(self.pos + 2),
        ) {
            (Some(Token::Word(w)), Some(Token::Op(a)), Some(Token::Op(b)))
                if a == "(" && b == ")" =>
            {
                w.literal().filter(|n| name(n)).map(str::to_owned)
            }
            _ => None,
        };
        if let Some(n) = function {
            if reserved_function_name(&n) {
                return Err("Unsupported function name: builtin and keyword replacement is outside this subset".into());
            }
            self.pos += 3;
            while self.eat("\n") {}
            self.expect("{")?;
            let def = self.code.len();
            self.code.push(Code::Define(n.clone(), def + 1, 0));
            self.body(&["}"])?;
            self.expect("}")?;
            self.code.push(Code::FunctionEnd);
            let end = self.code.len();
            self.code[def] = Code::Define(n, def + 1, end);
            return Ok(());
        }
        if self.is("(") || self.is("{") || self.is(")") || self.is("}") {
            return Err("Unsupported group/subshell syntax; only name() { commands; } function definitions are supported".into());
        }
        if self.eat("if") {
            let start = self.code.len();
            self.list(&["then"])?;
            if start == self.code.len() {
                return Err("if needs a condition".into());
            }
            self.expect("then")?;
            let branch = self.code.len();
            self.code.push(Code::JumpIf(false, 0));
            self.body(&["else", "fi"])?;
            if self.eat("else") {
                let jump = self.code.len();
                self.code.push(Code::Jump(0));
                self.patch(branch);
                self.body(&["fi"])?;
                self.expect("fi")?;
                self.patch(jump);
            } else {
                self.expect("fi")?;
                let jump = self.code.len();
                self.code.push(Code::Jump(0));
                self.patch(branch);
                self.code.push(Code::Status(0));
                self.patch(jump);
            }
            return Ok(());
        }
        if self.eat("while") {
            let entry = self.code.len();
            self.code.push(Code::WhileStart(None, 0));
            let start = self.code.len();
            self.list(&["do"])?;
            if start == self.code.len() {
                return Err("while needs a condition".into());
            }
            self.expect("do")?;
            let branch = self.code.len();
            self.code.push(Code::JumpIf(false, 0));
            self.body(&["done"])?;
            self.expect("done")?;
            let input = if self.eat("<") {
                let Some(Token::Word(word)) = self.tokens.get(self.pos) else {
                    return Err("Loop input redirection needs a file path".into());
                };
                let word = word.clone();
                self.note_word(&word)?;
                self.pos += 1;
                Some(word)
            } else {
                None
            };
            if matches!(self.tokens.get(self.pos), Some(Token::Op(op)) if op.contains('>') || op == "<")
            {
                return Err("Unsupported loop redirection; only one trailing < FILE on a while loop is supported".into());
            }
            self.code.push(Code::WhileBody);
            self.code.push(Code::Jump(start));
            self.patch(branch);
            self.code.push(Code::WhileEnd);
            self.code[entry] = Code::WhileStart(input, self.code.len());
            return Ok(());
        }
        if self.eat("for") {
            let Some(Token::Word(w)) = self.tokens.get(self.pos) else {
                return Err("for needs a variable name".into());
            };
            let n = w
                .literal()
                .filter(|s| name(s))
                .ok_or("Invalid for variable")?
                .to_string();
            variable_target(&n)?;
            self.pos += 1;
            self.expect("in")?;
            let mut words = Vec::new();
            while let Some(Token::Word(w)) = self.tokens.get(self.pos) {
                let word = w.clone();
                self.note_word(&word)?;
                words.push(word);
                self.pos += 1;
            }
            if !self.eat(";") && !self.eat("\n") {
                return Err("for needs a newline or ; before do".into());
            }
            while self.eat("\n") {}
            self.expect("do")?;
            let branch = self.code.len();
            self.code.push(Code::For(n, words, 0));
            let start = self.code.len();
            self.body(&["done"])?;
            self.expect("done")?;
            self.code.push(Code::Next(start));
            self.patch(branch);
            return Ok(());
        }
        if ["then", "else", "fi", "do", "done", "esac"]
            .iter()
            .any(|s| self.is(s))
        {
            return Err(
                "Syntax error: unexpected then/else/fi/do/done; check the matching if or loop"
                    .into(),
            );
        }
        if ["until", "select", "function", "!"]
            .iter()
            .any(|s| self.is(s))
        {
            return Err("Unsupported shell keyword".into());
        }
        let mut pipeline = vec![self.command()?];
        while self.eat("|") {
            while self.eat("\n") {}
            pipeline.push(self.command()?);
            if pipeline.len() > 16 {
                return Err("Pipeline exceeds 16 stages".into());
            }
        }
        self.code.push(Code::Pipe(pipeline));
        Ok(())
    }
    fn command(&mut self) -> Result<Command, String> {
        let mut words = Vec::new();
        let mut redirects = Vec::new();
        loop {
            match self.tokens.get(self.pos) {
                Some(Token::Word(w)) => {
                    let word = w.clone();
                    self.note_word(&word)?;
                    words.push(word);
                    self.pos += 1;
                }
                Some(Token::Op(op)) if op.contains('>') || op == "<" => {
                    let op = op.clone();
                    self.pos += 1;
                    let w = if op.contains('&') {
                        None
                    } else {
                        let Some(Token::Word(w)) = self.tokens.get(self.pos) else {
                            return Err("Redirection needs a file path".into());
                        };
                        let word = w.clone();
                        self.note_word(&word)?;
                        self.pos += 1;
                        Some(word)
                    };
                    redirects.push((op, w));
                }
                _ => break,
            }
        }
        // An unquoted reserved elif at a command head is excluded syntax, not
        // an executable command. Quoted/escaped/expanded heads and later words
        // keep their data roles, including redirection filenames.
        if words.first().and_then(Word::literal) == Some("elif") {
            return Err(
                "Unsupported elif clause; use nested if/else blocks in this simulator".into(),
            );
        }
        // compound() owns reserved case syntax. Subsequent pipeline stages and
        // leading redirects must not disguise a bare reserved head as a simple
        // command. Quotes/escapes/expansion and later arguments remain data.
        if words
            .first()
            .and_then(Word::literal)
            .is_some_and(|head| matches!(head, "case" | "esac"))
        {
            return Err("Unsupported case/esac command head; case compounds cannot be pipeline stages or follow leading redirects".into());
        }
        if words.is_empty() && redirects.is_empty() {
            return Err("Expected a command".into());
        }
        if words.len() > 128 || redirects.len() > 64 {
            return Err("Command exceeds argument/redirection limit".into());
        }
        Ok(Command { words, redirects })
    }
}
fn classified(message: String) -> String {
    if message.to_ascii_lowercase().contains("unsupported")
        || message.contains("exceeds")
        || message.contains("limit")
        || message.starts_with("Syntax error:")
    {
        message
    } else {
        format!("Syntax error: {message}")
    }
}
fn parse_unit(source: &str, compiler: &mut Compiler, subst_depth: usize) -> Result<Unit, String> {
    let tokens = lex_unit(source, compiler, subst_depth)?;
    let body_depths: Vec<_> = compiler.bodies.iter().map(|b| b.syntax_depth).collect();
    let mut parser = Parser {
        tokens,
        pos: 0,
        code: Vec::new(),
        depth: 0,
        syntax_depth: 0,
        body_depths: &body_depths,
    };
    parser.list(&[])?;
    Ok(Unit {
        code: parser.code,
        syntax_depth: parser.syntax_depth,
    })
}
fn compile(source: &str) -> Result<Arc<Program>, String> {
    let mut compiler = Compiler::default();
    let main = parse_unit(source, &mut compiler, 0).map_err(classified)?;
    compiler.finish(main)
}
/// The separate terminal remains literal-only; parsed child programs never run.
fn lex(source: &str) -> Result<Vec<Token>, String> {
    let mut compiler = Compiler::default();
    let tokens = lex_unit(source, &mut compiler, 0)?;
    if !compiler.bodies.is_empty() {
        return Err("Command substitution is unsupported in this terminal".into());
    }
    for token in &tokens {
        if let Token::Word(word) = token {
            case::check_static_word(word)?;
        }
    }
    Ok(tokens)
}
fn path(s: &str) -> Result<String, String> {
    let suffix = if let Some(suffix) = s.strip_prefix("/work/") {
        suffix
    } else if s.starts_with('/') {
        return Err("Only ordinary virtual paths under /work are supported".into());
    } else {
        s
    };
    if suffix.contains(['/', '\0']) || matches!(suffix, "" | "." | "..") || suffix.len() > 250 {
        return Err("Only ordinary virtual paths under /work are supported".into());
    }
    Ok(if s.starts_with('/') {
        s.to_owned()
    } else {
        format!("/work/{s}")
    })
}
#[derive(Clone)]
enum Dest {
    Out,
    Err,
    File(String),
    Buffer(capture::BufferId),
}
fn bounded_expansion_bytes(current: usize, additional: usize) -> Result<usize, String> {
    current
        .checked_add(additional)
        .filter(|n| *n <= MAX_TEXT)
        .ok_or_else(|| "Expansion memory limit reached (256 KiB of expanded text)".into())
}
fn expansion_fields_size(fields: &[String]) -> Result<usize, String> {
    if fields.len() > 128 {
        return Err("Argument limit exceeded (128 expanded fields)".into());
    }
    fields
        .iter()
        .try_fold(0, |n, s| bounded_expansion_bytes(n, s.len()))
}
fn strings_bytes(values: &[String]) -> usize {
    values.iter().map(String::len).sum::<usize>() + std::mem::size_of_val(values)
}
fn decimal_bytes(value: i64) -> usize {
    value.unsigned_abs().checked_ilog10().unwrap_or(0) as usize + 1 + usize::from(value < 0)
}
fn word_bytes(word: &Word) -> usize {
    word.0.iter().map(part_bytes).sum()
}
fn part_bytes(part: &Part) -> usize {
    std::mem::size_of::<Part>() + part.text().map_or(0, str::len)
}
fn command_bytes(command: &Command) -> usize {
    command.words.iter().map(word_bytes).sum::<usize>()
        + command
            .redirects
            .iter()
            .map(|(op, word)| op.len() + word.as_ref().map_or(0, word_bytes))
            .sum::<usize>()
}
struct Scratch<'a> {
    counter: &'a Cell<usize>,
    bytes: usize,
}
struct Expanded<'a> {
    text: String,
    memory: Scratch<'a>,
}
impl Scratch<'_> {
    fn grow(&mut self, vm: &Vm, bytes: usize) -> Result<(), String> {
        vm.reserve_memory(bytes)?;
        self.counter.set(self.counter.get() + bytes);
        self.bytes += bytes;
        Ok(())
    }
}
impl Drop for Scratch<'_> {
    fn drop(&mut self) {
        self.counter.set(self.counter.get() - self.bytes);
    }
}
type LocalDeclaration = (String, Option<String>);
struct InputFrame {
    input: String,
    pos: usize,
    file: String,
    id: usize,
}
struct CallFrame {
    pc: usize,
    args: Vec<String>,
    loops: usize,
    whiles: usize,
    inputs: usize,
    locals: BTreeMap<String, Option<Value>>,
    scanner: Option<getopts::Scanner>,
}
pub struct Vm {
    program: Arc<Program>,
    program_bytes: usize,
    pc: usize,
    range: Range<usize>,
    frame_executed: bool,
    steps: usize,
    expansion_work: Cell<usize>,
    regex_work: Cell<usize>,
    env: BTreeMap<String, Value>,
    scanner: getopts::Scanner,
    pipefail: bool,
    args: Vec<String>,
    input: String,
    input_pos: usize,
    input_id: usize,
    next_input_id: usize,
    loops: Vec<(String, Vec<String>, usize)>,
    while_status: Vec<(i32, bool)>,
    inputs: Vec<InputFrame>,
    functions: BTreeMap<String, usize>,
    calls: Vec<CallFrame>,
    pub outcome: Outcome,
    done: bool,
    pending_bytes: Cell<usize>,
    pending_output_bytes: Cell<usize>,
    scratch_bytes: Cell<usize>,
    ordinary_inputs: Vec<String>,
    pending: Option<execution::Pending>,
    base_stdout: Dest,
    base_stderr: Dest,
    frames: Vec<capture::Suspended>,
    buffers: Vec<Option<String>>,
    scope_exit: Option<i32>,
    capture_request: Cell<Option<BodyId>>,
}
impl Vm {
    #[cfg(test)]
    pub(crate) fn active_capture_depth(&self) -> usize {
        self.frames.len()
    }
    #[cfg(test)]
    pub(crate) fn active_getopts_scan(&self) -> bool {
        self.scanner.active()
    }
    pub fn new(source: &str, case: &Case) -> Result<Self, String> {
        if !case.valid() {
            return Err("Invalid simulation fixture".into());
        }
        let program = compile(source)?;
        Self::from_program(program, case)
    }
    fn from_program(program: Arc<Program>, case: &Case) -> Result<Self, String> {
        let range = program.main.clone();
        let program_bytes = program.text_bytes();
        Ok(Self {
            program,
            program_bytes,
            pc: range.start,
            range,
            frame_executed: false,
            steps: 0,
            expansion_work: Cell::new(0),
            regex_work: Cell::new(0),
            env: case
                .env
                .iter()
                .filter(|(k, _)| k.as_str() != "OPTERR")
                .map(|(k, v)| (k.clone(), Value::Scalar(v.clone())))
                .chain([
                    ("OPTIND".into(), Value::Scalar("1".into())),
                    ("OPTERR".into(), Value::Scalar("1".into())),
                ])
                .collect(),
            scanner: getopts::Scanner::default(),
            pipefail: false,
            args: case.args.clone(),
            input: case.stdin.clone(),
            input_pos: 0,
            input_id: 0,
            next_input_id: 1,
            loops: Vec::new(),
            while_status: Vec::new(),
            inputs: Vec::new(),
            functions: BTreeMap::new(),
            calls: Vec::new(),
            outcome: Outcome {
                stdout: String::new(),
                stderr: String::new(),
                status: 0,
                files: case.files.clone(),
                error: None,
            },
            done: false,
            pending_bytes: Cell::new(0),
            pending_output_bytes: Cell::new(0),
            scratch_bytes: Cell::new(0),
            ordinary_inputs: Vec::new(),
            pending: None,
            base_stdout: Dest::Out,
            base_stderr: Dest::Err,
            frames: Vec::new(),
            buffers: Vec::new(),
            scope_exit: None,
            capture_request: Cell::new(None),
        })
    }
    pub fn done(&self) -> bool {
        self.done
    }
    pub fn stop(&mut self) {
        self.fail("Stopped: this run was not graded".into())
    }
    fn fail(&mut self, message: String) {
        self.pending = None;
        self.pending_bytes.set(0);
        self.pending_output_bytes.set(0);
        self.frames.clear();
        self.buffers.clear();
        self.calls.clear();
        self.loops.clear();
        self.while_status.clear();
        self.inputs.clear();
        self.functions.clear();
        self.env.clear();
        self.scanner = getopts::Scanner::default();
        self.args.clear();
        self.input.clear();
        self.base_stdout = Dest::Out;
        self.base_stderr = Dest::Err;
        self.scope_exit = None;
        self.capture_request.set(None);
        self.ordinary_inputs.clear();
        self.outcome.error = Some(diagnostic(message));
        self.outcome.status = 2;
        self.done = true;
    }
    // One work ledger for every new ERE utility call/line/capture, never reset by
    // loop/function/substitution/pipeline context changes. Old expansion accounting
    // and semantic instruction ticks are intentionally unchanged.
    fn charge_regex(&self, units: usize) -> Result<(), String> {
        let next=self.regex_work.get().checked_add(units).filter(|v|*v<=MAX_REGEX_WORK)
            .ok_or("Regular-expression work limit reached (1,000,000 bounded units across this run); simplify input or Stop")?;
        self.regex_work.set(next);
        Ok(())
    }
    /// Charge all expansion paths, including preflight and arithmetic callbacks, before copying.
    fn charge_expansion(&self, bytes: usize) -> Result<(), String> {
        let next = self.expansion_work.get().checked_add(bytes)
            .filter(|n| *n <= MAX_EXPANSION_WORK)
            .ok_or("Expansion work limit reached (1,000,000 processed bytes). Reduce repeated large expansions; Stop is also available.")?;
        self.expansion_work.set(next);
        Ok(())
    }
    fn append_expansion(
        &self,
        output: &mut String,
        memory: &mut Scratch<'_>,
        text: &str,
    ) -> Result<(), String> {
        bounded_expansion_bytes(output.len(), text.len())?;
        self.charge_expansion(text.len())?;
        memory.grow(self, text.len())?;
        output.push_str(text);
        Ok(())
    }
    fn scratch(&self, bytes: usize) -> Result<Scratch<'_>, String> {
        self.reserve_memory(bytes)?;
        self.scratch_bytes.set(self.scratch_bytes.get() + bytes);
        Ok(Scratch {
            counter: &self.scratch_bytes,
            bytes,
        })
    }
    fn virtual_path(&self, value: &str) -> Result<Expanded<'_>, String> {
        let bytes = if value.starts_with('/') {
            value.len()
        } else {
            value.len().saturating_add(6)
        };
        let memory = self.scratch(bytes.min(256))?;
        Ok(Expanded {
            text: path(value)?,
            memory,
        })
    }
    fn rebase_pending(&self, old: usize, new: usize) -> Result<(), String> {
        let remaining = self
            .pending_bytes
            .get()
            .checked_sub(old)
            .ok_or("Invalid pending memory ownership")?;
        self.pending_bytes.set(remaining);
        if let Err(error) = self.reserve_memory(new) {
            self.pending_bytes.set(remaining + old);
            return Err(error);
        }
        self.pending_bytes.set(remaining + new);
        Ok(())
    }
    fn release_pending(&self, bytes: usize) -> Result<(), String> {
        self.rebase_pending(bytes, 0)
    }
    fn variable_view(&self, key: &str) -> Result<Cow<'_, str>, String> {
        if let Some(parameter) = values::array_ref(key)? {
            let value: Cow<'_, str> = match parameter {
                ArrayRef::Element(base,index) => Cow::Borrowed(self.env.get(base).and_then(|v|v.elements().get(index as usize)).map_or("",String::as_str)),
                ArrayRef::Count(base) => {
                    let count = self.env.get(base).map_or(0, |v| v.elements().len());
                    self.reserve_memory(decimal_bytes(count as i64))?;
                    Cow::Owned(count.to_string())
                },
                ArrayRef::All(_) => return Err("Unsupported mixed or unquoted array expansion; use sole quoted \"${name[@]}\" as one complete word".into()),
            };
            bounded_expansion_bytes(0, value.len())?;
            self.charge_expansion(value.len())?;
            return Ok(value);
        }
        if unmodeled_variable(key) {
            return Err(format!("Unsupported special variable: {key:.80}"));
        }
        let value: Cow<'_, str> = match key {
            "?" => {
                self.reserve_memory(decimal_bytes(i64::from(self.outcome.status)))?;
                Cow::Owned(self.outcome.status.to_string())
            }
            "IFS" => Cow::Borrowed(" \t\n"),
            "#" => {
                self.reserve_memory(decimal_bytes(self.args.len() as i64))?;
                Cow::Owned(self.args.len().to_string())
            }
            "@" | "*" | "0" => {
                return Err("Only sole quoted \"$@\", $1…$9, $# and $? are supported".into());
            }
            key if key.len() == 1 && key.as_bytes()[0].is_ascii_digit() => Cow::Borrowed(
                self.args
                    .get(key.parse::<usize>().unwrap() - 1)
                    .map_or("", String::as_str),
            ),
            key if name(key) => Cow::Borrowed(self.env.get(key).map_or("", Value::scalar)),
            _ => return Err("Unsupported parameter expansion".into()),
        };
        bounded_expansion_bytes(0, value.len())?;
        self.charge_expansion(value.len())?;
        Ok(value)
    }
    fn variable(&self, key: &str) -> Result<String, String> {
        let value = self.variable_view(key)?;
        self.reserve_memory(value.len())?;
        Ok(value.into_owned())
    }
    fn expand_text(&self, s: &str) -> Result<Expanded<'_>, String> {
        self.charge_expansion(s.len())?;
        let _characters = self.scratch(s.chars().count() * std::mem::size_of::<char>())?;
        let cs: Vec<char> = s.chars().collect();
        let mut i = 0;
        let mut result = String::new();
        let mut memory = self.scratch(0)?;
        while i < cs.len() {
            if cs[i] != '$' {
                let mut bytes = [0; 4];
                self.append_expansion(&mut result, &mut memory, cs[i].encode_utf8(&mut bytes))?;
                i += 1;
                continue;
            }
            if cs[i..].starts_with(&['$', '(', '(']) {
                let end = arithmetic::end(&cs, i)?;
                let _expression_memory =
                    self.scratch(cs[i + 3..end - 2].iter().map(|c| c.len_utf8()).sum())?;
                let expression = cs[i + 3..end - 2].iter().collect::<String>();
                let expanded = self.expand_text(&expression)?;
                let number = arithmetic::evaluate(&expanded.text, |name| self.variable(name))?;
                let _number_memory = self.scratch(decimal_bytes(number))?;
                self.append_expansion(&mut result, &mut memory, &number.to_string())?;
                i = end;
                continue;
            }
            i += 1;
            if i == cs.len() {
                self.append_expansion(&mut result, &mut memory, "$")?;
                break;
            }
            let start;
            let end;
            if cs[i] == '{' {
                i += 1;
                start = i;
                while i < cs.len() && cs[i] != '}' {
                    i += 1;
                }
                if i == cs.len() {
                    return Err("Close ${parameter}".into());
                }
                end = i;
                i += 1;
            } else if cs[i].is_ascii_alphabetic() || cs[i] == '_' {
                start = i;
                i += 1;
                while i < cs.len() && (cs[i].is_ascii_alphanumeric() || cs[i] == '_') {
                    i += 1;
                }
                end = i;
            } else {
                start = i;
                i += 1;
                end = i;
            }
            let _key_memory = self.scratch(cs[start..end].iter().map(|c| c.len_utf8()).sum())?;
            let key = cs[start..end].iter().collect::<String>();
            if let Some(expression) = parameters::parse(&key)? {
                let current = match expression.target {
                    parameters::Target::Named(name) => match self.env.get(name) {
                        Some(Value::Scalar(value)) => Some(value.as_str()),
                        Some(Value::Indexed(_)) => return Err("Unsupported default/alternate operator on an indexed array; use an ordinary scalar or positional argument".into()),
                        None => None,
                    },
                    parameters::Target::Positional(index) => self.args.get(index).map(String::as_str),
                };
                let value = expression.select(current);
                bounded_expansion_bytes(0, value.len())?;
                self.charge_expansion(value.len())?;
                self.append_expansion(&mut result, &mut memory, value)?;
                continue;
            }
            if let Some(expression) = literal_trim::parse(&key)? {
                let current = match expression.target {
                    parameters::Target::Named(name) => match self.env.get(name) {
                        Some(Value::Scalar(value)) => Some(value.as_str()),
                        Some(Value::Indexed(_)) => return Err("Unsupported literal removal on an indexed array; use an ordinary scalar or positional argument".into()),
                        None => None,
                    },
                    parameters::Target::Positional(index) => self.args.get(index).map(String::as_str),
                };
                let value = expression.select(current);
                bounded_expansion_bytes(0, value.len())?;
                self.charge_expansion(value.len())?;
                self.append_expansion(&mut result, &mut memory, value)?;
                continue;
            }
            let value = self.variable_view(&key)?;
            let _value_memory = self.scratch(if matches!(value, Cow::Owned(_)) {
                value.len()
            } else {
                0
            })?;
            self.append_expansion(&mut result, &mut memory, &value)?;
        }
        Ok(Expanded {
            text: result,
            memory,
        })
    }
    fn sole_fields(&self, part: &Part) -> Result<Option<Vec<String>>, String> {
        if part.quote == Quote::Double
            && let Some(key) = part
                .text()
                .and_then(|s| s.strip_prefix("${"))
                .and_then(|s| s.strip_suffix('}'))
            && !key.contains(['{', '}'])
            && let Some(ArrayRef::All(base)) = values::array_ref(key)?
        {
            let elements = self.env.get(base).map_or(&[][..], Value::elements);
            let bytes = expansion_fields_size(elements)?;
            bounded_expansion_bytes(bytes, std::mem::size_of_val(elements))?;
            self.charge_expansion(bytes + std::mem::size_of_val(elements))?;
            self.reserve_memory(strings_bytes(elements))?;
            return Ok(Some(elements.to_vec()));
        }
        if part.quote == Quote::Double && part.text() == Some("$@") {
            let bytes = expansion_fields_size(&self.args)?;
            self.charge_expansion(bytes)?;
            self.reserve_memory(strings_bytes(&self.args))?;
            return Ok(Some(self.args.clone()));
        }
        Ok(None)
    }
    fn split_word(&self, w: &Word, split: bool) -> Result<Vec<String>, String> {
        let mut memory =
            self.scratch(std::mem::size_of::<String>() + std::mem::size_of::<Vec<bool>>())?;
        let mut fields = vec![String::new()];
        let mut unquoted = vec![Vec::new()];
        let mut pattern_possible = false;
        let mut retained = 0;
        let mut keep = false;
        let mut pending = false;
        let next_field =
            |fields: &mut Vec<String>, memory: &mut Scratch<'_>| -> Result<(), String> {
                if fields.len() >= 128 {
                    return Err("Argument limit exceeded (128 expanded fields)".into());
                }
                memory.grow(
                    self,
                    std::mem::size_of::<String>() + std::mem::size_of::<Vec<bool>>(),
                )?;
                fields.push(String::new());
                Ok(())
            };
        for p in &w.0 {
            // Prepared text is data: it is never parameter-expanded or re-lexed here.
            let text = p.text().ok_or("Unprepared substitution node")?;
            if split && p.quote == Quote::Bare {
                pattern_possible |= text.contains(['*', '?', '[']);
                for c in text.chars() {
                    if matches!(c, ' ' | '\t' | '\n') {
                        pending = true;
                    } else {
                        if pending && (!fields.last().unwrap().is_empty() || keep) {
                            next_field(&mut fields, &mut memory)?;
                            unquoted.push(Vec::new());
                            keep = false;
                        }
                        retained = bounded_expansion_bytes(retained, c.len_utf8())?;
                        memory.grow(self, c.len_utf8() * 2)?;
                        fields.last_mut().unwrap().push(c);
                        let flags = unquoted.last_mut().unwrap();
                        flags.resize(flags.len() + c.len_utf8(), true);
                        pending = false;
                    }
                }
            } else {
                if pending && (!fields.last().unwrap().is_empty() || keep) {
                    next_field(&mut fields, &mut memory)?;
                    unquoted.push(Vec::new());
                }
                retained = bounded_expansion_bytes(retained, text.len())?;
                memory.grow(self, text.len() * 2)?;
                fields.last_mut().unwrap().push_str(text);
                let flags = unquoted.last_mut().unwrap();
                flags.resize(flags.len() + text.len(), false);
                keep = true;
                pending = false;
            }
        }
        if !keep && fields.last().is_some_and(String::is_empty) {
            fields.pop();
            unquoted.pop();
        }
        if !split || !pattern_possible {
            return Ok(fields);
        }
        let mut expanded = Vec::new();
        let mut expanded_memory = self.scratch(0)?;
        let mut bytes = 0;
        let mut push = |value: Cow<'_, str>| -> Result<(), String> {
            if expanded.len() >= 128 {
                return Err("Argument limit exceeded (128 expanded fields)".into());
            }
            bytes = bounded_expansion_bytes(bytes, value.len())?;
            self.charge_expansion(value.len())?;
            expanded_memory.grow(self, value.len() + std::mem::size_of::<String>())?;
            expanded.push(value.into_owned());
            Ok(())
        };
        for (field, flags) in fields.into_iter().zip(unquoted) {
            self.charge_expansion(field.len())?;
            let Some(pattern) = globbing::Pattern::parse(&field, &flags)? else {
                push(Cow::Owned(field))?;
                continue;
            };
            let locale = self.scalar("LC_ALL").filter(|s| !s.is_empty());
            let non_c = if let Some(value) = locale {
                !matches!(value, "C" | "POSIX")
            } else {
                ["LC_COLLATE", "LC_CTYPE", "LANG"].iter().any(|key| {
                    self.scalar(key)
                        .is_some_and(|v| !v.is_empty() && !matches!(v, "C" | "POSIX"))
                })
            };
            if non_c {
                return Err("Unsupported filename-matching locale; this simulator supports C/POSIX byte ordering only".into());
            }
            let mut matched = false;
            for file in self.outcome.files.keys() {
                let name = &file[6..];
                if pattern.matches(name, |n| self.charge_expansion(n))? {
                    let value = if pattern.absolute {
                        file.as_str()
                    } else {
                        name
                    };
                    // The shared push checks count, bytes and work before cloning.
                    push(Cow::Borrowed(value))?;
                    matched = true;
                }
            }
            if !matched {
                push(Cow::Owned(field))?;
            }
        }
        Ok(expanded)
    }
    pub fn tick(&mut self, budget: usize) {
        let mut instructions = 0;
        let mut preparations = 0;
        while instructions < budget.min(64) {
            if self.done {
                return;
            }
            if let Some(pending) = self.pending.take() {
                // Preparation yields independently of semantic instructions;
                // it cannot hide an entire large word list in one tick.
                if preparations == 32 {
                    self.pending = Some(pending);
                    return;
                }
                preparations += 1;
                match self.advance_pending(pending) {
                    Ok(next) => self.pending = next,
                    Err(error) => {
                        self.fail(error);
                        return;
                    }
                }
                continue;
            }
            if self.scope_exit.is_some() || (self.calls.is_empty() && self.pc == self.range.end) {
                if self.frames.is_empty() {
                    self.done = true;
                    return;
                }
                if preparations == 32 {
                    return;
                }
                preparations += 1;
                if let Err(error) = self.finish_capture() {
                    self.fail(error);
                    return;
                }
                continue;
            }
            if self.pc >= self.program.code.len() {
                self.fail("Invalid execution boundary".into());
                return;
            }
            if self.steps >= MAX_STEPS {
                self.fail("Execution limit reached (2,000 instructions). Check the loop condition; Stop is also available.".into());
                return;
            }
            self.steps += 1;
            self.frame_executed = true;
            instructions += 1;
            let code = &self.program.code[self.pc];
            let bytes = match code {
                Code::Pipe(commands) => commands.iter().map(command_bytes).sum(),
                Code::ArrayAssign(key, words) | Code::For(key, words, _) => {
                    key.len() + words.iter().map(word_bytes).sum::<usize>()
                }
                Code::WhileStart(Some(word), _) => word_bytes(word),
                Code::Define(key, _, _) => key.len(),
                _ => 0,
            };
            if let Err(error) = self.reserve_memory(bytes) {
                self.fail(error);
                return;
            }
            let code = code.clone();
            self.pc += 1;
            let r = (|| -> Result<(), String> {
                match code {
                    Code::ArrayAssign(key, words) => {
                        self.pending = Some(execution::Pending::Array(
                            key,
                            preparation::Words::new(words.into_iter().map(|w| (w, true)).collect()),
                        ));
                    }
                    Code::Case(definition) => {
                        self.pending =
                            Some(execution::Pending::Case(case::Task::new(self, definition)?));
                    }
                    Code::Status(s) => self.outcome.status = s,
                    Code::WhileStart(word, end) => {
                        if let Some(word) = word {
                            self.pending = Some(execution::Pending::While(
                                end,
                                preparation::Words::new(vec![(word, true)]),
                            ));
                        } else {
                            self.while_status.push((0, false));
                        }
                    }
                    Code::WhileBody => {
                        self.while_status.last_mut().ok_or("Invalid while state")?.0 =
                            self.outcome.status
                    }
                    Code::WhileEnd => {
                        let (status, redirected) =
                            self.while_status.pop().ok_or("Invalid while state")?;
                        self.outcome.status = status;
                        if redirected {
                            self.restore_input()?;
                        }
                    }
                    Code::Define(n, start, end) => {
                        let names = self.live_function_names();
                        if names.len() >= 64 && !names.contains(n.as_str()) {
                            return Err("Function definition limit exceeded (64)".into());
                        }
                        self.functions.insert(n, start);
                        self.pc = end;
                        self.outcome.status = 0;
                    }
                    Code::FunctionEnd => self.leave_function()?,
                    Code::Pipe(cmds) => {
                        self.pending = Some(execution::Pending::Pipeline(Box::new(
                            execution::Pipeline::new(cmds),
                        )));
                    }
                    Code::Jump(t) => self.pc = t,
                    Code::JumpIf(success, t) => {
                        if (self.outcome.status == 0) == success {
                            self.pc = t
                        }
                    }
                    Code::For(n, words, end) => {
                        self.pending = Some(execution::Pending::For(
                            n,
                            end,
                            preparation::Words::new(words.into_iter().map(|w| (w, true)).collect()),
                        ));
                    }
                    Code::Next(start) => {
                        let copy_bytes = self
                            .loops
                            .last()
                            .and_then(|(_, values, index)| values.get(*index))
                            .map_or(0, String::len);
                        self.charge_expansion(copy_bytes)?;
                        let (n, values, index) =
                            self.loops.last_mut().ok_or("Invalid loop state")?;
                        if *index < values.len() {
                            let (key, value) = (n.clone(), values[*index].clone());
                            *index += 1;
                            self.assign_scalar(key, value)?;
                            self.pc = start;
                        } else {
                            self.loops.pop();
                        }
                    }
                }
                if let Some(pending) = &self.pending {
                    self.pending_bytes.set(pending.bytes());
                    self.pending_output_bytes.set(pending.output_bytes());
                }
                self.limits()
            })();
            if let Err(e) = r {
                self.fail(e);
                return;
            }
        }
    }
    fn leave_function(&mut self) -> Result<(), String> {
        let frame = self
            .calls
            .pop()
            .ok_or("Unsupported return outside a function")?;
        for (name, previous) in frame.locals {
            if let Some(value) = previous {
                self.env.insert(name, value);
            } else {
                self.env.remove(&name);
            }
        }
        if let Some(scanner) = frame.scanner {
            self.scanner = scanner;
        }
        self.pc = frame.pc;
        self.args = frame.args;
        self.loops.truncate(frame.loops);
        self.while_status.truncate(frame.whiles);
        while self.inputs.len() > frame.inputs {
            self.restore_input()?;
        }
        Ok(())
    }
    fn enter_function(
        &mut self,
        cmd: &Command,
        mut args: Vec<String>,
        start: usize,
    ) -> Result<(), String> {
        if !cmd.redirects.is_empty() {
            return Err("Unsupported redirection on a function call; redirect individual commands inside its body".into());
        }
        if self.calls.len()
            + self
                .frames
                .iter()
                .map(|frame| frame.calls.len())
                .sum::<usize>()
            >= 32
        {
            return Err("Function call depth limit exceeded (32)".into());
        }
        self.reserve_memory(strings_bytes(&args[1..]))?;
        args.remove(0);
        let old_args = std::mem::replace(&mut self.args, args);
        self.calls.push(CallFrame {
            pc: self.pc,
            args: old_args,
            loops: self.loops.len(),
            whiles: self.while_status.len(),
            inputs: self.inputs.len(),
            locals: BTreeMap::new(),
            scanner: None,
        });
        self.pc = start;
        Ok(())
    }
    fn restore_input(&mut self) -> Result<(), String> {
        let saved = self.inputs.pop().ok_or("Invalid loop input state")?;
        self.input = saved.input;
        self.input_pos = saved.pos;
        self.input_id = saved.id;
        Ok(())
    }
    fn writable_file(&self, file: &str) -> Result<(), String> {
        if self.inputs.iter().any(|f| f.file == file)
            || self
                .frames
                .iter()
                .any(|frame| frame.inputs.iter().any(|input| input.file == file))
        {
            Err("Unsupported write to an active loop input; finish reading the source before changing it".into())
        } else if self.ordinary_inputs.iter().any(|source| source == file) {
            Err("Unsupported write to an active input source; finish reading it before changing the file".into())
        } else {
            Ok(())
        }
    }
    fn scalar(&self, key: &str) -> Option<&str> {
        self.env.get(key).map(Value::scalar)
    }
    fn reserve_memory(&self, additional: usize) -> Result<(), String> {
        if self
            .memory_bytes()
            .checked_add(additional)
            .is_none_or(|n| n > MAX_TEXT)
        {
            Err("Virtual memory/file limit exceeded".into())
        } else {
            Ok(())
        }
    }
    fn memory_bytes(&self) -> usize {
        capture::context_bytes(
            &self.env,
            &self.args,
            &self.loops,
            &self.inputs,
            &self.functions,
            &self.calls,
        ) + self.scanner.bytes()
            + self.input.len()
            + self.outcome.stdout.len()
            + self.outcome.stderr.len()
            + self
                .outcome
                .files
                .iter()
                .map(|(key, value)| key.len() + value.len())
                .sum::<usize>()
            + strings_bytes(&self.ordinary_inputs)
            + self.pending_bytes.get()
            + self.scratch_bytes.get()
            + self
                .frames
                .iter()
                .map(capture::Suspended::bytes)
                .sum::<usize>()
            + self.buffer_bytes()
            + capture::dest_bytes(&self.base_stdout)
            + capture::dest_bytes(&self.base_stderr)
            + self.program_bytes
    }
    fn fresh_input_id(&mut self) -> Result<usize, String> {
        let id = self.next_input_id;
        self.next_input_id = id
            .checked_add(1)
            .ok_or("Input binding identity limit exceeded")?;
        Ok(id)
    }
    fn preflight_value(&self, key: &str, bytes: usize) -> Result<(), String> {
        readable_variable(key)?;
        if !name(key) {
            return Err("Unsupported variable name".into());
        }
        let old = self.env.get(key).map_or(0, Value::bytes);
        let additional_name = if self.env.contains_key(key) {
            0
        } else {
            key.len()
        };
        let next = self
            .memory_bytes()
            .checked_sub(old)
            .and_then(|n| n.checked_add(bytes))
            .and_then(|n| n.checked_add(additional_name));
        if next.is_none_or(|n| n > MAX_TEXT) {
            return Err("Virtual memory/file limit exceeded".into());
        }
        let visible = self.live_variable_names();
        if !visible.contains(key)
            && (visible.len() >= MAX_ORDINARY_NAMES + MAX_SYSTEM_NAMES
                || ordinary_variable_name(key)
                    && visible
                        .iter()
                        .filter(|name| ordinary_variable_name(name))
                        .count()
                        >= MAX_ORDINARY_NAMES)
        {
            return Err(
                "Variable limit exceeded (256 ordinary names plus three fixed getopts names)"
                    .into(),
            );
        }
        Ok(())
    }
    fn live_variable_names(&self) -> BTreeSet<&str> {
        self.env
            .keys()
            .map(String::as_str)
            .chain(
                self.calls
                    .iter()
                    .flat_map(|call| call.locals.keys().map(String::as_str)),
            )
            .chain(
                self.frames
                    .iter()
                    .flat_map(|frame| frame.env.keys().map(String::as_str)),
            )
            .chain(self.frames.iter().flat_map(|frame| {
                frame
                    .calls
                    .iter()
                    .flat_map(|call| call.locals.keys().map(String::as_str))
            }))
            .collect()
    }
    fn live_function_names(&self) -> BTreeSet<&str> {
        self.functions
            .keys()
            .map(String::as_str)
            .chain(
                self.frames
                    .iter()
                    .flat_map(|frame| frame.functions.keys().map(String::as_str)),
            )
            .collect()
    }
    fn store_value(&mut self, key: String, value: Value) -> Result<(), String> {
        variable_target(&key)?;
        if value.elements().len() > 128 {
            return Err("Array element limit exceeded (128)".into());
        }
        self.preflight_value(&key, value.bytes())?;
        self.env.insert(key, value);
        Ok(())
    }
    fn assign_scalar(&mut self, key: String, value: String) -> Result<(), String> {
        scalar_assignment_target(&key)?;
        if key == "OPTIND" {
            if value != "1" {
                return Err("Unsupported OPTIND write; only scalar value1 resets getopts".into());
            }
            self.preflight_value(&key, value.len())?;
            self.env.insert(key, Value::Scalar(value));
            self.scanner = getopts::Scanner::default();
            return Ok(());
        }
        let bytes = match self.env.get(&key) {
            Some(Value::Indexed(elements)) => {
                elements.iter().skip(1).map(String::len).sum::<usize>()
                    + elements.len().max(1) * std::mem::size_of::<String>()
                    + value.len()
            }
            _ => value.len(),
        };
        self.preflight_value(&key, bytes)?;
        if let Some(previous) = self.env.get_mut(&key) {
            previous.assign_scalar(value);
        } else {
            self.env.insert(key, Value::Scalar(value));
        }
        Ok(())
    }
    fn limits(&self) -> Result<(), String> {
        if !variable_names_within_limit(&self.live_variable_names())
            || self.live_function_names().len() > 64
            || self.outcome.files.len() > 64
            || self.memory_bytes() > MAX_TEXT
        {
            return Err("Virtual memory/file limit exceeded".into());
        }
        if self.output_bytes() > MAX_OUTPUT {
            return Err("Output limit reached (32 KiB)".into());
        }
        Ok(())
    }
    fn emit(&mut self, dest: &Dest, text: &str) -> Result<(), String> {
        if let Dest::File(file) = dest {
            self.writable_file(file)?;
        }
        if text.len() > MAX_OUTPUT {
            return Err("Command output exceeds 32 KiB".into());
        }
        match dest {
            Dest::Out | Dest::Err | Dest::Buffer(_)
                if self.output_bytes() + text.len() > MAX_OUTPUT =>
            {
                return Err("Output limit reached (32 KiB)".into());
            }
            Dest::File(_)
                if self
                    .memory_bytes()
                    .checked_add(
                        text.len()
                            + if let Dest::File(file) = dest {
                                file.len()
                            } else {
                                0
                            },
                    )
                    .is_none_or(|n| n > MAX_TEXT) =>
            {
                return Err("Virtual memory/file limit exceeded".into());
            }
            Dest::Out | Dest::Err | Dest::Buffer(_)
                if self
                    .memory_bytes()
                    .checked_add(text.len())
                    .is_none_or(|n| n > MAX_TEXT) =>
            {
                return Err("Virtual memory/file limit exceeded".into());
            }
            _ => {}
        }
        match dest {
            Dest::Out => self.outcome.stdout.push_str(text),
            Dest::Err => self.outcome.stderr.push_str(text),
            Dest::File(p) => self
                .outcome
                .files
                .entry(p.clone())
                .or_default()
                .push_str(text),
            Dest::Buffer(id) => self
                .buffers
                .get_mut(id.0)
                .and_then(Option::as_mut)
                .ok_or("Invalid output binding")?
                .push_str(text),
        }
        self.limits()
    }
    fn apply_local_declarations(
        &mut self,
        declarations: Vec<LocalDeclaration>,
    ) -> Result<(), String> {
        for (key, new_value) in declarations {
            self.release_pending(std::mem::size_of::<LocalDeclaration>())?;
            if !self.calls.last().unwrap().locals.contains_key(&key) {
                if key == "OPTIND" {
                    self.reserve_memory(std::mem::size_of::<getopts::Scanner>())?;
                    // First declaration moves the prior scanner. Repeated local
                    // resets keep this original snapshot until scope exit.
                    let scanner = std::mem::take(&mut self.scanner);
                    self.calls.last_mut().unwrap().scanner = Some(scanner);
                }
                self.reserve_memory(key.len())?;
                let previous = self.env.remove(&key);
                let array = matches!(&previous, Some(Value::Indexed(_)));
                self.calls
                    .last_mut()
                    .unwrap()
                    .locals
                    .insert(key.clone(), previous);
                if array {
                    self.reserve_memory(key.len())?;
                    self.env.insert(key.clone(), Value::Indexed(Vec::new()));
                }
            }
            if let Some(value) = new_value {
                self.release_pending(key.len() + value.len())?;
                self.assign_scalar(key, value)?;
            } else {
                self.release_pending(key.len())?;
            }
            self.limits()?;
        }
        Ok(())
    }
    fn builtin(
        &mut self,
        a: &[String],
        read_mode: execution::ReadMode,
    ) -> Result<(String, String, i32), String> {
        let empty = String::new();
        let Some(cmd) = a.first().map(String::as_str) else {
            return Ok((empty.clone(), empty, 0));
        };
        let args = &a[1..];
        let ok = |s: String| (s, String::new(), 0);
        let usage = || Err(format!("Unsupported {cmd} arguments; see the shell subset"));
        match cmd {
            "getopts" => self.getopts_builtin(args),
            "set" if args.len() == 2 && args[1] == "pipefail" && matches!(args[0].as_str(), "-o" | "+o") => {
                self.pipefail = args[0] == "-o";
                Ok((empty.clone(), empty, 0))
            }
            "local" => Err("Unsupported declaration command name: use literal unquoted local inside a function".into()),
            // Arguments have already passed ordinary expansion and redirects.
            // Colon ignores their resulting values, not their evaluation effects.
            ":" => Ok((empty.clone(), empty, 0)),
            "true" | "false" if args.is_empty() => {
                Ok((empty.clone(), empty, i32::from(cmd == "false")))
            }
            "printf" => {
                if args.is_empty() || args[0].starts_with('-') {
                    return usage();
                }
                Ok(ok(format_printf(&args[0], &args[1..], |bytes| self.reserve_memory(bytes))?))
            }
            "echo" => {
                if args.first().is_some_and(|s| s.starts_with('-'))
                    || args.iter().any(|s| s.contains('\\'))
                {
                    return usage();
                }
                self.reserve_memory(args.iter().map(String::len).sum::<usize>() + args.len())?;
                let mut output = args.join(" ");
                output.push('\n');
                Ok(ok(output))
            }
            "cat" => {
                // Only the first -- ends options; a second -- is a filename.
                // There is one optional operand, never host utility lookup.
                let operands = if args.first().is_some_and(|s| s == "--") {
                    &args[1..]
                } else {
                    args
                };
                let file = match operands {
                    [] => None,
                    [file] if file == "-" => None,
                    [file] if args.first().is_some_and(|s| s == "--") || !file.starts_with('-') => Some(file),
                    _ => return usage(),
                };
                if let Some(file) = file {
                    let p = self.virtual_path(file)?;
                    Ok(match self.outcome.files.get(&p.text) {
                        Some(s) => { self.reserve_memory(s.len())?; ok(s.clone()) },
                        None => {
                            self.reserve_memory(file.len() + "cat: : No such file or directory\n".len())?;
                            (empty, format!("cat: {file}: No such file or directory\n"), 1)
                        },
                    })
                } else {
                    let text = &self.input[self.input_pos..];
                    self.reserve_memory(text.len())?;
                    let output = text.to_owned();
                    self.input_pos = self.input.len();
                    Ok(ok(output))
                }
            }
            "grep" if (args.len() == 2 || args.len() == 3) && args[0] == "-F" => {
                if args[1].starts_with('-') {
                    return Err("Unsupported grep pattern starting with -; option parsing with -e/-- is not yet implemented".into());
                }
                let stdin = args.len() != 3;
                let text = if let Some(file) = args.get(2) {
                    if file.starts_with('-') { return usage(); }
                    let p = self.virtual_path(file)?;
                    match self.outcome.files.get(&p.text) {
                        Some(text) => text.as_str(),
                        None => {
                            self.reserve_memory(file.len() + "grep: : No such file or directory\n".len())?;
                            return Ok((empty, format!("grep: {file}: No such file or directory\n"), 2));
                        },
                    }
                } else { &self.input[self.input_pos..] };
                if args[1].contains('\n') { return Err("Multi-line grep patterns are unsupported".into()); }
                let mut output = String::new();
                for line in text.split_inclusive('\n').filter(|line| line.contains(&args[1])) {
                    let extra = usize::from(!line.ends_with('\n'));
                    self.reserve_memory(output.len() + line.len() + extra)?;
                    if output.len() + line.len() + extra > MAX_OUTPUT { return Err("Command output exceeds 32 KiB".into()); }
                    output.push_str(line);
                    if extra != 0 { output.push('\n'); }
                }
                if stdin { self.input_pos = self.input.len(); }
                let status = i32::from(output.is_empty());
                Ok((output, empty, status))
            }
            "sort" | "uniq" | "cut" => {
                self.reserve_memory(args.iter().map(String::len).sum())?;
                let invocation = text_tools::parse(cmd, args)?;
                let invocation_memory = self.scratch(invocation.file.as_ref().map_or(0, String::len) + invocation.temporary_text_bytes())?;
                // These utilities model only the C locale. LC_ALL overrides the
                // individual category variables and LANG, as in real tools.
                let locale = self.scalar("LC_ALL").filter(|s| !s.is_empty());
                let unsupported_locale = if let Some(value) = locale {
                    value != "C" && value != "POSIX"
                } else {
                    ["LC_COLLATE", "LC_CTYPE", "LANG"].iter().any(|name| {
                        self.scalar(name).is_some_and(|v| !v.is_empty() && v != "C" && v != "POSIX")
                    })
                };
                if unsupported_locale {
                    return Err("Unsupported text-utility locale; this simulator supports C/POSIX byte ordering only".into());
                }
                let file = invocation.file.as_deref().filter(|s| *s != "-");
                let text = if let Some(file) = file {
                    let p = self.virtual_path(file)?;
                    let Some(text) = self.outcome.files.get(&p.text) else {
                        self.reserve_memory(file.len() + 60)?;
                        let message = if cmd == "sort" {
                            format!("sort: cannot read: {file}: No such file or directory\n")
                        } else {
                            format!("{cmd}: {file}: No such file or directory\n")
                        };
                        return Ok((empty, message, if cmd == "sort" { 2 } else { 1 }));
                    };
                    text
                } else {
                    &self.input[self.input_pos..]
                };
                let available = MAX_TEXT.saturating_sub(self.memory_bytes()).min(MAX_OUTPUT);
                let output = invocation.execute(text, available)?;
                drop(invocation_memory);
                if file.is_none() { self.input_pos = self.input.len(); }
                Ok(ok(output))
            }
            "read" if args.len() == 2 && args[0] == "-r" && name(&args[1]) => {
                variable_target(&args[1])?;
                let rest = &self.input[self.input_pos..];
                let (line, status, used) = if let Some(i) = rest.find('\n') {
                    (&rest[..i], 0, i + 1)
                } else {
                    (rest, 1, rest.len())
                };
                let line = match read_mode {
                    execution::ReadMode::Default => line.trim_matches([' ', '\t']),
                    execution::ReadMode::EmptyIfs => line,
                };
                self.reserve_memory(args[1].len() + line.len())?;
                self.assign_scalar(args[1].clone(), line.into())?;
                self.input_pos += used;
                Ok((empty.clone(), empty, status))
            }
            "test" | "[" => {
                let args = if cmd == "[" {
                    if args.last().map(String::as_str) != Some("]") {
                        return Err("Close the test with a separate ]".into());
                    }
                    &args[..args.len() - 1]
                } else {
                    args
                };
                let value = match args {
                    [] => false,
                    [s] => !s.is_empty(),
                    [op, s] if op == "-n" => !s.is_empty(),
                    [op, s] if op == "-z" => s.is_empty(),
                    [op, s] if op == "-f" => self.outcome.files.contains_key(&self.virtual_path(s)?.text),
                    [l, op, r] => match op.as_str() {
                        "=" => l == r,
                        "!=" => l != r,
                        "-eq" | "-ne" | "-lt" | "-le" | "-gt" | "-ge" => {
                            let (Ok(l), Ok(r)) = (l.trim().parse::<i64>(), r.trim().parse::<i64>())
                            else {
                                return Err("Unsupported integer-test operand; this subset accepts signed 64-bit decimal integers only".into());
                            };
                            match op.as_str() {
                                "-eq" => l == r,
                                "-ne" => l != r,
                                "-lt" => l < r,
                                "-le" => l <= r,
                                "-gt" => l > r,
                                _ => l >= r,
                            }
                        }
                        _ => return usage(),
                    },
                    _ => return usage(),
                };
                Ok((empty.clone(), empty, i32::from(!value)))
            }
            "shift" if args.len() <= 1 => {
                let count = if args.is_empty() {
                    1
                } else {
                    args[0]
                        .parse::<usize>()
                        .map_err(|_| "Unsupported shift count; use a nonnegative decimal integer")?
                };
                let status = if count > self.args.len() {
                    1
                } else {
                    self.args.drain(..count);
                    0
                };
                Ok((empty.clone(), empty, status))
            }
            "return" if args.len() <= 1 => {
                let status = if args.is_empty() {
                    self.outcome.status
                } else {
                    args[0]
                        .parse::<u8>()
                        .map_err(|_| "Unsupported return status; use 0…255")?
                        as i32
                };
                self.leave_function()?;
                Ok((empty.clone(), empty, status))
            }
            "exit" if args.len() <= 1 => {
                let status = if args.is_empty() {
                    self.outcome.status
                } else {
                    args[0]
                        .parse::<u8>()
                        .map_err(|_| "exit supports status 0…255")? as i32
                };
                self.scope_exit = Some(status);
                Ok((empty.clone(), empty, status))
            }
            _ => Err(format!(
                "Unsupported command: {cmd:.80}. Only documented in-process commands are available; nothing was sent to the host."
            )),
        }
    }
}
fn format_printf(
    format: &str,
    args: &[String],
    reserve: impl Fn(usize) -> Result<(), String>,
) -> Result<String, String> {
    let mut out = String::new();
    let mut index = 0;
    loop {
        let mut chars = format.chars();
        let before = index;
        while let Some(c) = chars.next() {
            match c {
                '%' => match chars.next() {
                    Some('s') => {
                        let text = args.get(index).map(String::as_str).unwrap_or("");
                        if out.len() + text.len() > MAX_OUTPUT {
                            return Err("printf output exceeds 32 KiB".into());
                        }
                        reserve(out.len() + text.len())?;
                        out.push_str(text);
                        index += 1;
                    }
                    Some('%') => {
                        reserve(out.len() + 1)?;
                        out.push('%');
                    }
                    _ => {
                        return Err(
                            "printf supports %s and %% only; put data in a %s argument".into()
                        );
                    }
                },
                '\\' => match chars.next() {
                    Some('n') => {
                        reserve(out.len() + 1)?;
                        out.push('\n');
                    }
                    Some('t') => {
                        reserve(out.len() + 1)?;
                        out.push('\t');
                    }
                    Some('\\') => {
                        reserve(out.len() + 1)?;
                        out.push('\\');
                    }
                    _ => return Err("printf supports \\n, \\t and \\\\ escapes only".into()),
                },
                _ => {
                    reserve(out.len() + c.len_utf8())?;
                    out.push(c);
                }
            }
            if out.len() > MAX_OUTPUT {
                return Err("printf output exceeds 32 KiB".into());
            }
        }
        if index >= args.len() || index == before {
            break;
        }
    }
    Ok(out)
}
pub fn run(source: &str, case: &Case) -> Outcome {
    match Vm::new(source, case) {
        Ok(mut vm) => {
            while !vm.done() {
                vm.tick(64)
            }
            vm.outcome
        }
        Err(e) => Outcome {
            stdout: String::new(),
            stderr: String::new(),
            status: 2,
            files: case.files.clone(),
            error: Some(diagnostic(e)),
        },
    }
}
pub fn checks(source: &str, record: &Option<Record>, cases: &[Case]) -> Vec<(bool, String)> {
    cases.iter().enumerate().map(|(i,c)|{let r=record.as_ref().filter(|r|r.current() && r.source==source && r.results.len()==cases.len()).and_then(|r|r.results.get(i));let pass=r.is_some_and(|r|c.accepts(r));let message=if pass{format!("Pass · {}",c.name)}else{format!("{} · {}",c.name,match r{None=>"Run the current script against this case".into(),Some(r)=>if let Some(e)=&r.error{e.clone()}else if r.status!=c.status{format!("Exit status {}; expected {}. Inspect the last command or an early exit.",r.status,c.status)}else if r.stdout!=c.stdout{"Standard output differs. Check spacing, quoting and line breaks.".into()}else if r.stderr!=c.stderr{"Error output differs. Send the requested diagnostic to standard error.".into()}else{"Virtual files differ. Check the destination, append/truncate behavior and untouched input files.".into()}})};(pass,message)}).collect()
}

/// Literal argv for the separate virtual terminal; no expansion or evaluation.
pub fn literal_command(source: &str) -> Result<Vec<String>, String> {
    if source.len() > 4096 {
        return Err("Command exceeds 4,096 bytes".into());
    }
    lex(source)?.into_iter().map(|token| match token {
        Token::Word(w) => {
            if w.0.iter().any(|p| p.quote == Quote::Bare && p.text().is_some_and(|s| s.contains(['*', '?']))) { return Err("Filename expansion is unsupported in this terminal; use a literal quoted path. Script labs have a separate supported subset.".into()); }
            if w.0.iter().any(|p| p.quote != Quote::Single && p.text().is_none_or(|s| s.contains('$'))) { return Err("Variable expansion is unsupported in this terminal; use a literal quoted path.".into()); }
            Ok(w.0.into_iter().map(|p| match p.value { PartValue::Text(text) => text, PartValue::Substitution(_) => unreachable!("terminal substitution gate") }).collect())
        },
        Token::Op(_) => Err("Shell operators are unsupported in this terminal. Enter one supported command at a time.".into()),
    }).collect()
}
