use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;

use crate::backup::manifest::{Manifest, ManifestEntry};

/// Create a backup snapshot of the given files.
///
/// Copies each existing file into a timestamped backup directory
/// and writes a manifest.json describing the backup.
///
/// Returns the manifest for the created backup.
pub fn create_snapshot(backup_base_dir: &Path, files: &[PathBuf]) -> Result<Manifest> {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_dir = backup_base_dir.join(&timestamp);

    fs::create_dir_all(&backup_dir).with_context(|| {
        format!(
            "Failed to create backup directory: {}",
            backup_dir.display()
        )
    })?;

    let mut entries = Vec::new();

    for (i, original_path) in files.iter().enumerate() {
        let existed = original_path.exists();

        if existed {
            // Create a unique backup filename based on index + original name
            let file_name = original_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("file_{i}"));

            let backup_path = backup_dir.join(format!("{i:04}_{file_name}"));

            fs::copy(original_path, &backup_path).with_context(|| {
                format!(
                    "Failed to backup {} -> {}",
                    original_path.display(),
                    backup_path.display()
                )
            })?;

            entries.push(ManifestEntry {
                original_path: original_path.clone(),
                backup_path,
                existed_before: true,
            });
        } else {
            // File doesn't exist yet - record it so rollback can delete it
            entries.push(ManifestEntry {
                original_path: original_path.clone(),
                backup_path: PathBuf::new(),
                existed_before: false,
            });
        }
    }

    let manifest = Manifest {
        created_at: timestamp,
        backup_dir: backup_dir.clone(),
        entries,
    };

    // Write manifest
    let manifest_path = backup_dir.join("manifest.json");
    let manifest_json =
        serde_json::to_string_pretty(&manifest).context("Failed to serialize backup manifest")?;
    fs::write(&manifest_path, manifest_json)
        .with_context(|| format!("Failed to write manifest: {}", manifest_path.display()))?;

    Ok(manifest)
}

/// List all available backup snapshots, sorted by timestamp (newest first).
pub fn list_snapshots(backup_base_dir: &Path) -> Result<Vec<Manifest>> {
    if !backup_base_dir.exists() {
        return Ok(vec![]);
    }

    let mut manifests = Vec::new();

    let mut entries: Vec<_> = fs::read_dir(backup_base_dir)
        .with_context(|| {
            format!(
                "Failed to read backup directory: {}",
                backup_base_dir.display()
            )
        })?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    // Sort by name descending (timestamps sort lexicographically)
    entries.sort_by_key(|b| std::cmp::Reverse(b.file_name()));

    for entry in entries {
        let manifest_path = entry.path().join("manifest.json");
        if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path)?;
            if let Ok(manifest) = serde_json::from_str::<Manifest>(&content) {
                manifests.push(manifest);
            }
        }
    }

    Ok(manifests)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_snapshot_existing_files() {
        let tmp = TempDir::new().unwrap();
        let backup_dir = tmp.path().join("backups");

        // Create files to backup
        let file1 = tmp.path().join("file1.txt");
        let file2 = tmp.path().join("file2.json");
        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();

        let manifest = create_snapshot(&backup_dir, &[file1.clone(), file2.clone()]).unwrap();

        assert_eq!(manifest.entries.len(), 2);
        assert!(manifest.entries[0].existed_before);
        assert!(manifest.entries[1].existed_before);
        assert!(manifest.entries[0].backup_path.exists());
        assert!(manifest.entries[1].backup_path.exists());

        // Verify backup content
        assert_eq!(
            fs::read_to_string(&manifest.entries[0].backup_path).unwrap(),
            "content1"
        );
    }

    #[test]
    fn test_create_snapshot_nonexistent_files() {
        let tmp = TempDir::new().unwrap();
        let backup_dir = tmp.path().join("backups");
        let nonexistent = tmp.path().join("does_not_exist.txt");

        let manifest = create_snapshot(&backup_dir, &[nonexistent]).unwrap();

        assert_eq!(manifest.entries.len(), 1);
        assert!(!manifest.entries[0].existed_before);
    }

    #[test]
    fn test_create_snapshot_writes_manifest() {
        let tmp = TempDir::new().unwrap();
        let backup_dir = tmp.path().join("backups");
        let file = tmp.path().join("test.txt");
        fs::write(&file, "data").unwrap();

        let manifest = create_snapshot(&backup_dir, &[file]).unwrap();

        let manifest_path = manifest.backup_dir.join("manifest.json");
        assert!(manifest_path.exists());

        let loaded: Manifest =
            serde_json::from_str(&fs::read_to_string(manifest_path).unwrap()).unwrap();
        assert_eq!(loaded.entries.len(), 1);
    }

    #[test]
    fn test_list_snapshots() {
        let tmp = TempDir::new().unwrap();
        let backup_dir = tmp.path().join("backups");
        let file = tmp.path().join("test.txt");
        fs::write(&file, "data").unwrap();

        // Create two snapshots
        create_snapshot(&backup_dir, &[file.clone()]).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        create_snapshot(&backup_dir, &[file]).unwrap();

        let snapshots = list_snapshots(&backup_dir).unwrap();
        assert_eq!(snapshots.len(), 2);
        // Newest first
        assert!(snapshots[0].created_at >= snapshots[1].created_at);
    }
}
