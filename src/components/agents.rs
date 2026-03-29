use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::components::{ApplyResult, Component, PipelineContext};
use crate::config::paths::ClaudePaths;
use crate::config::types::ConfigScope;
use crate::filemerge::writer;

/// All sub-agent definitions: SDD workflow + general purpose.
const ALL_AGENTS: &[AgentDef] = &[
    // ── SDD workflow agents ──
    AgentDef {
        id: "sdd-explore",
        name: "SDD Explorer",
        description: "Analyzes codebases to discover patterns, architecture, and conventions. Use when exploring unfamiliar code or mapping project structure.",
        tools: "Read, Grep, Glob, Bash",
    },
    AgentDef {
        id: "sdd-propose",
        name: "SDD Proposer",
        description: "Creates change proposals with rationale, scope, and risk assessment. Use after exploration to propose specific changes.",
        tools: "Read, Grep, Glob",
    },
    AgentDef {
        id: "sdd-spec",
        name: "SDD Spec Writer",
        description: "Writes detailed technical specifications from proposals. Defines interfaces, data models, and contracts.",
        tools: "Read, Grep, Glob, Write",
    },
    AgentDef {
        id: "sdd-design",
        name: "SDD Designer",
        description: "Makes architecture and design decisions. Evaluates trade-offs, selects patterns, defines component boundaries.",
        tools: "Read, Grep, Glob",
    },
    AgentDef {
        id: "sdd-tasks",
        name: "SDD Task Planner",
        description: "Breaks specifications into ordered implementation tasks with dependencies and acceptance criteria.",
        tools: "Read, Grep, Glob",
    },
    AgentDef {
        id: "sdd-apply",
        name: "SDD Implementer",
        description: "Implements changes according to specs and task plans. Writes production code following project conventions.",
        tools: "Read, Grep, Glob, Write, Edit, Bash",
    },
    AgentDef {
        id: "sdd-verify",
        name: "SDD Verifier",
        description: "Verifies implementation correctness against specs. Runs tests, checks types, validates behavior.",
        tools: "Read, Grep, Glob, Bash",
    },
    AgentDef {
        id: "sdd-archive",
        name: "SDD Archiver",
        description: "Archives completed changes with summary, lessons learned, and documentation updates.",
        tools: "Read, Grep, Glob, Write",
    },
    // ── General purpose agents ──
    AgentDef {
        id: "code-review",
        name: "Code Reviewer",
        description: "Reviews code changes, PRs, and diffs for correctness, quality, security, and maintainability. Use when reviewing pull requests or evaluating code quality.",
        tools: "Read, Grep, Glob, Bash",
    },
    AgentDef {
        id: "debugger",
        name: "Debugger",
        description: "Diagnoses and fixes bugs by analyzing error messages, stack traces, logs, and code flow. Use when tracking down the root cause of a bug or unexpected behavior.",
        tools: "Read, Grep, Glob, Bash, Edit",
    },
    AgentDef {
        id: "test-writer",
        name: "Test Writer",
        description: "Writes unit tests, integration tests, and e2e tests. Use when adding test coverage or creating test suites for existing code.",
        tools: "Read, Grep, Glob, Write, Edit, Bash",
    },
    AgentDef {
        id: "docs-writer",
        name: "Docs Writer",
        description: "Writes and updates documentation, READMEs, API docs, and inline comments. Use for documentation tasks that don't require deep architectural thinking.",
        tools: "Read, Grep, Glob, Write, Edit",
    },
    AgentDef {
        id: "refactor",
        name: "Refactorer",
        description: "Restructures and improves existing code without changing behavior. Handles renaming, extracting functions, redesigning modules, and improving architecture.",
        tools: "Read, Grep, Glob, Write, Edit, Bash",
    },
    AgentDef {
        id: "searcher",
        name: "Searcher",
        description: "Finds files, patterns, references, symbols, and dependencies across the codebase. Use for quick lookups and codebase navigation.",
        tools: "Read, Grep, Glob",
    },
    AgentDef {
        id: "git-ops",
        name: "Git Operator",
        description: "Handles git operations: commits, branches, merges, rebases, cherry-picks, and PR creation. Use for version control tasks.",
        tools: "Bash",
    },
    AgentDef {
        id: "planner",
        name: "Planner",
        description: "Creates implementation plans, breaks down complex tasks, identifies dependencies, and designs technical approaches. Use before starting complex multi-step work.",
        tools: "Read, Grep, Glob",
    },
];

struct AgentDef {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    tools: &'static str,
}

impl AgentDef {
    /// Render this agent definition as a markdown file with YAML frontmatter.
    /// The model is resolved from the ModelPreset at render time.
    fn render(&self, model_preset: &crate::config::types::ModelPreset) -> String {
        let model = model_preset.model_for_phase(self.id);
        let mut content = String::from("---\n");
        content.push_str(&format!("name: \"{}\"\n", self.name));
        content.push_str(&format!("description: \"{}\"\n", self.description));
        content.push_str(&format!("tools: [{}]\n", self.tools));
        content.push_str(&format!("model: {}\n", model));
        content.push_str("---\n\n");
        content.push_str(&format!("# {}\n\n", self.name));
        content.push_str(self.description);
        content.push('\n');
        content
    }
}

/// Creates sub-agent definition files in Claude Code's agents directory.
pub struct AgentsComponent;

impl Component for AgentsComponent {
    fn id(&self) -> &str {
        "sub-agents"
    }

    fn name(&self) -> &str {
        "Sub-Agents"
    }

    fn affected_paths(&self, paths: &ClaudePaths) -> Vec<PathBuf> {
        ALL_AGENTS
            .iter()
            .map(|a| paths.agent_file(a.id))
            .collect()
    }

    fn prepare(&self, ctx: &PipelineContext) -> Result<()> {
        let agents_dir = self.agents_dir(ctx);
        fs::create_dir_all(&agents_dir)
            .with_context(|| format!("Cannot create agents directory: {}", agents_dir.display()))?;
        Ok(())
    }

    fn apply(&self, ctx: &PipelineContext) -> Result<ApplyResult> {
        let mut files_written = Vec::new();
        let mut any_changed = false;

        for agent in ALL_AGENTS {
            let target = match ctx.selection.scope {
                ConfigScope::User => ctx.paths.agent_file(agent.id),
                ConfigScope::Project => {
                    let cwd = std::env::current_dir().unwrap_or_default();
                    ctx.paths.project_agents_dir(&cwd).join(format!("{}.md", agent.id))
                }
            };

            let content = agent.render(&ctx.selection.model_preset);
            let result = writer::write_file_atomic_str(&target, &content)
                .with_context(|| format!("Failed to write agent: {}", target.display()))?;

            if result.changed {
                any_changed = true;
                files_written.push(target);
            }
        }

        Ok(ApplyResult {
            changed: any_changed,
            files_written,
            messages: vec![format!("{} sub-agent definitions installed", ALL_AGENTS.len())],
        })
    }

    fn rollback(&self, _ctx: &PipelineContext) -> Result<()> {
        Ok(())
    }
}

impl AgentsComponent {
    fn agents_dir(&self, ctx: &PipelineContext) -> PathBuf {
        match ctx.selection.scope {
            ConfigScope::User => ctx.paths.agents_dir(),
            ConfigScope::Project => {
                let cwd = std::env::current_dir().unwrap_or_default();
                ctx.paths.project_agents_dir(&cwd)
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
    fn test_agents_creates_all_files() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        let component = AgentsComponent;
        component.prepare(&ctx).unwrap();
        let result = component.apply(&ctx).unwrap();

        assert!(result.changed);
        assert_eq!(result.files_written.len(), ALL_AGENTS.len());

        // Verify each agent file exists and has correct frontmatter
        for agent in ALL_AGENTS {
            let path = ctx.paths.agent_file(agent.id);
            assert!(path.exists(), "Agent file missing: {}", agent.id);
            let content = fs::read_to_string(&path).unwrap();
            assert!(content.starts_with("---\n"));
            assert!(content.contains(&format!("name: \"{}\"", agent.name)));
        }
    }

    #[test]
    fn test_agents_idempotent() {
        let tmp = TempDir::new().unwrap();
        let ctx = make_ctx(&tmp);

        let component = AgentsComponent;
        component.prepare(&ctx).unwrap();
        component.apply(&ctx).unwrap();

        let result = component.apply(&ctx).unwrap();
        assert!(!result.changed);
    }

    #[test]
    fn test_agent_render_format() {
        use crate::config::types::ModelPreset;
        let agent = &ALL_AGENTS[0]; // sdd-explore
        let content = agent.render(&ModelPreset::Balanced);
        assert!(content.starts_with("---\n"));
        assert!(content.contains("name:"));
        assert!(content.contains("description:"));
        assert!(content.contains("tools:"));
        assert!(content.contains("model: sonnet")); // explore = sonnet in Balanced
        assert!(content.contains("---\n\n#"));
    }

    #[test]
    fn test_agent_model_varies_by_preset() {
        use crate::config::types::ModelPreset;
        let propose = ALL_AGENTS.iter().find(|a| a.id == "sdd-propose").unwrap();

        let balanced = propose.render(&ModelPreset::Balanced);
        assert!(balanced.contains("model: opus"));

        let economy = propose.render(&ModelPreset::Economy);
        assert!(economy.contains("model: sonnet"));
    }

    #[test]
    fn test_agent_archive_always_haiku() {
        use crate::config::types::ModelPreset;
        let archive = ALL_AGENTS.iter().find(|a| a.id == "sdd-archive").unwrap();

        for preset in &[ModelPreset::Balanced, ModelPreset::Performance, ModelPreset::Economy] {
            let content = archive.render(preset);
            assert!(content.contains("model: haiku"), "archive should be haiku in {:?}", preset);
        }
    }
}
