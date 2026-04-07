use rust_embed::Embed;

#[derive(Embed)]
#[folder = "assets/"]
pub struct Assets;

/// Get an embedded asset as a string.
pub fn get_asset(path: &str) -> Option<String> {
    Assets::get(path).map(|f| String::from_utf8_lossy(&f.data).to_string())
}

/// Get the SDD orchestrator template.
pub fn orchestrator_template() -> String {
    get_asset("templates/orchestrator.md")
        .expect("embedded asset templates/orchestrator.md missing")
}

/// Get the default permissions overlay JSON.
pub fn default_permissions() -> String {
    get_asset("defaults/permissions.json")
        .expect("embedded asset defaults/permissions.json missing")
}

/// Get the context isolation template.
pub fn context_isolation_template() -> String {
    get_asset("templates/context_isolation.md")
        .expect("embedded asset templates/context_isolation.md missing")
}

/// Get the default Context7 MCP server config.
pub fn default_mcp_context7() -> String {
    get_asset("defaults/mcp/context7.json")
        .expect("embedded asset defaults/mcp/context7.json missing")
}

/// Get a behavioral profile template by profile ID.
pub fn behavior_profile_template(profile: &crate::config::types::ProfileId) -> String {
    let path = profile.template_asset();
    get_asset(path).unwrap_or_else(|| panic!("embedded asset {path} missing"))
}
