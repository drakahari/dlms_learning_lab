//! Pure bounded C-byte ERE boolean line matching. No I/O or evaluator integration.
//! This deliberately excludes captures, backreferences and unspecified extensions.
use std::fmt;
#[path = "ere/cooperative.rs"]
mod cooperative;
pub type Cursor = cooperative::Cursor;

pub const MAX_PATTERN: usize = 512;
pub const MAX_NODES: usize = 256;
pub const MAX_DEPTH: usize = 16;
pub const MAX_STATES: usize = 1024;
pub const MAX_REPEAT: usize = 32;
pub const MAX_LINE: usize = 65_536;
pub const MAX_WORK: usize = 1_000_000;
pub const MAX_COMPILE_VISITS: usize = 16_384;
const WORDS: usize = MAX_STATES / 64;
const MAX_COMPILE_STORAGE: usize = 131_072;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Syntax,
    Unsupported,
    Limit,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub byte: Option<usize>,
    pub message: &'static str,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} ERE: {}", self.kind, self.message)?;
        if let Some(byte) = self.byte {
            write!(f, " (pattern/input byte {})", byte + 1)?;
        }
        Ok(())
    }
}
impl std::error::Error for Error {}
fn err(kind: ErrorKind, byte: Option<usize>, message: &'static str) -> Error {
    Error {
        kind,
        byte,
        message,
    }
}
fn limit(message: &'static str) -> Error {
    err(ErrorKind::Limit, None, message)
}

#[derive(Clone, Copy, Debug)]
struct Mask([u64; 4]);
impl Mask {
    fn empty() -> Self {
        Self([0; 4])
    }
    fn byte(byte: u8) -> Self {
        let mut m = Self::empty();
        m.add(byte);
        m
    }
    fn add(&mut self, byte: u8) {
        self.0[usize::from(byte) / 64] |= 1u64 << (byte % 64);
    }
    fn has(self, byte: u8) -> bool {
        self.0[usize::from(byte) / 64] & (1u64 << (byte % 64)) != 0
    }
    fn union(&mut self, other: Self) {
        for (a, b) in self.0.iter_mut().zip(other.0) {
            *a |= b;
        }
    }
    fn negate(&mut self) {
        for w in &mut self.0 {
            *w = !*w;
        }
    }
}
#[derive(Clone, Copy, Debug)]
enum Kind {
    Empty,
    Byte(Mask),
    Start,
    End,
    Concat(usize, usize),
    Alt(usize, usize),
    Repeat {
        child: usize,
        min: usize,
        max: Option<usize>,
    },
}
#[derive(Clone, Copy, Debug)]
struct Node {
    kind: Kind,
    cost: usize,
    visits: usize,
}
struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
    nodes: Vec<Node>,
    depth: usize,
}
impl<'a> Parser<'a> {
    fn node(&mut self, kind: Kind) -> Result<usize, Error> {
        let add = |a: usize, b: usize| {
            a.checked_add(b)
                .ok_or_else(|| limit("Compiled graph size overflow"))
        };
        let mul = |a: usize, b: usize| {
            a.checked_mul(b)
                .ok_or_else(|| limit("Compiled graph size overflow"))
        };
        let cost = match kind {
            Kind::Empty => 0,
            Kind::Byte(_) | Kind::Start | Kind::End => 1,
            Kind::Concat(a, b) => add(self.nodes[a].cost, self.nodes[b].cost)?,
            Kind::Alt(a, b) => add(add(self.nodes[a].cost, self.nodes[b].cost)?, 1)?,
            Kind::Repeat { child, min, max } => match max {
                Some(max) => add(mul(self.nodes[child].cost, max)?, max - min)?,
                None => add(mul(self.nodes[child].cost, min + 1)?, 1)?,
            },
        };
        let visits = if cost == 0 {
            1
        } else {
            match kind {
                Kind::Empty | Kind::Byte(_) | Kind::Start | Kind::End => 1,
                Kind::Concat(a, b) | Kind::Alt(a, b) => {
                    add(add(self.nodes[a].visits, self.nodes[b].visits)?, 1)?
                }
                Kind::Repeat { child, min, max } => {
                    add(mul(self.nodes[child].visits, max.unwrap_or(min + 1))?, 1)?
                }
            }
        };
        if self.nodes.len() >= MAX_NODES {
            return Err(limit("Pattern exceeds 256 syntax nodes"));
        }
        let id = self.nodes.len();
        self.nodes.push(Node { kind, cost, visits });
        Ok(id)
    }
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }
    fn alternate(&mut self) -> Result<usize, Error> {
        let mut left = self.concat()?.ok_or_else(|| {
            err(
                ErrorKind::Unsupported,
                Some(self.pos),
                "Empty groups and alternatives are outside this subset; use an optional item",
            )
        })?;
        while self.peek() == Some(b'|') {
            self.pos += 1;
            let right = self.concat()?.ok_or_else(|| {
                err(
                    ErrorKind::Unsupported,
                    Some(self.pos),
                    "Empty alternatives are outside this subset",
                )
            })?;
            left = self.node(Kind::Alt(left, right))?;
        }
        Ok(left)
    }
    fn concat(&mut self) -> Result<Option<usize>, Error> {
        let mut left = None;
        let mut first = true;
        while self.peek().is_some_and(|b| b != b'|' && b != b')') {
            let atom = self.atom(first)?;
            first = false;
            let atom = self.repeat(atom)?;
            left = Some(if let Some(previous) = left {
                self.node(Kind::Concat(previous, atom))?
            } else {
                atom
            });
        }
        Ok(left)
    }
    fn atom(&mut self, branch_first: bool) -> Result<usize, Error> {
        let at = self.pos;
        let byte = self.peek().expect("concat checks input");
        self.pos += 1;
        let kind = match byte {
            b'(' => {
                if self.peek() == Some(b'?') {
                    return Err(err(
                        ErrorKind::Unsupported,
                        Some(at),
                        "Lookaround, inline flags and special groups are not supported; use plain grouping",
                    ));
                }
                if self.depth >= MAX_DEPTH {
                    return Err(limit("Grouping exceeds 16 nested levels"));
                }
                self.depth += 1;
                let child = self.alternate()?;
                if self.peek() != Some(b')') {
                    return Err(err(
                        ErrorKind::Syntax,
                        Some(at),
                        "Close the opening parenthesis",
                    ));
                }
                self.pos += 1;
                self.depth -= 1;
                return Ok(child);
            }
            b'[' => return self.bracket(at),
            b'.' => Kind::Byte(Mask([u64::MAX; 4])),
            b'^' => {
                if !branch_first {
                    return Err(err(
                        ErrorKind::Unsupported,
                        Some(at),
                        "Use ^ only at the start of an alternative",
                    ));
                }
                Kind::Start
            }
            b'$' => {
                if self.peek().is_some_and(|b| b != b'|' && b != b')') {
                    return Err(err(
                        ErrorKind::Unsupported,
                        Some(at),
                        "Use $ only at the end of an alternative",
                    ));
                }
                Kind::End
            }
            b'\\' => {
                let escaped = self.peek().ok_or_else(|| {
                    err(
                        ErrorKind::Syntax,
                        Some(at),
                        "A final backslash needs a literal special character",
                    )
                })?;
                self.pos += 1;
                if !b".^$[](){}|*+?\\".contains(&escaped) {
                    return Err(err(
                        ErrorKind::Unsupported,
                        Some(at),
                        "Backreferences and ordinary backslash escapes are not supported; escape a special ERE character only",
                    ));
                }
                Kind::Byte(Mask::byte(escaped))
            }
            b'*' | b'+' | b'?' | b'{' | b'}' => {
                return Err(err(
                    ErrorKind::Unsupported,
                    Some(at),
                    "A repetition needs a preceding item; escape literal braces or operators",
                ));
            }
            _ => Kind::Byte(Mask::byte(byte)),
        };
        self.node(kind)
    }
    fn number(&mut self) -> Result<usize, Error> {
        let at = self.pos;
        let mut n = 0usize;
        while let Some(byte @ b'0'..=b'9') = self.peek() {
            n = n
                .checked_mul(10)
                .and_then(|n| n.checked_add(usize::from(byte - b'0')))
                .ok_or_else(|| limit("Repetition count overflow"))?;
            self.pos += 1;
            if n > MAX_REPEAT {
                return Err(limit("Repetition counts must be between 0 and 32"));
            }
        }
        if self.pos == at {
            return Err(err(
                ErrorKind::Unsupported,
                Some(at),
                "Use decimal interval counts; GNU {,m} and literal unescaped braces are outside this subset",
            ));
        }
        Ok(n)
    }
    fn repeat(&mut self, child: usize) -> Result<usize, Error> {
        let at = self.pos;
        let counts = match self.peek() {
            Some(b'?') => {
                self.pos += 1;
                Some((0, Some(1)))
            }
            Some(b'*') => {
                self.pos += 1;
                Some((0, None))
            }
            Some(b'+') => {
                self.pos += 1;
                Some((1, None))
            }
            Some(b'{') => {
                self.pos += 1;
                let min = self.number()?;
                let max = if self.peek() == Some(b',') {
                    self.pos += 1;
                    if self.peek() == Some(b'}') {
                        None
                    } else {
                        Some(self.number()?)
                    }
                } else {
                    Some(min)
                };
                if self.peek() != Some(b'}') {
                    return Err(err(
                        ErrorKind::Syntax,
                        Some(at),
                        "Close the bounded repetition with }",
                    ));
                }
                self.pos += 1;
                if max.is_some_and(|max| max < min) {
                    return Err(err(
                        ErrorKind::Syntax,
                        Some(at),
                        "Repetition maximum is below its minimum",
                    ));
                }
                Some((min, max))
            }
            _ => None,
        };
        let Some((min, max)) = counts else {
            return Ok(child);
        };
        if matches!(self.nodes[child].kind, Kind::Start | Kind::End) {
            return Err(err(
                ErrorKind::Unsupported,
                Some(at),
                "Repeating an anchor is outside this subset",
            ));
        }
        if self.peek().is_some_and(|b| b"?*+{".contains(&b)) {
            return Err(err(
                ErrorKind::Unsupported,
                Some(self.pos),
                "Use one repetition operator per item; lazy, possessive and stacked forms are unsupported",
            ));
        }
        self.node(Kind::Repeat { child, min, max })
    }
    fn bracket(&mut self, at: usize) -> Result<usize, Error> {
        let negative = self.peek() == Some(b'^');
        if negative {
            self.pos += 1;
        }
        if let Some(mark @ (b':' | b'.' | b'=')) = self.peek()
            && self.bytes[self.pos..]
                .windows(2)
                .position(|w| w == [mark, b']'])
                .is_some_and(|n| n > 0)
        {
            return Err(err(
                ErrorKind::Unsupported,
                Some(self.pos),
                "Character classes require outer brackets, such as [[:digit:]]; collating/equivalence forms are not supported",
            ));
        }
        let mut mask = Mask::empty();
        let mut count = 0usize;
        loop {
            if self.peek() == Some(b']') && count > 0 {
                self.pos += 1;
                break;
            }
            if self.peek().is_none() {
                return Err(err(
                    ErrorKind::Syntax,
                    Some(at),
                    "Close the bracket expression with ]",
                ));
            }
            let initial = self.pos;
            let (element, literal) = self.bracket_element()?;
            if literal == Some(b'-') && count > 0 && self.peek() != Some(b']') {
                return Err(err(
                    ErrorKind::Unsupported,
                    Some(initial),
                    "Place a literal hyphen first or last in a bracket expression",
                ));
            }
            if self.peek() == Some(b'-') && self.bytes.get(self.pos + 1) != Some(&b']') {
                self.pos += 1;
                let (_, end) = self.bracket_element()?;
                let (start, end) = literal.zip(end).ok_or_else(|| {
                    err(
                        ErrorKind::Unsupported,
                        Some(initial),
                        "Range endpoints must be literal ASCII bytes, not character classes",
                    )
                })?;
                if start == b'-' {
                    return Err(err(
                        ErrorKind::Unsupported,
                        Some(initial),
                        "Use an unambiguous literal hyphen first or last",
                    ));
                }
                if start > end {
                    return Err(err(
                        ErrorKind::Syntax,
                        Some(initial),
                        "Range start is above its end in the C locale",
                    ));
                }
                for byte in start..=end {
                    mask.add(byte);
                }
            } else {
                mask.union(element);
            }
            count += 1;
        }
        if negative {
            mask.negate();
        }
        self.node(Kind::Byte(mask))
    }
    fn bracket_element(&mut self) -> Result<(Mask, Option<u8>), Error> {
        let at = self.pos;
        let byte = self.peek().ok_or_else(|| {
            err(
                ErrorKind::Syntax,
                Some(at),
                "Missing bracket range endpoint",
            )
        })?;
        if byte == b'\\' {
            return Err(err(
                ErrorKind::Unsupported,
                Some(at),
                "Backslash inside brackets is outside this subset; use literal first ] or first/last -",
            ));
        }
        if byte == b'['
            && self
                .bytes
                .get(self.pos + 1)
                .is_some_and(|b| matches!(*b, b'.' | b'='))
        {
            return Err(err(
                ErrorKind::Unsupported,
                Some(at),
                "Collating and equivalence elements are not supported",
            ));
        }
        if byte == b'[' && self.bytes.get(self.pos + 1) == Some(&b':') {
            self.pos += 2;
            let begin = self.pos;
            while self.peek().is_some_and(|b| b.is_ascii_lowercase()) {
                self.pos += 1;
            }
            if self.bytes.get(self.pos..self.pos + 2) != Some(b":]") {
                return Err(err(
                    ErrorKind::Syntax,
                    Some(at),
                    "Use a complete [[:name:]] character class",
                ));
            }
            let name = &self.bytes[begin..self.pos];
            self.pos += 2;
            let mut mask = Mask::empty();
            for b in 0u8..=127 {
                let yes = match name {
                    b"alnum" => b.is_ascii_alphanumeric(),
                    b"alpha" => b.is_ascii_alphabetic(),
                    b"blank" => matches!(b, b' ' | b'\t'),
                    b"cntrl" => b.is_ascii_control(),
                    b"digit" => b.is_ascii_digit(),
                    b"graph" => b.is_ascii_graphic(),
                    b"lower" => b.is_ascii_lowercase(),
                    b"print" => b.is_ascii_graphic() || b == b' ',
                    b"punct" => b.is_ascii_punctuation(),
                    b"space" => matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c),
                    b"upper" => b.is_ascii_uppercase(),
                    b"xdigit" => b.is_ascii_hexdigit(),
                    _ => {
                        return Err(err(
                            ErrorKind::Unsupported,
                            Some(at),
                            "Unknown or non-POSIX character class; use one of the documented twelve C-locale classes",
                        ));
                    }
                };
                if yes {
                    mask.add(b);
                }
            }
            return Ok((mask, None));
        }
        self.pos += 1;
        Ok((Mask::byte(byte), Some(byte)))
    }
}

#[derive(Clone, Copy, Debug)]
enum State {
    Accept,
    Byte(Mask, u16),
    Start(u16),
    End(u16),
    Split(u16, u16),
}
struct Compiler<'a> {
    nodes: &'a [Node],
    states: Vec<State>,
    visits: usize,
}
impl Compiler<'_> {
    fn push(&mut self, state: State) -> Result<u16, Error> {
        if self.states.len() >= MAX_STATES {
            return Err(limit("Compiled expression exceeds 1024 states"));
        }
        if self.states.len() >= self.states.capacity() {
            return Err(limit("Compiled state count exceeds its admitted capacity"));
        }
        let id = self.states.len() as u16;
        self.states.push(state);
        Ok(id)
    }
    fn node(&mut self, id: usize, next: u16) -> Result<u16, Error> {
        self.visits += 1;
        if self.visits > MAX_COMPILE_VISITS {
            return Err(limit("Regex compilation work exceeds its admitted bound"));
        }
        // Exact zero-state subtrees are epsilon only, including nested {0} forms.
        // Do not traverse exponentially many copies of an empty expression.
        if self.nodes[id].cost == 0 {
            return Ok(next);
        }
        Ok(match self.nodes[id].kind {
            Kind::Empty => next,
            Kind::Byte(mask) => self.push(State::Byte(mask, next))?,
            Kind::Start => self.push(State::Start(next))?,
            Kind::End => self.push(State::End(next))?,
            Kind::Concat(a, b) => {
                let tail = self.node(b, next)?;
                self.node(a, tail)?
            }
            Kind::Alt(a, b) => {
                let left = self.node(a, next)?;
                let right = self.node(b, next)?;
                self.push(State::Split(left, right))?
            }
            Kind::Repeat { child, min, max } => {
                let mut tail = next;
                if let Some(max) = max {
                    for _ in min..max {
                        let one = self.node(child, tail)?;
                        tail = self.push(State::Split(one, tail))?;
                    }
                } else {
                    let split = self.push(State::Split(0, 0))?;
                    let one = self.node(child, split)?;
                    self.states[usize::from(split)] = State::Split(one, next);
                    tail = split;
                }
                for _ in 0..min {
                    tail = self.node(child, tail)?;
                }
                tail
            }
        })
    }
}
#[derive(Debug)]
pub struct Regex {
    states: Vec<State>,
    start: u16,
    compile_work: usize,
}
impl Regex {
    pub fn compile(pattern: &str) -> Result<Self, Error> {
        if pattern.len() > MAX_PATTERN {
            return Err(limit("Pattern exceeds 512 bytes"));
        }
        if let Some((at, _)) = pattern
            .bytes()
            .enumerate()
            .find(|(_, b)| !b.is_ascii() || matches!(*b, 0 | b'\n'))
        {
            return Err(err(
                ErrorKind::Unsupported,
                Some(at),
                "Patterns use ASCII bytes without NUL or LF; input is matched using C-locale bytes",
            ));
        }
        let mut nodes = Vec::new();
        nodes
            .try_reserve_exact(MAX_NODES)
            .map_err(|_| limit("Cannot reserve bounded syntax storage"))?;
        if nodes.capacity() > MAX_NODES {
            return Err(limit("Syntax storage capacity exceeds its bound"));
        }
        let mut p = Parser {
            bytes: pattern.as_bytes(),
            pos: 0,
            nodes,
            depth: 0,
        };
        let root = if pattern.is_empty() {
            p.node(Kind::Empty)?
        } else {
            p.alternate()?
        };
        if p.pos != pattern.len() {
            return Err(err(
                ErrorKind::Unsupported,
                Some(p.pos),
                "An unmatched closing parenthesis is outside this ERE subset",
            ));
        }
        let count = p.nodes[root]
            .cost
            .checked_add(1)
            .ok_or_else(|| limit("Compiled graph size overflow"))?;
        if count > MAX_STATES {
            return Err(limit(
                "Compiled expression exceeds 1024 states before repetition expansion",
            ));
        }
        if p.nodes[root].visits > MAX_COMPILE_VISITS {
            return Err(limit(
                "Regex compilation exceeds 16384 visits before repetition expansion",
            ));
        }
        let request = p
            .nodes
            .capacity()
            .checked_mul(std::mem::size_of::<Node>())
            .and_then(|n| {
                count
                    .checked_mul(std::mem::size_of::<State>())
                    .and_then(|s| n.checked_add(s))
            })
            .ok_or_else(|| limit("Regex storage size overflow"))?;
        if request > MAX_COMPILE_STORAGE {
            return Err(limit("Regex compiler storage exceeds 128 KiB"));
        }
        let mut states = Vec::new();
        states
            .try_reserve_exact(count)
            .map_err(|_| limit("Cannot reserve bounded state storage"))?;
        if states.capacity() > MAX_STATES {
            return Err(limit("Compiled storage capacity exceeds its bound"));
        }
        let actual_storage = p
            .nodes
            .capacity()
            .checked_mul(std::mem::size_of::<Node>())
            .and_then(|n| {
                states
                    .capacity()
                    .checked_mul(std::mem::size_of::<State>())
                    .and_then(|s| n.checked_add(s))
            })
            .ok_or_else(|| limit("Regex actual storage overflow"))?;
        if actual_storage > MAX_COMPILE_STORAGE {
            return Err(limit("Regex actual compiler storage exceeds 128 KiB"));
        }
        let mut compiler = Compiler {
            nodes: &p.nodes,
            states,
            visits: 0,
        };
        let accept = compiler.push(State::Accept)?;
        let start = compiler.node(root, accept)?;
        if compiler.states.len() != count {
            return Err(limit("Compiled state accounting mismatch"));
        }
        let compile_work = compiler.visits + pattern.len() + p.nodes.len();
        Ok(Self {
            states: compiler.states,
            start,
            compile_work,
        })
    }
    pub fn state_count(&self) -> usize {
        self.states.len()
    }
    pub fn compile_work(&self) -> usize {
        self.compile_work
    }
    pub fn retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.states.capacity() * std::mem::size_of::<State>()
    }
    pub fn matcher(&self) -> Result<Matcher<'_>, Error> {
        Matcher::new(self)
    }
}

/// One caller-owned allowance for a whole filter operation, not a fresh allowance per line.
#[derive(Debug)]
pub struct Budget {
    remaining: usize,
    initial: usize,
}
impl Default for Budget {
    fn default() -> Self {
        Self {
            remaining: MAX_WORK,
            initial: MAX_WORK,
        }
    }
}
impl Budget {
    pub fn new(steps: usize) -> Result<Self, Error> {
        if steps > MAX_WORK {
            return Err(limit(
                "Regex allowance cannot exceed one million transitions",
            ));
        }
        Ok(Self {
            remaining: steps,
            initial: steps,
        })
    }
    pub fn spent(&self) -> usize {
        self.initial - self.remaining
    }
    pub fn remaining(&self) -> usize {
        self.remaining
    }
    fn charge(&mut self) -> Result<(), Error> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
            limit("Regex work allowance exhausted; simplify the pattern or input")
        })?;
        Ok(())
    }
}
struct Threads {
    ids: Vec<u16>,
    seen: [u64; WORDS],
    matched: bool,
}
impl Threads {
    fn new(states: usize) -> Result<Self, Error> {
        let mut ids = Vec::new();
        ids.try_reserve_exact(states)
            .map_err(|_| limit("Cannot reserve bounded thread storage"))?;
        if ids.capacity() > MAX_STATES {
            return Err(limit("Thread capacity exceeds its bound"));
        }
        Ok(Self {
            ids,
            seen: [0; WORDS],
            matched: false,
        })
    }
    fn clear(&mut self) {
        self.ids.clear();
        self.seen.fill(0);
        self.matched = false;
    }
    fn visit(&mut self, id: u16, budget: &mut Budget, stack: &mut Vec<u16>) -> Result<(), Error> {
        budget.charge()?;
        let id_usize = usize::from(id);
        let bit = 1u64 << (id_usize % 64);
        let word = &mut self.seen[id_usize / 64];
        if *word & bit == 0 {
            *word |= bit;
            if stack.len() >= stack.capacity() {
                return Err(limit("Closure count exceeds its admitted capacity"));
            }
            stack.push(id);
        }
        Ok(())
    }
}
pub struct Matcher<'a> {
    regex: &'a Regex,
    current: Threads,
    next: Threads,
    stack: Vec<u16>,
}
impl<'a> Matcher<'a> {
    fn new(regex: &'a Regex) -> Result<Self, Error> {
        let states = regex.states.len();
        let mut stack = Vec::new();
        stack
            .try_reserve_exact(states)
            .map_err(|_| limit("Cannot reserve bounded closure storage"))?;
        if stack.capacity() > MAX_STATES {
            return Err(limit("Closure capacity exceeds its bound"));
        }
        Ok(Self {
            regex,
            current: Threads::new(states)?,
            next: Threads::new(states)?,
            stack,
        })
    }
    pub fn scratch_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + (self.current.ids.capacity() + self.next.ids.capacity() + self.stack.capacity())
                * std::mem::size_of::<u16>()
    }
    fn closure(
        states: &[State],
        start: u16,
        pos: usize,
        length: usize,
        set: &mut Threads,
        stack: &mut Vec<u16>,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        debug_assert!(stack.is_empty());
        set.visit(start, budget, stack)?;
        while let Some(id) = stack.pop() {
            budget.charge()?;
            match states[usize::from(id)] {
                State::Accept => set.matched = true,
                State::Byte(_, _) => {
                    if set.ids.len() >= set.ids.capacity() {
                        return Err(limit("Active thread count exceeds its admitted capacity"));
                    }
                    set.ids.push(id)
                }
                State::Start(next) => {
                    if pos == 0 {
                        set.visit(next, budget, stack)?;
                    }
                }
                State::End(next) => {
                    if pos == length {
                        set.visit(next, budget, stack)?;
                    }
                }
                State::Split(a, b) => {
                    set.visit(a, budget, stack)?;
                    set.visit(b, budget, stack)?;
                }
            }
        }
        Ok(())
    }
    /// Boolean search of one LF-stripped line. No captures or match-span ordering is exposed.
    pub fn matches_line(&mut self, line: &str, budget: &mut Budget) -> Result<bool, Error> {
        if line.len() > MAX_LINE {
            return Err(limit("A regex input line exceeds 64 KiB"));
        }
        for (at, byte) in line.bytes().enumerate() {
            budget.charge()?;
            if matches!(byte, 0 | b'\n') {
                return Err(err(
                    ErrorKind::Unsupported,
                    Some(at),
                    "Pass one line without NUL or its terminating LF",
                ));
            }
        }
        self.current.clear();
        self.next.clear();
        self.stack.clear();
        Self::closure(
            &self.regex.states,
            self.regex.start,
            0,
            line.len(),
            &mut self.current,
            &mut self.stack,
            budget,
        )?;
        if self.current.matched {
            return Ok(true);
        }
        for (pos, byte) in line.bytes().enumerate() {
            budget.charge()?;
            self.next.clear();
            for &id in &self.current.ids {
                budget.charge()?;
                if let State::Byte(mask, next) = self.regex.states[usize::from(id)]
                    && mask.has(byte)
                {
                    Self::closure(
                        &self.regex.states,
                        next,
                        pos + 1,
                        line.len(),
                        &mut self.next,
                        &mut self.stack,
                        budget,
                    )?;
                }
            }
            Self::closure(
                &self.regex.states,
                self.regex.start,
                pos + 1,
                line.len(),
                &mut self.next,
                &mut self.stack,
                budget,
            )?;
            std::mem::swap(&mut self.current, &mut self.next);
            if self.current.matched {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
