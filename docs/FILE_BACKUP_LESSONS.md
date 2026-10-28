# Backup recovery unit — authored draft, not registered

Four distinct LPIC-2 assignments have tested simulation, teaching, grading and
progress backends. They are **not in the normal catalog yet**: native workspace,
complete-input presentation, course navigation and installed acceptance remain.
No objective is promoted. See FILE_BACKUP_SUBSET.md for exact GNU-style flags,
virtual administrator role, abstract archive format and exclusions.

| Assignment | Meaningful work | What plausible wrong work misses |
| --- | --- | --- |
| Recover one file without losing newer work | Select a literal archive member, inspect text and stat metadata, preserve the newer draft | Full extraction overwrites the newer working file |
| Recover the complete backup chain | Apply three dependent levels in order and use directory inventories for deletion | The newest level omits middle-level changes; overlay retains retired files |
| Branch a backup without rewriting its history | Copy the older snapshot, create an incremental branch, preserve originals and verify recovery | A newer snapshot omits a needed earlier change; a full archive is not the requested branch |
| Deliver a recovery that survives independent verification | Investigate two structurally valid candidates with identical member names but different contents, preserve evidence and deliver a working chain | Names and an earlier correct-looking restore cannot prove the delivered archive's data |

The first assignment follows the existing Linux file-handover foundation. Each
later task names the preceding backup prerequisite. Introductions explain each
new flag before use, offer different runnable example files and an adverse
change/repair, and retain the complete independent assignment. stat's format
letters, literal member paths, GNU-specific incremental options and snapshot
updates are explained explicitly. Four distinct worked explanations remain
available on request/review rather than being unsolicited Challenge solutions.

Grading inspects actual names, text, modes and owners, payload/root/type,
snapshot state and retained history. Recovery requirements additionally restore
the delivered levels onto separate empty/stale sample inputs. Logical times are
not silently called physical timestamp preservation. Source and practice paths
are disjoint; the persisted practice retains its complete initial definition.
Same-path work cannot be relabeled as a different example. Running it cannot
award assignment credit.

M179:68 focused Rust checks plus3 native-memo refusal regressions pass, including
references, long-option alternatives, wrong/partial/broken work, example changes
and repairs, Step0 completion, draft isolation, selected reset, tempfile save/
fresh deserialize and exact synthetic original154 preservation. This is **not**
a process-cold native or installed-app test. All-target Clippy and254Python pass;
full composed inclusive/native/installer gates remain. Actual old evaluation
history remains covered separately by the required inclusive retained-history
suite, not replaced by the synthetic compatibility fixture.

No tar binary parsing, compression, links/devices, tape, remote rsync, actual
filesystem traversal, timestamps or privileged operations run. The broader
206.2 knowledge/tool requirements remain explicitly Open in the objective map.
