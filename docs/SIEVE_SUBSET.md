# Sieve filtering simulation

Initial subset implemented,2026-11-14. Three registered lessons in470labs/186units
teach header filing, compound conditions and rule-order repair. Their nine teaching
screens were individually reviewed; development native162-action/48-capture and
normal two-process draft/result reopening checks pass. Current source gates pass:30scoped runtime,8UI,5normal-catalog,1319Python and
one-job all-targetClippy with warnings denied. An installed470candidate is now verified, including private upgrade, native162/48
and normal two-process partial-work reopening. See CHECKPOINT.md for its protected
launcher and exact identity. This is partial LPIC-2 objective211.2, not a complete Sieve implementation.

## Language and messages

The in-process tokenizer, typed parser and evaluator implement:

- `require "fileinto";` before other statements; capability strings are literal
  and case-sensitive. Other required capabilities prevent any evaluation.
- `if`, `elsif`, `else`; `allof`, `anyof`, `not`, `true`, `false`.
- `header` with `:is`, `:contains`, `:matches` and optional `:comparator`.
  Supported comparators are `i;ascii-casemap` (default) and `i;octet`.
  Header names always compare without ASCII case distinctions. Values discard
  leading/trailing space/tab; keys remain literal. Duplicate header fields and
  string lists use any matching combination. Missing fields never match a key,
  even an empty key. Invalid header-name arguments match nothing.
- `exists` requires every named field. `size :over` and `size :under` are strict
  comparisons; equality satisfies neither. K/M/G mean powers of1024.
- `keep`, `fileinto`, `discard`, `stop`; quoted single-line strings, string lists,
  hash/block comments, semicolons and braces. Keywords/tags ignore ASCII case.
  Unknown quoted escapes lose the backslash per the base language; there is no
  shell interpolation. Glob matching is whole-value, octet-based, with escaped
  wildcards and bounded dynamic programming, not recursive backtracking.

Sample messages have named, already-unfolded, plain ASCII headers and a body.
Encoded/multiline/non-ASCII header values are explicitly unsupported, rather than
silently decoded or miscompared. The body is not searched. Message size includes
represented header names, colon/space, values, CRLF lines, the blank separator and
body bytes; LF body lines count as canonical CRLF. Native script statement/comment
lines accept LF or CRLF. Multiline quoted values and `text:` strings are explicitly
unsupported. A lone carriage return outside a comment is rejected.

Mailboxes are supplied virtual names, not filesystem paths. INBOX ignores ASCII
case; other names are exact. Only the supplied destinations may be used. No host
file, actual mail store, real delivery, process, network or service is accessed.

## Actions, errors and grading

Filing does not itself stop subsequent rules. Actions accumulate until `stop` or
the script ends. Duplicate delivery to the same mailbox coalesces. `discard`
cancels implicit keep; it does not undo earlier filing/keep. `stop` ends processing
without canceling implicit keep. If no action cancels it, the message goes to INBOX.

The complete script parses and checks supported syntax before any action. Runtime
filing/limit errors are atomic: proposed actions are discarded, the sample is kept
in INBOX and the error remains explicit. An error cannot earn a passing result even
if that recovery mailbox happens to match a requirement. Syntax, unsupported syntax,
filing errors, limits and ordinary incorrect outcomes remain distinct.

The workspace model keeps practice and assignment drafts/results separately.
Run starts a new run; a tick evaluates at most one bounded sample, allowing a
native Stop between samples. Stop retains the draft and partial observations;
Run retries from the first sample. Edits clear current results/credit. Selected
reset affects only that scope. Saved results are checked against current source,
samples and draft. Completion additionally reevaluates actual behavior, never
trusting printed output or a saved success flag. Actual native editor/Run/Stop, failure detail, retry, separate drafts, reset and
normal two-process partial-work reopening have been checked on Fedora. These are
development checks. The installed470candidate also passes the corresponding native
and normal two-process partial-work reopening checks; no independent novice acceptance.

## Explicit bounds and remaining gaps

Source16KiB;4096tokens;1024bytes/string;32entries/list;16block/test nesting levels;
32headers/message;1024bytes/header value;64KiB/canonical message;32actions and
2,097,152comparison-work units and250ms elapsed time/message. Checks run at
statement/test boundaries and each bounded wildcard row; a time-limit error is
reported explicitly rather than graded as an ordinary wrong answer. Workspace datasets allow at most12samples,
256KiBtotal sample bytes and16supplied mailboxes; independent tasks require at
least4cases and worked practice3different samples. No Sieve loops, recursion or
external programs exist. These are simulator policy bounds, not real server limits.

Address-aware matching, envelope tests, redirects, reject/ereject, vacation,
procmail, variables, MIME decoding, external filters and all other extensions are
unsupported. Action options such as `fileinto :copy` fail explicitly. These remain
curriculum/fidelity gaps; header matching does not establish sender authentication.
No reference implementation is installed; focused specification-derived tests are
not an external Sieve execution comparison or independent review.

Sources checked2026-11-14:

- [LPI LPIC-2v4.5,201-450/202-450,211.2](https://www.lpi.org/our-certifications/exam-201-202-objectives/)
- [RFC5228](https://www.rfc-editor.org/rfc/rfc5228.html), particularly2.3–2.10,3–5.
- [RFC5429](https://www.rfc-editor.org/rfc/rfc5429.html) for the explicitly
  unimplemented reject/ereject boundary.

No source code is copied from those references. New implementation/authored prose
uses the owner-approved project MIT license; existing third-party notices remain.
