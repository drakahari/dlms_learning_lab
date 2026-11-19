# Settled OSPF designated roles on a shared segment

After point-to-point OSPF and reading installed routes, learners configure
interface priorities for one shared virtual broadcast segment. They examine
three separate observations. A fresh group has no incumbent roles. A scheduled
higher-priority neighbor arrives after the original roles settle. The original
primary then becomes unavailable, and the original backup can take over.

Priority zero excludes a router from DR and BDR roles while retaining its
neighbor relationships. In a fresh simultaneous election the model compares
positive priority first and numeric Router ID next. When a compatible
incumbent remains, a higher-priority arrival does not replace it. The
independent outage trial checks actual promotion, which requires that the
router served as backup before the outage. A router already holding the
designated role cannot receive promotion credit.

The model shows whether two compatible two-way neighbors would attempt
adjacency, based on their roles. It does not assert that database
synchronization completed. The exercise uses one supplied segment with
consistent settled declarations. It does not model concurrent election
views, packet delivery, wait and dead timers, the neighbor FSM, LSAs,
authentication, live reconvergence, configuration commands or hardware.
Priority fields change only the in-process exercise.

This is narrow practical coverage for current CCNA v1.1 3.4.c. The earlier
P2P compatibility, Router ID, passive LAN and cost lessons retain their
existing behavior. Sources checked November 17, 2026.

- [Cisco 200-301 v1.1 exam topics](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf)
- [RFC 2328 sections 9.4 and 10.4](https://www.rfc-editor.org/rfc/rfc2328)
- [Cisco OSPF interface output](https://www.cisco.com/c/en/us/support/docs/ip/open-shortest-path-first-ospf/13689-17.html)
