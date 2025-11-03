use thiserror::Error;

#[derive(Error, Debug)]
pub enum CausalError {
    /// Error when adding an edge would create a cycle in the DAG
    #[error("Adding edge '{0}' -> '{1}' would create a cycle.")]
    CycleDetected(String, String),

    /// Error if a node in the SCM is missing its functional assignment
    #[error("Node '{0}' not found in SCM assignments.")]
    NodeAssignmentMissing(String),

    /// Error if the graph is not a DAG and toposort fails
    #[error("Topological sort failed, graph is not a DAG.")]
    TopologicalSortFailed,

    /// Wrapper for Polars errors
    #[error(transparent)]
    Polars(#[from] polars::prelude::PolarsError),

    /// Wrapper for standard I/O errors
    #[error(transparent)]
    Io(#[from] std::io::Error),
}