# Fixed RPM VERSION sorting reference

2026-10-26. The original bare-label collector passed208 Python workflow tests, including23 new
fake-process groups, in both its isolated checkout and MAIN. The first actual
Fedora capture completed40 sequential observations under the default development
sandbox. **It does not establish RPM version conformance:**12 strict observations
matched,16 did not, and12 tied controls remain inconclusive. The bare labels were interpreted as package names, so those observations test
name sorting rather than version comparison. They must not change the core or
be reported as reference agreement. The pure RPM core's23 model checks are separate.
This tool does not provide an application command, package transaction, dependency
resolver, learner-script host runner, curriculum lab or objective coverage.

The corrected reference is deliberately small:20 reviewed VERSION-component pairs,
each supplied in both literal input orders as `dlms-oracle-VERSION-1`. The fixed
NAME and RELEASE are identical across each comparison. The only program is the reviewed
`/usr/bin/rpmsort` ELF. Its retained nofollow file descriptor is hashed before and
after execution and used as the executable path. There are **no options or file
arguments**, including no `--version`, `--reverse`, `--config` or `--`. Nothing
invokes a shell, package manager, macro evaluation, RPM database or package script.
Both the original VERSION fixture and the new, separately hash-pinned NVR fixture
remain empty of observations. The collector validates every original pair, expectation
and meaning against the unchanged original fixture before accepting its framed copy.
No input can be supplied by a
learner or caller; its full bytes must match the reviewed digest.

The official [rpmsort(1)](https://rpm.org/docs/6.0.x/man/rpmsort.1) and
[rpm-version(7)](https://rpm.org/docs/6.0.x/man/rpm-version.7), checked2026-10-26,
support the documented expectations. Manual URLs/date and the pinned reviewed
contract are provenance; no claim of a retained full manual byte copy is made.
The15 original full-EVR vectors remain unchanged. Their release-text projections
are only VERSION inputs here; epoch ordering and full EVR composition are excluded.
The5 extras were separately reviewed. Only VERSION varies inside the fixed
NVR labels; there is no epoch/architecture/full-EVR inference. An observed sorted pair cannot prove
comparator equality: tied controls always leave equality and strict compatibility
null, regardless of input/output order. A strict mismatch is retained as an actual
observation and requires scope review; it never rewrites the manual expectation
or adjusts the model automatically. COMPLETE means all observations were captured,
not that the model matches or the cases prove full RPM conformance.

## Input-domain diagnosis and source-only correction

The retained first capture supplied bare VERSION strings. The matching official
[6.0.2 tool wrapper](https://github.com/rpm-software-management/rpm/blob/rpm-6.0.2-release/tools/rpmsort.cc),
checked2026-10-26, splits package labels using the last two hyphens (lines48–82).
It first compares the remaining NAME as plain text; only equal names reach its
VERSION/RELEASE comparator calls (lines85–115). A no-hyphen label therefore stays
entirely in NAME. The observed `10` before `2`, and a release before its `~rc`
text, agree with that name comparison. They are not RPM VERSION-comparator
counterexamples. This is a collector input-domain mistake, not evidence to change
our independent model or expected orders. The matching5280-byte upstream wrapper
retains SHA-256 `86bca909466243a90423cb4f7cd8f14ceb1915d99f47a6341cef945fb31d1786`.
Exact downstream build correspondence remains qualified as in the startup audit.
No GPL comparator implementation was read or copied into the model.

The new `rpmsort-version-nvr-inputs.json` preserves the original20 VERSION pairs
and their authored expected orders and adds fixed `dlms-oracle-` / `-1` framing.
This puts the varying labels in the wrapper's VERSION position while NAME and
RELEASE remain identical. The collector accepts no caller-selected name, release,
fixture or pattern. It preserves raw full-NVR stdin/stdout and checks full-NVR
multisets/orders derived only by framing the independent original expectations.
The258-byte stdin limit, argv, environment and all process/evidence limits remain
unchanged. This correction and its25 fake-process guard groups passed as part of211 MAIN
Python workflow checks on2026-10-26. The corrected actual capture passed its scoped strict-order checks; only a
new capture ID after source review may supply observations. Old fixtures, old
COMPLETE/raw evidence, the15 full-EVR model vectors and the pure model stay exact.

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

## Workflow and captured evidence

After independent source review and integration into MAIN, the guarded commands are:

```sh
python3 tools/dev.py python-tests
python3 tools/dev.py tool rpm_version_oracle.py --capture-id version-via-nvr-r2
```

The first command discovers `tools/tests/test_rpm_version_oracle.py` using only fake
processes. It never runs the reference program. The second command requires a new ID and the fixed trusted-tool conditions above.
It accepts no caller-provided fixture or command. A new actual corrected capture
requires the reviewed fixed-tool conditions; the corrected actual capture is recorded below. The211-check MAIN fake/workflow
gate passed before that launch. Tests cover closed input/CLI admission, lock/MAIN/environment,
actual aggregate growth, exact ELF FD and output protocols, inconclusive ties,
fixed NAME/RELEASE framing and unchanged original expected pairs,40-child full
fake capture, no-clobber/partial retention, source drift, whole
deadline before launch and publication, fake timeout/held-pipe cleanup, and actual
stream/aggregate byte limits. No real utility subprocess, compiler, VM or model
was started while authoring these files.


The retained first capture is `version-components-m126-r1`, source `a194ff8`.
Its54,624-byte manifest SHA-256 is
`e32213a32c505ac39db7d8943f495b59e016c80b1f610df29ee5c5a824d86c00`.
All retained output hashes and the no-clobber completion marker were independently
read back. `COMPLETE` records collection success, while
`strict_cases_compatible=false` records disagreement. None of the20 expected
pairs, original15 full-EVR model vectors or observed outputs was rewritten.
The runner exited with no active/unfinished-child marker; its one memory sample
reported23.48GiB available, zero memory pressure and zero OOM kills. This short
collection is not a stress test or a guarantee for other workloads.


## Why bare VERSION inputs were the wrong reference domain

Review of the matching official
[rpmsort wrapper at rpm-6.0.2-release](https://github.com/rpm-software-management/rpm/blob/rpm-6.0.2-release/tools/rpmsort.cc)
on2026-10-26 explains the mismatch. The wrapper splits package labels using the
last two hyphens and sorts package names before their version and release parts.
A label with no hyphens remains a package name. For example, the captured `10`
before `2` is lexicographic name ordering, not evidence against numeric version
comparison. The earlier startup audit deliberately did not inspect that callback;
the initial component-only contract therefore overstated the usable input domain.
The mistake and all raw observations are retained.

A corrected, separately pinned fixture will surround each unchanged version with
the same package name and release (`dlms-oracle-VERSION-1`). That keeps name and
release constant so differing versions reach the version comparison branch. It
requires fresh fake-process gates and a new capture identity; the original capture
will never be overwritten or relabeled as a conformant comparison. Equality,
complete epoch/version/release behavior and actual package operations remain
outside the proposed sorting evidence. No upstream implementation is copied.


## Corrected fixed-NVR observation

The separately identified `version-via-nvr-m126-r2` capture ran from committed
source `78d5e1b9f48f67cd41b5b81e16a5b09dceccc81c`, using the default development
sandbox and unchanged execution limits. All40 observations completed:28 strict
orders agree with their unchanged documented expectations;12 tie controls remain
inconclusive. There were no strict-order mismatches. The59,881-byte manifest hash
is `375537c30568f0104dc90202469563d62e2b5d8be0330f27ebd21e7f4139b280`.
Its completion marker and all162 retained output hashes were independently read
back. The earlier bare-label capture remains intact. This supports only these
VERSION ordering examples through identical-name/release framing. It does not
prove equality, complete EVR composition, package operations or full RPM fidelity.
