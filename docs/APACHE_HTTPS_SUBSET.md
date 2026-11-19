# Apache HTTPS identity subset

The two LPIC-2 v4.5 208.2 activities start with the static-site lesson
`lpic2-web-routing`. The first repairs an incomplete chain while verifying a
second site still works. The second fixes the names and served certificate
pairings across two virtual hosts and preserves the first-host default.
The course records partial coverage for these distinct activities.

## Virtual configuration and client trials

The editor accepts up to four `<VirtualHost *:443>` sections in supplied
order, one literal `ServerName`, `SSLEngine on`, `SSLCertificateFile` and
`SSLCertificateKeyFile` per host. Full-line comments and optional double
quotes around a single literal path are supported. Draft content is bounded
to 4 KiB, and saved configuration to 48 lines. Virtual paths are literal
entries under `/etc/ssl/` or `/etc/pki/`, excluding traversal.

**Save configuration** parses the editor. **Apply saved configuration**
checks certificate and key pairing for every configured host. An invalid
apply leaves the previous live configuration in place. Client trials read
live settings. Editing, saving or applying invalidates an earlier check.
Separate example drafts and the active state persist independently of
assignment work.

A synthetic certificate record has a supplied public-key ID, issuer ID,
exercise validity interval and literal DNS names. The first ID in a
supplied certificate file is the leaf. Intermediate IDs and the client's
fixed anchors are the only candidate issuers. The model walks the supplied
issuer links, rejects a repeated link, requires issuing-CA records, checks
the exercise date and ends at a supplied trusted anchor. Reordering the
entries after the leaf does not reorder the issuer links. A served root
does not change the client's anchors. Applying requires an existing key
record matching the first certificate's public-key ID. The client checks
its intended hostname against that certificate's DNS names separately.

The supplied listener matches the received SNI by case-insensitive
`ServerName`. When SNI is absent or unrecognized, it falls back to the
first host. This simulated listener does not model strict SNI policies,
aliases, DNS lookup, IP-based vhost selection or HTTP requests. DNS name
verification uses literal names and excludes wildcard and IP SAN matching.

The model omits cryptography, certificates on disk, CA requests, encrypted
keys, live listeners, chain fetching, alternate trust paths, revocation,
constraints beyond the declared issuing role, actual wall-clock times and
TLS versions/ciphers. Key records contain no real private material. The app
does not access host certificate stores, reload a service or start Apache.

## Primary references checked November 17, 2026

- [LPI LPIC-2 v4.5 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/)
- [Apache 2.4 mod_ssl certificate-file directives](https://httpd.apache.org/docs/2.4/mod/mod_ssl.html#sslcertificatefile)
- [Apache 2.4 virtual host matching](https://httpd.apache.org/docs/2.4/vhosts/details.html)
- [RFC 9525 service identity in TLS](https://www.rfc-editor.org/rfc/rfc9525.html)
