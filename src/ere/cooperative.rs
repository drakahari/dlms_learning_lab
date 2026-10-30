//! Owned, tick-sized boolean searches; no I/O and no borrowed self-reference.
use super::{Budget, Error, ErrorKind, MAX_LINE, MAX_STATES, Regex, State, Threads, err, limit};
use std::{ops::Range, sync::Arc};
const MAX_SNAPSHOT: usize = 262_144;
#[derive(Clone, Copy)]
enum Side {
    Current,
    Next,
}
#[derive(Clone, Copy)]
enum Resume {
    Initial,
    State { pos: usize, index: usize },
    Restart { pos: usize },
}
enum Phase {
    Idle,
    Validate(usize),
    Initialize,
    Closure {
        side: Side,
        pos: usize,
        edges: [Option<u16>; 2],
        edge: usize,
        resume: Resume,
    },
    InitialDecision,
    ClearNext(usize),
    Scan {
        pos: usize,
        index: usize,
    },
    RestartDecision(usize),
    Done(bool),
}
/// The graph and immutable input have one stable identity across all advances.
/// Input can contain many LF records; each started range must be one LF-free line.
pub struct Cursor {
    regex: Regex,
    input: Arc<str>,
    line: Range<usize>,
    current: Threads,
    next: Threads,
    stack: Vec<u16>,
    phase: Phase,
}
impl Regex {
    /// Worst-case temporary heap admitted before entering the bounded compiler.
    pub const fn compile_storage_bound() -> usize {
        super::MAX_COMPILE_STORAGE
    }
    /// A conservative bound, not the compiler's partial compile_work metric.
    /// Covers at most256 primitive parsing/mask units per pattern byte plus the
    /// independently admitted compiler-node visits and AST slots.
    pub fn compile_charge(pattern_bytes: usize) -> Result<usize, Error> {
        if pattern_bytes > super::MAX_PATTERN {
            return Err(limit("Pattern exceeds512 bytes"));
        }
        pattern_bytes
            .checked_mul(256)
            .and_then(|v| v.checked_add(super::MAX_COMPILE_VISITS + super::MAX_NODES))
            .ok_or_else(|| limit("Regex compile allowance overflow"))
    }
    pub fn cursor_storage_bound(&self, input_bytes: usize) -> Result<usize, Error> {
        if input_bytes > MAX_SNAPSHOT {
            return Err(limit("Regex snapshot exceeds256KiB"));
        }
        std::mem::size_of::<Cursor>()
            .checked_add(self.states.capacity() * std::mem::size_of::<State>())
            .and_then(|v| v.checked_add(input_bytes + 2 * std::mem::size_of::<usize>()))
            .and_then(|v| v.checked_add(3 * MAX_STATES * std::mem::size_of::<u16>()))
            .ok_or_else(|| limit("Regex cursor storage overflow"))
    }
    pub fn into_cursor(self, input: Arc<str>) -> Result<Cursor, Error> {
        self.cursor_storage_bound(input.len())?;
        let states = self.states.len();
        let mut stack = Vec::new();
        stack
            .try_reserve_exact(states)
            .map_err(|_| limit("Cannot reserve bounded closure storage"))?;
        if stack.capacity() > MAX_STATES {
            return Err(limit("Closure capacity exceeds its bound"));
        }
        Ok(Cursor {
            regex: self,
            input,
            line: 0..0,
            current: Threads::new(states)?,
            next: Threads::new(states)?,
            stack,
            phase: Phase::Idle,
        })
    }
}
impl Budget {
    /// Utility boundary/line-framing work shares the same allowance as matching.
    pub fn consume(&mut self, count: usize) -> Result<(), Error> {
        self.remaining = self
            .remaining
            .checked_sub(count)
            .ok_or_else(|| limit("Regex work allowance exhausted"))?;
        Ok(())
    }
}
impl Cursor {
    pub fn input(&self) -> &str {
        &self.input
    }
    pub fn retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.regex.states.capacity() * std::mem::size_of::<State>()
            + self.input.len()
            + 2 * std::mem::size_of::<usize>()
            + (self.current.ids.capacity() + self.next.ids.capacity() + self.stack.capacity())
                * std::mem::size_of::<u16>()
    }
    pub fn start_line(&mut self, range: Range<usize>) -> Result<(), Error> {
        if !matches!(self.phase, Phase::Idle | Phase::Done(_)) {
            return Err(err(
                ErrorKind::Unsupported,
                None,
                "Finish or discard the active line search before starting another",
            ));
        }
        let Some(line) = self.input.get(range.clone()) else {
            return Err(err(
                ErrorKind::Unsupported,
                None,
                "Line range must be an ordered UTF8 boundary inside the immutable snapshot",
            ));
        };
        if line.len() > MAX_LINE {
            return Err(limit("A regex input line exceeds64KiB"));
        }
        self.line = range;
        self.phase = Phase::Validate(0);
        Ok(())
    }
    pub fn advance(&mut self, slice: usize, budget: &mut Budget) -> Result<Option<bool>, Error> {
        if matches!(self.phase, Phase::Idle) {
            return Err(err(
                ErrorKind::Unsupported,
                None,
                "Start a line before advancing its search",
            ));
        }
        for _ in 0..slice.min(64) {
            if let Phase::Done(value) = self.phase {
                return Ok(Some(value));
            }
            budget.charge()?;
            let phase = std::mem::replace(&mut self.phase, Phase::Idle);
            let length = self.line.end - self.line.start;
            self.phase = match phase {
                Phase::Validate(at) => {
                    if at == length {
                        Phase::Initialize
                    } else {
                        let b = self.input.as_bytes()[self.line.start + at];
                        if matches!(b, 0 | b'\n') {
                            return Err(err(
                                ErrorKind::Unsupported,
                                Some(at),
                                "Pass one line without NUL or its terminating LF",
                            ));
                        }
                        Phase::Validate(at + 1)
                    }
                }
                Phase::Initialize => {
                    self.current.clear();
                    self.next.clear();
                    self.stack.clear();
                    Phase::Closure {
                        side: Side::Current,
                        pos: 0,
                        edges: [Some(self.regex.start), None],
                        edge: 0,
                        resume: Resume::Initial,
                    }
                }
                Phase::Closure {
                    side,
                    pos,
                    edges,
                    edge,
                    resume,
                } => {
                    let set = match side {
                        Side::Current => &mut self.current,
                        Side::Next => &mut self.next,
                    };
                    if edge < 2 {
                        if let Some(id) = edges[edge] {
                            let n = usize::from(id);
                            let bit = 1u64 << (n % 64);
                            let word = &mut set.seen[n / 64];
                            if *word & bit == 0 {
                                *word |= bit;
                                if self.stack.len() >= self.stack.capacity() {
                                    return Err(limit("Closure exceeds admitted storage"));
                                }
                                self.stack.push(id);
                            }
                        }
                        Phase::Closure {
                            side,
                            pos,
                            edges,
                            edge: edge + 1,
                            resume,
                        }
                    } else if let Some(id) = self.stack.pop() {
                        let mut next_edges = [None, None];
                        match self.regex.states[usize::from(id)] {
                            State::Accept => set.matched = true,
                            State::Byte(_, _) => {
                                if set.ids.len() >= set.ids.capacity() {
                                    return Err(limit("Active threads exceed admitted storage"));
                                }
                                set.ids.push(id);
                            }
                            State::Start(next) if pos == 0 => next_edges[0] = Some(next),
                            State::End(next) if pos == length => next_edges[0] = Some(next),
                            State::Split(a, b) => next_edges = [Some(a), Some(b)],
                            _ => {}
                        }
                        Phase::Closure {
                            side,
                            pos,
                            edges: next_edges,
                            edge: 0,
                            resume,
                        }
                    } else {
                        match resume {
                            Resume::Initial => Phase::InitialDecision,
                            Resume::State { pos, index } => Phase::Scan { pos, index },
                            Resume::Restart { pos } => Phase::RestartDecision(pos),
                        }
                    }
                }
                Phase::InitialDecision => {
                    if self.current.matched {
                        Phase::Done(true)
                    } else if length == 0 {
                        Phase::Done(false)
                    } else {
                        Phase::ClearNext(0)
                    }
                }
                Phase::ClearNext(pos) => {
                    self.next.clear();
                    self.stack.clear();
                    Phase::Scan { pos, index: 0 }
                }
                Phase::Scan { pos, index } => {
                    if let Some(&id) = self.current.ids.get(index) {
                        let State::Byte(mask, next) = self.regex.states[usize::from(id)] else {
                            return Err(limit("Invalid admitted regex thread"));
                        };
                        if mask.has(self.input.as_bytes()[self.line.start + pos]) {
                            Phase::Closure {
                                side: Side::Next,
                                pos: pos + 1,
                                edges: [Some(next), None],
                                edge: 0,
                                resume: Resume::State {
                                    pos,
                                    index: index + 1,
                                },
                            }
                        } else {
                            Phase::Scan {
                                pos,
                                index: index + 1,
                            }
                        }
                    } else {
                        Phase::Closure {
                            side: Side::Next,
                            pos: pos + 1,
                            edges: [Some(self.regex.start), None],
                            edge: 0,
                            resume: Resume::Restart { pos },
                        }
                    }
                }
                Phase::RestartDecision(pos) => {
                    std::mem::swap(&mut self.current, &mut self.next);
                    if self.current.matched {
                        Phase::Done(true)
                    } else if pos + 1 == length {
                        Phase::Done(false)
                    } else {
                        Phase::ClearNext(pos + 1)
                    }
                }
                Phase::Idle | Phase::Done(_) => unreachable!("Handled before charged search unit"),
            };
        }
        if let Phase::Done(value) = self.phase {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}
