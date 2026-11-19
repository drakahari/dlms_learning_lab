# Service names in router rules — registered capstone

The490catalog now includes the source-bound editor/router recovery capstone.
Development native/cold review passed; final490package validation is pending.
See [lesson guide](LINUX_SERVICES_ROUTER_LESSON.md). The foundation record below
is historical; its no-authored-task status describes that earlier stage only.

# Virtual service names in router rules — foundation

This original in-process adapter is implemented and tested, but no new authored
router-file task is registered yet. Catalog489/195 and objective coverage remain
unchanged. Existing numeric labs do not opt in; their supported command lists,
grades and saved state remain unchanged. The verified489package predates this
foundation; do not attest it against later MAIN.

`linux_named_ports::v4/v6` accepts the existing literal bounded filter command
subset and resolves a single source/destination port name or alias against supplied
virtual services text. The explicit TCP/6 or UDP/17 protocol selects the lookup;
case matters, and a port operand does not contain a /tcp suffix. The two numeric
protocol aliases normalize within this adapter only. Numeric port0..65535 matches
are retained; this is packet filtering, not a claim that port0 is a usable client
service. Numeric-only commands do not consult the name database.

Each resolved rule stores a number. Editing the supplied mapping file later cannot
retroactively change that rule. Creation/replacement must occur again to use the
new mapping. Listing shows actual numeric rules; the adapter separately reports
the names, protocols and numbers it resolved. Lookup is not service startup,
neighbor discovery, delivery or application success.

The existing `service_names::Database` bounds/strict validation apply:32KiBfile,
512byte lines,64records,8aliases, simple case-sensitive names, TCP/UDP only.
Duplicate name/alias or port/protocol entries are explicitly unsupported instead
of guessing real provider precedence. The literal command lexer is reused:
2KiB/64arguments, quoted/escaped literals, no shell expansion/operators. Each
option consumes its operand; a quoted flag-like interface stays a literal value.
The existing numeric state table performs final validation and scratch-copy commit.
Any lookup, syntax, target, bound or validation failure retains the complete table.
96rule and32KiBlisting bounds remain. No includes, host filesystem, NSS, subprocess,
network, root privileges, kernel or external service participates.

Eight focused checks pass: protocol-specific aliases/numeric protocols, two-port
stateful insertion, fixed old rules after file changes, numeric operation without
a valid database, literal flag-like interfaces, malformed/unsupported/expansion
and limits, actual header outcomes after rule replacement, and atomic failure of
a second lookup or final target. The first run exposed the old IPv4 parser's lack
of numeric protocol aliases; the opt-in normalization fixed it without weakening
tests or changing old parser behavior.393current-only source guards and focused
library Clippy pass; unadmitted-source failure is retained. All1368other selected
old source/data inputs are exact against the verified489snapshot; only lib.rs
exports this one new helper. No inclusive suite or kernel/NSS runtime comparison.

Primary references checked2026-11-15UTC(local2026-11-14):
- [iptables extensions1.8.13](https://man7.org/linux/man-pages/man8/iptables-extensions.8.html): TCP/UDP port-name/number options; ranges/inversion remain unsupported here.
- [services5,Linuxman-pages6.19](https://man7.org/linux/man-pages/man5/services.5.html): protocol-specific names/aliases and lookup versus running services. Real NSS/invalid-line handling is broader than this strict supplied-file subset.
- [Official historical iptables HOWTO](https://www.iptables.org/documentation/HOWTO/packet-filtering-HOWTO-7.html): corroborates name lookup, not current kernel acceptance.

The numeric-at-creation model follows those documented interfaces; tests compare
our typed behavior, not an actual kernel installation. No upstream implementation
was copied. Original project MIT license and existing third-party notices remain.
Next: meaningful editor plus router-rule recovery, separate practice, exact whole-
range policy/traffic grading and progress/native integration before curriculum credit.
