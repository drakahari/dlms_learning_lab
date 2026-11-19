# Installed IPv4 route reading

This scoped work order extends the point-to-point OSPF cost unit. The activity
combines the existing virtual OSPF SPF result with connected/local entries
and eligible on-link static candidates for one inspected router.

The implemented comparison has three stages.

1. The OSPF snapshot derives its lowest-cost path or paths from directed
   configured costs. Intra-area route cost includes the announced LAN.
2. For the same exact network and mask, routes compete on administrative
   distance. The virtual local/connected distance is zero and the supplied
   default OSPF distance is 110. Static eligibility is derived from the
   existing IPv4 route model. Distance 255 prevents installation.
3. Destination lookup chooses the longest prefix among installed entries.
   A separate exact query uses a contiguous dotted-decimal mask.

The activity displays C, L, O, S and S* for the included choices. The standby
trial withdraws only the virtual service announcement on a scratch copy. It
does not modify the current table or present local lookup as end-to-end traffic.
Grade checks cover two inside addresses, an address outside the service prefix,
both route sources, one local address, a connected subnet, and the default route.
Multiple settings that produce the required outcomes are accepted.

## Boundaries

This module is not a general IOS CLI or live redistribution model. It provides
one local IPv4 table, the supplied single-area OSPF graph, and on-link static
resolution. It does not compare unlike metrics when administrative distances
tie. Real-time convergence, recursive/BFD/track resolution, default origination,
DR/BDR operation, other protocols and real equipment are outside this activity.
The withdrawal trial is a planning comparison, not a protocol timer emulation.
All device and table values are in-process. No packets or host commands run.

The modeled practical slices are current CCNA v1.1 3.1.a, 3.1.c, 3.1.f and
3.2.c. This is partial practice for those rows. V2.0 remains separately
planned. Sources checked November 17, 2026.

- [Cisco 200-301 v1.1 exam topics](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf)
- [Cisco route selection discussion](https://www.cisco.com/c/en/us/support/docs/ip/enhanced-interior-gateway-routing-protocol-eigrp/8651-21.html)
- [Cisco administrative distance and floating static routes](https://www.cisco.com/c/en/us/support/docs/ip/border-gateway-routing-protocol-bgp/15986-admin-distance.html)
- [Cisco IOS XE routing command reference](https://www.cisco.com/c/en/us/td/docs/ios-xml/ios/iproute_pi/command/iri-cr-book/iri-cr-s1.html)
- [RFC 2328](https://www.rfc-editor.org/rfc/rfc2328)
