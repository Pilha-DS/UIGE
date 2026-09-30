//! UIGE core library.
//!
//! Domain logic, manifests, process execution and persistence.
//! Must not depend on Slint or UI details.

mod db;
pub mod execution;
pub mod history;
mod ids;
pub mod manifest;
pub mod paths;
pub mod profile;
pub mod store;
pub mod tool;
pub mod workflow;
pub mod workspace;

pub use db::{init_database, DatabaseError};
pub use execution::{
    build_argv, execute_command, ExecutionContext, ExecutionError, ExecutionOutcome,
    ExecutionStatus,
};
pub use history::{
    ExecutionRecord, ManifestVersionInfo, WorkflowExecutionRecord, WorkflowStepRecord,
};
pub use manifest::{
    bundled_demo_manifest, bundled_git_manifest, bundled_manifests, discover_manifests,
    find_command, load_manifest, parse_manifest, Command, Manifest, ManifestCandidate,
    ManifestError, Parameter, ParameterType,
};
pub use paths::list_subdirectories;
pub use profile::Profile;
pub use store::{ImportOutcome, Store, StoreError};
pub use tool::{Tool, ToolLibrary};
pub use workflow::{
    execute_workflow, StepFailurePolicy, StepStatus, Workflow, WorkflowError, WorkflowOutcome,
    WorkflowStep, WorkflowStepResult,
};
pub use workspace::{Workspace, DEFAULT_WORKSPACE_ID};
/// Trivial core response retained for smoke checks.
pub fn greeting() -> String {
    "Core respondeu: olá do UIGE.".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greeting_is_non_empty() {
        assert!(!greeting().is_empty());
    }
}
