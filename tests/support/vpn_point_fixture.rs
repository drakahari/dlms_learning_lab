use dlms_learning_lab::vpn_point::*;
pub fn peer(server: bool) -> String {
    let (role, cert, key, local, remote, expected, subject, route) = if server {
        (
            "tls-server",
            "hq.crt",
            "hq.key",
            "10.8.0.1",
            "10.8.0.2",
            "client",
            "branch",
            "10.40.0.0",
        )
    } else {
        (
            "tls-client",
            "branch.crt",
            "branch.key",
            "10.8.0.2",
            "10.8.0.1",
            "server",
            "hq",
            "10.20.0.0",
        )
    };
    format!(
        "# Virtual peer configuration, not a host deployment\ndev tun\nproto udp\n{role}\n{}ifconfig {local} {remote}\nca /etc/openvpn/ca.crt\ncert /etc/openvpn/{cert}\nkey /etc/openvpn/{key}\nremote-cert-tls {expected}\nverify-x509-name {subject} name\nroute {route} 255.255.255.0\n",
        if server {
            ""
        } else {
            "remote 203.0.113.10 1194\n"
        }
    )
}
pub fn supplied() -> Supplied {
    let file = |path: &str, material| VirtualFile {
        path: format!("/etc/openvpn/{path}"),
        material,
    };
    Supplied {
        peers: [
            Peer {
                label: "hq".into(),
                endpoint: "203.0.113.10".parse().unwrap(),
            },
            Peer {
                label: "branch".into(),
                endpoint: "203.0.113.20".parse().unwrap(),
            },
        ],
        underlay_available: true,
        files: vec![
            file(
                "ca.crt",
                Material::Ca {
                    id: "classroom-ca".into(),
                },
            ),
            file(
                "other-ca.crt",
                Material::Ca {
                    id: "other-ca".into(),
                },
            ),
            file(
                "hq.crt",
                Material::Certificate {
                    subject: "hq".into(),
                    issuer: "classroom-ca".into(),
                    key_id: "hq-pair".into(),
                    role: Role::Server,
                    valid: true,
                },
            ),
            file(
                "branch.crt",
                Material::Certificate {
                    subject: "branch".into(),
                    issuer: "classroom-ca".into(),
                    key_id: "branch-pair".into(),
                    role: Role::Client,
                    valid: true,
                },
            ),
            file(
                "hq.key",
                Material::Key {
                    key_id: "hq-pair".into(),
                },
            ),
            file(
                "branch.key",
                Material::Key {
                    key_id: "branch-pair".into(),
                },
            ),
        ],
    }
}
pub fn configs() -> [Config; 2] {
    [parse(&peer(true)).unwrap(), parse(&peer(false)).unwrap()]
}
