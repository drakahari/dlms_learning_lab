# Private office allocation project

One substantive project follows the existing IPv4, VLSM and gateway foundations.
It asks the learner to choose available private space for a small office, meet a
20–62 usable-host policy, avoid three reserved allocations, give two computers
different compatible addresses and verify request/reply traffic. It supplements
the existing workstation subnet lesson; it is not a numerical variant or its UI
replacement. Normal integration and final acceptance are pending for this draft.

Learn introduces reusable private pools, whole-block sizing, mask agreement and
overlap before assessment. A separate Aspen/Willow example uses different names,
requirements and addresses. Filling that worked draft does not execute or grade
anything. The suggested mask change deliberately leaves traffic working while
the same-network requirement fails; repair and another valid host can be tried
without changing the assignment. Challenge retains the complete office task and
native editor, with assistance requested explicitly.

The existing native topology/configuration workspace is retained. Select Alder
or Birch, edit address/prefix, then use the single Check configuration action.
Wrong private space, capacity, overlap, host choice and differing masks receive
specific feedback. Routing is unnecessary on this supplied single LAN. Larger
text uses ordinary page scrolling and shared centered single-line controls.

The model adds typed PrivateSubnet and SameSubnet requirements. PrivateSubnet
checks the complete aligned block against one RFC1918 pool, conventional usable
capacity and every reserved prefix in both overlap directions. SameSubnet checks
usable, different hosts and equal aligned network/prefixes. Logical Reachable
checks also require the modeled reply. Traffic alone cannot replace mask
agreement: two nearby hosts with different masks can still exchange packets.
Conventional interface prefixes remain /1–/30; /31 and /32 remain excluded.

Source authoring checked2026-10-29 against the current
[Cisco CCNA v1.1 topics](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf),
[RFC1918](https://www.rfc-editor.org/rfc/rfc1918.html) and
[IANA special-use registry](https://www.iana.org/assignments/iana-ipv4-special-registry).
Only a narrow current1.7 practice claim is proposed. Other special-use addresses
are not automatically public. This activity models neither Internet reachability
nor NAT/security, actual packets, ARP, a host OS or equipment. Private addresses
do not themselves provide encryption. Future CCNA versions get no inherited
credit, and this project is not complete exam preparation.

The declarative pack is ipv4-private-planning-v1.json. Requirements and native
drafts live in ipv4_routing.rs, ipv4_routing_teaching.rs and their existing
workspace. Content tests exercise two different valid pools/plans, capacity and
reserved-boundary positives,13 wrong/partial plans, malformed input, forbidden
prefixes, practice changes/repair, stale-success invalidation and actual progress
receipts/reset. Added registration tests compare complete previous objects and
unit order. Verification receipts must retain exact source/executable identities;
a prior component screen is not installed-candidate acceptance.
