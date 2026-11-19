# Coverage ledger: explicit catalog profiles

The preserved normal catalog and the current teaching-review candidate are
separate registries. `bundled` remains the historical normal catalog (201 labs).
Use the explicit current profile for continuation-ledger bookkeeping:

```sh
python3 tools/dev.py tool verify_coverage.py --catalog teaching-review
```

The default stays `--catalog bundled`; it must refuse evidence about lessons
outside that catalog. This is expected when current continuation rows are selected.
It never falls back from a malformed requested profile to a different catalog.

The tool follows only the named public entry and its finite literal helper chain.
Unknown/dynamic/conditional helpers, cycles, duplicate or unsafe paths fail closed.
It never executes Rust, scans all includes for credit, chooses the highest snapshot
number or counts an unregistered file as a lesson. Static unit-index documents
are validated and listed separately for reporting; their `labs` lists award no
coverage. Files, total bytes, JSON schemas and chain depth remain bounded.

## Verified 2026-11-05

Seven new fixture groups cover explicit profiles, unreachable drafts, conditional/
dynamic/malformed sources, cycles/depth/path limits, inert comments/quoted/nested
headers, unit-only reporting and invalid unit metadata. All earlier tests remain.
The Python suite passes 735 groups (12.170 s). That is separate from the actual
current-repository invocation, which passes after three scoped CSV repairs:

- 102.4 parent evidence now includes the three file-provider/source lessons already
  named in the planning parent. No child skill is automatically promoted.
- 104.2 parent records the two implemented offline graph-repair lessons and their
  tests; the parent planning vocabulary is corrected to `Partial`.
- 109.4 parent records the five implemented IPv4 name-source/search lessons and
  their actual scope. Real resolver/utility behavior remains explicitly excluded.

Actual current result: 281 registered labs, no unregistered lesson drafts, 1,240
source references: 197 partial-practice, three supporting-explanation, 1,040 open.
References overlap and many partial rows cover only a narrow exercise. These
numbers are **not a completion percentage**, exam preparation guarantee, or an
independent evaluation of instruction. Other continuation slices remain detailed
in CONTINUATION_COVERAGE.csv; no implicit credit fills their remaining child rows.

The first actual run failed on the pre-existing 102.4 parent/leaf mismatch; the
failed log and the successful second run are retained privately. Historical
source fingerprints and sealed shell proof identities are unchanged. Older
Markdown inventories explicitly labeled 201 remain historical records, not the
current candidate count. The current catalog is the actual compiled authority;
metadata/parser fixtures alone cannot establish runtime lab availability.

After the FHS milestone, the actual current profile passes283 registered lessons:
202partial,3supporting,1035open across the same1240overlapping references. Only
104.7parent/K1–K3/find gain scoped evidence; broader tools remain open. Counts are
bookkeeping, never educational completion percentages.
