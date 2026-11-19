## Combined Fedora candidate verified — 2026-11-13

This extension is now included in the separate 442-zone-query R1 candidate. Exact
launch, hashes and acceptance limits are in [CHECKPOINT.md](CHECKPOINT.md). Private
upgrade, both installed native query workflows, NFS cold reopening, final selected
readback and retention pass. Original app/progress untouched. This is not full
curriculum, real utility conformance, novice or other-platform acceptance. The
following source-stage records remain historical evidence.

# Published-zone lookup commands

The existing LPIC-2 dual-stack, alias-repair and primary/secondary lessons now
provide optional **Use lookup commands** beneath the record editor. The native
name/type controls remain the default and a valid alternative. No lab, grading
requirement, saved-state version or dependency is added. Old progress opens with
its original fields; command drafts are optional, bounded and separate for
practice and assignment. Filling a worked example changes only its editor and
preserves its inert command draft. Queries cannot repair or complete a task.

## Taught behavior and limits

`dig @SERVER NAME. TYPE +norecurse +short` and
`host -r -t TYPE NAME. SERVER` query the supplied **published** primary or secondary.
The disclosed parser also accepts `-t TYPE` for dig, leading `+short`, and `-4`
with host. It accepts A, AAAA, CNAME, MX, NS and PTR, explicit numeric IPv4 endpoints
and absolute ASCII names. Native controls and these commands read the same typed
`dns::Server`/`Zone.query`; saved but unpublished configuration is not an endpoint.

The exercises distinguish NXDOMAIN (missing name), NOERROR without the requested
record type, a CNAME link versus an A lookup through a looping alias, and distinct
published copies. A dig command can receive NXDOMAIN with utility status0; host
returns1 for DNS error responses and0 for NOERROR, including a missing type. These
statuses are observations, not assignment grades. Output labels the simulator's
DNS response, selected copy and utility status separately from answer text.

No live DNS, recursive resolver, cache, socket, timer, transport simulation,
DNSSEC, default multi-type lookup, implicit server/type, reverse-address shorthand,
TTL/header projection or arbitrary shell syntax is implemented. Unsupported input
fails explicitly. Alias traversal follows the existing bounded zone model, not
external delegations. Input is at most512ASCII bytes; complete output is bounded
to32768bytes and an oversized result fails instead of reporting truncated success.
Existing zone limits remain64records/16alias steps/131072serialized bytes. Only
source-provided private/documentation endpoint addresses are accepted.

## Content and preservation

Current `dns-forward-v2.json` retains all six original scenario values except
explicit source-owned lookup metadata and one reference on three lessons:
`lpic2-dns-dualstack`, `lpic2-dns-aliases`, `lpic2-dns-secondary`. All initial states,
IDs, versions, objectives, grading conditions and authored solutions are unchanged.
The old v1 pack and original presentation shard remain intact. A bounded overlay
adds lesson-specific instruction, separate worked values and actual-control
practice. No optional-command mastery is fabricated for historical completion.
Progress tests save a genuine old-format completed attempt and load it with the
new catalog, then save/restore separate new command drafts.

## References and evidence

Primary behavior reference checked2026-11-13:
[BIND9 stable manual pages](https://bind9.readthedocs.io/en/stable/manpages.html),
then identified as9.20.30. The current development9.21manual is separate. Official
ISC host.c was read for output/status interpretation; its unpinned main branch is
supporting source inspection, not a stable-version executable comparison. No
BIND executable or real network query was run. Existing DNS RFC sources remain.
No third-party implementation text/code was imported.

Focused final model/content/progress29 pass: adapter6, optional workspace4,
authored content4, retained DNS workspace8 and retained client/curriculum7. Content
checks include valid alternatives, partial work, protected-state breakage and a
real old-format progress save/load. Actual-widget4 and fixed native-plan1 pass,
including Enter, stale output, separate drafts, all five scales/two themes/two widths,
and a650logical-pixel-high output regression. Final development-native208actions/
27PNG pass. Viewed original lesson-specific introductions, worked table, assignments,
error/alias/copy observations and representative150%Light/200%Dark controls/output.
Source expected-result wording is clarified afterward; packaged rendering remains
pending. Native R10/R14 functional passes retain the insufficient output-height
captures; R17 reserves actual lines and shows the error/explanation. No test was
weakened or historical producer/seal changed. Inclusive Python1230, one-job all-target Clippy warnings-denied and strict
coverage/inventory pass. Combined candidate gates remain pending. Latest442direct R1package predates this
and the host extension. No full inclusive Rust runtime rerun, real BIND conformance,
physical keyboard/device, screen-reader, independent novice or other-OS acceptance.
