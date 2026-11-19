//! Supplied fictional vendor facts, never inferred from an upstream version.
use crate::rpm_packages::{Evr, Identity, State};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Origin {
    #[serde(deserialize_with = "crate::rpm_packages_workspace::string::<_,64>")]
    pub product: String,
    #[serde(deserialize_with = "crate::rpm_packages_workspace::string::<_,64>")]
    pub release: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Advisory {
    #[serde(deserialize_with = "crate::rpm_packages_workspace::string::<_,64>")]
    pub id: String,
    pub package: Identity,
    #[serde(deserialize_with = "crate::rpm_packages_workspace::string::<_,64>")]
    pub product: String,
    #[serde(deserialize_with = "crate::rpm_packages_workspace::string::<_,64>")]
    pub release: String,
    pub fixed_in: Evr,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    #[serde(deserialize_with = "crate::rpm_packages_workspace::list::<_,Advisory,4>")]
    pub advisories: Vec<Advisory>,
    #[serde(deserialize_with = "origins")]
    pub origins: BTreeMap<String, Origin>,
}
fn label(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_. ".contains(&b))
}
impl Policy {
    pub fn validate(&self, initial: &State) -> Result<(), String> {
        let catalog = initial.catalog();
        if self.advisories.is_empty()
            || self.advisories.len() > 4
            || self.origins.len() != catalog.archives.len()
            || self.origins.len() > 12
            || self.origins.iter().any(|(id, origin)| {
                !catalog.archives.contains_key(id)
                    || !label(&origin.product)
                    || !label(&origin.release)
            })
        {
            return Err(
                "Patch activities need a bounded supplied origin for every archive.".into(),
            );
        }
        for (index, a) in self.advisories.iter().enumerate() {
            if !a.id.starts_with("TRAINING-")
                || !label(&a.id)
                || !label(&a.product)
                || !label(&a.release)
                || self.advisories[..index].iter().any(|old| old.id == a.id)
                || !catalog.archives.iter().any(|(id, archive)| {
                    archive.identity == a.package
                        && self.origins[id].product == a.product
                        && self.origins[id].release == a.release
                        && !archive.evr.compare(&a.fixed_in).is_lt()
                })
            {
                return Err(
                    "Use distinct fictional training advisories with a compatible supplied fix."
                        .into(),
                );
            }
        }
        Ok(())
    }
    pub fn check(&self, state: &State, id: &str) -> (bool, String) {
        let Some(a) = self.advisories.iter().find(|a| a.id == id) else {
            return (false, "The supplied advisory is unavailable.".into());
        };
        let Some(installed) = state
            .runtime()
            .installed
            .iter()
            .find(|p| p.identity == a.package)
        else {
            return (
                false,
                format!(
                    "{} is missing; removing the application does not apply {id}.",
                    a.package.name
                ),
            );
        };
        let Some(archive) = state.catalog().archives.get(&installed.archive) else {
            return (false, "The installed archive is unavailable.".into());
        };
        let Some(origin) = self.origins.get(&installed.archive) else {
            return (
                false,
                "The installed archive has no supplied vendor/release facts.".into(),
            );
        };
        if origin.product != a.product || origin.release != a.release {
            return (
                false,
                format!(
                    "{} targets {} {}, but {id} requires {} {}. A higher version from another release is not this fix.",
                    a.package.name, origin.product, origin.release, a.product, a.release
                ),
            );
        }
        if archive.evr.compare(&a.fixed_in).is_lt() {
            return (
                false,
                format!(
                    "{} is {}, below the supplied fixed build {} for {id}. Compare the whole package version and release; a backport can retain the upstream version.",
                    a.package.name,
                    archive.evr.as_str(),
                    a.fixed_in.as_str()
                ),
            );
        }
        (
            true,
            format!(
                "{} {} satisfies {id} for {} {}.",
                a.package.name,
                archive.evr.as_str(),
                a.product,
                a.release
            ),
        )
    }
}

fn origins<'de, D: serde::Deserializer<'de>>(d: D) -> Result<BTreeMap<String, Origin>, D::Error> {
    use serde::de::{Error as _, MapAccess, Visitor};
    struct Entries;
    impl<'de> Visitor<'de> for Entries {
        type Value = BTreeMap<String, Origin>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("At most twelve distinct bounded archive origins")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut result = BTreeMap::new();
            while let Some(crate::rpm_packages_workspace::Text(key)) =
                map.next_key::<crate::rpm_packages_workspace::Text<32>>()?
            {
                if result.len() == 12 || result.contains_key(&key) {
                    return Err(A::Error::custom("Duplicate/extra patch origin before body"));
                }
                result.insert(key, map.next_value()?);
            }
            Ok(result)
        }
    }
    d.deserialize_map(Entries)
}
