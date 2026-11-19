//! Source-enabled virtual availability; no host device or process I/O.
//! Availability is distinct from storage metadata, mounting and boot policy.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    #[serde(deserialize_with = "map::<_,bool,16>")]
    pub active: BTreeMap<String, bool>,
    #[serde(deserialize_with = "ids")]
    pub open: Vec<String>,
}
fn ids<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Id(#[serde(deserialize_with = "text::<_,64>")] String);
    sequence::<_, Id, 16>(d).map(|v| v.into_iter().map(|s| s.0).collect())
}
pub const HELP: &str = "Supplied virtual LV availability: vgchange -a y|n VG; vgchange --activate y|n VG; vgchange -ay VG; vgchange -an VG. Exactly one named group is required. y makes all its LVs available; n makes them unavailable while retaining allocations and data. This subset refuses group deactivation if any supplied LV is marked open; real tools can have partial effects, and this is not a real-system safety guarantee. No automatic activation ay, missing-PV/degraded/partial activation, lvchange, force, omitted-group/all-host scan, boot policy, mount, udev or real kernel/devmapper operation. lvs adds a simulator ModelActive column only here. Existing disclosed linear growth/creation and raw-retirement forms remain available; consult the unit reference for their exact syntax. Unsupported shell syntax never runs a host shell.";
impl Model {
    pub(super) fn validate(&self, s: &State) -> Result<()> {
        if s.provisioning != Some(provision::Profile::Lifecycle)
            || self.active.len() > MAX_VOLUMES
            || self.open.len() > MAX_VOLUMES
            || self.active.keys().any(|id| id.len() > 64)
        {
            return Err(invalid(
                "Availability needs its supplied lifecycle profile and at most16 LVs",
            ));
        }
        let expected = s
            .groups
            .iter()
            .flat_map(|(g, d)| d.volumes.keys().map(move |v| format!("{g}/{v}")))
            .collect::<BTreeSet<_>>();
        if self.active.keys().ne(expected.iter()) {
            return Err(invalid(
                "Availability must name every current LV exactly once, using canonical VG/LV IDs",
            ));
        }
        let mut seen = BTreeSet::new();
        for id in &self.open {
            if !seen.insert(id) || self.active.get(id) != Some(&true) {
                return Err(invalid(
                    "Supplied open-LV facts must be unique, present and active",
                ));
            }
        }
        Ok(())
    }
}
pub(crate) fn same_context(a: &State, b: &State) -> bool {
    match (&a.activation, &b.activation) {
        (None, None) => true,
        (Some(a), Some(b)) => a.open == b.open,
        _ => false,
    }
}
pub fn is_active(s: &State, id: &str) -> Result<bool> {
    let (g, lv) = volume_name(id)?;
    s.volume(id)?;
    match &s.activation {
        None => Err(error(
            ErrorKind::Unsupported,
            "Activation is not modeled in this source profile",
        )),
        Some(a) => a
            .active
            .get(&format!("{g}/{lv}"))
            .copied()
            .ok_or_else(|| invalid("Availability entry is missing")),
    }
}
pub(super) fn require_active(s: &State, id: &str) -> Result<()> {
    if s.activation.is_none() {
        return Ok(());
    }
    if !is_active(s, id)? {
        return Err(error(
            ErrorKind::Occupied,
            "LV inactive: its allocation and files are retained. Activate only the required group to make the simulated reader available.",
        ));
    }
    Ok(())
}
pub(super) fn removable(s: &State, id: &str) -> Result<()> {
    let (g, lv) = volume_name(id)?;
    if s.activation
        .as_ref()
        .is_some_and(|a| a.open.contains(&format!("{g}/{lv}")))
    {
        return Err(error(
            ErrorKind::Occupied,
            "This supplied LV is marked open. This subset cannot remove it or close its consumer; retained work is unchanged.",
        ));
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Action {
    group: String,
    active: bool,
}
pub(super) fn parse(args: &[&str]) -> Result<Action> {
    let (value, group) = match args {
        ["vgchange", "-a" | "--activate", value, group] => (*value, *group),
        ["vgchange", "-ay", group] => ("y", *group),
        ["vgchange", "-an", group] => ("n", *group),
        _ => return Err(error(ErrorKind::Unsupported, HELP)),
    };
    if !name(group) || !matches!(value, "y" | "n") {
        return Err(error(ErrorKind::Unsupported, HELP));
    }
    Ok(Action {
        group: group.into(),
        active: value == "y",
    })
}
pub(super) fn apply(s: &mut State, a: Action) -> Result<String> {
    let Some(model) = &mut s.activation else {
        return Err(error(ErrorKind::Unsupported, s.command_help()));
    };
    let group = s.groups.get(&a.group).ok_or_else(|| {
        error(
            ErrorKind::Unknown,
            "No supplied VG by that name; inspect vgs and lvs",
        )
    })?;
    if !a.active
        && model
            .open
            .iter()
            .any(|id| id.split_once('/').is_some_and(|(g, _)| g == a.group))
    {
        return Err(error(
            ErrorKind::Occupied,
            "This group contains an LV marked open in the supplied facts. The bounded simulator refuses this deactivation before changing any LV; real tools may have partial effects. Preserve the live group and choose only the intended unused storage.",
        ));
    }
    for lv in group.volumes.keys() {
        *model.active.get_mut(&format!("{}/{lv}", a.group)).unwrap() = a.active;
    }
    Ok(format!(
        "Virtual availability: {} LV(s) in {} now {}. Allocations, files and device sizes retained; no mount or boot setting changed.",
        group.volumes.len(),
        a.group,
        if a.active { "active" } else { "inactive" }
    ))
}
