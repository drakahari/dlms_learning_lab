//! Bounded virtual device-reference decoding; not a new command/admission route.
//! Current simple LVM mapper encoding only; applications should prefer /dev/VG/LV.
//! No real /dev access, path normalization, symlinks, udev, mounts or host I/O.
use super::*;
pub const MAX_PATH: usize = 256;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Canonical,
    Mapper,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parts {
    pub form: Form,
    pub group: String,
    pub volume: String,
}
fn names(group: &str, volume: &str) -> Result<()> {
    if !name(group) || group == "mapper" || !provision::valid_volume_name(volume) {
        return Err(invalid(
            "Use supported simple VG/LV names, at most32characters each; mapper is a reserved path namespace",
        ));
    }
    Ok(())
}
pub fn canonical(group: &str, volume: &str) -> Result<String> {
    names(group, volume)?;
    Ok(format!("/dev/{group}/{volume}"))
}
/// An observed-node convention for this simulator, not a production ABI promise.
pub fn modeled_mapper(group: &str, volume: &str) -> Result<String> {
    names(group, volume)?;
    Ok(format!(
        "/dev/mapper/{}-{}",
        group.replace('-', "--"),
        volume.replace('-', "--")
    ))
}
fn mapper_parts(raw: &str) -> Result<(String, String)> {
    if raw.is_empty() || raw.len() > 129 || !raw.is_ascii() {
        return Err(invalid("Unsupported bounded mapper node name"));
    }
    let bytes = raw.as_bytes();
    let mut at = 0;
    let mut current = String::new();
    let mut group = None;
    while at < bytes.len() {
        if bytes[at] == b'-' {
            if bytes.get(at + 1) == Some(&b'-') {
                current.push('-');
                at += 2;
                continue;
            }
            if group.is_some() {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Only a simple VG and LV are modeled; internal layers or unescaped name hyphens are unsupported",
                ));
            }
            group = Some(std::mem::take(&mut current));
        } else {
            current.push(char::from(bytes[at]));
        }
        at += 1;
    }
    let group=group.ok_or_else(||invalid("A simple mapper node needs a VG–LV separator; name hyphens are doubled in this modeled convention"))?;
    names(&group, &current)?;
    Ok((group, current))
}
/// Decode an absolute saved application reference. LVM command operands retain
/// their existing parser; this does not silently broaden any terminal subset.
pub fn parse(path: &str) -> Result<Parts> {
    if path.is_empty()
        || path.len() > MAX_PATH
        || path
            .bytes()
            .any(|b| b.is_ascii_control() || b.is_ascii_whitespace())
    {
        return Err(invalid(
            "Use one device reference up to256bytes, without whitespace or control characters",
        ));
    }
    let (form, group, volume) = if let Some(raw) = path.strip_prefix("/dev/mapper/") {
        let (g, v) = mapper_parts(raw)?;
        (Form::Mapper, g, v)
    } else if let Some(raw) = path.strip_prefix("/dev/") {
        let (g, v) = raw.split_once('/').ok_or_else(|| {
            invalid("Use /dev/VG/LV; kernel dm-N names and other device forms are not modeled")
        })?;
        names(g, v)?;
        (Form::Canonical, g.into(), v.into())
    } else {
        return Err(error(
            ErrorKind::Unsupported,
            "Only /dev/VG/LV and this simulator’s simple /dev/mapper/VG-LV convention are supported; no host path is accessed",
        ));
    };
    Ok(Parts {
        form,
        group,
        volume,
    })
}
#[derive(Debug)]
pub struct Resolved<'a> {
    pub parts: Parts,
    pub group: &'a VolumeGroup,
    pub volume: &'a LogicalVolume,
}
pub fn resolve<'a>(s: &'a State, path: &str) -> Result<Resolved<'a>> {
    s.validate()?;
    let parts = parse(path)?;
    let group = s.groups.get(&parts.group).ok_or_else(|| {
        error(
            ErrorKind::Unknown,
            "This saved path names no current modeled volume group; inspect the current names",
        )
    })?;
    let volume=group.volumes.get(&parts.volume).ok_or_else(||error(ErrorKind::Unknown,"This saved path names no current modeled logical volume; a rename changes name-based references"))?;
    Ok(Resolved {
        parts,
        group,
        volume,
    })
}
/// Simulator application read, not a Linux utility or mounted filesystem read.
pub fn read<'a>(s: &'a State, path: &str, file: &str) -> Result<&'a str> {
    let resolved = resolve(s, path)?;
    s.read_file(
        &format!("{}/{}", resolved.parts.group, resolved.parts.volume),
        file,
    )
}
