use anyhow::{Context, Result};

use crate::backup::snapshot;
use crate::components::{Component, PipelineContext};
use crate::pipeline::runner::{ProgressCallback, StageRunner};
use crate::pipeline::stages::*;

/// The main pipeline orchestrator.
///
/// Executes the full Prepare -> Backup -> Apply -> (Rollback on failure) flow.
pub struct PipelineOrchestrator {
    components: Vec<Box<dyn Component>>,
    progress: Option<ProgressCallback>,
}

impl PipelineOrchestrator {
    pub fn new(components: Vec<Box<dyn Component>>) -> Self {
        Self {
            components,
            progress: None,
        }
    }

    pub fn with_progress(mut self, callback: ProgressCallback) -> Self {
        self.progress = Some(callback);
        self
    }

    /// Execute the full pipeline.
    pub fn execute(&self, ctx: &PipelineContext) -> Result<ExecutionResult> {
        let runner = StageRunner::new();

        // Phase 1: Prepare - validate all components
        let prepare_result = runner.run_prepare(&self.components, ctx);
        if !prepare_result.success() {
            return Ok(ExecutionResult {
                prepare: prepare_result,
                apply: StageResult {
                    stage: Stage::Apply,
                    steps: vec![],
                },
                rollback: None,
            });
        }

        // Phase 2: Backup affected files (unless dry run)
        if !ctx.dry_run {
            self.create_backup(ctx)
                .context("Failed to create backup before applying changes")?;
        }

        // Phase 3: Apply all components
        let (apply_result, failed_index) = runner.run_apply(&self.components, ctx);

        // Phase 4: Rollback if apply failed
        let rollback_result = if let Some(failed_idx) = failed_index {
            if !ctx.dry_run && failed_idx > 0 {
                let rollback = runner.run_rollback(&self.components, failed_idx, ctx);
                Some(rollback)
            } else {
                None
            }
        } else {
            None
        };

        Ok(ExecutionResult {
            prepare: prepare_result,
            apply: apply_result,
            rollback: rollback_result,
        })
    }

    /// Create a backup of all files that will be affected.
    fn create_backup(&self, ctx: &PipelineContext) -> Result<()> {
        let mut all_paths = Vec::new();
        for component in &self.components {
            let paths = component.affected_paths(&ctx.paths);
            all_paths.extend(paths);
        }

        // Deduplicate paths
        all_paths.sort();
        all_paths.dedup();

        if all_paths.is_empty() {
            return Ok(());
        }

        let backup_dir = ctx.paths.backup_dir();
        snapshot::create_snapshot(&backup_dir, &all_paths)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ApplyResult;
    use crate::config::paths::ClaudePaths;
    use crate::config::types::Selection;
    use std::path::PathBuf;

    struct MockComponent {
        id: &'static str,
        prepare_ok: bool,
        apply_ok: bool,
    }

    impl Component for MockComponent {
        fn id(&self) -> &str {
            self.id
        }
        fn name(&self) -> &str {
            self.id
        }
        fn affected_paths(&self, _paths: &ClaudePaths) -> Vec<PathBuf> {
            vec![]
        }
        fn prepare(&self, _ctx: &PipelineContext) -> Result<()> {
            if self.prepare_ok {
                Ok(())
            } else {
                anyhow::bail!("prepare failed for {}", self.id)
            }
        }
        fn apply(&self, _ctx: &PipelineContext) -> Result<ApplyResult> {
            if self.apply_ok {
                Ok(ApplyResult {
                    changed: true,
                    files_written: vec![],
                    messages: vec![],
                })
            } else {
                anyhow::bail!("apply failed for {}", self.id)
            }
        }
        fn rollback(&self, _ctx: &PipelineContext) -> Result<()> {
            Ok(())
        }
    }

    fn make_ctx(dry_run: bool) -> PipelineContext {
        let tmp = tempfile::TempDir::new().unwrap();
        PipelineContext {
            paths: ClaudePaths::with_home(tmp.path().to_path_buf()),
            selection: Selection::default(),
            dry_run,
            backup_manifest: None,
        }
    }

    #[test]
    fn test_successful_pipeline() {
        let components: Vec<Box<dyn Component>> = vec![
            Box::new(MockComponent {
                id: "a",
                prepare_ok: true,
                apply_ok: true,
            }),
            Box::new(MockComponent {
                id: "b",
                prepare_ok: true,
                apply_ok: true,
            }),
        ];

        let orch = PipelineOrchestrator::new(components);
        let ctx = make_ctx(false);
        let result = orch.execute(&ctx).unwrap();

        assert!(result.success());
        assert!(result.prepare.success());
        assert!(result.apply.success());
        assert!(result.rollback.is_none());
        assert_eq!(result.apply.steps.len(), 2);
    }

    #[test]
    fn test_prepare_failure_skips_apply() {
        let components: Vec<Box<dyn Component>> = vec![
            Box::new(MockComponent {
                id: "a",
                prepare_ok: true,
                apply_ok: true,
            }),
            Box::new(MockComponent {
                id: "b",
                prepare_ok: false,
                apply_ok: true,
            }),
        ];

        let orch = PipelineOrchestrator::new(components);
        let ctx = make_ctx(false);
        let result = orch.execute(&ctx).unwrap();

        assert!(!result.success());
        assert!(!result.prepare.success());
        assert!(result.apply.steps.is_empty());
    }

    #[test]
    fn test_apply_failure_triggers_rollback() {
        let components: Vec<Box<dyn Component>> = vec![
            Box::new(MockComponent {
                id: "a",
                prepare_ok: true,
                apply_ok: true,
            }),
            Box::new(MockComponent {
                id: "b",
                prepare_ok: true,
                apply_ok: false,
            }),
        ];

        let orch = PipelineOrchestrator::new(components);
        let ctx = make_ctx(false);
        let result = orch.execute(&ctx).unwrap();

        assert!(!result.success());
        assert!(result.rollback.is_some());
        let rollback = result.rollback.unwrap();
        assert_eq!(rollback.steps.len(), 1); // Only "a" gets rolled back
        assert_eq!(rollback.steps[0].component_id, "a");
    }

    #[test]
    fn test_dry_run_skips_apply() {
        let components: Vec<Box<dyn Component>> = vec![Box::new(MockComponent {
            id: "a",
            prepare_ok: true,
            apply_ok: true,
        })];

        let orch = PipelineOrchestrator::new(components);
        let ctx = make_ctx(true);
        let result = orch.execute(&ctx).unwrap();

        assert!(result.success());
        assert_eq!(result.apply.steps[0].status, StepStatus::Skipped);
    }
}
