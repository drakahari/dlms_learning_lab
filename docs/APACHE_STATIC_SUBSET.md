# Apache static-serving foundation — proposed pure subset

Source drafted 2026-10-25 against Apache HTTP Server **2.4** documentation,
RFC9110/9112 (June2022), and current LPI LPIC-2 v4.5 Topic208. The prospective
tests are authored model vectors, not observations from a real Apache server.
This foundation has not been compiled or executed. There is no app State/Goal,
editor, command adapter, bundled lesson, progress field or coverage promotion.

## Purpose and closed configuration

An independent in-process model separates saved raw Apache-style configuration,
an active source snapshot, and live contents of fixed virtual files. It has no
host filesystem, listener, HTTP connection, process, subprocess or dependencies.
The existing Service and UnitManager models are unchanged. In particular,
UnitManager's fictional PortDaemon does not prove static HTTP serving.

The parser accepts one config document with one `Listen PORT`, an explicit
global root baseline, and1–8 ordered `<VirtualHost *:PORT>` sections. All vhosts
use the Listen port. IP-specific/mixed/wildcard ports, multiple Listen directives
and main-server response fallback are explicitly Unsupported.

```apache
Listen 8080
<Directory "/">
    Options None
    AllowOverride None
    Require all denied
</Directory>
<VirtualHost *:8080>
    ServerName gallery.example.test
    ServerAlias art.example.test
    DocumentRoot "/srv/gallery"
    DirectoryIndex welcome.html index.html
    <Directory "/srv/gallery">
        Require all granted
    </Directory>
    <Directory "/srv/gallery/private">
        Require all denied
    </Directory>
</VirtualHost>
```

Directive/container names are ASCII case-insensitive. Arguments are literal
bare tokens or whole double-quoted tokens; whitespace separates adjacent
arguments. Blank lines and whole-line # comments are allowed. There are no
escapes, single quotes, inline comments, continuation, expansion, Includes or
unlisted directives. These are model restrictions; real Apache supports more.

Each vhost needs one literal DNS ServerName and canonical absolute DocumentRoot;
optional one ServerAlias list has up to8 literal names. Duplicate names across
vhosts are allowed and first-match shadowing remains visible. One DirectoryIndex
line has1–4 simple basenames or sole case-insensitive `disabled`; omission means
`index.html`. A same-named file never overrides that keyword. Mixed basename/
disabled lists (any keyword case) are explicitly Unsupported in this closure.
Repeated scalar/DirectoryIndex lines are Unsupported. Actual Apache appends
repeated DirectoryIndex directives; this parser does not replace them silently.

The global root baseline requires exactly `Options None`, `AllowOverride None`
and one `Require all granted|denied`. Each vhost has up to4 literal Directory
sections, each one Require rule. Parent authorization is replaced by the most
specific matching child rule under default AuthMerging Off. Matching uses actual
directory ancestry, not arbitrary string prefixes or the final filename. Same-
path duplicate sections, regex/wildcards, Files/Location, authentication,
htaccess, scripts, symlinks and OS permissions are excluded. DirectoryIndex
basenames share the enclosing directory's rule under this grammar; no broader
denied-index fallback/subrequest behavior is claimed.

## Saved, active and file layers

SaveConfig preserves bounded malformed raw work. CheckConfig parses without
mutation or credit. Start when stopped parses and activates the saved source;
Start when already running changes nothing. Stop removes the active snapshot.
Running-only logical Reload parses first and replaces the active snapshot only
on success. A rejected action preserves the exact State. Bad saved configuration
does not erase a healthy active service. No `apachectl`/`systemctl` alias, graceful
worker lifecycle, socket-binding or readiness claim is made.

Only exact active source is persisted. The parsed configuration is derived again
for validation and requests, rather than stored as another import authority.
Static file contents are live: writing a supplied page changes later requests
without configuration reload. Inventory fixes fixture/server/config-slot and
all directory/file identities/kinds; only file contents can change. All fixed
file texts are required. Foreign files, kind changes, missing parents and foreign
Inventory context are rejected. Canonical nonexistent DocumentRoot values remain
parseable so configuration mistakes are not validated away. A DocumentRoot that
names a regular file is explicitly outside this subset at request time.

## Typed requests and results

A virtual request contains server identity, destination port, GET/HEAD, up to2
Host values and a canonical origin-form path. No listener yields typed NoListener,
not an HTTP denial. Among the one-port vhosts, name/alias matching is case-
insensitive in source order; no name match selects the first vhost. Host's optional
numeric port is ignored for selection; actual destination port is authoritative.

Only verified malformed Host syntax produces400 (missing/duplicate fields,
embedded whitespace/controls, delimiters forbidden in Host, nondecimal port).
Valid or unclassified excluded reg-name forms, including underscore, percent,
trailing dot, IP literals and other non-DNS names, return Unsupported. This is
not a general HTTP parser. Canonical paths exclude dot segments, percent escapes,
query/fragment, duplicate slash, backslash and non-ASCII. Duplicate slashes are
checked in the original spelling before a final directory slash is removed;
GET/HEAD `//` cannot collapse into `/`. Excluded paths are not
normalized and no traversal-security claim is made.

An authorized file gives200/exact UTF-8 content. An authorized missing path gives
404. A denied existing resource gives403; denied nonexistent paths are Unsupported
until that precedence is independently pinned. An existing authorized directory
without slash gives301 plus a typed target path, not serialized Location/authority
headers. With slash, index basenames are tried in order. A directory index candidate
that is itself a directory is Unsupported. No index/disabled index yields403
because listing is absent. An extra final slash on a regular file gives404.
HEAD has the corresponding selection/status and no body; file representation
length is its UTF-8 **byte** length. No invented Date/Server/MIME, error HTML,
cache/range/compression, HTTP/2, TLS, proxy or DNS behavior is present.

All observations are derived from current active config/current file contents.
Responses cannot be decoded back as proof. Later task grading must include
positive healthy-host/file outcomes alongside denials, preserve named outcomes
rather than full Runtime equality, and use a fresh start from saved source when
next-start behavior matters. LoadedCurrent equality is coherence, not proof a
reload action occurred. No cached inspection or operation-history credential.

## Resource and import contract

- Config16KiB;256 physical lines;512 bytes per line (CRLF ending excluded).
-8 vhosts,64 aliases total,32 Directory sections including global root.
- Paths160 ASCII bytes/depth8; hostname253, Host field260, request path256 bytes.
-128 fixed tree entries,96 files;4KiB per file/256KiB aggregate file text.
- Full serialized pure State2MiB; raw Action128KiB; raw Request8KiB.
- Fixed diagnostic prose512 UTF-8 bytes in a fresh allocation; no user text dump.

Borrowed scalar/count/aggregate checks precede derived structures and copies.
String/map/sequence visitors reject bounds and duplicate keys while reading;
public decode methods also cap whole raw input. State decode revalidates active
source, file shape and immutable context. Direct construction must validate too;
all observation/mutation entrypoints do so. No source truncation or automatic
repair occurs. Configuration failure is distinct from unsupported syntax or
resource limits. The later app retains existing2MiB pack/16MB profile ceilings;
not every pure maximum will fit an app document. No new app admission is added here.

## Prospective gate and objective limits

The20 integration groups cover independent first/default/name/alias/source-order
vectors, verified400 versus excluded Host forms, actual-port behavior, index and
redirect behavior, child authorization/sibling/file-name boundaries, non-ASCII
HEAD bytes/query purity, saved-active/live-file separation, atomic reload failure,
missing-root repair, strict decode/context, maximum escaped compact/pretty State,
count/physical-line bounds and positive-service safeguards. They remain **UNRUN**.

A first guarded gate, only after explicit root transfer, would run the isolated
`--test apache_static`, then all-target Clippy and formatting through the documented
MAIN workflow. No actual Apache/reference invocation is part of this proposal.
Future adapters/content/native acceptance are separate reviewed milestones.

This models a limited portion of208.1 virtual-host/file-access reasoning. Install,
module management/scripting, authentication, logging/performance/resource controls,
real service operation and full208.1 remain unproved.208.2 HTTPS,208.3 Squid and
208.4 Nginx remain separate Open capabilities. No platform/runtime/security claim.

## Primary source references (checked2026-10-25)

- [LPI v4.5 overview](https://www.lpi.org/our-certifications/lpic-2-overview/)
- [LPI Topic208 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/)
- [Apache2.4 configuration syntax](https://httpd.apache.org/docs/2.4/en/configuring.html)
- [Apache2.4 vhost matching](https://httpd.apache.org/docs/2.4/en/vhosts/details.html)
- [DocumentRoot/Directory](https://httpd.apache.org/docs/2.4/en/mod/core.html)
- [Configuration section order](https://httpd.apache.org/docs/2.4/en/sections.html)
- [Require/AuthMerging](https://httpd.apache.org/docs/2.4/en/mod/mod_authz_core.html)
- [DirectoryIndex/DirectorySlash](https://httpd.apache.org/docs/2.4/en/mod/mod_dir.html)
- [Restart syntax-check limits](https://httpd.apache.org/docs/2.4/en/stopping.html)
- [RFC9112 section3.2 Host](https://www.rfc-editor.org/rfc/rfc9112.html#section-3.2)
- [RFC9110 section9.3.2 HEAD](https://www.rfc-editor.org/rfc/rfc9110.html#section-9.3.2)

## MAIN first guarded execution — 2026-10-25

The independently reviewed frozen source was integrated incrementally over the
current package/DHCP application. No existing family hook was removed. The first
guarded command passes68 library,20 pure STP,18 STP adapter and20 pure Apache
checks. All-target Clippy with warnings denied also passes on the first run.
Historical source-only/UNRUN statements above describe the original handoff, not
the current execution status. Broader progress regression and native/content
acceptance are separate gates. No real protocol/server/package process was run.
The catalog and objective coverage are unchanged.
