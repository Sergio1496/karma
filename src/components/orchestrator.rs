use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::assets;
use crate::components::{ApplyResult, Component, PipelineContext};
use crate::config::paths::ClaudePaths;
use crate::config::types::ConfigScope;
use crate::filemerge::section;
use crate::filemerge::writer;

const ROUTING_SECTION: &str = "model-routing";
const ASSIGNMENTS_SECTION: &str = "model-assignments";

/// Injects model routing instructions and assignments table into CLAUDE.md.
pub struct OrchestratorComponent;

impl Component for OrchestratorComponent {
    fn id(&self) -> &str {
        "orchestrator"
    }

    fn name(&self) -> &str {
        "Model Assignments"
    }

    fn affected_paths(&self, paths: &ClaudePaths) -> Vec<PathBuf> {
        vec![paths.global_claude_md()]
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
        let routing_instructions = assets::orchestrator_template();

        let existing = if target.exists() {
            fs::read_to_string(&target)
                .with_context(|| format!("Failed to read {}", target.display()))?
        } else {
            String::new()
        };

        // 1. Inject routing instructions
        let updated =
            section::inject_markdown_section(&existing, ROUTING_SECTION, &routing_instructions);

        // 2. Inject model assignments table
        let assignments = self.render_assignments(&ctx.selection);
        let updated = section::inject_markdown_section(&updated, ASSIGNMENTS_SECTION, &assignments);

        let result = writer::write_file_atomic_str(&target, &updated)
            .with_context(|| format!("Failed to write {}", target.display()))?;

        Ok(ApplyResult {
            changed: result.changed,
            files_written: if result.changed { vec![target] } else { vec![] },
            messages: vec![format!(
                "Model assignments ({}) injected into CLAUDE.md",
                ctx.selection.model_preset.display_name()
            )],
        })
    }

    fn rollback(&self, _ctx: &PipelineContext) -> Result<()> {
        Ok(())
    }
}

impl OrchestratorComponent {
    fn target_path(&self, ctx: &PipelineContext) -> PathBuf {
        match ctx.selection.scope {
            ConfigScope::User => ctx.paths.global_claude_md(),
            ConfigScope::Project => {
                let cwd = std::env::current_dir().unwrap_or_default();
                ctx.paths.project_claude_md(&cwd)
            }
        }
    }

    fn render_assignments(&self, selection: &crate::config::types::Selection) -> String {
        let mut content = String::from("## Model Assignments\n\n");
        content.push_str(&format!(
            "**Preset: {}** — {}\n\n",
            selection.model_preset.display_name(),
            selection.model_preset.description()
        ));
        content.push_str(&selection.render_markdown_table());
        content
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
    fn test_injects_model_routing() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        let component = OrchestratorComponent;
        component.prepare(&ctx).unwrap();
        let result = component.apply(&ctx).unwrap();

        assert!(result.changed);
        let content = fs::read_to_string(ctx.paths.global_claude_md()).unwrap();
        assert!(content.contains("<!-- karma:model-routing -->"));
        assert!(content.contains("Model Routing"));
        assert!(content.contains("<!-- karma:model-assignments -->"));
        assert!(content.contains("**opus**"));
        assert!(content.contains("**sonnet**"));
        assert!(content.contains("**haiku**"));
    }

    #[test]
    fn test_preserves_existing_content() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        let claude_md = ctx.paths.global_claude_md();
        fs::create_dir_all(claude_md.parent().unwrap()).unwrap();
        fs::write(
            &claude_md,
            "# My Custom Instructions\n\nDo things my way.\n",
        )
        .unwrap();

        let component = OrchestratorComponent;
        component.prepare(&ctx).unwrap();
        component.apply(&ctx).unwrap();

        let content = fs::read_to_string(&claude_md).unwrap();
        assert!(content.contains("# My Custom Instructions"));
        assert!(content.contains("Do things my way."));
        assert!(content.contains("<!-- karma:model-routing -->"));
    }

    #[test]
    fn test_idempotent() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        let component = OrchestratorComponent;
        component.prepare(&ctx).unwrap();
        component.apply(&ctx).unwrap();
        let result = component.apply(&ctx).unwrap();

        assert!(!result.changed);
    }
}
