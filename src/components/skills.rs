use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::components::{ApplyResult, Component, PipelineContext};
use crate::config::paths::ClaudePaths;
use crate::config::types::ConfigScope;
use crate::filemerge::writer;

/// Injects skill files into Claude Code's skills directory.
///
/// Each skill is a SKILL.md file placed at:
/// - User scope: ~/.claude/skills/{skill-id}/SKILL.md
/// - Project scope: .claude/skills/{skill-id}/SKILL.md
pub struct SkillsComponent;

impl Component for SkillsComponent {
    fn id(&self) -> &str {
        "skills"
    }

    fn name(&self) -> &str {
        "Skills"
    }

    fn affected_paths(&self, _paths: &ClaudePaths) -> Vec<PathBuf> {
        // Skills are individual files - exact paths depend on selection.
        // Return empty here; backup of individual skill files happens
        // only if they already exist (handled by the snapshot system).
        vec![]
    }

    fn prepare(&self, ctx: &PipelineContext) -> Result<()> {
        let skills_dir = self.skills_dir(ctx);
        fs::create_dir_all(&skills_dir)
            .with_context(|| format!("Cannot create skills directory: {}", skills_dir.display()))?;
        Ok(())
    }

    fn apply(&self, ctx: &PipelineContext) -> Result<ApplyResult> {
        let mut files_written = Vec::new();
        let mut messages = Vec::new();
        let mut any_changed = false;

        if ctx.selection.skills.is_empty() {
            return Ok(ApplyResult {
                changed: false,
                files_written: vec![],
                messages: vec!["No skills selected".to_string()],
            });
        }

        for skill_id in &ctx.selection.skills {
            match self.install_skill(ctx, skill_id) {
                Ok(Some(path)) => {
                    any_changed = true;
                    files_written.push(path);
                    messages.push(format!("Installed skill: {skill_id}"));
                }
                Ok(None) => {
                    messages.push(format!("Skill unchanged: {skill_id}"));
                }
                Err(e) => {
                    // Graceful degradation: log and skip failed skills
                    messages.push(format!("Skipped skill {skill_id}: {e}"));
                }
            }
        }

        Ok(ApplyResult {
            changed: any_changed,
            files_written,
            messages,
        })
    }

    fn rollback(&self, _ctx: &PipelineContext) -> Result<()> {
        Ok(())
    }
}

impl SkillsComponent {
    fn skills_dir(&self, ctx: &PipelineContext) -> PathBuf {
        match ctx.selection.scope {
            ConfigScope::User => ctx.paths.skills_dir(),
            ConfigScope::Project => {
                let cwd = std::env::current_dir().unwrap_or_default();
                ctx.paths.project_skills_dir(&cwd)
            }
        }
    }

    /// Install a single skill by ID.
    ///
    /// Returns Some(path) if the file was written/changed, None if unchanged.
    fn install_skill(&self, ctx: &PipelineContext, skill_id: &str) -> Result<Option<PathBuf>> {
        // Try to get skill content from the remote cache first,
        // then fall back to embedded assets.
        let content = self.resolve_skill_content(ctx, skill_id)?;

        let target = match ctx.selection.scope {
            ConfigScope::User => ctx.paths.skill_file(skill_id),
            ConfigScope::Project => {
                let cwd = std::env::current_dir().unwrap_or_default();
                ctx.paths
                    .project_skills_dir(&cwd)
                    .join(skill_id)
                    .join("SKILL.md")
            }
        };

        let result = writer::write_file_atomic_str(&target, &content)
            .with_context(|| format!("Failed to write skill: {}", target.display()))?;

        if result.changed {
            Ok(Some(target))
        } else {
            Ok(None)
        }
    }

    /// Resolve skill content from cache, embedded assets, or generate placeholder.
    fn resolve_skill_content(&self, ctx: &PipelineContext, skill_id: &str) -> Result<String> {
        // 1. Check embedded assets
        let asset_path = format!("skills/{skill_id}/SKILL.md");
        if let Some(content) = crate::assets::get_asset(&asset_path) {
            return Ok(content);
        }

        // 2. Check local cache (populated by `karma sync` or prior fetches)
        let cache = crate::remote::cache::FileCache::new(ctx.paths.cache_dir());
        let cache_key = format!("skill_{skill_id}");
        if let Some(content) = cache.get(&cache_key) {
            return Ok(content);
        }

        // 3. Skill not available locally - generate a placeholder
        //    User should run `karma sync` to download from remote
        Ok(format!(
            "---\nname: {skill_id}\ndescription: Skill {skill_id}\n---\n\n# {skill_id}\n\nThis skill was installed by karma. Run `karma sync` to download the full content from the remote catalog.\n"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::Selection;
    use tempfile::TempDir;

    fn make_ctx(tmp: &TempDir, skills: Vec<String>) -> PipelineContext {
        PipelineContext {
            paths: ClaudePaths::with_home(tmp.path().to_path_buf()),
            selection: Selection {
                skills,
                ..Selection::default()
            },
            dry_run: false,
            backup_manifest: None,
        }
    }

    #[test]
    fn test_no_skills_selected() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp, vec![]);

        let component = SkillsComponent;
        component.prepare(&ctx).unwrap();
        let result = component.apply(&ctx).unwrap();

        assert!(!result.changed);
    }

    #[test]
    fn test_install_skill_creates_file() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp, vec!["go-testing".to_string()]);

        let component = SkillsComponent;
        component.prepare(&ctx).unwrap();
        let result = component.apply(&ctx).unwrap();

        assert!(result.changed);
        let skill_path = ctx.paths.skill_file("go-testing");
        assert!(skill_path.exists());
        let content = fs::read_to_string(&skill_path).unwrap();
        assert!(content.contains("go-testing"));
    }

    #[test]
    fn test_install_multiple_skills() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp, vec!["skill-a".to_string(), "skill-b".to_string()]);

        let component = SkillsComponent;
        component.prepare(&ctx).unwrap();
        let result = component.apply(&ctx).unwrap();

        assert!(result.changed);
        assert_eq!(result.files_written.len(), 2);
        assert!(ctx.paths.skill_file("skill-a").exists());
        assert!(ctx.paths.skill_file("skill-b").exists());
    }
}
