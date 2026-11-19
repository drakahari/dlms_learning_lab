## Current source370; new deployment lesson pending native review

The earlier foundation checkpoints below are historical. Source370/135now
registers lpic2-ldap-config-deployment; actual native and exact package gates are
pending. R10all27configuration tests, R13all26registered/retained tests, R14new4
and R15retained4widget/capture checks pass. Ready369R1is latest admitted package
and does not contain this later capability. No new objective credit yet.

# Bounded directory-backed access configuration

Development foundation, not a registered lesson or a full server. Catalog remains
369/134; Ready369R1is the latest verified package and does not contain this later
capability. Prior static policy/contacts/utility behavior stays opt-in and unchanged.

## Exact source and input subset

One supplied frontend, one config database and one/two disjoint MDB data suffixes.
A separate config-name type accepts cn=config and indexed olcDatabase entries:
frontend{-1}, config{0}, MDB1through31. This bound is a simulator limit. Unindexed
creation names, other backends/branches and escaped config names are unsupported;
the old data-DN parser still accepts only cn/ou/uid/dc. Source version1has bounded
entry/value/name/DN counts and lengths and rejects unknown fields.

The separate LDIF codec requires version:1, explicit changetype:modify and exactly
one replace:olcAccess per record. LF/CRLF, comments, folding and bounded printable
base64 use the existing decoder. Max64KiB/1024physical lines/16records, each with
16values of1024decoded bytes. The old contact codec retains its256byte value limit
and mail/description-only changes. URL/control and unknown syntax fail; no resource
is opened. An empty replacement removes the local access list.

Values are to/by expressions using the existing ordered-policy subset. Complete
replacement accepts all unindexed values or a unique contiguous explicit{N}set
starting at0; mixed/gapped/other indexing fails. Canonical readback has automatic
tags. Backend rules precede frontend rules; rootDN and empty-list read default
reuse the ordered evaluator. Combined policy bounds are checked before commit.
Only MDB access values can change. Global/config/frontend, suffix, rootDN, modules,
schema, passwords, host paths and other server settings are unsupported.

Each request stages and commits once. Complete-document syntax preflight happens
before execution; later semantic rejection retains earlier committed records and
reports failure. No whole-document transaction is claimed. Administrator identity
is a supplied established scenario fact, never a simulated fresh Bind credential
or permission for host activity. This is not a server/process/network/crypto API.

## Evidence

Private next-ldap-config R01library51.67s, R02all15configuration tests and R03all66
retained LDIF/query/change/utility checks pass. Tests include actual protected
contact Modify, peer denial and whole-directory preservation after a later field
is denied; correct ordered values, defaults, rootDN, wrong/protected targets and
source/input bounds. R04Clippy found a collapsible if; corrected without suppression,
R05focusedClippy passes28.49s. R06workspace integration compiles51.46s.
R07retained authored6and workspace7checks pass; five of six new integration
checks pass. One new test helper incorrectly treated a full LDIF document as
a static fragment; its unused duplicate construction is removed without changing
any assertion. Corrected rerun follows. No dependent assignment, UI, coverage credit, real OpenLDAP/slapacl
comparison, cold reopen or platform/novice/accessibility acceptance is claimed.

## Primary references

Checked2026-11-09, OpenLDAP2.6AdminGuide:
https://www.openldap.org/doc/admin26/slapdconf2.html
https://www.openldap.org/doc/admin26/access-control.html (8.3.6ordering)
OpenLDAP-authored manual hosted by Debian:
https://manpages.debian.org/trixie/slapd/slapd-config.5.en.html
The official man.cgi endpoint was unavailable; no tools/dependencies installed.
Configuration is changed through LDAP operations; never teach directly editing
slapd.d backing LDIF as live administration. No external source text/code imported.

Final source gates: R22all-targetClippy1m16s, R25all992Python12.888s, R26inventory
370(365updated/5already), R27strictcoverage pass.301Partial/7Supporting/932Open;
all136parents incomplete. Exact isolated370package remains pending; selected
inputs now freeze through its readbacks/native review/receipt. Ready369retained.

## Exact isolated370package verified

R28optimized build8m57s, R29/R32selected identity, R30content370/135, R31native
76actions/17PNG and R33receipt pass. Root9originals opened in run-woch6zet across
all3views, wrong/partial feedback, dark150and light200. Receipt precedes freeze
lift; do not replay old frozen MAINreaders after these notes. Selected identity
is not full compiler closure, independent seal or broader acceptance. No installed
app/original progress or system/publication changes. CHECKPOINT.md records exact
launch/hashes and limitations.
