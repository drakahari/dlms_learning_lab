//! Bounded source-held FAT32 volume and case-folded EFI paths.
//! This is a typed layout profile, not a FAT formatter or on-disk parser.
use serde::{Deserialize, Serialize};

pub const ESP_TYPE_GUID: &str = "c12a7328-f81f-11d2-ba4b-00a0c93ec93b";
pub const MAX_PATH: usize = 128;
use super::SECTOR_BYTES;
const RESERVED_SECTORS: u64 = 32;
const FAT_COUNT: u64 = 2;
const MIN_FAT32_CLUSTERS: u32 = 65_525;
const MAX_FAT32_CLUSTERS: u32 = 0x0fff_ffed;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Volume {
    /// Eight uppercase hexadecimal digits as reported by blkid, with a hyphen.
    #[serde(deserialize_with = "crate::cron::text::<_,9>")]
    pub serial: String,
    #[serde(deserialize_with = "crate::cron::text::<_,11>")]
    pub label: String,
    pub sectors_per_cluster: u8,
    pub data_clusters: u32,
}
impl Volume {
    pub fn validate(&self, partition_sectors: u64) -> Result<(), String> {
        if self.serial.len() != 9
            || !self.serial.bytes().enumerate().all(|(i, b)| {
                if i == 4 {
                    b == b'-'
                } else {
                    b.is_ascii_digit() || matches!(b, b'A'..=b'F')
                }
            })
            || self.label.len() > 11
            || !self
                .label
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b" _-".contains(&b))
            || !matches!(self.sectors_per_cluster, 1 | 2 | 4 | 8 | 16 | 32 | 64)
            || !(MIN_FAT32_CLUSTERS..=MAX_FAT32_CLUSTERS).contains(&self.data_clusters)
        {
            return Err("Use the supplied 512-byte-sector FAT32 profile with a valid volume serial, label and cluster count.".into());
        }
        // This profile explicitly reserves 32 sectors and two FATs, each with
        // a four-byte entry per data cluster and the two reserved cluster IDs.
        let fat_sectors = ((u64::from(self.data_clusters) + 2) * 4).div_ceil(SECTOR_BYTES);
        let needed = RESERVED_SECTORS + FAT_COUNT * fat_sectors + self.data_sectors();
        if needed > partition_sectors {
            return Err(format!(
                "The supplied FAT32 layout needs {needed} sectors, but this partition has {partition_sectors}."
            ));
        }
        Ok(())
    }
    fn data_sectors(&self) -> u64 {
        u64::from(self.data_clusters) * u64::from(self.sectors_per_cluster)
    }
    pub fn data_bytes(&self) -> u64 {
        self.data_sectors() * SECTOR_BYTES
    }
    pub fn allocation_bytes(&self, file_bytes: u64) -> Option<u64> {
        let cluster_bytes = u64::from(self.sectors_per_cluster) * SECTOR_BYTES;
        if cluster_bytes == 0 {
            return None;
        }
        file_bytes
            .div_ceil(cluster_bytes)
            .checked_mul(cluster_bytes)
    }
}

/// Canonical upper-case ASCII path, independent of the Linux mount directory.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Path(#[serde(deserialize_with = "read_path")] String);
fn read_path<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let raw = crate::cron::text::<_, 128>(d)?;
    let canonical = Path::parse(&raw).map_err(serde::de::Error::custom)?;
    if canonical.0 != raw {
        return Err(serde::de::Error::custom(
            "Saved FAT paths must use canonical case and separators.",
        ));
    }
    Ok(raw)
}
impl Path {
    pub fn parse(raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_PATH || !raw.is_ascii() {
            return Err("Use a supported ASCII EFI path of at most 128 bytes.".into());
        }
        let (rest, separator) = if let Some(rest) = raw.strip_prefix('/') {
            (rest, '/')
        } else if let Some(rest) = raw.strip_prefix('\\') {
            (rest, '\\')
        } else {
            return Err(
                "Use an absolute path on the EFI partition, independent of its Linux mount point."
                    .into(),
            );
        };
        let parts = rest.split(separator).collect::<Vec<_>>();
        if !(2..=8).contains(&parts.len())
            || !parts[0].eq_ignore_ascii_case("EFI")
            || parts.iter().any(|part| {
                part.is_empty()
                    || part.len() > 48
                    || part.starts_with('.')
                    || part.ends_with('.')
                    || !part
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            })
        {
            return Err("Use normalized EFI path components; mixed separators, traversal and unsupported characters are refused.".into());
        }
        Ok(Self(format!("/{}", parts.join("/").to_ascii_uppercase())))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn firmware_spelling(&self) -> String {
        self.0.replace('/', "\\")
    }
    pub fn is_application(&self) -> bool {
        self.0.ends_with(".EFI")
    }
}
