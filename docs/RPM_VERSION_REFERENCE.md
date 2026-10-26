# Fixed RPM VERSION sorting reference

2026-10-26. Source-only developer collector; all23 new Python groups and actual
reference collection are **unrun** at this handoff. The pure RPM core passed its
separate23 model checks and scoped Clippy; those checks are not real RPM results.
This tool does not provide an application command, package transaction, dependency
resolver, learner-script host runner, curriculum lab or objective coverage.

The first reference is deliberately small:20 reviewed VERSION-component pairs,
each supplied in both literal input orders. The only program is the reviewed
`/usr/bin/rpmsort` ELF. Its retained nofollow file descriptor is hashed before and
after execution and used as the executable path. There are **no options or file
arguments**, including no `--version`, `--reverse`, `--config` or `--`. Nothing
invokes a shell, package manager, macro evaluation, RPM database or package script.
The source fixture remains empty of observations. No input can be supplied by a
learner or caller; its full bytes must match the reviewed digest.

The official [rpmsort(1)](https://rpm.org/docs/6.0.x/man/rpmsort.1) and
[rpm-version(7)](https://rpm.org/docs/6.0.x/man/rpm-version.7), checked2026-10-26,
support the documented expectations. Manual URLs/date and the pinned reviewed
contract are provenance; no claim of a retained full manual byte copy is made.
The15 original full-EVR vectors remain unchanged. Their release-text projections
are only VERSION inputs here; epoch ordering and full EVR composition are excluded.
The5 extras were separately reviewed. An observed sorted pair cannot prove
comparator equality: tied controls always leave equality and strict compatibility
null, regardless of input/output order. A strict mismatch is retained as an actual
observation and requires scope review; it never rewrites the manual expectation
or adjusts the model automatically. COMPLETE means all observations were captured,
not that the model matches or the cases prove full RPM conformance.

## Trusted executable and qualified startup evidence

The reviewed15832-byte ELF has SHA-256
`54bc254cc76ebe6340bf26e841fe0ad14f243419e2b1f4061a992d2eac0c19aa`.
Its embedded package note identifies rpm6.0.2-1.fc44/x86_64 and GNU build-id
`c556addbd0c3965e0dda0648b924a619f6438896`. This is read-only ELF metadata,
not an executed version probe or proof of build authenticity. The matching official
upstream6.0.2 startup/stdio wrapper was reviewed without using its comparison
implementation. No GPL implementation code was copied. Root accepted this exact
ELF as a trusted development tool with explicitly qualified evidence. Downstream
patch correspondence and all linked-library constructors/transitive effects were
not proven; neither a clean HOME nor symbol absence is an absolute no-network or
no-configuration-read proof. System loader/libraries remain trusted host components.

The private reviewed contract and startup audit hashes are required source pins.
They remain under `.build-artifacts/reviews/rpm-package-foundation/`; ordinary
source checkouts without those approved artifacts cannot run this local collector.
An ELF or provenance change fails closed rather than automatically repinning.

## Guarded limits and retained evidence

Only MAIN's `tools/dev.py tool rpm_version_oracle.py --capture-id ID` can launch
the capture. It requires the actual inherited shared-lock FD, owned process group,
active-command marker,2-job/2-thread environment, MAIN default target and verified
disk temporary path. Worktrees, arbitrary executable/source/output paths, unknown
flags and duplicate IDs/options are refused. Keep the actual invocation in the
**default execution sandbox**. If it denies access or execution, stop and report;
do not request escalation or bypass the sandbox for this reference.

There are at most40 sequential children,2seconds each, with a30-second deadline
checked before launches and completion. Bounded cleanup and synchronous filesystem
work may extend elapsed time. Each launch rechecks a positive remaining
budget after setup and uses its minimum with2seconds. Every child receives258B
or less stdin and no inherited caller environment. Its fresh0700 disk HOME,
TMPDIR/TMP/TEMP and XDG config/cache/data directories live below the owned capture;
PATH is empty and locale C. Unexpected private-directory writes fail completion.
No file argument, personal profile, macro/config variable, proxy or loader override
is copied. The existing bounded nonblocking capture helper streams4KiB per output
stream, charging actual bytes into one512KiB aggregate **including metadata, stdin
and manifest**, not only stdout/stderr. Held pipes, timeouts and output pressure
abort collection and reap its own child; the MAIN guard cleans only its owned
process group. Quiescence is required before another child and publication.

Source/test/fixture/helper/core-vector and exact executable pins are captured
before and after. Both stat preflight and actual hashed pin bytes are checked
against per-file/8MiB aggregate limits. Raw stdin/stdout/stderr, status and
observations remain on disk. Every retained output is independently repinned and
its actual aggregate checked before publication. New IDs use no-clobber0700
directories/0600 files. Failures retain partial evidence without COMPLETE.
The final marker links a synced prepared hash only after all40 observations,
unchanged pins, exact outputs, quiescence and final deadline checks. A later
directory-sync failure is reported as a committed durability warning, never
misreported as a retryable uncommitted capture. No old evidence is deleted.

## Prospective gates

After independent source review and integration into MAIN, root may run:

```sh
python3 tools/dev.py python-tests
python3 tools/dev.py tool rpm_version_oracle.py --capture-id version-components-r1
```

The first command discovers `tools/tests/test_rpm_version_oracle.py` using only fake
processes. It never runs the reference program. The second command requires an
explicit later actual-reference grant and a new ID; it is not authorized by this
source-only handoff. Tests cover closed input/CLI admission, lock/MAIN/environment,
actual aggregate growth, exact ELF FD and output protocols, inconclusive ties,
40-child full fake capture, no-clobber/partial retention, source drift, whole
deadline before launch and publication, fake timeout/held-pipe cleanup, and actual
stream/aggregate byte limits. No real utility subprocess, compiler, VM or model
was started while authoring these files.
