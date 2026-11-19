# Local maintenance notices — supported simulation subset

Source406 registers two individually reviewed lessons. Their development-native
checks pass; installed406 acceptance is pending. Latest admitted package remains
404R2. See MAINTENANCE_NOTICE_LESSONS.md and CHECKPOINT.md for current evidence.

## Implemented model

Supplied session facts have a username, unique terminal and recorded login time.
A user can have several terminals. A recorded UTC calendar date and finite minute
counter replace the host clock. All provided terminals are writable by the
simulated administrator; actual ownership, tty permissions and PAM are not used.

Saving `/etc/motd` affects the message available after a future successful login;
it does not notify a current session. Saving `/etc/issue` changes pre-login
identification. Broadcast delivery records the actual supplied terminal set and
message separately from normal command output. These are virtual lookup keys,
never host paths. No text or script can create another file or access personal data.

One future restart can be scheduled, cancelled or reached by explicitly advancing
the simulated clock. Its pending time and the ordinary-account five-minute login
gate are modeled. Reaching the event stops this recorded-host activity; it never
restarts the computer. Recurring countdown broadcasts, actual login execution,
root/PAM exceptions and reboot/startup operation are outside scope. Replacement of
an existing schedule requires cancellation rather than guessing host-version
replacement behavior. This restriction is disclosed, not claimed universal.

## Literal terminal adapter

| Input | Modeled behavior |
| --- | --- |
| `who` | supplied users, terminals and recorded login dates/times |
| `date +%H:%M` | current simulated hour/minute, in recording UTC |
| `cat /etc/issue` or `cat /etc/motd` | saved virtual notice; no other files |
| `wall` | reads the visible composer as simulated standard input; no stdout on success |
| `shutdown -r +MINUTES [message]` | schedules a future virtual restart and records its announcement |
| `shutdown -c [message]` | cancels the pending virtual restart and records its cancellation announcement |

A native receipt view must label observations as simulator evidence rather than
inventing wall stdout. The message composer is explicit input, not a hidden
clipboard or executable script. Real wall supports additional operands/options;
this adapter rejects those rather than interpreting a user path as a real file.
Advance simulated clock and login previews are simulator activities, not commands.

Only1–1440future minutes, finite literal quotes/escapes,1024command bytes and
24words are supported. Instant and absolute shutdown times, extra options,
systemctl, wall files/argv, expansion, redirection, pipelines, compound commands,
networking, SSH banners and getty substitutions are unsupported and diagnose
honestly. The parser reuses only the existing in-process literal lexer; no BIOS
command dispatcher or host tool runs. Empty/read-only commands change no state.

## Messages and behavior grading

An organisation-specific bulletin format uses Service, Starts, Ends
and Contact lines. It is **not Linux syntax**. This finite check accepts supported
case/spacing and line order variations; it does not interpret arbitrary prose.
Missing/duplicate/unknown fields, malformed or reversed same-day times and control
sequences fail clearly. A lesson must state this format before assessing it; do
not imply that Linux requires labelled notice files or that keywords prove meaning.

The source-bound workspace replays bounded actions to derive files, current live
announcements and schedule state. It checks every supplied terminal, the future
login notice, preserved pre-login bytes and the requested plan. Earlier correct
messages cannot hide a later wrong announcement; an expired notice or insufficient
remaining notice period invalidates current success. Unsaved files and unsent
composer edits block a fresh check. Separate practice never awards assignment
credit. Source methods must be reachable and starters must be meaningfully unsolved.

## Bounds, progress and current verification

Version1 facts/drafts use closed typed data. There are at most eight terminals,
64actions,4KiB per notice and a two-day clock within the existing2000–2099calendar
model. Unknown fields, source/task mismatches, forged sent flags, erased inherited
actions, path-like terminals and excessive text/history refuse. Drafts keep
separate editor text, applied files and replay history; outcomes are recomputed.
No existing scenario, global progress schema, theme or user profile is changed.

Core9, bulletin2, workspace10, bindings4, authored9 and UI/capture3 pass;
development native170 actions pass. Focused retention and restore checks pass.
Packaged406 acceptance remains pending. Later source changes require their focused
checks. No full-objective, independent novice, physical-device, screen-reader or
other-OS acceptance is inferred.

## Authoritative references, checked2026-11-11

- [LPI LPIC-2 v4.5.0,206.3](https://www.lpi.org/our-certifications/exam-201-202-objectives/): logon messages and active-user maintenance communication; its complete terms remain separately mapped.
- [Linux man-pages, issue(5)](https://man7.org/linux/man-pages/man5/issue.5.html) and [motd(5)](https://man7.org/linux/man-pages/man5/motd.5.html): before-login identification versus the traditional post-login message. Actual display depends on configured login services.
- [util-linux wall source manual](https://github.com/util-linux/util-linux/blob/master/term-utils/wall.1.adoc): stdin/message/file delivery and terminal permissions; this model supplies writable terminals and a narrower stdin-only interface.
- [systemd shutdown source manual](https://github.com/systemd/systemd/blob/main/man/shutdown.xml): relative scheduling, cancellation and the ordinary-login gate. No actual shutdown/wall/systemctl reference execution is performed.
