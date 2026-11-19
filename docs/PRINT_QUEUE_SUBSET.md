# Print-queue simulator subset

Three source-registered lessons passed development and installed415R1 native review. This page defines
behavior, not certification completion or real-printer acceptance.

## State and behavior

Each activity supplies named printers, immutable documents under `/work`, an
operator identity, local default and existing requests. Accepting new jobs and
enabling processing are independent controls. Each job retains its ID, printer,
document, copy count and Pending, Held, Completed or Canceled state. Enabled
printers immediately finish pending jobs in this finite model. A held job stays
held until explicitly released. Changing the default affects new submissions;
it never relocates an existing request. Completed mistakes cannot be undone by
canceling their history.

The outcome grader checks configuration, preserved jobs and other-printer
settings, exact additional deliveries, and absence of unintended completed or
unfinished work. Several independent sample requests exercise the repaired state
without adding jobs to the assignment. Two one-copy requests and one two-copy
request are equivalent where their total delivery meets the task. Replacing an
existing held request does not satisfy a requirement to finish that original job.

## Supported tools

- `lpstat`: default active jobs, `-t`, `-d`, `-p [PRINTER]`, `-a [PRINTER]`,
  `-o [PRINTER]`, `-l`, and `-W completed|not-completed` before `-o`.
- `lp [-d PRINTER] [-n COPIES] [-H hold|immediate] FILE`;
  `lp -i REQUEST -H hold|resume`.
- `lpr [-P PRINTER] [-# COPIES] [-q] FILE`; attached `-Pname` and `-#2`
  forms are supported. Ordinary successful `lpr` submission is silent.
- `lpq [-l] [-P PRINTER]`; `lprm NUMBER` for exactly one explicit numeric job.
- `cancel REQUEST`, `cupsaccept PRINTER`, `cupsreject [-r REASON] PRINTER`,
  `cupsenable PRINTER`, `cupsdisable [-r REASON] PRINTER`, `lpoptions -d PRINTER`.
- `ls [/work]` and `cat FILE` for supplied documents only.

Literal quoted arguments and escapes are parsed. No shell substitutions,
operators, scripts or external commands run. Unsupported forms fail explicitly.
All printer names in the model are case-insensitive; command names are not.
`lpq -l` reports documents/copies. `lpstat -l` reports status and destination.
Timestamps and device URIs are explicitly simulated. Spool byte counts round the
supplied text to KiB; they are not an actual filtered printer-ready stream.

## Bounds and persistence

At most eight printers/eight documents, 64 total jobs, 1–20 copies per request,
96 successful commands per draft, and 1,024 bytes/24 words per command. Documents
are bounded individually and in total. Practice and assignment histories, input
and last-run provenance are separate. Restoring progress replays commands against
the versioned scenario; serialized worlds, output or grades cannot grant credit.
A state-changing action invalidates the previous check. Merely editing an unrun
terminal command does not alter an already repaired simulated printer.

No actual files, printer, daemon, network, subprocess, device, credentials or
system settings are accessed. Printer/device setup, drivers, classes, remote
servers, actual LPD transport, authentication, environment/server-default
precedence, implicit/cancel-all `lprm`, recurring polling and unlisted flags remain
unsupported. BSD-compatible command names do not imply a working LPD protocol.

## Authoritative references

Checked 2026-11-11. [LPI's current LPIC-1 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/)
state version 5.0, exams 101-500/102-500. This unit addresses a narrow practical
part of 108.4; configuring physical/remote printers and `/etc/cups` is still open.

Command behavior was checked against OpenPrinting's official manuals for
[lp](https://openprinting.github.io/cups/doc/man-lp.html),
[lpstat](https://openprinting.github.io/cups/doc/man-lpstat.html),
[lpoptions](https://openprinting.github.io/cups/doc/man-lpoptions.html),
[cupsaccept](https://openprinting.github.io/cups/doc/man-cupsaccept.html),
[cupsenable](https://openprinting.github.io/cups/doc/man-cupsenable.html),
[lpr](https://openprinting.github.io/cups/doc/man-lpr.html),
[lpq](https://openprinting.github.io/cups/doc/man-lpq.html) and
[lprm](https://openprinting.github.io/cups/doc/man-lprm.html).
Output details were inspected in upstream CUPS v2.4.12
[lpstat.c](https://github.com/OpenPrinting/cups/blob/v2.4.12/systemv/lpstat.c) and
[lpq.c](https://github.com/OpenPrinting/cups/blob/v2.4.12/berkeley/lpq.c).
No upstream code is incorporated and no actual CUPS process comparison has been
performed. This is an original bounded model, not a full CUPS emulator.
