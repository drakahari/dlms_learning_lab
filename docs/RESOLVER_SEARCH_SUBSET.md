# Resolver-search subset — source unit pending native review

Existing roadmap109.4 search-order gap, after local files/DNS source selection.
First establish a pure search/ndots model; do not author dependent assignments
before focused capability tests pass. Preserve current250 labs and all progress.
No new dependency or shell evaluator change.

Model only the search/domain/options-ndots excerpt of a virtual resolv.conf and
positive IPv4 A or definite missing-name replies from a supplied DNS snapshot.
Source order and nameserver routing are separate. No OS hostname/resolver, actual
files, packets, retries/timeouts, CNAME/IPv6/DNSSEC/cache, HOSTALIASES, environment
variables, alternative resolver flags or systemd-resolved. Unsupported directives,
query grammar or unmodeled unavailable/retry response sequencing refuse explicitly.

Bound16KiB/128lines, at most three search entries and a four-candidate plan. Include
literal root suffix, last search/domain directive, default ndots1, explicit0–15,
ASCII case-insensitive ordinary labels and absolute trailing-dot queries. Supplied
hostname determines the default domain; no host OS value is read. Try a name as-is
first at the ndots threshold, search suffixes after definite misses, then as-is if
not previously tried and no root suffix. An absolute query never gets a suffix.
Stop at the first positive answer. Preserve repeated candidate attempts where
configured; never silently deduplicate or truncate names. Overlong expanded names
produce an honest safety diagnostic instead of a fake DNS result.

References read2026-11-02:
- https://man7.org/linux/man-pages/man5/resolv.conf.5.html (upstream Linux man-pages).
- GNU manual index retrieved as2.44; it is documentation, not installed acceptance:
  https://sourceware.org/glibc/manual/latest/html_node/index.html
- Actual source material inspected through
  https://codebrowser.dev/glibc/glibc/resolv/res_query.c.html ; footer identifies
  glibc-2.39-288-gce65d944e3, generated2026-02-08, NOT current2.44 source proof.
  https://raw.githubusercontent.com/bminor/glibc/master/resolv/res_query.c also read;
  master is unpinned supporting inspection, not versioned runtime conformance.
- Sourceware raw2.44 URL was inaccessible; mirrored2.44 tag cache miss and2.43 tag
  404. These were not silently treated as verified current-version code.

Use original implementation and paraphrased teaching; do not import/copy upstream
code. Pure source review does not establish real glibc behavior on this Fedora
machine. Keep unavailable conformance and broader DNS objectives explicit.

## Source implementation gates

The9pure groups, initial scoped Clippy and126Python pass. Sources/Search task
mode then reuses client_names_workspace without changing old serialized default
goals or initial sessions. Exact mode-specific file sets and supplied-hostname
bounds precede evaluation. Six scope/format/cancel/resume groups plus retained
source groups pass (37combined); scoped workspace Clippy passes. New original
source252/71 adds two suffix-order/ndots-absolute assignments; five content/
persistence groups pass including exact250predecessor labs/70units. Eight new
UI/native-phase groups pass. Actual native/content review remains pending: these
are not installed, OS resolver, screen-reader, physical or novice acceptances.
