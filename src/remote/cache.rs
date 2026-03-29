use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};

/// Default TTL for cached files: 24 hours.
const DEFAULT_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Local file cache for downloaded remote assets.
///
/// Stores files at `~/.karma/cache/` with TTL-based invalidation.
pub struct FileCache {
    cache_dir: PathBuf,
    ttl: Duration,
}

impl FileCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            ttl: DEFAULT_TTL,
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Get a cached file if it exists and is not expired.
    pub fn get(&self, key: &str) -> Option<String> {
        let path = self.key_path(key);
        if !path.exists() {
            return None;
        }

        // Check TTL
        if self.is_expired(&path) {
            return None;
        }

        fs::read_to_string(&path).ok()
    }

    /// Store content in the cache.
    pub fn set(&self, key: &str, content: &str) -> Result<()> {
        let path = self.key_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create cache dir: {}", parent.display()))?;
        }
        fs::write(&path, content)
            .with_context(|| format!("Failed to write cache: {}", path.display()))?;
        Ok(())
    }

    /// Check if a key exists in the cache (regardless of expiry).
    pub fn has(&self, key: &str) -> bool {
        self.key_path(key).exists()
    }

    /// Check if a key exists and is still valid (not expired).
    pub fn is_valid(&self, key: &str) -> bool {
        let path = self.key_path(key);
        path.exists() && !self.is_expired(&path)
    }

    /// Remove a specific key from cache.
    pub fn remove(&self, key: &str) -> Result<()> {
        let path = self.key_path(key);
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Clear the entire cache.
    pub fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    fn key_path(&self, key: &str) -> PathBuf {
        // Sanitize key to be a valid filename
        let safe_key = key.replace(['/', '\\', ':'], "_");
        self.cache_dir.join(safe_key)
    }

    fn is_expired(&self, path: &Path) -> bool {
        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return true,
        };

        let modified = match metadata.modified() {
            Ok(t) => t,
            Err(_) => return true,
        };

        match SystemTime::now().duration_since(modified) {
            Ok(age) => age > self.ttl,
            Err(_) => false, // Modified in the future? Not expired.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_set_and_get() {
        let tmp = TempDir::new().unwrap();
        let cache = FileCache::new(tmp.path().join("cache"));

        cache.set("skill_go-testing", "content here").unwrap();
        let result = cache.get("skill_go-testing");
        assert_eq!(result, Some("content here".to_string()));
    }

    #[test]
    fn test_cache_miss() {
        let tmp = TempDir::new().unwrap();
        let cache = FileCache::new(tmp.path().join("cache"));

        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn test_cache_expired() {
        let tmp = TempDir::new().unwrap();
        let cache = FileCache::new(tmp.path().join("cache")).with_ttl(Duration::from_secs(0));

        cache.set("key", "value").unwrap();
        std::thread::sleep(Duration::from_millis(10));
        assert!(cache.get("key").is_none());
    }

    #[test]
    fn test_cache_is_valid() {
        let tmp = TempDir::new().unwrap();
        let cache = FileCache::new(tmp.path().join("cache"));

        assert!(!cache.is_valid("key"));
        cache.set("key", "value").unwrap();
        assert!(cache.is_valid("key"));
    }

    #[test]
    fn test_cache_clear() {
        let tmp = TempDir::new().unwrap();
        let cache = FileCache::new(tmp.path().join("cache"));

        cache.set("a", "1").unwrap();
        cache.set("b", "2").unwrap();
        cache.clear().unwrap();
        assert!(cache.get("a").is_none());
        assert!(cache.get("b").is_none());
    }

    #[test]
    fn test_cache_key_sanitization() {
        let tmp = TempDir::new().unwrap();
        let cache = FileCache::new(tmp.path().join("cache"));

        cache.set("skills/go-testing/SKILL.md", "content").unwrap();
        let result = cache.get("skills/go-testing/SKILL.md");
        assert_eq!(result, Some("content".to_string()));
    }
}
