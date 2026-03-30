pub mod app_state;
pub mod screens;
pub mod styles;
pub mod widgets;

use std::io;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;

use crate::components;
use crate::components::PipelineContext;
use crate::config::paths::ClaudePaths;
use crate::pipeline::orchestrator::PipelineOrchestrator;
use crate::pipeline::planner;
use crate::tui::app_state::{AppState, Screen, ScreenAction};

/// Run the interactive TUI.
pub fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut state = AppState::new();
    let result = run_loop(&mut terminal, &mut state);

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AppState,
) -> Result<()> {
    loop {
        // Render current screen
        terminal.draw(|frame| render(frame, state))?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            let action = handle_key(state, key.code);

            match action {
                ScreenAction::Continue => {}
                ScreenAction::Next => state.next_screen(),
                ScreenAction::Back => state.prev_screen(),
                ScreenAction::Quit => break,
                ScreenAction::Execute => {
                    state.next_screen(); // go to Progress screen
                    terminal.draw(|frame| render(frame, state))?;
                    execute_pipeline(state);
                    state.next_screen(); // go to Result screen
                }
            }

            if state.should_quit {
                break;
            }
        }
    }

    Ok(())
}

fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    match state.screen {
        Screen::Welcome => screens::welcome::render(frame, area),
        Screen::ModelPresetSelect => screens::model_preset_select::render(frame, area, state),
        Screen::PresetSelect => screens::preset_select::render(frame, area, state),
        Screen::ComponentSelect => screens::component_select::render(frame, area, state),
        Screen::SkillSelect => screens::skill_select::render(frame, area, state),
        Screen::OptimizerSelect => screens::optimizer_select::render(frame, area, state),
        Screen::ScopeSelect => screens::scope_select::render(frame, area, state),
        Screen::Confirm => screens::confirm::render(frame, area, state),
        Screen::Progress => screens::progress::render(frame, area, state),
        Screen::Result => screens::result::render(frame, area, state),
    }
}

fn handle_key(state: &mut AppState, key: KeyCode) -> ScreenAction {
    // Global keys
    if let KeyCode::Char('q') = key {
        return ScreenAction::Quit;
    }

    // Screen-specific keys
    match state.screen {
        Screen::Welcome => match key {
            KeyCode::Enter => ScreenAction::Next,
            _ => ScreenAction::Continue,
        },

        Screen::ModelPresetSelect => match key {
            KeyCode::Up => {
                state.cursor_up();
                state.model_preset = screens::model_preset_select::preset_at(state.cursor);
                ScreenAction::Continue
            }
            KeyCode::Down => {
                state.cursor_down();
                state.model_preset = screens::model_preset_select::preset_at(state.cursor);
                ScreenAction::Continue
            }
            KeyCode::Enter => ScreenAction::Next,
            KeyCode::Esc => ScreenAction::Back,
            _ => ScreenAction::Continue,
        },

        Screen::PresetSelect => match key {
            KeyCode::Up => {
                state.cursor_up();
                state.preset = screens::preset_select::preset_at(state.cursor);
                ScreenAction::Continue
            }
            KeyCode::Down => {
                state.cursor_down();
                state.preset = screens::preset_select::preset_at(state.cursor);
                ScreenAction::Continue
            }
            KeyCode::Enter => ScreenAction::Next,
            KeyCode::Esc => ScreenAction::Back,
            _ => ScreenAction::Continue,
        },

        Screen::ComponentSelect => match key {
            KeyCode::Up => {
                state.cursor_up();
                ScreenAction::Continue
            }
            KeyCode::Down => {
                state.cursor_down();
                ScreenAction::Continue
            }
            KeyCode::Char(' ') => {
                state.toggle_current();
                ScreenAction::Continue
            }
            KeyCode::Enter => ScreenAction::Next,
            KeyCode::Esc => ScreenAction::Back,
            _ => ScreenAction::Continue,
        },

        Screen::SkillSelect => match key {
            KeyCode::Up => {
                state.cursor_up();
                ScreenAction::Continue
            }
            KeyCode::Down => {
                state.cursor_down();
                ScreenAction::Continue
            }
            KeyCode::Char(' ') => {
                state.toggle_current();
                ScreenAction::Continue
            }
            KeyCode::Char('a') => {
                let all_selected = state.skills.iter().all(|(_, _, _, s, _)| *s);
                for (_, _, _, selected, _) in &mut state.skills {
                    *selected = !all_selected;
                }
                ScreenAction::Continue
            }
            KeyCode::Enter => ScreenAction::Next,
            KeyCode::Esc => ScreenAction::Back,
            _ => ScreenAction::Continue,
        },

        Screen::OptimizerSelect => match key {
            KeyCode::Up => {
                state.cursor_up();
                ScreenAction::Continue
            }
            KeyCode::Down => {
                state.cursor_down();
                ScreenAction::Continue
            }
            KeyCode::Right | KeyCode::Char(' ') => {
                state.cycle_optimizer_right();
                ScreenAction::Continue
            }
            KeyCode::Left => {
                state.cycle_optimizer_left();
                ScreenAction::Continue
            }
            KeyCode::Enter => ScreenAction::Next,
            KeyCode::Esc => ScreenAction::Back,
            _ => ScreenAction::Continue,
        },

        Screen::ScopeSelect => match key {
            KeyCode::Up => {
                state.cursor_up();
                state.scope = screens::scope_select::scope_at(state.cursor);
                ScreenAction::Continue
            }
            KeyCode::Down => {
                state.cursor_down();
                state.scope = screens::scope_select::scope_at(state.cursor);
                ScreenAction::Continue
            }
            KeyCode::Enter => ScreenAction::Next,
            KeyCode::Esc => ScreenAction::Back,
            _ => ScreenAction::Continue,
        },

        Screen::Confirm => match key {
            KeyCode::Enter => ScreenAction::Execute,
            KeyCode::Esc => ScreenAction::Back,
            _ => ScreenAction::Continue,
        },

        Screen::Progress => ScreenAction::Continue,

        Screen::Result => match key {
            KeyCode::Enter | KeyCode::Char('q') => ScreenAction::Quit,
            _ => ScreenAction::Continue,
        },
    }
}

fn execute_pipeline(state: &mut AppState) {
    state
        .progress_messages
        .push("Building execution plan...".to_string());

    let selection = state.build_selection();

    // Expand dependencies and build plan
    let expanded = planner::expand_dependencies(&selection.components);
    let plan_selection = crate::config::types::Selection {
        components: expanded,
        ..selection
    };
    let plan = planner::build_plan(&plan_selection);

    state.progress_messages.push(format!(
        "Installing {} components...",
        plan.ordered_components.len()
    ));

    // Create component instances
    let component_instances = components::create_components(&plan.ordered_components);

    // Build pipeline context
    let paths = match ClaudePaths::detect() {
        Ok(p) => p,
        Err(e) => {
            state.progress_messages.push(format!("Error: {e}"));
            return;
        }
    };

    let ctx = PipelineContext {
        paths,
        selection: plan_selection,
        dry_run: false,
        backup_manifest: None,
    };

    // Handle skill uninstalls (unchecked but were installed in project)
    let cwd = std::env::current_dir().unwrap_or_default();
    for (id, _name, _cat, selected, was_installed) in &state.skills {
        if !selected && *was_installed {
            let skill_dir = cwd.join(".claude").join("skills").join(id);
            if skill_dir.exists() {
                match std::fs::remove_dir_all(&skill_dir) {
                    Ok(()) => state
                        .progress_messages
                        .push(format!("[Skill] {} eliminado del proyecto", id)),
                    Err(e) => state
                        .progress_messages
                        .push(format!("[Skill] Error borrando {}: {e}", id)),
                }
            }
        }
    }

    // Handle optimizer uninstalls
    for (id, action, _status) in &state.optimizers {
        if *action == components::OptimizerAction::UninstallProject {
            match id {
                crate::config::types::ComponentId::Rtk => {
                    let rtk = components::rtk::RtkComponent;
                    match rtk.uninstall(&ctx) {
                        Ok(r) => {
                            for msg in &r.messages {
                                state.progress_messages.push(format!("[Uninstall] {msg}"));
                            }
                        }
                        Err(e) => state
                            .progress_messages
                            .push(format!("[Uninstall] RTK error: {e}")),
                    }
                }
                crate::config::types::ComponentId::CodeReviewGraph => {
                    let crg = components::code_review_graph::CodeReviewGraphComponent;
                    match crg.uninstall(&ctx) {
                        Ok(r) => {
                            for msg in &r.messages {
                                state.progress_messages.push(format!("[Uninstall] {msg}"));
                            }
                        }
                        Err(e) => state
                            .progress_messages
                            .push(format!("[Uninstall] CRG error: {e}")),
                    }
                }
                crate::config::types::ComponentId::ContextIsolation => {
                    let ci = components::context_isolation::ContextIsolationComponent;
                    match ci.uninstall(&ctx) {
                        Ok(r) => {
                            for msg in &r.messages {
                                state.progress_messages.push(format!("[Uninstall] {msg}"));
                            }
                        }
                        Err(e) => state
                            .progress_messages
                            .push(format!("[Uninstall] Context Isolation error: {e}")),
                    }
                }
                _ => {}
            }
        }
    }

    // Execute
    let orch = PipelineOrchestrator::new(component_instances);
    match orch.execute(&ctx) {
        Ok(result) => {
            for step in &result.prepare.steps {
                state
                    .progress_messages
                    .push(format!("[Prepare] {} - {}", step.component_id, step.status));
            }
            for step in &result.apply.steps {
                state
                    .progress_messages
                    .push(format!("[Apply] {} - {}", step.component_id, step.status));
            }

            if result.success() {
                let count = result.total_files_written().len();
                state
                    .progress_messages
                    .push(format!("Installation complete! {count} files written."));
            } else {
                state
                    .progress_messages
                    .push("Installation failed.".to_string());
            }

            state.execution_result = Some(result);
        }
        Err(e) => {
            state.progress_messages.push(format!("Pipeline error: {e}"));
        }
    }
}
