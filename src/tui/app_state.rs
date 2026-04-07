use crate::components::{self, OptimizerAction, OptimizerStatus};
use crate::config::catalog::{self, SkillCatalog, SkillCategory};
use crate::config::paths::ClaudePaths;
use crate::config::types::{
    BehaviorProfileStatus, ComponentId, ConfigScope, ModelAlias, ModelPreset, PresetId, ProfileId,
    Selection, AGENT_PHASES,
};
use crate::pipeline::stages::ExecutionResult;

/// Which screen is currently active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    ModelPresetSelect,
    CustomModelSelect,
    PresetSelect,
    ComponentSelect,
    SkillSelect,
    OptimizerSelect,
    BehaviorProfileSelect,
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
    pub custom_models: Vec<ModelAlias>,
    pub progress_messages: Vec<String>,
    pub should_quit: bool,
    /// Selected behavioral profile (None = skip/ninguno).
    pub behavior_profile: Option<ProfileId>,
    /// Detected profile status at startup.
    pub profile_status: BehaviorProfileStatus,
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

        let paths = ClaudePaths::detect()
            .unwrap_or_else(|_| ClaudePaths::with_home(std::path::PathBuf::from(".")));

        let profile_status = components::detect_behavior_profile(&paths);
        let behavior_profile = match profile_status {
            BehaviorProfileStatus::Installed(p) => Some(p),
            _ => None,
        };

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
            custom_models: ModelPreset::Balanced.default_custom_models(),
            execution_result: None,
            progress_messages: Vec::new(),
            should_quit: false,
            behavior_profile,
            profile_status,
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

        let custom_models = if self.model_preset == ModelPreset::Custom {
            Some(
                AGENT_PHASES
                    .iter()
                    .zip(self.custom_models.iter())
                    .map(|((id, _), model)| (id.to_string(), *model))
                    .collect(),
            )
        } else {
            None
        };

        // Add behavior profile component if a profile is selected
        if self.behavior_profile.is_some()
            && !components.contains(&ComponentId::BehaviorProfile)
        {
            components.push(ComponentId::BehaviorProfile);
        }

        Selection {
            components,
            skills,
            preset: self.preset,
            model_preset: self.model_preset,
            custom_models,
            scope: self.scope,
            dry_run: false,
            behavior_profile: self.behavior_profile,
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
            Screen::ModelPresetSelect => {
                if self.model_preset == ModelPreset::Custom {
                    Screen::CustomModelSelect
                } else {
                    Screen::ScopeSelect
                }
            }
            Screen::CustomModelSelect => Screen::ScopeSelect,
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
            Screen::OptimizerSelect => Screen::BehaviorProfileSelect,
            Screen::BehaviorProfileSelect => Screen::Confirm,
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
            Screen::CustomModelSelect => Screen::ModelPresetSelect,
            Screen::ScopeSelect => {
                if self.model_preset == ModelPreset::Custom {
                    Screen::CustomModelSelect
                } else {
                    Screen::ModelPresetSelect
                }
            }
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
            Screen::BehaviorProfileSelect => Screen::OptimizerSelect,
            Screen::Confirm => Screen::BehaviorProfileSelect,
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
            Screen::ModelPresetSelect => 4,
            Screen::CustomModelSelect => AGENT_PHASES.len(),
            Screen::PresetSelect => 4,
            Screen::ComponentSelect => self.components.len(),
            Screen::SkillSelect => self.skills.len(),
            Screen::OptimizerSelect => self.optimizers.len(),
            Screen::BehaviorProfileSelect => ProfileId::ALL.len() + 1, // profiles + "Ninguno"
            Screen::ScopeSelect => 2,
            _ => 0,
        }
    }

    /// Cycle custom model right for the current phase.
    pub fn cycle_custom_model_right(&mut self) {
        if let Some(model) = self.custom_models.get_mut(self.cursor) {
            *model = model.next();
        }
    }

    /// Cycle custom model left for the current phase.
    pub fn cycle_custom_model_left(&mut self) {
        if let Some(model) = self.custom_models.get_mut(self.cursor) {
            *model = model.prev();
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
