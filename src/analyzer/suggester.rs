use crate::analyzer::detector::ProjectInfo;
use crate::config::types::ComponentId;

/// A suggestion for the user based on project analysis.
#[derive(Debug)]
pub struct Suggestion {
    pub component: ComponentId,
    pub reason: String,
    pub priority: Priority,
}

/// Suggestion priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl Priority {
    pub fn label(&self) -> &'static str {
        match self {
            Priority::High => "Recommended",
            Priority::Medium => "Suggested",
            Priority::Low => "Optional",
        }
    }
}

/// Skill suggestion.
#[derive(Debug)]
pub struct SkillSuggestion {
    pub skill_id: String,
    pub reason: String,
}

/// Full analysis result with suggestions.
#[derive(Debug)]
pub struct AnalysisResult {
    pub info: ProjectInfo,
    pub component_suggestions: Vec<Suggestion>,
    pub skill_suggestions: Vec<SkillSuggestion>,
    pub warnings: Vec<String>,
}

/// Analyze a project and generate suggestions.
pub fn suggest(info: &ProjectInfo) -> AnalysisResult {
    let mut components = Vec::new();
    let mut skills = Vec::new();
    let mut warnings = Vec::new();

    // Orchestrator: always recommended for any project
    if !info.technologies.is_empty() {
        components.push(Suggestion {
            component: ComponentId::Orchestrator,
            reason: "SDD workflow improves code quality for any project".to_string(),
            priority: Priority::High,
        });
    }

    // Permissions: always recommended
    components.push(Suggestion {
        component: ComponentId::Permissions,
        reason: "Security-first defaults prevent accidental destructive operations".to_string(),
        priority: Priority::High,
    });

    // MCP (Context7): recommended when frameworks are detected
    if !info.frameworks.is_empty() {
        let fw_names: Vec<&str> = info.frameworks.iter().map(|f| f.display_name()).collect();
        components.push(Suggestion {
            component: ComponentId::McpServers,
            reason: format!(
                "Context7 provides live docs for {}",
                fw_names.join(", ")
            ),
            priority: Priority::High,
        });
    } else {
        components.push(Suggestion {
            component: ComponentId::McpServers,
            reason: "Context7 provides live framework documentation".to_string(),
            priority: Priority::Medium,
        });
    }

    // Sub-Agents: recommended for larger projects (multiple technologies)
    if info.technologies.len() > 1 || !info.frameworks.is_empty() {
        components.push(Suggestion {
            component: ComponentId::SubAgents,
            reason: "Sub-agents help delegate tasks in complex projects".to_string(),
            priority: Priority::High,
        });
    } else {
        components.push(Suggestion {
            component: ComponentId::SubAgents,
            reason: "Sub-agents enable SDD task delegation".to_string(),
            priority: Priority::Medium,
        });
    }

    // Skills: always suggested
    components.push(Suggestion {
        component: ComponentId::Skills,
        reason: "Slash commands para workflows (branch-pr, issue-creation)".to_string(),
        priority: Priority::Medium,
    });

    // RTK: always recommended for token savings
    components.push(Suggestion {
        component: ComponentId::Rtk,
        reason: "Proxy CLI que ahorra 60-90% de tokens en comandos shell".to_string(),
        priority: Priority::High,
    });

    // Skill suggestions based on tech stack
    skills.push(SkillSuggestion {
        skill_id: "branch-pr".to_string(),
        reason: "PR workflow automation".to_string(),
    });

    skills.push(SkillSuggestion {
        skill_id: "issue-creation".to_string(),
        reason: "Issue creation workflow".to_string(),
    });

    // Warnings about existing configuration
    if info.has_claude_md {
        warnings.push(
            "CLAUDE.md already exists. Orchestrator will inject managed sections without overwriting your content.".to_string()
        );
    }

    if info.has_settings {
        warnings.push(
            ".claude/settings.json exists. Permissions will be merged additively.".to_string()
        );
    }

    if info.has_mcp_config {
        warnings.push(
            ".mcp.json exists. MCP configuration will be merged, not replaced.".to_string()
        );
    }

    // Sort by priority
    components.sort_by(|a, b| a.priority.cmp(&b.priority));

    AnalysisResult {
        info: info.clone(),
        component_suggestions: components,
        skill_suggestions: skills,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::detector::{Framework, Technology};

    #[test]
    fn test_suggest_empty_project() {
        let info = ProjectInfo::default();
        let result = suggest(&info);
        // Should still suggest permissions and basics
        assert!(!result.component_suggestions.is_empty());
        assert!(result
            .component_suggestions
            .iter()
            .any(|s| s.component == ComponentId::Permissions));
    }

    #[test]
    fn test_suggest_rust_project() {
        let info = ProjectInfo {
            technologies: vec![Technology::Rust],
            frameworks: vec![Framework::Axum],
            ..Default::default()
        };
        let result = suggest(&info);

        // Should recommend MCP with Axum context
        let mcp = result
            .component_suggestions
            .iter()
            .find(|s| s.component == ComponentId::McpServers)
            .unwrap();
        assert!(mcp.reason.contains("Axum"));
        assert_eq!(mcp.priority, Priority::High);
    }

    #[test]
    fn test_suggest_complex_project() {
        let info = ProjectInfo {
            technologies: vec![Technology::TypeScript, Technology::Python],
            frameworks: vec![Framework::NextJs, Framework::FastApi],
            ..Default::default()
        };
        let result = suggest(&info);

        // Multi-tech should recommend sub-agents as High
        let agents = result
            .component_suggestions
            .iter()
            .find(|s| s.component == ComponentId::SubAgents)
            .unwrap();
        assert_eq!(agents.priority, Priority::High);
    }

    #[test]
    fn test_warnings_existing_config() {
        let info = ProjectInfo {
            has_claude_md: true,
            has_settings: true,
            has_mcp_config: true,
            ..Default::default()
        };
        let result = suggest(&info);
        assert_eq!(result.warnings.len(), 3);
    }

    #[test]
    fn test_branch_pr_always_suggested() {
        let info = ProjectInfo::default();
        let result = suggest(&info);
        assert!(result
            .skill_suggestions
            .iter()
            .any(|s| s.skill_id == "branch-pr"));
    }
}
