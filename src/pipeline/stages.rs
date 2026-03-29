use std::fmt;
use std::time::Instant;

/// Execution stage in the pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Prepare,
    Apply,
    Rollback,
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stage::Prepare => write!(f, "Prepare"),
            Stage::Apply => write!(f, "Apply"),
            Stage::Rollback => write!(f, "Rollback"),
        }
    }
}

/// Status of an individual step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Skipped,
    RolledBack,
}

impl fmt::Display for StepStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StepStatus::Pending => write!(f, "Pending"),
            StepStatus::Running => write!(f, "Running"),
            StepStatus::Succeeded => write!(f, "OK"),
            StepStatus::Failed => write!(f, "FAILED"),
            StepStatus::Skipped => write!(f, "Skipped"),
            StepStatus::RolledBack => write!(f, "Rolled back"),
        }
    }
}

/// Result of executing a single pipeline step.
#[derive(Debug)]
pub struct StepResult {
    pub component_id: String,
    pub stage: Stage,
    pub status: StepStatus,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub files_written: Vec<std::path::PathBuf>,
}

/// Result of executing an entire stage (Prepare, Apply, or Rollback).
#[derive(Debug)]
pub struct StageResult {
    pub stage: Stage,
    pub steps: Vec<StepResult>,
}

impl StageResult {
    pub fn success(&self) -> bool {
        self.steps.iter().all(|s| {
            matches!(
                s.status,
                StepStatus::Succeeded | StepStatus::Skipped | StepStatus::RolledBack
            )
        })
    }

    pub fn failed_step(&self) -> Option<&StepResult> {
        self.steps.iter().find(|s| s.status == StepStatus::Failed)
    }
}

/// Final result of the entire pipeline execution.
#[derive(Debug)]
pub struct ExecutionResult {
    pub prepare: StageResult,
    pub apply: StageResult,
    pub rollback: Option<StageResult>,
}

impl ExecutionResult {
    pub fn success(&self) -> bool {
        self.prepare.success() && self.apply.success()
    }

    pub fn total_files_written(&self) -> Vec<&std::path::Path> {
        self.apply
            .steps
            .iter()
            .flat_map(|s| s.files_written.iter().map(|p| p.as_path()))
            .collect()
    }
}

/// Progress event emitted during pipeline execution (for TUI integration).
#[derive(Debug, Clone)]
pub struct ProgressEvent {
    pub component_id: String,
    pub component_name: String,
    pub stage: Stage,
    pub status: StepStatus,
    pub error: Option<String>,
}

/// A timer utility for measuring step duration.
pub struct StepTimer {
    start: Instant,
}

impl StepTimer {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}
