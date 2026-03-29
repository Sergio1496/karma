use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Resolves all Claude Code configuration paths.
///
/// Since we only target Claude Code (no multi-agent abstraction),
/// all paths are known constants.
#[derive(Debug, Clone)]
pub struct ClaudePaths {
    home: PathBuf,
}

impl ClaudePaths {
    /// Create a new ClaudePaths using the system home directory.
    pub fn detect() -> Result<Self> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(Self { home })
    }

    /// Create a ClaudePaths with a custom home directory (for testing).
    pub fn with_home(home: PathBuf) -> Self {
        Self { home }
    }

    // ── User-level paths (~/.claude/) ──

    /// The global Claude config directory: ~/.claude/
    pub fn global_config_dir(&self) -> PathBuf {
        self.home.join(".claude")
    }

    /// Global CLAUDE.md: ~/.claude/CLAUDE.md
    pub fn global_claude_md(&self) -> PathBuf {
        self.global_config_dir().join("CLAUDE.md")
    }

    /// Global settings: ~/.claude/settings.json
    pub fn global_settings_json(&self) -> PathBuf {
        self.global_config_dir().join("settings.json")
    }

    /// Skills directory: ~/.claude/skills/
    pub fn skills_dir(&self) -> PathBuf {
        self.global_config_dir().join("skills")
    }

    /// A specific skill directory: ~/.claude/skills/{id}/
    pub fn skill_dir(&self, id: &str) -> PathBuf {
        self.skills_dir().join(id)
    }

    /// A specific skill file: ~/.claude/skills/{id}/SKILL.md
    pub fn skill_file(&self, id: &str) -> PathBuf {
        self.skill_dir(id).join("SKILL.md")
    }

    /// Agents directory: ~/.claude/agents/
    pub fn agents_dir(&self) -> PathBuf {
        self.global_config_dir().join("agents")
    }

    /// A specific agent file: ~/.claude/agents/{name}.md
    pub fn agent_file(&self, name: &str) -> PathBuf {
        self.agents_dir().join(format!("{name}.md"))
    }

    /// User-level MCP config: ~/.claude.json
    pub fn mcp_user_config(&self) -> PathBuf {
        self.home.join(".claude.json")
    }

    /// Hooks directory: ~/.claude/hooks/
    pub fn hooks_dir(&self) -> PathBuf {
        self.global_config_dir().join("hooks")
    }

    // ── Project-level paths (.claude/) ──

    /// Project CLAUDE.md: {root}/CLAUDE.md
    pub fn project_claude_md(&self, root: &Path) -> PathBuf {
        root.join("CLAUDE.md")
    }

    /// Project settings: {root}/.claude/settings.json
    pub fn project_settings_json(&self, root: &Path) -> PathBuf {
        root.join(".claude").join("settings.json")
    }

    /// Project MCP config: {root}/.mcp.json
    pub fn project_mcp_json(&self, root: &Path) -> PathBuf {
        root.join(".mcp.json")
    }

    /// Project skills: {root}/.claude/skills/
    pub fn project_skills_dir(&self, root: &Path) -> PathBuf {
        root.join(".claude").join("skills")
    }

    /// Project agents: {root}/.claude/agents/
    pub fn project_agents_dir(&self, root: &Path) -> PathBuf {
        root.join(".claude").join("agents")
    }

    // ── Tool state paths (~/.karma/) ──

    /// Our tool's state directory: ~/.karma/
    pub fn state_dir(&self) -> PathBuf {
        self.home.join(".karma")
    }

    /// State file: ~/.karma/state.json
    pub fn state_file(&self) -> PathBuf {
        self.state_dir().join("state.json")
    }

    /// Backup directory: ~/.karma/backups/
    pub fn backup_dir(&self) -> PathBuf {
        self.state_dir().join("backups")
    }

    /// Cache directory for remote assets: ~/.karma/cache/
    pub fn cache_dir(&self) -> PathBuf {
        self.state_dir().join("cache")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths_with_custom_home() {
        let paths = ClaudePaths::with_home(PathBuf::from("/home/test"));

        assert_eq!(paths.global_config_dir(), PathBuf::from("/home/test/.claude"));
        assert_eq!(
            paths.global_claude_md(),
            PathBuf::from("/home/test/.claude/CLAUDE.md")
        );
        assert_eq!(
            paths.skill_file("go-testing"),
            PathBuf::from("/home/test/.claude/skills/go-testing/SKILL.md")
        );
        assert_eq!(
            paths.agent_file("sdd-explore"),
            PathBuf::from("/home/test/.claude/agents/sdd-explore.md")
        );
        assert_eq!(
            paths.mcp_user_config(),
            PathBuf::from("/home/test/.claude.json")
        );
        assert_eq!(
            paths.state_dir(),
            PathBuf::from("/home/test/.karma")
        );
    }

    #[test]
    fn test_project_paths() {
        let paths = ClaudePaths::with_home(PathBuf::from("/home/test"));
        let root = Path::new("/projects/myapp");

        assert_eq!(
            paths.project_claude_md(root),
            PathBuf::from("/projects/myapp/CLAUDE.md")
        );
        assert_eq!(
            paths.project_settings_json(root),
            PathBuf::from("/projects/myapp/.claude/settings.json")
        );
        assert_eq!(
            paths.project_mcp_json(root),
            PathBuf::from("/projects/myapp/.mcp.json")
        );
    }
}
