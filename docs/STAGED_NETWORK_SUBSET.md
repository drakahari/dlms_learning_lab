# Stage network changes and verify installed policy

These offline exercises address parts of the current CCNA v1.1 scope
under 6.1, 6.2 and 6.3. They follow the structured inventory and REST
foundations. Each starts with an annotated example, offers a separate
guided workspace and checks an independent assignment.

## Controller rollout and direct device edits

The supplied controller resource inventory uses the existing access-port
projection and Ethernet path checks. A device edit introduces a local
override. A shared profile edit updates its bound slots, while
reconciliation restores the controller's selected settings. Preview
lists both effective port changes and binding or override changes. A
preview binds to the exact snapshot and source. A later change makes
that preview stale. No effects occur until the learner applies it.

The replayed virtual rollout history records when one approved pilot
reached the intended service, the peer still had its initial settings,
and reconciliation would preserve the pilot. Final checks repair the
other approved desk, keep the reserve scanner in its initial state and
preserve separation from another workgroup. Either approved desk can
be the pilot. The model does not implement source control, automated
deployment systems, remote APIs or vendor controller protocols.

## Desired policy, installed rules and underlay

A northbound request changes the desired rule. An explicit publish
copies it to the installed table. Trial results compare each independent
layer. Actual routed IPv4 probe and reply observations check the
supplied underlay. This lesson's virtual overlay uses a small effective
table. It separately checks intended forwarding, expected exclusions,
and the restored transport uplink.

No packet encapsulation, control protocol, dynamic topology,
credential, server, real network activity or vendor fabric is used.
The forward and reply underlay observations support a deliberately
bound TCP/IP concept lesson. They do not claim implementation of a
specific overlay or device forwarding pipeline.

Primary sources checked November 17, 2026:
[Cisco CCNA v1.1 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf) ·
[SDN architecture terminology, RFC 7426](https://www.rfc-editor.org/rfc/rfc7426.html)
