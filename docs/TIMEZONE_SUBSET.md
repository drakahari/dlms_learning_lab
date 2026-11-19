# Time-zone display foundation

Two source-registered LPIC-1 lessons address narrow 107.3 and overlapping108.1
regional display/configuration skills. Source419/161 adds a typed goal, validated
source-bound progress and native terminal workspace; all417 previous scenarios and
160units remain exact. Core/task/session/bindings24, authored-content5 and UI/capture7
checks pass. Actual development252-action runs expose and correct native defects;
final candidate/lifecycle evidence is recorded separately in TIMEZONE_LESSONS.md.
The separate419R1 review candidate now passes packaged content/private lifecycle,
installed native252actions and NFS cold reopening. Exact selected inputs are retained;
its source freeze has ended. See CHECKPOINT.md for observed identities/limits.

## Deliberate bounds

The original in-process model converts **2026 UTC instants** into supplied UTC,
Europe/Berlin, America/New_York and Asia/Kolkata display zones. It uses explicit
2026 transitions derived from IANA tzdb2026f facts, not guessed rules for every
place/year. Gregorian calendar helpers cover1970–2100 internally; public zone
conversion rejects instants outside2026. No leap-second representation, arbitrary
TZ strings, TZif loading, future rule extrapolation or host zone inventory.

The virtual UTC clock is immutable. Changing the configured local zone or process
TZ changes its representation, never the represented instant. An unset TZ follows
the supplied system zone. An empty TZ meansUTC; this deliberately differs from
locale-category fallback. Unsupported zone strings produce an honest subset
error rather than silently imitating libc's fallback for arbitrary invalid TZ.

Date output uses the supplied C-locale names and a bounded subset of real date
format fields. Default date output has the conventional weekday/month/day/time/
zone/year shape. Explicit formats support Y,m,d,e,H,M,S,F,T,R,z,:z,Z,s,a,b, literal
percent, newline and tab. Other modifiers/specifiers are refused, including ones
that a full GNU date may support or print literally. The formatter is not general
strftime, locale-sensitive formatting or general date-string parsing.

The finite literal command adapter supports date with -u/--utc/--universal,
-d/--date @SECONDS (including the documented attached/equals forms), and one
+FORMAT. All explicit instants must fall in2026. Duplicate options outside the
stated subset are diagnosed. Only supplied time zones can be selected through
simulated timedatectl set-timezone; show -p Timezone [--value] and list-timezones
inspect the modeled property/inventory. A complete daemon status is not faked.
Literal export TZ=value, unset TZ, printenv TZ and readlink /etc/localtime inspect
or change only these typed virtual values. printenv returns1 for an unset TZ.
No host command, clock, file, link, service, process environment or network target
is used or changed by learner input. This adapter does not extend Bash semantics.

One ASCII literal command is limited to1,024bytes/24words; formats256bytes and
output2KiB. Operators, pipelines, redirection, expansion, scripts, shell execution,
clock setting, arbitrary paths and other commands are unsupported. There is no
learner loop or host fallback. The configured localtime link points into the
supplied virtual zoneinfo names. No /etc/timezone write behavior, manual ln,
interactive tzselect or systemd daemon lifecycle has been guessed or credited.

## Evidence and primary references

Core10 checks cover month/leap/year boundaries, transition seconds, repeated local
times, half-hour offsets, TZ precedence, parsed date forms, immutable UTC and
unsupported/malicious inputs. Fixed referenceR1 passed30 actual date comparisons under MAIN's shared lock, with
stripped environment and disposable project-disk output. It pins ten selected
compiled inputs, helpers, executables and fixed timezone files; not a full compiler
closure or independent seal. Host clock values are not fixture inputs; no syscall-
level tracing of the utility's internal clock reads was performed. No actual systemd
configuration/daemon operation was compared.
It will never become the learner execution engine. Passing core tests is not
native lesson, novice, AT, physical-device or other-OS acceptance.

Checked2026-11-11:

- [IANA tzdb2026f Europe](https://data.iana.org/time-zones/tzdb-2026f/europe),
  [North America](https://data.iana.org/time-zones/tzdb-2026f/northamerica),
  [Asia](https://data.iana.org/time-zones/tzdb-2026f/asia): supplied zone/transition
  facts. These data sources identify themselves as public domain. This project
  uses an original implementation; no upstream tzcode or systemd code is copied.
- [tzset(3)](https://man7.org/linux/man-pages/man3/tzset.3.html): unset/empty TZ
  distinctions and configuration lookup.
- [localtime(5)](https://man7.org/linux/man-pages/man5/localtime.5.html): the
  zoneinfo-linked configuration convention.
- [Official systemd v258 timedated source](https://raw.githubusercontent.com/systemd/systemd/v258/src/timedate/timedated.c):
  configured-zone handling. Rendered timedatectl manual fetches were unavailable;
  no actual systemd operation or /etc/timezone synchronization is claimed.
- [GNU coreutils v9.7 manual source](https://raw.githubusercontent.com/coreutils/coreutils/v9.7/doc/coreutils.texi):
  supported date format/option meanings and extension distinctions.
