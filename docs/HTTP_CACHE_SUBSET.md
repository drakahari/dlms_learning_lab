## Packaged432R1 verified,2026-11-12

Installed192actions/53PNG, private430→432lifecycle and NFS two-process cold
reopen pass; originals inspected. Exact path, hashes and limits: CHECKPOINT.md.
Immutable package documents precede this post-build receipt; do not re-attest
against later source changes. Historical pending statements below describe the
prepackage development state, not a newly claimed platform/novice acceptance.

# Shared HTTP cache foundation — in development

Added2026-11-12 for the existing LPIC-2 4.5/208.3 roadmap. Current source has
432labs/167units and two individually reviewed header/cache lessons. Only narrow
generic-cache foundation credit applies to the208.3parent; Squid cache configuration
is not implemented. The verified430R1candidate predates this foundation. No new
dependencies, schema, Bash evaluator, original progress or existing graders change.

## Deliberately closed model

`src/http_cache.rs` is original, bounded in-process Rust. It replays supplied GET
requests against one virtual origin and a new private shared-cache instance per
trace. Paths, times, Date/Age, availability, strong opaque ETags and full bodies are
source-held facts. No real HTTP messages, sockets, listeners, network clocks, host
files, commands, subprocesses or actual Squid execute. Bodies are plain data.

The response Cache-Control value parser supports `max-age`, `s-maxage`, `public`,
unqualified `private`, `no-cache`, `no-store`, `must-revalidate` and
`proxy-revalidate`. Names are case-insensitive; durations accept decimal tokens
or quoted decimals. Request values support only `no-cache`, `no-store` and
`max-age`. Unknown directives, qualified flags, duplicates and empty list members
report Unsupported; malformed supported syntax reports Syntax. This diagnostic
boundary is smaller than valid HTTP syntax and extension handling. Never teach
an unsupported construct as universally invalid HTTP.

## Modeled decisions

Freshness accounts for supplied origin Date/Age, transit and residence. Shared
freshness uses s-maxage before max-age; exact expiry requires origin contact.
No-cache permits storage but requires successful validation for reuse. Private
and no-store responses cannot enter this shared cache. A matching strong ETag
produces304validation with refreshed metadata; a changed body/tag produces200.
Request no-store can use an already fresh entry but prevents storing new data.

Without explicit freshness there is no heuristic lifetime. Stale content is
never served offline, even where another implementation could permit it.
After origin contact this model evicts the old entry before optionally storing
the new metadata. That conservative eviction is a permitted implementation
choice, not a requirement imposed on every HTTP cache. It does not occur on a
valid hit. Response flag precedence remains restrictive: public cannot override
private or no-store. Returned bodies and actual storage are separate observations.

## Limits and admission

At most16configured paths,64requests,1024bytes per header value,128bytes per path,
80bytes per opaque tag and4096bytes per body. Times must be nondecreasing after
preceding supplied responses, with at most one hour of supplied transit. Durations
and age arithmetic saturate at2^31seconds. One strong tag cannot identify different
bodies at the same path. Dot segments, query strings and unknown paths are refused.
Validation precedes replay; reports cannot deserialize as grading authority and
internal cached entries have no import route. Multiple traces never share a cache.

No private-browser execution, auth/cookies, Vary, ranges, non-GET/unsafe methods,
other origin statuses, weak tags, Last-Modified, Expires, heuristic freshness,
stale-if-error/max-stale, distributed cache, eviction/resource tuning, Squid cache
directives or full HTTP/Squid interoperability is claimed. These need additional
capabilities and source review before dependent assignments. Headers alone do not
complete the broader proxy administration objective.

## Sources and checks

Checked2026-11-12:

- [LPI LPIC-2 official objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/): current4.5/201-450/202-450,208.3.
- [RFC9111](https://www.rfc-editor.org/rfc/rfc9111.html): shared storage, freshness/age, validation and Cache-Control, sections3–5.2. No RFC code or text bundle copied.
- [Squid3.5.19 cache directive](https://www.squid-cache.org/Versions/v3/3.5/cfgman/cache.html) and [object-size directive](https://www.squid-cache.org/Versions/v3/3.5/cfgman/maximum_object_size.html): references for still-unimplemented configuration work, not behavior this library implements. Direct lookups initially failed; following official3.5links succeeded. Current/static mirrors are not silently substituted for historical3.5behavior.

Core/binding31, authored5/retainedproxy5 and widget/capture10checks pass. Actual
development R1/R2each192actions/53PNG and individual lesson render review are
recorded in HTTP_CACHE_LESSONS.md. Final Python1179/all-targetClippy/normal4/
retainedproxyUI11/coverage/inventory pass; separate432package acceptance is pending.
No independent novice, screen-reader, physical-device or other-OS acceptance.
Original implementation/content follows the owner's MIT decision; upstream
references are linked, not redistributed as original project material.
