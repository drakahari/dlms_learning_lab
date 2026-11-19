//! Current279 unit metadata only. Scenario validation remains in pack loading;
//! the final index is validated against every supplied caller catalog.
use super::{Index, Unit};

fn fragment(text: &str, version_error: &str) -> Result<Vec<Unit>, String> {
    let index: Index = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if index.version != 1 {
        return Err(version_error.into());
    }
    Ok(index.units)
}

fn insert_after(
    units: &mut Vec<Unit>,
    extra: Vec<Unit>,
    anchors: &[(&str, &str)],
) -> Result<(), String> {
    let mut latest = 0;
    for (id, missing) in anchors {
        let at = units.iter().position(|u| u.id == *id).ok_or(*missing)?;
        latest = latest.max(at);
    }
    let at = latest + 1;
    units.splice(at..at, extra);
    Ok(())
}

pub(super) fn current279_index() -> Result<Index, String> {
    // Existing metadata-only base: magic first, then base54, then gateway unit.
    let mut index = super::accepted_index()?;
    index.units.extend(fragment(
        include_str!("../../scenarios/continuation-units-v1.json"),
        "Unsupported continuation unit index",
    )?);
    index.units.extend(fragment(
        include_str!("../../scenarios/syslog-units-v1.json"),
        "Unsupported syslog unit index",
    )?);
    index.units.extend(fragment(
        include_str!("../../scenarios/capacity-units-v1.json"),
        "Unsupported capacity unit index",
    )?);
    index.units.extend(fragment(
        include_str!("../../scenarios/repository-units-v1.json"),
        "Unsupported repository unit index",
    )?);
    index.units.extend(fragment(
        include_str!("../../scenarios/linux-host-units-v1.json"),
        "Unsupported host unit index",
    )?);
    index.units.extend(fragment(
        include_str!("../../scenarios/shell-debug-recovery-units-v1.json"),
        "Unsupported debugging/recovery unit index",
    )?);
    index.units.extend(fragment(
        include_str!("../../scenarios/linux-logging-units-v1.json"),
        "Unsupported Linux logging unit index",
    )?);
    index.units.extend(fragment(
        include_str!("../../scenarios/linux-startup-units-v1.json"),
        "Unsupported startup-policy unit index",
    )?);
    index.units.extend(fragment(
        include_str!("../../scenarios/qos-units-v1.json"),
        "Unsupported QoS unit index",
    )?);
    index.units.extend(fragment(
        include_str!("../../scenarios/backup-selection-units-v1.json"),
        "Unsupported backup-selection unit index",
    )?);
    index.units.extend(fragment(
        include_str!("../../scenarios/linux-dhcp-units-v1.json"),
        "Unsupported DHCP configuration unit index",
    )?);
    index.units.extend(fragment(
        include_str!("../../scenarios/linux-nm-units-v1.json"),
        "Unsupported persistent-networking unit index",
    )?);
    index.units.extend(fragment(
        include_str!("../../scenarios/removable-mount-units-v1.json"),
        "Unsupported removable-mount unit index",
    )?);
    index.units.extend(fragment(
        include_str!("../../scenarios/client-name-units-v1.json"),
        "Unsupported client-name unit index",
    )?);
    index.units.extend(fragment(
        include_str!("../../scenarios/resolver-search-units-v1.json"),
        "Unsupported resolver-search unit index",
    )?);
    index.units.extend(fragment(
        include_str!("../../scenarios/debian-contents-units-v1.json"),
        "Unsupported Debian Contents unit index",
    )?);
    index.units.extend(fragment(
        include_str!("../../scenarios/debian-sources-units-v1.json"),
        "Unsupported source-metadata unit index",
    )?);
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/shell-tracing-units-v1.json"),
            "Unsupported tracing unit index",
        )?,
        &[("shell-foundations", "Missing tracing prerequisite unit")],
    )?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/environment-units-v1.json"),
            "Unsupported environment unit index",
        )?,
        &[(
            "linux-file-foundations",
            "Missing environment prerequisite unit",
        )],
    )?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/user-mount-units-v1.json"),
            "Unsupported user-mount unit index",
        )?,
        &[(
            "linux-removable-startup",
            "Missing caller-mount prerequisite unit",
        )],
    )?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/debian-conflict-units-v1.json"),
            "Unsupported package-conflict unit index",
        )?,
        &[(
            "debian-package-administration",
            "Missing package-conflict prerequisite unit",
        )],
    )?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/linux-nm-dhcp-units-v1.json"),
            "Unsupported automatic Ethernet unit index",
        )?,
        &[(
            "linux-persistent-networking",
            "Missing automatic Ethernet prerequisite unit",
        )],
    )?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/fs-integrity-units-v1.json"),
            "Unsupported filesystem-consistency unit index",
        )?,
        &[
            (
                "linux-removable-callers",
                "Missing consistency mount prerequisite",
            ),
            (
                "linux-link-handover",
                "Missing consistency file-identity prerequisite",
            ),
        ],
    )?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/shell-startup-units-v1.json"),
            "Unsupported shell startup unit index",
        )?,
        &[(
            "linux-shell-environment",
            "Missing startup environment prerequisite",
        )],
    )?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/shell-file-lifecycle-units-v1.json"),
            "Unsupported file-lifecycle unit index",
        )?,
        &[
            (
                "shell-snapshot-rollout",
                "Missing file-lifecycle snapshot prerequisite",
            ),
            (
                "shell-tracing-diagnosis",
                "Missing file-lifecycle debugging prerequisite",
            ),
        ],
    )?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/debian-conffile-units-v1.json"),
            "Unsupported configuration-upgrade unit index",
        )?,
        &[(
            "debian-package-conflicts",
            "Missing configuration-upgrade prerequisite",
        )],
    )?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/raid1-units-v1.json"),
            "Unsupported mirror-recovery unit index",
        )?,
        &[
            (
                "linux-disk-preparation",
                "Missing disk preparation prerequisite",
            ),
            ("persistent-mounts", "Missing persistent-mount prerequisite"),
        ],
    )?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/shell-associative-units-v1.json"),
            "Unsupported named-map unit index",
        )?,
        &[
            ("shell-arrays", "Missing named-map array prerequisite"),
            ("shell-text", "Missing named-map text prerequisite"),
        ],
    )?;
    Ok(index)
}

/// Current metadata extension; old279 metadata and all original unit values stay intact.
pub(super) fn current281_index() -> Result<Index, String> {
    let mut index = current279_index()?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/shared-library-units-v1.json"),
            "Unsupported shared-library unit index",
        )?,
        &[
            ("linux-link-handover", "Missing library link prerequisite"),
            (
                "linux-shell-environment",
                "Missing library environment prerequisite",
            ),
        ],
    )?;
    Ok(index)
}

/// File-location practice follows both file operations and bounded search.
pub(super) fn current283_index() -> Result<Index, String> {
    let mut index = current281_index()?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/fhs-units-v1.json"),
            "Unsupported file-location unit index",
        )?,
        &[
            ("linux-file-foundations", "Missing file-operation prerequisite"),
            ("linux-file-search", "Missing file-search prerequisite"),
        ],
    )?;
    Ok(index)
}

/// Reconcile owned settings after recovery, file publication and named maps.
pub(super) fn current284_index() -> Result<Index, String> {
    let mut index = current283_index()?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/shell-managed-config-units-v1.json"),
            "Unsupported managed-configuration unit index",
        )?,
        &[
            ("shell-debug-recovery", "Missing recovery prerequisite"),
            ("shell-file-lifecycle", "Missing publication prerequisite"),
            ("shell-associative-maps", "Missing named-map prerequisite"),
        ],
    )?;
    Ok(index)
}

/// Numeric listener investigation follows process control and host networking.
pub(super) fn current285_index() -> Result<Index, String> {
    let mut index = current284_index()?;
    insert_after(
        &mut index.units,
        fragment(include_str!("../../scenarios/listener-audit-units-v1.json"), "Unsupported listener unit index")?,
        &[("linux-processes", "Missing process prerequisite"),
          ("linux-host-networking", "Missing host-network prerequisite")],
    )?;
    Ok(index)
}

/// Inherited-worker command search follows environment and file-location practice.
pub(super) fn current286_index() -> Result<Index, String> {
    let mut index = current285_index()?;
    insert_after(
        &mut index.units,
        fragment(include_str!("../../scenarios/path-lookup-units-v1.json"), "Unsupported command-search unit index")?,
        &[("linux-shell-environment", "Missing environment prerequisite"),
          ("linux-file-locations", "Missing file-location prerequisite")],
    )?;
    Ok(index)
}

/// Account-default maintenance follows explicit onboarding and file-location practice.
pub(super) fn current287_index() -> Result<Index, String> {
    let mut index = current286_index()?;
    insert_after(&mut index.units,
        fragment(include_str!("../../scenarios/account-skeleton-units-v1.json"), "Unsupported account-default unit index")?,
        &[("local-account-lifecycle", "Missing account-onboarding prerequisite"),
          ("linux-file-locations", "Missing file-location prerequisite")])?;
    Ok(index)
}

/// Eligibility diagnosis follows both ordered search and permission/traversal practice.
pub(super) fn current288_index() -> Result<Index,String> {
    let mut index=current287_index()?;
    insert_after(&mut index.units,
        fragment(include_str!("../../scenarios/command-location-units-v1.json"),"Unsupported command-location unit index")?,
        &[("linux-command-search","Missing ordered-search prerequisite"),
          ("linux-file-access","Missing permission prerequisite")])?;
    Ok(index)
}

/// Private indexed discovery follows live search and file-role handover practice.
pub(super) fn current289_index()->Result<Index,String>{
    let mut index=current288_index()?;
    insert_after(&mut index.units,
        fragment(include_str!("../../scenarios/file-index-units-v1.json"),"Unsupported private-index unit index")?,
        &[("linux-file-locations","Missing file-role prerequisite"),("linux-file-search","Missing live-search prerequisite")])?;
    Ok(index)
}

/// Support-artifact discovery follows command eligibility and current-file handover.
pub(super) fn current290_index()->Result<Index,String>{
    let mut index=current289_index()?;
    insert_after(&mut index.units,
        fragment(include_str!("../../scenarios/whereis-units-v1.json"),"Unsupported application-discovery unit index")?,
        &[("linux-command-location-diagnosis","Missing command-discovery prerequisite"),("linux-file-locations","Missing file-role prerequisite")])?;
    Ok(index)
}

/// Restricted-deletion practice builds on the existing access-policy handover.
pub(super) fn current292_index()->Result<Index,String>{
    let mut index=current290_index()?;
    insert_after(&mut index.units,
        fragment(include_str!("../../scenarios/sticky-directory-units-v1.json"),"Unsupported directory-policy unit index")?,
        &[("linux-access-policy","Missing ordinary directory-access prerequisite")])?;
    Ok(index)
}

/// Bounded indexed content recovery follows the managed-configuration project.
pub(super) fn current294_index()->Result<Index,String>{
    let mut index=current292_index()?;
    insert_after(&mut index.units,
        fragment(include_str!("../../scenarios/shell-content-backup-units-v1.json"),"Unsupported content-recovery unit index")?,
        &[("shell-managed-configuration","Missing managed-configuration prerequisite")])?;
    Ok(index)
}

/// TCP/UDP activities build on both routed return paths and directional ACLs.
pub(super) fn current296_index()->Result<Index,String>{
    let mut index=current294_index()?;
    insert_after(&mut index.units,fragment(include_str!("../../scenarios/transport-units-v1.json"),"Unsupported transport unit index")?,&[("ipv4-static-foundations","Missing return-path prerequisite"),("ipv4-access-policy","Missing ACL prerequisite")])?;
    Ok(index)
}

/// Complete tree handovers follow ordinary files, masks, access and search.
pub(super) fn current298_index()->Result<Index,String>{
    let mut index=current296_index()?;
    insert_after(&mut index.units,fragment(include_str!("../../scenarios/recursive-tree-units-v1.json"),"Unsupported recursive handover unit index")?,&[("linux-file-foundations","Missing ordinary file prerequisite"),("linux-file-search","Missing tree search prerequisite")])?;
    Ok(index)
}

/// Size-based evidence work follows complete handovers and bounded searches.
pub(super) fn current300_index()->Result<Index,String>{let mut index=current298_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/find-size-units-v1.json"),"Unsupported size-evidence unit index")?,&[("linux-recursive-handovers","Missing recursive handover prerequisite"),("linux-file-search","Missing bounded search prerequisite")])?;Ok(index)}

/// Supplied loadable drivers follow ordinary files and saved/current startup policy.
pub(super) fn current303_index()->Result<Index,String>{
    let mut index=current300_index()?;
    insert_after(&mut index.units,fragment(include_str!("../../scenarios/kernel-module-units-v1.json"),"Unsupported supplied-module unit index")?,&[("linux-file-foundations","Missing file inspection prerequisite"),("linux-startup-policy","Missing saved/current policy prerequisite")])?;
    Ok(index)
}

/// Exact saved caller names follow current/effective driver configuration.
pub(super) fn current304_index()->Result<Index,String>{let mut index=current303_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/kernel-module-names-units-v1.json"),"Unsupported named-module handover unit index")?,&[("linux-module-options","Missing saved/effective module prerequisite")])?;Ok(index)}

pub(super) fn current305_index()->Result<Index,String>{let mut index=current304_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/shell-log-triage-units-v1.json"),"Unsupported log-correlation unit index")?,&[("shell-associative-maps","Missing named-map prerequisite"),("shell-ere-evidence","Missing ERE prerequisite"),("shell-debug-recovery","Missing status-debugging prerequisite")])?;Ok(index)}

/// Metadata-preserving repair follows package investigation and access policy.
pub(super) fn current307_index()->Result<Index,String>{let mut index=current305_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/rpm-metadata-units-v1.json"),"Unsupported RPM recovery unit index")?,&[("rpm-package-investigation","Missing RPM investigation prerequisite"),("linux-access-policy","Missing access-policy prerequisite")])?;Ok(index)}

/// IPv6 is taught beside already working Linux IPv4 host state.
pub(super) fn current310_index()->Result<Index,String>{let mut index=current307_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/host-ipv6-units-v1.json"),"Unsupported dual-stack host unit index")?,&[("linux-host-networking","Missing Linux host prerequisite")])?;Ok(index)}

/// Prepared-space activation follows disk preparation and saved/live caller policy.
pub(super) fn current312_index()->Result<Index,String>{let mut index=current310_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/swap-policy-units-v1.json"),"Unsupported swap-policy unit index")?,&[("linux-disk-preparation","Missing disk preparation prerequisite"),("linux-removable-callers","Missing saved/live mount prerequisite")])?;Ok(index)}

/// New raw allocation follows the existing supplied-LVM growth prerequisite.
pub(super) fn current315_index()->Result<Index,String>{let mut index=current312_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/lvm-provisioning-units-v1.json"),"Unsupported linear provisioning unit index")?,&[("lvm-volume-growth","Missing LVM growth prerequisite")])?;Ok(index)}

pub(super) fn current317_index()->Result<Index,String>{let mut index=current315_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/lvm-retirement-units-v1.json"),"Unsupported raw retirement unit index")?,&[("lvm-linear-provisioning","Missing linear provisioning prerequisite")])?;Ok(index)}

pub(super) fn current319_index()->Result<Index,String>{let mut index=current317_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/lvm-activation-units-v1.json"),"Unsupported LVM availability unit index")?,&[("lvm-raw-retirement","Missing raw retirement prerequisite")])?;Ok(index)}

pub(super) fn current321_index()->Result<Index,String>{let mut index=current319_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/lvm-rename-units-v1.json"),"Unsupported LVM storage names unit index")?,&[("lvm-availability","Missing availability prerequisite")])?;Ok(index)}

pub(super) fn current323_index()->Result<Index,String>{let mut index=current321_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/lvm-pv-capacity-units-v1.json"),"Unsupported PV capacity unit index")?,&[("lvm-storage-names","Missing storage names prerequisite")])?;Ok(index)}

pub(super) fn current326_index()->Result<Index,String>{let mut index=current323_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/mbr-layout-units-v1.json"),"Unsupported MBR layout unit index")?,&[("linux-disk-preparation","Missing disk preparation prerequisite")])?;Ok(index)}

/// Classic raw-record COW planning follows the accepted LVM capacity layers.
pub(super) fn current329_index()->Result<Index,String>{let mut index=current326_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/lvm-cow-units-v1.json"),"Unsupported COW recovery unit index")?,&[("lvm-pv-capacity","Missing PV capacity prerequisite")])?;Ok(index)}

/// Saved application references follow existing-object name management.
pub(super) fn current331_index()->Result<Index,String>{let mut index=current329_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/lvm-reference-units-v1.json"),"Unsupported LVM application-reference unit index")?,&[("lvm-storage-names","Missing storage name prerequisite")])?;Ok(index)}

pub(super) fn current333_index()->Result<Index,String>{let mut index=current331_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/lvm-config-units-v1.json"),"Unsupported LVM configuration unit index")?,&[("lvm-application-references","Missing LVM application-reference prerequisite")])?;Ok(index)}

pub(super) fn current335_index()->Result<Index,String>{let mut index=current333_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/lvm-mount-units-v1.json"),"Unsupported LVM filesystem mount unit index")?,&[("lvm-device-selection","Missing LVM device selection prerequisite")])?;Ok(index)}

/// AutoFS follows explicit saved versus mounted filesystem practice.
pub(super) fn current337_index()->Result<Index,String>{let mut index=current335_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/autofs-units-v1.json"),"Unsupported local AutoFS unit index")?,&[("lvm-filesystem-mounting","Missing saved/current mount prerequisite")])?;Ok(index)}

pub(super) fn current338_index()->Result<Index,String>{let mut index=current337_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/raid-assembly-units-v1.json"),"Unsupported clean RAID1 assembly unit index")?,&[("linux-raid1-recovery","Missing RAID1 recovery prerequisite")])?;Ok(index)}

pub(super) fn current340_index()->Result<Index,String>{let mut index=current338_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/linux-vlan-units-v1.json"),"Unsupported Linux VLAN unit index")?,&[("linux-persistent-networking","Missing saved profile prerequisite")])?;Ok(index)}

pub(super) fn current341_index()->Result<Index,String>{let mut index=current340_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/bond-vlan-units-v1.json"),"Unsupported bonded VLAN unit index")?,&[("linux-vlan-profiles","Missing VLAN profile prerequisite")])?;Ok(index)}

pub(super) fn current342_index()->Result<Index,String>{let mut index=current341_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/bridge-lan-units-v1.json"),"Unsupported untagged bridge unit index")?,&[("linux-bonded-vlan","Missing controller/port prerequisite")])?;Ok(index)}

/// PAM control-flow follows local identity administration; prerequisites stay advisory.
pub(super) fn current343_index()->Result<Index,String>{let mut index=current342_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/pam-auth-units-v1.json"),"Unsupported authentication-stack unit index")?,&[("local-account-lifecycle","Missing local account prerequisite")])?;Ok(index)}

/// Aging follows local records and supplies prerequisites before independent policy repair.
pub(super) fn current346_index()->Result<Index,String>{let mut index=current343_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/aging-units-v1.json"),"Unsupported password-aging unit index")?,&[("local-account-lifecycle","Missing local account prerequisite")])?;Ok(index)}

/// Secure access-port policy follows the unchanged Ethernet-learning handover.
pub(super) fn current349_index()->Result<Index,String>{let mut index=current346_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/port-security-units-v1.json"),"Unsupported secure-port unit index")?,&[("ethernet-learning","Missing Ethernet-learning prerequisite")])?;Ok(index)}

/// DHCP inspection follows source admission and the earlier initial-exchange unit.
pub(super) fn current351_index()->Result<Index,String>{let mut index=current349_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/dhcp-snooping-units-v1.json"),"Unsupported DHCP-snooping unit index")?,&[("secure-access-ports","Missing secure access-port prerequisite")])?;Ok(index)}

pub(super) fn current354_index()->Result<Index,String>{let mut index=current351_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/arp-inspection-units-v1.json"),"Unsupported ARP-inspection unit index")?,&[("dhcp-snooping-foundations","Missing DHCP-snooping prerequisite")])?;Ok(index)}

pub(super) fn current356_index()->Result<Index,String>{let mut index=current354_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/management-policy-units-v1.json"),"Unsupported management-policy unit index")?,&[("arp-inspection-foundations","Missing preceding security unit")])?;Ok(index)}

pub(super) fn current358_index()->Result<Index,String>{let mut index=current356_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/local-access-units-v1.json"),"Unsupported local-access unit index")?,&[("management-policy-foundations","Missing management-policy prerequisite")])?;Ok(index)}

pub(super) fn current360_index()->Result<Index,String>{let mut index=current358_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/aaa-access-units-v1.json"),"Unsupported AAA decisions unit index")?,&[("local-access-foundations","Missing local-access prerequisite")])?;Ok(index)}

/// Directory lookup follows local Linux client foundations; authentication remains separate.
pub(super) fn current362_index()->Result<Index,String>{let mut index=current360_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/ldap-client-units-v1.json"),"Unsupported directory-client unit index")?,&[("linux-authentication-stacks","Missing preceding Linux administration unit")])?;Ok(index)}

pub(super) fn current364_index()->Result<Index,String>{let mut index=current362_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/ldap-change-units-v1.json"),"Unsupported directory-change unit index")?,&[("ldap-client-queries","Missing preceding LDAP client unit")])?;Ok(index)}

pub(super) fn current366_index()->Result<Index,String>{let mut index=current364_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/ldap-utility-units-v1.json"),"Unsupported directory utility unit index")?,&[("ldap-directory-changes","Missing preceding LDIF data unit")])?;Ok(index)}

pub(super) fn current367_index()->Result<Index,String>{let mut index=current366_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/ldap-retirement-units-v1.json"),"Unsupported directory retirement unit index")?,&[("ldap-utility-work","Missing preceding utility unit")])?;Ok(index)}

pub(super) fn current368_index()->Result<Index,String>{let mut index=current367_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/ldap-password-units-v1.json"),"Unsupported protected password unit index")?,&[("ldap-account-retirement","Missing retirement prerequisite")])?;Ok(index)}

pub(super) fn current369_index()->Result<Index,String>{let mut index=current368_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/ldap-access-units-v1.json"),"Unsupported LDAP access-policy unit index")?,&[("ldap-protected-passwords","Missing protected identity prerequisite")])?;Ok(index)}

pub(super) fn current370_index()->Result<Index,String>{let mut index=current369_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/ldap-config-units-v1.json"),"Unsupported directory-backed configuration unit index")?,&[("ldap-access-policy","Missing static policy prerequisite")])?;Ok(index)}

pub(super) fn current371_index()->Result<Index,String>{let mut index=current370_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/ldap-schema-units-v1.json"),"Unsupported custom schema unit index")?,&[("ldap-config-deployment","Missing configuration prerequisite")])?;Ok(index)}

pub(super) fn current372_index()->Result<Index,String>{let mut index=current371_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/ldap-schema-query-units-v1.json"),"Unsupported schema-consumer unit index")?,&[("ldap-schema-extension","Missing custom schema prerequisite")])?;Ok(index)}

pub(super) fn current373_index()->Result<Index,String>{let mut index=current372_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/ipv6-interface-id-units-v1.json"),"Unsupported IPv6 interface-ID unit index")?,&[("ipv6-static-foundations","Missing IPv6 workstation prerequisite")])?;Ok(index)}

pub(super) fn current376_index()->Result<Index,String>{let mut index=current373_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/boot-entry-units-v1.json"),"Unsupported boot-entry unit index")?,&[("linux-module-foundations","Missing supplied-driver prerequisite"),("linux-startup-policy","Missing saved/current startup prerequisite")])?;Ok(index)}

pub(super) fn current378_index()->Result<Index,String>{let mut index=current376_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/grub-menu-units-v1.json"),"Unsupported literal GRUB menu unit index")?,&[("linux-boot-entry-recovery","Missing early-boot handoff prerequisite")])?;Ok(index)}

pub(super) fn current380_index()->Result<Index,String>{let mut index=current378_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/grub-settings-units-v1.json"),"Unsupported persistent GRUB unit index")?,&[("linux-grub-menu-recovery","Missing GRUB menu prerequisite")])?;Ok(index)}

pub(super) fn current381_index()->Result<Index,String>{let mut index=current380_index()?;insert_after(&mut index.units,fragment(include_str!("../../scenarios/bios-loader-units-v1.json"),"Unsupported BIOS loader unit index")?,&[("linux-grub-persistent-configuration","Missing persistent GRUB prerequisite")])?;Ok(index)}


pub(super) fn current382_index() -> Result<Index, String> {
    let mut index = current381_index()?;
    insert_after(
        &mut index.units,
        fragment(include_str!("../../scenarios/efi-loader-units-v1.json"), "Unsupported EFI loader unit index")?,
        &[("linux-bios-loader-recovery", "Missing BIOS loader prerequisite")],
    )?;
    Ok(index)
}


pub(super) fn current383_index() -> Result<Index, String> {
    let mut index = current382_index()?;
    insert_after(
        &mut index.units,
        fragment(include_str!("../../scenarios/removable-efi-units-v1.json"), "Unsupported removable EFI unit index")?,
        &[("linux-unit-manager-foundations", "Missing LPIC-2 service foundations")],
    )?;
    Ok(index)
}


pub(super) fn current384_index() -> Result<Index, String> {
    let mut index = current383_index()?;
    insert_after(
        &mut index.units,
        fragment(include_str!("../../scenarios/grub-once-units-v1.json"), "Unsupported temporary GRUB unit index")?,
        &[("linux-removable-efi-recovery", "Missing removable EFI prerequisite")],
    )?;
    Ok(index)
}


pub(super) fn current385_index() -> Result<Index, String> {
    let mut index = current384_index()?;
    insert_after(
        &mut index.units,
        fragment(include_str!("../../scenarios/rescue-startup-units-v1.json"), "Unsupported rescue startup unit index")?,
        &[("linux-temporary-grub-recovery", "Missing temporary GRUB prerequisite")],
    )?;
    Ok(index)
}


pub(super) fn current386_index() -> Result<Index, String> {
    let mut index = current385_index()?;
    insert_after(
        &mut index.units,
        fragment(include_str!("../../scenarios/chrony-source-units-v1.json"), "Unsupported time-source unit index")?,
        &[
            ("linux-host-networking", "Missing Linux networking prerequisite"),
            ("linux-startup-policy", "Missing saved startup settings prerequisite"),
        ],
    )?;
    Ok(index)
}


pub(super) fn current387_index() -> Result<Index, String> {
    let mut index = current386_index()?;
    insert_after(
        &mut index.units,
        fragment(include_str!("../../scenarios/copied-journal-units-v1.json"), "Unsupported copied-journal unit index")?,
        &[("linux-syslog-routing", "Missing Linux logging prerequisite")],
    )?;
    Ok(index)
}


pub(super) fn current390_index() -> Result<Index, String> {
    let mut index = current387_index()?;
    insert_after(
        &mut index.units,
        fragment(include_str!("../../scenarios/local-mail-units-v1.json"), "Unsupported local-mail unit index")?,
        &[
            ("linux-file-foundations", "Missing virtual file prerequisite"),
            ("local-account-lifecycle", "Missing local-user prerequisite"),
        ],
    )?;
    Ok(index)
}


pub(super) fn current392_index() -> Result<Index, String> {
    let mut index = current390_index()?;
    insert_after(&mut index.units,
        fragment(include_str!("../../scenarios/device-path-units-v1.json"), "Unsupported device-path unit index")?,
        &[("ipv4-foundations", "Missing IPv4 addressing prerequisite")],
    )?;
    Ok(index)
}

pub(super) fn current394_index() -> Result<Index, String> {
    let mut index = current392_index()?;
    insert_after(&mut index.units,
        fragment(include_str!("../../scenarios/json-inventory-units-v1.json"), "Unsupported JSON inventory unit index")?,
        &[("vlan-data-foundations", "Missing VLAN handover prerequisite")],
    )?;
    Ok(index)
}

pub(super) fn current396_index()->Result<Index,String>{
 let mut index=current394_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/layout-planning-units-v1.json"),"Unsupported layout-planning unit index")?,&[("linux-disk-preparation","Missing disk-preparation prerequisite")])?;
 Ok(index)
}

pub(super) fn current399_index()->Result<Index,String>{
 let mut index=current396_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/measurement-report-units-v1.json"),"Unsupported measurement-report unit index")?,&[("shell-log-correlation","Missing report-analysis prerequisite")])?;
 Ok(index)
}

pub(super) fn current402_index()->Result<Index,String>{
 let mut index=current399_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/resource-observation-units-v1.json"),"Unsupported resource observation unit index")?,&[("linux-capacity-handover","Missing existing resource investigation prerequisite")])?;
 Ok(index)
}

pub(super) fn current404_index()->Result<Index,String>{
 let mut index=current402_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/capacity-planning-units-v1.json"),"Unsupported capacity-planning unit index")?,&[("linux-resource-observations","Missing measurement prerequisite")])?;
 Ok(index)
}


pub(super) fn current406_index()->Result<Index,String>{
 let mut index=current404_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/maintenance-notice-units-v1.json"),"Unsupported maintenance notice unit index")?,&[("linux-capacity-planning","Missing capacity-planning maintenance prerequisite")])?;
 Ok(index)
}

pub(super) fn current409_index()->Result<Index,String>{
 let mut index=current406_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/source-installation-units-v1.json"),"Unsupported source-installation unit index")?,&[("linux-maintenance-notices","Missing maintenance sequence predecessor")])?;
 Ok(index)
}

pub(super) fn current412_index()->Result<Index,String>{
 let mut index=current409_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/dns-access-units-v1.json"),"Unsupported DNS access unit index")?,&[("dns-reverse","Missing preceding DNS publication units")])?;
 Ok(index)
}

pub(super) fn current415_index()->Result<Index,String>{
 let mut index=current412_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/print-queue-units-v1.json"),"Unsupported print queue unit index")?,&[("linux-local-mail-routing","Missing preceding local-mail unit")])?;
 Ok(index)
}

pub(super) fn current417_index()->Result<Index,String>{
 let mut index=current415_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/localization-units-v1.json"),"Unsupported locale and encoding unit index")?,&[("linux-cron-foundations","Missing preceding recurring-job unit")])?;
 Ok(index)
}

pub(super) fn current419_index()->Result<Index,String>{
 let mut index=current417_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/timezone-units-v1.json"),"Unsupported time-zone unit index")?,&[("linux-localization-reports","Missing preceding locale and encoding unit")])?;
 Ok(index)
}

pub(super) fn current421_index()->Result<Index,String>{
 let mut index=current419_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/wireless-channel-units-v1.json"),"Unsupported wireless channel unit index")?,&[("ccna-wired-device-paths","Missing preceding wired-device unit")])?;
 Ok(index)
}

pub(super) fn current424_index()->Result<Index,String>{
 let mut index=current421_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/wlan-profile-units-v1.json"),"Unsupported WLAN profile unit index")?,&[("ccna-wireless-channel-foundations","Missing preceding wireless channel unit")])?;
 Ok(index)
}

pub(super) fn current426_index() -> Result<Index, String> {
 let mut index=current424_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/wlan-path-units-v1.json"),"Unsupported AP-mode/forwarding unit index")?,&[("ccna-wlan-personal-security","Missing preceding WLAN profile unit")])?;
 Ok(index)
}

pub(super) fn current428_index()->Result<Index,String>{
 let mut index=current426_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/snmp-monitor-units-v1.json"),"Unsupported SNMP monitoring unit index")?,&[("management-policy-foundations","Missing preceding remote-management unit")])?;
 Ok(index)
}

pub(super) fn current430_index()->Result<Index,String>{
 let mut index=current428_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/proxy-policy-units-v1.json"),"Unsupported proxy-policy unit index")?,&[("static-web-services","Missing static-web prerequisite"),("linux-host-networking","Missing host-network prerequisite")])?;
 Ok(index)
}

pub(super) fn current432_index()->Result<Index,String>{
 let mut index=current430_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/http-cache-units-v1.json"),"Unsupported shared-cache unit index")?,&[("linux-proxy-access-policy","Missing proxy-policy prerequisite")])?;
 Ok(index)
}


pub(super) fn current434_index()->Result<Index,String>{
 let mut index=current432_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/nginx-routing-units-v1.json"),"Unsupported reverse-proxy routing unit index")?,&[("linux-shared-http-cache","Missing preceding shared-cache unit"),("static-web-services","Missing static-web prerequisite")])?;
 Ok(index)
}

pub(super) fn current436_index()->Result<Index,String>{
 let mut index=current434_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/service-name-units-v1.json"),"Unsupported service-name unit index")?,&[("linux-host-networking","Missing host-network prerequisite")])?;
 Ok(index)
}

pub(super) fn current437_index()->Result<Index,String>{
 let mut index=current436_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/mixed-ip-units-v1.json"),"Unsupported mixed-IP unit index")?,&[("linux-host-networking","Missing Linux host-network prerequisite")])?;
 Ok(index)
}

pub(super) fn current438_index()->Result<Index,String>{
 let mut index=current437_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/protocol-evidence-units-v1.json"),"Unsupported protocol-evidence unit index")?,&[("linux-service-names","Missing service-name prerequisite"),("linux-mixed-ip-foundations","Missing mixed-IP prerequisite")])?;
 Ok(index)
}

pub(super) fn current439_index()->Result<Index,String>{
 let mut index=current438_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/echo-diagnosis-units-v1.json"),"Unsupported Echo diagnosis unit index")?,&[("linux-protocol-evidence","Missing protocol-evidence prerequisite"),("linux-listener-audit","Missing listener-audit prerequisite")])?;
 Ok(index)
}

pub(super) fn current441_index()->Result<Index,String>{
 let mut index=current439_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/rest-controller-units-v1.json"),"Unsupported REST unit index")?,&[("structured-network-inventory","Missing JSON inventory prerequisite")])?;
 Ok(index)
}

pub(super) fn current442_index()->Result<Index,String>{
 let mut index=current441_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/client-diagnostic-units-v1.json"),"Unsupported client-diagnostic unit index")?,&[("linux-resolver-search","Missing source/search prerequisites")])?;
 Ok(index)
}

pub(super) fn current445_index()->Result<Index,String>{
 let mut index=current442_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/ssh-access-units-v1.json"),"Unsupported SSH administration unit index")?,&[("linux-host-networking","Missing host-network prerequisite"),("linux-access-policy","Missing access-policy prerequisite")])?;
 Ok(index)
}

pub(super) fn current448_index()->Result<Index,String>{
 let mut index=current445_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/modal-editing-units-v1.json"),"Unsupported modal editing unit index")?,&[("linux-file-foundations","Missing file foundations")])?;
 Ok(index)
}

pub(super) fn current449_index()->Result<Index,String>{
 let mut index=current448_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/editor-handover-units-v1.json"),"Unsupported standard-editor handover unit index")?,&[("linux-shell-environment","Missing shell environment prerequisite")])?;
 Ok(index)
}

pub(super) fn current452_index()->Result<Index,String>{
 let mut index=current449_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/network-transfers-units-v1.json"),"Unsupported network file-transfer unit index")?,&[("ccna-transport-conversations","Missing TCP/UDP and ACL prerequisites")])?;
 Ok(index)
}

pub(super) fn current454_index()->Result<Index,String>{
 let mut index=current452_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/campus-architecture-units-v1.json"),"Unsupported campus architecture unit index")?,&[("ccna-wired-device-paths","Missing wired-device prerequisite")])?;
 Ok(index)
}

pub(super) fn current455_index()->Result<Index,String>{
 let mut index=current454_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/leaf-spine-units-v1.json"),"Unsupported leaf/spine unit index")?,&[("ccna-campus-architectures","Missing campus architecture prerequisite")])?;
 Ok(index)
}

pub(super) fn current458_index()->Result<Index,String>{
 let mut index=current455_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/samba-standalone-units-v1.json"),"Unsupported standalone-sharing unit index")?,&[("nfs-network-file-sharing","Missing network file-sharing prerequisite")])?;
 Ok(index)
}

pub(super) fn current461_index()->Result<Index,String>{
 let mut index=current458_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/virtual-isolation-units-v1.json"),"Unsupported virtual-isolation unit index")?,&[("ccna-leaf-spine-fabrics","Missing physical-architecture anchor")])?;
 Ok(index)
}

pub(super) fn current464_index()->Result<Index,String>{
 let mut index=current461_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/guest-driver-units-v1.json"),"Unsupported guest image unit index")?,&[("linux-module-foundations","Missing module foundation"),("rpm-package-investigation","Missing package investigation foundation")])?;
 Ok(index)
}

pub(super) fn current467_index()->Result<Index,String>{
 let mut index=current464_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/smtp-routing-units-v1.json"),"Unsupported SMTP unit index")?,&[("linux-local-mail-routing","Missing local-mail foundation"),("linux-host-networking","Missing host-network foundation")])?;
 Ok(index)
}

pub(super) fn current470_index()->Result<Index,String>{
 let mut index=current467_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/sieve-filter-units-v1.json"),"Unsupported Sieve unit index")?,&[("linux-local-mail-routing","Missing local-mail foundation"),("linux-smtp-routing","Missing SMTP foundation")])?;
 Ok(index)
}

pub(super) fn current472_index()->Result<Index,String>{
 let mut index=current470_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/sieve-address-units-v1.json"),"Unsupported Sieve address unit index")?,&[("linux-sieve-filters","Missing Sieve filtering foundation")])?;
 Ok(index)
}

pub(super) fn current475_index()->Result<Index,String>{
 let mut index=current472_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/sieve-delivery-units-v1.json"),"Unsupported Sieve delivery unit index")?,&[("linux-sieve-address-routing","Missing structured-address foundation")])?;
 Ok(index)
}

pub(super) fn current477_index()->Result<Index,String>{
 let mut index=current475_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/sieve-vacation-units-v1.json"),"Unsupported vacation unit index")?,&[("linux-sieve-envelope-delivery","Missing delivery foundation")])?;
 Ok(index)
}

pub(super) fn current478_index()->Result<Index,String>{
 let mut index=current477_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/procmail-migration-units-v1.json"),"Unsupported legacy-filter migration unit index")?,&[("linux-sieve-filters","Missing Sieve filtering foundation")])?;
 Ok(index)
}

pub(super) fn current481_index()->Result<Index,String>{
 let mut index=current478_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/mailbox-access-units-v1.json"),"Unsupported mailbox-access unit index")?,&[("linux-smtp-routing","Missing SMTP foundation"),("linux-host-networking","Missing host-networking foundation")])?;
 Ok(index)
}

pub(super) fn current482_index()->Result<Index,String>{
 let mut index=current481_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/mailbox-administration-units-v1.json"),"Unsupported mail-administration unit index")?,&[("linux-mailbox-access","Missing protected-mailbox foundation")])?;
 Ok(index)
}

pub(super) fn current485_index()->Result<Index,String>{
 let mut index=current482_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/linux-router-units-v1.json"),"Unsupported Linux-router unit index")?,&[("ssh-administration","Missing SSH administration foundation"),("linux-host-networking","Missing host-networking foundation")])?;
 Ok(index)
}

pub(super) fn current487_index()->Result<Index,String>{
 let mut index=current485_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/linux-flow-units-v1.json"),"Unsupported Linux flow unit index")?,&[("linux-router-policies","Missing router policy foundation")])?;
 Ok(index)
}

pub(super) fn current489_index()->Result<Index,String>{
 let mut index=current487_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/linux-v6-units-v1.json"),"Unsupported IPv6-router unit index")?,&[("linux-host-ipv6","Missing host IPv6 foundation"),("linux-connection-nat","Missing tracked-flow foundation")])?;
 Ok(index)
}

pub(super) fn current490_index()->Result<Index,String>{
 let mut index=current489_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/linux-services-router-units-v1.json"),"Unsupported services/router unit index")?,&[("linux-service-names","Missing service-name foundation"),("linux-ipv6-router-policy","Missing IPv6-router foundation")])?;
 Ok(index)
}

pub(super) fn current493_index()->Result<Index,String>{
 let mut index=current490_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/linux-v6-admin-units-v1.json"),"Unsupported IPv6 administration unit index")?,&[("linux-host-ipv6","Missing IPv6 host foundation"),("linux-router-policies","Missing Linux router foundation")])?;
 Ok(index)
}

pub(super) fn current494_index()->Result<Index,String>{
 let mut index=current493_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/linux-v6-startup-units-v1.json"),"Unsupported IPv6 filter-startup unit index")?,&[("linux-ipv6-router-configuration","Missing IPv6 configuration foundation"),("linux-ipv6-router-policy","Missing tracked IPv6 policy foundation")])?;
 Ok(index)
}

pub(super) fn current497_index()->Result<Index,String>{
 let mut index=current494_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/ftp-identity-units-v1.json"),"Unsupported FTP identity unit index")?,&[("samba-standalone-shares","Missing shared-identity foundation"),("linux-unit-manager-foundations","Missing service foundation")])?;
 Ok(index)
}

pub(super) fn current499_index()->Result<Index,String>{
 let mut index=current497_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/service-exposure-units-v1.json"),"Unsupported service-exposure unit index")?,&[("linux-listener-audit","Missing listener investigation foundation"),("linux-router-policies","Missing host-filter foundation")])?;
 Ok(index)
}

pub(super) fn current501_index()->Result<Index,String>{
 let mut index=current499_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/vpn-point-units-v1.json"),"Unsupported VPN unit index")?,&[("linux-service-exposure","Missing service-exposure foundation"),("linux-router-policies","Missing router-policy foundation")])?;
 Ok(index)
}

pub(super) fn current503_index()->Result<Index,String>{
 let mut index=current501_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/advisory-patch-units-v1.json"),"Unsupported advisory-patching unit index")?,&[("rpm-payload-recovery","Missing metadata/configuration foundation"),("rpm-package-investigation","Missing package investigation foundation")])?;
 Ok(index)
}


pub(super) fn current505_index()->Result<Index,String>{
 let mut index=current503_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/auth-filter-units-v1.json"),"Unsupported authentication-filter unit index")?,&[("linux-service-exposure","Missing listener/filter investigation foundation"),("linux-advisory-patching","Missing security-maintenance foundation")])?;
 Ok(index)
}


pub(super) fn current507_index()->Result<Index,String>{
 let mut index=current505_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/ids-rule-units-v1.json"),"Unsupported IDS-rule unit index")?,&[("linux-auth-log-filters","Missing supplied-log investigation foundation"),("linux-service-exposure","Missing endpoint and service-port foundation")])?;
 Ok(index)
}


pub(super) fn current509_index()->Result<Index,String>{
 let mut index=current507_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/poe-units-v1.json"),"Unsupported PoE power-plan unit index")?,&[("ccna-wired-device-paths","Missing wired-device foundation"),("physical-uplink-diagnosis","Missing physical-interface foundation")])?;
 Ok(index)
}

pub(super) fn current511_index()->Result<Index,String>{
 let mut index=current509_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/kernel-build-units-v1.json"),"Unsupported kernel-build unit index")?,&[("linux-source-installation","Missing supplied-source foundation"),("linux-boot-entry-recovery","Missing boot-entry foundation"),("linux-module-options","Missing module foundation")])?;
 Ok(index)
}

pub(super) fn current513_index()->Result<Index,String>{
 let mut index=current511_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/dkms-units-v1.json"),"Unsupported DKMS unit index")?,&[("linux-kernel-build-staging","Missing kernel-build foundation"),("linux-module-options","Missing module foundation")])?;
 Ok(index)
}

pub(super) fn current515_index()->Result<Index,String>{
 let mut index=current513_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/initramfs-units-v1.json"),"Unsupported early-userspace unit index")?,&[("linux-kernel-build-staging","Missing kernel-build foundation"),("linux-boot-entry-recovery","Missing boot-entry foundation"),("linux-module-options","Missing module foundation")])?;
 Ok(index)
}

pub(super) fn current517_index()->Result<Index,String>{
 let mut index=current515_index()?;
 insert_after(&mut index.units,fragment(include_str!("../../scenarios/pxe-units-v1.json"),"Unsupported network boot unit index")?,&[("linux-dhcp-configuration","Missing DHCP foundation"),("ccna-file-transfers","Missing file transfer foundation"),("linux-initramfs-recovery","Missing early-userspace foundation")])?;
 Ok(index)
}

pub(super) fn current519_index() -> Result<Index, String> {
    let mut index = current517_index()?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/iscsi-units-v1.json"),
            "Unsupported iSCSI unit index",
        )?,
        &[("persistent-mounts", "Missing stable storage identity foundation")],
    )?;
    Ok(index)
}

pub(super) fn current521_index() -> Result<Index, String> {
    let mut index = current519_index()?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/ata-smart-units-v1.json"),
            "Unsupported ATA SMART unit index",
        )?,
        &[("persistent-mounts", "Missing stable storage identity foundation"), ("linux-iscsi-initiator", "Missing previous storage unit")],
    )?;
    Ok(index)
}

pub(super) fn current523_index() -> Result<Index, String> {
    let mut index = current521_index()?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/apache-https-units-v1.json"),
            "Unsupported Apache HTTPS unit index",
        )?,
        &[("static-web-services", "Missing Apache virtual-host foundation")],
    )?;
    Ok(index)
}

pub(super) fn current526_index() -> Result<Index, String> {
    let mut index = current523_index()?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/desktop-profile-units-v1.json"),
            "Unsupported workstation unit index",
        )?,
        &[("linux-file-foundations", "Missing Linux file foundations")],
    )?;
    Ok(index)
}

pub(super) fn current528_index() -> Result<Index, String> {
    let mut index = current526_index()?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/security-investigation-units-v1.json"),
            "Unsupported security investigation unit index",
        )?,
        &[("management-policy-foundations", "Missing remote management foundation"),
          ("ccna-syslog-policy", "Missing Syslog evidence foundation")],
    )?;
    Ok(index)
}

pub(super) fn current530_index() -> Result<Index, String> {
    let mut index = current528_index()?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/credentials-vpn-units-v1.json"),
            "Unsupported credentials and VPN unit index",
        )?,
        &[("aaa-decisions", "Missing identity and authorization foundation"),
          ("ccna-security-investigation", "Missing security investigation foundation")],
    )?;
    Ok(index)
}

pub(super) fn current532_index() -> Result<Index, String> {
    let mut index = current530_index()?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/staged-network-units-v1.json"),
            "Unsupported staged network changes unit index",
        )?,
        &[("rest-network-changes", "Missing REST foundations"),
          ("ccna-credentials-and-vpn", "Missing CCNA credential and VPN unit")],
    )?;
    Ok(index)
}

pub(super) fn current534_index() -> Result<Index, String> {
    let mut index = current532_index()?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/network-evidence-review-units-v1.json"),
            "Unsupported network evidence review unit index",
        )?,
        &[("rest-network-changes", "Missing REST foundations"),
          ("ccna-staged-network-changes", "Missing staged network changes unit")],
    )?;
    Ok(index)
}

pub(super) fn current536_index() -> Result<Index, String> {
    let mut index = current534_index()?;
    insert_after(
        &mut index.units,
        fragment(
            include_str!("../../scenarios/site-connectivity-units-v1.json"),
            "Unsupported site connectivity unit index",
        )?,
        &[("ccna-campus-architectures", "Missing campus architecture prerequisite")],
    )?;
    Ok(index)
}

pub(super) fn current538_index() -> Result<Index, String> {
    let mut index = current536_index()?;
    let additions = fragment(
        include_str!("../../scenarios/host-and-dns-security-units-v1.json"),
        "Unsupported Linux host and DNS security unit index",
    )?;
    let [retirement, dns]: [Unit; 2] = additions.try_into()
        .map_err(|_| "Expected separate service retirement and DNS transfer units")?;
    insert_after(&mut index.units, vec![retirement],
        &[("linux-listener-audit", "Missing listener audit prerequisite")])?;
    insert_after(&mut index.units, vec![dns],
        &[("linux-dns-access", "Missing DNS access prerequisite")])?;
    Ok(index)
}

pub(super) fn current540_index() -> Result<Index, String> {
    let mut index = current538_index()?;
    let additions = fragment(
        include_str!("../../scenarios/ccna-vlan-gateway-units-v1.json"),
        "Unsupported CCNA default VLAN and gateway unit index",
    )?;
    let [gateways]: [Unit; 1] = additions.try_into()
        .map_err(|_| "Expected one default VLAN and gateway unit")?;
    insert_after(&mut index.units, vec![gateways],
        &[ ("ipv4-static-foundations", "Missing IPv4 return-path prerequisite"),
           ("vlan-data-foundations", "Missing VLAN trunk prerequisite")])?;
    Ok(index)
}

pub(super) fn current541_index() -> Result<Index, String> {
    let mut index = current540_index()?;
    let additions = fragment(
        include_str!("../../scenarios/nfs-export-operations-units-v1.json"),
        "Unsupported NFS export operations unit index",
    )?;
    let [retirement]: [Unit; 1] = additions.try_into()
        .map_err(|_| "Expected the NFS export operations unit")?;
    insert_after(&mut index.units, vec![retirement],
        &[("nfs-network-file-sharing", "Missing NFS handover prerequisite")])?;
    Ok(index)
}

pub(super) fn current542_index() -> Result<Index, String> {
    let mut index = current541_index()?;
    let additions = fragment(
        include_str!("../../scenarios/ccna-route-reading-units-v1.json"),
        "Unsupported installed route reading unit index",
    )?;
    let [routes]: [Unit; 1] = additions.try_into()
        .map_err(|_| "Expected one installed route reading unit")?;
    insert_after(&mut index.units, vec![routes],
        &[("ospf-p2p-foundations", "Missing point-to-point OSPF prerequisite")])?;
    Ok(index)
}

pub(super) fn current543_index() -> Result<Index, String> {
    let mut index = current542_index()?;
    let additions = fragment(
        include_str!("../../scenarios/ccna-ospf-broadcast-units-v1.json"),
        "Unsupported broadcast designated roles unit index",
    )?;
    let [broadcast]: [Unit; 1] = additions.try_into()
        .map_err(|_| "Expected one broadcast designated roles unit")?;
    insert_after(&mut index.units, vec![broadcast],
        &[("ipv4-installed-route-reading", "Missing installed route reading prerequisite")])?;
    Ok(index)
}
