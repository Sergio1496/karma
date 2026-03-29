use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use tempfile::NamedTempFile;

/// Result of an atomic file write operation.
#[derive(Debug)]
pub struct WriteResult {
    /// Whether the file content actually changed.
    pub changed: bool,
    /// Whether the file was newly created (didn't exist before).
    pub created: bool,
}

/// Write content to a file atomically.
///
/// Strategy: write to a temp file in the same directory, then rename.
/// This ensures the file is never left in a partially-written state.
///
/// If the file already exists with identical content, this is a no-op.
pub fn write_file_atomic(path: &Path, content: &[u8]) -> Result<WriteResult> {
    // Check if content is identical (skip write if so)
    if path.exists() {
        let existing = fs::read(path)
            .with_context(|| format!("Failed to read existing file: {}", path.display()))?;
        if existing == content {
            return Ok(WriteResult {
                changed: false,
                created: false,
            });
        }
    }

    let created = !path.exists();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Write to temp file in the same directory (for atomic rename)
    let parent = path.parent().unwrap_or(Path::new("."));
    let temp = NamedTempFile::new_in(parent)
        .with_context(|| format!("Failed to create temp file in: {}", parent.display()))?;

    fs::write(temp.path(), content)
        .with_context(|| format!("Failed to write to temp file: {}", temp.path().display()))?;

    // Atomic rename (tempfile::persist handles Windows rename-over-existing)
    temp.persist(path)
        .with_context(|| format!("Failed to persist file: {}", path.display()))?;

    Ok(WriteResult {
        changed: true,
        created,
    })
}

/// Write string content to a file atomically.
pub fn write_file_atomic_str(path: &Path, content: &str) -> Result<WriteResult> {
    write_file_atomic(path, content.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_new_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");

        let result = write_file_atomic(&path, b"hello").unwrap();
        assert!(result.changed);
        assert!(result.created);
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");
    }

    #[test]
    fn test_write_identical_is_noop() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");

        fs::write(&path, "hello").unwrap();
        let result = write_file_atomic(&path, b"hello").unwrap();
        assert!(!result.changed);
        assert!(!result.created);
    }

    #[test]
    fn test_write_updates_content() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");

        fs::write(&path, "old").unwrap();
        let result = write_file_atomic(&path, b"new").unwrap();
        assert!(result.changed);
        assert!(!result.created);
        assert_eq!(fs::read_to_string(&path).unwrap(), "new");
    }

    #[test]
    fn test_write_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("a").join("b").join("test.txt");

        let result = write_file_atomic(&path, b"nested").unwrap();
        assert!(result.changed);
        assert!(result.created);
        assert_eq!(fs::read_to_string(&path).unwrap(), "nested");
    }

    #[test]
    fn test_write_str_convenience() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");

        let result = write_file_atomic_str(&path, "string content").unwrap();
        assert!(result.changed);
        assert_eq!(fs::read_to_string(&path).unwrap(), "string content");
    }
}
