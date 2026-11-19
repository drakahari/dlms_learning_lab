//! Literal Apache 2.4 name-based TLS virtual hosts for a single supplied listener.
use super::{Client, Inventory, Name, Result, Verification, path, verify};
use std::collections::BTreeSet;

pub const CONFIG_LIMIT: usize = 4096;
const MAX_SITES: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Site {
    pub name: Name,
    pub certificate_file: String,
    pub key_file: String,
}

#[derive(Default)]
struct OpenSite {
    name: Option<Name>,
    tls: Option<bool>,
    certificate_file: Option<String>,
    key_file: Option<String>,
}

impl OpenSite {
    fn finish(self) -> Result<Site> {
        if self.tls != Some(true) {
            return Err(
                "This supplied HTTPS listener expects SSLEngine on in each virtual host.".into(),
            );
        }
        Ok(Site {
            name: self
                .name
                .ok_or("Add one ServerName to this virtual host.")?,
            certificate_file: self.certificate_file.ok_or("Add an SSLCertificateFile.")?,
            key_file: self.key_file.ok_or("Add an SSLCertificateKeyFile.")?,
        })
    }

    fn directive(&mut self, name: &str, arg: &str) -> Result<()> {
        match name.to_ascii_lowercase().as_str() {
            "servername" => {
                let dns = Name(arg.into());
                if !dns.dns() {
                    return Err("Enter one literal DNS name in ServerName.".into());
                }
                field(&mut self.name, dns)
            }
            "sslengine" => {
                let flag = match arg.to_ascii_lowercase().as_str() {
                    "on" => true,
                    "off" => false,
                    _ => return Err("Use SSLEngine on or off. Optional upgrade modes are outside this workspace.".into()),
                };
                field(&mut self.tls, flag)
            }
            "sslcertificatefile" => field(&mut self.certificate_file, file_argument(arg)?),
            "sslcertificatekeyfile" => field(&mut self.key_file, file_argument(arg)?),
            "sslcertificatechainfile" => Err("This Apache 2.4 profile expects intermediates in SSLCertificateFile after the leaf.".into()),
            _ => Err("This virtual host editor supports only ServerName, SSLEngine, SSLCertificateFile and SSLCertificateKeyFile.".into()),
        }
    }
}

fn field<T>(slot: &mut Option<T>, value: T) -> Result<()> {
    if slot.is_some() {
        return Err("Supply each supported directive once per virtual host.".into());
    }
    *slot = Some(value);
    Ok(())
}

fn file_argument(word: &str) -> Result<String> {
    let value = if let Some(rest) = word.strip_prefix('"') {
        rest.strip_suffix('"').ok_or("Close the path quote.")?
    } else {
        word
    };
    if !path(value) {
        return Err("Use a supplied literal filename under /etc/ssl/ or /etc/pki/.".into());
    }
    Ok(value.into())
}

pub fn parse(text: &str) -> Result<Vec<Site>> {
    if text.len() > CONFIG_LIMIT
        || text.lines().count() > 48
        || !text.is_ascii()
        || text
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\t' | '\n' | '\r'))
        || text
            .as_bytes()
            .iter()
            .enumerate()
            .any(|(i, &b)| b == b'\r' && text.as_bytes().get(i + 1) != Some(&b'\n'))
    {
        return Err("Use up to 4 KiB and 48 plain-text lines.".into());
    }
    let mut sites = Vec::new();
    let mut active = None::<OpenSite>;
    let mut names = BTreeSet::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let error = |message: &str| format!("Line {}. {message}", n + 1);
        if line.eq_ignore_ascii_case("<VirtualHost *:443>") {
            if active.is_some() || sites.len() == MAX_SITES {
                return Err(error(
                    "Enter up to four unnested virtual hosts on the supplied listener.",
                ));
            }
            active = Some(OpenSite::default());
        } else if line.eq_ignore_ascii_case("</VirtualHost>") {
            let site = active
                .take()
                .ok_or_else(|| error("No virtual host is open."))?
                .finish()
                .map_err(|e| error(&e))?;
            if !names.insert(site.name.0.to_ascii_lowercase()) {
                return Err(error(
                    "Use different ServerName values for these virtual hosts.",
                ));
            }
            sites.push(site);
        } else if let Some(site) = &mut active {
            let args = line.split_ascii_whitespace().collect::<Vec<_>>();
            let [name, value] = args.as_slice() else {
                return Err(error(
                    "Use one directive and one literal value. Put comments on their own lines.",
                ));
            };
            site.directive(name, value).map_err(|e| error(&e))?;
        } else {
            return Err(error(
                "Begin a supplied <VirtualHost *:443> block before adding directives.",
            ));
        }
    }
    if active.is_some() {
        return Err("Close the last virtual host with </VirtualHost>.".into());
    }
    if sites.is_empty() {
        return Err("Supply at least one HTTPS virtual host.".into());
    }
    Ok(sites)
}

pub fn load(inventory: &Inventory, text: &str) -> Result<Vec<Site>> {
    let sites = parse(text)?;
    for site in &sites {
        inventory
            .binding(&site.certificate_file, &site.key_file)
            .map_err(|e| format!("{}. {e}", site.name.0))?;
    }
    Ok(sites)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Connection {
    pub selected: Name,
    pub used_default: bool,
    pub verification: Verification,
}

pub fn inspect(inventory: &Inventory, sites: &[Site], client: &Client) -> Result<Connection> {
    let matched = client.sni.as_ref().and_then(|name| {
        sites
            .iter()
            .find(|site| site.name.0.eq_ignore_ascii_case(&name.0))
    });
    let used_default = matched.is_none();
    let selected = matched
        .or_else(|| sites.first())
        .ok_or("No virtual host is currently applied.")?;
    let (bundle, leaf) = inventory.binding(&selected.certificate_file, &selected.key_file)?;
    Ok(Connection {
        selected: selected.name.clone(),
        used_default,
        verification: verify(inventory, bundle, leaf, client),
    })
}
