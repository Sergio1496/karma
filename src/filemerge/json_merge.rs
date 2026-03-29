use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value;

/// Strip JavaScript-style comments from a JSON-like string.
///
/// Handles:
/// - Single-line comments: `// comment`
/// - Multi-line comments: `/* comment */`
/// - Trailing commas before `}` or `]`
///
/// This is needed because Claude Code's config files may contain comments
/// despite not being valid JSON.
pub fn strip_json_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_string = false;

    while i < len {
        // Handle string literals (don't strip inside strings)
        if chars[i] == '"' && (i == 0 || chars[i - 1] != '\\') {
            in_string = !in_string;
            result.push(chars[i]);
            i += 1;
            continue;
        }

        if in_string {
            result.push(chars[i]);
            i += 1;
            continue;
        }

        // Single-line comment
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
            // Skip until end of line
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Multi-line comment
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2; // Skip */
            continue;
        }

        result.push(chars[i]);
        i += 1;
    }

    // Strip trailing commas before } or ]
    let re = Regex::new(r",(\s*[}\]])").unwrap();
    re.replace_all(&result, "$1").to_string()
}

/// Deep-merge two JSON objects.
///
/// Rules:
/// - Overlay values override base values for scalar types.
/// - Nested objects are merged recursively.
/// - Arrays from overlay replace base arrays (no concatenation).
/// - A special `"__replace__": true` key in an overlay object forces full replacement
///   of that level instead of deep merging.
pub fn deep_merge(base: &Value, overlay: &Value) -> Value {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            // Check for __replace__ sentinel
            if overlay_map
                .get("__replace__")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                let mut result = overlay_map.clone();
                result.remove("__replace__");
                return Value::Object(result);
            }

            let mut merged = base_map.clone();
            for (key, overlay_val) in overlay_map {
                let merged_val = if let Some(base_val) = base_map.get(key) {
                    deep_merge(base_val, overlay_val)
                } else {
                    overlay_val.clone()
                };
                merged.insert(key.clone(), merged_val);
            }
            Value::Object(merged)
        }
        // For non-object types, overlay wins
        (_, overlay) => overlay.clone(),
    }
}

/// Merge a JSON overlay into a base JSON string.
///
/// Handles commented/malformed base JSON gracefully (falls back to empty object).
pub fn merge_json_str(base: &str, overlay: &str) -> Result<String> {
    let clean_base = strip_json_comments(base);
    let clean_overlay = strip_json_comments(overlay);

    let base_val: Value = if clean_base.trim().is_empty() {
        Value::Object(serde_json::Map::new())
    } else {
        serde_json::from_str(&clean_base).unwrap_or(Value::Object(serde_json::Map::new()))
    };

    let overlay_val: Value =
        serde_json::from_str(&clean_overlay).context("Failed to parse overlay JSON")?;

    let merged = deep_merge(&base_val, &overlay_val);
    serde_json::to_string_pretty(&merged).context("Failed to serialize merged JSON")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_strip_single_line_comments() {
        let input = r#"{
  // This is a comment
  "key": "value"
}"#;
        let result = strip_json_comments(input);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn test_strip_multi_line_comments() {
        let input = r#"{
  /* Multi-line
     comment */
  "key": "value"
}"#;
        let result = strip_json_comments(input);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn test_strip_trailing_commas() {
        let input = r#"{"a": 1, "b": 2,}"#;
        let result = strip_json_comments(input);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["b"], 2);
    }

    #[test]
    fn test_preserve_strings_with_slashes() {
        let input = r#"{"url": "https://example.com"}"#;
        let result = strip_json_comments(input);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["url"], "https://example.com");
    }

    #[test]
    fn test_deep_merge_simple() {
        let base = json!({"a": 1, "b": 2});
        let overlay = json!({"b": 3, "c": 4});
        let result = deep_merge(&base, &overlay);
        assert_eq!(result, json!({"a": 1, "b": 3, "c": 4}));
    }

    #[test]
    fn test_deep_merge_nested() {
        let base = json!({"outer": {"a": 1, "b": 2}});
        let overlay = json!({"outer": {"b": 3, "c": 4}});
        let result = deep_merge(&base, &overlay);
        assert_eq!(result, json!({"outer": {"a": 1, "b": 3, "c": 4}}));
    }

    #[test]
    fn test_deep_merge_replace_sentinel() {
        let base = json!({"nested": {"a": 1, "b": 2}});
        let overlay = json!({"nested": {"__replace__": true, "x": 10}});
        let result = deep_merge(&base, &overlay);
        assert_eq!(result, json!({"nested": {"x": 10}}));
    }

    #[test]
    fn test_deep_merge_arrays_replaced() {
        let base = json!({"arr": [1, 2, 3]});
        let overlay = json!({"arr": [4, 5]});
        let result = deep_merge(&base, &overlay);
        assert_eq!(result, json!({"arr": [4, 5]}));
    }

    #[test]
    fn test_merge_json_str_with_comments() {
        let base = r#"{
  // A comment
  "existing": true
}"#;
        let overlay = r#"{"new_key": "hello"}"#;
        let result = merge_json_str(base, overlay).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["existing"], true);
        assert_eq!(parsed["new_key"], "hello");
    }

    #[test]
    fn test_merge_json_str_empty_base() {
        let result = merge_json_str("", r#"{"key": "value"}"#).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn test_merge_json_str_malformed_base() {
        let result = merge_json_str("not json at all", r#"{"key": "value"}"#).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["key"], "value");
    }
}
