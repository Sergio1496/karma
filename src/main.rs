use anyhow::Result;
use clap::Parser;

use karma::cli::commands::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            // Launch interactive TUI
            karma::tui::run()?;
        }
        Some(Commands::Install(args)) => {
            cmd_install(args)?;
        }
        Some(Commands::Sync(args)) => {
            cmd_sync(args).await?;
        }
        Some(Commands::Restore(args)) => {
            cmd_restore(args)?;
        }
        Some(Commands::Analyze) => {
            cmd_analyze()?;
        }
        Some(Commands::Status) => {
            cmd_status()?;
        }
        Some(Commands::Update) => {
            println!("Checking for updates...");
            println!("Self-update will be available via `cargo install karma` or Homebrew.");
        }
        Some(Commands::Version) => {
            println!("karma v{}", env!("CARGO_PKG_VERSION"));
        }
        Some(Commands::HookGuard) => {
            cmd_hook_guard()?;
        }
    }

    Ok(())
}

fn cmd_install(args: karma::cli::commands::InstallArgs) -> Result<()> {
    use karma::components::{self, PipelineContext};
    use karma::config::paths::ClaudePaths;
    use karma::config::types::{PresetId, Selection};
    use karma::pipeline::orchestrator::PipelineOrchestrator;
    use karma::pipeline::planner;
    use karma::state::AppState;
    let preset = args.preset.unwrap_or(PresetId::Recommended);
    let mut selected_components = if args.component.is_empty() {
        preset.components()
    } else {
        args.component
    };

    // Auto-include Skills component if skills are selected
    if !args.skill.is_empty()
        && !selected_components.contains(&karma::config::types::ComponentId::Skills)
    {
        selected_components.push(karma::config::types::ComponentId::Skills);
    }

    // Expand implicit dependencies
    selected_components = planner::expand_dependencies(&selected_components);

    let selection = Selection {
        components: selected_components,
        skills: args.skill,
        preset,
        model_preset: args.model_preset,
        scope: args.scope,
        dry_run: args.dry_run,
    };

    // Build execution plan (ordered by dependencies)
    let plan = planner::build_plan(&selection);

    println!("karma v{}", env!("CARGO_PKG_VERSION"));
    println!();

    if args.dry_run {
        println!("[DRY RUN] Would install:");
    } else {
        println!("Installing:");
    }

    println!("  Preset: {}", selection.preset.display_name());
    println!(
        "  Models: {} ({})",
        selection.model_preset.display_name(),
        selection.model_preset.description()
    );
    println!("  Scope:  {:?}", selection.scope);
    println!("  Components (in order):");
    for c in &plan.ordered_components {
        println!("    - {} ({})", c.display_name(), c.description());
    }
    if !selection.skills.is_empty() {
        println!("  Skills:");
        for s in &selection.skills {
            println!("    - {s}");
        }
    }
    println!();

    // Create component instances
    let component_instances = components::create_components(&plan.ordered_components);

    // Build pipeline context
    let paths = ClaudePaths::detect()?;
    let ctx = PipelineContext {
        paths,
        selection,
        dry_run: args.dry_run,
        backup_manifest: None,
    };

    // Execute pipeline
    let orch = PipelineOrchestrator::new(component_instances);
    let result = orch.execute(&ctx)?;

    // Report results
    println!("--- Results ---");

    for step in &result.prepare.steps {
        println!("  [Prepare] {} - {}", step.component_id, step.status);
        if let Some(ref err) = step.error {
            println!("            Error: {err}");
        }
    }

    for step in &result.apply.steps {
        let files_note = if !step.files_written.is_empty() {
            format!(" ({} files)", step.files_written.len())
        } else {
            String::new()
        };
        println!(
            "  [Apply]   {} - {}{}",
            step.component_id, step.status, files_note
        );
        if let Some(ref err) = step.error {
            println!("            Error: {err}");
        }
    }

    if let Some(ref rollback) = result.rollback {
        println!();
        println!("  Rollback was triggered:");
        for step in &rollback.steps {
            println!("  [Rollback] {} - {}", step.component_id, step.status);
        }
    }

    println!();
    if result.success() {
        let written: Vec<_> = result.total_files_written();
        if args.dry_run {
            println!("Dry run complete. No files were modified.");
        } else {
            println!("Installation complete! {} files written.", written.len());
            for path in &written {
                println!("  {}", path.display());
            }

            // Persist state
            let mut state = AppState::load(&ctx.paths.state_file()).unwrap_or_default();
            state.record_install(&ctx.selection.components, &ctx.selection.skills);
            if let Err(e) = state.save(&ctx.paths.state_file()) {
                println!("  Warning: failed to save state: {e}");
            }
        }
    } else {
        println!("Installation failed. Check errors above.");
        if result.rollback.is_some() {
            println!("Changes have been rolled back.");
        }
    }

    Ok(())
}

fn cmd_status() -> Result<()> {
    use karma::backup::snapshot;
    use karma::config::paths::ClaudePaths;
    use karma::state::AppState;

    let paths = ClaudePaths::detect()?;

    println!("karma v{}", env!("CARGO_PKG_VERSION"));
    println!();

    // Load state
    let state = AppState::load(&paths.state_file()).unwrap_or_default();

    // Installed components
    if state.installed_components.is_empty() {
        println!("  No components installed yet.");
        println!("  Run `karma install` or launch TUI with `karma`.");
    } else {
        println!("  Installed components:");
        for c in &state.installed_components {
            println!("    - {}", c.display_name());
        }
    }

    // Installed skills
    if !state.installed_skills.is_empty() {
        println!();
        println!("  Installed skills:");
        for s in &state.installed_skills {
            println!("    - {s}");
        }
    }

    // Timestamps
    if let Some(ref ts) = state.last_install {
        println!();
        println!("  Last install: {ts}");
    }
    if let Some(ref ts) = state.last_sync {
        println!("  Last sync:    {ts}");
    }

    // Backups
    let backups = snapshot::list_snapshots(&paths.backup_dir()).unwrap_or_default();
    println!();
    println!("  Backups available: {}", backups.len());

    // Paths
    println!();
    println!("  Paths:");
    println!("    Config:  {}", paths.global_config_dir().display());
    println!("    State:   {}", paths.state_dir().display());
    println!("    Cache:   {}", paths.cache_dir().display());
    println!("    Backups: {}", paths.backup_dir().display());

    Ok(())
}

fn cmd_analyze() -> Result<()> {
    use karma::analyzer::suggester::Priority;
    use karma::analyzer::{detector, suggester};

    let cwd = std::env::current_dir()?;
    println!("karma analyze");
    println!("  Directory: {}", cwd.display());
    println!();

    let info = detector::detect(&cwd);

    // Tech stack
    if info.technologies.is_empty() {
        println!("  No technologies detected.");
    } else {
        println!("  Tech stack:");
        for tech in &info.technologies {
            println!("    - {}", tech.display_name());
        }
    }

    if !info.frameworks.is_empty() {
        println!("  Frameworks:");
        for fw in &info.frameworks {
            println!("    - {}", fw.display_name());
        }
    }

    // Claude Code status
    println!();
    println!("  Claude Code status:");
    println!(
        "    CLAUDE.md:     {}",
        if info.has_claude_md {
            "found"
        } else {
            "not found"
        }
    );
    println!(
        "    .claude/ dir:  {}",
        if info.has_claude_config {
            "found"
        } else {
            "not found"
        }
    );
    println!(
        "    settings.json: {}",
        if info.has_settings {
            "found"
        } else {
            "not found"
        }
    );
    println!(
        "    skills/:       {}",
        if info.has_skills {
            "found"
        } else {
            "not found"
        }
    );
    println!(
        "    agents/:       {}",
        if info.has_agents {
            "found"
        } else {
            "not found"
        }
    );
    println!(
        "    .mcp.json:     {}",
        if info.has_mcp_config {
            "found"
        } else {
            "not found"
        }
    );

    // Suggestions
    let analysis = suggester::suggest(&info);

    println!();
    println!("  Component suggestions:");
    for s in &analysis.component_suggestions {
        let badge = match s.priority {
            Priority::High => "[HIGH]",
            Priority::Medium => "[MED] ",
            Priority::Low => "[LOW] ",
        };
        println!(
            "    {} {} - {}",
            badge,
            s.component.display_name(),
            s.reason
        );
    }

    if !analysis.skill_suggestions.is_empty() {
        println!();
        println!("  Skill suggestions:");
        for s in &analysis.skill_suggestions {
            println!("    - {} ({})", s.skill_id, s.reason);
        }
    }

    if !analysis.warnings.is_empty() {
        println!();
        println!("  Warnings:");
        for w in &analysis.warnings {
            println!("    ! {w}");
        }
    }

    println!();
    println!("  Run `karma install --preset recommended` to install suggested components.");

    Ok(())
}

async fn cmd_sync(args: karma::cli::commands::SyncArgs) -> Result<()> {
    use karma::config::catalog;
    use karma::config::paths::ClaudePaths;
    use karma::config::remote_catalog;
    use karma::filemerge::writer;
    use karma::remote::cache::FileCache;
    use karma::remote::fetcher::SkillFetcher;
    use karma::state::AppState;

    let paths = ClaudePaths::detect()?;

    println!("karma sync");
    println!();

    // Try to fetch remote catalog; fall back to static
    println!("Fetching skill catalog...");
    let catalog = match remote_catalog::fetch_remote_catalog().await {
        Ok(remote) => {
            println!("  Found {} skills in remote catalog", remote.skills.len());
            remote
        }
        Err(e) => {
            println!("  Remote catalog unavailable: {e}");
            println!("  Using built-in catalog");
            catalog::default_catalog()
        }
    };

    // Load state to know what's installed
    let state = AppState::load(&paths.state_file()).unwrap_or_default();
    let skill_ids: Vec<String> = if state.installed_skills.is_empty() {
        // If no state, sync all catalog skills
        catalog
            .skill_ids()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        state.installed_skills.clone()
    };

    if skill_ids.is_empty() {
        println!("No skills to sync.");
        return Ok(());
    }

    // Fetch skills
    let cache = FileCache::new(paths.cache_dir());
    let fetcher = SkillFetcher::new(cache);

    println!("Downloading {} skills...", skill_ids.len());
    let results = fetcher
        .fetch_skills(&catalog, &skill_ids, args.refresh)
        .await;

    let mut synced = 0u32;
    let mut failed = 0u32;

    for result in &results {
        if let Some(ref content) = result.content {
            // Write to skills directory
            let skill_path = paths.skill_file(&result.skill_id);
            match writer::write_file_atomic_str(&skill_path, content) {
                Ok(write_result) => {
                    if write_result.changed {
                        println!("  Updated: {}", result.skill_id);
                        synced += 1;
                    } else {
                        println!("  Unchanged: {}", result.skill_id);
                    }
                }
                Err(e) => {
                    println!("  Error writing {}: {e}", result.skill_id);
                    failed += 1;
                }
            }
        } else {
            println!(
                "  Skipped: {} ({})",
                result.skill_id,
                result.error.as_deref().unwrap_or("unknown error")
            );
            failed += 1;
        }
    }

    println!();
    println!(
        "Sync complete: {} updated, {} failed, {} total",
        synced,
        failed,
        results.len()
    );

    Ok(())
}

fn cmd_hook_guard() -> Result<()> {
    use std::io::Read;

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let parsed: serde_json::Value = serde_json::from_str(&input).unwrap_or_default();
    let tool_name = parsed["tool_name"].as_str().unwrap_or("");
    let tool_input = &parsed["tool_input"];
    let session_id = parsed["session_id"].as_str().unwrap_or("");

    if let Some(output) =
        karma::components::context_isolation::evaluate_hook(tool_name, tool_input, session_id)
    {
        println!("{}", serde_json::to_string(&output)?);
    }
    // No output + exit 0 = allow the tool call

    Ok(())
}

fn cmd_restore(args: karma::cli::commands::RestoreArgs) -> Result<()> {
    use karma::backup::{restore, snapshot};
    use karma::config::paths::ClaudePaths;

    let paths = ClaudePaths::detect()?;
    let backup_dir = paths.backup_dir();

    let snapshots = snapshot::list_snapshots(&backup_dir)?;

    if snapshots.is_empty() {
        println!("No backups found.");
        return Ok(());
    }

    let manifest = if args.latest {
        snapshots.into_iter().next().unwrap()
    } else if let Some(id) = args.id {
        snapshots
            .into_iter()
            .find(|m| m.created_at == id)
            .ok_or_else(|| anyhow::anyhow!("Backup '{id}' not found"))?
    } else {
        println!("Available backups:");
        for (i, m) in snapshots.iter().enumerate() {
            println!("  {}. {} ({} files)", i + 1, m.created_at, m.entries.len());
        }
        println!("\nUse --latest or --id <timestamp> to restore.");
        return Ok(());
    };

    println!("Restoring backup from {}...", manifest.created_at);
    let result = restore::restore_from_manifest(&manifest)?;

    println!(
        "Restore complete: {} restored, {} deleted",
        result.restored, result.deleted
    );
    if !result.errors.is_empty() {
        println!("Errors:");
        for err in &result.errors {
            println!("  - {err}");
        }
    }

    Ok(())
}
