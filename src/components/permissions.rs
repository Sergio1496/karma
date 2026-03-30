use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::assets;
use crate::components::{ApplyResult, Component, PipelineContext};
use crate::config::paths::ClaudePaths;
use crate::config::types::ConfigScope;
use crate::filemerge::{json_merge, writer};

/// Overlays security-first permission defaults into Claude Code's settings.json.
///
/// Additive merge: never removes existing permissions, only adds deny rules.
pub struct PermissionsComponent;

impl Component for PermissionsComponent {
    fn id(&self) -> &str {
        "permissions"
    }

    fn name(&self) -> &str {
        "Permissions"
    }

    fn affected_paths(&self, paths: &ClaudePaths) -> Vec<PathBuf> {
        vec![paths.global_settings_json()]
    }

    fn prepare(&self, ctx: &PipelineContext) -> Result<()> {
        let target = self.target_path(ctx);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Cannot create directory: {}", parent.display()))?;
        }
        Ok(())
    }

    fn apply(&self, ctx: &PipelineContext) -> Result<ApplyResult> {
        let target = self.target_path(ctx);
        let overlay = assets::default_permissions();

        let existing = if target.exists() {
            fs::read_to_string(&target)
                .with_context(|| format!("Failed to read {}", target.display()))?
        } else {
            "{}".to_string()
        };

        let merged = json_merge::merge_json_str(&existing, &overlay)
            .context("Failed to merge permissions")?;

        let result = writer::write_file_atomic_str(&target, &merged)
            .with_context(|| format!("Failed to write {}", target.display()))?;

        Ok(ApplyResult {
            changed: result.changed,
            files_written: if result.changed { vec![target] } else { vec![] },
            messages: vec!["Security permissions applied".to_string()],
        })
    }

    fn rollback(&self, _ctx: &PipelineContext) -> Result<()> {
        Ok(())
    }
}

impl PermissionsComponent {
    fn target_path(&self, ctx: &PipelineContext) -> PathBuf {
        match ctx.selection.scope {
            ConfigScope::User => ctx.paths.global_settings_json(),
            ConfigScope::Project => {
                let cwd = std::env::current_dir().unwrap_or_default();
                ctx.paths.project_settings_json(&cwd)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::Selection;
    use tempfile::TempDir;

    fn make_ctx(tmp: &TempDir) -> PipelineContext {
        PipelineContext {
            paths: ClaudePaths::with_home(tmp.path().to_path_buf()),
            selection: Selection::default(),
            dry_run: false,
            backup_manifest: None,
        }
    }

    #[test]
    fn test_permissions_creates_settings() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        let component = PermissionsComponent;
        component.prepare(&ctx).unwrap();
        let result = component.apply(&ctx).unwrap();

        assert!(result.changed);
        let content = fs::read_to_string(ctx.paths.global_settings_json()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed["permissions"]["deny"].is_array());
    }

    #[test]
    fn test_permissions_merges_with_existing() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        let settings = ctx.paths.global_settings_json();
        fs::create_dir_all(settings.parent().unwrap()).unwrap();
        fs::write(&settings, r#"{"customKey": true}"#).unwrap();

        let component = PermissionsComponent;
        component.prepare(&ctx).unwrap();
        component.apply(&ctx).unwrap();

        let content = fs::read_to_string(&settings).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["customKey"], true);
        assert!(parsed["permissions"]["deny"].is_array());
    }
}
