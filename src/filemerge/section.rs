/// Marker-based markdown section injection.
///
/// Uses paired HTML comment markers to manage sections within markdown files
/// without clobbering user-authored content:
///
/// ```markdown
/// <!-- karma:orchestrator -->
/// [managed content here]
/// <!-- /karma:orchestrator -->
/// ```
///
/// Three operations:
/// 1. **Replace**: If markers exist, replace content between them.
/// 2. **Append**: If markers don't exist, append new marked section.
/// 3. **Remove**: If content is empty, remove the entire marked block.

const MARKER_PREFIX: &str = "karma";

/// Build the opening marker for a section.
fn open_marker(section_id: &str) -> String {
    format!("<!-- {MARKER_PREFIX}:{section_id} -->")
}

/// Build the closing marker for a section.
fn close_marker(section_id: &str) -> String {
    format!("<!-- /{MARKER_PREFIX}:{section_id} -->")
}

/// Inject or replace a managed section in a markdown document.
///
/// - If `content` is empty, the existing section (if any) is removed.
/// - If markers already exist, the content between them is replaced.
/// - If markers don't exist, a new section is appended at the end.
///
/// Returns the modified document content.
pub fn inject_markdown_section(existing: &str, section_id: &str, content: &str) -> String {
    let open = open_marker(section_id);
    let close = close_marker(section_id);

    let open_pos = existing.find(&open);
    let close_pos = existing.find(&close);

    match (open_pos, close_pos, content.is_empty()) {
        // Remove: markers exist and content is empty
        (Some(start), Some(end), true) => {
            let before = existing[..start].trim_end_matches('\n');
            let after_end = end + close.len();
            let after = existing[after_end..].trim_start_matches('\n');

            let mut result = String::new();
            if !before.is_empty() {
                result.push_str(before);
                if !after.is_empty() {
                    result.push_str("\n\n");
                }
            }
            if !after.is_empty() {
                result.push_str(after);
            }
            result
        }

        // Replace: markers exist in correct order and content is non-empty
        (Some(start), Some(end), false) if start < end => {
            let before = &existing[..start];
            let after_end = end + close.len();
            let after = &existing[after_end..];

            format!("{before}{open}\n{content}\n{close}{after}")
        }

        // Append: markers don't exist (or are malformed) and content is non-empty
        (_, _, false) => {
            let mut result = existing.to_string();
            // Ensure proper spacing before appending
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&open);
            result.push('\n');
            result.push_str(content);
            result.push('\n');
            result.push_str(&close);
            result.push('\n');
            result
        }

        // Remove but no markers exist: no-op
        (_, _, true) => existing.to_string(),
    }
}

/// Check if a document contains a managed section with the given ID.
pub fn has_section(document: &str, section_id: &str) -> bool {
    let open = open_marker(section_id);
    let close = close_marker(section_id);
    document.contains(&open) && document.contains(&close)
}

/// Extract the content of a managed section (without markers).
pub fn extract_section(document: &str, section_id: &str) -> Option<String> {
    let open = open_marker(section_id);
    let close = close_marker(section_id);

    let start = document.find(&open)?;
    let end = document.find(&close)?;
    if start >= end {
        return None;
    }

    let content_start = start + open.len();
    let content = document[content_start..end].trim();
    Some(content.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_into_empty_document() {
        let result = inject_markdown_section("", "orchestrator", "Hello World");
        assert_eq!(
            result,
            "<!-- karma:orchestrator -->\nHello World\n<!-- /karma:orchestrator -->\n"
        );
    }

    #[test]
    fn test_inject_appends_to_existing_content() {
        let existing = "# My CLAUDE.md\n\nSome user content here.\n";
        let result = inject_markdown_section(existing, "orchestrator", "Managed content");
        assert!(result.starts_with("# My CLAUDE.md\n\nSome user content here.\n"));
        assert!(result.contains("<!-- karma:orchestrator -->"));
        assert!(result.contains("Managed content"));
        assert!(result.contains("<!-- /karma:orchestrator -->"));
    }

    #[test]
    fn test_replace_existing_section() {
        let existing = "Before\n\n<!-- karma:test -->\nOld content\n<!-- /karma:test -->\n\nAfter";
        let result = inject_markdown_section(existing, "test", "New content");
        assert!(result.contains("New content"));
        assert!(!result.contains("Old content"));
        assert!(result.contains("Before"));
        assert!(result.contains("After"));
    }

    #[test]
    fn test_remove_section_with_empty_content() {
        let existing =
            "Before\n\n<!-- karma:test -->\nSome content\n<!-- /karma:test -->\n\nAfter";
        let result = inject_markdown_section(existing, "test", "");
        assert!(!result.contains("karma:test"));
        assert!(!result.contains("Some content"));
        assert!(result.contains("Before"));
        assert!(result.contains("After"));
    }

    #[test]
    fn test_remove_nonexistent_section_is_noop() {
        let existing = "Some content";
        let result = inject_markdown_section(existing, "nonexistent", "");
        assert_eq!(result, existing);
    }

    #[test]
    fn test_multiple_sections_coexist() {
        let doc = inject_markdown_section("", "section-a", "Content A");
        let doc = inject_markdown_section(&doc, "section-b", "Content B");
        assert!(has_section(&doc, "section-a"));
        assert!(has_section(&doc, "section-b"));
        assert_eq!(extract_section(&doc, "section-a").unwrap(), "Content A");
        assert_eq!(extract_section(&doc, "section-b").unwrap(), "Content B");
    }

    #[test]
    fn test_idempotent_injection() {
        let doc = inject_markdown_section("", "test", "Content");
        let doc2 = inject_markdown_section(&doc, "test", "Content");
        assert_eq!(doc, doc2);
    }

    #[test]
    fn test_has_section() {
        let doc = "<!-- karma:foo -->\nbar\n<!-- /karma:foo -->";
        assert!(has_section(doc, "foo"));
        assert!(!has_section(doc, "baz"));
    }

    #[test]
    fn test_extract_section() {
        let doc = "before\n<!-- karma:test -->\ninner content\n<!-- /karma:test -->\nafter";
        assert_eq!(extract_section(doc, "test").unwrap(), "inner content");
        assert!(extract_section(doc, "missing").is_none());
    }
}
