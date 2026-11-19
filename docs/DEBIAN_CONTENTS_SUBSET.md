# Repository file lookup: bounded apt-file subset

Original in-process implementation,2026-11-02. No host utility, network, archive
extraction, package process or filesystem is reached by learner input. Reuses
`debian_packages::State` and the existing literal package terminal. No dependency,
shell evaluator revision, State/Goal variant or progress-schema field is added.

## Model and supported spellings

A supplied publication contains coherent package and Contents metadata. The
Contents rows derive from that snapshot's declared regular-file manifests,
independently of installed records, locally cached archives and current files.
The existing explicit metadata refresh selects the supplied publication; it does
not install software. This finite model cannot represent absent or independently
stale Contents targets, or claim every real repository offers them.

- `apt-file search PATTERN`: case-sensitive literal substring in a file path.
- `apt-file -F search PATH`: literal complete path. `--fixed-string` and ordinary
  option placement are supported; `find` is the search alias.
- `apt-file list PACKAGE`: one exact package name; `show` is the list alias.
  Fixed-string mode is the default for this package listing.

One initial slash is accepted for file searches. Complete-path search removes
that slash; ordinary substring search retains its path-component boundary:
`/usr/bin/tool` matches the root-relative path or an internal `/usr/bin/tool`,
not `xusr/bin/tool`. Printed rows are `package: path` without an initial slash.
Native-architecture and architecture-independent metadata are included. There
are no directories in these finite manifests. Rows are deduplicated and ordered.

Status0 means a matching result;1 means a valid search with no result;3 means no
cached index. Unsupported input reports2. Empty-cache diagnostics remain in the
saved transcript for retry; the status3 admission is limited to a correctly
parsed Contents command. It does not admit arbitrary dpkg status3 entries.
Transcript history never grants completion or supplies trusted expected state.

Patterns are bounded to160ordinary ASCII bytes; commands retain the existing
2048-byte and literal-tokenizer limits. Controls, traversal components, patterns,
regex/globs, selectors/filters, source/config/architecture options, binary archive
search and the apt-file update wrapper refuse explicitly. Use the already taught
`apt-get --error-on=any update` for this sample's coherent metadata refresh.
Results above32KiB fail before publishing output. Existing catalog bounds admit
at most256manifest rows,24names and32files per archive; no unbounded search loop.

## References and limits

Authoritative sources checked2026-11-02:

- [LPI objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/):
  version5.0,101-500/102-500, narrow102.4file-provider practice only.
- [apt-file3.3 manual](https://manpages.debian.org/trixie/apt-file/apt-file.1.en.html).
- [Versioned apt-file3.3 source](https://sources.debian.org/src/apt-file/3.3/apt-file/):
  inspected literal versus complete matching, leading-slash and list behavior;
  original Rust implementation, no source imported/copied.
- [APT3.0.3 cache manual](https://manpages.debian.org/trixie/apt/apt-cache.8.en.html):
  cached metadata can be older and is distinct from installed software.

No actual apt-file/dpkg comparison is claimed on Fedora. Multiple sources,
independent Contents acquisition, archive extraction, signatures, trust, full
APT policy/dependency solver, source-list editing and dpkg-reconfigure stay Open.
The existing package installer remains its disclosed script-free final-state
model; this extension does not make it a full package manager.

## New-only outcome predicate and terminal behavior

The new task uses `InstalledFileProvider { path }`: a positive ordinary-file
outcome over actual installed records and intact ordinary payloads. It accepts
any equivalent supplied provider, rather than naming a hidden answer package.
Copied/unowned bytes, a configuration-only record, damaged ordinary files and
conffile-only providers fail. The path must name an ordinary supplied manifest
entry; preservation/negative-only definitions still refuse. Every old requirement
branch and serialized original goal remains unchanged. This is a new task’s
state-based grading capability, not a silent regrade of existing lessons.

An explicit submission now asks the existing Debian terminal to show its latest
output once. Per-mode transient UI state does not change progress or grading;
manual scrollback remains available afterward. The focused actual-widget test
covers scrollback, another Run and the new result at the terminal end. This is
functional output visibility in the reused tool, not a workspace redesign.
