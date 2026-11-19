//! Optional typed PV-size goals alongside the existing LV/filesystem outcomes.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub device: String,
    pub bytes: u64,
}
pub(super) fn targets<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Target>, D::Error> {
    bounded::sequence::<_, Target, 8>(d)
}
pub(super) fn valid(initial: &lvm::State, targets: &[super::Target]) -> bool {
    let count = targets.iter().map(|t| t.pv_sizes.len()).sum::<usize>();
    if count == 0 {
        return initial.pv_capacity.is_none() && targets.iter().all(|t| !t.preserve_volume);
    }
    if initial.pv_capacity.is_none() || count > 8 {
        return false;
    }
    let mut seen = BTreeSet::new();
    let mut expected = initial.clone();
    for t in targets {
        let Some((group, _)) = t
            .volume
            .strip_prefix("/dev/")
            .unwrap_or(&t.volume)
            .split_once('/')
        else {
            return false;
        };
        if t.preserve_volume
            && (t.pv_sizes.is_empty()
                || initial.lv_bytes(&t.volume).ok() != Some(t.minimum_lv)
                || initial.filesystem_bytes(&t.volume).ok() != Some(t.minimum_filesystem))
        {
            return false;
        }
        for p in &t.pv_sizes {
            let Some(d) = initial.devices.get(&p.device) else {
                return false;
            };
            if !seen.insert(&p.device)
                || d.role != lvm::DeviceRole::PhysicalVolume
                || d.group != group
                || p.bytes < lvm::DATA_START + lvm::MIB
                || p.bytes > d.bytes
                || p.bytes % 512 != 0
                || (p.bytes != d.bytes && Some(p.bytes) != d.pv_bytes && p.bytes % lvm::MIB != 0)
            {
                return false;
            }
            expected.devices.get_mut(&p.device).unwrap().pv_bytes = Some(p.bytes);
        }
    }
    // Refuse required ends that lose original allocated extents or exceed the
    // profile's existing count/wire limits. No reference command runs on idle.
    expected.validate().is_ok()
}
pub(super) fn outcomes(
    initial: &lvm::State,
    current: &lvm::State,
    targets: &[super::Target],
) -> Vec<(bool, String)> {
    if initial.pv_capacity.is_none() {
        return vec![];
    }
    let mut rows = Vec::new();
    let mut intended = BTreeSet::new();
    for t in targets {
        for p in &t.pv_sizes {
            intended.insert(p.device.as_str());
            let d = &current.devices[&p.device];
            rows.push((d.pv_bytes==Some(p.bytes),format!("{}: recognized PV {} bytes; require {} bytes. Supplied device {} bytes; LV/filesystem sizes are separate.",p.device,d.pv_bytes.unwrap(),p.bytes,d.bytes)));
        }
        if t.preserve_volume {
            let before = initial.volume(&t.volume).unwrap().1;
            rows.push((current.volume(&t.volume).is_ok_and(|(_,v)|v==before),format!("{}: preserve the original LV allocation, filesystem capacity and files while changing only the required PV end",t.volume)));
        }
    }
    for (path, d) in &initial.devices {
        if !intended.contains(path.as_str()) {
            rows.push((current.devices.get(path)==Some(d),format!("Unrelated {path}: keep its supplied signature, recognized capacity, device size and group")));
        }
    }
    rows
}
