use crate::components::{Component, PipelineContext};
use crate::pipeline::stages::*;

/// Callback type for progress reporting.
pub type ProgressCallback = Box<dyn Fn(ProgressEvent) + Send>;

/// Executes a list of components through a given stage.
pub struct StageRunner {
    progress: Option<ProgressCallback>,
}

impl Default for StageRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl StageRunner {
    pub fn new() -> Self {
        Self { progress: None }
    }

    pub fn with_progress(mut self, callback: ProgressCallback) -> Self {
        self.progress = Some(callback);
        self
    }

    fn emit(&self, event: ProgressEvent) {
        if let Some(cb) = &self.progress {
            cb(event);
        }
    }

    /// Run the Prepare stage for all components.
    /// Stops at the first failure.
    pub fn run_prepare(
        &self,
        components: &[Box<dyn Component>],
        ctx: &PipelineContext,
    ) -> StageResult {
        let mut steps = Vec::new();

        for component in components {
            self.emit(ProgressEvent {
                component_id: component.id().to_string(),
                component_name: component.name().to_string(),
                stage: Stage::Prepare,
                status: StepStatus::Running,
                error: None,
            });

            let timer = StepTimer::start();
            let result = component.prepare(ctx);

            let (status, error) = match result {
                Ok(()) => (StepStatus::Succeeded, None),
                Err(e) => (StepStatus::Failed, Some(e.to_string())),
            };

            self.emit(ProgressEvent {
                component_id: component.id().to_string(),
                component_name: component.name().to_string(),
                stage: Stage::Prepare,
                status,
                error: error.clone(),
            });

            steps.push(StepResult {
                component_id: component.id().to_string(),
                stage: Stage::Prepare,
                status,
                duration_ms: timer.elapsed_ms(),
                error,
                files_written: vec![],
            });

            // Stop on first failure
            if status == StepStatus::Failed {
                break;
            }
        }

        StageResult {
            stage: Stage::Prepare,
            steps,
        }
    }

    /// Run the Apply stage for all components.
    /// Stops at the first failure, returning the index of the failed component.
    pub fn run_apply(
        &self,
        components: &[Box<dyn Component>],
        ctx: &PipelineContext,
    ) -> (StageResult, Option<usize>) {
        let mut steps = Vec::new();
        let mut failed_index = None;

        for (i, component) in components.iter().enumerate() {
            self.emit(ProgressEvent {
                component_id: component.id().to_string(),
                component_name: component.name().to_string(),
                stage: Stage::Apply,
                status: StepStatus::Running,
                error: None,
            });

            let timer = StepTimer::start();

            if ctx.dry_run {
                let paths = component.affected_paths(&ctx.paths);
                self.emit(ProgressEvent {
                    component_id: component.id().to_string(),
                    component_name: component.name().to_string(),
                    stage: Stage::Apply,
                    status: StepStatus::Skipped,
                    error: None,
                });

                steps.push(StepResult {
                    component_id: component.id().to_string(),
                    stage: Stage::Apply,
                    status: StepStatus::Skipped,
                    duration_ms: timer.elapsed_ms(),
                    error: None,
                    files_written: paths,
                });
                continue;
            }

            let result = component.apply(ctx);

            let (status, error, files) = match result {
                Ok(apply_result) => (StepStatus::Succeeded, None, apply_result.files_written),
                Err(e) => (StepStatus::Failed, Some(e.to_string()), vec![]),
            };

            self.emit(ProgressEvent {
                component_id: component.id().to_string(),
                component_name: component.name().to_string(),
                stage: Stage::Apply,
                status,
                error: error.clone(),
            });

            steps.push(StepResult {
                component_id: component.id().to_string(),
                stage: Stage::Apply,
                status,
                duration_ms: timer.elapsed_ms(),
                error,
                files_written: files,
            });

            if status == StepStatus::Failed {
                failed_index = Some(i);
                break;
            }
        }

        (
            StageResult {
                stage: Stage::Apply,
                steps,
            },
            failed_index,
        )
    }

    /// Run rollback for components that were already applied (in reverse order).
    pub fn run_rollback(
        &self,
        components: &[Box<dyn Component>],
        up_to_index: usize,
        ctx: &PipelineContext,
    ) -> StageResult {
        let mut steps = Vec::new();

        // Rollback in reverse order, up to (but not including) the failed component
        for i in (0..up_to_index).rev() {
            let component = &components[i];

            self.emit(ProgressEvent {
                component_id: component.id().to_string(),
                component_name: component.name().to_string(),
                stage: Stage::Rollback,
                status: StepStatus::Running,
                error: None,
            });

            let timer = StepTimer::start();
            let result = component.rollback(ctx);

            let (status, error) = match result {
                Ok(()) => (StepStatus::RolledBack, None),
                Err(e) => (StepStatus::Failed, Some(e.to_string())),
            };

            self.emit(ProgressEvent {
                component_id: component.id().to_string(),
                component_name: component.name().to_string(),
                stage: Stage::Rollback,
                status,
                error: error.clone(),
            });

            steps.push(StepResult {
                component_id: component.id().to_string(),
                stage: Stage::Rollback,
                status,
                duration_ms: timer.elapsed_ms(),
                error,
                files_written: vec![],
            });
        }

        StageResult {
            stage: Stage::Rollback,
            steps,
        }
    }
}
