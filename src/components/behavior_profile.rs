use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::assets;
use crate::components::{ApplyResult, Component, PipelineContext};
use crate::config::paths::ClaudePaths;
use crate::config::types::{BehaviorProfileStatus, ConfigScope, ProfileId};
use crate::filemerge::{section, writer};

const SECTION_ID: &str = "behavior-profile";

/// Signatures for detecting manually-pasted profile content (without karma markers).
const MANUAL_SIGNATURES: &[(ProfileId, &[&str])] = &[
    (
        ProfileId::Coding,
        &["code-first", "No abstractions for single-use"],
    ),
    (
        ProfileId::Agents,
        &["Structured output only", "serialization"],
    ),
    (
        ProfileId::Analysis,
        &["Lead with the finding", "Preserve meaningful precision"],
    ),
    (
        ProfileId::Universal,
        &["No sycophantic openers", "Keep solutions simple and direct"],
    ),
];

/// Detect the behavioral profile status by reading CLAUDE.md files.
pub fn detect(paths: &ClaudePaths) -> BehaviorProfileStatus {
    let cwd = std::env::current_dir().unwrap_or_default();

    // Check project-level first, then user-level
    let files_to_check = [
        paths.project_claude_md(&cwd),
        paths.global_claude_md(),
    ];

    for claude_md in &files_to_check {
        if let Ok(content) = fs::read_to_string(claude_md) {
            // 1. Check for karma markers
            if content.contains(&format!("<!-- karma:{SECTION_ID} -->")) {
                for profile in ProfileId::ALL {
                    if content.contains(profile.signature()) {
                        return BehaviorProfileStatus::Installed(*profile);
                    }
                }
                // Has marker but no recognizable profile signature
                return BehaviorProfileStatus::Installed(ProfileId::Universal);
            }

            // 2. Check for manually-pasted content (more specific profiles first)
            for (profile, signatures) in MANUAL_SIGNATURES {
                if signatures.iter().all(|sig| content.contains(sig)) {
                    return BehaviorProfileStatus::ManuallyDetected(*profile);
                }
            }
        }
    }

    BehaviorProfileStatus::NotInstalled
}

/// Injects a behavioral profile into CLAUDE.md to reduce output tokens.
pub struct BehaviorProfileComponent;

impl BehaviorProfileComponent {
    fn claude_md_path(&self, ctx: &PipelineContext) -> PathBuf {
        match ctx.selection.scope {
            ConfigScope::User => ctx.paths.global_claude_md(),
            ConfigScope::Project => {
                let cwd = std::env::current_dir().unwrap_or_default();
                ctx.paths.project_claude_md(&cwd)
            }
        }
    }

    /// Remove the karma-managed behavior profile section from CLAUDE.md.
    pub fn uninstall(&self, ctx: &PipelineContext) -> Result<ApplyResult> {
        let mut messages = Vec::new();
        let claude_md = self.claude_md_path(ctx);

        if claude_md.exists() {
            let content = fs::read_to_string(&claude_md)?;
            if content.contains(&format!("<!-- karma:{SECTION_ID} -->")) {
                let cleaned = section::inject_markdown_section(&content, SECTION_ID, "");
                writer::write_file_atomic_str(&claude_md, &cleaned)?;
                messages.push("Perfil de comportamiento eliminado de CLAUDE.md".to_string());
            }
        }

        if messages.is_empty() {
            messages.push("Perfil de comportamiento no estaba configurado via karma".to_string());
        }

        Ok(ApplyResult {
            changed: !messages.is_empty(),
            files_written: vec![],
            messages,
        })
    }
}

impl Component for BehaviorProfileComponent {
    fn id(&self) -> &str {
        "behavior-profile"
    }

    fn name(&self) -> &str {
        "Behavioral Profile"
    }

    fn affected_paths(&self, paths: &ClaudePaths) -> Vec<PathBuf> {
        vec![paths.global_claude_md()]
    }

    fn prepare(&self, ctx: &PipelineContext) -> Result<()> {
        let path = self.claude_md_path(ctx);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Cannot create directory: {}", parent.display()))?;
        }
        Ok(())
    }

    fn apply(&self, ctx: &PipelineContext) -> Result<ApplyResult> {
        let profile = match ctx.selection.behavior_profile {
            Some(p) => p,
            None => {
                return Ok(ApplyResult {
                    changed: false,
                    files_written: vec![],
                    messages: vec!["No behavior profile selected (skipped)".to_string()],
                });
            }
        };

        let claude_md = self.claude_md_path(ctx);
        let template = assets::behavior_profile_template(&profile);

        let existing = if claude_md.exists() {
            fs::read_to_string(&claude_md)
                .with_context(|| format!("Failed to read {}", claude_md.display()))?
        } else {
            String::new()
        };

        let updated = section::inject_markdown_section(&existing, SECTION_ID, &template);
        let result = writer::write_file_atomic_str(&claude_md, &updated)
            .with_context(|| format!("Failed to write {}", claude_md.display()))?;

        let mut files_written = Vec::new();
        let mut messages = Vec::new();

        if result.changed {
            files_written.push(claude_md);
            messages.push(format!(
                "Perfil '{}' inyectado en CLAUDE.md",
                profile.display_name()
            ));
        } else {
            messages.push(format!(
                "Perfil '{}' ya configurado (sin cambios)",
                profile.display_name()
            ));
        }

        Ok(ApplyResult {
            changed: !files_written.is_empty(),
            files_written,
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

    fn make_ctx(tmp: &TempDir, profile: Option<ProfileId>) -> PipelineContext {
        let mut selection = Selection::default();
        selection.behavior_profile = profile;
        PipelineContext {
            paths: ClaudePaths::with_home(tmp.path().to_path_buf()),
            selection,
            dry_run: false,
            backup_manifest: None,
        }
    }

    #[test]
    fn test_apply_injects_profile() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp, Some(ProfileId::Coding));

        let component = BehaviorProfileComponent;
        component.prepare(&ctx).unwrap();
        let result = component.apply(&ctx).unwrap();

        assert!(result.changed);
        let md = fs::read_to_string(ctx.paths.global_claude_md()).unwrap();
        assert!(md.contains("<!-- karma:behavior-profile -->"));
        assert!(md.contains("<!-- profile:coding -->"));
        assert!(md.contains("Code Rules"));
    }

    #[test]
    fn test_apply_none_skips() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp, None);

        let component = BehaviorProfileComponent;
        component.prepare(&ctx).unwrap();
        let result = component.apply(&ctx).unwrap();

        assert!(!result.changed);
    }

    #[test]
    fn test_idempotent() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp, Some(ProfileId::Universal));

        let component = BehaviorProfileComponent;
        component.prepare(&ctx).unwrap();
        component.apply(&ctx).unwrap();
        let result = component.apply(&ctx).unwrap();

        assert!(!result.changed);
    }

    #[test]
    fn test_switch_profile_replaces() {
        let tmp = TempDir::new().unwrap();

        let component = BehaviorProfileComponent;

        // Install coding profile
        let ctx = make_ctx(&tmp, Some(ProfileId::Coding));
        component.prepare(&ctx).unwrap();
        component.apply(&ctx).unwrap();

        // Switch to agents profile
        let ctx = make_ctx(&tmp, Some(ProfileId::Agents));
        let result = component.apply(&ctx).unwrap();

        assert!(result.changed);
        let md = fs::read_to_string(ctx.paths.global_claude_md()).unwrap();
        assert!(md.contains("<!-- profile:agents -->"));
        assert!(!md.contains("<!-- profile:coding -->"));
    }

    #[test]
    fn test_uninstall_removes_section() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp, Some(ProfileId::Universal));

        let component = BehaviorProfileComponent;
        component.prepare(&ctx).unwrap();
        component.apply(&ctx).unwrap();

        let uninstall_ctx = make_ctx(&tmp, None);
        let result = component.uninstall(&uninstall_ctx).unwrap();

        assert!(result.changed);
        let md = fs::read_to_string(uninstall_ctx.paths.global_claude_md()).unwrap();
        assert!(!md.contains("karma:behavior-profile"));
    }

    #[test]
    fn test_preserves_existing_content() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp, Some(ProfileId::Analysis));

        let claude_md = ctx.paths.global_claude_md();
        fs::create_dir_all(claude_md.parent().unwrap()).unwrap();
        fs::write(&claude_md, "# My Rules\n\nCustom content here.\n").unwrap();

        let component = BehaviorProfileComponent;
        component.prepare(&ctx).unwrap();
        component.apply(&ctx).unwrap();

        let content = fs::read_to_string(&claude_md).unwrap();
        assert!(content.contains("# My Rules"));
        assert!(content.contains("Custom content here."));
        assert!(content.contains("<!-- karma:behavior-profile -->"));
    }

    #[test]
    fn test_detect_karma_installed() {
        let tmp = TempDir::new().unwrap();
        let paths = ClaudePaths::with_home(tmp.path().to_path_buf());
        let claude_md = paths.global_claude_md();
        fs::create_dir_all(claude_md.parent().unwrap()).unwrap();
        fs::write(
            &claude_md,
            "# Config\n\n<!-- karma:behavior-profile -->\n<!-- profile:coding -->\nrules\n<!-- /karma:behavior-profile -->\n",
        ).unwrap();

        let status = detect(&paths);
        assert_eq!(status, BehaviorProfileStatus::Installed(ProfileId::Coding));
    }

    #[test]
    fn test_detect_manual() {
        let tmp = TempDir::new().unwrap();
        let paths = ClaudePaths::with_home(tmp.path().to_path_buf());
        let claude_md = paths.global_claude_md();
        fs::create_dir_all(claude_md.parent().unwrap()).unwrap();
        fs::write(
            &claude_md,
            "# Rules\n\n- No sycophantic openers\n- Keep solutions simple and direct\n",
        )
        .unwrap();

        let status = detect(&paths);
        assert_eq!(
            status,
            BehaviorProfileStatus::ManuallyDetected(ProfileId::Universal)
        );
    }

    #[test]
    fn test_detect_not_installed() {
        let tmp = TempDir::new().unwrap();
        let paths = ClaudePaths::with_home(tmp.path().to_path_buf());
        let claude_md = paths.global_claude_md();
        fs::create_dir_all(claude_md.parent().unwrap()).unwrap();
        fs::write(&claude_md, "# My normal CLAUDE.md\n").unwrap();

        let status = detect(&paths);
        assert_eq!(status, BehaviorProfileStatus::NotInstalled);
    }
}
