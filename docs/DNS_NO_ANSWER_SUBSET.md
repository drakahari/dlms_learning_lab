# Selected-copy authoritative absence

Source contract, 2026-10-24. This adds an original typed grading predicate to
the existing offline authoritative-DNS model. It adds no lesson, State variant,
dependency, host resolver, learner command, packet or coverage claim.

`Requirement::NoAnswer { copy, name, record_type }` requires a syntactically
supported name inside the configured origin and a Primary or Secondary copy.
Configured is rejected: saved records do not establish a served answer. Existing
Server validation still runs before checking an outcome; mismatched origins,
invalid serving records and unsupported names fail honestly.

The check invokes the selected Zone's existing bounded `query` exactly once.
It passes only when the answer is authoritative, its code is NOERROR or
NXDOMAIN, and no returned record has the requested type. It rejects query errors,
REFUSED and SERVFAIL (including alias loops and walks beyond the existing
16-alias limit). Neither a missing name nor a present name without that type is
preferred: describe the task as returning no requested data, rather than promise
one particular response code.

Every returned CNAME target is canonicalized and checked against that same
origin. If any target leaves the authority, absence is unproven and the check
fails. In this model an external alias may produce authoritative NOERROR with
only CNAME records; that incomplete resolution is not negative evidence. An
internal alias ending in an in-zone missing name or missing type can pass; an
alias reaching the requested data fails. External PTR targets are ordinary
returned data, not walked names, and are not rejected as external aliases.
No logic parses the human-readable `Answer.note`.

This is current evidence about one selected serving copy, not synchronization
or event history. An unsaved/saved edit does not alter an old loaded answer.
Separate negative rows can disagree until publication. A handover assignment
must independently require `Published { secondary: true }`, both selected-copy
negative rows and positive served outcomes. Existing goal validation is
unchanged: at most 20 outcomes, with at least one positive `Resolves` and one
`Published`. Absence-only goals remain invalid. Equivalent normalized names and
legitimate publication orders remain acceptable.

Wire shape is the existing internally tagged Requirement object, with one new
`kind: "NoAnswer"` variant and no progress-schema change. Names use the existing
ASCII label/254-text-byte limit. Query record/alias/work limits and existing
pack/profile read ceilings are unchanged; this is not a new streaming decoder.
The new check returns one short feedback row (under 512 bytes even at the maximum
name length), without dumping expected names, full zone data or file contents.
The native secondary-control predicate recognizes a Secondary NoAnswer row just
as it recognizes existing selected-copy outcomes; no new control is added.

Prospective checks cover NXDOMAIN/NODATA, internal/external aliases, PTR targets,
loops/depth failure, malformed authority/context, stale saved/primary/secondary
copies, positive/publication guards, tagged serialization, exact malformed-draft
resume and origin pins, bounded feedback and secondary-control visibility.
All11 model checks pass in the combined941-check MAIN gate, and the secondary
visibility UI check passes in the full180 run. There are no authored reverse-DNS
labs registered yet; this is capability acceptance, not curriculum credit.

Semantics use the existing bounded model, informed by
[RFC1035 query responses](https://www.rfc-editor.org/rfc/rfc1035.html),
[RFC2308 negative responses](https://www.rfc-editor.org/rfc/rfc2308.html) and
[RFC2181 alias/record distinctions](https://www.rfc-editor.org/rfc/rfc2181.html).
This predicate does not implement negative caching, recursive resolution,
delegation, full BIND behavior or real DNS transport.
