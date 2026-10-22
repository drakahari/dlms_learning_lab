//! Bounded, pure shell-subset interpreter. No host I/O, subprocesses or network.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const MAX_SOURCE: usize = 16_384;
const MAX_TEXT: usize = 262_144;
const MAX_OUTPUT: usize = 32_768;
const MAX_STEPS: usize = 2000;
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
            && self.env.keys().all(|k| name(k))
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
                + self.env.values().map(String::len).sum::<usize>()
                + self.files.values().map(String::len).sum::<usize>()
                + self.args.iter().map(String::len).sum::<usize>()
                < MAX_TEXT / 2
            && self.stdout.len() + self.stderr.len() <= MAX_OUTPUT
            && self.after.values().map(String::len).sum::<usize>() < MAX_TEXT / 2
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
    pub source: String,
    pub results: Vec<Outcome>,
}
impl Record {
    pub fn valid(&self) -> bool {
        self.source.len() <= MAX_SOURCE
            && self.results.len() <= 8
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
            word(&mut tokens, &mut parts, &mut bare);
            if c == '\n' {
                tokens.push(Token::Op("\n".into()));
                i += 1;
                continue;
            }
            let mut op = c.to_string();
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
                "(", ")", ";", "|", "&&", "||", "<", ">", ">>", "1>", "1>>", "2>", "2>>", ">&2",
                "1>&2", "2>&1",
            ]
            .contains(&op.as_str())
            {
                return Err(format!("Unsupported operator: {op}"));
            }
            tokens.push(Token::Op(op));
            continue;
        }
        if matches!(c, '*' | '?' | '~') {
            if c == '?' && bare.ends_with('$') {
                bare.push(c);
                i += 1;
                continue;
            }
            return Err(
                "Globs and tilde expansion are unsupported; quote literal characters".into(),
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
                    && p.text.contains(['[', ']'])
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
    Jump(usize),
    JumpIf(bool, usize),
    For(String, Vec<Word>, usize),
    Next(usize),
    Status(i32),
    WhileStart,
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
            self.code.push(Code::WhileStart);
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
            self.code.push(Code::WhileBody);
            self.code.push(Code::Jump(start));
            self.patch(branch);
            self.code.push(Code::WhileEnd);
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
struct CallFrame {
    pc: usize,
    args: Vec<String>,
    loops: usize,
    whiles: usize,
}
pub struct Vm {
    code: Vec<Code>,
    pc: usize,
    steps: usize,
    env: BTreeMap<String, String>,
    args: Vec<String>,
    input: String,
    input_pos: usize,
    loops: Vec<(String, Vec<String>, usize)>,
    while_status: Vec<i32>,
    functions: BTreeMap<String, usize>,
    calls: Vec<CallFrame>,
    pub outcome: Outcome,
    done: bool,
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
            env: case.env.clone(),
            args: case.args.clone(),
            input: case.stdin.clone(),
            input_pos: 0,
            loops: Vec::new(),
            while_status: Vec::new(),
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
    fn variable(&self, key: &str) -> Result<String, String> {
        Ok(match key {
            "?" => self.outcome.status.to_string(),
            "#" => self.args.len().to_string(),
            "@" | "*" | "0" => {
                return Err("Only sole quoted \"$@\", $1…$9, $# and $? are supported".into());
            }
            s if s.len() == 1 && s.as_bytes()[0].is_ascii_digit() => self
                .args
                .get(s.parse::<usize>().unwrap() - 1)
                .cloned()
                .unwrap_or_default(),
            s if name(s) => self.env.get(s).cloned().unwrap_or_default(),
            _ => return Err("Unsupported parameter expansion".into()),
        })
    }
    fn expand_text(&self, s: &str) -> Result<String, String> {
        let cs: Vec<char> = s.chars().collect();
        let mut i = 0;
        let mut result = String::new();
        while i < cs.len() {
            if cs[i] != '$' {
                result.push(cs[i]);
                i += 1;
                continue;
            }
            i += 1;
            if i == cs.len() {
                result.push('$');
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
            result.push_str(&self.variable(&key)?);
            if result.len() > MAX_TEXT {
                return Err("Expansion exceeds memory limit".into());
            }
        }
        Ok(result)
    }
    fn expand(&self, w: &Word, split: bool) -> Result<Vec<String>, String> {
        if w.0.len() == 1 && w.0[0].quote == Quote::Double && w.0[0].text == "$@" {
            return Ok(self.args.clone());
        }
        let mut fields = vec![String::new()];
        let mut keep = false;
        let mut pending = false;
        for p in &w.0 {
            let text = if p.quote == Quote::Single {
                p.text.clone()
            } else {
                self.expand_text(&p.text)?
            };
            if split && p.quote == Quote::Bare {
                if text.contains(['*', '?']) {
                    return Err("Globbing is unsupported; quote the expanded value".into());
                }
                for c in text.chars() {
                    if matches!(c, ' ' | '\t' | '\n') {
                        pending = true;
                    } else {
                        if pending && (!fields.last().unwrap().is_empty() || keep) {
                            fields.push(String::new());
                            keep = false;
                        }
                        fields.last_mut().unwrap().push(c);
                        pending = false;
                    }
                }
            } else {
                if pending && (!fields.last().unwrap().is_empty() || keep) {
                    fields.push(String::new());
                }
                fields.last_mut().unwrap().push_str(&text);
                keep = true;
                pending = false;
            }
        }
        if !keep && fields.last().is_some_and(String::is_empty) {
            fields.pop();
        }
        Ok(fields)
    }
    fn words(&self, words: &[Word]) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for w in words {
            out.extend(self.expand(w, true)?);
            if out.len() > 128 || out.iter().map(String::len).sum::<usize>() > MAX_TEXT {
                return Err("Argument limit exceeded".into());
            }
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
                    Code::Status(s) => self.outcome.status = s,
                    Code::WhileStart => self.while_status.push(0),
                    Code::WhileBody => {
                        *self.while_status.last_mut().ok_or("Invalid while state")? =
                            self.outcome.status
                    }
                    Code::WhileEnd => {
                        self.outcome.status =
                            self.while_status.pop().ok_or("Invalid while state")?
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
                            self.env.insert(n.clone(), values[0].clone());
                            self.loops.push((n, values, 1));
                        }
                    }
                    Code::Next(start) => {
                        let (n, values, index) =
                            self.loops.last_mut().ok_or("Invalid loop state")?;
                        if *index < values.len() {
                            self.env.insert(n.clone(), values[*index].clone());
                            *index += 1;
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
        self.pc = frame.pc;
        self.args = frame.args;
        self.loops.truncate(frame.loops);
        self.while_status.truncate(frame.whiles);
        Ok(())
    }
    fn enter_function(&mut self, cmd: &Command) -> Result<bool, String> {
        if self.functions.is_empty()
            || cmd
                .words
                .first()
                .and_then(Word::literal)
                .is_some_and(|s| s.split_once('=').is_some_and(|(n, _)| name(n)))
        {
            return Ok(false);
        }
        let args = self.words(&cmd.words)?;
        let Some(start) = args.first().and_then(|n| self.functions.get(n)).copied() else {
            return Ok(false);
        };
        if !cmd.redirects.is_empty() {
            return Err("Unsupported redirection on a function call; redirect individual commands inside its body".into());
        }
        if self.calls.len() >= 32 {
            return Err("Function call depth limit exceeded (32)".into());
        }
        let old_args = std::mem::replace(&mut self.args, args[1..].to_vec());
        self.calls.push(CallFrame {
            pc: self.pc,
            args: old_args,
            loops: self.loops.len(),
            whiles: self.while_status.len(),
        });
        self.pc = start;
        Ok(true)
    }
    fn limits(&self) -> Result<(), String> {
        if self.env.len() > 256
            || self.outcome.files.len() > 64
            || self.env.values().map(String::len).sum::<usize>()
                + self.outcome.files.values().map(String::len).sum::<usize>()
                + self.args.iter().map(String::len).sum::<usize>()
                + self
                    .calls
                    .iter()
                    .flat_map(|f| &f.args)
                    .map(String::len)
                    .sum::<usize>()
                + self
                    .loops
                    .iter()
                    .flat_map(|(_, values, _)| values)
                    .map(String::len)
                    .sum::<usize>()
                > MAX_TEXT
        {
            return Err("Virtual memory/file limit exceeded".into());
        }
        if self.outcome.stdout.len() + self.outcome.stderr.len() > MAX_OUTPUT {
            return Err("Output limit reached (32 KiB)".into());
        }
        Ok(())
    }
    fn emit(&mut self, dest: &Dest, text: &str) -> Result<(), String> {
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
                if self.env.values().map(String::len).sum::<usize>()
                    + self.outcome.files.values().map(String::len).sum::<usize>()
                    + text.len()
                    > MAX_TEXT =>
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
    fn pipeline(&mut self, cmds: &[Command]) -> Result<(), String> {
        if cmds.len() > 1 {
            for cmd in cmds {
                let args = self.words(&cmd.words)?;
                if args
                    .first()
                    .is_some_and(|s| self.functions.contains_key(s) || s == "return")
                {
                    return Err("Unsupported function call or return in a pipeline; call functions as separate commands".into());
                }
            }
        }
        let saved_env = self.env.clone();
        let saved_args = self.args.clone();
        let saved_status = self.outcome.status;
        let saved_input = self.input.clone();
        let saved_pos = self.input_pos;
        let mut pipe = String::new();
        for (i, cmd) in cmds.iter().enumerate() {
            if cmds.len() > 1 {
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
            self.input_pos = saved_pos;
        }
        Ok(())
    }
    fn command(&mut self, cmd: &Command) -> Result<String, String> {
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
        // Assignment-only commands preserve quoting but never perform field splitting.
        let assignments = cmd.words.iter().all(|w| {
            w.0.first().is_some_and(|p| {
                p.quote == Quote::Bare && p.text.split_once('=').is_some_and(|(n, _)| name(n))
            })
        });
        let (out, err, status) = if assignments && !cmd.words.is_empty() {
            for w in &cmd.words {
                let mut w = w.clone();
                let (n, rest) = w.0[0].text.split_once('=').unwrap();
                let n = n.to_string();
                w.0[0].text = rest.to_string();
                let value = self.expand(&w, false)?.join("");
                self.env.insert(n, value);
                self.limits()?;
            }
            (String::new(), String::new(), 0)
        } else {
            let args = self.words(&cmd.words)?;
            if args
                .first()
                .is_some_and(|a| a.split_once('=').is_some_and(|(n, _)| name(n)))
            {
                return Err("Assignments before a command are unsupported; put the assignment on its own line".into());
            }
            self.builtin(&args)?
        };
        if redirected {
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
            "read" if args.len() == 2 && args[0] == "-r" && name(&args[1]) => {
                let rest = &self.input[self.input_pos..];
                let (line, status, used) = if let Some(i) = rest.find('\n') {
                    (&rest[..i], 0, i + 1)
                } else {
                    (rest, 1, rest.len())
                };
                self.env
                    .insert(args[1].clone(), line.trim_matches([' ', '\t']).into());
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
                "Unsupported command: {cmd}. Only documented in-process builtins are available; nothing was sent to the host."
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
    cases.iter().enumerate().map(|(i,c)|{let r=record.as_ref().filter(|r|r.source==source).and_then(|r|r.results.get(i));let pass=r.is_some_and(|r|c.accepts(r));let message=if pass{format!("Pass · {}",c.name)}else{format!("{} · {}",c.name,match r{None=>"Run the current script against this case".into(),Some(r)=>if let Some(e)=&r.error{e.clone()}else if r.status!=c.status{format!("Exit status {}; expected {}. Inspect the last command or an early exit.",r.status,c.status)}else if r.stdout!=c.stdout{"Standard output differs. Check spacing, quoting and line breaks.".into()}else if r.stderr!=c.stderr{"Error output differs. Send the requested diagnostic to standard error.".into()}else{"Virtual files differ. Check the destination, append/truncate behavior and untouched input files.".into()}})};(pass,message)}).collect()
}
