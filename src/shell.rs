//! Bounded, pure shell-subset interpreter. No host I/O, subprocesses or network.
mod arithmetic;
mod globbing;
mod text_tools;
mod values;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
};
use values::{ArrayRef, Value};
// Increment when previously saved outcomes can change under corrected evaluator semantics.
pub const MAX_CASES: usize = 12;
pub const EVALUATOR_REVISION: u32 = 6;
pub(crate) fn legacy_revision(revision: &u32) -> bool {
    *revision == 0
}
pub const MAX_SOURCE: usize = 16_384;
const MAX_TEXT: usize = 262_144;
const MAX_OUTPUT: usize = 32_768;
const MAX_STEPS: usize = 2000;
const MAX_EXPANSION_WORK: usize = 1_000_000;
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
    text: String,
    quote: Quote,
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
            p.quote == Quote::Bare && p.text.split_once('=').is_some_and(|(n, _)| name(n))
        })
    }
    fn literal(&self) -> Option<&str> {
        if self.0.len() == 1 && self.0[0].quote == Quote::Bare {
            Some(&self.0[0].text)
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
            "OPTARG",
            "OPTIND",
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
fn variable_target(n: &str) -> Result<(), String> {
    if n == "IFS" {
        return Err(
            "Unsupported custom IFS; this subset uses default space, tab and newline splitting"
                .into(),
        );
    }
    if unmodeled_variable(n) {
        return Err(format!(
            "Unsupported special-variable name or reserved namespace {n}; use an ordinary variable name within this subset"
        ));
    }
    Ok(())
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
fn lex(source: &str) -> Result<Vec<Token>, String> {
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
                text: std::mem::take(bare),
                quote: Quote::Bare,
            })
        }
    }
    fn word(tokens: &mut Vec<Token>, parts: &mut Vec<Part>, bare: &mut String) {
        flush(parts, bare);
        if !parts.is_empty() {
            tokens.push(Token::Word(Word(std::mem::take(parts))))
        }
    }
    while i < cs.len() {
        let c = cs[i];
        if c == '#' && parts.is_empty() && bare.is_empty() {
            while i < cs.len() && cs[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '\r' {
            return Err(
                "Use LF script line breaks; carriage returns outside quotes are unsupported".into(),
            );
        }
        if c == ' ' || c == '\t' {
            word(&mut tokens, &mut parts, &mut bare);
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
            let mut closed = false;
            while i < cs.len() {
                if quote == Quote::Double && cs[i..].starts_with(&['$', '(', '(']) {
                    let end = arithmetic::end(&cs, i)?;
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
                                text: std::mem::take(&mut text),
                                quote,
                            });
                        }
                        parts.push(Part {
                            text: x.to_string(),
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
            parts.push(Part { text, quote });
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
                    text: cs[i].to_string(),
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
            let end = arithmetic::end(&cs, i)?;
            bare.extend(cs[i..end].iter());
            i = end;
            continue;
        }
        if c == '$' && cs.get(i + 1) == Some(&'{') {
            bare.push(c);
            i += 1;
            while i < cs.len() {
                bare.push(cs[i]);
                let end = cs[i] == '}';
                i += 1;
                if end {
                    break;
                }
            }
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
            word(&mut tokens, &mut parts, &mut bare);
            let array_opener = adjacent_array
                && matches!(tokens.last(), Some(Token::Word(w)) if w.literal().is_some_and(|s| s.ends_with('=')));
            if c == '\n' {
                tokens.push(Token::Op("\n".into()));
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
                && ((cs[i] == c && matches!(c, '|' | '&' | '>' | '<'))
                    || (cs[i] == '>' && op.ends_with('>')))
            {
                op.push(cs[i]);
                i += 1;
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
                "=(", "(", ")", ";", "|", "&&", "||", "<", ">", ">>", "1>", "1>>", "2>", "2>>",
                ">&2", "1>&2", "2>&1",
            ]
            .contains(&op.as_str())
            {
                return Err(format!("Unsupported operator: {op}"));
            }
            tokens.push(Token::Op(op));
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
    word(&mut tokens, &mut parts, &mut bare);
    if tokens.len() > 4096 {
        return Err("Script exceeds 4096 tokens".into());
    }
    for token in &tokens {
        if let Token::Word(w) = token {
            for p in &w.0 {
                if p.quote == Quote::Bare
                    && values::bracket_glob(&p.text)?
                    && p.text != "["
                    && p.text != "]"
                {
                    return Err("Bracket globbing is unsupported; quote literal brackets".into());
                }
            }
        }
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
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    code: Vec<Code>,
    depth: usize,
}
impl Parser {
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
            if self.pos < self.tokens.len() && !self.eat(";") && !self.eat("\n") {
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
                    Some(Token::Word(word)) => {words.push(word.clone());self.pos+=1;}
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
                words.push(w.clone());
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
        if ["then", "else", "fi", "do", "done"]
            .iter()
            .any(|s| self.is(s))
        {
            return Err(
                "Syntax error: unexpected then/else/fi/do/done; check the matching if or loop"
                    .into(),
            );
        }
        if ["case", "until", "select", "function", "!"]
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
                    words.push(w.clone());
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
                        self.pos += 1;
                        Some(w.clone())
                    };
                    redirects.push((op, w));
                }
                _ => break,
            }
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
fn compile(source: &str) -> Result<Vec<Code>, String> {
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
    let mut p = Parser {
        tokens: lex(source).map_err(classified)?,
        pos: 0,
        code: Vec::new(),
        depth: 0,
    };
    p.list(&[]).map_err(classified)?;
    Ok(p.code)
}
fn path(s: &str) -> Result<String, String> {
    let s = if s.starts_with('/') {
        s.to_string()
    } else {
        format!("/work/{s}")
    };
    if !s.starts_with("/work/")
        || s[6..].contains('/')
        || matches!(&s[6..], "." | "..")
        || s.len() > 256
        || s.contains('\0')
        || s.ends_with('/')
    {
        Err("Only ordinary virtual paths under /work are supported".into())
    } else {
        Ok(s)
    }
}
#[derive(Clone)]
enum Dest {
    Out,
    Err,
    File(String),
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
type LocalDeclaration = (String, Option<String>);
struct InputFrame {
    input: String,
    pos: usize,
    file: String,
}
struct CallFrame {
    pc: usize,
    args: Vec<String>,
    loops: usize,
    whiles: usize,
    inputs: usize,
    locals: BTreeMap<String, Option<Value>>,
}
pub struct Vm {
    code: Vec<Code>,
    pc: usize,
    steps: usize,
    expansion_work: Cell<usize>,
    env: BTreeMap<String, Value>,
    args: Vec<String>,
    input: String,
    input_pos: usize,
    loops: Vec<(String, Vec<String>, usize)>,
    while_status: Vec<(i32, bool)>,
    inputs: Vec<InputFrame>,
    functions: BTreeMap<String, usize>,
    calls: Vec<CallFrame>,
    pub outcome: Outcome,
    done: bool,
    pipeline_bytes: usize,
}
impl Vm {
    pub fn new(source: &str, case: &Case) -> Result<Self, String> {
        if !case.valid() {
            return Err("Invalid simulation fixture".into());
        }
        Ok(Self {
            code: compile(source)?,
            pc: 0,
            steps: 0,
            expansion_work: Cell::new(0),
            env: case
                .env
                .iter()
                .map(|(k, v)| (k.clone(), Value::Scalar(v.clone())))
                .collect(),
            args: case.args.clone(),
            input: case.stdin.clone(),
            input_pos: 0,
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
            pipeline_bytes: 0,
        })
    }
    pub fn done(&self) -> bool {
        self.done
    }
    pub fn stop(&mut self) {
        self.fail("Stopped: this run was not graded".into())
    }
    fn fail(&mut self, message: String) {
        self.outcome.error = Some(diagnostic(message));
        self.outcome.status = 2;
        self.done = true;
    }
    /// Charge all expansion paths, including preflight and arithmetic callbacks, before copying.
    fn charge_expansion(&self, bytes: usize) -> Result<(), String> {
        let next = self.expansion_work.get().checked_add(bytes)
            .filter(|n| *n <= MAX_EXPANSION_WORK)
            .ok_or("Expansion work limit reached (1,000,000 processed bytes). Reduce repeated large expansions; Stop is also available.")?;
        self.expansion_work.set(next);
        Ok(())
    }
    fn append_expansion(&self, output: &mut String, text: &str) -> Result<(), String> {
        bounded_expansion_bytes(output.len(), text.len())?;
        self.charge_expansion(text.len())?;
        output.push_str(text);
        Ok(())
    }
    fn variable(&self, key: &str) -> Result<String, String> {
        if let Some(parameter) = values::array_ref(key)? {
            let value: Cow<'_, str> = match parameter {
                ArrayRef::Element(base,index) => Cow::Borrowed(self.env.get(base).and_then(|v|v.elements().get(index as usize)).map_or("",String::as_str)),
                ArrayRef::Count(base) => Cow::Owned(self.env.get(base).map_or(0,|v|v.elements().len()).to_string()),
                ArrayRef::All(_) => return Err("Unsupported mixed or unquoted array expansion; use sole quoted \"${name[@]}\" as one complete word".into()),
            };
            bounded_expansion_bytes(0, value.len())?;
            self.charge_expansion(value.len())?;
            return Ok(value.into_owned());
        }
        if unmodeled_variable(key) {
            return Err(format!("Unsupported special variable: {key}"));
        }
        let value: Cow<'_, str> = match key {
            "?" => Cow::Owned(self.outcome.status.to_string()),
            "IFS" => Cow::Borrowed(" \t\n"),
            "#" => Cow::Owned(self.args.len().to_string()),
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
        Ok(value.into_owned())
    }
    fn expand_text(&self, s: &str) -> Result<String, String> {
        self.charge_expansion(s.len())?;
        let cs: Vec<char> = s.chars().collect();
        let mut i = 0;
        let mut result = String::new();
        while i < cs.len() {
            if cs[i] != '$' {
                let mut bytes = [0; 4];
                self.append_expansion(&mut result, cs[i].encode_utf8(&mut bytes))?;
                i += 1;
                continue;
            }
            if cs[i..].starts_with(&['$', '(', '(']) {
                let end = arithmetic::end(&cs, i)?;
                let expression = cs[i + 3..end - 2].iter().collect::<String>();
                let expanded = self.expand_text(&expression)?;
                let number = arithmetic::evaluate(&expanded, |name| self.variable(name))?;
                self.append_expansion(&mut result, &number.to_string())?;
                i = end;
                continue;
            }
            i += 1;
            if i == cs.len() {
                self.append_expansion(&mut result, "$")?;
                break;
            }
            let key = if cs[i] == '{' {
                i += 1;
                let start = i;
                while i < cs.len() && cs[i] != '}' {
                    i += 1;
                }
                if i == cs.len() {
                    return Err("Close ${parameter}".into());
                }
                let key = cs[start..i].iter().collect::<String>();
                i += 1;
                key
            } else if cs[i].is_ascii_alphabetic() || cs[i] == '_' {
                let start = i;
                i += 1;
                while i < cs.len() && (cs[i].is_ascii_alphanumeric() || cs[i] == '_') {
                    i += 1;
                }
                cs[start..i].iter().collect()
            } else {
                let key = cs[i].to_string();
                i += 1;
                key
            };
            self.append_expansion(&mut result, &self.variable(&key)?)?;
        }
        Ok(result)
    }
    fn expand(&self, w: &Word, split: bool) -> Result<Vec<String>, String> {
        if w.0.len() == 1
            && w.0[0].quote == Quote::Double
            && let Some(key) = w.0[0]
                .text
                .strip_prefix("${")
                .and_then(|s| s.strip_suffix('}'))
            && !key.contains(['{', '}'])
            && let Some(ArrayRef::All(base)) = values::array_ref(key)?
        {
            let elements = self.env.get(base).map_or(&[][..], Value::elements);
            let bytes = expansion_fields_size(elements)?;
            bounded_expansion_bytes(bytes, std::mem::size_of_val(elements))?;
            self.charge_expansion(bytes + std::mem::size_of_val(elements))?;
            return Ok(elements.to_vec());
        }
        if w.0.len() == 1 && w.0[0].quote == Quote::Double && w.0[0].text == "$@" {
            let bytes = expansion_fields_size(&self.args)?;
            self.charge_expansion(bytes)?;
            return Ok(self.args.clone());
        }
        let mut fields = vec![String::new()];
        let mut unquoted = vec![Vec::new()];
        let mut pattern_possible = false;
        let mut retained = 0;
        let mut keep = false;
        let mut pending = false;
        let next_field = |fields: &mut Vec<String>| -> Result<(), String> {
            if fields.len() >= 128 {
                return Err("Argument limit exceeded (128 expanded fields)".into());
            }
            fields.push(String::new());
            Ok(())
        };
        for p in &w.0 {
            let text = if p.quote == Quote::Single {
                self.charge_expansion(p.text.len())?;
                p.text.clone()
            } else {
                self.expand_text(&p.text)?
            };
            self.charge_expansion(text.len())?;
            if split && p.quote == Quote::Bare {
                pattern_possible |= text.contains(['*', '?', '[']);
                for c in text.chars() {
                    if matches!(c, ' ' | '\t' | '\n') {
                        pending = true;
                    } else {
                        if pending && (!fields.last().unwrap().is_empty() || keep) {
                            next_field(&mut fields)?;
                            unquoted.push(Vec::new());
                            keep = false;
                        }
                        retained = bounded_expansion_bytes(retained, c.len_utf8())?;
                        fields.last_mut().unwrap().push(c);
                        let flags = unquoted.last_mut().unwrap();
                        flags.resize(flags.len() + c.len_utf8(), true);
                        pending = false;
                    }
                }
            } else {
                if pending && (!fields.last().unwrap().is_empty() || keep) {
                    next_field(&mut fields)?;
                    unquoted.push(Vec::new());
                }
                retained = bounded_expansion_bytes(retained, text.len())?;
                fields.last_mut().unwrap().push_str(&text);
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
        let mut bytes = 0;
        let mut push = |value: Cow<'_, str>| -> Result<(), String> {
            if expanded.len() >= 128 {
                return Err("Argument limit exceeded (128 expanded fields)".into());
            }
            bytes = bounded_expansion_bytes(bytes, value.len())?;
            self.charge_expansion(value.len())?;
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
    fn words(&self, words: &[Word]) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        let mut retained = 0;
        for w in words {
            let fields = self.expand(w, true)?;
            if out.len() + fields.len() > 128 {
                return Err("Argument limit exceeded (128 arguments)".into());
            }
            retained = bounded_expansion_bytes(retained, expansion_fields_size(&fields)?)?;
            out.extend(fields);
        }
        Ok(out)
    }
    pub fn tick(&mut self, budget: usize) {
        for _ in 0..budget.min(64) {
            if self.done {
                return;
            }
            if self.pc >= self.code.len() {
                self.done = true;
                return;
            }
            if self.steps >= MAX_STEPS {
                self.fail("Execution limit reached (2,000 instructions). Check the loop condition; Stop is also available.".into());
                return;
            }
            self.steps += 1;
            let code = self.code[self.pc].clone();
            self.pc += 1;
            let r = (|| -> Result<(), String> {
                match code {
                    Code::ArrayAssign(key, words) => {
                        // Expand all words using the prior binding before publishing a new list.
                        let elements = self.words(&words)?;
                        self.store_value(key, Value::Indexed(elements))?;
                        self.outcome.status = 0;
                    }
                    Code::Status(s) => self.outcome.status = s,
                    Code::WhileStart(word, end) => {
                        if let Some(word) = word {
                            let fields = self.expand(&word, true)?;
                            if fields.len() != 1 {
                                return Err(
                                    "Ambiguous loop input redirection: quote a single file path"
                                        .into(),
                                );
                            }
                            let file = path(&fields[0])?;
                            if let Some(text) = self.outcome.files.get(&file).cloned() {
                                let input = std::mem::replace(&mut self.input, text);
                                let pos = std::mem::replace(&mut self.input_pos, 0);
                                self.inputs.push(InputFrame { input, pos, file });
                                self.while_status.push((0, true));
                            } else {
                                self.outcome.status = 1;
                                self.emit(
                                    &Dest::Err,
                                    &format!("shell: {}: No such file\n", fields[0]),
                                )?;
                                self.pc = end;
                            }
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
                        if self.functions.len() >= 64 && !self.functions.contains_key(&n) {
                            return Err("Function definition limit exceeded (64)".into());
                        }
                        self.functions.insert(n, start);
                        self.pc = end;
                        self.outcome.status = 0;
                    }
                    Code::FunctionEnd => self.leave_function()?,
                    Code::Pipe(cmds) => {
                        if cmds.len() != 1 || !self.enter_function(&cmds[0])? {
                            self.pipeline(&cmds)?;
                        }
                    }
                    Code::Jump(t) => self.pc = t,
                    Code::JumpIf(success, t) => {
                        if (self.outcome.status == 0) == success {
                            self.pc = t
                        }
                    }
                    Code::For(n, words, end) => {
                        let values = self.words(&words)?;
                        if values.is_empty() {
                            self.pc = end;
                            self.outcome.status = 0;
                        } else {
                            self.charge_expansion(values[0].len())?;
                            self.assign_scalar(n.clone(), values[0].clone())?;
                            self.reserve_memory(n.len() + strings_bytes(&values))?;
                            self.loops.push((n, values, 1));
                        }
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
        self.pc = frame.pc;
        self.args = frame.args;
        self.loops.truncate(frame.loops);
        self.while_status.truncate(frame.whiles);
        while self.inputs.len() > frame.inputs {
            self.restore_input()?;
        }
        Ok(())
    }
    fn enter_function(&mut self, cmd: &Command) -> Result<bool, String> {
        if cmd.words.first().and_then(Word::literal) == Some("local") {
            return Ok(false);
        }
        if self.functions.is_empty() || cmd.words.first().is_some_and(Word::assignment) {
            return Ok(false);
        }
        let Some(start) = self
            .command_head(cmd)?
            .and_then(|n| self.functions.get(&n).copied())
        else {
            return Ok(false);
        };
        let mut args = self.words(&cmd.words)?;
        if !cmd.redirects.is_empty() {
            return Err("Unsupported redirection on a function call; redirect individual commands inside its body".into());
        }
        if self.calls.len() >= 32 {
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
        });
        self.pc = start;
        Ok(true)
    }
    fn restore_input(&mut self) -> Result<(), String> {
        let saved = self.inputs.pop().ok_or("Invalid loop input state")?;
        self.input = saved.input;
        self.input_pos = saved.pos;
        Ok(())
    }
    fn writable_file(&self, file: &str) -> Result<(), String> {
        if self.inputs.iter().any(|f| f.file == file) {
            Err("Unsupported write to an active loop input; finish reading the source before changing it".into())
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
        self.env
            .iter()
            .map(|(n, v)| n.len() + v.bytes())
            .sum::<usize>()
            + self
                .outcome
                .files
                .iter()
                .map(|(n, v)| n.len() + v.len())
                .sum::<usize>()
            + self.input.len()
            + self
                .inputs
                .iter()
                .map(|f| f.input.len() + f.file.len())
                .sum::<usize>()
            + self
                .calls
                .iter()
                .flat_map(|f| &f.locals)
                .map(|(n, v)| n.len() + v.as_ref().map_or(0, Value::bytes))
                .sum::<usize>()
            + strings_bytes(&self.args)
            + self
                .calls
                .iter()
                .map(|f| strings_bytes(&f.args))
                .sum::<usize>()
            + self
                .loops
                .iter()
                .map(|(n, v, _)| n.len() + strings_bytes(v))
                .sum::<usize>()
            + self.pipeline_bytes
    }
    fn preflight_value(&self, key: &str, bytes: usize) -> Result<(), String> {
        variable_target(key)?;
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
        let visible: BTreeSet<_> = self
            .env
            .keys()
            .chain(self.calls.iter().flat_map(|f| f.locals.keys()))
            .map(String::as_str)
            .collect();
        if !visible.contains(key) && visible.len() >= 256 {
            return Err("Variable limit exceeded (256 names)".into());
        }
        Ok(())
    }
    fn store_value(&mut self, key: String, value: Value) -> Result<(), String> {
        if value.elements().len() > 128 {
            return Err("Array element limit exceeded (128)".into());
        }
        self.preflight_value(&key, value.bytes())?;
        self.env.insert(key, value);
        Ok(())
    }
    fn assign_scalar(&mut self, key: String, value: String) -> Result<(), String> {
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
        let visible: BTreeSet<_> = self
            .env
            .keys()
            .chain(self.calls.iter().flat_map(|f| f.locals.keys()))
            .collect();
        if visible.len() > 256 || self.outcome.files.len() > 64 || self.memory_bytes() > MAX_TEXT {
            return Err("Virtual memory/file limit exceeded".into());
        }
        if self.outcome.stdout.len() + self.outcome.stderr.len() > MAX_OUTPUT {
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
            Dest::Out | Dest::Err
                if self.outcome.stdout.len() + self.outcome.stderr.len() + text.len()
                    > MAX_OUTPUT =>
            {
                return Err("Output limit reached (32 KiB)".into());
            }
            Dest::File(_)
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
        }
        self.limits()
    }
    fn command_head(&self, cmd: &Command) -> Result<Option<String>, String> {
        if cmd.words.first().is_some_and(Word::assignment) {
            return Ok(None);
        }
        for word in &cmd.words {
            if let Some(head) = self.words(std::slice::from_ref(word))?.first() {
                return Ok(Some(head.clone()));
            }
        }
        Ok(None)
    }
    fn pipeline(&mut self, cmds: &[Command]) -> Result<(), String> {
        if cmds.len() == 1 {
            let output = self.command(&cmds[0])?;
            return self.emit(&Dest::Out, &output);
        }
        let snapshot_bytes = self
            .env
            .iter()
            .map(|(n, v)| n.len() + v.bytes())
            .sum::<usize>()
            + self.args.iter().map(String::len).sum::<usize>()
            + self.args.len() * std::mem::size_of::<String>()
            + self.input.len();
        if self
            .memory_bytes()
            .checked_add(snapshot_bytes)
            .is_none_or(|n| n > MAX_TEXT)
        {
            return Err(
                "Virtual memory/file limit exceeded while retaining pipeline variables".into(),
            );
        }
        if cmds.len() > 1 {
            for cmd in cmds {
                if self.command_head(cmd)?.as_ref().is_some_and(|s| {
                    self.functions.contains_key(s) || s == "return" || s == "local"
                }) {
                    return Err("Unsupported function call, return or local declaration in a pipeline; use separate commands".into());
                }
            }
        }
        self.charge_expansion(snapshot_bytes)?;
        self.pipeline_bytes += snapshot_bytes;
        let saved_env = self.env.clone();
        let saved_args = self.args.clone();
        let saved_status = self.outcome.status;
        let saved_input = self.input.clone();
        let saved_pos = self.input_pos;
        let mut consumed = 0;
        let mut pipe = String::new();
        for (i, cmd) in cmds.iter().enumerate() {
            if cmds.len() > 1 {
                self.charge_expansion(snapshot_bytes + if i == 0 { 0 } else { pipe.len() })?;
                self.outcome.status = saved_status;
                self.env = saved_env.clone();
                self.args = saved_args.clone();
                self.input = if i == 0 {
                    saved_input[saved_pos..].to_string()
                } else {
                    pipe.clone()
                };
                self.input_pos = 0;
            }
            let output = self.command(cmd)?;
            if i == 0 && cmds.len() > 1 {
                consumed = self.input_pos;
            }
            pipe = output;
            if self.done && cmds.len() > 1 {
                self.done = false;
            }
            if i + 1 == cmds.len() {
                self.emit(&Dest::Out, &pipe)?;
            }
        }
        if cmds.len() > 1 {
            self.env = saved_env;
            self.args = saved_args;
            self.input = saved_input;
            self.input_pos = saved_pos + consumed;
        }
        self.pipeline_bytes -= snapshot_bytes;
        Ok(())
    }
    fn local_declarations(&self, cmd: &Command) -> Result<Option<Vec<LocalDeclaration>>, String> {
        if cmd.words.first().and_then(Word::literal) != Some("local") {
            return Ok(None);
        }
        if self.calls.is_empty() {
            return Err(
                "Unsupported local outside a function; local variables belong to a function call"
                    .into(),
            );
        }
        if cmd.words.len() == 1 {
            return Err("Unsupported local listing; supply a variable name or NAME=value".into());
        }
        // Bash declaration assignments suppress splitting; all arguments expand before mutation.
        let mut declarations = Vec::new();
        for word in &cmd.words[1..] {
            let assignment = word.assignment();
            let values = if assignment {
                self.expand(word, false)?
            } else {
                self.expand(word, true)?
            };
            for value in values {
                let (key, new_value) = value
                    .split_once('=')
                    .map_or((value.as_str(), None), |(key, value)| {
                        (key, Some(value.to_string()))
                    });
                variable_target(key)?;
                if !name(key) {
                    return Err("Unsupported local argument; use a valid variable name or NAME=value, without options or attributes".into());
                }
                declarations.push((key.to_string(), new_value));
                if declarations.len() > 128
                    || declarations
                        .iter()
                        .map(|(k, v)| k.len() + v.as_ref().map_or(0, String::len))
                        .sum::<usize>()
                        > MAX_TEXT
                {
                    return Err("Local declaration argument limit exceeded".into());
                }
            }
        }
        if declarations.is_empty() {
            return Err("Unsupported local listing; supply a valid variable argument".into());
        }
        Ok(Some(declarations))
    }
    fn apply_local_declarations(
        &mut self,
        declarations: Vec<LocalDeclaration>,
    ) -> Result<(), String> {
        for (key, new_value) in declarations {
            let locals = &mut self.calls.last_mut().unwrap().locals;
            if !locals.contains_key(&key) {
                let previous = self.env.remove(&key);
                let array = matches!(&previous, Some(Value::Indexed(_)));
                locals.insert(key.clone(), previous);
                if array {
                    self.env.insert(key.clone(), Value::Indexed(Vec::new()));
                }
            }
            if let Some(value) = new_value {
                self.assign_scalar(key, value)?;
            }
            self.limits()?;
        }
        Ok(())
    }
    fn command(&mut self, cmd: &Command) -> Result<String, String> {
        // Command/declaration argument expansion precedes redirection. Assignment-only
        // values remain sequential and follow the Bash no-command ordering below.
        let assignments = cmd.words.iter().all(Word::assignment);
        let declarations = self.local_declarations(cmd)?;
        let args = if declarations.is_none() && !assignments {
            Some(self.words(&cmd.words)?)
        } else {
            None
        };
        // With no command name, Bash applies assignments sequentially before
        // redirection; even a later missing input retains those assignments.
        if assignments && !cmd.words.is_empty() {
            for w in &cmd.words {
                let mut w = w.clone();
                let (n, rest) = w.0[0].text.split_once('=').unwrap();
                let n = n.to_string();
                variable_target(&n)?;
                w.0[0].text = rest.to_string();
                let value = self.expand(&w, false)?.join("");
                self.assign_scalar(n, value)?;
                self.limits()?;
            }
        }
        let mut stdout = Dest::Out;
        let mut stderr = Dest::Err;
        let mut input = None;
        for (op, w) in &cmd.redirects {
            if op == ">&2" || op == "1>&2" {
                stdout = stderr.clone();
                continue;
            }
            if op == "2>&1" {
                stderr = stdout.clone();
                continue;
            }
            let fields = self.expand(w.as_ref().unwrap(), true)?;
            if fields.len() != 1 {
                return Err("Ambiguous redirection: quote a single file path".into());
            }
            let p = path(&fields[0])?;
            if op == "<" {
                let Some(text) = self.outcome.files.get(&p) else {
                    self.outcome.status = 1;
                    let e = format!("shell: {}: No such file\n", fields[0]);
                    self.emit(&stderr, &e)?;
                    return Ok(String::new());
                };
                input = Some(text.clone());
            } else {
                self.writable_file(&p)?;
                if !self.outcome.files.contains_key(&p) && self.outcome.files.len() >= 64 {
                    return Err("Virtual file limit exceeded".into());
                }
                if !op.ends_with(">>") {
                    self.outcome.files.insert(p.clone(), String::new());
                }
                let d = Dest::File(p);
                if op.starts_with('2') {
                    stderr = d
                } else {
                    stdout = d
                }
            }
        }
        let old_input = self.input.clone();
        let old_pos = self.input_pos;
        let redirected = input.is_some();
        if let Some(s) = input {
            self.input = s;
            self.input_pos = 0;
        }
        let input_depth = self.inputs.len();
        let (out, err, status) = if let Some(declarations) = declarations {
            self.apply_local_declarations(declarations)?;
            (String::new(), String::new(), 0)
        } else if assignments && !cmd.words.is_empty() {
            (String::new(), String::new(), 0)
        } else {
            let args = args.unwrap_or_default();
            if args
                .first()
                .is_some_and(|a| a.split_once('=').is_some_and(|(n, _)| name(n)))
            {
                return Err("Assignments before a command are unsupported; put the assignment on its own line".into());
            }
            self.builtin(&args)?
        };
        if redirected && self.inputs.len() == input_depth {
            self.input = old_input;
            self.input_pos = old_pos;
        }
        self.outcome.status = status;
        let mut pipe = String::new();
        if matches!(stdout, Dest::Out) {
            pipe.push_str(&out);
        } else {
            self.emit(&stdout, &out)?;
        }
        if matches!(stderr, Dest::Out) {
            pipe.push_str(&err);
        } else {
            self.emit(&stderr, &err)?;
        }
        Ok(pipe)
    }
    fn builtin(&mut self, a: &[String]) -> Result<(String, String, i32), String> {
        let empty = String::new();
        let Some(cmd) = a.first().map(String::as_str) else {
            return Ok((empty.clone(), empty, 0));
        };
        let args = &a[1..];
        let ok = |s: String| (s, String::new(), 0);
        let usage = || Err(format!("Unsupported {cmd} arguments; see the shell subset"));
        match cmd {
            "local" => Err("Unsupported declaration command name: use literal unquoted local inside a function".into()),
            ":" | "true" | "false" if args.is_empty() => {
                Ok((empty.clone(), empty, i32::from(cmd == "false")))
            }
            "printf" => {
                if args.is_empty() || args[0].starts_with('-') {
                    return usage();
                }
                Ok(ok(format_printf(&args[0], &args[1..])?))
            }
            "echo" => {
                if args.first().is_some_and(|s| s.starts_with('-'))
                    || args.iter().any(|s| s.contains('\\'))
                {
                    return usage();
                }
                Ok(ok(format!("{}\n", args.join(" "))))
            }
            "cat" if args.len() == 1 => {
                if args[0].starts_with('-') {
                    return usage();
                }
                let p = path(&args[0])?;
                Ok(match self.outcome.files.get(&p) {
                    Some(s) => ok(s.clone()),
                    None => (
                        empty,
                        format!("cat: {}: No such file or directory\n", args[0]),
                        1,
                    ),
                })
            }
            "grep" if (args.len() == 2 || args.len() == 3) && args[0] == "-F" => {
                if args[1].starts_with('-') {
                    return Err("Unsupported grep pattern starting with -; option parsing with -e/-- is not yet implemented".into());
                }
                let text = if args.len() == 3 {
                    if args[2].starts_with('-') {
                        return usage();
                    }
                    let p = path(&args[2])?;
                    match self.outcome.files.get(&p) {
                        Some(s) => s.clone(),
                        None => {
                            return Ok((
                                empty,
                                format!("grep: {}: No such file or directory\n", args[2]),
                                2,
                            ));
                        }
                    }
                } else {
                    let t = self.input[self.input_pos..].to_string();
                    self.input_pos = self.input.len();
                    t
                };
                if args[1].contains('\n') {
                    return Err("Multi-line grep patterns are unsupported".into());
                }
                let output = text
                    .split_inclusive('\n')
                    .filter(|l| l.contains(&args[1]))
                    .map(|l| {
                        if l.ends_with('\n') {
                            l.to_string()
                        } else {
                            format!("{l}\n")
                        }
                    })
                    .collect::<String>();
                let status = i32::from(output.is_empty());
                Ok((output, empty, status))
            }
            "sort" | "uniq" | "cut" => {
                let invocation = text_tools::parse(cmd, args)?;
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
                    let p = path(file)?;
                    let Some(text) = self.outcome.files.get(&p) else {
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
                let output = invocation.execute(text, MAX_OUTPUT)?;
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
                self.assign_scalar(args[1].clone(), line.trim_matches([' ', '\t']).into())?;
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
                    [op, s] if op == "-f" => self.outcome.files.contains_key(&path(s)?),
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
                self.done = true;
                Ok((empty.clone(), empty, status))
            }
            _ => Err(format!(
                "Unsupported command: {cmd}. Only documented in-process commands are available; nothing was sent to the host."
            )),
        }
    }
}
fn format_printf(format: &str, args: &[String]) -> Result<String, String> {
    let mut out = String::new();
    let mut index = 0;
    loop {
        let mut chars = format.chars();
        let before = index;
        while let Some(c) = chars.next() {
            match c {
                '%' => match chars.next() {
                    Some('s') => {
                        out.push_str(args.get(index).map(String::as_str).unwrap_or(""));
                        index += 1;
                    }
                    Some('%') => out.push('%'),
                    _ => {
                        return Err(
                            "printf supports %s and %% only; put data in a %s argument".into()
                        );
                    }
                },
                '\\' => match chars.next() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('\\') => out.push('\\'),
                    _ => return Err("printf supports \\n, \\t and \\\\ escapes only".into()),
                },
                _ => out.push(c),
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
            if w.0.iter().any(|p| p.quote == Quote::Bare && p.text.contains(['*', '?'])) { return Err("Filename expansion is unsupported in this terminal; use a literal quoted path. Script labs have a separate supported subset.".into()); }
            if w.0.iter().any(|p| p.quote != Quote::Single && p.text.contains('$')) { return Err("Variable expansion is unsupported in this terminal; use a literal quoted path.".into()); }
            Ok(w.0.into_iter().map(|p| p.text).collect())
        },
        Token::Op(_) => Err("Shell operators are unsupported in this terminal. Enter one supported command at a time.".into()),
    }).collect()
}
