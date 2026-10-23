//! Bounded application-owned local-file input. Never dispatched by learner commands.
use std::{io::Read, path::Path};

pub fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let ceiling = u64::try_from(limit)
        .ok()
        .and_then(|n| n.checked_add(1))
        .ok_or("The requested file size limit is too large")?;
    if !std::fs::symlink_metadata(path)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_file()
    {
        return Err(
            "Choose an ordinary file; symbolic links and special files are not supported".into(),
        );
    }
    let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if !f.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("Choose an ordinary file".into());
    }
    let mut bytes = Vec::new();
    f.take(ceiling)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err("File exceeds the supported size; original retained".into());
    }
    Ok(bytes)
}
