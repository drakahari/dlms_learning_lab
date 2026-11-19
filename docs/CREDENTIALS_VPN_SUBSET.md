# CCNA password policy and IPsec design subsets

The two workspaces extend bounded practice for CCNA v1.1 sections 5.4 and
5.5. Both provide worked concept examples, a separate editable practice
case and a graded assignment using different inputs.

## Password policy

The learner repairs a staff policy and compares observed outcomes for
proposed passwords, staff sign-ins and a confirmed compromise. Approved
and denied attempts are tested independently. A length boundary and a
list check act on a proposed password. Replacement conditions and the
evidence of an additional factor act on sign-in. A second memorized
secret remains a knowledge factor. Certificate-based proof in this
exercise includes verified possession of the associated key. A supplied
local biometric unlock is followed by proof from the bound device.

Only supplied facts enter the simulator. No raw credentials, actual
hashing, cryptography, biometric information, authenticator enrollment
or account recovery are implemented. The numeric thresholds in these
lessons belong to the supplied organization. They cannot establish
compliance with a standard.

## Site-to-site and remote access design

The learner applies two virtual profiles and checks selected traffic
rather than running a protocol. Site-to-site profiles use two gateways
and their LANs. Remote access uses the individual client and an office
gateway, with a deliberately narrower protected office range. Inner
addresses are compared as aligned IPv4 ranges. A /32 specifies a
single remote host. Actual trial output separates the traffic's inner
addresses from both outer endpoints.

The workspace checks exact scope and path behavior. It accepts either
profile order and an equivalent exchange of endpoint sides. Broader
selectors, a wrong endpoint role and an unapplied draft fail the
relevant checks. The model supplies the topology and treats same-LAN
traffic locally. It does not implement a real routing decision,
encryption, authentication, IKE, security associations, packet exchange
or vendor CLI. No input can configure or contact a host, network or
actual VPN service.

Primary references checked November 17, 2026:
[CCNA v1.1 exam topics](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf) ·
[NIST authenticator requirements](https://pages.nist.gov/800-63-4/sp800-63b.html) ·
[IPsec architecture and tunnel headers](https://www.rfc-editor.org/rfc/rfc4301.html)
