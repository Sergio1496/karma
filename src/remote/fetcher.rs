use anyhow::{Context, Result};

use crate::config::catalog::{SkillCatalog, SkillEntry};
use crate::remote::cache::FileCache;

/// Fetches skill files from a remote source with local caching.
pub struct SkillFetcher {
    cache: FileCache,
    client: reqwest::Client,
}

impl SkillFetcher {
    pub fn new(cache: FileCache) -> Self {
        Self {
            cache,
            client: reqwest::Client::new(),
        }
    }

    /// Fetch a skill's content, using cache when available.
    ///
    /// Returns the SKILL.md content as a string.
    pub async fn fetch_skill(
        &self,
        catalog: &SkillCatalog,
        skill: &SkillEntry,
        force_refresh: bool,
    ) -> Result<String> {
        let cache_key = format!("skill_{}", skill.id);

        // Check cache first (unless force refresh)
        if !force_refresh {
            if let Some(cached) = self.cache.get(&cache_key) {
                return Ok(cached);
            }
        }

        // Download from remote
        let url = catalog.download_url(skill);
        let content = self.download(&url).await.with_context(|| {
            format!("Failed to download skill '{}' from {}", skill.id, url)
        })?;

        // Store in cache
        self.cache.set(&cache_key, &content)?;

        Ok(content)
    }

    /// Fetch multiple skills, returning results for each.
    pub async fn fetch_skills(
        &self,
        catalog: &SkillCatalog,
        skill_ids: &[String],
        force_refresh: bool,
    ) -> Vec<FetchResult> {
        let mut results = Vec::new();

        for skill_id in skill_ids {
            let result = match catalog.find(skill_id) {
                Some(entry) => match self.fetch_skill(catalog, entry, force_refresh).await {
                    Ok(content) => FetchResult {
                        skill_id: skill_id.clone(),
                        content: Some(content),
                        error: None,
                        from_cache: !force_refresh && self.cache.is_valid(&format!("skill_{skill_id}")),
                    },
                    Err(e) => {
                        // Try to fall back to expired cache
                        let fallback = self.cache.get(&format!("skill_{skill_id}"));
                        FetchResult {
                            skill_id: skill_id.clone(),
                            content: fallback,
                            error: Some(e.to_string()),
                            from_cache: true,
                        }
                    }
                },
                None => FetchResult {
                    skill_id: skill_id.clone(),
                    content: None,
                    error: Some(format!("Skill '{skill_id}' not found in catalog")),
                    from_cache: false,
                },
            };
            results.push(result);
        }

        results
    }

    /// Download a URL and return its content as a string.
    async fn download(&self, url: &str) -> Result<String> {
        let response = self
            .client
            .get(url)
            .header("User-Agent", "karma")
            .send()
            .await
            .context("HTTP request failed")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "HTTP {} for {}",
                response.status(),
                url
            );
        }

        response.text().await.context("Failed to read response body")
    }
}

/// Result of fetching a single skill.
#[derive(Debug)]
pub struct FetchResult {
    pub skill_id: String,
    pub content: Option<String>,
    pub error: Option<String>,
    pub from_cache: bool,
}

impl FetchResult {
    pub fn success(&self) -> bool {
        self.content.is_some() && self.error.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::catalog;
    use tempfile::TempDir;

    #[test]
    fn test_fetch_result_success() {
        let r = FetchResult {
            skill_id: "test".to_string(),
            content: Some("content".to_string()),
            error: None,
            from_cache: false,
        };
        assert!(r.success());
    }

    #[test]
    fn test_fetch_result_failure() {
        let r = FetchResult {
            skill_id: "test".to_string(),
            content: None,
            error: Some("not found".to_string()),
            from_cache: false,
        };
        assert!(!r.success());
    }

    #[tokio::test]
    async fn test_fetch_unknown_skill() {
        let tmp = TempDir::new().unwrap();
        let cache = FileCache::new(tmp.path().join("cache"));
        let fetcher = SkillFetcher::new(cache);
        let catalog = catalog::default_catalog();

        let results = fetcher
            .fetch_skills(&catalog, &["nonexistent-skill".to_string()], false)
            .await;

        assert_eq!(results.len(), 1);
        assert!(!results[0].success());
        assert!(results[0].error.as_ref().unwrap().contains("not found in catalog"));
    }

    #[tokio::test]
    async fn test_fetch_uses_cache() {
        let tmp = TempDir::new().unwrap();
        let cache = FileCache::new(tmp.path().join("cache"));

        // Pre-populate cache
        cache.set("skill_branch-pr", "cached content").unwrap();

        let fetcher = SkillFetcher::new(cache);
        let catalog = catalog::default_catalog();

        let result = fetcher
            .fetch_skill(
                &catalog,
                catalog.find("branch-pr").unwrap(),
                false,
            )
            .await
            .unwrap();

        assert_eq!(result, "cached content");
    }
}
