use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

use crate::components::{ApplyResult, Component, PipelineContext};
use crate::config::paths::ClaudePaths;
use crate::config::types::ConfigScope;
use crate::filemerge::{json_merge, writer};

/// Installs code-review-graph and configures it for the current project.
///
/// This tool is always per-project (builds a knowledge graph of the codebase).
/// On scope=user it only installs the pip package; on scope=project it also
/// runs `install` and `build` to set up the graph.
pub struct CodeReviewGraphComponent;

impl CodeReviewGraphComponent {
    fn is_installed(&self) -> bool {
        Command::new("uvx")
            .args(["code-review-graph", "--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn install_package(&self) -> Result<()> {
        let status = Command::new("pip")
            .args(["install", "code-review-graph"])
            .status()
            .context("Failed to run pip install code-review-graph")?;

        if !status.success() {
            anyhow::bail!("pip install code-review-graph failed");
        }
        Ok(())
    }

    fn mcp_path(&self, ctx: &PipelineContext) -> PathBuf {
        match ctx.selection.scope {
            ConfigScope::User => ctx.paths.mcp_user_config(),
            ConfigScope::Project => {
                let cwd = std::env::current_dir().unwrap_or_default();
                ctx.paths.project_mcp_json(&cwd)
            }
        }
    }

    fn settings_path(&self, ctx: &PipelineContext) -> PathBuf {
        match ctx.selection.scope {
            ConfigScope::User => ctx.paths.global_settings_json(),
            ConfigScope::Project => {
                let cwd = std::env::current_dir().unwrap_or_default();
                ctx.paths.project_settings_json(&cwd)
            }
        }
    }

    /// Remove code-review-graph configuration (keeps pip package).
    pub fn uninstall(&self, ctx: &PipelineContext) -> Result<ApplyResult> {
        let mut files_written = Vec::new();
        let mut messages = Vec::new();

        // Remove MCP server from config
        let mcp_path = self.mcp_path(ctx);
        if mcp_path.exists() {
            let content = fs::read_to_string(&mcp_path)?;
            let parsed: serde_json::Value =
                serde_json::from_str(&json_merge::strip_json_comments(&content))
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            if parsed["mcpServers"].get("code-review-graph").is_some() {
                let mut new_parsed = parsed.clone();
                if let Some(servers) = new_parsed["mcpServers"].as_object_mut() {
                    servers.remove("code-review-graph");
                }
                let new_content = serde_json::to_string_pretty(&new_parsed)?;
                writer::write_file_atomic_str(&mcp_path, &new_content)?;
                files_written.push(mcp_path);
                messages.push("MCP server code-review-graph eliminado".to_string());
            }
        }

        // Remove PostToolUse hook from settings.json
        let settings_path = self.settings_path(ctx);
        if settings_path.exists() {
            let content = fs::read_to_string(&settings_path)?;
            let parsed: serde_json::Value =
                serde_json::from_str(&json_merge::strip_json_comments(&content))
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            if let Some(hooks) = parsed.get("hooks") {
                if let Some(post) = hooks.get("PostToolUse") {
                    if post.to_string().contains("code-review-graph") {
                        let mut new_parsed = parsed.clone();
                        if let Some(arr) = new_parsed["hooks"]["PostToolUse"].as_array().cloned() {
                            let filtered: Vec<_> = arr
                                .into_iter()
                                .filter(|h| !h.to_string().contains("code-review-graph"))
                                .collect();
                            new_parsed["hooks"]["PostToolUse"] = serde_json::Value::Array(filtered);
                        }
                        let new_content = serde_json::to_string_pretty(&new_parsed)?;
                        writer::write_file_atomic_str(&settings_path, &new_content)?;
                        files_written.push(settings_path);
                        messages.push("Hook auto-update eliminado".to_string());
                    }
                }
            }
        }

        // Remove .code-review-graph/ directory
        let cwd = std::env::current_dir().unwrap_or_default();
        let graph_dir = cwd.join(".code-review-graph");
        if graph_dir.exists() {
            match fs::remove_dir_all(&graph_dir) {
                Ok(()) => messages.push(".code-review-graph/ eliminado".to_string()),
                Err(e) => messages.push(format!("No se pudo eliminar .code-review-graph/: {e}")),
            }
        }

        // Remove .code-review-graphignore if exists
        let ignore_file = cwd.join(".code-review-graphignore");
        if ignore_file.exists() {
            let _ = fs::remove_file(&ignore_file);
            messages.push(".code-review-graphignore eliminado".to_string());
        }

        Ok(ApplyResult {
            changed: !files_written.is_empty() || !messages.is_empty(),
            files_written,
            messages,
        })
    }
}

impl Component for CodeReviewGraphComponent {
    fn id(&self) -> &str {
        "code-review-graph"
    }

    fn name(&self) -> &str {
        "Code Review Graph"
    }

    fn affected_paths(&self, paths: &ClaudePaths) -> Vec<PathBuf> {
        vec![paths.mcp_user_config(), paths.global_settings_json()]
    }

    fn prepare(&self, ctx: &PipelineContext) -> Result<()> {
        let settings = self.settings_path(ctx);
        if let Some(parent) = settings.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    fn apply(&self, ctx: &PipelineContext) -> Result<ApplyResult> {
        let mut messages = Vec::new();

        // Step 1: Ensure package is available
        if !self.is_installed() {
            messages.push("Instalando code-review-graph...".to_string());
            self.install_package()?;
            messages.push("code-review-graph instalado".to_string());
        } else {
            messages.push("code-review-graph ya disponible".to_string());
        }

        // Step 2: Run native install command (sets up MCP + hooks)
        match ctx.selection.scope {
            ConfigScope::Project => {
                messages.push("Configurando code-review-graph en proyecto...".to_string());

                // `code-review-graph install` configures MCP server + hooks
                let status = Command::new("uvx")
                    .args(["code-review-graph", "install"])
                    .status()
                    .context("Failed to run code-review-graph install")?;

                if status.success() {
                    messages.push("MCP server y hooks configurados".to_string());
                } else {
                    messages.push(
                        "Advertencia: code-review-graph install fallo, configurando manualmente..."
                            .to_string(),
                    );
                    // Fallback: manual config
                    self.manual_config(ctx, &mut messages)?;
                }

                // Build the knowledge graph
                messages.push("Construyendo knowledge graph del proyecto...".to_string());
                let status = Command::new("uvx")
                    .args(["code-review-graph", "build"])
                    .status()
                    .context("Failed to run code-review-graph build")?;

                if status.success() {
                    messages.push("Knowledge graph construido".to_string());
                } else {
                    messages.push("Advertencia: graph build fallo (puedes ejecutar 'code-review-graph build' despues)".to_string());
                }
            }
            ConfigScope::User => {
                messages.push(
                    "code-review-graph es per-project: solo se instalo el paquete".to_string(),
                );
                messages.push(
                    "Ejecuta 'karma install -c crg --scope project' en cada proyecto".to_string(),
                );
            }
        }

        Ok(ApplyResult {
            changed: true,
            files_written: vec![],
            messages,
        })
    }

    fn rollback(&self, _ctx: &PipelineContext) -> Result<()> {
        Ok(())
    }
}

impl CodeReviewGraphComponent {
    /// Manual MCP + hooks config as fallback.
    fn manual_config(&self, ctx: &PipelineContext, messages: &mut Vec<String>) -> Result<()> {
        let mcp_overlay = r#"{"mcpServers": {"code-review-graph": {"command": "uvx", "args": ["code-review-graph", "serve"]}}}"#;
        let hooks_overlay = r#"{"hooks": {"PostToolUse": [{"matcher": "Write|Edit", "hooks": [{"type": "command", "command": "code-review-graph update"}]}]}}"#;

        // MCP
        let mcp_path = self.mcp_path(ctx);
        let existing = if mcp_path.exists() {
            fs::read_to_string(&mcp_path)?
        } else {
            "{}".to_string()
        };
        let merged = json_merge::merge_json_str(&existing, mcp_overlay)?;
        writer::write_file_atomic_str(&mcp_path, &merged)?;
        messages.push("MCP configurado manualmente".to_string());

        // Hooks
        let settings_path = self.settings_path(ctx);
        let existing = if settings_path.exists() {
            fs::read_to_string(&settings_path)?
        } else {
            "{}".to_string()
        };
        let merged = json_merge::merge_json_str(&existing, hooks_overlay)?;
        writer::write_file_atomic_str(&settings_path, &merged)?;
        messages.push("Hooks configurados manualmente".to_string());

        Ok(())
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
    fn test_manual_mcp_merge() {
        let existing = r#"{"mcpServers": {"context7": {"command": "npx"}}}"#;
        let overlay = r#"{"mcpServers": {"code-review-graph": {"command": "uvx", "args": ["code-review-graph", "serve"]}}}"#;
        let merged = json_merge::merge_json_str(existing, overlay).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&merged).unwrap();

        assert!(parsed["mcpServers"]["context7"].is_object());
        assert!(parsed["mcpServers"]["code-review-graph"].is_object());
    }
}
