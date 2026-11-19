## Packaged434R1verified,2026-11-12

Installed236actions/52PNG, private432→434lifecycle and NFS two-process cold
reopening pass; actual originals inspected. Exact path/hashes/limits: CHECKPOINT.md.
The immutable package predates this post-build acceptance record; historical pending
statements below describe development, not a new platform or novice acceptance.

# Bounded reverse-proxy routing subset

Implemented foundation2026-11-12 against the existing LPIC-2 4.5/208.4 roadmap. Two registered source-bound configuration assignments now use this capability.
Only narrow208.4parent/K1/K2credit applies; the full objective remains incomplete. Current source434/168extends the preserved432/167catalog. Verified432R1 remains
immutable and available; its source freeze ended before this additive work.

The pure model evaluates one Linux-style Nginx server fragment, intended for an http
context. It is not a complete nginx.conf. The deliberately closed subset supports
one explicit decimal listen port (no privilege-dependent defaults), optional single literal server_name, nonnested prefix
and exact locations, and one literal return or IPv4 HTTP proxy_pass per location.
Whitespace/comments and simple quoted literal tokens are parsed, never executed.
Exact locations win over prefixes; otherwise the longest prefix wins. A proxy
URL's URI replaces the matched location; a URL without a URI retains the request
path. The automatic trailing-slash redirect is modeled only with unambiguous
prefix siblings; unsupported ambiguity is diagnosed instead of guessed.

Virtual IPv4 endpoints are routing targets only, not real or simulated responding
backend servers. Only explicit return statuses and slash-redirect destinations
are calculated; no backend response bodies or502failures are invented. No DNS,
TLS, sockets, actual HTTP bytes, web/filesystem
root, body parsing, command execution, real Nginx, clock or service manipulation.
Request paths are already normalized ASCII absolute paths. Percent encoding,
query strings, repeated slashes and dot segments are unsupported, not evidence
that real Nginx universally rejects them. No variables, regex, named/nested
locations, rewrite, include, upstream groups, load balancing, authentication,
headers, caching, timeout or installation/config-reload behavior is claimed.
Return bodies are optional literal text; default error-page bytes are outside
the model. Case-sensitive matching follows the declared Linux exercise semantics,
regardless of which host OS eventually runs this simulator.

Bound config bytes, tokens, locations, path/body sizes and fixture resources before
evaluation. Invalid and unsupported constructs have different honest diagnostics.
No loop, recursive include or runtime allocation from learner code exists. Only
bounded virtual records evolve; no learner string is passed to a subprocess or
host file/network operation.

Sources checked2026-11-12: LPI current4.5/201-450/202-450 objective208.4,
https://www.lpi.org/our-certifications/exam-201-202-objectives/ ; official Nginx
https://nginx.org/en/docs/http/ngx_http_core_module.html#location and
https://nginx.org/en/docs/http/ngx_http_proxy_module.html#proxy_pass . These
references establish exact/prefix selection and URI replacement/preservation.
They are not imported executable code or a claim of full version interoperability.
Original code/content retains the owner's MIT decision; no upstream code copied.


## Focused development evidence

Core15tests pass: real lexical parsing, exact/longest-prefix selection, URI
replacement/preservation, automatic slash redirects, limits, unsupported syntax,
atomic saved/applied virtual state and whole supported-path grading partitions.
Two first-run failures are retained: bracketed IPv6 now reports Unsupported before
attempting decimal-port parsing; a proposed equivalent extra slash location really
introduced a redirect. Its negative assertion remains, and the valid alternative
includes the needed exact bare-path override. The first compile attempt also had a
JSON fixture boolean type error, corrected without weakening its rejection test.
Core/workspace/binding28, authored5/retainedcache5 and widget/capture11pass.
Actual R1/R2each236actions/52PNG; individual lesson review and limitations are in
REVERSE_PROXY_LESSONS.md. Final434package acceptance remains pending. No actual
Nginx, wire-protocol, package, novice or accessibility conformance follows.


The parser requires an explicit listener and a / fallback; it does not guess
privilege-dependent default ports or implicit filesystem behavior. Supported
return statuses are200,403,404 and503 with optional literal bodies. Simple single/
double quoted literals and comments are supported; interpolation, escapes and
concatenated quoted tokens are not. At most8192config bytes,1024tokens,32locations,
128bytes per normalized client path and256bytes per token. Forwarded paths retain
literal concatenation (up to256bytes), rather than silently normalizing it.
Full path comparison is bounded by both configurations’ finite prefix tries. A
missing required error body is unconstrained, not an assertion of empty wire bytes.
No parsed tree/report imports as truth; saved/applied raw virtual text is reparsed.
