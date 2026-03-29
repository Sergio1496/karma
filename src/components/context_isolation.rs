use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::{json, Value};

use crate::assets;
use crate::components::{ApplyResult, Component, PipelineContext};
use crate::config::paths::ClaudePaths;
use crate::config::types::ConfigScope;
use crate::filemerge::{json_merge, section, writer};

const SECTION_ID: &str = "context-isolation";
const HOOK_MATCHER: &str = "Grep|Glob|Read";
const HOOK_COMMAND: &str = "karma hook-guard";

/// Max Read calls per session before nudging toward Explore subagent.
const READ_NUDGE_THRESHOLD: u32 = 3;

// ── Hook Guard Evaluation (public for testing) ──

/// Evaluate a PreToolUse hook call. Returns Some(json) to output, or None to allow silently.
pub fn evaluate_hook(tool_name: &str, tool_input: &Value, session_id: &str) -> Option<Value> {
    match tool_name {
        "Grep" | "Glob" => evaluate_search(tool_input),
        "Read" => evaluate_read(session_id),
        _ => None,
    }
}

/// Grep/Glob: deny if the search is broad (no specific path AND recursive/missing glob).
fn evaluate_search(tool_input: &Value) -> Option<Value> {
    let path = tool_input["path"].as_str().unwrap_or("");
    let glob_pattern = tool_input["glob"].as_str().unwrap_or("");
    let pattern = tool_input["pattern"].as_str().unwrap_or("");

    // Specific path (not root) → targeted, allow
    if !path.is_empty() && path != "." && path != "./" {
        return None;
    }

    // Broad path but narrow glob (no recursive **) → allow
    // e.g. "*.rs" searches only CWD level, not recursive
    if !glob_pattern.is_empty() && !glob_pattern.contains("**") {
        return None;
    }

    // Glob tool with a pattern that includes a directory prefix → targeted
    // e.g. pattern = "src/components/**/*.rs" has a clear scope
    if !pattern.is_empty() && (pattern.contains('/') || pattern.contains('\\')) {
        let first_segment = pattern.split(&['/', '\\'][..]).next().unwrap_or("");
        // If the first segment is NOT a wildcard, the pattern is scoped
        if !first_segment.is_empty() && !first_segment.contains('*') {
            return None;
        }
    }

    // Broad search → deny
    Some(json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": "Context Isolation: busqueda amplia sin path especifico.",
            "additionalContext": "CONTEXT ISOLATION: Esta busqueda cubre todo el codebase y contaminaria el contexto principal. Usa el Agent tool (subagent_type=\"Explore\") para ejecutar esta busqueda en un contexto desechable. El subagente devolvera solo un resumen compacto. Ejemplo: Agent(subagent_type=\"Explore\", prompt=\"Busca X en el codebase. Devuelve: rutas, lineas, y resumen de 5 bullets max.\")"
        }
    }))
}

/// Read: always allow, but nudge after READ_NUDGE_THRESHOLD reads per session.
fn evaluate_read(session_id: &str) -> Option<Value> {
    if session_id.is_empty() {
        return None; // No session tracking possible
    }

    let count = increment_read_counter(session_id);

    if count <= READ_NUDGE_THRESHOLD {
        return None; // Under threshold, allow silently
    }

    // Allow but add context nudge
    Some(json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": format!(
                "CONTEXT ISOLATION: Has leido {} archivos en esta sesion. \
                 Considera delegar el resto de la investigacion a un subagente \
                 Explore (Agent tool, subagent_type=\"Explore\") para mantener \
                 limpio el contexto principal.",
                count
            )
        }
    }))
}

/// Path to the session read counter temp file.
pub fn read_counter_path(session_id: &str) -> PathBuf {
    std::env::temp_dir().join(format!("karma-ctx-iso-{session_id}"))
}

/// Increment and return the read counter for this session.
fn increment_read_counter(session_id: &str) -> u32 {
    let path = read_counter_path(session_id);
    let current = fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0);
    let new_count = current + 1;
    let _ = fs::write(&path, new_count.to_string());
    new_count
}

/// Injects context isolation into Claude Code via two mechanisms:
///
/// 1. **CLAUDE.md section** — behavioral rules telling Claude when to delegate to subagents
/// 2. **settings.json hook** — PreToolUse hook that intercepts broad Grep/Glob and denies them,
///    forcing Claude to use Explore subagents instead
pub struct ContextIsolationComponent;

impl ContextIsolationComponent {
    fn claude_md_path(&self, ctx: &PipelineContext) -> PathBuf {
        match ctx.selection.scope {
            ConfigScope::User => ctx.paths.global_claude_md(),
            ConfigScope::Project => {
                let cwd = std::env::current_dir().unwrap_or_default();
                ctx.paths.project_claude_md(&cwd)
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

    /// Build the hook entry JSON for PreToolUse.
    fn hook_entry() -> Value {
        json!({
            "matcher": HOOK_MATCHER,
            "hooks": [
                {
                    "type": "command",
                    "command": HOOK_COMMAND
                }
            ]
        })
    }

    /// Check if our hook is already present in the PreToolUse array.
    fn has_hook(settings: &Value) -> bool {
        settings["hooks"]["PreToolUse"]
            .as_array()
            .map(|arr| {
                arr.iter().any(|entry| {
                    entry["hooks"]
                        .as_array()
                        .map(|hooks| {
                            hooks.iter().any(|h| {
                                h["command"].as_str() == Some(HOOK_COMMAND)
                            })
                        })
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    }

    /// Inject our hook entry into settings.json without clobbering existing hooks.
    /// Appends to the PreToolUse array instead of replacing it.
    fn inject_hook(&self, ctx: &PipelineContext) -> Result<bool> {
        let settings_path = self.settings_path(ctx);

        let existing_str = if settings_path.exists() {
            fs::read_to_string(&settings_path)
                .with_context(|| format!("Failed to read {}", settings_path.display()))?
        } else {
            "{}".to_string()
        };

        let clean = json_merge::strip_json_comments(&existing_str);
        let mut settings: Value = if clean.trim().is_empty() {
            json!({})
        } else {
            serde_json::from_str(&clean).unwrap_or(json!({}))
        };

        if Self::has_hook(&settings) {
            return Ok(false); // Already installed, idempotent
        }

        // Ensure hooks.PreToolUse array exists
        if settings.get("hooks").is_none() {
            settings["hooks"] = json!({});
        }
        if settings["hooks"].get("PreToolUse").is_none() {
            settings["hooks"]["PreToolUse"] = json!([]);
        }

        // Append our entry to the existing array
        settings["hooks"]["PreToolUse"]
            .as_array_mut()
            .unwrap()
            .push(Self::hook_entry());

        let output = serde_json::to_string_pretty(&settings)
            .context("Failed to serialize settings")?;

        writer::write_file_atomic_str(&settings_path, &output)
            .with_context(|| format!("Failed to write {}", settings_path.display()))?;

        Ok(true)
    }

    /// Full uninstall: remove hook from settings.json + section from CLAUDE.md.
    pub fn uninstall(&self, ctx: &PipelineContext) -> Result<ApplyResult> {
        let mut messages = Vec::new();

        // 1. Remove hook from settings.json
        let hook_removed = self.remove_hook(ctx)?;
        if hook_removed {
            messages.push("Hook context-isolation eliminado de settings.json".to_string());
        }

        // 2. Remove section from CLAUDE.md
        let claude_md = self.claude_md_path(ctx);
        if claude_md.exists() {
            let content = fs::read_to_string(&claude_md)?;
            if content.contains(&format!("<!-- karma:{SECTION_ID} -->")) {
                let cleaned = section::inject_markdown_section(&content, SECTION_ID, "");
                writer::write_file_atomic_str(&claude_md, &cleaned)?;
                messages.push("Seccion context-isolation eliminada de CLAUDE.md".to_string());
            }
        }

        // 3. Clean up any leftover session counter files
        let temp_dir = std::env::temp_dir();
        if let Ok(entries) = fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                if entry
                    .file_name()
                    .to_str()
                    .map(|n| n.starts_with("karma-ctx-iso-"))
                    .unwrap_or(false)
                {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }

        if messages.is_empty() {
            messages.push("Context isolation no estaba configurado".to_string());
        }

        Ok(ApplyResult {
            changed: !messages.is_empty(),
            files_written: vec![],
            messages,
        })
    }

    /// Remove our hook from settings.json.
    pub fn remove_hook(&self, ctx: &PipelineContext) -> Result<bool> {
        let settings_path = self.settings_path(ctx);

        if !settings_path.exists() {
            return Ok(false);
        }

        let existing_str = fs::read_to_string(&settings_path)
            .with_context(|| format!("Failed to read {}", settings_path.display()))?;

        let clean = json_merge::strip_json_comments(&existing_str);
        let mut settings: Value = serde_json::from_str(&clean).unwrap_or(json!({}));

        if !Self::has_hook(&settings) {
            return Ok(false);
        }

        // Filter out our hook entry from PreToolUse
        if let Some(arr) = settings["hooks"]["PreToolUse"].as_array_mut() {
            arr.retain(|entry| {
                !entry["hooks"]
                    .as_array()
                    .map(|hooks| {
                        hooks.iter().any(|h| h["command"].as_str() == Some(HOOK_COMMAND))
                    })
                    .unwrap_or(false)
            });
        }

        let output = serde_json::to_string_pretty(&settings)
            .context("Failed to serialize settings")?;

        writer::write_file_atomic_str(&settings_path, &output)
            .with_context(|| format!("Failed to write {}", settings_path.display()))?;

        Ok(true)
    }
}

impl Component for ContextIsolationComponent {
    fn id(&self) -> &str {
        "context-isolation"
    }

    fn name(&self) -> &str {
        "Context Isolation"
    }

    fn affected_paths(&self, paths: &ClaudePaths) -> Vec<PathBuf> {
        vec![paths.global_claude_md(), paths.global_settings_json()]
    }

    fn prepare(&self, ctx: &PipelineContext) -> Result<()> {
        // Ensure parent dirs exist for both targets
        for path in [self.claude_md_path(ctx), self.settings_path(ctx)] {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Cannot create directory: {}", parent.display()))?;
            }
        }
        Ok(())
    }

    fn apply(&self, ctx: &PipelineContext) -> Result<ApplyResult> {
        let mut files_written = Vec::new();
        let mut messages = Vec::new();

        // 1. Inject CLAUDE.md section (behavioral rules)
        let claude_md = self.claude_md_path(ctx);
        let template = assets::context_isolation_template();

        let existing = if claude_md.exists() {
            fs::read_to_string(&claude_md)
                .with_context(|| format!("Failed to read {}", claude_md.display()))?
        } else {
            String::new()
        };

        let updated = section::inject_markdown_section(&existing, SECTION_ID, &template);
        let md_result = writer::write_file_atomic_str(&claude_md, &updated)
            .with_context(|| format!("Failed to write {}", claude_md.display()))?;

        if md_result.changed {
            files_written.push(claude_md);
            messages.push("Context isolation rules injected into CLAUDE.md".to_string());
        }

        // 2. Inject PreToolUse hook into settings.json
        let hook_changed = self.inject_hook(ctx)?;
        if hook_changed {
            files_written.push(self.settings_path(ctx));
            messages.push("PreToolUse hook for context isolation added to settings.json".to_string());
        }

        if messages.is_empty() {
            messages.push("Context isolation already configured (no changes)".to_string());
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

    fn make_ctx(tmp: &TempDir) -> PipelineContext {
        PipelineContext {
            paths: ClaudePaths::with_home(tmp.path().to_path_buf()),
            selection: Selection::default(),
            dry_run: false,
            backup_manifest: None,
        }
    }

    #[test]
    fn test_injects_claude_md_and_hook() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        let component = ContextIsolationComponent;
        component.prepare(&ctx).unwrap();
        let result = component.apply(&ctx).unwrap();

        assert!(result.changed);
        assert_eq!(result.files_written.len(), 2);

        // Verify CLAUDE.md
        let md = fs::read_to_string(ctx.paths.global_claude_md()).unwrap();
        assert!(md.contains("<!-- karma:context-isolation -->"));
        assert!(md.contains("Context Isolation"));

        // Verify settings.json hook
        let settings_str = fs::read_to_string(ctx.paths.global_settings_json()).unwrap();
        let settings: Value = serde_json::from_str(&settings_str).unwrap();
        assert!(ContextIsolationComponent::has_hook(&settings));

        let hook = &settings["hooks"]["PreToolUse"][0];
        assert_eq!(hook["matcher"], "Grep|Glob|Read");
        assert_eq!(hook["hooks"][0]["command"], "karma hook-guard");
    }

    #[test]
    fn test_idempotent() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        let component = ContextIsolationComponent;
        component.prepare(&ctx).unwrap();
        component.apply(&ctx).unwrap();
        let result = component.apply(&ctx).unwrap();

        assert!(!result.changed);

        // Verify hook was NOT duplicated
        let settings_str = fs::read_to_string(ctx.paths.global_settings_json()).unwrap();
        let settings: Value = serde_json::from_str(&settings_str).unwrap();
        assert_eq!(settings["hooks"]["PreToolUse"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_preserves_existing_hooks() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        // Pre-populate settings.json with an existing RTK hook
        let settings_path = ctx.paths.global_settings_json();
        fs::create_dir_all(settings_path.parent().unwrap()).unwrap();
        let existing = json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Bash",
                        "hooks": [{"type": "command", "command": "rtk hook claude"}]
                    }
                ]
            }
        });
        fs::write(&settings_path, serde_json::to_string_pretty(&existing).unwrap()).unwrap();

        let component = ContextIsolationComponent;
        component.prepare(&ctx).unwrap();
        component.apply(&ctx).unwrap();

        let settings_str = fs::read_to_string(&settings_path).unwrap();
        let settings: Value = serde_json::from_str(&settings_str).unwrap();
        let hooks = settings["hooks"]["PreToolUse"].as_array().unwrap();

        // Both hooks should be present
        assert_eq!(hooks.len(), 2);
        assert_eq!(hooks[0]["hooks"][0]["command"], "rtk hook claude");
        assert_eq!(hooks[1]["hooks"][0]["command"], "karma hook-guard");
    }

    #[test]
    fn test_remove_hook() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        let component = ContextIsolationComponent;
        component.prepare(&ctx).unwrap();
        component.apply(&ctx).unwrap();

        // Verify hook exists
        assert!(ContextIsolationComponent::has_hook(
            &serde_json::from_str::<Value>(
                &fs::read_to_string(ctx.paths.global_settings_json()).unwrap()
            ).unwrap()
        ));

        // Remove it
        let removed = component.remove_hook(&ctx).unwrap();
        assert!(removed);

        // Verify it's gone
        let settings_str = fs::read_to_string(ctx.paths.global_settings_json()).unwrap();
        let settings: Value = serde_json::from_str(&settings_str).unwrap();
        assert!(!ContextIsolationComponent::has_hook(&settings));
        assert_eq!(settings["hooks"]["PreToolUse"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_remove_preserves_other_hooks() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        // Set up with RTK + context isolation hooks
        let settings_path = ctx.paths.global_settings_json();
        fs::create_dir_all(settings_path.parent().unwrap()).unwrap();
        let existing = json!({
            "hooks": {
                "PreToolUse": [
                    {"matcher": "Bash", "hooks": [{"type": "command", "command": "rtk hook claude"}]},
                    {"matcher": "Grep|Glob", "hooks": [{"type": "command", "command": "karma hook-guard"}]}
                ]
            }
        });
        fs::write(&settings_path, serde_json::to_string_pretty(&existing).unwrap()).unwrap();

        let component = ContextIsolationComponent;
        component.remove_hook(&ctx).unwrap();

        let settings_str = fs::read_to_string(&settings_path).unwrap();
        let settings: Value = serde_json::from_str(&settings_str).unwrap();
        let hooks = settings["hooks"]["PreToolUse"].as_array().unwrap();

        // Only RTK hook should remain
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0]["hooks"][0]["command"], "rtk hook claude");
    }

    #[test]
    fn test_preserves_existing_claude_md() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        let claude_md = ctx.paths.global_claude_md();
        fs::create_dir_all(claude_md.parent().unwrap()).unwrap();
        fs::write(&claude_md, "# My Rules\n\nCustom content here.\n").unwrap();

        let component = ContextIsolationComponent;
        component.prepare(&ctx).unwrap();
        component.apply(&ctx).unwrap();

        let content = fs::read_to_string(&claude_md).unwrap();
        assert!(content.contains("# My Rules"));
        assert!(content.contains("Custom content here."));
        assert!(content.contains("<!-- karma:context-isolation -->"));
    }

    #[test]
    fn test_coexists_with_other_md_sections() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        let claude_md = ctx.paths.global_claude_md();
        fs::create_dir_all(claude_md.parent().unwrap()).unwrap();
        fs::write(
            &claude_md,
            "# Config\n\n<!-- karma:model-routing -->\nRouting\n<!-- /karma:model-routing -->\n",
        )
        .unwrap();

        let component = ContextIsolationComponent;
        component.prepare(&ctx).unwrap();
        component.apply(&ctx).unwrap();

        let content = fs::read_to_string(&claude_md).unwrap();
        assert!(content.contains("<!-- karma:model-routing -->"));
        assert!(content.contains("<!-- karma:context-isolation -->"));
    }

    // ── evaluate_hook tests ──

    #[test]
    fn test_grep_no_path_no_glob_denied() {
        let input = json!({"pattern": "TODO"});
        let result = evaluate_hook("Grep", &input, "test-session");
        assert!(result.is_some());
        let out = result.unwrap();
        assert_eq!(out["hookSpecificOutput"]["permissionDecision"], "deny");
    }

    #[test]
    fn test_grep_with_specific_path_allowed() {
        let input = json!({"pattern": "TODO", "path": "src/components"});
        let result = evaluate_hook("Grep", &input, "test-session");
        assert!(result.is_none());
    }

    #[test]
    fn test_grep_dot_path_denied() {
        let input = json!({"pattern": "TODO", "path": "."});
        let result = evaluate_hook("Grep", &input, "test-session");
        assert!(result.is_some());
    }

    #[test]
    fn test_grep_no_path_but_narrow_glob_allowed() {
        // "*.rs" has no ** → only current dir, not recursive → allow
        let input = json!({"pattern": "TODO", "glob": "*.rs"});
        let result = evaluate_hook("Grep", &input, "test-session");
        assert!(result.is_none());
    }

    #[test]
    fn test_grep_no_path_recursive_glob_denied() {
        // "**/*.rs" is recursive → broad → deny
        let input = json!({"pattern": "TODO", "glob": "**/*.rs"});
        let result = evaluate_hook("Grep", &input, "test-session");
        assert!(result.is_some());
    }

    #[test]
    fn test_glob_scoped_pattern_allowed() {
        // Pattern "src/components/**/*.rs" starts with a concrete dir → scoped
        let input = json!({"pattern": "src/components/**/*.rs"});
        let result = evaluate_hook("Glob", &input, "test-session");
        assert!(result.is_none());
    }

    #[test]
    fn test_glob_wildcard_pattern_denied() {
        // Pattern "**/*.ts" with no path → broad
        let input = json!({"pattern": "**/*.ts"});
        let result = evaluate_hook("Glob", &input, "test-session");
        assert!(result.is_some());
    }

    #[test]
    fn test_read_under_threshold_allowed_silently() {
        // Use a unique session ID to avoid leaking state between tests
        let session = format!("test-read-under-{}", std::process::id());
        let counter_path = read_counter_path(&session);

        // Clean up any leftover counter
        let _ = fs::remove_file(&counter_path);

        let input = json!({"file_path": "/some/file.rs"});
        for _ in 0..READ_NUDGE_THRESHOLD {
            let result = evaluate_hook("Read", &input, &session);
            assert!(result.is_none(), "Should allow silently under threshold");
        }

        let _ = fs::remove_file(&counter_path);
    }

    #[test]
    fn test_read_over_threshold_nudges() {
        let session = format!("test-read-over-{}", std::process::id());
        let counter_path = read_counter_path(&session);
        let _ = fs::remove_file(&counter_path);

        let input = json!({"file_path": "/some/file.rs"});

        // Exhaust threshold
        for _ in 0..READ_NUDGE_THRESHOLD {
            evaluate_hook("Read", &input, &session);
        }

        // Next read should nudge (allow + additionalContext)
        let result = evaluate_hook("Read", &input, &session);
        assert!(result.is_some());
        let out = result.unwrap();
        assert_eq!(out["hookSpecificOutput"]["permissionDecision"], "allow");
        assert!(out["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap()
            .contains("CONTEXT ISOLATION"));

        let _ = fs::remove_file(&counter_path);
    }

    #[test]
    fn test_read_no_session_id_always_allows() {
        let input = json!({"file_path": "/some/file.rs"});
        let result = evaluate_hook("Read", &input, "");
        assert!(result.is_none());
    }

    #[test]
    fn test_unknown_tool_always_allowed() {
        let input = json!({"command": "ls"});
        let result = evaluate_hook("Bash", &input, "test-session");
        assert!(result.is_none());
    }
}
