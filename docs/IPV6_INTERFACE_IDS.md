# Modified EUI-64: configure and verify a workstation

Stable ID `ccna-ipv6-modified-eui64`; unit `ipv6-interface-identifiers`.
CCNA → **IPv6 interface identifiers: modified EUI-64** → **Connect a workstation using modified EUI-64**.
The unit follows the existing IPv6 static foundations and names the earlier
workstation repair as an advisory prerequisite. Existing progress is not reset.

## Learning and actual behavior

Introduction teaches hexadecimal bytes, the /64 split, insertion and bit toggling,
then demonstrates a complete worked configuration. Guided practice uses a different
MAC/network, actual address and route controls, an intentional unreachable next
hop, and a suggested wrong-bit experiment. Assignment starts with an empty address;
its supplied MAC has the relevant bit set, so always setting that bit cannot pass.
The entire address, routing and preservation task remains visible. Challenge uses
the same task with optional assistance. No independent answer is shown by default.

`ipv6::Requirement::ModifiedEui64` computes the required address from supplied
facts at grading time: split the six bytes, insert `ff:fe`, XOR the first byte with
`02`, and join the canonical supplied /64. This is not command-string recognition.
The existing topology still grades actual request/reply paths and protected
addresses independently. Expanded/uppercase address spelling and reachable global
or scoped link-local next hops can pass. An address-only repair stays partial.

Feedback distinguishes empty/malformed input, wrong network half/prefix, missed
bit toggling, other identifier mistakes and an incorrect on-link declaration.
Edits invalidate current success; example drafts and assignment work remain separate.
No new saved-progress version, host API, dependency or executable pack mechanism
is introduced. The simulator does not discover a MAC or generate real addresses.

## Scope and sources

This is a disclosed legacy MAC-derived exercise. Modern stable/privacy schemes
need not embed a permanent MAC; arbitrary interface-identifier bits have no generic
universal/local meaning. Only nonzero unicast48-bit colon-separated MAC facts and
canonical supported global/ULA /64s are admitted here. General64-bit hardware
tokens, SLAAC, ND/RA/DAD, privacy generation and physical-device conformance remain
outside the implemented model. Numeric test vectors are not separately authored labs.

Primary references checked2026-11-10:

- [Cisco CCNA v1.1 blueprint,1.9.d](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf).
- [RFC4291,Appendix A](https://www.rfc-editor.org/rfc/rfc4291.txt): legacy construction.
- [RFC7136](https://www.rfc-editor.org/rfc/rfc7136): later clarification of identifier-bit semantics.
- [RFC8064](https://www.rfc-editor.org/rfc/rfc8064): modern stable-IID recommendations.

Original prose/code only; no external implementation or manual text is imported.
Current1.9.d gains partial practice; no parent objective or future blueprint is
marked complete. Existing multicast, anycast and broader IPv6 gaps remain explicit.

## Preservation and pack authoring

Source373/138 comprises89CCNA,117LPIC-1,90LPIC-2and77Bash labs. All372old scenario
values and order,137old unit objects and88CCNApresentation objects remain exact.
Two adjacent LDAP packs are consolidated into `ldap-administration-v2.json`; their
10whole scenario objects/order and original files are preserved. This frees one
slot for the separate IPv6 pack without increasing128files,32MBaggregate,2MBper
pack,1000scenarios or64registry-traversal bounds. Historical getters stay intact.

Old teaching rows retain their authored origin. The reporting tool accepts only
the two explicit consolidation origins, and only if the entire old scenario
matches its current bundled counterpart. Changed goals, versions, content or
unrelated paths fail. This is source bookkeeping, not extra curriculum or grading.

## Verification evidence

Five arithmetic/diagnostic groups include128unicast first-byte values across two
prefixes and invalid formats/facts. Retained IPv6 core/workspace tests pass. Six
new content/progress tests cover the taught/alternate methods, plausible wrong and
partial repairs, invalid packs, separate practice and actual private save/load.
The latest10registered/progress targets pass37tests, including old LDAP content and
persistence after consolidation. Five new widget/capture tests and nine retained
IPv6 widget/capture tests pass. The final retained native layout check passes after the font-width refinement. These groups overlap; they are not an inclusive-suite count.

Actual development-native R2 `run-_j0gm169` completes36actions/10PNGs, exit0:
keyboard address/next-hop edits, practice isolation, partial/wrong-bit feedback,
alternate next hop, stale-success invalidation, same-process save/read and reset.
Root opened7original PNGs: worked Introduction, practice, assignment, bit feedback,
dark150address and light200address/result. R1is retained; it exposed worked-table
horizontal clipping and an undersized field. The final example has short complete
calculation lines; the input measures native font metrics and retains normal
TextEdit cursor/selection/focus geometry. No text is shrunk.

This is technical development verification, not a novice study, screen-reader or
physical accessibility acceptance. No new installed lifecycle/cold-reopen,
all28theme sweep, inclusive Rust runtime or Windows/macOS/Omarchy acceptance is
claimed. Owner prioritizes core-app completion before further OS delivery work.
Latest admitted packaged review build remains372R1 until a new candidate passes
its own validation. No original profile/installed app is changed or release published.

## Normal application integration

The current catalog is now the default for real learner sessions and unflagged
content validation, not only review builds. The closed201-lab legacy fixture is
retained for historical tests. Normal/private migration tests preserve old whole
attempts and favorites without creating new attempts or credit. Review flags still
select a separate profile. Actual unflagged development-native run-hwo0k4dl passes
37actions/11captures, including the373-lab library. Four originals were opened,
reusing seven reviewed R2screens for unchanged content. Two real normal NFS processes
also pass save/full exit/relaunch/repair/regrade in the private current373 session;
this is not an installed-package lifecycle or original-profile operation.
