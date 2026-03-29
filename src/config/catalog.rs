use serde::{Deserialize, Serialize};

/// Metadata for a skill in the catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEntry {
    /// Unique skill identifier (e.g., "go-testing", "branch-pr").
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Short description.
    pub description: String,
    /// Category for grouping.
    pub category: SkillCategory,
    /// Relative path in the source repo (e.g., "skills/go-testing/SKILL.md").
    pub source_path: String,
}

/// Skill categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    /// SDD workflow phases.
    Sdd,
    /// Foundation skills (testing, workflow, etc.).
    Foundation,
}

impl SkillCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            SkillCategory::Sdd => "SDD Workflow",
            SkillCategory::Foundation => "Foundation",
        }
    }
}

/// The full skill catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCatalog {
    pub skills: Vec<SkillEntry>,
    /// Base URL for downloading skill files.
    pub base_url: String,
}

impl SkillCatalog {
    /// Get skills by category.
    pub fn by_category(&self, category: SkillCategory) -> Vec<&SkillEntry> {
        self.skills
            .iter()
            .filter(|s| s.category == category)
            .collect()
    }

    /// Find a skill by ID.
    pub fn find(&self, id: &str) -> Option<&SkillEntry> {
        self.skills.iter().find(|s| s.id == id)
    }

    /// Get all skill IDs.
    pub fn skill_ids(&self) -> Vec<&str> {
        self.skills.iter().map(|s| s.id.as_str()).collect()
    }

    /// Build the download URL for a skill.
    pub fn download_url(&self, skill: &SkillEntry) -> String {
        format!("{}/{}", self.base_url, skill.source_path)
    }
}

/// The default base URL for the gentle-ai repository raw files.
const DEFAULT_BASE_URL: &str =
    "https://raw.githubusercontent.com/Gentleman-Programming/gentle-ai/main";

/// Build the default catalog from known gentle-ai skills.
pub fn default_catalog() -> SkillCatalog {
    SkillCatalog {
        base_url: DEFAULT_BASE_URL.to_string(),
        skills: vec![
            skill("branch-pr", "Branch & PR", "Workflow completo de crear branch + PR con convenciones", SkillCategory::Foundation),
            skill("issue-creation", "Issue Creation", "Workflow de crear issues con templates y labels", SkillCategory::Foundation),
        ],
    }
}

fn skill(id: &str, name: &str, description: &str, category: SkillCategory) -> SkillEntry {
    SkillEntry {
        id: id.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        category,
        source_path: format!("skills/{id}/SKILL.md"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_catalog_has_skills() {
        let catalog = default_catalog();
        assert_eq!(catalog.skills.len(), 2);
    }

    #[test]
    fn test_find_skill() {
        let catalog = default_catalog();
        let skill = catalog.find("branch-pr").unwrap();
        assert_eq!(skill.name, "Branch & PR");
        assert_eq!(skill.category, SkillCategory::Foundation);
    }

    #[test]
    fn test_download_url() {
        let catalog = default_catalog();
        let skill = catalog.find("branch-pr").unwrap();
        let url = catalog.download_url(skill);
        assert!(url.starts_with("https://raw.githubusercontent.com"));
        assert!(url.ends_with("skills/branch-pr/SKILL.md"));
    }
}
