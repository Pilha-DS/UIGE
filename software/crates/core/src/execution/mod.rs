//! Build argv and run processes with structured arguments.

mod argv;
mod runner;

pub use argv::{build_argv, ArgvError};
pub use runner::{
    execute_command, ExecutionContext, ExecutionError, ExecutionOutcome, ExecutionStatus,
};
