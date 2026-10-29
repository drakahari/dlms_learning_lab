//! Immutable lexical loop targets; no new instructions, dynamic scope escape,
//! host execution, result strings, or independently refreshed execution budgets.
use super::*;

#[derive(Clone, Copy, Debug)]
enum Kind {
    For,
    While,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Site {
    entry: usize,
    end: usize,
    next: usize,
    owner: Option<usize>,
    body: usize,
    kind: Kind,
}
/// A function's source may sit inside a caller loop, but it is not that loop's
/// execution scope. Command-substitution bodies have disjoint arena ranges.
fn owner(program: &Program, pc: usize) -> Option<usize> {
    program
        .code
        .iter()
        .filter_map(|code| {
            if let Code::Define(_, start, end) = code {
                (*start <= pc && pc < *end).then_some(*start)
            } else {
                None
            }
        })
        .max()
}
fn metadata_bytes(count: usize) -> Result<usize, String> {
    count
        .checked_mul(std::mem::size_of::<Site>())
        .filter(|bytes| *bytes <= MAX_TEXT)
        .ok_or_else(|| "Virtual loop metadata exceeds existing memory limit".into())
}
pub(super) fn sites(program: &Program) -> Result<Vec<Site>, String> {
    let count = program
        .code
        .iter()
        .filter(|code| matches!(code, Code::For(_, _, _) | Code::WhileStart(_, _)))
        .count();
    // Checked before reserving, not merely charged after constructing metadata.
    metadata_bytes(count)?;
    let mut sites = Vec::new();
    sites
        .try_reserve_exact(count)
        .map_err(|_| "Unable to reserve bounded loop metadata")?;
    metadata_bytes(sites.capacity())?;
    for body in std::iter::once(&program.main).chain(&program.bodies) {
        for entry in body.clone() {
            let (end, next, kind) = match program.code[entry] {
                Code::For(_, _, end) => (end, end - 1, Kind::For),
                Code::WhileStart(_, end) => (end, entry + 1, Kind::While),
                _ => continue,
            };
            // Targets come exclusively from the existing typed compiler; never
            // learner-controlled integer offsets or command-text recognition.
            debug_assert!(entry < next && next < end && end <= body.end);
            sites.push(Site {
                entry,
                end,
                next,
                owner: owner(program, entry),
                body: body.start,
                kind,
            });
        }
    }
    debug_assert_eq!(sites.len(), count);
    Ok(sites)
}
#[derive(Clone, Copy)]
enum Transfer {
    Break,
    Continue,
}
impl Vm {
    pub(super) fn loop_control(&mut self, args: &[String]) -> Result<(), String> {
        let action = match args.first().map(String::as_str) {
            Some("break") => Transfer::Break,
            Some("continue") => Transfer::Continue,
            _ => return Err("Invalid loop-control dispatch".into()),
        };
        let count = match &args[1..] {
            [] => 1,
            [value] if !value.is_empty()
                && !value.starts_with('0')
                && value.len() <= 2
                && value.bytes().all(|b| b.is_ascii_digit()) => {
                    let count = value.parse::<usize>().map_err(|_| "Unsupported loop count")?;
                    if !(1..=24).contains(&count) {
                        return Err("Unsupported loop count; use canonical decimal 1 through 24".into());
                    }
                    count
                },
            _ => return Err("Unsupported loop-control arguments; use break/continue with an optional canonical decimal 1 through 24".into()),
        };
        let (target, fors, whiles) = {
            let _scratch = self.scratch(std::mem::size_of::<[Option<Site>; 24]>())?;
            let pc = self
                .pc
                .checked_sub(1)
                .ok_or("Invalid loop-control location")?;
            self.charge_expansion(self.program.code.len() + self.program.loop_sites.len())?;
            let scope = owner(&self.program, pc);
            // An inherited function can be defined in the parent's arena body
            // while running inside a capture. Its own loops still belong to its
            // definition, not the capture's top-level range or the caller loop.
            let body = std::iter::once(&self.program.main)
                .chain(&self.program.bodies)
                .find(|body| body.contains(&pc))
                .ok_or("Invalid loop-control body")?
                .start;
            // Syntax nesting is already limited to24. Keep a fixed stack rather
            // than allocating from N or duplicating retained for-values/input text.
            let mut active = [None; 24];
            let mut len = 0;
            for site in &self.program.loop_sites {
                if site.body == body && site.owner == scope && site.entry < pc && pc < site.end {
                    if len == active.len() {
                        return Err("Loop-control nesting limit exceeded (24)".into());
                    }
                    active[len] = Some(*site);
                    len += 1;
                }
            }
            if len == 0 {
                return Err("Unsupported loop control outside an active loop in this scope; caller loops across functions or command substitutions cannot be controlled".into());
            }
            let target_index = len.saturating_sub(count);
            let target = active[target_index].ok_or("Invalid loop-control target")?;
            let first_removed = target_index + usize::from(matches!(action, Transfer::Continue));
            // Calculate/validate all unwind counts before changing any loop stack.
            let mut fors = 0;
            let mut whiles = 0;
            for site in active[first_removed..len].iter().flatten() {
                match site.kind {
                    Kind::For => fors += 1,
                    Kind::While => whiles += 1,
                }
            }
            (target, fors, whiles)
        };
        let for_base = self.calls.last().map_or(0, |call| call.loops);
        let while_base = self.calls.last().map_or(0, |call| call.whiles);
        if self.loops.len().saturating_sub(for_base) < fors
            || self.while_status.len().saturating_sub(while_base) < whiles
        {
            return Err("Invalid loop-control stack boundary".into());
        }
        // Reverse lexical nesting is runtime nesting, even for mixed for/while
        // loops. Each redirected while restores its exact outer cursor/identity.
        self.loops.truncate(self.loops.len() - fors);
        for _ in 0..whiles {
            let (_, redirected) = self.while_status.pop().ok_or("Invalid while state")?;
            if redirected {
                self.restore_input()?;
            }
        }
        if matches!(action, Transfer::Continue) && matches!(target.kind, Kind::While) {
            self.while_status
                .last_mut()
                .ok_or("Invalid while continuation")?
                .0 = 0;
        }
        self.pc = match action {
            Transfer::Break => target.end,
            Transfer::Continue => target.next,
        };
        self.outcome.status = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metadata_has_existing_targets_and_distinct_function_and_capture_owners() {
        let program = compile("for x in a; do f() { while true; do break; done; }; f; v=$(for z in q; do continue; done); done").unwrap();
        assert_eq!(program.loop_sites.len(), 3);
        assert_eq!(program.loop_sites[0].owner, None);
        assert!(program.loop_sites[1].owner.is_some());
        assert_ne!(program.loop_sites[0].body, program.loop_sites[2].body);
        for site in &program.loop_sites {
            assert!(site.next > site.entry && site.next < site.end);
        }
        assert!(
            program.text_bytes() >= program.loop_sites.capacity() * std::mem::size_of::<Site>()
        );
    }
    #[test]
    fn metadata_adds_no_semantic_instructions() {
        let program = compile("for x in a b; do :; done; while false; do :; done").unwrap();
        assert_eq!(program.code.len(), 10);
        assert_eq!(program.loop_sites.len(), 2);
        assert!(matches!(program.code[0], Code::For(_, _, 3)));
        assert!(matches!(program.code[2], Code::Next(1)));
        assert!(matches!(program.code[3], Code::WhileStart(None, 10)));
    }
    #[test]
    fn metadata_count_is_checked_before_allocation_at_exact_and_over_existing_bound() {
        let at = MAX_TEXT / std::mem::size_of::<Site>();
        assert_eq!(
            metadata_bytes(at).unwrap(),
            at * std::mem::size_of::<Site>()
        );
        assert!(metadata_bytes(at + 1).is_err());
        assert!(metadata_bytes(usize::MAX).is_err());
        assert_eq!(metadata_bytes(0).unwrap(), 0);
    }
}
