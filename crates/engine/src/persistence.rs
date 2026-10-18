//! Validated same-filesystem atomic replacement, two backups, recovery journal.
use crate::{Error, ProjectState, Result, invalid};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
const MAX_FILE: u64 = 1_048_576;
pub fn backup_path(path: &Path, slot: usize) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(format!(".bak{slot}"));
    PathBuf::from(name)
}
fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_FILE + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE {
        return Err(invalid("Project exceeds 1 MiB limit"));
    }
    Ok(bytes)
}
fn decode(bytes: &[u8]) -> Result<ProjectState> {
    let mut p: ProjectState = serde_json::from_slice(bytes)?;
    // Schema 1 had only procedural descriptors. New fields deserialize to
    // identity processing, and its input IDs, bus state and layout are retained.
    if p.schema == 1 {
        p.schema = 2;
    }
    p.validate()?;
    Ok(p)
}
pub fn load(path: &Path) -> Result<ProjectState> {
    decode(&read_bytes(path)?)
}
fn atomic_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|e| Error::Io(e.error))?;
    Ok(())
}
pub fn save(path: &Path, project: &ProjectState) -> Result<()> {
    project.validate()?;
    let bytes = serde_json::to_vec_pretty(project)?;
    if bytes.len() as u64 > MAX_FILE {
        return Err(invalid("Project exceeds size limit"));
    }
    let roundtrip: ProjectState = serde_json::from_slice(&bytes)?;
    roundtrip.validate()?;
    if path.exists() {
        // Preserve validated original bytes: migrating a project must not also
        // migrate the rollback copy that an older application may need.
        let old = read_bytes(path)?;
        decode(&old)?;
        let first = backup_path(path, 1);
        if first.exists() {
            let older = read_bytes(&first)?;
            decode(&older)?;
            atomic_bytes(&backup_path(path, 2), &older)?;
        }
        atomic_bytes(&first, &old)?;
    }
    let journal = backup_path(path, 0);
    atomic_bytes(&journal, &bytes)?;
    atomic_bytes(path, &bytes)?;
    // Keep the validated journal for explicit operator recovery; no silent restore.
    Ok(())
}
