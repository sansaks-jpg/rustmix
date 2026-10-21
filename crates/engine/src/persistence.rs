//! Validated same-filesystem atomic replacement, two backups, recovery journal.
use crate::{
    Error, OverlayState, ProjectState, Result, SCHEMA, invalid,
    overlays::{OverlayBank, STANDARD_CHANNELS},
};
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
    let mut value: serde_json::Value = serde_json::from_slice(bytes)?;
    let schema = value
        .get("schema")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| invalid("Project schema is missing"))?;
    if !(1..=u64::from(SCHEMA)).contains(&schema) {
        return Err(invalid(&format!(
            "Unsupported project version; this application supports schema {SCHEMA}",
        )));
    }
    if let Some(raw) = value.get_mut("overlays")
        && let Some(panels) = raw.as_array()
    {
        // Schemas 1–4 stored four title panels. Expand them into the schema-5
        // sixteen-channel bank while keeping every authored title, subtitle and
        // position; channels 5–8 keep their fresh defaults and stingers idle.
        let legacy: Vec<OverlayState> =
            serde_json::from_value(serde_json::Value::Array(panels.clone()))?;
        let mut bank = OverlayBank::demo();
        for (slot, panel) in legacy.into_iter().take(STANDARD_CHANNELS).enumerate() {
            if let Some(target) = bank.standard.get_mut(slot) {
                *target = panel;
            }
        }
        *raw = serde_json::to_value(bank)?;
    }
    let Some(object) = value.as_object_mut() else {
        return Err(invalid("Project JSON must be an object"));
    };
    // Only pre-output schemas may omit the bank. Missing current-schema state
    // is corruption, not a request to silently clear all output assignments.
    if schema < 6 && !object.contains_key("outputs") {
        object.insert(
            "outputs".into(),
            serde_json::to_value(crate::ExternalOutputBank::default())?,
        );
    }
    object.insert("schema".into(), SCHEMA.into());
    let mut p: ProjectState = serde_json::from_value(value)?;
    // Older descriptors retain source IDs, routing and layout. New production
    // configuration defaults to idle, with no playlist started during migration.
    if matches!(schema, 1 | 2) {
        for slot in &mut p.transition_slots {
            slot.duration_ms = p.duration_ms;
        }
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
    // A short-lived Windows sharing/antivirus hold must not discard a flushed
    // replacement. Keep the same file and bound retries on the I/O worker.
    for delay_ms in [5, 15, 40] {
        match temp.persist(path) {
            Ok(_) => return Ok(()),
            Err(error) => {
                if error.error.kind() != std::io::ErrorKind::PermissionDenied {
                    return Err(Error::Io(error.error));
                }
                temp = error.file;
                std::thread::sleep(std::time::Duration::from_millis(delay_ms));
            }
        }
    }
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
