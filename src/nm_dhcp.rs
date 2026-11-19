//! Finite supplied DHCP outcomes, never a daemon, host request or lease engine.
use crate::{ipv4_routing::Assignment, linux_host};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, Visitor},
};
use std::{collections::BTreeMap, fmt, net::Ipv4Addr};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Outcome {
    Bound {
        address: Assignment,
        router: Option<Ipv4Addr>,
    },
    Unavailable {},
}
pub type Outcomes = BTreeMap<String, Outcome>;

pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Outcomes, D::Error> {
    struct Inputs;
    impl<'de> Visitor<'de> for Inputs {
        type Value = Outcomes;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most two distinct interface-bound DHCP outcomes")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Outcomes, A::Error> {
            #[derive(Deserialize)]
            struct Name(#[serde(deserialize_with = "crate::cron::text::<_,15>")] String);
            struct Excess;
            impl<'de> Deserialize<'de> for Excess {
                fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
                    Err(D::Error::custom("DHCP outcome count limit"))
                }
            }
            let mut out = Outcomes::new();
            while out.len() < 2 {
                let Some(Name(name)) = a.next_key()? else {
                    return Ok(out);
                };
                if out.contains_key(&name) {
                    return Err(A::Error::custom("Duplicate DHCP interface"));
                }
                out.insert(name, a.next_value()?);
            }
            let _ = a.next_key::<Excess>()?;
            Ok(out)
        }
    }
    d.deserialize_map(Inputs)
}

pub fn validate(inputs: &Outcomes, physical: &linux_host::Profile) -> Result<(), String> {
    if inputs.len() > 2 {
        return Err("At most two supplied DHCP outcomes".into());
    }
    for (name, outcome) in inputs {
        let link = physical
            .links
            .iter()
            .find(|l| l.name == *name)
            .ok_or("DHCP outcome refers to an unknown interface")?;
        if let Outcome::Bound { address, router } = outcome {
            if !(1..=30).contains(&address.prefix)
                || !address.usable()
                || !crate::ipv4_routing::supported_address(address.address)
                || !link.network.contains(address.address)
                || crate::ipv4_routing::Prefix::new(address.address, address.prefix)
                    != Some(link.network)
            {
                return Err(format!(
                    "{name}: supplied DHCP address/prefix must be usable and match the modeled physical subnet"
                ));
            }
            if let Some(router) = router {
                let gateway = Assignment {
                    address: *router,
                    prefix: address.prefix,
                };
                if !gateway.usable()
                    || !link.network.contains(*router)
                    || *router == address.address
                {
                    return Err(format!(
                        "{name}: supplied DHCP router must be a different usable address in this subnet"
                    ));
                }
            }
        }
    }
    Ok(())
}
