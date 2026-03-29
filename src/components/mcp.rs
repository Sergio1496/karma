use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::components::{ApplyResult, Component, PipelineContext};
use crate::config::paths::ClaudePaths;
use crate::config::types::ConfigScope;
use crate::filemerge::{json_merge, writer};

/// Configures MCP servers for Claude Code.
///
/// Writes to:
/// - User scope: ~/.claude.json (mcpServers key)
/// - Project scope: .mcp.json
pub struct McpComponent;

impl Component for McpComponent {
    fn id(&self) -> &str {
        "mcp-servers"
    }

    fn name(&self) -> &str {
        "MCP Servers"
    }

    fn affected_paths(&self, paths: &ClaudePaths) -> Vec<PathBuf> {
        vec![paths.mcp_user_config()]
    }

    fn prepare(&self, _ctx: &PipelineContext) -> Result<()> {
        Ok(())
    }

    fn apply(&self, ctx: &PipelineContext) -> Result<ApplyResult> {
        let target = self.target_path(ctx);
        let overlay = self.platform_mcp_config();

        // Read existing config or empty
        let existing = if target.exists() {
            fs::read_to_string(&target)
                .with_context(|| format!("Failed to read {}", target.display()))?
        } else {
            "{}".to_string()
        };

        // Deep merge
        let merged = json_merge::merge_json_str(&existing, &overlay)
            .context("Failed to merge MCP configuration")?;

        let result = writer::write_file_atomic_str(&target, &merged)
            .with_context(|| format!("Failed to write {}", target.display()))?;

        Ok(ApplyResult {
            changed: result.changed,
            files_written: if result.changed {
                vec![target]
            } else {
                vec![]
            },
            messages: vec!["MCP server Context7 configured".to_string()],
        })
    }

    fn rollback(&self, _ctx: &PipelineContext) -> Result<()> {
        Ok(())
    }
}

impl McpComponent {
    /// Generate platform-aware MCP config.
    /// Windows needs `cmd /c` wrapper to execute npx.
    fn platform_mcp_config(&self) -> String {
        if cfg!(target_os = "windows") {
            r#"{"mcpServers": {"context7": {"command": "cmd", "args": ["/c", "npx", "-y", "@upstash/context7-mcp@latest"]}}}"#.to_string()
        } else {
            r#"{"mcpServers": {"context7": {"command": "npx", "args": ["-y", "@upstash/context7-mcp@latest"]}}}"#.to_string()
        }
    }

    fn target_path(&self, ctx: &PipelineContext) -> PathBuf {
        match ctx.selection.scope {
            ConfigScope::User => ctx.paths.mcp_user_config(),
            ConfigScope::Project => {
                let cwd = std::env::current_dir().unwrap_or_default();
                ctx.paths.project_mcp_json(&cwd)
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
    fn test_mcp_creates_config() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        let component = McpComponent;
        let result = component.apply(&ctx).unwrap();

        assert!(result.changed);
        let config_path = ctx.paths.mcp_user_config();
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed["mcpServers"]["context7"].is_object());
    }

    #[test]
    fn test_mcp_merges_with_existing() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        // Create existing config with another MCP server
        let config_path = ctx.paths.mcp_user_config();
        fs::write(
            &config_path,
            r#"{"mcpServers": {"existing-server": {"command": "node", "args": ["server.js"]}}}"#,
        )
        .unwrap();

        let component = McpComponent;
        component.apply(&ctx).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        // Both servers should exist
        assert!(parsed["mcpServers"]["existing-server"].is_object());
        assert!(parsed["mcpServers"]["context7"].is_object());
    }

    #[test]
    fn test_mcp_idempotent() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        let component = McpComponent;
        component.apply(&ctx).unwrap();
        let result = component.apply(&ctx).unwrap();

        assert!(!result.changed);
    }
}
