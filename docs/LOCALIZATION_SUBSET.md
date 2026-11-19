# Locale and text-encoding foundation

Two registered LPIC-1 lessons now supply narrow partial107.3 practice in the
normal417-lab/160-unit catalog. Their individual teaching/content and native
workflows are reviewed; see LOCALIZATION_LESSONS.md. Separate417R1 packaging/content/private lifecycle and224 actual installed actions
pass; NFS cold reopening also passes. Exact build-time inputs remain retained.
Timezone coverage is still open; these lessons do not complete107.3.

## Supplied behavior

The model distinguishes LANG defaults, six POSIX category overrides and nonempty
LC_ALL precedence. Empty overrides fall back to the category, then LANG, then C.
The supplied inventory supports C/POSIX, C.UTF-8, en_US.UTF-8 and de_DE.UTF-8;
known utf8 aliases normalize to the same typed values. No host locale is read.

The saved /etc/locale.conf editor uses bounded literal assignments, simple quoted
values, blank lines and comments. LC_ALL is forbidden in that systemd file.
Duplicate settings and other shell/config features are explicitly outside this
simulator subset. A saved edit does not change an already running session.
Opening a fresh simulated login reads only these saved defaults. It is not a
child terminal inheriting the old process environment; real user/session-manager
and startup-file overrides are not modeled here.

The scoped terminal parses and evaluates real supported tool forms:

- locale: a normalized six-category listing; locale -a; locale [-k] charmap or
  decimal_point. The supplied available list is virtual, not the host inventory.
- export NAME=value and unset NAME, limited to known locale bindings. Unlike a
  general shell, unknown variable names or unavailable locale values are refused.
- iconv -f FROM -t TO [-o OUTPUT] INPUT, with attached short values and the
  corresponding long forms. Supported codecs are strict ASCII, ISO-8859-1 and
  Unicode UTF-8. No stdin, multi-input conversion, duplicate flags, transliteration,
  discard-invalid flags, arbitrary codecs or gconv modules.
- ls [/work] and cat FILE inspect only supplied/created virtual byte documents.
  No other directory or host path is accessed. Pipes, redirection, expansion,
  scripts and unlisted commands produce an honest unsupported diagnostic.

This is separate from the existing Bash interpreter; its reserved LANG/LC_*
boundaries and evaluator revision are unchanged.

## Byte files and errors

One to eight supplied inputs are read-only, at most16KiB each/64KiB total.
Eight output files,32KiB each/128KiB total, may exist beneath a simple /work/name
namespace. Directory components, traversal and host paths are refused. Supplied
input immutability is an activity boundary, not a complete Unix permission model.
Viewing converted output alone does not save a file.

An executed conversion can return failure while writing a valid prefix. The
model retains that partial file and grades its actual bytes. Retry replaces the
output; it does not alter the original. Unknown syntax/options and simulator
resource refusals are different from an executed conversion failure.
UTF-8 characters remain intact in valid text displays; invalid/nonprinting bytes
use an explicit escaped view. This display never converts the saved document.

The original sample reporting application reads UTF-8 item;cents documents with
unquoted labels and whole cents. It formats numbers using LC_NUMERIC, chooses its
own English/German headings using LC_MESSAGES, and encodes output using LC_CTYPE.
It is not a Linux utility, gettext implementation or general CSV parser.
Up to16rows and96UTF-8 bytes per label are supported. Timezone/date, actual
collation, regex/case rules and general localized utilities remain unimplemented.
Mentioning a category does not provide practical coverage of those behaviors.

## Outcomes and persistence

Tasks compare multiple actual current-session/new-login reports or saved output
files, with distinct deterministic inputs. Equivalent effective category settings
and supported command forms can pass. A wrong current session cannot hide behind
correct future defaults, nor can a merely valid UTF-8 file hide changed text.
Partial output, missing files and unsaved settings receive separate feedback.

Practice and assignment have separate config/command drafts and histories.
Restoration replays at most96typed actions against their declared source; serialized
worlds, outputs and grades are not trusted. Edits/state changes invalidate current
success where applicable. Future workspace versions and foreign input/task IDs
are refused. App progress/catalog integration and a synthetic disk round trip pass; no learner's
original progress or installed application is used by the development fixtures.

## Evidence and primary references

Core/task/session/progress bindings45, authored-content5 and UI/capture7 checks
pass. NativeR3 completed224 actual widget actions with separate drafts and
configuration/byte-file feedback. All415 prior scenario values and159 units are
retained; all415 legacy virgin attempts retain their exact JSON values in a
synthetic disk round trip. Separate417R1 packaged checks now pass; see LOCALIZATION_LESSONS.md. The fixed developer-only referenceR1 passed28comparisons:
16iconv conversions,12locale queries, with no unavailable-locale skips. Inputs and
executables were pinned; all fixtures and results stayed on the project disk under
MAIN runner's shared lock. No learner program or system-setting operation ran.

R1 records an earlier selected-source checkpoint. Preserve its CONTROL/report;
later additive modules do not extend that evidence. Rebuild the fixed probe and
use a new evidence directory for a later current comparison. Rebuilt current
referenceR2 has now repeated28/28 passing comparisons with no skips. This is not a
historical evaluator seal, full compiler closure or independent acceptance.

Checked2026-11-11:

- [LPI current LPIC-1 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/):
  version5.0,101-500/102-500;107.3 now has narrow partial locale/encoding practice,
  while timezone settings and the unlisted utility behavior remain open.
- [Linux man-pages locale(7)](https://man7.org/linux/man-pages/man7/locale.7.html):
  category roles and environment precedence.
- [systemd locale.conf](https://www.freedesktop.org/software/systemd/man/latest/locale.conf.html):
  literal saved defaults, category settings and the LC_ALL exclusion.
- [Linux man-pages iconv(1)](https://man7.org/linux/man-pages/man1/iconv.1.html):
  explicit source/target encodings and output/error behavior.

GNU libc manual fetches and initial localectl/locale1 API fetches were unavailable;
no unspecified-key localectl behavior has been guessed. No upstream implementation
was incorporated. No actual systemd service/file-setting comparison, timezone,
independent novice, screen-reader or other-OS acceptance is claimed. Native
technical review is separate from those acceptance criteria.
