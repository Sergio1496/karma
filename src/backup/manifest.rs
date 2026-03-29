use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A backup manifest records which files were backed up and where.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Timestamp of the backup creation.
    pub created_at: String,
    /// Directory where backup files are stored.
    pub backup_dir: PathBuf,
    /// Individual file entries in this backup.
    pub entries: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// Original file path.
    pub original_path: PathBuf,
    /// Path to the backup copy.
    pub backup_path: PathBuf,
    /// Whether the file existed before (false = was created by us).
    pub existed_before: bool,
}
