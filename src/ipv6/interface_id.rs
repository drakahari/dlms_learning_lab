//! Pure legacy modified EUI-64 arithmetic, not SLAAC or a host-interface API.
use super::{Assignment, Prefix};
use std::net::Ipv6Addr;

/// Accept the six-byte, colon-delimited unicast MAC format disclosed by the lab.
/// This activity excludes the unassigned all-zero and group-address cases.
pub fn modified_eui64(mac: &str) -> Result<[u8; 8], String> {
    if mac.len() != 17 {
        return Err("Use six hexadecimal byte pairs separated by colons.".into());
    }
    let mut bytes = [0_u8; 6];
    let mut count = 0;
    for (i, part) in mac.split(':').enumerate() {
        if i >= 6 || part.len() != 2 || !part.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("Use six hexadecimal byte pairs separated by colons.".into());
        }
        bytes[i] = u8::from_str_radix(part, 16)
            .map_err(|_| "A MAC byte must contain two hexadecimal digits.".to_owned())?;
        count += 1;
    }
    if count != 6 || bytes == [0; 6] || bytes[0] & 1 != 0 {
        return Err("This exercise requires a nonzero unicast 48-bit MAC address.".into());
    }
    Ok([
        bytes[0] ^ 0x02,
        bytes[1],
        bytes[2],
        0xff,
        0xfe,
        bytes[3],
        bytes[4],
        bytes[5],
    ])
}

/// Combine a canonical supplied /64 with the derived identifier.
/// Modern stable/privacy IID generation is deliberately outside this helper.
pub fn address(mac: &str, network: &Prefix) -> Result<Ipv6Addr, String> {
    if network.length != 64 || !network.valid() || !super::supported_unicast(network.address) {
        return Err("Use a canonical supported global or unique-local /64 network.".into());
    }
    let iid = modified_eui64(mac)?;
    let mut octets = network.address.octets();
    octets[8..].copy_from_slice(&iid);
    Ok(Ipv6Addr::from(octets))
}

pub(super) fn check(a: Option<&Assignment>, mac: &str, network: &Prefix) -> (bool, String) {
    let expected = match address(mac, network) {
        Ok(a) => a,
        Err(e) => return (false, e),
    };
    let Some(a) = a else {
        return (false, "Enter the full IPv6 address: the supplied network half plus the MAC-derived interface half.".into());
    };
    let issue = if a.prefix != 64 {
        Some("Keep /64: this task uses 64 network bits and 64 interface-identifier bits.")
    } else if !network.contains(a.address) {
        Some(
            "The network half differs from the supplied /64. Keep that half; derive only the final 64 bits from the MAC.",
        )
    } else if a.address != expected {
        let mut unflipped = expected.octets();
        unflipped[8] ^= 0x02;
        if a.address.octets() == unflipped {
            Some(
                "The ff:fe insertion is right, but the first MAC byte was not toggled. Invert its 0x02 bit; do not always set or always clear it.",
            )
        } else {
            Some(
                "The interface half differs. Keep the six MAC bytes in order, insert ff:fe between the third and fourth bytes, then toggle the first byte's 0x02 bit.",
            )
        }
    } else if !a.on_link {
        Some(
            "The address is derived correctly. Mark this supplied /64 on-link so the workstation treats its neighbours as local.",
        )
    } else {
        None
    };
    match issue {
        Some(s) => (false, s.into()),
        None => (
            true,
            "The MAC-derived address, /64 and on-link setting meet the task.".into(),
        ),
    }
}
