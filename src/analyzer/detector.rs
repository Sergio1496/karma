use std::path::Path;

use serde::{Deserialize, Serialize};

/// Detected technology in a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Technology {
    Rust,
    Go,
    TypeScript,
    JavaScript,
    Python,
    Java,
    CSharp,
    Ruby,
    Php,
    Swift,
    Kotlin,
    Dart,
    Elixir,
}

impl Technology {
    pub fn display_name(&self) -> &'static str {
        match self {
            Technology::Rust => "Rust",
            Technology::Go => "Go",
            Technology::TypeScript => "TypeScript",
            Technology::JavaScript => "JavaScript",
            Technology::Python => "Python",
            Technology::Java => "Java",
            Technology::CSharp => "C#",
            Technology::Ruby => "Ruby",
            Technology::Php => "PHP",
            Technology::Swift => "Swift",
            Technology::Kotlin => "Kotlin",
            Technology::Dart => "Dart/Flutter",
            Technology::Elixir => "Elixir",
        }
    }
}

/// Detected framework or tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Framework {
    React,
    NextJs,
    Vue,
    Angular,
    Svelte,
    Express,
    NestJs,
    Django,
    FastApi,
    Flask,
    Spring,
    Rails,
    Laravel,
    Actix,
    Axum,
    Gin,
    Flutter,
    Tauri,
}

impl Framework {
    pub fn display_name(&self) -> &'static str {
        match self {
            Framework::React => "React",
            Framework::NextJs => "Next.js",
            Framework::Vue => "Vue",
            Framework::Angular => "Angular",
            Framework::Svelte => "Svelte",
            Framework::Express => "Express",
            Framework::NestJs => "NestJS",
            Framework::Django => "Django",
            Framework::FastApi => "FastAPI",
            Framework::Flask => "Flask",
            Framework::Spring => "Spring",
            Framework::Rails => "Rails",
            Framework::Laravel => "Laravel",
            Framework::Actix => "Actix",
            Framework::Axum => "Axum",
            Framework::Gin => "Gin",
            Framework::Flutter => "Flutter",
            Framework::Tauri => "Tauri",
        }
    }
}

/// Result of analyzing a project directory.
#[derive(Debug, Clone, Default)]
pub struct ProjectInfo {
    /// Primary languages detected.
    pub technologies: Vec<Technology>,
    /// Frameworks detected.
    pub frameworks: Vec<Framework>,
    /// Whether Claude Code is already configured.
    pub has_claude_config: bool,
    /// Whether a CLAUDE.md file exists.
    pub has_claude_md: bool,
    /// Whether .claude/settings.json exists.
    pub has_settings: bool,
    /// Whether there are existing skills.
    pub has_skills: bool,
    /// Whether there are existing agents.
    pub has_agents: bool,
    /// Whether an MCP config exists.
    pub has_mcp_config: bool,
}

/// Analyze a project directory and detect its tech stack.
pub fn detect(root: &Path) -> ProjectInfo {
    let mut info = ProjectInfo::default();

    // Detect languages by marker files
    let markers: &[(&str, Technology)] = &[
        ("Cargo.toml", Technology::Rust),
        ("go.mod", Technology::Go),
        ("tsconfig.json", Technology::TypeScript),
        ("package.json", Technology::JavaScript),
        ("pyproject.toml", Technology::Python),
        ("requirements.txt", Technology::Python),
        ("setup.py", Technology::Python),
        ("pom.xml", Technology::Java),
        ("build.gradle", Technology::Java),
        ("build.gradle.kts", Technology::Kotlin),
        ("*.csproj", Technology::CSharp),
        ("Gemfile", Technology::Ruby),
        ("composer.json", Technology::Php),
        ("Package.swift", Technology::Swift),
        ("pubspec.yaml", Technology::Dart),
        ("mix.exs", Technology::Elixir),
    ];

    for (file, tech) in markers {
        if file.starts_with('*') {
            // Glob pattern - check if any matching file exists
            let ext = &file[1..];
            if has_file_with_extension(root, ext) && !info.technologies.contains(tech) {
                info.technologies.push(tech.clone());
            }
        } else if root.join(file).exists() && !info.technologies.contains(tech) {
            info.technologies.push(tech.clone());
        }
    }

    // Detect frameworks from package.json
    if root.join("package.json").exists() {
        if let Ok(content) = std::fs::read_to_string(root.join("package.json")) {
            detect_js_frameworks(&content, &mut info);
        }
    }

    // Detect Rust frameworks from Cargo.toml
    if root.join("Cargo.toml").exists() {
        if let Ok(content) = std::fs::read_to_string(root.join("Cargo.toml")) {
            detect_rust_frameworks(&content, &mut info);
        }
    }

    // Detect Python frameworks from requirements
    for req_file in &["requirements.txt", "pyproject.toml"] {
        if root.join(req_file).exists() {
            if let Ok(content) = std::fs::read_to_string(root.join(req_file)) {
                detect_python_frameworks(&content, &mut info);
            }
        }
    }

    // Detect Go frameworks from go.mod
    if root.join("go.mod").exists() {
        if let Ok(content) = std::fs::read_to_string(root.join("go.mod")) {
            detect_go_frameworks(&content, &mut info);
        }
    }

    // Flutter detection
    if root.join("pubspec.yaml").exists() {
        if let Ok(content) = std::fs::read_to_string(root.join("pubspec.yaml")) {
            if content.contains("flutter:") {
                info.frameworks.push(Framework::Flutter);
            }
        }
    }

    // Claude Code configuration detection
    info.has_claude_md = root.join("CLAUDE.md").exists();
    info.has_claude_config = root.join(".claude").exists();
    info.has_settings = root.join(".claude").join("settings.json").exists();
    info.has_skills = root.join(".claude").join("skills").exists();
    info.has_agents = root.join(".claude").join("agents").exists();
    info.has_mcp_config = root.join(".mcp.json").exists();

    info
}

fn has_file_with_extension(root: &Path, ext: &str) -> bool {
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(ext) {
                    return true;
                }
            }
        }
    }
    false
}

fn detect_js_frameworks(package_json: &str, info: &mut ProjectInfo) {
    let deps_check = |dep: &str| -> bool {
        package_json.contains(&format!("\"{dep}\""))
    };

    if deps_check("next") {
        info.frameworks.push(Framework::NextJs);
    }
    if deps_check("react") && !info.frameworks.contains(&Framework::NextJs) {
        info.frameworks.push(Framework::React);
    }
    if deps_check("vue") {
        info.frameworks.push(Framework::Vue);
    }
    if deps_check("@angular/core") {
        info.frameworks.push(Framework::Angular);
    }
    if deps_check("svelte") {
        info.frameworks.push(Framework::Svelte);
    }
    if deps_check("express") {
        info.frameworks.push(Framework::Express);
    }
    if deps_check("@nestjs/core") {
        info.frameworks.push(Framework::NestJs);
    }
    if deps_check("@tauri-apps") || deps_check("tauri") {
        info.frameworks.push(Framework::Tauri);
    }

    // If TypeScript config exists, upgrade JS to TS
    if deps_check("typescript") && !info.technologies.contains(&Technology::TypeScript) {
        info.technologies.push(Technology::TypeScript);
    }
}

fn detect_rust_frameworks(cargo_toml: &str, info: &mut ProjectInfo) {
    if cargo_toml.contains("actix") {
        info.frameworks.push(Framework::Actix);
    }
    if cargo_toml.contains("axum") {
        info.frameworks.push(Framework::Axum);
    }
    if cargo_toml.contains("tauri") {
        info.frameworks.push(Framework::Tauri);
    }
}

fn detect_python_frameworks(content: &str, info: &mut ProjectInfo) {
    if content.contains("django") || content.contains("Django") {
        info.frameworks.push(Framework::Django);
    }
    if content.contains("fastapi") || content.contains("FastAPI") {
        info.frameworks.push(Framework::FastApi);
    }
    if content.contains("flask") || content.contains("Flask") {
        info.frameworks.push(Framework::Flask);
    }
}

fn detect_go_frameworks(go_mod: &str, info: &mut ProjectInfo) {
    if go_mod.contains("gin-gonic") {
        info.frameworks.push(Framework::Gin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_rust_project() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n[dependencies]\naxum = \"0.7\"\n",
        )
        .unwrap();

        let info = detect(tmp.path());
        assert!(info.technologies.contains(&Technology::Rust));
        assert!(info.frameworks.contains(&Framework::Axum));
    }

    #[test]
    fn test_detect_node_react_project() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"dependencies": {"react": "^18", "typescript": "^5"}}"#,
        )
        .unwrap();
        fs::write(tmp.path().join("tsconfig.json"), "{}").unwrap();

        let info = detect(tmp.path());
        assert!(info.technologies.contains(&Technology::TypeScript));
        assert!(info.frameworks.contains(&Framework::React));
    }

    #[test]
    fn test_detect_nextjs_project() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"dependencies": {"next": "^14", "react": "^18"}}"#,
        )
        .unwrap();

        let info = detect(tmp.path());
        assert!(info.frameworks.contains(&Framework::NextJs));
        // Should NOT also add React since Next includes it
        assert!(!info.frameworks.contains(&Framework::React));
    }

    #[test]
    fn test_detect_python_django() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("requirements.txt"), "django==4.2\ncelery\n").unwrap();

        let info = detect(tmp.path());
        assert!(info.technologies.contains(&Technology::Python));
        assert!(info.frameworks.contains(&Framework::Django));
    }

    #[test]
    fn test_detect_claude_config() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".claude").join("skills")).unwrap();
        fs::write(tmp.path().join("CLAUDE.md"), "# My instructions").unwrap();
        fs::write(tmp.path().join(".claude").join("settings.json"), "{}").unwrap();

        let info = detect(tmp.path());
        assert!(info.has_claude_md);
        assert!(info.has_claude_config);
        assert!(info.has_settings);
        assert!(info.has_skills);
    }

    #[test]
    fn test_detect_empty_directory() {
        let tmp = TempDir::new().unwrap();
        let info = detect(tmp.path());
        assert!(info.technologies.is_empty());
        assert!(info.frameworks.is_empty());
        assert!(!info.has_claude_config);
    }

    #[test]
    fn test_detect_go_project() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("go.mod"),
            "module example.com/app\nrequire github.com/gin-gonic/gin v1.9\n",
        )
        .unwrap();

        let info = detect(tmp.path());
        assert!(info.technologies.contains(&Technology::Go));
        assert!(info.frameworks.contains(&Framework::Gin));
    }
}
