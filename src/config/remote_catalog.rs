use anyhow::{Context, Result};
use serde::Deserialize;

use crate::config::catalog::{SkillCatalog, SkillCategory, SkillEntry};

/// Default GitHub API URL for listing skills in gentle-ai.
const GITHUB_API_SKILLS_URL: &str =
    "https://api.github.com/repos/Gentleman-Programming/gentle-ai/contents/skills";

/// Default raw content base URL.
const RAW_BASE_URL: &str =
    "https://raw.githubusercontent.com/Gentleman-Programming/gentle-ai/main";

/// GitHub API response for directory listing.
#[derive(Debug, Deserialize)]
struct GitHubEntry {
    name: String,
    #[serde(rename = "type")]
    entry_type: String,
}

/// Fetch the skill catalog from the gentle-ai GitHub repository.
///
/// Discovers skills by listing the `skills/` directory via the GitHub API,
/// then builds a catalog with download URLs.
pub async fn fetch_remote_catalog() -> Result<SkillCatalog> {
    let client = reqwest::Client::new();

    let response = client
        .get(GITHUB_API_SKILLS_URL)
        .header("User-Agent", "karma")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .context("Failed to fetch skill list from GitHub")?;

    if !response.status().is_success() {
        anyhow::bail!(
            "GitHub API returned {} when listing skills",
            response.status()
        );
    }

    let entries: Vec<GitHubEntry> = response
        .json()
        .await
        .context("Failed to parse GitHub API response")?;

    let skills: Vec<SkillEntry> = entries
        .into_iter()
        .filter(|e| e.entry_type == "dir")
        .map(|e| {
            let category = if e.name.starts_with("sdd-") {
                SkillCategory::Sdd
            } else {
                SkillCategory::Foundation
            };

            SkillEntry {
                id: e.name.clone(),
                name: humanize_skill_name(&e.name),
                description: format!("Skill: {}", e.name),
                category,
                source_path: format!("skills/{}/SKILL.md", e.name),
            }
        })
        .collect();

    Ok(SkillCatalog {
        base_url: RAW_BASE_URL.to_string(),
        skills,
    })
}

/// Convert a skill ID like "go-testing" to "Go Testing".
fn humanize_skill_name(id: &str) -> String {
    id.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    upper + chars.as_str()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanize_skill_name() {
        assert_eq!(humanize_skill_name("go-testing"), "Go Testing");
        assert_eq!(humanize_skill_name("sdd-init"), "Sdd Init");
        assert_eq!(humanize_skill_name("branch-pr"), "Branch Pr");
        assert_eq!(humanize_skill_name("single"), "Single");
    }
}
