use crate::components::{self, OptimizerAction, OptimizerStatus};
use crate::config::catalog::{self, SkillCatalog, SkillCategory};
use crate::config::paths::ClaudePaths;
use crate::config::types::{ComponentId, ConfigScope, ModelPreset, PresetId, Selection};
use crate::pipeline::stages::ExecutionResult;

/// Which screen is currently active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    ModelPresetSelect,
    PresetSelect,
    ComponentSelect,
    SkillSelect,
    OptimizerSelect,
    ScopeSelect,
    Confirm,
    Progress,
    Result,
}

/// Action returned by a screen's event handler.
pub enum ScreenAction {
    Continue,
    Next,
    Back,
    Quit,
    Execute,
}

/// Full TUI application state.
pub struct AppState {
    pub screen: Screen,
    pub preset: PresetId,
    pub components: Vec<(ComponentId, bool)>,
    pub optimizers: Vec<(ComponentId, OptimizerAction, OptimizerStatus)>,
    pub skills: Vec<(String, String, SkillCategory, bool, bool)>, // (id, name, category, selected, installed_in_project)
    pub model_preset: ModelPreset,
    pub scope: ConfigScope,
    pub cursor: usize,
    pub catalog: SkillCatalog,
    pub execution_result: Option<ExecutionResult>,
    pub progress_messages: Vec<String>,
    pub should_quit: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let catalog = catalog::default_catalog();
        let cwd = std::env::current_dir().unwrap_or_default();
        let skills: Vec<_> = catalog
            .skills
            .iter()
            .map(|s| {
                // Detect if skill is already installed in project
                let project_skill = cwd
                    .join(".claude")
                    .join("skills")
                    .join(&s.id)
                    .join("SKILL.md");
                let installed = project_skill.exists();
                (
                    s.id.clone(),
                    s.name.clone(),
                    s.category,
                    installed,
                    installed,
                )
            })
            .collect();

        Self {
            screen: Screen::Welcome,
            preset: PresetId::Recommended,
            components: ComponentId::ALL
                .iter()
                .map(|c| {
                    let default_on = PresetId::Recommended.components().contains(c);
                    (*c, default_on)
                })
                .collect(),
            optimizers: {
                let paths = ClaudePaths::detect()
                    .unwrap_or_else(|_| ClaudePaths::with_home(std::path::PathBuf::from(".")));
                ComponentId::OPTIMIZERS
                    .iter()
                    .map(|c| {
                        let status = components::detect_optimizer(*c, &paths);
                        let action = if status.is_active() {
                            OptimizerAction::Skip // already in project, don't reinstall
                        } else {
                            OptimizerAction::InstallProject // suggest installing
                        };
                        (*c, action, status)
                    })
                    .collect()
            },
            skills,
            model_preset: ModelPreset::Balanced,
            scope: ConfigScope::User,
            cursor: 0,
            catalog,
            execution_result: None,
            progress_messages: Vec::new(),
            should_quit: false,
        }
    }

    /// Build a Selection from the current TUI state.
    pub fn build_selection(&self) -> Selection {
        let mut components: Vec<ComponentId> = self
            .components
            .iter()
            .filter(|(_, selected)| *selected)
            .map(|(id, _)| *id)
            .collect();

        // Add optimizers that are set to install
        for (id, action, _) in &self.optimizers {
            if *action == OptimizerAction::InstallProject {
                components.push(*id);
            }
        }

        let skills: Vec<String> = self
            .skills
            .iter()
            .filter(|(_, _, _, selected, _)| *selected)
            .map(|(id, _, _, _, _)| id.clone())
            .collect();

        Selection {
            components,
            skills,
            preset: self.preset,
            model_preset: self.model_preset,
            scope: self.scope,
            dry_run: false,
        }
    }

    /// Apply preset selection to components.
    pub fn apply_preset(&mut self) {
        let preset_components = self.preset.components();
        for (id, selected) in &mut self.components {
            *selected = preset_components.contains(id);
        }
    }

    /// Refresh optimizer detection based on the current scope.
    pub fn refresh_optimizer_detection(&mut self) {
        let paths = ClaudePaths::detect()
            .unwrap_or_else(|_| ClaudePaths::with_home(std::path::PathBuf::from(".")));
        for (id, action, status) in &mut self.optimizers {
            *status = components::detect_optimizer(*id, &paths);
            *action = if status.is_active() {
                OptimizerAction::Skip
            } else {
                OptimizerAction::InstallProject
            };
        }
    }

    /// Cycle optimizer action right (→).
    pub fn cycle_optimizer_right(&mut self) {
        if let Some((_, action, status)) = self.optimizers.get_mut(self.cursor) {
            *action = action.next(*status);
        }
    }

    /// Cycle optimizer action left (←).
    pub fn cycle_optimizer_left(&mut self) {
        if let Some((_, action, status)) = self.optimizers.get_mut(self.cursor) {
            *action = action.prev(*status);
        }
    }

    /// Navigate to next screen.
    pub fn next_screen(&mut self) {
        self.cursor = 0;
        self.screen = match self.screen {
            Screen::Welcome => Screen::ModelPresetSelect,
            Screen::ModelPresetSelect => Screen::ScopeSelect,
            Screen::ScopeSelect => Screen::PresetSelect,
            Screen::PresetSelect => {
                if self.preset == PresetId::Custom {
                    Screen::ComponentSelect
                } else {
                    self.apply_preset();
                    Screen::SkillSelect
                }
            }
            Screen::ComponentSelect => Screen::SkillSelect,
            Screen::SkillSelect => {
                // Refresh detection now that we know the scope
                self.refresh_optimizer_detection();
                Screen::OptimizerSelect
            }
            Screen::OptimizerSelect => Screen::Confirm,
            Screen::Confirm => Screen::Progress,
            Screen::Progress => Screen::Result,
            Screen::Result => Screen::Result,
        };
    }

    /// Navigate to previous screen.
    pub fn prev_screen(&mut self) {
        self.cursor = 0;
        self.screen = match self.screen {
            Screen::Welcome => Screen::Welcome,
            Screen::ModelPresetSelect => Screen::Welcome,
            Screen::ScopeSelect => Screen::ModelPresetSelect,
            Screen::PresetSelect => Screen::ScopeSelect,
            Screen::ComponentSelect => Screen::PresetSelect,
            Screen::SkillSelect => {
                if self.preset == PresetId::Custom {
                    Screen::ComponentSelect
                } else {
                    Screen::PresetSelect
                }
            }
            Screen::OptimizerSelect => Screen::SkillSelect,
            Screen::Confirm => Screen::OptimizerSelect,
            Screen::Progress => Screen::Confirm,
            Screen::Result => Screen::Result,
        };
    }

    /// Toggle item at cursor for multi-select screens.
    pub fn toggle_current(&mut self) {
        match self.screen {
            Screen::ComponentSelect => {
                if let Some((_, selected)) = self.components.get_mut(self.cursor) {
                    *selected = !*selected;
                }
            }
            Screen::SkillSelect => {
                if let Some((_, _, _, selected, _)) = self.skills.get_mut(self.cursor) {
                    *selected = !*selected;
                }
            }
            Screen::OptimizerSelect => {
                // Handled by cycle_optimizer_right/left instead
            }
            _ => {}
        }
    }

    /// Get current list length for cursor bounds.
    pub fn list_len(&self) -> usize {
        match self.screen {
            Screen::ModelPresetSelect => 3,
            Screen::PresetSelect => 4,
            Screen::ComponentSelect => self.components.len(),
            Screen::SkillSelect => self.skills.len(),
            Screen::OptimizerSelect => self.optimizers.len(),
            Screen::ScopeSelect => 2,
            _ => 0,
        }
    }

    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn cursor_down(&mut self) {
        let len = self.list_len();
        if len > 0 && self.cursor < len - 1 {
            self.cursor += 1;
        }
    }
}
