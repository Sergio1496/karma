use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::types::ComponentId;

/// Persisted state for the tool.
///
/// Stored at ~/.karma/state.json.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppState {
    /// Components that are currently installed.
    pub installed_components: Vec<ComponentId>,
    /// Skills that are currently installed (by ID).
    pub installed_skills: Vec<String>,
    /// Last installation timestamp.
    pub last_install: Option<String>,
    /// Last sync timestamp.
    pub last_sync: Option<String>,
}

impl AppState {
    /// Read state from disk. Returns default state if file doesn't exist.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read state file: {}", path.display()))?;

        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse state file: {}", path.display()))
    }

    /// Write state to disk.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create state directory: {}", parent.display())
            })?;
        }

        let content = serde_json::to_string_pretty(self).context("Failed to serialize state")?;

        fs::write(path, format!("{content}\n"))
            .with_context(|| format!("Failed to write state file: {}", path.display()))
    }

    /// Record a successful installation.
    pub fn record_install(&mut self, components: &[ComponentId], skills: &[String]) {
        // Merge components (don't duplicate)
        for c in components {
            if !self.installed_components.contains(c) {
                self.installed_components.push(*c);
            }
        }
        // Merge skills
        for s in skills {
            if !self.installed_skills.contains(s) {
                self.installed_skills.push(s.clone());
            }
        }
        self.last_install = Some(chrono::Utc::now().to_rfc3339());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_nonexistent_returns_default() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("state.json");
        let state = AppState::load(&path).unwrap();
        assert!(state.installed_components.is_empty());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("state.json");

        let mut state = AppState::default();
        state.record_install(
            &[ComponentId::Skills, ComponentId::Orchestrator],
            &["go-testing".to_string()],
        );

        state.save(&path).unwrap();

        let loaded = AppState::load(&path).unwrap();
        assert_eq!(loaded.installed_components.len(), 2);
        assert!(loaded.installed_components.contains(&ComponentId::Skills));
        assert_eq!(loaded.installed_skills, vec!["go-testing"]);
        assert!(loaded.last_install.is_some());
    }

    #[test]
    fn test_record_install_deduplicates() {
        let mut state = AppState::default();
        state.record_install(&[ComponentId::Skills], &[]);
        state.record_install(&[ComponentId::Skills, ComponentId::Permissions], &[]);
        assert_eq!(state.installed_components.len(), 2);
    }
}
