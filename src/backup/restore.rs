use std::fs;

use anyhow::Result;

use crate::backup::manifest::Manifest;

/// Restore files from a backup manifest.
///
/// For each entry:
/// - If the file existed before: copy the backup back to the original path.
/// - If the file did NOT exist before: delete it (it was created by us).
pub fn restore_from_manifest(manifest: &Manifest) -> Result<RestoreResult> {
    let mut restored = 0u32;
    let mut deleted = 0u32;
    let mut errors = Vec::new();

    for entry in &manifest.entries {
        if entry.existed_before {
            // Restore from backup
            if entry.backup_path.exists() {
                if let Some(parent) = entry.original_path.parent() {
                    fs::create_dir_all(parent).ok();
                }
                match fs::copy(&entry.backup_path, &entry.original_path) {
                    Ok(_) => restored += 1,
                    Err(e) => errors.push(format!(
                        "Failed to restore {}: {e}",
                        entry.original_path.display()
                    )),
                }
            }
        } else {
            // File didn't exist before - remove it if it exists now
            if entry.original_path.exists() {
                match fs::remove_file(&entry.original_path) {
                    Ok(()) => deleted += 1,
                    Err(e) => errors.push(format!(
                        "Failed to remove {}: {e}",
                        entry.original_path.display()
                    )),
                }
            }
        }
    }

    Ok(RestoreResult {
        restored,
        deleted,
        errors,
    })
}

/// Result of a restore operation.
#[derive(Debug)]
pub struct RestoreResult {
    pub restored: u32,
    pub deleted: u32,
    pub errors: Vec<String>,
}

impl RestoreResult {
    pub fn success(&self) -> bool {
        self.errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::manifest::ManifestEntry;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_restore_existing_file() {
        let tmp = TempDir::new().unwrap();

        // Original file (now modified)
        let original = tmp.path().join("file.txt");
        fs::write(&original, "modified content").unwrap();

        // Backup
        let backup = tmp.path().join("backup.txt");
        fs::write(&backup, "original content").unwrap();

        let manifest = Manifest {
            created_at: "test".to_string(),
            backup_dir: tmp.path().to_path_buf(),
            entries: vec![ManifestEntry {
                original_path: original.clone(),
                backup_path: backup,
                existed_before: true,
            }],
        };

        let result = restore_from_manifest(&manifest).unwrap();
        assert!(result.success());
        assert_eq!(result.restored, 1);
        assert_eq!(fs::read_to_string(&original).unwrap(), "original content");
    }

    #[test]
    fn test_restore_deletes_new_file() {
        let tmp = TempDir::new().unwrap();

        // File that was created by our tool
        let created = tmp.path().join("new_file.txt");
        fs::write(&created, "should be deleted").unwrap();

        let manifest = Manifest {
            created_at: "test".to_string(),
            backup_dir: tmp.path().to_path_buf(),
            entries: vec![ManifestEntry {
                original_path: created.clone(),
                backup_path: PathBuf::new(),
                existed_before: false,
            }],
        };

        let result = restore_from_manifest(&manifest).unwrap();
        assert!(result.success());
        assert_eq!(result.deleted, 1);
        assert!(!created.exists());
    }
}
