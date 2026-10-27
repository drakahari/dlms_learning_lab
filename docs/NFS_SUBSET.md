# Bounded NFS-style pure model

This module is a deterministic supplied-inventory simulation. It has no host
filesystem, mount, network, RPC, service, process or command interface. It does
not add a lesson, engine goal, progress codec, terminal, renderer or curriculum
coverage. The first unit remains future work.

The curriculum target used to scope this capability is LPIC-2 v4.5, detailed
objectives 4.5.0, Exam 202-450, Topic 209, objective 209.2 NFS Server
Configuration (weight 3). This small subset does not satisfy that entire
objective. Samba 209.1 is separate. The source contract and primary-document
review receipts remain private review evidence, outside authored course data.

## Observation and identity boundary

The observation is a **Simulated ACCESS-style server permission decision** for
an NFSv3/AUTH_SYS-style declared client and numeric identity, followed by a
bounded virtual operation on an existing regular text file. It is neither
trusted authentication nor a complete Linux client open or raw READ/WRITE RPC.
No real utility output, errno, numeric protocol status or shell exit code is
fabricated.

For a root-squashed matched export, UID 0 maps to anonuid; primary GID 0 and
each supplementary GID 0 independently map to anongid. Nonzero supplementary
groups survive even when UID 0 maps. Mapped groups are deduplicated; there is no
anonymous-account group lookup or manage-gids expansion. Request profiles and
inode ownership use numeric IDs, never their display labels. The profile range
0..65534 is a simulator bound, not Linux's maximum ID range. Anonymous IDs must
be nonzero in that range.

The server uses the effective owner class, else matching primary or
supplementary group class, else other. Classes never union or fall through.
Search requires directory x from the export root through the file's parent;
directory r is unnecessary for a known name. An existing-file overwrite needs
file w and path search, not parent-directory w. Paths above the export root and
local client-parent DAC are excluded.

The model does **not** add a hypothetical second raw-ID generic DAC check.
Linux v6.12's ACCESS-capable NFSv3 client permission path uses the returned
server ACCESS mask; generic_permission is the excluded no-ACCESS fallback.
Directory LOOKUP maps to search permission and adds no separate raw-ID DAC
pass. A nonroot request whose primary or supplementary GID 0 maps to anongid
can therefore gain mapped-group search/read/write in this profile. This is
tested alongside negative unmapped-group controls.

Regular files must have owner read and write set and no execution or special
bits: 0600 plus optional group/other r/w. Directories allow ordinary 0777 bits
only. These restrictions avoid nfsd owner/open overrides and executable-read
exceptions. `no_root_squash` may be represented. For writes, the declared client ro gate
and then the export ro gate return their defined denials before numeric mapping;
these outcomes do not depend on modeling UID 0 capabilities. Reads and writes
that pass both mode gates require mapping: effective UID 0 then returns
`SimulatorUnsupported` (UnsquashedRootPrivileges), never a permission denial.
Nonroot identities with GID 0 under no_root_squash are ordinary numeric group cases. A squashed request can write an anonymous-owned 0600 file:
squashing is identity mapping, not deny-all-root.

Excluded: caches, access fallback, retained handles/open descriptors, ACLs,
capabilities, special files, symlinks, hard links, mutable inode metadata,
namespaces, security modules, v4/fsid0, Kerberos/TLS, DNS, discovery, service and
firewall state, privileged ports, retries, locking, durability, stable-storage
measurement and export revocation semantics.

## Supplied context and separate mutable layers

`State::new` and State deserialization validate complete inventory and runtime.
State fields are private; `inventory()` and `runtime()` borrow read-only data.
Full Inventory equality defines `same_context`, not fixture-name equality.
There is no unchecked public validation credential or mutable State accessor.
An eventual authored adapter must compare this complete context to its own
initial definition. This pure module supplies no engine or grading adapter.

The inventory fixes servers/addresses, clients/addresses, allowed mount points,
allowed identity-profile IDs, numeric identities, disjoint exportable roots,
directory/file paths, inode ownership and modes. Each path parent within a root
must be a supplied directory. Current content keys are exactly all supplied
regular files. No action creates, removes, renames, chmods or chowns a file.

Server draft text, saved text and typed applied exports are independent.
Client draft and saved fstab text are independent of current mount records.
Save copies draft bytes exactly, including bounded malformed text; it neither
parses nor applies nor mounts. Apply parses the entire saved server document.
MountSaved parses the entire relevant saved client document. Decoding performs
none of these actions. There is no solved flag, observation history or cached
success authority.

Allowed reads return actual current text. Allowed writes atomically replace the
existing file's entire text, including an empty replacement, and preserve all
metadata and other files. This is virtual text editing, not byte-offset protocol
writes or a durability promise. Denied and error outcomes leave live State exact.

## Closed text grammars

Exports use `/supplied/root IPv4-or-CIDR[(options)] ...`, one line per supplied
root with 1..8 clauses. Blank lines, end-of-line `#` comments, LF and CRLF are
accepted; raw saved bytes remain exact. Parentheses must be adjacent to the
selector. Options are ro/rw, sync, root_squash/no_root_squash, anonuid=N,
anongid=N and optional sec=sys. Defaults are ro, sync, root_squash and anonymous
UID/GID 65534. Duplicates, contradictions and unknown options are refused.
Empty documents compile to empty tables. Sync is declarative, not measured.

IPv4 literals and network/length 0..32 must be canonical. Host bits in a CIDR,
dotted netmasks, hostnames and IPv6 are outside this parser. A host and /32
match identically. Intersecting clauses for the same root are unsupported;
arithmetic intersection checks do not invent longest-prefix or Linux mixed
selector precedence. /0 is representable as a broad policy, not silently
normalized into safety. Only supplied clients can make requests.

Saved mounts use six fields:

`IPv4:/supplied/root /declared/mountpoint nfs vers=3,proto=tcp,sec=sys,ro-or-rw 0 0`

The four options may be reordered but all are required exactly once. Server,
root and local target must be supplied. Targets are unique and nonnested for
each client. No quoting, continuation, wildcard, netgroup, DNS, v4 negotiation,
noauto, automount, systemd boot or unknown option behavior is modeled. The two
zero fields are syntax, not simulated dump/fsck. The real exports whitespace
pitfall must not be taught as safe host rejection merely because this parser
declines it. Whole-document admission and all such refusals are simulator
policies, not claimed Linux utility behavior.

## Mounts, modes and incompatible policy edits

Mount admission uses the supplied client's IPv4 and applied export table. It
does not prove a user can search or read. Client ro/rw is independent of server
ro/rw: a client rw mount on a server ro export succeeds, then writes fail at the
server gate; a client ro mount on a rw export succeeds, then writes fail at the
client gate. RFC 1813's MNT request does not carry the client ro/rw flag; this
module does not implement that exchange.

Mount records bind normalized effective options. Before Apply commits a table,
every current mount of that server must still match the same root and identical
server mode, squashing and anonymous IDs. A compatible host/CIDR rewrite can
preserve a mount. Removal or option changes return the explicitly labeled
`SimulatorUnsupported` ActiveMountConflict naming an affected mount; draft,
saved, applied policy, mounts and files remain exact. Explicitly unmount
affected mounts before Apply. Unrelated mounts remain exact. This conservative
planner restriction is **not** Linux exportfs rollback or old-filehandle
revocation. State admission also rejects stale bindings.

File-access context and canonical supplied-file checks precede permissions.
Write gates are mount present, effective applied binding, client rw, server rw,
supported numeric mapping, each directory search, then file DAC. Read omits the
mode gates. Denials name a typed first stage; unsupported/context/limit faults
are separate types. No-match MountExplicit returns ExportAdmission denial and
adds no mount. Reusing an occupied target requires explicit Unmount.

## Evaluation and resource bounds

`apply(&Action)` validates input and State before private transitions.
`evaluate_batch(&Evaluation)` validates the complete current State once and
uses private query/transition inners. Current queries can inspect mount
presence, actual reads and write permission; permission checks do not publish
file bytes. Fresh trials use one temporary State at a time, with current file
contents and empty scratch mounts. Scratch State is never installed live.

Applied trials use current applied tables. Saved trials name 1..2 servers and
one client: all selected saved export tables and the complete selected saved
mount document parse before the scratch copy; unselected tables are empty in
that trial. Actions must use the selected client and servers. A next saved
mount is an explicit request, not a boot-readiness promise. Trial steps permit
only mount, unmount, read and write, never draft/save/apply or policy replacement.
Expected Denied matches only a typed Decision::Denied at that exact stage.
Unsupported, malformed, foreign-context and limit errors cannot pass it. In
particular, unsquashed UID 0 on a read or rw permission path cannot satisfy an
expected denial; a preceding declared ro write gate is a defined mode denial.

| Resource | Maximum |
| --- | ---: |
| servers / clients / profiles | 2 / 8 / 16 |
| roots / clauses per root | 8 / 8 |
| directories including roots / regular files | 32 / 32 total |
| local mount points per client / total | 4 / 16 |
| supplementary groups | 16 |
| ID / path bytes; path components | 32 / 160; 8 |
| document / line bytes | 8192 / 512 |
| file / aggregate current contents | 4096 / 65536 |
| compact Inventory / raw and compact State | 131072 / 262144 |
| raw or compact Action/Evaluation | 65536 |
| current requests / fresh trials / steps per trial | 32 / 4 / 16 |
| combined calls, including one trial initialization | 128 |
| diagnostic / joint report bytes | 512 / 65536 |

The maximum currently reachable call total is 32 + 4*(1+16) = 100, below the
explicit 128 ceiling. These are model-call counts, not CPU instructions or
wall-clock claims. Reports charge exact escaped borrowed payload sizes before
copying read text, with conservative bounded wrapper reserves, and verify the
final compact report too. Oversized reports return Limit, not denial or silent
truncation. Individually legal maxima need not fit together.

Canonical paths allow ASCII letters/digits and `/_-.` only. They reject empty
components, trailing slash, . and .., metacharacters, controls, backslashes and
percent escapes. An exported root is not `/`. Joins check size/depth before
allocation and stay in the supplied component tree. File text permits UTF8 and
controls except NUL; documents permit tab, LF and CRLF only. No host
canonicalization or filesystem path API is used.

Nested counts and scalar bounds precede context checks, escaped aggregate
streaming counts and candidate copies. Bounded collection visitors refuse item
N+1 before its typed body, with no size_hint reserve; derived structs reject
duplicate/unknown fields. State admission is custom and validated. Public raw
JSON helpers enforce their raw caps before serde. Generic serde callers must
also supply bounded outer input: internally tagged Action/Request/TrialBase
deserialization may buffer serde Content before field visitors; that bounded
outer buffering is not described as streaming allocation immunity.

Set, Save and Write count exact borrowed JSON replacement size before copying
the changed text, without a full-State clone. Mount addition counts a borrowed
prospective array before cloning its small record. Apply parses one bounded
candidate table and preflights bindings/compact size before commit. Fresh
initialization counts all proposed table substitutions before its sole bounded
State copy. There is no global cache or dependency addition.

## Prospective evidence

The new pure-model tests cover two distinct valid plans for each of four
prospective task shapes (restricted reference, write diagnosis, explicit
handover, two-share capstone), negative saved-only/current-missing controls,
mapping each zero group independently, anonymous-owner allow, retained nonzero
groups, no class fallthrough, directory search, mode gates, strict decoding,
full-context inequality, N+1-before-body, canonical paths, joint read/report
limits and escaped candidate frontiers. These are implementation checks, not
authored labs, native acceptance, complete Linux conformance or utility equality.
The initial source freeze marks every test and runtime claim UNRUN.

Primary source boundary: [exports(5)](https://man7.org/linux/man-pages/man5/exports.5.html),
[exportfs(8)](https://man7.org/linux/man-pages/man8/exportfs.8.html),
[nfs(5)](https://man7.org/linux/man-pages/man5/nfs.5.html),
[path_resolution(7)](https://man7.org/linux/man-pages/man7/path_resolution.7.html),
[RFC 1813, 5.2.1](https://www.rfc-editor.org/rfc/rfc1813.html#section-5.2.1),
Linux v6.12 [nfsd identity mapping](https://raw.githubusercontent.com/torvalds/linux/v6.12/fs/nfsd/auth.c),
[nfsd permission exceptions](https://raw.githubusercontent.com/torvalds/linux/v6.12/fs/nfsd/vfs.c),
[client permission path](https://raw.githubusercontent.com/torvalds/linux/v6.12/fs/nfs/dir.c)
and [NFSv3 ACCESS](https://raw.githubusercontent.com/torvalds/linux/v6.12/fs/nfs/nfs3proc.c).
These were read as source/documentation. No NFS utility or kernel behavior was
executed or captured.
