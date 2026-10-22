# Progress, backups and recovery

Your assignments, example drafts, favorites and navigation views are local. There is no account, cloud synchronization or telemetry. Settings → Your data manages your own files; simulator paths such as `/work/report` never access this feature or the host filesystem.

## Export

Choose **Settings → Your data → Export backup…**, select a folder and an unused filename, then Save backup. Documents is the initial location; you do not need to know an application profile path. The native chooser lists folders/files and accepts a filename in the selected folder. Existing files and reserved application filenames cannot be replaced. Copy the resulting JSON backup to your own safe storage. It contains your scripts and other work: do not publish it in the project repository.

The backup contains progress, both assignment/example drafts, case results and favorites. Appearance is independent and stays unchanged when you restore. Backups use a versioned envelope; old raw version-1 progress files are also accepted for recovery.

## Restore

1. Close any other copy of Learning Lab. Choose **Restore backup…** and select a backup.
2. The app checks the format, identifiers, versions, bounded state and drafts. A background worker re-runs saved script/example results inside the bounded simulator. Cancel stops checking without writing anything. Forged or stale result text cannot earn script completion merely by being present in JSON.
3. Review the saved-lab/completion/favorite summary. Nothing is replaced until **Restore checked backup** is selected. Cancel preserves the current session.
4. Before replacement, current writable session edits are saved. An exact copy of the existing progress file goes to `progress-before-restore-N.json` beside it, using an unused filename. If the app opened corrupt progress read-only and you made session edits, those edits are also exported to `session-before-restore-N.json` first. If protecting work fails, restore stops.
5. The already checked in-memory backup is atomically installed. Changing the selected file after the preview cannot change what is restored. The app returns to the library and displays the safety-copy path.

A write can be committed while final directory durability confirmation fails. The app reports that specific warning instead of falsely claiming that no replacement happened. Keep an additional backup when such a storage warning occurs. The controlled failure test covers reporting; real power-loss/filesystem durability has not been hardware tested.

## Old work and newly added requirements

Version-1 progress stays readable. Existing scenario IDs/versions are retained. Previously completed four-answer IPv4 analysis remains visible as earlier credit; a new workstation requirement is never filled in automatically. Values an older version supplied are retained for review, and an explicit check is needed before current completion. Reset affects only its selected attempt. Older binaries may not understand later optional fields; do not downgrade your only progress copy.

## Corrupt, future or oversized files

Invalid/future imports are rejected without replacing current work. Invalid normal progress opens read-only so the original remains recoverable. Symbolic links, devices and other special files are not accepted. The current progress size ceiling is 16 MB; the backup envelope allows only small additional metadata.

If the *current* progress file exceeds 16 MB, automatic restore refuses to replace it because a bounded exact safety copy cannot be made. Recovery steps:

1. Close all app instances.
2. Use your OS file manager to copy the oversized file to a safe location, then move the original out of the displayed local-progress location. Keep that copy; do not delete it.
3. Reopen the app and restore a known valid backup through Settings. The app will not truncate an oversized file on your behalf.

A corrupted file with no known backup may need manual review; do not import edited data into your only copy. The application reports unknown versions rather than silently dropping unfamiliar labs or fields.

## Multiple instances and platform status

The app takes an OS writer lock **before reading** progress. A second instance opens read-only, including preferences. Close/reopen it after the owning instance exits to load the newest data. The OS releases the lock on exit/crash; the small lock file is deliberately retained rather than deleted, avoiding inode replacement races. On storage that cannot safely lock, the app refuses writable operation. This protects cooperating app instances, not arbitrary external file editors or adversarial host filesystem races.

Fedora backend/UI/native evidence is recorded in IMPLEMENTATION_LEDGER.md. The standard-library lock and atomic-write architecture is portable, but Windows/macOS runtime behavior still needs actual platform acceptance.

## Shell evaluator upgrades

Saved script runs now identify the evaluator revision that produced them. Earlier records without that field remain readable and preserve their original source, output, file snapshots and earned credit. They are labeled **Earlier run complete · Rerun to verify**, separately from current success. Opening the assignment moves the older result to optional saved history without changing the draft. Run verifies the current draft; a failure or unsupported construct cannot reuse earlier success. A successful current Run restores current completion, while earlier history remains available. Reset explicitly clears that selected attempt, including history. Successive upgrades retain earlier runs together, including a later earned result alongside an older incomplete run. Optional Earlier saved runs shows their scripts, output and simulated files. History is bounded to32 records per attempt and the existing16MB profile limit; a full history needing another migration is rejected with the original retained, rather than silently discarding work.

Before the first upgraded write, the exact older progress bytes are copied once to `progress-v1.before-shell-vN.json` (N is the new evaluator revision; currently5) beside the main file; existing backups are never replaced. Reads alone do not rewrite progress. Save size/bounds and atomic-write protections still apply. Export/restore retains earlier history and rechecks active saved runs with the current evaluator before approval; example output has separate revision labeling and never awards assignment credit. Future incompatible record revisions are rejected with the original file retained. Downgrade compatibility after saving new metadata is not claimed.


Evaluator4 tightens shared text-expansion bounds and cumulative replacement work. A source that earned credit under3 can now reach a resource limit, even without printing large output. Its exact earlier source/results/credit remain historical; a current Run is required for current completion. Existing earlier histories stay alongside it, and the first upgraded write preserves the exact prior profile in the non-overwriting v4 backup. Reset clears only the selected attempt/history; it cannot remove that original-byte safety copy. See SHELL_EXPANSION_LIMITS.md.


Evaluator5 adds bounded * and ? filename matching and corrects unquoted variable-supplied bracket expressions that earlier ran literally. A clean revision4 probe confirmed that earlier result before migration tests were authored. Unsupported bracket patterns now ask for the supported subset or literal quoting; prior results/credit remain history, with the first upgraded write protected by the non-overwriting v5 backup. New syntax never earns prior learners unrecorded answers. See SHELL_GLOBBING.md.


The IPv4 routing integration adds only optional version1 workspace fields, omitted from earlier attempts. Assignment and ungraded practice each retain their own topology and incomplete raw configuration text. A pending different draft clears current completion even if the previously applied topology still passes; Check configuration applies all valid edits atomically. Invalid imported device/interface/cable structure or routing metadata attached to another activity is rejected before restoration. Existing original subnet, IPv6, terminal and script attempts keep their IDs/versions and data.
