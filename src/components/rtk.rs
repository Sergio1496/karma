use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

use crate::components::{ApplyResult, Component, PipelineContext};
use crate::config::paths::ClaudePaths;
use crate::config::types::ConfigScope;

const RTK_REPO: &str = "https://github.com/Sergio1496/rtk";
const RTK_BRANCH: &str = "feat/flutter-support";

/// Installs RTK and configures Claude Code hooks for token-optimized command proxying.
pub struct RtkComponent;

impl RtkComponent {
    fn is_rtk_installed(&self) -> bool {
        Command::new("rtk")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn install_rtk(&self) -> Result<()> {
        let status = Command::new("cargo")
            .args(["install", "--git", RTK_REPO, "--branch", RTK_BRANCH])
            .status()
            .context("Failed to run cargo install for RTK")?;

        if !status.success() {
            anyhow::bail!("cargo install rtk failed with exit code: {}", status);
        }
        Ok(())
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

    fn claude_md_path(&self, ctx: &PipelineContext) -> PathBuf {
        match ctx.selection.scope {
            ConfigScope::User => ctx.paths.global_claude_md(),
            ConfigScope::Project => {
                let cwd = std::env::current_dir().unwrap_or_default();
                ctx.paths.project_claude_md(&cwd)
            }
        }
    }

    fn rtk_md_path(&self, ctx: &PipelineContext) -> PathBuf {
        match ctx.selection.scope {
            ConfigScope::User => ctx.paths.global_config_dir().join("RTK.md"),
            ConfigScope::Project => {
                let cwd = std::env::current_dir().unwrap_or_default();
                cwd.join(".claude").join("RTK.md")
            }
        }
    }

    /// Remove RTK configuration from this project (keeps binary and global config).
    pub fn uninstall(&self, ctx: &PipelineContext) -> Result<ApplyResult> {
        let mut messages = Vec::new();
        messages.push("Desinstalando RTK del proyecto...".to_string());

        // Clean CLAUDE.md — remove all RTK content
        let claude_md = self.claude_md_path(ctx);
        if claude_md.exists() {
            if let Ok(content) = fs::read_to_string(&claude_md) {
                let mut cleaned = content.clone();

                // Remove karma marker section
                if cleaned.contains("<!-- karma:rtk-reference -->") {
                    cleaned = crate::filemerge::section::inject_markdown_section(
                        &cleaned,
                        "rtk-reference",
                        "",
                    );
                }

                // Remove RTK's native block: <!-- rtk-instructions ... --> ... <!-- /rtk-instructions -->
                if let (Some(start), Some(end)) = (
                    cleaned.find("<!-- rtk-instructions"),
                    cleaned.find("<!-- /rtk-instructions -->"),
                ) {
                    if start < end {
                        let block_end = end + "<!-- /rtk-instructions -->".len();
                        let before = cleaned[..start].trim_end_matches('\n');
                        let after = cleaned[block_end..].trim_start_matches('\n');
                        cleaned = if before.is_empty() {
                            after.to_string()
                        } else if after.is_empty() {
                            before.to_string()
                        } else {
                            format!("{before}\n\n{after}")
                        };
                    }
                }

                // Remove standalone @RTK.md lines
                let lines: Vec<&str> = cleaned.lines().collect();
                let filtered: Vec<&str> = lines
                    .into_iter()
                    .filter(|l| l.trim() != "@RTK.md")
                    .collect();
                cleaned = filtered.join("\n");

                // Remove trailing empty lines
                while cleaned.ends_with("\n\n") {
                    cleaned.pop();
                }
                if !cleaned.is_empty() && !cleaned.ends_with('\n') {
                    cleaned.push('\n');
                }

                if cleaned != content {
                    let _ = crate::filemerge::writer::write_file_atomic_str(&claude_md, &cleaned);
                    messages.push("Contenido RTK eliminado de CLAUDE.md".to_string());
                }
            }
        }

        // Remove RTK.md if rtk init --uninstall didn't
        let rtk_md = self.rtk_md_path(ctx);
        if rtk_md.exists() {
            let _ = fs::remove_file(&rtk_md);
            messages.push("RTK.md eliminado".to_string());
        }

        Ok(ApplyResult {
            changed: true,
            files_written: vec![],
            messages,
        })
    }
}

impl Component for RtkComponent {
    fn id(&self) -> &str {
        "rtk"
    }

    fn name(&self) -> &str {
        "RTK (Token Killer)"
    }

    fn affected_paths(&self, paths: &ClaudePaths) -> Vec<PathBuf> {
        vec![
            paths.global_settings_json(),
            paths.global_claude_md(),
            paths.global_config_dir().join("RTK.md"),
        ]
    }

    fn prepare(&self, ctx: &PipelineContext) -> Result<()> {
        let settings = self.settings_path(ctx);
        if let Some(parent) = settings.parent() {
            fs::create_dir_all(parent)?;
        }
        let claude_md = self.claude_md_path(ctx);
        if let Some(parent) = claude_md.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    fn apply(&self, ctx: &PipelineContext) -> Result<ApplyResult> {
        let mut messages = Vec::new();

        // Step 1: Install RTK binary if not present
        if !self.is_rtk_installed() {
            messages.push("Instalando RTK desde fork...".to_string());
            self.install_rtk()?;
            messages.push("RTK binario instalado".to_string());
        } else {
            messages.push("RTK binario ya disponible".to_string());
        }

        // Step 2: Run rtk init with the right scope
        // rtk init handles: hook in settings.json, RTK.md, CLAUDE.md reference
        let mut cmd = Command::new("rtk");
        cmd.arg("init").arg("--auto-patch");

        match ctx.selection.scope {
            ConfigScope::User => {
                cmd.arg("-g"); // global: ~/.claude/
                messages.push("Configurando RTK globalmente (rtk init -g)...".to_string());
            }
            ConfigScope::Project => {
                // sin -g: configura el proyecto actual
                messages.push("Configurando RTK en proyecto (rtk init)...".to_string());
            }
        }

        let status = cmd.status().context("Failed to run rtk init")?;

        if status.success() {
            messages.push("RTK configurado correctamente".to_string());
        } else {
            anyhow::bail!("rtk init failed with exit code: {}", status);
        }

        Ok(ApplyResult {
            changed: true,
            files_written: vec![self.settings_path(ctx)],
            messages,
        })
    }

    fn rollback(&self, _ctx: &PipelineContext) -> Result<()> {
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
    fn test_rtk_binary_check() {
        let component = RtkComponent;
        // Just verify the method runs without panic
        let _ = component.is_rtk_installed();
    }

    #[test]
    fn test_rtk_md_asset_exists() {
        let content = crate::assets::get_asset("defaults/rtk.md");
        assert!(content.is_some());
        let text = content.unwrap();
        assert!(text.contains("RTK - Rust Token Killer"));
        assert!(text.contains("rtk gain"));
        assert!(text.contains("flutter"));
    }
}
